# Strixhaven implementation tracker

This file tracks two adjacent Strixhaven catalogs:

1. **Secrets of Strixhaven (SOS)** — the 255-card supplemental set, parsed
   from Scryfall set `sos`. Cards live under `catalog::sets::sos` and are
   listed in the per-color tables below.
2. **Strixhaven: School of Mages (STX)** — the 2021 base set. Cards live
   under `catalog::sets::stx` and are listed in their own table at the end
   of this file (see "Strixhaven base set (STX)" below).

## Legend

- ✅ done — wired in `crate::catalog` with full functionality
- 🟡 partial — exists with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Set Overview

| Color | Cards |
|---|---|
| White | 32 |
| Blue | 32 |
| Black | 33 |
| Red | 30 |
| Green | 32 |
| Prismari (Blue-Red) | 17 |
| Witherbloom (Black-Green) | 17 |
| Silverquill (White-Black) | 16 |
| Quandrix (Green-Blue) | 16 |
| Lorehold (Red-White) | 16 |
| Colorless | 14 |

## Implementation Progress

Counts reflect the regenerated tables below (audited via
`scripts/audit_strixhaven2.py` against `catalog::sets::sos`).

Audit script numbers (per `scripts/audit_strixhaven2.py`):
- ✅ done: **121** (push XXIX: +11 SOS cards from 🟡 → ✅ via Opus /
  Increment / mana-spent rider wirings — Spectacular Skywhale, Cuboid
  Colony, Hungry Graffalon, Muse Seeker, Aberrant Manawurm, Tackle
  Artist, Thunderdrum Soloist, Molten-Core Maestro, Expressive
  Firedancer, Deluge Virtuoso, Exhibition Tidecaller. Push XXVIII:
  Dina's Guidance, Pursue the Past, Heated Argument, Stadium
  Tidalmage, Witherbloom Charm, plus 4 MDFCs verified end-to-end.)
- 🟡 partial: **133** (push XXIX: -11 from promotions, Colorstorm
  Stallion and Elemental Mascot stay 🟡 since the big-body branches
  still need permanent-copy / cast-from-exile primitives.)
- ⏳ todo: **1** (Improvisation Capstone — still needs the
  cast-from-exile pipeline + copy-spell primitive)

Plus the STX side adds 3 promotions this push (Daemogoth Woe-Eater +
Witherbloom Pledgemage + Honor Troll all 🟡 → ✅) via `Effect::MayDo`
on optional-sac triggers and the new `Honor Troll` compute-time
conditional-anthem injection in `compute_battlefield`.

All ✅ and 🟡 cards have a corresponding factory in
`crabomination/src/catalog/sets/sos/`; the audit script reports 0 false
positives and 0 stale ⏳ rows.

## 2026-05-13 push XXIX: Prismari Opus rider closure + Increment promotions + STX table sync

Push XXIX (`claude/modern_decks` branch) — wires three Prismari cards
that were body-only (Spectacular Skywhale, Colorstorm Stallion,
Elemental Mascot) with the proper Opus rider via `shortcut::opus_trigger`
(promoting 1 to ✅, leaving 2 at 🟡 since their big-body branches need
primitives we don't have yet). Also promotes ~12 stale 🟡 → ✅ rows in
the doc where the underlying code was already complete via
`shortcut::increment_self_plus_one()` /
`shortcut::opus_trigger(small, big)` / `Predicate::CastSpellManaSpent`
introspection. Tests at 1192 (+4 net — added 4 Opus / Skywhale rider
tests).

### Engine usage (no new primitives — only call-site wiring)

- **`shortcut::opus_trigger(small, big)`** — wraps the standard Opus
  pattern: a `SpellCast / YourControl` trigger filtered on
  `cast_is_instant_or_sorcery()` plus an `Effect::If` whose
  `Predicate::CastSpellManaSpentAtLeast(5)` arm fires the bigger body.
- **`shortcut::increment_self_plus_one()`** — wraps Increment's
  intervening-if pattern: a SpellCast/YourControl trigger filtered on
  `Predicate::IncrementSatisfied` plus an `Effect::If` re-check at
  resolution (matches CR 603.4).

### Newly promoted ✅ (was 🟡)

- **Spectacular Skywhale** (SOS Prismari) 🟡 → ✅ — Opus rider fully
  wired. Small: +3/+0 EOT. Big (≥5 mana): three +1/+1 counters instead.
- **Cuboid Colony** (SOS Quandrix) 🟡 → ✅ — Increment wired (was
  already in code; doc was stale).
- **Hungry Graffalon** (SOS Green) 🟡 → ✅ — Increment wired.
- **Muse Seeker** (SOS Blue) 🟡 → ✅ — Opus loot rider wired.
- **Aberrant Manawurm** (SOS Green) 🟡 → ✅ — mana-spent pump wired.
- **Tackle Artist** (SOS Red) 🟡 → ✅ — Opus +1/+1 (small) / +2/+2 (big)
  counters wired.
- **Thunderdrum Soloist** (SOS Red) 🟡 → ✅ — Opus ping each opp
  (1 small / 3 big) wired.
- **Molten-Core Maestro** (SOS Red) 🟡 → ✅ — Opus +1/+1 counter
  (small) + counter + R-mana-per-power (big) wired.
- **Expressive Firedancer** (SOS Red) 🟡 → ✅ — Opus +1/+1 EOT (small)
  / +1/+1 EOT + DoubleStrike (big) wired.
- **Deluge Virtuoso** (SOS Blue) 🟡 → ✅ — ETB tap+stun + Opus
  +1/+1 (small) / +2/+2 (big) EOT pump wired.
- **Exhibition Tidecaller** (SOS Blue) 🟡 → ✅ — Opus mill (3 small /
  10 big) wired.

### Partial promotions 🟡 (small body wired this push)

- **Colorstorm Stallion** (SOS Prismari) — Opus small body (+1/+1 EOT)
  wired. Big body (token-copy) still 🟡 pending permanent-copy primitive.
- **Elemental Mascot** (SOS Prismari) — Opus small body (+1/+0 EOT)
  wired. Big body (exile-and-play) still 🟡 pending cast-from-exile
  primitive.

### CR audit (push XXIX)

- ✅ **CR 506.4 — Permanent removed from combat on zone change**:
  "A permanent is removed from combat if it leaves the battlefield, …".
  Push XXIX adds the new `GameState::remove_from_combat(cid)` helper
  in `game/stack.rs`, called from `move_card_to`,
  `remove_from_battlefield_to_graveyard`, and
  `remove_from_battlefield_to_exile`. The helper prunes
  `self.attacking` (attacker slot) and `self.block_map` (both blocker
  keys and attacker values). Before this push, orphan attacker
  entries lingered until end of combat — combat damage resolution
  filter-mapped against the computed battlefield, but downstream
  selectors / trigger dispatchers could see stale entries. New
  test: `destroying_attacker_mid_combat_prunes_attacking_per_cr_506_4`.

- ✅ **CR 603.4 — Intervening 'if' clause**: "A triggered ability of
  the form 'When/Whenever/At [trigger event], if [condition], [effect]'
  checks the condition at the time the trigger event occurs and again
  when it resolves." Wired in `shortcut::increment_trigger` (calling
  `Predicate::IncrementSatisfied` both as `EventSpec.filter` and inside
  a resolution-time `Effect::If`). Test:
  `increment_trigger_re_checks_intervening_if_on_resolution`
  (push XVII).

## 2026-05-13 push XXVIII: STX Witherbloom promotions + SOS ⏳ closure + Honor Troll engine wiring (CR 614.10 audit)

Push XXVIII (`claude/modern_decks` branch) — promotes 3 STX cards from
🟡 → ✅ (Witherbloom Pledgemage, Daemogoth Woe-Eater, Honor Troll), 5
SOS cards from 🟡 → ✅ (Dina's Guidance, Pursue the Past, Heated
Argument, Stadium Tidalmage, Witherbloom Charm), and 2 SOS cards from
⏳ → 🟡 (Archaic's Agony, Quandrix the Proof). Tests at 1192 (+7 net).

### Engine improvements

- **Honor Troll conditional anthem (CR 122.1 / continuous statics)**: A
  compute-time injection in `GameState::compute_battlefield` (same
  pattern as Cruel Somnophage / Tarmogoyf) reads
  `Player.life_gained_this_turn` and, when > 0, layers in
  `Modification::ModifyPowerToughness(+2, +0)` and
  `Modification::AddKeyword(Lifelink)` targeting the Troll as
  `AffectedPermanents::Source`. The gate re-evaluates every recompute
  so a mid-turn lifegain flips the troll on for the rest of the turn,
  and `do_untap`'s `life_gained_this_turn = 0` reset flips it back off
  next turn. The pattern is reusable for Hofri-style anthems but stays
  hard-coded here since "self-gated conditional anthem" is only
  exercised by Honor Troll today.

- **`Effect::MayDo` on Daemogoth Woe-Eater attack trigger**: the
  printed "you may sacrifice another creature" is now wrapped in
  `Effect::MayDo`. AutoDecider says no, `ScriptedDecider::new([Bool
  (true)])` exercises the paid path. The ETB sacrifice remains
  mandatory (printed Oracle has no "you may").

- **`Effect::ChooseMode` on Dina's Guidance**: 🟡 → ✅. Mode 0 search →
  hand, mode 1 search → graveyard. AutoDecider defaults to mode 0
  (hand) for unguided bots; a Lorehold/Witherbloom reanimator deck
  picks mode 1 via scripted decision. No new primitive — just an
  extension of the existing 2-mode `ChooseMode` pattern.

### New SOS cards

- **Archaic's Agony** (R) 🟡 — `{4}{R}` Sorcery. Converge X damage to a
  creature target via `Value::ConvergedValue` (same primitive as
  Rancorous Archaic / Sundering Archaic). The "exile cards equal to
  excess damage + may play" rider is still omitted.

- **Quandrix, the Proof** (G/U) 🟡 — `{4}{G}{U}` 6/6 Legendary Elder
  Dragon, Flying + Trample. Body wired faithfully; Cascade and IS-grant-
  cascade static still ⏳ (no Cascade keyword primitive yet).

### CR audit (push XXVIII)

- ✅ **CR 502.4 — No priority during untap step**: "No player receives
  priority during the untap step, so no spells can be cast or resolve
  and no abilities can be activated or trigger." The engine's
  `advance_to_next_step` already handles this:
  `crabomination/src/game/stack.rs:62` — "Untap has no priority window
  — auto-execute and move on." The untap step runs `do_untap` then
  immediately calls `pass_priority` to step into Upkeep. State-based
  actions check happens in the SBA loop, not gated on priority. Test
  coverage is implicit (existing turn-progression tests run through
  the untap step without observing a priority window).
- 🟡 **CR 614.10 — Skip effects are replacement effects**: "An effect
  that causes a player to skip an event, step, phase, or turn is a
  replacement effect. 'Skip [something]' is the same as 'Instead of
  doing [something], do nothing.'" We have `skip_first_draw` for the
  start-of-game first-draw skip (CR 103.6), but no general
  skip-effect framework. Cards like Mind's Desire's "extra turn" or
  Time Sieve's "extra turns" depend on this. Tracked in TODO.md
  under "Replacement Effects".

## 2026-05-13 push XXVII: STX Commands cycle + utility cards + magecraft helpers (CR 700.2)

Push XXVII (`claude/modern_decks` branch) — adds the full Strixhaven
Command cycle (5 cards) plus 6 STX utility cards across the
Silverquill / Witherbloom / Lorehold / Quandrix / Prismari schools.
Tests at 1180 (+15 net). All cards land in `catalog::sets::stx::extras`.

### New STX cards

- **Witherbloom Command** (B/G) 🟡 — `{2}{B}{G}` Sorcery. `ChooseMode`
  collapse: mill 4 / destroy noncreature-nonland MV ≤ 2 / drain 2 /
  Indestructible EOT (regen approximation).
- **Lorehold Command** (R/W) 🟡 — `{2}{R}{W}` Sorcery. `ChooseMode`
  collapse: 4 dmg to opp / -2/-0 EOT / return creature card from gy →
  hand / mint two 2/2 R/W Spirits **with flying** (Lorehold STX
  Spirits include Flying).
- **Quandrix Command** (G/U) 🟡 — `{1}{G}{U}` Instant. `ChooseMode`
  collapse: two +1/+1 counters / `CounterAbility` / mill 2 / bounce
  nonland to owner's hand.
- **Silverquill Command** (W/B) 🟡 — `{2}{W}{B}` Instant. `ChooseMode`
  collapse: `CounterAbility` / drain 2 / return MV ≤ 2 permanent card
  from your gy → bf / two +1/+1 counters.
- **Prismari Command** (U/R) 🟡 — `{1}{U}{R}` Instant. `ChooseMode`
  collapse: 2 dmg to any target / loot 1 / Treasure token / destroy
  artifact.
- **Defend the Campus** (W) ✅ — `{3}{W}{W}` Sorcery. Mints three 1/1
  W/B Inkling tokens with flying via `Effect::CreateToken { count: 3 }`.
- **Hall Monitor** (W) ✅ — `{W}` 1/1 Human Cleric. Magecraft: untap
  this creature.
- **Stonebinder's Familiar** (Colorless) ✅ — `{1}` 0/1 Artifact
  Creature — Spirit. Cards-leave-graveyard +1/+1 counter trigger,
  same shape as SOS Spirit Mascot.
- **Necrotic Fumes** (B) 🟡 — `{2}{B}{B}` Sorcery. Sacrifice a
  creature + exile target creature (additional cost collapses into
  resolution; net effect preserved).
- **Make Your Mark** (W) ✅ — `{1}{W}` Instant. +1/+1 EOT on creature,
  draw a card.
- **Containment Breach** (W) ✅ — `{1}{W}` Sorcery. Destroy enchantment
  + Surveil 1.

### Engine notes

- The Command cycle uses the standard `ChooseMode` single-mode-pick
  approximation (matching Witherbloom Charm, Moment of Reckoning, and
  the existing SOS modal cards). The "choose two" mega-pick is tracked
  in TODO.md as a future engine primitive.
- Lorehold Command needed a fresh `TokenDefinition` for the flying
  R/W Spirit since the SOS catalog's `lorehold_spirit_token()` is the
  no-flying STX-Lorehold default; both definitions coexist (different
  printings ship different keyword loadouts).
- Stonebinder's Familiar caught a subtle bug in my first wiring: I'd
  used `EventScope::SelfSource` for `EventKind::CardLeftGraveyard`,
  but `event_matches_spec` only handles `SelfSource` for
  `PermanentEntered / AttackerDeclared / CreatureDied / Blocks /
  BecomesBlocked / CounterAdded`. The correct scope for "cards leave
  YOUR graveyard" is `YourControl` (matched against the
  `event_actor`'s seat), which is how the existing SOS Spirit Mascot,
  Owlin Historian, etc. wire their analogous triggers.

## 2026-05-13 push XXVI: 12 new SOS cards + 1 STX promotion + EventKind::Blocks (CR 509.1i)

Push XXVI (`claude/modern_decks` branch) — wires 12 SOS cards out of ⏳
into the catalog (mostly body-only / partial wirings where the primary
effect needs primitives that don't exist yet), promotes Daemogoth Titan
(STX) from 🟡 to ✅ via the new `EventKind::Blocks` event (per CR 509.1i),
plus engine improvements. Tests at 1165 (+15 net).

### New SOS cards

- **Colorstorm Stallion** (UR) 🟡 — `{1}{U}{R}` 3/3 Elemental Horse,
  Ward(1) + Haste body. Opus rider omitted (mana-spent introspection +
  permanent-copy primitive).
- **Elemental Mascot** (UR) 🟡 — `{1}{U}{R}` 1/4 Elemental Bird, Flying
  + Vigilance body. Opus rider omitted.
- **Prismari, the Inspiration** (UR Legendary) 🟡 — `{5}{U}{R}` 7/7
  Legendary Elder Dragon, Flying + Ward(5) body. Storm static on IS
  spells omitted.
- **Campus Composer // Aqueous Aria** (U MDFC) 🟡 — Front 3/4 Merfolk
  Bard, Ward(1). Back: draw 3.
- **Emeritus of Ideation // Ancestral Recall** (U MDFC) 🟡 — Front 5/5
  Human Wizard, Ward(1). Back: {U} draw 3.
- **Grave Researcher // Reanimate** (B MDFC) 🟡 — Front 3/3 Troll
  Warlock with ETB Surveil 1. Back: {B} reanimate one creature from
  your graveyard.
- **Strife Scholar // Awaken the Ages** (R MDFC) 🟡 — Front 3/2 Orc
  Sorcerer, Ward(1). Back: {5}{R} mass-recursion of creatures from
  graveyard.
- **Choreographed Sparks** (R) 🟡 — `{R}{R}` Instant. Single-mode
  CopySpell against IS-on-stack (the "or copy a creature spell" branch
  needs a permanent-spell copy variant).
- **Flashback** (R, the SOS instant) 🟡 — `{R}` Approximated as "return
  a target IS card from your graveyard to your hand" — strictly weaker
  than the printed "flashback for its mana cost this turn" but
  preserves the spell-recovery outcome.
- **Echocasting Symposium** (U Lesson) 🟡 — `{4}{U}{U}` Mints a 3/3
  Wizard "Echocast" placeholder token (no permanent-copy primitive
  yet). Tagged as Lesson via SpellSubtype.
- **Applied Geometry** (GU) 🟡 — `{2}{G}{U}` Mints a 0/0 Fractal token
  with six +1/+1 counters → 6/6 Fractal for 4 mana.
- **Strixhaven Skycoach** (Colorless artifact) 🟡 — `{3}` 3/2 Vehicle
  artifact with Flying body + ETB basic-land tutor to hand. Crew not
  enforced (no crew-as-tap-cost primitive yet).

### STX promotion

- **Daemogoth Titan** (B) 🟡 → ✅ — Block-half sacrifice trigger now
  wired via the new `EventKind::Blocks/SelfSource` (push XXVI). Both
  the attack and block "sac another creature" halves resolve via
  `Effect::Sacrifice` against creatures you control; the auto-decider
  prefers lowest-power non-source candidates so a fresh Titan sacs
  fodder rather than itself. Test:
  `daemogoth_titan_blocks_sacrifices_another_creature`.

### Engine improvements

- **`EventKind::Blocks`** — first-class "whenever this creature blocks"
  event. Fires from the blocker's side of the `GameEvent::BlockerDeclared`
  event (CR 509.1i). The trigger dispatcher (`event_matches_spec`) now
  pattern-matches on `(EventKind::Blocks, BlockerDeclared)` and routes
  `SelfSource` scope by checking `blocker == source.id` (the prior
  `BecomesBlocked` matched against `attacker == source.id`). Both
  events share the same underlying `BlockerDeclared` payload; the
  dispatcher picks the right side via `spec.kind`.

### CR audit (push XXVI)

- ✅ **CR 509.1i — Block triggers fire on blocker declaration**:
  "Once the chosen creatures are declared as blockers, any abilities
  that trigger on blockers being declared trigger." Our
  `declare_blockers` emits one `GameEvent::BlockerDeclared` event per
  blocker, and `dispatch_triggers_for_events` now picks up
  `EventKind::Blocks` triggers off those events. Verified by
  Daemogoth Titan's block-half sacrifice trigger
  (`daemogoth_titan_blocks_sacrifices_another_creature`).

## 2026-05-12 push XXV: 7 new SOS cards + 1 STX card + Killian's promotion + combat-damage gy triggers

Push XXV (`claude/modern_decks` branch) — wires 7 SOS cards out of ⏳
into the catalog (body-only where the primary effect needs primitives
that don't exist yet) plus promotes **Killian's Confidence** from 🟡 to
✅ via the new `EventScope::FromYourGraveyard` extension on the combat-
damage-to-player trigger dispatcher. Adds Quick Study (STX U mono).
Tests at 1150 (+11 net).

New SOS cards (`catalog::sets::sos::*`):

- **Fix What's Broken** (W/B) 🟡 — {X}{2}{W}{B} Sorcery. Pay X life
  (folded into resolution as `LoseLife(XFromCost)`); for each artifact/
  creature card in your graveyard with mana value exactly X, return it
  to the battlefield. The MV=X equality is synthesized as
  `All([ValueAtLeast(MV, X), ValueAtMost(MV, X)])` since there's no
  `ValueEquals` predicate. Test exercises X=1 → Sol Ring (MV 1)
  returns; Grizzly Bears (MV 2) stays in the graveyard.

- **Silverquill, the Disputant** (W/B Legendary) 🟡 — {2}{W}{B} 4/4
  Legendary Elder Dragon Flying+Vigilance body. Casualty-1 grant
  static omitted (no Casualty keyword yet).

- **Nita, Forum Conciliator** (W/B Legendary) 🟡 — {1}{W}{B} 2/3
  Legendary Human Advisor body. "Cast a spell you don't own" trigger +
  cast-from-opp-graveyard activated ability omitted.

- **Biblioplex Tomekeeper** (Colorless) 🟡 — {4} 3/4 Construct artifact
  creature body. Prepare toggle omitted (no Prepare keyword yet).

- **The Dawning Archaic** (Colorless Legendary) 🟡 — {10} 7/7 Legendary
  Avatar with Reach. IS-in-gy cost reduction + attack-trigger cast-
  from-graveyard rider omitted.

- **Mica, Reader of Ruins** (R Legendary) 🟡 — {3}{R} 4/4 Legendary
  Human Artificer. Magecraft sac-an-artifact-to-copy-the-spell wired
  via `magecraft(MayDo + Seq(Sacrifice(Artifact, 1) + CopySpell{what:
  TriggerSource}))` — same template as Aziza, Mage Tower Captain's
  tap-three-to-copy. Two tests cover the yes-path (sacrifice fires
  copy) and no-path (decline keeps artifact).

- **Skycoach Waypoint** (Colorless Land) 🟡 — Land with `{T}: Add {C}`
  via `tap_add_colorless()`. {3},{T}: prepare-target omitted.

Engine extension:

- **`fire_combat_damage_to_player_triggers` walks graveyards for
  `EventScope::FromYourGraveyard`.** Previously, combat damage events
  to a player only fired triggers on the attacker's own battlefield
  card. Now, after the existing battlefield walk, every player's
  graveyard is scanned for triggers scoped `FromYourGraveyard` whose
  card owner matches the attacker's controller — matching the
  printed "creatures you control deal combat damage" filter on the
  attacker side. The trigger's effective controller is the gy owner
  and source binding is the graveyard card. This unblocks Killian's
  Confidence's "may pay {W/B} to return to hand on combat damage"
  rider, plus any future combat-damage-payoff sitting in graveyard
  (a niche but well-defined design space).

- **CR audit (push XXV): consulted **CR 603.7d / 603.10** — graveyard
  triggers and zone-change triggers.** A trigger's source on the
  battlefield is "this object as it last existed on the battlefield";
  the engine's `Selector::This` resolution under
  `FromYourGraveyard` correctly binds to the graveyard-resident card
  (ctx.source = card.id) so `Move(This → Hand)` finds it via the
  graveyard walk in `move_card_to`. The rule's "last existed on the
  battlefield" wording specifically applies to dies-style triggers,
  not graveyard-resident triggers fired from a card that was
  originally cast as a sorcery (Killian's Confidence pattern). Our
  binding matches both readings.

STX additions:

- **Quick Study** (U mono) ✅ — {1}{U} Instant: Target player draws
  two cards.

Tests added (11 net):

- `quick_study_draws_two_cards_for_target_player`
- `silverquill_the_disputant_is_a_four_four_flying_vigilance_dragon`
- `biblioplex_tomekeeper_is_a_three_four_construct`
- `the_dawning_archaic_is_a_seven_seven_legendary_avatar_with_reach`
- `nita_forum_conciliator_is_a_two_three_legendary_human_advisor`
- `skycoach_waypoint_taps_for_colorless`
- `fix_whats_broken_returns_mana_value_x_artifact_from_graveyard`
- `mica_reader_of_ruins_magecraft_sac_artifact_to_copy_when_decider_agrees`
- `mica_reader_of_ruins_magecraft_skips_copy_when_decider_declines`
- `killians_confidence_returns_to_hand_when_creature_deals_combat_damage`
- `killians_confidence_stays_in_graveyard_when_no_damage_or_no_pay`

## 2026-05-12 push XXIV: 14 new STX cards + CR 121.5 fix + cost-label rendering

Push XXIV (`claude/modern_decks` branch) — 14 new STX cards spanning
Prismari (the school with the previously-shortest card list), shared
Boros/Lorehold removal, and small mono-color staples. Plus a CR 121.5
("put-into-hand is not a draw") engine compliance fix uncovered while
auditing the unified-dispatcher trigger paths, plus a server/UI
`ability_cost_label` rendering improvement for `exile_other_filter`
(Lorehold Pledgemage / Postmortem Professor). Tests at 1140 (+17 net).

All new cards live in `catalog::sets::stx::extras`. Net per-school
status: Prismari **promoted from 3 to 9 cards** (more than tripled);
Witherbloom +1 (Eyetwitch Brood); Boros (R/W) shared +2 (Sacred Fire,
Rip Apart); Quandrix +1 (Karok Wrangler).

New STX cards:

- **Galvanic Iteration** (UR Prismari) 🟡 — {U}{R} Instant. "Copy
  target instant or sorcery spell you control" via `Effect::CopySpell`
  on a stack-spell target. The Magecraft-driven exile-then-replay-from-
  exile rider is omitted (no exile-self-on-resolution primitive).
- **Expressive Iteration** (UR Prismari) 🟡 — {U}{R} Sorcery.
  Collapsed to `Scry 2 → Draw 1` (the full exile-and-play-from-exile
  pattern wants a new primitive).
- **Magma Opus** (UR Prismari) 🟡 — {7}{U}{R} Sorcery. 4 damage to one
  creature + tap each opp creature + create a 3/3 Elemental (from
  shared `elemental_token()`) + draw 2. The "divided as you choose"
  damage half collapses to single-target; the discard alt-mode is
  omitted.
- **Reckless Amplimancer** (G Quandrix) 🟡 — {2}{G} 2/2 Elf Druid.
  Activated `{4}{G}{G}: +3/+3 EOT`. Printed `+X/+X where X is mana
  spent` collapses to fixed `+3/+3` (no per-activation mana-spent
  tracker).
- **Crashing Drawbridge** (C artifact) ✅ — {3} 0/4 Artifact Creature
  — Construct. "Creatures you control have haste" via
  `StaticEffect::GrantKeyword`.
- **Eyetwitch Brood** (BG Witherbloom) ✅ — {1}{B}{G} 1/1 Pest with
  Lifelink. "Whenever another Pest you control dies, put a +1/+1
  counter on this creature." Named to disambiguate from SOS's
  "Pest Mascot" (which fires on LifeGained).
- **First Day of Class** (W Silverquill) 🟡 — {W} Sorcery. ForEach
  creature you control gets +1/+1 EOT. The combat-damage-to-Pest
  rider is omitted.
- **Verdant Mastery** (G base) 🟡 — {3}{G}{G} Sorcery. You search
  for a basic + each opponent searches for a basic tapped via the
  `PlayerRef::Triggerer` reference inside a `ForEach(EachOpponent)`
  wrapper.
- **Sacred Fire** (RW Lorehold) ✅ — {R}{W} Sorcery. 3 damage + 3 life
  + Flashback {5}{R}{W} via `Keyword::Flashback(ManaCost)`.
- **Rip Apart** (RW Lorehold) ✅ — {R}{W} Sorcery. Two-mode
  `ChooseMode`: 3 dmg to creature/PW or destroy artifact/enchantment.
- **Codespell Cleric** (W Silverquill) ✅ — {W} 1/1 Kor Cleric with
  Lifelink. Vanilla body for low-curve Silverquill.
- **Sparkmage Apprentice** (R Prismari) ✅ — {1}{R} 1/2 Human Wizard.
  ETB: deals 2 damage to any target.
- **Karok Wrangler** (GU Quandrix) ✅ — {1}{G}{U} 2/2 Elf Druid.
  Magecraft: +1/+1 counter on target creature you control.
- **Soothsayer Adept** (U Prismari) ✅ — {1}{U} 2/2 Merfolk Wizard.
  Activated `{2}{U}: Surveil 1`.

**CR 121.5 engine compliance fix.** Goblin Guide's "puts it into
their hand" path was incorrectly emitting `GameEvent::CardDrawn` and
incrementing `cards_drawn_this_turn` via the `RevealTopAndDrawIf`
effect resolver. Per CR 121.5 ("If an effect moves cards from a
player's library to that player's hand without using the word
'draw,' the player has not drawn those cards"), this is not a draw;
no draw-payoff trigger should fire. Removed the spurious
`CardDrawn` push and dropped the `cards_drawn_this_turn` increment.
New test
`goblin_guide_put_into_hand_is_not_a_draw_per_cr_121_5` pins the
invariant.

**Server/UI: ability_cost_label renders `exile_other_filter`.** The
previous renderer only described `tap_cost`, `mana_cost`,
`sac_cost`, `life_cost`, and `exile_self_cost`; abilities with
`exile_other_filter: Some(_)` (Lorehold Pledgemage's
`{2}{R}{W}, Exile a card from your graveyard: +1/+1 EOT`) lost the
exile-cost rider in the UI tooltip. Now renders as
`{2}{R}{W}, Exile a card from gy` (bare-bones; the engine still
auto-picks the lowest-CMC matching card).

## 2026-05-12 push XXIII: 11 new STX cards + engine fixes for cross-zone selectors

Adds 11 new STX printings (mostly mono / multi-college fills) to the
catalog plus two cross-zone engine fixes that unblock several future
cards. All ship in `catalog::sets::stx::extras`. Net +14 functionality
tests (one Letter of Acceptance + Star Pupil split into two), tests at
1123 (up from 1098).

New STX cards (all ✅ unless flagged 🟡):

- **Star Pupil** (B) ✅ — {B} Creature — Cat Spirit, 0/1. ETB enters
  with a +1/+1 counter (Pterafractyl-shape ETB trigger). Dies → put
  a +1/+1 counter on target creature. Audited against CR 122.8 ("if
  a triggered ability instructs a player to put one object's
  counters on another object and that ability's trigger condition
  or effect checks that the object with those counters left the
  battlefield, the player doesn't move counters"): the printed card
  hard-codes one counter to dodge 122.8, so we ship Value::Const(1).

- **Ageless Guardian** (W) ✅ — {2}{W} 1/4 Spirit Cleric. Pure
  magecraft self-pump via `magecraft_self_pump(1, 0)`. Same shape
  as Symmetry Sage's first half without the flying-grant rider.

- **Returned Pastcaller** (W) ✅ — {4}{W} 3/3 Spirit Cleric, Flying.
  ETB return target IS card from your graveyard to your hand. Same
  shape as Pillardrop Rescuer (Lorehold) at one more mana, no R.

- **Letter of Acceptance** (Colorless) ✅ — {1} Artifact. ETB +1
  life; `{T}: Add {C}`; `{2}, {T}, Sac: Draw a card`. Three-mode
  artifact mana-rock that scales into the late game.

- **Charge Through** (G) ✅ — {G} Sorcery. Target creature you
  control gets +1/+1 and gains trample EOT. Standard combat-trick
  pump + keyword grant.

- **Devious Cover-Up** (U) 🟡 — {2}{U}{U} Instant. Counter target
  spell + exile up to one target card from any graveyard. The
  printed "any number of target cards from graveyards" rider
  collapses to "one card" via `Selector::Take(EachMatching(gy,
  Any), 1)` — the cross-player gy iteration is now correct (engine
  fix below) but multi-target prompting still needs the action
  shape to expose a target list. The single-strip approximation
  captures the most common play pattern.

- **Manifestation Sage** (GU) ✅ — {2}{G}{U} 2/2 Fractal Wizard,
  Flying. ETB mints a 0/0 G/U Fractal token + N +1/+1 counters
  where N = `HandSizeOf(You)`. Test seeds 3 hand cards and asserts
  the Fractal enters as a 3/3.

- **Crackle with Power** (R) 🟡 — {X}{R}{R}{R}{R}{R} Sorcery.
  `Value::Times(5, XFromCost)` damage to a single target. The
  printed "divided as you choose among any number of targets"
  rider collapses to single-target absorbing the full 5X damage
  (multi-target prompt gap).

- **Mentor's Guidance** (GU) ✅ — {1}{G}{U} Instant. Two-mode
  `ChooseMode`: deal `CountOf(YourCreatures)` damage to target opp
  creature, OR draw `CountOf(YourCreaturesWithCounter(+1/+1))`
  cards.

- **Dragonsguard Elite** (G) ✅ — {1}{G}{G} 2/2 Human Warrior.
  Magecraft adds a +1/+1 counter on self; `{3}{G}: +X/+X EOT`
  where X = `Value::PowerOf(This)`. Self-scaling combat trick.

- **Quintorius, Field Historian** (RW Legendary) 🟡 — {2}{R}{W}
  3/3 Legendary Elephant Cleric Spirit, Vigilance. ETB exile
  target gy card + create a 3/2 R/W Spirit token. The printed
  "Spirit creatures you control get +1/+0" static anthem is
  omitted (tribal-lord-with-creature-type selector shape gap).

Engine fixes (push XXIII):

- ✅ **CR 122.1a / 122.2 — `Value::CountersOn` cross-zone search.**
  The counter-count primitive used to scope to the battlefield
  only, so any selector resolving to a card in graveyard/exile
  zero'd silently. The lookup now falls through to graveyards and
  exile (matching `evaluate_requirement_static`'s zone walk for
  `WithCounter` filtering). Unblocks future cards that want to
  read counters from a non-battlefield card. **Note:** This is in
  technical tension with CR 122.2 ("counters on an object are not
  retained if that object moves from one zone to another") — the
  engine retains counters across zones (a benign deviation that
  most Magic cards work *with*, e.g. Felisa's death-counter
  filter). A future engine pass could clear counters on zone
  change to be strictly CR 122.2-compliant.

- ✅ **Multi-player zone iteration in `entities_in_zone`.** The
  `EachMatching { zone: Graveyard(EachPlayer), filter }` shape
  used to silently resolve to a single player's graveyard (via
  `resolve_player`'s singular fallback). Now uses `resolve_players`
  to iterate every matching player's zone. Unblocks Devious
  Cover-Up's cross-graveyard exile, Necrogenesis-style total
  count payoffs, and any future "from any graveyard" effect.

CR audit (push XXIII): Consulted **CR 122.8 — Counter movement
when source has left the battlefield**: "If a triggered ability
instructs a player to put one object's counters on another object
and that ability's trigger condition or effect checks that the
object with those counters left the battlefield, the player
doesn't move counters from one object to the other." Star Pupil's
printed Oracle dodges this by using a fixed "+1/+1 counter"
language; we ship `Value::Const(1)` accordingly. The cross-zone
`Value::CountersOn` lookup added in this push DOES NOT change Star
Pupil's behaviour (which still uses Const(1)), but it does mean
future die-trigger-readers-of-counters could implement the rare
errata-compatible "its counters" pattern in a CR 122-compliant way
once a "snapshot counters at trigger time" mechanism is added.

## 2026-05-12 push XXI: 6 new STX cards (multicolor + Quandrix utility)

Adds 6 new STX printings on top of push XX's Witherbloom/Quandrix
batch. All ship in `catalog::sets::stx::extras`. 6 new functionality
tests; total tests at 1098 (+6).

New STX cards:

- **Confront the Past** (R) 🟡 — {3}{R} Sorcery. 3-mode `ChooseMode`:
  reanimate target PW from gy / bounce target PW / 3-dmg to target
  PW. Mode 2 collapses the "X = loyalty counters" damage to a flat 3
  (no per-card loyalty introspection yet).
- **Specter of the Fens** (B) ✅ — {4}{B} 3/4 Flying Specter. ETB
  return target creature or planeswalker card from your graveyard to
  your hand. Standard reanimation-to-hand pattern.
- **Mascot Interception** (RW) ✅ — {4}{R}{W} Instant. Threaten +
  Untap + Haste EOT against any permanent. Similar shape to push
  XX's Tempted by the Oriq but instant-speed and any-permanent.
- **Twinscroll Shaman** (UR) ✅ — {2}{U}{R} 3/3 Human Wizard.
  Magecraft via `Effect::CopySpell { what: TriggerSource, count: 1 }`
  — the first cube card pointed at the push XVII / XVIII copy-spell
  wiring through Magecraft's trigger_source binding.
- **Practical Research** (GU) ✅ — {1}{G}{U} Sorcery. Doubles +1/+1
  counters on the chosen creature via `AddCounter(amount =
  CountersOn(target, +1/+1))`. Same shape as Growth Curve's
  doubling tail without the initial bump.
- **Hall of Oracles** (Land) ✅ — `{T}: Add {C}` + `{2}, {T}: +1/+1
  counter on target Wizard or Fractal creature you control`. Tribal
  filter via `HasCreatureType(Wizard) ∪ HasCreatureType(Fractal)`.

Rule alignment audited against CR 707 (Copying Objects), specifically
707.10 (copying spells): the new `Effect::CopySpell` wiring (push
XVII/XVIII) clones decisions (mode/targets/X/converged) for each
copy, matches the "you may choose new targets" default (we auto-keep,
matching MTG's "you may keep them"). 707.10d ("copy for each target")
and 707.10f ("permanent-spell copies become tokens on resolution")
remain as future engine work (tracked in TODO.md).

## 2026-05-12 push XX: 10 new STX cards (Witherbloom + Quandrix + utility)

Card additions (10 new STX cards in `catalog::sets::stx::extras`):

- **Curate** (U new) 🟡 — {1}{U} Instant: "Look at the top four cards of
  your library. Put one of them into your hand and the rest on the
  bottom of your library in a random order." Approximated as `Scry 3 →
  Draw 1` — the engine's Scry per-card top/bottom picker captures the
  pick-one + discard-rest shape, with the minor deviation that
  scry-bottomed cards land at the bottom in scry order rather than
  random order.

- **Solve the Equation** (U new) ✅ — {2}{U} Sorcery: faithful instant/
  sorcery tutor via `Effect::Search` against `HasCardType(Instant) ∨
  HasCardType(Sorcery)` → `Hand(You)`. The printed "mana value 4 or
  less" cap is omitted for simplicity — search picks any IS card, same
  shape as Mystical Tutor cousins. Tested via scripted decider picking
  Lightning Bolt out of a 3-card library.

- **Resculpt** (U new) ✅ — {1}{U} Instant: Exile target creature or
  artifact; its *original controller* (via `PlayerRef::ControllerOf(
  Target(0))`) creates a 4/4 blue Elemental creature token. Clean
  unconditional removal-with-trade.

- **Mortality Spear** (B/G new) ✅ — {3}{B}{G} Instant: catch-all
  destroy against Creature ∨ Planeswalker. Battle subtype is omitted
  (no MoM cards in this catalog).

- **Daemogoth Titan** (B new) 🟡 — {B}{B} 11/11 Demon Horror with
  Attack-trigger sacrifice ("sac another creature"). The block-half
  trigger is omitted (no `Blocks` event in the engine yet). The
  Sacrifice's auto-decider sorts by (tokens-first, lowest CMC, lowest
  power), so a fresh Titan will sac fodder before itself when both
  exist — matching the "another creature" intent.

- **Daemogoth Woe-Eater** (B/G new) 🟡 — {2}{B}{G} 4/4 Demon Horror.
  ETB sacrifice + Attack-trigger sac-into-counter wired as a `Seq`
  body. The "may" optionality on the attack trigger collapses to
  always-sac (engine's Sacrifice no-ops cleanly when no legal target
  exists, so the AddCounter then no-ops by symmetry).

- **Honor Troll** (B/G new) 🟡 — {1}{B}{G} 1/4 Troll Warrior with
  Trample. The conditional +2/+0 + lifelink rider (gated on
  "you've gained life this turn") is ⏳ pending a per-turn life-gained
  tracker; body ships full.

- **Quandrix Cultivator** (G/U new) ✅ — {3}{G}{U} 3/3 Elf Druid. ETB
  ramp via `Effect::Search` against `IsBasicLand & (HasLandType(Forest)
  ∨ HasLandType(Island))` → battlefield tapped. Tested with scripted
  decider picking a basic Forest.

- **Hofri Ghostforge** (R/W Legendary new) 🟡 — {2}{R}{W} 3/4 Legendary
  Spirit Cleric. Body + supertypes/creature-types ship full. The
  printed +1/+0 anthem and the exile-on-death/return-as-spirit
  graveyard cycle are ⏳ pending conditional anthems + a delayed-
  trigger replacement primitive.

- **Tempted by the Oriq** (B new) ✅ — {2}{B} Sorcery: temp-steal +
  untap + grant haste EOT, faithful to the printed Threaten/Act of
  Treason template. The printed Magecraft rider ("Whenever you cast or
  copy an IS spell, that creature deals 1 damage to any target") is
  omitted — needs a delayed trigger tied to the controlled creature.

Engine notes:

- The `Strategic Planning` factory already lives in
  `catalog::sets::decks::modern` (Mill 3 + Draw 1 approximation); the
  STX push deliberately does *not* add a duplicate factory — the glob
  re-export in `catalog::mod` would conflict on ambiguous names.

10 new tests in `tests::stx::*` covering: Curate draw, Solve the
Equation tutor, Resculpt exile + token-to-original-controller,
Mortality Spear destroy, Daemogoth Titan stats, Daemogoth Woe-Eater
ETB sacrifice, Honor Troll trample stats, Quandrix Cultivator basic-
land ETB ramp, Hofri Ghostforge stats, Tempted by the Oriq steal +
haste. All 1091 lib tests pass (was 1079; +12 net).

## 2026-05-12 push XIX: Monocolored predicate + Tanazir ETB + 10 new STX cards + CR 119.4

Engine primitives:

- **`SelectionRequirement::Monocolored`** — `distinct_colors() == 1`
  filter, sibling to `Multicolored` (≥2) and `Colorless` (0). Wired
  through both `evaluate_requirement_static` (battlefield/stack) and
  `evaluate_requirement_on_card` (library/hand/graveyard). Promotes
  Strixhaven's Vanishing Verse to ✅ — its target filter is now the
  printed "nonland, monocolored permanent" shape.

- **CR 119.4 — Life-payment pre-flight on alternative casts** — the
  alt-cost cast path (`cast_spell_alternative`) was missing the
  pre-flight life-cost gate that the activated-ability path already
  honored. Per CR 119.4 ("a player may pay X life only if their life
  total is greater than or equal to X"), the engine now rejects the
  alt cast cleanly with `GameError::InsufficientLife` instead of
  driving life negative mid-cast. Existing activated-ability gate at
  `actions.rs::activate_ability` is unchanged; the alt-cost path
  mirrors it now.

Card promotions / new cards:

- **Vanishing Verse** (STX W/B) 🟡 → ✅ — target filter is now exactly
  the printed "nonland, monocolored permanent" via the new
  `SelectionRequirement::Monocolored` predicate. Multicolored and
  colorless permanents are correctly rejected by the cast-time target
  validator.

- **Tanazir Quandrix** (STX G/U Legendary) 🟡 → 🟡 (ETB **now wired**) —
  ETB "double the number of +1/+1 counters on each creature you
  control" is wired via `ForEach(Creature & ControlledByYou)` binding
  `Selector::TriggerSource` to each iteration entity, plus
  `AddCounter(+1/+1, amount: CountersOn(TriggerSource, +1/+1))`. A
  creature with N +1/+1 counters before resolution ends with 2N after
  resolution — matches the printed double behaviour. No new
  primitive needed — `ForEach` already binds `TriggerSource` to the
  iteration entity, and `CountersOn` already reads the count off a
  selector.

- **Bookwurm** (STX G new) ✅ — {5}{G}{G} 5/5 Wurm with trample. ETB
  Seq(GainLife 4 + Draw 1). Simple late-game value finisher.

- **Field Trip** (STX G new) ✅ — {2}{G} Sorcery: search basic Forest
  → battlefield, then Learn (→ Draw 1 approximation).

- **Reduce to Memory** (STX U new) ✅ — {2}{U} Sorcery: Exile nonland
  permanent, then its **controller** (via `PlayerRef::ControllerOf(
  Target(0))`) creates a 2/2 colorless Inkling artifact creature
  token.

- **Baleful Mastery** (STX B new) 🟡 — {2}{B} Instant: Exile target
  creature/planeswalker; an opponent draws a card (collapses to "each
  opp draws 1" via `Selector::Player(EachOpponent)`). Alt cost {1}{B}
  not wired (alt-cost-implies-mode primitive gap).

- **Igneous Inspiration** (STX R new) ✅ — {2}{R} Sorcery: 3 damage to
  target creature/PW, then Learn (→ Draw 1).

- **Combat Professor** (STX W new) 🟡 — {3}{W} 2/4 Cat Cleric, Flying +
  Vigilance. Mentor approximated as Attacks trigger that adds +1/+1
  counter on a target attacking creature with power ≤ 1 (the "lesser
  power than this" base-2 collapse).

- **Conspiracy Theorist** (STX R new) 🟡 — {1}{R} 2/1 Human Shaman.
  Body only (Mentor's loot-into-exile-with-timer is the same gap as
  Suspend Aggression).

- **Beaming Defiance** (STX W new) ✅ — {1}{W} Instant: target friendly
  creature gets +2/+0 and Hexproof EOT.

- **Excavated Wall** (STX C new) ✅ — {2} Wall Artifact Creature 0/4
  with Defender. ETB gains 2 life.

- **Snow Day** (STX U/R new) 🟡 — {U}{R} Instant: tap one creature +
  stun counter (printed "up to two targets" collapsed — multi-target
  prompt is the same gap as Vibrant Outburst).

- **Spell Satchel** (STX C new) 🟡 — {3} Artifact: `{T}: Add {C}` mana
  ability + `{3},{T},Sac: return a low-CMC instant/sorcery from your
  graveyard to your hand`. Multi-target return ("any number, total
  CMC ≤ 4") collapsed to single-target ≤ 4.

15 new tests in `tests::stx::*` (Vanishing Verse Monocolored
acceptance + rejection, Tanazir ETB counter doubling + zero-counter
no-op, plus per-card body tests for all 10 new cards). All 1079 lib
tests pass (was 1064; +15 net).

## 2026-05-12 push XVIII: exile_other_filter + CR 605 + Lorehold STX closer

Engine primitives:

- **`ActivatedAbility.exile_other_filter: Option<SelectionRequirement>`** —
  new cost variant: exile a *different* card from the activator's
  graveyard matching the filter. Pre-flight gate (rejects cleanly with
  `GameError::SelectionRequirementViolated` when no match), then exile
  after tap/mana/life payment succeeds. Picks the lowest-CMC matching
  card so the activator keeps higher-value cards in graveyard. Emits
  `CardLeftGraveyard` + bumps the per-turn exile + gy-leave tallies
  through the existing instrumentation. Closes the long-standing
  "exile-a-different-card-from-gy as cost" gap in TODO.md (Postmortem
  Professor's `{1}{B}, Exile an IS card from gy: return self to bf`,
  Lorehold Pledgemage's `{2}{R}{W}, Exile a card from gy: +1/+1 EOT`).

- **Trigger dispatcher bugfix** — `dispatch_triggers_for_events`
  prepared an `event_subject` (the attacker / cast spell / ETBing
  permanent / etc.) but **never threaded it** into the resulting
  `StackItem::Trigger.trigger_source` field (hardcoded to `None`).
  Effects that referenced `Selector::TriggerSource` from unified-
  dispatcher triggers silently zero'd. Now correctly wired —
  unblocking Sparring Regimen's per-attacker counter bump and every
  future trigger that reads its source via `TriggerSource`.

- **`EventScope::AnotherOfYours` controller check** — the scope was
  previously only checking that the subject card isn't the trigger
  source itself. Now also enforces that the subject's controller
  (or graveyard-owner for `CreatureDied` subjects) matches the
  trigger source's controller. Without this, Sparring Regimen would
  pump opp attackers, Felisa would mint Inklings on opp creature
  deaths, Cauldron of Essence's drain trigger would fire on opp deaths
  (already broken; now correct). 1062 lib tests still pass post-fix.

- **`effect::shortcut::prowess()`** — one-liner trigger constructor
  for the printed Prowess reminder text ("Whenever you cast a
  noncreature spell, this creature gets +1/+1 EOT"). Filter uses
  `Predicate::EntityMatches { what: TriggerSource, filter:
  HasCardType(Creature).negate() }`. Promoted Spectacle Mage's
  Prowess keyword tag to a functional self-pump trigger.

Card promotions / new cards:

- **Postmortem Professor** (SOS B) 🟡 → ✅ — graveyard-recursion
  activation `{1}{B}, Exile an IS card from gy: return self` now
  wired via `from_graveyard: true` + `exile_other_filter:
  Some(HasCardType(Instant) ∨ HasCardType(Sorcery))`. On-attack drain
  + CantBlock keyword unchanged.

- **Lorehold Apprentice** (STX R/W) 🟡 → ✅ — Magecraft now also fires
  the "1 damage to any target" half via a `Seq([GainLife, DealDamage])`
  trigger body, auto-targeting an opponent or creature for the ping.

- **Lorehold Pledgemage** (STX R/W) 🟡 → ✅ — `{2}{R}{W}, Exile a
  card from gy: +1/+1 EOT` activated ability wired via the new
  `exile_other_filter: Some(Any)` cost. Reach + body unchanged.

- **Beledros Witherbloom** (STX W/B Legendary) 🟡 → 🟡 (activation
  fully wired) — `Pay 10 life: Untap each land you control. Activate
  only as a sorcery.` uses the existing `life_cost: 10` +
  `sorcery_speed: true` + `Effect::Untap` over
  `EachPermanent(Land & ControlledByYou)`. Pre-flight gate rejects
  cleanly with `GameError::InsufficientLife` on insufficient life.

- **Tanazir Quandrix** (STX G/U Legendary) 🟡 → 🟡 (attack-trigger
  wired) — "Whenever Tanazir attacks, double target creature you
  control's toughness EOT" now uses `Effect::PumpPT` with
  `toughness: Value::ToughnessOf(Target(0))` — the +Toughness pump
  by the target's own toughness effectively doubles it (T + T = 2T)
  until end of turn. ETB counter-doubling remains 🟡 (counter-multiplier
  primitive missing).

- **Spectacle Mage** (STX U/R) 🟡 → 🟡 (Prowess functional) —
  declared `Keyword::Prowess` is now functionally wired through the
  new `prowess()` shortcut, with the printed +1/+1 EOT pump firing
  on noncreature spell casts.

- **Sparring Regimen** (STX R/W) 🟡 → ✅ — "Whenever you attack, put
  a +1/+1 counter on each attacking creature you control" wired as
  a per-attacker `Attacks/AnotherOfYours` trigger that puts a counter
  on `Selector::TriggerSource` (the attacker). Unblocked by the
  dispatcher trigger_source bugfix above.

- **Storm-Kiln Artist** (STX R/W) 🟡 → 🟡 (faithful damage target) —
  Magecraft damage half upgraded from `EachOpponent` to
  `target_filtered(Creature ∨ Player ∨ Planeswalker)` to match the
  printed "1 damage to any target" — auto-target picker selects the
  best legal target. Treasure half unchanged.

- **Killian, Ink Duelist** (STX W/B) 🟡 (subtype fix) — creature
  type fixed from `Warrior` → `Warlock` to match the printed Oracle.

- **Molten Note** (SOS R/W) ⏳ → ✅ (new factory) — `{X}{R}{W}` sorcery
  in `catalog::sets::sos::sorceries::molten_note`. `Effect::Seq` of
  `DealDamage(target Creature, XFromCost) + Untap(each Creature
  ControlledByYou)`. Flashback {6}{R}{W} wired via `Keyword::Flashback`.

- **Witherbloom Pledgemage** (STX B/G) refactor: `{T}, Pay 1 life:
  Add {B}/{G}` now uses `life_cost: 1` + pure `AddMana(AnyOneColor)`
  effect. By CR 605.1a (no target, could add mana, not loyalty), the
  ability now qualifies as a mana ability and resolves **without the
  stack** per CR 605.4a. Previous behaviour was `Seq([LoseLife,
  AddMana])` which went onto the stack — breaking the printed
  instant-speed mana ramp semantics.

MagicCompRules audit additions:

- **CR 605.1a — Mana abilities** ✅ wired. The engine's
  `is_mana_ability` recogniser in `game/actions.rs` matches an
  activated ability's effect against the rule's three criteria
  conservatively: pure `Effect::AddMana` (no target field, always can
  add mana) or `Effect::Seq` of mana abilities. Witherbloom
  Pledgemage's refactor (push XVIII) demonstrates the round-trip.
- **CR 605.4a — Mana abilities don't go on the stack** ✅ wired.
  `activate_ability` branches on `is_mana_ability(&ability.effect)`:
  the mana-ability path resolves immediately via
  `continue_ability_resolution`; the non-mana path pushes a
  `StackItem::Trigger`. New test
  `witherbloom_pledgemage_is_a_mana_ability_per_cr_605` asserts the
  stack length is unchanged after activation.

Tests:
- `postmortem_professor_returns_from_graveyard_by_exiling_instant_or_sorcery`
- `postmortem_professor_rejects_activation_without_eligible_gy_card`
- `molten_note_deals_x_damage_and_untaps_your_creatures`
- `molten_note_has_flashback_keyword`
- `lorehold_apprentice_magecraft_drains_one_to_opponent_and_gains_life`
- `lorehold_pledgemage_gy_exile_cost_pumps_self`
- `lorehold_pledgemage_rejects_activation_with_empty_graveyard`
- `beledros_witherbloom_pay_ten_life_untaps_all_lands`
- `beledros_witherbloom_rejects_activation_with_insufficient_life`
- `tanazir_quandrix_attack_trigger_doubles_target_toughness`
- `spectacle_mage_prowess_fires_on_noncreature_spell`
- `spectacle_mage_prowess_does_not_fire_on_creature_spell`
- `sparring_regimen_creates_spirit_etb_and_pumps_attacker`
- `witherbloom_pledgemage_is_a_mana_ability_per_cr_605`
- `witherbloom_pledgemage_rejects_activation_with_zero_life`

All 1064+ lib tests pass.

## 2026-05-12 push XVII: from_graveyard activations + Effect::CopySpell

Three engine primitives + 7 SOS card promotions (5 from 🟡→✅, 1 ⏳→🟡,
1 trigger-filter-evaluation cleanup). All 1048 lib tests pass (was
1036; +12 new tests).

Engine improvements:

- **`ActivatedAbility.from_graveyard: bool`** + **`exile_self_cost: bool`** —
  first-class support for activated abilities whose source is in the
  controller's graveyard rather than the battlefield. The
  `activate_ability` engine path now searches the graveyard when the
  ability is flagged, and `exile_self_cost` exiles the source as part
  of the cost (mirror to `sac_cost` for battlefield permanents). Both
  fields are `#[serde(default)]` for snapshot back-compat. Powers SOS
  Summoned Dromedary's `{1}{W}: return from gy to hand`, Teacher's
  Pest's `{B}{G}: return from gy to bf tapped`, Stone Docent's `{W},
  Exile from gy: gain 2 + surveil 1`, and Eternal Student's `{1}{B},
  Exile from gy: create 2 Inklings`. Closes TODO.md "Activated-Ability
  `From Your Graveyard` Path".

- **`Effect::CopySpell { what, count }`** — proper implementation
  (previously a stub). Locates the matching `StackItem::Spell` for the
  selected target (defaults: `Selector::TriggerSource` for cast-time
  triggers, `Selector::This` for the cast spell itself, or any
  selector that resolves to a Card/Permanent CardId), then pushes
  `count` copies on the stack above the original. Each copy gets a
  fresh CardId, inherits target/mode/x_value/converged_value, and is
  flagged `uncounterable: true` (copies can't be countered by spells).
  The original spell still resolves after the copies. Emits
  `GameEvent::SpellsCopied { original, count }` per resolution. Powers
  Aziza Mage Tower Captain's Magecraft-style copy + Lumaret's Favor's
  Infusion copy + Social Snub's "may copy" rider.

- **`StackItem::Trigger.trigger_source: Option<EntityRef>`** —
  preserves the trigger's source entity (the just-cast spell for
  Magecraft, the ETBing permanent for ETB) through stack-based
  resolution. Previously the trigger_source context was lost when the
  trigger went on the stack, breaking lookups like
  `Effect::CopySpell`'s "find the just-cast spell on the stack" path
  and `Selector::CastSpellTarget`. Now propagated end-to-end via the
  new `continue_trigger_resolution_with_source` helper. Made
  `EntityRef` `Serialize`/`Deserialize` so it survives snapshots; the
  field is `#[serde(default)]` for back-compat.

- **`push_on_cast_triggers` filter evaluation** — the SpellCast/
  SelfSource trigger path now evaluates the trigger's filter
  (Infusion's `LifeGainedThisTurnAtLeast` for Lumaret's Favor, Social
  Snub's "you control a creature" check, etc.) at trigger-creation
  time. Previously the filter was dropped silently — every SelfSource
  cast trigger fired unconditionally. The fix collects
  `(Option<Predicate>, Effect)` instead of bare effects, evaluating
  the predicate in the `EffectContext::for_cast`-shaped ctx where
  `trigger_source` points to the cast card.

- **Sorcery-speed activation gate** — `ActivatedAbility.sorcery_speed`
  was declared but never enforced; activations leaked through during
  opponent turns / non-main steps. Now checked at the top of
  `activate_ability`, returning `GameError::SorcerySpeedOnly` cleanly.
  Used by Cauldron of Essence's reanimation, Stone Docent's
  graveyard-exile, Summoned Dromedary's recursion, and any future
  activated ability flagged sorcery-speed.

Cards (8 promotions):

| Card | School | Status | Notes |
|---|---|---|---|
| Summoned Dromedary | White | ✅ (was 🟡) | {1}{W} sorcery-speed gy-recursion now wired via the new `from_graveyard` field. |
| Teacher's Pest | Witherbloom | ✅ (was 🟡) | {B}{G} gy-recursion to bf tapped wired via the new `from_graveyard` field. |
| Stone Docent | White | ✅ (was 🟡) | {W} sorcery-speed exile-from-gy: gain 2 + surveil 1. Uses `from_graveyard` + `exile_self_cost`. |
| Eternal Student | Black | ✅ (was 🟡) | {1}{B} exile-from-gy: create 2 Inklings. Uses `from_graveyard` + `exile_self_cost`. |
| Aziza, Mage Tower Captain | Lorehold | ✅ (was 🟡) | Magecraft "may tap 3 to copy" now wired via `Effect::MayDo` + `Effect::Tap(take 3)` + `Effect::CopySpell { what: TriggerSource }`. |
| Lumaret's Favor | Green | ✅ (was 🟡) | Infusion copy now wired via `Effect::CopySpell { what: Self }` gated on `LifeGainedThisTurnAtLeast(1)`. |
| Social Snub | Silverquill | 🟡 (was ⏳) | Cast-time copy rider + main "each player sacs + drain" wired. The "you control a creature" filter uses `Predicate::SelectorExists(EachPermanent(Creature & ControlledByYou))`. |
| Harsh Annotation | White | ✅ (was 🟡) | Inkling token now goes to the target creature's owner via `PlayerRef::OwnerOf(Target(0))` (previously created under the caster). |

Tests in `tests::sos::*`:

- `summoned_dromedary_returns_from_graveyard_to_hand`
- `summoned_dromedary_activation_rejected_during_opponent_priority`
- `teachers_pest_returns_from_graveyard_to_battlefield_tapped`
- `stone_docent_exiles_self_and_gains_life`
- `eternal_student_exiles_self_and_creates_two_inklings`
- `stone_docent_rejected_at_instant_speed`
- `aziza_copies_instant_via_magecraft_when_decider_agrees`
- `aziza_skips_copy_when_decider_declines`
- `lumarets_favor_infusion_copies_when_life_gained_this_turn`
- `lumarets_favor_infusion_does_not_copy_without_lifegain`
- `social_snub_copies_when_caster_controls_a_creature_and_decider_agrees`
- `social_snub_does_not_copy_without_a_creature`

## 2026-05-01 push XVI: CastSpellHasX + MayPay + HasXInCost + LibrarySizeOf

Six engine primitives + 10 SOS/STX card promotions. Five new
predicates/effects/values + a `CardsInZone(Hand)` filter-evaluation
fix that was silently breaking hand-source MayDo riders. Tests pass at
1025 (+13).

Engine improvements:

- **`Predicate::CastSpellHasX`** — cast-time introspection on the
  just-cast spell's `{X}` symbols. Used by Quandrix's "whenever you
  cast a spell with `{X}` in its mana cost" payoffs (Geometer's
  Arthropod, Matterbending Mage). Reads the cast spell's
  `card.definition.cost.has_x()` from `StackItem::Spell` via
  `ctx.trigger_source`.
- **`Effect::MayPay { description, mana_cost, body }`** — sibling to
  push XV's `Effect::MayDo`, but with a mana-cost payment. The
  controller is asked yes/no; on yes + sufficient pool, the engine
  deducts the mana and runs `body`. Decline / can't-afford skip the
  body silently. Powers Bayou Groff's printed "may pay {1} to return
  on death" rider, Killian's Confidence-style optional reanimation,
  and any future "may pay X to do Y" pattern with a pure-mana cost.
- **`SelectionRequirement::HasXInCost`** — card-level filter that
  matches if the card's printed mana cost contains an `{X}` pip.
  Wires Paradox Surveyor's "land OR card with {X} in cost" reveal
  filter to its exact-printed shape.
- **`Value::LibrarySizeOf(PlayerRef)`** — reads `players[p].library.
  len()` at evaluation time. Promotes Body of Research's "for each
  card in your library" Fractal scaling from the prior
  `GraveyardSizeOf` approximation to the printed exact predicate.
- **`shortcut::cast_has_x_trigger(effect)`** — Magecraft/Repartee-
  style helper for "whenever you cast a spell with {X}" payoffs.
  Bundles `EventKind::SpellCast / YourControl` with a `CastSpellHasX`
  filter.
- **`Selector::CardsInZone(Hand)` filter-evaluation fix** — was
  routing through `evaluate_requirement_static` (which only walks
  battlefield → graveyard → exile → stack), so any predicate against
  a hand-resident card silently returned false (e.g. Embrace the
  Paradox's `MayDo(Move(one_of(CardsInZone(Hand, Land)) → bf
  tapped))` couldn't see the lands in hand). Now routes through
  `evaluate_requirement_on_card`, the card-level evaluator that
  works for non-battlefield zones. Also benefits any future
  hand/library/exile-source `Move` chain.

Cards (10 promotions):

| Card | Status before | Status after |
|---|---|---|
| Geometer's Arthropod | ⏳ | ✅ (CastSpellHasX trigger fully wired) |
| Matterbending Mage | 🟡 | ✅ (X-cast trigger grants Unblockable EOT) |
| Paradox Surveyor | 🟡 | ✅ (Land OR HasXInCost reveal filter) |
| Embrace the Paradox | 🟡 | ✅ (MayDo land-from-hand → bf tapped) |
| Sundering Archaic | 🟡 | 🟡 ({2}: graveyard → bottom-of-library wired) |
| Aziza, Mage Tower Captain | ⏳ | 🟡 (body-only; copy-spell still gating ult) |
| Zaffai and the Tempests | ⏳ | 🟡 (body-only; once-per-turn free cast omitted) |
| Bayou Groff (STX) | 🟡 | ✅ (MayPay {1} → return-to-hand on death) |
| Felisa, Fang of Silverquill (STX) | 🟡 | ✅ (counter-bearing-creature dies → Inkling) |
| Body of Research (STX) | 🟡 | ✅ (LibrarySizeOf promotes from gy proxy) |

13 new tests in `tests::sos::*` and `tests::stx::*`. All 1025 lib
tests pass.

## 2026-05-01 push XV: MayDo primitive + Witherbloom MDFC closer + life-cost activations

Closes out the **Witherbloom (B/G) school** (Lluwen MDFC was the last ⏳)
and lands two long-tracked engine primitives — `Effect::MayDo` and
`ActivatedAbility.life_cost`. New cards: **Lluwen, Exchange Student //
Pest Friend** (Witherbloom MDFC), **Great Hall of the Biblioplex**
(legendary colorless utility land), **Follow the Lumarets** (G Infusion
sorcery). Promotions of "you may"-bearing 🟡 cards: Stadium Tidalmage,
Pursue the Past, Witherbloom Charm mode 0, Heated Argument, Rubble
Rouser; Erode now wires the basic-land tutor for the target's
controller.

Engine improvements:

- **`Effect::MayDo { description, body }`** — first-class "you may [body]"
  primitive. Emits a yes/no via `Decision::OptionalTrigger`; `AutoDecider`
  defaults to `Bool(false)` (skip), matching MTG's "you may defaults to no"
  rule. The `description` is a `String` (rather than `&'static str`)
  because `Effect` is bound to `Deserialize` via `GameState`'s serde
  derive. Walkers (`requires_target`, `primary_target_filter`,
  `target_filter_for_slot_in_mode`) recurse into the inner body.

- **`ActivatedAbility.life_cost: u32`** — additional life-payment cost on
  activations. Pre-flight gate rejects activation cleanly when controller
  has insufficient life (new `GameError::InsufficientLife`); life is
  paid up front after tap/mana succeed. Powers Great Hall of the
  Biblioplex's `{T}, Pay 1 life: Add one mana of any color` printed
  ability faithfully — the effect itself is a pure `AddMana`, so the
  ability still qualifies as a true mana ability and resolves
  immediately without going on the stack. New
  `ActivatedAbilityView.cost_label` rendering shows "Pay N life" tokens.

- **Erode's basic-land tutor for target's controller** — uses
  `PlayerRef::ControllerOf(Box::new(Selector::Target(0)))` for both the
  `Search.who` field and the `ZoneDest::Battlefield.controller` field, so
  the tutor and battlefield-place both target the correct (gy-resident)
  player. Unblocks the printed "its controller may search their library
  for a basic land" half.

Cards (3 new + 6 promotions = 9 SOS rows touched):

| Card | Status before | Status after |
|---|---|---|
| Lluwen, Exchange Student // Pest Friend | ⏳ | 🟡 |
| Great Hall of the Biblioplex | ⏳ | 🟡 |
| Follow the Lumarets | ⏳ | 🟡 |
| Stadium Tidalmage | 🟡 | 🟡 (may-loot now opt-in) |
| Pursue the Past | 🟡 | 🟡 (may-discard now opt-in) |
| Witherbloom Charm mode 0 | 🟡 | 🟡 (may-sac now opt-in) |
| Heated Argument | 🟡 | 🟡 (may-exile rider now opt-in) |
| Rubble Rouser | 🟡 | 🟡 (may-rummage now opt-in) |
| Erode | 🟡 | ✅ (basic-land tutor now wired) |

All 1012 lib tests pass.

## 2026-05-01 pushes XI/XII: 29 MDFCs + per-spell-type tallies + CastFace audit log

Two batches of MDFCs (modal-double-faced cards) under a new
`catalog::sets::sos::mdfcs` module, plus three engine improvements:

- **`enum CastFace { Front, Back, Flashback }`** — threaded through
  `GameEvent::SpellCast.face` + `GameEventWire::SpellCast.face`. Replays
  / spectator UIs can distinguish a back-face MDFC cast from a normal
  hand cast and from a graveyard-replay flashback cast. New transient
  `GameState.pending_cast_face` field; `cast_spell_back_face` sets it to
  `Back`, `cast_flashback` emits `Flashback` directly, default cast
  paths emit `Front`.
- **`Player.instants_or_sorceries_cast_this_turn`** +
  **`Player.creatures_cast_this_turn`** — refines
  `spells_cast_this_turn` (which counts every spell type) so cards that
  explicitly gate on the IS or creature subset can target the
  exact-printed predicate. Backed by two new predicates:
  `Predicate::InstantsOrSorceriesCastThisTurnAtLeast` and
  `Predicate::CreaturesCastThisTurnAtLeast`. Surfaced through
  `PlayerView` for client UIs (with `#[serde(default)]` for snapshot
  back-compat).
- **Potioner's Trove** — gate promoted from the proxy
  `SpellsCastThisTurnAtLeast(You, 1)` → exact
  `InstantsOrSorceriesCastThisTurnAtLeast(You, 1)` matching the printed
  Oracle text.

Push XI (17 MDFCs, all 🟡):

| Card | Color | Front | Back |
|---|---|---|---|
| Elite Interceptor // Rejoinder | W | 1/2 Human Wizard | Counter target creature spell |
| Emeritus of Truce // Swords to Plowshares | W | 3/3 Cat Cleric | Exile + lifegain via `PlayerRef::ControllerOf` |
| Honorbound Page // Forum's Favor | W | 3/3 Cat Cleric | +1/+1 EOT + 1 life |
| Joined Researchers // Secret Rendezvous | W | 2/2 Human Cleric Wizard | Draw 3 (collapsed to caster) |
| Quill-Blade Laureate // Twofold Intent | W | 1/1 Human Cleric | +1/+1 EOT + Inkling token |
| Spiritcall Enthusiast // Scrollboost | W | 3/3 Cat Cleric | Fan-out +1/+1 counter on each creature you control |
| Encouraging Aviator // Jump | U | 2/3 Bird Wizard w/ Flying | Grant Flying EOT |
| Harmonized Trio // Brainstorm | U | 1/1 Merfolk Bard Wizard | Draw 3 + put 2 on top |
| Cheerful Osteomancer // Raise Dead | B | 4/2 Orc Warlock | Return creature card from gy → hand |
| Emeritus of Woe // Demonic Tutor | B | 5/4 Vampire Warlock | Search library for any card → hand |
| Scheming Silvertongue // Sign in Blood | B | 1/3 Vampire Warlock | Target player draws 2 + loses 2 |
| Adventurous Eater // Have a Bite | B | 3/2 Human Warlock | -3/-3 EOT |
| Emeritus of Conflict // Lightning Bolt | R | 2/2 Human Wizard | 3 dmg to any target |
| Goblin Glasswright // Craft with Pride | R | 2/2 Goblin Sorcerer | +2/+0 + Haste EOT |
| Emeritus of Abundance // Regrowth | G | 3/4 Elf Druid | Return any card from gy → hand |
| Vastlands Scavenger // Bind to Life | G | 4/4 Trample Bear Druid | Return up to 2 creatures from gy |
| Leech Collector // Bloodletting | B | 2/2 Human Warlock | Drain 2 |
| Pigment Wrangler // Striking Palette | R | 4/4 Orc Sorcerer | 2 dmg to any target |

Push XII (12 more MDFCs, all 🟡):

| Card | Color | Front | Back |
|---|---|---|---|
| Spellbook Seeker // Careful Study | U | 3/3 Bird Wizard w/ Flying | Draw 2 + discard 2 |
| Skycoach Conductor // All Aboard | U | 2/3 Bird Pilot w/ Flying | Bounce target creature to owner's hand |
| Landscape Painter // Vibrant Idea | U | 2/1 Merfolk Wizard | Draw 3 |
| Blazing Firesinger // Seething Song | R | 2/3 Dwarf Bard | Add `{R}{R}{R}{R}{R}` |
| Maelstrom Artisan // Rocket Volley | R | 3/2 Minotaur Sorcerer | 2 dmg each opp + 2 dmg to opp creature |
| Scathing Shadelock // Venomous Words | B | 4/6 Snake Warlock w/ Deathtouch | -2/-2 EOT |
| Infirmary Healer // Stream of Life | G | 2/3 Cat Cleric w/ Lifelink | Gain X life |
| Jadzi, Steward of Fate // Oracle's Gift | U | Legendary 2/4 Human Wizard w/ Flying | Draw 2X cards |
| Sanar, Unfinished Genius // Wild Idea | U/R | Legendary 0/4 Goblin Sorcerer | Draw 3 |
| Tam, Observant Sequencer // Deep Sight | G/U | Legendary 4/3 Snake Wizard w/ Deathtouch | Scry 4 + Draw 1 |
| Kirol, History Buff // Pack a Punch | R/W | Legendary 2/3 Vampire Cleric w/ Lifelink | 3 dmg to creature |
| Abigale, Poet Laureate // Heroic Stanza | W/B | Legendary 2/3 Bird Bard w/ Flying | +2/+2 EOT + Lifelink |

Cube color pool wiring:
- White: + 6 white MDFCs.
- Blue: + 6 blue MDFCs (incl. Jadzi).
- Black: + 6 black MDFCs.
- Red: + 5 red MDFCs.
- Green: + 3 green MDFCs.
- Cross-pools: Sanar (UR), Tam (GU), Kirol (RW), Abigale (WB).

Test additions: 42 new tests in `tests::sos::*`. All 997 lib tests
pass.

## 2026-05-01 push X: Flashback wirings, Selector::Take, MDFC back-face cast

5 new SOS card factories (4 🟡 + 1 ✅), 4 promotions from 🟡 to ✅, plus
three engine primitives:

- **`Selector::Take { inner, count }`** — wraps another selector to
  clamp how many entities flow through (in resolution order). Primary
  payoff is "select up to N from a graveyard / library / hand": it
  promotes Practiced Scrollsmith's gy-exile from "every matching
  noncreature/nonland card" to "exactly one"; lifts Pull from the
  Grave's gy-recursion from one card to two; available for future SOS
  Heated Argument-style "may exile a card" wraps. Sugar:
  `Selector::one_of(inner)` and `Selector::take(inner, n)`.

- **`GameAction::CastSpellBack`** + **`cast_spell_back_face`** —
  generalises `PlayLandBack` to non-land MDFC back faces. Mirrors the
  PlayLandBack flow: swaps the in-hand card's `definition` to the
  back face's, then routes through the regular `cast_spell` path so
  cost / type / target filters / effect all resolve against the back
  face. Unblocks the SOS MDFC cycles whose backs are
  creatures/instants/sorceries (Studious First-Year // Rampant
  Growth wired as the first one; the rest of the cycle is wireable
  one-by-one as oracle text becomes available). The 3D client picks
  this up automatically: the right-click flip on hand cards now
  routes flipped non-land MDFCs through `CastSpellBack` (in addition
  to the existing `PlayLandBack` for land MDFCs). New
  `TargetingState.back_face_pending` flag carries the routing through
  the targeting prompt for back-face spells that need a target.

- **`Keyword::Flashback` wirings on 7 SOS cards** — Daydream, Dig
  Site Inventory, Practiced Offense, Antiquities on the Loose,
  Pursue the Past, Tome Blast, Duel Tactics. The engine's existing
  `cast_flashback` path replays each card's body identically when
  cast from graveyard for the flashback cost; the result is exiled
  on resolution.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Daydream | White | ✅ (was 🟡) | Flashback {2}{W} now wired via `Keyword::Flashback`. The flicker pattern + counter rider already worked. |
| Dig Site Inventory | White | ✅ (was 🟡) | Flashback {W} now wired. |
| Tome Blast | Red | ✅ (was 🟡) | Flashback {4}{R} now wired. |
| Duel Tactics | Red | ✅ (was 🟡) | Flashback {1}{R} now wired (mainline ping + transient `CantBlock` from push VI). |
| Studious First-Year // Rampant Growth | Green (MDFC) | ✅ (was ⏳) | First non-land MDFC. Front: 1/1 vanilla Bear Wizard at {G}; back: Rampant Growth ({1}{G} basic-land tutor). The new `GameAction::CastSpellBack` casts the back face. |
| Inkshape Demonstrator | White | 🟡 (was ⏳) | Body + `Keyword::Ward(2)` wired (Ward keyword tagged for future enforcement). Repartee body wired faithfully via `repartee()` shortcut: source +1/+0 + Lifelink (EOT). |
| Fractal Tender | Quandrix | 🟡 (was ⏳) | Body + `Keyword::Ward(2)` wired. Increment trigger and end-step Fractal-with-counters payoff omitted. |
| Thornfist Striker | Green | 🟡 (was ⏳) | Body + `Keyword::Ward(1)` wired. Infusion continuous static omitted. |
| Lumaret's Favor | Green | 🟡 (was ⏳) | Mainline +2/+4 EOT pump wired faithfully. Infusion copy half omitted (no copy-spell primitive). |
| Practiced Scrollsmith | Lorehold | 🟡 | ETB now exiles **exactly one** matching noncreature/nonland card via the new `Selector::Take(_, 1)`. May-cast-until-next-turn rider still omitted. |
| Pull from the Grave | Black | 🟡 | Returns up to **two** creature cards (was: one) via `Selector::Take(_, 2)`. Lifegain unchanged. |
| Antiquities on the Loose | White | 🟡 | Flashback {4}{W}{W} now wired; cast-from-elsewhere counter rider still omitted. |
| Pursue the Past | Lorehold | 🟡 | Flashback {2}{R}{W} now wired; "may discard" optionality still collapsed. |
| Practiced Offense | White | 🟡 | Flashback {1}{W} now wired; lifelink-or-DS mode pick still collapsed. |

Cube color pool updates:
- White: + Inkshape Demonstrator
- Green: + Studious First-Year, Thornfist Striker, Lumaret's Favor
- G/U (Quandrix): + Fractal Tender

Test additions: 14 new tests in `tests::sos::*` covering the new
primitives (Selector::Take, CastSpellBack), Flashback keyword presence,
Flashback graveyard-replay (Daydream + Pursue the Past), and per-card
body shape (Inkshape Demonstrator, Fractal Tender, Thornfist Striker,
Lumaret's Favor, Studious First-Year MDFC). All 953 lib tests pass.

## 2026-05-01 push IX: Witherbloom finisher + Surveil-anchored cards + creatures-died tally

12 new SOS card factories (5 ✅, 7 🟡) plus one new engine primitive,
finishing the Witherbloom (B/G) school except for the Lluwen MDFC
(blocked on cast-from-secondary-face plumbing):

- **`Player.creatures_died_this_turn` + `Predicate::CreaturesDiedThisTurnAtLeast`**
  — per-turn tally bumped from the SBA dies handler in `stack.rs`
  (lethal-damage path) and from `remove_to_graveyard_with_triggers`
  (destroy-effect path). Reset to 0 in `do_untap`. Surfaced on
  `PlayerView.creatures_died_this_turn` so a UI can hint
  "Witherbloom end-step ready". Powers Essenceknit Scholar's
  end-step gated draw.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Essenceknit Scholar | Witherbloom | ✅ (was ⏳) | ETB Pest token (with on-attack lifegain rider) + end-step gated draw via the new `Predicate::CreaturesDiedThisTurnAtLeast`. Hybrid `{B/G}` pip approximated as `{B}`. |
| Professor Dellian Fel | Witherbloom | 🟡 (was ⏳) | New `PlaneswalkerSubtype::Dellian` + 5 base loyalty. +2 (gain 3 life), 0 (draw 1 / lose 1 life), -3 (destroy target creature) all wired faithfully. The -7 emblem ult is omitted (no emblem zone yet). |
| Unsubtle Mockery | Red | ✅ (was ⏳) | 4-to-creature + Surveil 1. Surveil is a first-class engine primitive (the script's `COMPLEX_KWS` heuristic was stale). |
| Muse's Encouragement | Blue | ✅ (was ⏳) | Mints a 3/3 U/R Flying Elemental token + Surveil 2. Same Surveil-already-shipped fix. |
| Prismari Charm | Prismari | ✅ (was ⏳) | 3-mode: Surveil 2 + draw / 1 dmg to creature-or-PW / bounce nonland to owner. Single-target collapse on mode 1 (printed "one or two targets" — multi-target gap). |
| Textbook Tabulator | Blue | 🟡 (was ⏳) | 0/3 Frog Wizard + ETB Surveil 2. Increment rider omitted (mana-spent introspection). |
| Deluge Virtuoso | Blue | 🟡 (was ⏳) | 2/2 Human Wizard + ETB tap+stun against target opp creature. Opus +1/+1-or-+2/+2 rider omitted. |
| Moseo, Vein's New Dean | Black | 🟡 (was ⏳) | 2/1 Flying Bird Skeleton Warlock + ETB Pest token. Infusion end-step rider omitted (oracle truncated; no MayDo per-turn-lifegain primitive). |
| Stone Docent | White | 🟡 (was ⏳) | 3/1 Spirit body. Graveyard-exile activated ability omitted (engine activated-ability walker only iterates the battlefield — same gap as Eternal Student, Summoned Dromedary). |
| Page, Loose Leaf | Colorless | 🟡 (was ⏳) | 0/2 Legendary Construct artifact creature + `{T}: Add {C}` mana ability. Grandeur (discard-named-this-card) ability omitted (no card-name-as-cost activation). |
| Ral Zarek, Guest Lecturer | Black | 🟡 (was ⏳) | 3 base loyalty + +1 Surveil 2 / -1 each opp discards 1 (single-target collapse) / -2 return ≤3 MV creature card from your gy → bf. -7 coin-flip emblem omitted. |
| Flow State | Blue | 🟡 (was ⏳) | Approximated as `Scry 3 + Draw 1`. The conditional "instead pick 2 to hand" upgrade rider when both an instant and sorcery sit in your gy is omitted (no "look-and-distribute-by-count" primitive). |

Cube color pool updates:
- White: + Stone Docent
- Blue: + Deluge Virtuoso, Flow State, Muse's Encouragement, Textbook Tabulator
- Black: + Moseo Vein's New Dean, Ral Zarek Guest Lecturer
- Red: + Unsubtle Mockery
- Witherbloom (B/G): + Essenceknit Scholar, Professor Dellian Fel
- Prismari (U/R): + Prismari Charm

Test additions: 17 new tests in `tests::sos::*` (ETB triggers, end-step
gated draws, planeswalker loyalty activations, Surveil card resolution,
plus a tally-bumps-on-lethal-damage SBA test). All 932 lib tests pass.

## 2026-05-01 push VIII: Lesson cycle + Berta's CounterAdded(SelfSource) + ActivatedAbility.condition

14 new card factories (2 ✅, 12 🟡) bridging the Lesson cycle
(Decorum Dissertation, Restoration Seminar, Germination Practicum)
and a handful of body-only converge / Increment / Repartee bodies.
Engine ships two new primitives unblocking these cards (and several
others):

- **`ActivatedAbility.condition: Option<Predicate>`** — first-class
  "activate only if …" gating. Evaluated against the controller/
  source context **before** any cost is paid, so a failed gate doesn't
  burn the tap-cost or once-per-turn budget. Powers Resonating Lute's
  `{T}: Draw a card. Activate only if you have seven or more cards in
  your hand.` (gate: `ValueAtLeast(HandSizeOf(You), 7)`). Promotes
  Potioner's Trove's lifegain ability — `{T}: You gain 2 life.
  Activate only if you've cast an instant or sorcery spell this turn.`
  — to its printed gate (`SpellsCastThisTurnAtLeast(You, 1)`, an
  approximation of the printed "instant or sorcery"-only filter; the
  engine's per-turn spell tally tracks all spells today). New
  `GameError::AbilityConditionNotMet` for failed gates.
- **`EventScope::SelfSource` for `EventKind::CounterAdded`** —
  `event_card`/`SelfSource` extended to recognise CounterAdded events.
  Berta, the Wise Extrapolator's "whenever one or more +1/+1 counters
  are put on Berta, add one mana of any color" trigger now fires only
  when counters land on Berta (not on every +1/+1 counter on every
  permanent). Same hook will unblock Heliod-style "whenever a counter
  is put on this …" payoffs in future sets.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Primary Research | White | ✅ (was ⏳) | ETB return Nonland & ManaValueAtMost(3) gy → bf + end-step gated draw via `Predicate::CardsLeftGraveyardThisTurnAtLeast`. |
| Artistic Process | Red | ✅ (was ⏳) | Three modes wired: 6-to-creature, 2-to-each-opp-creature (via `Selector::EachPermanent(Creature & ControlledByOpponent)`), Elemental + transient haste via `LastCreatedToken`. |
| Decorum Dissertation | Black (Lesson) | 🟡 (was ⏳) | Mode 0 wired (you draw 2, lose 2 life — collapses target-player to caster). Paradigm rider omitted. |
| Restoration Seminar | White (Lesson) | 🟡 (was ⏳) | Mode 0 wired (return Nonland gy → bf). Paradigm rider omitted. |
| Germination Practicum | Green (Lesson) | 🟡 (was ⏳) | `ForEach Creature & ControlledByYou → AddCounter +1/+1 ×2` fan-out. Paradigm rider omitted. |
| Ennis, Debate Moderator | White | 🟡 (was ⏳) | 1/1 body + ETB flicker (`Exile + DelayUntil(NextEndStep, Move(Target → Battlefield(OwnerOf)))`) + end-step gated counter via `CardsLeftGraveyardThisTurnAtLeast` proxy. |
| Tragedy Feaster | Black | 🟡 (was ⏳) | 7/6 Trample Demon body. Ward—Discard rider + Infusion sac-unless-lifegain rider both omitted. |
| Forum Necroscribe | Black | 🟡 (was ⏳) | 5/4 Troll Warlock body + Repartee gy-creature-recursion (`repartee()` chained with `Move(target Creature → bf)`). Ward—Discard rider omitted. |
| Berta, Wise Extrapolator | Quandrix | 🟡 (was ⏳) | 1/4 Legendary Frog Druid + CounterAdded(+1/+1, SelfSource) → `AddMana(AnyOneColor)` trigger + X-cost Fractal-token activation (X resolves to 0 today). Increment rider omitted. |
| Paradox Surveyor | Quandrix | 🟡 (was ⏳) | 3/3 Reach Elf Druid + ETB `RevealUntilFind(IsBasicLand, cap 5)`. Hybrid {G/U} approximated as {G}. |
| Magmablood Archaic | Red | 🟡 (was ⏳) | 2/2 Trample+Reach Avatar + Converge ETB `AddCounter(Value::ConvergedValue)`. IS-cast pump rider omitted (per-cast converge introspection). Hybrid `{2/R}` approximated as `{2}+{R}` per pip. |
| Wildgrowth Archaic | Green | 🟡 (was ⏳) | 0/0 Trample+Reach Avatar + Converge ETB AddCounter. Hybrid `{2/G}` approximated as `{2}+{G}` per pip. The "creature spells you cast enter with X extra counters" rider is omitted. |
| Ambitious Augmenter | Green | 🟡 (was ⏳) | 1/1 Turtle Wizard at {G}. Increment + dies-with-counters → Fractal-with-counters omitted. |
| Resonating Lute | Prismari | 🟡 (was ⏳) | {T}: Draw a card with new `ActivatedAbility.condition: ValueAtLeast(HandSizeOf(You), 7)` gate. Lands-grant tap-for-2 omitted. |

Cube color pool updates:
- White: + Primary Research, Restoration Seminar, Ennis the Debate Moderator
- Black: + Decorum Dissertation, Tragedy Feaster, Forum Necroscribe
- Red: + Artistic Process, Magmablood Archaic
- Green: + Germination Practicum, Wildgrowth Archaic, Ambitious Augmenter
- G/U (Quandrix): + Berta the Wise Extrapolator, Paradox Surveyor
- U/R (Prismari): + Resonating Lute

Test additions: 22 new tests in `tests::sos::*` covering the new
factories' primary play patterns + the new Resonating Lute and
Potioner's Trove condition gates (positive + negative cases). All
910 lib tests pass.

## 2026-05-01 push VII: Multicolored predicate + MDFC bodies + Lorehold capstone

10 new card factories (3 ✅, 7 🟡) plus 2 promotions (Owlin Historian
🟡 → ✅, Postmortem Professor — kept 🟡 but now carries the printed
`Keyword::CantBlock`). Engine ships two new primitives:

- **`SelectionRequirement::Multicolored`** + **`Colorless`** — count
  the *distinct* colored pips in a card's mana cost. Hybrid pips
  (`{W/B}`) contribute both halves; Phyrexian (`{B/P}`) contributes
  the colored half; generic / colorless / Snow / X don't count. Cost
  ≥ 2 distinct ⇒ multicolored; cost = 0 distinct ⇒ colorless.
  Powered by the new `ManaCost::distinct_colors()` helper.

- **`tap_add_colorless()` shared land helper** — `{T}: Add {C}` mana
  ability shorthand under `catalog::sets::mod`. Used by Petrified
  Hamlet (and ready for Wastes / future Eldrazi-flavoured colorless
  lands).

| Card | School / Color | Status | Note |
|---|---|---|---|
| Mage Tower Referee | Colorless | ✅ (was ⏳) | Trigger filtered on `EntityMatches(TriggerSource, Multicolored)`. |
| Additive Evolution | Green | ✅ (was ⏳) | ETB Fractal token + 3 +1/+1 counters via `Selector::LastCreatedToken`; begin-combat counter+vigilance pump. |
| Owlin Historian | White | ✅ (was 🟡) | Now wires the cards-leave-graveyard +1/+1 EOT pump via `EventKind::CardLeftGraveyard`. |
| Spectacular Skywhale | Prismari | 🟡 (was ⏳) | 1/4 Flying body. Opus rider omitted. |
| Lorehold, the Historian | Lorehold | 🟡 (was ⏳) | 5/5 Flying+Haste Legendary Elder Dragon body. Miracle grant + opp-upkeep loot omitted. |
| Homesickness | Blue | 🟡 (was ⏳) | Draw 2 (you) + Tap target creature + Stun 1. Multi-target prompts collapsed. |
| Fractalize | Blue | 🟡 (was ⏳) | `PumpPT(+(X+1), +(X+1)) EOT` — base-(X+1)/(X+1) rewrite collapsed. |
| Divergent Equation | Blue | 🟡 (was ⏳) | Single-target return one IS card from graveyard to hand. |
| Rubble Rouser | Red | 🟡 (was ⏳) | ETB rummage (collapsed `you may` to always-do). Activated `{T}, Exile from graveyard` omitted. |
| Zimone's Experiment | Green | 🟡 (was ⏳) | RevealUntilFind(creature → hand) + Search(IsBasicLand → bf tapped) approximation. |
| Petrified Hamlet | Colorless | 🟡 (was ⏳) | `{T}: Add {C}` only. Choose-name prompt + name-keyed lock-out omitted. |
| Postmortem Professor | Black | 🟡 (was 🟡) | Now carries `Keyword::CantBlock`; gy recursion still omitted. |

Cube color pool updates:
- Blue: + Divergent Equation, Fractalize, Homesickness
- Red: + Rubble Rouser
- Green: + Additive Evolution, Zimone's Experiment
- U/R (Prismari): + Spectacular Skywhale
- R/W (Lorehold): + Lorehold, the Historian

Test additions: 11 new tests in `tests::sos::*` covering Homesickness,
Fractalize, Divergent Equation, Spectacular Skywhale (def shape),
Lorehold the Historian (def shape), Mage Tower Referee (multicolored
+ mono-color cast), Rubble Rouser, Additive Evolution (ETB +
combat), Zimone's Experiment, Petrified Hamlet, Owlin Historian's
new pump trigger, Postmortem Professor's CantBlock keyword. Plus
3 mana-cost tests covering `ManaCost::distinct_colors`. All 885
lib tests pass.

## 2026-05-01 push VI: Lorehold completion + token-side triggers + ManaPayload::OfColor

12 new card factories + ~12 new functionality tests under
`tests::sos::*`. Engine ships three new primitives unblocking these
cards (and several more):

- **`TokenDefinition.triggered_abilities`** — token definitions now
  carry triggered abilities. `token_to_card_definition` copies them
  through, so the SOS Pest token's "Whenever this token attacks, you
  gain 1 life" rider and the STX Pest token's "When this creature
  dies, you gain 1 life" rider both fire. Witherbloom payoffs (Pest
  Mascot's lifegain → +1/+1 counter, Blech's per-creature-type counter
  fan-out, Bogwater Lumaret, Cauldron of Essence's death drain triple-
  loop) get the printed lifegain trickle for free. Promoted Send in
  the Pest, Pestbrood Sloth, Pest Summoning, Tend the Pests, Hunt for
  Specimens (their tokens now print correctly).
- **`ManaPayload::OfColor(Color, Value)`** — fixed-color, value-scaled
  mana adder. Single AddMana call produces N pips of a specified
  color. Powers Topiary Lecturer's "{T}: Add G equal to power"
  (replaces the prior `Repeat × Colors([Green])` approximation), and
  is ready for any future power-scaled mana ability (Llanowar Mentor,
  Wirewood Channeler, Cryptolith Rite-style scaling).
- **`Keyword::CantBlock`** — "this creature can't block" as a first-
  class keyword. Enforced inside `declare_blockers` and the
  `can_block_*` helpers. Used by Duel Tactics's "1 damage + can't
  block this turn" pump and the static restriction on Postmortem
  Professor (which can now be promoted).
- **`move_card_to` library traversal** — `Effect::Move` from a
  `Selector::TopOfLibrary` source now actually moves the top library
  card to the destination. Previously the library branch in
  `move_card_to` was missing, so Suspend Aggression's "exile the top
  card of your library" half silently no-op'd. The library-source
  move is now last in the search order (battlefield → graveyard →
  exile → hand → library) to avoid accidentally consuming a hand card
  with the same id.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Daydream | White | 🟡 (was ⏳) | Restoration-Angel-style flicker pattern (`Exile + Move(target → bf) + AddCounter +1/+1`). Flashback half omitted. |
| Soaring Stoneglider | White | 🟡 (was ⏳) | 4/3 Flying-Vigilance Elephant Cleric at the **paid** cost path: full {3}{W}. Alt cost (exile two from gy) omitted. |
| Tome Blast | Red | 🟡 (was ⏳) | 2-to-any-target burn. Flashback half omitted. |
| Duel Tactics | Red | 🟡 (was ⏳) | 1-to-creature ping + new `Keyword::CantBlock` (EOT). Flashback half omitted. |
| Snarl Song | Green | ✅ (was ⏳) | Two Fractal tokens, each stamped with X +1/+1 counters via `Selector::LastCreatedToken`, plus X life. X = `Value::ConvergedValue`. |
| Wild Hypothesis | Green | ✅ (was ⏳) | Fractal token + X +1/+1 counters + Surveil 2 (all first-class primitives). |
| Topiary Lecturer | Green | 🟡 (was ⏳) | Now uses `ManaPayload::OfColor(Green, PowerOf(This))` — single AddMana, value-scaled count. Increment rider still omitted. |
| Ark of Hunger | Lorehold | 🟡 (was ⏳) | `EventKind::CardLeftGraveyard` drain trigger + {T}: Mill 1. May-play-from-mill rider omitted. |
| Suspend Aggression | Lorehold | 🟡 (was ⏳) | Exile target nonland permanent + exile top of library (library traversal added to `move_card_to`). May-play rider omitted. |
| Wilt in the Heat | Lorehold | 🟡 (was ⏳) | 5-to-creature. Cost reduction + die-replace-with-exile rider omitted. |
| Practiced Scrollsmith | Lorehold | 🟡 (was ⏳) | First strike body + ETB exiles every matching noncreature/nonland in your gy (no `Selector::OneOf` primitive yet). May-cast-until-next-turn rider omitted. Hybrid `{R/W}` approximated as `{R}`. |
| Send in the Pest | Black | ✅ (was 🟡) | Token-side attack-trigger lifegain wired. |
| Pestbrood Sloth | Green | ✅ (was 🟡) | Token-side attack-trigger lifegain wired. |

Cube color pool updates:
- White: + Daydream, Soaring Stoneglider
- Red: + Tome Blast, Duel Tactics
- Green: + Snarl Song, Wild Hypothesis, Topiary Lecturer
- R/W (Lorehold): + Ark of Hunger, Wilt in the Heat

## 2026-04-30 push V: CardLeftGraveyard event + Lorehold/mono-color batch

12 new card factories + 13 new functionality tests under
`tests::sos::*` (+1 in `tests::modern::*` for the new Untap cap).
Engine ships three new primitives unblocking these cards (and others
in the future):

- **`EventKind::CardLeftGraveyard`** + `GameEvent::CardLeftGraveyard`
  — fires per card removed from a graveyard. Plumbed in:
  `move_card_to`'s graveyard branch, `cast_spell_flashback` in
  actions.rs, and persist/undying battlefield-returns in stack.rs.
  Each emission also bumps the new
  `Player.cards_left_graveyard_this_turn` tally (reset on `do_untap`),
  which `Predicate::CardsLeftGraveyardThisTurnAtLeast` reads. Surfaced
  through `PlayerView` so client UIs can render "Lorehold ready"
  hints.
- **`Predicate::SpellsCastThisTurnAtLeast`** — gates Burrog Barrage's
  conditional pump on "have you already cast another spell this turn".
- **`Effect::Fight { attacker, defender }`** — proper bidirectional
  fight primitive. Snapshots both creatures' powers up-front so the
  back-swing isn't affected by the first hit. No-ops cleanly when
  either selector resolves to no permanent (matches MTG's "if either
  is no longer a creature, no damage is dealt"). Used by Chelonian
  Tackle; future cards (Decisive Denial mode 1, fight-style green
  removal) can drop in trivially.
- **`Effect::Untap.up_to: Option<Value>`** — untap-with-cap. Frantic
  Search's "untap up to three lands" now honors the printed cap
  precisely (previously the engine collapsed it to "untap all"). Other
  Untap callers (Cryptolith Rite, mass-untap effects) opt-out via
  `up_to: None`.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Hardened Academic | Lorehold | ✅ (was 🟡) | Cards-leave-gy → +1/+1 counter on a friendly creature. Triggers per-card (the printed "one or more" wording is approximated by per-card emission). |
| Spirit Mascot | Lorehold | ✅ (was 🟡) | Self +1/+1 counter on every gy-leave event. |
| Garrison Excavator | Red mono / Lorehold | ✅ (was 🟡) | 2/2 R/W Spirit token on every gy-leave event. |
| Living History | Red mono / Lorehold | 🟡 (was ⏳) | ETB Spirit token + on-attack +2/+0 to attacker if a card left your graveyard this turn (gated on the new per-turn tally). |
| Witherbloom, the Balancer | Witherbloom | 🟡 (was ⏳) | 5/5 Legendary Elder Dragon (new `CreatureType::Elder`); Flying + Deathtouch. Affinity-for-creatures cost reduction stub (engine work tracked in TODO.md). |
| Burrog Barrage | Green mono | 🟡 (was ⏳) | Conditional pump (gated on `Predicate::SpellsCastThisTurnAtLeast`) + power-as-damage to target. |
| Chelonian Tackle | Green mono | 🟡 (was ⏳) | +0/+10 EOT pump + `Effect::Fight` against auto-selected opp creature (single-target collapse on the defender pick). |
| Rabid Attack | Black mono | 🟡 (was ⏳) | +1/+0 friendly pump (multi-target prompt + die-to-draw rider omitted). |
| Practiced Offense | White mono | 🟡 (was ⏳) | Fan-out +1/+1 to friendly creatures + double-strike grant on a target. Mode-pick (DS vs lifelink) collapsed to DS. |
| Mana Sculpt | Blue mono | 🟡 (was ⏳) | Counter target spell + wizard-mana-refund rider (fixed +{C}{C} as mana-spent introspection unavailable). |
| Tablet of Discovery | Red mono | 🟡 (was ⏳) | ETB-mill + `{T}: Add {R}` mana abilities (may-play-from-mill rider + spend-restriction omitted). |
| Steal the Show | Red mono | 🟡 (was ⏳) | Modal sorcery: discard-then-draw (collapsed to "discard 2, draw 2") OR damage = #IS-cards-in-your-gy to target creature/PW. |
| Frantic Search | Modern (cube) | ✅ upgrade | Untap-up-to-three now precise (was "untap all") via the new `Effect::Untap.up_to` cap. |

Cube color pool updates:
- White: + Practiced Offense
- Blue: + Mana Sculpt
- Black: + Rabid Attack
- Red: + Tablet of Discovery, Garrison Excavator, Living History, Steal the Show
- Green: + Burrog Barrage, Chelonian Tackle
- R/W (Lorehold): + Hardened Academic, Spirit Mascot
- B/G (Witherbloom): + Witherbloom, the Balancer

## 2026-04-30 push IV: post-push-III modern_decks batch

10 new card factories + 11 new functionality tests under
`tests::sos::*`. Engine ships five new primitives unblocking these
cards (and others in the future):

- **`Value::Pow2(Box<Value>)`** — two raised to a value, capped at
  exponent 30. Powers Mathemagics's "draw 2ˣ cards".
- **`Value::HalfDown(Box<Value>)`** — half of a value, rounded down.
  Powers Pox Plague's "loses half their life / discards half / sacs
  half" clauses.
- **`Value::PermanentCountControlledBy(PlayerRef)`** — counts the
  permanents controlled by the resolved player. Lets Pox Plague's
  per-player iteration compute the right "half their permanents"
  count under a `ForEach` over each player.
- **`Selector::CastSpellTarget(u8)`** — resolves the chosen target
  slot of the just-cast spell whose `SpellCast` event produced the
  current trigger. Walks the stack for the topmost matching spell.
  Used by Conciliator's Duelist's Repartee exile half (the body
  pulls the target off the cast spell rather than choosing a fresh
  target). Future Repartee-exile-bounce-counter spells get this for
  free.
- **`AffectedPermanents::AllWithCounter`** — counter-filtered
  lord-style statics. `affected_from_requirement` recognises
  `SelectionRequirement::WithCounter(...)` and routes through the
  new variant. Powers Emil's "creatures with +1/+1 counters have
  trample" + future "monstrous / leveled creatures gain
  [keyword]" buffs.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Mathemagics | Blue | ✅ | Draw 2ˣ cards via `Value::Pow2(XFromCost)`; multi-target collapsed to "you draw". |
| Visionary's Dance | Prismari | ✅ | Two 3/3 flying Elemental tokens (new `elemental_token()` helper); discard activation from hand omitted. |
| Pox Plague | Black | ✅ | `ForEach Player` body using `HalfDown(Life/Hand/PermanentCount)` for each clause. |
| Emil, Vastlands Roamer | Green | ✅ | Static GrantKeyword(Trample) filtered to creatures with +1/+1 counters via `AllWithCounter`; tap+{4}{G} fractal-token activation scaling on lands. |
| Orysa, Tide Choreographer | Blue | ✅ | ETB draw 2; conditional cost reduction omitted. |
| Conciliator's Duelist | Silverquill | 🟡 | Repartee exile of cast spell's target via `Selector::CastSpellTarget(0)`; "return at next end step" rider still omitted. |
| Abstract Paintmage | Prismari | 🟡 | Trigger fires {U}{R} on PreCombatMain step; spend restriction omitted. |
| Matterbending Mage | Blue | 🟡 | ETB bounce target creature; "spell with X" trigger omitted. |
| Exhibition Tidecaller | Blue | 🟡 | 0/2 body wired; Opus mill rider omitted. |
| Colossus of the Blood Age | Lorehold | 🟡 | ETB drain wired; death "discard any number" collapsed to discard 1 / draw 2. |

Cube color pool updates:
- Blue: + Mathemagics, Matterbending Mage, Orysa Tide Choreographer,
  Exhibition Tidecaller
- Black: + Pox Plague
- Green: + Emil, Vastlands Roamer
- U/R (Prismari): + Visionary's Dance, Abstract Paintmage
- R/W (Lorehold): + Colossus of the Blood Age

Earlier counts in this section quoted 86 ✅ / 38 🟡 / 138 ⏳; that
header drift was reconciled when `scripts/gen_strixhaven2.py` was
re-run with full Scryfall oracle text. Some rows that had been claimed
✅ on a stub (no implementation in the SOS module) were corrected back
to 🟡 or ⏳, and several ⏳ cards from the body-only batch were
correctly reclassified to 🟡.

## 2026-04-30 push II: SOS Increment / Opus body-only batch

13 cards bumped from ⏳ → 🟡 by shipping the printed cost / type / P/T /
keywords without their Increment / Opus / mana-spent-pump rider. Each
rider depends on a "mana-paid-to-cast introspection on cast" engine
primitive (tracked in TODO.md). The vanilla bodies are still useful: they
fill out cube color pools, take combat correctly, and can be promoted to
✅ once the engine grows the right hooks. 11 functionality tests in
`tests::sos::*` exercise the bodies (P/T, keywords, ETB/attack triggers
where relevant).

Plus one new ✅-functionality card with omitted-rider note:
- **Ajani's Response** {4}{W} — destroy creature; cost-reduction-when-
  target-tapped omitted (logged as a TODO under "target-aware cost
  reduction").

New CreatureTypes: `Dwarf` (Thunderdrum Soloist, Scolding
Administrator), `Badger` (Shopkeeper's Bane), `Salamander` (Noxious
Newt), `Giraffe` (Hungry Graffalon).

Plus a follow-up Silverquill card: **Scolding Administrator** {W}{B} 2/2
Dwarf Cleric — Menace + Repartee +1/+1 counter on self (the truncated
"When this creature dies, …" trigger is unimplemented pending an
oracle-fetch refresh).

Cube color pool updates:
- White: + Ajani's Response
- Blue: + Pensive Professor, Tester of the Tangential, Muse Seeker
- U/G: + Cuboid Colony
- Red: + Tackle Artist, Thunderdrum Soloist, Molten-Core Maestro,
  Expressive Firedancer
- Green: + Aberrant Manawurm, Hungry Graffalon
- Black: + Eternal Student, Postmortem Professor

## 2026-04-30 push: Silverquill / Lorehold / mono-color expansion

11 new cards bridging the Silverquill (W/B) school's gap and a
handful of cross-school removal/utility staples, all wired entirely on
existing primitives (no engine work needed). Plus 11 functionality
tests in `tests::sos::*`.

| Card | School | Status | Note |
|---|---|---|---|
| Moment of Reckoning | Silverquill | ✅ | Modal destroy / graveyard return — "choose up to four" collapsed to single mode pick. |
| Stirring Honormancer | Silverquill | ✅ | ETB look-at-X-find-creature via `RevealUntilFind`. |
| Conciliator's Duelist | Silverquill | 🟡 | ETB draw + each-player-loses-1; Repartee exile-with-return rider omitted. |
| Dissection Practice | Black mono | 🟡 | Drain 1 + creature -1/-1; +1/+1 mode collapsed (multi-target gap). |
| Heated Argument | Red mono | 🟡 | 6 to creature + 2 to controller; "may exile" optionality dropped. |
| End of the Hunt | Black mono | 🟡 | Exile opponent's creature/PW; "greatest mana value" picker not enforced. |
| Vicious Rivalry | Witherbloom | ✅ | X-life cost approximation via `LoseLife` + `ForEach.If(ManaValueAtMost X)` destroy. |
| Proctor's Gaze | Quandrix | ✅ | Bounce nonland + Search basic to bf tapped. |
| Lorehold Charm | Lorehold | ✅ | All three modes (sac-artifact / return ≤2-mv / +2/+1). |
| Borrowed Knowledge | Lorehold | 🟡 | Mode 0 wired faithfully; mode 1 collapsed to "draw 7" (no track-discarded primitive). |
| Planar Engineering | Green mono | ✅ | Sacrifice 2 lands + Repeat×4 Search basic to bf tapped. |

Cross-pool wiring updated in `cube.rs`:
- W/B (Silverquill): added Moment of Reckoning, Stirring Honormancer,
  Conciliator's Duelist
- B/G (Witherbloom): added Vicious Rivalry
- G/U (Quandrix): added Proctor's Gaze
- R/W (Lorehold): added Borrowed Knowledge, Lorehold Charm

Mono-color pools picked up Dissection Practice, End of the Hunt
(Black), Heated Argument (Red), Planar Engineering (Green).

## Oracle re-fetch (2026-04-30)

`scripts/gen_strixhaven2.py` no longer truncates oracle text (was 600
chars, now unlimited). 52 SOS rows whose oracle column was previously
clipped have been tagged **🔍 needs review (oracle previously
truncated)** in the Notes column. Re-running the script against fresh
Scryfall pages (`sos_p1.json`, `sos_p2.json`) will replace the
truncated bodies with the full oracle, but until that happens the
🔍 marker flags every row whose status / engine-gap analysis was
based on incomplete text. When implementing one of these cards the
first step is to fetch the full oracle from Scryfall and verify the
existing Notes are still accurate.

First-pass coverage focuses on Silverquill (W/B) and Witherbloom (B/G)
plus the easier mono-color removal/utility cards. School lands
(Forum of Amity, Titan's Grave, Fields of Strife, Paradox Gardens) are
all wired since the surveil primitive is already in the engine. Quandrix
(G/U) and Prismari (U/R) now have first-pass coverage too — Pterafractyl,
Fractal Mascot, Mind into Matter, Growth Curve, Quandrix Charm,
Stadium Tidalmage, Vibrant Outburst, Stress Dream, and Traumatic Critique
are wired. Remaining SOS gaps are mostly **Repartee** (spell-targets-
creature predicate), **Increment** / **Opus** (mana-paid introspection on
cast), **Casualty** (copy-spell primitive), and **Flashback / Paradigm**
(cast-from-graveyard pipeline).

**X-cost ETB triggers** now correctly read the spell's paid X via
`StackItem::Trigger.x_value`. Pterafractyl's "enters with X +1/+1
counters" and Static Prison's "enters with X stun counters" both
honour the cast-time X (previously the trigger context defaulted to
`x_value: 0`).

**Infusion** ("if you gained life this turn, …") is now wired engine-side
via `Predicate::LifeGainedThisTurnAtLeast`, with **Foolish Fate**,
**Old-Growth Educator**, and **Efflorescence** as the first three
beneficiaries. Remaining Infusion cards (Tenured Concocter,
Ulna Alley Shopkeep, Tragedy Feaster, Withering Curse, …) need either
Ward enforcement or static "as long as you've gained life this turn"
gating which is a separate engine primitive.

### Prepare mechanic (SOS colorless)

A small SOS sub-theme where one card toggles a "prepared" flag on a
creature and another card cares about it. The flag works like a stun /
phased / monstrosity counter: it's a per-permanent boolean (or counter
of count 1) set by `becomes prepared` and cleared by `becomes
unprepared`. Cards that *care* about the flag have a **Prepare {cost}**
activated/triggered ability and reminder text "(Only creatures with
prepare spells can become prepared.)".

Cards in the SOS table that touch the mechanic:

- **Biblioplex Tomekeeper** ({4} 3/4) — ETB toggle (prepare or unprepare).
- **Skycoach Waypoint** (land) — `{3},{T}: prepare target`.
- Cards whose oracle text was previously truncated by the gen script's
  220-char cap (now 600 chars) may also expose a `Prepare {cost}` ability;
  when re-running `scripts/gen_strixhaven2.py` look for "prepare " or
  "prepared" in the oracle column to spot them.

Engine-side this needs:

1. A new `CounterType::Prepared` (or a `PermanentFlag::Prepared` boolean)
   on `Permanent`, surfaced through `PermanentView` for the client UI.
2. `Effect::SetPrepared { what, value: bool }` to flip the flag.
3. A `Predicate::IsPrepared` so prepare-payoff cards (the cards
   *granting* a Prepare ability) can gate their riders on the flag.
4. The activated ability *itself* on payoff cards — those need an
   ability authored from the truncated body of the card.

None of these are wired today; all prepare cards are ⏳ until at least
(1) and (2) land. Track in `TODO.md` under Engine — Missing Mechanics.

---

## White

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Ajani's Response | {4}{W} | Instant |  | This spell costs {3} less to cast if it targets a tapped creature. / Destroy target creature. | 🟡 | Wired in `catalog::sets::sos::instants` as a {4}{W} hard destroy. The "costs {3} less if target is tapped" cost-reduction rider is omitted (no target-aware cost reduction primitive). |
| Antiquities on the Loose | {1}{W}{W} | Sorcery |  | Create two 2/2 red and white Spirit creature tokens. Then if this spell was cast from anywhere other than your hand, put a +1/+1 counter on each Spirit you control. / Flashback {4}{W}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Wired in `catalog::sets::sos::sorceries` — creates two 2/2 R/W Spirit tokens. The cast-from-elsewhere rider and Flashback half are omitted (no cast-from-graveyard pipeline yet). |
| Ascendant Dustspeaker | {4}{W} | Creature — Orc Cleric | 3/4 | Flying / When this creature enters, put a +1/+1 counter on another target creature you control. / At the beginning of combat on your turn, exile up to one target card from a graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` with both ETB pump + combat-step exile triggers. |
| Daydream | {W} | Sorcery |  | Exile target creature you control, then return that card to the battlefield under its owner's control with a +1/+1 counter on it. / Flashback {2}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired in `catalog::sets::sos::sorceries` as the standard Restoration-Angel-style flicker pattern (`Exile + Move(target → battlefield) + AddCounter`). Flashback {2}{W} now wired via `Keyword::Flashback` (push X) — graveyard replay reuses the engine's existing `cast_flashback` path. The library traversal in `move_card_to` was extended to handle library-source moves so the flicker round-trip resolves end-to-end. |
| Dig Site Inventory | {W} | Sorcery |  | Put a +1/+1 counter on target creature you control. It gains vigilance until end of turn. / Flashback {W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Mainline pump+vigilance wired in `catalog::sets::sos::sorceries`; Flashback {W} clause now wired via `Keyword::Flashback` (push X). |
| Eager Glyphmage | {3}{W} | Creature — Cat Cleric | 3/3 | When this creature enters, create a 1/1 white and black Inkling creature token with flying. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Elite Interceptor // Rejoinder | {W} // {1}{W} | Creature — Human Wizard // Sorcery | 1/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Emeritus of Truce // Swords to Plowshares | {1}{W}{W} // {W} | Creature — Cat Cleric // Instant | 3/3 |  | ✅ | Push XXVIII promotion: vanilla 3/3 Cat Cleric front + faithful Swords to Plowshares back (`Exile target creature` + `GainLife(target's power) → controller of target`). The `PlayerRef::ControllerOf` resolves at resolution time so the *target's controller* gets the life, not the caster. Tests: `emeritus_of_truce_front_is_three_three_cat_cleric`, `emeritus_of_truce_back_face_is_swords_to_plowshares`, `emeritus_of_truce_back_exiles_creature_and_grants_life`. |
| Ennis, Debate Moderator | {1}{W} | Legendary Creature — Human Cleric | 1/1 | When Ennis enters, exile up to one other target creature you control. Return that card to the battlefield under its owner's control at the beginning of the next end step. / At the beginning of your end step, if one or more cards were put into exile this turn, put a +1/+1 counter on Ennis. | 🟡 | Wired in `catalog::sets::sos::creatures` — ETB flicker (`Exile + DelayUntil(NextEndStep, Move(Target → Battlefield(OwnerOf)))` pattern, same as Restoration Angel) + end-step counter gated on `Predicate::CardsLeftGraveyardThisTurnAtLeast` as a proxy for "any card put into exile this turn" (under-counts pure hand-exile / bounce-to-exile, covers gy-leave / flicker / exile-from-gy). Once a per-turn exile-count tally lands the gate can swap to the exact predicate. |
| Erode | {W} | Instant |  | Destroy target creature or planeswalker. Its controller may search their library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | Push XV: now fully wired. Destroy + `Search { who: ControllerOf(Target), filter: IsBasicLand, to: Battlefield(ControllerOf(Target), tapped) }`. The "may" optionality is collapsed to always-search (decline path covered by `Effect::Search`'s decider returning `Search(None)`). |
| Graduation Day | {W} | Enchantment |  | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on target creature you control. | ✅ | Wired in `catalog::sets::sos::enchantments` via `repartee()` shortcut + `target_filtered(Creature & ControlledByYou)` AddCounter. |
| Group Project | {1}{W} | Sorcery |  | Create a 2/2 red and white Spirit creature token. / Flashback—Tap three untapped creatures you control. (You may cast this card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Mainline 2/2 R/W Spirit token wired (new `spirit_token()` helper); Flashback "tap three" cost omitted. |
| Harsh Annotation | {1}{W} | Instant |  | Destroy target creature. Its controller creates a 1/1 white and black Inkling creature token with flying. | ✅ | Push XVII: token now goes to the target creature's owner via `PlayerRef::OwnerOf(Target(0))`. `place_card_in_dest` resolves the player against cast-time ctx (the target id stays valid through `find_card_owner`'s zone walk after the destroy step). |
| Honorbound Page // Forum's Favor | {3}{W} // {W} | Creature — Cat Cleric // Sorcery | 3/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Informed Inkwright | {1}{W} | Creature — Human Wizard | 2/2 | Vigilance / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, create a 1/1 white and black Inkling creature token with flying. | ✅ | Vigilance body + Repartee Inkling token wired via `repartee()` + `inkling_token()`. |
| Inkshape Demonstrator | {3}{W} | Creature — Elephant Cleric | 3/4 | Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.) / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gets +1/+0 and gains lifelink until end of turn. | 🟡 | Body + `Keyword::Ward(2)` wired in `catalog::sets::sos::creatures` (Ward keyword tagged for future enforcement; not yet a counter-the-spell trigger). Repartee body wired faithfully via the `repartee()` shortcut: pump +1/+0 on the source + grant Lifelink (EOT). |
| Interjection | {W} | Instant |  | Target creature gets +2/+2 and gains first strike until end of turn. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Joined Researchers // Secret Rendezvous | {1}{W} // {1}{W}{W} | Creature — Human Cleric Wizard // Sorcery | 2/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Owlin Historian | {2}{W} | Creature — Bird Cleric | 2/3 | Flying / When this creature enters, surveil 1. (Look at the top card of your library. You may put it into your graveyard.) / Whenever one or more cards leave your graveyard, this creature gets +1/+1 until end of turn. | ✅ | All three abilities wired. The cards-leave-graveyard pump uses the SOS-V `EventKind::CardLeftGraveyard` event (per-card emission; the printed "one or more" wording approximates as per-card). |
| Practiced Offense | {2}{W} | Sorcery |  | Put a +1/+1 counter on each creature target player controls. Target creature gains your choice of double strike or lifelink until end of turn. / Flashback {1}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Wired in `catalog::sets::sos::sorceries` as fan-out +1/+1 (collapsed to "each creature you control") + double-strike grant on the chosen creature target. Lifelink-or-DS mode pick collapses to DS. Flashback omitted (cast-from-graveyard pipeline not wired yet). |
| Primary Research | {4}{W} | Enchantment |  | When this enchantment enters, return target nonland permanent card with mana value 3 or less from your graveyard to the battlefield. / At the beginning of your end step, if a card left your graveyard this turn, draw a card. | ✅ | Wired in `catalog::sets::sos::enchantments`. ETB returns target Nonland & ManaValueAtMost(3) gy → bf. End-step gated draw uses `Predicate::CardsLeftGraveyardThisTurnAtLeast`. |
| Quill-Blade Laureate // Twofold Intent | {1}{W} // {1}{W} | Creature — Human Cleric // Sorcery | 1/1 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Rapier Wit | {1}{W} | Instant |  | Tap target creature. If it's your turn, put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) / Draw a card. | ✅ | Wired in `catalog::sets::sos::instants` with `IsTurnOf` gating on the stun counter. |
| Rehearsed Debater | {2}{W} | Creature — Djinn Bard | 3/3 | Vigilance / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gets +1/+1 until end of turn. | ✅ | Vigilance + Repartee +1/+1 EOT, via `effect::shortcut::repartee()` + `Predicate::CastSpellTargetsMatch`. |
| Restoration Seminar | {5}{W}{W} | Sorcery — Lesson |  | Return target nonland permanent card from your graveyard to the battlefield. / Paradigm (...) | 🟡 | Wired in `catalog::sets::sos::sorceries`. Mode 0 (`Move target Nonland gy → bf untapped`) wired faithfully. Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive — same gap as Decorum Dissertation, Improvisation Capstone, Echocasting Symposium). |
| Shattered Acolyte | {1}{W} | Creature — Dwarf Warlock | 2/2 | Lifelink / {1}, Sacrifice this creature: Destroy target artifact or enchantment. | ✅ | Wired in `catalog::sets::sos::creatures` with `sac_cost` activation. |
| Soaring Stoneglider | {2}{W} | Creature — Elephant Cleric | 4/3 | As an additional cost to cast this spell, exile two cards from your graveyard or pay {1}{W}. / Flying, vigilance | 🟡 | Wired in `catalog::sets::sos::creatures` as a 4/3 Flying+Vigilance Elephant Cleric at the **paid** cost path: full {3}{W} (base {2}{W} + the {1}{W} payment fork). The alternative additional cost (exile two from gy) is omitted (no alt-cost-with-exile-from-gy primitive). |
| Spiritcall Enthusiast // Scrollboost | {2}{W} // {1}{W} | Creature — Cat Cleric // Sorcery | 3/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Stand Up for Yourself | {2}{W} | Instant |  | Destroy target creature with power 3 or greater. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Stirring Hopesinger | {2}{W} | Creature — Bird Bard | 1/3 | Flying, lifelink / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on each creature you control. | ✅ | Flying/lifelink body + Repartee fan-out via `ForEach(Creature & ControlledByYou) → AddCounter`. |
| Stone Docent | {1}{W} | Creature — Spirit Chimera | 3/1 | {W}, Exile this card from your graveyard: You gain 2 life. Surveil 1. Activate only as a sorcery. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Push XVII: graveyard-exile activation wired via the new `ActivatedAbility.from_graveyard: bool` + `exile_self_cost: bool` fields. The `activate_ability` engine path now walks the graveyard for `from_graveyard` abilities and exiles the source as part of the cost (mirror to `sac_cost` for battlefield activations). Sorcery-speed gate also now enforced. |
| Summoned Dromedary | {3}{W} | Creature — Spirit Camel | 4/3 | Vigilance / {1}{W}: Return this card from your graveyard to your hand. Activate only as a sorcery. | ✅ | Push XVII: graveyard-recursion activation wired via the new `from_graveyard: bool` field. Cost `{1}{W}` + sorcery-speed + effect `Move(Self → Hand(You))`. |

## Blue

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Banishing Betrayal | {1}{U} | Instant |  | Return target nonland permanent to its owner's hand. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Brush Off | {2}{U}{U} | Instant |  | This spell costs {1}{U} less to cast if it targets an instant or sorcery spell. / Counter target spell. | 🟡 | Wired in `catalog::sets::sos::instants` as a 4-mana hard counter. The conditional cost-reduction-when-targeting-IS rider is omitted (no target-aware cost reduction primitive). |
| Campus Composer // Aqueous Aria | {3}{U} // {4}{U} | Creature — Merfolk Bard // Sorcery | 3/4 |  | 🟡 | Push XXVI: Front 3/4 Merfolk Bard with `Keyword::Ward(1)` (engine keyword tag; targeting enforcement still pending). Back-face Aqueous Aria draws three cards (collapses "target player" to "you" — no multi-target prompt yet). Was a ⏳ row blocked on the Ward keyword primitive; Ward is wired as a keyword tag now. |
| Chase Inspiration | {U} | Instant |  | Target creature you control gets +0/+3 and gains hexproof until end of turn. (It can't be the target of spells or abilities your opponents control.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Deluge Virtuoso | {2}{U} | Creature — Human Wizard | 2/2 | When this creature enters, tap target creature an opponent controls and put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, this creature gets +2/+2 until end of turn instead. | ✅ | Push XXIX: ETB tap+stun wired (same shape as Fractal Mascot). Opus rider **now wired** via `shortcut::opus_trigger(+1/+1 EOT, +2/+2 EOT)`. Mana-spent introspection threads `Predicate::CastSpellManaSpentAtLeast(5)`. Test: `deluge_virtuoso_opus_pumps_one_one_or_two_two`. |
| Divergent Equation | {X}{X}{U} | Instant |  | Return up to X target instant and/or sorcery cards from your graveyard to your hand. / Exile Divergent Equation. | 🟡 | Wired in `catalog::sets::sos::instants` as a single-target return. The "up to X" multi-target prompt is collapsed to one target (no `Selector::OneOf` / count-bounded pick primitive yet — TODO.md). The "exile this" rider is omitted (no replay-prevention payoff). |
| Echocasting Symposium | {4}{U}{U} | Sorcery — Lesson |  | Target player creates a token that's a copy of target creature you control. / Paradigm (Then exile this spell. After you first resolve a spell with this name, you may cast a copy of it from exile without paying its mana cost at the beginning of each of your first main phases.) | 🟡 | Push XXVI: body-only wire — mints a 3/3 blue Wizard "Echocast" placeholder token via `Effect::CreateToken` (since the engine has no permanent-copy primitive yet). Lesson SpellSubtype tagged. Paradigm rider still ⏳ (cast-from-exile pipeline). |
| Emeritus of Ideation // Ancestral Recall | {3}{U}{U} // {U} | Creature — Human Wizard // Instant | 5/5 |  | 🟡 | Push XXVI: Front 5/5 Human Wizard with `Keyword::Ward(1)` (keyword tag). Back-face Ancestral Recall is wired as a {U} draw-3 instant. Was a ⏳ row blocked on Ward — now lands as 🟡 once Ward enforcement layer is added. |
| Encouraging Aviator // Jump | {2}{U} // {U} | Creature — Bird Wizard // Instant | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Exhibition Tidecaller | {U} | Creature — Djinn Wizard | 0/2 | Opus — Whenever you cast an instant or sorcery spell, target player mills three cards. If five or more mana was spent to cast that spell, that player mills ten cards instead. | ✅ | Push XXIX: Body + Opus rider wired via `shortcut::opus_trigger(Mill 3, Mill 10)`. The mill target uses `PlayerRef::Target(0)` so the auto-target picker hits an opponent. |
| Flow State | {1}{U} | Sorcery |  | Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order. If there is an instant card and a sorcery card in your graveyard, instead put two of… | 🟡 | Approximated as `Scry 3 + Draw 1`. Conditional "instead pick 2 to hand" gy-IS-pair upgrade rider is omitted (no "look-and-distribute-by-count" primitive). |
| Fractal Anomaly | {U} | Instant |  | Create a 0/0 green and blue Fractal creature token and put X +1/+1 counters on it, where X is the number of cards you've drawn this turn. | ✅ | Wired in `catalog::sets::sos::instants` using the engine's new `Selector::LastCreatedToken` + `Value::CardsDrawnThisTurn` primitives. X=0 → 0/0 token dies to SBA (matches printed). |
| Fractalize | {X}{U} | Instant |  | Until end of turn, target creature becomes a green and blue Fractal with base power and toughness each equal to X plus 1. (It loses all other colors and creature types.) | 🟡 | Collapsed to `PumpPT(+(X+1), +(X+1)) EOT` in `catalog::sets::sos::instants`. The "becomes a base-(X+1)/(X+1) Fractal" rewrite is omitted (no `Effect::ResetCreature` primitive); the printed creature-type loss + color rewrite would change tribal interactions but at typical X≥2 the buffed P/T plays correctly in combat. |
| Harmonized Trio // Brainstorm | {U} // {U} | Creature — Merfolk Bard Wizard // Instant | 1/1 |  | ✅ | Push XXVIII promotion: vanilla 1/1 Merfolk Bard Wizard front + faithful Brainstorm back (`Draw 3 + PutOnLibraryFromHand 2`). All Oracle clauses wired. Tests: `harmonized_trio_back_face_is_brainstorm`, `harmonized_trio_back_face_draws_three_then_puts_two_back`. |
| Homesickness | {4}{U}{U} | Instant |  | Target player draws two cards. Tap up to two target creatures. Put a stun counter on each of them. (If a permanent with a stun counter would become untapped, remove one from it instead.) | 🟡 | Wired in `catalog::sets::sos::instants` as `Draw 2 (you) → Tap target creature → Stun 1`. Multi-target prompt for both the draw player and the second creature is collapsed to single targets (caster draws + one creature stunned) — engine has no multi-target prompt for instants/sorceries (TODO.md). |
| Hydro-Channeler | {1}{U} | Creature — Merfolk Wizard | 1/3 | {T}: Add {U}. Spend this mana only to cast an instant or sorcery spell. / {1}, {T}: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell. | 🟡 | Wired in `catalog::sets::sos::creatures` with both mana abilities (`{T}: Add {U}` and `{1},{T}: Add one mana of any color`). The "spend this mana only to cast an instant or sorcery" restriction is omitted (no spend-restricted mana primitive — TODO.md). |
| Jadzi, Steward of Fate // Oracle's Gift | {2}{U} // {X}{X}{U} | Legendary Creature — Human Wizard // Sorcery | 2/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Landscape Painter // Vibrant Idea | {1}{U} // {4}{U} | Creature — Merfolk Wizard // Sorcery | 2/1 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Mana Sculpt | {1}{U}{U} | Instant |  | Counter target spell. If you control a Wizard, add an amount of {C} equal to the amount of mana spent to cast that spell at the beginning of your next main phase. | 🟡 | Wired in `catalog::sets::sos::instants` — counterspell + conditional `If(SelectorExists Wizard).then(AddMana(2 colorless))`. The "amount of mana spent on the countered spell" introspection is unavailable, so we approximate the rider as a flat +{C}{C}; the "delay-until-next-main" rider collapses to immediate add. |
| Mathemagics | {X}{X}{U}{U} | Sorcery |  | Target player draws 2ˣ cards. (2º = 1, 2¹ = 2, 2² = 4, 2³ = 8, 2⁴ = 16, 2⁵ = 32, and so on.) | ✅ | Wired in `catalog::sets::sos::sorceries` via the new `Value::Pow2(XFromCost)` primitive. Multi-target slot collapsed to "you" (caster draws); exponent capped at 30 to avoid deck-out. |
| Matterbending Mage | {2}{U} | Creature — Human Wizard | 2/2 | When this creature enters, return up to one other target creature to its owner's hand. / Whenever you cast a spell with {X} in its mana cost, this creature can't be blocked this turn. | ✅ | Push XVI: both abilities wired. ETB bounce stays as before; the X-cast trigger uses the new `Predicate::CastSpellHasX` + `Effect::GrantKeyword(Unblockable, EOT)` on `Selector::This`. |
| Muse Seeker | {1}{U} | Creature — Elf Wizard | 1/2 | Opus — Whenever you cast an instant or sorcery spell, draw a card. Then discard a card unless five or more mana was spent to cast that spell. | ✅ | Push XXIX: Body + Opus rider wired via `shortcut::opus_trigger`. Small body draws + discards; big body (≥5 mana) skips the discard. |
| Muse's Encouragement | {4}{U} | Instant |  | Create a 3/3 blue and red Elemental creature token with flying. / Surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Mints a 3/3 U/R Flying Elemental via the shared `elemental_token()` helper + `Effect::Surveil 2`. |
| Orysa, Tide Choreographer | {4}{U} | Legendary Creature — Merfolk Bard | 2/2 | This spell costs {3} less to cast if creatures you control have total toughness 10 or greater. / When Orysa enters, draw two cards. | 🟡 | ETB draw 2 wired faithfully. The conditional "{3} less if total toughness ≥ 10" alt-cost rider is omitted (alt-cost-with-board-state-predicate primitive). |
| Pensive Professor | {1}{U}{U} | Creature — Human Wizard | 0/2 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / Whenever one or more +1/+1 counters are put on this cr… | 🟡 | Push XXIX: Increment **now wired** via `shortcut::increment_self_plus_one()`. The secondary "+1/+1 counter is put on this creature, …" rider is still ⏳ pending re-fetch (oracle previously truncated). |
| Procrastinate | {X}{U} | Sorcery |  | Tap target creature. Put twice X stun counters on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::sorceries` with `Value::Times(2, XFromCost)`. |
| Run Behind | {3}{U} | Instant |  | This spell costs {1} less to cast if it targets an attacking creature. / Target creature's owner puts it on their choice of the top or bottom of their library. | 🟡 | Wired in `catalog::sets::sos::instants` — moves target creature to bottom of owner's library (conditional cost reduction omitted; "owner's choice top/bottom" collapsed to bottom-only since bottom is the typical removal outcome). |
| Skycoach Conductor // All Aboard | {2}{U} // {U} | Creature — Bird Pilot // Instant | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Spellbook Seeker // Careful Study | {3}{U} // {U} | Creature — Bird Wizard // Sorcery | 3/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Tester of the Tangential | {1}{U} | Creature — Djinn Wizard | 1/1 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / At the beginning of combat on your turn, you may pay {X}. When you do, move X +1/+1 counters from this creature onto another target creature. | 🟡 | Vanilla 1/1 body wired. Increment + combat-step pay-to-pump riders omitted. 🔍 needs review (oracle previously truncated). |
| Textbook Tabulator | {2}{U} | Creature — Frog Wizard | 0/3 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / When this creature enters, surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | 🟡 | Body wired (0/3 Frog Wizard) + ETB Surveil 2 via `Effect::Surveil`. Increment rider omitted. |
| Wisdom of Ages | {4}{U}{U}{U} | Sorcery |  | Return all instant and sorcery cards from your graveyard to your hand. You have no maximum hand size for the rest of the game. / Exile Wisdom of Ages. | 🟡 | Mass instant/sorcery recursion wired in `catalog::sets::sos::sorceries` via `Selector::CardsInZone` filter. The "no maximum hand size" rider and the "exile this" replacement are omitted. |

## Black

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Adventurous Eater // Have a Bite | {2}{B} // {B} | Creature — Human Warlock // Sorcery | 3/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Arcane Omens | {4}{B} | Sorcery |  | Converge — Target player discards X cards, where X is the number of colors of mana spent to cast this spell. | ✅ | Wired in `catalog::sets::sos::sorceries` via `Effect::Discard { amount: Value::ConvergedValue }` against `EachOpponent`. |
| Arnyn, Deathbloom Botanist | {2}{B} | Legendary Creature — Vampire Druid | 2/2 | Deathtouch / Whenever a creature you control with power or toughness 1 or less dies, target opponent loses 2 life and you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures` (deathtouch + `CreatureDied/AnotherOfYours` trigger gated by `Predicate::EntityMatches { what: TriggerSource, filter: PowerAtMost(1).or(ToughnessAtMost(1)) }`). |
| Burrog Banemaker | {B} | Creature — Frog Warlock | 1/1 | Deathtouch / {1}{B}: This creature gets +1/+1 until end of turn. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Cheerful Osteomancer // Raise Dead | {3}{B} // {B} | Creature — Orc Warlock // Sorcery | 4/2 |  | ✅ | Push XXVIII promotion: vanilla 4/2 Orc Warlock front + faithful Raise Dead back (`Move target creature card from graveyard → hand`). All Oracle clauses wired. Test: `cheerful_osteomancer_back_face_returns_creature_from_graveyard`. |
| Cost of Brilliance | {2}{B} | Sorcery |  | Target player draws two cards and loses 2 life. Put a +1/+1 counter on up to one target creature. | 🟡 | Wired in `catalog::sets::sos::sorceries` with the player target collapsed to "you" (you draw 2 + lose 2) and the +1/+1 counter on a single creature target. The 2-target prompt isn't expressible yet. |
| Decorum Dissertation | {3}{B}{B} | Sorcery — Lesson |  | Target player draws two cards and loses 2 life. / Paradigm (...) | 🟡 | Wired in `catalog::sets::sos::sorceries`. Mode 0 (you draw 2, lose 2 life) wired — collapses the "target player" prompt to the caster (engine has no multi-target prompt for sorceries). Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive). |
| Dissection Practice | {B} | Instant |  | Target opponent loses 1 life and you gain 1 life. / Up to one target creature gets +1/+1 until end of turn. / Up to one target creature gets -1/-1 until end of turn. | 🟡 | Wired in `catalog::sets::sos::instants` — drain 1 + creature target gets -1/-1 EOT. The optional creature +1/+1 mode is dropped (multi-target gap). |
| Emeritus of Woe // Demonic Tutor | {3}{B} // {1}{B} | Creature — Vampire Warlock // Sorcery | 5/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| End of the Hunt | {1}{B} | Sorcery |  | Target opponent exiles a creature or planeswalker they control with the greatest mana value among creatures and planeswalkers they control. | 🟡 | Wired in `catalog::sets::sos::sorceries` as a single-target Exile against `Creature ∨ Planeswalker & ControlledByOpponent`. The "greatest mana value" picker isn't enforced (auto-target picks first eligible). |
| Eternal Student | {3}{B} | Creature — Zombie Warlock | 4/2 | {1}{B}, Exile this card from your graveyard: Create two 1/1 white and black Inkling creature tokens with flying. | ✅ | Push XVII: graveyard-exile activation wired via the new `from_graveyard: bool` + `exile_self_cost: bool` fields. Cost `{1}{B}` + exile-self-as-cost + effect creates 2 Inkling tokens. |
| Foolish Fate | {2}{B} | Instant |  | Destroy target creature. / Infusion — If you gained life this turn, that creature's controller loses 3 life. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate. |
| Forum Necroscribe | {5}{B} | Creature — Troll Warlock | 5/4 | Ward—Discard a card. / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, return target creature card from your graveyard to the battlefield. | 🟡 | Wired in `catalog::sets::sos::creatures` (5/4 Troll Warlock body + Repartee gy-creature-recursion via the `repartee()` shortcut chained with `Effect::Move(target Creature → Battlefield(You))`). Ward—Discard a card omitted (no Ward keyword primitive yet — tracked in TODO.md). |
| Grave Researcher // Reanimate | {2}{B} // {B} | Creature — Troll Warlock // Sorcery | 3/3 |  | 🟡 | Push XXVI: Front 3/3 Troll Warlock with ETB Surveil 1 via `Effect::Surveil` (the "Surveil keyword primitive" gate was stale — Surveil is a first-class primitive). Back-face Reanimate at {B} returns one creature from your graveyard to the battlefield. The MV-equals-power life cost rider is omitted. |
| Lecturing Scornmage | {B} | Creature — Human Warlock | 1/1 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on this creature. | ✅ | Repartee +1/+1 counter via `effect::shortcut::repartee()`. |
| Leech Collector // Bloodletting | {1}{B} // {B} | Creature — Human Warlock // Sorcery | 2/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Masterful Flourish | {B} | Instant |  | Target creature you control gets +1/+0 and gains indestructible until end of turn. (Damage and effects that say "destroy" don't destroy it.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Melancholic Poet | {1}{B} | Creature — Elf Bard | 2/2 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, each opponent loses 1 life and you gain 1 life. | ✅ | Repartee drain 1 via `effect::shortcut::repartee()`. |
| Moseo, Vein's New Dean | {2}{B} | Legendary Creature — Bird Skeleton Warlock | 2/1 | Flying / When Moseo enters, create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." / Infusion — At the beginning of your end step, if you gained life this turn, return up… | 🟡 | Body + Flying + ETB Pest token wired faithfully (Pest's on-attack lifegain rider rides on the shared `pest_token()` helper). Infusion end-step rider omitted (oracle truncated; no MayDo per-turn-lifegain primitive). |
| Poisoner's Apprentice | {2}{B} | Creature — Orc Warlock | 2/2 | Infusion — When this creature enters, target creature an opponent controls gets -4/-4 until end of turn if you gained life this turn. | ✅ | Wired in `catalog::sets::sos::creatures` with the `LifeGainedThisTurnAtLeast` Infusion gate on the ETB trigger. |
| Postmortem Professor | {1}{B} | Creature — Zombie Warlock | 2/2 | This creature can't block. / Whenever this creature attacks, each opponent loses 1 life and you gain 1 life. / {1}{B}, Exile an instant or sorcery card from your graveyard: Return this card from your graveyard to the battlefield. | 🟡 | On-attack drain + the printed `Keyword::CantBlock` static restriction (now first-class via SOS push VI) wired in `catalog::sets::sos::creatures`. The graveyard-exile recursion activated ability is still omitted (engine activated-ability walker only iterates the battlefield — TODO.md "Activated-Ability `From Your Graveyard` Path"). |
| Pox Plague | {B}{B}{B}{B}{B} | Sorcery |  | Each player loses half their life, then discards half the cards in their hand, then sacrifices half the permanents they control of their choice. Round down each time. | ✅ | Wired in `catalog::sets::sos::sorceries` via `ForEach Player(EachPlayer)` body using the new `Value::HalfDown` + `Value::PermanentCountControlledBy(Triggerer)` primitives. Half-life / half-hand / half-permanents per player. |
| Pull from the Grave | {2}{B} | Sorcery |  | Return up to two target creature cards from your graveyard to your hand. You gain 2 life. | 🟡 | Returns up to **two** creature cards from your graveyard via the new `Selector::Take(_, 2)` primitive (push X) — strictly closer to the printed "up to two" cap than the prior single-card return. Multi-target prompt for the printed "target creature card" slots is still collapsed (no UI multi-target picker yet); the auto-decider picks the top two matching creature cards in graveyard order. Lifegain half resolves regardless. |
| Rabid Attack | {1}{B} | Instant |  | Until end of turn, any number of target creatures you control each get +1/+0 and gain "When this creature dies, draw a card." | 🟡 | Wired in `catalog::sets::sos::instants` as a +1/+0 EOT pump on a single target. The "any number of target creatures" multi-target prompt and the transient die-to-draw triggered ability grant are both omitted (engine TODOs). |
| Ral Zarek, Guest Lecturer | {1}{B}{B} | Legendary Planeswalker — Ral | [3] | +1: Surveil 2. / −1: Any number of target players each discard a card. / −2: Return target creature card with mana value 3 or less from your graveyard to the battlefield. / −7: Flip five coins. Target opponent skips their next X turns, where X is the number of coins that came up heads. | 🟡 | +1 Surveil 2 / -1 each opp discards 1 (single-target collapse) / -2 return ≤3-MV creature card from your gy → bf. -7 coin-flip / skip-turns ult omitted (no coin-flip + no skip-turns primitive). |
| Scathing Shadelock // Venomous Words | {4}{B} // {B} | Creature — Snake Warlock // Sorcery | 4/6 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Scheming Silvertongue // Sign in Blood | {1}{B} // {B}{B} | Creature — Vampire Warlock // Sorcery | 1/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Send in the Pest | {1}{B} | Sorcery |  | Each opponent discards a card. You create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." | ✅ | Discard + Pest token wired; the token's "gain 1 on attack" rider now fires (via the new `TokenDefinition.triggered_abilities` field). |
| Sneering Shadewriter | {4}{B} | Creature — Vampire Warlock | 3/3 | Flying / When this creature enters, each opponent loses 2 life and you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Tragedy Feaster | {2}{B}{B} | Creature — Demon | 7/6 | Trample / Ward—Discard a card. / Infusion — At the beginning of your end step, sacrifice a permanent unless you gained life this turn. | 🟡 | Body-only wire in `catalog::sets::sos::creatures` (7/6 Trample Demon). Ward—Discard rider and Infusion upkeep-sac-unless-lifegain rider both omitted (no Ward keyword primitive; no `MayDo` / per-turn-lifegain-sac primitive). The vanilla shell is still useful as a 4-mana finisher. |
| Ulna Alley Shopkeep | {2}{B} | Creature — Goblin Warlock | 2/3 | Menace (This creature can't be blocked except by two or more creatures.) / Infusion — This creature gets +2/+0 as long as you gained life this turn. | 🟡 | Body-only wire (2/3 Menace Goblin Warlock). The Infusion static "+2/+0 while you've gained life this turn" rider is omitted (no continuous-static-on-predicate primitive). |
| Wander Off | {3}{B} | Instant |  | Exile target creature. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Withering Curse | {1}{B}{B} | Sorcery |  | All creatures get -2/-2 until end of turn. / Infusion — If you gained life this turn, destroy all creatures instead. | ✅ | `If LifeGainedThisTurnAtLeast(1)` branch: Infusion-path = ForEach(Creature) Destroy; mainline = ForEach(Creature) PumpPT(-2/-2 EOT). |

## Red

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Archaic's Agony | {4}{R} | Sorcery |  | Converge — Archaic's Agony deals X damage to target creature, where X is the number of colors of mana spent to cast this spell. Exile cards from the top of your library equal to the excess damage dealt to that creature this way. You may play those cards until the end of your next turn. | 🟡 | Push XXVIII: ⏳ → 🟡. Body wired — Converge X damage to a creature target via `Value::ConvergedValue` (same primitive as Rancorous Archaic / Sundering Archaic). The "exile cards equal to excess damage + may play until next end step" rider is still omitted (cast-from-exile pipeline + "exile N for excess damage" primitive both missing). At converge 5 this is a 5-damage burn spell for {4}{R}. |
| Artistic Process | {3}{R}{R} | Sorcery |  | Choose one — / • Artistic Process deals 6 damage to target creature. / • Artistic Process deals 2 damage to each creature you don't control. / • Create a 3/3 blue and red Elemental creature token with flying. It gains haste until end of turn. | ✅ | Wired in `catalog::sets::sos::sorceries`. All three modes wired: 6-to-creature, 2-to-each-opp-creature (via `Selector::EachPermanent(Creature & ControlledByOpponent)`), Elemental token + transient haste via `Selector::LastCreatedToken`. |
| Blazing Firesinger // Seething Song | {2}{R} // {2}{R} | Creature — Dwarf Bard // Instant | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Charging Strifeknight | {2}{R} | Creature — Spirit Knight | 3/3 | Haste / {T}, Discard a card: Draw a card. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Choreographed Sparks | {R}{R} | Instant |  | This spell can't be copied. / Choose one or both — / • Copy target instant or sorcery spell you control. You may choose new targets for the copy. / • Copy target creature spell you control. The copy gains haste and "At the beginning of the end step, sacrifice this token." | 🟡 | Push XXVI: Single-mode wire via `Effect::CopySpell` against an IS-on-stack target (the "or copy a creature spell" branch needs a permanent-spell copy variant). The "this spell can't be copied" rider is omitted (no `CantBeCopied` keyword tag yet). |
| Duel Tactics | {R} | Sorcery |  | Duel Tactics deals 1 damage to target creature. It can't block this turn. / Flashback {1}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired as `DealDamage(1) + GrantKeyword(CantBlock, EOT)` — pulls in the new `Keyword::CantBlock` (enforced inside `declare_blockers` and the `can_block_*` helpers). Flashback {1}{R} now wired via `Keyword::Flashback` (push X). |
| Emeritus of Conflict // Lightning Bolt | {1}{R} // {R} | Creature — Human Wizard // Instant | 2/2 |  | ✅ | Push XXVIII promotion: vanilla 2/2 Human Wizard front + faithful Lightning Bolt back (`DealDamage 3 to target`). All Oracle clauses wired. Test: `emeritus_of_conflict_back_face_burns_three`. |
| Expressive Firedancer | {1}{R} | Creature — Human Sorcerer | 2/2 | Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, this creature also gains double strike until end of turn. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: +1/+1 EOT. Big body (≥5 mana): +1/+1 EOT + DoubleStrike EOT. Test: `expressive_firedancer_opus_grants_double_strike_at_five_mana`. |
| Flashback | {R} | Instant |  | Target instant or sorcery card in your graveyard gains flashback until end of turn. The flashback cost is equal to its mana cost. (You may cast that card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Push XXVI: Approximated as a {R} "return a target IS card from your graveyard to your hand" instant — the player can re-cast it next turn at its normal cost. Strictly weaker than the printed "flashback for its mana cost this turn" but preserves the recovery outcome. A true wiring needs a transient per-instance grant on a graveyard card. |
| Garrison Excavator | {3}{R} | Creature — Orc Sorcerer | 3/4 | Menace (This creature can't be blocked except by two or more creatures.) / Whenever one or more cards leave your graveyard, create a 2/2 red and white Spirit creature token. | ✅ | Wired against the new `EventKind::CardLeftGraveyard` event — every gy-leave mints a 2/2 R/W Spirit token via the shared `spirit_token()` helper. |
| Goblin Glasswright // Craft with Pride | {1}{R} // {R} | Creature — Goblin Sorcerer // Sorcery | 2/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Heated Argument | {4}{R} | Instant |  | Heated Argument deals 6 damage to target creature. You may exile a card from your graveyard. If you do, Heated Argument also deals 2 damage to that creature's controller. | ✅ | Push XV → ✅ in push XXVIII: 6-to-creature is unconditional; the gy-exile + 2-to-controller chain is wrapped in `Effect::MayDo` and either both fire or both skip — faithful to the printed "you may". Uses `Selector::take(CardsInZone(GY), 1)` to pick exactly one gy card (matching "a card", not "every card"). |
| Impractical Joke | {R} | Sorcery |  | Damage can't be prevented this turn. Impractical Joke deals 3 damage to up to one target creature or planeswalker. | 🟡 | 3-to-creature/PW wired; "damage can't be prevented" rider is a no-op (engine has no damage-prevention layer). |
| Improvisation Capstone | {5}{R}{R} | Sorcery — Lesson |  | Exile cards from the top of your library until you exile cards with total mana value 4 or greater. You may cast any number of spells from among them without paying their mana costs. / Paradigm (Then exile this spell. After you first resolve a spell with this name, you may cast a copy of it from exile without paying its mana cost at the beginning of each of your first main phases.) | ⏳ | 🔍 needs review (oracle previously truncated). Needs: copy-spell/permanent primitive; cast-from-exile pipeline. |
| Living History | {1}{R} | Enchantment |  | When this enchantment enters, create a 2/2 red and white Spirit creature token. / Whenever you attack, if a card left your graveyard this turn, target attacking creature gets +2/+0 until end of turn. | 🟡 | ETB Spirit token + on-attack +2/+0 EOT (gated on the new `Predicate::CardsLeftGraveyardThisTurnAtLeast`). The "target attacking creature" picks the trigger source (the just-declared attacker) rather than a fresh target — collapsed for the per-attacker auto-target framework. |
| Maelstrom Artisan // Rocket Volley | {1}{R}{R} // {1}{R} | Creature — Minotaur Sorcerer // Sorcery | 3/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Magmablood Archaic | {2/R}{2/R}{2/R} | Creature — Avatar | 2/2 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. / Whenever you cast an instant or sorcery spell, creatures you control get +1/+0 until end of turn for each color of mana spent to cast that spell. | 🟡 | Body wired in `catalog::sets::sos::creatures` (2/2 Avatar with Trample+Reach + Converge ETB AddCounter using `Value::ConvergedValue`). The IS-cast pump rider is omitted pending per-cast converge introspection on the *just-cast* spell (the trigger fires but reads the Archaic's own ETB converge value, not the iterated cast's). Hybrid `{2/R}` pips approximated as `{2}+{R}` per pip. |
| Mica, Reader of Ruins | {3}{R} | Legendary Creature — Human Artificer | 4/4 | Ward—Pay 3 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 3 life.) / Whenever you cast an instant or sorcery spell, you may sacrifice an artifact. If you do, copy that spell and you may choose new targets for the copy. | 🟡 | Push XXV: Body wired (4/4 Legendary Human Artificer). Magecraft sac-artifact-to-copy rider wired via `magecraft(MayDo + Seq(Sacrifice(Artifact, 1) + CopySpell{what: TriggerSource}))` — same template as Aziza, Mage Tower Captain. Ward—Pay 3 life tagged via `Keyword::Ward(3)`; ward enforcement still pending. |
| Molten-Core Maestro | {1}{R} | Creature — Goblin Bard | 2/2 | Menace / Opus — Whenever you cast an instant or sorcery spell, put a +1/+1 counter on this creature. If five or more mana was spent to cast that spell, add an amount of {R} equal to this creature's power. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: +1/+1 counter on this creature. Big body (≥5 mana): counter + add {R}×power via `ManaPayload::OfColor(Red, PowerOf(This))`. |
| Pigment Wrangler // Striking Palette | {4}{R} // {R} | Creature — Orc Sorcerer // Sorcery | 4/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Rearing Embermare | {4}{R} | Creature — Horse Beast | 4/5 | Reach, haste | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Rubble Rouser | {2}{R} | Creature — Dwarf Sorcerer | 1/4 | When this creature enters, you may discard a card. If you do, draw a card. / {T}, Exile a card from your graveyard: Add {R}. When you do, this creature deals 1 damage to each opponent. | 🟡 | Push XV: ETB rummage now wrapped in `Effect::MayDo` so the "you may discard" optionality is honored. The `{T}, Exile a card from your graveyard:` activated ability is still omitted (engine activated-ability path has no `from-your-graveyard` cost variant — separate from `sac_cost`). |
| Steal the Show | {2}{R} | Sorcery |  | Choose one or both — / • Target player discards any number of cards, then draws that many cards. / • Steal the Show deals damage equal to the number of instant and sorcery cards in your graveyard to target creature or planeswalker. | 🟡 | Modal sorcery: mode 0 (target player discards N then draws N — collapsed to "discard 2, draw 2" since the engine has no "any number" prompt for the targeted player); mode 1 deals damage = `Value::CountOf(CardsInZone(your graveyard, IS-cards))` to a creature/PW. The "choose one or both" rider collapses to "pick one mode" (no multi-mode-pick primitive yet). |
| Strife Scholar // Awaken the Ages | {2}{R} // {5}{R} | Creature — Orc Sorcerer // Sorcery | 3/2 |  | 🟡 | Push XXVI: Front 3/2 Orc Sorcerer with `Keyword::Ward(1)` (keyword tag). Back-face Awaken the Ages at {5}{R} returns all creature cards from your graveyard to the battlefield via `Selector::CardsInZone(Graveyard, Creature)`. The "sacrifice this spell" rider is a no-op (sorceries hit graveyard on resolve anyway). |
| Tablet of Discovery | {2}{R} | Artifact |  | When this artifact enters, mill a card. You may play that card this turn. (To mill a card, put the top card of your library into your graveyard.) / {T}: Add {R}. / {T}: Add {R}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::artifacts` — ETB Mill 1 + two `{T}: Add {R}` mana abilities. The "may play that card this turn" mill-rider is omitted (no per-card may-play primitive yet). The spend-restriction on the {T}: Add {R}{R} ability is omitted (no spend-restricted mana primitive). |
| Tackle Artist | {3}{R} | Creature — Orc Sorcerer | 4/3 | Trample / Opus — Whenever you cast an instant or sorcery spell, put a +1/+1 counter on this creature. If five or more mana was spent to cast that spell, put two +1/+1 counters on this creature instead. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: one +1/+1 counter. Big body (≥5 mana): two +1/+1 counters instead. Tests: `tackle_artist_opus_lands_*_counter*`. |
| Thunderdrum Soloist | {1}{R} | Creature — Dwarf Bard | 1/3 | Reach / Opus — Whenever you cast an instant or sorcery spell, this creature deals 1 damage to each opponent. If five or more mana was spent to cast that spell, this creature deals 3 damage to each opponent instead. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: 1 dmg to each opp. Big body (≥5 mana): 3 dmg to each opp. Test: `thunderdrum_soloist_opus_pings_one_at_small_three_at_big`. |
| Tome Blast | {1}{R} | Sorcery |  | Tome Blast deals 2 damage to any target. / Flashback {4}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired as a 2-to-any-target burn spell. Flashback {4}{R} now wired via `Keyword::Flashback` (push X). |
| Unsubtle Mockery | {2}{R} | Instant |  | Unsubtle Mockery deals 4 damage to target creature. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | `DealDamage(4) + Surveil 1` via `Effect::Surveil`. |
| Zealous Lorecaster | {5}{R} | Creature — Giant Sorcerer | 4/4 | When this creature enters, return target instant or sorcery card from your graveyard to your hand. | ✅ | Wired in `catalog::sets::sos::creatures` with a Move-target-from-graveyard ETB trigger. |

## Green

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Aberrant Manawurm | {3}{G} | Creature — Wurm | 2/5 | Trample / Whenever you cast an instant or sorcery spell, this creature gets +X/+0 until end of turn, where X is the amount of mana spent to cast that spell. | ✅ | Push XXIX: Magecraft-style trigger **now wired** via `shortcut::magecraft(PumpPT{power: CastSpellManaSpent})`. The pump reads the just-cast spell's mana_spent so a 5-mana spell gives the wurm +5/+0 EOT. Test: `aberrant_manawurm_pumps_by_mana_spent_eot`. |
| Additive Evolution | {3}{G}{G} | Enchantment |  | When this enchantment enters, create a 0/0 green and blue Fractal creature token. Put three +1/+1 counters on it. / At the beginning of combat on your turn, put a +1/+1 counter on target creature you control. It gains vigilance until end of turn. | ✅ | Wired in `catalog::sets::sos::enchantments`. ETB Fractal-with-3-counters via the existing `fractal_token()` helper + `Selector::LastCreatedToken` AddCounter. Begin-combat +1/+1 counter + Vigilance (EOT) on a friendly creature, gated through the active-player StepBegins(BeginCombat) trigger. |
| Ambitious Augmenter | {G} | Creature — Turtle Wizard | 1/1 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / When this creature dies, if it had one or more counters on it, create a 0/0 green and blue Fractal creature token, then put this creature's counters on that token. | 🟡 | Body-only wire in `catalog::sets::sos::creatures` (1/1 Turtle Wizard at {G}). Increment pump omitted (mana-spent-on-cast introspection missing — tracked in TODO.md). The death-with-counters → Fractal-with-counters trigger is also omitted pending a counter-transfer-on-death primitive. |
| Burrog Barrage | {1}{G} | Instant |  | Target creature you control gets +1/+0 until end of turn if you've cast another instant or sorcery spell this turn. Then it deals damage equal to its power to up to one target creature an opponent controls. | 🟡 | Wired in `catalog::sets::sos::instants` — conditional pump (gated on the new `Predicate::SpellsCastThisTurnAtLeast(2)`) + power-as-damage to the chosen target. The 2-target prompt for the opp-creature defender is collapsed (single-target spell), so the spell ends up dealing self-damage rather than hitting an opp creature. Tracked in TODO.md. |
| Chelonian Tackle | {2}{G} | Sorcery |  | Target creature you control gets +0/+10 until end of turn. Then it fights up to one target creature an opponent controls. (Each deals damage equal to its power to the other.) | 🟡 | +0/+10 EOT pump + the new `Effect::Fight` against an auto-selected opp creature (no multi-target prompt for the defender slot). Fight no-ops cleanly when no opp creature is on the battlefield, preserving the printed "up to one" semantics. |
| Comforting Counsel | {1}{G} | Enchantment |  | Whenever you gain life, put a growth counter on this enchantment. / As long as there are five or more growth counters on this enchantment, creatures you control get +3/+3. | 🟡 | Lifegain → Growth counter trigger wired in `catalog::sets::sos::enchantments`. The "≥5 counters → anthem" static is omitted (no self-counter-gated `StaticEffect` primitive). |
| Efflorescence | {2}{G} | Instant |  | Put two +1/+1 counters on target creature. / Infusion — If you gained life this turn, that creature also gains trample and indestructible until end of turn. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate. |
| Emeritus of Abundance // Regrowth | {2}{G} // {1}{G} | Creature — Elf Druid // Sorcery | 3/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Emil, Vastlands Roamer | {2}{G} | Legendary Creature — Elf Druid | 3/3 | Creatures you control with +1/+1 counters on them have trample. / {4}{G}, {T}: Create a 0/0 green and blue Fractal creature token. Put X +1/+1 counters on it, where X is the number of differently named lands you control. | ✅ | Wired in `catalog::sets::sos::creatures` — `StaticEffect::GrantKeyword(Trample)` filtered to creatures with +1/+1 counters via the new `AffectedPermanents::AllWithCounter` layer variant; activated `{4}{G},{T}` creates a Fractal + counters scaled to land count. "Differently named" filter on X is collapsed to total land count (typical cube games have unique land slots). |
| Environmental Scientist | {1}{G} | Creature — Human Druid | 2/2 | When this creature enters, you may search your library for a basic land card, reveal it, put it into your hand, then shuffle. | ✅ | Wired with `Effect::Search` over `IsBasicLand`. |
| Follow the Lumarets | {1}{G} | Sorcery |  | Infusion — Look at the top four cards of your library. You may reveal a creature or land card from among them and put it into your hand. If you gained life this turn, you may instead reveal two creature and/or land cards from among them and put them into your hand. Put the rest on the bottom of your library in a random order. | 🟡 | Push XV: wired as `If(LifeGainedThisTurnAtLeast(1)) → 2× RevealUntilFind(cap=4) → Hand : 1× RevealUntilFind(cap=4) → Hand`. Find filter = Creature OR Land. Approximations: misses go to graveyard (not bottom of library) — `RevealUntilFind`'s engine default; "you may reveal" optionality collapsed to always-do (declining would mill 4, strictly worse). |
| Germination Practicum | {3}{G}{G} | Sorcery — Lesson |  | Put two +1/+1 counters on each creature you control. / Paradigm (...) | 🟡 | Wired in `catalog::sets::sos::sorceries` as a `ForEach Creature & ControlledByYou → AddCounter +1/+1 ×2` fan-out. Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive). |
| Glorious Decay | {1}{G} | Instant |  | Choose one — / • Destroy target artifact. / • Glorious Decay deals 4 damage to target creature with flying. / • Exile target card from a graveyard. Draw a card. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Hungry Graffalon | {3}{G} | Creature — Giraffe | 3/4 | Reach / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) | ✅ | Push XXIX: Increment **now wired** via `shortcut::increment_self_plus_one()`. Cast a 5-mana spell with the Giraffe out → lands a +1/+1 counter (5 > 4 toughness). Tests: `hungry_graffalon_increment_*`. |
| Infirmary Healer // Stream of Life | {1}{G} // {X}{G} | Creature — Cat Cleric // Sorcery | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Lumaret's Favor | {1}{G} | Instant |  | Infusion — When you cast this spell, copy it if you gained life this turn. You may choose new targets for the copy. / Target creature gets +2/+4 until end of turn. | ✅ | Push XVII: Infusion copy now wired via the new `Effect::CopySpell` primitive. The SelfSource cast trigger is gated by `Predicate::LifeGainedThisTurnAtLeast(You, 1)` — when you gained life this turn, the trigger fires and copies the spell. The trigger's filter is evaluated at trigger-creation time (cast-time), matching MTG's "intervening if" wording. |
| Mindful Biomancer | {1}{G} | Creature — Dryad Druid | 2/2 | When this creature enters, you gain 1 life. / {2}{G}: This creature gets +2/+2 until end of turn. Activate only once each turn. | ✅ | Wired in `catalog::sets::sos::creatures`; once-per-turn enforced engine-side. |
| Noxious Newt | {1}{G} | Creature — Salamander | 1/2 | Deathtouch / {T}: Add {G}. | ✅ | Wired in `catalog::sets::sos::creatures`. Now uses the new `Salamander` creature subtype. |
| Oracle's Restoration | {G} | Sorcery |  | Target creature you control gets +1/+1 until end of turn. You draw a card and gain 1 life. | ✅ | Wired in `catalog::sets::sos::sorceries`. |
| Pestbrood Sloth | {3}{G} | Creature — Plant Sloth | 4/4 | Reach / When this creature dies, create two 1/1 black and green Pest creature tokens with "Whenever this token attacks, you gain 1 life." | ✅ | Death trigger creates two Pests; each token now ships with the "gain 1 on attack" rider (via the new `TokenDefinition.triggered_abilities` field). |
| Planar Engineering | {3}{G} | Sorcery |  | Sacrifice two lands. Search your library for four basic land cards, put them onto the battlefield tapped, then shuffle. | ✅ | Wired in `catalog::sets::sos::sorceries` — `Sacrifice 2 lands` + `Repeat × 4 Search { IsBasicLand → Battlefield(tapped) }`. |
| Shopkeeper's Bane | {2}{G} | Creature — Badger Pest | 4/2 | Trample / Whenever this creature attacks, you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures` with the new `Badger` creature subtype. |
| Slumbering Trudge | {X}{G} | Creature — Plant Beast | 6/6 | This creature enters with a number of stun counters on it equal to three minus X. If X is 2 or less, it enters tapped. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::creatures` using `Value::NonNeg(3-X)` stun counters. The "enters tapped if X≤2" half is implemented as always-tap-on-ETB; for X≥3 the no-stun-counter trudge naturally untaps next turn. |
| Snarl Song | {5}{G} | Sorcery |  | Converge — Create two 0/0 green and blue Fractal creature tokens. Put X +1/+1 counters on each of them and you gain X life, where X is the number of colors of mana spent to cast this spell. | ✅ | Wired in `catalog::sets::sos::sorceries`: two `CreateToken` calls each followed by `AddCounter(LastCreatedToken, +1/+1, ConvergedValue)`, plus `GainLife(ConvergedValue)`. Powered by `Value::ConvergedValue` (Rancorous Archaic) and `Selector::LastCreatedToken` (Fractal Anomaly). |
| Studious First-Year // Rampant Growth | {G} // {1}{G} | Creature — Bear Wizard // Sorcery | 1/1 | Front: vanilla 1/1 Bear Wizard. Back: search your library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | First non-land MDFC. Front face is wired as a vanilla 1/1 Bear Wizard at `{G}`; back face is `Rampant Growth`. Cast either face via `GameAction::CastSpell` (front) or the new `GameAction::CastSpellBack` (back, added in push X — mirror to `PlayLandBack` but for non-land back faces). The engine's `cast_spell_back_face` helper swaps the in-hand `definition` to the back face's before validating cost / type / effect, so the printed back-face Sorcery resolves end-to-end. |
| Tenured Concocter | {4}{G} | Creature — Troll Druid | 4/5 | Vigilance / Whenever this creature becomes the target of a spell or ability an opponent controls, you may draw a card. / Infusion — This creature gets +2/+0 as long as you gained life this turn. | 🟡 | Vanilla 4/5 vigilance body wired in `catalog::sets::sos::creatures`. The "becomes the target" trigger is omitted (no `BecameTarget` event). The Infusion static pump is omitted (no continuous-static-on-predicate primitive). |
| Thornfist Striker | {2}{G} | Creature — Elf Druid | 3/3 | Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.) / Infusion — Creatures you control get +1/+0 and have trample as long as you gained life this turn. | 🟡 | Body + `Keyword::Ward(1)` wired in `catalog::sets::sos::creatures`. Infusion continuous static (creatures you control get +1/+0 + trample while you gained life this turn) is omitted (no continuous-static-on-predicate primitive). |
| Topiary Lecturer | {2}{G} | Creature — Elf Druid | 1/2 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / {T}: Add an amount of {G} equal to this creature's power. | 🟡 | Wired with the new `ManaPayload::OfColor(Green, PowerOf(This))` primitive — fixed color, value-scaled count, so the {T}: Add G mana ability cleanly tracks `power-many G pips`. The Increment rider is omitted (mana-spent introspection on cast). |
| Vastlands Scavenger // Bind to Life | {1}{G}{G} // {4}{G} | Creature — Bear Druid // Instant | 4/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Wild Hypothesis | {X}{G} | Sorcery |  | Create a 0/0 green and blue Fractal creature token. Put X +1/+1 counters on it. / Surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Wired in `catalog::sets::sos::sorceries`: `CreateToken(fractal) + AddCounter(LastCreatedToken, +1/+1, XFromCost) + Surveil 2`. `Effect::Surveil` is a first-class primitive so this resolves end-to-end with no approximation. |
| Wildgrowth Archaic | {2/G}{2/G} | Creature — Avatar | 0/0 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. / Whenever you cast a creature spell, that creature enters with X additional +1/+1 counters on it, where X is the number of colors of mana spent to cast it. | 🟡 | Body wired in `catalog::sets::sos::creatures` (0/0 Avatar with Trample+Reach + Converge ETB AddCounter via `Value::ConvergedValue`). Hybrid `{2/G}` pips approximated as `{2}+{G}` per pip. The "creature spells you cast enter with X extra counters" rider is omitted pending per-cast converge introspection on the *just-cast* creature spell. Mono-G casts will die immediately to SBA (printed 0/0); 2-color casts land as 2/2. |
| Zimone's Experiment | {3}{G} | Sorcery |  | Look at the top five cards of your library. You may reveal up to two creature and/or land cards from among them, then put the rest on the bottom of your library in a random order. Put all land cards revealed this way onto the battlefield tapped and put all creature cards revealed this way into your hand. | 🟡 | Approximated as `RevealUntilFind { find: Creature, cap: 5, → Hand }` followed by a `Search { filter: IsBasicLand, → Battlefield(tapped) }`. The "look at top 5, choose ≤2 matching from among them" two-destination split isn't expressible (no "look + sort by category" primitive yet); the approximation pulls one creature into hand and one basic into play, which is the typical play pattern. |

## Prismari (Blue-Red)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abstract Paintmage | {U}{U/R}{R} | Creature — Djinn Sorcerer | 2/2 | At the beginning of your first main phase, add {U}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::creatures` with a `StepBegins(PreCombatMain)/ActivePlayer` trigger that adds {U}{R} via `ManaPayload::Colors`. The spend restriction is omitted (no per-pip mana metadata). The hybrid `{U/R}` pip is approximated as `{U}`. |
| Colorstorm Stallion | {1}{U}{R} | Creature — Elemental Horse | 3/3 | Ward {1}, haste / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, create a token that's a copy of this creature. | 🟡 | Push XXIX: Body + Opus small body (+1/+1 EOT) wired via `shortcut::opus_trigger`. Big body (≥5 mana) "copy this creature" collapses to the same +1/+1 EOT pump pending a permanent-copy primitive (distinct from `Effect::CopySpell`). Test: `colorstorm_stallion_opus_small_body_pumps_one_one_eot`. |
| Elemental Mascot | {1}{U}{R} | Creature — Elemental Bird | 1/4 | Flying, vigilance / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn. If five or more mana was spent to cast that spell, exile the top card of your library. You may play that card until the end of your next turn. | 🟡 | Push XXIX: Body + Opus small body (+1/+0 EOT) wired via `shortcut::opus_trigger`. Big body (≥5 mana) "exile top + may play" collapses to the same +1/+0 EOT pump pending a cast-from-exile-with-timer primitive. Test: `elemental_mascot_opus_small_body_pumps_one_zero_eot`. |
| Prismari Charm | {U}{R} | Instant |  | Choose one — / • Surveil 2, then draw a card. / • Prismari Charm deals 1 damage to each of one or two targets. / • Return target nonland permanent to its owner's hand. | ✅ | 3-mode `ChooseMode`: Surveil 2 + draw / 1 dmg to creature-or-PW / bounce nonland to owner. Single-target collapse on mode 1 (printed "one or two targets" — multi-target gap). |
| Prismari, the Inspiration | {5}{U}{R} | Legendary Creature — Elder Dragon | 7/7 | Flying / Ward—Pay 5 life. / Instant and sorcery spells you cast have storm. (Whenever you cast an instant or sorcery spell, copy it for each spell cast before it this turn. You may choose new targets for the copies.) | 🟡 | Push XXVI: Body wired (7/7 Flying Legendary Elder Dragon with `Keyword::Ward(5)`). The "your IS spells have storm" static is omitted — storm grants need a per-cast trigger that fans out copies for each prior spell cast this turn. |
| Rapturous Moment | {4}{U}{R} | Sorcery |  | Draw three cards, then discard two cards. Add {U}{U}{R}{R}{R}. | ✅ | Wired in `catalog::sets::sos::sorceries`: Draw 3 + Discard 2 + AddMana with the printed UU/RRR pool. |
| Resonating Lute | {2}{U}{R} | Artifact |  | Lands you control have "{T}: Add two mana of any one color. Spend this mana only to cast instant and sorcery spells." / {T}: Draw a card. Activate only if you have seven or more cards in your hand. | 🟡 | Wired in `catalog::sets::sos::artifacts`. The {T}: Draw activation uses the new `ActivatedAbility.condition: Predicate::ValueAtLeast(HandSizeOf(You), 7)` gate — the engine rejects the activation cleanly when hand size < 7. Lands-grant tap-for-2 static is omitted (no spend-restricted-mana primitive yet — tracked in TODO.md). |
| Sanar, Unfinished Genius // Wild Idea | {U}{R} // {3}{U}{R} | Legendary Creature — Goblin Sorcerer // Sorcery | 0/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Spectacle Summit |  | Land |  | This land enters tapped. / {T}: Add {U} or {R}. / {2}{U}{R}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands` via the shared `school_land` builder. |
| Spectacular Skywhale | {2}{U}{R} | Creature — Elemental Whale | 1/4 | Flying / Opus — Whenever you cast an instant or sorcery spell, this creature gets +3/+0 until end of turn. If five or more mana was spent to cast that spell, put three +1/+1 counters on this creature instead. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: +3/+0 EOT on the Skywhale. Big body (≥5 mana): three +1/+1 counters instead. Tests: `spectacular_skywhale_opus_small_body_pumps_three_zero_eot`, `spectacular_skywhale_opus_big_body_adds_three_counters`. |
| Splatter Technique | {1}{U}{U}{R}{R} | Sorcery |  | Choose one — / • Draw four cards. / • Splatter Technique deals 4 damage to each creature and planeswalker. | ✅ | Wired in `catalog::sets::sos::sorceries` as a `ChooseMode` with Draw 4 (mode 0) and DealDamage to `EachPermanent(Creature ∪ Planeswalker)` (mode 1). |
| Stadium Tidalmage | {2}{U}{R} | Creature — Djinn Sorcerer | 4/4 | Whenever this creature enters or attacks, you may draw a card. If you do, discard a card. | ✅ | Push XV → ✅ in push XXVIII: ETB + Attacks loot triggers use the `Effect::MayDo` primitive faithfully. The "you may" prompt asks the controller via `Decision::OptionalTrigger` — `AutoDecider` says no, `ScriptedDecider::new([Bool(true)])` for tests. Both Oracle clauses (ETB + attack) are fully wired. |
| Stress Dream | {3}{U}{R} | Instant |  | Stress Dream deals 5 damage to up to one target creature. Look at the top two cards of your library. Put one of those cards into your hand and the other on the bottom of your library. | 🟡 | 5-damage half wired in `catalog::sets::sos::instants`; the "look at top 2, choose 1 to hand and other to bottom" half is approximated as `scry 1 + draw 1` (no choose-which-zone primitive). |
| Traumatic Critique | {X}{U}{R} | Instant |  | Traumatic Critique deals X damage to any target. Draw two cards, then discard a card. | ✅ | Wired in `catalog::sets::sos::instants` (X damage via `Value::XFromCost` + draw 2 + discard 1 loot tail). |
| Vibrant Outburst | {U}{R} | Instant |  | Vibrant Outburst deals 3 damage to any target. Tap up to one target creature. | 🟡 | 3-damage half wired in `catalog::sets::sos::instants` against any target; the optional second creature target (tap half) is omitted (multi-target prompt gap). |
| Visionary's Dance | {5}{U}{R} | Sorcery |  | Create two 3/3 blue and red Elemental creature tokens with flying. / {2}, Discard this card: Look at the top two cards of your library. Put one of them into your hand and the other into your graveyard. | ✅ | Wired in `catalog::sets::sos::sorceries` (uses the new `elemental_token()` helper). The `{2}, Discard this card:` activation from hand is omitted (engine activation walker is battlefield-only). |
| Zaffai and the Tempests | {5}{U}{R} | Legendary Creature — Human Bard Sorcerer | 5/7 | Once during each of your turns, you may cast an instant or sorcery spell from your hand without paying its mana cost. | 🟡 | Push XVI: body-only wire (5/7 Legendary Human Bard Sorcerer). The "once per turn cast IS for free" rider is omitted (no per-turn alt-cost-grant primitive — would need `Player.zaffai_free_cast_used: bool` consumed by an alternative-cost path keyed off a cast-time static). |

## Witherbloom (Black-Green)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Blech, Loafing Pest | {1}{B}{G} | Legendary Creature — Pest | 3/4 | Whenever you gain life, put a +1/+1 counter on each Pest, Bat, Insect, Snake, and Spider you control. | ✅ | `LifeGained` (YourControl) trigger + `ForEach` fan-out filtered to Pest ∪ Bat ∪ Insect ∪ Snake ∪ Spider. |
| Bogwater Lumaret | {B}{G} | Creature — Spirit Frog | 2/2 | Whenever this creature or another creature you control enters, you gain 1 life. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Cauldron of Essence | {1}{B}{G} | Artifact |  | Whenever a creature you control dies, each opponent loses 1 life and you gain 1 life. / {1}{B}{G}, {T}, Sacrifice a creature: Return target creature card from your graveyard to the battlefield. Activate only as a sorcery. | ✅ | Death drain trigger (`CreatureDied/AnotherOfYours`) + `{1}{B}{G},{T},Sac:` reanimation activation, sorcery-speed gated. 🔍 needs review (oracle previously truncated). |
| Dina's Guidance | {1}{B}{G} | Sorcery |  | Search your library for a creature card, reveal it, put it into your hand or graveyard, then shuffle. | ✅ | Push XXVIII: now a 2-mode `ChooseMode` — mode 0 search → hand, mode 1 search → graveyard. AutoDecider defaults to mode 0 (hand) for the unguided bot; a Lorehold / Witherbloom reanimator deck picks mode 1. Tests: `dinas_guidance_searches_creature_to_hand`, `dinas_guidance_mode_one_sends_creature_to_graveyard`. |
| Essenceknit Scholar | {B}{B/G}{G} | Creature — Dryad Warlock | 3/1 | When this creature enters, create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." / At the beginning of your end step, if a creature died under your control this turn, draw a card. | ✅ | ETB Pest token (with on-attack lifegain rider) + end-step gated draw via the new `Predicate::CreaturesDiedThisTurnAtLeast` (backed by `Player.creatures_died_this_turn`). Hybrid `{B/G}` pip approximated as `{B}` (cost: `{B}{B}{G}`). New `CreatureType::Dryad`. |
| Grapple with Death | {1}{B}{G} | Sorcery |  | Destroy target artifact or creature. You gain 1 life. | ✅ | Wired in `catalog::sets::sos::sorceries`. |
| Lluwen, Exchange Student // Pest Friend | {2}{B}{G} // {B/G} | Legendary Creature — Elf Druid // Sorcery | 3/4 |  | 🟡 | Push XV: front 3/4 Legendary Elf Druid vanilla + back-face Sorcery `Pest Friend` ({B/G} → {B}, hybrid pip approximation) creates one Pest token (with the on-attack lifegain rider via the shared `pest_token()` helper). Closes out the Witherbloom (B/G) school. |
| Mind Roots | {1}{B}{G} | Sorcery |  | Target player discards two cards. Put up to one land card discarded this way onto the battlefield tapped under your control. | 🟡 | Discard half wired in `catalog::sets::sos::sorceries` (each opponent discards 2). The "land discarded → battlefield" half is omitted (no track-discarded primitive). |
| Old-Growth Educator | {2}{B}{G} | Creature — Treefolk Druid | 4/4 | Vigilance, reach / Infusion — When this creature enters, put two +1/+1 counters on it if you gained life this turn. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate on the ETB trigger. |
| Pest Mascot | {1}{B}{G} | Creature — Pest Ape | 2/3 | Trample / Whenever you gain life, put a +1/+1 counter on this creature. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Professor Dellian Fel | {2}{B}{G} | Legendary Planeswalker — Dellian | [5] | +2: You gain 3 life. / 0: You draw a card and lose 1 life. / −3: Destroy target creature. / −6: You get an emblem with "Whenever you gain life, target opponent loses that much life." | 🟡 | New `PlaneswalkerSubtype::Dellian` + 5 base loyalty. +2 (gain 3 life), 0 (draw 1 / lose 1 life), -3 (destroy target creature) all wired faithfully. The -6 emblem ult is omitted (no emblem zone yet). |
| Root Manipulation | {3}{B}{G} | Sorcery |  | Until end of turn, creatures you control get +2/+2 and gain menace and "Whenever this creature attacks, you gain 1 life." (A creature with menace can't be blocked except by two or more creatures.) | 🟡 | `ForEach(Creature & ControlledByYou) → PumpPT(+2/+2 EOT) + GrantKeyword(Menace EOT)`. The "whenever this creature attacks → gain 1 life" rider is omitted (no transient-trigger-grant primitive yet). |
| Teacher's Pest | {B}{G} | Creature — Skeleton Pest | 1/1 | Menace (This creature can't be blocked except by two or more creatures.) / Whenever this creature attacks, you gain 1 life. / {B}{G}: Return this card from your graveyard to the battlefield tapped. | ✅ | Push XVII: graveyard-recursion activation wired via the new `from_graveyard: bool` field. Menace + attacks-gain-1 trigger unchanged. Cost `{B}{G}` + effect `Move(Self → Battlefield(You, tapped))`. |
| Titan's Grave |  | Land |  | This land enters tapped. / {T}: Add {B} or {G}. / {2}{B}{G}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Vicious Rivalry | {2}{B}{G} | Sorcery |  | As an additional cost to cast this spell, pay X life. / Destroy all artifacts and creatures with mana value X or less. | ✅ | Wired in `catalog::sets::sos::sorceries` — `LoseLife X` (approximating the additional cost) + `ForEach(Creature ∨ Artifact).If(ManaValueOf ≤ X) → Destroy`. |
| Witherbloom Charm | {B}{G} | Instant |  | Choose one — / • You may sacrifice a permanent. If you do, draw two cards. / • You gain 5 life. / • Destroy target nonland permanent with mana value 2 or less. | ✅ | Push XV → ✅ in push XXVIII: mode 0 (sacrifice → draw 2) wrapped in `Effect::MayDo` — picking mode 0 then declining the sac-prompt keeps everything else stable. Mode 1 (gain 5) and mode 2 (destroy mv≤2) are direct primitives. All three printed modes are wired faithfully. |
| Witherbloom, the Balancer | {6}{B}{G} | Legendary Creature — Elder Dragon | 5/5 | Affinity for creatures (This spell costs {1} less to cast for each creature you control.) / Flying, deathtouch / Instant and sorcery spells you cast have affinity for creatures. | 🟡 | Body wired in `catalog::sets::sos::creatures` with the new `CreatureType::Elder` subtype. Both Affinity-for-creatures cost-reduction clauses are omitted (no per-cast cost reduction whose discount scales off caster's permanent count — tracked in TODO.md). |

## Silverquill (White-Black)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abigale, Poet Laureate // Heroic Stanza | {1}{W}{B} // {1}{W/B} | Legendary Creature — Bird Bard // Sorcery | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Conciliator's Duelist | {W}{W}{B}{B} | Creature — Kor Warlock | 4/3 | When this creature enters, draw a card. Each player loses 1 life. / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, exile up to one target creature. Return that card to the battlefield under its owner's control at the beginning of the next end step. | 🟡 | ETB body wired (draw 1 + each player loses 1). Repartee exile half wired via the new `Selector::CastSpellTarget(0)` primitive. The "return at next end step" rider is still omitted (no capture-as-target-from-selector primitive yet). |
| Fix What's Broken | {2}{W}{B} | Sorcery |  | As an additional cost to cast this spell, pay X life. / Return each artifact and creature card with mana value X from your graveyard to the battlefield. | 🟡 | Push XXV: Pay-X-life folds into resolution as `LoseLife(XFromCost)` (Vicious-Rivalry pattern). The MV=X gate on the gy walk uses `Predicate::All([ValueAtLeast(MV, X), ValueAtMost(MV, X)])` to synthesize equality (no `ValueEquals` primitive). Returns each matching artifact/creature card via `ForEach(EachMatching(Graveyard(You), …)) + Move → Battlefield`. |
| Forum of Amity |  | Land |  | This land enters tapped. / {T}: Add {W} or {B}. / {2}{W}{B}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Imperious Inkmage | {1}{W}{B} | Creature — Orc Warlock | 3/3 | Vigilance / When this creature enters, surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Inkling Mascot | {W}{B} | Creature — Inkling Cat | 2/2 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gains flying until end of turn. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Repartee trigger grants Flying (EOT) on `Selector::This` + Surveil 1. |
| Killian's Confidence | {W}{B} | Sorcery |  | Target creature gets +1/+1 until end of turn. Draw a card. / Whenever one or more creatures you control deal combat damage to a player, you may pay {W/B}. If you do, return this card from your graveyard to your hand. | ✅ | Push XXV: Body (+1/+1 EOT + draw 1) was wired; the graveyard-recursion trigger is now also wired via the new `EventScope::FromYourGraveyard` extension on `fire_combat_damage_to_player_triggers`. The combat-damage trigger fires off the graveyard-resident card; `Effect::MayPay { mana_cost: {W} }` asks the controller yes/no and on yes returns `Selector::This` to its owner's hand via `Move`. Hybrid {W/B} approximated as {W} (matches Practiced Scrollsmith). |
| Moment of Reckoning | {3}{W}{W}{B}{B} | Sorcery |  | Choose up to four. You may choose the same mode more than once. / • Destroy target nonland permanent. / • Return target nonland permanent card from your graveyard to the battlefield. | 🟡 | Wired in `catalog::sets::sos::sorceries` as a 2-mode `ChooseMode`. The "choose up to four / same mode more than once" rider is collapsed to "pick one mode and target one permanent" — same-resolution multi-mode replay needs an engine primitive. |
| Nita, Forum Conciliator | {1}{W}{B} | Legendary Creature — Human Advisor | 2/3 | Whenever you cast a spell you don't own, put a +1/+1 counter on each creature you control. / {2}, Sacrifice another creature: Exile target instant or sorcery card from an opponent's graveyard. You may cast it this turn, and mana of any type can be spent to cast that spell. If that spell would be put into a graveyard, exile it instead. Activate only as a sorcery. | 🟡 | Push XXV: Body wired (2/3 Legendary Human Advisor). The "cast a spell you don't own" trigger + cast-from-opp-graveyard activated ability are omitted — engine has no owned-vs-controlled-spell predicate and no cast-from-graveyard-without-paying for arbitrary cards. |
| Render Speechless | {2}{W}{B} | Sorcery |  | Target opponent reveals their hand. You choose a nonland card from it. That player discards that card. / Put two +1/+1 counters on up to one target creature. | 🟡 | Discard half wired via `DiscardChosen`; the optional creature target is collapsed into a required creature target (no two-target prompt yet). |
| Scolding Administrator | {W}{B} | Creature — Dwarf Cleric | 2/2 | Menace (This creature can't be blocked except by two or more creatures.) / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on this creature. / When this creature dies, if it had counters on it, put those counters on up to one target creature. | 🟡 | Menace + Repartee +1/+1 counter wired in `catalog::sets::sos::creatures` (uses the new `Dwarf` subtype). The truncated "When this creature dies, …" trigger is omitted pending an oracle re-fetch. 🔍 needs review (oracle previously truncated). |
| Silverquill Charm | {W}{B} | Instant |  | Choose one — / • Put two +1/+1 counters on target creature. / • Exile target creature with power 2 or less. / • Each opponent loses 3 life and you gain 3 life. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Silverquill, the Disputant | {2}{W}{B} | Legendary Creature — Elder Dragon | 4/4 | Flying, vigilance / Each instant and sorcery spell you cast has casualty 1. (As you cast that spell, you may sacrifice a creature with power 1 or greater. When you do, copy the spell and you may choose new targets for the copy.) | 🟡 | Push XXV: Body wired (4/4 Legendary Elder Dragon Flying+Vigilance). The casualty-1 grant on instant/sorcery casts is omitted — engine has no static "spells of type X gain casualty N" primitive, and no Casualty keyword yet. |
| Snooping Page | {1}{W}{B} | Creature — Human Cleric | 2/3 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature can't be blocked this turn. / Whenever this creature deals combat damage to a player, you draw a card and lose 1 life. | ✅ | Repartee grants `Keyword::Unblockable` (EOT) on the source; combat-damage trigger wired (draw + lose 1). |
| Social Snub | {1}{W}{B} | Sorcery |  | When you cast this spell while you control a creature, you may copy this spell. / Each player sacrifices a creature of their choice. Each opponent loses 1 life and you gain 1 life. | 🟡 | Push XVII: cast-IS-while-you-control-a-creature copy now wired via the new `Effect::CopySpell` primitive. Trigger filter uses `Predicate::SelectorExists(EachPermanent(Creature & ControlledByYou))`. Main effect: each-player-sac + drain 1. The "of their choice" sac picker is collapsed to auto-pick (each player's auto-decider selects). |
| Stirring Honormancer | {2}{W}{W/B}{B} | Creature — Rhino Bard | 4/5 | When this creature enters, look at the top X cards of your library, where X is the number of creatures you control. Put one of those cards into your hand and the rest into your graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` via `Effect::RevealUntilFind` (find: Creature, cap: count of creatures you control, misses go to graveyard). The hybrid `{W/B}` pip is approximated as `{W}` so cost is `{2}{W}{W}{B}`. |

## Quandrix (Green-Blue)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Applied Geometry | {2}{G}{U} | Sorcery |  | Create a token that's a copy of target non-Aura permanent you control, except it's a 0/0 Fractal creature in addition to its other types. Put six +1/+1 counters on it. | 🟡 | Push XXVI: Body wired as "mint a 0/0 Fractal token + 6 +1/+1 counters" — collapses the "copy a non-Aura permanent" half to a vanilla Fractal mint (no permanent-copy primitive). Net play pattern is a 6/6 Fractal for 4 mana. |
| Berta, Wise Extrapolator | {2}{G}{U} | Legendary Creature — Frog Druid | 1/4 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / Whenever one or more +1/+1 counters are put on Berta, add one mana of any color. / {X}, {T}: Create a 0/0 green and blue Fractal creature token and put X +1/+1 counters on it. | 🟡 | Wired in `catalog::sets::sos::creatures`. Counter-add → AnyOneColor mana trigger uses `EventKind::CounterAdded(PlusOnePlusOne)` + `EventScope::SelfSource` (powered by the new SelfSource → CounterAdded engine recognition). X-cost `{X}{T}: Fractal token + X +1/+1 counters` activation is wired but X resolves to 0 today (engine has no X-cost activated ability path; the X-from-cost path zeroes for activations). Increment rider omitted (mana-spent-on-cast introspection missing). |
| Cuboid Colony | {G}{U} | Creature — Insect | 1/1 | Flash / Flying, trample / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) | ✅ | Push XXIX: Increment **now wired** via `shortcut::increment_self_plus_one()`. Tests: `cuboid_colony_increment_lands_counter_on_two_mana_cast`, `cuboid_colony_increment_skips_one_mana_cast`. |
| Embrace the Paradox | {3}{G}{U} | Instant |  | Draw three cards. You may put a land card from your hand onto the battlefield tapped. | ✅ | Push XVI: draw 3 + `MayDo` rider that picks (at most) one land from hand via `Selector::one_of(CardsInZone(Hand, Land))` and moves it to bf tapped. AutoDecider answers "no" by default; `ScriptedDecider::new([Bool(true)])` exercises the paid path in tests. |
| Fractal Mascot | {4}{G}{U} | Creature — Fractal Elk | 6/6 | Trample / When this creature enters, tap target creature an opponent controls. Put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::creatures` (trample + ETB tap+stun). |
| Fractal Tender | {3}{G}{U} | Creature — Elf Wizard | 3/3 | Ward {2} / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / At the beginning of each end step, if you put a counter on this creature this turn, create a 0/0 green and blue Fractal creature token and put three +1/+1 counters on it. | 🟡 | Body + `Keyword::Ward(2)` wired in `catalog::sets::sos::creatures`. Increment trigger and end-step Fractal-with-counters payoff are both omitted (Increment requires mana-spent introspection on cast; the end-step trigger needs a "did this creature gain a counter this turn" per-permanent flag the engine doesn't track yet). |
| Geometer's Arthropod | {G}{U} | Creature — Fractal Crab | 1/4 | Whenever you cast a spell with {X} in its mana cost, look at the top X cards of your library. Put one of them into your hand and the rest on the bottom of your library in a random order. | ✅ | Push XVI: trigger fully wired via the new `Predicate::CastSpellHasX` + `RevealUntilFind { cap: XFromCost, to: Hand }`. Misses go to graveyard (engine default for `RevealUntilFind`); the printed "rest to bottom random order" rider stays approximated since the engine has no random-bottom primitive. |
| Growth Curve | {G}{U} | Sorcery |  | Put a +1/+1 counter on target creature you control, then double the number of +1/+1 counters on that creature. | ✅ | Wired in `catalog::sets::sos::sorceries`: AddCounter(+1) then AddCounter(`Value::CountersOn`) faithfully doubles. |
| Mind into Matter | {X}{G}{U} | Sorcery |  | Draw X cards. Then you may put a permanent card with mana value X or less from your hand onto the battlefield tapped. | 🟡 | Draw X wired in `catalog::sets::sos::sorceries` via `Value::XFromCost`. The "may put a permanent ≤ X tapped" half is omitted (no hand-to-battlefield mana-value-gated primitive yet). |
| Paradox Gardens |  | Land |  | This land enters tapped. / {T}: Add {G} or {U}. / {2}{G}{U}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Paradox Surveyor | {G}{G/U}{U} | Creature — Elf Druid | 3/3 | Reach / When this creature enters, look at the top five cards of your library. You may reveal a land card or a card with {X} in its mana cost from among them and put it into your hand. Put the rest on the bottom of your library in a random order. | ✅ | Push XVI: filter promoted to `Land OR HasXInCost` via the new `SelectionRequirement::HasXInCost` primitive — exact-printed reveal filter. Hybrid `{G/U}` pip stays approximated as `{G}` (cost: `{G}{G}{U}`). Misses go to graveyard. |
| Proctor's Gaze | {2}{G}{U} | Instant |  | Return up to one target nonland permanent to its owner's hand. Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | Wired in `catalog::sets::sos::instants`: bounce target nonland to owner's hand, then `Search { filter: IsBasicLand, to: Battlefield(tapped) }`. |
| Pterafractyl | {X}{G}{U} | Creature — Dinosaur Fractal | 1/0 | Flying / This creature enters with X +1/+1 counters on it. / When this creature enters, you gain 2 life. | 🟡 | Wired in `catalog::sets::sos::creatures` with base toughness bumped 1/0→1/1 (no replacement-effect primitive yet, so a 1/0 body would die to SBA before its X-counter ETB trigger fires). The X-counter ETB now reads the cast's X correctly via the engine's new trigger-context `x_value` plumbing. |
| Quandrix Charm | {G}{U} | Instant |  | Choose one — / • Counter target spell unless its controller pays {2}. / • Destroy target enchantment. / • Target creature has base power and toughness 5/5 until end of turn. | 🟡 | Modes 0 (counter unless {2}) and 1 (destroy enchantment) wired in `catalog::sets::sos::instants`. Mode 2 is approximated as a flat +3/+3 EOT (the engine's `Effect::ResetCreature` is a stub, so a true "set base 5/5" rewrite isn't possible yet). |
| Quandrix, the Proof | {4}{G}{U} | Legendary Creature — Elder Dragon | 6/6 | Flying, trample / Cascade (When you cast this spell, exile cards from the top of your library until you exile a nonland card that costs less. You may cast it without paying its mana cost. Put the exiled cards on the bottom in a random order.) / Instant and sorcery spells you cast from your hand have cascade. | 🟡 | Push XXVIII: ⏳ → 🟡. Body wired faithfully — 6/6 Legendary Elder Dragon with Flying + Trample. The Cascade keyword and the IS-grant-cascade static are still ⏳ (no Cascade keyword primitive, no cast-from-exile-without-paying pipeline). At raw stats this is a 6-mana 6/6 flying trampler — strong finisher even without Cascade. |
| Tam, Observant Sequencer // Deep Sight | {2}{G}{U} // {G}{U} | Legendary Creature — Gorgon Wizard // Sorcery | 4/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|

## Lorehold (Red-White)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Ark of Hunger | {2}{R}{W} | Artifact |  | Whenever one or more cards leave your graveyard, this artifact deals 1 damage to each opponent and you gain 1 life. / {T}: Mill a card. You may play that card this turn. | 🟡 | Wired against the `EventKind::CardLeftGraveyard` event — drain 1 (1 to each opp + you gain 1) per gy-leave emission. The {T}: Mill activation is wired faithfully; the "may play that card this turn" rider is omitted (same gap as Tablet of Discovery, Suspend Aggression). |
| Aziza, Mage Tower Captain | {R}{W} | Legendary Creature — Djinn Sorcerer | 2/2 | Whenever you cast an instant or sorcery spell, you may tap three untapped creatures you control. If you do, copy that spell. You may choose new targets for the copy. | ✅ | Push XVII: cast-IS copy rider now wired via the new `Effect::CopySpell` primitive. The "may tap three" cost uses `Effect::MayDo` (yes/no decision) + `Effect::Tap` with `Selector::Take(Untapped Creatures, 3)` + `Effect::CopySpell { what: Selector::TriggerSource }`. The picker may include Aziza herself in the tap-three pool. |
| Borrowed Knowledge | {2}{R}{W} | Sorcery |  | Choose one — / • Discard your hand, then draw cards equal to the number of cards in target opponent's hand. / • Discard your hand, then draw cards equal to the number of cards discarded this way. | 🟡 | Mode 0 wired faithfully (discard hand → draw target opp's hand size). Mode 1 collapsed to "discard hand → draw 7" — engine has no track-discarded-by-this-effect primitive yet. |
| Colossus of the Blood Age | {4}{R}{W} | Artifact Creature — Construct | 6/6 | When this creature enters, it deals 3 damage to each opponent and you gain 3 life. / When this creature dies, discard any number of cards, then draw that many cards plus one. | 🟡 | ETB drain wired (3-to-each-opp + gain 3). Death "discard any number, draw that many plus one" collapsed to discard 1 + draw 2 (no track-discarded-by-this-effect primitive). |
| Fields of Strife |  | Land |  | This land enters tapped. / {T}: Add {R} or {W}. / {2}{R}{W}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Hardened Academic | {R}{W} | Creature — Bird Cleric | 2/1 | Flying, haste / Discard a card: This creature gains lifelink until end of turn. / Whenever one or more cards leave your graveyard, put a +1/+1 counter on target creature you control. | ✅ | All three abilities wired. The cards-leave-graveyard trigger uses the new `EventKind::CardLeftGraveyard` event (per-card emission; "one or more" rider is naturally per-card). |
| Kirol, History Buff // Pack a Punch | {R}{W} // {1}{R}{W} | Legendary Creature — Vampire Cleric // Sorcery | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Lorehold Charm | {R}{W} | Instant |  | Choose one — / • Each opponent sacrifices a nontoken artifact of their choice. / • Return target artifact or creature card with mana value 2 or less from your graveyard to the battlefield. / • Creatures you control get +1/+1 and gain trample until end of turn. | ✅ | Wired in `catalog::sets::sos::instants` — all three modes wired with existing primitives (`Sacrifice`, `Move from gy`, `ForEach(Creature) → PumpPT`). |
| Lorehold, the Historian | {3}{R}{W} | Legendary Creature — Elder Dragon | 5/5 | Flying, haste / Each instant and sorcery card in your hand has miracle {2}. (You may cast a card for its miracle cost when you draw it if it's the first card you drew this turn.) / At the beginning of each opponent's upkeep, you may discard a card. If you do, draw a card. | 🟡 | Body-only wire (5/5 Flying+Haste Legendary Elder Dragon, R/W). Miracle grant on instants/sorceries in hand is omitted (no miracle/alt-cost-on-draw primitive); per-opp-upkeep loot trigger omitted (no opp-upkeep step trigger that fires for non-active player). The vanilla finisher is the most impactful printed clause — both omitted clauses are tracked in TODO.md. |
| Molten Note | {X}{R}{W} | Sorcery |  | Molten Note deals damage to target creature equal to the amount of mana spent to cast this spell. Untap all creatures you control. / Flashback {6}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Body wired: X damage to target creature (X = `Value::XFromCost`, an approximation of "mana spent to cast"; matches the typical play pattern since cost-to-X is mostly identical absent cost reductions). Untap all your creatures wired. Flashback {6}{R}{W} via `Keyword::Flashback`. |
| Practiced Scrollsmith | {R}{R/W}{W} | Creature — Dwarf Cleric | 3/2 | First strike / When this creature enters, exile target noncreature, nonland card from your graveyard. Until the end of your next turn, you may cast that card. | 🟡 | Wired in `catalog::sets::sos::creatures`. ETB now exiles **exactly one** matching noncreature/nonland card in the controller's graveyard via the new `Selector::Take(_, 1)` primitive (push X) — closer to the printed "target one card" semantics; the prior implementation exiled every matching card. The hybrid `{R/W}` pip is approximated as `{R}` (cost: `{R}{R}{W}`). The "may cast until next turn" rider is omitted (no cast-from-exile-with-time-limit primitive). |
| Pursue the Past | {R}{W} | Sorcery |  | You gain 2 life. You may discard a card. If you do, draw two cards. / Flashback {2}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Push XV → ✅ in push XXVIII: gain 2 + the discard+draw chain wrapped in `Effect::MayDo` so the printed "you may discard" optionality is honored. Flashback wired via `Keyword::Flashback`. The lifegain half always resolves; the loot half is opt-in. All Oracle clauses wired. |
| Spirit Mascot | {R}{W} | Creature — Spirit Ox | 2/2 | Whenever one or more cards leave your graveyard, put a +1/+1 counter on this creature. | ✅ | Wired against the new `EventKind::CardLeftGraveyard` event. Trigger fires per-card emission (the printed "one or more" wording is approximated per-card). |
| Startled Relic Sloth | {2}{R}{W} | Creature — Sloth Beast | 4/4 | Trample, lifelink / At the beginning of combat on your turn, exile up to one target card from a graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` (trample + lifelink + begin-combat exile-from-GY trigger; same shape as Ascendant Dustspeaker's combat trigger). Sloth subtype bridged through Beast (no Sloth creature type yet). |
| Suspend Aggression | {1}{R}{W} | Instant |  | Exile target nonland permanent and the top card of your library. For each of those cards, its owner may play it until the end of their next turn. | 🟡 | Wired in `catalog::sets::sos::instants` as a `Seq` of two `Move → Exile` calls (target nonland permanent + caster's top of library). `move_card_to` was extended to walk libraries when locating the source card so the top-of-library exile resolves end-to-end. The "may play those cards until next end step" rider is omitted (no per-card "may-play-from-exile-until-EOT" primitive). |
| Wilt in the Heat | {2}{R}{W} | Instant |  | This spell costs {2} less to cast if one or more cards left your graveyard this turn. / Wilt in the Heat deals 5 damage to target creature. If that creature would die this turn, exile it instead. | 🟡 | Wired as a 5-damage-to-target-creature spell. The cost-reduction-when-cards-left-gy clause is omitted (no `StaticEffect::CostReduction` variant gated on a per-turn tally). The "if it would die, exile instead" replacement is omitted (no damage-replacement primitive). |

## Colorless

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Biblioplex Tomekeeper | {4} | Artifact Creature — Construct | 3/4 | When this creature enters, choose up to one — / • Target creature becomes prepared. (Only creatures with prepare spells can become prepared.) / • Target creature becomes unprepared. | 🟡 | Push XXV: Body wired (3/4 Construct artifact creature). The Prepare toggle is omitted — engine has no Prepare keyword nor a prepared-state flag (same gap as Skycoach Waypoint). |
| Diary of Dreams | {2} | Artifact — Book |  | Whenever you cast an instant or sorcery spell, put a page counter on this artifact. / {5}, {T}: Draw a card. This ability costs {1} less to activate for each page counter on this artifact. | 🟡 | Page-counter accrual on instant/sorcery cast (counter type approximated as Charge — engine has no Page counter) + flat {5},{T} draw. The page-counter-scaled cost reduction is omitted (no self-counter cost-reduction primitive). |
| Great Hall of the Biblioplex |  | Land |  | {T}: Add {C}. / {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell. / {5}: If this land isn't a creature, it becomes a 2/4 Wizard creature with "Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn." It's still a land. | 🟡 | Push XV: legendary colorless utility land. `{T}: Add {C}` faithful + `{T}, Pay 1 life: Add one mana of any color` via the new `ActivatedAbility.life_cost: u32` field — the effect is a pure mana ability (`AddMana(AnyOneColor 1)`) so it resolves immediately without going on the stack. The `{5}: becomes 2/4 Wizard creature` clause is omitted (no land-becomes-creature primitive — same gap as Mishra's Factory). The spend-restriction rider on the rainbow ability is omitted (no per-pip mana metadata yet). |
| Mage Tower Referee | {2} | Artifact Creature — Construct | 2/1 | Whenever you cast a multicolored spell, put a +1/+1 counter on this creature. | ✅ | Wired in `catalog::sets::sos::creatures` with a `SpellCast/YourControl` trigger filtered on `EntityMatches(TriggerSource, Multicolored)` — uses the new `SelectionRequirement::Multicolored` predicate (≥ 2 distinct colored pips, hybrid both halves, Phyrexian colored side). Mono-color and colorless casts don't bump the Referee. |
| Page, Loose Leaf | {2} | Legendary Artifact Creature — Construct | 0/2 | {T}: Add {C}. / Grandeur — Discard another card named Page, Loose Leaf: Reveal cards from the top of your library until you reveal an instant or sorcery card. Put that card into your hand and the rest on the bottom of your library in a random order. | 🟡 | Body wired (0/2 Legendary Construct artifact creature) + `{T}: Add {C}` mana ability via the shared `tap_add_colorless()` helper. Grandeur (discard-named-this-card-as-cost activation) omitted. |
| Petrified Hamlet |  | Land |  | When this land enters, choose a land card name. / Activated abilities of sources with the chosen name can't be activated unless they're mana abilities. / Lands with the chosen name have "{T}: Add {C}." / {T}: Add {C}. | 🟡 | Mana ability `{T}: Add {C}` wired via the new shared `tap_add_colorless()` helper in `catalog::sets`. The "choose a land card name" prompt + name-keyed lock-out static + name-keyed grant of `{T}: Add {C}` on opp lands are all omitted (no name-prompt decision, no name-match selector). The card still slots into colorless utility roles. |
| Potioner's Trove | {3} | Artifact |  | {T}: Add one mana of any color. / {T}: You gain 2 life. Activate only if you've cast an instant or sorcery spell this turn. | 🟡 | Mana ability + lifegain ability wired; the "if you've cast an instant or sorcery this turn" gate on the lifegain activation is omitted (no per-turn-cast gate on activated abilities yet). |
| Rancorous Archaic | {5} | Creature — Avatar | 2/2 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. | ✅ | Wired in `catalog::sets::sos::creatures` (trample/reach + ETB AddCounter using `Value::ConvergedValue`). Powered by the engine's new `StackItem::Trigger.converged_value` plumbing. |
| Skycoach Waypoint |  | Land |  | {T}: Add {C}. / {3}, {T}: Target creature becomes prepared. (Only creatures with prepare spells can become prepared.) | 🟡 | Push XXV: `{T}: Add {C}` mana ability wired via `tap_add_colorless()`. The {3},{T} prepare-target ability is omitted — engine has no Prepare keyword (same gap as Biblioplex Tomekeeper). |
| Strixhaven Skycoach | {3} | Artifact — Vehicle | 3/2 | Flying / When this Vehicle enters, you may search your library for a basic land card, reveal it, put it into your hand, then shuffle. / Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.) | 🟡 | Push XXVI: Body wired — 3/2 Vehicle artifact with Flying. ETB basic-land tutor-to-hand via `Effect::Search { filter: IsBasicLand, to: Hand(You) }`. Crew is not enforced (no crew-as-tap-cost primitive yet); the Skycoach stays a non-creature artifact until that lands. |
| Sundering Archaic | {6} | Creature — Avatar | 3/3 | Converge — When this creature enters, exile target nonland permanent an opponent controls with mana value less than or equal to the number of colors of mana spent to cast this creature. / {2}: Put target card from a graveyard on the bottom of its owner's library. | 🟡 | Push XVI: `{2}: gy → bottom of owner's library` activated ability now wired via `Effect::Move { what: Target(0), to: ZoneDest::Library { who: OwnerOf(Target(0)), pos: Bottom } }`. ETB Converge exile is wired against `Nonland & ControlledByOpponent`; the mana-value cap against `ConvergedValue` is still approximated to "any nonland opp permanent" (no `Value`-keyed `ManaValueAtMost` predicate yet — tracked in TODO.md). |
| The Dawning Archaic | {10} | Legendary Creature — Avatar | 7/7 | This spell costs {1} less to cast for each instant and sorcery card in your graveyard. / Reach / Whenever The Dawning Archaic attacks, you may cast target instant or sorcery card from your graveyard without paying its mana cost. If that spell would be put into your graveyard, exile it instead. | 🟡 | Push XXV: Body wired (7/7 Legendary Avatar with Reach). The IS-in-gy cost-reduction static + attack-trigger cast-from-graveyard rider are omitted — engine has no per-graveyard-IS-count cost-reduction primitive nor cast-from-graveyard-without-paying for arbitrary cards. |
| Together as One | {6} | Sorcery |  | Converge — Target player draws X cards, Together as One deals X damage to any target, and you gain X life, where X is the number of colors of mana spent to cast this spell. | 🟡 | Damage and life-gain halves wired in `catalog::sets::sos::sorceries`; the "target player draws X" half collapses to "you draw X" (multi-target prompt gap). |
| Transcendent Archaic | {7} | Creature — Avatar | 6/6 | Vigilance / Converge — When this creature enters, you may draw X cards, where X is the number of colors of mana spent to cast this spell. If you draw one or more cards this way, discard two cards. | 🟡 | Body wired (6/6 Vigilance Avatar). ETB Converge draw is wired via `Value::ConvergedValue`; the conditional discard 2 is gated on `ConvergedValue ≥ 1`. The "you may" optionality is collapsed to always-draw-when-X-≥-1 (no may-do primitive yet). |

## Strixhaven base set (STX)

Strixhaven 2021 cards (separate from the supplemental SOS catalog above).
Cards live under `catalog::sets::stx` and use the same engine primitives
as the SOS module. The two catalogs are independent — bringing them up to
parity is a matter of porting card factories one at a time.

### Silverquill (W/B)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Spirited Companion | {1}{W} | ✅ | 1/2 Dog Spirit. ETB: draw a card. |
| Eyetwitch | {B} | ✅ | 1/1 Pest. When dies: "learn" approximated as `Draw 1` (no Lesson sideboard yet). |
| Closing Statement | {X}{W}{W} | ✅ | Sorcery. Exile target nonland permanent. You gain X life (`Value::XFromCost`). |
| Vanishing Verse | {W}{B} | ✅ | Instant. Exile target nonland, **monocolored** permanent. Push XIX: filter promoted to the printed exact-shape `Permanent & Nonland & Monocolored` via the new `SelectionRequirement::Monocolored` predicate (`distinct_colors() == 1`). Multicolored and colorless permanents are correctly rejected by the cast-time target validator. |
| Killian, Ink Duelist | {W}{B} | 🟡 | 2/3 Legendary Human Warlock. Lifelink wired. "Spells you cast that target a creature cost {2} less" static still ⏳ (target-aware cost reduction primitive). |
| Devastating Mastery | {4}{W}{W} | 🟡 | Sorcery. Destroy each nonland permanent ("Wrath of God + lands"). Alt cost {7}{W}{W} reanimate clause is ⏳ (alt-cost-implies-mode primitive). |
| Felisa, Fang of Silverquill | {2}{W}{B} | ✅ | 4/3 Legendary Cat Cleric, Flying + Lifelink. Push XVI: counter-bearing-creature-dies → Inkling trigger now wired via `EventKind::CreatureDied/AnotherOfYours` filtered by `EntityMatches { what: TriggerSource, filter: WithCounter(+1/+1) }`. Counters persist on a card after move-to-graveyard (only `damage`/`tapped`/`attached_to` are cleared on zone-out), so the post-die graveyard-resident card still reports its `+1/+1` counters via `evaluate_requirement_static`. |
| Mavinda, Students' Advocate | {1}{W}{W} | 🟡 | 1/3 Legendary Human Cleric, Flying + Vigilance. Cast-from-graveyard activated ability is ⏳. |
| Eager First-Year | {W} | ✅ | 2/1 Human Student. Magecraft: target creature gets +1/+1 EOT. Uses the new `effect::shortcut::magecraft()` helper. |
| Hunt for Specimens | {3}{B} | 🟡 | Sorcery. Creates a 1/1 black Pest token (death-trigger lifegain now rides on the token via SOS-VI's `TokenDefinition.triggered_abilities`) + draws a card (Learn approx — no Lesson sideboard yet). |
| Star Pupil | {B} | ✅ | Push XXIII: 0/1 Cat Spirit. ETB +1/+1 counter; dies → put a +1/+1 counter on target creature. Audited against CR 122.8. |
| Silverquill Command | {2}{W}{B} | 🟡 | Push XXVII: Instant — `ChooseMode` collapse to single-mode pick. Modes wired: counter activated/triggered ability, drain 2, return MV ≤ 2 permanent card from your gy → bf, two +1/+1 counters on creature. The "choose two" mega-pick is the standard Command-cycle approximation tracked in TODO.md. |
| Defend the Campus | {3}{W}{W} | ✅ | Push XXVII: Sorcery. Creates three 1/1 W/B Inkling tokens with flying via `Effect::CreateToken { count: 3 }`. Reuses the SOS catalog's `inkling_token()` definition. |
| Hall Monitor | {W} | ✅ | Push XXVII: 1/1 Human Cleric. Magecraft: untap this creature. Wired via `magecraft(Effect::Untap)`. |
| Stonebinder's Familiar | {1} | ✅ | Push XXVII: 0/1 Artifact Creature — Spirit. "Whenever one or more cards leave your graveyard, put a +1/+1 counter on this creature." Uses the `EventKind::CardLeftGraveyard / YourControl` trigger (per-card emission, same as Spirit Mascot). |
| Necrotic Fumes | {2}{B}{B} | 🟡 | Push XXVII: Sorcery. As an additional cost, sacrifice a creature. Exile target creature. Wired as `Seq(Sacrifice + Move→Exile)` at resolution time (the engine has no cast-time additional-cost prompt yet, so the sacrifice happens during resolution; net effect is preserved). |
| Make Your Mark | {1}{W} | ✅ | Push XXVII: Instant. +1/+1 EOT on target creature, draw a card. Trivial pump+cantrip wire. |
| Containment Breach | {1}{W} | ✅ | Push XXVII: Sorcery. Destroy target enchantment + Surveil 1. |

### Witherbloom (B/G)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Witherbloom Apprentice | {B}{G} | ✅ | 2/2 Human Warlock. Magecraft: drain 1 (each opponent loses 1; you gain 1). |
| Pest Summoning | {B}{G} | ✅ | Sorcery (Lesson). Creates two 1/1 Pest tokens; the death-trigger lifegain rider rides on the token via SOS-VI's `TokenDefinition.triggered_abilities`. |
| Witherbloom Pledgemage | {1}{B}{G} | ✅ | Push XXVIII: 3/3 Plant Warrior. `{T}, Pay 1 life: Add {B} or {G}.` Refactored in push XVIII to use the `life_cost: 1` field on the activated ability — the activation pays 1 life up front during cost-payment, leaving the effect as a pure `AddMana`. CR 605.1a's "no target, could add mana" criteria are met and the engine's `is_mana_ability` recogniser resolves this **without the stack** (matching the printed instant-speed mana ramp). The "B or G" choice is approximated as `ManaPayload::AnyOneColor`. Promotion to ✅ — the prior timing-nuance note didn't reflect any missing functionality. |
| Bayou Groff | {2}{B}{G} | ✅ | 5/4 Beast. Push XVI: "may pay {1} on death to return to hand" rider now wired via the new `Effect::MayPay` primitive (sibling to push XV's `Effect::MayDo`). On the death trigger, the controller is asked yes/no; on yes + sufficient mana, the engine pays {1} and `Move(SelfSource → Hand(OwnerOf(Self)))`. |
| Honor Troll | {1}{B}{G} | ✅ | Push XXVIII: 1/4 Troll Warrior with Trample. The conditional `+2/+0` and Lifelink rider is **now wired** via a compute-time injection in `GameState::compute_battlefield` (same pattern as Cruel Somnophage / Tarmogoyf). The gate reads `Player.life_gained_this_turn` (already tracked for the `LifeGainedThisTurnAtLeast` predicate); when > 0, layers 6 (keyword) and 7b (P/T modify) inject `AddKeyword(Lifelink)` and `ModifyPowerToughness(+2, +0)` targeting the Troll as `AffectedPermanents::Source`. The gate re-evaluates every recompute, so a mid-turn lifegain flips the troll on for the rest of the turn, and `do_untap`'s reset to `life_gained_this_turn = 0` flips it back off next turn. Tests: `honor_troll_base_state_no_lifegain_is_one_four`, `honor_troll_with_lifegain_is_three_four_lifelink`. |
| Daemogoth Titan | {B}{B} | ✅ | Push XX + push XXVI: 11/11 Demon Horror. Attack-trigger sacrifice + block-trigger sacrifice now both wired. Block-half uses the new `EventKind::Blocks` event added in push XXVI (per CR 509.1i — blocker-side triggers). The sacrifice's auto-decider picks fodder before the Titan itself when both exist. |
| Daemogoth Woe-Eater | {2}{B}{G} | ✅ | Push XXVIII: 4/4 Demon Horror. ETB sacrifice (mandatory) + attack-trigger sac-into-+1/+1-counter `Seq` now wrapped in `Effect::MayDo` so the printed "you may sacrifice" optionality is honored. AutoDecider defaults to "no" (skip the sac); `ScriptedDecider::new([Bool(true)])` exercises the paid path. Tests: `daemogoth_woe_eater_etb_sacrifices_another_creature`, `daemogoth_woe_eater_attack_optional_sac_can_be_declined`, `daemogoth_woe_eater_attack_optional_sac_can_be_accepted`. |
| Mortality Spear | {3}{B}{G} | ✅ | Push XX: Instant. Destroy target creature or planeswalker (Battle subtype omitted — not modelled in this catalog). |
| Tempted by the Oriq | {2}{B} | 🟡 | Push XX: Sorcery. Temp-steal + untap + Haste EOT (Threaten template). The printed Magecraft rider on the controlled creature is ⏳. |
| Witherbloom Command | {2}{B}{G} | 🟡 | Push XXVII: Sorcery — `ChooseMode` collapse to single-mode pick. Modes wired: target opponent mills 4, destroy noncreature/nonland MV ≤ 2, drain 2, grant Indestructible EOT (regenerate approximation — engine has `Keyword::Regenerate(N)` as a tag but no regen-shield enforcement). The "choose two" mega-pick is the standard Command-cycle approximation tracked in TODO.md. |

### Lorehold (R/W)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Lorehold Apprentice | {R}{W} | 🟡 | 1/1 Human Cleric. Magecraft: gain 1 life (the "1 damage to any target" half is omitted — magecraft trigger doesn't yet auto-pick a target). |
| Lorehold Pledgemage | {1}{R}{W} | 🟡 | 2/2 Spirit Cleric with Reach. Activated `{2}{R}{W}, exile a card from your graveyard: +1/+1 EOT` is ⏳ (no exile-from-GY cost primitive). |
| Pillardrop Rescuer | {3}{R}{W} | ✅ | 3/3 Spirit Cleric with Flying. ETB: return target instant or sorcery card from your graveyard to your hand. |
| Heated Debate | {2}{R} | ✅ | Instant. 4 damage to target creature. Damage-can't-be-prevented rider is a no-op (engine has no prevention layer). |
| Storm-Kiln Artist | {2}{R}{W} | 🟡 | 3/3 Human Wizard. Magecraft: 1 damage to each opponent + create a Treasure (printed: "1 damage to any target"; collapsed to each-opponent for the auto-target framework). |
| Sparring Regimen | {2}{R}{W} | 🟡 | Enchantment. ETB creates a 2/2 R/W Spirit token. The "whenever you attack, +1/+1 counter on each attacker" rider is ⏳ pending a `PlayerAttackedWith` event over all declared attackers. |
| Lorehold Command | {2}{R}{W} | 🟡 | Push XXVII: Sorcery — `ChooseMode` collapse to single-mode pick. Modes wired: 4 damage to target opponent, -2/-0 EOT on target creature (collapsed from "until your next turn"), return creature card from your gy → hand, create two 2/2 R/W Spirit tokens **with flying** (matching the printed Lorehold STX printing — distinct from the SOS catalog's `lorehold_spirit_token()` no-flying default). |

### Quandrix (G/U)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Quandrix Apprentice | {G}{U} | ✅ | 1/1 Elf Druid. Magecraft: target creature you control gets +1/+1 EOT. |
| Quandrix Pledgemage | {1}{G}{U} | ✅ | 2/2 Fractal Wizard. Activated `{1}{G}{U}: +1/+1 counter on this creature`. |
| Decisive Denial | {G}{U} | 🟡 | Instant. Mode 0 (counter target noncreature spell unless its controller pays {2}) wired; mode 1 (fight at variable power) ⏳ pending multi-target prompt. |
| Quandrix Cultivator | {3}{G}{U} | ✅ | Push XX: 3/3 Elf Druid. ETB search basic Forest or Island → battlefield tapped. |
| Manifestation Sage | {2}{G}{U} | ✅ | Push XXIII: 2/2 Fractal Wizard, Flying. ETB mints 0/0 Fractal + X +1/+1 counters where X = `HandSizeOf(You)`. |
| Quandrix Command | {1}{G}{U} | 🟡 | Push XXVII: Instant — `ChooseMode` collapse to single-mode pick. Modes wired: two +1/+1 counters on creature, counter target activated/triggered ability via `Effect::CounterAbility`, target opponent mills 2 (X collapsed from "twice your creature count" — engine has no `Value::Times(N, CountOf(...))` for cast-time Mill counts), bounce nonland permanent to owner's hand. |
| Mentor's Guidance | {1}{G}{U} | ✅ | Push XXIII: Instant. Two-mode `ChooseMode` — damage = creatures you control, or draw = creatures with +1/+1 counters. |

### Prismari (U/R)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Prismari Pledgemage | {1}{U}{R} | ✅ | 2/3 Elemental with Trample + Haste. |
| Prismari Apprentice | {U}{R} | 🟡 | 2/2 Human Wizard. Magecraft: Scry 1. The "+1/+0 EOT" alt-mode is ⏳ pending a let-the-controller-pick hook on triggered ChooseMode. |
| Symmetry Sage | {U} | ✅ | 1/2 Human Wizard. Magecraft: this creature gets +1/+0 and gains flying until end of turn. |
| Galvanic Iteration | {U}{R} | 🟡 | Push XXIV: Instant. Copy target instant or sorcery spell via `Effect::CopySpell`. Magecraft self-exile rider omitted. |
| Expressive Iteration | {U}{R} | 🟡 | Push XXIV: Sorcery. Collapsed to `Scry 2 → Draw 1` (the exile-and-play-from-exile primitive is ⏳). |
| Magma Opus | {7}{U}{R} | 🟡 | Push XXIV: Sorcery. 4 dmg + tap opp creatures + 3/3 Elemental token + draw 2. Multi-target divided damage + discard alt-mode omitted. |
| Sparkmage Apprentice | {1}{R} | ✅ | Push XXIV: 1/2 Human Wizard. ETB: deals 2 damage to any target. |
| Soothsayer Adept | {1}{U} | ✅ | Push XXIV: 2/2 Merfolk Wizard. Activated `{2}{U}: Surveil 1`. |
| Prismari Command | {1}{U}{R} | 🟡 | Push XXVII: Instant — `ChooseMode` collapse to single-mode pick. Modes wired: 2 damage to any target, loot 1 (discard + draw, "extra draw if discarded card is noncreature/nonland" rider collapsed to flat draw), create a Treasure token (uses engine's `treasure_token()`), destroy target artifact. |

### Mono-color staples (`stx::mono`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Pop Quiz | {1}{W} | ✅ | Sorcery (Lesson). Draw 2, then put a card from your hand on top of your library. |
| Mascot Exhibition | {5}{W}{W} | ✅ | Sorcery. Creates a 3/3 Elephant, 2/2 lifelink Cat, and 1/1 flying Bird. |
| Plumb the Forbidden | {X}{B}{B} | ✅ | Instant. Sacrifice X creatures, draw X cards, lose X life. |
| Owlin Shieldmage | {3}{W} | 🟡 | 2/3 Bird Wizard, Flash + Flying. Combat-damage prevention ETB is ⏳ (replacement primitive). |
| Frost Trickster | {1}{U} | ✅ | 2/2 Spirit Wizard, Flash + Flying. ETB taps + stuns target opponent's creature. |
| Body of Research | {4}{G}{U} | ✅ | Push XVI: now uses the new `Value::LibrarySizeOf(You)` primitive — Fractal token enters with one +1/+1 counter per library card, matching the printed Oracle exactly (was approximating via `GraveyardSizeOf` since `LibrarySize` wasn't a primitive). |
| Show of Confidence | {1}{W} | ✅ | Instant. Adds `StormCount + 1` +1/+1 counters on target creature you control. |
| Bury in Books | {3}{U} | ✅ | Sorcery. Put target creature on top of its owner's library. |
| Test of Talents | {1}{U}{U} | 🟡 | Counter target instant or sorcery; the search-and-exile-by-name follow-up is ⏳. |
| Multiple Choice | {1}{U}{U} | 🟡 | Modal sorcery with three modes wired (Scry 2 / 1/1 Pest / +1/+0 hexproof EOT). The "all four" mega-mode is ⏳. |
| Curate | {1}{U} | 🟡 | Push XX: Instant. "Look at top 4, put 1 in hand, rest on bottom in random order" approximated as `Scry 3 → Draw 1`. |
| Solve the Equation | {2}{U} | ✅ | Push XX: Sorcery. Tutor an instant or sorcery from library to hand (printed mana-value cap omitted for simplicity). |
| Resculpt | {1}{U} | ✅ | Push XX: Instant. Exile target creature or artifact; its original controller creates a 4/4 blue Elemental token. |
| Ageless Guardian | {2}{W} | ✅ | Push XXIII: 1/4 Spirit Cleric. Magecraft: this creature gets +1/+0 EOT (`magecraft_self_pump`). |
| Returned Pastcaller | {4}{W} | ✅ | Push XXIII: 3/3 Spirit Cleric, Flying. ETB return target IS card from your graveyard to hand. |
| Letter of Acceptance | {1} | ✅ | Push XXIII: Artifact. ETB +1 life; `{T}: Add {C}`; `{2},{T},Sac: Draw a card`. |
| Charge Through | {G} | ✅ | Push XXIII: Sorcery. Target creature you control gets +1/+1 and gains trample EOT. |
| Devious Cover-Up | {2}{U}{U} | 🟡 | Push XXIII: Instant. Counter target spell + exile up to one gy card. Multi-target prompt for "any number" still gates the full Oracle. |
| Crackle with Power | {X}{R}{R}{R}{R}{R} | 🟡 | Push XXIII: Sorcery. 5X damage to single target (multi-target divided rider collapses to one target). |
| Dragonsguard Elite | {1}{G}{G} | ✅ | Push XXIII: 2/2 Human Warrior. Magecraft +1/+1 counter; `{3}{G}: +X/+X EOT` where X = `PowerOf(This)`. |
| Quintorius, Field Historian | {2}{R}{W} | 🟡 | Push XXIII: 3/3 Legendary Elephant Cleric Spirit, Vigilance. ETB exile gy card + create 3/2 R/W Spirit. Tribal anthem static omitted. |
| Crashing Drawbridge | {3} | ✅ | Push XXIV: 0/4 Artifact Creature — Construct. "Creatures you control have haste" via `StaticEffect::GrantKeyword`. |
| Eyetwitch Brood | {1}{B}{G} | ✅ | Push XXIV: 1/1 Pest with Lifelink. "Whenever another Pest you control dies, put a +1/+1 counter on this creature." Disambiguated from SOS's "Pest Mascot" (LifeGained trigger). |
| First Day of Class | {W} | 🟡 | Push XXIV: Sorcery. ForEach creature you control gets +1/+1 EOT. Combat-damage-to-Pest rider omitted. |
| Verdant Mastery | {3}{G}{G} | 🟡 | Push XXIV: Sorcery. You + each opponent fetch a basic land (opponent's tapped). {6}{G}{G} two-basics alt-cost omitted. |
| Sacred Fire | {R}{W} | ✅ | Push XXIV: Sorcery. 3 damage to any target + 3 life + Flashback {5}{R}{W}. |
| Rip Apart | {R}{W} | ✅ | Push XXIV: Sorcery. Two-mode `ChooseMode`: 3 dmg to creature/PW or destroy artifact/enchantment. |
| Codespell Cleric | {W} | ✅ | Push XXIV: 1/1 Kor Cleric with Lifelink. Vanilla low-curve Silverquill body. |
| Reckless Amplimancer | {2}{G} | 🟡 | Push XXIV: 2/2 Elf Druid. Activated `{4}{G}{G}: +3/+3 EOT` (printed `+X/+X = mana spent` collapses to fixed +3/+3). |
| Karok Wrangler | {1}{G}{U} | ✅ | Push XXIV: 2/2 Elf Druid. Magecraft: +1/+1 counter on target creature you control. |
| Quick Study | {1}{U} | ✅ | Push XXV: Instant. Target player draws two cards. Simple `Effect::Draw { who: Player(You), amount: 2 }`. |
| Bookwurm | {5}{G}{G} | ✅ | Push XXIX (doc sync): 5/5 Wurm with Trample. ETB: gain 4 life + draw 1. |
| Field Trip | {2}{G} | ✅ | Push XXIX (doc sync): Sorcery. Search basic Forest → battlefield + Learn (→ Draw 1). |
| Reduce to Memory | {2}{U} | ✅ | Push XXIX (doc sync): Sorcery. Exile target nonland permanent + its controller mints a 4/4 blue Elemental. |
| Baleful Mastery | {2}{B} | 🟡 | Push XXIX (doc sync): Instant. Exile target creature/planeswalker + an opp draws a card. The {1}{B} alt-cost path is omitted (only adds the "opp draws" rider which already resolves either way). |
| Igneous Inspiration | {2}{R} | ✅ | Push XXIX (doc sync): Sorcery. 3 damage to creature/PW + Learn (→ Draw 1). |
| Combat Professor | {3}{W} | 🟡 | Push XXIX (doc sync): 2/4 Cat Cleric, Flying + Vigilance. Mentor approximated as `Attacks/SelfSource → AddCounter(target attacking creature with PowerAtMost(1))`. |
| Conspiracy Theorist | {1}{R} | 🟡 | Push XXIX (doc sync): 2/1 Human Shaman. Attack-trigger rummage-into-exile-and-play and empty-hand activated are both omitted (no cast-from-exile-with-timer primitive). |
| Beaming Defiance | {1}{W} | ✅ | Push XXIX (doc sync): Instant. Target creature you control gets +2/+0 + hexproof EOT. |
| Spell Satchel | {3} | 🟡 | Push XXIX (doc sync): Artifact. `{T}: Add {C}` + `{3},{T},Sac:` returns single target IS card from gy to hand. Multi-target "any number with total MV ≤ 4" picker still pending. |
| Excavated Wall | {2} | ✅ | Push XXIX (doc sync): 0/4 Artifact Creature — Wall with Defender. ETB: gain 2 life. |
| Snow Day | {U}{R} | 🟡 | Push XXIX (doc sync): Instant. Tap + stun one creature (multi-target "up to two" collapsed to one, same gap as Vibrant Outburst). |
| Confront the Past | {3}{R} | 🟡 | Push XXIX (doc sync): Sorcery, 3-mode `ChooseMode`: reanimate PW from gy / bounce PW / 3 damage to PW (loyalty-counter-X collapses to flat 3). |
| Specter of the Fens | {4}{B} | ✅ | Push XXIX (doc sync): 3/4 Flying Specter. ETB: return creature/PW from your gy → hand. |
| Mascot Interception | {4}{R}{W} | ✅ | Push XXIX (doc sync): Instant. Gain control of target permanent EOT + Untap + Haste EOT. |
| Twinscroll Shaman | {2}{U}{R} | ✅ | Push XXIX (doc sync): 3/3 Human Wizard. Magecraft: Copy that spell via `Effect::CopySpell{what: TriggerSource}`. |
| Practical Research | {1}{G}{U} | ✅ | Push XXIX (doc sync): Sorcery. Doubles +1/+1 counters on target creature you control via `AddCounter(amount = CountersOn(target, +1/+1))`. |
| Hall of Oracles | — | ✅ | Push XXIX (doc sync): Land. `{T}: Add {C}` + `{2},{T}: +1/+1 counter on target Wizard or Fractal creature you control`. |

### Shared / multi-college

| Card | Cost | Status | Notes |
|---|---|---|---|
| Inkling Summoning | {3}{W}{B} | ✅ | Sorcery (Lesson). Creates a 2/1 white-and-black Inkling token with flying. |
| Tend the Pests | {1}{B}{G} | ✅ | Sacrifice a creature; create X 1/1 Pest tokens (X = sacrificed power); "When this dies, gain 1 life" trigger now rides on the token via SOS-VI's `TokenDefinition.triggered_abilities`. |

### Iconic / legendary (`stx::iconic` + `stx::legends`)

Cards added in the latest push that didn't fit a single college file —
each college's flagship Dragon, plus a few cross-college staples.

| Card | Cost | Status | Notes |
|---|---|---|---|
| Strict Proctor | {1}{W} | 🟡 | 1/3 Spirit Cleric, Flying. ETB-tax replacement is omitted (no replacement-effect primitive). |
| Sedgemoor Witch | {2}{B}{B} | ✅ | 3/2 Human Warlock, Menace + Ward(1) keyword. Magecraft creates a Pest token. Ward enforcement still pending — keyword tag is correct. Test: `sedgemoor_witch_magecraft_creates_pest_token`. |
| Spectacle Mage | {U}{R} | 🟡 | 1/2 Human Wizard with Prowess. Hybrid {U/R}{U/R} approximated as {U}{R}. Prowess keyword tag is correct (engine-side wiring still pending). |
| Mage Hunters' Onslaught | {2}{B}{B} | ✅ | Sorcery. Destroy target creature; draw a card. Test: `mage_hunters_onslaught_destroys_creature_and_draws_card`. |
| Galazeth Prismari | {2}{U}{R} | 🟡 | 3/4 Legendary Dragon Wizard, Flying. ETB creates a Treasure token (full real-card behaviour). The "artifacts you control are mana sources" static is still ⏳ (no `GrantActivatedAbility(applies_to)` primitive). Test: `galazeth_prismari_is_three_four_flying_dragon_with_etb_treasure`. |
| Beledros Witherbloom | {3}{B}{B}{G}{G} | 🟡 | 6/6 Legendary Demon, Flying + Trample + Lifelink. Pay-10-life mass-untap activated is ⏳. |
| Hofri Ghostforge | {2}{R}{W} | 🟡 | Push XX: 3/4 Legendary Spirit Cleric. Anthem static + exile-on-death/return-as-Spirit cycle ⏳ pending conditional-anthem static + delayed-replacement primitive. |
| Velomachus Lorehold | {3}{R}{R}{W} | 🟡 | 5/5 Legendary Dragon, Flying + Vigilance + Haste. Attack-trigger reveal-and-cast is ⏳ (cast-from-exile-without-paying primitive). |
| Tanazir Quandrix | {2}{G}{G}{U}{U} | 🟡 | 5/5 Legendary Dragon, Flying + Trample. Push XIX: ETB +1/+1-counter doubling **now wired** via `ForEach(Creature & ControlledByYou)` + `AddCounter(+1/+1, amount: CountersOn(TriggerSource, +1/+1))`. The attack-trigger toughness doubling was already wired (push prior). |
| Shadrix Silverquill | {2}{W}{B} | 🟡 | 4/4 Legendary Dragon, Flying + Double Strike. Choose-2-of-3 attack-trigger is ⏳ (no multi-mode-pick primitive). |

### Engine pieces driven by STX

- ✅ **`effect::shortcut::magecraft(effect)` helper** — bundles the
  spell-cast trigger with `cast_is_instant_or_sorcery()`, so card
  factories use a one-liner. Used by Eager First-Year and Witherbloom
  Apprentice; future Apprentices (Lorehold, Prismari, Quandrix) will
  reuse it.
- ✅ **Token death-trigger lifegain** — `TokenDefinition` now carries
  `triggered_abilities` (added in SOS push VI). The STX Pest token's
  "die → gain 1" trigger fires consistently for Pest Summoning, Tend
  the Pests, Hunt for Specimens. SOS Pest token's "attack → gain 1"
  rider also rides on every minted copy.
- ⏳ **Lesson sideboard model** — Eyetwitch, Hunt for Specimens, Pest
  Summoning all use Learn at some point. Currently approximated as
  `Draw 1`.
