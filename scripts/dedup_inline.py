"""Dedup `pub fn NAME() -> CardDefinition` definitions that appear
twice in the same file (a common merge artifact). Keeps the FIRST
occurrence."""

import sys


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


def dedup(path, names):
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines(keepends=True)
    spans_to_delete = []
    for name in names:
        pattern = f"pub fn {name}() -> CardDefinition {{"
        hits = []
        for i, line in enumerate(lines):
            if line.lstrip().startswith(pattern):
                hits.append(i)
        # Keep first hit, delete the rest (including doc comments above)
        for hit in hits[1:]:
            end = find_fn_span(lines, hit)
            if end is None:
                continue
            start = hit
            while start > 0 and (lines[start - 1].lstrip().startswith("///") or lines[start - 1].strip() == "" or lines[start - 1].lstrip().startswith("//")):
                start -= 1
                if start == 0:
                    break
                if lines[start - 1].rstrip().endswith("}"):
                    break
            spans_to_delete.append((start, end, name))
    spans_to_delete.sort(reverse=True)
    for start, end, name in spans_to_delete:
        del lines[start:end]
        print(f"  removed duplicate {name} (lines {start+1}..{end})")
    if spans_to_delete:
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(lines)


if __name__ == "__main__":
    targets = {
        "crabomination/src/catalog/sets/sos/mdfcs.rs": ["campus_composer", "emeritus_of_ideation", "grave_researcher"],
        "crabomination/src/catalog/sets/sos/sorceries.rs": ["fix_whats_broken", "molten_note"],
        "crabomination/src/catalog/sets/mod_set/creatures.rs": ["guardian_scalelord"],
    }
    for path, names in targets.items():
        print(path)
        dedup(path, names)
