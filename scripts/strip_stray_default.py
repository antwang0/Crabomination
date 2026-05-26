"""Remove stray `..Default::default()` lines that appear after a `}`
but before the function's closing `}`, where they're outside any
struct literal."""

import os

files = []
for root, _dirs, fnames in os.walk("crabomination/src"):
    for fn in fnames:
        if fn.endswith(".rs"):
            files.append(os.path.join(root, fn).replace("\\", "/"))

total_removed = 0
for path in files:
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines(keepends=True)
    new_lines = []
    removed = 0
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        # Pattern: `..Default::default()` line where the previous
        # non-blank line is exactly `}` (no comma, no other content).
        # That's outside any struct.
        if stripped == "..Default::default()" or stripped == "..Default::default(),":
            # Look at the previous non-blank line in new_lines.
            j = len(new_lines) - 1
            while j >= 0 and new_lines[j].strip() == "":
                j -= 1
            if j >= 0 and new_lines[j].strip() == "}":
                # Stray — drop it.
                removed += 1
                i += 1
                continue
        new_lines.append(line)
        i += 1
    if removed:
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(new_lines)
        print(f"{path}: removed {removed} stray ..Default::default() lines")
        total_removed += removed

print(f"TOTAL: {total_removed} stray defaults removed")
