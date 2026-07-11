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

/// Distinct card count of the deployed catalog, computed once. Lets operators
/// confirm which content build is live (`crab_catalog_cards`) without shelling
/// into the container. Building the registry is non-trivial, so it's memoized.
fn catalog_card_count() -> usize {
    use std::sync::OnceLock;
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| crabomination::catalog::all_known_factories().len())
}

/// Render the full status body. Split from the serving loop for testing.
fn render_status(started: Instant, slots: &SlotManager) -> String {
    let stats_snapshot = *match_stats().lock().unwrap_or_else(|p| p.into_inner());
    let sl = slots.snapshot();
    format!(
        "crabomination_server\nuptime: {}\ncatalog: {} cards\n{}\nconnections: {} current, {} peak, \
         {} accepted, {} refused ({}% refusal rate)\n",
        format_duration(started.elapsed()),
        catalog_card_count(),
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
         \"avg_turns\":{},\"min_turns\":{},\"max_turns\":{},\"turn_stddev\":{:.2},\
         \"median_turns\":{},\"turn_p90\":{},\
         \"inconclusive\":{},\"inconclusive_pct\":{},\"decisive_pct\":{},\"draw_pct\":{},\
         \"draws\":{},\"damage_wins\":{},\"poison_wins\":{},\
         \"deckout_wins\":{},\"commander_damage_wins\":{},\"other_wins\":{},\
         \"first_seat_win_pct\":{},\"avg_win_life_delta\":{},\
         \"connections_current\":{},\"connections_peak\":{},\
         \"accepted\":{},\"refused\":{},\"refused_global\":{},\"refused_per_ip\":{},\
         \"refusal_rate_pct\":{},\"distinct_ips\":{},\"max_per_ip\":{},\"peak_per_ip\":{},\
         \"avg_duration_secs\":{},\"min_duration_secs\":{},\"max_duration_secs\":{},\
         \"duration_buckets\":[{},{},{},{},{},{}],\
         \"catalog_cards\":{}}}\n",
        started.elapsed().as_secs(),
        st.total_matches(),
        st.bot_matches,
        st.pair_matches,
        st.avg_turns(),
        st.min_turns.unwrap_or(0),
        st.max_turns.unwrap_or(0),
        st.turn_count_stddev(),
        st.turn_percentile(0.5),
        st.turn_percentile(0.9),
        st.inconclusive,
        st.inconclusive_pct(),
        st.decisive_pct(),
        st.draw_pct(),
        st.draws,
        st.damage_wins,
        st.poison_wins,
        st.deck_wins,
        st.commander_damage_wins,
        st.other_wins,
        st.first_seat_win_pct(),
        st.avg_win_life_delta(),
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
        st.avg_duration().as_secs(),
        st.min_duration.map(|d| d.as_secs()).unwrap_or(0),
        st.max_duration.map(|d| d.as_secs()).unwrap_or(0),
        st.duration_buckets[0],
        st.duration_buckets[1],
        st.duration_buckets[2],
        st.duration_buckets[3],
        st.duration_buckets[4],
        st.duration_buckets[5],
        catalog_card_count(),
    )
}

/// Render a Prometheus text-exposition (`/metrics`) snapshot — the same numbers
/// as `/status.json`, one `# HELP`/`# TYPE`/sample triple per metric, so a
/// Prometheus scraper ingests the server directly (the conventional `/metrics`
/// path expects this text format, not JSON).
fn render_metrics(started: Instant, slots: &SlotManager) -> String {
    let st = *match_stats().lock().unwrap_or_else(|p| p.into_inner());
    let sl = slots.snapshot();
    let refused = sl.refused_global + sl.refused_per_ip;
    let mut out = String::new();
    let mut m = |name: &str, kind: &str, help: &str, value: String| {
        out.push_str(&format!(
            "# HELP crab_{name} {help}\n# TYPE crab_{name} {kind}\ncrab_{name} {value}\n"
        ));
    };
    m("uptime_seconds", "counter", "Server uptime in seconds.", started.elapsed().as_secs().to_string());
    m("matches_total", "counter", "Matches served.", st.total_matches().to_string());
    m("bot_matches_total", "counter", "Matches against a bot.", st.bot_matches.to_string());
    m("pair_matches_total", "counter", "Human-vs-human matches.", st.pair_matches.to_string());
    m("avg_turns", "gauge", "Average turns per match.", st.avg_turns().to_string());
    m("min_turns", "gauge", "Fewest turns in a completed match.", st.min_turns.unwrap_or(0).to_string());
    m("max_turns", "gauge", "Most turns in a completed match.", st.max_turns.unwrap_or(0).to_string());
    m("turn_stddev", "gauge", "Standard deviation of final turn counts.", format!("{:.2}", st.turn_count_stddev()));
    m("median_turns", "gauge", "Median (p50) final turn count.", st.turn_percentile(0.5).to_string());
    m("turn_p90", "gauge", "90th-percentile final turn count.", st.turn_percentile(0.9).to_string());
    m("inconclusive_total", "counter", "Matches that ended with no declared outcome (stuck / disconnected).", st.inconclusive.to_string());
    m("inconclusive_pct", "gauge", "Percent of completed matches that were inconclusive.", st.inconclusive_pct().to_string());
    m("decisive_pct", "gauge", "Percent of resolved matches (wins+draws) that ended decisively.", st.decisive_pct().to_string());
    m("draw_pct", "gauge", "Percent of completed matches that ended in a draw.", st.draw_pct().to_string());
    m("avg_duration_seconds", "gauge", "Average match duration in seconds.", st.avg_duration().as_secs().to_string());
    m("min_duration_seconds", "gauge", "Shortest match duration in seconds.", st.min_duration.map(|d| d.as_secs()).unwrap_or(0).to_string());
    m("max_duration_seconds", "gauge", "Longest match duration in seconds.", st.max_duration.map(|d| d.as_secs()).unwrap_or(0).to_string());
    m("connections_current", "gauge", "Active connections.", sl.current.to_string());
    m("connections_peak", "gauge", "Peak concurrent connections.", sl.peak.to_string());
    m("connections_accepted_total", "counter", "Connections accepted.", sl.accepted.to_string());
    m("connections_refused_total", "counter", "Connections refused.", refused.to_string());
    m("distinct_ips", "gauge", "Distinct client IPs seen.", sl.distinct_ips.to_string());
    m("peak_per_ip", "gauge", "Highest simultaneous connection count from a single IP.", sl.peak_per_ip.to_string());
    m("catalog_cards", "gauge", "Distinct cards in the deployed catalog.", catalog_card_count().to_string());
    // Win-kind breakdown (CR 104.3) — how decided games ended, as a labelled
    // `crab_wins_total{kind="…"}` series so operators can watch the
    // damage/poison/deck-out/commander mix shift without parsing the page.
    m("draws_total", "counter", "Matches ending in a draw.", st.draws.to_string());
    // Play/draw balance: the share of decided matches the first seat won. A
    // persistent skew above ~55% is the classic "first-player advantage" signal
    // operators watch when tuning bot mulligans or seat assignment.
    m("first_seat_win_pct", "gauge", "Percent of decided matches won by the first seat.", st.first_seat_win_pct().to_string());
    m("avg_win_life_delta", "gauge", "Average life margin of victory across wins.", st.avg_win_life_delta().to_string());
    out.push_str("# HELP crab_wins_total Decided matches by win kind (CR 104.3).\n");
    out.push_str("# TYPE crab_wins_total counter\n");
    for (kind, value) in [
        ("damage", st.damage_wins),
        ("poison", st.poison_wins),
        ("deckout", st.deck_wins),
        ("commander_damage", st.commander_damage_wins),
        ("other", st.other_wins),
    ] {
        out.push_str(&format!("crab_wins_total{{kind=\"{kind}\"}} {value}\n"));
    }
    // Match-duration histogram (see `MatchStats.duration_buckets`) as a labelled
    // series so operators can watch the distribution shift (e.g. a spike in the
    // "<30s" bucket flags bots conceding turn 1).
    out.push_str("# HELP crab_match_duration_bucket Completed matches by duration band.\n");
    out.push_str("# TYPE crab_match_duration_bucket gauge\n");
    for (band, count) in [
        ("<30s", st.duration_buckets[0]),
        ("30s-1m", st.duration_buckets[1]),
        ("1-2m", st.duration_buckets[2]),
        ("2-5m", st.duration_buckets[3]),
        ("5-10m", st.duration_buckets[4]),
        ("10m+", st.duration_buckets[5]),
    ] {
        out.push_str(&format!("crab_match_duration_bucket{{band=\"{band}\"}} {count}\n"));
    }
    // Refusal breakdown so operators can tell a capacity-limit refusal (server
    // full) from an abuse refusal (one IP over its per-IP cap).
    out.push_str("# HELP crab_connections_refused_by_reason_total Refused connections by reason.\n");
    out.push_str("# TYPE crab_connections_refused_by_reason_total counter\n");
    for (reason, value) in [("global", sl.refused_global), ("per_ip", sl.refused_per_ip)] {
        out.push_str(&format!("crab_connections_refused_by_reason_total{{reason=\"{reason}\"}} {value}\n"));
    }
    out
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
        "/metrics" => ("200 OK", "text/plain; version=0.0.4", render_metrics(started, slots)),
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
        assert!(body.contains("\ncatalog: ") && body.contains(" cards\n"), "catalog line present");
        assert!(body.contains("served "), "match stats line present");
        assert!(body.contains("connections: 0 current, 0 peak"), "slot line present");
    }

    #[test]
    fn render_status_json_is_well_formed() {
        let slots = SlotManager::new(10, 5);
        let body = render_status_json(Instant::now(), &slots);
        assert!(body.starts_with('{') && body.trim_end().ends_with('}'), "JSON object");
        // Key fields present with numeric values (no fresh-server nulls).
        for key in ["\"matches\":0", "\"connections_current\":0", "\"refusal_rate_pct\":0",
                    "\"poison_wins\":0", "\"deckout_wins\":0", "\"other_wins\":0",
                    "\"first_seat_win_pct\":50", "\"avg_win_life_delta\":0",
                    "\"min_turns\":0", "\"max_turns\":0", "\"turn_stddev\":0.00",
                    "\"median_turns\":0", "\"turn_p90\":0",
                    "\"inconclusive\":0", "\"inconclusive_pct\":0",
                    "\"decisive_pct\":0", "\"draw_pct\":0",
                    "\"avg_duration_secs\":0", "\"min_duration_secs\":0",
                    "\"duration_buckets\":[0,0,0,0,0,0]"] {
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

    #[test]
    fn render_metrics_is_prometheus_text() {
        let slots = SlotManager::new(10, 5);
        let body = render_metrics(Instant::now(), &slots);
        assert!(body.contains("# TYPE crab_matches_total counter"));
        assert!(body.contains("crab_connections_current 0"));
        assert!(body.contains("# HELP crab_uptime_seconds"));
        // Win-kind breakdown is a labelled series.
        assert!(body.contains("crab_wins_total{kind=\"poison\"} 0"));
        assert!(body.contains("crab_wins_total{kind=\"commander_damage\"} 0"));
        assert!(body.contains("# TYPE crab_draws_total counter"));
        // Match-outcome health gauges (stuck-match / decisive / draw shares).
        assert!(body.contains("crab_inconclusive_total 0"));
        assert!(body.contains("crab_decisive_pct 0"));
        // Duration gauges + histogram bands.
        assert!(body.contains("crab_avg_duration_seconds 0"));
        assert!(body.contains("crab_match_duration_bucket{band=\"<30s\"} 0"));
        // Refusal breakdown by reason + peak-per-ip gauge.
        assert!(body.contains("crab_connections_refused_by_reason_total{reason=\"global\"} 0"));
        assert!(body.contains("crab_connections_refused_by_reason_total{reason=\"per_ip\"} 0"));
        assert!(body.contains("crab_peak_per_ip 0"));
        // Turn-count distribution gauges.
        assert!(body.contains("crab_min_turns 0"));
        assert!(body.contains("crab_max_turns 0"));
        assert!(body.contains("crab_turn_stddev 0.00"));
        assert!(body.contains("crab_median_turns 0"));
        assert!(body.contains("crab_turn_p90 0"));
        // Play/draw balance + win-margin gauges.
        assert!(body.contains("crab_first_seat_win_pct 50"));
        assert!(body.contains("# TYPE crab_avg_win_life_delta gauge"));
        // Routed as Prometheus text exposition.
        let now = Instant::now();
        assert_eq!(route("GET", "/metrics", now, &slots).1, "text/plain; version=0.0.4");
    }
}
