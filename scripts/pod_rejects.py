#!/usr/bin/env python3
"""Count the actions the engine refuses from the pod bots, by (action, error).

    scripts/pod_rejects.py BIN SEED GAMES [--a PILOT]

Plays GAMES random 3-6-seat Commander pods (one game each, seeds SEED*1000+i)
under `CRAB_POD_TRACE=0`, which prints a `REJECT …` line for every refused
action, and prints each class with one replay command. A rejected declaration
costs the seat its whole combat, so every class is a bot (or engine) bug. Add
`CRAB_SIM_REJECTS=names` to a replay for the `combat.rs` line that refused it.
"""
import argparse, collections, random, re, subprocess

ap = argparse.ArgumentParser()
ap.add_argument('bin'); ap.add_argument('seed', type=int); ap.add_argument('games', type=int)
ap.add_argument('--a')
a = ap.parse_args()
rng = random.Random(a.seed)
probe = subprocess.run([a.bin, '--commander', '--pod-decks', '99999,1', '--games', '1'], capture_output=True, text=True)
ndecks = int(re.findall(r'1\.\.=(\d+)', probe.stdout + probe.stderr)[-1])
count, example = collections.Counter(), {}
for i in range(a.games):
    k = rng.choice([3, 4, 5, 6])
    decks = ','.join(str(rng.randint(1, ndecks)) for _ in range(k))
    seed = a.seed * 1000 + i
    cmd = [a.bin, '--commander', '--seats', str(k), '--pod-decks', decks, '--games', '1', '--seed', str(seed), '--threads', '1']
    cmd += ['--a', a.a] if a.a else []
    p = subprocess.Popen(cmd, env={'CRAB_POD_TRACE': '0', 'PATH': '/usr/bin:/bin'},
                         stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    for line in p.stderr:
        if line.startswith('REJECT'):
            act = re.search(r'p\d+ p\d+ (\w+)', line)
            err = re.search(r':: (\w+)', line)
            key = (act.group(1) if act else '?', err.group(1) if err else '?')
            count[key] += 1
            example.setdefault(key, f"--seats {k} --pod-decks {decks} --seed {seed}{' --a ' + a.a if a.a else ''} :: {line[:300]}")
    p.wait()
print(f"{sum(count.values())} rejections in {a.games} games")
for key, n in count.most_common():
    print(n, key, '\n   ', example[key])
