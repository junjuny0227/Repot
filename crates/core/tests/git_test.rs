use repot_core::{
    env::FakeEnv,
    model::*,
    os::{LinuxFamily, Os},
    providers::git::Git,
    scan::Provider,
};

const GITCONFIG: &str = "[user]\n\tname = T\n[core]\n\texcludesFile = ~/.gitignore_global\n\tautocrlf = input\n\teditor = /usr/local/bin/code --wait\n[credential \"https://github.com\"]\n\thelper = osxkeychain\n[url \"https://ghp_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA@github.com/\"]\n\tinsteadOf = https://github.com/\n";

#[test]
fn gitconfig_findings() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file("/Users/t/.gitconfig", GITCONFIG)
        .with_file("/Users/t/.gitignore_global", ".DS_Store\n");
    let items = Git.scan(&env).unwrap();
    assert_eq!(
        items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["git:file:.gitconfig", "git:file:.gitignore_global"]
    );
    let mut f = items[0].findings.clone();
    f.sort_by_key(|f| f.line);
    assert_eq!(
        f,
        vec![
            Finding {
                line: Some(5),
                kind: FindingKind::OsSpecificLine {
                    key: "core.autocrlf".into()
                }
            },
            Finding {
                line: Some(6),
                kind: FindingKind::OsSpecificLine {
                    key: "core.editor".into()
                }
            },
            Finding {
                line: Some(8),
                kind: FindingKind::OsSpecificLine {
                    key: "credential.helper".into()
                }
            },
            Finding {
                line: Some(9),
                kind: FindingKind::Secret { name: None }
            },
        ]
    );
    assert_eq!(
        items[1].details,
        ItemDetails::ConfigFile {
            source: "/Users/t/.gitignore_global".into(),
            target: TargetPath::Home(".gitignore_global".into())
        }
    );
    assert!(items[0].description.is_some());
    assert_eq!(items[0].catalog_id.as_deref(), Some("file:gitconfig"));
    assert_eq!(
        items[1].catalog_id.as_deref(),
        Some("file:gitignore-global")
    );
}

#[test]
fn default_ignore_and_unreadable() {
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Debian), "/home/t")
        .with_unreadable_file("/home/t/.gitconfig")
        .with_file("/home/t/.config/git/ignore", "");
    let items = Git.scan(&env).unwrap();
    assert_eq!(
        items[0].findings,
        vec![Finding {
            line: None,
            kind: FindingKind::Unreadable
        }]
    );
    assert_eq!(items[1].id, "git:file:.config/git/ignore"); // 빈 파일도 항목
}
