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
reaches it. Next precons by `scripts/precon_scan.py`: Heads I Win, Tails You
Lose (SLD, 9 missing), Wretched Ranks (FDC, 9 — releases 2026-10-02).

🟡 **Residuals in the list** (each also on its card's doc): Welcome the Dead
(X counts every card binned this turn, not only from hand/library), Colossal
Grave-Reaver (returns the first milled creature, not a chosen one), River
Kelpie (exile casts fire the cast trigger), Lethal Scheme (convokers don't
connive), Shigeki (the kept land passes through the graveyard), Cephalid
Coliseum (the sacrifice is folded into resolution). Steward of the Harvest and Life from the Loam pick at resolution
rather than target.

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

🟡 **Residuals in the list**: Grell Philosopher (only Grell gains the
artifact's abilities; no blue-as-any rider), Psionic Ritual (no tap-a-Horror
replicate), Herald's Horn (no upkeep reveal), Overcharged Amalgam (counters
spells only, not abilities). Sewer Nemesis always chooses an opponent.

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

🟡 **Residuals in the list**: Master of Dark Rites (the restriction reads a
creature spell's types, so a Demon *noncreature* spell can't use it), New
Blood (the text change is approximated), Secluded Courtyard (only the cast
half of the restriction).

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
