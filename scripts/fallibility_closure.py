#!/usr/bin/env python3
"""Which engine functions can an `Err` actually come out of, reachable from X?

The forty-fourth pass used this to size `perform_action`'s transaction
checkpoint (PERF candidate (-13)). Reading the *signatures* says
`pass_priority` is uncapturable — it mutates, then propagates from three
places. Reading the *closure* says something else: 46 of the engine's ~150
`Result`-returning functions are reachable from `pass_priority` and exactly
five raise at all, none of them in the step machinery. That is what made the
round-closing pass's checkpoint skip provable.

The same run over the other checkpointed action shapes is what sizes the rest
of (-13), and it says the answer is no for most of them: `play_land` reaches
6 and 2 raise; `submit_decision` reaches 137 and **70** raise. A shape with
seventy raisers is not going to be proven arm by arm — it keeps its
checkpoint.

    python3 scripts/fallibility_closure.py pass_priority
    python3 scripts/fallibility_closure.py cast_spell activate_ability

Deliberately over-approximates the call graph: any call to a
`Result`-returning function is an edge, whether or not it carries `?` (a tail
`return self.f(x)` propagates too). Under-approximating would hide a raiser,
which is the direction that matters. Name-based, so two functions with one
name merge — check the reported file before trusting a single hit.

Output is the reachable set's size and, per raising function, the file and how
many `Err(` / `ok_or` sites it has. Read those sites; the question is never
"does it raise" but "can it raise *after* it has written".
"""

import collections
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent / "crabomination" / "src"

FN_RE = re.compile(r"\bfn\s+([a-zA-Z_0-9]+)\s*(?:<[^>]*>)?\s*\(")
RESULT_RE = re.compile(r"->\s*Result<")


def fn_spans(text):
    """(name, signature, body_start, body_end) for every fn with a body."""
    out = []
    for m in FN_RE.finditer(text):
        i, depth = m.end() - 1, 0
        while i < len(text):
            if text[i] == "(":
                depth += 1
            elif text[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        j = i
        while j < len(text) and text[j] not in "{;":
            j += 1
        if j >= len(text) or text[j] == ";":
            continue
        k, d = j, 0
        while k < len(text):
            if text[k] == "{":
                d += 1
            elif text[k] == "}":
                d -= 1
                if d == 0:
                    break
            k += 1
        out.append((m.group(1), text[m.start() : j], j, k))
    return out


def main():
    roots = sys.argv[1:] or ["pass_priority"]
    fns = {}
    for p in sorted(ROOT.rglob("*.rs")):
        if "/bin/" in str(p):
            continue
        text = p.read_text(errors="replace")
        for name, sig, s, e in fn_spans(text):
            fns.setdefault(name, []).append((str(p.relative_to(ROOT)), sig, text[s:e]))

    result_fns = {n for n, i in fns.items() if any(RESULT_RE.search(s) for _, s, _ in i)}

    edges = collections.defaultdict(set)
    raisers = {}
    for n in result_fns:
        for f, sig, body in fns[n]:
            if not RESULT_RE.search(sig):
                continue
            clean = re.sub(r"//[^\n]*", "", body)
            for m in re.finditer(r"([a-zA-Z_0-9]+)\s*\(", clean):
                callee = m.group(1)
                if callee in result_fns and callee != n:
                    edges[n].add(callee)
            errs = len(re.findall(r"Err\(", clean))
            oks = len(re.findall(r"ok_or", clean))
            if errs or oks:
                raisers.setdefault(n, []).append((f, errs, oks))

    seen, stack = set(), list(roots)
    while stack:
        n = stack.pop()
        if n in seen:
            continue
        seen.add(n)
        stack += [c for c in edges.get(n, ()) if c not in seen]

    print(f"{len(result_fns)} Result-returning fns in the engine")
    print(f"{len(seen)} reachable from {roots}")
    hits = [n for n in sorted(seen) if n in raisers]
    print(f"{len(hits)} of those raise:")
    for n in hits:
        for f, errs, oks in raisers[n]:
            print(f"  {n:<40} {f:<24} Err( x{errs}  ok_or x{oks}")


if __name__ == "__main__":
    main()
