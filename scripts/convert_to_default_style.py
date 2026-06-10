#!/usr/bin/env python3
"""Convert fully-specified `CardDefinition { ... }` literals to
`..Default::default()` style: drop top-level fields whose value is the
type default and append the spread if missing.

Brace/string-aware so nested struct literals (TokenDefinition, vec!
bodies, `{T}` in strings) are untouched. Idempotent.

Usage: python3 scripts/convert_to_default_style.py FILE...
"""
import re
import sys

# Whole-segment (whitespace-normalized) values equal to the field default.
DEFAULT_SEGMENTS = {
    "cost: ManaCost::default()",
    "supertypes: vec![]",
    "subtypes: Subtypes::default()",
    "power: 0",
    "toughness: 0",
    "base_loyalty: 0",
    "keywords: vec![]",
    "static_abilities: vec![]",
    "effect: Effect::Noop",
    "activated_abilities: no_abilities()",
    "activated_abilities: vec![]",
    "activated_abilities: Vec::new()",
    "triggered_abilities: vec![]",
    "loyalty_abilities: vec![]",
    "alternative_cost: None",
    "back_face: None",
    "opening_hand: None",
    "enters_with_counters: None",
    "enters_as_copy: None",
    "max_counters_of_kind: None",
    "exile_on_resolve: false",
    "affinity_filter: None",
    "affinity_graveyard_filter: None",
    "equipped_bonus: None",
    "soulbond_bonus: None",
    "additional_cast_cost: vec![]",
    "bestow: None",
    "foretell_cost: None",
    "adventure: None",
    "plot_cost: None",
    "split: None",
    "saga_chapters: vec![]",
    "miracle: None",
    "room: None",
}


def find_block_end(src, open_idx):
    """Index just past the matching `}` for the `{` at open_idx.
    Skips string literals and // comments."""
    depth = 0
    i = open_idx
    n = len(src)
    while i < n:
        c = src[i]
        if c == '"':
            i += 1
            while i < n and src[i] != '"':
                i += 2 if src[i] == "\\" else 1
        elif c == "/" and i + 1 < n and src[i + 1] == "/":
            while i < n and src[i] != "\n":
                i += 1
        elif c == "'":
            # char literal or lifetime; skip 'x' / '\x' forms only
            if i + 2 < n and (src[i + 2] == "'" or (src[i + 1] == "\\" and i + 3 < n and src[i + 3] == "'")):
                i += 3 if src[i + 2] == "'" else 4
                continue
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise ValueError("unbalanced braces")


def split_fields(body):
    """Split a struct-literal body into top-level comma-separated segments."""
    segs, start, depth, i, n = [], 0, 0, 0, len(body)
    while i < n:
        c = body[i]
        if c == '"':
            i += 1
            while i < n and body[i] != '"':
                i += 2 if body[i] == "\\" else 1
        elif c == "/" and i + 1 < n and body[i + 1] == "/":
            while i < n and body[i] != "\n":
                i += 1
        elif c in "{[(":
            depth += 1
        elif c in "}])":
            depth -= 1
        elif c == "," and depth == 0:
            segs.append(body[start:i])
            start = i + 1
        i += 1
    if body[start:].strip():
        segs.append(body[start:])
    return segs


def normalize(seg):
    code = "\n".join(
        l for l in seg.splitlines() if not l.strip().startswith("//")
    )
    return re.sub(r"\s+", " ", code).strip()


def comment_lines(seg):
    return [l for l in seg.splitlines() if l.strip().startswith("//")]


def convert(src):
    out = []
    pos = 0
    changed = False
    for m in re.finditer(r"CardDefinition\s*\{", src):
        open_idx = m.end() - 1
        if open_idx < pos:
            continue  # nested literal already handled by outer rebuild
        # skip `-> [path::]CardDefinition {` fn signatures — only struct
        # literals
        k = m.start()
        while k > 0 and (src[k - 1].isalnum() or src[k - 1] in ":_"):
            k -= 1
        if src[:k].rstrip().endswith("->"):
            continue
        end = find_block_end(src, open_idx)
        body = src[open_idx + 1 : end - 1]
        body_code = "\n".join(
            l for l in body.splitlines() if not l.strip().startswith("//")
        )
        if re.search(r"CardDefinition\s*\{", body_code):
            continue  # nested literals: leave the whole outer block alone
        segs = split_fields(body)
        # literals built on a base-fn spread (`..drone(...)`) already
        # inherit defaults — leave them alone
        if any(
            normalize(s).startswith("..") and normalize(s) != "..Default::default()"
            for s in segs
        ):
            continue
        kept, pending_comments = [], []
        has_spread = False
        for seg in segs:
            norm = normalize(seg)
            if norm == "..Default::default()":
                has_spread = True
                continue
            if norm in DEFAULT_SEGMENTS:
                pending_comments.extend(comment_lines(seg))
                continue
            if pending_comments:
                seg_lines = seg.splitlines()
                # splice carried comments before the field's first code line
                idx = next(
                    (k for k, l in enumerate(seg_lines) if l.strip()), 0
                )
                seg = "\n".join(seg_lines[:idx] + pending_comments + seg_lines[idx:])
                pending_comments = []
            kept.append(seg)
        line_start = src.rfind("\n", 0, m.start()) + 1
        indent = " " * (len(src[line_start:m.start()]) - len(src[line_start:m.start()].lstrip()))
        field_indent = indent + "    "
        pieces = []
        for seg in kept:
            text = seg.strip("\n")
            # keep multi-line segments verbatim; re-indent single-line ones
            if "\n" in text:
                pieces.append(text.rstrip())
            else:
                pieces.append(field_indent + text.strip())
        for c in pending_comments:
            pieces.append(field_indent + c.strip())
        pieces.append(field_indent + "..Default::default()")
        new_block = "{\n" + ",\n".join(p for p in pieces[:-1]) + ",\n" + pieces[-1] + "\n" + indent + "}"
        out.append(src[pos:open_idx])
        out.append(new_block)
        pos = end
        if new_block != src[open_idx:end] or has_spread:
            changed = True
    out.append(src[pos:])
    return "".join(out), changed


def main(paths):
    for p in paths:
        src = open(p).read()
        new, changed = convert(src)
        if changed and new != src:
            open(p, "w").write(new)
            print(f"converted {p}")


if __name__ == "__main__":
    main(sys.argv[1:])
