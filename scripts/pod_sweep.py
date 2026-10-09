#!/usr/bin/env python3
"""Audit-build pod sweep: every deck in shuffled `--pod-decks` groups.

    scripts/pod_sweep.py BIN SEED SEATS GAMES [--jobs N] [--a PROFILE] [--env K=V ...]

SEATS is a comma list (`4,6,8`); each seat count shuffles every target deck
into groups of that size (topped up at random) and plays GAMES a group with
seed `SEED*100 + seats*7 + offset`. A block that aborts, fails to parse, or
leaves a game undecided is printed with its replay command, and with
`--classify` each undecided game is replayed under `CRAB_POD_TRACE=0` and
named by the last 60 actions' stack tops (a loop names its triggers). Build
BIN as NEXT's audit build; `--a dflt` is the production pilot (~10x slower
than the default `baseline`).
"""
import argparse, collections, os, random, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

ap = argparse.ArgumentParser()
ap.add_argument('bin'); ap.add_argument('seed', type=int); ap.add_argument('seats'); ap.add_argument('games', type=int)
ap.add_argument('--jobs', type=int, default=1); ap.add_argument('--a'); ap.add_argument('--env', nargs='*', default=[])
ap.add_argument('--classify', action='store_true')
a = ap.parse_args()
env = dict(os.environ, **dict(kv.split('=', 1) for kv in a.env))
probe = subprocess.run([a.bin, '--commander', '--pod-decks', '99999,1', '--games', '1'], capture_output=True, text=True)
ndecks = int(re.findall(r'1\.\.=(\d+)', probe.stdout + probe.stderr)[-1])
extra = ['--a', a.a] if a.a else []
rng = random.Random(a.seed)
work = []
for k in map(int, a.seats.split(',')):
    decks = list(range(1, ndecks + 1)); rng.shuffle(decks)
    while len(decks) % k:
        decks.append(rng.randint(1, ndecks))
    for g in range(0, len(decks), k):
        work.append((k, ','.join(map(str, decks[g:g + k])), a.seed * 100 + k * 7 + g))

def cmd(k, decks, seed, games, first=None):
    c = [a.bin, '--commander', '--seats', str(k), '--pod-decks', decks, '--games', str(games), '--seed', str(seed)] + extra
    return c + (['--first', str(first)] if first is not None else [])

def classify(k, decks, seed, game):
    out = subprocess.run(cmd(k, decks, seed, 1, game), stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                         env=dict(env, CRAB_POD_TRACE='0')).stdout.splitlines()
    acts = [l for l in out if re.match(r'^\d+ t\d+', l)]
    tops = collections.Counter(re.findall(r'\{top: ([^}]*)\}', ' '.join(acts[-60:])))
    by = [l.strip() for l in out if 'undecided_by' in l]
    return f"    game {game}: {acts[-1][:100] if acts else ''}\n      tops {dict(tops.most_common(3))} {by}"

def run(w):
    k, decks, seed = w
    p = subprocess.run(cmd(k, decks, seed, a.games), capture_output=True, text=True, env=env)
    out = p.stdout + p.stderr
    m = re.search(r'decided (\d+).*undecided (\d+)', out)
    games = [int(g) for g in re.findall(r'undecided game (\d+):', out)]
    if p.returncode == 0 and m and not games:
        return None
    line = f"BAD k={k} seed={seed} decks={decks} rc={p.returncode} " + (f"dec={m.group(1)} und={m.group(2)}" if m else "NOPARSE")
    if p.returncode != 0 or not m:
        line += '\n    ' + '\n    '.join(out.strip().splitlines()[-8:])
    if a.classify:
        line += ''.join('\n' + classify(k, decks, seed, g) for g in games)
    return line

bad = 0
with ThreadPoolExecutor(a.jobs) as ex:
    for line in ex.map(run, work):
        if line:
            bad += 1
            print(line, flush=True)
print(f"SWEEP DONE seed={a.seed} blocks={len(work)} games={len(work) * a.games} bad={bad}", flush=True)
