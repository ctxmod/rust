---
paths:
  - "Cargo.toml"
  - "*/Cargo.toml"
  - "*/*/Cargo.toml"
  - "*/*/*/Cargo.toml"
  - "*/*/*/*/Cargo.toml"
  - ".clippy.toml"
  - "*/.clippy.toml"
  - "*/*/.clippy.toml"
  - "*/*/*/.clippy.toml"
  - "*/*/*/*/.clippy.toml"
  - "clippy.toml"
  - "*/clippy.toml"
  - "*/*/clippy.toml"
  - "*/*/*/clippy.toml"
  - "*/*/*/*/clippy.toml"
---

# Clippy Configuration

Clippy is the unified linting standard: `clippy::all` at deny level plus selective `clippy::pedantic` lints at warn — never blanket pedantic deny (false positive rates); enable individual pedantic lints only after team evaluation. The `clippy::restriction` category is cherry-picked only, due to internal contradictions: `unwrap_used`, `todo`, `unimplemented`, and `panic` denied, `expect_used` warned.

The cargo shape is `cargo clippy --workspace --all-targets -- -D warnings`; where a repository routes lints through its own check wrapper, use the wrapper — it is the gate, not an optional convenience. Triage narrowest-first when a finding surfaces: per-crate check → per-crate clippy → workspace lint → any repository-level file scans. The manifest lint baseline lives in `[workspace.lints]` with explicit member opt-in (see lint-inheritance): a wrapper's command-line flag must arm the same group only for crates without a `[lints]` table, so manifest levels are never overridden.

Suppress specific lints only through `#[allow(...)]` annotations carrying documented rationale. cargo-deny (dependency licensing and advisories) and cargo-machete (unused dependencies) are configured alongside. Formatting is part of the same gate: root `rustfmt.toml` pins `style_edition = "2024"` and `cargo fmt --all --check` runs beside the lint stages.

- the base and selective-pedantic Cargo.toml blocks, the lint-inheritance mechanics and landed manifests, the rustfmt pin, the wrapper posture and escalation ladder, cargo-deny and cargo-machete configuration, and the allow-annotation exception — [Lint Enforcement](../reference/lint_enforcement.md)
- the workspace principles the lints enforce — [Workspace Organization](../../workspace/reference/workspace_organization.md)
- the error-handling principles the lints enforce — [Error Handling](../../errors/reference/error_handling.md)
