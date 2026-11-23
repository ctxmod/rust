---
description: >-
  Index of the rust module's normative references: the core profile's
  structure, error, CLI, code-generation, and lint references, and what
  the data_integrity, verification, capnp, and capnp_rpc profiles add.
binds_to_depth: main...
---

# Rust

Rust development conventions covering workspace organization, error
handling, linting, testing, and visibility standards.

## Normative references

- **Structure and errors** (this profile) — [edition and language baseline](../workspace/reference/language_baseline.md) · [workspace organization](../workspace/reference/workspace_organization.md) · [visibility](../visibility/reference/visibility.md) · [error handling](../errors/reference/error_handling.md) · [CLI structure](../cli/reference/cli_structure.md) · [code generation](../codegen/reference/code_generation_mechanisms.md)
- **Linting** (this profile) — [lint enforcement](../lints/reference/lint_enforcement.md)
- **Input hardening and data integrity** (the `data_integrity` profile) — input hardening · persistence envelopes · source spans · path containment · canonical data · determinism · index freshness and benchmarks
- **Validation** (the `verification` profile) — test runners (rstest, cargo-nextest, doctests) · property testing · fixture-driven verification
- **Cap'n Proto** (the `capnp` and `capnp_rpc` profiles) — zero-copy serialization, code generation, schema validation and evolution; promise pipelining, reconnection, RPC error handling, and WebSocket tunneling

Select the opt-in profiles by name when the work needs them: a tool that
persists, parses, or indexes data selects `data_integrity` (which brings
`verification` with it); conformance and property work selects
`verification`.

## Examples

- `layered_workspace` — a four-crate virtual-manifest workspace in
  dependency layers (utils → types → domain → app).
- `lib_patterns` — a library crate with thiserror error enums,
  visibility controls, feature modules, and `#[non_exhaustive]`.
