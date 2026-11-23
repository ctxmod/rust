---
paths:
  - "*.rs"
  - "*/*.rs"
  - "*/*/*.rs"
  - "*/*/*/*.rs"
  - "*/*/*/*/*.rs"
  - "*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*/*/*.rs"
---

# Fixture-Driven Verification

Conformance and behavior verification for parsers, compilers, and emitters is fixture-driven: fixtures are committed data consumed by a black-box harness, never conventional test files and never product test modes.

- Fixture sets live under a schema-validated tree (for example `tests/fixtures/**` beside the harness tool) with a `data/` subtree for byte-exact inputs and an `invalid/` subtree for negative controls; every fixture validates against the schema at load.
- Every concrete scenario cites stable conformance case IDs traced to spec sections; closure is bidirectional — every case ID has at least one scenario, every scenario cites at least one — and the census is re-runnable arithmetic, not judgment.
- Expected values come from an independent oracle: spec text, a second independently derived implementation, or a specified relation (round-trip, metamorphic) — never the implementation under test; a data file used as input anywhere never serves as a snapshot source anywhere.
- Every `invalid` fixture declares its rejection (`expect-invalid:`) and the harness proves every file rejected as declared; a positive-only suite cannot detect over-acceptance — the parser-security failure mode.
- Never-blessable safety classes: exit codes, absence, no-change, and containment results are never accepted by regenerating a snapshot; only declared blessable oracle classes may be rewritten, only single-oracle data files — never fixture YAML, never inputs.
- Vacuity defenses: a filter selecting zero scenarios is a failure, not a pass; overwrite-mode environments fence the harness; detection meta-fixtures prove the harness fails exactly where declared before product work proceeds.
- Under an effective-uid-0 harness, permission-dependent assertions record `indeterminate` — evidence without a verdict — never a silent pass.
- the corpus model, case-ID closure censuses, oracle independence, the blessability taxonomy, adversarial coverage, verification honesty, and harness-before-product sequencing — [Fixture-Driven Verification](../reference/fixture_verification.md)
