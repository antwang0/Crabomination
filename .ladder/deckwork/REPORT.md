# Deck work report: `real_build_converge3.txt` against deep-pool bots

2026-09-06. Pre-registration in `PLAN.md`, the run log in `JOURNAL.md`,
every cell in `results.txt` (`RESULT` lines; `scripts/gauntlet_summary.py`
reads them). Base commit f250eb488 plus this session's tooling.

## The number that matters

| list | scored pilot, 24k games/seed, seeds 43+97 | 256-search pilot both seats, 1,200 games |
|---|---|---|
| converge3 (the user's 46-card list) | 40.77 | 35.42 ± 2.8 |
| v4_removal (40 cards) | 57.20 | 51.67 ± 2.8 |
| **converge4 = inc3 (the proposal)** | **66.73** | **63.17 ± 2.9** |

Opponents: 24 fixed decks, six each from 9, 12, 16 and 20 booster pools,
each the best-of-16 build the client's `--opponent-packs` sealed mode uses.
Both seats piloted by the same bot; antithetic seat pairs; the paired
estimate is what the table shows.

## What was wrong with the list, in order of size

1. **46 cards.** Trimming to 40 by cutting Divergent Equation, Arcane Omens,
   Traumatic Critique, one Sundering Archaic, the Mountain and Fields of
   Strife is +13.9 on its own (v1_trim40, 54.69). The lands alone are only
   ~2 of that (v6: all 26 spells at 17 lands, 43.01); the four spells are
   the rest. In 1,200 search-pilot replays of converge3, Divergent Equation
   returned 0 cards in every one of 221 casts and had the lowest
   win-rate-when-drawn of any card (32.6 %); Arcane Omens was cast in 34 %
   of the games it was drawn.
2. **The two charms.** Cutting Quandrix Charm for a cantrip is +3.9; cutting
   Witherbloom Charm +1.3 to +1.7. The bot plays 123 of 130 Quandrix casts
   as the 5/5 mode at mean turn 12.9 and counters once. Cheap modal instants
   are the bot's known blind spot (the redealt hand in its sims), so these
   are "as the bot plays it" findings; a human may keep one.
3. **Missing bodies.** Over the same slot as a cantrip: Rancorous Archaic #3
   +2.3, Shopkeeper's Bane #2 +2.0, Fractal Mascot +1.9, Thornfist Striker
   +1.8, Arnyn +1.2, Sundering #2 +1.2, Spellbook Seeker +1.2, the two white
   fliers +1.1 each. Vastlands Scavenger (in since v4) is -2.1 when cut.
4. **Lands.** 18 in 40 is right: -Plains/-Island +0.3, +Mountain +0.6 (not
   significant), +Island/+Plains -1.3, **-Forest -1.9**.

What is *not* wrong: Slumbering Trudge (-6.4 when cut), Emeritus (-6.1),
Snarl Song (-4.2), Dellian Fel (-3.2), Rancorous (-3.1), Blech (-3.0),
Shopkeeper (-2.6). The converge top end is the deck; v5 (three cheap
creatures for Together as One, Snarl Song and a Rancorous) is -5.7 vs v1.
Fractal Mascots for Snarl Song and Send in the Pest (v3) is -1.4 vs v1.

## The search

| step | change | win% | delta |
|---|---|---|---|
| converge3 | — | 40.77 | |
| v1_trim40 | six cuts to 40 | 54.69 | +13.9 |
| v4_removal | v1 +Witherbloom Charm +Vastlands Scavenger -Pensive Professor -Zimone's Experiment | 57.20 | +2.5 |
| inc1 | -Quandrix Charm +Oracle's Restoration | 61.11 | +3.9 |
| inc2 | -Witherbloom Charm +Rancorous Archaic #3 | 64.73 | +3.6 |
| inc3 | -Shared Roots +Shopkeeper's Bane #2 | 66.73 | +2.0 |
| gen 3 | best -Proctor's Gaze +Thornfist Striker | 68.05 | +1.3, under the bar: stop |

Each step cleared the pre-registered bar (>= 1.5 pts and >= 2 se, both
seeds agreeing). Gen 2's runner-up (-Studious First-Year +Shopkeeper) was
0.01 behind the winner: the cheap cuts are interchangeable within ~0.5.

## Bot findings (the "fix the bots" part)

- **Converge payment is correct.** Rancorous X 3-4 (mean colours paid
  3.5-3.8), Snarl Song / Together 4-5; the engine tapped W-U-B-R-G for X=5
  when it could. Nothing to fix.
- **`converge_lands` (the colour-diversifying land drop, off by default) is
  -3.5 on the deck it was written for** (36.85/37.60 vs 40.55/41.00). Not a
  fix; a measured negative for the ML program.
- **Slumbering Trudge X.** 44 % of casts are X=0 on turn 2-3. A flag that
  holds it for X>=3 (`stun_x_hold`, profile `stun-hold`) reads **-1.4**
  (59.44/60.19 vs 60.83/61.39): the early tapped 6/6 is right as the bot
  plays both sides. Parked with its numbers in the flag's doc.
- **Own-graveyard picks.** `decide_choose_cards`'s graveyard branch only
  ever took opponents' cards, so an optional "choose up to N" over the
  bot's own graveyard (Divergent Equation; Bind to Life's "put a creature
  from among them") moved nothing. Fixed behind `own_graveyard_picks`
  (profile `gy-pick`, unit-tested). Zero incidence on every gating pool
  (gauntlet identical to the hundredth, sealed-ladder mirrors 50.0 with
  every pair split) — a correctness fix with no measured win, left off per
  the pre-registration; recommended for adoption as a correctness change.
- **Suicide attacks** are 10-11 % of attacks on BOTH seats under the 256
  search (round-65 hole; the guard was a search-level loss). Unchanged.

## Follow-up: charms, converge payment, ramp fetches (built after the report)

Three flags, gated the same way (`results6.txt`, journal 16:12):

| fix | where it fires | gauntlet | verdict |
|---|---|---|---|
| `trick_modes_combat_only`: an instant's stat mode waits for the combat window; the trick picker learns modal instants and base-P/T | the list with both charms | **+3.6** (60.87 / 60.70 vs 57.25 / 57.14) | **adopted into the default** |
| `converge_rarest`: a dual pays the fresh colour with the fewest other sources | 18 % of converge casts | +0.05 | correct, opt-in |
| `converge_fetch`: a new colour outranks a covered one once hand pips are covered | 86 % of ramp fetches | +0.06, seeds disagree | null, off |

So the charm finding reverses: Quandrix Charm was a liability only because
of the bot's timing. With the trick window fixed, a list that keeps the
charm is back within reach; converge4 itself is unchanged by any of the
three (no charm, and the extra converge colour is worth ~0).

## Final confirmation

| cell | converge4 | converge3 |
|---|---|---|
| screen field, scored pilot (seeds 43 / 97) | 66.36 / 67.10 | 40.55 / 41.00 |
| **held-out field** 0xDECC1000, scored pilot (43 / 97) | 64.72 / 65.11 | 38.16 / 38.79 |
| 256-search pilot both seats, 1,200 games, replays | 63.17 ± 2.9 | 35.42 ± 2.8 |
| `deck_duel` converge4 vs converge3, 2,000 antithetic pairs | **72.0 % [70.6, 73.5]** | |

The held-out field gives back 2 of the 26 screen points — the search fitted
its 24 decks a little, not a lot. The search pilot's gap (+27.8) matches the
scored pilot's (+26.0). Head to head, the proposal beats the current list
72-28.

Regression scan of the final's 1,200 search-pilot replays: <4 lands after
own turn 4 in 17 % of games (30 % for converge3); flood proxy unchanged
(62 %); converge X 3-4 as before; Trudge still cast for X=0 in 59 % of its
sub-3 casts (the hold was measured and lost); suicides 10 % of the deck's
attacks and 12 % of the opponents' — the round-65 hole, both seats.
Top WR-when-seen: Ancestral Recall 74, Emeritus 71, Blech 70, Trudge 70,
Dellian 70, Knockout 70, Vastlands 70.

**The one swap under the bar worth knowing:** -Proctor's Gaze +Thornfist
Striker is +1.3 on the screen field (4.2σ) and +1.2 on the held-out field
(66.23 / 66.05 vs 64.72 / 65.11) — replicated, below the pre-registered
1.5. It is noted in the proposal's header, not applied.

## Rerun under the new default (`r2/`)

With trick modes held for combat, the search was rerun from converge4 with
the charms back in the candidate pool (`scripts/deck_search.py`, the same
bar). Nothing clears it: the best change is once more Thornfist Striker for
Proctor's Gaze at +1.3 (both seeds, third replication), and 18 lands stand.
Quandrix Charm for Studious First-Year now reads −0.3 — a fair card, no
longer a liability — and Witherbloom Charm −2.3 (its modes are not tricks).
The original 46-card list gains +2.8 from the timing fix alone (43.6 on the
screen field). Confirmation of converge4 on this binary: held-out 64.72 /
65.11 vs converge3's 40.77 / 41.16; search pilot 61.7 ± 2.9; head to head
**69.7 %** [68.3, 71.1] (72.0 earlier was under the old default). So
`decks/real_build_converge4.txt` is the answer under both defaults.

## The bot was aiming its own spells at itself (r3, the same evening)

The user asked why Oracle's Restoration survives while Arcane Omens,
Divergent Equation and Traumatic Critique were cut. The replays answered:
the auto-target picker that fills a spell's slots tried the **caster first
for every player slot**. Right for "target player draws"; wrong for discard
and damage. Both pilots build their candidate list through it, so the
search never saw the other player.

| card (converge3, 1,200 search-pilot games) | what happened |
|---|---|
| Arcane Omens, 113 casts | the caster discarded in 98, the opponent in 0 |
| Traumatic Critique, 201 casts | X to the caster's own face every player-aimed cast (455 damage) |
| Together as One, 151 casts | X to own face every cast (616 damage), offset by the X life |
| Divergent Equation, 439 casts | 286 at X=0; the pick returned nothing (flag off) |

The deck seat hit its own face with a spell 376 times and self-discarded 344
times; the 24 opponent decks did it 12 and 34 times. Every earlier verdict
on those cards, and on Together as One (−0.15 when cut), was a verdict on
the bug. Two more findings on the way: the committed pool file lacked five of
the user's own cards (Arcane Omens among them, so it was never offered as an
add), and Bind to Life put a creature onto the battlefield in ~5 of 86 casts.

**Fixed and adopted (round 67)**, gated on the deck seat vs the field and
on the sealed mirrors:

| flag | converge3 | converge4 | sealed |
|---|---|---|---|
| picker aims hostile player slots at the opponent (seat flag) | 51.26 / 51.79 vs 43.20 / 43.95 (+8.0) | +2.0 | 50.4 / 50.5 |
| the other player as a cast-time arm | +4.8 alone, −0.3 beside the flag | −0.3 | **51.7 / 51.7** |
| own-graveyard pick + X=0 prune | +1.1 | identical | 50.0 |
| all four (the new default) | **52.14 / 52.65** | **68.24 / 68.84** | 51.7 |

Search pilot on converge3, both seats: **48.42 ±1.5 vs 39.17 ±1.5**. 

## The search under the fixed bot (`r3/search/`, `r3/search4/`)

From the user's 46-card list (card-for-card swaps only, so the count stays
46): the cheapest cuts are now Zimone's Experiment (+2.9 for a cantrip),
Divergent Equation (+2.3), Arcane Omens (+1.6), Traumatic Critique (+0.5) —
still below a one-mana cantrip in this deck as the bot plays it, but by a
third to a half of what the bug made them look. Together as One is no
longer a cheap cut. Its first move was the third Rancorous Archaic for
Zimone's Experiment (+5.0), the first pass's direction; it was stopped
there in favour of the question that matters.

From converge4 (40 cards) with Arcane Omens back in the pool: nothing clears
the bar. Cuts are flat (the five cheapest within 0.2 of the incumbent);
Thornfist Striker for Send in the Pest is +1.2 (69.65 vs 68.48); Arcane
Omens for that slot is under +0.1; a nineteenth land (a Mountain) for
Oracle's Restoration is **+1.4** (69.89), the closest miss of the four
passes. So `decks/real_build_converge4.txt` stands under the fixed bot too.

| cell, fixed bot | converge4 | converge3 |
|---|---|---|
| screen field, scored pilot (43 / 97) | 68.21 / 68.76 | 52.12 / 52.65 |
| held-out field 0xDECC1000 (43 / 97) | 66.24 / 66.62 | 49.09 / 49.71 |
| 256-search pilot both seats, 1,200 games | 66.58 ± 2.8 | 48.42 ± 2.9 |
| `deck_duel` converge4 vs converge3, 2,000 pairs | **60.3 %** [58.8, 61.8] | |

The original list gains the most from the fix (it holds all four affected
cards), so the head-to-head narrows from 69.7 to 60.3 and the field gap from
26 to 16 points. In the final replays Together as One is the deck's best card
by win rate when seen (73.6 %, X = 4 in 187 of 261 casts); Oracle's
Restoration is second (71.0 %) as a one-mana cantrip in a deck that now plays
its top end. One residue for the next session: under the 256-search pilot,
Together as One's draw still goes to the opponent in 24 % of its cards (228
of 963) — the search taking the offered arm, where the scored pilot takes it
in 2 %.

## Caveats

- Every number is "as the bot plays both decks" against consistent
  bot-built decks from deep pools. A human converge pilot moves the charm
  and X findings most.
- The local search compared ~170 lists on one field; the held-out field
  in the confirmation is the guard against fitting those 24 decks.
- The scored pilot is the screen; the 256-search pilot confirmed the
  first 16-point step exactly (+16.4 vs +16.3) and re-checks the final.

## Tooling added (committed)

- `crabomination/src/bin/deck_gauntlet.rs` — a list vs a fixed deep-pool
  field, fast path or replay-recording server path.
- `scripts/replay_scan.py` — per-game / per-card / converge / combat report
  over a replay directory.
- `scripts/deck_variants.py` — cuts / adds / swaps / lands variant generator.
- `scripts/gauntlet_summary.py` — pooled ranking with deltas vs an incumbent.
- `.ladder/deckwork/` — plan, journal, results, field lists, variant lists,
  runner scripts.
