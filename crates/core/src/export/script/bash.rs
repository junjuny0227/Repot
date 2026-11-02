use super::*;
use crate::export::{
    paths::{bash_path, bash_quote as q, os_label},
    step::guide_url,
};
use std::fmt::Write;

const HELPERS: &str = r#"DIR="$(cd "$(dirname "$0")" && pwd)"
STAMP="$(date +%Y%m%d%H%M%S)"
OK_LIST=""; SKIP_LIST=""; FAIL_LIST=""; FAILED=0
have() { command -v "$1" >/dev/null 2>&1; }
record_ok()   { OK_LIST="$OK_LIST  - $1
"; echo "  ok: $1"; }
record_skip() { SKIP_LIST="$SKIP_LIST  - $1
"; echo "  skipped: $1"; }
record_fail() { FAIL_LIST="$FAIL_LIST  - $1
"; FAILED=1; echo "  FAILED: $1"; }
missing() { echo "  $1 not found. Install it first: $2"; }
copy_file() {
  src="$DIR/files/$1"; dest="$2"
  if [ -f "$dest" ] && cmp -s "$src" "$dest"; then return 2; fi
  [ -f "$src" ] || return 1
  mkdir -p "$(dirname "$dest")" || return 1
  if [ -e "$dest" ]; then cp -p "$dest" "$dest.repot-backup-$STAMP" || return 1; fi
  cp "$src" "$dest"
}
"#;

fn tool(step: &Step) -> Option<(&'static str, &'static str)> {
    match step {
        Step::AddTap { manager, .. } | Step::InstallPackage { manager, .. } => {
            Some((package_cmd(*manager), package_cmd(*manager)))
        }
        Step::InstallRuntime { tool, .. } => Some((runtime_cmd(*tool), runtime_name(*tool))),
        Step::InstallGlobal { manager, .. } => Some((global_cmd(*manager), global_cmd(*manager))),
        Step::InstallExtension { editor, .. } => Some((editor_cmd(*editor), editor_cmd(*editor))),
        Step::GitClone { .. } => Some(("git", "oh-my-zsh")),
        Step::CopyFile { .. } => None,
    }
}

fn commands(step: &Step, target: Os) -> (String, String) {
    match step {
        Step::AddTap { name, .. } => (
            format!("brew tap | grep -qx {}", q(name)),
            format!("brew tap {}", q(name)),
        ),
        Step::InstallPackage {
            manager,
            kind,
            name,
        } => {
            let n = q(name);
            match manager {
                PackageManager::Brew => {
                    let k = if *kind == PackageKind::Cask {
                        "--cask"
                    } else {
                        "--formula"
                    };
                    (
                        format!("brew list {k} {n}"),
                        format!(
                            "brew install {}{n}",
                            if *kind == PackageKind::Cask {
                                "--cask "
                            } else {
                                ""
                            }
                        ),
                    )
                }
                PackageManager::Apt => (
                    format!("dpkg -s {n}"),
                    format!("sudo apt-get install -y {n}"),
                ),
                PackageManager::Dnf => (format!("rpm -q {n}"), format!("sudo dnf install -y {n}")),
                PackageManager::Pacman => (
                    format!("pacman -Q {n}"),
                    format!("sudo pacman -S --needed --noconfirm {n}"),
                ),
                _ => ("false".into(), "false".into()),
            }
        }
        Step::InstallRuntime {
            tool,
            runtime,
            version,
            set_default,
        } => {
            let v = q(version);
            let r = q(runtime);
            let (check, mut install, default) = match tool {
                RuntimeTool::Nvm => (
                    format!("nvm ls {v}"),
                    format!("nvm install {v}"),
                    format!("nvm alias default {v}"),
                ),
                RuntimeTool::Fnm => (
                    format!("fnm list | grep -q 'v'{v}"),
                    format!("fnm install {v}"),
                    format!("fnm default {v}"),
                ),
                RuntimeTool::Volta => (
                    format!("volta list {r} --format plain | grep -q {r}'@'{v}"),
                    format!("volta install {r}'@'{v}"),
                    String::new(),
                ),
                RuntimeTool::Pyenv => (
                    format!("pyenv versions --bare | grep -qx {v}"),
                    format!("pyenv install {v}"),
                    format!("pyenv global {v}"),
                ),
                RuntimeTool::Rustup => (
                    format!("rustup toolchain list | grep -q '^'{v}"),
                    format!("rustup toolchain install {v}"),
                    format!("rustup default {v}"),
                ),
                RuntimeTool::Sdkman => (
                    format!("[ -d \"$SDKMAN_DIR/candidates/\"{r}'/'{v} ]"),
                    format!("sdk install {r} {v}"),
                    format!("sdk default {r} {v}"),
                ),
                _ => ("false".into(), "false".into(), String::new()),
            };
            if *set_default && !default.is_empty() {
                write!(install, " && {default}").unwrap();
            }
            (check, install)
        }
        Step::InstallGlobal { manager, name } => {
            let n = q(name);
            match manager {
                GlobalManager::Npm => (
                    format!("npm ls -g --depth=0 {n}"),
                    format!("npm install -g {n}"),
                ),
                GlobalManager::Pnpm => (
                    format!("pnpm ls -g --depth=0 | grep -q '^'{n}' '"),
                    format!("pnpm add -g {n}"),
                ),
                GlobalManager::Pipx => (
                    format!("pipx list --short | grep -q '^'{n}' '"),
                    format!("pipx install {n}"),
                ),
                GlobalManager::Cargo => (
                    format!("cargo install --list | grep -q '^'{n}' v'"),
                    format!("cargo install {n}"),
                ),
            }
        }
        Step::InstallExtension { editor, id } => {
            let c = editor_cmd(*editor);
            (
                format!("{c} --list-extensions | grep -qix {}", q(id)),
                format!("{c} --install-extension {}", q(id)),
            )
        }
        Step::GitClone { url, dest } => (
            format!("[ -d {} ]", bash_path(dest, target)),
            format!("git clone {} {}", q(url), bash_path(dest, target)),
        ),
        Step::CopyFile { .. } => unreachable!("copy uses its own return codes"),
    }
}

pub fn render_bash(plan: &ScriptPlan) -> String {
    let h = &plan.header;
    let mut out = format!(
        "#!/usr/bin/env bash\n# Generated by Repot {} on {}\n# Source: {} -> Target: {}\n# Review this script before running it. Usage: ./restore.sh [--yes]\n\n{HELPERS}\n",
        h.repot_version,
        h.generated_at,
        os_label(h.source),
        os_label(h.target)
    );
    let kernel = if h.target == Os::MacOs {
        "Darwin"
    } else {
        "Linux"
    };
    writeln!(out, "if [ \"$(uname -s)\" != {} ]; then echo 'This script targets a different OS.'; exit 1; fi\necho 'This script will:'", q(kernel)).unwrap();
    for s in &plan.steps {
        writeln!(
            out,
            "printf '%s\\n' {}",
            q(&format!("  - {}", s.label(h.target)))
        )
        .unwrap();
    }
    out.push_str("if [ \"${1:-}\" != '--yes' ]; then\n  printf 'Continue? [y/N] '\n  read -r answer\n  case \"$answer\" in y|Y) ;; *) exit 0 ;; esac\nfi\n");
    let mut i = 0;
    let mut section = None;
    while i < plan.steps.len() {
        let s = &plan.steps[i];
        if section != Some(s.section()) {
            section = Some(s.section());
            writeln!(
                out,
                "\necho {}",
                q(&format!("== {} ==", section_name(s.section())))
            )
            .unwrap();
            if s.section() == Section::Runtimes {
                if plan.steps.iter().any(|s| {
                    matches!(
                        s,
                        Step::InstallRuntime {
                            tool: RuntimeTool::Nvm,
                            ..
                        }
                    )
                }) {
                    out.push_str("export NVM_DIR=\"${NVM_DIR:-$HOME/.nvm}\"; [ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\"\n");
                }
                if plan.steps.iter().any(|s| {
                    matches!(
                        s,
                        Step::InstallRuntime {
                            tool: RuntimeTool::Sdkman,
                            ..
                        }
                    )
                }) {
                    out.push_str("export SDKMAN_DIR=\"${SDKMAN_DIR:-$HOME/.sdkman}\"; [ -s \"$SDKMAN_DIR/bin/sdkman-init.sh\" ] && . \"$SDKMAN_DIR/bin/sdkman-init.sh\"\n");
                }
            }
        }
        if let Step::CopyFile { src, dest } = s {
            let label = q(&s.label(h.target));
            writeln!(out, "copy_file {} {}; rc=$?\nif [ $rc -eq 0 ]; then record_ok {label}\nelif [ $rc -eq 2 ]; then record_skip {label}\nelse record_fail {label}; fi", q(src), bash_path(dest, h.target)).unwrap();
            i += 1;
            continue;
        }
        let t = tool(s).unwrap();
        let start = i;
        while i < plan.steps.len()
            && plan.steps[i].section() == s.section()
            && tool(&plan.steps[i]) == Some(t)
        {
            i += 1;
        }
        writeln!(
            out,
            "if have {}{}; then",
            q(t.0),
            if t.1 == "oh-my-zsh" {
                " && [ -d \"$HOME/.oh-my-zsh\" ]"
            } else {
                ""
            }
        )
        .unwrap();
        for step in &plan.steps[start..i] {
            let (check, install) = commands(step, h.target);
            let label = q(&step.label(h.target));
            writeln!(out, "  if {check} >/dev/null 2>&1; then record_skip {label}\n  elif {install}; then record_ok {label}\n  else record_fail {label}; fi").unwrap();
        }
        writeln!(out, "else\n  missing {} {}", q(t.1), q(guide_url(t.1))).unwrap();
        for step in &plan.steps[start..i] {
            writeln!(out, "  record_skip {}", q(&step.label(h.target))).unwrap();
        }
        out.push_str("fi\n");
    }
    out.push_str("\nprintf '\\nSummary\\nOK:\\n%sSkipped:\\n%sFailed:\\n%s' \"$OK_LIST\" \"$SKIP_LIST\" \"$FAIL_LIST\"\n");
    if !plan.masked.is_empty() {
        out.push_str("echo 'Fill in these masked secrets:'\n");
        for m in &plan.masked {
            writeln!(
                out,
                "printf '%s\\n' {}",
                q(&format!("  - {}", masked_text(m)))
            )
            .unwrap();
        }
    }
    if !plan.notes.is_empty() {
        out.push_str("echo 'Notes:'\n");
        for n in &plan.notes {
            writeln!(out, "printf '%s\\n' {}", q(&note_text(n))).unwrap();
        }
    }
    out.push_str("exit $FAILED\n");
    out
}
