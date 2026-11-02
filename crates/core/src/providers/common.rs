use crate::{env::Env, scan::ProviderError};
pub(crate) fn run_ok(env: &dyn Env, program: &str, args: &[&str]) -> Result<String, ProviderError> {
    let out = env.run(program, args)?;
    if !out.success() {
        return Err(ProviderError::CommandFailed {
            command: format!("{} {}", program, args.join(" ")),
            code: out.code,
            stderr: out.stderr,
        });
    }
    Ok(out.stdout)
}
