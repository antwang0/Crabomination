#!/usr/bin/env python3
"""Remove pure-data tests listed on stdin (one `file:line fn_name` per line)
from the source tree. For each victim, delete the `#[test]` attribute,
the `fn name(...) { ... }` body, and the trailing blank line if present.
"""
import sys
from collections import defaultdict
from pathlib import Path

victims_by_file = defaultdict(set)
for raw in sys.stdin:
    raw = raw.strip()
    if not raw or raw.startswith("awk:"):
        continue
    loc, name = raw.split(" ", 1)
    file_path, _line = loc.rsplit(":", 1)
    victims_by_file[file_path].add(name.strip())

def process(path: str, names: set[str]) -> int:
    src = Path(path).read_text(encoding="utf-8")
    lines = src.splitlines(keepends=True)
    out: list[str] = []
    i = 0
    removed = 0
    while i < len(lines):
        line = lines[i]
        # Detect start of a #[test] block.
        if line.lstrip().startswith("#[test]"):
            # Scan ahead for the fn name.
            attr_start = i
            j = i + 1
            while j < len(lines) and not lines[j].lstrip().startswith("fn "):
                j += 1
            if j >= len(lines):
                out.append(line); i += 1; continue
            fn_line = lines[j]
            # Extract function name.
            after_fn = fn_line.lstrip()[3:]  # past "fn "
            paren = after_fn.find("(")
            if paren < 0:
                out.append(line); i += 1; continue
            fn_name = after_fn[:paren].strip()
            if fn_name not in names:
                out.append(line); i += 1; continue
            # Walk forward to find the matching closing brace of the body.
            # The body opens with the first { after the fn signature.
            depth = 0
            seen_open = False
            k = j
            while k < len(lines):
                for ch in lines[k]:
                    if ch == "{":
                        depth += 1; seen_open = True
                    elif ch == "}":
                        depth -= 1
                if seen_open and depth == 0:
                    break
                k += 1
            # Skip lines [attr_start, k] inclusive. Also drop one trailing
            # blank line so we don't leave double blank gaps.
            end = k + 1
            if end < len(lines) and lines[end].strip() == "":
                end += 1
            i = end
            removed += 1
        else:
            out.append(line); i += 1
    Path(path).write_text("".join(out), encoding="utf-8")
    return removed

total = 0
for path, names in victims_by_file.items():
    n = process(path, names)
    print(f"{path}: removed {n}/{len(names)} tests")
    total += n
print(f"total: {total}")
