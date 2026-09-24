# Incomplete cards — implemented but missing a printed capability

Cards that resolve and play, but whose implementation **drops or approximates a
real-Magic capability** (the canonical example: a card that should be castable
from the graveyard but isn't). Distinct from *blank* cards (see
`audit_stubs.rs`) — these look done.

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
graveyard), Pull from the Grave / Rabid Attack ✅ (were stale rows). Remaining:
Yosei (taps all of that player's permanents instead of "up to five target" —
needs a player-slot-dependent permanent multi-slot).

### 2. No "choose two of four" modal selection (player can't pick modes)
**Sublime Epiphany** now has all five modes real (CounterAbility +
CreateTokenCopyOf). The five STX guild Commands still resolve two fixed default
modes: real fix needs **cast-time** mode choice (CR 601.2b) — the engine resolves
`ChooseN` at resolution, so per-mode targets for arbitrary picks can't be supplied
at cast. Tracked: Silverquill / Lorehold / Witherbloom / Quandrix / Prismari
Commands · Moment of Reckoning · Vanquish the Horde.

### 3. MDFC back faces — **mechanism is fully wired** (`back_face` + `GameAction::CastSpellBack`/`PlayLandBack`; 71 cards use it)
The "engine-wide ⏳" notes on these were stale. Status:
- ✅ **Pestilent Cauldron // Restorative Burst** — back attached; from-hand back-cast test. ⚠ It has **no** transform-cast-from-graveyard rider — the row below says why the claim that used to stand here was wrong on both halves.
- ✅ **Wandering Archaic // Explore the Vastlands** — back wired (`{4}` → add 6 colorless, gain 3 life) + test.
- ✅ **Selfless Glyphweaver // Deadly Vanity** — back wired via new `Effect::EachPlayerKeepsOneSacrificeRest` (each player keeps one creature/PW, sacrifices the rest) + test.
- ✅ **Birgi // Harnfel** — Harnfel back wired (`CardDiscarded` → `ExileTopAndGrantMayPlay { 2 }`), cast from hand as an artifact + test.
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
Birthing Pod & Mox Diamond ({G/P} pip on an activation, land-discard) ·
Vicious Rivalry.

### 7. Whole keyword mechanics unmodeled (each = a cluster)
- **Learn** → modeled as Draw 1 (Reduce // Rubble, Mascot Interpretation, the Lessons cycle, Quandrix Field Trip).
- **Adventure** → half omitted (Callous Sell-Sword).
- **Delve** → cast at full cost (Tasigur, Magmatic Sinkhole).
- **Splice onto Arcane** → omitted (Desperate Ritual).
- **Awaken / sac-Spawn alt-cast** → omitted (Birthing Hulk, Hand of Emrakul, OGW Eldrazi).
- **Channel land cost-reductions / land-search** → dropped (decks/lands.rs cluster).
- **Free-with-commander alt-cost / color-identity** → dropped (Fierce Guardianship, Deflecting Swat, Command Tower, Jeska's Will).
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
| Absolute Virtue | "you have protection from each of your opponents" is modelled as `ControllerHasHexproof` — the can't-be-targeted half only, not the damage or aura halves. |
| Damping Engine | the whole "controls more permanents than each other player" lock, plus its sacrifice-to-ignore rider. |
| Lavinia, Azorius Renegade | "each opponent can't cast noncreature spells with mana value greater than the number of lands **that player** controls" — a per-opponent threshold. |
| Spikeshell Harrier | the speed-comparison clause is dropped (the bounce half is complete). |
| Grim Reminder | not a gap — `Effect::SearchRevealPunishSameNameCasters` is the per-opponent walk, under a name the regex can't see. |

Two residuals the CR 603.2c batch ratchet
(`catalog_registration::every_batched_damage_trigger_fires_once_a_batch`) signs
off by name rather than fixes, both filed here:

| Card | Residual |
| --- | --- |
| Quartzwood Crasher | 🟡 "create an X/X … where X is the amount of damage **those creatures** dealt to that player" wants the batch's *summed* damage, and `Value::TriggerEventAmount` is the one dealer the fire landed on. It keeps the unbatched shape (one token per trampler, each sized by its own damage — the right total across too many bodies) rather than take `once_per_batch` and mint one token of the wrong size. Needs the batch to carry a sum, which the per-attacker walk cannot do: the later attackers' damage is not dealt yet when the first one's trigger is pushed. |
| Magmatic Galleon | 🟡 "Whenever one or more creatures your opponents control are dealt **excess** noncombat damage, create a Treasure token" is not modelled at all — only the ETB 5 damage ships. Needs excess-damage tracking (CR 120.3c), which no primitive carries. |

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

⚠ **Two rows are handed to `audit_incomplete`, not fixed**: the clause is not
modelled at all, so there is no filter to correct.

| Card | Residual |
| --- | --- |
| Rootcoil Creeper | 🟡 two of its three abilities are missing: the graveyard-restricted "add two mana of any one color", and "{G}{U}, {T}, Exile this creature: Return **target card with flashback you own from exile** to your hand". Only the any-colour mana ability ships. |
| Bloodthirsty Adversary | 🟡 the kicker payoff — "exile up to that many target instant and/or sorcery cards with mana value 3 or less from your graveyard and copy them" — is not modelled; only `Multikicker` and the +1/+1 counters ship. |
| Rydia, Summoner of Mist | 🟡 the **Summon** ability ("{X}, {T}: Return target Saga card with mana value X from your graveyard to the battlefield with a finality counter on it") is not modelled; only the landfall loot ships. |

Three approximations the fixes left standing, each documented at its variant:

| Card | Residual |
| --- | --- |
| Scrabbling Claws | 🟡 the first ability prints "**target player** exiles a card from their graveyard" — the *player* is the target and the choice is theirs. It is modelled as "exile target card from a graveyard", now correctly zoned, which is the second ability's wording. |
| Goblin Welder | 🟡 the second target ("target artifact card in **that player's** graveyard") is auto-picked by `Effect::WeldArtifacts` at highest mana value rather than declared as a target — a cross-target constraint no slot can express today. |
| Ambush Wolf / Angel of Serenity | 🟡 "up to one" / "up to three **other**" — the Angel's `other` is now in the filter; the Wolf's "up to one" still fizzles the trigger rather than declining it, which differs only when a graveyard is empty. |

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
| Jackdaw Savior | 🟡 "another" is other than the creature that **died**, not other than the Savior. The two coincide only when the Savior itself dies; no `SelectionRequirement` names the trigger source, so the restriction is dropped. |
| Blade of Shared Souls | 🟡 "another" is other than the creature the Equipment is **attached to**, which is not the source either. Same gap, same missing ref. |
| Etched Slith | 🟡 the whole "when you do, remove a counter from another target permanent or opponent" clause is unmodelled — only the +1/+1 counter ships. |
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
against Scryfall's `color_identity`: **18,058 cards resolved, 15 divergences**,
each a card gap rather than an identity-walk gap. The list in that test is the
ratchet; this is the engineering view of it.

| Card | Gap | What it needs |
|---|---|---|
| Mythos of Nethroi / Illuna / Vadrok / Snapdax | the "if {C}{C} was spent to cast this" half is dropped; each is modelled as its unconditional half only | wire `Predicate::ManaSpentOfColorAtLeast` into the four resolve effects (the predicate exists and the Ravnica "if {R} was spent" cards already use its sibling) |
| Balduvian Fallen | the "+1/+0 for each {R} spent to pay cumulative upkeep" payoff is dropped | a per-color read of what paid a cumulative upkeep; no primitive |
| Tribal Golem | the granted `{B}: Regenerate` (Zombie clause) is dropped, along with four other conditional grants | `StaticEffect` granting an *activated ability* conditionally |
| Archangel of Wrath | `Keyword::Kicker` is a single cost; "Kicker {B} and/or {R}" keeps only one half | a multi-kicker-cost shape (also unblocks the rest of the "and/or" kicker cycle) |
| Branch of Vitu-Ghazi | "add two mana of any one color" is modelled as a fixed `{W}{W}` — wrong mana *and* a white identity the card does not have | `AddManaKeptThisTurn` takes `Vec<Color>`; it wants a `ManaPayload` so `AnyOneColor` can flow through it |
| Callous Sell-Sword | the Burn Together adventure half ({R}) is unimplemented | adventure wiring exists; the half was never written |
| Cruel Somnophage | the Can't Wake Up adventure half ({1}{U}) is unimplemented | as above |
| Augusta, Dean of Order | the Plargg, Dean of Chaos MDFC face ({1}{R}) is unimplemented | MDFC wiring exists; the face was never written |
| Fist of Suns, Leyline of Mutation | "You may pay {W}{U}{B}{R}{G} rather than pay the mana cost for spells you cast" | no primitive for a static alternative cost granted to *other* spells |
| Maraxus of Keld, Bounty Hunter | not card gaps — the Scryfall cache resolves the name to a different card (one Vanguard avatar, one creature) than the catalog holds | nothing; the rows document the collision |

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
| Phyrexian Battleflies | "{B}: …  Activate **no more than twice** each turn" ships as `once_per_turn: true`, i.e. once | `ActivatedAbility::once_per_turn` is a `bool`, and `CardCold::once_per_turn_used` is a `Vec<usize>` of ability indices with no count beside it. A per-turn *limit* wants both widened, plus the wire and serde forms — a primitive, not a card job |
| Arcanum Wings | `sorcery_speed: true` with no printed clause | CR 702.64a — aura swap **is** "Activate only as a sorcery" by rule |
| Squee, the Immortal | same | CR 307.1/302.1 — casting a creature is sorcery-speed; the graveyard/exile permission is not a second one |
| Roadkill Rodney | same, reported in the audit's token bucket | the flag is on the **Mutagen token**, whose own printed text says "Activate only as a sorcery"; a factory holds its tokens' rules text and the card's oracle does not |

Three of the first eleven kept a documented residual rather than shipping whole:

| Card | Residual | Why |
|---|---|---|
| Sage of the Beyond | "spells you cast **from anywhere other than your hand** cost {2} less" is spelled as its two reachable zones — `GraveyardCastCostReduction` + `ExileCastCostReduction`. A cast from the **command zone** is not discounted | there is no "any zone but hand" cost-reduction static; the two that exist cover flashback/retrace/escape/disturb/aftermath and foretell/plot/adventure/impulse, which is every zone the engine actually casts from outside a Commander game |
| Sproutback Trudge | the graveyard recursion is Gravecrawler's shape — pay the cost, `Move` to the battlefield — so it is **not a cast**: nothing counters it and no cast trigger sees it. And "this spell costs {X} less to cast, where X is the amount of life you gained this turn" is **absent** | no `StaticEffect` reduces a card's own cost by a `Value`; the `SelfCostReduced*` family is one variant per counted thing (`…PerDiscardThisTurn`, `…PerSpellCastThisTurn`, …). A `SelfCostReducedByValue` would close this one and generalise the whole family |
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
| Silverquill Lecturer | "creature spells you cast **have demonstrate**" | two pieces: a `StaticEffect::YourCreatureSpellsHave…` in the shape of `YourISSpellsHaveReplicate`, and a `TriggerSource` variant of `Effect::Demonstrate` — the existing one reads `ctx.source` as *the spell on the stack*, so it cannot fire from a permanent's own `SpellCast` trigger |
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
| ~~Mox Diamond~~ ✅ | discarded *any* card, unconditionally, and kept the Mox either way | `MayDiscardMatching { filter: Land, then: Noop, else_: SacrificeSource }` — Drekavac's shape |
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
in for spelled-out text (Frost Titan's ward, Exalted Angel's lifelink,
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
| Wizard's Rockets | {T}, sac: one mana of any colour (free) | {X}, {T}, sac: X mana — modelled at X = 1 ({1}), the documented approximation and the one row the column still lists |

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
Whirling Dervish and Skizzik model an end-step conditional as the event
that would satisfy it (a combat-damage counter; an ETB that schedules
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

Residue, five rows: Whirling Dervish, Skizzik and Lone Rider (the
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
| Augusta, Dean of Order | the same — the untap-and-retap ran once per attacker | `YouAttack` |
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
already names (multi-target "up to N" collapsed to one slot: Conduct
Electricity, Trick Shot, Heartless Act's counters, Secret Tunnel; a
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

Residue one row: Redirect Lightning's "pay 5 life" (the Amounts residue
already names it).

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
for Strict Proctor), a keyword (Lifelink for Exalted Angel and Paladin of
Prahv, Deathtouch for Stinkweed Imp, `SuspendAccelerant` for Deep-Sea
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
Shrieker, Eladamri, Pinnacle Starcage, Rydia, ~~Diviner's Wand~~, Urborg
Panther, Tephraderm's spell half, ~~Skophos Maze-Warden~~, ~~Shieldmage
Elder~~, Iroh, Lumbering Laundry (Tephraderm's spell half, Hauntwoods Shrieker's
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
  `decks/lands.rs`. Re-read 2026-09-07: Mishra's Factory and Blinkmoth Nexus
  are not in the catalog at all; Ghost Quarter and Field of Ruin both ship
  their basic-land search riders (Ghost Quarter's doc comment claimed
  otherwise and is fixed); Thespian's Stage's only residual is the printed
  "except it has this ability" on the copy. Nothing verified open here.
- **Missing creature subtypes bridged to a related type** (cosmetic):
  Dwarf→Warlock, Rhino→Bard, Kor→Warlock, Gorgon→Snake, …

For the full machine-generated list, run the auditor (`--comments-only`).

### Swell the Host (C15, pod seat 32) — open residuals, 2026-09-24

| Card | Gap |
|---|---|
| 🟡 Loaming Shaman | "any number of target cards from target player's graveyard" shuffles the **whole** graveyard back; no pick. Needs a multi-target any-number graveyard selector. |
| 🟡 Forgotten Ancient | "move any number of +1/+1 counters" is all-or-nothing, spread evenly over your other creatures (`DistributeCountersFromSource`); no per-recipient amounts. |
| 🟡 Verdant Confluence | modes are the default picks (two counters, two basics); `ChooseN` has no cast-time mode choice. |
| 🟡 Skullwinder | "choose an opponent" is the engine's pick (fewest creatures). |

### Seats 33, 34, 36, 37 and 40 (Angels SLD, Built From Scratch C14, Vampiric Bloodline VOC, Plunder the Graves C15, Call the Spirits C15) — open residuals, 2026-09-24

| Card | Deck | Gap |
|---|---|---|
| 🟡 Angel of Destiny | Angels (SLD) | "each player this creature attacked this turn" is the last player it attacked. |
| 🟡 Dawnbreak Reclaimer | Angels (SLD) | both graveyard picks are the engine's (the cheapest creature card each way). |
| 🟡 Bitter Feud | Built From Scratch (C14) | the two chosen players are its controller and the engine's pick of opponent. |
| 🟡 Impact Resonance | Built From Scratch (C14) | X is read as the spell resolves, from per-source tallies and each player's largest single hit. |
| 🟡 Volcanic Offering | Built From Scratch (C14) | the choosing opponent is the caster's most hostile one, for both halves. |
| 🟡 Avacyn's Judgment | Vampiric Bloodline (VOC) | no madness (its `{X}{R}` needs an X a madness cast can't choose); a plain 2-damage divider. |
| 🟡 Shadowgrange Archfiend | Vampiric Bloodline (VOC) | no madness ("{2}{B}, Pay 8 life" has a life half a madness cost can't carry). |
| 🟡 Imposing Grandeur | Vampiric Bloodline (VOC) | counts a commander in any zone. |
| 🟡 Predators' Hour | Vampiric Bloodline (VOC) | the stolen card is exiled face up. |
| 🟡 Sandstone Oracle | Call the Spirits (C15) | the chosen opponent is the one with the most cards in hand. |
| 🟡 Hashaton, Scarab's Fist | Eternal Might (DRC) | "except it's a 4/4 black Zombie" adds Zombie rather than replacing the creature types. |
| 🟡 God-Pharaoh's Gift | Eternal Might (DRC) | exiles your greatest-power creature card (no choice); the copy adds Zombie rather than replacing the creature types. |
| 🟡 Rot Hulk | Eternal Might (DRC) | returns your greatest-power Zombie cards, not targets chosen on entry. |
| 🟡 Scaretiller | Land's Wrath (ZNC) | the mode is the engine's: a land from hand when there is one, else the first land card in your graveyard (untargeted). |
| 🟡 The Mending of Dominaria | Land's Wrath (ZNC) | chapters I and II return your greatest-power creature card; the "may" is always taken. |
| 🟡 Trove Warden | Land's Wrath (ZNC) | the exiled cards return when it leaves the battlefield by any route, not only when it dies. |
| 🟡 Kynaios and Tiro of Meletis | Stalwart Unity (C16) | an opponent holding a land who declines to put it onto the battlefield doesn't draw. |
| 🟡 Humble Defector | Stalwart Unity (C16) | the opponent who gains control is random, not targeted. |
| 🟡 Sidar Kondo of Jamuraa | Stalwart Unity (C16) | the evasion covers creatures you control with power 2 or less, not other players' small attackers. |
| 🟡 Orzhov Advokist | Stalwart Unity (C16) | a taker's counters go on their greatest-power creature; the attack restriction covers the creatures they control as it resolves. |
| 🟡 Highcliff Felidar | Raining Cats and Dogs (SLD) | the destructions run one opponent at a time, not simultaneously; the engine picks among tied creatures. |
| 🟡 Jinnie Fay, Jetmir's Second | Raining Cats and Dogs (SLD) | not optional: a creature token is replaced by the bigger Cat or Dog whenever one beats its printed body, and a noncreature token never is. |
| 🟡 Pack Leader | Raining Cats and Dogs (SLD) | the shield covers the Dogs you control as the trigger resolves, not one that arrives later that turn. |
| 🟡 Showdown of the Skalds | Raining Cats and Dogs (SLD) | chapters II and III choose the counter's target as each trigger resolves. |
| 🟡 Cataclysmic Gearhulk | Growing Threat (MOC) | each player keeps their highest-mana-value permanent of each type rather than choosing. |
| 🟡 Filigree Vector | Growing Threat (MOC) | the counters go on every creature and artifact you control rather than on chosen targets. |
| 🟡 Path of the Schemer | Growing Threat (MOC) | the creature card is the greatest-power one among all graveyards. |
| 🟡 Vulpine Harvester | Growing Threat (MOC) | any artifact card in your graveyard may be targeted; the mana-value check runs as the trigger resolves. |
| 🟡 Dromoka's Command | Call for Backup (MOC) | the fight mode targets only your creature; it fights the greatest-power creature you don't control. |
| 🟡 Inscription of Abundance | Call for Backup (MOC) | no kicker (one mode only); the fight mode's second creature is the greatest-power one you don't control. |
| 🟡 Eumidian Wastewaker | World Shaper (EOC) | you and the defending player each discard a card; neither may sacrifice a permanent instead. |
| 🟡 Loamcrafter Faun | World Shaper (EOC) | the cards are targeted as the trigger goes on the stack and capped at the discard count as it resolves. |
| 🟡 Moraug, Fury of Akoum | World Shaper (EOC) | +1/+0 once however many times a creature attacked; the untap rides every later combat this turn. |
| 🟡 Planetary Annihilation | World Shaper (EOC) | each player keeps the engine's pick of six lands. |
| 🟡 Soul of Windgrace | World Shaper (EOC) | the land comes from the first graveyard holding one. |
| 🟡 Emissary of Grudges | Nature's Vengeance (C18) | the opponent is chosen openly, and the reveal redirects any spell that targets you or your permanents, not only the chosen player's. |
| 🟡 Hunting Wilds | Nature's Vengeance (C18) | the animated Forests keep their own color rather than becoming green. |
| 🟡 Charnelhoard Wurm | Nature's Vengeance (C18) | damage to any player fires it, not only an opponent. |
| 🟡 Flameblast Dragon | Nature's Vengeance (C18) | {X} is asked before {R}; a bot seat answers X out of floating mana only. |
| 🟡 Martial Impetus | Peace Offering (BLC) | the attack pump also reaches the enchanted creature. |
| 🟡 Octomancer | Peace Offering (BLC) | the Octopus gift goes to a random opponent rather than a chosen one. |
| 🟡 Perch Protection | Peace Offering (BLC) | the extra-turn gift goes to a random opponent; the life lock lasts this turn only. |
| 🟡 Promise of Loyalty | Peace Offering (BLC) | the attack restriction outlives a removed vow counter. |
| 🟡 Tamiyo, Field Researcher | Peace Offering (BLC) | a targeted opponent's creature draws its own controller the card. |
| 🟡 Fiery Justice | Nature of the Beast (C13) | the 5 life goes to the engine's most hostile opponent. |
| 🟡 Magus of the Arena | Nature of the Beast (C13) | you pick the opponent's creature; the opponent should. |
| 🟡 Naya Soulbeast | Nature of the Beast (C13) | the top cards are read as it enters, not revealed as it is cast. |
| 🟡 Carnelian Orb of Dragonkind | Reign of Dragons (FDC) | its mana gives any creature spell haste, not only a Dragon. |
| 🟡 Goddric, Cloaked Reveler | Reign of Dragons (FDC) | while celebrating it keeps its Human Noble types beside Dragon. |
| 🟡 Leyline Tyrant | Reign of Dragons (FDC) | the dying payment is any mana, not only {R}. |
| 🟡 Thundermane Dragon | Reign of Dragons (FDC) | a creature cast from the top doesn't gain haste; the top card isn't shown to you. |
| 🟡 Battle at the Helvault | Enduring Enchantments (CMM) | "for each player" covers opponents only: you can't exile one of your own permanents. |
| 🟡 Battle for Bretagard | Enduring Enchantments (CMM) | chapter III copies every artifact and creature token you control, duplicate names included. |
| 🟡 Cacophony Unleashed | Enduring Enchantments (CMM) | the animated 6/6 isn't legendary. |
| 🟡 Ghoulish Impetus | Enduring Enchantments (CMM) | the goad is applied on entry and each of your upkeeps, not held by a static, so it lasts until your next turn after the Aura leaves. |
| 🟡 Ondu Spiritdancer | Enduring Enchantments (CMM) | declining the copy still spends the turn's use. |
| 🟡 Saskia the Unyielding | Open Hostility (C16) | "choose a player" is the engine's most hostile opponent (the card allows any player, you included). |
| 🟡 Brutal Hordechief | Open Hostility (C16) | its activated ability makes opponents' creatures block if able, but how they block stays their controllers' choice. |
| 🟡 Mirror Entity | Open Hostility (C16) | "gain all creature types" is a granted Changeling: it reaches type filters and targeting, not type-keyed layer anthems. |
| 🟡 The Mimeoplasm | Devour for Power (CMD) | the engine picks the two cards: it copies the greatest-power creature card in any graveyard and counts the runner-up's power; a `*` power reads as its printed 0. |
| 🟡 Desecrator Hag | Devour for Power (CMD) | a tie for greatest power is broken by graveyard order, not by the player. |
| 🟡 Intet, the Dreamer | Mirror Mastery (CMD) | the card is exiled face up. |
| 🟡 Ray of Command | Mirror Mastery (CMD) | the creature is tapped at the next end step, not as its control returns. |
| 🟡 Hordewing Skaab | Undead Unleashed (MIC) | draws/discards one per opponent dealt combat damage this turn, not only those its Zombies' batch damaged. |
| 🟡 Hour of Eternity | Undead Unleashed (MIC) | the 4/4 black copies are Zombies in addition to their creature types, not instead. |
| 🟡 Shadow Kin | Undead Unleashed (MIC) | copies the greatest-power creature card milled (the engine's pick). |
| 🟡 Rooftop Storm | Undead Unleashed (MIC) | the free cast covers Zombie creature spells cast from hand, not the command zone or other zones. |
| 🟡 Hazel of the Rootbloom | Squirreled Away (BLC) | "tap X untapped tokens" taps every other untapped token you control (X is all of them). |
| 🟡 Hazel's Brewmaster | Squirreled Away (BLC) | your Foods gain the activated abilities of every card exiled with it, not only the creature cards. |
| 🟡 Sword of the Squeak | Squirreled Away (BLC) | "base power or toughness 1" reads printed power and toughness. |
| 🟡 Insatiable Frugivore | Squirreled Away (BLC) | the three cards exiled each time are the first three in your graveyard, not your pick. |
| 🟡 Curse of Inertia | Evasive Maneuvers (C13) | the attacking player's tap-or-untap is the engine's pick of permanent and direction. |
| 🟡 Deluxe Dragster | Divine Convocation (MOC) | the free spell may come from any opponent's graveyard, not only the damaged player's. |
| 🟡 Path of the Ghosthunter | Divine Convocation (MOC) | with no planar deck the Will of the Planeswalkers vote isn't held (planeswalk and chaos would do nothing). |
| 🟡 Joyful Stormsculptor | Divine Convocation (MOC) | battles take no damage (the engine has none). |
| 🟡 Indomitable Might | Virtue and Valor (WOC) | the damage always goes as though unblocked; the controller doesn't choose. |
| 🟡 Mantle of the Ancients | Virtue and Valor (WOC) | the returned Aura and Equipment cards are picked (greatest mana value first), not targeted. |
| 🟡 Unfinished Business | Virtue and Valor (WOC) | the two Aura / Equipment cards are picked, not targeted. |
| 🟡 Retether | Virtue and Valor (WOC) | each Aura's host is the engine's pick (your greatest-power creature first). |
| 🟡 Knickknack Ouphe | Virtue and Valor (WOC) | every eligible Aura goes onto the battlefield; each host is the engine's pick. |
| 🟡 Songbirds' Blessing | Virtue and Valor (WOC) | the Aura always goes onto the battlefield when it has a host; the host is the engine's pick. |
| 🟡 Liberated Livestock | Virtue and Valor (WOC) | each token's Aura is the engine's pick (graveyard first). |
| 🟡 Zinnia, Valley's Voice | Family Matters (BLC) | the granted offspring copy is Zinnia's own trigger (lost if Zinnia leaves first); a creature with its own kicker or offspring gets no second one. |
| 🟡 Echoing Assault | Family Matters (BLC) | one copy per combat, not one per player attacked. |
| 🟡 Combat Celebrant | Family Matters (BLC) | a second exert in a turn is allowed and does nothing (the bonus is once a turn). |
| 🟡 Rose Room Treasurer | Family Matters (BLC) | the {X} is paid from floating mana. |
| 🟡 Atarka Monument | Draconic Destruction (SCD) | animated, it is a colorless Dragon (the printed red and green aren't applied). |
| 🟡 Footbottom Feast | Counterpunch (CMD) | the cards are chosen as it resolves, not targeted on cast; they go on top in mana-value order (the greatest on top). |

### Seats 42, 46, 49, 53 and 55 (Peer Through Time C14, Wade into Battle C15, Chaos Incarnate SCD, Heavenly Inferno and Political Puppets CMD) — open residuals, 2026-09-24

| Card | Deck | Gap |
|---|---|---|
| 🟡 Domineering Will | Peer Through Time (C14) | "target player" is always you; the three creatures are the real targets. |
| 🟡 Intellectual Offering | Peer Through Time (C14) | each "choose an opponent" is the engine's pick (fewest creatures). |
| 🟡 Shaper Parasite | Peer Through Time (C14) | +2/−2 or −2/+2 is chosen as the trigger goes on the stack, not as it resolves. |
| 🟡 Infinite Reflection | Peer Through Time (C14) | the ETB copy also rewrites the enchanted creature itself when it is yours (a copy of itself). |
| 🟡 Dream Pillager | Wade into Battle (C15) | the exiled cards may be played, lands included, not only cast. |
| 🟡 Kardur, Doomscourge | Chaos Incarnate (SCD) | creatures entering after its ETB are goaded by a delayed trigger, not a static rule. |
| 🟡 Theater of Horrors | Chaos Incarnate (SCD) | the permission outlives the enchantment; lands among the exiled cards can't be played. |
| 🟡 Wildfire Devils | Chaos Incarnate (SCD) | the random player gives up their first instant or sorcery in graveyard order. |
| 🟡 Archangel of Strife | Heavenly Inferno (CMD) | war or peace is chosen as its ETB trigger resolves, not as it enters. |
| 🟡 Kaalia of the Vast | Heavenly Inferno (CMD) | also triggers attacking an opponent's planeswalker. |
| 🟡 Jötun Grunt | Political Puppets (CMD) | the graveyard and the two cards of each installment are the engine's pick (an opponent's fullest, highest mana values). |
| 🟡 Ruhan of the Fomori | Political Puppets (CMD) | the random opponent is stored on Ruhan, so another effect storing a player on it overwrites the pick. |

### The `modern_decks` Commander routine's precons (seats 23, 25, 27, 29, 30) — open residuals, 2026-09-24

| Card | Deck | Gap |
|---|---|---|
| 🟡 Ancient Cornucopia | Hatsune Miku (SLD) | "do this only once each turn" limits the trigger, so a declined gain still spends the turn's one. |
| 🟡 Lazotep Quarry | Hatsune Miku (SLD) | the 4/4 black Zombie copy keeps the card's creature types and adds Zombie (the shared eternalize shape). |
| 🟡 Aurora Phoenix | Exit from Exile (CLB) | a spell given cascade by a trigger (Wild-Magic Sorcerer) doesn't carry the keyword, so it doesn't return the Phoenix. |
| 🟡 Chaos Wand | Exit from Exile (CLB) | a found instant or sorcery you don't cast stays in exile instead of going to the bottom. |
| 🟡 Durnan of the Yawning Portal | Exit from Exile (CLB) | exiles the first creature card among the top four (no choice), and the cast from exile has no undaunted. |
| 🟡 Stolen Strategy | Exit from Exile (CLB) | an exiled land may be played, not only spells cast. |
| 🟡 Revival Experiment | Witherbloom Witchcraft (C21) | the engine picks the cards — the highest mana value per permanent type, a multi-typed card counting for the first type it fills. |
| 🟡 Suffer the Past | Witherbloom Witchcraft (C21) | the X cards are chosen as it resolves (from the target player's graveyard), not targeted. |
| 🟡 Gorma, the Gullet | Witherbloom Pestilence (SOC) | the extra +1/+1 counters reach creatures you cast; a nontoken creature put onto the battlefield another way enters without them. |
| 🟡 Stensian Sanguinist | Witherbloom Pestilence (SOC) | "whenever that creature deals combat damage to a player this combat" lasts the turn. |
| 🟡 Breena, the Demagogue | Silverquill Statement (C21) | the two +1/+1 counters go on your greatest-power creature (the engine's pick). |
| 🟡 Author of Shadows | Silverquill Statement (C21) | the castable card is the first nonland card exiled this way, not a chosen one. |
| 🟡 Bold Plagiarist | Silverquill Statement (C21) | copies +1/+1 counters only, and reads the counters' recipient, not who put them. |
| 🟡 Guardian Archon | Silverquill Statement (C21) | protection from the chosen player is hexproof and indestructible on the permanent; your own protection isn't modeled; the choice is the engine's most hostile opponent. |
| 🟡 Inkshield | Silverquill Statement (C21) | the Inklings count the unblocked power attacking you as it resolves, not the damage prevented. |
| 🟡 Nils, Discipline Enforcer | Silverquill Statement (C21) | each player's counter goes on their first creature, chosen rather than targeted. |
| 🟡 Tragic Arrogance | Silverquill Statement (C21) | the engine chooses for the caster: its own best of each type, each opponent's weakest (lowest mana value). |
| 🟡 Victory Chimes | Silverquill Statement (C21) | the mana always goes to you, not a player of your choice. |
| 🟡 Agitator Ant | Blame Game (MKC) | each taker's two counters go on their greatest-power creature, not one they choose. |
| 🟡 Feather, Radiant Arbiter | Blame Game (MKC) | a headless caster only copies onto its own creatures (a person is offered every legal one). |
| 🟡 Immortal Obligation | Blame Game (MKC) | a duty counter put back after the first one left re-arms the goad and the restrictions (CR 611.2b ends them for good). |
| 🟡 Deceptive Frostkite | Temur Roar (TDC) | the copy isn't optional when a creature with power 4 or greater is there to copy. |
| 🟡 Will of the Temur | Temur Roar (TDC) | "if you control a commander as you cast this spell" is read as it resolves. |
| 🟡 Estrid, the Masked | Adaptive Enchantment (C18) | the −7's Auras go on hosts the engine picks. |
| 🟡 Genesis Storm | Adaptive Enchantment (C18) | the revealed permanent always goes onto the battlefield ("you may" isn't offered). |
| 🟡 Myth Unbound | Adaptive Enchantment (C18) | the discount counts both partners' casts from the command zone together. |
| 🟡 Orator of Ojutai | Draconic Domination (C17) | the Dragon check reads your board and hand as it enters; there is no optional reveal. |
| 🟡 Descendants' Fury | Sliver Swarm (CMM) | "one of them" is any attacker of yours that damaged a player this turn, not only this combat-damage batch's. |
| 🟡 Bloodlord of Vaasgoth | Vampiric Bloodlust (C17) | the granted bloodthirst is checked as the cast trigger resolves, not as the creature enters. |
| 🟡 Mathas, Fiend Seeker | Vampiric Bloodlust (C17) | a bounty counter grants its dies trigger only while Mathas is on the battlefield. |
| 🟡 Kheru Mind-Eater | Vampiric Bloodlust (C17) | the exiled card is exiled face up. |
| 🟡 Sylvan Offering | Guided by Nature (C14) | "choose an opponent" is the engine's pick (fewest creatures). |
| 🟡 Siege Behemoth | Guided by Nature (C14) | the per-creature "you may assign as though unblocked" is always yes. |
| 🟡 Havengul Lich | Grave Danger (SCD) | the cast permission lands; the Lich does not gain the cast card's activated abilities. |
| 🟡 Liliana, Untouched by Death | Grave Danger (SCD) | the −3 covers the Zombie cards in your graveyard as it resolves, not later arrivals that turn. |
| 🟡 Arcane Lighthouse | Forged in Stone (C14) | creatures lose hexproof/shroud until end of turn; a grant made later that turn is not stopped ("can't have"). |
| 🟡 Benevolent Offering | Forged in Stone (C14) | each "choose an opponent" is the engine's pick. |
| 🟡 Nahiri, the Lithomancer | Forged in Stone (C14) | the +2 attaches your first Equipment; the −2 puts your first Equipment card from hand, else graveyard — no pick. |
| 🟡 Mystic Confluence | Seize Control (C15) | the modes are the default picks (counter unless {3}, draw two); `ChooseN` has no cast-time mode choice with repeats. |
| 🟡 Collective Effort | Rebellion Rising (ONC) | escalate is paid as it resolves (the standing `Escalate` approximation), tapping your first untapped creature. |
| 🟡 Goldwardens' Gambit | Rebellion Rising (ONC) | each token takes your highest-mana-value unattached Equipment; no pick, and an attached one is never moved. |
| 🟡 Neyali, Suns' Vanguard | Rebellion Rising (ONC) | "tokens attack a player" also counts tokens attacking a planeswalker an opponent controls. |
| 🟡 Divine Reckoning | Feline Ferocity (C17) | each player keeps their highest-mana-value creature (the engine's pick, as Deadly Vanity). |
| 🟡 Stalking Leonin | Feline Ferocity (C17) | the opponent is chosen openly by the engine (the one with the fewest creatures), not secretly by the player. |
| 🟡 Cliffside Rescuer | Primal Genesis (C19) | protection from each opponent is protection from what opponents control (`ProtectionFromMatching(ControlledByOpponent)`). |
| 🟡 Tahngarth, First Mate | Primal Genesis (C19) | it attacks its new controller's default opponent, not a chosen player that opponent is attacking. |
| 🟡 Aeon Chronicler | Entropic Uprising (C16) | no Suspend X: suspend takes no X, and bots suspend only cards with no mana cost, so the time-counter draw never comes up. |
| 🟡 Vial Smasher the Fierce | Entropic Uprising (C16) | the random opponent is always dealt the damage, never one of their planeswalkers. |
| 🟡 Blood Tyrant | Entropic Uprising (C16) | grows by the number of living players, not the life actually lost. |
| 🟡 Star Athlete | Endless Punishment (DSC) | "up to one target" always takes a target when one exists. |
| 🟡 Brudiclad, Telchor Engineer | Exquisite Invention (C18) | the token the others copy is your greatest-power token, not a free choice. |
| 🟡 Prototype Portal | Exquisite Invention (C18) | the imprint takes the first artifact card in hand. |
| 🟡 Tawnos, Urza's Apprentice | Exquisite Invention (C18) | as Strionic Resonator: the target is the ability's source permanent, and the copy keeps its targets. |
| 🟡 Winter, Cynical Opportunist | Death Toll (DSC) | the engine picks the exiled set (the greatest-mana-value permanent card plus the cheapest cards covering four card types); the finality counter is added as the card enters. |
| 🟡 Cemetery Tampering | Death Toll (DSC) | a hidden land is put onto the battlefield rather than played (no land drop used). |
| 🟡 Polluted Cistern // Dim Oubliette | Death Toll (DSC) | Cistern counts milled cards, not every card put into your graveyard from your library (surveil, reveal-until). |
| 🟡 Demonic Covenant | Death Toll (DSC) | the draw also fires when Demons attack only a planeswalker. |
| 🟡 Into the Pit | Death Toll (DSC) | the sacrifice is paid as the cast completes rather than as a cost before it. |
| 🟡 Old Stickfingers | Death Toll (DSC) | reveals until one creature card X times, bottoming each run of misses before the next. |
| 🟡 True-Name Nemesis | Mind Seize (C13) | the chosen player is the engine's most hostile opponent, not the controller's pick. |
| 🟡 Eye of Doom | Mind Seize (C13) | each player's doom counter goes on the nonland permanent the engine picks. |
| 🟡 Gond Gate | 20 Ways to Win (SLD) | "any color a Gate you control could produce" makes any color. |
| 🟡 Indulge // Excess | Cabaretti Cacophony (NCC) | Excess counts creatures you control that dealt damage (combat or not) to a player this turn. |
| 🟡 Killer Service | Cabaretti Cacophony (NCC) | the token sacrificed is the engine's pick. |
| 🟡 Sizzling Soloist | Cabaretti Cacophony (NCC) | "attacks during its controller's next combat phase" is must-attack until your next turn. |
| 🟡 Vivien's Stampede | Cabaretti Cacophony (NCC) | the draw happens at end of combat, not at the next main phase. |
| 🟡 Zurzoth, Chaos Rider | Cabaretti Cacophony (NCC) | the Devils' loot reaches the defending player of the attack, one player per batch. |
| 🟡 Benthic Anomaly | Eldrazi Incursion (M3C) | each opponent's greatest-power creature is chosen, and the copy is of the one with the greatest mana value. |
| 🟡 Bismuth Mindrender | Eldrazi Incursion (M3C) | the exiled card may be cast for life until end of turn, not only as the trigger resolves. |
| 🟡 Selective Obliteration | Eldrazi Incursion (M3C) | each player's color is the one most common among their permanents. |
| 🟡 Twins of Discord | Eldrazi Incursion (M3C) | the granted bloodthirst rides colorless creature spells you cast, not every entry. |
| 🟡 Chandra, Legacy of Fire | Planeswalker Party (CMM) | the 0 removes a loyalty counter from each planeswalker you control with two or more, not "any number of permanents" chosen. |
| 🟡 Guff Rewrites History | Planeswalker Party (CMM) | only opponents' permanents are chosen (never your own); the exiled lands go to the bottom in exile order. |
| 🟡 Leori, Sparktouched Hunter | Planeswalker Party (CMM) | the planeswalker type is the one most common among yours on the battlefield and in hand, not a free choice. |
| 🟡 Narset of the Ancient Way | Planeswalker Party (CMM) | the −2's damage target is chosen as it's activated, not by a reflexive trigger. |
| 🟡 Repeated Reverberation | Planeswalker Party (CMM) | the instant, sorcery and loyalty-ability halves are three separate "next" riders; each can fire. |
| 🟡 Sparkshaper Visionary | Planeswalker Party (CMM) | all or none of your planeswalkers become Birds; they keep their colours and lack the scry trigger. |
| 🟡 Vronos, Masked Inquisitor | Planeswalker Party (CMM) | the +1 phases out every other planeswalker you control, not up to two targets. |
| 🟡 Bell Borca, Spectral Sergeant | Legends' Legacy (DMC) | the noted mana values are every card exiled this turn (through the two exile funnels), including those exiled before Bell Borca entered. |
| 🟡 Bladewing, Deathless Tyrant | Legends' Legacy (DMC) | combat damage to a planeswalker doesn't trigger it. |
| 🟡 The Peregrine Dynamo | Legends' Legacy (DMC) | as Strionic Resonator: the target is the ability's source permanent, and the copy keeps its targets. |
| 🟡 Verrak, Warped Sengir | Legends' Legacy (DMC) | only a fixed life cost counts as life paid (not X or half your life); the copy keeps its targets. |
| 🟡 Dance with Calamity | Tinker Time (MOC) | the exiling stops once the total mana value reaches nine (the engine's stop point), not as many times as the controller chooses. |
| 🟡 Path of the Animist | Tinker Time (MOC) | Will of the Planeswalkers is not voted: outside Planechase planeswalking and chaos do nothing, but "whenever players vote" triggers don't see it. |
| 🟡 Pain Distributor | Tinker Time (MOC) | "that player" is the dying artifact's owner, not its controller. |
| 🟡 Gimbal, Gremlin Prodigy | Tinker Time (MOC) | the trample grant matches printed card types, so an animated artifact doesn't get it (CR 613.8; ENGINE_BACKLOG). |
| 🟡 Mishra, Eminent One | Mishra's Burnished Banner (BRC) | the Warform keeps the copied artifact's name (it is non-legendary, so the legend rule leaves it alone as the rename would). |
| 🟡 Ashnod the Uncaring | Mishra's Burnished Banner (BRC) | the copy finds the ability through its source, so an ability whose source was the sacrificed permanent can't be copied. |
| 🟡 Blast-Furnace Hellkite | Mishra's Burnished Banner (BRC) | "creatures attacking your opponents" also counts creatures attacking an opponent's planeswalker. |
| 🟡 Smelting Vat | Mishra's Burnished Banner (BRC) | each card is capped at the sacrificed artifact's mana value, not the pair's total. |
| 🟡 Lithoform Engine | Mishra's Burnished Banner (BRC) | the ability copy is Strionic Resonator's: the target is the ability's source permanent, and the copy keeps its targets. |
| 🟡 Workshop Elders | Mishra's Burnished Banner (BRC) | the flying grant matches printed card types, so an animated artifact doesn't fly (CR 613.8; ENGINE_BACKLOG). |
| 🟡 Glint Raker | Mishra's Burnished Banner (BRC) | the reveal isn't optional. |
| 🟡 Sanwell, Avenger Ace | Urza's Iron Alliance (BRC) | the cast offer is the first matching card of the six, not a choice; the rest go to the bottom in exile order, not a random one. |
| 🟡 Scholar of New Horizons | Urza's Iron Alliance (BRC) | when the Plains may go onto the battlefield it always does. |
| 🟡 Cosmic Intervention | Phantom Premonition (KHC) | the exile-instead replacement covers the permanents you control as it resolves (not ones that arrive later that turn); the end-step return takes the cards you own that a this-turn "exile it instead" moved. |
| 🟡 Mairsil, the Pretender | Arcane Wizardry (C17) | a borrowed ability can be activated any number of times a turn, not once; the cage takes the highest-mana-value artifact or creature card. |
| 🟡 Magus of the Abyss | Arcane Wizardry (C17) | "target … of their choice" is a choice, not a target: a hexproof creature can still be picked (The Abyss likewise). |
| 🟡 Shifting Shadow | Arcane Wizardry (C17) | reveals from the Aura controller's library; the new creature enters before the old one is destroyed. |
| 🟡 Vindictive Lich | Arcane Wizardry (C17) | always all three modes, in a fixed order (lose five, discard two, sacrifice); a mode with no fresh opponent does nothing. |
| 🟡 Spiked Corridor // Torture Pit | Endless Punishment (DSC) | Torture Pit's +2 also reaches permanents opponents control (the shared `NoncombatDamageToOpponentsBonus`). |
| 🟡 Duneblast | Breed Lethality (C16) | the survivor is the chooser's pick among all creatures, and one always survives when any exist ("up to one" never picks none). |
