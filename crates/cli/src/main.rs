use clap::Parser;
use repot_cli::{args::Cli, run};
use repot_core::{RealEnv, export::ExportMeta, os::current_os, scan::ProviderStatus};
use std::sync::Arc;

fn main() {
    let cli = Cli::parse();
    let progress = |report: &repot_core::scan::ProviderReport| {
        if let ProviderStatus::Ok { count } = report.status {
            eprintln!("  {}: {count}", report.id);
        }
    };
    let mut out = std::io::stdout().lock();
    let mut err = std::io::stderr();
    let mut ctx = run::Context {
        env: Arc::new(RealEnv::new()),
        this_os: current_os(),
        meta: ExportMeta::now(),
        on_progress: &progress,
        out: &mut out,
        err: &mut err,
    };
    std::process::exit(run::run(cli, &mut ctx));
}
