use eyre::Context;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub release: i64,
    pub dependencies: Vec<String>,
    pub source: String,
}

impl Manifest {
    pub fn fullname(&self) -> String {
        format!("{}-{}-{}", self.name, self.version, self.release)
    }
}

impl PartialEq for Manifest {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version && self.release == other.release
    }
}

pub fn load_manifest(path: &str) -> eyre::Result<Manifest> {
    let manifest_src = std::fs::read_to_string(path).context("Failed to read manifest.toml")?;
    let manifest: Manifest =
        toml::from_str(&manifest_src).context("Failed to parse manifest.toml")?;
    Ok(manifest)
}

pub fn save_manifest(manifest: &Manifest, path: &str) -> eyre::Result<()> {
    let manifest_src = toml::to_string(manifest).context("Failed to serialize manifest.toml")?;
    std::fs::write(path, manifest_src).context("Failed to write manifest.toml")?;
    Ok(())
}
