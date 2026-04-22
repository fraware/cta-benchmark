//! Canonical identifier types for the CTA benchmark.
//!
//! Every identifier is validated at construction and round-trips through
//! `serde` via its canonical string form.
//!
//! # Examples
//!
//! ```
//! use cta_core::{BenchmarkVersion, InstanceId, RunId, SystemId};
//!
//! // Valid ids are accepted and round-trip cleanly.
//! let iid = InstanceId::new("graph_dijkstra_001").unwrap();
//! assert_eq!(iid.as_str(), "graph_dijkstra_001");
//!
//! let sys = SystemId::new("full_method_v1").unwrap();
//! let run = RunId::new("run_2026_04_21_full_method_v1_eval_001").unwrap();
//! let ver = BenchmarkVersion::new("v0.1").unwrap();
//!
//! // Invalid ids are rejected at construction time.
//! assert!(InstanceId::new("Graph_Dijkstra_001").is_err());
//! assert!(SystemId::new("no_version_suffix").is_err());
//! assert!(RunId::new("run_shorthand").is_err());
//! assert!(BenchmarkVersion::new("1.0").is_err());
//!
//! // Ids serialise as plain strings.
//! let serialised = serde_json::to_string(&iid).unwrap();
//! assert_eq!(serialised, "\"graph_dijkstra_001\"");
//! ```

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{CoreError, Result};

macro_rules! define_id {
    (
        $(#[$meta:meta])*
        $mod:ident :: $name:ident, $kind:literal, $pattern:literal
    ) => {
        mod $mod {
            use super::*;

            static RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(concat!("^", $pattern, "$")).expect("valid id regex")
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
                        Err(CoreError::InvalidId {
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

                /// Consume into owned string.
                #[must_use]
                pub fn into_inner(self) -> String {
                    self.0
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

define_id!(
    /// Canonical benchmark instance id, e.g. `graph_dijkstra_001`.
    instance_id_mod::InstanceId,
    "instance",
    r"[a-z][a-z0-9]*(?:_[a-z0-9]+)*_[0-9]{3}"
);

define_id!(
    /// Canonical obligation id within an instance, e.g. `obl_003`.
    obligation_id_mod::ObligationId,
    "obligation",
    r"obl_[0-9]{3}"
);

define_id!(
    /// Canonical semantic-unit id, e.g. `SU3`.
    semantic_unit_id_mod::SemanticUnitId,
    "semantic_unit",
    r"SU[0-9]+"
);

define_id!(
    /// Canonical run id, e.g. `run_2026_04_21_full_method_v1_eval_001`.
    run_id_mod::RunId,
    "run",
    r"run_[0-9]{4}_[0-9]{2}_[0-9]{2}_[a-z][a-z0-9_]*_[a-z]+_[0-9]{3}"
);

define_id!(
    /// Canonical system id, e.g. `full_method_v1`.
    system_id_mod::SystemId,
    "system",
    r"[a-z][a-z0-9_]*_v[0-9]+"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn instance_id_accepts_canonical() {
        let id = InstanceId::new("graph_dijkstra_001").unwrap();
        assert_eq!(id.as_str(), "graph_dijkstra_001");
    }

    #[test]
    fn instance_id_rejects_bad() {
        assert!(InstanceId::new("Graph_Dijkstra_001").is_err());
        assert!(InstanceId::new("graph_dijkstra").is_err());
        assert!(InstanceId::new("graph_dijkstra_1").is_err());
        assert!(InstanceId::new("").is_err());
    }

    #[test]
    fn obligation_id_accepts_canonical() {
        assert!(ObligationId::new("obl_000").is_ok());
        assert!(ObligationId::new("obl_999").is_ok());
    }

    #[test]
    fn obligation_id_rejects_bad() {
        assert!(ObligationId::new("obl_1").is_err());
        assert!(ObligationId::new("OBL_001").is_err());
    }

    #[test]
    fn semantic_unit_id_accepts_canonical() {
        assert!(SemanticUnitId::new("SU0").is_ok());
        assert!(SemanticUnitId::new("SU42").is_ok());
    }

    #[test]
    fn run_id_accepts_canonical() {
        let r = RunId::new("run_2026_04_21_full_method_v1_eval_001").unwrap();
        assert_eq!(r.as_str(), "run_2026_04_21_full_method_v1_eval_001");
    }

    #[test]
    fn system_id_accepts_canonical() {
        assert!(SystemId::new("text_only_v1").is_ok());
        assert!(SystemId::new("full_method_v12").is_ok());
    }

    #[test]
    fn system_id_rejects_bad() {
        assert!(SystemId::new("TextOnly_v1").is_err());
        assert!(SystemId::new("text_only").is_err());
    }

    #[test]
    fn instance_id_roundtrips_through_serde() {
        let id = InstanceId::new("arrays_binary_search_001").unwrap();
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "\"arrays_binary_search_001\"");
        let back: InstanceId = serde_json::from_str(&s).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn instance_id_serde_rejects_invalid() {
        let err = serde_json::from_str::<InstanceId>("\"bad id\"").unwrap_err();
        assert!(err.to_string().contains("invalid instance id"));
    }
}
