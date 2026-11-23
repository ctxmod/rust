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

# CLI Structure

Every Rust binary is a thin `src/main.rs` over a library body — the lib/bin split exists from day one, and where build tooling discovers tools by path (a `tools/*/src/main.rs` glob, for instance) the entry point's location is what makes a crate discoverable as a tool. Parse arguments with clap's derive API, the ecosystem default; the builder API is for compile-time-sensitive or exotic cases only. There is no separate Rust CLI profile: this rule is the CLI guidance for any Rust binary the rust `core` profile reaches.

Exit codes follow the std strata: `0` success, `1` general failure (return `Err` from `main` and let `Termination` render it), `2` usage errors (clap's default), `101` panic. Prefer `std::process::ExitCode` / the `Termination` trait over `std::process::exit` — the latter skips destructors; at scale, an own `ExitCode` enum with `From<ExitCode> for process::ExitCode` (fd's pattern) keeps call sites typed. Errors stay typed enums at library boundaries and become `anyhow` + exit codes at the binary edge — never `miette`/`eyre`. A typical worked pair is a CLI and its lazily spawned daemon, both thin entries over library bodies.

- the clap derive/builder split, the exit-code table, the fd `ExitCode` worked pattern, the thin-main shape, and `.cargo/` zig-cc cross-compile wiring for macOS targets — [CLI Structure](../reference/cli_structure.md)
- the error-stack split this CLI edge consumes — [Error Handling](../../errors/reference/error_handling.md)
- the workspace wiring behind these crates (member layout, toolchain pin, MSRV) — [Workspace Organization](../../workspace/reference/workspace_organization.md)
