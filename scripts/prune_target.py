#!/usr/bin/env python3
"""Reclaim `target/` space without costing a rebuild that would not happen anyway.

Cargo never deletes a build output. Every configuration — profile, feature
set, check vs build vs test, stable vs `scripts/fast.sh`'s nightly, each rustc
upgrade — gets its own hash, and the old ones stay forever; rustc also keeps
the previous incremental session beside the newest one. That is how `target/`
grew to 91-131 GB three times in September 2026.

This removes, in every profile directory under `target/`, only what no
build will miss:

  1. incremental caches untouched for `--age` hours: a configuration nobody
     has built since. Cargo does not fingerprint them, so a live one costs
     one non-incremental compile of that crate the next time it is dirty,
     and nothing else;
  2. in the incremental caches that remain, every session but the newest —
     rustc deletes those itself at the next compile and never reads them;
  3. workspace build outputs in `deps/` (libraries, bins, test binaries and
     their `.dwo` files) that are dirty: a source listed in the unit's `.d`
     file is newer than the unit or gone, or a workspace crate it depends
     on has changed since, so cargo rebuilds it on its next use anyway.

⚠ **Never delete an output cargo still considers fresh**, however old or
unread it looks. Cargo rebuilds a missing output under the same hash, and the
new mtime makes every unit above it stale in turn: deleting third-party
dependencies no build had read for 14 days (by atime) took out `cc`,
`pkg-config` and proc macros — read only when what uses them recompiles —
and cost a 3-minute rebuild of Bevy and the engine in both `debug` and `play`
(2026-09-30). A workspace crate nobody edited for two days is the same trap.
Old third-party copies (a Bevy feature change leaves the previous build
behind) can only be told from live ones by cargo's own unit graph, which
this script does not have; `cargo clean` is the tool for those.

Refuses to run while rustc or cargo is running in this checkout (another
session may be building here).

    scripts/prune_target.py --dry-run      # what it would delete
    scripts/prune_target.py                # delete it
    scripts/prune_target.py --age 12       # a tighter cut for incremental caches
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "target"
# `<name>-<16 hex>` with an optional `lib` prefix, then an extension or a
# `.<cgu>.rcgu.dwo` tail.
UNIT = re.compile(r"^(?:lib)?([A-Za-z0-9_]+)-([0-9a-f]{16})(?:\..*)?$")
# A finalized incremental session: `s-<time>-<random>-<svh>`; an unfinished
# one ends in `-working`.
SESSION = re.compile(r"^s-[a-z0-9]+-[a-z0-9]+-[a-z0-9]+$")


def busy() -> bool:
    """Whether a cargo or rustc is running in this checkout (one building
    some other project does not count)."""
    for tool in ("rustc", "cargo"):
        found = subprocess.run(["pgrep", "-x", tool], capture_output=True, text=True).stdout.split()
        for pid in found:
            try:
                cwd = Path(os.readlink(f"/proc/{pid}/cwd"))
            except OSError:
                return True  # can't tell, so assume it's ours
            if cwd == ROOT or ROOT in cwd.parents:
                return True
    return False


def workspace() -> tuple[dict[str, str], dict[str, float]]:
    """Each workspace target's package, and the newest library source of
    each package or anything in the workspace it depends on: a unit older
    than that is rebuilt on its next use whatever its own sources say."""
    meta = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    )
    packages = json.loads(meta.stdout)["packages"]
    ours = {p["name"] for p in packages}
    package_of: dict[str, str] = {}
    deps_of: dict[str, set[str]] = {}
    own_newest: dict[str, float] = {}
    for p in packages:
        for t in p["targets"]:
            package_of[t["name"].replace("-", "_")] = p["name"]
        package_of[p["name"].replace("-", "_")] = p["name"]
        deps_of[p["name"]] = {d["name"] for d in p["dependencies"] if d["name"] in ours}
        manifest = Path(p["manifest_path"])
        src = manifest.parent / "src"
        own_newest[p["name"]] = max(
            [manifest.stat().st_mtime]
            + [f.stat().st_mtime for f in src.rglob("*.rs") if "bin" not in f.relative_to(src).parts[:1]]
        )

    def closure(name: str, seen: set[str]) -> set[str]:
        for d in deps_of[name] - seen:
            seen.add(d)
            closure(d, seen)
        return seen

    upstream = {n: max(own_newest[d] for d in closure(n, set())) if deps_of[n] else 0.0 for n in deps_of}
    return package_of, upstream


def size_of(path: Path) -> int:
    if path.is_dir() and not path.is_symlink():
        return sum(f.stat().st_size for f in path.rglob("*") if f.is_file() and not f.is_symlink())
    return path.stat().st_size


def newest_mtime(path: Path) -> float:
    if path.is_dir():
        return max([path.stat().st_mtime] + [f.stat().st_mtime for f in path.rglob("*")])
    return path.stat().st_mtime


def dirty(dep_info: Path, built: float) -> bool:
    """Whether a source the unit was built from is newer than the unit or gone."""
    try:
        text = dep_info.read_text()
    except OSError:
        return False
    for line in text.splitlines():
        if line.startswith("#"):
            continue
        head, _, deps = line.partition(": ")
        for src in [head.rstrip(":")] + deps.replace("\\ ", "\0").split():
            path = ROOT / src.replace("\0", " ")
            if TARGET in path.parents or ROOT not in path.parents:
                continue
            try:
                if path.stat().st_mtime > built:
                    return True
            except FileNotFoundError:
                return True
    return False


def plan(profile: Path, cutoff: float, package_of: dict[str, str], upstream: dict[str, float]) -> list[tuple[Path, str]]:
    doomed: list[tuple[Path, str]] = []

    incremental = profile / "incremental"
    if incremental.is_dir():
        for unit in sorted(incremental.iterdir()):
            if not unit.is_dir():
                continue
            if newest_mtime(unit) < cutoff:
                doomed.append((unit, "stale incremental"))
                continue
            sessions = sorted(
                (s for s in unit.iterdir() if s.is_dir() and SESSION.match(s.name)),
                key=lambda s: s.stat().st_mtime,
            )
            for old in sessions[:-1]:
                doomed.append((old, "superseded session"))
                lock = old.with_name("-".join(old.name.split("-")[:3]) + ".lock")
                if lock.exists():
                    doomed.append((lock, "superseded session"))

    deps = profile / "deps"
    if deps.is_dir():
        units: dict[tuple[str, str], list[Path]] = {}
        for f in deps.iterdir():
            m = UNIT.match(f.name)
            if m and m.group(1) in package_of:
                units.setdefault((m.group(1), m.group(2)), []).append(f)
        for (name, hash_), files in sorted(units.items()):
            built = max(f.stat().st_mtime for f in files)
            if upstream[package_of[name]] > built or dirty(deps / f"{name}-{hash_}.d", built):
                doomed.extend((f, "dirty output") for f in files)
    return doomed


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--age", type=float, default=48.0, help="hours untouched before an incremental cache counts as stale (default 48)")
    ap.add_argument("--dry-run", action="store_true", help="list what would go, delete nothing")
    args = ap.parse_args()

    if not TARGET.is_dir():
        print("no target/ directory")
        return 0
    if busy():
        print("rustc or cargo is running in this checkout; not pruning under a live build", file=sys.stderr)
        return 1

    cutoff = time.time() - args.age * 3600
    package_of, upstream = workspace()
    totals: dict[str, int] = {}
    for profile in sorted(p for p in TARGET.iterdir() if p.is_dir()):
        for path, why in plan(profile, cutoff, package_of, upstream):
            size = size_of(path)
            totals[why] = totals.get(why, 0) + size
            if args.dry_run:
                print(f"{size / 1e6:9.1f} MB  {why:18}  {path.relative_to(ROOT)}")
            elif path.is_dir():
                shutil.rmtree(path)
            else:
                path.unlink()

    verb = "would free" if args.dry_run else "freed"
    for why, size in sorted(totals.items()):
        print(f"{verb} {size / 1e9:6.2f} GB  {why}")
    print(f"{verb} {sum(totals.values()) / 1e9:6.2f} GB  total")
    return 0


if __name__ == "__main__":
    sys.exit(main())
