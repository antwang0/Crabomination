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
```

It has two independent passes:

1. **Structural** (comment-free, authoritative): serde-walks every card's
   effect tree and flags *dead modes* (a `ChooseMode`/`ChooseN`/`Escalate` arm
   that resolves to `Noop`/empty) and *dead abilities* (a triggered / activated
   / loyalty ability with an empty effect). An empty arm is a bug regardless of
   what the card "should" do.
2. **Comment scan**: lists every `pub fn … -> CardDefinition` factory whose doc
   comment carries an approximation marker (`approximation`, `modeled as`,
   `omitted`, `stub`, `body only`, `collapsed`, …). As of the last run: **470
   of 8092 factories** carry such a note.

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

A selectable mode that resolves to nothing. Needs human triage — a `Noop` mode
is also the idiom for a deliberate "you may … (or decline)" option.

| Card | Location | Verdict |
|---|---|---|
| Sublime Epiphany ✓ | stx/extras_02.rs:668 | **Fixed.** All five modes now real: mode 1 → `Effect::CounterAbility`, mode 3 → `Effect::CreateTokenCopyOf`. |
| Elite Interceptor ✓ | sos/mdfcs.rs:181 | **Not a gap.** Arm #2 (`Noop`) is the deliberate "decline" half of "you may tap or untap"; the draw is unconditional. |

> Notes: (1) an earlier manual pass mislabeled the Sublime Epiphany finding as a
> card called "Persist" — the structural auditor is the source of truth. (2) The
> Elite Interceptor flag shows even the structural pass needs triage: a `Noop`
> mode is ambiguous between "unimplemented" and "intentional decline."

---

## Missing-primitive buckets (the engineering view)

Fix the primitive → fix the whole cluster.

### 1. No multi-target / "up to N targets" / "divided as you choose" prompt
Mostly **solved** via `Effect::ApplyToTargets` (up to N) and
`Effect::DealDamageDivided` (divide). Done: Elemental Expressionism (bounce ≤2),
Daring Diversion (4 divided). Remaining: Fireball (any-number split) · Return to
Dust (conditional 2nd target) · Yosei (taps all → up-to-5) · Rag Dealer ·
Skullsnatcher · Pull from the Grave · Rabid Attack.

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
- ✅ **transform-and-cast-from-graveyard** — `GameAction::CastSpellBack` now hops a permitted graveyard card into hand for the back-face cast pipeline (Muldrotha idiom), gated by a one-shot `CardInstance::may_cast_back_from_graveyard` flag set by `Effect::GrantCastBackFromGraveyard`. Pestilent Cauldron's sac ability grants it; Restorative Burst is then castable from the graveyard.

### 4. No "controller-of-target" / "that player" actor (forces each-opponent / you)
~~Generous Gift~~ ✅ **FIXED** (now mints the 3/3 Elephant for the target's controller via `CreateToken { who: ControllerOf(Target(0)) }`, created before the Destroy — the "no primitive" doc note was stale) · Harsh Annotation · Kemuri-Onna ·
Hellrider · Emeritus of Truce // STP · Channeled Force · several CHK Ninjas.

### 5. No "first/Nth spell this turn" / "no card drawn this turn" gate (over-triggers)
Thalia, Heretic Cathar · Quandrix Mathwarden / Spellmage / Streamcaller ·
Prismari Mage-Mentor · Frostpyre Arcanist · Quandrix Field Trip.

### 6. No additional-cost-with-life/exile · no Phyrexian mana (riders dropped / folded into resolution)
Deep Analysis (pay 3 life) · Resurgent Belief (exile a gy card) · Necrotic Fumes ·
Final Payment · Birthing Pod & Mox Diamond ({G/P}, land-discard) · Vicious Rivalry ·
Mana Vault (pay {4} skip) · Channel.

### 7. Whole keyword mechanics unmodeled (each = a cluster)
- **Learn** → modeled as Draw 1 (Reduce // Rubble, Mascot Interpretation, the Lessons cycle, Quandrix Field Trip).
- **Adventure** → half omitted (Callous Sell-Sword).
- **Delve** → cast at full cost (Tasigur, Magmatic Sinkhole).
- **Splice onto Arcane** → omitted (Desperate Ritual).
- **Awaken / sac-Spawn alt-cast** → omitted (Birthing Hulk, Hand of Emrakul, OGW Eldrazi).
- **Boast payoff** → omitted (Dragonkin Berserker).
- **Channel land cost-reductions / land-search** → dropped (decks/lands.rs cluster).
- **Free-with-commander alt-cost / color-identity** → dropped (Fierce Guardianship, Deflecting Swat, Command Tower, Jeska's Will).
- **Wish (cast from outside the game)** → omitted (Spawnsire of Ulamog).

### 8. No copy-token / "you may choose new targets on the copy" primitive
Quandrix Snapcaster · Prismari Maestro · Echocasting Symposium · Lorehold Tomb Robber ·
Spark Double (no planeswalker-copy) · Prismari, the Inspiration · Mirror Image (legendary-strip).

### 9. No type/color rewrite on existing permanents (layers 4–5)
Kasmina's Transmutation · Fractalize · Lorehold Reclamation (Spirit-typing) ·
Fractal-token color/type riders.

### 10. "Rest on bottom of library" approximated as "leave on top" / "to graveyard"
**Stale** — `LookPickToHand { rest_to_graveyard: false }` already bottoms the
rest. Augur of Bolas ✅ (also got its instant/sorcery filter) and Sea Gate Oracle
✅ verified + de-stale-commented. Remaining to spot-check: Geometer's Arthropod ·
Paradox Surveyor · Conjurer's Bauble.

---

## Highest-priority single-card gaps

### Body-only stubs — entire signature ability missing (all ✓ code-verified)
| Card | Location | Missing |
|---|---|---|
| ~~Vendilion Clique~~ ✅ **FIXED** | mod_set/creatures.rs:3852 | ETB hand disruption — wired via new `Effect::BottomChosenFromHandAndDraw` (look at hand → choose nonland → bottom + draw). Targets `EachOpponent` (1v1-faithful; self-cast mode pending player-targeting on triggers). Tests: `vendilion_clique_is_3_1_legendary_flash_flying`, `…_etb_bottoms_chosen_card_and_target_draws` |
| ~~Torrential Gearhulk~~ ✅ **FIXED** | mod_set/creatures.rs:3875 | ETB "cast instant from graveyard" — wired via `CastWithoutPayingImmediate { Graveyard, exile_after }` (tests: `torrential_gearhulk_is_5_6_artifact_flash`, `…_etb_casts_instant_from_graveyard_and_exiles_it`) |
| ~~Phyrexian Obliterator~~ ✅ **FIXED (1v1-approx)** | mod_set/creatures.rs:4006 | Damage-retaliation now wired: `DealtDamage`/`SelfSource` → `Sacrifice { count: TriggerEventAmount }`. Sacrificer is `EachOpponent` (faithful in 1v1; "that source's controller" can't be read — `GameEvent::DamageDealt` carries no source). Doc P/T corrected 5/8→5/5. Tests: `phyrexian_obliterator_is_5_5_trample`, `…_damage_forces_opponent_to_sacrifice_that_many` |
| ~~Alesha, Who Smiles at Death~~ ✅ **FIXED** | ktk/mod.rs | Attack trigger now wired: `on_attack(MayPay { {W/B}{W/B} → Move(target gy creature pow≤2 → battlefield tapped) + JoinCombatAttacking(LastMoved) })`. New `Effect::JoinCombatAttacking` puts the reanimated creature into combat attacking (CR 508.3a); `Move→Battlefield` + `MayPay` now bias the trigger auto-targeter to the graveyard. Test `cr_508_3a_alesha_reanimates_tapped_and_attacking`. |

### Wrong-effect substitutions — implemented card is functionally a different card
| Card | Location | Substitution |
|---|---|---|
| ~~Silverquill Penkeeper~~ ✅ **FIXED** | silverquill.rs:14312 | now `magecraft(Effect::Discard { EachOpponent })` — matches its own documented "each opponent discards" intent (was Drain 1) |
| ~~Silverquill Wordweaver~~ ✅ **FIXED** | silverquill.rs:14653 | now `etb(Effect::Discard { EachOpponent })` (was Drain 2) |
| ~~Witherbloom Necromancer~~ ✅ **FIXED** | witherbloom.rs:10706 | now `on_other_dies(MayPay { {1} → Move(TriggerSource → battlefield) })` — real reanimate-the-just-died-creature (was Drain 1), same mechanism as Minion's Return |
| Channel | modern.rs:11313 | "pay 1 life instead of {1} EOT" → one-shot lose-1-add-{C} |
| ~~Echocasting Symposium~~ ✅ | sos/sorceries.rs | already on `CreateTokenCopyOf` (doc was stale) |
| ~~Rush of Knowledge~~ ✅ | stx/mono.rs | `Value::HighestManaValueAmong` (was hardcoded draw 4) |

Note: Silverquill Penkeeper/Wordweaver and Witherbloom Necromancer above are
**synthesized** fabricated-name cards (only `_b###` factories exist), so the
"substitution" is moot — there's no real oracle to match.

### Dropped static abilities on legendaries (whole side absent)
| Card | Location | Missing |
|---|---|---|
| ~~Callaphe, Beloved of the Sea~~ ✅ **FIXED** | thb.rs | "{1} tax on opponents' spells targeting your creatures/enchantments" now wired via the existing `StaticEffect::TaxOpponentSpellsTargeting` (the stale doc claimed `extra_cost_for_spell` couldn't read the cast target — Jubilant Skybonder already proved otherwise). Test `callaphe_taxes_opponent_spells_targeting_your_permanents`. |
| Siona, Captain of the Pyleas | thb.rs:6134 | "Aura becomes attached → make a 1/1 Soldier" (no aura-attach event) |

### Verified-but-overrated (real gaps, but 1v1-equivalent or strictly-better — MED, not HIGH)
| Card | Location | Note |
|---|---|---|
| Oko, Thief of Crowns ✓ | modern.rs:12238 | +2 Food→gain 3 life; −5 exchange→one-way gain control |
| Spell Queller ✓ | modern.rs:15114 | counters instead of exile-until-LTB (opponent can't recast) |
| Generous Gift ✓ | modern.rs:8226 | victim gets no 3/3 Elephant (strictly stronger) |

### Other notable HIGH (doc-derived — confirm)
Veil of Summer (✅ now wires the draw gate + uncounterable + lifegain-lock;
only the hexproof-from-blue/black rider remains) · Approach of the Second Sun
(✅ wins via `WinGame` — doc was stale) · ~~Heroic Intervention~~ ✅ (code
already granted **both** hexproof + indestructible — only the in-code comment was
stale; test now proves granted hexproof blocks opponent targeting) ·
Fractal Tender (both triggers omitted) ·
Pestilent Cauldron / Wandering Archaic (back faces).

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
- **Optional "may pay" → mandatory** (~10): Witherbloom Pestcaster ({B}{G}),
  Aura Shards, Leonin Snarecaster, Heated Argument, Pursue the Past, …
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
