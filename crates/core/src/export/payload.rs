use super::paths::{os_label, payload_path};
use crate::{model::*, os::Os, providers::git::os_specific_lines, secrets};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskedLine {
    pub file: String,
    pub line: u32,
    pub name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportNote {
    CommentedOsLine {
        file: String,
        line: u32,
        key: String,
    },
    KeybindingsMayDiffer {
        file: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFile {
    pub path: String,
    pub contents: String,
    pub masked: Vec<MaskedLine>,
    pub notes: Vec<ExportNote>,
}

/// Prepare a configuration payload without reading or writing the filesystem.
///
/// # Panics
/// Panics if `item.details` is not `ItemDetails::ConfigFile`.
pub fn prepare_config(
    item: &Item,
    content: &str,
    reveal: &[u32],
    source: Os,
    target: Os,
) -> PreparedFile {
    let ItemDetails::ConfigFile {
        source: source_path,
        target: dest,
    } = &item.details
    else {
        panic!("prepare_config requires ItemDetails::ConfigFile");
    };
    let path = payload_path(dest);
    let mut masked: Vec<MaskedLine> = vec![];
    for secret in secrets::detect(content)
        .into_iter()
        .filter(|s| !reveal.contains(&s.line))
    {
        if let Some(previous) = masked.last_mut().filter(|m| m.line == secret.line) {
            if previous.name.is_none() {
                previous.name = secret.name;
            }
        } else {
            masked.push(MaskedLine {
                file: path.clone(),
                line: secret.line,
                name: secret.name,
            });
        }
    }
    let mut contents = secrets::mask(content, reveal);
    let mut notes = vec![];
    if !source.same_kind(target) {
        // Source paths can belong to a different OS than the machine doing the export.
        let source_path = source_path.to_string_lossy().replace('\\', "/");
        let filename = source_path.rsplit('/').next().unwrap_or("");
        if item.provider == "git" && matches!(filename, ".gitconfig" | "config") {
            let findings = os_specific_lines(&contents);
            contents = contents
                .split_inclusive('\n')
                .enumerate()
                .map(|(index, raw)| {
                    let line = (index + 1) as u32;
                    let Some(Finding {
                        kind: FindingKind::OsSpecificLine { key },
                        ..
                    }) = findings.iter().find(|f| f.line == Some(line))
                    else {
                        return raw.to_owned();
                    };
                    let (body, ending) = if let Some(body) = raw.strip_suffix("\r\n") {
                        (body, "\r\n")
                    } else if let Some(body) = raw.strip_suffix('\n') {
                        (body, "\n")
                    } else {
                        (raw, "")
                    };
                    let indent = &body[..body.len() - body.trim_start().len()];
                    let suggestion = match key.as_str() {
                        "credential.helper" => Some(match target {
                            Os::MacOs => "osxkeychain",
                            Os::Windows => "manager",
                            Os::Linux(_) => "libsecret",
                        }),
                        "core.autocrlf" => Some(if target == Os::Windows {
                            "true"
                        } else {
                            "input"
                        }),
                        _ => None,
                    };
                    let label = os_label(target);
                    let guidance = match suggestion {
                        Some(value) => format!("on {label} try: {value}"),
                        None => format!("adjust it for {label}"),
                    };
                    notes.push(ExportNote::CommentedOsLine {
                        file: path.clone(),
                        line,
                        key: key.clone(),
                    });
                    format!(
                        "{indent}# [repot] {}  # {key} is OS-specific; {guidance}{ending}",
                        body.trim()
                    )
                })
                .collect();
        }
        if filename == "keybindings.json" && ((source == Os::MacOs) != (target == Os::MacOs)) {
            notes.push(ExportNote::KeybindingsMayDiffer { file: path.clone() });
        }
    }
    PreparedFile {
        path,
        contents,
        masked,
        notes,
    }
}
