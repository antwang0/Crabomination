#!/usr/bin/env python3
"""Attribute a function's stack frame to its locals from llvm-dwarfdump:
every DW_TAG_variable / formal_parameter with a DW_OP_fbreg location, its
type, declaration line, and the inlined-subroutine chain (with call lines)
it sits in. Slots are distinct fbreg offsets; a slot's size is the gap to
the next offset up — an upper bound, and an *unnamed* 8 KB gap beside a
small variable is a temporary with no DWARF variable (`(-276)`: that was
`Arc::make_mut`'s inlined clone path).

The debuginfo of a `profiling-fast` build is split into per-CGU `.dwo`
files, and `--name=<fn>` finds only the *declaration* DIE. Three steps:

    D=$(ls -t target/profiling-fast/deps/crabomination-*cgu.00.rcgu.dwo | head -1)
    llvm-dwarfdump --debug-info --name=run_effect $D | grep -B1 'linkage_name.*10run_effect17h'   # -> decl DIE 0x...
    llvm-dwarfdump --debug-info $D | grep -B4 'DW_AT_specification.*(0x<decl>' \
        | grep -oE '^0x[0-9a-f]+:\s+DW_TAG_subprogram'                                         # -> concrete DIE
    llvm-dwarfdump --debug-info=0x<concrete> -c $D > dump.txt
    python3 scripts/dwarf_frame.py dump.txt 0x<concrete>

`ls -t`: a feature change (`--no-default-features`) gives the crate a new
hash and a second set of `.dwo` files beside the old one."""
import re, sys, collections
path, fnname = sys.argv[1], sys.argv[2]
lines = open(path, errors="replace").read().split("\n")
tag_re = re.compile(r"^(0x[0-9a-f]+):(\s*)DW_TAG_(\w+)")
attr_re = re.compile(r"^\s+DW_AT_(\w+)\s+\((.*)\)\s*$")
# parse into a flat list of DIEs with depth and attrs
dies = []
cur = None
for ln in lines:
    m = tag_re.match(ln)
    if m:
        cur = {"off": m.group(1), "depth": len(m.group(2)), "tag": m.group(3), "attrs": {}, "raw": []}
        dies.append(cur)
        continue
    if cur is None:
        continue
    cur["raw"].append(ln)
    m = attr_re.match(ln)
    if m:
        cur["attrs"].setdefault(m.group(1), m.group(2))
    else:
        # continuation lines of a location list
        if "DW_OP_fbreg" in ln and "fbreg" not in cur["attrs"]:
            mm = re.search(r"DW_OP_fbreg\s+([+-]?\d+)", ln)
            if mm:
                cur["attrs"]["fbreg"] = mm.group(1)
for d in dies:
    loc = d["attrs"].get("location", "")
    mm = re.search(r"DW_OP_fbreg\s+([+-]?\d+)", loc)
    if mm and "fbreg" not in d["attrs"]:
        d["attrs"]["fbreg"] = mm.group(1)
    if "fbreg" not in d["attrs"]:
        for ln in d["raw"]:
            mm = re.search(r"DW_OP_fbreg\s+([+-]?\d+)", ln)
            if mm:
                d["attrs"]["fbreg"] = mm.group(1); break

def name_of(d):
    n = d["attrs"].get("name")
    if n: return n.strip('"')
    ao = d["attrs"].get("abstract_origin", "")
    mm = re.search(r'"([^"]+)"', ao)
    return mm.group(1) if mm else "?"

# find the subprogram named fnname (exact), walk its subtree
start = None
for i, d in enumerate(dies):
    if d["tag"] == "subprogram" and (d["attrs"].get("name", "").strip('"') == fnname or d["off"] == fnname):
        start = i; break
if start is None:
    sys.exit(f"no subprogram named {fnname}")
base_depth = dies[start]["depth"]
stack = []  # (depth, name) of enclosing inlined subroutines
slots = {}
owner_lines = collections.Counter()
for d in dies[start + 1:]:
    if d["depth"] <= base_depth:
        break
    while stack and stack[-1][0] >= d["depth"]:
        stack.pop()
    if d["tag"] == "inlined_subroutine":
        cl = d["attrs"].get("call_line", "?").strip("()")
        stack.append((d["depth"], f"{name_of(d)}@{cl}"))
        continue
    if d["tag"] in ("variable", "formal_parameter") and "fbreg" in d["attrs"]:
        off = int(d["attrs"]["fbreg"])
        ty = d["attrs"].get("type", "")
        mm = re.search(r'"([^"]+)"', ty)
        ty = mm.group(1) if mm else ty
        owner = stack[0][1] if stack else fnname
        chain = " <- ".join(n for _, n in reversed(stack)) if stack else fnname
        dl = d["attrs"].get("decl_line", "?").strip("()")
        slots.setdefault(off, []).append((name_of(d) + "@" + dl, ty, owner, chain))
offs = sorted(slots)
print(f"# {len(offs)} distinct fbreg slots, span {offs[0]}..{offs[-1]} (frame base is the bottom of the frame)")
sizes = {}
for i, o in enumerate(offs):
    nxt = offs[i + 1] if i + 1 < len(offs) else 0
    sizes[o] = nxt - o
by_owner = collections.Counter()
for o in offs:
    by_owner[slots[o][0][2]] += sizes[o]
print("\n# bytes by top-level owner (the inlined callee the slot's first variable sits in)")
for k, v in by_owner.most_common(25):
    print(f"{v:>8,}  {k[:110]}")
print("\n# largest slots")
for o in sorted(offs, key=lambda o: -sizes[o])[:40]:
    n, ty, owner, chain = slots[o][0]
    print(f"{sizes[o]:>8,}  fbreg {o:>8}  {n:28} : {ty[:44]:44}  in {chain[:150]}")
