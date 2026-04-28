# Release process

This document describes how to freeze a benchmark version, add a new
version, and regenerate paper reports.

## Evidence-Hardening Update (2026-04-28)

For the current paper track, include these release-stage commands after the
paper metrics pass:

```powershell
python scripts\implement_evidence_hardening.py
python scripts\validate_release_artifact.py
python scripts\ci_reviewer_readiness.py
```

This enforces:

- human-pass v2 agreement exports (`annotation/human_pass_v2/*`);
- selector/token/cross-model/repair transparency outputs under `results/` and
  `repairs/`;
- artifact completeness plus checksum validation through
  `artifacts/evidence_hardening_manifest.json`.

## Freezing `v0.1`

1. Confirm every instance under `benchmark/v0.1/instances/**` passes:
   - `cta validate benchmark --version v0.1 --release`
   - `cta benchmark lint --version v0.1`
   - `lake build` from `lean/` (full scaffold elaboration)
2. Regenerate the benchmark manifest:
   `cta benchmark manifest --version v0.1`.
3. Commit the manifest. `content_hash` must not change after this commit
   except by version bump.
4. Tag the repository: `git tag benchmark-v0.1 -m "freeze v0.1"`.
5. From this point on, treat every file under `benchmark/v0.1/` as
   immutable. The CI `benchmark-lint` job fails if content_hash changes
   without a version bump.

## Adding `v0.2`

1. `cp -r benchmark/v0.1 benchmark/v0.2`.
2. Update `benchmark_version` in every `instance.json` and in
   `splits/*.json`.
3. Apply changes only within `benchmark/v0.2/`.
4. Repeat the freeze process for `v0.2`.

Paper-track note: from `v0.2` onward, release validation also enforces
held-out evaluation (`dev`/`eval` disjoint, `eval` >= 24), full annotation
coverage for any experiment that sets
`require_full_annotation_coverage: true`, and a two-reviewer gold audit
signoff (`benchmark/<version>/audit/gold_signoff.json`).

### v0.3 gold audit posture (template vs complete)

For `v0.3`, the repository may ship `benchmark/v0.3/audit/gold_signoff.json` in
**template** mode: set `release_gold_audit_status` to
`template_pending_human_review`, keep `"approved": false`, and use explicit
`Unassigned` reviewer lines until humans finish `audit/evidence/*.csv` (see
`benchmark/v0.3/audit/review_checklist.md`). `cargo run -p cta_cli -- validate benchmark --version v0.3 --release` accepts that posture.

To claim a completed audit, curators fill the evidence workbooks, update
`signoff_notes.md`, set real reviewer names, set `"approved": true`, and set
`release_gold_audit_status` to `human_audit_complete` (or remove the field to
use the legacy strict rule).

`v0.2/dev.json` is intentionally empty at this stage. Policy rationale:
for paper-track readiness we block on held-out `eval` quality first
(coverage + signoff + provider runs) and avoid mixing prompt-tuning
diagnostics into release gating. If a future cycle reintroduces active
dev-tuning, populate `dev.json` with non-overlapping instances and keep
`eval` disjoint.

Rule: never reuse an `instance_id` across versions with different
semantic content. If an instance changes meaning, give it a new id and
increment the 3-digit suffix (e.g. `arrays_binary_search_002`).

## Paper-track closure flow (`v0.2`)

Use this sequence as the authoritative release path:

1. Initialize and track annotation queue:
   - `cta annotate plan --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --out benchmark/v0.2/annotation/task_board/`
   - `cta annotate batches --benchmark-version v0.2 --missing-pairs benchmark/v0.2/annotation/task_board/missing_pairs.json --out benchmark/v0.2/annotation/task_board/batches/`
2. Sync adjudicator outputs and rebuild coverage:
   - `cta annotate sync-review-packets --benchmark-version v0.2 --from benchmark/v0.2/annotation/review_packets --out benchmark/v0.2/annotation/adjudicated_subset/`
   - `cta annotate pack --version v0.2 --from-benchmark`
   - `cta annotate coverage --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --pack benchmark/v0.2/annotation/adjudicated_subset/pack.json --out benchmark/v0.2/annotation/adjudicated_subset/`
3. Prepare audit workbook and collect human signoff:
   - `cta benchmark audit-workbook --version v0.2`
   - update `benchmark/v0.2/audit/gold_signoff.json` with real reviewer names and `approved: true`
4. Run release gate:
   - If instances or scaffolds changed, refresh `benchmark/v0.2/manifests/benchmark_manifest.json` first with `cta benchmark manifest --version v0.2` so `MANIFEST_CONTENT_HASH_STALE` cannot fire.
   - `cta validate benchmark --version v0.2 --release` (after step 3; until `gold_signoff.json` carries two non-empty reviewer names and `"approved": true`, expect `GOLD_AUDIT_SIGNOFF_INVALID` as the sole release error on an otherwise green tree).
5. Refresh Lean proof-status and enforce strict M1 contract (mutates
   `packet.json` / diagnostics / dashboards):
   - `cta annotate refresh-lean-check --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --strict-m1`
   - expected current output baseline: `m2_ready_packets = 94 / 94`, empty global proof worklist
6. Run review-packet audit gate (must pass before packaging; run **after**
   refresh so `verification_summary.signed.json` hashes final `packet.json`):
   - `cta annotate verify-review-packets --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --schema schemas/review_packet.schema.json --out benchmark/v0.2/annotation/review_packets/verification_summary.signed.json`
7. Run experiment and package paper artifacts:
   - `cta experiment --config configs/experiments/benchmark_v1.json`
   - `cta reports package --benchmark-version v0.2 --canonical-run-ids <run_id_1>,<run_id_2>,...`

For automation, the fail-fast orchestrator chains these gates:

`cta benchmark paper-orchestrate --benchmark-version v0.2 --canonical-run-ids <run_ids>`

`paper-orchestrate` runs `annotate refresh-lean-check --strict-m1` first, then
`annotate verify-review-packets`, before `reports package`, and exits non-zero
if either gate fails.

## Rigorous release note (`2026-04-24`, updated)

Before tagging a paper-track release, explicitly verify curated hardening for:

- `sorting_insertion_sort_{001,002}`, `sorting_merge_sort_{001,002}`,
  `trees_bst_insert_{001,002}` (all four review-packet systems), and
- `dp_knapsack_01_{001,002}` (all four systems, aligned with
  `lean/CTA/Benchmark/DP/KnapsackTheory.lean`).

Checklist:

1. no packet `lean_statement` begins with `axiom`,
2. no curated release candidate reports `proof_mode: "axiom_backed"` where
   the bar is definition-backed completion,
3. packet obligations avoid benchmark-facing vacuous/trivial theorem forms,
4. strict-M1 benchmark-facing obligations avoid wrapper self-copy shapes
   (`(h : P) : P := by exact h` / `simpa using h`) and tautological theorem
   equalities (`x = x`).

The first two are machine-checkable with `rg`; the third is enforced by
`review_packet_lean_lint` and the focused regression tests plus manual signoff
during release prep. Full **M1 elaboration** (`lean_check.elaborated = true`)
is required only for pairs in `is_m1_target_packet`
(`crates/cta_cli/src/cmd/annotate.rs`); run
`annotate refresh-lean-check … --strict-m1` and confirm zero M1 violations in
`proof_completion_dashboard.csv`.

## Code-only remediation protocol

When `code_only_v1` packets show scaffold-heavy or vacuous obligations, run
this targeted remediation loop before broad benchmark refresh:

1. Tighten prompt constraints in `configs/prompts/code_only_v1.json`:
   - benchmark-facing obligations first
   - optional auxiliary obligations second
   - no vacuous theorem forms (`True`, `P -> True`, `P ∧ True`, `∃ x, True`)
   - no off-spec promotion in benchmark-facing output
2. Tighten normalizer filtering (`crates/cta_generate/src/normalize.rs`):
   - drop vacuous obligations
   - demote off-spec extras to auxiliary
3. Regenerate only the scoped packet set with `cta generate --instances ...`.
4. Rebuild scoped packets with `cta annotate build-review-packets --pairs ...`.
5. Run packet regression and schema gates:
   - `cargo test -p cta_generate --test code_only_packet_regression`
   - `cargo test -p cta_generate --test family_packet_regression`
   - `cta annotate verify-review-packets ...`

The exact instance ids covered by `code_only_packet_regression` live in the
`targets` array inside `crates/cta_generate/tests/code_only_packet_regression.rs`
(currently nineteen instances spanning arrays, graphs, greedy, sorting, trees,
and DP, including both Dijkstra instances, both knapsack instances, LCS `001`,
both LCA instances, `trees_bst_insert_002`, and both insertion-sort ids). Extend
that array when you add a new first-class exemplar packet so it cannot silently
rot.

The `naive_concat_packet_regression` `targets` array includes the same style of
pilot list (including both knapsack instances) so naive-concat gold packets
cannot drift on layers, `quality_summary`, or vacuity.

`code_only_packet_regression` must fail on each of the following malformed cases:

- interval witness theorem using `∀ iv, iv ∈ S ↔ iv ∈ intervals` for selected subsets
- BFS path-edge theorem using malformed self-membership forms like `p.get? i ∈ adj[p.get? i]`, or omitting valid consecutive-vertex adjacency (`w ∈ adj[u]`, `list.mem w (adj[u].tolist)`, or `(adj.get? u).getd []` alongside `u = p[i]` / `w = p[i+1]` as appropriate to the scaffold)
- BST benchmark-facing key-change theorem encoded as implication-disjunction instead of absent/present multiset split
- Dijkstra benchmark-facing preconditions containing redundant `w ≥ 0` / `w >= 0` clauses when edge weights are already `Nat`

`family_packet_regression` must fail on the following family-specific regressions:

- LCS subsequence semantics drifting to contiguous embedding
- interval scheduling witness not encoding subset selection
- BFS witness/minimality edge clauses not using consecutive-vertex adjacency with one of the adjacency spellings accepted above
- BST-LCA lowestness expressed only via helper abstractions
- binary-search success theorem assuming bounds instead of proving them from `binarySearch … = some i` (must chain to `i < arr.size` or `i < arr.length` per `family_packet_regression`)
- coin-change canonicality appearing without explicit optimality dependence

## Family remediation execution order

For broad cross-family cleanup cycles, follow this exact order:

1. patch `dp_longest_common_subsequence_002`
2. patch `greedy_interval_scheduling_002`
3. patch `graph_bfs_shortest_path_001` and `graph_bfs_shortest_path_002`
4. patch `trees_lowest_common_ancestor_001` and `trees_lowest_common_ancestor_002`
5. patch `arrays_binary_search_002`
6. patch `greedy_coin_change_canonical_001` and `greedy_coin_change_canonical_002`
7. run `code_only_packet_regression` + `family_packet_regression`
8. rebuild packets and run `annotate verify-review-packets`

Focus-first policy: do not broaden instance scope until the targeted packet
set is clean under both regression checks and packet schema verification.

## Naive-concat remediation protocol

Apply the same quality discipline to `naive_concat_v1`:

1. Tighten prompt constraints in `configs/prompts/naive_concat_v1.json`:
   - benchmark-facing obligations first
   - optional auxiliary obligations second
   - no vacuous/filler theorem forms
   - semantic-unit linked obligations using canonical SU ids
2. Reuse normalizer filters (`crates/cta_generate/src/normalize.rs`) so
   vacuous obligations are dropped and off-spec extras are demoted.
3. Regenerate only the scoped packet set with `cta generate --instances ...`.
4. Rebuild scoped packets with `cta annotate build-review-packets --pairs ...`.
5. Run focused regression and packet verification:
   - `cargo test -p cta_generate --test naive_concat_packet_regression`
   - `cargo test -p cta_generate --test text_only_packet_regression`
   - `cta annotate verify-review-packets ...`

As with `code_only_v1`, do not broaden to full-system refresh until the
scoped packet set is clean under regression + schema gates.

## Schema evolution

- Additive, non-breaking changes (new optional fields): bump the schema to
  `schema_v2` in a new file, update `cta_schema` to load both, and leave
  existing artifacts alone.
- Any breaking change requires `schema_v<n+1>` and bumping
  `benchmark_version` so the artifacts under the old schema remain valid
  in place.

## Metrics evolution

Metric names are frozen under `metrics_v2`. Never redefine an existing
metric. Introduce new metrics in `metrics_v3` and record the contract
version in every `run_manifest.json`. `metrics_v1` is retained only for
archival comparison; the current pipeline emits `metrics_v2` by default.
Schema validation allows `metrics_vN` for archival runs, while paper
aggregation should enforce the current contract version.

## Rubric evolution

Annotation rubrics are frozen under `rubric_v<n>`. Adjudicated records
under the old rubric remain authoritative for runs that used it. New
rubric versions may only be used on new annotation batches.

## Regenerating paper reports

1. Pick a committed run id under `runs/`.
2. Run `cta reports build --run <run_id>`. Reports are emitted next to
   the source bundle at `runs/<run_id>/reports/` and include a per-system
   primary metrics CSV, per-instance CSV, Markdown summary, and LaTeX
   table (override the destination with `--out <dir>`).
3. Diff the committed reports against the regenerated ones; a clean diff
   (modulo timestamps) is required before publication. The exact CSV,
   Markdown, and LaTeX shape is pinned by snapshot tests under
   `crates/cta_reports/tests/snapshots/`, so any non-trivial diff is a
   contract change and must be accompanied by a snapshot update and a
   metrics-version bump.

## Versioning toward `v0.4`

Until `benchmark/v0.4/` exists, all release energy stays on tightening `v0.3`
(protocol freeze, gold audit posture, provider baselines). Opening `v0.4` is
explicitly gated: new instances live only under a new benchmark directory with
their own `splits/`, `protocol_freeze.json`, and manifest rows; partial edits
to `v0.3` instances after freeze require a new `instance_id` suffix policy
described in `docs/benchmark_spec.md`.

Compatibility checklist when bumping benchmark version:

1. Duplicate the prior tree (`benchmark/v0.N` → `benchmark/v0.{N+1}`) and bump
   every `benchmark_version` field in instances and splits.
2. Regenerate `benchmark/manifest.jsonl` and `benchmark_paper_summary.json`.
3. Re-run `cargo test` plus `cta validate benchmark --version v0.{N+1} --release`.
4. Refresh `docs/PROVENANCE.md` and `docs/REVIEWER_MAP.md` rows for moved paths.
