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
$script:Ok = @(); $script:Done = @(); $script:NotDone = @(); $script:Failed = @(); $script:Tools = @()
function Test-Tool($Name) { [bool](Get-Command $Name -ErrorAction SilentlyContinue) }
function Add-Ok($Label) { $script:Ok += $Label; Write-Host "  ok: $Label" }
# Already installed or identical: nothing to do.
function Add-Done($Label) { $script:Done += $Label; Write-Host "  already done: $Label" }
# Not attempted; $Reason says why and what the user can do about it.
function Add-NotDone($Label, $Reason) { $script:NotDone += "$Label ($Reason)"; Write-Host "  not done: $Label ($Reason)" }
function Add-Fail($Label) { $script:Failed += $Label; Write-Host "  FAILED: $Label" }
function Write-Missing($Name, $Url) {
  Write-Host "  $Name not found. Install it first: $Url"
  if ($script:Tools -notcontains "${Name}: $Url") { $script:Tools += "${Name}: $Url" }
}
# Preflight: report whether a tool is here before anything runs.
$script:PreflightMissing = $false
function Test-Preflight($Name, $Command, $Url) {
  if (Test-Tool $Command) { Write-Host "  ${Name}: found" }
  else { Write-Host "  ${Name}: not found yet. Install it from $Url"; $script:PreflightMissing = $true }
}
function Test-Admin { ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator) }
# Byte comparison instead of Get-FileHash: Get-FileHash lives in a script module that fails to
# load in Windows PowerShell 5.1 when it is started from PowerShell 7 (inherited PSModulePath).
function Test-SameFile($A, $B) {
  $x = [System.IO.File]::ReadAllBytes($A); $y = [System.IO.File]::ReadAllBytes($B)
  if ($x.Length -ne $y.Length) { return $false }
  for ($i = 0; $i -lt $x.Length; $i++) { if ($x[$i] -ne $y[$i]) { return $false } }
  return $true
}
function Copy-RepotFile($Src, $Dest) {
  $from = Join-Path $Dir (Join-Path 'files' $Src)
  if (-not (Test-Path $from)) { return 1 }
  if ((Test-Path $Dest) -and (Test-SameFile $from $Dest)) { return 2 }
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
        Step::InstallIdePlugin { ide, .. } => Some((ide.command(), ide.product())),
        Step::CopyFile { .. } | Step::GitClone { .. } => None,
    }
}
// Output checks are PowerShell expressions; exit-code checks are commands.
fn commands(s: &Step) -> (String, bool, String) {
    match s {
        // `scoop bucket list` returns objects: compare the Name column, not the printed
        // table, so `java` does not match `java-extra` or a URL that contains it.
        Step::AddTap { name, url, .. } => (
            format!(
                "@((& scoop bucket list 2>$null).Name) -contains {}",
                q(name)
            ),
            true,
            match url {
                Some(url) => format!("& scoop bucket add {} {}", q(name), q(url)),
                None => format!("& scoop bucket add {}", q(name)),
            },
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
            components,
            targets,
        } => {
            let v = q(version);
            if *tool == RuntimeTool::Rustup {
                return rustup_commands(version, *set_default, components, targets);
            }
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
                _ => return ("$false".into(), true, "$global:LASTEXITCODE = 1".into()),
            };
            // PS 5.1 has no &&. Do not let a successful default command hide install failure.
            if *set_default && !default.is_empty() {
                write!(install, "; if ($LASTEXITCODE -eq 0) {{ {default} }}").unwrap();
            }
            // Whole words of the list, so 20.1.1 does not match an installed 20.1.10.
            (
                format!("@(& {check} 2>$null | ForEach-Object {{ -split $_ }}) -contains {value}"),
                true,
                install,
            )
        }
        Step::InstallGlobal { manager, name } => {
            let c = global_cmd(*manager);
            let n = q(name);
            // Same checks as the Bash script: npm by its exit code, the others by the
            // first word of each line (`<name> <version>`), so `eslint` does not match
            // `eslint-plugin-react`.
            let (check, action) = match manager {
                GlobalManager::Npm => {
                    return (
                        format!("npm ls -g --depth=0 {n}"),
                        false,
                        format!("& npm install -g {n}"),
                    );
                }
                GlobalManager::Pnpm => ("ls -g --depth=0", "add -g"),
                GlobalManager::Pipx => ("list --short", "install"),
                GlobalManager::Cargo => ("install --list", "install"),
            };
            (
                format!(
                    "@(& {c} {check} 2>$null | ForEach-Object {{ ($_ -split ' ')[0] }}) -contains {n}"
                ),
                true,
                format!("& {c} {action} {n}"),
            )
        }
        Step::InstallExtension { editor, id } => {
            let c = editor_cmd(*editor);
            let id = q(id);
            (
                format!("@(& {c} --list-extensions 2>$null) -contains {id}"),
                true,
                format!("& {c} --install-extension {id}"),
            )
        }
        Step::InstallIdePlugin { ide, id, folder } => (
            format!(
                "Test-Path (Join-Path (Get-JetBrainsPluginDir {}) {})",
                q(ide.prefix()),
                q(folder)
            ),
            true,
            format!("& $JbCli installPlugins {}", q(id)),
        ),
        _ => unreachable!("file steps use their own return codes"),
    }
}

fn ps_list(names: &[String]) -> String {
    let names: Vec<String> = names.iter().map(|n| q(n)).collect();
    format!("@({})", names.join(", "))
}

/// Same behavior as the Bash script: the check needs every component and target, and
/// the toolchain is installed only if missing (installing an existing channel updates it).
fn rustup_commands(
    version: &str,
    set_default: bool,
    components: &[String],
    targets: &[String],
) -> (String, bool, String) {
    let v = q(version);
    let check = format!(
        "Test-RustupToolchain {v} {} {}",
        ps_list(components),
        ps_list(targets)
    );
    let mut install = if components.is_empty() && targets.is_empty() {
        format!("& rustup toolchain install {v}")
    } else {
        format!(
            "$hasToolchain = Test-RustupToolchain {v} @() @(); if (-not $hasToolchain) {{ & rustup toolchain install {v} }}"
        )
    };
    // PS 5.1 has no &&: each command runs only after the previous one succeeded.
    let mut first = true;
    for (kind, names) in [("component", components), ("target", targets)] {
        if !names.is_empty() {
            let names: Vec<String> = names.iter().map(|n| q(n)).collect();
            let condition = if first {
                "$hasToolchain -or $LASTEXITCODE -eq 0"
            } else {
                "$LASTEXITCODE -eq 0"
            };
            first = false;
            write!(
                install,
                "; if ({condition}) {{ & rustup {kind} add --toolchain {v} {} }}",
                names.join(" ")
            )
            .unwrap();
        }
    }
    if set_default {
        write!(
            install,
            "; if ($LASTEXITCODE -eq 0) {{ & rustup default {v} }}"
        )
        .unwrap();
    }
    (check, true, install)
}

/// `Test-RustupToolchain`, only when the plan has rustup steps.
fn write_rustup_helper(out: &mut String, plan: &ScriptPlan) {
    if !plan.steps.iter().any(|s| {
        matches!(
            s,
            Step::InstallRuntime {
                tool: RuntimeTool::Rustup,
                ..
            }
        )
    }) {
        return;
    }
    out.push_str(
        r#"# The toolchain is installed with every listed component and target.
# rustup prints host-specific components as <name>-<host>, so the host suffix is dropped.
# "<channel>-<letter>" keeps "nightly" from matching a dated nightly.
function Test-RustupToolchain($Toolchain, $Components, $Targets) {
  $pattern = '^' + [regex]::Escape($Toolchain) + '-[a-z]'
  $line = @(& rustup toolchain list 2>$null | Where-Object { $_ -cmatch $pattern })
  if ($line.Count -eq 0) { return $false }
  $name = ($line[0] -split ' ')[0]
  $suffix = $name.Substring($Toolchain.Length)
  $haveComponents = @(& rustup component list --installed --toolchain $name 2>$null | ForEach-Object { if ($_.EndsWith($suffix)) { $_.Substring(0, $_.Length - $suffix.Length) } else { $_ } })
  $haveTargets = @(& rustup target list --installed --toolchain $name 2>$null)
  foreach ($c in $Components) { if ($haveComponents -cnotcontains $c) { return $false } }
  foreach ($t in $Targets) { if ($haveTargets -cnotcontains $t) { return $false } }
  return $true
}
"#,
    );
}

/// The JetBrains IDE a tool tuple stands for, if any.
fn jetbrains_ide(tool: (&str, &str)) -> Option<JetBrainsIde> {
    JetBrainsIde::ALL
        .into_iter()
        .find(|ide| (ide.command(), ide.product()) == tool)
}

/// `Find-JetBrainsCli 'idea' @('IntelliJ IDEA 20*', …)`: the IDE's launcher or `$null`.
fn jb_cli_call(ide: JetBrainsIde) -> String {
    let dirs: Vec<String> = ide.windows_dirs().iter().map(|d| q(d)).collect();
    format!(
        "Find-JetBrainsCli {} @({})",
        q(ide.command()),
        dirs.join(", ")
    )
}

/// Helpers for JetBrains steps and files, only when the plan has some (plan 14).
fn write_jetbrains_helpers(out: &mut String, plan: &ScriptPlan) {
    let used = plan.steps.iter().any(|s| match s {
        Step::InstallIdePlugin { .. } => true,
        Step::CopyFile { dest, .. } => matches!(dest, TargetPath::JetBrains { .. }),
        _ => false,
    });
    if !used {
        return;
    }
    out.push_str(r#"$JbBase = Join-Path $env:APPDATA 'JetBrains'
# The IDE's newest settings folder here, or the source machine's version if it never ran.
function Get-JetBrainsConfig($Prefix, $Version) {
  $pattern = '^' + [regex]::Escape($Prefix) + '\d{4}\.\d{1,2}$'
  $best = Get-ChildItem -Path $JbBase -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match $pattern } |
    Sort-Object { $v = $_.Name.Substring($Prefix.Length).Split('.'); [int]$v[0] * 100 + [int]$v[1] } |
    Select-Object -Last 1
  if ($best) { $best.FullName } else { Join-Path $JbBase "$Prefix$Version" }
}
function Get-JetBrainsPluginDir($Prefix) { Join-Path (Get-JetBrainsConfig $Prefix '') 'plugins' }
function Find-JetBrainsCli($Command, $Dirs) {
  $found = Get-Command $Command -ErrorAction SilentlyContinue
  if ($found) { return $found.Source }
  $toolbox = Join-Path $env:LOCALAPPDATA "JetBrains\Toolbox\scripts\$Command.cmd"
  if (Test-Path $toolbox) { return $toolbox }
  foreach ($root in @((Join-Path $env:ProgramFiles 'JetBrains'), (Join-Path $env:LOCALAPPDATA 'Programs'))) {
    foreach ($dir in $Dirs) {
      $match = Get-ChildItem -Path (Join-Path $root $dir) -Directory -ErrorAction SilentlyContinue | Sort-Object Name | Select-Object -Last 1
      if ($match) {
        $exe = Join-Path $match.FullName "bin\$($Command)64.exe"
        if (Test-Path $exe) { return $exe }
      }
    }
  }
  return $null
}
"#);
}

/// Lists every tool the steps need, once, before the user is asked to continue.
fn write_preflight(out: &mut String, plan: &ScriptPlan) {
    let mut tools: Vec<(&str, &str)> = vec![];
    for t in plan.steps.iter().filter_map(tool) {
        if !tools.contains(&t) {
            tools.push(t);
        }
    }
    if tools.is_empty() {
        return;
    }
    out.push_str("Write-Host 'Tools this script uses:'\n");
    for t in tools {
        if let Some(ide) = jetbrains_ide(t) {
            writeln!(
                out,
                "if ({}) {{ Write-Host {} }} else {{ Write-Host {}; $script:PreflightMissing = $true }}",
                jb_cli_call(ide),
                q(&format!("  {}: found", t.1)),
                q(&format!("  {}: not found yet. Install it from {}", t.1, guide_url(t.1)))
            )
            .unwrap();
            continue;
        }
        writeln!(
            out,
            "Test-Preflight {} {} {}",
            q(t.1),
            q(t.0),
            q(guide_url(t.1))
        )
        .unwrap();
        if t.0 == "choco" {
            out.push_str("if ((Test-Tool 'choco') -and -not (Test-Admin)) { Write-Host '  choco needs an Administrator PowerShell; choco steps will be listed under \"Not done\".' }\n");
        }
    }
    out.push_str("if ($script:PreflightMissing) { Write-Host 'Steps that need a missing tool will be listed under \"Not done\". An earlier step may install some of them; otherwise install them and run this script again.' }\n");
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
    write_jetbrains_helpers(&mut out, plan);
    write_rustup_helper(&mut out, plan);
    write_preflight(&mut out, plan);
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
            writeln!(out, "$rc = Copy-RepotFile {} {}\nif ($rc -eq 0) {{ Add-Ok {label} }} elseif ($rc -eq 2) {{ Add-Done {label} }} else {{ Add-Fail {label} }}", q(src), ps_path(dest)).unwrap();
            i += 1;
            continue;
        }
        let Some(t) = tool(s) else {
            writeln!(out, "Add-NotDone {label} 'not supported on Windows'").unwrap();
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
                writeln!(
                    out,
                    "  Add-NotDone {} 'run PowerShell as Administrator'",
                    q(&step.label(h.target))
                )
                .unwrap();
            }
            out.push_str("} else {\n");
        }
        if let Some(ide) = jetbrains_ide(t) {
            writeln!(out, "$JbCli = {}\nif ($JbCli) {{", jb_cli_call(ide)).unwrap();
            writeln!(
                out,
                "  Write-Host {}",
                q(&format!(
                    "  Close {} first: plugins cannot be installed while it is open.",
                    t.1
                ))
            )
            .unwrap();
        } else {
            writeln!(out, "if (Test-Tool {}) {{", q(t.0)).unwrap();
        }
        for step in group {
            if let Some(origin) = step.mapped_origin() {
                writeln!(out, "  # from {origin}").unwrap();
            }
            let (check, output_check, install) = commands(step);
            let label = q(&step.label(h.target));
            // Some managers exit 0 without installing anything (choco 2.x when its feed
            // returns 504), so a package counts as installed only if the check finds it.
            let verify = if output_check {
                format!("$installed = {check}; if ($installed)")
            } else {
                format!("& {check} *> $null; if ($LASTEXITCODE -eq 0)")
            };
            if output_check {
                writeln!(
                    out,
                    "  $installed = {check}\n  if ($installed) {{ Add-Done {label} }}"
                )
                .unwrap();
            } else {
                writeln!(
                    out,
                    "  & {check} *> $null\n  if ($LASTEXITCODE -eq 0) {{ Add-Done {label} }}"
                )
                .unwrap();
            }
            if matches!(step, Step::InstallPackage { .. }) {
                writeln!(out, "  else {{ {install}; if ($LASTEXITCODE -eq 0) {{ {verify} {{ Add-Ok {label} }} else {{ Write-Host '  installer reported success, but the package is not installed'; Add-Fail {label} }} }} else {{ Add-Fail {label} }} }}").unwrap();
            } else {
                writeln!(out, "  else {{ {install}; if ($LASTEXITCODE -eq 0) {{ Add-Ok {label} }} else {{ Add-Fail {label} }} }}").unwrap();
            }
        }
        writeln!(
            out,
            "}} else {{\n  Write-Missing {} {}",
            q(t.1),
            q(guide_url(t.1))
        )
        .unwrap();
        let reason = q(&format!("{} not found", t.1));
        for step in group {
            writeln!(out, "  Add-NotDone {} {reason}", q(&step.label(h.target))).unwrap();
        }
        out.push_str("}\n");
        if t.0 == "choco" {
            out.push_str("}\n");
        }
    }
    out.push_str("\nWrite-Host 'Summary'\nWrite-Host 'OK:'; $script:Ok | ForEach-Object { Write-Host \"  - $_\" }\nWrite-Host 'Already done:'; $script:Done | ForEach-Object { Write-Host \"  - $_\" }\nWrite-Host 'Not done:'; $script:NotDone | ForEach-Object { Write-Host \"  - $_\" }\nif ($script:Tools.Count -gt 0) { Write-Host 'To finish these, install the missing tools and run this script again:'; $script:Tools | ForEach-Object { Write-Host \"  $_\" } }\nWrite-Host 'Failed:'; $script:Failed | ForEach-Object { Write-Host \"  - $_\" }\n");
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
