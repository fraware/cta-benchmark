# CTA-Bench

**Classical Algorithm Tasks (CTA)** — a research benchmark and toolkit for studying how well systems turn problem descriptions and reference code into **Lean 4** proof obligations, and how faithful those obligations are to the intended algorithm.

---

## In one minute

| | |
|--|--|
| **What** | Curated programming tasks (sorting, graphs, dynamic programming, …), reference implementations, checks, and evaluation outputs. |
| **Why** | Measure and compare **semantic faithfulness** and related properties—not a general theorem prover or a full verification pipeline. |
| **How** | Rust tooling for runs and validation, Lean for checking generated obligations, Python scripts for tables and paper artifacts. |

**Scale (current headline release):** 84 task instances · 12 algorithm families · evaluation artifacts shipped in-repo for reproducibility.

---

## Who this is for

- **Researchers** reproducing or extending benchmark numbers or submitting follow-up work.  
- **Reviewers** checking claims against artifacts (see [Reproducibility](docs/reproducibility.md), [Reviewer map](docs/reviewer_map.md), and [REPRODUCE.md](REPRODUCE.md)).  
- **Contributors** improving tasks, tooling, or docs ([Contributing](CONTRIBUTING.md), [Code of conduct](CODE_OF_CONDUCT.md)).

---

## Quick start

**Requirements:** Rust **1.88.0** ([`rust-toolchain.toml`](rust-toolchain.toml)) · Lean **4.12.0** ([`lean/lean-toolchain`](lean/lean-toolchain)), e.g. via [elan](https://github.com/leanprover/elan).

```bash
cargo build --workspace
cargo test --workspace --all-targets
```

**Validate the benchmark and schemas** (after build):

```bash
cargo run -p cta_cli -- validate schemas
cargo run -p cta_cli -- validate benchmark --version v0.3 --release
```

**Check Lean scaffolds:**

```bash
cd lean && lake build
```

For a full command reference (generate, experiments, metrics, reports), see [**Contributing**](CONTRIBUTING.md) and [**Architecture**](docs/architecture.md).

---

## Reproducing paper-ready results

Headline tables and checks are produced by an **ordered** script pipeline. Do not skip steps or mix partial recipes—counts and strictness guarantees depend on the full sequence.

**Start here:** [`REPRODUCE.md`](REPRODUCE.md) · one-page env pins: [`docs/reproducibility.md`](docs/reproducibility.md) · artifact meanings: [`docs/reviewer_map.md`](docs/reviewer_map.md)

**Gate scripts** (run from repository root; order matters—see `REPRODUCE.md` and `scripts/run_paper_readiness_gate.*`):

```bash
python scripts/materialize_v03_adjudication_artifacts.py
python scripts/materialize_repair_hotspot_artifacts.py
python scripts/reproduce_agreement_report.py
python scripts/implement_evidence_hardening.py
python scripts/repair_counterfactual_metrics.py
python scripts/ci_reviewer_readiness.py
python scripts/check_paper_claim_sources.py
```

**Shell helpers:** `scripts/run_paper_readiness_gate.ps1` or `scripts/run_paper_readiness_gate.sh` · submission checks: `scripts/verify_submission_readiness.ps1` / `scripts/verify_submission_readiness.sh`

**NeurIPS 2026 E&D (Hugging Face):** public dataset card at
[`fraware/cta-bench`](https://huggingface.co/datasets/fraware/cta-bench). Build and upload
from a frozen branch with `pip install -r requirements-hf.txt`, `hf auth login` (or `HF_TOKEN`), then
`make hf-release` (see [`docs/reproducibility.md`](docs/reproducibility.md) for the ordered steps).

---

## What this project does *not* claim

- Full formal verification of arbitrary Rust programs  
- Solving all of interactive theorem proving  
- Ranking commercial models as a product leaderboard  
- Guaranteeing semantic correctness from Lean checks alone  

For precise limits and threats to validity, read [**Architecture**](docs/architecture.md) (non-goals and scope) and [**Evaluation contract**](docs/evaluation_contract.md).

---

## Repository layout

```
cta-benchmark/
├── benchmark/          # Versioned tasks (v0.1 … v0.3), instances, annotations
├── configs/            # Experiments, prompts, provider settings
├── schemas/            # JSON schemas for artifacts
├── crates/             # Rust library and CLI (`cta` binary)
├── lean/               # Lean 4 project tied to tasks
├── scripts/            # Python automation for tables and gates
├── docs/               # Specifications, evaluation contract, paper maps
├── tests/              # Integration and fixtures
├── runs/               # Local experiment outputs (gitignored by default)
└── reports/            # Generated reports (gitignored by default)
```

**Rust packages** (workspace):

| Package | Role |
|---------|------|
| `cta_core` | Shared types, IDs, versions |
| `cta_schema` | Load and validate JSON against schemas |
| `cta_benchmark` | Load tasks, lint, build manifests |
| `cta_rust_extract` | Signals from reference Rust code |
| `cta_generate` | Build candidate obligations from configs |
| `cta_lean` | Write Lean files, run checks |
| `cta_behavior` | Behavioral tests against specs |
| `cta_annotations` | Human and machine annotation flows |
| `cta_metrics` | Deterministic metrics |
| `cta_reports` | Tables and exports |
| `cta_cli` | Command-line entrypoint `cta` |

---

## Versions of the benchmark

- **v0.1** — Small pilot (immutable once released).  
- **v0.2** — Paper-oriented track with richer annotation and review packets.  
- **v0.3** — Larger grid (84 instances); primary surface for current headline numbers.

Released task definitions are **not rewritten in place**; new work adds a new version folder. Details: [`docs/architecture.md`](docs/architecture.md), [`CONTRIBUTING.md`](CONTRIBUTING.md).

---

## Documentation index

| Document | Contents |
|----------|----------|
| [Architecture](docs/architecture.md) | Components, data flow, and non-goals |
| [Evaluation contract](docs/evaluation_contract.md) | Metrics and definitions |
| [Reviewer map](docs/reviewer_map.md) | Paper sections ↔ files ↔ commands |
| [Annotation manual](docs/annotation_manual.md) | Rubric and review workflow |
| [Reproducibility](docs/reproducibility.md) | Toolchain pins and regen index |
| [REPRODUCE.md](REPRODUCE.md) | Ordered regeneration checklist |
| [Maintainers](docs/maintainers.md) | Security contact |
| [Citation](CITATION.cff) | Cite this repository |
| [Security](SECURITY.md) | Reporting issues, scope, scans |
| [Contributing](CONTRIBUTING.md) | PR expectations and deep CLI |
| [Code of conduct](CODE_OF_CONDUCT.md) | Community norms |

---

## License

MIT — see [`LICENSE`](LICENSE).
