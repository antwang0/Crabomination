"""Patch one-line `CastSpell { card_id: ..., target: ..., mode: ..., x_value: ... }`
struct literals that are missing `additional_targets`."""

import re
import os


def main():
    files = []
    for root, _dirs, fnames in os.walk("crabomination/src/tests"):
        for fn in fnames:
            if fn.endswith(".rs"):
                files.append(os.path.join(root, fn))

    for path in files:
        with open(path, encoding="utf-8") as f:
            text = f.read()
        original = text
        out_lines = []
        for line in text.splitlines(keepends=True):
            if (
                "CastSpell" in line
                or ("target:" in line and "mode:" in line and "additional_targets" not in line)
            ) and "additional_targets" not in line and "mode:" in line and "target:" in line:
                # Insert additional_targets: vec![], between the
                # `target: <expr>,` and ` mode:` segments.
                # Robust handling: split on ", mode:" and insert before it.
                line = re.sub(
                    r"(target:\s*[^,]+(?:\([^)]*\))?\s*,)(\s*mode:)",
                    r"\1 additional_targets: vec![],\2",
                    line,
                    count=1,
                )
            out_lines.append(line)
        new_text = "".join(out_lines)
        if new_text != original:
            with open(path, "w", encoding="utf-8") as f:
                f.write(new_text)
            n_added = new_text.count("additional_targets: vec![]") - original.count("additional_targets: vec![]")
            print(f"{path}: added {n_added} additional_targets")


if __name__ == "__main__":
    main()
