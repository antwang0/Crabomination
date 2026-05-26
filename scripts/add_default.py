"""One-shot script: add `..Default::default()` to every struct initializer
that's currently triggering an E0063 missing-fields error. Run cargo check
to get the location list, then patch each struct."""

import re
import subprocess
import sys


def main():
    out = subprocess.run(
        ["cargo", "check", "--message-format=short"],
        capture_output=True,
        text=True,
        cwd=".",
    ).stderr
    locations = []
    for line in out.splitlines():
        m = re.match(r"^(crabomination\\src\\[^:]+):(\d+):(\d+): error\[E0063\]:", line)
        if m:
            path = m.group(1).replace("\\", "/")
            ln = int(m.group(2))
            col = int(m.group(3))
            locations.append((path, ln, col))
    print(f"Found {len(locations)} E0063 errors")

    by_file = {}
    for path, ln, col in locations:
        by_file.setdefault(path, []).append((ln, col))

    def find_struct_close(lines, start_line_idx, start_col_idx):
        depth = 0
        started = False
        for i in range(start_line_idx, len(lines)):
            cs = start_col_idx if i == start_line_idx else 0
            for j in range(cs, len(lines[i])):
                ch = lines[i][j]
                if ch == "{":
                    depth += 1
                    started = True
                elif ch == "}":
                    depth -= 1
                    if started and depth == 0:
                        return i
        return None

    for path, locs in by_file.items():
        with open(path, encoding="utf-8") as f:
            lines = f.read().splitlines(keepends=True)
        locs.sort(key=lambda x: -x[0])
        edited = 0
        for ln, col in locs:
            line_idx = ln - 1
            line = lines[line_idx]
            brace_idx = line.find("{", col - 1)
            if brace_idx < 0:
                # check next few lines
                for k in range(ln, min(ln + 3, len(lines))):
                    if "{" in lines[k]:
                        line_idx = k
                        brace_idx = lines[k].find("{")
                        break
            if brace_idx < 0:
                print(f"WARN: couldn't find opening brace for {path}:{ln}")
                continue
            close_idx = find_struct_close(lines, line_idx, brace_idx + 1)
            if close_idx is None:
                print(f"WARN: couldn't find closing brace for {path}:{ln}")
                continue
            snippet = "".join(lines[line_idx : close_idx + 1])
            if "..Default::default()" in snippet:
                continue
            close_line = lines[close_idx]
            indent = close_line[: len(close_line) - len(close_line.lstrip())]
            lines.insert(close_idx, indent + "    ..Default::default(),\n")
            edited += 1
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(lines)
        print(f"{path}: inserted {edited} ..Default::default() lines")


if __name__ == "__main__":
    main()
