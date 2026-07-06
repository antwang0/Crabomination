//! Browser (wasm32) WebSocket transport.
//!
//! Mirrors the native TCP transport in `net_plugin`/`lobby_ui`: produces the
//! same `(NetOutbox, NetInbox)` mpsc pair the rest of the client consumes,
//! backed by the browser's WebSocket API instead of a `TcpStream` + threads.
//! Wire format matches the server's `ws.rs`: one WebSocket binary message =
//! one JSON `ClientMsg`/`ServerMsg`.
//!
//! There are no threads on wasm: the `onmessage` callback runs on the main
//! JS thread between frames and pushes decoded `ServerMsg`s straight into
//! the inbox mpsc, while [`pump_ws`] (a Bevy system) drains the outbox mpsc
//! into `WebSocket::send` each frame. Because `connect()` is asynchronous
//! in the browser, outbound messages queue in the mpsc until `onopen` fires
//! — nothing is lost, just deferred a few frames.
//!
//! Disconnect detection mirrors the native reader-thread convention: when
//! the socket closes/errors, [`pump_ws`] drops the [`WsSocket`] resource,
//! which drops the `onmessage` closure and with it the inbox sender — so
//! `NetInbox::drain` reports `disconnected` exactly like it does natively.

#![cfg(target_arch = "wasm32")]

use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;

use bevy::prelude::*;
use crabomination::net::{ClientMsg, ServerMsg};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{BinaryType, MessageEvent, WebSocket};

use crate::net_plugin::{NetInbox, NetOutbox};

/// The live browser socket plus the JS callbacks keeping it fed. A NonSend
/// resource (`web_sys::WebSocket` is not `Send`); dropped to close the
/// connection and signal disconnect to the inbox.
pub struct WsSocket {
    socket: WebSocket,
    out_rx: mpsc::Receiver<ClientMsg>,
    open: Rc<Cell<bool>>,
    closed: Rc<Cell<bool>>,
    // Keep the callbacks alive for the socket's lifetime; dropping them
    // unregisters the handlers (and drops the inbox sender they hold).
    _onmessage: Closure<dyn FnMut(MessageEvent)>,
    _onopen: Closure<dyn FnMut()>,
    _onclose: Closure<dyn FnMut()>,
    _onerror: Closure<dyn FnMut()>,
}

impl Drop for WsSocket {
    fn drop(&mut self) {
        // Detach handlers before the closures die so the browser can't call
        // into freed memory, then close politely.
        self.socket.set_onmessage(None);
        self.socket.set_onopen(None);
        self.socket.set_onclose(None);
        self.socket.set_onerror(None);
        let _ = self.socket.close();
    }
}

/// Normalize a user-entered address into a WebSocket URL: `host:port` →
/// `ws://host:port`, keeping explicit `ws://`/`wss://` untouched. Pages
/// served over HTTPS can only open `wss://`, so an explicit scheme wins.
fn ws_url(addr: &str) -> String {
    if addr.starts_with("ws://") || addr.starts_with("wss://") {
        addr.to_string()
    } else {
        format!("ws://{addr}")
    }
}

/// Open a WebSocket to `addr` and wrap it into the client channel pair.
/// Returns the same `(NetOutbox, NetInbox)` the native `tcp_client` path
/// produces, plus the [`WsSocket`] to insert as a NonSend resource (its drop
/// closes the connection). Fails only on an invalid URL — connection
/// failures surface asynchronously as a disconnect.
pub fn ws_connect(addr: &str) -> Result<(NetOutbox, NetInbox, WsSocket), String> {
    let url = ws_url(addr);
    let socket = WebSocket::new(&url).map_err(|e| format!("{e:?}"))?;
    socket.set_binary_type(BinaryType::Arraybuffer);

    let (in_tx, in_rx) = mpsc::channel::<ServerMsg>();
    let (out_tx, out_rx) = mpsc::channel::<ClientMsg>();
    let open = Rc::new(Cell::new(false));
    let closed = Rc::new(Cell::new(false));

    let onmessage = {
        let in_tx = in_tx;
        Closure::<dyn FnMut(MessageEvent)>::new(move |ev: MessageEvent| {
            let data = ev.data();
            let bytes: Vec<u8> = if let Ok(buf) = data.clone().dyn_into::<js_sys::ArrayBuffer>() {
                js_sys::Uint8Array::new(&buf).to_vec()
            } else if let Some(text) = data.as_string() {
                text.into_bytes()
            } else {
                return;
            };
            match serde_json::from_slice::<ServerMsg>(&bytes) {
                Ok(msg) => {
                    let _ = in_tx.send(msg);
                }
                Err(e) => {
                    warn!("ws: dropping malformed server message: {e}");
                }
            }
        })
    };
    let onopen = {
        let open = Rc::clone(&open);
        Closure::<dyn FnMut()>::new(move || open.set(true))
    };
    let onclose = {
        let closed = Rc::clone(&closed);
        Closure::<dyn FnMut()>::new(move || closed.set(true))
    };
    let onerror = {
        let closed = Rc::clone(&closed);
        Closure::<dyn FnMut()>::new(move || closed.set(true))
    };
    socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    socket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    socket.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    socket.set_onerror(Some(onerror.as_ref().unchecked_ref()));

    Ok((
        NetOutbox::new(out_tx),
        NetInbox(std::sync::Mutex::new(in_rx)),
        WsSocket {
            socket,
            out_rx,
            open,
            closed,
            _onmessage: onmessage,
            _onopen: onopen,
            _onclose: onclose,
            _onerror: onerror,
        },
    ))
}

/// Per-frame pump: once the socket is open, drain queued outbound
/// `ClientMsg`s into `WebSocket::send`; when the socket dies, remove the
/// [`WsSocket`] resource so the inbox reports disconnected (the reconnect /
/// menu-return flows key off that, same as native). Exclusive system so it
/// can remove the NonSend resource itself.
pub fn pump_ws(world: &mut World) {
    let Some(ws) = world.get_non_send::<WsSocket>() else { return };
    if ws.closed.get() {
        world.remove_non_send::<WsSocket>();
        return;
    }
    if !ws.open.get() {
        return; // still connecting; outbound messages wait in the mpsc
    }
    let mut dead = false;
    while let Ok(msg) = ws.out_rx.try_recv() {
        let Ok(payload) = serde_json::to_vec(&msg) else { continue };
        if ws.socket.send_with_u8_array(&payload).is_err() {
            dead = true;
            break;
        }
    }
    if dead {
        world.remove_non_send::<WsSocket>();
    }
}
