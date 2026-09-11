# Engine backlog

Triaged topically at the sixty-seventh pass — the header below used to ask
for this and nobody had done it. Three changes, all reversible from
`git log -p`:

1. **Shipped rows dropped.** A bullet marked ✅ or struck through is gone
   *unless* its text carries an open residual (`Residual:`, `Remaining`,
   `still`, ⏳, 🟡, …) — 111 such rows were kept in place. 1 070 lines went;
   nothing open was touched, and nothing was summarized away.
2. **Client work moved to `CLIENT_BACKLOG.md`.** ~400 lines of GUI backlog
   interleaved with the engine's. (It *does* build here after four apt
   packages — that file's header has the command.)
3. **Sections reordered into four parts**, below. Section bodies are
   verbatim.

Sizes are the point of the split: this file is the archive, `TODO.md` is
the handoff.

| Part | Section | Lines |
| --- | --- | --- |
| Bugs & robustness | [OPEN 2026-09-11 — the first capped board in ~1.3 M swept games, diagnosed and NOT a rules defect: Beacon of Immortality makes a WG cube mirror unwinnable](#open-2026-09-11--the-first-capped-board-in-13-m-swept-games-diagnosed-and-not-a-rules-defect-beacon-of-immortality-makes-a-wg-cube-mirror-unwinnable) | 42 |
| Bugs & robustness | [FIXED 2026-09-11 (nineteenth find) — twenty-six shipped bodies enumerated ZERO legal targets: a player-only slot 0 classified by its BODY, not by its slot](#fixed-2026-09-11-nineteenth-find--twenty-six-shipped-bodies-enumerated-zero-legal-targets-a-player-only-slot-0-classified-by-its-body-not-by-its-slot) | 41 |
| Bugs & robustness | [FIXED 2026-09-11 (eighteenth find) — "target player's graveyard" was not a player slot: the two walkers disagreed about which selectors read a player](#fixed-2026-09-11-eighteenth-find--target-players-graveyard-was-not-a-player-slot-the-two-walkers-disagreed-about-which-selectors-read-a-player) | 29 |
| Bugs & robustness | [FIXED 2026-09-11 (seventeenth find) — a target slot declared *bare* above slot 0 is unaimable, and five shipped cards declared one](#fixed-2026-09-11-seventeenth-find--a-target-slot-declared-bare-above-slot-0-is-unaimable-and-five-shipped-cards-declared-one) | 36 |
| Bugs & robustness | [FIXED 2026-09-11 (sixteenth find) — an effect that suspends OFF the stack was a silent no-op, and the signal it left behind is not inert](#fixed-2026-09-11-sixteenth-find--an-effect-that-suspends-off-the-stack-was-a-silent-no-op-and-the-signal-it-left-behind-is-not-inert) | 60 |
| Bugs & robustness | [FIXED 2026-09-11 (the plumbing column, worked one by one) — six asks answered by the wrong player, and the column's repeat rows are CLOSED](#fixed-2026-09-11-the-plumbing-column-worked-one-by-one--six-asks-answered-by-the-wrong-player-and-the-columns-repeat-rows-are-closed) | 48 |
| Bugs & robustness | [FIXED 2026-09-11 (fifteenth find) — `MayRepeat`'s loop was abandoned the moment its body suspended: Forbidden Ritual took one permanent of eight](#fixed-2026-09-11-fifteenth-find--mayrepeats-loop-was-abandoned-the-moment-its-body-suspended-forbidden-ritual-took-one-permanent-of-eight) | 37 |
| Bugs & robustness | [FIXED 2026-09-11 (fourteenth find) — a suspended trigger is the SAME trigger, and its dying source's LKI was torn down at the suspend](#fixed-2026-09-11-fourteenth-find--a-suspended-trigger-is-the-same-trigger-and-its-dying-sources-lki-was-torn-down-at-the-suspend) | 45 |
| Bugs & robustness | [FIXED 2026-09-11 (thirteenth find) — six more arms mutated inside the loop that asks, so a suspend did it again](#fixed-2026-09-11-thirteenth-find--six-more-arms-mutated-inside-the-loop-that-asks-so-a-suspend-did-it-again) | 63 |
| Bugs & robustness | [FIXED 2026-09-11 (twelfth find) — a spell's TAIL (the fused right half, the spliced effects) re-ran on every resume: Far // Away took two creatures](#fixed-2026-09-11-twelfth-find--a-spells-tail-the-fused-right-half-the-spliced-effects-re-ran-on-every-resume-far--away-took-two-creatures) | 26 |
| Bugs & robustness | [FIXED 2026-09-11 (eleventh find) — CR 608.2b was re-checked on every resume, so a spell that killed its own target stopped mid-effect](#fixed-2026-09-11-eleventh-find--cr-6082b-was-re-checked-on-every-resume-so-a-spell-that-killed-its-own-target-stopped-mid-effect) | 18 |
| Bugs & robustness | [FIXED 2026-09-11 (what the census found in 28 seconds) — a RESUMED resolution reset its own scratch, so every "from among them" pick after a suspend read an empty set: Bind to Life milled seven and put nothing onto the battlefield](#fixed-2026-09-11-what-the-census-found-in-28-seconds--a-resumed-resolution-reset-its-own-scratch-so-every-from-among-them-pick-after-a-suspend-read-an-empty-set-bind-to-life-milled-seven-and-put-nothing-onto-the-battlefield) | 85 |
| Bugs & robustness | [FIXED 2026-09-11 (the tenth find's production half) — both resume channels leaked: two arms never cleared, no arm clears on an error unwind, and the stash had no guard at all; the RESOLUTION owns them now](#fixed-2026-09-11-the-tenth-finds-production-half--both-resume-channels-leaked-two-arms-never-cleared-no-arm-clears-on-an-error-unwind-and-the-stash-had-no-guard-at-all-the-resolution-owns-them-now) | 70 |
| Bugs & robustness | [FIXED 2026-09-11 (tenth find) — one resolution's leftover answer stranded the next arm's ask for ever: the seed-835 Karn stall](#fixed-2026-09-11-tenth-find--one-resolutions-leftover-answer-stranded-the-next-arms-ask-for-ever-the-seed-835-karn-stall) | 46 |
| Bugs & robustness | [FIXED 2026-09-11 (ninth find) — the same prose sniff one family over: Devour sacrificed the bot's whole board](#fixed-2026-09-11-ninth-find--the-same-prose-sniff-one-family-over-devour-sacrificed-the-bots-whole-board) | 40 |
| Bugs & robustness | [FIXED 2026-09-11 (eighth find) — the bare resolution-time asks: six "up to N" picks resolved as a no-op and every "choose a color" named White](#fixed-2026-09-11-eighth-find--the-bare-resolution-time-asks-six-up-to-n-picks-resolved-as-a-no-op-and-every-choose-a-color-named-white) | 55 |
| Bugs & robustness | [FIXED 2026-09-10 (seventh find) — every "you may pay" / "pay or else" effect paid from the floating pool only, so a bot seat never paid: 149 `MayPay` sites dead in self-play](#fixed-2026-09-10-seventh-find--every-you-may-pay--pay-or-else-effect-paid-from-the-floating-pool-only-so-a-bot-seat-never-paid-149-maypay-sites-dead-in-self-play) | 26 |
| Bugs & robustness | [FIXED 2026-09-10 (sixth find) — the graveyard walk had none of the battlefield walk's rules: no fan-out, no once-per-turn, no intervening-if gate; one step walk, one scope; Attuned Hunter dead on the battlefield](#fixed-2026-09-10-sixth-find--the-graveyard-walk-had-none-of-the-battlefield-walks-rules-no-fan-out-no-once-per-turn-no-intervening-if-gate-one-step-walk-one-scope-attuned-hunter-dead-on-the-battlefield) | 30 |
| Bugs & robustness | [FIXED 2026-09-10 (fifth find) — a cast or activation resumed from a cost-choice prompt returned its events to nobody](#fixed-2026-09-10-fifth-find--a-cast-or-activation-resumed-from-a-cost-choice-prompt-returned-its-events-to-nobody) | 17 |
| Bugs & robustness | [FIXED 2026-09-10 (fourth find) — a cost-paid permanent's own dies trigger stacked below the spell or ability it paid for; Mine Collapse's alternative cost](#fixed-2026-09-10-fourth-find--a-cost-paid-permanents-own-dies-trigger-stacked-below-the-spell-or-ability-it-paid-for-mine-collapses-alternative-cost) | 22 |
| Bugs & robustness | [FIXED 2026-09-10 (third find) — helper-built abilities were unread by every catalog column: 13 cards, a dead Steam Vines half, a dispatcher arm](#fixed-2026-09-10-third-find--helper-built-abilities-were-unread-by-every-catalog-column-13-cards-a-dead-steam-vines-half-a-dispatcher-arm) | 24 |
| Bugs & robustness | [FIXED 2026-09-10 (second find) — an Aura's own `effect:` is its attach and the cast path runs nothing after it; six shipped Auras had a dead entry half](#fixed-2026-09-10-second-find--an-auras-own-effect-is-its-attach-and-the-cast-path-runs-nothing-after-it-six-shipped-auras-had-a-dead-entry-half) | 22 |
| Bugs & robustness | [FIXED 2026-09-10 — step triggers and targeting triggers under `EventScope::EnchantedBySource` never fired: nine shipped Auras](#fixed-2026-09-10--step-triggers-and-targeting-triggers-under-eventscopeenchantedbysource-never-fired-nine-shipped-auras) | 30 |
| Bugs & robustness | [FIXED 2026-09-09 (fourth find) — the CR 732.3 activation guard was reset by the mana ability that paid for the loop, so Basalt Monolith's tap-and-untap ran an `abilarms` game to the action cap](#fixed-2026-09-09-fourth-find--the-cr-7323-activation-guard-was-reset-by-the-mana-ability-that-paid-for-the-loop-so-basalt-monoliths-tap-and-untap-ran-an-abilarms-game-to-the-action-cap) | 40 |
| Bugs & robustness | [FIXED 2026-09-09 (third find) — the sickness pre-check on the bot's mana estimate matched a bare `AddMana`, so a sick Wall of Roots' `Seq`-wrapped counter mana made the `AB_SAC` / `AB_SELF_COUNTER` gates unsound](#fixed-2026-09-09-third-find--the-sickness-pre-check-on-the-bots-mana-estimate-matched-a-bare-addmana-so-a-sick-wall-of-roots-seq-wrapped-counter-mana-made-the-ab_sac--ab_self_counter-gates-unsound) | 32 |
| Bugs & robustness | [FIXED 2026-09-09 (second find) — the bot's mana estimate skipped a summoning-sick creature whole, so a Crystalline Crawler's counter mana made the `AB_DAMAGE` gate unsound](#fixed-2026-09-09-second-find--the-bots-mana-estimate-skipped-a-summoning-sick-creature-whole-so-a-crystalline-crawlers-counter-mana-made-the-ab_damage-gate-unsound) | 30 |
| Bugs & robustness | [FIXED 2026-09-09 — the mandatory-loop watchdog saw only period-1 loops; two Portable Holes and a mandatory Leonin Relic-Warder cycled three boards to the action cap](#fixed-2026-09-09--the-mandatory-loop-watchdog-saw-only-period-1-loops-two-portable-holes-and-a-mandatory-leonin-relic-warder-cycled-three-boards-to-the-action-cap) | 40 |
| Bugs & robustness | [FIXED 2026-09-08 (third run) — a permanent leaving the battlefield by dying or bouncing kept its damage, tap and attachment into its next zone (CR 400.7); a recast Golgari Thug died on entry every turn](#fixed-2026-09-08-third-run--a-permanent-leaving-the-battlefield-by-dying-or-bouncing-kept-its-damage-tap-and-attachment-into-its-next-zone-cr-4007-a-recast-golgari-thug-died-on-entry-every-turn) | 24 |
| Bugs & robustness | [FIXED 2026-09-08 (third run) — a token-doubling board had no bound: the simulator now ends a game whose battlefield passes 1,024 permanents as a cap](#fixed-2026-09-08-third-run--a-token-doubling-board-had-no-bound-the-simulator-now-ends-a-game-whose-battlefield-passes-1024-permanents-as-a-cap) | 26 |
| Bugs & robustness | [FIXED 2026-09-08 (second run) — a combat-damage `dealer_filter` could never say `IsSource`, and "when you next attack this turn" had no primitive](#fixed-2026-09-08-second-run--a-combat-damage-dealer_filter-could-never-say-issource-and-when-you-next-attack-this-turn-had-no-primitive) | 38 |
| Bugs & robustness | [FIXED 2026-09-08 — a trigger grant knew two durations, and a planeswalker never recorded who damaged it](#fixed-2026-09-08--a-trigger-grant-knew-two-durations-and-a-planeswalker-never-recorded-who-damaged-it) | 38 |
| Bugs & robustness | [FIXED 2026-09-07 — keyword grants with an "until your next turn" duration were permanent, and `UntilNextTurn` itself meant "any player's next turn"](#fixed-2026-09-07--keyword-grants-with-an-until-your-next-turn-duration-were-permanent-and-untilnextturn-itself-meant-any-players-next-turn) | 52 |
| Bugs & robustness | [FIXED 2026-09-07 — the LKI walk dropped until-end-of-turn granted triggers, and a conditional Equipment rider granted its abilities unconditionally](#fixed-2026-09-07--the-lki-walk-dropped-until-end-of-turn-granted-triggers-and-a-conditional-equipment-rider-granted-its-abilities-unconditionally) | 44 |
| Bugs & robustness | [FIXED 2026-09-07 — the attack and combat-damage hooks dropped `once_per_turn`](#fixed-2026-09-07--the-attack-and-combat-damage-hooks-dropped-once_per_turn) | 36 |
| Bugs & robustness | [FIXED 2026-09-07 — `triggers_on_equipment` was honoured by two hooks and dropped by the dispatcher](#fixed-2026-09-07--triggers_on_equipment-was-honoured-by-two-hooks-and-dropped-by-the-dispatcher) | 44 |
| Bugs & robustness | [FIXED 2026-09-07 — "leaves the battlefield" fired only on death, and the untap step minted one trigger for a board](#fixed-2026-09-07--leaves-the-battlefield-fired-only-on-death-and-the-untap-step-minted-one-trigger-for-a-board) | 34 |
| Bugs & robustness | [The `debug-assertions` sweep found FIVE real defects, and the committed grid was green on all of them](#the-debug-assertions-sweep-found-five-real-defects-and-the-committed-grid-was-green-on-all-of-them) | 120 |
| Bugs & robustness | [CLOSED — the two stall-sweep leads, and why neither is a bug](#closed--the-two-stall-sweep-leads-and-why-neither-is-a-bug) | 28 |
| Bugs & robustness | [CLOSED — what the seeded cube smoke test left behind (eighty-fifth pass)](#closed--what-the-seeded-cube-smoke-test-left-behind-eighty-fifth-pass) | 72 |
| Bugs & robustness | [Engine correctness audit — 2026-06-11](#engine-correctness-audit--2026-06-11) | 83 |
| Bugs & robustness | [Engine — Robustness / defects: the closed audits and the twenty-three filters](#engine--robustness--defects-the-closed-audits-and-the-twenty-three-filters) | 238 |
| Bugs & robustness | [Decision-plumbing audit (2026-07): bare `decider.decide` sites](#decision-plumbing-audit-2026-07-bare-deciderdecide-sites) | 91 |
| Engine mechanics & primitives | [Engine — Missing Mechanics](#engine--missing-mechanics) | 357 |
| Engine mechanics & primitives | [Discovered engine follow-ups (claude/modern_decks)](#discovered-engine-follow-ups-claudemoderndecks) | 296 |
| Engine mechanics & primitives | [Follow-ups noticed (not yet done)](#follow-ups-noticed-not-yet-done) | 1311 |
| Engine mechanics & primitives | [Suggested next-up tasks](#suggested-next-up-tasks) | 1053 |
| Rules coverage | [MagicCompRules coverage audit](#magiccomprules-coverage-audit) | 312 |
| Tooling | [Recommender: two builder defects fixed, one lesson recorded](#recommender-two-builder-defects-fixed-one-lesson-recorded) | 17 |


# Bugs & robustness

## OPEN 2026-09-11 — the first capped board in ~1.3 M swept games, diagnosed and NOT a rules defect: Beacon of Immortality makes a WG cube mirror unwinnable

`cube` seed **1018**, archetype 0 (`cube WG`), 2 games of 3,200 — the only
`cap` the last three sweep blocks produced (355,200 games). Reproduce with
`target-audit/overflow/bot_ladder --a dflt --b dflt --games 400 --threads 3
--seed 1018 --decks cube`; it is deterministic, and `CRAB_CAP_DIAG=4000` prints
the board.

```text
  cap: 6001 actions, turn 243, stack 0
    p0: life 2147483647 bf 30 hand 5 gy 22 lib 1
    p1: life 2147483641 bf 30 hand 5 gy 22 lib 1
```

Both seats at `i32::MAX`, both libraries at **one card**, for two hundred
turns. The cause is one card in the deck: **Beacon of Immortality** — "double
target player's life total, then shuffle Beacon of Immortality into its owner's
library". It is a mirror, so both seats hold one, and each turn a seat draws its
Beacon (library 0), casts it on itself (library 1 again), and doubles. Thirty-
odd casts saturate the life total; after that the doubling is a no-op, neither
seat can ever deck out, and no amount of damage can kill either. The game is
genuinely unwinnable and the action cap is the backstop doing its job.

**Nothing here is a rules bug**, and the two obvious "fixes" are not fixes:
life saturating instead of overflowing is the robustness filter working (the
`overflow` profile has `overflow-checks = true` and did **not** panic), and the
self-shuffle is the printed card. What is left is two real but separate
questions, both out of scope for a bug fix:

* **Adjudication.** CR 104.4b calls a loop nobody can break a draw; the engine
  reports `cap`. `TODO.md`'s "early adjudication of stalled games via
  `eval_material`" is the lever, and it is a *training-data* decision (it
  changes what the actors record), not a correctness one.
* **Encoding.** A saturated life total is an `i32::MAX` feature going into
  `encode_state_inner` and `eval_material`. Nobody has checked what the net
  does with it. Cheap to check, and the encoding caution in `TODO.md` applies
  to any change that follows.

The bot-strength half — casting a life-doubling spell at maximum life is a
strictly wasted action — is real and small, and is the one arm that would
shorten the game without touching the rules.

## FIXED 2026-09-11 (nineteenth find) — twenty-six shipped bodies enumerated ZERO legal targets: a player-only slot 0 classified by its BODY, not by its slot

The eighteenth find made `target_filter_for_slot(0)` answer `Player` for a
zone selector's `who`. That exposed the next walker in the chain, and this one
is not a census gap — it is the wrong question.

`accepts_player_target()` classifies by what the effect *does*:
`Effect::Move`, `Effect::Attach`, `Effect::PumpPT`, `Effect::GrantKeyword`,
`Effect::BecomeCreature` and friends are "permanent-targeting", which is right
for Regrowth (an `Any`-filtered `Move` must not offer the caster as a player
and fizzle) and wrong for "return all artifacts **target player** owns". Both
of its call sites — the auto-picker and `enumerate_legal_targets` — use it to
drop every `Target::Player` candidate, so those bodies had **no legal target at
all**. Measured, not argued: `enumerate_legal_targets` on a two-player board
returned an empty list for Mudhole, Hurkyl's Recall, Curse of the Pierced Heart
and Arms of Hadar. Uncastable, and nothing logged.

Twenty-six bodies: the seven "enchant player" Curses (Cruel Reality, Curse of
Bloodletting / Death's Hold / Exhaustion / the Pierced Heart, Grievous Wound,
Psychic Possession), six `Move`s (Drafna's Restoration, Hurkyl's Recall,
Mudhole, Raven Guild Master, River's Rebuke, Tormod's Cryptkeeper), and
Jolrael, Incite War, Primeval Light, Instigator, Marsh Casualties, Ink-Eyes,
Arms of Hadar, Corrosion, Equipoise, Jace the Mind Sculptor's −1, Mogg
Infestation, Practiced Offense and Tsabo's Decree.

The fix is at the two call sites, not in the classifier: both already hold slot
0's own filter (`req`), so both now OR `SelectionRequirement::is_player_only`
into the body's classification — **no extra walk**. `is_player_only` is the
narrow half of `can_match_player` (which answers "could a player match" and so
says yes to `Any`); `can_match_player` was missing `OpponentPlayer` /
`YouPlayer` and defers to it now.

Gate: `core_rules::target_walkers::a_player_only_slot_zero_enumerates_a_player`
asserts the end-to-end behaviour — every body whose slot 0 is a *bare* player
face enumerates a player on a two-player board — over a population of 566, with
the floor asserted beside the finding list. It is end-to-end deliberately: the
walker internals are what was fooled. A conjunct condition
(`Player.and(CastSorceryThisTurn)` — Backdraft, Fire and Brimstone, Wicked
Akuba) is legitimately unsatisfiable on a bare board and is out of the
population.

## FIXED 2026-09-11 (eighteenth find) — "target player's graveyard" was not a player slot: the two walkers disagreed about which selectors read a player

The `requires_target` / `target_filter_for_slot` pair again, and this time the
drift is one list written twice. `requires_target`'s selector census walks
`TopOfLibrary` / `BottomOfLibrary` / `CardsInZone` / `ControlledBy` / `Player`
for a `PlayerRef::Target`; the slot-filter walk had arms for only the last two.

So `Effect::Move { what: CardsInZone { who: Target(0), zone: Graveyard, filter:
Land } }` reported **no** slot-0 filter, `primary_target_filter` fell through to
its own subject walk, and that returned `CardsInZone`'s **card** filter — so
**Mudhole** ("exile all land cards from target player's graveyard") offered a
*land permanent* for its player slot and **Drafna's Restoration** an artifact.
A pick the `PlayerRef` cannot resolve: `resolve_selector` gives nothing and the
spell does nothing at all. Exactly the shape the file already records for
Feedback Bolt and Reins of Power, one selector family over.

The fix is the list, not the arms: `selector_player_ref(&Selector) ->
Option<&PlayerRef>` is written once and both walks read it, so a new
player-carrying selector joins both censuses at the same time. Value-identical
for the two arms it replaced (`IMPLICIT_PLAYER_TARGET` *is*
`SelectionRequirement::Player`).

Test: `core_rules::target_walkers::a_target_players_zone_is_a_player_slot`.
**What did not catch it**, and this is the useful half:
`every_reachable_target_player_is_visible_to_the_player_gate` censuses
`Selector::Player(PlayerRef::Target(_))` only, so a player reached through a
zone selector's `who` was invisible to it. Widening that census to any `who`
holding a bare `Target` is the next move on this family (NEXT).

## FIXED 2026-09-11 (seventeenth find) — a target slot declared *bare* above slot 0 is unaimable, and five shipped cards declared one

`core_rules::target_walkers` holds "every `Selector::TargetFiltered` slot is
answerable" at 0 and `unbound_target_slots` holds "every such slot is bound" at
0 — but **both census `TargetFiltered` only**, so a slot written as a bare
`Selector::Target(n)` was invisible to the pair. Above slot 0 that is fatal by
construction: `auto_extra_distinct_slot_targets` breaks the moment
`target_filter_for_slot(slot)` is `None`, and `primary_target_filter`'s
fallback only ever answers slot 0. The slot resolves to nothing, a `Value` over
it reads 0, and nothing is logged. (Slot 0 itself is *correct* bare — "any
target", 414 shipped cards.)

The new gate (`every_bare_target_slot_above_zero_is_aimable`) found five cards
in three shapes:

* **A bare `Target(1)` where the printed card names a filter.** Shifting
  Borders' second land, Hunter's Edge's victim ("target creature you don't
  control" — and slot 0's "creature you control" was bare too), Prismari
  Tideflame (b171)'s "target opponent". Each is one `Selector::TargetFiltered`.
* **A body under `Effect::ApplyToTargets` reading `Target(1..3)`.** That effect
  rebinds `ctx.targets` to **one** element per iteration, so every slot above 0
  inside it is dead by construction — and slot 0 inside it is whatever the
  wrapper's own filter picked, not what the author meant. **Curse of the
  Werefox** fought *with* the opponent's creature and *against* nothing;
  **Urgent Necropsy** ran one mode of four (the artifact), because its three
  other `ApplyToTargets` wrappers destroyed `Target(1..3)`, which was empty.
  Both are `Effect::OptionalTargets` now — it declares declinable slots and
  runs its body on the enclosing target list rather than rebinding it. Urgent
  Necropsy takes `min: 0` as the approximation of "choose one or more"; the
  engine has no at-least-one modal target rule.

The gate skips the two shapes whose slots are not the root's to fill: a
kicked-only branch (asked for again with `kicked = true`) and a `Reflexive` /
`ReflexiveTrigger` body (CR 603.7, targeted at push time) — the same
`RESOLUTION_TIME_TARGETING` list the neighbouring gates use.
## FIXED 2026-09-11 (sixteenth find) — an effect that suspends OFF the stack was a silent no-op, and the signal it left behind is not inert

The class's last shape, and the first where the suspension had nowhere to go
rather than the wrong thing in it.

A suspend only means something where something above can park the continuation
and hand the answer back — a stack item, through `pending_decision`. A card
effect resolved off that path sets `suspend_signal` into a field nobody there
reads, returns, and never runs the rest of the body. For a `wants_ui` seat —
every seat the training actors run — the whole body is a NO-OP. And the signal
stays set: the next `continue_*_resolution` takes whatever is in the field, so a
stray ask surfaces inside an unrelated spell carrying the wrong continuation.

`GameState::resolve_effect_driven` answers those asks through the installed
decider instead — the same answer the same decider gives a seat without
`wants_ui` — and clears the field. It is safe to call anywhere: at
`resolution_depth > 0` there IS something above that can park a continuation, so
it leaves the signal to propagate and behaves exactly like `resolve_effect`.
A real UI prompt would be better and needs the call site to be able to park a
continuation; this is the floor.

`game::answer_log_tests::an_off_stack_ask_is_driven_instead_of_dropped` resolves
one asking effect off the stack twice and asserts both halves directly.

**The population, all card-defined effects that can contain any arm:**

| site | what it runs |
| --- | --- |
| `draw_one`'s `next_draw_replacements` | the Words cycle (CR 614) — **Words of Wind returned nothing at all** |
| `draw_one`'s three draw-diggers | `LookPickToHand` (Tomorrow, Azami's Familiar), `Search`, `RevealUntilFind` — each a no-op, and the draw it replaced did not happen either |
| `apply_as_enters_effect` | 40 shipped cards, routinely an ask (`NameCreatureType`, `ChooseColorForSelf`, `ChooseBasicLandTypeForSource`, `SacrificeAnyNumber`, Devour) |
| the Shapeshifter copy pick | `BecomeCopyOf` off a replacement |
| `movement.rs`'s shield rider | a reflexive trigger on a prevention shield |
| `stack.rs`'s Gift | the gifted half as the permanent enters |
| `mod.rs`'s fuse payoff, Gemstone Caverns' extra, the CR 605.4a triggered mana ability | |

Every `as_enters_effect` card today enters through a spell resolution, so that
site is the guard's no-op case in practice — and stops being one the moment a
permanent enters off a resolution (a Leyline starting on the battlefield, a
put-onto-the-battlefield outside a resolution).

### The second defect underneath it: a seat loop asking through the SINGLE-slot channel

Driving the Words body is what made this visible.
`Effect::PlayerReturnsPermanentsToHand` loops over SEATS and asked through
`ask_seat_cards`, whose channel is `stashed_resolution_answer` — one slot, no
cursor. Seat 0's ask ran again on every one of seat 1's resumes, swallowed the
answer seat 1 had just given (none of whose cards are seat 0's), and the
forced-pick shortfall auto-filled it: **seat 0 bounced another of its own
permanents per round trip, and with three it emptied its board.** `Words of
Wind` is the shipped `EachPlayer` case; Silverquill Command and Mono-white's
sweep use `EachOpponent`, one seat in a duel, and were safe by accident.

`ask_seat_cards_logged` (cursor-indexed) gives each seat its own slot, and the
moves move after every ask. **The rule the audit should carry: a loop over seats
may not ask through the single-slot channel.** Two other sites use it in a loop
and are safe only because their `who` resolves to one player today —
`ShuffleGraveyardCardsIntoLibrary` (`You` / `Target(0)` in all six callers) and
`MayRepeat`'s own repeat question, whose continuation consumes the slot itself.

## FIXED 2026-09-11 (the plumbing column, worked one by one) — six asks answered by the wrong player, and the column's repeat rows are CLOSED

`audit_decision_plumbing`'s "gates a loop repetition" rows, taken after
`MayRepeat` turned out to be a live defect rather than a cosmetic one.

* **Mind Bomb** (`EachPlayerMayDiscardUpToThenDamage`) — "each player may
  discard up to three cards; this deals 3 minus that to them". Routed through
  the RESOLVER's decider, whose headless default for a `min 0` `ChooseCards` is
  the empty pick: nobody ever discarded and the card always dealt its full
  three. A `wants_ui` seat was never offered the trade at all.
* **Borderland Explorer** (`EachPlayerMayDiscardThenTutorBasic`) — same, for
  "each player may discard a card to search for a basic land".
* **Wandering Archaic** (`CopySpellUnlessPaid`) — the {2} is the CASTER's to
  decline and the ask went to the resolving seat's decider; **and** the payment
  was `pool.pay`, the seventh find's floating-pool-only bug still live at this
  one site, so even a "yes" could not pay with lands.

All three also had the thirteenth find's shape (mutations inside the ask loop)
and take the two-pass split. The headless defaults are unchanged, so no headless
outcome moves; what changes is that a seat that suspends is asked, and answers
for itself.

**And then the other three, which closed the column: `audit_decision_plumbing`
reads 0 "gates a loop repetition" rows now, from 7.** Each asks about doing the
round AGAIN, so the round's own mutation is before the ask and cannot be moved
after it — the two-pass recipe does not apply. The accounting that does:

> **One logged answer is one round already performed.** The arm suspends before
> its answer exists, so at entry the channel holds exactly as many answers as
> there are rounds behind it. Replay them in place, skip those rounds, and let
> the newest answer pay for the round this pass performs.

* **Trade Secrets** — the repeat belongs to the OPPONENT ("then that player may
  repeat this process") and the resolving seat's decider was answering it. The
  first draft of the fix drew a round and *then* read the "no"; a decline has to
  replay as a decline and stop before drawing anything.
* **Kindle the Carnage** — the controller's repeat, never asked. Skipping the
  replayed rounds also keeps the RNG that picks the random discard from being
  re-drawn. One prompt is gone on purpose: an empty hand makes the next round a
  no-op, so the arm breaks instead of posing it.
* **Tainted Pact** (`ExileUntilDuplicateName`) — needed one more thing, because
  a local `Vec<String>` of the names it had exiled cannot survive the re-run and
  the cards sit in a zone with everyone else's. They carry `exiled_with =
  source` now — which is what the printed "exiled this way" means — and both the
  seen-names list and the round's own card are read back off the zone.

### The last answer kind without a seat-routed helper: `Target`

Two sites had no better option than a bare `decider.decide` because there was no
cursor-indexed `Target` ask, so the resolving seat's policy answered a choice the
card gives to someone else. `ask_seat_target_logged` +
`PendingEffectState::SeatTargetAnswerPending` close that, on the LOG rather than
the single slot (`scripts/audit_stash_in_loop.py`'s rule).

* **Cuombajj Witches** (`OpponentChoosesTargetForDamage`) — "an opponent chooses
  a target" for the second point, and the CONTROLLER's decider was choosing
  where it landed: the one thing the card exists to prevent.
* **Will of the Council** (`run_council_card_vote`, CR 701.31 — Custodi Squire
  and Custodi Lich) — every player votes and the controller's decider cast every
  ballot. Routed per voter, through the CR 701.38 vote-control grant that
  `asked` already computed and nothing consumed. The test is two candidates and
  two UI seats: one ballot each, and the tie moves BOTH, which is only
  observable if the ballots came from two seats.

Then the rest of the named-chooser rows, with the same helper:

* **Grenzo's Rebuttal** (`EachPlayerDestroysChosenFromLeftNeighbor`) — EACH
  player strips their left-hand neighbour and the resolving seat's decider made
  every pick. Per seat now, cursor-indexed (a loop over seats); the destroys
  were already after the loop and `doomed` rebuilds identically from the
  replayed answers, so no seat's legal set moves on a re-run.
* **Blight** (CR 701.68a) — the chooser was right (the controller) and only the
  prompt was missing.

**What is left in the bare column is 62 sites** (from 74 at the start of the
run), 15 with a degenerate headless default, **0 DEAD and 0 repeat**. They are
mode / colour / die-roll picks whose default is a real choice rather than a
no-op, plus one named-chooser row deliberately NOT taken:

* `ChangeTargetOfAbility` (Reroute) — the chooser is the controller, so it is a
  missing prompt rather than a wrong player, and it WRITES the new target
  between its two asks. Routing it needs the two-pass split first (collect the
  primary and every additional slot, then write them all), or the second ask's
  `legal` set shifts under the first ask's write and the cursor desyncs. That
  gate-stability rule is the same one that decides between the two recipes
  everywhere else in this section.

## FIXED 2026-09-11 (fifteenth find) — `MayRepeat`'s loop was abandoned the moment its body suspended: Forbidden Ritual took one permanent of eight

Same class again, one layer out: a body that suspends carries only its OWN
effect, so an arm that was iterating around it never iterates again. The twelfth
find was the spell's tail, this is a loop.

`Effect::MayRepeat` is "run the body, ask, run it again, …", and exactly one
shipped card builds it: **Forbidden Ritual** ("sacrifice another nontoken
permanent; repeat, up to eight"). Its body is a `MaySacrifice` whose pick
suspends for a `wants_ui` seat — every seat the training actors run
(`build_match_template` sets it on both) — so the loop was dropped after the
first body and the card sacrificed **one** permanent of eight, every time. The
`UnlessPlayerPays` ward it feeds charged the opponent once instead of up to
eight times. Four bears in, one bear out.

Two changes, both in the arm:

* the outstanding repetitions are spliced in behind the body's own continuation
  (`SearchUpToN`'s shape) as `Seq[tail, MayDo{repeat what is left}]`.
  `MayRepeat`'s own first iteration is unconditional, so a bare `MayRepeat`
  continuation would run one body for free; wrapping it in `MayDo` puts the
  question back in front of it, which is what the loop does between iterations
  anyway.
* the repeat question is routed to the seat. It went straight to
  `decider.decide`, so a `wants_ui` controller was **never asked** and the
  `AutoDecider` answered for them — one of the seven "gates a loop repetition"
  rows in `audit_decision_plumbing`'s bare column. The suspend carries
  `MayDoAnswerPending` and the same continuation, so iterations already run are
  not run again.

`MayDo`'s ask is inlined rather than delegated so the loop stays a loop:
`run_effect`'s frame is fat enough that `max` nested ones are a stack risk,
which is why `SearchUpToN` iterates too.

**The remaining six rows of that plumbing column are the same shape waiting to
be checked** — an ask that gates a loop repetition and goes straight to the
decider. This one turned out to be a live defect; the others have not been
taken on the suspending path.

## FIXED 2026-09-11 (fourteenth find) — a suspended trigger is the SAME trigger, and its dying source's LKI was torn down at the suspend

Third instance at a third layer of the class this branch keeps finding: state
scoped to "this resolution" is dismantled when the resolution suspends, so the
continuation runs without it. The scratch was the first (`f6bad006`), CR
608.2b's target-legality check the second (`c2cb8c30`), and the spell's tail the
twelfth find; this is CR 603.10's leaves-battlefield LKI.

`resolve_trigger` arms `resolving_lki_source` / `resolving_lki_subject` and
drops the `leaves_bf_lki` entries when the body returns — **including when it
returned only because it suspended**. A "when this dies" body that asks anything
therefore re-derives, on the resume, against a board with no dying object in it:

* **Giant Albatross.** `DestroyEachUnlessPaysLife` filters on
  `DealtDamageToSourceThisTurn`, which reads the dead source's own damage log
  off its LKI. First pass: two victims, ask the first, suspend. Resume: LKI
  gone, filter matches nothing, empty victim list — **the creatures that killed
  the Albatross walk away free, and the card does nothing at all** for every
  seat that suspends. The headless test covering it passes either way.

The fix keeps the entries across the suspend and lets whichever pass finishes
without suspending remove them; the scoping FLAGS stay per-pass (dropped at the
suspend, re-armed by `submit_decision`'s `ResumeContext::Trigger` arm) because
between the two passes nothing of this resolution is running and the flags must
not colour what is. Nothing else resolves while a decision is pending, which is
what makes that safe.

**The neighbouring ambient state was audited at the same time and is clean, for
reasons worth keeping:**

* `accepting_player` (`AnyPlayerMayAccept`) is restored the moment the body
  returns, suspended or not — which would hand the continuation
  `AcceptingPlayer = None` and let Worms of the Earth destroy itself without
  the two lands. It does not, because `Effect::Sacrifice` writes
  `PlayerRef::Seat(q)` into its own continuation (`per_seat_continuation`)
  before suspending, so the re-run never consults the ambient state. Pinned by
  `drk::worms_of_the_earth_still_costs_two_lands_when_the_sacrifice_suspends`.
  A future `accepted` body that suspends WITHOUT concretizing needs the seat
  carried, not the restore.
* `separated_piles` (`run_separate_into_piles`) has the identical shape and its
  doc already requires its bodies to stay non-interactive.
* All 13 suspend sites inside a loop build a `rest` continuation for the seats
  or items they have not reached (`per_seat_continuation` and friends), so the
  "a suspend drops the rest of the loop" shape does not exist today.

## FIXED 2026-09-11 (thirteenth find) — six more arms mutated inside the loop that asks, so a suspend did it again

The Fade Away commit's class, worked to the bottom of the census. An arm that
suspends re-runs from the top, so any mutation before the ask it suspended on
happens a second time: correct headless, wrong for every seat that suspends
(which is every training seat — `build_match_template` sets `wants_ui` on both),
and invisible to a suite whose tests are all headless. Two shapes, because one
recipe does not fit both.

**Two passes — ask everything, then mutate — where the asks are independent:**

| arm | card | what repeated |
| --- | --- | --- |
| `DestroyEachUnlessPaysLife` | Giant Albatross | the life each earlier victim's controller paid |
| `RevealTopPayOrTake` | Sword-Point Diplomacy | the life per denial: three denials cost 18, not 9 |
| `RevealHandDiscardMatchingUnlessPayLife` | Sirocco | the life, and the discards |
| `AnteTopOfLibrary` | Rebirth (the only multi-seat ante, `ante_only`) | the ante and the `then` it ran |

Unlike the mana sibling, the life arms lose **nothing** to the split: a `budget`
vector carries what each seat has already committed, so CR 118.6 / 119.4 see the
same running totals the single-pass loop saw and no prompt moves. Sirocco's
pass 2 does one thing per hit in the same order, so even its event stream is
unchanged; Sword-Point's `LifeLost` events move from interleaved with the asks
to all together before the cards change zones.

**Skip what an earlier pass already spent**, where the next ask exists only
because the previous payment succeeded and no split is possible:

| arm | card | what repeated |
| --- | --- | --- |
| `MayPayRepeatedly` | Magnetic Mountain, Dream Tides | the mana, **quadratically** — with `k` suspends iteration 1's cost was paid `k` times |
| `CoinFlipDestroyLoop` | Crooked Scales | the repeat cost, and the coin |

`answer_already_acted_on(cursor)` is the test and it is **"something was logged
after this answer"**: only the arm continuing past an answer can log anything
after it, and the LAST entry is the one this pass was resumed with, still owed.
Off by one there skips the payment the seat has just agreed to. It is sound only
because both arms ask unconditionally — a gate before an ask would re-evaluate
against the already-mutated state and shift which slot holds which question,
which is exactly why the other four take the split.

Crooked Scales also flips **before** its ask, and a flip cannot be moved after
the ask it is about. `flip_one_coin_logged` gives the flip a slot in the replay
log beside the answers (`[flip, answer, flip, …]`, one shared cursor), so a
resume reads the result back instead of drawing a second one — CR 705.1, one
flip is one event. Before it, a suspending controller re-rolled every flip it
had been told about and a re-run that won destroyed a creature nobody had been
asked about.

**The four the census called "probably benign" are now benign with a reason**,
checked rather than assumed: `PlayersMayAccept`, `AnyPlayerMayAccept` and
`AnyPlayerMayExileFromGraveyard` return out of the loop on the first acceptor,
so nothing they mutate precedes a later ask; `OtherPlayerMayPayToCounter`'s
failed payment is rolled back by `try_pay_after_snapshot_mode`'s snapshot (pool
and tapped flags) and a successful one returns. `scripts/audit_answer_log.py`
reads **63 arms / 6 suspicious**, down from 10, and every remaining MID is one
of those tail calls or a terminal `break` the static walk cannot see.

Each fix has a regression test on the **suspending** path, and every one of them
fails without its fix.

## FIXED 2026-09-11 (twelfth find) — a spell's TAIL (the fused right half, the spliced effects) re-ran on every resume: Far // Away took two creatures

`continue_spell_resolution` resolves the main effect, then — in the same
function, below it — a fused split's right half (CR 709 / 702.102) and each
spliced effect (CR 702.47b). Neither was gated on which pass this is, so a
suspension **anywhere in the spell** replayed the whole tail every time the
answer came back.

Far // Away fused bounces a creature (Far) and makes its controller sacrifice
one (Away). Away's sacrifice pick suspends for a `wants_ui` seat — every bot
seat, every UI seat — and the resume re-entered the fused-right block:
**two creatures sacrificed, not one** (`core_rules::cr_recent103::
a_fused_split_resolves_its_right_half_once`, three creatures down to zero
without the fix). The mirror case is a left half that suspends: the right half
then ran once *before* the answer and again after it. Fifteen shipped cards
carry `fuse: true`; every spliced cast has the same shape with `k + 1`
resolutions of each spliced effect.

`ResumeContext::Spell` carries a `tail_stage` now — 0 the main effect, 1 the
fused right half, 2 + i spliced effect `i` — and the continuation runs in
**that part's own context** (the right half reads `additional_targets[0]`, not
the main `target`), which the old shape could not do at all: a right half that
suspended resumed with the left half's target. `#[serde(default)]`, so a
snapshot taken before it resumes at stage 0 as before.

## FIXED 2026-09-11 (eleventh find) — CR 608.2b was re-checked on every resume, so a spell that killed its own target stopped mid-effect

`continue_spell_resolution` ran the CR 608.2b target-legality fizzle on the
resume pass as well as the initial one. A resume pass is the same resolution
continuing after a player choice, and CR 608.2b is checked **once, as the spell
begins to resolve**. Lash Out deals 3 to a creature, clashes, and suspends for
the clash question; the creature is in the graveyard by the time the answer
arrives, so the re-entry fizzled the spell — no clash, no reveal, no on-win
damage. Only `wants_ui` seats reach it (every bot seat, every UI seat); the
synchronous `AutoDecider` path never suspends, which is why the shipped Lash
Out test was green — the eighth find's lesson a third time.

Both fizzle blocks are gated on `is_initial_pass` now. Test:
`core_rules::cr_recent103::a_resumed_spell_does_not_fizzle_on_the_target_it_killed_itself`
(fails without the gate).

**Same function, same family — FIXED beside it, see the twelfth find below:**
the fused-split second pass and the spliced effects sit BELOW the main
`resolve_effect` in the same function and were replayed on every resume.

## FIXED 2026-09-11 (what the census found in 28 seconds) — a RESUMED resolution reset its own scratch, so every "from among them" pick after a suspend read an empty set: Bind to Life milled seven and put nothing onto the battlefield

**The instrument paid for itself on its first sweep cell.** `--decks cube --seed
890` under `CRAB_ANSWER_LOG=strict` aborted with

```text
answer-log leak: 0 logged + 1 stashed answer(s) left by Bind to Life / MoveChosen
  resolve_effect_into <- continue_spell_resolution <- submit_decision_inner
```

and `--decks all --seed 890` gave the identical line. An unclaimed *stashed*
answer means the arm never reached its ask on the re-run, and the reason is one
line at the top of `resolve_effect_into`: it resets the per-resolution scratch.

**A resumed resolution is the same resolution.** `submit_decision` →
`continue_spell_resolution` re-enters `resolve_effect_into` with the suspended
arm's effect, and that call cleared `last_moved_cards`, `last_created_token(s)`,
`named_card_this_resolution`, every per-resolution tally — the state the *first*
pass built. Bind to Life is "Mill seven cards. Then put a creature card **from
among them** onto the battlefield": the pick is `Selector::LastMoved` over the
seven just-milled cards, so after the suspend `ids` came back **empty**, the arm
returned on `if ids.is_empty()`, and the card milled seven and put nothing onto
the battlefield. The stranded stash is only the fingerprint; the lost creature is
the bug.

**And it was invisible in the suite**, because a headless seat answers
synchronously and never re-runs the arm:
`sos::vastlands_scavenger_prepare_spell_mills_seven_and_reanimates` passes on
both sides of this fix. The eighth find's lesson again — *the suite tests the
path that does not suspend, and the training path is the one that does* — so the
regression test sets `wants_ui` and drives the resume:
`sos::bind_to_life_still_reanimates_when_the_pick_suspends`.

**The fix is the resolution's own knowledge, not the card.**
`resolve_effect_into_kind` carries a `resuming` flag and
`resolve_effect_resumed_into` is the entry a continuation uses: spells already
knew (`override_effect.is_some()` is the resume), and the trigger and ability
paths take a `resuming` argument that is `false` at their five ordinary call
sites. Only the outermost call of a continuation is flagged, so the nested
resolutions it runs still reset normally. Every scratch-reading selector and
`Value` after a suspend is fixed at once (`LastMoved`, `LastCreatedToken(s)`,
`NamedBySource`, `CardsDiscardedThisEffect`, `DamageDealtThisResolution`, the
sacrifice tallies …), not just this card's.

Verified on the cells that found it: the same recipe at `cube 890` and `all 890`
under `CRAB_ANSWER_LOG=strict` reads **2 cells / 10,000 games / 0 failures / 0 cap
/ 0 stuck** after the fix, against two rc-134 aborts before it. The suite is
19,416 / 0 / 5 with the flag exported and **golden_trace 10 / 10 unmoved** — the
fix changes behaviour only where the bug fired, and no committed trace reaches it.

⚠ **The first version of this fix was a `GameState` bool, and the size guard
caught it**: one inline byte cost the struct eight (1,608 against
`cow::tests::game_state_stays_small`'s 1,600 cap), because the state's
byte-sized region was exactly full. An argument costs nothing per probe clone —
and PERF `(-144)`'s guard is why the wrong version never shipped.

### The same census's structural finding: seven cards nest two answer-log arms — RETIRED 2026-09-11 as a hazard, not a defect, and the redesign below is NOT work

⚠ **Read this box before the rest of the section, which is preserved as the
reasoning that led here and is wrong about the severity.** The redesign it
prices — `run_effect` -> `run_effect_parked` at ~60 sites inside 38 asking arms
— **is not needed and should not be done.** The seven nestings were reasoned
about from the static walk and never taken; taken, on the suspending path, for
the three shapes that differ, all three are correct:

* `stx::part_03::conspiracy_theorist_nested_asks_pay_once_when_both_suspend`
  (a nested pair on one seat, `MayPay > MayDiscard`),
* `vis::forbidden_ritual_nested_asks_each_take_their_own_answer_when_both_suspend`
  (three levels asking two different seats, `MayRepeat > MaySacrifice >
  UnlessPlayerPays`),
* `hml::giant_albatross_charges_each_creature_once_when_the_asks_suspend`
  (the loop arm).

The reason generalizes to all five outer arms, and it is two properties, both
needed: **each clears the channel before running its body**, so the inner arm's
`cursor = 0` never sees the outer's answers; and **none has any work after the
body**, so a suspend inside the body re-entering at the INNER arm (the suspend
signal carries the inner effect) loses nothing. Neither is structural, so the
gate stays and the allowlist stays one line per card — what it is guarding
against is a new nesting whose outer arm keeps working after its body, or asks
again after it (the `MayPayRepeatedly` shape), not the nesting as such. The
gate's own doc comment now says that.

Also noted while taking those paths, open and cheap: **`Effect::MayRepeat` asks
its repeat question through the raw decider rather than `ask_seat_bool`**, so a
`wants_ui` seat is never asked and never suspends there — the AutoDecider
answers for a human. Forbidden Ritual and every other `MayRepeat` card.

---

`core_rules::structural_audit::no_shipped_card_nests_two_answer_log_arms` (new)
walks every card and flags one log-using effect inside another. The log is one
channel per resolution and `run_effect` recursion does not open a new one, so the
inner arm's `cursor = 0` replays the **outer** arm's answers — and
`ask_seat_bool` reads any kind, so it silently inherits the outer "yes" instead
of asking. Seven shipped cards do it (each safe today, per the box above):

| card | nesting | what the inner ask inherits |
| --- | --- | --- |
| Conspiracy Theorist | `MayPay > MayDiscard` | the payment's yes |
| Emberwilde Djinn | `MayPayBy > MayPayLife` | the payment's yes |
| Forbidden Ritual | `MaySacrifice > UnlessPlayerPays` | the sacrifice's yes |
| Giant Albatross | `MayPay > DestroyEachUnlessPaysLife` | the payment's yes |
| Rottenmouth Viper | `MaySacrifice > MayDiscard` | the sacrifice's yes |
| Skirk Drill Sergeant | `MayPay > RevealTopMayPutOntoBattlefield` | the payment's yes |
| Worms of the Earth | `AnyPlayerMayAccept > AnyPlayerMayAccept` | the outer vote |

**Allowlisted, not fixed, and the reason is worth reading before anyone tries.**
Clearing the log before the outer arm runs its body strands the outer arm's own
`cursor`, which keeps counting; `MayPayRepeatedly` asks again *after* its body,
so it cannot clear at all; and a per-arm base offset into the log cannot be
recovered on a re-run without answer provenance, which the channel does not
carry. The same arm also shows the *other* half of that shape:
`MayPayRepeatedly` pays mana and runs its body **between** asks, so every suspend
re-pays and re-runs them on the re-run — the "keep all side effects after the
final ask" contract is violated there today, **quadratically**: with `k` suspends
the first iteration's cost is paid `k` times. Two shipped cards reach it, both
upkeep loops — **Magnetic Mountain** (`arn/gaps.rs`) and **Dream Tides**
(`vis2.rs`), each "Pay {2} to untap a tapped creature?" — and both only for a
seat that suspends, so the suite's headless tests cannot see it. The arm cannot
be fixed by clearing (see above): it needs the asks separated from the payments,
which is the same provenance redesign. **The design that does work, sketched so the next run does not re-derive it.**
Provenance is the wrong axis — the arm's `effect` is *cloned* into the suspend
signal, so no pointer or identity survives the re-run. What does work is parking
the channel at the nesting boundary, which is where the two logs are known to be
different: a helper `with_parked_answer_log(|s| s.run_effect(body, ctx, events))`
used by the four outer arms (`MayPay`, `MayPayBy`, `MaySacrifice`,
`AnyPlayerMayAccept`) takes the log before the body and, on return, **restores it
only if the body did not suspend** — if it did, the resume re-runs the *body's*
arm (the suspend signal carries the inner arm's effect), so the parked outer
entries are dead and must be dropped rather than restored. **Park it at the arm, never inside `run_effect`** — and not only for the cost:
the dispatcher cannot tell "the arm I am about to run is the one these answers
belong to" from "a different arm". A resume re-enters as
`run_effect(Seq[the suspended arm's effect, rest])`, so a park at that outer call
would hide the arm's own replay from it and re-ask for ever — the tenth find's
stall, reintroduced. Inside an arm, after its own asks, the distinction is free:
whatever the callee logs is not this arm's.

**The population is bigger than the seven cards.** 38 asking arms run a nested
effect (~60 `run_effect` call sites after their own first ask — the census is a
`run_effect` grep inside each `let mut cursor = 0` block); the seven are just the
cards whose nested body happens to be another asking arm *today*, and the gate is
what keeps a new catalog entry from quietly joining them. So the change is a
mechanical `run_effect` -> `run_effect_parked` at those sites, and it is worth
doing in one commit with the gate's allowlist emptied in the same breath. That closes the
seven; the two `MayPayRepeatedly` cards need the *other* half (asks separated
from payments) and are not fixed by it.

### The other half, now a column: a MUTATION before the arm's first ask

The re-run repeats everything the arm did before the ask it suspended on, so any
mutation there happens again — which makes "keep all side effects after the final
ask" a checkable property, and `scripts/audit_answer_log.py` checks it now
(`PRE`, matched on the engine's own mutators). **Reading: 63 arms, 1 PRE.**

* `Effect::CoinFlipDestroyLoop` (**Crooked Scales**) — it **flips a coin** before
  the ask and pays the repeat cost between asks. A suspending seat therefore
  re-flips on every resume: the coin the player was told about is re-rolled, and
  a re-run that wins destroys the opponent's creature without ever replaying the
  answer. CR 705.1 makes a flip one event; this makes it `k + 1` of them.
* `Effect::MayPayRepeatedly` (**Magnetic Mountain**, **Dream Tides**) — the
  quadratic re-payment above. The audit's `PRE` column does not flag it because
  its payment is *after* the first ask, inside the loop; the loop is what repeats
  it.

**And that second census — "a mutation inside the loop that asks" — reads 11 of
the 63 arms**, which makes this the widest open member of the class:

| arm | what repeats per suspend | status |
| --- | --- | --- |
| `run_each_unless_pays` (Fade Away, Cut the Tethers) | the mana each earlier seat paid | FIXED (the worked example) |
| `run_destroy_each_unless_pays_life` (Giant Albatross) | the life each earlier seat paid | FIXED, two passes + `budget` |
| `Effect::RevealTopPayOrTake` (Sword-Point Diplomacy) | the life paid: three denials cost 18, not 9 | FIXED, two passes + `budget` |
| `Effect::RevealHandDiscardMatchingUnlessPayLife` (Sirocco) | the life, and the discards | FIXED, two passes + `budget` |
| `Effect::AnteTopOfLibrary` (Rebirth) | the ante, and the `then` branch it ran | FIXED, two passes |
| `Effect::MayPayRepeatedly` (Magnetic Mountain, Dream Tides) | the mana, quadratically | FIXED, `answer_already_acted_on` |
| `Effect::CoinFlipDestroyLoop` (Crooked Scales) | the coin flip AND the repeat cost | FIXED, `flip_one_coin_logged` + the same skip |
| `Effect::OtherPlayerMayPayToCounter` | — | BENIGN: a failed payment is rolled back by `try_pay_after_snapshot_mode`'s snapshot (pool + tapped flags); a successful one returns |
| `Effect::PlayersMayAccept`, `AnyPlayerMayExileFromGraveyard`, `AnyPlayerMayAccept` | — | BENIGN: each returns out of the loop on the first acceptor, so nothing it mutates precedes a later ask |

**Closed 2026-09-11 (thirteenth find)** — see that section for the two recipes
and why one does not fit both. The audit now reads 63 arms / 6 suspicious
(from 10), and every remaining MID is one of the tail calls above or a terminal
`break` the static walk cannot see. The prose below is the recipe, kept.

The fix is the same two passes every time — ask everyone first, mutate second —
and each one needs its own regression test **on the suspending path**, because the
headless path is correct in all of them and the suite only tests that. The
`run_each_unless_pays` commit is the worked example, and the shape is mechanical:

```text
  let mut answers = Vec::with_capacity(targets.len());   // pass 1: ASK ONLY
  for (id, seat) in targets {
      let paid = gate && match self.ask_seat_bool(..) { Some(y) => y, None => return Ok(()) };
      answers.push((id, seat, paid));                    // no mutation in here
  }
  self.clear_answer_log();
  for (id, seat, paid) in answers {                      // pass 2: MUTATE
      if !(paid && self.pay(..)) { doomed.push(id); }
  }
```

The one thing to check per arm is the *gate* in pass 1 (`could_pay_cost`, a life
total, an emptiness test): it is now evaluated before any payment, so a seat with
two affected permanents is asked about both before either is paid for. Where the
gate is per-seat and the payments are per-permanent that changes which prompt is
posed but not which permanent leaves — say so in the commit, as the worked example
does.

Both are the same shape as the seven: correct for a headless seat, wrong for
every seat that suspends. Neither is fixed here — the flip wants its outcome
carried across the suspend (the answer log can hold it: the re-run must replay
the flip, not re-roll it), and the payment wants the asks separated from the
effects.

## FIXED 2026-09-11 (the tenth find's production half) — both resume channels leaked: two arms never cleared, no arm clears on an error unwind, and the stash had no guard at all; the RESOLUTION owns them now

The tenth find closed the *consumption* half of this class (a kind mismatch on
slot 0 is dropped, so a leftover cannot strand the next ask for ever). It left
the production half open and said so: *"the leak is upstream and in some other
arm"*. This names the arms and removes the leak.

**The instrument first, because the arms were invisible without it.**
`CRAB_ANSWER_LOG=warn` (or `=strict`, which panics) prints the card and the
top-level effect variant of any resolution that ends without suspending and
leaves `scratch.resolution_answer_log` non-empty — the point where the channel
is provably garbage. Debug builds only, one `OnceLock` read, and the whole check
is three reads on the cold side of `!log.is_empty()`, so it costs an untraced
build nothing. One `CRAB_ANSWER_LOG=strict` suite run (19,409 tests as the suite
stood then, 128 s) named both leakers, and a static census over the 63
`let mut cursor = 0` arms
(`scripts/audit_answer_log.py`) agreed and added the two shapes a test run
cannot reach:

* `Effect::EachPlayerChoosesNumberHighestLoses` (Menacing Ogre) — **no
  `clear_answer_log()` anywhere in the arm.** Every seat's sealed bid stayed in
  the channel, and the arm then runs `on_you_win` as a nested resolution.
* `Effect::MoveChosenKeyword` (Phyrexian Splicer) — same, one ask, then two
  nested resolutions (`LoseKeyword` / `GrantKeyword`) with its answer still in.
* `Effect::AnteTopOfLibrary` and `Effect::MayPayRepeatedly` — both clear on the
  normal path and both propagate a `GameError` past the ask with `?`. **No arm
  clears on an error unwind**, so every asking arm leaks on that path.

**The fix is the lifetime, not the four arms.** The channel belongs to the
resolution, so `resolve_effect_into` drops it at the outermost exit when nothing
suspended — the `Err` path included, which is what makes the unwind leak
unreachable rather than patched four times. `drop_stale_answer_log` stays as the
*within*-resolution guard; the two arms' missing clears are fixed too, because
inside one resolution the net cannot help (below).

**And the same net over the other channel, which had no guard at all.**
`scratch.stashed_resolution_answer` is the single-slot sibling (`MayDo`,
`ChooseMode`, `Amount`, `Cards`, `CreatureType`, the damage division — every
`take_opt_scratch!` site): written by `apply_pending_effect_answer`, taken by the
re-running arm, and **cleared by nothing**. An arm that suspends and then never
reaches its take — the game ends, an `?` unwinds, the re-run takes another
branch — leaves it for the next arm of the *same answer kind* to consume as its
own, and there is no `drop_stale_answer_log` equivalent to catch the mismatch:
the take removes it, so a wrong-kind leftover self-heals in one round trip and a
right-kind one is silently the wrong answer. It is dropped at the same exit. No
park is needed for it: the engine surfaces one decision at a time, so a value
sitting there while a *new* answer is applied can only be a leftover.

**The one hazard, and why the park exists.** `apply_pending_effect_answer` runs
*outside* any resolution on the resume path (`submit_decision`), and two of its
arms resolve effects of their own — `ImpulsePending` and `MayCastExiledPending`,
i.e. **outermost** resolutions whose exit would drop the suspended arm's live
replay. The log is parked across that call and the arm's own pushes are appended
behind it, so ordering is unchanged. In-resolution callers (28 sites in
`effects/mod.rs`, the synchronous non-`wants_ui` path) are at depth > 0 and the
net never fires for them.

**What the net does NOT cover — the per-arm clears stay load-bearing.** Inside
one resolution the channel is shared: a `Seq` of two asking arms, and an arm and
the nested resolution it runs. A nested arm's `cursor` starts at 0 and replays
whatever the outer arm logged. `Effect::MayPayRepeatedly` is the live shape — it
asks, pays, runs `body` as a nested resolution, and asks again — so the answers
must survive the nested resolution and parking them per resolution is *not*
obviously safe: a suspend inside the nested resolution re-runs from the nested
arm's own effect (`ask_seat_bool` stores the asking arm's `effect`), so a parked
outer log would have to be either discarded or restored and the two choices
differ. Left open deliberately, priced here so nobody parks it blind.

**The family is bigger than these two channels, so the instrument covers it
too — CENSUS, not a net.** Ten one-shot channels have the identical shape: set
by a `submit_decision` resume just before it replays a cast or an activation,
`take()`n by that replay, cleared by nothing — `stashed_resolution_answer`,
`pending_cast_sacrifices`, `pending_cast_discards`, `pending_spree_modes`,
`pending_ability_exile_other`, `pending_ability_sac_any`,
`pending_cast_spend_float`, `pending_landcycle_pick`,
`pending_ability_sac_other`, `pending_ability_tap_other`. A replay that returns
early (an unaffordable cost, an illegal target, an error) never reaches its take,
and the pick then waits for the next cast or activation *of the same shape* to
consume as its own — a player's sacrifice pick paying for someone else's spell.
`submit_decision` now names any that survive a round trip which left nothing
pending (`CRAB_ANSWER_LOG`, debug builds, behind a "something is Some" test).
**First reading: 0 stale channels across the 19,414-test suite under
`CRAB_ANSWER_LOG=strict`, and 0 across the 100,400 fresh-seed self-play games the sweeps completed.** So
the shape is there and nothing exercises it yet — which is the same sentence the
`OptionalKind` find ended on about its eight unsampled kinds, and the reason to
leave the instrument in rather than net a path no evidence names. **Nothing is
netted for these on purpose**: the channels legitimately span several
`submit_decision` calls while a decision stays pending (pick the sacrifices, then
tap for mana manually), so a drop has to be keyed on "nothing pending", and the
census is what would justify keying it.

Tests: `game::answer_log_tests::a_finished_resolution_drops_whatever_was_left_in_the_answer_log`,
`::a_finished_resolution_drops_a_stash_no_arm_came_back_for`,
`::applying_an_answer_appends_it_behind_the_parked_replay`.

## FIXED 2026-09-11 (tenth find) — one resolution's leftover answer stranded the next arm's ask for ever: the seed-835 Karn stall

Found by a fresh-seed sweep, not by a filed lead. `--decks cube --seed 835`
capped **12 of 1,600 games** (and `all` 14 of 3,400), and the boards were
nothing like the recorded Beacon of Immortality fingerprint: turn 13-25, an
empty stack, ordinary life totals, ~460 actions a turn. `CRAB_DUMP_TRACES`
named the loop in one line — `p0 SubmitDecision(Cards([CardId(106)]))` repeated
5,600 times with the board byte-identical between them.

**The mechanism.** `scratch.resolution_answer_log` is the replay channel for an
arm that asks more than one question: the arm suspends, the seat's answer is
appended, the effect re-runs from the top and each ask replays its slot by
`cursor`. It is a *per-resolution* channel and every arm that uses it is
supposed to `clear_answer_log()` on each completing path — one that returns
without doing so leaves its answers in it. The next arm's first ask then finds
the wrong **kind** of answer in slot 0, and the kind-matched replays
(`ask_seat_amount`, `ask_seat_option`, `ask_seat_cards_logged`) MISS on it:
the ask re-suspends, the answer is appended *behind* the stale one, slot 0 never
changes, and the arm re-asks until the action cap. A probe printed the log as
`[Bool(true), Cards([97]), Cards([97]), Cards([97]), …]` — a leftover `Bool`
from an earlier `ask_seat_bool`, then one `Cards` per re-ask. `ask_seat_bool`
reads any kind, so the same poison made it answer "no" to a "may" instead of
looping — quieter and just as wrong.

**The fix is the class, not the leaker.** `drop_stale_answer_log` runs before
all four replays: a kind mismatch on the arm's FIRST slot (`cursor == 0`) is
always a previous resolution's leftover, never this arm's own answer, so it
drops the log and lets the ask suspend normally. One extra round trip where it
fires, instead of a capped game — and it holds for every arm that leaks, found
or not. Slots past 0 are left alone: a mismatch there would be this arm's own
sequence and clearing it could strand the arm a different way.

Karn, Scion of Urza's +1 (`RevealTopOpponentChoosesToHand`, the "an opponent
chooses one of them" ask) is the card that exposed it; the leak is upstream and
in some other arm, so the card is not the bug. **Not caused by this run** —
`--decks cube --seed 835` reproduces the same 12 caps on `38b05af8`, the branch
tip this run started from.
After: cube 835 and all 835 both **0 undecided**; the caps left on `all` 804 /
812 / 819 are the documented Beacon board (turn 210, `i32::MAX` life).
Tests: `game::answer_log_tests::a_stale_answer_log_entry_does_not_strand_the_next_ask`.

⚠ **The transferable half is the instrument, and it is nearly free on this
box.** A 3,200-game `cube` sweep cell is 7 s on the release-fast binary; this
stall sat in a pool the branch has swept for months because nobody had taken
seeds past 761. Sweep wider.

## FIXED 2026-09-11 (ninth find) — the same prose sniff one family over: Devour sacrificed the bot's whole board

The eighth find's bot half took the prose read out of `decide_choose_cards`;
this is the family the residue named as next, and it had a live defect the size
of a board wipe. `decide_pending_policy`'s `ChooseAmount` arm recovered what the
number was *for* from the prompt — `contains("destroy all creatures with
power")`, `to_lowercase().contains("life")`, `starts_with("Pay {X}")` — and
everything those three missed answered **`max`**.

`Effect::SacrificeAnyNumber` poses `"Sacrifice how many?"` with
`max = every permanent the filter matches that you control`, sorted weakest
first, and runs `per_each` once per sacrifice. It matches none of the three
patterns. **Every bot seat sacrificed its entire board**: the whole Devour
family (22 catalog users — Devouring Hellion, `devour`/`devour_filter`, Famished
Worldsire's "devour land") and God-Eternal Bontu, whose ETB then drew that many.
A linear payoff for the board is a bad trade at any size, and at `max` it is the
game.

`Decision::ChooseAmount` carries `kind: AmountKind` now — `Upside` (the
historical `max`, and still right for "how many extra counters?"), `Cost`
(the chooser's own permanents; the bot declines), `Life`, `Mana`,
`DestroyPowerCutoff`. Eighteen asks and the `ask_seat_amount` helper; the field
has no default in a struct literal, so a new ask cannot skip it. Every other
site keeps exactly the answer the prose match gave it, so the only behaviour
that moves is the Devour one.

**Two more prompts were falling through to `max` and are worth knowing about
even though this run left their answers alone:** `"Pay how much {E}?"` (spend
all energy) and `"Read ahead — choose a starting chapter"` (start at the LAST
chapter, i.e. skip the saga to its final ability — the audit lists this one as
`ack`, so it is deliberate). `"Pay how much?"`'s `max` is already capped to the
floating pool, so `Mana` is what it was doing anyway.

Residue, **the `Cost` half fixed the same day**: "give up the tokens and nothing
else" is the right answer and only the effect knows the order it sacrifices in,
so the ask carries the count — `AmountKind::Cost { free }`, where
`SacrificeAnyNumber` counts the leading run of tokens in its own
tokens-first / cheapest-first ordering and the bot answers exactly that. A
Devour creature now enters with counters for the chooser's spare tokens and the
board survives; before this run it ate the board, and between the two it did
nothing. **And one family is still prose-keyed**: `OptionalTrigger`'s
`description` (57 sites), which the bot branches on in three places
(`starts_with("Pay ") && contains(" life to deny ")`, `starts_with("Reveal the
top card (")`, and the generic upside screen). It is the last thing standing
between `(-288)`'s flag and "no UI text built in a simulator at all".
Tests: `server::bot::tests::bot_choose_amount_reads_the_kind_not_the_prompt`.

## FIXED 2026-09-11 (eighth find) — the bare resolution-time asks: six "up to N" picks resolved as a no-op and every "choose a color" named White

Same shape as the seventh find, one layer up: the *decision*, not the payment.
`AutoDecider`'s blanket answers are `ChooseCards` -> the first `min` (nothing at
`min: 0`), `ChooseAmount` -> 0, `ChooseColor` -> the first legal colour. Bot
seats set `wants_ui`, so a **bare** `decider.decide` never reaches
`decide_pending_policy`: every such site is answered by the blanket default for
every seat in the training path.

`scripts/audit_decision_plumbing.py` now sorts its bare sites by what the
default does — *DEAD* (in front of a whole effect body), *repeat* (a loop gate;
the printed minimum still happens, which is what the 2026-07 audit miscounted)
and *ack* (a comment beside the ask names the headless answer). It read
**195 sites / 99 plumbed / 96 bare — 11 DEAD, 7 repeat, 15 ack** before this run
and **178 / 104 / 74 — 0 DEAD, 7 repeat, 17 ack** after; the site count falls
because fourteen hand-rolled colour asks became one helper call. **The DEAD
column is a gate now that it is zero**; the other two are a triage population.

The DEAD half, fixed: `ExileUpToNFromGraveyards` (18 cards, and Soul-Shackled
Zombie's "if a creature card was exiled this way" rider with them) and
`PutAuraFromHandAttachedTo` are plumbed through `choose_up_to_cards`, so a
`wants_ui` seat gets `decide_choose_cards`' real policy; `MillThenToHandN`
(9 cards — Gather the Pack milled five and took nothing),
`ReturnSelfDeployBlocker` (Aetherplasm bounced itself and deployed nobody) and
`ShuffleAnyNumberFromHandThenDraw` stay synchronous **because the arm has
already mutated the board when it asks** — a suspend re-runs the arm from the
top, so it would mill or bounce twice — and take a computed default instead.
That is the general rule for this family: *plumb when nothing has moved yet,
give the headless seat a real default when something has.*

The colour half: `Effect::ChooseColorForSelf` is one choice shared by 42 cards
whose answer is read off **either** side of the table — Ward Sliver's protection
and Iona's lock off theirs, Heraldic Banner's anthem and Caged Sun's mana off
ours — and the split is about 21/21, so no single blanket answer was better than
a coin flip. It keys on the card's own consumer now
(`CardDefinition::chosen_color_aimed_at_opponents`, a bit in the `debug_flags`
word: the hostile markers are `GrantProtectionFromChosenColor`,
`OpponentsCantCastChosenColor`, `RedirectChosenColorSpellDamageToController`,
`PermanentsOfChosenColorOpponentsControl`, `HasChosenColorOfSource` and
`PreventAllDamageFromChosenColor`). The two censuses behind it are
`densest_color_of_among` / `densest_color_among_opponents_of`, both over
`color_weights` — **three hand-written copies of the opposing census are gone**,
one of them (`GrantProtectionFromChosenColor`, 41 cards) battlefield-only, so
that grant now counts hands too. The "choose a basic land type" asks
(Realmwright, Terraformer, Vision Charm, Land Tax's basic) rode the same
decision and fell through to Forest; they take the hand's mana need, and
`basic_land_type_for` replaces four hand-written colour→type tables.

### FIXED 2026-09-11 — the bot side: `PickValue` on the ask, and the prose sniff gone

`decide_choose_cards` decides cost-vs-upside by **sniffing the prompt prose**:
`detrimental = prompt.contains("sacrifice") || prompt.contains("discard")`. The
engine writes those strings and the bot pattern-matches them, which is the
walker-drift shape one level up — so the fix looks like "put the flag on
`Decision::ChooseCards` instead of inferring it", ~54 construction sites.

**Censused first, and it is much smaller than it looks.** 95 distinct
ChooseCards-family prompts reach the engine; 8 carry a cost word and 87 do not.
But 87 is not the bug surface, because two of the three source branches already
land on the right answer for an own-side pick:

* **battlefield-source** ranks *enemy* creatures only, so an own-board cost
  finds no candidate and, at `min: 0`, answers empty — a decline, which is what
  a cost wants (Cloudstone Curio, "Return any number of permanents you control",
  the convoke-shaped "Tap any number of untapped creatures you control").
* **own-graveyard** is gated on `!prompt.contains("exile")`, and every
  cost-shaped graveyard prompt says "exile" — so those decline too.
* **hand-source** is the live one: not-detrimental means "take the biggest
  card(s) up to `max`", i.e. hand over the best cards. That is Scroll Rack's
  "Exile any number of cards from your hand", "Exile a card from your hand",
  and "Choose a card to put on the bottom" — **three or four sites, not 87.**

**Built at the size the census priced, not the 54-site one.**
`Decision::ChooseCards` carries a `value: PickValue` (`Cost` | `Gain`, `Cost` the
serde default) and `decide_choose_cards` reads it instead of
`prompt.contains("sacrifice") || prompt.contains("discard")`. A struct literal
has no default for it and the three shared asks (`ask_seat_cards`,
`ask_seat_cards_logged`, `choose_up_to_cards`) take it positionally, so a new ask
cannot skip the question — that is what makes it a class kill rather than eight
patches. **104 asks; 39 say `Gain` outright and three more decide it from the
board**: `MoveChosen`'s two read the destination zone (a card landing in the
chooser's hand or on their battlefield is `Gain`, one leaving for a graveyard,
exile or library is `Cost` — Divergent Equation against Emeritus of Ideation),
and the "exile a card from the revealed hand" ask flips on `victims_turn`,
because the victim exiling one is keeping it.

What actually moved is the hand and own-board half the census named:

* The cost-shaped prompts that say **"exile"** rather than "sacrifice" or
  "discard" — "Exile a card from your hand", "Exile a card from your hand or a
  permanent you control", Scroll Rack's "Exile any number of cards from your
  hand", "Choose a creature you control to exile" — plus the two library asks
  ("Put which card from your hand on top of your library?", "…on the bottom…").
  All read as upside and handed over the biggest card in hand; they shed the
  least useful now, and an optional one (`min: 0`) declines outright.
* **"Choose N permanents to keep" kept the worst.** The battlefield branch ranks
  *enemy* creatures, found none on an own board, and the min-fill then gave up
  the least valuable — i.e. kept it. `Gain` over the battlefield ranks ours
  first, best first; the same arm fixes "Untap which permanents?" (which had
  been untapping the opponent's biggest, the candidate list being every tapped
  permanent) and "Choose a permanent to attach this to".
* Retraced Image's reveal puts the card onto the battlefield, so it is `Gain`,
  as are the graveyard-recursion asks ("Put up to N cards on top of your
  library", the sideboard wish).

Tests: `server::bot::tests::bot_choose_cards_cost_over_hand_sheds_the_worst`,
`…_gain_keeps_own_best_permanent`, `…_gain_prefers_our_own_permanents`.

**The shape a two-valued flag does not reach, found by classifying all 104:**
an "any number" pick where picking *is* the mechanism and declining wastes the
whole action. Scroll Rack is one (exile any number, draw that many, the exiled
cards go back on top in the order named) and reads `Gain` — a wider pick digs
deeper and the order is the library order. The ones that need a *computed* set
and so are still dead for a `wants_ui` seat, filed not fixed:

* ~~**Collect evidence declines for every bot seat.**~~ **FIXED the same day.**
  `CollectEvidence` / `CollectEvidenceX` branch on `players[p].wants_ui`, and a
  bot seat is `wants_ui`, so the ask went to `decide_choose_cards`: own
  graveyard, `Cost`, `min: 0` — answer empty, `picked < need`, decline, on all
  17 catalog users. The threshold is not something a generic policy can size,
  so the *ask* carries it now: `CollectEvidence`'s `min` is the size of the
  cheapest qualifying set (the same set the `else` branch auto-picks, and that
  branch already collects whenever it can, so the two sides agree rather than a
  "may" becoming mandatory), and `CollectEvidenceX` is a `Gain` because X *is*
  the total exiled — a wider pick is strictly better, which is why the Auto
  branch exiles the whole graveyard. Beside it, `decide_choose_cards`' forced
  own-graveyard fill now gives up the **cheapest** cards under `Cost` and keeps
  the biggest under `Gain` (it took the biggest either way), so the bot's
  answer to a collect-evidence floor is exactly the engine's auto set.
  Tests: `server::bot::tests::bot_choose_cards_cost_over_own_graveyard_gives_up_the_cheapest`.
* **Fateseal** ("put which cards on the bottom?") and **Stronghold Gambit**
  ("choose a card in your hand", where the cheapest revealed creature enters)
  are the same shape one step further: the right answer is a computed set
  (their engine-side `auto` defaults have it), not "the best" or "the least".

Residue, unchanged: `BecomeChosenColor` picks per *source* rather than per
*target*, so it cannot yet dodge a specific hoser on the board; Cloudstone
Curio's decline is deliberate (every candidate is ours, and the bot's
battlefield branch declines an all-own board at `min: 0` too); Credit Voucher
still bypasses `choose_up_to_cards` for its computed "the part of the hand we
cannot cast", for the same reason Scroll Rack could not. **The same prose
sniff is still live one family over** — `Decision::ChooseAmount` branches on
`prompt.contains("destroy all creatures with power")` / `"life"` /
`starts_with("Pay {X}")` and `OptionalTrigger` on `description`, both of which
`(-288)` deliberately kept readable for exactly that reason. That is the next
member of this class, and it wants the same treatment: put the question on the
ask.
Earlier tests for the engine half: `core_rules::cr_recent101` (8),
`core_rules::cr_recent102` (6).

## FIXED 2026-09-10 (seventh find) — every "you may pay" / "pay or else" effect paid from the floating pool only, so a bot seat never paid: 149 `MayPay` sites dead in self-play

`Effect::MayPay`, `MayPayBy`, `MayPayRepeatedly`, `MayPayX`, `PayManaOrElse`,
`run_each_unless_pays` (the mass "pay to keep this permanent" walk) and
Transmute Artifact's surcharge all ran `mana_pool.pay(cost)`
— the pool and nothing else, with a comment saying mana abilities are not
activatable mid-resolve. Echo, cumulative upkeep and every "unless [player]
pays" already went through `try_pay_with_auto_tap` (CR 605.3a: a mana
ability may be activated while paying a cost during resolution). A bot
seat's pool is empty at trigger resolution (pools empty between steps and
the bot taps as it casts), so Punishing Fire never came back, Horizon
Spellbomb never drew, a Rhystic-style "pay {1} to deny" never denied, a
"pay {2}: draw" never drew — the yes/no policy answered yes and the payment
failed silently. Now every site pays through `pay_mana_cost_with_picks(
payer, cost, None, events)` (pool first, then untapped sources, the
payer holding priority for the taps) and `MayPayX`'s X bound is the pool
plus one per untapped source. The bot screens a `MayPay` prompt it cannot
fund (`may_pay_prompt_affordable`: `find_maypay_cost` by description,
`can_afford_from` over `available_mana`) so the outcome second opinion
never prices a body that cannot happen. Tests: `core_rules::cr_recent53::
cr_605_3a_*` (two), `server::bot::tests::bot_declines_a_maypay_it_cannot_fund`.
Golden traces unmoved (no traced game reaches a MayPay); the `--bench`
counters are the gate for the `fixed` pool. **Still pool-only, by design
for now:** the hand-initiated actions (`cycle_card`, `landcycle_card`,
`reinforce_card`, `activate_discard_ability`, `offer_madness_cast`,
`reconfigure`, `ninjutsu`, `player_casts_cheap_creature_free`) — the bot
takes none of them (`rg 'GameAction::(Cycle|Ninjutsu|Reconfigure|Madness)'
crabomination/src/server/bot.rs` is empty) and a client seat taps first
under CR 601.2g proper tapping; give them `try_pay_with_auto_tap` when a
bot learns one.

## FIXED 2026-09-10 (sixth find) — the graveyard walk had none of the battlefield walk's rules: no fan-out, no once-per-turn, no intervening-if gate; one step walk, one scope; Attuned Hunter dead on the battlefield

`dispatch_triggers_for_events`' graveyard walk (`FromYourGraveyard` and the
graveyard `SelfSource` kinds) pushed one candidate per (card, trigger) per
batch and `break`ed — no CR 603.6 fan-out (Punishing Fire fired once for
two life gains in a batch; Furious Forebear once for a board wipe), no CR
603.3d `once_per_turn` (no shipped card combined the two, so latent), no
CR 603.4 intervening-if before the slot, no `died_card_snapshots` /
Hushbringer death checks, and `actor: None` where the battlefield walk binds
the event's player. The fan-out kind list was an inline `matches!` in the
battlefield walk that nothing else could read. Now: `events::
event_kind_fans_out` is the one list both walks read; the graveyard walk
reads the filter before spending a once-per-turn slot, fans out by kind,
and skips a replaced or suppressed death. `EventScope::from_graveyard()`
replaces the seven hand-written `matches!(scope, FromYourGraveyard)` sites
(the battlefield skip, the lane predicate, the step / cast / combat-damage
/ dispatcher walks). Beside it:

- **`EventSpec::once_per_batch`** — "whenever one or more …" (CR 603.2c):
  one fire a batch, no turn cap, honoured by the dispatcher's two walks,
  the `declare_attackers` Attacks walk (a local set) and the combat-damage
  hook (the per-sub-step set, now `combat_trigger_fired_this_step`, keyed
  `(source, index)`). The catalog's "one or more" census (`rg -B14
  'once_per_turn\(\)' | rg -i 'one or more'`): four cards spelled the batch
  cap with `.once_per_turn()` and under-fired across batches — Mu Yanling,
  Wind Rider; Vengeful Townsfolk; Frostcliff Siege; Invasion Tactics —
  migrated; the other six print "This ability triggers only once each
  turn" and keep the turn cap.
- **`EventScope::FromYourGraveyardAnyPlayer`** — the graveyard `AnyPlayer`:
  `fire_step_triggers` walked only the active player's graveyard, so no
  graveyard card could say "at the beginning of *each* end step" (Kami of
  Transience). The walk now visits every graveyard behind the lane and
  admits `FromYourGraveyard` on the owner's step, the new scope on any.
- **`YouAttack` from the graveyard** — `declare_attackers`' "whenever you
  attack" walk read the battlefield only; Persistent Marshstalker's
  threshold return could never fire. The attacking player's graveyard joins
  it behind the lane.
- **Attuned Hunter** was a *battlefield* trigger about your graveyard
  written with the graveyard scope — dead on the battlefield since it
  shipped (the walk there skips the scope), and firing only while the
  Hunter was itself in the graveyard, where its counter is a no-op. Now
  `YourControl` + `once_per_batch`.
- Kami of Transience's end-step return, Sneaky Snacker's printed "third
  card" return (its synthesised `{2}{B}` activation replaced) and
  Persistent Marshstalker's threshold return shipped on the primitives
  above — every one was already spellable except the walks
  (`AttackedWithCreatureMatching`, `PlayerDrewAtLeastThisTurn`,
  `PutIntoGraveyardFromBattlefieldThisTurn`, `JoinCombatAttacking` all
  existed; the "Triggers that live in the graveyard" entry below misread
  the gates). Tests: `core_rules::cr_recent53::cr_603_*` (four),
  `recent_a::recent_091_100::kami_of_transience_returns_at_each_end_step_after_an_enchantment_died`,
  `recent_a::recent_001_010::persistent_marshstalker_returns_attacking_on_a_rat_attack_at_threshold`,
  `stx::part_03::sneaky_snacker_returns_tapped_on_the_third_draw_of_a_turn`.

## FIXED 2026-09-10 (fifth find) — a cast or activation resumed from a cost-choice prompt returned its events to nobody

`submit_decision`'s `CastAdditionalCost` and `ActivateAbilityChoice` arms
replayed the suspended action by calling `cast_spell` / `activate_ability`
directly and returning the result, where every other resume arm goes
through `perform_action`. The events came back to `perform_action_inner`'s
early `SubmitDecision` return, which dispatches nothing: a `manual_mana`
seat's Foundry Helix fired no cast trigger and its sacrifice no death
trigger (Blood Artist silent), and no SBA sweep followed the payment. Bot
seats never pose these prompts (both are gated on `manual_mana`, the
1-in-96 livelock fix), so the client seat was the only victim. Both arms
now replay through `perform_action` (`replay_activation`), and the cost
picks stashed for the replay are cleared after it — a failed replay
restores the checkpoint, stash included. Tests:
`mh::mh2f::foundry_helix_resumed_after_the_sacrifice_choice_fires_the_deaths_triggers`,
`core_rules::game::a_sacrifice_cost_picked_through_the_prompt_still_fires_death_triggers`.

## FIXED 2026-09-10 (fourth find) — a cost-paid permanent's own dies trigger stacked below the spell or ability it paid for; Mine Collapse's alternative cost

- CR 603.3: the death funnel (`remove_to_graveyard_with_triggers`) pushes
  the leaving permanent's own dies / leaves triggers the moment it leaves,
  and every sacrifice COST called it before the spell or ability was on the
  stack — so Horizon Spellbomb's "pay {G}: draw" resolved after its search,
  a casualty / emerge / offering / Fireblast sacrifice's own trigger after
  the spell. `remove_to_graveyard_as_cost` (the ten cost sites: `sac_cost`,
  `sac_other`, casualty, bargain, forage, the additional-cost and
  alternative-cost sacrifices) parks what the funnel pushed on
  `scratch.pending_cost_triggers`; the dispatcher's top drains it onto the
  stack, above the spell and below the batch's own triggers. Resolution
  deaths (Destroy, the SBA sweeps) are untouched. Test:
  `modern::devotion_theros::a_sacrifice_costs_own_dies_trigger_stacks_above_the_ability`.
  Residue: a sacrifice cost picked through a `pending_decision` resume goes
  through the same funnel and the same drain. The order moved two suite
  tests: Digsite Conservator's discover trigger now asks first, and
  Knowledge Vault's "{0}: Sacrifice this artifact. If you do, .." was a
  `sac_cost` that only worked under the wrong order — re-shaped as the
  effect's first step (`SacrificeSource`), the printed shape.
- `AlternativeCost` had a sacrifice half (`sacrifice_permanents`, Fireblast)
  and a `not_your_turn_only` gate but no your-turn gate; `your_turn_only`
  mirrors it at the cast and in the client view, and Mine Collapse (training
  pool) casts for a Mountain on its controller's turn
  (`modern::supplement_batches::mine_collapse_sacrifices_a_mountain_on_your_turn_only`).

## FIXED 2026-09-10 (third find) — helper-built abilities were unread by every catalog column: 13 cards, a dead Steam Vines half, a dispatcher arm

`audit_catalog_stats.py`'s nine ability columns each answered `None` for a
card whose ability vec held a helper call, and ~2,000 abilities are built
that way. Inlining the helper (INCOMPLETE_CARDS "Helper-built abilities")
read them for the first time and found thirteen shipped cards at the wrong
cost / tap / event / scope / amount — three of them training-pool cards
(Stonework Packbeast's `{T}` for a printed `{2}`, Engineered Explosives'
tap, the end-step Elementals on `YourControl`). Two engine halves under
them:

- `EventScope::EnchantedBySource` served a death, an exile, a damage, an
  attack, a block, a tap, a face-up, a draw and a targeting of the host, but
  not a non-death leave: a `PermanentLeavesBattlefield` trigger under it
  (Traveling Plague, re-shaped from `CreatureDied`) fired on a death and
  never on a bounce. The arm now matches `PermanentLeftBattlefield` — the
  Aura is still attached at dispatch (the orphaning SBA runs after), the
  `auras_at_death` snapshot is the fallback (`traveling_plague_returns_when_
  its_host_is_bounced`).
- `Effect::ReturnSelfAttachedToChoiceOf` was Necrotic Plague's shape only
  (a graveyard source, a creature the chooser doesn't control), so Steam
  Vines' "attaches to a land of their choice" — an on-battlefield source, a
  land pool — was a no-op every time it ran. It takes a `filter` read from
  the chooser's seat and re-attaches an on-battlefield source in place
  (`steam_vines_destroys_the_land_it_taps` pins both halves).

The finder generalises: every column now reads a helper-built ability, and
the number readers a file's `fn -> Effect / StaticAbility / Predicate /
Value` helpers. What still hides a number is a helper in *another* file
(`shortcut::animate_land`'s +1/+1 — Rootwise Survivor's row).

## FIXED 2026-09-10 (second find) — an Aura's own `effect:` is its attach and the cast path runs nothing after it; six shipped Auras had a dead entry half

Found while re-shaping Venarian Gold (below): its "tap enchanted creature and
put X sleep counters on it" was spelled as `effect: Seq([Attach, Tap,
AddCounter])`, and a cast Venarian Gold attached and did nothing else — the
resolving Aura permanent enters attached to its cast target (stack.rs, CR
303.4f) and its `effect:` is never run. A census of the shape (an Aura whose
`effect` is a `Seq` whose first step is `Attach { what: This }`) found five
more: Hardlight Containment (the exile), Meltstrider's Resolve (the fight),
Pain for All (the ping), Mark of the Oni (the control change), Shielding Plax
(the draw). Every one had a test — resolving the `effect` by hand through
`resolve_effect`, which is why the suite was green.

Fix (`70f037b1`..): the entry half is an ETB trigger (`etb(..)`, the
Persuasion / Confiscate / Sleeping Potion shape), the six cards re-shaped,
their tests moved to the trigger (and cast-based ones added for Shielding
Plax, Mark of the Oni, Venarian Gold), and a suite gate
(`core_rules::structural_audit::no_aura_spells_an_entry_effect_after_its_attach`)
refuses the shape. The lesson: **a test that resolves a definition's effect
by hand tests the effect, not the card** — the cast path is the card.

## FIXED 2026-09-10 — step triggers and targeting triggers under `EventScope::EnchantedBySource` never fired: nine shipped Auras

`fire_step_triggers` treats every event-based scope as "never" (they are
matched per event), and the `EnchantedBySource` matcher served deaths,
exiles, damage, attacks, blocks, taps, face-ups and draws of the host — not
a step, and not `BecameTarget`. So "at the beginning of the upkeep of
enchanted creature's controller" spelled with that scope was dead: Paroxysm
(also missing its "otherwise +3/+3" — `RevealTopThenIf` had no `else_`),
Numbing Dose (whose `PlayerRef::EnchantedPlayer` is the enchanted *player*,
`None` on a creature host), Takklemaggot, Venarian Gold; "when enchanted
creature becomes the target" was dead too: Sleeping Potion, Spinal Graft,
Fractured Loyalty (which also named the Aura's controller, not the
targeting spell's — `TriggerEventPlayer` is the stamped actor); and Viridian
Harvest listened on `PutIntoGraveyard`, a kind the host scope never carried
(`PermanentDied` is the host leaving to the graveyard).

Fix: the step triggers take the documented Wanderlust shape (`AnyPlayer` +
`ActivePlayerControls(host)`); the matcher gains the `BecameTarget` arm and
the implicit "source is the target" rule exempts the host scope; `RevealTopThenIf`
gains `else_`; `SacrificeSourceUnlessSacrifice` gains `count` (Cosmic Larva's
and Mold Demon's "two lands / two Swamps" sacrificed one). One test per card,
and a suite gate (`no_trigger_sits_under_a_scope_the_dispatcher_never_matches`)
that reads every catalog trigger against the served kinds — it found the
last three (Takklemaggot, Venarian Gold, Viridian Harvest) the moment it
existed. The lesson is the audit-column one again: **a scope the dispatcher
special-cases is a column nobody had read**; the gate reads it every run.

## FIXED 2026-09-09 (fourth find) — the CR 732.3 activation guard was reset by the mana ability that paid for the loop, so Basalt Monolith's tap-and-untap ran an `abilarms` game to the action cap

Found by the searching-pilot mirrors at fresh seeds (`abilarms` vs `gang`,
`--decks all --seed 773`, one cell of 272 games): `cap: 6000 actions, turn
21, stack 1 / ability Basalt Monolith x1`, and the trace is one
`ActivateAbility { CardId(25), ability_index: 1 }` every three actions from
turn 21 on — `{3}: untap this artifact`, paid by tapping the Monolith
itself for `{C}{C}{C}`. Mana-neutral, so the board is identical at every
announcement, which is exactly the shape `check_free_activation_loop`
refuses after fifty repeats. It never fired because the auto-tapper's
mana activation went through the same entry and **reset the watch**: the
key stream read (untap, tap, untap, tap, …), never two of a kind in a row.
The plain-land-tap fast path reset it too, on the argument that a reset
"is what the initial state is" — true of the watch's state, false of the
loop it hides (three lands tapped to pay for untapping three lands is the
same shape).

Fix (one commit): the guard is asked after the activation resolves its
ability, only when that ability is not a mana ability (CR 602.2b — a mana
ability activated to pay a cost is part of the announcement it pays for,
not a game choice of its own; and its own key can never repeat on an
unmoved state, since it moves a pool), and the land fast path leaves the
watch alone. Cheaper on the bot path, not dearer: the ~24 k mana
activations a six-game cube run no longer touch it, and `ability_is_mana`
was already computed for the thirteen gates below. Test:
`core_rules::cr_recent76::cr_732_3_a_paid_loop_through_the_sources_own_
mana_ability_is_rejected` (Basalt Monolith, tap / untap / drain, refused
at ~50). The `dflt` pilot never walks this loop (257,600 games at fresh
seeds this run, 0 caps of the kind), so no trace, bench counter or pool
game moves; the guard's other two tests stand.

The lesson: **an instrument that a sub-step of the watched action can
reach is not measuring the action.** The watch keyed announcements, and a
cost payment announces too; the fix is to name what is *not* an
announcement, not to widen the period.

## FIXED 2026-09-09 (third find) — the sickness pre-check on the bot's mana estimate matched a bare `AddMana`, so a sick Wall of Roots' `Seq`-wrapped counter mana made the `AB_SAC` / `AB_SELF_COUNTER` gates unsound

The same sweep on the next box, `all` seed 737 (the sixty-first cell of
the run's 108,000 fresh-seed games): two gate audits aborted on one
thread each — "`sink::AB_SELF_COUNTER` skipped a real action: Basking
Broodscale ability 0, printed cost `{1}{G}`, available total 1 by_color
[0, 0, 0, 0, 0]; board [... Wall of Roots (sick)]" and the `AB_SAC` twin on
a Haywire Mite with `by_color [0, 2, 0, 1, 0]` and the same sick Wall. The
Crawler fix above reads a sick or tapped permanent per ability, but put a
cheap pre-check in front of the grant walk — "prints a non-tap
`Effect::AddMana`" — and Wall of Roots' "put a -0/-1 counter: add {G}" is
`Effect::Seq([PumpPT, AddMana])`: the wrapper failed the `matches!`, the
Wall was skipped whole, green read zero, and the engine paid `{G}` off it
(`effect_produced_colors` walks the `Seq`, which is how the payer saw it).

Fix (one commit): the pre-check asks `effect_produced_colors` — the read
the opaque arm and the payer already use — instead of matching the effect
shape, so the two cannot disagree on a wrapper again; a tapped land's
`{T}` ability still short-circuits on `tap_cost` before the walk. Priced:
sealed +0.014 % / cube +0.010 % / fixed +0.018 % Ir, 120 / 120 traces
identical (PERF, the 2026-09-09 second addendum). Tests:
`server::bot::tests::available_mana_sick_wall_of_roots_makes_the_colour_
budget_opaque` and `core_rules::golden_trace::the_sick_wall_of_roots_
pair_passes_the_gate_audit` (archetype 3 of the seed-737 cube pool, pair
seed 8709371159938462617, replayed under the suite's debug assertions).

The lesson is the Crawler entry's, one level down: **a gate's pre-filter
must be the same read as the gate**, or the pre-filter is a second walker
of the same tree and the pair drifts on the first shape one of them does
not name.

## FIXED 2026-09-09 (second find) — the bot's mana estimate skipped a summoning-sick creature whole, so a Crystalline Crawler's counter mana made the `AB_DAMAGE` gate unsound

The same sweep, one seed later: cube seed 729 aborted (rc 134) on the
`gated_pick!` audit in `main_phase_action_with` — "gate `sink::AB_DAMAGE`
skipped a real action: Pyrite Spellbomb ability 0, printed cost `{R}`,
available total 0 by_color [0, 0, 0, 0, 0]". The blame string now also
prints the seat's hand and board, and the replay (archetype 2, pair seed
6018027449014118342) read `hand []; board [... Crystalline Crawler (sick)
...]`. Crystalline Crawler's "remove a +1/+1 counter: add one mana of any
color" has no `{T}`, so it fires the turn it enters (CR 302.6 stops tap
abilities only); the engine paid `{R}` off it and the Spellbomb's
activation was real. `available_mana` skipped a summoning-sick creature
*before* reading its abilities, so the source never reached the opaque
arm that widens `by_color` to `u32::MAX`, and `colors_coverable`'s
"a shortfall is a proof" was false on that board.

Fix (one commit): the sickness check is per ability — `sick && a.tap_cost`
skips the `{T}` ones, and the rest still make the budget opaque. `total`
is unchanged (a counter-fed source is not spare mana, the same bias as a
Lotus Petal). Tests: `server::bot::tests::available_mana_sick_crawler_
makes_the_colour_budget_opaque` (opaque budget, zero total; a sick
Llanowar Elves still reads zero) and
`core_rules::golden_trace::the_sick_crawler_pair_passes_the_gate_audit`
(the pair replays to a decision under the suite's debug assertions).

Not a hole of the same shape: a from-hand mana ability (Simian Spirit
Guide) is a payment the engine's auto-tapper does not take either
(`hand_mana_source_could_pay` makes it a manual choice), so the estimate
and the engine agree on it. The audit itself is the finding's instrument
— a release binary would have played the game with one fewer ping and
nobody would have known; the debug-assertions sweep is where these live.

## FIXED 2026-09-09 — the mandatory-loop watchdog saw only period-1 loops; two Portable Holes and a mandatory Leonin Relic-Warder cycled three boards to the action cap

Found by the fresh-seed dflt sweep on the debug-assertions build: cube seed
726 capped its archetype-6 mirror pair (pair seed 11400714845093003057) at
6,001 actions on turn 11, `stack 1` throughout. `CRAB_CAP_DIAG` now prints
the stack item's target and the linked exiles, and the replay read
`trigger Portable Hole -> Portable Hole` over `exile: Leonin Relic-Warder
(until Portable Hole leaves)`. The cycle, three resolutions long: Hole A
exiles Hole B, whose return link brings the Warder back; the Warder's ETB
has only its controller's own Hole A to exile, which returns Hole B, whose
ETB exiles the Warder, which returns Hole A. Every state differs from the
one before it, so `mandatory_loop_watch` — one fingerprint, compared with
the previous resolution's — reset on every step and never fired.

Three fixes, each its own commit:

1. **The watchdog anchors a fingerprint and counts returns to it**
   (`GameState::mandatory_loop_watch` is `(anchor, repeats, since)`;
   `MANDATORY_LOOP_MAX_PERIOD = 8` resolutions before a chain that has not
   come back re-anchors). A loop of any period up to eight is a draw after
   `MANDATORY_LOOP_DRAW_REPEATS` returns; a progressing chain never returns
   and re-anchors every eight. Same size as before, no clone cost. Tests:
   `core_rules::cr_recent75::cr_104_4b_period_two_trigger_loop_draws_the_game`
   and `::cr_732_4_two_portable_holes_and_a_mandatory_warder_end_as_a_draw`
   (the found loop, played with an inline mandatory Warder).
2. **Leonin Relic-Warder is "you may exile"** (`Effect::MayDo`), as printed
   — any controller's artifact or enchantment, which the card does say.
   Test `modern::altars_flips_artifacts::leonin_relic_warder_may_leave_its_
   controllers_own_artifact_alone`; the existing exile test scripts the yes.
3. **The bot declines a "you may" removal aimed at its own permanent.**
   `optional_trigger_beneficial` could not see it: the target is chosen when
   the trigger goes on the stack and lives on the pending `ResumeContext`,
   not in the effect tree. `removal_targets_own_permanent` reads it there
   for the exile / destroy / bounce shapes. Test
   `server::bot::stack_response_tests::optional_removal_aimed_at_own_
   permanent_is_declined`; the pair itself is pinned by
   `core_rules::golden_trace::the_two_hole_relic_warder_pair_decides`.

The class: the resolution watchdog and the announcement guard (CR 732.3)
both compared a fingerprint with *one* predecessor, and a loop through
several distinct boards is the common shape (any two "until ~ leaves"
permanents with return links). The announcement guard is keyed on one
ability's repeats and does not have the period problem.

## FIXED 2026-09-08 (third run) — a permanent leaving the battlefield by dying or bouncing kept its damage, tap and attachment into its next zone (CR 400.7); a recast Golgari Thug died on entry every turn

Found by sweeping cube under the new battlefield bound: seed 702's dflt
mirror capped a pair at 50,000 actions on **turn 2,272** with libraries,
life and board unchanged for two thousand turns. The trace
(`CRAB_DUMP_TRACES`): on turn 5 both seats' Golgari Thugs trade in
combat; each Thug's dies trigger puts a creature card from its graveyard
on top of the library and takes itself; each seat draws it, recasts it
(the only spell it can cast), and it **dies on resolution with nothing on
the board to kill it** — then tops itself again. Reproduced in a unit
test: the card in hand after the top-deck still carried `damage: 1` from
the block, so the recast 1/1 entered with lethal damage marked and the
state-based check killed it.

Two leave-the-battlefield routes skipped the CR 400.7 reset that
`move_card_to` (the effect-driven mover) performs: `place_card_at_resolved_
zone` (every death and every exile through `remove_from_battlefield_to_
graveyard_raw` / `_to_exile`) reset "until end of turn" effects, cases,
rooms and copies but not `damage` / `tapped` / `attached_to`, and
`remove_from_battlefield_to_hand` cleared counters only — and the
resolving spell's own entry (`stack.rs`) reset nothing at all, so the
same trip also carried a **permanent-duration pump** (`perm_power_bonus`,
Wall of Roots's kind), a **paid echo** and an "enchant player" attachment
onto the new object (the test probes the pump: a recast Thug came back
+2/+2). The fix is split by what each side can know. **Leaving**
(`CardInstance::leave_battlefield_state`, from `place_card_at_resolved_
zone` and `remove_from_battlefield_to_hand` beside the tap): the
permanent pump, both attachment kinds and the paid echo, every write
guarded because each is a CoW copy of a card a bot clone may share and
nearly no leaving card holds any; the end-of-turn effects, saddle and crew
were already `clear_effects_on_zone_change`'s. **Entering**: the
resolving spell's push resets **damage only** — CR 400.7d keeps a
characteristic change applied to the permanent spell on the permanent it
becomes (`cr_recent50::cr_112_4_permanent_spell_pump_survives_resolution`
caught the first draft, which cleared the pumps there too) — while the
non-spell routes, `move_card_to` (reanimation, blink) and `play_land`
(behind the read-only `carries_old_object_state` probe), take the full
`enter_as_new_object` list; the phasing paths keep their state by
CR 702.26 and tokens are fresh. Measured reason for damage-at-entry:
`damage` sits behind the CoW `CardData` and is non-zero on nearly every
dying creature, so resetting it on the leave side deep-copies a card
every bot clone still shares — **+0.107 % sealed Ir on identical games**
(PERF Baseline addendum); no rule reads damage off a card outside the
battlefield. Not a bot defect: the bot was
right that the recast was free, and it was the engine that made it
worthless. Golden traces 7/7 unmoved (no seeded game reaches the case).
Regression test:
`core_rules::game::golgari_thug_recast_after_topdecking_itself_survives`.

## FIXED 2026-09-08 (third run) — a token-doubling board had no bound: the simulator now ends a game whose battlefield passes 1,024 permanents as a cap

Found by the run's own census: a `dflt` cube mirror at seed 43 (`--games
1200`) held one thread for over ninety minutes and never finished. Named
with the new `CRAB_MAX_ACTIONS=4000 CRAB_CAP_DIAG=1` pair (PERF "How to
measure"): at action 4,001, turn 55, the board carried **1,377 Scute
Swarms** and the stack **671 of their landfall triggers** — each land drop
doubles the copies (CR-correct: the trigger creates a copy of Scute Swarm
with six or more lands), every resolution is a priority stop the bot
answers over a board that size, and the 50,000-action cap was hours away.
The other 9,598 games of the run took 122 s on one thread.

The class is *finite exponential growth*, which no rule ends and the
action cap cannot reach in time. Fix: `recommend::MAX_BATTLEFIELD` (1,024)
and `StopReason::BoardCap`, checked by one shared `recommend::stop_reason`
that both driver loops — the ladder's `play_one_game_traced` and the
actor's `play_recorded_game_mcts` — now read, in the loop's own order
(rules, action cap, board cap, staleness). A board-capped game is counted
with the action-capped ones everywhere (`SimCost`, the actor's
`stalls_capped`, the grid's `cap` bucket), and `CRAB_CAP_DIAG` prints its
board. No legitimate 40-card game approaches a thousand permanents; the
robustness grid's caps stay a defect signal (the Beacon of Immortality
seeds are action caps, unchanged). Regression test:
`core_rules::golden_trace::a_board_past_max_battlefield_ends_the_game_as_a_cap`.

## FIXED 2026-09-08 (second run) — a combat-damage `dealer_filter` could never say `IsSource`, and "when you next attack this turn" had no primitive

Two card jobs that each turned out to be one engine line, found by
closing rows other passes had filed:

* **`fire_combat_damage_triggers` evaluated a trigger's `dealer_filter`
  with no source hint** (phases 1 and 1.6), so `SelectionRequirement::
  IsSource` inside one was always false — the "Calix **or** an enchanted
  creature you control deals combat damage" disjunction had been filed as
  unmodellable for that reason. The listener is the source now (`Some(c.id)`);
  no shipped card had used the field (the catalog's only other `dealt_by`
  is Cabal Slaver's creature-type filter). Calix, Guided by Fate carries
  its second ability and `TRIGGER_LIMIT_ABILITY_DROPPED` is empty;
  `recent_a::recent_111_117::recent114::calix_combat_damage_copies_an_
  enchantment_once_a_turn`.
* **`Effect::OnYourNextAttackThisTurn` / `DelayedKind::YourNextAttackThisTurn`**
  (CR 603.7e, one-shot, consumed by the dispatcher's attack-declared leg
  after the attackers are tapped, expired at cleanup). All-Out Assault had
  shipped its extra combat without "untap each creature you control", which
  left the second combat with nobody to attack in it;
  `recent_b::recent_223_237::recent226::all_out_assault_untaps_the_team_
  on_the_next_attack_once`. The `untap` row of the oracle-verb audit is down
  to Breath of Fury, whose untap lives inside a bespoke effect.

Robustness at the tip (PERF Baseline, the `(-277)` addendum): the wide
debug-assertions grid — 52 ladder cells / 301,600 games — read 0 panics,
0 assertion fires, 0 stuck, 4 caps, 12 draws; the four caps are seeds 53
and 73's Beacon of Immortality boards, the fingerprint the closed
stall-sweep entry below records (twin `i32::MAX` life totals), read with
`CRAB_CAP_DIAG=1`.

## FIXED 2026-09-08 — a trigger grant knew two durations, and a planeswalker never recorded who damaged it

The last arm of the duration axis (the entry below): `Effect::
GrantTriggeredAbility` had an EOT bucket (`granted_triggers_eot`, cleared
at cleanup) and a permanent one (baked onto the definition), so **Vraska
the Unseen's +1 — "until your next turn, whenever a creature deals combat
damage to Vraska, destroy that creature" — was permanent**, the only
catalog grant with a third duration. The map is `granted_triggers_timed:
HashMap<CardId, Vec<GrantedTrigger>>` now, each entry carrying the layer
system's `EffectDuration` from `effect_duration_for` (so "until your next
turn" is the controller's, CR 611.2b) and its source; `expire_granted_
triggers(pred)` runs where the matching continuous effects are swept —
turn start (`UntilNextTurn` / `UntilYourNextTurn`), upkeep
(`UntilYourNextUpkeep`), the end-of-combat step (`UntilEndOfCombat`, which
used to wait for cleanup), cleanup (`UntilEndOfTurn`) and the SBA sweep
(the three `WhileSource*` clauses, no catalog use yet). Every consumer
reads through `granted_triggers(id)` (an iterator over the abilities);
the dispatcher's per-permanent slice is the only direct read, and the
presence gates are unchanged. The two hooks the previous matrix left
unread — `fire_step_triggers` and `fire_spell_cast_triggers` — read it
now too, behind a kind-filtered presence gate that also widens their
member-list walk to the whole board (a grant can land on a permanent with
no printed trigger); `core_rules::cr_recent49::cr_603_2_instance_granted_
step_and_cast_triggers_fire`. **The three combat listener walks
(`ControllerAttackedByOpponent`, `YouAttack`, `ControllerDealtCombatDamage`)
read it too since 2026-09-08 (the following run)**, behind one shared
presence read, `any_granted_trigger_of_kind`, that the step and cast hooks
now go through as well — every hook outside the dispatcher asks the same
question the same way; `core_rules::cr_recent49::cr_603_2_instance_
granted_combat_listeners_fire`. Still printed-only: the self-ETB hook (a
permanent cannot carry an instance grant before it enters). Latent: a
granted trigger's `once_per_turn` is unenforced (sentinel index) — no
catalog grant carries the flag. **Both are ratchets now:**
`core_rules::catalog_registration::every_granted_trigger_kind_reaches_a_
walk_that_reads_its_bucket` serializes every factory, keys each static /
instance / attachment grant by the granted ability's (kind, scope), and
for every kind dispatched outside the generic dispatcher asserts the
bucket against a hand-read table of walks (`READS`) — a new grant of a
push-site kind through a bucket its walk does not read fails, and so does
any granted `once_per_turn`. The same census answered the one question the
first pass had guessed at: the catalog's step and cast instance grants
(Veiled Apparition, Obsidian Fireheart, Glyph of Delusion, Great Hall of
the Biblioplex) are all `Duration::Permanent`, which bakes onto the
definition — nothing shipped dead through those two hooks.

And one more parallel walker, found by the catalog's step-gate pass the
same day: `gather_continuous_effects_inner`'s live-filter leg for
`SetBasePtForFilter` / `GrantKeyword` (filters `selector_to_affected`
cannot decompose — `IsOutlaw`, `IsEnchanted`) matched `sa.effect` raw
while the eager leg and the layer path peel the `WhileYourTurn` /
`WhileClassLevelAtLeast` / `WhileCountersAtLeast` / `WhileCondition`
wrappers through `active_static`, so a wrapped live-filter grant emitted
nothing on either path ("during your turn, outlaws you control have first
strike" granted nothing on any turn). The leg peels through
`active_static` now; `core_rules::cr_recent49::cr_611_2_your_turn_wrapper_
reaches_a_live_filter_keyword_grant`. No shipped card carried the shape (At
Knifepoint ships the `only_your_turn` anthem).

The test for it found the second half: **the planeswalker branch of
combat damage (`deal_combat_damage_to_target`) and of `deal_damage`
removed loyalty and recorded nothing on the victim** — no
`damaged_by_this_turn`, no `record_damage_from` — so `Selector::
LastDamagerOf(This)` on a planeswalker was always empty and the +1 would
have destroyed nothing even as a permanent grant. Both branches keep the
creature branch's record now. Tests: `classic_sets::rtr::vraska_the_
unseen_plus_one_spans_the_opponents_turn_and_then_expires` (the grant
survives cleanup, destroys the attacker on the opponent's turn, is gone
as her controller's turn begins, and a later attacker is spared) and
`…_ignores_a_pinger` (combat-only, and the damager is on record).

## FIXED 2026-09-07 — keyword grants with an "until your next turn" duration were permanent, and `UntilNextTurn` itself meant "any player's next turn"

The fourth find of the consumer-read method, from the *duration* axis:
for each effect arm that matches on its `duration`, which `Duration`
variants does it actually distinguish? `GrantKeyword`, `GrantKeywords` and
`GrantProtectionFromChosenColor` knew two buckets — `EndOfTurn |
EndOfCombat` to the per-instance EOT bag, everything else baked onto the
definition's printed keyword list — so **every "until your next turn" /
"until your next untap step" / "while this is tapped" keyword grant was
permanent**: Academic Probation's "can't attack / can't block / can't
activate until your next turn" never ended, The Akroan War's and Nahiri
the Unforgiving's "attacks each combat if able" was forever, Bond of
Revival's haste, Emeria's Call's, Venat's and For the Common Good's
indestructible, Elspeth Storm Slayer's flying, Vivien's vigilance and
reach, Erhnam Djinn's forestwalk, Spatial Binding's phase lock, Gabriel
Angelfire's chosen keyword, Crown of Vigor's tapped-only grant — thirteen
shipped cards. `LoseKeyword` had the mirror pair (`Permanent` or the EOT
bag), so Ertai's Familiar regained phasing a turn early.

The layer system already carries every one of those durations
(`EffectDuration::UntilYourNextTurn { player, installed_turn }`,
`WhileSourceTapped`, …) and `PumpPT` / `SetBasePT` already route through
`effect_duration_for`. The fix is one carrier choice, `grant_keyword_for`:
EOT / end of combat to the bag, `Permanent` to the printed list, anything
else a layer-6 `AddKeyword` (`RemoveKeyword` for the loss) continuous
effect under `effect_duration_for` (`keyword_layer_effect`).

The same read then found the enum itself wrong: `Duration::UntilNextTurn`
was documented and mapped as "the start of the next turn (any player's)",
and **all thirteen catalog uses print "until your next turn"** — so the
PumpPT-style ones (Liliana, the Last Hope's -2/-1, Mouth of the Storm's
-3/-0, Fear of Falling, Azure Beastbinder, Mila) wore off as the
opponent's turn *began*, before the combat they were cast to blunt.
`effect_duration_for` now maps it beside `UntilYourNextUntap` (CR 611.2b)
and the variant's doc says so; the controller-less `map_effect_duration`
keeps its old arm for a caller without a seat (there is none today).

Tests: `core_rules::cr_recent35::cr_611_2b_keyword_grant_and_loss_honour_
until_your_next_turn` (three durations, grant and loss, across the turn
cycle) and `classic_sets::eoe::mouth_of_the_storm_shrinks_opponents`
(extended across the opponent's turn).

The same axis, one more arm: `GrantActivatedAbilityToMatching` returned
early for every duration but `EndOfTurn`, so **Life Matrix's `Permanent`
"remove a matrix counter: regenerate" grant put a bare counter on the
creature and nothing to spend it on**. It has `GainActivatedAbility`'s two
carriers now (the EOT list, or the permanent list deduped — the filter
re-matches every earlier recipient on each resolution);
`classic_sets::leg4::life_matrix_grants_the_regeneration_ability`. A
census of every `run_effect` arm that returns `Ok(())` on a field's value
finds no other such guard, and every other duration-carrying arm either
maps through `effect_duration_for` or sees only the durations the catalog
hands it. Beside it, a catalog shape: five "activate only during your
upkeep" abilities (Clockwork Swarm, Black Carriage, Trade Caravan, Life
Chisel, Life Matrix) gated on `CurrentStepIs(Upkeep)` alone, which is any
player's upkeep; each is `All([CurrentStepIs, IsTurnOf(You)])` now, and
the four that print "any upkeep" / "during the … step" were left alone. The
one arm left latent here — `Effect::GrantTriggeredAbility` with
`UntilNextTurn` (Vraska the Unseen's +1), baked as permanent because the
EOT trigger map had no duration stamp — is the entry above this one.

## FIXED 2026-09-07 — the LKI walk dropped until-end-of-turn granted triggers, and a conditional Equipment rider granted its abilities unconditionally

The third find of the consumer-read method, this time as a **consumer x
source matrix** for the trigger-grant family (printed / `granted_triggers_
eot` / `GrantTriggeredAbility` statics / `turn_granted_triggers` /
`equipped_bonus` / `soulbond_bonus` / station bands, against the
dispatcher, the cast, attack, combat-damage, step and self-ETB hooks, the
death path and the dispatcher's LKI walk). One cell was a shipped bug:
`dispatch_triggers_for_events`' `died_card_snapshots` walk read the
snapshot's printed triggers plus `statics_granted_dying_triggers` and never
`granted_triggers_eot`, so an until-EOT grant of a kind that fires from LKI
was dead on the leave. **Requiem Monolith's own play line — grant the
"whenever this is dealt damage, draw that many and lose that much" and ping
an X/1 — drew nothing**, because the creature died to the ping and the
`DealtDamage` copy lives on the EOT list. The map is cleared only at
cleanup, so the fix is one `.chain(self.granted_triggers(snap.id))` on the
printed side of the walk (not `is_granted`: the death path already
collects the EOT dies copy, exactly as it does the printed one).
`classic_sets::eoe2::requiem_monolith_draws_off_a_lethal_ping`.

The rest of the matrix, so nobody re-walks it: the step, cast and self-ETB
hooks do not read `granted_triggers_eot` either, and no catalog EOT grant
is of a kind they own (the 20 `Effect::GrantTriggeredAbility` EOT sites
are `DealtDamage`, `CreatureDied`, `DealsCombatDamageToPlayer`, `Attacks`,
`BecomesBlocked`, `PermanentLeavesBattlefield`, `DealtCombatDamage` —
every one on a hook that reads the list). Permanent-duration grants bake
onto the definition and reach every consumer. `soulbond_bonus.triggered_
abilities` is read only by the combat-damage hook, and Tandem Lookout is
the only card that sets it. `turn_granted_triggers` (`Blocks`,
`DealsCombatDamageToPlayer`) reaches every consumer through
`statics_granted_triggers_inner`.

The same read on `EquipBonus`: `ConditionalEquipBonus` carries a
`condition` the layer walk applies to the rider's P/T and keywords and
`granted_abilities_of` skipped for the rider's `activated_abilities` (it
read `host_filter` only). No shipped card sets both — 17 riders, none
with a `condition` and abilities — so it is pinned by a synthetic
Equipment, `core_rules::cr_rules::cr_702_6e_conditional_equip_ability_
grant_honours_its_condition`. `ActivatedAbility`'s ~80 fields were
censused too: every one is read inside `activate_ability_inner` (the
single engine consumer; `affordances.rs` dry-runs it, `server/bot.rs` is a
prefilter), so that family has no parallel walker to drift from.

## FIXED 2026-09-07 — the attack and combat-damage hooks dropped `once_per_turn`

The second find of the same shape as the entry below it, from the same
method (read a definition field's consumers): `EventSpec::once_per_turn`
is gated by the dispatcher (`triggered_once_per_turn_used`, keyed
`(source, printed index)`) and by the cast-trigger walk — and by neither
of the two hooks that push their kinds themselves. `declare_attackers_
banded` carried `(source, effect, controller, filter)` and nothing else,
so **"whenever Aurelia attacks for the first time each turn" fired on her
second attack too**: the extra combat she bought untapped the team and
bought a third, and so on until the turn cap — Aurelia, Godo, Scourge of
the Throne and Fear of Missing Out all print the clause and all four were
an unbounded-combat loop in self-play whenever the bot kept attacking.
`fire_combat_damage_triggers` runs once per dealer, so a `YourControl`
"one or more creatures you control deal combat damage" listener with the
flag (Vaan, Yarus, Frostcliff Siege, Mu Yanling, Invasion Tactics) fired
once per connecting creature.

The fix threads the key through both: the attack hook's tuple and
`DamageTrigger` gain an `Option<usize>` (`once_key`: the printed index
when the ability says once, `None` for a granted one — the dispatcher's
`trig_idx < n_printed` rule), and each consumer inserts into the same set
*after* its intervening-if passes, so a rejected declaration does not
spend the slot (CR 603.4, the dispatcher's order). The step-trigger walk,
the SelfSource ETB push and the death path were read too: no shipped card
puts `once_per_turn` on a kind they own (the catalog scan is
`scripts`-less: every `once_per_turn` keyed by the nearest `EventSpec`).
The inverse was scanned too: every oracle trigger line printing "only
once each turn" / "for the first time each turn" has a once gate in its
code — the nine Valiant cards through `shortcut::valiant()`, Venat through
`CastSpellFirstMatchingThisTurn` — so nothing is missing the flag.

Tests: `recent_b::recent_208_222::aurelia_does_not_fire_on_her_second_attack`
(no third combat, the team stays tapped),
`classic_sets::fin::vaan_exiles_once_when_two_thieves_connect`.

The same read, one more field: `EventSpec::dealer_filter` ("whenever a
Goblin deals combat damage to a player") was applied by the dispatcher
and by the hook's Phase 1.6 (a bystander's `AnyPlayer` trigger) and not
by Phase 1 (the dealer's *own* `AnyPlayer` trigger), so Cabal Slaver — a
Cleric — stripped a card off its own hit. Gated the same way now;
`classic_sets::ons::cabal_slaver_does_not_punish_its_own_hit`. Bellowing
Fiend, the other `dealt_by` card, is a `DealtDamage` listener the
dispatcher owns and was right. The remaining `EventSpec` fields
(`actor_is_opponent`, `exclude_attacker_taps`, `exclude_tap_cost_
abilities`, `per_subject_cap`) are read only by the dispatcher and are
only set on dispatcher-owned kinds — checked, nothing to do.

## FIXED 2026-09-07 — `triggers_on_equipment` was honoured by two hooks and dropped by the dispatcher

`EquipBonus::triggers_on_equipment` ("the granted trigger fires off the
Equipment, so `This` is the Equipment" — Jitte) was read by the
combat-damage hook and the step-trigger walk, and `equip_granted_trigger_
sources` / `dispatch_scan_bits` *excluded* every flagged attachment from
the general dispatcher on the theory that the combat hook owned them. So
any flagged trigger on a non-combat, non-step kind never fired: Godsend's
"whenever equipped creature blocks" (`Blocks`) and Crystalline Nautilus's
bestowed "when this creature becomes the target" (`BecameTarget`) were
dead, with no test on either. Mask of Griselbrand's dies trigger fired
only because the death path ignores the flag and collects every
attachment's `CreatureDied` with the creature as source.

The fix is one shape everywhere: a grant is `(host, source, abilities)`,
the host is always the event subject and the source is the attachment
under the flag. `equip_granted_triggers_with` returns `(source, ability)`
pairs, the three consumers (the dispatcher, `fire_spell_cast_triggers`,
`declare_attackers_banded`) push that source, and the two death-path
collectors do the same with the dying creature bound explicitly as the
trigger's subject (`TriggerPush::trigger_source`). No double-firing:
the combat-damage kinds and `StepChanged` never match in the dispatcher.

Catalog, the same commit: Godsend fires on `Blocks` *and* `BecomesBlocked`
and exiles *one* partner (`ChooseOneAmong`); Crystalline Nautilus drops
the flag (the printed "sacrifice it" is the host, CR 702.6e —
`SacrificeSource` under the flag would have sacrificed the Nautilus);
Kusari-Gama gains it (its splash is the Equipment's damage — a lifelink
bearer gains only for its own hit); Impending Doom moves its burn onto the
Aura's own `EnchantedBySource` trigger, Death Watch's shape, dealt to the
creature's controller by the Aura.

Tests: `classic_sets::jou::godsend_exiles_a_blocker_with_the_equipment`,
`jou::bestowed_crystalline_nautilus_sacrifices_the_targeted_host`,
`chk2::kusari_gama_splashes_onto_the_other_defenders` (the lifelink
assert), `thb::impending_doom_burns_on_death` (now through the SBA death
path, not a hand-built context), and the catalog-independent pin
`core_rules::cr_rules::cr_702_6e_flagged_equip_trigger_fires_through_the_
dispatcher` (a synthetic Equipment, `Tapped/SelfSource`, the counter lands
on the Equipment).

Checked and clean, so nobody re-checks: `once_per_turn` on a *granted*
trigger is skipped by the dispatcher (grants carry the `usize::MAX`
sentinel index, and the CR 603.3d gate is `trig_idx < n_printed`) — a
brace-nesting scan of the catalog finds **zero** `once_per_turn: true`
triggers inside an `equipped_bonus` / `GrantTriggeredAbility` /
`soulbond_bonus`, so it is a latent gap, not a shipped bug. The first card
to need it keys the gate on `(source, grant index)` instead.

## FIXED 2026-09-07 — "leaves the battlefield" fired only on death, and the untap step minted one trigger for a board

Found by the `trig` audit column (INCOMPLETE_CARDS, "Trigger events"):
Thragtusk's Beast moved to `PermanentLeavesBattlefield` and its bounce
test failed, because `event_kind_bits` / `event_kind_matches` paired that
kind with `GameEvent::CreatureDied` and nothing else. The non-graveyard
exits (the `Move` path, `remove_from_battlefield_to_hand`, the exile
path, Glimpse of Tomorrow, meld) reported `CreatureLeftWithoutDying` for
creatures — a different kind, used by two cards — and nothing for the
87 `PermanentLeavesBattlefield` literals. Every one of them was a dies
trigger in practice; every test of one killed the permanent.

The fix is one new event, `GameEvent::PermanentLeftBattlefield {
card_id, controller }`, pushed by `GameState::note_left_without_dying`
beside `on_left_battlefield` at each of those exits. The `SelfSource`
half needs the card after it has gone, so the same helper snapshots the
leaver into `died_card_snapshots` and the dispatcher's LKI walk gained a
`lki_self_left` arm that pairs a `SelfSource` leaves trigger with *that
event only* — a death's copy of the trigger is still collected before
removal in `remove_to_graveyard_with_triggers`, so nothing double-fires.
Scope / subject / actor plumbing in `events.rs` treats it as
`CreatureLeftWithoutDying`'s sibling; `GameEventWire` mirrors it.

The same run: `BecomesUntapped` was absent from the CR 603.6 fan-out
list in `push_ordered_trigger_candidates`, so an untap step untapping
three permanents minted one Mesmeric Orb trigger. Added, and `Tapped`
with it the same day (Verity Circle drew once for two creatures tapped by
one effect — `classic_sets::rna::verity_circle_draws_once_per_creature_
tapped_in_a_batch`, which fails without the entry).

Tests: `modern::decks_16_17_misc::thragtusk_bounced_still_makes_a_beast`,
`classic_sets::rav::twilight_drover_grows_on_token_bounce`,
`modern::cascade_dredge_auras::rancor_exiled_from_the_battlefield_stays_in_exile`
(the dies-only card must *not* hear the new event),
`modern::coverage_backfill::mesmeric_orb_mills_one_per_permanent_untapped`.

## The `debug-assertions` sweep found FIVE real defects, and the committed grid was green on all of them

**Run because nothing had, for thirty-odd passes.** `RUSTFLAGS="-C
debug-assertions=yes" CARGO_TARGET_DIR=target-audit cargo build --profile
overflow`, then `--a gang --b gang --games 400 --threads 3 --decks all` over 26
seeds (176,800 games), plus 20 seeds of `--decks sealed` (96,000 games — the
deck builder a training actor runs twice a game), six of `--decks sos`, and the
new `--pilots` leg. The plain `release-fast` sweep this file already carries
runs the same games and reads them all clean; **the two profiles are not the
same instrument**, and the difference is five shipped bugs.

```text
  what fired                                                   where
  1  debug_assert  main_phase gate sink::AB_TOKEN skipped ...   seed 2
  2  overflow      bot.rs  (life + clock - 1) / clock           seeds 53, 73
  3  overflow      bot.rs  (4*hand + emblems + crown) * unit    seeds 53, 73
  4  cap 50000     a counter cost left its dead source in play  --pilots
  5  cap 50000     the CR 732.3 guard counted its own stack     --pilots
```

⚠ **AND THE COMMITTED DEFAULT GRID IS GREEN ON THE TREE THAT HAD ALL FIVE.**
Measured, not argued: a `debug-assertions` binary built at the pre-fix commit
runs `scripts/robustness_grid.sh`'s 30-cell default (five pools x six seeds x
120 games) with **0 failures**. 1-3 sit on seeds the default list does not carry
and 2-3 need a board that takes hundreds of turns to build; 4-5 need a pilot
other than `gang`, which the grid had never run. That is why the script now has
`--wide` and `--pilots`, and why its header carries this table.

**All five are silent in release.** A `debug_assert!` is compiled out; an
overflow *wraps*; an unbounded stack only shows as a capped game somebody has to
be looking at. That is the whole argument for the profile: the five syntax
filters in this file cannot reach any of them, and the suite cannot either — a
wrap needs a board, and 19,201 tests carry fewer interesting boards than 400
games of `--decks all`.

**(6) — found at the presence-flag pass, `d98c4212`: a permanent leaving the
battlefield kept its "until end of turn" effects.** A creature bounced with
"can't block" and recast the same turn still could not block; a pumped one
came back pumped. Neither leave-the-battlefield bundle (`move_card_to`, the
`remove_*` raw path) touched the `eot_wear_off` fields. What found it was
not a test but a **memo audit**: `board_instance_keywords` (PERF `(-190)`)
is exact after cleanup and set by every grant, so a card that *re-entered*
carrying a grant nobody made was the one state the flag could not explain,
and the `debug_assert!` said so on six grid cells. **A presence flag's
audit is a free invariant check on every write path that feeds it** — the
fix (`clear_effects_on_zone_change`) keeps the per-turn *history* fields,
which are LKI for the death triggers (Scythe of the Wretched pins it).

**(1) A graveyard-only ability functioned on the battlefield.**
`activate_ability_inner` had four wrong-zone gates and only in one direction.
Eternal Student's "{1}{B}, Exile this card from your graveyard: create two
Inklings" was activatable while it was a 4/2 in play, and the payment path —
asked to exile a graveyard card that is on the battlefield — **accepted the
activation without the mana** (one blue against `{1}{B}`). 103 catalog abilities
carry `from_graveyard`, 40 `from_hand`, 4 `from_command_zone`. The mirror gate
closes the class.
⚠ **The bot's presence-gate audit is what caught it, and it caught an ENGINE bug
rather than a gate bug** — `colors_coverable` was right and `would_accept` was
wrong. A gate audit that fires is not automatically a gate to widen.

**(2) and (3) are the same board: Beacon of Immortality, twice.** The closed
stall lead above says life doubles to `i32::MAX` and the game caps. It does
more than that on the way: `pick_attacks_inner`'s race check
`(life + clock - 1) / clock` wraps *negative*, which reads as "the opponent
kills us before our next untap", so **the bot stops racing exactly when it is
being raced**; and `life_value` multiplies the total three different ways, so
the whole material term wraps and **the seat with unbounded life evaluates as
the one that is losing**. `turns_to_lethal` removes the addition; `life_value`
clamps at 10,000, far past anything the evaluator has to tell apart.

⚠ **The transferable half: a *closed* stall lead is not a closed board.** This
file recorded Beacon as "a correct card doing what it prints, 4 games in
183,600" and stopped there. Every arithmetic path that reads a life total was
wrong on those four games, and two of them changed what the bot *did* rather
than only what it reported. **When a sweep files a board as benign, ask what
else reads the number that made it unusual.**

⚠ **And the same for a capped game.** `undecided_by cap N` was read as a turn
limit and left there; each of the two caps in `abilarms`' `cube` cell was ~50,000
copies of *one ability* on *one stack*, i.e. two engine bugs wearing a rate.
`CRAB_CAP_DIAG` names the board in one line and cost nothing to run.

### The same profile on the ACTOR, which is the binary a training run uses

`bot_ladder` is a proxy. `selfplay_train` is the program, and it runs code
`bot_ladder` never does — the encoder, the sealed deck builder (twice a game),
the replay window and its reuse cap. Built the same way and run clean:

```text
  RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit \
    cargo build --profile overflow -p crabomination_ml --bin selfplay_train
  CRAB_NO_JITTER=1 RUST_MIN_STACK=33554432 target-audit/overflow/selfplay_train \
    --actors 3 --games 60000 --steps 40 --seed 20260901 --window 250000 --out /tmp/actorprof

  60,000 games / 5,785,847 rows / 0 stalls / 0 panic / 0 assertion, 389 s
  plus 5 seeds x 6,000 games at --window 20000: 0 panic, 0 stalls
  re-run at the pass's closing tip, after all five fixes, 3 seeds x 20,000:
    60,000 games / 5,773,403 rows / 0 stalls / 0 panic, 157-160 games/s
```

⚠ **The `--window 250000` leg matters on its own**: `(-52)`'s correction says
the window is ~10.4 KiB a row over a ~370 MiB floor, so 250 k rows is the
configuration that actually fills, and 5.8 M rows pushed through it is the first
time this file has run the reuse cap under arithmetic checks.


### The `--pilots` leg found two unbounded stacks, and one of them means the CR 732.3 guard never worked

The grid ran one pilot (`gang`) for its whole history. Running the other
forty-five against it turned up exactly one cell that could not finish 680
games in fifteen minutes — `abilarms` on `--decks cube` — and
`CRAB_CAP_DIAG=3000` named both causes in one line each:

```text
  cap: 50000 actions, turn 4,  stack 49905   ability Blinking Spirit x49905
  cap: 50000 actions, turn 25, stack 49427   ability Greater Good    x49427
  --a abilarms --b gang --games 4 --threads 1 --seed 23 --decks cube
    before  846,610 ms   30 decided / 2 undecided / 2 capped
    after       304 ms   32 decided / 0 undecided / 0 capped
```

**2,785x on that cell, and the two capped games decide.** `--bench` is
byte-identical to the invariant, so neither fix moves the default pilot's play,
and `robustness_grid.sh --wide` at the fixed tip reads **52 ladder cells /
301,600 games, 0 failures** (cap 4 / stuck 0 / draw 26 — the four caps are the
Beacon board below, unchanged).

**(4) A counter cost that kills its own source left the corpse in play.**
CR 704.3 — the activating player receives priority as soon as the ability is on
the stack — so state-based actions belong there, and the engine checked them
only on stack resolution. A Devoted Druid (0/2, "put a -1/-1 counter on this:
untap this") ran to toughness **-6** and kept untapping. Walking Ballista is the
mirror: a 0/0 that removes its last +1/+1 to ping, whose test asserted the old
behaviour and now asserts the rule (it dies, and CR 608.2 still resolves the
announced ping).

✅ **THE GENERAL CR 704.3 SWEEP IS LANDED** (`GameAction::pays_a_cost` —
every `Cast*` and non-loyalty activation — sweeps in `perform_action_inner`
*after* the action's own `dispatch_triggers_for_events`, which is the
placement the Pest Brewmaster interleave needed; the counter-cost-scoped
sweep in `activate_ability_inner` is gone). What made it land was closing
the **0/0 window at entry** first, in its own commit: a token "with N
counters" now carries them as part of the mint
(`TokenDefinition::enters_with_counters`, evaluated with the token on the
battlefield, placed before `PermanentEntered` and its ETB triggers — the 70
`Seq([CreateToken, AddCounter { LastCreatedToken }])` sites and the
`create_token_with_counter` shortcut all fold into it), nine real cards
that modelled a printed "enters with N counters" as an ETB *trigger* ride
`enters_with_counters`, and the land drop applies its printed counters
before its ETB triggers like the other three entry paths. With that in,
the sweep broke **12 tests, every one a fixture**: eight seeded a printed
0/0 Fractal card bare through `add_card_to_battlefield` (now
`move_card_to_battlefield_for_test`, which applies its entry counters),
two seeded a bare Aura (CR 704.5m sweeps it), one two copies of a
legendary planeswalker under one controller (CR 704.5j), one a 0/0 whose
converge count is cast-time. ⚠ **A fixture that puts a printed 0/0, a
bare Aura or a duplicate legend on the battlefield and then casts a spell
is now testing the sweep, not the card.** The sweep is gated on the
payment's events (`GameEvent::inert_for_state_based_actions`; ungated it
was +1.7 % of `fixed`, gated +0.12 % — PERF's Log has the row), and the
gate is why the four remove-counters-as-cost payments now emit
`CounterRemoved`: they moved counters silently, and Walking Ballista's
last +1/+1 would have slipped past. ⚠ **A cost path that mutates the board
without an event defeats the gate; audit new cost kinds for their event.**

Still open in the same family, sized and left: `apply_as_enters_effect`
runs a full `resolve_effect` (which can sweep) before the entry-counter
block in both the cast path (`stack.rs`) and the move path
(`movement.rs`), deliberately — Mimeoplasm's count reads what the
as-enters effect did — so a 0/0 with an as-enters effect *whose body
sweeps* would die before its counters. Two shipped cards carry the pair:
Riptide Replicator (an artifact, nothing to die) and Mimeoplasm, whose
as-enters body is the dedicated `AsEntersExileFromYourGraveyard` (no
sweep inside it; `mimeoplasm_exiles_for_counters_then_copies` pins the
order). A card that pairs a 0/0 body with a *generic* as-enters effect
would be the first to hit it.

**(5) The CR 732.3 loop guard hashed a fingerprint that counts the stack.** An
announcement is exactly a `stack.len() + 1`, so the watch never matched its own
previous call and the cap of 50 was unreachable. It also only watched
`is_free()` abilities — and a cost spelled in the *effect* (Greater Good's
sacrifice) is paid at resolution, so the printed cost is not the question.
⚠ **The one test of the guard drained the stack between activations**, which is
the single shape a real loop does not take: it kept `stack.len()` at 0, the
fingerprint matched, and the guard looked like it worked for its whole life.
**A test that arranges away the thing under test is worse than no test**, and
the new sibling test runs the same loop without resolving.


**Re-run at `82a3953e` (after `(-175)` / `(-176)`), fresh seeds:** the same
`debug-assertions` + `overflow` binary, `--games 400 --threads 2 --decks all`
over seeds 101-108 (54,400 games) and `--decks sealed` over seeds 101-104
(19,200 games). **0 panic, 0 assertion, 0 overflow, cap 2 / stuck 0 / draw 4**
— the two capped games are seed 108's twin `i32::MAX` life totals on the
Beacon of Immortality board below (`CRAB_CAP_DIAG=1`: turn 1,837, `lib 0` /
`lib 1`, 43 basic lands), the fingerprint this file says is not a finding.

## CLOSED — the two stall-sweep leads, and why neither is a bug

⚠ **Read the section above first: the Beacon lead is closed as a *stall* and was
not closed as a *board*.** Two of the bot's arithmetic paths wrapped on it, both
silently in release, and neither shows in any `release-fast` sweep.

A `--games 400 --decks all` sweep over 27 seeds (183,600 games at `22a79dcc`)
found **no panic, no hang, 4 capped, 22 draws**. Both leads are closed as
*printed cards working correctly*. **Do not re-open them.**

**A fourth re-run, on twenty seeds no sweep or grid had carried (primes
103..199, 2026-09-04, the `(-247)` tip's tree, debug-assertions overflow
build): 200,000 games — 20 x `--decks all` x 400 and 20 x `--decks cube` x
400 — 0 panics, 0 assertion fires, 0 stuck, 26 caps, 24 draws.** Every cap
is seed 149 or 193 (6 + 8 on `all`, 6 + 6 on `cube`, the same pairing in
both pools) and every one is fingerprint (a) below — twin `i32::MAX` life
totals, a library of one card, turn ~2,200 — on Orzhov and Azorius boards
that differ from the recorded ones only in what else is on them. **What is
new is the price, not the board:** a capped Beacon game runs ~50,000
actions on a 40-permanent board and costs ~100 s of a thread on the
overflow build, so seed 193's `all` cell took 892 s against ~85 s for its
neighbours and its `cube` cell 488 s against ~40 s. At 14 caps in 136,000
`all` games that is a ~2 % rate of cells carrying a 10x tail; an actor run
on a pool with the card pays it too. The action cap is the engine's clock
and lowering it is a harness decision (a legitimate Scute Swarm game
decided at 9,223 actions), so this is recorded, not changed.

**Re-run three times before that and both leads reproduce verbatim, which is the
point of recording their fingerprints here**: 81,600 games / 12 seeds at
`bd42107c` (cap 2 / stuck 0 / draw 14), again at `d825411f` after seven card
rewrites (cap 2 / stuck 0 / draw 18), and **68,000 games / 10 seeds after
`(-158)`'s `SmallIdSet` / `SmallIdMap` swap — the first change to reach
`declare_blockers` and `pick_blocks_inner` in a while — 0 panic, cap 4 /
stuck 0 / draw 22**. `CRAB_CAP_DIAG=5000` names the same two boards
each time — Scute Swarm x4,091 at turn 46 and the twin `i32::MAX` life totals
at turn 2,159. **A capped game whose diagnostic matches one of the two boards
below is not a new finding**; one that does not is.

**(a) The four capped games are Beacon of Immortality.** "Double target
player's life total. Then shuffle this into its owner's library."
`CRAB_LIFE_WATCH=1000` prints the series — 1,580 → 3,161 → 6,322 → 12,644, one
doubling every other turn — so ~31 casts saturate `i32::MAX` and **neither
player can lose to damage**, while the shuffle-back keeps the library from ever
emptying (`lib 1` at turn 2,159). Paper ends this by agreement or a clock; the
action cap is this engine's clock. 4 games in 183,600.

**(b) The nine-minute game is Scute Swarm** — 4,091 copies of itself on the
board at turn 46, the printed landfall doubling. Every board walk is O(4,091),
so 9,223 actions take 597 s. The game *decides* (the opponent is at −4,072), so
**no undecided count can ever see it**; `CRAB_CAP_DIAG=5000` is what found it.

**(d) A slow decided game with a dozen permanents of one creature's name is
Mirrorform** (2026-09-09, `planner` cube seed 773: 4,553 actions at turn 60,
Soulherder x12 on a deck that holds one). "Each nonland permanent you
control becomes a copy of target non-Aura permanent" is the printed card,
`duration: Permanent` is CR 707.2's indefinite copy, and twelve Soulherders
blink each other twelve times an end step. The deck check that proved it
(`cube_deck` at the pair's seeds, one Soulherder, every non-basic under the
four-copy cap) is the first thing to run on a board like it.

**(c) A cell with a dozen `draw`s is Flame Rift (2026-09-09, cube seed 766:
12 draws, six pairings x both seat orders, 0 elsewhere in 257,600 games).**
"Flame Rift deals 4 damage to each player" with both seats at 4 life or
less is a simultaneous loss, CR 104.4a. The trace ends `CastSpell { target:
None }` → `life -1/0` → `= winner none`. The bot casts it because
`eval_material_inner` scores a decided draw at 0 and the board it holds
below that — a strength question for an ML session (a draw is worth 0 only
when the alternative is worse), not an engine defect, and `draw` is already
outside the sweep's failure count.

⚠ **The transferable half: a correct board can be quadratic.** An actor's
throughput has a tail that is not a defect, and the two env-gated instruments
above are how to tell one from the other before spending a build:
`CRAB_CAP_DIAG=1` names a capped game's cause in one line, `CRAB_CAP_DIAG=<n>`
names *any* game past `n` actions — the only way to see a **slow** game, since
one that decides is never "undecided". A `--games 400 --decks all` sweep is
20–30 s a seed and catches what `robustness_grid.sh` does not (the grid is 120
games a cell); run it before the grid.

## Targeting — a fifth rule, from the field question rather than the wrapper one

`audit_target_walkers.py` asks whether a walker recurses into a **wrapper**.
`audit_target_fields.py` (new, hundred-and-fifteenth pass) asks the other half:
inside one arm, does `requires_target` read every field that can *hold* a
target? It found 21 (variant, field) pairs a shipped card aims and the walker
never looks at; twenty are fixed, one allowlisted, and six were a wrong
*answer* rather than an unread field — Autumn Willow, Questing Phelddagrif,
Guiding Spirit, Wand of Denial, Carrion Beetles, Rapid Decay.

**Rule 5: a variant joins a shared arm for the field it has, and the arm never
grows the field it doesn't.** Four of the five variants behind those six sat
in one `| … => false` group. `GainControl { what, to }` — the case that
started this — is the same shape one arm over.

**Rule 5a: `requires_target` and the two filter walkers are one answer, not
two.** Making `requires_target` honest for those six failed
`core_rules::target_walkers::every_targeting_spell_or_ability_says_what_it_targets`
in the same run: a body that needs a target and gives slot 0 no filter is
offered `SelectionRequirement::Any` — every permanent *and* a player. Any fix
to one walker is a fix to all three, through `IMPLICIT_PLAYER_TARGET`.

**Rule 5b: an arm for a chooser above a shared `body` arm shadows it.**
`MayDoBy { who }` placed above `MayDo { body } | MayDoBy { body }` silently
takes the body's filter away. Clippy's `unreachable_pattern` catches it; merge
into the existing arm (`who` first, body as the fallback) rather than adding
one above.

## Targeting — the four rules pass 104 closed on

Moved verbatim from TODO's NEXT (which is capped at ~15 lines) at the
hundred-and-seventh pass. The lane is closed and gated; these are the
rules, not a status.

**TARGETING IS CLOSED AND GATED (pass 104's other half, `13435f3e`..`9fec2a6f`).**
A slot with no filter enumerated against `Any` and was re-checked nowhere, so the
printed noun was enforced nowhere: **Terminate destroyed any permanent, Zombify with
an empty graveyard STOLE a creature, Banefire offered a Forest, "target player
discards" offered the board.** 79 reanimation filters, 20 per-card nouns, ~50 walker
arms; three invariants now gate it
(`every_reanimating_move_says_which_zone_its_target_is_in`,
`every_targeting_spell_or_ability_says_what_it_targets`, and the pre-existing
`every_declared_target_slot_is_answerable`). **Four rules, in order of reuse.**
(a) **The aim walker and the slot walker are a PAIR** — `primary_target_filter`
surfaces, `target_filter_for_slot` re-checks at CR 608.2b; a filter only one sees
aims right and re-checks against nothing, which is worse than none because it looks
fixed. Two invariants caught that twice in two commits. Add both arms.
(b) **An implicit filter belongs to the FIELD, not the card**
(`IMPLICIT_CREATURE_TARGET` / `IMPLICIT_ANY_TARGET` /
`implicit_player_if_bare_player_field`); a per-card filter is only for nouns narrower
than the field's own type.
(c) **Discover a class by joining a census against `scripts/.scryfall_cache.json`,
then gate it on a STRUCTURAL predicate** — all 38 blink bodies name `ControlledByYou`
/ `OwnedByYou` / `ExiledWithSource`, which is what made the test an invariant instead
of a list of 79 names that goes stale on the next card.
(d) **Group a census by the nearest enclosing enum key**: 204 card rows were ~40 arms.
(e) **An exception list is a walker you have not written yet.** The invariant shipped
with three of them — Officious Interrogation, Jeska's Will and Tithe name their target
player only from inside a `Value` or a `Predicate`, which the `Selector`-descending
walkers cannot reach — and they are `implicit_player_in_value` /
`implicit_player_in_predicate` / `implicit_player_in_payload` now (`eb13fa43`,
`cube` -0.036 %). Same argument as (c): a name goes stale on the next card, a walker
does not. **The invariant has no exceptions.**
**Checked, do not re-check:** the counterspells (target a spell `Target` cannot
express) and the ~25 reflexive "that creature" triggers (`combat.rs` stamps the slot
at push time); and there is **no `std::collections` default-hasher iteration in engine
or bot logic**, so cross-process determinism holds.

## CLOSED — what the seeded cube smoke test left behind (eighty-fifth pass)

Both defects are fixed; the section stays for the **method** — the 4,000-seed
sweep and `bot_rejection_count()` are how a targeting change is checked, and
the reverted `prefers_graveyard_target` widening is a trap worth not
re-entering.

`server::tests::bot_vs_bot_random_cube_decks_terminate` draws from
`crate::cube::build_cube_state_seeded(seed)`, which pins the decks, the
shuffle and `GameState::rng`; the test installs the bot's tie-break seed
inside the match thread, so a trial replays exactly. A **4,000-seed sweep runs
in ~870 s in a debug build**: set the loop to `0..4000u64`, add an
`eprintln!` of the seed, and run it `--no-capture`.

`server::bot_rejection_count()` — the live-match twin of `CRAB_SIM_REJECTS` —
counted **four** illegal bot actions across those 4,000 games before the
eighty-fifth pass and **zero** after it. **Re-run at the eighty-seventh pass
and still zero** (883 s under nextest), with that pass's three behaviour
changes in it: the picker's off-board gate, `EntityMatches`' empty-selector
answer, and the two block selectors' watcher fallback. Every pairing
terminated. Both bugs are fixed (the exile-target modal and
CR 508.1d vs the attack tax). What is open is the pair of defects underneath
them, neither of which the sweep can currently see because the thing that
would surface them is gated off.

### ~~The target enumerator is zone-blind, and one gate stands in for a zone~~ — closed at the hundred-and-fourth pass

`legal_targets_for_filter_inner` (`game/effects/targeting.rs`) walks the
battlefield, then every graveyard, then exile, applying the *same*
`SelectionRequirement` to all three. The filter language has no zone
predicate, so an exiled creature card satisfies "target creature" and a bare
`Not(Player)` satisfies everything anywhere. Callers separate the results with
`SelectionRequirement::mentions_offboard_zone()`, which is true only for
`InGraveyard` / `InYourGraveyard` / `InOpponentGraveyard` / `InExile`.

Two consequences, one fixed and one open:

* **Fixed** — a board-shaped filter's off-board matches used to be posed to
  the player as a `ChooseCards` modal whenever nothing on the board was
  legal. They are dropped now, and an empty legal set resolves targetless.
* **Closed at the hundred-and-fourth pass, and it took both halves the entry
  named.** (a) *A zone argument to the enumerator*: `legal_targets_for_filter`
  walked every graveyard and exile for any filter, so `Any` listed every card
  in every zone (Cuombajj Witches) and a board-shaped `Destroy` offered an
  exiled card. It takes the scope now
  (`legal_targets_for_filter_scoped`), and `enumerate_legal_targets_xc` passes
  the **same** `may_target_offboard_card || mentions_offboard_zone` question
  the auto-picker was gated on at the eighty-sixth pass — the UI path and the
  training path had been targeting different sets. (b) *A zone in the filter*:
  **seventy-nine bodies** said "from your graveyard" in their oracle text and
  nothing in their filter, so Zombify with an empty graveyard stole a
  battlefield creature. `SelectionRequirement::from_your_graveyard` /
  `from_any_graveyard` is the spelling, and
  `core_rules::target_walkers::every_reanimating_move_says_which_zone_its_
  target_is_in` is the invariant. Timeless Witness is one of the seventy-nine.

**Do not fix it by widening the gate to `Effect::prefers_graveyard_target`.**
That was tried at the eighty-fifth pass and reverted: it is true for exactly
this effect, and `Not(Player)` then matches every card in every graveyard
*and in exile*, so the modal offers illegal candidates and the bot answers
with none. Seed 62 is in the smoke test to keep that from being re-added
silently. **The filter is what fixed it** — `Not(Player).from_your_graveyard()`
is `mentions_offboard_zone`, so the modal path opens for it without any change
to that gate.

### ~~And the auto-picker has the same blindness with no gate at all~~ — fixed at the eighty-sixth pass

`auto_target_for_effect_avoiding_set_xc_inner`'s final fallback walked every
graveyard and then exile for *any* filter, so a "destroy target creature"
trigger with no legal battlefield creature auto-targeted an exiled card in the
**training** path (`wants_ui` false), where it silently fizzled instead of
resolving targetless — the same defect as the modal one above, on the side no
instrument watches.

It is gated now, on `Effect::may_target_offboard_card()` or the filter naming
the zone. **The gate is deliberately not `prefers_graveyard_target`**, which
decides walk *order* and has to stay narrow: making Condemn ("put target
attacking creature on the bottom of its owner's library") prefer a graveyard
would aim it at one. The new classifier is the superset — *any* zone change of
the target, because a `Move`'s destination is all the engine has to tell
Mortuary Mire's "return target creature card from your graveyard to the top of
your library" from Condemn — plus the modal and wrapper recursion its siblings
(`requires_target`, `primary_target_filter`, `accepts_player_target`) already
carried and it did not. Three tests in `core_rules::target_walkers::
offboard_gate`; the `--bench` invariant and the golden traces are unmoved, so
the defect does not fire on `--decks fixed`.

~~**What it does not fix is the enumerator itself.**~~ — stale, and closed by
the entry above at the hundred-and-fourth pass. The enumerator takes a scope
(`legal_targets_for_filter_scoped`, `targeting.rs:531`) and the filter
language has the zone predicate (`SelectionRequirement::from_your_graveyard` /
`from_any_graveyard`, `card.rs:2881`), so neither half of this paragraph is
true any more. Left in place struck through because it was the *plan* the
hundred-and-fourth pass executed.

### The gate's own wrappers — audited at the ninety-ninth pass (three cards fixed), the walkers' invariants closed at the hundredth

`prefers_graveyard_target` and `may_target_offboard_card` end in `_ => false`,
so a wrapper neither names closes the gate for its whole subtree.
`scripts/audit_target_walkers.py` prints the matrix: `requires_target` names
all 130 `Effect` wrappers because it is exhaustive, the other four name 26-61.
`core_rules::target_walkers::every_reachable_reanimation_is_visible_to_the_
offboard_gate` is the catalog half of it and is an invariant, not a ratchet.

Fixed: Reap's four graveyard slots were bare `Selector::Target(n)` and
surfaced no filter at all; Rise from the Wreck's four board-shaped filters
named no zone and `OptionalTargets` hid the `Move { to: Hand(You) }` from the
walk-order classifier, so it bounced a battlefield permanent; Ugin, Eye of the
Storms wrote "exile target permanent" as `Move { to: Exile }`, which that
classifier reads as reanimation, and is `Effect::Exile` now.

**The other three walkers — closed at the hundredth pass, two with an
invariant and one by construction.** All three are 0 findings on a
non-vacuous population, so they are invariants from the day they landed, not
ratchets. Each is the narrow shape its walker is actually about, which is
what the blanket version could not be.

* **`accepts_player_target` — its "101 unnamed wrappers" are NOT a defect
  census, and this is the correction the audit's uniform framing needs.**
  Alone in the family its fallthrough is **`_ => true`**, not `_ => false`:
  an unnamed wrapper is *permitted*, and the function's own comment calls
  that a conservative default because the legality gate still rejects a
  mismatch. So there is no silent-fizzle drift to close here, and a test
  claiming to close one would be vacuous. What *can* go wrong is the ~30
  arms that answer `false` on purpose (the `CounterSpell` family,
  `SupportCounters`, `DistributeCounters`, `Fight`), and
  `core_rules::target_walkers::every_reachable_target_player_is_visible_to_
  the_player_gate` holds those: **population 295, 0 findings** — no shipped
  card routes a target player through a refusing arm.
  The shape it looks for is `Selector::Player(PlayerRef::Target(_))`, the
  only player-target form that survives serialization unambiguously:
  `Selector::Target(n)` and `PlayerRef::Target(n)` are both a bare
  `{"Target": n}`, and `PlayerRef` sits in **65 distinct JSON positions**, so
  a walk keyed on those would go stale as variants are added.
  `{"Player": {"Target": n}}` needs no list.
  **Read a walker's fallthrough arm before reading its unnamed count as a
  bug list** — `scripts/audit_target_walkers.py` prints the same column for
  all five and only three of them restrict.
* **`primary_target_filter` (69 unnamed) — `..::the_primary_target_filter_
  agrees_with_the_slot_walker_on_slot_zero`, population 7,728, and it needs
  no tree walk at all.** `primary_target_filter()` and
  `target_filter_for_slot(0)` are two answers to the same question, and
  `auto_targets_for_effect` falls back to `Any` when the first is `None` — so
  a disagreement offers a target the card's own restriction forbids. **A
  walker checked against another walk of the same tree cannot false-report
  the way one checked against a guess at what the tree means can**; that is
  the general form of what made the blanket test useless, and it is the first
  thing to look for on the next walker.
* **`may_target_offboard_card` (104 unnamed) — closed by construction, no new
  test.** Its reachable population is already covered, and the missing shape
  does not exist: for a zone change the *destination* is the only signal that
  the source is off board, and `to: Hand(You)` / `Battlefield(You)` — the
  reanimation invariant's shape — is the whole of it. Every other `Move` with
  a target is a bounce or a removal aimed at the battlefield (which is why
  the blanket "holds a `Move`" version reported 29 bodies that were all right
  to answer `false`), and the graveyard-hate cases name the zone in the
  filter, where `mentions_offboard_zone` is the half that opens the gate.

**The structural fix shipped at the hundredth pass.**
`Effect::for_each_inner` is the one recursion, **130 of 130 wrappers**, held
there by `core_rules::target_walkers::the_shared_recursion_names_every_
effect_wrapper` reading `effect.rs` with the same extraction the audit script
uses. `prefers_graveyard_target` and `may_target_offboard_card` defer to it
instead of answering `false` for an unnamed wrapper's whole subtree, which
moved **67 and 61 shipped bodies** from a closed gate to an open one.
`Reflexive` / `ReflexiveTrigger` are named `=> false` explicitly (CR 603.7:
their targets are picked fresh at resolution).

**All three restricting walkers are switched.** `primary_target_filter`
joined them: its fallthrough takes the **first inner effect that has one**,
which is not a new rule — every explicit arm already followed it (`If` takes
`then` before `else_`, `FlipCoin` heads before tails, `RollDie` the first
arm) and `for_each_inner` yields in declaration order. **+32 catalog bodies**
now surface their real slot-0 filter instead of the `Any` fallback, and the
slot-agreement invariant (population 7,728) stayed green, which is the check
that the two walks still answer slot 0 the same way. `for_each_inner` took an
explicit lifetime for it: the reference that walker returns is borrowed from
the tree, and a higher-ranked `FnMut(&Effect)` will not let one escape.

**`accepts_player_target` must NOT be switched** — its fallthrough is
`_ => true`, so recursing would make it *more* restrictive with no drift to
fix.

**The audit script now reports the REGIME, not just the count.** Its "unnamed"
column stopped being a defect census the moment the three started deferring:
a wrapper they do not name is now *covered generically*, which is the point
of the fix. `scripts/audit_target_walkers.py` labels each walker
`exhaustive` / `deferred to for_each_inner` / `fallback true — permitted` /
`fallback RESTRICTS — these are gaps`, and only the last counts toward
`--check`. **An instrument that survives the fix it measured will misreport
it**; that is the general form, and it cost this pass a re-read to notice.

**A test whose job is to notice an absence has to be run against a
deliberately introduced one.** The completeness test passed with an arm
deleted on its first draft: it searched from `pub fn for_each_inner` to
end-of-file, and the other four walkers' mentions satisfied every lookup. It
brace-matches the function now. Do this to the next such test before
believing it.

**Unresolved, recorded rather than dropped: one non-reproducing failure of
`server::bot::stack_response_tests::mulligan_sim_prefers_the_functional_hand`.**
It failed once in a full-suite run at the hundredth pass and has not
reproduced in **nine** subsequent full runs (six with the walker change, three
without). What was ruled out, with numbers, so nobody re-derives it:

* **Not the tie-break jitter.** `mulligan_branch_value` seeds its own shuffle
  but leaves `bot::jitter_below` on the unseeded stream, which is the obvious
  suspect and is *not* it: the assertion holds for all **40** explicit jitter
  seeds tried, and 200 consecutive unseeded runs in one process produced
  **one distinct result pair** (`Some(18)`, `Some(0)`). The function is
  deterministic for this input.
* **Not a wall clock.** There is no `Instant::now` anywhere under
  `server/bot.rs` or `game/` — only `server/lobby.rs`, which this path does
  not touch.
* **Not cross-test state.** nextest is process-per-test here, so the
  thread-local jitter seed another test installs cannot leak.

Left standing: resource pressure during that particular run (the container
was at ~9 GB free and falling). If it recurs, capture the assertion message —
the test has two, and which one fired narrows this a lot.

### ~~Vacuous `true` in `Predicate::EntityMatches`~~ — closed at the eighty-seventh pass, and one layer dependency fell out

`EntityMatches` answered with `all` over the resolved selector, and `all` over
an **empty** selector is vacuously true — so a clause about *the* entity was
true when there was no entity. Closing the picker's off-board gate surfaced it
(Eagle of Deliverance drew a card off an indestructible counter it had put on
nothing), and it is `false` on the empty set now.

**It could not be closed in one step, because two of its own selectors were
resolving empty.** The eighty-sixth pass scoped the guard to an unbound
`Selector::Target(n)` and recorded the other two; the eighty-seventh fixed the
selectors and widened the guard:

* **`Selector::BlockedAttacker` reads `attackers_blocked_by(ctx.source)`, and
  for a third-party *watcher* `ctx.source` is the ability's host.** A trigger's
  event filter is built with the ability's card as `source` and the event's
  subject as `trigger_source`, so Righteous Indignation ("whenever a creature
  blocks a black or red creature") asked what the *enchantment* was blocking.
  Both block selectors fall back to `trigger_source` when `source`'s own
  answer is empty; `source` is tried first, so no self-trigger moves.
  Regression: `classic_sets::mmq4::righteous_indignation_ignores_a_green_
  attacker`.
* **`EntityMatches` over `EachPermanent(…)` is a plain existence test**, and
  the empty set now answers it correctly. Tide Shaper's "+1/+1 as long as an
  opponent controls an Island" was unconditional; it reads a printed opponent
  Island in both directions now (`mh::mh2e::tide_shaper_pump_reads_a_printed_
  opponent_island`), and its own Island does not count.

`EntityMatchesAny` is `any` and was always correct on the empty set. It is the
shape to prefer for a "some entity matches" clause.

### ~~A layer-7 condition cannot see a layer-4 type change (CR 613.8)~~ — FIXED at the eighty-ninth pass, by design (b)

**The two-phase gather shipped.** The three condition-gated statics
(`PumpSelfIf`, `SetBasePtIf`, `GrantPumpSelfIf`) are now the **last** thing
`gather_continuous_effects_inner` does, and while they evaluate their
predicates the effects gathered so far are installed in
`GameState::gather_partial` — a thread-local slot that
`computed_permanent`'s reentrancy branch reads instead of answering with the
printed view. The read *takes* the slot out for the duration of the layer
application, so a `computed_permanent` reached from inside it falls back to
printed: **exactly one ply, bounded by construction rather than by a depth
counter**, which is what a condition asking about another permanent's
characteristics needs. Two permanents each gating on the other's computed
shape is the cycle CR 613.8 resolves by dependency ordering and this does not
model.

`GameState::layer_reads_are_printed()` is the one place the two conditions
(`in_layer_gather` **and** no partial installed) are spelled out; the three
mid-gather printed fast paths — `effective_power`, `effective_toughness` and
the requirement walker's `computed()` cell — all ask it, and the last of
those is why the first attempt looked inert: it had its *own* `in_layer_gather`
fast path and never reached `computed_permanent` at all.

The ordering is asserted, not assumed: a `debug_assert_eq!` on
`all_effects.len()` at the end of the function fails if anything is ever
emitted after phase two. Three tests in `mh::mh2e` cover the two routes —
a resolved `continuous_effects` entry (`tide_shaper_kicked`, which now reads
power **2**) and a layer-4 grant the gather emits itself
(`..._made_by_a_layer_4_static`, Leyline of the Guildpact).

    callgrind, profiling-fast --no-default-features, --games 6 --seed 1
      fixed  +0.031 %   cube  +0.103 %   sealed  +0.031 %
    suite 19,064 / 0 / 5, golden traces unmoved, --bench byte-identical
    (no bench archetype carries an affected condition)

**AND THE OTHER HALF, WHICH THE MECHANISM CANNOT REACH: a predicate that
never asks the board.** The census above counts `MetalcraftActive` (4 uses
under the three gated statics) among the affected population, but it counted
`c.definition.is_artifact()` **directly** — so a Mycosynth Lattice still did
not turn Metalcraft on, mechanism or no mechanism. Measured on the shipped
fix before the change: Ardent Recruit beside three Forests under an
opponent's Lattice read power 1, not 3. It now counts the computed type line,
with the layer read second and behind `card_type_change_unscoped()` (the
memo-backed "can anything on this board change a card's types" gate, `false`
on almost every board) and a stop at three.
`cr_rules::cr_613_metalcraft_counts_computed_artifacts` covers it in both
directions. `FerociousActive` and `FormidableActive` next to it already read
`computed_permanent`; they are the pattern.

**~~Still open, one predicate and one use~~ — FIXED at the ninetieth pass, and
the section has no open entry left.**
`Predicate::ColorIsMostCommonAmongPermanents` tallied
`definition.printed_colors()` through `most_common_permanent_colors()`, so a
layer-5 colour change (Mycosynth Lattice's own `GrantColorless`, Painter's
Servant) was invisible to it. The gate it wanted —
`GameState::card_color_change_unscoped()` — is built, the layer-5 twin of
`card_type_change_unscoped()`: `AddColor` / `SetColors` / `LoseAllColors` on a
resolved effect, or a printed static folded into `card_can_change_colors`.
The tally reads the computed colours behind it and the printed ones otherwise.

**The gate is deliberately not memoized, unlike its type twin.** `type_bits`
earns its `CardMemo` slot because `card_type_change_unscoped` is on hot paths;
this one is reached from `most_common_permanent_colors` alone, which the whole
catalog touches from one predicate (the four-card Djinn cycle) and two
effects — on a board with none of them the function is never called and the
gate costs exactly zero. A new memo family widens the miss path for *every*
consumer of that word (the eighty-seventh pass measured that at `fixed`
+0.135 %) and there is no call rate here to pay for it.

`cr_rules::cr_613_most_common_color_counts_computed_colors` is the regression
test, in both directions and **verified to fail on the pre-fix tally** (Goham
Djinn reads power 3 alone, 5 under a Lattice that makes every permanent
colourless, 3 again when it leaves). Suite 19,073 / 0 / 5, clippy clean,
`--bench` byte-identical — no bench archetype carries a colour changer.

The entry as filed, kept for its census:

Fixing the above exposed it. `StaticEffect::PumpSelfIf`'s condition is
evaluated **inside `gather_continuous_effects_inner`**, where the
`in_layer_gather` reentrancy guard pins every characteristic read to the
*printed* one. So Tide Shaper's kicked ETB retypes an opponent's land to
Island (layer 4) and its own "as long as an opponent controls an Island"
condition (layer 7) cannot see it: `mh::mh2e::tide_shaper_kicked` asserts
power **1**, with the reason in the test.

CR 613.8's dependency rule would order the type change before the pump. The
engine models exactly one shape of this — `AffectedPermanents::
CardMatchPowerGated`, the second per-card pass that runs once the gate-free
power is known (Temur Ascendancy) — and a type-gated sibling would be the same
device. **Not a one-liner, and the guard it has to get past is the reentrancy
one**: the condition reads a computed characteristic of a *different*
permanent than the source, so it cannot simply drop `in_layer_gather`.

**SCOPED at the eighty-eighth pass, by counting the catalog rather than the
boards — and it is a *class*, not Tide Shaper.** Three statics evaluate a
`Predicate` inside the gather: `PumpSelfIf` (**194** catalog uses),
`SetBasePtIf` (5), `GrantPumpSelfIf` (2). Their conditions split into two
populations, and only one of them can be wrong:

```text
reads a characteristic a layer can change (types, subtypes, colours, P/T,
keywords) — AFFECTED, ~60 of the 194:
   34  SelectorCountAtLeast      "you control N Islands / artifacts"
   11  EntityMatches
   10  SelectorExists            "an opponent controls an Island" (Tide Shaper)
    4  MetalcraftActive          three artifacts — a computed card-type read
    1  ColorIsMostCommonAmongPermanents
reads a player- or zone-level fact no layer touches — CORRECT AS IS:
   17  ValueAtLeast     9 ThresholdActive   8 IsTurnOf   7 SpeedAtLeast
    5  HellbentActive   4 SourceIsMonstrous 4 SourceIsEquipped
    4  DescendActive    4 DeliriumActive    4 CelebrationActive
    …life totals, spells cast, crime, city's blessing, extra turn
```

**So a fix has to serve ~60 cards, and the two designs are these.** (a) The
`CardMatchPowerGated` sibling: a second per-card pass with the *computed*
answer, which works only when the condition reads the affected card and not
the board — `SelectorExists`/`SelectorCountAtLeast` read the board, so this
covers `EntityMatches` and little else. (b) A genuine two-phase gather: move
the three condition-gated blocks to the **end** of
`gather_continuous_effects_inner`, install `all_effects`-so-far as the frozen
set, and evaluate the predicates against it. (b) is CR 613.8's dependency
ordering for this case and covers all ~60 — **but do not build it against
source order**: `all_effects` is sorted by layer in `apply_layers`, not as it
is pushed, so "everything below layer 7 is already in the buffer" is only
true once the three blocks are genuinely last, and that has to be asserted,
not assumed. It also changes golden traces (legitimately) and sits in the
hottest function in the program, so it needs a `--decks cube` reading and the
`-C debug-assertions=yes` ladder gate, not just the suite.

### ~~"No panic reachable from bot self-play" had never been checked statically~~ — checked at the ninety-first pass, and the bare population taken 23 -> 4 at the hundred-and-first

The standing goal was audited only by *reaching* code: the 33,120-game
`-C debug-assertions=yes` grid proves what a game touches, and says nothing
about the site nobody touched. `scripts/audit_panics.py` is the static half —
the seventh filter — and its first reading is:

```text
109 panicking sites off the bin/test paths
     75 guarded      a proof (is_empty / is_some / len / match bind / filter)
                     in the site's own statement region
     11 lock-poison  Mutex/RwLock, reachable only after some other panic
     23 bare         no proof the filter's 22-line lookback can see
```

**All 23 bare sites were read, and every one is safe** — by a guard the
filter cannot see, which is the useful part of the result:

* **8x `source_owner.unwrap()` in `activate_ability_inner`** — sound, but by a
  *correlated flag*: `source_in_gy`/`_hand`/`_exile`/`_command` and
  `source_owner` come out of one tuple 57 lines up, so `Some` is implied by
  the flag the branch tested. Non-local, and the shape to watch: an enum
  (`SourceZone::Graveyard(owner)`) would make the binding structural, at the
  cost of churn in a 1.65 %-of-`fixed` function.
* **5x `remove_from_hand(..).unwrap()` in the cast paths** — every one is
  preceded by an `Err(CardNotInHand)` early return, up to ~270 lines above.
* **2x `try_pay_after_snapshot_mode`** — the `expect` message *is* the proof
  ("pool covered the cost a line ago").
* the rest are match arms on a length, `unreachable!` on a variant
  `perform_action` handles before dispatch, and deck-builder / recommender
  paths that are not game logic.

**One landmine was real and is gone**: `CounterBag`'s `Index<&CounterType>`
panicked on a kind the bag does not hold and had **no engine or server call
site at all** — its only two users were assertions in one test file, which now
ask `get(..).copied()`. The next caller to write
`c.counters[&CounterType::PlusOnePlusOne]` would have got a panic where `get`
returns `None`.

Re-run the filter after touching a hot path; the bare count is the number to
compare, not a pass/fail.

**AND THE BARE POPULATION IS NOW FOUR (2026-08-30), because "safe by a guard
the filter cannot see" is a claim that dates.** The ninety-first pass read all
23 and cleared them; every clearance was a *proof at a distance*, which is
exactly the thing a later edit to one arm breaks silently — and the engine's
caller is a training actor where a panic at game 400,000 costs hours. Nineteen
were converted to the error the site's own guard would have returned:

```text
112 sites / 23 bare   ->   84 sites / 4 bare
  14x source_owner.unwrap()      -> `src_owner!()`, a macro that returns the
     (activate_ability_inner)       same `CardNotOnBattlefield(card_id)` the
                                    construction site's own miss returns.
                                    The enum this entry proposed would have
                                    been the same guarantee at ~30 sites of
                                    churn in a hot function; the macro is 15
                                    lines and the compiler folds the branch.
   6x remove_from_hand(..).unwrap() -> `.ok_or(CardNotInHand(card_id))?`
   3x .expect("has_in_hand verified") -> the same
   2x pay_for_spell(..).expect(..)  -> `map_err(GameError::Mana)?`; the
     (try_pay_after_snapshot_mode)     second one restores the payment
                                       snapshot first, because the life cost
                                       and the colorless add already ran.
                                       **A proof on a *clone* is not a proof
                                       the audit can see** — that was the
                                       entry's "the expect message IS the
                                       proof", and the message was right and
                                       is not a mechanism.
   1x effs.pop().unwrap()          -> `unwrap_or(Effect::Noop)`
   1x candidates ... .expect("non-empty") -> `let Some(..) else { return }`
   1x GameAction::SubmitDecision(_) => unreachable!()
                                   -> `Err(NoDecisionPending)`
   1x draft.rs's secondary colour  -> a total `unwrap_or`
```

Ir-neutral on all three pools (see PERF's Log), suite green, grid green.

**The four that stay, and why they are not conversions.** One is
`perform_action_inner`-adjacent and three are deck construction —
`build_random_deck_from`'s second `build_shape`, `evaluate_candidates_slots`'
racing leader, `best_build_by`'s `n > 0`. Each would have to invent a
fallback deck, and **an actor that silently trains on a 0-card deck is worse
than one that crashes**: the crash is visible in the first minute, the poisoned
rows are not. They are contracts on a config value, not on game state, and
they stay loud on purpose. Do not "fix" them into `unwrap_or_default()`.

### ~~Two headless `OptionalTrigger` sites answered `no` where their own comments said `yes`~~ — fixed, and the third is load-bearing

The decision-plumbing audit's own docstring says a **bare** site is not
automatically a bug. This is the sub-population that is: a site whose
*comment already states the intended headless policy* while the code relies on
`AutoDecider`'s blanket `Decision::OptionalTrigger => Bool(false)`, which
contradicts it. That pair is greppable and it found three of the sixteen bare
`OptionalTrigger` sites. Two were bugs:

* `apply_etb_trigger_tax` — Strict Proctor. `catalog::strict_proctor`'s doc
  said "AutoDecider opts in to paying when the controller has enough mana
  floated"; it never paid, so a bot under a Proctor lost **every ETB trigger
  it controlled**, whatever it had floated. Now: the tax is pure generic, so
  headless pays when `mana_pool.total()` covers it.
  `stx::part_12::strict_proctor_headless_pays_the_tax_when_it_can_afford_it`.
* `Effect::LookTopEachPayLifeOrBin` — Moonlight Bargain. The comment said "the
  auto decider says yes, so bots keep the cards they can pay for"; it said no,
  so a bot spent five mana to bin all five cards. Now: affordability is the
  whole decision headless.
  `classic_sets::rav::moonlight_bargain_headless_buys_every_card_it_can_afford`.

**And the third is why the audit is a triage list and not a gate.**
`Effect::MayCopyThisSpell` (the CR 706 Chain cycle) reads the same way and the
blanket `no` is **what makes the chain terminate**: a copy carries its own
`MayCopyThisSpell`, nothing in the loop shrinks a resource that bounds it, and
a `ChainCopyCost::Free` link stays payable even when the copy finds no legal
target. Built, and it spun `ons::chain_of_acid_offers_the_copy_onward` at
100 % CPU until killed. Reverted with the reason written at the site, so the
next sweep does not re-take it.

**The remaining thirteen were read and are deliberate**: repeat loops
(`MayRepeat`, Kindle the Carnage, Trade Secrets) where `no` bounds the loop the
same way; guesses with no basis (`Is a card named X in their hand?`); and four
whose comments already document `no` as the chosen policy (Tainted Pact takes
the card on `false`, Wandering Archaic lets the copy happen).

### ~~The bot answers a mandatory off-board modal with nothing~~ — fixed

`bot::decide_choose_cards`'s five exits each filled `min` from the pile that
branch understands (the hand, the board, the bot's own graveyard) and none
covered a candidate in exile or in a graveyard the owner lookup did not
resolve, so the answer came back empty — and `min: 1` rejects an empty
answer, which ends the match where it stands. Every exit now goes through a
`fill_to_min` that tops up from the candidate list itself, so a well-formed
answer is always produced when one exists. Behaviour-identical wherever the
old answer was already legal (it only adds while `len < min`, and `min <=
max`); suite 19,056 / 0 / 5 and the seeded sweep unchanged.

## Engine correctness audit — 2026-06-11

Five-reviewer deep pass over the engine core (`game/mod.rs`, `effects/`,
`actions.rs`/`affordances.rs`, `stack.rs`/`combat.rs`/`layers.rs`/`types.rs`,
`crabomination_base`). Every finding was verified against call sites; known
approximations already logged elsewhere in this file were excluded. Line
numbers are as of commit `683d1416` — re-grep before fixing.

Two recurring failure modes generated most of these (see the P3 root-cause
items): effect arms **bypassing the rich centralized funnels** (death /
discard / zone-move / damage) for a bare cheaper helper, and **parallel
hand-maintained walkers drifting apart** with no exhaustiveness guard.

### P0–P1 — resolved (2026-06-11 audit)

All P0 (game-deciding / state-corrupting) and P1 (rules-visible) findings from
the five-reviewer pass are fixed and regression-tested. Per-finding detail (call
sites, CR clauses, test names) was elided in a compaction pass — recover it from
`git log -p -- TODO.md`. Classes closed: blocked-attacker-stays-blocked (510.1c),
trigger fizzle vs re-target (608.2b), cast-pipeline atomicity (`cast_atomically`),
pump-duration respect, the death-funnel-bypass family, life/draw/damage
replacement coverage, real coin-flip RNG, non-combat wither/infect/deathtouch,
per-source combat-damage aggregation, layer timestamps, and the hybrid-mana
solver. The two recurring root causes (effect arms bypassing the rich funnels;
parallel hand-maintained walkers drifting) are tracked in P3 below.

### P2 — open

- ✅ **Deck-out loss is applied too eagerly (CR 104.3c / 704.5c)** — FIXED.
  `PlayerData::pending_deck_loss` is armed by the failed draw and promoted by
  `check_state_based_actions`, behind the same `player_cant_lose_game` /
  `apply_loss_reset` guards as the other loss SBAs. That also closes the
  second half nobody had noticed: `objects_leave_with_player` runs only for
  the seats the SBA sweep itself eliminated, so a decked player's whole board
  used to stay on the battlefield for the rest of the game (CR 800.4a).
  The ~24 tests this entry warned about were **19**, all of them decking
  themselves by accident because `two_player_game()` seats empty libraries;
  `game::stock_libraries(&mut g, n)` is the shared harness, one call each.
  One of the nineteen (`kenriths_transformation_draws_and_makes_a_3_3_green_elk`)
  was passing vacuously — its ETB draw had never fired. Regression tests:
  `core_rules::cr_recent16::cr_104_3c_decked_opponent_still_seen_by_the_same_resolution`
  and `::cr_800_4a_decked_players_permanents_leave_with_them`.

- ✅ **Sand Golem's discard trigger — FIXED at `39528f0f`, and it was a whole
  family.** `EventScope::SelfSource` on a card in a graveyard is decided by two
  hand-written walkers that had drifted: the dispatcher's graveyard walk
  admitted `CardCycled` / `CardMilled` / `CardDiscarded` / `PutIntoGraveyard`,
  and `event_matches_spec`'s SelfSource chain matched only three of those four
  by id. `CardDiscarded` was admitted and never matched;
  `OpponentCausedYouToDiscard` was in neither. Dead as a result: Sand Golem,
  Mangara's Blessing, and Pure Intentions' own return-to-hand trigger. Both
  walkers now read one predicate, `is_graveyard_self_source_kind`.

  The earlier probe that "swapping Sand Golem onto Pure Intentions' idiom did
  not make it fire either" was right and pointed at the third defect: that
  idiom was `CardDiscarded` + `Predicate::CausedByOpponentSpellOrAbility`, and
  the predicate reads `resolution_causer`, which is cleared before the
  discard's triggers dispatch. Pure Intentions moves to the event that stamps
  the causer at discard time.

  **A fourth defect, one level up in the dispatcher (`6a0a79ca`):**
  `OpponentCausedYouToDiscard` was missing from the CR 603.6 fan-out list, so
  a batch carrying two of the event minted one trigger — Spiritual Focus paid
  2 life for a Mind Rot that takes two cards. Its twin `CardDiscarded` was
  already in that list; the same two-lists-must-agree shape, one level higher.

  Ratchets: `events.rs`' unit test walks the graveyard family and asserts the
  two walkers agree; `catalog_registration::every_self_source_trigger_kind_
  reaches_a_dispatcher` asks the population question — no card in the catalog
  carries a `SelfSource` trigger on a kind nothing admits, with the fifteen
  kinds dispatched by a *push* site listed with their sites.

- ✅ **A stripped permanent's *printed* mana ability still activated
  (CR 305.7 / 613.1f) — FIXED at `(-206)`.** `activate_ability_inner`'s
  printed-index gate was `stripped && !is_mana_ability(..)`, with the
  comment "no catalog card stripping abilities has a mana ability of
  interest right now" — Blood Moon on a Temple of Epiphany and Turn to Frog
  on a Llanowar Elves are both in the catalog. The auto-tapper's source
  table already dropped the printed list on `lost_all_abilities` (and keeps
  granted mana abilities), so the bot never exploited it; the direct
  `GameAction::ActivateAbility` path (UI seat, scripted tests) did. Found
  by the `(-204)` audit (`core_rules::land_tap_fast_path`'s Blood Moon
  board, on which both paths agreed and both were wrong). The printed leg
  now refuses on `stripped` outright; the granted and intrinsic legs are
  unchanged, matching the tapper. Regression tests:
  `modern::decks_16_17_misc::blood_moon_refuses_a_nonbasics_printed_mana_
  ability_on_activation` and `::turn_to_frog_takes_a_mana_dorks_mana_
  ability`. The land-tap fast path gained `ability_strip_possible` (the
  strip presence read behind the dispatch lane); PERF prices it.

**P2 has no other open correctness entries.**

### P2 — performance

- 🟡 **Uncached layer recomputation is the dominant engine cost.**
  Largely addressed via `GameState::with_frozen_layers` — a scoped,
  lazily-filled memo of the gathered continuous-effect set (sound by
  construction: the closure only holds `&GameState`; clones reset to
  unfrozen, so bot dry-runs stay correct). Frozen scopes now cover
  `resolve_selector` (every `EachPermanent`/`ControlledBy` filter),
  `legal_attackers`/`legal_blockers`, the bot's `pick_blocks`, the full
  client-view projection (`project_for`), and
  `damage_prevented_by_protection`. Test
  `frozen_layers_match_unfrozen_computation`. (A global generation-counter
  dirty-flag cache was rejected: `GameState` fields are mutated directly
  throughout tests/server, so invalidation can't be guaranteed.)
  Remaining: within a frozen scope `compute_battlefield` still re-applies
  layers per call (`apply_layers` over all permanents per blocker in
  `legal_blockers`); hoist `&[ComputedPermanent]` snapshots there if
  profiles still show it.
- 🟡 **Affordance probing clones the world per candidate**
  (`affordances.rs`). `compute_hand_affordances` now builds **one**
  library-stripped template per sweep and threads it through every
  category's `_on` variant; keyword-gated categories (buyback / dash /
  blitz / …) pre-filter to matching hand cards before any dry-run.
  Remaining: each candidate still pays one `template.clone()` +
  `perform_action` dry-run — a non-mutating `validate_action` path would
  eliminate the per-candidate clone entirely (large refactor; only worth
  it if profiles show view projection hot).

### P3 — structural root causes (fix once, prevent the class)

- ✅ **CLOSED — `primary_target_filter` defers to `target_filter_for_slot(0)`
  (`5ae08799`), the classifier one level up follows it, and both are pinned by
  tests.** The census below is what made the fix a one-liner rather than two
  card patches: the two bugs were the same missing shape (a slot-0
  `Selector::Player(Target(n))`), and every other disagreement was the picker
  answering about a slot the checker was not asked about. Deferring makes the
  two agree by construction wherever the checker speaks, and hands the 253
  definitions with a slot-0 filter and no arm in the picker a filter where
  they had `None`. The fallback walk stays for the 466 mass effects whose
  "subject" filter is not a target at all.

  **The deferral alone left `Reins of Power` with an empty legal-target list,
  and nothing in the tree noticed.** `accepts_player_target`'s `Seq` / `If`
  arms pick the child that classifies the spell by "first one with a
  `primary_target_filter`" — and that walker answers about non-target
  *subject* selectors too. Reins of Power is `Seq([Untap(each creature),
  GainControl(creatures target player controls), …])`: the `Untap` names a
  group, owns no slot, and was deciding that the spell targets permanents.
  While the picker was *also* wrong the two errors cancelled and the list came
  back full of creatures; once the picker answered with slot 0's player
  filter, `legal_targets_for_filter` was asked for permanents matching a
  player filter and returned nothing. Both arms look for the child that owns
  **slot 0** first, then fall back as before. Cling to Dust's ordering rule
  (the reason those arms exist) is unchanged: its `Move` owns slot 0.

  **Two guards, and the second is what found the half above.**
  `primary_target_filter_defers_to_the_608_2b_checker` is the equality over
  `all_known_factories()` — **83 definitions** fail it without the deferral
  (65 spell bodies plus 18 ability bodies the census did not walk), 0 with it.
  `feedback_bolt_and_reins_of_power_offer_players_not_permanents` pins both
  cards at `enumerate_legal_targets`, the site the deferral actually moved
  (the cast path reads the slot walker directly and was never wrong) — the
  structural invariant could not have caught the classifier, because by then
  both walkers agreed. Neither is the blanket ratchet the sixty-fifth pass
  deleted: that one compared the walkers everywhere and needed 587 -> 83 -> 27
  exceptions, and these assert what the code now establishes.

  The census, kept because it is the reason the fix is one line:
  `primary_target_filter` (what the auto-picker aims with) and
  `target_filter_for_slot(0)` (what CR 608.2b checks against) are
  hand-written and independent. The measured breakdown over
  `all_known_factories()` (both walkers `Some` and unequal), which supersedes
  the "27 single-slot bodies" this entry used to claim:

  | | count | verdict |
  |---|---|---|
  | both `Some`, agree | 3,421 | — |
  | both `Some`, **disagree** | **65** | below |
  | of those, effect also has a slot 1 | 47 | **not a bug** — the two walkers are describing different slots (the fight family: `Prey Upon`, `Rabid Bite`, `Pit Fight`, … all read pick=`Creature+ControlledByOpponent` / check=`Creature+ControlledByYou`) |
  | single-slot and modal | 10 | **not a bug** — slot 0 differs per mode; `target_filter_for_slot_in_mode` resolves it (`Jund Charm`, the Charm cycle, `Flame of Anor`) |
  | single-slot and kicker-branched | 4 | **not a bug** — `Bloodchief's Thirst`, `Overload`, `Prohibit`, `Tear Asunder`; `…_in_mode_kicked` resolves it |
  | **single-slot, non-modal, non-kicker** | **2** | **were bugs, one root cause — FIXED at the seventy-fifth pass** |
  | `primary_target_filter` `Some` / slot-0 `None` | 466 | mass effects; the primary walker is reporting a *subject* filter, not a target |
  | slot-0 `Some` / primary `None` | 253 | already covered by the picker's fallback |

  **The two bugs share a root cause and it is not per-card.**
  `Feedback Bolt` (pick `Artifact+ControlledByYou`, check `Player`) and
  `Reins of Power` (pick `Creature`, check `Player`) both target a **player**
  in slot 0, and `primary_target_filter`'s `sel_filter` has no arm for
  `Selector::Player(Target(n))` / `ControlledBy { who: Target(n) }` — so it
  falls through to a non-target `EachPermanent` subject filter (`Feedback
  Bolt`'s artifact count, `Reins of Power`'s `Untap` clause, which is
  `Seq`'s *first* element and wins the `find_map`). **The cast path is
  unaffected** — `auto_targets_for_effect_all_slots` sees `slot0_has_filter`
  and never reaches the heuristic picker — so the blast radius is
  `enumerate_legal_targets_xc` (the client's legal-target list),
  `view.rs`'s `target_noun`, and the two `bot.rs` fallback pickers.
  **FIXED at the seventy-fifth pass** (`5ae08799`): `primary_target_filter`
  defers to `target_filter_for_slot(0)` when that answers, and keeps its own
  walk for the 466 mass effects with no target at all. `--decks fixed` and
  `--decks cube` play byte-identical games (so +0.008 % / +0.030 % is what
  the extra walk costs); `--decks sos` diverges by 128 decisions and reads
  **-2.037 %**, i.e. **-2.80 % per decision** — the bot stops enumerating and
  probing targets the CR 608.2b check would have rejected. Completed casts
  flat. **There is no strength gate available for a change of this shape**:
  `bot_ladder` compares two *profiles* inside one binary, not two binaries,
  so the justification is the argument (the aim now uses the filter the check
  uses), the census, and 7 byte-identical golden traces.
  **The general invariant holds with no exceptions now** — the seventy-sixth
  pass's guard test compares the two walkers over the whole catalog and finds
  0, where the sixty-fifth pass's ratchet over the same comparison needed
  587 -> 83 -> 27, because the deferral makes them equal by construction
  rather than by exception list.
  **The silent-fallback half is already fixed** — the picker
  falls back to the checker's own filter before `Any`, so the two agree by
  construction wherever the primary walker is silent (that fix is what made
  creature Haunt work at all; see `core_rules::unbound_target_slots`).

- 🟡 **The combat planners and the combat legality checks are two readings of
  the same rules, and five of them disagreed.** Found at the seventy-sixth
  pass with `CRAB_SIM_REJECTS` (PERF's (-55)): the bot's declarations are the
  only actions in the simulator that go through `perform_action`'s checkpoint,
  so a rejected declaration leaves exactly one trace — a rollback — and until
  that instrument landed nothing read it.

```text
  --games 20 --threads 1      before                            after
  cube seed 7      82/9,664  (0.85 %) atk 18  blk 64    0/9,862    (0.00 %)
  cube seed 11    434/13,034 (3.33 %) atk 324 blk 110   372/13,428 (2.77 %) atk 324 blk 48
  all  seed 3      64/33,608 (0.19 %) atk 0   blk 64    0/33,714   (0.00 %)
  sos  seed 5       0/6,892                             0/6,892
```

  **Every block rejection on `cube` seed 7 and `all` seed 3 is gone; seed 11's
  blocks go 110 -> 48 and its 324 attack rejections are untouched**, because
  they have a different cause — the open list at the end of this entry.

  **The engine rejects the *batch*, not the pair**, which is what makes this
  expensive: one illegal gang member cost the defender every block it had
  planned, and in the simulator the candidate was then scored against a board
  where nothing blocked at all.

  | # | site | what it read | what `declare_*` reads |
  |---|---|---|---|
  | 1 | `blocker_can_block_attacker_pair` | no landwalk gate at all | CR 702.15 / 702.14c / 702.14 / 702.43, plus `CantBeBlockedIfControllerCastSpells`, `…UnlessDefenderSharedType`, `…ByPowerLessThanCount` |
  | 2 | `pick_blocks_inner`'s gang pass | `bot_can_block` + flying/reach only | the whole pair gate |
  | 3 | `min_blockers_required` | printed keywords + `granted_keywords_eot` | the **computed** set, so a granted Menace under-filled the block |
  | 4 | `pick_attacks`' `raw_attackers` | printed `Haste` | the computed set, so a granted-haste must-attacker was left home (CR 508.1d) |
  | 5 | `pick_attacks` | `CantAttackAlone` only | also `AttacksAlone` (CR 508.0), which bars a *multi*-attacker batch |

  **The shape all five share: a legality question answered off the printed or
  instance view when the engine answers it off the computed one, or a
  candidate pass that skips the shared gate.** Four regression tests in
  `server::bot::tests` pin them, and each ends by handing the plan to
  `declare_attackers` / `declare_blockers` — the engine is the oracle, so the
  test cannot drift away from the rule it is checking. Three of the four fail
  without their fix; the landwalk one needed the defender put under life
  pressure before the planner would try the block at all, which is the note
  worth keeping: **a planner test that does not make the planner *want* the
  illegal move proves nothing.**

  **What `CRAB_SIM_REJECTS=names` still names on `cube` seed 11**, and these
  are the next leads rather than anything this pass fixed. **~~Leads~~ —
  every row here is a seventy-sixth-pass reading and the counter has read
  zero in every configuration run since the eighty-first; the table is kept
  for its *shapes*, not as a work list. Its "`declare_attackers_banded`'s
  thirty `CannotAttack` returns need a per-site tag — build that first" is
  also done**: `attack_reject(line!(), …)` tags sixteen sites and
  `block_reject` the block side, both printed by `CRAB_SIM_REJECTS=names`.

  | count | error | card | the tell |
  |---|---|---|---|
  | 154 | `CannotAttack` | `Angel`, `computed_kw=[Flying]` | no restriction keyword at all, so the batch is illegal for a *batch* reason attributed to one card — goading, or an external "attacks each combat if able" the planner never sees. `declare_attackers_banded`'s thirty `CannotAttack` returns need a per-site tag before this can be bisected; **build that first** |
  | 56 | `SummoningSickness` | `Kestia, the Cultivator`, `sick=Some(false)` | a contradiction on its face — the engine says summoning-sick about a card whose flag is clear. Bestow, or a controller change this turn |
  | 42 | `MustBeBlockedIfAble` | `Crested Craghorn` | CR 509.1c: the planner's must-be-blocked top-up found no legal blocker where the engine says one exists. `bot_can_block` requires *untapped*; `declare_blockers` allows a tapped blocker under `tapped_creatures_can_block` |
  | 24 | `CannotAttack` | `Nimble Mongoose`, `computed_kw=[Shroud]` | the `Angel` row's shape again |
  | 6 | `CannotBlock` | `Arclight Phoenix` | the residue of the pair gate below |

  **~~Still open~~ — STALE, and the ~310-line refactor it asks for is already
  done. Re-read at the eighty-ninth pass.** The paragraph below described the
  pre-eighty-first-pass state and survived the fix that closed it; it is kept
  because its *cost* argument is still the reason not to widen the shared body
  further, and struck because its premise is not true any more. **The pair
  gate is one body and every reading routes through it**, checked by grep
  rather than by reading:

  * `declare_blockers` calls `blocker_self_block` and then
    `blocker_pair_block` per assignment, and has no other *per-pair*
    rejection — every other `block_reject` in it is batch-level (below).
  * `blocker_can_block_attacker_pair` is `blocker_pair_block(..).is_none()`
    and `blocker_can_block_anything` is `blocker_self_block(..).is_none()` —
    two one-line wrappers in `game/mod.rs`, and
    `blocker_can_block_attacker` is their composition.
  * the planner asks only those three: **eleven** call sites in
    `server/bot.rs` and nothing else, `bot_can_block` among them.
    `grep -n 'blocker_can_block' crabomination/src/server/bot.rs` is the
    check, and it is the whole check.

  **What is genuinely still two readings is the *batch* level, and it cannot
  live in a pair function**: CR 509.1c "can't block alone", Okk's
  bigger-partner rule, the Silent Arbiter cap, the menace count and the
  must-be-blocked requirements are all statements about the whole
  declaration. Those have their own unification (`block_requirement_able`,
  `block_requirement_binds`, `min_blockers_required_kws`) two paragraphs
  down, and `CRAB_SIM_REJECTS` is what watches the join — **0 of 126,608
  simulated declarations at the eighty-ninth tip**. The entry stays 🟡 for
  *that* and for the missing printed-vs-computed guards, not for the pair
  gate.

  The paragraph as it stood, for its cost argument: the two readings are still
  two hand-written lists. `declare_blockers` has ~20 per-pair gates and
  `blocker_can_block_attacker_pair` now has ~12 of them; the rest are
  blocker-side gates reached through `blocker_can_block_anything`, and nothing
  proves the union is complete. **The class fix is to extract
  `declare_blockers`' per-pair body into one function both call**, which is a
  ~310-line mechanical move plus a cost problem: the bot asks the gate 25,694
  times a `cube` run against the engine's handful, and the engine's body is
  ~20 keyword scans. It wants a per-card "carries any block-restriction
  keyword" bit (the `AttackerFacts` / blocker-facts structs already exist to
  hold one) so the shared body runs only for the pairs that can fail.
  **`CRAB_SIM_REJECTS` is the guard**: `CRAB_SIM_REJECTS=1
  bot_ladder --a gang --b gang --games 20 --threads 1 --decks all --seed 3`
  reads 0; `=names` names anything that is not. **Run it over `cube` seeds
  1-24, not three of them** — the three-seed census read the block half as
  closed while eight other seeds carried 186 rejections between them across
  four unmodelled rules (PERF (-55)). ~90 s at `--games 8`.

  **The CR 509.1 requirement family is now one predicate, and the count rule
  gates it.** `block_requirement_able` is the single "able" the four
  requirement loops ask (Provoke's `must_block`, `MustBeBlocked`,
  `AllMustBlock`, and the blocker-side `MustBlock`/`MustAttackOrBlock`); three
  of the four had drifted from it on the tapped term before the unification.
  `block_requirement_binds` is CR 509.1b outranking CR 509.1c: a requirement
  no *legal* declaration can satisfy does not bind. Without it the engine
  demanded a declaration it also forbade — a Lure+Menace attacker facing one
  able blocker had **no legal block at all**, and that board is reachable in
  `cube` seed 15. Pinned by
  `server::bot::tests::a_count_restriction_unbinds_a_block_requirement`.

  **The general shape is worth stating, because this is the second family it
  has bitten:** wherever a "must" and a "can't" are two independent checks,
  the pair can be unsatisfiable, and only the census finds it — the site tag
  named CR 509.1b on a board whose actual defect was CR 509.1c's binding rule.
  The remaining candidates to audit for it are CR 509.1b's
  `CantBeBlockedUnlessAllBlock` (Tromokratis) against `CantBlock` grants, and
  CR 508.1d's must-attack against the CR 613 hand-size power cap.

  **And the same trap one level up, which was the last six rejections: two
  *requirements* naming one creature.** It blocks one attacker, so they can
  never both be satisfied, and each loop asked about its own in isolation — a
  Lure attacker, a provoker and one able defender had no legal declaration
  either. CR 509.1c's "the **maximum number** of requirements" is the rule;
  `block_spoken_for_elsewhere` excuses a blocker already assigned to an
  attacker whose own requirement binds it, so both single-block plans are
  legal and "block with nobody" is not. Pinned by
  `server::bot::tests::two_block_requirements_on_one_creature_are_both_satisfiable`.
  Full maximization over arbitrary requirement sets is still the documented
  approximation; what is handled is the case that a creature can only be in
  one place.

  **`CRAB_SIM_REJECTS` now reads 0 in all 69 configurations run** — `cube`
  1-24+42 at `--games 20`, `cube` 25-45 at `--games 12`, and the four other
  pools at seeds 1-12 — from 470/91,438 when the instrument landed. Method
  note: **the site tag names the clause that rejected a declaration and never
  the pass that built it**, and two plausible fixes to the wrong pass measured
  exactly inert before a throwaway probe printing the plan at each pass
  boundary found the cause in one run. Build that probe first.

  Two known non-bugs the counter also surfaces, both in
  `attack_candidates_for_mcts` and both deliberate: the "all home" candidate
  and any "greedy minus one" that drops a must-attack creature are illegal by
  CR 508.1d, so `simulate_attack_outcome` scores them not at all (its doc says
  so). They cost a sim start each — 62 in a twenty-game `cube` run — and the
  candidate generator could skip them instead.

- 🟡 **Parallel hand-maintained walkers** (combat pair closed) — guard test
  `cr_601_2c_every_catalog_target_filter_is_surfaced` now serde-walks every
  catalog effect for `TargetFiltered` slots and asserts
  `target_filter_for_slot_in_mode_kicked` surfaces each one (caught + fixed
  `DiscardChosen` / `ManaClash` holes; ChooseN gets a cast-time fallback
  filter). `evaluate_requirement_static` no longer `unreachable!`s on
  zone-agnostic atoms (HasSpellSubtype/HasEnchantmentSubtype/…) — it delegates
  to `evaluate_requirement_on_card` against the located card.

  **The combat pair is CLOSED at the eighty-first pass, both sides, and each
  one was hiding a state with no legal declaration at all.** Attack:
  `attacker_self_block` / `attacker_target_block`, with `attacker_is_able` and
  `may_declare_attacker` as compositions — a must-attack creature under any of
  the twenty-two restrictions the four-gate `able` never read was *required to
  attack and then rejected for attacking*. Block: `blocker_self_block` /
  `blocker_pair_block`, with `block_requirement_able`,
  `blocker_can_block_anything`/`_pair` and the planner's `bot_can_block` all
  compositions of those two — a provoked creature that is detained, or that
  its provoker islandwalks past, could neither block nor be left home. **And
  the block drift ran the other way too:** seven `CantAttackOrBlock*` families
  (hand size, delirium, a creature died this turn, Descend N, the city's
  blessing, cards in exile, Hollow Warrior's helper) plus Space Beleren's
  sector lock lived only in the *mirror*, so `declare_blockers` never enforced
  them and those cards' blocking restrictions did nothing on the real play
  path. Eleven tests in `cr_recent100`. Both walkers return `(site, error)`,
  so `CRAB_SIM_REJECTS=names` still names the rule rather than the card.

  **The last pair now has its guard, and the guard found drift on its first
  run.** `audit_p3_requirement_walkers_agree_on_an_unlayered_permanent`
  (`core_rules/cr_rules.rs`) collects every `SelectionRequirement` the catalog
  actually uses — 882 of them, off the effects' own serde trees, so a new
  variant reaches the test the moment a card uses it — and asserts the two
  walkers agree **on a battlefield permanent with no continuous effect in
  play**, where computed equals printed. Off the battlefield or under layers
  they are supposed to differ; that is what `_on_card` is for.

  **Fourteen disagreements, from four root variants — and they are all
  deliberate.** `Tapped`, `Untapped`, `HasGreatestPowerAmongAllCreatures`
  and `HasGreatestManaValueAmongControlled` (plus the `And`/`Or` compositions
  the catalog builds from them) have explicit `false` arms in
  `evaluate_requirement_on_card` that say why: *"Battlefield-state predicates
  can't be evaluated for library cards."* It is the library/hand-search path.
  So the invariant the test enforces is not "the two agree" — it is **"they
  differ only where a documented arm says they may"**, and the allowlist is
  that documentation, machine-checked and self-guarding: each entry is
  asserted to *still* differ, so a change cannot silently close one.

  **The real defect the list exposed was one level up, and it is fixed.**
  `ManaValueAtMostYourCount`, `ToughnessAtMostYourCount` and
  `PowerAtMostYourCount` (both walkers' copies, six sites) walk
  `self.battlefield` and filtered it through the *zone-blind* walker, so a
  counting requirement whose inner filter is `Tapped` counted **zero** tapped
  permanents on a board full of them. They now use
  `evaluate_requirement_static_on`, which takes the instance and costs no
  lookup. Pinned by
  `a_counting_requirement_counts_tapped_permanents`, verified by putting the
  bug back. **`--bench` is byte-identical on all five pools**, so no bench
  deck reaches it — which is why it needed a test and not a ladder run.

  **Method note: the first reading of the list was "these variants have no
  arm", and the compiler refuted it** — the fix drew `unreachable pattern` on
  arms that were already there. The allowlist was evidence of a defect, just
  not the one it looked like.

  **The printed-vs-computed combat checks — HALF CLOSED at the eighty-ninth
  pass, and the half that was open was live.** The eighty-first pass unified
  the *pair gate*; the planner's **pre-filters in front of it** were still
  four hand-written copies reading the printed keyword list. A granted Flying
  was pre-filtered as blockable and then rejected by the authoritative gate,
  so `CRAB_SIM_REJECTS` never saw it — but a granted **Reach** went the other
  way and dropped the pair before the gate ran, so a legal, wanted block was
  invisible in every plan the bot made. That is the direction no rejection
  counter can report, and it is why "the counter reads zero" is not a proof.

  `legal_blockers` now returns the computed view it had to build anyway,
  `evasion_bars_block` is the one pre-filter, the two passes that call
  `blocker_can_block_attacker` immediately after lost theirs outright, and
  `bot_block_plan_sees_a_granted_reach` / `..._honours_a_granted_flying` pin
  both directions. It read `fixed` -0.276 % / `cube` -0.229 % as well: the
  computed view was being resolved twice.

  **The attack side was read at the same pass and it is NOT the same
  finding.** Every *legality* decision there is already the engine's:
  `raw_attackers` filters on `may_declare_attacker` with the computed view,
  the participation cap and both CR 508.0 alone-rules read
  `computed_permanent(..).keywords`, and CR 508.1d goes through
  `restore_forced_attackers`. What still reads `has_keyword` is the
  **hold-back heuristic** — deathtouch/menace/first-strike/flying parity
  against the opponent's bodies — and a printed read there is a wrong
  *estimate*, not an illegal or invisible declaration.
  
  It is not nothing: a creature the greedy pass holds back is in no candidate
  the search can recover, so a granted-Flying attacker can be held home
  against ground blockers that could never block it. But that is a **strength
  change**, and `bot_ladder` compares two profiles inside one binary rather
  than two binaries — there is no gate for it (the seventy-fifth pass's
  `primary_target_filter` note has the same problem and the same conclusion).
  Take it with a strength harness, not as a bug fix. The one documented
  legality approximation that remains is `raw_attackers`' printed
  `is_creature`: a permanent *animated* into a creature is never considered,
  and reading the layer view there measured `fixed` **+0.62 %**.

## Engine — Robustness / defects: the closed audits and the twenty-three filters

*Moved verbatim from `TODO.md` at the fifty-fourth pass, when that file passed
the ~1k-line trigger. **No open entries** — this is the record of what each
sweep hunted and why it is closed, kept so nobody re-derives one.*

### Robustness filters (the determinism entry itself closed 2026-08-11, `841dd40b`)

**Re-checked at the hundred-and-sixth pass, and moved here verbatim from
`TODO.md`'s NEXT when that section passed its ~15-line budget:** no
`std::collections` default-hasher iteration in engine or bot logic — nine uses
exist, the only two on a game path are membership-only (`bot.rs`'s belief
redeal) and lookup-only (`wants_converge`'s L2 cache). Cross-process
determinism holds; `golden_trace::seeded_games_match_their_digests` is the
check, since a test process is a new process.

**And the encoder leg of the `-C debug-assertions=yes` grid, which had never
been run:** `bot_ladder` encodes no state on any pool, so the 30-cell grid
cannot reach an assertion that only the encoder trips. The actor leg is
`target-audit/overflow/selfplay_train --actors 3 --steps 2 --games N --seed S`
after the same `RUSTFLAGS` build, and 6,000 games / 577,283 rows came back
clean at the hundred-and-sixth pass (PERF's Baseline has the numbers).

The cube pool's fixed-seed nondeterminism is fixed and the whole class is
shut: `crate::fxhash::HashMap` / `HashSet` (rustc's seedless FxHasher)
replace `std`'s across the engine, so no map's walk order can differ
between two runs of one seed. Same-seed decision counts are identical over
repeated runs on every pool (`cube` 1,130,728, `all` 2,548,986, `sos`
684,268, `fixed` 193,232), `determinism ok` on all of them, and `all`'s
stall rate is a stable 6 rules draws / 5,100 games (0.12 %). A separate leak
fixed in the same sitting (`125108c1`): CR 705.1 coin flips read
`rand::random()` inside `AutoDecider`, and Mana Crypt is in the cube pool.
**Re-checked at the forty-seventh tip**: `--decks all --games 400 --threads
3`, seeds 11/12/13 — 20,400 games, 20,396 decided, no panic, all 10,198
mirrored pairs split.

**What is left of it, as a rules question, not a determinism one.** A map
whose walk order picks a *game outcome* is still arbitrary, just
reproducibly so. **The sweep this entry used to invite is done (ninety-second
pass) and the site it named is already fixed**, which is why the list below
replaces the invitation rather than extending it.

*The named site, now the pattern to copy.* `actions.rs:15416`'s discard-cost
gate reads `by_name.iter().filter(..).min_by_key(|(name, _)| (mv_of[name],
**name))` — the mana value first, **the name as a total-order tie-break
second**. One extra tuple element and the walk order cannot reach the answer.

*The three siblings the locals sweep found, none of them a bug.* Each is a
free choice under the rules (any legal pick is legal), so each is recorded
rather than changed — a fix moves golden traces on tie boards for no
correctness gain. What they cost is **fragility**: the tie-break is the hash
layout, so adding a card to a pool can silently move a trace.

| site | what the walk order decides | shape |
|---|---|---|
| `effects/mod.rs:28718` | `chosen_number` — the most common mana value among opponents' graveyards | `counts.into_iter().max_by_key(\|(_, n)\| *n)`, ties by walk order |
| `effects/mod.rs:4183` | which three differently-named creatures survive `truncate(3)` before the random pick | `into_values()` then a **stable** `sort_by_key(Reverse(mv))`, so equal-mv names keep hash order |
| `bot.rs:3362` | the bot's answer to "choose a creature type" | `tally.into_iter().max_by_key(\|(_, n)\| *n)`, ties by walk order |

*Checked and clean, so nobody re-checks them:* `eval.rs:666` / `838`,
`effects/mod.rs:770` and every `counters.values()` read are `sum` / `max` /
`any` / `all` folds; `effects/mod.rs:5091` and `32559` and `stack.rs:3058`
are looked up by key; `selfplay.rs:341` collects `avail.values()` into a Vec
and then indexes it with the RNG, which is uniform whatever the order;
`recommend.rs:2452`'s `strata.values()` pools `f64`s, so the order reaches a
*metric* and not an outcome. Outside the engine crates the same sweep found
one `std::collections::HashMap` that is iterated —
`selfplay_train.rs:2895`'s `by_traj`, into an `f64` sum — and it is the only
site in the tree whose order can differ *between processes*; it prints at
`{:.4}` on a probability scale, four orders below where the reassociation
shows, so it is listed here rather than fixed.

*(No open entries. The audits that closed here are an index; `git log -S` on
each hash has the prose.)* `df87c2d1` — `CardData.counters` becomes the
insertion-ordered `CounterBag`; `86670250` — the same for `KeywordCounters`;
`ea8cc1fd` — `died_card_snapshots` becomes `IdMap`, because a
`TriggerCandidate`'s position decides stack order and two LKI deaths stacked
differently per process. **That was the survey's one leak in 31 fields** —
every `HashMap`/`HashSet` on `GameState` / `ColdState` / `Player`, asked of
each consumer whether it sums, tests membership or looks up by key (all safe)
or `find`s / `collect`s / iterates into an ordered structure (not). The three
that *look* risky and are not, so nobody re-checks them: `encode.rs` sums
`block_map` into a map read by key, `bot.rs`'s two `block_map.keys().collect()`
are `contains` + `len`, and `combat.rs`'s `block_map.keys().for_each(want)`
decides which permanents get computed, never what a reader sees. Also
`a67c5b9a` (actor-sampler panic) and `9db8557c` (Mirror Gallery aborting the
whole SBA sweep — a `return` inside a `let … = { … };` initializer, so one
board skipped every later state-based action and the game could not be won or
lost; regression test in `classic_sets/bok`, and the filter that found it was
swept workspace-wide).

**The panic/unwrap sweep of the self-play path — CLOSED 2026-08-23 by the
census under filter 16 (written up below): it wanted triage, not the blanket
rewrite this entry used to ask for, and came back clean.** The narrower
filters below are what got run instead, and nine of them found nothing —
which is the result worth keeping. The section has **no open entries**.

**The seventeen filters, compacted to an index.** Each is a *shape* that fails
the way a training run notices — a silent wrap at game 400 k, a loud panic,
or a hang — swept over `game/` + `bot.rs` (some wider). The prose is in
`git log -- TODO.md`; what is kept is what each hunted and why it is
closed, so none of them is re-derived.

| # | date | the shape it hunts | result |
|---|---|---|---|
| 1 | 08-10 | A `debug_assert!` standing in for a runtime guard, or a `len() - 1` / bare index on a slice whose emptiness the *caller* tolerates | **Found `a67c5b9a`** (`sample_scored_index`, on the one path only a training actor takes). Both halves then swept clean: 13 `len() - 1` sites all guarded; the two surviving `debug_assert!`s (`mod.rs:3900`, `stack.rs:5911`) fall through to defined release behaviour |
| 2 | 08-10 | A `return` inside a `let … = { … };` initializer | **Found `9db8557c`** — CR 704.5j's Mirror Gallery check aborted the *whole* SBA sweep, so one board skipped every later state-based action and the game could not be won or lost. Regression test in `classic_sets/bok`. The other nine workspace hits are `Err` / let-else guards that legitimately abort |
| 3 | 08-10 | Unsigned `len() - k` where the caller tolerates empty; a stale index across a mutation (`position()` then `battlefield[pos]`) | Clean. 16 + 53 sites; the one path that mutates in between (the equip sacrifice) re-finds by id and says why |
| 4 | 08-10 | `evaluate_value(…) as usize` with no `.max(0)`; `power()`/`toughness()`/`life` cast to `usize` | Clean. One hit each, both already clamped (`mod.rs:20879` is `.max(1)`; `bot.rs`'s `LIFE_TENTHS[life as usize]` sits under its own two branches) |
| 5 | 08-10 | A precondition *some* sites enforce and a sibling might not — documented `///` preconditions, and the `i.min(xs.len() - 1)` clamp family | Clean. Eight doc'd preconditions all validated or structural; all ten clamps guarded, by four different idioms |
| 6 | 08-11 | Not syntax — **run the arithmetic**. `[profile.overflow]` (`release-fast` + `overflow-checks`) turns every silent wrap into a panic with a backtrace | Clean. `bot_ladder` 4 seeds x 4 pools = **17,693 games, 0 panics**; `selfplay_train --actors 3 --games 600` = 600 games / 56,353 rows / 0 panics. **Rerun after any change to counters, damage, mana or the encoder** — one ~9-minute build, ~1 minute a seed |
| 7 | 08-11 | The opposite of 1-6: a `/` or `%` whose denominator is a runtime count the caller can zero (panics loudly, or goes `NaN`) | Clean. Every non-constant divisor under `game/`, `bot.rs`, `crabomination_ml/` read; seat rotation always has a seat, the rest are `.max(1)` or guarded by an `is_empty()` in the same condition |
| 8 | 08-11 | A std collection/slice op whose runtime argument is a *length*, not an index — `split_off`/`split_at`/`copy_from_slice`, `chunks`/`step_by` with runtime `n`, `&xs[a..b]`, `Vec::remove`/`insert` | Clean. Five + two + two + ~30 sites; the ML `copy_from_slice`s copy fixed-width **arrays**, so a mismatch is a compile error, not a panic |
| 9 | 08-11 | A comparator that is not a total order (`sort_by` panics on one; a `NaN` produces one for free) | Clean. **No `partial_cmp(…).unwrap()` in the workspace**; every float comparator is `total_cmp` or `unwrap_or(Equal)`, and the three that could see a `NaN` are in `recommend.rs`, off the self-play path |
| 10 | 08-11 | The failure a training run sees as a *hang*: an unbounded `loop`/`while` whose exit condition is game state | Clean. All eight `loop {` and ~40 `while`s bounded by one of three shapes — a strictly shrinking collection, a finite effect-tree peel, or an explicit counter. The one bounded by none of the three is the top-level game loop, and its two counters are exactly what a *stall* is |
| 11 | 08-11 | **One invariant written out by hand in more than one place** | **Found `15ec11c1`**: `stale < 8` appeared six times across five files, so the ladder's stall rate and the training actor's were never the same measurement. All six read `recommend::STALE_ROUNDS` now; no value changed. The per-context *action* budgets beside them are deliberately different and were left alone |
| 12 | 08-14 | **A predicate two callers each re-derive** | **Found two**, `caa44eb2` — see below |
| 13 | 08-14 | **A reentrancy guard some sites spell out by hand and a sibling does not** | **Found a stack overflow** — `in_layer_gather`, unguarded at ~a dozen computed-P/T arms the gather evaluates. See below |
| 14 | 08-15 | **A comment that states a cost or a shape** | **Found one**, in the measuring device: `host_calib_ms`' doc claimed you could scale a throughput comparison by it. The syntactic half is clean; see below |
| 15 | 08-23 | **A claim made by a tool's output rather than by its source** | **Found one**, in the measuring device again (`95453974`): `--bench` printed "release build" for `release-fast` / `profiling-fast` / `overflow`. See below |
| 16 | 08-23 | A default no caller ever overrides | Clean in four readings; don't re-run — see below |
| 17 | 08-23 | **A comment that names a call count or a share** | **Found five** across two concurrent runs. The share survives, the count rots. See below |
| 18 | 08-23 | **A tool's own extraction step** | **Found two** (`ac85463f`), both in the profiling scripts: `cg_edges.py`'s total was ~18x high, `cg_lines.py` returned a silent zero. See below |
| 18b | 08-24 | filter 18 re-run on `cg_lines.py` (**a tool's own extraction step**) | **Found two more.** It folded every mapped object's addresses (libc, ld.so, libm — 16.5 % of the run) in with the binary's and hardcoded the PIE bias. 36 % of the run resolved to `??` and the rest to the wrong symbols; `Effect::clone` read 2.65 % against the 0.5 % its call edges account for. See below |
| 20 | 08-24 | **A default that only one caller ever exercises** (the inverse of the sixteenth) | **Nearly clean — one hit.** 14 of 36 `EvalWeights` knobs have exactly one overriding profile; ten of those profiles carry an on/off test, four do not, and three of the four are correctly untested (a scoring weight, a historical control, a net-dependent blend). The fourth, `smart_tap`, was real engine behaviour with no test; it has one now. See below |
| 21 | 08-24 | **An invariant checked at one point in its parameter space** | **Found a determinism bug** (`c6898506`). The wide-pool sweep had only ever run three seeds at one thread count; ten more seeds across `--threads 1/2/3` produced a self-mirror pair that did not split. `restart_game` (CR 727) rebuilt the state with `GameState::new`, whose `GameRng` is `from_entropy`. See below |
| 22 | 08-24 | **State the harness installs that a rules path can reset** | **Three sites, one real.** `restart_game` dropped the seeded `rng`, the live `decider` and two pilot flags (fixed, filter 21's bug). `play_subgame` already forks the stream and is correct. `GameState::rng` is `#[serde(skip)]` deliberately — but this module's summary claimed a `GameState` round-trip was bit-exact, which it is not; the claim is corrected, the field is not |
| 19 | 08-24 | **A threshold or cap that silently truncates a listing** | **Found seven** across two concurrent runs — `cg_edges.py`'s three tables and `cg_lines.py`'s two caps (`4107e017`), plus `cg_symbolize.py`'s recommended `--threshold` and three ranked report tables in `bot_probe` / `selfplay_train` / `recommend_pool`. See below |
| 23 | 08-24 | **An invariant checked at one thread count** (filter 21's shape, at the measuring device) | **Guard added, clean (`1c304384`).** The `--bench` self-mirror determinism check ran at one thread count and the decision count was never asserted invariant across counts, yet the aggregate is a commutative sum over seed-fixed jobs. `CRAB_THREAD_CHECK` replays the identical workload at a contrasting count and asserts the order-independent outcome matches; `run_jobs` factored so the loop is not written twice (filter 11). Clean at the tip (196,220 dec, 3 vs 1 threads). See below |

**A note the table would lose**: filters 3-5 and 7-10 are syntactic and
found nothing between them. Filter 6 is not syntactic — it *runs* the
program with the checks on — and filters 2, 11, 12 and 13 look at structure
rather than syntax. Four of the five filters that found something are in
that second group. Prefer a filter that runs the code or reads its
structure over one that greps it.

**Filters 12-14, compacted; `git log -- TODO.md` has the prose.** All three
hunted structure rather than syntax and all three found something.

* **12 (`caa44eb2`) — a predicate two callers each re-derive.** The search
  that works is not syntactic: it is `grep -niE "must (also )?(appear|be)
  (listed|added|here)|kept in sync|must agree|drift from"`, i.e. the doc
  comments that admit to the pairing. Nine hits, three real.
  `CardData::clear_end_of_turn_effects` wrote 26 fields and
  `end_of_turn_effects_are_clear` guarded that write by listing the same 26 —
  both expand from one `eot_wear_off!` list now, so a field cannot reach one
  without the other. `rewrites_land_types` asked in prose to be kept in step
  with `layers::compute_permanent_pass` — now the `ability_strip_in_scope`
  device, a `debug_assert!` at the gate that runs the layer pass it skipped
  and fails if the computed land-type line differs from the printed one.
* **13 — a reentrancy guard some sites spell out by hand and a sibling does
  not.** Found a **stack overflow**: `in_layer_gather` was unguarded at ~a
  dozen computed-P/T arms the gather itself evaluates, so a card pairing a
  gather-evaluated filter with a P/T requirement overflowed rather than
  answering wrong. The guard lives in `computed_permanent` now, once, where
  it cannot be forgotten.
* **14 — a comment that states a cost or a shape.** Found one, in the
  measuring device: `host_calib_ms`' doc claimed you could scale a throughput
  comparison by it, and two containers with the same `host_cpu` and
  overlapping calib differed by **24 %** on `--bench`. **The syntactic half is
  clean and should not be re-run** — ~60 hits for present-tense cost claims
  over `game/`, `server/`, `crabomination_base/`, all game-semantics uses of
  "free"/"cheap" or past-tense justifications. Both filters' yield was in
  claims a *measurement* relies on, not in the engine's prose.
**Sixteenth (2026-08-23) — a default no caller overrides — CLEAN in four
readings, don't re-run:** zero bool params pinned to one literal, zero
`Option<T>` always-`None`, zero of 36 `EvalWeights` fields, zero of 196/91
`GameState`/`ColdState` fields never read. The loose bool-literal pass
returns 14, all noise (`default_damage_split(has_trample)` etc.).

**Standing panic goal, audited the same day — CLEAN.** 118 non-poison
`unwrap`/`expect` in engine code; every self-play-reachable one is guarded
and names its guard (`back_face…` behind `has_back`, `max_by_key().unwrap()`
behind `is_empty()`, …). One dead hazard, not reachable:
`Index<&CounterType> for CounterBag` panics on a missing kind and has no call
site. Re-run the census after a batch of new cards, not per run.

**Seventeenth (2026-08-23) — a comment naming a call count or a share — four
hits (plus two from pass 48), all corrected in place:** `printed_color_set`,
`team_of`, dispatch's death-synthesis chain, `auto_tap_for_cost_inner`'s mana
table, `mod.rs`'s gather-iterator note, `types.rs`'s `IdSet` doc. **The rule:
the share survives, the count rots** — a share is re-derived on every profile
read; a call count is copied forward and drifts as its caller's count moves.
Caveat: a callgrind edge count *undercounts* an inlined caller, so only
correct a number down when the callee's own node makes the old claim
arithmetically impossible (why `team_of`'s correction is stated through
`same_team`'s node).

**Twenty-first (2026-08-24) — an invariant checked at one point in its
parameter space — NOT clean, a determinism bug 49 passes of wide-pool sweeps
had missed.** The sweep (`--decks all --games 400 --threads 3`, seeds
11/12/13) had never run on another seed or thread count; ten more seeds found
a self-mirror pair that did **not** split. Cause: `GameState::restart_game`
(CR 727) rebuilds with `GameState::new` and copies back `next_id`/
`attack_option`/`teams` but not **`rng`** (nor `decider`/`smart_tap`/
`wants_ui`), and `GameState::new` installs `from_entropy`, so a restarted game
stopped being a function of its seed. Fixed `c6898506`; the regression test
replays the exact pair on eight threads. **Two things worth keeping:** the
thread count was a red herring (it only changed how often the halves drew the
same entropy) — *the parameter that exposes a bug is not always the one that
causes it*; and `CRAB_PAIR_SWEEPS=1` naming the offending pair + its replay
seed turned "somewhere in 68,000 games" into a 1.2-second test. The structural
fix is the CLAUDE.md rule: a profile number in a comment carries the tip it
was measured at, or is past-tense justification for the shape already there.

**Eighteenth (2026-08-23) — a tool's own extraction step — NOT clean, two
hits in the profiling scripts (`ac85463f`).** `cg_edges.py`'s program total
ran ~18x high (it summed each call-edge's whole *inclusive* subtree), so
every share was an order out — it read `dispatch_triggers_for_events` at
**0.30 %** where it is 5.63 %; PERF's note that the total double-counts did
not fix the percentage column the tool actually prints. `cg_lines.py`
printed "0 Ir, exit 0" on a dump without `--dump-instr=yes` — nothing-is-hot,
not wrong-input. **The rule: every extraction step either agrees with a
number its source computed itself, or refuses** — an extraction that yields
nothing looks exactly like a measurement that found nothing.

**Nineteenth (2026-08-24) — a cap that silently truncates a listing — NOT
clean, seven hits (`4107e017`, `17d0a5e1`).** Both profiling scripts capped
their tables (`most_common(40/45/60000)`) and read as finished at the cap;
`cg_edges.py`'s docstring promised a complete table above one;
`cg_symbolize.py` recommended the `callgrind_annotate --threshold` truncation
`cg_edges.py` exists to escape; three ranked reports named no denominator.
Each reports the rows and Ir it dropped now (`--rows 0` lifts the cap). It did
NOT flag the engine's own named search caps (`attack_search`,
`MAX_CANDIDATES`, …). **Filter 18 re-run on `cg_lines.py` found two more:** it
folded every mapped object's addresses in with the binary's and hardcoded a
`0x108000` bias (`Effect::clone` read 2.65 % against its 0.5 % call edges); it
keeps one object and auto-detects the bias now, and PERF's `drift::sort` row
blamed on lld ICF is probably this bug. **The self-cost table's top 45 rows
are 68.5 % of the program, 1,150 rows hold the rest** — why pass 49 counted
call rows rather than ranking by self cost.

**Twentieth (2026-08-24) — a default only one caller exercises, the inverse
of the sixteenth — NEARLY clean, one hit.** Of `EvalWeights`, 14 fields have
exactly one overriding profile; ten carry an on/off unit test beside their
`bot_ladder` pilot name ("flag off: the class is invisible / flag on: the
activation is a candidate"). **Three of the four that don't are correctly
untested:** `power_emphasis_only` is a scoring *weight* (`power: 15`), so
the question it asks is a ladder question; `legacy_cashout_on` is the
*historical* planeswalker rule kept as a control, and the shipped behaviour is
what a test should pin; `net_eval_blend_ply` needs a loaded net. **The fourth
was a real gap.** `smart_tap` routes through `PlayerData::smart_tap` into
`auto_tap_for_cost_inner`'s source choice, where it makes a coloured pip spend
the *least flexible* source — the engine's own comment says "a Swamp pays {B}
before a Dimir dual does" — and nothing tested it.
`core_rules::game::smart_tap_spends_the_narrowest_colour_source_first` does
now: same board both ways, and the two arms assert opposite outcomes, so it
cannot pass vacuously.

**The rule it yields:** an opt-in flag in this codebase is expected to carry
both a ladder pilot name *and* an on/off test, and the ten that do are what
make the four that don't findable. A flag whose question is a *measurement*
(a weight, a control, a net) is the documented exception — say so at the
flag, so the next sweep does not re-derive it.

**Stall rate — CLOSED 2026-08-14, and the answer is "nothing to fix".**
`419d2ea6` put `recommend::StopReason` on the outcome and a `stalls_by cap /
stuck / draw` line on `--bench` (and `stalls_capped` / `stalls_stuck` in
`selfplay_train`'s `stats.jsonl`), and reading it settled the entry: `--decks
all --games 300 --seed 11` reads 6 stalls in 5,100 games (0.12 %), **cap 0 /
stuck 0 / draw 6** — all rules draws, so neither held-open fix applies.
`--decks fixed` reads 0 and always has. Keep the instrumentation; re-open
only if `cap` or `stuck` goes non-zero.

**Twenty-third (2026-08-24, `1c304384`) — an invariant checked at one thread
count, at the measuring device — guard added, clean.** The `--bench` self-mirror
determinism check (every mirrored pair must split) ran only at the one thread
count a bench invocation uses, and the decision count was never asserted
invariant across counts — yet every job is fixed by its `--seed`-derived
stream and the aggregate is a commutative sum over jobs, so it *must* be
independent of how many workers pull them. `CRAB_THREAD_CHECK` replays the
identical workload at a contrasting thread count and asserts the
order-independent outcome (SimCost fields + per-archetype win tallies + sorted
pairs) matches; the chunked job loop is factored into `run_jobs` so the two
runs share one loop, not a second drifting copy (filter 11). Clean at the tip
(1 vs 2). This is the cheap in-process form of filter 21's wide seed x thread
sweep — the class where `restart_game` drew from OS entropy diverges the two
counts here. **The rule: a determinism check is only as wide as the parameter
it varies; a harness that measures at one thread count should be able to prove
the count does not matter.**

## Decision-plumbing audit (2026-07): bare `decider.decide` sites

> ⚠ **RE-RUN MECHANICALLY 2026-08-28 AND THE "~45 LIVE BUGS" BELOW IS STALE.**
> `scripts/audit_decision_plumbing.py` classifies every `decider.decide` site
> by whether its own statement region carries one of the plumbing markers —
> `seat_suspends` + `suspend_signal`, `stashed_resolution_answer`,
> `pending_decision`/`wants_ui` (the action-time suspension), or an explicit
> `DeciderKind` branch. Reading, re-run 2026-08-31: **195 sites, 99 plumbed,
> 96 bare** (was 97/98 on 2026-08-28 — two more plumbed since).
>
> **Every effect the classes below name by name is plumbed now** — Cascade,
> Madness, Dredge, Ripple, Cipher, Forage, Collect Evidence, Discover, the
> four free-cast primitives, Possibility Storm, Fateseal,
> `ChooseNumberDestroyByPower` ("the worst single finding"), `MayPayGenericUpTo`
> — checked by grepping the filter's bare list for each: zero hits. The work
> landed across the intervening passes and nobody updated this section.
>
> **"Bare" is not "bug", and the calibration matters**: three sampled at
> random came out one false positive (`gather_combat_damage_decisions`
> suspends through `pending_decision`, which is why that marker is in the
> list), one live class-5 default (`Effect::AddMana`'s colour picks — **fixed
> 2026-08-28**, see below), and one arguable (`Effect::MayRepeat` declines
> for a bot, which is a weak choice rather than a wrong one). Treat the 98 as
> a **triage population and a number to compare against**, which is what the
> filter is for; the class list below is history.
>
> **Closed 2026-08-28 — the `Effect::AddMana` colour family (class 5).**
> Seven sites asked "add one mana of a colour of your choice" and every one
> fell back to `legal[0]`/White for a headless seat, i.e. for every seat in
> the training path, wasting the pip for any non-white deck. They go through
> one `GameState::chosen_mana_color` now, which asks a real decider and
> answers a headless one with `best_color_for_hand_among` — the needs-aware
> pick the extra-mana riders had used since they were written, and the third
> shape of the question was the only one still asking. Regression:
> `classic_sets::nms4::harvest_mage_picks_the_colour_the_hand_needs_when_nobody_is_asked`.

The 2026-07 reading, kept as history. ~125
direct `decide` call sites audited across `effects/mod.rs`,
`effects/movement.rs`, `combat.rs`, `stack.rs`, `game/mod.rs`,
`actions.rs`. ~45 were live bugs, in five classes. AutoDecider defaults
for reference: OptionalTrigger→no, ChooseAmount→0, ChooseCards→first
`min` (empty when min=0, the "up to N" case), ChooseColor→first legal
(≈ always White), ChooseMode→0.

**Class 1 — whole keywords dead for every seat** (bare OptionalTrigger,
auto-declined): Madness (`mod.rs:8510`, ~17 cards), Dredge
(`mod.rs:9022`, ~15 cards), Cascade (`effects/mod.rs:17508`), Ripple
(17587), Cipher (18311), Forage (17933), Collect Evidence (17775/17797
AND 17852/17867 — both the wants_ui and bot branches are broken),
Discover's free-cast half (17650), CastFromHandWithoutPaying (18150),
CastWithoutPayingImmediate (17995 — kills SOS Improvisation Capstone),
CastAnyOrderWithoutPaying (18098), CastFreeParadigmCopy (18259),
Obzedat-style exile-blink (5807), Amped Raptor energy-cast (4065 —
worse than no-op: exiles the top card, then never casts it).
**Possibility Storm (15340) is actively destructive**: the original
spell is gone and the dug card stays in exile.

**Class 2 — "choose up to N" resolves as zero** (ChooseCards min=0):
Command the Dreadhorde (6640), three reanimation piles (6560, 6596,
6793), tutor-to-total-MV (6689), tap-any-number pump (6370),
Archipelagore tap (6968), Aether Vial-style PutFromHandOntoBattlefield
(10884), DeployCreatureFromHandAttacking (10975), Fateseal (4931 —
Jace +2 is a no-op), mill-then-take (4486), dig-to-hand (4965 — still
pays the self-mill, takes nothing), MayExileFromYourGraveyard rider
(5968), graveyard-exile hate (5924, 19899), SearchSplitOpponentChooses
(11629).

**Class 3 — amount defaults to 0**: ChooseNumberDestroyByPower (5580)
— **destroys every creature including the controller's own board**
(worst single finding; Expel the Interlopers); MayPayGenericUpTo (2607
— Wildborn Preserver never pumps); Sanctum Prelate locks 0 (16317);
Read Ahead sagas always start at chapter I (stack.rs:770).

**Class 4 — inverted wants_ui gates**: the human branch calls `decide`
synchronously (no suspension) while the bot branch has a real
heuristic — interactive seats play WORSE than bots:
SacrificeSourceUnlessSacrifice (10656 — a human's Gitrog dies every
upkeep, a bot's survives), ReturnGraveyardCardsToHand (6832),
ShuffleGraveyardCardsIntoLibrary (6879), PlayerReturnsPermanentsToHand
(10571), DistributeCountersAmongLastCreated (13534), PayAnyEnergy
(3741 — polarity fully reversed: bots pay all, humans pay zero),
CollectEvidence (see class 1). Also stack.rs:67's modal-trigger gate
skips suspension whenever ANY mode requires a target.

**Class 5 — quality-of-play defaults** (playable but wrong):
ChooseColor → White everywhere it matters
(GrantProtectionFromChosenColor 8058 — Mother of Runes always names
white; extra-mana AnyColor actions.rs:1692; Oona 19945 — the intended
Blue fallback is unreachable); legend rule keeps the NEWEST copy
(stack.rs:2858 — sacrifices the aura'd/countered older copy); owner
tuck choices always pick bottom (movement.rs:1197, 1216); coin-flip
repeat loops always stop at one win (1893); `MoveChosen` (10520) has a
dead `up_to` ternary — both arms identical, so "up to N" is enforced
as "exactly N" for every seat.

**Bot-side mirror bugs** (`server/bot.rs`): un-introspectable
ask_seat_bool prompts fall into `optional_trigger_beneficial`'s
`.unwrap_or(true)` — blind YES to "Pay N life to deny…", "Accept the
tempting offer?" (always accepts opponents' offers), echo/cumulative
upkeep (pays forever), clash (always bottoms), tribute (always
counters). Root gap: the source lookup scans battlefield/graveyard/hand
but NOT the stack, so any resolving spell's self-costly MayDo gets
blanket-yes.

**STATUS (fixed on claude/modern_decks, 2026-07):** all five classes
plus the bot-side mirrors are addressed — suspensions (AmountAnswerPending,
new CardsAnswerPending + ask_seat_cards/choose_up_to_cards, new
MayCastExiledPending completion), DeciderKind::Auto policies where
suspension is out of architectural reach, and bot prompt policies
(life-tax guard, tempting-offer decline, upkeep-value check, stack-zone
source lookup). ScriptedDecider always retains authority (suspension and
policies engage only for the live AutoDecider).

Deliberate remainders (policy-only or unchanged, each documented at the
site): Madness/Dredge interactive modals need resumable discard/draw
flows; Fiery Gambit's flip-again loop; Read Ahead's chapter pick
(ETB-time, no suspension reach); Amped Raptor's energy free-cast and
Ripple's chained offers (policy yes); per-token counter distribution
(even split for all seats); owner tuck choices and the AnyColor
extra-mana pick (smart defaults, no agency); legend-keep is a smart
default — the client's ChooseLegendToKeep modal still needs an engine
suspension to ever fire; single-stash constraint limits multi-ui-player
loops (EachPlayer shuffles) to one suspension per resolution.


# Engine mechanics & primitives

## Engine — Missing Mechanics

**"LOSES ALL ABILITIES" — CLOSED AS A CLASS, two cards, and it is now a
ratchet.** `a_card_that_prints_losing_all_abilities_strips_them` reads the
printed oracle for the clause and requires the definition to reach one of the
seven strippers. **Merfolk Trickster** and **Tishana's Tidebinder** printed it
and never stripped: the Trickster's rider was documented as dropped, which is
how it stayed dropped, and the Tidebinder's was not documented at all. Both
riders are the half that wins games — a blanked blocker has no flying, no
ward, no death trigger.

The Tidebinder needed a primitive: `Duration::WhileSourceOnBattlefield`, for
an effect a *resolution* installs "for as long as this creature remains on
the battlefield". A static with the same clause needs no sweep (it is
re-gathered every pass); a resolved one does, and it now has the same
`continuous_effects` retain the `WhileSourceTapped` and `WhileSourceAttached`
clauses have, next to them.

**"IN ADDITION TO ITS OTHER TYPES" — CLOSED AS A CLASS, seven cards, and it
is now a ratchet.** `no_card_replaces_the_types_its_oracle_adds`
(`core_rules/catalog_registration.rs`) reads the printed oracle for the
clause and refuses a definition built on a primitive that *replaces*. The
list of replacing primitives is spelled in the test rather than inferred,
which is the point: a new one has to be added to it.

The seven, and they read alike from the outside — **Ensoul Artifact**,
**Zoetic Glyph** and **Unable to Scream** on `equipped_bonus.set_card_types` /
`set_creature_types`; **Xenograft** on `CreaturesYouControlAreChosenType`,
which is *Conspiracy's* replacing static and the two cards' text differs by
three words; **Prismatic Omen**, **Leyline of the Guildpact** and **Nylea's
Presence** on a `GrantAllBasicLandTypes` that emitted a `SetLandTypes`. An
ensouled Darksteel Citadel stopped being a Land and stopped tapping for
mana.

**`SetLandTypes` FOR "IN ADDITION" — CLOSED, and the read turned up a second
defect next to it.** `StaticEffect::GrantAllBasicLandTypes` emitted one
`Modification::SetLandTypes`, and all three cards on it — Prismatic Omen,
Leyline of the Guildpact, Nylea's Presence — say "every basic land type **in
addition to their other types**". The Set replaced, so a Prismatic Omen took
the `Gate` off a Gate and the `Urzas` off an Urza's Tower, Tron check
included. Now five `AddLandType`s.

The other half was the opposite error: `equipped_bonus.set_land_types` has
three users and the Set is right for all three ("enchanted land **is** an
Island / a colorless Forest"), but **two of them shipped without CR 305.7's
ability loss** — Sea's Claim and Lingering Mirage left the enchanted land its
old abilities, so a Sea's Claim on a tri-land made it tap for four colours.
Song of the Dryads had `remove_abilities: true` throughout.



### ~~Triggers that live in the graveyard — the scope exists; three gates do not~~ — CLOSED 2026-09-10 (sixth find)

`EventScope::FromYourGraveyard` is the graveyard-resident trigger (the
dispatcher walks graveyards for it; 44 cards use it). The `cnt` triage
filed three cards as wanting a gate; every gate already existed
(`AttackedWithCreatureMatching`, `PutIntoGraveyardFromBattlefieldThisTurn`
as a requirement over `CardsInGraveyardMatching`, `PlayerDrewAtLeastThisTurn`
+ `once_per_turn`, `JoinCombatAttacking` for "tapped and attacking") — what
was missing was the *walks*: `YouAttack` and any-player step triggers from
the graveyard, and once-per-turn in the dispatcher's graveyard walk. All
three cards shipped with the fix above.

### The cnt triage's other primitives

One line each, from the same triage — the engine has nothing to spell
them and each blocks one shipped card: a name-prefix requirement
(`NameStartsWith("Gideon")`, Gideon's Company); "Curses attached to you"
as a selector (Witchbane Orb); a spend restriction for Mount / Vehicle
spells and one for casting from the graveyard (Intrepid Stablemaster,
Rootcoil Creeper); a variable "tap X untapped artifacts" cost —
`tap_n_filter` is fixed-N (Secluded Starforge); "whenever you *cast* a
creature spell this turn" as a delayed watcher — only the entering variant
exists, so Glimpse of Nature draws on tokens (a WRONG row, not a missing
one); a kicked-cast trigger (Sowing Mycospawn); "an opponent gains control
of a permanent from you" (Zidane); "all non-Wall creatures you control
attack" (Mob Mentality); an Aura returning attached to what it enchanted
(Takklemaggot); "crewed by this creature this turn" (Balthier and Fran);
an excess-damage *event* — `Value::ExcessDamageDealtThisResolution` exists
but nothing fires on it (Magmatic Galleon); craft (Market Gnome); claim
the prize (The Most Dangerous Gamer); an outside-the-game zone (Spawnsire
of Ulamog); copying a triggered ability on the stack (Firebender
Ascension); Wickerwing Effigy's cast-from-library rewrite; Ghost Vacuum's
mass return of the cards it exiled; Bloodthirsty Adversary's exile-and-
copy of graveyard spells; Conduit of Worlds' cast-from-graveyard with a
one-spell lockout. An Aura's granted keyword is invisible to
`sacrifice_when` (it reads the instance's own and EOT-granted keywords):
Floodgate under Flight stays.

### Replacement Effects
The engine has no general replacement-effect primitive.  Many real cards need one:
- ETB replacements (Containment Priest, Torpor Orb, Rest in Peace)
- Damage replacements (protection, preventing damage):
  - 🟡 **Combat damage prevention** (Owlin Shieldmage, Holy Day, Constant
    Mists) is partially supported via the new `Effect::PreventAllCombatDamage
    ThisTurn` primitive + `GameState.prevent_combat_damage_this_turn` flag
    (CR 615.1). Per-source / per-N shields (Wojek Apothecary, Stave Off,
    Lapse of Certainty) are still ⏳. Non-combat damage prevention
    (Reverse Damage, Mending Hands) is also ⏳.
- Draw replacements (Leyline of the Void)
- Death replacements (Kalitas, Oubliette)
Until this lands, cards with "instead" clauses are either stubbed or collapsed
into a close approximation.

### Per-Activation Mana-Spent Introspection
Reckless Amplimancer reads "+X/+X where X is the amount of mana spent to
activate this ability". The engine tracks per-cast `mana_spent` on
`StackItem::Spell` and per-trigger on `StackItem::Trigger`, but the
activated-ability path (`activate_ability`) doesn't capture mana spent.
Adding this requires:
1. An `x_value: Option<u32>` field on `GameAction::ActivateAbility` for
   X-cost activations (parallel to `CastSpell.x_value`).
2. Threading `mana_spent` through the activation's `StackItem::Trigger`
   construction in `activate_ability` (the field exists but is always 0).
3. Wiring `Value::CastSpellManaSpent` to read from the stack item.
Then Reckless Amplimancer's +3/+3 hardcode can be replaced with
`Value::CastSpellManaSpent` for printed-Oracle parity. Tracked as engine
work — same shape would unlock other X-cost activations (Berta's
{X},{T}: Create Fractal with X counters).

### Cast-From-Exile Pipeline
Many cards exile a spell/card temporarily and later cast it (Foretell,
Suspend, Rebound, Flashback-from-exile, Escape, Adventure second cast,
Cascade resolution).  Currently each is handled ad-hoc or omitted.  A shared
"cast from alternate zone" code path would unlock dozens of cards.

### Triggered-Ability Event Gaps
`EventKind` is missing several commonly-needed triggers:
- `PermanentLeftBattlefield(CardId)` — needed for general "LTB" abilities.
  (Linked exile-until-LTB now handled directly via `return_linked_exiles`
  / `CardInstance.exiled_by`, not via an event.)
- `DamageDealtToCreature` — needed for enrage, lifelink gain on creature damage
- `TokenCreated` — needed for populate, alliance triggers
- `CounterAdded / CounterRemoved` — needed for proliferate payoffs, Heliod combo
- `SpellCopied` — storm payoffs, Bonus Round
- `PlayerAttackedWith` — needed for Battalion and similar attack-count effects
- ~~`SpellCastTargetingCreature` (or a `Predicate::SpellTargetsCreature`
  knob) — needed for Strixhaven Repartee.~~ **Done**: see
  `Predicate::CastSpellTargetsMatch` + `effect::shortcut::repartee()`.
  Stirring Hopesinger, Rehearsed Debater, Informed Inkwright, Inkling
  Mascot, Snooping Page, Lecturing Scornmage, Melancholic Poet, and
  Graduation Day all use it. Remaining Repartee cards are blocked on
  separate primitives (exile-until-X, copy-spell). Ward enforcement
  (mana-cost variant) shipped in push (modern_decks) — see Inkshape
  Demonstrator promotion + `push_ward_triggers_for_cast` in
  `game/actions.rs`.
- ~~`CardLeftGraveyard` — needed for Lorehold "cards leave your
  graveyard" payoffs.~~ **Done** in push V: see
  `EventKind::CardLeftGraveyard` + `Predicate::CardsLeftGraveyardThisTurnAtLeast`.
  Hardened Academic, Spirit Mascot, Garrison Excavator, Living
  History all wired. Remaining gy-leave-aware cards (Ark of Hunger,
  Owlin Historian, Primary Research, Wilt in the Heat) need only
  catalog wiring against the event.

### Multi-Card Batch Triggers
The engine emits `CardLeftGraveyard` per card removed; printed cards
say "Whenever **one or more** cards leave your graveyard". We
approximate by firing the trigger per-card (a strict power upgrade
on multi-card-removal turns, but harmless in 2-player play where
single-card returns dominate). A future refinement: collapse a
batch of `CardLeftGraveyard` events emitted in the same resolution
window into one trigger fire (similar to MTG's "looks back in time"
rule for batch triggers). Same shape applies to `CardDiscarded`,
`CreatureDied`, and any future per-zone-move event.

**Per-event fan-out fix (push c4b7b14)**: The dispatcher previously
broke after the first matching event per (source, trigger) pair,
silently swallowing later events in the same batch. This was a
regression for multi-attacker swings (Sparring Regimen) and any
"whenever X happens" trigger over a batch of N events. The
dispatcher now keeps iterating over events for batch-fanout-friendly
event kinds (Attacks, CreatureDied, CardDrawn, CardDiscarded,
CardLeftGraveyard, CounterAdded, Blocks, BecomesBlocked, LifeGained,
LifeLost, BecameTarget) — one trigger fires per matching event,
matching the printed Oracle wording. Other event kinds (ETB,
StepBegins, …) keep the at-most-once guard because they don't emit
duplicate events in a single batch.

### Spell-Side Predicate: Mana-Spent-On-Cast
SOS introduces **Increment** ("if mana spent > this creature's P or T,
+1/+1 counter") and **Opus** ("Whenever you cast an instant or sorcery,
do X. If five or more mana was spent, do bigger X"). Both need a
per-cast "mana value paid" snapshot exposed as a `Value` (or a
`Predicate::ManaSpentAtLeast(n)`). The engine already retains the cost
on the `StackItem`; lifting that into the `EffectContext` for trigger
filters should unlock a few dozen Strixhaven cards.

### X-Cost and Converge
`Value::XFromCost` exists but converge (number of *distinct colors* of mana
spent) is not tracked per cast.  `Value::ConvergedValue` is a stub that always
returns 0 for non-Prismatic-Ending uses.  Fix: record color set paid at cast
time and expose it as a `Value` primitive.

### Cost-Reduction Stacking
Delve, Improvise, Convoke, and generic cost-reducers each have separate
branches.  There is no unified "reduce mana cost by X before payment" hook,
making cards like Hogaak (Convoke + Delve) or Affinity impossible to express
cleanly.

### Target-Aware Cost Reduction
"This spell costs {X} less to cast if it targets [some condition]" is a
Strixhaven design pattern (Ajani's Response, Brush Off, Run Behind,
Mavinda, Killian, Orysa). Today we either drop the discount and ship the
spell at its printed full cost, or omit the spell entirely. Engine fix:
let `CostReduction` static / per-card alt-cost evaluate against the
candidate-cast's chosen target before payment. Probably a new
`SelectionRequirement`-keyed cost discount that the cast path consults.

### Mana Ability from Non-Battlefield Zone
`activate_ability` only walks the battlefield.  Cards like Elvish Spirit Guide
and Simian Spirit Guide (exile from hand: add mana) ship as vanilla bodies;
the "exile from hand: add mana" half needs a from-hand activation zone (adding
an `ActivatedAbility.from_hand` flag parallel to `from_graveyard` would mean
touching ~240 literal constructors — migrate them to `..Default::default()`
first).

### Delirium-conditional static buffs
`Predicate::DeliriumActive` now gates spell effects (Unholy Heat). A
*continuous* delirium buff — "as long as you have delirium, this gets +2/+2
and has flying" (Dragon's Rage Channeler, Traverse the Ulvenwald-adjacent
cards) — needs a layer-system static whose application is gated on a
predicate. DRC isn't implemented yet pending this.

### Damage-as-(-1/-1)-counters replacement
Soul-Scar Mage / Phyrexian Vatmother-style "if a source you control would
deal noncombat damage to a creature, it deals that much in -1/-1 counters
instead" needs a damage-replacement hook. Soul-Scar Mage ships as 1/2 Prowess
without it. (Native Infect/Wither on the non-combat funnel shipped —
`deal_damage_to_from` lands -1/-1 counters / poison; CR 702.80a/702.90e.)

### Phyrexian mana
Mutagenic Growth ({G/P}), Gut Shot, Dismember, etc. — a mana symbol payable
with 2 life. Mutagenic Growth ships at the {G} cost (the life-pay alt is
omitted).

### "Look At Top X, Pick One, Put Rest in Graveyard" Primitive
Stirring Honormancer ("look at top X cards where X is creatures you
control, put one in hand, rest into graveyard") and similar look-and-
sort effects need a "look at top N, choose K, mill the rest" primitive
to express faithfully. `Effect::Surveil` covers the "look + may put in
graveyard" shape but with a fixed number; the SOS variant is dynamic
and forces the rest-to-graveyard branch unconditionally.

### Choice of "Which Zone" for a Tutor Result
Dina's Guidance ("search a creature, put into hand or graveyard")
exposes a 2-option destination prompt that no other primitive currently
needs. Adding a `Effect::Search` flavor with `to: Either(ZoneDest,
ZoneDest)` (or a separate decision shape) would honor the toggle for
this and a handful of black/green search effects.

### Multi-Target Prompt for Sorceries / Instants
A handful of SOS cards specify two target slots with different filters
(Render Speechless: opponent + creature; Cost of Brilliance: player +
creature; Homesickness: player + up to two creatures). The engine
today only exposes a single-target slot per spell at cast time, so
these collapse one of the two halves. A multi-target cast prompt
(`Vec<Target>` in `GameAction::CastSpell`) would unlock all of them.

### Auto-Target Picker: Source-Avoidance + Best-Pick Heuristics
~~The current `auto_target_for_effect` walks the battlefield in `Vec`
order and returns the first legal match.~~ **Source-avoidance done**:
the new `auto_target_for_effect_avoiding(eff, controller, avoid_source)`
takes the trigger source and prefers any *other* legal target,
falling back to the source only when nothing else is legal. All
trigger-creation paths (`stack.rs`'s `flush_pending_triggers`,
`actions.rs`'s ETB triggers, `combat.rs`'s combat triggers, the
delayed-trigger fire path, Dies/PermanentLeavesBattlefield triggers)
now pass the source ID. Quandrix Apprentice's Magecraft pump now
deterministically targets the bear over the Apprentice, and the test
suite asserts the source-fallback when no other target is legal.

~~Prefer the highest-power creature for friendly pumps.~~ **Done** in
push VI: `auto_target_for_effect_avoiding` now sorts the primary-player
candidate set by descending current power when the effect prefers a
friendly target (Magecraft / Repartee fan-outs, transient PumpPT
spells). Hostile picks still use first-match.

Remaining best-pick heuristics still ⏳:
- Prefer creatures whose current power matches what the pump would
  unlock (lethal swing, post-pump unblockable, etc.).

### Mana-Cost Reduction with Target Predicate
Killian, Ink Duelist's "spells you cast that target a creature cost
{2} less" needs a `StaticEffect::CostReduction` variant whose filter
inspects the cast spell's targets. Today's `CostReduction` filters
on the spell card's own attributes only. Plumbing the cast-time
target list into the cost-reduction site would unlock this card and
similar Lorehold/Witherbloom cost-cutters.

### Transient Triggered-Ability Grants on Pump Spells
SOS Root Manipulation ("Until end of turn, creatures you control get
+2/+2 and gain menace and 'Whenever this creature attacks, you gain
1 life.'") needs a way to attach a *triggered* ability to a creature
for a duration, on top of the keyword-grant primitive. Today the engine
has `Effect::GrantKeyword { what, keyword, duration }` but no
`Effect::GrantTriggeredAbility { what, ability, duration }`. Adding
this would unlock the third clause of Root Manipulation, similar
"creatures gain combat-damage trigger until EOT" pump spells, and
the on-attack rider on tokens (Pest token's "gain 1 on attack",
Spirit token combat triggers).

### Self-Counter-Scaled Cost Reduction
SOS Diary of Dreams's `{5},{T}: Draw a card` activation costs `{1}`
less per page counter on the source. There's no
`StaticEffect::CostReduction` variant whose discount scales off the
source's own counter count. Adding a `CostReduction { delta:
Value::CountersOn { what: Selector::This, kind: Charge } }` shape
would unlock Diary of Dreams cleanly, plus other counter-scaled cost
reducers (M21 Mazemind Tome).

### Counter-Removal Activation Cost
✅ Shipped as `ActivatedAbility.remove_counter_cost` (Walking Ballista's
`Remove a +1/+1 counter: deal 1`, Barkhide Troll's hexproof pump).
Experiment One's `Remove two: Regenerate` still pending a per-card pass.

### Page Counter Type
SOS Diary of Dreams (and the rest of the SOS book/grandeur subtheme)
references "page counter" but the engine `CounterType` enum has no
`Page` variant. Diary is currently approximated with `CounterType::
Charge`, which is fine in 2-player play (no other card uses Charge as
a payoff source) but obscures the printed identity. Adding `Page`,
`Knowledge`, and the small handful of other novelty counters from
recent sets would close the gap.

### `Move`-with-count for Selecting One Card from a Zone
Today `Effect::Move { what: Selector::CardsInZone { zone: Graveyard, ... } }`
moves *every* matching card. Cards like Heated Argument's "you may
exile a card from your graveyard" need a "move at most one matching
card" primitive. A `Selector::OneOf(inner)` wrapper, or a `count` knob
on `CardsInZone`, would fix this. The current workaround for Heated
Argument collapses the optionality into "always do the rider".

### "Choose Up To N Modes (with Repetition)" for `ChooseMode`
Strixhaven's "Choose up to four. You may choose the same mode more
than once." pattern (Moment of Reckoning, Witherbloom Charm-style
spells with N copies) needs an extension on `Effect::ChooseMode` that
takes a list of (index, target) tuples per cast. Today the engine's
modal flow picks exactly one mode and one target per cast — the
"choose up to N" wrappers collapse to single-mode resolution.

### "X Life as Additional Cost" Primitive
Vicious Rivalry, Fix What's Broken, and a handful of SOS sorceries
have "As an additional cost to cast this spell, pay X life." The
engine has no per-cast life-payment cost — we approximate by reading
X from the spell's `{X}` slot and running `LoseLife X` at resolution
time, but that double-counts X (paying X mana via XFromCost AND X
life). A `cost.life: Value` field on `CardDefinition` (or an
`alternative_cost` variant whose payment also requires the life)
would make this faithful.

### "Track Cards Discarded by This Effect" Counter
Borrowed Knowledge ("draw cards equal to the number of cards
discarded this way") needs a per-resolution counter that
`Effect::Discard` increments. The mode 1 path is currently
approximated as "draw 7" — a flat-7 reload that misses the printed
"draw exactly as many as you discarded" precision but preserves the
card-advantage tally for typical hand sizes.

### Capture-As-Target From Selector (Repartee Exile-Until-End-Step)
Conciliator's Duelist's Repartee body wants to:
1. Exile the cast spell's chosen creature target
   (`Selector::CastSpellTarget(0)` — wired).
2. Schedule a delayed trigger that returns *the exiled card* to
   battlefield at next end step.

Step (2) collides with `Effect::DelayUntil`'s capture model — it
captures `ctx.targets.first()`, but a Repartee trigger has no
target slot of its own (the selector is what tracks the spell's
target). Need either:
- An `Effect::CaptureTargetFromSelector { slot, selector }` that
  mutates ctx.targets so the subsequent DelayUntil reads it back, OR
- An `Effect::ExileWithDelayedReturn { what, kind, controller }`
  combinator that pre-resolves the selector at registration time.

The latter is more general. (Tidehollow Sculler / Banisher Priest /
Fiend Hunter are now handled by the dedicated
`Effect::ExileUntilSourceLeaves` / `ExileChosenUntilSourceLeaves`
primitives — see FEATURE_ROADMAP Tier-1 #4.) The former is smaller
surface but introduces effect-side mutation of ctx.

### "Move at most one matching card" — `Selector::OneOf`
Several SOS effects exile/move "a card" from a graveyard, hand, or
top of library where the count is at most 1 (Heated Argument's "may
exile a card from your graveyard", Practiced Scrollsmith's "exile
target noncreature/nonland card from your graveyard"). Today
`Selector::CardsInZone { ... }` returns ALL matching cards. Adding
`Selector::OneOf(Box<Selector>)` (or a `count` knob on `CardsInZone`)
would let these spells correctly pick exactly one. Without it, the
catalog approximates by "exile every matching card" which over-
shoots when the graveyard has multiple matches.

### Snow Mana Validation
`ManaPool` tracks a `snow` counter but `pay()` never validates that a `Snow`
mana symbol must be paid from a snow source.  Any mana from any land currently
satisfies a `{S}` pip.

### Multiplayer / Commander / Planeswalkers — mostly SHIPPED, index only
This entry claimed the engine had no command zone, no commander damage, no
emblems and no planeswalker attacks. All four exist: `Player::command` with
`command_zone_abilities_active()` and a `ClientView` field,
`commander_damage` with a per-source tally surfaced in the view (CR 903.10a)
and a regression test, `Player::emblems` joining the layer gather's anthem
walk, and `AttackTarget::Planeswalker` chosen by the bot's attack search
(`walker_chip`) and resolved by combat. **Still open, and that is all that is
left here:** four-player free-for-all match setup in `run_match` /
`build_cube_state`, colour-identity deck building and commander tax, the
"your opponents" vs "each other player" multiplayer targeting split, and
CR 118.3c planeswalker damage redirection.

### Saga Lore Counters
✅ Non-DFC Sagas ship via `CardDefinition.saga_chapters` + `saga_advance`
(ETB chapter I, +1 lore each precombat main, final-chapter sacrifice SBA).
History of Benalia, The Eldest Reborn. Remaining ⏳: DFC/transforming sagas
(The Everflowing Well saga-land) and read-ahead chapter-choice variants.

### Vehicle / Crew and divided damage — SHIPPED, index only
`GameAction::Crew` / `Saddle` are real actions with a bot picker
(`pick_crew_vehicle`), and divided combat damage is resolved by
`free_division_targets` + the CR 510.1c assignment path (Butcher Orgg's
`DividesCombatDamageAmongDefenders` included). **Still open:** a
`DealDamageDivided { total, targets }` *spell* effect — Pyrokinesis-style
"4 damage divided as you choose among any number of targets" is still
collapsed to a single-target hit.

### Affinity / Self-Permanent-Scaled Cost Reduction
Witherbloom, the Balancer's "Affinity for creatures (this spell costs
{1} less to cast for each creature you control)" needs a per-cast cost
reduction whose discount scales off the caster's permanent count.
`StaticEffect::CostReduction { filter, amount }` is a fixed amount
today. Generalising to `amount: Value::CountOf(Selector)` (or a sister
variant `AffinityCostReduction { filter, scaler: Selector }`) would
unlock Affinity for Artifacts (Modern Affinity / Cranial Plating-era
shells), Affinity for X (Strixhaven Witherbloom + future), and Awaken
the Woods-style "X = forests" payoff costs.

### Exile Zone as Viewable State
Exile is a zone in the engine (`Zone::Exile`) and cards move there.
`ClientView.exile` now projects the shared exile zone with each card's
owner so the UI can render an exile browser (added with the
Strixhaven coverage push). Remaining gaps:
- The 3D client has no exile browser UI yet.
- Graveyard-order information is lost (cards are a flat Vec).

---

## Discovered engine follow-ups (claude/modern_decks)

- **Noticed but not tackled this run:**
  - `Effect::ChooseUnchosenMode` auto-picks the first unused mode for bots and
    for a `wants_ui` seat alike (it uses the synchronous decider rather than a
    suspend). A human controller should get the real modal.
  - `apply_enters_under_opponent_control` picks the first alive opponent in seat
    order instead of asking; the printed text is "an opponent of your choice",
    which matters only in multiplayer.
  - `Selector::RandomAmong` re-rolls per resolution and can pick the source
    itself; a "chosen at random" that must exclude the source would need a
    filter-side `OtherThanSource` at the call site (Goblin Test Pilot doesn't).
- **Multi-block follow-ups — CLOSED.** Engine + client both ship (the
  order/assign modals are noun-aware via `damage_recipient_noun`, reading
  `PermanentView.attacking` / `.blocking_attackers`). CR 509.3a–e is now wired
  (see the CR audit). Umezawa's Jitte does *not* over-count: its
  `DealsCombatDamageToCreature` trigger isn't in the fan-out set, so it mints
  one instance per damage sub-step.
- **RNA/DGM cards deferred, each blocked on one primitive:**
  - **Domri, Chaos Bringer** — "+1: add {R} or {G}. If that mana is spent on a
    creature spell, it gains riot." Needs mana provenance (a rider attached to
    a specific mana unit, checked at the spell it pays for). Same blocker as
    the roadmap's "mana provenance" item.
  - **Captive Audience — SHIPPED** (`CardDefinition.enters_under_opponent_control`
    + `Effect::ChooseUnchosenMode` backed by `CardInstance.modes_chosen`).
  - **Theater of Horrors** — the exile half works with
    `ExileTopAndGrantMayPlay`, but "during your turn, if an opponent lost life
    this turn, you may play cards exiled with this" needs a CONDITION on
    `MayPlayPermission` (the struct is `Copy`, so it wants a small Copy-able
    gate enum rather than a `Predicate`).
  - **Melek, Izzet Paragon — SHIPPED** (`CardInstance.cast_from_library` +
    `Predicate::CastSpellFromLibrary`; the library-top cast hops through hand,
    so the origin rides `GameState.casting_from_library_top`).
  - **Goblin Test Pilot — SHIPPED** (`Selector::RandomAmong(filter)`).
  - **Plasm Capture — SHIPPED** (`Value::CounteredSpellManaValue` +
    `AddManaAtNextMainPhase { any_color }`); **Catch // Release — SHIPPED**
    (five-type edict off existing primitives). **Reap Intellect**,
    **Flesh // Blood**, and **Legion's Initiative** shipped too — DGM is
    complete.
- **`EffectDuration::UntilNextTurn` was never expired** — fixed; both it and
  `UntilYourNextTurn { player, installed_turn }` (CR 611.2b — Amplifire) now
  clear at the untap step of the turn they name. The 18 catalog sites were
  re-read against their oracle text: all of them print a real "until your next
  turn" clause, so none wanted the old permanence.
- **Erebos's Emissary (THS) — SHIPPED** (`Predicate::SourceIsBestowedAura`
  branches the pump between the source and its host).

- **RNA batch-7 leftovers (each needs one primitive):** Persistent Petitioners' "tap four untapped
  Advisors: mill 12" (a tap-N-other-of-a-type activation cost); Rakdos, the
  Showstopper (per-creature coin-flip destroy filtered by type). Opponent-threat
  displays in `player_stats.rs` still value a High Alert/Doran wall by power
  (0), not toughness — refine when convenient. (Pestilent Spirit's I/S-spell
  deathtouch shipped in batch 9 via `StaticEffect::YourISSpellsHaveDeathtouch`.)
- **RNA batch-9 deferrals — SHIPPED** (Galloping Lizrog remove-and-double,
  Combine Guildmage turn-scoped enters-with counter, Forbidding Spirit
  `TaxAttackersUntilYourNextTurn`, Font of Agonies blood counters +
  `EventKind::PaidLife` trigger, Verity Circle `EventSpec::not_as_attacker`,
  Angel of Grace `CantLoseThisTurn{damage_floor}` + gy-recur, Rhythm of the
  Wild riot anthem via `GrantTriggeredAbility`, Rumbling Ruin low-power
  can't-block). Still open: Ravager Wurm mode 2 — "destroy a land with a
  non-mana activated ability" (a land-with-nonmana-ability target filter);
- **Multi-block — SHIPPED.** `block_map` is now blocker → `Vec<attacker>` with
  `Keyword::CanBlockAdditional(n)` / `CanBlockAnyNumber`, blocker-side damage
  division (CR 510.1e), and a bot pass that spends spare block capacity. Still
  open on top of it: "blocks two or more creatures" batch counting (CR 509.3e),
  and the client has no UI yet for assigning a multi-blocker's damage split
  (the engine suspends correctly; the panel reuses the attacker-side modal).
- **New primitives that would unblock batches of gap cards (recent274–279 run):**
  - **Enlist** (CR 702.148) — no keyword yet; blocks the DMU Enlist commons
    (Barkweave Crusher, Coalition Warbrute, Argivian Cavalier, …). `Effect::Enlist`
    exists but no `Keyword::Enlist` + attack-time tap-a-nonattacker wiring.
  - **Backup N** (CR 702.164) — no keyword; blocks the MOM Backup commons
    (Chomping Kavu, Consuming Aetherborn, Cragsmasher Yeti, Archpriest of Shadows).
    Needs an ETB "put N +1/+1 counters on target; if another creature, it gains
    this creature's abilities until EOT" primitive.
  - **Player-curse Auras** — `PlayerStaticTarget::EnchantedPlayer` + a battlefield
    permanent→player attachment link so an Aura's static/trigger can scope to the
    enchanted player. Blocks Grievous Wound (can't-gain-life + damage→lose-half).
  - **Move a battlefield permanent to owner's library top/bottom (owner choice)** —
    `ZoneDest::OwnerLibraryTopOrBottom` is a countered-spell zone only; no
    permanent-move dest. Blocks Desynchronize, Diver Skaab's exploit rider.
  - **Edict-exile (target opponent exiles a permanent of a type, their choice)** —
    blocks Debt to the Kami's modal.
  - **"If you didn't put a card into your hand this way, gain N life"** — the
    inverse of `LookPickToHand.gain_life_if_pick`. Blocks Blossom Prancer.
  - **Blitz** field exists on `CardDefinition`; wire Caldaia Strongarm-style
    creatures (ETB counters + Blitz {cost}) once verified end-to-end.

- **Single-primitive cards scoped this run (each unblocks one card):**
  - Miasma Demon (DSK) — reflexive "discard any number; when you do, up to that
    many target creatures get -2/-2" (`Reflexive` + target count = cards
    discarded this way; the count-links-targets wiring is the gap).
  - Undead Sprinter (DSK) — conditional graveyard cast gated on "a non-Zombie
    creature died this turn" + enters-with-a-counter-if-cast-from-graveyard.
  - Tin Street Gossip (MKM) — `SpendRestriction::FaceDownOrTurnFaceUp` mana.
  - Public Thoroughfare (MKM) — "sacrifice unless you tap an untapped artifact
    or land" (tap-a-permanent as an alternative-to-sacrifice cost; convoke-kin).
  - Unyielding Gatekeeper (MKM) — turn-face-up exile branching on whether the
    caster controlled the exiled permanent (blink-or-give-opponent-a-token).

- **MKM Cases shipped (`decks::recent242`, 6 + Case File Auditor); remaining Cases
  need new primitives:** Case of the Gorgon's Kiss (solved = self-animates to a
  4/4 creature — needs a "this permanent becomes a creature" static, plus a
  "3+ creature cards to graveyards this turn" solve counter), Pilfered Proof
  (solved token-replacement adding a Clue), Locked Hothouse (extra-land static +
  play-from-top-of-library static), Ransacked Lab (solve = "4+ instant/sorcery
  spells cast this turn" — no I/S-specific per-turn count predicate yet), Stashed
  Skeleton (solve = "no suspected Skeletons you control" — `SelectionRequirement::
  IsSuspected` now ships, so only the per-controller solve counter remains),
  Burning Masks (solve = "3+ sources you controlled dealt damage this turn" —
  needs a distinct-damage-source-count tracker).
- **"Sacrificed an artifact this turn" — SHIPPED** (`recent248`):
  `Player.artifacts_sacrificed_this_turn` + `Predicate::SacrificedArtifactThisTurn`
  + `SelectionRequirement::ControllerSacrificedArtifactThisTurn` +
  `self_cost_reduction_if_sacrificed_artifact` power Suspicious Detonation and
  Furtive Courier's unblockable rider. Magnetic Snuffler still needs a
  "return an Equipment card from your graveyard to the battlefield attached to
  this creature" ETB effect (no reanimate-attached primitive yet); its
  "whenever you sacrifice an artifact → +1/+1" half is a
  `PermanentSacrificed`/`YourControl` trigger filtered to `R::Artifact`.
- **Cross-permanent death-stat triggers:** "whenever a creature dies, if its
  [power/toughness] was X" on a *different* permanent (Massacre Girl) reads the
  dying creature's death-time stat correctly through the trigger **filter** (the
  death snapshot backs `R::ToughnessAtMost`, etc.), but `Value::ToughnessOf(
  TriggerSource)` in the trigger **body** resolves empty (the LKI subject is only
  set for the dying creature's own die-triggers). Prefer filter-gating such cards
  until the resolving-LKI-subject plumbing covers cross-permanent watchers.
- **Collect evidence as an activated-ability cost:** ✅ shipped —
  `ActivatedAbility.collect_evidence_cost: Option<u32>`, pre-flight-gated on
  `graveyard_can_collect_evidence` and paid through the shared
  `collect_evidence_from_graveyard` exile path (emits
  `GameEvent::EvidenceCollected`). Forensic Researcher is fully modeled. Hedge
  Whisperer still blocked only on the "target land becomes a 5/5 *for as long as
  this creature remains tapped*" conditional land-animation duration (a
  source-tapped-gated continuous grant — no primitive yet).
- **MKM remaining gaps (~50 cards):** legends (Delney, Etrata, Teysa, Judith,
  Kaya PW, Tolsimir's Wolf-attack lure, …), the remaining split cards (Flotsam //
  Jetsam, Push // Pull, Hustle // Bustle, Fuss // Bother ✅, Cease // Desist ✅),
  Disguise/Cloak value (Coveted Falcon, Fugitive Codebreaker), the reanimators
  (Relive the Past, Anzrag's Rampage), Krenko's Buzzcrusher (per-player land
  destruction + fetch), Officious Interrogation (per-target cost + investigate X),
  and the remaining lands (Public Thoroughfare, Branch of Vitu-Ghazi).
  `scripts/set_gaps.py mkm` lists them. Notable primitives still blocking cards:
  - **Wolf-attack lure** (Tolsimir) — "target creature blocks *that Wolf* if
    able" needs a MustBlock variant pointing at the trigger source, not the
    ability source (`MustBlockSource` binds `ctx.source`).
  - **Reflexive gy-target return** (Blood Spatter Analysis) — "sacrifice this if
    5+ bloodstain; when you do, return target creature card from your graveyard"
    needs the return target chosen only when the sacrifice fires, not every death.
    Also needs a Bloodstain counter type + a "whenever one or more creatures die,
    mill + add a counter" trigger.
  - **Tenth District Hero** — first ability is ready (`collect_evidence_cost` +
    `BecomeCreature` sets 4/4 Detective + vigilance); second ability blocks on a
    rename + "Other creatures you control have indestructible" anthem granted by
    a self-becomes effect.
  - **Sudden Setback** — "put target spell or nonland permanent on library, owner
    chooses top/bottom" needs a spell-or-permanent target (the `Target` enum has
    no Spell variant) + a library-owner-choice move effect.
  - **Tin Street Gossip / Goblin Maskmaker** — restricted / discounted mana for
    face-down casts needs a face-down-spell spend restriction + cost reduction.

- **FDN/DSK gap cards shipped (`decks::recent202`–`recent205`, 20):** Rite of the
  Dragoncaller, Koma World-Eater, Niv-Mizzet Visionary, Perforating Artist, Kiora
  the Rising Tide, Soulstone Sanctuary, Lunar Insight, Valkyrie's Call, Infernal
  Vessel, Fiery Annihilation, Violent Urge, Elenda Saint of Dusk, Quilled
  Greatwurm, Saw, Unable to Scream, Sporogenic Infection, Under the Skin, Don't
  Make a Sound, Keys to the House, Osseous Sticktwister. Approximations left:
  Fiery Annihilation's exile-attached-Equipment rider, Quilled Greatwurm's
  graveyard-cast, Elenda's hexproof-from-instants, Sporogenic Infection's
  "other than enchanted" sacrifice clause, Don't Make a Sound's reflexive
  surveil-2, Keys to the House's Room lock/unlock mode. Remaining FDN/DSK gaps
  needing new primitives: Drake Hatcher / Nine-Lives Familiar (incubation /
  revival counter types), Banner of Kinship (choose-type + fellowship-counter
  anthem), Alesha (reanimate MV ≤ source power), Tinybones / Abyssal Harvester
  (stash / gy-exile copy), Kykar (modal cast trigger), Zimone (double each kind
  of counter on up-to-2 targets), Miasma Demon / Orphans of the Wheat (discard-
  any-number / tap-any-number variable counts), Creeping Peeper
  (enchantment-only spend restriction).
- 🟡 **Aristocrats self-death scope audit** — fixed Zulaport Cutthroat, Cruel
  Celebrant, Vengeful Bloodwitch (`AnotherOfYours`→`YourControl`; their oracle is
  "this *or* another creature you control dies", so their own death now drains).
  Both self-death funnels (the SBA lethal-damage `die_triggers` push **and** the
  destroy/sacrifice `remove_to_graveyard_with_triggers` path) now evaluate the
  trigger's `.with_filter` against the dying creature (bound as `TriggerSource`
  via the death snapshot), so a *filtered* `YourControl`/`AnyPlayer` "this or
  another [type] you control dies" trigger fires on self-death only when the
  source matches. Remaining (card work): sweep the ~49 `AnotherOfYours`
  CreatureDied cards and switch any whose oracle includes "this" to
  `YourControl` after verifying each against Scryfall.
- 🟡 **MH3 gaps still open** (`python3 scripts/set_gaps.py mh3`). Shipped since:
  the `{C}`-spent predicate (Drowner, Wumpus), Propagator Drone, Path of
  Annihilation, Deem Inferior, Snow-Covered Wastes, Imskir Iron-Eater
  (`Value::HalvedRoundDown`), Bespoke Battlewagon (energy Vehicle), Monstrous
  Vortex (`Effect::Discover`), Aether Revolt
  (`StaticEffect::NoncombatDamageToOpponentsBonus`), Idol of False Gods
  (`StaticEffect::SelfHasKeywordWhileCountersAtLeast`), Spymaster's Vault
  (targeted connive-X), Monumental Henge (dig-for-historic), Inventor's Axe
  (`CardDefinition.equip_energy_cost`), Emissary of Soulfire (exalted counters
  modeled as permanently-granted `exalted()` via `Effect::GrantTriggeredAbility`
  now honoring `Duration::Permanent`), Winter Moon
  (`StaticEffect::MaxOneNonbasicLandUntap`), Cursed Wombat
  (`StaticEffect::CounterAmplifierOncePerTurn` — once-per-turn per-permanent
  +1/+1 amplifier), Rush of Inspiration (energy modal DFC), Rosecot Knight
  (ETB dig for artifact/enchantment). **mh3d batch (20 cards) shipped:** Depth
  Defiler (`CastSpellWasKicked` choose-one/both), Expel the Unworthy
  (kicker-widens-target), Collective Resistance (mana-Escalate — fixed the
  `Escalate` cost overflow), Twisted Riddlekeeper + Herigast (Emerge, now used),
  Ugin's Binding, Abstruse Appropriation, Dog Umbra, Thief of Existence,
  Amphibian Downpour, Ondu Knotmaster // Throw a Line, Hydroelectric Specimen,
  Eladamri, Party Thrasher, Suppression Ray, Bloodsoaked Insight, Genku,
  Charitable Levy (`Predicate::SourceHasCountersAtLeast`), Emperor of Bones,
  Ripples of Undeath. **mh3e batch (12 cards, `sets::mh3e`, tests `tests/mh3e.rs`)
  shipped:** Vega (`SpellNotCastFromHand` trigger), Chthonian
  Nightmare (`ActivatedAbility.energy_x_cost` — pay X {E}, reanimate MV-X),
  Glimpse the Impossible (impulse-3 + per-card end-step Spawn), Argent Dais
  (`Predicate::AttackedWithCountAtLeast` + AnyPlayer attack observers), Lethal
  Throwdown (modal additional-sac + conditional draw), Jolted Awake
  (`Effect::PayEnergyValue`), Volatile Stormdrake (`Effect::PayEnergyOrElseValue`
  + ExchangeControl auto-target fix), Planar Genesis
  (`Effect::LookTopDeployLandOrHand`), Pyretic Rebirth (gy-return + MV burn),
  Reiterating Bolt (base bolt), Unstable Amulet (energy ETB + `SpellNotCastFromHand`
  ping + impulse), Izzet Generatorium (`StaticEffect::EnergyGainBonus` +
  `Player.energy_spent_this_turn`/`GameState::spend_energy` +
  `Predicate::EnergyPaidThisTurnAtLeast`). **Since:** Volatile Stormdrake now has
  `Keyword::HexproofFromAbilities` (CR 702.11d — opponents' abilities can't target
  it) and Reiterating Bolt has `Keyword::ReplicateEnergy(3)` (energy-paid Replicate,
  copy-per-payment). Still open, each needing one primitive:
  optional Exert + haste-if-spent-on-creature mana (Arena of Glory);
  alt-cost-by-energy permission (Primal Prayers); a "may reveal + else +1/+1
  counter" look-top rider (Rosecot Knight);
  two-independent-kickers (Wastescape Battlemage); the real Sundering Eruption //
  Volcanic Fissure (name collides with an existing fabricated `sundering_eruption`
  in `decks::modern` — replacing it means rewriting that card's two tests);
  sacrifice-count-driven search (The Hunger Tide Rises IV).
  **Other MH3 gaps worth doing next (existing-primitive-friendly):** Nissa's
  Pilgrimage (search-2-basics-split-to-bf+hand + spell-mastery-to-3 — needs a
  split-destination search), Powerbalance (opponent-cast → reveal-top free-cast
  if same MV), Baru, Wurmspeaker (Wurm anthem + cost-reduction-by-greatest-power),
  Shilgengar (Blood-sac engine + mass finality reanimate), Echoes of Eternity
  (colorless-trigger doubler + copy-colorless-spell-on-cast). Card-level
  approximations are noted on each mh3d/mh3e factory doc comment (Party Thrasher
  plays both exiled cards; Ripples has no {1}+3-life gate; Dog Umbra drops the
  opponent-control rider; Emperor drops the counter reanimation; Herigast drops
  the emerge-granting static; Pyretic Rebirth/Jolted Awake model "up to one"
  targets as required).

- ⏳ **Noticed this run (recent110/111 sweep):**
  - **Counter-placer attribution** still open (see All Will Be One entry) —
    `GameEvent::CounterAdded` has ~55 construction sites; a `placed_by`
    field is mechanical but wide.
  - **Skipped cards needing a primitive each:** Lightning Storm (any-player
    stack-only activated ability), Tibalt's Trickery (random 1–3 mill +
    exile-until-different-name free cast), Bottled Cloister (end-step hand
    exile / upkeep return), Cenn's Tactician (counter-gated multi-block),
    Nourishing Shoal (pitch-X alt cost reading the pitched card's MV),
    Prismatic Strands (prevent-by-color + tap-white-creature flashback
    cost), Abundance (draw-replacement dig), Experimental Frenzy
    (can't-play-from-hand static + top-of-library play), Mycosynth Lattice
    (all-colorless + spend-any halves). (Pili-Pala / Phyrexian Unlife /
    Salvage Titan / Qasali Ambusher shipped in `recent112` — {Q} costs via
    `ActivatedAbility.untap_self_cost`, `ControllerDoesntLoseFromLife`.)
  - **Approximations to revisit:** Tidebinder Mage's lock is a one-shot
    `SkipNextUntap` (printed: while you control it); Hypergenesis dumps all
    hand permanents at once (printed: alternating one-at-a-time loop);
    Molten Psyche's metalcraft burn reads the first opponent's draw count
    (exact in 1v1); Loaming Shaman shuffles the whole graveyard (printed:
    any number of target cards); Hurkyl's Recall bounces artifacts the
    target *controls* (printed: owns); Emrakul's cast-trigger mind-control
    turn unmodeled; Oath of Nissa's planeswalker any-color rider unmodeled;
    Balance auto-picks keeps (a wants_ui picker would be faithful).

- ✅ **Self-ETB trigger `EventSpec.filter` was dropped.** The inline
  spell-resolution path (`stack.rs`) collected `SelfSource` `EntersBattlefield`
  triggers by kind+scope only, discarding `event.filter`, so filtered self-ETB
  triggers (Corrupted, kicker/bargain-gated ETBs) fired unconditionally. Fixed:
  the collection now carries the filter and the execution loop re-evaluates it
  once the source is on the battlefield (CR 603.4), building a context that
  carries the cast-mode flags (`kicked`/`bargained`/`cast_from_hand`/mayhem) so
  cast-property intervening-ifs still read true. (Attack/etc. SelfSource triggers
  already went through the general dispatch, which evaluated filters.)

### Enchantress package follow-ups (recent114)
- **`EquipScale` breadth** — the P/T-per-count scale only counts the
  *controller's* battlefield and can't honor `OtherThanSource`, so "for each
  other enchantment on the battlefield" (Ancestral Mask) and "per card in your
  hand" (Empyrial Armor) aren't expressible. Add an `all_players` flag + a
  hand-count source, then wire those two Auras.
- **`ExtraManaKind::AnyColor`** — Fertile Ground / Market Festival / New
  Horizons want "add one/two mana of any color" on a triggered land-tap. Needs
  either a wildcard mana token or a player choice at the trigger; deferred.
- **Karmic Justice** — needs an event for "a spell/ability an opponent controls
  destroys a *noncreature* permanent you control" (destroyer + victim-type).
- **Aura re-attach riders** — Shielded by Faith / Ajani's Chosen's "attach to a
  creature that enters" clauses are dropped; want a `MayAttachOnCreatureEnters`.
- **Calix combat-copy** — the "copy a nonlegendary enchantment once per turn on
  combat damage" half is dropped; the constellation +1/+1 is modeled.

## Follow-ups noticed (not yet done)

- ⏳ **Noticed this run (recent264 MOM/BRO batch):**
  - **Tapped token creation** — `Effect::CreateToken` has no `enters_tapped`
    flag (only `CreateTokenCopyOf` does), so "create a *tapped* Powerstone"
    (Argothian Opportunist, Koilos Roc) and similar tapped-token cards can't be
    modeled faithfully. Add a `tapped` field to `Effect::CreateToken`.
  - **Three-way library split on look** — `Effect::LookPickToHand` bottoms OR
    graveyards the rest, not "one to hand, one to graveyard, one to bottom"
    (Moment of Truth). Wants a per-pile routing look effect.
- ⏳ **Noticed this run (recent80 primitive batch):**
  - **Champion** (`Effect::Champion`) auto-picks the lowest-power creature to
    exile; the printed "you may instead sacrifice this" decline + a `wants_ui`
    picker is a follow-up.
  - **Run Away Together** stays "any two creatures" — a `distinct_controllers`
    flag on `Effect::ApplyToTargets` (enforced at cast-time targeting) would make
    it and similar "different players" spells faithful. (Deferred: the flag would
    have to be threaded through all 84 `ApplyToTargets` construction sites.)
  - **Goblin Recruiter** "any number" is capped at 10 via `SearchUpToN`; a true
    unbounded search-to-top would need an "any number" search count.
- ⏳ **Noticed this run (recent84–89, chosen-type/tribal batches):**
  - **Herald's Horn upkeep reveal** — the "look at top card; if it's a chosen-type
    creature, may reveal it to hand" rider is dropped (cost-reduction half is
    faithful). Wants a "top-card-of-chosen-type" reveal effect.
  - **Still-missing tribal payoffs needing new primitives:** Brass Herald
    (ETB reveal-4, keep chosen-type creatures), Belbe's Portal (put a chosen-type
    creature from hand onto the battlefield), Kindred Charge (token-copy each of
    your chosen-type creatures), Shared Animosity (attack: +1/+0 per other
    attacker sharing a type — a per-attacker shared-type count), Mirror Entity
    (set your team's base P/T to X + grant all types), Kindred Summons / Kindred
    Dominance (cast-time creature-type choice on a spell with no permanent to
    stamp `chosen_creature_type`).
- ⏳ **Noticed this run (recent81–83 batches):**
  - **Auto-targeter ignores target slots embedded in a `Value`.** A trigger
    whose only target lives inside `Value::PowerOf(Selector::TargetFiltered{..})`
    (e.g. Wall of Reverence's "gain life equal to the power of target creature
    you control") isn't auto-targeted, so it resolves as 0. Wall of Reverence is
    modeled with `Value::GreatestPowerControlledMatching` to sidestep this;
    the general fix is to walk `Value` trees for target slots in the auto-target
    candidate scan. Would also make Ballista Squad's "attacking or blocking"
    restriction expressible once an `IsAttacking`/`IsBlocking` requirement exists.
  - **`AutoDecider` declines every `Effect::MayDo`/`OptionalTrigger`,** so
    pure-upside "you may draw / gain / untap" triggers must be modeled as direct
    effects to fire under the bot-less test decider (the *bot* decider already
    accepts beneficial ones via `optional_trigger_beneficial`). Snake Umbra /
    Curious Obsession / Renewed Faith / Fecundity all use direct effects for this
    reason. A test-friendly "accept clearly-beneficial MayDo" AutoDecider policy
    would let those cards keep the printed "may" without breaking tests.
  - ✅ **Chosen-type *event* predicate** — `Predicate::TriggerObjectIsChosenType`
    matches an event subject's creature types against the source's
    `chosen_creature_type` (Changeling satisfies any). Ships Vanquisher's Banner's
    cast-of-type draw (now faithful), Kindred Discovery (enters/attacks → draw),
    and Door of Destinies (`AnthemForChosenType.per_counter` counter-scaled
    anthem + cast-of-type charge counter). Herald's Horn's chosen-type upkeep
    reveal still wants a top-card-of-chosen-type check.

- ⏳ **Flash-loyalty client affordance.** Engine ships `CardDefinition.flash_loyalty`
  (CR 606.3b — The Wandering Emperor activates loyalty at instant speed the turn
  it enters). The client's loyalty-activation affordance should surface those
  abilities while the flash window is open (any priority), not only at sorcery
  speed. Engine + server (bot) paths are wired; only the client highlight is a
  follow-up.
- ⏳ **Prototype (CR 702.160) follow-ups.** The mechanic + 15 BRO cards ship
  (`CardDefinition.prototype` + `GameAction::CastPrototype`). Client click casts
  the prototype face only when the full cost is unaffordable; a modifier
  (Shift-click) to choose the prototype face when *both* are affordable is a
  follow-up. Deferred BRO prototype cards need primitives the engine still
  lacks: Hulking Metamorph (enter-as-copy with prototype P/T), Arcane Proxy
  (exile-and-cast I/S with MV ≤ power from gy), Woodcaller Automaton (untap +
  animate a land), Rootwire Amalgam (X/X token = 3× power), Forgefire/Warzone
  (perpetual). The bot always prefers the cheapest legal line; no value-eval of
  full-vs-prototype.
- ⏳ **Cast-time modal choice (CR 601.2b) for "choose two of four" cards.**
  `Effect::ChooseN` resolves the mode pick at *resolution* via the decider, so
  per-mode targets for an arbitrary pick can't be supplied at cast. The five STX
  guild Commands (Silverquill/Lorehold/Witherbloom/Quandrix/Prismari) therefore
  still resolve two fixed default modes. Real fix: choose modes during casting
  and gather each chosen mode's targets then (also unblocks Sublime Epiphany's
  mode-pick UI for arbitrary combinations). Oracle modes captured 2026-06-19.
- ⏳ **Conditional-keyword statics beyond P/T** — `PumpSelfIf.keywords` covers the
  self case (Bloodghast's opp-≤10 haste). A team/granted conditional-keyword
  static (e.g. "creatures you control gain X while …") would generalize it.
- ⏳ **CHK cards/primitives deferred:**
  - `Effect::ApplyToTargets` now does "do X to each of up to N targets" — Yosei's
    "tap up to five target permanents that player controls" could be remodeled
    on it (filter `ControlledBy(targetPlayer)`), as could other "up to N" cards
    across sets (Frost Breath, Aether Tradewinds-style multi-bounce, etc.).
  - Pious Kitsune / Eight-and-a-Half-Tails devotion-counter conditional payoff.
  - Yosei taps **up to five** target permanents (modeled as tapping all of the
    target player's board); a true "up to N target permanents that player
    controls" clause needs the Tier-2 "up to N targets" work.
  - Sosuke's Warrior-damage destroy is **immediate** (printed "at end of
    combat"); wants a delayed end-of-combat destroy trigger.
  - Genju aura cycle (animate-a-land aura that returns to hand when the
    creature dies), Honden cycle's "Pious Kitsune / Eight-and-a-Half-Tails"
    devotion-counter conditional.
  - Kamigawa cards skipped this run for want of a primitive:
    Sokenzan Renegade / Kiyomaro
    (hand-size-gated keyword grants + "player with most cards" predicate);
    Takeno, Samurai General (anthem scaled by each Samurai's bushido total);
    Sachi, Daughter of Seshiro (granting "Shamans you control have {T}: Add
    {G}{G}" — group-granted mana ability).
  - Generalize "target player discards" auto-targeting so an ETB
    `Discard { who: Player(Target(0)) }` picks an opponent (Kemuri-Onna is
    modeled as `EachOpponent` to sidestep this).
  - Cranial Extraction (name a card → exile all copies from gy/hand/library);
    Cut the Tethers (per-Spirit
    "return unless pay {3}"); Petals of Insight (look-3, bottom-or-draw with
    conditional self-return); Devouring Greed / Devouring Rage (additional-cost
    "sacrifice any number of Spirits" that scales the spell — needs cast-time
    variable sac feeding `Value`).
  - Generalize `Player.zuberas_died_this_turn` into a type-filtered
    died-this-turn count if another tribe ever needs it.
- 🟡 **Bot: general value-activated-ability generator.** `pick_removal_ping`
  fires single-target "{cost}: deal damage to any target" abilities that kill
  an opposing creature outright (constant amount, or Kiku's
  damage-equal-to-its-power shape); `pick_removal_sacrifice` activates
  "Sacrifice this: destroy target creature" on favorable/even trades (Pus
  Kami). Remaining: X-value selection for scalable pings, and pointing a ping
  at the opponent's face for reach.
- ⏳ **THB cards still missing (need new primitives):**
  - **Aura-host-death trigger** (an Aura/enchantment-creature that triggers
    when its enchanted creature dies — there's no `EventScope::EnchantedBy`
    yet): Minion's Return (dies → return under your control), Dawn Evangel,
    Bronzehide Lion (dies → returns as an Aura), Hateful Eidolon (draw per
    Aura that was on it). LKI for the auras attached at death is the hard part.
  - **Aura-attach event** ("whenever an Aura you control becomes attached to a
    creature you control, …"): Siona's token half (Siona's ETB look-for-Aura
    *does* ship).
  - **Per-permanent ward-tax static** ("spells opponents cast targeting this
    cost {1} more" — `extra_cost_for_spell` can't see the cast's target yet):
    Callaphe's static half (its devotion power *does* ship).
  - **Pile-split decision** (Fact-or-Fiction style): Atris, Oracle of
    Half-Truths.
  - **Random choose + protection-from-mana-value**: Haktos the Unscarred.
  - **Continuous combat-damage-to-self replacement → counter**: Ironscale Hydra.
  - **Reveal-until-permanent → battlefield** end-step engine: Dreamshaper Shaman.
  - Aura-reanimation with exile-at-EOT (Storm Herald); reveal-6 opponent-exile
    (Allure of the Unknown); counter-and-Nevermore (Ashiok's Erasure);
    untap-lock tapper (Entrancing Lyre); combat-damage-prevention-except-
    enchanted fog (Inspire Awe); Medomai's Prophecy saga (chapter III delayed
    "first cast of named spell" trigger).
  - Heliod's Punishment ships without its task-counter self-removal timer (the
    lock is modeled as permanent).
- ⏳ **Tainted Pact UI**: the per-iteration "keep digging?" decision isn't
  wired for `wants_ui` players (AutoDecider takes the first card; a client
  modal + suspend/resume loop is the follow-up).

- ⏳ **Noticed this run (recent4 staples batch):** real gaps left for a
  follow-up, each needing a small new primitive:
  - **Smokestack / Tangle Wire** — "at each player's upkeep, sacrifice/tap N =
    counters on this" wants a counter-scaled per-upkeep cost (`Value::SourceCounters`
    over an active-player-only sacrifice/tap). Fading (CR 702.32) for Tangle Wire.
  - **Sanctum Prelate / Notion Thief / Hullbreacher** — chosen-number can't-cast
    gate (like Chalice) for Prelate; opponent-draw → you-draw / Treasure
    replacement (CR 121 / 614) for Thief/Hullbreacher.
  - **Outpost Siege** — Khans/Dragons mode-on-ETB + the two ongoing effects.
  - **Figure of Destiny** — activated set-base-P/T + add-creature-types gated on
    current type (leveler-adjacent, but conditional).
  - **Ancient Excavation / Insidious Dreams** — `Value::CardsInYourHand` and an
    additional-cost "discard X" with an X-bounded library search.
  - **It That Betrays** — "whenever an opponent sacrifices a nontoken permanent,
    put it onto the battlefield under your control" replacement.
- ⏳ **Noticed this run (modern_decks Kamigawa/Channel batch):**
  - **Ghost-Lit Drifter** deferred — its Channel grants flying to *X* target
    creatures, but `Effect::ApplyToTargets.max_targets` is a fixed `u8`, not a
    cast-time `Value`. A `Value`-bounded "up to N targets" would unblock it
    (and tighten Yosei's "up to five permanents that player controls").
  - **Kitsune Palliator** deferred — "{T}: prevent the next 1 damage to *each*
    creature and *each* player" needs a mass prevention-shield install
    (`PreventNextDamage` is single-target today).
  - **Ravenous (CR 702.156)** models the "draw if X≥5" clause off the resulting
    +1/+1 counter count; a counter-doubler would shift the threshold vs. printed
    X. A permanent-remembers-cast-X field would make it exact.

- ⏳ **Noticed this run (recent5 staples batch):** approximations left for a
  follow-up, each needing a small primitive:
  - **Plaguecrafter** drops the "each player who can't sacrifice, discards"
    rider (no sacrifice-or-discard fallback primitive).
  - **Misdirection** drops the printed "spell with a *single* target"
    restriction; **Venser** / **Hullbreaker Horror** model "target spell or
    permanent" as permanent-only (no bounce-a-spell-off-the-stack effect), and
    Hullbreaker drops its "up to one" mode choice.
  - **Skrelv, Defector Mite**'s grant is simplified to hexproof (no
    toxic-grant + unblockable-by-chosen-color + color choice).
  - **Flawless Maneuver** drops the free-if-you-control-a-commander alt cost
    (no `IsCommander` selector for `AlternativeCost.condition`).
  - **Neoform** counters every creature that entered this turn (no `Selector`
    for the just-searched permanent); exact only on a clean cast.
  - **Guardian Project** drops the same-name exclusion (no unique-name
    predicate).
  - **Deferred (not implemented):** Carpet of Flowers (once-per-turn main-phase
    "add X mana of one color = opp Islands"), Cultivator Colossus (etb
    put-land/draw loop), Plague Engineer (chosen-type opponents'-creatures
    -1/-1 static), Mystic Sanctuary (enters-tapped-unless-N-Islands +
    entered-untapped trigger), Wrenn and Seven, Reidane, Malevolent Hermit,
    Old-Growth Troll, Tarmogoyf Nest, Agadeem's Awakening, Joraga Treespeaker
    (LevelBand can't grant the `{T}: add {G}{G}` / Elf-lord ability — needs
    ability-granting level bands).

- ⏳ **MH3 batch shipped** (`catalog::sets::mh3`, tests `tests/mh3.rs`, 36
  cards): energy (Solstice Zealot, Tempest Harvester, Roil Cartographer,
  Solar Transformer, Phyrexian Ironworks, Hexgold Slith, Thriving Skyclaw,
  Conduit Goblin, Smelted Chargebug, Inspired Inventor), devoid/Eldrazi
  (Fanged Flames, Snapping Voidcraw, Unfathomable Truths, Titans' Vanguard,
  Skittering Precursor), plus Accursed Marauder, Faithful Watchdog, Wing It,
  Gift of the Viper, Mogg Mob, Retrofitted Transmogrant, Consuming Corruption
  (`Value::ColorCountOf` powers Breathe Your Last), Fowl Strike (Reinforce),
  Aerie Auxiliary, Scurrilous Sentry, Wither and Bloom, Fetid Gargantua,
  Dreadmobile (Vehicle/Crew), Proud Pack-Rhino, Warren Soultrader,
  Horrid Shadowspinner, Sarpadian Simulacrum, Serum Visionary, Nightshade
  Dryad, Null Elemental Blast. **Deferred — each wants one primitive:**
  Modular N / Fabricate N keywords (Arcbound Condor, Marionette Apprentice);
  exalted counter type (Emissary of Soulfire); colorless-or-abilities spend
  restriction (Sage of the Unknowable); continuous base-P/T anthem static
  (Kudo, King Among Bears); put-N-from-hand-on-top (Brainsurge); untap-count
  restriction static (Winter Moon); countered-spell-controller token mint
  (Strix Serenade); cast-or-cycle trigger (Drownyard Lurker). Also: the
  triggered-modal `AddCounter(Shield)` isn't auto-targeted (preference fn
  only auto-picks +1/+1) — a UI seat must pick the target.

- ⏳ **Noticed this run (prowl / faeries / triggered-mana batch):**
  - **AutoDecider declines all `SearchLibrary` picks** (`Search(None)`) — a
    bot heuristic that takes the first eligible candidate would make
    fetch/tutor effects function under bots; many tests assume the decline,
    so flip carefully.
  - **`EventSpec::per_subject_cap` is per-turn**, so Spined Sliver won't
    re-trigger in a second combat phase the same turn.
  - **`ExtraManaOnLandTap` Mirror** mirrors the *first* produced pip; the
    printed Mana Flare lets the tapping player choose among produced types
    (matters only for multi-type productions).
  - **Notorious Throng X** uses `LifeLostThisTurn(EachOpponent)` (a max) —
    exact in 2P; multiplayer wants a damage-dealt-to-opponents sum.

- ⏳ **Noticed this run (gods / rope / split-second batch):** Rope client
  UI ✅ (`ServerMsg::Rope` + countdown banner), Nylea's may-bin reveal ✅,
  `AutoDecider` empty-`ChooseTarget` fallback ✅. Remaining:
  - **`Selector::LastCreatedTokens` + `GrantKeyword`** (Sokenzan) grants
    haste only to tokens minted in the same resolution — fine today; a
    "they gain haste" rider on `CreateToken` would be tidier.

- ⏳ **Noticed this run (slivers / seat-routed asks batch):**
  - **TemptingOffer ordering** — opponents now answer before the body
    runs (re-run idempotency); printed timing shows them the
    controller's result first.
  - **Statics-granted triggers from died-LKI snapshots** — the
    died-card Enrage walk only reads printed triggers; a granted
    "when this is dealt damage" wouldn't fire on lethal damage.
  - **Answer-log nesting** — `ask_seat_bool` users can't nest another
    log-using effect inside their own ask sequence (single shared log).

- ⏳ **Noticed (Modern staples batch, 2026-06-11):** 38 staples shipped
  across three waves (see git). The "deferred, each wanting one primitive"
  list is now almost fully shipped: Conspicuous Snoop ✅
  (`HasActivatedAbilitiesOfLibraryTop` + Goblin `PlayFromLibraryTop`),
  Alpine Moon ✅ (`NamedLandsNeutralized` + `NamedBySource` ability grant),
  Bring to Light ✅ (`ManaValueAtMostConverged` resolved in `Search`),
  Ad Nauseam ✅ (`RevealTopToHandLoseLifeRepeat`), Kataki ✅
  (`StaticEffect::GrantTriggeredAbility` — statics-granted triggers in both
  dispatchers), Porphyry Nodes ✅ (`Selector::LeastPowerAmongAll`),
  Shield of the Oversoul / Steel of the Godhead ✅
  (`EquipBonus.conditional`), Ravenous Trap ✅
  (`Player.cards_to_graveyard_this_turn` +
  `Predicate::CardsToGraveyardThisTurnAtLeast`), Spellskite ✅. Remaining:
  - **Witchbane Orb** ships without the destroy-Curses ETB (player-attached
    Curses unmodeled). **Counterbalance**'s reveal is a MayDo (bots decline
    by default).
  - **Lightning Storm** — any-player stack activations (the "discard a land,
    choose new targets" response loop).

- ⏳ **Noticed this run (THB / splice / split-picker pass, 2026-06-12):**
  - **Splice UI/bot** — `CastSpellSpliced` is engine-only; the client has no
    splice picker and the bot never splices.
  - **Callaphe, Beloved of the Sea** — wants a "spells your opponents cast
    that target [your permanents] cost {1} more" static
    (`extra_cost_for_spell` doesn't see the cast's target today).
  - **Calix, Destiny's Hand** — -3 wants `ExileUntilSourceLeaves` anchored to
    a *chosen* permanent rather than the effect source.
  - **Hateful Eidolon / Bronzehide Lion** — die-with-attached-Aura LKI count
    and a dies→returns-as-Aura transform are both unmodeled.
  - **Tectonic Giant** mode 1 grants may-play on both impulsed cards (the
    printed "choose one of them" pick is dropped); the grant bills MV-generic
    rather than the card's real cost.
  - **Fused split casts with targets** — the client's half-picker greys the
    Fused button when either half targets (the targeting cursor collects one
    target; fused needs left + right slots).

- ⏳ **Noticed this run (ZNR MDFC + hexproof-from-color batch):**
  - **Dropped riders on shipped ZNR cards:** Hagra Mauling's "{1} less if an
    opponent controls no basic lands" cost reduction; Turntimber Symbiosis's
    "+3 counters if the deployed creature's MV ≤ 3" (the `LookPickToHand
    { to_battlefield }` primitive can't condition counters on the pick).
  - **ZNR cards still unimplemented** (each wants a new primitive):
    Valakut Awakening (put any number from hand on bottom, then draw that
    many +1 — no bottom-then-draw effect); Agadeem's Awakening (mass-reanimate
    any number of *distinct-MV* creatures ≤ X — no different-MV multi-target
    reanimation); Sea Gate Stormcaller (copy-your-next-cheap-I/S delayed
    trigger). Sporeweb Weaver / Garruk's Harbinger want a general
    "when this is dealt damage" trigger (non-combat enrage) + a combat-damage
    library-look.
- ⏳ **Noticed this run (claude/modern_decks, 2026-06-11 second pass):**
  `UnlessPlayerPays` per-seat routing ✅ (rhystic/Kataki taxes now prompt
  the taxed `wants_ui` seat via `ask_seat_bool`). Remaining:
  - **`RevealTopToHandLoseLifeRepeat` + Seek library pick** still answer
    through the single global decider (non-Bool decisions; the
    `ask_seat_bool` replay-log only covers yes/no questions).
    Kataki under AutoDecider still declines → bots sacrifice their
    artifacts even with open mana (needs a bot heuristic, not routing).
  - **`SacrificeOrPay` chooser** — the auto rule (sacrifice when a match
    exists, else fold the pay into the cost) is deterministic; a wants_ui
    "which half?" picker would make Bayou Groff interactive.

- ⏳ **Noticed this run (follow-ups sweep):** `Effect::MayPay` wants_ui
  suspend ✅ (seat-routed). Remaining:
  - **Nadu's granted ability** is modeled as a trigger on Nadu itself with
    a per-subject cap (behaviorally equivalent); a true "creatures you
    control have [triggered ability]" static grant framework is still open
    (matters for ability-reading effects).
  - **Karplusan Minotaur's lose-a-flip ping** lets the controller aim the
    damage; printed text has an opponent choose the target.
  - **`EventSpec::per_subject_cap`** only counts permanent subjects; a
    player-subject cap would need an EntityRef-keyed map.
- ⏳ **Noticed (modern_decks batches 4-6):** all the listed cards shipped
  (Nadu / Six / Ajani MDFC / Kozilek / Ulamog the Defiler / Springheart /
  Not Dead After All / Indomitable Creativity — each with its primitive).
  Remaining: none — Clash (CR 701.30) now prompts each `wants_ui` seat
  to bottom or keep via the seat-routed answer log.

- ⏳ **Noticed (staples expansion / audit):** The Ozolith, Soulless Jailer,
  Underworld Breach, Karn the Great Creator, and Sunken Citadel all shipped
  with their primitives. Remaining:
  - **Ulamog, the Ceaseless Hunger** cast trigger is modeled as two
    single-target exile triggers (multi-target triggers still unsupported —
    see the existing multi-target ETB note).
  - **Madcap Experiment** bills its reveal count as life loss rather than
    damage (`RevealUntilFind.life_per_revealed`); a damage rider would be
    more faithful vs prevention effects.

- ⏳ **Noticed this run (multikicker / mill batch):**
  - **MayDo wants_ui suspend** ✅ — `Effect::MayDo` now suspends for a
    `wants_ui` controller via the stash-and-rerun path
    (`PendingEffectState::MayDoAnswerPending`); the client's existing
    OptionalTrigger yes/no modal answers it. Bots/tests still use the
    synchronous decider.
  - **Squad/Replicate/Multikicker stepper cap** — the bot probes kick counts
    1–4; an exact max-affordable computation would kick higher with big pools.

- ✅ **Staple/mill/landfall follow-up batch — all eight shipped:**
  - **Everflowing Chalice** ✅ — `Keyword::Multikicker` (CR 702.33c) +
    `GameAction::CastSpellMultikicked { times }` + `CardInstance.kick_count`
    read by `Value::TimesKicked`; client pay-times stepper generalized to
    Squad/Replicate/Multikicker (`PayTimesMechanic`). Hangarback's cast-X →
    ETB counters already worked (x_value threads into the ETB ctx).
  - **Archive Trap** ✅ — `Player.searched_library_this_turn` (stamped at the
    Search funnels, reset each turn) + `Predicate::SearchedLibraryThisTurn`
    gating the `AlternativeCost.condition` free cast.
  - **Dauthi Voidwalker** ✅ — `ExileCardsBoundForGraveyard.void_counter`
    stamps `CounterType::Void`; the sac ability rides `GrantMayPlay` over
    `InExile + WithCounter(Void)`.
  - **Chandra, Torch of Defiance** ✅ — `ExileTopAndGrantMayPlay.uncast_penalty`
    registers a next-end-step still-`InExile` check that runs the fallback.
  - **Scrap Trawler** ✅ — `SelectionRequirement::ManaValueLessThanEventAmount`;
    died events now carry the dying card's MV (`event_amount_for`) into
    `trigger_event_amount_scratch`.
  - **Torbran, Thane of Red Fell** ✅ — `StaticEffect::AddDamageToOpponents`;
    `scale_damage_to` is source-aware (`resolving_source` carries in-flight
    spell color/controller).
  - **Conflagrate** ✅ — `AdditionalCastCost::DiscardXFromCost` takes the
    cast's X (Flashback—discard X cards).
  - **Urza's Saga** ✅ — `Effect::GainActivatedAbility` →
    `CardInstance.granted_activated_abilities` (cleared on leave, CR 400.7);
    saga lands advance on the land drop (`place_land_card`).


- ⏳ **Noticed this run (claude/modern_decks):**
  - **Room rules corners** — lock-a-door effects (709.5g), "fully unlock"
    triggers (709.5i), and combined MV in non-stack zones (709.4b) are not
    modeled; door casts also skip the convoke/delve/alt-cost riders.

- ✅ **This batch shipped** (was the "deferred, each wants one primitive"
  list): DFC sagas (`Effect::ExileSelfReturnTransformed` — Fable of the
  Mirror-Breaker), search statics (`OpponentsSearchTopN` / `SearchTax` —
  Aven Mindcensor, Leonin Arbiter), end the turn (CR 728 —
  `Effect::EndTheTurn`; Sundial, Day's Undoing), color-filtered gy-hate
  (`ExileCardsBoundForGraveyard.colors` — Sanctifier en-Vec), activation
  tax (`StaticEffect::ActivationTax` — Suppression Field), Reckoner
  Bankbuster (charge-empty payout via `remove_counter_cost` + If).
- ⏳ **Still deferred:**
  - **Exalted Angel's printed trigger** is modeled as Lifelink (gains on
    any damage it deals — equivalent in practice).
  - **Eon Hub vs. suspend/pacts**: skipped upkeeps also skip suspend ticks
    and pact payments — correct per CR 614.10b, but worth a regression test
    when pact decks meet Eon Hub.
- ⏳ **Tempting offer / opponent-may wants_ui suspend** —
  `Effect::TemptingOffer` and the new `Effect::PlayersMayAccept` (Vexing
  Devil, Browbeat, Risk Factor) ask via the synchronous decider; a
  networked human seat gets the AutoDecider default (decline). Same family
  as the existing inline-picker gaps.

- ✅ **Cipher follow-ups.** Hidden Strings, Rubblehulk, and Trait Doctoring
  (CR 612 layer-3 text change) all ship.
- ✅ **Continuous "becomes a copy" (CR 707.2)** — `Effect::BecomeCopyOfFor`
  swaps the definition with a scheduled revert (`GameState.temporary_copies`,
  the Act-of-Treason plumbing pattern): reverts at duration end and on
  battlefield-leave; `non_legendary` strips Legendary (707.2e). Ships Echoing
  Equation, Vesuva, Thespian's Stage. Remaining ⏳: "while attached" aura
  copies (Mirrorform) want a WhileSourceOnBattlefield-style duration tied to
  the aura.
- ⏳ **MKM Disguise riders dropped this run (each wants one small primitive).**
  - Experiment Twelve / Pyrotechnic Performer — "or another creature you control
    is turned face up" collapses to a SelfSource-only trigger (no per-creature
    turned-up binding for other permanents).
  - Deferred (need new primitives): Coveted Falcon (control-swap + draw-per),
    Aurelia's Vindicator (X-cost Disguise + exile-up-to-X + return-on-leave),
    Concert Kaboomist (noncreature-spells-since-last-turn count), Boltbender
    (choose new targets), Polygraph Orb (collect evidence).
- ⏳ **Face-down follow-ups (this run shipped manifest + the 2/2 object).**
  - **Morph cast-face-down spell path** (CR 702.36): a `GameAction::CastFaceDown`
    that pays {3} and casts the card as a face-down 2/2 creature spell, reusing
    the new `CardInstance.face_up_def` swap + `turn_face_up_action`. No catalog
    Morph cards yet, so deferred.
  - Disguise (CR 702.166) ✅ (`Keyword::Disguise` + `facedown_disguise_definition`)
    and Cloak (CR 702.182) ✅ (`Effect::Cloak` + serialized `CardInstance.cloaked`).
    Follow-up ⏳: Hide in Plain Sight's full "look at top five, cloak two, rest to
    bottom random" selection is simplified to cloaking the top two.
  - **Manifest-dread "turn up if a creature card"** already works via
    `TurnFaceUp`; a face-down noncreature can't be turned up (correct).
- ⏳ **Cards deferred this run (each wants one small primitive):**
- 🟡 **Resolution-time target legality (CR 608.2b).** General now: every
  single-target spell whose primary target was a *battlefield permanent at
  cast time* (`CardInstance.cast_target_was_battlefield`, stamped in
  `finalize_cast`) fizzles on resolution if the target left the battlefield,
  stopped matching the (mode/kicker-aware) filter, or gained Hexproof/Shroud;
  a fizzled real card is countered into its owner's graveyard. Token copies
  keep the bare filter re-check. **Multi-target all-illegal fizzle ✅** —
  battlefield-aimed multi-target spells fizzle only when every slot is
  illegal (Arc Trail tests). Remaining ⏳: Aura spells (permanent path) and
  protection-from-color on resolution. (Audit follow-up closed — triggered
  abilities fizzle per CR 608.2b and flashbacked fizzles route to exile.)
- ⏳ **Demonstrate "you may" + opponent choice (CR 702.150).** `Effect::
  Demonstrate` always copies (the optional "you may" collapses) and auto-picks
  the lowest-seat opponent rather than prompting the caster. Fine for bots;
  a `wants_ui` caster should get a yes/no + opponent picker.
- ⏳ **Impending / Hideaway follow-ups (this run shipped the keywords).**
  - Hideaway (CR 702.76, `Effect::Hideaway`): the hidden-card pick auto-resolves
    to the highest-MV card rather than prompting. The Lorwyn land cycle ✅ —
    Mosswort Bridge / Spinerock Knoll / Windbrisk Heights ship with their
    printed gates (`Value::PowerOf` fan-out, `Value::LifeLostThisTurn`,
    `Value::CreaturesAttackedWithThisTurn`).
- ⏳ **Card riders dropped (each wants one small primitive):**
  Glissa Sunslayer ✅ (full combat-damage `ChooseMode` — draw/lose, destroy
  enchantment, remove-all-counters); Bristly Bill ✅; Nowhere to Run ✅;
  Get Lost / Sip of Hemlock use the destroyed permanent's *owner* for the
  follow-up (differs from "controller" only under control-stealing).

- ⏳ **Cube bombs still needing primitives.** Skyclave Apparition ✅,
  Grafdigger's Cage ✅ (`StaticEffect::GraveyardLibraryLockdown` — gates
  flashback/escape/Muldrotha/library-top/free-casts and gy/library →
  battlefield creature entries; search-to-battlefield pending states don't
  consult it yet), Hostage Taker ✅ + Gonti ✅ (paid casts from exile via
  `GrantMayPlay { pay_own_cost }` / `LookTopExileOneMayPlay` + the
  `WhileExiled` may-play duration — the any-color spend clause is still
  dropped). Remaining: Duplicant (imprint + P/T-from-exiled CDA).
- ⏳ **`EachOpponentPlaneswalker` was unneeded** — Saheeli's "each planeswalker
  they control" rides `EachPermanent(Planeswalker & ControlledByOpponent)` with
  damage-to-PW (CR 120.3c). Karn Liberated's -14 and Ugin's -X exile-by-MV
  still approximate (no X-aware `ManaValueAtMostX` requirement yet).
- ⏳ **Dedicated immediate-blink primitive.** Restoration-style instant flicker
  is carded via `Exile { target } + Move { Target → Battlefield }` (Restoration
  Angel, Felidar Guardian). A single `Effect::FlickerImmediate { what }` would be
  cleaner (one trigger, no two-step target capture) but isn't required.
- ⏳ **Cast-from-exile (any color) rider on linked exile.** `ExileUntilSourceLeaves`
  has no may-play grant, so Hostage Taker ("exile … you may cast it, any mana
  type") and similar can only ship the exile half. Pair the linked-exile with a
  grant-may-play-from-exile + any-color spend permission.
- ✅ **Tap-N activation cost.** `ActivatedAbility.tap_n_filter` taps N matching
  untapped permanents (source eligible) as a cost — Heritage Druid. (An "X can't
  be blocked this turn" grant for Whirler Rogue-style payoffs is still ⏳.)
- ⏳ **Multi-target ETB / triggered abilities.** `StackItem::Trigger` carries a
  single `target`, so a triggered ability needing *two* targets (Vedalken
  Plotter's "exchange control of target land you control and target land an
  opponent controls") can't be auto-targeted for both slots. Spells already
  thread `additional_targets`; triggers need the same. (Switcheroo, a sorcery,
  exercises `Effect::ExchangeControl` cleanly meanwhile.)

- ✅ **Chosen-creature-type anthem static.** `StaticEffect::AnthemForChosenType
  { power, toughness, exclude_source }` reads the source's live
  `chosen_creature_type` (set at ETB via `Effect::NameCreatureType`) and emits a
  layer-7 pump over the controller's matching creatures in
  `gather_continuous_effects`. Ships Adaptive Automaton (`exclude_source`) and
  Patchwork Banner. Remaining: Metallic Mimic's enters-with-a-counter rider (a
  chosen-type ETB-counter replacement, not an anthem) and the "this is the
  chosen type in addition to its other types" self-type-add layer-4 effect.
- ✅ **Exile-self activation cost (graveyard + battlefield).** The gy/hand path
  (Stone Docent / Eternal Student) powers Daring Fiendbonder; `exile_self_cost`
  now also fires for a *battlefield* source via `move_card_to(.., Exile)` in
  `activate_ability` (Hanged Executioner's "{3}{W}, Exile this: exile target
  creature"). Daring Waverider's ETB cast-from-graveyard is a separate
  primitive (cast-IS-from-gy-for-free) still ⏳.
- ⏳ **Bloomburrow follow-ups (noticed this run):**
  - ✅ **Gift** (CR 702.165) ships (`CardDefinition.gift` + `GameAction::CastGift`
    + `CardInstance.gift_promised`; `TokenDefinition.tapped`; client right-click
    promise + `KnownCard.{has_gift,gift_label,gift_needs_target}`). Batch in
    `decks::gift` + Nocturnal Hunger upgraded. Remaining gift cards need new
    primitives: Coiling Rebirth (reanimate + 1/1 token-copy), Mind Spiral
    (draw-N + tap/stun), Pool Resources / Sazacap's Brew (Seek), Cruelclaw's Heist
    (exile-and-may-cast), Perch Protection (gift an extra turn). Also: the
    client's legal-target highlight for a promised gift still derives from the
    *base* effect, so a broadened gift target (Flood Maw's noncreature) isn't
    highlighted though the server accepts it.
  - ✅ **Survival** (CR 702.180) ships ("at your second main, if tapped …" —
    `StepBegins(PostCombatMain)`/`ActivePlayer` + tapped intervening-`if`;
    `decks::survival`). Remaining Survivors need primitives: Kona (put a
    permanent from hand onto the battlefield), Wary Zone Guard (enters tapped +
    perpetual +1/+1), Improvising Aerialist (perpetual flying), Veteran Survivor
    (exile-with-source count static), Rip / Effie (reveal-N-distinct-powers, seek).
  - **Expend** (CR 700.14) ships (`mana_spent_on_spells_this_turn` +
    `EventKind::Expend` + `Predicate::ExpendReached`; Roughshod Duo). Remaining:
    a `Value::ManaSpentOnSpellsThisTurn` reader for "expend 8" payoffs that
    scale, and bot awareness of expend thresholds when sequencing spells.
  - **Equipment tokens** ship via `TokenDefinition.equipped_bonus` (Mabel's
    Cragflame). Remaining: token Equipment whose equip cost or granted abilities
    aren't expressible as a flat `EquipBonus` (e.g. activated-ability grants).
  - **Pawpatch Recruit** "whenever another creature you control becomes the
    target of an opponent's spell/ability, +1/+1 on a different creature" —
    needs the `YourPermanentTargetedByOpponent` scope wired to a +1/+1-on-another
    body (the engine has the scope; the "other than that creature" target
    constraint is the gap).
- ⏳ **Bargain / Eldraine follow-ups (this run):**
  - ✅ Heartfire Hero **Valiant** — rides `BecameTarget + YourControl` +
    `once_per_turn` (CR 603.3d). Pawpatch Recruit's "another creature you
    control becomes targeted by an opponent" variant still ⏳.
  - **Gift** (Wilds of Eldraine; Sazacap's Brew, Coiling Rebirth) — promise an
    opponent a gift as an optional rider.
  - The bot never pays Bargain (always casts the base spell); a client
    "sacrifice for Bargain?" picker + bot fodder-choice are both unwired —
    `PlayerView.bargainable_hand` is surfaced but unused by the UI.
- ⏳ **Transform-DFC batch — dropped riders to revisit:**
  - ✅ Vildin-Pack Alpha's "when a Werewolf you control enters, you may
    transform it" (MayDo + `Transform { TriggerSource }`); ✅ Frenzied
    Trapbreaker's on-attack "destroy target artifact/enchantment defending
    player controls". Remaining: The Myriad Pools' "copy a permanent spell"
    cast trigger; Azcanta's "you *may* transform" (auto-transforms now);
    Search for Azcanta back-face dig ships but the "may reveal" is auto.
  - Daybound (CR 702.146): ETB "becomes day" ✅ and the cast-time "casting a
    daybound spell while neither day nor night makes it day" half ✅ (702.146e,
    in `finalize_cast`). The per-player night-entry rule beyond CR 502.2 is
    still ⏳.
  - Werewolf night→day check approximates "a player cast two or more spells
    last turn" as the global `spells_cast_last_turn >= 2`; a true per-player
    last-turn tally would be more faithful.
  - Manifest dread ✅ (Hauntwoods Shrieker; `Effect::Manifest`/`ManifestDread`
    + face-down 2/2 object + `GameAction::TurnFaceUp`). DFC sagas + Rooms
    (Unholy Annex) + meld (Westvale/Hanweir, Mightstone/Weakstone) + the Morph
    cast-face-down spell path still need their own subsystems on top.

- ✅ **Remaining STX printed cards** — all shipped (this run): layer-1 copy
  (Echoing Equation), Jadzi // Journey, Codie, Ecological Appreciation,
  Flamescroll // Revel. Historical blocker list below; only Kasmina's
  ability-sharing static + the inline `wants_ui` picker gaps remain.
- (historical) **Remaining STX printed cards (each needed a new primitive):**
  - **Continuous "becomes a copy of" (layer 1)** — until-EOT/permanent copy of
    a chosen permanent (Echoing Equation, Helm of the Host loop, Mirrorform).
  - **Fixed alternative cost "cast for {N} instead"** + **put-lands-from-hand-
    onto-battlefield** — Jadzi // Journey to the Oracle.
  - **`StaticEffect::CantCastPermanentSpells`** + a next-spell-cast reflexive
    impulse keyed to the cast spell's MV — Codie, Vociferous Codex.
  - **Up-to-N variable targets + opponent-split** — Ecological Appreciation.
  - **Variable-sacrifice cost reduction** ("sacrifice any number, {N} less
    each") — Awaken the Blood Avatar (currently 🟡: flat cost, sac dropped).
  - **Opponent-ability-activation trigger + spell-lock** — Flamescroll // Revel.
  - ✅ done this run: Plargg//Augusta, Extus//Awaken (🟡), Rowan//Will,
    Mila//Lukka, Valentin//Lisette (exile-instead + reflexive),
    Radiant Scrollwielder (non-combat lifelink, CR 702.15), Mascot Exhibition
    (corrected), tapped/untapped anthem filters, cross-type legend-rule fix.
  - **`Effect::Fateseal` / `Effect::DigToHandLoseLife` `wants_ui` suspend path**
    — both currently decide inline (the bot/scripted path); a networked human
    isn't prompted. Same gap as the existing inline pickers.
  - **Detain interactions** — `detained_by` blocks attack/block/activate and
    lifts at the detainer's next turn; a granted-static "permanents your
    opponents control enter detained" variant (Lavinia of the Tenth) is ⏳.

- ⏳ **Discovered this run (coin-flip / artifact batch — deferred cards):**
  - **Squee, the Immortal** — needs a static "you may cast this from your
    graveyard or from exile" permission (a real cast onto the stack, unlike
    Gravecrawler's `from_graveyard` Move approximation).
  - **Karplusan Minotaur** — cumulative upkeep whose cost is a coin flip
    (CR 702.24 + 705) + the win/lose-flip "deal 1 to any target" pair.
  - **Cursed Scroll** — name-a-card + reveal-at-random-from-hand + conditional
    damage if the random card matches.
  - **Price of Progress / Pyromancer Ascension / Tibalt's Trickery /
    Daretti, Scrap Savant** — per-player-scaled damage, quest-counter spell
    copying, counter-and-cascade-from-exile, and a planeswalker, respectively.
  - **Grafted Wargear** — equip {0} with "when unattached, sacrifice the
    creature" (no on-unequip sacrifice hook yet).
- ⏳ **Discovered this run (modern_decks staples/cleave/multi-pick run):**
  - **Engineered Explosives / Zabaz** — both need a counter snapshot that
    survives the source's sacrifice-as-cost: EE's "destroy each nonland
    with MV equal to its charge counters" reads the sacrificed source's
    counters at resolution (extend the `sacrificed_power` scratch family
    with a counter map, or concretize `ManaValueEqualsSourceCounters` at
    activation); Zabaz additionally wants a modular-trigger counter-bonus
    replacement.
  - **Hogaak, Arisen Necropolis** — needs "you may cast from your
    graveyard" on the *main* cast path (today only `from_graveyard`
    activations and flashback leave the graveyard), plus a "can't spend
    mana on this" gate forcing full Convoke+Delve payment.
  - **Runed Halo / protection from a card name** — `named_card` exists for
    ability suppression but not as a protection quality.
  - **Tidebinder Mage** — "doesn't untap while you control this" wants a
    linked `PreventUntap` (stamped like `exiled_by`), not a stun counter.
  - **Hallowed Moonlight / Containment Priest as EOT grant** — needs a
    turn-scoped `ExileNontokenCreaturesNotCast` (flag on GameState, not a
    battlefield static).
  - **Cultivator Colossus** — repeat-until-decline ETB loop primitive.
  - **Fell Stinger** — exploit payoff is bound to the controller; a real
    "target player" inside an exploit `MayDo` needs trigger-target plumbing
    through the reflexive body.
  - **Shacklegeist** — "can block only creatures with flying" restriction
    (inverse of CantBlockFlying) not modeled; rider dropped.
- ⏳ **Discovered (modern_decks landfall/exile batch):**
  - **Awaken the Blood Avatar** variable-sacrifice cost reduction still ⏳
    (auto-path sacrifices 0; needs a cast-time "sacrifice N, {2} less each"
    decision threaded into the cost computation).
  - **Before adding a "new" card, grep the catalog for its name** — Omnath
    already existed in `decks/modern.rs`; nearly duplicated it.
- ⏳ **Discovered this run (STX sweep / extras_17):**
  - ✅ **"Sacrifice X or pay {N}" OR additional cost** —
    `AdditionalCastCost::SacrificeOrPay` (Bayou Groff faithful; a wants_ui
    "which half?" chooser is a follow-up).
  - The STX "still wrong" list in *Suggested next-up tasks* was largely stale:
    Frost Trickster / Eager First-Year / Owlin Shieldmage / Promising Duskmage /
    Rise of Extus / Verdant Mastery / Illuminate History were already faithful.
    Re-verify before picking a sweep target.
- ⏳ **Phasing (CR 702.26) follow-ups**: a permanent that **enters phased out**
  (Reality Ripple-adjacent). **Granted phasing ✅** — `do_phasing` now reads
  computed keywords, so a layer-granted Phasing phases out at the untap step.
  **Mid-combat `Effect::PhaseOut` ✅** — removes the permanent from the combat
  arrays (702.26e). **"When this phases in" triggers ✅** — `EventKind::PhasesIn`
  + `GameEvent::PermanentPhasedIn`. **Linked "until [source] leaves" ✅** —
  `PhaseOut.until_source_leaves` + `CardInstance.phased_out_by`: skipped by
  the untap-step phase-in, returned by `on_left_battlefield` (Out of Time,
  with a time counter per phased permanent). Phased-out permanents surfaced
  per player via `PlayerView.phased_out` + a client HUD chip. The side-zone
  model (`GameState.phased_out`) is the hook.
- ℹ️ **Client build needs system libs** — `apt-get install -y libwayland-dev
  libasound2-dev libudev-dev` unblocks `cargo build/clippy -p
  crabomination_client` in the web sandbox (wayland-sys / alsa-sys / libudev
  build scripts otherwise panic). Install them once per session, then the
  client compiles and clippy runs clean.
- ⏳ **Discovered this run (allied-color card batch):**

- ⏳ **Discovered this run (sagas / attack-tax / pillowfort batch):**
  - **Attack-tax interactive pay** — `AttackTaxToController` auto-pays from the
    active player's floating mana; a wants_ui player needs a real "pay {N}?"
    prompt during declare-attackers (and a per-attacker / partial-pay choice).
  - **DFC / read-ahead Sagas** — `saga_chapters` covers single-faced Sagas only;
    transforming saga-lands (The Everflowing Well) and read-ahead chapter choice
    are still ⏳.

- ✅ **Emerge (CR 702.119).** `AlternativeCost.emerge` + `shortcut::emerge` —
  sacrifice a creature, reduce the emerge cost generically by its MV. Wretched
  Gryff ✅. Remaining emerge cards (Elder Deep-Fiend's "tap up to four",
  Distended Mindbender's reveal-and-choose-two) need their cast-trigger riders.
- ✅ **Awaken (CR 702.113) + Surge (702.108) + Rally — OGW/BFZ blockers.**
  All three ship via existing primitives + a small `AlternativeCost.marks_kicked`
  flag. Awaken/Surge live in `shortcut::{awaken, surge, animate_land}`; Rally is
  an `EntersBattlefield`/`YourControl` trigger filtered to `HasCreatureType(Ally)`.
  Wired Sheer Drop, Mire's Malice, Coastal Discovery, Roil Spout (Awaken);
  Comparative Analysis, Containment Membrane, Boulder Salvo, Goblin Freerunner,
  Reckless Bushwhacker, Tyrant of Valakut (Surge); Kor Bladewhirl, Tajuru
  Warcaller (Rally); Wall of Resurgence, Cyclone Sire (animate-land riders).
  - ⏳ **Awaken-cast UI targeting.** The client alt-cast modal now offers a
    direct "Cast" for plain alt costs (Surge/Awaken/Emerge), but doesn't yet
    drop into the targeting cursor for the awaken land (and any base target).
    Bots/tests pass targets explicitly; the human UI needs an alt-cast →
    targeting follow-up so Awaken's land slot can be chosen.
- ⏳ **OGW/BFZ cards skipped this batch (need a primitive).**
  - **Oblivion Sower** — process-onto-battlefield (target opp exiles top 4,
    then put any number of *their* land cards from exile onto the battlefield
    under your control). Needs a "play lands from opponent's exile" move.
  - **Processor Assault** — Process as a cast-time *additional cost* (not a
    trigger); needs the additional-cost-process hook.
  - **Vile Redeemer / Inverter of Truth / Conduit of Ruin** —
    per-creature-died token scaling, whole-library-exile, and
    tutor+cost-reduction respectively. (Cyclone Sire ✅ — animate-land on death.)
- ⏳ **Test harness: `check_state_based_actions()` doesn't dispatch
  *another-creature-died* watcher triggers.** A creature killed via raw
  `damage = N; check_state_based_actions()` fires its own death (SelfSource)
  triggers but not other permanents' "whenever another creature you control
  dies" watchers — those need the full event-dispatch path (kill via a damage
  spell + `drain_stack`, as the Grim Haruspex / Sifter of Skulls tests do).
  Worth auditing whether the direct-SBA path should also gather watcher
  triggers, or whether this is purely a test-only shortcut.
- ⏳ **Eldrazi-titan pass leftovers (this run).** Remaining primitives:
  (a) **Process** ✅ — `Effect::Process { count, then }` (put N cards an
  opponent owns from exile into their graveyards; `then` is the "if you do"
  rider). Ships Wasteland Strangler, Mind Raker, Blight Herder. Still ⏳:
  Oblivion Sower (process puts *lands onto battlefield*, not graveyard) and
  Processor Assault (process as a cast-time *additional cost*, not a trigger).
  (b) **conditional static keyword grant** ✅ — Eldrazi Aggressor rides
  `StaticEffect::PumpSelfIf { keywords: [Haste], … }` gated on an
  `OtherThanSource` colorless-creature count.
  (c) **non-linked exile-from-opponent-hand** ("you choose a nonland
  card and exile it" + a separate LTB draw) — Thought-Knot Seer; (d) Reaver
  Drone ✅ — the `OtherThanSource` self-exclusion threads through the
  `SelectorCountAtLeast` upkeep-condition path correctly (verified by test).
- ⏳ **Hand of Emrakul / Spawnsire alt-cost & wish.** Hand of Emrakul's
  "sacrifice four Eldrazi Spawn rather than pay mana" alt-cost and Spawnsire's
  {20} cast-from-outside-the-game are both dropped (no sacrifice-N-of-a-type
  alt-cost / wish primitives).
- ✅ **Goldvein Hydra death-treasure rider (LKI).** CR 603.10 leaves-battlefield
  LKI ships: `leaves_bf_lki` snapshots the dying object at every removal funnel
  (SBA lethal, destroy/sacrifice, `push_pending_trigger`) and survives until the
  trigger resolves, scoped by `resolving_lki_source`. `Value::PowerOf` /
  `ToughnessOf` read it (priority over the graveyard's printed P/T). Goldvein
  Hydra mints power-many Treasures; Cacophony Scamp / Heartfire Hero ping for
  last-known power. Remaining ⏳: LKI for other characteristics (color/types)
  read by leaves-battlefield bodies, and the tapped-Treasure rider.
- ⏳ **"Up to one target" for Suspect (Reasonable Doubt).** Currently modeled
  as a required creature target; a true optional single-target slot would let
  it resolve with the counter clause alone.
- ✅ **Client suspect/goaded/monstrous badges.** `build_tooltip_body`
  (`systems/counter_tooltip.rs`) renders "(suspected …)" / "(goaded …)" /
  "(monstrous)" status lines from the wire flags. A 3D on-card glyph (vs.
  the hover tooltip) is still a possible follow-up.

- **Look-at-hand riders (Peek, Telepathy).** Informational "look at target
  player's hand" has no mechanical primitive; only the cantrip half is
  modelable today.
- ✅ **Board-bounce to each card's owner (Aetherize / Evacuation).** Shipped
  via `PlayerRef::OwnerOfMoved`, resolved per-card in `place_card_in_dest`, so
  a single `Move { what: EachPermanent, to: Hand(OwnerOfMoved) }` routes each
  card to its own owner. Ships Aetherize / Evacuation. (AEther Gale's "six
  *target* nonland permanents" still needs a multi-target prompt.)
- **Evoke Incarnation faithfulness (MH2).** Subtlety's ETB targets any
  `IsSpellOnStack` rather than only creature/planeswalker spells (no
  card-type-on-stack filter yet). Endurance's "up to one target player"
  is narrowed to `EachOpponent` (no single-effect player-target slot —
  `ShuffleGraveyardIntoLibrary` takes a `PlayerRef`, not a targetable
  `Selector`). Add an `IsCreatureOrPlaneswalkerSpellOnStack` requirement
  (+ auto-target hook in `targeting.rs`) and a targetable player slot to
  promote both to fully faithful.
- **Graveyard-hate dies-trigger nuance.** `route_to_graveyard` /
  `ExileCardsBoundForGraveyard` redirect the *placement* to exile, but
  `remove_to_graveyard_with_triggers` still collects `CreatureDied` /
  LTB-to-graveyard triggers before the redirect. Under Rest in Peace a
  creature that's exiled-instead technically never "dies" (CR 700.4), so
  those dies-triggers shouldn't fire. Check `graveyard_exiled_for` before
  collecting dies-triggers to suppress them.
- **Modal 3-mode charms with per-mode targets** (Esper/Golgari/Azorius Charm).
  `ChooseMode` + per-mode `target_filter_for_slot_in_mode` works, but the
  2-color cube pools can't slot 3-color Esper Charm; add a guild-charm batch
  once a per-mode target picker / multicolor pool exists. Modes that need new
  primitives: "creatures gain lifelink EOT" mass keyword grant, "put attacking
  creature on top of library", split mill.
- **Oracle of Mul Daya / play-from-top-of-library.** Needs a
  "play lands from the top of your library" permission + top-card reveal.

- **Client modals for `ChooseMode` / `ChooseModes` / `DivideDamage` /
  `ChooseAmount` / `NameCard`.** `decision_ui.rs` only renders Scry / Search /
  PutOnLibrary / Discard / Mulligan / ChooseColor / Learn / OrderTriggers /
  ChooseTarget; the rest fall through `_ => {}`, so a networked human casting a
  modal spell (Commands, Callous Bloodmage) or an X-amount effect gets no
  picker and the seat degrades to the AutoDecider default. `ChooseMode` needs
  the mode label strings threaded onto `Decision::ChooseMode` (today it carries
  only `source` + `num_modes`); `effect_short_text` already renders each mode.
- **Amped Raptor energy free-cast (still 🟡).** Needs a `MayPlayPermission`
  alt-cost slot ("cast without paying mana by paying {E}{E}") + a cast-from-
  exile path that substitutes the energy cost.

- **Split-card follow-ups (CR 709 shipped this run).** The split primitive
  (`CardDefinition.split` + `CastSplitRight` / `CastSplitFused` / `CastAftermath`)
  and the bot/affordance wiring are in. Remaining:
  - **Client cast UI for the right/fused/aftermath halves.** The
    `splittable_right_hand` affordance now lights the cyan alt-cast border, but
    there's no modal to pick *which* half (left vs right vs fuse) — the click
    path only submits the left (`CastSpell`). Needs a small half-picker, like
    the MDFC face chooser.
  - **Fused targeting** currently assumes each half is single-target (left →
    `target`, right → `additional_targets[0]`); a fusable card with a
    multi-target half would need the slot convention generalized.

- **DSK/MKM gap cards deferred (recent240–241 follow-ups).** Each wants one
  small primitive (verified absent this run):
  - **Miasma Demon** — "discard any number; up to that many target creatures
    each get -2/-2." Needs a reflexive discard whose count caps a
    resolution-time multi-target debuff (`ApplyToTargets.max_targets` is a
    fixed `u8`; make it read a `Value`, or add a reflexive discard-then-targets
    effect).
  - **Grievous Wound** — enchant-*player* Aura with "enchanted player can't gain
    life" + "when dealt damage, they lose half their life." The `PlayerCannotGainLife`
    static and `LoseHalf` effect exist; needs a player-enchant Aura + a
    `PlayerRef::EnchantedPlayer` actor.
  - **Leyline of Transformation** — opening-hand + choose-a-creature-type static
    that adds the type to your creatures *and* spells/cards in other zones.
    Needs a continuous creature-type-add static keyed on `chosen_creature_type`.
  - **Leyline of Mutation** — "pay {W}{U}{B}{R}{G} rather than mana cost for
    spells you cast." Needs a general alt-cost static.
  - **Leyline of Resonance** — "copy your I/S that targets only a single
    creature you control." Needs a copy-on-cast static keyed on target shape.
  - **Leering Onlooker / Rubblebelt Maverick** — graveyard-activated abilities
    (`ActivatedAbility.from_graveyard` + `exile_self_cost` fields exist — wire a
    catalog card through them and confirm the activation path).
  - **Frantic Scapegoat** — the "when other creatures enter, if suspected, you
    may move the suspicion" rider (front haste + ETB-suspect ship; the reflexive
    suspect-another/`ClearSuspected`-self rider is dropped).
  - **Say Its Name** — the three-copy graveyard-exile combo that tutors Altanak
    (front mill+regrowth ships).
  - **Unidentified Hovership / Hedge Shredder / Dissection Tools / Chainsaw /
    Cursed Recording** — exile-remember-owner LTB manifest-dread; mill-lands-to-
    battlefield replacement; equip-cost-as-sacrifice; self-counter-scaled equip
    CDA; cast-count time-counter artifact.

- **Card primitives deferred this run (claude/modern_decks).** Real cards
  skipped for lack of a primitive — each is a small, reusable addition:
  - **Protection-from-each-color as one keyword/state** (Metalcraft-gated
    multi-protection) — Etched Champion.
  - **Skyclave-Apparition-style "exile until leaves, then owner makes an X/X"**
    (linked-exile with a leave-replacement that mints a token instead of
    returning) — Skyclave Apparition.

- **Embalm/Eternalize token color + cost overrides.** `sets::akh` tokens ride
  `CreateTokenCopyOf` and gain a Zombie type (+4/4 for Eternalize), but the
  copy keeps the original's color and printed mana cost rather than becoming
  "white/black with no mana cost." Add `token_color: Option<Color>` +
  `strip_cost: bool` to `Effect::CreateTokenCopyOf` to make it faithful.
- **More AKH/HOU Embalm cards.** Aven Wind Guide ✅ (token-scoped
  `GrantKeyword` anthems), Heart-Piercer Manticore ✅ (`MayDo` →
  `SacrificeAndRemember` → fling). Remaining: Vizier of Many Faces (embalm
  clone — needs the embalm-copy-any-creature path); `fanatic_of_rhonas`
  is missing its real Eternalize {2}{G}{G} — upgrade it.
- **Earthshaker Khenra's "≤ its power" filter is fixed at 2.** The ETB
  can't-block uses `PowerAtMost(2)` (the printed power); the eternalized 4/4
  token still reads 2. A source-relative `PowerAtMostSource` requirement would
  make it exact.

- **Equip-granted triggers — general dispatch.** Skullclamp ✅ (the equipped
  creature's `CreatureDied` equip-grant is now collected on the death path in
  `resolve_stack`). Still ⏳: chaining `EquipBonus.triggered_abilities` (and
  Soulbond-granted triggers) into the general `dispatch_triggers_for_events`
  walk so *any* equip-granted trigger shape (ETB, attacks, draws, …) fires —
  today only `DealsCombatDamageToPlayer` (combat.rs) and `CreatureDied`
  (death path) are covered.
- **Ghost Quarter's basic-land search rider** is dropped (the destroyed land's
  controller may fetch a basic). Needs last-known-controller resolution after
  the land leaves; pairs with a `PlayerRef::ControllerOf(last-known)` lookup.

- **Soulbond pairing is auto-resolved (CR 702.95).** `apply_soulbond_pairing`
  pairs with the lowest-CardId eligible partner instead of prompting the
  controller. Add a `Decision::ChooseSoulbondPartner` (with a decline option)
  so a UI seat can pick / decline the pair.
- **Soulbond-granted triggered abilities only cover combat damage.**
  `SoulbondBonus.triggered_abilities` are dispatched via the combat
  `DealsCombatDamageToPlayer` hook only (enough for Tandem Lookout). A general
  path (chain them into `dispatch_triggers_for_events` like
  `granted_triggers_eot`) would cover any future soulbond trigger shape.
- **Dethrone (CR 702.105) has no catalog card.** The `dethrone()` shortcut +
  `Predicate::PlayerHasMostLife` are wired and tested, but the only printed
  Dethrone cards are complex (Marchesa, the Black Rose — needs "other creatures
  you control have dethrone" trigger-grant-to-filter + die-return recursion).
  Ship one when those primitives land.
- **Reconfigure unattach (CR 702.151) — ✅ engine.** `GameAction::Reconfigure
  { equipment, target: Option<CardId> }` attaches (`Some`) or detaches (`None`)
  for the reconfigure cost; unattach restores creature-ness. Remaining: a
  client UI affordance to trigger the unattach (the `E`-key equip flow only
  attaches today).
- **Warp alt-cast keyword.** Warp (Mightform Harmonizer, Pinnacle Emissary —
  cast cheaply, exile at end step, recast later — a Suspend/Plot-adjacent
  exile-and-recast) is still dropped on its cards. **Miracle (CR 702.94) ✅** —
  `CardDefinition.miracle` + `maybe_grant_miracle` (first-draw alt-cost grant);
  Metamorphosis Fanatic can now wire its real miracle cost.
  **Offspring {N}** (CR 702.166) now ships
  via `Keyword::Offspring(cost)` reusing the Kicker pipeline (`has_kicker`
  returns the cost; `SpellWasKicked` gates an ETB 1/1 token-copy) — Thundertrap
  Trainer.
- **Card lookups now work offline.** `scripts/.scryfall_cache.json` has been
  expanded from 332 cards to the full Scryfall oracle set (~35.5k cards, every
  unique card keyed by name, with DFC/adventure front-face aliases), so the
  routine can implement any card without network access. Rebuild/refresh it
  with `python scripts/build_oracle_cache.py` (downloads the latest
  `oracle_cards` bulk and merges, preserving curated entries). Remaining card
  work: land monarch / Ascend / day-night payoff cards (the engine now
  supports all three) plus the long tail in `CUBE_FEATURES.md`.
- **Energy abilities as real costs.** `{E}{E}{E}: +1/+1` payoffs (Longtusk
  Cub, Bristling Hydra via `pay_energy_counter`) currently model the energy
  as an `Effect::PayEnergy` paid *at resolution* with `energy_cost: 0`, so
  they're technically activatable with no energy (the resolve no-ops). Now
  that `ActivatedAbility.energy_cost` exists, convert these to a true cost
  (gated up front). The bot's `pick_energy_payoff` now recognises both the
  `energy_cost`-bearing form and the resolve-time `Effect::PayEnergy` rider —
  remaining work is migrating the card definitions onto the real cost.

- **Energy-pay-to-cast-from-exile (Amped Raptor).** Needs a `MayPlay
  Permission` alt-cost slot ("cast without paying mana cost by paying {E}{E}")
  + a cast-from-exile path that substitutes the energy cost. Pairs with the
  existing `ExileTopAndGrantMayPlay` primitive.

- **Additional combat phase — main-phase variant (CR 505.1b).** The
  combat-phase loop ships (`Effect::AdditionalCombatPhase` +
  `GameState.additional_combat_phases`; Hellkite Charger-style combat-only
  activation re-loops Begin Combat at End of Combat). Still ⏳: main-phase
  sorceries that read "after this main phase, there is an additional combat
  phase followed by an additional main phase" (Relentless Assault, Aggravated
  Assault) — these need the extra combat (and main) inserted after the
  *current main phase*, not the End of Combat loop. Likely a small phase-queue
  on `GameState` consulted at both the main-phase and combat-phase exits.
- **Daybound / Nightbound DFC transform** (CR 702.146) — ✅ DONE.
  `Keyword::{Daybound,Nightbound}` ride the transform engine (CR 712):
  `set_day_night` flips daybound→nightbound DFCs to their back face when it
  becomes night and back when it becomes day; a daybound permanent entering
  while it's neither day nor night makes it day (702.146e). Ships Village Watch
  // Village Reavers. Remaining ⏳: the "casting a daybound spell makes it day"
  half (only the ETB rule is wired), and the no-spells-cast night entry rule
  beyond the existing CR 502.2 turn check.
- **The Initiative** (CR 726) reuses the monarch infrastructure (designation +
  combat-damage steal + leaves-game transfer) but needs Venture into the
  Dungeon / the Undercity (CR 701.49) for its payoff — implement the dungeon
  zone first, then the Initiative is a thin wrapper over the monarch pattern.
- **Client HUD for monarch / day-night / city's blessing — ✅ DONE.** The
  viewer's stat-chip row (`game_ui/player_stats.rs`) now spawns a crown chip
  (`👑`, CR 724) when the viewer is monarch, a `✦ blessed` chip (CR 700.6)
  when they have the city's blessing, and a `☀ day` / `☾ night` chip (CR 731)
  whenever the global day/night designation is set. Remaining: surface
  monarch on *opponents'* rows too (the chip row only renders the viewer
  today) and a board-center day/night ambient cue.

- **Block-restriction follow-ups (CR 509.1b).** The `CantBeBlockedExceptBy`
  filter matcher (`blocker_matches_block_filter`) covers type/color/keyword/
  P-T; "except by Walls/multicolored/specific subtype" compose already. Still
  needing other primitives: Signal Pest / Goblin Piledriver, Soldier of the
  Pantheon ("protection from
  multicolored" — a non-color protection grant). Brimaz's block-token rider
  and Whirler Rogue's "tap an artifact: grant unblockable" activated cost are
  also still ⏳.
- **`AffectedPermanents::CardMatch` could absorb P/T-gated anthems** if its
  matcher read *computed* power/toughness (it's card-printed-only today, so
  power/toughness thresholds still fall through to `None` — the P/T-gated lord
  gap noted under "Anthem coverage" below).

- **Protection on *ability* targeting + damage from spell sources.** CR
  702.16e/f are wired for spell targeting, equip, and the combat/noncombat
  *permanent*-source damage paths, but `check_target_legality` (activated/
  triggered ability targets) doesn't yet reject a protected target, and a
  *spell* damage source (Pyroclasm-style mass damage) isn't color-known at
  damage time (the card is in transient ownership), so its protection-from-
  color prevention degrades. Thread the resolving spell's color into the
  damage path and add a protection check to `check_target_legality`.
  Also: "protection from artifacts/colorless" (Giver of Runes, Apostle's
  Blessing's artifact mode) needs a non-color protection grant.
- **Per-player "half their own X" generalization.** `Effect::LoseHalfLife`
  scales to each target's own life; the same per-player pattern would finish
  Lord Xander (mill half *their* library, sacrifice half *their* permanents)
  — generalize to `Effect::MillHalf`/`SacrificeHalf` or a context-bound
  current-player ref so `Mill`/`Sacrifice` can read each target's count.
- **Anthem `affected_from_requirement` coverage.** Color (`HasColor`),
  `IsToken`/`NotToken` (→ `AffectedPermanents::All.token`, ships Intangible
  Virtue / Always Watching) are decomposed, and the opponent path
  (`ControlledByOpponent`) composes with type filters regardless of And-tree
  order. Remaining: power/toughness thresholds still fall through to `None`
  (anthem silently doesn't apply) — needed for P/T-gated lords.
- **Plague Engineer / named-creature-type -1/-1.** Needs a
  `StaticEffect` that diminishes only a chosen creature type among opponents
  (the existing `DiminishCreaturesExceptChosenType` is the inverse). Dropped
  this run to avoid an inaccurate flat anthem.
- **"Can't be blocked except by …" restrictions — ✅ DONE (primitive).**
  `Keyword::CantBeBlockedExceptBy(filter)` / `CantBeBlockedBy(filter)` (CR
  509.1b) are read in `can_block_attacker_computed` via
  `blocker_matches_block_filter` (a computed-characteristic matcher: type,
  color, keyword, power/toughness thresholds). Ships Silhana Ledgewalker
  (except by flyers) and Steel Leaf Champion (not by power ≤ 2). Remaining
  consumers: Goblin Piledriver / Soldier of the Pantheon (these have other
  riders — protection-from-color is their real evasion), Signal Pest.
- **Unleash bot nuance.** `optional_trigger_beneficial` accepts the Unleash
  +1/+1 counter as pure upside, but the counter disables blocking
  (`Keyword::CantBlock`). A defensive bot should weigh board state before
  taking it.

- **Adventure / Plot client modals** (CR 715 / 702.170). Engine + bot +
  affordance hints (`adventurable_hand` / `plottable_hand`) ship, but a
  `wants_ui` human gets no modal to *choose* between casting the creature vs.
  the adventure half, or to plot a card / cast it from exile later. Wire a
  client cast-mode picker off the new affordance sets (mirror the kicker /
  bestow toggle). `CastAdventureCreature` / `CastPlotted` from exile also have
  no client surface yet.
- **Protection-from-chosen-color grant — ✅ DONE.**
  `Effect::GrantProtectionFromChosenColor { what, duration }` surfaces
  `Decision::ChooseColor` then grants `Keyword::Protection(color)` for the
  duration (Mother of Runes, Gods Willing wired). Spell-targeting protection
  now reads *computed* keywords so the granted protection is honored.
  Remaining: protection isn't checked on *ability* targeting
  (`check_target_legality`) or combat-damage prevention reads — extend those
  to read computed protection if a card needs it (Giver of Runes "protection
  from colorless" also needs a colorless option).
- **Suspend (CR 702.62) — ✅ DONE (primitive + haste + accelerant +
  granted suspend 702.62e via `Effect::GrantSuspend`/`granted_suspend`, and
  the CR 601.3e suspend-only cast gate `CardDefinition.suspend_only`).**
  `Keyword::Suspend(n, cost)` + `GameAction::Suspend` + `process_suspend`
  ship the exile-with-time-counters → tick-at-upkeep → free-cast loop
  (Rift Bolt, Ancestral Vision, Lotus Bloom). A suspend-cast creature now
  gains haste (CR 702.62f) via `CardInstance.cast_from_suspend`; Deep-Sea
  Kraken's accelerant ships via `Keyword::SuspendAccelerant` +
  `process_suspend_accelerants` (opponent's cast ticks a time counter).
  Remaining: the free cast auto-targets via the AutoDecider's first-legal
  pick; a `wants_ui` human should be prompted for the targets (and X) of the
  cast spell. Also: no client affordance exists to suspend a card from hand.
- **One-shot spell-cost discount — ✅ DONE (primitive).**
  `Effect::GrantNextInstantOrSorceryDiscountThisTurn { amount }` pushes a
  `(amount, granted_at)` entry onto `Player.pending_is_discounts`;
  `cost_reduction_for_spell` adds it for IS spells while the player's
  `instants_or_sorceries_cast_this_turn` tally still equals `granted_at`, so it
  self-expires on the next IS cast with no consume hook. Cleared in lockstep
  with the tally each turn. A real consumer card (Thundertrap Trainer's dropped
  discount rider) has a synthesized catalog body, so the exact amount should
  be re-checked against the Scryfall cache.
- **Squad / Bargain keywords.** Squad (CR 702.157) needs "pay an
  additional cost any number of times" tracking + copy-of-self tokens (the
  `CreateTokenCopyOf` half exists). Bargain (CR 702.176) is an
  optional sacrifice-as-additional-cost (shares the unbuilt Casualty cost-mode
  primitive). Backup N (CR 702.164) is ✅ via `shortcut::backup(n, keywords)`
  (ETB +N/+N counters on target + EOT keyword grant; Conclave Sledge-Captain,
  Death-Greeter's Champion). Remaining: granting *triggered* abilities (not
  just keywords) to the backed-up creature.
- **Bot accepts beneficial Exploit/Devour.** `shortcut::exploit` /
  `devour` resolve their sacrifice via `MayDo` / `SacrificeAnyNumber`;
  `AutoDecider` and the current bot decline (the body is self-costly by
  `optional_trigger_beneficial`). A value-aware bot would accept when it
  controls a spare token/weak creature and the payoff outweighs it
  (`Decision::ChooseAmount` for devour, `OptionalTrigger` for exploit).
- **Client `Decision::ChooseCards` modal.** The new "exile any number of
  target cards" decision (`ExileAnyNumberFromGraveyards`, Devious Cover-Up)
  has wire + bot + AutoDecider support but no Bevy multi-select modal yet —
  a `wants_ui` human degrades to the AutoDecider "exile nothing". Add a
  graveyard multi-pick modal (mirrors the Discard hand-pick UI).
- **Buyback / Bestow client + bot.** `GameAction::CastSpellBuyback` (CR
  702.27) and `GameAction::CastBestow` (CR 702.103) are wired + tested and
  surfaced in `PlayerView.buyback_hand` / `bestowable_hand`. The bot now
  offers a Bestow line (enchant its sturdiest creature) in
  `main_phase_action`; **Buyback** is still bot-TODO, and the Bevy client
  still has no "pay buyback?" / "bestow on a creature?" affordance.
- **Foretell (CR 702.143) — ✅ DONE.** `CardDefinition.foretell_cost` +
  `GameAction::Foretell` (pay {2}, exile face-down, sorcery speed) +
  `GameAction::CastForetold` (cast from exile for the foretell cost on a
  later turn; gated by `GameState.foretold_this_turn`). Wired Saw It Coming,
  Doomskar, Behold the Multiverse; surfaced as `PlayerView.foretellable_hand`
  + cyan client highlight. Remaining: a client affordance to invoke Foretell /
  cast a foretold card (no Bevy modal yet), and AI never foretells.
- **"Exile any number of target cards" (graveyard hate).** ✅ Wired via
  `Effect::ExileAnyNumberFromGraveyards` + `Decision::ChooseCards`
  (AutoDecider exiles nothing; the bot exiles opponents' cards). Devious
  Cover-Up is now faithful. Remaining: extend `ChooseCards` to *battlefield*
  / hand "any number of target permanents" pickers (it's graveyard-only
  today) and surface a client multi-select modal.
- **Enduring cycle breadth.** `Effect::ReturnSelfAsEnchantment` handles the
  "return as enchantment" half (Enduring Innocence). The other Enduring
  cards (Vitality, Tenacity, Courage, Curiosity) keep distinct enchantment-
  side static abilities, which this primitive doesn't preserve/swap — extend
  it to carry the enchantment-side ability set when those cards are added.
- **Discard / exile-from-gy as real activation costs.** Psychic Frog (and
  similar) model "Discard a card:" / "Exile three cards from your graveyard:"
  as the first step of the resolved effect rather than a paid activation
  cost. Gameplay-equivalent today (nothing responds between cost and
  resolution), but a real cost (new `ActivatedAbility` fields) would gate
  activation on having the cards and let the cost be paid before the ability
  goes on the stack.
- **Ninjutsu client UI** — `GameAction::Ninjutsu` is wired + tested in the
  engine (Fallen Shinobi), but the Bevy client has no affordance to invoke
  it during the declare-blockers step (pick a ninja in hand + an unblocked
  attacker to return). Add a button/flow like Crew. The bot doesn't use
  Ninjutsu either (it would need a "swap up" heuristic).
- **Reuse `StaticEffect::PumpSelfByControlledPermanents`** — the new
  self-buff-scaled-by-controlled-permanents static (Karn's Construct token)
  also fits Master of Etherium, Tempered Steel-style self-counts, and any
  "this gets +1/+1 for each [type] you control" body currently stubbed as a
  fixed P/T. Apply opportunistically when real card data is available.
- **Client build in CI/web env** — `crabomination_client` (Bevy) fails to
  build here because `wayland-client` system libs aren't installed, so
  client-side changes can't be compiled/tested in this environment. UI
  parity is fed through the server `view.rs` projection (cost labels,
  static/triggered ability labels) which *is* testable.
- **`Decision::ChooseAmount` UI suspend** — `SacrificeAnyNumber` /
  `PayLifeLookTake` resolve the number-choice synchronously via the decider
  (AutoDecider picks 0). A `wants_ui` player should suspend on a number-picker
  modal instead of degrading to 0. Add a `ChooseAmountPending` suspend path +
  client widget (like the Learn modal).
- **`SacrificeAnyNumber` reuse** — Devour and Fling-with-count can now ride
  `Effect::SacrificeAnyNumber` + `Value`-scaled payoffs.
- **Opponent-controlled pay-to-copy** — Chain Lightning's "the damaged player
  may pay {R}{R} to copy this spell." `Effect::CopySpell*` exist but are all
  controller-side; needs a copy offered to a different player.
- **Card-data audit vs Scryfall cache** (`cargo run --bin dump_cards` diffed
  against `scripts/.scryfall_cache.json`). The claude/modern_decks run fixed
  18 mana-cost bugs and 4 keyword bugs this way. **Remaining diffs are all
  legitimate** and should NOT be "fixed": X-spells store the base cost
  without `{X}` (Banefire, Earthquake, Mind Twist, Repeal, Prismatic
  Ending); free spells store an empty cost = `{0}` (Ornithopter, the Pacts,
  Zuran Orb); Adventure/MDFC fronts (Callous Sell-Sword, Cruel Somnophage);
  cost-reduction approximations (Blasphemous Act ships flat `{4}{R}` vs the
  printed `{8}{R}` minus a per-creature reduction the engine can't scale);
  colorless-pip approximations (Devourer of Destiny `{7}` for `{5}{C}{C}`);
  CDA P/T (Cosmogoyf, Lumra, Cruel Somnophage); and the custom card
  Crabomination. Re-run the audit after big card batches to catch new typos.

- **Multi-slot "up to two target" works** for explicit casts (proved by
  Read the Tides' modal bounce). Cards still collapsing it to one (Aether
  Helix's bounce, etc.) can adopt the two-slot `Move` pattern; the
  remaining gap is the *auto-target* picker only filling slot 0 for bots.

- **"May" triggers: bot now value-aware; human suspend still ⏳.**
  `AutoDecider` still declines every `Decision::OptionalTrigger`
  (`Bool(false)`), but **`HeuristicBot` now takes beneficial ones**
  (`optional_trigger_beneficial` — accept unless the matching `MayDo` body
  imposes a self-cost: lose life / sacrifice / discard). Tests:
  `bot_takes_beneficial_optional_trigger`,
  `bot_declines_self_costly_optional_trigger`. Remaining: a `wants_ui`
  suspend so a networked human is actually prompted (today they land on the
  AutoDecider `false` default), and revisiting `shortcut::provoke`'s
  collapse-to-mandatory now that bots can opt in.

- **AutoDecider declines all library searches** (`Decision::SearchLibrary
  → Search(None)` in `decision.rs`) — kept as-is so tests stay
  deterministic. The **bot** now overrides this: `HeuristicBot` handles
  `Decision::SearchLibrary` via `decide_library_search` (prefer a basic
  land toward the weakest color, else fetch the first candidate), so
  singleplayer tutors actually fix mana. Tests: `bot_search_*`. Remaining:
  a smarter non-land pick (fetch the best spell, not just the first).
- **Divided damage through a trigger fills only one slot.** Fury's evoke
  ETB (`DealDamageDivided { max_targets: 2 }`) auto-targets a single
  creature and dumps the whole total there; the multi-slot fill in
  `auto_targets_for_effect_all_slots` isn't reached from the trigger
  dispatch path. Thread the multi-slot picker through `fire_step_triggers`
  / trigger auto-target. (Single-slot auto-target through step/emblem
  triggers works — Saheeli Rai's -7 emblem copy body resolves correctly.)
- **Client kicker affordance.** `kickable_hand` (and `pitchable_hand`) now
  light up green as "playable now" via `update_castable_highlights` (unioned
  into the castable set alongside `dashable_hand`). Still wanted: a *distinct*
  "pay kicker?" badge/toggle that submits `GameAction::CastSpellKicked`
  (vs. the plain castable-green). Not compile-verified here (client can't
  build in this sandbox).
- **Provoke (targeted must-block).** `Keyword::AllMustBlock` (Lure) +
  `MustBeBlocked` (Academic Dispute) cover the untargeted 509.1c cases;
  Provoke's "that creature must block this + untap it" needs a per-blocker
  `CardInstance.must_block_attacker` link set by an attack trigger and
  cleared at end of combat.
- **Kicker — ✅ wired (CR 702.32, claude/modern_decks).**
  `GameAction::CastSpellKicked` folds the optional kicker cost into the
  spell's mana cost and stamps `CardInstance.kicked`;
  `Predicate::SpellWasKicked` reads it at resolution (via
  `EffectContext.kicked`) and `target_filter_for_slot_in_mode_kicked` makes
  cast-time target legality follow the `If(SpellWasKicked, …)` branch that
  will resolve. Tear Asunder promoted (exile artifact/enchantment, or any
  nonland permanent when kicked). Remaining: a client affordance to opt
  into the kick (a "pay kicker?" toggle on cast) and a bot heuristic to
  kick when profitable (today the bot only casts unkicked); more kicker
  cards (multikicker, kicker-with-different-effect riders).
- **Pitch affordance in client** — `pitchable_hand` cards (Force of Will /
  Spirit Guides) now light up green as "playable now" (unioned into
  `update_castable_highlights`), so a card uncastable for mana but pitchable
  still shows as playable. Still wanted: a *distinct* edge/badge separating
  pitch-castable from hard-castable. Not compile-verified here (client can't
  build in this sandbox).

- **Counter-mechanic follow-ons** (after Modular/Graft/Renown/Outlast/Melee/
  Bloodthirst this run): **Monstrosity** ✅ (`CardInstance.monstrous` +
  `Effect::Monstrosity` + `EventKind::BecameMonstrous`; Nessian Wilds Ravager,
  Ember Swallower). "As long as this is monstrous, …" statics ✅ via
  `Predicate::SourceIsMonstrous` + `StaticEffect::PumpSelfIf` (now multi-keyword
  — Fleecemane Lion gains hexproof + indestructible; Dragon's Rage Channeler's
  delirium grants flying + attacks-each-combat); **Devour** ✅ and **Amass** ✅ (`Effect::Amass` grows /
  creates a 0/0 black Army with N +1/+1 counters; `CreatureType::Army`).
  **Melee** is a
  flat +1/+1 — wants a per-combat attacked-opponent tally for multiplayer.
  **Renown** ✅ now keys off a real `CardInstance.renowned` flag
  (`Predicate::SourceIsRenowned` + `Effect::BecomeRenowned`), so unrelated
  +1/+1 counters no longer suppress it.
- **Mulligan color-screw** — ✅ done (claude/modern_decks). `decide_mulligan`
  now unions the producible colors of the hand's lands (`land_color_output`:
  basic land types + `AddMana` payloads; "any color" → WUBRG) and only counts
  an early play whose colored pips are a subset. Test:
  `bot_mulligans_color_screwed_hands`. Remaining: dual/fetch lands that fetch
  off-color sources aren't followed transitively (a lone fetchland reads as
  colorless).
- **Client build (this env)** — `crabomination_client` can't compile here
  (`wayland-sys` build script fails: no system `wayland-client`). UI changes
  this run (keyword reminder-text additions in `counter_tooltip.rs`) are
  additive `&'static str` data and weren't compile-verified in this sandbox.
- **Divided damage** — ✅ shipped: `Effect::DealDamageDivided { total, filter,
  max_targets }` + `Decision::DivideDamage` (AutoDecider spreads evenly; UI/
  scripted deciders choose the split). Wired Forked Bolt, Pyrokinesis, Crackle
  with Power, Magma Opus, Electrolyze, Pyrotechnics, Pyromathematics,
  Lorehold Ignis/Bookburn, Arc/Forked Lightning, Chandra's Pyrohelix.
  Remaining: (a) a **client modal** so a networked human picks the split
  (today the inline decider resolves it — fine for bots/tests/AutoDecider;
  no resolution-time *suspend* path for `DivideDamage` yet), and (b)
  divided *non-damage* riders ("tap up to N", split-mill — Snow Day, Devious
  Cover-Up).
- **Network note (this run):** Scryfall (`scripts/fetch_cards.py`) returns
  HTTP 403 under the sandbox network policy, so new cards this run were limited
  to ones whose definitions are already in the repo (comments/md) or
  high-confidence staples. The Verge / Landscape / Horizon-canopy land cycles
  and other cube ⏳ entries still want Scryfall-verified definitions before
  wiring — re-run with network access.
- **Pool registration** — this run's new cards are wired into `cube.rs`
  color pools (blue: Aether Adept, Augury Owl, Cloudkin Seer, Merfolk Skydiver,
  Benthic Biomancer, Pteramander, Quandrix Cryptomancer; white: Pridemalkin;
  red: Arc/Forked Lightning, Chandra's Pyrohelix). Pridemalkin's "trample for
  countered creatures" static and the Verge/Landscape land cycles still want
  Scryfall-verified definitions.
- **`Effect::NameCard` for spells** — currently only stamps a *battlefield*
  permanent (`named_card`). Spoils of the Vault / Cabal Therapy name a card
  during *spell* resolution; that needs the chosen name captured into
  `EffectContext` (e.g. `EffectContext.named_card`) so a following Seq step
  (reveal-until-find by name, hand-discard-by-name) can read it. Pair with a
  `SelectionRequirement::HasNamedCardInContext`.
- **"Name a card"** primitive — ✅ base shipped: `Decision::NameCard`,
  `DecisionAnswer::NamedCard`, `Effect::NameCard`, `CardInstance.named_card`,
  and `activate_ability` ability-suppression for matching sources (Pithing
  Needle, Phyrexian Revoker). Remaining consumers that need the named value
  threaded into resolution: same-name exile (Crumble to Dust), reveal-until-
  find (Spoils of the Vault), hand-discard-by-name (Cabal Therapy). The
  client picker UI (free text over the catalog) is also still TODO.
- **Stale "two-target prompt ⏳" notes** — several catalog doc-comments still
  claim multi-target sorcery prompts are unavailable; the slot-1+ picker
  (`auto_targets_for_effect_all_slots`) is wired and the bot uses it. Sweep
  and update the remaining notes (Channeled Force done this run).


- **Tracker staleness** — CUBE_FEATURES.md / DECK_FEATURES.md carry many 🟡/⏳
  rows that are already fully implemented + tested in code (verified + promoted
  this run: Conclave Sledge-Captain, Temur Ascendancy, Trinisphere — all had
  the needed primitive wired but a stale "⏳ primitive" note). Earlier runs hit
  Opposition, Omniscience, the shock/fast/surveil/bridge/pathway land families.
  Many doc-comments still claim a primitive "doesn't exist yet" when it does
  (e.g. Stadium Tidalmage's `MayDo`, the SOS placeholder-copy cards vs
  `CreateTokenCopyOf`). A reconciliation pass would shrink both trackers.
- **Remaining 🟡 cube/deck partials are primitive- or data-blocked.** The
  cleanly-completable ones were finished this run (Cryptic Command,
  Kolaghan's Command, Master of Cruelties, Lotus Field, Coalition Relic,
  Wishclaw Talisman). What's left needs new engine primitives — split cards
  (Wear // Tear), name-a-card (Pithing Needle, Crumble to Dust), loyalty-set
  (Geyadrone), energy (Amped Raptor), divided damage / "any number of targets"
  (Pyrokinesis, the STX Outburst/Snow Day cycle), escalate (Collective
  Brutality), multi-player choice (Indulgent Tormentor) — or are synthesized
  bodies whose exact text should be re-derived from the Scryfall cache.
- **Remaining ⏳ cube cards are each blocked on a distinct new subsystem.**
  After this run's clean adds (Kestia, Brightglass, Korvold, Maelstrom Nexus,
  Conclave, Death-Greeter's, Shiko, Parallax Dementia, Mutable Explorer, Teval,
  Sab-Sunen), the rest of the missing list maps 1:1 to a sizable engine feature,
  grouped here so the next run can pick a subsystem and clear several at once:
  **dynamic/scaling equip bonus + Reconfigure + Living weapon** (Lion Sash,
  Nettlecyst, Sword of Body and Mind, Helm of the Host); **face-down permanents
  / manifest dread** (Hauntwoods Shrieker, Concealing Curtains); **Mutate**
  (Mutated Cultist + others); **ETB-control replacement** (Gather Specimens);
  **clone-many / continuous copy** (Mirrorform); **borrow activated abilities
  from graveyard/exile** (Necrotic Ooze, Agatha's Soul Cauldron); **cast-from-
  graveyard engine** (Muldrotha, The Gitrog Monster); **Saga + lore counters**
  (The Everflowing Well, Rediscover the Way); **Hideaway** (Shelldock Isle);
  **Storm cast-from-top** (Mind's Desire); **Companion** (Zirda); **DFC //
  Land** (Sink into Stupor, Unholy Annex); **phasing system** (Talon Gates);
  **all-colors / all-land-types static** (Leyline of the Guildpact);
  **tempting-offer multiplayer choice** (Tempt with Bunnies); **`LookPickToHand`
  take-N** (Consult the Star Charts); **parity attack-gate** (Sab-Sunen → ✅).
- **Multi-target "choose two"** — `Effect::ChooseN` allocates a target slot
  per chosen mode; Cryptic Command (counter/bounce) and Kolaghan's Command
  (reanimate/any-target) now ship the faithful "choose two". Remaining:
  cast-time mode *selection* so a non-default pick routes its targets (see
  CR 700.2d below), and *divided* targeting within one mode/effect (Vibrant
  Outburst, Snow Day, Crackle with Power — split-N / divided-damage slots).
- **Dynamic P/T CDA generalization** — characteristic-defining `*/*` P/T
  (Nightmare = Swamps you control, Master of Etherium) is hand-wired per card in
  `compute_battlefield` (Tarmogoyf pattern). A `StaticEffect::SetPtFromValue`
  layer-7b primitive would let Nightmare-class cards drop in.
- **More combat keywords** — Frenzy/Afflict/Afterlife shipped this run as
  trigger shortcuts; Melee (CR 702.121, needs an "opponents attacked this
  combat" Value), Provoke, Dash, Boast remain ⏳.
- **"Becomes a copy" continuous layer-1 effects** — the one-shot copiers
  (Clone, Phantasmal Image, Mirror Image, Stunt Double, Spark Double,
  Mockingbird) ship via `Effect::BecomeCopyOf`. Mockingbird's name-retention
  exception (CR 707.2) is wired via `EntersAsCopy.keep_name`. Still open:
  continuous layer-1 "becomes a copy" effects (Helm of the Host loop,
  Mirrorform), copied enters-with-counters, and a real copy-target picker
  (auto-picks highest power today).
- **Overload (CR 702.96)** — Cyclonic Rift's `{6}{U}` mode. Needs an
  alt-cost that rewrites "target X" → "each X" at cast time (the alt-cost
  model can't yet swap a selector's target into an each-selector).
- **Linked-exile return as a stack trigger** — `return_linked_exiles`
  returns the card directly rather than via a stack-based "when ~ leaves"
  trigger. Fine for observable behavior; only matters for response windows
  on the return (e.g. a board-wipe race).
- **Nexus of Fate graveyard replacement** — needs a
  shuffle-instead-of-graveyard replacement once a leaves-graveyard
  replacement primitive exists (the rest of the extra-turn pipeline ships).
- **Choose-N modes ("choose two")** — still open per `FEATURE_ROADMAP.md`
  Tier 1 (additional cast costs, `GrantActivatedAbility` static, and "when
  target dies this turn" delayed trigger already shipped).
- **Echoing Truth same-name bounce** routes every copy to `OwnerOf(Target0)`;
  mixed-ownership same-named permanents would all go to the target's owner.
  Needs a per-moved-card owner destination to be fully correct.
- **Nykthos UI** — the `DevotionOfChosenColor` payload suspends on a
  `ChooseColor` for wants_ui players; a devotion preview on the chip would
  help (the count is shown in the HUD already).
- **Theros gods** ✅ — the full THS-block pantheon ships (Heliod, Purphoros,
  Pharika, Karametra, Keranos, Xenagos, Athreos, Ephara, Iroas, Kruphix,
  Mogis, Phenax + the earlier Nylea/Thassa/Erebos), with new primitives
  `PreventDamageToYourAttackers` (Iroas), `UnspentManaBecomesColorless`
  (Kruphix), and `Predicate::AnotherCreatureEnteredControlLastTurn`
  (Ephara — per-turn `creatures_entered_{this,last}_turn` log). Remaining:
  the Theros: Beyond Death two-pip gods.
- **Client build deps** — building the client in the web sandbox needs
  `libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev` (install via
  apt). Once present `cargo build/clippy -p crabomination_client` works.

## Suggested next-up tasks

- ⏳ **A "next spell only" spend permission.** North Star grants CR 609.4b for
  the whole turn (`Player.may_spend_any_color_this_turn`); the printed card
  scopes it to one spell.
- ⏳ **`Duration::UntilYourNextUpkeep`** — Halfdane, Gabriel Angelfire and the
  rest of the "until your next upkeep" wordings currently round to
  `Permanent` / `UntilYourNextUntap`.

- ⏳ **Legends is at 9 gaps** (`set_gaps.py leg`). Seven waves shipped 264
  cards; see "Legends — opened" above for what's left and why — each remaining
  card is blocked on one primitive.
- ⏳ **The client can't declare attacking bands.** The engine action
  (`GameAction::DeclareAttackersBanded`) and the `ClientView.attack_bands`
  read-back ship, and the tooltip names a creature's bandmates, but the
  attack UI has no band-grouping affordance — a human player can only attack
  unbanded.
- ⏳ **`Effect::ReplaceCreatureTypeText` rewrites the definition via a serde
  round-trip.** It substitutes any string value equal to the type's variant
  name, skipping the `name` key. That reaches every filter and effect body
  without a per-variant visitor, but a future enum with a unit variant named
  after a creature type would be caught in the same net.
- ⏳ **The auto-targeter fills only one graveyard slot of an "up to N target"
  trigger** (Celestial Gatekeeper). `auto_extra_targets_for` now peels a `Seq`
  whose only targeting member is the multi-target body, and the graveyard walk
  honors the `avoid` set, but the extra slot still comes back empty — the
  remaining break is somewhere between the peel and
  `auto_target_for_effect_avoiding_set`. A `wants_ui` seat picks both.
- ⏳ **Spy Network's "top card of that player's library" clause is dropped.**
  The hand and face-down halves ship (`LookAtHand` + `LookAtFaceDown`); a
  one-card library peek needs a `library_top_revealed_to` twin of
  `GameState.face_down_revealed_to`.
- ⏳ **`RevealTopOpponentChoosesToHand`'s opponent is a heuristic**, not a
  prompt — it hands over the lowest-mana-value eligible card. Fine for
  Karn's +1 and Animal Magnetism, but a real pick belongs on the opposing
  seat's decider.
- ⏳ **`Effect::EachPlayerChoosesCreatureTypeThen` asks the synchronous
  decider for every seat**, so a UI player isn't prompted for their own
  Harsh Mercy / Patriarch's Bidding pick (same gap as `TemptingOffer`). The
  single-chooser `ChooseCreatureTypeThen` does suspend correctly.
- ⏳ **`Effect::HeadGames`' search is a single `ChooseCards` prompt**, not the
  standard `SearchLibrary` flow, so it doesn't route through the search-tax /
  can't-search statics. Fold it into `Effect::Search` once that path can
  search one player's library on another player's picks (CR 701.19a).
- ⏳ **`Effect::MayCopyThisSpell` prompts the affected seat through the
  installed decider**, not that seat's UI suspend — see the Server bullet
  above. Same for the chain's retarget (`repoint_copy_target`).
- ⏳ **The Chain cycle's toll is all-or-nothing per link.** The printed cards
  let the affected player decline the copy *after* paying (they may sacrifice
  a land, and only then choose whether to copy); the engine asks first and pays
  only on a yes. Observationally identical unless a sacrifice trigger changes
  the player's mind.
- ⏳ **CR 121.8 / 121.9** — mid-cast face-down draw and reveal-on-draw, the two
  remaining CR 121 clauses.
- 🟡 **CR 115.7c** — "change any targets" now walks every declared slot
  (`Effect::ChangeTargetOfAbility`; test `cr_115_7c_reroute_repoints_every_slot`).
  Remaining: letting the chooser keep a *subset* of the current targets rather
  than repointing each slot that has an alternative.
- ⏳ **Sector designations are auto-assigned.** `GameState::assign_sectors`
  (CR 704.5u) spreads a player's creatures round-robin instead of asking; a
  `wants_ui` seat should get the real choice. `Effect::ChooseSector` likewise
  auto-picks the fullest sector for a bot/auto seat.
- ⏳ **Search the City's return is auto-picked.** With several exiled copies of
  a name, `Effect::SearchTheCityReturn` returns the first — the printed text
  lets the controller choose which.

- ⏳ **recent239 (DSK/OTJ/MKM) deferred, each blocked on one primitive:**
  - **Type-filtered death tally** — "if a non-Zombie creature died this turn"
    (Undead Sprinter's graveyard-cast condition). Needs either a filtered
    death predicate or a small per-turn typed tally on `Player`.
  - **Tap-1-or-2-then-each-deals-power** — Coordinated Clobbering (needs
    explicit tapper target slots + a shared recipient slot).
  - **Dual-pile exile-return-to-hand linked to LTB** — Fear of Abduction (the
    additional-cost-exiled own creature and the ETB-exiled opponent creature
    both return to their owners' hands when it leaves).
- ⏳ **Newly-noticed primitives (RNA batch):**
  - **Your instants/sorceries have deathtouch** static — Pestilent Spirit
    ("Instant and sorcery spells you control have deathtouch"). No static
    grants deathtouch to a player's I/S spell damage yet.
  - **Opponent activates a nonmana ability of an artifact/creature/land →
    ping** — Immolation Shaman. `EventKind::AbilityActivated` exists but there
    is no scope/filter for "source is an artifact/creature/land, nonmana."
  - **Tap N untapped creatures of a type as a cost** — Persistent Petitioners'
    "Tap four untapped Advisors you control: target player mills twelve" (only
    its `{1},{T}: mill 1` half would ship without this). Also its
    "any number of copies in a deck" deckbuild waiver.
  - **Land animation with haste that stays a land** — Clan Guildmage's second
    mode ("target land becomes a 4/4 Elemental with haste; still a land").
  - **Move a +1/+1 counter between your creatures** — Combine Guildmage's
    second ability + its "creatures enter with an extra counter this turn."
  - **Riot as a granted static** (Rhythm of the Wild) — riot currently only
    ships as an intrinsic ETB trigger, not a "nontoken creatures you control
    have riot" anthem; plus its "creature spells can't be countered."
  - **Opening-hand reveal → first-upkeep bonus** (Sphinx of Foresight) —
    approximated as a recurring upkeep scry 1; the reveal-from-opening-hand
    path (an `OpeningHandEffect`) isn't wired for the scry-3 rider.
  - **Spells targeting this cost {2} more for opponents** (Sphinx of New Prahv)
    — a self-referential targeted-spell tax static.
- ⏳ **Newly-noticed primitives (discovered during the DSK/BLB gap batch):**
  - **Gift on a permanent (creature/artifact)** — the gift's `gifted_effect`
    resolves only on the instant/sorcery spell path; a Gift *creature*
    (Scrapshooter, Starforged Sword) needs the permanent-ETB path to check
    `card.gift_promised` and run `gifted_effect` as the ETB.
  - **Forage / cost-hybrid mana abilities** — Thornvault Forager's
    "{T}, Forage: add two mana" wants a forage additional cost on
    `ActivatedAbility` (only cast-cost `Effect::Forage` exists today).
  - **Enchant-player auras + `PlayerStaticTarget::Enchanted`** — Grievous Wound
    ("enchanted player can't gain life; when dealt damage, lose half life"):
    no player-attaching aura support today.
  - **"You gave a gift" trigger** (`EventKind::GaveGift`) — Jolly Gerbils.
  - **Delirium-gated modal count** ("choose one; if delirium, choose one or
    more instead") — Let's Play a Game.
  - **Per-turn ability-resolution count** ("draw if this is the second time
    this ability resolved this turn") — Harvestrite Host.
  - **"No mana spent to cast" ETB gate** — Freestrider Commando's
    enters-with-two-counters (verify `ctx.mana_spent` is threaded to a
    self-ETB trigger before wiring; the plot/reanimate cases both want 0).
  - **Type/ability rewrite auras** ("becomes a colorless Food artifact with …,
    loses all other card types and abilities") — Sugar Coat.
- ⏳ **Deferred cards from the recent156-161 waves (each blocked on one
  primitive):**
  - **Two-target "your creature deals damage = power to their creature"** —
    Felling Blow. The `Selector::Target(0/1)` shape works (Hunter's Edge) but
    the per-slot you-control / opponent target filters aren't declared, so it's
    approximated; wants explicit multi-target-slot filters.
  - **Per-creature "prevent all combat/creature damage this turn" shield** —
    Fleeting Flight, Eerie Interference (fog scoped to one creature / player).
  - **Reflexive "discard N, then N targets get -2/-2"** — Miasma Demon links a
    variable discard count to a variable target count.
  - **"Your +1/+1-counter creatures have first strike during your turn"** —
    Inspiring Paladin's team clause (a PumpTeamIf gated on both a turn predicate
    and a per-creature counter filter).
- ⏳ **recent127-128 (OTJ/WOE) follow-ups / deferred:**
  - **Young Hero Role toughness gate** — the granted attack trigger fires
    unconditionally; the printed "if its toughness is 3 or less" wants a
    trigger-source toughness predicate.
  - **Boneyard Desecrator** — the effect-path sacrifice (`SacrificeAndRemember`)
    doesn't stamp `sacrificed_was_outlaw` (only the activated `sac_other_filter`
    path does); wire the tuple if a spell ever needs it.
  - **Cactarantula / Consuming Ashes** (OTJ) still need a control-a-Desert cost
    reduction and a target-mana-value reflexive predicate, respectively. (Aloe
    Alchemist ✅ via the new `EventKind::BecomesPlotted` trigger.)
- ⏳ **recent131-134 (WOE waves 4-7) follow-ups / noticed:**
  - New primitives this run: `DynamicPt::NonlandPermanentsControlled` (Regal
    Bunnicorn `*/*`), `Keyword::CantBeBlockedByPowerAtLeast(N)` (Squeak By —
    the fixed-threshold mirror of `CantBeBlockedByPowerAtMost`), and the
    enchantment-matters idiom (`PermanentDied`/`EntersBattlefield` +
    `EntityMatches { TriggerSource, Enchantment/Aura }` — Wicked Visitor,
    Savior of the Sleeping, Ashiok's Reaper, Rimefur Reindeer, Tanglespan
    Lookout). Role tokens (Sorcerer/Cursed/Royal/Wicked) reused via
    `CreateTokenAttachedTo`; the Wicked Role's death-drain needed the engine to
    collect **`PermanentDied`/`SelfSource`** leave-triggers for non-creatures
    (previously only `CreatureDied`/`PermanentLeavesBattlefield` were gathered —
    fixed in `stack.rs`). Also new: `Value`-free `MayPay` reflexives on ETBs
    (Unassuming Sage, Snaremaster Sprite).
- ⏳ **recent139 (WOE wave 12) noticed / deferred:**
  - **Gnawing Crescendo**'s "whenever a nontoken creature you control dies this
    turn, make a Rat" wants a delayed-death turn-scoped trigger sibling of
    `Effect::CreaturesYouControlEnteringThisTurn` (only the enters variant
    exists). The +2/+0 team-pump half is trivial once that lands.
  - **Eerie Interference** ("prevent all damage by creatures to you and your
    creatures this turn") wants a source-filtered scoped fog — the existing
    `PreventAllDamageThisTurn`/`PreventAllCombatDamageInvolving` don't gate on
    *dealer is a creature*.
  - **Expel the Interlopers** (destroy all creatures with power ≥ a chosen
    0–10) wants a dynamic power threshold in the destroy filter (filters take a
    fixed `i32`; the chosen number would need `PowerAtLeastValue`).
  - **Frantic Firebolt** approximates X = 2 + instant/sorcery cards in gy,
    dropping the "…or have an Adventure" graveyard contribution (no
    graveyard-card `HasAdventure` filter).
  - **Rotisserie Elemental** (skewer-counter impulse) and **Sentinel of Lost
    Lore** (exile-Adventure modal) still deferred. (Discerning Financier shipped
    — recent290.)
- ⏳ **Noticed in recent146-148 (approximations worth revisiting):**
  - **Back for Seconds** returns only one card to hand if bargained but the
    reanimation is declined (the "up to two total" cap models the
    battlefield-put as *replacing* the second return); faithful when you take
    the reanimation. A true "choose up to two targets, then optionally redirect
    one" would need a post-target redirect step.
  - **Faebloom Trick / Twisted Sewer-Witch-style "when you do" reflexive taps**
    are modeled as a plain `Effect::Seq` (targets chosen up front) rather than a
    CR 603.7 reflexive trigger.
  - **ManifestDread + attach** (Cursed Windbreaker) attaches to "a face-down
    creature you control" because `Selector::LastMoved` is clobbered by the
    dread's second card going to the graveyard after the manifest. A
    `Selector::LastManifested` (or having `ManifestDread` stamp the manifested
    id) would let "attach to that creature" be exact when multiple face-downs
    exist.
  - **Johann once-per-turn** is a per-player flag, so two Johanns still grant
    only one top-of-library cast per turn (each printed ability is independently
    "once each turn").
- ⏳ **recent113 (MH1 + Eldrazi) follow-ups / deferred:**
  - **Vorinclex, Voice of Hunger** — needs a "whenever you/an opponent tap a
    land for mana" trigger (no `EventKind` for tap-land-for-mana yet); the
    mana-doubling half + opponent "that land doesn't untap next" half both
    hang on it. Praetor cycle is otherwise complete.
  - **It That Betrays** — "whenever an opponent sacrifices a nontoken
    permanent, put that card onto the battlefield under your control": needs a
    sacrifice-watching trigger + LKI of the sacrificed card for a reflexive
    reanimation (no such event today).
  - **Void Winnower** X-spell corner: the even-MV cast lock reads the *printed*
    mana value, so an `{X}` spell counts as MV 0 (even) regardless of the
    chosen X. Faithful for fixed-cost spells; thread the announced X to be
    exact.
- ⏳ **Deferred (noticed, not tackled):**
  - **Mycosynth Lattice** — "all permanents are artifacts" fits
    `AddCardTypeToMatching`, but the all-colorless + spend-any-color halves
    have no primitives.
  - **Glimpse of Tomorrow / Emperor of Bones** — shuffle-permanents-and-
    redeploy; exiled-with + counters-added reflexive return.
- ✅ **MH2 sweep — COMPLETE.** `python3 scripts/set_gaps.py mh2` reports 0
  missing (the script now checks full split-card names and skips `A-`
  Alchemy rebalances). ~180 cards across `decks::mh2b`–`mh2i`. Remaining
  per-card approximations are noted on their factory docs (Garth's copy is a
  real hand card; Ghost-Lit Drifter's channel hits one target; Chef's Kiss
  keeps the target when no hostile retarget exists).
- ⏳ **All Will Be One placer attribution** — `GameEvent::CounterAdded` carries
  no "who placed" seat, so the enchantment fires off counters landing on your
  permanents + poison hitting opponents (exact in two-player). Threading the
  placing controller through the counter funnel would make it exact in
  multiplayer and unlock "whenever an opponent puts a counter…" designs.
- ⏳ **Rhuk's dies-half** — "equipped creature … attacks or dies"; the dies
  half needs the victim's attachment list snapshotted before the equipment
  unattaches (LKI for attachments).
- ⏳ **CastWithoutPayingImmediate copy-mode** — Capricious Hellraiser should
  cast a *copy* (original stays exiled); a `copy: bool` rider on the effect
  would also serve future "copy it and you may cast the copy" cards.

- ⏳ **recent34–38 follow-ups / deferred cards (this run):**
  - Approximations still to revisit: Pir's Whim (full friend/foe vote →
    you=friend/opponents=foe), Three Dreams (different-names search dropped).
    ✅ Gather the Pack (spell mastery's 2nd creature via `Effect::MillThenToHandN`
    + `Value::IfAtLeast` over I/S in gy), ✅ Hour of Promise (3+ Deserts Zombies),
    Golden Demise
    (Ascend + city's-blessing opponents-only), Yahenni's Expertise (MV≤3 free
    cast via `CastFromHandWithoutPaying`), and Goblin Assault ("Goblins attack
    each combat" via `GrantKeyword(MustAttack)`) are now faithful.

- ⏳ **recent31 (multicolor staples) follow-ups / deferred cards:**
  - Dimir Charm mode 3 ("look at top three, put one back, rest into graveyard")
    is modeled as **mill 2** — wants a look-top-N-keep-one-rest-to-graveyard
    effect (a target-player surveil-to-graveyard variant).
  - Atarka's Command mode 3 ("put a land from your **hand**") reuses
    `PutFromHandOrGraveyardOntoBattlefield` (also allows graveyard) — wants a
    hand-only put primitive.
  - Foul-Tongue Invocation drops the "reveal a Dragon from hand" additional
    cost; the bonus 4 life is gated on controlling a Dragon instead.
  - **Deferred — need new primitives:** Necropolis Fiend ({X},{T}, exile X from
    gy: −X/−X — activated abilities have no `{X}` cost + Value-count gy-exile);
    Bonehoard (living-weapon equip
    +X/+X where X = creature cards in all graveyards — `EquipScale` counts
    battlefield permanents, not graveyards); Dromoka's Command (mode "prevent
    all damage target instant/sorcery would deal" — no prevent-spell-damage
    effect); Pyromancer Ascension (quest-counter + spell-copy enchantment);
    Crime // Punishment (split card).
- ⏳ **recent20 (OTJ) approximations / follow-ups:** (✅ Magda, the Hoardmaster's
  "Sacrifice three Treasures: make a 4/4 Scorpion Dragon" now ships via
  `sac_other_filter: Some((Treasure, 3))`.) Gisa's "Ward—{2}, Pay 2 life" is
  modeled as Ward—{2} (the life half is dropped — `WardCost` has no compound
  variant); Bovine Intervention mints the Ox before the destroy so
  `ControllerOf(Target)` still resolves. Also: CR 700.13 crime detection covers
  cast + activated-ability targets but not a *triggered* ability that targets an
  opponent's stuff as it's put on the stack, nor targeting a spell/ability an
  opponent controls beyond stack *spells* (abilities on the stack aren't
  checked). **Spree** (Lively Dirge) still ⏳ — multi-chosen additional costs.
- ⏳ **recent21 approximations:** Trick Shot drops the "2 to another target
  creature token" rider (just 6 to a creature); Patient Naturalist drops the
  "else create a Treasure" when no land is milled. Stingerback Terror / Canyon
  Crab / Bedrock Tortoise deferred (card-in-hand-scaled CDA P/T, a "didn't cast
  from hand this turn" flag, and assigns-damage-by-toughness, respectively).
- ⏳ **recent17–18 (Foundations) approximations to revisit:** Kitsa, Otterball
  Elite drops the "{2},{T}: copy target instant/sorcery you control" ability
  (needs a copy-spell activated ability gated on power ≥ 3); Run Away Together
  is modeled as any-two-creatures bounce (the "controlled by different players"
  restriction isn't enforced); Sky Crier / Dryad Greenseeker approximate
  "put into hand" as a draw; Angel of Finality / Burglar Rat target each
  opponent rather than a chosen player (1v1-faithful). (✅ Charmed Sleep ships as
  a `tap_down_aura` — ETB tap + `PreventUntap` on the host.)
- ⏳ **Cards noticed this run (recent12–15) but deferred — need new primitives:**
  Kutzil's Flanker (mode 1 wants a "creatures that left the battlefield under
  your control this turn" count); Caustic Bronco (attack-reveal life-loss/drain
  equal to the *revealed card's* mana value — a Value reading a just-revealed
  card); Mosswood Dreadknight (cast-from-graveyard-as-Adventure death rider);
  Ao, the Dawn Sky (dies-modal "look top 7, deploy nonland permanents with total
  MV ≤ 4"); Gix, Yawgmoth Praetor (combat-damage "may pay 1 life: draw" +
  discard-X exile-and-play); Valgavoth (opponent-graveyard-exile replacement +
  play-from-exile-paying-life); Battle Cry Goblin (Pack tactics — "if you
  attacked with total power ≥ N this combat"); Goblin Recruiter (search any
  number + arrange on top); Divergent Transformations / Seeds-cycle's last
  Undaunted card (polymorph-reveal-until-creature).

- ⏳ **Equipment-matters follow-ups** (`decks::recent12`): a
  **token-exile-at-next-end-step** delayed trigger (Valduk's Elementals
  currently persist); Nahiri's −8 currently drops the deployed permanent's
  haste + return-to-hand rider (search-to-battlefield only). Bruenor
  Battlehammer needs a per-creature "+2/+0 per Equipment attached to *it*"
  anthem + a free-first-equip-each-turn allowance. (While-equipped team anthems
  + conditional keyword-by-attached-count ✅ — Auriok Steelshaper, Balan.)

> **Reprioritized 2026-06-11:** the correctness-audit section at the top of
> this file outranks everything below. New-card/primitive work should wait
> behind at least the audit P0 tier (and the P3 root-cause refactors, which
> make every subsequent card batch safer to land).

- ℹ️ **Client builds headless once dev libs are installed.** `apt-get install -y
  libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev` lets
  `crabomination_client` (Bevy) compile + clippy + `--no-run` its tests in the
  remote/headless env (the wayland-sys/alsa-sys/libudev/xkbcommon build scripts
  just need the `.pc` files). Runtime/GPU verification still needs the local
  `verifier-client` skill. Keyword chips + tooltips for the new protection
  keywords are compile-checked here.
- ⏳ **recent2 card approximations** (`decks::recent2`, all noted in doc
  comments): March of Otherworldly Light drops the "exile white cards to reduce
  cost" rider; Conduit of Worlds ships only the play-lands-from-graveyard static
  (not the {T} cast-from-graveyard half); Lord Skitter's Rat-ETB exiles a card
  rather than "up to one target"; Llanowar Greenwidow drops the Domain cost
  reduction + the exile-if-it-would-leave rider. Newer wave: Sunfall's Incubate
  now ships (`Effect::Incubate`, CR 701.53); Ossification is modeled as a standalone O-Ring (no enchant-a-basic
  rider); Steamcore Scholar drops the "unless you discard an I/S or flyer"
  reprieve; Subterranean Schooner explores any creature you control (not
  specifically the one that crewed it); Gathering Throng searches up to three;
  Hexgold Slith drops the optional pay-{E}-for-first-strike attack ability.
- ⏳ **Noticed this run (recent2 MOM/WOE/OTJ wave):** real-card primitives still
  missing — a Value for "noncreature spells a player cast this
  turn" (Magebane Lizard); **Spree** multi-additional-cost casting (Phantom
  Interference, Three Steps Ahead); chosen-card-type cost reduction (Stenn);
  cast-from-an-opponent's-graveyard on combat damage (Tinybones). Warren
  Warleader needs a "create a tapped, attacking token" mint + a "whenever you
  attack" (declare-attackers) trigger distinct from `Attacks/SelfSource`.

- ⏳ **Haunt / Ripple / Unearth follow-ups** (shipped this push):
  - Haunt's haunted-creature is auto-picked (prefers an opponent's) and the
    exile-haunting is modeled as a `route_to_graveyard` replacement, not a real
    targeted stack trigger — add a controller choice + a proper trigger.
    Combat-damage haunt (Souls of the Faultless) is unmodeled.
  - Ripple's free-cast prompts go through `Decision::OptionalTrigger`; the
    "Spells you cast have ripple N" static (Thrumming Stone) isn't wired.
  - Unearth models only the end-step exile, not "exile it if it would leave the
    battlefield" (same gap as Goryo's). A client affordance to surface
    graveyard-activated abilities (the bot already offers them) is missing.
  - Card approximations: Surging Æther's "target spell or permanent" → creature;
    Surging Sentinels' protection-on-white-cast rider dropped.

- ⏳ **Missing keyword mechanics:** Sunburst-on-noncreature charge counters (the
  +1/+1 creature path ships via `Value::ConvergedValue`). (Haunt ✅ —
  `Effect::HauntCreature`; Ripple ✅ — `shortcut::ripple`/`Effect::Ripple`.)

- ✅ **Mutate (CR 702.140).** Shipped: `CardDefinition.mutate: Option<ManaCost>`,
  `GameAction::CastMutate { card_id, target, on_top }`, `CardInstance.mutate_stack`
  (component cards top-to-bottom; live `definition` = union of the top card's
  characteristics + every card's abilities), `EventKind::Mutated` /
  `GameEvent::Mutated`, leave-the-battlefield scatter (all three meld sites), and
  snapshot round-trip (union rebuilt on load). Cycle: Glowstone Recluse,
  Trumpeting Gnarr, Cubwarden, Cavern Whisperer, Dirge Bat, Migratory Greathorn,
  Boneyard Lurker, Pollywog Symbiote (`HasMutate` filter), Vulpikeet, Majestic
  Auricorn, Sawtusk Demolisher, Gemrazer, Insatiable Hemophage, Chittering
  Harvester, Regal Leosaur, Cloudpiercer, Sea-Dasher Octopus, Essence Symbiote,
  Porcuparrot (`Value::MutateCount`), Archipelagore (`Effect::TapUpToValue` —
  dynamic-count resolution-time picker). Tests in `tests/modern.rs`. Follow-ups:
  - ⏳ **Client cast-mutate UI + `mutatable` affordance** (host picker). Engine
    path is fully wired and tested; only the UI is missing.

- ⏳ **Ikoria cards deferred (need new primitives or are complex):**
  - **IKO walkers still missing:** Narset of the Ancient Way (restricted-mana +1
    spendable only on noncreature spells + discard-linked damage; −6 emblem) and
    Lukka, Coppercoat Outcast (+1 exile-top-3 with conditional cast-from-exile;
    −2 reveal-until-greater-MV deploy). Vivien, Monsters' Advocate now ships
    (cast-from-top static, +1 token+keyword-counter, −2 lesser-MV tutor via the
    new next-spell `event_amount` wiring).
  - ✅ **Winota, Joiner of Forces** — `Effect::LookTopMayDeployAttacking`
    (look top six, deploy a Human creature tapped-and-attacking with
    indestructible EOT, bottom the rest; auto-picks highest power). Test
    `winota_deploys_human_when_nonhuman_attacks`. Remaining ⏳: a `wants_ui`
    picker (currently auto-pick) and the "up to one" decline.
  - ✅ **Memory Leak** — `Effect::ExileChosenFromHandOrGraveyard` (cross-zone
    exile of a nonland from the target's hand or graveyard; auto-picks highest
    MV) + Cycling {1}. Test `memory_leak_exiles_highest_mv_across_zones`.
    Remaining ⏳: a `wants_ui` chooser (currently auto-pick).
  - **Other complex IKO holdouts** (next-run candidates):
    Kinnan (tap-for-mana doubling + big-creature dig), Quartzwood's faithful
    "any trampler you control" batch trigger, Sea Serpent
    (can't-attack-unless-defender-has-Island + sac-if-no-Islands), Titans' Nest
    (surveil + restricted exile-for-mana).
  - **Brokkos, Apex of Forever** ships with mutate+trample; the "cast from
    graveyard using its mutate ability" rider is dropped — `cast_mutate` only
    reads the hand. A `mutate_from_graveyard` flag + a graveyard cast path
    (mirroring `cast_escape`) would finish it.
  - **Glimpse the Cosmos** ships the dig-3-take-1; the "cast from graveyard
    while you control a Giant" rider is dropped — needs a board-conditional
    graveyard-cast permission (conditional flashback) primitive.
  - Client keyword label/tooltip arms for `ProtectionFromManaValueParity` are
    compile-verified — `crabomination_client` now builds headless via the
    pkg-config + linker shim recipe above (rustc 1.95, `LIBRARY_PATH=/tmp/pc`).
    Runtime/GPU verification still needs the local `verifier-client` skill.
  - **Approximations shipped this run** (dropped riders, all noted in the card
    doc comments): Gust of Wind / Tentative Connection / Mythos of Brokkos's
    "spent {X}{Y}" upgrades (no mana-provenance-by-color spend-tracking yet);
    Mythos of Nethroi's {G}{W} upgrade; Parcelbeast's "you may" on the land.

- ✅ **Catalog-wide stat sweep (2026-06-16) — same problem beyond STX.** The
  modern supplement (`decks/`, `mod_set/`) and small older sets carried the same
  synthesized-stat drift. New tooling `scripts/audit_catalog_stats.py` (cost +
  P/T + creature-type + keyword, all sets) and `scripts/fix_catalog_stats.py`
  (cost/P-T/type fixer with a custom-card exclude list) drove a sweep across
  `decks`/`mod_set`/`ths`/`kld`/`ktk`/`lea`/`dis`/`khm`/`sos`, regenerating coupled
  tests via `fix_test_mana.py` + `regen_test_assertions.py`. Catalog-wide drift:
  **cost 253→2, P/T 131→6, type 120→8, keyword 55→41** (full suite green, 8551).
  Lessons baked into the tooling: cost rebuilds use the *front* face (don't sum
  split halves), the cost field is found as the depth-1 `CardDefinition.cost`
  (never a nested `mana_cost:`/deferred-Pact/`GrantMiracle` cost), and the keyword
  audit reads only the top-level vec. **Keyword pass:** 13 clear simple-keyword
  bugs fixed (spurious/wrong/missing — e.g. Shriekmaw Menace→Fear, Mockingbird
  Flash→Flying, Loot +Double-strike/Haste); the other ~41 are deliberately left —
  conditional keywords modeled as base (Paradise Druid's untapped-only hexproof),
  keywords that *model an evasion ability* (Silhana/Signal Pest "blocked only
  by…" as Flying, Reality Smasher/Frost Titan counter-tax as Ward), DFC
  back-face keywords, Protection/Ward that need a quality/arg, and manlands
  (Mutavault). Those need real ability modeling, not a stat tweak. **Other
  remaining:** cost/PT/type leftovers are CDA P/T, the 3 synergy-coupled
  synthesized types, missing enum variants, and the 2 excluded customs (Cosmogoyf,
  Crabomination). Run `python3 scripts/audit_catalog_stats.py` for the live table.
- ⚠️ **Fabricated real-name STX cards (correctness sweep).** Many STX factories
  reuse *real* STX card names but carry invented cost/types/oracle text (the
  synthesizer collided with real names). **Cost + P/T are now fully swept**:
  `scripts/audit_stx_drift.py` reports 0 cost/PT drift across the whole `stx/`
  tree (148 mana-cost literals + 61 power/toughness literals corrected to the
  Scryfall cache this run, doc-comment titles synced via
  `scripts/fix_doc_costs.py`, coupled test fixtures rewritten via
  `scripts/fix_test_mana.py`). Re-run `python3 scripts/audit_stx_drift.py` to
  keep it at zero after adding cards.
  ✅ **Type-line + keyword sweep (2026-06-14/15).** `audit_stx_drift.py` only
  checks cost + P/T; it never inspects type line or keywords. Added
  `scripts/audit_stx_types.py` to cover those (top-level keyword field only, so
  it skips conditional/granted keywords nested in statics/equip-bonuses/tokens).
  Against a freshly-refetched real Scryfall cache it found **49 creature-type +
  ~15 real keyword drifts**. Fixed: **47 creature types** + **20 keywords**
  (Mavinda Cleric+Vigilance → Bird Advisor+Flying; Beledros Demon+Trample/Lifelink
  → Elder Dragon+Flying; Galazeth → Elder Dragon; Disciplined Duelist FirstStrike
  → DoubleStrike; Codespell Cleric → Vigilance; Spectacle Mage Prowess → Flying;
  Inkfathom Witch → Fear; Inkfathom Divers → Islandwalk; Lone Rider → First
  strike+Lifelink; etc. — two coupled tests in `tests/stx/part_25.rs` updated,
  one Intimidate test in `tests/modern.rs` given a Reach blocker). Full suite
  green (8551). Audits now clean except:
  - **3 creature types** — Eyetwitch, Quandrix Pledgemage, Silverquill Pledgemage
    are synthesized cards whose Pest/Fractal/Inkling *synergy tests* depend on the
    wrong type (retyping breaks the tests; needs card + test reworked together).
    (Eccentric Apprentice fixed — added `CreatureType::Tiefling`.)
  - **1 keyword** (Lone Rider) — a benign DFC artifact: the modeled front face is
    correct (First strike+Lifelink); the flagged Trample is the *back* face only.
  Note: several conditional/granted keywords were left as the catalog already
  models them correctly via statics (Leech Fanatic's your-turn lifelink, Sticky
  Fingers' aura-granted menace, Silverquill Pledgemage's magecraft flying) — the
  audit no longer flags those. Many fixed cards are fabricated-real-name
  collisions whose **bodies are still synthesized**; a correct stat block ≠ a
  faithful card.
  **Effect-body sweep complete**: Hofri Ghostforge, Fervent Mastery, and
  Strixhaven Stadium (point counters + ten-point `Effect::LoseGame`) are now
  faithful. ✅ this run: **Stonebinder's Familiar**
  (`EventKind::CardExiled` once-per-turn-during-your-turn trigger, retyped Spirit
  Dog), **Confront the Past** (faithful 2-mode: reanimate gy PW + remove 2X
  loyalty from an opp PW — the "MV X or less" reanimation gate is dropped, no
  X-aware MV target filter yet). Per card:
  replace the body with the Scryfall text and rewrite its test(s); watch for
  fixture coupling. Swept faithful this run: **Mage Duel** (+1/+2 then fight),
  **Tempted by the Oriq** (permanent MV≤3 steal), **Mentor's Guidance**
  (conditional copy-on-cast + scry/draw), **Bayou Groff** (Plant Dog 5/4 +
  sacrifice-a-creature additional cost; pay-{3} alternative dropped). Confirmed
  already-faithful (stale notes): Frost Trickster (Bird Wizard, ETB tap+stun),
  Eager First-Year (magecraft self-pump), Owlin Shieldmage (Flying + Ward 3
  life), Promising Duskmage (death-draw if +1/+1 counter).
  Bayou Groff is now faithful — `AdditionalCastCost::SacrificeOrPay`
  auto-sacrifices when a match exists, else folds the pay into the cost.
- ✅ **Remaining real STX (Strixhaven 2021) cards — complete.** A Scryfall
  `set:stx` diff vs the registered catalog now shows 0 unimplemented
  non-Arena cards (the last 13 — Deans, Culling Ritual, Professor Onyx,
  Zimone, … — existed but weren't registered; the crate-wide generated
  factory list closed that, and Zimone's fabricated body was rewritten
  faithful). Historical note: this run previously added the
  single-faced **Efreet Flamepainter** (`CastWithoutPayingImmediate` from gy on
  combat damage), **Thunderous Orator** (conditional keyword-share via
  `If` + `Predicate::SelectorExists`), **Venerable Warsinger** (combat-damage
  reanimation, MV gate fixed at 3), and **Ardent Dustspeaker** (impulse-draw
  two on attack; the gy-to-bottom enabler dropped). Still unimplemented,
  grouped by the primitive they're blocked on:
  - **Study / hone counters** — Kianne/Imbraham, Uvilda/Nassari Deans.
  - ✅ **Entered-this-turn filter** (`SelectionRequirement::EnteredThisTurn`,
    `CardInstance.entered_turn` stamped at every ETB via the dispatcher) —
    ships **Shaile // Embrose**, the first Dean MDFC. **First Day of Class** is
    also done (its own turn-scoped `Effect::CreaturesYouControlEnteringThisTurn`
    delayed trigger, CR 603.4).
  - **MDFC legends** — Codie/Extus/Blex/Jadzi + the rest of the Dean cycle.
  - ✅ **Group land-search** — `Effect::CatchUpBasicLands` (each player behind
    the land leader fetches basics up to the deficit, tapped, then shuffles).
    Ships Scholarship Sponsor.
  - **Variable-number-of-targets** — Ecological Appreciation ("up to four with
    different names" + opponent-chooses-two split).
  - ✅ **Draconic Intervention** — shipped via new
    `AdditionalCastCost::ExileFromGraveyard { filter }` (exiles a gy card, its MV
    becomes the spell's X) + `ExileIfWouldDieThisTurn` for the "exile instead"
    rider.
  - **Single-faced, still blocked**: Codie (can't-cast-permanents static +
    when-you-next-cast reflexive discover — needs a new delayed-trigger kind).
    ✅ Elite Spellbinder (`Effect::ExileFromHandTaxed` — exile a nonland from an
    opp's hand; owner may play it for +{2} while exiled; cost bug {1}{W}{B} →
    {1}{W}{W} fixed). Radiant Scrollwielder already ✅.
  Diff `set:stx` Scryfall names against the catalog string literals (note:
  helper-built names like the Snarl cycle are passed as `name` params, so
  grep the whole file, not just `name: "…"`).
- ✅ **Variable-X loyalty abilities** (CR 606.5) — `LoyaltyAbility.x_cost: bool`
  (Default-derived; literals migrated). `ActivateLoyaltyAbility { x_value }`
  threads the chosen X; `activate_loyalty_ability` clamps X to current loyalty,
  spends X, and stacks the effect with `x_value: X` so the body reads
  `Value::XFromCost`. Kasmina's -X Fractal is now faithful. Remaining ⏳: a
  `Decision::ChooseAmount` UI prompt for X (the bot commits full loyalty; the
  client doesn't yet build the loyalty action). Sorin/Saheeli -X ultimates can
  now reuse the same `x_cost` path.
- ✅ **`Effect::PayManaOrElse { mana_cost, otherwise }`** (this run) —
  the mana sibling of `PayEnergyOrElse`; pays from the floating pool when
  able, else runs the fallback (Archway Commons' "sacrifice unless pay
  {1}"). Remaining ⏳: a `wants_ui`/bot mid-resolution pay prompt (today a
  bot with no floating mana always takes the fallback, same limitation as
  `MayPay`).

- ⏳ **Discovered during the Eldrazi/devoid pass (not yet done):**
  - **Generalize variable-power CDA** (`*/N` from a count). Tarmogoyf, Vile
    Aggregate (`DynamicPt::ColorlessCreaturesControlled`, shipped this run),
    etc. are each a name-keyed row in `dynamic_pt_for_name`; a
    `Modification::SetPowerToughness` fed directly by a `Value` would drop the
    per-card name table entirely (e.g. Walker of the Wastes = lands named
    Wastes you control).
  - ✅ **"Defending player exiles N permanents they control"** (opponent-chosen)
    — `Effect::PlayerExilesPermanents { who, count, filter }`; the exile
    analogue of Annihilator's forced sac. Ships Bane of Bala Ged. The affected
    player auto-picks the weakest N; a human-defender chooser (a UI suspend
    like the Sacrifice path) is the remaining follow-up.
  - ✅ **Devoid-aware `Colorless` filter.** `SelectionRequirement::Colorless`
    now treats `Keyword::Devoid` as colorless (CR 702.114 CDA) at every static
    eval site (`eval.rs` ×2, `layers.rs`), so Devoid creatures with colored
    pips count for colorless-matters triggers/filters. Exercised by Flayer
    Drone (drains on a Devoid creature entering). Full color-setting effects
    (rare type/color changers) still read cost pips — a deeper follow-up.
- ⏳ **Discovered this run (modern_decks card pass), not yet done:**

- 🟡 **Energy ({E}) follow-ups.** (b) **✅ "pay {E}{E} or sacrifice/bounce"
  rider** — `Effect::PayEnergyOrElse { amount, otherwise }` ships Lathnu
  Hellion (sac) and Greenbelt Rampager (bounce). (c) **✅ EnergyGained trigger
  event** — `EventKind::EnergyGained` (CR 107.16) fires "whenever you get one
  or more {E}"; Aetherborn Marauder wired. (d) **✅ damage→energy feedback** —
  Harnessed Lightning (deal 3; get {E}{E}{E} if it hit a permanent). (a)
  **✅ energy-gated mana abilities** — `ActivatedAbility.energy_cost` (CR
  107.16) gates an ability on {E}, spent up front like the mana/life
  pre-pay; Aether Hub (`{T}: Add {C}` + `{T}, Pay {E}: Add any color`) and
  Servant of the Conduit are now faithful. The affordance/bot paths gate via
  `would_accept`, so unpayable energy abilities are auto-excluded.

- ✅ **`ActivatedAbility` `..Default::default()` sweep + `remove_counter_cost`.**
  Swept the ~220 remaining full-field literals to `..Default::default()` and
  added `remove_counter_cost: Option<(CounterType, u32)>` (CR 602.5b "Remove a
  [kind] counter from this:") as a real cost paid in `activate_ability` before
  the effect goes on stack. Walking Ballista / Triskelion now pay the counter
  as a cost (can't be over-activated off the stack); test
  `walking_ballista_counter_is_a_real_cost_not_overactivatable`.

- ⏳ **Future batch — focus on engine-feature-unlocking cards**: priority
  candidates are Helix Pinnacle (keyword counter), Walking Ballista
  (Nth-counter trigger), and cards that exercise CR 122.4 (counter cap)
  / 122.7 (Nth-counter threshold trigger). Each lands new engine
  capability tracked in the rules-audit section above.

- 🟡 **CR 119.7 — "Can't gain life"** (push modern_decks claude/modern_decks
  branch). The gain-life half of CR 119.7 is now wired via the new
  `StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget }`
  primitive + the `player_cannot_gain_life_now(seat)` helper called
  from `GameState::adjust_life`. The `Player.cannot_gain_life: bool`
  flag is also exposed (set by emblems / future grant effects but
  currently dormant); `adjust_life` ORs the dynamic battlefield check
  with the cached flag. Witherbloom Lifeglobe (b143) ships the
  "Your opponents can't gain life" static; lock-in tests
  `witherbloom_lifeglobe_b143_prevents_opp_lifegain`,
  `witherbloom_lifeglobe_b143_releases_lifegain_lock_when_it_leaves`.
  The lose-life half (CR 119.8) is also ✅ — `StaticEffect::
  PlayerCannotLoseLife { target }` + `player_cannot_lose_life_now(seat)`
  drops negative deltas in `adjust_life` (covering both `Effect::LoseLife`
  and the damage path). Silverquill Lifeward (b146) ships "Your opponents
  can't lose life"; tests `cr_119_8_player_cannot_lose_life_blocks_lose_life_paths`,
  `cr_119_8_player_cannot_lose_life_blocks_burn_damage`. Remaining ⏳: (b)
  the redistribute-life-totals clause (CR 119.7, last sentence) still wants a
  `Effect::DistributeLifeTotals` check. **Exchange-life-totals already respects
  the lock** — `Effect::ExchangeLifeTotals` routes through `adjust_life_applied`,
  so the gaining half is dropped for a can't-gain player (test
  `cr_119_7_exchange_life_totals_respects_cant_gain_life`). (c) Tainted Remedy's
  "instead, that player loses that much life" replacement is now ✅ via
  `StaticEffect::LifeGainBecomesLoss` + `life_gain_becomes_loss_now`
  (redirects positive deltas in `adjust_life`; Silverquill Reproach b209;
  test `cr_614_life_gain_becomes_loss_for_opponent`).

- ⏳ **Damage-source choice primitive (CR 120.7)** (push
  claude/modern_decks batch 119 — new suggestion, paired with the new
  CR 120.7 audit row). The current `Effect::DealDamage` path threads
  `ctx.source` correctly, but the catalog has no spells / abilities
  that ask the controller to *choose* a source of damage (Browbeat,
  Burning of Xinye, Vendetta-style "deal damage equal to source's
  power"). A `Selector::ChosenSourceOfDamage { filter }` plus a
  `DecisionKind::ChooseSource` decision-point would unblock these.
  Engine-wide ⏳; low priority since no current STX/SOS/cube card
  needs it.

- 🟡 **Copy-token primitive** — `Effect::CreateTokenCopyOf { who, count,
  source, extra_creature_types, override_pt }` ships the token-copy half
  (Cackling Counterpart-style), and `Effect::BecomeCopyOf` ships the
  enter-as-a-copy half (Clone, Phantasmal Image, Mockingbird). Both carry
  `extra_creature_types`; the token variant also has `override_pt`.
  Remaining: a *continuous* layer-1 "becomes a copy" effect (Helm of the
  Host's per-combat haste-token loop, Mirrorform aura) — these still need a
  layer-1 copy effect rather than the one-shot definition rewrite.


- 🟡 **`effect::shortcut::magecraft_loot()` callsite reduction** (push
  claude/modern_decks batch 107 — partial pass). Eight inline
  `magecraft(Seq([Draw 1, Discard 1]))` callsites across `stx::prismari`
  (3) and `stx::quandrix` (5) collapsed onto the existing
  `magecraft_loot()` helper. Remaining ⏳ inline callsites may still
  exist in `stx::extras` and other set modules — future cleanup pass
  can run the same regex sweep there.

- ⏳ **Transient triggered-ability grant primitive** (push
  modern_decks batch 47 — new suggestion). Several STX/SOS cards
  print "until end of turn, each [creature] you control gains
  [trigger]" — e.g. SOS Root Manipulation ("creatures you control
  get +2/+2 and gain menace and 'Whenever this creature attacks,
  you gain 1 life.' until end of turn") and Rabid Attack ("any
  number of target creatures each get +1/+0 and gain 'When this
  creature dies, draw a card.' until end of turn"). The engine has
  no primitive that grants a trigger for a duration; today these
  riders are dropped (the body half ships, the trigger-grant
  half doesn't). Wiring shape: a new `Effect::GrantTriggeredAbility
  { what: Selector, trigger: TriggeredAbility, duration: Duration }`
  primitive that injects a transient trigger onto each matched
  permanent (stored alongside `granted_keywords_eot` for cleanup
  per CR 614.7c). Cards unblocked: Root Manipulation, Rabid Attack,
  plus future "gain 'attack-trigger gain life'" / "gain 'dies-draw'"
  patterns.

- ⏳ **Permanent-copy primitive** (push modern_decks batch 47 —
  new suggestion). Multiple STX/SOS cards print "create a token
  that's a copy of target X" (Echocasting Symposium, Applied
  Geometry, the Colorstorm Stallion / Elemental Mascot "if 5+
  mana spent, create a token that's a copy of this" Opus halves).
  Today these collapse to a vanilla token mint. Engine needs a
  `Effect::CreateCopyToken { what: Selector, modifier: Option<TokenModifier> }`
  primitive that copies the chosen permanent's printed
  characteristics (P/T, types, abilities) into a fresh
  `TokenDefinition` at resolution time. The `modifier` field
  would carry the optional "except it's also a Fractal" /
  "except its base P/T is 4/4" overrides per the printed cards.
  Cards unblocked: Echocasting Symposium, Applied Geometry,
  Colorstorm Stallion (big-body), Elemental Mascot (big-body),
  any future Saheeli / Sublime Epiphany permanent-copy mode.

- ⏳ **Layered-effect `Effect::GrantKeyword` for `UntilNextTurn`** —
  The batch-24 fix above honors `EndOfTurn` and `Permanent` durations.
  `UntilNextTurn`/`UntilYourNextUntap` is wired to permanent mutation
  (no cleanup), which is incorrect. Needs a separate `granted_keywords_
  untilnext: Vec<Keyword>` slot or routing through the proper layered
  system. No STX/SOS card uses this duration today, so the gap is
  doc-tracked but unaddressed.

- ⏳ **Batched sacrifice picker for cost-paid filters** (push
  modern_decks batch 18 suggested) — `Effect::Sacrifice { filter, …}`
  works for the post-resolution sac (Witherbloom Pestkeeper's
  activation step uses it). The cost-paid sac branch (the engine's
  `sac_cost: true` field on `ActivatedAbility`) is a single source-only
  sac and doesn't expose a filter. Wiring shape: extend the activation
  cost field to optionally carry a `SelectionRequirement` filter that
  drives the cost-time fodder picker, so cards like Pestkeeper can
  declare "sac a Pest you control" as a *cost* (rejecting activation
  without a Pest) rather than as the first step of the effect
  (resolves even if no Pest exists). Today's resolve-time filter is
  permissive — if no Pest is available, the sac step is skipped and
  the -2/-2 still resolves.

- ⏳ **`Predicate::CastFromZone(zone)`** (push modern_decks batch 18
  suggested) — the just-landed `CastFromHand` / `CastFromGraveyard`
  pair covers the hand/gy split, but a generalised `CastFromZone(Exile)`
  / `CastFromZone(Library)` is still ⏳. Threading shape: stamp a
  `cast_zone: Zone` field on `CardInstance` alongside `cast_from_hand`
  + propagate to `EffectContext.cast_zone` via
  `for_spell_with_source`. Future Cascade / Suspend / Flashback-from-
  exile riders ("if cast from exile, …") would key off this.

- ⏳ **Inkling / Pest tribal completeness** (push modern_decks
  current): with the 22-card extras drop the Silverquill Inkling pool
  now has 1+/+1 lord support, lifelink fliers, drain payoff, and
  artifact drain. The Witherbloom Pest pool similarly has token
  spawners + a destroy-plus-Pest sorcery + a 2-Pest ETB body. A
  cross-college BG/WB sealed pool could lean into these new shells.
  Slot into the SoS Silverquill / Witherbloom pool selector once the
  decklist generators support tribal weighting.

- ⏳ **Spirit-tribal Lorehold archetype** (push modern_decks): the new
  Spirit Banner (+1/+1 anthem for Spirits) joins Quintorius's
  pre-existing Spirit lord and the Lorehold token chain (Sparring
  Regimen, Lorehold Excavation, Quintorius). With this in place,
  a Spirit-tribal Lorehold variant deck could lean into the
  Sparring-Regimen-attack → counter rain → anthem combo. Slot it
  into the SoS Lorehold pool selector.

- ⏳ **Inkling-tribal Silverquill archetype** (push modern_decks): the
  new Quartzwood Inkling + Inkwell Strider + Inkling Studies join the
  pre-existing Tenured Inkcaster tribal anthem and Felisa Fang of
  Silverquill's Inkling generator. With at least 5 distinct Inkling
  minters and a +2/+2 lord in the catalog, a Silverquill Inkling
  tribal pool is now viable.

- ⏳ **`SelectionRequirement::ManaValueAtMostX`** (push modern_decks
  batch 39 suggested) — the current `ManaValueAtMost(u32)` predicate
  takes a compile-time constant, but several STX/SOS cards print
  "mana value X or less" gates where X is the spell's cast-time X
  (Mind into Matter's "put a permanent with mana value X or less
  from your hand onto the battlefield tapped"). Wiring shape: add a
  new variant that reads `EffectContext.x_value` at evaluation time,
  same as `Value::XFromCost` reads it for damage / counters / draws.
  The evaluator (`evaluate_requirement_static` in
  `game/effects/eval.rs`) would need to thread the X value through,
  same way it threads `source` today. Cards unblocked: Mind into
  Matter, future X-cost search-and-cheat-onto-battlefield primitives.

- ⏳ **Refactor existing STX/SOS Silverquill drain creatures to use
  `etb_drain`/`etb_gain_life`** (push modern_decks batch 39 suggested)
  — the new `effect::shortcut::etb_drain(N)` and
  `effect::shortcut::etb_gain_life(N)` helpers (added in batch 39)
  collapse the canonical 7-line ETB drain / gain-life trigger into
  one helper call. ~40 existing cards across `stx::silverquill`,
  `stx::witherbloom`, and `stx::lorehold` (Silverquill Marshal,
  Silverquill Loremender, Silverquill Drainmaster, Inkling Scriptwarden,
  Inkling Pamphleteer, Lorehold Skydefender, etc.) inline the same
  pattern manually. A future cleanup pass should refactor them to
  reduce code duplication; functional behavior is unchanged.

- ⏳ **"Tap N creatures as additional cost" cost primitive** (push
  modern_decks batch 39 noted) — Group Project's Flashback cost is
  "Tap three untapped creatures you control" (no mana cost), which
  doesn't fit the existing `AlternativeCost { mana_cost,
  exile_from_graveyard_count, ... }` shape. Wiring shape: extend
  `AlternativeCost` with `tap_count: Option<(u32, SelectionRequirement)>`
  so a cost-paid validator can require N permanents matching the
  filter to be untapped + tap them as the spell finishes paying.
  Cards unblocked: Group Project (Flashback), future "Tap an
  untapped artifact you control" cost shapes from Mirrodin /
  Convoke siblings.


- ⏳ **`Predicate::ManaValueAtMostV(Value)` — value-keyed mana-value
  filter** (suggested by push modern_decks's Mind into Matter +
  Sundering Archaic gaps) — both cards want a target / candidate
  filter capped by a runtime-evaluated `Value` (X-from-cost for Mind
  into Matter, ConvergedValue for Sundering Archaic's "exile target
  nonland permanent an opponent controls with mana value less than
  or equal to the number of colors of mana spent"). The current
  `SelectionRequirement::ManaValueAtMost(u32)` is a static cap. A
  Value-keyed sibling needs to thread `EffectContext` (for the X
  value) into both `evaluate_requirement_static` and
  `evaluate_requirement_on_card` — significant call-site refactor.
  Cast-time validation also needs to know the chosen X at the time
  targets are picked (currently the engine picks targets first then
  pays X, so this would need either re-ordering or a "deferred
  validation" pass). Two ⏳ cards exercise this gap; deferring until
  a third card stacks on or the cast pipeline is otherwise touched.

- ⏳ **Augusta, Dean of Order — same-power attackers trigger** (push
  modern_decks STX Silverquill 🟡) — the printed "Whenever you attack
  with three or more creatures with the same power, each of those
  creatures gets +1/+1 and gains your choice of flying, first strike,
  vigilance, or lifelink until end of turn" needs a **batched** post-
  attacker-declaration event (not the per-attacker `Attacks` event
  we have today). Suggested shape: new `EventKind::AttackersDeclared`
  that fires once after `declare_attackers` resolves, with the list
  of attackers exposed via `ctx.attackers_declared`. The trigger
  would then need to find the largest same-power group and pump only
  those creatures (custom selector logic). Skipped until a second
  batched-attack trigger appears in the catalog.

- ⏳ **Mavinda, Students' Advocate — cast-IS-from-graveyard static**
  (push modern_decks STX Silverquill 🟡) — the printed "Once during
  each of your turns, you may cast an instant or sorcery spell that
  targets only a single creature from your graveyard. If a spell
  cast this way would be put into your graveyard, exile it instead."
  is a static ability that grants a cast-permission, not an
  activated ability. Needs (a) a per-player "this-turn cast-from-gy
  budget" counter, (b) a target-introspection at cast time
  ("targets only a single creature"), and (c) a delayed replacement
  to route the resolving spell to exile instead of graveyard.
  Update (was stale): the {0} graveyard-cast ability *is* wired
  (`silverquill.rs::mavinda_students_advocate`) — but as a {0}
  once-per-turn **activated** ability, not the printed static, and the
  "targets only a single creature" sub-filter is dropped (any IS card
  in your graveyard is eligible). The body is 2/3. (Its creature type and
  keywords were wrong — Human Cleric + Vigilance — and were corrected to
  Bird Advisor + Flying in the 2026-06-14 type/keyword sweep; see "Fabricated
  real-name STX cards".)

- ⏳ **Foretell alt-cost primitive** (suggested by push modern_decks's
  Saw It Coming addition) — Foretell ({2} on cast, alt cost {1}{U} on
  the turn after it's foretold from hand for {2}). Wiring shape:
  (a) a new `ActivatedAbility`-style "Foretell" action that exiles
  the card face-down from hand for {2}; (b) a per-card "foretold
  this turn" flag tracked on the exiled card; (c) an `AlternativeCost`
  variant with `not_this_turn_only: bool` that gates the alt cost on
  the prior-turn foretell. Currently Saw It Coming ships as a
  vanilla {2}{U} counter — the Foretell discount path is engine-wide
  ⏳.

- ⏳ **`Predicate::AnyOppHasMoreLandsThanYou`** (suggested by push
  modern_decks's Gift of Estates ramp-spell addition) — Gift of
  Estates's printed gate is "If an opponent controls more lands than
  you, search your library for up to three Plains cards." Today the
  gate is omitted and the spell unconditionally searches three
  Plains. Wiring shape: add a new `Predicate::AnyOppHasMoreLandsThanYou`
  primitive that walks `self.players[opponent]` count of permanents
  matching `SelectionRequirement::Land` and compares against
  `self.players[controller]`'s land count. Same primitive unblocks
  any future "if you're behind on lands" catch-up effect (Tithe,
  Knight of the White Orchid's ETB trigger, Land Tax).

- ⏳ **`EventKind::BecameTarget`** (suggested by push modern_decks's
  Battle Mammoth addition) — Battle Mammoth's printed rider is
  "Whenever a permanent you control becomes the target of a spell or
  ability an opponent controls, draw a card." Today the body ships
  as a 6/5 trampler with the trigger omitted. Wiring shape: a new
  `EventKind::BecameTarget { target, source, source_controller }`
  event emitted by `validate_target_legality` at cast-time and by the
  ability-activation walker. Triggers listening on the event would
  fire post-cast / post-activation. Same primitive unblocks
  Witchstalker Frenzy, Bygone Bishop variants, Glasspool Mimic's
  copy trigger, and any "becomes target" cycle.

- ⏳ **`Predicate::ManaValueGreatest` — sacrifice picker filter**
  (suggested by push modern_decks's Soul Shatter addition) — Soul
  Shatter's printed Oracle is "Each opponent sacrifices a creature or
  planeswalker with the greatest mana value among permanents that
  player controls." Today the auto-picker takes the lowest-CMC
  matching permanent. Wiring shape: a new sacrifice-filter that
  reads each candidate's `card.definition.cost.cmc()` and picks the
  max. Same primitive unblocks future "with the highest power" /
  "with the lowest toughness" picker variants (Skull Fracture,
  Slaughter Specialist, etc.).

- ⏳ **`Effect::DiscardOrSacrifice` — additional-cost picker for "discard
  a card or sacrifice a creature"** — STA Bone Shards (already wired as a
  Sorcery in `mod_set::instants`) uses a `Seq(ChooseMode([Sacrifice 1
  creature, Discard 1]) + Destroy target creature)` approximation. The
  Strixhaven Mystical Archive reprint of Bone Shards is an *instant*
  with the same pick-as-additional-cost rider. Suggested shape: bump
  the picker into a real cost-time decision (so insufficient resources
  to pay one option force the other), wire it via `AlternativeCost`
  with two cost branches keyed off a `ChooseAlternativeCost` decision
  shape. Same primitive unlocks "Pay {X}, sacrifice a creature, or
  discard a card" cycles in future sets.

- ⏳ **Burst Lightning kicker / kicker-as-modal** — STA reprint Burst
  Lightning's "Kicker {4} → 4 damage instead of 2" is an alt-cost-
  implies-mode shape: paying the kicker changes the spell's behavior at
  resolution. Currently wired as the unkicked 2-damage body only. The
  engine's `AlternativeCost` is one cost branch; threading the *paid*
  alt-cost into resolution-time mode selection would unblock Burst
  Lightning, Rite of Replication, Aether Vial-style kicker shells.
  Suggested shape: add `Predicate::CastWithKicker(name)` + thread the
  kicker payment status into `EffectContext`.

- ⏳ **`Predicate::ManaValueEquals(N)` — exact MV target filter** —
  Postmortem Lunge's "target creature card with mana value X" target
  filter (push modern_decks) synthesizes equality as
  `All([ValueAtLeast(MV, X), ValueAtMost(MV, X)])`. A first-class
  `ValueEquals` (or `ManaValueEquals`) predicate would compress the
  expression and let auto-target pickers natively narrow to the exact
  candidate set. The `If` gate on Postmortem Lunge could then drop to
  a plain target filter.

- ⏳ **`Value::PowerOfTargetExiledThisResolution`** — push (modern_decks)
  closed the simpler half via the `Value::PowerOf` evaluator-zone-walk
  extension (gy/exile/hand lookups now work), unlocking Lorehold
  Excavation's "X = its power" rider. The leftover gap is the
  ordering subtlety: a card that triggers _after_ exile (e.g.
  Lavaball Trap's hypothetical "exile a creature; you create an X/X
  where X is its power") needs to read power from the post-Move
  exile zone, not the pre-Move graveyard. The eval extension already
  walks exile, so most cases are covered — only the corner case of
  "the source card itself was exiled by the same effect" might need
  a temp-cached power. Suggested shape: stash `last_zone_changed_card`
  on `EffectContext` (sibling to `trigger_source`) and add
  `Value::PowerOfLastExiled` that reads from it. Open until a real
  card surfaces the gap (currently none in the Crabomination
  catalog).

- ⏳ **Multi-target prompts on instants/sorceries** — recurring 🟡
  reason across STRIXHAVEN2.md (Divergent Equation, Vibrant Outburst,
  Snow Day, Devious Cover-Up, Crackle with Power, Magma Opus,
  Homesickness, Dissection Practice, Cost of Brilliance, Render
  Speechless, Conciliator's Duelist, Rabid Attack, Together as One,
  Reconstruct History's "or more" mode-count picker, …). The engine's
  spell-cast path takes a single `Target` and the auto-decider can't
  pick multiple. Suggested shape: change `GameAction::CastSpell.target`
  from `Option<Target>` to `Vec<Target>` (or `Option<TargetSet>`),
  thread the slot index into `Selector::Target(n)` (already there),
  and bump cast-time target validation to walk every slot. The bot
  harness's AutoDecider needs a per-effect target-count introspection
  to pick N targets; a lazy first pass could just pick the same
  target N times (with deduplication on per-slot legality). Worth
  ~10 🟡 → ✅ promotions.

- ⏳ **Partner-pair primitive** — Plargg / Augusta (STX Dean cycle), the
  Battlebond Partner cycle, and the C20 Commander Partners all share a
  printed "Partner with [other Legendary]" rider that searches the
  library for the named partner on the Partner-carrier's ETB. Engine
  has no `Keyword::PartnerWith(name)` or `Effect::SearchByName`
  primitive yet. Suggested shape: add `Keyword::PartnerWith(&'static
  str)` + an ETB trigger that fires `Effect::Search { filter:
  HasExactName(name), to: Hand(You) }`. Once landed, the STX Dean
  cycle (Augusta + Plargg, Embrose + Valentin, Imbraham + Lisette,
  Lukka + Adrix) and the Battlebond legendaries can wire the partner
  half faithfully.

- ⏳ **`PlayerRef::Opponent` (single-opponent helper)** — engine has
  `EachOpponent` (all opps) and `Target(_)` (cast-time targeting) but
  no "the singular non-controller opp" ref. In 2-player games these
  collapse to the same player, but `Selector::Player(PlayerRef::
  Opponent)` would read more naturally for single-opp effects (e.g.
  "target opponent draws a card" in Baleful Mastery). Workaround
  today is `EachOpponent` which fan-outs in multiplayer.

- ⏳ **Add Inkling-tribal payoffs to the cube/SOS pools** — push XXXI
  added Tenured Inkcaster as an Inkling lord (+2/+2 to other
  Inklings). The catalog now has 4+ Inkling minters (Inkling
  Summoning, Defend the Campus, Silverquill Pledgemage,
  Promising Duskmage, Felisa Fang of Silverquill's Inkling
  generator) — a Silverquill SOS variant pool could lean heavily
  into the tribal pump. Add Inkling Mascot's printed "draw or pump"
  payoff variants once the multi-target prompt lands.

- ⏳ **Audit and update STRIXHAVEN2.md tables on every push** — push
  XXXI found 5 cards (Lorehold Apprentice, Lorehold Pledgemage,
  Storm-Kiln Artist, Sparring Regimen, Spectacle Mage) whose code
  was fully wired but whose 🟡 notes hadn't been updated. A simple
  end-of-push audit script (`audit_strixhaven2.py` already exists
  for SOS) extended to also walk STX-row notes against the
  factory's `triggered_abilities` / `static_abilities` / activated-
  ability complexity could flag stale rows automatically.



- ⏳ **`StaticEffect::SelfPumpIf` (conditional anthem on the source)** —
  Honor Troll's "as long as you've gained life this turn, gets +2/+0
  and lifelink" wants a conditional self-pump that checks a
  predicate (typically `LifeGainedThisTurnAtLeast(1)`) every time
  layers recompute. Shape:
  `StaticEffect::SelfPumpIf { condition: Predicate, power, toughness, keywords }`.
  Wire into `static_ability_to_effects` to conditionally emit the
  PumpPT + GrantKeyword pair only when `condition` is true.

- 🟡 **Multi-target action shape** — Push (modern_decks) lands the
  foundational primitive: `GameAction::CastSpell` (and the other four
  cast variants) gain an `additional_targets: Vec<Target>` field
  alongside the existing `target: Option<Target>`. Slot 0 stays in
  `target`, slots 1+ flow through `additional_targets`. The new field
  has `#[serde(default)]` for snapshot back-compat. Threaded through
  `StackItem::Spell`, `ResumeContext::Spell`, `cast_spell`,
  `cast_spell_with_convoke`, `cast_spell_back_face`, `cast_flashback`,
  `cast_spell_alternative`, `finalize_cast`,
  `continue_spell_resolution`, `EffectContext::for_spell_with_source`
  (merges both into `ctx.targets`). Cast-time validation walks every
  slot via `target_filter_for_slot_in_mode(slot_idx, mode)` and runs
  hexproof/legality checks on each. **Snow Day promoted** as the
  first two-slot card: `Effect::Seq([Tap(target_filtered slot 0),
  AddCounter(Target(0)), Tap(TargetFiltered slot 1), AddCounter(
  Target(1))])`. "Up to two" semantics fall out naturally — slot-1
  selectors resolve to nothing when only one target is passed, so
  the second tap+stun pair is a no-op. Tests:
  `snow_day_taps_and_stuns_target_creature` (slot 0 only),
  `snow_day_taps_and_stuns_two_target_creatures` (both slots).
  **Still 🟡 because the AutoDecider's auto-target picker does not
  yet populate `additional_targets`** — cards relying on the bot to
  pick slot-1 targets need manual promotion (Crackle with Power,
  Render Speechless, Vibrant Outburst, Devious Cover-Up, Decisive
  Denial mode 1, etc.). The cast API supports them; the bot harness
  hasn't been updated to drive them. Easy follow-on push: extend
  `auto_target_for_effect_avoiding` to take a slot count and return
  `Vec<Target>` with per-slot legality.

- 🟡 **Lesson sideboard model** — primitive landed. `Player.sideboard`
  holds Lessons "outside the game"; `Effect::Learn { who }` surfaces
  `Decision::Learn` (reveal a Lesson into hand / discard-to-draw /
  decline) via `DecisionAnswer::Learn(LearnChoice)`, and falls back to
  `Draw 1` when no Lessons sideboard is configured (so existing
  no-sideboard games and tests are unchanged). **All** Strixhaven Learn
  cards are now wired to `Effect::Learn` — the four canonical ones plus the
  Lessons that themselves Learn (Guiding Voice, Mascot Interpretation,
  Reduce // Rubble, Lesson in Honor) and Professor of Symbology.
  `cube::build_cube_state` seats each player with the standard
  `cube::lessons_sideboard()` via `GameState::add_card_to_sideboard`, so
  Learn fetches in real cube games. Covered by
  `tests::game::{learn_fetches_a_lesson_from_the_sideboard,
  learn_rummage_discards_then_draws, learn_decline_does_nothing}` and
  `cube::tests::build_cube_state_gives_each_seat_a_lessons_sideboard`.
  The client UI suspend flow is wired: a `wants_ui` player's Learn suspends
  on `Decision::Learn` (`PendingEffectState::LearnPending`) and the client's
  `decision_ui::spawn_learn_modal` / `handle_learn_buttons` render the
  reveal-a-Lesson / discard-to-draw / decline modal, submitting
  `DecisionAnswer::Learn(LearnChoice)`. Covered by
  `tests::game::learn_ui_player_suspends_and_resumes_via_submit_decision`.
  Remaining: populate sideboards in the other deck-build paths (formats /
  draft).
- ⏳ **Counter-multiplier primitive** — Already used by Tanazir
  (via the ForEach idiom). Future cards (Vorinclex, Doubling
  Season) want a true multiplier on counter accrual; tracked
  separately.
- ⏳ **Mana-spent-on-cast introspection** — Opus / Increment
  riders read "amount of mana spent to cast that spell" on the
  just-cast spell event. The engine doesn't yet preserve the
  numeric mana-paid total per stack item; this would unblock
  Aberrant Manawurm, Tackle Artist, Expressive Firedancer, etc.
  Suggested shape: `Value::ManaSpentOnCast(Box<Selector>)` that
  reads from `StackItem::Spell.mana_paid_total`.
- 🟡 **CR 700.2d — modal "choose two" / "choose more than one"** —
  `Effect::ChooseN { picks: Vec<u8>, modes: Vec<Effect> }`. Each
  target-bearing mode owns its own cast-time target slot, assigned in
  default-`picks` order (`target_filter_for_slot_in_mode` + the
  resolution-time `slot_of_mode` map both key off `picks`), so a
  "choose two" spell can take e.g. a spell target for one mode and a
  permanent target for another (Cryptic Command counter+bounce,
  Kolaghan's Command reanimate + any-target damage, Steal the Show,
  the five Strixhaven Commands). The auto-decider/UI run the default
  `picks`; a `ScriptedDecider` can pick any subset, but **targets only
  route correctly for mode-subsets of the default `picks`** (both the
  cast-time validation and the resolution slot map are keyed off the
  card's default `picks`, and the dense `target`+`additional_targets`
  vec can't represent a slot-1-only pick). Closing that needs cast-time
  mode selection: bump `GameAction::CastSpell.mode: Option<usize>` →
  carry the chosen ChooseN picks, validate/route slots against them
  rather than the default. Still ⏳.
- ⏳ **`magecraft_self_untap()` / `magecraft_drain_each_opp(N)`
  shortcuts** — push XXVII added two new shortcut helpers in
  `effect::shortcut`. Future STX/SOS Magecraft creatures should
  prefer these over the verbose inline form for consistency. Hall
  Monitor (push XXVII) and Witherbloom Apprentice (refactored in
  push XXVII) demonstrate the pattern.


# Rules coverage

## MagicCompRules coverage audit

Periodic spot-check against the rules document (`MagicCompRules_20260417.txt`).
One line per rule: status (✅ wired · 🟡 partial · ⏳ todo) plus the still-open
gap. The full per-clause accounting (every sub-rule, code line, and test name)
was elided in a doc-compaction pass — recover it from
`git log -p -- TODO.md`. Markers are a point-in-time read; re-verify before
picking an item up.

### Done (✅) — wired
- ✅ **CR 400.7 — cast provenance doesn't follow a card between zones** — the
  leave-the-battlefield reset now clears `cast_from_hand` / `cast_from_exile` /
  `cast_from_library` / `cast_via_flashback` / `cast_from_suspend` /
  `cast_from_escape` alongside the granted-ability and per-object activation
  limits, so a reanimated permanent isn't still "cast from your hand" (Phage
  the Untouchable; `classic_sets/lgn::phage_only_survives_a_hand_cast`).
- ✅ **CR 407 — Ante** — `Zone::Ante` + `Player.ante` + `ZoneDest::Ante`;
  `GameState::begin_ante_game` does the 407.2 opening ante and
  `award_ante_to` the winner-takes-all (fired from the game-over SBA).
  407.3's "remove this card from your deck" rides
  `CardDefinition.ante_only` → `DeckError::AnteCardOutsideAnteGame`, and
  `Effect::ExchangeOwnership` is the only ownership change in the engine.
  All nine printed ante cards ship in `sets::ante`; tests
  `core_rules/cr_recent72::cr_407_*`. ⏳ residual: Darkpact picks the first
  ante card rather than targeting one, and Bronze Tablet folds its
  exile-both into a sacrifice.
- ✅ **CR 211 / 212 / 313 / 902 — Vanguard** — `CardType::Vanguard` +
  `CardDefinition.{hand_modifier, life_modifier}`; `GameState::seat_vanguard`
  seats the avatar in the command zone and applies both modifiers (and its
  `NoMaximumHandSize` static). Its abilities function from there:
  activated via `ActivatedAbility.from_command_zone`, step triggers via
  `fire_step_triggers`, cast triggers via the SpellCast gather, other events
  via `dispatch_triggers_for_events`. `sets::vanguard` (8 avatars);
  `core_rules/cr_recent66`. ⏳ residual: general statics from the command zone.
- ✅ **CR 502.4 — "permanents don't untap"** — `StaticEffect::PermanentsDontUntap`
  short-circuits `do_untap` for every seat while still clearing summoning
  sickness (Mist of Stagnation; `cr_502_4_global_dont_untap_stops_every_seat`).
  CR 502.2's active-player-only untap is covered by
  `cr_502_2_only_the_active_player_untaps`.

One line per wired rule; implementation detail (code symbols, tests) elided —
recover from `git log -p -- TODO.md`. A few rows carry a residual ⏳ gap inline.

- ✅ CR 701.9 — Discard *batching*. `GameEvent::DiscardedBatch { player, count }`
  / `EventKind::DiscardedOneOrMore` fire one "you discarded one or more cards"
  event per effect resolution (alongside the per-card `CardDiscarded`s), carrying
  the count via `Value::TriggerEventAmount` — Magmakin Artillerist deals that much
  to each opponent, once. Emitted from `resolve_effect` off the
  `cards_discarded_per_player_this_resolution` scratch; test
  `cr_701_9_discard_batch_fires_once_with_count`. The CR 514.1 cleanup
  discard-down now emits the batch too (both the deterministic and UI-resume
  paths; `cr_514_3_cleanup_discard_fires_batch_trigger`). Activation-cost discards
  (`discard_cost`, `discard_hand_cost`) emit the batch from `activate_ability`
  (`cr_701_9_cost_payment_discard_fires_the_batch`). Cycling and landcycling
  emit it too (`cr_701_9_cycling_fires_the_discard_batch`). Remaining: the other
  spell-level "discard this card" costs.
- ✅ CR 120.10 — Excess damage — `Effect::DealDamageExcessToController` deals N
  to a creature and spills the overkill (past its remaining toughness) onto its
  controller (Flame Spill; `flame_spill_excess_hits_controller`). Combat
  damage→token scaled by `Value::TriggerEventAmount` (Quartzwood Crasher,
  `DealsCombatDamageToPlayer`; CR 510.2/119.3).
- ✅ CR 702.179 — Freerunning. Alt cost gated on `Predicate::DealtCombatDamageToPlayerThisTurn` (`Player.dealt_combat_damage_to_player_this_turn`, set in `fire_combat_damage_to_player_triggers`). ACR batch in `decks::freerunning` (Brotherhood Ambushers, Merciless Harlequin, Achilles Davenport, Eagle Vision, Distract the Guards, Chain Assassination, Restart Sequence, Viewpoint Synchronization, Escape Detection, Overpowering Attack). The "with an Assassin or commander" sub-clause is approximated as "with any creature." ⏳ remaining cards: Petty Larceny (exile-and-play-from-exile + any-color), Monastery Raid (Freerunning {X} + was-freerun provenance rider).
- 🟡 CR 708 — Face-Down Permanents
- ✅ CR 310 — Battle / Siege. `CardType::Battle` + `BattleSubtype::Siege`, defense
  counters (310.7), protector choice (310.6), attack-your-own-Siege
  (`AttackTarget::Battle`), combat **and noncombat** damage strip defense
  counters (310.10 — noncombat path added in `deal_damage_to_from`; Onakke
  Javelineer, `onakke_javelineer_damages_a_battle`), defeat→exile/transform SBA
  (704.5x via `defeat_battle`). 6 MOM Invasions in `decks::mom`. ⏳ multiplayer
  protector choice.

- 🟡 **CR 616 — Interaction of Replacement and/or Prevention Effects** —
  616.1c/616.1g ✅: the enters-as-a-copy replacement outranks the enters-tapped
  one, so tappedness is re-decided against the copied characteristics
  (`reapply_enters_tapped_after_copy`; Clone of Rusted Sentinel enters tapped —
  `cr_recent42::cr_616_1c_*`). **616.1e player choice ✅ for draws** — the
  competing "dig instead of drawing" replacements (Parallel Thoughts, Tomorrow,
  Archmage Ascension, Abundance) are enumerated as `DrawDig` and the drawing
  player picks which applies (`choose_draw_replacement` / `apply_draw_dig`); a
  declined optional pick drops out and the choice is offered again. A headless
  seat keeps the canonical order. `cr_recent74::cr_616_1e_*`. Remaining:
  616.1a self-replacement priority, and the same player choice for the
  non-draw replacement families (ETB, damage, counters).

### Partial (🟡) — remaining gap noted

- ✅ **CR 808 — Team vs. Team** — teams partition seats (`assign_teams`,
  `same_team`, `teammates`) with per-seat resources (no shared hand, mana or
  life — `Team.shared_life` stays `None`, unlike CR 810 2HG), and 808.3a's
  attack-multiple-players default falls out of `declare_attackers` rejecting a
  teammate as defender. `cr_recent74::cr_808_*`. ⏳ residual: 808.4's
  center-seat first-player rule isn't modeled (seat 0 always starts).
- 🟡 **CR 509.2 / 510.1c — Banding** — a banding blocker routes the blocked
  attacker's damage order + assignment to the defending player, including
  banding *granted during the combat* (Wall of Caltrops' block trigger;
  `cr_509_2_banding_blocker_lets_defender_assign_damage`,
  `cr_recent74::cr_509_2_banding_gained_midcombat_still_routes_assignment`).
  Attacking bands (`declare_attackers_banded`) and "bands with other"
  (`Keyword::BandsWithOther` + `bands_with_other_qualities`) both ship.
  Remaining: the band-blocks-multiple damage-distribution corner.
- 🟡 **CR 303 — Auras** — characteristic-overriding Auras ✅ (`EquipBonus.{set_base_pt,set_card_types,set_creature_types,set_colors,remove_abilities}` install layer 4/5/6/7b continuous effects on the host — Ichthyomorphosis "0/1 blue Fish, no abilities", One with the Stars "becomes an enchantment", Heliod's Punishment "loses abilities + can't attack/block"; removal is ordered before the aura's own keyword grants so they survive — test `cr_613_aura_set_base_pt_then_counter`). **Aura/Equipment-granted step triggers ✅** (CR 702.6e — `fire_step_triggers` now dispatches `EquipBonus.triggered_abilities` whose kind is a step, sourced on the host and scoped to the host's controller; Pillory of the Sleepless's "enchanted creature has: at your upkeep, you lose 1 life" — `cr_702_6e_aura_granted_upkeep_trigger_keys_on_host_controller`). **CR 303.4a "enchant player" ✅** — `CardInstance.attached_to_player` anchors an Aura to a seat, `PlayerRef::EnchantedPlayer` and `EventScope::EnchantedBySource` read it, `StaticEffect::PumpPT` takes a `Selector::ControlledBy` anthem scope, and the orphan-Aura SBA leaves player-Auras alone (`catalog::sets::curses`; tests `core_rules/cr_recent37`). **CR 702.103f ✅** — a bestowed Aura that is unattached *or* attached to an illegal object reverts to a creature instead of dying. Remaining: replacement-style Aura ETB (enters attached under another rule).
- 🟡 **CR 603.10 — Last-Known Information** — full LKI for mid-resolution stack sources (e.g. lifelink 702.15c). Aura death LKI is now path-independent: `remove_to_graveyard_with_triggers` records `auras_at_death` before the host leaves, so `EventScope::EnchantedBySource` triggers fire on the destroy/sacrifice funnel as well as the lethal-damage SBA (`cr_603_10_enchanted_dies_trigger_fires_on_a_sacrifice`). (CR 603.6d "leaves the battlefield" self-source triggers now also fire on the lethal-damage SBA path, not just the destroy/sacrifice path — Thought-Knot Seer's LTB draw.) Sac-as-cost activated abilities that read the sacrificed source's own counters at resolution now stash `leaves_bf_lki` during cost payment (it outlives the per-dispatch `died_card_snapshots` clear) so `Value::TotalCountersOn { This }` reads the last-known total — Twitching Doll's "Spider per counter on it" (`twitching_doll_nests_then_sacs_for_spiders`). `SelectionRequirement::ControlledByYou` now falls back to `died_card_snapshots` for the LKI controller, so a graveyard-scoped "a creature you control dies" trigger fires only for your creatures — Furious Forebear (`cr_603_10_died_creature_controller_read_from_lki`). CR 603.10a self-death: both self-death funnels (SBA lethal-damage + destroy/sacrifice) now evaluate a filtered `YourControl`/`AnyPlayer` death trigger's `.with_filter` against the dying creature via the death snapshot, and the destroy/sacrifice path fires self-inclusive scopes (was SelfSource-only) so an aristocrat drains for its own sacrifice (`cruel_celebrant_drains_on_its_own_sacrifice`).
- 🟡 **CR 704 — State-Based Actions** — Saga SBA ✅ (`saga_chapters` reach
  final chapter → sacrifice, unless a chapter ability is still on the stack);
  spell-copy-off-stack identity ✅ (704.5d/e — the token-purge SBA sweeps
  copies from every non-stack zone; test
  `cr_704_5e_countered_spell_copy_ceases_to_exist`); Role uniqueness ✅
  (704.5y). Illegally-attached Aura ✅ (704.5n / 303.4f — an Aura whose live
  host fails its printed `aura_enchant_filter`, e.g. a "you control" Aura on a
  stolen creature, goes to the owner's graveyard; tests `cr_704_5n_*`).
  Zero-toughness → graveyard ✅ (704.5g, test
  `cr_704_5g_zero_toughness_creature_dies`). Battle-with-no-defense-counters
  defeat ✅ (704.5x via `defeat_battle`, `tests/mom.rs`). Speed SBA ✅ (704.5z —
  `check_state_based_actions` seeds speed 1 for engines controllers; test
  `cr_704_5z_engines_seed_speed_sba`). Multi-SBA "collapse into one
  replacement" ✅ (704.7 — `StaticEffect::ReplaceControllerLossWithReset` +
  `GameState::apply_loss_reset`; Lich's Mirror replaces a life *and* poison
  loss once, and covers the draw-from-empty loss too; `cr_recent42::cr_704_7_*`).
  Dungeon removal ✅ (CR 309.6 — room abilities use the stack and the
  finished dungeon leaves the game as the last one resolves;
  `cr_recent84::cr_309_6_*`).
- 🟡 **CR 613 — Interaction of Continuous Effects** — 613.7 timestamps ✅ (object timestamps stamped on entry/attach/face-up/transform from the shared effect counter; statics order by `object_timestamp()`; tests `cr_613_7_*`). Remaining: no dependency analyzer (613.8); CDA-first pre-pass (613.3). (EOT keyword grants now join the walk timestamped — audit P1 row closed. Static keyword-grant scopes now route a `ToughnessGreaterThanPower` leaf through the `CardMatch` dynamic path — read against printed P/T + counters per the `CardMatchPowerGated` approximation — so Tapestry Warden / Ancient Lumberknot grant their keyword only to your T>P creatures.) Layer-4 additive card-type static ✅ (`StaticEffect::AddCardTypeToMatching` — "nontoken artifacts you control are lands in addition to their other types", Toph, the First Metalbender; `toph_metalbender_artifacts_are_lands_and_end_step_earthbend`). **CR 613.2 computed-subtype consistency ✅** — `HasArtifactSubtype`/`HasLandType`/`HasSupertype` requirements now read a battlefield permanent's *computed* (post-layer) subtypes/supertypes, matching card-type and creature-type checks, so continuous subtype grants (Sugar Coat's Food, Vraska's Treasure, Song of the Dryads' Forest, the Ring-bearer's Legendary) are seen by aura-legality SBAs and filters (`blb::sugar_coat_makes_a_food`; fixed the Alpine Moon test that had leaned on the printed-subtype read). `EquipBonus.set_artifact_types` installs the layer-4 artifact-subtype override.
- 🟡 **CR 208 — Power/Toughness** — base-P/T-only checks (208.4b). 208.3 noncreature P/T now observable for `*`-power Vehicles: `DynamicPt::LandsControlledPower` sets power off a count while toughness stays printed, `computed_permanent()` reports it on a non-crewed (noncreature) Vehicle (Lumbering Worldwagon `*`/4; test `lumbering_worldwagon_power_tracks_lands`). Conditional base-P/T set ✅ (`StaticEffect::SetBasePtIf` — live layer-7b SetPowerToughness gated on a predicate; counters/+N stack on top per 613.7c/f — Snowmelt Stag "5/2 during your turn"; `snowmelt_stag_*`). CR 604.3 CDAs: `DynamicPt::LandsControlledPlusLandsInControllerGraveyard` (Multani, Yavimaya's Avatar), `DynamicPt::CardTypesInOpponentsGraveyards` (Nighthawk Scavenger), `DynamicPt::InstantsSorceriesInControllerGraveyard` (Enigma Drake), `DynamicPt::CreaturesControlledPower` (Suki `*`/4), `DynamicPt::PlusCountersOnLandsControlledPower` (Toph `*`/3), `DynamicPt::NoncreatureNonlandCardsInControllerGraveyard` (Dragonfly Swarm `*`/3), `DynamicPt::ColorsAmongAlliesControlledPower` (Earthen Ally `*`/2), `DynamicPt::EnchantmentsInPlay` (Yavimaya Enchantress `2/2`, +1/+1 per enchantment in play — `tests/recent72.rs`), `DynamicPt::ForestsInPlay` (Traproot Kami `0/*`, toughness = Forests on the battlefield — `tests/recent100.rs`), all live-recomputed by `computed_permanent()`; `tests/recent47.rs`, `tests/recent50.rs`, `tests/tla.rs`.
- 🟡 **CR 119 — Life** — 119.7 set-to-lowest ✅ (`Value::LowestLifeTotal` + Repay in Kind); exchange-life-totals ✅ (Soul Conduit, Mirror Universe, Magus of the Mirror); life-gain→loss replacement ✅ (`StaticEffect::LifeGainBecomesLoss`, Tainted Remedy); life-gain **bonus** replacement ✅ (119.10 — `StaticEffect::LifeGainBonus { target, amount }` folded into `adjust_life` via `life_gain_bonus_now`; Honor Troll's "gain that much plus 1"). 119.7 rest-of-game lifegain lock ✅ (`Effect::LifeGainLockGame` sets the permanent `Player.cannot_gain_life` flag, distinct from the turn-scoped lock — Screaming Nemesis via `Selector::Target(0)`; test `screaming_nemesis_redirects_damage`). Life-total-threshold statics ✅ (`Predicate::PlayerLifeAtLeast` gates a live self-anthem — Angel of Vitality's +2/+2 at 25+ life; `cr_119_*`, `tests/recent17.rs`). Life-vs-*starting*-total statics ✅ (`Predicate::PlayerLifeAtLeastAboveStarting` gates tiered self-pumps — Elenda, Saint of Dusk +1/+1/menace above starting, +5/+5 more at 10+ above; `elenda_scales_with_life`). Exact-life gate ✅ (`Predicate::PlayerLifeExactly` — Hidetsugu's Second Rite deals 10 only if the targeted player is at exactly 10; `hidetsugus_second_rite_needs_exactly_ten`). Redistribute-life-totals (119.7) is exact at two players — Reverse the Sands rides `ExchangeLifeTotals`, `reverse_the_sands_swaps_life_totals`; a true multiplayer redistribution (each player picks which total they get back) is still open. Remaining: per-source life-gain replacement breadth. (Audit follow-up closed: every `LifeGained` emitter now uses `adjust_life_applied`, and `SetLifeTotal`/`ExchangeLifeTotals` route through the funnel — so a can't-gain-life lock on the player who would gain blocks their half of an exchange while the other still loses; test `cr_119_7_exchange_life_totals_respects_cant_gain_life`.)
- 🟡 **CR 121 — Drawing a Card** — one-shot draw replacement ✅
  (`Effect::ReplaceYourNextDrawThisTurn` queues a charge on
  `Player.next_draw_replacements`; `draw_one` spends the front charge and
  resolves its body — auto-targeting it when it needs one — and unused charges
  clear at the turn boundary. The Onslaught Words cycle;
  `classic_sets/ons::words_cycle_replaces_the_next_draw`). Draw-count
  replacement (121.2a) ✅ via `StaticEffect::ControllerDrawsDoubled` in `draw_one` (Thought Reflection; stacks per 614.5, reentrancy-guarded); **condition-gated** draw doubling ✅ (`ControllerDrawsDoubledIf` — Vnwxt's max-speed draw-two; test `cr_121_2a_conditional_draw_replacement`). Draw-count board gates ✅ via `SelectionRequirement::ControllerDrewAtLeastThisTurn(n)` (reads `Player.cards_drawn_this_turn`), wired as a `SelfHasKeywordWhile` condition (Foggy Swamp Hunters lifelink/menace, June unblockable). Choose-to-draw (121.3 / 121.2b) ✅ — `GameState::may_choose_to_draw` stops `Effect::MayDo` / `Effect::MayPay` offering an optional draw to a capped player (a rules-declined `MayPay` still runs its `else_`), and the per-turn cap now gates `draw_one` itself so *every* draw source is capped, not just `Effect::Draw`'s count; an empty library deliberately doesn't block the choice. Chains of Mephistopheles ships as a global replacement in `draw_one` with a CR 614.5 reentrancy guard (`cr_recent74::cr_121_2a_chains_replaces_each_extra_draw_once`). Remaining: mid-cast face-down draw (121.8); reveal-on-draw (121.9).
- ✅ **CR 613.8 — type-gated grants see a retype.** `GameState::
  shallow_creature_types` reads stored layer-4 `SetCreatureTypes`/
  `AddCreatureType` effects without a full layer pass, so a requirement walk
  running *inside* the layer gather (where the computed view is off-limits)
  still sees a retyped permanent — Mistform Wall keeps defender only while it is
  a Wall (`cr_613_8_type_gated_grant_sees_a_retype`). `SelectionRequirement::
  FaceDown` also joined the card-only set so face-down-matters anthems apply
  (Ixidor, Reality Sculptor).
- ✅ **CR 605.1b / 605.4a — triggered mana abilities** — a targetless
  mana-adding trigger fired from a mana ability resolves off-stack, so its mana
  reaches the pool before the payment in progress finishes (Overabundance).
  `TriggerCandidate`/`PendingTriggerPush` carry `from_mana_ability`; tests
  `cr_recent63::cr_605_{1b,4a,5a}_*`. Remaining ⏳: 605.3c ("can't be activated
  again until it has resolved") isn't modelled.
- 🟡 **CR 502 — Untap Step** — untap caps are now filtered (`StaticEffect::MaxOneUntapPerStep { filter }` — Winter Moon's nonbasic lands and Imi Statue's artifacts share one path; `imi_statue_caps_artifact_untaps_at_one`). CR 502.3 "doesn't untap while it has a [kind] counter" now reads the **computed** keywords at both untap gates, so a *granted* lock counts (Temporal Distortion's hourglass counters), not just a printed one — `cr_recent63::cr_502_3_counter_gated_permanent_doesnt_untap`. Phasing (502.1 / 702.26) ✅: `do_phasing`
  runs as a turn-based action at the top of the untap step, moving the active
  player's phasing permanents (and their attachments) to `GameState.phased_out`
  and phasing back in everything they control there — modelled as a side zone
  so every battlefield query ignores phased-out cards and no ETB/LTB fires, all
  state retained (Tolarian Drake). Targeted phase-out ✅ via `Effect::PhaseOut`
  (Vodalian Illusionist). Daybound/Nightbound DFC transform (502.2) ✅ — see
  CR 712 below.
  `StaticEffect::PreventUntap` honors `Selector::This` (Basalt/Grim Monolith)
  and `Selector::AttachedTo(This)` (Claustrophobia/Dehydration). Per-player
  one-step land-untap lock ✅ (502.3 — `Effect::LandsDontUntapNextUntapStep` +
  `Player.lands_dont_untap_next_untap`, consumed in `do_untap`; Bontu's Last
  Reckoning, `cr_502_3_bontus_lands_skip_one_untap_step`). Self-scoped
  untap-on-every-step ✅ (502.3 — `StaticEffect::UntapSelfEachUntapStep`, a
  `do_untap` follow-up pass untaps the source on each *other* player's untap
  step too, Stun counters still interpose; Thousand Moons Infantry,
  `thousand_moons_infantry_untaps_on_opponent_untap`).
- ✅ **CR 702.158 — Space Sculptor.** `Keyword::SpaceSculptor`,
  `CardInstance.sector`, the CR 704.5u assignment SBA (opponents assign first;
  designations clear with the last sculptor per 702.158b),
  `Effect::ChooseSector` + `Selector::CreaturesInChosenSector` (702.158d), and
  the same-sector block lock. Space Beleren ships; tests
  `core_rules/cr_recent36`. Residual: the assignment and the sector pick are
  auto-decided rather than prompted.
- 🟡 **CR 509 — Declare Blockers** — cost-to-block (509.1d-f). **509.3a–e ✅**: "whenever this blocks" / "becomes blocked" fire ONCE per creature (the `BlockerDeclared` fan-out dedupes on the trigger's own side of the pair), `Selector::BlockedAttacker` resolves every attacker a multi-blocker is blocking so the per-object wordings (509.3b/d) reach all of them from one instance, and `EventKind::{BlocksNOrMore,BecomesBlockedByNOrMore}` gate on the finished block assignment (509.3e — Lairwatch Giant). Tests `core_rules/cr_recent35::cr_509_3*`. **Multi-block ✅** (509.1b — `block_map` is blocker → `Vec<attacker>`; `Keyword::CanBlockAdditional(n)` / `CanBlockAnyNumber` set the per-combat cap; Guardian of the Gateless, Knight of Sorrows, Valor Made Real; tests `core_rules/cr_recent35`). Put-onto-battlefield-blocking (509.4) ✅ — `Effect::CreateTokenBlocking` + the `cast_only_after_blockers` gate (Flash Foliage; test `cr_509_4_flash_foliage_blocks_the_attacker`). Blocker legality now reads the computed view ✅ (509.1a — animated manlands / crewed Vehicles block). ("Can't be blocked except by N or more creatures" ✅ via `Keyword::CantBeBlockedExceptByN` — Pathrazer of Ulamog, generalizing Menace.) Per-pair block restriction (509.1b — "target creature can't block this creature this turn") ✅ via `Effect::CantBlockSourceThisTurn` + `GameState.cant_block_pairs` (Kozilek's Pathfinder); "must be blocked if able" (509.1c) ✅ via `Keyword::MustBeBlocked` (Loathsome Catoblepas). Power-based block restriction ✅ (`Keyword::CantBeBlockedByPowerLess` — Formation Breaker; inverse of Skulk, `formation_breaker_blocks_only_by_equal_or_greater_power`). The bot's block planner now satisfies the minimum-blocker count for Menace **and** `CantBeBlockedExceptByN(n)` (tops up or drops the block), so it never submits an illegal under-filled multi-block. Protection-by-mana-value block restriction ✅ (`Keyword::ProtectionFromManaValueExcept` — Haktos can't be blocked by a creature whose MV isn't the chosen number; test `cr_509_1b_protection_from_mv_restricts_blockers`). Protection-by-mana-value-**parity** ✅ (`Keyword::ProtectionFromManaValueParity { odd }` — Lavabrink Venturer's ETB odd/even choice; gates targeting, blocking, and combat-damage prevention CR 702.16e; tests `lavabrink_venturer_parity_protection`, `cr_702_16e_parity_protection_prevents_combat_damage`). Blocker-side "can block only creatures with flying" ✅ (`Keyword::CanBlockOnlyFlying` — Wanderlight Spirit, Shacklegeist, Pinnacle Emissary's Drone; test `cr_509_1b_can_block_only_flying_restriction`). Conditional attack/block gates (509.1a / 508.1a) ✅ — `Keyword::CantAttackOrBlockUnlessHandSizeAtMost(n)` (Hazoret the Fervent), `Keyword::CantAttackOrBlockUnlessDelirium` (Patchwork Beastie, via `GameState::delirium_active`), and `Keyword::CantAttackOrBlockUnlessDescend(n)` (The Ancient One, via `GameState::descend_count`), enforced in `declare_attackers` + `blocker_can_block_attacker` + `legal_attackers`/affordances and surfaced as client chips. "Can't attack or block alone" (509.1c) ✅ — `Keyword::CantAttackOrBlockAlone` rejects a lone-attacker / lone-blocker batch (Toby's Beast token; tests `cant_attack_or_block_alone_*`, `cant_block_alone_*`).
- 🟡 **CR 118 — Costs** — interactive mana-ability decline (118.3c); hybrid-pip per-reduction choice (118.7e); general unpayable-cost gate (118.6). Board-conditional self cost reduction ✅ (CR 601.2f — `StaticEffect::SelfCostReducedIfControlEach`, discounts a spell while you control a permanent matching each filter — Of One Mind's Human + non-Human). Opponent target-tax ✅ (`StaticEffect::TaxOpponentSpellsTargeting`, threaded through `extra_cost_for_spell` with the spell's chosen target — Jubilant Skybonder, Callaphe Beloved of the Sea). Mana-spent-vs-MV gate ✅ (`Effect::CounterSpellDrawIfUnderpaid` reads the countered spell's stored `mana_spent` against its mana value — Unravel draws only on a cost-reduced/alt-cast spell). Total-power self-reduction ✅ (`StaticEffect::SelfCostReducedByTotalPower` — Ghalta, Primal Hunger; `ghalta_costs_less_per_total_power`). Per-graveyard-creature self-reduction ✅ (`StaticEffect::SelfCostReducedPerCreatureInGraveyard` — Ghoultree; `ghoultree_costs_less_per_graveyard_creature`). Death-gated self-reduction ✅ (`StaticEffect::SelfCostReducedIfCreatureDiedThisTurn` — Bone Picker; `bone_picker_is_cheap_after_a_death`). Player-wide predicate-gated reduction ✅ (`StaticEffect::CostReductionWhile { filter, amount, condition }` — Gran-Gran's "noncreature spells you cast cost {1} less while 3+ Lessons in your gy"; generic-only clamp tested in `cr_601_2f_gran_gran_lesson_discount_is_generic_only`). Source-power-scaled reduction ✅ (`StaticEffect::CostReductionBySourcePower` — "Aura and Equipment spells cost {X} less, X = this creature's power" — Golden-Tail Trainer). Board-count "affinity for [type]" reduction (`SelfCostReducedPerPermanentMatching`) now evaluates board-state filters (`IsModified`, tapped, …) through `evaluate_requirement_static`, so Walking Skyscraper's "costs {1} less per modified creature" works; `tests/recent100.rs`. **CR 107.16 variable {E} cost ✅** — `ActivatedAbility.energy_x_cost` spends the activation's chosen `x_value` in energy and threads that X into resolution so `ManaValueExactlyXFromCost` gates the target (Chthonian Nightmare; `cr_107_16_variable_energy_cost_pays_chosen_x`). Value-amount energy pay/upkeep ✅ (`Effect::PayEnergyValue`, `Effect::PayEnergyOrElseValue` — Jolted Awake, Volatile Stormdrake). **CR 107.16 variable life cost ✅** — `ActivatedAbility.x_life_cost` drains the chosen X in life and threads that X into resolution (Krumar Initiate's "Pay X life: endure X"; `cr_107_16_pay_x_life_variable_activation_cost`). Card-level "costs {N} less if you've cast another spell this turn" ✅ (`self_cost_reduction_if_cast_spell` — Rally the Monastery).
- 🟡 **CR 113 — Abilities** — emblems+CDA zones (113.6); full ability removal (113.10b); "can't have" anti-grant (113.11). Counter-target-ability (113.9) ✅ — `Effect::CounterAbility` (Consign to Memory, Stifle) with precise targeting via `SelectionRequirement::HasAbilityOnStack`.
- 🟡 **CR 115 — Targets** — Aura subtype (115.1b); zero-target cast-time gate (115.6 — **blocked**: many targeted spells are cast with `target: None` and auto-target at resolution (counterspells → top of stack; "target player" discard → an opponent), so a naive "requires_target ⇒ reject None" gate breaks Pact of Negation / Pyroblast / Cabal Therapy / Metallurgic Summonings. A real fix must make cast-time supply the target for every targeted spell first); change-target corners (115.7a-d, cross-spell exchange). Same-target rejection *within one multi-target instance* (115.3) ✅ — `Effect::distinct_target_count` + a cast-time duplicate check reject the same object filling two divide/support slots (Forked Bolt); cross-clause sharing stays legal. "Up to N target" triggers now fill every slot ✅ (115.1c) on both the **Attacks** path (combat.rs — Lagorin's "up to two Mounts/Vehicles"; `cr_115_1c_attack_trigger_fills_all_target_slots`) and the **ETB** path (stack.rs's `auto_extra_targets_for` — Azorius Justiciar detains two; `cr_115_1c_etb_trigger_fills_all_target_slots`). "Counter target spell that targets you or a permanent you control" ✅ via `SelectionRequirement::SpellTargetsControllerOrControlled`, which reads a stack spell's chosen targets per CR 115.9b (Hindering Light; `cr_115_9b_target_filter_reads_the_current_targets`). **CR 601.2c "must be chosen as a target" ✅** — `StaticEffect::FlagbearersMustBeTargeted` + `flagbearer_violation` gate both the cast and activation paths, the auto-targeter prefers a Flagbearer, and `PermanentView.is_flagbearer` explains the rejection client-side (Standard Bearer, Coalition Honor Guard, Coalition Flag; `cr_recent60::cr_601_2c_*`).
- 🟡 **CR 116 — Special Actions** — Companion ✅ (116.2g / 702.139 —
  `GameAction::CompanionToHand`, {3} sorcery-speed sideboard→hand; deck-build
  restriction ✅ via `CardDefinition.companion` + `format::companion_restriction_met`,
  enforced by the server deck loader). (Foretell/Plot/Suspend ✅; manifest turn-face-up `GameAction::TurnFaceUp` ✅ — CR 708.5. Morph cast-face-down spell path still ⏳.)
- 🟡 **CR 105 — Colors** — type-line + color rewrite rider (105.3 second half).
  Color-count value (105.2 — `Value::ColorCountOf`, "for each of its colors";
  colorless/devoid counts 0 per 105.2c) ✅ — Breathe Your Last; tests
  `cr_105_2c_colorless_counts_zero_colors`, `breathe_your_last_gains_life_per_color`.
- ✅ **CR 705 — Flipping a Coin** — Mana Clash two-player flip-off loop (705.2), 705.3 advantage/Krark's Thumb, win-a-flip trigger (`EventKind::WonCoinFlip`/`GameEvent::CoinFlipWon`, Chance Encounter) and lose-a-flip trigger (`EventKind::LostCoinFlip`/`GameEvent::CoinFlipLost`, emitted on the tails path of FlipCoin + ManaClash). Sequential "flip until you lose or stop" ✅ via `Effect::FlipCoinsUntilLoseOrStop { tiers }` (a lost flip cancels everything; win-count tiers fire in order — Fiery Gambit). Per-flip `RemoveFromCombat`/`PhaseOut` payoffs ship Mijae Djinn, Ydwen Efreet, Frenetic Efreet; copy-or-bounce-your-spell on flip ships Krark, the Thumbless. Remaining ⏳: opponent-chooses-half flips (Karplusan Minotaur). (AutoDecider now flips a real random coin; scripted tests stay deterministic.)
- ✅ **CR 309 / 701.49 — Dungeons & Venture** — `base::dungeons` (all three
  AFR dungeons), `Effect::Venture` (enter/advance with `ChooseMode` branch
  picks; room abilities resolve inline), `Player.{dungeon,dungeons_completed}`,
  `EventKind::DungeonCompleted` (battlefield + graveyard dispatch — Dungeon
  Crawler), `Value::DungeonsCompleted` (Cloister Gargoyle). Tests `tests/afr.rs`.
  Remaining ⏳: room abilities don't use the stack; Tomb's two pay-or-lose
  rooms are flat life loss; Mad Wizard's Lair free-cast collapsed to the draws.
- 🟡 **CR 122 — Counters** — defense counters / Battle type (122.1g) ✅ (`CounterType::Defense`, CR 310). Counter-clear on zone change (122.2) ✅ strict — cleared at every zone-change funnel; dies-with-counters triggers read the `died_card_snapshots` / `leaves_bf_lki` LKI caches (Felisa, Ambitious Augmenter). `-0/-1` / `-1/-0` counter types ✅. Counter-removal as an activation gate ✅ — `CounterType::Fuse` + an `ActivatedAbility.condition` on `Value::CountersOn` ≥ N (Goblin Bomb's "remove five fuse counters: deal 20"). "Choose a kind of counter at random it doesn't have" ✅ via `Effect::AddRandomMissingCounter` (keyword counters + +1/+1, never duplicating a present kind; respects Solemnity — Crystalline Giant). Return-a-died-creature-with-a-keyword-counter ✅ — a `CreatureDied`/`AnotherOfYours` trigger `Move`s `Selector::TriggerSource` (its gy card) back to the battlefield, then `AddKeywordCounter` on `Selector::LastMoved` (Luminous Broodmoth's flying counter; `luminous_broodmoth_returns_with_flying`). CR 614.16 additive replacement for *every* counter kind ✅ — `StaticEffect::ExtraCounterAllKinds` (Winding Constrictor) adds one to any counter placed on your creatures, via `GameState::scaled_counter_count`; composes with Hardened Scales (+1/+1-only) and Doubling Season. The player-counter "counters you'd get" half now covers energy **and** experience (`AddExperience` honors `extra_any_kind_adders_for`; `cr_614_16_winding_constrictor_boosts_experience`). Poison now scales too ✅ — `GameState::scaled_player_counter_count` (adder + doublers) routes every poison site (AddPoison, AddCounter(Player), Infect/Toxic combat); `cr_614_16_winding_constrictor_boosts_poison`. Keyword counters granting the keyword via layers ✅; test `cr_122_1_keyword_counter_grants_keyword` (Gift of the Viper). "Enters with N counters" ✅ (`CardDefinition.enters_with_counters` — Argent Dais's two oil; `cr_122_1_permanent_enters_with_printed_counters`). CR 122.5 relocation now moves **keyword counters** too — `Effect::MoveAllCounters` drains the separate `keyword_counters` map alongside `counters` (Reluctant Role Model; `cr_122_5_move_all_counters_relocates_keyword_counters`). CR 122.6 "remove up to N counters" ✅ — `Effect::RemoveCountersUpTo { what, amount }` drains any kinds from a permanent (greedy) or poison from a player (Price of Betrayal; `price_of_betrayal_strips_permanent_counters`, `_strips_player_poison`).
- 🟡 **CR 401 — Library** — play-with-top-revealed + play/cast-from-top ✅
  (401.5/401.6 — `StaticEffect::{TopOfLibraryRevealed,PlayFromLibraryTop}` plus
  the turn-scoped `Player.play_from_top_this_turn` grant
  (`Effect::GrantPlayFromTopThisTurn` — The Belligerent), both honored by
  `library_top_playable` + `known_library_top`/HUD chip; Courser, Oracle of Mul
  Daya, Mystic Forge). Remaining: the mid-cast "new top stays hidden until
  the spell finishes" timing nuance (401.5 second sentence); multi-card
  same-position picker (401.4). (401.7 `LibraryPosition::FromTop` ✅.)
- 🟡 **CR 706 — Rolling a Die** — ignore-roll riders (the roll-extra-and-
  ignore-lowest replacement now also covers `Effect::RollAndStoreDice`, CR
  706.2 — `cr_recent82::cr_706_2_*`). Stored rolls (706.8) ✅
  (`CardInstance.stored_die_results`, `Effect::{RollAndStoreDice,
  RerollStoredResults}`, `Value::GreatestSameStoredResult` — Centaur of
  Attention; `cr_706_8_*`). Roll trigger (706.6) ✅ — `EventKind::RolledDice`/`GameEvent::DiceRolled { player, count, high }` fires once per roll instruction ("whenever you roll one or more dice"). Result-referencing effects ✅ via `Value::LastDieRoll` (706.4 — Ancient Copper Dragon). **Result-gated triggers ✅** — `Predicate::DieResultAtLeast(n)` filters a roll trigger on the roll's greatest result (Ground Pounder's "roll a 5+ → trample"), reading `DiceRolled.high` through `event_amount`. (modifier / reroll-at-most / doubles ✅.) Remaining ⏳: ignore/reroll-replacement riders; the CR 706.8b reroll is auto-chosen (keep the most common face, reroll the rest) rather than prompting per result.
- 🟡 **CR 707 — Copying Objects** — in-place copy (707.4); MDFC-face copy (707.8); static copy effects (707.2c); copied "as enters" choices (707.6); spell-copy exceptions (707.9). (Enter-as-copy "except it's also [type]" ✅ via `EntersAsCopy.extra_card_types` — Phyrexian Metamorph copies any artifact/creature and stays an artifact. Token-copies with haste + delayed sacrifice ✅ via `Effect::CreateTokenCopiesHasteSac` — Devastating Onslaught's X copies, CR 707.2 + 111 + 701.16. `CreateTokenCopyOf` now takes `override_colors` (exact-color copy — Ardyn's 5/5 black Demon) and `enters_tapped` (Sin's tapped copies; `cr_707_2_token_copy_enters_tapped`).)
- 🟡 **CR 205 / 613.4 — Adding subtypes** — `Effect::AddCreatureTypes` grants
  creature types *in addition* to a permanent's own via a layer-4 additive
  `AddCreatureType` (Jenova, Ancient Calamity's "becomes a Mutant in addition to
  its other types"; `jenova_buffs_and_grants_mutant`), complementing
  `BecomeCreatureType`'s full set. Remaining: adding card types/supertypes via
  the same one-shot shape.
- 🟡 **CR 506 — Combat Phase** — remove-from-combat ✅ (506.4 — `Effect::RemoveFromCombat` pulls a targeted attacker/blocker out of combat, releasing its blockers; Labyrinth of Skophos, test `cr_506_4_*`). **Skip-combat ✅** (`Effect::SkipNextCombatPhase` + `Player.skip_next_combat`; `advance_step` jumps Begin Combat → postcombat main when the active player has a charge — Stonehorn Dignitary; tests `cr_506_active_player_skips_their_combat_phase`, `cr_506_skip_only_eats_one_combat`). Surfaced in `PlayerView.skip_next_combat` + a "⚔ skip" client chip. "block as though" restrictions (506.6); combat-step cast-timing gates (506.7). `PlayerRef::DefendingPlayer` now resolves off the *triggering attacker* for `YourControl`-scoped Attacks triggers (not just the ability source), so "whenever a creature you control attacks, defending player loses N" fires correctly (Leeching Sliver, CR 509.2). Combat-damage-to-player triggers now carry the damage dealt as `event_amount` (CR 119.3), so `Value::TriggerEventAmount` riders scale by the hit (Visions of Brutality). Such triggers now also **auto-target a graveyard card** when their effect prefers one (`prefers_graveyard_target`) instead of always binding slot 0 to the damaged player — Efreet Flamepainter recasts an instant, Venerable Warsinger reanimates a creature. (`CopySpell` / `CastWithoutPayingImmediate` are now surfaced by `primary_target_filter`, so on-cast self-copy and gy-recast triggers auto-target correctly; `CastWithoutPayingImmediate` accepts a `Permanent` entity-ref for the targeted gy card.)
- 🟡 **CR 508.1a — Attack restrictions** — the keyword gate list now covers "can't attack if it attacked during your last turn" (`Keyword::CantAttackIfAttackedLastTurn`, off the `attacked_own_turn` → `attacked_last_turn` untap roll-over) and a one-turn ban armed by an effect (`Effect::CantAttackNextTurn` + `CardInstance.attack_ban`, promoted at the bearer's untap and cleared the turn after). Both surface as `PermanentView.cant_attack_this_turn`. Tests `cr_508_1a_attacked_last_turn_restriction_lifts_after_one_turn`, `wall_of_dust_benches_what_it_blocks`. Remaining: the restriction list is a hand-written match rather than a general predicate.
- 🟡 **CR 508.3a — Put onto the battlefield attacking** — `Effect::CreateTokenAttacking`
  (tokens) and `Effect::JoinCombatAttacking { what }` (existing permanents — a
  reanimated/blinked creature joins combat tapped + attacking; Alesha, Who
  Smiles at Death reanimates via `Move→Battlefield` + `JoinCombatAttacking`).
  Remaining: choose the attacked defender/planeswalker (currently follows the
  source's attack, else the first opponent).
  (token attackers — Mobilize/Myriad) and `Effect::LookTopMayDeployAttacking`
  (deploy a real library card tapped-and-attacking with indestructible EOT,
  bottom the rest in random order per 401.4 — Winota) both join the current
  combat by pushing onto `attacking` past the declare-attackers gate. Remaining:
  a controller's-choice defender pick (currently follows the triggering creature).
- ✅ **CR 606 — Loyalty Abilities** — sorcery-speed, once-per-turn-per-walker gating ✅; loyalty-set effects ✅ (`Effect::SetLoyalty`); variable `-X` loyalty ✅ (606.5 — `LoyaltyAbility.x_cost`, `ActivateLoyaltyAbility { x_value }`, body reads `Value::XFromCost`; Kasmina); opponent loyalty-activation tax ✅ (`StaticEffect::OpponentLoyaltyActivationTax`, paid as extra generic mana — Eidolon of Obstruction, test `cr_606_eidolon_*`). Instant-speed-the-turn-it-entered ✅ (606.3b — `CardDefinition.flash_loyalty` + `entered_turn` gate skips the sorcery-speed check while it's the entry turn; The Wandering Emperor, test `wandering_emperor_flash_loyalty_window`). Remaining ⏳: unconditional "activate any time" riders; a UI `Decision::ChooseAmount` X prompt.
- 🟡 **CR 701.45 — Learn** — reveal-Lesson / discard-to-draw decision ✅; the in-graveyard "if you would learn, you may instead return this" replacement ✅ via `StaticEffect::MayReturnFromGraveyardInsteadOfLearn` consulted at the top of `Effect::Learn` (Retriever Phoenix). Remaining ⏳: Lesson sideboard population in some deck-build paths.
- ✅ **CR 701.12 — Exchange (control)** — `Effect::ExchangeControl { a, b }` swaps the controllers of two resolved permanents simultaneously (Switcheroo). Exchange-life-totals + exchange-hand/graveyard already ✅. Vedalken Plotter ✅ via `Effect::ExchangeControlChoosing` (controller picks their own permanent at resolution, the opponent's is the cast target). Remaining ⏳: an *until-end-of-turn* exchange variant.
- ✅ **CR 701.16 — Sacrifice** — `GameEvent::CreatureSacrificed`/`PermanentSacrificed` distinct from the lethal-damage/`Destroy` die path; `EventKind::CreatureSacrificed` triggers fire only on genuine sacrifice (Mortician Beetle). Targeted sacrifice of an already-chosen permanent ✅ via `Effect::SacrificePermanent { what }` (fires sacrifice + death triggers; Footsteps of the Goryo / Apprentice Necromancer sacrifice their reanimated creature at the next end step; `cr_701_16_targeted_sacrifice_fires_death_triggers`). Pay-mana-value-or-sacrifice-the-source ✅ via `Effect::SacrificeSourceUnlessPayManaValue` (Soul Tithe's upkeep tithe, granted to the enchanted permanent; the controller keeps it by auto-tapping its mana value, else it's sacrificed — `soul_tithe_*`). Remaining ⏳: batched multi-permanent sacrifice-cost picker. (Audit follow-up closed — the P1 death-funnel bypass family is fixed; all arms route through the shared funnels.)
- ✅ **CR 700.13 — Commit a crime** — `EventKind::CommittedCrime` /
  `GameEvent::CommittedCrime` fires once per spell-cast or ability-activation
  whose chosen targets include an opponent, a permanent/card an opponent
  controls or owns, or a spell they control (detected at the cast / activate
  choke points via `target_is_crime`). `Player.committed_crime_this_turn` +
  `Predicate::CommittedCrimeThisTurn` back "if you've committed a crime this
  turn" gates. Ships Gisa, Magda, Marchesa, Forsaken Miner, Nimble Brigand
  (`decks::recent20`). ⏳: "commit a crime" by an ability targeting a spell/
  ability an opponent controls (only spell targets are checked on the stack).
- ✅ **CR 701.35 — Detain** — `Effect::Detain { what }` + `CardInstance.detained_by`; a detained permanent can't attack/block (combat gates) or have its abilities activated (`activate_ability` gate), lifting at the detainer's next turn (`do_untap`). Surfaced in `PermanentView.detained` + a client tooltip badge. Ships Lyev Skyknight. ⏳: granted "enters detained" statics. (Loyalty activation now honors `detained_by`; Detain's target filter is enforced at cast time.)
- ✅ **CR 701.29 — Fateseal** — `Effect::Fateseal { who, amount }`: look at the top N of a targeted opponent's library, the controller may bottom any (Scry's library-side mirror). Decided inline (the `wants_ui` suspend prompt is a follow-up).
- 🟡 **CR 614 — Replacement Effects** — general "instead" framework. Damage *halving* ✅ (614.5 — `StaticEffect::HalveDamageDealt`, Ghosts of the Innocent; composed with doublers via `scale_damage` at both damage funnels). Skip-step (614.10) ✅ via `StaticEffect::SkipStep` consulted in `advance_step` — a skipped upkeep/draw never occurs (no turn-based actions, triggers, or priority); a skipped untap skips untapping/phasing/day-night but the turn still starts (Eon Hub, Stasis). Skip-*turn* ✅ (`Player.skip_turns`, Chronatog / Ral Zarek -7). Damage *redirection* (614.9) ✅ via `StaticEffect::RedirectDamageToSelf` at both damage funnels (Palisade Giant; one redirect per event per 614.5). (ETB-counters, token/counter/damage *doubling*, regen, EtbTriggerTax, Maze-of-Ith per-source prevention ✅. Creature-ETB / death **trigger suppression** ✅ via `StaticEffect::SuppressCreatureEtbTriggers { also_dies }` — Torpor Orb / Tocatli Honor Guard / Hushbringer; `etb_trigger_multiplier` returns 0 for creature entrants and the dies-trigger gather paths skip while a suppressor is in play.) Enters-*untapped* replacement ✅ — `StaticEffect::LandsEnterUntapped` overrides any enters-tapped effect for the controller's lands in `apply_enters_tapped_replacement` (Spelunking).
- 🟡 **CR 615 / 614.9 — Prevention & redirection** — source+target-scoped prevention ✅ (`PreventDamageToYourCreaturesFromYourSources` — Light of Sanction; `PreventThisDamageToColor` — Indentured Oaf's own damage to red creatures; both wired into the combat + noncombat funnels — `cr_recent14`). Damage **redirection** (614.9) ✅ — `Effect::RedirectNextDamage` + `PreventionShield.redirect_to` deals the soaked N to a chosen permanent (Carom, Razia); `RedirectControllerDamageToEquippedCreature` sends a player's damage to the equipped creature (Pariah's Shield). Global "combat damage can't be prevented" ✅ (`StaticEffect::CombatDamageCantBePrevented` — Frenzied Baloth; bypasses shields for any creature-sourced damage, sharing the Questing-Beast combat approximation). Source-scoped "damage dealt by this can't be prevented" ✅ (`StaticEffect::SourceDamageCantBePrevented` — Excruciator; keyed on the damage source in `apply_prevention_shields`, so only its own damage bypasses shields — `cr_615_12_excruciator_source_scoped_unpreventable`). Per-source / per-N shields ✅ (`PreventionShield.source` + `Effect::PreventNextDamageFromChosenSource` — Wojek Apothecary, Stave Off). Prevented damage can now be **redirected to a player**, not just a permanent (`PreventionShield.redirect_to_player` — Acolyte's Reward at face). Non-combat prevention breadth — Mending Hands ✅ (next-4 shield on any target); prevent-and-gain ✅ via `Effect::PreventNextDamageAndGainLife` + `PreventionShield.gain_life` (Reverse Damage, Candles' Glow — `candles_glow_prevents_and_gains`). Attachment-scoped combat fog ✅ (`StaticEffect::PreventAllCombatDamageToAttached` — General's Kabuto carries the prevention for its host). Player-scoped combat fog ✅ (`Effect::PreventAllCombatDamageToPlayerThisTurn` — "prevent all combat damage that would be dealt to you this turn", Druid's Deliverance; `GameState.combat_damage_prevented_to_players_this_turn`, honored in `prevent_combat_to_target` — `druids_deliverance_prevents_combat_damage_to_you`). Player+permanents noncombat prevention ✅ (`StaticEffect::PreventNoncombatDamageToYouAndYourPermanents` — The Wanderer; gates the noncombat funnel for both the controller and any permanent they control — `the_wanderer_prevents_noncombat_damage_to_you`). Source-of-your-choice prevention (615.7) ✅ via
  `Effect::PreventAllDamageFromChosenSourceThisTurn` +
  `GameState.damage_prevented_sources`, consulted at both damage funnels
  (Burrenton Forge-Tender; the source is chosen as the ability resolves,
  among stack spells and battlefield permanents). Per-shield source
  restriction ✅ — `PreventionShield.{source,one_event}` +
  `Effect::PreventNextDamageFromChosenSource` (the damage source is now
  threaded through `apply_prevention_shields` at both funnels; Circle of
  Protection cycle, Rune of Protection: Red/Black). Blanket controller immunity
  ✅ — `StaticEffect::PreventAllDamageToController` (Glacial Chasm) at the
  player-directed branch of both funnels; surfaced as `PlayerView
  .damage_fully_prevented` + a client "🛡 immune" chip. Your-creatures noncombat
  immunity ✅ — `StaticEffect::PreventNoncombatDamageToYourCreatures` (Mark of
  Asylum; noncombat-only because combat damage to creatures is marked off the
  shared funnel). Turn-scoped incoming-only combat prevention ✅ —
  `Effect::PreventCombatDamageToTargetThisTurn` + `GameState
  .combat_damage_prevented_to_this_turn`, consulted at the
  `combat_damage_prevented_to_self` chokepoint (Fleeting Flight; the creature
  still deals its own combat damage). Remaining ⏳: outgoing-only combat
  prevention; per-source combat shields for a single creature.
- 🟡 **CR 500 — Turn structure** — `Predicate::CurrentStepIs(TurnStep)` gates "activate only during [your] upkeep/end step" abilities (Mirror Universe, Magus of the Mirror). Extra **combat-phase** insertion ✅ (CR 505.1b — `AdditionalCombatPhase` at End of Combat + `AdditionalCombatPhaseAfterMain` post-main re-entry, Relentless Assault). Extra **upkeep steps** ✅ (CR 500.9 — `Effect::AdditionalUpkeepStep` + `Predicate::IsFirstUpkeepThisTurn`; Paradox Haze, `cr_500_9_*`). Remaining ⏳: extra draw/main steps (no card yet needs them).
- 🟡 **CR 305 — Lands** — see git for the per-clause detail. `LandType::Cave`
  added (CR 305.6 land subtypes), unblocking the LCI Cave lands + Caves-matter
  payoffs (Forgotten Monument grant, Compass Gnome tutor, Gargantuan Leech
  affinity, Spelunking). One-shot additive basic-land-type grant ✅
  (`Effect::GainAllBasicLandTypes` — layer-4 `AddLandType` ×5 per resolved land,
  CR 305; Energybending, `energybending_fixes_lands_and_draws`). Counter-gated
  land-type static ✅ (CR 305.7 — `StaticEffect::LandTypeChangerWhileCounters`
  only materializes while the source holds ≥N of a counter kind; Zhao, the Moon
  Slayer — "nonbasic lands are Mountains while Zhao has a conqueror counter";
  `zhao_taps_nonbasics_and_conquers_to_mountains`). As-enters *chosen*-basic-type
  additive static ✅ (CR 305.6/305.7 — `Effect::ChooseBasicLandTypeForSource`
  stamps `CardInstance.chosen_land_type`, `StaticEffect::LandsYouControlAreChosenType`
  adds it to your lands with the intrinsic mana ability following; Realmwright,
  `cr_305_6_realmwright_land_taps_for_chosen_color`).
- 🟡 **CR 701.48 — Learn** — populate Lesson sideboards in the format / draft deck-build paths (engine + cube ✅).
- 🟡 **CR 702.15 — Lifelink** — LKI corner (702.15c): triggered-ability source leaving the battlefield mid-resolution.
- 🟡 **CR 701.34 — Proliferate** — permanents' counters + player poison ✅;
  player experience/energy ✅; "whenever you proliferate" triggers ✅
  (`EventKind::Proliferated`, fires once per instance, incl. from the
  graveyard — Voidwing Hybrid); "proliferate twice instead" ✅
  (`StaticEffect::ProliferateTwice`, 2^n for n Tekuthals). Remaining:
  per-player UI choice of which permanents/players to proliferate.
- 🟡 **CR 601 — Casting Spells** (logged as "CR 706 — Casting spells") — minor; see git. Symmetric off-turn cast lock ✅ (`StaticEffect::PlayersCastOnlyOnOwnTurn` — Dosan the Falling Leaf gates every seat that isn't the active player, its controller included; `dosan_locks_off_turn_casts_for_both_seats`). "Opponents can't cast from anywhere but their hands" ✅ via `StaticEffect::OpponentsCantCastFromAnywhereButHand`, checked in `cast_from_zone_blocked`. The foretell / plot / adventure-creature exile-cast paths now gate on it too (`cast_foretold`/`cast_plotted`/`cast_adventure_creature`; test `drannith_magistrate_blocks_foretold_cast`). Suspend's eventual cast gates on the same lock ✅ (`cast_card_for_free` → `cast_from_zone_blocked`; test `cr_702_62e_suspend_final_cast_blocked_by_drannith`). CR 601.2 "unless"-cost affordability: `punisher_option_affordable` now rejects an empty-hand `Discard` dodge (can't choose a cost you can't pay), so a hand-empty player takes the penalty (`perforating_artist_*`, `osseous_sticktwister_delirium_punisher`). CR 702.8 flash-timing: the cast-timing check now honors the `ControllerSorceriesAsFlash` static (was a no-op — only `ControllerSpellsHaveFlash` was consulted), so Teferi, Time Raveler's static and Hypersonic Dragon let their controller cast sorceries at instant speed (`teferi_static_grants_controller_sorceries_as_flash`); the six duplicated `flash_granted` blocks collapsed into one `battlefield_grants_flash` helper.
- 🟡 **CR 117.1 — Order of priority** — APNAP corner cases; see git.
- 🟡 **CR 301 — Artifacts** — see git.
- 🟡 **CR 800 — Multiplayer / leaving the game** — see git.
- 🟡 **CR 903 — Commander Variant** — 903.4d back-face identity ✅; 903.4
  color-indicator + activated-ability-cost + adventure/split-half identity ✅
  (`format::color_identity` unions them; `cr_903_4_identity_*`). Remaining:
  903.9 optional rider.

### Todo (⏳)
- ✅ **CR 314 / 900 / 904 — Archenemy.** `CardType::Scheme` +
  `Supertype::Ongoing`; `Player.scheme_deck`, `GameState.archenemy` and
  `seat_archenemy` (40 life, first turn, CR 904.5/904.6). CR 904.9's
  set-in-motion is a turn-based action at the archenemy's precombat main
  (`set_scheme_in_motion` + `EventKind::SetInMotion`); CR 904.10's sweep is an
  SBA (`sweep_finished_schemes`); CR 701.33 abandon ships as
  `Effect::AbandonThisScheme`. A face-up scheme's statics and step triggers
  function from the command zone (CR 904.8) — anthem gather and
  `fire_step_triggers` both walk it. `sets::arc` (8 schemes),
  `classic_sets/arc`. Residual ⏳: the CR 904.2 team/attack-multiple-players
  seating is left to the caller, and All in Good Time's "schemes can't be set
  in motion that turn" rider isn't modeled.
- ✅ **CR 612 — Text-Changing Effects** — layer-3 `Modification::ReplaceColorWord`
  / `ReplaceBasicLandType` + `Effect::ReplaceColorWord`/`ReplaceBasicLandType`
  (two ChooseColor prompts pick from/to; basics map 1:1 onto colors). Rewrites
  Protection-from-color, landwalk, and the type line (a swapped basic taps for
  the new color). Trait Doctoring (EOT + Cipher), Mind Bend (permanent).
  Remaining ⏳: full text-box swaps (Spy Kit, Volrath's Shapeshifter) and
  ability-text color words beyond keywords.


# Tooling

## Recommender: two builder defects fixed, one lesson recorded

Both were found by asking why Emeritus of Ideation never appeared in a
build for a pool that contained it (2026-08-04). Fixed behind
`SimConfig::builder_v2`; see FEATURE_ROADMAP Tier 13 for the measured
adoption. Recorded here because the *consequence* outlived the fix:

- Every recommendation produced before this — including per-card
  attribution tables — came from a builder that could not see power,
  toughness, keywords, or a card's attached preparation spell. Re-run
  any archived recommendation before trusting its card rankings.
- `recommend_pool`'s anchor lens (`per_card_attribution_within`) reports
  over the *surviving* population only. A shape eliminated in racing
  contributes no variants, so "0 variants play it" means "no survivor is
  that color", not "the card is bad". The lens now matches anchor names
  case-insensitively — an exact-match miss used to return an empty
  subset silently, which reads identically to a real negative result.
