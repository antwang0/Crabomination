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

## Commander target decks (`crabomination::pod::decks`)

The pod runner's fixed field. Every card is implemented; legality is asserted
by `pod::tests::cr_903_5a_target_decks_are_legal_commander_decks`, which runs
each list through `format::validate_commander_deck` (100 cards including the
commander, singleton outside basics, CR 903.4 identity subset, ban list).
Every card was also checked against Scryfall's `legalities.commander` when the
lists were picked.

| Deck | Commander | Identity | Cards | State |
|---|---|---|---|---|
| Sigarda GW | Sigarda, Host of Herons | GW | 100 | ✅ complete |
| Judith BR | Judith, the Scourge Diva | BR | 100 | ✅ complete |
| Hanna UW | Hanna, Ship's Navigator | UW | 100 | ✅ complete |
| Tatyova GU | Tatyova, Benthic Druid | GU | 100 | ✅ complete |
| Krark/Rograkh R | Krark, the Thumbless **+** Rograkh, Son of Rohgahh (Partner) | R | 98 + 2 | ✅ complete |
| Edgar Markov BRW | Edgar Markov (Eminence) | BRW | 100 | ✅ complete |
| Freyalise G | Freyalise, Llanowar's Fury (**planeswalker**, CR 903.3a) | G | 100 | ✅ complete |
| Zellix + Background UR | Zellix, Sanity Flayer **+** Passionate Archaeologist (**Choose a Background**, CR 702.124k) | UR | 98 + 2 | ✅ complete |
| Yuriko UB | Yuriko, the Tiger's Shadow (**commander ninjutsu**, CR 702.49d) | UB | 100 | ✅ complete |
| Adriana RW | Adriana, Captain of the Guard (**melee**, CR 702.121) | RW | 100 | ✅ complete |
| **Sultai Arisen** (TDC precon) BGU | Teval, the Balanced Scale | BGU | 100 | 🟡 all 100 implemented, 8 carry residuals (below) |
| **Mind Flayarrrs** (CLB precon) UB | Captain N'ghathrod | UB | 100 | 🟡 all 100 implemented, 5 carry residuals (below) |
| **Blood Rites** (LCC precon) WB | Clavileño, First of the Blessed | WB | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Heads I Win, Tails You Lose** (SLD) UR | Zndrsplt, Eye of Wisdom **+** Okaun, Eye of Chaos (Partner with) | UR | 98 + 2 | ✅ complete |
| **Goblin Storm** (SLD) R | Zada, Hedron Grinder | R | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Wretched Ranks** (FDC precon) B | Ghoulcaller Gisa | B | 100 | ✅ complete |
| **Tramplesaurus Rex** (FDC precon) G | Ghalta, Primal Hunger | G | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Keen Engineering** (FDC precon) U | Sai, Master Thopterist | U | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Reap the Tides** (CMR precon) GU | Aesi, Tyrant of Gyre Strait | GU | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Corrupting Influence** (ONC precon) WBG | Ixhel, Scion of Atraxa | WBG | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Sneak Attack** (ZNC precon) UB | Anowon, the Ruin Thief | UB | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Jeskai Striker** (TDC precon) URW | Shiko and Narset, Unified | URW | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Sliver Swarm** (CMM precon) WUBRG | Sliver Gravemother | WUBRG | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Quick Draw** (OTC precon) UR | Stella Lee, Wild Card | UR | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Vampiric Bloodlust** (C17 precon) BRW | Edgar Markov | BRW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Calling All Angels** (FDC precon) W | Giada, Font of Hope | W | 100 | ✅ complete |
| **Guided by Nature** (C14 precon) G | Freyalise, Llanowar's Fury (**planeswalker**) | G | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Animated Army** (BLC precon) RG | Bello, Bard of the Brambles | RG | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Grave Danger** (SCD precon) UB | Gisa and Geralf | UB | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Forged in Stone** (C14 precon) W | Nahiri, the Lithomancer (**planeswalker**) | W | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Swell the Host** (C15 precon) GU | Ezuri, Claw of Progress | GU | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Angels: They're Just Like Us** (SLD precon) W | Gisela, the Broken Blade (**meld**) | W | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Built From Scratch** (C14 precon) R | Daretti, Scrap Savant (**planeswalker**) | R | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Vampiric Bloodline** (VOC precon) BR | Strefan, Maurer Progenitor | BR | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Plunder the Graves** (C15 precon) BG | Meren of Clan Nel Toth | BG | 100 | ✅ complete |
| **Graveyard Overdrive** (M3C precon) BRG | Disa the Restless | BRG | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Sworn to Darkness** (C14 precon) B | Ob Nixilis of the Black Oath (**planeswalker**) | B | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Seize Control** (C15 precon) UR | Mizzix of the Izmagnus | UR | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Rebellion Rising** (ONC precon) RW | Neyali, Suns' Vanguard | RW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Feline Ferocity** (C17 precon) GW | Arahbo, Roar of the World | GW | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Primal Genesis** (C19 precon) RGW | Ghired, Conclave Exile | RGW | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Breed Lethality** (C16 precon) WUBG | Atraxa, Praetors' Voice | WUBG | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Call the Spirits** (C15 precon) WB | Daxos the Returned | WB | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Peer Through Time** (C14 precon) U | Teferi, Temporal Archmage (**planeswalker**) | U | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Quantum Quandrix** (C21 precon) GU | Adrix and Nev, Twincasters | GU | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Lorehold Legacies** (C21 precon) RW | Osgir, the Reconstructor | RW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Counterpunch** (CMD precon) WBG | Ghave, Guru of Spores | WBG | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Wade into Battle** (C15 precon) RW | Kalemne, Disciple of Iroas | RW | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Chaos Incarnate** (SCD) BR | Kardur, Doomscourge | BR | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Heavenly Inferno** (CMD precon) RWB | Kaalia of the Vast | RWB | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Political Puppets** (CMD precon) URW | Zedruu the Greathearted | URW | 100 | 🟡 all 100 implemented (Trade Secrets, banned, swapped for Divination), 2 carry residuals (below) |
| **Eternal Might** (DRC precon) WUB | Temmet, Naktamun's Will | WUB | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Land's Wrath** (ZNC precon) RGW | Obuun, Mul Daya Ancestor | RGW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Stalwart Unity** (C16 precon) RGWU | Kynaios and Tiro of Meletis | RGWU | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Token Triumph** (SCD starter) GW | Emmara, Soul of the Accord | GW | 100 | ✅ complete |
| **Raining Cats and Dogs** (SLD) RGW | Rin and Seri, Inseparable | RGW | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Growing Threat** (MOC precon) WB | Brimaz, Blight of Oreskos | WB | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Call for Backup** (MOC precon) RGW | Bright-Palm, Soul Awakener | RGW | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **World Shaper** (EOC precon) BRG | Hearthhull, the Worldseed | BRG | 100 | 🟡 all 100 implemented, 5 carry residuals (below) |
| **Nature's Vengeance** (C18 precon) BRG | Lord Windgrace | BRG | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Peace Offering** (BLC precon) GWU | Ms. Bumbleflower | GWU | 100 | 🟡 all 100 implemented, 5 carry residuals (below) |
| **Nature of the Beast** (C13 precon) RGW | Marath, Will of the Wild | RGW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Reign of Dragons** (FDC precon) R | Lathliss, Dragon Queen | R | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Spirit Squadron** (VOC precon) WU | Millicent, Restless Revenant | WU | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Jump Scare!** (DSC precon) GU | Zimone, Mystery Unraveler | GU | 100 | 🟡 all 100 implemented, 5 carry residuals (below) |
| **Obscura Operation** (NCC precon) WUB | Kamiz, Obscura Oculus | WUB | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Enduring Enchantments** (CMM precon) WBG | Anikthea, Hand of Erebos | WBG | 100 | 🟡 all 100 implemented, 5 carry residuals (below) |
| **Open Hostility** (C16 precon) WBRG | Saskia the Unyielding | WBRG | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Devour for Power** (CMD precon) BGU | The Mimeoplasm | BGU | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Mirror Mastery** (CMD precon) GUR | Riku of Two Reflections | GUR | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **First Flight** (SCD precon) WU | Isperia, Supreme Judge | WU | 100 | ✅ complete |
| **Undead Unleashed** (MIC precon) UB | Wilhelt, the Rotcleaver | UB | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Arm for Battle** (CMR precon) RW | Wyleth, Soul of Steel | RW | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Fae Dominion** (WOC precon) UB | Tegwyll, Duke of Splendor | UB | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Invent Superiority** (C16 precon) WUBR | Breya, Etherium Shaper | WUBR | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Entropic Uprising** (C16 precon) UBRG | Yidris, Maelstrom Wielder | UBRG | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Endless Punishment** (DSC precon) BR | Valgavoth, Harrower of Souls | BR | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Arcane Wizardry** (C17 precon) UBR | Inalla, Archmage Ritualist | UBR | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Urza's Iron Alliance** (BRC precon) WUB | Urza, Chief Artificer | WUB | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Phantom Premonition** (KHC precon) WU | Ranar the Ever-Watchful | WU | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Squirreled Away** (BLC precon) BG | Hazel of the Rootbloom | BG | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Evasive Maneuvers** (C13 precon) GWU | Derevi, Empyrial Tactician | GWU | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Draconic Destruction** (SCD starter) RG | Atarka, World Render | RG | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Family Matters** (BLC precon) URW | Zinnia, Valley's Voice | URW | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Virtue and Valor** (WOC precon) GW | Ellivere of the Wild Court | GW | 100 | 🟡 all 100 implemented, 7 carry residuals (below) |
| **Divine Convocation** (MOC precon) URW | Kasla, the Broken Halo | URW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Symbiotic Swarm** (C20 precon) WBG | Kathril, Aspect Warper | WBG | 100 | 🟡 all 100 implemented, 6 carry residuals (below) |
| **Painbow** (DMC precon) WUBRG | Jared Carthalion | WUBRG | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Hatsune Miku** (SLD precon) GW | Trostani, Selesnya's Voice | GW | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Witherbloom Witchcraft** (C21 precon) BG | Willowdusk, Essence Seer | BG | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Witherbloom Pestilence** (SOC precon) BG | Dina, Essence Brewer | BG | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Silverquill Statement** (C21 precon) WB | Breena, the Demagogue | WB | 100 | 🟡 all 100 implemented, 7 carry residuals (below) |
| **Draconic Domination** (C17 precon) WUBRG | The Ur-Dragon | WUBRG | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Silverquill Influence** (SOC precon) WB | Killian, Decisive Mentor | WB | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Blame Game** (MKC precon) RW | Nelly Borca, Impulsive Accuser | RW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Temur Roar** (TDC precon) GUR | Eshki, Temur's Roar | GUR | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Adaptive Enchantment** (C18 precon) GWU | Estrid, the Masked (**planeswalker**) | GWU | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Aura of Courage** (AFC precon) GWU | Galea, Kindler of Hope | GWU | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Upgrades Unleashed** (NEC precon) RG | Chishiro, the Shattered Blade | RG | 100 | 🟡 all 100 implemented, 4 carry residuals (Concord with the Kami, Agitator Ant, Forgotten Ancient, Shifting Shadow) |
| **Mardu Surge** (TDC precon) RWB | Zurgo Stormrender | RWB | 100 | 🟡 all 100 implemented, 1 carries a residual (Gix, Yawgmoth Praetor) |
| **Timeless Wisdom** (C20 precon) URW | Gavi, Nest Warden | URW | 100 | 🟡 all 100 implemented, 4 carry residuals (Akim, the Soaring Wind, Crystalline Resonance, Ethereal Forager, Nimble Obstructionist) |
| **Miracle Worker** (DSC precon) WUB | Aminatou, Veil Piercer | WUB | 100 | 🟡 all 100 implemented, 6 carry residuals (INCOMPLETE_CARDS) |
| **Ruthless Regiment** (C20 precon) RWB | Jirina Kudro | RWB | 100 | 🟡 all 100 implemented, 2 carry residuals (Sanctuary Blade, Odric, Master Tactician) |
| **Most Wanted** (OTC precon) RWB | Olivia, Opulent Outlaw | RWB | 100 | 🟡 all 100 implemented, 3 carry residuals (Back in Town, Dire Fleet Ravager, Vihaan, Goldwaker) |
| **Tricky Terrain** (M3C precon) GU | Omo, Queen of Vesuva | GU | 100 | 🟡 all 100 implemented, 7 carry residuals (Omo, Horizon of Progress, Desert Warfare, Sunken Palace, Magus of the Candelabra, Rampant Frogantua, March from Velis Vel) |
| **Everyone's Invited!** (SLD) WUBRG | Morophon, the Boundless | WUBRG | 100 | 🟡 all 100 implemented, 7 carry residuals (Amoeboid Changeling, Nameless Inversion, Shields of Velis Vel, Moritte of the Frost, Unsettled Mariner, Stick Together, Harper Recruiter) |
| **Planar Portal** (AFC precon) BR | Prosper, Tome-Bound | BR | 100 | 🟡 all 100 implemented, 5 carry residuals (Karazikar, Hellish Rebuke, Dead Man's Chest, Share the Spoils, Danse Macabre) |
| **Prismari Artistry** (SOC precon) UR | Rootha, Mastering the Moment | UR | 100 | 🟡 all 100 implemented, 2 carry residuals (Abstract Performance, Plargg and Nassari) |
| **Desert Bloom** (OTC precon) RGW | Yuma, Proud Protector | RGW | 100 | 🟡 all 100 implemented, 2 carry residuals (Cataclysmic Prospecting, Dune Chanter) |
| **Lorehold Spirit** (SOC precon) RW | Quintorius, History Chaser (**planeswalker**) | RW | 100 | 🟡 all 100 implemented, 3 carry residuals (Ao, the Dawn Sky, Quintorius, Loremaster, Serra Paragon) |
| **Dungeons of Death** (AFC precon) WUB | Sefris of the Hidden Ways | WUB | 100 | 🟡 all 100 implemented, 4 carry residuals (Grave Endeavor, Nihiloor, Phantom Steed, Rod of Absorption) |
| **Deadly Disguise** (MKC precon) RGW | Kaust, Eyes of the Glade | RGW | 100 | 🟡 all 100 implemented, 4 carry residuals (Boltbender, Tesak, Unexplained Absence, Veiled Ascension) |
| **Quandrix Unlimited** (SOC precon) GU | Zimone, Infinite Analyst | GU | 100 | 🟡 all 100 implemented, 3 carry residuals (Kinetic Ooze, Primo, Unbound Flourishing) |
| **Maestros Massacre** (NCC precon) UBR | Anhelo, the Painter | UBR | 100 | 🟡 all 100 implemented, 7 carry residuals (Maestros Confluence, Parnesse, Sinister Concierge, Syrix, Waste Management, Xander's Pact, Zndrsplt's Judgment) |
| **Deep Clue Sea** (MKC precon) GWU | Morska, Undersea Sleuth | GWU | 100 | 🟡 all 100 implemented, 3 carry residuals (Aerial Extortionist, Alandra, Sky Dreamer, Erdwal Illuminator) |
| **Counter Intelligence** (EOC precon) URW | Inspirit, Flagship Vessel (**Spacecraft**) | URW | 100 | 🟡 all 100 implemented, 6 carry residuals (Cloud Key, Inspirit, Depthshaker Titan, Moxite Refinery, Resourceful Defense, Ripples of Potential) |
| **Coven Counters** (MIC precon) GW | Leinore, Autumn Sovereign | GW | 100 | 🟡 all 100 implemented, 4 carry residuals (Curse of Conformity, Celestial Judgment, Sigardian Zealot, Moorland Rescuer) |
| **Arcane Maelstrom** (C20 precon) GUR | Kalamax, the Stormsire | GUR | 100 | 🟡 all 100 implemented, 4 carry residuals (Eon Frolicker, Haldan, Pako, Lavabrink Floodgates) |
| **Enhanced Evolution** (C20 precon) BGU | Otrimi, the Ever-Playful | BGU | 100 | 🟡 all 100 implemented, 4 carry residuals (Capricopian, Manascape Refractor, Mindleecher, Vastwood Hydra) |
| **Riveteers Rampage** (NCC precon) BRG | Henzie "Toolbox" Torre | BRG | 100 | 🟡 all 100 implemented, 7 carry residuals (below) |
| **Grand Larceny** (OTC precon) BGU | Gonti, Canny Acquisitor | BGU | 100 | 🟡 all 100 implemented, 5 carry residuals (below) |
| **Party Time** (CLB precon) WB | Nalia de'Arnise | WB | 100 | 🟡 all 100 implemented, 2 carry residuals (Calculating Lich, Glorious Protector) |
| **Faceless Menace** (C19 precon) BGU | Kadena, Slinking Sorcerer | BGU | 100 | 🟡 all 100 implemented, 5 carry residuals (Gift of Doom, Rayami, Road of Return, Vesuvan Shapeshifter, Volrath) |
| **Cavalry Charge** (MOC precon) WUB | Sidar Jabari of Zhalfir | WUB | 100 | 🟡 all 100 implemented, 3 carry residuals (Path of the Enigma, Syr Elenora, Aryel) |
| **Eternal Bargain** (C13 precon) WUB | Oloro, Ageless Ascetic | WUB | 100 | 🟡 all 100 implemented, 4 carry residuals (Order of Succession, Lim-Dûl's Vault, Springjack Pasture, Serene Master) |
| **Power Hungry** (C13 precon) BRG | Prossh, Skyraider of Kher | BRG | 100 | 🟡 all 100 implemented, 4 carry residuals (Sudden Demise, Night Soil, Widespread Panic, Capricious Efreet) |
| **Living Energy** (DRC precon) GUR | Saheeli, Radiant Creator | GUR | 100 | 🟡 all 100 implemented, 4 carry residuals (Aetherflux Conduit, Territorial Aetherkite, Rampaging Aetherhood, Saheeli) |
| **Exit from Exile** (CLB precon) RG | Faldorn, Dread Wolf Herald | RG | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Exquisite Invention** (C18 precon) UR | Saheeli, the Gifted (**planeswalker**) | UR | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Mishra's Burnished Banner** (BRC precon) UBR | Mishra, Eminent One | UBR | 100 | 🟡 all 100 implemented, 6 carry residuals (below) |
| **Tinker Time** (MOC precon) GUR | Gimbal, Gremlin Prodigy | GUR | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Legends' Legacy** (DMC precon) RWB | Dihada, Binder of Wills (**planeswalker**) | RWB | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Planeswalker Party** (CMM precon) URW | Commodore Guff (**planeswalker**) | URW | 100 | 🟡 all 100 implemented, 7 carry residuals (below) |
| **Eldrazi Incursion** (M3C precon) WUBRG | Ulalek, Fused Atrocity | WUBRG | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Eldrazi Unbound** (CMM precon) C | Zhulodok, Void Gorger | C | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Subjective Reality** (C18 precon) WUB | Aminatou, the Fateshifter (**planeswalker**) | WUB | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Death Toll** (DSC precon) BG | Winter, Cynical Opportunist | BG | 100 | 🟡 all 100 implemented, 6 carry residuals (below) |
| **Mind Seize** (C13 precon) UBR | Jeleva, Nephalia's Scourge | UBR | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **20 Ways to Win** (SLD) WUBRG | Go-Shintai of Life's Origin | WUBRG | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Cabaretti Cacophony** (NCC precon) RGW | Kitt Kanto, Mayhem Diva | RGW | 100 | 🟡 all 100 implemented, 5 carry residuals (below) |
| **Draconic Rage** (AFC precon) RG | Vrondiss, Rage of Ancients | RG | 100 | 🟡 all 100 implemented, 6 carry residuals (below) |
| **Draconic Dissent** (CLB precon) UR | Firkraag, Cunning Instigator | UR | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Prismari Performance** (C21 precon) UR | Zaffai, Thunder Conductor | UR | 100 | 🟡 all 100 implemented, 5 carry residuals (below) |

The **ninth** is the pod's only **commander ninjutsu** seat (CR 702.49d) and
the only one whose commander leaves the command zone by an action that is not
a cast, so CR 903.8's tax never applies to that route — `commander_cast_count`
stays where it was and every ninja after the first is free. It is **after**
`pod_field(8)` in `target_decks` for the reason the sixth, seventh and eighth
are, so every committed 2..8-seat number is unchanged; `--commander --seats 9`
is what reaches it, and `bot_ladder`'s seat clamp is now the length of
`target_decks()` rather than a literal 8.

Its 99 is built around *connecting* rather than around size: fourteen Ninjas,
twelve one- and two-mana creatures that cannot be blocked or fly, and Rogue's
Passage for the board that stalls. Yuriko's own trigger is the pod half —
"whenever a Ninja you control deals combat damage to a player … **each
opponent** loses life equal to that card's mana value" — so one connection
drains the whole table and its rate scales with the seat count where every
other seat's damage does not.

⚠⚠ **And the number says that prediction was right, which makes it a finding
about the clause rather than a deck to tune.** Yuriko wins **30.0 %** of
nine-seat pods (2,000 games, seed 9113; field 11.7 / 3.2 / 4.5 / 6.0 / 1.0 /
37.1 / 5.8 / 0.7 / **30.0**), against a seat's share of 11.1 % — second only
to Edgar Markov, and for the same structural reason the Edgar row records:
**a payoff that reads the whole table grows with the table.** Edgar's is a
free body per Vampire spell from the command zone; Yuriko's is one unblocked
Ninja draining eight opponents at once. Neither is a card that is strong in
a duel. The two sit at opposite ends of the field's spread precisely because
every other seat's clock is per-opponent.

The **eleventh** is the first **official precon**, Sultai Arisen (Tarkir:
Dragonstorm Commander, 2025), taken card for card from MTGJSON's
`SultaiArisen_TDC` deck file — the offline Scryfall cache can't say how a
precon splits into decks, MTGJSON can. A scan of every MTGJSON Commander deck
against the catalog ranked it first: 99 of 100 already implemented, the
hundredth (Lord of the Forsaken) needing one primitive,
`SpendRestriction::SpellFromGraveyard` + `SpellKind::from_graveyard` (CR
106.6 / 601.2a). It is the pod's graveyard seat: Kotis's once-a-turn
graveyard cast, Steward of the Harvest's granted land abilities, Teval's
"cards leave your graveyard" tokens. After `pod_field(10)`; `--seats 11`
reaches it. The next two by `scripts/precon_scan.py` became seats 15 and
16, and Tramplesaurus Rex (FDC, 12) seat 17 (below); after them the scan
read Keen Engineering (FDC, 12, seat 18) and then 13 (Reap the Tides,
Corrupting Influence, Sliver Swarm).

🟡 **Residuals in the list** (each also on its card's doc): Colossal
Grave-Reaver (returns the first milled creature, not a chosen one), Lethal
Scheme (convokers don't connive), Cephalid Coliseum (the sacrifice is folded into resolution).
Steward of the Harvest and Life from the Loam pick at resolution rather than
target.

The **twelfth** is the second official precon, **Mind Flayarrrs** (Commander
Legends: Battle for Baldur's Gate, 2022), MTGJSON's `MindFlayarrrs_CLB` card
for card; `--seats 12` reaches it. Its seven missing cards took four
primitives, each built generally: `AdditionalCastCost::OneOf` (Dusk Mangler,
CR 601.2b), `TriggerZone::WhileSuspended` (Nihilith, CR 702.62b),
`Effect::ExileIfLeavesBattlefield` (From the Catacombs — and four cards that
approximated the same clause: Geth, Whip of Erebos, Gruesome Encore,
Llanowar Greenwidow) and `DynamicPt::ChosenPlayerGraveyardMatching`'s
`scales_toughness` (Sewer Nemesis). Haunted One also exposed
`SharesCreatureTypeWithSource` answering false for any living source.
First reading (12 seats, 1,000 games, seed 9920): all decided, every card of
all twelve lists played (566 distinct), N'ghathrod **10.3 %** against a
seat's 8.3 %, Teval 4.5 %.

🟡 **Residuals in the list**: Grell Philosopher (no blue-as-any rider). Sewer
Nemesis always chooses an opponent.

The **thirteenth** is the third official precon, **Blood Rites** (Lost Caverns
of Ixalan Commander, 2023), MTGJSON's `BloodRites_LCC` card for card;
`--seats 13` reaches it. Nine missing cards, one new primitive
(`Effect::ExileLinkedTo`, Timothar's Bat) — and one bug class found on the
way: **persist and undying read only printed keywords**, so every grant
(Undying Evil, Mikaeus, Haunted One, Dusk Legion Sergeant) was cosmetic.
Voldaren Estate got its Vampire-only mana and per-Vampire discount back.
First reading (13 seats, 1,000 games, seed 9930): all decided, every card of
all thirteen lists played (610 distinct), Clavileño **27.0 %** against a
seat's 7.7 % — the Edgar finding again: a Vampire payoff that drains the
whole table grows with the table.

🟡 **Residuals in the list**: New Blood (the text change is approximated).

The **fourteenth** is the Secret Lair **Heads I Win, Tails You Lose** list
(SLD, 2022), MTGJSON's `HeadsIWinTailsYouLose_SLD` card for card — coin
flips, and the pod's "Partner with" pair (CR 702.124j). Its commanders were
missing too (the scanner checked only the 99; fixed). Daretti's -10 emblem
exposed the emblem matcher's hand-kept ten-kind whitelist.

⚠⚠ **And fourteen seats is where the flat 50,000-action cap starts to bite:
1 game in 1,000, at two seeds.** Both capped games are *long*, not looping —
309 and 314 turns against a 170 mean, 9-10 of 14 players still alive, empty
hands, boards of lands after the sweepers, priority passes spread evenly
across seats; libraries of 20-60 cards would have ended them by decking.
The new `longest` column says why it starts here: the longest game runs
1.7-2.8x the mean at every seat count (42,266 actions at 13 seats), and the
14-seat mean is 24,196. Recorded, not "fixed" by raising the cap — a
stalled attrition board at fourteen players is the finding.

The **fifteenth** is the Secret Lair **Goblin Storm** list (MTGJSON's
`GoblinStorm_SLD`): rituals, Zada copying a one-target spell onto every
creature, and General Kreat's Goblin-per-attacking-Goblin. Residual: Throne
of Eldraine's draw ability doesn't enforce "spend only mana of the chosen
color". ⚠⚠ **Its first 15-seat run found a CR 508.4 bug in the engine, not
the deck:** every "put onto the battlefield attacking" site (token-attacking,
Myriad, Mobilize, Ninjutsu, two put-a-card-in-attacking effects) emitted
`AttackerDeclared`, so Kreat's attack trigger fired for its own tokens —
922 Goblins on one seat by turn 57, eight board-capped games in 1,000 (seed
9960). Fixed at every site at once (`game/enter_attacking.rs`); the same
seed now reads **991 / 1,000 decided, board cap 0, action cap 9** (0.9 %,
all 272-323-turn attrition games, as at fourteen seats); Zada wins 4.2 %.

The **sixteenth** is Foundations Commander's **Wretched Ranks** (MTGJSON's
`WretchedRanks_FDC`, 2026-10-02 — the list is published ahead of release):
mono-black Zombies, 33 Swamps. Nine cards were missing; one needed a
primitive, Razorlash Transmogrant's "costs {4} less if an opponent controls
four or more nonbasic lands" — `Predicate::AnOpponentControlsAtLeast`,
asked of each opponent on their own (CR 102.2), and
`ActivatedAbility::cost_reduction_if`. Seed 9970, 1,000 games at 16 seats:
**976 decided, 24 action caps (2.4 %), 0 board caps, zero panics**; Gisa
wins 10.5 %; `--card-census` 784 distinct, Gisa's 100 all played. ⚠ **At
sixteen seats the flat cap lands inside an ordinary game:** a capped game at
turn 249 is only ~15 turns per seat, libraries still 54-74 cards, boards
live — not the lands-only attrition of fourteen seats. The 16-seat mean is
31,037 actions. Left at 50,000 by the cap's own contract (it is what the
stall rate is read against); a budget that scales with seats is the lever if
the field grows further.

The **seventeenth** is Foundations Commander's **Tramplesaurus Rex**
(`TramplesaurusRex_FDC`): mono-green stompy under Ghalta. Twelve cards were
missing and three needed primitives, each built for its class —
`SelectionRequirement::IsAttackingYou` (CR 506.3: Arachnogenesis counts the
creatures attacking *you*, not the table's attackers),
`Effect::CreateTokensToFightEach` (CR 701.14: Ezuri's Predation) and
`Value::GreatestCommanderManaValue` (CR 903.3: Tangleweave Armor). Residual:
Monstrous Onslaught reads X at resolution, not as it is cast. Seed 9980,
1,000 games at 17 seats: **1,000 decided, zero panics**; Ghalta wins 8.2 %;
`--card-census` 841 distinct, every list fully played.

⚠⚠ **The seventeenth seat retired the flat action cap.** Capped games went
0.4 / 2.4 / 6.8 % at 15 / 16 / 17 seats and every one read was an ordinary
game cut short, not a stall. `bot_ladder`'s pod budget is now 50,000 up to
ten seats and 5,000 a seat above (readings at 2..10 unchanged): seed 9941
decides 1,000 / 1,000 at 11, 14, 15, 16 and 17 seats, the longest 17-seat
game 71,689 of 85,000.

The **eighteenth** is Foundations Commander's **Keen Engineering**
(`KeenEngineering_FDC`): mono-blue artifacts under Sai, 34 Islands. Twelve
cards were missing; four needed engine work, each general —
`Effect::ReselectAttackTarget` (Misleading Signpost, CR 508.1b: one
`ChooseOption` ask, a headless seat sends the attacker at its ranked hostile
opponent and never at itself), `SelectionRequirement::ControllerDamagedBySourceThisTurn`
(Steel Hellkite), `ExtraManaKind::MirrorColorless` reaching nonland
permanents (Forsaken Monument's "tap a permanent for {C}"), and
`StaticEffect::PreventUntapGlobal` honouring an Aura's `AttachedTo(This)`
(Fall from Favor — it fell to `_ => false`, so the lock never held).
Residual: **Steel Hellkite** reads `creatures_that_damaged_me_this_turn`, which
also holds noncombat damage. Seed 9990, 1,000 games at 18 seats: **998
decided, 1 action cap, 1 board cap, zero panics**; Sai wins 3.1 %;
`--card-census` 892 distinct, every card of all eighteen lists played.

The **nineteenth** is Commander Legends' **Reap the Tides**
(`ReapTheTides_CMR`): Simic lands-matter under Aesi. Twelve cards were
missing (`precon_scan` read thirteen: its factory regex missed a
`-> crate::card::CardDefinition` signature, so wwk2's Terastodon looked
absent — fixed). No new primitive: emerge, retrace, Fact or Fiction's pile
split, kicker and the Seedborn-style filtered untap all existed. Residual:
**Stumpsquall Hydra** puts all X counters on itself and then moves any onto
commanders *you control* (the headless seat spreads them); an opponent's
commander, which "any number of commanders" allows, is never offered. Seed
9999, 1,000 games at 19 seats: **997 decided, 2 action caps (419-457-turn
games at the 95,000 budget), 1 board cap, zero panics**; Aesi wins 5.1 %;
`--card-census` 940 distinct, every card of all nineteen lists played. The
board cap is legitimate: a Krenko seat doubling to 1,214 Goblins behind the
Sai seat's Propaganda, whose tax lets six Mountains send three a turn.

The **twentieth** is Phyrexia: All Will Be One's **Corrupting Influence**
(`CorruptingInfluence_ONC`): Abzan poison under Ixhel — the field's first
deck that wins by CR 704.5c. Thirteen cards were missing and they took five
primitives, each general: `Value::PoisonCountersAmong` / `PlayersWithPoisonAtLeast`
(`PoisonCountersOf` reads one seat, so "your opponents' poison" and "each
corrupted opponent" were wrong at a table), `EventSpec::once_per_batch_across_players`
(CR 603.2c "to one or more players" — Contaminant Grafter), `Predicate::AnAttackedPlayerHasPoisonAtLeast`
(Norn's Decree) and `EquipBonus::sacrifice_host_when_unattached` (`game/unattach.rs`,
queued at all seven unattach sites) — which also restored **Grafted Wargear**'s
dropped rider. Residuals: **Geth's Summons** and **Glissa's Retriever** pick
their cards at resolution instead of targeting (the Summons reads poison then,
not as it is cast), and **Ixhel** exiles face up (hidden information only).
Seed 10020, 1,000 games at 20 seats: **999 decided, 1 action cap (427 turns at
the 100,000 budget), 0 board caps, zero panics**; Ixhel wins 5.0 %;
`--card-census` 999 distinct, every card of all twenty lists played. In
four-seat pods (`--pod-decks`, seed 10021, 1,000 each, all decided) Ixhel
wins 16.8 % beside the FDC trio — where **Ghalta takes 60.7 %**, a deck-strength
reading worth a look — and 13.9 % beside Sigarda / Edgar / Clavileño.

The **twenty-first** is Zendikar Rising's **Sneak Attack** (`SneakAttack_ZNC`):
Dimir Rogues under Anowon. Fifteen cards were missing; two primitives:
`EventSpec::once_per_batch_summing_damage` (CR 603.2c — a batched combat
trigger read the FIRST dealer's damage; Anowon mills the batch's total) and
`Value::GreatestGraveyardSizeAmong` ("an opponent has eight or more cards in
their graveyard" read only the first opponent — Jace's Phantasm fixed with
it). Residual: **Whispersteel Dagger** opens every creature card in the
graveyard, not one (Master Thief's "for as long as you control" is exact since
`Effect::GainControlWhileYouControlSource`). Seed 10031, 1,000 games at 21 seats: **996 decided, 4
action caps, 0 board caps, zero panics**; Anowon wins 7.9 %, and 32.8 % of
four-seat pods beside Teval / N'ghathrod / Ixhel (seed 10032, all decided).
⚠ **The census found a card castable by no path: Spinal Embrace** ("cast
this spell only during combat" — the default profile's combat window casts
pump tricks only, and the main-phase cast is refused). `server/combat_only.rs`
gives such a spell the post-block window; the same seed then played **every
card of all twenty-one lists** (1,043 distinct, 997 decided).

The **twenty-third** is Commander Masters' **Sliver Swarm** (`SliverSwarm_CMM`),
the field's first five-color list. Thirteen cards were missing; the
primitives, each general: `StaticEffect::LegendRuleDoesntApplyToYourMatching`
(CR 704.5j, Gravemother's Slivers), `GraveyardCardsHaveEncore` (CR 702.141,
encore {X} = mana value, beside Varolz's scavenge grant),
`YourSpellsHaveReplicate` (CR 702.107 — Hatchery; the bot now offers a
*granted* replicate, Djinn Illuminatus's included) and
`SelectionRequirement::DamagedAPlayerThisTurn`. Two latent bugs surfaced:
`SharesCreatureTypeWithSource` read the printed line on both sides, so a
chosen or granted type never counted (Titan of Littjara), and Galerider Sliver
gave every player's Slivers flying (Spiteful and Lavabelly could not aim at a
planeswalker either). Residual: **Descendants' Fury**'s "one of them" is any
attacker of yours that has damaged a player this turn, not only this batch's.
⚠ It also found an engine-wide rules bug: attackers were removed from combat
as regular damage was dealt, so every post-damage and "at end of combat" read
of the attackers saw none — fixed (CR 511.3, `combat.rs::remove_all_from_combat`,
run as the end of combat step ends).
Seed 10041, 300 games at 23 seats: **298 decided, 2 action caps, zero panics,
every card of all twenty-three lists but one played** (Narset's Reversal);
Gravemother wins 5.7 %, and **40.4 %** of four-seat pods beside Shiko /
Anowon / Ixhel (seed 10043, 1,000 games, all decided).

The **twenty-sixth** is Foundations Commander's **Calling All Angels**
(`CallingAllAngels_FDC`), the field's first mono-white identity: Giada's
growing Angels, lieutenants, the monarch. Seventeen cards were missing; one
primitive, `Predicate::ControlsLandsWithSameNameAtLeast` (Endless Atlas — the
largest same-name group of YOUR lands; `SharesNameWithAnotherPermanent` read
every player's and summed across names). Firemane Commando's other-player half
is an `AnyPlayer` observer — writing it found **Tomik, Wielder of Law**'s
attack trigger dead since it shipped (`OpponentControl` is a scope the attack
dispatch never consults for a non-active listener). No residuals.
Seed 10061, 1,000 games at 26 seats: **944 decided, 56 action caps, 0 board
caps, zero panics, every card of all twenty-six lists played** (1,303
distinct); Giada wins **11.5 %** there and **68.6 %** of four-seat pods beside
Hanna / Sigarda / Clavileño (seed 10062, all decided) — a strength reading
like Edgar's Eminence, not a bug: the cost reducers stack on a commander that
makes every Angel bigger. The action caps are long stalls, not loops: the one
read (game 3) had Clavileño at 4,729 life off Exquisite Blood — each of
twenty-five opponents' life loss is a gain — and twenty seats still alive.

The **twenty-fifth** is Commander 2017's **Vampiric Bloodlust**
(`VampiricBloodlust_C17`) — Edgar Markov's own precon, so the field now seats
two Edgars. Sixteen cards were missing; the primitives:
`Effect::SpellEntersWithCounters` (Bloodlord's granted bloodthirst),
`Predicate::PlayerControlsACommander` (CR 903.3, Crimson Honor Guard) — and
`YouControlACommander` now counts **any** player's commander, as the CR's "a
commander" does (a stolen one never satisfied Akroma's / Jeska's Will),
`Selector::PowerAbove` (Fell the Mighty: threshold read once, every match picked
before any dies), `StaticEffect::SelfCostReducedByValue` (Licia) and
`CreateTokenCopiesHasteSac.exile` (Kindred Charge). `Effect::ExileFromHand`
used to take `hand[0]` from everyone; a bot seat now chooses (Kheru Mind-Eater,
Ashiok). Residuals: **Bloodlord of Vaasgoth**'s granted bloodthirst is checked
as its cast trigger resolves; **Mathas, Fiend Seeker**'s bounty grant lasts only
while Mathas stays; **Kheru Mind-Eater** exiles face up. Seed 10051, 300 games
at 25 seats: **289 decided, 10 action caps, 1 board cap, zero panics**; the
C17 Edgar wins 12.3 % and every one of its cards was played. ⚠ The board cap
is **not a loop**: Gravemother's encore at 24 opponents made 21 Brood Sliver
tokens, and 21 Broods × ~40 connecting Slivers minted 813 tokens in one combat.
Four-seat pods beside hand-built Edgar / Gravemother / Clavileño (seed 10052,
1,000 games, all decided): C17 Edgar 19.6 %, hand-built Edgar **41.3 %**.

The **twenty-eighth** is Bloomburrow Commander's **Animated Army**
(`AnimatedArmy_BLC`), the field's first Gruul identity: Bello animates the
big artifacts and enchantments on its turn. Seventeen cards were missing; six
primitives — Treasure mana provenance (`ManaPool`'s counter +
`Predicate::CastWithTreasureMana`, Alchemist's Talent / Rain of Riches),
`Effect::RevealTopMayCastOneFree` (Sunbird's Invocation), `Effect::WithX` +
`SelectionRequirement::PutOntoBattlefieldBySource` (Kodama of the East Tree's
equal-or-lesser put and its no-chain clause) and
`Effect::EachPlayerDrawsDamageTheyDealtToSource` (Grothama). ⚠ **Bello found a
layer bug**: the continuous-effect matcher had no mana-value leaf, so
`Not(ManaValueAtMost(3))` was true of everything and Mind Stone became a 4/4.
Residuals: **Evercoat Ursine** can't play a hidden land; **Grothama**'s fight
offer is its own trigger asking the attacker's controller, not a granted
ability; **Tendershoot Dryad**'s ascend is checked on entry and at each upkeep.
Seed 10071, 1,000 games at 28 seats: **860 decided, 140 action caps, 0 board
caps, zero panics, every card of all twenty-eight lists played** (1,400
distinct). The caps are the per-seat budget binding, not loops: passes per
turn grow with the table, so 5,000 actions a seat is about nineteen rounds at
28 seats, and the game read (17) had thirteen players alive on stalled boards
of 20-31 permanents at turn 536. Bello wins 0.9 % there and 5.8 % of
four-seat pods beside Ghalta / Freyalise / Tatyova (seed 10072, all decided —
Ghalta takes 80.8 %).

The **twenty-seventh** is Commander 2014's **Guided by Nature**
(`GuidedByNature_C14`) — mono-green Elves under Freyalise, the field's second
planeswalker commander (the hand-built Freyalise seat is its model). Sixteen
cards were missing; the primitives: `Keyword::CantBeSacrificed` (CR 701.16 —
Assault Suit; honored by effect sacrifices, `sacrifice_one` and both
activation-cost sacrifice walkers), `EachPlayerChoosesCreatureTypeThen.per_player`
(Grave Sifter — each player acts on their own pick; departed seats no longer
choose, and a headless seat names its own commonest type rather than Demon,
which also fixes Harsh Mercy / Patriarch's Bidding in bot play),
`Effect::ChooseOpponentThen` (Sylvan Offering), `Value::OpponentsWithHandSizeAtLeast`,
`Value::SacrificedThisResolutionBy` (Wave of Vitriol) and
`ActivatedAbility.mana_cost_increase` (Loreseeker's Stone). Residuals:
**Sylvan Offering**'s opponent is the engine's pick (fewest creatures); **Siege
Behemoth** always assigns as though unblocked. Seed 10081, 300 games at 27
seats: **278 decided, 22 action caps, zero panics**, every C14 card played;
the caps are attrition (game 297: 18 of 27 seats alive at turn 351) — the cap
rate climbs with seats (0.6 % at 15, 3.3 % at 25, 7.3 % at 27) and was left
alone. Four-seat pods beside Freyalise / Ghalta / Aesi (seed 10082, 1,000
games, all decided): C14 Freyalise 11.8 %, Ghalta **67.4 %**.

The **twenty-ninth** is Secret Lair's **Grave Danger** (`GraveDanger_SCD`) —
Dimir Zombies under Gisa and Geralf. Sixteen cards were missing; the
primitives: `StaticEffect::GraveyardCastOncePerTurn` (the no-sacrifice sibling
of Exploration Broodship's grant), `AlternativeCost.from_graveyard` with
`cast_alternative_from_graveyard` (Scourge of Nel Toth) and
`Predicate::UsedGraveyardThisTurn` (Laboratory Drudge). ⚠ **The find: no bot
block ever cast a graveyard card through a board permission** — Muldrotha,
Lurrus, Exploration Broodship and now Gisa and Geralf were dead in bot play;
`spec::GY_GRANT` offers those casts and the graveyard-only alternative costs.
Residuals: **Havengul Lich** doesn't gain the cast card's activated abilities;
**Liliana, Untouched by Death**'s −3 covers the graveyard as it resolves.
Seed 10091, 300 games at 29 seats: **262 decided, 38 action caps, zero
panics**, every Grave Danger card played; four-seat pods beside N'ghathrod /
Gisa / Anowon (seed 10092, 1,000 games, all decided): Gisa and Geralf 35.6 %.
⚠ The action-cap rate keeps climbing with seats (7.3 % at 27, 12.7 % at 29):
the budget grows by 5,000 a seat above ten while actions/game grow faster
(27.5 k at 15, 78 k at 25, 106 k at 29) — attrition, not loops (every diagnosed
cap). 2..8 seats at seed 10093: all decided.

The **thirtieth** is Commander 2014's **Forged in Stone** (`ForgedInStone_C14`)
— mono-white Equipment and Kor under Nahiri, the Lithomancer, the field's
third planeswalker commander. Fifteen cards were missing (Karoo was already
`karoo_land`); the primitives were `EquipScale` reading its equipment as the
source and `count_named_like_exiled_with_source` (Strata Scythe). Residuals:
**Arcane Lighthouse** strips hexproof/shroud once rather than "can't have",
**Benevolent Offering**'s opponent is the engine's pick, **Nahiri**'s +2/−2
take your first Equipment. Seed 10101, 200 games at 30 seats: **168 decided,
32 action caps, zero panics** (the cap rate: 16 % at 30 seats, attrition as
before). ⚠ **The find: games 13 and 15 took 568 s each** against ~1 s for a
normal game — a board of ~200 permanents, nothing runaway. The profile put
every sample in `gather_continuous_effects_inner` under `computed_permanent`:
an **unfrozen** computed read gathered afresh on every call, bypassing the
`(-303)` cross memo, so each SBA walk paid a full gather per permanent. Routed
through the memo, game 13 runs 552 s -> 177 s with the same 110,161 actions,
and the bench stays byte-identical. The memo's debug audit then caught two
statics gated on turn scalars the key did not witness (Thrasta's entry turn,
Medomai's extra turn); `turn_number` / `step` / `current_turn_is_extra` now
fold into the key.

The **thirty-first** is Modern Horizons 3 Commander's **Graveyard Overdrive**
(`GraveyardOverdrive_M3C`), the field's first Jund identity: Lhurgoyfs under
Disa the Restless. Seventeen cards were missing; the primitives:
`MayPlayDuration::UntilYourNextEndStep` (which also moved **eight impulse cards**
that had been playable for one turn off to their printed window),
`StaticEffect::DoubleDamageToChosenPlayer` (Sawhorn Nemesis),
`Effect::CopySpellOntoAnotherOpponentsPermanent` (Exterminator Magmarch) and
`StaticEffect::MayPlayCardsMilledThisTurn` (Coram, the Undertaker, with its own
bot candidates). ⚠ **Two engine finds**: a spell copy was controlled by the
original caster (CR 707.10c — Narset's Reversal handed the copy back), and "deals
damage equal to its power to any target" never aimed at a player (Pyrogoyf hit
its own creature). Residuals: **Disa**'s "not from the battlefield" reads "not
put there from the battlefield this turn"; **Find // Finality** picks its two
cards at resolution; **Ziatora** sacrifices the weakest other creature. Seed
10081, 600 games at 31 seats: **510 decided, 90 action caps, 0 board caps, zero
panics**, 1,535 distinct cards played. ⚠ The census found **Tempt with Mayhem
castable by no bot path** — no picker copied the bot's own spell — and
`pick_copy_response` now does. Disa wins 5.7 % there and **47.2 %** of four-seat
pods beside Teval / Ixhel / Bello (seed 10082, 1,000 games, all decided).
`--bench` byte-identical.

The **thirty-fifth** is Commander 2014's **Sworn to Darkness**
(`SwornToDarkness_C14`) — mono-black Demons and morbid under Ob Nixilis of the
Black Oath, a planeswalker commander. Fifteen cards were
missing; the primitives: `Keyword::MustAttackChosenPlayer` +
`PlayerRef::RandomOpponent` (Raving Dead, CR 508.1d), emblem statics that grant
activated abilities (CR 114.4 — Ob's −8 had granted nothing), and
`Effect::DrainLifeLost`. ⚠ **Ob found a Commander bug class**: `Drain` gains its
amount once, so "you gain life equal to the life lost this way" gave one
opponent's worth at a table — Gray Merchant, Kokusho, Exsanguinate and six more.
A sweep of all 187 each-opponent drains against oracle text moved exactly those
nine (the seeded pod table re-blessed: Judith runs two of them). ⚠ **Malicious
Affliction found a second**: a self-copied Destroy defaulted to the original's
target and did nothing (CR 608.2b); it now picks another opposing permanent.
Residuals: **Infernal Offering**'s opponents are the engine's pick and each
return takes the first creature card in graveyard order; **Profane Command**'s
two modes are resolution-time picks (default: life loss and −X/−X).
Seed 10101, 300 games in the 32-seat field before the rebase added Ezuri,
Gisela and Daretti (Ob was seat 32): **229 decided, 71 action caps, 0 board caps,
zero panics**, 1,570 distinct cards; the census found **Wake the Dead castable
by no bot path** (combat on an opponent's turn is no off-turn window) and
`pick_combat_only_instant` now casts it; Tempt with Mayhem played once the copy
picker landed. Ob wins 5.0 % there and 26.5 % of four-seat pods beside Judith /
Gisa / Yuriko (seed 10102, 1,000 games, all decided). `--bench` byte-identical.

The **thirty-second**, **thirty-third**, **thirty-fourth**, **thirty-sixth**
and **thirty-seventh** are Commander 2015's **Swell the Host** (Ezuri, GU
+1/+1 counters), the Secret Lair **Angels: They're Just Like Us** (Gisela, the
**first meld commander**), Commander 2014's **Built From Scratch** (Daretti,
Scrap Savant, mono-red artifacts under a planeswalker), Crimson Vow
Commander's **Vampiric Bloodline** (Strefan, Rakdos Blood tokens, with
Kamber and Laurine as a Partner-with pair in the 99) and Commander 2015's
**Plunder the Graves** (Meren, Golgari sacrifice and recursion). Engine finds
along the way: ⚠ **a melded permanent is its component commander** (CR 712.4 +
903.3 — Brisela's damage now tallies as Gisela's commander damage and it can
go home); ⚠ **evoke never sacrificed** (CR 702.74a); ⚠ **graft's move trigger
had no intervening-if**; ⚠ **Myriad copied toward a seat that had left**
(CR 702.116a); ⚠ **a parked `WithX` / `AsPlayer` body lost its X or its
player on resume** (CR 608.2a); ⚠ **the auto-targeter aimed a divided spell's
later slots at the same player, or at the caster** (Forked Bolt and Avacyn's
Judgment were uncastable by the bot); ⚠ **the equip sink gate missed granted
Equipment** (Arterial Alchemy's Blood tokens tripped its debug assert); and
seven shipped cards were corrected against oracle (Sakura-Tribe Elder,
Evolving Wilds, Mosswort Bridge, Arbor Colossus, Trygon Predator, Forgotten
Ancient, Caller of the Claw). Residuals are in INCOMPLETE_CARDS.

⚠ **The pod budget counts plays, not priority passes.** Priority passes were
~90 % of actions and grow with seats² (1,921 / 7,098 / 29,654 / 73,911
actions per game at 4 / 8 / 16 / 24 seats, against 190 / 397 / 891 / 1,505
plays), so the old action cap stopped 28–32 % of 33–34-seat games that were
still progressing. `bot_ladder` now budgets `max(seats, 4) × 1,000` plays with a
200-passes-per-play backstop. Readings on this budget: seed 9312, **60 games
at all 37 seats: 60 decided, 0 caps, zero panics** (467 turns / 2,298 plays a
game; Gisela 20.0 %); seed 9311, 400 four-seat games of the first four decks,
all decided; seed 9323, 400 eight-seat games, all decided (Edgar 48.5 %).
Four-seat pods of the new seats: seed 9321 (Ezuri / Gisela / Daretti /
Strefan, 1,000 games, all decided) — Gisela **68.4 %**, Strefan 16.8 %, Ezuri
13.0 %, Daretti 1.8 %; seed 9322 (Meren / Ob / Disa / Gisa and Geralf, 1,000
games, 997 decided, 3 rule draws) — Meren 8.9 %. `--bench` byte-identical.

The **thirty-eighth** is Commander 2015's **Seize Control** (`SeizeControl_C15`)
— Izzet spells under Mizzix of the Izmagnus (seat 38; the seat commit's message
says "thirty-fifth", from before two rebases). Fifteen cards were missing; the
primitives: `Effect::GainControlOfSpell` (Aethersnatch — the spell resolves
under its new controller, CR 608.3) and `StaticEffect::ChosenColorsSpellCostReduction`
(Seal of the Guildpact). ⚠ **The census found three cards castable by no bot
path**: "each of X targets" (Meteor Blast, and Doppelgang before it) never built
a legal cast because the slot walker fills every slot — `exactly_x_targets`
picks distinct targets and sets X to their count; and the response picker's
`effect_counters_spells` read neither a `ChooseN`'s default picks (Mystic
Confluence) nor a spell steal (Aethersnatch). Residual: **Mystic Confluence**
runs its default picks (counter unless {3}, draw two). Four-seat Izzet pods
beside Zellix / Zndrsplt / Stella (seed 10111, 1,000 games, all decided, every
Mizzix card played): Mizzix 16.2 %. Seed 10121, 300 games at 38 seats: **300
decided, 0 caps, zero panics** (190.6 k actions/game, 1,038 s on 4 threads).
`--bench` byte-identical.

The **fortieth** is Commander 2015's **Call the Spirits** (`CallTheSpirits_C15`)
— Orzhov enchantments under Daxos the Returned. Fifteen cards were missing; the
primitives: `Keyword::CantAttackAuraController` (Vow of Duty / Vow of Malice,
CR 508.1a — the enchanted creature may still attack anyone but the Aura's
controller), `Value::CreaturesDestroyedThisResolutionControlledBy` (Deadly
Tempest charges each player for the creatures they controlled as they were
destroyed, tokens included) and `Value::PlayersWithGreaterTally` (Oreskos
Explorer). Residual: **Sandstone Oracle**'s
opponent is the one with the most cards in hand. Debug pods beside Adrix and Nev /
Meren / Gisela / Judith (seeds 9251/9252, 60 games) decided 60/60 with zero
panics, and a 120-game census (seed 9253) leaves no card unplayed.

The **forty-first** is Phyrexia: All Will Be One Commander's **Rebellion Rising**
(`RebellionRising_ONC`) — Boros tokens and Equipment under Neyali, Suns'
Vanguard. Seventeen cards were missing, the commander among them; the
primitives: `MayPlayDuration::TurnsHolderAttacksWithAToken` (Neyali's "during
any turn you attacked with a token, you may play that card" — the turn sweep
parks the grant and a declared token attacker re-arms it, CR 508.1),
`SelectionRequirement::IsAttackingAnOpponent` (Roar of Resistance),
`SelectionRequirement::Unattached` and `CounterType::Story`. ⚠ **The census
found Clever Concealment castable by no bot path**: nothing answered a
sweeper with a protective instant. `pick_sweeper_shield` resolves the
opponent's top spell in a clone and, when it would take two or more of our
nonland permanents, casts a phase-out / indestructible instant aimed at every
own-side slot. Residuals: **Collective Effort**'s escalate is paid at
resolution, **Goldwardens' Gambit** hands each token your best unattached
Equipment (no pick), **Neyali** counts a token attacking a planeswalker as
attacking a player. Four-seat pods beside Adriana / Giada / Gisela (seed
10131, 1,000 games, 999 decided + 1 draw, every card played): Neyali 13.0 %.
Seed 10141, 300 games at 43 seats: **299 decided + 1 draw, 0 caps, zero
panics** (230.8 k actions/game, 427 s on 4 threads). `--bench` byte-identical.

The **fifty-second** is Commander 2017's **Feline Ferocity** (`FelineFerocity_C17`)
— Selesnya Cats and Equipment under Arahbo, Roar of the World. Seventeen cards
were missing; the primitives: `StaticEffect::OpponentMultiDrawBecomesOneEach`
(Alms Collector, CR 614.1a — read by `Effect::Draw` on a batch of two or
more), `Effect::PlayerGainsProtectionFromChosenColor` with
`Player.protection_colors_eot` (Seht's Tiger, CR 702.16b/e — no targeting, no
damage; the cast path checks the out-of-zone card's colors), Mirri's
`Effect::OpponentsBlockWithAtMost` + `StaticEffect::AttackerCapAgainstControllerWhileTapped`
(the bot's block planner trims to the cap), `EachPlayerKeepsOneSacrificeRest.destroy`
(Divine Reckoning) and `IsSourceChosenCreatureType` answering for library cards
while a `ChooseCreatureTypeThen` resolves (Kindred Summons). Residuals:
**Divine Reckoning** keeps each player's highest-mana-value creature;
**Stalking Leonin**'s opponent is picked openly. Four-seat pods beside Sigarda /
Ghalta / Giada (seed 10151, 1,000 games, all decided, every card played):
Arahbo 6.3 %. Seed 10161, 300 games at 52 seats: **299 decided, 1 board cap,
zero panics** (329 k actions/game, 1,452 s on 4 threads). `--bench`
byte-identical.

The **fifty-sixth** is Commander 2019's **Primal Genesis** (`PrimalGenesis_C19`)
— Naya tokens and populate under Ghired, Conclave Exile. Sixteen cards were
missing, the commander among them; the primitives:
`StaticEffect::OpponentsCantCastDuringCombat` (Marisi, CR 506.1 — Basandra's
lane scoped to opponents), `CreatureType::Sculpture` (Doomed Artisan) and
`Effect::PlayersWithMostSacrifice` (Tectonic Hellion — the tied-for-most set
is fixed before anyone sacrifices). Ghired's attack populates and the copy
joins the attack (`Populate` + `JoinCombatAttacking` on the last created
token). Residuals: **Cliffside Rescuer**'s protection is from what opponents
control; **Tahngarth** attacks its borrower's default opponent. Four-seat pods
beside Sigarda / Ghalta / Bello (seed 10171, 1,000 games, all decided, every
card played): Ghired 27.6 %. `--bench` byte-identical.

The **sixty-first** is Commander 2016's **Breed Lethality** (`BreedLethality_C16`)
— four-color +1/+1 counters and proliferate under Atraxa, Praetors' Voice.
Sixteen cards were missing; Champion of Lambholt and Dreadship Reef landed
meanwhile with Token Triumph and Devour for Power, so fourteen are this
seat's. The primitive: `Effect::RevealTopEachOpponentChoosesToHand`
(Manifold Insights — each opponent in turn order picks a different card for
you, the rest bottom at random). Shared helpers `sets::storage_land` and
`sets::pain_tap`. ⚠ **The census found an optional loop the bot never
stopped**: two Enduring Scalelords feed each other a counter per "you may"
forever (2 of 1,000 games capped). `HeuristicBot` now declines past 64
optional yeses in one step (CR 732.2), and the rerun decided 1,000/1,000 with
every card played: Atraxa 27.9 % beside Teval / Ixhel / Bello (seed 10191).
Residual: **Duneblast** always keeps one creature. The 56-seat smoke of the
field before this seat (seed 10181, 200 games): **197 decided, 3 board caps,
zero panics**. `--bench` byte-identical.

The **forty-second** is Commander 2014's **Peer Through Time**
(`PeerThroughTime_C14`) — mono-blue control under Teferi, Temporal Archmage,
the pod's fifth planeswalker commander, with Stormsurge Kraken as its
lieutenant. Seventeen cards were missing; the primitives:
`StaticEffect::CreaturesEnterAsCopyOf` (Infinite Reflection, CR 707.2 — the
static sibling of `enters_as_copy`), `StaticEffect::LoyaltyAbilitiesAtInstantSpeed`
(Teferi's emblem, CR 606.3; the bot already offers loyalty lines in its
opponent's-end-step window, so the emblem is played), `Effect::MustAttackPlayerThisTurn`
(Dulcet Sirens, CR 508.1d), `SelectionRequirement::SourceOwnerPlayer` (Crown of
Doom's "other than its owner") with `GainControl` now reading a recipient
selector's filter, and `Selector::TakeGreatestPower` (Stitcher Geralf).
Residuals: **Domineering Will**'s "target player" is always you;
**Intellectual Offering**'s opponents are the engine's pick; **Shaper
Parasite**'s ±2 is picked as the trigger goes on the stack; **Infinite
Reflection**'s ETB also "copies" the enchanted creature onto itself when it is
yours.

The **thirty-ninth** is Commander 2021's **Quantum Quandrix**
(`QuantumQuandrix_C21`) — Simic Fractals and token doubling under Adrix and
Nev. Sixteen cards were missing; five primitives:
`StaticEffect::FirstTokensOnYourTurnBecomeCopiesOfChosen` (Esix, beside Moonlit
Meditation's replacement), `Effect::StealOpponentTokensThisTurn` (Crafty
Cutpurse, a per-player flag the token mint funnel reads),
`Effect::WheneverCreatureEntersThisTurn` (Theoretical Duplication — any
controller's creature), `SpendRestriction::CommanderCastScry` (Study Hall,
riding Path of Ancestry's deferred scry push) and
`Effect::ExileAllThenTokenPerPlayerByPower` (Oversimplify). ⚠ **Guardian
Augmenter found a layer class**: a `PumpPT` / `GrantKeyword` static over a
filter with a stateful leaf the gather's live pass didn't know was dropped
whole — eight shipped cards did nothing (Song of Serenity, Shield of Kaldra,
Hellspur Posse Boss, Radiant Destiny, Shimmer, …); a catalog ratchet now holds
the class at zero. Residuals: **Esix** copies the engine's pick (greatest mana
value) and always takes the may; **Primal Empathy**'s counter goes on your
creature of greatest power; **Ruxa** reads printed "no abilities" and always
takes the unblocked-damage option. Four-seat pods beside Tatyova / Aesi / Ezuri
(seed 10122, 1,000 games, all decided): Adrix 25.8 %; a 300-game census
(seed 10124) leaves no card unplayed. `--bench` byte-identical.

The **forty-third** is Commander 2021's **Lorehold Legacies**
(`LoreholdLegacies_C21`) — Boros artifact recursion under Osgir, the
Reconstructor. Seventeen cards were missing; the primitives:
`StaticEffect::PreventAllCombatDamageToMatching` (Losheel's "attacking artifact
creatures you control") and `Effect::RevealUntilOneToBattlefieldRestBottom`
(Audacious Reshapers). ⚠ **Osgir found a cost bug**: an `exile_other_filter`
naming X ("an artifact card with mana value X") was evaluated unresolved and
refused every card; it now reads the activation's X. Wake the Past's "they gain
haste" needed `ReturnAllMatchingFromGraveyardToBattlefield` to record its cards
for `Selector::LastMoved`. Residuals: **Archaeomancer's Map** reads "that player
controls more lands than you" as any opponent; **Key to the City**'s "up to
one" always targets; **Laelia** counts only her own attack's library exile (no
event announces a library exile), and a battlefield exile beside the graveyard.
Four-seat pods beside Mizzix / Adrix / Daxos (seed 10123, 1,000 games, all
decided): Osgir 19.4 %; the 300-game census (seed 10124) leaves no card of the
four decks unplayed. `--bench` byte-identical.

The **forty-fourth** is Commander (2011)'s **Counterpunch** (`Counterpunch_CMD`)
— Abzan Saprolings and +1/+1 counters under Ghave, Guru of Spores (seat 41
before rebasing over Rebellion Rising, Peer Through Time and Lorehold Legacies).
Fourteen cards were missing (the Vows had landed with Call the Spirits); the
primitive: `Effect::PutAnyNumberFromGraveyardOnTop` (Footbottom Feast). ⚠ **The
census found Nemesis Trap castable by no bot path**: "target attacking
creature" exists only in combat, and the rule-based defensive picker skips
exile by design, so `pick_combat_only_instant` now also takes an instant whose
first target requires an attacker. ⚠ Behind it, an **engine gap**: a targeted
spell was accepted with no target at all (Murder and Doom Blade too — CR
601.2c; since enforced by another session). Residual: **Footbottom Feast** picks its cards as it resolves.
Debug pods beside Daxos / Ixhel / Edgar / Sigarda (seeds 9261/9262, 60 games)
decided 60/60, zero panics; after the picker fix a 120-game census (seed 9263)
leaves no card unplayed. Ghave wins 0–4 % of those pods. `--bench`
byte-identical.

The **forty-sixth** is Commander 2015's **Wade into Battle**
(`WadeIntoBattle_C15`) — Boros Giants under Kalemne, Disciple of Iroas.
Thirteen cards were missing; the primitives: `StaticEffect::SpellDamageToOpponentsBecomesTokens`
(Hostility, CR 615 — in the noncombat damage funnel ahead of any doubler, CR
616.1), `Value::OpponentsBelowHalfStartingLife` (Anya) and
`Value::RevealedForCostManaValue` (Disaster Radius). Residual: **Dream
Pillager**'s exiled cards may be played, not only cast. ⚠ **Its first debug
pod found an Aura bug with Peer Through Time's Fool's Demise**: the SBA
exemption for an Aura whose own trigger is on the stack (Animate Dead) also
kept an *attached* Aura whose host had left, the host returned with its old
id and re-acquired it, and Bottle Gnomes looped 4,543 sacrifices — a
30,757-action cap (CR 704.5m / 400.7; fixed). Seed 10311, 1,000 six-seat pods
(Kalemne ×2, Teferi, Osgir, Neyali, Daxos): **1,000 decided, every card of the
six lists played**; `--bench` byte-identical.

The **forty-seventh** is Commander Legends' **Arm for Battle** (`ArmForBattle_CMR`)
— Boros Auras and Equipment under Wyleth, Soul of Steel (seat 46 before
rebasing over Wade into Battle). Seventeen cards were missing; the primitives:
⚠ **CR 205.4e was unenforced** — the catalog had no legendary sorcery, and the
cast gate now refuses Jaya's Immolating Inferno without a legendary creature or
planeswalker (`game/legendary_spell.rs`); `StaticEffect::AttachedIsLegendary`
(On Serra's Wings, layer 4, with the legend rule's scan bit). ⚠ **The census
found Wild Ricochet castable by no bot path**: the copy window read only a
`Seq`'s first step, and Ricochet retargets before it copies. The two residuals
it landed with were closed after: **Dawn Charm**'s counter mode takes only a
spell that targets you (`SpellTargetsMatching(Player & ControlledByYou)`), and
**Timely Ward**'s flash reads its declared target
(`StaticEffect::SelfFlashIfTargets`). ⚠ The same pass found an Equipment's
"whenever equipped creature is dealt damage" lost when the damage killed the
host (Blazing Sunsteel; CR 603.2 — fixed in the death-snapshot walk). Debug pods
beside Tegwyll / Ghave / Edgar / Sigarda (seeds 9271/9272, 60 games) decided
60/60, zero panics; after the fix a 120-game census (seed 9273) leaves no card
unplayed. `--bench` byte-identical.

The **forty-fifth** is Wilds of Eldraine's **Fae Dominion** (`FaeDominion_WOC`)
— Dimir Faeries under Tegwyll, Duke of Splendor (Alela is in the 99).
Seventeen cards were missing; the primitives: `Effect::GoadForTheGame`
(Nettling Nuisance's Pirate, CR 701.38 — a flag the goader's untap expiry
skips), `Keyword::CantAttackPlayer` + `Effect::GrantCantAttackYou`
(Illusionist's Gambit, CR 508.1a) and `CardDefinition::flash_additional_cost`
(Tegwyll's Scouring's "flash by tapping three fliers", CR 601.2b). ⚠ **Halo
Forager found a reflexive bug**: `Effect::Reflexive` auto-targeted its body
with no X, so a "mana value X" payoff behind "pay {X}" matched nothing (CR
603.7). The first census found Illusionist's Gambit cast by no bot path;
`combat_only::pick_combat_only_spell` now takes a spell whose cast condition
names the declare blockers step when the seat is attacked. Residuals:
**Blightwing Bandit** exiles face up; **Halo Forager** can't cast a
mana-value-0 card; **Illusionist's Gambit**'s grants last the turn;
**Puppeteer Clique** exiles at the next end step, not necessarily yours.
Four-seat pods beside Osgir / Ghave / Adrix (seed 10125, 1,000 games, all
decided): Tegwyll 41.4 %.

The **forty-eighth** is Commander 2016's **Invent Superiority**
(`InventSuperiority_C16`) — four-colour artifacts under Breya, Etherium Shaper,
the pod's first WUBR identity. Seventeen cards were missing; the one new rule
piece is `EventKind::EnchantedPlayerLeftGame` (Curse of Vengeance's "when
enchanted player loses the game", queued from `objects_leave_with_player`,
the one CR 800.4a funnel, while the Aura is still attached) with
`CounterType::Spite`. Residual: **Armory Automaton** attaches the Equipment
you control, not other players'. Four-seat pods beside Tegwyll / Osgir /
Adrix (seed 10127, 1,000 games, all decided): Breya 17.1 %; a 300-game census
(seed 10128) leaves no card of the four decks unplayed. `--bench`
byte-identical.

The **forty-ninth** is Secret Lair's **Chaos Incarnate** (`ChaosIncarnate_SCD`)
— Rakdos goad and punishment under Kardur, Doomscourge. Sixteen cards were
missing; the primitives are table choices (`SacrificeAllButN`,
`EachOpponentChoosesFromGraveyard`, `EachOtherPlayerMayDraw`,
`GreatestDiscardersLoseLife`, `PlayerRef::RandomPlayer`), Kardur's
`WheneverCreatureEntersUntilYourNextTurn` and Theater of Horrors' dormant
`HolderTurnsAfterOpponentLostLife` grant. ⚠ **Kardur found a CR 603.10 gap**:
a creature that died attacking had already been removed from combat when
"whenever an attacking creature dies" was checked, so it never fired —
`left_while_attacking` now answers `IsAttacking` for this combat. Residuals:
**Kardur**'s later entrants are goaded by a delayed trigger, not a static;
**Theater of Horrors**' grant outlives it and can't play lands; **Wildfire
Devils**' random player gives up their first instant or sorcery.

The **fiftieth** is Aetherdrift Commander's **Eternal Might** (`EternalMight_DRC`)
— Esper Zombies under Temmet, Naktamun's Will, with Hashaton, Scarab's Fist in
the 99 (seat 49 before rebasing over Chaos Incarnate). Seventeen cards were
missing; the primitive: `StaticEffect::GrantCyclingToYourHandCards`
(Rhet-Tomb Mystic). ⚠ **No bot path cycles at all** — nothing constructs
`GameAction::Cycle`, so the three Deserts and the Mystic's grant are human and
UI paths only. Residuals: the "4/4 black Zombie" copies (Hashaton,
God-Pharaoh's Gift) add Zombie rather than replace the creature types; the
Gift and Rot Hulk take the greatest-power creature cards rather than a choice
or targets. Debug pods beside Breya / Wyleth / Edgar / Sigarda (seeds
9281/9282, 60 games) decided 60/60, zero panics, Temmet winning 30 % of each;
a 120-game census (seed 9283) leaves no card unplayed (Temmet 41.7 % there).

The **fifty-third** is Commander (2011)'s **Heavenly Inferno**
(`HeavenlyInferno_CMD`) — Mardu Angels, Demons and Dragons under Kaalia of the
Vast. Seventeen cards were missing; the primitives: `StaticEffect::WarOrPeace`
+ `Effect::EachPlayerChoosesWarOrPeace` (Archangel of Strife),
`StaticEffect::PlayersCantCastDuringCombat` (Basandra, a new cast-lock bit),
`Value::SourceActivationsThisTurn` (Dragon Whelp), and the target walk now reads
a player ref's own selector filter (`pref_find`) — "target opponent's
graveyard" (Tariel) and "each player other than target player" (Death by
Dragons) had no way to say which players were legal. Residuals: **Archangel of
Strife**'s choice is made as its ETB resolves; **Kaalia** also triggers
attacking a planeswalker.

The **fifty-fourth** is Zendikar Rising Commander's **Land's Wrath**
(`LandSWrath_ZNC`) — Naya landfall under Obuun, Mul Daya Ancestor (seat 53
before rebasing over Heavenly Inferno). Sixteen cards were missing and every one
fit existing primitives (aftermath Struggle // Survive, a Saga, bolster,
support, `WhenTargetDiesThisTurn`). Two bot gaps behind it: ⚠ **the
`ChooseModesCast` enumerator offered single modes only**, so a spell with
`min > 1` — the Strixhaven Commands' "choose two", a Confluence's "choose three,
repeats allowed" — had no legal bot cast; it now offers every pick of exactly
`min` modes, and Righteous and Wretched Confluence moved to cast-time modes (both
residuals closed). ⚠ **No bot path cycled** (Eternal Might): a pod seat now
cycles a spare land (six lands out) or a card more than two mana over its land
count at an opponent's end step (`server/cycling.rs`, pods only). Residuals:
**Scaretiller** picks its mode (hand land first, else an untargeted graveyard
land); **The Mending of Dominaria** regrows the greatest-power creature card;
**Trove Warden**'s cards return when it leaves by any route. Debug pods beside
Arahbo / Yidris / Temmet / Sigarda (seeds 9301/9302, 60 games) decided 60/60,
zero panics; a 120-game census (seed 9303) leaves no Obuun card unplayed.
`--bench` byte-identical through all of it.

The **fifty-fifth** is Commander (2011)'s **Political Puppets**
(`PoliticalPuppets_CMD`) — Jeskai donations and table politics under Zedruu the
Greathearted, with **one swap**: Trade Secrets is on the Commander ban list, so
Divination takes its slot (the pod asserts legality, and the list as printed
fails it). Sixteen cards were missing; the primitives:
`CumulativeUpkeepCost::GraveyardCardsToBottom` (Jötun Grunt),
`Effect::EachOpponentSacrificesSharingTypeWith` (Martyr's Bond),
`CardDefinition::library_bottom_on_resolve` (Spell Crumple), and clash now
picks the most hostile opponent and binds them as "that player" (Pollen
Lullaby). ⚠ **Crescendo of War found a layer gap**: a
`PumpPTPerCounterOnSource` over a live filter (attacking, blocking) was dropped
whole — only `PumpPT`/`GrantKeyword` rode the gather's live pass; it now shares
it, and the dropped-static ratchet covers it. Residuals: **Jötun Grunt**'s
graveyard and cards are the engine's pick; **Ruhan**'s random opponent is
stored on Ruhan.

The **fifty-seventh** is Commander 2016's **Stalwart Unity**
(`StalwartUnity_C16`) — four-colour group hug under Kynaios and Tiro of Meletis,
with Ludevic, Kraum and Sidar Kondo (three partner commanders) in the 99 (seat
56 before rebasing over Primal Genesis). Sixteen cards were missing; the
primitives: `CounterType::Bribery` and `Hoofprint`,
`Effect::PreventAllDamageToPlayerThisTurn` + `EventKind::DamageToPlayerPrevented`
(Selfless Squire, CR 615 — the event had no kind bit at first, so the trigger
could not fire), `Effect::OnMatchingBlocksThisTurn` (Benefactor's Draught) and
`Effect::EachPlayerMayCounterForPeace` (Orzhov Advokist, `game/effects/
politics.rs`). ⚠ **The strict debug pod found Collective Voyage broken**: its
join-forces body searched only seat 0's library and put those lands under the
caster (`resolve_player`'s fan-out assert) — now `EachPlayerDoes`, with a CR
207.2c test. Residuals: an opponent who declines Kynaios's land offer while
holding one doesn't draw; Humble Defector goes to a random opponent; Sidar
Kondo's evasion covers only your small creatures; Advokist counters a taker's
greatest-power creature. Debug pods beside Zedruu / Obuun / Kaalia / Sigarda
(seeds 9311/9312, 60 games) decided 60/60, zero panics; a 120-game census
(seed 9313) leaves no card unplayed (Kynaios 22.5 % there).

The **fifty-eighth** is Commander (2011)'s **Devour for Power**
(`DevourForPower_CMD`) — Sultai graveyard value under The Mimeoplasm, with
Damia, Vorosh and Skullbriar in the 99. Nine cards were missing (Spell Crumple
and Vow of Flight had landed with Political Puppets). Primitives:
`CardDefinition.keeps_counters_off_battlefield` + `CardInstance::
drop_counters_for_zone_change` (CR 122.2 in one helper, with Skullbriar's
exception; the redirect path used to keep a dead creature's counters on the
card in exile, a hand or the command zone), and an as-enters copy re-reading
its self-ETB list (CR 707.5 — The Mimeoplasm copying Mulldrifter draws two).
Residuals: The Mimeoplasm's two cards are the engine's pick; Desecrator Hag's
power tie. Pods (release, seed 9301, 400 games beside Krark / Giada / Neyali):
400/400 decided, zero panics, no Mimeoplasm card unplayed; seed 9302 × 500 at
2/4/6/8 seats all decided; 56 seats × 30 (seed 9303) all decided. `--bench`
byte-identical.

The **sixty-second** is Commander (2011)'s **Mirror Mastery**
(`MirrorMastery_CMD`) — Temur copies under Riku of Two Reflections, with
Animar and Intet in the 99. Sixteen cards were missing; the one primitive is
`Effect::EndMayPlayOnCardsExiledWithSource` (Intet's "for as long as Intet
remains on the battlefield", run by its leaves trigger). Residuals: Intet
exiles face up; Ray of Command taps at the next end step. Pods (release, seed
9331, 400 games beside Hanna / Yuriko / Gisa and Geralf): 400/400 decided, no
card of the four lists unplayed. `--bench` byte-identical.

The **sixty-fifth** is Starter Commander's **First Flight** (`FirstFlight_SCD`)
— Azorius fliers under Isperia, Supreme Judge. Eighteen cards were missing;
the one primitive is **CR 508.1d's lure**, `Effect::LureCreaturesToSourceNextTurn`
+ `PlayerCold.attack_lure` (`game/attack_lure.rs`) — Gideon Jura's +2: during
the target opponent's next turn every creature of theirs that can attack
must attack Gideon, and the bot's forced-attacker repair and target pass
honour it. No residuals. Pods (release, seed 9341, 1,000 games beside Judith /
Ghalta / Breya): 1,000/1,000 decided, no card of the four lists unplayed,
Isperia 29.4 %; 6 and 12 seats × 300 (seed 9343) all decided. `--bench`
byte-identical.

The **seventy-third** is Innistrad: Midnight Hunt's **Undead Unleashed**
(`UndeadUnleashed_MIC`) — Dimir Zombies under Wilhelt, the Rotcleaver.
Eighteen cards were missing. Primitives: `CardDefinition.
graveyard_exile_discount` (Gorex's filtered delve at {2} a card, the cards
stamped exiled-with the spell), `StaticEffect::CombatDamageToPlayersBecomesMill`
(Undead Alchemist, CR 614), `RevealUntilMatchingToBattlefield.rest_bottom`
(Empty the Laboratory), and `MoveChosen` now records `LastMoved` (Ghouls' Night
Out's "those cards"). Two bot finds: the delve gate ignored Gorex (the debug
gate caught it) and the X-target picker never looked in a graveyard, so Hour
of Eternity went uncast in 1,000 pods — cast after the fix, with a test.
Residuals: Hordewing Skaab, Hour of Eternity, Shadow Kin, Rooftop Storm
(INCOMPLETE_CARDS). Pods (release, seed 9361, 1,000 games beside Mimeoplasm /
Temmet / Stella Lee): 1,000/1,000 decided, no card unplayed, Wilhelt 21.1 %;
64 seats × 12 (seed 9363) all decided. `--bench` byte-identical.

**The field passed 64 lists this run and the engine's seat masks are `u64`**:
a 65-seat debug pod tripped `note_acted_on_own_turn`'s assert (in release seat
64 aliased seat 0's bit). `game::MAX_SEATS` is the cap, asserted in
`GameState::new`; `bot_ladder` clamps `--seats`/`--pod-decks` to it, so the
later lists are reached with `--pod-decks`.

**Smoke at 54 seats** (seed 9411, before seats 55-57, release, strict answer
log): 60 games, **60 decided, 0 caps, zero panics** — 608 turns, 350 k actions
and 2,979 plays a game, the longest 891 turns; four-seat pods (seed 9412, 400
games) all decided. `--bench` byte-identical throughout.
The **fifty-first** is Commander 2016's **Entropic Uprising**
(`EntropicUprising_C16`) — four-colour (UBRG) cascade and chaos under Yidris,
Maelstrom Wielder. Seventeen cards were missing; the primitives:
`EventKind::PlayerLeftGame` (Blood Tyrant, queued from the CR 800.4a funnel),
`Effect::PlayersControlEachOthersNextTurn` (Cruel Entertainment, CR 722) and
`Effect::ManifestFromGraveyard` (Ghastly Conscription, CR 701.40). ⚠⚠ **Its
first 1,000-game pod found a stack overflow**: a mana source whose own {T}
ability cost {1} paid that {1} by tapping itself, recursing through
`activate_ability_inner` until the stack blew; the pre-activation snapshot is
now taken after the {T}/{Q} costs are paid (CR 601.2g), with a replay test
(`pod::tests::a_mana_source_paying_for_itself_does_not_overflow`). ⚠ **Its
census found Wheel of Fate unplayed**: no bot path suspended a card, and
suspend-only cards (no mana cost) had no other way in —
`server/suspend.rs` now suspends them, and the census counts a suspend or a
foretell as a play. Residuals: **Aeon Chronicler** has no Suspend X;
**Vial Smasher** never hits a planeswalker; **Blood Tyrant** grows by the
living players, not the life lost. Four-seat pods beside Breya / Tegwyll /
Adrix (seed 10129, 1,000 games, all decided): Yidris 28.6 %; the census (seed
10130) leaves no card unplayed.

The **fifty-ninth** is Duskmourn Commander's **Endless Punishment**
(`EndlessPunishment_DSC`) — Rakdos punisher under Valgavoth, Harrower of Souls
(seat 57 before rebasing over Stalwart Unity and Devour for Power). Fifteen cards were missing; the primitives: `SpendRestriction::InstantSorceryOrTypes`
(Séance Board, CR 106.6), `CardDefinition::exile_on_resolve_time_counters` +
`CounterType::Soul` (Suspended Sentence, CR 702.62), and
`Effect::EachPlayerChoosesToDestroy` (Sadistic Shell Game). ⚠ **The Lord of
Pain found spell-cast triggers blind to their caster** (CR 603.2): the cast
trigger aimed with no event player set and stamped none on the stack item, so
"another target player" could never exclude the caster — now both are the
caster. Residuals: **Star Athlete**'s "up to
one" always takes a target; **Torture Pit**'s +2 also reaches opponents'
permanents (Barbflare Gremlin's and Enchanter's Bane's damage now comes from
the land / the enchantment, through Open Hostility's `Effect::DealDamageFrom`).
Four-seat pods beside Ghired / Zedruu / Arahbo (seed 10132, 1,000
games, all decided): Valgavoth 47.1 %; a 300-game census (seed 10133) leaves no
card of the four lists unplayed; 12 seats (57..46, seed 10134): 200 / 200
decided (seat numbers as they were then). `--bench` byte-identical.

The **sixty-sixth** is Commander 2017's **Arcane Wizardry**
(`ArcaneWizardry_C17`) — Grixis Wizards under Inalla, Archmage Ritualist (seat 64
before rebasing over Open Hostility and First Flight), whose eminence copies each nontoken Wizard that enters (from the command zone too).
Sixteen cards were missing; the primitives: `EntersAsCopy::from_graveyards`
(Body Double), `GraveyardCastOncePerTurn::exile_after` (Kess),
`Effect::PlayerChoosesToDestroy` (Magus of the Abyss) and
`StaticEffect::HasActivatedAbilitiesOfOwnedExiledWithCounter` + `CounterType::Cage`
with `AddCounter` reaching exile (Mairsil, CR 122.1). It found four engine
bugs: ⚠ **The Abyss destroyed every nonartifact creature** of the active player
(it is one, of their choice); "tap N untapped [X] you control" never let the
source pay (Inalla is one of its five Wizards; Crookclaw Elder one of its two
Birds); ⚠⚠ **both death funnels pushed a trigger with its first target slot
only** (CR 115.1c), and the slot filler offered only the *first* opponent — a
two-seat assumption, so Vindictive Lich's "each mode must target a different
player" hit one seat at four. Residuals: **Mairsil** activates a borrowed
ability any number of times a turn and cages the highest-mana-value card;
**Magus of the Abyss**'s pick is a choice, so hexproof doesn't stop it;
**Shifting Shadow** reveals from the Aura controller's library and the new
creature enters before the old one is destroyed; **Vindictive Lich** always
picks all three modes in a fixed order. Four-seat pods beside Rin and Seri /
Riku / Atraxa (seed 10140, 1,000 games, all decided): Inalla 7.8 % — Rin and
Seri take 53 %; a 300-game census (seed 10141) leaves no card of the four
unplayed; 12 seats (64..53 as numbered then, seed 10142): 200 / 200 decided. `--bench`
byte-identical.

The **sixty-ninth** is The Brothers' War Commander's **Urza's Iron Alliance**
(`UrzaSIronAlliance_BRC`) — Esper artifacts under Urza, Chief Artificer, and the
first list past `MAX_SEATS` (64): it is reached by `--pod-decks`, not `--seats`.
Sixteen cards were missing; the primitives: `GraveyardCardsHaveEncore::mana_cost`
(Wire Surgeons, CR 702.141). It found two engine gaps: ⚠ **an any-kind "remove a
counter" cost never saw keyword counters** (CR 122.1b — Hexavus could hand out
flying counters but never take them back), and **a self "prevent all damage to
this" static peeled `WhileYourTurn` alone**, so Sanwell's "as long as an
artifact creature you control is attacking" was dead. Residuals: **Sanwell**
offers the first matching card of the six and bottoms the rest in exile order;
**Scholar of New Horizons** always takes the battlefield when it may.
Four-seat pods beside Brimaz / Inalla / Isperia (seed 10150, 1,000 games,
all decided): Urza 41.7 %; a 300-game census (seed 10151) leaves no card of the
four unplayed; 12 seats (68..57 as numbered before rebasing over Enduring Enchantments, seed 10152): 200 / 200 decided. `--bench`
byte-identical.

The **seventieth** is Kaldheim Commander's **Phantom Premonition**
(`PhantomPremonition_KHC`) — Azorius foretell and flicker under Ranar the
Ever-Watchful. Sixteen cards were missing (Inspired Sphinx and Thunderclap
Wyvern landed with First Flight first). ⚠⚠ **No bot path foretold a card or
cast a foretold one**, so all thirteen foretell cards in the catalog were plain
hard-casts in self-play: the bot now foretells when its main phase would pass
(`server/foretell.rs`) and casts foretold cards through the exile enumeration.
⚠ **CR 702.143a — the foretell action demanded sorcery speed**; it is legal any
time its owner has priority during their turn. The primitives:
`EventKind::CardsExiledFromHandOrByYou` (cards leaving a hand for exile, and
permanents a player's spells and abilities exile, tallied at the exile
chokepoints and synthesized once per player per dispatch with the count —
Ranar, Hero of Bretagard), `StaticEffect::FirstForetellEachTurnFree`,
`Effect::ForetellFromHand` + granted foretell costs (Ethereal Valkyrie — either
cost may be paid), `Value::ForetoldCardsOwnedInExile`,
`SpendRestriction::ForetellOnly` (Niko; foretold casts now pay with the spell's
own kind, so restricted mana sees them), `R::HasForetell`,
`R::ExiledInsteadOfDyingThisTurn` (Cosmic Intervention),
`Effect::ExileFromHandCopyCreature` (Arcane Artisan) and `CounterType::Night`.
The census then found **Eerie Interlude unplayed**: the sweeper-shield response
knew phasing and indestructible only; an exile-until-end-step blink now counts.
A strict debug pod beside Inalla / Anikthea / Urza (seed 9524) caught **the
`AB_TOKEN` gate dropping Heliod's {2}{W}{W} under Mirari's Wake** — extra mana on
a land tap now widens the bot's colour budget. Residual: Cosmic Intervention's
replacement covers the permanents you control as it resolves. Release pods
beside Saskia / Inalla / Kaalia (seed 9521, 1,000 games): 1,000 decided, Ranar
52.2 %; a 300-game census (seed 9522) leaves no card of the four unplayed;
strict debug pods (seeds 9523/9524, 120 games) decided 120/120. `--bench`
byte-identical.

The **seventy-second** is Battle for Baldur's Gate's **Exit from Exile**
(`ExitFromExile_CLB`) — Gruul impulse draw, cascade and Wolves under
Faldorn, Dread Wolf Herald, reached by `--pod-decks 72`. Seventeen cards
were missing (`cmdr_faldorn.rs`). ⚠ **"Cast from exile" was three casts
short**: foretold, adventure-creature and plotted casts never stamped
`cast_from_exile` (CR 702.143a / 715.4 / 702.170d), and a land *played*
from exile never counted as entering from exile (CR 305.1) — so Faldorn's
own Wolves, Fire Lord Zuko and Nassari missed them. The primitives:
`Predicate::FirstSpellCastFromExileThisTurn` over
`Player.spells_cast_from_exile_this_turn` (Wild-Magic Sorcerer) and
`StaticEffect::FreeExileCastOncePerTurnMatching` (Tlincalli Hunter's
creature-only {0}; the shared `free_exile_cast_waiver` now reaches an
adventurer's cast, Warped Space's too). Residuals: **Aurora Phoenix** doesn't
see cascade granted by a trigger; **Chaos Wand** leaves an uncast find in
exile; **Durnan** takes the first creature of the four and its cast has no
undaunted; **Stolen Strategy** lets an exiled land be played. Pods (release,
seed 10221, 1,000 games beside Trostani / Ranar / Urza): 1,000/1,000
decided, no card of the four lists unplayed, Faldorn 13.4 %; six seats
beside Anikthea / Trostani / Ranar / Urza / Brimaz (seed 10222, 400 games)
399 decided, one board cap — 17,194 Pegasi from Storm Herd under Angelic
Chorus in the Trostani list. `--bench` byte-identical.

The **seventy-first** is the Secret Lair **Hatsune Miku** deck
(`HatsuneMiku_SLD`) — Selesnya lifegain and tokens under Trostani, Selesnya's
Voice; like Urza it is past `MAX_SEATS` and reached by `--pod-decks 71`
(committed as "sixty-eighth" before rebasing over three concurrent seats).
Seventeen cards were missing; the primitives: `ActivatedAbility::untap_others_cost`
(Halo Fountain), `ArtifactSubtype::Junk` + `tokens::junk_token` (Break Down),
`Effect::RevokeGrantedActivatedAbility` (Song of Freyalise's "until your next
turn"). It found two engine gaps: **`Value::ColorCountOf` never saw a spell on
the stack** (it walked battlefield/graveyard/hand/exile by hand; now
`find_card_anywhere`), and ⚠ **the bot stacked its own Aetherflux Reservoir
shots forever** (CR 117.3c hands priority back to the activator — 5,486 on the
stack at a 6-seat action cap); a HeuristicBot seat now lets its own ability from
a source resolve first. Residuals: **Ancient Cornucopia**'s once-a-turn limit is
on the trigger; **Lazotep Quarry**'s token keeps its creature types. Four-seat
pods beside Sigarda / Teval / Disa (seed 11068, 1,000 games, all decided):
Trostani 46.1 %; the census leaves no card of the four unplayed. ⚠ **The
lifegain engine is the pod's new board-cap source**: at seed 11069, 1,000 games
each at 4 / 6 / 8 seats read 1 / 0 / 2 undecided, every one `board cap` — Boon
Reflection doubles Trostani / Angelic Chorus life per creature, and Storm Herd
at ~1,000+ life makes that many Pegasi in one resolution, past
`MAX_BATTLEFIELD` (1,024). The cards are right; the bound is the simulator's.
`--bench` byte-identical.

The **seventy-eighth** is Commander 2021's **Witherbloom Witchcraft**
(`WitherbloomWitchcraft_C21`) — Golgari life gain and loss under Willowdusk,
Essence Seer (committed as seat 73, then 77, while four concurrent seats
landed). Seventeen cards were missing; the primitives:
`Effect::NextSpellThisTurnMayCostLife` (Marshland Bloodcaster),
`Effect::ReturnFaceDownAsForest` (Yedora — a face-down land is not a
manifested card, so no mana cost turns it up), `Effect::ReturnOnePerPermanentType`
(Revival Experiment, CR 110.4), `Value::NontokenCreaturesEnteredThisTurn`
(Gyome). It found three engine gaps: ⚠ **a resolving creature spell never
stamped `creatures_entered_this_turn`** — only tokens and zone moves did, so
Geralf's and Ephara's counts missed cast creatures; **`PowerOf` / `ToughnessOf`
could not see the library or the stack**; **`CastSpellMatches` answered every
source-reading atom `false`** ("a spell of the chosen color"). Residuals:
**Revival Experiment**'s picks are the engine's; **Suffer the Past** chooses as
it resolves. Four-seat pods beside Sigarda / Teval / Disa (seed 11073, 1,000
games, all decided): Willowdusk 19.5 %; the census leaves no card of the four
unplayed. At 6 seats (73 then, beside Krark / Ixhel / Daxos / Emmara / Ranar,
seed 11074) 3 of 1,000 hit `board cap` — **Emmara's Selesnya Guildmage /
Vitu-Ghazi Saprolings (769-939 of them)**, not this list: the same shape as
Storm Herd, a token engine outgrowing `MAX_BATTLEFIELD` in a long stall.
`--bench` byte-identical.

The **eighty-sixth** is Secrets of Strixhaven Commander's **Witherbloom
Pestilence** (`WitherbloomPestilence_SOC`) — Golgari Pests and sacrifice under
Dina, Essence Brewer (committed as 81, then 83, while five concurrent seats
landed). Fourteen cards were missing and **no primitive was**: prepare
(Eccentric Pestfinder // Turn Stones, Stensian Sanguinist // Exsanguinate),
devour, `AdditionalCastCost::SacrificeAnyNumber` and `LifeGainBonus` all
existed. Its census found one bot gap with two causes: ⚠ **Immoral Bargain
was never cast** — the bot picked no X for a cost without an {X} pip, and a
prompting seat was asked for a target past `TargetsExactlyX`'s X, so the cast
suspended and `would_accept` read it as illegal (`Effect::slot_past_x_cap`,
also Pest Infestation's "up to X"). Residuals: **Gorma**'s extra counters
reach cast creatures only; **Stensian Sanguinist**'s "this combat" is "this
turn". Four-seat pods beside Sigarda / Teval / Disa (seed 11081, 1,000 games,
all decided): Dina 15.9 %; the census leaves no card of the four unplayed.
`--bench` byte-identical.

The **ninety-third** is Murders at Karlov Manor Commander's **Blame Game**
(`BlameGame_MKC`) — Boros goad under Nelly Borca, Impulsive Accuser. Eighteen
cards were missing; the primitive is **goad with one reader**:
`GameState::goaders` (`game/goad.rs`) joins a resolved goad, a held one
(`CardCold::goad_holds`, CR 611.2b — Hot Pursuit while it remains, Immortal
Obligation while its duty counter stays, which also bars attacking and blocking
the caster) and `StaticEffect::AttachedIsGoaded` on an Aura or Equipment; combat,
the bot, `pod_attack` and the view read through it, `R::IsGoaded` matches it.
Martial / Shiny / Ghoulish / Parasitic Impetus and Bloodthirsty Blade had
re-goaded on triggers (outliving the Aura); all five are the static now. Also
`EachPlayerVotesForAPlayer` (CR 701.38 secret council), `OpponentsChooseSilenceOrSnitch`,
`EachPlayerMayCounterThenGoad`, `DoubleDamageToOpponentPlayers` (CR 614.5),
`PreventAllCombatDamageToPlayerAndWalkersThisTurn`, `Predicate::PlayersLostAtLeast`,
`MayPayToCopyOntoOtherCreatures` and `ReturnSelfRetypedWithCounters`. Its pods
found ⚠ **Boros Reckoner hitting itself forever** (a bare `Target(0)` on a
`DealtDamage` trigger binds the damaged object) and ⚠ **the bot's any-target
fallback choosing its own board before an opponent's face**; strict debug pods
found the block planner's gate skipping the duty counter's block bar.
Residuals: Agitator Ant's counters go on the greatest-power creature; a
headless Feather copies onto its own creatures only; a re-added duty counter
re-arms Immortal Obligation. Four-seat pods beside Dihada / Gimbal / Mishra
(seed 10210, 1,000 games, all decided after the fixes): Nelly 53.9 %; beside
Ms. Bumbleflower / Derevi / Windgrace (seed 10211): 1,000 / 1,000, Nelly 48.0 %;
census (seed 10212): no card of the four unplayed; 12 seats (seed 10213):
200 / 200; strict debug pods (seeds 10214-10217, 4 and 6 seats, 240 games):
240 / 240. `--bench` byte-identical; cube/sos/sealed 300 games an archetype
(seed 10220): 7,500 decided.

The **ninetieth** is Commander 2021's **Silverquill Statement**
(`SilverquillStatement_C21`) — Orzhov politics and Inklings under Breena, the
Demagogue (committed as 88). Eighteen cards were missing; the primitive is
**CR 508.1's "whenever a player attacks one of your opponents"** —
`EventScope::OpponentOfYoursAttacked`, fired once per attacked player for every
listener that player opposes, the attacker in the target slot and the attacked
opponent as `Triggerer` (Breena, Combat Calligrapher) — plus
`Value::LowestOpponentLife` / `MostControlledByAnOpponent` and an attack tax that
reads its attacker (Nils). Its pods found a bot bug: ⚠ **3,512 Aetherflux
Reservoir shots into an indestructible Zetalpa** — the pinger called damage
lethal to an indestructible creature and never aimed a paid repeat shot at a
60-life face. Residuals (seven now — the Blade and Parasitic Impetus goad by static since seat 93 — all on the card docs): Breena's counters and
Nils's targets are engine picks; Bold Plagiarist copies +1/+1 counters only; Guardian Archon's
protection from a player is hexproof + indestructible; Inkshield counts
unblocked power; Author of Shadows takes the first nonland card; Tragic
Arrogance keeps each player's best; Victory Chimes's mana is yours. Four-seat
pods beside Sigarda / Teval / Disa (seed 11088, 1,000 games, all decided):
Breena 25.1 %; census: no card of the four unplayed; 6 and 8 seats and a
Strixhaven-only pod (seed 11089): 3,000 / 3,000 decided after the fix.
`--bench` byte-identical.

The **hundred-and-sixth** is Adventures in the Forgotten Realms Commander's
**Aura of Courage** (`AuraOfCourage_AFC`) — Bant Auras and Equipment under
Galea, Kindler of Hope. Seventeen cards were missing; the primitives:
`StaticEffect::LibraryTopEquipmentAttachesOnEntry` + `CardCold::attach_on_entry`
(Galea — the rider rides `statics_granted_triggers_for`, because a resolving
spell and `fire_self_etb_triggers` gather self-ETBs in two walkers),
`EquipCostReducedByTargetPower` (Belt of Giant Strength),
`Effect::ReturnSelfAttachedTo { host }` (Gryff's Boon), CR 706.2's
`RolledNaturalMax` event (Netherese Puzzle-Ward) and
`NextSpellHasFlashThisTurn` (Ride the Avalanche, whose delayed "when you next
cast" trigger now picks its target with the spell's mana value in scope).
Residuals: Clay Golem rolls on resolution; Song of Inspiration returns the
cards before the roll; Valiant Endeavor destroys one creature at a time.
Four-seat pods beside Killian / Vrondiss / Kathril (seed 10250, 1,000 games,
all decided): Galea 5.3 % — the bot does not build a Voltron threat; beside
Estrid / Eshki / Nelly (seed 10251): 1,000 / 1,000, 5.3 %; census (seed 10252):
no card of the four unplayed; strict debug pods (seeds 10253-10255, 4 and 6
seats): 180 / 180. `--bench` byte-identical.
The **hundred-and-fortieth** is Streets of New Capenna Commander's
**Maestros Massacre** (`MaestrosMassacre_NCC`, 2022-04-29) — Grixis
spell-copying and casualty under Anhelo, the Painter, `--pod-decks 140` (measured as 139).
Twenty-two cards were missing (`cmdr_anhelo.rs`). New primitives:
`StaticEffect::FirstInstantSorceryHasCasualty` read through
`GameState::casualty_for` by both the `CastSpellCasualty` action and the bot's
casualty block (Anhelo); `Effect::OwnerShufflesInExilesTopPlaysOrCasts`
(Audacious Swap). The rest compose: `SeparateIntoPiles` per opponent (Make an
Example), `GrantSuspend` (Sinister Concierge), graveyard-scoped triggers
(Dogged Detective's opponent draw, Skyclave Shade's landfall, Syrix),
`GrantMayPlayForLife` (Xander's Pact), `Hideaway` + a capped free cast
(Smuggler's Buggy). Residuals: **Maestros Confluence** goads the hostile
opponent's creatures; **Parnesse** protects only your permanents and offers
no copy; **Sinister Concierge** suspends an opponent's creature;
**Syrix** reads any card leaving your graveyard; **Waste Management** kicked
takes the hostile opponent's graveyard; **Xander's Pact** lets exiled lands be
played; **Zndrsplt's Judgment** makes you the only friend. Pods (1,000 games
each, all decided): 4 seats beside Urza / Osgir / Eshki (seed 11112) Anhelo
12.0 %, census: no card of the four unplayed; 6 seats beside Saheeli / Zimone
/ Nalia / Kaust / Morska and 8 seats (seed 11113). `--bench` byte-identical.

The **hundred-and-thirty-seventh** is Secrets of Strixhaven Commander's
**Quandrix Unlimited** (`QuandrixUnlimited_SOC`, 2026-04-24) — Simic X spells
and +1/+1 counters under Zimone, Infinite Analyst, `--pod-decks 137`.
Twenty-one cards were missing (`cmdr_zimone_ia.rs`; the Jump Scare! Zimone is
`cmdr_zimone.rs`). New primitives: `StaticEffect::ExtraPlusOneCountersMatching`
(Benevolent Hydra's "another creature", Ozolith's "artifact or creature");
`StaticEffect::FirstMatchingSpellEachTurnCostsLessPerCounter` (Zimone);
`Effect::OnYourNextSpellMatchingThisTurn`, which hands the body the cast
spell's X (Brass Infiniscope); `Effect::DoubleXOfSpell` (Unbound Flourishing);
`Effect::GrantSpellsFlashThisTurn` (Alchemist's Refuge). ⚠ On the stack an X
spell's cost reads with X filled in (CR 202.3e), so `R::HasXInCost` inside
`Predicate::CastSpellMatches` is false for any X > 0 — gate on
`Predicate::CastSpellHasX` instead. Residuals: **Kinetic Ooze** at X 10
doubles each other creature of yours, not chosen targets; **Primo** reads the
batch's first damage event; **Unbound Flourishing** doesn't copy {X}
abilities. Pods (1,000 games each, all decided): 4 seats beside Urza / Osgir /
Eshki (seed 11110) Zimone 17.2 %, census: no card of the four unplayed; 6
seats beside Nalia / Kaust / Morska / Zimone (DSC) / Sefris and 8 seats (seed
11111). `--bench` byte-identical.

The **hundred-and-thirty-fifth** is Murders at Karlov Manor Commander's
**Deadly Disguise** (`DeadlyDisguise_MKC`, 2024-02-09) — Naya morph, disguise
and cloak under Kaust, Eyes of the Glade, `--pod-decks 135` (measured as 134). Eighteen cards
were missing (`cmdr_kaust.rs`). New primitives: `R::TurnedFaceUpThisTurn`
(Kaust — `TurnRegistries.turned_face_up_this_turn`, stamped where a
`TurnedFaceUp` event is dispatched); `Effect::PutFaceDownOntoBattlefield`
(Ashcloud Phoenix); `Effect::NextFaceDownSpellCostsLessThisTurn`, spent by the
next face-down cast, and `StaticEffect::DoubleControllerTurnedFaceUpTriggers`
over a new `triggered_by_face_up` candidate flag (Panoptic Projektor); and
`CardDefinition.turned_face_up_counters`, applied inside
`CardInstance::turn_face_up`, so Hooded Hydra is never a face-up 0/0 for state-based actions to kill. Residuals: **Boltbender** re-aims one
target spell; **Tesak** doesn't grant unleash; **Unexplained Absence** targets
only opponents' permanents, without the one-per-player limit; **Veiled
Ascension**'s entering flying counter comes from a trigger. Pods (1,000 games
each, all decided): 4 seats beside Urza / Osgir / Eshki (seed 11108) Kaust
20.5 %, census: no card of the four unplayed; 6 seats beside Zimone / Sefris /
Morophon / Prossh / Gonti and 8 seats (seed 11109). `--bench` byte-identical.

The **hundred-and-thirty-second** is Adventures in the Forgotten Realms
Commander's **Dungeons of Death** (`DungeonsOfDeath_AFC`, 2021-07-23) — Esper
reanimation and venturing under Sefris of the Hidden Ways, `--pod-decks 132` (measured as 128).
Twenty cards were missing; nineteen are in `cmdr_sefris.rs` (Extract Brain landed first in `cmdr_gonti.rs`). New primitives:
`PlayerRef::PlayerToYourRight` (Bucknard's Everfull Purse — the nearest living
seat against turn order); `StaticEffect::DungeonRoomsTriggerTwice` (Hama
Pashar — extra room copies push above the original so the CR 309.6
completion tail resolves once, last); `StaticEffect::
ExileResolvingInstantsAndSorceries` plus a `total_mana_value` cap on
`CastAnyOrderWithoutPaying` (Rod of Absorption); and `CastFromHandWithoutPaying`
now resolves an X-relative filter, so `WithX` over a die result gates it
(Arcane Endeavor). Residuals: **Grave Endeavor** returns the greatest-power creature card; **Nihiloor** always
taps itself and takes one creature; **Phantom Steed**'s copy isn't an
Illusion; **Rod of Absorption** exiles spells cast before it arrived. Pods
(1,000 games each, all decided): 4 seats beside Urza / Osgir / Eshki (seed
11106) Sefris 20.3 %, census: no card of the four unplayed; 6 seats beside
Quintorius / Oloro / Henzie / Millicent / Yuma (seed 11107) and 8 seats
(seed 11107). The 6-seat run takes 128 s, 104 s of it one game (259): a Yuma
Scute Swarm board of 814 copies whose 8,171 no-op landfall triggers each
cycle six seats of priority. It still decides. `--bench` byte-identical.

The **hundred-and-twenty-seventh** is Secrets of Strixhaven Commander's
**Lorehold Spirit** (`LoreholdSpirit_SOC`, 2026-04-24) — Boros Spirits and
graveyard departures under the planeswalker commander Quintorius, History
Chaser, `--pod-decks 127`. Twenty-one cards were missing
(`cmdr_quintorius.rs`); they compose from existing parts
(`CardLeftGraveyard` batched, Class levels, `NonHandCastCostReduction`,
`Vote`/`PerVote`, `MoveWithinTotalManaValue`, `PumpPTByValue` over
`CommanderCastsFromCommandZone`) plus `PlaneswalkerSubtype::Quintorius`. It
found one engine gap: ⚠ **`ExileFromGraveyard` / `ExileBottomOfGraveyard`
never recorded what they exiled this resolution**, so an "exiled this way"
count after them read 0 (Augusta, Order Returned). Residuals: **Ao, the Dawn
Sky** leaves the unpicked cards on top; **Quintorius, Loremaster** casts the
exiled card as the ability resolves and doesn't bottom it; **Serra Paragon**'s
lands don't share its once-a-turn limit and its rider isn't granted. Four-seat pods beside Sigarda / Teval / Disa (seed 11104,
1,000 games, all decided): Quintorius 35.7 %; census: no card of the four
unplayed; 6 and 8 seats (seed 11105) 1,000 / 1,000 each once a self-copying
token stops at `BOARD_GATE` (they were 997 / 998, every cap a Scute Swarm).
`--bench` byte-identical.

The **hundred-and-twenty-third** is Outlaws of Thunder Junction
Commander's **Desert Bloom** (`DesertBloom_OTC`, 2024-04-19) — Naya Deserts
and lands-matter under Yuma, Proud Protector, `--pod-decks 123` (committed as 122). Nineteen
cards were missing (`cmdr_yuma.rs`); no new primitives — they compose from
landwalk, `MayPlayLandsFromGraveyardMatching`, `LandPutIntoGraveyard` /
`PutIntoGraveyard` triggers, `LandTypeChanger`, `GrantActivatedAbility`,
`ApplyToTargets` and `ForEach` over graveyard lands. Residuals:
**Cataclysmic Prospecting** can't see mana spent from Deserts (its Treasures
count your tapped Deserts); **Dune Chanter** doesn't make land cards off the
battlefield Deserts. ⚠ The deck's Scute Swarm, fed by lands entering from
effects, doubles pods to the board cap: 4 seats (seed 11102) 999 / 1,000
decided, 6 seats (seed 11103) 996, 8 seats 994, every cap a Scute Swarm (or
Extravagant Replication) board. A land-drop gate stops the bot's own drop
past `BOARD_GATE`; clamping token creation at the gate instead removed every
cap but tripled the 8-seat pod's time (38 s → 107 s) on the ~900-permanent
boards it left running, so it was reverted (another session's land-drop gate,
`40198e5d`, landed alongside). Yuma 31.0 % in the four-seat
pods; census: no card of the four unplayed. `--bench` byte-identical.

The **hundred-and-fifteenth** is Secrets of Strixhaven Commander's **Prismari
Artistry** (`PrismariArtistry_SOC`, 2026-04-24) — Izzet spells and Elementals
under Rootha, Mastering the Moment, `--pod-decks 115` (committed as 107, 110
and 113 while eight concurrent seats landed). Nineteen cards were missing
(`cmdr_rootha.rs`); Surge to Victory and Redoubled Stormsinger also landed
with Prismari Performance (`cmdr_zaffai.rs`) and Mardu Surge (`cmdr_zurgo.rs`),
whose versions were kept.
The primitives: `Value::GreatestInstantOrSorceryManaValueCastThisTurn`
(stamped at cast time with X — Rootha), `CopySpellForEachOtherLegalCreature.
casters_creatures` (Mirrorwing Dragon — the caster's creatures, each checked
against the spell's own filter), `Effect::OpponentVetoesOne` (Plargg and
Nassari) and `Effect::FaceDownFaceUpPiles` (Abstract Performance — the
chooser is asked off the library before anything moves). It found two engine
bugs: ⚠ **a cast trigger read an X spell's mana value off its printed cost**
(CR 202.3e — Braingeyser for 3 was a 2 to "mana value 5 or greater") and ⚠
**`R::HasName` wasn't card-only, so every name-scoped continuous effect was
dropped** (Leitmotif Composer's unblockability did nothing). And three
rules-correct doublings the simulator can't hold: Leitmotif Composers copying
themselves on every big spell, Redoubled Stormsinger tokens copying each
other's copies under Harmonic Prodigy, Surge to Victory's free copies over a
Composer board — now board-bound gates (cast-trigger fan-out and pending
stack tokens in the cast gate, a declaration gate on attack triggers, and a
declined free copy past `MAX_BATTLEFIELD`). Residuals: **Abstract
Performance**'s "face-down" pile is exiled face up and the chooser is the
hostile opponent; **Plargg and Nassari**'s vetoing opponent is the hostile
one. ⚠ The 8-seat pod also found **Orzhov Advokist's peace offer taken from
the last opponent standing** (617 Pegasi held home for twenty turns, one game
42 s): the ask is `OptionalKind::PeaceOffer` now and the bot declines it
then. Four-seat pods
beside Sigarda / Teval / Disa (seed 11096, 1,000 games, all decided): Rootha
20.3 %; census: no card of the four unplayed (Twinflame, unplayed before the
"up to N targets" bot fix, now casts); 6 and 8 seats (seed 11097) 1,000 /
1,000 each. `--bench` byte-identical.

The **hundred-and-ninth** is Kamigawa: Neon Dynasty Commander's **Upgrades
Unleashed** (`UpgradesUnleashed_NEC`) — Gruul "modified" creatures (CR 700.9)
under Chishiro, the Shattered Blade, `--pod-decks 109` (committed as 102 while
seven concurrent seats landed). ⚠ The printed deck ships **two Mossfire
Valleys**, which isn't singleton (CR 903.5b); the seat runs a thirteenth
Mountain in the second one's place. Eighteen cards were missing
(`cmdr_chishiro.rs`); the primitive is `Selector::AttackedBySource` (Mage
Slayer). Its tests found ⚠ **a leaving permanent's last-known P/T dropped every
static and attachment bonus** (CR 603.10 — the death snapshots were plain
clones; `GameState::lki_clone` bakes the computed P/T in), and `R::IsModified`
answered "unmodified" for a creature whose own Aura had already been orphaned
(it now reads `auras_at_death`, CR 603.10a). Its pods found two bot gaps: ⚠
**`ApplyToTargets` read hostile to the auto-targeter**, so an "up to X target
creatures you control" counter spell never filled a slot (Silkguard uncast in
1,800 games), and **a karoo played onto an empty board bounced itself every
turn** (3 no-progress draws in 2,000 games). Residual: Concord with the Kami
takes every mode that can act. Four-seat pods beside Kasla / Ulalek / Eshki
(seeds 10311, 10321-10323, 4,000 games) and Nelly / Ellivere / Lathliss (seed
10312, 1,000): all decided, Chishiro 32.0 % and 14.6 %; census (seed 10323):
no card of the four unplayed. `--bench` byte-identical; cube/sos/sealed (seed
10320): 7,500 decided.

The **hundred-and-forty-first** is Adventures in the Forgotten Realms
Commander's **Planar Portal** (`PlanarPortal_AFC`) — Rakdos exile-and-play
under Prosper, Tome-Bound, `--pod-decks 141` (committed as 137 and 138 while
four other seats landed). Twenty cards were missing (`cmdr_prosper.rs`; Grim
Hireling came with Party Time). The primitives:
`StaticEffect::DiesToExileInstead` (Lorcan's Warlocks, read against the
battlefield permanent so an effect-given type counts); CR 603.10
`left_while_blocking` (Death Tyrant); `Selector::SacrificedThisResolution`
(Danse Macabre); `EventScope::YouAttackedPlayer` (Karazikar);
`DelayedKind::OpponentPermanentDamagesYouThisTurn` (Hellish Rebuke);
`Predicate::TriggerCardExiledWithSource` (Share the Spoils). Its survey
found ⚠ **a turn-granted trigger's filter was read from each permanent's own
side**, so Predators' Hour's and Arm with Aether's "creatures you control"
armed every seat's creatures (the entry now carries its granter); its tests
found `SacrificeAndRemember` skipping the resolution's sacrifice bookkeeping.
Residuals: five (INCOMPLETE_CARDS). Four-seat pods beside Morophon / Omo /
Olivia (seed 10440) and Nelly / Eshki / Estrid (seed 10441), 1,000 games
each: all decided, Prosper 7.6 % and 7.3 %; census: no card of the list
unplayed. `--bench` byte-identical; cube/sos/sealed (seed 10402): 7,500
decided.

The **hundred-and-thirty-first** is the Secret Lair Commander deck
**Everyone's Invited!** (`EveryoneSInvited_SLD`, 2025) — five-color
changelings under Morophon, the Boundless, `--pod-decks 131` (committed as 129
until Grand Larceny and Power Hungry landed). Nineteen cards were missing
(`cmdr_morophon.rs`). The primitives: CR 700.8 largest-party matching in
`effects/party.rs`, shared by `Effect::EachPlayerKeepsPartySacrificesRest`
(Stick Together) and `Effect::LookTopTakeParty` (Harper Recruiter);
`Effect::StampTokenCopyExceptions` (`effects/token_riders.rs` — Brenard's Food
Golem copies carry the Food subtype and the sacrifice ability as copiable
values, CR 707.9b); `EntersAsCopy.extra_supertypes` (Moritte's snow). Its
tests found ⚠ **`EntersAsCopy.legendary` was declared and never read** (a
Sakashima the Impostor copy entered non-legendary), and ⚠ **CR 611.2b: a
temporary copy "until your next turn" reverted at this turn's cleanup** (and
an "until your next untap" one never reverted) — `TempCopy` now carries its
controller and installed turn, and those end as that player's next turn
begins. Residuals: seven (INCOMPLETE_CARDS). Four-seat pods beside Omo /
Olivia / Jirina (seed 10430) and Nelly / Eshki / Estrid (seed 10431), 1,000
games each: all decided, Morophon 11.9 % and 8.4 %; census: no card of the
list unplayed. `--bench` byte-identical.

The **hundred-and-twenty-second** is Modern Horizons 3 Commander's **Tricky
Terrain** (`TrickyTerrain_M3C`) — Simic lands-matter under Omo, Queen of
Vesuva, `--pod-decks 122`. Eighteen cards were missing (`cmdr_omo.rs`). The
primitives: `CounterType::Everything` with `LandType::NONBASIC` (Omo's lands
are every land type); `Effect::ExileAllOtherSpellsCounterAllAbilities`
(Summary Dismissal, `effects/stack_sweep.rs` — exiling isn't countering, CR
701.5); `Effect::EachPushesTrigger` (Aggressive Biomancy's copies each fight
as their own trigger, `effects/token_triggers.rs`);
`Predicate::CostReturnedHadNonbasicLandType`, read off the layered type line
as the cost is paid (Wonderscape Sage, CR 603.10);
`SpendRestriction::SpellOrAbilityCopy` (Sunken Palace's spell half);
`DelayedTriggerKind::YourNextEndStep` (Desert Warfare). Its pods found ⚠ **a
Scute Swarm board doubling past the board cap** on a turn of extra land drops
(930 and 978 swarms); the bot now holds a hand land drop whose landfall
fan-out would pass `BOARD_GATE` (`landfall_token_estimate`) — lands put onto
the battlefield by effects still fire landfall and still reach the cap once
in 1,000 (open queue). Residuals: seven (INCOMPLETE_CARDS). Four-seat pods
beside Olivia / Jirina / Galea (seed 10420) and Nelly / Eshki / Estrid (seed
10421), 1,000 games each: 999 decided (one board cap each), Omo 24.7 % and
17.3 %; beside Chishiro / Zurgo / Otrimi (seed 10423): 999 — one
Chishiro no-progress draw; census: no card of the list unplayed. `--bench`
byte-identical; cube/sos/sealed (seed 10422): 7,500 decided.

The **hundred-and-sixteenth** is Outlaws of Thunder Junction Commander's
**Most Wanted** (`MostWanted_OTC`) — Mardu outlaws and Treasure under Olivia,
Opulent Outlaw, `--pod-decks 116` (committed as 115 until Prismari Artistry
landed there). Eighteen cards were missing (`cmdr_olivia.rs`). The
primitives: `R::IsOutlaw` reads the layered type line on the battlefield (CR
613.1d — Vihaan's animated Treasure Assassins are outlaws to Olivia's trigger)
and counts a Kindred card off it; `CounterType::Hit` (Mari) —
⚠ **`Suspect` was missing from the client's exhaustive counter-label
matches**, so the client did not build; `Effect::RemoveCounter` reaches a
card in exile (CR 122.1); `SpendRestriction::OutlawSpellsOrAbilities`
(Discreet Retreat). Residuals: Back in Town chooses as it resolves; Dire
Fleet Ravager's losses are sequential; Vihaan's own grant reads printed types
(the layer machinery's `requirement_matches_card` keeps type leaves printed).
Four-seat pods beside Jirina / Zurgo / Aminatou (seed 10410, 1,000 games):
all decided, Olivia 26.9 %; beside Nelly / Galea / Chishiro (seed 10411,
1,000): 999 decided — one Chishiro / Nelly no-progress draw after Olivia was
out — Olivia 20.5 %; census: no card of the list unplayed. `--bench`
byte-identical; cube/sos/sealed (seed 10412): 7,500 decided.

The **hundred-and-eleventh** is Commander 2020's **Ruthless Regiment**
(`RuthlessRegiment_C20`) — Mardu Humans under Jirina Kudro, `--pod-decks 111`
(committed as 110 until Painbow landed there first). Nineteen cards were
missing (`cmdr_jirina.rs`). The primitives:
`StaticEffect::GraveyardMatchingEntersWithExtraCounters` (Dearly Departed, CR
614.1c — it rides the graveyard anthem lane and `chosen_type_etb_counter_specs`
reads it) and `GrantProtectionFromChosenCreatureType` (Riders of Gavony, CR
702.16). Its tests found ⚠ **a pod seat's as-enters creature-type naming always
named Demon** — the ask is driven off the stack, where only the headless
decider answered (Metallic Mimic, Cavern of Souls, Species Specialist); a
controller's own naming now takes the bot heuristic, and Riders' hostile naming
leads with the opponents' most common type. And **`TriggerObjectIsChosenType`
missed a dying token** (it had ceased to exist, CR 704.5d — it now reads the
died-card snapshot, CR 603.10a). Residuals: Sanctuary Blade chooses its colour
by a trigger; Odric's block choice holds for the turn. Four-seat pods beside
Galea / Kathril / Chishiro (seed 10400, 1,000 games) and Nelly / Eshki /
Estrid (seed 10401, 1,000): all decided, Jirina 28.4 % and 24.3 %; census
(seed 10401): no card of the list unplayed. `--bench` byte-identical;
cube/sos/sealed (seed 10402): 7,500 decided.

The **hundred-and-thirty-eighth** is Aetherdrift Commander's **Living Energy**
(`LivingEnergy_DRC`) — Temur energy and artifacts under Saheeli, Radiant
Creator, `--pod-decks 138`. Twenty-one cards were missing
(`cmdr_saheeli_radiant.rs`). The primitives: `AlternativeCost.energy_cost` and
`StaticEffect::EnergyAlternativeCostForFilter` (Nissa, Worldsoul Speaker —
CR 118.9, cast a permanent for eight {E}), `StaticEffect::
ArtifactTokenCreationAddsToken` (Stridehangar Automaton's extra Thopter per
artifact-token batch) and `Effect::DoublePlayerCounters` (Aetheric Amplifier).
⚠ **Its pods found a bot loop:** Whirler Virtuoso under Decoction Module,
Panharmonicon and Stridehangar nets +1 {E} per activation (a real infinite),
and `pick_energy_payoff` ran it to 980 Thopters and the board cap (seed 138
game 561). `board_is_saturated` (`server/renewal_guard.rs`) now stops both
token sinks once the seat has 60+ creatures with three times the opponents'
remaining life in power. Re-run (seed 138, 1,000 games each): 4 seats
(Sigarda / Judith / Hanna) 1,000 decided, Saheeli 23.7 %; 6 seats against
the Leinore / Sidar / Oloro / Prossh / Nalia precons 1,000 decided, 1.3 %;
8 seats (seats 4-10) 1,000 decided, 6.0 %. Residuals: Aetherflux Conduit's
free casts last the turn; Territorial Aetherkite / Rampaging Aetherhood pay
all their {E}; Saheeli's copy target is chosen as the trigger goes on the
stack. Census: no card of the list unplayed. `--bench` byte-identical.

The **hundred-and-thirtieth** is Commander 2013's **Power Hungry**
(`PowerHungry_C13`) — Jund tokens and sacrifice under Prossh, Skyraider of
Kher, `--pod-decks 130`. Twenty-one cards were missing (`cmdr_prossh.rs`). The
primitives: `StaticEffect::DoubleTokensEveryone` /
`DoublePlusOneCountersEveryone` (Primal Vigor — the existing doubler counts
read them for every player), `Effect::OwnersGainControlOfNontokens` (Brooding
Saurian) and `Effect::DamageEachCreatureOfChosenColor` (Sudden Demise), plus
`CounterType::Eyeball` and `CreatureType::Graveborn`. ⚠ **Its pods found a bot
loop: Tooth and Claw under Primal Vigor** traded two Carnivores for two
Carnivores 3,936 times in one turn (10 action caps in 3,000 games) — both the
priced sacrifice owner and the token sink took it; `renews_its_own_fodder`
(`server/renewal_guard.rs`) now refuses a sacrifice whose fodder is only
tokens the ability itself makes, and the re-run is 3,000 / 3,000 decided.
Residuals: Sudden Demise's color is the engine's; Night Soil reads your own
graveyard; Widespread Panic counts any effect shuffle; Capricious Efreet's
opposing targets are the auto-picker's. Prossh wins 1.5-3.1 % against modern
lists and 20.3 % against its C13 cohort (Derevi / Marath / Jeleva, seed
10534) — the list's age, not a fault. Census: no card of the list unplayed.
Debug strict pods (400 games): clean. `--bench` byte-identical.

The **hundred-and-twenty-sixth** is Commander 2013's **Eternal Bargain**
(`EternalBargain_C13`) — Esper lifegain and artifacts under Oloro, Ageless
Ascetic (already in `sets::cmdr`, its command-zone upkeep life CR 113.6b),
`--pod-decks 126` (committed as 125 while Riveteers Rampage landed there).
Twenty cards were missing (`cmdr_oloro.rs`). The primitives, each in its own
module under `game/effects`: `Effect::ExchangePower` (CR 701.10g, Serene
Master), `Effect::EachPlayerTakesCreatureOfNext` (Order of Succession — every
chooser takes the next living player's most valuable creature, the caster
turns the circle the way that nets it the most; left is the next seat) and
`Effect::LookTopFiveDigForLife` (Lim-Dûl's Vault). Residuals: Order of
Succession's and the Vault's choices are the engine's; no bot path picks
Springjack Pasture's X; Serene Master's exchanged power counts counters twice.
Pods (release, 3,000 games: 4 seats beside Sidar / Leinore / Sigarda, seed
10520; 6 seats, 10521; 8 seats, 10522): 2,995 decided — one draw is Famine
killing the last three players at once (CR 104.4a), four board caps are Desert
Bloom's Scute Swarm (TODO); Oloro 34.2 % at four seats. Census: no card of the
four lists unplayed. Debug strict pods (400 games): clean. `--bench`
byte-identical.

The **hundred-and-twenty-first** is March of the Machine Commander's
**Cavalry Charge** (`CavalryCharge_MOC`) — Esper Knights under Sidar Jabari of
Zhalfir, `--pod-decks 121`. Nineteen cards were missing (`cmdr_sidar.rs`). The
primitives: `ActivatedAbility::tap_n_x` ("Tap X untapped Knights" — Aryel; with
no X named it is the target's power) and Haakon's Knights ride
`CastFromGraveyardMatching` (a duplicate `GraveyardCastFreely` landed first
and was folded in). Its finds: ⚠ **an Eminence "whenever you attack" never
fired from the command zone** (CR 113.6b — the you-attack walk read the
battlefield and graveyards only); ⚠ **cards drawn "this turn" reset only for
the active player**, so off-turn draw counts were stale (Elenda and Azor's end
step, second-card-each-turn triggers, Spirit of the Labyrinth's cap); ⚠ **the
bot's graveyard-grant gate opened only on the seat's own turn** although
Sarcophagus's and Haakon's grants work on any turn (a debug pod tripped the
gate's assertion). Residuals: **Path of the Enigma** holds no planar vote;
**Syr Elenora**'s power is a battlefield static; **Aryel**'s X is the
target's power. Pods (release, 3,000 games: 4 seats beside Leinore / Sigarda /
Teval, seed 10510; 6 seats, 10511; 8 seats, 10512): 2,998 decided, Sidar
48.1 % at four seats; the two board caps are Miracle Worker's Extravagant
Replication copying itself (below, TODO). Census: no card of the four lists
unplayed. Debug `CRAB_ANSWER_LOG=strict` pods (400 games, 4 and 6 seats):
clean after the gate fix. `--bench` byte-identical.

The **hundred-and-twentieth** is Innistrad: Midnight Hunt Commander's **Coven
Counters** (`CovenCounters_MIC`) — Selesnya +1/+1 counters and coven under
Leinore, Autumn Sovereign, `--pod-decks 120` (committed as 114, 115, 118 and 119
while six concurrent seats landed). Eighteen cards were missing
(`cmdr_leinore.rs`). The primitives: `Selector::OnePerDistinctPower` (Celestial
Judgment's survivors, Sigardian Zealot's pumped set — the controller's own
best per power, else an opponent's weakest), `StaticEffect::
MatchingLoseAllCreatureTypes` plus a `ControlledBy` seat on
`SetBasePtForFilter`, both resolved live (Curse of Conformity, CR 303.4a), and
`Selector::TakeWithSumCap` now spends its cap largest-first (Moorland
Rescuer). Residuals: a changeling keeps its types under Curse of Conformity;
the per-power picks and Moorland Rescuer's set are the engine's. Pods
(release, 5,000 games: 4 seats beside Sigarda / Teval / Jirina, seed 10500,
and Aminatou / Zurgo / Zaffai, seed 10501; 6 and 8 seats, seeds 10502-10503):
all decided, Leinore 37.2 % / 41.0 % at four seats; census: no card of the
four lists unplayed. Debug `CRAB_ANSWER_LOG=strict` pods (480 games, 4 and 6
seats): clean. `--bench` byte-identical.

The **hundred-and-twenty-eighth** is Ikoria Commander's **Timeless Wisdom**
(`TimelessWisdom_C20`) — Jeskai cycling under Gavi, Nest Warden,
`--pod-decks 128` (committed as 122, 123 and 127 while Omo, Yuma, Millicent, Henzie, Oloro and Quintorius landed). Nineteen cards were missing; eighteen landed here (`cmdr_gavi.rs` — Descend upon the Sinful came first with Desert Bloom). The
primitives: `StaticEffect::FirstCyclingEachTurnFree` (Gavi) and
`CyclingFreeWhileHandAtLeast` (New Perspectives), read where the cycling cost
is paid; `Player.cards_cycled_this_turn` / `Value::CardsCycledThisTurn`
(Spellpyre Phoenix's graveyard end-step return); Abandoned Sarcophagus's
`CastFromGraveyardMatching` (any turn, no budget) and
`ExileOwnCyclingCardsUnlessCycled` (a death-redirect-lane static; the cycle
path marks `Player.cycling_card` while it discards). The pod bot cycles more
freely when the cycle is free. ⚠ Glimmerpoint Stag returned the blinked
permanent under *its* controller with a +1/+1 counter (Semester's End's
return) — now its owner's, no counter. ⚠ Pods: Extravagant Replication's
"copy another permanent you control" picked a Replication token (the
own-side heuristic gives up the cheapest) and doubled the board to the cap —
the bot now passes over a token-copy replicator on a crowded board.
Residuals: Akim's "first time each turn" counts from its arrival; Crystalline
Resonance's copy lasts until it copies again; Ethereal Forager returns the
first linked instant or sorcery; Nimble Obstructionist's "you don't control"
reads the source permanent. Pods (release: 4 seats beside Sigarda / Hanna /
Jirina, seed 12200, 1,000 games, and Aminatou / Zurgo / Quintorius, seed
12201, 1,000 — and beside Kalamax 1,500 and Omo 1,000 while numbered 122/123;
6 seats, seed 12202, 600; 8 seats, seed 12203, 400): all decided, Gavi 11.7 % /
6.9 % at four seats. Suite at seat 128: 22,162 / 0 / 7; census: no card of the list unplayed.
`--bench` byte-identical; cube/sos/sealed (seed 12204): 62,500 decided.

The **hundred-and-fifth** is Secrets of Strixhaven Commander's **Silverquill
Influence** (`SilverquillInfluence_SOC`, 2026-04-24) — Orzhov Auras and goad
under Killian, Decisive Mentor, `--pod-decks 105` (committed as 103 while two
concurrent seats landed). Seventeen cards were
missing (`cmdr_killian.rs`). The primitives: `R::EnchantedByYourAura` (a
permanent with an Aura its viewer controls attached — Killian's draw,
Eriette's attack shield) and a card `filter` on
`Effect::RevealTopMayCastOneFree` (Herald of Amity casts only an Aura).
Residuals: **Armored Skyhunter** leaves an Equipment unattached; **Coercive
Impetus** renews its goad by a beginning-of-combat trigger; **Herald of
Amity** reveals rather than exiles (Intermediate Chirography's "modified"
became exact with Upgrades Unleashed's CR 603.10a look-back). Four-seat pods beside Sigarda / Teval / Disa (seed 11094,
1,000 games, all decided): Killian 36.1 %; census: no card of the four
unplayed; 6 seats (seed 11095) 1,000 / 1,000 decided. 8 seats (seed 11095)
board-capped 2 / 1,000 on Trostani's Storm Herd cast at 3,672 and 4,481 life
(1,025 Pegasi each); ⚠ the bot now holds a token spell that would carry the
battlefield past `MAX_BATTLEFIELD`, and the rerun is 1,000 / 1,000 decided.
`--bench` byte-identical.

The **ninety-ninth** is Commander 2017's **Draconic Domination**
(`DraconicDomination_C17`) — five-color Dragons under The Ur-Dragon (committed
as 92, 97 and 98 while seven concurrent seats landed; Territorial Hellkite
landed twice and Temur Roar's exact version was kept). Eighteen cards were
missing; the primitives: `Player::attacked_players_this_turn` +
`R::PlayerAttackedYouLastTurn` (O-Kagachi) and
`Effect::EachPlayerSparesOneTheyDontControl` (Fortunate Few). It found four
engine/card bugs: ⚠ **`Predicate::EntityMatches` on a player answered only a
bare `R::Player`** (every other player atom read false); **a trigger's
auto-target never had "that player" off a became-target event** (the actor
wasn't read — Scalelord Reckoner); ⚠ **Niv-Mizzet, Dracogenius drew when it
was dealt damage**, not when it dealt damage to a player; and **two Bold
Plagiarists fed each other to a million-action cap** (the counter event carries
no actor). Residual: **Orator of Ojutai** reads board and hand instead of a
reveal. Four-seat pods beside Sigarda / Teval / Disa (seed 11092, 1,000 games,
all decided): The Ur-Dragon 25.1 %; census: no card of the four unplayed; 6
and 8 seats (seed 11093) 2,000 / 2,000 decided. `--bench` byte-identical.

The **hundred-and-eighteenth** is Duskmourn Commander's **Miracle Worker**
(`MiracleWorker_DSC`) — Esper enchantments and miracles under Aminatou, Veil
Piercer, `--pod-decks 118` (committed as 116 while two concurrent seats
landed). Sixteen cards were missing (`cmdr_veilpiercer.rs`; Life Insurance
landed first in Most Wanted). The primitives: `LookPick::rest_on_top`
(Diabolic Vision), `Effect::GrantMiracleReduced` (Aminatou's "mana cost
reduced by {4}", CR 702.94 — granted as the turn's first card is drawn),
`StaticEffect::OpponentsStunCountersStay` (Fear of Sleep Paralysis, on the
untap-static lane) and `StaticEffect::ZeroAlternativeCostOncePerYourTurn` (One
with the Multiverse). Its bot find: ⚠ **the {0} alternative-cost grants were
invisible to the bot** (`grants_alt_cost` listed Kentaro and Demon of Fate's
Design but not Darksteel Monolith). Residuals: six, INCOMPLETE_CARDS. Four-seat
pods beside Zurgo / Chishiro / Olivia (seeds 10341-10342, 2,000 games): all
decided, Aminatou 9.6 % and 11.0 %; beside Estrid / Aminatou the Fateshifter /
Kalamax (seed 10343, 1,000): 999 decided — ⚠ **one board cap: Extravagant
Replication copying itself each upkeep** (213 copies, 770 Signets by turn 73; a
legal explosion, the Storm Herd class); census: no card of the Miracle Worker
list unplayed. `--bench` byte-identical; cube/sos/sealed (seed 10340): 7,500
decided.

The **hundred-and-thirteenth** is Tarkir: Dragonstorm Commander's **Mardu
Surge** (`MarduSurge_TDC`) — Mardu tokens and attack triggers under Zurgo
Stormrender, `--pod-decks 113` (committed as 112 while a concurrent seat
landed). Nineteen cards were missing (`cmdr_zurgo.rs`). The primitives:
`StaticEffect::CreatureTokensBecome` (Divine Visitation, CR 614.1a — mandatory,
at the mint), `Effect::DoubleTokensThisTurn` (Kaya's −2),
`Value::TokensCreatedThisTurn` (Thalisse), `Value::CreaturesAttackingPlayer`
(Within Range), `MayPlayDuration::TurnsHolderAttacksWithACommander` (Neriv —
Neyali's token grant's sibling) and `ActivatedAbility::discard_cost_x` (Gix).
Its tests found ⚠ **Mobilize sacrificed its Warriors at end of combat** where
CR 702.181a says the next end step (the shortcut's doc called the two
equivalent; War Effort and Dalkovan Encampment shared it) — Zurgo's "draw if it
was attacking, otherwise drain" reads the difference. Residual: Gix's exiled
cards stay playable for the turn. ⚠ Bot gap: no generator picks an X for Gix's
"Discard X cards:" activation. Four-seat pods beside Chishiro / Jirina / Eshki
(seeds 10331-10332, 2,000 games) and Kasla / Estrid / Nelly (seed 10333,
1,000): all decided, Zurgo 29.5 % and 28.9 %; census: no card of the four
unplayed. `--bench` byte-identical; cube/sos/sealed (seed 10330): 7,500
decided.

The **hundred-and-twelfth** is Commander 2021's **Prismari Performance**
(`PrismariPerformance_C21`) — Izzet big spells and magecraft under Zaffai,
Thunder Conductor, reached by `--pod-decks 112` (110, then 111, before
rebasing over Painbow and Ruthless Regiment). Eighteen cards were missing
(`cmdr_zaffai.rs`). The primitives: `SpendRestriction::RedInstantSorceryCopy`
(Pyromancer's Goggles — a rider whose pips each push a copy trigger above the
red instant or sorcery they funded, through the commander-mana rider hook),
`SpendRestriction::XCostsOnly` (Elementalist's Palette),
`Effect::CopySpellForEachOtherLegalTarget` (Radiant Performer,
`effects/copy_each_target.rs`) and `DynamicPt::ExiledWithSourceManaValue`
(Living Lore). ⚠ **An emblem's `GraveyardInstantsSorceriesHaveFlashback` was
never read** — the grant scanned the battlefield only (CR 114.4; Jaya
Ballard's −8). ⚠ **A trigger targeting "a spell with a single target" found
no target**: the auto-picker walked the stack only for counter-class filters.
Residuals: **Apex of Power**'s exiled cards may be played as lands; **Dazzling
Sphinx** leaves an uncast find in exile; **Muse Vortex** bottoms in exile
order; **Radiant Performer** copies spells only; **Zaffai** triggers once per
copy event. Pods (release, seed 10331, 1,000 games beside Firkraag /
Vrondiss / Jared): 1,000/1,000 decided, no card of the four lists unplayed,
Zaffai 12.1 %; 8 seats (seed 10332) 500/500 decided. `--bench`
byte-identical.

The **hundred-and-seventh** is Commander Legends: Battle for Baldur's Gate's
**Draconic Dissent** (`DraconicDissent_CLB`) — Izzet goad and Dragons under
Firkraag, Cunning Instigator, reached by `--pod-decks 107`. Eighteen cards
were missing; Psychic Impetus landed with Aura of Courage meanwhile, so
seventeen are this seat's (`cmdr_firkraag.rs`). The primitives: a fourth goad
source, board-wide statics read by `GameState::goaders` —
`OpponentCreaturesWithLesserPowerAreGoaded` (Baeloth) and
`OthersNamedLikeThisAreGoaded` (Mocking Doppelganger's copy rider), compared
on each instance's own power so goad never re-enters the layer system;
`OpponentsGoadedCreaturesCantBlock` (Bothersome Quasit, behind Void
Winnower's block-lock presence gate); `Effect::GoadACreatureOfEachOpponentAttackedBy`
(`effects/goad_attacked.rs`); `Effect::SpellDiscountUntilYourNextTurn` (Will
Kenrith's −2, a discount-form `TurnScopedSpellTax`); `CreatureType::Beholder`.
Residuals: **Baeloth** compares powers without static anthems; **Firkraag**'s
goaded creature is the engine's pick and "had to attack" reads as goaded or
must-attack; **Rowan Kenrith**'s +2 lasts until your next turn and reaches
the creatures the target has at resolution; **Stuffy Doll**'s player is the
most hostile opponent. Pods (release, seed 10321, 1,000 games beside
Vrondiss / Galea / Killian): 1,000/1,000 decided, no card of the four lists
unplayed, Firkraag 29.5 %; 8 seats (seed 10322) 500/500 decided. `--bench`
byte-identical.

The **hundred-and-fourth** is Adventures in the Forgotten Realms Commander's
**Draconic Rage** (`DraconicRage_AFC`) — Gruul dragons and dice under Vrondiss,
Rage of Ancients, reached by `--pod-decks 104` (103 before rebasing over
Symbiotic Swarm). Eighteen cards were missing
(`cmdr_vrondiss.rs`). The primitives: `RollDie.ignore_lowest` (CR 706.6,
Berserker's Frenzy's "roll two d20 and ignore the lower roll"),
`cast_only_before_blockers_step`, `Effect::ChooseBlocksThisTurn` (Master
Warcraft's block half alone), and in `effects/dice_choices.rs`
`Effect::RollTwoDiceAssign` (Wild Endeavor — both faces are fixed before the
controller's pick, so a suspended choice replays them),
`Effect::EachPlayerRollsSourceCantAttackHighest` (Chaos Dragon) and
`Effect::AddManaKeptThisTurnAnyColors` (Klauth); also
`PlayerRef::RandomOtherOpponentThanEnchanted` and `CounterType::Component`.
⚠ **`Effect::Attach` to an empty selection detached the attachment** — a
Maddening Hex with no other opponent fell off; it now stays (CR 701.3b).
Residuals: **Berserker's Frenzy**'s 1–14 creatures are every opposing one;
**Component Pouch**'s two colors may match; **Dragonborn Champion** ignores
damage to you; **Druid of Purification** has no "may" and starts with the next
player; **Klauth**'s mana isn't spell-only; **Sword of Hours** rolls per
recipient. Pods (release, seed 10311, 1,000 games beside Kitt / Go-Shintai /
Jeleva): 1,000/1,000 decided, no card of the four lists unplayed, Vrondiss
35.8 %; 8 seats (seed 10312) 500/500 decided. `--bench` byte-identical.

The **ninety-eighth** is Streets of New Capenna Commander's **Cabaretti
Cacophony** (`CabarettiCacophony_NCC`) — Naya Citizens, alliance and goad under
Kitt Kanto, Mayhem Diva, reached by `--pod-decks 98` (seat 96 before rebasing
over two others). Sixteen cards were missing (`cmdr_kitt.rs`). The
primitives: `Effect::EachOpponentChooses` (`effects/politics.rs` — every
living opponent picks in turn order, then each pick's body runs with
`PlayerRef::CurrentVoter` bound to its chooser; Master of Ceremonies, Seize
the Spotlight; not a CR 701.38 vote) and `R::BasePowerToughnessIs` (Bess).
The structural audit caught Killer Service nesting `MaySacrifice` inside
`MayPay` (the inner ask replays the outer answer); it pays, then sacrifices
the engine's pick. Residuals: **Excess** counts your creatures that damaged a
player; **Killer Service**'s token is the engine's pick; **Sizzling
Soloist**'s must-attack runs until your next turn; **Vivien's Stampede**
draws at end of combat; **Zurzoth**'s loot reaches one defender per batch.
Pods (release, seed 10261, 1,000 games beside Go-Shintai / Jeleva / Winter):
1,000/1,000 decided, no card of the four lists unplayed, Kitt 16.8 %.
`--bench` byte-identical.

The **ninety-second** is the Secret Lair **20 Ways to Win** (`20WaysToWin_SLD`)
— alternate win conditions, Gates and Shrines under Go-Shintai of Life's
Origin, reached by `--pod-decks 92`. Seventeen cards were missing; Tragic
Arrogance landed with Silverquill Statement meanwhile, so sixteen are this
seat's (`cmdr_goshintai.rs`). The primitives:
`StaticEffect::MatchingEnterUntapped` (Gond Gate — `enters_untapped_override`
is now the one card-aware check on both enters-tapped paths),
`Predicate::ControlsSameNamedAtLeast` (Mechanized Production),
`Value::CardTypesAmongPermanentsAndGraveyard` (Happily Ever After) and
`Effect::SacrificeAllButOnePerTypeYouChoose` — Tragic Arrogance's caster now
keeps its own best of each type and leaves each opponent the weakest (it kept
everyone's best). Residual: **Gond Gate**'s second ability makes any color.
Pods (release, seed 10251, 1,000 games beside Jeleva / Winter / Faldorn):
1,000/1,000 decided, no card of the four lists unplayed, Go-Shintai 15.5 %
(Winter 60.8 % — its end-step reanimation is the strongest engine in these
fields). `--bench` byte-identical.

The **eighty-ninth** is Commander 2013's **Mind Seize** (`MindSeize_C13`) —
Grixis spells, theft and wheels under Jeleva, Nephalia's Scourge, reached by
`--pod-decks 89` (seat 83, then 87 and 88, before rebasing over six others).
Eighteen cards were missing; Curse of Chaos and Terra Ravager landed with
Nature of the Beast meanwhile, so sixteen are this seat's (`cmdr_jeleva.rs`).
The primitives: `Keyword::ProtectionFromChosenPlayer` (True-Name Nemesis —
the chosen player's spells, abilities, blockers and damage at all four
protection sites, CR 702.16), `Keyword::CantBeBlockedIfDefenderHasMostCreatures`
(Hooded Horror, CR 509.1b), `Keyword::SplitSecondIfKicked` (Molten Disaster,
CR 702.61), `StaticEffect::AllPlayersCostReduction` (Arcane Melee) and
`StaticEffect::AllPlayersNoMaximumHandSize` (Price of Knowledge — and Anvil
of Bogardan, whose "Players have no maximum hand size" only lifted its
controller's cap). Rebasing found Feral Appetite (Witherbloom Pestilence)
declaring its target only inside an `If`, so it resolved against no target;
the exile now declares the slot. Residuals: **True-Name Nemesis** protects
from the engine's most hostile opponent; **Eye of Doom**'s counters go where
the engine picks. Pods (release, seed 10241, 1,000 games beside Winter /
Faldorn / Anikthea): 1,000/1,000 decided, no card of the four lists unplayed,
Jeleva 7.2 %. `--bench` byte-identical.

The **seventy-seventh** is Duskmourn Commander's **Death Toll**
(`DeathToll_DSC`) — Golgari self-mill and delirium under Winter, Cynical
Opportunist, reached by `--pod-decks 77`. Eighteen cards were missing
(`cmdr_winter.rs`). The primitives: `EventSpec::batch_counts_card_types`
(Polluted Cistern — one "one or more cards" trigger whose amount is the card
types among the whole batch, CR 603.2c); `Effect::ExileTypeSpreadReturnPermanent`
(`effects/graveyard_spread.rs`, Winter); `Predicate::TwoShareAllCardTypes`
(Demonic Covenant); `StaticEffect::PlayFromLibraryTopBySacrificing` (Into
the Pit, CR 401.6 — spells only, tokens and cheap permanents offered as the
sacrifice first); `StaticEffect::MayPlayLandsFromGraveyardMatching`
(Titania's Forests). Rendmaw's "two or more card types" is an exact OR of
the printable pairs. ⚠ **Its first census hung**: game 364 (seed 10231,
beside Saheeli / Hazel / Faldorn) never left one bot probe — Chatterfang
re-counted every earlier token on each resumed piece of an Insatiable
Frugivore repeat, doubling the Squirrels per answer. Session `01XnuL2a`
landed the same fix first (`f0522648`), so this seat carries none. Residuals:
**Winter**'s exiled set is the engine's pick; **Cemetery Tampering** puts a
hidden land onto the battlefield rather than playing it; **Polluted Cistern**
counts milled cards only; **Demonic Covenant** draws on an attack at a
planeswalker too; **Into the Pit**'s sacrifice is paid as the cast
completes; **Old Stickfingers** reveals creature by creature. Pods (release,
seed 10231, 1,000 games beside Faldorn / Anikthea / Ranar): 1,000/1,000
decided, no card of the four lists unplayed, Winter 45.1 %. `--bench`
byte-identical.

The **seventy-sixth** is Commander 2018's **Exquisite Invention**
(`ExquisiteInvention_C18`) — Izzet artifacts under Saheeli, the Gifted, the
sixth planeswalker commander (seat 71 before rebasing over Hatsune Miku, Exit
from Exile, Undead Unleashed, Call for Backup and Squirreled
Away). Seventeen cards were missing; the primitives:
`Effect::CastCommanderWithoutPaying` (Geode Golem — the tax is still owed, CR
903.8), `Effect::NextSpellHasAffinityForArtifacts` (Saheeli's +1, counted at
cast, CR 702.41a), `Duration::UntilEndOfYourNextTurn` for layer effects and
control steals (Treasure Nabber, CR 611.2b) and
`LookTopPutMatchingOntoBattlefield::rest_to_graveyard` (Saheeli's Directive).
Residuals: **Brudiclad**'s model token is your greatest-power one; **Prototype
Portal** imprints the first artifact card in hand; **Tawnos, Urza's
Apprentice** copies as Strionic Resonator does (the target is the source
permanent, the copy keeps its targets).
Four-seat pods beside Ranar / Urza / Anikthea (seed 10160, 1,000 games, all
decided): Saheeli 11.2 %; a 300-game census (seed 10161) leaves no card of the
four unplayed; 12 seats (71..60 as numbered then, seed 10162): 200 / 200 decided. `--bench`
byte-identical.

The **hundred-and-fourteenth** is Commander 2018's **Subjective Reality**
(`SubjectiveReality_C18`) — Esper top-of-library under Aminatou, the Fateshifter,
the ninth planeswalker commander. Nineteen cards were missing; the primitives (new
module `game/fateshift.rs`): `Effect::ManifestTopAttachSource` (Cloudform /
Lightform — the printed enchantment *becomes* an Aura through a `temporary_copies`
definition swap, reverted as it leaves), `ExileSourceAndTopThenManifest` (Jeskai
Infiltrator), `ChooseTwoPlayersForSource` + `OtherChosenPlayerLosesLife` (Sower of
Discord), `RotateNonlandPermanents` (Aminatou's −6), `GrantFreeCastOnePerCardType`
(Aminatou's Augury), `CastTopFreeIfElseDraw` (Yennett), `PlayTopFreeElseExile`
(Djinn of Wishes), `Predicate::OpponentControlsAtLeastMoreLands` (Isolated
Watchtower). Found and fixed: `GrantKeywordWhileControllerControlsAtMost` evaluated
its filters with no source, so `IsSource` / `OtherThanSource` read vacuously; and in
the 12-seat gate **the bot shuffled Belt of Giant Strength between two 11/11s 11,365
times** (a base-P/T setter lowers an 11/11, so "equip the biggest" flipped every
tick) — moving an Equipment now needs a strictly stronger new host. Residuals:
**Aminatou's Augury** picks its free spells at resolution; **Portent** never
shuffles; **Primordial Mist**'s exile is its target, not its cost; **Sower of
Discord**'s pair is the two least-life opponents. Four-seat pods beside Chishiro /
Zhulodok / Firkraag (seed 10500, 1,000 games): Aminatou 22.7 %, 999 decided plus one
genuine draw; a 300-game census (seed 10501) leaves no card unplayed; 12 seats
(110..99, seed 10502) 199 decided plus one draw, no action cap. `--bench`
byte-identical.

The **hundred-and-eighth** is Commander Masters' **Eldrazi Unbound**
(`EldraziUnbound_CMM`) — colorless Eldrazi ramp under Zhulodok, Void Gorger.
Eighteen cards were missing. Engine: `StaticEffect::ZeroAlternativeCostOncePerTurn`
(Darksteel Monolith, CR 118.9), `SelectionRequirement::TargetsAPermanentYouControlMatching`
(Not of This World — a spell *or ability* aimed at your permanent),
`Predicate::ColorlessManaSpentAtLeast` (Desecrate Reality's adamant) and
`CounterType::Suspect`. Found in the gate: **Desecrate Reality was never cast
in 1,000 pods**. Every pod seat is `wants_ui`, so a cast leaving an "up to N" slot
empty suspends to ask for it, and `would_accept` reads a bot seat's replay suspension
as a rejection — any multi-target spell the bot under-filled was silently dropped in
pods. Fixed as a class: on a prompting seat the bot fills extra slots with the
engine's own `auto_extra_targets_for`; the engine's extra-slot prompt and the bot's
slot walk both honor `ForEachOpponentTarget` (CR 601.2c). Two regression tests. Residuals: **Abstruse Archaic** targets the
ability's source permanent; **Ugin's Mastery** turns up the first face-down
creature. Four-seat pods beside Estrid / Kasla / Ulalek (seed 10400, 1,000 games, all decided): Zhulodok 35.0 %; a 300-game census (seed 10401) leaves no card of the four unplayed; 12 seats (then 103..92, seed 10402) 200 / 200. `--bench` byte-identical.

The **hundredth** is Modern Horizons 3 Commander's **Eldrazi Incursion**
(`EldraziIncursion_M3C`) — colorless Eldrazi under Ulalek, Fused Atrocity.
Eighteen cards were missing. Engine: `ManaSymbol::ColorlessHybrid` ({C/W}, CR
107.4e — Ulalek's printed cost: mana value 1, {C} tried first in the hybrid
solver, its color counts for identity), a new module `game/eldrazi.rs`
(`Effect::EachPlayerChoosesColorExileOthers` — Selective Obliteration;
`CopyAllSpellsAndAbilitiesYouControl` — Ulalek, CR 707.10;
`CopyOnePerOpponentWithTotalStats` — Benthic Anomaly, CR 707.9),
`SelectionRequirement::BasePowerOrToughnessAtMost` (Angelic Aberration) and
`Predicate::CastSpellWasKickedWith` (Wastescape Battlemage). Found and fixed: the
shared `eldrazi_spawn_token` had lost its **Spawn** creature type. Residuals:
**Benthic Anomaly**'s choices are the engine's (each opponent's greatest power; the
copy of the greatest mana value); **Bismuth Mindrender**'s card is castable for life
until end of turn; **Selective Obliteration**'s colors are each player's most common;
**Twins of Discord**'s granted bloodthirst rides colorless creature spells cast.
Four-seat pods beside Eshki / Ellivere / Lathliss (seed 10300, 1,000 games, all
decided): Ulalek 5.7 %; a 300-game census (seed 10301) leaves no card of the four
unplayed; 12 seats (seed 10302) 200 / 200. `--bench` byte-identical.

The **ninety-first** is Commander Masters' **Planeswalker Party**
(`PlaneswalkerParty_CMM`) — Jeskai superfriends under Commodore Guff, the
eighth planeswalker commander. Seventeen cards were missing; the primitives
(new module `game/loyalty_copy.rs`): `StaticEffect::LoyaltyAbilitiesTwiceEachTurn`
(Oath of Teferi, CR 606.3), loyalty-ability copy grants —
`Effect::CopyNextLoyaltyAbility` (Jaya's Phoenix, Repeated Reverberation) and
`Effect::CopyLoyaltyAbilitiesOfChosenTypeThisTurn` (Leori), CR 707.10 —
`Effect::ChooseAttackDirectionUntilYourNextTurn` (Teyo, CR 508.1a, the Mystic
Barrier walk without a permanent), `StaticEffect::AttackTaxOnYourPlaneswalkers`
(Onakke Oathkeeper, CR 508.1g) and `Effect::ShuffleInThenCastFromTopFree` (Guff
Rewrites History). Residuals: **Chandra, Legacy of Fire**'s 0 takes one loyalty
from each walker with two or more; **Guff Rewrites History** picks opponents'
permanents only; **Leori**'s type is the most common one you have; **Narset of the
Ancient Way**'s −2 target is chosen on activation; **Repeated Reverberation**'s
three halves are separate riders; **Sparkshaper Visionary** turns all or none;
**Vronos**'s +1 phases out all your other walkers.
Four-seat pods beside Marath / Atarka / Dina (seed 10200, 1,000 games, all
decided): Guff 38.4 %; a 300-game census (seed 10201) leaves none of Guff's cards
unplayed (Marath's and Atarka's lists each leave one — theirs); 12 seats (89..78 at
the time, seed 10202): 199 decided and one genuine draw (game 53, CR 104.4a — game over, no
winner). `--bench` byte-identical.

The **eighty-fifth** is Dominaria United Commander's **Legends' Legacy**
(`LegendsLegacy_DMC`) — Mardu legends under Dihada, Binder of Wills, the
seventh planeswalker commander. Eighteen cards were missing; the primitives:
`GameEvent::AbilityActivated::life_paid` and
`EventKind::AbilityActivatedWithLifePaid` (Verrak, Warped Sengir, CR 602.2 +
119.4 — the event amount is the life paid), `StaticEffect::PlayersSkipExtraTurns`
(Gerrard's Hourglass Pendant, CR 614.10 — binds its controller too) and
`Value::GreatestManaValueExiledThisTurn` over
`GameState::greatest_exiled_mv_this_turn` (Bell Borca's noted mana values,
written by the library/graveyard exile funnel and the battlefield exile funnel).
Residuals: **Bell Borca** notes cards exiled before it entered too; **Bladewing**
doesn't trigger on damage to a planeswalker; **The Peregrine Dynamo** copies as
Strionic Resonator does; **Verrak** sees only fixed life costs.
Four-seat pods beside Gimbal / Mishra / Ms. Bumbleflower (seed 10190, 1,000 games, all decided): Dihada 31.1 %; a 300-game census (seed 10191) leaves no card unplayed; 12 seats (seed 10192): 200 / 200. `--bench` byte-identical.
The **eighty-fourth** is March of the Machine Commander's **Tinker Time**
(`TinkerTime_MOC`) — Temur artifact tokens under Gimbal, Gremlin Prodigy.
Sixteen cards were missing; the primitives: `StaticEffect::GrantImproviseToSpells`
(Inspiring Statuary, CR 702.126a — rides the convoke grant's cast gate, helper
picker and bot block) and `Effect::ExileTopPushingLuck` (Dance with Calamity).
⚠ **Found: `Selector::ExiledThisResolution` evaluated its filter with no X**, so
a "mana value X or less" filter matched nothing (CR 107.3; Rashmi and Ragavan).
Residuals: **Dance with Calamity** stops exiling at a total of nine rather than by
choice; **Path of the Animist**'s Will of the Planeswalkers vote does nothing
outside Planechase; **Pain Distributor** damages the artifact's owner;
**Gimbal**'s trample grant reads printed types.
Four-seat pods beside Mishra / Ms. Bumbleflower / Derevi (seed 10180, 1,000 games, all decided): Gimbal 29.0 %; a 300-game census (seed 10181) leaves no card of the four unplayed; 12 seats (seed 10192): 200 / 200. `--bench` byte-identical.
The **eighty-third** is The Brothers' War Commander's **Mishra's Burnished
Banner** (`MishraSBurnishedBanner_BRC`) — Grixis artifacts under Mishra,
Eminent One. Seventeen cards were missing; the primitives:
`GameEvent::AbilityActivated::sacrificed` and
`EventKind::AbilityActivatedWithSacrifice` (Ashnod the Uncaring, CR 602.2 +
118.3) and `EntersAsCopy::not_a_creature` (Machine God's Effigy, CR 707.9b).
⚠ **Found: a static grant's "artifact creatures" filter reads printed types**
(`affected_includes_gated`), so Workshop Elders' flying misses the artifact it
animates (CR 613.8) — ENGINE_BACKLOG. Residuals: **Mishra**'s Warform keeps the
artifact's name; **Ashnod** can't copy an ability whose source was the thing
sacrificed; **Blast-Furnace Hellkite** also counts attacks on planeswalkers;
**Smelting Vat** caps each card, not the pair's total; **Lithoform Engine**
copies abilities as Strionic Resonator does; **Workshop Elders** (above);
**Glint Raker**'s reveal isn't optional.
Four-seat pods beside Ms. Bumbleflower / Derevi / Lord Windgrace (seed 10170, 1,000 games, all decided): Mishra 14.6 %; a 300-game census (seed 10171) leaves no card of the four unplayed; 12 seats (85..74, seed 10192): 200 / 200 decided. `--bench` byte-identical. ⚠ The first gate, on a binary built before rebasing over the Insatiable Frugivore / Chatterfang loop fixes, ran 1h45m on one 1,000-game block without finishing; the rebuilt binary does it in 6.6 s — the hang was not this list's.
The **sixtieth** is the Starter Commander Decks' **Token Triumph**
(`TokenTriumph_SCD`) — Selesnya tokens and anthems under Emmara, Soul of the
Accord (seat 59 before rebasing over Endless Punishment). Fifteen cards were
missing; the primitive: `Keyword::CantBeBlockedByPowerLessThanGreatestAmong`
(Champion of Lambholt; its sibling `CantBeBlockedExceptByWhilePowerAtMost`
moved Sidar Kondo off a power-filtered grant the layers dropped whole — the
dropped-static ratchet caught it). ⚠ **The census found Devouring Light
castable by no bot path**: "attacking or blocking" is an `Or`, which the
combat-only window didn't read as combat-only. No residuals. Debug pods beside
the Mimeoplasm / Kynaios / Ghired / Sigarda (seeds 9321/9322, 60 games) decided
60/60, zero panics, Emmara winning 45 %; a 120-game census (seed 9323) leaves no
card unplayed (Emmara 38.3 % there). `--bench` byte-identical.

The **sixty-third** is Secret Lair's **Raining Cats and Dogs**
(`RainingCatsAndDogs_SLD`) — Naya Cats and Dogs under Rin and Seri,
Inseparable (seat 62 before rebasing over Mirror Mastery). Sixteen cards were
missing; the primitives: `StaticEffect::TokensMayBecome` (Jinnie Fay's CR
614.1a token replacement, applied per token at mint time) and
`Value::LandCardsRevealedThisEffect` (Phabine's parley splits lands from
nonlands). Mirror Entity landed in Open Hostility at the same time; one
definition stays. Residuals: **Highcliff Felidar** destroys one opponent at a
time and picks among ties itself; **Jinnie Fay** always takes a bigger body
and never replaces a noncreature token; **Pack Leader**'s shield covers the
Dogs present as it resolves; **Showdown of the Skalds** II/III choose the
counter's target on resolution. A debug pod beside Nahiri / Tegwyll /
N'ghathrod (seed 9501, 30 games) decided 30/30, zero panics; a 120-game census
beside Ghired / Kynaios / Emmara (seed 9502) decided 120/120 and leaves no card
of the four lists unplayed, Rin and Seri winning 24.2 %.

The **sixty-seventh** is March of the Machine Commander's **Growing Threat**
(`GrowingThreat_MOC`) — Orzhov Phyrexians, incubate and proliferate under
Brimaz, Blight of Oreskos (seat 65, then 66, before rebasing over First
Flight and Arcane Wizardry). Seventeen cards were missing; the primitives:
`Effect::RollPlanarDie` / `Planeswalk` / `ChaosEnsues` and
`StaticEffect::ExtraPlanarDie` (`game/effects/planar.rs` — Fractured
Powerstone, Path of the Schemer's will of the planeswalkers, Ichor Elixir,
whose extra die now reaches the special action too). ⚠ **The first census
found Brimaz winning 1 of 60**: no bot path activated an Incubator's "{2}:
Transform", so every incubated token stayed an inert artifact.
`server/transform_sink.rs` flips one whose back face would survive; Brimaz
went to 5 of 60 on the same seed, and 12.0 % over 200 games beside Ghired /
Riku / Adriana (seed 9512, all decided, no card of the four lists unplayed).
Residuals: **Cataclysmic Gearhulk** keeps the highest mana value of each
type; **Filigree Vector** counters everything you control; **Path of the
Schemer** takes the greatest-power creature card; **Vulpine Harvester** checks
the mana value on resolution. `--bench` byte-identical.

The **seventy-fourth** is March of the Machine Commander's **Call for
Backup** (`CallForBackup_MOC`) — Naya +1/+1 counters and backup under
Bright-Palm, Soul Awakener (seat 73 before rebasing over Undead Unleashed).
Sixteen cards were missing, and three engine finds came with them:
⚠ **a multi-mode cast checked every target against mode 0** (CR 700.2 —
Dromoka's Command choosing the enchantment sacrifice and the counter was
refused; `game/spree_targets.rs`); **LookPick's "mana value X or less" never
read the resolving X** (CR 107.3 — Emergent Woodwurm's `WithX` power); and
**auto-targeting doubled the wrong creature's counters** (the highest-power
friendly creature, and never the trigger's own source — Bright-Palm doubled a
counterless body). Residuals: **Dromoka's Command** and **Inscription of
Abundance** fight the greatest-power creature you don't control, and the
Inscription is never kicked. A 200-game census beside Brimaz / Rin and Seri /
Ghired (seed 9521) decided 200/200 with zero panics, Bright-Palm winning 9.5 %;
Semester's End was the one card never cast (it is a sweeper shield and no
sweeper was cast).

The **seventy-fifth** is Bloomburrow Commander's **Squirreled Away**
(`SquirreledAway_BLC`) — Golgari Squirrels, Food and tokens under Hazel of the
Rootbloom. Seventeen cards were missing; one primitive, `R::BasePowerOrToughnessIs`
(Sword of the Squeak). ⚠⚠ **Its first 1,000-game pod hung** (seed 9531, game 158):
a bot lookahead through Insatiable Frugivore's repeat resumed the same
resolution per answer, and **Chatterfang's rider re-counted every token the
resolution had minted — its own Squirrels included — on each resume**, doubling
the board per answer (CR 614.13). The riders now count only the tokens each
resumed segment mints; a prompting-seat test pins it (it hangs without the
fix). Frugivore's first repeat was also taken unasked (`MayRepeat` runs its
first pass unconditionally) — a `MayDo` asks now. Residuals: **Hazel**'s "tap X
tokens" taps every other untapped token; **Hazel's Brewmaster** lends the
abilities of every exiled card, not only creatures; **Sword of the Squeak**
reads printed power and toughness; **Frugivore** exiles the graveyard's first
three. Release pods beside Ranar / Saskia / Valgavoth (seed 9531, 1,000 games):
1,000 decided, Hazel 14.5 %; a 300-game census (seed 9532) leaves no card of
Hazel's list unplayed; strict debug pods (seeds 9533/9534, 120 games) decided
120/120. `--bench` byte-identical.

The **eighty-first** is Commander 2013's **Evasive Maneuvers**
(`EvasiveManeuvers_C13`) — Bant tempo, curses and tap/untap tricks under Derevi,
Empyrial Tactician. Nineteen cards were missing; the primitives:
`LibraryPosition::BeneathTopX` (Unexpectedly Absent — `FromTop` with the
resolving spell's X) and `Value::PlayersWithLandsAtLeastMore` (Surveyor's
Scope); Hada Spy Patrol rides `level_bands`. ⚠ **`move_card_to` never looked in
the command zone**, so Derevi's "put Derevi onto the battlefield from the command
zone" did nothing; a card may now move *itself* out of it — a broader search let
a flicker's delayed return pull a commander home again (CR 400.7), which the
committed seeded-pod table caught. ⚠ **The bot cursed itself**: "enchant player"
Auras aimed their player slot at the caster; `Attach` now marks the slot
hostile. Residual: Curse of Inertia's tap-or-untap is the engine's pick. Release
pods beside Hazel / Ranar / Saskia (seed 9541, 1,000 games): 1,000 decided —
Derevi 3.0 % (4.5 % beside Teferi / Kalemne / Zedruu, seed 9545: a tempo list the
bot pilots poorly); a 300-game census (seed 9542) leaves no card unplayed;
strict debug pods (seeds 9543/9544, 120 games) decided 120/120. `--bench`
byte-identical.

The **ninety-seventh** is Tarkir: Dragonstorm Commander's **Temur Roar**
(`TemurRoar_TDC`) — Temur Dragons under Eshki, Temur's Roar. Thirteen cards
were missing; the primitive is `Effect::ChooseRandomOpponentNotAttackedLastCombat`
(Territorial Hellkite): `CardCold::combat_defenders` is armed by the trigger and
filled at declare attackers, so only that creature pays. Its tests found ⚠ **a
battlefield cast trigger picked its target before the cast spell's mana value
was in scope** — Skyfire Kirin's and Hammerhead Tyrant's "mana value equal to /
up to that spell's" filters never matched at targeting. Residuals: Deceptive
Frostkite's copy isn't optional; Will of the Temur reads "as you cast" at
resolution. Four-seat pods beside Ellivere / Lathliss / Zinnia (seed 10230,
1,000 games, all decided): Eshki 13.5 %; beside Nelly / Go-Shintai / Guff (seed
10231): 1,000 / 1,000, 19.0 %; census (seed 10232): no card of the four
unplayed; strict debug pods (seeds 10233-10235, 4 and 6 seats): 180 / 180.
`--bench` byte-identical; cube/sos/sealed (seed 10236): 7,500 decided.

The **hundred-and-second** is Commander 2018's **Adaptive Enchantment**
(`AdaptiveEnchantment_C18`) — Bant enchantments under Estrid, the Masked, a
planeswalker commander. Eighteen cards were missing; the primitives: **CR 903.9's
`CommanderPutIntoCommandZone` event**, emitted by both routes home (the 903.9a
SBA and the 903.9b replacement; Myth Unbound, whose discount rides
`CostReductionByValue` over `CommanderCastsFromCommandZone`),
`CostReductionForYourSpellsTargetingThis` (Elderwood Scion),
`ExileDyingOpponentCreaturesGrowingThis` (Ravenous Slime),
`SpellsOfTypeCastThisTurnAtMost` (Tuvasa), `AttachAnyNumberTo` (Bruna, Heavenly
Blademaster), `CounterType::Slumber` and `PlaneswalkerSubtype::Estrid`. Its pods
found ⚠ **two Scalelord Reckoners stacking 3,905 triggers**, each retargeting
the other player's Dragon: the bot's hostile pick now skips a permanent its
own stack items already aim at. Residuals: Estrid's −7 Auras land on
engine-picked hosts; Genesis Storm always puts the card in; Myth Unbound counts
partners' casts together. Four-seat pods beside Kasla / Ulalek / The Ur-Dragon
(seed 10240, 1,000 games, all decided after the fix): Estrid 20.2 %; beside
Eshki / Nelly / Go-Shintai (seed 10241): 1,000 / 1,000, 15.3 %; census (seed
10242): no card of the four unplayed; strict debug pods (seeds 10244-10246, 4
and 6 seats): 180 / 180. `--bench` byte-identical; cube/sos/sealed (seed 10243):
7,500 decided.

The **hundred-and-twenty-ninth** is Outlaws of Thunder Junction
Commander's **Grand Larceny** (`GrandLarceny_OTC`) — Sultai theft under
Gonti, Canny Acquisitor, `--pod-decks 129` (committed as 128 until Timeless
Wisdom landed there first). Eighteen cards were missing (`cmdr_gonti.rs`);
the primitives: `SpendRestriction::SpellsYouDontOwn` + `SpellKind::not_owned`
(Thieving Varmint, CR 106.6), `StaticEffect::SpellsYouDontOwnCostLess`
(Gonti), `StaticEffect::DoubleControllerCombatDamageToPlayerTriggers` (Felix
Five-Boots — one more fire at the combat-damage push), and
`game/effects/theft.rs`: `ExileSpellLinked` (Smirking Spelljacker — exiled,
not countered, so an uncounterable spell goes too; a spell copy just ceases),
`LookTopExileOneFaceDownMayPlay` (Thief of Sanity, Siphon Insight),
`OpponentChoosesXFromHandCastOneFree` (Extract Brain),
`ExileTopMayCastFreeIfNonland` (Mind's Dilation),
`ExileTopOfEachLibraryMayPlayForLife` (Nashi) and
`ManifestTopOfLibraryUnderYou` (CR 701.34 — Thieving Amalgam, Orochi
Soul-Reaver: the card's owner and the manifesting player apart). Residuals:
**Bladegriff Prototype**, **Extract Brain**, **Nashi**, **Siphon Insight**,
**Thief of Sanity** (INCOMPLETE_CARDS). Release pods beside Henzie / Otrimi /
Kalamax (seed 14001, 1,000 games): 1,000 decided, Gonti 19.5 %, no card of
the four lists unplayed. `--bench` byte-identical.

The **hundred-and-twenty-fifth** is Streets of New Capenna Commander's
**Riveteers Rampage** (`RiveteersRampage_NCC`) — Jund blitz under Henzie
"Toolbox" Torre, `--pod-decks 125` (committed as 123 and 124 while Desert
Bloom and Spirit Squadron landed first; its World Shaper is Desert Bloom's).
Nineteen cards were missing (`cmdr_henzie.rs`); the primitives:
`StaticEffect::GrantBlitzToSpells` + `BlitzCostLessPerCommanderCast` (Henzie,
CR 702.152 — the spell's own mana cost as its blitz, {1} less per commander
cast, printed blitz costs discounted too; the bot's alt-cost facts see the
grant), `StaticEffect::TreasureCreationAddsTreasure` (Jolene),
`StaticEffect::CanAttackPlayersWhoAttackedYouLastTurn` (Weathered Sentinels,
CR 508.1a — Defender waived per defending player, checked per attack; the bot
re-aims it or leaves it home), `Effect::ExileOnePerCardTypeFromGraveyardGrow`
(Grime Gorger, CR 205.2a) and `CounterType::Contested` +
`ContestOneLandPerPlayer` / `TakeContestedLand` (Turf War). ⚠
`R::DamagedAPlayerThisTurn` read only the battlefield, so Wave of Rats' own
death trigger never saw its hit — it reads the death snapshot now (CR
603.10). Two base-branch ratchets were red from another seat and fixed here:
`for_each_inner` didn't recurse `EachPushesTrigger`, and the reanimation
audit read a `DelayUntilWithCapture` return as an unzoned target. Residuals:
**Henzie**, **First Responder**, **Mezzio Mugger**, **Next of Kin**,
**Protection Racket**, **The Beamtown Bullies**, **Turf War**
(INCOMPLETE_CARDS). Release pods beside Millicent / Yuma / Otrimi (seed
13001, 1,000 games): 999 decided (the one board cap is Yuma's Scute Swarm
runaway, already open), Henzie 18.2 %, no card of the four lists unplayed.
`--bench` byte-identical.

The **hundred-and-seventeenth** is Ikoria Commander's **Arcane Maelstrom**
(`ArcaneMaelstrom_C20`) — Temur instants and copies under Kalamax, the
Stormsire, `--pod-decks 117` (committed as 116 until Most Wanted landed
there first). Eighteen cards were missing (`cmdr_kalamax.rs`); the
primitives: `StaticEffect::SpellCopiesPlusOne` (Twinning Staff, CR 707.10 —
one more copy per static, for the copies' controller),
`StaticEffect::GrantConspireToSpells` + `GameState::spell_has_conspire` (Wort,
the Raidmother, CR 702.78 — the cast path, the affordance and the bot's
conspire block all read it), `Effect::RevealTopCastFreeIfLesserElseHand`
(Rashmi) and `RevealUntilCreatureBecomeCopy` (Nascent Metamorph) in
`game/effects/reveal_top_misc.rs`, and
`Predicate::AnOpponentHadCreaturesEnterAtLeast` (Whiplash Trap's alternative
cost). Curious Herd names its target opponent with a zero-card draw (the
target-walker ratchet caught a bare slot). ⚠ **The bot never cast a pure
retarget spell** (Deflecting Swat unplayed in 1,000 pods): it now answers an
opponent's single-target spell aimed at its board or face with a held
Redirect-shaped instant (the free commander cost first), kept only when a
clone shows the retarget pointing the spell away (`pick_retarget_shield`).
Residuals: **Eon Frolicker**, **Haldan** / **Pako**, **Lavabrink Floodgates**
(INCOMPLETE_CARDS). Release pods beside Olivia / Jared / Kathril (seed 7,
1,000 games): 1,000 decided, Kalamax 4.2 %, and the census left only
Deflecting Swat unplayed — the find above; after it, beside Otrimi /
Kathril / Jared (seed 12001), every card of the four lists is played.

The **hundred-and-nineteenth** is Ikoria Commander's **Enhanced Evolution**
(`EnhancedEvolution_C20`) — Sultai mutate under Otrimi, the Ever-Playful,
`--pod-decks 119` (committed as 118 until Miracle Worker landed there
first). Nineteen cards were missing (`cmdr_otrimi.rs`); Otrimi, Mindleecher,
Pouncing Shoreshark and Souvenir Snatcher ride the existing mutate path (CR
702.140), Cazur and Ukkima partner with each other, and Nissa, Steward of
Elements' X loyalty (CR 306.5b) is `enters_with_counters`. The primitives
(`game/effects/library_top_deploy.rs`):
`Effect::RevealTopPutLandsRestBottomRandom` (Animist's Awakening — spell
mastery, CR 207.2c, read as it resolves), `LookTopMayPutLandOrCreatureMvAtMost`
(Nissa's 0, its "you may" through `MayDo`), and
`StaticEffect::HasActivatedAbilitiesOfBattlefieldLands` (Manascape
Refractor, the battlefield sibling of Mirran Safehouse). Residuals:
**Capricopian**, **Manascape Refractor**, **Mindleecher**, **Vastwood Hydra**
(INCOMPLETE_CARDS). Release pods beside Kalamax / Kathril / Jared (seed
12001, 1,000 games): 1,000 decided, Otrimi 23.2 %, no card of the four lists
unplayed; strict debug pods beside Olivia / Aminatou (seeds 12003/12004, 120
games) decided 120/120. `--bench` byte-identical.

The **hundred-and-tenth** is Dominaria United Commander's **Painbow**
(`Painbow_DMC`) — five-color multicolor under Jared Carthalion, a
planeswalker commander (CR 903.3a). Eighteen cards were missing; the
primitives: `R::AllColors` (CR 105.2c — the Kavu tokens Iridian Maelstrom
spares), `PumpTeamByControlledPermanents::per_own_color` (Knight of New
Alara), `StaticEffect::OthersEnterWithSourceTapState` (Archelos, CR 614.1c),
`PlaneswalkerSubtype::Jared`. Three finds: ⚠ **a "whenever you cast a spell
with {X}" trigger read X as 0** — the cast-trigger push carried no X, so
Geometer's Arthropod looked at nothing and Zaxara's Hydra died 0/0; ⚠ **a
static over "multicolored creatures you control" was dropped whole**
(Rienne) — `Multicolored`/`Monocolored` now route like `HasColor`; ⚠ **the
bot's mana-value gate rejected affordable spells** under next-spell affinity
or a turn-granted discount (a strict-pod `debug_assert`). Fallaji Wayfarer's
all-colors CDA is exempt from its identity, as printed (CR 903.4).
Residuals: **Primeval Spawn**, **Knight of New Alara**, **Unite the
Coalition** (INCOMPLETE_CARDS). Release pods beside Kathril / Kasla /
Ellivere (seed 11001, 1,000 games): 1,000 decided, Jared 10.8 %, and the
census leaves no card of the four lists unplayed; strict debug pods beside
the Chishiro / Zhulodok / Firkraag seats (seeds 11003/11004, 120 games)
decided 120/120. Suite 21,644 / 0; `--bench` byte-identical.

The **hundred-and-third** is Ikoria Commander's **Symbiotic Swarm**
(`SymbioticSwarm_C20`) — Abzan keyword counters under Kathril, Aspect
Warper. Nineteen cards were missing; the primitives
(`game/effects/keyword_gifts.rs`): `Effect::KeywordCountersFromGraveyard`
(Kathril), `GainKeywordsYourCreaturesHave` (Majestic Myriarch),
`RevealTopChooseByKeyword` (Selective Adaptation), `MoveCountersFromAmongOnto`
and `MoveOneCounter`; `ChooseCardTypeAmongForSource` +
`StaticEffect::NoOneCastsChosenCardType` (Archon of Valor's Reach, on Iona's
cast-lock lane); delve-linked cards and
`Value::CardsExiledWithSourceMatching` (Soulflayer, CR 702.66);
`Predicate::TriggerSourceHadCounters` (Nikara, CR 603.10 LKI). Cairn
Wanderer is one `PumpTeamIf` per keyword. ⚠ **An "up to N target cards"
slot named one graveyard card twice** (CR 115.3): Ever After was never cast
until the graveyard sweep skipped cards an earlier slot took. Residuals:
**Cairn Wanderer**, **Slippery Bogbonder**, **Tayam**, **Vitality Hunter**,
**Yannik**, **Archon of Valor's Reach** (INCOMPLETE_CARDS). Release pods
beside Kasla / Ellivere / Zinnia (seed 10302, 1,000 games): 1,000 decided,
Kathril 21.8 %, and the census leaves no card of the four lists unplayed;
strict debug pods (seeds 10303/10304, 120 games) decided 120/120. Suite
21,600 / 0; `--bench` byte-identical.

The **hundred-and-first** is March of the Machine Commander's **Divine
Convocation** (`DivineConvocation_MOC`) — Jeskai convoke under Kasla, the
Broken Halo. Nineteen cards were missing. ⚠ **Convoking tapped creatures
silently** (CR 702.51): the cast path set `tapped` with no `PermanentTapped`
event, so no "becomes tapped" trigger ever fired off a convoke — Fallowsage,
Mistmeadow Vanisher, Saint Traft and Rem Karolus and Wildfire Awakener's
Elementals are built around exactly that. The primitives:
`Effect::NextSpellGainsConvokeThisTurn` (Wand of the Worldsoul, Flockchaser
Phantom; the bot sees the grant), `R::HasConvoke` (printed, static- or
next-spell-granted — Kasla, Joyful Stormsculptor, Saint Traft),
`Selector::CreaturesThatConvokedSource` (Venerated Loxodon) and
`R::LoyaltyActivatedThisTurn` (Cut Short). Residuals: **Deluxe Dragster**,
**Path of the Ghosthunter**, **Joyful Stormsculptor** (INCOMPLETE_CARDS).
Release pods beside Ellivere / Zinnia / Atarka (seed 10101, 1,000 games):
1,000 decided, Kasla 13.5 %, and the census leaves no card of the four lists
unplayed; strict debug pods beside Ulalek / The Ur-Dragon / Kitt Kanto (seeds
10103/10104, 120 games) decided 120/120. Suite 21,567 / 0; `--bench`
byte-identical.

The **ninety-sixth** is Wilds of Eldraine Commander's **Virtue and Valor**
(`VirtueAndValor_WOC`) — Selesnya Auras and Roles under Ellivere of the Wild
Court. Nineteen cards were missing; the primitive family is **Aura and
Equipment cards put onto the battlefield attached** (CR 303.4f / 301.5c,
`game/effects/attach_from_zone.rs`): `Effect::PutOntoBattlefieldAttached`
(from graveyard and/or hand, to a named host or each to its own best legal
one; no legal host leaves the card where it is, CR 303.4i) and its library
forms `RevealTopPutAttached` / `RevealUntilPutAttachedElseHand`. Also
`Value::AttachmentsOn`, `R::AttachedToCreature`, the Virtuous Role and
`StaticEffect::AurasOnYourPermanentsHaveUmbraArmor`. Residuals: **Indomitable
Might**, **Mantle of the Ancients**, **Unfinished Business**, **Retether**,
**Knickknack Ouphe**, **Songbirds' Blessing**, **Liberated Livestock**
(INCOMPLETE_CARDS). Release pods beside Zinnia / Atarka / Derevi (seed 9601,
1,000 games): 1,000 decided, Ellivere 53.0 %, and the census leaves no card of
the four lists unplayed; strict debug pods beside Lathliss / Nelly /
Go-Shintai (seeds 9603/9604, 120 games) decided 120/120. `--bench`
byte-identical.

The **ninety-fourth** is Bloomburrow Commander's **Family Matters**
(`FamilyMatters_BLC`) — Jeskai tokens and offspring under Zinnia, Valley's
Voice. Nineteen cards were missing; the primitives:
`StaticEffect::CreatureSpellsGainOffspring` (CR 702.175 — the kicked cast
path pays a board-granted offspring cost, `R::PaidGrantedOffspring` marks the
permanent, the bot offers and prefers it), `NonHandCastCostReduction`
(Fortune Teller's Talent level 3), `Effect::GainControlWhileCounter` (Shield
Broker, CR 611.2c), `LookTopMayDeployAttacking`'s end-of-combat return
(Arthur), `R::BasePowerIs`, and `Keyword::UnblockableWhilePowerOrToughnessAtMost`
(Tetsuko — read at block declaration, since a P/T filter can't route through
the layers). ⚠ **A flickered or reanimated creature still read as kicked**
(CR 400.7): its kicker ETB and an offspring copy fired again; the new-object
reset now clears it. ⚠ `library_top_playable` ignored class-level and
condition wrappers. `JoinCombatAttacking` from a non-attacking source joins
the attack its target makes (Echoing Assault). Residuals: **Zinnia**,
**Echoing Assault**, **Combat Celebrant**, **Rose Room Treasurer**
(INCOMPLETE_CARDS). Release pods beside Atarka / Derevi / Hazel (seed 9401,
1,000 games): 1,000 decided, Zinnia 32.7 %, and the census leaves no card of
the four lists unplayed; strict debug pods beside Nelly / Go-Shintai / Guff
(seeds 9403/9404, 120 games) decided 120/120. `--bench` byte-identical.

The **eighty-seventh** is the Starter Commander Decks' **Draconic
Destruction** (`DraconicDestruction_SCD`) — Gruul Dragons under Atarka, World
Render. Nineteen cards were missing; the primitive is `EventKind::Fights` /
`GameEvent::CreatureFought` (CR 701.12 — Foe-Razer Regent's "whenever a creature
you control fights", emitted per fighter by the Fight resolver). Demanding
Dragon rides Browbeat's `PlayersMayAccept` shape with the targeted opponent as
the chooser. ⚠ **The bot never cast Unleash Fury**: it was classed as a combat
trick, so the main phase held it, but `pick_combat_trick` read only constant
pumps — a pump by the creature's own power is now priced at its current power.
Residual: **Atarka Monument** animates colorless. Release pods beside Derevi /
Hazel / Ranar (seed 8701, 1,000 games): 1,000 decided, Atarka 38.0 %; a
1,000-game census (seed 8703) leaves no card of the four lists unplayed; strict
debug pods (seed 8704, 60 games) decided 60/60. `--bench` byte-identical.

The **seventy-ninth** is Edge of Eternities Commander's **World Shaper**
(`WorldShaper_EOC`) — Jund land sacrifice under Hearthhull, the Worldseed, a
stationed Spacecraft commander (seat 78 before rebasing over Witherbloom
Witchcraft). Seventeen cards were missing (Windgrace's
Judgment landed with Squirreled Away at the same time; one definition stays).
⚠ **A static over "creatures that attacked this turn" was dropped whole**
(CR 611.3a): `requirement_live_leaves` didn't list `AttackedThisTurn` /
`BlockedThisTurn`, so Moraug's pump went through the state-blind path and
never applied. Residuals: **Eumidian Wastewaker** (discard only),
**Loamcrafter Faun** (targets chosen before the discard), **Moraug** (+1/+0
once), **Planetary Annihilation** (the engine keeps the lands), **Soul of
Windgrace** (the first graveyard's land). A 200-game census beside
Bright-Palm / Brimaz / Rin and Seri (seed 9531) decided 200/200 with zero
panics, Hearthhull winning 21.5 %, every World Shaper card played.

The **eighty-second** is Bloomburrow Commander's **Peace Offering**
(`PeaceOffering_BLC`) — Bant group hug and politics under Ms. Bumbleflower
(seat 81 before rebasing over Evasive Maneuvers). Nineteen cards were
missing; the primitives: `CounterType::Ingredient` / `Vow`,
`StaticEffect::TokenNamedBecomes` (Fisher's Talent's Fish → Shark → Octopus,
chained once per rule and gated by Class level), `Effect::ChooseColorForSelfOtherThan`
(the Thriving lands) and `Effect::EachPlayerMayDrawThenTakersGainLife`
(Kwain). Two engine finds: ⚠ **a "whenever you cast a spell" trigger bound
only its first target** (CR 603.3d — Ms. Bumbleflower's counter never landed;
the self-cast and ETB paths already filled the rest), and **an emblem's
free-cast static never applied** (CR 114.4 — Tamiyo's −7). Residuals:
**Martial Impetus**, **Octomancer**, **Perch Protection**, **Promise of
Loyalty**, **Tamiyo** (INCOMPLETE_CARDS). A 200-game census beside
Hearthhull / Bright-Palm / Brimaz (seed 9551) decided 200/200 with zero
panics and no card of the four lists unplayed, Ms. Bumbleflower winning
38.0 %. `--bench` byte-identical.

The **hundred-and-thirty-ninth** is Edge of Eternities Commander's **Counter
Intelligence** (`CounterIntelligence_EOC`, 2025-08-01) — Jeskai artifacts and
counters under the Spacecraft commander Inspirit, Flagship Vessel (CR 903.3's
Spacecraft clause), `--pod-decks 139`. Twenty cards were missing
(`cmdr_inspirit.rs`). The primitives: `Effect::SpellGainsSunburst` (CR 702.44,
Lux Artillery / Solar Array), `Effect::PutCountersOf` (leaver counters through
LKI, Resourceful Defense), any-kind `remove_counter_among_x` (Moxite
Refinery). It found three engine holes: ⚠ **a card-type-filtered set read
printed types** (CR 613.8 — an anthem missed an animated land; "artifact
creatures you control have …" missed an animated artifact; now a `SecondPass`
gate), ⚠ **`CostReduction` over the source's chosen card type never
matched** (Cloud Key; the card check has no source), ⚠ **a Station band's
`PumpPTByValue` static was dropped** (Uthros Research Craft). Residuals: six
(INCOMPLETE_CARDS). Four-seat pods beside Morska / Saheeli / Zimone (seed
13901, 1,000): all decided, Inspirit 7.2 %; six seats (13902, 1,000): all
decided, 1.4 % — ⚠ a weak bot pilot, lead open; census (13903, 500): nothing
unplayed. `--bench` byte-identical.

The **hundred-and-thirty-fourth** is Murders at Karlov Manor Commander's
**Deep Clue Sea** (`DeepClueSea_MKC`, 2024-02-09) — Bant Clues and
second-draw payoffs under Morska, Undersea Sleuth, `--pod-decks 134` (committed
as 131 and 132 while three other lists landed first). Twenty cards were
missing (`cmdr_morska.rs`; Sophia, Dogged Detective landed with Everyone's
Invited!). The primitives: ⚠ **permanent ascend is a static now**
(`StaticEffect::Ascend`, CR 702.131b — `game/ascend.rs` checks every permanent
entry and control change; six cards that re-checked at upkeep moved onto it),
`Effect::Ungoad` (CR 701.15a, Serene Sleuth),
`Effect::SilencePlayersUntilTheirNextTurn` (Innocuous Researcher) and
`StaticEffect::FirstArtifactAbilityEachTurnCostsLess` (Tezzeret, Betrayer of
Flesh). It also fixed ⚠ **Thirst for Knowledge** (in two pod lists), which
discarded two even with an artifact in hand. Residuals: **Aerial
Extortionist**, **Alandra, Sky Dreamer**, **Erdwal Illuminator**
(INCOMPLETE_CARDS). Four-seat pods beside Morophon / Sefris / Zimone (seed
13401, 1,000 games): all decided, Morska 30.7 %; six seats beside Quintorius /
Gavi / Gonti / Prossh / Nelly (seed 13402, 1,000): all decided, 16.2 %;
census beside Omo / Olivia / Jirina (seed 13403, 500): no card of the four
lists unplayed. cube / sos / sealed 7,500 decided. `--bench` byte-identical.

The **hundred-and-forty-third** is Streets of New Capenna Commander's
**Obscura Operation** (`ObscuraOperation_NCC`) — Esper connive and evasion
under Kamiz, Obscura Oculus (seat 142 before rebasing over Faceless
Menace). Twenty cards were missing and Aerial Extortionist landed with Deep
Clue Sea meanwhile, so nineteen are this seat's (`cmdr_kamiz.rs`). The
primitives: `Effect::Connive { what, amount }` (CR 701.50 — per permanent,
counting only that creature's own discards; the `shortcut::connive` Seq
read the whole resolution's list, so "each of X creatures connive"
over-counted) and `StaticEffect::DamageDoesntCauseControllerLifeLoss`
(CR 120.3a, Archon of Coronation under a monarch gate, read at the damage
life sites). ⚠ **Escape's exiled cost cards weren't "exiled with" the
escaping card**, so Skyway Robber's escaped trigger had nothing to cast.
Residuals: **Commit // Memory**, **Kamiz, Obscura Oculus**, **Obscura
Confluence**, **Oskar, Rubbish Reclaimer**. A 200-game census beside Zimone
/ Millicent / Lathliss (seed 143) decided 200/200 with zero panics and no
card unplayed, Kamiz winning 7.0 %. `--bench` byte-identical.

The **hundred-and-thirty-third** is Duskmourn Commander's **Jump Scare!**
(`JumpScare_DSC`) — Simic manifest dread and morph under Zimone, Mystery
Unraveler (seat 131 before rebasing over two other lists). Twenty cards were
missing (`cmdr_zimone.rs`); the primitive is `SelectionRequirement::
PowerParity` (Zimone's Hypothesis). Two engine holes: ⚠ **a "counter,
exile, cast it free" trigger aimed at itself** — `CounterSpellExileMayPlayFree`
had no slot-filter walker arm, so Kheru Spellsnatcher's turned-face-up
trigger read an `Any` filter and took the first permanent (it and three
sibling counter variants now also refuse a player); and ⚠ **a card cast
from the graveyard skipped its own additional costs** (CR 601.2f — Skaab
Ruinator came back without exiling three creature cards; only flashback
riders were paid). Residuals: **Ashaya, Soul of the Wild**, **Deathmist
Raptor**, **Disorienting Choice**, **Zimone, Mystery Unraveler**, **Zimone's
Hypothesis**. A 200-game census beside Millicent / Lathliss / Bright-Palm
(seed 133) decided 200/200 with zero panics and no card of the list
unplayed, Zimone winning 18.0 %. `--bench` byte-identical.

The **hundred-and-twenty-fourth** is Innistrad: Crimson Vow Commander's
**Spirit Squadron** (`SpiritSquadron_VOC`) — Azorius Spirits under
Millicent, Restless Revenant (seat 123 before rebasing over Desert Bloom).
Eighteen cards were missing (`cmdr_millicent.rs`); the primitives:
`Effect::CopySpellAsOneOneSpirit` (Donal, CR 707.9b — the exception is
written into the copy's definition, so the token keeps it),
`StaticEffect::PreventNoncombatDamageToMatching` (Drogskol Reinforcements),
`Value::CardTypesAmong` (Occult Epiphany) and
`Value::OpponentsControllingAnyOf` (Sudden Salvation). ⚠
**`CreateTokenCopyOf` never found a library card**: the source lookup walked
the battlefield, exile and graveyards, so Haunting Imitation's copies of
each player's top card were silently skipped. Residuals: **Donal, Herald of
Wings**, **Haunting Imitation**, **Spectral Arcanist**. A 200-game census
beside Lathliss / Marath / Bright-Palm (seed 124) decided 200/200 with zero
panics and no card of the list unplayed, Millicent winning 27.0 %.

The **ninety-fifth** is Foundations Commander's **Reign of Dragons**
(`ReignOfDragons_FDC`) — mono-red Dragons under Lathliss, Dragon Queen
(seat 92, then 94, before rebasing over three other lists). Thirteen cards
were missing (`cmdr_lathliss.rs`); the primitive is
`Value::DiscoveredManaValue` (Hit the Mother Lode's "the difference"). ⚠
**Celebration never counted a spell**: a resolving permanent spell only
emitted `PermanentEntered`, whose funnel logs creatures but not the
nonland/artifact entry tallies, so two creature spells never turned Goddric
on (`game/entry_tally.rs`, CR 608.3; the persist/undying return had the
same hole). Residuals: **Carnelian Orb of Dragonkind**, **Goddric, Cloaked
Reveler**, **Leyline Tyrant**, **Thundermane Dragon**. A 200-game census
beside Marath / Brimaz / Bright-Palm (seed 92) decided 200/200 with zero
panics and no card unplayed, Lathliss winning 59.5 %. `--bench`
byte-identical.

The **eighty-eighth** is Commander 2013's **Nature of the Beast**
(`NatureOfTheBeast_C13`) — Naya big creatures under Marath, Will of the
Wild (seat 83 before rebasing over four other lists). Sixteen cards were
missing; the primitives: **Mystic Barrier**'s CR 508.1a "attack only the
nearest opponent in the chosen direction" (`game/mystic_barrier.rs`, riding
the CR 803 attack-left/right walk so the declaration gate and the bot's
`attackable_players_for` agree),
`Value::PermanentsDestroyedThisResolutionControlledBy` (From the Ashes), and
`Selector::TopOfLibrary` fanning out over "each player" (Naya Soulbeast).
⚠ **No bot path spent Marath's counters**: "{X}, remove X counters" had no X
chooser; `server/x_counter_sink.rs` dry-runs every mode at every payable X.
Residuals: **Fiery Justice**, **Magus of the Arena**, **Naya Soulbeast**.
A 200-game census beside Ms. Bumbleflower / Hearthhull / Bright-Palm (seed
9561) decided 200/200 with zero panics; Marath won 4-6 %, and **Fireball
went uncast** — the bot's any-target burn goes face-first and fires only for
lethal (TODO open queue).

The **eightieth** is Commander 2018's **Nature's Vengeance**
(`NatureSVengeance_C18`) — Jund lands under the planeswalker commander Lord
Windgrace (claimed after yielding Death Toll to the session that claimed it
first). Sixteen cards were missing (Forge of Heroes and Moldgraf Monstrosity
landed with Exquisite Invention and Death Toll meanwhile; one definition
each stays). Primitives: `Keyword::CantAttackOwner` (CR 508.1a, Xantcha —
read against the defender, so the pod's attack picker honours it too),
`Effect::TokenCopyAttackingUntilEndOfCombat` (Gyrus: a copy of a graveyard
card, tapped and attacking, exiled at end of combat) and
`Effect::RevealUntilSharesCardTypeToBattlefield` (Reality Scramble).
Residuals: **Emissary of Grudges** (open choice, any redirectable spell),
**Hunting Wilds** (Forests keep their color), **Charnelhoard Wurm** (any
player), **Flameblast Dragon** (X asked before {R}; a bot pays X only from
floating mana). A 400-game census beside Hearthhull / Willowdusk / Winter
(seed 9431, release) decided 400/400, Windgrace winning 12.2 %, every card of
the four lists played. `--bench` byte-identical.

The **sixty-eighth** is Commander Masters' **Enduring Enchantments**
(`EnduringEnchantments_CMM`) — Abzan Sagas, enchantresses and constellation
under Anikthea, Hand of Erebos (seat 63, then 67, before rebasing over four
others; a pod holds at most 64 players, so it seats in 4-seat and cycled
fields). Seventeen cards were missing and Greater Tanuki landed with Raining
Cats and Dogs meanwhile, so sixteen are this seat's (`cmdr_anikthea.rs`). The
primitives: `CounterType::Blessing`;
`StaticEffect::LifeAlternativeCostOncePerYourTurn` (Demon of Fate's Design,
CR 118.9 — spent through `Player.life_alt_cast_used_this_turn`); and
`StaticEffect::SagaFinalChapterRider` (Narci, CR 714.2c — appended to a
Saga's final chapter, so a countered chapter drains nobody). ⚠ **The bot
never cast through a board-granted alternative cost** — its alt-cost block
read only a card's printed one, so Demon of Fate's Design, Kentaro and Fist
of Suns were dead text; `BoardFacts.grants_alt_cost` now asks
`effective_alternative_cost` (a life payment of at most a third of the
seat's life). Residuals: **Battle at the Helvault** targets opponents'
permanents only; **Battle for Bretagard** copies duplicate names;
**Cacophony Unleashed** isn't legendary when animated; **Ghoulish Impetus**
goads each of your upkeeps (the goad outlives the Aura until your next
turn); **Ondu Spiritdancer**'s declined copy spends the turn. Pods (release,
seed 10211, 1,000 games beside Brimaz / Inalla / Isperia): 1,000/1,000
decided, no card of the four lists unplayed, Anikthea 47.1 %. 12 seats (seed
10212) 200/200; 64 seats (seed 10213) 40/40; 63 seats (seed 10202, before
the rebase) 54/60 with 6 board caps — 62 seats on the same seed capped 4/60,
so the caps are the table's length, not this list. `--bench` byte-identical.

The **sixty-fourth** is Commander 2016's **Open Hostility** (`OpenHostility_C16`)
— four-color (no blue) aggression under Saskia the Unyielding, with Tana, Tymna
and Ravos (partner commanders) in the 99 (seat 58 before rebasing over six
others). `Order` is `order_chaos`. Fifteen cards were missing (Dauntless Escort
landed with Token Triumph first; Mirror Entity is shared with Raining Cats and
Dogs); the primitives: `StaticEffect::AllDamageDealtAsThoughWither` (Everlasting
Torment, riding the prevent-static lane so a board without it pays one lane
read; the noncombat funnel and both combat legs honour it),
`Effect::DealDamageFrom` (Saskia — the trigger's creature, not Saskia, deals the
echo, so its lifelink and infect apply),
`StaticEffect::OpponentsCantCastDuringYourTurnWhileAttached` (Conqueror's Flail),
`Value::PlayersDealtCombatDamageThisTurn` (Tymna) and `CounterType::Fury`
(Charging Cinderhorn). ⚠ **Tymna's test found postcombat-main triggers dead**:
`advance_step` never fired `StepBegins(PostCombatMain)`, so every "at the
beginning of your postcombat main phase" card (Survival, Florian, Tymna) did
nothing in real games — the tests had called `fire_step_triggers` by hand. Now
fired on entry (CR 505.1a). Residuals: Saskia's chosen player is the most
hostile opponent; Brutal Hordechief's forced blocks stay the blockers' choice;
Mirror Entity's all-types is a granted Changeling. Release pods beside Zedruu /
Kaalia / Kynaios (seed 9511, 1,000 games) decided 1,000/1,000 (Saskia 25.7 %);
strict-answer-log debug pods (seeds 9513/9514, 120 games) decided 120/120, zero
panics; a 200-game census (seed 9512) leaves no card unplayed (seat numbers as
they were then). `--bench` byte-identical.

⚠ **The board cap was a bot bug, not a loop.** A Krenko seat sent all 292
of its Goblins at one player on 24 life each turn while two others sat on
40, so it killed one seat a turn until Krenko's doubling passed the 1,024-
permanent bound. `server/pod_attack.rs::spread_face_attacks` now sends the
attackers past a kill (with one blocker's margin per untapped creature) at
the next opponent by `hostile_opponent_score`; the same game now decides.
A duel returns before allocating, so `--bench` is byte-identical.

The **twenty-second** is Tarkir: Dragonstorm's **Jeskai Striker**
(`JeskaiStriker_TDC`): flurry spells under Shiko and Narset. Fifteen cards
were missing; two primitives: `CounterType::Rally` (Aligned Heart's flurry
tally) and `Effect::MayCastFromHandFreeMatching` (a filtered free cast from
hand, shared with Kellan). Residuals: **Shiny Impetus** re-goads
at each beginning of combat rather than holding one continuous goad while
attached; **Tempest Technique**'s storm copies keep the original's target.
Seed 10041, 1,000 games at 22 seats: **995 decided, 5 action caps (432-511
turns), 0 board caps, zero panics**; Shiko wins 1.2 %, and 28.9 % of four-seat
pods beside Zellix / Zndrsplt / Hanna (seed 10042, all decided).
⚠ **The census found one card no bot cast: Narset's Reversal.** The response
picker's `effect_counters_spells` read only a `Seq`'s first step, and Reversal
copies before it bounces. Any top-level step now counts; the same seed then
played **every card of all twenty-two lists** (1,100 distinct), and `--bench`
stayed byte-identical.

The **twenty-fourth** is Outlaws of Thunder Junction's **Quick Draw**
(`QuickDraw_OTC`): Izzet storm and cascade under Stella Lee. Fifteen cards
were missing; five primitives: `Effect::OnYourNextSpellOfTypeThisTurn` (CR
603.7e — Smoldering Stagecoach's separate instant and sorcery cascades),
`Value::DistinctManaValuesInGraveyardMatching` (Eris) and
`Value::CommanderCastsFromCommandZone` (CR 903.8, Thunderclap Drake),
`Effect::ExileSelfSuspended` (CR 702.62a, Rousing Refrain) and
`SpendRestriction::SmallInstantSorceryExileInstead` (CR 106.6, Forger's
Foundry). Residuals: **Crackling Spellslinger**'s storm count is read as its
copy trigger resolves, not as the spell is cast; **Forger's Foundry**'s "may
exile" has no prompt (it exiles while you still control the Foundry, else the
graveyard, where Eris / Octavia / Stagecoach count it).
Seed 10051, 1,000 games at 24 seats: **979 decided, 20 action caps (all long
board stalls at the per-seat cap, 311-489 turns), 1 board cap, zero panics**;
the board cap is Sliver Gravemother's hive — twenty Brood Slivers make twenty
tokens per connecting Sliver, real exponential growth like Krenko's. Stella
wins 0.9 % there and 22.7 % of four-seat pods beside Zellix / Zndrsplt /
Shiko (seed 10052, all decided). ⚠ **The census found Finale of Promise cast
by no bot**: the all-slots auto-targeter never concretized its "mana value X
or less" slots (CR 601.2b), so it had no target; with the cast's X passed
through, the same seed played **every card of all twenty-four lists** (1,200
distinct). Wiring Stella's impulse draw also turned up the free-impulse class
(ENGINE_BACKLOG): forty-three "you may play that card" grants cast for {0}.

The **tenth** is the first list built around what only happens at three seats
or more, and it exists because of a census, not a hunch: **not one of the nine
decks above played a single multiplayer-native mechanic**. The monarch (CR
725), goad (CR 701.15), melee (CR 702.121), will of the council and council's
dilemma (CR 701.38, both ability words under CR 207.2c), tempting offer and
join forces (ability words, CR 207.2c), the initiative (CR 726) and myriad
(CR 702.116) were all implemented, all tested, and none had ever resolved in
bot self-play.
**A mechanic the field never plays is a mechanic self-play never crashes on**,
which is the whole point of a pod smoke test. Adriana's 99 carries 27 such
cards. It is **after** `pod_field(9)` for the reason the sixth through ninth
are; `--commander --seats 10` is what reaches it.

⚠⚠ **The number the tenth seat produces is about the *curve*, not the win
rate, and one candidate cause was tested and ruled out.** Turns/game is
linear in seats at **~12.9 a seat** and the tenth adds **+24.4** (98.81 →
123.46 and 99.44 → 123.57 at two fresh seeds, 1,000 games a block) — roughly
two seats' worth for one seat. The two seats that *shorten* the curve are
Edgar (+6.9) and Yuriko (+6.5), the two decks whose clock reads the whole
table, so the spread is a property of each added list rather than of the
format: every increment below the ninth is unchanged at the same seed.

The first hypothesis was **Grand Melee**, whose second line ("all creatures
block each combat if able") is symmetric and cancels exactly the attacks goad
and Fumiko force. It was cut for Impact Tremors — a clock that reads "each
opponent", the property Edgar and Yuriko have — and **the A/B refutes it**:
same seed, same field, one card different, turns/game 124.15 → **123.07**, a
1.1-turn move. What it did move is the deck, 4.6 → **7.2 %** of ten-seat pods
against a seat's 10.0 %. ⚠ Two later runs at fresh seeds read **5.0 %** and
**4.7 %** over 1,000 games each, so the 7.2 % was a 500-game reading and the
seat sits around **5 %** — a third of par, and the field's slowest list.
💡 **So the +24.4 is still unexplained**; the
remaining suspects are Protector of the Crown (a sponge that redirects *all*
damage dealt to its controller) and the monarch itself, which hands an extra
card a turn to whoever holds it and so lengthens every seat's game rather
than this one's. Worth one more A/B, not worth guessing in this file.

Adriana is the thesis rather than the engine: melee on every creature she
controls, so the pump reads the number of **distinct opponents attacked**, not
the number of attackers. Goad supplies those opponents by forcing the table to
swing at each other; the monarch supplies the cards while daring them to swing
back. Every one of those three clauses is a no-op in a duel.

⚠ The same run reads Zellix at **0.7 %** and Krark/Rograkh at **1.0 %** at
nine seats. That is the other half of the same fact — a deck whose clock does
not scale simply never gets there — and not (on this evidence) a defect in
either list; both are 100 % decided and neither stalls.

The **sixth** is the pod's first three-colour identity and its first
**Eminence** commander (CR 113.6b): Edgar Markov's "whenever you cast another
Vampire spell, if Edgar Markov is in the command zone or on the battlefield,
create a 1/1 Vampire" runs from the command zone in every game rather than
only in a unit test, and CR 903.4 is exercised over three colours (Command
Tower, Arcane Signet, Commander's Sphere, Path of Ancestry and Opal Palace all
read it). It is **after** `pod_field(5)` in `target_decks`, so the committed
outcome table and every 2/3/4/5-seat reading are unchanged by its existence;
`--commander --seats 6` is what reaches it.

⚠⚠ **And the number it produces is a finding about Eminence, not a deck to
tune.** Edgar wins **52.1 %** of six-seat pods (2,000 games, seed 43; field
17.6 / 7.8 / 6.8 / 12.5 / 3.1 / **52.1**) — roughly three times a seat's
share. Two rounds of nerfing said the payoffs are not the cause: cutting the
seven best tribal payoffs (Shared Animosity, Banner of Kinship, Vampire
Nocturnus, Sanctum Seeker …) took it **up**, 61.6 → 64.0 %, and only cutting
**Vampire density** — ten Vampires out for removal and ramp — moved it, 64.0 →
52.1 %. The engine is the commander: a free 1/1 per Vampire spell, from the
command zone, from turn one, with no card spent and no commander tax paid.
None of the other five commanders does anything from the command zone.

The **seventh** is the pod's only seat led by a **non-creature** (CR 903.3a).
`CardDefinition::can_be_commander` had been validated since it shipped and
never piloted; this list is what pilots it, and two consequences of a
planeswalker commander both hold in play: it is recast from the command zone
under the CR 903.8 tax like any other, and its (commander, player) damage
tally stays at zero all game, because CR 903.10a counts **combat** damage and
a planeswalker deals none — so the seat wins and loses by every other route
instead. It sits after `pod_field(6)`, so every committed 2..6-seat reading is
unchanged; `--commander --seats 7` is what reaches it.

The 99 is mono-green ramp into fat with green's own interaction, built to what
a one-ply material evaluator can price (the Judith lesson below) and to **one**
wipe's worth of interaction rather than two, per the board-wipe measurement.
Like mono-red Krark it takes neither the guild Signet nor the guild Talisman:
both cycles are two-colour in CR 903.4 identity.

`scripts/pod_deck_candidates.py` is how the list was picked — it joins every
implemented factory to the Scryfall cache and keeps the ones whose
`color_identity` is a subset of the commander's and whose
`legalities.commander` is legal, i.e. the CR 903.4 / 903.5b gates the suite
will apply, answered before the list is written rather than after it is
rejected. Mono-green had **4,323** candidates. ⚠ It cannot tell a complete
implementation from an approximated one; read `INCOMPLETE_CARDS.md` for a card
before leaning on it.

The **eighth** is the pod's only **Choose a Background** pair (CR 702.124k).
`Keyword::ChooseABackground` and `format::is_background_pair` had been
validated since they shipped and never piloted; this list pilots them, and
three things fall out that the Partner seat does not produce: the second
commander is a legendary **enchantment**, which `is_legal_commander` rejects
on its own and only the pair check lets in; CR 702.124c combines the identity
across a creature ({U}) and an enchantment ({R}); and CR 702.124d's second
21-damage tally is zero all game for CR 903.10a's reason — an enchantment
deals no combat damage, which is the planeswalker seat's consequence reached
from the other side. That last point is why the pair is Zellix +
Archaeologist rather than the mono-red Gut + Archaeologist one: UR is also a
colour identity the field did not have.

Zellix's own trigger is the pod half — "whenever **a player** mills one or
more creature cards" is `EventScope::AnyPlayer` with `once_per_batch`
(CR 603.2c), so it reads the whole table rather than its controller. The 99 is
built to the measured shape the Judith and Freyalise retunes settled on
(removal, card advantage, fat) rather than to the mill theme; the self-mill
that is there — Hedron Crab, Careful Study, Faithless Looting — feeds the Hive
Mind trigger without asking a one-ply material evaluator to price a graveyard.
It sits after `pod_field(7)`, so every committed 2..7-seat reading is
unchanged; `--commander --seats 8` is what reaches it. The UR pool had **7,272**
candidates.

⚠ **"Two commanders" stopped being a unique shape** when this list landed —
CR 702.124k is also a pair — so the Partner test now looks for two commanders
that are both *creatures* (CR 702.124h) and the planeswalker test for a deck
with exactly *one* non-creature commander.

⚠ **Two tests used to find their deck with `last()`** — the Partner one broke
when Edgar was appended and would have broken again here. Both search by
**shape** now (two commanders / a non-creature commander).

**Seven-seat field, seed 7301, 3,000 games, at the closing tip — 100 %
decided, every `undecided_by` column zero, turns/game 77.13:**

| Sigarda | Judith | Hanna | Tatyova | Krark | Edgar | Freyalise |
|---|---|---|---|---|---|---|
| 16.8 | 5.9 | 6.1 | 10.1 | 2.1 | **51.4** | 7.7 |

⚠ **Read that against the Edgar finding above, not against 1/7.** Edgar takes
51.4 % of the table, so an even share of what is left is 8.1 %, and Freyalise's
7.7 % is **15.8 % of the non-Edgar remainder against a 16.7 % even share** —
mid-field, ahead of Krark, Judith and Hanna, behind Tatyova and Sigarda. A
first build that needs no retune. The six-seat control at the same seed reads
18.8 / 7.6 / 7.2 / 11.2 / 3.6 / **51.7**, so the seventh seat costs Edgar
nothing and the rest of the field a proportional slice, which is what adding a
seat should do.

⚠ **Six seats is also the pod's only configuration with a non-zero stall
rate**: 1 action-capped game in 2,000 at seed 43 and 1 in 3,000 at seed 99
(~0.03 %), against **0 in 3,000 at both four and five seats**. The signature
is in `CRAB_CAP_DIAG=1` — pod seed `11643393128411363082`, turn 29, 50,002
actions, and the Krark seat holding **3,990 floating mana** with three
untapped permanents, i.e. a mana source the bot activates and cannot spend.
Six-seat games run ~65 turns against ~57 at five, so this is a longer tail
rather than a new defect; the seed is recorded so the next run can start
from it.

The fifth is the pod's only **two-commander** seat (CR 702.124b/d): both start
in the command zone, their CR 903.8 tax and 21-damage tallies are separate,
and Rograkh's {0} cost makes every recast pure tax. It is **last** in
`pod::target_decks`, so `pod_field(4)` never draws it and the committed
outcome table is untouched by its existence; `--commander --seats 5` and
`pod::tests::cr_702_124b_a_two_commander_seat_plays_a_pod_game` are what run
it.

Not official preconstructed lists: a precon's partition into four decks is not
derivable from the offline Scryfall cache this repo carries, and a list nobody
can verify is worse than one the suite checks every run. What they keep from
the precon idea is what the pod needs — fixed, legal, five different color
identities, built to play against each other. Each is ~37 lands (12 nonbasic),
the colorless format staples (Sol Ring, Command Tower, Arcane Signet,
Commander's Sphere, Mind Stone), then ramp / removal / draw / a creature curve.

✅ **The colorless staples are all in**: Sol Ring, Command Tower, Arcane
Signet, Commander's Sphere, Mind Stone, and — since CR 106.6 mana provenance
shipped — **Path of Ancestry** and **Opal Palace**, one of each per deck,
swapped in for a basic apiece so the land count stays at ~37 (12 nonbasic).

✅ **Each two-colour deck now runs its guild Signet and Talisman** too
(Selesnya / Rakdos / Azorius / Simic + Unity / Indulgence / Progress /
Curiosity), swapped in for the two weakest vanilla creatures apiece. The
generated 99s had passed them over because the picker ranks ramp by mana
value and the one-mana dorks won the slots. Mono-red Krark/Rograkh takes
neither cycle, and that is the interesting half rather than an omission: both
are two-colour in CR 903.4 identity (the pips are in the rules text, not the
mana cost), so `validate_commander_deck` rejects either in a mono-red list.

✅ **Three target-deck cards stopped hitting the whole table (2026-09-19).**
Endurance ("up to one **target player** puts their graveyard on the bottom",
Tatyova), Indulgent Tormentor ("unless **target opponent** sacrifices … or pays
3 life", Judith) and Nihil Spellbomb ("exile **target player's** graveyard",
Judith) each shipped as `PlayerRef::EachOpponent` — the same seat in a duel,
three seats in a four-seat pod. They are three of the 126 in CARD_BACKLOG's
"TARGET-clause class"; each has an N-seat regression test, and the committed
pod outcome table is re-blessed for them. **Aggregate, seed 43, 3,000 games at
four seats: field 39.8 / 14.7 / 20.0 / 25.5 against the recorded
40.8 / 14.6 / 20.3 / 24.3** — inside the noise, 100 % decided, 0 stalls, 41.26
turns a game. Judith held at 14.7 even though two of the three cards are hers:
they each got *weaker* (one seat instead of three), which says the retune's
win rate is not carried by them.

⏳ **Open deck-quality work**, no engine work needed — and the pod now has a
number that says which to do first:

- ✅ **Judith was not competitive and now is nearer.** Retuned 2026-09-19;
  seed 43, 3,000 games a configuration: Judith 6.2 → **14.6 %** at four seats
  (field 43.5/6.2/22.2/28.0 → **40.8/14.6/20.3/24.3**), 16.0 → 26.2 at three,
  23.2 → **33.6 %** head to head against Sigarda. `bot_ladder --commander
  --seats N --games N --seed 43` prints the table.
  ⚠⚠ **And the experiment that got there is the reusable part: the obvious
  repair made it worse.** A real aristocrats engine — free sacrifice outlets,
  drain payoffs, recursive fodder, 35 of 72 nonbasics — measured **6.3 %** at
  four seats and **16.8 %** head to head, *below the filler list it replaced*.
  `pick_sacrifice_value` takes a sacrifice only when `eval_material` improves,
  and a one-ply material evaluator cannot see a value engine: a body for a
  scry, or for one life off each opponent at 40, is material-negative every
  time it is asked. The evaluator is **not** two-player-shaped — it already
  sums over every live hostile seat — so this is a horizon limit, not a
  seat-count bug. **Build pod lists out of what a greedy evaluator can price:
  removal, card advantage, fat, and per-opponent effects that resolve in one
  shot** (Gray Merchant, Massacre Wurm, Sepulchral Primordial).
- ⚠ **Krark/Rograkh is the number now**: **9.0 %** of five-seat games (seed
  43, 3,000 games; field 37.9/14.3/16.4/22.4/9.0), 3.6 % at six and **2.1 % at
  seven**. Its two commanders are a 0/1 for {1}{R} and a 0/1 for {0}, chosen
  to exercise Partner rather than to win, so some of this is structural — but
  **mono-colour is not the explanation**: Freyalise sits under the same
  CR 903.4 constraint (neither guild cycle is legal in a one-colour identity)
  and runs at 3.7x Krark's share.
- ✅ **Every deck can now break a stalled board.** Five swaps, 2026-09-19:
  **Wrath of God** and **Fumigate** into Sigarda, **River's Rebuke** into
  Hanna, **Evacuation** and **Bane of Progress** into Tatyova; Judith already
  ran Blasphemous Act and Toxic Deluge, Krark Blasphemous Act and Mizzium
  Mortars. ⚠ **The shape of the wipe matters more than having one**, and that
  was measured rather than assumed: Hanna took *Supreme Verdict* first and
  dropped 20.0 → 15.3 %, because a deck whose plan is a wide artifact board
  cannot afford a symmetric wrath. Swapping it for the one-sided River's
  Rebuke, and dropping Evacuation, put it at **17.3 %** — and the four-seat
  spread **tightens from 25.1 to 23.6 points** overall
  (39.8/14.7/20.0/25.5 → **40.9/17.1/17.3/24.7**, seed 43, 3,000 games).
  Five seats reads 37.9/14.3/16.4/22.4/**9.0** (Krark up from 8.1).
  Cost: games run ~9 % longer (41.3 → 45.1 turns at four seats, 52.8 → 57.5 at
  five), 100 % decided and zero stalls at every seat count.

Any of these moves the committed outcome table
(`pod::tests::cr_903_seeded_pod_outcomes_match_the_committed_table`), so
re-bless in the same commit.

## Modern supplement (`catalog::sets::decks::modern`)

Extra Modern- and cube-playable cards. Most ride existing engine primitives;
newer batches also added small reusable ones (no-max-hand-size,
play-lands-from-graveyard, mana-doubling, ability-lock statics, Cipher, block
tax, landfall, graveyard escape/retrace, level bands, sideboard wishes,
manifest-from-hand, token Role Auras, Absorb, Cleave, reflect-prevention
shields, restricted colorless mana, multi-pick reveals, Spree (702.172),
Read Ahead (702.155), Frenzy keyword (702.35), …). Each card has at
least one test in `crabomination/src/tests/modern.rs`. OTJ Spree spells live in
`catalog::sets::decks::spree` (tests `tests/spree.rs`); other OTJ staples in
`catalog::sets::decks::recent66` (tests `tests/recent66.rs`).

All Modern-supplement cards are wired (including Karn, Scion of Urza and
Tezzeret, Cruel Captain, on real oracle text). A later cube sweep added a batch
of classic staples riding existing primitives — burn (Chandra's Ignition,
Psionic Blast, Reckless Rage, Electrostatic Bolt, Kaervek's Torch, Boulderfall,
Flame Jab), land destruction (Molten Rain, Rain of Tears/Salt, Choking Sands,
Seismic Spike, Fissure), removal (Kill Shot, Assassinate, Afterlife,
Excommunicate), card advantage (Weave Fate, Pilfered Plans, Aggressive Urge,
Sudden Impact, Recoup), tokens (Bestial Menace), a fight (Wild Instincts), a
combat wheel (Barbed Shocker), and simple Equipment (Short Bow, Neurok
Hoversail, Leather Armor).

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

`catalog::sets::fin` is the Final Fantasy (FIN) set module (tests in
`tests/fin.rs`), ~55 cards and growing. Most ride existing primitives (landfall,
ETB, Vehicle/Crew, Landcycling, mass −X/−X, Job-Select equipment, aristocrats
drain). New engine work this batch: `StaticEffect::PumpTeamByControlledPermanents`
(team anthem scaled by a controlled/graveyard count — Cid, Timeless Artificer;
Warrior of Light), Warrior's legendary-cast impulse via `RevealUntilFind` +
`ManaValueLessThanEventAmount`, and `DealDamageEqualToPower` now reads
last-known power when the source was sacrificed as a cost (CR 608.2h — Blazing
Bomb). A later wave added `Effect::DealDamageEqualToPowerToEach` (Nibelheim
Aflame; `each_opponent` → Chandra's Ignition), `Effect::DigForLandToBattlefield`
(Ignis Scientia), the first-combat/end-step gates
(`Predicate::IsFirst{CombatPhase,EndStep}ThisTurn` + `Effect::AdditionalEndStep`
— Genji Glove, Y'shtola Rhul), and rideable legends (Ultima, Summon: Knights of
Round, The Lunar Whale, Tellah, Ragnarok, Omega, Beatrix, Kain). Deferred FIN
cards needing more primitives are logged in TODO.md.

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

`catalog::sets::decks::recent11` (tests in `tests/recent11.rs`) exercises the
bending effects in *spell* form — Earthbending Lesson (Earthbend 4 sorcery) and
the modal Dai Li Indoctrination (discard-a-nonland **or** earthbend 2),
confirming `Effect::Earthbend` targets correctly through the modal cast path.

`catalog::sets::decks::recent31` (tests in `tests/recent31.rs`) adds the
wedge/guild modal charms & commands (Gruul/Dimir/Orzhov/Naya/Jund/Grixis Charm,
Silumgar's/Ojutai's/Atarka's Command) and the graveyard-CDA *goyf* family, on
three new reusable primitives: `DynamicPt::CreatureCardsInAllGraveyards`
(Lhurgoyf, Mortivore), `SelectionRequirement::OwnedByYou` (Gruul Charm's
"gain control of all permanents you own"), and `Effect::DestroyAndRemember`
(Orzhov Charm's "destroy and lose life equal to its toughness"). Plus Disciple
of Bolas, Agony Warp, Savage Knuckleblade, Butcher of the Horde, Demonic Dread
(Cascade), Glory (graveyard-only protection grant), Foul-Tongue Invocation, and
The First Sliver (Sliver spells you cast have cascade).

`catalog::sets::decks::recent32` (tests in `tests/recent32.rs`) is an
aristocrats / sacrifice-matters batch: Cartel Aristocrat, Bloodflow
Connoisseur, Vampire Aristocrat, Yahenni, Bontu the Glorified, Smothering
Abomination, Butcher Ghoul, Elas il-Kor, Mahadi, and Heartless Summoning.
⚠ Sacrifice-as-cost activated abilities pay the sacrifice as a COST
(`sac_other_filter`, CR 602.5b). They used to fold it in as the effect's first
step behind a `condition`, which is **not** a bound — the condition stays true
until the ability resolves, so the announcement repeats (ENGINE_BACKLOG,
twenty-third find; the ratchet is
`no_free_activation_spells_its_sacrifice_cost_in_its_effect`). New keyword
`CantAttackOrBlockUnlessCreatureDiedThisTurn` (Bontu's combat gate, wired into
attack/block legality + the client HUD strip/tooltip).

`catalog::sets::decks::recent33` (tests in `tests/recent33.rs`) adds more
sacrifice-outlet staples — Endless Cockroaches (dies → hand), Poison-Tip Archer
(reach/deathtouch aristocrat drain), Altar of Dementia (sac → mill = power),
Sadistic Hypnotist (sac → discard two, sorcery speed), Sprout Swarm (Convoke +
Buyback token).

`catalog::sets::decks::recent34` (tests in `tests/recent34.rs`) is the Zendikar
quest-counter cycle on the existing `CounterType::Quest` + `remove_counter_cost`
+ `sac_cost` primitives — Quest for the Goblin Lord (counter-gated team anthem),
Gravelord (dies → counter; remove 3 + sac → 5/5 Zombie Giant), Gemblades
(combat-damage-to-creature → counter; remove 1 + sac → four +1/+1), Ancient
Secrets (card-to-gy → counter; remove 5 + sac → shuffle gy into library), Holy
Relic (cast-creature → counter; remove 5 + sac → tutor an Equipment to play; the
auto-attach rider is dropped). Plus standalone gaps: Magebane Lizard (new
`Value::NoncreatureSpellsCastThisTurn`), Atog, Origin Spellbomb, Land Tax.

`catalog::sets::decks::recent35` (tests in `tests/recent35.rs`) adds blink/tempo/
tutor staples and the Spike counter engine. New engine primitive
`Effect::SkipNextCombatPhase` (CR 506 — `Player.skip_next_combat`, consumed in
`advance_step` when the active player would enter Begin Combat) powers Stonehorn
Dignitary. Cards: Spike Weaver (enters-with-3-counters; counter-to-target / Fog
outlets), Glimmerpoint Stag (ETB blink), Weathered Wayfarer (conditional land
tutor), Plea for Guidance, Three Dreams (enchantment/Aura tutors), Fleetfoot
Dancer, Stormscape Apprentice, Cavern Harpy, Stonecloaker, Narcolepsy (Aura
tap-lock), Bile Blight (`Selector::SharingNameWith`).

`catalog::sets::decks::recent36` (tests in `tests/recent36.rs`) adds ramp/token/
graveyard-fill commons and two punisher enchantments. New engine primitive
`Keyword::CantAttackOrBlockUnlessCityBlessing` (CR 702.131, wired into attack/
block legality + affordances + client chip/tooltip) powers Wayward Swordtooth
(also `StaticEffect::ExtraLandPerTurn` + `Effect::Ascend` on ETB/upkeep). Cards:
Hour of Promise, Pir's Whim, Gather the Pack & Tracker's Instincts
(`MillThenToHand`), Dictate of Kruphix (each draw step extra draw), Mogg
Flunkies, Wily Goblin, Hunted Witness, Brindle Shoat, Goblin Assault, Goblin
Rally, Bottomless Pit.

`catalog::sets::decks::recent37` (tests in `tests/recent37.rs`) is the Enchantress
draw cycle (Mesa/Verduran on `SpellCast`+enchantment, Femeref on enchantment→gy,
Eidolon of Blossoms constellation), three black board wipes (Mutilate scaling on
Swamp count via `Value::Times`, Golden Demise, Yahenni's Expertise — free-cast/
ascend riders dropped), Sword of the Animist (Legendary Equipment, attack →
fetch a basic), and Dawn of Hope (`LifeGained` may-draw + Soldier token).

`catalog::sets::decks::recent38` (tests in `tests/recent38.rs`) completes the
Amonkhet Monument cycle (Bontu's already shipped) on the existing cost-reduction
static + creature-cast trigger: Oketra's (Warrior token), Kefnet's
(`SkipNextUntap` on an opponent's creature), Hazoret's (`MayDo` loot), Rhonas's
(+2/+2 & trample via `PumpPT` + `GrantKeyword`).

`catalog::sets::decks::recent39` (tests in `tests/recent39.rs`) adds defensive
walls. New engine primitive `StaticEffect::PreventAllCombatDamageToThis` (CR 615,
honored in the combat-damage resolver, respecting the can't-be-prevented switch)
powers Fog Bank and Guard Gomazoa; Wall of Denial is Defender/Flying/Shroud.

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

`catalog::sets::c21` is the Strixhaven Commander (C21) module (tests in
`tests/c21.rs`) — precon staples not covered by their original printings. Mostly
lands on existing primitives: the Theros scrylands, Onslaught cycling lands,
Karoo-free utility lands (Radiant Fountain, Rogue's Passage, Mikokoro, High
Market, Temple of the False God, Blighted Woodland / Myriad Landscape sac-fetch,
Phyrexia's Core), plus Boros Locket, Zetalpa, Verdant Sun's Avatar, Sanctum
Gargoyle, Sculpting Steel, and the spells Chain Reaction, Gaze of Granite,
Biomass Mutation, Perplexing Test, Taste of Death, Brass's Bounty, Oblation.

## Engine features

| Feature | Status | Notes |
|---|---|---|
| Uncounterable spell flag | ✅ | `StackItem::Spell.uncounterable`, respected by `CounterSpell`. Cavern of Souls stamps casts uncounterable via mana provenance; Veil of Summer is a turn-scoped grant. |
| Assigns combat damage by toughness (CR 510.1c) | ✅ | `Keyword::AssignsCombatDamageByToughness`, read by `combat_damage_value` for attackers, blockers, and the cached-assignment path. Granted globally (Doran), filtered to your T>P creatures via a `ToughnessGreaterThanPower` `CardMatch` static (Tapestry Warden, Ancient Lumberknot), or as a temporary EOT grant (Bill the Pony's Food-sac). Bot scores Doran attackers at their real threat. `decks::recent23`. |
| Mutate (CR 702.140) | ✅ | `CardDefinition.mutate` + `GameAction::CastMutate`; merges onto a non-Human host you own (`CardInstance.mutate_stack`, union definition), `EventKind::Mutated` triggers (`Value::MutateCount`, `SelectionRequirement::HasMutate`), scatters on leave, snapshot round-trip. Ikoria cycle in `decks::modern` (incl. Archipelagore — `Effect::TapUpToValue` taps a runtime-`Value` count of creatures chosen at resolution). Only the client cast-mutate UI remains — see TODO.md. |
| Gift (CR 702.165) | ✅ | `CardDefinition.gift` (`Gift.gifted_effect`) + `GameAction::CastGift` + `CardInstance.gift_promised`; promising the gift resolves the enhanced effect (which bestows the gift on an opponent) and broadens cast-time/608.2b target filters (Into the Flood Maw, Long River's Pull). `TokenDefinition.tapped` mints the tapped-Fish/Treasure gifts. Client right-click "promise gift" cast; `KnownCard.{has_gift,gift_label,gift_needs_target}`. Bloomburrow batch in `decks::gift` (10 cards) + Nocturnal Hunger upgraded. `EventKind::GiftGiven` fires "whenever you give a gift" (Jolly Gerbils), including permanent gifts (emitted on enter); `Predicate::SourceGiftPromised` gates a permanent-gift ETB (Scrapshooter); the bot promises gifts via `CastGift`; client recap surfaces gifts given. |
| Survival (CR 702.180) | ✅ | "At the beginning of your second main phase, if this creature is tapped, …" — a `StepBegins(PostCombatMain)`/`ActivePlayer` trigger under an `EntityMatches{This,Tapped}` intervening-`if` (`decks::survival`: Cautious Survivor, Defiant Survivor, Shrewd Storyteller, Savior of the Small). |
| Omen (CR 702.183) | ✅ | `CardDefinition.omen` (reuses the `Adventure` shape) + `GameAction::CastOmen` + `CardInstance.omen_casting`; the creature card is cast as its instant/sorcery Omen half and shuffles into its owner's library on resolution *or* counter (handled at the `route_to_graveyard` funnel). Client right-click "Cast the Omen"; `KnownCard.{has_omen,omen_label,omen_needs_target}`. Full Tarkir Dragon-Omen cycle (17) in `decks::omen`, seeking via `Effect::Seek` (CR 701.52 — random library pick: Roost Seek, Nesting Instinct, Divining Dive). |
| Waterbend (CR 701.67) | ✅ | Completes the bending family (earthbend/airbend/blight). `CardDefinition.waterbend` (`GameAction::CastSpellWaterbend`) for the additional cast cost — mandatory + optional ("you may waterbend"), `waterbend {X}` via `Value::XFromCost`, provenance `cast_via_waterbend` → `Predicate::SpellWasWaterbend`; and `ActivatedAbility.waterbend` (`GameAction::ActivateAbilityWaterbend`) for the ability cost. Helpers (untapped artifacts/creatures you control) ride the generalized `convoke_creatures` slot, each {1}, clamped to the amount. `KnownCard.{has_waterbend,waterbend_amount}`. `decks::avatar_water` (20 cards). |
| Mayhem (CR 702.187) | ✅ | `Keyword::Mayhem(cost)` + `GameAction::CastMayhem` (delegates to the flashback machinery; exile-after tail) gated on `Player.discarded_this_turn`. The "if the mayhem cost was paid" rider now works via `CardInstance.cast_via_mayhem` → `Predicate::SpellWasMayhem` (Sandman's Quicksand). Spider-Man batch in `decks::mayhem`. |
| Harmonize (CR 702.180) | ✅ | `Keyword::Harmonize(cost)` + `GameAction::CastHarmonize` — graveyard recast; optionally tap one creature you control to reduce the total cost by generic mana = its power; exile-after (flashback tail). Bot + graveyard-browser badge. `decks::tarkir`: Channeled Dragonfire, Unending Whisper, Ureni's Rebuff, Wild Ride, Mammoth Bellow. |
| Freerunning (CR 702.179) | ✅ | `AlternativeCost.condition: DealtCombatDamageToPlayerThisTurn` (`Player.dealt_combat_damage_to_player_this_turn`, set at the combat-damage-to-player choke point). ACR batch in `decks::freerunning` (10 cards). "With an Assassin or commander" approximated as "with any creature". |
| Marvel's Spider-Man (SPM) | ✅ | `decks::spm` — Standard staples on existing primitives: Spiders-matter (Aunt May's enter-buff, Mary Jane's once-per-turn draw, Thwip!/Grow Extra Arms Spider riders, Radioactive Spider tutor, Spider-Suit type-grant), Villain value (Common Crook/Merciless Enforcers, Mob Lookout connive, Morlun `Value::XFromCost` counters+burn), plus City Pigeon/Spider-Girl LTB tokens, Doc Ock's Tentacles `MayDo` auto-attach, and Vibrant Cityscape ramp. New: `CreatureType::Performer`. Tests in `tests/spm.rs`. |
| Web-slinging (CR 702.188) | ✅ | Modeled on the alt-cost primitive (`AlternativeCost.mana_cost` + `return_to_hand` of one tapped creature). `decks::webslinging`: Spider-Man Web-Slinger, Amazing Spider-Girl, Silk, Spider-Man India. The "if cast using web-slinging" provenance riders are deferred (TODO.md). |
| Job Select (CR 702.182) | ✅ | Equipment ETB mints a 1/1 colorless Hero token and self-attaches (living-weapon shape — `job_select_equipment`): Monk's Fist, Bard's Bow. The "is also a [class]" type-add rider is dropped (`EquipBonus` overrides types, doesn't add). |
| Tarkir: Dragonstorm (non-Omen) | ✅ | `decks::tarkir` — ~110 cards. Khans wedge tri-lands (`tri_land`), Monuments (ETB basic tutor + sac payoff), Devotees (`OfColors` once-per-turn mana), the Exhale "behold a Dragon" cycle (Dragon-control rider), plus Formation Breaker (`CantBeBlockedByPowerLess`), Krotiq Nestguard (`AttackDespiteDefenderThisTurn`), Snowmelt Stag (`SetBasePtIf`). **Flurry** (`shortcut::flurry`: Cori Mountain Stalwart, Monk of the Open Hand, Jeskai Devotee, Wingblade Disciple, Poised Practitioner, Devoted Duelist, Wayspeaker Bodyguard), **Mobilize** N (`shortcut::mobilize`) + **Mobilize X** (`shortcut::mobilize_value` — Avenger of the Fallen, Dalkovan Packbeasts, Nightblade/Shock Brigade, Reigning Victor), **Renew** = graveyard-exile activated ability incl. keyword-counter grants (Champion of Dusan/Sagu Pummeler/Qarsi Revenant/Alchemist's Assistant + Agent of Kotis, Adorned Crocodile, Lasyd Prowler, Constrictor Sage), Bone-Cairn Butcher, Sage of the Fang / Naga Fleshcrafter, Mox Jasper, Sky Skiff, Severance Priest, Omenpath to Naya. |
| The Ring tempts you (CR 701.54) | ✅ | `Effect::RingTempts` + `Player.{ring_temptations,ring_bearer}`; the four cumulative emblem abilities ride the level (can't-be-blocked-by-greater-power, attack-loot, blocked-creature `Effect::SacrificeAtEndOfCombat`, combat-damage drain) **plus the level-1 "Ring-bearer is legendary" rider (CR 701.54c)** via a synthetic `Modification::AddSupertype` layer-4 effect. `EventKind::RingTempted` powers "choose a Ring-bearer" payoffs. `decks::ltr` (~60 cards: Birthday Escape, Call of the Ring, Bilbo, Frodo Baggins, Samwise Gamgee, Quickbeam, Mirror of Galadriel, Olog-hai Crusher, Glóin, Nazgûl, Gandalf's Sanction, …). Bearer auto-picked (highest power); per-player UI choice is a TODO.md follow-up. |

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
