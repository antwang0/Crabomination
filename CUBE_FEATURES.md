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
| Containment Priest | 🟡 | Body wired: 2/2 W flash. The replacement effect ("nontoken creatures entering not from a spell get exiled instead") needs an ETB-replacement primitive the engine doesn't have yet — the body is in the cube as a flash flier replacement until the primitive lands. Test: `containment_priest_is_a_flash_two_two`. |
| Lion Sash | ⏳ | Equipment + grow via exile-from-graveyard. Needs equipment + counters wiring. |
| Enduring Innocence | 🟡 | 2/1 Lifelink Glimmer; ETB-of-another-nontoken-creature → Draw 1. The "return from exile after death" self-revive clause is omitted. |
| Ranger-Captain of Eos | 🟡 | ETB tutor for ≤1-CMC creature. The sac-for-no-noncreature-spells static is omitted (no sac-as-cost activation primitive). |
| Tempt with Bunnies | ⏳ | Tempting offer (chain-creating) — needs multi-player choice primitive. |
| Virtue of Loyalty | ⏳ | Adventure + enchantment side. Needs Adventure cost-mode primitive. |

### Blue

| Card | Status | Notes |
|---|---|---|
| Dandân | 🟡 | {2}{U} 4/1 Fish. Wired with the "if you control no Island at your upkeep, sacrifice this" downside (an upkeep `If(Not(SelectorExists(Island ∧ ControlledByYou)), Move(self → Graveyard))` trigger). The "can attack only if defending player controls an Island" half is omitted (no per-attacker target restriction). Tests: `dandan_sacrifices_at_upkeep_when_no_island`, `dandan_stays_in_play_with_an_island`. |
| Phantasmal Image | 🟡 | Enters as a copy of any creature via the `enters_as_copy` CR-707 hook, keeping its Illusion type + the "sacrifice when targeted" rider. Copied ETB triggers don't re-fire. Tests: `phantasmal_image_copies_and_keeps_illusion_plus_sacrifice_rider`. (Clone is fully faithful — `clone_enters_as_a_copy_of_a_creature`.) |
| Thundertrap Trainer | 🟡 | 2/2 Flash; ETB taps target creature an opponent controls. Body fully wired; the trap/discount text is dropped. |
| Deadeye Navigator | ⏳ | Soulbond + activated flicker. Reuses Flicker primitive; needs Soulbond. |
| Consult the Star Charts | ⏳ | Look-at-top-N + draw — needs Foretell-adjacent decision. |
| Gather Specimens | ⏳ | Replace creature ETB control-shift. Replacement effect primitive. |
| Mirrorform | ⏳ | Aura + clone target. |
| Mind's Desire | ⏳ | Storm + cast-from-top. Needs Storm count + cast-from-top primitive. |
| The Everflowing Well | ⏳ | Saga land flip; needs Saga lore counters + DFC. |
| Proft's Eidetic Memory | ⏳ | Investigate + scaling +1/+1. Needs investigate (Clue). |
| Parallax Tide | ⏳ | Fading + exile lands. Needs fade counters. |
| Shelldock Isle | ⏳ | Hideaway land (DFC-like setup). |
| Sink into Stupor | ⏳ | Counter + DFC into Lair land. |
| Concealing Curtains | ⏳ | DFC into Revealing Eye creature. |

### Black

| Card | Status | Notes |
|---|---|---|
| Mutated Cultist | ⏳ | Mutate primitive needed. |
| Ichorid | 🟡 | {B} 3/1 Horror with Haste. `StepBegins(Upkeep)` + `FromYourGraveyard` trigger returns Selector::This to the battlefield, then schedules a `NextEndStep` delayed exile (reuses Goryo's reanimate-then-exile pattern). The "opponent's graveyard contains a black creature" gate is omitted (no graveyard-color trigger filter yet). Test: `ichorid_returns_at_upkeep_then_exiles_at_end_step`. |
| Necrotic Ooze | ⏳ | Gains all activated abilities of creatures in graveyards. Big ability-borrow primitive. |
| Indulgent Tormentor | 🟡 | `{3}{B}{B}` 5/3 Demon Flying. End-step trigger drains 3 life from each opponent. The full Oracle gives the opponent a choice ("you draw a card or sacrifice a creature; if not, lose 3 life") — we resolve straight to the most punishing line (lose 3) since the multi-player choice primitive isn't wired. Test: `indulgent_tormentor_drains_each_opponent_at_end_step`. |
| Doomsday Excruciator | 🟡 | 6/6 Flying Demon; ETB each player mills 20. The Doomsday deck-stacking text is approximated by the symmetric mill. |
| Metamorphosis Fanatic | ⏳ | Unknown — TBD. |
| Collective Brutality | 🟡 | {1}{B} Sorcery. 3-mode ChooseMode (discard IS from opp / -2/-2 creature / drain 2). Escalate omitted. Tests: `collective_brutality_mode_two_drains`. |
| Parallax Dementia | ⏳ | Fading + reanimate; needs fade counters. |
| Parallax Nexus | 🟡 | Enters with 5 charge counters; upkeep removes one and it's sacrificed at 0 (Fading approximation). `{0}`: target opponent discards a card (proxy for "exile target card from a hand"). |
| Unholy Annex // Ritual Chamber | ⏳ | DFC enchantment land. |

### Red

| Card | Status | Notes |
|---|---|---|
| Amped Raptor | 🟡 | 2/1 Dinosaur; ETB gains 2 life as a placeholder. The energy ({E}{E}) + cast-from-exile clause is omitted (no energy system). |
| Magda, Brazen Outlaw | 🟡 | {1}{R} 2/1 Legendary Dwarf Berserker. Static +1/+0 to Dwarves + "whenever a Dwarf you control becomes tapped, create a Treasure" (via the new `EventKind::Tapped` + CR 508.1f attack-tap event). Only the five-Treasure-sacrifice Dragon/artifact tutor is omitted. |
| Robber of the Rich | 🟡 | 2/2 Reach + Haste body. The attack-trigger exile + cast-from-opponent's-library clause is omitted. |
| Detective's Phoenix | 🟡 (was ⏳) | Push (claude/modern_decks batch 103): {2}{R} 2/2 Phoenix with Flying + Haste. Dies trigger schedules a `DelayUntil(NextEndStep)` body that returns Self to its owner's hand. Approximation of the printed "return from gy at end step if you control a Detective" — the conditional gate is collapsed (always returns). Test: `detectives_phoenix_dies_schedules_delayed_return`. |
| Simian Spirit Guide | 🟡 | `{2}{R}` 2/2 Ape Spirit. Body wired; the alt-cost "exile from hand to add {R}" half is still ⏳ — the existing `AlternativeCost` path replaces the entire spell's resolution, so an alt-cost mana ability would need a new "alt cast = mana ability" mode. Available in any red pool. |
| Goldspan Dragon | 🟡 | 4/4 Flying Haste; attack-trigger Treasure (using the now-functional Treasure mana ability). "Becomes target of a spell" trigger and the Treasure-2-mana static rider are omitted. |
| Chaos Warp | 🟡 | {2}{R} Instant. `Move(target Permanent → Library(OwnerOf, Shuffled))` — the new `LibraryPosition::Shuffled` engine path actually reshuffles the library. The "reveal top, cast if permanent" half is collapsed (info-only without a cast-from-top pipeline). Test: `chaos_warp_sends_target_permanent_to_owners_library`. |
| Pyrokinesis | 🟡 | {4}{R}{R} Instant. Pitch-cost alt cast: exile a red card from hand → 4 damage. The "divide 4 damage among any number of creatures" half is approximated as a single 4-damage hit. |
| Legion Extruder | ⏳ | Equip-ish artifact. |

### Green

| Card | Status | Notes |
|---|---|---|
| Collector Ouphe | 🟡 | 2/2 Ouphe body. Artifact-ability-lock static omitted. |
| Keen-Eyed Curator | 🟡 | {2}{G} 3/3 Elf Druid. ETB +1/+1 counter. Full gy-hate exile omitted. Tests: `keen_eyed_curator_etb_adds_counter`. |
| Elvish Spirit Guide | 🟡 | {2}{G} 2/2 Elf Spirit body wired. The "exile from hand: add {G}" alt-mana ability needs a hand-activated-ability primitive (`activate_ability` only walks the battlefield today); promote to ✅ once that lands. Test: `elvish_spirit_guide_is_a_two_two_elf_spirit`. |
| Enduring Vitality | ⏳ | Roomba-style return on death + creature mana untap. |
| Hauntwoods Shrieker | ⏳ | Token + transform. |
| Mossborn Hydra | 🟡 (was ⏳) | Push (claude/modern_decks batch 103): {X}{G} 0/0 Hydra. Enters with X +1/+1 counters via `enters_with_counters: Some((PlusOnePlusOne, XFromCost))`. The "double counters if X ≥ 4" rider is collapsed (no counter-multiplier-on-cast primitive). Test: `mossborn_hydra_enters_with_x_counters`. |
| Mutable Explorer | ⏳ | Mutate primitive. |
| Baloth Prime | ⏳ | TBD. |
| Icetill Explorer | ⏳ | TBD. |
| Mightform Harmonizer | ⏳ | TBD. |
| Ouroboroid | ⏳ | TBD. |
| Railway Brawler | ⏳ | Train (vehicle-like). |
| Springleaf Parade | ⏳ | TBD. |
| Aluren | ⏳ | Free-cast 3 or less creatures. |
| Shifting Woodland | ⏳ | DFC land. |

### Artifacts & Planeswalkers (mono / colorless)

| Card | Status | Notes |
|---|---|---|
| Tezzeret, Cruel Captain | 🟡 | {3}{B} 4-loyalty walker. **+1**: target creature gets -2/-2 EOT. **-2**: drain 2 life from each opponent (you gain 2). Static "your artifact creatures get +1/+1" wired; the ult remains collapsed. Tests: `tezzeret_minus_two_drains_each_opponent_for_two`, `tezzeret_plus_one_shrinks_target_creature`. |
| Karn, Scion of Urza | 🟡 | {4} 5-loyalty Karn. **+1**: Draw 1 + mill 1 (the opp-pile-split is information-only at this engine fidelity). **-1**: ForEach Construct creature you control + AddCounter(+1/+1). **-2**: Create a 1/1 Construct artifact creature token (artifact-count scaling rider collapses). Tests: `karn_scion_of_urza_minus_two_creates_a_construct_token`, `karn_plus_one_draws_a_card_and_mills_one`. |
| Kozilek's Command | 🟡 | {X} Instant wired as a 3-mode `ChooseMode`. The printed "choose two" modal picker collapses to a single mode. |
| Eldrazi Confluence | 🟡 | {4} Instant wired as a 3-mode `ChooseMode`. The "choose three, modes may repeat" charm structure collapses to one pick. |
| Pithing Needle | 🟡 | {1} Artifact body-only stub. Name-a-card + ability suppression static omitted. |
| Agatha's Soul Cauldron | ⏳ | Borrow activated abilities of exiled creatures. |
| Fellwar Stone | 🟡 | {T}: Add one mana of any color. (Approximation: drops the "matches opponent's lands" restriction — engine has no per-source mana provenance yet.) |
| Nettlecyst | ⏳ | Living-equipment + token. |
| Sword of Body and Mind | ⏳ | Equipment + protection + token + mill. |
| Helm of the Host | ⏳ | Equipment that token-copies on attack. |
| The Mightstone and Weakstone | 🟡 | {5} Artifact; ETB `ChooseMode` (Draw 2 / target creature -5/-5 EOT); `{T}`: Add {C}{C}. Meld/assemble omitted. |
| Coveted Jewel | 🟡 | {6} Artifact; ETB Draw 3; `{T}`: Add 3 mana of any color. The "steal on combat damage / must-attack" control mechanic is omitted. |
| The Endstone | ⏳ | TBD. |

### Multicolor

| Card | Status | Notes |
|---|---|---|
| Shorikai, Genesis Engine | ⏳ | Vehicle-walker hybrid; loots on activate. |
| Ashiok, Nightmare Weaver | 🟡 | Push (claude/modern_decks batch 102): {1}{U}{B} 3-loyalty Planeswalker. **+2**: target opponent mills 3 (the "exiled with Ashiok" linkage is engine-wide ⏳ — milled cards land in opp graveyard). **-1**: Exile target opp creature (the "create a copy" half collapses). **-10**: Approximated as `WinGame { You }` (the "each opp draws 7 from exile" plinker ultimate is dropped). Tests: `ashiok_nightmare_weaver_plus_two_mills_opponent_three`, `ashiok_nightmare_weaver_minus_one_exiles_creature`. |
| Fallen Shinobi | 🟡 | 5/4 Zombie Ninja. Combat damage mills 2 from defender. Ninjutsu and play-from-exile omitted. |
| Carnage Interpreter | 🟡 (was ⏳) | Push (claude/modern_decks batch 103): {2}{B}{R} Vampire 4/3 with Trample. ETB makes each opponent discard a random card. (Synthesised body; the real Oracle has more text.) Test: `carnage_interpreter_etb_makes_each_opp_discard`. |
| Bloodbraid Challenger | ⏳ | Cascade. The `Effect::Cascade`/`shortcut::cascade(mv)` primitive now exists (Bloodbraid Elf, Apex Devastator); this card was left ⏳ only because its exact printed stats could not be Scryfall-verified this run (the api.scryfall.com host is blocked by the environment network policy). |
| Brightglass Gearhulk | 🟡 (was ⏳) | Push (claude/modern_decks batch 103): {4} Artifact Creature — Construct 4/4. ETB Scry 2 + Draw 1. (Real card likely has more text; ships as a colorless 4-mana cantripping body.) Test: `brightglass_gearhulk_etb_scries_and_draws`. |
| Torsten, Founder of Benalia | 🟡 | 7/7 Legendary Human Soldier. ETB searches 3 basic lands to battlefield tapped. |
| Sorin, Grim Nemesis | 🟡 | Push (claude/modern_decks batch 102): {4}{B}{B} 6-loyalty Planeswalker. **+1**: Draw 1 + Lose 3 life (approximation; reveal/MV-life-loss/conditional-token chain dropped). **-X**: ping (the X-cost loyalty path uses `Value::XFromCost` against creature/PW + 1 gain life). **-9**: drain 10 from each opponent (the printed "X = cards in opp's graveyard" scaling collapses). Tests: `sorin_grim_nemesis_plus_one_draws_and_loses_three_life`, `sorin_grim_nemesis_minus_nine_drains_each_opponent`. |
| Pinnacle Emissary | ⏳ | TBD. |
| Saheeli Rai | 🟡 | Push (claude/modern_decks batch 102): {1}{U}{R} 3-loyalty Planeswalker. **+1**: Scry 1 + ping each opponent for 1 (the "and each PW they control" half drops — no `EachOpponentsPlaneswalker` selector). **-2**: Create a token copy of target friendly creature/artifact, grant haste, delay-trigger Exile at next end step. **-7**: Same body fired twice (the emblem-recurring "each end step" auto-recur is approximated). Tests: `saheeli_rai_plus_one_pings_each_opponent`, `saheeli_rai_minus_two_creates_haste_copy`. |
| Broodspinner | ⏳ | TBD. |
| The Gitrog Monster | ⏳ | Land-as-discard / dredge engine. |
| Wear // Tear | 🟡 | Push (claude/modern_decks batch 102): {1}{R} Sorcery, single-spell approximation that destroys target artifact OR enchantment. The Split-Card primitive (CR 709) is engine-wide ⏳ — both halves collapse to a single faithful effect at the Wear cost. Fuse mode is dropped. Test: `wear_tear_destroys_target_artifact`. |
| Zirda, the Dawnwaker | ⏳ | Companion + activated-cost reduction. Needs Companion primitive. |
| Lonis, Genetics Expert | 🟡 (was ⏳) | Push (claude/modern_decks batch 103): {1}{G}{U} Legendary 2/2 Otter Detective. ETB trigger on other creatures you control creates a Clue token (the printed "investigate" rider via `clue_token()`). The "Sacrifice X Clues: target opp reveals top X cards" activated ability is engine-wide ⏳ (no per-activation X-cost prompt yet). Test: `lonis_genetics_expert_creates_clue_when_other_creature_enters`. |
| Tamiyo, Collector of Tales | 🟡 | Push (claude/modern_decks batch 102): {2}{G}{U} 4-loyalty Planeswalker. **-2**: Return target card from gy → hand. **-3**: Search library → hand (the "same name as a card in target opponent's graveyard" filter is engine-wide ⏳ — falls back to `Any`). **-7**: Draw 4 (the "distinct nonland types in gy" scaling drops). The static "spells your opps control can't make you discard or sac" is engine-wide ⏳. Test: `tamiyo_collector_minus_two_returns_card_from_graveyard`. |
| Sab-Sunen, Luxa Embodied | ⏳ | TBD. |
| Kestia, the Cultivator | ⏳ | Aura/enchantment matters. |
| Messenger Falcons | 🟡 | 2/2 Flying Bird. ETB draw a card. GW cross-pool. |
| Dakkon, Shadow Slayer | 🟡 | WUB Legendary Planeswalker. +1: Surveil 2. -3: Exile target creature. Emblem ult omitted. |
| Urza, Chief Artificer | ⏳ | Planeswalker / commander. |
| Geyadrone Dihada | 🟡 | Push (claude/modern_decks batch 102): {2}{B}{R} 3-loyalty Planeswalker. **+1**: Each opp loses 1 + you draw 1 (the "if you have less life than an opp, reset loyalty" rider drops — no loyalty-set primitive). **-3**: Threaten — `GainControl(EOT) + Untap + GrantKeyword(Haste, EOT)`. **-7**: Each opp loses 10 (half-life approximation). Tests: `geyadrone_dihada_plus_one_drains_each_opponent_for_one`, `geyadrone_dihada_minus_three_steals_creature`. |
| Lord Xander, the Collector | 🟡 | Push (claude/modern_decks batch 102): {3}{U}{B}{R} 6/6 Flying Legendary Vampire Demon Noble. ETB makes target opp discard 3 (`DiscardChosen`). Attack trigger mills each opp 8 (the "half their library" scaling collapses to a fixed midgame value). Die trigger makes each opp sacrifice 3 nonland permanents (the "half their permanents" scaling collapses). Test: `lord_xander_the_collector_etb_makes_opponent_discard_three`. |
| Loot, the Pathfinder | 🟡 (was ⏳) | Push (claude/modern_decks batch 103): {1}{G}{W} Legendary 2/3 Otter Scout with Vigilance. ETB creates a Clue token (Map token primitive — CR 111.10s explore-token — is engine-wide ⏳; the Clue approximation gives the same "sac for value" pattern via the existing primitive). Test: `loot_the_pathfinder_etb_creates_map_approximation`. |
| Dragonback Assault | ⏳ | TBD. |
| Rediscover the Way | ⏳ | TBD. |
| Shiko and Narset, Unified | ⏳ | TBD. |
| Fangkeeper's Familiar | ⏳ | TBD. |
| Teval, Arbiter of Virtue | ⏳ | TBD. |
| Muldrotha, the Gravetide | ⏳ | Cast from graveyard each turn (one of each card type). |
| Rakshasa's Bargain | 🟡 | Pay 4 life + Draw 4. The "exile creature card from your graveyard" alternate additional cost is folded away (modal additional-cost not modeled). |
| Omnath, Locus of Creation | ⏳ | Landfall-quad-color. |
| Atraxa, Grand Unifier | 🟡 | Already wired with ETB Draw 4 approximation. Real reveal-and-sort still ⏳. |
| Leyline of the Guildpact | ⏳ | "Your permanents are all colors." Needs is-all-colors primitive. |
| Maelstrom Nexus | ⏳ | Cascade-on-first-spell static. Needs first-spell-of-turn tracking + a static that injects a cascade trigger; the `Effect::Cascade` primitive now exists (used by Bloodbraid Elf / Apex Devastator), so this is "wire the static + the per-turn gate". |

### Lands

| Card | Status | Notes |
|---|---|---|
| Blazemire Verge | ⏳ | BR DFC verge. |
| Thornspire Verge | ⏳ | RG verge. |
| Bleachbone Verge | ⏳ | WB verge. |
| Riverpyre Verge | ⏳ | UR verge. |
| Wastewood Verge | ⏳ | BG verge. |
| Twisted Landscape | ⏳ | Tri-color landcycle. |
| Sheltering Landscape | ⏳ | Tri-color landcycle. |
| Bountiful Landscape | ⏳ | Tri-color landcycle. |
| Planar Nexus | 🟡 | Land, ETB tapped, `{T}`: Add one mana of any color — a rainbow tapland. The basic-land-type / fetch riders collapse. |
| Power Depot | ⏳ | Charge-counter mana storage. |
| Talon Gates of Madara | ⏳ | TBD. |
| Three Tree City | 🟡 | Legendary Land. Enters with 3 charge counters; {T}, remove counter: add any color. Sacrifice-on-empty omitted. |
| Trenchpost | 🟡 | Locus Land. {T}: Add {C}{C}. Locus-count scaling omitted. |

## Engine features needed

The cube draws on a much wider mechanic set than the BRG / Goryo's pair.
This list is the work needed to bring most of the cube online; the engine
features already implemented (Pact / Flashback / Convoke / Rebound / etc.)
are listed in `DECK_FEATURES.md`.

| Feature | Status | Cards depending on it |
|---|---|---|
| Equipment + equip-cost activated ability | 🟡 | `GameAction::Equip` + `equipped_bonus` ship Shuko, Lavaspur Boots (✅). Lion Sash (counter-scaled bonus), Nettlecyst, Sword of Body and Mind, Helm of the Host still need dynamic/triggered equip bonuses. |
| Cycling | ✅ | Engine support complete: `Keyword::Cycling(cost)` + `GameAction::Cycle` (`cycle_card`) pay the cost, discard, draw, and fire `CardCycled`. Catalog cards already use it. The listed cube cards (Aether Spellbomb, Sundering Eruption-adjacent) just aren't added yet. |
| Adventure (cost-mode duality) | ⏳ | Virtue of Loyalty. |
| Mutate | ⏳ | Mutated Cultist, Mutable Explorer. |
| Storm count + cast-from-top | 🟡 | Storm count is wired (`Value::StormCount`, `Keyword::Storm`, per-turn `spells_cast_this_turn`) and `Effect::CopySpell` exists. Cast-from-top / cast-from-exile (Mind's Desire, Amped Raptor, Robber of the Rich) is still ⏳. |
| Ninjitsu | ⏳ | Fallen Shinobi (any future ninjas). |
| Soulbond | ⏳ | Deadeye Navigator. |
| Companion (deck-construction restriction + start-side mana cost) | ⏳ | Zirda, the Dawnwaker. |
| Saga lore counters + DFC | ⏳ | The Everflowing Well; future sagas. |
| Hideaway lands | ⏳ | Shelldock Isle. |
| Horizon-canopy "pay 1 + life to draw" lands | ⏳ | Horizon Canopy, Sunbaked Canyon, Waterlogged Grove. |
| Verge / surveil land family expansion | ⏳ | Each color pair's `*verge` and surveil-land entry. |
| ETB-replacement effects (suppress entirely) | ⏳ | Containment Priest, Static Prison-adjacent, Gather Specimens. |
| Spell-tax statics ("costs {1} more", "costs at least {3}") | 🟡 | Damping Sphere wired (`AdditionalCostAfterFirstSpell`); Trinisphere needs a "minimum cost" flavor. Elite Spellbinder reuses the existing tax static. |
| "Cast spells without paying mana" static | ⏳ | Omniscience, Maelstrom Archangel (combat-damage variant), Aluren (free-cast under-3 creatures). |
| Name-a-card / name-a-creature-type primitive | 🟡 | Cavern of Souls has the ETB ChooseMode framing but the chosen type doesn't gate mana provenance yet. Pithing Needle would need the same primitive. |
| Token-copy of permanent | ⏳ | Phantasmal Image, Helm of the Host, Mockingbird, Growing Ranks (populate). |
| Multi-pick decisions over revealed library cards | 🟡 | Atraxa Draw-4 stand-in is wired. Reveal-and-sort by card type, Dig Through Time, Mind's Desire all need a richer multi-pick decision. |
| Investigate + Clue token | 🟡 | Clue tokens ship (`clue_token()`; Tireless Tracker, Lonis, Loot create them). The Investigate keyword-action naming and "sacrifice a Clue" payoff abilities are still ⏳. |
| Landfall trigger | 🟡 | Bloodghast wired via the new `EventScope::FromYourGraveyard` (graveyard-source `LandPlayed` trigger). Standard battlefield-side landfall (Omnath) still uses the existing `LandPlayed` + `YourControl` path; both are functional. |
| Loyalty abilities w/ static | 🟡 | Teferi works fully; needs further variants (Ashiok exile-and-cast, Karn -2 fetch, Saheeli token-copy, Sorin drain, Tezzeret etheric, Tamiyo). |
| Split / DFC modal cards (`Wear // Tear`) | ⏳ | Wear // Tear, Sundering Eruption (sorcery side), most "// Land" DFCs in the cube. |
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
