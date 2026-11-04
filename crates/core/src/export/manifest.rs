use super::{
    OutputFile,
    paths::{display_path, os_label},
    payload::{ExportNote, MaskedLine},
    script::{ScriptHeader, masked_text, note_text},
    step::Step,
};
use crate::{model::*, os::Os};
use serde_json::json;
use std::fmt::Write;

fn add(files: &mut Vec<OutputFile>, path: &str, contents: String) {
    files.push(OutputFile {
        path: path.into(),
        contents,
        executable: false,
    });
}
fn lines(values: &[&str]) -> String {
    format!("{}\n", values.join("\n"))
}
fn restore_command(path: &str, windows: bool) -> String {
    match path {
        "Brewfile" => "brew bundle --file=Brewfile".into(),
        "winget.json" => {
            "winget import -i winget.json --accept-package-agreements --accept-source-agreements"
                .into()
        }
        "scoop.json" => "scoop import scoop.json".into(),
        "choco-packages.config" => {
            "# Run in an Administrator PowerShell\nchoco install choco-packages.config -y".into()
        }
        "apt-packages.txt" => "xargs -a apt-packages.txt sudo apt-get install -y".into(),
        "dnf-packages.txt" => "sudo dnf install -y $(cat dnf-packages.txt)".into(),
        "pacman-packages.txt" => "sudo pacman -S --needed - < pacman-packages.txt".into(),
        ".tool-versions" => "mise install # or asdf install".into(),
        "jetbrains-plugins.txt" => if windows {
            "# Close the IDEs first; use the full path to idea64.exe etc. if the command is not on PATH\nGet-Content jetbrains-plugins.txt | ForEach-Object { $ide, $id = -split $_; & $ide installPlugins $id }"
        } else {
            "# Close the IDEs first; use the Toolbox script or the app's launcher if the command is not on PATH\nwhile read -r ide id; do \"$ide\" installPlugins \"$id\"; done < jetbrains-plugins.txt"
        }
        .into(),
        _ => {
            let (command, one) = match path {
                "npm-global.txt" => ("npm install -g", false),
                "pnpm-global.txt" => ("pnpm add -g", false),
                "pipx.txt" => ("pipx install", true),
                "cargo.txt" => ("cargo install", true),
                "vscode-extensions.txt" => ("code --install-extension", true),
                "cursor-extensions.txt" => ("cursor --install-extension", true),
                _ => unreachable!("only known manifest files are rendered"),
            };
            if windows {
                format!("Get-Content {path} | ForEach-Object {{ {command} $_ }}")
            } else {
                format!("xargs {}{command} < {path}", if one { "-n1 " } else { "" })
            }
        }
    }
}

pub(crate) fn mapped_packages(steps: &[Step]) -> String {
    let mut text = String::new();
    for step in steps {
        if let (Some(origin), Step::InstallPackage { manager, name, .. }) =
            (step.mapped_origin(), step)
        {
            if text.is_empty() {
                text.push_str("\n## Mapped packages\n\nNames come from the embedded package map. Verify them before restoring.\n\n");
            }
            writeln!(
                text,
                "- {origin} -> {}: {name}",
                super::script::package_cmd(*manager).trim_end_matches("-get")
            )
            .unwrap();
        }
    }
    text
}

pub fn render_manifests(
    header: &ScriptHeader,
    steps: &[Step],
    masked: &[MaskedLine],
    notes: &[ExportNote],
) -> Vec<OutputFile> {
    let mut files = vec![];
    let mut brew = String::new();
    for s in steps {
        if let Step::AddTap {
            manager: PackageManager::Brew,
            name,
        } = s
        {
            writeln!(brew, "tap \"{name}\"").unwrap();
        }
    }
    for s in steps {
        if let Step::InstallPackage {
            manager: PackageManager::Brew,
            kind,
            name,
            ..
        } = s
        {
            writeln!(
                brew,
                "{} \"{name}\"",
                if *kind == PackageKind::Cask {
                    "cask"
                } else {
                    "brew"
                }
            )
            .unwrap();
        }
    }
    if !brew.is_empty() {
        add(&mut files, "Brewfile", brew);
    }
    let packages = |manager| {
        steps
            .iter()
            .filter_map(move |s| match s {
                Step::InstallPackage {
                    manager: m, name, ..
                } if *m == manager => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let winget = packages(PackageManager::Winget);
    if !winget.is_empty() {
        let value = json!({"$schema":"https://aka.ms/winget-packages.schema.2.0.json", "CreationDate":header.generated_at, "Sources":[{"Packages":winget.iter().map(|n| json!({"PackageIdentifier":n})).collect::<Vec<_>>(),"SourceDetails":{"Argument":"https://cdn.winget.microsoft.com/cache","Identifier":"Microsoft.Winget.Source_8wekyb3d8bbwe","Name":"winget","Type":"Microsoft.PreIndexed.Package"}}],"WinGetVersion":"1.0.0"});
        add(
            &mut files,
            "winget.json",
            format!("{}\n", serde_json::to_string_pretty(&value).unwrap()),
        );
    }
    let buckets = steps
        .iter()
        .filter_map(|s| match s {
            Step::AddTap {
                manager: PackageManager::Scoop,
                name,
            } => Some(json!({"Name":name})),
            _ => None,
        })
        .collect::<Vec<_>>();
    let scoop = packages(PackageManager::Scoop);
    if !buckets.is_empty() || !scoop.is_empty() {
        let apps = scoop
            .iter()
            .map(|n| {
                let (bucket, name) = n.split_once('/').unwrap_or(("main", n));
                json!({"Name":name, "Source":bucket})
            })
            .collect::<Vec<_>>();
        add(
            &mut files,
            "scoop.json",
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({"buckets":buckets,"apps":apps})).unwrap()
            ),
        );
    }
    let choco = packages(PackageManager::Choco);
    if !choco.is_empty() {
        let mut xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<packages>\n".to_string();
        for n in choco {
            writeln!(xml, "  <package id=\"{n}\" />").unwrap();
        }
        xml.push_str("</packages>\n");
        add(&mut files, "choco-packages.config", xml);
    }
    for (manager, path) in [
        (PackageManager::Apt, "apt-packages.txt"),
        (PackageManager::Dnf, "dnf-packages.txt"),
        (PackageManager::Pacman, "pacman-packages.txt"),
    ] {
        let values = packages(manager);
        if !values.is_empty() {
            add(&mut files, path, lines(&values));
        }
    }
    let mut runtimes: Vec<(&str, Vec<(&str, bool)>)> = vec![];
    for s in steps {
        if let Step::InstallRuntime {
            runtime,
            version,
            set_default,
            ..
        } = s
        {
            if !matches!(runtime.as_str(), "node" | "python") {
                continue;
            }
            if let Some((_, versions)) = runtimes.iter_mut().find(|(r, _)| *r == runtime) {
                versions.push((version, *set_default));
            } else {
                runtimes.push((runtime, vec![(version, *set_default)]));
            }
        }
    }
    if !runtimes.is_empty() {
        let mut contents = String::new();
        for (runtime, mut versions) in runtimes {
            versions.sort_by_key(|(_, default)| !default);
            let versions = versions.iter().map(|(v, _)| *v).collect::<Vec<_>>();
            writeln!(contents, "{runtime} {}", versions.join(" ")).unwrap();
        }
        add(&mut files, ".tool-versions", contents);
    }
    for (manager, path) in [
        (GlobalManager::Npm, "npm-global.txt"),
        (GlobalManager::Pnpm, "pnpm-global.txt"),
        (GlobalManager::Pipx, "pipx.txt"),
        (GlobalManager::Cargo, "cargo.txt"),
    ] {
        let values = steps
            .iter()
            .filter_map(|s| match s {
                Step::InstallGlobal { manager: m, name } if *m == manager => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !values.is_empty() {
            add(&mut files, path, lines(&values));
        }
    }
    for (editor, path) in [
        (Editor::VsCode, "vscode-extensions.txt"),
        (Editor::Cursor, "cursor-extensions.txt"),
    ] {
        let values = steps
            .iter()
            .filter_map(|s| match s {
                Step::InstallExtension { editor: e, id } if *e == editor => Some(id.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !values.is_empty() {
            add(&mut files, path, lines(&values));
        }
    }
    // One `<ide command> <plugin id>` per line; README shows the loop (plan 14).
    let plugins = steps
        .iter()
        .filter_map(|s| match s {
            Step::InstallIdePlugin { ide, id, .. } => Some(format!("{} {id}", ide.command())),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !plugins.is_empty() {
        let plugins = plugins.iter().map(String::as_str).collect::<Vec<_>>();
        add(&mut files, "jetbrains-plugins.txt", lines(&plugins));
    }
    let windows = header.target == Os::Windows;
    let fence = if windows { "powershell" } else { "bash" };
    let mut readme = format!(
        "# Repot export\n\nGenerated by Repot {} on {}\n\nSource: {} -> Target: {}\n\nReview these files before restoring. Install the required package and version managers first.\n\n## Restore\n",
        header.repot_version,
        header.generated_at,
        os_label(header.source),
        os_label(header.target)
    );
    for file in &files {
        write!(
            readme,
            "\n### {}\n\n```{fence}\n{}\n```\n",
            file.path,
            restore_command(&file.path, windows)
        )
        .unwrap();
    }
    let other = steps
        .iter()
        .filter_map(|s| match s {
            Step::InstallRuntime {
                tool,
                runtime,
                version,
                ..
            } if !matches!(runtime.as_str(), "node" | "python") => Some(match tool {
                RuntimeTool::Rustup => format!("rustup toolchain install {version}"),
                RuntimeTool::Sdkman => format!("sdk install {runtime} {version}"),
                _ => s.label(header.target),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !other.is_empty() {
        write!(
            readme,
            "\n### Other runtimes\n\n```{fence}\n{}\n```\n",
            other.join("\n")
        )
        .unwrap();
    }
    let copies = steps
        .iter()
        .filter_map(|s| match s {
            Step::CopyFile { src, dest } => Some((src, dest)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !copies.is_empty() {
        readme.push_str("\n### Config files\n\nBack up existing destination files before copying.\n\n| Source | Destination |\n| --- | --- |\n");
        for (src, dest) in copies {
            writeln!(
                readme,
                "| `files/{src}` | `{}` |",
                display_path(dest, header.target)
            )
            .unwrap();
        }
    }
    let clones = steps
        .iter()
        .filter_map(|s| match s {
            Step::GitClone { url, dest } => Some(format!(
                "git clone {url} {}",
                display_path(dest, header.target)
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !clones.is_empty() {
        write!(readme, "\n### oh-my-zsh plugins\n\nInstall oh-my-zsh first: https://ohmyz.sh\n\n```{fence}\n{}\n```\n", clones.join("\n")).unwrap();
    }
    readme.push_str(&mapped_packages(steps));
    if !masked.is_empty() {
        readme.push_str("\n## Masked secrets\n\nFill in these masked secrets:\n");
        for m in masked {
            writeln!(readme, "  - {}", masked_text(m)).unwrap();
        }
    }
    if !notes.is_empty() {
        readme.push_str("\n## Notes\n");
        for n in notes {
            writeln!(readme, "\n{}", note_text(n)).unwrap();
        }
    }
    readme.push_str("\nSecret detection is pattern-based and may not catch every secret. Review all files before sharing.\n");
    add(&mut files, "README.md", readme);
    files
}
