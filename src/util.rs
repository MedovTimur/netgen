use std::{fs, path::Path};

use anyhow::Result;
use serde::de::DeserializeOwned;

/// Read YAML file of arbitrary type.
pub fn read_yaml<T: DeserializeOwned>(path: &str) -> Result<T> {
    let s = fs::read_to_string(path)?;
    let value = serde_yaml::from_str(&s)?;
    Ok(value)
}

/// Output directory selection logic:
/// 1) cli_out_dir (if provided),
/// 2) cfg_out_dir (from YAML),
/// 3) default_name (usually project_name).
pub fn resolve_out_dir(
    cli_out_dir: Option<String>,
    cfg_out_dir: Option<String>,
    default_name: &str,
) -> String {
    cli_out_dir
        .or(cfg_out_dir)
        .unwrap_or_else(|| default_name.to_string())
}

/// Create directory and return Path.
pub fn ensure_out_dir(path_str: &str) -> Result<&Path> {
    use std::path::Path;
    let path = Path::new(path_str);
    std::fs::create_dir_all(path)?;
    Ok(path)
}
