//! The commands. Everything they judge comes from core (`session`); this file decides
//! what to print and which exit code to return.

use crate::args::{Cli, Command, ExportArgs};
use crate::text;
use repot_core::availability::{Availability, UnavailableReason};
use repot_core::compare::Presence;
use repot_core::env::Env;
use repot_core::export::{ExportBundle, ExportMeta, write_export};
use repot_core::os::Os;
use repot_core::scan::{ProviderReport, ProviderStatus, ScanResult};
use repot_core::session::{OpenError, Session};
use repot_core::snapshot::SnapshotError;
use repot_core::{
    Category, FindingKind, Item, PROVIDER_TIMEOUT, Selection, Target, default_providers,
};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

/// What the commands need from the outside world, so tests can pass fakes.
pub struct Context<'a> {
    pub env: Arc<dyn Env>,
    /// The OS this runs on: the source of a scan and the only OS `compare` can judge.
    pub this_os: Os,
    pub meta: ExportMeta,
    pub on_progress: &'a (dyn Fn(&ProviderReport) + Sync),
    pub out: &'a mut dyn Write,
    pub err: &'a mut dyn Write,
}

/// A failure with the message shown after `error:`; exit code 1.
struct Failure(String);
impl From<std::io::Error> for Failure {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}
type Outcome = Result<(), Failure>;
fn fail<T>(message: impl Into<String>) -> Result<T, Failure> {
    Err(Failure(message.into()))
}

/// Runs one command and returns the process exit code (0 or 1; clap uses 2).
pub fn run(cli: Cli, ctx: &mut Context) -> i32 {
    let result = match cli.command {
        Command::Scan { json } => scan_command(ctx, json),
        Command::Compare { file, json } => compare_command(ctx, &file, json),
        Command::Export(args) => export_command(ctx, &args),
    };
    match result {
        Ok(()) => 0,
        Err(Failure(message)) => {
            let _ = writeln!(ctx.err, "error: {message}");
            1
        }
    }
}

fn scan_this_machine(ctx: &Context) -> ScanResult {
    repot_core::scan(
        Arc::clone(&ctx.env),
        default_providers(),
        PROVIDER_TIMEOUT,
        ctx.on_progress,
    )
}

fn open_file(ctx: &mut Context, path: &Path) -> Result<Session, Failure> {
    let shown = path.display();
    match Session::open(ctx.env.as_ref(), path) {
        Ok((session, info)) => {
            if info.skipped_items > 0 {
                let _ = writeln!(
                    ctx.err,
                    "warning: skipped {} items this version of Repot does not understand",
                    info.skipped_items
                );
            }
            Ok(session)
        }
        Err(OpenError::ReadFailed) => fail(format!("could not read {shown}")),
        Err(OpenError::Snapshot(SnapshotError::TooLarge)) => {
            fail(format!("{shown} is too large to be a Repot file"))
        }
        Err(OpenError::Snapshot(SnapshotError::NotRepotFile)) => {
            fail(format!("{shown} is not a Repot file"))
        }
        Err(OpenError::Snapshot(SnapshotError::NewerVersion { found })) => fail(format!(
            "{shown} was written by a newer Repot (file version {found}); update Repot"
        )),
        Err(OpenError::Snapshot(SnapshotError::Invalid)) => {
            fail(format!("{shown} is damaged or was edited by hand"))
        }
    }
}

fn label(item: &Item) -> String {
    match &item.version {
        Some(version) => format!("{} {version}", item.name),
        None => item.name.clone(),
    }
}

/// Width of the label column in `scan` and `compare` lists.
const LABEL_WIDTH: usize = 40;

/// One list line: the label padded to [`LABEL_WIDTH`], then the id. A longer label (usually a
/// Homebrew cask's `version,commit`) is cut with `…` so every id starts in the same column; the
/// id itself is never cut, and `--json` keeps the whole version.
fn item_line(item: &Item) -> String {
    let mut label = label(item);
    if label.chars().count() > LABEL_WIDTH {
        label = label.chars().take(LABEL_WIDTH - 1).collect();
        label.push('…');
    }
    format!("  {label:<LABEL_WIDTH$} {}", item.id)
}

fn scan_command(ctx: &mut Context, json: bool) -> Outcome {
    let result = scan_this_machine(ctx);
    if json {
        serde_json::to_writer_pretty(&mut *ctx.out, &result).map_err(|e| Failure(e.to_string()))?;
        writeln!(ctx.out)?;
        return Ok(());
    }
    writeln!(
        ctx.out,
        "{}: {} items",
        text::os(ctx.this_os),
        result.items.len()
    )?;
    for category in [
        Category::PackageManager,
        Category::Runtime,
        Category::GlobalPackage,
        Category::Editor,
        Category::Git,
        Category::Shell,
        Category::Ssh,
    ] {
        let items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.category == category)
            .collect();
        if items.is_empty() {
            continue;
        }
        writeln!(ctx.out, "\n{} ({})", text::category(category), items.len())?;
        for item in items {
            writeln!(ctx.out, "{}", item_line(item))?;
        }
    }
    let missing: Vec<_> = result
        .providers
        .iter()
        .filter(|p| p.status == ProviderStatus::NotDetected)
        .map(|p| p.id.as_str())
        .collect();
    if !missing.is_empty() {
        writeln!(ctx.out, "\nNot installed: {}", missing.join(", "))?;
    }
    for p in &result.providers {
        if matches!(
            p.status,
            ProviderStatus::Failed { .. } | ProviderStatus::TimedOut
        ) {
            writeln!(
                ctx.err,
                "warning: {} {}",
                p.id,
                text::provider_status(&p.status)
            )?;
        }
    }
    Ok(())
}

/// Opens `file`, scans this machine and judges each item for this machine's OS.
fn compared(
    ctx: &mut Context,
    file: &Path,
) -> Result<(Session, HashMap<String, Presence>), Failure> {
    let mut session = open_file(ctx, file)?;
    session.machine = Some(scan_this_machine(ctx));
    let presence = session.presence(
        ctx.env.as_ref(),
        ctx.this_os,
        Target::default_for(ctx.this_os),
    );
    Ok((session, presence))
}

fn compare_command(ctx: &mut Context, file: &Path, json: bool) -> Outcome {
    let (session, presence) = compared(ctx, file)?;
    if json {
        let sorted: BTreeMap<_, _> = presence.iter().collect();
        serde_json::to_writer_pretty(&mut *ctx.out, &sorted).map_err(|e| Failure(e.to_string()))?;
        writeln!(ctx.out)?;
        return Ok(());
    }
    writeln!(
        ctx.out,
        "{} (from {}) compared with this machine",
        file.display(),
        text::os(session.source)
    )?;
    let of = |state: Presence| -> Vec<&Item> {
        session
            .scan
            .items
            .iter()
            .filter(|i| presence.get(&i.id) == Some(&state))
            .collect()
    };
    let here = of(Presence::Present);
    writeln!(ctx.out, "\nAlready here: {}", here.len())?;
    for (title, state) in [
        ("Different here", Presence::Differs),
        ("Missing", Presence::Missing),
    ] {
        let items = of(state);
        writeln!(ctx.out, "\n{title}: {}", items.len())?;
        for item in items {
            writeln!(ctx.out, "{}", item_line(item))?;
        }
    }
    let unusable = session
        .scan
        .items
        .iter()
        .filter(|i| !presence.contains_key(&i.id))
        .count();
    if unusable > 0 {
        writeln!(
            ctx.out,
            "\nNot usable on {}: {unusable} (not compared)",
            text::os(ctx.this_os)
        )?;
    }
    Ok(())
}

fn target_of(ctx: &Context, args: &ExportArgs) -> Result<Target, Failure> {
    let os = args.to.map_or(ctx.this_os, |t| t.os());
    let mut target = Target::default_for(os);
    if let Some(packages) = args.packages {
        target.packages = Some(packages.manager());
    }
    if target.validate().is_err() {
        return fail(format!(
            "--packages {} cannot be used for {}",
            text::manager(target.packages.expect("set above")),
            text::os(os)
        ));
    }
    Ok(target)
}

/// Item ids the export starts with, after `--include` and `--exclude`.
fn selection_of(
    session: &Session,
    target: Target,
    presence: Option<&HashMap<String, Presence>>,
    args: &ExportArgs,
) -> Result<Vec<String>, Failure> {
    for id in args.include.iter().chain(&args.exclude) {
        if !session.scan.items.iter().any(|i| &i.id == id) {
            return fail(format!(
                "no item with id {id}; `repot scan` or `repot compare` shows the ids"
            ));
        }
    }
    let mut selected = session.default_selection(target, presence);
    for id in &args.include {
        if !selected.contains(id) {
            selected.push(id.clone());
        }
    }
    selected.retain(|id| !args.exclude.contains(id));
    if selected.is_empty() {
        return fail("nothing to export: no item is selected");
    }
    Ok(selected)
}

fn export_command(ctx: &mut Context, args: &ExportArgs) -> Outcome {
    let target = target_of(ctx, args)?;
    // Before a scan that can take a while.
    if !args.dry_run && !args.out.is_dir() {
        return fail(format!("{} is not a folder", args.out.display()));
    }
    let mut session = match &args.file {
        Some(file) => open_file(ctx, file)?,
        None => Session::from_scan(ctx.this_os, scan_this_machine(ctx)),
    };
    let presence = if args.only_missing {
        if args.file.is_none() {
            return fail("--only-missing compares a Repot file with this machine; give the file");
        }
        if target.os != ctx.this_os {
            return fail(format!(
                "--only-missing compares with this machine, so the destination must be {}",
                text::os(ctx.this_os)
            ));
        }
        session.machine = Some(scan_this_machine(ctx));
        Some(session.presence(ctx.env.as_ref(), ctx.this_os, target))
    } else {
        None
    };
    if presence
        .as_ref()
        .is_some_and(|p| !p.values().any(|s| *s == Presence::Missing))
        && args.include.is_empty()
    {
        writeln!(
            ctx.out,
            "This machine already has everything in the file; nothing to export."
        )?;
        return Ok(());
    }
    let selected = selection_of(&session, target, presence.as_ref(), args)?;
    let left_out = left_out(&session, target, &selected);
    let selection = Selection {
        items: selected.clone(),
        reveal_secret_lines: vec![],
    };
    let bundle = session
        .preview(
            ctx.env.as_ref(),
            &selection,
            target,
            args.format.format(),
            ctx.meta.clone(),
        )
        .map_err(|e| Failure(e.into()))?;
    report_warnings(ctx, &session, &selected, &bundle)?;
    for (item, reason) in &left_out {
        writeln!(
            ctx.err,
            "note: left out {}: {}",
            item.name,
            text::reason(reason)
        )?;
    }
    let folder = if args.dry_run {
        None
    } else {
        Some(write_export(&bundle, &args.out, &ctx.meta.date).map_err(|e| Failure(e.to_string()))?)
    };
    if args.json {
        let files: Vec<_> = bundle
            .files
            .iter()
            .map(|f| serde_json::json!({ "path": f.path, "executable": f.executable }))
            .collect();
        let value = serde_json::json!({
            "folder": folder.as_ref().map(|p| p.to_string_lossy()),
            "target": target,
            "selected": selected,
            "files": files,
            "warnings": bundle.warnings,
            "leftOut": left_out
                .iter()
                .map(|(item, reason)| serde_json::json!({ "id": item.id, "reason": reason }))
                .collect::<Vec<_>>(),
        });
        serde_json::to_writer_pretty(&mut *ctx.out, &value).map_err(|e| Failure(e.to_string()))?;
        writeln!(ctx.out)?;
        return Ok(());
    }
    let packages = target
        .packages
        .map(|m| format!(" ({})", text::manager(m)))
        .unwrap_or_default();
    let count = selected.len() - bundle.warnings.len();
    writeln!(
        ctx.out,
        "{count} {} for {}{packages}",
        if count == 1 { "item" } else { "items" },
        text::os(target.os)
    )?;
    match &folder {
        None => {
            writeln!(ctx.out, "Would write:")?;
            for f in &bundle.files {
                writeln!(ctx.out, "  {}", f.path)?;
            }
        }
        Some(folder) => {
            writeln!(ctx.out, "Wrote {}", folder.display())?;
            let script = bundle
                .files
                .iter()
                .find(|f| f.executable && !f.path.contains('/'));
            match script {
                Some(s) if s.path.ends_with(".ps1") => writeln!(
                    ctx.out,
                    "Next: copy the folder to the new machine and run {} in PowerShell.",
                    s.path
                )?,
                Some(s) => writeln!(
                    ctx.out,
                    "Next: copy the folder to the new machine and run `bash {}`.",
                    s.path
                )?,
                None => writeln!(
                    ctx.out,
                    "Next: copy the folder to the new machine and follow its README.md."
                )?,
            }
        }
    }
    Ok(())
}

/// Items the destination cannot use, which the export leaves out unless `--include` names them.
/// The app shows these greyed out; the command line says so on stderr.
fn left_out<'a>(
    session: &'a Session,
    target: Target,
    selected: &[String],
) -> Vec<(&'a Item, UnavailableReason)> {
    let avail = session.availability(target);
    session
        .scan
        .items
        .iter()
        .filter(|item| !selected.contains(&item.id))
        .filter_map(|item| match avail.get(&item.id) {
            Some(Availability::Unavailable(reason)) => Some((item, reason.clone())),
            _ => None,
        })
        .collect()
}

fn report_warnings(
    ctx: &mut Context,
    session: &Session,
    selected: &[String],
    bundle: &ExportBundle,
) -> Outcome {
    let name = |id: &str| {
        session
            .scan
            .items
            .iter()
            .find(|i| i.id == id)
            .map_or_else(|| id.to_owned(), |i| i.name.clone())
    };
    for w in &bundle.warnings {
        let id = match w {
            repot_core::export::ExportWarning::Unavailable { item_id, .. }
            | repot_core::export::ExportWarning::InvalidValue { item_id }
            | repot_core::export::ExportWarning::DuplicateTarget { item_id }
            | repot_core::export::ExportWarning::ReadFailed { item_id } => item_id,
        };
        writeln!(ctx.err, "warning: {}", text::warning(w, &name(id)))?;
    }
    // Secrets are always masked from the command line; say where.
    for item in session
        .scan
        .items
        .iter()
        .filter(|i| selected.contains(&i.id))
    {
        let lines: Vec<String> = item
            .findings
            .iter()
            .filter(|f| matches!(f.kind, FindingKind::Secret { .. }))
            .map(|f| f.line.map_or_else(|| "?".into(), |l| l.to_string()))
            .collect();
        if !lines.is_empty() {
            writeln!(
                ctx.err,
                "note: masked secrets in {} ({} {})",
                item.name,
                if lines.len() == 1 { "line" } else { "lines" },
                lines.join(", ")
            )?;
        }
    }
    Ok(())
}
