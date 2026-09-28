#!/usr/bin/env python3
"""Run every `scripts/audit_*.py` that has a `--gate` and report its exit code.

Each gated script names its input in its docstring as
`target/debug/dump_cards <flags> > /tmp/<file>`; the dump for each distinct
flag set is built once. A script whose docstring names no dump runs with no
input argument. Exits 1 when any gate fails. Gates rot silently otherwise:
nothing runs them, and a concurrent card commit can turn one red.

    cargo build -p crabomination --bin dump_cards
    python3 scripts/audit_gates.py [--verbose]
"""

import ast
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DUMP = os.path.join(ROOT, "target", "debug", "dump_cards")
FLAGS = re.compile(r"dump_cards((?: +--?[\w-]+(?: +\"[^\"]*\"| +'[^']*'| +[^>\s-][^>\s]*)?)*) *>")


def gated_scripts():
    for name in sorted(os.listdir(os.path.join(ROOT, "scripts"))):
        if not (name.startswith("audit_") and name.endswith(".py")) or name == "audit_gates.py":
            continue
        path = os.path.join(ROOT, "scripts", name)
        src = open(path, encoding="utf-8").read()
        if "--gate" not in src:
            continue
        doc = ast.get_docstring(ast.parse(src)) or ""
        m = FLAGS.search(doc)
        yield name, path, (m.group(1).strip() if m else None)


def main():
    verbose = "--verbose" in sys.argv
    tmp = tempfile.mkdtemp(prefix="crab_gates_")
    dumps, failed = {}, []
    for name, path, flags in gated_scripts():
        args = [sys.executable, path]
        if flags is not None:
            if flags not in dumps:
                out = os.path.join(tmp, f"dump{len(dumps)}.tsv")
                with open(out, "w") as f:
                    subprocess.run(f"{DUMP} {flags}", shell=True, stdout=f, check=True, cwd=ROOT)
                dumps[flags] = out
            args.append(dumps[flags])
        args.append("--gate")
        r = subprocess.run(args, capture_output=True, text=True, cwd=ROOT)
        tail = (r.stdout + r.stderr).strip().splitlines()[-1:] or [""]
        print(f"{'FAIL' if r.returncode else 'ok  '} {name:40} {tail[0][:90]}")
        if r.returncode:
            failed.append(name)
            if verbose:
                print(r.stdout[-2000:], r.stderr[-2000:])
    print(f"# {len(failed)} failing gate(s): {' '.join(failed)}", file=sys.stderr)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
