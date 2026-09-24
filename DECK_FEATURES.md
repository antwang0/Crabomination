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
| **Plunder the Graves** (C15 precon) BG | Meren of Clan Nel Toth | BG | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Graveyard Overdrive** (M3C precon) BRG | Disa the Restless | BRG | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Sworn to Darkness** (C14 precon) B | Ob Nixilis of the Black Oath (**planeswalker**) | B | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Seize Control** (C15 precon) UR | Mizzix of the Izmagnus | UR | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Rebellion Rising** (ONC precon) RW | Neyali, Suns' Vanguard | RW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Call the Spirits** (C15 precon) WB | Daxos the Returned | WB | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Peer Through Time** (C14 precon) U | Teferi, Temporal Archmage (**planeswalker**) | U | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Quantum Quandrix** (C21 precon) GU | Adrix and Nev, Twincasters | GU | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Lorehold Legacies** (C21 precon) RW | Osgir, the Reconstructor | RW | 100 | 🟡 all 100 implemented, 3 carry residuals (below) |
| **Counterpunch** (CMD precon) WBG | Ghave, Guru of Spores | WBG | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Wade into Battle** (C15 precon) RW | Kalemne, Disciple of Iroas | RW | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |
| **Arm for Battle** (CMR precon) RW | Wyleth, Soul of Steel | RW | 100 | 🟡 all 100 implemented, 2 carry residuals (below) |
| **Fae Dominion** (WOC precon) UB | Tegwyll, Duke of Splendor | UB | 100 | 🟡 all 100 implemented, 4 carry residuals (below) |
| **Invent Superiority** (C16 precon) WUBR | Breya, Etherium Shaper | WUBR | 100 | 🟡 all 100 implemented, 1 carries a residual (below) |

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
Explorer). Residuals: **Righteous Confluence** offers its repeatable modes as
one choice among the four non-targeting combinations; **Sandstone Oracle**'s
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
spell is accepted with no target at all (Murder and Doom Blade too — CR
601.2c; TODO). Residual: **Footbottom Feast** picks its cards as it resolves.
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
