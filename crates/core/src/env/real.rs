use super::{CommandOutput, Env, EnvError};
use crate::os::{Os, current_os};
use std::ffi::OsString;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

pub fn candidate_names(program: &str, pathext: Option<&str>) -> Vec<String> {
    match pathext {
        Some(exts) if Path::new(program).extension().is_none() => exts
            .split(';')
            .filter(|ext| !ext.is_empty())
            .map(|ext| format!("{program}{}", ext.to_ascii_lowercase()))
            .collect(),
        _ => vec![program.into()],
    }
}

#[derive(Debug, Clone)]
pub struct RealEnv {
    os: Os,
    home: PathBuf,
    path: OsString,
    pathext: Option<String>,
    timeout: Duration,
}
impl Default for RealEnv {
    fn default() -> Self {
        Self::new()
    }
}
impl RealEnv {
    pub fn new() -> Self {
        let os = current_os();
        let home = std::env::home_dir().unwrap_or_default();
        let mut path = std::env::var_os("PATH").unwrap_or_default();
        if os == Os::MacOs {
            let login_path = std::env::var_os("SHELL").and_then(|shell| {
                let mut cmd = Command::new(shell);
                cmd.args(["-ilc", "printf %s \"$PATH\""]);
                execute(cmd, Duration::from_secs(5))
                    .ok()
                    .filter(|out| out.success() && !out.stdout.is_empty())
                    .map(|out| OsString::from(out.stdout))
            });
            path = login_path.unwrap_or_else(|| {
                let mut dirs: Vec<_> = std::env::split_paths(&path).collect();
                dirs.extend([
                    PathBuf::from("/opt/homebrew/bin"),
                    PathBuf::from("/usr/local/bin"),
                    home.join(".cargo/bin"),
                ]);
                std::env::join_paths(dirs).unwrap_or(path)
            });
        }
        Self {
            os,
            home,
            path,
            pathext: if os == Os::Windows {
                Some(std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into()))
            } else {
                None
            },
            timeout: COMMAND_TIMEOUT,
        }
    }
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    fn resolve(&self, program: &str) -> Option<PathBuf> {
        let names = candidate_names(program, self.pathext.as_deref());
        let dirs: Vec<_> =
            if Path::new(program).components().count() > 1 || Path::new(program).is_absolute() {
                vec![PathBuf::new()]
            } else {
                std::env::split_paths(&self.path).collect()
            };
        dirs.iter()
            .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
            .find(|path| executable(path))
    }
}
impl Env for RealEnv {
    fn os(&self) -> Os {
        self.os
    }
    fn home_dir(&self) -> PathBuf {
        self.home.clone()
    }
    fn var(&self, name: &str) -> Option<String> {
        if name == "PATH" {
            Some(self.path.to_string_lossy().into_owned())
        } else {
            std::env::var(name).ok()
        }
    }
    fn which(&self, program: &str) -> bool {
        self.resolve(program).is_some()
    }
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, EnvError> {
        let path = self.resolve(program).ok_or(EnvError::NotFound)?;
        let mut cmd = Command::new(path);
        cmd.args(args).env("PATH", &self.path);
        execute(cmd, self.timeout)
    }
    fn read_file(&self, path: &Path) -> Result<String, EnvError> {
        std::fs::read(path)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .map_err(io_error)
    }
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, EnvError> {
        let mut entries = std::fs::read_dir(path)
            .map_err(io_error)?
            .map(|entry| entry.map(|entry| entry.path()).map_err(io_error))
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort();
        Ok(entries)
    }
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
fn executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}
fn io_error(err: io::Error) -> EnvError {
    match err.kind() {
        io::ErrorKind::NotFound => EnvError::NotFound,
        io::ErrorKind::PermissionDenied => EnvError::PermissionDenied,
        _ => EnvError::Io(err.to_string()),
    }
}
fn execute(mut command: Command, timeout: Duration) -> Result<CommandOutput, EnvError> {
    let start = Instant::now();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(io_error)?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let (tx, rx) = mpsc::channel();
    for (index, mut reader) in [
        (0, Box::new(stdout) as Box<dyn Read + Send>),
        (1, Box::new(stderr) as Box<dyn Read + Send>),
    ] {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = reader
                .read_to_end(&mut bytes)
                .map(|_| String::from_utf8_lossy(&bytes).into_owned())
                .map_err(io_error);
            let _ = tx.send((index, result));
        });
    }
    drop(tx);
    let status = match child.wait_timeout(timeout.saturating_sub(start.elapsed())) {
        Ok(Some(status)) => status,
        result => {
            let _ = child.kill();
            let _ = child.wait();
            return match result {
                Err(err) => Err(io_error(err)),
                _ => Err(EnvError::Timeout),
            };
        }
    };
    // Bound pipe draining too: descendants can inherit the handles after the child exits.
    let mut streams = [String::new(), String::new()];
    for _ in 0..2 {
        let (index, output) = rx
            .recv_timeout(timeout.saturating_sub(start.elapsed()))
            .map_err(|err| match err {
                mpsc::RecvTimeoutError::Timeout => EnvError::Timeout,
                mpsc::RecvTimeoutError::Disconnected => {
                    EnvError::Io("output reader disconnected".into())
                }
            })?;
        streams[index] = output?;
    }
    let [stdout, stderr] = streams;
    Ok(CommandOutput {
        code: status.code(),
        stdout,
        stderr,
    })
}
