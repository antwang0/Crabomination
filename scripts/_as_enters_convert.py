#!/usr/bin/env python3
"""One-shot helper: move a card's single ETB trigger into `as_enters_effect`.

CR 614.12 — an "As this ~ enters, …" clause is a replacement, not a trigger.
Handles the two shapes the catalog writes:

    triggered_abilities: vec![TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: <E>,
    }],
    triggered_abilities: vec![etb(<E>)],

and rewrites both to `as_enters_effect: Some(<E>),`. Refuses anything else,
including a `triggered_abilities` vec with more than the one entry --
those get converted by hand.

Usage: python3 scripts/_as_enters_convert.py <factory-ident> [<factory-ident> …]
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog/src"


def find_balanced(src, start, opener, closer):
    """Index just past the closer matching the opener at `start`."""
    depth = 0
    i = start
    while i < len(src):
        c = src[i]
        if c == '"':
            i += 1
            while i < len(src) and src[i] != '"':
                i += 2 if src[i] == "\\" else 1
        elif c == opener:
            depth += 1
        elif c == closer:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise ValueError("unbalanced")


def convert_body(body):
    m = re.search(r"^(\s*)triggered_abilities:\s*vec!\[", body, re.M)
    if not m:
        return None, "no triggered_abilities"
    indent = m.group(1)
    vec_open = body.index("[", m.end() - 1)
    vec_end = find_balanced(body, vec_open, "[", "]")
    inner = body[vec_open + 1 : vec_end - 1].strip().rstrip(",").strip()

    if inner.startswith("TriggeredAbility"):
        brace = inner.index("{")
        if find_balanced(inner, brace, "{", "}") != len(inner):
            return None, "more than one trigger"
        fields = inner[brace + 1 : -1]
        ev = re.search(
            r"event:\s*EventSpec::new\(\s*EventKind::EntersBattlefield,\s*"
            r"EventScope::SelfSource,?\s*\)\s*,",
            fields,
        )
        if not ev:
            return None, "not a self-source ETB trigger"
        rest = (fields[: ev.start()] + fields[ev.end() :]).strip()
        m2 = re.match(r"effect:\s*", rest)
        if not m2:
            return None, "trigger carries fields beyond event+effect"
        effect = rest[m2.end() :].strip().rstrip(",").strip()
        dedent = 4  # the literal sat one brace deeper inside `TriggeredAbility`
    elif inner.startswith("etb("):
        end = find_balanced(inner, inner.index("("), "(", ")")
        if end != len(inner):
            return None, "more than one trigger"
        effect = inner[inner.index("(") + 1 : end - 1].strip().rstrip(",").strip()
        dedent = 0  # `vec![etb(<E>)]` already sits at the field's own indent
    else:
        return None, f"unrecognised trigger shape: {inner[:40]!r}"

    # Re-indent the effect literal to where `as_enters_effect:` puts it.
    lines = effect.split("\n")
    if dedent and len(lines) > 1:
        effect = "\n".join(
            [lines[0]]
            + [line[dedent:] if line.startswith(" " * dedent) else line for line in lines[1:]]
        )
    repl = f"{indent}as_enters_effect: Some({effect}),"
    # Drop the trailing comma of the original field, and the newline it owned.
    tail = body[vec_end:]
    tail = re.sub(r"^,", "", tail, count=1)
    return body[: m.start()] + repl + tail, None


def main():
    idents = sys.argv[1:]
    if not idents:
        print(__doc__)
        return 1
    fn_re = re.compile(r"pub fn (\w+)\(\) -> CardDefinition \{")
    todo = set(idents)
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text()
        fns = [(m.start(), m.group(1)) for m in fn_re.finditer(src)]
        out = src
        for i, (start, ident) in enumerate(fns):
            if ident not in todo:
                continue
            stop = fns[i + 1][0] if i + 1 < len(fns) else len(src)
            body = src[start:stop]
            new, err = convert_body(body)
            if err:
                print(f"SKIP {ident} ({path.name}): {err}")
                continue
            out = out.replace(body, new, 1)
            todo.discard(ident)
            print(f"ok   {ident} ({path.name})")
        if out != src:
            path.write_text(out)
    for ident in sorted(todo):
        print(f"NOT FOUND/SKIPPED: {ident}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
