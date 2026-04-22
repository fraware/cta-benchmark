//! Error types for `cta_core`.

use thiserror::Error;

/// Errors produced by identifier and version parsing.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum CoreError {
    /// An identifier did not match its canonical pattern.
    #[error("invalid {kind} id: expected pattern '{pattern}', got '{value}'")]
    InvalidId {
        /// Kind of id (e.g. "instance", "obligation").
        kind: &'static str,
        /// The expected regex pattern.
        pattern: &'static str,
        /// The offending value.
        value: String,
    },

    /// A version string did not match its canonical pattern.
    #[error("invalid {kind} version: expected pattern '{pattern}', got '{value}'")]
    InvalidVersion {
        /// Kind of version (e.g. "benchmark", "schema", "metrics", "rubric").
        kind: &'static str,
        /// The expected regex pattern.
        pattern: &'static str,
        /// The offending value.
        value: String,
    },

    /// An enum discriminant was unknown.
    #[error("unknown {kind} variant: '{value}'")]
    UnknownVariant {
        /// Enum name.
        kind: &'static str,
        /// The unknown value.
        value: String,
    },
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, CoreError>;
