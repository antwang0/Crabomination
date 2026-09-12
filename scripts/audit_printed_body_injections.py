#!/usr/bin/env python3
"""Injection battery for `audit_printed_body.py`'s readers.

    python3 scripts/audit_printed_body_injections.py    # 9/9 as expected

**A GATE THAT CANNOT FAIL IS WORSE THAN NO GATE**, and this session proved the
point three times: `fn legend`'s injection passed because the resolver took
whichever definition came first; `fn aura`'s passed because "no literal at all"
was read as unreadable, which made the base recursion dead for every
pure-helper factory; and `fn sliver`'s passed because a cost chain the reader
gave up on skipped the card out of the TYPE and SUBTYPE columns too. Each was
invisible in a report that said 0.

So the injections are runnable rather than a list in a docstring. Each case
breaks ONE idiom in the catalog, runs the audit, and restores the file. A case
marked `silent` is a NEGATIVE test: the reader is supposed to be looking
somewhere else, and a row there would be the bug — Bronzehide Lion's BACK face
(the reader takes the returned literal) and `fn ally`'s `mut types`, which is
pushed to before the call it feeds, so binding it to the call site would report
twenty-odd correct Allies.

Run it after touching any reader in `audit_printed_body.py`. It edits catalog
files in place and restores them; `git status` must be clean afterwards.
"""
import pathlib, re, subprocess, sys
ROOT = pathlib.Path("/home/user/Crabomination")
CASES = [
 ("fires", "literal field (Archive Trap)", "sets/mod_set/instants.rs",
  "spell_subtypes: vec![crate::card::SpellSubtype::Trap],", "spell_subtypes: vec![],"),
 ("fires", "base-struct helper (modern fn sliver)", "sets/decks/modern.rs",
  "fn sliver(name: &'static str, c: ManaCost, power: i32, toughness: i32) -> CardDefinition {\n"
  "    CardDefinition {\n        name,\n        cost: c,\n"
  "        card_types: vec![CardType::Creature],\n        subtypes: Subtypes {\n"
  "            creature_types: vec![CreatureType::Sliver],",
  "fn sliver(name: &'static str, c: ManaCost, power: i32, toughness: i32) -> CardDefinition {\n"
  "    CardDefinition {\n        name,\n        cost: c,\n"
  "        card_types: vec![CardType::Creature],\n        subtypes: Subtypes {\n"
  "            creature_types: vec![],"),
 ("fires", "bound parameter (bng3 fn creature)", "sets/bng3.rs",
  "            creature_types: ct,", "            creature_types: vec![],"),
 ("fires", "two-link chain (lgn fn sliver -> fn creature)", "sets/lgn.rs",
  "    creature(name, c, vec![CreatureType::Sliver], p, t)",
  "    creature(name, c, vec![], p, t)"),
 ("fires", "pure-helper base (jou3 fn aura)", "sets/jou3.rs",
  "            enchantment_subtypes: vec![EnchantmentSubtype::Aura],\n", ""),
 ("fires", "inner-literal base (war ..CardDefinition { })", "sets/war.rs",
  "            subtypes: creatures(vec![CreatureType::Golem]),\n            power: 2,\n            toughness: 1,",
  "            subtypes: creatures(vec![]),\n            power: 2,\n            toughness: 1,"),
 ("fires", "Subtypes-returning helper (war fn creatures)", "sets/war.rs",
  "fn creatures(t: Vec<CreatureType>) -> Subtypes {\n    Subtypes {\n        creature_types: t,",
  "fn creatures(t: Vec<CreatureType>) -> Subtypes {\n    Subtypes {\n        creature_types: vec![],"),
 # Negative: the reader must be looking at the RETURNED literal (the front
 # face), so emptying the BACK face's subtype must change nothing.
 ("silent", "the returned literal, not the first (thb Bronzehide Lion back face)",
  "sets/thb.rs",
  "            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],",
  "            enchantment_subtypes: vec![],"),
 # Negative: a `mut` parameter is mutated before the call it feeds, so binding
 # it to the caller's argument would report 20-odd correct Allies.
 ("silent", "a mut parameter is not its caller's argument (zen3 fn ally)",
  "sets/zen3.rs", "    types.push(CreatureType::Ally);\n", ""),
]
def run():
    out = subprocess.run([sys.executable, "scripts/audit_printed_body.py", "--rows", "0"],
                         cwd=ROOT, capture_output=True, text=True).stdout
    rows = len(re.findall(r"^  (?:sub|types|super|cost|p/t|no-cost) ", out, re.M))
    head = next((l for l in out.split("\n") if l.startswith("# compared")), "")
    return rows, head
base, line = run()
print(f"BASELINE {base} rows\n  {line}\n")
bad = 0
for expect, label, rel, old, new in CASES:
    p = ROOT / "crabomination_catalog/src" / rel
    orig = p.read_text()
    n = orig.count(old)
    if n != 1:
        print(f"BROKEN CASE     pattern occurs {n}x   {label}"); bad += 1; continue
    try:
        p.write_text(orig.replace(old, new, 1))
        rows, _ = run()
    finally:
        p.write_text(orig)
    delta = rows - base
    ok = (delta > 0) if expect == "fires" else (delta == 0)
    bad += not ok
    print(f"{'ok' if ok else 'FAILED':8} expect {expect:6} got {delta:+5d} rows   {label}")
print(f"\n{len(CASES) - bad}/{len(CASES)} as expected")
sys.exit(1 if bad else 0)
