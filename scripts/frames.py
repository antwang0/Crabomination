#!/usr/bin/env python3
"""Stack frame size of named functions in a binary, read off the prologue:
the inline probe loop's `sub $N,%r11` (frames past a page) plus the trailing
`sub $N,%rsp`. Found `(-276)`: `run_effect` at 97 KB, the only frame in
the binary past 4 KB (PERF "run_effect's frame").

    python3 scripts/frames.py target/release-fast/bot_ladder \
        "GameState>::run_effect" "GameState::perform_action_inner"

A whole-binary census (every probed prologue) is one streamed pass:
    objdump -d --no-show-raw-insn <bin> | grep -B40 'sub +\$0x[0-9a-f]+,%r11'
"""
import re, subprocess, sys
b = sys.argv[1]
syms = subprocess.run(["nm", "-C", b], capture_output=True, text=True).stdout.splitlines()
for needle in sys.argv[2:]:
    hits = [l for l in syms if re.search(r" [tT] .*" + re.escape(needle) + r"$", l)]
    if not hits:
        print(f"{needle:48} (no symbol)"); continue
    addr = int(hits[0].split()[0], 16)
    d = subprocess.run(["objdump", "-d", "--no-show-raw-insn", f"--start-address={addr:#x}", f"--stop-address={addr+96:#x}", b], capture_output=True, text=True).stdout
    probe = re.search(r"sub\s+\$0x([0-9a-f]+),%r11", d)
    subs = re.findall(r"sub\s+\$0x([0-9a-f]+),%rsp", d)
    subs = [int(x, 16) for x in subs if int(x, 16) != 0x1000]
    frame = (int(probe.group(1), 16) if probe else 0) + (subs[-1] if subs else 0)
    print(f"{needle:48} {frame:>8,} bytes" + ("   (probed)" if probe else ""))
