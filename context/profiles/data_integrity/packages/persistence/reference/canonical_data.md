# Canonical Data

Data that survives hashing, caching, comparison, or publication is exact by domain, canonical by contract, and stamped with its version. Anything compared by bytes, addressed by hash, or cached across runs needs all three; anything that only lives inside one process run needs none of this machinery.

## Exact Numerics

- Binary floats cannot represent `0.1`. Parsing `"19.99"`, multiplying, and rounding is the documented anti-pattern: the error enters at parse and every downstream consumer inherits it.
- The numeric domain is chosen and pinned at the schema boundary: base-10 decimal (e.g. `rust_decimal`) for quantities with fractional parts, arbitrary precision (`bigdecimal`-class) where scale is unbounded, integer minor units (cents, satoshis) where the domain allows. The schema declares which; readers reject the wrong domain as a type error, not a coercion.
- Trailing-zero identity is decided explicitly: whether `1.50` equals `1.5` is a domain decision (identity-visible until normalize, or normalize-on-construct) — it is never inherited from whatever a crate's `PartialEq` happens to do.
- Even binary fixed-point is not exactness: exactness comes from base-10 digits. A fixed-point type with a binary scale still rounds values its scale cannot represent; the schema boundary says which values the domain can carry at all.

## Canonical Encoding

Canonical encoding is an encoder-side contract; decoders accept any valid encoding and never depend on receiving canonical form.

- The current standard model is RFC 8949 §4.2.1 Core Deterministic Encoding: shortest-form head scalars MUST; map keys sorted bytewise-lexicographically on their *encoded* form (not their decoded value); preferred representation choices fixed. RFC 7049's length-first "canonical CBOR" is superseded — do not cite it as the model.
- The CDE profile adds definite-length-only construction and no-duplicate-keys. Duplicates serialize identically and sort adjacent, so a streaming encoder can enforce uniqueness by comparing each key with its encoded predecessor.
- Tag and representation choice (integer vs string enums, offset vs ISO timestamps) is part of the determinism contract: both encodings may be valid; only one is canonical, and the contract names it.
- Serialization-level determinism does not fix application-level ambiguity: a timestamp recorded as local time, an integral float, a set keyed by a non-total order — the canonical encoder faithfully reproduces whatever ambiguity the application layer put in. Canonicalize the semantics first, then the bytes.

## Domain-Separated Hashing

- Every distinct hash role carries its own domain string: `"<application> <purpose> v1"` (the BLAKE3 `derive_key` recommended format). Content hashing, fingerprinting, integrity stamps, and keyed derivation are different roles even over identical bytes.
- A hash value never crosses domains: a digest minted as a content address is never compared against, or reused as, a fingerprint of the same bytes in another role. One key or one context string across use cases is generally forbidden.
- Identical bytes in different roles never collide semantically: because the domain string prefixes the hashed material, role confusion produces a different digest by construction instead of relying on callers to keep the contexts apart.
- Every persisted hash is stamped with its algorithm identifier and its domain — the index-freshness reference's algorithm-stamp rule generalized to a full taxonomy; the envelope mechanics live in the persistence-envelopes reference.

## Version Stamping and Envelopes

- Every persisted semantic object carries schema and canonicalization versions in its envelope — the persistence-envelopes reference owns the header fields (magic, schema version, format-control stamp, producer version) and the corruption posture; this rule adds the requirement that canonicalization version rides beside schema version, because two producers can agree on the schema and disagree on the encoding.
- A version mismatch is a typed outcome — reject-then-rebuild in the cache posture, a named variant in the load taxonomy — never a parse error discovered three layers deep.
- A stored value always equals the hash of the bytes it claims to describe: content-address integrity is verified at load, not assumed at write.

## Writer Profiles and Rebuild-and-Hash

Byte-identity of build outputs is proven, never assumed from flag inspection:

- **The pinned writer profile**: same toolchain version, same flags, remapped source paths, fixed `SOURCE_DATE_EPOCH`. The profile is recorded with the artifact so a later rebuild can reproduce it.
- **The proof is rebuild-and-hash**: build twice — or on two machines — under the profile and compare hashes. A flag audit is not evidence; cargo#6914 records why remap does not belong in ad-hoc `RUSTFLAGS` — cargo hashes the whole `RUSTFLAGS` value into `-C metadata`, so flag placement perturbs fingerprinting — and rust#50556 records codegen nondeterminism that survived even remapping and `codegen-units = 1`.
- **Variance sources to hunt**: proc-macros, `build.rs`, and the `cc` crate reintroduce host variance (absolute paths, timestamps, local toolchain) — each is a suspect when a rebuild diff appears. `diffoscope` on the differing artifacts localizes the first divergent byte.
- Semantic-equivalence and byte-reproducibility are separate claims and are reported separately; the general semantic-vs-invariant discipline (in the agentic module's verification doctrine) owns the general doctrine.

## Related

- The envelope header, corruption posture, and atomic publication: [Persistence Envelopes for Zero-Copy Archives](persistence_envelopes.md)
- Schedule independence and chosen iteration order feeding the encoder: [Determinism](../../determinism/reference/determinism.md)
- Fingerprint kinds and the algorithm-identifier rule this reference generalizes: [Index Freshness and Benchmarks](../../indexing/reference/index_freshness_and_benchmarks.md)
