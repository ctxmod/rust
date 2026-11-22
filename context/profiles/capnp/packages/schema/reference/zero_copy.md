# Zero-Copy Serialization

Cap'n Proto achieves zero-copy serialization by designing the wire format to match in-memory structures, eliminating encode/decode steps entirely. Data remains in place; readers access fields through generated views.

## Core Mechanics

**Wire format alignment**:
- Data uses aligned 64-bit words
- Relative pointers link structures
- No parsing or transformation required

**Reader/Builder pattern**:
- `Reader<'a>` borrows from raw buffer (immutable view)
- `Builder<'a>` borrows from raw buffer (mutable view)
- Lifetimes prevent invalidation during use

**Lazy validation**:
- Getters perform bounds checks on access
- No upfront validation cost
- Errors surface at point of use

## Performance Characteristics

- "Infinity times faster" than Protobuf (no encode/decode)
- Outperforms Protobuf in Rust serialization throughput (MB/s)
- 10x+ throughput vs JSON (no per-event parsing)
- <1% CPU at 50KB events
- Direct memory access without copying

## Optimization Strategies

**Packed encoding**:
- Enable `packed=true` for bandwidth savings
- Minimal unpack CPU cost
- 10-30% size reduction

**Fixed-size arrays**:
- Use `<N> :Type` for constant dimensions
- SIMD-friendly for vector operations
- Example: `<1536> :Float32` for embeddings

**Async integration (v0.22+)**:
- Stabilized `async fn` in generated `Server` traits
- Idiomatic `async/await` with `?` and `&self`
- `RefCell` for interior mutability

## Memory Safety

Rust lifetimes enforce zero-copy safety:
- Readers/Builders borrow from buffers
- Buffer cannot be dropped while in use
- No clones needed in hot paths

## Related

[Code Generation Pipeline](code_generation.md)
