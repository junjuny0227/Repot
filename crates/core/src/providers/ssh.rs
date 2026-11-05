//! SSH client config (plan 16): `~/.ssh/config` and the files it includes from `~/.ssh`.
//! Keys are never read: key paths are only taken from the config text.
use super::common::{ConfigFileSpec, config_file_item};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
    secrets,
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub struct Ssh;
const CATALOG_KEY: &str = "file:ssh-config";
/// OpenSSH's `READCONF_MAX_DEPTH`.
const MAX_DEPTH: usize = 16;
const MAX_FILES: usize = 32;
/// Keywords whose value is a path; an absolute or `~/Library/` path belongs to the source OS.
const PATH_KEYWORDS: [&str; 5] = [
    "IdentityFile",
    "CertificateFile",
    "IdentityAgent",
    "UserKnownHostsFile",
    "ControlPath",
];
/// Connection multiplexing, which Windows OpenSSH does not support.
const CONTROL_KEYWORDS: [&str; 3] = ["ControlMaster", "ControlPath", "ControlPersist"];
const KEY_KEYWORDS: [&str; 2] = ["IdentityFile", "CertificateFile"];

/// Whether a scan or an opened Repot file may write an SSH config to `rel` (home-relative,
/// `/`): a file under `.ssh/` that is not a key, `known_hosts`, `authorized_keys` or
/// another file OpenSSH reads for something other than client config.
pub fn is_config_target(rel: &str) -> bool {
    let Some(rest) = rel.strip_prefix(".ssh/") else {
        return false;
    };
    let parts: Vec<&str> = rest.split('/').collect();
    if parts
        .iter()
        .any(|p| p.is_empty() || *p == "." || *p == "..")
    {
        return false;
    }
    let name = parts[parts.len() - 1].to_ascii_lowercase();
    !(name.starts_with("known_hosts")
        || name.starts_with("authorized_keys")
        || name.starts_with("id_")
        || name.ends_with(".pub")
        || name == "environment"
        || name == "rc")
}

/// `(line, keyword, arguments)` for each directive. Accepts `Keyword args` and
/// `Keyword=args`; comments and blank lines are skipped.
fn directives(content: &str) -> impl Iterator<Item = (u32, &str, &str)> {
    content.lines().enumerate().filter_map(|(index, line)| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let end = line
            .find(|c: char| c.is_whitespace() || c == '=')
            .unwrap_or(line.len());
        let (keyword, rest) = line.split_at(end);
        let rest = rest.trim_start();
        let rest = rest.strip_prefix('=').unwrap_or(rest).trim();
        Some(((index + 1) as u32, keyword, rest))
    })
}

/// Whitespace-separated arguments; double quotes group, an unquoted `#` starts a comment.
fn args(text: &str) -> Vec<String> {
    let mut result = vec![];
    let mut chars = text.chars().peekable();
    loop {
        while chars.next_if(|c| c.is_whitespace()).is_some() {}
        match chars.peek() {
            None | Some('#') => break,
            _ => {}
        }
        let mut arg = String::new();
        let mut quoted = false;
        while let Some(c) = chars.next_if(|c| quoted || !c.is_whitespace()) {
            if c == '"' {
                quoted = !quoted;
            } else {
                arg.push(c);
            }
        }
        result.push(arg);
    }
    result
}

/// The canonical spelling of `keyword` if it is one of `known` (keywords ignore case).
fn known(keyword: &str, known: &[&'static str]) -> Option<&'static str> {
    known
        .iter()
        .find(|k| k.eq_ignore_ascii_case(keyword))
        .copied()
}

fn is_absolute(value: &str) -> bool {
    let b = value.as_bytes();
    value.starts_with('/')
        || value.starts_with("\\\\")
        || (b.len() > 2
            && b[0].is_ascii_alphabetic()
            && b[1] == b':'
            && matches!(b[2], b'\\' | b'/'))
}

/// A path that only exists on the source OS.
fn is_source_path(value: &str) -> bool {
    is_absolute(value) || value.starts_with("~/Library/")
}

/// The first value of each directive in `keywords`, keyed by its canonical spelling.
fn values<'a>(
    content: &'a str,
    keywords: &'a [&'static str],
) -> impl Iterator<Item = (u32, &'static str, String)> + 'a {
    directives(content).filter_map(|(line, keyword, rest)| {
        let keyword = known(keyword, keywords)?;
        let value = args(rest).into_iter().next()?;
        Some((line, keyword, value))
    })
}

/// Scan findings for lines that are commented out on another OS: `UseKeychain` and
/// source-OS paths. `Control*` depends on the destination and is left to the export.
pub(crate) fn os_specific_lines(content: &str) -> Vec<Finding> {
    let mut keywords = vec!["UseKeychain"];
    keywords.extend(PATH_KEYWORDS);
    values(content, &keywords)
        .filter(|(_, keyword, value)| *keyword == "UseKeychain" || is_source_path(value))
        .map(|(line, key, _)| Finding {
            line: Some(line),
            kind: FindingKind::OsSpecificLine { key: key.into() },
        })
        .collect()
}

/// Lines to comment out when exporting to `target` from another kind of OS.
pub(crate) fn lines_to_comment(content: &str, target: Os) -> Vec<(u32, String)> {
    let mut keywords = vec!["UseKeychain"];
    keywords.extend(PATH_KEYWORDS);
    keywords.extend(CONTROL_KEYWORDS);
    values(content, &keywords)
        .filter(|(_, keyword, value)| {
            (*keyword == "UseKeychain" && target != Os::MacOs)
                || (CONTROL_KEYWORDS.contains(keyword) && target == Os::Windows)
                || (PATH_KEYWORDS.contains(keyword) && is_source_path(value))
        })
        .map(|(line, key, _)| (line, key.to_owned()))
        .collect()
}

/// `IdentityFile` and `CertificateFile` values as written.
pub(crate) fn key_paths(content: &str) -> Vec<(u32, String)> {
    values(content, &KEY_KEYWORDS)
        .filter(|(_, _, value)| !value.eq_ignore_ascii_case("none"))
        .map(|(line, _, value)| (line, value))
        .collect()
}

/// `Include` arguments with their line, in reading order.
pub(crate) fn includes(content: &str) -> Vec<(u32, String)> {
    directives(content)
        .filter(|(_, keyword, _)| keyword.eq_ignore_ascii_case("Include"))
        .flat_map(|(line, _, rest)| args(rest).into_iter().map(move |arg| (line, arg)))
        .collect()
}

fn has_wildcard(part: &str) -> bool {
    part.contains(['*', '?'])
}

/// An `Include` argument as a pattern relative to `~/.ssh`, or `None` when Repot does not
/// follow it: outside `~/.ssh`, `..`, tokens, or a wildcard before the last part.
fn resolve_include(arg: &str, home: &Path) -> Option<String> {
    let arg = arg.replace('\\', "/");
    let strip = |text: &str, prefix: &str| {
        (text.len() >= prefix.len()
            && text.is_char_boundary(prefix.len())
            && text[..prefix.len()].eq_ignore_ascii_case(prefix))
        .then(|| text[prefix.len()..].to_owned())
    };
    let rel = if arg.starts_with('~') {
        strip(&arg, "~/.ssh/")?
    } else if is_absolute(&arg) {
        let home = home.to_string_lossy().replace('\\', "/");
        strip(&arg, &format!("{}/.ssh/", home.trim_end_matches('/')))?
    } else {
        arg
    };
    if rel.contains(['%', '$']) {
        return None;
    }
    let parts: Vec<&str> = rel
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    let (_, dirs) = parts.split_last()?;
    if parts.contains(&"..") || dirs.iter().any(|p| has_wildcard(p)) {
        return None;
    }
    Some(parts.join("/"))
}

/// Shell-style match of `*` and `?`; like glob(3), a leading dot must be matched literally.
fn matches(pattern: &str, name: &str) -> bool {
    fn go(p: &[char], n: &[char]) -> bool {
        match p.split_first() {
            None => n.is_empty(),
            Some(('*', rest)) => (0..=n.len()).any(|i| go(rest, &n[i..])),
            Some(('?', rest)) => !n.is_empty() && go(rest, &n[1..]),
            Some((c, rest)) => n.first() == Some(c) && go(rest, &n[1..]),
        }
    }
    if name.starts_with('.') && !pattern.starts_with('.') {
        return false;
    }
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    go(&p, &n)
}

/// Files under `~/.ssh` a pattern names, in name order, that may become items.
fn expand(env: &dyn Env, ssh: &Path, pattern: &str) -> Vec<String> {
    let (dir, name) = pattern.rsplit_once('/').unwrap_or(("", pattern));
    let join = |file: &str| {
        if dir.is_empty() {
            file.to_owned()
        } else {
            format!("{dir}/{file}")
        }
    };
    let candidates = if has_wildcard(name) {
        let base = if dir.is_empty() {
            ssh.to_path_buf()
        } else {
            ssh.join(dir)
        };
        let mut names: Vec<String> = env
            .list_dir(&base)
            .unwrap_or_default()
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(str::to_owned))
            .filter(|n| matches(name, n))
            .collect();
        names.sort();
        names.into_iter().map(|n| join(&n)).collect()
    } else {
        vec![pattern.to_owned()]
    };
    candidates
        .into_iter()
        // The name check comes first so a key file is never even looked at.
        .filter(|rel| is_config_target(&format!(".ssh/{rel}")))
        .filter(|rel| {
            let path = ssh.join(rel);
            env.exists(&path) && env.list_dir(&path).is_err()
        })
        .collect()
}

/// Findings beyond secrets, by line: not-followed includes, OS lines, key paths.
fn findings(content: &str, home: &Path) -> Vec<Finding> {
    let mut ranked: Vec<(u32, u8, Finding)> = includes(content)
        .into_iter()
        .filter(|(_, arg)| resolve_include(arg, home).is_none())
        .map(|(line, path)| {
            (
                line,
                0,
                Finding {
                    line: Some(line),
                    kind: FindingKind::ExternalInclude { path },
                },
            )
        })
        .collect();
    ranked.extend(
        os_specific_lines(content)
            .into_iter()
            .map(|f| (f.line.unwrap_or(0), 1, f)),
    );
    ranked.extend(key_paths(content).into_iter().map(|(line, path)| {
        (
            line,
            2,
            Finding {
                line: Some(line),
                kind: FindingKind::KeyFile { path },
            },
        )
    }));
    ranked.sort_by_key(|(line, rank, _)| (*line, *rank));
    ranked.into_iter().map(|(_, _, f)| f).collect()
}

struct Walk<'a> {
    env: &'a dyn Env,
    home: PathBuf,
    ssh: PathBuf,
    seen: HashSet<String>,
    items: Vec<Item>,
}
impl Walk<'_> {
    /// Depth-first in reading order, like OpenSSH.
    fn visit(&mut self, rel: &str, depth: usize) {
        if self.items.len() >= MAX_FILES {
            return;
        }
        let path = self.ssh.join(rel);
        let content = self.env.read_file(&path).ok();
        if content.as_deref().is_some_and(secrets::has_private_key) {
            return;
        }
        let name = format!(".ssh/{rel}");
        let home = self.home.clone();
        let Some(item) = config_file_item(
            self.env,
            ConfigFileSpec {
                provider: "ssh",
                category: Category::Ssh,
                id: format!("ssh:file:{name}"),
                name: name.clone(),
                path,
                target: TargetPath::Home(name.into()),
                portability: Portability::Any,
                catalog_key: CATALOG_KEY,
            },
            |content| findings(content, &home),
        ) else {
            return;
        };
        self.items.push(item);
        let Some(content) = content else {
            return;
        };
        if depth >= MAX_DEPTH {
            return;
        }
        for (_, arg) in includes(&content) {
            let Some(pattern) = resolve_include(&arg, &self.home) else {
                continue;
            };
            for rel in expand(self.env, &self.ssh, &pattern) {
                if self.seen.insert(rel.clone()) {
                    self.visit(&rel, depth + 1);
                }
            }
        }
    }
}

impl Provider for Ssh {
    fn id(&self) -> &'static str {
        "ssh"
    }
    fn category(&self) -> Category {
        Category::Ssh
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.exists(&env.home_dir().join(".ssh").join("config"))
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let home = env.home_dir();
        let mut walk = Walk {
            env,
            ssh: home.join(".ssh"),
            home,
            seen: HashSet::from(["config".to_owned()]),
            items: vec![],
        };
        walk.visit("config", 0);
        Ok(walk.items)
    }
}
