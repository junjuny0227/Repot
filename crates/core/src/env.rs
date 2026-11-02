pub mod real;
pub use real::RealEnv;

use crate::os::Os;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct CommandOutput {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}
impl CommandOutput {
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EnvError {
    #[error("not found")]
    NotFound,
    #[error("permission denied")]
    PermissionDenied,
    #[error("timed out")]
    Timeout,
    #[error("I/O error: {0}")]
    Io(String),
}

pub trait Env: Send + Sync {
    fn os(&self) -> Os;
    fn home_dir(&self) -> PathBuf;
    fn var(&self, name: &str) -> Option<String>;
    fn which(&self, program: &str) -> bool;
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, EnvError>;
    fn read_file(&self, path: &Path) -> Result<String, EnvError>;
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, EnvError>;
    fn exists(&self, path: &Path) -> bool;
    fn temp_file(&self, name: &str) -> PathBuf;
    fn remove_file(&self, path: &Path);
    fn read_link(&self, path: &Path) -> Option<PathBuf>;
}

type CommandKey = (String, Vec<String>);
#[derive(Debug, Clone)]
pub struct FakeEnv {
    os: Os,
    home: PathBuf,
    vars: BTreeMap<String, String>,
    programs: BTreeSet<String>,
    commands: BTreeMap<CommandKey, Result<CommandOutput, EnvError>>,
    files: BTreeMap<PathBuf, Result<String, EnvError>>,
    links: BTreeMap<PathBuf, PathBuf>,
}
impl FakeEnv {
    pub fn new(os: Os, home: impl Into<PathBuf>) -> Self {
        Self {
            os,
            home: home.into(),
            vars: BTreeMap::new(),
            programs: BTreeSet::new(),
            commands: BTreeMap::new(),
            files: BTreeMap::new(),
            links: BTreeMap::new(),
        }
    }
    pub fn with_var(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(name.into(), value.into());
        self
    }
    pub fn with_program(mut self, name: &str) -> Self {
        self.programs.insert(name.into());
        self
    }
    pub fn with_command(self, program: &str, args: &[&str], stdout: &str) -> Self {
        self.with_command_output(
            program,
            args,
            Ok(CommandOutput {
                code: Some(0),
                stdout: stdout.into(),
                stderr: String::new(),
            }),
        )
    }
    pub fn with_command_output(
        mut self,
        program: &str,
        args: &[&str],
        out: Result<CommandOutput, EnvError>,
    ) -> Self {
        self.programs.insert(program.into());
        self.commands.insert(
            (program.into(), args.iter().map(|s| (*s).into()).collect()),
            out,
        );
        self
    }
    pub fn with_link(mut self, path: impl Into<PathBuf>, target: impl Into<PathBuf>) -> Self {
        self.links.insert(path.into(), target.into());
        self
    }
    pub fn with_file(mut self, path: impl Into<PathBuf>, content: &str) -> Self {
        self.files.insert(path.into(), Ok(content.into()));
        self
    }
    pub fn with_unreadable_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.files
            .insert(path.into(), Err(EnvError::PermissionDenied));
        self
    }
}
impl Env for FakeEnv {
    fn temp_file(&self, name: &str) -> PathBuf {
        // Built as a string so the path, and command arguments derived from it, are identical
        // on every OS; `join` would insert `\` on Windows and break fixtures that spell it out.
        PathBuf::from(format!("/tmp/repot-fake/{name}"))
    }
    fn remove_file(&self, _path: &Path) {}
    fn read_link(&self, path: &Path) -> Option<PathBuf> {
        self.links.get(path).cloned()
    }
    fn os(&self) -> Os {
        self.os
    }
    fn home_dir(&self) -> PathBuf {
        self.home.clone()
    }
    fn var(&self, name: &str) -> Option<String> {
        self.vars.get(name).cloned()
    }
    fn which(&self, program: &str) -> bool {
        self.programs.contains(program)
    }
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, EnvError> {
        self.commands
            .get(&(program.into(), args.iter().map(|s| (*s).into()).collect()))
            .cloned()
            .unwrap_or(Err(EnvError::NotFound))
    }
    fn read_file(&self, path: &Path) -> Result<String, EnvError> {
        self.files
            .get(path)
            .cloned()
            .unwrap_or(Err(EnvError::NotFound))
    }
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, EnvError> {
        let children: BTreeSet<_> = self
            .files
            .keys()
            .chain(self.links.keys())
            .filter_map(|file| {
                let rel = file.strip_prefix(path).ok()?;
                let first = rel.components().next()?;
                Some(path.join(first.as_os_str()))
            })
            .collect();
        if children.is_empty() {
            Err(EnvError::NotFound)
        } else {
            Ok(children.into_iter().collect())
        }
    }
    fn exists(&self, path: &Path) -> bool {
        self.files
            .keys()
            .chain(self.links.keys())
            .any(|file| file.starts_with(path))
    }
}
