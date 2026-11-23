---
description: >-
  Index of the rust module's normative references: the core profile's
  structure, error, CLI, code-generation, and lint references, and what
  the data_integrity, verification, capnp, and capnp_rpc profiles add.
binds_to_depth: main...
---

# Rust

Rust development conventions covering workspace organization, error
handling, linting, testing, and visibility standards. The normative
references of each profile and the module's examples are indexed in
[module_index.md](reference/module_index.md).

Select the opt-in profiles by name when the work needs them: a tool that
persists, parses, or indexes data selects `data_integrity` (which brings
`verification` with it); conformance and property work selects
`verification`.
