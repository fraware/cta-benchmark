//! `cta_rust_extract` — pragmatic Rust semantic-cue extractor.
//!
//! Produces a compact [`RustSummary`] describing the shape of a reference
//! implementation (control flow, comparisons, collections, recursion /
//! iteration, mutable-state count, helper calls, and a small set of
//! machine-verifiable semantic tags). It is **not** a Rust verifier; it is a
//! syntactic oracle for the generation pipeline.

#![deny(missing_docs)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced by extraction.
#[derive(Debug, Error)]
pub enum ExtractError {
    /// Failed to read the file.
    #[error("failed to read {0}: {1}")]
    Io(std::path::PathBuf, #[source] std::io::Error),
    /// Failed to parse the file as Rust.
    #[error("failed to parse Rust: {0}")]
    Parse(#[source] syn::Error),
    /// The requested entry function was not found.
    #[error("entry function not found: `{0}`")]
    EntryNotFound(String),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, ExtractError>;

/// High-level summary of a Rust function's shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustSummary {
    /// Function name.
    pub fn_name: String,
    /// Parameters: ordered (name, type) pairs.
    pub params: Vec<Param>,
    /// Rendered return type (or `()` if none).
    pub return_type: String,
    /// Coarse classification of the return type.
    pub return_kind: String,
    /// Control-flow motifs.
    pub control_flow: ControlFlow,
    /// Count of `let mut` bindings syntactically visible in the function.
    pub mutable_locals: u32,
    /// Collections used or referenced (by coarse type name).
    pub collections: Vec<String>,
    /// Comparison operators observed.
    pub comparisons: Vec<String>,
    /// Non-recursive function calls observed by last-path-segment name.
    pub helper_calls: Vec<String>,
    /// Stable machine-verifiable semantic tags the extractor is willing to emit.
    pub semantic_tags: Vec<String>,
}

/// Named parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    /// Parameter name.
    pub name: String,
    /// Rendered Rust type.
    #[serde(rename = "type")]
    pub ty: String,
}

/// Control-flow features.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControlFlow {
    /// Function contains an explicit loop expression (`while`, `loop`, `for`).
    pub uses_loop: bool,
    /// Function (directly) recurses on itself.
    pub uses_recursion: bool,
    /// Function contains an `early return` statement prior to its tail expression.
    pub early_return: bool,
}

/// Parse a Rust source file and extract the summary for `entry_fn`.
pub fn extract_from_file(path: impl AsRef<Path>, entry_fn: &str) -> Result<RustSummary> {
    let path = path.as_ref();
    let src = fs::read_to_string(path).map_err(|e| ExtractError::Io(path.to_path_buf(), e))?;
    extract_from_source(&src, entry_fn)
}

/// Parse a Rust source string and extract the summary for `entry_fn`.
pub fn extract_from_source(src: &str, entry_fn: &str) -> Result<RustSummary> {
    let file = syn::parse_file(src).map_err(ExtractError::Parse)?;
    let func = find_fn(&file, entry_fn)
        .ok_or_else(|| ExtractError::EntryNotFound(entry_fn.to_string()))?;
    Ok(summarize_fn(func))
}

fn find_fn<'a>(file: &'a syn::File, name: &str) -> Option<&'a syn::ItemFn> {
    file.items.iter().find_map(|item| {
        if let syn::Item::Fn(f) = item {
            if f.sig.ident == name {
                return Some(f);
            }
        }
        None
    })
}

fn summarize_fn(f: &syn::ItemFn) -> RustSummary {
    let fn_name = f.sig.ident.to_string();
    let params: Vec<Param> = f
        .sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            syn::FnArg::Typed(pt) => {
                let name = match &*pt.pat {
                    syn::Pat::Ident(id) => id.ident.to_string(),
                    other => quote::quote!(#other).to_string(),
                };
                let ty = render_type(&pt.ty);
                Some(Param { name, ty })
            }
            syn::FnArg::Receiver(_) => None,
        })
        .collect();

    let return_type = match &f.sig.output {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => render_type(ty),
    };

    let mut visitor = Visitor {
        fn_name: fn_name.clone(),
        control_flow: ControlFlow::default(),
        mutable_locals: 0,
        collections: BTreeSet::new(),
        comparisons: BTreeSet::new(),
        helper_calls: BTreeSet::new(),
        in_tail: true,
    };
    visitor.visit_block(&f.block);

    // Enrich collection tags from parameter and return types.
    for p in &params {
        push_collection_tags(&p.ty, &mut visitor.collections);
    }
    push_collection_tags(&return_type, &mut visitor.collections);

    let return_kind = classify_return_kind(&return_type);

    let semantic_tags =
        emit_semantic_tags(&return_kind, &visitor.control_flow, visitor.mutable_locals);

    let collections = visitor.collections.into_iter().collect::<Vec<_>>();
    let comparisons = visitor.comparisons.into_iter().collect::<Vec<_>>();
    let helper_calls = visitor.helper_calls.into_iter().collect::<Vec<_>>();

    RustSummary {
        fn_name,
        params,
        return_type,
        return_kind,
        control_flow: visitor.control_flow,
        mutable_locals: visitor.mutable_locals,
        collections,
        comparisons,
        helper_calls,
        semantic_tags,
    }
}

/// Render a `syn::Type` back to a compact string.
///
/// `quote!` inserts spaces around punctuation (e.g. `Vec < i32 >`); we collapse
/// runs of whitespace next to punctuation but keep a single space between
/// adjacent identifier-like tokens, so `&mut self` stays legible while
/// `Vec<i32>` is compact.
fn render_type(ty: &syn::Type) -> String {
    let raw = quote::quote!(#ty).to_string();
    let mut out = String::with_capacity(raw.len());
    let mut prev_ident = false;
    for token in raw.split_ascii_whitespace() {
        let starts_ident = token
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        if !out.is_empty() && prev_ident && starts_ident {
            out.push(' ');
        }
        out.push_str(token);
        prev_ident = token
            .chars()
            .last()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
    }
    out
}

/// Map a rendered type string to coarse collection-name tags. Deterministic,
/// purely syntactic: we substring-match on the standard container names and
/// on the reference-slice shape `&[...]`.
fn push_collection_tags(rendered: &str, out: &mut BTreeSet<String>) {
    const NAMED: [&str; 7] = [
        "BinaryHeap",
        "BTreeMap",
        "BTreeSet",
        "HashMap",
        "HashSet",
        "Vec",
        "VecDeque",
    ];
    for name in NAMED {
        if contains_whole_ident(rendered, name) {
            out.insert(name.to_string());
        }
    }
    if rendered.contains("&[")
        || rendered.contains("& [")
        || rendered.contains("&mut[")
        || rendered.contains("&mut [")
    {
        out.insert("slice".to_string());
    }
}

/// True if `needle` occurs in `hay` on an identifier boundary.
fn contains_whole_ident(hay: &str, needle: &str) -> bool {
    let bytes = hay.as_bytes();
    let n = needle.len();
    let mut i = 0usize;
    while i + n <= bytes.len() {
        if &bytes[i..i + n] == needle.as_bytes() {
            let before = i == 0 || !is_ident_byte(bytes[i - 1]);
            let after = i + n == bytes.len() || !is_ident_byte(bytes[i + n]);
            if before && after {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Classify the return type into a small, stable enum-like tag.
fn classify_return_kind(rendered: &str) -> String {
    let t = rendered.trim();
    if t == "()" {
        return "unit".to_string();
    }
    if t == "bool" {
        return "bool".to_string();
    }
    if t.starts_with("Option<") || t == "Option" {
        return "option".to_string();
    }
    if t.starts_with("Result<") || t == "Result" {
        return "result".to_string();
    }
    if t.starts_with("Vec<") || t == "Vec" {
        return "vec".to_string();
    }
    const NUMERIC_EXACT: [&str; 14] = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64",
    ];
    if NUMERIC_EXACT.contains(&t) {
        return "numeric".to_string();
    }
    "other".to_string()
}

/// Stable heuristic tags derivable from the other fields; no model or type
/// inference involved. Ordered deterministically.
fn emit_semantic_tags(return_kind: &str, cf: &ControlFlow, mutable_locals: u32) -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    match return_kind {
        "option" => {
            out.insert("uses_option_return".to_string());
        }
        "result" => {
            out.insert("uses_result_return".to_string());
        }
        _ => {}
    }
    match (cf.uses_loop, cf.uses_recursion) {
        (true, false) => {
            out.insert("iterative".to_string());
        }
        (false, true) => {
            out.insert("recursive".to_string());
        }
        (true, true) => {
            out.insert("mixed_control".to_string());
        }
        (false, false) => {
            out.insert("straight_line".to_string());
        }
    }
    if cf.early_return {
        out.insert("uses_early_return".to_string());
    }
    if mutable_locals > 0 {
        out.insert("uses_mutable_state".to_string());
    }
    out.into_iter().collect()
}

struct Visitor {
    fn_name: String,
    control_flow: ControlFlow,
    mutable_locals: u32,
    collections: BTreeSet<String>,
    comparisons: BTreeSet<String>,
    helper_calls: BTreeSet<String>,
    in_tail: bool,
}

impl Visitor {
    fn visit_block(&mut self, b: &syn::Block) {
        let n = b.stmts.len();
        for (i, s) in b.stmts.iter().enumerate() {
            let save = self.in_tail;
            self.in_tail = save && i + 1 == n;
            self.visit_stmt(s);
            self.in_tail = save;
        }
    }

    fn visit_stmt(&mut self, s: &syn::Stmt) {
        match s {
            syn::Stmt::Local(l) => {
                if pattern_has_mut(&l.pat) {
                    self.mutable_locals = self.mutable_locals.saturating_add(1);
                }
                if let Some(init) = &l.init {
                    self.visit_expr(&init.expr);
                }
            }
            syn::Stmt::Item(_) => {}
            syn::Stmt::Expr(e, _) => self.visit_expr(e),
            syn::Stmt::Macro(_) => {}
        }
    }

    fn visit_expr(&mut self, e: &syn::Expr) {
        match e {
            syn::Expr::While(w) => {
                self.control_flow.uses_loop = true;
                self.visit_expr(&w.cond);
                self.visit_block(&w.body);
            }
            syn::Expr::Loop(l) => {
                self.control_flow.uses_loop = true;
                self.visit_block(&l.body);
            }
            syn::Expr::ForLoop(fl) => {
                self.control_flow.uses_loop = true;
                self.visit_expr(&fl.expr);
                self.visit_block(&fl.body);
            }
            syn::Expr::Return(r) => {
                if !self.in_tail {
                    self.control_flow.early_return = true;
                }
                if let Some(inner) = &r.expr {
                    self.visit_expr(inner);
                }
            }
            syn::Expr::Call(c) => {
                if let syn::Expr::Path(p) = &*c.func {
                    if let Some(last) = p.path.segments.last() {
                        let id = last.ident.to_string();
                        if id == self.fn_name {
                            self.control_flow.uses_recursion = true;
                        } else {
                            if matches!(
                                id.as_str(),
                                "Vec"
                                    | "BinaryHeap"
                                    | "HashMap"
                                    | "HashSet"
                                    | "BTreeMap"
                                    | "BTreeSet"
                                    | "VecDeque"
                            ) {
                                self.collections.insert(id.clone());
                            }
                            self.helper_calls.insert(id);
                        }
                    }
                }
                for a in &c.args {
                    self.visit_expr(a);
                }
            }
            syn::Expr::MethodCall(m) => {
                self.visit_expr(&m.receiver);
                for a in &m.args {
                    self.visit_expr(a);
                }
            }
            syn::Expr::Binary(b) => {
                use syn::BinOp::{Eq, Ge, Gt, Le, Lt, Ne};
                let sym = match b.op {
                    Lt(_) => Some("<"),
                    Le(_) => Some("<="),
                    Gt(_) => Some(">"),
                    Ge(_) => Some(">="),
                    Eq(_) => Some("=="),
                    Ne(_) => Some("!="),
                    _ => None,
                };
                if let Some(s) = sym {
                    self.comparisons.insert(s.to_string());
                }
                self.visit_expr(&b.left);
                self.visit_expr(&b.right);
            }
            syn::Expr::If(i) => {
                self.visit_expr(&i.cond);
                self.visit_block(&i.then_branch);
                if let Some((_, el)) = &i.else_branch {
                    self.visit_expr(el);
                }
            }
            syn::Expr::Block(b) => self.visit_block(&b.block),
            syn::Expr::Match(m) => {
                self.visit_expr(&m.expr);
                for arm in &m.arms {
                    self.visit_expr(&arm.body);
                }
            }
            syn::Expr::Paren(p) => self.visit_expr(&p.expr),
            syn::Expr::Reference(r) => self.visit_expr(&r.expr),
            syn::Expr::Unary(u) => self.visit_expr(&u.expr),
            syn::Expr::Assign(a) => {
                self.visit_expr(&a.left);
                self.visit_expr(&a.right);
            }
            syn::Expr::Index(idx) => {
                self.visit_expr(&idx.expr);
                self.visit_expr(&idx.index);
            }
            syn::Expr::Range(r) => {
                if let Some(s) = &r.start {
                    self.visit_expr(s);
                }
                if let Some(e) = &r.end {
                    self.visit_expr(e);
                }
            }
            syn::Expr::Tuple(t) => {
                for e in &t.elems {
                    self.visit_expr(e);
                }
            }
            syn::Expr::Array(a) => {
                for e in &a.elems {
                    self.visit_expr(e);
                }
            }
            syn::Expr::Cast(c) => self.visit_expr(&c.expr),
            syn::Expr::Field(f) => self.visit_expr(&f.base),
            syn::Expr::Try(t) => self.visit_expr(&t.expr),
            syn::Expr::Let(l) => self.visit_expr(&l.expr),
            _ => {}
        }
    }
}

/// True if a let-binding pattern declares any `mut` binding (at any depth we
/// typically encounter in benchmark reference code).
fn pattern_has_mut(p: &syn::Pat) -> bool {
    match p {
        syn::Pat::Ident(id) => id.mutability.is_some(),
        syn::Pat::Tuple(t) => t.elems.iter().any(pattern_has_mut),
        syn::Pat::TupleStruct(t) => t.elems.iter().any(pattern_has_mut),
        syn::Pat::Struct(s) => s.fields.iter().any(|f| pattern_has_mut(&f.pat)),
        syn::Pat::Or(o) => o.cases.iter().any(pattern_has_mut),
        syn::Pat::Reference(r) => pattern_has_mut(&r.pat),
        syn::Pat::Slice(s) => s.elems.iter().any(pattern_has_mut),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn extracts_binary_search_shape() {
        let src = r#"
            pub fn binary_search(arr: &[i32], target: i32) -> Option<usize> {
                let mut lo = 0usize;
                let mut hi = arr.len();
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    if arr[mid] == target { return Some(mid); }
                    if arr[mid] < target { lo = mid + 1; } else { hi = mid; }
                }
                None
            }
        "#;
        let s = extract_from_source(src, "binary_search").unwrap();
        assert_eq!(s.fn_name, "binary_search");
        assert!(s.control_flow.uses_loop);
        assert!(!s.control_flow.uses_recursion);
        assert!(s.control_flow.early_return);
        assert!(s.comparisons.contains(&"<".to_string()));
        assert!(s.comparisons.contains(&"==".to_string()));
        assert_eq!(s.mutable_locals, 2);
        assert_eq!(s.return_kind, "option");
        assert!(s.collections.contains(&"slice".to_string()));
        assert!(s.semantic_tags.contains(&"iterative".to_string()));
        assert!(s.semantic_tags.contains(&"uses_early_return".to_string()));
        assert!(s.semantic_tags.contains(&"uses_option_return".to_string()));
        assert!(s.semantic_tags.contains(&"uses_mutable_state".to_string()));
    }

    #[test]
    fn detects_direct_recursion() {
        let src = r#"
            pub fn fact(n: u64) -> u64 {
                if n == 0 { 1 } else { n * fact(n - 1) }
            }
        "#;
        let s = extract_from_source(src, "fact").unwrap();
        assert!(s.control_flow.uses_recursion);
        assert!(!s.control_flow.uses_loop);
        assert_eq!(s.mutable_locals, 0);
        assert_eq!(s.return_kind, "numeric");
        assert!(s.semantic_tags.contains(&"recursive".to_string()));
        // The function calls itself (`fact`) which is self-recursion, so
        // it must NOT appear in helper_calls.
        assert!(!s.helper_calls.contains(&"fact".to_string()));
    }

    #[test]
    fn helper_calls_captured() {
        let src = r#"
            fn foo(xs: Vec<i32>) -> usize {
                let n = xs.len();
                std::cmp::max(n, 0)
            }
        "#;
        let s = extract_from_source(src, "foo").unwrap();
        assert!(s.helper_calls.contains(&"max".to_string()));
        assert!(s.collections.contains(&"Vec".to_string()));
        assert_eq!(s.return_kind, "numeric");
    }

    #[test]
    fn missing_entry_fn_is_error() {
        let src = "fn other() {}";
        let err = extract_from_source(src, "missing").unwrap_err();
        assert!(matches!(err, ExtractError::EntryNotFound(_)));
    }

    #[test]
    fn classify_return_kind_basic() {
        assert_eq!(classify_return_kind("()"), "unit");
        assert_eq!(classify_return_kind("bool"), "bool");
        assert_eq!(classify_return_kind("u32"), "numeric");
        assert_eq!(classify_return_kind("Option<usize>"), "option");
        assert_eq!(classify_return_kind("Result<(), E>"), "result");
        assert_eq!(classify_return_kind("Vec<i32>"), "vec");
        assert_eq!(classify_return_kind("MyStruct"), "other");
    }

    #[test]
    fn collection_tag_extraction_from_types() {
        let mut out = BTreeSet::new();
        push_collection_tags("&[i32]", &mut out);
        push_collection_tags("Vec<i32>", &mut out);
        push_collection_tags("VecDeque<u32>", &mut out);
        push_collection_tags("MyVecWrapper", &mut out);
        assert!(out.contains("slice"));
        assert!(out.contains("Vec"));
        assert!(out.contains("VecDeque"));
        // must not match inside a longer identifier
        assert_eq!(out.len(), 3);
    }
}
