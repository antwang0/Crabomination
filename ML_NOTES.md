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
- 🟡 **Round 16 — sim-judge distillation closes the exploit, two
  iterations in.** The fix round 15 demanded: retrain the deck net on
  *gauntlet win rates* (240 games vs a fixed 20-deck field per label)
  over a deck mix that covers search-visited space — builder picks,
  best-of-32 picks, 3/8/15-swap mutants, and climb trajectories under
  the current judge (`--distill-gen` / `--distill-train`,
  `deck_labels.bin`, `DECK_SHARD` format). DAgger-shaped: each iteration
  labels the previous judge's own climb endpoints.

  | climb-vs-pick gate (the exploit test) | result |
  |---|---|
  | original net (round 15) | 11.6 % / 14.3 % |
  | distilled, iteration 1 (400 labels, holdout pair-order 93 %) | 30.0 % |
  | distilled, iteration 2 (+200 labels under iter-1 judge) | 34.5 % [33.2, 35.9] |

  The within-distribution gate holds at 58.1 % / 57.0 % vs static — no
  regression — and the label spread (0.00–0.76 win rate) is what gave
  the judge its missing "bad deck" gradient. Not converged: the climb is
  still net-positive-scored and real-negative, but each ~10-minute
  iteration buys ground and the loop's stopping rule is explicit — stop
  when climbed-vs-pick stops losing. **Iterations 3–7 (2026-08-08):
  39.0 → 42.3 → 39.3 → 39.1 → 43.0 % — the curve plateaus around 40 %**
  at the 200-labels-per-iteration budget: the catastrophic exploit is
  patched (12 % → ~40 %) but the climb still finds mild off-distribution
  optimism, and marginal iterations stopped buying ground. The final
  judge's vs-static gate is the best yet: **61.0 % [59.6, 62.4]**. Open
  ends, in order: bigger label batches and rotated gauntlet fields per
  iteration (the plateau may be a label-budget ceiling, not a method
  ceiling); a trust region on the climb (≤3 swaps) which at ~40 %
  exploit-severity is likely already net-positive; and the deck_duel
  rematch of the final judge's pick against the simulation pick on
  decks/sealed_pool.txt. Also this round: the three builder
  gates run pools in parallel (~12× faster; statistically equivalent,
  not bit-identical — game jitter is thread-RNG by design).
- 🔴 **Round 15 — hill-climbing the deck net is adversarial, not
  optimizing.** `hill_climb_build_by` (greedy single-spell swaps from the
  net's best-of-32 pick, same judge) gated against the pick it started
  from: the climbed builds won **11.6 % [10.7, 12.5] and 14.3 %
  [13.4, 15.4]** of 4 800 games per seed — catastrophically worse — while
  the net's own score of them rose from ~0.70 to ~0.95 across ~20 of 23
  spell slots changed. Textbook off-distribution exploitation: the net
  was trained to rank *noisy-greedy builder outputs*, and unconstrained
  search maximizes its errors, not its knowledge. Best-of-32 works
  precisely because all candidates come from the distribution the judge
  is calibrated on. This also sharpens the deck-duel reading: best-of-512
  is a mild dose of the same tail-walking. Rules of engagement from here:
  the deck net is a **within-distribution surrogate**, never an
  objective; any search under it needs either a tight trust region
  (≤2–3 swaps from a builder-generated start) or a judge trained on
  search-visited builds (sim-judge distillation with climb states in the
  training set). Both are open; the unconstrained form is closed.
- 🟡 **Round 14 — the self-play loop runs end to end; the gains don't
  compound.** The label self-consistency candidate got its test: train a
  fresh net on games piloted *by the net* (`--use-best`), gate the
  result, promote, repeat. Replacement-pilot gates, 1 200 paired
  sealed-mirror games per cell:

  | vs `atk-sim` | seed 43 | seed 97 |
  |---|---|---|
  | control (heuristic-labelled, r12) | 47.6 % | 48.5 % |
  | gen 1 (labels from r12-net play) | **51.4 % [49.5, 53.3]** | **50.4 % [48.6, 52.2]** |
  | gen 2 (labels from gen-1 play) | 48.7 % [46.7, 50.6] | 49.8 % [48.0, 51.6] |

  Generation 1 produced the first above-50 point estimates in fourteen
  rounds — +3.8/+1.9 over control, exactly the direction self-consistency
  predicts. Generation 2 gave it back. So the loop does not compound at
  this scale, and the hypothesis in its strong form (each generation's
  labels make the next generation stronger) is not supported. What
  remains defensible: one generation of training on net-piloted games is
  worth roughly the historical deficit — the gen-1 net is the first
  replacement pilot at genuine parity with the heuristic search — but at
  ±1.9 pts per cell against a ~2-pt effect, distinguishing "small real
  one-off gain" from "the good half of noise" needs bigger gates than
  the effect justifies. Two structural notes for whoever reopens this:
  the loop promotes unconditionally (a gatekeeper that only promotes a
  candidate beating the incumbent is the standard refinement), and
  net-piloted generation runs at 70 games/s — the full
  generate/train/gate cycle is ~10 minutes, so a many-generation run is
  affordable if a reason to believe in it appears.
- 🔴 **Round 13 — both survivors fall, and the paradox is now fully
  unexplained.** Two measurements, two seeds each.

  **The sim-leaf mismatch hypothesis is REFUTED.** `--calibrate-leaves`
  (via `server::leaf_capture`, hooks at all four search leaf sites)
  scores net and heuristic on the *simulated* positions the searches
  actually rank, labelled by the real game's winner — the direct test of
  explanation 3, the last one standing. The edge does not shrink
  off-distribution; it holds or grows:

  | net − heur AUC | snapshots | sim leaves |
  |---|---|---|
  | seed 43 | +0.0202 | +0.0211 |
  | seed 97 | +0.0452 | +0.0519 |

  (~5 600 leaves and ~30 000 snapshots per seed from the same 300 games;
  the net's absolute AUC on leaves is 0.84–0.85, *better* than on
  snapshots.) Every mechanistic explanation proposed for "strictly better
  predictor, strictly worse pilot" has now been tested and refuted:
  local discrimination (net is better), manufactured noise (quantisation
  moved nothing), memorised checkpoints (~4 pts, not the gap),
  saturation (collapsed 4×, still loses), late-game-carried AUC
  (inverted), and distribution mismatch (this entry). What remains is
  not a measurement artifact but something structural — the leading
  candidate being label self-consistency: both evaluators are judged at
  predicting outcomes of games *played by heuristic-maximising pilots*,
  a fixed point the heuristic is definitionally consistent with and the
  net can only approximate from outside. If that is the mechanism, no
  calibration can see it, and the test is training on net-piloted games
  (`--use-best` self-play) so the net's labels come from its own play.

  **The ply-scheduled blend is a null.** The one design
  three rounds of stratified calibration pointed at: the net's predictive
  edge peaks at ply 8–11 and vanishes by ply 32+, so `netb-ply`
  (`net_eval_blend_ply`) scales the blend-300 bias by turn — full through
  turn 5, linear to zero at turn 12 — spending the net's voice only where
  it measurably knows more. Gated 1 200 paired sealed-mirror games per
  cell, two seeds: **47.9 % [46.3, 49.6] / 49.1 % [47.6, 50.6] vs `gang`**
  and **49.0 % [47.5, 50.5] / 49.4 % [48.0, 50.8] vs constant
  `net-blend300`** — the taper is at best neutral against the constant
  blend it modifies, and the pair loses to the default outright on one
  seed. A predictive edge concentrated in the contested phase still does
  not convert to wins through a value-bias blend; whatever the bias tilts
  in the opening is not the thing the search needed help with. The
  profile stays as a measured control (`netb-ply`), not a live candidate.
- 🟡 **Round 12 — the fast loop is real; the quality levers were not.**
  Two halves, one replicated, one refuted by its own replication.

  **The training loop got 3–6× faster, and the speedup replicated.**
  Three changes (`--stop-after-stale N`: stop generation once the holdout
  AUC goes N checkpoints without a new best; `--relabel-mode new`:
  λ-relabel only rows pushed since the last pass, instead of the whole
  250 k window; a learner wall-clock decomposition in `stats.jsonl`).
  Measured against round 11's `ab_full` (90 k games, 1 539 s, learner at
  1.56× of its 6× reuse cap because 66 M relabel forward-rows dwarfed
  13.5 M trained): complete measured runs now take 417–712 s, relabel
  drops to 6–7 % of learner time — and generation jumped 58.5 → 81–85
  games/s, because the full-window relabel's CPU-side batch packing had
  been stealing actor cores all along. Quality is no worse: the control's
  calibration AUC is 0.7835 (seed 43) / 0.7973 (seed 97) against round
  11's 0.7798. An experiment cycle is now ~7 minutes, seeds included.

  **Representation, aux heads, and capacity: all null at this run
  length.** One change per run, seed 43 then seed 97, scored by
  `--calibrate 500` on `best.safetensors`:

  | calib AUC | control (`--ablate rel`) | +relations/stack | +aux head | +2× width |
  |---|---|---|---|---|
  | seed 43 | 0.7835 | 0.7830 | 0.7784 | 0.7834 |
  | seed 97 | **0.7973** | 0.7717 | — | 0.7686 |

  Every seed-43 signal — the relation block's early-ply gains, the wide
  net's saturation collapse to 1.1 % — reversed or vanished on seed 97,
  where the *control* was the best run of the round. The load-bearing
  observation: identical config across seeds moves calibration AUC by
  ~0.014, which is as large as every treatment effect measured. **At
  this run length, single-seed calibration deltas below ~0.015 AUC are
  unreadable — treatments need multi-seed means, which the fast loop now
  makes affordable (~7 min/seed).** The v5 format itself stays: it is an
  information superset with an `--ablate rel` control, the relation
  features cost nothing measurable at encode time, and a Pacified
  creature no longer encodes identically to a free one — but no quality
  claim attaches to it yet.

  Infrastructure that landed: encoder v5 (relation flags, special
  counters, stack zone groups; `OBJ_FEATS` 37, `NUM_GROUPS` 8,
  `SHARD_VERSION` 5), the `--aux` short-horizon head (next-snapshot
  life/power/creature deltas + opp hand, labelled from the trajectory),
  width flags (`--emb-dim/--obj-hidden/--h1/--h2` — the engine reads
  shapes, so capacity is a flag), and `[profile.release-fast]` for
  iteration builds. Old checkpoints are shape-incompatible with v5
  binaries and fail loudly at load.
- 🟡 **Round 11 — what the net was never told.** Ten rounds varied the
  model, the targets and the data volume; none of them varied *what is in
  the input*. Four changes, one shard-format bump (`SHARD_VERSION` 4):

  1. **The seat's own library is encoded** (`G_LIB_SELF`, a sixth zone
     group) as a multiset deduplicated by card name, each entry carrying
     its remaining count. Before this the library was a single scalar,
     `library.len() / 40`: "22 cards left, three of them removal and one a
     bomb" and "22 lands" encoded identically — though a seat's own
     decklist is information it plainly has. Entries are emitted in
     vocabulary-index order so the shuffle cannot leak, whatever pooling
     or attention does downstream (`library_order_does_not_reach_the_encoding`).
     It is also the one zone where bag-of-cards is unambiguously the right
     prior, which is the same argument that makes the deck net work.
  2. **Castability** (`OBJ_FEATS` 20 → 28, `GLOBAL_FEATS` 24 → 36).
     Coloured pips on every object; castable-now and
     castable-after-one-more-land flags on hand cards; untapped mana by
     colour as globals for *both* seats. `cmc / 8` said a card cost four
     and could not say the four was `{2}{G}{G}` against three Forests —
     nor that the opponent is holding two untapped blue, which is the
     shape of every instant-speed decision. Affordability is Hall's
     condition over the 32 colour subsets against
     `GameState::untapped_mana_colors`: exact for the one-mana-per-source
     model it assumes, and explicitly a feature rather than a legality
     check (multi-mana sources, cost reduction and `{X}` fall outside it).
  3. **Embedding transfer** (`selfplay_train --seed-emb DECK.safetensors`).
     The play net's card embeddings start from a trained *deck* net's
     instead of from noise. Nothing is frozen — the claim is only that it
     is a better starting point, and the control that tests it is the same
     run without the flag. The deck net is the only learned component that
     has ever cleared the house bar, and it learns what a card is worth
     from a signal the play net never receives: one label per decklist,
     tens of thousands of decklists, no board state in the way.
  4. **Ply-stratified calibration** — `--calibrate` now breaks its
     net-vs-heuristic comparison out by position-in-game. See explanation
     4 below; this is the instrument for it.

  Every trained checkpoint was already stale before these landed: the
  upstream SOS additions moved the vocabulary 153 → 164, which invalidates
  embedding tables by design.

  **Results (2 × 90 k games, λ 0.7, attention, seed 43, paired):**

  - **The runs overfit hard, and the checkpoint being scored was the worst
    one.** Held-out AUC peaks at step 4–6 k (0.740 control, 0.759 seeded)
    and falls to 0.675 by step 53 k while the training win-loss goes to
    0.0011. `latest.safetensors` is whatever the run ended on, so *every
    gate and calibration in this program so far was scored on a memorised
    net.* Fixed: the learner now also publishes `best.safetensors`, keyed
    on held-out AUC.
  - **Embedding transfer helps where it can be seen.** Seeded leads the
    control at 10 of the first 12 checkpoints (up to +0.046 AUC at step
    16 k), and peaks +0.019 higher. On the *final* checkpoints the
    ordering reverses (0.7009 vs 0.7071) — but that comparison is between
    two overfit nets and measures which memorised differently, not which
    learned more. One seed pair; not adopted, not refuted.
  - Cost was not the problem: 58–69 games/s against the prior 44–45,
    despite roughly twice the objects per state.

  **Ablation (`--ablate lib,cast`, `.ladder/run_ablate.sh`) — four matched
  90 k-game runs, each scored on its own `best.safetensors` over 500 fresh
  games. `neither` is the old encoder on the new vocabulary, which is the
  control the vocab change destroyed.**

  | arm | net AUC | log-loss | Brier | heur AUC | outside [.05,.95] |
  |---|---|---|---|---|---|
  | **full** (lib + cast) | 0.7798 | **0.5575** | **0.1902** | 0.7377 | **3.3 %** |
  | `nolib` (cast only) | **0.7809** | 0.5720 | 0.1947 | 0.7329 | 11.6 % |
  | `nocast` (lib only) | 0.7748 | 0.5680 | 0.1942 | 0.7341 | 4.8 % |
  | `neither` (old encoder) | 0.7599 | 0.6033 | 0.2058 | 0.7404 | 12.8 % |

  1. **Both blocks earn their place: +0.020 AUC, −0.046 log-loss, −0.0156
     Brier over the old encoder.** Keep `full`.
  2. **The library group buys calibration, not ranking.** Its AUC effect is
     nil (`nolib` is +0.001, inside noise), but it collapses saturation
     **12.8 % → 3.3 %** and takes log-loss with it. That matters more than
     the AUC here: a flat, saturated output band is the *mechanism* by
     which a better predictor was made a worse player, so this is the
     first change to attack that mechanism directly.
  3. **Castability buys ranking**, consistently in both conditions:
     `full − nocast` = +0.005 AUC, `nolib − neither` = +0.021.
  4. **The earlier "the new encoder regressed to 0.707" reading was an
     artefact of scoring `latest`.** Same config, same seed, same games:
     `latest` 0.7071 / 0.792, `best` 0.7798 / 0.558. **Checkpoint
     selection was worth +0.073 AUC — more than every feature in this
     round combined.**

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

  **Re-gated on a clean checkpoint (round 11) — the dead end survives it.**
  Every number in the table above piloted `latest.safetensors`, which
  round 11 showed is a memorised checkpoint; that made the closure itself
  suspect. Rerun with `nets_ab_full/best.safetensors` (peak-holdout-AUC
  selection, 0.7798 AUC, 3.3 % saturated), 1 200 paired sealed-mirror
  games per cell, two seeds:

  | profile | vs `gang` s43 | vs `gang` s97 | vs `atk-sim` s43 | vs `atk-sim` s97 |
  |---|---|---|---|---|
  | `net` (replacement) | 48.1 % [46.0, 50.2] | 48.8 % [46.7, 51.0] | 47.6 % [45.5, 49.7] | 48.5 % [46.5, 50.5] |
  | `net-blend` | 47.8 % [45.7, 49.8] | 49.3 % [47.4, 51.3] | 50.7 % [48.8, 52.5] | 50.2 % [48.5, 52.0] |

  Two things at once: the contamination was *material* — the clean
  checkpoint recovers ~4 points of the replacement's deficit (44.8 →
  48.1/48.8 vs `gang`) — and it was *not the cause*: no cell clears 50 %.
  The saturation account also falls with it: saturation dropped 12.8 % →
  3.3 % between the two checkpoints and the replacement still loses, so a
  flat output landscape was not what was losing the gates either.
  Explanation 3 below (sim-leaf distribution mismatch) is now the only
  proposed cause left standing, and the ply table further down still
  points at a phase-dependent evaluator as the one untried design.

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
  3. *"Distribution mismatch: the search evaluates simulated leaves the
     net is neither trained on nor measured at."* **REFUTED, round 13**
     — `--calibrate-leaves` captured the leaves the searches rank and
     the net's edge held or grew there on both seeds (+0.021 vs +0.020
     and +0.052 vs +0.045 net-minus-heuristic AUC, leaves vs snapshots).
     The net predicts *better* on the search's own distribution.
  4. **REFUTED, and inverted — the most useful thing round 11 found.**
     The proposal was: AUC pools every snapshot, late positions are
     already decided and numerous, so a net that is better late and worse
     early would post a better aggregate and play worse — winning the
     pooled comparison on exactly the positions where being right is free.
     `--calibrate` now prints the breakdown by ply. The data says the
     opposite. Net minus heuristic AUC, round-11 `full` on
     `best.safetensors`:

     | ply | n | net AUC | heur | delta |
     |---|---|---|---|---|
     | 0–3 | 4000 | 0.6404 | 0.5832 | +0.0572 |
     | 4–7 | 4000 | 0.6640 | 0.5841 | +0.0798 |
     | 8–11 | 4000 | 0.6749 | 0.5785 | **+0.0964** |
     | 12–19 | 8000 | 0.6947 | 0.6055 | +0.0891 |
     | 20–31 | 11956 | 0.7887 | 0.7362 | +0.0525 |
     | 32+ | 19688 | 0.8565 | 0.8404 | +0.0161 |

     The margin **peaks in the contested early-mid game and decays to
     nearly nothing by ply 32+**: both evaluations converge once the board
     is developed enough that counting settles it. Since 61 % of snapshots
     are ply 20+, the pooled figure is dominated by the phase where the
     net adds least, and therefore **understates** it where the search's
     choices still matter.

     *Careful with the checkpoint here.* On `latest` (i.e. memorised) the
     same config reads +0.038/+0.051/+0.039/+0.019 then **−0.044/−0.041**
     — the net apparently *losing* late. That version of this finding was
     an artefact, and the old-encoder `neither` arm still shows it (−0.011
     at ply 32+), so it is what overfitting and a thin input look like, not
     a property of value nets. Score `best.safetensors`.

     Read against explanation 3, this still argues for a
     **phase-dependent** evaluator — the net's marginal value over the
     heuristic is concentrated before ply 20 — rather than the
     replacement/blend pair that has been gated eight times. It is the
     first concrete design the diagnostics have pointed at rather than
     away from.

     Base rates are exactly 0.5 in every stratum — both seats are
     snapshotted at the same instants and carry the same `ply` — so the
     buckets are directly comparable to each other.

  Levers already exhausted: data volume, window reuse, capacity (round 4),
  snapshot coverage, target shape (MC → TD(λ)), architecture (pooling →
  attention), output shaping (quantisation), and checkpoint selection
  (`best` over `latest` — worth ~4 points, not the gap). Making the net a
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
  `selfplay_train --gate-builder`). **Re-gated after the 153 → 164 vocab
  change on a round-11 net at 8× the sample: 60.0 % [59.1, 61.0] (seed 43)
  and 61.8 % [60.8, 62.7] (seed 97), 9 600 games each, winning 11 of 12
  pools on both.** Four independent gates now, all in the 60–62 % band —
  this is the one stable result in the program. Remaining Phase C wiring:
  use the net-judged builder for training-run decks and as
  `recommend_pool`'s instant surrogate. Play-net replacement/blend still
  not adopted.

  What the gate does *not* say: it compares the net judge against the
  **static score**, not against `recommend_pool`'s simulated ranking,
  which plays games and is the stronger judge of the two. "Net beats
  static" and "net beats simulation" are different claims — and the
  second is now measured, in the direction expected. On the
  `decks/sealed_pool.txt` W/B builds, `deck_duel` played the simulation
  judge's top pick against the deck net's top pick (they differ by three
  spells and a land) with identical pilots: the simulated pick won
  **56.1 % [54.6, 57.6]** and **55.8 % [54.3, 57.2]** on independent
  seeds (2 000 antithetic pairs = 4 000 games each, seeds 11/12). One
  pool, top-pick-vs-top-pick — not a refutation of the gate, but a clean
  bound: the net is a fast surrogate for the simulation judge, not a
  replacement, and when the two disagree about a build the simulation is
  the one to trust.
- ⏳ **Difficulty levels**; optional **search-based AI** (MCTS over snapshots).
