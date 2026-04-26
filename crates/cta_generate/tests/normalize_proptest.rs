//! Property-based tests for the LLM-response normalizer.
//!
//! The normalizer is the single biggest attack surface when facing real
//! providers: input is adversarial, arbitrarily sized, and may contain
//! malformed JSON, partial output, embedded prose, and unicode edge cases.
//! These properties pin the invariants that every downstream consumer
//! (metrics, reports, schema validation) assumes:
//!
//! 1. The normalizer never panics, regardless of input.
//! 2. A successful parse yields well-formed obligations:
//!    - every `kind` is one of the schema-allowed enum values,
//!    - every `lean_statement` is non-empty,
//!    - every `confidence`, if present, is in `[0, 1]` and finite.
//! 3. A failed parse yields `ParseStatus { ok: false, error_class:
//!    Some(..), error_message: Some(..) }`.
//! 4. Valid canonical JSON always round-trips cleanly.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cta_generate::normalize_response;
use proptest::prelude::*;

const ALLOWED_KINDS: &[&str] = &[
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

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 512,
        .. ProptestConfig::default()
    })]

    #[test]
    fn never_panics_on_arbitrary_string(raw in ".*") {
        let _ = normalize_response(&raw);
    }

    #[test]
    fn never_panics_on_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let raw = String::from_utf8_lossy(&bytes);
        let _ = normalize_response(&raw);
    }

    #[test]
    fn ok_implies_well_formed_obligations(
        kind in "[a-z_]{1,20}",
        statement in "[^\"\\\\]{1,40}",
        gloss in "[^\"\\\\]{0,40}",
        conf in proptest::option::of(-10.0f64..10.0),
    ) {
        let conf_str = match conf {
            Some(c) if c.is_finite() => format!(", \"confidence\": {c}"),
            _ => String::new(),
        };
        let raw = format!(
            "{{\"obligations\":[{{\"kind\":\"{kind}\",\"lean_statement\":\"{statement}\",\"nl_gloss\":\"{gloss}\"{conf_str}}}]}}"
        );
        let (obs, status) = normalize_response(&raw);
        if status.ok {
            prop_assert_eq!(obs.len(), 1);
            let ob = &obs[0];
            prop_assert!(ALLOWED_KINDS.contains(&ob.kind.as_str()),
                "kind `{}` not in schema-allowed set", ob.kind);
            prop_assert!(!ob.lean_statement.is_empty());
            if let Some(c) = ob.confidence {
                prop_assert!(c.is_finite());
                prop_assert!((0.0..=1.0).contains(&c));
            }
        }
    }

    #[test]
    fn err_implies_typed_status(
        garbage in "[^{}\\[\\]\"]{0,200}"
    ) {
        let (obs, status) = normalize_response(&garbage);
        if !status.ok {
            prop_assert!(obs.is_empty());
            prop_assert!(status.error_class.is_some());
            prop_assert!(status.error_message.is_some());
        }
    }

    #[test]
    fn canonical_object_form_always_parses(
        kind in prop::sample::select(ALLOWED_KINDS.to_vec()),
        n in 1usize..7,
    ) {
        let items: Vec<String> = (0..n)
            .map(|i| format!(
                "{{\"kind\":\"{kind}\",\"lean_statement\":\"stmt_{i}\",\"nl_gloss\":\"g\"}}"
            ))
            .collect();
        let raw = format!("{{\"obligations\":[{}]}}", items.join(","));
        let (obs, status) = normalize_response(&raw);
        prop_assert!(status.ok, "canonical shape must parse cleanly");
        prop_assert_eq!(obs.len(), n);
        for ob in &obs {
            prop_assert_eq!(&ob.kind, &kind);
            prop_assert!(!ob.lean_statement.is_empty());
        }
    }

    #[test]
    fn prose_around_canonical_json_is_scraped(
        prefix in "[^{}]{0,80}",
        suffix in "[^{}]{0,80}",
    ) {
        let raw = format!(
            "{prefix}{{\"obligations\":[{{\"kind\":\"structural\",\"lean_statement\":\"P\",\"nl_gloss\":\"\"}}]}}{suffix}"
        );
        let (obs, status) = normalize_response(&raw);
        prop_assert!(status.ok);
        prop_assert_eq!(obs.len(), 1);
    }
}
