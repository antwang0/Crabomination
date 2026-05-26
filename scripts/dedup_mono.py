"""Remove duplicate card functions from `stx/mono.rs` when they're
defined in their college-specific module (silverquill, quandrix, etc.)."""

import os
import re


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


NAMES = ["fractal_summoning", "clever_lumimancer", "silverquill_silencer"]

path = "crabomination/src/catalog/sets/stx/mono.rs"
with open(path, encoding="utf-8") as f:
    lines = f.read().splitlines(keepends=True)
spans = []
for name in NAMES:
    pattern = f"pub fn {name}() -> CardDefinition {{"
    for i, line in enumerate(lines):
        if line.lstrip().startswith(pattern):
            end = find_fn_span(lines, i)
            start = i
            while start > 0 and (lines[start - 1].lstrip().startswith("///") or lines[start - 1].strip() == ""):
                start -= 1
                if start == 0:
                    break
                if lines[start - 1].rstrip().endswith("}"):
                    break
            spans.append((start, end, name))
            break
spans.sort(reverse=True)
for start, end, name in spans:
    del lines[start:end]
    print(f"removed {name} from mono.rs")
with open(path, "w", encoding="utf-8") as f:
    f.writelines(lines)
