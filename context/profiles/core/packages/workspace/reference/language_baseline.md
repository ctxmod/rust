# Edition and Language Baseline

The edition, toolchain, and language features Rust code is written against, current through Rust 1.100 (2026-11). New crates use edition 2024; a feature is usable only when the crate's declared `rust-version` is at least the release that stabilized it.

## Release Cadence and the Floor

Rust ships a stable release every six weeks; as of 2026-11-22 the current stable is 1.100.0 (2026-11-12). Only the latest stable receives fixes, so the toolchain pin moves forward deliberately and the declared `rust-version` (the MSRV) is a separate, older floor. Edition 2024 requires Rust 1.85, which is the lowest meaningful floor for an edition-2024 crate; raise it when a needed feature below lands later.

## Edition 2024 (Rust 1.85, 2025-02-20)

Migrate with `cargo fix --edition`, then set `edition = "2024"` (shared through `[workspace.package]`). The changes that shape everyday code:

| Change | What to write |
|---|---|
| RPIT lifetime capture | Return-position `impl Trait` now captures every in-scope generic parameter, lifetimes included. Narrow it with precise capturing — `fn f<'a>(x: &'a T) -> impl Iterator<Item = u8> + use<T>` — instead of the old `Captures` trick (`use<..>` stable since 1.82; in trait definitions since 1.87) |
| `unsafe extern` blocks | `unsafe extern "C" { ... }`; items inside may be marked `safe fn` or `unsafe fn`, unmarked items are unsafe |
| Unsafe attributes | `#[unsafe(no_mangle)]`, `#[unsafe(export_name = "...")]`, `#[unsafe(link_section = "...")]` |
| `unsafe_op_in_unsafe_fn` warns | Wrap unsafe operations inside an `unsafe fn` in explicit `unsafe {}` blocks, each with a `// SAFETY:` comment |
| `if let` temporary scope; tail-expression temporary scope | Temporaries (lock guards, `RefCell` borrows) drop before the `else` branch and before block-local variables; the classic `if let Some(x) = mutex.lock().get() { } else { mutex.lock() }` deadlock is gone |
| `gen` is a reserved keyword | Rename identifiers named `gen` (`r#gen` where an external API forces it); generator blocks themselves remain unstable |
| Never-type fallback to `!` | Code that relied on `()` fallback in diverging expressions may need a type annotation |
| `static mut` references denied | Use `&raw const`/`&raw mut` (1.82) or an atomic, `Mutex`, or `LazyLock` instead of taking `&STATIC_MUT` |
| Style edition 2024 | `rustfmt.toml` `style_edition = "2024"` (see the lint-enforcement reference) |
| Resolver 3 | MSRV-aware dependency resolution (see the workspace reference) |

## Stable Features Since Edition 2024 Worth Using

| Release | Feature | Replaces |
|---|---|---|
| 1.80 | `std::sync::LazyLock`, `LazyCell` | `once_cell::sync::Lazy`, `lazy_static!` |
| 1.81 | `#[expect(lint, reason = "...")]` lint level; `core::error::Error` | `#[allow]` that silently outlives its cause (see the lint-enforcement reference) |
| 1.82 | `&raw const` / `&raw mut`; `use<..>` precise capturing | `ptr::addr_of!`; `Captures<'a>` helper traits |
| 1.85 | async closures (`async || {}`, `AsyncFn`/`AsyncFnMut`/`AsyncFnOnce` in the prelude) | closures returning `async move` blocks with `F: Fn() -> Fut, Fut: Future` bounds |
| 1.86 | trait upcasting (`&dyn Sub` → `&dyn Super`) | hand-written `as_super()` methods |
| 1.88 | let chains (edition 2024 only): `if let Some(x) = a && let Ok(y) = f(x) && y > 0 { }` | nested `if let` pyramids and `match` towers |
| 1.100 | the never type `!` is nameable everywhere (`Result<T, !>`; `Infallible` is now an alias); core `Allocator` API; `core::mem::DropGuard` | `Infallible` workarounds; ad-hoc scope-guard crates for simple cases |

Not stable as of 1.100 — do not design around them: `gen` blocks and `gen fn` (tracking issue #117078), `Error::provide` generic member access (#99301), `std::error::Report` (`error_reporter`, #90172).

## Async Baseline

`async fn` and return-position `impl Trait` in traits are stable (1.75), and async closures (1.85) cover callback-taking APIs. A trait used through `dyn` still cannot have `async fn` methods directly; box the future (`Pin<Box<dyn Future<Output = T> + Send + '_>>`) or keep the trait static. Public async trait methods whose futures must be `Send` spell that out (`fn run(&self) -> impl Future<Output = ()> + Send`), because callers cannot add the bound afterwards; the `trait-variant` crate generates a `Send` and a non-`Send` variant of one trait when both are needed.

## Sources

- https://blog.rust-lang.org/releases/ — release history; Rust 1.100.0 on 2026-11-12 (accessed 2026-11-22)
- https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/ — Rust 2024 edition stabilized, its change list, async closures (Rust 1.85.0; accessed 2026-11-22)
- https://doc.rust-lang.org/edition-guide/rust-2024/index.html — the per-change edition 2024 guide and `cargo fix --edition` (accessed 2026-11-22)
- https://blog.rust-lang.org/2024/10/17/Rust-1.82.0/ — `use<..>` precise capturing, `unsafe extern`, unsafe attributes, `&raw` (Rust 1.82.0; accessed 2026-11-22)
- https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/ — precise capturing in trait definitions (Rust 1.87.0; accessed 2026-11-22)
- https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/ — let chains, edition 2024 only (Rust 1.88.0; accessed 2026-11-22)
- https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/ — `async fn` and RPIT in traits (1.75): not dyn-compatible, the `Send`-bound recommendation, `trait-variant` (accessed 2026-11-22)
- https://blog.rust-lang.org/2024/07/25/Rust-1.80.0/ — `LazyCell` and `LazyLock` (Rust 1.80.0; accessed 2026-11-22)
- https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/ — trait upcasting (Rust 1.86.0; accessed 2026-11-22)
- https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/ — `#[expect]` and lint reasons, `core::error::Error` (Rust 1.81.0; accessed 2026-11-22)
- https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/ — never type, `Allocator`, `DropGuard` (Rust 1.100.0; accessed 2026-11-22)
- https://doc.rust-lang.org/unstable-book/language-features/gen-blocks.html — `gen` blocks unstable, tracking issue #117078 (accessed 2026-11-22)
- https://doc.rust-lang.org/std/error/trait.Error.html — `provide` nightly-only (std 1.100.0; accessed 2026-11-22)
- https://doc.rust-lang.org/std/error/struct.Report.html — `Report` nightly-only under `error_reporter` (std 1.100.0; accessed 2026-11-22)
- https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html and https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html — `unsafe_op_in_unsafe_fn` warns and `static_mut_refs` denies by default in 2024 (accessed 2026-11-22)
