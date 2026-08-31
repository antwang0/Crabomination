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

- 🟡 **Sand Golem's "return this card with a +1/+1 counter" could not be shown
  to fire, and the reason is unestablished.** The counter itself was missing
  from the tree and is fixed; the trigger under it is the open half.
  `on_forced_discard` (Mangara's Blessing, Sand Golem) is
  `EventKind::OpponentCausedYouToDiscard` + `EventScope::SelfSource`, and
  neither half of that reads as reachable: `event_matches_spec`'s SelfSource
  arm is an explicit `matches!` chain over ~40 events that **does not name
  `OpponentCausedYouToDiscard`**, and the dispatcher's graveyard walk
  (`mod.rs`, "Also walk every player's graveyard") admits a `SelfSource`
  trigger only for `CardCycled` / `CardMilled` / `CardDiscarded` /
  `PutIntoGraveyard`. Pure Intentions' idiom — `CardDiscarded` + `SelfSource`
  + `Predicate::CausedByOpponentSpellOrAbility` — is the one that demonstrably
  works, and **swapping Sand Golem onto it did not make the trigger fire in a
  Mind Rot test either**, so the blocker is not (only) the event kind.

  What *was* established, and is the reason this is filed rather than fixed:
  `sok3::pure_intentions_returns_opponent_forced_discards` is **not** vacuous
  (probed: Pure Intentions reaches the graveyard, and the Forest added beside
  the Bear is discarded and returned, `hand 2 / gy 1`), so the family works
  for a card that was already in the graveyard before the discard. Sand Golem
  is the case where the source **is** the discarded card, and that is the
  distinction to test next. It is not a dead *arm*, so `audit_incomplete`
  cannot see it and neither can `audit_oracle_verbs`.

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
- ⏳ **Bot matches aren't reproducible.** `HeuristicBot` draws from the global
  RNG, so `bot_vs_bot_commander_demo_terminates` varies 0.5s–15s+ run to run
  and occasionally blew its old 120s ceiling. The ceiling is now 600s, but
  it's a *wall-clock* budget inside a test binary that runs 450 other tests
  in parallel: under a loaded `cargo test --workspace` the binary takes ~620s
  and the assertion trips even though the same test finishes in ~50s alone.
  The real fix is a seeded RNG on `HeuristicBot` (with the seed printed on
  failure) and an action-count ceiling instead of a clock.
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
