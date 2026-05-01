# Claim lock — NeurIPS 2026 (E&D)

This file freezes **wording posture** and **evidence tiering** for the NeurIPS 2026 Evaluations & Datasets submission. Code and tables must not contradict it. Companion one-pager: [`EVIDENCE_NOTE_NEURIPS2026.md`](EVIDENCE_NOTE_NEURIPS2026.md).

**Primary scientific posture:** CTA-Bench is a **benchmark and evaluation protocol** for measuring **semantic faithfulness** in **Lean-facing correctness-obligation generation** from text plus code. It is **not** full Rust verification, full theorem proving, or an end-to-end proof-agent leaderboard.

---

## Tier A — Headline claims

Allowed in abstract, introduction, results, and conclusion.

- CTA-Bench evaluates **semantic faithfulness** of **Lean-facing correctness obligations**.
- **CTA-Bench v0.3** contains **84 instances** across **12 algorithm families**.
- The **strict headline view** contains **274 direct strict rows** over **84** unique instances, with **no** `mapped_from_canonical` rows.
- The **expanded view** contains **336** rows over **84** instances, including **114** `mapped-from-canonical` rows, and is **appendix / robustness evidence only** (not headline).
- The **strict evidence layer** has **independent human agreement** artifacts and **disagreement logs** over the strict overlap.
- **Code-grounded conditioning** (system id `code_only_v1`) is the **strongest semantic baseline** in the **current strict direct-adjudication view** (same-model conditioning study).
- **Scaffolded / full-method** conditioning can **improve proof-facing structure** while still producing **polished semantic failures**; do not generalize past this release and this grid.
- CTA-Bench **separates** semantic faithfulness, critical-unit coverage, code consistency, vacuity, proof utility, and proof-facing maturity.

**Approved headline sentence (human agreement):**  
“The strict headline view is independently double-annotated and adjudicated over **274** direct rows covering all **84** instances.”

**Do not** claim the full **336**-row expanded grid is independently double-annotated in the same sense.

---

## Tier B — Supporting claims

Allowed in methods, artifact description, appendix, limitations.

- The repository uses a **Rust workspace**, **Lean** project, **Python** scripts, **JSON** schemas, **reports**, **annotations**, **repairs**, and **experiments**.
- **Rust** is pinned via `rust-toolchain.toml`; **Lean** is pinned via `lean/lean-toolchain`.
- **Strict headline** tables and numbers are generated from **`results/raw_metrics_strict.json`** and the **`results/paper_strict_*`** exports.
- **Expanded** outputs are **not** headline evidence.
- **Repair** examples are **diagnostic**, not universal repair rates or guarantees.
- The **proof-facing** subset demonstrates **elaboration / proof-engineering maturity** for selected packets, **not** full implementation verification.
- **`docs/REVIEWER_MAP.md`** separates strict headline metrics from expanded mapped metrics and maps topics to regen commands.

**Composite reliability score:**  
The composite is **not** the construct being validated. It is a **compact secondary diagnostic** for multi-axis failure. **Primary** claims use semantic faithfulness, coverage, code consistency, vacuity, proof utility, and failure-mode analysis **separately**. Any sensitivity analysis for composite weights belongs in the **appendix**.

**Cross-model sanity pilot (`results/cross_model_*`):**  
The repo ships a **small, strict-metrics-derived slice** (twelve instances, one per family) under `results/cross_model_pilot_*` to show how the **same two conditioning regimes** behave on that slice. It is **diagnostic**, **not** a multi-provider leaderboard. Extending it with additional public models is optional hardening, **not** required for Tier A strict claims.

---

## Tier C — Claims to avoid or weaken

Do **not** say (without qualification):

- “CTA-Bench solves autoformalization.”
- “CTA-Bench verifies Rust programs.”
- “CTA-Bench proves generated obligations correct.”
- “Full-method is worse/better **in general**.”
- “Code-only is the best model/system **overall**.” (Use **code-grounded**; keep id `code_only_v1` in tables.)
- “The repair subset proves repairability at scale.”
- “The proof-facing subset proves semantic correctness.”

**Prefer**

- “in this release,”
- “in the **strict direct-adjudication** view,”
- “under this **same-model conditioning** study,”
- “**diagnostic** rather than leaderboard evidence,”
- “**proof-facing maturity**, not full verification.”

**Terminology:** In prose, say **code-grounded baseline** for the conditioning regime identified as `code_only_v1` in artifacts. The system id remains `code_only_v1` for historical compatibility; the regime still receives the **problem summary** and **Rust-derived** context, not “raw code alone.”

---

## Manuscript number discipline

- All **headline** numerics must be traceable to **`results/raw_metrics_strict.json`**, **`results/paper_strict_*`**, and the **strict** rows of **`results/paper_table_annotation_evidence.csv`** / **`results/paper_table_agreement_evidence.csv`**, as enforced by `python scripts/check_paper_claim_sources.py` and `docs/paper/paper_claim_sources.yaml`.
