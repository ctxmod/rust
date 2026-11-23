---
description: >-
  Index of the rust module's normative references: the core profile's
  structure, error, CLI, code-generation, and lint references, and what
  the data_integrity, verification, capnp, and capnp_rpc profiles add.
binds_to_depth: main...
---

# Rust

This module's rules load once the session reads or writes a `.rs` file or a `Cargo.toml`. Even the first Rust file keeps every binary a thin `src/main.rs` over a library crate, keeps items private unless outside code must reach them, and returns thiserror enums from libraries and anyhow errors only from applications.
The on-demand references are indexed in `context/profiles/core/packages/overview/reference/module_index.md` in the github.com/ctxmod/rust repository.
