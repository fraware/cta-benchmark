pub mod annotate;
pub mod behavior;
pub mod benchmark;
pub mod experiment;
pub mod extract;
pub mod generate;
pub mod lean;
pub mod metrics;
pub mod reports;
pub mod validate;

use std::path::{Path, PathBuf};

pub fn schemas_dir(workspace: &Path) -> PathBuf {
    workspace.join("schemas")
}

pub fn benchmark_dir(workspace: &Path, version: &str) -> PathBuf {
    workspace.join("benchmark").join(version)
}
