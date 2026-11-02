use super::*;
use crate::export::{
    paths::{os_label, ps_path, ps_quote as q},
    step::guide_url,
};
use std::fmt::Write;

const HELPERS: &str = r#"param([switch]$Yes)
$ErrorActionPreference = 'Continue'
$Dir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Stamp = Get-Date -Format 'yyyyMMddHHmmss'
$script:Ok = @(); $script:Skipped = @(); $script:Failed = @()
function Test-Have($Name) { [bool](Get-Command $Name -ErrorAction SilentlyContinue) }
function Add-Ok($Label) { $script:Ok += $Label; Write-Host "  ok: $Label" }
function Add-Skip($Label) { $script:Skipped += $Label; Write-Host "  skipped: $Label" }
function Add-Fail($Label) { $script:Failed += $Label; Write-Host "  FAILED: $Label" }
function Write-Missing($Name, $Url) { Write-Host "  $Name not found. Install it first: $Url" }
function Test-Admin { ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator) }
function Copy-RepotFile($Src, $Dest) {
  $from = Join-Path $Dir (Join-Path 'files' $Src)
  if (-not (Test-Path $from)) { return 1 }
  if ((Test-Path $Dest) -and ((Get-FileHash $from).Hash -eq (Get-FileHash $Dest).Hash)) { return 2 }
  try {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Dest) | Out-Null
    if (Test-Path $Dest) { Copy-Item $Dest "$Dest.repot-backup-$Stamp" -ErrorAction Stop }
    Copy-Item $from $Dest -ErrorAction Stop
    return 0
  } catch { return 1 }
}
"#;
fn tool(s: &Step) -> Option<(&'static str, &'static str)> {
    match s {
        Step::AddTap { manager, .. } | Step::InstallPackage { manager, .. } => {
            Some((package_cmd(*manager), package_cmd(*manager)))
        }
        Step::InstallRuntime { tool, .. } => Some((runtime_cmd(*tool), runtime_name(*tool))),
        Step::InstallGlobal { manager, .. } => Some((global_cmd(*manager), global_cmd(*manager))),
        Step::InstallExtension { editor, .. } => Some((editor_cmd(*editor), editor_cmd(*editor))),
        Step::CopyFile { .. } | Step::GitClone { .. } => None,
    }
}
// Output checks are PowerShell expressions; exit-code checks are commands.
fn commands(s: &Step) -> (String, bool, String) {
    match s {
        Step::AddTap { name, .. } => (
            format!(
                "& scoop bucket list 2>$null | Select-String -SimpleMatch {} -Quiet",
                q(name)
            ),
            true,
            format!("& scoop bucket add {}", q(name)),
        ),
        Step::InstallPackage { manager, name, .. } => {
            let n = q(name);
            match manager {
                PackageManager::Winget => (
                    format!("winget list --id {n} --exact"),
                    false,
                    format!(
                        "& winget install --id {n} --exact --accept-package-agreements --accept-source-agreements"
                    ),
                ),
                PackageManager::Scoop => (
                    format!("scoop prefix {n}"),
                    false,
                    format!("& scoop install {n}"),
                ),
                PackageManager::Choco => (
                    format!(
                        "& choco list --exact {n} --limit-output 2>$null | Select-String -SimpleMatch {n} -Quiet"
                    ),
                    true,
                    format!("& choco install {n} -y"),
                ),
                _ => ("$false".into(), true, "$global:LASTEXITCODE = 1".into()),
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
            let (check, value, mut install, default) = match tool {
                RuntimeTool::NvmWindows => (
                    "nvm list".into(),
                    v.clone(),
                    format!("& nvm install {v}"),
                    format!("& nvm use {v}"),
                ),
                RuntimeTool::Fnm => (
                    "fnm list".into(),
                    q(&format!("v{version}")),
                    format!("& fnm install {v}"),
                    format!("& fnm default {v}"),
                ),
                RuntimeTool::Volta => (
                    format!("volta list {r} --format plain"),
                    q(&format!("{runtime}@{version}")),
                    format!("& volta install {}", q(&format!("{runtime}@{version}"))),
                    String::new(),
                ),
                RuntimeTool::PyenvWin => (
                    "pyenv versions --bare".into(),
                    v.clone(),
                    format!("& pyenv install {v}"),
                    format!("& pyenv global {v}"),
                ),
                RuntimeTool::Rustup => (
                    "rustup toolchain list".into(),
                    v.clone(),
                    format!("& rustup toolchain install {v}"),
                    format!("& rustup default {v}"),
                ),
                _ => return ("$false".into(), true, "$global:LASTEXITCODE = 1".into()),
            };
            // PS 5.1 has no &&. Do not let a successful default command hide install failure.
            if *set_default && !default.is_empty() {
                write!(install, "; if ($LASTEXITCODE -eq 0) {{ {default} }}").unwrap();
            }
            (
                format!("& {check} 2>$null | Select-String -SimpleMatch {value} -Quiet"),
                true,
                install,
            )
        }
        Step::InstallGlobal { manager, name } => {
            let c = global_cmd(*manager);
            let n = q(name);
            let (check, action) = match manager {
                GlobalManager::Npm => ("ls -g --depth=0", "install -g"),
                GlobalManager::Pnpm => ("ls -g --depth=0", "add -g"),
                GlobalManager::Pipx => ("list --short", "install"),
                GlobalManager::Cargo => ("install --list", "install"),
            };
            (
                format!("& {c} {check} 2>$null | Select-String -SimpleMatch {n} -Quiet"),
                true,
                format!("& {c} {action} {n}"),
            )
        }
        Step::InstallExtension { editor, id } => {
            let c = editor_cmd(*editor);
            let id = q(id);
            (
                format!("& {c} --list-extensions 2>$null | Select-String -SimpleMatch {id} -Quiet"),
                true,
                format!("& {c} --install-extension {id}"),
            )
        }
        _ => unreachable!("file steps use their own return codes"),
    }
}

pub fn render_powershell(plan: &ScriptPlan) -> String {
    let h = &plan.header;
    let mut out = format!(
        "# Generated by Repot {} on {}\n# Source: {} -> Target: Windows\n# Review this script before running it. Usage: powershell -ExecutionPolicy Bypass -File restore.ps1 [-Yes]\n{HELPERS}\n",
        h.repot_version,
        h.generated_at,
        os_label(h.source)
    );
    out.push_str("if ($env:OS -ne 'Windows_NT') { Write-Host 'This script targets Windows.'; exit 1 }\nWrite-Host 'This script will:'\n");
    for s in &plan.steps {
        writeln!(
            out,
            "Write-Host {}",
            q(&format!("  - {}", s.label(h.target)))
        )
        .unwrap();
    }
    out.push_str("if (-not $Yes) { $answer = Read-Host 'Continue? [y/N]'; if ($answer -cne 'y' -and $answer -cne 'Y') { exit 0 } }\n");
    let mut i = 0;
    let mut section = None;
    while i < plan.steps.len() {
        let s = &plan.steps[i];
        if section != Some(s.section()) {
            section = Some(s.section());
            writeln!(
                out,
                "\nWrite-Host {}",
                q(&format!("== {} ==", section_name(s.section())))
            )
            .unwrap();
        }
        let label = q(&s.label(h.target));
        if let Step::CopyFile { src, dest } = s {
            writeln!(out, "$rc = Copy-RepotFile {} {}\nif ($rc -eq 0) {{ Add-Ok {label} }} elseif ($rc -eq 2) {{ Add-Skip {label} }} else {{ Add-Fail {label} }}", q(src), ps_path(dest)).unwrap();
            i += 1;
            continue;
        }
        let Some(t) = tool(s) else {
            writeln!(out, "Add-Skip {label}").unwrap();
            i += 1;
            continue;
        };
        let start = i;
        while i < plan.steps.len()
            && plan.steps[i].section() == s.section()
            && tool(&plan.steps[i]) == Some(t)
        {
            i += 1;
        }
        let group = &plan.steps[start..i];
        if t.0 == "choco" {
            out.push_str("if (-not (Test-Admin)) {\n  Write-Host '  choco requires an Administrator PowerShell. Skipping choco packages.'\n");
            for step in group {
                writeln!(out, "  Add-Skip {}", q(&step.label(h.target))).unwrap();
            }
            out.push_str("} else {\n");
        }
        writeln!(out, "if (Test-Have {}) {{", q(t.0)).unwrap();
        for step in group {
            let (check, output_check, install) = commands(step);
            let label = q(&step.label(h.target));
            if output_check {
                writeln!(out, "  if ({check}) {{ Add-Skip {label} }}").unwrap();
            } else {
                writeln!(
                    out,
                    "  & {check} *> $null\n  if ($LASTEXITCODE -eq 0) {{ Add-Skip {label} }}"
                )
                .unwrap();
            }
            writeln!(out, "  else {{ {install}; if ($LASTEXITCODE -eq 0) {{ Add-Ok {label} }} else {{ Add-Fail {label} }} }}").unwrap();
        }
        writeln!(
            out,
            "}} else {{\n  Write-Missing {} {}",
            q(t.1),
            q(guide_url(t.1))
        )
        .unwrap();
        for step in group {
            writeln!(out, "  Add-Skip {}", q(&step.label(h.target))).unwrap();
        }
        out.push_str("}\n");
        if t.0 == "choco" {
            out.push_str("}\n");
        }
    }
    out.push_str("\nWrite-Host 'Summary'\nWrite-Host 'OK:'; $script:Ok | ForEach-Object { Write-Host \"  - $_\" }\nWrite-Host 'Skipped:'; $script:Skipped | ForEach-Object { Write-Host \"  - $_\" }\nWrite-Host 'Failed:'; $script:Failed | ForEach-Object { Write-Host \"  - $_\" }\n");
    if !plan.masked.is_empty() {
        out.push_str("Write-Host 'Fill in these masked secrets:'\n");
        for m in &plan.masked {
            writeln!(out, "Write-Host {}", q(&format!("  - {}", masked_text(m)))).unwrap();
        }
    }
    if !plan.notes.is_empty() {
        out.push_str("Write-Host 'Notes:'\n");
        for n in &plan.notes {
            writeln!(out, "Write-Host {}", q(&note_text(n))).unwrap();
        }
    }
    out.push_str("if ($script:Failed.Count -gt 0) { exit 1 } else { exit 0 }\n");
    out
}
