# Schema-First Development

Cap'n Proto schema files (`.capnp`) are the authoritative source for all shared types, constants, and RPC interface definitions. Rust code consuming these concepts must use generated types from `capnpc`, not hand-written equivalents.

## Rationale

When schemas exist in both `.capnp` and Rust, the two representations drift independently. Generated code eliminates this class of bug by construction: the `.capnp` schema is the single source, and the Rust representation is a deterministic derivation. Changes propagate automatically through the generation pipeline.

[Zero-Copy Serialization](../reference/zero_copy.md)

## Examples

**Correct**:
```capnp
// Define in .capnp, generate Rust
@0xdeadbeef;

struct Config {
    environment @0 :Text;
    logLevel @1 :Text;
}
```

Generated via `build.rs` (paths follow a polyglot monorepo layout; substitute your own schema and output roots):
```rust
capnpc::CompilerCommand::new()
    .file("pkg/capnp/config.capnp")
    .output_path("pkg/rs/gen/")
    .src_prefix("pkg/capnp/")
    .run()
    .expect("schema compilation failed");
```

**Incorrect**:
```rust
// Hand-written Rust type duplicating a .capnp schema
struct Config {
    environment: String,
    log_level: String,
}
```

## Exceptions

**Bootstrapping migration**: When migrating existing Rust types to Cap'n Proto, manually create `.capnp` schemas based on the existing types, then generate and compare. Unaudited reverse engineering does not qualify as a valid schema source.
