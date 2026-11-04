use repot_core::{
    compare::{Presence, compare},
    os::{Os, Shell},
    snapshot::{Snapshot, StoredConfig},
    *,
};
use std::collections::BTreeMap;

fn item(
    id: &str,
    name: &str,
    provider: &str,
    portability: Portability,
    details: ItemDetails,
) -> Item {
    Item {
        id: id.into(),
        name: name.into(),
        provider: provider.into(),
        category: Category::Shell,
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Command("fixture".into()),
        portability,
        details,
        findings: vec![],
    }
}
fn package(manager: PackageManager, kind: PackageKind, name: &str) -> Item {
    item(
        &format!("{manager:?}:{kind:?}:{name}").to_lowercase(),
        name,
        "fixture",
        Portability::Package,
        ItemDetails::Package { manager, kind },
    )
}
fn runtime(tool: RuntimeTool, runtime: &str, version: &str) -> Item {
    let mut i = item(
        &format!("{tool:?}:{runtime}:{version}").to_lowercase(),
        runtime,
        "fixture",
        Portability::Mapped(tool),
        ItemDetails::Runtime {
            tool,
            runtime: runtime.into(),
            is_default: false,
        },
    );
    i.version = Some(version.into());
    i
}
fn global(manager: GlobalManager, name: &str) -> Item {
    item(
        &format!("{manager:?}:{name}").to_lowercase(),
        name,
        "fixture",
        Portability::Any,
        ItemDetails::GlobalPackage { manager },
    )
}
fn extension(id: &str) -> Item {
    item(
        &format!("vscode:code:ext:{id}"),
        id,
        "vscode",
        Portability::Any,
        ItemDetails::Extension {
            editor: Editor::VsCode,
        },
    )
}
fn config(id: &str, source: &str, rel: &str) -> Item {
    item(
        id,
        rel,
        "shell",
        Portability::Any,
        ItemDetails::ConfigFile {
            source: source.into(),
            target: TargetPath::Home(rel.into()),
        },
    )
}
/// A Repot file made on macOS with the given items and config contents.
fn file(items: Vec<Item>, configs: &[(&str, &str)]) -> Snapshot {
    Snapshot {
        created_at: "2026-11-04T10:00Z".into(),
        repot_version: "0.5.0".into(),
        source: Os::MacOs,
        items,
        configs: configs
            .iter()
            .map(|(id, content)| {
                (
                    (*id).to_owned(),
                    StoredConfig {
                        content: (*content).into(),
                        masked: vec![],
                        revealed: vec![],
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
        not_exported: Default::default(),
    }
}
fn ids(map: &std::collections::HashMap<String, Presence>, presence: Presence) -> Vec<&str> {
    let mut v: Vec<_> = map
        .iter()
        .filter(|(_, p)| **p == presence)
        .map(|(id, _)| id.as_str())
        .collect();
    v.sort();
    v
}

#[test]
fn same_os_matches_packages_runtimes_globals_and_extensions() {
    use PackageKind::*;
    use PackageManager::*;
    let snap = file(
        vec![
            package(Brew, Formula, "ripgrep"),
            package(Brew, Formula, "jq"),
            package(Brew, Cask, "Firefox"),
            package(Brew, Formula, "firefox"), // same name, other kind: not the cask
            runtime(RuntimeTool::Nvm, "node", "22.11.0"),
            runtime(RuntimeTool::Nvm, "node", "20.18.0"),
            global(GlobalManager::Npm, "typescript"),
            extension("Rust-Lang.Rust-Analyzer"),
        ],
        &[],
    );
    let env = FakeEnv::new(Os::MacOs, "/Users/new");
    let machine = vec![
        package(Brew, Formula, "ripgrep"),
        package(Brew, Cask, "firefox"),
        runtime(RuntimeTool::Nvm, "node", "22.11.0"),
        runtime(RuntimeTool::Nvm, "node", "18.0.0"),
        global(GlobalManager::Npm, "typescript"),
        extension("rust-lang.rust-analyzer"),
    ];
    let result = compare(
        &snap.items,
        snap.source,
        Target::default_for(Os::MacOs),
        &snap,
        &machine,
        Os::MacOs,
        &env,
    );
    assert_eq!(
        ids(&result, Presence::Present),
        vec![
            "brew:cask:firefox",
            "brew:formula:ripgrep",
            "npm:typescript",
            "nvm:node:22.11.0",
            "vscode:code:ext:Rust-Lang.Rust-Analyzer"
        ]
    );
    // Another runtime version does not count; a formula is not the cask of the same name.
    assert_eq!(
        ids(&result, Presence::Missing),
        vec![
            "brew:formula:firefox",
            "brew:formula:jq",
            "nvm:node:20.18.0"
        ]
    );
}

#[test]
fn other_os_compares_mapped_names_with_the_chosen_manager() {
    use PackageKind::*;
    use PackageManager::*;
    let snap = file(
        vec![
            package(Brew, Formula, "ripgrep"),
            package(Brew, Cask, "firefox"),
            package(Brew, Cask, "rectangle"), // no Windows counterpart
            runtime(RuntimeTool::Nvm, "node", "22.11.0"),
        ],
        &[],
    );
    let env = FakeEnv::new(Os::Windows, "C:/Users/new");
    let machine = vec![
        package(Winget, Package, "burntsushi.ripgrep.msvc"),
        package(Scoop, Package, "firefox"), // Scoop scan names carry no bucket
        runtime(RuntimeTool::NvmWindows, "node", "22.11.0"),
    ];
    let winget = compare(
        &snap.items,
        snap.source,
        Target::default_for(Os::Windows),
        &snap,
        &machine,
        Os::Windows,
        &env,
    );
    assert_eq!(
        ids(&winget, Presence::Present),
        vec!["brew:formula:ripgrep", "nvm:node:22.11.0"]
    );
    // Installed with Scoop, but the winget script would still install it.
    assert_eq!(ids(&winget, Presence::Missing), vec!["brew:cask:firefox"]);
    assert!(!winget.contains_key("brew:cask:rectangle"));

    let scoop = compare(
        &snap.items,
        snap.source,
        Target {
            os: Os::Windows,
            packages: Some(Scoop),
        },
        &snap,
        &machine,
        Os::Windows,
        &env,
    );
    // extras/firefox matches the scanned "firefox"; the bucket step is not judged.
    assert_eq!(scoop.get("brew:cask:firefox"), Some(&Presence::Present));
}

#[test]
fn config_files_compare_contents_with_secrets_masked_on_both_sides() {
    let snap = file(
        vec![
            config("shell:file:.zshrc", "/Users/old/.zshrc", ".zshrc"),
            config("git:file:.gitconfig", "/Users/old/.gitconfig", ".gitconfig"),
            config(
                "shell:file:.config/starship.toml",
                "/Users/old/.config/starship.toml",
                ".config/starship.toml",
            ),
            item(
                "shell:omz:plugins/zsh-autosuggestions",
                "zsh-autosuggestions",
                "shell",
                Portability::SameShell(Shell::Zsh),
                ItemDetails::ShellPlugin {
                    url: "https://github.com/zsh-users/zsh-autosuggestions".into(),
                    target: TargetPath::Home(
                        ".oh-my-zsh/custom/plugins/zsh-autosuggestions".into(),
                    ),
                },
            ),
        ],
        &[
            // Masked in the file; this machine has a different token on the same line.
            (
                "shell:file:.zshrc",
                "export PATH=\"$HOME/bin:$PATH\"\nexport GITHUB_TOKEN=<REDACTED>\n",
            ),
            (
                "git:file:.gitconfig",
                "[user]\n\tname = Me\n[core]\n\teditor = vim\n",
            ),
            ("shell:file:.config/starship.toml", "add_newline = false\n"),
        ],
    );
    let env = FakeEnv::new(Os::MacOs, "/Users/new")
        .with_file(
            "/Users/new/.zshrc",
            "export PATH=\"$HOME/bin:$PATH\"\nexport GITHUB_TOKEN=different-secret\n",
        )
        .with_file(
            "/Users/new/.gitconfig",
            "[user]\n\tname = Me\n[core]\n\teditor = code\n",
        );
    let machine = vec![
        config("shell:file:.zshrc", "/Users/new/.zshrc", ".zshrc"),
        config("git:file:.gitconfig", "/Users/new/.gitconfig", ".gitconfig"),
        item(
            "shell:omz:plugins/zsh-autosuggestions",
            "zsh-autosuggestions",
            "shell",
            Portability::SameShell(Shell::Zsh),
            ItemDetails::ShellPlugin {
                url: "https://github.com/someone-else/fork".into(),
                target: TargetPath::Home(".oh-my-zsh/custom/plugins/zsh-autosuggestions".into()),
            },
        ),
    ];
    let result = compare(
        &snap.items,
        snap.source,
        Target::default_for(Os::MacOs),
        &snap,
        &machine,
        Os::MacOs,
        &env,
    );
    assert_eq!(result["shell:file:.zshrc"], Presence::Present);
    assert_eq!(result["git:file:.gitconfig"], Presence::Differs);
    assert_eq!(
        result["shell:file:.config/starship.toml"],
        Presence::Missing
    );
    // A plugin is judged by where it is installed, like the script's skip check.
    assert_eq!(
        result["shell:omz:plugins/zsh-autosuggestions"],
        Presence::Present
    );
}

#[test]
fn an_unreadable_file_on_this_machine_counts_as_different() {
    let snap = file(
        vec![config("shell:file:.zshrc", "/Users/old/.zshrc", ".zshrc")],
        &[("shell:file:.zshrc", "export A=1\n")],
    );
    let env = FakeEnv::new(Os::MacOs, "/Users/new").with_unreadable_file("/Users/new/.zshrc");
    let machine = vec![config("shell:file:.zshrc", "/Users/new/.zshrc", ".zshrc")];
    let result = compare(
        &snap.items,
        snap.source,
        Target::default_for(Os::MacOs),
        &snap,
        &machine,
        Os::MacOs,
        &env,
    );
    assert_eq!(result["shell:file:.zshrc"], Presence::Differs);
}
