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
*text* does not match either — Tempest Angler prints "whenever you cast a
noncreature spell, put a +1/+1 counter on this creature" and ships an ETB
scry 2; Outcaster Trailblazer prints an ETB mana, a power-4 draw trigger and
plot {2}{G}, and ships a "cast a spell with mana value 5+" draw-and-token.
**Neither is visible to any column this audit has**, and both were found only
because a keyword row pointed at the card. That is the next audit: the oracle
text against the effect tree.

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

### Verified-but-overrated (real gaps, but 1v1-equivalent or strictly-better — MED, not HIGH)
| Card | Location | Note |
|---|---|---|
| Spell Queller ✓ | modern.rs:15114 | counters instead of exile-until-LTB (opponent can't recast) |
| Generous Gift ✓ | modern.rs:8226 | victim gets no 3/3 Elephant (strictly stronger) |

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
  effect fires unconditionally (Silverquill Standardbearer, Lorehold Crackleflame,
  Witherbloom Mortislide, Witherbloom Apprentice, …).
- **Optional "may" → mandatory — now has an auditor, and it is bigger than
  ~10.** `scripts/audit_dropped_may.py` diffs every catalog definition against
  the offline Scryfall cache and flags the ones whose oracle says "you may"
  and whose definition carries no optional primitive: **349 of 11,094 cards
  checked** (3,901 synthesized `(b###)` names are skipped — they have no
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
  the primitive.

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
  Hellrider, Bojuka Bog, Tormod's Crypt, Barbed Servitor, Lorehold Apprentice, …
- **Per-N counting flattened to a constant**: Witherbloom per-creature-milled
  lifegain (×3 → flat +1), Manifold Key, Fractal Caller, Prismari Pyroshaper.
- **Manland pump / artifact-creature type / land detail dropped** across
  `decks/lands.rs` (Mishra's Factory, Blinkmoth Nexus, Thespian's Stage, Ghost
  Quarter, Field of Ruin, …).
- **Missing creature subtypes bridged to a related type** (cosmetic):
  Dwarf→Warlock, Rhino→Bard, Kor→Warlock, Gorgon→Snake, …

For the full machine-generated list, run the auditor (`--comments-only`).
