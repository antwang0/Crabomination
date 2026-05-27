"""Remove cross-file duplicate function definitions."""

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


def delete_from(path, names):
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines(keepends=True)
    spans = []
    for name in names:
        pattern = f"pub fn {name}() -> CardDefinition {{"
        for i, line in enumerate(lines):
            if line.lstrip().startswith(pattern):
                end = find_fn_span(lines, i)
                if end is None:
                    continue
                start = i
                while start > 0 and (
                    lines[start - 1].lstrip().startswith("///")
                    or lines[start - 1].strip() == ""
                    or lines[start - 1].lstrip().startswith("//")
                ):
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
        print(f"  {path}: removed {name}")
    if spans:
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(lines)


# Remove from decks/modern.rs (mod_set/* has the canonical versions)
delete_from("crabomination/src/catalog/sets/decks/modern.rs", [
    "descendant_of_storms",
    "elder_gargaroth",
    "explore",
    "gush",
    "intervention_pact",
])

# Remove from stx/extras.rs (mod_set has the canonical version)
delete_from("crabomination/src/catalog/sets/stx/extras.rs", [
    "elite_spellbinder",
])
