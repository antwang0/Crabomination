//! Optional HTTP status endpoint: set `CRAB_STATUS_BIND` (e.g. `0.0.0.0:7778`)
//! to serve `GET /healthz` → `ok` (load-balancer probe) and any other path →
//! a plaintext operator snapshot (uptime, rolling match stats, slot
//! accounting). Unset = disabled. One thread, HTTP/1.0, connection-per-request.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Instant;

use crate::slots::SlotManager;
use crate::stats::{format_duration, format_match_stats, match_stats};

/// Render the full status body. Split from the serving loop for testing.
fn render_status(started: Instant, slots: &SlotManager) -> String {
    let stats_snapshot = {
        let s = match_stats().lock().unwrap_or_else(|p| p.into_inner());
        s.clone()
    };
    let sl = slots.snapshot();
    format!(
        "crabomination_server\nuptime: {}\n{}\nconnections: {} current, {} peak, \
         {} accepted, {} refused ({}% refusal rate)\n",
        format_duration(started.elapsed()),
        format_match_stats(&stats_snapshot),
        sl.current,
        sl.peak,
        sl.accepted,
        sl.refused_global + sl.refused_per_ip,
        sl.refusal_rate_pct(),
    )
}

/// Spawn the status listener thread if `CRAB_STATUS_BIND` is set. Bind
/// failures are non-fatal (the match server keeps running without telemetry).
pub(crate) fn spawn_from_env(started: Instant, slots: SlotManager) {
    let Some(bind) = std::env::var_os("CRAB_STATUS_BIND") else { return };
    let bind = bind.to_string_lossy().into_owned();
    let listener = match TcpListener::bind(&bind) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("warning: CRAB_STATUS_BIND={bind} failed to bind: {e}; status endpoint disabled");
            return;
        }
    };
    eprintln!("status endpoint listening on http://{bind} (/healthz, /status)");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            // Read just enough for the request line; ignore errors — a
            // malformed probe gets the full status page.
            let mut buf = [0u8; 512];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request_line = std::str::from_utf8(&buf[..n])
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("");
            let healthz = request_line.split_whitespace().nth(1) == Some("/healthz");
            let body = if healthz {
                "ok\n".to_string()
            } else {
                render_status(started, &slots)
            };
            let _ = write!(
                stream,
                "HTTP/1.0 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_status_includes_uptime_stats_and_slots() {
        let slots = SlotManager::new(10, 5);
        let body = render_status(Instant::now(), &slots);
        assert!(body.starts_with("crabomination_server\nuptime: "));
        assert!(body.contains("served "), "match stats line present");
        assert!(body.contains("connections: 0 current, 0 peak"), "slot line present");
    }
}
