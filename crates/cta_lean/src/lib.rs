//! `cta_lean` — Lean 4 integration.
//!
//! Responsibilities:
//!
//! - serialize generated obligations to Lean files under a run workspace
//! - invoke `lake env lean` and collect diagnostics with a hard timeout
//! - normalize diagnostics to a machine-readable JSON shape
//! - extract theorem names from a generated Lean source file
//!
//! This crate is deliberately **infrastructural only**; it does not make
//! annotation or scoring decisions.

#![deny(missing_docs)]

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use cta_core::{InstanceId, SystemId};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wait_timeout::ChildExt;

/// Errors produced by Lean integration.
#[derive(Debug, Error)]
pub enum LeanError {
    /// IO failure (write, read, spawn).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// The Lean toolchain could not be invoked.
    #[error("lean toolchain invocation failed: {0}")]
    Toolchain(String),
    /// Failed to parse Lean diagnostics.
    #[error("diagnostics parse error: {0}")]
    DiagnosticsParse(String),
    /// The Lean subprocess exceeded the configured timeout.
    #[error("elaboration timed out after {0:?}")]
    Timeout(Duration),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, LeanError>;

/// Request to elaborate a generated Lean file.
#[derive(Debug, Clone)]
pub struct ElaborateRequest {
    /// Lean toolchain invocation (`lake` binary, typically).
    pub lake_bin: PathBuf,
    /// Working directory for the Lean project (contains `lakefile.lean`).
    pub lean_project_dir: PathBuf,
    /// Path to the file to elaborate. May be absolute or project-relative.
    pub file_path: PathBuf,
    /// Hard timeout for the entire Lean subprocess.
    pub timeout: Duration,
}

/// Single normalized diagnostic line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    /// Severity reported by Lean. One of `"error"`, `"warning"`, `"info"`.
    pub severity: String,
    /// 1-indexed line number, if parsed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// 1-indexed column, if parsed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    /// Coarse error class (see [`classify_error`]).
    pub error_class: String,
    /// First line of the diagnostic message.
    pub message: String,
    /// Any continuation lines that belong to this diagnostic.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub message_tail: Vec<String>,
}

/// Result of an elaboration attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElaborateResult {
    /// Whether the file elaborated cleanly (exit 0, no error-severity diags).
    pub elaborates: bool,
    /// Structured diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Raw stdout captured from the Lean process (may be large).
    pub stdout: String,
    /// Raw stderr captured from the Lean process (may be large).
    pub stderr: String,
    /// Exit code if the process finished, None if it was killed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Path to the file elaborated (absolute).
    pub file_path: PathBuf,
    /// Normalized theorem names declared in the source file.
    pub theorem_names: Vec<String>,
}

/// Build a deterministic theorem name from instance/system/index.
#[must_use]
pub fn theorem_name(instance: &InstanceId, system: &SystemId, index: u32) -> String {
    format!("{}__{}__obl_{index:02}", instance.as_str(), system.as_str())
}

/// Write a set of obligations as a standalone generated Lean file.
///
/// `scaffold_import` is the Lean import path of the benchmark scaffold, e.g.
/// `CTA.Benchmark.Arrays.BinarySearch001`. The generated file uses
/// `sorry`-filled proofs so elaboration checks the *statement* well-formedness,
/// which is the signal we care about for obligation generation.
pub fn write_generated_lean(
    out_path: &Path,
    scaffold_import: &str,
    namespace: &str,
    theorem_names: &[String],
    lean_statements: &[String],
) -> Result<()> {
    if theorem_names.len() != lean_statements.len() {
        return Err(LeanError::DiagnosticsParse(format!(
            "theorem_names ({}) and lean_statements ({}) length mismatch",
            theorem_names.len(),
            lean_statements.len()
        )));
    }
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut body = String::new();
    body.push_str("-- AUTO-GENERATED. DO NOT EDIT.\n");
    body.push_str(&format!("import {scaffold_import}\n\n"));
    body.push_str(&format!("namespace {namespace}.Generated\n\n"));
    body.push_str(&format!("open {scaffold_import}\n\n"));
    for (name, stmt) in theorem_names.iter().zip(lean_statements.iter()) {
        body.push_str(&format!("theorem {name} : {stmt} := by\n  sorry\n\n"));
    }
    body.push_str(&format!("end {namespace}.Generated\n"));
    std::fs::write(out_path, body)?;
    Ok(())
}

/// Extract the list of `theorem` / `lemma` / `example` names declared in a
/// Lean source file via purely-textual scanning. Robust enough for generated
/// files whose structure is well-formed; not a Lean parser.
#[must_use]
pub fn extract_theorem_names(source: &str) -> Vec<String> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^\s*(?:theorem|lemma|def)\s+([A-Za-z0-9_']+)")
            .expect("theorem-name regex compiles")
    });
    let mut out: Vec<String> = Vec::new();
    for cap in RE.captures_iter(source) {
        if let Some(m) = cap.get(1) {
            out.push(m.as_str().to_string());
        }
    }
    out
}

/// Classify a diagnostic message into a coarse error-class bucket. The
/// classification is purely lexical and deliberately conservative — it maps
/// recognizable Lean error phrases to stable identifiers used by the metrics
/// layer.
#[must_use]
pub fn classify_error(severity: &str, message: &str) -> String {
    if severity == "warning" {
        return "warning".to_string();
    }
    if severity == "info" {
        return "info".to_string();
    }
    let lower = message.to_ascii_lowercase();
    if lower.contains("unknown identifier") || lower.contains("unknown constant") {
        return "unknown_identifier".to_string();
    }
    if lower.contains("type mismatch") {
        return "type_mismatch".to_string();
    }
    if lower.contains("unexpected token") || lower.contains("expected") && lower.contains("syntax")
    {
        return "syntax_error".to_string();
    }
    if lower.contains("unsolved goals") {
        return "unsolved_goals".to_string();
    }
    if lower.contains("declaration uses 'sorry'") || lower.contains("declaration uses sorry") {
        return "sorry".to_string();
    }
    if lower.contains("unknown tactic") {
        return "unknown_tactic".to_string();
    }
    if lower.contains("failed to synthesize") {
        return "typeclass_failure".to_string();
    }
    "other".to_string()
}

/// Parse combined stdout+stderr output from `lake env lean` into structured
/// diagnostics.
///
/// Lean 4 emits diagnostics of the form `path:line:col: severity: message`.
/// Continuation lines (indented or non-prefixed) are attached to the most
/// recent diagnostic's `message_tail`.
#[must_use]
pub fn parse_diagnostics(combined: &str) -> Vec<Diagnostic> {
    static HEAD: Lazy<Regex> = Lazy::new(|| {
        // Allow both forward and back slashes in the path prefix; allow drive
        // letters on Windows.
        Regex::new(
            r"^(?P<path>(?:[A-Za-z]:)?[^:\r\n]+):(?P<line>\d+):(?P<col>\d+):\s*(?P<sev>error|warning|info):\s*(?P<msg>.*)$",
        )
        .expect("diagnostic-head regex compiles")
    });

    let mut out: Vec<Diagnostic> = Vec::new();
    for raw_line in combined.lines() {
        if let Some(cap) = HEAD.captures(raw_line) {
            let severity = cap.name("sev").map_or("", |m| m.as_str()).to_string();
            let line = cap
                .name("line")
                .and_then(|m| m.as_str().parse::<u32>().ok());
            let column = cap.name("col").and_then(|m| m.as_str().parse::<u32>().ok());
            let message = cap.name("msg").map_or("", |m| m.as_str()).to_string();
            let error_class = classify_error(&severity, &message);
            out.push(Diagnostic {
                severity,
                line,
                column,
                error_class,
                message,
                message_tail: Vec::new(),
            });
        } else if let Some(last) = out.last_mut() {
            // Continuation: include as tail if it looks like one.
            let trimmed = raw_line.trim_end();
            if trimmed.is_empty() {
                continue;
            }
            last.message_tail.push(trimmed.to_string());
            // Upgrade classification once a more-specific phrase appears.
            let combined_msg = format!("{} {}", last.message, trimmed);
            let refined = classify_error(&last.severity, &combined_msg);
            if last.error_class == "other" && refined != "other" {
                last.error_class = refined;
            }
        }
    }
    out
}

/// Produce an absolute path for the Lean file given the project directory.
fn canonical_file_path(project_dir: &Path, file: &Path) -> PathBuf {
    if file.is_absolute() {
        file.to_path_buf()
    } else {
        project_dir.join(file)
    }
}

/// Invoke `lake env lean <file>` with a timeout and collect structured
/// diagnostics. The caller is responsible for ensuring `file_path` is a file
/// Lake can resolve (i.e. that it lives under the project's search path).
pub fn elaborate(req: &ElaborateRequest) -> Result<ElaborateResult> {
    let abs_file = canonical_file_path(&req.lean_project_dir, &req.file_path);
    if !abs_file.is_file() {
        return Err(LeanError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("lean file not found: {}", abs_file.display()),
        )));
    }
    let source = std::fs::read_to_string(&abs_file)?;
    let theorem_names = extract_theorem_names(&source);

    let mut cmd = Command::new(&req.lake_bin);
    cmd.current_dir(&req.lean_project_dir)
        .arg("env")
        .arg("lean")
        .arg(&abs_file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        LeanError::Toolchain(format!("failed to spawn `{}`: {e}", req.lake_bin.display()))
    })?;

    let status = match child.wait_timeout(req.timeout).map_err(LeanError::Io)? {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(LeanError::Timeout(req.timeout));
        }
    };

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        out.read_to_string(&mut stdout)?;
    }
    let mut stderr = String::new();
    if let Some(mut err) = child.stderr.take() {
        err.read_to_string(&mut stderr)?;
    }

    let combined = if stderr.is_empty() {
        stdout.clone()
    } else if stdout.is_empty() {
        stderr.clone()
    } else {
        format!("{stdout}\n{stderr}")
    };
    let diagnostics = parse_diagnostics(&combined);

    let exit_code = status.code();
    let any_error = diagnostics.iter().any(|d| d.severity == "error");
    let elaborates = exit_code == Some(0) && !any_error;

    Ok(ElaborateResult {
        elaborates,
        diagnostics,
        stdout,
        stderr,
        exit_code,
        file_path: abs_file,
        theorem_names,
    })
}

/// Check whether `lake` is available on PATH (or at the given binary path).
#[must_use]
pub fn lake_available(lake_bin: &Path) -> bool {
    Command::new(lake_bin)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theorem_name_is_deterministic() {
        let i = InstanceId::new("graph_dijkstra_001").unwrap();
        let s = SystemId::new("full_method_v1").unwrap();
        assert_eq!(
            theorem_name(&i, &s, 2),
            "graph_dijkstra_001__full_method_v1__obl_02"
        );
    }

    #[test]
    fn write_generated_lean_is_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("out.lean");
        write_generated_lean(
            &path,
            "CTA.Benchmark.Arrays.BinarySearch001",
            "CTA.Generated.BinarySearch001",
            &["foo__bar_v1__obl_01".to_string()],
            &["∀ n : Nat, n + 0 = n".to_string()],
        )
        .unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("import CTA.Benchmark.Arrays.BinarySearch001"));
        assert!(content.contains("theorem foo__bar_v1__obl_01"));
        assert!(content.contains("end CTA.Generated.BinarySearch001.Generated"));
    }

    #[test]
    fn extract_theorem_names_finds_all_forms() {
        let src = "theorem foo : True := by trivial\nlemma bar : True := by trivial\ndef baz : Nat := 0\nexample : True := by trivial\n";
        let names = extract_theorem_names(src);
        assert_eq!(
            names,
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );
    }

    #[test]
    fn parse_diagnostics_basic() {
        let out = "/tmp/a.lean:3:4: error: unknown identifier 'foo'\n  at some site\n/tmp/a.lean:5:1: warning: unused variable `x`\n";
        let diags = parse_diagnostics(out);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].severity, "error");
        assert_eq!(diags[0].line, Some(3));
        assert_eq!(diags[0].column, Some(4));
        assert_eq!(diags[0].error_class, "unknown_identifier");
        assert_eq!(diags[0].message_tail, vec!["  at some site".to_string()]);
        assert_eq!(diags[1].severity, "warning");
        assert_eq!(diags[1].error_class, "warning");
    }

    #[test]
    fn parse_diagnostics_windows_drive_letter() {
        let out = "C:\\work\\a.lean:10:5: error: type mismatch\n  expected: Nat\n  got: Int\n";
        let diags = parse_diagnostics(out);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].line, Some(10));
        assert_eq!(diags[0].error_class, "type_mismatch");
        assert_eq!(diags[0].message_tail.len(), 2);
    }

    #[test]
    fn classify_error_variants() {
        assert_eq!(
            classify_error("error", "unknown identifier 'foo'"),
            "unknown_identifier"
        );
        assert_eq!(
            classify_error("error", "type mismatch at binder"),
            "type_mismatch"
        );
        assert_eq!(classify_error("error", "unsolved goals"), "unsolved_goals");
        assert_eq!(
            classify_error("warning", "declaration uses 'sorry'"),
            "warning"
        );
        assert_eq!(classify_error("error", "random text"), "other");
    }

    #[test]
    fn elaborate_reports_missing_file() {
        let req = ElaborateRequest {
            lake_bin: PathBuf::from("lake"),
            lean_project_dir: PathBuf::from("."),
            file_path: PathBuf::from("/nonexistent/definitely_missing.lean"),
            timeout: Duration::from_secs(1),
        };
        let err = elaborate(&req).unwrap_err();
        assert!(matches!(err, LeanError::Io(_)));
    }
}
