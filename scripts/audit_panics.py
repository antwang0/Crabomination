#!/usr/bin/env python3
"""Which panicking sites on the engine path are *proved* safe by code beside
them, and which are only asserted by their message.

The standing robustness goal this is a filter for: **no panic, unwrap or
out-of-bounds reachable from bot self-play.** A panic at game 400,000 kills a
training run that has been going for hours, and the 33,120-game
`-C debug-assertions=yes` grid only covers what a game actually reaches — it
cannot tell you that the site nobody reached is safe.

A **guarded** site has a proof within its own statement region: a
`is_empty()` / `is_none()` / `len()` test that returns, breaks or continues
before it; a `match` arm or `if let` that already bound the shape; the same
collection pushed to a few lines up; or a short-circuit (`x.is_none() ||
f(x.unwrap())`) that cannot reach the unwrap with `None`. A **lock** site is
`Mutex`/`RwLock` poisoning, which is only reachable once some *other* panic
has already happened, so it is not a first cause.

A **bare** site has none of those. That is not automatically a bug — plenty
are safe for a reason two frames up, which is exactly why the output is a
triage list rather than a gate. It *is* the population a robustness pass
should read, and re-running this is how "the engine cannot panic in
self-play" gets re-checked instead of re-quoted.

`src/bin/` is excluded: a CLI entry point that panics on a bad argument has
failed correctly, and those are not on the self-play path. `#[cfg(test)]`
tails and `src/tests/` are excluded for the same reason.

    python3 scripts/audit_panics.py            # counts + the bare list
    python3 scripts/audit_panics.py --verbose  # + guarded, with the proof

Exit status is 0 always: the bare count is a number to compare against the
last reading, not a pass/fail.
"""

import os
import re
import sys

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

ROOTS = ["crabomination/src", "crabomination_base/src"]
SKIP_DIRS = ("/bin/", "/tests/")

SITE = re.compile(r"\.unwrap\(\)|\.expect\(|\bpanic!\(|\bunreachable!\(")
# Poisoning only — reachable only after some other panic, so never a first
# cause. `into_inner` is the same lock's consuming form.
LOCK = re.compile(r"\.(lock|read|write|into_inner)\(\)\s*\.\s*(unwrap|expect)\b")
# How far back a proof still covers the site. The widest real one in the
# engine is `EachPlayerPutsHandCardOnTop`, 20 lines from its `is_empty()`
# `continue` to the `first().unwrap()` it protects.
LOOKBACK = 22
FN = re.compile(r"^\s*(?:pub(?:\(crate\))?\s+)?(?:async\s+)?fn ([a-z_0-9]+)")

# A proof is one of these appearing in the lookback window *and* an exit or a
# binding that makes the site unreachable on the empty/None branch. Kept
# deliberately syntactic: this reports what it can see, and says so.
PROOFS = (
    ("is_empty", re.compile(r"\.is_empty\(\)")),
    ("is_some/is_none", re.compile(r"\.is_some\(\)|\.is_none\(\)")),
    ("len test", re.compile(r"\.len\(\)\s*[<>=!]")),
    ("if let / match bind", re.compile(r"\bif let\s+(Some|Ok)\b|=>\s*$|\bSome\(")),
    ("pushed just above", re.compile(r"\.push\(|\.insert\(")),
    ("filtered", re.compile(r"\.filter\(|\.filter_map\(|\.retain\(")),
)
# A proof only counts if the site can be skipped on the failing branch. That
# is either an exit (`return`/`continue`/`break`/`?`), a short-circuit on the
# site's own line, or — the common one — the proof being the *condition of the
# block the site sits in*, which reads as a line ending in `{`.
EXITS = re.compile(r"\breturn\b|\bcontinue\b|\bbreak\b|\?;|\belse\b|\{\s*$", re.M)


def rust_files():
    seen = set()
    for root in ROOTS:
        for dirpath, _, files in os.walk(os.path.join(BASE, root)):
            for f in files:
                if not f.endswith(".rs"):
                    continue
                p = os.path.relpath(os.path.join(dirpath, f), BASE)
                if any(s in "/" + p for s in SKIP_DIRS):
                    continue
                if p not in seen:
                    seen.add(p)
                    yield p


def main() -> int:
    verbose = "--verbose" in sys.argv
    bare, guarded, locks = [], [], []
    for path in sorted(rust_files()):
        lines = open(os.path.join(BASE, path)).read().split("\n")
        # Everything from a column-0 `#[cfg(test)]` on is test code.
        cut = len(lines)
        for i, l in enumerate(lines):
            if l.startswith("#[cfg(test)]"):
                cut = i
                break

        fns = [(i, m.group(1)) for i, l in enumerate(lines) if (m := FN.match(l))]

        def owner(idx):
            best = "?"
            for s, n in fns:
                if s <= idx:
                    best = n
                else:
                    break
            return best

        for i, l in enumerate(lines[:cut]):
            s = l.strip()
            if s.startswith("//") or not SITE.search(l):
                continue
            entry = (path, i + 1, owner(i), s[:100])
            if LOCK.search(l):
                locks.append(entry)
                continue
            window = lines[max(0, i - LOOKBACK) : i + 1]
            text = "\n".join(window)
            found = [name for name, rx in PROOFS if rx.search(text)]
            # A test alone is not a proof; it has to be able to skip the site.
            if found and (EXITS.search(text) or "||" in l or "&&" in l):
                guarded.append(entry + (found,))
            else:
                bare.append(entry)

    total = len(bare) + len(guarded) + len(locks)
    print(
        f"{total} panicking sites off the bin/test paths: "
        f"{len(guarded)} guarded, {len(locks)} lock-poison, {len(bare)} bare"
    )
    for path, ln, fn, src in bare:
        print(f"    BARE  {path}:{ln}  in {fn}\n          {src}")
    if verbose:
        for path, ln, fn, src, why in guarded:
            print(f"    ok    {path}:{ln}  in {fn}  ({', '.join(why)})")
        for path, ln, fn, src in locks:
            print(f"    lock  {path}:{ln}  in {fn}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
