"""Dedup catalog functions: when the same `pub fn NAME() -> CardDefinition`
is defined in `stx/extras.rs` AND another sibling file, delete the one
in `extras.rs`. The newer modular files (`silverquill.rs`, `mono.rs`,
etc.) are the canonical home; `extras.rs` holds older bulk additions."""

import os
import re
import sys

NAMES = """academic_dispute
adventurous_impulse
clever_lumimancer
conspiracy_theorist
dragonsguard_elite
elemental_summoning
environmental_sciences
eureka_moment
fractal_summoning
humiliate
introduction_to_annihilation
introduction_to_prophecy
lorehold_command
prismari_command
quandrix_command
returned_pastcaller
rip_apart
silverquill_apprentice
silverquill_command
silverquill_silencer
spirit_summoning
teach_by_example
witherbloom_command""".strip().splitlines()


def find_fn_span(lines, start_idx):
    depth = 0
    started = False
    for i in range(start_idx, len(lines)):
        for ch in lines[i]:
            if ch == "{":
                depth += 1
                started = True
            elif ch == "}":
                depth -= 1
                if started and depth == 0:
                    return i + 1
    return None


# Files where duplicates appear besides extras.rs / lessons.rs — the
# canonical (keeper) homes.
extras_files = [
    "crabomination/src/catalog/sets/stx/extras.rs",
    "crabomination/src/catalog/sets/stx/lessons.rs",
]

total = 0
for path in extras_files:
    if not os.path.exists(path):
        continue
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines(keepends=True)
    # Collect spans to delete (descending so deletes don't shift later)
    spans_to_delete = []
    for name in NAMES:
        pattern = f"pub fn {name}() -> CardDefinition {{"
        for i, line in enumerate(lines):
            if line.lstrip().startswith(pattern):
                end = find_fn_span(lines, i)
                if end is None:
                    continue
                # Walk back past doc comments
                start = i
                while start > 0 and (lines[start - 1].lstrip().startswith("///") or lines[start - 1].strip() == ""):
                    start -= 1
                    if start == 0:
                        break
                    if lines[start - 1].rstrip().endswith("}"):
                        break
                spans_to_delete.append((start, end, name))
                break
    spans_to_delete.sort(reverse=True)
    removed = 0
    for start, end, name in spans_to_delete:
        del lines[start:end]
        removed += 1
        print(f"  removed {name}")
    if removed:
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(lines)
        print(f"{path}: {removed} removed")
        total += removed
print(f"TOTAL: {total} removed")
