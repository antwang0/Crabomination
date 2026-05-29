"""Collapse the dominant test pattern

    g.perform_action(GameAction::CastSpell {
        card_id: ID, target: T, additional_targets: vec![], mode: None, x_value: None,
    }).expect(MSG);          // or .unwrap()
    drain_stack(&mut g);     // result discarded

into `cast(&mut g, ID);` (target None) or `cast_at(&mut g, ID, INNER);`.

Only matches when additional_targets/mode/x_value are the defaults and the
drain result is discarded, so behavior is identical.
"""
import re, sys

PAT = re.compile(
    r'(?P<ind>[ \t]*)(?P<recv>\w+)\.perform_action\(GameAction::CastSpell\s*\{\s*'
    r'card_id:\s*(?P<id>[^,\n]+?),\s*'
    r'target:\s*(?P<tgt>None|Some\((?P<inner>.+?)\)),\s*'
    r'additional_targets:\s*vec!\[\],\s*mode:\s*None,\s*x_value:\s*None,\s*'
    r'\}\)\s*\.\s*(?:expect\([^;]*?\)|unwrap\(\))\s*;\s*'
    r'drain_stack\(&mut\s+(?P=recv)\)\s*;',
    re.DOTALL,
)


def repl(m):
    ind, recv, cid = m['ind'], m['recv'], m['id'].strip()
    if m['tgt'] == 'None':
        return f'{ind}cast(&mut {recv}, {cid});'
    return f'{ind}cast_at(&mut {recv}, {cid}, {m["inner"].strip()});'


def process(path):
    raw = open(path, 'rb').read()
    eol = '\r\n' if b'\r\n' in raw else '\n'
    text = raw.decode('utf-8').replace('\r\n', '\n')
    new, n = PAT.subn(repl, text)
    open(path, 'w', encoding='utf-8', newline='').write(new.replace('\n', eol))
    return n


if __name__ == '__main__':
    for p in sys.argv[1:]:
        print(f'{p}: {process(p)} blocks collapsed')
