#!/usr/bin/env python3
"""Greedy single-swap deck search on the deck gauntlet, the 2026-09-06 rules.

    deck_search.py --base LIST --pool POOL --cards cards.json --work DIR
                   [--results FILE] [--gens 3] [--pairs 500] [--threads 20]
                   [--neutral "Oracle's Restoration"] [--bar 1.5] [--tag r2]
                   [--exclude "A;B"] [--gauntlet ./target/release-fast/deck_gauntlet]

Each generation, from the incumbent: every single cut (neutral card in),
every legal add (for the weakest card the cuts found), then swaps of the
three cheapest cuts x the four best adds. Every list is screened at
`--pairs` x 24 opponents on seeds 43 and 97 (cells already in --results are
reused). The best list replaces the incumbent only if it beats it by
>= --bar points AND >= 2 se on the pooled estimate AND on both seeds in
sign; otherwise the search stops. Then +/-1 basic on the final. Writes
`<work>/final.txt` and a log of every decision.
"""
import argparse
import math
import os
import re
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
VARIANTS = os.path.join(HERE, "deck_variants.py")
FIELD = "9,12,16,20x6"
FIELD_SEED = "0xdecc0000"
SEEDS = (43, 97)


def log(work, msg):
    line = f"[{time.strftime('%H:%M:%S')}] {msg}"
    print(line, flush=True)
    with open(os.path.join(work, "search.log"), "a") as f:
        f.write(line + "\n")


def parse_results(path):
    cells = {}
    if not os.path.exists(path):
        return cells
    for line in open(path):
        if not line.startswith("RESULT "):
            continue
        kv = dict(re.findall(r"(\w+)=(\S+)", line))
        if kv.get("pilot") != "dflt" or kv.get("opp_pilot") != "dflt":
            continue
        if kv.get("field") != f"{FIELD}/{FIELD_SEED}":
            continue
        try:
            cells[(kv["deck"], int(kv["seed"]), int(kv["pairs"]))] = (float(kv["win"]), float(kv["se"]))
        except (KeyError, ValueError):
            continue
    return cells


def run_cell(a, deck_path, seed):
    cmd = [a.gauntlet, "--deck", deck_path, "--pairs", str(a.pairs), "--seed", str(seed),
           "--threads", str(a.threads), "--out", a.results]
    subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)


def screen(a, paths):
    """name -> (pooled win, pooled se, {seed: win}) for every list in `paths`."""
    out = {}
    for p in paths:
        name = os.path.basename(p)[:-4]
        cells = parse_results(a.results)
        for s in SEEDS:
            if (name, s, a.pairs) not in cells:
                run_cell(a, p, s)
        cells = parse_results(a.results)
        per = {s: cells.get((name, s, a.pairs)) for s in SEEDS}
        if any(v is None for v in per.values()):
            log(a.work, f"  ! no result for {name}: {per}")
            continue
        w = [1.0 / (v[1] ** 2) for v in per.values()]
        pooled = sum(wi * v[0] for wi, v in zip(w, per.values())) / sum(w)
        se = 1.0 / math.sqrt(sum(w))
        out[name] = (pooled, se, {s: v[0] for s, v in per.items()})
    return out


def gen_variants(a, deck_path, out_dir, mode, prefix, **kw):
    cmd = [sys.executable, VARIANTS, "--deck", deck_path, "--pool", a.pool, "--cards", a.cards,
           "--out", out_dir, "--prefix", prefix, "--exclude", a.exclude, mode]
    for k, v in kw.items():
        cmd += [f"--{k.replace('_', '-')}", v]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL)
    return sorted(os.path.join(out_dir, f) for f in os.listdir(out_dir) if f.endswith(".txt"))


def note_of(path):
    with open(path) as f:
        first = f.readline().strip()
    return first.split(": ", 1)[1] if ": " in first else first


def clears(best, inc, bar):
    """The pre-registered bar: delta >= bar, >= 2 se(diff), both seeds agree."""
    (bw, bse, bseeds), (iw, ise, iseeds) = best, inc
    d = bw - iw
    dse = math.sqrt(bse ** 2 + ise ** 2)
    both = all(bseeds[s] - iseeds[s] > 0 for s in SEEDS)
    return d >= bar and d >= 2 * dse and both, d, dse


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True)
    ap.add_argument("--pool", required=True)
    ap.add_argument("--cards", required=True)
    ap.add_argument("--work", required=True)
    ap.add_argument("--results", default=None)
    ap.add_argument("--gens", type=int, default=3)
    ap.add_argument("--pairs", type=int, default=500)
    ap.add_argument("--threads", type=int, default=20)
    ap.add_argument("--neutral", default="Oracle's Restoration")
    ap.add_argument("--bar", type=float, default=1.5)
    ap.add_argument("--tag", default="r2")
    ap.add_argument("--exclude", default="")
    ap.add_argument("--gauntlet", default="./target/release-fast/deck_gauntlet")
    a = ap.parse_args()
    os.makedirs(a.work, exist_ok=True)
    a.results = a.results or os.path.join(a.work, "results.txt")

    # The incumbent is copied under a stable name so its cells are reusable.
    inc_path = os.path.join(a.work, f"{a.tag}_inc0.txt")
    with open(a.base) as src, open(inc_path, "w") as dst:
        dst.write(f"# {a.tag}_inc0: {a.base}\n")
        dst.write("".join(l for l in src if not l.startswith("#")))
    log(a.work, f"search {a.tag}: base {a.base}, bar {a.bar}, {a.pairs} pairs x 24 x 2 seeds")
    inc = screen(a, [inc_path])[f"{a.tag}_inc0"]
    inc_name = f"{a.tag}_inc0"
    log(a.work, f"incumbent {inc_name}: {inc[0]:.2f} ±{1.96 * inc[1]:.2f} {inc[2]}")

    for gen in range(1, a.gens + 1):
        pre = f"{a.tag}g{gen}_"
        cuts_dir = os.path.join(a.work, f"{a.tag}_g{gen}_cuts")
        cuts = gen_variants(a, inc_path, cuts_dir, "cuts", pre, neutral=a.neutral)
        res_cuts = screen(a, cuts)
        ranked_cuts = sorted(res_cuts.items(), key=lambda kv: -kv[1][0])
        log(a.work, f"gen {gen} cuts ({len(ranked_cuts)}), top 5: " + "; ".join(
            f"{note_of(os.path.join(cuts_dir, n + '.txt'))} {v[0]:.2f}" for n, v in ranked_cuts[:5]))
        # The weakest card is the one whose removal hurt least (or helped).
        weakest_note = note_of(os.path.join(cuts_dir, ranked_cuts[0][0] + ".txt"))
        weakest = weakest_note.split(" +")[0].lstrip("-")
        adds_dir = os.path.join(a.work, f"{a.tag}_g{gen}_adds")
        adds = gen_variants(a, inc_path, adds_dir, "adds", pre, neutral_cut=weakest)
        res_adds = screen(a, adds)
        ranked_adds = sorted(res_adds.items(), key=lambda kv: -kv[1][0])
        log(a.work, f"gen {gen} adds for {weakest} ({len(ranked_adds)}), top 6: " + "; ".join(
            f"{note_of(os.path.join(adds_dir, n + '.txt'))} {v[0]:.2f}" for n, v in ranked_adds[:6]))
        top_cuts = [note_of(os.path.join(cuts_dir, n + ".txt")).split(" +")[0].lstrip("-")
                    for n, _ in ranked_cuts[:3]]
        top_adds = [note_of(os.path.join(adds_dir, n + ".txt")).split(" -")[0].lstrip("+")
                    for n, _ in ranked_adds[:4]]
        swaps_dir = os.path.join(a.work, f"{a.tag}_g{gen}_swaps")
        swaps = gen_variants(a, inc_path, swaps_dir, "swaps", pre, cuts=";".join(top_cuts), adds=";".join(top_adds))
        res_swaps = screen(a, swaps)
        pool = {}
        for d, r in ((cuts_dir, res_cuts), (adds_dir, res_adds), (swaps_dir, res_swaps)):
            for n, v in r.items():
                pool[n] = (v, os.path.join(d, n + ".txt"))
        best_name, (best, best_path) = max(pool.items(), key=lambda kv: kv[1][0][0])
        ok, d, dse = clears(best, inc, a.bar)
        log(a.work, f"gen {gen} best: {note_of(best_path)} {best[0]:.2f} ({d:+.2f} ±{1.96 * dse:.2f}, seeds {best[2]}) -> {'ADOPT' if ok else 'STOP'}")
        if not ok:
            break
        inc, inc_name, inc_path = best, best_name, best_path

    lands_dir = os.path.join(a.work, f"{a.tag}_lands")
    lands = gen_variants(a, inc_path, lands_dir, "lands", f"{a.tag}l_", neutral=a.neutral)
    res_lands = screen(a, lands)
    ranked = sorted(res_lands.items(), key=lambda kv: -kv[1][0])
    log(a.work, "lands, top 3: " + "; ".join(f"{note_of(os.path.join(lands_dir, n + '.txt'))} {v[0]:.2f}" for n, v in ranked[:3]))
    if ranked:
        ok, d, dse = clears(ranked[0][1], inc, a.bar)
        if ok:
            inc, inc_name, inc_path = ranked[0][1], ranked[0][0], os.path.join(lands_dir, ranked[0][0] + ".txt")
            log(a.work, f"lands ADOPT {note_of(inc_path)} ({d:+.2f})")
    final = os.path.join(a.work, "final.txt")
    with open(inc_path) as src, open(final, "w") as dst:
        dst.write(f"# final ({a.tag}): {inc_name} = {note_of(inc_path)}; {inc[0]:.2f} ±{1.96 * inc[1]:.2f} on the screen field\n")
        dst.write("".join(l for l in src if not l.startswith("#")))
    log(a.work, f"FINAL {inc_name}: {inc[0]:.2f} ±{1.96 * inc[1]:.2f} {inc[2]} -> {final}")


if __name__ == "__main__":
    main()
