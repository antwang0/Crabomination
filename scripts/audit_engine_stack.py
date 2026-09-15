#!/usr/bin/env python3
"""Every production thread that resolves effects is built with the engine's
stack — the class check behind `server::ENGINE_STACK_BYTES`.

A resolution recurses through trigger / copy / replacement chains, and the
default stack (8 MB main, 2 MB spawned) does not hold them in an unoptimized
build. `RUST_MIN_STACK` only reaches a process cargo launched, so a thread the
tree spawns itself has to state the size. Every binary's game threads already
did; **three in the server did not** (the lobby's match driver and
`crabomination_server`'s bot-match and pair-match threads), and a stack
overflow is a `SIGSEGV` with no message — strictly worse than the panic class
`audit_panics.py` chases, because nothing prints and nothing unwinds.

    python3 scripts/audit_engine_stack.py            # 0 = clean
    python3 scripts/audit_engine_stack.py --list     # every site it classified

What it flags: a bare `thread::spawn(` in non-test code whose closure body, or
the function it immediately tail-calls, names one of the engine entry points
below. `thread::Builder::new()` sites are read for a `stack_size(` within the
builder chain. Test modules are skipped — a `#[cfg(test)]` thread that drives a
two-card fixture is not the shape this exists for, and the suite would fail
loudly rather than silently if it were.

⚠ The entry-point list is a *name* list, so a new match runner needs a line
here. That is the same bargain every auditor in `scripts/` makes, and the
alternative (following call graphs out of a grep) is the thing this is meant to
be cheaper than.
"""

import os
import re
import sys

ROOTS = [
    "crabomination/src",
    "crabomination_server/src",
    "crabomination_ml/src",
]

# A closure naming one of these runs the engine's resolution loop.
ENGINE_ENTRY = [
    "run_match",
    "run_bot_match",
    "run_pair_match",
    "run_match_reconnectable",
    "play_one_game",
    "next_action",
    "actor_loop",
    "simulate_match",
    "parallel_spend",
]

SPAWN = re.compile(r"\bthread::spawn\s*\(")
BUILDER = re.compile(r"\bthread::Builder::new\s*\(\s*\)")
STACK = re.compile(r"\bstack_size\s*\(")


def test_module_start(src):
    """Line index (0-based) where the file's test *module* begins, or None.

    ⚠ Not "the first `#[cfg(test)]`": `crabomination_server/src/main.rs`
    carries one at line 50 on a helper, and cutting there hid both of the
    threads this auditor exists for. The cut is a `#[cfg(test)]` whose next
    non-blank line opens a `mod`.
    """
    for i, line in enumerate(src):
        if not line.strip().startswith("#[cfg(test)]"):
            continue
        for nxt in src[i + 1 :]:
            if not nxt.strip():
                continue
            if nxt.strip().startswith("mod "):
                return i
            break
    return None


def scan(path, listing):
    src = open(path).read().split("\n")
    cut = test_module_start(src)
    findings = []
    for i, line in enumerate(src):
        if cut is not None and i >= cut:
            break
        is_spawn = SPAWN.search(line)
        is_builder = BUILDER.search(line)
        if not (is_spawn or is_builder):
            continue
        # The closure body: to the end of the statement, capped so a long
        # thread body does not drag an unrelated call in.
        body = "\n".join(src[i : i + 14])
        entry = next((e for e in ENGINE_ENTRY if e in body), None)
        if entry is None:
            continue
        # A builder chain states the size in the same statement; a bare spawn
        # never can.
        sized = bool(is_builder and STACK.search(body))
        rel = path
        if listing:
            print(f"  {'ok  ' if sized else 'BARE'} {rel}:{i + 1}  ({entry})")
        if not sized:
            findings.append((rel, i + 1, entry))
    return findings


def main():
    listing = "--list" in sys.argv
    here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    os.chdir(here)
    findings = []
    for root in ROOTS:
        for dirpath, _, files in os.walk(root):
            for f in sorted(files):
                if f.endswith(".rs"):
                    findings += scan(os.path.join(dirpath, f), listing)
    if findings:
        print(f"\n{len(findings)} engine thread(s) on the DEFAULT stack:")
        for rel, line, entry in findings:
            print(f"  {rel}:{line}  runs {entry}() with no stack_size")
        print(
            "\nBuild them with `thread::Builder::new()"
            ".stack_size(server::ENGINE_STACK_BYTES)` and handle the "
            "`spawn` Result — a failed spawn means the closure's guards were "
            "never constructed."
        )
        return 1
    print("0 engine threads on the default stack")
    return 0


if __name__ == "__main__":
    sys.exit(main())
