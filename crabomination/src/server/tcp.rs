//! TCP transport for the match server.
//!
//! Each TCP connection is wrapped into a [`SeatChannel`] (server side) or a
//! [`ClientChannel`] (client side) by spawning a reader thread (socket → mpsc)
//! and a writer thread (mpsc → socket). The match actor then talks to the
//! socket-backed channels exactly as it does to in-process ones.
//!
//! Wire format: 4-byte big-endian length prefix followed by JSON-serialized
//! [`ClientMsg`] / [`ServerMsg`] payload.

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use socket2::{SockRef, TcpKeepalive};

use crate::net::{ClientMsg, ServerMsg};

use super::{ClientChannel, SeatChannel};

const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

/// How many outbound messages may queue for one connection before the
/// coalescer starts shedding the oldest *streaming* message (a `View` or
/// `Events`). Control messages (seat assignment, match start/over, action
/// errors) are never shed, and consecutive `View`s collapse to the latest
/// before counting against this cap — so in practice it only bites when a
/// wedged socket lets a burst of distinct `Events` pile up. Bounds per-seat
/// memory to O(cap) regardless of how long a socket write blocks, instead of
/// the old unbounded `mpsc` backlog that a stalled client could grow without
/// limit.
const MAX_PENDING_OUT: usize = 256;

/// TCP keepalive parameters. With these defaults the OS detects a peer that
/// stops responding (process killed, machine crash, NAT eviction, cable
/// pulled) within roughly two minutes of the last successful read/write,
/// rather than letting a match thread block forever on a dead socket.
///
/// - Idle: probes begin after 60s of no traffic in either direction.
/// - Interval: subsequent probes are 15s apart.
/// - Retries: after 4 unanswered probes the connection is reported dead.
///
/// Total worst-case detection time after the peer disappears:
/// `idle + interval * retries` ≈ 60s + 60s = ~2 min.
const KEEPALIVE_IDLE: Duration = Duration::from_secs(60);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);
const KEEPALIVE_RETRIES: u32 = 4;

/// Enable OS-level TCP keepalive with aggressive defaults so dead peers are
/// detected within ~2 minutes instead of indefinitely. Best-effort: individual
/// `with_*` fields are silently unsupported on some platforms, so we apply
/// them through `socket2` which handles that portably.
fn enable_keepalive(stream: &TcpStream) -> io::Result<()> {
    let ka = TcpKeepalive::new()
        .with_time(KEEPALIVE_IDLE)
        .with_interval(KEEPALIVE_INTERVAL)
        .with_retries(KEEPALIVE_RETRIES);
    SockRef::from(stream).set_tcp_keepalive(&ka)
}

/// Read one length-prefixed JSON frame and decode it as `T`.
pub fn read_frame<T: for<'de> Deserialize<'de>>(stream: &mut TcpStream) -> io::Result<T> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
    }
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload)?;
    serde_json::from_slice(&payload).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Encode `value` and write it as one length-prefixed JSON frame.
pub fn write_frame<T: Serialize>(stream: &mut TcpStream, value: &T) -> io::Result<()> {
    let payload = serde_json::to_vec(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let len = u32::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "frame too large"))?;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}

/// Outbound-message policy for the coalescing writer outbox. Lets the
/// generic [`Outbox`] treat server→client `View`s specially (only the latest
/// snapshot matters) without the queue knowing the concrete message type.
trait Outboxable {
    /// True if `self`, arriving immediately behind an already-queued `prev`
    /// of the same kind, may *replace* it — i.e. `prev` is now stale and
    /// need never reach the wire. Used to collapse a run of consecutive
    /// `View`s to the newest one.
    fn supersedes(&self, prev: &Self) -> bool;
    /// True if this message may be dropped to stay under [`MAX_PENDING_OUT`]
    /// — stream data (`View`/`Events`) whose loss the next `View` repairs,
    /// as opposed to control messages that must be delivered.
    fn is_sheddable(&self) -> bool;
}

impl Outboxable for ServerMsg {
    fn supersedes(&self, prev: &Self) -> bool {
        // A View fully describes authoritative state, so a newer View makes
        // an older still-unsent one redundant.
        matches!((self, prev), (ServerMsg::View(_), ServerMsg::View(_)))
    }
    fn is_sheddable(&self) -> bool {
        // Stream data the next update repairs: a standalone View/Events, or a
        // combined per-action Update. A combined Update is not coalesced
        // (it carries animation events), but it is sheddable under the cap
        // since a later Update/View re-establishes authoritative state.
        matches!(
            self,
            ServerMsg::View(_) | ServerMsg::Events(_) | ServerMsg::Update { .. }
        )
    }
}

impl Outboxable for ClientMsg {
    // Client→server traffic is low-volume user input: never coalesce or drop
    // it. The outbox still bounds it, but nothing is sheddable so it simply
    // grows if a (misbehaving) peer floods — acceptable, since the real
    // flood risk is the server's per-action View stream in the other
    // direction.
    fn supersedes(&self, _prev: &Self) -> bool {
        false
    }
    fn is_sheddable(&self) -> bool {
        false
    }
}

/// A bounded, coalescing single-consumer mailbox sitting between the match
/// actor (which enqueues via an `mpsc::Sender` it never blocks on) and the
/// socket writer thread (which may block arbitrarily long on a slow peer).
///
/// A dedicated coalescer thread drains the actor's `mpsc` into this outbox
/// the instant messages arrive, so the actor-facing channel stays near-empty
/// even while a write is stuck; the outbox itself caps memory by collapsing
/// consecutive `View`s and shedding the oldest streaming message once over
/// [`MAX_PENDING_OUT`].
struct Outbox<T> {
    inner: Mutex<OutboxInner<T>>,
    cv: Condvar,
}

struct OutboxInner<T> {
    queue: VecDeque<T>,
    /// Set once the producer (coalescer) or consumer (writer) goes away, so
    /// the other side unblocks and winds down.
    closed: bool,
}

impl<T: Outboxable> Outbox<T> {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(OutboxInner {
                queue: VecDeque::new(),
                closed: false,
            }),
            cv: Condvar::new(),
        })
    }

    /// Enqueue `msg`, coalescing it onto the tail if it supersedes the last
    /// queued message, then shedding the oldest sheddable message while over
    /// the cap. Returns `false` if the consumer has closed the outbox (the
    /// writer thread exited), signalling the producer to stop.
    fn push(&self, msg: T) -> bool {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if g.closed {
            return false;
        }
        match g.queue.back() {
            Some(prev) if msg.supersedes(prev) => {
                *g.queue.back_mut().unwrap() = msg;
            }
            _ => g.queue.push_back(msg),
        }
        while g.queue.len() > MAX_PENDING_OUT {
            // Drop the oldest sheddable (stream) message; never a control
            // message. If none is sheddable, stop and let the queue exceed
            // the cap rather than dropping something that must be delivered.
            match g.queue.iter().position(|m| m.is_sheddable()) {
                Some(idx) => {
                    g.queue.remove(idx);
                }
                None => break,
            }
        }
        self.cv.notify_one();
        true
    }

    /// Block until a message is available, returning `None` once the outbox
    /// is closed *and* drained.
    fn pop(&self) -> Option<T> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        loop {
            if let Some(m) = g.queue.pop_front() {
                return Some(m);
            }
            if g.closed {
                return None;
            }
            g = self.cv.wait(g).unwrap_or_else(|p| p.into_inner());
        }
    }

    fn close(&self) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.closed = true;
        self.cv.notify_all();
    }
}

/// Spawn the coalescer + writer thread pair that drains `out_rx` (the
/// actor-facing channel) through a bounded [`Outbox`] and onto `write_stream`
/// as length-prefixed frames. Used by both [`tcp_seat`] and [`tcp_client`].
///
/// Two threads, not one, so that a write stuck on a slow/wedged peer can't
/// let `out_rx` grow without bound: the coalescer keeps draining + coalescing
/// into the capped outbox while the writer is parked in `write_frame`.
fn spawn_coalescing_writer<T>(write_stream: TcpStream, out_rx: mpsc::Receiver<T>)
where
    T: Outboxable + Serialize + Send + 'static,
{
    let outbox = Outbox::<T>::new();

    // Coalescer: actor mpsc → bounded outbox. Stops when the actor drops its
    // sender (recv errors) or the writer has closed the outbox.
    let producer = Arc::clone(&outbox);
    thread::spawn(move || {
        while let Ok(msg) = out_rx.recv() {
            if !producer.push(msg) {
                break;
            }
        }
        producer.close();
    });

    // Writer: bounded outbox → socket. Stops on any socket error (incl. a
    // keepalive-detected dead peer) or once the outbox is closed and drained.
    let consumer = outbox;
    thread::spawn(move || {
        let mut s = write_stream;
        while let Some(msg) = consumer.pop() {
            if write_frame(&mut s, &msg).is_err() {
                break;
            }
        }
        consumer.close();
    });
}

/// Wrap a [`TcpStream`] into a server-side [`SeatChannel`]. Spawns one reader
/// thread (socket → mpsc<ClientMsg>) plus a coalescing writer pair
/// (mpsc<ServerMsg> → bounded outbox → socket). When either side
/// disconnects, the threads exit.
pub fn tcp_seat(stream: TcpStream) -> io::Result<SeatChannel> {
    stream.set_nodelay(true)?;
    enable_keepalive(&stream)?;
    let read_stream = stream.try_clone()?;
    let write_stream = stream;

    let (in_tx, in_rx) = mpsc::channel::<ClientMsg>();
    let (out_tx, out_rx) = mpsc::channel::<ServerMsg>();

    // Reader: socket → ClientMsg → in_tx
    thread::spawn(move || {
        let mut s = read_stream;
        while let Ok(msg) = read_frame::<ClientMsg>(&mut s) {
            if in_tx.send(msg).is_err() {
                break;
            }
        }
    });

    // Writer: out_rx → ServerMsg → bounded coalescing outbox → socket.
    spawn_coalescing_writer(write_stream, out_rx);

    Ok(SeatChannel {
        tx: out_tx,
        rx: in_rx,
    })
}

/// Wrap a [`TcpStream`] into a client-side [`ClientChannel`]. Symmetric of
/// [`tcp_seat`]: reader thread converts incoming `ServerMsg` frames into the
/// inbox, writer thread serializes outbound `ClientMsg`s.
pub fn tcp_client(stream: TcpStream) -> io::Result<ClientChannel> {
    stream.set_nodelay(true)?;
    enable_keepalive(&stream)?;
    let read_stream = stream.try_clone()?;
    let write_stream = stream;

    let (in_tx, in_rx) = mpsc::channel::<ServerMsg>();
    let (out_tx, out_rx) = mpsc::channel::<ClientMsg>();

    // Reader: socket → ServerMsg → in_tx
    thread::spawn(move || {
        let mut s = read_stream;
        while let Ok(msg) = read_frame::<ServerMsg>(&mut s) {
            if in_tx.send(msg).is_err() {
                break;
            }
        }
    });

    // Writer: out_rx → ClientMsg → bounded coalescing outbox → socket.
    spawn_coalescing_writer(write_stream, out_rx);

    Ok(ClientChannel {
        tx: out_tx,
        rx: in_rx,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::net::{TcpListener, TcpStream};
    use std::time::{Duration, Instant};

    use crate::card::CardId;
    use crate::game::GameAction;
    use crate::net::{ClientMsg, ServerMsg};

    use super::*;

    /// Write a single frame to an in-memory `Cursor<Vec<u8>>`-style stream and
    /// read it back, verifying the wire format roundtrips.
    #[test]
    fn frame_roundtrip_in_memory() {
        // Use a paired TCP listener+stream so we exercise the real wire format
        // without needing the threaded reader/writer wrappers.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let mut writer = TcpStream::connect(addr).expect("connect");
        let (mut reader, _) = listener.accept().expect("accept");

        let msg = ClientMsg::SubmitAction(GameAction::PlayLand(CardId(42)));
        write_frame(&mut writer, &msg).expect("write");

        let got: ClientMsg = read_frame(&mut reader).expect("read");
        assert!(matches!(
            got,
            ClientMsg::SubmitAction(GameAction::PlayLand(CardId(42)))
        ));
    }

    /// `tcp_seat` (server side) and `tcp_client` (client side) must form a
    /// bidirectional pipe: a `ClientMsg` sent from the client appears in the
    /// server's `SeatChannel` inbox, and a `ServerMsg` sent from the server
    /// appears in the client's `ClientChannel` inbox.
    #[test]
    fn tcp_seat_and_client_pair() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");

        // Client connects in a thread so we can accept on this thread.
        let client_thread = std::thread::spawn(move || {
            let stream = TcpStream::connect(addr).expect("client connect");
            tcp_client(stream).expect("wrap client")
        });

        let (server_stream, _) = listener.accept().expect("accept");
        let server_seat = tcp_seat(server_stream).expect("wrap seat");
        let client_chan = client_thread.join().expect("client thread");

        // Client → server.
        client_chan
            .tx
            .send(ClientMsg::JoinMatch { name: "alice".into() })
            .expect("send");

        let received = recv_within(&server_seat.rx, Duration::from_secs(2))
            .expect("server should receive ClientMsg");
        match received {
            ClientMsg::JoinMatch { name } => assert_eq!(name, "alice"),
            other => panic!("unexpected variant: {other:?}"),
        }

        // Server → client.
        server_seat
            .tx
            .send(ServerMsg::YourSeat(1))
            .expect("send");

        let received = recv_within(&client_chan.rx, Duration::from_secs(2))
            .expect("client should receive ServerMsg");
        match received {
            ServerMsg::YourSeat(s) => assert_eq!(s, 1),
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    /// `read_frame` must reject frames whose declared length exceeds
    /// `MAX_FRAME_BYTES` rather than allocating gigabytes.
    #[test]
    fn read_frame_rejects_oversized_length() {
        // We can't easily exercise read_frame against an in-memory buffer
        // since it takes &mut TcpStream. Instead, send a 4-byte length
        // header that is larger than the limit and confirm read_frame errors.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let mut writer = TcpStream::connect(addr).expect("connect");
        let (mut reader, _) = listener.accept().expect("accept");

        // Write a length prefix that's well over the cap.
        let oversized: u32 = (super::MAX_FRAME_BYTES as u32).saturating_add(1);
        std::io::Write::write_all(&mut writer, &oversized.to_be_bytes()).expect("write");

        let result: std::io::Result<ClientMsg> = read_frame(&mut reader);
        assert!(result.is_err(), "expected error for oversized frame");

        // Suppress "in-memory cursor unused" warning so this builds even if
        // we add a Cursor-based helper later.
        let _ = Cursor::new(Vec::<u8>::new());
    }

    /// Block on the receiver until a message arrives or the timeout elapses.
    fn recv_within<T>(rx: &std::sync::mpsc::Receiver<T>, timeout: Duration) -> Option<T> {
        let deadline = Instant::now() + timeout;
        loop {
            match rx.try_recv() {
                Ok(v) => return Some(v),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if Instant::now() >= deadline {
                        return None;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return None,
            }
        }
    }

    /// `read_frame` uses `read_exact`, so a peer that writes the length
    /// header and payload across multiple TCP segments (with delays in
    /// between) must still produce one complete decoded frame. Simulates a
    /// slow link by chunking each write and inserting small sleeps.
    #[test]
    fn read_frame_handles_fragmented_writes() {
        use std::io::Write;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");

        let writer_thread = thread::spawn(move || {
            let mut writer = TcpStream::connect(addr).expect("connect");
            let msg = ClientMsg::SubmitAction(GameAction::PlayLand(CardId(7)));
            let payload = serde_json::to_vec(&msg).unwrap();
            let len_bytes = (payload.len() as u32).to_be_bytes();
            // Header in two halves with a sleep between.
            writer.write_all(&len_bytes[..2]).unwrap();
            thread::sleep(Duration::from_millis(20));
            writer.write_all(&len_bytes[2..]).unwrap();
            thread::sleep(Duration::from_millis(20));
            // Payload in two halves.
            let mid = payload.len() / 2;
            writer.write_all(&payload[..mid]).unwrap();
            thread::sleep(Duration::from_millis(20));
            writer.write_all(&payload[mid..]).unwrap();
            writer.flush().unwrap();
        });

        let (mut reader, _) = listener.accept().expect("accept");
        let got: ClientMsg = read_frame(&mut reader).expect("read");
        assert!(matches!(
            got,
            ClientMsg::SubmitAction(GameAction::PlayLand(CardId(7)))
        ));
        writer_thread.join().unwrap();
    }

    /// When the network peer closes the connection at the OS level, the
    /// server's reader thread sees EOF on `read_exact`, returns Err, drops
    /// its `in_tx`, and `seat.rx` becomes `Disconnected`. The server can
    /// detect peer-loss purely by polling its mpsc receiver — no separate
    /// keepalive needed.
    ///
    /// Note: dropping the `ClientChannel` wrapper alone is *not* sufficient
    /// to close the socket — the wrapper's reader thread still owns one
    /// clone of the stream and stays blocked in `read_exact`. We force a
    /// true close via `shutdown(Both)` on the raw socket.
    #[test]
    fn tcp_seat_rx_disconnects_on_peer_shutdown() {
        use std::net::Shutdown;
        use std::sync::mpsc::TryRecvError;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");

        let peer_thread = thread::spawn(move || {
            let stream = TcpStream::connect(addr).expect("connect");
            stream.shutdown(Shutdown::Both).expect("shutdown");
            // Hold the stream briefly so the server has time to detect EOF
            // before the OS could fold any subsequent connect attempts onto
            // the same port.
            thread::sleep(Duration::from_millis(100));
        });

        let (server_stream, _) = listener.accept().expect("accept");
        let server_seat = tcp_seat(server_stream).expect("wrap seat");

        let deadline = Instant::now() + Duration::from_secs(2);
        let disconnected = loop {
            match server_seat.rx.try_recv() {
                Ok(_) => panic!("server seat rx received unexpected message"),
                Err(TryRecvError::Disconnected) => break true,
                Err(TryRecvError::Empty) => {
                    if Instant::now() >= deadline {
                        break false;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        };
        assert!(
            disconnected,
            "server seat rx should disconnect after peer shutdown"
        );
        peer_thread.join().unwrap();
    }

    /// `tcp_seat` must enable OS-level TCP keepalive on the socket. Without
    /// it, a peer that disappears silently (process killed, NAT eviction)
    /// would leave the reader thread blocked in `read_exact` forever — the
    /// match thread would never observe the disconnect.
    #[test]
    fn tcp_seat_enables_keepalive() {
        use socket2::SockRef;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");

        let client_thread = std::thread::spawn(move || {
            TcpStream::connect(addr).expect("connect")
        });
        let (server_stream, _) = listener.accept().expect("accept");
        let _client = client_thread.join().expect("client");

        // Clone first so the wrapper can take ownership of the original.
        let probe = server_stream.try_clone().expect("clone");
        let _seat = tcp_seat(server_stream).expect("wrap seat");

        let sock = SockRef::from(&probe);
        assert!(
            sock.keepalive().expect("read keepalive"),
            "SO_KEEPALIVE should be enabled after tcp_seat"
        );
    }

    /// The reader thread inside `tcp_client` must keep up with a tight burst
    /// of `ServerMsg`s — N independent messages submitted back-to-back from
    /// the server thread arrive in order on the client receiver, none lost.
    /// Exercises the (server-writer thread → socket → client-reader thread →
    /// client mpsc → consumer) pipeline under sustained load.
    #[test]
    fn tcp_client_receives_burst_in_order() {
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");

        let client_thread = thread::spawn(move || {
            let stream = TcpStream::connect(addr).expect("connect");
            tcp_client(stream).expect("wrap client")
        });

        let (server_stream, _) = listener.accept().expect("accept");
        let server_seat = tcp_seat(server_stream).expect("wrap seat");
        let client_chan = client_thread.join().expect("client thread");

        const N: u32 = 50;
        let send_thread = thread::spawn(move || {
            for i in 0..N {
                server_seat.tx.send(ServerMsg::YourSeat(i as usize)).unwrap();
            }
            server_seat
        });

        let mut received = Vec::with_capacity(N as usize);
        let deadline = Instant::now() + Duration::from_secs(5);
        while (received.len() as u32) < N {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            match client_chan.rx.recv_timeout(deadline - now) {
                Ok(ServerMsg::YourSeat(s)) => received.push(s),
                Ok(other) => panic!("unexpected message: {other:?}"),
                Err(_) => break,
            }
        }
        let _server_seat = send_thread.join().unwrap();

        assert_eq!(received.len(), N as usize, "lost messages: {received:?}");
        for (i, seat) in received.iter().enumerate() {
            assert_eq!(*seat, i, "out-of-order delivery at idx {i}: {received:?}");
        }
    }

    // ── Coalescing outbox ────────────────────────────────────────────────

    /// Synthetic message type for exercising the generic `Outbox` mechanics
    /// without building heavyweight `ServerMsg::View` payloads.
    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        /// Stream snapshot: supersedes a trailing snapshot, sheddable.
        Snapshot(u32),
        /// Stream delta: never supersedes, but sheddable under the cap.
        Delta(u32),
        /// Control: never superseded, never shed.
        Control(u32),
    }

    impl Outboxable for TestMsg {
        fn supersedes(&self, prev: &Self) -> bool {
            matches!((self, prev), (TestMsg::Snapshot(_), TestMsg::Snapshot(_)))
        }
        fn is_sheddable(&self) -> bool {
            matches!(self, TestMsg::Snapshot(_) | TestMsg::Delta(_))
        }
    }

    fn drain(outbox: &Outbox<TestMsg>) -> Vec<TestMsg> {
        let mut g = outbox.inner.lock().unwrap();
        g.queue.drain(..).collect()
    }

    /// Consecutive snapshots collapse to the latest; an interposed delta
    /// breaks the run so the earlier snapshot survives ahead of it.
    #[test]
    fn outbox_coalesces_trailing_snapshots() {
        let ob = Outbox::<TestMsg>::new();
        ob.push(TestMsg::Snapshot(1));
        ob.push(TestMsg::Snapshot(2));
        ob.push(TestMsg::Snapshot(3));
        assert_eq!(drain(&ob), vec![TestMsg::Snapshot(3)], "only latest survives");

        ob.push(TestMsg::Snapshot(1));
        ob.push(TestMsg::Delta(9)); // breaks the snapshot run
        ob.push(TestMsg::Snapshot(2));
        assert_eq!(
            drain(&ob),
            vec![TestMsg::Snapshot(1), TestMsg::Delta(9), TestMsg::Snapshot(2)],
            "a delta between snapshots prevents coalescing across it",
        );
    }

    /// Over the cap, the oldest *sheddable* message is dropped while control
    /// messages and the newest snapshot are retained.
    #[test]
    fn outbox_sheds_oldest_stream_message_over_cap() {
        let ob = Outbox::<TestMsg>::new();
        // A control message at the very front must never be shed.
        ob.push(TestMsg::Control(0));
        // Fill with distinct deltas (deltas don't coalesce) past the cap.
        for i in 0..(MAX_PENDING_OUT as u32 + 10) {
            ob.push(TestMsg::Delta(i));
        }
        let drained = drain(&ob);
        assert!(drained.len() <= MAX_PENDING_OUT, "queue stays within cap");
        assert_eq!(drained.first(), Some(&TestMsg::Control(0)), "control retained at front");
        // The most recent deltas survive; the oldest were shed.
        assert_eq!(
            drained.last(),
            Some(&TestMsg::Delta(MAX_PENDING_OUT as u32 + 9)),
            "newest stream message retained",
        );
    }

    /// Control messages are never shed even when they alone exceed the cap —
    /// the outbox grows rather than drop something that must be delivered.
    #[test]
    fn outbox_never_sheds_control_messages() {
        let ob = Outbox::<TestMsg>::new();
        let n = MAX_PENDING_OUT as u32 + 50;
        for i in 0..n {
            ob.push(TestMsg::Control(i));
        }
        let drained = drain(&ob);
        assert_eq!(drained.len(), n as usize, "no control message dropped");
    }

    /// `pop` blocks until pushed, then returns `None` once closed and drained.
    #[test]
    fn outbox_pop_drains_then_reports_close() {
        let ob = Outbox::<TestMsg>::new();
        ob.push(TestMsg::Delta(1));
        ob.push(TestMsg::Control(2));
        assert_eq!(ob.pop(), Some(TestMsg::Delta(1)));
        assert_eq!(ob.pop(), Some(TestMsg::Control(2)));
        ob.close();
        assert_eq!(ob.pop(), None, "closed + drained → None");
        assert!(!ob.push(TestMsg::Delta(3)), "push after close is refused");
    }

    /// `ServerMsg`'s policy: only `View` supersedes `View`; `View`/`Events`
    /// are sheddable while control messages are not.
    #[test]
    fn servermsg_outbox_policy() {
        assert!(ServerMsg::Events(vec![]).is_sheddable());
        assert!(!ServerMsg::YourSeat(0).is_sheddable());
        assert!(!ServerMsg::MatchOver { winner: None }.is_sheddable());
        assert!(!ServerMsg::ActionError("x".into()).is_sheddable());
        // Events don't supersede each other (each frame is animation data).
        assert!(!ServerMsg::Events(vec![]).supersedes(&ServerMsg::Events(vec![])));
        assert!(!ServerMsg::YourSeat(1).supersedes(&ServerMsg::YourSeat(0)));

        // A combined Update is sheddable (a later update repairs state) but
        // never coalesced — it carries animation events that mustn't be lost
        // silently the way a superseded View can be.
        use crate::game::GameState;
        use crate::player::Player;
        let state = GameState::new(vec![Player::new(0, "P0"), Player::new(1, "P1")]);
        let mk_update = || ServerMsg::Update {
            events: vec![],
            view: Box::new(crate::server::view::project(&state, 0)),
        };
        assert!(mk_update().is_sheddable());
        assert!(!mk_update().supersedes(&mk_update()), "Updates never coalesce");

        // Two real Views coalesce to the latest in the outbox.
        let v1 = ServerMsg::View(Box::new(crate::server::view::project(&state, 0)));
        let v2 = ServerMsg::View(Box::new(crate::server::view::project(&state, 0)));
        let ob = Outbox::<ServerMsg>::new();
        ob.push(v1);
        ob.push(v2);
        {
            let mut g = ob.inner.lock().unwrap();
            assert_eq!(g.queue.len(), 1, "consecutive Views coalesce to one");
            assert!(matches!(g.queue.pop_front(), Some(ServerMsg::View(_))));
        }

        // Two Updates do NOT coalesce — both remain queued.
        let ob2 = Outbox::<ServerMsg>::new();
        ob2.push(mk_update());
        ob2.push(mk_update());
        assert_eq!(ob2.inner.lock().unwrap().queue.len(), 2, "Updates don't collapse");
    }
}
