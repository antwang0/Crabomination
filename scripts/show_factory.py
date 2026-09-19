#!/usr/bin/env python3
"""Print a card factory's doc comment and body by `pub fn` name.

    python3 scripts/show_factory.py wrenn_and_six [more_fns…]

A reading aid for the audit scripts, which report `path::fn` rows. It brace-
matches the same way they do, so what it prints is exactly the literal an
own-literal audit read.
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")


def find(fn):
    for dp, _, fs in os.walk(CATALOG):
        for f in fs:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(dp, f)
            s = open(p, encoding="utf-8").read()
            m = re.search(r"pub fn %s\(\) -> CardDefinition \{" % re.escape(fn), s)
            if not m:
                continue
            start, depth, i = m.end() - 1, 0, m.end() - 1
            while i < len(s):
                if s[i] == "{":
                    depth += 1
                elif s[i] == "}":
                    depth -= 1
                    if depth == 0:
                        break
                i += 1
            doc = []
            for line in s[: m.start()].rstrip().split("\n")[::-1]:
                if line.lstrip().startswith("///") or line.lstrip().startswith("//"):
                    doc.append(line)
                else:
                    break
            return p, "\n".join(doc[::-1]), s[start : i + 1]
    return None


def main():
    for fn in sys.argv[1:]:
        hit = find(fn)
        print(f"===== {fn} =====")
        if hit is None:
            print("  not found")
            continue
        path, doc, body = hit
        print(os.path.relpath(path, ROOT))
        if doc:
            print(doc)
        print(body)


if __name__ == "__main__":
    main()
