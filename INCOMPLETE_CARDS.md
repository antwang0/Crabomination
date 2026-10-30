# Incomplete cards — implemented but missing a printed capability

Cards that resolve and play, but whose implementation **drops or approximates a
real-Magic capability** (the canonical example: a card that should be castable
from the graveyard but isn't). Distinct from *blank* cards (gated by
`core_rules::structural_audit::no_shipped_card_is_a_blank_stub`, formerly the
`audit_stubs` binary) — these look done.

## Reading key — most of the tables here are history, not a worklist

An audit pass lands as a narrative plus a `| Card | Was | Printed |` table of
what it **fixed**, so a card named in one of those rows is fixed, not open:
the "Was" column is the old code. Only these are open work — 🟡 rows, a
"Residue" list under a fixed table, and the numbered `### N. No … primitive`
sections. Checked 2026-09-20 by matching the ten pod decks' 410 card names
against this file: 48 mentions, every substantive one a fixed-table row.

## How this file was produced / how to regenerate

Run the companion auditor:

```
cargo run -p crabomination --bin audit_incomplete                  # both passes
cargo run -p crabomination --bin audit_incomplete -- --structural-only
cargo run -p crabomination --bin audit_incomplete -- --comments-only
python3 scripts/audit_variant_coverage.py     # the third filter, no build
```

It has two independent passes:

1. **Structural** (comment-free, authoritative): serde-walks every card's
   effect tree and flags *dead modes* (a `ChooseMode`/`ChooseN`/`Escalate` arm
   that resolves to `Noop`/empty) and *dead abilities* (a triggered / activated
   / loyalty ability with an empty effect). An empty arm is a bug regardless of
   what the card "should" do. **Re-run 2026-09-03 (unchanged since 2026-08-23): 21,795 unique cards, one
   finding — Elite Interceptor, already triaged below as not a gap.**
   `audit_stubs` on the same tip: 21,795 scanned, **0 flagged**.
2. **Comment scan**: lists every `pub fn … -> CardDefinition` factory whose doc
   comment carries an approximation marker (`approximation`, `modeled as`,
   `omitted`, `stub`, `body only`, `collapsed`, …). As of 2026-08-23: **910 of
   22,568 factories** across 707 catalog files carry such a note (was 470 of
   8,092 — the catalog has roughly tripled since, and the *rate* is flat at
   ~4 %).

3b. **Oracle cross-reference** (`scripts/audit_bottom_random.py`,
   `scripts/audit_target_walkers.py`; no build): the passes above read the
   *code*, so neither can see a card whose tree is well-formed and says
   something the printed card does not. These read a field out of the source
   and compare it against the committed scryfall cache. **Both were written
   after a tracker row turned out to be a claim about code that nobody had
   re-read** — the pattern is worth repeating for any field whose default is
   the wrong answer.

3. **Variant coverage** (`scripts/audit_variant_coverage.py`, comment-free,
   no build): the two passes above look *inside* one card's effect tree, so
   neither can see a card whose tree is fine and whose *engine* arm is a
   no-op. This one cross-references each capability enum against the catalog
   and the engine both ways. **2026-08-31: 0 dead
   capabilities over 1,695 variants; 2 dead primitives.** 2026-09-13: 0 dead
   capabilities over 1,697; **1** dead primitive. See "The other direction" below.

The tables below are a human triage of those 470 + the structural findings,
grouped by the **missing engine primitive** so each cluster is one work-item.

## ⚠️ Caveat: doc comments lie (~30% stale on the HIGH tier)

Spot-checking the worst findings turned up comments that say "omitted" over code
that *does* wire the capability (and the reverse). The structural pass exists
precisely because it can't be fooled this way. Cross-reference the comment scan
against pass 1 to hunt stale comments. Confirmed stale so far:

| Comment claims | Reality |
|---|---|
| **Arclight Phoenix** — "graveyard recursion omitted" | Fully wired (`FromYourGraveyard` begin-combat trigger gated on 3+ I/S). Stale first comment block. |
| **Silversmote Ghoul** — "returns ALL graveyard creatures" | Code uses `Selector::This` → returns only itself, which is *correct*. |
| **Griselbrand** — "Stub: vanilla 7/7" | Draw-7/pay-7-life is actually wired. |

Items below marked **✓** were code-verified; unmarked HIGH items are
doc-derived — confirm before acting.

---

## Structural dead capabilities (authoritative — from pass 1)

**Dead abilities are now a suite gate.** `crabomination_tests`
`core_rules::structural_audit::no_shipped_card_has_a_dead_ability` asserts
over the whole catalog that no card ships a triggered / activated / loyalty
ability whose effect resolves to nothing, so that half of pass 1 can't
regress silently and doesn't need a triage table here any more. The walker
lives in `crabomination::audit` — one copy, shared by both audit binaries
and the test.

Dead **modes** stay out of the gate and still want human triage: a `Noop`
arm is also the idiom for a deliberate "you may … (or decline)" option. Run
`cargo run -p crabomination --bin audit_incomplete -- --structural-only`
for the current list.

| Card | Location | Verdict |
|---|---|---|
| Sublime Epiphany ✓ | stx/extras_02.rs:668 | **Fixed.** All five modes now real: mode 1 → `Effect::CounterAbility`, mode 3 → `Effect::CreateTokenCopyOf`. |
| Elite Interceptor ✓ | sos/mdfcs.rs:181 | **Not a gap.** Arm #2 (`Noop`) is the deliberate "decline" half of "you may tap or untap"; the draw is unconditional. |

Closed as of the 2026-08-09 re-mine (all three were real, all three fixed at
the helper rather than per card — see the commit):

| Card | Was | Fix |
|---|---|---|
| Magosi, the Waterveil | empty ETB trigger | `tapped_etb_land` no longer emits `etb(Noop)` |
| Oran-Rief, the Vastwood | empty ETB trigger | same helper |
| Annie Joins Up | empty "legendary creature enters" trigger | `joins_up`'s `ongoing` is now `Option<TriggeredAbility>`; Annie's second ability is static |
| Circling Vultures | flagged, **not a bug** | the auditor's cost-only carve-out now covers the five self-moving costs, not just `sac_cost` |

> Note: an earlier manual pass mislabeled the Sublime Epiphany finding as a
> card called "Persist" — the structural auditor is the source of truth.

### The other direction — a shipped card whose ability the *engine* drops

Both audit binaries look **inside** one card's effect tree, so neither can see
the failure where the tree is well-formed and the engine has nothing to do
with it: an exhaustive `match` is satisfied by an `A | B | C => {}` arm, so a
variant can be on shipped cards, type-check everywhere, resolve without a
panic, and do nothing. `scripts/audit_variant_coverage.py` asks that question
by cross-referencing each capability enum against the catalog and the engine,
with the no-op arms **discovered** rather than hardcoded (the layer pass's
"these statics are not continuous effects" arm alone is 780 lines).

**Reading at 2026-08-27 — zero.** Over `StaticEffect` (471 variants), `Effect`
(987) and `Keyword` (237): **no variant that shipped cards use lacks an engine
arm outside a no-op**, including all 441 statics the layer pass explicitly
declines to turn into continuous effects. The filter is cheap (~40 s, no
build) and it gates on this half only.

**One dead primitive falls out of the other direction** — an implemented
effect nothing constructs, i.e. capability waiting for the card that wanted
it, at no engine cost. (`ExileTopAndMayCastUpToMv` was one of three and is
constructed now; `AddRadCounters` was the second and got its cards
2026-09-13.)

| Primitive | Resolver | The card shape it is for |
|---|---|---|
| `Effect::GrantCastBackFromGraveyard { what }` | `effects/mod.rs` | "you may cast it from your graveyard" — ⚠ **no printed card prints this**; the row below says why the lane stays anyway |
| `Effect::LockSpellsAfterGrantedCast { what }` | `effects/mod.rs` | the granted cast's "then you can't cast spells this turn" rider |
| `Effect::DestroyWithinTotalManaValue { filter, cap }` | `effects/mod.rs` | "destroy any number of target … with total mana value N or less" |
| `Effect::EnlistThen { then }` | `effects/mod.rs` | "if it enlisted, …" |
| `Effect::RevealOpponentTopPutOntoBattlefield { count, filter }` | `effects/mod.rs` | "reveal the top N of target opponent's library; put a [filter] onto the battlefield under your control" |

**Reading at 2026-10-08 (ninth Commander run):** structural pass 0 over
25,679 cards (three reviewed empties: Braids, Elite Interceptor, Hullbreaker
Horror); `audit_variant_coverage.py`: 0 dead capabilities, **5** dead
primitives (the table). None is a shipped card's gap; each waits for a card
or a deletion.

**Closed 2026-09-13:** `Effect::AddRadCounters { who, amount }` — Nuclear
Fallout (`decks::recent329`) plus `sets::pip`'s Contaminated Drink, Glowing One
and Feral Ghoul. The CR 728.2 turn-based action had been implemented and
`core_rules`-tested the whole time with nothing to drive it. Both sessions
working this branch picked Nuclear Fallout off the same audit row the same day;
the duplicate was dropped at the rebase.

**Check the encoding caution in TODO before adding a card for one**: whether a
new catalog entry moves `Vocab` decides whether it invalidates the trained
nets, and that question is not answered here.

### The same direction again, found 2026-09-17 — a *leaf* the engine drops

`audit_variant_coverage.py` asks whether a `Keyword` variant has an engine
arm. It cannot ask whether the arm's **argument** is one the arm can read:
`Keyword::CantBeBlockedBy` / `CantBeBlockedExceptBy` carry a
`SelectionRequirement`, and `blocker_matches_block_filter` (`game/mod.rs`) is
a third hand-written walker over that enum whose `_` arm answers `false`. It
is state-free on purpose — it runs inside `blocker_pair_block`'s
(blocker x attacker) loop, PERF `(-333)` — so a leaf that needs the board
silently drops. **And `false` is not conservative in both directions:** under
`CantBeBlockedExceptBy` it makes the attacker unblockable, under
`CantBeBlockedBy` it kills the restriction.

Now a suite gate: `core_rules::cr_rules::audit_block_restriction_filters_use_
leaves_the_block_walker_handles` walks every factory's serialized definition
(printed keywords and granted ones alike), pulls every block filter out of it
and asserts each leaf is one the walker handles or a named exception. ✅ **ZERO
exceptions as of 2026-09-17, out of 30 filters** — the audit's `KNOWN_DEAD`
list is empty and the test fails if a line is added back without a card to
carry it.

The one entry it held, Temple Thief's `IsEnchanted`, is fixed: CR 303.4 is a
board fact, so it is a **parameter** (`blocker_enchanted`) threaded through
`can_block_attacker_computed`, answered once per *blocker* rather than per
(blocker, attacker) pair, and behind `attachment_in_scope()` so a board with
nothing attached pays a fold read. 📐 **The useful half is the shape, not the
card**: a state-free walker that needs one board fact takes it as an argument
from the caller that already holds `&GameState` — and the same commit made
`GameState::permanent_is_enchanted` the single oracle, which the requirement
walker's two `IsEnchanted` leaves now call instead of open-coding the same
`battlefield.iter().any(..)` a third and fourth time.

---

## Missing-primitive buckets (the engineering view)

Fix the primitive → fix the whole cluster.

### 1. No multi-target / "up to N targets" / "divided as you choose" prompt
**Solved** via `Effect::ApplyToTargets` (up to N), `Effect::DealDamageDivided`
(divide as you choose), and `Effect::DealDamageDividedEvenly` + per-extra-target
cast tax (Fireball). Return to Dust ✅ (real instant, main-phase-gated 2nd
target), Rag Dealer ✅ (single-graveyard lock), Skullsnatcher ✅ (that player's
graveyard), Pull from the Grave / Rabid Attack ✅ (were stale rows), Yosei ✅
(`SlotGroups` + `SameControllerAsTargetSlot(0)`, 2026-10-23).

### 2. "Choose two of four" modal selection — ✅ closed 2026-09-28
Spells choose their modes as they are cast (`ChooseModesCast`, CR 700.2a): 23
former resolution-time `ChooseN` spells moved (Kolaghan's / Kozilek's / Austere
/ Cryptic / Lorehold Command, Sublime Epiphany, …), the Will cycle and the
Inscriptions via `CardDefinition::modes_widen`. `ChooseN` is left on modal
*triggers* and on Steal the Show (the SOS pool's two-seat bot is single-mode).

### 3. MDFC back faces — **mechanism is fully wired** (`back_face` + `GameAction::CastSpellBack`/`PlayLandBack`; 71 cards use it)
The "engine-wide ⏳" notes on these were stale. Status:
- ✅ **Pestilent Cauldron // Restorative Burst** — back attached; from-hand back-cast test. ⚠ It has **no** transform-cast-from-graveyard rider — the row below says why the claim that used to stand here was wrong on both halves.
- ✅ **Wandering Archaic // Explore the Vastlands** — back wired (`{4}` → add 6 colorless, gain 3 life) + test.
- ✅ **Selfless Glyphweaver // Deadly Vanity** — back wired via new `Effect::EachPlayerKeepsOneSacrificeRest` (each player keeps one creature/PW, sacrifices the rest) + test.
- ✅ **Birgi // Harnfel** — Harnfel back wired ("Discard a card:" → `ExileTopAndGrantMayPlay { 2 }` this turn), cast from hand as an artifact + test.
- 🟡 **transform-and-cast-from-graveyard** — the *mechanism* is wired:
  `GameAction::CastSpellBack` hops a permitted graveyard card into hand for the
  back-face cast pipeline (Muldrotha idiom), gated by a one-shot
  `CardInstance::may_cast_back_from_graveyard` flag that
  `Effect::GrantCastBackFromGraveyard` sets. **No card grants it**, and the
  claim that used to stand here — "Pestilent Cauldron's sac ability grants it;
  Restorative Burst is then castable from the graveyard" — is wrong on both
  halves: the real Cauldron's third ability is `{4}, {T}: Exile four target
  cards from a single graveyard. Draw a card.` (checked against the oracle
  cache, 2026-08-28), it has no sacrifice cost and no rider, and the shipped
  definition matches the oracle exactly. `GrantCastBackFromGraveyard` is a
  **dead primitive** in `audit_variant_coverage.py`'s table below, which is how
  this was caught — a tracker row asserting a construction the filter says does
  not exist.

### 4. No "controller-of-target" / "that player" actor (forces each-opponent / you)
**The primitive shipped long ago** (`PlayerRef::ControllerOf`, `OwnerOf`,
`OwnerOfMoved`, and a plain `TargetFiltered { filter: Player }` slot); what was
left here were rows nobody had re-read against the code.

~~Generous Gift~~ ✅ · ~~Hellrider~~ ✅ (now `Attacks/YourControl` →
`DealDamage { DefendingPlayer, 1 }`; was `SelfSource` → `EachOpponent`, doubly
wrong) · ~~Harsh Annotation~~ ✅ — **the row was stale**, it has used
`ControllerOf(Target(0))` for the token since it was written ·
~~Kemuri-Onna~~ ✅ (2026-08-29) — the ETB was `Discard { EachOpponent }` under a
comment calling that "the only sensible target"; CR 115.1 says it is a target
slot, and the bounce now goes to `OwnerOfMoved` rather than `You` ·
~~Channeled Force~~ ✅ (2026-08-29) — **not this bucket at all: the whole card
was invented.** It shipped as a Sorcery whose effect was "the chosen player
draws the difference between the two chosen players' hand sizes", with no
additional cost and no damage clause. The real card is an Instant, "as an
additional cost discard X cards; target player draws X; deal X to up to one
target creature or planeswalker" — rebuilt on `AdditionalCastCost::
DiscardXFromCost`, the shape Sickening Dreams and Firestorm already use, and
the test that locked in the invented text is replaced.

~~several CHK Ninjas~~ ✅ (2026-08-29) — all six re-read against the oracle.
Four were already right (Okiba-Gang and Walker bind the damaged seat as
`PlayerRef::Target(0)`, Skullsnatcher as `ExileUpToNFromGraveyards { of }`,
Higure and Ninja of the Deep Hours are self-scoped). **Two were not:** Throat
Slitter and Mistblade Shinobi filtered "that player controls" as
`ControlledByOpponent`, which agrees in a heads-up game and lets the Ninja
destroy or bounce an *uninvolved* third seat's creature. Both carry
`ControlledByTriggerPlayer` now, and the regression test builds the third seat.

**Still open:** Emeritus of Truce // STP — an **SOS** card, so its definition
*is* its spec (no oracle to arbitrate) and TODO's encoding caution names that
pool; leave it to a run that intends a pool change. **And the
lesson the two closed rows share is a filter for the rest of this file:** a row
here is a claim about code, and three of the five in this bucket were wrong
about it in both directions — one card was already fixed, one was broken
differently than the row said. Re-read the definition against the oracle cache
before working a row; it costs a minute and it changed the answer twice.

### 5. No "first/Nth spell this turn" / "no card drawn this turn" gate (over-triggers)
**Stale** — `Predicate::SpellsCastThisTurn{Equals,AtLeast}` + `EventSpec::
once_per_turn` ship. Frostpyre Arcanist — **this row was about a card that does
not exist**: the shipped body was an invented once-each-turn Magecraft return,
and the printed card is an ETB library tutor with a Giant/Wizard cost reduction
(`audit_oracle_verbs.py`, `search_library` class). Rewritten; the row is moot.
Thalia, Heretic Cathar ✅ (enters-tapped static; never had this gap). The
Quandrix/Prismari rows are fabricated `_b###` synthesized cards — moot.

### 6. No additional-cost-with-life/exile · no Phyrexian mana (riders dropped / folded into resolution)
Mostly **solved**: Deep Analysis ✅ (`PayLife`) · Resurgent Belief ✅ (flashback
gy-exile rider) · Necrotic Fumes ✅ (`ExilePermanent`) · Final Payment ✅
(`SacrificeOrPayLife`) · Mana Vault ✅ (upkeep may-pay-{4} untap + draw-step
burn) · Channel ✅ (real life-for-{C} via the payment funnel). Remaining:
Birthing Pod ({G/P} pip on an activation; Mox Diamond ✅ `EntersOnlyIfPaid`) ·
Vicious Rivalry · **Mechtitan Core** (Shorikai's EDHREC seat): "exile this
and four other artifact creatures and/or Vehicles you control" is a cost; the
Core's own exile is (`exile_self_cost`), the four are exiled as the ability
resolves behind a `condition` that four exist — so a response that removes
one still mints Mechtitan off three. Needs an exile-other-permanents cost
beside `sac_other_filter`. · ✅ **Firebender Ascension**'s quest-counter
copy ships (`Keyword::AttackTriggerQuestCopy`, `game/quest_copy.rs`; firebending
now uses the stack, CR 605.1b). An equipment-granted attack trigger the attack
commit sources off the Equipment does not count as the creature's.

### 7. Whole keyword mechanics unmodeled (each = a cluster)
- **Learn** → modeled as Draw 1 (Reduce // Rubble, Mascot Interpretation, the Lessons cycle, Quandrix Field Trip).
- **Delve** → cast at full cost (Tasigur, Magmatic Sinkhole).
- **Splice onto Arcane** → omitted (Desperate Ritual).
- **Awaken / sac-Spawn alt-cast** → omitted (Birthing Hulk, Hand of Emrakul, OGW Eldrazi).
- **Channel land cost-reductions / land-search** → dropped (decks/lands.rs cluster).
- ~~**Free-with-commander alt-cost / color-identity**~~ ✅ stale (checked 2026-10-25): `AlternativeCost` on `YouControlACommander`, `ModesWiden::If(YouControlACommander)`, `ManaPayload::AnyColorInCommanderIdentity`.
- **Wish (cast from outside the game)** → omitted (Spawnsire of Ulamog).

### 8. No copy-token / "you may choose new targets on the copy" primitive
Quandrix Snapcaster · Prismari Maestro · Echocasting Symposium · Lorehold Tomb Robber ·
Spark Double (no planeswalker-copy) · Prismari, the Inspiration · Mirror Image (legendary-strip).

### 9. No type/color rewrite on existing permanents (layers 4–5)
**Largely solved** — `Effect::BecomeCreatureType` (one-shot layer-4 set-types) +
`EquipBonus.set_creature_types`/`set_land_types`/`set_card_types`/`set_colors`
+ the CR 613.8 type-lord recompute now ship the "becomes a [color] [type]"
family (Turn to Frog, Snakeform, Polymorphist's Jest, Frogify, Darksteel
Mutation, Witness Protection, Song of the Dryads, Imprisoned in the Moon).
~~Kasmina's Transmutation~~ ✅ (now its real Aura). Remaining: Fractalize ·
Lorehold Reclamation (Spirit-typing) · Fractal-token color/type riders ·
type-gated `CardMatch` lords.

### 10. "Rest on bottom of library" approximated as "leave on top" / "to graveyard"
**Stale as written** — `LookPickToHand { rest_to_graveyard: false }` already
bottoms the rest. Augur of Bolas ✅ and Sea Gate Oracle ✅ verified.

**But the spot-check found a different defect underneath it, and it was a
class of 41** (2026-08-29). The rest reaches the bottom; it reaches it **in the
order it was revealed**. `LookPick::rest_bottom_random` is what shuffles the
batch, it is `false` by default, and 38 of the catalog's 135 `LookPick` cards
print "…on the bottom of your library in a random order" without it — Memory
Deluge, Vivien Reid, Ellywick Tumblestrum, Narset, Militia Bugler, Carth the
Lion and 32 more. Three had it and should not (Sleight of Hand and Stress Dream
bottom exactly one card, so the shuffle is a no-op that still draws game RNG;
Flow State prints "in any order", which is a choice, not chance). All 41 fixed.

**Why it matters more here than on paper:** the player has just seen those
cards, so a deterministic bottom leaves their order as known information in
the state the encoder observes, and a self-play net can learn it. The printed
rule exists to destroy exactly that information.

`scripts/audit_bottom_random.py` keeps it closed — it cross-references each
factory's `name:` against the committed scryfall cache in both directions;
129 checked, 0 findings. Geometer's Arthropod and Conjurer's Bauble were the
two spot-check rows that came back clean (Bauble's "up to one **target**
card" is a `Selector::one_of` chosen card rather than a target slot, which is
the Kemuri-Onna shape in bucket 4 and is left as the one open residual here);
Paradox Surveyor was one of the 38.

---

## Commander format staples — the provenance pair, complete

Path of Ancestry and Opal Palace both ship on CR 106.6 mana provenance
(`SpendRestriction::is_rider`, the riders `CommanderTypeScry` /
`CommanderCastCounters`), CR 106.6a's per-mana count included:
`PaymentSideEffects::spent_restrictions` carries `(restriction, pips)`, so two
rider pips on one cast are two scry triggers and two counters-per-cast, which
is what the 2020-11-10 Mana Reflection rulings on both cards say. Nothing is
rounded off; no row here.

## Pod-deck cards read against their oracle (2026-09-26) — what the fixes left

`audit_incomplete` intersected with `pod/decks.rs`, each flagged body read
against Scryfall. Fixed in the same pass (tests in `modern/dropped_riders.rs`):
Etali, Primal Storm; Puresteel Paladin; Colossus Hammer; Terror of the Peaks;
Hagra Mauling; Plaguecrafter; Angelic Destiny; Legion Warboss; Veyran;
Shalai; Howling Mine; Claim Jumper; Ophiomancer; Smuggler's Copter; Magda;
Nahiri, the Harbinger; Kari Zev; Marionette Master; Bloodthirsty Adversary;
Aura of Silence; Compulsive Research; Mask of Griselbrand; Cankerbloom;
Bitter Triumph; Ursine Monstrosity; Finale of Devastation; Teferi's
Protection; Comeuppance; Decree of Justice; Experiment Twelve; Guardian
Project; Pteramander; Mobilized District; Dragonkin Berserker; Voracious Fell Beast; Lethal Scheme;
Circuitous Route; Augur of Autumn; Imprisoned in the Moon; Serra Avenger;
Smoke; Sylvan Library ("drawn this turn" read as the hand's tail — see
`Effect::PayLifeOrPutBackDrawnThisTurn`); Master Biomancer's Mutant rider; Compulsive Research's target player;
Ajani's Chosen's Aura move (offered for an Aura on your own creature only);
Comeuppance's reflection; Conduit of Worlds' {T} cast (cast as it resolves, per
its ruling, since `01BaSouG`); Deep Analysis' target
player; Mishra's Factory's pump; Astrologian's Planisphere's third draw;
Fiery Justice's life (a targeted opponent since `SlotGroups`).

Read and left open: none. Telling Time, Zack Fair, Waste Management and
Quandrix Command's "target player" (a player slot inside the modal cast, as
Maestros Confluence's goad) since done.

## Oracle-vs-definition scans (2026-09-27, session `01J1Ud7u`) — what the fixes left

~45 cards fixed from `dump_cards --grep` / `--shape` scans (TODO's Commander
NEXT item 13 lists them); residuals the scans surfaced but did not close:

| Card | Residual | Why |
|---|---|---|
| Chevill, Bane of Monsters | "a permanent an opponent controls" dies is read as any permanent with a bounty counter | bounty counters only go on opponents' permanents, so it differs only after a control change |

## Oracle-vs-definition scans (2026-09-28, session `01BA6uAJ`) — what the fixes left

| Card | Residual | Why |
|---|---|---|
| Heirloom Mirror | the definition is an invented "STX-flavor" card (a mana rock with a sac-draw) under the real card's name | the transform-at-three-ritual-counters DFC is unbuilt (CARD_BACKLOG) |
| Nesting Dragon, Clown Extruder | the Dragon Egg / Clown Robot tokens lack Egg / Clown | neither is a `CreatureType`; adding one touches the observation vocabulary (`audit_token_types.py` allowlist) |
| Angelic Intervention | "protection from colorless" | no colorless-protection chooser (the planeswalker branch shipped) |

## Resolution-time picks handed to the player (2026-10-02) — what the fixes left

"Choose a card / a permanent" made on resolution (CR 608.2d) was a `Take` /
`TakeGreatestPower` / `TakeGreatestManaValue` engine pick on ~25 pod cards;
they now go through `shortcut::choose_one_then` (`Effect::ChooseOneAmong`,
whose headless default is the best pick for a gain and the least for a harm
on your own things). Fixed: God-Pharaoh's Gift, The Mending of Dominaria
(its "may" too), Path of the Schemer, Soul of Windgrace, Prototype Portal,
Obscura Confluence (the target player chooses), Nahiri, the Lithomancer
(+2 and −2, with their "may"), Zara, Ugin's Mastery, Armed and Armored,
Desecrator Hag (only the cards tied for greatest power — `PowerAbove` now
reads a card's printed power), Sanwell, Mission Briefing, Idris, Currency
Converter, Journey to the Lost City, Astarion's Thirst, This Is How It Ends
(the facing player picks), The War Games, Kefnet the Mindful, Scarlet Witch,
West Coast Expansion, Desecrate Reality, Wreck and Rebuild (its "may" too),
Fugitive of the Judoon (two picks in turn), Infernal Offering (each player
returns their own pick; its "choose an opponent" was already asked — the
residual note was stale), and per player inside a `ForEach`: Summon: Esper
Valigarmanda, The Weaver King, Valki. **`Effect::MaySacrifice` asks which
permanent** (CR 701.21a) — 82 cards; the source stays a last resort and a
headless seat keeps the weakest-first pick.

Multi-card picks: `Effect::ChooseSomeAmong` (`shortcut::choose_some_then`,
exactly N or up to N) — Haunting Voyage's "up to two", Insatiable
Frugivore's three each loop. Left on the same scan: Foreboding
Steamboat, Neyam Shai Murad's "that player chooses". (Dawnbreak Reclaimer's
two choosers and `Effect::ExileFromGraveyard`'s per-player pick are asked
since `01VKHeZo`.)

## Oracle-vs-definition scans (2026-09-28, session `01WRiz7c`) — what the fixes left

Scans: `audit_copy_except.py`, `audit_nontoken.py`, `audit_control_duration.py`,
`audit_return_controller.py`, `audit_other_and_side.py`, `audit_missing_static.py`
(all exploratory; the pod-scoped rows are fixed or listed here).

| Card | Residual | Why |
|---|---|---|
| Zidane, Tantalus Thief | "whenever an opponent gains control of a permanent from you, create a Treasure" is absent | no control-change event from the loser's side (only `LostControlOfThis` on the permanent) |
| Clement, the Worrywort | Frogs you control don't gain "{T}: Add {G} or {U}" (creature-spell-only) | no static grant of a restricted mana ability to a filter |
| The Necrobloom | land cards in your graveyard don't have dredge 2 | dredge is read off the card's own definition |
| Preacher | the caster picks the stolen creature (printed: an opponent's choice) | `GainControlWhileSourceTapped` resolves a caster target |
| Croaking Counterpart | the Frog copy keeps its other creature types | `CreateTokenCopyOf` adds creature types but can't replace them |
| Sunscourge Champion | eternalize's "Discard a card" additional cost isn't paid | the `eternalize` shortcut carries a mana cost only |
| Towering-Wave Mystic | mills only on combat damage, and mills the damaged player rather than a chosen target player | `DealsDamage*` binds slot 0 to the damaged object, so "target player" has no free slot |
| Enter the Infinite | "until your next turn, you have no maximum hand size" lasts the rest of the game | no timed no-max-hand-size effect |

## The pod-deck `audit_incomplete` rows (2026-09-26, session `0199ZAiL`)

`audit_incomplete`'s comment scan intersected with the 183 pod decks gave 153
rows. Read against their bodies: **37 docs were stale** (the body already did
what the doc said was missing — rewritten, among them Ponder, Chain Lightning,
Jeska's Will, Kodama's Reach, Ephemerate), **11 were substring false
positives** (Indomitable / Brontodon / Oversimplify — the audit now matches a
marker only at a word start), and **104 are real**: the audit's output is the
list, each doc names its gap. One was a body bug and is fixed (Expressive
Iteration bottomed an unseen card). Whole catalog: 841 → 795 rows.

## Commander additions of session `01Jywzky` (2026-10-21) — the approximations named

Open 🟡 rows (each card plays; the clause named is approximated):

| Card | Gap |
| --- | --- |
| 🟡 Covenant of Minds | The top three are not revealed before the opponent chooses (they go to the graveyard unrevealed — not milled — on a decline). |
| 🟡 Grand Master of Flowers | "Search your library and/or graveyard" takes a graveyard Monk first and searches the library only when there is none — no choice. |
| 🟡 Aegar, the Freezing Flame | "A spell you controlled" is read as an instant or sorcery you own (a resolved spell's controller is gone by the time the trigger checks); a spell of yours cast by an opponent, or one you stole, is misread. |

## Commander additions of session `01NkAE5B` (2026-10-20) — the approximations named

Open 🟡 rows (each card plays; the clause named is approximated):

| Card | Gap |
| --- | --- |
| 🟡 Cheering Fanatic | "Choose a card name" offers only the nonland cards in its controller's hand (the names its discount can pay off for them); an arbitrary name — an opponent's card — can't be chosen. |
| 🟡 Animate Artifact | "As long as enchanted artifact isn't a creature" reads the host's **printed** card; an artifact animated by another effect (a crewed Vehicle) still gets MV/MV (the CR 613.8 dependency is not modeled). |
| 🟡 Nephalia Academy | The "may reveal" is always taken, and no reveal event is emitted. |
| 🟡 Sphere of Annihilation | Exiles the creatures, planeswalkers and graveyard cards one at a time, then itself — not simultaneously. |
| 🟡 Turtles Forever | No prompt: the searcher takes the up-to-four highest-MV distinct-name legends and the opponent hands over the two smallest — a fixed heuristic on both sides, not a choice. |
| 🟡 The Book of Vile Darkness | Eye of Vecna and Hand of Vecna are exiled as the ability resolves (gated on both being there), not paid as part of the cost. |

## Commander additions of session `015BCEt5` (2026-09-26) — the approximations named

`decks::cmdr_top1000` and `decks::cmdr_legends` add 27 COMMANDER_BACKLOG cards
on existing primitives. What each rounds off:

| Card | Approximation | What it needs |
|---|---|---|
| Tiamat | "with different names" isn't enforced (`SearchUpToN`) — moot in a singleton deck | a distinct-names picker |
| Mythos of Illuna | the {R}{G} fight runs inside the spell, not as the token's own ETB trigger — no window to respond between entry and fight | a token copy carrying a granted triggered ability |
| Torment of Hailfire | the victim's choice is a fixed heuristic (life while above 3, then discard, then sacrifice) | a per-round punisher prompt |
| Najeela, the Blade-Blossom | an opponent's Warrior attacking asks the "you may" (its controller gets the token); the headless seat declines | nothing — a strategy choice, noted |
| Balduvian Fallen (older) | "+1/+0 per {B}/{R} spent on cumulative upkeep" dropped | a `CumulativeUpkeepPaid` event carrying the spent colours |

## "Returns … with a counter on it" placed after entry (CR 122.6 / 614.1c)

Fixed 2026-10-23: 27 cards that moved a card onto the battlefield and then put
the counter on `LastMoved` now stamp it first (`Effect::SpellEntersWithCounters`
ahead of the `Move`), so it enters with it — Shilgengar, Queen's Bay Paladin,
Graceful Restoration, Battle of Hoover Dam and the rest. Still placed after
entry, because the moved card is chosen *by* the move (a search or a pick):

| Card | Mover |
|---|---|
| Cauldron's Gift | `MoveChosen` |
| Vision Quest | `SearchZones` |
| Magmatic Hellkite | the land search |
| The War in Heaven | its chapter's pick |
| Emperor of Bones | `SeparatedPile` |
| Sothera, the Supervoid | `Take` over its exile |

## Multiplayer wording — the N-seat audit, and what it still misses

Every implemented card whose oracle says "each opponent" was read against its
body for a fan-out ref. Adeline, Resplendent Cathar was fixed with
`Effect::ForEachOpponent` and Esper Sentinel with a per-caster count.

✅ **Per-opponent *targeting* is closed.** The eight cards that read "for each
opponent, [verb] up to one target X that player controls" — the five
Primordials, Grasp of Fate, Omega Heartless Evolution and Tempted by the Oriq
— now carry `Effect::ForEachOpponentTarget` (CR 601.2c): at most one target
per controller, at most one per opponent, chosen as the spell or ability goes
on the stack. It is a constraint on the chosen set rather than one target slot
per opponent, because a slot is declared statically by the card literal and
"one per opponent" is a count only the game knows.

⚠ **What it rounds off**, both minor and both documented at the variant:
- "Up to one" is what seven of the eight print, so the optional slot is
  faithful. Sylvan Primordial's clause is *mandatory* per opponent and could
  in principle be declined here; its targets are pure upside, so no real seat
  does.
- Sylvan's "for each permanent destroyed this way, search your library for a
  Forest" fires once per *target* rather than once per permanent that actually
  died, so an indestructible or regenerated target still fetches.

### The `each_opponent` audit's standing residuals

`scripts/audit_each_opponent.py` is the sweep in a script: every implemented
card's printed text read for a clause that names opponents as the *recipients*
of an effect, its factory body (plus the same-file helpers and
`effect::shortcut`) read for a fan-out reference. Trigger *timing* ("at the
beginning of each opponent's upkeep") is excluded — the event system already
fires once per such step. **Seven survivors as of 2026-09-18**, none of them a
plain missing fan-out:

| Card | Residual |
|---|---|
| ~~Malcolm, Keen-Eyed Navigator~~ ✅ **FIXED** | The batched trigger now carries `EventSpec::once_per_batch` and the engine's batch key carries the damaged subject (CR 603.2c), so it fires once per damaged opponent: two Pirates into one seat make one Treasure, one Pirate into each of two seats makes two. Printed is one trigger carrying a count rather than one per seat; nothing counts a trigger and the Treasures match. |
| Kaya, Spirits' Justice | the −2's "for each other player, exile up to one target creature that player controls" can't take `ForEachOpponentTarget`: slot 0 is a fixed own-creature target and `ApplyToTargets` rebinds every supplied target to slot 0. |
| ~~Absolute Virtue~~ ✅ | `ControllerProtectionFromMatching { ControlledByOpponent }` (2026-10-23): targeting and damage. A Curse an opponent already attached isn't shed (704.5m reads only seat protection). |
| Damping Engine | the whole "controls more permanents than each other player" lock, plus its sacrifice-to-ignore rider. |
| ~~Lavinia, Azorius Renegade~~ ✅ | Stale row (checked 2026-09-28, `01MgDzfC`): `cast_lock::ABOVE_LANDS` compares the spell against the **caster's own** land count (`game/mod.rs`), which is the per-opponent threshold. |
| ~~Spikeshell Harrier~~ ✅ | fixed 2026-10-25: `Effect::ReduceSpeedIfFastest` on the bounced creature's controller (CR 702.179f) — `spikeshell_harrier_slows_only_the_fastest_opponent`. |
| Grim Reminder | not a gap — `Effect::SearchRevealPunishSameNameCasters` is the per-opponent walk, under a name the regex can't see. |

Two residuals the CR 603.2c batch ratchet
(`catalog_registration::every_batched_damage_trigger_fires_once_a_batch`) signs
off by name rather than fixes, both filed here:

| Card | Residual |
| --- | --- |
| ~~Quartzwood Crasher~~ ✅ | Stale row (checked 2026-09-28, `01MgDzfC`): the trigger is `once_per_batch_summing_damage()` — X is the batch's summed damage to that player. |
| ~~Magmatic Galleon~~ ✅ | Stale row (checked 2026-10-24): the Treasure trigger is `DealtExcessDamage { noncombat_only: true }` scoped to opponents' creatures, `once_per_batch()` (`586474bda`); tested in `recent_b/cmdr_edhrec18.rs`. |

## Cultural Exchange's two player slots (2026-09-20)

"Choose any number of creatures **target player** controls. Choose the same
number of creatures **another target player** controls. Those players exchange
control of those creatures." Modelled as `ExchangeControlChoosing` between
**you** and an opponent, so there are no two player slots at all and the
printed "another" has nowhere to hang.

⚠ **The narrowing has teeth at N seats**: the printed card can swap two
*opponents'* creatures, handing one of them a board it never had, and this
cannot. In a duel the two readings coincide. Allowlisted in
`audit_another_target` with that reason rather than silently passing.

## The "defending player" class (2026-09-20) — closed, with two named references and one residual

32 cards print a clause that selects a permanent "defending player controls";
all are on `SelectionRequirement::ControlledByDefendingPlayer` now.
`scripts/audit_defending_player.py` gates it. Two rows answer the seat by a
**different, equally exact reference** rather than by the atom, and both are
allowlisted with the reason:

- **Tromokratis** — "can't be blocked unless all creatures defending player
  controls block it" is `Keyword::CantBeBlockedUnlessAllBlock`, and CR
  509.1b's loop in `combat.rs` scopes it with `defender_for(atk.target)`,
  per attack. Exact at any seat count.
- **Kusari-Gama** — its `DealsCombatDamageToCreature` body resolves *after*
  the combat teardown, so no attack record is left to read the seat from.
  The trigger binds the damaged **blocker** as slot 0 and that creature's
  controller is the defending player by definition.

⚠ **One residual, unrelated to the seat.** Kusari-Gama's "each **other**
creature defending player controls" is approximated as *the non-blocking
ones*, so a second blocker that was not the one damaged is spared where the
printed card splashes onto it. Pre-existing, and orthogonal to this class.

⚠ **Two cards can only be cast during combat and always could have been.**
Yare and Mercadia's Downfall are instants whose only reference is "defending
player", which CR 506.2 defines relative to an attacking creature. With no
attack declared they have no legal target and cannot be cast — correct, and
a change from the previous behaviour, which let them name any opponent's
creature at any time. A combat with **two** different defenders leaves the
clause ambiguous and the filter takes nothing; the printed cards predate
multiplayer templating and the CR gives no tiebreak.

## Non-targeted clauses that still declare a target slot (2026-09-20)

69 non-Aura cards print no "target" anywhere and still carry a
`target_filtered` slot. The **six** where the target provably cannot see what
the clause is about are fixed and gated by
`scripts/audit_resolution_order.py` (a mill or surveil that fills the zone
the same resolution then reaches into). What remains is a census, not a
defect list:

- **The karoo shape** — "when this land enters, return a land you control to
  its owner's hand": Azorius Chancery, Boros Garrison, Dimir Aqueduct,
  Golgari Rot Farm, Gruul Turf, Izzet Boilerworks, Rakdos Carnarium, Selesnya
  Sanctuary, Simic Growth Chamber, plus Kor Skyfisher, Whitemane Lion,
  Species Gorger and the Planeshift gainlands (Fleetfoot Panther, Horned
  Kavu, Lava Zombie, Silver Drake, Shivan Wurm, Sawtooth Loon, …).
- ⚠ **What it rounds off**: the choice is made when the trigger goes on the
  stack rather than on resolution, so it fizzles if the chosen permanent
  leaves in response where the printed trigger would pick another; and a
  permanent with **shroud** cannot be chosen at all, because a target slot
  is a target.
- These are all "a permanent **you control**", so the practical exposure is
  narrow — your own shroud creature, or your own permanent bounced in
  response to your own trigger.

Left as a census on purpose: nothing here is wrong on a still board, and a
ratchet whose rows are not bugs is one nobody closes.

## Exert: the cost is announced, the modal is not (2026-09-20)

CR 508.1g / 701.43d. `GameAction::DeclareAttackersExerting { attacks, exert }`
carries the announcement and `EventKind::Exerted` carries the linked "when you
do", so a creature that attacks without paying gets neither the bonus nor the
skipped untap. All six exert cards are on it.

⚠ **What is left is the ask.** A declaration that does not announce the cost
falls to `exert_pays_off` — take the exert when the linked bonus has a legal
target or needs none — which is a policy, not a choice. A `wants_ui` seat
therefore cannot decline an exert its policy would take, or take one its
policy declines, because no `Decision` is surfaced at declare-attackers time.
The shape for it is a per-attacker ask inside the declaration, and it is the
same gap `DeclareAttackersBanded` has for bands.

💡 The policy is answerable *only because the bonus is linked*: with the old
unconditional `Attacks` trigger there was nothing on the card that said which
trigger the exert was buying.

## Melee is applied off-stack when it is a printed keyword (2026-09-20)

CR 702.121a — "Melee is a **triggered** ability. 'Melee' means 'Whenever this
creature attacks, it gets +1/+1 until end of turn for each opponent you
attacked with a creature this combat.'" The engine has two representations of
it and only one is a trigger:

- `Keyword::Melee` is applied as a **direct pump inside `declare_attackers`**
  (`combat.rs`, next to the exert and Raid bookkeeping) rather than put on the
  stack. Wings of the Guard, Grenzo's Ruffians, Deputized Protester.
- `effect::shortcut::melee()` is the real trigger
  (`Value::OpponentsAttackedThisCombat`), which is what a card that *grants*
  melee has to use. Adriana, Captain of the Guard's second line.

The **size** is the same either way — the count is fixed at declaration and
CR 702.121a's own ruling says it does not matter whether the attackers are
still attacking, still on the battlefield, or still in the game. What the
off-stack half rounds off is that the bonus cannot be responded to, is not
seen by "whenever a player casts a spell or a triggered ability is put onto
the stack" effects, and survives a Stifle. Both halves do stack, which is CR
702.121b ("if a creature has multiple instances of melee, each triggers
separately") — a Wings of the Guard under Adriana gets both, which
`cr_702_121_melee_counts_opponents_and_each_instance_triggers_separately`
pins at 1/1 → 5/5 over two attacked opponents.

⚠ The citation was **702.122** in `effect.rs`, `effect/shortcut.rs` and one
test; 702.122 is *Crew*. Melee is **702.121**. Fixed at the same commit.

## The zone-blind target class (2026-09-20) — closed, with two handed on

`scripts/audit_target_zone.py`. **The filter language has no implicit zone**:
`legal_targets_for_filter` applies the same `SelectionRequirement` to every
zone an effect can reach, so `target_filtered(R::Creature)` on a printed
"target creature card **from your graveyard**" is satisfied by a creature on
the *battlefield* — and `auto_target_for_effect` walks the battlefield first.
27 cards when the audit was written, **0** now (`--gate` fails on any row).

What made it a class rather than a list is that half of it was **invisible in
self-play**: `Effect::prefers_graveyard_target` sends the picker to the
graveyard for a `Move` to your hand, to your battlefield, or to exile, so
those cards mostly did the right thing for the wrong reason and only misfired
when every graveyard was empty. The other half has no such classifier — every
`Move` to a **library**, and every bespoke effect — and misfired always.
Mortuary Mire (a Judith pod-deck land) tucked live creatures; Cremate exiled a
permanent off the battlefield for {B}; Scrabbling Claws, Ghost Vacuum, Ambush
Wolf, Leonin of the Lost Pride and Break Ties were the same shape.

- The filter says the zone with `from_your_graveyard()` /
  `from_any_graveyard()` / `InOpponentGraveyard`, or the effect variant
  carries it in its own name.
- A **mixed** clause ("from the battlefield and/or from graveyards") needs
  `SelectionRequirement::OnBattlefield` for its on-board half; without it that
  half is zone-free and reaches a hand, a library and exile as well. Angel of
  Serenity and Daretti's ultimate are the two.
- **Exile is the same defect and a much rarer wording.** Two rows, and a
  clause can name both zones: Sorceress's Schemes returns "target instant or
  sorcery card from your graveyard **or** exiled card with flashback you
  own", and a body that says the first is not a body that says the second —
  the audit asks for every zone a clause names, not the first.
- Hand and library were measured and are empty.
- Two exemptions, both conditioned on the engine arm and re-checked every run:
  `Effect::CommandTheDreadhorde` and `Effect::WeldArtifacts` build their own
  candidate lists out of `players[*].graveyard`, so no filter can aim them at
  the battlefield.

⚠ **Two rows were handed to `audit_incomplete`** (Rydia's Summon shipped 2026-10-11; Elrond's mana-ability gap filed the same day and fixed late that day): the clause is not
modelled at all, so there is no filter to correct.

| Card | Residual |
| --- | --- |
| Elrond, Moon-Reader | ✅ fixed 2026-10-11: a `{T}` creature mana ability draws off `TappedForMana`, any other off `AbilityActivated { mana: true }` (`ActivationTrait::ManaWithoutTap`, emitted only for `{T}`-less mana abilities, so no hot-path cost); one shared once-a-turn. |
| Jeska, Thrice Reborn | ✅ fixed 2026-10-28: a prompting seat names each extra slot of a multi-target loyalty ability as it activates (`ResumeContext::LoyaltyExtraTargetPick`, validated like slot 0, CR 115.3 distinct, nothing paid until named — `jeska_minus_x_asks_a_prompting_seat_for_each_extra_target`); a headless seat keeps `auto_extra_targets_for`. (Tasha's +1 fills one card from **each** opponent's graveyard — `tasha_pages_one_card_from_each_opponents_graveyard`, 2026-10-24.) |

Three approximations the fixes left standing, each documented at its variant:

| Card | Residual |
| --- | --- |
| Scrabbling Claws | ✅ targets the **player** (`TargetPlayerThen` + `ExileFromGraveyard { Target(0) }`, CR 115.1) and that player picks the card (`ExileFromGraveyard` asks the exiling seat; the headless pick is the cheapest). |
| Goblin Welder | ✅ fixed 2026-10-26: the graveyard card is slot 1, `InGraveyardOfTargetSlotController(0)` (CR 601.2c), and `WeldArtifacts { a, b }` does nothing unless both are still legal. |
| Ambush Wolf / Angel of Serenity | ✅ closed 2026-10-30: the Wolf's "up to one" is `OptionalTargets { min: 0 }` (declines, never takes its own card) and the Angel is `ApplyToTargets { min_targets: 0 }` with `OtherThanSource`. |

## A layer observation, filed by the card that surfaced it (2026-09-20)

**Cyberdrive Awakener** animates each *noncreature* artifact you control into
a 4/4 with `Effect::BecomeCreature` (not `BecomeCreatureLosingTypes` — the
printed line adds the creature type and keeps the artifact one, which the test
asserts). Its own static, "Other artifact creatures you control have flying",
then does **not** reach the Sol Ring it just animated, although a permanent
that was *printed* an artifact creature does get the keyword.

🔎 That is a **layer** question, not a card gap: a
`StaticEffect::GrantKeyword { applies_to: EachPermanent(filter) }` whose
filter names a card type does not see a type another continuous effect added
in layer 4 (CR 613.1d, and the dependency in CR 613.8). Nothing else in the
catalog pairs an animate-all with a type-filtered anthem on the same card, so
this is the first place it shows. The card is complete and its test says both
halves; the interaction is the open item, and it wants a run that can price
re-evaluating `applies_to` against computed characteristics rather than
printed ones.

## The "another target" class (2026-09-20) — closed, with four approximations named

`scripts/audit_another_target.py`, and ENGINE_BACKLOG's fifty-third find.
41 cards print "another target"; the census splits them by where the *other*
object comes from and both columns are now 0.

Four rows are allowlisted on the **source-relative** side with their reasons,
and two of them are real approximations rather than false positives:

| Card | Residual |
| --- | --- |
| ~~Jackdaw Savior~~ ✅ | Not a gap (checked 2026-10-24): the target needs mana value **less than** the dying creature's (`R::ManaValueLessThanEventAmount`), so the creature that died can never be it — "another" is implied by the MV filter. |
| Blade of Shared Souls | 🟡 "another" is other than the creature the Equipment is **attached to**, which is not the source either. Same gap, same missing ref. |
| ~~Etched Slith~~ ✅ | Fixed 2026-10-24: the "when you do" is a `ReflexiveTrigger` over a `MayDo` `RemoveCountersUpTo` targeting another permanent or an opponent. |
| Fiendish Panda | ✅ not a gap: the printed filter is "another target **non-Bear** creature card" and the Panda is a Bear Demon, so the type line already excludes it. |
| Atzocan Archer / Nessian Wilds Ravager | 🟡 "may have it fight **another** target creature" is narrowed to "a creature you don't control" **on purpose**. The "may" is answered by `optional_trigger_beneficial`, which takes a body with no self-cost; the picker's last-resort rank then hands a mandatory defender slot the bot's *own* creature on a board with no opposing one. A narrowing never allows an illegal play, which is what this class is about. |

Five are allowlisted on the **slot-relative** side because the two slots
cannot name one object: Pit Fight, Go for Blood's shape, Domri Rade's −2 and
Ulvenwald Tracker are all "you control" against "an opponent controls";
Stiltzkin, Moogle Merchant's slot 0 is a *player*; Comet Storm is one
`ApplyToTargets` instance, where CR 115.3 already forbids the repeat.

⚠ **Four fight cards are approximations in the other direction and stay.**
Domri's −2, Ulvenwald Tracker, Atzocan Archer and Nessian Wilds Ravager all
print "another target creature", which includes one of your own;
`OtherThanTargetSlot(0)` (or `OtherThanSource`) is the faithful filter and
each was tried. **A `Fight` defender slot is classified hostile, so the
picker ranks an own creature last — but "last" is still "picked" on a
mandatory slot with nothing else legal**, so the faithful filter hands the
bot a strictly losing fight on an empty opposing board. A narrowing cannot
make an illegal play; widening here makes a legal bad one. The rules argument
is real and the fix belongs with the bot's fight evaluation, not the filter.

## CR 903.4 color-identity divergences (from the Scryfall audit)

`cr_903_4_computed_color_identity_matches_scryfall` (core_rules /
`catalog_registration.rs`) compares every implemented card's computed identity
against Scryfall's `color_identity`: **18,058 cards resolved, 3 divergences**,
each a card gap rather than an identity-walk gap. The list in that test is the
ratchet; this is the engineering view of it.

| Card | Gap | What it needs |
|---|---|---|
| Balduvian Fallen | the "+1/+0 for each {R} spent to pay cumulative upkeep" payoff is dropped | a per-color read of what paid a cumulative upkeep; no primitive |
| Bounty Hunter | not a card gap — the Scryfall cache resolves the name to a different card (a Vanguard avatar) than the catalog's creature | nothing; the row documents the collision (Maraxus of Keld's computes since the creature landed in `cmdr_015x_a`) |

## The INVENTED-ability columns (2026-09-19) — residuals only

Eleven cards that did something they do not print were rewritten to the
oracle at the forty-fifth find (ENGINE_BACKLOG has the method and the list).
Both auditors read **0** at the closing tip:

```
python3 scripts/audit_invented_may.py      # 0 / 948 bodies with a needle
python3 scripts/audit_invented_trigger.py  # 0 / 4,103 claims, 20 allowed, 0 stale
python3 scripts/audit_invented_rider.py    # 0 / 1,248 bodies with a needle, 4 rule-implied
python3 scripts/audit_synthesised_name.py  # 0 / 21,383 documented factories
```

The **rider** column is a table — one line per rider — and opened with four
rows: `DestroyNoRegen`, `CantBeRegeneratedThisTurn`, `once_per_turn: true` and
`sorcery_speed: true`; **ten** more followed, every one of them the mirror of
a `clause_ratchet` row in `core_rules/catalog_registration.rs`
(`EntersTapped`, `etb_tap()`, `enters_with_counters: Some(`,
`Keyword::CantBeCountered`, `HexproofFrom`, `Keyword::MustAttack`,
`MustBeBlocked`, `PreventUntap`, `Keyword::Unblockable`,
`Keyword::Defender`), taking the column from 392 bodies to 1,248 and naming
four more cards:

| Card | Shipped | Prints |
|---|---|---|
| Cephalid Coliseum | enters tapped; wheel costs `{2}{U}`; no Threshold gate | no enters-tapped clause; `{U}`; "Activate only if there are seven or more cards in your graveyard" |
| Glimmerpost | enters tapped (Cloudpost's clause, not its own) | no enters-tapped clause |
| Deep-Sea Kraken | `CantBeCountered`; Suspend 9—`{1}{U}` | "This creature can't be blocked"; Suspend 9—`{2}{U}` |
| Gleaming Overseer | `Unblockable` to every Zombie you control | "Zombie **tokens** you control have hexproof and **menace**" |

Two of those were one bug each behind the card. `locus_count_value` is shared
by Glimmerpost and Cloudpost and filtered to Loci **you control** where both
print "on the battlefield"; and Gleaming Overseer's token filter was
*unreachable* until the engine stopped dropping it (ENGINE_BACKLOG, the
forty-sixth find). ⚠ **A keyword is printed wording**: modular (CR 702.43a)
and STX's prepared each cost a false positive on the
`enters_with_counters` row, and "hexproof from" is the modern keywording of
"can't be the target of …" on three older cards.

It read 14 and closed at 0: eight sweepers and spot
removal carried a "can't be regenerated" rider nothing printed (Akroma's
Vengeance, Fumigate, Hush, **Mortify**, Planar Cleansing, Planar Outburst,
Pure Reflection, Starfall Invocation — Mortify's own test asserted the wrong
rule and was rewritten), Ominous Seas ran on a different clock in three ways
at once, and Lonis, Genetics Expert carried **Lonis, Cryptozoologist's**
abilities. One residual and two rule-implied allowances:

| Card | Residual | Why |
|---|---|---|
| ✅ Phyrexian Battleflies | closed 2026-09-29: one ability with `max_activations_per_turn: Some(2)` (it had become an uncapped copy plus a once-per-turn copy, i.e. unlimited) | `core_rules/cr_recent109.rs::phyrexian_battleflies_pumps_at_most_twice` |
| Arcanum Wings | `sorcery_speed: true` with no printed clause | CR 702.64a — aura swap **is** "Activate only as a sorcery" by rule |
| Roadkill Rodney | same, reported in the audit's token bucket | the flag is on the **Mutagen token**, whose own printed text says "Activate only as a sorcery"; a factory holds its tokens' rules text and the card's oracle does not |

Two of the first eleven kept a documented residual rather than shipping whole:

| Card | Residual | Why |
|---|---|---|
| Tome of the Infinite | the activation's `Draw 1` stands in for "conjure a random card from its spellbook" | Alchemy conjure has no primitive (pre-existing row below). The invented ETB scry beside it is gone |

## The synthesised-name class (2026-09-19) — closed on the docs, and what it left

`scripts/audit_synthesised_name.py` reads **0**. ⚠ **Read that as a hygiene
ratchet, not a proof about bodies**: it asks whether a factory's doc comment
still *claims* to be a synthesised card under a name Scryfall owns. All 27
rows were read body-against-oracle by hand this run — eight were wrong cards
and were rewritten, the rest were correct bodies under docs describing the
card they used to be, which is `audit_doc_drift`'s 338-stale-comments
population showing up again by a different route.

Wrong bodies fixed: **Cunning Rhetoric** (the trigger was right, the payload
was a flat drain instead of exile-and-may-play), **Brilliant Restoration**
(one target creature plus 2 life, where the print returns *all* artifact and
enchantment cards), **Reduce // Rubble** (a Lesson sorcery dealing 3 damage,
where the print is an Amonkhet split with Aftermath), **Detective's Phoenix**
(an invented dies-trigger), **Silverquill Lecturer** (an invented Magecraft
pump), **Sigardian Savior** (one target, no cast gate), **Grim Bounty** (no
planeswalker half) and **Pyrotechnics** (creatures and planeswalkers only,
where the print says any number of *targets*).

Residuals:

| Card | Residual | Why |
|---|---|---|
| Detective's Phoenix | the whole **bestow** half — the alternative cost, the Aura mode, its +2/+2 and keyword grant, and the graveyard-cast permission that hangs off it | CR 702.103 Bestow has no `Keyword::Bestow` and Collect evidence has no cost form. The body is the printed 2/2 Enchantment Creature with flying and haste and nothing else |
| Witch's Cauldron | the sacrifice is an activation **cost** on the print and is resolved in the body here | it happens on resolution rather than on announcement, so a response can no longer be made to a creature that is already gone. ⚠ Its old row in this file claimed the lifegain scaled with the sacrificed creature's toughness; the printed card gains **1**, the body always did, and the doc comment asserted the opposite |

### Body-only stubs — entire signature ability missing (all ✓ code-verified)
| Card | Location | Missing |
|---|---|---|
| ~~Vendilion Clique~~ ✅ **FIXED** | mod_set/creatures.rs:3852 | ETB hand disruption — wired via new `Effect::BottomChosenFromHandAndDraw` (look at hand → choose nonland → bottom + draw). Targets `EachOpponent` (1v1-faithful; self-cast mode pending player-targeting on triggers). Tests: `vendilion_clique_is_3_1_legendary_flash_flying`, `…_etb_bottoms_chosen_card_and_target_draws` |
| ~~Torrential Gearhulk~~ ✅ **FIXED** | mod_set/creatures.rs:3875 | ETB "cast instant from graveyard" — wired via `CastWithoutPayingImmediate { Graveyard, exile_after }` (tests: `torrential_gearhulk_is_5_6_artifact_flash`, `…_etb_casts_instant_from_graveyard_and_exiles_it`) |
| ~~Phyrexian Obliterator~~ ✅ **FIXED (1v1-approx)** | mod_set/creatures.rs:4006 | Damage-retaliation now wired: `DealtDamage`/`SelfSource` → `Sacrifice { count: TriggerEventAmount }`. Sacrificer is `EachOpponent` (faithful in 1v1; "that source's controller" can't be read — `GameEvent::DamageDealt` carries no source). Doc P/T corrected 5/8→5/5. Tests: `phyrexian_obliterator_is_5_5_trample`, `…_damage_forces_opponent_to_sacrifice_that_many` |
| ~~Alesha, Who Smiles at Death~~ ✅ **FIXED** | ktk/mod.rs | Attack trigger now wired: `on_attack(MayPay { {W/B}{W/B} → Move(target gy creature pow≤2 → battlefield tapped) + JoinCombatAttacking(LastMoved) })`. New `Effect::JoinCombatAttacking` puts the reanimated creature into combat attacking (CR 508.3a); `Move→Battlefield` + `MayPay` now bias the trigger auto-targeter to the graveyard. Test `cr_508_3a_alesha_reanimates_tapped_and_attacking`. |

**Three more found at the fifty-fourth pass, and the finder was not this
file** — it was `scripts/audit_dropped_may.py`, reading the oracle beside the
definition for the "you may sacrifice / tap" cluster. All three were whole
abilities absent, not dropped riders:

| Card | Location | Was missing |
|---|---|---|
| ~~Springbloom Druid~~ ✅ **FIXED** | decks/modern.rs | "you may sacrifice a land. **If you do**, search…" — the sacrifice was not there at all, so the card was free two-land ramp; its doc comment asserted that wrong oracle *and* a 2/2 body for a 1/1. Now `Effect::MaySacrifice`. Tests take it and decline it |
| ~~Tidal Terror~~ ✅ **FIXED** | eoe.rs | the whole attack trigger ("you may tap two other untapped creatures you control. If you do, this creature can't be blocked this turn") — the card was a vanilla 5/6 with Islandcycling. Now `Effect::MayTap` into an end-of-turn `Unblockable` |
| ~~Bristlebud Farmer~~ ✅ **FIXED** | decks/recent.rs | the whole attack trigger ("you may sacrifice a Food. If you do, mill three cards. You may put a permanent card from among them into your hand") — the ETB minted two Foods with nothing to feed them to. Now `MaySacrifice` into `MillThenToHand` |

**The load-bearing cluster was read to the end, 2026-08-25, and it is nine
entries — five real, four false positives.** `audit_dropped_may.py` now reads
346 of 11,094 cached names; filtering its output to the verbs where declining
is a real choice (`you may destroy / sacrifice / tap / discard`) leaves nine,
and every one was checked against the oracle:

| Card | Was | Now |
|---|---|---|
| ~~Awaken the Honored Dead~~ ✅ | a `{5}{W}{B}` **Sorcery** returning *every* creature card in your graveyard to the battlefield | the printed `{B}{G}{U}` **Enchantment — Saga**: I destroy target nonland permanent, II mill three, III `MayDiscard` → return target creature/land **from your graveyard** to hand. Not a dropped "may" at all — a different card |
| ~~Myr Battlesphere~~ ✅ | tapped *every* untapped Myr on every attack | `Effect::MayDo` over the whole package (the printed intermediate X has no primitive) |
| ~~Mox Diamond~~ ✅ | discarded *any* card, unconditionally, and kept the Mox either way | `StaticEffect::EntersOnlyIfPaid` (`game/entry_cost.rs`, CR 614.1c) — paid or refused before it enters, on all four entry hops; refused, it never enters (2026-10-09) |
| ~~Cloudpiercer~~ ✅ | mutate trigger discarded and drew unconditionally | `Effect::MayDiscard` |
| ~~Highway Robbery~~ ✅ | discarded and drew two unconditionally, and would draw two off an empty hand | `Effect::MayDiscard`; the "or sacrifice a land" half of the choice is still not modelled and the doc says so |
| Voltage Surge ✓ | — | false positive: `kicker_action_cost` is already optional |
| Plumb the Forbidden ✓ | — | false positive: `AdditionalCastCost::SacrificeAnyNumber` — "any number" includes zero |
| Devouring Greed ✓ | — | false positive, same shape |
| Devouring Rage ✓ | — | false positive, same shape |

**Two rules the pass yields.** *One:* four of nine were false positives and all
four are the same shape — the choice spelled as an **optional additional cast
cost** rather than an in-tree `May*`, which the auditor cannot see. Check
`kicker_action_cost` / `additional_cast_cost` before writing a fix. *Two:* a
"return target [thing] **from your graveyard**" needs
`SelectionRequirement::InYourGraveyard`; without it the walker will take the
creature your own removal just killed out of the **opponent's** graveyard,
which is what Awaken's chapter III did on the first attempt.

**The other ~337 findings are the "you may draw / search / put into your hand"
tail**, where declining is almost never right and the dropped choice costs a
game nothing. Read the oracle before fixing one; the residue still has false
positives, and this pass's four are the reason.

**THE LESSON WAS TESTED AT THE EIGHTY-FIRST PASS AND IT HOLDS — filter the
tail by "If you do".** Of the 341 findings, **46 have "you may X. If you do,
Y" in the full oracle**, and that is the sub-tail worth reading: a dropped
"may" there means the *consequence* is forced too, so the card pays a printed
cost it was allowed to decline. The filter is four lines against
`.scryfall_cache.json` (the auditor truncates its snippet at 80 chars, so
`grep` on its output finds none of them). Two of the 46 were real and both
are now fixed:

| Card | Was | Now |
|---|---|---|
| ~~Sanctuary Wall~~ ✅ | `{2}{W}, {T}`: tap target **and** stun it **and** stun itself, unconditionally — the activation cost its own untap step every time | the stun pair is one `Effect::MayDo`; the tap is still mandatory. Tests take it and decline it |
| ~~Frantic Scapegoat~~ ✅ | a 1/1 haste body with only the ETB self-suspect — **the second ability was absent**, so it kept menace-and-can't-block forever | "whenever another creature you control enters, **if this is suspected**, you may suspect that one instead" — `EntersBattlefield`/`YourControl` + `Predicate::SelectorExists(IsSource.and(IsSuspected))` (CR 603.4) + `MayDo(Suspect, ClearSuspected)`. Two approximations, both documented on the card: "another creature you control" rather than one of this batch, and per-creature rather than per-batch |

And the third thing the sub-tail teaches, which is the same shape as the
2026-08-25 pass's false positives: **check for a bespoke optional primitive
before writing a fix.** Obzedat, Ghost Council reads as a dropped "may" and
is not one — it is `Effect::MayExileSelfReturnNextUpkeepHaste`, an optional
effect the auditor's `OPTIONAL` list does not name. Of the rest, the
cost-bearing ones (Lamplight Phoenix, Izoni, Kozilek's Return, Aphemia) route
through `Effect::CollectEvidence` / `Effect::If`, where the forced branch is
already gated on being able to pay and the payoff dominates.

**The lesson for the next sweep**: the dropped-"may" audit is a *body-stub*
finder as much as an optionality finder. A card whose printed text is "you may
X. If you do, Y" and whose definition has neither X nor the choice reads to
the audit as a dropped "may" and is really a missing ability — which is the
more serious defect and the easier one to confirm.

### Wrong-effect substitutions — implemented card is functionally a different card
| Card | Location | Substitution |
|---|---|---|
| ~~Silverquill Penkeeper~~ ✅ **FIXED** | silverquill.rs:14312 | now `magecraft(Effect::Discard { EachOpponent })` — matches its own documented "each opponent discards" intent (was Drain 1) |
| ~~Silverquill Wordweaver~~ ✅ **FIXED** | silverquill.rs:14653 | now `etb(Effect::Discard { EachOpponent })` (was Drain 2) |
| ~~Witherbloom Necromancer~~ ✅ **FIXED** | witherbloom.rs:10706 | now `on_other_dies(MayPay { {1} → Move(TriggerSource → battlefield) })` — real reanimate-the-just-died-creature (was Drain 1), same mechanism as Minion's Return |
| ~~Echocasting Symposium~~ ✅ | sos/sorceries.rs | already on `CreateTokenCopyOf` (doc was stale) |
| ~~Rush of Knowledge~~ ✅ | stx/mono.rs | `Value::HighestManaValueAmong` (was hardcoded draw 4) |
| ~~Stingerback Terror~~ ✅ **FIXED** 2026-08-30 | decks/modern.rs | shipped as an **invented card**: a {2}{R}{R} *Legendary* 7/7 Scorpion Dragon with menace, ward—pay 3 life and saddle 3, whose saddled attack drained half of each opponent's life. No such card exists and the cache holds no Mount with that text. Now the printed one — flying, trample, `PumpSelfByValue { HandSizeOf(You), -1/-1 }`, `plot_cost` {2}{R} — all four primitives already shipped. Its three saddle-mechanics tests moved to Brightfield Glider. |
| ~~Descendant of Storms~~ ✅ **FIXED** 2026-08-30 | mod_set/creatures.rs | shipped as a 2/2 flying **Spirit** with a dies-trigger that made a Human Soldier token — the card's own types and its token's, swapped, plus a keyword and a trigger it does not have. Now the printed {W} 2/1 Human Soldier with "whenever it attacks, you may pay {1}{W}; if you do, it endures 1" (`MayPay` + `Effect::Endure`, both shipped). |

| ~~Surging Might~~ ✅ **FIXED** 2026-08-30 | decks/recent.rs | printed a {2}{G} **Aura** for +2/+2 with ripple 4; shipped as a {2}{G} Instant for +1/+1 and trample. Now the Aura. |
| ~~Mob Mentality~~ ✅ **FIXED** 2026-08-30 | stx/extras_04.rs | printed a {R} Aura granting trample; shipped as a *synthesised* "creatures you control get +1/+1, and first strike if you cast another spell" Instant — **under a name Scryfall owns**, which is the catalog's own synthesis rule broken. Now the Aura; its "whenever all non-Wall creatures you control attack, enchanted creature gets +X/+0" clause wants an all-attack trigger and is the residual. |
| ~~Heroic Defiance~~ ✅ **FIXED** 2026-08-30 | stx/iconic.rs | same shape as Mob Mentality: a printed {1}{W} Aura for +3/+3, shipped as a synthesised "+1/+1, hexproof and indestructible" Instant under a real name. The printed "unless it shares a color with the most common color among all permanents" gate wants a board-wide colour census and is the residual. |
| ~~Thing in the Ice~~ ✅ **FIXED** 2026-08-30 | decks/modern.rs | a Wall, not a `Creature — Horror`; its Awoken Horror back was a plain Horror, not a `Kraken Horror`. |
| ~~Sundering Eruption~~ ✅ **FIXED** 2026-08-30 | mod_set/sorceries.rs | {1}{R} for a {2}{R} card, and its back face was named "Mount Tyrhus" — the printed back is **Volcanic Fissure**. |

**Both were found by the same instrument and neither was on any tracker row**:
`scripts/audit_catalog_stats.py`'s type and keyword columns, read against the
committed scryfall cache after that script's own cost column was fixed (it had
been reading an ability's `cost:` as the card's, which reported the four
Legends Elder Dragons as drift against their own upkeep). **Audit the audit
before believing its silence**: the cost column read 8 findings and 0 of them
were real; after the fix it reads 0 findings, and the eight real defects were
in the columns nobody had disbelieved.

**And then the audit grew the two columns nobody had ever run catalog-wide —
card type and supertype — and they were worth 107 more cards.** The classes,
in descending damage: 34 spells at the wrong *speed* (24 printed Instants
shipped as Sorceries, 10 the other way — a Treasure Cruise the bot could cast
on the opponent's turn); 23 permanents missing **Legendary**, so the legend
rule never fired on Karn Liberated, Liliana of the Veil, Gaea's Cradle or the
four God-Eternals, and 9 carrying it wrongly, so a second Gray Merchant of
Asphodel was being sacrificed; 5 affinity creatures (Broodstar, Somber
Hoverguard, Carapace Forger, Qumulox, Glaring Fleshraker) typed as **artifact**
creatures, counting themselves toward their own affinity; 9 artifact creatures
missing Artifact; 6 enchantment creatures missing Enchantment; 13 tribal spells
missing Kindred.

**The keyword column then came down 48 -> 23 the same way, and 25 of the 48
were the reader.** A flat `Keyword::(\w+)` scan read the keyword inside a
*payload* as printed, so `CantBeBlockedExceptBy(HasKeyword(Flying))` — the
right primitive for "can't be blocked except by creatures with flying" — made
five correct cards read as fliers; a file-local `fn ward_1() -> Keyword` read
as no keyword; and `HexproofFrom*` / `ProtectionFromMatching` are what Scryfall
reports as plain "Hexproof" / "Protection". The eleven real ones were three
unprinted evasion keywords (Tempest Angler, Outcaster Trailblazer, Aquastrand
Spider), Daxos's Indestructible, Feral Throwback's Trample, The Locust God's
Haste (its *tokens* have it), Moonshadow's Flying-for-Menace, Kitesail
Larcenist's missing Ward {1}, Azorius First-Wing's missing protection from
enchantments, and two cards whose printed *grant* had been flattened to a
static keyword (Putrid Imp, Voltaic Brawler).

**The 23 that remain are modelling choices, not defects** — a keyword standing
in for spelled-out text (Frost Titan's ward,
Necromancy's flash, Mistform Ultimus's changeling) or a conditional/granted one
(Paradise Druid's untapped hexproof, three Goblins' haste, prowess-as-a-trigger,
granted vigilance). Closing that residue is shape work on the filter.

**And the stat columns are now clean while the EFFECT trees have never been
cross-referenced at all.** Two of the eleven turned up cards whose printed
*text* does not match either — Tempest Angler printed "whenever you cast a
noncreature spell, put a +1/+1 counter on this creature" and shipped an ETB
scry 2; Outcaster Trailblazer printed an ETB mana, a power-4 draw trigger and
plot {2}{G}, and shipped a "cast a spell with mana value 5+" draw-and-token.
**Neither is visible to any column this audit has**, and both were found only
because a keyword row pointed at the card. ~~That is the next audit~~ — both
are FIXED (Tempest Angler earlier; **Outcaster Trailblazer 2026-09-05**: ETB
`AddMana AnyOneColor`, the `EntersBattlefield`/`YourControl` draw filtered on
`Creature ∧ PowerAtLeast(4) ∧ OtherThanSource`, plot kept; test
`modern::theros_fading_adventure::outcaster_trailblazer_etb_mana_and_power_four_draw`).
The oracle-text-against-effect-tree audit is `scripts/audit_oracle_verbs.py`.

**The reusable half is what it cost to make those columns believable.** Four
distinct reading bugs came out of getting there and three were already wrong
for the audit's *existing* columns — a card's body ran past the next private
`fn`, a bound `let back = CardDefinition { … }` answered for the card, an
aliased `Sup::Legendary` read as no supertype at all, and a helper whose
"constant" was `vec![if sorcery { … } else { … }]` read as both types. **Every
one of them was found by disbelieving a class of findings that was too large
to be true, and checking three of its rows in the source.** The four smallest
classes in the first run were the real defects; the two largest were bugs in
the reader.

Note: Silverquill Penkeeper/Wordweaver and Witherbloom Necromancer above are
**synthesized** fabricated-name cards (only `_b###` factories exist), so the
"substitution" is moot — there's no real oracle to match.

### Dropped static abilities on legendaries (whole side absent)
| Card | Location | Missing |
|---|---|---|
| ~~Callaphe, Beloved of the Sea~~ ✅ **FIXED** | thb.rs | "{1} tax on opponents' spells targeting your creatures/enchantments" now wired via the existing `StaticEffect::TaxOpponentSpellsTargeting` (the stale doc claimed `extra_cost_for_spell` couldn't read the cast target — Jubilant Skybonder already proved otherwise). Test `callaphe_taxes_opponent_spells_targeting_your_permanents`. |
| ~~Siona, Captain of the Pyleas~~ ✅ **FIXED** | thb.rs | "Aura becomes attached → make a 1/1 Soldier" — wired via new `GameEvent::AuraAttached`/`EventKind::AuraAttached` (CR 303.4), emitted when an Aura resolves attached; `EventScope::YourControl` requires the host to be a creature you control. Test `siona_makes_a_soldier_when_aura_attaches_to_your_creature`. |

### Activated-ability costs — a column nobody had, 2026-09-07: 27 shipped cards at the wrong price

`scripts/audit_catalog_stats.py` read a card's *own* mana cost against the
cache and never an ability's. Manifold Key (above: {1},{T} for a printed
{3},{T}, a free untap for a printed {1}) was found by hand; the new `abil`
column reads every `ActivatedAbility` literal's `mana_cost` in the card's
own `activated_abilities` vec against the oracle's `{cost}: effect` lines
and compares the mana-bearing multisets. First run: 31 findings, 27 real,
all fixed in one commit (tests re-pinned to the printed costs):

| Card | Was | Printed |
|---|---|---|
| Elvish Reclaimer | {T}, sac a land | {2}, {T}, sac a land |
| Geier Reach Sanitarium | {1}, {T} | {2}, {T} |
| Yavimaya Elder | {2}{G}, sac | {2}, sac |
| Wishclaw Talisman | {T} | {1}, {T} |
| Golgari Grave-Troll | {T}, remove four counters | {1}, remove one counter |
| Pyrite Spellbomb | {T}, sac: 2 damage · {R}, sac: draw | {R}, sac: 2 damage · {1}, sac: draw |
| Soul Snare | {1}, sac | {W}, sac |
| Scrapheap Scrounger | {1} | {1}{B} |
| Oblivion Stone | {10}, {T}, sac | {5}, {T}, sac |
| Kitsa, Otterball Elite | {2}{U}, {T} | {2}, {T} |
| Elvish Clancaller | {3}{G}{G}, {T} | {4}{G}{G}, {T} |
| Earthen Ally | {W}{U}{B}{R}{G} | {2}{W}{U}{B}{R}{G} |
| Dynavolt Tower | {5}, {T}, 5 energy: 4 damage | {T}, 5 energy: 3 damage |
| Woodweaver's Puzzleknot | {2}, sac | {2}{G}, sac |
| Soul-Guide Lantern | a different card (no ETB; repeatable {T}: exile opponents' graveyards; {2},{T},sac: exile all + draw) | ETB exile target graveyard card; {T}, sac: exile opponents' graveyards; {1}, {T}, sac: draw |
| Cankerbloom | {G}, sac | {1}, sac |
| Hedron Archive | {T}, sac: draw two | {2}, {T}, sac: draw two |
| Birthing Pod | {1}{G}, {T}, sac | {1}{G/P}, {T}, sac (the engine pays Phyrexian mana) |
| Pteramander | {7}: adapt 4 | {7}{U}: adapt 4 |
| Haywire Mite | {2}, sac | {G}, sac |
| Spike Feeder | the {2} counter-move ability absent | both printed abilities |
| Frenzied Arynx | {3}{R}{G} | {4}{R}{G} |
| Ember Hauler | {2}, sac | {1}, sac |
| Tome of the Infinite | {2}, {T} (a synthesized cantrip under an Alchemy name) | {U}, {T}; the draw stands in for conjure |
| Waker of Waves | {2}{U}{U}, discard | {1}{U}, discard |
| Tome of the Guildpact | an invented "{2}, {T}: draw" under a real name | "whenever you cast a multicolored spell, draw"; {T}: add any colour |
| Wizard's Rockets | {T}, sac: one mana of any colour (free) | {X}, {T}, sac: X mana in any combination — ✅ printed since 2026-10-29 (the auto-tapper names X = 1) |

The four false positives taught the reader its three rules: an
ability-word prefix ("Delirium — {2}{G}{G}: ..") is still an activation
line; a helper call among the vec's elements (`tutor_chain(6, ..)`, a mana
ability) means the card cannot be compared at all — never "the literals
beside it are the whole card"; and `generic_cost_value: Some(..)` is a
value-defined {X} (Bargaining Table). **The lesson is the Manifold Key
one at scale: the cost column was clean on 17,229 cards while the
ability costs had never been read, and 27 of them were wrong — audit the
column nobody has run before trusting the silence of the ones everybody
has.**

### Activation timing — the second column of that shape, the same day: 9 more

The `tim` column reads each literal's `sorcery_speed` / `once_per_turn` /
`IsTurnOf(PlayerRef::You)` condition against the oracle line's "Activate
only as a sorcery" / "only once each turn" / "only during your turn"
(sorcery speed was accepted as the documented model of "only during your
turn" until 2026-09-08). First run 29 rows; five reader rules later, 19 ("only during your upkeep"
is the your-turn rider plus the upkeep step); eighteen real, fixed:

| Card | Was | Printed |
|---|---|---|
| Pernicious Deed, Domesticated Hydra (monstrosity), Soul Conduit, Clattering Augur, Llanowar Greenwidow, Relentless X-ATM092, Scrapheap Scrounger, Geier Reach Sanitarium's loot | `sorcery_speed: true` | no timing rider — the gate was invented, each card was weaker than printed |
| Essence Anchor | the graveyard gate only | "only during your turn and only if …" — `All([IsTurnOf(You), ..])` |
| Amulet of Quoz, Hammer of Bogardan, Hell's Caretaker, Undead Gladiator, Necrosavant, Eternal Dragon, Nim Devourer, Grim Reminder | `CurrentStepIs(Upkeep)` alone — activatable in the *opponent's* upkeep, so a graveyard recursion ran twice a turn cycle | "only during your upkeep" — `All([IsTurnOf(You), CurrentStepIs(Upkeep)])`, the `atq.rs` `upkeep_only()` shape |
| Mtenda Griffin | no gate at all | the same |
| Stern Marshal, Rag Man | `sorcery_speed: true` (the documented approximation) | "only during your turn, before attackers are declared" — `All([IsTurnOf(You), Any([CurrentStepIs(Upkeep / Draw / PreCombatMain / BeginCombat)])])` (2026-09-08) |
| Wishclaw Talisman, June Bounty Hunter, Professor Zei, Path to Redemption, Bitter Work, Nebuchadnezzar | `sorcery_speed: true` (the same approximation: it refused your own upkeep and beginning of combat) | "only during your turn" — `IsTurnOf(You)` (2026-09-08) |
| Vampire Bats | `once_per_turn` | "no more than twice each turn" — `max_activations_per_turn: Some(2)` (2026-09-08; the reader learned `twice`) |

No residue: the sorcery-for-your-turn leniency left the reader with the
last eight (2026-09-08), so a future `sorcery_speed` on a printed "only
during your turn" is a row, not a model. The reader's rules:
the rider is a substring, not a sentence ("and only once each turn"); an
`IsTurnOf(You)` nested in an `All(..)` is the rider and a
`Not(IsTurnOf(You))` is the opposite one (Maddening Imp); a step list that
merely includes the upkeep (Cao Cao's "before attackers are declared") is
not an upkeep rider; and the old "activate only as a sorcery" printings
(Soul Conduit) are not the current oracle.

### The `{T}` and "Sacrifice this" halves of a cost — the third column, ten more

`tapsac` reads each literal's `tap_cost` / `sac_cost` (a self-sacrifice
spelled as the effect's first `Move { This -> Graveyard }` counts) against
the `{T}` and "Sacrifice this …" halves of the oracle's cost line (or a
sacrifice that *is* the effect's first sentence — Hopeful Vigil's shape).
First run 23 rows; three reader rules later, 13; ten real, fixed:

| Card | Was | Printed |
|---|---|---|
| Black Lotus | `{T}`: three mana, **and it stayed** | `{T}, Sacrifice this artifact` |
| Path to Redemption | `{5}`: exile the enchanted creature, Aura stays | `{5}, Sacrifice this Aura` |
| Scavenging Ooze, Spectral Sailor, Timmerian Fiends | a `{T}` the oracle never prints — one activation a turn | `{G}` / `{3}{U}` / `{B}{B}{B}, sacrifice` |
| Knight of the Reliquary | `sac_cost: true` — **the Knight sacrificed itself** to tutor (its doc said so) | `sac_other_filter` on Forest ∨ Plains |
| Throne of Geth | always sacrificed itself | "Sacrifice an artifact" — another artifact (`sac_other_filter` cannot name the source; a Throne alone still cannot proliferate) |
| Relic of Progenitus, Metallurgic Summonings, Zombie Assassin | "Exile this" costs spelled as `sac_cost` — the card went to the graveyard | `exile_self_cost: true` |

Residue, three rows: Bronze Tablet (an ante card, the self-exile folded
into a sacrifice by its doc's own choice), Gemstone Mine (the sacrifice is
the conditional last step of the effect, printed that way) and Tomb of
Urami ("Sacrifice all lands you control" includes itself).

### Loyalty costs — the fourth column, one finding

`loy` reads each `LoyaltyAbility` literal's signed `loyalty_cost` (`x_cost`
as −X) and the card's `base_loyalty` against the oracle's "+1:" / "−2:" lines
and `loyalty`. 128 walkers, one wrong: **Karn, the Great Creator** shipped
its two abilities with the signs swapped — a +2 that tutored an artifact
from outside the game every turn and a −1 animation. Fixed 2026-09-07.

### Token P/T — the fifth column, clean

`tok` reads every `TokenDefinition` literal's P/T in a card body against
the oracle's "N/N … token" mentions: 509 cards compared, **zero**
mismatches (2026-09-07). Kept as a standing check; the first reading is
the useful one to know about.

### Trigger events — the sixth column, 2026-09-07: 40 shipped cards on the wrong event, and one engine gap under nine of them

`trig` folds each `TriggeredAbility` literal's `EventKind` (a `StepBegins`
by its step) and each oracle "When / Whenever / At the beginning of" line
(the clause up to its first comma) to a coarse class — etb / dies /
leaves / attacks / combat_damage / deals_damage / upkeep / … — and asks
for a one-to-one assignment when the counts match. A clause naming two
events ("enters or attacks", "at the beginning of your upkeep and
whenever …") is not compared: a card may spell it as one literal or two.
First run 63 rows; the reader's four leniencies (below) took it to 50,
forty real, all fixed in one commit:

| Shape | Cards | Was | Printed |
|---|---|---|---|
| combat-only damage trigger | Hypnotic Specter, Abyssal Specter, Thieving Magpie, Thieving Otter, Looter il-Kor, Goblin Lackey, Warren Instigator, Barbed Shocker, Reef Pirates, Order of Yawgmoth, Ruinous Minotaur, Fungal Shambler, Nafs Asp, Akki Lavarunner, Malcolm (Pirates) | `DealsCombatDamageToPlayer` | "deals damage to a player / an opponent" — `DealsDamageToPlayer` (a Fling, a fight, a ping all count) |
| the same, creature side | Spiritmonger, Vampire Slayer, East-Mark Cavalier | `DealsCombatDamageToCreature` | `DealsDamageToCreature` |
| the same, any recipient | Cecil, Dark Knight | `DealsCombatDamageToPlayer` | "whenever Cecil deals damage" — `DealsDamage` |
| leaves for dies | Rancor, Chromatic Star, Nutrient Block, Hatching Plans, Zoetic Glyph, Demonic Ruckus, Audacity, Reach for the Sky, Fire Nation Warship | `PermanentLeavesBattlefield` | "is put into a graveyard from the battlefield" / "dies" — `PermanentDied` (an exiled Rancor came back to hand) |
| sacrifice for dies | Terrarion, Implement of Combustion, Disciple of the Vault | `PermanentSacrificed` | `PermanentDied` (a Shatter drew nothing / drained nothing) |
| anywhere for dies | Origin Spellbomb, Wizard's Rockets, Glistening Oil, Femeref Enchantress | `PutIntoGraveyard` | `PermanentDied` (a discarded enchantment drew Femeref a card) |
| dies for anywhere | Vulturous Zombie | `CreatureDied` on another creature | "a card is put into an opponent's graveyard from anywhere" — `PutIntoGraveyard` / `OpponentControl` |
| dies for leaves | Thragtusk | `CreatureDied` | "leaves the battlefield" — `PermanentLeavesBattlefield` |
| on-cast for ETB | Quantum Riddler | `SpellCast` / `SelfSource` (a countered Riddler drew; its own test asserted it) | `EntersBattlefield` |
| a different card | Aetherborn Marauder | an energy-gain +2 counters grower | ETB `MoveAllCountersOfKind` from your other permanents |
| approximations retired | Mesmeric Orb (upkeep mill-3), Dramatic Accusation (tap each upkeep) | | `BecomesUntapped` per permanent; ETB tap + `PreventUntap` |

**And the engine gap the Thragtusk fix exposed: `EventKind::
PermanentLeavesBattlefield` matched only `CreatureDied`.** Every "when
this / whenever a … leaves the battlefield" in the catalog (87 literals,
25 on `SelfSource`) fired on a death and slept through a bounce, an exile
or a shuffle — Thought-Knot Seer, Vesperlark, Momo, Twilight Drover
among them, and their tests all killed the permanent. Fixed the same day:
`GameEvent::PermanentLeftBattlefield` at every non-graveyard exit (the
`Move` path, return-to-hand, exile, Glimpse of Tomorrow, meld), the
leaver snapshotted into `died_card_snapshots` so its own trigger fires
off the LKI walk, paired with that event only (a death's copy is still
collected before removal). Tests: `thragtusk_bounced_still_makes_a_beast`,
`twilight_drover_grows_on_token_bounce`, `rancor_exiled_from_the_
battlefield_stays_in_exile`. **The CR 603.6 fan-out list also lacked
`BecomesUntapped`**: an untap step untapping three permanents minted one
Mesmeric Orb trigger.

The reader's leniencies, each a documented engine spelling rather than a
gap: "a source you control deals damage to an opponent" as the
recipient-keyed `PlayerDamaged` / `DealtDamage` with a dealer filter
(Night Dealings, Shocker, Bellowing Fiend, Talon of Pain, Niv-Mizzet ×2,
Fear of Burning Alive, Teysa's "combat damage to you"); "a land enters"
as `LandPlayed`; "becomes blocked by a creature" as a per-blocker
`Blocks` + `TriggerBlocksSource` (Nessian Boar); "cast a spell that
targets" as `BecameTarget` (Gnarlback Rhino). Residue, six rows:
Skizzik models an end-step conditional as the event
that would satisfy it (Whirling Dervish's became the printed end-step
intervening-if, 2026-09-29) (a combat-damage counter; an ETB that schedules
the sacrifice), and four `stx/extras_*` names (Sproutback Trudge,
Cunning Rhetoric, Lorehold Archivist, Lone Rider) are supplemental
inventions that predate the printed cards of the same name. Not
compared, worth knowing: Barret, Avalanche Leader ships the Equipment
token trigger and not the begin-combat attach (the counts differ, so
the column skips it); Frost Titan's targeting tax is absent behind an
"enters or attacks" clause.

### Trigger scopes — the seventh column, the same day: seven more

`scope` reads the same literals for their `EventScope` against the
clause's subject — "this creature" / "you" / "another … you control" /
"… you control" / "an opponent" / "a creature, a player" — and for a step
trigger against "your" / "each" / "each opponent's" (`ActivePlayer` is
"your step" in `fire_step_triggers`, so it is accepted beside `SelfSource`
and `YourControl` there). A literal on `AnyPlayer` narrowed by a filter or
a `dealt_by` is not compared (the filter is usually the scope); one with
`.from_opponent()` reads as the opponent's; `YouAttack` reads as "you" on
either spelling; "a creature you control" accepts `AnotherOfYours` (an
enchantment cannot be its own subject, and that is how the family is
spelled). First run 298 rows, eleven reader rules later 12, nine real (the last two once the `YouAttack` leniency was narrowed):

| Card | Was | Printed |
|---|---|---|
| Ghazbán Ogre | `AnyPlayer` — checked the life leader on every upkeep | "your upkeep" — `SelfSource` |
| Ophiomancer | `YourControl` — a Snake on your upkeep only | "each upkeep" — `AnyPlayer` |
| Zopandrel, Hunger Dominus | `ActivePlayer` — doubled on your combat only, so your blockers never grew | "each combat" — `AnyPlayer` |
| Savai Thundermane | `AnyPlayer` — paid off the opponent's cycling too | "whenever you cycle" — `YourControl` |
| Fecundity | `AnotherOfYours`, you drew | "whenever a creature dies, that creature's controller may draw" — `AnyPlayer`, `ControllerOf(TriggerSource)` draws |
| Quartzwood Crasher | `SelfSource` | "one or more creatures you control with trample" — `YourControl` + a trample filter (per creature) |
| Prosperous Thief | `SelfSource` | "one or more Ninja or Rogue creatures you control" — `YourControl` + the type filter (per creature) |
| Voja, Jaws of the Conclave; Attack-in-the-Box | `YouAttack` / `SelfSource` — fired whenever you attacked with anything | "whenever this creature attacks" — `Attacks` / `SelfSource` (found once the `YouAttack` leniency was narrowed to the "you" scopes) |

Residue, five rows (Whirling Dervish closed 2026-09-29): Skizzik and Lone Rider (the
end-step conditional modelled as the event that satisfies it, the
`trig` residue), Teo, Spirited Glider (`YouAttack` on `SelfSource` with
an `AttackedWithCreatureMatching` predicate — "you attack with a flyer",
read as "you"), and Fatespinner, which the reader learned from
(`.from_opponent()` on `AnyPlayer`). **The two columns together are the
whole trigger header: kind and scope. What neither reads is the
*filter* — "a nontoken creature", "a spell with mana value 3 or less" —
and the effect's own text, which is the eighth column.**

### Trigger filters — the eighth column, the same day: five more, and the reader's limits

`filt` reads the *type words* of a literal's filter — `R::Creature`,
`HasCreatureType(Goblin)`, `NotToken` / `IsToken.negate()`,
`HasKeyword(Flying)`, a colour, an artifact or enchantment subtype, the
`.dealt_by(..)` of a damage kind — against the type words of the clause's
subject ("another nontoken Goblin creature you control", cut before its
object: "… deals combat damage *to a player or planeswalker*"). A
creature-only kind implies "creature" on the code side, a creature type
implies it on both, `CreatureOrArtifactDied` implies both words, a Blood /
Clue / Food is a token by construction. Not read, so not compared: the
source's own type (`SelfSource`), the kinds that carry their subject
(targeting, damage recipients, activations, steps), a filter that is not a
plain `EntityMatches` on the trigger source (a `Not`, a power or mana-value
bound, a `let` helper), and the struct-form `EventSpec { .. }`. First run
573 rows, five reader passes later 12, five real:

| Card | Was | Printed |
|---|---|---|
| Long Feng, Grand Secretariat | `CreatureDied` | "another creature you control or a land you control is put into a graveyard from the battlefield" — `PermanentDied` + creature-or-land |
| Phyrexian Ironworks | `Attacks` / `YourControl` — {E} per attacker | "whenever you attack" — `YouAttack`, once a combat |
| Foundry Street Denizen, Court Street Denizen, Sage's Row Denizen | a colour alone — a red / white / blue enchantment or artifact entering triggered | "another red creature" — the colour *and* `Creature` |

And one the `trig` fixes exposed rather than the column: **the ten
`sword()` Swords fired their rider off the equipped creature** (CR 702.6e's
default), so Sword of War and Peace's burn was the *Looter's* damage — a
second loot, and a colourless burn stopped by protection from red. The
helper sets `triggers_on_equipment` now; Sword of Truth and Justice's
`Selector::This` (the creature, under the old source) became the printed
"target creature you control". `sword_of_war_and_peace_burns_by_hand_and_
gains_life` asserts the single loot. The flag then turned out to be
honoured by two hooks only (ENGINE_BACKLOG, first section): Godsend and a
bestowed Crystalline Nautilus were dead, Kusari-Gama and Impending Doom
dealt their damage as the creature's — all four fixed the same day.
Of the eight other attachments with a damage rider, six print "this
creature deals" and are right as they are.

False positives worth knowing: Valley Mightcaller's Squirrel sat in a
multi-line `HasCreatureType(\n CreatureType::Squirrel,\n)` the first regex
did not span (the reader now does; nothing was wrong with the card), and
every `[]` against `["creature"]` on the first run was a `SelfSource`
"When this creature enters" whose subject is the source itself. **The next reader rules, taken the same day:** a line break after
`.with_filter(` (234 literals the regex had never opened — the single
largest gap), `CastSpellMatches(R)` as the cast spell's own filter,
`Not(Box::new(R::Creature))` as "noncreature" (the six type words with a
printed negation), `PowerAtLeast(n)` / `ManaValueAtMost(n)` as
"power n or greater" / "mana value n or less" words, multi-line
`HasCardType(\n ..)` calls, land types (a Mountain is a land), "historic"
(artifact / legendary / Saga), and `SpellTargetsMatching` /
`IsHostOfSource` as unreadable, a `Predicate::All` whose other members are
bare conditions (`IsTurnOf`, `SpellsCastThisTurnEquals`, ...) as its one
entity member, and a file-local `fn helper() -> Predicate` as its body.
Literals the column names: **362 -> 602** (2,591 -> 2,840 compared);
mismatches **0** — the nine the wider read first raised were all reader
limits (each is one of the rules above), and the last two rules added
three literals, so the reader is at its useful floor. Still unread: a
`let` filter inside the card body (Valley Questcaller's `typal()`), a
`Not` of a creature type.

### Amounts — the ninth column, 2026-09-10: 39 shipped cards at the wrong number

`num` reads the amounts an ability prints — "deals 3 damage", "draw two
cards", "mana value 3 or less", "+2/+0", "surveil 3" — against the integer
literals in the code for the same ability (its `ActivatedAbility` /
`TriggeredAbility` literal, or a spell's whole body: `Const(n)`, `ONE` /
`ZERO`, a `PlusOnePlusOne`, a trailing `slot: 1`, the numbers of a
`let`-bound helper), one-to-one under the same count gate, and lists an
oracle amount the code carries nowhere. Mana symbols, a token's or a
face's N/N, loyalty and keyword lines ("Suspend 4"), reminder and quoted
text, "Choose two", the ability-word thresholds a predicate helper meets
(delirium's four card types, metalcraft's three artifacts, coven, ferocious,
fateful hour, corrupted, celebration) and the alternative / additional-cost
sentences a spell's other fields carry are not amounts. First run 838
rows; twelve reader rules later 64, thirty-nine real, all fixed in one
commit (tests re-pinned to the printed numbers; every doc comment had
agreed with the wrong code, so `audit_doc_drift` could not see any):

| Card | Was | Printed |
|---|---|---|
| Languish | -2/-2 | -4/-4 |
| Searing Wind | 5 damage | 10 damage |
| Built to Smash | +2/+2 | +3/+3 |
| Frenzied Arynx | +2/+2 | +3/+0 |
| Blazing Rootwalla | `{1}{R}`: +1/+1 | `{R}`: +2/+0 |
| Foundry Street Denizen, Violent Outburst | +1/+1 | +1/+0 |
| Abrupt Decay | mana value 2 or less | 3 or less |
| Unearth | mana value 1 or less | 3 or less |
| Sigardian Savior | mana value 3 or less | 2 or less |
| Pest Control | mana value ≤ the converged value (a synthesised text) | 1 or less |
| Fatal Push | 2 or less, always | 4 or less under Revolt (`RevoltActive`, checked on resolution) |
| Wrangle | any creature | power 4 or less |
| General Kudro of Drannith | destroy any creature | power 4 or greater |
| Ezuri, Claw of Progress | power 1 or less | power 2 or less |
| Jaya's Greeting | scry 2 | scry 1 |
| Grim Flayer | surveil 2 | surveil 3 |
| Oust | 5 life | 3 life |
| Chevill, Bane of Monsters | gain 1 | gain 3 |
| Sweettooth Witch | loses 3 | loses 2 |
| Mine Collapse | 4 damage to any target | 5 damage to target creature or planeswalker |
| Searing Blaze | 3 and 3, always | 1 and 1; 3 and 3 under landfall (`LandsPlayedThisTurn`) |
| Gather the Townsfolk | two tokens | five at 5 or less life (`PlayerLifeAtMost`) |
| Secrets of the Key | one Clue | two if cast from a graveyard (`CastFromGraveyard`) |
| Blasphemous Edict | each player sacrifices one | thirteen |
| Genemorph Imago | 5/5 at six lands | 6/6 |
| Mercurial Chemister | draw a card | draw two |
| Brood Butcher | -1/-1 | -2/-2 |
| Sacred Fire | 3 damage, 3 life, Flashback `{5}{R}{W}` | 2, 2, `{4}{R}{W}` |
| Sparkmage Apprentice | 2 damage | 1 damage |
| Magmatic Sinkhole | surveil 2, 4 damage | 5 damage |
| Reduce to Ashes | 4 damage (creature or planeswalker) | 5 damage to target creature |
| Acolyte of Affliction | each player mills three, return from any graveyard | you mill two, return from yours |
| Shore Up | untap any permanent + hexproof | +1/+1, hexproof, untap — a creature you control |
| Inkfathom Divers | ETB: opponent discards a nonland card (a different card) | look at the top four, put them back (`LookAtTop`) |
| Waker of Waves | `{1}{U}`, exile from graveyard: +5/+5 trample (a different card) | `{1}{U}`, discard this: look at two, one to hand, one to graveyard (the channel shape) |
| Step Through | tutor an instant or sorcery (a different card) | return two target creatures; Wizardcycling `{2}` |
| Inscription of Ruin | mode 2 any creature card to hand; mode 3 any creature | mana value 2 or less to the battlefield; mana value 3 or less |
| Cathartic Pyre | discard any number; target creature | up to two; creature or planeswalker |

**Triaged again 2026-09-12 at 83 rows (from 92: see below), and the sample
held — ten rows across five columns, zero defects.** The ones worth naming so
nobody walks them a third time, each a READER limit rather than a card:

| Row | Why it is not a finding |
|---|---|
| Valakut, the Molten Pinnacle `num` 6 vs 5 | The predicate counts the ENTRANT plus five others, which is six; the comment beside it says so. |
| Springbloom Druid `num` 1 vs 2 | The `1` is the SACRIFICE count ("sacrifice a land"); the two searches are `Seq[fetch(), fetch()]`. |
| Earwig Squad `num` 0 vs 3 | Three `pick()` calls of a local closure, and the `0` the reader shows is a `PlayerRef::Target(0)` slot index. |
| Skizzik `trig` etb vs end | `EntersBattlefield` + `Not(SpellWasKicked)` wrapping an `AtNextEndStep { Sacrifice }` — the delayed trigger IS the end step, and kicked-ness cannot change after ETB. |
| Telling Time `num` 2,1 vs 3 | "Look at three, one to hand / top / bottom" modelled as Scry 2 + Draw 1 — an approximation, since scry cannot bottom a card. |

`audit_oracle_verbs.py` got the same treatment the same day and came out the
same way: **56 rows from 172, and a ten-row sample found zero code defects.**
Two reader rows are closed for good rather than listed — `rest_to_graveyard`
IS a mill, which is how Six spells "mill three cards, you may put a land from
among them into your hand" (and Six's OWN second ability retraces out of the
graveyard it fills, so reading it as a non-mill said the card's two halves did
not connect); that took the `mill` column 4 -> 3. The rest is three shapes,
each named in that script's docstring: a REPLACEMENT effect the oracle words
with the verb, a mechanic with no primitive, and the deathtouch family.

The closure case is now READ rather than listed: `code_numbers` counts any
zero-argument call that repeats, not only `search`-shaped ones (Earwig Squad's
`pick()`, Springbloom Druid's `fetch()`). That took `num` 92 -> 83, `stat`
14 -> 12 and `kw` 18 -> 17, and it can only remove rows — an extra number in
the code is never a finding — so its cost is a masked row, not a false one.
`python3 scripts/audit_catalog_stats.py all` prints every set's detail in ONE
catalog scan; triaging a column used to be thirty scans of the same data.

Six of the seven `decks/modern.rs` rows are training-pool cards, which is
the column's point: the pool the nets learn on had a -2/-2 Languish and a
half-strength Searing Wind. Residue, 64 rows, each an approximation a doc
already names (multi-target "up to N" collapsed to one slot: ~~Conduct
Electricity, Trick Shot~~ (slot groups, `01WwQaPJ`), ~~Heartless Act's counters~~ (`only_helpful`, `01M3bbRx`), Secret Tunnel; a
missing conditional branch: Systems Override's counters, The Necrobloom's
Zombie, Agency Coroner's suspect, Paroxysm's pump; an alternative or
additional cost the engine does not price: Bitter Triumph, Redirect
Lightning, Blasphemous Edict's `{B}`; a custom `Effect` whose numbers are
in the engine: Allure of the Unknown, Guild Feud, Mana Clash) or an
oracle number that is not an amount the reader can tell apart ("two or
more" ties, "second from the top", Black Waltz No. 3's own name). The
reader's rules: a helper defined in another file hides its numbers, so a
row on `fetch()` / `sweep()` is not a finding until the helper is read; a
number in a `//` comment is not code; and the direction is one-way — an
extra number in the code (a `slot: 1`, a `max_targets`) is never a row.

The same reader over a permanent's static lines (`stat`: the whole body
minus its own cost, name and P/T, so an `equipped_bonus` and a local
binding count) found two more the same day — Carapace Forger's +1/+1 for
a printed +2/+2 and Elvish Reclaimer's "seven or more cards" for "three or
more land cards" — and left 16 rows, every one a custom `StaticEffect`
whose amount lives in the engine (`ControllerDrawsDoubled`,
`ManaProductionTripled`, `NonAuraEnchantmentsAreCreatures { requires_five }`)
or, in one case, a different shape: The Water Crystal's "mill that many
plus four" ships as `OpponentMillDoubled`.

And a `mana` column the same day — an `AddMana` activation literal's
payload (a `Colors` list or its repeat form, `Colorless` / `OfColor` /
`AnyOneColor` at a literal count, the parts of a `Seq` summed, an `If`'s
`else_` branch as the printed base) against the oracle's "{cost}: Add .."
lines as symbol multisets — read 8 rows, two real: The Great Henge tapped
for `{G}` (printed `{G}{G}`) and Cultivator Drone for `{C}{C}` (printed
`{C}`). Zero residue; mana abilities built through a helper are not
compared.

Two more classes the same day, both now suite gates rather than columns
(ENGINE_BACKLOG "FIXED 2026-09-10"): nine Auras whose trigger sat under
`EnchantedBySource` on an event the dispatcher never matches for that
scope (a step, a targeting, a graveyard kind), and six Auras whose entry
half was spelled after the attach in `effect:`, which the cast path never
runs. Fifteen cards, one test each, no residue.

### Helper-built abilities — the tenth read, 2026-09-10: every column opened, 13 shipped cards

Every activation and trigger column above answered `None` for a card whose
`activated_abilities` / `triggered_abilities` vec held a helper call
(`tap_add(Color::Green)`, `self_pump(cost(&[r()]), 1, 0)`, `upkeep(effect)`,
`returns_to_hand()`) — 85 file-local `fn .. -> ActivatedAbility` helpers plus
the four in `sets/mod.rs`, 132 `fn .. -> TriggeredAbility`, ~2,000 abilities
unread by nine columns. `inline_ability_helpers` now rewrites the call into
the helper's own literal with the arguments (and its simple `let` bindings)
substituted for the parameters, a spread (`ActivatedAbility { tap_cost:
false, ..helper(args) }`) keeping its override fields in front; and the
number readers consult a per-file table of `fn .. -> Effect / StaticAbility
/ Predicate / Value / SelectionRequirement` helpers, so `effect: drain_two()`
carries the helper's amounts. Three reader rules fell out: a Class's
"{cost}: Level N" is sorcery-speed (CR 716.2b, the rider is in stripped
reminder text) and its N is the level index, not an amount; `search()` as a
called closure counts like `search_*()`. Thirteen real rows, one commit each
class:

| Card | Was | Printed |
|---|---|---|
| Stonework Packbeast (training pool) | `{T}: Add one mana of any color` | `{2}:` — no tap; a sick Packbeast activates |
| Eiganjo Castle | `{2}{W}`, no tap: prevent 2 to a legend | `{W}, {T}` |
| Engineered Explosives (training pool) | `{2}, {T}, Sacrifice` (the Ratchet Bomb helper's tap) | `{2}, Sacrifice` |
| Brilliant Halo, Despondency, Launch, Fiery Mantle, Fortitude | return to hand on any leave (exile included) | "put into a graveyard from the battlefield" (`PermanentDied`) |
| Matsu-Tribe Sniper | lock on combat damage (the Snake helper) | "deals damage to a creature" — its own `{T}` ping locks |
| Traveling Plague | return when the host dies | "leaves the battlefield" — a bounce too (the dispatcher's `EnchantedBySource` arm now serves the non-death leave; the pool was "a creature they don't control", now any creature) |
| Ball Lightning, Spark Elemental, Groundbreaker, Hellspark Elemental, Lightning Skelemental (training pool) | sacrificed at your end step | "the end step" — an opponent's too |
| Bloodchief Ascension | quest counter on any life lost | "2 or more" (`ValueAtLeast(LifeLostThisTurn(EachOpponent), 2)`) |

Two more the same effect re-shape found: Steam Vines' "that player attaches
Steam Vines to a land of their choice" was dead (`ReturnSelfAttachedToChoiceOf`
wanted a graveyard source and a creature pool; it now takes a `filter` and
re-attaches an on-battlefield source in place), and The Water Crystal's
"mill that many plus four" shipped as mill-doubling (`OpponentMillExtra {
count }`, the `stat` column's one non-custom row). Residue after the read:
`trig` 7 / `scope` 3 the documented rows (Whirling Dervish, Skizzik), `num`
76 / `stat` 15 (Thunderscape Master and Blighted Woodland's helper rows
closed; the rest the approximations above), `T-sac` 3, `abil` 1 (Wizard's
Rockets' `{X}`).

### The other cost halves — the eleventh column (`ocost`), 2026-09-10: seven cards paying a cost as an effect

`T/sac` reads `{T}` and the self-sacrifice; every other half of an activation's
cost had no column — "Sacrifice a creature", "Discard a card", "Pay 2 life",
"Pay {E}", "Exile a card from your graveyard", "Remove a counter", "Tap an
untapped Elf", "Return this to its owner's hand", "Put a page counter on
this". `ocost` reads them as a flag set per activation line against the
literal's cost fields grouped by what they pay (`sac_other_filter` and its
kin, `discard_*`, `life_cost`, `energy_cost`, `exile_*`, `remove_counter_*`,
`tap_*`, `bounce_*`, `add_counter_cost`, ...). Two catalog shapes are read
as the cost they stand for: `SacrificeAndRemember` as the effect's first
step (the bot's legality reads it as a cost) and a `RemoveCounter` behind a
`condition: ValueAtLeast(CountersOn ..)` gate, or one that removes them all.
First run 51 rows; the reader rules above and seven real ones, every one a
cost paid inside the effect — so the ability could be activated with nothing
to pay and, for the bot, was priced as free:

| Card | Was | Printed |
|---|---|---|
| Wishclaw Talisman (training pool) | `RemoveCounter` as the first effect step, ungated — a tutor with no wish counters | "Remove a wish counter" (`remove_counter_cost`) |
| Dynavolt Tower | a pay-if-able `PayEnergy { 5 }` inside the effect | "Pay {E}{E}{E}{E}{E}" (`energy_cost: 5`) |
| Prismatic Vista | "Pay 1 life" as a `LoseLife` step | `life_cost: 1` |
| Bomat Courier, Connecting the Dots | "Discard your hand" as a `Discard` step | `discard_hand_cost` |
| Mazemind Tome, Bloodletter Quill | the page / blood counter added inside the effect | `add_counter_cost` |

And the timing column grew an `if` flag the same day: a printed "Activate
only if .." against a `condition:` naming anything beyond the turn / step
riders (one-way — a code gate with no printed one is the catalog's device
for a Class level or a counter to remove; "only if this card is in your
graveyard" is `from_graveyard`). Two cards had no gate at all: Dreadlight
Monstrosity's "only if you own a card in exile" and Skitterskin's "only if
you control another colorless creature" — both `SelectorCountAtLeast`. Zero
residue.

Residue of `ocost`, five rows, each a shape the engine cannot spell yet or a
deliberate order: Krasis Incubation (its "Return this Aura to its owner's hand" cost
runs last, so "enchanted creature" still has a referent — no LKI for a
bounced Aura's host), Legion's Initiative (the self-exile after the
`ExileLinked` that must see the source), Mercurial Chemister ("equal to the
discarded card's mana value" — no value reads a cost's discard), Geometric
Nexus (the removed count through a helper), Anthroplasm (the reverse: its
"remove all counters" *effect* spelled as `remove_all_counters_cost`, the
same outcome).

### Additional cast costs — the twelfth column (`addl`), 2026-09-10: three cards, one of them the reverse defect

"As an additional cost to cast this spell, sacrifice a creature / discard a
card / pay N life / reveal .." against the definition's `additional_cast_cost`
variants (and `additional_cost_pay_x_life`), as kinds. The catalog's usual
shape for the mandatory ones is the effect's first step — a
`SacrificeAndRemember` / `SacrificeAnyNumber` (the bot's legality reads the
sacrifice as a cost), a `Sacrifice` / `Discard` / `LoseLife` of yours, a
`ChooseMode` whose every arm is such a step — and the reader takes it as
the cost when the oracle prints one (one-way: Vampiric Tutor's "you lose 2
life" is its effect). The optional forms ("you may ..") and a helper-built
spell (`instant("Name", ..)`) are unread; "pay {4} or sacrifice" is
`SacrificeOrPay`, "sacrifice an artifact or discard a card" is met by either
kind, and the older "Sacrifice an artifact. If you do .." wording (Transmute
Artifact, Dredge) reads as a sacrifice. First run 62 rows; three real:

| Card | Was | Printed |
|---|---|---|
| Morkrut Behemoth | no additional cost | "sacrifice a creature or pay {1}{B}" (`SacrificeOrPay`; the variant prices the alternative in generic mana, so {2}) |
| Caustic Exhale | no additional cost | "behold a Dragon or pay {1}" (`RevealFromHandOrPay`; the "choose a Dragon you control" half of behold is not read) |
| Mine Collapse (training pool) | a MANDATORY Mountain sacrifice on top of {3}{R}, then the alternative dropped | "you may sacrifice a Mountain rather than pay this spell's mana cost" on your turn — `AlternativeCost { sacrifice_permanents, your_turn_only }` (the gate added 2026-09-10, third run) |

Redirect Lightning's "pay 5 life or pay {2}" ✅ 2026-10-08
(`AdditionalCastCost::PayLifeOrPay`).

### The card's OWN cost and body — the fourteenth column (`scripts/audit_printed_body.py`), 2026-09-11: seven free spells

Every column above reads an *ability's* field. This one reads the field they
all assume is right: the card's printed mana cost and P/T, straight off the
`CardDefinition` literal rather than off the doc comment (which is what
`audit_doc_drift.py` compares, so it audits only the factories whose doc states
a cost in the `Name — {2}{R} 3/3` shape).

**10,268 factories compared against the oracle: 0 wrong P/T, 1 wrong cost, and
7 cards missing `no_mana_cost`** — the last of which is the finding. CR 202.1b
/ 601.3e: a card that prints NO mana cost cannot be cast by paying one, and the
engine enforces exactly that off `CardDefinition.no_mana_cost`. Seven shipped
cards did not set it, and four of them had no `cost:` either, so they were
castable from hand **for free**:

| card | was | is |
| --- | --- | --- |
| Profane Tutor | free "search your library for a card, put it into your hand" | suspend only |
| Mox Tantalite | a free Mox | suspend only |
| Sol Talisman | a free `{T}: add {C}{C}` rock | suspend only |
| Ragnarok, Divine Deliverance | a free 7/6 vigilance menace trample reach haste | uncastable |
| Asmoranomardicadaistinaculdacar | free from hand (its `{B/R}` alternative cost was already modelled) | the alternative cost only |
| Urza, Planeswalker | a free 7-loyalty walker | uncastable (a meld result) |
| Resurgent Belief | castable for an invented `{3}{W}` | suspend only |

**The TYPE LINE, read in the same pass, is CLEAN: 0 of 10,270.** Card types
and supertypes against the oracle's type line — a Sorcery shipped as an Instant
is castable at instant speed, a missing Legendary is a legend rule that never
fires, and neither was audited anywhere. The first cut said 49 wrong, all
missing `Legendary`, clustered in two files: `Sup::Legendary` behind an alias
import and `supertypes: legendary()` behind a helper. The matcher is
alias- and helper-tolerant now, and it is proved by injection rather than by
its own zero — breaking Karn, Scion of Urza to `{3}`, `Creature`, no supertype
reports all three rows.

**Coverage, re-read 2026-09-12: 16,439 factories priced, from 10,270** (+60 %),
and **16,185 type lines read, from 10,409** — with 0 findings on all four
columns. The "219 real spells over ~40 bespoke helpers" above was a mis-read of
the script's own skip counts. Four holes, none of them the helper signatures:
the **`..base` struct-update form** (2,890 factories, 13 % of the catalog: a
literal, so the helper fallback never ran, and no `cost:` in it, so the literal
read found nothing); the **pure-helper factory** (2,392 more —
`pub fn x() -> CardDefinition { sorcery("Name", cost(..), effect) }` was dropped
as `noname` *before* the name was resolved, which also made every
`body is None` branch in the file dead code); **`crate::mana::`-qualified
symbols and multi-line costs**; and **factories named only by their `pub fn`**.

**The two readings that matter more than the coverage:**

*One: the cost is not the argument next to the name.* It is for
`creature("Name", cost(&[r()]), ..)` and it is NOT for `skullbomb("Name",
mode_cost, ..)`, `keeper("Name", activation_cost, ..)` or `shard("Name",
ability_cost, ..)`, whose helpers print a cost of their own — **fourteen cards a
name-anchored reader priced at their ability's cost**, every one a false
positive it would have reported forever. `resolve_helper_cost` follows the
helper: its own `cost:`, a bare parameter bound back to the caller's argument,
its own base up to five links, and a skip (never a report) when the expression
is anything else.

*Two: the type line is rarely in the literal.* Three idioms carry it elsewhere —
the base-struct form, the **wrapper** form (`legend(CardDefinition { .. })` over
`fn legend(mut def) { def.supertypes = ..; def }`, six Invasion legends), and a
helper whose own base is another helper. A chain the reader cannot follow is
skipped, not reported: the first cut reported 77 rows of "supertypes: (none)"
and every one was `..legend(..)` supplying what the literal never claimed.

**Six injections, one per idiom**, and one of them **silently passed at first**:
`legend` is defined in five files with three different shapes, the resolver took
whichever came first, and an assigning wrapper was reading a base-struct one's
supertypes — so the column would have read clean forever whatever those six
cards said. Same-file first now. A gate that cannot fail is worse than no gate,
and the only thing that catches one is breaking it per idiom rather than once
per column.

What is left: `nocache` (3,778 synthesized names), `nonliteral` (972 — a cost
chain built from a local binding), `notyped` (254), `noname` (178), and the
deliberate faces / split / star / not-a-spell skips.

Resurgent Belief also carried a `flashback_additional_cost` for a Flashback it
does not have — the comment two lines below it already said "Suspend 2—{1}{W},
not Flashback".

Two rules the pass yields. *One:* **a filter that reads a factory BLOCK reads
the tokens inside it.** The first run of this audit reported 117 wrong P/T;
every one was a `let token = TokenDefinition { power: 1, .. }` defined above
the card, at the same indent. Brace-match the `CardDefinition` literal and read
its own depth-1 fields — and count braces only, skipping string literals, or
`cost(&[generic(4), w()])` and every prompt containing `{2}` break it in turn.
*Two:* **an empty `ManaCost` is two different cards.** Ornithopter prints `{0}`
and is castable; Living End prints nothing and is not. The engine tells them
apart by `no_mana_cost` alone, so the suite's regression for this class is a
by-name table (`structural_audit::every_card_that_prints_no_mana_cost_says_so`,
17 cards) plus one behaviour test that the cast path refuses; the oracle-backed
finder stays in `scripts/`.

### The shortcut helpers and the ability count — the thirteenth column (`cnt`), 2026-09-10: 61 shipped cards

Two reads the same day. First, the base crate's `effect::shortcut` module
(~140 `fn .. -> ActivatedAbility / TriggeredAbility` helpers every set file
calls: `etb`, `landfall`, `monstrosity`, `on_you_attack`, `boast`, ..) had
stayed outside the inliner's table — the last "helper in another file" —
so every column above now opens those too (the inliner takes the outermost
literal; a tribute's granted trigger is nested inside its own). Forecast
and Boast riders read off their reminder text, and a trigger's
self-reference ("this creature", the card's name) is not a subject filter.
That opened `abil` 1 → 6, `timing` 0 → 13, `trig` 7 → 28, `scope` 3 → 9,
`num` 76 → 101: 24 shipped cards, every one a helper-built ability nobody
had read — four monstrosity prices (Ember Swallower `{5}{R}{R}`, Arbor
Colossus `{3}{G}{G}{G}`, Ill-Tempered Cyclops and Stormbreath Dragon
Monstrosity 3), eleven wrong events (Emperor's Vanguard and Lightning
Skelemental on combat damage to a player, Murktide Regent on an instant or
sorcery leaving the graveyard, Marsh Viper on any damage, Titania on a land
of yours dying, Nulldrifter on the cast, Hopeful Vigil / Hopeless Nightmare
/ Bitter Chill on `PermanentDied`, Duskworker and Groffskithur on becoming
blocked, Custodi Lich on becoming the monarch, Mai on a player's noncreature
cast, Wandering Mind's ETB dig, Cunning Rhetoric on an opponent's attack,
Spitfire Lagac's landfall), four "whenever you attack" scopes that fired for
any attacker (`on_you_attack` on Seedpod Squire, Waterspout Warden, Finneas,
Civic Gardener), two dropped halves (Consul's Lieutenant's renowned pump,
Shrieking Grotesque's ETB discard) and Skeletal Kathari's `{B}, sacrifice a
creature: regenerate` (it shipped with unearth).

Second, the column every one above skips behind its `len(..) == len(..)`
gate: `cnt` counts the `ActivatedAbility { .. }` / `TriggeredAbility { .. }`
literals the code spells (the card's own and the ones its Aura / Equipment
grants; a keyword-modelled Regenerate, a `discard_activated`, a spell's
`DelayUntil` count) against the oracle's `{cost}:` lines and its When /
Whenever / At lines, one-way — fewer is a dropped ability, more is the
engine's shape (a static modelled as a trigger, a reminder-text mana
ability). Trigger lines the engine models as something else are not
counted: "is tapped for mana" (a static), the O-Ring return (the linked
exile), a pain land's tap damage, ward's counter-unless, the basilisk's
end-of-combat destroy, "When you cycle". A helper the table cannot open
(`cycling_land("..")`, a `..helper(..)` spread) is unread. First run 197
rows; 37 real, two commits:

| Shape | Cards |
|---|---|
| Firebreathing dropped | Shivan Dragon, Inferno Titan, Furnace Hellkite; Inkrise Infiltrator's `{3}{B}: +2/+2` |
| Regenerate dropped | Servant of Tymaret, Twisted Abomination, Asphodel Wanderer, Experiment One (two counters) |
| A second activation dropped | Cranial Plating's `{B}{B}` attach, Jack-o'-Lantern's graveyard mana, Seasoned Pyromancer's graveyard Elementals, Cloudgoat Ranger's tap-three-Kithkin flight, Triskaidekaphile's `{3}{U}` draw, Ride the Shoopuf's 7/7 Beast, Aquastrand Spider's reach and Plaxcaster Frogling's shroud grants |
| Wrong amount under the count | Bogardan Hellkite 5 (was 4), Rathi Dragon two Mountains (was one), Festival Crasher +2/+0 until end of turn (was a permanent counter) |
| A trigger dropped | Omnath, Locus of Rage's "another Elemental dies" burn; Bria's "noncreature cast: unblockable"; Prison Realm's scry; Sengir Autocrat's Serfs leaving with it; Lonis's counter per Clue sacrificed; Krydle's `{2}` on attack; Hexgold Slith's two energy on attack; Court Hussar's sacrifice without white; Lembas shuffling back; Pit Scorpion's Poisonous 1 (a keyword the engine has) |
| A trigger stood in for by an invented activation | Nihil / Horizon Spellbomb (training pool): "when this dies, you may pay {B}/{G}: draw" shipped as a made-up `{W}, Sacrifice: draw` |
| The wrong ability word | Doomskar Titan: `boast` for an ETB |

Two false rows taught the reader: Magma Opus's discard is `discard_activated`
(counted now), Secluded Starforge's robot was already there. The Horizon
Spellbomb fix surfaced an engine order: a sacrifice-cost source's own dies
trigger stacked below the ability — fixed the same day (ENGINE_BACKLOG
"FIXED 2026-09-10 (fourth find)").

The 130 rows left were then read one by one (two read-only passes, the
training pool and the rest). 47 are the engine's shape — a static
(`ExtraManaOnLandTap` for Mana Flare / Heartbeat of Spring / Nirkana
Revenant / Dictate of Karametra, `CollectsLeaverCounters` for The Ozolith,
`PumpSelfIf` for Borderland Marauder and Flummoxed Cyclops, `EtbTriggerTax`
for Strict Proctor), a keyword (Deathtouch for Stinkweed Imp, `SuspendAccelerant` for Deep-Sea
Kraken, Multikicker for Intrepid Adversary), a field (`sacrifice_when` for
Last Laugh, `flip_when_has_keyword` for Student of Elements,
`enters_under_opponent_control` for Akroan Horse, `exile_countdown` for All
Hallow's Eve, `sacrifice_and_burn_when_stolen` for Bronze Bombshell), a
delayed effect (`OnMatchingAttacksThisTurn`, `AtEachCombatThisTurn`,
`CreaturesYouControlEnteringThisTurn`, `HauntCreature`,
`ExileSpellWithDelayCounters`), or the state trigger folded into the
activation that feeds it (Mazemind Tome, Dark Depths, Bomb Squad, Porphyry
Nodes). 12 are the reader's: an `on_attack(..)` inside `equipped_bonus`
or a `StationBand`, a `landfall(..)` — helper calls the count does not
open. 71 were real, 63 with the ability wholly absent; 22 of those had
every primitive in hand and were fixed the same day (Murderous Rider,
Queen of Ice, Merchant of the Vale, Keys to the House, Overgrown Zealot,
Chancellor of the Annex, Dai Li Agents, Badgermole Cub, Aang, Emrakul the
Promised End, Shielded by Faith; Voltaic Brawler, Aetherstream Leopard,
Riparian Tiger, Floodgate, Galvanic Iteration, Brimaz, Haywire Mite,
Glacier Godmaw, Pain for All, Cavalry Pegasus, Cytoplast Root-Kin —
`cnt` 130 → 109), then ten more that were one card-file edit each
(Questing Beast's planeswalker damage, Edgar's Awakening's
`discard_activated` — its body is targetless, so the creature card is
chosen at resolution — Earthbender Ascension's landfall counters,
Buzzard-Wasp Colony's counter move, Reaper of Night's hand-size gate and
Harvest Fear's chosen discard, Shadow Urchin's exile-per-counter impulse
with the engine's end-of-your-next-turn window, Earthrumbler, Alacrian
Armory, Golos's free-play, Inkfathom Witch's 4/1 in place of an unprinted
ETB discard — `cnt` 109 → 100). Two engine lines fell out: a blocks
trigger's `CreateTokenBlocking` reads the blocked attacker (Brimaz), and
`MoveAllCounters` reads its source as a card ref, since a dies trigger
binds the dying creature that way (the LKI branch already found it). The
full suite also caught Mai, Scornful Striker draining the *opponent*
(`TriggerEventPlayer` is unbound on a SpellCast; `Triggerer` is the
caster) — a trigger-column fix whose test had never run. Still one edit
each: Touch the Spirit Realm's channel (the discard-activated body cannot
target), Gideon's Company once a name-prefix requirement exists, Hauntwoods
Shrieker, Eladamri, Pinnacle Starcage, ~~Rydia~~, ~~Diviner's Wand~~, Urborg
Panther, Tephraderm's spell half, ~~Skophos Maze-Warden~~, ~~Shieldmage
Elder~~, ~~Iroh~~ (`01WwQaPJ`), Lumbering Laundry (Tephraderm's spell half, Hauntwoods Shrieker's
reveal and Severance Priest's Spirit shipped the same day, with
`LastDamagerOf` returning a departed damager — a resolved spell — as a card
ref; Barret's begin-combat attach the day after). **Three shipped 2026-09-11
and all three were one edit as filed:** Diviner's Wand, which shipped as a flat
+2/+1 and flying where the printed card is blank until you draw and then grows
once per draw (`EquipBonus` already carries `triggered_abilities` and
`activated_abilities`, and `AttachSourceTo` behind a `MayDo` is the
Shielded-by-Faith shape, so the Wizard-attach rider came with it); Shieldmage
Elder's Wizard half —
`PreventAllDamageByTargetThisTurn` already takes a card id and
`R::IsSpellOnStack` already names a spell — and Skophos Maze-Warden's
Labyrinth fight, where `EventSpec::caused_by` gates on the object that
*declared* the target and an activated ability's is the permanent it came
from, so "an ability of a land you control named X" needed no primitive.
**Eladamri's own doc comment is stale in the way this file's caveat warns
about**: it says the activated ability is "omitted for want of a
tap-two-creatures cost", and `ActivatedAbility::tap_n_filter` is exactly that
cost — Shieldmage Elder uses it twice. What Eladamri still wants is the body
("reveal a card from your hand **or** the top card of your library"), a
two-zone reveal nothing spells. **Three of the rest are not one edit, and the
reason is worth having before someone re-reads them:** Urborg Panther's second
ability is a sacrifice cost naming *three specific cards* (no multi-named
sacrifice cost exists);
Lumbering Laundry's `{2}` is information-only ("you may look at face-down
creatures you don't control") plus Disguise, so there is nothing for the
engine to model until face-down information is modelled; Touch the Spirit
Realm's channel is the filed blocker (a discard-activated body cannot
target). Two read as one edit and
are not: Emperor of Bones' counter-triggered deploy wants `DeployExiledCreature`
to stamp `last_moved_cards` so a `Seq` can add the finality counter and
`SacrificeAtNextEndStep { LastMoved }` behind it (a `ResolutionScratch`
write — price it, CLAUDE.md's CoW rule); Ersatz Gnomes' spell half wants
"target spell becomes colorless" (`SpellBecomesChosenColor` picks a colour). The triage filed five cards as wanting a
graveyard trigger zone; `EventScope::FromYourGraveyard` is that zone, and
two of them shipped as plain uses of it (Kozilek's Return, Afterburner
Expert) — the other three (Kami of Transience, Sneaky Snacker, Persistent
Marshstalker) wanted the graveyard *walks*, not a gate, and shipped with
them (ENGINE_BACKLOG "FIXED 2026-09-10 (sixth find)"); the last 23 want a primitive
(ENGINE_BACKLOG "The cnt triage's other primitives"). The reader then
learned the triage's own lesson — a helper call it leaves in
`equipped_bonus` or a station band is an ability, the delayed-trigger
effects and the state-trigger fields are trigger lines, "taps a land for
mana" is a static — and `cnt` reads 62: those 37 plus the engine-shape
rows it has no hook for (a keyword standing in for a trigger, a static
`PumpSelfIf`, an `If` folded into an activation).

### Verified-but-overrated (real gaps, but 1v1-equivalent or strictly-better — MED, not HIGH)
| Card | Location | Note |
|---|---|---|
| ~~Spell Queller~~ ✅ **FIXED 2026-09-07** | modern.rs | was a plain counter; now the printed shape off existing primitives — `CounterSpellToZone { ExileWithSource }` (MV ≤ 4) and an LTB `EachPlayerDoes { OwnerOf(CardExiledWithSource) }` around `CastWithoutPayingImmediate`, so the card's owner casts it free when the Queller leaves. Tests: `modern::decks_16_17_misc::spell_queller_*` |
| ~~Generous Gift~~ ✓ | modern.rs | stale — ships `CreateToken { who: ControllerOf(Target(0)) }` (the Elephant) before the Destroy, Beast Within's shape; re-read 2026-09-07 |

### Other notable HIGH — **all closed; every remaining entry here was stale**

Re-checked against the code at the fifty-fourth pass, which is what this
file's own "doc comments lie" caveat exists for. The four that still read as
open were all shipped, three of them with tests already in the suite:

- **Veil of Summer** — the hexproof rider is wired.
  `Effect::GrantHexproofFromColorThisTurn` fills
  `PlayerData::hexproof_from_colors_this_turn`, which the targeting-legality
  checks read **for the player and their permanents** (`actions.rs`
  10976-11042). Tests `veil_of_summer_draws_when_opponent_cast_blue_or_black`,
  `veil_of_summer_grants_hexproof_from_blue_and_black`.
- **Fractal Tender** — both triggers are wired: `increment_self_plus_one`
  and the end-step Fractal on `Predicate::SourceGainedCounterThisTurn`.
  Tests `fractal_tender_end_step_mints_fractal_when_gained_counter` and
  `…_skips_when_no_counter_gained` (the second is what makes the first
  non-vacuous).
- **Pestilent Cauldron / Wandering Archaic** — both back faces are real
  `back_face` definitions (Restorative Burst, Explore the Vastlands), not
  comments. Tests `*_back_*_castable_from_hand` for each, plus
  `explore_the_vastlands_digs_both_players_and_gains_three`.
- **Approach of the Second Sun** and **Heroic Intervention** were already
  marked stale here.

The tier has no open entries. A future run adding one should say which
audit pass found it (`audit_incomplete --structural-only` is the
authoritative one, and it came back with a single triaged finding).

### Fixed this run (protection / keyword / copy primitives)
Sublime Epiphany (CounterAbility + CreateTokenCopyOf modes) · Qasali Pridemage
(Exalted) · Goblin King (mountainwalk) · Yawgmoth (protection from Humans) ·
Baneslayer Angel (protection from Demons/Dragons) · Stonecoil Serpent
(`ProtectionFromMulticolored`, new) · Gingerbrute (haste-only evasion) ·
Built to Smash (+2/+2 + artifact trample) · Mirror Image (`non_legendary` copy) ·
Bloodghast (can't-block + conditional haste) · Augur of Bolas (instant/sorcery
filter) · Pestermite (SkipNextUntap) · Elemental Expressionism (bounce up to 2) ·
Daring Diversion (DealDamageDivided) · Desperate Ritual / Magmatic Sinkhole /
Tasigur (Splice / Delve).

---

## Lower-severity inventory

~150 MED/LOW findings beyond the above. They cluster into the buckets in §
"Missing-primitive buckets," plus:

- **Conditional-gate flattening** (~12): a printed "if/may" condition dropped,
  effect fires unconditionally. **Re-read 2026-09-07 and the named examples
  are stale**: Silverquill Standardbearer, Lorehold Crackleflame and
  Witherbloom Mortislide are synthesized names with no oracle to be wrong
  against, and Witherbloom Apprentice prints no condition (magecraft drain
  1, which is what ships). The bucket has no verified member; a future
  entry names the card *and* the oracle clause it drops.
- **Optional "may" → mandatory — now has an auditor, and it is bigger than
  ~10.** `scripts/audit_dropped_may.py` diffs every catalog definition against
  the offline Scryfall cache and flags the ones whose oracle says "you may"
  and whose definition carries no optional primitive: **349 of 11,094 cards
  checked** (314 of 11,092 at 2026-09-09) (3,901 synthesized `(b###)` names are skipped — they have no
  oracle to be wrong against, which is why the hand-written list above was
  ten). Reminder text and "…rather than pay this spell's mana cost" are
  filtered out; the residue still contains false positives where the engine
  models the choice elsewhere, so read the oracle before fixing one.
  **The cluster where declining actually matters is "you may destroy /
  sacrifice / tap"** — an effect that can hurt its own controller, and a
  trigger's targets are mandatory once it triggers, so the "may" is the only
  out. Eight of those were fixed at the fifty-fourth pass (Aura Shards,
  Reclamation Sage, Manglehorn, Noxious Gearhulk — which also could target
  itself despite "another" — Trygon Predator, Leonin Snarecaster, Choking
  Tethers' cycling trigger, Gitaxian Anatomist); each has a decline test.
  Bounding Krasis and Pestermite followed (`Effect::TapOrUntap` inside a
  `MayDo`; Pestermite also carried a `SkipNextUntap` rider it does not
  print), and Chain Stasis / Thassa's Ire / Sword of the Paruns already had
  the primitive. Detention Sphere (2026-09-09): mandatory *and* able to
  target itself — the printed "not named Detention Sphere" is
  `OtherThanSource` now, the may restored. The rest of that cluster, read
  the same day: Fiend Hunter's filter was `ControlledByOpponent` then, so the
  may was moot — ⚠ **the filter is `OtherThanSource` since 2026-09-20** (the
  fifty-third find; the printed clause is "another target creature", which
  includes one of yours), so the may is a live choice again and the row
  above stands on its own. Mistbreath Elder's bounce is printed
  mandatory (the may is on the "otherwise" fallback the doc approximates),
  Cloudstone Curio already carries `MayReturnSharingPermanentType` (the
  auditor's token list does not see it), and Restoration Angel / Felidar
  Guardian's blink of your own creature is a may the bot would always take.

  **The inverse audit was run and is not worth repeating: noise-dominated.**
  Definitions that are optional where the oracle has no "may" come back 37
  strong and almost all of them are name aliasing — a transform back face or
  a token whose `name:` field matches a *different* real card ("Ghostly
  Castigator", "Vildin-Pack Alpha", "Merfolk", "Spirit"), so the lookup finds
  the wrong oracle. The two genuine-looking ones checked (Coalition Relic,
  Hullbreaker Horror) were not defects: Hullbreaker's `MayDo` models "choose
  up to one", which really is optional. Any future name-keyed audit against
  the cache has to reject a definition whose `name:` is a back face or a
  token before it can say anything.
- **"each opponent" instead of "target/defending player"** (multiplayer drift):
  Bojuka Bog, Tormod's Crypt, … — 1v1-equivalent. Re-read 2026-09-07:
  Hellrider already reads `PlayerRef::DefendingPlayer`; Barbed Servitor's
  oracle has no player clause at all; Lorehold Apprentice *prints* "each
  opponent". Three of the five named were not members.
- **Per-N counting flattened to a constant**: Witherbloom per-creature-milled
  lifegain (×3 → flat +1); Fractal Caller and Prismari Pyroshaper are
  synthesized names (no oracle). **Manifold Key was on this row for the wrong
  reason and was a real defect of another kind — FIXED 2026-09-07**: it
  shipped with the Voltaic Key costs it was written from ({1},{T} evasion,
  {T} untap, no "another"); the printed card is {3},{T} for the evasion and
  {1},{T} to untap *another* artifact. Tests in `stx::part_01` pin both
  costs and the self-target refusal.
- **Manland pump / artifact-creature type / land detail dropped** across
  `decks/lands.rs`. Mishra's Factory, Blinkmoth Nexus, Inkmoth Nexus and
  Mishra's Foundry animate as artifact creatures and carry their pumps
  (2026-09-27); Ghost Quarter and Field of Ruin both ship
  their basic-land search riders (Ghost Quarter's doc comment claimed
  otherwise and is fixed); Thespian's Stage's only residual is the printed
  "except it has this ability" on the copy. Nothing verified open here.
- **Missing creature subtypes bridged to a related type** (cosmetic):
  Dwarf→Warlock, Rhino→Bard, Kor→Warlock, Gorgon→Snake, …

For the full machine-generated list, run the auditor (`--comments-only`).

### Swell the Host (C15, pod seat 32) — open residuals, 2026-09-24

| Card | Gap |
|---|---|

### Seats 33, 34, 36, 37 and 40 (Angels SLD, Built From Scratch C14, Vampiric Bloodline VOC, Plunder the Graves C15, Call the Spirits C15) — open residuals, 2026-09-24

| Card | Deck | Gap |
|---|---|---|

### The `modern_decks` Commander routine's precons (seats 23, 25, 27, 29, 30) — open residuals, 2026-09-24

| Card | Deck | Gap |
|---|---|---|


## Search filters (2026-09-28, `scripts/audit_search_filters.py --gate`, 0)

Fixed: Farseek (any basic → Plains/Island/Swamp/Mountain, nonbasics too), Gift of
Estates (basic Plains → any Plains), Gatecreeper Vine (+ Gate), Shefet Monitor
(+ Desert), Invasion of Theros (+ Aura, Demigod), Brightglass Gearhulk and
Micromancer ("mana value 1" was ≤ 1). Residual: Tallowisp fetches any Aura, not
only one "with enchant creature".

Target adjective / type-list scan (same session): fixed Azorius Charm and Ballista
Squad (had dropped "attacking or blocking"), Restless Ridgeline ("attacking"),
Teferi TR / Divine Gambit / Earth Kingdom Jailer ("artifact, creature, or
enchantment" read as any nonland permanent), Trickster Mage ("artifact, creature,
or land"), Divine Gambit's gift-back (a land is a permanent card). Residuals:
Kitesail Larcenist is still a plain exile (the per-player "becomes a Treasure
for as long as" wants a while-source-on-battlefield `BecomeTreasure`); Hollow
Marauder draws off the last discard only, not one per opponent who discarded
low; Solitude / Shriekmaw / Pit Fight / Teferi TR keep the documented
opponent-only narrowing (Atzocan Archer's reason).

## Scope and slot scans (2026-09-28, session `012RbSk7`) — what the fixes left

Fixed: "target opponent <verb>" read as every opponent (`scripts/audit_target_vs_each.py
--gate`, 0 — the Thunder Junction deserts, Highway Robber, Diregraf Captain, Al Bhed
Salvagers, Keldon Marauders, Clackbridge Troll, Hunted Bonebrute), the CR 506.3 attack scopes (`scripts/audit_attack_scope.py --gate`, 0 —
thirteen bare "attacks you" cards, Oath of Kaya, Revenge of Ravens), the
"for each player / opponent … target … that player controls" slots (Blatant
Thievery, Nils, Riptide Gearhulk, Demonic Junker), 21 "up to" spells whose
slots were mandatory, "if you cast it" riders (Lutri, Iridescent Tiger,
Sentinel's Mark, Weftwalking), Wojek Investigator's per-opponent count, Rain
of Embers' players and flyers, Ink-Eyes / Boneyard Parley targets, Laelia's
library-exile event.

| Card | Residual | Why |
|---|---|---|
| Weftwalking | "the first spell each player casts during each of their turns may be cast without paying its mana cost" is absent | no once-per-turn-per-player free-cast static |
| Coveted Jewel, Flame Sweep, The One Ring, Doomsday Excruciator, Wilt-Leaf Liege, Amped Raptor, Chandra ToD, Ragavan, Robber of the Rich, Gonti, Reckoner Bankbuster | a bare "attack you" fires on planeswalker attacks; "except creatures you control with flying" spares every flyer; the One Ring's protection and the Excruciator's library exile ignore "if you cast it" (fix: `TriggerSourceEnteredByCast`); the Liege has no opponent-discard deploy (fix: `opponent_discard_deploys`); the Raptor's exile ignores "if you cast it from your hand", takes the top card only and costs a flat {E}×4; the impulse "you may cast" of Chandra / Ragavan / Robber / Gonti lets an exiled land be played; Bankbuster's Pilot lacks the +2 crew bonus | cube pool — frozen |
| Choreographed Sparks, Render Speechless, Cost of Brilliance, Homesickness, Vibrant Outburst, Burrog Barrage, Chelonian Tackle, Dissection Practice, Impractical Joke | copy any-controller spell / "up to" slots still mandatory | SOS pool — frozen; this also blocks the CR 601.2c later-slot gate (TODO) |
| Devious Cover-Up | the up-to-four graveyard cards are chosen as it resolves, not targeted | `ShuffleGraveyardCardsIntoLibrary` is a resolution-time pick (Cathartic Parting's shape) |
| Rhuk, Hexgold Nabber | only the attack half: an equipped creature of yours DYING doesn't move its Equipment to Rhuk | the Equipment falls off before the trigger resolves; `AttachedToMe` of the dead creature needs last-known attachments |
| Imprison | neither the {T}-ability tax nor the attacks-or-blocks tax is implemented | two pay-or-destroy triggers on the enchanted creature's actions |
| Jace, Vryn's Prodigy | the −9 emblem mills EACH opponent five, not one target opponent | an emblem trigger with a player target is untested ground; the ultimate is rarely reached |
| ~~Rayne, Academy Chancellor~~ ✅ | fixed 2026-10-25: `EventScope::YouOrYourPermanentTargetedByOpponent` hears `GameEvent::PlayerBecameTarget` (once per target, the 2004-10-04 ruling), and the enchanted second draw is read on resolution; the same scope fixed Leovold, which drew only when Leovold itself was targeted — `leovold_draws_when_you_or_your_permanents_are_targeted_by_an_opponent`, `rayne_draws_off_its_controller_and_twice_when_enchanted` | — |
| ~~Witchbane Orb~~ ✅ | fixed 2026-10-25: the ETB destroys `Curse` + `R::AttachedToYou` (CR 303.4) — `witchbane_orb_destroys_only_the_curses_attached_to_you` | — |
| Psychic Purge | "when a spell or ability an opponent controls causes you to discard this, that player loses 5 life" is absent | `GameEvent::OpponentCausedYouToDiscard` doesn't carry the causing seat, and no `PlayerRef` names it |
| The Raven's Warning | III "you may put a card you own from outside the game on top of your library" does nothing | no wish-to-library-top effect (`WishToLibrary` shuffles in); a Commander game has no sideboard, so the gap only shows in sideboarded play |
| 🟡 Cryptic Spires | the two colors are circled as the deck is built | no deck-building choice: `ManaPayload::CircledColors` reads the controller's commander identity — exact for a one- or two-color identity, its first two colors (WUBRG order) in a 3+ color deck, white and blue with no commander |
| 🟡 Dawnhand Dissident | the blight player picks the creature; the caster picks which three counters to remove | both picks are automatic: blight goes on a creature that survives it (weakest first), the counter drain takes -1/-1 counters first, weakest creature first (as Ward—Blight's payer already does) |
| 🟡 Brine Comber // Brinebound Gift | the Gift's "enchanted creature becomes the target of an Aura spell" Spirit is the Aura controller's | the trigger rides the enchanted creature (`EquipBonus.triggered_abilities`), so on an opponent's creature its controller gets the Spirit |
| 🟡 Zur's Weirding | "players play with their hands revealed" covers its controller too; each other player decides whether to pay | the reveal is `OpponentsPlayWithHandsRevealed` (the controller's hand stays hidden); the pay-2-life question is asked inside the draw replacement, which has no stack item to suspend on, so a prompting seat gets the headless answer |

### EDHREC average-deck seats (`decks::cmdr_edhrec3`, 2026-10-11 Commander run) — open residuals

Ardenn and Elrond were closed 2026-10-11 late (CR 701.3b host legality;
`ManaWithoutTap`). New seats' residuals go here.

| Card | Residual | Why |
|---|---|---|
| 🟡 Exchange of Words | the text-box exchange outlasts the enchantment (it ends only as either creature leaves) | `ExchangeTextBoxes` bakes the swap into the two instances; a "for as long as this remains" duration needs a reversible layer-3 effect |
| 🟡 Mikey & Don, Party Planners | "if you cast a creature spell this way" also counts a Mutant, Ninja or Turtle creature cast off the library top through another permission (Vizier of the Menagerie) | the extra counter keys on `R::WasCastFromLibrary`; an uncapped `PlayFromLibraryTop` cast records no granting source |
| ~~Scuttlemutt~~ ✅ | fixed 2026-10-28: "the color or colors of your choice" is any nonempty set (`BecomeChosenColors`; Quickchange, Dream Coat, Govern the Guildless's forecast and Prismatic Lace share it) | — |
| ~~Scrounging Bandar~~ ✅ | fixed 2026-10-27: `MoveAnyNumberOfCounters { kind: Some(+1/+1) }` asks only about +1/+1 counters (Bioshift shares it) | — |
| 🟡 Shelob, Child of Ungoliant; Supper for Spiders | the copy / the returned cards enter as the creature and become Food artifacts right after (CR 707.9b / 614.1c want them to enter as Food) | `CreateTokenCopyOf` and `Move` have no type-replacing entry rider; `Effect::BecomeFoodArtifact` rewrites them on the battlefield |
| ~~Myojin of Cryptic Dreams~~ ✅ | fixed 2026-10-24: enters with the counter (`SelfEntersWithKeywordCounter{if_cast_from_hand}`, also Grim Betrayal), no trigger window — `myojins_count_only_a_cast_from_hand` | — |
| ~~Goldberry, River-Daughter~~ ✅ | fixed 2026-10-28: the {U} ability asks a count per kind (`MoveAnyNumberOfCounters`) and draws only if her total fell | — |
| ~~Court of Locthwain~~ ✅ | fixed 2026-10-24: the monarch's free cast is `MayCastExiledWithSource` during the resolution, so no exiled card's lasting permission is replaced | — |
| 🟡 Elemental Teachings | the four lands and the opponent's two are picked by `SearchSplitWithOpponent`'s heuristic (library order; the first two denied), not asked of either seat | the split has no decision of its own; Ecological Appreciation shares it |
| 🟡 Night Shift of the Living Dead | a UI seat isn't asked: it takes the headless shift (+1 when that reaches a later results arm or a die-trigger threshold) | the shift is offered inside `RollDie`'s resolver after the dice land; it can't park there and resume with the same faces |
| ~~Animation Module~~ ✅ | fixed 2026-10-24: `AddCounterOfPresentKind` takes a player target (an opponent's poison, your experience / energy) | — |
| 🟡 Cover of Winter | "prevent X of that damage" is X from each combat damage *event*, so a creature damaging two of your creatures (or a blocker and you, trample) at once has X prevented from each rather than X in total | `StaticEffect::ReduceCreatureCombatDamageToYouAndYoursByCounters` shaves in the per-recipient prevention funnel; there is no per-source combat damage total to split X over |
| 🟡 Nael, Avizoa Aeronaut; Voda Sea Scavenger | "put up to one of them on top" / "you may put one … on top" always keeps one card on top (the seat picks which) | `LookTopKeepOneRestToGraveyard { rest_bottom_random }` has no keep-none answer |
| 🟡 Chandra, Pyromaster | +1's second target is "a creature an opponent controls", not one controlled by the targeted player or planeswalker's controller, and as a loyalty slot past the first it is auto-filled; −7's instant or sorcery is the priciest exiled one, not a choice | a cross-slot controller constraint has no slot filter; `ActivateLoyaltyAbility` carries one target |
| 🟡 Chandra, Heart of Fire | −9's "any number" exiles every red instant and sorcery, and the library half is a move, not a search (no search triggers or taxes) | `Move` over `Selector::Both(graveyard, library)` |
| 🟡 Chandra, the Firebrand | −6's "each of up to six targets" past the first is the loyalty auto-fill (the Jeska row) | `ActivateLoyaltyAbility` carries one target |
| 🟡 Rohirrim Chargers | the Equipment revealed attaches to its best legal host, not "that creature" (the one exerted), and one that can't attach goes to hand instead of the bottom | `RevealUntilPutAttachedElseHand` (Songbirds' Blessing) has no host argument |
| ~~Drought~~ ✅ | fixed 2026-10-25 (`01Y6ZRFx`): mana abilities pay too — the auto-tapper fires a costed mana source through `activate_ability_into`, which joins the Swamp sacrifice | — |
| 🟡 Lightning Storm | deals 3: its "discard a land card: two charge counters, you may retarget" ability — any player, only while it is on the stack — is missing, so no counter lands | no ability activates from the stack (legal actions walk permanents and cards in zones, not stack items) |
| 🟡 Sen Triplets | the chosen opponent can still activate mana abilities | `PlayerCantActivateNonManaAbilitiesThisTurn` (Abeyance's lock) is the nearest; there is no all-abilities lock |
| ~~Teferi, Temporal Pilgrim~~ ✅ | fixed 2026-10-24: the −12 runs `EachPlayerDoes { Target(0) }` over a `MoveChosen` to hand before the shuffle — the opponent's pick, not a target | — |
| 🟡 Smaug the Impenetrable | combat damage to it still puts its trigger on the stack (it resolves to nothing) | `EventKind` has no bit left for a noncombat kind; `Effect::IfTriggerDamageNoncombat` gates the body per event instead |
| ~~Rhys, the Evermore~~ ✅ | fixed 2026-10-28: a count per counter kind (`RemoveAnyNumberOfCounters`, keyword counters too); headless strips what its owner doesn't want | — |
| ~~Lara Croft, Tomb Raider~~ ✅ | fixed 2026-10-27: the grants form one `OneCastAmongGranted` group — one play, a land drop included | — |
| ~~Loki, God of Mischief~~ ✅ | fixed 2026-10-28: listens to `BecameTarget` under `EventScope::PlayerOrPermanentTargetedByYourAbility` (battlefield permanents and players only) | — |
| 🟡 Sentinel Tower | the count reads only your own instants and sorceries this turn; one an opponent cast during your turn adds nothing to a later shot (and theirs triggers it for your count) | `Value::InstantsOrSorceriesCastThisTurn` is per player; there is no table-wide instant/sorcery tally |
| ~~Wernog, Rider's Chaplain~~ ✅ | fixed 2026-10-24: `TemptingOffer::declined` runs "loses 1 life" under each opponent who declined | — |
| 🟡 Huatli, Warrior Poet | the −X's targets past the first are the engine's pick | a loyalty activation carries one target (`GameAction::ActivateLoyaltyAbility`) |
| ~~Nine Lives~~ ✅ | fixed 2026-10-28: its replacement (and The Mindskinner's) is a prevention — unpreventable damage is dealt (CR 615.12) | — |
| ~~Roshan, Hidden Magister~~ ✅ | fixed 2026-10-28: creature spells and owned cards off the battlefield are Assassins too (`OwnedCardsOffBattlefieldAreTypeToo`) | — |
| ~~Gandalf the White~~ ✅ | fixed 2026-10-24: `StaticEffect::DoubleControllerLeaveTriggers` doubles a trigger a leaving legendary permanent or artifact causes | — |
| ~~Jade Monolith~~ ✅ | fixed 2026-10-25: `RedirectNextDamageTo { chosen_source }` keys the redirect on the source picked on resolution (Kor Chant shares it with `whole_turn`) — `jade_monolith_redirects_only_the_chosen_source` | — |
| 🟡 Verdant Mastery, Fervent Mastery | their picks are still separate searches (two taxes under Leonin Arbiter) | left out of `OneSearch` (`01M3bbRx`): STX cards in the two-player pool |
| ~~Lochmere Serpent~~ ✅ | fixed 2026-10-25: exactly five targets, all in one opponent's graveyard (`SameGraveyardAsTargetSlot(0)`, the 2019-10-04 ruling) — `lochmere_serpent_needs_five_targets_in_a_single_opponents_graveyard` | — |
| 🟡 Lagrella, the Magpie | exiles only opponents' creatures, and a returning card gets no +1/+1 counters | the "enters under your control" rider has no engine hook; your own creatures would come back without it |
| 🟡 Battletide Alchemist | the "you may prevent" is always taken for its controller's side and never for an opponent | `ReduceDamageToYourTeamByCount` has no per-event prompt |
| ~~King T'Challa // Black Panther, Hope Enduring~~ ✅ | fixed 2026-10-24: a modal DFC whose front also transforms (CR 712.11b) — the back face carries its {4}{W}{U} and is cast from hand | — |
| 🟡 Dire Undercurrents | the discard trigger can only target an opponent | the bot never targets itself with it anyway |
| 🟡 Phyrexian Purge | the 3 life per target is lost as the spell resolves, not paid as a cast cost | no per-target life surcharge in the cast pipeline |
| 🟡 Primal Prayers | a creature cast for {E} doesn't gain flash | `EnergyAlternativeCostForFilter` has no timing rider |
| ~~Bane, Lord of Darkness~~ ✅ | fixed 2026-10-27: `AnyPlayerMayAccept` asks the target opponent; declined, `MoveChosen` with `ToughnessAtMostXFromCost` (X = the dead creature's LKI toughness; `MoveChosen` now resolves X in its filter) | — |
| ~~Queen Kayla bin-Kroog~~ ✅ | fixed 2026-10-30: `Effect::ChoosePerManaValue` asks every mana value first, then one `Move` returns the picks together (Gruesome Menagerie shares it) | — |
| ~~The Fifteenth Doctor~~ ✅ | fixed 2026-10-24: gated on `CastSpellThisTurnWithout { types: [Artifact] }`; the improvise grant also never applied through its gate before this (`spell_granted_improvise` now peels) | — |
| 🟡 The Muckslinger Gang | "creature spells you control and creature cards you own in any zone other than the battlefield or the stack get +3/+3" is not modelled — only the battlefield half (other creatures not attacking or blocking get +3/+3) | an off-battlefield P/T static read by every hand/graveyard/stack power check; no free board-presence lane to gate it |
| 🟡 Hostile Negotiations | the face-up pile is fixed (the second three), not turned up by your choice | rides `FaceDownFaceUpPiles` |
| 🟡 Fortune's Favor | the opponent's split has no face-down pile — you see both | rides `FactOrFiction` |
| 🟡 Woolly Razorback | with an ice counter it assigns no combat damage rather than having it prevented ("can't be prevented" doesn't restore it) | `Keyword::DealsNoCombatDamage` stands in for the prevention |
| ~~The Enigma Jewel~~ ✅ | fixed 2026-10-28: "four or more" (`ActivatedAbility.craft_or_more`) exiles every matching graveyard card past four (Ore-Rich Stalactite shares it) | — |
| 🟡 Elda, Conjurer of Spectacle | entering after a nonlegendary creature spell this turn, the next one still gets demonstrate | a once-per-turn cast trigger, not a count of this turn's nonlegendary creature spells |
| ~~Kyodai, Soul of Kamigawa~~ ✅ | fixed 2026-10-28: `Duration::WhileYouControlSource` (CR 611.2b) | — |
| 🟡 Fleeting Memories | an artifact that is a Clue only through Senator Peacock doesn't count as a Clue once sacrificed | `lki_clone` snapshots creature types, not artifact subtypes |
| 🟡 Metamorphic Alteration | the copy starts when an enters trigger resolves (respondable), not as the Aura enters, and moving the Aura to another creature doesn't copy the new host | `BecomeCopyOfFor` with `WhileSourceAttached`; no static layer-1 copy for an attachment |
| ~~The Capitoline Triad~~ ✅ | fixed 2026-10-28: a manual seat names the exiled set (a short total takes the auto-pick) | — |
| 🟡 Hama, the Bloodbender | the exiled card is cast for its mana value in any mana — your artifacts and creatures can't help pay (CR 701.67). Your-turn-only and "for as long as you control Hama" are modelled since 2026-10-28 | `GrantMayPlay { pay_own_cost, any_color }` + `WhileHolderControlsSource { your_turn }` |
| ~~Myojin of Blooming Dawn~~ ✅ | fixed 2026-10-24: gated on `WasCastFromHand` (a table test covers all ten Myojins: hand, reanimated, cast from exile) | — |
| 🟡 Primal Surge | after each card put onto the battlefield it asks "repeat?" separately from "put it?" (declining either stops), and a seat past 150 permanents stops putting | `Effect::RepeatWhileMay` re-asks each loop; the board bound is the engine's, not the card's |
| 🟡 Suppressor Skyguard | the shield lasts the turn, not just that combat | `PreventAllCombatDamageToPlayerThisTurn` has no per-combat duration |
| 🟡 Zethi, Arcane Blademaster | the ETB chooses its instants (the cheapest) instead of targeting them | `Effect::ExileFromGraveyard` with `Value::TimesKicked`; an ETB's auto-targeting fills one slot of a `CapTargetsAt` |
| ~~Yasharn, Implacable Earth~~ ✅ | fixed 2026-10-24: emerge, offering, casualty and bargain sacrifices ask `sacrificeable_for_cost` like the additional-cost and alternative-cost paths | — |
| 🟡 Rootweaver Druid | the land given away is the engine's pick among the searched ones, not the searching opponent's | `Selector::one_of(LastMoved)` |
| 🟡 Game Plan | no assist: nobody else may pay toward it | there is no assist keyword |
| 🟡 Baba Lysaga, Night Witch | can't sacrifice itself; a headless seat sacrifices a full three-type batch or nothing | `sac_any_number_filter` excludes the source; `distinct_type_sac_batch` is the auto-line |
| 🟡 Sheltering Ancient | every counter of an upkeep goes on the opponents' smallest creature (the engine's pick) | `CumulativeUpkeepCost::PutCounterOnOpponentCreature` auto-pays |
| 🟡 Tolarian Contempt | the owner's top-or-bottom choice is the engine's | `LibraryPosition::OwnerChoice` auto-picks |
| 🟡 Disciple of Perdition | at exactly 13 life both modes always happen (the "may choose both" is taken) | `Effect::If` on `PlayerLifeExactly` picks the pair over the one-mode `ChooseMode` |
| ~~Irencrag Feat~~ ✅ | stale row (checked 2026-10-24): `Effect::CapSpellsThisTurn` sets `spell_cap_this_turn` as the spell resolves; `irencrag_feat_allows_one_more_spell` | — |
| ~~Ochre Jelly~~ ✅ | fixed 2026-10-24: the count is a `WithX` snapshot at death, and `DelayUntilWithCapture` now carries it to the end-step copy | — |
| ~~Hooded Blightfang~~ ✅ | fixed 2026-10-24: `DealsDamage` (combat or not) gated on a planeswalker in slot 0; the noncombat funnel now fires it for planeswalker / battle hits | — |
| 🟡 Soulfire Grand Master | the buyback rider catches the next instant or sorcery cast from any zone, not only from your hand, and rides a trigger | `Effect::OnYourNextInstantSorceryThisTurn` + `SpellReturnsToHandOnResolve` |
| ~~Altar of the Wretched // Wretched Bonemass~~ ✅ | fixed 2026-10-24: twelve `SelfHasKeywordWhilePredicate` statics read the exiled crafting cards (protection, parametrized, is not copied) | — |
| 🟡 The Creation of Avacyn | the tutored card is exiled face up (the face-down hiding isn't modelled) | `ZoneDest::ExileWithSourceStamp` |
| 🟡 Magar of the Magic Strings | a snapshot reload rebuilds the face-down puppet as a plain 2/2 (as it does Missy's Cybermen) | the serde wire stores only `face_down_permanent` |
| 🟡 Intimidation Bolt | "other creatures can't attack this turn" is fixed at resolution: a creature entering later can attack | `CantAttackThisTurn` over a resolved set |
| 🟡 Ride Down | the creatures it blocked "this combat" are read as "this turn" | `Selector::CreaturesBlockedByThisTurn` |
| 🟡 Tranquil Frillback | each {G} payment buys one mode under its own reflexive trigger, not one trigger choosing up to that many | three `MayPay` → `ReflexiveTrigger` |
| ~~Experimental Overload~~ ✅ | stale 2026-10-28: the return is already a `MayDo` + `MoveChosen` pick | — |
| ~~Demonic Embrace~~ ✅ | fixed 2026-10-30: two `CastSelfFromGraveyardWithCost` statics (3 life, a discard) — additional costs on its graveyard cast (CR 601.2f), no longer an alternative cost | — |
| 🟡 Kraven the Hunter | "greatest power among creatures that player controls" compares the dead creature with the survivors, so two of the biggest dying together both count | `GreatestPowerControlled` after the death |
| ~~Quilled Greatwurm~~ ✅ | fixed 2026-10-24: `CastSelfFromGraveyardWithCost(RemoveCountersFromCreatures { count: 6 })` | — |
| 🟡 Vaevictis Asmadi, the Dire | its per-player target is mandatory in print; `ForEachPlayerTarget` lets a seat leave one out | `ForEachPlayerTarget`'s "up to" |
| ~~Gix's Command~~ ✅ | fixed 2026-10-28: the counters mode's "up to one creature" is any creature, chosen as it resolves (`ChooseSomeAmong`) | — |
| ~~Ziatora's Envoy~~ ✅ | fixed 2026-10-27: `CastTopFreeIfElseDraw { land_or_hand }` plays a top land with a land play left and puts a miss into hand | — |
| 🟡 Reed Richards, Smartest Man | on your own turn past the draw step, the draw-step draw is assumed to have happened (a skipped draw step mis-counts by one) | `FirstExtraDrawEachTurnBecomes` reads `cards_drawn_this_turn` |
| ~~Leatherhead, Swamp Stalker~~ ✅ | fixed 2026-10-24: `StaticEffect::SelfEntersWithKeywordCounter` puts the hexproof counter on as she enters (CR 614.1c) — `leatherhead_enters_already_hexproof` | — |
| 🟡 Hope Tender | the exert lands as the ability resolves; there is no exert activation cost | `SkipNextUntap` in the effect |
| 🟡 Steward of Solidarity | the exert lands as the ability resolves; there is no exert activation cost | `SkipNextUntap` in the effect |
| ~~Mishra's Research Desk~~ ✅ | stale row: `impulse_choose_one` grants only the chosen card (`mishras_research_desk_impulses_and_unearths`) | — |
| ~~Ajani, Sleeper Agent~~ ✅ | fixed 2026-10-28: a missed +1 reveal may stay on top | — |
| 🟡 Invoke the Ancients | each Spirit's keyword-counter pick is "up to one", so a seat may leave one without a counter | `ChooseUpToN { max: 1 }` per token |
| ~~Rona, Disciple of Gix~~ ✅ | stale row: both grants already ride `RestrictMayPlayToCasting` (`rona_cant_play_an_exiled_land`) | — |
| ~~Wanda's Vision~~ ✅ | fixed 2026-10-24: `ExileTopUntilNonland` + `CastWithoutPayingImmediate` during the resolution | — |
| ~~Stangg, Echo Warrior~~ ✅ | fixed 2026-10-28: a token copy of each Aura and Equipment on him enters attached to the Twin (`CreateTokenCopyOfAttachedToEach`), all sacrificed at the next end step | — |
| ~~Spider-Verse~~ ✅ | fixed 2026-10-24: `once_each_turn_on_take` — the use is spent on a copy taken (Riveteers Ascendancy had the same shape, fixed with it) | — |
| ~~Casting of Bones~~ ✅ | fixed 2026-10-27: `Effect::DiscardOneOfDrawn` picks among this resolution's `CardDrawn` cards | — |
| 🟡 Skill Borrower | the top card of your library is only visible to you, not revealed to the table | `MayLookAtOwnLibraryTop` |
| ~~Doctor Jane Foster~~ ✅ | fixed 2026-10-27: with life gained this turn the card moves graveyard → battlefield in one move (`If` picks the destination) | — |
| 🟡 Trelane, Squire of Gothos | no `Q` creature type in the engine, so it has no creature type | vigilance / ward {2} / unblockable |
| 🟡 Sensation Gorger | the top card is checked before you choose to look; a non-sharing top card is never offered | `If` over `SharesCreatureTypeWithSource`, then `MayDo` |
| 🟡 Flying Drone | "costs {1}{U} less to activate if …" is two abilities: the full-price loot gated off the condition and a free one gated on it | two `ActivatedAbility::condition` arms |
| 🟡 Lightmine Field | "each of those creatures" / "the number of attacking creatures" read every attacker as the trigger resolves, not the declaration that triggered it | `YouAttack` + `IsAttacking` |
| 🟡 Pick Up the Pace | "cards exiled with this enchantment" stay playable after the enchantment leaves (the permission is not tied to it, like Neyali's) | `ExileTopAndGrantMayPlay { TurnsHolderAttacks }` |
| 🟡 Volcano Hellion | "an amount of damage of your choice" is offered up to the target creature's toughness, not without limit | `PlayerChoosesNumber { max: ToughnessOf(Target(0)) }` |
| 🟡 Hawkeye, Master Marksman | modes are chosen as the trigger resolves and each targeted mode is its own reflexive trigger | `MayPayGenericUpTo` + `ChooseUpToN` + `Reflexive` |
| 🟡 Zareth San, the Trickster | the returned unblocked Rogue is auto-picked | from-hand `bounce_other_filter` + `JoinCombatAttackingChosen` |
| 🟡 Hew the Entwood | chosen cards enter one at a time in pick order (not all nonlands, then all lands); the bot sacrifices no lands | `SacrificeAnyNumber` + `LookPick` |
| ~~Baron Helmut Zemo~~ ✅ | fixed 2026-10-28: a manual seat names the boast's exiled set, and the controller picks which copies (up to three) to cast | — |
| 🟡 Lethal Exploit | the modified creatures are counted as it resolves, not as it is cast | `PumpPT` with `CountOf` |
| 🟡 Fall of the Impostor | a tie for greatest power exiles the first in battlefield order, not the controller's choice | `TakeGreatestPower` |
| 🟡 Shah of Naar Isle | "up to three" is three yes/no draws | `EachPlayerDoes` + `MayDo` |
| 🟡 The Kingpin of Crime | the attack trigger's "creatures with toughness greater than their power" is the set as it resolves; a creature that qualifies later in the turn doesn't assign by toughness (CR 611.2c) | `GrantKeyword { AssignsCombatDamageByToughness, EndOfTurn }` |
| 🟡 Safe Passage | "creatures you control" are the ones you control as it resolves; a creature that enters later this turn is unshielded | two `PreventAllDamageThisTurn` |
| 🟡 Mordor on the March | "exile a creature card from your graveyard" is a target (a hexproof-from-graveyard effect would stop it); each storm copy re-aims at another card | `CreateTokenCopyOf` + `Exile` + `DelayUntil` |
| 🟡 Whtz, the Bibliophile | the "{3} less with 200 or more cards in your starting deck" discount isn't modelled (no starting-deck size is kept; every seated deck has 100) | `ActivatedAbility` |
| 🟡 Aragorn, Company Leader | "one of each of those kinds" is one trigger per kind put, so two kinds at once may land on two different creatures | `AddCounterOfTriggerKind` on `YouPutCounters` |
| 🟡 Touch of Moonglove | the granted "a creature it damaged dies" trigger doesn't look back (CR 603.10a) when the pumped creature dies in the same event as its victim | `GrantTriggeredAbility { EndOfTurn }` |
| 🟡 Ib Halfheart, Goblin Tactician | the blocked Goblin deals its 4 just before it is sacrificed (the blockers leave the block map with it), and only while it is still yours | `DealDamageFrom` + `SacrificeSelected` |
| 🟡 Tocasia, Dig Site Mentor | the graveyard ability returns cheapest-first rather than the chosen targets | `MoveWithinTotalManaValue` |
| 🟡 Nishoba Brawler | domain counts printed land types while the layers compute (CR 613.8), so a basic type a land only gains (Navigator's Compass) doesn't raise its power | `SelfBasePtFromValue { DomainCount }` |
| 🟡 Crush Underfoot | the Giant is the greatest-power one you control, picked automatically rather than chosen (ties go to the first) | `DealDamageEqualToPower { TakeGreatestPower }` |
| 🟡 Nimble Hobbit | with a Food out, sacrificing a Food is the only offered cost (paying {2}{W} is offered only with no Food) | `If(SelectorExists Food)` |
| 🟡 Raven Clan War-Axe | "you may search your library and/or graveyard" takes Eivor from the graveyard when one is there, else searches the library; the search can't be declined | `If(CardsInZone gy) Move else Search` |
| ~~Master Skald~~ ✅ | fixed 2026-10-28: the exiled creature card is the controller's pick (`MoveChosen`, a cost pick) | — |
| 🟡 Lens of Clarity | the look-at-top half is `MayLookAtOwnLibraryTop`; looking at face-down creatures you don't control isn't surfaced to the seat's view | — |
| 🟡 Jasmine Boreal of the Seven | an *attacker's* "no abilities" reads its printed card (an anthem's keyword doesn't disqualify it); the blocker side reads granted keywords and lose-all-abilities | `GrantKeyword { CantBeBlockedBy }` filtered `HasNoAbilities` |
| 🟡 Rhino, Terrible Trampler | the ETB is two triggers (destroy; distribute + trample) rather than one, so a response can land between them and the distribution still resolves if the destroy's target goes | two `etb` triggers |
| 🟡 Patch Up | the three cards are picked cheapest-first at resolution, not targeted | `MoveWithinTotalManaValue` |
| ~~Cantankerous Keepers~~ ✅ | fixed 2026-10-28: `Mill` then `Move` of every Elf among `LastMoved` — no pick | — |
| 🟡 Wonder Man, Hollywood Hero | two Wonder Men still allow only one extra activation (it's legendary) | `StaticEffect::PowerUpOnceMore` |
| 🟡 Rhino's Rampage | the excess-damage check reads the creature's excess damage this turn, so excess dealt to it earlier the same turn also counts | `ExcessDamageDealtThisTurnTo` + `ReflexiveTrigger` |
| 🟡 Sméagol, Helpful Guide | "target opponent" is the most hostile opponent, not a target (no ward/hexproof check) | `RevealUntilFind { who: HostileOpponent }` |
| 🟡 Glamerdye / Sleight of Mind | the color-word swap targets a permanent only; "target spell" isn't offered | `ReplaceColorWord` |
| 🟡 Kargan Intimidator | the mode and its target are picked at resolution (auto-targeted), not as the ability is activated (CR 700.2) | `ChooseUnchosenModeThisTurn` |
| 🟡 Obsidian Battle-Axe | the attach offer fires only for Warriors entering under your control (attaching to an opponent's is legal, never wanted) | `EventScope::YourControl` + `MayDo(Attach)` |
| ~~Planar Atlas~~ ✅ | fixed 2026-10-28: an optional look that keeps up to one LAND on top (`LookPick { pick_filter: Land, keep_on_top }`), the rest bottomed at random | — |
| 🟡 Portal Manipulator | the player and the attackers are picked as the trigger resolves, not targeted as it triggers; every attack on you that you could redirect moves (CR 115.1) | `AttackersNowAttackChosenPlayer` |
| 🟡 Aurora Awakener | "put any number of those permanent cards onto the battlefield" puts all of them | `RevealUntilMatchingToBattlefield` |
| 🟡 Tetsuo, Imperial Champion | N reads every permanent attached to Tetsuo (an Aura's mana value counts, not only Equipment) | `HighestManaValueAmong(AttachedToMe(This))` |
| 🟡 Wicked Slumber | with two targets the two stun counters go one each; "both on either" isn't offered | `ApplyToTargets` + `If(count(AllTargets) <= 1)` |
| 🟡 Arc Spitter | the granted ping may also aim at a creature the wearer is blocking, not only one blocking it | `R::BlockingOrBlockedBySource` |
| 🟡 Alhammarret, High Arbiter | the name is any nonland card, not one revealed from the opponents' hands (no reveal step) | `NameCard { restrict_to: Nonland }` + `OpponentsCantCastNamed` |
| 🟡 The Lady of Otaria | "a land you controlled" reads a land card in your graveyard that left the battlefield this turn (a stolen land of yours / your land an opponent owns are misread) | `EachMatching { Graveyard(You), PutIntoGraveyardFromBattlefieldThisTurn }` |
| 🟡 Nick Fury, Agent of S.H.I.E.L.D. | "if it's a double-faced card, you may transform it" is never offered: the card enters front face up | `LookTopPutMatchingOntoBattlefield { max: 1 }` |
| 🟡 Falco Spara, Pactweaver | the counter a library-top cast removes is auto-picked (-1/-1 first, then the weakest creature), not chosen | `AdditionalCastCost::RemoveCountersFromCreatures` |
| 🟡 Evelyn, the Covetous | "once each turn, you may play a card" is a one-spell-a-turn grant (`WhileSourceOnBattlefield { one_spell_a_turn }`): a land exiled this way stays playable beside the turn's spell, and the grant ends when Evelyn leaves | `GrantMayPlay` over `LastMoved` |
| 🟡 Rootpath Purifier | "basic" is read by the `IsBasicLand` / `IsNonbasicLand` walkers (searches, counts, nonbasic hate in effects); a layer-4 static over nonbasic lands (Blood Moon) still reads the printed supertype | `StaticEffect::LandsAndLibraryLandsAreBasic` |
| 🟡 Boiling Rock Rioter | "an Ally spell from among cards you own exiled with it" casts the first such card, not a chosen one | `CastWithoutPayingImmediate { MatchingAmong(CardExiledWithSource), pay_own_cost }` |
| 🟡 Nathan Drake, Treasure Hunter | "spend mana as though any color" covers only the spell his attack casts (and that one only when you do not own it), not other unowned spells or abilities of permanents you control but do not own; the cast takes the first eligible exiled card, an unowned one first | `CastImmediateAnyManaType` / `CastWithoutPayingImmediate` |
| 🟡 Kotose, the Silent Spider | "you may play one of the exiled cards" is one a turn while Kotose stays (`WhileSourceOnBattlefield { one_spell_a_turn }`), not exactly one ever; lands among them stay playable beside it | `GrantMayPlay` over the swept `LastMoved` |
| 🟡 Soundwave, Sonic Spy | the instant or sorcery card is chosen as the trigger resolves (`MoveChosen` from the damaged player's graveyard), not targeted as it triggers | `MoveChosen` + `CastWithoutPayingImmediate { copy }` + `CastImmediatelyThisResolution` |
| 🟡 Titania, Rugged Rumbler | Ward's "discard a card or pay {2}" auto-pays the mana when it can (no menu); the discard is the hand's first card | `WardCost::ManaOrDiscard` |
| 🟡 Norritt | "controlled continuously since the beginning of the turn" reads "didn't enter this turn" (a creature that changed control this turn qualifies) | `R::Not(EnteredThisTurn)` + `DelayUntilWithCapture` |
| 🟡 Chandra's Embercat | its {R} pays for Elemental spells only; "or a Chandra planeswalker spell" has no `SpendRestriction` | `SpendRestriction::CreatureOfType(Elemental)` |
| 🟡 Thelon of Havenwood | "each Fungus creature gets +1/+1 per spore counter" pumps only the Fungi its controller controls (`PumpTeamByControlledPermanents` is you-scoped) | `per_own_counter: Some(Spore)` |
| 🟡 The Ruinous Wrecking Crew | the up-to-X modes are chosen as the trigger resolves (CR 700.2b chooses them as it is put on the stack), so a target is named for every targeted mode up front | `ChooseN { up_to: Some(XFromCost) }` |
| 🟡 Draugr Necromancer | "spend mana from snow sources as though any color" is not modelled (the exiled cards cost their printed colors); only the Draugr that exiled a card grants its cast | `ExileDyingOpponentCreaturesWithCounterCastable` |
| 🟡 Grub, Notorious Auntie | the copy's "at the beginning of the end step, sacrifice this token" is a delayed sacrifice, not an ability the token has (a copy of the token lacks it) | `BlightThen` + `SacrificeAtNextEndStep` |
| 🟡 Wee Champion | the art-sticker counter never comes up — only name stickers exist (CR 123.6) | `KeywordAct::StickerPlaced` |
| 🟡 Werewolf Ransacker | the 3 damage goes to the destroyed artifact's owner (its last controller in almost every game) | `OwnerOf(Target(0))` |
| 🟡 Taster of Wares | a headless revealer shows the cards it would discard first (its own choice of which X); "for as long as you control this creature" lasts while it is on the battlefield | `RevealFromHandExileChosen` + `WhileSourceOnBattlefield` |
| 🟡 Noctis, Heir Apparent | warp-strike's return is a delayed trigger read off its activator's next declaration, not its owner's when they differ | `DelayedKind::PlayersNextDeclareAttackers` |
| 🟡 Floodchaser | "target land becomes an Island until end of turn" adds Island instead of replacing its land types (the attack gate reads the type; the land keeps its own mana ability) | `Effect::GainLandType` |
| 🟡 Magical Hack | the basic-land-type text change can target a permanent only, not a spell on the stack | `Effect::ReplaceBasicLandType` |
| 🟡 Bloodfeather Phoenix | only an instant or sorcery's damage to an *opponent* returns it — damage to a battle doesn't | `FromYourGraveyardSourceDamagedOpponent` + `dealt_by` |
| 🟡 Curse of Shaken Faith | "or copies a spell" doesn't trigger it — only a cast after the first | `SpellCast` + `SpellsCastThisTurnEquals` |
| 🟡 Curse of Surveillance | "any number of target players other than that player" is only you | `Draw { who: You }` |
| 🟡 The Unluckiest Planeswalker | the +1's Gremlin rides your next Aura spell this turn, not the spending of that specific mana | `OnYourNextSpellMatchingThisTurn` |
| 🟡 Winter's Chill | X isn't capped at your snow lands, and the "pay only {1}: prevent its combat damage" middle option is dropped — each target's controller pays {2} or it's destroyed at end of combat | `CapTargetsAtX` + `UnlessPlayerPays` |
| 🟡 Rimewind Cryomancer | "target activated ability" is a permanent with an ability on the stack and no triggered one there | `HasAbilityOnStack ∧ ¬HasTriggeredAbilityOnStack` |
| ~~Slight Malfunction~~ ✅ | fixed 2026-10-24: the die arm is a reflexive trigger over `CapTargetsAt { LastDieRoll, ApplyToTargets }` — targeted, up to the result | — |
| ~~Critical Hit~~ ✅ | fixed 2026-10-24: triggers on `RolledNaturalMax` gated on a natural face of 20 (`LastRollFaceCount(20)`), so a modified 20 doesn't return it | — |
| 🟡 Resolute Veggiesaur | when one roll of several dice spans your third die, every die of that roll reads the turn's count after the roll, so it can trigger more than once | `Value::DiceRolledThisTurn` |
| 🟡 Xenosquirrels | the counter-for-a-shift choice is the headless pick for every seat (it only ever raises a die into a better arm or onto a named threshold; never lowers) | `StaticEffect::RemoveCounterToShiftDie` |
| 🟡 Bamboozling Beeble | the ignored die is chosen by rule, not asked: an opponent's best face, a friendly roll's worst | `Effect::NextRollExtraDieThisTurn` |
| 🟡 Selective Snare | "the creature type of your choice" is read off the targets: each must share a creature type with the first, not one type common to all | `SharesCreatureTypeWithTargetSlot(0)` |
| 🟡 Illusory Gains | "you control enchanted creature" is an ETB/re-attach trigger holding the steal while attached (Control Magic's shape), not a layer-2 static | `GainControlWhileSourceAttached` |
| ~~Enduring Angel~~ ✅ | fixed 2026-10-27: the funnel's `Transformed` event waits in `pending_cost_events` and dispatches with the action (Neglected Heirloom follows it) | — |
| 🟡 Sol'Kanar the Tainted | the donation's opponent is the engine's ranked hostile opponent, not asked | `GainControl { to: HostileOpponent }` |
| 🟡 Tyrannical Pitlord | the leaves trigger sacrifices the chosen creature by whoever controls it (a stolen one is the thief's sacrifice, where the printed "you" does nothing) | `SacrificePermanent { ChosenPermanentOfSource }` |
| 🟡 Realm Razer | the two printed triggers are one "until it leaves" exile: a Razer gone before its ETB resolves exiles nothing, where the printed wording exiles the lands for good | `ExileUntilSourceLeaves` |
| 🟡 Out of Bounds | assist (another player paying up to {3} of the cost) is not modeled | an `AdditionalCastCost::Assist` with a per-seat payment ask |
| ~~Consecrate Land~~ ✅ | fixed 2026-10-27: `EquipBonus.no_other_auras` sheds every other Aura on the land (CR 704.5m; Anti-Magic Aura shares it) | — |
| 🟡 The End | the name sweep and the refund read the permanent's owner's zones, not "its controller's" (a stolen permanent differs) | an `ExileSameNameAsTarget` keyed on the controller |
| 🟡 The Celestial Toymaker | the attack trigger's face-down pile is face down to no one: a headless defender reads its names when picking | a hidden-pile option in `run_separate_into_piles` |
| 🟡 Phyrexian Portal | the opponent's two piles are face down, but the headless exile pick reads their names | the same hidden-pile option |
| 🟡 Immortal Coil | the Coil exiles the graveyard's top cards per point (not a choice); its state trigger, like every `state_trigger`, re-arms only once the graveyard has been non-empty (ruling 2008-10-01: it re-triggers at once under Platinum Angel) | an `ExileChosenFromGraveyard` cost-free pick |
| 🟡 Venser's Diffusion | "or suspended card" is not offered as a target — nonland permanents only | a suspended-card target leaf |
| 🟡 Moonlit Lamenter / Burdened Stoneback | "remove a counter from this creature" pays only with -1/-1 counters (the kind each enters with); a +1/+1 or other counter can't pay | an any-kind `remove_counter_cost` |
| 🟡 Three Bowls of Porridge | the unchosen mode and its target are picked at resolution (CR 602.2b wants both on activation) | `ChooseUnchosenMode` picking at activation |
| 🟡 Quicksmith Spy | the artifact is chosen as it enters, not targeted, and keeps "{T}: Draw a card" while the Spy is on the battlefield under any controller | an ETB target slot for `RememberPermanentOnSource`; a control-scoped grant |
| 🟡 Graceful Takedown | every enchanted creature you control deals its damage untargeted (the printed "any number of target enchanted creatures" is all of them); the other creature is targeted | an any-number target slot |
| 🟡 Quicksmith Rebel | as Quicksmith Spy: the artifact is chosen as it enters, not targeted | the same ETB target slot |
| 🟡 Liberating Combustion | searches only the library for Chandra, Pyrogenius (the graveyard half is not offered) | a library-and/or-graveyard search |
| 🟡 Magma Pummeler | its "counters removed this way" trigger fires on any +1/+1 counter removal, not only the damage-prevention one | a prevention-sourced removal event |
| 🟡 Turtle Tracks | only the caster searches ("any number of target players" — the caster would target themselves) | an any-number player target slot |
| 🟡 Farrel's Mantle | "assigns no combat damage" also prevents combat damage dealt to the creature (`PreventAllCombatDamageInvolving`, as Ophidian) | an assign-no-combat-damage flag |
| 🟡 Assist (The Crowd Goes Wild, Spellweaver Duo) | no other player is offered the assist payment; the caster pays it all (CR 702.132a — every opponent declining is legal) | a seat-routed payment offer at cast time |
| 🟡 Sacellum Godspeaker | always reveals every power-5+ creature card in hand | a reveal-count choice |
| 🟡 Blessed Alliance | the untap mode untaps one target creature, not "up to two" | a multi-slot mode inside `ChooseModesCast` |
| 🟡 Vedalken Squirrel-Whacker | the d6 exchange is never asked: every seat (a human's too) takes the headless rule — a 1 or 2 traded for a base stat of 5+ — inside `roll_one_die` | a seat-routed ask outside an effect |
| ~~Braided Net~~ ✅ | stale 2026-10-28: `Keyword::CantActivateAbilities` locks mana abilities too (`mana_abilities_lock`, CR 605.1a) | — |
| 🟡 Mindlink Mech | "becomes crewed for the first time each turn" is `once_per_turn` on the crew trigger; the copy's Vehicle subtype is an indefinite `SetArtifactSubtypes` (harmless — the Mech is a Vehicle) | a first-time-this-turn crew event |
| 🟡 Kor Dirge | Kor Dirge's "source of your choice" is the headless first hostile source (`choose_damage_prevention_source`) | a ranked source pick |
| 🟡 Start // Finish | Finish sacrifices its creature on resolution, not as an additional cost (`SplitHalf` carries no additional cost) | an additional-cost field on `SplitHalf` |
| 🟡 Graven Abomination | exiles a card from any opponent's graveyard, not specifically the defending player's | a defending-player graveyard target filter |
| 🟡 Suncleanser | the opponent mode removes their counters but not the "can't get counters" lock | a player-scoped counter lock |
| 🟡 Moonlace | only permanents are offered as targets (not spells) | a spell-color layer |
| ~~Bound by Moonsilver~~ ✅ | fixed 2026-10-27: `StaticEffect::CantTransform { IsHostOfSource }` | — |
| 🟡 Sacred Boon | its +0/+1 counters land as the damage is prevented, not at the beginning of the next end step | a delayed per-prevention payout |
| 🟡 Farmstead | the upkeep "pay {W}{W}: gain 1" trigger rides the Aura (the Aura controller's upkeep), not the enchanted land | a granted-trigger static on `AttachedTo(This)` |
| 🟡 Jester's Scepter | the exiled card paid as the cost is always one that names the target spell (paying with a non-matching card — legal, pointless — isn't offered); the controller can't look at the face-down cards in the client | a free pick of the exiled card as a cost |
| 🟡 Guardian Angel | the "until end of turn, you may pay {1} any time you could cast an instant: prevent the next 1 damage to that permanent or player" rider is missing — only the prevent-X shield resolves | a player-hosted activated ability (an emblem with `activated`, offered by the action enumerator and the bot) |
| 🟡 Monitor Monitor | the {1} reroll is never asked: every seat takes the headless rule (a die in the bottom third, one die per turn) inside `roll_one_die` | a seat-routed ask outside an effect |
| 🟡 Command Performance | {TK}{TK} is state only (no ticket costs exist); the sticker mode puts a name sticker (ability / P-T stickers aren't modelled) | ticket-cost stickers |
| 🟡 Done for the Day | {TK} is state only, and the all-three sticker is a name sticker (as Command Performance) | ticket-cost stickers |
| 🟡 Snowblind | Y caps at the host's PRINTED toughness minus 1, not its current toughness (a pumped or shrunk host is read as printed) | a layer-7 read of the host's toughness that excludes this Aura's own -Y |

## Seated commanders with a printed gap (`01KabMxS` audit, 2026-10-22)

Every seated commander's doc was scanned for approximation markers and each claim checked against the body (stale
docs fixed: Aang, Emrakul, Gut, Long Feng, Baral, Teval; fixed in code: Herigast's granted emerge, Alirios's untap
lock, Basri's exert, Frodo's Ring-bearer must-be-blocked, Reno and Rude sacrificing itself; 2026-10-23: Absolute Virtue, Alpharael, Edgar,
Elenda, Gorm, Olivia, Storrev, Surrak, Tomik, Yosei; 2026-10-23 second: Kang, Mm'menon, Fblthp, Heart of Kiran, The Unagi, Zidane, Balthier and Fran, Minsc's subtype, The Lunar Whale's top-card look — `recent_b/cmdr_seated_gaps.rs`). Still open:

| Card | Missing | Needs |
| --- | --- | --- |
| 🟡 Gogo / Arachne | Gogo's "can't be copied"; the hand look | cosmetic or minor |

## "When you do" payoffs that target too early (CR 603.12, 2026-10-26)

`scripts/audit_when_you_do.py` lists the implemented cards whose printed "When you do, … target …" has no
`Reflexive` / `ReflexiveTrigger` in its factory: the payoff targets as the outer trigger goes on the stack,
before the cost or choice that gates it, and resolves inline (no response window). **80 cards** at the audit;
closed: the Plans (`plan_payoff`), Bolg of the North, Yotia Declares War, Undead Butler; 2026-10-26 second pass:
Armix (73 seats), Ancient Bronze Dragon, Leyline Tyrant, The Fugitive Doctor, Earthbender Ascension (also "target
creature you control", was any creature), Eastfarthing Farmer, The Fire Nation Drill (`MayTap` on `IsSource`: a
tapped Drill can't "do"), Doombot Harbinger (`MayExileSelfThen`), Escape Protocol, Foray of Orcs, Dream Eater, Dáin Ironfoot
(`ResolutionCarry` now carries `LastCreatedToken`), Spellbook Vendor, Blaster Hulk, Curious Forager, Ty Lee, Artful
Acrobat, Faebloom Trick, Killmonger, Meanders Guide, Sandbender Scavengers, Unscrupulous Contractor, Zoraline,
Atraxa's Skitterfang (an absent oil counter no longer "does"; its keyword is the reflexive trigger's mode, picked
as it goes on the stack — `push_reflexive_trigger` now picks modes, CR 603.3c), Edgar's Awakening (was an untargeted pick), General
Traag, Heart-Piercer Manticore, Hurska Sweet-Tooth, Icewrought Sentry (was an untargeted pick), Saheeli, Radiant
Creator, Snaremaster Sprite, Aurora Shifter, Merry Bards, Selfcraft Mechan, Forum Filibuster, and the zero-seat
Anthropede, Boilerbilges Ripper, Cornered Crook, Cyclops Superconductor, Diregraf Horde (was an untargeted pick),
Gastal Blockbuster, Kishla Trawlers (no creature card exiled, no payoff), Mount Velus Manticore, Sample Collector,
Sunfire Torch, Wick's Patrol; a reflexive trigger's
hostile player slot now aims at an opponent at two seats too (Leyline Tyrant hit its own controller); the script now strips reminder text (madness / conspire / casualty) and counts an exert
trigger as the reflexive one;
**Open:** Guide
of Souls 46 (cube pool — frozen; it also grants flying instead of a flying counter and makes the Angel type
end-of-turn, printed indefinite), Earth Rumble and Curse of the Werefox are done (the Werefox's "that creature" is the Role's host — `AttachedTo(LastCreatedToken)`
through the carry); Earth Rumble's "up to one" creature of yours is modelled as one. Kappa Tech-Wrecker done (`Predicate::SourceHasKeywordCounter`). Open: Savai Thundermane and Awaken the Honored Dead (cube — frozen), Miasma Demon (its "up to that many" cap on a
reflexive's slots is untested). Anthropede's "discard a card or pay {2}" still offers only the payment. The bot declines a "may [cost]. When you do" whose payoff has no legal target
(`reflexive_payoff_untargetable`). ⚠ A "may [cost]. When you do" wrapper
changes what `optional_trigger_beneficial` / the may-discard ask can see of the payoff — check the bot's
decline still holds when no target exists before converting one (Armix's discard).

## "Any number of target players / opponents" modelled as one seat or as a fan-out (2026-10-26)

`/tmp`-free check — the oracle text says "any number of (other) target …" and the factory holds no multi-target
construct (`max_targets` / `ApplyToTargets` / `ForEach*Target` / a slot past 0). Closed this pass: Priest of
Forgotten Gods (79 seats), Brago, King Eternal (39 — and it is now a legal target of its own ability, per the
2020-11-10 ruling), Hollow Marauder, Thraben Charm, Officious Interrogation. **Open, by seats:** Command the Dreadhorde 8 (the resolver owns the graveyard walk, `audit_target_zone`'s one exemption);
Iname, Life Aspect 7; Paradoxical Outcome 6; Nethroi, Apex of Death 6 and Reunion of the House 5
(total power 10 or less — needs `SlotsTotalManaValueAtMost`'s power sibling); Ral Zarek, Guest Lecturer 5 (the
one row `audit_target_opponent` still prints, 2-player pool); Portal Manipulator 4; Curse of Surveillance 2;
Chong and Lily, Nomads 2; Frantic Salvage 2; Bone Harvest 2; Tocasia, Dig Site Mentor 1; Graceful Takedown 1;
Turtle Tracks 1; Drafna's Restoration 1; Cabal Conditioning 1; Weaver of Lies / Reward the Faithful / Piper's
Melody / Embolden 0.

## "Target player [acts]" modelled without a player target (2026-10-26)

`scripts/audit_target_player_acts.py` — the printed clause makes a PLAYER the target and has them act
("target player searches their library", "target player exiles a card from their graveyard"), but the body
names no player target, so player hexproof, "becomes the target" and the seat choice at N > 2 are all missing
(CR 115.1). Closed this pass: Fertilid (41 seats — it searched the CONTROLLER's library and put the land on
their own side, with no target at all) and Scrabbling Claws' tap ability. Lonis, Cryptozoologist 54 and the two other
"target **opponent**" unit variants (Fertile Imagination, Liar's Pendulum) synthesized `Player`, so the caster was
a legal choice, and the three "target **player**" ones (Cabal Therapy, Brain Pry, Stain the Mind) declared the
slot only in `primary_target_filter` — so the ACTIVATION / CAST validator took a target the enumerator never
offered, and Cabal Therapy cast with no target at all. **Open, by seats:** Jace, the Perfected Mind 12 (a loyalty ability carries one
target — the Jeska row); Vindictive Lich 5 (its modes target, the sacrifice choice is the engine's); End of the
Hunt 2; Cabal Therapist / Cabal Therapy / Sméagol 1 each (Careful Consideration closed 2026-10-27); eleven more at 0 seats.

## Star Trek Commander precons (2026-10-28)

The three TRC precons (seats 6094-6096, `decks::trek_federation` / `trek_klingon` / `trek_borg`). Open 🟡 rows
(each card plays; the clause named is approximated):

| Card | Printed | Approximation |
|---|---|---|
| 🟡 B'Elanna Torres, Scrapper | "you may reveal the top card" | always revealed (`RevealTopThenIf`); a seat preferring the counters to an artifact draw can't decline |
| 🟡 Guinan, Ancient Advisor | draws "if you put two or more +1/+1 counters" | draws when two or more of your creatures entered this turn — differs only when a counter can't be placed |
| 🟡 Cmdr. Troi, Outgoing Empath | the opponent's free spell is copied | a UI opponent whose target pick parks the cast loses the copy rider (`PlayerMayCastFromHandFree`); headless seats are exact |
| 🟡 Right of Vengeance | "whenever a creature deals combat damage to you" | a creature's damage to you during a combat damage step (a this-turn emblem can't hear the combat path's own `DealsCombatDamageToPlayer`), so a creature's noncombat ping in that step also triggers it |
| 🟡 Kang, Suspicious Captain | goaded "for as long as you control Kang" | held while Kang is on the battlefield (`GoadLasts::WhileSourceOnBattlefield`) — differs only if Kang changes control |
| 🟡 Martok, Hero of the Empire; Galvanizing Tenor | up to X targets | `CapTargetsAt` over a seven-slot ceiling, so an eight-seat game's seventh opponent is the most it can reach |
| 🟡 Reconnaissance Partition | "remove a counter" of any kind | only a charge counter pays it (`remove_counter_cost`) |
| 🟡 Võx, Child of the Collective | undaunted counts opponents as the spell is cast | the count is fixed when the grant resolves (`GrantNextMatchingSpellUndauntedThisTurn`) |
| 🟡 The Borg Queen | the opponent "exiles" cards until a creature | they are revealed (the rest go to the bottom either way; nothing sees them exiled) |
| 🟡 Assimilate (Borg Queen, Gift of Perfection, Assimilate the Wounded, Assimilation Assessor, The Best of Both Worlds) | enters "with a +1/+1 counter" | the counter is placed as it lands, after entering — an ETB that reads its counters sees none |
| 🟡 Shore Leave Planet | "a Robot artifact in addition" is a copy exception | layer-4 additions until end of turn beside a nonlegendary layer-1 copy, so a copy of the copy lacks them |
