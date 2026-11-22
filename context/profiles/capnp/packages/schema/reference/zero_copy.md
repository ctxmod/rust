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

**Async integration (capnpc 0.22+, current shape since 0.23)**:
- Generated `Server` trait methods are `async fn` returning `Result<(), capnp::Error>`, with `?` propagation
- The receiver is `self: Rc<Self>` (0.23+; it was `&self` in 0.22 and `&mut self` before): the RPC system is single-threaded and shares the object, so mutable state needs interior mutability (`Cell`, `RefCell`), and a method can hold `self` across `.await` points
- Register an object you also keep a handle to with `capnp_rpc::new_client_from_rc(rc)`

## Memory Safety

Rust lifetimes enforce zero-copy safety:
- Readers/Builders borrow from buffers
- Buffer cannot be dropped while in use
- No clones needed in hot paths

## Related

[Code Generation Pipeline](code_generation.md)

## Sources

- https://github.com/capnproto/capnproto-rust/blob/master/capnpc/CHANGELOG.md — generated `Server` traits support `async fn` with `&self` (capnpc 0.22.0) and take `self: Rc<Self>` (0.23.0) (capnpc 0.28.1, 2026-11-16; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp/CHANGELOG.md — capnp 0.28.0 (MSRV 1.87.0, `Error::details`, `extra()` accessor), 0.28.2 (2026-11-22; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp-rpc/CHANGELOG.md — capnp-rpc 0.22.0–0.28.0: `async fn` method support, `new_client_from_rc`, `set_background_error_handler` (capnp-rpc 0.28.0, 2026-11-15; accessed 2026-11-22)
