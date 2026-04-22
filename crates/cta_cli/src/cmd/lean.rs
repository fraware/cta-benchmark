use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::Args;
use cta_lean::{elaborate, ElaborateRequest};

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Path to the Lean file to elaborate. May be absolute or
    /// workspace-relative.
    #[arg(long, value_name = "FILE")]
    pub file: PathBuf,

    /// Lean project directory containing `lakefile.lean`. Defaults to
    /// `<workspace>/lean`.
    #[arg(long, value_name = "DIR")]
    pub project: Option<PathBuf>,

    /// Path to the `lake` binary. Defaults to `lake` on PATH.
    #[arg(long, default_value = "lake")]
    pub lake_bin: PathBuf,

    /// Hard timeout in seconds for the Lean subprocess.
    #[arg(long, default_value_t = 120)]
    pub timeout_secs: u64,

    /// Optional path to write the structured JSON report. If omitted, the
    /// report is written to stdout.
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
}

pub fn check(workspace: &Path, args: CheckArgs) -> Result<()> {
    let project = args
        .project
        .clone()
        .unwrap_or_else(|| workspace.join("lean"));
    if !project.join("lakefile.lean").is_file() {
        return Err(anyhow!(
            "no lakefile.lean under {} (set --project)",
            project.display()
        ));
    }
    let file = if args.file.is_absolute() {
        args.file.clone()
    } else {
        workspace.join(&args.file)
    };
    if !file.is_file() {
        return Err(anyhow!("lean file not found: {}", file.display()));
    }

    let req = ElaborateRequest {
        lake_bin: args.lake_bin.clone(),
        lean_project_dir: project,
        file_path: file,
        timeout: Duration::from_secs(args.timeout_secs),
    };
    let result = elaborate(&req).context("lake env lean invocation failed")?;

    let json = serde_json::to_string_pretty(&result)?;
    if let Some(out) = &args.out {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out, &json)?;
    } else {
        println!("{json}");
    }

    if result.elaborates {
        Ok(())
    } else {
        Err(anyhow!(
            "lean elaboration failed: exit_code={:?}, {} error(s)",
            result.exit_code,
            result
                .diagnostics
                .iter()
                .filter(|d| d.severity == "error")
                .count()
        ))
    }
}
