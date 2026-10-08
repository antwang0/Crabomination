#!/usr/bin/env python3
"""Parked asks that name no seat.

A suspended ask is answered by `PendingDecision::acting_player`, which takes
the seat from the `PendingEffectState` (`answering_player`) and otherwise
falls back to the RESUME'S OWNER — the caster, the trigger's controller. An
arm that prompted anyone else (`seat_prompts(p)` / `seat_suspends(p)` for a
fan-out's opponent, a payer, a chooser — and `ctx.controller` IS that
opponent under `EachPlayerDoes` / `AsPlayer`) and parked a seat-less state
had the caster answer for them. Found 2026-10-08 off a strict pod sweep
(`MayDo` under "each opponent may draw"); 26 sites were fixed with
`PendingEffectState::Seated` / `seated(player, inner)`.

A finding: a `suspend_signal = Some(...)` whose state is neither a seated
variant (one `answering_player` maps) nor wrapped in `seated(`, after a
`seat_prompts` / `seat_suspends` gate. `--gate` exits 1 on any.
"""
import glob
import re
import sys

TYPES = "crabomination/src/game/types.rs"
GATE = re.compile(r"(?:seat_prompts|seat_suspends)\(([^)]*)\)")


def seated_variants():
    src = open(TYPES).read()
    body = src[src.index("fn answering_player") :]
    body = body[: body.index("\n    }\n")]
    return set(re.findall(r"PendingEffectState::(\w+)", body))


def main():
    seated = seated_variants()
    findings = []
    for f in sorted(glob.glob("crabomination/src/game/**/*.rs", recursive=True)):
        lines = open(f).read().split("\n")
        for i, line in enumerate(lines):
            m = re.search(r"PendingEffectState::(\w+)", line)
            if not m or m.group(1) in seated or m.group(1) == "Seated":
                continue
            window = "\n".join(lines[max(0, i - 8) : i + 1])
            if "suspend_signal = Some" not in window and "suspend_signal =\n" not in window:
                continue
            if any("seated(" in x for x in lines[max(0, i - 2) : i + 1]):
                continue
            gate = None
            for j in range(i, max(0, i - 60), -1):
                g = GATE.search(lines[j])
                if g:
                    gate = g.group(1).strip()
                    break
            if gate is None:
                continue
            findings.append((f, i + 1, m.group(1), gate))
    for f, line, variant, gate in findings:
        print(f"- {f}:{line}  {variant} after seat_prompts({gate})")
    print(f"\n{len(findings)} parked asks that prompt a seat and name none")
    return 1 if findings and "--gate" in sys.argv else 0


if __name__ == "__main__":
    sys.exit(main())
