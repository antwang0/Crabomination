#!/usr/bin/env python3
"""Compare the engine's FRA card ratings with Draftsim's (and 17lands').

Reads a `fra_card_eval` output directory and prints rank correlations
(Spearman) between Draftsim's rating and four engine-side ratings:

  static   the sealed builder's colour-neutral card score
           (`CardBrief::base_score + quality`) — what deck building ranks by
  played   maindeck rate: decks that played the card / pools that had it
  gpwr     win rate of the decks that played the card (17lands' GP WR)
  value    ridge-regression card value: each match's score margin regressed
           on (copies in deck A - copies in deck B), so a card is credited
           for its deck's results with its deck-mates' values held fixed;
           in win-rate points per copy

usage: fra_card_eval.py RUN_DIR [DRAFTSIM_TSV] [--l17 17LANDS_JSON]

Without DRAFTSIM_TSV (a set with no Draftsim file, `fra_card_eval --set
sos`) it skips the comparisons and only writes ratings.tsv.

Inputs (third-party, not tracked):
  DRAFTSIM_TSV  the `../data/FRA.txt` template literal embedded in Draftsim's
                draft-app bundle (https://draftsim.com/draft.php?s=FRA names
                the `/draft-app/dist/assets/index-<hash>.js` it loads)
  17LANDS_JSON  https://www.17lands.com/card_ratings/data?expansion=FRA
                &format=PremierDraft&start_date=2020-01-01
"""
import argparse
import json
import math
import os

import numpy as np


def ranks(x):
    x = np.asarray(x, float)
    order = x.argsort(kind="mergesort")
    r = np.empty(len(x))
    r[order] = np.arange(len(x))
    # average ties
    for v in np.unique(x):
        m = x == v
        if m.sum() > 1:
            r[m] = r[m].mean()
    return r


def spearman(a, b):
    a, b = np.asarray(a, float), np.asarray(b, float)
    ok = ~(np.isnan(a) | np.isnan(b))
    a, b = a[ok], b[ok]
    n = len(a)
    if n < 4:
        return float("nan"), n, (float("nan"), float("nan"))
    rho = float(np.corrcoef(ranks(a), ranks(b))[0, 1])
    # Fisher-z 95% interval (Bonett-Wright variance for Spearman)
    z = math.atanh(max(min(rho, 0.9999), -0.9999))
    se = math.sqrt((1 + rho * rho / 2) / (n - 3))
    return rho, n, (math.tanh(z - 1.96 * se), math.tanh(z + 1.96 * se))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("draftsim", nargs="?")
    ap.add_argument("--l17")
    ap.add_argument("--min-decks", type=int, default=30)
    ap.add_argument("--show", type=int, default=12)
    args = ap.parse_args()

    ds = {}
    for line in (open(args.draftsim).read().strip().split("\n")[1:] if args.draftsim else []):
        c = line.split("\t")
        if len(c) >= 8 and c[7] == "base":
            ds[c[0].replace("_", " ")] = float(c[6])

    cards = []
    for line in open(os.path.join(args.run, "cards.tsv")).read().strip().split("\n")[1:]:
        name, rar, static, qual, in_pool, in_deck = line.split("\t")
        cards.append(dict(name=name, rarity=rar, static=float(static), quality=float(qual),
                          in_pool=int(in_pool), in_deck=int(in_deck)))
    idx = {c["name"]: i for i, c in enumerate(cards)}
    n = len(cards)

    # decks: card name lists -> count vectors (front-face names too)
    def key(nm):
        if nm in idx:
            return idx[nm]
        return idx.get(nm.split(" // ")[0])

    decks = []
    for line in open(os.path.join(args.run, "decks.tsv")):
        parts = line.rstrip("\n").split("\t")
        v = np.zeros(n)
        for nm in parts[1:]:
            k = key(nm)
            if k is not None:
                v[k] += 1
        decks.append(v)
    D = np.array(decks)
    present = D > 0

    games = np.loadtxt(os.path.join(args.run, "games.tsv"), dtype=int, ndmin=2)
    a, b, wa, wb = games[:, 0], games[:, 1], games[:, 2], games[:, 3]
    decided = wa + wb
    m = decided > 0
    a, b, wa, wb, decided = a[m], b[m], wa[m], wb[m], decided[m]
    print(f"{len(decks)} decks, {len(a)} matches, {decided.sum()} decided games, "
          f"{games[:, 4].sum()} undecided")

    # GP WR: per-deck wins/games, then per card over decks playing it
    dw = np.zeros(len(decks))
    dg = np.zeros(len(decks))
    np.add.at(dw, a, wa)
    np.add.at(dw, b, wb)
    np.add.at(dg, a, decided)
    np.add.at(dg, b, decided)
    gp_w = present.T.astype(float) @ dw
    gp_g = present.T.astype(float) @ dg
    gpwr = np.where(gp_g > 0, gp_w / np.maximum(gp_g, 1), np.nan)

    # Ridge on the match margin, weighted by decided games; lambda by 5-fold CV
    X = D[a] - D[b]
    y = (wa - wb) / decided
    w = decided.astype(float)
    rng = np.random.default_rng(0)
    fold = rng.integers(0, 5, len(y))

    def fit(Xt, yt, wt, lam):
        A = (Xt * wt[:, None]).T @ Xt + lam * np.eye(n)
        return np.linalg.solve(A, (Xt * wt[:, None]).T @ yt)

    best = None
    for lam in [1, 3, 10, 30, 100, 300, 1000, 3000]:
        err = 0.0
        for f in range(5):
            tr, te = fold != f, fold == f
            beta = fit(X[tr], y[tr], w[tr], lam)
            err += float((w[te] * (X[te] @ beta - y[te]) ** 2).sum())
        if best is None or err < best[1]:
            best = (lam, err)
    lam = best[0]
    beta = fit(X, y, w, lam)
    value = beta * 50.0  # margin is 2p-1, so 50*beta = win-rate points per copy
    print(f"ridge lambda {lam} (5-fold CV)")

    played = np.array([c["in_deck"] / c["in_pool"] if c["in_pool"] else np.nan for c in cards])
    decks_with = present.sum(0)
    for i, c in enumerate(cards):
        c.update(played=played[i], gpwr=gpwr[i], value=value[i], decks=int(decks_with[i]),
                 ds=ds.get(c["name"], np.nan))

    l17 = {}
    if args.l17:
        for r in json.load(open(args.l17)):
            if r.get("ever_drawn_win_rate") is not None:
                l17[r["name"]] = r["ever_drawn_win_rate"] * 100
                l17[r["name"].split(" // ")[0]] = r["ever_drawn_win_rate"] * 100
    for c in cards:
        c["gih"] = l17.get(c["name"], np.nan)

    nonland = [c for c in cards]
    enough = [c for c in cards if c["decks"] >= args.min_decks]

    def col(rows, k):
        return [r[k] for r in rows]

    def table(title, rows, ref):
        print(f"\n{title}")
        print(f"  {'engine rating':<10} {'rho':>6}  {'95% CI':>15}  {'n':>4}")
        for k, label in [("static", "static"), ("played", "played"), ("gpwr", "gpwr"),
                         ("value", "value")]:
            rr = rows if k in ("static", "played") else [r for r in rows if r["decks"] >= args.min_decks]
            rho, nn, (lo, hi) = spearman(col(rr, ref), col(rr, k))
            print(f"  {label:<10} {rho:6.3f}  [{lo:6.3f}, {hi:6.3f}]  {nn:4d}")

    if not args.draftsim:
        write_ratings(args.run, cards)
        return
    table("Spearman vs Draftsim rating (all FRA cards)", nonland, "ds")
    for rar in "CURM":
        rows = [c for c in nonland if c["rarity"] == rar]
        table(f"  ...rarity {rar} only", rows, "ds")
    if l17:
        rho, nn, (lo, hi) = spearman(col(cards, "ds"), col(cards, "gih"))
        print(f"\nReference: Draftsim vs 17lands GIH WR: rho {rho:.3f} [{lo:.3f}, {hi:.3f}] n={nn}")
        table("Spearman vs 17lands GIH WR", cards, "gih")

    # disagreements on the best engine rating that has data
    rows = [c for c in enough if not math.isnan(c["ds"])]
    rd = ranks(col(rows, "ds")) / (len(rows) - 1)
    rv = ranks(col(rows, "value")) / (len(rows) - 1)
    for r, x, y_ in zip(rows, rd, rv):
        r["gap"] = y_ - x
    rows.sort(key=lambda r: r["gap"])
    hdr = f"  {'card':<34}{'rar':>3}{'draftsim':>9}{'value':>7}{'gpwr':>6}{'played':>7}{'static':>7}"
    fmt = lambda r: (f"  {r['name'][:33]:<34}{r['rarity']:>3}{r['ds']:9.1f}{r['value']:7.2f}"
                     f"{r['gpwr']*100:6.1f}{r['played']*100:6.0f}%{r['static']:7.0f}")
    print(f"\nDraftsim rates higher than the sim (value):\n{hdr}")
    for r in rows[: args.show]:
        print(fmt(r))
    print(f"\nThe sim rates higher than Draftsim:\n{hdr}")
    for r in rows[::-1][: args.show]:
        print(fmt(r))

    write_ratings(args.run, cards)


def write_ratings(run, cards):
    with open(os.path.join(run, "ratings.tsv"), "w") as f:
        f.write("name\trarity\tdraftsim\tl17_gih\tstatic\tplayed\tgpwr\tvalue\tdecks\n")
        for c in sorted(cards, key=lambda c: -c["value"]):
            f.write(f"{c['name']}\t{c['rarity']}\t{c['ds']}\t{c['gih']}\t{c['static']}\t"
                    f"{c['played']:.3f}\t{c['gpwr']:.4f}\t{c['value']:.3f}\t{c['decks']}\n")


if __name__ == "__main__":
    main()
