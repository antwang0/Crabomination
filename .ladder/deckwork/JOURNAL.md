# Deck work journal (times are BST, 2026-09-06)

## 04:45 Phase 0 — tooling built, smoke passed
- `deck_gauntlet` fast path: 4,800 games in 2 s on 20 threads (~8 ms a game a
  thread; PERF's per-thread bench rate). Server+replay path with `mcts64`:
  6 s for two 21-25 turn games. Field of 24 dumped to `field/`.
- Raw events of a Rancorous Archaic cast (turn 9, smoke replay): the engine
  tapped U, R, B, G, W and put 5 counters on it — converge payment is
  correct. The scanner's first X=1 was a parse-order bug (enters-with
  counters are reported before `PermanentEntered`); fixed.

## AMENDMENT A (04:50, before any variant was screened)
Screen cells are ~free, so the screen cell is **500 pairs x 24 = 24,000
games per seed, both seeds (43, 97) for every variant** (+/-0.63 a cell,
+/-0.45 pooled). Adoption bar unchanged (delta >= 1.5 pts and >= 2 se, both
seeds agreeing in sign). Final confirmation adds a **held-out field**
(`--field-seed 0xDECC1000`) so a winner is not fitted to 24 decks.

## 04:52 Phase 1 — baselines (dflt both seats, 24,000 games a cell)
| list | seed 43 | seed 97 |
|---|---|---|
| converge3 (46 cards) | 40.55 | 41.00 |
| converge3, deck seat `convlands` | 36.85 | 37.60 |
| recommended W/B (yardstick) | 38.66 | — |
- The converge-lands land-drop flag is **-3.5 pts** on the deck it was
  written for: not a fix, a finding for the ML program (it was "off until
  laddered"; now laddered on its own deck and it loses).
- The converge deck already beats the recommender's W/B list against this
  field. Per-pack breakdown at 100 pairs showed no monotone trend with pack
  count (36.8/40.7/39.2/41.5) — the 16- and 20-pack builders are not
  visibly stronger than the 9-pack ones at this builder.

## 04:59 Phase 3 — hand-made variants (dflt, 48,000 games each, seeds 43+97)
| variant | win% | vs converge3 |
|---|---|---|
| v4_removal (V1 +Witherbloom Charm +Vastlands Scavenger -Pensive Professor -Zimone's Experiment) | 57.20 | +16.4 |
| v2a_17lands (V1 -Plains +Oracle's Restoration) | 55.42 | +14.7 |
| v2b_5colour (V1 -Forest +Mountain) | 55.04 | +14.3 |
| v1_trim40 (-Divergent Equation -Arcane Omens -Traumatic Critique -1 Sundering -Mountain -Fields of Strife) | 54.69 | +13.9 |
| v3_bombs (V1 +2 Fractal Mascot -Snarl Song -Send in the Pest) | 53.26 | +12.5 |
| v5_curve (V1 +Thornfist +Essenceknit +Pest Mascot -Together -Snarl -1 Rancorous) | 49.01 | +8.2 |
| v7_notrudge (V1 -Slumbering Trudge +Oracle's Restoration) | 48.50 | +7.7 |
| v6_46at17 (all 26 spells, 17 lands, 43 cards) | 43.01 | +2.2 |
- The 46-card size and the four weakest spells together cost ~14 pts; the
  lands alone (v6) only ~2 of them. Slumbering Trudge is worth ~6 pts in
  V1 (v7). The converge top end is worth ~5.5 (v5). Fractal Mascots for
  Snarl Song + Send in the Pest are -1.4 (v3) — not the upgrade they look.
- **Incumbent := v4_removal** (clears the bar vs v1_trim40 by +2.5).
- Phase 4 gen 1 started from v4_removal: every single cut (neutral add =
  Oracle's Restoration), then every legal add, then lands, then combos.

## 05:08 Phase 1 confirm — converge3 under mcts256 both seats: 35.42 (±2.8, 1,200 games)
Lower than the scored pilot's 40.8; same direction. Replay scan (1,200 games):
- Mana: <4 lands after own T4 in 30 % of games, yet >=8 lands after T8 in
  48 % of games reaching T8 — 20 lands in 46 cards floods AND screws.
- Per-card WR-when-seen, top: Ancestral Recall 63 (Emeritus re-prepares
  it), Blech 52, Snarl Song 51.7, Emeritus 50.8, Trudge 50.8, Dellian 49.
  Bottom: Divergent Equation 32.6 (cast 86 %, mean turn 4.2 — cast for
  tiny X), Arcane Omens 35 (cast 34 %), Zimone's 37.9, Feed the Swarm 38.2,
  Quandrix Charm 38.7, Traumatic Critique 38.9. The V1 cuts match.
- Converge payment is right: Rancorous X mostly 3-4 (mean colours 3.8),
  Snarl/Together X 4-5. No payment bug.
- **Bot: Slumbering Trudge X.** Of 410 casts, 179 were X=0 (turn ~2-3:
  a 6/6 with three stun counters, untaps turn 6), 78 X=1, 54 X=2; ~99
  X>=3. Candidate fix behind a flag; the card is +6 pts even so (v7).
- **Bot: suicides** — 11 % of the deck's attacks (453/4129) and 9 % of the
  opponents' die for nothing under the 256 search: the round-65 hole,
  both seats, not deck-specific.

## 05:23 Phase 4 gen 1 — every single cut from v4_removal (neutral add Oracle's Restoration)
Passing the bar: -Quandrix Charm **+3.9** (61.11), -Witherbloom Charm +1.7.
Near zero: Shared Roots +0.5, Proctor's Gaze +0.2, Studious First-Year 0.0,
Feed the Swarm -0.3, Together as One -0.4, Send in the Pest -0.6.
Costly to cut: Trudge -6.4, Emeritus -6.1, Snarl Song -4.2, Dellian -3.2,
Rancorous -3.1, Blech -3.0, Shopkeeper's Bane -2.6, Vastlands -2.1,
Knockout -1.6, Environmental Scientist -1.5, Sundering -1.4, Wander Off -1.0.
- Both modal charms are liabilities as the bot plays them (cheap instants:
  the trick/counter windows are the bot's known blind spot).
- **Incumbent := inc1 = v4_removal - Quandrix Charm + Oracle's Restoration.**
- Next: every legal add into inc1 for Witherbloom Charm (reference =
  the second Oracle's Restoration), +/-1 basic, and the stun-hold gate —
  all on the rebuilt binary, after inc1 reproduces on it.

## 05:25 Bot diagnostics from the converge3 search replays (600 games read)
- Quandrix Charm: 123 of 130 casts were the 5/5 mode, mean turn 12.9, one
  counter. The bot plays it as a late pump; cutting it for a cantrip is
  +3.9 — the modal-instant windows are the bot's blind spot, not the card.
- **Divergent Equation: 221 casts, 0 cards returned in every one** (mean
  turn 8.6). The bot casts an X spell that does nothing 86 % of the games
  it sees it. Engine/bot follow-up (ChooseCards on `MoveChosen` with an
  X cap, or X stamped 0); the card is already out of the deck.

## 05:28 Two gated bot flags built (release-fast rebuilt 05:24 and 05:27; screens resume on the new binary after inc1 reproduces)
- `stun_x_hold` (`stun-hold` / gauntlet `stunhold`): skip an X creature's
  cast while `stun_counters_at_x(def, max X)` > 0 — Trudge waits for X>=3.
- `own_graveyard_picks` (`gy-pick` / gauntlet `gypick`): an optional
  "choose up to N" over the bot's own graveyard takes up to N, priciest
  first (Divergent Equation, Bind to Life). Root cause: the graveyard
  branch of `decide_choose_cards` only ever picked opponents' cards.
- Gates queued: deck seat on inc1 (both seeds) for each flag and both;
  sealed-ladder mirror `--a FLAG --b dflt`, 500 games x 12 decks, seeds
  43/97 (must not read below 49.5).

## 05:30 Phase 1/3 confirm — v4_removal under mcts256 both seats: 51.67 (±2.8, 1,200 games)
converge3 read 35.42 on the same cell: **+16.3 under the search pilot, +16.4
under the scored pilot** — the screen ranking transfers. Replay scan: flood
proxy 63 % of games reaching T8 (18 lands in 40; the lands sweep is queued);
Trudge X=0 in 239 of 405 recorded casts (the stun-hold gate is queued);
suicides 10 % of the deck's attacks (round-65 hole, unchanged).
Top WR-when-seen: Ancestral Recall 72.8, Emeritus 64.1, Dellian 61.2, Snarl
Song 60.0, Blech 59.9, Trudge 59.7, Knockout 59.2, Rancorous 58.6.

## 05:55 Phase 4 gen 1 — every legal add into inc1 for Witherbloom Charm, and +/-1 basic
Reference (the slot as a second Oracle's Restoration) = 62.44 (+1.3 over inc1: Witherbloom out is itself a gain).
Adds over the cantrip reference: Rancorous #3 +2.3 (64.73), Shopkeeper #2 +2.0,
Fractal Mascot +1.9, Thornfist Striker +1.8, Arnyn +1.2, Sundering #2 +1.2,
Spellbook Seeker +1.2, Ascendant Dustspeaker +1.1, Soaring Stoneglider +1.1,
Pensive Professor +0.9, Adventurous Eater +0.9, Tragedy Feaster +0.6.
Below the cantrip: Page -1.2, Zimone's -2.6, Hydro-Channeler -2.6, Pull from
the Grave -2.3, Emeritus of Truce -2.3, Chase Inspiration -3.3, Brain Freeze -4.9.
Lands (inc1 has 18): -Plains +0.3, -Island +0.2, +Mountain +0.6, +Forest +0.1,
-Swamp -0.5, +Swamp -0.8, +Island -1.3, +Plains -1.3, **-Forest -1.9**. Stay at 18.
- **Incumbent := inc2 = inc1 - Witherbloom Charm + Rancorous Archaic #3 (64.73).**
- Gen 2: swaps of {Shared Roots, Proctor's Gaze, Studious First-Year, Feed the
  Swarm, Together as One, Send in the Pest, Oracle's Restoration} x {Shopkeeper
  #2, Fractal Mascot, Thornfist, Arnyn, Sundering #2, Spellbook Seeker,
  Dustspeaker, Stoneglider, Pensive} on inc2 — 63 lists, both seeds.

## 05:57 Gate — `stun_x_hold` on the deck seat of inc1: **-1.4, FAILS**
stunhold 59.44 / 60.19 vs dflt 60.83 / 61.39 (seeds 43 / 97, 24,000 games a
cell). A 6/6 for {G} on turn 2 that untaps on turn 6 is worth more than the
untapped one two turns later, as the bot plays both sides. Hypothesis
refuted; flag parked with its numbers (not used for the deck's seat; the
sealed-ladder mirror cells run anyway for the record).

## 05:58 Gate — `own_graveyard_picks`: zero incidence everywhere
gypick 60.83 / 61.39 = dflt to the hundredth on inc1 (no Divergent Equation
left; Bind to Life's exactly-one pick already went through the min>=1
fallback); sealed-ladder mirrors 50.0 with every pair split on both seeds.
A correctness fix (unit-tested: flag on takes the priciest own card, flag off
takes nothing, an "exile" prompt is untouched) that no gating pool exercises.
Left OFF per the pre-registration; recommended for adoption as a correctness
change in the report. Stun-hold ladder mirrors: 50.0 (s43), 49.7 ±0.26 (s97).

## 06:26 Phase 4 gen 2 — 63 swaps on inc2 (both seeds)
Top: -Shared Roots +Shopkeeper's Bane #2 **66.73 (+2.0)**; -Studious +Shopkeeper
66.72; -Proctor's Gaze +Shopkeeper 66.46; -Together as One +Shopkeeper 66.45;
-Oracle's +Shopkeeper 66.30; -Studious +Thornfist 66.26; -Oracle's +Fractal
Mascot 66.20. Shopkeeper's Bane #2 is the add; the cheap cuts are
interchangeable within ~0.5 (Shared Roots, Studious, Proctor's, Together,
Oracle's). Bottom of the table is still +0 to -0.2 vs inc2: nothing in the
candidate set hurts, the deck is near its plateau.
- **Incumbent := inc3 = inc2 - Shared Roots + Shopkeeper's Bane #2 (66.73).**
- Gen 3: {Studious, Proctor's Gaze, Together as One, Oracle's, Send in the
  Pest, Feed the Swarm} x {Thornfist, Fractal Mascot, Arnyn, Spellbook
  Seeker, Sundering #2, Pensive} on inc3 — 36 lists, both seeds.

## 06:57 Gen 3 relaunched — the first launch ran nothing
`screen_dir.sh` skips cells already in results.txt by name, and gen 3's
swap names (`swap_<cut>__<add>`) and the lands names collided with gen 2's
and gen 1's, so every cell was "done". `deck_variants.py --prefix` added;
gen 3 regenerated as `g3_*` on inc3. Thirty minutes lost, no data
contaminated (the collided cells were never re-run or overwritten).

## 07:12 Lands on inc3 (+/-1 basic, neutral Oracle's Restoration)
+Mountain -Oracle's +0.9 (67.67, 3σ; under the 1.5 bar), +Forest +0.6, -Island
+0.5, -Plains +0.4, +Swamp -0.2, -Swamp -0.3, +Plains -0.7, +Island -0.8,
-Forest -1.4. **18 lands stand.** The Oracle's Restoration slot is the
deck's weakest card (a 19th land reads as well as it).

## 07:18 Phase 4 gen 3 — 36 swaps on inc3: the search stops
Top: -Proctor's Gaze +Thornfist Striker 68.05 (+1.3, 4.2σ), -Oracle's
+Thornfist 67.87 (+1.1), four more Thornfist swaps at 67.67 (+0.9),
-Together +Fractal 67.67. Nothing clears the pre-registered 1.5-pt bar →
**final = inc3**, written as `decks/real_build_converge4.txt`. Thornfist
Striker for Proctor's Gaze is the one swap still worth a look (held-out
field: 66.23 / 66.05 — compare with converge4's held-out cells in phase 5).
Search summary: 40.77 → 54.69 → 57.20 → 61.11 → 64.73 → 66.73 on the screen
field, five accepted steps, ~180 lists screened at 48,000 games each.

## 07:38 Phase 5 — final confirmation of converge4 (= inc3)
Held-out field 64.72 / 65.11 (converge3 38.16 / 38.79); mcts256 both seats
63.17 ±2.9 (converge3 35.42); deck_duel vs converge3 72.0 % [70.6, 73.5]
over 4,000 games. Thornfist-for-Proctor's on the held-out field 66.23 /
66.05 (+1.2, replicated, under the bar; noted in the deck header).
Session total: ~190 lists, ~9.2 million scored-pilot games, 4,800
search-pilot games, 5 accepted steps, 2 bot flags measured (both off),
1 correctness bug found (own-graveyard picks), 1 hypothesis refuted
(stun hold), 1 existing flag measured negative (converge_lands).

## 08:05 Addendum — the user's follow-up: charms, converge, X, ramp fetches (read-only analysis of the 3 x 1,200 search-pilot replay sets)
- **Converge payment leaves a colour on the table in 18 % of casts** (175 of
  959 in converge4's games, tapped state reconstructed per turn): e.g. paid
  B/U/W off Paradox Gardens(U) + Island + Swamp + Swamp + Shattered(W) with
  a Forest-less board where Paradox's G was the only green. Cause, in
  `auto_tap_for_cost_inner`'s `diverse` generic loop: a dual takes the FIRST
  fresh colour in WUBRG order it can make (Paradox → U), so the colour only
  that dual could supply (G) is lost when a basic already makes the other
  (Island → U). Fix shape: rarest-fresh-colour first (assign the colour
  with the fewest untapped sources before the common ones). Engine-side;
  gate like `smart_tap` (a seat flag), golden traces must hold with it off.
- **Ramp fetches ignore converge**: of 1,384 basics fetched onto the
  battlefield (Rampant Growth 589, Terramorphic 433, Proctor's 362), 1,187
  added a colour the board already made with no pip in hand asking for it;
  only 197 added a new colour. `rank_library_search` scores unmet demand
  then scarcity; Forest wins because G demand dominates. Fix shape: when
  the seat has converge cards, a basic whose colour the board cannot make
  yet outranks a covered colour once every pip in hand is coverable.
  Environmental Scientist's to-hand fetch is invisible in replays (no card
  id) but goes through the same ranker.
- **Terramorphic timing is fine**: 65 % cracked the turn it is played, 25 %
  the next own turn. Rampant Growth: cast the turn Studious enters (198) or
  the next own turn (207) of 619; no issue.
- **Charm modes**: Quandrix Charm's 5/5 mode was 126 of 163 own-precombat
  casts and 34 + 52 casts at the OPPONENT's end step across the two sets —
  a base-P/T pump at end of turn does nothing. The bot never plays it as a
  post-block trick: `is_combat_trick`/`pick_combat_trick` accept only
  constant `PumpPT`, not `SetBasePT`, and the temporary-leaf skip
  (`contains_temporary_leaf`) only removes the outcome eval, leaving the
  static score to cast it when nothing else is castable. Witherbloom: 181
  destroy-MV<=2, 139 gain-5 (Blech/Pest synergy makes that a real mode).
  Fix shape: temporary-leaf modes are combat-window-only (extend the trick
  picker to SetBasePT as "pump to 5/5"), never end-step candidates.
- **X**: Trudge's hold lost (-1.4); Divergent's zero return is the fixed
  `own_graveyard_picks`. No further X lever with evidence.

## 08:20 The converge trio built behind flags (user: "build the fixes")
- `converge_rarest` (seat flag `Player::converge_rarest`, pushed like
  `smart_tap`): the diverse auto-tap's generic loop assigns a dual the fresh
  colour with the fewest other untapped sources, recomputed per pip.
- `converge_fetch`: `rank_library_search` puts a colour the board cannot
  make first once every pip in hand is covered, for a seat holding converge
  cards anywhere (`seat_wants_converge`).
- `trick_modes_combat_only`: an instant's until-end-of-turn stat mode
  (`contains_temp_stat_leaf`, not bounces) leaves the main-phase / end-step
  enumeration; `pick_combat_trick` gains modal instants and the base-P/T
  shape (delta = new base − printed base).
- Profiles: `conv-rarest`, `conv-fetch`, `trick-modes`, `conv-fixes`
  (ladder) / `convrarest`, `convfetch`, `trickmodes`, `convfixes`,
  `mcts256-convfixes` (gauntlet). Unit tests: the five-land Rancorous cast
  pays 3 colours off / 4 on; the fetch prefers Plains/Swamp over Forest;
  the charm leaves the main phase and is cast on a blocked bear after blocks.
- Gate (`phase6.sh`, `results6.txt`): converge4 with all four profiles and
  v4_removal (both charms) with the trick flag, dflt reference cells on the
  same binary, seeds 43/97; sealed-ladder mirrors for the four profiles.

## 16:12 Gate — the converge trio (results6.txt; deck seat vs dflt opponents, 24,000 games a cell)
| flag | converge4 s43 / s97 (ref 66.36 / 67.10) | v4_removal s43 / s97 (ref 57.25 / 57.14) | sealed mirrors s43 / s97 |
|---|---|---|---|
| convrarest | 66.40 / 67.15 (+0.05) | — | 50.0 / 50.0 (4 and 1 non-split pairs) |
| convfetch | 66.71 / 66.88 (+0.06, signs disagree) | — | 50.0 / 50.0 |
| trickmodes | 66.36 / 67.10 (identical: no charm) | **60.87 / 60.70 (+3.6)** | 50.0 / 50.2 ±0.30 (87 non-split) |
| convfixes | 66.75 / 66.91 | 60.91 / 60.71 | 50.0 / 50.2 |
- **`trick_modes_combat_only` ADOPTED into `default_const()`** (round 66):
  +3.6 on both seeds on the list that plays the charms, identical where
  nothing fires, ladder not below 49.5. Control `trick_modes_off` /
  `trick-modes-off` / gauntlet `trickoff`. Re-measured on the flipped
  binary in results7.txt (new default should read the flag-on numbers;
  the off control the old ones). Cutting Quandrix Charm was +3.9 under the
  old timing — the charm was fine, the bot's windows were the problem.
- `converge_rarest`: correct (unit test: 4 colours off the five-land board
  vs 3) and worth +0.05 — kept as an opt-in correctness fix.
- `converge_fetch`: +0.06 with the seeds disagreeing — null, off.

## 16:16 The flipped default re-measured (results7.txt)
v4_removal: new default 60.99 / 60.86 vs `trickoff` control 57.50 / 57.45
(+3.5 / +3.4 — the flag now on both seats). converge4: 66.36 / 67.10,
identical to the hundredth. Sealed mirror `trick-modes-off` vs dflt: 50.0
(all split) / 49.8 ±0.30. Golden traces 7/7 unchanged under the new
default; 255 bot tests pass; release-fast typecheck clean.
