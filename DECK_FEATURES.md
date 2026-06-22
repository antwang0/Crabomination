# Deck Implementation Tracker

Tracking two fully-playable decks:
- **BRG combo** (Cosmogoyf + Thud, Pact-style)
- **Goryo's Vengeance reanimator**

Both ship as the default demo match
(`crabomination::demo::build_demo_state` — P0 = BRG, P1 = Goryo's).

Done (✅) cards and engine features are elided; only remaining 🟡/⏳ work is
listed. Full per-card history is in git.

## Legend

- 🟡 partial — card exists, key behavior missing
- ⏳ todo — not yet implemented

### BRG main deck / sideboard

All ✅ and elided.

## Modern supplement (`catalog::sets::decks::modern`)

Extra Modern- and cube-playable cards. Most ride existing engine primitives;
newer batches also added small reusable ones (no-max-hand-size,
play-lands-from-graveyard, mana-doubling, ability-lock statics, Cipher, block
tax, landfall, graveyard escape/retrace, level bands, sideboard wishes,
manifest-from-hand, token Role Auras, Absorb, Cleave, reflect-prevention
shields, restricted colorless mana, multi-pick reveals, …). Each card has at
least one test in `crabomination/src/tests/modern.rs`.

All Modern-supplement cards are wired (including Karn, Scion of Urza and
Tezzeret, Cruel Captain, on real oracle text).

> **Stat-fidelity sweep (2026-06-16).** The supplement's *printed* stats were
> never audited against Scryfall and carried many synthesized errors (e.g. Grief
> `{1}{B}{B}`→`{2}{B}{B}`, Elesh Norn MoM `{3}{W}{W}`→`{4}{W}`, Riftwing
> Cloudskate `{3}{U}`→`{3}{U}{U}`). A catalog-wide sweep against a refetched real
> Scryfall cache (`scripts/audit_catalog_stats.py` + `fix_catalog_stats.py`)
> corrected **~150 costs, ~74 P/T, ~80 creature types** in `decks` (plus the
> mod_set / ths / kld / ktk / lea sets), regenerating the coupled tests; full
> suite green. Catalog-wide drift fell to **cost 2 / P-T 6 / type 8 / keyword 41**
> (from 253 / 131 / 120 / 55). So "wired" now means correct cost/P-T/type-line too
> — but several card *bodies* remain simplified approximations (the abilities, not
> the stats). The **keyword** pass fixed 13 clear bugs; the ~41 left are
> conditional/ability-modeling keywords (e.g. evasion modeled as Flying, counter-
> tax as Ward), DFC back faces, and Protection/Ward args — those need real ability
> work, not a stat tweak. Run `audit_catalog_stats.py` for the live list. Customs
> (Cosmogoyf, Crabomination) are excluded — no Scryfall truth.

## Engine features

| Feature | Status | Notes |
|---|---|---|
| Uncounterable spell flag | ✅ | `StackItem::Spell.uncounterable`, respected by `CounterSpell`. Cavern of Souls stamps casts uncounterable via mana provenance; Veil of Summer is a turn-scoped grant. |
| Mutate (CR 702.140) | ✅ | `CardDefinition.mutate` + `GameAction::CastMutate`; merges onto a non-Human host you own (`CardInstance.mutate_stack`, union definition), `EventKind::Mutated` triggers (`Value::MutateCount`, `SelectionRequirement::HasMutate`), scatters on leave, snapshot round-trip. Ikoria cycle in `decks::modern` (incl. Archipelagore — `Effect::TapUpToValue` taps a runtime-`Value` count of creatures chosen at resolution). Only the client cast-mutate UI remains — see TODO.md. |
| Gift (CR 702.165) | ✅ | `CardDefinition.gift` (`Gift.gifted_effect`) + `GameAction::CastGift` + `CardInstance.gift_promised`; promising the gift resolves the enhanced effect (which bestows the gift on an opponent) and broadens cast-time/608.2b target filters (Into the Flood Maw, Long River's Pull). `TokenDefinition.tapped` mints the tapped-Fish/Treasure gifts. Client right-click "promise gift" cast; `KnownCard.{has_gift,gift_label,gift_needs_target}`. Bloomburrow batch in `decks::gift` (10 cards) + Nocturnal Hunger upgraded. |
| Survival (CR 702.180) | ✅ | "At the beginning of your second main phase, if this creature is tapped, …" — a `StepBegins(PostCombatMain)`/`ActivePlayer` trigger under an `EntityMatches{This,Tapped}` intervening-`if` (`decks::survival`: Cautious Survivor, Defiant Survivor, Shrewd Storyteller, Savior of the Small). |
| Omen (CR 702.183) | ✅ | `CardDefinition.omen` (reuses the `Adventure` shape) + `GameAction::CastOmen` + `CardInstance.omen_casting`; the creature card is cast as its instant/sorcery Omen half and shuffles into its owner's library on resolution *or* counter (handled at the `route_to_graveyard` funnel). Client right-click "Cast the Omen"; `KnownCard.{has_omen,omen_label,omen_needs_target}`. Full Tarkir Dragon-Omen cycle (17) in `decks::omen`, seeking via `Effect::Seek` (CR 701.52 — random library pick: Roost Seek, Nesting Instinct, Divining Dive). |
| Mayhem (CR 702.187) | ✅ | `Keyword::Mayhem(cost)` + `GameAction::CastMayhem` (delegates to the flashback machinery; exile-after tail) gated on `Player.discarded_this_turn`. Spider-Man batch in `decks::mayhem`: Electro's Bolt, Sadistic Slash, Raging Goblinoids. |
| Job Select (CR 702.182) | ✅ | Equipment ETB mints a 1/1 colorless Hero token and self-attaches (living-weapon shape — `job_select_equipment`): Monk's Fist, Bard's Bow. The "is also a [class]" type-add rider is dropped (`EquipBonus` overrides types, doesn't add). |
| Tarkir: Dragonstorm (non-Omen) | ✅ | `decks::tarkir` — ~25 cards incl. the Sarkhan's Resolve / Sibsig / Stormscale / Magmatic Hellkite batch plus **Flurry** (`shortcut::flurry` — your-second-spell trigger: Cori Mountain Stalwart, Equilibrium Adept), Bone-Cairn Butcher (live `IsAttacking` deathtouch grant), Monastery Messenger (gy→library-top), Cunning Coyote (Plot), Sage of the Fang / Naga Fleshcrafter (**Renew** = graveyard-exile activated ability), Mox Jasper (Dragon-gated mana), Sky Skiff (Vehicle/Crew), Severance Priest (exile-from-hand-until-leaves), Frontline Rush, Stormplain Detainment, Strategic Betrayal, plus Omenpath to Naya (Vanishing tri-land). |
| X-cost creature side-effects | 🟡 | Thud / Burn at the Stake ride `SacrificeAndRemember` + `Value::SacrificedPower`. Casualty and Adventure ✅. |
| Sacrifice-as-cost effects | 🟡 | Thud ✅; variable-count sacrifice ✅ (`Effect::SacrificeAnyNumber`); flashback-with-additional-cost ✅ (Lava Dart, Dread Return). |

## Plan

Work top-down; each phase unlocks more behavior:

1. **Catalog stubs** — correct cost/types/P-T/keywords, effects = `Noop` where
   unsupported. Both decks playable as bodies.
2. **Wire `demo.rs`** for the singleplayer match (P0 = BRG, P1 = Goryo's).
3. **Tractable engine features** unlocking multiple cards: alternative pitch
   costs, shock/surveil/fastland ETB choices, Convoke/Converge.
4. **Card-specific features:** Pact upkeep costs, Rebound, Goryo's exile-at-EOT,
   Atraxa reveal-and-sort, static effects, counter-an-ability.
5. **Opening-hand effects** (Chancellor, Leyline, Gemstone Caverns, Serum
   Powder) — need pre-game mulligan-window machinery.

When promoting a card, flip its dependent engine-feature row too.
