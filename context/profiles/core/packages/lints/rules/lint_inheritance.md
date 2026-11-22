---
paths:
  - "Cargo.toml"
  - "*/Cargo.toml"
  - "*/*/Cargo.toml"
  - "*/*/*/Cargo.toml"
  - "*/*/*/*/Cargo.toml"
---

# Lint Inheritance

Declare lints once in the root manifest's `[workspace.lints]` and have every member crate opt in explicitly with `[lints] workspace = true` — inheritance is NOT implicit, and cargo ships the `missing_lints_inheritance` warn (plus an unused-root-table companion) precisely because teams get this wrong. A conservative landed baseline: root `Cargo.toml` carries `[workspace.lints.clippy] all = { level = "warn", priority = -1 }` (the same group and level a clippy wrapper may arm via `-W clippy::all` only for crates declaring no `[lints]` table, so for this warn-level baseline the finding set is unchanged and manifest levels govern every inheriting crate), and every member crate inherits it. Members may also set `autotests = false`: no auto-discovered `tests/` targets, a deterministic implicit target set — any test-placement policy is enforced by the repository's own checks, not by cargo target discovery.

- the inheritance mechanics, the landed manifests, rustfmt's `style_edition` pin, and the wrapper posture — [Lint Enforcement](../reference/lint_enforcement.md)
