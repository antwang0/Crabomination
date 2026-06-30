# Cube Implementation Tracker

Tracking the work to make this Modern-style cube fully playable — efficient
creatures, interaction, value engines, and combo lines (see the maybeboard at
the end).

The catalog already implements the BRG / Goryo's demo decks
(`DECK_FEATURES.md` is source of truth there); some overlap the cube. Most of
the cube is still ⏳. Done (✅) cards and engine features are elided — only
remaining 🟡/⏳ work is listed.

## Legend

- 🟡 partial — exists with a simplified/stub effect; key behavior missing
- ⏳ todo — not yet implemented

### Per-color card status

Per-color tables (White / Blue / Black / Red / Green / Artifacts &
Planeswalkers / Multicolor / Lands) are empty — done cards are elided and no
partials remain. Outstanding cards are in the maybeboard below; populate a
color table only when a card is in-progress (🟡) but not finished.

## Engine features needed

The engine features already done (Pact / Flashback / Convoke / Rebound / etc.)
are in `DECK_FEATURES.md`. This is the remaining work to bring most of the cube
online.

| Feature | Status | Notes |
|---|---|---|
| Equipment + equip-cost ability | ✅ | `GameAction::Equip` + `equipped_bonus`. Board-scaled bonus (`EquipBonus.scale` — Nettlecyst), equipment-granted triggers (Sword of Body and Mind), Reconfigure (Lion Sash), living weapon (Batterskull), equip-triggers that charge the *Equipment* (Umezawa's Jitte), instant-speed equip (Leonin Shikari), equip-cost reduction (Auriok Steelshaper), per-Equipment CDA (Goblin Gaveleer), "while equipped, has [kw]" (`SelfHasKeywordWhile` — Kor Duelist), while-equipped team anthems (`PumpTeamIf` + `Predicate::SourceIsEquipped` — Auriok Steelshaper's Soldier/Knight buff), conditional keyword by attached-count (`SelectionRequirement::EquippedByAtLeast` — Balan's double strike), and two-target Attach via spell/ETB (Embercleave) all ship. |
| Adventure (cost-mode duality) | ✅ | `CardDefinition.adventure` + `CastAdventure`/`CastAdventureCreature` (incl. Virtue of Loyalty enchantment // instant). |
| Storm count + cast-from-top | 🟡 | Cast-from-library-top statics ✅ (Mystic Forge, Courser, Oracle of Mul Daya). Storm count wired (`Value::StormCount`, auto-copy on cast) + `Effect::CopySpell`. Exile-top-and-grant-free-play ✅ (Robber of the Rich, Mind's Desire); energy-gated free cast-from-exile ✅ (Amped Raptor). |
| Soulbond | ✅ | `Keyword::Soulbond` + `soulbond_partner` + `SoulbondBonus` (P/T, keywords, granted abilities). Pairs auto-resolve on ETB, break on leave (Wolfir Silverheart, Deadeye Navigator, …). |
| Saga lore counters | ✅ | `saga_chapters` (History of Benalia). DFC sagas ✅ (`ExileSelfReturnTransformed` — Fable of the Mirror-Breaker). |
| Transforming DFCs | ✅ | `Effect::Transform` + `CardInstance.{transformed,front_face}`; swaps face in place, round-trips through snapshots (Delver, Concealing Curtains, The Everflowing Well). Daybound/Nightbound, DFC sagas, manifest/disguise all ship. |
| Hideaway lands | ✅ | `Effect::Hideaway { count }` + `CardExiledWithSource` (Shelldock Isle, Mosswort Bridge, Spinerock Knoll, Windbrisk Heights). |
| Impending (CR 702.183) | ✅ | `Keyword::Impending(n)` + `AlternativeCost.impending` — enters with N time counters, not a creature until they tick off (all five Duskmourn Overlords). |
| Verge / surveil land families | ✅ | All five `*verge` lands (`verge_land`), the horizon-canopy six (`horizon_land`), and all ten MKM surveil lands (`etb_tap_then_surveil_one`). |
| ETB-replacement (suppress entirely) | 🟡 | Exile-non-cast-creature-instead (Containment Priest), ETB/death-trigger suppression (`SuppressCreatureEtbTriggers` — Torpor Orb, Hushbringer), and steal-instead (Gather Specimens) all ship. |
| Spell-tax statics | ✅ | Damping Sphere, flat `AdditionalCost`, and the Trinisphere minimum-cost floor (`SpellCostFloor`) all ship. Elite Spellbinder reuses the tax static. |
| Cast spells without paying mana | ✅ | Omniscience, Aluren, Maelstrom Archangel (`CastFromHandWithoutPaying`). |
| Name-a-card primitive | 🟡 | `Effect::NameCard` + `Decision::NameCard` (Pithing Needle, Phyrexian Revoker); same-name exile (Crumble to Dust), reveal-until-find (Spoils of the Vault), hand-discard-by-name (Cabal Therapy), name-then-sort (Tamiyo) all ship. |
| Face-down permanents (morph/manifest) | 🟡 | `face_up_def` stashes the real card behind a vanilla 2/2; Manifest/ManifestDread, `CastFaceDown` ({3}), `TurnFaceUp`. Disguise ✅ (warded 2/2), Cloak ✅ (`CardInstance.cloaked`). |
| Token-copy of permanent | 🟡 | Populate ✅; `CreateTokenCopyOf` ✅ (with `non_legendary` rider — Helm of the Host); clone-enter ✅ (`BecomeCopyOf`); continuous "becomes a copy" ✅ (`BecomeCopyOfFor` — Mirrorform, Vesuva). |
| Multi-pick over revealed cards | ✅ | `LookPickToHand` with `take > 1` (Dig Through Time); `RevealTopTakeOnePerType` (Atraxa, Grand Unifier). Put-into-hand is not a draw (121.5). |
| Investigate + Clue/Map tokens | 🟡 | Clue tokens ✅ (Tireless Tracker, Lonis); Map tokens ✅ (Loot, the Pathfinder). Sac-a-Clue payoffs ride `sac_other_filter: HasArtifactSubtype(Clue)`; variable "Sacrifice X Clues" ✅ (`sac_other_x` — Lonis). |
| Landfall trigger | ✅ | Graveyard-source landfall via `EventScope::FromYourGraveyard` (Bloodghast); battlefield-side via `LandPlayed` + `YourControl` (Omnath). |
| Loyalty abilities w/ static | 🟡 | Teferi, loyalty-set (`SetLoyalty` — Geyadrone Dihada), variable `-X` (`x_cost` — Kasmina), and the rest (Ashiok, Sorin, Tamiyo, Dakkon, Saheeli -7, Karn Scion, Tezzeret) all ship. No remainder of note. |
| Split cards (709) + Fuse + Aftermath | ✅ | `CardDefinition.split` + `CastSplitRight`/`CastSplitFused`/`CastAftermath` (Wear // Tear, Fire // Ice, the aftermath splits, …). Client half-picker UI ⏳ (TODO.md). |
| Protection-from-color (more colors) | ✅ | Multi-color printed protection (Stillmoon Cavalier) + EOT grants. |
| Charge counters as mana storage | 🟡 | Gemstone Mine, Coalition Relic (precombat burst), Power Depot, Pentad Prism, Chalice of the Void all reuse the charge-counter primitive. |

## Plan

The cube is too large to wire card-by-card. Most leverage comes from finishing
the engine features above, then sweeping card groups in batches:

1. **Token primitives** (Treasure/Blood/Clue/Food) — ~25 cards in one batch.
2. **Equipment + Vehicles** (equip cost / crew).
3. **Cascade / Storm / Madness / Delve / Cycling** — small features, 3–5 cards each.
4. **DFC / Split-card / Adventure** infrastructure — a wide swath of modern lands.
5. **Multi-pick decisions** — promotes Atraxa to ✅, unlocks blue payoffs.
6. **Counters / charge-mana / proliferate** — Mossborn Hydra, Heliod, Coalition Relic.

Promote cards ⏳ → 🟡 → ✅ as their dependent feature lands; update the engine row
alongside.

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
