//! WebSocket transport for the match server.
//!
//! Browsers can't open raw TCP sockets, so the web client speaks WebSocket
//! instead. Wire format: each WebSocket **binary message** carries exactly
//! one JSON-serialized [`ClientMsg`] / [`ServerMsg`] payload — the WebSocket
//! frame layer replaces the 4-byte length prefix of the TCP transport
//! (`tcp.rs`), the JSON body is identical. Text messages are accepted too
//! (some JS clients send text for JSON) and parsed the same way.
//!
//! [`ws_seat`] mirrors [`super::tcp::tcp_seat`]: it wraps an
//! already-accepted TCP stream into a [`SeatChannel`] the lobby/match code
//! consumes without knowing the transport. Internally it differs from the
//! TCP transport in one way: tungstenite's `WebSocket` owns the whole
//! connection (reads and writes share framing state), so instead of a
//! reader thread + writer thread pair there is a single socket thread that
//! multiplexes both, using a short read timeout as its tick. The coalescing
//! [`Outbox`](super::tcp::Outbox) between actor and socket is reused
//! unchanged, so slow/wedged browser peers get the same bounded-memory,
//! latest-View-wins behavior as TCP peers.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tungstenite::protocol::{Message, WebSocket};

use crate::net::{ClientMsg, ServerMsg};

use super::tcp::Outbox;
use super::SeatChannel;

/// Read-timeout tick for the multiplexing socket thread: the upper bound on
/// added outbound latency (a queued `ServerMsg` waits at most this long for
/// the blocked `read` to time out before being written).
const TICK: Duration = Duration::from_millis(15);

/// Largest accepted incoming WebSocket message, matching the TCP
/// transport's frame cap.
const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

/// Perform the server side of the WebSocket handshake on `stream` and wrap
/// the connection into a [`SeatChannel`]. Mirrors
/// [`tcp_seat`](super::tcp::tcp_seat); the handshake itself runs on the
/// caller's thread and is bounded by a 5s read timeout so a
/// connect-and-stall peer can't wedge the accept loop.
pub fn ws_seat(stream: TcpStream) -> io::Result<SeatChannel> {
    stream.set_nodelay(true)?;
    // Bound the HTTP upgrade: without a timeout a peer that connects and
    // sends nothing blocks accept-loop progress indefinitely.
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut ws = tungstenite::accept_with_config(
        stream,
        Some(
            tungstenite::protocol::WebSocketConfig::default()
                .max_message_size(Some(MAX_MESSAGE_BYTES))
                .max_frame_size(Some(MAX_MESSAGE_BYTES)),
        ),
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    // Post-handshake: switch to the short multiplexing tick.
    ws.get_ref().set_read_timeout(Some(TICK))?;

    let (in_tx, in_rx) = mpsc::channel::<ClientMsg>();
    let (out_tx, out_rx) = mpsc::channel::<ServerMsg>();

    // Coalescer: actor mpsc → bounded outbox (same shape as the TCP
    // transport's coalescer thread).
    let outbox = Outbox::<ServerMsg>::new();
    let producer = std::sync::Arc::clone(&outbox);
    thread::spawn(move || {
        while let Ok(msg) = out_rx.recv() {
            if !producer.push(msg) {
                break;
            }
        }
        producer.close();
    });

    // Socket thread: multiplex outbox → socket and socket → in_tx.
    let consumer = outbox;
    thread::spawn(move || {
        run_socket_loop(&mut ws, &consumer, &in_tx);
        consumer.close();
        let _ = ws.close(None);
        // Best-effort: flush the close frame so the browser sees a clean
        // shutdown; ignore errors (the peer may already be gone).
        let _ = ws.flush();
    });

    Ok(SeatChannel { tx: out_tx, rx: in_rx })
}

/// Pump the connection until either side goes away: write everything queued
/// in the outbox, then block in `read` for at most [`TICK`].
fn run_socket_loop<S: Read + Write>(
    ws: &mut WebSocket<S>,
    outbox: &Outbox<ServerMsg>,
    in_tx: &mpsc::Sender<ClientMsg>,
) {
    loop {
        // Outbound: drain everything currently queued. `write` feeds
        // tungstenite's send queue; the explicit `flush` pushes it onto the
        // socket (and also flushes any auto-queued pong replies).
        let mut wrote = false;
        while let Some(msg) = outbox.try_pop() {
            let payload = match serde_json::to_vec(&msg) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if ws.write(Message::binary(payload)).is_err() {
                return;
            }
            wrote = true;
        }
        if wrote && ws.flush().is_err() {
            return;
        }
        if outbox.is_finished() {
            // Actor hung up and everything was delivered.
            return;
        }

        // Inbound: block for at most TICK. tungstenite buffers partial
        // frames internally, so a timeout mid-frame just resumes on the
        // next call. Ping/close are handled inside `read` (pong is queued
        // automatically and flushed above).
        match ws.read() {
            Ok(Message::Binary(payload)) => {
                let Ok(msg) = serde_json::from_slice::<ClientMsg>(&payload) else {
                    // Malformed JSON from a browser peer: drop the message,
                    // not the connection state — mirrors the TCP reader,
                    // which treats it as a fatal stream error; here the
                    // framing is still intact, so we just disconnect.
                    return;
                };
                if in_tx.send(msg).is_err() {
                    return;
                }
            }
            Ok(Message::Text(payload)) => {
                let Ok(msg) = serde_json::from_slice::<ClientMsg>(payload.as_bytes()) else {
                    return;
                };
                if in_tx.send(msg).is_err() {
                    return;
                }
            }
            // Control frames tungstenite surfaces after handling.
            Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => {}
            Ok(Message::Close(_)) => return,
            Err(tungstenite::Error::Io(e))
                if e.kind() == io::ErrorKind::WouldBlock
                    || e.kind() == io::ErrorKind::TimedOut =>
            {
                // Read tick elapsed with no data — loop to service writes.
            }
            Err(_) => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    /// Client side of the handshake for tests, using tungstenite's client.
    fn ws_connect(addr: std::net::SocketAddr) -> WebSocket<TcpStream> {
        let stream = TcpStream::connect(addr).expect("connect");
        let (ws, _resp) =
            tungstenite::client(format!("ws://{addr}/"), stream).expect("handshake");
        ws
    }

    #[test]
    fn round_trips_client_and_server_messages() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap();

        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let seat = ws_seat(stream).expect("ws_seat");
            // Echo protocol for the test: expect a JoinMatch, answer Chat.
            let msg = seat.rx.recv_timeout(Duration::from_secs(5)).expect("client msg");
            let ClientMsg::JoinMatch { name } = msg else { panic!("expected JoinMatch") };
            assert_eq!(name, "webby");
            seat.tx
                .send(ServerMsg::Chat { seat: 0, name: "srv".into(), text: "hello".into() })
                .expect("send");
            // Keep the channel alive until the client has read the reply.
            thread::sleep(Duration::from_millis(300));
        });

        let mut client = ws_connect(addr);
        let payload =
            serde_json::to_vec(&ClientMsg::JoinMatch { name: "webby".into() }).unwrap();
        client.send(Message::binary(payload)).expect("send");
        let reply = loop {
            match client.read().expect("read") {
                Message::Binary(b) => break serde_json::from_slice::<ServerMsg>(&b).unwrap(),
                _ => continue,
            }
        };
        let ServerMsg::Chat { text, .. } = reply else { panic!("expected Chat") };
        assert_eq!(text, "hello");
        server.join().unwrap();
    }

    #[test]
    fn text_messages_parse_like_binary() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let seat = ws_seat(stream).expect("ws_seat");
            seat.rx.recv_timeout(Duration::from_secs(5)).expect("client msg")
        });
        let mut client = ws_connect(addr);
        let json = serde_json::to_string(&ClientMsg::ListLobbies).unwrap();
        client.send(Message::text(json)).expect("send");
        let got = server.join().unwrap();
        assert!(matches!(got, ClientMsg::ListLobbies));
    }

    #[test]
    fn dropping_the_seat_closes_the_socket() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let seat = ws_seat(stream).expect("ws_seat");
            drop(seat); // actor goes away immediately
        });
        let mut client = ws_connect(addr);
        server.join().unwrap();
        // The client should observe a close (or an error), not hang.
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            assert!(std::time::Instant::now() < deadline, "no close observed");
            match client.read() {
                Ok(Message::Close(_)) | Err(_) => break,
                Ok(_) => continue,
            }
        }
    }

    #[test]
    fn garbage_handshake_is_rejected() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap();
        let client = thread::spawn(move || {
            let mut s = TcpStream::connect(addr).expect("connect");
            s.write_all(b"not an http upgrade\r\n\r\n").expect("write");
        });
        let (stream, _) = listener.accept().expect("accept");
        assert!(ws_seat(stream).is_err());
        client.join().unwrap();
    }
}
