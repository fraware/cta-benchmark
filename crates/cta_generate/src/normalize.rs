//! Output normalization: parse a raw provider response into a list of
//! [`GeneratedObligation`] records plus a [`ParseStatus`].
//!
//! The parser is intentionally lenient:
//! - It first tries to parse the entire response as a JSON object.
//! - If that fails, it scans for the largest top-level JSON object and tries
//!   that.
//! - It recognizes both `{"obligations": [...]}` and a bare array form.
//! - Missing optional fields default to sensible values.
//! - Unknown obligation kinds become `"unknown"` (which is allowed by the
//!   schema enum).

use crate::{GeneratedObligation, ParseStatus};

/// Canonical kinds allowed by the schema.
const KNOWN_KINDS: &[&str] = &[
    "precondition",
    "postcondition",
    "invariant",
    "termination",
    "bounds",
    "uniqueness",
    "monotonicity",
    "optimality",
    "structural",
    "auxiliary",
    "unknown",
];

/// Normalize a raw provider response string into obligations + status.
#[must_use]
pub fn normalize_response(raw: &str) -> (Vec<GeneratedObligation>, ParseStatus) {
    if raw.trim().is_empty() {
        return (
            Vec::new(),
            ParseStatus::err("empty_output", "provider returned empty response"),
        );
    }

    let value = match parse_loose(raw) {
        Ok(v) => v,
        Err(msg) => {
            return (Vec::new(), ParseStatus::err("json_parse_error", msg));
        }
    };

    let array = match extract_obligations_array(&value) {
        Some(arr) => arr,
        None => {
            return (
                Vec::new(),
                ParseStatus::err(
                    "missing_fields",
                    "expected top-level `obligations` list or bare array",
                ),
            );
        }
    };

    let mut out = Vec::with_capacity(array.len());
    for (i, item) in array.iter().enumerate() {
        match normalize_obligation(item) {
            Ok(ob) => out.push(ob),
            Err(msg) => {
                return (
                    Vec::new(),
                    ParseStatus::err("schema_validation_error", format!("obligation #{i}: {msg}")),
                );
            }
        }
    }

    (out, ParseStatus::ok())
}

fn parse_loose(raw: &str) -> std::result::Result<serde_json::Value, String> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) {
        return Ok(v);
    }
    for slice in scrape_candidates(raw) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(slice) {
            return Ok(v);
        }
    }
    Err(format!(
        "response is not valid JSON and no JSON object could be scraped (len={} bytes)",
        raw.len()
    ))
}

/// Iterate over every balanced `{...}` or `[...]` region in `raw`, from the
/// earliest start position to the latest, ignoring brackets inside
/// double-quoted strings. This is necessary because LLM outputs routinely
/// contain stray brackets in prose before the real JSON payload; trying only
/// the first open bracket misses the real object in those cases.
fn scrape_candidates(raw: &str) -> Vec<&str> {
    let bytes = raw.as_bytes();
    let mut out: Vec<&str> = Vec::new();
    let mut cursor = 0usize;
    while let Some(off) = bytes[cursor..].iter().position(|&b| b == b'{' || b == b'[') {
        let start = cursor + off;
        let open = bytes[start];
        let close = if open == b'{' { b'}' } else { b']' };
        if let Some(end) = scan_balanced(bytes, start, open, close) {
            out.push(&raw[start..=end]);
            cursor = start + 1;
        } else {
            cursor = start + 1;
        }
    }
    // Longest candidates first: the real JSON payload is almost always the
    // largest balanced region, and trying it first minimizes wasted work.
    out.sort_by_key(|s| std::cmp::Reverse(s.len()));
    out
}

fn scan_balanced(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut i = start;
    let mut in_string = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else if b == b'"' {
            in_string = true;
        } else if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn extract_obligations_array(v: &serde_json::Value) -> Option<&Vec<serde_json::Value>> {
    match v {
        serde_json::Value::Array(a) => Some(a),
        serde_json::Value::Object(m) => m.get("obligations").and_then(|x| x.as_array()),
        _ => None,
    }
}

fn normalize_obligation(v: &serde_json::Value) -> std::result::Result<GeneratedObligation, String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "obligation is not a JSON object".to_string())?;
    let kind_raw = obj
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("unknown")
        .trim()
        .to_ascii_lowercase();
    let kind = if KNOWN_KINDS.contains(&kind_raw.as_str()) {
        kind_raw
    } else {
        "unknown".to_string()
    };
    let lean_statement = obj
        .get("lean_statement")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "missing `lean_statement`".to_string())?
        .trim()
        .to_string();
    if lean_statement.is_empty() {
        return Err("`lean_statement` is empty".into());
    }
    let nl_gloss = obj
        .get("nl_gloss")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let linked_semantic_units: Vec<String> = obj
        .get("linked_semantic_units")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let confidence = obj.get("confidence").and_then(|c| c.as_f64()).map(|f| {
        if f.is_finite() {
            f.clamp(0.0, 1.0)
        } else {
            0.0
        }
    });
    Ok(GeneratedObligation {
        kind,
        lean_statement,
        nl_gloss,
        linked_semantic_units,
        confidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_object_form() {
        let raw = r#"{"obligations": [
            {"kind":"postcondition","lean_statement":"True","nl_gloss":"g"}
        ]}"#;
        let (obs, ok) = normalize_response(raw);
        assert!(ok.ok);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].kind, "postcondition");
    }

    #[test]
    fn parses_bare_array() {
        let raw = r#"[{"kind":"invariant","lean_statement":"A","nl_gloss":""}]"#;
        let (obs, ok) = normalize_response(raw);
        assert!(ok.ok);
        assert_eq!(obs[0].kind, "invariant");
    }

    #[test]
    fn unknown_kind_maps_to_unknown() {
        let raw = r#"{"obligations":[{"kind":"weirdo","lean_statement":"X","nl_gloss":""}]}"#;
        let (obs, ok) = normalize_response(raw);
        assert!(ok.ok);
        assert_eq!(obs[0].kind, "unknown");
    }

    #[test]
    fn missing_lean_statement_errors() {
        let raw = r#"{"obligations":[{"kind":"invariant","nl_gloss":""}]}"#;
        let (obs, st) = normalize_response(raw);
        assert!(obs.is_empty());
        assert!(!st.ok);
        assert_eq!(st.error_class.as_deref(), Some("schema_validation_error"));
    }

    #[test]
    fn scrapes_json_embedded_in_prose() {
        let raw = r#"Sure! Here you go:
```json
{"obligations":[{"kind":"bounds","lean_statement":"P","nl_gloss":""}]}
```
Let me know if you need changes."#;
        let (obs, ok) = normalize_response(raw);
        assert!(ok.ok);
        assert_eq!(obs[0].kind, "bounds");
    }

    #[test]
    fn empty_response_errors() {
        let (obs, st) = normalize_response("   ");
        assert!(obs.is_empty());
        assert_eq!(st.error_class.as_deref(), Some("empty_output"));
    }

    #[test]
    fn confidence_is_clamped() {
        let raw = r#"[{"kind":"structural","lean_statement":"X","nl_gloss":"","confidence":2.5}]"#;
        let (obs, ok) = normalize_response(raw);
        assert!(ok.ok);
        assert_eq!(obs[0].confidence, Some(1.0));
    }
}
