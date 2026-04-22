//! Snapshot tests pinning the rendered form of every checked-in prompt
//! template against a canonical context. A diff in these snapshots
//! indicates either an intentional prompt-library change (which requires a
//! new template version per the evaluation contract) or a regression in
//! rendering.

use std::path::PathBuf;

use cta_generate::{PromptContext, PromptKind, PromptTemplate};

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .unwrap_or(manifest)
}

fn load(name: &str) -> PromptTemplate {
    let path = workspace_root()
        .join("configs")
        .join("prompts")
        .join(format!("{name}.json"));
    PromptTemplate::load(&path).unwrap_or_else(|e| panic!("load {}: {e}", path.display()))
}

fn canonical_ctx(kind: PromptKind) -> PromptContext {
    let mut ctx = PromptContext::new();
    match kind {
        PromptKind::TextOnly => {
            ctx.insert(
                "informal_statement",
                "Return the index of `target` in the sorted slice `a`, or `None` if absent.",
            );
        }
        PromptKind::CodeOnly => {
            let code =
                "pub fn binary_search(a: &[i64], target: i64) -> Option<usize> { /* ... */ }";
            ctx.insert("reference_rs", code)
                .insert("rust_reference", code);
        }
        PromptKind::NaiveConcat => {
            let code =
                "pub fn binary_search(a: &[i64], target: i64) -> Option<usize> { /* ... */ }";
            ctx.insert("informal_statement", "Return the index of `target` in `a`.")
                .insert("reference_rs", code)
                .insert("rust_reference", code);
        }
        PromptKind::FullMethod => {
            ctx.insert("problem_summary", "Binary search on a sorted slice.")
                .insert(
                    "reference_rs",
                    "pub fn binary_search(a: &[i64], target: i64) -> Option<usize> { /* ... */ }",
                )
                .insert(
                    "semantic_units",
                    "SU1 sortedness; SU2 index-correctness; SU3 none-on-miss.",
                )
                .insert("rust_summary", "{\"shape\":\"divide_and_conquer\"}")
                .insert(
                    "lean_scaffold",
                    "theorem binary_search_correct : True := trivial",
                );
        }
    }
    ctx
}

#[test]
fn snapshot_text_only_v1() {
    let t = load("text_only_v1");
    let rendered = t.render(&canonical_ctx(PromptKind::TextOnly));
    insta::assert_snapshot!("text_only_v1", rendered);
}

#[test]
fn snapshot_code_only_v1() {
    let t = load("code_only_v1");
    let rendered = t.render(&canonical_ctx(PromptKind::CodeOnly));
    insta::assert_snapshot!("code_only_v1", rendered);
}

#[test]
fn snapshot_naive_concat_v1() {
    let t = load("naive_concat_v1");
    let rendered = t.render(&canonical_ctx(PromptKind::NaiveConcat));
    insta::assert_snapshot!("naive_concat_v1", rendered);
}

#[test]
fn snapshot_full_method_v1() {
    let t = load("full_method_v1");
    let rendered = t.render(&canonical_ctx(PromptKind::FullMethod));
    insta::assert_snapshot!("full_method_v1", rendered);
}
