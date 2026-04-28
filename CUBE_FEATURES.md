# Cube Implementation Tracker

Tracking the work to make this Modern-style cube fully playable. The cube
is built around a mix of efficient creatures, interaction, value engines,
and combo lines — see the maybeboard at the end of this file.

The catalog already implements the cards used by the BRG / Goryo's demo
decks (`DECK_FEATURES.md` is the source of truth there); some of those
cards overlap with the cube and are flagged ✅ here. Most of the cube is
still ⏳.

## Legend

- ✅ done — already wired in `crate::catalog` with full functionality
- 🟡 partial — exists with a simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Cards

### White

| Card | Status | Notes |
|---|---|---|
| Descendant of Storms | ⏳ | Spirit token + tap-trigger; needs the Spirit creature-token primitive. |
| Cathar Commando | ✅ | Flash + {1}, sac: destroy artifact/enchantment (`ActivatedAbility::sac_cost`). |
| Containment Priest | ⏳ | Replacement effect: nontoken creatures entering tapped get exiled. Needs ETB-replacement primitive. |
| Lion Sash | ⏳ | Equipment + grow via exile-from-graveyard. Needs equipment + counters wiring. |
| Elite Spellbinder | ⏳ | ETB look-at-opp-hand + cost-tax static while in play. Reuses `AdditionalCostAfterFirstSpell`-style hooks. |
| Enduring Innocence | ⏳ | Lifelink + draw-on-creature-ETB trigger; "Roomba"-style return-from-exile post-death. Needs ETB-other listener + delayed self-revive. |
| Flickerwisp | ✅ | 3/1 Flying; ETB exile target permanent + `DelayUntil(NextEndStep, Move-back-to-OwnerOf)`. |
| Heliod, Sun-Crowned | ⏳ | Lifelink-grant ability + counter combo (with Walking Ballista). Needs +1/+1-on-life-gain trigger primitive. |
| Loran of the Third Path | ✅ | 1/3 Vigilance; ETB destroy artifact/enchantment; {T}: you and target opponent each draw a card. |
| Ranger-Captain of Eos | 🟡 | ETB tutor for ≤1-CMC creature. The sac-for-no-noncreature-spells static is omitted (no sac-as-cost activation primitive). |
| Restoration Angel | ✅ | Flash + ETB exile-and-return target non-Angel creature you control (`Exile + Move-back` flicker pattern). |
| Guardian Scalelord | ⏳ | Flying + grant flying to attackers via attack trigger. |
| Serra Angel | ✅ | Already in `crate::catalog::serra_angel` (4/4 flying + vigilance). |
| Intervention Pact | ⏳ | Free prevent-damage + delayed `PayOrLoseGame` upkeep cost (reuses Pact primitive). |
| Isolate | ✅ | Exile target permanent with mana value 1 (`ManaValueAtLeast(1) ∧ ManaValueAtMost(1)` filter). |
| Tempt with Bunnies | ⏳ | Tempting offer (chain-creating) — needs multi-player choice primitive. |
| Static Prison | ⏳ | Pay X to enter with X stun counters; tap target permanent. |
| Virtue of Loyalty | ⏳ | Adventure + enchantment side. Needs Adventure cost-mode primitive. |

### Blue

| Card | Status | Notes |
|---|---|---|
| Mockingbird | ⏳ | ETB copy-the-name-of a creature you control. |
| Dandân | ⏳ | Defender variant; sacrifice when defender no Island. |
| Phantasmal Image | ⏳ | ETB enter-as-copy of any creature. Needs token-copy-of-permanent primitive. |
| Thundertrap Trainer | ⏳ | Trap/discount creature. |
| Tishana's Tidebinder | ⏳ | ETB counter target activated/triggered ability — reuses `Effect::CounterAbility`. |
| Quantum Riddler | ✅ | UB 4/4 flying + on-cast Draw 1 (already in catalog). |
| Deadeye Navigator | ⏳ | Soulbond + activated flicker. Reuses Flicker primitive; needs Soulbond. |
| Pact of Negation | ✅ | Free counter + delayed `PayOrLoseGame` upkeep. |
| Consider | ✅ | Surveil 1 then Draw 1 (reuses Surveil + Draw primitives). |
| Spell Snare | ✅ | Counter target spell with mana value 2 (CounterSpell + `ManaValueAt{Most,Least}(2)` sandwich). |
| Swan Song | 🟡 | Counter enchantment/instant/sorcery; 2/2 Flying Bird token. (Token goes to caster's opponents — equivalent in 2-player; engine has no `ControllerOf` lookup for stack/graveyard cards.) |
| Thought Scour | ✅ | Mill 2 + draw 1. |
| Consult the Star Charts | ⏳ | Look-at-top-N + draw — needs Foretell-adjacent decision. |
| Daze | 🟡 | Counter target spell unless its controller pays {1}. The "return an Island" alt cost is omitted (alt-cost model only supports exile-from-hand). |
| Lose Focus | ⏳ | Counter spell + delve. |
| Frantic Search | 🟡 | Draw 2, discard 2, untap your tapped lands (approximation: untaps every tapped land you control rather than "up to three"). |
| Cryptic Command | ⏳ | Modal four-way counterspell. Reuses ChooseMode + counter/draw/tap/return. |
| Paradoxical Outcome | ✅ | Return each non-land permanent you control + draw equal (`ForEach + Move + Draw 1`). |
| Turnabout | ⏳ | Tap/untap target permanent type. |
| Gush | ⏳ | Free draw if pitching two Islands; alternative cost variant. |
| Gather Specimens | ⏳ | Replace creature ETB control-shift. Replacement effect primitive. |
| Mirrorform | ⏳ | Aura + clone target. |
| Dig Through Time | ⏳ | Delve + look at top 7, take 2. Multi-pick primitive. |
| Windfall | ⏳ | Each player discards then draws equal. |
| Mind's Desire | ⏳ | Storm + cast-from-top. Needs Storm count + cast-from-top primitive. |
| Upheaval | ✅ | Return all permanents to their owners' hands (`ForEach + Move → Hand(OwnerOf)`). |
| Treasure Cruise | ⏳ | Delve + Draw 3. Needs Delve cost reduction. |
| Aether Spellbomb | ✅ | {U}, sac: return target creature to hand. {1}, sac: draw a card. Both via `ActivatedAbility::sac_cost`. |
| The Everflowing Well | ⏳ | Saga land flip; needs Saga lore counters + DFC. |
| Proft's Eidetic Memory | ⏳ | Investigate + scaling +1/+1. Needs investigate (Clue). |
| Back to Basics | ⏳ | Static "nonbasic lands don't untap". Needs land-untap restriction. |
| Opposition | ⏳ | Tap creatures to tap permanents. |
| Parallax Tide | ⏳ | Fading + exile lands. Needs fade counters. |
| Omniscience | ⏳ | "Cast spells without paying". Needs free-cast static. |
| Shelldock Isle | ⏳ | Hideaway land (DFC-like setup). |
| Sink into Stupor | ⏳ | Counter + DFC into Lair land. |
| Concealing Curtains | ⏳ | DFC into Revealing Eye creature. |

### Black

| Card | Status | Notes |
|---|---|---|
| Moonshadow | ⏳ | Faerie / discard support. |
| Bitterbloom Bearer | ⏳ | Token-creating creature. |
| Bloodghast | ⏳ | Land-fall return-from-graveyard. Needs landfall trigger + recursion. |
| Golgari Thug | ⏳ | Dredge 4. Needs Dredge primitive. |
| Mai, Scornful Striker | ⏳ | New creature — abilities TBD. |
| Mutated Cultist | ⏳ | Mutate primitive needed. |
| Silversmote Ghoul | ⏳ | Recursive creature on lifegain. |
| Ichorid | ⏳ | Recursive return-from-graveyard during upkeep. |
| Necrotic Ooze | ⏳ | Gains all activated abilities of creatures in graveyards. Big ability-borrow primitive. |
| Indulgent Tormentor | ⏳ | Choice on attack. |
| Crabomination | ⏳ | Custom card — TBD. |
| Doomsday Excruciator | ⏳ | Doomsday-adjacent. |
| Metamorphosis Fanatic | ⏳ | Unknown — TBD. |
| Slaughter Pact | ✅ | Destroy nonblack creature + delayed `PayOrLoseGame` upkeep ({2}{B}). |
| Deadly Dispute | ⏳ | Sac creature/artifact + draw 2 + Treasure. Needs Treasure tokens. |
| Corpse Dance | ⏳ | Buyback + reanimate creature top of grave. |
| Baleful Mastery | ⏳ | Exile target nonland; opp may draw 2 to halve cost. Modal alt-cost. |
| Bloodchief's Thirst | ⏳ | Kicker + remove counter. |
| Bone Shards | ⏳ | Sac-or-discard cost flexibility. |
| Disentomb | ✅ | Return target creature card to hand (Move from graveyard). |
| Collective Brutality | ⏳ | Escalate-modal removal. |
| Drown in Ichor | ✅ | 3 damage to target creature + Surveil 1. |
| Fell | ✅ | Destroy target tapped creature + Surveil 2. |
| Night's Whisper | ✅ | Pay 2 life + Draw 2 (already in `decks::modern`). |
| Dread Return | ⏳ | Sac-3-creatures flashback reanimation. Reuses Flashback. |
| Blasphemous Edict | ✅ | Each player sacrifices a creature (`Sacrifice` + `EachPlayer`). |
| Wishclaw Talisman | ⏳ | Wish-style tutor with downside. |
| Parallax Dementia | ⏳ | Fading + reanimate; needs fade counters. |
| Parallax Nexus | ⏳ | Fading + hand-strip. |
| Unholy Annex // Ritual Chamber | ⏳ | DFC enchantment land. |

### Red

| Card | Status | Notes |
|---|---|---|
| Blazing Rootwalla | ⏳ | Madness creature. Needs Madness. |
| Greasewrench Goblin | ⏳ | Sacrifice-payoff. |
| Grim Lavamancer | ⏳ | Tap + exile-2-from-graveyard for damage. |
| Marauding Mako | ⏳ | Pinger/aggressive creature. |
| Orcish Lumberjack | ⏳ | Sac-Forest for {GGG}. |
| Voldaren Epicure | ✅ | ETB: create a Blood token + 1 damage to each opponent (`ForEach EachOpponent`). |
| Amped Raptor | ⏳ | ETB cast spell from top. |
| Cam and Farrik, Havoc Duo | ⏳ | Dual creature. |
| Dreadhorde Arcanist | ⏳ | Attack-trigger flashback from grave. Reuses Flashback. |
| Magda, Brazen Outlaw | ⏳ | Treasure-on-tap + tutor. |
| Robber of the Rich | ⏳ | Cast-from-opp-library. Big primitive. |
| Anje's Ravager | ⏳ | Madness payoff. |
| Death-Greeter's Champion | ⏳ | Aggressive creature. |
| Detective's Phoenix | ⏳ | Recurring Phoenix variant. |
| Simian Spirit Guide | ⏳ | Exile from hand for {R}. Needs alt-cost-from-hand. |
| Arclight Phoenix | ⏳ | Three-spell-cast trigger from graveyard. Needs spells-cast-this-turn count + recursion. |
| Goldspan Dragon | 🟡 | 4/4 Flying Haste; attack-trigger Treasure (using the now-functional Treasure mana ability). "Becomes target of a spell" trigger and the Treasure-2-mana static rider are omitted. |
| Shivan Dragon | ✅ | Already in catalog. |
| Balefire Dragon | ⏳ | Combat damage trigger; needs filtered DealDamage(EachCreature). |
| Pact of the Titan | ✅ | 4/4 red Giant token + delayed `PayOrLoseGame` upkeep ({4}{R}). |
| Tarfire | 🟡 | 2 damage to any target. Tribal type omitted (engine has no Tribal card type). |
| Chaos Warp | ⏳ | Shuffle target permanent + reveal-from-top. |
| Big Score | ✅ | Discard + 2 Treasure + Draw 2. Treasure tokens are now fully functional — each carries its `{T}, Sac: Add one mana of any color` activated ability via `TokenDefinition::activated_abilities`. |
| Mine Collapse | ⏳ | Sac-land alt-cost removal. |
| Fireblast | ⏳ | Sac-Mountains alt-cost burn. |
| Pyrokinesis | ⏳ | Pitch-cost mass burn. Reuses pitch-cost path (Force of Will). |
| Vandalblast | 🟡 | Single-target artifact destruction; Overload {4}{R} mode omitted (no overload primitive yet). |
| Legion Extruder | ⏳ | Equip-ish artifact. |
| Sundering Eruption | ✅ | MDFC: front is `{1}{R}` sorcery dealing 3 damage to a creature/planeswalker; back face Mount Tyrhus is a Mountain that ETBs tapped and taps for {R}. |

### Green

| Card | Status | Notes |
|---|---|---|
| Basking Rootwalla | ⏳ | Madness creature. |
| Elvish Reclaimer | ⏳ | Land-tutor activated ability. |
| Haywire Mite | ✅ | {2}, sac: destroy artifact/enchantment/planeswalker + gain 1 life (`ActivatedAbility::sac_cost`). |
| Sylvan Safekeeper | ⏳ | Sac-Forest grant shroud. |
| Basking Broodscale | ⏳ | Eldrazi token-maker. |
| Cankerbloom | ⏳ | Sac for proliferate. |
| Collector Ouphe | ⏳ | Static "artifact abilities can't be activated". |
| Fanatic of Rhonas | ⏳ | Bigger Llanowar Elves. |
| Keen-Eyed Curator | ⏳ | Graveyard hate + counter pump. |
| Rofellos, Llanowar Emissary | ⏳ | Tap for {G}{G} per Forest. |
| Satyr Wayfinder | ⏳ | ETB top-4 mill, take a land. |
| Sylvan Caryatid | ✅ | 0/3 Hexproof Defender; {T}: Add one mana of any color. |
| Elvish Spirit Guide | ⏳ | Exile-from-hand for {G}. |
| Enduring Vitality | ⏳ | Roomba-style return on death + creature mana untap. |
| Hauntwoods Shrieker | ⏳ | Token + transform. |
| Mossborn Hydra | ⏳ | Hydra +1/+1 counter scaling. |
| Mutable Explorer | ⏳ | Mutate primitive. |
| Sentinel of the Nameless City | 🟡 | Vigilance + attack-trigger 1/1 green Citizen token. Ward 2 omitted (keyword exists but not enforced at targeting time); Plant subtype dropped (no `Plant` in `CreatureType`). |
| Tireless Tracker | 🟡 | Filtered ETB-other trigger: when a land enters under your control, investigate (create a Clue). Sac-Clue +1/+1 ability omitted (no sac-of-other-permanent activation primitive). |
| Ursine Monstrosity | ⏳ | Adapt-style P/T scaling. |
| Baloth Prime | ⏳ | TBD. |
| Icetill Explorer | ⏳ | TBD. |
| Mightform Harmonizer | ⏳ | TBD. |
| Ouroboroid | ⏳ | TBD. |
| Sowing Mycospawn | ⏳ | Eldrazi land-search. |
| Vengevine | ⏳ | Recurring on two-creatures-cast-this-turn. |
| Elder Gargaroth | ⏳ | Trigger-on-attack/block creature. |
| Golgari Grave-Troll | ⏳ | Dredge 6. |
| Railway Brawler | ⏳ | Train (vehicle-like). |
| Conclave Sledge-Captain | ⏳ | TBD. |
| Lumra, Bellow of the Woods | ⏳ | Land-from-graveyard return. |
| Zopandrel, Hunger Dominus | ⏳ | TBD. |
| Apex Devastator | ⏳ | Cascade x4 (cascade primitive). |
| Summoner's Pact | ✅ | Already wired (search green creature + delayed Pact upkeep). |
| Nature's Claim | ⏳ | Destroy artifact/enchantment + 4 life. |
| Archdruid's Charm | ⏳ | Modal — destroy land/artifact, search creature, +1/+1 counters. |
| Finale of Devastation | ⏳ | Tutor + pump scaling with X. |
| Life from the Loam | ⏳ | Return up-to-3 lands; Dredge 3. |
| Nature's Lore | ✅ | Search Forest, put onto battlefield untapped. |
| Kodama's Reach | ⏳ | Two-basic ramp. |
| Biorhythm | ⏳ | Each player's life = creatures. |
| Esika's Chariot | ⏳ | Vehicle + crew + token. |
| Springleaf Parade | ⏳ | TBD. |
| Up the Beanstalk | ✅ | ETB Draw 1 + filtered SpellCast trigger (mana value ≥ 5 → Draw 1). |
| Aluren | ⏳ | Free-cast 3 or less creatures. |
| Greater Good | ⏳ | Sac creature + Draw P. |
| Shifting Woodland | ⏳ | DFC land. |

### Artifacts & Planeswalkers (mono / colorless)

| Card | Status | Notes |
|---|---|---|
| Ornithopter | ✅ | {0} Artifact creature 0/2 with Flying. |
| Ornithopter of Paradise | ✅ | {1} Artifact creature 0/2 Flying; {T}: Add one mana of any color. |
| Glaring Fleshraker | ⏳ | TBD. |
| Tezzeret, Cruel Captain | ⏳ | Planeswalker — loyalty abilities + ult. |
| Karn, Scion of Urza | ⏳ | +1 / -1 / -2 walker; constructs. |
| Kozilek's Command | ⏳ | Modal Eldrazi instant. |
| Eldrazi Confluence | ⏳ | Modal x3. |
| Chalice of the Void | ⏳ | X charge counters; counter spells with mana value X. |
| Zuran Orb | ⏳ | Sac-land for life. |
| Candelabra of Tawnos | ⏳ | Untap N lands for {X}. |
| Chromatic Star | ⏳ | Sac for 1 mana of any color + Draw 1. |
| Ghost Vacuum | ⏳ | Graveyard-hate artifact. |
| Lavaspur Boots | ⏳ | Equipment grants haste. |
| Pithing Needle | ⏳ | Name-a-card; activated abilities can't be activated. Needs name-a-card primitive (Cavern shares). |
| Shuko | ⏳ | Equipment with free-equip. |
| Soul-Guide Lantern | ⏳ | Graveyard-exile artifact. |
| Agatha's Soul Cauldron | ⏳ | Borrow activated abilities of exiled creatures. |
| Fellwar Stone | 🟡 | {T}: Add one mana of any color. (Approximation: drops the "matches opponent's lands" restriction — engine has no per-source mana provenance yet.) |
| Mesmeric Orb | ⏳ | Mill-on-untap symmetric. |
| Millstone | ✅ | {2}, {T}: target player mills 2. |
| Mind Stone | ✅ | {T}: Add {C}. {1}, {T}, Sacrifice this: Draw a card. Both abilities wired (uses `ActivatedAbility::sac_cost`). |
| Pentad Prism | ⏳ | Sunburst counters; tap for one mana. |
| Smuggler's Copter | ⏳ | Vehicle + crew + loot trigger. Needs Vehicle primitive. |
| Coalition Relic | ⏳ | Mana rock + charge counter. |
| Monument to Endurance | ⏳ | Graveyard-recursion artifact. |
| Nettlecyst | ⏳ | Living-equipment + token. |
| Sword of Body and Mind | ⏳ | Equipment + protection + token + mill. |
| Trinisphere | ⏳ | Static "spells cost at least {3}". Reuses cost-tax static. |
| Helm of the Host | ⏳ | Equipment that token-copies on attack. |
| The Mightstone and Weakstone | ⏳ | Modal artifact (assemble). |
| Coveted Jewel | ⏳ | Mana + force-attack control mechanic. |
| The Endstone | ⏳ | TBD. |
| Portal to Phyrexia | ⏳ | Big sac-3 + reanimate. |
| Talisman of Progress | ⏳ | Two-color mana rock; loses life on tap-for-color. |

### Multicolor

| Card | Status | Notes |
|---|---|---|
| Shorikai, Genesis Engine | ⏳ | Vehicle-walker hybrid; loots on activate. |
| Cruel Somnophage | ⏳ | UB scaling-with-graveyard creature. |
| Talisman of Dominance | ⏳ | UB mana rock. |
| Ashiok, Nightmare Weaver | ⏳ | Planeswalker — exile/scry/copy. |
| Master of Death | ⏳ | UB recursion + discard. |
| Fallen Shinobi | ⏳ | Ninjitsu + reveal-and-take. |
| Bloodtithe Harvester | 🟡 | ETB and attack triggers each create a Blood token. Sac-Blood ping ability omitted (no sac-of-other-permanent activation primitive). |
| Terminate | ✅ | Already in catalog (destroy can't-regenerate). |
| Carnage Interpreter | ⏳ | TBD. |
| Kolaghan's Command | ⏳ | Modal x2. |
| Master of Cruelties | ⏳ | First-strike + life-to-1 attack trigger. |
| Territorial Kavu | ⏳ | Color-matters. |
| Bloodbraid Challenger | ⏳ | Cascade. |
| Qasali Pridemage | ⏳ | Exalted + sac to destroy artifact/enchantment. |
| Knight of the Reliquary | ⏳ | Sac-land tutor scaling P/T. |
| Brightglass Gearhulk | ⏳ | TBD. |
| Growing Ranks | ⏳ | Populate token-copy on upkeep. |
| Torsten, Founder of Benalia | ⏳ | TBD. |
| Tidehollow Sculler | ⏳ | ETB exile-then-discard. |
| Gift of Orzhova | ⏳ | Aura — flying + lifelink. |
| Stillmoon Cavalier | ⏳ | Mana abilities for protection toggling. |
| Sorin, Grim Nemesis | ⏳ | Planeswalker. |
| Expressive Iteration | ⏳ | UR look-at-top-3 multi-pick. |
| Talisman of Creativity | ⏳ | UR mana rock. |
| Pinnacle Emissary | ⏳ | TBD. |
| Saheeli Rai | ⏳ | UR planeswalker. |
| Tempest Angler | ⏳ | TBD. |
| Abrupt Decay | ⏳ | BG removal: destroy nonland with mana value ≤ 3, can't be countered. |
| Assassin's Trophy | ⏳ | BG removal — opp searches for basic. |
| Broodspinner | ⏳ | TBD. |
| Tear Asunder | ⏳ | BG kicker. |
| Wight of the Reliquary | ⏳ | Land-tutor sac variant. |
| The Gitrog Monster | ⏳ | Land-as-discard / dredge engine. |
| Talisman of Conviction | ⏳ | RW mana rock. |
| Wear // Tear | ⏳ | Split-card; needs Split-card primitive. |
| Zirda, the Dawnwaker | ⏳ | Companion + activated-cost reduction. Needs Companion primitive. |
| Talisman of Curiosity | ⏳ | GU mana rock. |
| Lonis, Genetics Expert | ⏳ | Investigate + Clue draw. |
| Tamiyo, Collector of Tales | ⏳ | Planeswalker. |
| Sab-Sunen, Luxa Embodied | ⏳ | TBD. |
| Koma, Cosmos Serpent | ⏳ | Token-on-upkeep + sac counters. |
| Kestia, the Cultivator | ⏳ | Aura/enchantment matters. |
| Messenger Falcons | ⏳ | TBD. |
| Dakkon, Shadow Slayer | ⏳ | TBD. |
| Urza, Chief Artificer | ⏳ | Planeswalker / commander. |
| Geyadrone Dihada | ⏳ | Planeswalker. |
| Lord Xander, the Collector | ⏳ | ETB / attack / death triggers. |
| Korvold, Fae-Cursed King | ⏳ | Sac-trigger draw + +1/+1. |
| Temur Ascendancy | 🟡 | Filtered ETB trigger (creatures w/ power ≥ 4 entering under your control → Draw 1). Static "creatures you control have haste" currently grants haste to all your creatures rather than only ≥ 4 power (selector-decomposer doesn't yet thread `PowerAtLeast` into static-effect targeting). |
| Loot, the Pathfinder | ⏳ | TBD. |
| Dragonback Assault | ⏳ | TBD. |
| Rediscover the Way | ⏳ | TBD. |
| Shiko and Narset, Unified | ⏳ | TBD. |
| Awaken the Honored Dead | ⏳ | Reanimate-multiple. |
| Fangkeeper's Familiar | ⏳ | TBD. |
| Teval, Arbiter of Virtue | ⏳ | TBD. |
| Muldrotha, the Gravetide | ⏳ | Cast from graveyard each turn (one of each card type). |
| Rakshasa's Bargain | 🟡 | Pay 4 life + Draw 4. The "exile creature card from your graveyard" alternate additional cost is folded away (modal additional-cost not modeled). |
| Omnath, Locus of Creation | ⏳ | Landfall-quad-color. |
| Atraxa, Grand Unifier | 🟡 | Already wired with ETB Draw 4 approximation. Real reveal-and-sort still ⏳. |
| Leyline of the Guildpact | ⏳ | "Your permanents are all colors." Needs is-all-colors primitive. |
| Golos, Tireless Pilgrim | ⏳ | ETB land-search; activated cast-from-top. |
| Maelstrom Archangel | ⏳ | Combat-damage cast-free trigger. |
| Maelstrom Nexus | ⏳ | Cascade-on-first-spell static. |
| Ramos, Dragon Engine | ⏳ | Charge counters per-color spells; convert to mana. |

### Lands

| Card | Status | Notes |
|---|---|---|
| Celestial Colonnade | ⏳ | UW manland. |
| Meticulous Archive | ✅ | UW surveil land (catalog). |
| Razortide Bridge | ⏳ | UW bridge — colorless tap; pay life for color. |
| Seachrome Coast | ✅ | UW fastland (reuses fastland trigger). |
| Creeping Tar Pit | ⏳ | UB manland. |
| Darkslick Shores | ✅ | UB fastland. |
| Mistvault Bridge | ⏳ | UB bridge. |
| Undercity Sewers | ✅ | UB surveil land (catalog). |
| Blackcleave Cliffs | ✅ | BR fastland (catalog). |
| Blazemire Verge | ⏳ | BR DFC verge. |
| Drossforge Bridge | ⏳ | BR bridge. |
| Raucous Theater | ⏳ | BR surveil land. |
| Commercial District | ⏳ | RW surveil land. |
| Copperline Gorge | ✅ | RG fastland (catalog). |
| Slagwoods Bridge | ⏳ | RG bridge. |
| Thornspire Verge | ⏳ | RG verge. |
| Horizon Canopy | ⏳ | GW horizon canopy: pay 1 + life to draw. |
| Lush Portico | ⏳ | GW surveil land. |
| Razorverge Thicket | ✅ | GW fastland. |
| Thornglint Bridge | ⏳ | GW bridge. |
| Bleachbone Verge | ⏳ | WB verge. |
| Concealed Courtyard | ✅ | WB fastland. |
| Goldmire Bridge | ⏳ | WB bridge. |
| Shadowy Backstreet | ✅ | WB surveil land (catalog). |
| Riverpyre Verge | ⏳ | UR verge. |
| Silverbluff Bridge | ⏳ | UR bridge. |
| Spirebluff Canal | ✅ | UR fastland. |
| Thundering Falls | ⏳ | UR surveil land. |
| Blooming Marsh | ✅ | BG fastland (catalog). |
| Darkmoss Bridge | ⏳ | BG bridge. |
| Underground Mortuary | ⏳ | BG surveil land. |
| Wastewood Verge | ⏳ | BG verge. |
| Elegant Parlor | ⏳ | RW surveil land. |
| Inspiring Vantage | ✅ | RW fastland. |
| Rustvale Bridge | ⏳ | RW bridge. |
| Sunbaked Canyon | ⏳ | RW horizon canopy. |
| Botanical Sanctum | ✅ | UG fastland. |
| Hedge Maze | ⏳ | UG surveil land. |
| Tanglepool Bridge | ⏳ | UG bridge. |
| Waterlogged Grove | ⏳ | UG horizon canopy. |
| Twisted Landscape | ⏳ | Tri-color landcycle. |
| Sheltering Landscape | ⏳ | Tri-color landcycle. |
| Bountiful Landscape | ⏳ | Tri-color landcycle. |
| Ancient Den | ✅ | Artifact land — Plains. {T}: Add {W}. |
| Cloudpost | ⏳ | Locus mana scaling. |
| Darksteel Citadel | ✅ | Indestructible artifact land. {T}: Add {C}. |
| Evolving Wilds | ⏳ | Sac-search basic. |
| Exotic Orchard | ⏳ | Mana matching opponents' lands. |
| Glimmerpost | ⏳ | Locus + lifegain. |
| Great Furnace | ✅ | Artifact-Mountain. {T}: Add {R}. |
| Lotus Field | ⏳ | ETB sac-2-lands; tap for {GGG}/{UUU}/etc. |
| Planar Nexus | ⏳ | Tri-color rainbow. |
| Power Depot | ⏳ | Charge-counter mana storage. |
| Rishadan Port | ⏳ | Tap-to-tap-opp-land. |
| Seat of the Synod | ✅ | Artifact-Island. {T}: Add {U}. |
| Talon Gates of Madara | ⏳ | TBD. |
| Three Tree City | ⏳ | TBD. |
| Tree of Tales | ✅ | Artifact-Forest. {T}: Add {G}. |
| Trenchpost | ⏳ | TBD. |
| Vault of Whispers | ✅ | Artifact-Swamp. {T}: Add {B}. |

## Engine features needed

The cube draws on a much wider mechanic set than the BRG / Goryo's pair.
This list is the work needed to bring most of the cube online; the engine
features already implemented (Pact / Flashback / Convoke / Rebound / etc.)
are listed in `DECK_FEATURES.md`.

| Feature | Status | Cards depending on it |
|---|---|---|
| Sacrifice-as-activation-cost (`ActivatedAbility::sac_cost`) | ✅ | Mind Stone, Aether Spellbomb, Cathar Commando, Haywire Mite — and any token-sac mana ability (Treasure, Food, Blood, Clue). |
| Token activated abilities (`TokenDefinition::activated_abilities`) | ✅ | Treasure (`{T}, Sac: Add any color`), Food (`{2}, {T}, Sac: Gain 3 life`), Clue (`{2}, Sac: Draw 1`), Blood (loot). |
| Trigger-filter enforcement (`EventSpec::filter` evaluated in `dispatch_triggers_for_events` + `fire_spell_cast_triggers`) | ✅ | Up the Beanstalk, Temur Ascendancy, and any future "whenever you cast a spell with property X" / "whenever a creature enters with property Y" trigger. |
| Equipment + equip-cost activated ability | ⏳ | Lion Sash, Shuko, Lavaspur Boots, Nettlecyst, Sword of Body and Mind, Helm of the Host. |
| Vehicles + Crew | ⏳ | Smuggler's Copter, Esika's Chariot, Shorikai, Pinnacle Emissary, Coveted Jewel-adjacent. |
| Madness | ⏳ | Anje's Ravager, Blazing Rootwalla, Basking Rootwalla. |
| Cycling | ⏳ | Aether Spellbomb (cycle), Sundering Eruption-adjacent. |
| Adventure (cost-mode duality) | ⏳ | Virtue of Loyalty. |
| Mutate | ⏳ | Mutated Cultist, Mutable Explorer. |
| Cascade | ⏳ | Bloodbraid Challenger, Apex Devastator, Maelstrom Nexus. |
| Storm count + cast-from-top | ⏳ | Mind's Desire (Storm), Amped Raptor / Robber of the Rich (cast-from-top). |
| Delve cost reduction | ⏳ | Treasure Cruise, Dig Through Time, Lose Focus. |
| Ninjitsu | ⏳ | Fallen Shinobi (any future ninjas). |
| Soulbond | ⏳ | Deadeye Navigator. |
| Companion (deck-construction restriction + start-side mana cost) | ⏳ | Zirda, the Dawnwaker. |
| Saga lore counters + DFC | ⏳ | The Everflowing Well; future sagas. |
| Hideaway lands | ⏳ | Shelldock Isle. |
| Horizon-canopy "pay 1 + life to draw" lands | ⏳ | Horizon Canopy, Sunbaked Canyon, Waterlogged Grove. |
| Verge / surveil land family expansion | ⏳ | Each color pair's `*verge` and surveil-land entry. |
| Bridge artifact lands | ⏳ | All 10 `*bridge` entries. |
| Locus mana scaling | ⏳ | Cloudpost, Glimmerpost. |
| ETB-replacement effects (suppress entirely) | ⏳ | Containment Priest, Static Prison-adjacent, Gather Specimens. |
| Spell-tax statics ("costs {1} more", "costs at least {3}") | 🟡 | Damping Sphere wired (`AdditionalCostAfterFirstSpell`); Trinisphere needs a "minimum cost" flavor. Elite Spellbinder reuses the existing tax static. |
| Land-untap restriction static | ⏳ | Back to Basics. |
| "Cast spells without paying mana" static | ⏳ | Omniscience, Maelstrom Archangel (combat-damage variant), Aluren (free-cast under-3 creatures). |
| Name-a-card / name-a-creature-type primitive | 🟡 | Cavern of Souls has the ETB ChooseMode framing but the chosen type doesn't gate mana provenance yet. Pithing Needle would need the same primitive. |
| Token-copy of permanent | ⏳ | Phantasmal Image, Helm of the Host, Mockingbird, Growing Ranks (populate). |
| Multi-pick decisions over revealed library cards | 🟡 | Atraxa Draw-4 stand-in is wired. Reveal-and-sort by card type, Dig Through Time, Mind's Desire all need a richer multi-pick decision. |
| Investigate + Clue token | ⏳ | Tireless Tracker, Lonis, Proft's Eidetic Memory. |
| Dredge | ⏳ | Golgari Grave-Troll, Golgari Thug, Life from the Loam, Gitrog. |
| Landfall trigger | ⏳ | Bloodghast, Omnath. |
| Pact-style upkeep cost (`PayOrLoseGame`) | ✅ | Engine primitive already exists; reuse for Slaughter Pact, Pact of the Titan, Intervention Pact, etc. |
| Counter-spell / counter-ability primitives | ✅ | `CounterSpell`, `CounterUnlessPaid`, `CounterAbility` are all wired (Spell Snare, Daze, Tishana's Tidebinder, Mystical Dispute reuse them). |
| Flashback / Rebound | ✅ | Wired for Faithful Mending and Ephemerate; reusable by Dread Return, Dreadhorde Arcanist. |
| Surveil + Scry primitives | ✅ | Already wired (Quantum Riddler, surveil lands, Consider). |
| Loyalty abilities w/ static | 🟡 | Teferi works fully; needs further variants (Ashiok exile-and-cast, Karn -2 fetch, Saheeli token-copy, Sorin drain, Tezzeret etheric, Tamiyo). |
| Split / DFC modal cards (`Wear // Tear`) | ⏳ | Wear // Tear, Sundering Eruption (sorcery side), most "// Land" DFCs in the cube. |
| Living-weapon-on-ETB token | ⏳ | Nettlecyst, Sword-of-Body-and-Mind-adjacent. |
| Protection-from-color (more colors) | 🟡 | Engine has `Keyword::Protection(Color)`. Cards like Stillmoon Cavalier need toggle abilities + multi-color protection. |
| Charge counters as mana storage | 🟡 | Gemstone Mine wires charge counters + sac-on-empty. Coalition Relic, Power Depot, Pentad Prism, Chalice of the Void all reuse the same primitive. |

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
