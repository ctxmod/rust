# Error Handling

Rust error handling splits between library and application concerns: libraries use thiserror for matchable error enums, applications use anyhow for ergonomic context chains. Protocol mappings (gRPC, HTTP) at boundaries preserve type information while enabling cross-service communication.

## Flagship Posture (2026 corpus)

Across the eight flagship repositories studied (ripgrep, fd, bat, just, cargo, uv, zed, rust-analyzer): anyhow-only or anyhow+thiserror is the dominant shape; hand-rolled error enums are a legitimate at-scale choice (just's 1,100-line `Error<'src>`, rust-analyzer's typed diagnostics); `miette` and `eyre` appear in none of them; and binaries terminate through an own `ExitCode` enum with `From<ExitCode> for process::ExitCode` (fd's pattern) rather than raw `std::process::exit`. The split below is the two-crate expression of that posture.

## Core Concepts

- **thiserror for libraries**: Public error types use `#[derive(thiserror::Error)]` to generate `std::error::Error` implementations with structured enum variants. Callers match on specific variants for recovery logic. Each variant embeds relevant context as fields. The `#[non_exhaustive]` attribute enables API evolution without breaking changes
- **anyhow for applications**: Binary crates use `anyhow::Result` with `.context()` chains to attach diagnostic information. Error propagation via `?` operator works identically, but application code prioritizes readable traces over structured matching
- **Conversion at boundaries**: Libraries implement `From<DomainError>` for protocol types (`tonic::Status`, `http::StatusCode`) at service boundaries. The `?` operator triggers automatic conversion. This preserves type information within library boundaries while translating to protocol-specific representations externally
- **Category classification**: Error enums embed category field (permanent/transient/unknown) to drive retry logic in generated clients. Permanent errors (validation failures, not-found) do not retry. Transient errors (timeouts, resource exhaustion) retry with exponential backoff. Unknown errors treated as transient

## Error Taxonomy Pattern

### Library Error Enum

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UserError {
    #[error("user {user_id} not found")]
    NotFound { user_id: String },

    #[error("invalid email format: {email}")]
    InvalidEmail { email: String },

    #[error("database error: {source}")]
    Database {
        source: sqlx::Error,
        #[from]
        category: ErrorCategory,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
    Permanent,
    Transient,
    Unknown,
}

impl From<UserError> for tonic::Status {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound { .. } => tonic::Status::not_found(err.to_string()),
            UserError::InvalidEmail { .. } => tonic::Status::invalid_argument(err.to_string()),
            UserError::Database { .. } => tonic::Status::internal(err.to_string()),
        }
    }
}
```

### Application Error Chain

```rust
use anyhow::{Context, Result};

fn process_user(id: &str) -> Result<()> {
    let user = db::find_user(id)
        .context(format!("failed to load user {}", id))?;

    validate_email(&user.email)
        .context("email validation failed")?;

    Ok(())
}
```

## Protocol Mappings

### gRPC (tonic)

Map domain errors to gRPC status codes:

```rust
impl From<UserError> for tonic::Status {
    fn from(err: UserError) -> Self {
        let code = match &err {
            UserError::NotFound { .. } => tonic::Code::NotFound,
            UserError::InvalidEmail { .. } => tonic::Code::InvalidArgument,
            UserError::Database { category, .. } => match category {
                ErrorCategory::Transient => tonic::Code::Unavailable,
                _ => tonic::Code::Internal,
            },
        };
        tonic::Status::new(code, err.to_string())
    }
}
```

### HTTP (axum)

Map domain errors to HTTP status codes:

```rust
impl From<UserError> for axum::http::StatusCode {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound { .. } => axum::http::StatusCode::NOT_FOUND,
            UserError::InvalidEmail { .. } => axum::http::StatusCode::BAD_REQUEST,
            UserError::Database { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

## Enum vs Struct Pattern

Two patterns scale differently based on error volume:

- **Enum variants** (libraries, <50 error codes): Maximum matchability and recovery granularity. Consumers match specific variants in `match` expressions. Use for public library APIs where callers need fine-grained error handling
- **Single struct + kind enum** (applications, 100+ error codes): Scales better for high-volume errors with shared fields (backtrace, source, timestamp). Error handling uniform across all kinds. Use when error handling is consistent regardless of specific error type

Threshold: switch to struct pattern if error count exceeds 50 codes with field similarity.

## Stable Machine Codes

Diagnostics that scripts, dashboards, and tests key on carry stable machine codes from a maintained registry — the registry is a design artifact that exists before the first diagnostic ships, because retrofitting means renumbering, which the discipline forbids:

- **Code format** `<TOOL>-<AREA><NNN>` — a short tool prefix, a three-letter area, and three digits. One home per code: the document whose algorithm detects the condition defines the code, and every other document cites that home instead of restating the code. This extends the enum-vs-struct threshold above — a registry is the code-volume discipline applied independently of which Rust shape carries the codes.
- **Seven-column definition table** — Code | Severity | Exit class | Trigger | Message template | Correction | Source. "Correction" states the safe, actionable fix; when there is no safe fix, write `—` with a reason. The table is the registry; code comments and doc comments cite it.
- **Stability rules.** A code is stable once defined: never renumbered, never reused. Retired codes stay in their table marked "retired" — the SQLite result-code precedent, where numbering survives even after semantics retire. Severity is immutable: new behavior gets a new code, an old code never reclassifies (the gRPC canonical-code model — a small prescriptive set with fixed semantics — is the scaling precedent). Message text is free to change; the code is not.

## Source Chains and Honest Causality

- **`source()` end-to-end.** thiserror's `#[from]` implies `#[source]`: preserve the causal chain from protocol surface to root cause, and keep `.context()` layers (anyhow) on top of it rather than flattening it. The official error guidelines predate chaining guidance and leave the `# Errors` / `# Panics` doc-section contract (the C-FAILURE-style checklist) without a chain companion — the profile owns the discipline here.
- **Corrupt input is an engine error, never a domain no-result.** "The data lied" and "the row does not exist" are different failures: corrupt bytes surface through the error path with a causal chain pointing at the validation that failed; absence is a domain outcome (empty, not-found). Collapsing corruption into a no-result hides the distinction from every caller and retry policy above.
- **Unavailable vs invalid at protocol mappings.** The same distinction maps onto transport: an upstream that cannot be reached is Unavailable/503-class and retryable; a request the data cannot satisfy is InvalidArgument/400-class and is not. The ErrorCategory retry classification above consumes this distinction.
- For error enums designed to grow across versions, `#[non_exhaustive]` coverage lives in the visibility reference — [Visibility Standards](../../visibility/reference/visibility.md)

## Anti-Patterns

- **Error-to-string conversion**: `err.to_string()` loses type information. Callers cannot match on specific errors. Solution: propagate via `?` or convert to protocol type
- **Box<dyn Error> everywhere**: Erases type information at library boundaries. Callers must downcast to recover specifics. Solution: use concrete error enums for public APIs
- **Unwrap in library code**: Panics crash applications without recovery. Solution: return `Result` with specific error variant
- **Missing context in applications**: Propagating errors via `?` alone loses diagnostic information. Solution: attach context via `.context("operation description")?`
- **anyhow in library crates**: `pub fn library_func() -> anyhow::Result<()>` — callers cannot match on the error for recovery logic. Solution: use thiserror error enums for public library APIs; reserve anyhow for application crates
- **thiserror proliferation in applications**: `fn internal_helper() -> Result<(), AppError>` for every internal helper — enum overhead without benefit. Solution: use `anyhow::Result` with `.context()` chains

## Related

[Workspace Organization](../../workspace/reference/workspace_organization.md)
[Code Generation Mechanisms](../../codegen/reference/code_generation_mechanisms.md)
