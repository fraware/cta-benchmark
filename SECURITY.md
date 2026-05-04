# Security policy

## Supported versions

Only the `main` branch of `cta-benchmark` receives security fixes.
Tagged benchmark versions (e.g. `v0.1`) are immutable by design and are
**not** patched in place; vulnerabilities that require a fix will roll
forward into the next tagged benchmark version.

## Reporting a vulnerability

Please report security vulnerabilities privately rather than opening a
public issue.

1. On GitHub, open this repository’s home page, click **Security**, then
   **Report a vulnerability** to start a private advisory (preferred), or use
   the same flow from the **Security** tab’s advisories list for this repo.
2. Alternatively, email the security contact listed in
   [`docs/maintainers.md`](docs/maintainers.md) with a minimal reproducer.

You should receive an acknowledgement within **3 business days**. We
aim to publish a fix or coordinated disclosure within **30 days** of
acknowledgement, sooner for high-severity issues.

## Scope

The benchmark toolchain does not handle user data or secrets directly.
In-scope concerns include:

- Supply-chain issues flagged by `cargo-deny` / `cargo-audit`
  (see `.github/workflows/supply-chain.yml`).
- Schema-validation bypasses that allow non-conforming artifacts to
  round-trip through `cta validate`.
- Deterministic-reproducibility regressions that cause a sealed run to
  drift without a benchmark version bump.
- Code execution paths in `cta_behavior` and `cta_lean` that spawn
  external processes (`lake env lean`) without the documented timeouts.

Out-of-scope:

- Live provider API keys: the benchmark treats `OPENAI_API_KEY`,
  `ANTHROPIC_API_KEY`, and any provider-specific environment variables
  as untrusted secrets that **must never** be committed or logged.
  Misuse of these variables by a contributor is an operator issue, not
  a benchmark vulnerability.

## Misuse (public repository threat model)

This project is a **research benchmark and offline toolchain**, not a hosted
inference service. Public release increases automated scraping of prompts,
configs, and committed model outputs. That behavior may violate provider terms
of service or local policy; it is outside the vulnerability scope above, but
maintainers may block abusive traffic at the platform level. Do not use the
benchmark as a general-purpose remote code execution service; `cta_behavior`
and Lean drivers spawn local subprocesses with documented timeouts only.

## Hardening expectations

- `cta_generate` performs no network calls during `cargo build`.
- Live providers (`OpenAiProvider`, `AnthropicProvider`) require their
  credential environment variable to be present; otherwise they refuse
  to run and surface a typed `GenerateError::Provider`.
- All JSON artifacts governed by `schemas/*.schema.json` are validated
  before writing to disk and before downstream metrics consumption.
- See [`.env.example`](.env.example) for variable names only (never commit `.env`).
- Full-history secret scan (maintainers / CI): [`scripts/scan_secrets_history.ps1`](scripts/scan_secrets_history.ps1) or [`scripts/scan_secrets_history.sh`](scripts/scan_secrets_history.sh) (Docker + [`.gitleaks.toml`](.gitleaks.toml)); the same configuration runs in [`.github/workflows/gitleaks.yml`](.github/workflows/gitleaks.yml) on pushes and pull requests to `main`.
