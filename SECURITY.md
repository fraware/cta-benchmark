# Security policy

## Supported versions

Only the `main` branch of `cta-benchmark` receives security fixes.
Tagged benchmark versions (e.g. `v0.1`) are immutable by design and are
**not** patched in place; vulnerabilities that require a fix will roll
forward into the next tagged benchmark version.

## Reporting a vulnerability

Please report security vulnerabilities privately rather than opening a
public issue.

1. Use GitHub's private
   [security advisories](https://github.com/fraware/cta-benchmark/security/advisories/new)
   form on this repository, or
2. Email the maintainers listed in `Cargo.toml` (`authors`) with a
   minimal reproducer.

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

## Hardening expectations

- `cta_generate` performs no network calls during `cargo build`.
- Live providers (`OpenAiProvider`, `AnthropicProvider`) require their
  credential environment variable to be present; otherwise they refuse
  to run and surface a typed `GenerateError::Provider`.
- All JSON artifacts governed by `schemas/*.schema.json` are validated
  before writing to disk and before downstream metrics consumption.
