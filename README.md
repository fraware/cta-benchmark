# cta-benchmark

A versioned benchmark of Classical Algorithm Tasks (CTA) with a Rust-first
orchestration and evaluation stack, a Lean 4 target/checking layer for
generated obligations, and a reproducible experiment pipeline that outputs
paper-ready tables, figures, and adjudication artifacts.

## Mission

This repository produces four things, and nothing else should be allowed to
muddy the scope:

1. A versioned benchmark of classical algorithm tasks.
2. A Rust-first orchestration and evaluation stack.
3. A Lean target/checking layer for generated obligations.
4. A reproducible experiment pipeline that outputs paper-ready tables,
   figures, and adjudication artifacts.

The scientific purpose is not "general Rust verification" and not "full
theorem proving." It is a benchmark and baseline system for generating Lean
obligations from text plus code, then measuring elaboration, semantic
faithfulness, code consistency, vacuity, and proof utility.

## Non-goals

- No general-purpose IDE plugin
- No attempt to verify arbitrary unsafe Rust
- No full end-to-end proof agent as a required milestone
- No web app before the benchmark/eval loop is stable
- No large-scale dataset scraping
- No fancy distributed training infrastructure
- No ambiguous benchmark tasks that cannot be adjudicated cleanly

## Repository layout

```
cta-benchmark/
├── Cargo.toml              # Rust workspace
├── rust-toolchain.toml     # Pinned toolchain (1.88.0)
├── deny.toml               # cargo-deny supply-chain policy
├── configs/                # Versioned providers / prompts / experiments / metrics
├── schemas/                # JSON schemas (authoritative)
├── benchmark/
│   ├── v0.1/               # Pilot benchmark artifacts (immutable)
│   └── v0.2/               # Paper-track benchmark artifacts (in progress)
├── lean/                   # Lean 4 project (scaffolds, generated, proofs)
├── crates/                 # Rust crates (see below)
├── runs/                   # Immutable per-run output directories
├── reports/                # Paper-ready tables and figures
├── notebooks/              # Optional Python analysis (non-core)
├── docs/                   # Architecture, spec, rubric, evaluation contract
└── tests/                  # Integration, golden, fixtures
```

### Rust crates

| Crate               | Role                                                    |
| ------------------- | ------------------------------------------------------- |
| `cta_core`          | Canonical domain types, IDs, versions, enums            |
| `cta_schema`        | JSON schema loading and validation                      |
| `cta_benchmark`     | Benchmark loading, indexing, linting, manifest building |
| `cta_rust_extract`  | Semantic cue extraction from Rust reference code        |
| `cta_generate`      | Candidate obligation generation (providers + prompts)   |
| `cta_lean`          | Lean file writer, elaboration driver, diagnostics       |
| `cta_behavior`      | Behavioral harness / falsification runner               |
| `cta_annotations`   | Annotation ingest, adjudication, aggregation            |
| `cta_metrics`       | Pure, deterministic metric computation                  |
| `cta_reports`       | Tables, figures, LaTeX / Markdown / CSV exports         |
| `cta_cli`           | Single entry point (`cta ...`)                          |

## Quickstart

### Prerequisites

- Rust `1.88.0` (pinned via `rust-toolchain.toml`).
- Lean 4 `v4.12.0` (pinned via `lean/lean-toolchain`, managed by `elan`).
- Optional: `cargo-deny`, `cargo-audit`, `cargo-insta` for local parity with
  CI supply-chain and snapshot jobs.

### Build and test

```bash
cargo build --workspace
cargo test --workspace --all-targets
cargo test --workspace --doc
```

### CLI surface

```bash
# Schema + benchmark validation
cargo run -p cta_cli -- validate schemas
cargo run -p cta_cli -- validate benchmark --version v0.2
cargo run -p cta_cli -- validate benchmark --version v0.2 --release
cargo run -p cta_cli -- validate file --schema run_manifest --path runs/<run_id>/run_manifest.json

# Benchmark loader / linter / manifest / summary
cargo run -p cta_cli -- benchmark lint     --version v0.2
cargo run -p cta_cli -- benchmark stats    --version v0.2
cargo run -p cta_cli -- benchmark manifest --version v0.2

# Semantic extractors and per-instance diagnostics
cargo run -p cta_cli -- extract rust-summary --instance arrays_binary_search_001 --version v0.2
cargo run -p cta_cli -- behavior check       --instance arrays_binary_search_001 --version v0.2
cargo run -p cta_cli -- lean check           --file benchmark/v0.2/instances/arrays/arrays_binary_search_001/scaffold.lean

# Single-system generation (stub provider is offline and deterministic)
cargo run -p cta_cli -- generate \
  --version v0.2 --split dev --system full_method_v1 \
  --provider configs/providers/local_stub.json

# Subset generation: `--instances` restricts to ids that must appear in the split
# (comma-separated). Useful for cross-domain smoke runs.
cargo run -p cta_cli -- generate \
  --version v0.2 --split eval --system code_only_v1 \
  --instances arrays_binary_search_002,graph_dijkstra_001 \
  --provider stub --run-id run_2026_04_22_code_only_v1_eval_zfix_001

# Prompt safety: `generate` / `experiment` render prompts with strict placeholder
# resolution (no leftover `{{...}}`) and refuse empty `reference.rs` for
# `code_only_v1` / `naive_concat_v1` before any provider call.

# Annotation pack (reads benchmark/<v>/annotation/adjudicated_subset by default).
# Add --from-benchmark to write the canonical release-grade pack back into
# the benchmark tree (benchmark/<v>/annotation/adjudicated_subset/pack.json).
cargo run -p cta_cli -- annotate pack --version v0.2 --policy prefer-adjudicator
cargo run -p cta_cli -- annotate pack --version v0.2 --from-benchmark

# Annotation closure planning / batching / coverage
cargo run -p cta_cli -- annotate plan \
  --benchmark-version v0.2 \
  --experiment-config configs/experiments/benchmark_v1.json \
  --out benchmark/v0.2/annotation/task_board
cargo run -p cta_cli -- annotate batches \
  --benchmark-version v0.2 \
  --missing-pairs benchmark/v0.2/annotation/task_board/missing_pairs.json \
  --out benchmark/v0.2/annotation/task_board/batches
cargo run -p cta_cli -- annotate coverage \
  --benchmark-version v0.2 \
  --experiment-config configs/experiments/benchmark_v1.json \
  --pack benchmark/v0.2/annotation/adjudicated_subset/pack.json \
  --out benchmark/v0.2/annotation/adjudicated_subset

# Sync adjudicator records from review packets into the canonical subset
cargo run -p cta_cli -- annotate sync-review-packets \
  --benchmark-version v0.2 \
  --from benchmark/v0.2/annotation/review_packets \
  --out benchmark/v0.2/annotation/adjudicated_subset

# Lean proof-status refresh (strict M1 gate + proof-completion artifacts).
# M1 full elaboration is required only for (system_id, instance_id) pairs in
# `is_m1_target_packet` (see `crates/cta_cli/src/cmd/annotate.rs`).
cargo run -p cta_cli -- annotate refresh-lean-check \
  --benchmark-version v0.2 \
  --packets-root benchmark/v0.2/annotation/review_packets \
  --strict-m1

# Strict review-packet audit gate (schema + integrity + signed summary).
# Run after refresh so `packet_hashes` match final `packet.json` bytes.
cargo run -p cta_cli -- annotate verify-review-packets \
  --benchmark-version v0.2 \
  --packets-root benchmark/v0.2/annotation/review_packets \
  --schema schemas/review_packet.schema.json \
  --out benchmark/v0.2/annotation/review_packets/verification_summary.signed.json

# Metrics + reports for a single run directory. Use the benchmark-local
# pack for paper-reportable numbers; the runs/annotation_packs/ copy is
# only for ad-hoc adjudication sessions.
cargo run -p cta_cli -- metrics compute \
  --run <run_id> \
  --annotations benchmark/v0.2/annotation/adjudicated_subset/pack.json
cargo run -p cta_cli -- reports build --run <run_id>

# Config-driven experiment orchestration
cargo run -p cta_cli -- experiment --config configs/experiments/pilot_v1.json --dry-run
cargo run -p cta_cli -- experiment --config configs/experiments/benchmark_v1_openai_only.json

# Gold-audit workbook generation for v0.2 eval instances
cargo run -p cta_cli -- benchmark audit-workbook --version v0.2

# Paper artifact package from canonical run ids
cargo run -p cta_cli -- reports package \
  --benchmark-version v0.2 \
  --canonical-run-ids <run_id_1>,<run_id_2>,...

# End-to-end fail-fast paper-track orchestrator
# (plan -> batches -> coverage -> validate --release -> refresh-lean-check -> verify-review-packets -> package)
cargo run -p cta_cli -- benchmark paper-orchestrate \
  --benchmark-version v0.2 \
  --canonical-run-ids <run_id_1>,<run_id_2>,...
```

### Lean

```bash
cd lean
lake build
```

## Versioning discipline

Everything benchmark-related is versioned explicitly:

- benchmark version: `v0.1`, `v0.2`, ...
- schema version: `schema_v1`
- metric contract version: `metrics_v2`
- annotation rubric version: `rubric_v1`

`v0.1` is a 12-instance pilot release: 2 instances per domain across the
6 domains (`arrays`, `sorting`, `graph`, `dp`, `greedy`, `trees`). Both
`dev` and `eval` cover all 12 instances; `dev` is kept as a diagnostic
duplicate of `eval`. Adjudicated gold annotations currently cover a
3-instance subset (one per representative domain) and will expand with
subsequent releases; paper numbers are only reportable on
`(instance, system)` pairs present in the pack.

`v0.2` is the paper-track release scaffold. It enforces held-out evaluation
in release validation (disjoint `dev`/`eval`, `eval` size >= 24 for
paper-claim eligibility), full annotation coverage for experiments that opt
into `require_full_annotation_coverage`, and a two-reviewer gold-audit
signoff file under `benchmark/v0.2/audit/gold_signoff.json`.

Released benchmark instances are **never mutated in place**. Add a new
version instead. The benchmark linter additionally enforces byte-identity
between each instance scaffold and its canonical Lean module under
`lean/CTA/Benchmark/**`.

## Reproducibility

Every run emits a `run_manifest.json` (see `schemas/run_manifest.schema.json`)
capturing commit hash, benchmark / schema / metrics / rubric versions,
system and prompt identifiers, prompt SHA-256 hashes per instance, seeds,
toolchain versions, timestamp, and provider metadata. **No result is
accepted into the paper without a manifest.**

The generation pipeline is build-pure: no network calls happen during
`cargo build`. Live providers (`OpenAiProvider`, `AnthropicProvider`) only
issue HTTP requests at runtime, and only when their respective credential
environment variables are present; otherwise they refuse to run and surface
a typed `GenerateError::Provider`.

`cta` auto-loads `<workspace>/.env` on startup, so local provider credentials
can be set once in the repo root for interactive runs.

## Quality gates

Every push runs:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --no-deps`
- `cargo test --workspace --all-targets`
- `cargo test --workspace --doc`
- `cta validate schemas`
- `cta validate benchmark --version v0.1 --release` (pilot checks)
- `cta validate benchmark --version v0.2 --release` (paper-track checks)
- `cta experiment --config configs/experiments/pilot_v1.json --dry-run` and
  the full run against the stub provider, with schema validation of every
  emitted `run_manifest.json` and `results_bundle.json`.
- `cargo-deny check --all-features` and `cargo audit --deny warnings`
  (see `.github/workflows/supply-chain.yml`).
- `lake build` in `lean/` over the full `CTA` Lean project (scaffolds must compile).

Snapshot tests in `crates/cta_reports` and `crates/cta_generate` pin the
exact CSV/Markdown/LaTeX shape of every report artifact and the rendered
form of every prompt template. Property-based tests in
`crates/cta_generate/tests/normalize_proptest.rs` assert that the
LLM-output normalizer never panics and only accepts well-formed
obligations.

### Obligation quality gate

`code_only_v1` and `naive_concat_v1` enforce a strict two-layer obligation model:

- benchmark-facing layer: only direct theorems used for semantic-faithfulness scoring
- auxiliary layer: proof scaffolding (invariants, helper lemmas, termination machinery)

The generator/normalizer pipeline applies the following fail-closed rules:

- strict template rendering (`render_strict`) with no unresolved placeholders
- required non-empty `reference.rs` for `code_only_v1` / `naive_concat_v1`
- vacuous theorem rejection (`True`, `P -> True`, `P ∧ True`, `∃ x, True`, `: Prop := by trivial`, placeholder-style glosses)
- off-spec theorem demotion (for example stability claims when stability is optional)
- benchmark-facing-first ordering in normalized obligations

Paper-track `code_only_v1` review packets under `benchmark/v0.2/annotation/review_packets/code_only_v1/` are expected to carry machine-checkable structure beyond the minimum JSON schema: each `generated_obligations[]` entry should include `layer` (`benchmark_facing` or `auxiliary`), stable `index`, non-empty `linked_semantic_units` where the rubric expects linkage, and a `quality_summary` block consistent with the benchmark-facing theorems. Operational markdown under batch directories is a human workflow aid only; the canonical contract is `packet.json` plus `schemas/review_packet.schema.json` and the regressions below.

Review packets include per-packet QA metadata under `quality_summary`:

- `critical_units_covered_by_direct_theorems`
- `critical_units_only_indirectly_covered`
- `off_spec_theorems_present`
- `vacuous_theorems_present`

The benchmark-facing layer is expected to stay compact (typically <= 6 theorems)
and sufficient for evaluating semantic faithfulness without auxiliary lemmas.

Final theorem-hygiene constraints for benchmark-facing obligations:

- interval scheduling feasibility witnesses must use subset semantics (`iv ∈ S → iv ∈ intervals`) and never `iv ∈ S ↔ iv ∈ intervals`
- BFS witness/minimality path-edge clauses must encode consecutive vertices `u = p[i]`, `w = p[i+1]` with adjacency expressed in scaffold-consistent form, for example `w ∈ adj[u]`, `list.mem w (adj[u].tolist)`, or membership in `(adj.get? u).getd []` (List-based `Adj` scaffolds); self-loop forms such as `p.get? i ∈ adj[p.get? i]` are rejected
- BST key-change obligations must use absent/present split theorems with multiset semantics, not malformed disjunctions over implications
- Dijkstra preconditions must avoid vacuous non-negativity clauses when edge weights are already typed as `Nat`

Run these after any change to prompts, normalizers, or committed `packet.json` files for the listed systems:

```bash
cargo test -p cta_generate --test code_only_packet_regression
cargo test -p cta_generate --test family_packet_regression
cargo test -p cta_generate --test naive_concat_packet_regression
cargo test -p cta_generate --test text_only_packet_regression
cargo test -p cta_generate --test full_method_priority1_packet_regression
cargo test -p cta_generate --test full_method_priority2_packet_regression
cargo test -p cta_generate --test review_packet_lean_lint
cargo run -p cta_cli -- annotate refresh-lean-check \
  --benchmark-version v0.2 \
  --packets-root benchmark/v0.2/annotation/review_packets \
  --strict-m1
cargo run -p cta_cli -- annotate verify-review-packets \
  --benchmark-version v0.2 \
  --packets-root benchmark/v0.2/annotation/review_packets \
  --schema schemas/review_packet.schema.json \
  --out benchmark/v0.2/annotation/review_packets/verification_summary.signed.json
```

**Canonical `code_only_v1` regression roster** (must stay green; source of truth is the `targets` array in `crates/cta_generate/tests/code_only_packet_regression.rs`): `arrays_binary_search_001`, `arrays_binary_search_002`, `arrays_max_subarray_001`, `arrays_max_subarray_002`, `graph_bfs_shortest_path_001`, `graph_bfs_shortest_path_002`, `graph_dijkstra_001`, `graph_dijkstra_002`, `greedy_interval_scheduling_001`, `sorting_insertion_sort_001`, `sorting_insertion_sort_002`, `sorting_merge_sort_001`, `sorting_merge_sort_002`, `trees_bst_insert_001`, `trees_bst_insert_002`, `trees_lowest_common_ancestor_001`, `trees_lowest_common_ancestor_002`, `dp_knapsack_01_001`, `dp_knapsack_01_002`, `dp_longest_common_subsequence_001`.

**Canonical `naive_concat_v1` regression roster** (same idea; `targets` in `crates/cta_generate/tests/naive_concat_packet_regression.rs`): includes `graph_dijkstra_001`, `graph_dijkstra_002`, `dp_knapsack_01_001`, and `dp_knapsack_01_002` alongside the other pilot instances so layers and `quality_summary` cannot rot silently.

**`text_only_v1`:** `crates/cta_generate/tests/text_only_packet_regression.rs` pins migrated pilots that already use `layer` + `quality_summary` (both knapsack instances and `graph_dijkstra_{001,002}`); extend `targets` when additional `text_only_v1` packets are brought up to the same schema.

Regression suites pin this contract for focused cleanup packets:

- `crates/cta_generate/tests/code_only_packet_regression.rs`
- `crates/cta_generate/tests/naive_concat_packet_regression.rs`
- `crates/cta_generate/tests/text_only_packet_regression.rs`
- `crates/cta_generate/tests/family_packet_regression.rs`
- `crates/cta_generate/tests/full_method_priority1_packet_regression.rs` and `full_method_priority2_packet_regression.rs` (curated `full_method_v1` graph + knapsack + LCA + binary-search packets)
- `crates/cta_generate/tests/review_packet_lean_lint.rs` (repo-wide static checks on every `review_packets/**/packet.json`)
- strict proof-status gate:
  `cta annotate refresh-lean-check --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --strict-m1` (M1 elaboration required only for `is_m1_target_packet` pairs in `crates/cta_cli/src/cmd/annotate.rs`)

### Lean scaffolds (`lean/CTA/Benchmark/**`)

- Every instance `scaffold.lean` under `benchmark/**/instances/` and every v0.2 `annotation/review_packets/**/scaffold.lean` copy for that instance must remain **byte-identical** to the canonical module under `lean/CTA/Benchmark/...`. `cta benchmark lint --version <v>` enforces this.
- Prefer **definition-backed family theory modules** plus instance-level `abbrev` aliases for benchmark-facing symbols. Reserve **`axiom`** only for deliberate transitional gaps tracked by `lean_check.proof_mode = "axiom_backed"` in review packets.
- This toolchain’s prelude **`List`** does not ship `List.sum`; sums over `List Nat` in scaffolds should use **`List.foldl`** (see `Decomposes` in the coin-change canonical modules).

### Proof-completion status (`v0.2`)

`annotate refresh-lean-check` is the canonical source of truth for Lean proof
completion progress on review packets. It updates `packet.json` `lean_check`
metadata and emits:

- `proof_completion_dashboard.json/csv`
- `wave1_proof_worklist.json/csv`
- `global_proof_worklist.json/csv`
- `proof_execution_plan.json`

Current strict-refresh snapshot committed in this workspace:

- `total_packets = 94`
- `m2_ready_packets = 94`
- `summary_by_gap_reason = {m2_ready: 94}`
- `global_proof_worklist.count = 0`

Execution policy (now fully completed for `v0.2`):

- burn down `admit_debt` first for already definition-backed packets
- migrate highest-impact `axiom_backed_interface` families to definition-backed theory surfaces
- re-run strict refresh and commit regenerated dashboard/worklist artifacts after each migration wave
- keep `review_packet_lean_lint` and priority packet regressions green while migrating

For **new** theory-backed instances, add the corresponding `(system_id,
instance_id)` pairs to `is_m1_target_packet` when they must receive full M1
elaboration on every strict refresh; otherwise `lean_check.elaborated` may
remain `false` by design.

### Gold template packet families

The following `naive_concat_v1` packets are the canonical shape exemplars for
future prompt/normalizer behavior:

- `naive_concat_v1/greedy_interval_scheduling_001`
- `naive_concat_v1/sorting_merge_sort_001`
- `naive_concat_v1/trees_bst_insert_001`
- `naive_concat_v1/graph_dijkstra_001`
- `naive_concat_v1/dp_knapsack_01_001` (and the paired `002` id in regression rosters)

For these families, benchmark-facing obligations must remain:

- compact (small direct theorem surface)
- placeholder-free (no `{{...}}`)
- vacuity-free (no `True` shell theorem forms)
- direct with respect to critical semantic units
- free of proof-plan prose in theorem slots

### Family-specific theorem-shape guards

In addition to the generic quality gate, CI now enforces packet-family shape
checks for known failure modes:

- LCS: subsequence relation must preserve non-contiguous increasing-index
  semantics and remain consistent with recurrence/witness wording.
- Interval scheduling: reject unresolved placeholders and reject benchmark-facing
  `True` placeholders; feasibility witness must use subset semantics.
- BFS: reject witness/minimality forms that use malformed adjacency or `∨ True`
  escape hatches; accept the adjacency spellings enumerated in the obligation
  quality gate above.
- BST-LCA: benchmark-facing lowestness must be direct descendant exclusion, not
  only helper-predicate indirection.
- Binary search: success theorem must derive in-bounds facts from `= some i`
  (including `i < arr.length` when the scaffold uses `Arr` / `length`, or the
  `i < arr.size` variants checked in `family_packet_regression`).
- Coin change: canonicality must be explicit in optimality theorem shape, not an
  opaque unused predicate.

## Documentation

- `docs/architecture.md` — components, crate boundaries, artifact flow
- `docs/benchmark_spec.md` — instance schema, domain/split policy
- `docs/annotation_manual.md` — rubric, adjudication, calibration
- `docs/evaluation_contract.md` — metric definitions, acceptance criteria
- `docs/release_process.md` — how to freeze / bump versions
- `docs/paper_readiness.md` — paper-track release gates and run protocol
- `docs/paper/outline.md` — paper structure and reproducibility checklist
- `docs/authoring_examples.md` — gold obligation patterns and review-packet bar
- `docs/obligation_audit_v0.1.md` — archived v0.1 obligation audit (with v0.2 pointers)
- `SECURITY.md` — reporting vulnerabilities and supply-chain posture
- `CONTRIBUTING.md` — development loop, coding conventions, PR checklist

## License

MIT. See `LICENSE`.
