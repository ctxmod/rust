---
paths:
  - "Cargo.toml"
  - "*/Cargo.toml"
  - "*/*/Cargo.toml"
  - "*/*/*/Cargo.toml"
  - "*/*/*/*/Cargo.toml"
  - "*/*/*/*/*/Cargo.toml"
  - "*/*/*/*/*/*/Cargo.toml"
  - "*/*/*/*/*/*/*/Cargo.toml"
  - "*/*/*/*/*/*/*/*/Cargo.toml"
  - "*/*/*/*/*/*/*/*/*/Cargo.toml"
---

# Workspace Organization

Multi-crate projects use Cargo workspaces with virtual root manifests (no `[package]` in the root) and layer-based crate separation. The dependency graph must form a DAG: `core` (shared utilities, no internal dependencies) → `types` (domain types) → `domain` (business logic) → `apps` (binary crates), each layer depending only on lower layers, never same-level or higher. Root `Cargo.toml` unifies third-party versions through `[workspace.dependencies]`; member crates opt in with `serde = { workspace = true }`.

New crates use edition 2024 (Rust 1.85+), shared with `rust-version` through `[workspace.package]`. Supply-chain posture: declare and verify `rust-version` (MSRV) with MSRV-aware resolution enabled workspace-wide (`resolver = "3"`, set explicitly in a virtual manifest), consider a dependency cooldown (`global-min-publish-age`, Cargo 1.100+), pin git dependencies to `rev`, vendor for hermetic offline source builds, and keep a provenance ledger pinning every external claim to a primary source.

Split crates at 500+ LOC or when independent publishing is needed — fine-grained (>10 crates) suits independent versioning needs, coarse-grained (<5 crates) suits early-stage development. Single-crate projects below the threshold skip workspace structure and graduate when they cross it or need publishing.

- the layer-pattern directory tree, workspace and member manifest blocks, granularity guidance, the Module Organization section for within-crate feature modules, and the circular-dependency and premature-splitting anti-patterns — [Workspace Organization](../reference/workspace_organization.md)
- the edition 2024 changes, the stable language features since (let chains, async closures, precise capturing, `LazyLock`, `#[expect]`, the never type), and what is still unstable — [Edition and Language Baseline](../reference/language_baseline.md)
- unit and integration test placement across workspace crates follows the repository's test policy; the runner mechanics (rstest, cargo-nextest) live in the rust `verification` profile's test-runners reference
