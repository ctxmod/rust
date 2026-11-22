# CLI Structure

Thin-main clap-derive CLIs with typed exit codes at the binary edge. Application-suite Rust keeps logic out of `main.rs`: the binary entry parses arguments, constructs the runtime, hands off to a library call, and renders the result. A CLI and a companion daemon are both thin entries over library bodies in this shape. The thin-main convention is also load-bearing mechanically wherever build tooling discovers tools by path — a scanner that globs `tools/*/src/main.rs` makes the entry point's location and shape contract, not style.

## Argument Parsing

clap's derive API is the default — `#[derive(Parser)]` on an args struct with per-field `#[arg]` attributes. Reach for the builder API only when compile-time-sensitive or genuinely exotic cases demand it (dynamic subcommand sets, generated surfaces). Hand-rolled parsing (ripgrep's lexopt) is a legitimate extreme-scale choice, not a starting point. Usage errors exit `2` by clap's own convention — do not remap them.

## Exit-Code Strata

| Code | Meaning | Mechanism |
|---|---|---|
| `0` | Success | `main` returns normally, or `Ok(())` / `ExitCode::SUCCESS` |
| `1` | General failure | Return `Err` from `main` and let `Termination` render it (`ExitCode::FAILURE`), or an explicit own-code failure |
| `2` | Usage error | clap's default for argument/parse failures |
| `101` | Panic | The runtime's default panic exit |

Prefer `std::process::ExitCode` and the `Termination` trait over `std::process::exit`: `exit` terminates immediately and skips destructors, so drop-based cleanup (temp dirs, lock files, flush guards) silently does not run. At scale, keep call sites typed with an own `ExitCode` enum converted at the edge:

```rust
pub enum ExitCode { Ok, General, Usage }

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        match code {
            ExitCode::Ok => std::process::ExitCode::SUCCESS,
            ExitCode::General | ExitCode::Usage => std::process::ExitCode::FAILURE,
        }
    }
}

fn main() -> std::process::ExitCode {
    run().into()
}
```

(fd's pattern; sysexits-style codes via the `exitcode` crate are endorsed but optional.) The strata align with the common tooling contract for CLIs consumed by scripts and agents: `0` success, `1` general failure, `2` usage.

## Errors at the Binary Edge

The error-stack split terminates here: typed error enums below the boundary, `anyhow` at the edge. The binary converts library errors with `.context(...)` chains and maps the outcome to an exit code; it never unwraps into a panic where a diagnostic and a nonzero exit would do. `miette` and `eyre` appear in none of the eight flagship repos studied — rich diagnostic frameworks are contraindicated; structured output (JSON/NDJSON on stdout) is the machine-readable channel, diagnostics go to stderr.

## Library Half and Binary Half

When a CLI's library is consumed elsewhere — for example a daemon that path-depends on the CLI crate for shared schema bindings — the lib/bin split is real, not ceremonial: the library half is consumed by the other crate, the binary half by operators. Commit generated bindings and pin their generator dependencies exactly (`=x.y.z`) where regeneration must never require the network.

A daemon spawned lazily by its clients, with a bounded lifetime, is a different operational shape from a supervised always-on service; document which one a binary is. Keep its wire surfaces simple (flat `key=value` lines are enough for most local control protocols), and treat an async runtime as structural only where the design needs it (fan-out to blocked waiters, an idle-versus-activity select), not as incidental plumbing.

## Cross-Compiling to macOS from Linux

A workspace can cross-compile darwin targets from Linux through zig-cc wrapper linkers committed under `.cargo/` (for example `.cargo/wrappers/zigcc-<arch>-macos`, wired per target in `.cargo/config.toml`). Because zig ships only `libSystem` for darwin, commit the minimal macOS link stubs beside the wrappers — tapi-tbd stubs for libraries such as `libiconv`, framework stubs such as `CoreServices.framework`, and any extended `CoreFoundation` symbol list — and pass them through `-L`/`-F` in the wrappers. A file-watching backend built on FSEvents, for instance, pulls `-liconv` plus the CoreServices/CoreFoundation frameworks. When a new framework edge appears, extend the committed stubs rather than weakening the wrapper; the pristine cross-build must keep producing a Mach-O binary with real dylib load commands.

[Workspace Organization](../../workspace/reference/workspace_organization.md)

[Error Handling](../../errors/reference/error_handling.md)
