use repot_core::{
    Env, PROVIDER_TIMEOUT, RealEnv, Selection,
    availability::{Availability, availability},
    default_providers,
    export::{ExportMeta, ExportRequest, Format, build_export, write_export},
    scan,
};
use std::{path::PathBuf, sync::Arc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let format = match args.next().as_deref().and_then(|s| s.to_str()) {
        Some("script") | None => Format::Script,
        Some("manifest") => Format::Manifest,
        _ => return Err("usage: export [script|manifest] [parent folder]".into()),
    };
    let parent = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    if args.next().is_some() {
        return Err("usage: export [script|manifest] [parent folder]".into());
    }
    let env = Arc::new(RealEnv::new());
    let source = env.os();
    let result = scan(env.clone(), default_providers(), PROVIDER_TIMEOUT, &|p| {
        eprintln!("{}: {:?}", p.id, p.status);
    });
    let selection = Selection {
        items: result
            .items
            .iter()
            .filter(|item| {
                availability(item, source, repot_core::Target::default_for(source))
                    == Availability::Available
            })
            .map(|item| item.id.clone())
            .collect(),
        reveal_secret_lines: vec![],
    };
    let request = ExportRequest {
        items: &result.items,
        selection: &selection,
        source,
        target: source,
        format,
        meta: ExportMeta::now(),
    };
    let bundle = build_export(env.as_ref(), &request);
    for warning in &bundle.warnings {
        eprintln!("{warning:?}");
    }
    let path = write_export(&bundle, &parent, &request.meta.date)?;
    println!("{}", path.display());
    Ok(())
}
