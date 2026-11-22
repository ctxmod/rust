# Reconnection Patterns

Cap'n Proto surfaces `Disconnected` exceptions on connection drops, enabling client-side reconnection logic. Use `auto_reconnect` wrapper and heartbeat tasks for stale connection detection.

## Disconnect Detection

- Protocol detects drops via read/write errors (TCP RST, WS close frames)
- Aborts in-flight calls, invalidates capabilities
- Rust throws `ErrorKind::Disconnected`

## Auto-Reconnect Pattern

```rust
use capnp_rpc::auto_reconnect;

let (client, set_target) = auto_reconnect(|| async {
    let ws = tokio_tungstenite::connect_async("wss://server").await?;
    let network = VatNetwork::new(
        Box::new(WSReader::new(read_half)),
        Box::new(WSWriter::new(write_half)),
        Side::Client,
        Default::default(),
    );
    let rpc_system = RpcSystem::new(network, bootstrap);
    Ok(rpc_system.client)
});

// Spawn heartbeat task for stale detection
tokio::spawn(heartbeat(client.clone()));
```

## Heartbeat Pattern

```rust
async fn heartbeat(client: Client) {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        if let Err(e) = client.heartbeat().await {
            if matches!(e.kind, ErrorKind::Disconnected) {
                // Connection will auto-reconnect
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
