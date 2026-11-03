use repot_core::{
    export::{paths::*, step::*},
    os::*,
    *,
};
#[test]
fn quotes_single_quote() {
    assert_eq!(bash_quote("it's"), r#"'it'\''s'"#);
    assert_eq!(ps_quote("it's"), "'it''s'");
    assert_eq!(
        bash_path(&TargetPath::Home("it's.json".into()), Os::MacOs),
        r#""$HOME/"'it'\''s.json'"#
    );
    assert_eq!(bash_quote(""), "''");
    assert_eq!(ps_quote("$evil`\""), "'$evil`\"'");
}
#[test]
fn resolves_paths_per_os() {
    let vs = TargetPath::EditorUser {
        editor: Editor::VsCode,
        rel: "snippets/rust.json".into(),
    };
    assert_eq!(
        bash_path(&vs, Os::MacOs),
        r#""$HOME/Library/Application Support/Code/User/"'snippets/rust.json'"#
    );
    assert_eq!(
        bash_path(&vs, Os::Linux(LinuxFamily::Debian)),
        r#""$HOME/.config/Code/User/"'snippets/rust.json'"#
    );
    assert_eq!(
        ps_path(&vs),
        r"(Join-Path $env:APPDATA 'Code\User\snippets\rust.json')"
    );
    assert_eq!(
        ps_path(&TargetPath::Home(".gitconfig".into())),
        "(Join-Path $HOME '.gitconfig')"
    );
    assert_eq!(
        ps_path(&TargetPath::PowerShellProfile),
        r"(Join-Path ([Environment]::GetFolderPath('MyDocuments')) 'PowerShell\Microsoft.PowerShell_profile.ps1')"
    );
    assert_eq!(
        bash_path(&TargetPath::PowerShellProfile, Os::MacOs),
        r#""$HOME/.config/powershell/Microsoft.PowerShell_profile.ps1""#
    );
}
#[test]
fn display_and_payload_paths() {
    let cur = TargetPath::EditorUser {
        editor: Editor::Cursor,
        rel: "keybindings.json".into(),
    };
    assert_eq!(
        display_path(&TargetPath::Home(".zshrc".into()), Os::MacOs),
        "~/.zshrc"
    );
    assert_eq!(
        display_path(&TargetPath::Home(".gitconfig".into()), Os::Windows),
        r"%USERPROFILE%\.gitconfig"
    );
    assert_eq!(
        display_path(&cur, Os::Windows),
        r"%APPDATA%\Cursor\User\keybindings.json"
    );
    assert_eq!(
        display_path(&cur, Os::MacOs),
        "~/Library/Application Support/Cursor/User/keybindings.json"
    );
    assert_eq!(
        display_path(&cur, Os::Linux(LinuxFamily::Other)),
        "~/.config/Cursor/User/keybindings.json"
    );
    assert_eq!(
        display_path(&TargetPath::PowerShellProfile, Os::Windows),
        r"Documents\PowerShell\Microsoft.PowerShell_profile.ps1"
    );
    assert_eq!(
        payload_path(&TargetPath::Home(".config/fish/config.fish".into())),
        "home/.config/fish/config.fish"
    );
    assert_eq!(payload_path(&cur), "cursor/keybindings.json");
    assert_eq!(
        payload_path(&TargetPath::PowerShellProfile),
        "powershell/Microsoft.PowerShell_profile.ps1"
    );
    assert_eq!(
        payload_path(&TargetPath::Home(r".config\한 글\it's.json".into())),
        "home/.config/한 글/it's.json"
    );
}
#[test]
fn config_file_becomes_copy() {
    let target = TargetPath::Home(".zshrc".into());
    let item = Item {
        id: "shell:file:.zshrc".into(),
        provider: "shell".into(),
        category: Category::Shell,
        name: ".zshrc".into(),
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Path("/Users/t/.zshrc".into()),
        portability: Portability::Any,
        details: ItemDetails::ConfigFile {
            source: "/Users/t/.zshrc".into(),
            target: target.clone(),
        },
        findings: vec![],
    };
    assert_eq!(
        steps_for(&item, Os::Linux(LinuxFamily::Debian)),
        vec![Step::CopyFile {
            src: "home/.zshrc".into(),
            dest: target
        }]
    );
}
#[test]
fn labels_all_os_variants() {
    for (os, expected) in [
        (Os::MacOs, "macOS"),
        (Os::Windows, "Windows"),
        (Os::Linux(LinuxFamily::Debian), "Linux (Debian/Ubuntu)"),
        (Os::Linux(LinuxFamily::Fedora), "Linux (Fedora)"),
        (Os::Linux(LinuxFamily::Arch), "Linux (Arch)"),
        (Os::Linux(LinuxFamily::Other), "Linux"),
    ] {
        assert_eq!(os_label(os), expected);
    }
}
