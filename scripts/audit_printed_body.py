#!/usr/bin/env python3
"""The fourteenth column: does a card's own COST and BODY match what it prints?

`audit_doc_drift.py` reads the doc comment and compares it with the body and
the oracle, so it audits a factory only where the doc *states* a cost or a
P/T in the codebase's `Name — {2}{R} 3/3 …` shape. This reads the body alone,
so it covers the factories that shape misses: a doc that omits the cost, a doc
that is prose, a factory with no doc at all.

Every other column audit (activated-ability costs, timing, trigger events,
scopes, filters, amounts, ...) found shipped cards at the wrong value; this
one is the card's own price and body, which is the one field every other
column assumes is right.

    python3 scripts/audit_printed_body.py             # the table
    python3 scripts/audit_printed_body.py --rows 0    # every row
    python3 scripts/audit_printed_body.py --list nonliteral   # what a column skipped

COVERAGE, 2026-09-12 (second pass): **16,841 factories priced, 17,707 on the
type line, 17,167 on subtypes, 17,405 on keywords, 9,524 on P/T, 16,695 on
COLOURS and 124 on LOYALTY** — the last two columns are new, and the other five
moved by the idioms their readers could not follow. In order of what they
cost:

  * **a SHORTHAND `keywords,`** — `fn creature(.., keywords: Vec<Keyword>)`,
    how most per-set files spell a creature. 439 factories, read as unreadable
    where the P/T column had bound the same shape since it was opened.
  * **`shorthand()` WAS DEPTH-BLIND**, which is why the keyword one could not
    simply reuse it: a nested `EquipBonus { power, toughness, keywords, .. }`
    matched, so every `simple_aura` and land animation (Flight, the Genju
    cycle, the Zendikons, the Opal enchantments — 16 cards) read the keyword it
    GRANTS as one it prints. It counts braces now, like `raw_field`.
  * **`card_types: vec![if sorcery { Sorcery } else { Instant }]`** — 89
    factories across the deck files, and the branches are two braces deep, so
    the flattened body cannot hold them. It reads `literal_raw` for that field,
    the way the subtype column already does. `top_fields` was also *clearing*
    the line it had collected on a newline inside a nested brace, so the field
    lost its own NAME and read as undeclared.
  * **a base bound to a PARAMETER** — `landfall_self_pump(creature("Scythe
    Leopard", ..), (1, 1), vec![])` writes `..base`, and the chain every reader
    follows is one level out in the caller's argument list.
  * **a binding that is one ELEMENT of the vec** — `planeswalker_subtypes:
    vec![sub]`, every planeswalker in War of the Spark, 68 factories on the
    column `HasPlaneswalkerType` reads.
  * **a TUPLE parameter read field by field** — `..creature(name, mana, pt.0,
    pt.1, ..)`, the bestow creatures, 41 on P/T.
  * **`cost: cost(cost_syms)`** over a `&[ManaSymbol]` parameter — the
    alpha/beta creature helpers, 40 on the cost column.
  * **a MULTI-FACE card was skipped whole** — 276 factories with no column at
    all, the same shape as the `nocache` skip that hid three wrong costs. The
    oracle carries name / mana_cost / type_line / oracle_text / power /
    toughness PER FACE, so the three columns that read those run once the right
    face is picked: **by NAME, not by position.** Scryfall keys a DFC by
    "Plargg, Dean of Chaos // Augusta, Dean of Order" AND by each face name, so
    a lookup on the back face's name returns a card whose `card_faces[0]` is the
    OTHER side — face 0 priced Augusta at `{1}{R}`, which is Plargg's cost. The
    whole-card fields (`colors`, `keywords`, `loyalty`) are not per-face, so
    the columns that read them skip themselves. `faces` 276 -> **0**.
  * **`returned_literal` counted BRACES only**, so a literal passed as an
    ARGUMENT was at depth 0 and won: `sidequest("Sidequest: Catch a Fish",
    cost, .., <the back face's literal>)` builds its front through the helper,
    and the factory read as its own BACK FACE — the Campsite's name and Land
    type line over the enchantment's `{2}{W}`. It counts parens now, and the
    one argument that IS the card (`legend(CardDefinition { .. })`, the six
    Invasion legends) is told apart by being the call's first with nothing
    before it.
  * **a helper CALL inside the `keywords:` vec** — `cycling_two()`, `ward_1()`,
    `cu(CumulativeUpkeepCost::Life(1))`, `basic_landcycling(..)`: 60 factories,
    resolved through an index of `fn .. -> Keyword` bodies, and only where
    every definition of a name agrees on the variant.
  * **the factory's base call is not always the FIRST TOKEN after the
    signature.** `pub fn celestial_colonnade() { use crate::card::Keyword;
    manland("Celestial Colonnade", ..) }` opens with a statement, so the match
    found `use` and **105 factories — every one with a `use` or a `let` before
    their tail — were dropped as `noname` before any column ran**.
    `resolve_from_block` has taken the TAIL expression since `fn ally` taught
    it the same lesson; here it is the FALLBACK rather than the rule, because
    `tail_expression` splits on a `;` one brace deep too and that cuts a
    literal holding a closure (taking it unconditionally cost 125 priced
    factories, which is how the fallback shape was chosen).
  * **the head-string name matched on a PREFIX**, so a token the factory
    defines before the card won it — `fn sliver_queen` took the `"Sliver"` of
    its own token, `fn goblin_marshal` took `"Goblin"`, and the card then
    dropped out of EVERY column as `nocache`. `resolve_card_name` prefers the
    exact `pub fn` slug whenever the head string resolves to no card: 27
    factories back, and one of them (Surging Æther) was `{2}{U}` where the card
    prints `{3}{U}`. `scripts/audit_card_names.py` is the column that reads
    what is left of that skip.

The five columns were 10,270 / 10,409 / 0 / 0 / 5,578 when each was opened. And
the
2026-09-11 header's "219 real spells over ~40 bespoke helpers" was a mis-read of
its own skip counts. The reader had four holes and none of them was the helper
signatures:

  * **the `..base` struct-update form — 2,890 factories, 13 % of the catalog.**
    `CardDefinition { activated_abilities: .., ..creature("Name", cost(&[r()]),
    ..) }` has a literal, so the helper fallback never ran, and the literal has
    no `cost:` of its own, so the literal read found nothing.
  * **the PURE-HELPER factory — 2,392 more.** `pub fn x() -> CardDefinition {
    sorcery("Name", cost(&[generic(2), r()]), effect) }` has no literal at all
    and was dropped as `noname` BEFORE the name was resolved, which also made
    every `body is None` branch in the file dead code.
  * **`crate::mana::w()` and multi-line costs** — a third of the catalog spells
    the symbols with their path, and `top_fields` emits one line per field.
  * **factories named only by their `pub fn`** — resolved through `CACHE_SLUG`.
    Safer than the head-string heuristic it backs up, not less: no string from
    the body is involved.

**AND THE COST IS NOT THE ARGUMENT NEXT TO THE NAME.** It is for
`creature("Name", cost(&[r()]), ..)`; it is NOT for `skullbomb("Name",
mode_cost, ..)`, `keeper("Name", activation_cost, ..)` or `shard("Name",
ability_cost, ..)`, whose helpers print `cost(&[generic(1)])` and
`cost(&[sym, sym])` of their own — fourteen cards a name-anchored reader priced
at their ability's cost, every one of them a false positive it would have
reported forever. `resolve_helper_cost` follows the helper instead: it reads the
helper's own `cost:`, binds a bare parameter name back to the caller's argument,
follows the helper's own base up to five links, and **gives up (skips, never
reports) when the expression is anything else** — `cost(&[sym, sym])` over a
local binding is unreadable and says so.

**THE TYPE LINE IS RARELY IN THE LITERAL EITHER**, which is why that column read
10,409 factories and now reads **16,185**. Three idioms carry it elsewhere and
all three are followed (`resolve_type_line`): the **base-struct** form, the
**wrapper** form (`legend(CardDefinition { .. })` over a
`fn legend(mut def) { def.supertypes = ..; def }`, six Invasion legends), and a
helper whose own base is another helper. A chain it cannot follow is SKIPPED;
the first cut reported 77 rows of "supertypes: (none)" and every one was
`..legend(..)` supplying what the literal never claimed.

**AND THE SUBTYPE HALF IS A COLUMN NOW — 16,748 factories, 0 findings, 22 when
it was opened.** It was left alone for three passes ("the enums are per-kind and
the mapping is a second audit"), and every kind had a live consumer waiting for
it: `Keyword::Splice(cost, SpellSubtype::Arcane)` checks the host's
`spell_subtypes` (25 splice cards), `Effect::Learn` filters the sideboard on
`SpellSubtype::Lesson`, Zendikar's Trapfinder filters on `Trap`, and Nicol
Bolas reads `HasPlaneswalkerType(Bolas)`. So a missing subtype is not cosmetic:
Kodama's Reach could not be spliced onto, Illuminate History could not be
Learned, and Jace, the Mind Sculptor was a `Jace` no filter could see. The
`Subtypes` VALUE is one brace deeper than `top_fields` keeps, so this column
reads `literal_raw` while every other column still reads the flattened text —
one walker, two sources.

Six idioms carry a subtype and all six are followed: the literal field, a
`..base(..)` helper, a helper that is ITSELF a pure call (`fn sliver(..) {
creature(name, c, vec![Sliver], p, t) }`), a bound PARAMETER (`fn creature(..,
ct, ..)`'s `creature_types: ct` — 2,626 factories, the single biggest one), an
`..CardDefinition { .. }` inner-literal base, and a helper that returns a
`Subtypes` (`subtypes: creatures(vec![Golem])`, 600-odd). Every lookup is
same-file-first, which is the `legend` lesson.

The vocabulary is read off `crabomination_base`'s own enums, so a variant added
there is checked without a second edit. A card whose oracle subtype has no
variant at all is `nosubvariant`, NOT a finding: that is missing coverage, and
counting it as "the card is missing this subtype" would report a catalog that
cannot spell the word.

**AND THE KEYWORDS ARE A COLUMN — 16,601 factories, 0 findings, 17 when it was
opened, and it is the one that reaches the SIMULATOR'S HOT PATH.** Combat reads
`keywords` on every block, every damage assignment and every evasion check of
every self-play game, so a wrong one is paid for millions of times. Kurkesh,
Onakke Ancient was a 4/3 with FLYING it does not print; Glorybringer dealt its
4 damage on every attack and untapped anyway, because the exert that gates it
was missing; Dread Drone and Tar Snare were marked `Devoid`, which made two
BLACK Rise-of-the-Eldrazi cards colorless (the oracle's `colors` is `["B"]` for
both, and devoid was printed five years later); and Stonework Packbeast and
Tajuru Paragon were `Changeling` — every creature type — where the card prints
"is also a Cleric, Rogue, Warrior, and Wizard", four.

Only the EVERGREEN subset is compared: the oracle's `keywords` array mixes
keyword abilities with keyword ACTIONS (Scry, Mill, Fight) and ability words
(Landfall), and the engine models most of those as effects, so a whole-set
comparison would be noise rather than a column. ⚠ **The subset was too narrow
at first, and that is how the PROWESS class stayed hidden here.**
`audit_catalog_stats.py` checks Ward, Prowess, Hexproof and Protection too, and
its `kw` column is what turned up four cards whose printed prowess never fired
(the keyword's minted pump was suppressed by any cast trigger) and nine more
carrying the trigger without the keyword. Those four are in the set now — two
oracle-backed readers of one field, and the narrower vocabulary was the half
that cost a bug class. And the array counts keywords
the card GRANTS — Steel Seraph's is `['Prototype', 'Flying', 'Vigilance']` for a
card with flying whose trigger grants "your choice of flying, vigilance, or
lifelink" — so the MISSING direction reads the printed keyword LINES instead
(a line that is nothing but a comma-separated list of keyword words). The other
direction needs no such rule, and is where every finding came from.

Twelve rows are `REVIEWED_KEYWORDS`: a keyword the engine carries instead of the
printed wording it is equivalent to (Cockatrice's deathtouch for "destroy that
creature at end of combat", Exalted Angel's lifelink for "you gain that much
life", Necromancy's flash for "as though it had flash"). Each needed a
reviewer's judgement ONCE; the list is what stops it costing that judgement
every run, which is the same reason `REVIEWED_DEAD_MODES` exists one audit over. A PAYLOADED `Hexproof` /
`Protection` variant is handled by a RULE rather than by twelve more rows:
`HexproofExceptColors`, `ProtectionFromMatching` and the rest are how the
engine spells a printed "can't be the target of nongreen spells" / "has
hexproof unless it's attacking", prose the oracle array does not report as the
plain word, so they are excluded from the comparison. The plain
`Keyword::Hexproof` / `Keyword::Protection` is still compared both ways.

**AND THE SEVENTH COLUMN IS THE PRINTED COLOUR — 16,438 factories, 3 findings.**
Colour is DERIVED, not stored: `printed_color_set` is `color_override` if there
is one, empty under `Devoid`, and otherwise the `color_indicator` unioned with
the cost's pips. The catalog spelled `color_indicator` seven times in 21,795
cards, and **186 of the oracle's 33,045 single-faced entries print a colour
their mana cost cannot carry** — so the column is the gap between those two
numbers. Rograkh, Son of Rohgahh ({0}, red), Ragnarok, Divine Deliverance (no
cost, B/G) and Evermind (no cost, blue) each shipped colorless to `R::HasColor`,
protection- and hexproof-from-colour, devotion, `SpendRestriction` and the
"shares a colour" family. (NOT to the deck builders: `CardBrief::pip_colors` is
the cost's pips, which is the right reading for castability.) It needs the
cost column's chain (the pips) AND the keyword column's (`Devoid`), so it skips
wherever either does. One row is `REVIEWED_COLORS`: Transguild Courier's
all-colours is the CDA it prints, carried as a layer-5 static.

**AND THE EIGHTH IS THE STARTING LOYALTY — 121 planeswalkers, 0 findings, and
it is kept as a gate rather than as a discovery.** CR 306.5b: a walker enters
with loyalty counters equal to the printed number, `CardInstance::new` reads
`base_loyalty` for exactly that, and 0 kills it on entry to CR 704.5i — so the
field is silent when wrong and there was no check on it. 121 of the ~128
planeswalker factories are read; the rest are behind an earlier skip.

**AND THE `no_mana_cost` CHECK HAS BOTH DIRECTIONS NOW.** The missing flag was
checked from the day the audit was written; the flag on a card that DOES print
a cost is the worse half — `CannotCastNoCost` makes it a dead card in every
deck it is in — and had no check at all. Both are crude text reads of the
factory, which is sound because the flag is a plain block-level field and a
nested `TokenDefinition` carries none.

**AND THE P/T COLUMN WAS READING 5,578 OF 9,455 CREATURES AND COUNTING
NEITHER HALF.** It took `power:` / `toughness:` off the factory's own flattened
body and `continue`d — with no skip counter — whenever either was missing,
which is every card built through a `fn creature(.., p, t)` helper: **3,877
creatures, 41 % of them, with no P/T check at all** behind a column that
printed "0 wrong P/T". It walks the chain now, and three readings had to be
right before it read 0 again:

  * a SHORTHAND field (`power,`, which is how every `fn creature(.., power,
    toughness)` writes it) is the PARAMETER, and `bind_param` resolves it —
    reading the absent `power:` as "declares nothing" reported 401 correct
    creatures as 0/0;
  * one half without the other is UNREADABLE, not a 0 — a literal with `power,`
    and no `toughness,` takes the other half from its `..base`, and the naive
    reading reported eight Theros Gods as `6/0`;
  * a STATION card (CR 721, Edge of Eternities Spacecraft) has no P/T until it
    is fully stationed, and the printed one lives in the top `StationBand`'s
    `pt` — which is what the oracle reports, and what this compares, rather
    than the card's genuinely-`Default` 0/0 (21 Spacecraft).

**The coverage was the finding: all 3,877 are correct.** That is worth as much
as a defect would have been, and only because the number is printed now.

⚠ **ONE COLUMN'S SKIP IS NOT THE OTHERS'.** The loop used to `continue` on a
cost it could not read, so 900-odd cards behind a cost chain the resolver gives
up on were dropped from the TYPE and SUBTYPE columns as well — three columns'
coverage decided by the hardest one, and the type line now reads 583 MORE
factories than the cost does. Each column takes its own verdict.

What is left, with the reason — `--list <kind>` prints the factories:
  * `nocache` (3,749) — a name the oracle does not know. Mostly synthesized
    cards, and `scripts/audit_card_names.py` is the column that separates those
    from the misspellings: it shares `resolve_card_name` with this file, and a
    name no card has is audited by NOBODY, so two of them were hiding a wrong
    cost.
  * `nocolors` (909) — the colour column needs BOTH the cost chain (its pips)
    and the keyword chain (`Devoid`), so it skips wherever either does.
  * `noloyalty` (0) — a `base_loyalty` chain it cannot follow. Zero today; a
    row here is a new idiom, not a new card.
  * `nonliteral` (870) — a cost chain the resolver cannot read, mostly a helper
    that builds its cost from a local binding. **700 of them are LANDS**, which
    print no mana cost, so there is nothing for the cost column to compare and
    closing them buys that column nothing; 221 carry a real printed cost.
  * `noname` (101) — a factory whose name is in neither the literal, its head,
    nor its own `pub fn`.
  * `nosubtypes` (144) — a subtype the reader will not guess at: a SHORTHAND
    field (`Subtypes { creature_types, .. }` over a local built by `push`), a
    `mut` parameter (mutated before the call it feeds), or a card that BECOMES
    a creature (`SelfIsCreatureIf`, `creature_off_battlefield`), whose
    definition carries the types it becomes rather than the ones it prints —
    Gideon Blackblade and Grist.
  * `nokeywords` (88) — a `keywords:` the reader will not guess at: a helper
    call in the vec (`cycling_two()`), a local built by `push`, or any other
    non-`Keyword::` element.
  * `nopt` — **CLOSED, 0**. It was 70 until the halves walked the chain
    separately (`merge_pt`): a literal declaring one half of the P/T used to
    read as unreadable, which took out every `power: 1, ..phantom(.., 2, ..)`
    and every 0-power card, whose `power:` is omitted because `Default` is
    already 0. Every creature the oracle prints a numeric P/T for is compared.
  * `notyped` (7) — a type-line chain it cannot follow, and **a `notyped` card
    is audited by NOBODY**: the walk `continue`s there, before every other
    column. It was 338, then 52, and the six shapes that took it to 7 all
    reach their base through something other than a `..helper(` line —
    a tail after a BLOCK statement (`if cond { .. }` closes with a brace, not a
    `;`, and the stray brace failed the anchored match: seven strive spells), a
    `let`-bound tail name (`let mut d = dual_land_with(..); ..; d`, five cycling
    duals), an `if flag { A(..) } else { B(..) }` inside that `let` (four
    Wishes) or as the literal's own base (five `tor2::dreams`), a tail that is
    the helper's own PARAMETER (`fn with_buyback(mut def, c)`, ten cards — the
    columns it MUTATES are blanked one by one rather than read off the
    unmutated argument), and a module-qualified base (`..super::wwk::
    tapped_etb_land(..)`, four Zendikar lands). The seven left are one-offs.
  * `nosubvariant` (12) — above.
  * `faces` (0) — a multi-face factory whose name matches NO face of the card
    it resolves to. Zero today: every one of the 276 matches, so a row here is
    a new shape rather than a new card.
  * `split` / `star` / `notaspell` — documented below, all deliberate.
    `notaspell` also drops a Vanguard avatar named after a card: Maraxus of Keld
    is both, and the oracle lookup cannot tell them apart.

**PROVED BY INJECTION, NOT BY ITS OWN ZERO** — and the injections are
RUNNABLE now rather than a list here: `scripts/audit_printed_body_injections.py`
breaks one idiom at a time and states what the audit must do about it,
**40 / 40 as expected**, four of them NEGATIVE tests (a row there would be the
bug), and it counts rows from `audit_card_names.py` too — a broken NAME is
silent here and loud there. Run it after touching any reader below — and never run the audit while
the battery is running, because it edits catalog files in place.

The cost column's own four (Agent of Stromgald's `..creature(..)`; `instant(
"Consume Strength", ..)`; `fn skullbomb`'s OWN `cost:`; Karn, Scion of Urza to
`{3}` / `Creature` / no supertype) and `god_weapon`'s `supertypes:` are the
older ones, kept here because they cross columns.

**THREE INJECTIONS SILENTLY PASSED BEFORE THE READERS WERE FIXED, and all three
were the same mistake — a resolver reading the wrong definition:**
  * `fn legend` is defined in five files with three different shapes and the
    resolver took whichever came first, so an assigning wrapper read a
    base-struct one's supertypes. Same-file first now, everywhere.
  * the subtype reader called "no literal at all" UNREADABLE, which made the
    base recursion dead for every pure-helper factory: emptying `fn aura`'s
    `EnchantmentSubtype::Aura` came back byte-identical. "No literal" is *not
    declared*, not unreadable.
  * `fn sliver`'s cards were skipped as `nonliteral` — by the COST column —
    before the subtype column ever saw them.

And two readings were wrong in the other direction, reporting correct cards:
a factory with a `let` literal before the returned one (Bronzehide Lion's back
face read as the card), and a shorthand sub-field read as EMPTY rather than
unreadable (34 correct Allies and Mounts). Both are negative cases in the
battery now.

⚠ SKIPPED, with the reason, and the skip counts are printed:
  * multi-face cards (`card_faces` in the oracle) — the factory's `cost:` is
    the front face and the comparison needs a face the body does not name;
  * split cards (`Name // Name`) for the same reason;
  * `*` / `*+1` power or toughness (characteristic-defining) — the body holds
    a placeholder the oracle cannot adjudicate;
  * a `cost:` that is not a literal `cost(&[..])` of symbol constructors
    (`with_x_value`, a `const`, a builder call);
  * a name the cache does not hold (synthesised cards, tokens).
"""
import argparse
import collections
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog" / "src"
CACHE = json.load(open(pathlib.Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items() if isinstance(v, dict)}
# The cache keyed the way a factory name is spelled: lowercase, letters and
# digits only. A factory whose `CardDefinition` carries no `name:` and whose
# head holds no string the old heuristic accepted is still named by its own
# `pub fn` — 1,682 of 1,692 such factories resolve here, and resolving the
# FUNCTION name against the oracle is safer than reading a string out of the
# body, which is how a Spirit token's name once became the card's.
CACHE_SLUG = {re.sub(r"[^a-z0-9]", "", k.lower()): v
              for k, v in CACHE.items() if isinstance(v, dict)}
CACHE_SLUG_KEYS = {k.replace("_", "") for k in ()} | set(CACHE_SLUG)

FN = re.compile(r"^pub fn ([a-z0-9_]+)\(\) -> (?:[A-Za-z_]+::)*CardDefinition \{", re.M)
# ⚠ THE FIELDS MUST COME FROM THE `CardDefinition` LITERAL, NOT THE FACTORY
# BLOCK. A factory commonly opens with a `let token = TokenDefinition { power:
# 1, toughness: 1, .. }` at the same indent, and an `ActivatedAbility` has a
# `cost:` — reading the block by regex gave the Spirit token's 1/1 as
# Heron-Blessed Geist's body and called 117 correct cards wrong. Brace-match
# the literal and read only its own depth-1 fields.
LITERAL = re.compile(r"(?:[A-Za-z_]+::)*CardDefinition \{")
# The same literal anchored at its closing `{`, for "is THIS brace a literal's".
LITERAL_END = re.compile(r"(?:[A-Za-z_]+::)*CardDefinition \{$")
ANYNAME = re.compile(r'"((?:[^"\\]|\\.)*)"')
NAME = re.compile(r'^name: "((?:[^"\\]|\\.)*)",', re.M)
# ⚠ MULTI-LINE AND PATH-TOLERANT. `top_fields` emits one line per depth-1
# field, so a `cost: cost(&[\n  generic(3),\n  r(),\n])` arrives as three
# lines and a single-line pattern misses it; and half the catalog spells the
# symbols `crate::mana::w()` rather than `w()`. Both were counted as
# `nonliteral` — 180-odd factories on the second alone.
COST = re.compile(r"^cost: (?:crate::mana::)?cost\(&\[(.*?)\]\)", re.M | re.S)
# ⚠ NO NAME-ANCHORED COST READER. "The string next to the card's name, followed
# by a `cost(&[..])`" prices `creature("Name", cost(&[r()]), ..)` right and
# `skullbomb("Name", mode_cost, ..)` wrong — fourteen cards read at their
# ABILITY's cost that way. `resolve_helper_cost` follows the helper instead.
# ⚠ ALIAS-TOLERANT ON PURPOSE. The catalog spells these three ways —
# `CardType::Creature`, an aliased `CT::Creature` / `Sup::Legendary`, and a
# helper (`supertypes: legendary()`, `subtypes: types(vec![..])`). A matcher
# keyed on `Supertype::` reported 49 wrong type lines and every one was an
# alias or a helper, clustered in two files, which is the tell.
TYPES = re.compile(r"^card_types: (.*)$", re.M)
SUPER = re.compile(r"^supertypes: (.*)$", re.M)
WORD = re.compile(r"\b([A-Z][a-z]+)\b")
HELPERS = {"legendary": "Legendary", "basic": "Basic", "snow": "Snow"}
POWER = re.compile(r"^power: (-?\d+),", re.M)
TOUGH = re.compile(r"^toughness: (-?\d+),", re.M)


# Every `fn` in the catalog that returns a `CardDefinition` — the helpers a
# factory defers to, not just the factories themselves.
ANYFN = re.compile(r"^(?:pub(?:\(crate\))? )?fn ([a-z0-9_]+)[(<]", re.M)
RET = re.compile(r"->\s*(?:[A-Za-z_]+::)*CardDefinition\s*\{")
# `helper(CardDefinition { .. })` — the WRAPPER idiom, as opposed to the
# `..helper(..)` base-struct one. `fn legend(mut def) { def.supertypes = ..; def }`
# is the shape, and six Invasion legends are built that way.
WRAPS = re.compile(r"^\s*([a-z0-9_]+)\(\s*(?:[A-Za-z_]+::)*CardDefinition\s*\{", re.M)
ASSIGNS = re.compile(r"\bdef\.(supertypes|card_types)\s*=")

# ── the subtype half of the type line ────────────────────────────────────────
# The vocabulary is read off `crabomination_base`, not restated here: a variant
# added there has to reach this audit without a second edit, and a variant
# RENAMED there must not silently stop being checked.
BASE_CARD = ROOT / "crabomination_base" / "src" / "card.rs"
SUB_ENUMS = ("CreatureType", "LandType", "ArtifactSubtype", "EnchantmentSubtype",
             "SpellSubtype", "PlaneswalkerSubtype", "BattleSubtype")
SUB_FIELDS = ("creature_types", "land_types", "artifact_subtypes",
              "enchantment_subtypes", "spell_subtypes", "planeswalker_subtypes",
              "battle_subtypes")


def _subtype_variants():
    text = BASE_CARD.read_text()
    out = {}
    for name in SUB_ENUMS:
        m = re.search(r"pub enum %s \{(.*?)\n\}" % name, text, re.S)
        if not m:
            sys.exit(f"{name} not found — the subtype column reads its variants")
        for v in re.findall(r"\b([A-Z][A-Za-z0-9_]*)\b",
                            re.sub(r"//[^\n]*", "", m.group(1))):
            out.setdefault(v.lower(), v)
    return out


# `"Elf"` -> `CreatureType::Elf`. Lowercased because that is the only difference
# between an oracle word and a variant for all 393 of them.
SUB_WORD = _subtype_variants()
# `CT` is the catalog's one alias for `CreatureType` (two files).
SUBVAR = re.compile(r"\b(?:%s|CT)::([A-Za-z0-9_]+)" % "|".join(SUB_ENUMS))
SUB_FIELD = re.compile(r"\b(%s)\s*:" % "|".join(SUB_FIELDS))
VEC_OF_VARIANTS = re.compile(
    r"vec!\[\s*(?:(?:crate::card::)?(?:%s|CT)::[A-Za-z0-9_]+\s*,?\s*)*\]"
    % "|".join(SUB_ENUMS))
# A local binding assembled by `push` (`let mut subtypes = Subtypes::default();
# subtypes.creature_types.push(..)`) reaches the literal as the SHORTHAND field
# `subtypes,`. The value is not in the literal, so it is unreadable, not empty —
# Storm Crow is the shape, and reading it as empty reported it as a Bird-less
# Bird.
SUB_SHORTHAND = re.compile(r"(?:^|,)\s*subtypes\s*(?:,|$)")
# The same trap one level in: `Subtypes { creature_types, ..Default::default() }`
# over a `let mut creature_types = types.to_vec(); creature_types.push(Ally)`.
# The field-with-colon scan finds nothing there, and "nothing" read as EMPTY
# reported 34 correct cards as subtype-less. A shorthand is unreadable.
# ⚠ A SHORTHAND WHOSE NAME IS A PARAMETER IS THE ARGUMENT, not a blind spot:
# `fn construct(creature_types: Vec<CreatureType>) -> Subtypes { Subtypes {
# creature_types, .. } }` is `bro.rs`'s whole set and 18 more cards elsewhere.
# Unreadable is the answer only for a LOCAL of that name (the `let mut` shape
# above). Captured so the two can be told apart.
SUB_FIELD_SHORTHAND = re.compile(
    r"(?:^|,)\s*(%s)\s*(?:,|$)" % "|".join(SUB_FIELDS), re.M)
# `CardDefinition { .., ..CardDefinition { .., subtypes: .. } }` — a struct-update
# base that is another LITERAL rather than a helper call, so the walker's
# `..name(` scan does not see it (war.rs's Golems, Saheeli's Silverwing).
INNER_BASE = re.compile(r"\.\.(?:crate::card::)*CardDefinition \{")
# The two idioms for "this card becomes a creature": the definition's own
# `creature_types` are then the creature's, not the printed line's.
BECOMES_CREATURE = re.compile(r"\bSelfIsCreatureIf\b|\bcreature_off_battlefield:\s*true")

# ── the printed keywords ─────────────────────────────────────────────────────
# Only the ones BOTH sides spell as a plain keyword: the oracle's `keywords`
# array is a mix of keyword abilities, keyword ACTIONS (Scry, Mill, Fight) and
# ability words (Landfall), and the engine models most of those as effects, so
# a whole-set comparison would be noise. These are the ones with a `Keyword`
# variant and no effect half — and they are the ones combat reads every turn,
# which is why the column is worth having at all.
EVERGREEN = {
    "Flying": "Flying", "Trample": "Trample", "Vigilance": "Vigilance",
    "Haste": "Haste", "Menace": "Menace", "Reach": "Reach",
    "Lifelink": "Lifelink", "First strike": "FirstStrike",
    "Deathtouch": "Deathtouch", "Defender": "Defender",
    "Double strike": "DoubleStrike", "Shroud": "Shroud",
    "Indestructible": "Indestructible", "Flash": "Flash",
    "Intimidate": "Intimidate", "Fear": "Fear", "Infect": "Infect",
    "Wither": "Wither", "Changeling": "Changeling", "Skulk": "Skulk",
    "Shadow": "Shadow", "Horsemanship": "Horsemanship", "Persist": "Persist",
    "Undying": "Undying", "Exert": "Exert", "Devoid": "Devoid",
    # The four `audit_catalog_stats.py` sees and this did not, which is how the
    # prowess class stayed hidden from the reader that follows the helper chain.
    "Prowess": "Prowess", "Ward": "Ward", "Hexproof": "Hexproof",
    "Protection": "Protection",
}
# ⚠ A PAYLOADED HEXPROOF / PROTECTION IS THE PROSE FORM, NOT THE KEYWORD.
# `HexproofExceptColors`, `ProtectionFromMatching`, `HexproofUnlessAttackingOr\
# Blocking` and the rest are how the engine spells a printed "can't be the
# target of nongreen spells", "has hexproof unless it's attacking", "protection
# from the colors of …" — prose that Scryfall's `keywords` array does not report
# as the plain word. Comparing them against it reported twelve correct cards
# (Thrun, Gaea's Revenge, Tromokratis, the two Informers …), so they are
# excluded from the comparison rather than folded into it. The PLAIN
# `Keyword::Hexproof` / `Keyword::Protection` is still compared both ways, which
# is where a real one would show.
PAYLOADED = {
    "HexproofFromColor": "Hexproof", "HexproofFromMonocolored": "Hexproof",
    "HexproofFromMulticolored": "Hexproof", "HexproofFromAbilities": "Hexproof",
    "HexproofExceptColors": "Hexproof", "HexproofUnlessAttackingOrBlocking": "Hexproof",
    "ProtectionFromCreatureType": "Protection", "ProtectionFromSpellSubtype": "Protection",
    "ProtectionFromCardType": "Protection", "ProtectionFromColoredSpells": "Protection",
    "ProtectionFromCreatures": "Protection", "ProtectionFromEverything": "Protection",
    "ProtectionFromInstants": "Protection", "ProtectionFromMonocolored": "Protection",
    "ProtectionFromMulticolored": "Protection", "ProtectionFromSpells": "Protection",
    "ProtectionFromMatching": "Protection", "ProtectionFromManaValueExcept": "Protection",
    "ProtectionFromManaValueParity": "Protection", "ProtectionFromOwnColors": "Protection",
}
EVERGREEN_VARIANTS = set(EVERGREEN.values())
# ⚠ THE ORACLE'S `keywords` ARRAY COUNTS KEYWORDS THE CARD *GRANTS*. Steel
# Seraph's is `['Prototype', 'Flying', 'Vigilance']` and the card is a 5/4 with
# flying whose trigger grants "your choice of flying, vigilance, or lifelink" —
# so the array alone reported a correct card as missing vigilance. A keyword the
# card HAS is on a printed keyword LINE: a line of the oracle text that is
# nothing but a comma-separated list of keyword words ("Flying, first strike,
# lifelink"), or the keyword alone. The other direction — the engine has one the
# card does not print — needs no such rule and is where this column's findings
# came from.
KEYWORD_LINE_WORD = re.compile(r"[A-Za-z][A-Za-z\' -]*")

# A keyword the engine carries INSTEAD of the printed wording it is equivalent
# to. Each needed a reviewer's judgement once; without the list it costs that
# judgement every run, which is what made `REVIEWED_DEAD_MODES` necessary one
# audit over. A card here is exempt for that ONE variant only — every other
# keyword on it is still compared — and a NEW row is still the signal.
REVIEWED_KEYWORDS = {
    # "Whenever this creature deals combat damage to a creature, destroy that
    # creature" is deathtouch in everything combat reads. It is not identical
    # (deathtouch also makes 1 damage lethal for ASSIGNMENT, and the printed
    # ability spares Walls), so these are approximations, not equivalences.
    ("Stinkweed Imp", "Deathtouch"): "destroy-on-combat-damage, printed pre-deathtouch",
    ("Thicket Basilisk", "Deathtouch"): "destroy at end of combat, non-Wall only",
    ("Cockatrice", "Deathtouch"): "destroy at end of combat, non-Wall only",
    # "Whenever this creature deals damage, you gain that much life" is the
    # pre-lifelink templating of lifelink; same result, triggered rather than
    # static.
    ("Exalted Angel", "Lifelink"): "gain-that-much-life trigger, printed pre-lifelink",
    ("Paladin of Prahv", "Lifelink"): "gain-that-much-life trigger, printed pre-lifelink",
    # "is every creature type (even if this card isn't on the battlefield)" IS
    # changeling; the keyword was printed four years later.
    ("Mistform Ultimus", "Changeling"): "is every creature type, printed pre-changeling",
    # "You may cast this spell as though it had flash" — the engine's `Flash`
    # is exactly that permission. The sacrifice rider is the card's own half.
    ("Ward of Lights", "Flash"): "cast as though it had flash",
    ("Necromancy", "Flash"): "cast as though it had flash",
    # "Whenever this becomes the target of a spell an opponent controls,
    # counter it unless its controller {pays / discards}" is ward in everything
    # that matters; both cards were printed before the keyword existed.
    ("Reality Smasher", "Ward"): "counter-unless-discard, printed pre-ward",
    ("Frost Titan", "Ward"): "counter-unless-pay, printed pre-ward",
    # Magecraft is "cast OR COPY an instant or sorcery"; prowess is "cast a
    # noncreature spell". Neither contains the other, and prowess is the
    # engine's standing approximation for it.
    ("Veyran, Voice of Duality", "Prowess"): "magecraft, approximated as prowess",
    # "As long as this permanent is a creature, it has prowess" — the engine
    # grants it unconditionally, which is the same thing: as an Aura it is not
    # a creature and the trigger has nothing to pump.
    ("Triton Wavebreaker", "Prowess"): "prowess while it is a creature",
}

# The colour column's reviewed row: a card whose colour the engine carries as
# the STATIC IT PRINTS rather than as a printed-colour field. Same answer on
# the battlefield (the layer-5 `SetColors`), and the printed clause IS the
# card's own text — so a `color_override` here would be modelling it twice.
REVIEWED_COLORS = {
    "Transguild Courier": "`StaticEffect::GrantAllColors` — the printed CDA",
}


def printed_keyword_lines(text: str) -> set:
    """The evergreen keywords the card itself HAS, off its printed lines."""
    out = set()
    for line in (text or "").split("\n"):
        line = line.strip().rstrip(".")
        if not line or "(" in line:
            line = line.split("(")[0].strip().rstrip(",").rstrip(".")
        parts = [x.strip() for x in line.split(",") if x.strip()]
        if not parts or not all(KEYWORD_LINE_WORD.fullmatch(x) for x in parts):
            continue
        low = {x.lower() for x in parts}
        if not low <= {k.lower() for k in EVERGREEN}:
            continue
        out |= {v for k, v in EVERGREEN.items() if k.lower() in low}
    return out
KW_ELEM = re.compile(r"^(?:crate::card::)?Keyword::([A-Za-z0-9_]+)")
KW_SHORTHAND = re.compile(r"(?:^|,)\s*keywords\s*(?:,|$)")


_KW_HELPERS = None


def keyword_helpers():
    """`fn name(..) -> Keyword { Keyword::X(..) }` — name -> variant.

    A `keywords:` vec can hold a CALL as one of its elements (`cycling_two()`,
    `cu(CumulativeUpkeepCost::Life(1))`, `ward_1()`, `basic_landcycling(..)`),
    and giving up on the whole vec over one of them cost 50 factories their
    keyword check — on the column combat reads every turn, and now that Ward is
    in `EVERGREEN` on a payload this reader still has to see.

    A name defined in several files is resolved only when EVERY definition
    agrees on the variant, which is a stronger rule than the same-file-first one
    the other resolvers use and needs no path threaded through `parse_keywords`.
    `cycling_two` is defined three times and all three return `Cycling`.
    """
    global _KW_HELPERS
    if _KW_HELPERS is None:
        found = {}
        for path in sorted(CATALOG.rglob("*.rs")):
            text = path.read_text()
            for m in re.finditer(r"(?m)^fn ([a-z0-9_]+)\(", text):
                at = m.end() - 1
                close = matching(text, at)
                if close is None:
                    continue
                tail = text[close + 1:close + 60]
                rm = re.match(r"\s*->\s*(?:crate::card::)?Keyword\s*\{", tail)
                if not rm:
                    continue
                body = text[close + 1 + rm.end():close + 400]
                km = re.search(r"(?:crate::card::)?Keyword::([A-Za-z0-9_]+)", body)
                if km:
                    found.setdefault(m.group(1), set()).add(km.group(1))
        _KW_HELPERS = {k: next(iter(v)) for k, v in found.items() if len(v) == 1}
    return _KW_HELPERS


def matching(text: str, at: int):
    """Index of the `)` that closes the `(` at `at`, or None."""
    depth = 0
    for i in range(at, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return i
    return None


def parse_keywords(inner, params=(), args=()):
    """`(keyword variants, declared)` off a literal's raw inner text.

    None means unreadable, exactly as `parse_subtypes` — a `keywords:` built by
    a helper call or a local `push` is not an empty keyword list, and reading it
    as one reports a correct card as keywordless. A `Keyword::Ward(cost)` entry
    keeps its variant name: the payload is not this column's business, and
    skipping the whole card over one payloaded entry would drop the Flying next
    to it.
    """
    if inner is None:
        return set(), False
    # ⚠ A SHORTHAND `keywords,` IS THE PARAMETER, not an unreadable field —
    # the same reading the P/T column needed. `fn creature(.., keywords: Vec<
    # Keyword>)` is how most per-set files spell a creature, so calling the
    # shorthand unreadable skipped 439 factories, 25 of them on one file's
    # helper alone. `field_value` binds it back to the caller's argument and
    # still gives up (returns the bare name, which fails the `vec![` match
    # below) when there is nothing to bind it to.
    f = field_value(inner, "keywords", params, args)
    if f is None:
        bm = INNER_BASE.search(inner)
        if bm:
            nested = literal_raw(inner, bm.start())
            if nested is not None:
                return parse_keywords(nested, params, args)
        return set(), False
    fs = f.strip().rstrip(",")
    m = re.match(r"vec!\[(.*)\]$", re.sub(r"\s+", " ", fs), re.S)
    if not m:
        return None, True
    out = set()
    for elem in split_args(m.group(1)):
        e = elem.strip()
        em = KW_ELEM.match(e)
        if em:
            out.add(em.group(1))
            continue
        # A helper CALL as one element — `cycling_two()`, `ward_1()`, `cu(..)`.
        cm = re.match(r"(?:[a-z0-9_]+::)*([a-z0-9_]+)\s*\(", e)
        v = keyword_helpers().get(cm.group(1)) if cm else None
        if v is None:
            return None, True
        out.add(v)
    return out, True


WRAPPED = re.compile(r"\s*(?:crate::card::)?CardDefinition\s*")


def returned_literal(block: str, after: int):
    """Offset past the `CardDefinition {` the block RETURNS, or None.

    ⚠ NOT THE FIRST ONE. A factory that builds a second face first
    (`let aura_back = CardDefinition { .. }; CardDefinition { .. }`, Bronzehide
    Lion and the MDFCs) has two at the same depth, and reading the first gave
    the BACK face's `Enchantment — Aura` as the card's type line. The returned
    one is the last at brace depth 0 — which is also why depth matters:
    Prismite's `..CardDefinition { .. }` base sits INSIDE the outer literal and
    must not win.
    """
    depth, i, in_str, best, parens = 0, after, False, None, []
    while i < len(block):
        c = block[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "(":
            parens.append(i)
        elif c == ")":
            if parens:
                parens.pop()
        elif c == "{":
            # ⚠ A LITERAL PASSED AS AN ARGUMENT IS NOT THE CARD, and brace
            # depth cannot see that — the braces that would hide it are
            # PARENS. `sidequest("Sidequest: Catch a Fish", cost, .., <the back
            # face's literal>)` builds its front through the helper, so the
            # only literal in the factory is the BACK; read as the card it
            # gave the Campsite's name and Land type line over the
            # enchantment's `{2}{W}` cost.
            # The WRAPPER form is the one argument that IS the card —
            # `legend(CardDefinition { .. })`, the six Invasion legends — and
            # it is the call's FIRST argument with nothing before it, which is
            # what tells the two apart.
            if depth == 0 and LITERAL_END.search(block, after, i + 1) and (
                    not parens
                    or (len(parens) == 1
                        and WRAPPED.fullmatch(block[parens[0] + 1:i]))):
                best = i + 1
            depth += 1
        elif c == "}":
            depth -= 1
        i += 1
    return best


def top_fields(block: str):
    """The depth-1 text of the `CardDefinition { .. }` literal `block` RETURNS,
    one field per line with the indent stripped, or None when there is none.

    Depth counts BRACES ONLY — `cost(&[generic(4), w()])` has to survive whole,
    and a nested `TokenDefinition { power: 1 }` has to not. String literals are
    skipped while counting, because half the catalog's prompts contain `{2}`.
    """
    # Skip the signature line: `pub fn x() -> CardDefinition {` matches the
    # literal regex too, and reading the function body as the literal put every
    # field one level too deep (21,799 factories read as nameless).
    after_sig = block.find("\n") + 1
    start = returned_literal(block, after_sig)
    if start is None:
        return None
    i, depth, out, line, in_str = start, 1, [], [], False
    while i < len(block) and depth:
        c = block[i]
        if in_str:
            if c == "\\":
                if depth == 1:
                    line.append(block[i:i + 2])
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                break
        # ⚠ A NEWLINE INSIDE A NESTED BRACE IS NOT A FIELD BREAK. Resetting
        # `line` on every newline threw away the depth-1 prefix already
        # collected, so a field whose value carries a braced block over several
        # lines — `card_types: vec![if sorcery {\n CardType::Sorcery\n } else
        # {..}]`, 59 factories — lost its own NAME and read as undeclared.
        if c == "\n" and not in_str:
            if depth == 1:
                out.append("".join(line).strip())
                line = []
        elif depth == 1:
            line.append(c)
        i += 1
    out.append("".join(line).strip())
    return "\n".join(out)


def literal_raw_named(block: str, at: int, struct: str):
    """Raw inner text of the `<struct> { .. }` literal opening at/after `at`."""
    m = re.compile(r"(?:[A-Za-z_]+::)*%s \{" % struct).search(block, at)
    if not m:
        return None
    i, depth = m.end(), 1
    in_str = False
    while i < len(block) and depth:
        c = block[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return block[m.end():i]
        i += 1
    return None


def literal_raw(block: str, at: int = 0):
    """The RAW text between the braces of the first `CardDefinition {` literal.

    `top_fields` keeps depth-1 only, which is right for `cost:` and `card_types:`
    and useless for `subtypes:` — a `Subtypes { creature_types: vec![..] }` has
    its whole value one brace deeper, so the flattened line reads
    `subtypes: Subtypes {` and the types are gone. The subtype column reads this
    instead; everything else still reads `top_fields`, so one walker keeps
    following the chain and only the SOURCE differs per field.
    """
    begin = returned_literal(block, at)
    if begin is None:
        return None
    i, depth, in_str, start = begin, 1, False, begin
    while i < len(block) and depth:
        c = block[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return block[start:i]
        i += 1
    return None


def factory_raw(block: str):
    """`literal_raw` past the signature line — the factory's OWN literal."""
    return literal_raw(block, block.find("\n") + 1)


def helper_raw(blk: str):
    """`literal_raw` past a helper's multi-line signature (see `helper_body`)."""
    m = RET.search(blk)
    # `m.end()` is PAST the function body's own brace, so the scan starts inside
    # the body at depth 0 — starting at the brace itself buried every helper's
    # literal one level deep and took 4,223 cards' subtypes to "(none)".
    return literal_raw(blk, m.end()) if m else None


def raw_field(inner: str, field: str):
    """The text of the depth-0 field `field:` in a literal's raw inner text."""
    i, depth, in_str = 0, 0, False
    pat = field + ":"
    while i < len(inner):
        c = inner[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif (depth == 0 and inner.startswith(pat, i)
              and (i == 0 or not (inner[i - 1].isalnum() or inner[i - 1] == "_"))):
            j, d2, s2, start = i + len(pat), 0, False, i + len(pat)
            while j < len(inner):
                c2 = inner[j]
                if s2:
                    if c2 == "\\":
                        j += 2
                        continue
                    if c2 == '"':
                        s2 = False
                elif c2 == '"':
                    s2 = True
                elif c2 in "([{":
                    d2 += 1
                elif c2 in ")]}":
                    if d2 == 0:
                        break
                    d2 -= 1
                elif c2 == "," and d2 == 0:
                    break
                j += 1
            return inner[start:j].strip()
        i += 1
    return None


def shorthand(inner: str, name: str) -> bool:
    """Is `name` present as a SHORTHAND field (`power,`) at depth 0?

    Rust's field-init shorthand means the value IS the identifier, so a helper's
    `fn sliver(.., power, toughness)` writes `power,` and the field's value is
    the parameter — which `bind_param` resolves to the caller's literal. Reading
    the absent `power:` as "declares nothing" instead reported 401 correct
    creatures as 0/0.

    ⚠ DEPTH 0, like `raw_field` — not "anywhere in the text". A nested
    `EquipBonus { power, toughness, keywords, .. }` is the AURA'S BONUS, and a
    depth-blind scan read its three shorthands as the card's own fields: every
    `simple_aura` and land-animation (Flight, the Genju cycle, the Zendikons,
    the Opal enchantments — 16 cards) reported the keyword it GRANTS as one it
    prints. The `keywords:` reader is only sound because this one is.
    """
    i, depth, in_str = 0, 0, False
    while i < len(inner):
        c = inner[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif (depth == 0 and inner.startswith(name, i)
              and not (i and (inner[i - 1].isalnum() or inner[i - 1] == "_"))):
            j = i + len(name)
            while j < len(inner) and inner[j] in " \t\r\n":
                j += 1
            if j >= len(inner) or inner[j] == ",":
                k = i - 1
                while k >= 0 and inner[k] in " \t\r\n":
                    k -= 1
                if k < 0 or inner[k] in ",{":
                    return True
        i += 1
    return False


def field_value(inner, name, params, args):
    """A depth-0 field's value text, shorthand included, parameters bound."""
    v = raw_field(inner, name)
    if v is None:
        v = name if shorthand(inner, name) else None
    return None if v is None else bind_param(v, params, args)


# CR 721 — a STATION card (Edge of Eternities Spacecraft) has no power or
# toughness until it is fully stationed; the printed P/T lives in the top
# `StationBand`'s `pt`, which is what the oracle reports. The card's own
# `power` / `toughness` really are `Default` there, so comparing them would
# report 21 correct Spacecraft as 0/0 — read the band instead.
STATION_PT = re.compile(
    r"\bmin:\s*(\d+)[^}]*?\bpt:\s*Some\(\(\s*(-?\d+)\s*,\s*(-?\d+)\s*\)\)", re.S)


def station_pt(raw: str):
    """The highest station band's `pt`, or None when the card has no band."""
    best = None
    for m in STATION_PT.finditer(raw):
        n = int(m.group(1))
        if best is None or n > best[0]:
            best = (n, (int(m.group(2)), int(m.group(3))))
    return best[1] if best else None


def merge_pt(near, far):
    """`near`'s halves, each falling back to `far`'s — struct-update order.

    ⚠ A HALF IS A VALUE WHEN THE OTHER HALF HAS A SOURCE. `parse_pt` used to
    return "unreadable" for a literal that declared one half, which skipped 70
    creatures out of the column — every `power: 1, ..phantom(name, .., 2, ..)`
    and every 0-power card, whose `power:` is omitted because `Default` is
    already 0 (Wall of Omens, Birds of Paradise, Ornithopter). The two cases
    are one case: the missing half comes from the next link, and `Default`
    (0) is the last link, which is exactly what the column already did for a
    literal that declared NEITHER half.
    """
    if near is None or far is None:
        return near if far is None else far
    return tuple(n if n is not None else f for n, f in zip(near, far))


def pt_incomplete(pt, declared) -> bool:
    """True when the next link still has a half to supply."""
    return not declared or (pt is not None and None in pt)


def parse_pt(inner, params=(), args=()):
    """`((power, toughness) or None, declared)` off a literal's raw inner text.

    ⚠ EITHER HALF MAY BE `None` — see `merge_pt`. `None` for the PAIR is
    unreadable; `None` for one half is "this link does not say", and the base
    chain (then `Default`) supplies it.

    ⚠ THE P/T COLUMN READ 5,578 OF 9,455 CREATURES AND COUNTED NEITHER HALF.
    It took `power:` / `toughness:` off the factory's OWN flattened body and
    `continue`d — with no skip counter — whenever either was missing, which is
    every card built through a `fn creature(.., p, t)` helper: 3,877 creatures,
    41 % of them, with no P/T check at all behind a column that printed
    "0 wrong P/T". Same walk as the other fields now, same `bind_param`.

    None + declared False is "this literal says nothing", and the caller's base
    supplies it. None + declared True is UNREADABLE and skips.
    """
    if inner is None:
        return None, False
    got = {}
    for f in ("power", "toughness"):
        v = field_value(inner, f, params, args)
        if v is None:
            continue
        v = v.strip().rstrip(",")
        if not re.fullmatch(r"-?\d+", v):
            return None, True
        got[f] = int(v)
    if not got:
        bm = INNER_BASE.search(inner)
        if bm:
            nested = literal_raw(inner, bm.start())
            if nested is not None:
                return parse_pt(nested, params, args)
        return None, False
    # ⚠ ONE HALF IS NOT THIS LITERAL'S WHOLE ANSWER. A literal with `power,`
    # and no `toughness,` takes the other half from its `..base`, not from
    # `Default` — reading the missing half as 0 here reported eight Theros Gods
    # as `6/0`. So hand the half back as a half (`merge_pt` fills it from the
    # next link, `Default` last) rather than declaring the card unreadable.
    return (got.get("power"), got.get("toughness")), True


def parse_subtypes(inner, params=(), args=(), subs_index=None, path=None):
    """`(subtypes, declared)` off a literal's raw inner text.

    `subtypes` is None when the field is there but unreadable — a helper call
    (`subtypes: aura()`), a bare binding (`creature_types: types`), a shorthand
    `subtypes,` — because saying nothing beats reporting the reader's own blind
    spot. An absent field is an EMPTY set and NOT declared: the literal claims no
    subtype, and whether a `..base` supplies one is the caller's recursion.

    ⚠ NO LITERAL AT ALL IS "NOT DECLARED", NOT "UNREADABLE". The pure-helper
    factory (`pub fn x() -> CardDefinition { aura("Name", ..) }`) has none, and
    returning unreadable here made the base recursion dead for every one of
    them — 2,392 factories, and the injection that removed `fn aura`'s own
    `EnchantmentSubtype::Aura` came back byte-identical. A factory with neither
    a literal nor a resolvable base is dropped by the type-line test above this
    one, which already requires a non-empty `card_types`.
    """
    if inner is None:
        return set(), False
    f = raw_field(inner, "subtypes")
    if f is None:
        if SUB_SHORTHAND.search(inner):
            return None, True
        bm = INNER_BASE.search(inner)
        if bm:
            nested = literal_raw(inner, bm.start())
            if nested is not None:
                return parse_subtypes(nested, params, args, subs_index, path)
        return set(), False
    if not re.match(r"(?:crate::card::)?Subtypes\s*\{", f.strip()):
        # `subtypes: creatures(vec![CreatureType::Golem])` — a per-file helper
        # that RETURNS a `Subtypes`. 600-odd factories spell it that way, and
        # the helper's own literal is readable once its parameters are bound to
        # this call's arguments; same rule, same give-up, one struct over.
        hm = re.match(r"(?:crate::card::)?([a-z0-9_]+)\s*\(", f.strip())
        if hm and subs_index:
            cands = [c for c in subs_index.get(hm.group(1), []) if c[0] == path] \
                or subs_index.get(hm.group(1), [])
            if cands:
                _, hparams, hlit = cands[0]
                at = f.index("(", f.index(hm.group(1)))
                hargs = [bind_param(a, params, args) for a in call_args(f, at)]
                return parse_subtypes("subtypes: Subtypes {%s}," % hlit,
                                      hparams, hargs, subs_index, path)
        return None, True
    i = f.index("{")
    body, depth = None, 0
    for j in range(i, len(f)):
        if f[j] == "{":
            depth += 1
        elif f[j] == "}":
            depth -= 1
            if depth == 0:
                body = f[i + 1:j]
                break
    if body is None:
        return None, True
    got = set()
    fields = [(m.group(1), None) for m in SUB_FIELD.finditer(body)]
    fields += [(m.group(1), m.group(1)) for m in SUB_FIELD_SHORTHAND.finditer(body)]
    for name, short in fields:
        # Rust field-init shorthand IS the parameter — see SUB_FIELD_SHORTHAND.
        val = short if short is not None else raw_field(body, name)
        if val is None or (short is not None and short not in params):
            return None, True
        val = bind_param(val, params, args)
        # ⚠ THE BINDING CAN BE ONE ELEMENT, not the whole field.
        # `planeswalker_subtypes: vec![sub]` — war's `fn walker`, every
        # planeswalker in the set — is a vec whose single element is the
        # parameter, and binding only the whole expression left 117 factories
        # unreadable on the column `HasPlaneswalkerType` reads.
        vm = re.match(r"vec!\[(.*)\]$", re.sub(r"\s+", " ", val).strip(), re.S)
        if vm and params and vm.group(1).strip():
            val = "vec![%s]" % ", ".join(bind_param(a, params, args)
                                         for a in split_args(vm.group(1)))
        if not VEC_OF_VARIANTS.fullmatch(re.sub(r"\s+", " ", val).strip()):
            return None, True
        got |= set(SUBVAR.findall(val))
    return got, True


def bind_param(expr: str, params, args) -> str:
    """A bare parameter name resolved back to the caller's argument text.

    **The per-set `fn creature(name, mana, ct, p, t)` is how most of the catalog
    spells a creature**, and its literal reads `creature_types: ct` — a binding,
    unreadable on its own. 2,626 factories call one, so leaving it unreadable
    cost the subtype column more than every other skip put together. The cost
    reader has bound its `cost: mana` back to `args[i]` since it was written
    (`resolve_helper_cost`); this is the same rule for the same reason, and it
    gives up the same way when the expression is anything else.
    """
    e = expr.strip().rstrip(",")
    if e.endswith(".clone()"):
        e = e[: -len(".clone()")].strip()
    if e in params:
        i = params.index(e)
        return args[i] if i < len(args) else expr
    # ⚠ A TUPLE PARAMETER IS READ FIELD BY FIELD. `fn bestow_creature(..,
    # pt: (i32, i32), ..) { ..creature(name, mana, pt.0, pt.1, ..) }` hands the
    # next link `pt.0`, which is not a parameter name and not a literal, so the
    # P/T column lost all twelve bestow creatures one link before the value.
    tm = re.fullmatch(r"([a-z0-9_]+)\.(\d+)", e)
    if tm and tm.group(1) in params:
        i = params.index(tm.group(1))
        if i < len(args):
            outer = re.fullmatch(r"\((.*)\)", args[i].strip(), re.S)
            if outer:
                parts = split_args(outer.group(1))
                k = int(tm.group(2))
                if k < len(parts):
                    return parts[k].strip()
    return expr


BASE_PARAM = re.compile(r"^\s*\.\.([a-z0-9_]+)\s*,?\s*$")


def bind_base_param(body: str, params, args) -> str:
    """A `..base` over a `base: CardDefinition` PARAMETER, inlined.

    ⚠ A WRAPPER'S BASE IS NOT ALWAYS A CALL. `fn landfall_self_pump(base:
    CardDefinition, ..) { CardDefinition { keywords, triggered_abilities: ..,
    ..base } }` takes the card itself as an argument, so the chain the other
    resolvers follow (`..creature(..)`) is one level further out — in the
    CALLER's argument list. Rewriting `..base` to the argument text bound to it
    puts the call back where every reader already looks; a base bound to
    anything but a call is left alone and still skips.
    """
    if not params:
        return body
    out = []
    for line in body.split("\n"):
        m = BASE_PARAM.match(line)
        if m and m.group(1) in params:
            i = params.index(m.group(1))
            if i < len(args) and re.match(r"\s*(?:[A-Za-z_]+::)*[a-z0-9_]+\s*\(",
                                          args[i]):
                out.append(".." + args[i].strip())
                continue
        out.append(line)
    return "\n".join(out)


CARD_TYPES = {"Land", "Creature", "Artifact", "Enchantment", "Planeswalker",
              "Battle", "Instant", "Sorcery", "Kindred", "Tribal", "Scheme",
              "Vanguard"}
SUPERTYPES = {"Basic", "Legendary", "Snow", "World", "Ongoing"}
NOT_A_SPELL = ("Token", "Vanguard", "Scheme", "Plane ", "Phenomenon",
               "Conspiracy", "Emblem", "Dungeon", "Attraction", "Stickers")
SYM = re.compile(r"(\w+)\(([^()]*)\)")
LETTER = {"w": "W", "u": "U", "b": "B", "r": "R", "g": "G"}
COLOR = {"Color::White": "W", "Color::Blue": "U", "Color::Black": "B",
         "Color::Red": "R", "Color::Green": "G"}


PIP = re.compile(r"\{([^}]*)\}")
COLOR_TOKEN = re.compile(r"Color::[A-Za-z]+")


def cost_colors(cost: str):
    """The colors a printed mana cost carries — CR 202.2 / 105.

    Every pip half counts: `{W/U}` is both, `{G/P}` is green, `{2/R}` is red.
    Same reading as `printed_color_set`'s match over `ManaSymbol`.
    """
    out = set()
    for p in PIP.findall(cost or ""):
        for half in p.split("/"):
            if half in "WUBRG":
                out.add(half)
    return out


def parse_loyalty(inner, params=(), args=()):
    """`(starting loyalty or None, declared)` off a literal's raw inner text.

    CR 306.5b — a planeswalker enters with loyalty counters equal to the number
    printed in its lower right, and `CardInstance::new` reads `base_loyalty`
    for exactly that. A walker at the wrong number is wrong in every game it is
    in, and the field is a plain `u32` — the same walk as `parse_pt`, including
    `bind_param` for `fn walker(.., loyalty: u32)`.
    """
    if inner is None:
        return None, False
    v = field_value(inner, "base_loyalty", params, args)
    if v is None:
        bm = INNER_BASE.search(inner)
        if bm:
            nested = literal_raw(inner, bm.start())
            if nested is not None:
                return parse_loyalty(nested, params, args)
        return None, False
    v = v.strip().rstrip(",")
    return (int(v), True) if v.isdigit() else (None, True)


def parse_colorfield(inner, params=(), args=()):
    """`((override or None, indicator set), declared)` off a literal's raw text.

    The two printed-colour fields, read through the same chain as the subtypes
    and the P/T: `color_override` replaces the cost's colours outright, a
    `color_indicator` adds to them (CR 105.2c). Unreadable is None, exactly as
    `parse_subtypes` — `fn kobold(name) { CardDefinition { color_override:
    Some(vec![Red]), ..creature(..) } }` puts the field one link out from the
    three Kher Keep Kobolds, and calling that absent would report three
    correct red cards as colorless.
    """
    if inner is None:
        return None, False
    ov = raw_field(inner, "color_override")
    ind = raw_field(inner, "color_indicator")
    if ov is None and ind is None:
        bm = INNER_BASE.search(inner)
        if bm:
            nested = literal_raw(inner, bm.start())
            if nested is not None:
                return parse_colorfield(nested, params, args)
        return (None, set()), False
    def read(f, wrapper):
        m = re.fullmatch(wrapper, bind_param(f, params, args).strip().rstrip(","), re.S)
        if not m:
            return None
        toks = COLOR_TOKEN.findall(m.group(1))
        if any(t not in COLOR for t in toks):
            return None
        return {COLOR[t] for t in toks}
    got_ov = read(ov, r"Some\(vec!\[(.*)\]\)") if ov is not None else None
    if ov is not None and got_ov is None:
        return None, True
    got_ind = read(ind, r"vec!\[(.*)\]") if ind is not None else set()
    if got_ind is None:
        return None, True
    return (got_ov, got_ind), True


def body_cost(src: str):
    """`cost(&[generic(3), r()])` -> "{3}{R}", or None when not literal."""
    out = []
    pos = 0
    # `crate::mana::w()` is the same symbol as `w()`; the paths are what sits
    # "between the symbols" for a third of the catalog.
    src = src.replace("crate::mana::", "")
    for m in SYM.finditer(src):
        if src[pos:m.start()].strip(" ,\n"):
            return None  # something between the symbols we do not model
        pos = m.end()
        fn, arg = m.group(1), m.group(2).strip()
        if fn in LETTER and not arg:
            out.append("{%s}" % LETTER[fn])
        elif fn == "x" and not arg:
            out.append("{X}")
        elif fn == "generic":
            if not arg.isdigit():
                return None
            if arg != "0" or not out:
                out.append("{%s}" % arg)
        elif fn == "colorless":
            if not arg.isdigit():
                return None
            out.append("{C}" * int(arg))
        elif fn == "snow_mana" and not arg:
            out.append("{S}")
        elif fn == "colored":
            if arg not in COLOR:
                return None
            out.append("{%s}" % COLOR[arg])
        elif fn == "hybrid":
            a = [COLOR.get(p.strip()) for p in arg.split(",")]
            if len(a) != 2 or None in a:
                return None
            out.append("{%s/%s}" % (a[0], a[1]))
        elif fn == "phyrexian":
            if arg not in COLOR:
                return None
            out.append("{%s/P}" % COLOR[arg])
        elif fn == "mono_hybrid":
            parts = [p.strip() for p in arg.split(",")]
            if len(parts) != 2 or not parts[0].isdigit() or parts[1] not in COLOR:
                return None
            out.append("{%s/%s}" % (parts[0], COLOR[parts[1]]))
        elif fn == "phyrexian_hybrid":
            a = [COLOR.get(p.strip()) for p in arg.split(",")]
            if len(a) != 2 or None in a:
                return None
            out.append("{%s/%s/P}" % (a[0], a[1]))
        else:
            return None
    if src[pos:].strip(" ,\n"):
        return None
    return "".join(out)


def norm(cost: str) -> str:
    """A `ManaCost` is a multiset, and Scryfall prints pips in a canonical
    WUBRG-rotated order the engine does not model — `{2}{U}{B}{G}` and
    `{2}{B}{G}{U}` are the same cost. Compare the sorted symbols, or the audit
    reports five pip-order rows for every real one (it did)."""
    # `{0}` and an empty cost are the SAME symbol list to the engine — an
    # empty `ManaCost`. What tells Ornithopter (`{0}`, castable) from Living
    # End (no cost, not castable) is `no_mana_cost`, which is the check above,
    # so comparing them here would report Claws of Gix forever.
    return "".join(sorted(p for p in re.findall(r"\{[^}]*\}", cost.upper()) if p != "{0}"))


def split_args(text: str):
    """Top-level comma split of an argument list, brackets and strings aware."""
    out, depth, cur, in_str, esc = [], 0, [], False, False
    for c in text:
        if in_str:
            cur.append(c)
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                in_str = False
            continue
        if c == '"':
            in_str = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                break
            depth -= 1
        elif c == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(c)
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


def call_args(blk: str, at: int):
    """The argument list of the call whose `(` is at `at`."""
    return split_args(blk[at + 1:])


# ⚠ THE PARAMETER LIST CANNOT BE MATCHED WITH `[^)]*`. A tuple parameter
# (`pt: (i32, i32)`) closes the group early, so `fn bestow_creature(name,
# mana, bestow_cost, pt: (i32, i32), ct, kw, bonus)` reads as four
# parameters and every argument after the tuple binds to the wrong name.
# Brace-match instead: `split_args` stops at the unbalanced `)` and
# survives nested calls.
PARAMS = re.compile(r"fn\s+[a-z0-9_]+\s*(?:<[^>]*>)?\s*\(", re.S)
# ⚠ A `mut` PARAMETER IS NOT ITS CALLER'S ARGUMENT. `fn ally(.., mut types:
# Vec<CreatureType>, ..) { types.push(CreatureType::Ally); creature(name, c,
# types, ..) }` hands `creature` a list the caller never wrote, so binding
# `types` back to the call site drops the Ally and reports a correct card.
# Bound as `None`, which fails the literal test and skips.
MUT_PARAM = re.compile(r"\bmut\s+([a-z0-9_]+)\s*:")
COST_FIELD = re.compile(r"^cost: (.*?),?$", re.M)

# Methods on a mutated list that cannot LOSE an element, so folding the pushes
# into the caller's argument still reads the whole printed line. Anything else
# (`retain`, `remove`, `clear`, `pop`, `drain`, `truncate`, an index assignment)
# leaves the param unreadable, which is where it started.
MUT_ADD = ("push", "extend", "extend_from_slice", "insert")
MUT_READ_ONLY = ("contains", "iter", "len", "is_empty", "clone", "to_vec",
                 "as_slice", "first", "last", "sort", "sort_by", "dedup")


def mut_param_arg(src: str, name: str, arg: str):
    """The caller's argument with the helper's own ADDITIONS folded in, or None.

    ⚠ `fn ally(.., mut types: Vec<CreatureType>, ..) { types.push(CreatureType::
    Ally); creature(name, c, types, ..) }` hands `creature` a list the caller
    never wrote, so binding `types` straight back to the call site drops the Ally
    and reports a correct card. That was the reason `helper_params` bound a `mut`
    parameter to `None` — a give-up that skipped **52 Allies** out of the subtype
    column, in a family where the SAME omission had already shipped once (`tla`'s
    `fn ally` was `ct` under a misleading name until 2026-09-01 and ten of its
    callers had no Ally type).

    A give-up is not the only option: read the mutation. Every statement touching
    the parameter must be an add (`MUT_ADD`) or a read (`MUT_READ_ONLY`), and
    every added value a plain variant — then the bound expression is the caller's
    elements plus the helper's, which is exactly the printed line. A conditional
    push (`if !types.contains(&Ally) { types.push(Ally) }`) needs no special case
    because the column compares SETS. Anything else, including a reassignment,
    returns None and the parameter stays unreadable.
    """
    if re.search(r"(?<![=!<>:])\b%s\s*=(?!=)" % re.escape(name), src):
        return None
    added = []
    for m in re.finditer(r"\b%s\s*\.\s*([a-z_]+)\s*\(" % re.escape(name), src):
        meth = m.group(1)
        if meth in MUT_READ_ONLY:
            continue
        if meth not in MUT_ADD:
            return None
        at = m.end() - 1
        vals = call_args(src, at)
        if meth == "insert":
            vals = vals[1:]
        for v in vals:
            v = v.strip().lstrip("&")
            vm = re.fullmatch(r"vec!\[(.*)\]|\[(.*)\]", v, re.S)
            if vm:
                v = vm.group(1) if vm.group(1) is not None else vm.group(2)
            for part in split_args(v) if v.strip() else []:
                part = part.strip()
                if not re.fullmatch(r"(?:crate::card::)?[A-Za-z][A-Za-z0-9_]*"
                                    r"::[A-Za-z0-9_]+", part):
                    return None
                added.append(part)
    if not added:
        return arg
    inner = re.fullmatch(r"\s*(?:&)?(?:vec!)?\[(.*)\]\s*", arg.strip(), re.S)
    if not inner:
        return None
    have = inner.group(1).strip()
    return "vec![%s]" % ", ".join(([have] if have else []) + added)


def bind_mut_params(blk: str, params, args):
    """`(params, args)` with each `mut` parameter either READ or left unreadable.

    The one place the `None` convention of `helper_params` is undone, and only
    for a parameter whose every mutation `mut_param_arg` could account for.
    """
    if None not in params:
        return params, args
    m = RET.search(blk)
    src = blk[m.end():] if m else blk
    names = helper_params_all(blk)
    params, args = list(params), list(args)
    for i, p in enumerate(params):
        if p is not None or i >= len(args) or i >= len(names):
            continue
        bound = mut_param_arg(src, names[i], args[i])
        if bound is not None:
            params[i], args[i] = names[i], bound
    return params, args


def helper_params(blk: str):
    """This helper's parameter names, with a `mut` one replaced by `None`.

    `None` never equals an expression, so `bind_param` leaves the expression
    alone and the literal test skips the card — see `MUT_PARAM`. `bind_mut_params`
    puts the name back for the ones whose mutation it can read.
    """
    m = PARAMS.search(blk)
    if not m:
        return []
    out = []
    for a in split_args(blk[m.end():]):
        out.append(None if MUT_PARAM.match(a.strip()) else a.split(":")[0].strip())
    return out


# A `..base(..)` line's base, THROUGH A MODULE PATH. `..super::wwk::
# tapped_etb_land(..)` is how the Zendikar files reach a shared land helper,
# and an unqualified match read nothing there — four lands with no type line,
# no subtypes and no keywords, while the same helper called WITHOUT the
# struct-update syntax resolved fine.
BASE_CALL = re.compile(r"\.\.(?:[A-Za-z_][A-Za-z0-9_]*::)*([a-z0-9_]+)\s*\(")


def helper_params_all(blk: str):
    """`helper_params`, but keeping a `mut` parameter's NAME.

    Only for the tail-parameter path in `resolve_from_block`, which blanks the
    columns the wrapper mutates one by one instead of dropping the card.
    Everywhere else a `mut` parameter must stay `None`.
    """
    m = PARAMS.search(blk)
    if not m:
        return []
    return [re.sub(r"^mut\s+", "", a.strip()).split(":")[0].strip()
            for a in split_args(blk[m.end():])]


def resolve_helper_cost(index, path, helper, args, depth=0):
    """The PRINTED cost of a card built by `helper(args…)`, as a symbol list.

    **The positional argument next to the name is not the printed cost.** It is
    for `creature("Name", cost(&[r()]), ..)`, and it is NOT for
    `skullbomb("Name", mode_cost, ..)`, `keeper("Name", activation_cost, ..)`
    or `shard("Name", ability_cost, ..)`, whose helpers print `cost(&[generic(1)])`
    and `cost(&[sym, sym])` of their own — fourteen cards the name-anchored
    reader priced at their ability's cost. So follow the helper: read ITS
    `cost:`, bind a bare parameter name back to the caller's argument, and give
    up (skip, never report) when the expression is anything else.
    """
    if depth > 4:
        return None
    cands = [b for pth, b in index.get(helper, []) if pth == path] \
        or [b for _, b in index.get(helper, [])]
    if not cands:
        return None
    blk = cands[0]
    params, args = bind_mut_params(blk, helper_params(blk), args)
    body = helper_body(blk)
    if body is None:
        # A helper that is ITSELF a pure call — `fn sliver(name, c, p, t) {
        # creature(name, c, vec![CreatureType::Sliver], p, t) }` — has no
        # literal, and returning None here skipped its cards as `nonliteral`
        # BEFORE the type and subtype columns ran, which is how a cost the
        # reader could not follow silently cost three columns their cards.
        # Same pseudo-literal step the factory path takes.
        m = RET.search(blk)
        after = blk[m.end():] if m else ""
        hm = re.match(r"\s*(?:[A-Za-z_]+::)*([a-z0-9_]+)\s*\(", after)
        if not hm:
            return None
        at = hm.start() + hm.group(0).rindex("(")
        mapped = [args[params.index(a)] if a in params and params.index(a) < len(args) else a
                  for a in call_args(after, at)]
        return resolve_helper_cost(index, path, hm.group(1), mapped, depth + 1)
    body = bind_base_param(body, params, args)
    m = COST_FIELD.search(body)
    if m:
        expr = m.group(1).strip().rstrip(",")
        if expr in params:
            i = params.index(expr)
            return args[i] if i < len(args) else None
        # ⚠ A HELPER CAN BUILD ITS COST FROM A SYMBOL SLICE. `fn body(..,
        # cost_syms: &[ManaSymbol], ..) { cost: cost(cost_syms), .. }` — the
        # alpha/beta creature helpers — is a `cost(..)` call over the
        # PARAMETER, and the literal match below only knows `cost(&[..])`.
        # Bind it and read the caller's slice.
        cm = re.match(r"(?:crate::mana::)?cost\(([A-Za-z_][A-Za-z0-9_]*)\)$", expr)
        if cm and cm.group(1) in params:
            i = params.index(cm.group(1))
            sm = re.match(r"&\[(.*)\]$", args[i].strip(), re.S) if i < len(args) else None
            return sm.group(1) if sm else None
        inner = re.match(r"(?:crate::mana::)?cost\(&\[(.*)\]\)$", expr, re.S)
        return inner.group(1) if inner else None
    # No `cost:` of its own — follow its base, mapping the base's arguments
    # through this helper's parameters.
    for line in body.split("\n"):
        bm = re.match(BASE_CALL, line)
        if not bm:
            continue
        at = body.index(line) + line.index("(")
        inner_args = call_args(body, at)
        mapped = [args[params.index(a)] if a in params and params.index(a) < len(args) else a
                  for a in inner_args]
        return resolve_helper_cost(index, path, bm.group(1), mapped, depth + 1)
    return None


# A struct FIELD label — `cost:`, `power:` — and not the `::` of a path.
FIELD_LABEL = re.compile(r"(?<!:)\b[a-z_][a-z0-9_]*:(?!:)")


def helper_body(blk: str):
    """`top_fields` for a HELPER, whose signature spans several lines.

    `top_fields` skips one line and then takes the first `CardDefinition {` it
    finds — right for a factory (`pub fn x() -> CardDefinition {` is one line),
    and wrong for `fn god_weapon(\n  name: &'static str,\n ..\n) ->
    CardDefinition {`, where that first match IS the function body's brace and
    the read comes back as `"\n}\n"`. Start after the return type instead.
    """
    m = RET.search(blk)
    if not m:
        return None
    body = top_fields("fn _()\n" + blk[m.end():])
    if body is None:
        return None
    # A one-line literal (`CardDefinition { name, cost: c, card_types: .., .. }`
    # — most of the small per-file helpers) arrives as ONE line, because
    # `top_fields` emits a field per newline. Split it on top-level commas or
    # every `^field:` read below misses: 342 `enchantment(..)` cards read as
    # "this helper has no cost" for exactly that reason.
    #
    # ⚠ THE SPLIT MUST NOT EAT THE COMMAS. `line.count(":") > 1` matches any
    # line carrying two PATHS as well as a two-field literal — including
    # `vec![CreatureType::Phyrexian, CreatureType::Zombie,
    # CreatureType::Knight],`, one ARGUMENT of a multi-line `..base(..)` call —
    # and `split_args` drops the separators. That fused the argument with the
    # `3,` on the next line, so `usg2::creature`'s `p`/`t` bound one argument
    # short and both Paladins read as unreadable P/T. Tightening the predicate
    # instead is the wrong half: `..base(` is matched per LINE, so a one-field
    # line that stays fused hides the base call (121 cards left the type-line
    # column when that was tried). Split as before, and put every comma back.
    out = []
    for line in body.split("\n"):
        if line.count(":") > 1 and "," in line:
            parts = [x.strip() for x in split_args(line)]
            if parts:
                tail = "," if line.rstrip().endswith(",") else ""
                parts = [p + "," for p in parts[:-1]] + [parts[-1] + tail]
            out.extend(parts)
        else:
            out.append(line)
    return "\n".join(out)


SUB_RET = re.compile(r"^(?:pub(?:\(crate\))? )?fn ([a-z0-9_]+)\s*\(([^)]*)\)\s*->\s*"
                     r"(?:crate::card::)?Subtypes\s*\{", re.M)


def build_subtypes_index(catalog):
    """`name -> [(path, params, literal text)]` for every `fn .. -> Subtypes`.

    600-odd factories spell their subtypes through one — `subtypes: creatures(
    vec![CreatureType::Golem])`, `spirit(..)`, `arcane()`, `aura()` — and the
    same per-file shadowing applies (`creatures` is defined in a dozen files),
    so this is same-file-first like every other lookup here.
    """
    index = {}
    for path in sorted(catalog.rglob("*.rs")):
        text = path.read_text()
        for m in SUB_RET.finditer(text):
            lit = literal_raw_named(text, m.end() - 1, "Subtypes")
            if lit is not None:
                params = [a.split(":")[0].replace("mut ", "").strip()
                          for a in split_args(m.group(2))]
                index.setdefault(m.group(1), []).append((path, params, lit))
    return index


def build_helper_index(catalog):
    """`name -> [(path, block)]` for every fn in the catalog that returns a
    `CardDefinition`, so a factory's `..base` can be resolved to its types."""
    index = {}
    for path in sorted(catalog.rglob("*.rs")):
        text = path.read_text()
        hits = [(m.start(), m.group(1)) for m in ANYFN.finditer(text)]
        for i, (pos, fn) in enumerate(hits):
            end = hits[i + 1][0] if i + 1 < len(hits) else len(text)
            blk = text[pos:end]
            if RET.search(blk[:400]) or ASSIGNS.search(blk):
                index.setdefault(fn, []).append((path, blk))
    return index


TERNARY = re.compile(r"^\s*vec!\[\s*if\s+([a-z0-9_]+)\s*\{(.*?)\}\s*else\s*\{(.*?)\}\s*\]",
                     re.S)


def ternary_types(inner, params, args, vocab):
    """`card_types: vec![if flag { A } else { B }]`, resolved through the flag.

    ⚠ The BRANCHES are two braces deep, so the flattened body cannot hold them
    (`top_fields` keeps depth 1 only) and the field reads as an empty type set.
    `fn spell(name, mana, sorcery: bool, effect)` is how five of the deck files
    spell every instant and sorcery — 59 factories with no type-line check at
    all, which is the field where a Sorcery shipped as an Instant is castable
    at the wrong speed. An unbound flag stays unreadable and skips.
    """
    f = raw_field(inner, "card_types") if inner else None
    if f is None:
        return None
    m = TERNARY.match(f)
    if not m:
        return None
    cond = bind_param(m.group(1), params, args).strip()
    if cond not in ("true", "false"):
        return None
    branch = m.group(2) if cond == "true" else m.group(3)
    return {w for w in WORD.findall(branch) if w in vocab}


def parse_type_line(body: str, params=(), args=(), inner=None):
    """`(card_types, supertypes, base_helper, declares_types, declares_supers)`
    off one depth-1 field text.

    ⚠ THE CARD TYPE IS A PARAMETER IN 190 FACTORIES. `fn spell(name, mana, kind,
    effect) { CardDefinition { card_types: vec![kind], .. } }` is how five sets
    spell every instant and sorcery, and `vec![kind]` holds no `CardType` word,
    so the chain read as unresolvable and those cards had NO type-line check at
    all — the field where an Instant shipped as a Sorcery is castable at the
    wrong speed. `bind_param` was already doing this for the cost and the
    subtypes; it does it here too.
    """
    tm, sm = TYPES.search(body), SUPER.search(body)
    def words(field, vocab):
        raw = field.group(1)
        got = {w for w in WORD.findall(raw) if w in vocab}
        if not got and params:
            bound = " ".join(bind_param(a, params, args)
                             for a in split_args(raw.strip().lstrip("vec![").rstrip("],")))
            got = {w for w in WORD.findall(bound) if w in vocab}
        return got
    got_t = words(tm, CARD_TYPES) if tm else set()
    if not got_t:
        tern = ternary_types(inner, params, args, CARD_TYPES)
        if tern:
            got_t, tm = tern, tm or True
    got_t = {"Kindred" if t == "Tribal" else t for t in got_t}
    got_s = set()
    if sm:
        got_s = words(sm, SUPERTYPES)
        got_s |= {v for k, v in HELPERS.items() if k + "()" in sm.group(1)}
    base = None
    for line in body.split("\n"):
        if line.startswith("..") and not line.startswith("..Default::default"):
            m = re.match(BASE_CALL, line)
            if m:
                base = m.group(1)
    return got_t, got_s, base, tm is not None, sm is not None


Read = collections.namedtuple(
    "Read", "types supers subs keywords pt colorfield loyalty "
            "ok sub_ok kw_ok pt_ok col_ok loy_ok")


def tail_expression(body: str) -> str:
    """A function body's TAIL expression — what it returns.

    ⚠ NOT THE FIRST TOKEN AFTER THE SIGNATURE. `fn ally(.., mut types, ..) {
    types.push(CreatureType::Ally); creature(name, c, types, p, t) }` opens with
    a statement, and reading from the top found `types` followed by `.` rather
    than `(` — so 43 Allies, and every helper shaped like them, resolved to
    nothing and were dropped from the type-line column.

    ⚠ A BLOCK STATEMENT ENDS THE PREVIOUS STATEMENT TOO, and splitting on `;`
    alone does not see that. `fn strive_pump(..) { let mut body = vec![]; if
    (power, toughness) != (0, 0) { body.push(..); } .. strive(name, mana, kind,
    ..) }` closes its last `if` with a BRACE, so the chunk after the last `;`
    began `}\n strive(` — and the caller's anchored `ident(` match failed on
    the stray brace, which is why seven Journey into Nyx strive spells and two
    more helpers had no type line. Take the text after the last statement
    boundary — a `;` at depth 0 or the `}` that closes a depth-0 block — and
    stop at the brace that closes the function, which is also what keeps the
    doc comment of the NEXT factory out of the tail.
    """
    depth, in_str, start, i = 0, False, 0, 0
    while i < len(body):
        c = body[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth < 0:
                break
            if depth == 0 and c == "}":
                start = i + 1
        elif c == ";" and depth == 0:
            start = i + 1
        i += 1
    return body[start:i]


def braced(text: str, at: int):
    """`(inner, index past the closing brace)` for the `{` at `at`."""
    depth, in_str, i = 0, False, at
    while i < len(text):
        c = text[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[at + 1:i], i + 1
        i += 1
    return None, len(text)


IF_ELSE = re.compile(r"^\s*if\s+(!?[a-z_][a-z0-9_]*)\s*\{", re.S)


def choose_branch(expr: str, params=(), args=()):
    """`if flag { A } else { B }` resolved through the BOUND flag, or None.

    The same rule `ternary_types` uses for a card type, one level up: here the
    branches are whole BASE CALLS, which is how `jud2::wish` picks between
    `instant(..)` and `sorcery(..)` — and a Wish whose speed cannot be read is
    a card the type-line column drops entirely.
    """
    m = IF_ELSE.match(expr)
    if not m:
        return None
    flag = m.group(1)
    val = bind_param(flag.lstrip("!"), params, args).strip().rstrip(",")
    if val not in ("true", "false"):
        return None
    yes, after = braced(expr, m.end() - 1)
    if yes is None:
        return None
    em = re.match(r"\s*else\s*\{", expr[after:])
    if not em:
        return None
    no, _ = braced(expr[after:], em.end() - 1)
    return yes if (val == "true") != flag.startswith("!") else no


def let_bound(src: str, tail: str, params=(), args=()):
    """The initialiser of the `let` that bound a bare tail identifier.

    `fn cycling_dual(..) { let mut d = dual_land_with(..); d.keywords.push(..);
    d }` returns a NAME, not a call, so the chain stopped at the identifier and
    five cycling duals had no type line, no subtypes and no keywords. The name
    is a base call one statement up.
    """
    ident = tail.strip()
    if not re.fullmatch(r"[a-z_][a-z0-9_]*", ident):
        return None
    m = re.search(r"\blet\s+(?:mut\s+)?%s\b\s*(?::[^=;]*)?=\s*" % re.escape(ident), src)
    if not m:
        return None
    depth, in_str, i = 0, False, m.end()
    while i < len(src):
        c = src[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth < 0:
                break
        elif c == ";" and depth == 0:
            break
        i += 1
    init = src[m.end():i]
    return choose_branch(init, params, args) or init


# A field of the wrapped definition, and the verdict flag it invalidates when
# the wrapper MUTATES it. `fn with_buyback(mut def, c) { def.keywords.push(..);
# def }` returns its own parameter, so the card is readable through the
# argument — for every column the wrapper leaves alone. The keyword column is
# the one it changes, and reading the argument there would report the buyback
# cycle as printing a keyword it does not have, which is the mistake
# `zen3::ally`'s `types.push(Ally)` is the standing negative test for.
MUT_COLUMN = {"card_types": "ok", "supertypes": "ok", "subtypes": "sub_ok",
              "keywords": "kw_ok", "power": "pt_ok", "toughness": "pt_ok",
              "color_indicator": "col_ok", "base_loyalty": "loy_ok"}
CALL_HEAD = re.compile(r"\s*(?:[A-Za-z_]+::)*([a-z0-9_]+)\s*\(")


def resolve_from_block(index, path, blk, args, depth, subs_index=None):
    """`resolve_type_line` for a HELPER block, with the caller's arguments.

    A helper that is itself a pure call (`fn sliver(..) { creature(name, c,
    vec![CreatureType::Sliver], p, t) }`) has no literal, so `helper_body`
    returns None and the chain used to stop one link short of the value. Stand
    in the same pseudo-literal the factory path uses, and pass the call text so
    the base's arguments can be read out of it.
    """
    body, inner = helper_body(blk), helper_raw(blk)
    call_src, blanked = None, ()
    params, args = bind_mut_params(blk, helper_params(blk), args)
    if body is None:
        m = RET.search(blk)
        src = blk[m.end():] if m else ""
        after = tail_expression(src)
        hm = CALL_HEAD.match(after)
        if not hm:
            # The tail is not a call: a bare `let`-bound name, or an
            # `if flag { .. } else { .. }` over two of them.
            alt = (choose_branch(after, params, args)
                   or let_bound(src, after, params, args))
            hm = CALL_HEAD.match(alt or "")
            if hm:
                after = alt
        if not hm and re.fullmatch(r"\s*[a-z_][a-z0-9_]*\s*", after):
            # The tail is the helper's own PARAMETER — the wrapper idiom with
            # a wrapped CALL rather than a literal, which is how ten buyback
            # and rider cards are built. Every column the wrapper does not
            # mutate reads through the argument; the ones it does are blanked
            # rather than read off the unmutated definition.
            ident = after.strip()
            bound = bind_param(ident, helper_params_all(blk), args)
            hm = CALL_HEAD.match(bound)
            if hm:
                after = bound
                blanked = {MUT_COLUMN[f] for f in
                           re.findall(r"\b%s\.([a-z_]+)" % re.escape(ident), src)
                           if f in MUT_COLUMN}
        if not hm:
            return Read(set(), set(), None, None, None, None, None,
                        False, False, False, False, False, False)
        body, call_src = ".." + hm.group(1) + "(", after
    read = resolve_type_line(index, path, body, None, depth, inner,
                             params, args, call_src, subs_index)
    return read._replace(**{f: False for f in blanked}) if blanked else read


def resolve_type_line(index, path, body, raw, depth=0, inner=None,
                      params=(), args=(), call_src=None, subs_index=None):
    """The card's types, supertypes and SUBTYPES, following `..base` and wrappers.

    Returns `(types, supers, subs, ok, sub_ok)`. `ok` is False when a link in the
    chain is a helper this cannot find — saying nothing beats reporting the
    reader's own blind spot, which is how 77 `..legend(..)` cards read as
    "supertypes: (none)" the first time this column was opened. `sub_ok` is the
    same verdict for the subtype half, kept apart because a chain can be
    readable for one and not the other: the subtype column skips on its own
    without costing the type column a card.

    `inner` is the literal's RAW text (`factory_raw` / `helper_raw`); `body` is
    the flattened `top_fields` / `helper_body` one. One walk, two sources — see
    `literal_raw`. `params`/`args` bind this level's helper parameters back to
    what its caller passed (`bind_param`), and `call_src` is the text holding
    the base call when `body` is the pseudo-literal, which carries no arguments.
    """
    body = bind_base_param(body, params, args)
    got_t, got_s, base, has_t, has_s = parse_type_line(body, params, args, inner)
    # A CONDITIONAL BASE: `..if sorcery_speed { sorcery(name, c, effect) } else
    # { instant(name, c, effect) }` — `tor2::dreams`, five buyback-shaped
    # cards whose SPEED is the flag. The branches are two braces deep, so the
    # flattened body cannot hold them and the base scan above reads nothing;
    # take them off the raw literal and bind the flag like any other argument.
    cond_src = None
    if base is None and inner:
        bm = re.search(r"\.\.if\s+!?[a-z_][a-z0-9_]*\s*\{", inner)
        if bm:
            branch = choose_branch(inner[bm.start() + 2:], params, args)
            hm = CALL_HEAD.match(branch or "")
            if hm:
                base, cond_src = hm.group(1), branch
    got_sub, has_sub = parse_subtypes(inner, params, args, subs_index, path)
    got_kw, has_kw = parse_keywords(inner, params, args)
    got_pt, has_pt = parse_pt(inner, params, args)
    got_col, has_col = parse_colorfield(inner, params, args)
    got_loy, has_loy = parse_loyalty(inner, params, args)
    sub_ok, kw_ok = got_sub is not None, got_kw is not None
    pt_ok = not (has_pt and got_pt is None)
    col_ok = got_col is not None
    loy_ok = not (has_loy and got_loy is None)
    ok = True
    # The wrapper idiom sits OUTSIDE the literal, so it is read off `raw`.
    if raw is not None and depth == 0:
        after_sig = raw[raw.find("\n") + 1:]
        wm = WRAPS.match(after_sig)
        if wm:
            # Same-file first: `legend` is defined in five files with three
            # different shapes, and taking whichever came first made an
            # ASSIGNING wrapper read the supertypes of somebody else's
            # base-struct one — the injection that proved it silently passed.
            cands = [c for c in index.get(wm.group(1), []) if c[0] == path] \
                or index.get(wm.group(1), [])
            for wpath, wblk in cands:
                if ASSIGNS.search(wblk) or (helper_body(wblk) or ""):
                    wb = helper_body(wblk) or ""
                    wt, ws, _, wht, whs = parse_type_line(wb)
                    if not has_s and (whs or "def.supertypes" in wblk):
                        # An assigning wrapper's own value, or its literal's.
                        got_s = ws or {
                            w for w in WORD.findall(wblk) if w in SUPERTYPES
                        }
                        has_s = True
                    if not has_t and wht:
                        got_t, has_t = wt, True
                    if not has_sub:
                        wsub, whsub = parse_subtypes(helper_raw(wblk), (), (), subs_index, wpath)
                        if whsub:
                            got_sub, sub_ok, has_sub = wsub, wsub is not None, True
                    if not has_kw:
                        wkw, whkw = parse_keywords(helper_raw(wblk))
                        if whkw:
                            got_kw, kw_ok, has_kw = wkw, wkw is not None, True
                    if pt_incomplete(got_pt, has_pt):
                        wpt, whpt = parse_pt(helper_raw(wblk))
                        if whpt:
                            if wpt is None:
                                got_pt, pt_ok = None, False
                            else:
                                got_pt = merge_pt(got_pt if has_pt else None, wpt)
                            has_pt = True
                    if not has_col:
                        wc, whc = parse_colorfield(helper_raw(wblk))
                        if whc:
                            got_col, col_ok, has_col = wc, wc is not None, True
                    if not has_loy:
                        wl, whl = parse_loyalty(helper_raw(wblk))
                        if whl:
                            got_loy, loy_ok, has_loy = wl, wl is not None, True
                    break
            else:
                ok = sub_ok = kw_ok = pt_ok = col_ok = loy_ok = False
    if base:
        cands = [b for pth, b in index.get(base, []) if pth == path] \
            or [b for _, b in index.get(base, [])]
        if not cands or depth >= 5:
            return Read(got_t, got_s, got_sub, got_kw, got_pt, got_col, got_loy,
                        False, False, False, False, False, False)
        # The base call's arguments, mapped through THIS level's parameters, so
        # `fn sliver(name, c, p, t) { creature(name, c, vec![Sliver], p, t) }`
        # hands `creature` a literal and `fn creature(.., ct, ..)`'s
        # `creature_types: ct` resolves to it. Same mapping as
        # `resolve_helper_cost`'s.
        src = cond_src or (call_src if call_src is not None else body)
        bm = re.search(r"\.\.%s\s*\(" % re.escape(base), src) \
            or re.search(r"\b%s\s*\(" % re.escape(base), src)
        bargs = call_args(src, bm.start() + bm.group(0).rindex("(")) if bm else []
        mapped = [bind_param(a, params, args) for a in bargs]
        b = resolve_from_block(index, path, cands[0], mapped, depth + 1, subs_index)
        bsub, bsok = b.subs, b.sub_ok
        ok = ok and b.ok
        if not has_t:
            got_t = b.types
        if not has_s:
            got_s = b.supers
        if not has_kw:
            got_kw, kw_ok = b.keywords, b.kw_ok
        if pt_incomplete(got_pt, has_pt):
            if not b.pt_ok:
                pt_ok = False
            else:
                got_pt = merge_pt(got_pt if has_pt else None, b.pt)
            has_pt = True
        if not has_col:
            got_col, col_ok, has_col = b.colorfield, b.col_ok, True
        if not has_loy:
            got_loy, loy_ok, has_loy = b.loyalty, b.loy_ok, True
        # A `..base(..)` supplies the subtypes the literal did not declare —
        # `..creature("Storm Crow", .., types, 1, 2)`. A literal that DID
        # declare them wins, as the struct-update syntax says.
        if not has_sub:
            got_sub, sub_ok = bsub, bsok
    # Every card has card types; an empty set means the chain was not readable,
    # not that the card has none. Supertypes are genuinely optional, so they
    # cannot carry this test.
    return Read(got_t, got_s, got_sub, got_kw, got_pt, got_col, got_loy,
                ok and bool(got_t), sub_ok, kw_ok, pt_ok, col_ok, loy_ok)


def resolve_card_name(body, raw: str, fname: str):
    """The card's name, from its literal, its head string, or its `pub fn`.

    ONE reader, because `audit_card_names.py` needs the same answer this file's
    `nocache` skip is decided by — a second copy of the heuristic disagreed with
    this one the day it was written.

    A helper-built factory (`sorcery("Name", cost(..), ..)`) has no `name:`
    field, so a string literal from its head stands in — but it is checked
    against the FUNCTION NAME, or the first token a factory defines is read as
    the card ("Spirit" and "Centaur" both showed up that way, and a 4-line
    window to dodge them lost 3,900 real factories).

    ⚠ THE HEAD STRING MATCHES ON A PREFIX, so a TOKEN the factory defines
    before the card can still win it: `fn sliver_queen` takes the `"Sliver"` of
    its own token because the slug starts with it, and `fn goblin_marshal`
    takes `"Goblin"`. The `pub fn` lookup is EXACT, so it wins whenever the
    head string resolves to no card at all — a prefix guess beaten by the
    card's own name. 27 factories came back into every column that way.
    """
    nm = NAME.search(body) if body is not None else None
    name = nm.group(1) if nm else None
    if name is None:
        head = "\n".join(raw.split("\n")[:14])
        slug = fname.replace("_", "")
        for cand in ANYNAME.findall(head):
            flat = re.sub(r"[^a-z0-9]", "", cand.lower())
            if flat.startswith(slug[:10]) or slug.startswith(flat[:10]):
                name = cand
                break
    slug = fname.replace("_", "")
    if (name is None or (name not in CACHE and name.lower() not in CACHE_LC)) \
            and slug in CACHE_SLUG_KEYS:
        name = CACHE_SLUG[slug]["name"]
    return name


def factory_call_src(raw: str) -> str:
    """The factory text the pure-helper readers take their base call from.

    The call is usually the first token after the signature; when a `use` or a
    `let` comes first it is the TAIL expression instead. Taking the tail
    unconditionally costs more than it buys — `tail_expression` splits on a `;`
    one brace deep as well, which cuts a literal holding a closure — so the
    plain reading wins whenever it finds a call at all.
    """
    after = raw[raw.find("\n") + 1:]
    head = re.match(r"\s*(?:[A-Za-z_]+::)*[a-z0-9_]+\s*\(", after)
    return after if head else tail_expression(after)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rows", type=int, default=40)
    ap.add_argument("--list", dest="list_kind", default=None,
                    help="print the factories skipped under this kind")
    args = ap.parse_args()

    index = build_helper_index(CATALOG)
    subs_index = build_subtypes_index(CATALOG)
    checked = wrong_cost = wrong_pt = missing_flag = wrong_types = 0
    checked_types = checked_sub = checked_kw = wrong_kw = checked_pt = 0
    checked_col = wrong_col = checked_loy = wrong_loy = stray_flag = 0
    # A skip is a FACTORY, not a tally: `--list nonliteral` prints the ones a
    # column could not read, which is the only way to work the queue down.
    skip = {k: [] for k in ("nocache", "faces", "split", "star", "nonliteral",
                            "noname", "notaspell", "notyped", "nosubtypes",
                            "nosubvariant", "nokeywords", "nopt", "nocolors",
                            "noloyalty", "subtwoface")}
    rows = []
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text()
        starts = [(m.start(), m.group(1)) for m in FN.finditer(src)]
        for i, (pos, fname) in enumerate(starts):
            end = starts[i + 1][0] if i + 1 < len(starts) else len(src)
            raw = src[pos:end]
            # ⚠ STOP AT THE NEXT `fn`, NOT AT THE NEXT FACTORY. `starts` is the
            # PUBLIC factories, so a private helper defined under one of them
            # (`fn sac_on_damage_aura(..) -> CardDefinition {`, jou3) sits inside
            # its block — and `top_fields` / `factory_raw` then read the HELPER's
            # literal as the card's. Armament of Nyx was priced off
            # `sac_on_damage_aura`'s body that way. It degraded to a skip here
            # rather than a false row, but it is the `legend` lesson again: a
            # resolver that takes whichever definition comes first.
            nxt = ANYFN.search(raw, raw.find("\n") + 1)
            if nxt:
                raw = raw[:nxt.start()]
            body = top_fields(raw)
            pseudo = body is None
            if body is None:
                # The PURE-HELPER factory — `pub fn x() -> CardDefinition {
                # sorcery("Name", cost(&[generic(2), r()]), effect) }` — has no
                # literal at all. It used to be dropped here, before the name
                # was even resolved, which made every `body is None` branch
                # below dead code and left **2,392 factories audited by
                # nobody**. Stand in a pseudo-literal that defers to the
                # top-level call, so the type-line resolver treats it as a base
                # and the cost reader follows the helper it calls.
                # ⚠ NOT THE FIRST TOKEN AFTER THE SIGNATURE. `pub fn
                # celestial_colonnade() { use crate::card::Keyword;
                # manland("Celestial Colonnade", ..) }` opens with a statement,
                # and matching from the top found `use` rather than the call —
                # so 186 factories, every one with a `use` or a `let` before
                # their tail, were dropped as `noname` before any column ran.
                # `resolve_from_block` has taken the TAIL expression since `fn
                # ally` taught it the same lesson; the factory path did not.
                # ⚠ NOT ALWAYS THE FIRST TOKEN AFTER THE SIGNATURE. `pub fn
                # celestial_colonnade() { use crate::card::Keyword;
                # manland("Celestial Colonnade", ..) }` opens with a statement,
                # and matching from the top found `use` rather than the call —
                # 105 factories, every one with a `use` or a `let` before their
                # tail, dropped as `noname` before any column ran.
                # `resolve_from_block` has taken the TAIL expression since `fn
                # ally` taught it the same lesson; here it is the FALLBACK, not
                # the rule: `tail_expression` splits on a `;` one brace deep
                # too, which cuts a literal that holds a closure or a `let`.
                after_sig = raw[raw.find("\n") + 1:]
                hm = re.match(r"\s*(?:[A-Za-z_]+::)*([a-z0-9_]+)\s*\(", after_sig) \
                    or re.match(r"\s*(?:[A-Za-z_]+::)*([a-z0-9_]+)\s*\(",
                                tail_expression(after_sig))
                body = ".." + hm.group(1) + "(" if hm else None
                if body is None:
                    skip["noname"].append(f"{path.name}::{fname}")
                    continue
            name = resolve_card_name(body, raw, fname)
            if name is None:
                skip["noname"].append(f"{path.name}::{fname}")
                continue
            if " // " in name:
                skip["split"].append(f"{path.name}::{fname}")
                continue
            card = CACHE.get(name) or CACHE_LC.get(name.lower())
            if not isinstance(card, dict):
                skip["nocache"].append(f"{path.name}::{fname}")
                continue
            # ⚠ A MULTI-FACE CARD WAS SKIPPED WHOLE — 276 factories with no
            # column at all, which is the same shape as the `nocache` skip that
            # hid three wrong costs. The oracle carries name / mana_cost /
            # type_line / oracle_text / power / toughness PER FACE, so the
            # three columns that read those can run once the right face is
            # picked: **by NAME, not by position.** Scryfall keys a DFC by
            # "Plargg, Dean of Chaos // Augusta, Dean of Order" AND by each
            # face name, so a lookup on the back face's name returns a card
            # whose `card_faces[0]` is the OTHER side — face 0 reported Augusta
            # at `{1}{R}`, which is Plargg's cost.
            # The whole-card fields (`colors`, `keywords`, `loyalty`) are not
            # per-face, so the columns that read them skip themselves: the
            # merged view drops those keys and carries `_face`, which the two
            # checks that would otherwise misread an absent key consult.
            if card.get("card_faces"):
                face = next((f for f in card["card_faces"]
                             if f.get("name") == name), None)
                if face is None:
                    skip["faces"].append(f"{path.name}::{fname}")
                    continue
                card = {k: v for k, v in card.items()
                        if k not in ("colors", "keywords", "loyalty", "cmc",
                                     "mana_cost", "type_line", "oracle_text",
                                     "power", "toughness")}
                card.update({k: v for k, v in face.items() if k != "name"})
                card["_face"] = True
            # Tokens, Vanguards, schemes and the rest of the non-deck types all
            # print no mana cost and none of them is ever cast.
            tl = card.get("type_line") or ""
            if tl == "Card" or any(w in tl for w in NOT_A_SPELL):
                skip["notaspell"].append(f"{path.name}::{fname}")
                continue
            # CR 202.1b / 601.3e — a card with NO printed mana cost cannot be
            # cast by paying one, and its mana value is 0. The engine enforces
            # that off `no_mana_cost`, so a card missing the flag is either
            # castable for an invented price (a `cost:` that is not printed) or
            # castable for FREE (no `cost:` at all). Lands are excluded: they
            # print no mana cost by definition and the cast path stops them
            # with `CannotCastLand`. This check runs on EVERY factory, literal
            # or helper-built, because the flag is a plain block-level field.
            # ⚠ NOT ON A FACE. A transform card's back prints no mana cost by
            # definition and is not cast by paying one, so both directions of
            # this check are meaningless there.
            if (not card.get("_face") and card.get("mana_cost", "") == ""
                    and "no_mana_cost: true" not in raw
                    and "Land" not in (card.get("type_line") or "")):
                missing_flag += 1
                shipped = COST.search(body) if body is not None else None
                rows.append(("no-cost", name, f"{path.name}::{fname}",
                             (body_cost(shipped.group(1)) or "?") if shipped
                             else "{} and castable for free",
                             "no printed mana cost"))
            # ⚠ AND THE OTHER DIRECTION IS WORSE. The flag on a card that DOES
            # print a cost makes it uncastable for ever — `CannotCastNoCost`,
            # a dead card in every deck it is in and a bot seat that holds it
            # to the end. Same crude text read as above, which is sound in this
            # direction: a nested `TokenDefinition` carries no such field.
            elif (not card.get("_face") and card.get("mana_cost", "") != ""
                    and "no_mana_cost: true" in raw):
                stray_flag += 1
                rows.append(("no-cost", name, f"{path.name}::{fname}",
                             "`no_mana_cost: true` — uncastable",
                             card["mana_cost"]))
            # A helper-built factory passes the cost positionally:
            # `sorcery("Name", cost(&[generic(2), r()]), effect)`. Read that
            # form too, or 6,312 factories — most of the older sets — are
            # audited by nobody.
            cm = COST.search(body)
            cost_src = cm.group(1) if cm else None
            if cost_src is None:
                # No `cost:` of its own: follow the call it defers to, and take
                # the cost the HELPER prints rather than whatever sits next to
                # the name (`base_struct_cost` is the old heuristic and priced
                # fourteen cards at their ability's cost).
                after_sig = factory_call_src(raw)
                # ⚠ The PSEUDO literal is `..helper(` and nothing else, so its
                # `..` match has no argument list behind it — reading the args
                # out of it returned an empty list and every pure-helper card
                # resolved to `None`. The real call is in `after_sig` there.
                m1 = None if pseudo else re.search(r"\.\.([a-z0-9_]+)\s*\(", body)
                cm2 = m1 or re.match(r"\s*(?:[A-Za-z_]+::)*([a-z0-9_]+)\s*\(", after_sig)
                if cm2:
                    src_text = body if m1 else after_sig
                    at = src_text.index(cm2.group(0)) + cm2.group(0).rindex("(")
                    cost_src = resolve_helper_cost(
                        index, path, cm2.group(1), call_args(src_text, at)
                    )
                    if cost_src is not None:
                        inner = re.match(r"(?:crate::mana::)?cost\(&\[(.*)\]\)$",
                                         cost_src.strip(), re.S)
                        if inner:
                            cost_src = inner.group(1)
            # ⚠ ONE COLUMN'S SKIP IS NOT THE OTHERS'. This used to `continue` on
            # a cost it could not read, so a card behind a cost chain the
            # resolver gives up on was dropped from the TYPE and SUBTYPE columns
            # too — three columns' coverage decided by the hardest one. Each
            # column takes its own verdict now and the loop runs on.
            got = body_cost(cost_src) if cost_src is not None else None
            if got is None:
                skip["nonliteral"].append(f"{path.name}::{fname}")
            else:
                checked += 1
                want = card.get("mana_cost", "")
                # A card that prints NO mana cost has nothing to compare
                # against, and the `no_mana_cost` check above is the one that
                # owns it. This also keeps the helper form honest:
                # `blighted("Blighted Steppe", cost(&[generic(3), w()]), ..)`
                # passes the ACTIVATED ability's cost, and the card is a land,
                # so the only sound reading of it is "this card prints no mana
                # cost".
                if want != "" and norm(got) != norm(want):
                    wrong_cost += 1
                    rows.append(("cost", name, f"{path.name}::{fname}",
                                 got or "{}", want or "{}"))
            # The printed TYPE LINE. A Sorcery shipped as an Instant is castable
            # at instant speed; a missing Legendary is a legend rule that never
            # fires; and a missing SUBTYPE is a tribal card that does not see
            # its own creature, an Equipment nothing can equip, an Aura with
            # nothing to attach to, a land that taps for the wrong colour.
            line = (card.get("type_line") or "").split("—")[0]
            want_t = {w for w in line.replace("//", " ").split() if w in CARD_TYPES}
            want_s = {w for w in line.split() if w in SUPERTYPES}
            # ⚠ THE TYPE LINE IS RARELY IN THE LITERAL. Three idioms carry it
            # somewhere else, and a reader that only looks at the literal
            # reports its own blind spot: the BASE-STRUCT form
            # (`..creature("Name", cost, types, 1, 1)`, `..god_weapon(..)`),
            # the WRAPPER form (`legend(CardDefinition { .. })`, six Invasion
            # legends), and a helper whose own base is another helper. All
            # three are followed now; when a link cannot be found the card is
            # skipped rather than reported, because 77 rows of "supertypes:
            # (none)" the first time this column was opened were every one of
            # them `..legend(..)` supplying what the literal never claimed.
            # A pure-helper factory's pseudo-literal carries no argument list,
            # so the base call is read out of the factory text instead — the
            # same `after_sig` the cost reader falls back to.
            read = resolve_type_line(
                index, path, body, raw, inner=factory_raw(raw),
                call_src=factory_call_src(raw) if pseudo else None,
                subs_index=subs_index)
            got_t, got_s, got_sub = read.types, read.supers, read.subs
            resolved, sub_ok = read.ok, read.sub_ok
            if not resolved:
                skip["notyped"].append(f"{path.name}::{fname}")
                continue
            checked_types += 1
            # A Vanguard avatar named after a card is not that card, and the
            # oracle lookup cannot tell them apart (Maraxus of Keld is both).
            if "avatar" in (raw[raw.find("\n"):] or "") and "Vanguard" not in (card.get("type_line") or ""):
                if re.search(r"\.\.avatar\s*\(", raw):
                    skip["notaspell"].append(f"{path.name}::{fname}")
                    continue
            if got_t != want_t and want_t:
                wrong_types += 1
                rows.append(("types", name, f"{path.name}::{fname}",
                             " ".join(sorted(got_t)) or "(none)",
                             " ".join(sorted(want_t))))
            if got_s != want_s:
                wrong_types += 1
                rows.append(("super", name, f"{path.name}::{fname}",
                             " ".join(sorted(got_s)) or "(none)",
                             " ".join(sorted(want_s)) or "(none)"))
            # The SUBTYPE half. A multi-face card's oracle line carries a second
            # `—` and the engine ships one face per factory, so it is not this
            # column's to compare; and a subtype word no enum has a variant for
            # is a MISSING VARIANT, which is coverage rather than a defect — it
            # would otherwise read as "the card is missing this subtype" on a
            # catalog that cannot spell it.
            tl = card.get("type_line") or ""
            # ⚠ A CARD THAT *BECOMES* A CREATURE CARRIES THE TYPES IT BECOMES.
            # `StaticEffect::SelfIsCreatureIf { creature_types: vec![] }` and
            # `creature_off_battlefield: true` both read the definition's own
            # `creature_types`, so Gideon Blackblade ships Human Soldier and
            # Grist ships Insect while printing neither. That is the engine's
            # encoding of the creature, not a wrong type line, and both flags
            # are in the literal where this can see them.
            # ⚠ `layout == "normal"` USED TO GATE THIS AND IT COUNTED NOWHERE.
            # The face merge above already replaces `type_line` with the FACE's
            # line, so a transform / mdfc card reaches here with a perfectly
            # ordinary one-`—` line and was dropped anyway — along with every
            # saga, adventure, prototype, leveler, mutate, class, case, augment
            # and flip card, 1,200-odd of them, into no skip bucket at all. The
            # requirement was never the layout WORD: it is that the line
            # describes ONE face, which `//` and the `—` count already say.
            if BECOMES_CREATURE.search(raw):
                skip["nosubtypes"].append(f"{path.name}::{fname}")
            elif sub_ok and "//" not in tl and tl.count("—") <= 1:
                words = tl.split("—", 1)[1].split() if "—" in tl else []
                if any(w.lower() not in SUB_WORD for w in words):
                    skip["nosubvariant"].append(f"{path.name}::{fname}")
                else:
                    want_sub = {SUB_WORD[w.lower()] for w in words}
                    checked_sub += 1
                    if got_sub != want_sub:
                        wrong_types += 1
                        rows.append(("sub", name, f"{path.name}::{fname}",
                                     " ".join(sorted(got_sub)) or "(none)",
                                     " ".join(sorted(want_sub)) or "(none)"))
            elif not sub_ok:
                skip["nosubtypes"].append(f"{path.name}::{fname}")
            else:
                # Readable here, but the oracle line is not ONE face. Its own
                # bucket, because "readable and not compared" is the shape that
                # counted nowhere for as long as the layout gate existed.
                skip["subtwoface"].append(f"{path.name}::{fname}")
            # The PRINTED KEYWORDS, restricted to the evergreen set both sides
            # spell as a plain keyword. A missing Flying is not cosmetic: the
            # bot's block search, the damage assignment and every evasion check
            # read this field on every combat of every self-play game.
            # ⚠ NOT ON A FACE EITHER: the oracle's `keywords` array is a
            # whole-card field, so an absent one would read as "the card prints
            # none" and report every keyword the engine has as EXTRA.
            if not read.kw_ok or card.get("_face"):
                skip["nokeywords"].append(f"{path.name}::{fname}")
            else:
                want_kw = {EVERGREEN[k] for k in (card.get("keywords") or [])
                           if k in EVERGREEN}
                have_kw = read.keywords & EVERGREEN_VARIANTS
                # ⚠ A KEYWORD THE CARD GRANTS ITSELF IS NOT A MISSING ONE.
                # "As long as you control a Swamp, this has fear" is a
                # `StaticAbility`, not a `keywords:` entry, and the oracle
                # counts it in the card's keywords all the same. So a keyword
                # the factory mentions ANYWHERE is not reported missing; the
                # other direction (the engine has one the card does not print)
                # needs no such guard.
                printed = printed_keyword_lines(card.get("oracle_text"))
                missing = {k for k in (want_kw & printed) - have_kw
                           if f"Keyword::{k}" not in raw
                           and (name, k) not in REVIEWED_KEYWORDS}
                extra = {k for k in have_kw - want_kw
                         if (name, k) not in REVIEWED_KEYWORDS}
                checked_kw += 1
                if missing or extra:
                    wrong_kw += 1
                    rows.append(("kw", name, f"{path.name}::{fname}",
                                 " ".join(sorted(have_kw)) or "(none)",
                                 " ".join(sorted(want_kw)) or "(none)"))
            # The PRINTED COLOR — derived from the cost's pips, the colour
            # indicator and `Devoid`, so it is only readable where the cost is
            # and where the keyword chain resolved. 186 cards in the cache
            # print a colour their mana cost cannot carry.
            oc = card.get("colors")
            if oc is None or got is None or not read.kw_ok or not read.col_ok:
                skip["nocolors"].append(f"{path.name}::{fname}")
            else:
                ov, ind = read.colorfield
                have_c = (set() if "Devoid" in read.keywords
                          else ov if ov is not None else cost_colors(got) | ind)
                checked_col += 1
                if have_c != set(oc) and name not in REVIEWED_COLORS:
                    wrong_col += 1
                    rows.append(("color", name, f"{path.name}::{fname}",
                                 "".join(sorted(have_c)) or "colorless",
                                 "".join(sorted(oc)) or "colorless"))
            # CR 306.5b — the STARTING LOYALTY. A planeswalker enters with
            # that many loyalty counters (`CardInstance::new` reads
            # `base_loyalty`), so a wrong number is wrong in every game the
            # card is in — and 0 means it dies to the state-based action the
            # turn it lands (CR 704.5i).
            ol = card.get("loyalty")
            if ("Planeswalker" in (card.get("type_line") or "")
                    and ol is not None and ol.isdigit()):
                if not read.loy_ok:
                    skip["noloyalty"].append(f"{path.name}::{fname}")
                else:
                    checked_loy += 1
                    # No `base_loyalty` anywhere in the chain is a VALUE, not a
                    # gap — the engine ships `Default` (0) there, and CR 704.5i
                    # puts a 0-loyalty walker in the graveyard on entry.
                    if (read.loyalty or 0) != int(ol):
                        wrong_loy += 1
                        rows.append(("loyalty", name, f"{path.name}::{fname}",
                                     str(read.loyalty or 0), ol))
            # The printed P/T, through the same chain as everything else — see
            # `parse_pt` for what it used to read instead.
            op, ot = card.get("power"), card.get("toughness")
            if op is None or ot is None:
                continue
            if not (op.lstrip("-").isdigit() and ot.lstrip("-").isdigit()):
                skip["star"].append(f"{path.name}::{fname}")
                continue
            band = station_pt(raw)
            if band is not None:
                checked_pt += 1
                if band != (int(op), int(ot)):
                    wrong_pt += 1
                    rows.append(("p/t", name, f"{path.name}::{fname}",
                                 f"{band[0]}/{band[1]} (top station band)",
                                 f"{op}/{ot}"))
            elif not read.pt_ok:
                skip["nopt"].append(f"{path.name}::{fname}")
            else:
                checked_pt += 1
                # No `power:` anywhere in the chain is a VALUE, not a gap: the
                # engine ships `Default` there, and a creature at 0/0 dies to
                # state-based actions the turn it lands. `Default` is the LAST
                # link of `merge_pt`'s chain, so it fills a half the literal and
                # every base left alone — which is what a 0-power card is.
                got_pt = merge_pt(read.pt, (0, 0))
                if got_pt != (int(op), int(ot)):
                    wrong_pt += 1
                    rows.append(("p/t", name, f"{path.name}::{fname}",
                                 f"{got_pt[0]}/{got_pt[1]}", f"{op}/{ot}"))

    shown = rows if args.rows == 0 else rows[: args.rows]
    for kind, name, where, got, want in shown:
        print(f"  {kind}  {name}\n      {where}\n      body {got}   oracle {want}")
    if len(rows) > len(shown):
        print(f"  ... {len(rows) - len(shown)} more (--rows 0)")
    print(f"# compared against the oracle: {checked} priced, {checked_types} on the "
          f"type line, {checked_sub} on subtypes, {checked_kw} on keywords, "
          f"{checked_pt} on P/T, {checked_col} on colors, "
          f"{checked_loy} on loyalty — "
          f"**{wrong_cost} wrong cost, {wrong_pt} wrong P/T, "
          f"{missing_flag} missing / {stray_flag} stray `no_mana_cost`, "
          f"{wrong_types} wrong type line, "
          f"{wrong_kw} wrong keywords, {wrong_col} wrong colors, "
          f"{wrong_loy} wrong loyalty**")
    print("# skipped — " + ", ".join(f"{k} {len(v)}" for k, v in sorted(skip.items())))
    if args.list_kind:
        for where in skip.get(args.list_kind, []):
            print(where)
    return 0


if __name__ == "__main__":
    sys.exit(main())
