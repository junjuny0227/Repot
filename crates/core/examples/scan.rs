use repot_core::{PROVIDER_TIMEOUT, RealEnv, default_providers, scan};
use std::sync::Arc;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = scan(
        Arc::new(RealEnv::new()),
        default_providers(),
        PROVIDER_TIMEOUT,
        &|p| eprintln!("{}: {:?}", p.id, p.status),
    );
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
