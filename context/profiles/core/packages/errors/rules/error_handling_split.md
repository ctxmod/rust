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

# Error Handling Split

Libraries use thiserror for matchable error enums with protocol mappings. Applications use anyhow for ergonomic error chains with context. Convert between representations at service boundaries by implementing `From<DomainError>` for the protocol type (`tonic::Status`, HTTP status codes) there. Never return `anyhow::Result` from a library's public functions — callers cannot match — and do not build per-callsite error enums inside application crates — overhead without benefit.

Diagnostics carry stable machine codes from a maintained registry: one home per code, retired codes marked and never renumbered or reused, severity immutable — new behavior gets a new code. Corrupt input is an engine error with a causal `source()` chain, never a domain no-result.

Error catalogs with 50+ codes may switch to the single-struct + kind-enum pattern; below 50 the enum pattern is optimal. The flagship posture behind the split (2026 corpus, eight repos) is in Error Handling.

- the thiserror and anyhow worked examples, gRPC and HTTP protocol mappings, error-category retry classification, the enum-vs-struct scaling threshold, and the split's anti-patterns — [Error Handling](../reference/error_handling.md)
- the crate layers whose boundaries are the conversion points — [Workspace Organization](../../workspace/reference/workspace_organization.md)
- generated crates that consume the error taxonomy — [Code Generation Mechanisms](../../codegen/reference/code_generation_mechanisms.md)
- the binary-edge termination half of the split (exit codes, `ExitCode`) — [CLI Structure](../../cli/reference/cli_structure.md)
