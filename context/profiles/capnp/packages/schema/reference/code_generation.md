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
capnp = "0.20"
capnp-rpc = "0.20"  # For RPC
capnp-futures = "0.20"  # For async streams

[build-dependencies]
capnpc = "0.20"
```

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
