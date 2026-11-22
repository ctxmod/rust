# Code Generation Pipeline

Cap'n Proto generates Rust code from `.capnp` schemas during the build process using `build.rs` and the `capnpc` crate. Generated code lives in `pkg/rs/gen/` with `DO NOT EDIT` headers. The paths here follow a polyglot monorepo layout — schemas under `pkg/capnp/`, generated Rust under `pkg/rs/gen/` — and committing generated code there is a choice; the capnpc default writes into Cargo's `OUT_DIR` and includes it with `include!`. Substitute your own schema and output roots.

## Directory Structure

```
project/
├── pkg/
│   ├── capnp/           # Schema source
│   │   ├── config.capnp
│   │   └── rpc.capnp
│   └── rs/
│       └── gen/         # Generated output
│           ├── config_capnp.rs
│           └── rpc_capnp.rs
├── build.rs             # Generation script
└── Cargo.toml
```

## build.rs Configuration

```rust
fn main() {
    capnpc::CompilerCommand::new()
        .file("pkg/capnp/config.capnp")
        .file("pkg/capnp/rpc.capnp")
        .output_path("pkg/rs/gen/")
        .src_prefix("pkg/capnp/")
        .run()
        .expect("schema compilation failed");
}
```

## Cargo.toml Dependencies

```toml
[dependencies]
capnp = "0.28"
capnp-rpc = "0.28"      # For RPC
capnp-futures = "0.28"  # For async streams

[build-dependencies]
capnpc = "0.28"
```

The four crates release in lockstep: keep them on the same minor version, because generated code from one `capnpc` minor targets the runtime API of the matching `capnp` minor (0.27 and 0.28 both changed the base-crate API that generated code calls). The 0.28 line requires Rust 1.87 (`rust-version`) and is itself built on edition 2024. Generated code no longer needs `unsafe` (capnpc 0.25+) and avoids `unreachable_pub` warnings (0.25.3+). Before tightening a crate that includes it to `unsafe_code = "forbid"`, check the generated file for local `#[allow(unsafe_code)]` annotations (capnpc 0.24.1 added them): `forbid` cannot be relaxed by an inner `allow`, so use `deny` if any remain.

## Generated Output

Each `.capnp` file generates `{name}_capnp.rs` containing:
- `Reader` types for immutable access
- `Builder` types for mutation
- `Owned` types for ownership transfer
- RPC `Client`/`Server` traits (for interfaces)

## Namespace Control

Use `$Rust.parentModule("crate")` annotations to control generated module structure:

```capnp
@0xdeadbeef;
$Rust.parentModule("crate::gen");

struct Example {
    field @0 :Text;
}
```

Generates `crate::gen::example_capnp` module.

## Validation

CI checks verify generated code freshness:
- Run `cargo build` on schema changes
- Commit generated files
- CI fails if generated output differs from committed

## Related

[Schema-First Development](../rules/schema_first.md)
[Zero-Copy Serialization](zero_copy.md)

## Sources

- https://crates.io/crates/capnp, https://crates.io/crates/capnp-rpc, https://crates.io/crates/capnpc, https://crates.io/crates/capnp-futures — current 0.28.2 / 0.28.0 / 0.28.1 / 0.28.0 (2026-11-15 to 2026-11-22; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnpc/CHANGELOG.md — generated `Server` traits support `async fn` with `&self` (capnpc 0.22.0) and take `self: Rc<Self>` (0.23.0) (capnpc 0.28.1, 2026-11-16; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp/CHANGELOG.md — capnp 0.28.0 (MSRV 1.87.0, `Error::details`, `extra()` accessor), 0.28.2 (2026-11-22; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp-rpc/CHANGELOG.md — capnp-rpc 0.22.0–0.28.0: `async fn` method support, `new_client_from_rc`, `set_background_error_handler` (capnp-rpc 0.28.0, 2026-11-15; accessed 2026-11-22)
