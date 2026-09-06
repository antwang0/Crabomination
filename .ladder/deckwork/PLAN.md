# Deck work: improving `decks/real_build_converge3.txt` against deep-pool bots

Pre-registered 2026-09-06 before any game was played. Base commit f250eb488
(`claude/modern_decks`, in sync with origin at the time of writing).

## The question

"Which changes to my converge build make it win more against bot-built decks
from 9-20 pack pools, as the bot pilots it?" Both seats are piloted by the
same bot, so every number is a *deck* number; the pilot's known holes (PERF /
ML_NOTES round 65: greedy attack suicides, redealt-hand tricks) hit both seats.

## Starting facts (read before the plan, not measured)

* The deck is **46 cards: 20 lands, 26 spells**. Sealed minimum is 40.
* Colour sources: G 8, U 8, B 6, W 6, R 3 (Terramorphic counted once each).
  8 of 20 lands enter tapped or slow (2 Forum, 1 Fields, 2 Paradox, 2 slow
  lands, Terramorphic).
* Converge payoffs: 2 Rancorous Archaic (5), 2 Sundering Archaic (6),
  Together as One (6), Snarl Song (6), Arcane Omens (5). Eight cards at 5+
  mana with Emeritus (5); plus Slumbering Trudge (X).
* The bot's `converge_lands` (colour-diversifying land drops) is **off** in
  the default profile; the deck is exactly what it was written for.
* The pool (99 cards) holds candidates the deck does not play: 2 Fractal
  Mascot, Vastlands Scavenger, Tragedy Feaster, Thornfist Striker,
  Essenceknit Scholar, Witherbloom Charm, Pest Mascot, Embrace the Paradox,
  2 Oracle's Restoration, Spellbook Seeker, Efflorescence, Burrog Barrage,
  Wild Hypothesis, Follow the Lumarets, Melancholic Poet, Arnyn, Poisoner's
  Apprentice, Adventurous Eater, Pull from the Grave, Muse's Encouragement,
  2 Page, 2 Hydro-Channeler, Vibrant Outburst, Dissection Practice.

## Instruments (Phase 0 builds them; nothing else changes engine behaviour)

* `deck_gauntlet` (new bin): one decklist vs a **fixed field** of
  `random_sealed_opponent_packs(seed, packs)` decks (best-of-16 v3 builds
  from an N-pack pool). Antithetic seat pairs per opponent, paired estimate
  pooled over the field, per-pack-count breakdown. Two paths:
  - fast: the in-process loop `simulate_match_pairs_piloted` (no replays);
  - `--replays DIR`: the server's `run_match` (the only path that writes
    `CRAB_REPLAY_DIR` replays), one GAME line per game in the results file
    so an aborted run resumes and a replay maps back to its game.
* `scripts/replay_scan.py`: per-game and per-card report over a replay
  directory: mana development, cast rates and cast turns, drawn-in-game
  win rate per card, converge X per payoff cast vs colours paid, Trudge X,
  attack outcomes (the round-65 suicide classifier), life trajectory.
* Field: **24 opponents** = packs {9, 12, 16, 20} x 6 seeds each, seed
  `0xDECC0000 + packs*1000 + i`. Fixed for the whole session; every variant
  faces the same 24 decks. Their lists are dumped to `field/` once.
* Pilots: `dflt` = `EvalWeights::default()` scored bot (the rollout policy,
  fast) for screening; `mcts256` = the lobby pilot (`MctsBot` 256 / horizon
  3, net-free) for confirmation and replays. `convlands` variants = the same
  with `converge_lands` on.

## Cells and rules

* **Screen cell**: `dflt` both seats, 100 pairs x 24 opponents = 4,800
  games, seed 43. Expected precision ~ +/-1.0 pt paired.
* **Confirm cell**: `mcts256` both seats, 25 pairs x 24 = 1,200 games,
  seed 43, replays on, 12 threads (the server's 15 s bot watchdog aborts
  the process under `panic = abort`; fewer threads keep a searched
  decision well under it).
* **Adoption bar** for a variant over the incumbent (same binary, same
  field, same seed): delta >= 1.5 pts AND >= 2 x se(diff) on the screen
  cell; then a second screen seed (97) must agree in sign with pooled
  delta >= 1.5. A variant that passes replaces the incumbent for the next
  round. Ties keep the incumbent (fewer changes from the user's list).
* **Bot changes** stay behind flags. A flag is used for the deck's seat only
  if (a) it clears the adoption bar on the deck gauntlet and (b)
  `bot_ladder --a FLAG --b dflt --decks sealed --games 500 --seed 43` and
  `--seed 97` do not read below 49.5 (no general regression). Changing the
  adopted default is out of scope here and is recorded for the ML program.
* A rebuilt binary invalidates every earlier cell: re-run the incumbent's
  screen cell before comparing anything to it.
* No `--release` builds; `release-fast` for every game binary; never two
  engine builds at once (check `pgrep -f rustc` first).

## Phases (estimated wall clock; total 7-9 h)

0. **Tooling (40 min)**: write `deck_gauntlet`, `replay_scan.py`; build
   `release-fast`; smoke (1 opponent x 3 pairs, both paths, both pilots);
   dump the field.
1. **Baseline (40 min)**: converge3 screen cell (seed 43 and 97); converge3
   confirm cell with replays; the recommender's W/B list
   (`decks/recommended_from_pool.txt`) screen cell as a yardstick only —
   the deliverable stays a converge deck.
2. **Diagnosis (45 min)**: scan the replays. Questions, in order: mana
   (screw/flood, tapped-land tempo); cast rates of the 5-6 drops; converge
   X actually paid vs colours available (a payment bug is a bot fix);
   Trudge X; stuck cards; combat suicides on the deck's seat; per-card
   drawn-WR ranking. Then gate `convlands` vs `dflt` on the screen cell.
3. **Hand-made variants (2 h)**, one idea each, all cut to 40 unless noted:
   - V1 trim-to-40: cut Divergent Equation, Arcane Omens, Traumatic
     Critique, 1 Sundering Archaic, Mountain, Fields of Strife
     (18 lands / 22 spells; four colours).
   - V2 lands: V1 at 17 lands (cut Plains) / V1 at 18 with the Mountain
     back instead of a Forest (5-colour converge kept).
   - V3 bombs: V1 + 2 Fractal Mascot for Snarl Song and Send in the Pest.
   - V4 removal/lifegain: V1 + Witherbloom Charm, Vastlands Scavenger for
     Pensive Professor and Zimone's Experiment #1.
   - V5 curve: V1 + Thornfist Striker, Essenceknit Scholar, Pest Mascot
     for Together as One, Snarl Song, 1 Rancorous Archaic (converge-light).
   - V6 the 46-card list at 17 lands (only lands cut: is the size the
     problem or the spells?).
   - V7 Trudge test: V1 without Slumbering Trudge, +Oracle's Restoration.
   Each screened at seed 43; the top two by delta re-screened at 97.
4. **Local search (3 h)**: from the incumbent, every single swap (one deck
   card out, one pool card in, colour-legal) plus +/-1 land, screened at
   seed 43 in batches of ~20; the best passes the two-seed bar or the
   search stops. Up to three generations.
5. **Confirmation and report (1 h)**: final vs converge3: confirm cell
   with replays (both lists), `deck_duel` head-to-head 2,000 pairs, replay
   scan of the final for regressions (suicides, stuck cards). Write
   `decks/real_build_converge4.txt` (proposal; the user's file is untouched)
   and `.ladder/deckwork/REPORT.md`; commit tooling + report; push after a
   fetch/rebase.

## What this cannot tell the user

Numbers are "as the bot plays both decks" against bot-built decks; a human
piloting converge differently (X choices, holding tricks) moves them. The
field is best-of-16 builds from deep pools: consistent two-colour aggro
decks, so results lean toward consistency over ceiling.
