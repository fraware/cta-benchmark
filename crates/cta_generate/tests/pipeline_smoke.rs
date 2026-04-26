//! End-to-end smoke test for the generation pipeline.
//!
//! Loads every pilot instance's metadata, builds a [`full_method`] context,
//! and drives the stub provider to produce a schema-conforming
//! [`GeneratedOutputBundle`] for each.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use cta_core::{InstanceId, RunId, SystemId};
use cta_generate::{
    build_context, generate_bundle, GenerateParams, PromptKind, PromptTemplate, StubProvider,
};
use cta_schema::{SchemaName, SchemaRegistry};

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !p.join("benchmark").is_dir() {
        if !p.pop() {
            panic!("workspace root not found");
        }
    }
    p
}

#[test]
fn stub_generation_produces_schema_valid_bundles_for_all_pilot_instances() {
    let root = workspace_root();
    let registry = SchemaRegistry::load(root.join("schemas")).expect("load schemas");
    let template_path = root
        .join("configs")
        .join("prompts")
        .join("full_method_v1.json");
    let template = PromptTemplate::load(&template_path).expect("load template");
    assert!(matches!(template.kind, PromptKind::FullMethod));

    let provider = StubProvider::default();

    let run_id = RunId::new("run_2026_04_21_full_method_v1_dev_001").unwrap();
    let system_id = SystemId::new("full_method_v1").unwrap();

    let instances_root = root.join("benchmark").join("v0.1").join("instances");
    let mut checked = 0usize;
    for domain in std::fs::read_dir(&instances_root).expect("read instances") {
        let domain = domain.expect("entry").path();
        if !domain.is_dir() {
            continue;
        }
        for inst_dir in std::fs::read_dir(&domain).expect("read domain") {
            let inst_dir = inst_dir.expect("entry").path();
            if !inst_dir.is_dir() {
                continue;
            }
            let inst_json_path = inst_dir.join("instance.json");
            if !inst_json_path.is_file() {
                continue;
            }
            let inst_raw = std::fs::read_to_string(&inst_json_path).unwrap();
            let inst_json: serde_json::Value = serde_json::from_str(&inst_raw).unwrap();
            let iid = inst_json["instance_id"].as_str().unwrap().to_string();
            let instance_id = InstanceId::new(&iid).unwrap();

            let informal = inst_json["informal_statement"]["text"]
                .as_str()
                .unwrap()
                .to_string();
            let scaffold_rel = inst_json["lean_target"]["scaffold_path"].as_str().unwrap();
            let semantic_rel = inst_json["lean_target"]["semantic_units_path"]
                .as_str()
                .unwrap();
            let scaffold_path = root.join("benchmark").join("v0.1").join(scaffold_rel);
            let semantic_path = root.join("benchmark").join("v0.1").join(semantic_rel);

            let ctx = build_context(
                template.kind,
                &inst_dir,
                &informal,
                &scaffold_path,
                &semantic_path,
            )
            .expect("build context");

            let params = GenerateParams {
                run_id: run_id.clone(),
                system_id: system_id.clone(),
                instance_id: instance_id.clone(),
                seed: 0,
                max_tokens: 256,
                temperature: 0.0,
                raw_output_path: format!("generated/full_method_v1/raw/{iid}.txt"),
            };
            let bundle = generate_bundle(&provider, &template, &ctx, &params).unwrap();
            let bundle_json = serde_json::to_value(&bundle).unwrap();
            registry
                .validate(SchemaName::GeneratedOutput, &bundle_json)
                .unwrap_or_else(|e| panic!("bundle for {iid} invalid: {e}"));
            checked += 1;
        }
    }
    assert_eq!(checked, 12);
}
