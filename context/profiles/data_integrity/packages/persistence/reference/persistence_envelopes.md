# Persistence Envelopes for Zero-Copy Archives

Zero-copy readers reinterpret bytes in place, so on-disk bytes are untrusted until validated. An envelope adds an identity/version stamp, validation, an alignment contract, and atomic publication. rkyv is the worked case; the rules apply to any format whose readers borrow structure from a buffer. Crate facts below were verified against rkyv 0.8.18, bytecheck 0.8.3, memmap2 0.9.11, tempfile 3.27.0, and std 1.99.0 (2026-10-04).

## Checked Access by Default

- rkyv 0.8 enables `std`, `alloc`, and `bytecheck` by default. `rkyv::access` validates the buffer and returns `Result`; `access_unchecked` is `unsafe` and requires that the bytes "represent a valid archived type". The rkyv docs state that bytes from a potentially-malicious source should always be validated prior to access.
- Caches, downloads, and anything another process can write are untrusted: read them only through checked access (`access`, `from_bytes`).
- `access_unchecked` is allowed only behind a single, documented `unsafe` boundary. Its `// SAFETY:` comment names what established validity (validated earlier in-process and unchanged since, or a passed signature check); performance alone is no justification, and cache or untrusted bytes never qualify.

## Unsafe Isolation

- `unsafe` exists only inside declared validated-view modules; parsing/decoding code outside those modules never writes an `unsafe` block. Every block carries a SAFETY comment stating the invariant it relies on; `clippy::undocumented_unsafe_blocks` (in the selective-pedantic warn set) is the enforcement hook.
- Raw binary bytes never become Rust references before validation. The full checklist precedes typed access, in order: alignment, width, endianness, enum discriminant validity, offsets, UTF-8, and any domain checks; checked arithmetic (`checked_`/`saturating_`/`wrapping_`) before any slice construction; `.get()` over index slicing at validation boundaries.
- Parsing and decoding modules may enable `clippy::arithmetic_side_effects` scoped to the parsing path, where overflow is an attack surface. Blanket workspace-wide enablement measures as mostly noise — the clippy baseline stays as-is; the scope is the module, not the workspace.

## Alignment

- rkyv's default `aligned` format needs aligned buffers. `rkyv::util::AlignedVec` defaults to 16-byte alignment, and the rkyv book says 16 is sufficient for most use cases. Validation checks pointer alignment; unchecked access has alignment checks only in debug builds.
- The `access` family relies on the end of the buffer being tight to the end of the archived data. Hand it the exact payload slice, never the whole file with the header or trailing padding attached.
- Pad a leading header to a multiple of the archive alignment so the payload offset stays aligned.
- A memory-mapped base is page-aligned only when the map offset is 0 or a multiple of the page size. (per the source and mmap(2), not its rustdoc); compute alignment from base plus payload offset.

## Buffer Lifetimes

- An archived view borrows its buffer. Tie its lifetime to the buffer owner and convert to owned types (`from_bytes`) before anything outlives it.
- memmap2's file-backed constructors are `unsafe` because modifying the underlying file while it is mapped, in or out of process, is undefined behavior. Truncation counts. Safe options: read the payload into an `AlignedVec` and validate the copy, or map only immutable published generations no writer reopens.

## Envelope Header

Every artifact starts with a fixed header that is checked before any archive access:

| Field | Purpose |
|---|---|
| Magic bytes | Reject files that are not this format at all |
| Format/schema version | Reject artifacts written with a different archived layout |
| Format-control stamp | Endianness, alignment mode, pointer width the producer used |
| Producer crate version | Reject archives from a non-semver-compatible rkyv |
| Payload length (+ optional checksum) | Detect truncation before slicing |

rkyv has no built-in schema evolution: its feature-comparison table lists "Schema evolution: no". An archive stays readable only if the schema, the format-control features, and a semver-compatible rkyv version are all unchanged. Format-control features (`little_endian`/`big_endian`, `aligned`/`unaligned`, `pointer_width_16/32/64`) can change the serialized format; the defaults are little-endian, aligned, and 32-bit pointer width. Libraries should not enable them. Any schema or feature change bumps the envelope version; a mismatch forces a rebuild, never an in-place migration.

## Corruption and Untrusted-Cache Posture

Caches are rebuildable derived state. Every load resolves to exactly one explicit outcome:

- **Cold or missing**: rebuild.
- **Truncated, corrupt, or failed validation**: return a typed error variant, or discard and rebuild deterministically.
- **Foreign version or mismatched format stamp**: reject before access, then rebuild.
- **Stale** (inputs changed since the artifact was produced): rebuild.

A load never panics or yields partial or garbage data; model the states as a matchable error enum.

## Atomic Publication

- Write to a temporary file in the destination directory, `File::sync_all` it, rename it over the target, then fsync the containing directory.
- `std::fs::rename` replaces an existing target and maps to POSIX `rename(2)` on Unix. POSIX guarantees the target name always refers to either the old or the new file; std's docs do not call it atomic. It fails across mount points, hence the same-directory temp file.
- Linux fsync(2) states that fsyncing a file does not make its directory entry durable; the directory needs its own fsync.
- `tempfile::NamedTempFile::persist` replaces atomically but syncs neither contents nor directory — add both; `persist_noclobber` is not guaranteed atomic on all platforms.
- For multi-file artifacts, publish a complete generation directory, then switch a pointer file by the same rename protocol. A failed publish leaves the previous generation live and intact.
- The manifest is written last: the small-pointer swap is the single activation commit point, and everything before it is staging.
- Recovery is idempotent: re-running the staging sequence from any interrupted point converges on a complete generation.
- Readers pin generations so GC cannot retire a pinned one (reader lease or grace period); wherever two locks can be held together, the ordering is fixed and documented.

## Concurrency

- Use a single writer per store, serialized by an exclusive lock.
- Readers open only immutable published generations, without the writer lock.
- Remove an old generation only when no pointer references it and no reader holds it (reader lease or grace period).

## Sketch

This is a sketch, not a drop-in implementation:

```rust
fn load(bytes: &[u8]) -> Result<&ArchivedIndex, LoadError> {
    let (hdr, payload) = Header::parse(bytes)?; // magic, versions, stamp, length
    hdr.check_compatible()?;                    // foreign version -> LoadError::Foreign
    if payload.len() != hdr.payload_len as usize {
        return Err(LoadError::Truncated);
    }
    rkyv::access::<ArchivedIndex, ValidationError>(payload)
        .map_err(LoadError::Corrupt)
}
```

## Verification

Seed each state (cold, truncated, corrupt, foreign version, stale, interrupted publish), assert the typed outcome or rebuild, and re-read logical content. Never use serialized byte goldens: archive bytes legitimately change with crate versions and format features.

## Related

- Library error enums for the load-outcome taxonomy: [Error Handling](../../../../core/packages/errors/reference/error_handling.md)
- Keeping the unsafe access boundary private: [Visibility Standards](../../../../core/packages/visibility/reference/visibility.md)
