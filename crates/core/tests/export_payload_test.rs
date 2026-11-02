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
