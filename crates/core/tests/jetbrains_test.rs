use repot_core::{
    env::FakeEnv, model::*, os::LinuxFamily, os::Os, providers::jetbrains::JetBrains,
    scan::Provider,
};
use std::io::Write;

/// A plugin jar holding `META-INF/plugin.xml` (or nothing, when `xml` is `None`).
fn jar(xml: Option<&str>) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("META-INF/MANIFEST.MF", options).unwrap();
    zip.write_all(b"Manifest-Version: 1.0\n").unwrap();
    if let Some(xml) = xml {
        zip.start_file("META-INF/plugin.xml", options).unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
fn plugin_xml(id: &str, name: &str) -> String {
    format!(
        "<idea-plugin><id>{id}</id><name>{name}</name><version>1.2.3</version>\
         <description><![CDATA[<p>{name} helps. More text.</p>]]></description></idea-plugin>"
    )
}

const MAC: &str = "/Users/t/Library/Application Support/JetBrains";

fn mac_env() -> FakeEnv {
    let new = format!("{MAC}/IntelliJIdea2026.1");
    let old = format!("{MAC}/IntelliJIdea2025.3");
    FakeEnv::new(Os::MacOs, "/Users/t")
        // An older version of the same IDE is ignored.
        .with_file(format!("{old}/codestyles/Old.xml"), "<code_scheme/>")
        .with_bytes(
            format!("{old}/plugins/old/lib/old.jar"),
            jar(Some(&plugin_xml("old.plugin", "Old"))),
        )
        // A plugin folder: the jar named after the folder holds plugin.xml.
        .with_bytes(
            format!("{new}/plugins/github-copilot-intellij/lib/annotations.jar"),
            jar(None),
        )
        .with_bytes(
            format!("{new}/plugins/github-copilot-intellij/lib/github-copilot-intellij-1.7.1.jar"),
            jar(Some(&plugin_xml("com.github.copilot", "GitHub Copilot"))),
        )
        // A single-jar plugin.
        .with_bytes(
            format!("{new}/plugins/ideolog.jar"),
            jar(Some("<idea-plugin><name>Ideolog</name></idea-plugin>")),
        )
        // Not a plugin: skipped.
        .with_bytes(format!("{new}/plugins/broken/lib/broken.jar"), jar(None))
        .with_file(format!("{new}/codestyles/Default.xml"), "<code_scheme/>")
        .with_file(format!("{new}/codestyles/notes.txt"), "not a scheme")
        .with_file(format!("{new}/keymaps/Mine.xml"), "<keymap/>")
        .with_file(format!("{new}/colors/Dark.icls"), "<scheme/>")
        .with_file(format!("{new}/templates/Custom.xml"), "<templateSet/>")
        .with_file(format!("{new}/options/editor-font.xml"), "<application/>")
        // Machine-specific options stay behind.
        .with_file(format!("{new}/options/jdk.table.xml"), "<application/>")
        .with_file(format!("{new}/options/recentProjects.xml"), "<application/>")
        // A different IDE whose prefix starts like another one.
        .with_file(
            format!("{MAC}/PyCharmCE2026.1/codestyles/Default.xml"),
            "<code_scheme/>",
        )
        // Not an IDE folder.
        .with_file(format!("{MAC}/Toolbox/state.json"), "{}")
}

#[test]
fn reads_the_newest_settings_and_user_plugins() {
    let env = mac_env();
    assert!(JetBrains.detect(&env));
    let items = JetBrains.scan(&env).unwrap();
    let ids: Vec<_> = items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "jetbrains:IntelliJIdea:plugin:com.github.copilot",
            "jetbrains:IntelliJIdea:plugin:Ideolog",
            "jetbrains:IntelliJIdea:file:codestyles/Default.xml",
            "jetbrains:IntelliJIdea:file:keymaps/Mine.xml",
            "jetbrains:IntelliJIdea:file:colors/Dark.icls",
            "jetbrains:IntelliJIdea:file:templates/Custom.xml",
            "jetbrains:IntelliJIdea:file:options/editor-font.xml",
            "jetbrains:PyCharmCe:file:codestyles/Default.xml",
        ]
        .into_iter()
        .map(|id| id.replace("PyCharmCe", "PyCharmCE"))
        .collect::<Vec<_>>()
    );
    let copilot = &items[0];
    assert_eq!(copilot.name, "com.github.copilot");
    assert_eq!(copilot.version.as_deref(), Some("1.2.3"));
    assert_eq!(
        copilot.description.as_deref(),
        Some("GitHub Copilot helps.")
    );
    assert_eq!(copilot.portability, Portability::Any);
    assert_eq!(
        copilot.details,
        ItemDetails::IdePlugin {
            ide: JetBrainsIde::IntelliJIdea,
            folder: "github-copilot-intellij".into()
        }
    );
    assert_eq!(
        items[1].details,
        ItemDetails::IdePlugin {
            ide: JetBrainsIde::IntelliJIdea,
            folder: "ideolog.jar".into()
        }
    );
    // Without a description the plugin's display name is shown.
    assert_eq!(items[1].description.as_deref(), Some("Ideolog"));
    assert_eq!(
        items[2].details,
        ItemDetails::ConfigFile {
            source: format!("{MAC}/IntelliJIdea2026.1/codestyles/Default.xml").into(),
            target: TargetPath::JetBrains {
                ide: JetBrainsIde::IntelliJIdea,
                version: "2026.1".into(),
                rel: "codestyles/Default.xml".into()
            }
        }
    );
    assert_eq!(
        items[2].catalog_id.as_deref(),
        Some("file:jetbrains-codestyle")
    );
}

#[test]
fn finds_plugins_and_settings_on_linux_and_windows() {
    let linux = FakeEnv::new(Os::Linux(LinuxFamily::Debian), "/home/t")
        .with_file(
            "/home/t/.config/JetBrains/WebStorm2026.1/keymaps/K.xml",
            "<keymap/>",
        )
        .with_bytes(
            "/home/t/.local/share/JetBrains/WebStorm2026.1/svelte/lib/svelte.jar",
            jar(Some(&plugin_xml("dev.blachut.svelte.lang", "Svelte"))),
        );
    let ids: Vec<_> = JetBrains
        .scan(&linux)
        .unwrap()
        .into_iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(
        ids,
        vec![
            "jetbrains:WebStorm:plugin:dev.blachut.svelte.lang",
            "jetbrains:WebStorm:file:keymaps/K.xml"
        ]
    );
    let windows = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_var("APPDATA", "C:/Users/t/AppData/Roaming")
        .with_file(
            "C:/Users/t/AppData/Roaming/JetBrains/GoLand2025.3/colors/C.icls",
            "<scheme/>",
        );
    let items = JetBrains.scan(&windows).unwrap();
    assert_eq!(items[0].id, "jetbrains:GoLand:file:colors/C.icls");
}

#[test]
fn nothing_to_find_without_a_jetbrains_folder() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t");
    assert!(!JetBrains.detect(&env));
    assert!(JetBrains.scan(&env).unwrap().is_empty());
}

#[test]
fn folder_names_parse_to_ide_and_version() {
    use repot_core::jetbrains::is_version;
    assert_eq!(
        JetBrainsIde::parse_folder("PyCharmCE2026.1"),
        Some((JetBrainsIde::PyCharmCe, "2026.1".into()))
    );
    assert_eq!(
        JetBrainsIde::parse_folder("PyCharm2026.1"),
        Some((JetBrainsIde::PyCharm, "2026.1".into()))
    );
    assert_eq!(JetBrainsIde::parse_folder("IntelliJIdea"), None);
    assert_eq!(
        JetBrainsIde::parse_folder("IntelliJIdea2026.1-backup"),
        None
    );
    assert!(is_version("2025.10") && !is_version("25.1") && !is_version("2025.1.2"));
}
