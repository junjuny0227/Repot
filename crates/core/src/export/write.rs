use super::ExportBundle;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("invalid output path: {0}")]
    InvalidPath(String),
    #[error("could not write export: {0}")]
    Io(String),
}

fn output_path(value: &str) -> Result<PathBuf, ExportError> {
    // Reject Windows roots and traversal even when previewing on a Unix host.
    let portable = value.replace('\\', "/");
    if portable.is_empty()
        || portable.starts_with('/')
        || portable.contains(':')
        || portable.split('/').any(|part| part == "..")
    {
        return Err(ExportError::InvalidPath(value.into()));
    }
    let path = PathBuf::from(portable);
    if path.components().all(|c| c == std::path::Component::CurDir) {
        return Err(ExportError::InvalidPath(value.into()));
    }
    Ok(path)
}

/// Stage all files in the chosen parent, then publish the complete directory by rename.
pub fn write_export(
    bundle: &ExportBundle,
    parent: &Path,
    date: &str,
) -> Result<PathBuf, ExportError> {
    let paths = bundle
        .files
        .iter()
        .map(|f| output_path(&f.path))
        .collect::<Result<Vec<_>, _>>()?;
    if date.len() != 10
        || !date.bytes().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
    {
        return Err(ExportError::InvalidPath(date.into()));
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp = parent.join(format!(".repot-export-tmp-{}-{nanos}", std::process::id()));
    // Never clean up a directory we did not create.
    fs::create_dir(&temp).map_err(|e| ExportError::Io(e.to_string()))?;
    let result = (|| -> std::io::Result<PathBuf> {
        for (file, relative) in bundle.files.iter().zip(paths) {
            let path = temp.join(relative);
            if let Some(directory) = path.parent() {
                fs::create_dir_all(directory)?;
            }
            fs::write(&path, &file.contents)?;
            #[cfg(unix)]
            if file.executable {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
            }
        }
        let mut suffix = 1_u64;
        loop {
            let name = if suffix == 1 {
                format!("repot-export-{date}")
            } else {
                format!("repot-export-{date}-{suffix}")
            };
            let destination = parent.join(name);
            match fs::symlink_metadata(&destination) {
                Ok(_) => {
                    suffix += 1;
                    continue;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
            fs::rename(&temp, &destination)?;
            return Ok(destination);
        }
    })();
    match result {
        Ok(path) => Ok(path),
        Err(error) => {
            let cleanup = fs::remove_dir_all(&temp);
            let message = match cleanup {
                Ok(()) => error.to_string(),
                Err(cleanup) => format!("{error}; could not remove {}: {cleanup}", temp.display()),
            };
            Err(ExportError::Io(message))
        }
    }
}
