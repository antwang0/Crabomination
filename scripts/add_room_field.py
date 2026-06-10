"""Insert `room: None,` into fully-specified CardDefinition literals
missing the new field. Parse cargo E0063 output for the sites, then patch
each by anchoring after the `miracle:` field line."""

import json
import re
import subprocess


def main():
    proc = subprocess.run(
        ["cargo", "build", "-p", "crabomination_catalog",
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
        if "missing field `room`" not in msg.get("message", ""):
            continue
        for span in msg.get("spans", []):
            if not span.get("is_primary"):
                continue
            path = span["file_name"].replace("\\", "/")
            edits.setdefault(path, set()).add(span["line_start"])
            break

    print(f"Files to patch: {len(edits)}")
    for path, lns in edits.items():
        with open(path, encoding="utf-8") as f:
            lines = f.read().splitlines(keepends=True)
        for ln in sorted(lns, reverse=True):
            # The error points at `CardDefinition {`. Walk forward to the
            # `miracle:` field line and insert after it.
            insert_at = None
            for k in range(ln - 1, min(ln + 200, len(lines))):
                if re.match(r"\s*miracle:.*$", lines[k]):
                    insert_at = k + 1
                    break
            if insert_at is None:
                print(f"WARN: no saga_chapters in {path}:{ln}")
                continue
            indent = re.match(r"^(\s*)", lines[insert_at - 1]).group(1)
            lines.insert(insert_at, indent + "room: None,\n")
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(lines)
        print(f"{path}: patched {len(lns)} literals")


if __name__ == "__main__":
    main()
