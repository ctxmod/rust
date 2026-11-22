# Promise Pipelining

Promise pipelining allows chaining RPC calls in a single network round trip by sending dependent requests upfront, collapsing multi-step operations. Client receives promise immediately on RPC call and can chain subsequent calls without waiting.

## Mechanics

**Traditional approach** (multiple RTTs):
```
Client -> Server: foo()
Client <- Server: result1
Client -> Server: bar(result1)
Client <- Server: result2
Total: 2 RTTs
```

**Pipelined approach** (single RTT):
```
Client -> Server: foo()
Client -> Server: bar(foo_result)
Client -> Server: baz(bar_result)
Client <- Server: final_result
Total: 1 RTT
```

## Example

```rust
// Traditional (sequential): one round trip per call
let foo = client.foo_request().send().promise.await?;
let bar = foo.get()?.get_bar()?;                 // a capability in the response
let baz = bar.baz_request().send().promise.await?;

// Pipelined (single RTT): call through the pipeline before the response arrives
let bar = client.foo_request().send().pipeline.get_bar();
let baz = bar.baz_request().send().promise.await?; // only one await at the end
```

Pipelining reaches capabilities, not data: the generated `Pipeline` exposes interface-typed fields (to call on) and struct-typed fields (to reach capabilities nested inside them); primitive, text, and data fields have no pipeline accessor and must be awaited.

## Latency Benefits

- Collapses multi-step operations
- 200-500ms savings per chain on high-latency mobile/edge
- 4x+ faster than gRPC over high-latency links
- Prevents N+1 query problems

## Schema Design Impact

Interfaces define capabilities passed by reference:

```capnp
interface Factory {
    create @0 () -> (obj :Object);
}

interface Object {
    method @0 () -> (result :Text);
}
```

Client chains:
```rust
let obj = factory.create_request().send().pipeline.get_obj();
let result = obj.method_request().send().promise.await?;  // Single RTT
```

## Capability-Based Security

Object references serve as capabilities:
- Grant call permissions
- New objects accessible only by creator
- Passing reference delegates access securely
- IDs are connection-specific (prevent forgery)

## Related

[WebSocket Tunneling](../rules/websocket_tunneling.md)

## Sources

- https://github.com/capnproto/capnproto-rust/blob/master/capnp-rpc/examples/hello-world/server.rs and client.rs — `async fn` server methods with `self: Rc<Self>`, `LocalSet` plus `spawn_local(rpc_system)`, `request.send().promise.await` (capnp-rpc 0.28.0; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp-rpc/examples/calculator/client.rs — promise pipelining through `send().pipeline` (accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp-rpc/src/reconnect.rs — `auto_reconnect(connect: FnMut() -> capnp::Result<C>) -> capnp::Result<(C, Box<dyn SetTarget<C>>)>` and the `new_future_client` pattern (accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp/src/lib.rs — `capnp::Error` (`pub kind`, private `extra`/`details`), `ErrorKind` variants, `NotInSchema` (capnp 0.28.2; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnpc/CHANGELOG.md — the `&mut self` → `&self` (0.22) → `self: Rc<Self>` (0.23) receiver changes (accessed 2026-11-22)
