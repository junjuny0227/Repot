use repot_core::{export::payload::*, os::*, *};
const GITCONFIG: &str = "[user]\n\tname = Test\n\temail = t@example.com\n[core]\n\tautocrlf = input\n\teditor = /usr/local/bin/code --wait\n[credential]\n\thelper = osxkeychain\n\turl = https://synthetic-token@example.com/repo\n";
fn config(provider: &str, file: &str, target: TargetPath) -> Item {
    Item {
        id: file.into(),
        provider: provider.into(),
        category: Category::Git,
        name: file.into(),
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Path(file.into()),
        portability: Portability::Any,
        details: ItemDetails::ConfigFile {
            source: file.into(),
            target,
        },
        findings: vec![],
    }
}
fn git_item() -> Item {
    config(
        "git",
        "/Users/t/.gitconfig",
        TargetPath::Home(".gitconfig".into()),
    )
}
fn vscode_item(file: &str) -> Item {
    config(
        "vscode",
        file,
        TargetPath::EditorUser {
            editor: Editor::VsCode,
            rel: file.into(),
        },
    )
}
#[test]
fn masks_then_comments_os_lines_in_place() {
    let p = prepare_config(&git_item(), GITCONFIG, &[], Os::MacOs, Os::Windows);
    let lines: Vec<_> = p.contents.lines().collect();
    assert_eq!(lines.len(), GITCONFIG.lines().count());
    assert_eq!(
        lines[7],
        "\t# [repot] helper = osxkeychain  # credential.helper is OS-specific; on Windows try: manager"
    );
    assert_eq!(
        lines[4],
        "\t# [repot] autocrlf = input  # core.autocrlf is OS-specific; on Windows try: true"
    );
    assert_eq!(
        lines[5],
        "\t# [repot] editor = /usr/local/bin/code --wait  # core.editor is OS-specific; adjust it for Windows"
    );
    assert!(lines[8].contains("<REDACTED>"));
    assert_eq!(p.path, "home/.gitconfig");
    assert_eq!(
        p.masked,
        vec![MaskedLine {
            file: "home/.gitconfig".into(),
            line: 9,
            name: None
        }]
    );
    assert_eq!(
        p.notes,
        vec![
            ExportNote::CommentedOsLine {
                file: p.path.clone(),
                line: 5,
                key: "core.autocrlf".into()
            },
            ExportNote::CommentedOsLine {
                file: p.path.clone(),
                line: 6,
                key: "core.editor".into()
            },
            ExportNote::CommentedOsLine {
                file: p.path.clone(),
                line: 8,
                key: "credential.helper".into()
            },
        ]
    );
}
#[test]
fn same_os_keeps_os_lines_and_reveal_skips_masking() {
    let p = prepare_config(&git_item(), GITCONFIG, &[9], Os::MacOs, Os::MacOs);
    assert_eq!(p.contents, GITCONFIG);
    assert!(p.masked.is_empty());
    assert!(p.notes.is_empty());
    let p = prepare_config(
        &git_item(),
        GITCONFIG,
        &[9],
        Os::Linux(LinuxFamily::Debian),
        Os::Linux(LinuxFamily::Arch),
    );
    assert_eq!(p.contents, GITCONFIG);
    assert!(p.notes.is_empty());
}
#[test]
fn keybindings_note_only_when_mac_involved() {
    let kb = vscode_item("keybindings.json");
    for (source, target) in [(Os::MacOs, Os::Windows), (Os::Windows, Os::MacOs)] {
        assert_eq!(
            prepare_config(&kb, "[]", &[], source, target).notes,
            vec![ExportNote::KeybindingsMayDiffer {
                file: "vscode/keybindings.json".into()
            }]
        );
    }
    assert!(
        prepare_config(&kb, "[]", &[], Os::Windows, Os::Linux(LinuxFamily::Debian))
            .notes
            .is_empty()
    );
    assert!(
        prepare_config(
            &vscode_item("settings.json"),
            "{}",
            &[],
            Os::MacOs,
            Os::Windows
        )
        .notes
        .is_empty()
    );
}
#[test]
fn preserves_mixed_endings_and_masks_before_commenting() {
    let content =
        "[credential]\r\n  helper = https://test:fake@example.com\r\n[core]\n autocrlf = true";
    let p = prepare_config(&git_item(), content, &[], Os::Windows, Os::MacOs);
    assert_eq!(
        p.contents,
        "[credential]\r\n  # [repot] helper = https://<REDACTED>@example.com  # credential.helper is OS-specific; on macOS try: osxkeychain\r\n[core]\n # [repot] autocrlf = true  # core.autocrlf is OS-specific; on macOS try: input"
    );
    assert_eq!(p.masked[0].line, 2);
}
#[test]
fn masked_lines_are_unique_and_keep_variable_names() {
    let content = "export API_TOKEN=https://test:fake@example.com https://other:fake@example.org\nexport PASSWORD=fake\n";
    let p = prepare_config(
        &vscode_item("settings.json"),
        content,
        &[2],
        Os::MacOs,
        Os::MacOs,
    );
    assert_eq!(
        p.masked,
        vec![MaskedLine {
            file: "vscode/settings.json".into(),
            line: 1,
            name: Some("API_TOKEN".into())
        }]
    );
    assert!(!p.contents.contains("test:fake"));
    assert!(p.contents.contains("PASSWORD=fake"));
}
#[test]
fn git_conversion_requires_provider_and_config_basename() {
    let dest = TargetPath::Home(".config/git/config".into());
    let linux = Os::Linux(LinuxFamily::Fedora);
    let p = prepare_config(
        &config("git", r"C:\Users\t\.config\git\config", dest.clone()),
        "[credential]\nhelper = manager\n[core]\nautocrlf = true\n[gpg]\nprogram = gpg.exe\n",
        &[],
        Os::Windows,
        linux,
    );
    assert!(p.contents.contains("try: libsecret"));
    assert!(p.contents.contains("try: input"));
    assert!(p.contents.contains("adjust it for Linux (Fedora)"));
    for item in [
        config("shell", ".gitconfig", dest.clone()),
        config("git", "ignore", dest),
    ] {
        let p = prepare_config(&item, GITCONFIG, &[], Os::MacOs, Os::Windows);
        assert!(p.notes.is_empty());
        assert!(!p.contents.contains("# [repot]"));
    }
}

/// The plan's example `~/.ssh/config` (plan 16, 예시).
const SSH_CONFIG: &str = "Include config.d/*
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
fn ssh_item(rel: &str) -> Item {
    let mut item = config(
        "ssh",
        &format!("/Users/demo/{rel}"),
        TargetPath::Home(rel.into()),
    );
    item.category = Category::Ssh;
    item
}
#[test]
fn ssh_lines_are_commented_per_destination_and_keys_noted() {
    let commented = |line: &str, key: &str, os: &str| {
        format!("  # [repot] {line}  # {key} is OS-specific; adjust it for {os}")
    };
    let keys = vec![
        ExportNote::SshKey {
            file: "home/.ssh/config".into(),
            line: 7,
            path: "~/.ssh/id_ed25519".into(),
        },
        ExportNote::SshKey {
            file: "home/.ssh/config".into(),
            line: 12,
            path: "~/.ssh/work_ed25519".into(),
        },
    ];
    let note = |line: u32, key: &str| ExportNote::CommentedOsLine {
        file: "home/.ssh/config".into(),
        line,
        key: key.into(),
    };
    // (destination, changed lines, comment notes)
    type Case = (Os, Vec<(usize, String)>, Vec<ExportNote>);
    let cases: Vec<Case> = vec![
        (Os::MacOs, vec![], vec![]),
        (
            Os::Linux(LinuxFamily::Debian),
            vec![(
                6,
                commented("UseKeychain yes", "UseKeychain", "Linux (Debian/Ubuntu)"),
            )],
            vec![note(6, "UseKeychain")],
        ),
        (
            Os::Windows,
            vec![
                (6, commented("UseKeychain yes", "UseKeychain", "Windows")),
                (
                    17,
                    commented("ControlMaster auto", "ControlMaster", "Windows"),
                ),
                (
                    18,
                    commented("ControlPath ~/.ssh/cm-%r@%h:%p", "ControlPath", "Windows"),
                ),
                (
                    19,
                    commented("ControlPersist 10m", "ControlPersist", "Windows"),
                ),
            ],
            vec![
                note(6, "UseKeychain"),
                note(17, "ControlMaster"),
                note(18, "ControlPath"),
                note(19, "ControlPersist"),
            ],
        ),
    ];
    for (target, changed, notes) in cases {
        let p = prepare_config(&ssh_item(".ssh/config"), SSH_CONFIG, &[], Os::MacOs, target);
        assert_eq!(p.path, "home/.ssh/config");
        let mut expected: Vec<String> = SSH_CONFIG.lines().map(str::to_owned).collect();
        expected[19] = "  SetEnv DEPLOY_TOKEN=<REDACTED>".into();
        for (line, text) in changed {
            expected[line - 1] = text;
        }
        assert_eq!(
            p.contents.lines().collect::<Vec<_>>(),
            expected,
            "{target:?}"
        );
        assert_eq!(
            p.masked,
            vec![MaskedLine {
                file: "home/.ssh/config".into(),
                line: 20,
                name: Some("DEPLOY_TOKEN".into()),
            }]
        );
        assert_eq!(p.notes, [notes, keys.clone()].concat(), "{target:?}");
    }
    // Included files without OS lines pass through unchanged.
    let work = "Host *.corp.example.com\n  User alice\n  ProxyJump bastion\n";
    let p = prepare_config(
        &ssh_item(".ssh/config.d/work"),
        work,
        &[],
        Os::MacOs,
        Os::Windows,
    );
    assert_eq!((p.contents.as_str(), p.notes), (work, vec![]));
}
#[test]
fn ssh_source_paths_are_commented_for_any_other_os() {
    let text = "Host *\r\n  IdentityFile /Users/demo/keys/id\r\n  IdentityAgent \"~/Library/Group Containers/x/agent.sock\"\r\n  UserKnownHostsFile ~/.ssh/known_hosts2\r\n  CertificateFile C:\\Users\\demo\\.ssh\\cert.pub\r\n";
    let p = prepare_config(
        &ssh_item(".ssh/config"),
        text,
        &[],
        Os::MacOs,
        Os::Linux(LinuxFamily::Fedora),
    );
    let lines: Vec<_> = p.contents.split("\r\n").collect();
    assert!(lines[1].starts_with("  # [repot] IdentityFile /Users/demo/keys/id  # IdentityFile"));
    assert!(lines[2].starts_with("  # [repot] IdentityAgent"));
    assert_eq!(lines[3], "  UserKnownHostsFile ~/.ssh/known_hosts2");
    assert!(lines[4].starts_with("  # [repot] CertificateFile C:"));
    // Commented key lines are still listed for the user.
    assert_eq!(
        p.notes
            .iter()
            .filter_map(|n| match n {
                ExportNote::SshKey { line, path, .. } => Some((*line, path.as_str())),
                _ => None,
            })
            .collect::<Vec<_>>(),
        vec![
            (2, "/Users/demo/keys/id"),
            (5, "C:\\Users\\demo\\.ssh\\cert.pub")
        ]
    );
    // UseKeychain is Apple's own option: kept when the destination is a Mac.
    let to_mac = prepare_config(
        &ssh_item(".ssh/config"),
        "Host *\n  UseKeychain yes\n  ControlMaster auto\n",
        &[],
        Os::Linux(LinuxFamily::Debian),
        Os::MacOs,
    );
    assert_eq!(
        to_mac.contents,
        "Host *\n  UseKeychain yes\n  ControlMaster auto\n"
    );
    // Same kind of OS: nothing is commented.
    let same = prepare_config(
        &ssh_item(".ssh/config"),
        text,
        &[],
        Os::Linux(LinuxFamily::Debian),
        Os::Linux(LinuxFamily::Fedora),
    );
    assert_eq!(same.contents, text);
    // Other providers never get SSH rules.
    let other = prepare_config(
        &config(
            "shell",
            "/Users/demo/.zshrc",
            TargetPath::Home(".zshrc".into()),
        ),
        "UseKeychain yes\n",
        &[],
        Os::MacOs,
        Os::Windows,
    );
    assert_eq!(
        (other.contents.as_str(), other.notes),
        ("UseKeychain yes\n", vec![])
    );
}
