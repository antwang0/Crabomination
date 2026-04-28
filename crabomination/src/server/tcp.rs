//! TCP transport for the match server.
//!
//! Each TCP connection is wrapped into a [`SeatChannel`] (server side) or a
//! [`ClientChannel`] (client side) by spawning a reader thread (socket → mpsc)
//! and a writer thread (mpsc → socket). The match actor then talks to the
//! socket-backed channels exactly as it does to in-process ones.
//!
//! Wire format: 4-byte big-endian length prefix followed by JSON-serialized
//! [`ClientMsg`] / [`ServerMsg`] payload.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

use serde::{Deserialize, Serialize};

use crate::net::{ClientMsg, ServerMsg};

use super::{ClientChannel, SeatChannel};

const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

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

/// Wrap a [`TcpStream`] into a server-side [`SeatChannel`]. Spawns one reader
/// thread (socket → mpsc<ClientMsg>) and one writer thread (mpsc<ServerMsg> →
/// socket). When either side disconnects, both threads exit.
pub fn tcp_seat(stream: TcpStream) -> io::Result<SeatChannel> {
    stream.set_nodelay(true)?;
    let read_stream = stream.try_clone()?;
    let mut write_stream = stream;

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

    // Writer: out_rx → ServerMsg → socket
    thread::spawn(move || {
        while let Ok(msg) = out_rx.recv() {
            if write_frame(&mut write_stream, &msg).is_err() {
                break;
            }
        }
    });

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
    let read_stream = stream.try_clone()?;
    let mut write_stream = stream;

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

    // Writer: out_rx → ClientMsg → socket
    thread::spawn(move || {
        while let Ok(msg) = out_rx.recv() {
            if write_frame(&mut write_stream, &msg).is_err() {
                break;
            }
        }
    });

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
}
