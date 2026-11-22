# Workspace Organization

Cargo workspaces organize multi-crate projects with virtual root manifests, layer-based crate separation, DAG dependency structures, and supply-chain provenance (MSRV, git-dependency pinning, vendoring, and a provenance ledger). The workspace pattern scales from 2-crate projects to monorepos with 50+ crates without restructuring.

The examples below place crates under `pkg/rs/`, the layout of a polyglot monorepo that keeps each language's packages under `pkg/<language>/`. A Rust-only repository commonly uses `crates/` instead; substitute the member root throughout — nothing in the doctrine depends on the directory name.

## Core Concepts

- **Virtual manifest**: Root `Cargo.toml` declares `[workspace] members = ["pkg/rs/**/"]` without `[package]` section. Member crates maintain individual manifests with `[package]` metadata. This separates workspace-level concerns (dependency versions, shared metadata) from crate-level concerns (library vs binary, specific dependencies)
- **Layer-based separation**: Crates organize into dependency layers with strict DAG structure: `core` (no dependencies) → `types` (domain types) → `domain` (business logic) → `apps` (binaries). Circular dependencies fail at compile time. Each layer depends only on lower layers, never on same-level or higher-level crates
- **Workspace dependencies**: Root `[workspace.dependencies]` unifies third-party versions across crates, preventing version drift and reducing lock file churn. Adopt workspace dependencies when the workspace exceeds 3 crates. Path dependencies use `version` field for crates.io publishing, `path` field for local development
- **Feature-based modules**: Within crates, organize modules by domain features (user, auth, payment) rather than technical layers (handlers, models, services). Flat module structures outperform deep nesting for discoverability. Maximum 3 levels of nesting; prefer 2 levels

## Dependency Structure

The dependency graph must form a DAG (directed acyclic graph). Cycles fail at compile time with "cyclic package dependency" errors. This constraint enforces clean architecture boundaries.

### Layer Pattern

```
pkg/rs/
├── gen/                # Generated crates (DO NOT EDIT)
│   ├── types/
│   │   └── src/lib.rs
│   └── protocols/
│       └── src/lib.rs
├── core/               # Shared utilities (no internal dependencies)
│   └── utils/
│       └── src/lib.rs
├── domain/             # Business logic (depends on types, core)
│   ├── user/
│   │   └── src/lib.rs
│   └── auth/
│       └── src/lib.rs
└── app/                # Binary crates (depend on everything)
    ├── cli/
    │   └── src/main.rs
    └── server/
        └── src/main.rs
```

### Workspace Dependencies

Root `Cargo.toml`:

```toml
[workspace]
members = ["pkg/rs/**/"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
tokio = { version = "1.0", features = ["full"] }
```

Member crate `Cargo.toml`:

```toml
[package]
name = "my-domain"
version = "0.1.0"

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }

my-types = { path = "../gen/types" }
```

Generated crates in `pkg/rs/gen/` are regular workspace members with no special Cargo configuration. They follow the same workspace dependency patterns as handwritten crates.

## Root Files

The workspace root carries four config files, each with a reason to exist:

- **`Cargo.toml`** — the virtual manifest (`[workspace]`, no `[package]`), plus the shared lint baseline under `[workspace.lints]` consumed by every member's explicit `[lints] workspace = true` (mechanics in the lint-enforcement reference).
- **`rust-toolchain.toml`** — the toolchain pin, for example `channel = "1.94.1"`, with two deliberate choices: the channel matches the host rustup default so no network fetch is needed, and there is no `profile = "minimal"` because the clippy and rustfmt runners need the default profile's components. Pinning can be a deliberate asymmetry with other languages in the same repository (a Go side that always tracks the latest toolchain, say): when the Rust surface is a few crates with exact-pinned dependencies where regeneration must not require the network, the compiler moves only when a deliberate decision moves it.
- **`rustfmt.toml`** — `style_edition = "2024"`. Standalone rustfmt defaults to the 2015 style when no config is found; the pin matches the crates' edition and rust's own formatting posture. No other rules: rustfmt defaults are the style contract.
- **`Cargo.lock`** — committed. The lockfile freezes resolution; every cargo invocation after the initial bounded fetch runs `--offline`.

## Supply-Chain Provenance

Dependency posture is build substrate: resolver behavior and provenance rules apply from the first `cargo add`, before any layering above can matter.

### MSRV and Toolchain Policy

- Declare `rust-version` in every member manifest and verify the floor holds (`cargo-msrv` find/verify). The declared MSRV and the committed `rust-toolchain.toml` pin are different documents: the pin fixes what this machine builds with; `rust-version` declares the oldest toolchain the crates support. A pin posture that differs from another language's toolchain policy is recorded under Root Files above.
- Enable MSRV-aware resolution workspace-wide: `resolver.incompatible-rust-versions = "fallback"` in `.cargo/config.toml` (opt-in since 1.84; the implied default for edition 2024). **A virtual manifest must set the resolver explicitly** — the virtual root has no edition of its own, so the implied default silently does not apply workspace-wide.
- MSRV-aware selection changes which dependency versions resolve: without it, a freshly published crate requiring a newer compiler enters the graph and the declared floor becomes fiction.

### Git-Dependency Pinning

- A git dependency carries exactly one of `rev`, `tag`, or `branch`; prefer `rev`. The chosen commit is locked in `Cargo.lock` and moves only through `cargo update`.
- The `#<commit>` URL-fragment form is documented-invalid — a resolver that ignores it silently tracks the default branch while the manifest looks pinned.
- The same commit reached through different URLs registers as two crates; canonicalize the URL once and reference it everywhere.

### Vendoring and Hermetic Builds

- `cargo vendor` with `--versioned-dirs` plus a source replacement in `.cargo/config.toml` makes source builds hermetic and offline; document the `cargo fetch` bootstrap a fresh clone runs before the vendor directory exists.
- Known offline failure modes to design around: feature combinations over optional dependencies that the vendor directory cannot satisfy (cargo#10352), and git dependencies tripping `--frozen` even with a current lockfile.

### Provenance Ledger

Keep a pins table for every external claim — version, behavior, benchmark, advisory:

| Claim | Pin | Primary source | Verified by | Status |
|---|---|---|---|---|

- **One version per crate**: a document citing a different version than the ledger is a finding, not a style issue. Changing a pin re-verifies every row for that crate.
- **Survey pins for rejected candidates** stay in the ledger with the rejection reason, so a future re-survey does not re-read them.
- **Status is binary** — verified or unverified; "probably" is unverified.
- **`Cargo.lock` nuance, stated explicitly**: binaries commit the lockfile; `staticlib`/`cdylib` crates are end products and commit it too; the conventional library exception is deliberate for a monorepo shipping binaries.
- At larger scale: **cargo-vet** adds a trust network for shared audits (audit / relative / deferred audit mechanisms) — the named option if a project grows an audit network; **cargo-semver-checks** guards published-library semver — relevant only when library crates are published. Neither is baseline tooling for a binary-shipping workspace.

## Granularity Guidelines

Crate granularity balances recompilation time against complexity:

- **Split at 500+ LOC per crate** or when independent publishing needed
- **Fine-grained**: >10 crates, 50k+ LOC, independent versioning needs, independent publishing workflows
- **Coarse-grained**: <5 crates, early-stage development, internal-only, rapid iteration phase

Graduate from coarse to fine when: crate exceeds 500 LOC, independent publishing becomes necessary, or recompilation times impact development velocity.

## Anti-Patterns

- **Circular dependencies**: crates A and B depend on each other. Compiler rejects with cyclic dependency error. Solution: extract shared code to crate C that both A and B depend on
- **Deep nesting**: `src/handlers/user/auth/login.rs` creates import friction. Prefer `src/user/auth.rs` with flat structure
- **Layer violations**: domain crate depends on apps crate. Breaks DAG constraint. Solution: move shared code to appropriate layer or invert dependency via trait
- **Premature splitting**: 3-crate workspace with 200 LOC per crate. Overhead outweighs benefits. Solution: merge into single crate until 500+ LOC threshold

## Module Organization

Organize modules by domain features (user, auth, payment) rather than technical layers (handlers, models, services). Prefer flat module structures over deep nesting.

### Rationale

Feature-based modules group related functionality, improving discoverability and reducing cross-module coupling. Flat structures minimize import friction and cognitive overhead. Vertical slice architecture enables independent feature development without layer coordination.

### Feature-Based Structure

Organize around domain features:

```
src/
├── user/
│   ├── mod.rs
│   ├── service.rs
│   ├── repository.rs
│   └── types.rs
├── auth/
│   ├── mod.rs
│   ├── handler.rs
│   └── middleware.rs
└── payment/
    ├── mod.rs
    ├── processor.rs
    └── validation.rs
```

Each feature module contains all related code: types, business logic, handlers, storage.

### Flat Structure Preference

Limit nesting to 2-3 levels. Prefer `src/user/auth.rs` over `src/handlers/user/auth/login.rs`:

```
# Correct (flat)
src/
├── user/
│   ├── mod.rs
│   └── auth.rs

# Incorrect (deep nesting)
src/
└── handlers/
    └── user/
        └── auth/
            └── login.rs
```

### Module Re-exports

Use `pub use` to flatten public API:

```rust
// src/user/mod.rs
mod service;
mod repository;

pub use service::{UserService, UserError};
pub use repository::UserRepository;
```

### Examples

**Correct**:
```rust
// src/user/mod.rs
mod auth;
mod profile;
mod repository;

pub use auth::{authenticate, AuthError};
pub use profile::{Profile, ProfileService};
```

**Incorrect**:
```rust
// src/models/user.rs
// src/handlers/user.rs
// src/services/user.rs
// Technical layer separation fragments feature logic
```

### Exceptions

**Shared utilities**: Infrastructure code (logging, metrics, config) may organize by technical category when used across features.

**Generated code**: Generated types may use separate module tree (`src/gen/`) to isolate from hand-written code.

**Platform-specific code**: Platform conditional compilation may require platform subdirectories (`src/platform/linux/`, `src/platform/macos/`).

## Related

[Error Handling](../../errors/reference/error_handling.md)
