#!/usr/bin/env python3
"""Strip catalog-tautology and duplicate smoke tests from tests/stx.rs.

Categories removed:
  1. No `perform_action` call AND <=10 body lines AND <=3 asserts.
     (catalog roundtrip / keyword smoke tests)
  2. Subset of #1 (short has_keyword smoke tests).
  3. Tests sharing a name stem (stripped of `_bN` / `_bN_`) where the
     stem has 2+ copies — keep the first occurrence, delete the rest.
"""
import re
import sys
from collections import defaultdict
from pathlib import Path

SRC = Path("crabomination/src/tests/stx.rs")

lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)
n = len(lines)


def find_tests():
    """Yield (block_start, end_inclusive, name, body_text) for each test.

    block_start spans any preceding /// doc comments and attributes attached
    to the #[test]. end_inclusive is the line index of the closing `}`.
    """
    out = []
    i = 0
    while i < n:
        if lines[i].rstrip() != "#[test]":
            i += 1
            continue

        # Walk back over attributes / doc comments / cfg gates attached
        # to this test. Stop at a blank line, comment header, or code.
        block_start = i
        while block_start > 0:
            prev = lines[block_start - 1].rstrip()
            if (
                prev.startswith("///")
                or prev.startswith("//!")
                or (prev.startswith("#[") and prev != "#[test]")
            ):
                block_start -= 1
            else:
                break

        # Walk forward to fn NAME(
        j = i + 1
        name = None
        while j < n:
            m = re.match(r"fn ([a-z0-9_]+)", lines[j])
            if m:
                name = m.group(1)
                break
            j += 1
        if name is None:
            i += 1
            continue

        # Closing brace at column 0
        k = j + 1
        while k < n:
            if lines[k].rstrip() == "}":
                break
            k += 1
        if k >= n:
            i += 1
            continue

        body = "".join(lines[j + 1 : k])
        out.append((block_start, k, name, body))
        i = k + 1
    return out


tests = find_tests()
print(f"Total tests parsed: {len(tests)}", file=sys.stderr)


def is_cat1(body: str) -> bool:
    """No engine action, short, few asserts → catalog-roundtrip smoke test."""
    if "perform_action" in body or "resolve_combat" in body:
        return False
    if body.count("\n") > 10:
        return False
    if len(re.findall(r"\bassert", body)) > 3:
        return False
    return True


def stem(name: str) -> str:
    s = re.sub(r"_b\d+_", "_", name)
    s = re.sub(r"_b\d+$", "", s)
    return s


cat1_ids = {t[0] for t in tests if is_cat1(t[3])}

by_stem = defaultdict(list)
for t in tests:
    by_stem[stem(t[2])].append(t)

# Cat3: for each stem with >=2 tests, keep the first (lowest block_start), delete rest.
cat3_ids = set()
for s, group in by_stem.items():
    if len(group) >= 2:
        group_sorted = sorted(group, key=lambda t: t[0])
        for t in group_sorted[1:]:
            cat3_ids.add(t[0])

delete_ids = cat1_ids | cat3_ids
print(f"Cat1 deletions (smoke tests): {len(cat1_ids)}", file=sys.stderr)
print(f"Cat3 deletions (duplicate stems): {len(cat3_ids)}", file=sys.stderr)
print(f"Cat3 only (not also cat1): {len(cat3_ids - cat1_ids)}", file=sys.stderr)
print(f"Union (unique tests to delete): {len(delete_ids)}", file=sys.stderr)
print(f"Tests remaining: {len(tests) - len(delete_ids)}", file=sys.stderr)

# Build line-index set to drop. Include the test's block_start..=end
# AND one trailing blank line if present (to avoid double-blank pileups).
drop = set()
for start, end, name, body in tests:
    if start not in delete_ids:
        continue
    for ln in range(start, end + 1):
        drop.add(ln)
    # Eat one trailing blank line so consecutive deletions don't pile up.
    if end + 1 < n and lines[end + 1].strip() == "":
        drop.add(end + 1)

# Rebuild file, collapsing >1 consecutive blank line to 1.
out_lines = []
prev_blank = False
for idx, ln in enumerate(lines):
    if idx in drop:
        continue
    blank = ln.strip() == ""
    if blank and prev_blank:
        continue
    out_lines.append(ln)
    prev_blank = blank

new_text = "".join(out_lines)
SRC.write_text(new_text, encoding="utf-8")
print(f"Wrote {SRC} ({len(out_lines)} lines, was {n})", file=sys.stderr)
