# Reconnection Patterns

Cap'n Proto surfaces `Disconnected` exceptions on connection drops, enabling client-side reconnection logic. Use `auto_reconnect` wrapper and heartbeat tasks for stale connection detection.

## Disconnect Detection

- Protocol detects drops via read/write errors (TCP RST, WS close frames)
- Aborts in-flight calls, invalidates capabilities
- Rust throws `ErrorKind::Disconnected`

## Auto-Reconnect Pattern

`capnp_rpc::auto_reconnect` takes a synchronous connect closure (`FnMut() -> capnp::Result<C>`) and returns `capnp::Result<(C, Box<dyn SetTarget<C>>)>`. When a call through the client fails with `Disconnected`, the next call invokes the closure again. Make the closure return a *future client* so the asynchronous connect does not block and calls queue until it resolves:

```rust
use capnp_rpc::{auto_reconnect, new_future_client, rpc_twoparty_capnp::Side, twoparty, RpcSystem};

async fn connect() -> capnp::Result<service::Client> {
    let ws = open_websocket("wss://server").await?;          // WSReader/WSWriter halves
    let network = Box::new(twoparty::VatNetwork::new(
        ws.reader, ws.writer, Side::Client, Default::default(),
    ));
    let mut rpc_system = RpcSystem::new(network, None);
    let client: service::Client = rpc_system.bootstrap(Side::Server);
    tokio::task::spawn_local(rpc_system);                     // RpcSystem is !Send
    Ok(client)
}

let (client, set_target) = auto_reconnect(|| Ok(new_future_client(connect())))?;

// Heartbeat on the same LocalSet
tokio::task::spawn_local(heartbeat(client.clone()));
```

All of this runs inside a `tokio::task::LocalSet` (`LocalSet::new().run_until(..)`): Cap'n Proto RPC objects are `Rc`-based and `!Send`, so `tokio::spawn` does not compile with them. `set_target` swaps the underlying client explicitly, for example after a server-directed migration.

## Heartbeat Pattern

```rust
async fn heartbeat(client: service::Client) {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        if let Err(e) = client.heartbeat_request().send().promise.await {
            if matches!(e.kind, capnp::ErrorKind::Disconnected) {
                // The failed call marked the connection dead; the next call reconnects.
            }
        }
    }
}
```

## Offline-Resilient Streaming

Design idempotent resumption for long-running streams:

**Schema**:
```capnp
interface EventStreamer {
  getStream @0 (sessionId :Text, lastSeq :UInt64) -> (stream :Stream);
}
```

**Client**:
```rust
let session_id = Uuid::new_v4().to_string();
let mut last_seq = 0;

loop {
    // Pseudocode over the generated request API (`get_stream_request()`, then `send().promise`).
    match streamer.getStream(session_id, last_seq).await {
        Ok(stream) => {
            while let Some(event) = stream.nextEvent().await? {
                process(event);
                last_seq += 1;
            }
        }
        Err(e) if matches!(e.kind, ErrorKind::Disconnected) => {
            // Reconnect loop continues with last_seq
            tokio::time::sleep(backoff_duration()).await;
        }
        Err(e) => return Err(e),
    }
}
```

**Server-side**:
- Store event logs keyed by session UUID
- Resume from `lastSeq` on reconnect
- Pipeline catch-up: `resume().nextEvents(100)`

## Client-Side Offline Queue

For browsers/WASM:
- Queue RPCs in IndexedDB during offline (`navigator.onLine`)
- Replay on reconnect with exponential backoff (1s → 60s)
- Resume streams by sending `lastEventId`

## Performance

- 99%+ uptime even with 5-10min outages
- Far beyond polling reliability
- Session persistence in Redis for server restarts

## Related

[Error Handling](../rules/error_handling.md)
[WebSocket Tunneling](../rules/websocket_tunneling.md)

## Sources

- https://github.com/capnproto/capnproto-rust/blob/master/capnp-rpc/examples/hello-world/server.rs and client.rs — `async fn` server methods with `self: Rc<Self>`, `LocalSet` plus `spawn_local(rpc_system)`, `request.send().promise.await` (capnp-rpc 0.28.0; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp-rpc/examples/calculator/client.rs — promise pipelining through `send().pipeline` (accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp-rpc/src/reconnect.rs — `auto_reconnect(connect: FnMut() -> capnp::Result<C>) -> capnp::Result<(C, Box<dyn SetTarget<C>>)>` and the `new_future_client` pattern (accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnp/src/lib.rs — `capnp::Error` (`pub kind`, private `extra`/`details`), `ErrorKind` variants, `NotInSchema` (capnp 0.28.2; accessed 2026-11-22)
- https://github.com/capnproto/capnproto-rust/blob/master/capnpc/CHANGELOG.md — the `&mut self` → `&self` (0.22) → `self: Rc<Self>` (0.23) receiver changes (accessed 2026-11-22)
