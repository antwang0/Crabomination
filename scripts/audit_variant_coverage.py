#!/usr/bin/env python3
"""Cross-reference the base crate's capability enums against the catalog and
the engine, in both directions.

Two questions `audit_stubs` and `audit_incomplete`'s structural pass cannot
ask, because both look *inside* one card's effect tree:

  DEAD CAPABILITY  a variant that shipped cards use and no engine code names
                   outside a no-op `match` arm. The card resolves, the tree is
                   not empty, and the ability does nothing — invisible to a
                   structural audit and to the type checker alike, because an
                   exhaustive `match` is satisfied by a `A | B | C => {}` arm.
  DEAD PRIMITIVE   a variant the engine implements that no catalog card uses.
                   Not a bug; a maintenance cost and a hint that a card batch
                   was meant to land and did not.

The no-op arms are discovered, not hardcoded: any `match` arm whose body is
`{}`, `()` or `Ok(())` has its pattern's variant names collected, and mentions
inside those line ranges do not count as an implementation. That is what makes
the filter stronger than the compiler — exhaustiveness is satisfied by exactly
those arms.

Reading at 2026-08-27, over 471 + 987 + 237 = 1,695 variants: **zero dead
capabilities**, and three dead primitives (`Effect::AddRadCounters`,
`ExileTopAndMayCastUpToMv`, `GrantCastBackFromGraveyard`) — each an
implemented effect waiting for the card that wanted it.

Run: python3 scripts/audit_variant_coverage.py           # ~40 s
     python3 scripts/audit_variant_coverage.py --verbose # + the no-op arms
Exit status is 1 only on a dead *capability*; a dead primitive is
informational, so this can gate a run.
"""

import os
import re
import subprocess
import sys

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# (file the enum is declared in, enum name)
ENUMS = [
    ("crabomination_base/src/effect/abilities.rs", "StaticEffect"),
    ("crabomination_base/src/effect.rs", "Effect"),
    ("crabomination_base/src/card.rs", "Keyword"),
]

ENGINE_DIRS = [
    "crabomination/src",
    "crabomination_server/src",
    "crabomination_client/src",
    "crabomination_ml/src",
]
CATALOG_DIRS = ["crabomination_catalog/src"]
BASE_DIRS = ["crabomination_base/src"]
TEST_DIRS = ["crabomination_tests/tests"]

# `use Effect as E;`-style aliases the engine actually uses.
QUALIFIER = r"(?:Effect|StaticEffect|Keyword|SE|S|E|K)"
NOOP_BODY = re.compile(r"=>\s*(?:\{\s*\}|Ok\(\(\)\)|\(\))\s*,?\s*$")
# A mention in one of these positions is building the value, not matching it.
# Deliberately generous — `, ` catches both a `vec![a, Variant]` element and a
# `matches!(x, Variant)` pattern, and over-counting constructions only makes
# the *dead primitive* half under-report, which is the harmless direction.
RVALUE_BEFORE = re.compile(r"(=\s*|push\(|Some\(|Box::new\(|vec!\[|return\s+|:\s*|,[ \t]*)$")
BINDING_ONLY = re.compile(r"(\.\.|_|[a-z_][A-Za-z0-9_]*)")


def variants_of(path, enum_name):
    """Variant names of `enum_name`, read off the declaration."""
    src = open(os.path.join(BASE, path)).read()
    m = re.search(r"pub enum %s(?:<[^>]*>)?\s*\{" % re.escape(enum_name), src)
    if not m:
        raise SystemExit(f"enum {enum_name} not found in {path}")
    depth, j = 1, m.end()
    while depth:
        if src[j] == "{":
            depth += 1
        elif src[j] == "}":
            depth -= 1
        j += 1
    body = src[m.end() : j - 1]
    return sorted(set(re.findall(r"^    ([A-Z][A-Za-z0-9_]*)\s*(?:\{|\(|,|=)", body, re.M)))


def noop_ranges():
    """(path, first_line, last_line) for every no-op `match` arm in the engine.

    An arm's pattern can run to hundreds of `|`-joined lines (the layer pass's
    "these statics are not continuous effects" arm is 780), so walk back over
    continuation and comment lines from the `=> {}`.
    """
    out = []
    for root in ENGINE_DIRS:
        for dirpath, _, files in os.walk(os.path.join(BASE, root)):
            for f in files:
                if not f.endswith(".rs"):
                    continue
                p = os.path.relpath(os.path.join(dirpath, f), BASE)
                lines = open(os.path.join(BASE, p)).read().split("\n")
                for i, line in enumerate(lines):
                    if not NOOP_BODY.search(line):
                        continue
                    j = i
                    while j > 0 and (
                        lines[j].strip().startswith("|")
                        or lines[j - 1].rstrip().endswith("|")
                        or lines[j].strip().startswith("//")
                    ):
                        j -= 1
                    if re.search(QUALIFIER + r"::[A-Z]", "\n".join(lines[j : i + 1])):
                        out.append((p, j + 1, i + 1))
    return out


def constructed_variants(dirs, names):
    """Of `names`, the variants that appear somewhere in *construction*
    position rather than only as a `match` pattern.

    A dead primitive is one nothing builds. Mentions alone cannot tell: the
    resolver's arm for a variant no card uses reads exactly like the arm for
    one every card uses. Three shapes say "built": a struct literal with a
    field initializer (`{ who: … }` — a pattern uses shorthand), a tuple
    argument that is not a plain binding, and a unit variant sitting in an
    rvalue position.
    """
    found = set()
    pat = re.compile(QUALIFIER + r"::([A-Z][A-Za-z0-9_]*)")
    for root in dirs:
        for dirpath, _, files in os.walk(os.path.join(BASE, root)):
            for f in files:
                if not f.endswith(".rs"):
                    continue
                text = open(os.path.join(dirpath, f)).read()
                # A doc comment naming a variant is not a construction.
                text = re.sub(r"^\s*//.*$", "", text, flags=re.M)
                for m in pat.finditer(text):
                    v = m.group(1)
                    if v not in names or v in found:
                        continue
                    i = m.end()
                    while i < len(text) and text[i] in " \n\t":
                        i += 1
                    ch = text[i] if i < len(text) else ""
                    if ch in "{(":
                        close = "}" if ch == "{" else ")"
                        depth, j = 1, i + 1
                        while depth and j < len(text):
                            if text[j] == ch:
                                depth += 1
                            elif text[j] == close:
                                depth -= 1
                            j += 1
                        inner = text[i + 1 : j - 1].strip()
                        built = (
                            re.search(r"[A-Za-z0-9_]\s*:", inner)
                            if ch == "{"
                            else (inner and not BINDING_ONLY.fullmatch(inner))
                        )
                        if built:
                            found.add(v)
                            continue
                    if RVALUE_BEFORE.search(text[max(0, m.start() - 60) : m.start()]):
                        found.add(v)
    return found


def rg(pattern_args):
    return subprocess.run(
        ["rg", "--no-heading"] + pattern_args, capture_output=True, text=True, cwd=BASE
    ).stdout


def main():
    verbose = "--verbose" in sys.argv
    noops = noop_ranges()
    if verbose:
        print(f"no-op arms found: {len(noops)}")
        for p, a, b in noops:
            print(f"  {p}:{a}-{b}")

    findings = 0
    for path, name in ENUMS:
        vs = variants_of(path, name)
        built = constructed_variants(ENGINE_DIRS + CATALOG_DIRS + BASE_DIRS + TEST_DIRS, set(vs))
        dead_capability, dead_primitive = [], []
        for v in vs:
            in_catalog = bool(rg(["-l", "-w", v] + CATALOG_DIRS).strip())
            hits = rg(["-n", "-w", v] + ENGINE_DIRS)
            implemented = False
            for line in hits.split("\n"):
                if not line:
                    continue
                f, ln, rest = line.split(":", 2)
                ln = int(ln)
                if rest.strip().startswith("//"):
                    continue  # a doc comment is not an implementation
                if any(f == p and a <= ln <= b for p, a, b in noops):
                    continue  # a no-op arm is not an implementation
                implemented = True
                break
            if in_catalog and not implemented:
                dead_capability.append(v)
            elif not in_catalog and v not in built:
                dead_primitive.append(v)
        findings += len(dead_capability)
        print(
            f"{name}: {len(vs)} variants — "
            f"{len(dead_capability)} dead capability, {len(dead_primitive)} dead primitive"
        )
        for v in dead_capability:
            print(f"    DEAD CAPABILITY  {name}::{v}  (on shipped cards, no engine arm)")
        for v in dead_primitive:
            print(f"    dead primitive   {name}::{v}  (implemented, nothing builds it)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
