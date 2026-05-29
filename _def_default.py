"""Replace explicit default-valued fields in CardDefinition struct literals
with `..Default::default()`. CardDefinition already derives Default.

Strictly scoped: only lines at a given CardDefinition literal's own field
indent whose trimmed text is an exact default literal are removed, so
nested structs (Subtypes, TokenDefinition, TriggeredAbility, back_face
CardDefinitions) are handled only as their own literals.
"""
import sys

DEFAULTS = {
    'supertypes: vec![],',
    'base_loyalty: 0,',
    'loyalty_abilities: vec![],',
    'alternative_cost: None,',
    'back_face: None,',
    'opening_hand: None,',
    'enters_with_counters: None,',
    'max_counters_of_kind: None,',
    'exile_on_resolve: false,',
    'affinity_filter: None,',
    'equipped_bonus: None,',
    'static_abilities: vec![],',
    'activated_abilities: no_abilities(),',
}


def sanitize(line):
    """Drop // comments and string contents so brace counting is reliable."""
    out = []
    i = 0
    instr = False
    while i < len(line):
        c = line[i]
        if instr:
            if c == '\\':
                i += 2
                continue
            if c == '"':
                instr = False
            i += 1
            continue
        if c == '"':
            instr = True
            i += 1
            continue
        if c == '/' and i + 1 < len(line) and line[i + 1] == '/':
            break
        out.append(c)
        i += 1
    return ''.join(out)


def indent(line):
    return len(line) - len(line.lstrip())


def process(path):
    raw = open(path, 'rb').read()
    eol = '\r\n' if b'\r\n' in raw else '\n'
    lines = raw.decode('utf-8').replace('\r\n', '\n').split('\n')
    n = len(lines)
    removed = set()
    inserts = {}

    i = 0
    while i < n:
        san = sanitize(lines[i])
        col = san.find('CardDefinition {')
        if col == -1 or 'struct CardDefinition' in lines[i]:
            i += 1
            continue
        # brace-match from this '{' to its close
        depth = 0
        close_i = None
        for k in range(i, n):
            s = sanitize(lines[k])
            if k == i:
                s = s[col:]
            for ch in s:
                if ch == '{':
                    depth += 1
                elif ch == '}':
                    depth -= 1
                    if depth == 0:
                        close_i = k
                        break
            if close_i is not None:
                break
        if close_i is None:
            i += 1
            continue
        # field indent = indent of first non-blank line after the opening
        fi = None
        for k in range(i + 1, close_i):
            if lines[k].strip():
                fi = indent(lines[k])
                break
        if fi is not None:
            removed_here = False
            for k in range(i + 1, close_i):
                if indent(lines[k]) == fi and lines[k].strip() in DEFAULTS:
                    removed.add(k)
                    removed_here = True
            has_default = any(
                indent(lines[k]) == fi and lines[k].strip() == '..Default::default()'
                for k in range(i + 1, close_i)
            )
            if removed_here and not has_default:
                inserts.setdefault(close_i, []).append(' ' * fi + '..Default::default()')
        i += 1  # do not skip: nested back_face CardDefinitions handled too

    out = []
    for idx, l in enumerate(lines):
        if idx in inserts:
            out.extend(inserts[idx])
        if idx in removed:
            continue
        out.append(l)
    open(path, 'w', encoding='utf-8', newline='').write(eol.join(out))
    return n, len(out)


if __name__ == '__main__':
    for p in sys.argv[1:]:
        a, b = process(p)
        print(f'{p}: {a} -> {b} ({a-b} removed)')
