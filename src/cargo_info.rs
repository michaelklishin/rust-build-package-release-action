use crate::error::{Error, Result};
use serde::Deserialize;
use std::{env, fs};

#[derive(Debug, Clone)]
pub struct CargoInfo {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize)]
struct CargoToml {
    package: Option<PackageSection>,
    workspace: Option<WorkspaceSection>,
}

#[derive(Deserialize)]
struct PackageSection {
    name: Option<String>,
    version: Option<String>,
}

#[derive(Deserialize)]
struct WorkspaceSection {
    package: Option<WorkspacePackage>,
}

#[derive(Deserialize)]
struct WorkspacePackage {
    version: Option<String>,
}

pub fn get_cargo_info() -> Result<CargoInfo> {
    let manifest = env::var("MANIFEST_PATH").unwrap_or_else(|_| "Cargo.toml".to_string());
    get_cargo_info_from_path(&manifest)
}

pub fn get_cargo_info_from_path(manifest_path: &str) -> Result<CargoInfo> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| Error::User(format!("could not read {manifest_path}: {e}")))?;
    let cargo: CargoToml = toml::from_str(&content)?;

    let name = cargo
        .package
        .as_ref()
        .and_then(|p| p.name.clone())
        .unwrap_or_default();

    let version = cargo
        .package
        .as_ref()
        .and_then(|p| p.version.clone())
        .or_else(|| {
            cargo
                .workspace
                .as_ref()
                .and_then(|w| w.package.as_ref())
                .and_then(|p| p.version.clone())
        })
        .unwrap_or_default();

    Ok(CargoInfo { name, version })
}
