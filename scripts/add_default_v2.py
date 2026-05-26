"""Add `..Default::default()` to struct literals missing fields, using
the struct type from the error message to disambiguate which `{` to
target. Parses --message-format=json so we get the type name reliably."""

import json
import re
import subprocess


def main():
    proc = subprocess.run(
        ["cargo", "check", "--message-format=json", "-q"],
        capture_output=True, text=True, cwd=".",
    )
    edits = {}  # path -> list of (open_line_idx, close_line_idx)
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        msg = msg.get("message") if isinstance(msg, dict) else None
        if not msg:
            continue
        code = msg.get("code") or {}
        if code.get("code") != "E0063":
            continue
        text = msg.get("message", "")
        m = re.search(r"initializer of `([A-Za-z_][A-Za-z0-9_]*)`", text)
        if not m:
            continue
        struct_name = m.group(1)
        # The "primary" span starts at the struct-name identifier in the source.
        for span in msg.get("spans", []):
            if not span.get("is_primary"):
                continue
            path = span["file_name"].replace("\\", "/")
            ln = span["line_start"]
            col = span["column_start"]
            edits.setdefault(path, []).append((ln, col, struct_name))
            break

    def find_struct_close(lines, line_idx, col_idx):
        depth = 0
        started = False
        for i in range(line_idx, len(lines)):
            cs = col_idx if i == line_idx else 0
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

    total = 0
    for path, locs in edits.items():
        try:
            with open(path, encoding="utf-8") as f:
                lines = f.read().splitlines(keepends=True)
        except FileNotFoundError:
            continue
        # Sort descending so edits don't shift later targets.
        locs.sort(key=lambda x: -x[0])
        edited = 0
        for ln, col, struct_name in locs:
            # The span often points at the inner brace expression; we
            # need the outer struct of `struct_name`. Walk back up to
            # find `<struct_name> {`.
            target_line = ln - 1
            target_col = col - 1
            found = False
            # Walk backward through up to 80 lines looking for the
            # `<struct_name> {` opener. On the error line, search the
            # whole line (the error column might point right at the
            # struct identifier).
            needle = f"{struct_name} {{"
            for k in range(target_line, max(target_line - 80, -1), -1):
                idx = lines[k].rfind(needle)
                if idx >= 0:
                    target_line = k
                    target_col = idx + len(struct_name) + 1
                    found = True
                    break
            if not found:
                continue
            close_idx = find_struct_close(lines, target_line, target_col)
            if close_idx is None:
                continue
            # Skip if the OUTER struct already ends with ..Default::default().
            # Inner nested structs (e.g. Subtypes inside CardDefinition) can
            # also have it; we want to confirm the outer one needs it.
            prev_line = lines[close_idx - 1].strip() if close_idx > 0 else ""
            if prev_line == "..Default::default()" or prev_line == "..Default::default(),":
                continue
            close_line = lines[close_idx]
            indent = close_line[: len(close_line) - len(close_line.lstrip())]
            lines.insert(close_idx, indent + "    ..Default::default()\n")
            edited += 1
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(lines)
        total += edited
        print(f"{path}: inserted {edited} ..Default::default() lines")
    print(f"TOTAL: {total} insertions")


if __name__ == "__main__":
    main()
