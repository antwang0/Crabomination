"""Add `x_value: None,` to ActivateAbility struct literals missing it.
Uses a state machine to find the closing `}` of each missing struct."""

import json
import re
import subprocess


def main():
    proc = subprocess.run(
        ["cargo", "test", "-p", "crabomination", "--no-run",
         "--message-format=json"],
        capture_output=True, text=True, cwd=".",
    )
    edits = {}
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
        if "x_value" not in text:
            continue
        for span in msg.get("spans", []):
            if not span.get("is_primary"):
                continue
            path = span["file_name"].replace("\\", "/")
            path = re.sub(r"^crabomination/src/game/\.\./", "crabomination/src/", path)
            ln = span["line_start"]
            edits.setdefault(path, set()).add(ln)
            break

    for path, lns in edits.items():
        with open(path, encoding="utf-8") as f:
            lines = f.read().splitlines(keepends=True)
        for ln in sorted(lns, reverse=True):
            # Walk forward from ln looking for the `}` matching the
            # opening `{` of the struct literal.
            depth = 0
            started = False
            close_idx = None
            for i in range(ln - 1, len(lines)):
                for ch in lines[i]:
                    if ch == "{":
                        depth += 1
                        started = True
                    elif ch == "}":
                        depth -= 1
                        if started and depth == 0:
                            close_idx = i
                            break
                if close_idx is not None:
                    break
            if close_idx is None:
                print(f"WARN: no close for {path}:{ln}")
                continue
            # Insert `x_value: None,` on the line before close_idx,
            # with the indentation of the previous line's content.
            prev_line = lines[close_idx - 1]
            stripped = prev_line.lstrip()
            indent = prev_line[: len(prev_line) - len(stripped)]
            # If close brace is on same line as a field (e.g. `}).expect(...)`),
            # we need a different approach — insert into prev line.
            close_line = lines[close_idx]
            if close_line.lstrip().startswith("}"):
                lines.insert(close_idx, indent + "x_value: None,\n")
            else:
                # mixed line e.g. `target: Some(...) }).expect(...)`
                lines[close_idx] = re.sub(r"\s*}", " x_value: None }", close_line, count=1)
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(lines)
        print(f"{path}: patched {len(lns)} ActivateAbility calls")


if __name__ == "__main__":
    main()
