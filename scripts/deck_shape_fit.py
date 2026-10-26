"""usage: deck_shape_fit.py RUN_DIR DRAFTSIM_TSV (a `fra_card_eval` rating run).
Item 3 (round 84): do deck-level features explain match margins beyond per-card values?
Joint ridge: margin ~ (cards_a - cards_b) beta + (feat_a - feat_b) gamma.
Features from Draftsim's FRA sheet (type, casting cost)."""
import numpy as np, sys, re, collections
RUN = sys.argv[1]; DS = sys.argv[2]
info = {}
for line in open(DS).read().strip().split("\n")[1:]:
    c = line.split("\t")
    if len(c) < 8 or c[7] != "base": continue
    cost = c[2]; mv = 0
    for tok in re.findall(r"\d+|[WUBRGCX]", cost):
        mv += int(tok) if tok.isdigit() else (0 if tok == "X" else 1)
    info[c[0].replace("_", " ")] = (c[4], mv)
names = [l.split("\t")[0] for l in open(f"{RUN}/cards.tsv").read().strip().split("\n")[1:]]
idx = {n: i for i, n in enumerate(names)}
n = len(names)
def key(nm):
    return idx.get(nm, idx.get(nm.split(" // ")[0]))
def ginfo(nm):
    return info.get(nm) or info.get(nm.split(" // ")[0])
FEATS = ["creatures", "creatures_sq", "mv_le2", "mv_ge5", "lands", "mv_mean"]
D, F = [], []
for line in open(f"{RUN}/decks.tsv"):
    parts = line.rstrip("\n").split("\t")[1:]
    v = np.zeros(n); cre = 0; lands = 0; le2 = 0; ge5 = 0; mvs = []
    for nm in parts:
        k = key(nm)
        if k is not None: v[k] += 1
        gi = ginfo(nm)
        typ, mv = gi if gi else ("Land", 0)
        if "Land" in typ: lands += 1; continue
        mvs.append(mv)
        if "Creature" in typ: cre += 1
        if mv <= 2: le2 += 1
        if mv >= 5: ge5 += 1
    D.append(v); F.append([cre, 0, le2, ge5, lands, np.mean(mvs) if mvs else 0])
D = np.array(D); F = np.array(F, float)
F[:, 1] = (F[:, 0] - F[:, 0].mean()) ** 2
print("feature means", dict(zip(FEATS, F.mean(0).round(2))), "sd", dict(zip(FEATS, F.std(0).round(2))))
print("creature count distribution", sorted(collections.Counter(F[:, 0].astype(int)).items()))
g = np.loadtxt(f"{RUN}/games.tsv", dtype=int, ndmin=2)
a, b, wa, wb = g[:, 0], g[:, 1], g[:, 2], g[:, 3]; dec = wa + wb; m = dec > 0
a, b, wa, wb, dec = a[m], b[m], wa[m], wb[m], dec[m]
y = (wa - wb) / dec; w = dec.astype(float)
Fs = (F - F.mean(0)) / np.where(F.std(0) > 0, F.std(0), 1)
def design(use):
    X = D[a] - D[b]
    if use: X = np.hstack([X, (Fs[a] - Fs[b])[:, use]])
    return X
def fit(X, y, w, lam):
    A = (X * w[:, None]).T @ X + lam * np.eye(X.shape[1])
    return np.linalg.solve(A, (X * w[:, None]).T @ y)
rng = np.random.default_rng(0); fold = rng.integers(0, 5, len(y))
def cv(use, lam=30):
    X = design(use); err = 0.0
    for f in range(5):
        tr, te = fold != f, fold == f
        beta = fit(X[tr], y[tr], w[tr], lam); err += float((w[te] * (X[te] @ beta - y[te]) ** 2).sum())
    return err / w.sum()
for lam in (10, 30, 100):
    print(f"lambda {lam}: cards only CV mse {cv([], lam):.6f}")
base = cv([])
for use in ([0], [0, 1], [2, 3], [4], [5], [0, 1, 2, 3, 4, 5]):
    e = cv(use); print(f"+{[FEATS[i] for i in use]}: CV mse {e:.6f}  (delta {e-base:+.6f}, {100*(base-e)/base:+.3f}%)")
X = design(list(range(6))); beta = fit(X, y, w, 30)
# bootstrap-free SE: refit on 5 folds' complements for spread
gs = np.array([fit(X[fold != f], y[fold != f], w[fold != f], 30)[n:] for f in range(5)])
for i, nm in enumerate(FEATS):
    sd = F[:, i].std() if F[:, i].std() > 0 else 1
    print(f"  {nm:<13} {50*beta[n+i]:+.3f} win pts per SD ({sd:.2f} units)  per unit {50*beta[n+i]/sd:+.3f}  fold spread ±{50*gs[:, i].std():.3f}")

# Nonparametric: per-deck mean residual of the cards-only fit, by creature count
Xc = D[a] - D[b]; bc = fit(Xc, y, w, 30); r = y - Xc @ bc
rs = np.zeros(len(D)); ws = np.zeros(len(D))
np.add.at(rs, a, r * w); np.add.at(ws, a, w); np.add.at(rs, b, -r * w); np.add.at(ws, b, w)
print("\nresidual (win pts) by creature count, cards-only fit:")
for c in range(11, 25):
    sel = (F[:, 0] == c) & (ws > 0)
    if sel.sum() < 20: continue
    v = 50 * rs[sel] / ws[sel]
    print(f"  {c:2d} creatures: {v.mean():+.2f} ±{1.96*v.std()/np.sqrt(sel.sum()):.2f}  (n={sel.sum()})")
for j, nm in ((2, "mv<=2"), (3, "mv>=5")):
    print(f"residual by {nm} count:")
    for c in range(int(F[:, j].min()), int(F[:, j].max()) + 1):
        sel = (F[:, j] == c) & (ws > 0)
        if sel.sum() < 50: continue
        v = 50 * rs[sel] / ws[sel]
        print(f"  {c:2d}: {v.mean():+.2f} ±{1.96*v.std()/np.sqrt(sel.sum()):.2f}  (n={sel.sum()})")
