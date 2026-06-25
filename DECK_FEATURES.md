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

`catalog::sets::decks::recent` adds recent-set staples (MH3/BLB/DSK/OTJ/FDN/…)
— Questing Beast, Vaultborn Tyrant, Emberheart Challenger, Eldrazi Linebreaker,
Beza, No More Lies, Tyvar's Stand, Stock Up, Gird for Battle, … each with a
test in `tests/recent.rs`. This batch added the fixed-threshold evasion keyword
`CantBeBlockedByPowerAtMost`, fixed `YourControl` combat-damage triggers firing
for the dealing creature itself, and the `AnOpponentHasMoreCardsInHand`
predicate. Later batches added the card-intrinsic target-conditional cost
reduction (`self_cost_reduction_if_target` — Ride's End's "{3} less if it
targets a tapped permanent"), made the bot prefer paying Offspring, and wired
the client right-click "cast with Kicker/Offspring" path (`CastSpellKicked`).
An Innistrad (MID/VOW) batch added **Coven** (`Predicate::CovenActive` +
`coven_active` view field + "✸ coven" HUD chip), the **day/night transition
trigger** (`EventKind::DayNightChanged` — Brimstone Vandal), the **shares-card-
type** trigger (`Predicate::SharesCardTypeWithExiledBySource` — Cemetery
Gatekeeper/Protector), bound `CardMilled`'s trigger subject to the milled card,
taught `MoveAllCounters` to read a dead source's death-LKI counters, and fixed
`fire_spell_cast_triggers` to honor `once_per_turn` (Whispering Wizard). A later
MID/VOW wave added `StaticEffect::GraveyardInstantsSorceriesHaveFlashback`
(Lier — graveyard I/S gain flashback = mana cost, wired into the flashback-cast
path + graveyard view) and fixed `blocker_can_block_attacker` to reject Decayed
creatures (CR 702.147) so the UI/bot no longer offer them as blockers. A
counter/aristocrat/aggro wave added `StaticEffect::ExtraCounterAllKinds` (Winding
Constrictor — +1 to any counter on your creatures), the conditional combat-gate
keywords `CantAttackOrBlockUnlessHandSizeAtMost(n)` (Hazoret) and
`CantAttackOrBlockUnlessDelirium` (Patchwork Beastie, via `delirium_active`), and
the `pick_reach_burn` bot heuristic (fire "deal N to each opponent" abilities for
lethal).

`catalog::sets::decks::recent2` adds a second wave of staples (tests in
`tests/recent2.rs`). Engine primitives this wave: equip-granted *observer*
triggered abilities (`triggers_on_equipment == false` folded into the
battlefield dispatch — Tarrian's Soulcleaver), `Keyword::Poisonous` (CR 702.70,
reuses the Toxic combat-poison path), an `all_players` flag on
`SelfCostReducedPerCreatureAttackedThisTurn` (Witchstalker Frenzy), and
auto-target coverage for `CreateTokenAttachedTo` / graveyard-targeting
`GrantFlashbackThisTurn`.

`catalog::sets::decks::recent3` adds a third wave (tests in `tests/recent3.rs`):
Solphim, Atraxa, Deathrite Shaman, Grand Abolisher, Sundering Titan, Arcane
Laboratory, the color-hoser destroy-alls (Flashfires/Tsunami/Boiling Seas/
Shatterstorm/Anarchy), Creeping Mold, Liliana's Caress, Winter Orb, Choke, the
any-color mana rocks (Manalith/Darksteel Ingot/Cultivator's Caravan/Spinning
Wheel), Hurricane // Squall Line, Staff of Nin, Ivory Tower, Viridian Shaman,
Caustic Caterpillar, Noxious Revival, Bane of Progress, Ramunap Ruins. New engine
primitives: `StaticEffect::DoubleNoncombatDamageToOpponents` (Solphim — a
noncombat-only damage doubler in the `deal_damage_to_from` funnel),
`StaticEffect::OpponentsCantActDuringYourTurn` (Grand Abolisher — cast + A/C/E
ability lock), and `Effect::DestroyLandOfEachBasicType` (Sundering Titan). The
bot's `pick_reach_burn` now recognises each-opponent drain nested in
`Seq`/`ChooseMode` activations.

`catalog::sets::fin` is a new Final Fantasy (FIN) set module (tests in
`tests/fin.rs`): Iron Giant, Sazh's Chocobo, Sephiroth's Intervention, Cactuar,
Magitek Armor, Chocobo Racetrack, Malboro, Sephiroth Planet's Heir, Aerith
Gainsborough, Phoenix Down. All ride existing primitives (landfall, ETB,
end-step self-bounce, Vehicle/Crew, Landcycling, mass −X/−X, lifegain/death
counter-distribution, modal reanimate).

`catalog::sets::decks::recent8` is an eighth staples wave (tests in
`tests/recent8.rs`) built around three brand-new keyword actions:
**earthbend N** (`Effect::Earthbend` — CR 701.66: target land you control
becomes a 0/0 hasty land creature with N +1/+1 counters and a
`WhenCardLeavesBattlefield` return-tapped rider; Badgermole/Cub, Earthbending
Student, Earth Village Ruffians, Earthbender Ascension), **airbend**
(`Effect::Airbend` — CR 701.65: exile + a never-expiring `WhileExiled` may-play
grant stamped with a {2} alt-cast cost; Airbending Lesson, Aang, Airbender
Ascension, Whirlwind Technique, Glider Staff), and **blight N**
(`Effect::Blight` — CR 701.68: the controller puts N -1/-1 counters on a
creature they control; Blighted Blackthorn, Chaos Spewer, Boggart Mischief).
Plus rideable commons (Corrupt Court Official, Jeong Jeong's Deserters,
Forecasting Fortune Teller, Pretending Poxbearers, Merchant of Many Hats, Yuyan
Archers, Platypus-Bear, Compassionate Healer, Fire Nation Soldier).

`catalog::sets::decks::recent9` is a ninth wave (tests in `tests/recent9.rs`)
reusing those three bending/blight primitives on more Avatar/Lorwyn cards (Haru,
Avatar Enthusiasts, Aang Airbending Master, Sinister Gnarlbark, Dream Seizer,
Sourbread Auntie, Shadow Urchin) plus Ally-tribal, prowess, and second-draw
payoffs (Knowledge Seeker, Otter-Penguin). It also hardened the existing
"draw your Nth card each turn" triggers (Mischievous Mystic + two Modern-set
cards) with `once_per_turn` — a multi-card draw (Divination) leaves the running
draw count at N for *several* CardDrawn events at once, which used to fire those
payoffs once per drawn card instead of once per turn (CR 603.3d).

`catalog::sets::decks::recent10` is a tenth wave (tests in `tests/recent10.rs`)
of simple Avatar/Lorwyn commons on existing primitives — ETB value (Glider
Kids scry, Messenger Hawk Clue, Ostrich-Horse mill-then-grab-land, Rowdy
Snowballers tap), token-makers (Treetop Freedom Fighters), prowess (Iguana
Parrot), and sacrifice-/noncreature-cast counter payoffs (Pirate Peddlers,
Boar-q-pine).

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
| Mayhem (CR 702.187) | ✅ | `Keyword::Mayhem(cost)` + `GameAction::CastMayhem` (delegates to the flashback machinery; exile-after tail) gated on `Player.discarded_this_turn`. The "if the mayhem cost was paid" rider now works via `CardInstance.cast_via_mayhem` → `Predicate::SpellWasMayhem` (Sandman's Quicksand). Spider-Man batch in `decks::mayhem`. |
| Harmonize (CR 702.180) | ✅ | `Keyword::Harmonize(cost)` + `GameAction::CastHarmonize` — graveyard recast; optionally tap one creature you control to reduce the total cost by generic mana = its power; exile-after (flashback tail). Bot + graveyard-browser badge. `decks::tarkir`: Channeled Dragonfire, Unending Whisper, Ureni's Rebuff, Wild Ride, Mammoth Bellow. |
| Freerunning (CR 702.179) | ✅ | `AlternativeCost.condition: DealtCombatDamageToPlayerThisTurn` (`Player.dealt_combat_damage_to_player_this_turn`, set at the combat-damage-to-player choke point). ACR batch in `decks::freerunning` (10 cards). "With an Assassin or commander" approximated as "with any creature". |
| Web-slinging (CR 702.188) | ✅ | Modeled on the alt-cost primitive (`AlternativeCost.mana_cost` + `return_to_hand` of one tapped creature). `decks::webslinging`: Spider-Man Web-Slinger, Amazing Spider-Girl, Silk, Spider-Man India. The "if cast using web-slinging" provenance riders are deferred (TODO.md). |
| Job Select (CR 702.182) | ✅ | Equipment ETB mints a 1/1 colorless Hero token and self-attaches (living-weapon shape — `job_select_equipment`): Monk's Fist, Bard's Bow. The "is also a [class]" type-add rider is dropped (`EquipBonus` overrides types, doesn't add). |
| Tarkir: Dragonstorm (non-Omen) | ✅ | `decks::tarkir` — ~110 cards. Khans wedge tri-lands (`tri_land`), Monuments (ETB basic tutor + sac payoff), Devotees (`OfColors` once-per-turn mana), the Exhale "behold a Dragon" cycle (Dragon-control rider), plus Formation Breaker (`CantBeBlockedByPowerLess`), Krotiq Nestguard (`AttackDespiteDefenderThisTurn`), Snowmelt Stag (`SetBasePtIf`). **Flurry** (`shortcut::flurry`: Cori Mountain Stalwart, Monk of the Open Hand, Jeskai Devotee, Wingblade Disciple, Poised Practitioner, Devoted Duelist, Wayspeaker Bodyguard), **Mobilize** N (`shortcut::mobilize`) + **Mobilize X** (`shortcut::mobilize_value` — Avenger of the Fallen, Dalkovan Packbeasts, Nightblade/Shock Brigade, Reigning Victor), **Renew** = graveyard-exile activated ability incl. keyword-counter grants (Champion of Dusan/Sagu Pummeler/Qarsi Revenant/Alchemist's Assistant + Agent of Kotis, Adorned Crocodile, Lasyd Prowler, Constrictor Sage), Bone-Cairn Butcher, Sage of the Fang / Naga Fleshcrafter, Mox Jasper, Sky Skiff, Severance Priest, Omenpath to Naya. |
| The Ring tempts you (CR 701.54) | ✅ | `Effect::RingTempts` + `Player.{ring_temptations,ring_bearer}`; the four cumulative emblem abilities ride the level (can't-be-blocked-by-greater-power, attack-loot, blocked-creature `Effect::SacrificeAtEndOfCombat`, combat-damage drain) **plus the level-1 "Ring-bearer is legendary" rider (CR 701.54c)** via a synthetic `Modification::AddSupertype` layer-4 effect. `EventKind::RingTempted` powers "choose a Ring-bearer" payoffs. `decks::ltr` (~60 cards: Birthday Escape, Call of the Ring, Bilbo, Frodo Baggins, Samwise Gamgee, Quickbeam, Mirror of Galadriel, Olog-hai Crusher, Glóin, Nazgûl, Gandalf's Sanction, …). Bearer auto-picked (highest power); per-player UI choice is a TODO.md follow-up. |
| X-cost creature side-effects | 🟡 | Thud / Burn at the Stake ride `SacrificeAndRemember` + `Value::SacrificedPower`. Casualty and Adventure ✅. |
| Sacrifice-as-cost effects | 🟡 | Thud ✅; variable-count sacrifice ✅ (`Effect::SacrificeAnyNumber`); flashback-with-additional-cost ✅ (Lava Dart, Dread Return); reflexive "you may sacrifice X; if you do, …" ✅ (`Effect::MaySacrifice` — Bloodcrazed Socialite, Gut). |

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
