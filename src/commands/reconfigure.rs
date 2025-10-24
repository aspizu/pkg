use crate::package::run_hook;

pub fn reconfigure(package: &str) -> eyre::Result<()> {
    run_hook(package, "reconfigure")?;
    Ok(())
}
