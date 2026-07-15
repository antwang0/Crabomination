//! Optional HTTP status endpoint: set `CRAB_STATUS_BIND` (e.g. `0.0.0.0:7778`)
//! to serve `GET /healthz` → `ok` (load-balancer probe), `/status.json` → a
//! machine-readable metric object (for scrapers), `/metrics` → Prometheus text,
//! `/dashboard` → a self-contained HTML stat page for a browser, and any other
//! path → a plaintext operator snapshot (uptime, rolling match stats, slot
//! accounting). Unset = disabled. One thread, HTTP/1.0, connection-per-request.

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
         {} accepted, {} refused ({} global / {} per-IP, {}% refusal rate)\n",
        format_duration(started.elapsed()),
        catalog_card_count(),
        format_match_stats(&stats_snapshot),
        sl.current,
        sl.peak,
        sl.accepted,
        sl.refused_global + sl.refused_per_ip,
        sl.refused_global,
        sl.refused_per_ip,
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
         \"avg_turns\":{},\"avg_decisive_turns\":{},\"avg_draw_turns\":{},\
         \"min_turns\":{},\"max_turns\":{},\"turn_stddev\":{:.2},\
         \"median_turns\":{},\"turn_p10\":{},\"turn_p90\":{},\"turn_iqr\":{},\
         \"inconclusive\":{},\"inconclusive_pct\":{},\"decisive_pct\":{},\"draw_pct\":{},\
         \"draws\":{},\"damage_wins\":{},\"poison_wins\":{},\
         \"deckout_wins\":{},\"commander_damage_wins\":{},\"other_wins\":{},\
         \"first_seat_win_pct\":{},\"avg_win_life_delta\":{},\
         \"median_win_life_delta\":{},\"win_life_delta_p90\":{},\"win_life_delta_stddev\":{:.2},\
         \"win_life_delta_iqr\":{},\
         \"connections_current\":{},\"connections_peak\":{},\
         \"accepted\":{},\"refused\":{},\"refused_global\":{},\"refused_per_ip\":{},\
         \"refusal_rate_pct\":{},\"distinct_ips\":{},\"max_per_ip\":{},\"peak_per_ip\":{},\
         \"avg_duration_secs\":{},\"min_duration_secs\":{},\"max_duration_secs\":{},\
         \"duration_stddev_secs\":{},\"duration_buckets\":[{},{},{},{},{},{}],\
         \"catalog_cards\":{}}}\n",
        started.elapsed().as_secs(),
        st.total_matches(),
        st.bot_matches,
        st.pair_matches,
        st.avg_turns(),
        st.avg_decisive_turns(),
        st.avg_draw_turns(),
        st.min_turns.unwrap_or(0),
        st.max_turns.unwrap_or(0),
        st.turn_count_stddev(),
        st.turn_percentile(0.5),
        st.turn_percentile(0.1),
        st.turn_percentile(0.9),
        st.turn_count_iqr(),
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
        st.win_life_delta_median(),
        st.win_life_delta_percentile(0.9),
        st.win_life_delta_stddev(),
        st.win_life_delta_iqr(),
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
        st.duration_stddev().as_secs(),
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
    m("avg_decisive_turns", "gauge", "Average final-turn count of matches with a winner.", st.avg_decisive_turns().to_string());
    m("avg_draw_turns", "gauge", "Average final-turn count of drawn matches.", st.avg_draw_turns().to_string());
    m("min_turns", "gauge", "Fewest turns in a completed match.", st.min_turns.unwrap_or(0).to_string());
    m("max_turns", "gauge", "Most turns in a completed match.", st.max_turns.unwrap_or(0).to_string());
    m("turn_stddev", "gauge", "Standard deviation of final turn counts.", format!("{:.2}", st.turn_count_stddev()));
    m("median_turns", "gauge", "Median (p50) final turn count.", st.turn_percentile(0.5).to_string());
    m("turn_p10", "gauge", "10th-percentile final turn count (how fast the quickest games end).", st.turn_percentile(0.1).to_string());
    m("turn_p90", "gauge", "90th-percentile final turn count.", st.turn_percentile(0.9).to_string());
    m("turn_iqr", "gauge", "Interquartile range (p75-p25) of final turn counts.", st.turn_count_iqr().to_string());
    m("inconclusive_total", "counter", "Matches that ended with no declared outcome (stuck / disconnected).", st.inconclusive.to_string());
    m("inconclusive_pct", "gauge", "Percent of completed matches that were inconclusive.", st.inconclusive_pct().to_string());
    m("decisive_pct", "gauge", "Percent of resolved matches (wins+draws) that ended decisively.", st.decisive_pct().to_string());
    m("draw_pct", "gauge", "Percent of completed matches that ended in a draw.", st.draw_pct().to_string());
    m("avg_duration_seconds", "gauge", "Average match duration in seconds.", st.avg_duration().as_secs().to_string());
    m("min_duration_seconds", "gauge", "Shortest match duration in seconds.", st.min_duration.map(|d| d.as_secs()).unwrap_or(0).to_string());
    m("max_duration_seconds", "gauge", "Longest match duration in seconds.", st.max_duration.map(|d| d.as_secs()).unwrap_or(0).to_string());
    // Median/p90 match duration from the duration histogram — the average is
    // pulled up by a few grindy games, so operators watching "does a typical
    // match feel snappy?" want the p50, and the p90 flags the slow tail.
    m("median_duration_seconds", "gauge", "Median (p50) match duration in seconds (histogram upper bound).", st.percentile(0.5).as_secs().to_string());
    m("duration_p90_seconds", "gauge", "90th-percentile match duration in seconds (slow-tail bound).", st.percentile(0.9).as_secs().to_string());
    m("duration_stddev_seconds", "gauge", "Standard deviation of match duration in seconds (spread of game length).", st.duration_stddev().as_secs().to_string());
    m("connections_current", "gauge", "Active connections.", sl.current.to_string());
    m("connections_peak", "gauge", "Peak concurrent connections.", sl.peak.to_string());
    m("connections_accepted_total", "counter", "Connections accepted.", sl.accepted.to_string());
    m("connections_refused_total", "counter", "Connections refused.", refused.to_string());
    // Refusal rate — the share of connection attempts turned away. A rising
    // value is the "we're at capacity / under abuse" signal operators alert on;
    // derived here so it needn't be recomputed from two counters downstream.
    let attempts = sl.accepted + refused;
    let refused_pct = (refused * 100).checked_div(attempts).unwrap_or(0);
    m("connections_refused_pct", "gauge", "Percent of connection attempts refused.", refused_pct.to_string());
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
    m("median_win_life_delta", "gauge", "Median (p50) life margin of victory.", st.win_life_delta_median().to_string());
    m("win_life_delta_p90", "gauge", "90th-percentile life margin of victory (blowout tail).", st.win_life_delta_percentile(0.9).to_string());
    m("win_life_delta_stddev", "gauge", "Standard deviation of the win-by-life margin.", format!("{:.2}", st.win_life_delta_stddev()));
    m("win_life_delta_iqr", "gauge", "Interquartile range (p75-p25) of the win-by-life margin.", st.win_life_delta_iqr().to_string());
    // Split the refusals by cause so operators can tell "server at capacity"
    // (global cap) apart from "one IP hammering us" (per-IP cap) without diffing
    // two scrapes — the two alert on different runbooks.
    out.push_str("# HELP crab_connections_refused_by_reason_total Connections refused, split by which cap tripped.\n");
    out.push_str("# TYPE crab_connections_refused_by_reason_total counter\n");
    out.push_str(&format!("crab_connections_refused_by_reason_total{{reason=\"global\"}} {}\n", sl.refused_global));
    out.push_str(&format!("crab_connections_refused_by_reason_total{{reason=\"per_ip\"}} {}\n", sl.refused_per_ip));
    out.push_str("# HELP crab_wins_total Decided matches by win kind (CR 104.3).\n");
    out.push_str("# TYPE crab_wins_total counter\n");
    // `damage` + `alternate` reconciles to total decided wins; `poison`,
    // `decked`, `commander_damage`, and `other` are sub-splits of `alternate`
    // (a single win can flag more than one), so they are not additive with it.
    for (kind, value) in [
        ("damage", st.damage_wins),
        ("alternate", st.deckout_wins),
        ("poison", st.poison_wins),
        ("decked", st.deck_wins),
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
    // Turn-count histogram (see `MatchStats.turn_buckets`) as a labelled series,
    // mirroring the duration histogram so scrapers get the distribution shape —
    // a fat low-turn band flags a concession regression the average can hide.
    out.push_str("# HELP crab_match_turn_bucket Completed matches by final-turn band.\n");
    out.push_str("# TYPE crab_match_turn_bucket gauge\n");
    for (i, count) in st.turn_buckets.iter().enumerate() {
        let band = crate::stats::MatchStats::turn_bucket_label(i);
        out.push_str(&format!("crab_match_turn_bucket{{band=\"{band}\"}} {count}\n"));
    }
    // Per-seat wins — turn-order bias in bot ladders (a persistent seat-0 skew
    // is the classic first-player-advantage signal).
    out.push_str("# HELP crab_seat_wins_total Decided wins by seat index.\n");
    out.push_str("# TYPE crab_seat_wins_total counter\n");
    for (seat, wins) in st.seat_wins.iter().enumerate() {
        out.push_str(&format!("crab_seat_wins_total{{seat=\"{seat}\"}} {wins}\n"));
    }
    // Per-format completed matches + their average game length. Already in the
    // plaintext summary and dashboard; exposing them here lets a scraper alert
    // on a format-specific stall (e.g. cube averaging 3× demo's turns) or a
    // traffic shift between formats without parsing the human summary.
    out.push_str("# HELP crab_matches_by_format_total Completed matches by format.\n");
    out.push_str("# TYPE crab_matches_by_format_total counter\n");
    for (i, count) in st.format_buckets.iter().enumerate() {
        if let Some(label) = crate::stats::format_label_for_bucket(i) {
            out.push_str(&format!("crab_matches_by_format_total{{format=\"{label}\"}} {count}\n"));
        }
    }
    out.push_str("# HELP crab_format_avg_turns Average final-turn count by format.\n");
    out.push_str("# TYPE crab_format_avg_turns gauge\n");
    for (i, _) in st.format_buckets.iter().enumerate() {
        if let Some(label) = crate::stats::format_label_for_bucket(i) {
            let avg = st.format_avg_turns(i).unwrap_or(0);
            out.push_str(&format!("crab_format_avg_turns{{format=\"{label}\"}} {avg}\n"));
        }
    }
    out
}

/// Render a self-contained HTML operator dashboard: the same numbers as
/// `/status` and `/metrics`, laid out as stat tiles for a browser instead of a
/// scraper. Inline CSS only (no external fetches), a `<meta refresh>` for a
/// hands-off wall display, and theme-aware colors. Split for testing.
fn render_dashboard(started: Instant, slots: &SlotManager) -> String {
    let st = *match_stats().lock().unwrap_or_else(|p| p.into_inner());
    let sl = slots.snapshot();
    // One "tile" = a big number with a caption.
    let tile = |label: &str, value: String| {
        format!(
            "<div class=t><div class=v>{}</div><div class=l>{}</div></div>",
            html_escape(&value),
            html_escape(label),
        )
    };
    let mut tiles = String::new();
    tiles.push_str(&tile("uptime", format_duration(started.elapsed())));
    tiles.push_str(&tile("matches", st.total_matches().to_string()));
    tiles.push_str(&tile("connections", format!("{} / {} peak", sl.current, sl.peak)));
    tiles.push_str(&tile("avg turns", format!("{}", st.avg_turns())));
    tiles.push_str(&tile("median turns", st.turn_percentile(0.5).to_string()));
    tiles.push_str(&tile("decisive %", format!("{}%", st.decisive_pct())));
    tiles.push_str(&tile("draw %", format!("{}%", st.draw_pct())));
    tiles.push_str(&tile("first-seat win %", format!("{}%", st.first_seat_win_pct())));
    tiles.push_str(&tile("median duration", format_duration(st.percentile(0.5))));
    tiles.push_str(&tile("catalog cards", catalog_card_count().to_string()));
    // Win-kind mix, only the nonzero kinds so the row stays readable.
    let wins = [
        ("damage", st.damage_wins),
        ("poison", st.poison_wins),
        ("deck-out", st.deck_wins),
        ("cmdr dmg", st.commander_damage_wins),
        ("other", st.other_wins),
    ];
    let mut win_rows = String::new();
    for (kind, n) in wins.iter().filter(|(_, n)| *n > 0) {
        win_rows.push_str(&format!(
            "<tr><td>{}</td><td class=n>{}</td></tr>",
            html_escape(kind), n
        ));
    }
    if win_rows.is_empty() {
        win_rows.push_str("<tr><td colspan=2 class=muted>no decided matches yet</td></tr>");
    }
    format!(
        "<!doctype html><html lang=en><head><meta charset=utf-8>\
<meta name=viewport content=\"width=device-width,initial-scale=1\">\
<meta http-equiv=refresh content=5><title>crabomination status</title><style>\
:root{{color-scheme:light dark}}\
body{{font:14px/1.5 system-ui,sans-serif;margin:0;padding:24px;\
background:#0b0b10;color:#e8e4d8}}\
@media(prefers-color-scheme:light){{body{{background:#f5f3ec;color:#1a1a20}}}}\
h1{{font-size:18px;margin:0 0 16px;font-weight:600}}\
.grid{{display:grid;gap:12px;grid-template-columns:repeat(auto-fill,minmax(140px,1fr))}}\
.t{{border:1px solid #8884;border-radius:8px;padding:12px 14px}}\
.v{{font-size:22px;font-weight:600;font-variant-numeric:tabular-nums}}\
.l{{font-size:12px;opacity:.65;margin-top:2px;text-transform:uppercase;letter-spacing:.04em}}\
table{{margin-top:20px;border-collapse:collapse;min-width:240px}}\
td{{padding:4px 12px 4px 0}}.n{{text-align:right;font-variant-numeric:tabular-nums}}\
.muted{{opacity:.55}}h2{{font-size:13px;opacity:.7;margin:24px 0 4px;font-weight:600}}\
</style></head><body><h1>crabomination_server</h1><div class=grid>{tiles}</div>\
<h2>wins by kind</h2><table>{win_rows}</table></body></html>\n"
    )
}

/// Minimal HTML-escaping for the few dynamic strings the dashboard emits
/// (durations, labels). Everything interpolated is server-controlled today, but
/// escaping keeps the page injection-proof if a label ever grows dynamic.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
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
        "/dashboard" => ("200 OK", "text/html; charset=utf-8", render_dashboard(started, slots)),
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
                    "\"avg_decisive_turns\":0", "\"avg_draw_turns\":0",
                    "\"poison_wins\":0", "\"deckout_wins\":0", "\"other_wins\":0",
                    "\"first_seat_win_pct\":50", "\"avg_win_life_delta\":0",
                    "\"median_win_life_delta\":0", "\"win_life_delta_p90\":0",
                    "\"win_life_delta_stddev\":0.00",
                    "\"min_turns\":0", "\"max_turns\":0", "\"turn_stddev\":0.00",
                    "\"median_turns\":0", "\"turn_p10\":0", "\"turn_p90\":0", "\"turn_iqr\":0",
                    "\"inconclusive\":0", "\"inconclusive_pct\":0",
                    "\"decisive_pct\":0", "\"draw_pct\":0",
                    "\"avg_duration_secs\":0", "\"min_duration_secs\":0",
                    "\"duration_stddev_secs\":0",
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
        assert_eq!(route("GET", "/dashboard", now, &slots).1, "text/html; charset=utf-8");
    }

    #[test]
    fn render_dashboard_is_self_contained_html() {
        let slots = SlotManager::new(10, 5);
        let body = render_dashboard(Instant::now(), &slots);
        assert!(body.starts_with("<!doctype html>"), "html document");
        assert!(body.contains("<title>crabomination status</title>"));
        assert!(body.contains("uptime") && body.contains("matches"), "stat tiles present");
        assert!(body.contains("no decided matches yet"), "empty win table on a fresh server");
        // No external references — a strict CSP / offline viewer must render it.
        assert!(!body.contains("http://") && !body.contains("https://"), "no external URLs");
        assert!(!body.contains("src=") && !body.contains("href="), "no external assets");
    }

    #[test]
    fn html_escape_neutralizes_markup() {
        assert_eq!(html_escape("<b>&</b>"), "&lt;b&gt;&amp;&lt;/b&gt;");
    }

    #[test]
    fn render_metrics_is_prometheus_text() {
        let slots = SlotManager::new(10, 5);
        let body = render_metrics(Instant::now(), &slots);
        assert!(body.contains("# TYPE crab_matches_total counter"));
        assert!(body.contains("crab_connections_current 0"));
        assert!(body.contains("# HELP crab_uptime_seconds"));
        // Win-kind breakdown is a labelled series; `damage` + `alternate`
        // reconcile to total decided wins.
        assert!(body.contains("crab_wins_total{kind=\"damage\"} 0"));
        assert!(body.contains("crab_wins_total{kind=\"alternate\"} 0"));
        assert!(body.contains("crab_wins_total{kind=\"poison\"} 0"));
        assert!(body.contains("crab_wins_total{kind=\"commander_damage\"} 0"));
        // Refusals split by which cap tripped (global vs per-IP).
        assert!(body.contains("crab_connections_refused_by_reason_total{reason=\"global\"} 0"));
        assert!(body.contains("crab_connections_refused_by_reason_total{reason=\"per_ip\"} 0"));
        // Each metric name must carry exactly one HELP/TYPE block — Prometheus
        // rejects a whole scrape on a duplicated HELP line (regression guard).
        assert_eq!(
            body.matches("# TYPE crab_connections_refused_by_reason_total").count(),
            1,
            "refusal-by-reason metric must be declared exactly once",
        );
        assert!(body.contains("# TYPE crab_draws_total counter"));
        // Match-outcome health gauges (stuck-match / decisive / draw shares).
        assert!(body.contains("crab_inconclusive_total 0"));
        assert!(body.contains("crab_decisive_pct 0"));
        // Duration gauges + histogram bands.
        assert!(body.contains("crab_avg_duration_seconds 0"));
        assert!(body.contains("crab_median_duration_seconds 0"));
        assert!(body.contains("crab_duration_p90_seconds 0"));
        assert!(body.contains("crab_duration_stddev_seconds 0"));
        assert!(body.contains("crab_match_duration_bucket{band=\"<30s\"} 0"));
        // Refusal breakdown by reason + peak-per-ip gauge.
        assert!(body.contains("crab_connections_refused_by_reason_total{reason=\"global\"} 0"));
        assert!(body.contains("crab_connections_refused_by_reason_total{reason=\"per_ip\"} 0"));
        assert!(body.contains("crab_peak_per_ip 0"));
        assert!(body.contains("crab_connections_refused_pct 0"));
        // Turn-count distribution gauges.
        assert!(body.contains("crab_min_turns 0"));
        assert!(body.contains("crab_max_turns 0"));
        assert!(body.contains("crab_turn_stddev 0.00"));
        assert!(body.contains("crab_median_turns 0"));
        assert!(body.contains("crab_turn_p90 0"));
        assert!(body.contains("crab_turn_iqr 0"));
        // Turn-count histogram + per-seat wins as labelled series.
        assert!(body.contains("crab_match_turn_bucket{band=\"1-2\"} 0"));
        assert!(body.contains("crab_seat_wins_total{seat=\"0\"} 0"));
        // Per-format match counts + average game length as labelled series.
        assert!(body.contains("crab_matches_by_format_total{format=\"demo\"} 0"));
        assert!(body.contains("crab_matches_by_format_total{format=\"cube\"} 0"));
        assert!(body.contains("crab_format_avg_turns{format=\"cube\"} 0"));
        // Play/draw balance + win-margin gauges.
        assert!(body.contains("crab_first_seat_win_pct 50"));
        assert!(body.contains("# TYPE crab_avg_win_life_delta gauge"));
        // Routed as Prometheus text exposition.
        let now = Instant::now();
        assert_eq!(route("GET", "/metrics", now, &slots).1, "text/plain; version=0.0.4");
    }
}
