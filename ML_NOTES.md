# ML notes — bot/net experiment history

Long experiment narratives: what was gated, what was adopted, and the dead
ends. Moved here verbatim from `FEATURE_ROADMAP.md` "Tier 13 — AI" so nobody
re-derives a refuted hypothesis. Never summarize an entry away — a dead end
only stays dead while the reasoning that killed it is readable.

## Tier 13 — AI

- 🟡 **Smarter combat** — `server/bot.rs` blocking is heuristic (value trades,
  first-strike/deathtouch/trample/**indestructible** awareness — an
  indestructible body walls the biggest attacker for free and an indestructible
  attacker can't be cleanly traded — gang-block-to-survive, **and
  chump-blocking to save a planeswalker we control when its attackers are
  lethal to its loyalty — the life-threat calc counts only player-bound
  damage**); attacking has a suicide filter + evasion awareness + planeswalker
  redirection. Value-ping removal also aims an "any target" ping at an opponent's
  face when that hit is exactly lethal (reach for the win). The bot crews
  Vehicles (`pick_crew`) **and now saddles Mounts** (`pick_saddle`) before
  combat so attacks-while-saddled riders fire. **Attacking into open mana is
  now respected**: the adopted default (`attack_search_sim`) lets both seats
  cast spells inside the attack/block simulations, so the crack-back removal
  and the defender's tricks are visible at declaration time — adopted at
  54.4 % [53.0, 55.8] over 4 794 fixed+cube games with dimir control (the
  blind search's documented −5.2 archetype) the biggest winner at 61.3 %.
  Race math (`atk-race`: an attack sim that ends inside burn range, any
  life ≤ 10, extends one turn cycle) is measured and **not adopted**:
  the pre-registered 4× decision run read 50.2 % [49.5 %, 51.0 %] over
  19 200 fixed+cube games — the first decider's +1.2 (with mono-red at
  54.8 %/400) collapsed to +0.2, mono-red back at 49.9 %, the
  block-search replication failure reproduced on a fresh hypothesis.
  Kept as a documented profile (`attack_search_race` doc).
  **Multi-blocker math landed and is adopted**: value gang-blocks
  (`gang` / `block_gang`, now `EvalWeights::default`). The greedy pass
  gangs only under lethal threat and `block_search` could only ever
  *remove* blockers, so "two 2/2s eat a 4/4 at 20 life" was in no search
  space the bot had; gangs are now candidates the block sim prices (dead
  blockers against dead attacker). Two independent 28 800-game sealed
  runs: 51.3 % [50.7, 51.9] and 51.1 % [50.5, 51.7], after a 9 600-game
  screening at 51.0 % — the only one of five play-side profiles tried in
  this push whose edge did not shrink at 3× the sample. Adopting it also
  switches on `block_search`, which measured null alone: the search had
  nothing to find while its only candidates were "block with one fewer
  creature".
- 🟡 **Planeswalker piloting** — emblem values are priced by what the
  emblem actually does (draw/damage/drain/token/lifegain shapes, static
  buffs, clamped 20–60) instead of a near-flat constant, and a doomed
  walker cashes out: when enemy board power covers its loyalty, the
  ability pick keeps only loyalty-spending finalists, so the bot takes
  the removal/ultimate now rather than plussing into a free kill.
- 🟡 **Per-card attribution is now within-archetype**
  (`CardAttribution::stratified_delta`). The raw in-minus-out delta is a
  *cross-archetype marginal*, not a card grade: a black card is played by
  the black builds and benched by every white, blue and red one, so its
  "out" group is a different deck rather than the same deck without it.
  That is what made Professor Dellian Fel read −2.4. Attribution is now
  pooled across colour-identity strata by inverse variance, and a card no
  stratum both plays and benches reports **no** within-archetype number
  rather than passing the marginal off as one. `recommend_pool` prints
  `within` first and labels `raw` as the confound.
- 🔴 **Play net as an evaluator: documented dead end.** Ten gate rounds
  across every lever available, and it has never won. Recorded here so
  the next person does not re-derive it.

  The strange part is that it is now the *better predictor* and still the
  worse player. On identical fresh positions the attention net scores AUC
  0.798 / log-loss 0.551 against `eval_material`'s 0.760 / 0.574, and it
  replicates on a second seed (0.761 vs 0.747). Then:

  | profile | win rate vs `gang` |
  |---|---|
  | `net` (replacement) | 44.8 % [43.7, 45.9] |
  | `net-blend` | 48.8 % [47.8, 49.8] |
  | `net-q10` / `net-q20` | 44.4 % / 44.4 % |
  | `netb-q10` / `netb-q20` | 48.0 % / 48.9 % |

  **Three explanations proposed, two tested, both refuted:**

  1. *"AUC is global, the search needs local discrimination, so the net
     must be worse locally."* `--pairwise` says no — on adjacent same-game
     snapshots the net orders 54.3 % of separated pairs correctly against
     the heuristic's 51.7 %. It is slightly **better** locally.
  2. *"The net manufactures differences: it separates 100 % of adjacent
     pairs where `eval_material` ties on 46.9 %, so an argmax search
     follows its noise."* Quantising the output onto a 0.1 / 0.05 grid
     makes it tie exactly like the heuristic — and moves the win rate by
     less than a point in either direction. Refuted.
  3. *Untested:* **distribution mismatch.** Every diagnostic samples the
     snapshot cadence (turn start / postcombat main / end step), but the
     search evaluates *simulated leaves* inside `simulate_attack_outcome`
     — a distribution the net is neither trained on nor measured at. A
     net better on snapshots and worse on sim leaves would produce
     exactly this pattern, and every instrument built so far would show
     the former while the gate measures the latter. Testable by pulling
     calibration positions from inside the search.

  Levers already exhausted: data volume, window reuse, capacity (round 4),
  snapshot coverage, target shape (MC → TD(λ)), architecture (pooling →
  attention), and output shaping (quantisation). Making the net a
  strictly better predictor did not make it a better player at any point.

  **Where the evidence points instead:** the *deck* net, which clears the
  house bar (61.7 %, 60.7 %) and is under-exploited. A decklist genuinely
  is an unordered set, so bag-of-cards is the right prior; a board state
  is a set of matchups, so it is the wrong one. Same architecture,
  opposite verdicts — see [`selfplay_train --use-deck-best`] and
  `CRAB_DECKNET=… recommend_pool`.
- 🟢 **Value net finally beats the heuristic as a predictor** — and the
  reason six gate rounds failed was **overfitting, not architecture**.

  `selfplay_train --calibrate N` scores the net and `eval_material` as
  *predictors of the winner* on identical fresh positions (log-loss /
  Brier / AUC, plus an output histogram). It answered in minutes what
  thousands of gate games never did:

  | | pooled λ0.7 3.7M rows | pooled λ0.7 9.1M rows | **attention** 9.1M rows | `eval_material` |
  |---|---|---|---|---|
  | AUC | 0.7369 | 0.7805 | **0.7978** | ~0.753–0.761 |
  | log-loss | 0.7473 | 0.5912 | **0.5505** | ~0.571–0.574 |
  | Brier | 0.2384 | 0.2007 | **0.1859** | ~0.196–0.197 |
  | outside [.05,.95] | — | 16.1 % | **9.8 %** | — |

  Decomposed: **data volume + lower window reuse is worth +0.044 AUC**
  (2.5× more fresh games at 1.68× reuse instead of 4.16×), and
  **attention adds +0.017 on top**. Overfitting was the dominant effect
  and the architecture the smaller half — the reverse of the working
  hypothesis.

  This retro-invalidates the earlier gates: all six trained a ~481 k-param
  net on a memorised 250 k window at 4.2× reuse, so the 42–45 %
  replacement results measured an overfit net, and round 4's "capacity is
  the bottleneck" conclusion came from a run that could not have shown a
  capacity effect. **Training MSE is not progress here** — 0.017 at λ=1
  was memorisation, and out-of-sample log-loss was *worse than predicting
  0.5 every time* (1.1210 vs 0.6933).

  Two failures, separately fixed. *Calibration*: MSE on hard 0/1 targets
  rewards large logits, pinning 70 % of positions in the extreme bins and
  handing the search a flat landscape where every candidate line scores
  the same — a better ranker made into a worse player by the shape of its
  output. Soft TD(λ) targets plus more data cut that to 9.8 %.
  *Knowledge*: fixed by data volume first, attention second.

  Caveats before anyone invests: single seed (replication running), and
  AUC is not win rate — better prediction still has to survive the ladder.
- 🟡 **Value-net rework** — three changes, none yet gate-measured:
  bootstrapped **λ-returns** (`SampleWindow::relabel_lambda`, shard v3
  carries trajectory + ply; λ = 1 reproduces the historical Monte Carlo
  target exactly, so every prior gate round stays reachable), because
  labelling a turn-2 state with the winner of a twenty-turn game is
  mostly labelling noise; **opening-move exploration** in
  `play_recorded_game`, because both seats played the same deterministic
  policy and the net only ever saw the band of positions that policy
  reaches; and `--calibrate`, which scores the net and `eval_material` as
  *predictors* (log-loss / Brier / AUC, plus an output histogram) on
  identical positions. Four gate rounds answered "is the net-piloted bot
  stronger" expensively without ever answering "does the net know more
  than the heuristic does" — and those have different fixes. A saturated
  sigmoid would make a better predictor into a worse player by handing
  the search a flat landscape, which the histogram is there to catch.
- 🟡 **Build net has a consumer** (`selfplay_train --use-deck-best`). The
  deck net cleared the house bar twice (61.7 %, 60.7 %) and nothing read
  the result back: every training game was still played with heuristic
  builds. Actors can now judge best-of-32 candidates with it.
- 🟡 **Sealed builder repaired** (`SimConfig::builder_v2`, the previous
  builder kept as the control) — three defects found together while
  investigating why a pool's bomb never appeared in a build: the card
  scorer had **no body, keyword or ability terms at all** (it ranked a
  {3}{U}{U} 5/5 flier with ward 2 *below* a vanilla {U}{U} two-drop),
  splash candidates weren't pip-limited (so a double-pip bomb got
  "splashed" off three sources), and basics were split by linear pip
  demand (so double costs were under-served). Now: `draft::card_quality`
  (body, evasion/deathtouch/lifelink/ward, and a `prepare_spell` bonus —
  a preparation card is two cards in one slot), single-pip splashes, and
  squared pip demand in the mana split. **Adopted**: 56.9 %
  [54.1, 59.7] and 58.5 % [55.7, 61.3] on independent seeds over 1 200
  head-to-head games each vs the builder it replaces, same pools and
  pilots (`selfplay_train --gate-builder-v2`).
- 🟢 **Paired ladder sampling** (`bot_ladder --paired`, the default;
  `--unpaired` is the control). Each shuffle is played twice with the
  seats swapped, so deal luck *cancels within the pair* instead of being
  averaged away across thousands of games. Under a true null 2 032 of
  2 400 sealed pairs split — a direct measurement that only ~13 % of this
  ladder's games were ever decided by anything a profile could influence.
  Realized within-pair correlation −0.63 … −0.74, so 14 400 paired games
  carry the precision of ~35 000–40 000 unpaired ones; the efficiency is
  measured and printed, not assumed. Also seeds the bot's tie-break
  jitter (`bot::set_jitter_seed`) — `--seed` never made a run
  reproducible before, and under a null that jitter was the only thing
  that could break a pair (rho −0.694 → −0.735). The residual is
  engine-level randomness inside card effects.

  Re-measured at ~4× resolution against the current default, 14 400
  games each (seed 43): `landseq2` 50.4 % [49.9, 50.9], `mull2` 49.8 %
  [49.3, 50.3], `look1` 50.4 % [49.9, 50.9] — three nulls **confirmed**
  rather than overturned, which is the useful outcome: those rejections
  were correct, not underpowered. `race2` 49.3 % [48.8, 49.9] is the one
  reversal, mildly *harmful* where the unpaired run read 50.2 %.
  `look2` (two plies of sequence lookahead) read 50.6 % [50.1, 51.1] on
  seed 43 and **did not replicate**: 50.1 % [49.6, 50.7] on seed 97,
  pooling to 50.4 % [50.0, 50.7] over 28 800 games. Not adopted. Note
  what the paired ladder bought even here — the first seed's edge was
  identified as unreplicated at 14 400 games rather than 60 000.
- 🟡 **Castability-aware mana payment** (`Player::smart_tap` /
  `GameState::source_redundancy`, `EvalWeights::legacy_tap` as the
  control) — auto-tap paid generic pips by activation-cost rank with
  *battlefield order* as the tiebreak, so casting `{2}{B}` off 8 Swamp /
  6 Forest / 3 Island would tap an Island and strand the blue cards the
  splash exists to cast. Generic pips now spend the most replaceable
  source (a Swamp with 7 backups before an Island with 2) and coloured
  pips the narrowest one (a basic before a dual). It never changes
  whether the *current* cost can be paid, only which of several
  interchangeable sources pays it.

  **Measured null.** 50.9 % [50.4, 51.4] on seed 43 did *not* replicate:
  49.7 % [49.2, 50.3] on seed 97, pooling to 50.3 % [49.95, 50.68] over
  28 800 paired games. The fifth "obvious" improvement in this series to
  evaporate on replication.

  The field is not the excuse. The natural defence — "generated sealed
  builds don't run thin splashes, so the case never comes up" — was
  checked and is false: 3 of 12 decks on seed 43 and 4 of 12 on seed 97
  run a colour on ≤4 sources. The failure mode is present roughly a
  third of the time and still doesn't move the win rate.

  **Off by default** (`smarttap` opts in). It was briefly left on for
  the reasoning — the change cannot make a cost unpayable, the order it
  replaces was an accident of `battlefield` iteration rather than a
  decision, and the client's human-facing auto-tap is the case that
  motivated it — and then turned off to match how every other null in
  this tier was handled. The code and the profile stay so it can be
  re-measured, ideally on a field built to stress thin splashes.

  It also carried a **quadratic regression**: the selection called
  `effective_mana_abilities` per candidate per colour *inside the
  per-pip loop*, invisible in a two-player 40-card game and fatal in
  4-player Commander (`bot_vs_bot_commander_demo_terminates` went from
  seconds to past its 600 s timeout). Fixed by building the source
  table once per auto-tap — 600 s → 0.78 s. Worth remembering as the
  cost side of shipping a null on reasoning.

  The flag exists purely for measurement: the behaviour lives in the
  engine, so without a per-player switch both seats of a mirror would
  get it and the ladder would be structurally unable to show anything —
  the same blindness as the point below.
- 🟡 **Determinized combat search** (`EvalWeights::determinize`,
  `det1`/`det3`) — `simulate_attack_outcome` and `simulate_block_outcome`
  clone the true `GameState`, so the rollout opponent casts the cards
  they are actually holding and both seats draw the real top of library.
  The bot has been searching with perfect information. Redealing the
  hidden zones first costs **48.9 % [48.4, 49.4]** at one redeal and
  **49.4 % [48.9, 49.9]** averaged over three.

  Read that as the price of honesty, not a verdict on the idea. Both
  arms are mirror bots and the *control cheats*; taking information away
  from one side and not the other is expected to cost win rate. The
  mirror ladder cannot measure this fairly — it never could, which is
  why nothing before now had caught it. Against a human in the client
  the cheating is indefensible whatever the number says, so the open
  question is which default the client ships, not whether the search
  should be able to read a hand.
- 🟡 **Land-drop sequencing** (`landseq` / `EvalWeights::land_urgency`) —
  missing colors weighted by how cheap the cards demanding them are, and
  a per-land check for whether *that* land turns on a cast this turn (so
  a tapland is nearly free with no play and expensive otherwise).
  **Measured and not adopted**: 50.3 % [49.6, 51.0] over 19 200 sealed
  games. Two methodology notes worth more than the result: measured on
  `--decks both` first it read 49.4 % and *could not have read anything
  else* (those archetypes play basics, so tapland timing never fires),
  and the sealed +1.4 at 4 800 games collapsed to +0.3 at 19 200 — the
  third such evaporation after `blk` and `atk-race`.
- 🟡 **Better sequencing** (hold-up, when to cast) — reactive
  deployment landed: the stack-response value bar drops 10 → 5 with 6+
  cards in hand so answers get spent instead of rotting in a clogged
  hand; instant-speed removal fires at a declared attacker during
  DeclareBlockers when the attacker is worth it
  (`pick_defensive_removal`, ward- and outcome-gated); and
  sacrifice-for-value abilities are cracked when the settled outcome
  beats staying pat (`pick_sacrifice_value`). Self-cost optional
  triggers are likewise judged by settled outcome
  (`decide_optional_by_outcome`). Remaining: land-drop choice, deliberate
  hold-up planning.
- 🟡 **Mulligan decisions** — `RandomBot` ships flood/screw mulligans with
  color-screw awareness. A quality-aware rule (`mull` — card-quality sum,
  a redundancy requirement at two lands, on-the-draw allowance) is
  **measured and not adopted**: 50.2 % [49.6, 50.8] over 28 800 sealed
  games. Its tests stay as documentation of two hands the shipped rule
  reads backwards (a two-lander living off one two-drop is kept; six
  lands and a bomb is shipped). Remaining: transitive fetch/dual
  sources.
- 🟡 **Targeting / mode / X-value choices** — mid-resolution modals are picked
  by settled-outcome eval (`decide_mode_by_outcome`), scry/surveil/rearrange
  order for real (`decide_scry` — flood to the bottom, bricks off the top,
  wants first), and targeting/affordability is ward-aware (CR 702.21:
  a tax the bot can't pay after the spell's own cost drops the candidate,
  a payable one is priced into the score; `bot_wont_cast_removal_into_*`
  tests). SOS college mirrors run on the ladder (`bot_ladder --decks sos`)
  and probe (`bot_probe --deck sos`). X sizing now splits spare mana across
  multi-X pips ({X}{X} paid 2X but was sized as one) and covers
  prepare-casts (`max_affordable_x_for_def`). **Simulations answer
  decisions with the bot's own policy table** (`decide_pending_policy` in
  every lookahead/combat sim — they used to assume an AutoDecider future:
  bad scries, declined tutors, mode 0). Remaining: X chosen by outcome
  eval rather than max-dump.
- 🟡 **SOS mechanic play** — Prepare: inset-spell candidates, response casts
  when removal targets the prepared body (`pick_prepare_response`, plus the
  own-main response-timing dispatch fix that also revived counterspells
  there), a re-prepare mana sink, and a Prepared-counter term in
  `permanent_value`. Paradigm: the free-copy prompt is a real suspension
  now, and the bot declines life-draining copies at a low total
  (`self_life_loss`). On-cast payoff steering: Opus (prefer 5+-mana casts)
  and Infusion (lifegain first) score nudges; Repartee offers a
  creature-aimed sibling candidate the outcome eval judges; Increment
  nudges casts that clear the smallest body's threshold
  (`increment_threshold`); Converge casts pre-float one source per missing
  college color so the payment drains distinct colors
  (`pick_converge_prefloat` — bot-side, the engine payment funnel is
  untouched). Prepare-cast X is sized like a hand cast. The Prismari /
  Quandrix ≈ 49 % split was probed per college (`bot_probe --deck
  sos:<college> --vs baseline`): the losing pattern is over-attacking on
  small boards (82 % of eligible, 78 % all-in in Prismari; 41-42 % of
  creatures tapped at DeclareBlockers vs 27 % in healthy Witherbloom) plus
  reactive spells rotting in hand (42 cleanup discards / 60 games; ONE
  instant-timing cast). Two hypotheses measured and killed on 1000-game
  SOS ladders each: `atk-hold` (hold_instants — 49.4 %, Prismari *worse*
  at 46.0) and `blk` (block search — 50.1 %, tapped blockers are the
  cause, not assignment). Open lead: attack restraint that respects the
  defender's open mana / lets the attack sim cast spells for both sides
  (the sim casts nothing today, `simulate_attack_outcome` doc). A real
  `ChooseColor` policy (hand-pip demand) also landed off the Quandrix
  probe (11 % of its decisions were first-legal-White).
- 🟡 **Learned evaluation (SOS sealed)** — the ML stack's Phase A shipped:
  `crabomination_nn` (dependency-free inference + shard format, opt-3 in
  debug via a per-package override; wasm-safe, no framework in the engine
  or client path), `crabomination_ml` (candle trainer: deep-sets value
  net over card embeddings + zone-pooled objects, auxiliary life-diff /
  game-length heads per the KataGo credit-assignment result), and
  `server/encode.rs` (observable-info-only encoder, SOS sealed vocab).
  A parity test pins the candle model and the engine forward pass to the
  same numbers. Phase B shipped too: the concurrent `selfplay_train`
  loop (actor threads + throttled learner + atomic checkpoints, ~10.6
  sealed games/s on 22 debug-build actors), the `net_eval` slot registry,
  the `net`/`net-blend` bot profiles, and `bot_ladder --decks sealed`
  (same-deck sealed mirrors — build quality cancels, rows measure
  piloting). Release builds approved for the ML tooling (~82 games/s
  generation, 7.7× debug; gates cost ~15 s). Gates so far, 1 200
  sealed-mirror games each vs `atk-sim`: full net replacement 43.6 / 42.3
  / 43.4 % across round 1 (25k games), round 2 (100k), and round 2's
  over-reused tail — **flat across a 4× data jump**, so data volume is
  not the constraint; heuristic+net blend 49.3 / 49.2 / 50.7 % — stable
  parity. The tail experiment also priced window over-reuse (loss 0.30 →
  0.14, zero strength change → the trainer now caps the tail). Round 3
  measured (mid-turn snapshot cadence, 10.5 M rows, per-head loss
  logging, capped tail): replacement 44.7 % [41.9, 47.5] — best yet but
  within noise; blend 49.3 % — parity unchanged; **blend at 3× loudness
  45.9 %** — amplifying the net hurts, i.e. where it disagrees with the
  heuristic it is more often wrong. Standing diagnosis: `eval_material`
  scores outcomes of resolved sims (a one-ply search with a perfect
  model), so the net must carry long-horizon signal to add value, and
  ~125 k params of pooled encoder doesn't yet. Round 4 (5× capacity
  ~600 k params, keyword object features, CUDA-ready `cuda` feature
  flag): 43.8 / 48.8 / 47.1 % — same bands, **but the CPU learner only
  managed 0.4 visits/row before the tail cap**, so capacity remains
  untested until the learner moves to the GPU (`pacman -S cuda`, then
  `--features cuda`). Next levers: GPU-scale training, search-improved
  targets. **Phase C's build net passed its gate — the first learned
  component to clear the house bar**: `DeckNet` (D(decklist)→win prob,
  ~30 k params, trained free off the self-play stream's decklist
  labels) judging best-of-32 builds beat the heuristic static judge
  over the same candidate sets 61.7 % [58.9, 64.4] and 60.7 %
  [57.9, 63.4] on independent seeds (1 200 games each,
  `selfplay_train --gate-builder`). Remaining Phase C wiring: use the
  net-judged builder for training-run decks and as `recommend_pool`'s
  instant surrogate. Play-net replacement/blend still not adopted.
- ⏳ **Difficulty levels**; optional **search-based AI** (MCTS over snapshots).
