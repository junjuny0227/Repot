use repot_core::{
    env::{CommandOutput, Env, EnvError, FakeEnv},
    model::*,
    os::Os,
    providers::ssh::{Ssh, is_config_target},
    scan::Provider,
};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

/// The plan's example `~/.ssh/config` (plan 16, 예시).
const CONFIG: &str = "Include config.d/*
Include ~/.orbstack/ssh/config

Host *
  AddKeysToAgent yes
  UseKeychain yes
  IdentityFile ~/.ssh/id_ed25519

Host github-work
  HostName github.com
  User git
  IdentityFile ~/.ssh/work_ed25519
  IdentitiesOnly yes

Host bastion
  HostName bastion.example.com
  ControlMaster auto
  ControlPath ~/.ssh/cm-%r@%h:%p
  ControlPersist 10m
  SetEnv DEPLOY_TOKEN=abc123
";
const WORK: &str = "Host *.corp.example.com\n  User alice\n  ProxyJump bastion\n";
const KEY: &str = "-----BEGIN OPENSSH PRIVATE KEY-----\nAAAA\n-----END OPENSSH PRIVATE KEY-----\n";

fn ssh(home: &str, rel: &str) -> PathBuf {
    PathBuf::from(home).join(".ssh").join(rel)
}
fn mac() -> FakeEnv {
    FakeEnv::new(Os::MacOs, "/Users/demo")
}
fn with(env: FakeEnv, files: &[(&str, &str)]) -> FakeEnv {
    files.iter().fold(env, |env, (rel, content)| {
        let home = env.home_dir();
        env.with_file(ssh(home.to_str().unwrap(), rel), content)
    })
}
fn ids(items: &[Item]) -> Vec<&str> {
    items.iter().map(|i| i.id.as_str()).collect()
}
fn finding(line: u32, kind: FindingKind) -> Finding {
    Finding {
        line: Some(line),
        kind,
    }
}
fn external(line: u32, path: &str) -> Finding {
    finding(line, FindingKind::ExternalInclude { path: path.into() })
}
fn os_line(line: u32, key: &str) -> Finding {
    finding(line, FindingKind::OsSpecificLine { key: key.into() })
}
fn key_file(line: u32, path: &str) -> Finding {
    finding(line, FindingKind::KeyFile { path: path.into() })
}

/// Records every path the provider touches, to prove it never looks at key files.
struct Recording {
    inner: FakeEnv,
    touched: Mutex<Vec<PathBuf>>,
}
impl Recording {
    fn touch(&self, path: &Path) {
        self.touched.lock().unwrap().push(path.to_path_buf());
    }
}
impl Env for Recording {
    fn os(&self) -> Os {
        self.inner.os()
    }
    fn home_dir(&self) -> PathBuf {
        self.inner.home_dir()
    }
    fn var(&self, name: &str) -> Option<String> {
        self.inner.var(name)
    }
    fn which(&self, program: &str) -> bool {
        self.inner.which(program)
    }
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, EnvError> {
        self.inner.run(program, args)
    }
    fn read_file(&self, path: &Path) -> Result<String, EnvError> {
        self.touch(path);
        self.inner.read_file(path)
    }
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, EnvError> {
        self.touch(path);
        self.inner.read_bytes(path)
    }
    /// Reversed, because a real directory listing has no order.
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, EnvError> {
        let mut paths = self.inner.list_dir(path)?;
        paths.reverse();
        Ok(paths)
    }
    fn exists(&self, path: &Path) -> bool {
        self.touch(path);
        self.inner.exists(path)
    }
    fn file_len(&self, path: &Path) -> Result<u64, EnvError> {
        self.touch(path);
        self.inner.file_len(path)
    }
    fn temp_file(&self, name: &str) -> PathBuf {
        self.inner.temp_file(name)
    }
    fn remove_file(&self, path: &Path) {
        self.inner.remove_file(path)
    }
    fn read_link(&self, path: &Path) -> Option<PathBuf> {
        self.touch(path);
        self.inner.read_link(path)
    }
}

#[test]
fn scans_the_plan_example() {
    let env = with(mac(), &[("config", CONFIG), ("config.d/work", WORK)])
        .with_file("/Users/demo/.orbstack/ssh/config", "Host orb\n");
    assert!(Ssh.detect(&env));
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(
        ids(&items),
        vec!["ssh:file:.ssh/config", "ssh:file:.ssh/config.d/work"]
    );
    let config = &items[0];
    assert_eq!(config.provider, "ssh");
    assert_eq!(config.category, Category::Ssh);
    assert_eq!(config.name, ".ssh/config");
    assert_eq!(config.portability, Portability::Any);
    assert_eq!(config.catalog_id.as_deref(), Some("file:ssh-config"));
    assert_eq!(
        config.description.as_deref(),
        Some("SSH client configuration defines host aliases and connection options.")
    );
    assert_eq!(
        config.origin,
        Origin::Path("/Users/demo/.ssh/config".into())
    );
    assert_eq!(
        config.details,
        ItemDetails::ConfigFile {
            source: "/Users/demo/.ssh/config".into(),
            target: TargetPath::Home(".ssh/config".into()),
        }
    );
    // Secrets first (like every config file), then the SSH rules by line.
    assert_eq!(
        config.findings,
        vec![
            finding(
                20,
                FindingKind::Secret {
                    name: Some("DEPLOY_TOKEN".into())
                }
            ),
            external(2, "~/.orbstack/ssh/config"),
            os_line(6, "UseKeychain"),
            key_file(7, "~/.ssh/id_ed25519"),
            key_file(12, "~/.ssh/work_ed25519"),
        ]
    );
    let work = &items[1];
    assert_eq!(work.name, ".ssh/config.d/work");
    assert_eq!(work.catalog_id.as_deref(), Some("file:ssh-config"));
    assert_eq!(
        work.details,
        ItemDetails::ConfigFile {
            source: "/Users/demo/.ssh/config.d/work".into(),
            target: TargetPath::Home(".ssh/config.d/work".into()),
        }
    );
    assert!(work.findings.is_empty());
}

#[test]
fn targets_under_ssh_are_private() {
    for (rel, private) in [
        (".ssh/config", true),
        (".ssh/config.d/work", true),
        (".sshrc", false),
        (".config/ssh/config", false),
        (".zshrc", false),
    ] {
        assert_eq!(TargetPath::Home(rel.into()).is_private(), private, "{rel}");
    }
    assert!(!TargetPath::PowerShellProfile.is_private());
    assert!(
        !TargetPath::EditorUser {
            editor: Editor::VsCode,
            rel: ".ssh/config".into()
        }
        .is_private()
    );
}

#[test]
fn only_the_main_file_without_includes() {
    let env = with(
        mac(),
        &[
            ("config", "Host a\n  User me\n"),
            ("config.d/work", WORK),
            ("other", "Host b\n"),
        ],
    );
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(ids(&items), vec!["ssh:file:.ssh/config"]);
    assert!(items[0].findings.is_empty());
    assert!(!Ssh.detect(&with(mac(), &[("config.d/work", WORK)])));
    assert!(Ssh.scan(&mac()).unwrap().is_empty());
}

#[test]
fn includes_outside_ssh_dotdot_and_middle_wildcards_are_not_followed() {
    let config = "Include ~/.orbstack/ssh/config /etc/ssh/ssh_config.d/*\n\
                  Include config.d/../escape ~/.ssh/../.aws/config\n\
                  Match host x\n  Include */nested\n\
                  Include ~other/.ssh/x\n\
                  Include \"/Users/demo/.ssh/abs\"\n";
    let env = with(
        mac(),
        &[
            ("config", config),
            ("escape", "Host e\n"),
            ("dir/nested", "Host n\n"),
            ("abs", "Host abs\n"),
        ],
    )
    .with_file("/Users/demo/.orbstack/ssh/config", "Host orb\n")
    .with_file("/etc/ssh/ssh_config.d/x", "Host etc\n")
    .with_file("/Users/demo/.aws/config", "[default]\n");
    let rec = Recording {
        inner: env,
        touched: Mutex::new(vec![]),
    };
    let items = Ssh.scan(&rec).unwrap();
    // An absolute path inside ~/.ssh is followed, like a relative one.
    assert_eq!(
        ids(&items),
        vec!["ssh:file:.ssh/config", "ssh:file:.ssh/abs"]
    );
    assert_eq!(
        items[0].findings,
        vec![
            external(1, "~/.orbstack/ssh/config"),
            external(1, "/etc/ssh/ssh_config.d/*"),
            external(2, "config.d/../escape"),
            external(2, "~/.ssh/../.aws/config"),
            external(4, "*/nested"),
            external(5, "~other/.ssh/x"),
        ]
    );
    let touched = rec.touched.lock().unwrap();
    for outside in [
        "/Users/demo/.orbstack/ssh/config",
        "/etc/ssh/ssh_config.d/x",
        "/Users/demo/.aws/config",
        "/Users/demo/.ssh/escape",
        "/Users/demo/.ssh/dir/nested",
    ] {
        assert!(
            !touched.iter().any(|p| p == Path::new(outside)),
            "{outside} was touched"
        );
    }
}

#[test]
fn wildcards_match_the_last_part_in_name_order_and_skip_dot_files() {
    let env = with(
        mac(),
        &[
            ("config", "Include conf.d/*.conf host?\n"),
            ("conf.d/b.conf", "Host b\n"),
            ("conf.d/a.conf", "Host a\n"),
            ("conf.d/.hidden.conf", "Host h\n"),
            ("conf.d/notes.txt", "Host t\n"),
            ("conf.d/sub.conf/x", "Host x\n"),
            ("host1", "Host 1\n"),
            ("host10", "Host 10\n"),
        ],
    );
    let env = Recording {
        inner: env,
        touched: Mutex::new(vec![]),
    };
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(
        ids(&items),
        vec![
            "ssh:file:.ssh/config",
            "ssh:file:.ssh/conf.d/a.conf",
            "ssh:file:.ssh/conf.d/b.conf",
            "ssh:file:.ssh/host1",
        ]
    );
}

#[test]
fn keys_and_other_ssh_files_never_become_items() {
    let env = with(
        mac(),
        &[
            ("config", "Include *\nInclude keys/*\n"),
            ("id_ed25519", KEY),
            ("id_ed25519.pub", "ssh-ed25519 AAAA me\n"),
            ("known_hosts", "github.com ssh-ed25519 AAAA\n"),
            ("known_hosts.old", "github.com ssh-ed25519 AAAA\n"),
            ("authorized_keys", "ssh-ed25519 AAAA me\n"),
            ("authorized_keys2", "ssh-ed25519 AAAA me\n"),
            ("environment", "A=b\n"),
            ("rc", "echo hi\n"),
            ("ID_RSA", KEY),
            ("work.pub", "ssh-ed25519 AAAA me\n"),
            // Not a key name, but holds a private key block.
            ("deploy", KEY),
            ("keys/work", KEY),
            ("extra", "Host extra\n"),
        ],
    );
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(
        ids(&items),
        vec!["ssh:file:.ssh/config", "ssh:file:.ssh/extra"]
    );
}

#[test]
fn is_config_target_allows_only_config_files_under_ssh() {
    for rel in [
        ".ssh/config",
        ".ssh/config.d/work",
        ".ssh/conf.d/a.conf",
        ".ssh/extra",
    ] {
        assert!(is_config_target(rel), "{rel}");
    }
    for rel in [
        ".ssh/known_hosts",
        ".ssh/known_hosts.old",
        ".ssh/authorized_keys",
        ".ssh/authorized_keys2",
        ".ssh/Authorized_Keys",
        ".ssh/id_ed25519",
        ".ssh/ID_RSA",
        ".ssh/id_rsa.pub",
        ".ssh/work.pub",
        ".ssh/config.d/id_work",
        ".ssh/environment",
        ".ssh/rc",
        ".ssh",
        ".ssh/",
        ".ssh//config",
        ".ssh/./config",
        ".ssh/../.zshrc",
        ".ssh/config.d/../known_hosts",
        ".SSH/config",
        "config",
        ".config/ssh/config",
    ] {
        assert!(!is_config_target(rel), "{rel}");
    }
}

#[test]
fn cycles_are_followed_once() {
    let env = with(
        mac(),
        &[
            ("config", "Include a\nInclude config\n"),
            ("a", "Include config a b\n"),
            ("b", "Include a ~/.ssh/config\n"),
        ],
    );
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(
        ids(&items),
        vec!["ssh:file:.ssh/config", "ssh:file:.ssh/a", "ssh:file:.ssh/b"]
    );
}

#[test]
fn depth_and_file_count_are_limited() {
    // A chain config -> f1 -> f2 -> …: OpenSSH reads 16 levels of Include.
    let mut files = vec![("config".to_owned(), "Include f1\n".to_owned())];
    for n in 1..=20 {
        files.push((format!("f{n}"), format!("Include f{}\n", n + 1)));
    }
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let items = Ssh.scan(&with(mac(), &refs)).unwrap();
    assert_eq!(items.len(), 17);
    assert_eq!(items.last().unwrap().id, "ssh:file:.ssh/f16");

    let mut files = vec![("config".to_owned(), "Include conf.d/*\n".to_owned())];
    for n in 0..40 {
        files.push((format!("conf.d/{n:02}"), "Host x\n".to_owned()));
    }
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let items = Ssh.scan(&with(mac(), &refs)).unwrap();
    assert_eq!(items.len(), 32);
    assert_eq!(items.last().unwrap().id, "ssh:file:.ssh/conf.d/30");
}

#[test]
fn key_value_syntax_and_keyword_case() {
    let config = "include=conf.d/work\n\
                  INCLUDE   \"quoted name\"\n\
                  Host x\n\
                  \tusekeychain=yes\n\
                  \tIDENTITYFILE=\"/Users/demo/keys/id work\"\n\
                  \tIdentityfile = ~/.ssh/id_rsa\n\
                  \tCertificateFile ~/.ssh/id_rsa-cert.pub\n\
                  \tidentityagent \"~/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock\"\n\
                  \tUserKnownHostsFile /dev/null\n\
                  \tControlPath ~/.ssh/cm-%C\n\
                  \t# IdentityFile ~/.ssh/commented\n\
                  \tIdentityFile ~/.ssh/a # trailing comment\n\
                  Include conf.d/work # see note\n";
    let env = with(
        mac(),
        &[
            ("config", config),
            ("conf.d/work", WORK),
            ("quoted name", "Host q\n"),
            // Words after `#` are a comment, not more files to include.
            ("note", "Host n\n"),
            ("#", "Host h\n"),
        ],
    );
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(
        ids(&items),
        vec![
            "ssh:file:.ssh/config",
            "ssh:file:.ssh/conf.d/work",
            "ssh:file:.ssh/quoted name"
        ]
    );
    assert_eq!(
        items[0].findings,
        vec![
            os_line(4, "UseKeychain"),
            os_line(5, "IdentityFile"),
            key_file(5, "/Users/demo/keys/id work"),
            key_file(6, "~/.ssh/id_rsa"),
            key_file(7, "~/.ssh/id_rsa-cert.pub"),
            os_line(8, "IdentityAgent"),
            os_line(9, "UserKnownHostsFile"),
            key_file(12, "~/.ssh/a"),
        ]
    );
}

#[test]
fn windows_home_gives_the_same_ids_and_targets() {
    let home = r"C:\Users\demo";
    let env = with(
        FakeEnv::new(Os::Windows, home),
        &[
            (
                "config",
                "Include config.d/*\nInclude C:\\Users\\demo\\.ssh\\extra\nInclude c:/users/demo/.ssh/lower\n\
                 Include C:\\ProgramData\\ssh\\x\nHost w\n  IdentityFile C:\\Users\\demo\\.ssh\\id_w\n\
                 \x20 CertificateFile \\\\server\\keys\\cert.pub\n",
            ),
            ("config.d/work", WORK),
            ("extra", "Host e\n"),
            ("lower", "Host l\n"),
        ],
    );
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(
        ids(&items),
        vec![
            "ssh:file:.ssh/config",
            "ssh:file:.ssh/config.d/work",
            "ssh:file:.ssh/extra",
            "ssh:file:.ssh/lower",
        ]
    );
    assert_eq!(
        items[0].details,
        ItemDetails::ConfigFile {
            source: ssh(home, "config"),
            target: TargetPath::Home(".ssh/config".into()),
        }
    );
    assert_eq!(
        items[1].details,
        ItemDetails::ConfigFile {
            source: ssh(home, "config.d/work"),
            target: TargetPath::Home(".ssh/config.d/work".into()),
        }
    );
    assert_eq!(
        items[0].findings,
        vec![
            external(4, r"C:\ProgramData\ssh\x"),
            os_line(6, "IdentityFile"),
            key_file(6, r"C:\Users\demo\.ssh\id_w"),
            os_line(7, "CertificateFile"),
            key_file(7, r"\\server\keys\cert.pub"),
        ]
    );
}

#[test]
fn key_files_are_never_read_or_checked() {
    let config = "Host *\n  IdentityFile ~/.ssh/id_ed25519\n  CertificateFile /Users/demo/.ssh/id_ed25519-cert.pub\n";
    let without_keys = with(mac(), &[("config", config)]);
    let with_keys = with(
        mac(),
        &[
            ("config", config),
            ("id_ed25519", KEY),
            ("id_ed25519-cert.pub", "ssh-ed25519-cert AAAA\n"),
        ],
    );
    let rec = Recording {
        inner: with_keys,
        touched: Mutex::new(vec![]),
    };
    let scanned = Ssh.scan(&rec).unwrap();
    assert_eq!(scanned, Ssh.scan(&without_keys).unwrap());
    let touched = rec.touched.lock().unwrap();
    assert!(
        touched
            .iter()
            .all(|p| !p.to_string_lossy().contains("id_ed25519")),
        "{touched:?}"
    );
}

#[test]
fn an_unreadable_config_is_an_item_with_a_finding() {
    let env = mac().with_unreadable_file("/Users/demo/.ssh/config");
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(ids(&items), vec!["ssh:file:.ssh/config"]);
    assert_eq!(
        items[0].findings,
        vec![Finding {
            line: None,
            kind: FindingKind::Unreadable
        }]
    );
}
