#!/usr/bin/env python3
"""Reclaim `target/` space without costing a rebuild that would not happen anyway.

Cargo never deletes a build output. Every configuration — profile, feature
set, check vs build vs test, stable vs `scripts/fast.sh`'s nightly, each rustc
upgrade — gets its own hash, and the old ones stay forever; rustc also keeps
the previous incremental session beside the newest one. That is how `target/`
grew to 91-131 GB three times in September 2026.

This removes, in every profile directory under `target/`:

  1. incremental caches untouched for `--age` hours: a configuration nobody
     has built since. Cargo does not fingerprint them, so a live one costs
     one non-incremental compile of that crate the next time it is dirty,
     and nothing else;
  2. in the incremental caches that remain, every session but the newest —
     rustc deletes those itself at the next compile and never reads them;
  3. workspace build outputs in `deps/` (libraries, bins, test binaries and
     their `.dwo` files) that are dirty: a source listed in the unit's own
     `.d` file is newer than the unit or gone, which is cargo's own test, so
     it rebuilds them on their next use anyway. A unit stale only because a
     crate below it changed is left for cargo to overwrite in place: judging
     that from source times flags units cargo keeps (a comment in a
     `Cargo.toml`, a test-only file), and a deleted fresh library cascades;
  4. in `debug/` and `play/`, every build unit — third-party or ours — that
     no unit in use depends on, found by mark and sweep over cargo's own
     records. Each unit's fingerprint (`.fingerprint/<pkg>-<hash>/`, or
     `build/<pkg>/<hash>/fingerprint/` in nightly's layout) names the
     fingerprint of every unit it was built against, build scripts and proc
     macros included. The roots are the units cargo built, checked or ran
     in the last `--keep-days` days (`Unit.last_touched`); everything they
     reach is kept, and the rest — the Bevy a feature change left behind, a
     configuration nobody has used in weeks — goes with its fingerprint,
     its `deps/` files and its `build/` directory. A configuration idle
     that long rebuilds its dependencies (minutes, in these two profiles)
     when it comes back. The optimized profiles are left out: their
     dependencies carry debuginfo and take ~30 minutes cold, for about a
     gigabyte.

Checked 2026-09-30 against seven everyday commands (client dev / play /
test / clippy, the suite's `test --no-run`, workspace `check`, nightly
`fast.sh check`): with every `dep-*` access time backdated two days and the
sweep's window opened just before a no-op run of the seven, the sweep kept
all of them — each compiled nothing afterwards — and removed exactly the
1.2 GB a lone `cargo build -p crabomination --bin bot_ladder` had built
beside them (its own feature unification). A dirty-rule prune after an
engine edit left the rebuild counts identical to an unpruned control.

⚠ **Never delete an output a live unit was built against**, however old or
unread it looks. Cargo rebuilds a missing output under the same hash, and the
new mtime makes every unit above it stale in turn: pruning third-party
dependencies by access time alone took out `cc`, `pkg-config` and proc
macros — read only when what uses them recompiles — and cost a 3-minute
rebuild of Bevy and the engine in both `debug` and `play` (2026-09-30). That
is why rule 4 walks the graph from the roots instead of judging each unit by
its own times.

Refuses to run while rustc or cargo is running in this checkout (another
session may be building here).

    scripts/prune_target.py --dry-run      # what it would delete
    scripts/prune_target.py                # delete it
    scripts/prune_target.py --age 12       # a tighter cut for incremental caches
    scripts/prune_target.py --keep-days 30 # a looser one for the sweep
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
# A classic unit directory: `<package>-<16 hex>`.
HASHED = re.compile(r"^.+-([0-9a-f]{16})$")
# A finalized incremental session: `s-<time>-<random>-<svh>`; an unfinished
# one ends in `-working`.
SESSION = re.compile(r"^s-[a-z0-9]+-[a-z0-9]+-[a-z0-9]+$")
# Profiles swept by rule 4: dev settings, so a dependency rebuilds in minutes.
SWEPT_PROFILES = {"debug", "play"}


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


def workspace_targets() -> set[str]:
    """Crate names of every workspace target (libraries, bins, tests)."""
    meta = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    )
    names = set()
    for package in json.loads(meta.stdout)["packages"]:
        names.add(package["name"].replace("-", "_"))
        names.update(t["name"].replace("-", "_") for t in package["targets"])
    return names


def read(path: Path) -> str:
    """`path`'s text, read without touching its access time where the kernel
    allows (`O_NOATIME`, owner only): rule 4 reads access times, and a prune
    that refreshed them would keep everything it looked at."""
    try:
        fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOATIME", 0))
    except PermissionError:
        fd = os.open(path, os.O_RDONLY)
    with os.fdopen(fd, encoding="utf-8", errors="replace") as f:
        return f.read()


def files_under(path: Path) -> list[Path]:
    if path.is_dir() and not path.is_symlink():
        return [f for f in path.rglob("*") if f.is_file() and not f.is_symlink()]
    return [path]


def size_of(path: Path) -> int:
    return sum(f.stat().st_size for f in files_under(path))


def newest_mtime(path: Path) -> float:
    """Newest file mtime under `path`. Directory mtimes are left out: removing
    a superseded session (rule 2) stamps its parent, which would make an idle
    cache look freshly built."""
    return max([0.0] + [f.stat().st_mtime for f in files_under(path)])


def dirty(dep_info: Path, built: float) -> bool:
    """Whether a source the unit was built from is newer than the unit or gone."""
    try:
        text = read(dep_info)
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


class Unit:
    """One cargo build unit: where its fingerprint lives, every path it owns,
    the fingerprint values it answers to, and those it was built against."""

    def __init__(self, fingerprint: Path, owned: list[Path]):
        self.fingerprint = fingerprint
        self.owned = owned
        self.values: set[str] = set()
        self.deps: list[str] = []
        for record in fingerprint.glob("*.json"):
            try:
                value = read(record.with_suffix("")).strip()
                parsed = json.loads(read(record))
            except (OSError, ValueError):
                continue
            self.values.add(value)
            # `[package id hash, crate name, public, fingerprint]`; the value
            # file beside each record holds its fingerprint as the hex of the
            # little-endian bytes (cargo's `util::to_hex`).
            self.deps += [d[3].to_bytes(8, "little").hex() for d in parsed.get("deps", [])]

    def last_touched(self) -> float:
        """When cargo last used the unit: newest build of anything it owns;
        the last read of its fingerprint's `dep-*` file, which cargo's
        freshness check reads for every unit of the graph on every
        invocation, no-op builds included (third-party, build scripts and
        the nightly layout alike; checked 2026-09-30 by backdating the
        access times and running each command); or the last run of an
        executable it owns. `relatime` refreshes an access time at most
        daily, which the default `--keep-days` dwarfs. Nothing else is read
        for its access time: `.rmeta` headers and fingerprint records get
        read by tools like this one (it opens them `O_NOATIME`)."""
        stamps = [0.0]
        for path in self.owned:
            for f in files_under(path) if path.exists() else []:
                st = f.stat()
                stamps.append(st.st_mtime)
                if st.st_mode & 0o111 and not f.suffix:
                    stamps.append(st.st_atime)
        stamps += [f.stat().st_atime for f in self.fingerprint.glob("dep-*")]
        return max(stamps)


def units_of(profile: Path) -> list[Unit]:
    """Every unit in `profile`, in both the classic layout (`.fingerprint/`,
    `deps/`, `build/<pkg>-<hash>/`) and nightly's (`build/<pkg>/<hash>/`)."""
    deps_by_hash: dict[str, list[Path]] = {}
    if (profile / "deps").is_dir():
        for f in (profile / "deps").iterdir():
            m = UNIT.match(f.name)
            if m:
                deps_by_hash.setdefault(m.group(2), []).append(f)
    units = []
    if (profile / ".fingerprint").is_dir():
        for fp in (profile / ".fingerprint").iterdir():
            m = HASHED.match(fp.name)
            if not (m and fp.is_dir()):
                continue
            owned = [fp, *deps_by_hash.get(m.group(1), [])]
            if (profile / "build" / fp.name).is_dir():
                owned.append(profile / "build" / fp.name)
            units.append(Unit(fp, owned))
    if (profile / "build").is_dir():
        for package in (profile / "build").iterdir():
            if HASHED.match(package.name) or not package.is_dir():
                continue
            for unit in package.iterdir():
                if (unit / "fingerprint").is_dir():
                    units.append(Unit(unit / "fingerprint", [unit]))
    return units


def unreachable(profile: Path, keep_after: float) -> list[Path]:
    """Rule 4: what no unit built or read since `keep_after` depends on."""
    units = units_of(profile)
    by_value = {v: u for u in units for v in u.values}
    live = [u for u in units if u.last_touched() >= keep_after]
    seen = {id(u) for u in live}
    while live:
        for value in live.pop().deps:
            dep = by_value.get(value)
            if dep is not None and id(dep) not in seen:
                seen.add(id(dep))
                live.append(dep)
    return [path for u in units if id(u) not in seen for path in u.owned]


def plan(profile: Path, cutoff: float, keep_after: float, ours: set[str]) -> list[tuple[Path, str]]:
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

    if profile.name in SWEPT_PROFILES:
        doomed += [(path, "unreachable unit") for path in unreachable(profile, keep_after)]

    deps = profile / "deps"
    if deps.is_dir():
        units: dict[tuple[str, str], list[Path]] = {}
        for f in deps.iterdir():
            m = UNIT.match(f.name)
            if m and m.group(1) in ours:
                units.setdefault((m.group(1), m.group(2)), []).append(f)
        for (name, hash_), files in sorted(units.items()):
            built = max(f.stat().st_mtime for f in files)
            if dirty(deps / f"{name}-{hash_}.d", built):
                doomed.extend((f, "dirty output") for f in files)
    return doomed


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--age", type=float, default=48.0, help="hours untouched before an incremental cache counts as stale (default 48)")
    ap.add_argument("--keep-days", type=float, default=14.0, help="days a unit built or read counts as in use for the sweep (default 14)")
    ap.add_argument("--dry-run", action="store_true", help="list what would go, delete nothing")
    args = ap.parse_args()

    if not TARGET.is_dir():
        print("no target/ directory")
        return 0
    if busy():
        print("rustc or cargo is running in this checkout; not pruning under a live build", file=sys.stderr)
        return 1

    now = time.time()
    cutoff = now - args.age * 3600
    keep_after = now - args.keep_days * 86400
    ours = workspace_targets()
    totals: dict[str, int] = {}
    for profile in sorted(p for p in TARGET.iterdir() if p.is_dir()):
        seen: set[Path] = set()
        for path, why in plan(profile, cutoff, keep_after, ours):
            # A dirty output can also be an unreachable one.
            if path in seen or not path.exists():
                continue
            seen.add(path)
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
