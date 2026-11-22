#!/usr/bin/env python3
"""HTML report for `fra_card_eval --pool-adds` (one self-contained file).

    python3 scripts/fra_pool_adds_report.py .ladder/fra_pool_adds/s43 \
        --draftsim .ladder/fra_card_eval/FRA_draftsim.tsv \
        --l17 .ladder/fra_card_eval/l17_PremierDraft.json \
        --out fra_pool_report.html

Value-net numbers are game logits; the page shows them as win-rate points
(x 25, the logistic's slope at an even match). Sim values are already
win-rate points per copy.
"""
import argparse
import csv
import json
import math
import os
import sys


def read_tsv(path):
    with open(path, newline="") as f:
        lines = [l for l in f if not l.startswith("#")]
    return list(csv.DictReader(lines, delimiter="\t"))


def fnum(s):
    try:
        v = float(s)
        return None if math.isnan(v) else v
    except (TypeError, ValueError):
        return None


def spearman(xs, ys):
    pairs = [(x, y) for x, y in zip(xs, ys) if x is not None and y is not None]
    if len(pairs) < 3:
        return None, 0

    def ranks(v):
        order = sorted(range(len(v)), key=lambda i: v[i])
        r = [0.0] * len(v)
        i = 0
        while i < len(order):
            j = i
            while j + 1 < len(order) and v[order[j + 1]] == v[order[i]]:
                j += 1
            for k in range(i, j + 1):
                r[order[k]] = (i + j) / 2
            i = j + 1
        return r

    rx, ry = ranks([p[0] for p in pairs]), ranks([p[1] for p in pairs])
    mx, my = sum(rx) / len(rx), sum(ry) / len(ry)
    cov = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    vx = sum((a - mx) ** 2 for a in rx) ** 0.5
    vy = sum((b - my) ** 2 for b in ry) ** 0.5
    return cov / (vx * vy), len(pairs)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--draftsim")
    ap.add_argument("--l17")
    ap.add_argument("--confirm", action="append", default=[], help="extra confirm.tsv (another seed)")
    ap.add_argument("--meta", default="{}", help="JSON merged into the page's meta block")
    ap.add_argument("--set-played", help="set_played.tsv from fra_card_eval --set-played")
    ap.add_argument("--set-played-before", help="the same rating before a fix batch, shown beside it")
    ap.add_argument("--issues", help="JSON: card name -> {kind, text}")
    ap.add_argument("--out", required=True)
    a = ap.parse_args()

    L = 25.0  # logits -> win-rate points
    ds = {}
    if a.draftsim:
        for r in read_tsv(a.draftsim):
            ds[r["Name"].replace("_", " ")] = fnum(r["Rating"])
    l17 = {}
    if a.l17:
        for r in json.load(open(a.l17)):
            l17[r["name"]] = r

    def lookup(table, name):
        if name in table:
            return table[name]
        front = name.split(" // ")[0]
        for k, v in table.items():
            if k.split(" // ")[0] == front:
                return v
        return None

    cards = []
    for r in read_tsv(os.path.join(a.run, "set.tsv")):
        n = r["name"]
        l = lookup(l17, n) or {}
        gih = l.get("ever_drawn_win_rate")
        cards.append({
            "name": n,
            "rarity": r["rarity"],
            "cost": r["cost"],
            "colors": r["colors"],
            "cmc": int(r["cmc"]),
            "types": r["types"].strip("[]").replace(",", ""),
            "sim": fnum(r["sim_value"]),
            "a": None if fnum(r["dv_a"]) is None else fnum(r["dv_a"]) * L,
            "gain": None if fnum(r["dv_gain"]) is None else fnum(r["dv_gain"]) * L,
            "gain_ci": None if fnum(r["dv_gain_ci"]) is None else fnum(r["dv_gain_ci"]) * L,
            "fit": int(r["fit_decks"]),
            "played": int(r["played_in"]),
            "in_pool": int(r["in_pool"]),
            "knows": r["dv_knows"] == "1",
            "draftsim": lookup(ds, n),
            "gih": None if gih is None else 100 * gih,
            "gih_n": l.get("ever_drawn_game_count"),
            "img": l.get("url") or None,
        })
    by_name = {c["name"]: c for c in cards}
    for c in cards:
        c["pg"] = c["pg_ci"] = c["pg_n"] = c["pg_pred"] = c["issue"] = None
    if a.set_played:
        for r in read_tsv(a.set_played):
            c = by_name.get(r["name"])
            if c:
                c["pg"], c["pg_ci"], c["pg_n"], c["pg_pred"] = fnum(r["played_gain"]), fnum(r["played_ci"]), int(r["n"]), fnum(r["pred_gain"])
    for c in cards:
        c["pg_before"] = None
    if a.set_played_before:
        for r in read_tsv(a.set_played_before):
            c = by_name.get(r["name"])
            if c:
                c["pg_before"] = fnum(r["played_gain"])
    if a.issues:
        for n, v in json.load(open(a.issues)).items():
            if n in by_name:
                by_name[n]["issue"] = v

    adds = []
    for r in read_tsv(os.path.join(a.run, "adds.tsv")):
        adds.append({
            "name": r["name"],
            "swap": fnum(r["swap_gain"]) * L,
            "swap_out": r["swap_out"],
            "rebuild": fnum(r["rebuild_gain"]) * L,
            "rebuild_colors": r["rebuild_colors"],
            "rebuild_plays": r["rebuild_plays"] == "1",
            "kind": r["best_kind"],
            "best": fnum(r["best_gain"]) * L,
            "deck": r["best_deck"].split(";"),
            "rebuild_deck": r["rebuild_deck"].split(";"),
        })

    base = {r["kind"]: r for r in read_tsv(os.path.join(a.run, "base.tsv"))}

    def confirm_rows(path):
        with open(path) as f:
            head = f.readline().lstrip("# ").strip()
        return head, read_tsv(path)

    played = {}
    field_note = None
    for path in [os.path.join(a.run, "confirm.tsv")] + a.confirm:
        if not os.path.exists(path):
            continue
        head, rows = confirm_rows(path)
        if field_note is None:
            field_note = head
        elif head not in field_note:
            field_note += "; " + head
        for r in rows:
            e = played.setdefault(r["name"], {"reads": []})
            e["reads"].append({
                "share": fnum(r["share"]), "share_ci": fnum(r["share_ci"]),
                "gain": fnum(r["gain"]), "gain_ci": fnum(r["gain_ci"]),
                "w": int(r["wins"]), "l": int(r["losses"]),
            })
    # Pool independent reads: inverse-variance mean of the paired gains.
    for e in played.values():
        rs = e["reads"]
        e["share"] = sum(r["share"] for r in rs) / len(rs)
        ws = [1 / max(r["gain_ci"], 1e-9) ** 2 for r in rs if r["gain_ci"]]
        if ws and len(ws) == len(rs):
            e["gain"] = sum(w * r["gain"] for w, r in zip(ws, rs)) / sum(ws)
            e["gain_ci"] = (1 / sum(ws)) ** 0.5
        else:
            e["gain"], e["gain_ci"] = 0.0, 0.0
        e["games"] = sum(r["w"] + r["l"] for r in rs)

    for ad in adds:
        p = played.get(ad["name"])
        ad["played"] = None if p is None else {k: p[k] for k in ("share", "gain", "gain_ci", "games")}

    corr = {}
    for key, label in [("draftsim", "Draftsim rating"), ("gih", "17lands GIH WR"), ("sim", "Sim value")]:
        rho, n = spearman([c["pg"] for c in cards], [c[key] for c in cards])
        corr["pg_" + key] = {"label": "Played vs " + label, "rho": rho, "n": n}
    for key, label in [("draftsim", "Draftsim rating"), ("gih", "17lands GIH WR"), ("sim", "Sim value")]:
        rho, n = spearman([c["gain"] for c in cards], [c[key] for c in cards])
        corr[key] = {"label": "Value net vs " + label, "rho": rho, "n": n}
    rho, n = spearman([c["sim"] for c in cards], [c["draftsim"] for c in cards])
    corr["sim_vs_draftsim"] = {"label": "Sim value vs Draftsim", "rho": rho, "n": n}
    for lab, keep in [("all cards", lambda c: True), ("built in ≥ 20 sample decks", lambda c: c["played"] >= 20), ("built in < 20 (extrapolated)", lambda c: c["played"] < 20)]:
        sub = [c for c in cards if keep(c)]
        rho, n = spearman([c["pg_pred"] for c in sub], [c["pg"] for c in sub])
        corr["pred_" + lab] = {"label": "Value net prediction vs played, " + lab, "rho": rho, "n": n}
    pa = [ad for ad in adds if ad["played"] and ad["name"] in by_name]
    rho, n = spearman([ad["best"] for ad in pa], [ad["played"]["gain"] for ad in pa])
    corr["screen_vs_played"] = {"label": "Value screen vs played gain (confirmed adds)", "rho": rho, "n": n}

    meta = {
        "field": field_note,
        "your_deck": {"value": float(base["your_deck"]["value"]) * L, "colors": base["your_deck"]["colors"], "deck": base["your_deck"]["deck"].split(";")},
        "builder": {"value": float(base["builder_on_pool"]["value"]) * L, "colors": base["builder_on_pool"]["colors"], "deck": base["builder_on_pool"]["deck"].split(";")},
        "your_played": played.get("(your deck)"),
        "builder_played": played.get("(builder on your pool)"),
        "corr": corr,
        "free_played": played.get("(best sideboard swap)"),
    }
    fp = os.path.join(a.run, "free_swap.tsv")
    if os.path.exists(fp):
        r = read_tsv(fp)[0]
        meta["free"] = {"in": r["in"], "out": r["out"], "gain": float(r["gain"]) * L, "deck": r["deck"].split(";")}
    meta.update(json.loads(a.meta))
    data = {"meta": meta, "cards": cards, "adds": adds}
    html = TEMPLATE.replace("/*DATA*/null", json.dumps(data, separators=(",", ":")).replace("</", "<\\/"))
    with open(a.out, "w") as f:
        f.write(html)
    print(f"wrote {a.out}: {len(cards)} cards, {len(adds)} additions, {sum(1 for x in adds if x['played'])} played", file=sys.stderr)


TEMPLATE = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>FRA sealed: set ranking and pool additions</title>
<style>
:root {
  color-scheme: light;
  --page: #f9f9f7; --surface-1: #fcfcfb; --surface-2: #f3f2ee;
  --text-primary: #0b0b0b; --text-secondary: #52514e; --text-muted: #898781;
  --grid: #e1e0d9; --axis: #c3c2b7; --border: rgba(11,11,11,0.10);
  --series-1: #2a78d6; --series-2: #eb6834; --wash-1: rgba(42,120,214,0.10);
  --good-text: #006300; --bad-text: #b42323; --warn: #fab219;
}
@media (prefers-color-scheme: dark) {
  :root:where(:not([data-theme="light"])) {
    color-scheme: dark;
    --page: #0d0d0d; --surface-1: #1a1a19; --surface-2: #242422;
    --text-primary: #f0efec; --text-secondary: #c3c2b7; --text-muted: #898781;
    --grid: #2c2c2a; --axis: #383835; --border: rgba(255,255,255,0.10);
    --series-1: #3987e5; --series-2: #d95926; --wash-1: rgba(57,135,229,0.14);
    --good-text: #0ca30c; --bad-text: #e66767;
  }
}
:root[data-theme="dark"] {
  color-scheme: dark;
  --page: #0d0d0d; --surface-1: #1a1a19; --surface-2: #242422;
  --text-primary: #f0efec; --text-secondary: #c3c2b7; --text-muted: #898781;
  --grid: #2c2c2a; --axis: #383835; --border: rgba(255,255,255,0.10);
  --series-1: #3987e5; --series-2: #d95926; --wash-1: rgba(57,135,229,0.14);
  --good-text: #0ca30c; --bad-text: #e66767;
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--page); color: var(--text-primary);
  font: 14px/1.45 system-ui, -apple-system, "Segoe UI", sans-serif; }
header { padding: 20px 28px 0; }
h1 { font-size: 20px; margin: 0 0 4px; font-weight: 650; }
h2 { font-size: 16px; margin: 0 0 4px; font-weight: 620; }
.sub { color: var(--text-secondary); margin: 0; }
.muted { color: var(--text-muted); }
nav { display: flex; gap: 4px; padding: 14px 28px 0; border-bottom: 1px solid var(--grid); }
nav button { background: none; border: 0; padding: 8px 14px; font: inherit; color: var(--text-secondary);
  border-bottom: 2px solid transparent; cursor: pointer; }
nav button[aria-selected="true"] { color: var(--text-primary); border-bottom-color: var(--series-1); font-weight: 600; }
nav .spacer { flex: 1; }
main { padding: 20px 28px 60px; max-width: 1500px; }
.card { background: var(--surface-1); border: 1px solid var(--border); border-radius: 10px; padding: 16px 18px; margin-bottom: 18px; }
.tiles { display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 12px; margin-bottom: 18px; }
.tile { background: var(--surface-1); border: 1px solid var(--border); border-radius: 10px; padding: 12px 16px; }
.tile .k { color: var(--text-secondary); font-size: 12.5px; }
.tile .v { font-size: 26px; font-weight: 640; margin-top: 2px; }
.tile .d { color: var(--text-muted); font-size: 12.5px; }
.filters { display: flex; flex-wrap: wrap; gap: 10px 16px; align-items: center; margin-bottom: 12px; }
.filters input[type=search] { font: inherit; padding: 6px 10px; border-radius: 7px; border: 1px solid var(--axis);
  background: var(--surface-1); color: var(--text-primary); width: 220px; }
.chips { display: flex; gap: 4px; }
.chip { font: inherit; font-size: 12.5px; padding: 3px 9px; border-radius: 99px; border: 1px solid var(--axis);
  background: var(--surface-1); color: var(--text-secondary); cursor: pointer; }
.chip[aria-pressed="true"] { background: var(--text-primary); color: var(--surface-1); border-color: var(--text-primary); }
label.tog { color: var(--text-secondary); font-size: 13px; display: flex; gap: 6px; align-items: center; cursor: pointer; }
.tablewrap { overflow: auto; max-height: 72vh; border-top: 1px solid var(--grid); }
table { border-collapse: collapse; width: 100%; font-variant-numeric: tabular-nums; }
th, td { padding: 6px 10px; text-align: left; border-bottom: 1px solid var(--grid); white-space: nowrap; }
th { position: sticky; top: 0; background: var(--surface-1); font-weight: 600; font-size: 12.5px; color: var(--text-secondary);
  cursor: pointer; user-select: none; z-index: 1; }
th.num, td.num { text-align: right; }
th[aria-sort="descending"]::after { content: " \2193"; }
th[aria-sort="ascending"]::after { content: " \2191"; }
tbody tr:hover { background: var(--surface-2); }
tbody tr.sel { background: var(--wash-1); }
tbody tr { cursor: pointer; }
td.name { font-weight: 560; }
.pos { color: var(--good-text); } .neg { color: var(--bad-text); }
.pip { display: inline-flex; align-items: center; justify-content: center; width: 17px; height: 17px; border-radius: 50%;
  font-size: 10.5px; font-weight: 700; margin-right: 2px; border: 1px solid rgba(0,0,0,0.25); color: #0b0b0b; }
.pip.W { background: #f8f1d4; } .pip.U { background: #a9d3f0; } .pip.B { background: #b9b2ad; }
.pip.R { background: #f2a68c; } .pip.G { background: #9fd6a6; } .pip.C { background: #d7d5ce; }
.rar { font-size: 11.5px; font-weight: 650; padding: 1px 6px; border-radius: 4px; border: 1px solid var(--axis); color: var(--text-secondary); }
.flag { font-size: 11.5px; color: var(--text-muted); border: 1px dashed var(--axis); border-radius: 4px; padding: 0 5px; margin-left: 6px; }
.flag.issue { color: var(--bad-text); border-color: var(--bad-text); cursor: help; }
.flag.fixedflag { color: var(--good-text); border-color: var(--good-text); cursor: help; }
.badge-in { font-size: 11.5px; color: var(--text-secondary); background: var(--surface-2); border-radius: 4px; padding: 1px 6px; margin-left: 6px; }
.ci { color: var(--text-muted); font-size: 12px; }
.bar { display: inline-block; height: 8px; border-radius: 0 4px 4px 0; background: var(--series-1); vertical-align: middle; }
.bar.neg { background: var(--axis); border-radius: 4px 0 0 4px; }
.cell-bar { display: flex; align-items: center; gap: 8px; justify-content: flex-end; }
.cell-bar .track { width: 90px; display: flex; }
svg text { fill: var(--text-muted); font-size: 11.5px; }
svg .lbl { fill: var(--text-secondary); font-size: 12px; }
svg .grid { stroke: var(--grid); stroke-width: 1; }
svg .zero { stroke: var(--axis); stroke-width: 1; }
.legend { display: flex; gap: 18px; color: var(--text-secondary); font-size: 12.5px; margin: 6px 0 4px; }
.legend span { display: inline-flex; align-items: center; gap: 6px; }
#tip { position: fixed; pointer-events: none; z-index: 20; background: var(--surface-1); border: 1px solid var(--border);
  border-radius: 8px; padding: 8px 10px; box-shadow: 0 6px 24px rgba(0,0,0,0.18); display: none; max-width: 320px; font-size: 12.5px; }
#tip strong { font-size: 14px; }
#tip img { display: block; width: 250px; border-radius: 10px; margin-top: 6px; }
.detail { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 18px; }
.detail ul { margin: 6px 0 0; padding-left: 18px; }
.detail li { margin: 1px 0; }
.in { color: var(--good-text); font-weight: 600; } .out { color: var(--bad-text); font-weight: 600; text-decoration: line-through; }
.two { display: grid; grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr); gap: 18px; }
@media (max-width: 1100px) { .two, .detail { grid-template-columns: 1fr; } }
.note { color: var(--text-secondary); font-size: 13px; max-width: 900px; }
.note p { margin: 6px 0; }
code { font-size: 12.5px; background: var(--surface-2); padding: 1px 4px; border-radius: 4px; }
.hidden { display: none !important; }
#issues p, #issues ul { max-width: 1000px; color: var(--text-secondary); font-size: 13.5px; }
#issues table td { vertical-align: top; }
</style>
</head>
<body>
<header>
  <h1>Reality Fracture sealed: set ranking and best additions to your pool</h1>
  <p class="sub" id="subtitle"></p>
</header>
<nav role="tablist">
  <button role="tab" data-tab="adds" aria-selected="true">Add to my pool</button>
  <button role="tab" data-tab="set" aria-selected="false">Set ranking</button>
  <button role="tab" data-tab="issues" aria-selected="false">Issues found</button>
  <button role="tab" data-tab="method" aria-selected="false">How this was measured</button>
  <span class="spacer"></span>
  <button id="theme" title="Toggle light/dark">◐ Theme</button>
</nav>
<main>
<section id="tab-adds">
  <div class="tiles" id="tiles"></div>
  <div class="card">
    <h2>Confirmed by play: win-rate change when the card joins your pool</h2>
    <p class="sub">Each card's best deck (your deck with it swapped in, or a rebuild of pool + card) against the same field as your current deck. Gains are paired differences in win-rate points; whiskers are 95% intervals.</p>
    <div class="legend"><span><svg width="14" height="14"><circle cx="7" cy="7" r="4.5" fill="var(--series-1)"/></svg>Played gain (95% CI)</span>
      <span><svg width="14" height="14"><circle cx="7" cy="7" r="4" fill="none" stroke="var(--series-2)" stroke-width="2"/></svg>Value-net prediction</span></div>
    <div id="forest"></div>
    <p class="note" id="forest-note"></p>
  </div>
  <div class="card">
    <h2>Every FRA card as an addition</h2>
    <p class="sub">The value-net screen covers all cards. "Played" is filled in for the ones that went on to games. Click a row to see the deck it makes.</p>
    <div class="filters" id="adds-filters"></div>
    <div class="tablewrap"><table id="adds-table"></table></div>
  </div>
  <div class="card" id="adds-detail"><p class="muted">Select a card above to see the deck it makes.</p></div>
</section>
<section id="tab-set" class="hidden">
  <div class="two">
    <div class="card">
      <h2 id="scatter-title">Played rating vs value net</h2>
      <p class="sub" id="scatter-sub"></p>
      <div class="filters" id="scatter-ctl"></div>
      <div class="legend"><span><svg width="14" height="14"><circle cx="7" cy="7" r="4.5" fill="var(--series-1)"/></svg>Card</span>
        <span><svg width="14" height="14"><circle cx="7" cy="7" r="4.5" fill="var(--series-2)"/></svg>Known bot/engine issue</span>
        <span><svg width="14" height="14"><circle cx="7" cy="7" r="4" fill="none" stroke="var(--series-1)" stroke-width="1.5"/></svg>Rarely built (net extrapolates)</span></div>
      <div id="scatter"></div>
    </div>
    <div class="card">
      <h2>Agreement between rankings</h2>
      <p class="sub">Spearman rank correlation over the cards both sources rate.</p>
      <div id="corr"></div>
      <h2 style="margin-top:18px">Top 15 by played rating</h2>
      <div id="top15"></div>
    </div>
  </div>
  <div class="card">
    <h2>All FRA cards</h2>
    <p class="sub">"Played": win-rate points a deck that can cast the card gains by playing it in place of its weakest card, measured in games (up to 400 model-built sealed decks per card, each with and without the card against the same opponent and seeds). "Value net": the net's prediction of the same swaps. "Built in" counts the sample decks where the builder chose to play the card; below 3 the net is extrapolating ("rarely built"). A dashed badge marks a card with a known bot or engine issue (see "Issues found").</p>
    <div class="filters" id="set-filters"></div>
    <div class="tablewrap"><table id="set-table"></table></div>
  </div>
</section>
<section id="tab-issues" class="hidden">
  <div class="card" id="issues"></div>
</section>
<section id="tab-method" class="hidden">
  <div class="card note" id="method"></div>
</section>
</main>
<div id="tip" role="tooltip"></div>
<script>
const DATA = /*DATA*/null;
const M = DATA.meta;
const byName = Object.fromEntries(DATA.cards.map(c => [c.name, c]));
const $ = s => document.querySelector(s);
const el = (tag, attrs = {}, ...kids) => {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (k === "class") e.className = v; else if (k === "text") e.textContent = v;
    else if (k.startsWith("on")) e.addEventListener(k.slice(2), v); else e.setAttribute(k, v);
  }
  for (const k of kids) if (k != null) e.append(k);
  return e;
};
const svgEl = (tag, attrs = {}) => {
  const e = document.createElementNS("http://www.w3.org/2000/svg", tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  return e;
};
const fmt = (v, d = 1, sign = true) => v == null ? "–" : (sign && v > 0 ? "+" : "") + v.toFixed(d);
const cls = v => v == null ? "" : v > 0 ? "pos" : v < 0 ? "neg" : "";
const isFixed = i => i && i.status && i.status.startsWith("fixed");
const issueBadge = r => r.issue ? el("span", { class: "flag " + (isFixed(r.issue) ? "fixedflag" : "issue"), text: r.issue.status ? `${r.issue.status}: ${r.issue.kind}` : r.issue.kind, title: r.issue.text }) : null;

// ---------- theme ----------
$("#theme").onclick = () => {
  const r = document.documentElement;
  const dark = r.dataset.theme ? r.dataset.theme === "dark" : matchMedia("(prefers-color-scheme: dark)").matches;
  r.dataset.theme = dark ? "light" : "dark";
};

// ---------- tabs ----------
function selectTab(name) {
  document.querySelectorAll("nav [role=tab]").forEach(x => x.setAttribute("aria-selected", x.dataset.tab === name));
  for (const t of ["adds", "set", "issues", "method"]) $("#tab-" + t).classList.toggle("hidden", t !== name);
  if (name === "set") scatter(); else if (name === "adds") forest();
}
document.querySelectorAll("nav [role=tab]").forEach(b => b.onclick = () => { history.replaceState(null, "", "#" + b.dataset.tab); selectTab(b.dataset.tab); });

// ---------- tooltip ----------
const tip = $("#tip");
function showTip(ev, card, lines) {
  tip.replaceChildren();
  tip.append(el("strong", { text: card ? card.name : lines.shift() }));
  for (const l of lines) tip.append(el("div", { text: l }));
  if (card && card.img) tip.append(el("img", { src: card.img, alt: "" }));
  tip.style.display = "block";
  moveTip(ev);
}
function moveTip(ev) {
  const w = tip.offsetWidth, h = tip.offsetHeight;
  let x = ev.clientX + 16, y = ev.clientY + 12;
  if (x + w > innerWidth - 8) x = ev.clientX - w - 16;
  if (y + h > innerHeight - 8) y = Math.max(8, innerHeight - h - 8);
  tip.style.left = x + "px"; tip.style.top = y + "px";
}
const hideTip = () => tip.style.display = "none";
function hoverCard(node, name, extra = () => []) {
  node.addEventListener("pointerenter", ev => showTip(ev, byName[name] || { name }, extra()));
  node.addEventListener("pointermove", moveTip);
  node.addEventListener("pointerleave", hideTip);
}

function pips(colors) {
  const s = el("span");
  if (!colors) s.append(el("span", { class: "pip C", text: "C", title: "colorless" }));
  for (const c of colors) s.append(el("span", { class: "pip " + c, text: c }));
  return s;
}

// ---------- generic sortable / filterable table ----------
function makeTable(table, cols, rows, opts) {
  let sortKey = opts.sort, dir = -1;
  const thead = el("thead"), tbody = el("tbody");
  const tr = el("tr");
  for (const c of cols) {
    const th = el("th", { class: c.num ? "num" : "", text: c.label, title: c.title || "" });
    th.onclick = () => { if (sortKey === c.key) dir = -dir; else { sortKey = c.key; dir = c.num ? -1 : 1; } render(); };
    c.th = th; tr.append(th);
  }
  thead.append(tr); table.replaceChildren(thead, tbody);
  function render() {
    const keep = rows.filter(opts.filter);
    const col = cols.find(c => c.key === sortKey);
    const val = r => col.sortv ? col.sortv(r) : r[sortKey];
    keep.sort((a, b) => {
      const x = val(a), y = val(b);
      if (x == null && y == null) return 0; if (x == null) return 1; if (y == null) return -1;
      return (x < y ? -1 : x > y ? 1 : 0) * dir;
    });
    for (const c of cols) c.th.setAttribute("aria-sort", c.key === sortKey ? (dir < 0 ? "descending" : "ascending") : "none");
    tbody.replaceChildren(...keep.map((r, i) => {
      const row = el("tr");
      for (const c of cols) {
        const td = el("td", { class: (c.num ? "num " : "") + (c.cls || "") });
        const v = c.cell(r, i);
        if (v instanceof Node) td.append(v); else td.textContent = v;
        row.append(td);
      }
      if (opts.hover) hoverCard(row, r.name, () => opts.hover(r));
      if (opts.click) row.onclick = () => { tbody.querySelectorAll("tr.sel").forEach(x => x.classList.remove("sel")); row.classList.add("sel"); opts.click(r); };
      return row;
    }));
  }
  render();
  return render;
}

function filterBar(host, state, onchange, extra = []) {
  const q = el("input", { type: "search", placeholder: "Search card name…" });
  q.oninput = () => { state.q = q.value.toLowerCase(); onchange(); };
  const mk = (vals, key, title) => {
    const box = el("div", { class: "chips", "aria-label": title });
    for (const v of vals) {
      const b = el("button", { class: "chip", "aria-pressed": "false", text: v, title: title + " " + v });
      b.onclick = () => {
        const on = b.getAttribute("aria-pressed") !== "true";
        b.setAttribute("aria-pressed", on);
        on ? state[key].add(v) : state[key].delete(v); onchange();
      };
      box.append(b);
    }
    return box;
  };
  host.append(q, mk(["W", "U", "B", "R", "G", "C", "Multi"], "colors", "Color"), mk(["C", "U", "R", "M"], "rarity", "Rarity"), ...extra);
}
function baseFilter(state, c) {
  if (state.q && !c.name.toLowerCase().includes(state.q)) return false;
  if (state.rarity.size && !state.rarity.has(c.rarity)) return false;
  if (state.colors.size) {
    const cs = c.colors || "C";
    const ok = [...state.colors].some(f => f === "Multi" ? cs.length > 1 : f === "C" ? !c.colors : cs.includes(f));
    if (!ok) return false;
  }
  return true;
}
function toggle(label, state, key, onchange, init = false) {
  const cb = el("input", { type: "checkbox" }); cb.checked = init; state[key] = init;
  cb.onchange = () => { state[key] = cb.checked; onchange(); };
  return el("label", { class: "tog" }, cb, label);
}

// ---------- header ----------
const yp = M.your_played;
$("#subtitle").textContent = `Pool: ${M.pool_name || "fra_sealed.txt"} · your deck: ${M.deck_name || ""} (${M.your_deck.colors}) · pilot ${M.pilot || "fra-fixes"} · ${M.field || ""} · generated ${M.date || ""}`;

// ---------- tab 1: additions ----------
const IN_DECK = counts(M.your_deck.deck);
const owned = a => a.in_pool ? (IN_DECK.get(a.name) ? "2nd copy" : "in sideboard") : "";
const adds = DATA.adds.map(a => ({ ...a, ...pickCard(a.name) }));
function pickCard(n) { const c = byName[n] || {}; return { rarity: c.rarity, colors: c.colors, cost: c.cost, in_pool: c.in_pool, card: c }; }
const playedAdds = adds.filter(a => a.played).sort((a, b) => b.played.gain - a.played.gain);
{
  const best = playedAdds[0];
  const sig = playedAdds.filter(a => a.played.gain - a.played.gain_ci > 0);
  const beatFree = M.free_played ? playedAdds.filter(a => a.played.gain > M.free_played.gain) : null;
  const tiles = [
    ["Your deck vs the field", yp ? yp.share.toFixed(1) + "%" : "–", yp ? `${yp.games.toLocaleString()} games, ${M.your_deck.colors}` : ""],
    ["Best confirmed addition", best ? fmt(best.played.gain) + " pts" : "–", best ? best.name : ""],
    ["Additions clearly above zero", `${sig.length} of ${playedAdds.length}`, "played gain's 95% interval above 0" + (beatFree ? `; ${beatFree.length} beat the sideboard swap` : "")],
    ["Cards screened by the value net", String(adds.length), "every card on the FRA sheet"],
  ];
  if (M.free) tiles.splice(1, 0, ["Best swap from your own sideboard", M.free_played ? fmt(M.free_played.gain) + " pts" : fmt(M.free.gain) + " (net)", `${M.free.in} for ${M.free.out}` + (M.free_played ? `, ±${M.free_played.gain_ci.toFixed(1)} played` : "")]);
  if (M.builder_played) tiles.splice(1, 0, ["Model's own build of your pool", M.builder_played.share.toFixed(1) + "%", `${fmt(M.builder_played.gain)} vs your deck (${M.builder.colors})`]);
  $("#tiles").replaceChildren(...tiles.map(([k, v, d]) => el("div", { class: "tile" }, el("div", { class: "k", text: k }), el("div", { class: "v", text: v }), el("div", { class: "d", text: d }))));
}

function forest() {
  const host = $("#forest");
  const rows = playedAdds;
  if (!rows.length) { host.textContent = "No played confirmation in this run."; return; }
  const W = Math.max(640, host.clientWidth || 900), rowH = 22, top = 44, left = 250, right = 70;
  const H = top + rows.length * rowH + 28;
  const vals = rows.flatMap(r => [r.played.gain - r.played.gain_ci, r.played.gain + r.played.gain_ci, r.best]).concat(M.free_played ? [M.free_played.gain] : []);
  let lo = Math.min(0, ...vals), hi = Math.max(0, ...vals);
  const pad = (hi - lo) * 0.05; lo -= pad; hi += pad;
  const step = niceStep((hi - lo) / 8);
  const x = v => left + (v - lo) / (hi - lo) * (W - left - right);
  const svg = svgEl("svg", { width: W, height: H, viewBox: `0 0 ${W} ${H}`, role: "img", "aria-label": "Played gain per added card with 95% intervals" });
  for (let t = Math.ceil(lo / step) * step; t <= hi + 1e-9; t += step) {
    const z = Math.abs(t) < 1e-9;
    svg.append(svgEl("line", { x1: x(t), x2: x(t), y1: top - 6, y2: H - 22, stroke: z ? "var(--text-secondary)" : "var(--grid)", "stroke-width": 1 }));
    for (const ty of [top - 10, H - 6]) {
      const tx = svgEl("text", { x: x(t), y: ty, "text-anchor": "middle" }); tx.textContent = (t > 0 ? "+" : "") + t.toFixed(step < 1 ? 1 : 0); svg.append(tx);
    }
  }
  const zl = svgEl("text", { x: x(0) - 4, y: 12, "text-anchor": "end", class: "lbl" }); zl.textContent = "your deck (0)"; svg.append(zl);
  const head = svgEl("text", { x: W - right + 8, y: top - 10 }); head.textContent = "played"; svg.append(head);
  if (M.free_played) {
    const fx = x(M.free_played.gain);
    svg.append(svgEl("line", { x1: fx, x2: fx, y1: 16, y2: H - 22, stroke: "var(--axis)", "stroke-width": 1 }));
    const fr = fx > x(0);
    const ft = svgEl("text", { x: fx + (fr ? 4 : -4), y: 24, "text-anchor": fr ? "start" : "end" }); ft.textContent = `best sideboard swap ${fmt(M.free_played.gain)}`; svg.append(ft);
  }
  rows.forEach((r, i) => {
    const y = top + i * rowH + rowH / 2;
    const g = svgEl("g", { tabindex: 0 });
    g.append(svgEl("rect", { x: 0, y: y - rowH / 2, width: W, height: rowH, fill: "transparent" }));
    const name = svgEl("text", { x: left - 10, y: y + 4, "text-anchor": "end", class: "lbl" });
    name.textContent = r.name + (r.in_pool ? ` (${owned(r)})` : ""); g.append(name);
    g.append(svgEl("line", { x1: x(r.played.gain - r.played.gain_ci), x2: x(r.played.gain + r.played.gain_ci), y1: y, y2: y, stroke: "var(--series-1)", "stroke-width": 2, "stroke-linecap": "round" }));
    g.append(svgEl("circle", { cx: x(r.best), cy: y, r: 4, fill: "none", stroke: "var(--series-2)", "stroke-width": 2 }));
    g.append(svgEl("circle", { cx: x(r.played.gain), cy: y, r: 5, fill: "var(--series-1)", stroke: "var(--surface-1)", "stroke-width": 2 }));
    const v = svgEl("text", { x: W - right + 8, y: y + 4, class: "lbl" }); v.textContent = fmt(r.played.gain); g.append(v);
    hoverCard(g, r.name, () => [
      `Played: ${fmt(r.played.gain)} ± ${r.played.gain_ci.toFixed(1)} pts (${r.played.share.toFixed(1)}% vs field)`,
      `Value net predicted: ${fmt(r.best)} pts`,
      r.kind === "swap" ? `Swap in for ${r.swap_out}` : `Rebuild of pool + card (${r.rebuild_colors})`,
    ]);
    g.addEventListener("click", () => detail(r));
    svg.append(g);
  });
  host.replaceChildren(svg);
}
{
  const outs = counts(playedAdds.filter(a => a.kind === "swap").map(a => a.swap_out));
  const [top, n] = [...outs].sort((a, b) => b[1] - a[1])[0] || [];
  if (top) $("#forest-note").textContent = `${n} of the ${playedAdds.length} played additions go in as a swap for ${top}, the slot the value net likes least in your deck. Weak cards that take the same slot (e.g. a second Campus Crier) gain only about 0 to +1, so dropping it is worth little on its own, and most of each gain above is the new card's own value.`;
}
function niceStep(raw) { const p = Math.pow(10, Math.floor(Math.log10(raw))); for (const m of [1, 2, 2.5, 5, 10]) if (m * p >= raw) return m * p; return 10 * p; }

const addState = { q: "", colors: new Set(), rarity: new Set() };
const ADD_MAX = Math.max(...DATA.adds.map(a => a.best));
const SET_MAX = Math.max(...DATA.cards.filter(c => c.gain != null).map(c => c.gain));
const addCols = [
  { key: "best", label: "#", num: true, cell: (r, i) => String(i + 1), sortv: r => r.best },
  { key: "name", label: "Card", cls: "name", cell: r => { const s = el("span", { text: r.name }); if (r.in_pool) s.append(el("span", { class: "badge-in", text: owned(r), title: IN_DECK.get(r.name) ? "You already play this; the row is a second copy" : "You already own this in your sideboard: no new card needed" })); const ib = r.card && r.card.issue ? issueBadge(r.card) : null; if (ib) s.append(ib); return s; } },
  { key: "rarity", label: "Rarity", cell: r => el("span", { class: "rar", text: r.rarity || "" }) },
  { key: "colors", label: "Color", cell: r => pips(r.colors || "") },
  { key: "cost", label: "Cost", cell: r => r.cost || "" },
  { key: "best", label: "Value net (pts)", num: true, title: "Predicted win-rate points over your deck, best of swap-in / rebuild", cell: r => barCell(r.best, ADD_MAX) },
  { key: "kind", label: "How it goes in", cell: r => r.kind === "swap" ? `swap for ${r.swap_out}` : `rebuild (${r.rebuild_colors})` },
  { key: "swap", label: "Swap-in", num: true, title: "Your deck with this card at its best slot", cell: r => span(fmt(r.swap), cls(r.swap)) },
  { key: "rebuild", label: "Rebuild", num: true, title: "The model's build of pool + card, vs your deck", cell: r => span(fmt(r.rebuild) + (r.rebuild_plays ? "" : "*"), cls(r.rebuild)) },
  { key: "pg", label: "Played (pts)", num: true, sortv: r => r.played ? r.played.gain : null, title: "Paired win-rate change against the field, 95% CI", cell: r => r.played ? playedCell(r.played) : span("–", "muted") },
];
function span(t, c) { return el("span", { class: c || "", text: t }); }
function playedCell(p) { const s = el("span", { class: cls(p.gain - p.gain_ci > 0 ? 1 : p.gain + p.gain_ci < 0 ? -1 : 0), text: fmt(p.gain) }); return el("span", {}, s, el("span", { class: "ci", text: " ±" + p.gain_ci.toFixed(1) })); }
function barCell(v, scale) {
  const w = Math.min(90, Math.max(0, v) / scale * 90);
  const track = el("span", { class: "track" });
  const b = el("span", { class: "bar" + (v < 0 ? " neg" : ""), style: `width:${w.toFixed(0)}px` });
  track.append(b);
  return el("span", { class: "cell-bar" }, track, span(fmt(v), cls(v)));
}
const renderAdds = makeTable($("#adds-table"), addCols, adds, {
  sort: "best",
  filter: r => baseFilter(addState, r) && (!addState.played || r.played) && (!addState.fresh || owned(r) !== "in sideboard"),
  hover: r => [`${r.cost || ""} · ${r.rarity || ""}`, `Value net ${fmt(r.best)} pts${r.played ? `, played ${fmt(r.played.gain)} ± ${r.played.gain_ci.toFixed(1)}` : ""}`],
  click: r => detail(r),
});
filterBar($("#adds-filters"), addState, () => renderAdds(), [
  toggle("Played only", addState, "played", () => renderAdds()),
  toggle("Hide sideboard cards (free swaps)", addState, "fresh", () => renderAdds()),
]);

function counts(list) { const m = new Map(); for (const n of list) m.set(n, (m.get(n) || 0) + 1); return m; }
function detail(r) {
  const host = $("#adds-detail");
  const mine = counts(M.your_deck.deck), theirs = counts(r.deck);
  const plus = [], minus = [];
  for (const [n, k] of theirs) { const d = k - (mine.get(n) || 0); if (d > 0) plus.push([n, d]); }
  for (const [n, k] of mine) { const d = k - (theirs.get(n) || 0); if (d > 0) minus.push([n, d]); }
  const list = (items, cl) => el("ul", {}, ...items.map(([n, k]) => { const li = el("li", { class: cl, text: `${k} ${n}` }); hoverCard(li, n); return li; }));
  const full = [...theirs].sort((a, b) => ((byName[a[0]] || {}).cmc ?? 99) - ((byName[b[0]] || {}).cmc ?? 99));
  const c = byName[r.name] || {};
  host.replaceChildren(
    el("h2", { text: `${r.name}: the deck it makes` }),
    el("p", { class: "sub", text: (r.kind === "swap" ? `Your deck with ${r.name} in place of ${r.swap_out}.` : `The model's rebuild of your pool plus ${r.name} (${r.rebuild_colors}).`) +
      ` Value net ${fmt(r.best)} pts` + (r.played ? `; played ${fmt(r.played.gain)} ± ${r.played.gain_ci.toFixed(1)} pts (${r.played.share.toFixed(1)}% vs the field, ${r.played.games.toLocaleString()} games).` : "; not played.") }),
    el("div", { class: "detail" },
      el("div", {}, el("strong", { text: "Changes vs your deck" }), list(plus, "in"), list(minus, "out")),
      el("div", {}, el("strong", { text: `Full list (${r.deck.length})` }), list(full, "")),
      el("div", {}, el("strong", { text: "Card" }), c.img ? el("img", { src: c.img, alt: r.name, style: "width:250px;border-radius:12px;margin-top:6px;display:block" }) : span(" (no image)", "muted"),
        el("div", { class: "muted", text: `Set rating: played ${fmt(c.pg)} · value net ${fmt(c.gain)} · sim ${fmt(c.sim, 2)} · Draftsim ${c.draftsim ?? "–"} · 17lands GIH ${c.gih ? c.gih.toFixed(1) + "%" : "–"}` }))));
  host.scrollIntoView({ behavior: "smooth", block: "nearest" });
}

// ---------- tab 2: set ranking ----------
const LOW = 3;
const cards = DATA.cards.map(c => ({ ...c, support: c.played }));
const PG_MAX = Math.max(...cards.filter(c => c.pg != null).map(c => c.pg));

const setState = { q: "", colors: new Set(), rarity: new Set() };
const setCols = [
  { key: "pg", label: "#", num: true, cell: (r, i) => String(i + 1) },
  { key: "name", label: "Card", cls: "name", cell: r => { const s = el("span", { text: r.name }); if (r.in_pool) s.append(el("span", { class: "badge-in", text: "in pool ×" + r.in_pool })); if (r.support != null && r.support < LOW) s.append(el("span", { class: "flag", text: "rarely built", title: "The builder played it in fewer than 3 of 1,000 sample decks, so the net is extrapolating" })); const ib = issueBadge(r); if (ib) s.append(ib); return s; } },
  { key: "rarity", label: "Rarity", cell: r => el("span", { class: "rar", text: r.rarity }) },
  { key: "colors", label: "Color", cell: r => pips(r.colors) },
  { key: "cost", label: "Cost", cell: r => r.cost },
  { key: "types", label: "Type", cell: r => r.types },
  { key: "pg", label: "Played (pts)", num: true, title: "Measured in games: gain over the weakest slot, mean over up to 400 decks that can cast it, 95% CI", cell: r => r.pg == null ? "–" : el("span", { class: "cell-bar" }, barCell(r.pg, PG_MAX), span(" ±" + r.pg_ci.toFixed(1), "ci")) },
  { key: "pg_before", label: "Before fixes", num: true, title: "The played rating before the 2026-11-22 bot/engine fix batch", cell: r => r.pg_before == null ? "–" : span(fmt(r.pg_before), "muted") },
  { key: "gain", label: "Value net (pts)", num: true, title: "The net's prediction: gain over the weakest slot, mean over decks that can cast it", cell: r => r.gain == null ? "–" : span(fmt(r.gain), cls(r.gain)) },
  { key: "support", label: "Built in", num: true, title: "Sample decks (of those that could cast it) the builder actually played it in", cell: r => `${r.played} of ${r.fit}` },
  { key: "sim", label: "Sim value", num: true, title: "FRA_SIM_VALUE: win-rate points per copy, fitted on simulated sealed games", cell: r => span(fmt(r.sim, 2), cls(r.sim)) },
  { key: "draftsim", label: "Draftsim", num: true, cell: r => r.draftsim == null ? "–" : r.draftsim.toFixed(1) },
  { key: "gih", label: "17lands GIH", num: true, cell: r => r.gih == null ? "–" : `${r.gih.toFixed(1)}%` },
];
const renderSet = makeTable($("#set-table"), setCols, cards, {
  sort: "pg",
  filter: r => baseFilter(setState, r) && (!setState.hideLow || !(r.support != null && r.support < LOW)),
  hover: r => [`${r.cost} · ${r.types} · ${r.rarity}`, `Played ${fmt(r.pg)} ± ${(r.pg_ci || 0).toFixed(1)} pts (${r.pg_n || 0} decks) · value net ${fmt(r.gain)}`, ...(r.issue ? ["⚠ " + r.issue.kind + ": " + r.issue.text] : []), `Sim ${fmt(r.sim, 2)} · Draftsim ${r.draftsim ?? "–"} · GIH ${r.gih ? r.gih.toFixed(1) + "%" : "–"}`],
});
filterBar($("#set-filters"), setState, () => { renderSet(); scatter(); }, [toggle("Hide rarely-built cards", setState, "hideLow", () => { renderSet(); scatter(); })]);

const corrHost = $("#corr");
const corrLabel = (k, c) => c.label;
corrHost.append(el("table", {}, el("tbody", {}, ...Object.entries(M.corr).map(([k, c]) => el("tr", {}, el("td", { text: corrLabel(k, c) }), el("td", { class: "num", text: c.rho == null ? "–" : c.rho.toFixed(2) }), el("td", { class: "num muted", text: `n = ${c.n}` }))))));
{
  const top = [...cards].filter(c => c.pg != null).sort((a, b) => b.pg - a.pg).slice(0, 15);
  $("#top15").append(el("table", {}, el("tbody", {}, ...top.map((c, i) => { const tr = el("tr", {}, el("td", { class: "num muted", text: i + 1 }), el("td", { class: "name", text: c.name }), el("td", {}, pips(c.colors)), el("td", {}, el("span", { class: "rar", text: c.rarity })), el("td", { class: "num", text: fmt(c.pg) + " ±" + c.pg_ci.toFixed(1) })); hoverCard(tr, c.name); return tr; }))));
}

const scatterAxes = { gain: "Value net prediction (pts)", draftsim: "Draftsim rating (0–5)", gih: "17lands GIH win rate (%)", sim: "Sim value (pts per copy)", pg: "Played rating (pts)" };
let scatterX = "gain", scatterY = "pg";
{
  const mk = (keys, get, set, pre) => {
    const box = el("div", { class: "chips" });
    for (const k of keys) {
      const b = el("button", { class: "chip", "aria-pressed": k === get(), text: pre + scatterAxes[k].split(" (")[0] });
      b.onclick = () => { set(k); box.querySelectorAll("button").forEach(x => x.setAttribute("aria-pressed", x === b)); scatter(); };
      box.append(b);
    }
    return box;
  };
  $("#scatter-ctl").append(mk(["gain", "draftsim", "gih", "sim"], () => scatterX, k => scatterX = k, "x: "), mk(["pg", "gain"], () => scatterY, k => scatterY = k, "y: "));
}
function scatter() {
  const host = $("#scatter");
  const pts = cards.filter(c => c[scatterY] != null && c[scatterX] != null && baseFilter(setState, c) && !(setState.hideLow && c.support < LOW));
  const nm = k => scatterAxes[k].split(" (")[0];
  $("#scatter-title").textContent = `${nm(scatterY)} vs ${nm(scatterX)}`;
  const rho = (() => { const xs = pts.map(p => p[scatterX]), ys = pts.map(p => p[scatterY]); const rk = v => { const o = v.map((x, i) => [x, i]).sort((a, b) => a[0] - b[0]); const r = []; o.forEach(([, i], k) => r[i] = k); return r; }; const a = rk(xs), b = rk(ys), n = a.length, ma = (n - 1) / 2; let c = 0, va = 0, vb = 0; for (let i = 0; i < n; i++) { c += (a[i] - ma) * (b[i] - ma); va += (a[i] - ma) ** 2; vb += (b[i] - ma) ** 2; } return n > 2 ? c / Math.sqrt(va * vb) : null; })();
  $("#scatter-sub").textContent = `${pts.length} cards shown · Spearman ρ = ${rho == null ? "–" : rho.toFixed(2)}.` + (scatterX === "gain" && scatterY === "pg" ? " Points on the diagonal were predicted exactly." : "");
  const W = Math.max(520, host.clientWidth || 700), H = 420, l = 52, rgt = 16, t = 12, b = 40;
  const xs = pts.map(p => p[scatterX]), ys = pts.map(p => p[scatterY]);
  if (!pts.length) { host.textContent = "No cards match."; return; }
  let [x0, x1] = [Math.min(...xs), Math.max(...xs)], [y0, y1] = [Math.min(0, ...ys), Math.max(...ys)];
  if (scatterX === "gain" && scatterY === "pg") { x0 = y0 = Math.min(x0, y0); x1 = y1 = Math.max(x1, y1); }
  const px = (x1 - x0) * 0.04 || 1, py = (y1 - y0) * 0.05 || 1; x0 -= px; x1 += px; y0 -= py; y1 += py;
  const X = v => l + (v - x0) / (x1 - x0) * (W - l - rgt), Y = v => t + (1 - (v - y0) / (y1 - y0)) * (H - t - b);
  const svg = svgEl("svg", { width: W, height: H, viewBox: `0 0 ${W} ${H}`, role: "img", "aria-label": "Scatter of two card ratings" });
  const xs_ = niceStep((x1 - x0) / 6), ys_ = niceStep((y1 - y0) / 6);
  for (let v = Math.ceil(y0 / ys_) * ys_; v <= y1; v += ys_) {
    svg.append(svgEl("line", { x1: l, x2: W - rgt, y1: Y(v), y2: Y(v), class: Math.abs(v) < 1e-9 ? "zero" : "grid" }));
    const tx = svgEl("text", { x: l - 6, y: Y(v) + 4, "text-anchor": "end" }); tx.textContent = (v > 0 ? "+" : "") + v.toFixed(ys_ < 1 ? 1 : 0); svg.append(tx);
  }
  for (let v = Math.ceil(x0 / xs_) * xs_; v <= x1; v += xs_) {
    svg.append(svgEl("line", { x1: X(v), x2: X(v), y1: t, y2: H - b, class: "grid" }));
    const tx = svgEl("text", { x: X(v), y: H - b + 16, "text-anchor": "middle" }); tx.textContent = v.toFixed(xs_ < 1 ? 1 : 0); svg.append(tx);
  }
  if (scatterX === "gain" && scatterY === "pg") svg.append(svgEl("line", { x1: X(Math.max(x0, y0)), y1: Y(Math.max(x0, y0)), x2: X(Math.min(x1, y1)), y2: Y(Math.min(x1, y1)), stroke: "var(--text-muted)", "stroke-width": 1 }));
  const xl = svgEl("text", { x: (l + W - rgt) / 2, y: H - 6, "text-anchor": "middle", class: "lbl" }); xl.textContent = scatterAxes[scatterX]; svg.append(xl);
  const yl = svgEl("text", { x: 12, y: (t + H - b) / 2, transform: `rotate(-90 12 ${(t + H - b) / 2})`, "text-anchor": "middle", class: "lbl" }); yl.textContent = scatterAxes[scatterY]; svg.append(yl);
  // label the largest misses when comparing prediction to play, else the top five
  const lab = scatterX === "gain" && scatterY === "pg"
    ? [...pts].sort((a, b) => Math.abs(b.pg - b.gain) - Math.abs(a.pg - a.gain)).slice(0, 6)
    : [...pts].sort((a, b) => b[scatterY] - a[scatterY]).slice(0, 5);
  const labelled = new Set(lab.map(p => p.name));
  const placed = [];
  const draw = p => {
    const g = svgEl("g");
    const low = p.support != null && p.support < LOW;
    const cx = X(p[scatterX]), cy = Y(p[scatterY]);
    const col = p.issue ? "var(--series-2)" : "var(--series-1)";
    g.append(svgEl("circle", { cx, cy, r: 12, fill: "transparent" }));
    g.append(svgEl("circle", { cx, cy, r: 4, fill: low ? "var(--surface-1)" : col, stroke: low ? col : "var(--surface-1)", "stroke-width": low ? 1.5 : 2 }));
    if (labelled.has(p.name)) {
      const right = cx > W * 0.7, ly = cy + 4;
      if (!placed.some(q => Math.abs(q[1] - ly) < 14 && Math.abs(q[0] - cx) < 160)) {
        placed.push([cx, ly]);
        const tx = svgEl("text", { x: cx + (right ? -9 : 9), y: ly, "text-anchor": right ? "end" : "start", class: "lbl" });
        tx.textContent = p.name; g.append(tx);
      }
    }
    hoverCard(g, p.name, () => [`${nm(scatterY)} ${fmt(p[scatterY])} · ${nm(scatterX)} ${p[scatterX].toFixed(scatterX === "draftsim" ? 1 : 2)}`, ...(p.issue ? ["⚠ " + p.issue.kind] : []), ...(low ? ["rarely built: the net is extrapolating"] : [])]);
    svg.append(g);
  };
  pts.filter(p => !p.issue).forEach(draw);
  pts.filter(p => p.issue).forEach(draw);
  host.replaceChildren(svg);
}

function renderIssues() {
  const rows = cards.filter(c => c.issue).sort((a, b) => (a.pg ?? 0) - (b.pg ?? 0));
  const rank = Object.fromEntries([...cards].filter(c => c.pg != null).sort((a, b) => b.pg - a.pg).map((c, i) => [c.name, i + 1]));
  const n = Object.keys(rank).length;
  const t = el("table", {}, el("thead", {}, el("tr", {}, ...["Card", "Status", "Before", "After", "Draftsim", "What happens"].map(h => el("th", { text: h, style: "position:static" })))),
    el("tbody", {}, ...rows.map(c => { const tr = el("tr", {}, el("td", { class: "name", text: c.name }), el("td", {}, issueBadge(c)),
      el("td", { class: "num muted", text: fmt(c.pg_before) }), el("td", { class: "num", text: c.pg == null ? "–" : `${fmt(c.pg)} (#${rank[c.name]}/${n})` }), el("td", { class: "num", text: c.draftsim ?? "–" }),
      el("td", { text: c.issue.text, style: "white-space:normal;min-width:420px" })); hoverCard(tr.firstChild, c.name); return tr; })));
  $("#issue-list").replaceChildren(t);
}
// ---------- method ----------
$("#method").innerHTML = METHOD_HTML;
$("#issues").innerHTML = ISSUES_HTML;

selectTab(["set", "issues", "method"].includes(location.hash.slice(1)) ? location.hash.slice(1) : "adds");
addEventListener("resize", () => { forest(); scatter(); });
</script>
</body>
</html>
"""

METHOD = r"""
<h2>What the numbers are</h2>
<p><b>The model.</b> The ratings come from the engine's learned <i>same-pool deck value</i> (<code>nets/deck-value-fra.json</code>), the network that picks the deck for every FRA sealed bot opponent. It reads a whole 40-card deck: a weight per card, a card-by-context interaction term, and the deck's curve and mana-base features (castability per spell, starved spells, off-colour lands). Its output is a game logit, the log-odds that this deck beats another deck from the same pool. It was trained on about 240k played comparisons between candidate decks built from the same pool, so it is built to answer "which of these two decks is better", which is the question asked here. The page shows its output × 25 as win-rate points (the logistic's slope at an even match).</p>
<p><b>Set ranking ("Played").</b> The same sample decks, measured in games: for each card, up to 400 decks whose lands can cast it get the card in the slot the value net picks, and each deck plays one field opponent with and without the card, 4 seat-swapped pairs, same seeds (<code>fra_card_eval --set-played</code>, about 640,000 games). The rating is the mean paired difference in win-rate points with a 95 % interval (median ±1.2). This is the ranking to read; the value net column is its prediction.</p>
<p><b>Value net column.</b> The model's builder (policy proposes, value net picks) built decks from 1,000 random FRA sealed pools. For each card and each deck whose lands can cast it, the card replaces whichever slot the value net minds losing most (a spell for a spell, a basic for a land), and the value change is recorded. The rating is the mean over those decks: how much a typical deck that can play the card improves by playing it in place of its weakest card. <i>Built in</i> counts the sample decks where the builder chose to play the card. Below 3 of 1,000, the net has hardly seen the card in decks and is extrapolating, so those rows are flagged "rarely built" (mostly rares and mythics the builder never picks).</p>
<p><b>Sim value</b> is <code>FRA_SIM_VALUE</code>, a per-card win-rate-points fit (ridge) over hundreds of thousands of simulated sealed games. It is additive, so it does not see context or mana. <b>Draftsim</b> is Draftsim's FRA pick order (0–5). <b>17lands GIH</b> is the games-in-hand win rate from 17lands Premier Draft, snapshot 2026-10-23 (early data: 139 cards, at most about 7k games each; 17lands' public endpoint now returns no win rates).</p>
<h2>Pool additions</h2>
<p>For every card on the FRA sheet (a second copy if you already have one), two decks are tried: (1) <b>swap-in</b>: your current deck with the card in its best slot; (2) <b>rebuild</b>: the model's builder on your pool plus the card, which may change colours. The value net scores both against your deck, and the better one is the card's "Value net" figure and its deck.</p>
<p>The top 50 by that screen were then <b>played</b>, on two independent fields (seeds 43 and 97, 1,000 opponents × 4 seat-swapped pairs each, so 16,000 games per deck). The two reads agree closely and are pooled. Each candidate's deck, your deck, the model's own build of your pool and the best swap from your own sideboard (the control: what re-tuning buys without a new card) played the same fixed field of FRA sealed opponents (pools drawn by real FRA Sealed colour-pair shares, built by the rated builder) in seat-swapped pairs under the <code>fra-fixes</code> pilot. "Played" is the paired difference from your deck, averaged per opponent, with a 95 % interval. Treat a gain whose interval crosses zero as "no detectable change". The screen picks its own top, so expect played gains to be smaller than predicted (regression to the mean).</p>
<h2>Caveats</h2>
<ul>
<li>The pilot is the engine's bot, not a human. Cards whose value comes from decisions the bot handles badly (complex activated abilities, tricks held for combat) are under-rated by every simulated number here.</li>
<li>The confirmation plays the card's best deck <i>as the screen built it</i>. A hand-tuned version of that deck could do better, especially for rebuilds into new colours.</li>
<li>"Extended Absencem" in the pool file was read as Extended Absence.</li>
</ul>
<p class="muted">Generated by <code>fra_card_eval --pool-adds</code> + <code>scripts/fra_pool_adds_report.py</code>.</p>
"""

TEMPLATE = TEMPLATE.replace("$(\"#method\").innerHTML = METHOD_HTML;", "$(\"#method\").innerHTML = " + json.dumps(METHOD) + ";")
ISSUES = r'''
<h2>Investigation, 2026-11-22: why the value net and the online pick orders disagree with play</h2>
<p><b>1. The value net is accurate where it has seen the card, and guesses where it hasn't.</b> Every card's prediction was checked against a play measurement of the same swaps (79,000 deck pairs, about 640,000 games; the "Played" column). On cards the builder plays in at least 20 of 1,000 sample decks, prediction and play rank-correlate at <b>0.88</b>. On the 120 cards it rarely or never builds, that drops to 0.69, and every large miss is a card built in 0 decks: Fatehold Charm (predicted −10.6, played −0.4), Loyal Tutor (−15.1 vs −5.0), Theorix Charm (−12.4 vs −3.8). The net never saw these cards in decks, so it extrapolates. The set ranking now uses the played number, with the net beside it.</p>
<p><b>2. The pick-order gap is in the games, not the net.</b> The played rating correlates with Draftsim at <b>0.32</b>, exactly as the net does (0.32). So swapping in the net's numbers fixes nothing there; the gap comes from the simulated games themselves. Three causes:</p>
<ul>
<li><b>Bot and engine bugs on specific cards</b> (table below). Of the 15 cards with a known issue, 14 land in the bottom fifth of the played ranking (the exception, Lich's Relic, is #172 of 260). These ratings are wrong: they measure the bot's handling, not the card.</li>
<li><b>Complexity.</b> The bot plays simple cards well (creatures, removal, plain spells) and situational or build-around cards badly. In the played ranking commons sit 20 percentile points higher than on Draftsim, rares 14 and mythics 12 lower.</li>
<li><b>Genuine sealed-versus-draft differences.</b> Draftsim is a draft pick order. Some gaps are plausibly real for sealed against these decks, e.g. Ghalta the Immovable (played +5.5, Draftsim 2.0), though a human can't tell from here which gaps are real and which are bot artifacts.</li>
</ul>
<p><b>3. The card code itself is mostly right.</b> Three agents compared 30 suspect cards' implementations with Scryfall's oracle text clause by clause. 27 match exactly or with a harmless approximation. Cruel Calculations had a real card bug, fixed 2026-11-16 (after the net's training games, so the net's number for it is stale). Every other problem found is in the bot: how it picks targets, which abilities it ever uses, and when it casts.</p>
<h2>Fix batch, 2026-11-22</h2>
<p>The engine and bot fixes below are in, and the set was re-rated in play on the fixed engine and bot (same 1,000 sample decks, seed 43). Every other card's rating moved by under 0.1 points on average, so the ranking elsewhere is unchanged. Agreement with Draftsim rises from 0.32 to 0.35. The investigation above quotes the numbers from before the fixes.</p>
<ul>
<li><b>Equipment is equipped now.</b> The engine's equip paid only from floating mana and the bots never float mana, so they cast Equipment and never attached it. Equip now taps lands like every other cost (CR 602.2b). Medic's Kitesail −4.8 → +0.5, Hunter's Axe −4.5 → −0.1, Lich's Relic +0.5 → +4.5 (#172 → #16), Warrior's Blades −0.6 → +2.4.</li>
<li><b>Target picks fixed:</b> Vindictive Triumph (battlefield, not graveyard) −3.2 → +1.4; Puppet Crafting (your own artifact) −3.9 → −0.2; Yoshimaru (a winning fight or none) −2.9 → −1.6; Flickering Hound (no pointless blinks) unchanged at −2.6, because once it stops blinking things it shouldn't it's a 4-mana 2/2.</li>
<li><b>Three bot-policy fixes, gated and adopted into <code>fra-fixes</code>:</b> no-op casts are held, planeswalker loyalty is chosen better, and Gideon's Memorial's combat ability is used. Head-to-head against the old bundle every cell read 50.0–50.4 (no loss), and the blunder benchmark held (68.5 → 68.8 of 75). Chandra, Chill of Compliance +2.1; the others +0.3.</li>
<li><b>Still open:</b> Gideon's Memorial (−9, cause unknown), Fatehold Charm's mode policy, Emrakul's ramp (using it measured harmful), Chandra's surveil rider, and 158 catalog cards outside FRA coded like Vindictive Triumph that need an oracle check.</li>
</ul>
<h2>Cards with a known issue</h2>
<p class="muted">"Before" and "After" are the played rating before and after the fix batch. An open issue's rating still measures the bot's mistakes.</p>
<div id="issue-list"></div>
<h2>Shared root causes</h2>
<ul>
<li><b>(Fixed.) Equipment was almost never equipped</b> (1–2 equips per ~700 equipment casts, both pilots). Affects Medic's Kitesail, Hunter's Axe, Lich's Relic, Warrior's Blades and every other equipment.</li>
<li><b>(Partly fixed.) Abilities activated from the hand were never used</b> except discard-pumps; the combat-damage kind is now (Emrakul's ramp, Gideon's Memorial's 4 damage).</li>
<li><b>(Fixed for these three.) The default target picker classifies effects by shape</b>: an exile-and-reanimate spell is treated as graveyard hate (Vindictive Triumph), an "up to one creature you control" blink is never declined (Flickering Hound), an aura that sets base P/T is treated as hostile (Puppet Crafting).</li>
<li><b>(Fixed.) Conditional spells were cast when the condition was off</b> (Germinate Recruits with no life gained, Loyal Tutor with no planeswalker in the deck).</li>
</ul>
<p>Fixes that change how every bot plays were gated pilot-against-pilot before adoption (<code>.ladder/fix1/gate.sh</code>); engine rules fixes were kept on correctness and read on these cards' play ratings.</p>
'''
TEMPLATE = TEMPLATE.replace("$(\"#issues\").innerHTML = ISSUES_HTML;", "$(\"#issues\").innerHTML = " + json.dumps(ISSUES) + "; renderIssues();")

if __name__ == "__main__":
    main()
