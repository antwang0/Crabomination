# Strixhaven implementation tracker

Two adjacent catalogs: **Secrets of Strixhaven (SOS)** (`catalog::sets::sos`)
and **Strixhaven: School of Mages (STX)** (`catalog::sets::stx`).

All ✅-done cards have been elided — the tables below list only the
remaining **🟡 partial** and **⏳ todo** work. For full per-card history see
`git log -- crabomination/src/catalog/sets/{stx,sos}/`.

## Legend

- 🟡 partial — body wired with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Known engine gaps surfaced by these catalogs

- **Lessons sideboard / Learn** — Eyetwitch, Pest Summoning, Hunt for
  Specimens, Field Trip, Igneous Inspiration. Approximated as `Draw 1`.
- **Cast-from-graveyard / cast-from-exile pipelines** — block several
  Paradigm cards and the lone SOS ⏳ (Improvisation Capstone).
- **Multi-target prompts on instants/sorceries** — recurring 🟡 reason
  across SOS/STX (Vibrant Outburst, Snow Day, Devious Cover-Up, Crackle
  with Power, Magma Opus). Divided-damage / per-mode multi-target slots
  remain a gap distinct from the bag-of-targets primitives.

## Blue

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Brush Off | {2}{U}{U} | Instant |  | This spell costs {1}{U} less to cast if it targets an instant or sorcery spell. / Counter target spell. | 🟡 | Wired in `catalog::sets::sos::instants` as a 4-mana hard counter. The conditional cost-reduction-when-targeting-IS rider is omitted (no target-aware cost reduction primitive). |
| Divergent Equation | {X}{X}{U} | Instant |  | Return up to X target instant and/or sorcery cards from your graveyard to your hand. / Exile Divergent Equation. | 🟡 | Wired in `catalog::sets::sos::instants` as a single-target return. The "up to X" multi-target prompt is collapsed to one target (no `Selector::OneOf` / count-bounded pick primitive yet — TODO.md). The "exile this" rider is omitted (no replay-prevention payoff). |
| Encouraging Aviator // Jump | {2}{U} // {U} | Creature — Bird Wizard // Instant | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Flow State | {1}{U} | Sorcery |  | Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order. If there is an instant card and a sorcery card in your graveyard, instead put two of… | 🟡 | Approximated as `Scry 3 + Draw 1`. Conditional "instead pick 2 to hand" gy-IS-pair upgrade rider is omitted (no "look-and-distribute-by-count" primitive). |

## Black

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Forum Necroscribe | {5}{B} | Creature — Troll Warlock | 5/4 | Ward—Discard a card. / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, return target creature card from your graveyard to the battlefield. | 🟡 | Wired in `catalog::sets::sos::creatures` (5/4 Troll Warlock body + Repartee gy-creature-recursion via the `repartee()` shortcut chained with `Effect::Move(target Creature → Battlefield(You))`). Ward—Discard a card omitted (no Ward keyword primitive yet — tracked in TODO.md). |

## Red

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Goblin Glasswright // Craft with Pride | {1}{R} // {R} | Creature — Goblin Sorcerer // Sorcery | 2/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Impractical Joke | {R} | Sorcery |  | Damage can't be prevented this turn. Impractical Joke deals 3 damage to up to one target creature or planeswalker. | 🟡 | 3-to-creature/PW wired; "damage can't be prevented" rider is a no-op (engine has no damage-prevention layer). |
| Maelstrom Artisan // Rocket Volley | {1}{R}{R} // {1}{R} | Creature — Minotaur Sorcerer // Sorcery | 3/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Pigment Wrangler // Striking Palette | {4}{R} // {R} | Creature — Orc Sorcerer // Sorcery | 4/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Rubble Rouser | {2}{R} | Creature — Dwarf Sorcerer | 1/4 | When this creature enters, you may discard a card. If you do, draw a card. / {T}, Exile a card from your graveyard: Add {R}. When you do, this creature deals 1 damage to each opponent. | 🟡 | Push XV: ETB rummage now wrapped in `Effect::MayDo` so the "you may discard" optionality is honored. The `{T}, Exile a card from your graveyard:` activated ability is still omitted (engine activated-ability path has no `from-your-graveyard` cost variant — separate from `sac_cost`). |
| Steal the Show | {2}{R} | Sorcery |  | Choose one or both — / • Target player discards any number of cards, then draws that many cards. / • Steal the Show deals damage equal to the number of instant and sorcery cards in your graveyard to target creature or planeswalker. | 🟡 | Modal sorcery: mode 0 (target player discards N then draws N — collapsed to "discard 2, draw 2" since the engine has no "any number" prompt for the targeted player); mode 1 deals damage = `Value::CountOf(CardsInZone(your graveyard, IS-cards))` to a creature/PW. The "choose one or both" rider collapses to "pick one mode" (no multi-mode-pick primitive yet). |
| Tablet of Discovery | {2}{R} | Artifact |  | When this artifact enters, mill a card. You may play that card this turn. (To mill a card, put the top card of your library into your graveyard.) / {T}: Add {R}. / {T}: Add {R}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::artifacts` — ETB Mill 1 + two `{T}: Add {R}` mana abilities. The "may play that card this turn" mill-rider is omitted (no per-card may-play primitive yet). The spend-restriction on the {T}: Add {R}{R} ability is omitted (no spend-restricted mana primitive). |

## Prismari (Blue-Red)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abstract Paintmage | {U}{U/R}{R} | Creature — Djinn Sorcerer | 2/2 | At the beginning of your first main phase, add {U}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::creatures` with a `StepBegins(PreCombatMain)/ActivePlayer` trigger that adds {U}{R} via `ManaPayload::Colors`. The spend restriction is omitted (no per-pip mana metadata). The hybrid `{U/R}` pip is approximated as `{U}`. |
| Prismari, the Inspiration | {5}{U}{R} | Legendary Creature — Elder Dragon | 7/7 | Flying / Ward—Pay 5 life. / Instant and sorcery spells you cast have storm. | 🟡 | Flying + Ward(5) approximated as generic mana. Storm grant omitted. |

## Silverquill (White-Black)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abigale, Poet Laureate // Heroic Stanza | {1}{W}{B} // {1}{W/B} | Legendary Creature — Bird Bard // Sorcery | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Conciliator's Duelist | {W}{W}{B}{B} | Creature — Kor Warlock | 4/3 | When this creature enters, draw a card. Each player loses 1 life. / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, exile up to one target creature. Return that card to the battlefield under its owner's control at the beginning of the next end step. | 🟡 | ETB body wired (draw 1 + each player loses 1). Repartee exile half wired via the new `Selector::CastSpellTarget(0)` primitive. The "return at next end step" rider is still omitted (no capture-as-target-from-selector primitive yet). |
| Fix What's Broken | {2}{W}{B} | Sorcery |  | As an additional cost to cast this spell, pay X life. / Return each artifact and creature card with mana value X from your graveyard to the battlefield. | 🟡 | Wired in `catalog::sets::sos::sorceries` (push XVII): Lose 2 life + return artifact/creature cards with MV ≤ 2 from gy → bf. X collapsed to 2 (no X-life-as-cost primitive). |

## Lorehold (Red-White)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Lorehold, the Historian | {3}{R}{W} | Legendary Creature — Elder Dragon | 5/5 | Flying, haste / Each instant and sorcery card in your hand has miracle {2}. (You may cast a card for its miracle cost when you draw it if it's the first card you drew this turn.) / At the beginning of each opponent's upkeep, you may discard a card. If you do, draw a card. | 🟡 | Body-only wire (5/5 Flying+Haste Legendary Elder Dragon, R/W). Miracle grant on instants/sorceries in hand is omitted (no miracle/alt-cost-on-draw primitive); per-opp-upkeep loot trigger omitted (no opp-upkeep step trigger that fires for non-active player). The vanilla finisher is the most impactful printed clause — both omitted clauses are tracked in TODO.md. |
| Molten Note | {X}{R}{W} | Sorcery |  | Molten Note deals damage to target creature equal to the amount of mana spent to cast this spell. Untap all creatures you control. / Flashback {6}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Wired in `catalog::sets::sos::sorceries` (push XVII): X damage to creature + untap all your creatures + Flashback {6}{R}{W}. Damage = X from cost (mana-spent approximation). |
| Practiced Scrollsmith | {R}{R/W}{W} | Creature — Dwarf Cleric | 3/2 | First strike / When this creature enters, exile target noncreature, nonland card from your graveyard. Until the end of your next turn, you may cast that card. | 🟡 | Wired in `catalog::sets::sos::creatures`. ETB now exiles **exactly one** matching noncreature/nonland card in the controller's graveyard via the new `Selector::Take(_, 1)` primitive (push X) — closer to the printed "target one card" semantics; the prior implementation exiled every matching card. The hybrid `{R/W}` pip is approximated as `{R}` (cost: `{R}{R}{W}`). The "may cast until next turn" rider is omitted (no cast-from-exile-with-time-limit primitive). |
| Pursue the Past | {R}{W} | Sorcery |  | You gain 2 life. You may discard a card. If you do, draw two cards. / Flashback {2}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Push XV: gain 2 + the discard+draw chain wrapped in `Effect::MayDo` so the printed "you may discard" optionality is honored. Flashback wired via `Keyword::Flashback`. The lifegain half always resolves; the loot half is opt-in. |

### Engine pieces driven by STX

- ⏳ **Lesson sideboard model** — Eyetwitch, Hunt for Specimens, Pest
  Summoning all use Learn at some point. Currently approximated as
  `Draw 1`.
