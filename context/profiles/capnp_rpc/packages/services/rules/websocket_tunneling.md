---
paths:
  - "*ws*.rs"
  - "*/*ws*.rs"
  - "*/*/*ws*.rs"
  - "*/*/*/*ws*.rs"
  - "*/*/*/*/*ws*.rs"
  - "*/*/*/*/*/*ws*.rs"
  - "*/*/*/*/*/*/*ws*.rs"
  - "*websocket*.rs"
  - "*/*websocket*.rs"
  - "*/*/*websocket*.rs"
  - "*/*/*/*websocket*.rs"
  - "*/*/*/*/*websocket*.rs"
  - "*/*/*/*/*/*websocket*.rs"
  - "*/*/*/*/*/*/*websocket*.rs"
---

# WebSocket Tunneling

Browsers cannot open raw TCP connections. Cap'n Proto RPC requires WebSocket tunneling with a custom `VatNetwork` implementation wrapping `tokio-tungstenite`.

## Rationale

Cap'n Proto's RPC protocol operates over bidirectional byte streams. WebSockets provide the only browser-compatible bidirectional transport. Custom `VatNetwork` adapts WebSocket frames to Cap'n Proto's `AsyncRead + AsyncWrite` traits.

## Implementation Pattern

```rust
use capnp_rpc::{twoparty::VatNetwork, RpcSystem};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{AsyncRead, AsyncWrite};

async fn ws_rpc_client(url: &str) -> anyhow::Result<()> {
    let (ws_stream, _) = connect_async(url).await?;
    let (write_half, read_half) = ws_stream.split(); // futures StreamExt::split: (sink, stream)

    // Two-party VatNetwork over byte-stream adapters of the WebSocket halves
    let network = Box::new(VatNetwork::new(
        WSReader::new(read_half),
        WSWriter::new(write_half),
        Side::Client,
        Default::default(),
    ));

    let mut rpc_system = RpcSystem::new(network, None);
    let client: service::Client = rpc_system.bootstrap(Side::Server);

    // RpcSystem is !Send: drive it on the current LocalSet, not tokio::spawn
    tokio::task::spawn_local(rpc_system);
    // Use client for RPC calls
    Ok(())
}
```

Call `ws_rpc_client` from inside `tokio::task::LocalSet::new().run_until(..)`.

## Custom Reader/Writer

Implement `AsyncRead`/`AsyncWrite` for WebSocket adaptation:

```rust
struct WSReader<R> {
    inner: R,
    buffer: Vec<u8>,
}

impl<R: AsyncRead + Unpin> AsyncRead for WSReader<R> {
    // Read from WebSocket, extract Message::Binary payloads
}

struct WSWriter<W> {
    inner: W,
}

impl<W: AsyncWrite + Unpin> AsyncWrite for WSWriter<W> {
    // Wrap data in Message::Binary frames
}
```

## Latency Characteristics

- WebSockets eliminate HTTP handshake overhead (100-300ms → 10-50ms on edge)
- Pipelining hides additional RTTs
- 4x+ faster than gRPC over high-latency links
- Promise pipelining collapses sequential calls (200-500ms savings per chain)

## Exceptions

Backend services communicating over LAN may use raw TCP without WebSocket overhead. WebSocket tunneling is only required for browser clients.

## Related

[Reconnection Patterns](../reference/reconnection.md)
[Promise Pipelining](../reference/promise_pipelining.md)
