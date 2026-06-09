# Cube Implementation Tracker

Tracking the work to make this Modern-style cube fully playable. The cube
is built around a mix of efficient creatures, interaction, value engines,
and combo lines — see the maybeboard at the end of this file.

The catalog already implements the cards used by the BRG / Goryo's demo
decks (`DECK_FEATURES.md` is the source of truth there); some of those
cards overlap with the cube. Most of the cube is still ⏳. **Done (✅)
cards and engine features have been elided** — only the remaining 🟡/⏳
work is listed below.

## Legend

- 🟡 partial — exists with a simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

### White

| Card | Status | Notes |
|---|---|---|
| Tempt with Bunnies | ⏳ | Tempting offer (chain-creating) — needs multi-player choice primitive. |

### Blue

| Card | Status | Notes |
|---|---|---|
| Gather Specimens | ⏳ | Replace creature ETB control-shift. Replacement effect primitive. |
| Mirrorform | ⏳ | Aura + clone target. |

### Black

| Card | Status | Notes |
|---|---|---|
| Mutated Cultist | ⏳ | Mutate primitive needed. |
| Unholy Annex // Ritual Chamber | ⏳ | DFC enchantment land. |

### Red

| Card | Status | Notes |
|---|---|---|

### Green

| Card | Status | Notes |
|---|---|---|
| Shifting Woodland | ⏳ | Delirium-gated continuous "becomes a copy of a gy permanent" (layer-1 loop). |

### Artifacts & Planeswalkers (mono / colorless)

| Card | Status | Notes |
|---|---|---|
| Agatha's Soul Cauldron | ⏳ | Borrow activated abilities of *exiled* creatures (gated on a +1/+1 counter). The graveyard-borrow sibling now ships (`StaticEffect::HasActivatedAbilitiesOfGraveyardCreatures`, Necrotic Ooze ✅); Cauldron needs the exile-zone + counter variant. |
| The Mightstone and Weakstone | 🟡 | {5} Artifact; ETB `ChooseMode` (Draw 2 / target creature -5/-5 EOT); `{T}`: Add {C}{C}. Meld/assemble omitted. |

### Multicolor

| Card | Status | Notes |
|---|---|---|
| Ashiok, Nightmare Weaver | 🟡 | Push (claude/modern_decks batch 102): {1}{U}{B} 3-loyalty Planeswalker. **+2**: target opponent mills 3 (the "exiled with Ashiok" linkage is engine-wide ⏳ — milled cards land in opp graveyard). **-1**: Exile target opp creature (the "create a copy" half collapses). **-10**: Approximated as `WinGame { You }` (the "each opp draws 7 from exile" plinker ultimate is dropped). Tests: `ashiok_nightmare_weaver_plus_two_mills_opponent_three`, `ashiok_nightmare_weaver_minus_one_exiles_creature`. |
| Sorin, Grim Nemesis | 🟡 | Push (claude/modern_decks batch 102): {4}{B}{B} 6-loyalty Planeswalker. **+1**: Draw 1 + Lose 3 life (approximation; reveal/MV-life-loss/conditional-token chain dropped). **-X**: ping (the X-cost loyalty path uses `Value::XFromCost` against creature/PW + 1 gain life). **-9**: drain 10 from each opponent (the printed "X = cards in opp's graveyard" scaling collapses). Tests: `sorin_grim_nemesis_plus_one_draws_and_loses_three_life`, `sorin_grim_nemesis_minus_nine_drains_each_opponent`. |
| Saheeli Rai | 🟡 | {1}{U}{R} 3-loyalty Planeswalker. **+1**: Scry 1 + ping each opponent **and each planeswalker they control** for 1 (now faithful — the PW half rides `EachPermanent(Planeswalker & ControlledByOpponent)`, CR 120.3c). **-2**: Create a token copy of target friendly creature/artifact, grant haste, delay-trigger Exile at next end step. **-7**: Same body fired twice (the emblem-recurring "each end step" auto-recur is approximated). Tests: `saheeli_rai_plus_one_pings_each_opponent`, `saheeli_rai_minus_two_creates_haste_copy`. |
| Zirda, the Dawnwaker | ⏳ | Companion + activated-cost reduction. Needs Companion primitive. |
| Lonis, Genetics Expert | 🟡 (was ⏳) | Push (claude/modern_decks batch 103): {1}{G}{U} Legendary 2/2 Otter Detective. ETB trigger on other creatures you control creates a Clue token (the printed "investigate" rider via `clue_token()`). The "Sacrifice X Clues: target opp reveals top X cards" activated ability is engine-wide ⏳ (no per-activation X-cost prompt yet). Test: `lonis_genetics_expert_creates_clue_when_other_creature_enters`. |
| Tamiyo, Collector of Tales | 🟡 | Push (claude/modern_decks batch 102): {2}{G}{U} 4-loyalty Planeswalker. **-2**: Return target card from gy → hand. **-3**: Search library → hand (the "same name as a card in target opponent's graveyard" filter is engine-wide ⏳ — falls back to `Any`). **-7**: Draw 4 (the "distinct nonland types in gy" scaling drops). The static's **sacrifice** half now ships (`StaticEffect::OpponentsCantMakeYouSacrifice`, shared with Sigarda); the "can't make you discard" half is still engine-wide ⏳. Test: `tamiyo_collector_minus_two_returns_card_from_graveyard`. |
| Dakkon, Shadow Slayer | 🟡 | WUB Legendary Planeswalker. +1: Surveil 2. -3: Exile target creature. -6: emblem (`Effect::CreateEmblem`, approximated as "draw a card at your upkeep"). Body fully wired; only the emblem's exact text is an approximation. |
| Rediscover the Way | ⏳ | TBD. |
| Shiko and Narset, Unified | ⏳ | TBD. |
| Muldrotha, the Gravetide | ⏳ | Cast from graveyard each turn (one of each card type). |

### Lands

| Card | Status | Notes |
|---|---|---|
| Power Depot | 🟡 | Artifact Land, enters tapped, `{T}: Add {C}` + `{T}: Add one mana of any color`. The "spend only on artifact spells/abilities" mana restriction and Modular 1 are dropped. Test: `power_depot_enters_tapped_and_fixes_mana`. |

## Engine features needed

The cube draws on a much wider mechanic set than the BRG / Goryo's pair.
This list is the work needed to bring most of the cube online; the engine
features already implemented (Pact / Flashback / Convoke / Rebound / etc.)
are listed in `DECK_FEATURES.md`.

| Feature | Status | Cards depending on it |
|---|---|---|
| Equipment + equip-cost activated ability | 🟡 | `GameAction::Equip` + `equipped_bonus` ship Shuko, Lavaspur Boots (✅). Board-scaled equip bonus ships via `EquipBonus.scale` — Nettlecyst ✅. Equipment-granted triggered abilities ship via `EquipBonus.triggered_abilities` (CR 702.6e; `DealsCombatDamageToPlayer` SelfSource fires off the equipped creature, damaged player bound to `Target(0)`) — Sword of Body and Mind ✅. Lion Sash ✅ (counter-on-self scaled bonus + Reconfigure CR 702.151 — `Keyword::Reconfigure` reuses the equip path and strips Creature-ness while attached). Living weapon ✅ (Nettlecyst, Batterskull — ETB mint-a-Germ-and-attach). Reconfigure *unattach* ✅ (`Reconfigure { target: None }` clears `attached_to`, restoring creature-ness; test `reconfigure_unattach_restores_creatureness`). |
| Adventure (cost-mode duality) | ✅ | CR 715 — `CardDefinition.adventure` + `GameAction::CastAdventure`/`CastAdventureCreature`. Virtue of Loyalty (enchantment // instant) ships on it alongside the creature adventures. |
| Mutate | ⏳ | Mutated Cultist, Mutable Explorer. |
| Storm count + cast-from-top | 🟡 | Storm count is wired (`Value::StormCount`, `Keyword::Storm` auto-copies on cast, per-turn `spells_cast_this_turn`) and `Effect::CopySpell` exists. Exile-top-and-grant-free-play (`Effect::ExileTopAndGrantMayPlay`) ships Robber of the Rich and Mind's Desire (Storm × exile-top + may-play-free). Energy-gated free cast-from-exile (`Effect::ExileTopMayPayEnergyToCast`) ships Amped Raptor ✅. |
| Ninjitsu | ⏳ | Fallen Shinobi (any future ninjas). |
| Soulbond | ✅ | `Keyword::Soulbond` + `CardInstance.soulbond_partner` + `CardDefinition.soulbond_bonus` (`SoulbondBonus`: P/T, keywords, granted activated **and** triggered abilities). Pairs auto-resolve on ETB (`apply_soulbond_pairing`); the bonus rides both members as continuous effects and breaks on leave. Wolfir Silverheart, Wingcrafter, Nightshade Peddler, Trusted Forcemage, Hanweir Lancer, Silverblade Paladin, Nearheath Pilgrim, Deadeye Navigator (self-flicker), Tandem Lookout (combat-damage draw). |
| Companion (deck-construction restriction + start-side mana cost) | ⏳ | Zirda, the Dawnwaker. |
| Saga lore counters | ✅ | `saga_chapters` (History of Benalia, The Eldest Reborn). DFC sagas (transform-on-final-chapter) still ⏳. |
| Transforming DFCs | ✅ | CR 712 — `Effect::Transform` + `CardInstance.{transformed,front_face}` + `EventKind::Transformed`. Swaps the active face in place (counters/tapped/attachments persist), round-trips through snapshots. Ships Concealing Curtains // Revealing Eye, Delver of Secrets // Insectile Aberration, The Everflowing Well // The Myriad Pools (descend-8 upkeep flip). Remaining ⏳: Daybound/Nightbound auto-flip, DFC sagas, manifest/disguise face-down. |
| Hideaway lands | ✅ | CR 702.76 — `Effect::Hideaway { count }` looks at the top N, exiles the best face down stamped `exiled_with = source`, bottoms the rest; `Selector::CardExiledWithSource` + `CastWithoutPayingImmediate` play it later. Shelldock Isle ✅. Remaining ⏳: the Lorwyn cycle's harder gates (attacked-with-3, lost-7-life, total-power-8). |
| Impending (CR 702.183) | ✅ | `Keyword::Impending(n)` + `AlternativeCost.impending`: cast for the impending cost → enters with N time counters, isn't a creature (layer-4) until they tick off one per end step. All five Duskmourn Overlords ship with their enters-or-attacks triggers. |
| Verge / surveil land family expansion | ✅ | All five enemy/allied `*verge` lands ship via `verge_land`. Horizon-canopy cycle complete (`horizon_land`; all six). All ten MKM surveil lands ship via `dual_land_with` + `etb_tap_then_surveil_one`. |
| ETB-replacement effects (suppress entirely) | 🟡 | "Exile non-cast nontoken creature instead" wired (`StaticEffect::ExileNontokenCreaturesNotCast`, Containment Priest ✅). Creature-ETB / death **trigger** suppression ships via `StaticEffect::SuppressCreatureEtbTriggers { also_dies }` (Torpor Orb, Tocatli Honor Guard, Hushbringer ✅). Remaining: Gather Specimens (steal-instead). |
| Spell-tax statics ("costs {1} more", "costs at least {3}") | ✅ | Damping Sphere (`AdditionalCostAfterFirstSpell`), flat `AdditionalCost`, and the Trinisphere "minimum cost" floor (`SpellCostFloor`, untapped-gated) all ship. Elite Spellbinder reuses the existing tax static. |
| "Cast spells without paying mana" static | 🟡 | Omniscience ✅ (`StaticEffect::CastHandSpellsFree`); Aluren ✅ (`StaticEffect::AnyoneCastsCheapCreaturesFree { max_mv }` — any player free-casts MV≤3 creatures at flash). Remaining: Maelstrom Archangel (combat-damage variant). |
| Name-a-card primitive | 🟡 | `Effect::NameCard` + `Decision::NameCard` + `CardInstance.named_card` ship Pithing Needle / Phyrexian Revoker (ETB stamps a name; `activate_ability` suppresses non-mana abilities of matching sources). Same-name exile (Crumble to Dust) is wired via `Effect::ExileSameNameAsTarget`. Remaining consumers: reveal-until-find (Spoils of the Vault), hand-discard-by-name (Cabal Therapy). |
| Face-down permanents (morph / manifest) | 🟡 | CR 708 — `CardInstance.face_up_def` stashes the real card while `definition` is the vanilla 2/2 (`facedown_creature_definition`); turns face up on leaving the battlefield (CR 708.10). `Effect::Manifest` / `Effect::ManifestDread` (CR 701.34 / 702.166) put library cards onto the battlefield face down; `GameAction::TurnFaceUp` pays the Morph/manifest cost and fires `EventKind::TurnedFaceUp` (CR 708.5/708.8). Hauntwoods Shrieker ✅. Remaining ⏳: the full Morph **cast-face-down** spell path and Disguise/Cloak. |
| Token-copy of permanent | 🟡 | Populate ✅ (`Effect::Populate`, CR 701.32 — Growing Ranks). `CreateTokenCopyOf` ✅, with a `non_legendary` rider (CR 707.2e — strips the copy's supertypes; Helm of the Host ✅). Mockingbird/Phantasmal Image clone-enter ✅ via `BecomeCopyOf`. Remaining: a true *continuous* "becomes a copy" layer-1 loop (Mirrorform aura). |
| Multi-pick decisions over revealed library cards | 🟡 | Atraxa Draw-4 stand-in is wired. Reveal-and-sort by card type, Dig Through Time, Mind's Desire all need a richer multi-pick decision. |
| Investigate + Clue token | 🟡 | Clue tokens ship (`clue_token()`; Tireless Tracker, Lonis create them). **Map tokens** now ship too (`map_token()` — CR 111.10s explore-token: {1},{T},Sac → target creature you control explores; Loot, the Pathfinder mints one). The "sacrifice a Clue" payoff ✅ rides `sac_other_filter: HasArtifactSubtype(Clue)` (Tireless Tracker's `{2}, Sac a Clue: Draw` + its sacrifice-a-Clue trigger). Remaining ⏳: a variable-X "sacrifice X Clues" prompt (Lonis's second ability). |
| Landfall trigger | 🟡 | Bloodghast wired via the new `EventScope::FromYourGraveyard` (graveyard-source `LandPlayed` trigger). Standard battlefield-side landfall (Omnath) still uses the existing `LandPlayed` + `YourControl` path; both are functional. |
| Loyalty abilities w/ static | 🟡 | Teferi works fully; loyalty-set effects ship via `Effect::SetLoyalty` (CR 606 — Geyadrone Dihada's +1 reset-when-behind ✅). Variable `-X` loyalty ✅ via `LoyaltyAbility.x_cost` (player picks X≤loyalty, body reads `Value::XFromCost`; Kasmina's -X Fractal). Needs further variants (Ashiok exile-and-cast, Karn -2 fetch, Saheeli token-copy, Sorin drain, Tezzeret etheric, Tamiyo). |
| Split cards (CR 709) + Fuse + Aftermath | ✅ | `CardDefinition.split` + `CastSplitRight`/`CastSplitFused`/`CastAftermath`. Ships Wear // Tear, Fire // Ice, Far // Away, Assault // Battery, Stand // Deliver, Wax // Wane, Alive // Well, Rough // Tumble, Profit // Loss, Supply // Demand, Toil // Trouble, Dead // Gone, Give // Take, Ready // Willing, plus Aftermath splits (Spring // Mind, Onward // Victory, Cut // Ribbons, Consign // Oblivion, Mouth // Feed). Client half-picker UI still ⏳ (TODO.md). |
| Living-weapon-on-ETB token | ⏳ | Nettlecyst, Sword-of-Body-and-Mind-adjacent. |
| Protection-from-color (more colors) | 🟡 | Engine has `Keyword::Protection(Color)`. Cards like Stillmoon Cavalier need toggle abilities + multi-color protection. |
| Charge counters as mana storage | 🟡 | Gemstone Mine wires charge counters + sac-on-empty. Coalition Relic now does the faithful precombat-main "remove all charges → 1 mana of any color each" burst (`MayDo` + `AddMana(AnyColors(CountersOn))` + `RemoveCounter(CountersOn)`). Power Depot, Pentad Prism, Chalice of the Void reuse the same charge primitive. |

## Plan

The cube is far too large to wire card-by-card on its own. The most
leverage comes from finishing the engine features above, then sweeping
groups of cards in batches. Suggested order:

1. **Token primitives** (Treasure / Blood / Clue / Food). Unlocks ~25
   cube cards in one batch.
2. **Equipment + Vehicles** (equip cost / crew). Unlocks Lion Sash,
   Shuko, Sword of …, Helm, Nettlecyst, Lavaspur Boots, Smuggler's
   Copter, Esika's Chariot.
3. **Cascade / Storm / Madness / Delve / Cycling** are each fairly small
   targeted features that unlock 3–5 cards apiece.
4. **DFC / Split-card / Adventure** infrastructure unlocks a wide swath
   of modern lands plus Wear // Tear, Sundering Eruption, etc.
5. **Multi-pick decisions** (reveal-and-sort, Dig Through Time, Atraxa
   real ETB, Mind's Desire) — promotes Atraxa to ✅ and unlocks several
   blue payoff spells.
6. **Counters / charge-mana / proliferate** for Mossborn Hydra, Heliod,
   Coalition Relic, Power Depot, Cankerbloom, etc.

Promote individual cards from ⏳ to 🟡 → ✅ as their dependent feature
lands. The status table above is the source of truth; update it
alongside the engine row when an effect ships.

## Maybeboard (raw)

Source list, preserved verbatim:

```
Descendant of Storms
Cathar Commando
Containment Priest
Lion Sash
Elite Spellbinder
Enduring Innocence
Flickerwisp
Heliod, Sun-Crowned
Loran of the Third Path
Ranger-Captain of Eos
Restoration Angel
Guardian Scalelord
Serra Angel
Intervention Pact
Isolate
Tempt with Bunnies
Static Prison
Virtue of Loyalty
Mockingbird
Dandân
Phantasmal Image
Thundertrap Trainer
Tishana's Tidebinder
Quantum Riddler
Deadeye Navigator
Pact of Negation
Consider
Spell Snare
Swan Song
Thought Scour
Consult the Star Charts
Daze
Lose Focus
Frantic Search
Cryptic Command
Paradoxical Outcome
Turnabout
Gush
Gather Specimens
Mirrorform
Dig Through Time
Windfall
Mind's Desire
Upheaval
Treasure Cruise
Aether Spellbomb
The Everflowing Well
Proft's Eidetic Memory
Back to Basics
Opposition
Parallax Tide
Omniscience
Shelldock Isle
Sink into Stupor
Concealing Curtains
Moonshadow
Bitterbloom Bearer
Bloodghast
Golgari Thug
Mai, Scornful Striker
Mutated Cultist
Silversmote Ghoul
Ichorid
Necrotic Ooze
Indulgent Tormentor
Crabomination
Doomsday Excruciator
Metamorphosis Fanatic
Slaughter Pact
Deadly Dispute
Corpse Dance
Baleful Mastery
Bloodchief's Thirst
Bone Shards
Disentomb
Collective Brutality
Drown in Ichor
Fell
Night's Whisper
Dread Return
Blasphemous Edict
Wishclaw Talisman
Parallax Dementia
Parallax Nexus
Unholy Annex // Ritual Chamber
Blazing Rootwalla
Greasewrench Goblin
Grim Lavamancer
Marauding Mako
Orcish Lumberjack
Voldaren Epicure
Amped Raptor
Cam and Farrik, Havoc Duo
Dreadhorde Arcanist
Magda, Brazen Outlaw
Robber of the Rich
Anje's Ravager
Death-Greeter's Champion
Detective's Phoenix
Simian Spirit Guide
Arclight Phoenix
Goldspan Dragon
Shivan Dragon
Balefire Dragon
Pact of the Titan
Tarfire
Chaos Warp
Big Score
Mine Collapse
Fireblast
Pyrokinesis
Vandalblast
Legion Extruder
Sundering Eruption
Basking Rootwalla
Elvish Reclaimer
Haywire Mite
Sylvan Safekeeper
Basking Broodscale
Cankerbloom
Collector Ouphe
Fanatic of Rhonas
Keen-Eyed Curator
Rofellos, Llanowar Emissary
Satyr Wayfinder
Sylvan Caryatid
Elvish Spirit Guide
Enduring Vitality
Hauntwoods Shrieker
Mossborn Hydra
Mutable Explorer
Sentinel of the Nameless City
Tireless Tracker
Ursine Monstrosity
Baloth Prime
Icetill Explorer
Mightform Harmonizer
Ouroboroid
Sowing Mycospawn
Vengevine
Elder Gargaroth
Golgari Grave-Troll
Railway Brawler
Conclave Sledge-Captain
Lumra, Bellow of the Woods
Zopandrel, Hunger Dominus
Apex Devastator
Summoner's Pact
Nature's Claim
Archdruid's Charm
Finale of Devastation
Life from the Loam
Nature's Lore
Kodama's Reach
Biorhythm
Esika's Chariot
Springleaf Parade
Up the Beanstalk
Aluren
Greater Good
Shifting Woodland
Ornithopter
Ornithopter of Paradise
Glaring Fleshraker
Tezzeret, Cruel Captain
Karn, Scion of Urza
Kozilek's Command
Eldrazi Confluence
Chalice of the Void
Zuran Orb
Candelabra of Tawnos
Chromatic Star
Ghost Vacuum
Lavaspur Boots
Pithing Needle
Shuko
Soul-Guide Lantern
Agatha's Soul Cauldron
Fellwar Stone
Mesmeric Orb
Millstone
Mind Stone
Pentad Prism
Smuggler's Copter
Coalition Relic
Monument to Endurance
Nettlecyst
Sword of Body and Mind
Trinisphere
Helm of the Host
The Mightstone and Weakstone
Coveted Jewel
The Endstone
Portal to Phyrexia
Talisman of Progress
Shorikai, Genesis Engine
Cruel Somnophage
Talisman of Dominance
Ashiok, Nightmare Weaver
Master of Death
Fallen Shinobi
Bloodtithe Harvester
Terminate
Carnage Interpreter
Kolaghan's Command
Master of Cruelties
Territorial Kavu
Bloodbraid Challenger
Qasali Pridemage
Knight of the Reliquary
Brightglass Gearhulk
Growing Ranks
Torsten, Founder of Benalia
Tidehollow Sculler
Gift of Orzhova
Stillmoon Cavalier
Sorin, Grim Nemesis
Expressive Iteration
Talisman of Creativity
Pinnacle Emissary
Saheeli Rai
Tempest Angler
Abrupt Decay
Assassin's Trophy
Broodspinner
Tear Asunder
Wight of the Reliquary
The Gitrog Monster
Talisman of Conviction
Wear // Tear
Zirda, the Dawnwaker
Talisman of Curiosity
Lonis, Genetics Expert
Tamiyo, Collector of Tales
Sab-Sunen, Luxa Embodied
Koma, Cosmos Serpent
Kestia, the Cultivator
Messenger Falcons
Dakkon, Shadow Slayer
Urza, Chief Artificer
Geyadrone Dihada
Lord Xander, the Collector
Korvold, Fae-Cursed King
Temur Ascendancy
Loot, the Pathfinder
Dragonback Assault
Rediscover the Way
Shiko and Narset, Unified
Awaken the Honored Dead
Fangkeeper's Familiar
Teval, Arbiter of Virtue
Muldrotha, the Gravetide
Rakshasa's Bargain
Omnath, Locus of Creation
Atraxa, Grand Unifier
Leyline of the Guildpact
Golos, Tireless Pilgrim
Maelstrom Archangel
Maelstrom Nexus
Ramos, Dragon Engine
Celestial Colonnade
Meticulous Archive
Razortide Bridge
Seachrome Coast
Creeping Tar Pit
Darkslick Shores
Mistvault Bridge
Undercity Sewers
Blackcleave Cliffs
Blazemire Verge
Drossforge Bridge
Raucous Theater
Commercial District
Copperline Gorge
Slagwoods Bridge
Thornspire Verge
Horizon Canopy
Lush Portico
Razorverge Thicket
Thornglint Bridge
Bleachbone Verge
Concealed Courtyard
Goldmire Bridge
Shadowy Backstreet
Riverpyre Verge
Silverbluff Bridge
Spirebluff Canal
Thundering Falls
Blooming Marsh
Darkmoss Bridge
Underground Mortuary
Wastewood Verge
Elegant Parlor
Inspiring Vantage
Rustvale Bridge
Sunbaked Canyon
Botanical Sanctum
Hedge Maze
Tanglepool Bridge
Waterlogged Grove
Twisted Landscape
Sheltering Landscape
Bountiful Landscape
Ancient Den
Cloudpost
Darksteel Citadel
Evolving Wilds
Exotic Orchard
Glimmerpost
Great Furnace
Lotus Field
Planar Nexus
Power Depot
Rishadan Port
Seat of the Synod
Talon Gates of Madara
Three Tree City
Tree of Tales
Trenchpost
Vault of Whispers
```
