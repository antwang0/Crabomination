# Reality Fracture puzzles — a benchmark set for the bots and the value net

Drafted 2026-10-23. Each puzzle is a fixed two-player position with an
**outcome goal**, in the shape of the retired puzzle runner (`bot_puzzles`,
`server/puzzle{,_corpus}.rs`, recoverable from `dc7143057`): a goal is met or
not, every winning line passes, and the brute-force solver certifies that a
solution exists and that the default policy doesn't already find it. The
"solution" and "trap" below are the author's derivation, to be certified by
the solver, not trusted.

Why FRA specifically: **277 of the 280 FRA cards are outside the champion's
vocabulary** (only 3 FRA names are in `VOCAB_SNAPSHOT`'s trained rows), so
the net sees every FRA card through the generic object features alone —
P/T, keywords, counters (feats 48-58), type bits. These puzzles probe
FRA rules the features can't express (negative power, toughness damage,
"attacks alone", empowered Jace tokens, granted loyalty abilities), plus
two known weak spots: the trick blind spot (r63/r64) and the horizon
(r79).

Conventions for every position unless stated: seat 0 is the hero, it's
the hero's precombat main phase, both libraries hold 20 vanilla filler
cards, lands are untapped basics, the hero's creatures can attack (no
summoning sickness), and neither player has cards in hand beyond those
listed. "Vanilla N/N" means a creature with no abilities.

| id | tests | goal |
|---|---|---|
| FP1 `tetsuko_pump_after_blocks` | trick timing | win this turn |
| FP2 `yuriko_attack_alone` | attack-set choice | win this turn |
| FP3 `loot_negative_power` | negative power, self-sacrifice | win this turn |
| FP4 `ghalta_toughness_damage` | cost reduction, damage rule, main-1 sequencing | win this turn |
| FP5 `ingris_haste_cadets` | mana sink before combat | win this turn |
| FP6 `darklight_phoenix_main_one` | intervening-if timing | win this turn |
| FP7 `germinate_after_lifegain` | spell order | win this turn |
| FP8 `overwrite_into_warlord` | empower Jace + granted loyalty ability | win this turn |
| FP9 `sculptor_shield` | horizon: loyalty vs survival | survive 1 turn |
| FP10 `chandra_stun_the_threat` | horizon: stun counters | survive 1 turn |
| FP11 `kiora_activate_first` | "if you've activated a loyalty ability" | win this turn |
| FP12 `fblthp_empty_library` | alternate win, surveil to the graveyard | win this turn |
| FP13 `ferocity_exile_assignment` | removal assignment around a return | clear their creatures |
| FP14 `stingcaster_flashback_burn` | the graveyard as a resource | win this turn |
| FP15 `face_yourself_target_them` | player-target polarity | win this turn |
| FP16 `sanctum_lurker_ping` | burn + empower + granted +2 | win this turn |

**Engine support, checked 2026-10-23** by reading every card's definition
and its engine path. All 18 cards are faithful except one. **Ghalta the
Immovable is approximated:** its toughness-greater-than-power gate reads
printed P/T plus counters and ignores pumps (`layers.rs:2235`). FP4 still
holds, because Yargle's printed 3/9 qualifies, but a pumped variant of FP4
would be wrong. Two FP13 facts are confirmed:
- Essence Burn's exile suppresses Ferocity's return (`death_was_replaced`).
- No test covers that pairing.

---

### FP1 `tetsuko_pump_after_blocks` — trick timing

- **Hero:** Tetsuko Umezawa, Fugitive (1/3), two 1/1 vanilla tokens; 3
  Mountains. Hand: Charge the Sanctum ({2}{R/W} instant).
- **Opponent:** 7 life; three untapped vanilla 3/3s.
- **Solution:** attack with all three. Tetsuko makes creatures with power
  or toughness 1 or less unblockable, and the check happens at blocks.
  After blockers (none legal), cast Charge the Sanctum, mode "creatures you
  control get +2/+0": 3 + 3 + 3 = 9.
- **Traps:** pumping before blocks makes Tetsuko 3/3 and blockable. The
  3/1 tokens stay unblockable (toughness 1), so that line deals 6 and
  fails by 1. Not pumping deals 3. Mode 2 on one creature (+2/+0, first
  strike, a +1/+1 counter) makes that creature blockable.
- **Why it's hard:** this is the r63/r64 trick blind spot with a hard
  edge. The same card wins or loses depending on which side of declare
  blockers it's cast.

### FP2 `yuriko_attack_alone` — attack-set choice

- **Hero:** Yuriko, Blade of the Mighty (2/3), vanilla 5/5, two vanilla
  1/1s. No hand.
- **Opponent:** 10 life; no creatures.
- **Solution:** attack with the 5/5 alone. Yuriko gives a creature that
  attacks a player alone double strike, so it deals 10.
- **Traps:** attacking with everything into an empty board deals 9. The
  most natural line fails by one.

### FP3 `loot_negative_power` — negative power and self-sacrifice

- **Hero:** Loot, the Anomaly (−2/4), a vanilla 1/1; 7 cards in the
  graveyard (threshold). No hand.
- **Opponent:** 4 life; no creatures.
- **Solution:** activate Loot's threshold ability, sacrificing the 1/1.
  Loot becomes −4/4, and with negative power he assigns combat damage as
  though it were positive. Attack: 4.
- **Traps:** attacking with both deals 2 + 1 = 3. Sacrificing your own
  creature reads as losing material. For the net, the power feature is
  *negative* and means the opposite of what it says.
- **Variant (harder):** add Charge the Sanctum to hand with 3 Mountains.
  The +2/+0 mode moves Loot toward 0 and *costs* damage.

### FP4 `ghalta_toughness_damage` — cost reduction, damage rule, main-1 sequencing

- **Hero:** Yargle, Goliath of Otaria (3/9); 1 Plains. Hand: Ghalta the
  Immovable ({8}{W}; costs {X} less, X = greatest toughness among your
  creatures).
- **Opponent:** 9 life; no creatures.
- **Solution:** cast Ghalta for {W} *before combat*. A creature whose
  toughness exceeds its power then assigns combat damage equal to its
  toughness. Yargle attacks for 9.
- **Traps:** Ghalta's printed cost of 9 hides that it's castable. Its 0/7
  body is "bad", so attacking first and casting Ghalta in main 2 deals 3.

### FP5 `ingris_haste_cadets` — mana sink before combat

- **Hero:** Ingris Stingerquill (1/4 flying; "whenever a creature you
  control attacks, it deals 1 damage to each opponent"; "{4}: create a
  2/2 Cadet, then creatures you control gain haste"); 8 Mountains. Hand:
  Heartstring Puller ({3}{R} 3/1 trample; its ETB makes a Cadet).
- **Opponent:** 3 life; three untapped 0/5 walls with defender and reach.
- **Solution:** any line that attacks with three or more creatures,
  because every attacker pings for 1. The two that work are: activate
  {4} twice, then attack with Ingris and two hasty Cadets; or cast the
  Puller, then activate once and attack with four.
- **Traps:** casting the Puller and attacking with Ingris alone deals 1.
  The walls block three attackers, so combat damage is never the answer.

### FP6 `darklight_phoenix_main_one` — intervening-if timing

- **Hero:** no creatures; Darklight Phoenix (3/2 flying haste) in the
  graveyard; 3 Mountains. Hand: Fulminous Forte ({2}{R} instant).
- **Opponent:** 3 life; two vanilla 1/1s (no flyers, no reach).
- **Solution:** in main 1, cast Forte, mode "1 damage to each creature
  and planeswalker your opponents control". Two creatures die. At
  beginning of combat the Phoenix returns, attacks with haste, and deals
  3 in the air.
- **Traps:** holding the instant for combat. Casting it in the beginning
  of combat step is too late, because the trigger has already checked its
  condition. Mode 2 (5 damage to one creature) kills only one.

### FP7 `germinate_after_lifegain` — spell order

- **Hero:** Samut, Hazoret's Champion (2/2; your creatures have haste);
  5 Plains. Hand: Surgical Precision ({1}{W} sorcery; mode 2 draws a card
  and gains 2 life), Germinate Recruits ({2}{W} instant; X 2/2 Cadets,
  X = life gained this turn).
- **Opponent:** 6 life; no creatures.
- **Solution:** Surgical Precision for the life, then Germinate (X = 2),
  then attack with Samut and two hasty Cadets: 2 + 2 + 2 = 6.
- **Traps:** Germinate first makes zero Cadets. The mana is exactly
  enough for both.

### FP8 `overwrite_into_warlord` — empower Jace and a granted loyalty ability

- **Hero:** Way of the Warlord on the battlefield, put there directly so
  there is no Jace token yet ("planeswalkers you control have [−4]: 2
  damage to up to one target creature or planeswalker and 2 damage to
  target player"); 6 Swamps. Hand: Overwrite the Multiverse ({4}{B}{B};
  exile all creatures, empower Jace X = number exiled). No creatures.
- **Opponent:** 2 life; four vanilla 3/3s.
- **Solution:** Overwrite exiles four creatures. Empower Jace 4 creates
  a Jace token with 4 loyalty, which uses the granted [−4] to deal 2 to
  the opponent.
- **Traps:** none on the board. The win is two cards apart, and the Jace
  token's printed abilities (surveil, draw) don't hint at it.
- **Control:** with three opposing creatures the token has 3 loyalty, the
  puzzle is unsolvable, and the solver must say so.

### FP9 `sculptor_shield` — horizon: loyalty vs survival

- **Hero:** 3 life; Jace, Reality Sculptor with 4 loyalty; 5 Islands. No
  creatures, no hand.
- **Opponent:** three vanilla 4/4s, all able to attack next turn.
- **Goal:** survive the opponent's next turn.
- **Solution:** −3: until your next turn, every creature that attacks you
  or your planeswalkers gets −5/−0.
- **Traps:** +1 (empower Jace 5, so a token with 5 loyalty) is the
  "value" line and loses to 12 damage. A value net that prices loyalty
  without the horizon takes it.

### FP10 `chandra_stun_the_threat` — horizon: stun counters

- **Hero:** 6 life; Chandra, Chill of Compliance with **1** loyalty; 3
  Islands. No creatures.
- **Opponent:** an untapped vanilla 8/8 with trample, and a vanilla 1/1.
- **Goal:** survive the opponent's next turn.
- **Solution:** −1: tap the 8/8 with a stun counter. The counter is
  removed instead of the 8/8 untapping, so it can't attack. Chandra dies
  to the activation, and the 1/1 deals 1.
- **Traps:** either +1.
- **Revision (2026-10-23):** the first version had Chandra at 3 loyalty
  and no 1/1. The `pass` control *survived* it, because the default
  opponent spent the 8/8 on Chandra. Now the opponent can kill Chandra
  with the 1/1 and still swing lethal.

### FP11 `kiora_activate_first` — "if you've activated a loyalty ability this turn"

- **Hero:** Kiora of Salt and Sand (2/4), a vanilla 5/5, a Jace token
  with 2 loyalty ([−1]: surveil 1; [−3]: draw a card).
- **Opponent:** 5 life; one untapped vanilla 6/6.
- **Solution:** activate the Jace token's −1 in main 1, then attack. Kiora's
  attack trigger untaps the 5/5 and makes it unblockable: 5.
- **Traps:** the surveil looks pointless and spends loyalty. Without it,
  the 6/6 blocks the 5/5 and Kiora deals 2.

### FP12 `fblthp_empty_library` — alternate win

- **Hero:** Fblthp, Impossibly Lost (1/1); an Island and a Swamp;
  **exactly 4 cards in the library**. Hand: Theorix Charm ({U}{B}).
- **Opponent:** 20 life; no creatures.
- **Solution:** Theorix Charm, mode "mill three cards, then draw a card",
  empties the library. Fblthp attacks and deals combat damage, you draw two
  from an empty library, and win.
- **Traps:** milling your own library reads as a loss, and the other two
  modes are a counterspell with nothing to counter and a −2/−2.
- **Revision (2026-10-23):** the first version surveiled with a Jace
  token. A surveil is a decision answer, which the solver deliberately
  doesn't search (`legal_actions`), so it certified unsolvable. A mode is
  an action.

### FP13 `ferocity_exile_assignment` — removal assignment around a return

- **Hero:** 2 Mountains, 2 Swamps. Hand: Essence Burn ({1}{R}; 5 damage
  to target black or green creature or planeswalker; if it would die this
  turn, exile it instead), Last Gasp ({1}{B}; −3/−3).
- **Opponent:** a vanilla *green* 3/3 enchanted with Ferocity of the Hunt
  (+1/+0, deathtouch; when it dies it returns tapped), and a vanilla
  *black* 3/3.
- **Goal:** end the turn with no opposing creatures.
- **Solution:** Essence Burn the enchanted creature (exiled, no return),
  Last Gasp the black 3/3.
- **Traps:** Last Gasp on the enchanted creature kills it, it comes back,
  and the goal fails. Both spells are legal on both targets, so only the
  replacement effect decides the assignment.

### FP14 `stingcaster_flashback_burn` — the graveyard as a resource

- **Hero:** 4 Mountains; No Admittance ({1}{R} sorcery, 3 damage to any
  target) in the graveyard. Hand: Stingcaster Mage ({1}{R} 2/1 haste; ETB:
  an instant or sorcery in your graveyard gains flashback equal to its
  mana cost), Heartstring Puller ({3}{R}).
- **Opponent:** 5 life; no creatures.
- **Solution:** Stingcaster Mage, then flash back No Admittance at the
  opponent for 3, then the Mage attacks for 2: 5.
- **Traps:** the Puller is the bigger body for the same 4 mana and deals
  0 this turn.

### FP15 `face_yourself_target_them` — player-target polarity

- **Hero:** 7 Mountains; no creatures. Hand: Face Yourself ({5}{R}{R};
  for each creature *target player* controls, create a hasty token copy
  of it).
- **Opponent:** 10 life; a 4/4, a 3/3 and a 3/3, all vanilla and all
  *tapped* (they attacked last turn).
- **Solution:** Face Yourself targeting the opponent: three hasty copies
  attack for 10. The end-step sacrifice doesn't matter.
- **Traps:** targeting yourself copies nothing. This is the round-67
  shape, where the auto-target tried the caster first, with the polarity
  reversed: the *copies* are yours, but the *target* must be them.

### FP16 `sanctum_lurker_ping` — burn, empower and a granted +2

- **Hero:** Sanctum Lurker (3/2; ETB already resolved; "planeswalkers you
  control have [+2]: 1 damage to each opponent and you gain 1 life"); 2
  Mountains. Hand: No Admittance ({1}{R}; 3 damage to any target, then
  empower Jace 1).
- **Opponent:** 4 life; one untapped vanilla 5/5.
- **Solution:** No Admittance to the face (3) creates a Jace token with 1
  loyalty. The token's granted [+2] deals 1 more: 4.
- **Traps:** burn only counts as lethal if the bot sees the token's
  granted ability. 3 damage doesn't kill the 5/5, and attacking into it
  loses the Lurker.

---

## Evaluation pairs — the value net alone

Puzzles grade the pilot. These grade the net directly: two positions that
differ in one FRA-relevant fact, where the better side is not in doubt. The
net passes a pair if it scores the better position higher. All cards are
off-vocab, so each pair isolates a feature the net does or doesn't use.

| pair | better | worse | probes |
|---|---|---|---|
| E1 | opponent's 8/8 tapped with 2 stun counters | the same 8/8 untapped | feat 50 (stun) |
| E2 | our Loot at −4/4 (after the sac) attacking into an empty board, opponent at 4 | Loot at −2/4 | negative power |
| E3 | our Jace token at 7 loyalty | at 1 | loyalty feature on a token |
| E4 | our Dark Matter Manipulator with 7 cards in the graveyard (+2/+0) | with 6 | threshold, a static the feature P/T already shows |
| E5 | opponent's creature enchanted with Ferocity of the Hunt, our Essence Burn in hand | the same with our Last Gasp in hand | none — a control the net should fail (hand contents) |
| E6 | Yargle 3/9 with Ghalta the Immovable on the battlefield, opponent at 9 | Yargle alone | rule-changing static, invisible to features |

E5 and E6 are expected failures. They mark where the features stop, so
a later net with description tokens (`TableNet` step 3) has a before and
after.

## Results, 2026-10-23

`bot_puzzles --set fra --certify` (depth 3) keeps **7 of 16**. The
other 9 are solved by the default heuristic. `--all` scores all 16, and
the `pass` control (never chooses) fails all 16, so every trap is real.

| pilot | solved | misses |
|---|---|---|
| `pass` (control) | 0/16 | all |
| `dflt` | 9/16 | FP1, FP2, FP5, FP12, FP13, FP14, FP16 |
| `net67` (champion leaf) | 10/16 | FP1, FP2, FP5, FP12, FP13, FP14 |
| `mcts-dflt-256` | 8/16 | FP1, FP2, FP4, FP5, FP7, FP8, FP13, FP14 |
| `mcts-net67-256` | 8/16 | FP1, FP2, FP4, FP5, FP7, FP8, FP13, FP14 |
| `net67`, FRA net r82 s43 | 10/16 | FP1, FP2, FP5, FP12, FP13, FP14 |
| `net67`, FRA net r82 s97 | **11/16** | FP1, FP2, FP5, FP12, FP13 |
| `mcts-net67-256`, either FRA net | 8/16 | as on the champion |

Findings:
- **The search loses three sequencing puzzles the plain bot solves:**
  Ghalta precombat, Germinate after the lifegain, and Overwrite into
  Warlord. It solves Fblthp's mill and the Lurker's ping, which the plain
  bot misses.
- **No pilot passes FP1** (the trick blind spot), **FP2** (attack alone),
  **FP5** (activating Ingris before combat), **FP13** (the exile
  assignment) or **FP14** (flashback).
- **FP13 found an engine bug.** Essence Burn's exile replacement didn't
  stop Ferocity of the Hunt's return: the orphaned Aura's look-back
  trigger lane never asked `death_was_replaced`. It is fixed, with test
  `recent_b::fra::cr_614_6_essence_burn_exile_stops_ferocity_of_the_hunt`.

**Solver extensions this set needed:**
- `legal_actions` now offers modes, flashback casts, prepared-spell
  casts, and printed and granted loyalty abilities (−X at every payable X).
- Targets are brute-forced over none, each player and each permanent.
- Mana abilities are excluded: a cast pays its own cost, and an untapped
  land made every window "substantive", which spent the depth on passes.

## To run these

    cargo build --profile release-fast -p crabomination --bin bot_puzzles
    target/release-fast/bot_puzzles --set fra --certify --show-lines
    target/release-fast/bot_puzzles --set fra --all pass dflt net67 mcts-dflt-256 mcts-net67-256
    CRAB_NET=<weights> target/release-fast/bot_puzzles --set fra --all net67

(The plan below is what this section replaced.)


1. Restore the runner from `dc7143057` (`server/puzzle.rs`,
   `server/puzzle_corpus.rs`, `bin/bot_puzzles.rs`) onto the current
   engine. `Goal` already has `WinThisTurn`, `SurviveWithin(n)` and
   `ClearOpposingCreatures`.
2. Build each position from the real catalog factories. The old corpus
   used hand-made vanillas; these need the FRA cards themselves.
3. Certify: a puzzle stays only if the solver finds a line and the
   default pilot doesn't. Report pass/fail per pilot (`dflt`, `net67`,
   `mcts-net67-256`) and per tag.
