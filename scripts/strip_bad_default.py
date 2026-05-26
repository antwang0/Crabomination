import re
import subprocess

out = subprocess.run(
    ["cargo", "check", "--message-format=short"],
    capture_output=True, text=True,
).stderr

# E0436 errors look like:
#    crabomination\src\catalog\sets\decks\modern.rs:9248:23: error[E0436]: ...
# (with backslashes on Windows)
bad = []
for line in out.splitlines():
    m = re.match(r"^(\S+\.rs):(\d+):(\d+): error\[E0436\]:", line)
    if m:
        path = m.group(1).replace("\\", "/")
        ln = int(m.group(2))
        bad.append((path, ln))

print(f"Found {len(bad)} E0436 lines to strip")

by_file = {}
for p, ln in bad:
    by_file.setdefault(p, set()).add(ln)

for p, lns in by_file.items():
    with open(p, encoding="utf-8") as f:
        lines = f.read().splitlines(keepends=True)
    removed = 0
    for ln in sorted(lns, reverse=True):
        if ln - 1 < len(lines) and "..Default::default()" in lines[ln - 1]:
            del lines[ln - 1]
            removed += 1
    with open(p, "w", encoding="utf-8") as f:
        f.writelines(lines)
    print(f"{p}: removed {removed} lines")
