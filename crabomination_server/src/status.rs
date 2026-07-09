//! Optional HTTP status endpoint: set `CRAB_STATUS_BIND` (e.g. `0.0.0.0:7778`)
//! to serve `GET /healthz` → `ok` (load-balancer probe), `/status.json` → a
//! machine-readable metric object (for scrapers), and any other path → a
//! plaintext operator snapshot (uptime, rolling match stats, slot accounting).
//! Unset = disabled. One thread, HTTP/1.0, connection-per-request.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Instant;

use crate::slots::SlotManager;
use crate::stats::{format_duration, format_match_stats, match_stats};

/// Render the full status body. Split from the serving loop for testing.
fn render_status(started: Instant, slots: &SlotManager) -> String {
    let stats_snapshot = *match_stats().lock().unwrap_or_else(|p| p.into_inner());
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

/// Render a machine-readable JSON snapshot for metric scrapers. Flat object of
/// the same numbers the plaintext page carries, so Prometheus/textfile or a
/// simple `jq` probe can read them without parsing the human summary. Split
/// from the serving loop for testing.
fn render_status_json(started: Instant, slots: &SlotManager) -> String {
    let st = *match_stats().lock().unwrap_or_else(|p| p.into_inner());
    let sl = slots.snapshot();
    let refused = sl.refused_global + sl.refused_per_ip;
    format!(
        "{{\"uptime_secs\":{},\"matches\":{},\"bot_matches\":{},\"pair_matches\":{},\
         \"avg_turns\":{},\"connections_current\":{},\"connections_peak\":{},\
         \"accepted\":{},\"refused\":{},\"refused_global\":{},\"refused_per_ip\":{},\
         \"refusal_rate_pct\":{},\"distinct_ips\":{},\"max_per_ip\":{},\"peak_per_ip\":{}}}\n",
        started.elapsed().as_secs(),
        st.total_matches(),
        st.bot_matches,
        st.pair_matches,
        st.avg_turns(),
        sl.current,
        sl.peak,
        sl.accepted,
        refused,
        sl.refused_global,
        sl.refused_per_ip,
        sl.refusal_rate_pct(),
        sl.distinct_ips,
        sl.max_per_ip,
        sl.peak_per_ip,
    )
}

/// Route one request to `(status_line, content_type, body)`. Only GET/HEAD are
/// served; unknown paths 404 and other methods 405 so scrapers and probes read
/// correct semantics instead of a 200 status page for everything.
fn route(method: &str, path: &str, started: Instant, slots: &SlotManager) -> (&'static str, &'static str, String) {
    if method != "GET" && method != "HEAD" {
        return ("405 Method Not Allowed", "text/plain", "method not allowed\n".to_string());
    }
    match path {
        "/healthz" => ("200 OK", "text/plain", "ok\n".to_string()),
        "/status.json" | "/metrics.json" => ("200 OK", "application/json", render_status_json(started, slots)),
        "/status" | "/" => ("200 OK", "text/plain", render_status(started, slots)),
        _ => ("404 Not Found", "text/plain", "not found\n".to_string()),
    }
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
    eprintln!("status endpoint listening on http://{bind} (/healthz, /status, /status.json)");
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
            let mut parts = request_line.split_whitespace();
            let method = parts.next().unwrap_or("");
            let path = parts.next().unwrap_or("");
            let (status, content_type, body) = route(method, path, started, &slots);
            // HEAD gets headers only (CR-agnostic HTTP nicety for probes).
            let payload = if method == "HEAD" { "" } else { &body };
            let _ = write!(
                stream,
                "HTTP/1.0 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                content_type,
                body.len(),
                payload
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

    #[test]
    fn render_status_json_is_well_formed() {
        let slots = SlotManager::new(10, 5);
        let body = render_status_json(Instant::now(), &slots);
        assert!(body.starts_with('{') && body.trim_end().ends_with('}'), "JSON object");
        // Key fields present with numeric values (no fresh-server nulls).
        for key in ["\"matches\":0", "\"connections_current\":0", "\"refusal_rate_pct\":0"] {
            assert!(body.contains(key), "missing {key} in {body}");
        }
    }

    #[test]
    fn route_status_codes_and_content_types() {
        let now = Instant::now();
        let slots = SlotManager::new(10, 5);
        assert_eq!(route("GET", "/healthz", now, &slots).0, "200 OK");
        assert_eq!(route("GET", "/status.json", now, &slots).1, "application/json");
        assert_eq!(route("GET", "/status", now, &slots).0, "200 OK");
        assert_eq!(route("GET", "/bogus", now, &slots).0, "404 Not Found");
        assert_eq!(route("POST", "/status", now, &slots).0, "405 Method Not Allowed");
        assert_eq!(route("HEAD", "/healthz", now, &slots).0, "200 OK");
    }
}
