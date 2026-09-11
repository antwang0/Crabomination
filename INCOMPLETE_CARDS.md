# Incomplete cards — implemented but missing a printed capability

Cards that resolve and play, but whose implementation **drops or approximates a
real-Magic capability** (the canonical example: a card that should be castable
from the graveyard but isn't). Distinct from *blank* cards (see
`audit_stubs.rs`) — these look done.

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
   capabilities over 1,695 variants; 2 dead primitives.** See "The other direction" below.

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

**Two dead primitives fall out of the other direction** — implemented
effects nothing constructs, i.e. capability waiting for the card that wanted
it, at no engine cost. (`ExileTopAndMayCastUpToMv` was a third and is
constructed now.)

| Primitive | Resolver | The card shape it is for |
|---|---|---|
| `Effect::AddRadCounters { who, amount }` | `effects/mod.rs` | rad counters (Fallout) |
| `Effect::GrantCastBackFromGraveyard { what }` | `effects/mod.rs` | "you may cast it from your graveyard" |

**Check the encoding caution in TODO before adding a card for one**: whether a
new catalog entry moves `Vocab` decides whether it invalidates the trained
nets, and that question is not answered here.

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
- ✅ **Pestilent Cauldron // Restorative Burst** — back attached; from-hand back-cast test; **and** the transform-cast-from-graveyard rider now works (see below).
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

## Highest-priority single-card gaps

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
| Witch's Cauldron | {T}, sac: gain the toughness, draw | {1}{B}, {T}, sac: gain 1, draw |
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

**Coverage, so nobody re-derives it: 10,270 factories priced.** The gap is
helper-built factories whose helper takes the cost in its own shape
(`zubera("Name", r(), ..)`, `echo_creature("Name", &[generic(3), r()], ..)`) —
**219 real spells over ~40 bespoke per-file helpers**, i.e. 2 % of the catalog
for 40 signatures. The rest of the skips are schemes, Vanguards and tokens,
which print no mana cost by design.

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
  the same day: Fiend Hunter's filter is `ControlledByOpponent` (never its
  own, so the may is moot in 1v1), Mistbreath Elder's bounce is printed
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
