# rust

A ctxmod context module for Rust
development, published as `github.com/ctxmod/rust/context`. It requires
`github.com/ctxmod/engineering/context` for the language-neutral
principles it builds on.

## Profiles

| Profile | Selected | Holds |
| --- | --- | --- |
| `core` | by default | Workspace and module organization, error handling, visibility, lint enforcement, CLI structure, and code generation, with the `layered_workspace` and `lib_patterns` examples |
| `verification` | opt-in, extends `core` | Fixture-driven verification, property, metamorphic, and fuzz testing, and the rstest and cargo-nextest runner mechanics, with the `test_fixtures` example |
| `data_integrity` | opt-in, extends `core` and `verification` | For tools that persist, parse, or index data: persistence envelopes, canonical data, determinism, input hardening, source spans, path containment, and index freshness with benchmark discipline |
| `capnp` | opt-in, extends `core` | Cap'n Proto schema-first development: zero-copy, code generation, schema evolution, and schema validation |
| `capnp_rpc` | opt-in, extends `capnp` | Cap'n Proto RPC: promise pipelining, reconnection, RPC error handling, and WebSocket tunneling, with an embedding-service example |

## Selecting

Require the module from your repository's `context/module.yaml`:

```yaml
requires:
  - github.com/ctxmod/rust/context
```

`core` is the module's default profile. Select an opt-in profile by its
full name, for example by extending it from one of your own profiles:

```yaml
extends:
  - github.com/ctxmod/rust/context/profiles/data_integrity
```

## Checking

```sh
ctxmod vet --root .
```
