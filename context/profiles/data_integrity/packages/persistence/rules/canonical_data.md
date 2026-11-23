---
paths:
  - "*.rs"
  - "*/*.rs"
  - "*/*/*.rs"
  - "*/*/*/*.rs"
  - "*/*/*/*/*.rs"
  - "*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*/*/*.rs"
---

# Canonical Data

Data that survives hashing, caching, comparison, or publication is exact by domain, canonical by contract, and stamped with its version.

- Exact quantities, identifiers, and counted values never route through binary floating point; the numeric domain is pinned at the schema boundary (integers or base-10 decimal), and trailing-zero identity semantics are decided explicitly, never inherited from a crate default.
- Canonical encoding is an encoder-side contract: shortest-form scalars, bytewise-sorted encoded map keys, duplicate-key rejection. Decoders never depend on canonical form — they accept any valid encoding.
- Every distinct hash role carries its own domain string; a hash value never crosses domains; persisted hashes are stamped with algorithm and domain.
- Schema and canonicalization versions are stamped in every persisted semantic object; a mismatch is a typed outcome, not a parse error.
- Byte-identity is proven by rebuild-and-hash under a pinned writer profile — never assumed; path remapping is passed outside `RUSTFLAGS`.
- exact numerics, the deterministic-encoding model, domain-separated hashing, version envelopes, and writer-profile proof mechanics — [Canonical Data](../reference/canonical_data.md)
- the envelope header and corruption posture that carry the version stamps — [Persistence Envelopes for Zero-Copy Archives](../reference/persistence_envelopes.md)
