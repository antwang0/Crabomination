"""Add `additional_targets: vec![]` to GameAction::CastSpell struct
literals missing it. Parse cargo error output for missing-field
locations, then patch each."""

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
        m = re.search(r"missing field `additional_targets` in initializer", text)
        if not m:
            continue
        for span in msg.get("spans", []):
            if not span.get("is_primary"):
                continue
            path = span["file_name"].replace("\\", "/")
            # Normalize ..\ in path
            path = re.sub(r"^crabomination/src/game/\.\./", "crabomination/src/", path)
            ln = span["line_start"]
            edits.setdefault(path, set()).add(ln)
            break

    print(f"Files to patch: {len(edits)}")

    for path, lns in edits.items():
        try:
            with open(path, encoding="utf-8") as f:
                lines = f.read().splitlines(keepends=True)
        except FileNotFoundError:
            print(f"NOT FOUND: {path}")
            continue
        # Sort descending so edits don't shift.
        for ln in sorted(lns, reverse=True):
            # The error points at `GameAction::CastSpell {`. Walk forward
            # to find `target:` and insert `additional_targets: vec![],`
            # on the next line.
            insert_at = None
            for k in range(ln - 1, min(ln + 20, len(lines))):
                if re.match(r"\s*target:.*,\s*$", lines[k]):
                    insert_at = k + 1
                    break
            if insert_at is None:
                print(f"WARN: couldn't find target: in {path}:{ln}")
                continue
            indent = re.match(r"^(\s*)", lines[insert_at - 1]).group(1)
            lines.insert(insert_at, indent + "additional_targets: vec![],\n")
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(lines)
        print(f"{path}: patched {len(lns)} CastSpell calls")


if __name__ == "__main__":
    main()
