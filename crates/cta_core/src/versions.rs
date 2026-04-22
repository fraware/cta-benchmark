//! Canonical version types with explicit, independent namespaces.
//!
//! The mission spec requires these never to be conflated: benchmark, schema,
//! metrics, rubric. We model them as separate types with validated string
//! representations.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{CoreError, Result};

macro_rules! define_version {
    (
        $(#[$meta:meta])*
        $mod:ident :: $name:ident, $kind:literal, $pattern:literal
    ) => {
        mod $mod {
            use super::*;
            static RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(concat!("^", $pattern, "$")).expect("valid version regex")
            });

            $(#[$meta])*
            #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
            pub struct $name(String);

            impl $name {
                /// Construct from an owned string, validating the canonical pattern.
                pub fn new(value: impl Into<String>) -> Result<Self> {
                    let value = value.into();
                    if RE.is_match(&value) {
                        Ok(Self(value))
                    } else {
                        Err(CoreError::InvalidVersion {
                            kind: $kind,
                            pattern: $pattern,
                            value,
                        })
                    }
                }

                /// Canonical string view.
                #[must_use]
                pub fn as_str(&self) -> &str {
                    &self.0
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str(&self.0)
                }
            }

            impl FromStr for $name {
                type Err = CoreError;
                fn from_str(s: &str) -> Result<Self> {
                    Self::new(s)
                }
            }

            impl Serialize for $name {
                fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
                    s.serialize_str(&self.0)
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
                    let s = String::deserialize(d)?;
                    Self::new(s).map_err(serde::de::Error::custom)
                }
            }
        }
        pub use self::$mod::$name;
    };
}

define_version!(
    /// Benchmark version tag, e.g. `v0.1`.
    benchmark_version_mod::BenchmarkVersion,
    "benchmark",
    r"v[0-9]+\.[0-9]+"
);

define_version!(
    /// Schema contract version, e.g. `schema_v1`.
    schema_version_mod::SchemaVersion,
    "schema",
    r"schema_v[0-9]+"
);

define_version!(
    /// Metrics contract version, e.g. `metrics_v1`.
    metrics_version_mod::MetricsVersion,
    "metrics",
    r"metrics_v[0-9]+"
);

define_version!(
    /// Annotation rubric version, e.g. `rubric_v1`.
    rubric_version_mod::RubricVersion,
    "rubric",
    r"rubric_v[0-9]+"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_version_ok() {
        assert!(BenchmarkVersion::new("v0.1").is_ok());
        assert!(BenchmarkVersion::new("v12.3").is_ok());
        assert!(BenchmarkVersion::new("0.1").is_err());
        assert!(BenchmarkVersion::new("v0.1.2").is_err());
    }

    #[test]
    fn schema_version_ok() {
        assert!(SchemaVersion::new("schema_v1").is_ok());
        assert!(SchemaVersion::new("schema_v10").is_ok());
        assert!(SchemaVersion::new("schema_1").is_err());
    }
}
