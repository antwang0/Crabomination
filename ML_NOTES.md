# ML notes — bot/net experiment history

Long experiment narratives: what was gated, what was adopted, and the dead
ends. Moved here verbatim from `FEATURE_ROADMAP.md` "Tier 13 — AI" so nobody
re-derives a refuted hypothesis. Never summarize an entry away — a dead end
only stays dead while the reasoning that killed it is readable.

## Tier 13 — AI

- 🟢 **Deck-net re-gate (2026-08-24) — the fifth and sixth gates land on
  the historical band, and the committed-artifact defect closes without a
  retrain.** All seven committed `deck-latest` checkpoints were vocab 153
  against a live 164 (TODO "ML — defects found 2026-08-24"), which killed
  `--use-deck-best` and every deck-net consumer. But the deck stream rides
  along in every `selfplay_train` run, so the r41/r45 directories already
  held current-vocab deck nets. Two were gated, pre-registered in
  `.ladder/run_deck_regate.sh` (800 games × 12 pools per cell, ladder
  seeds 43/97, static judge as control — the round-11 re-gate shape):

  | artifact | seed 43 | seed 97 | pooled |
  |---|---|---|---|
  | nets_r45_ctrl_s43/deck-latest | 59.6 [58.6, 60.5] | 61.1 [60.2, 62.1] | 60.3 |
  | nets_r41_v7_s43/deck-latest | 59.7 [58.7, 60.7] | 61.5 [60.6, 62.5] | 60.6 |

  Six independent gates now (61.7 / 60.7 / 60.0 / 61.8 / 60.3 / 60.6):
  the deck net remains the program's one stable result, and the band
  survives training eras it was never re-measured under. The two
  artifacts are a statistical tie (0.3 pooled apart against ±1 cells), so
  the pick is by provenance, not by number: the r45 net — the newest
  champion-class run — is committed as **`nets/deck-champion.safetensors`**,
  reviving `--use-deck-best`, `--gate-builder-hc` and `--distill-gen`.
  Worth noting: the ladder-seed split (both nets ≈59.6 on seed 43, ≈61.3
  on seed 97) tracks the seed's pool field, not the net — pool-level
  clustering, the builder-v3 lesson restated. The structural hazard was
  closed separately by the fifty-fourth pass's `VOCAB_SNAPSHOT` freeze
  (see "The deck-net vocabulary freeze" below): names own frozen indices
  and the table only grows at the end, so a card addition no longer
  retires nets. This artifact is at the frozen size (164) and loads under
  `vocab_fit`.

- ⚪ **Round 51 (2026-08-23) — the fetch as searched arms is +0.35 and
  UNRESOLVED; the demand-aware ranking under it is a precise ZERO. The
  round's lesson is that a search-level flag cannot inherit the paired
  precision a heuristic-level flag gets.** Pre-registered in
  `.ladder/run_r51_fetch.sh`.

  | part | cells | pooled | widths | pairs that diverged |
  |---|---|---|---|---|
  | A `fetch_arms` | 50.3 [49.7, 50.9] / 50.4 [49.8, 51.0] | 50.35 | ±0.59 / ±0.60 | 4682 / 6000 |
  | B demand ranking | 49.9 [49.8, 50.0] / 50.0 [49.8, 50.1] | 49.95 | ±0.11 / ±0.15 | 46 / 6000 |

  **The hole was real and bigger than pitched.** `Decision::SearchLibrary`
  had never reached the MCTS menu at all: `MctsBot::next_action` falls
  through to the heuristic on *any* pending decision, so every tutor and
  every fetchland in this program's history was answered by
  `decide_library_search`'s fixed ranking — and inside rollouts by
  `AutoDecider`, which takes the **first eligible hit**. The rollouts were
  scoring fetches worse than the bot that plays them.

  **Part A does not adopt.** Both cells sit above 50 and both intervals
  *include* it. Under the round-50 rule (adopt a sub-0.5 effect only when
  both cells clear 50) that is not a result. Direction is consistent
  across two seeds and the flag is left in, off by default, as
  `mcts-net-fetcharms`.

  **Why Part A is imprecise and Part B is not — this is the transferable
  part.** Round 50 established that near-mirror antithetic pairs
  contribute no variance, so a rare flag gets measured unusually tightly.
  Part B inherits exactly that: 46 of 6000 pairs diverge, rho −0.985,
  ±0.11. Part A cannot. Adding arms perturbs the search's own random
  stream, so **4682 of 6000 pairs diverge** and the pairing collapses back
  to the program's usual ±0.6 — the *same* underlying decision density
  (~0.25 fetches/game), measured 5× less precisely, purely because the
  intervention is inside the search. A flag that changes what the search
  *does* buys precision only by running more games; a flag that changes a
  fixed answer gets it free. Budget accordingly: sub-0.5 search-level
  effects need ~4× the cells that a heuristic-level one does.

  **Part B is a genuine zero, not an underpowered one.** At ±0.11 and
  ±0.15 this is one of the tightest cells the program has ever run, and it
  says the demand-aware read is worth nothing: ranking basics by unmet
  pips in hand rather than by supply alone, and tutor hits by castability
  before mana value, changes 46 games in 6000 and wins none of them net.
  The supply-only heuristic was already picking the same card almost
  always — the two reads only differ when the hand's colour demand points
  away from the scarcest source, which sealed pools rarely produce.

  **What is kept, and the honest justification.** The demand-aware ranking
  stays on by default with `legacyfetch` as the control, and it is *not*
  justified by measurement — it bought nothing. It is kept as a
  consistency change: `ChooseColor` has counted pips in hand since it was
  written and this is its sibling decision reading the same signal, plus
  `rank_library_search` is the seam `fetch_arms` needs. Recorded here so
  nobody later cites round 51 as evidence that it helped.

  **Do not re-open the fetch on a valuation theory.** Two mechanisms tried
  at once, and the *menu* half is the only one with a pulse. That matches
  item 1b (menu holes pay, valuation refinements do not) — but this is the
  first menu hole to come back unresolved rather than positive, and the
  likely reason is the one the pre-registration named: a fetch's payoff
  usually lands past the 3-turn horizon, so the sims price it on noise.
  The same horizon limit killed the simulated mulligan (r49, 47.45).

- 🟢 **Round 50 (2026-08-23) — the planeswalker cash-out fix is +0.25,
  replicated, and the round's real lesson is that a *rare* effect is not
  an unmeasurable one.** Pre-registered in `.ladder/run_r50_walker.sh`.

  | cells | pooled | interval widths |
  |---|---|---|
  | 50.3 [50.1, 50.5] / 50.2 [50.1, 50.3] | **50.25** | **±0.21 / ±0.12** |

  **The fix.** `pick_loyalty_ability`'s guard restricted the bot to
  loyalty-SPENDING abilities whenever total enemy creature power met
  *current* loyalty. That fired essentially always past the opening
  turns, so the bot dumped loyalty the turn a walker landed and let it
  die: **zero plus activations against eight minuses** across every
  recorded human game, including Ral Zarek spending its last point to
  strip one card and die with a `+1` available. The read now counts only
  creatures that could actually attack, subtracts our untapped blockers,
  and compares against the loyalty the walker would have *after* its
  best plus.

  **My pre-registered reading of this band was wrong, and the error is
  worth more than the result.** The script called 50.0–50.5
  "underpowered by the deck field, not evidence against", assuming a
  rare class could not be resolved. Both cells' intervals *exclude* 50,
  so it is resolved: small, real, replicated. The reason is a property
  of the paired design nobody here had exploited: **when most antithetic
  pairs are exact mirrors they contribute no variance, so the few games
  that differ are measured with unusual precision.** ±0.21 and ±0.12
  against this program's usual ±0.57. The zero-incidence games are not
  wasted — they are perfect controls.

  **This splits a class the program had lumped together.** The five
  earlier "zero incidence" flags (buff2for1, convlands, walkerchip,
  impulse, …) returned **exactly 50.0 at ±0.00** — the flag never
  changed a single game. This one returned 50.25 with tight bounds: it
  fires rarely and helps when it does. Those are different findings and
  the ladder distinguishes them cleanly. **Do not write off a
  rare-card-class flag as unmeasurable before running it** — if it fires
  at all, the paired ladder will resolve it; if it returns a hard 50.0
  ±0.00, that is a genuine statement that the field never reached the
  situation.

  The fix is unflagged and on by default (it is a correctness-class
  repair of a guard that was wrong three ways at once); `walkerlegacy`
  stays as the reproduction control, and the two regression tests —
  `defended_walker_banks_the_plus` and the pre-existing
  `doomed_walker_cashes_out` — pin both sides of the read.

- 🔴 **Round 49 (2026-08-23) — simulating the mulligan is 2.5 points
  WORSE than the predicate it replaces, and it is not the horizon, not
  the sample count and not the cost.** The second mechanism to fail on
  this decision, and the first to fail *downward*. Pre-registered in
  `.ladder/run_r49_mullsim.sh` at +0.5 to +1.5.

  | cells | pooled | cost |
  |---|---|---|
  | 47.2 / 47.7 | **47.45** | 0.2 s/game both sides |

  **Why it was tried.** `bot_probe --deck sos` over 300 games: Mulligan
  is 358 of 1424 decisions — **25.1 %, more than double the next kind**
  — and it sets up the whole game. It was also the only high-volume
  decision still answered by a predicate that never looks past the
  opening hand, while modes, optional triggers and sacrifice-for-value
  are all judged by playing the state forward. `mull_sim` plays keep and
  mulligan forward four turns over six determinised samples each and
  takes the better, so the cost of going down a card is measured rather
  than priced by a threshold.

  **What was ruled out, in order.**
  - *Cost*: 0.2 s/game for both profiles. Not a latency trade.
  - *"It mulligans too eagerly"* — the first hypothesis, and wrong.
    Mulligan decisions per game are **1.245 (gang) vs 1.28 (mullsim)**,
    i.e. the same rate. Same volume with a worse win rate means the sim
    is a worse *discriminator*, not a more aggressive one.
  - *Sample noise*: verdicts were compared at 6 and 24 samples over 12
    hands — **zero flips**, with a stable ~5.8-point branch gap. The
    6-sample estimate is not the problem.
  - *A rigged comparison*: both branches reach the same turn with a
    pending-free state, so the mulligan branch is not being scored at an
    earlier, less-developed point.

  **What is left, stated as the hypothesis it is.** A four-turn horizon
  scored with `eval_material` measures *board development*, and that is
  a poor proxy for whether an opening hand wins games. The shipped
  predicate — 2–5 lands plus a castable early play, with colour-screw
  awareness — encodes real knowledge about the whole game that a short
  material rollout does not recover. This rhymes with round 44: the
  evaluator is unreliable on state shapes it was not built for, and
  "turn-5 material" is not what a mulligan decision is about.

  **The pair of results is the finding.** `mull_quality` (a better
  predicate) was 50.2 % [49.6, 50.8] over 28 800 games; `mull_sim` (a
  different mechanism entirely) is 47.45 %. Two independent attacks on
  the highest-volume decision in the game, one null and one clearly
  negative. **The opening-hand decision is not where this bot is losing
  games, and the shipped predicate is better than it looks.** Both
  flags stay default-off with their tests as documentation.

  Method note: `mull_sim`'s first test passed *vacuously* —
  `mulligan_branch_value` answers a pending decision with
  `perform_action` and returns `None` when none is pending, and the
  hand-built state had no mulligan pending, so both branches returned
  nothing and the comparison was empty. The shipped test builds through
  `start_mulligan_phase` and asserts both branches simulated before
  comparing them. A green test is not evidence until you have checked it
  can fail.

- 🟡 **Round 48 (2026-08-22) — target arms re-confirm at +1.05, but
  fixing their ranking flaw was worth ~+0.1 and this design cannot
  resolve it; the champion's real level is 55.2 / 53.65, not the ~52.7
  / ~51.2 that has been quoted since round 26.** Pre-registered in
  `.ladder/run_r48_rearm.sh`.

  | part | cells | pooled |
  |---|---|---|
  | A `mcts-net-deep` vs `mcts-net-noarms` | 51.1 / 51.0 | **51.05** |
  | B `net` vs `atk-sim` (reference) | 55.9 / 54.5 | **55.20** |
  | B `net` vs `gang` (reference) | 54.1 / 53.2 | **53.65** |

  **A: the adoption replicates tightly, the fix inside it does not
  measure.** Round 46 adopted target arms at 50.95 with a flawed
  alternate ranking — alternates ordered "opposite side from whatever
  the auto-targeter chose", which spends an arm on a self-target
  whenever the baked-in pick is already correct. Filtering alternates to
  the side the slot wants reads **51.05** against the same control:
  **+0.1 over r46, against a ±0.57 cell.** Pre-registered at +1.0 to
  +2.0; landed at the floor of that band.

  The mechanism explains the null and is worth keeping: with `max = 2`
  alternates the flawed ranking still surfaced the *correct* target — as
  the second arm rather than the first. So the flaw never removed the
  right option from the menu, it only added a junk one beside it, and a
  junk arm costs one arm of rollout budget rather than a decision. The
  fix is still right (a wasted arm is a wasted arm, and at `max = 1` the
  flaw would have been fatal), but "wasting the first arm on a
  self-target" was a worse-sounding description than the ladder
  supports. The two cells agree to 0.1, which is the tightest
  replication in the program's piloting history and says the +1.05 is
  real even though the delta inside it is not.

  **B is a reference measurement, not a gate, and it restates the
  program's headline number.** Round 47 found the net profiles had been
  missing both adopted blocking layers; this is the first clean read of
  the champion since. **`net` vs `atk-sim` 55.20 (was ~52.7) and vs
  `gang` 53.65 (was ~51.2)** — about +2.5 on both, consistent with r47's
  +3.2 head-to-head once win-rate compression away from 50 is allowed
  for (the two are not the same quantity). Consequence for how this
  program describes itself: the learned evaluator has been ~2.5 points
  stronger than the "roughly at parity with the heuristic" framing
  carried since round 26, and that framing was partly an artifact of the
  handicap rather than a finding. Nothing needs re-deciding — every
  adoption compared like with like — but **quote 55.20 / 53.65, not the
  old band.**

  **Not gated, deliberately, and the cost is recorded:** the
  hostile-value ranking ("prefer the biggest threat") lives in the
  engine's auto-targeter, which every profile shares, so an in-process
  A/B would need a weights flag threaded through ~10 bot call sites or a
  thread-local that flips engine behaviour mid-game. Justified by the
  replay and by `hostile_auto_target_prefers_the_biggest_threat`.

- 🟢 **Round 47 (2026-08-22) — the net profiles were missing both
  adopted blocking layers, and it was worth +3.2 pilot-side / +2.5 in
  the search. Every net-vs-heuristic level from round 26 to 46 is
  understated.** `net_eval` branched off `attack_search_sim`, which
  predates both blocking adoptions, so the ladder's `net`, the champion
  `mcts-net-deep` and the client's `local_bot` all piloted with
  `block_gang=false`, `block_search=0`, `chump_blocks=false`. Value
  gang-blocks was adopted at 51.3 % over 28 800 games and desperation
  chump blocks at 51.0/50.8 over 24 000; neither ever reached the code
  the net plays with. Found by *printing the flags per profile* rather
  than reading the constructor chain — the chain reads plausibly and is
  wrong. Pre-registered in `.ladder/run_r47_netblocks.sh`.

  | part | cells | pooled |
  |---|---|---|
  | A `net` vs `net-preblocks` (pilot) | 53.7 / 52.7 | **53.20** |
  | B `mcts-net-deep` vs `mcts-net-preblocks` (search) | 53.0 / 51.9 | **52.45** |

  **Both far above the pre-registered +1 to +2**, and larger than the
  two layers measured separately on the heuristic (+1.3 and +0.9).
  Working hypothesis, not established here: the net evaluator judges
  post-block boards better than the hand-written evaluator the layers
  were fitted against, so they are worth more to it than to the profile
  that adopted them. The search gains less than the pilot (+2.45 vs
  +3.2), which is the pre-registered direction — rollouts already play
  some of these blocks out — though it was NOT visible on seed 43 alone
  (53.0, level with the pilot) and only appears pooled. A one-cell read
  of this round would have got its own headline finding backwards.

  **The consequence for the program's history is the larger result.**
  Every `net` vs `gang` / `net` vs `atk-sim` gate from round 26 through
  46 differed in *two* ways rather than one — evaluator AND blocking —
  with the net side handicapped by ~2.5–3.2 points. Rankings *within*
  each round are unaffected, because both arms of every net-vs-net cell
  shared the deficit, which is exactly why nothing ever looked wrong.
  What is wrong is the *levels*: the champion's ~52.7 / ~51.2 band, the
  replacement-vs-blend spread, v7's +0.4 and r45's capacity null were
  all measured on a handicapped net. Nothing needs re-deciding — every
  adoption compared like with like — but no net-vs-heuristic number
  from that era should be quoted as the net's standing strength.

  **Method note worth keeping.** This was invisible for twenty rounds
  because the constructor chain is readable and wrong: `net_eval` looks
  like it derives from the adopted profile and does not. Print the
  resolved flags of a profile before trusting what it inherits.

- 🟢 **Round 46 (2026-08-22) — target arms replicate at +0.95 and are
  adopted; the abilarms rehabilitation hypothesis is refuted on its own
  pre-registered terms.** Pre-registered in
  `.ladder/run_r46_targeting.sh`; 1000 games/archetype × 12 sealed × 2
  ladder seeds per cell (±0.57 paired), same r41 net as r42–r45.

  | part | cells | pooled | disposition |
  |---|---|---|---|
  | A `mcts-net-targetarms` vs `mcts-net-deep` | 50.7 / 51.2 | **50.95** | **ADOPTED** |
  | B `impulse` vs `gang` | 50.0 / 50.0 | 50.00 | zero incidence |
  | C `abilarms` vs `gang` (re-run) | 47.7 / 50.1 | 48.90 | negative, replicates |

  **A: the search could not reject a mis-aimed spell because the right
  aim was never on the menu.** Every cast candidate calls
  `auto_targets_for_effect_all_slots` once and bakes that assignment
  into the arm, so a mis-targeted spell was accept-or-reject on the
  whole package and the correct targeting was *absent* — unreachable at
  any valuation, net capacity or search depth. Offering up to two
  alternative slot-0 targetings (opposite-side first) is worth **+0.95
  over two seeds, both intervals clear of 50** — the same magnitude and
  the same replication standard as the chump-block adoption (51.0 /
  50.8, r43), and the second confirmation that this program's piloting
  wins are *menu* holes rather than valuation errors. Both are
  candidates the bot could not previously express; every valuation
  refinement tried since round 29 has been a null.

  **C refutes a hypothesis this session proposed, and the refutation is
  the point of having pre-registered it.** Round 43 read abilarms at
  48.0 / 50.5 and concluded "auto-aimed activations can harm". That
  cell ran while the filtered auto-target walk had no side preference
  at all, so it measured ability enumeration *plus* a targeting bug,
  and the r46 script said outright that a repeat would mean the class
  is genuinely harmful. It repeats: **48.9 against r43's 48.25, same
  seed-split shape** (one strongly negative cell, one at parity) on a
  fixed targeter. The r43 conclusion stands and now rests on better
  evidence; the "it was really the targeting bug" story is dead.

  **The asymmetry between A and C is the round's lesson.** Both add
  candidates to the same capped menu, and they land 2 points apart. A
  varies a decision the search had already judged worth making — a
  *sibling* of a vetted arm, at the cost of one arm. C adds whole new
  action types that displace vetted casts under a six-arm cap, on a
  class the heuristic has no scoring competence for. Adding a line the
  bot could not express pays when the line is a *variant of a good
  play*; it loses when it is an unvetted new play competing for the
  scarcest resource the search has (r42: iterations are the only lever
  that reliably pays).

  **B measured nothing, and says so.** Zero incidence on both seeds —
  the flag never changed a game, so sealed mirrors never reached a
  board where an impulse-draw activation was available and the hand-size
  gate fired. Fifth zero-incidence flag in three rounds (buff2for1,
  convlands, walkerchip, and now impulse): bot-vs-bot mirrors are
  structurally blind to card classes the mirror decks do not contain.
  Ark of Hunger's five idle turns are in a recorded human game, which
  remains the only instrument that sees this class. Stays default-off,
  justified by the replay rather than by the ladder.

- 🔴 **Round 45 (2026-08-20) — capacity, finally fed, is a dead null:
  2× widths under the r41 recipe move the pilot +0.01 ± 0.3.** The
  last round-4 lever run under conditions that could answer it.
  Round 4's "5× capacity" verdict was recorded as untested (CPU
  learner, 0.4 visits/row); this round ran ctrl (emb 32 / obj 64 /
  trunk 512×256, `--attn`) against cap (all doubled, ~4× params) at
  the r41 measurement floor — four training seeds, paired within
  seed, 250 k games each on the CUDA learner with feeding asserted
  per run (`learner device: cuda`, ~85 % train duty, ~21 M rows
  consumed per run; the double-width smoke held 74 % duty).
  Pre-registered in `.ladder/run_r45_capacity.sh`.

  **Pilot gates** (net vs atk-sim / gang, 1000 games/archetype × 2
  ladder seeds per net, pooled per pair):

  | opp | per-seed diffs (cap − ctrl) | mean paired diff |
  |---|---|---|
  | atk-sim | +0.20 / +0.15 / −0.20 / −0.10 | **+0.013 ± 0.307** |
  | gang | −0.10 / +0.30 / −0.30 / +0.15 | **+0.012 ± 0.423** |

  Every cell in the r38 band; AUC pairs net to zero (+0.0035 /
  +0.0028 / −0.0068 / +0.0008). **This is the null round 4 could not
  produce: with the learner demonstrably fed, parameter count is not
  the binding constraint at 2× under this recipe.** With r27/36/38/39
  (labels/targets closed at three scales) the account is now: neither
  the value target nor capacity binds — what has moved the program is
  representation (v7 +0.4, the r39 aux-head AUC record) and search
  budget (r27/42), which is where the queued leaf-distribution round
  aims.

  Two screen-level observations, recorded not claimed: `val_policy`
  favored cap in **all four pairs** (0.85/0.83/0.80/0.86 vs
  0.82/0.70/0.72/0.76) — the ranking metric's fifth dissociation from
  strength, now with a capacity flavor; and the one-seed search cells
  read ctrl 53.1 vs cap 51.8 (±0.64 each) — suggestive that the wide
  net is *worse* as a search leaf, unresolved at one cell and worth a
  paired cell only if capacity ever re-opens.

  Infrastructure that fell out: the batch eval server and the frozen
  pilot scorer now derive their full architecture from the checkpoint
  file (`PlayNet::arch()`) — the first cap run aborted on a shape
  mismatch because widths followed the run's flags, a latent bug every
  champion-width run had been walking past.

- 🔴 **Round 44 (2026-08-20) — the rollout is not replaceable by a
  shallower-but-wider search: horizon carries *bias* correction, and
  iterations only buy variance.** The value-equivalence question
  (MuZero's lesson inverted), made worth asking by the forty-first
  perf pass: with the net's forward vectorized, the 3-turn rollout is
  88 % of search wall, so a 1-turn rollout at matched cost affords 3×
  the iterations — and iterations are the only search lever that pays
  (r27/29/42). Pre-registered in `.ladder/run_r44_horizon.sh`; run at
  1000 games/archetype × 12 × 2 ladder seeds per gated cell (±0.65
  paired); same net as r42/r43 (`nets_r41_v7_s151`).

  **Costs (Part C, serial, r42 convention):** h0 3.3 s/game, h1 6.2,
  h1@192 16.8, h3@64 16.3 — the cost match for Part B is exact to 3 %.

  | part | cells | pooled |
  |---|---|---|
  | A: h1@64 vs h3@64 | 42.5 / 43.3 | **42.9** |
  | B: h1@192 vs h3@64 (cost-matched) | 43.1 / 43.3 | **43.2** |
  | D: h0@64 vs h3@64 (anchor) | 15.0 | **15.0** |

  **A: the two extra turns are worth ~+7** — more than double the
  pre-registered 1–3 band. **B is the round's finding: tripling the
  iterations bought back +0.3 of the 7.1-point gap.** If the h1 leaf
  were an unbiased-but-noisy estimate, 192 iterations would climb the
  r27 curve (~+2 for 1.58 doublings); it climbed nothing, so the h1
  evaluation is systematically *biased* — the net misjudges
  states one turn after an action in a way more samples cannot fix,
  and the rollout's remaining turns are doing bias correction, not
  noise reduction. r42's iteration gains live at h3 because the deep
  leaf is nearly unbiased; iterations and horizon are not
  interchangeable currencies.

  **D is the mechanism, exposed:** h0 was pre-registered as "should
  approximate the 1-ply `net` pilot, ≈45 %" and instead lost by 35
  points. The 1-ply pilot's sims *settle* the state before the net
  scores it; h0 scores the raw successor — spell still on the stack,
  cost paid, benefit invisible — and the net reads that as pure loss,
  so the h0 bot is punished for ever acting. Settlement is
  load-bearing for every net consumer, which both explains the h1
  bias (one turn settles the stack but not the exchange it started)
  and independently corroborates the leaf-census diagnosis: the net
  is only trustworthy on the state shapes it was trained on, and
  moving the leaf distribution toward the training distribution (or
  `head_leaf` toward the leaves) is the live lever, not shortening
  the path to the leaf.

  **Disposition:** value-equivalence / short-horizon direction
  CLOSED with a replicated mechanism. `mcts-net-h0/h1/h1-192`
  profiles stay as controls. Search cost work continues on the
  engine action loop (PERF.md (-12)) and ladder-gated rollout
  early-adjudication, not on horizon.

- 🟢 **Round 43 (2026-08-19) — the first instrument-driven piloting
  round: chump blocks measured +0.9 and adopted, early-stop closed as a
  null with its mechanism, and three zero-incidence lessons about what
  mirror ladders can and cannot see.** Pre-registered in
  `.ladder/run_r43_piloting.sh` / `run_r43b_gates.sh`.

  **The instruments came first and set the agenda.** `CRAB_DECISION_LOG`
  (every human action logged beside what the bot would have done from
  the same position) and the v2 replay recorder + `replay_view` narrator
  turned four recorded human games into a defect list the ladder never
  surfaces: the bot at 5 life taking 4 to the face holding a chump, a
  planeswalker ultimating after ten unpressured turns, two banked
  prepared Ancestral Recalls never cast, zero non-mana activations
  across all games. The same recordings caught three interactive-play
  bugs bot measurement is structurally blind to (the Forum of Amity
  tap-cost soft-lock, the modal-castability affordance hiding Quandrix
  Charm, decision-log noise classes) — fixed alongside, each with a
  regression test from the recorded position.

  **Part A — early stop alone, the untested half of round 29's adaptive
  arm.** Serial cost 27.6 / 99.3 / **102.4** s per game
  (64 / 256 / 256+stop): the stop saves *nothing*. Parity 50.1 / 50.2
  (±0.4) vs fixed-256 and margin +2.1 / +2.5 vs 64 (reproducing round
  42) complete the picture: the confidence-bound condition simply never
  fires at 256 iterations' per-arm visit counts — strength identical
  because behavior is near-identical. Closed with mechanism; the client
  latency path remains leaf-eval batching (`PERF.md` candidate 11).

  **Part B — six piloting flags, each A vs its control (same weights
  minus the flag), mirror sealed decks:**

  | flag | vs | l43 | l97 | verdict |
  |---|---|---|---|---|
  | chumpblocks | gang | **51.0** [50.9, 51.1] | **50.8** [50.7, 50.9] | **ADOPTED** |
  | buff2for1 | gang | 50.0 ±0.00 | 50.0 ±0.00 | zero incidence |
  | convlands | gang | 50.0 ±0.02 | 50.0 ±0.02 | zero incidence |
  | walkerchip | atk-sim | 50.0 ±0.00 | 50.0 ±0.00 | zero incidence |
  | abilarms | gang | 48.0 [47.8, 48.2] | 50.5 [50.4, 50.7] | seed-split, null-to-negative |
  | mcts-net-prep | mcts-net-deep | 49.9 ±0.49 | 49.9 ±0.49 | null |

  **Chump blocks: +0.9 replicated, and the mechanism is structural, not
  a weight.** The block menu was built from *profitable* blocks only —
  a greedy pass that found none returned a bare "no blocks" and the
  simulations never ran, so no valuation could ever choose a chump.
  The flag adds desperation candidates (unblocked damage lethal within
  two swings) and lets the existing sims judge them. The tiny ±0.08
  bars are the signature of a targeted fix: nearly every pair splits
  identically and only chump-relevant games diverge. Adopted into
  `EvalWeights::default()` (measured on the `gang` base; determinize
  rides on top as with every earlier layer). Golden-trace seed 2
  re-blessed: same winner, same turn and action counts, one changed
  block. Largest adopted piloting layer since gang-blocks — and it
  came from reading two recorded games, not from a 12 000-game sweep.

  **The zero-incidence trio is a lesson, not a failure.** buff2for1
  (kill the creature under the opponent's own pump), convlands
  (converge-aware land drops) and walkerchip (chip an unfinishable
  walker) all measured *exactly* 50.0 with near-zero pair divergence:
  the triggering situations do not occur in heuristic sealed mirrors —
  these pilots make ~one instant-speed cast per 60 games (round 39's
  probe), two-color builds rarely diversify basics, and the gate pools
  contain no walkers. All three stay in-tree, default off, as
  human-facing behaviors the mirror instrument cannot price. A gate
  that cannot refute is not a gate; the decision log is the instrument
  of record for this class.

  **abilarms is the honest failure: enumeration without judgment can be
  worse than neither.** Generic activated-ability candidates split the
  seeds hard (−2.0 / +0.5, both "significant" alone — the pool-level
  clustering lesson again, twelve decks per ladder seed). Auto-aimed
  activations firing on the sims' say-so are evidently harmful in some
  pools. Not adopted; retry wants per-pool attribution of *which*
  abilities fired before another cell is spent.

  **mcts-net-prep: the menu-reservation theory of the banked Recalls is
  dead** (49.9 both seeds). The probe had already killed the
  eval-can't-see-card-advantage theory (the eval prices a hand card at
  4 and the heuristic casts the banked Recall correctly — the recorded
  passivity remains unexplained, now bounded to the net-pilot side).

  **What was built.** The decision shadow log with noise equivalences
  (pass ≡ empty declarations, mana taps skipped, payment retries
  deduped); replay v2 (first-appearance card-name tables) +
  `replay_view`; `EvalWeights` flags `chump_blocks` (adopted),
  `buff_2for1`, `converge_lands`, `walker_chip`, `ability_arms`,
  `prepare_arm` (default off) with ladder profiles; the Forum of Amity
  tap-cost rollback on `ManualTapRequired`; every-mode castability
  probing in the affordance sweep; `mcts-net-256es` kept as the
  measured early-stop control.

- 🟢 **Builder v3 (2026-08-17) — quality-and-curve-aware shape ranking
  is worth ~+3 points of deck strength, replicated on both harness
  seeds; best-of-N selection on top adds nothing measurable; and the
  first gate run is a kept lesson in what the unit of measurement is.**
  Pre-registered in `.ladder/run_builder_v3_gate.sh`.

  **Motivation.** `builder_v2` repaired the per-card picks
  (`score_card_quality`) but left the *shape ranker* body-blind:
  `static_build_score` ranks candidate color shapes with the legacy
  pip/type/curve-bucket scorer, so a pair of individually-worse colors
  could outrank the pair holding the bombs. And nothing anywhere
  enforced a curve — the only signal was the flat CMC bucket bonus.
  `static_build_score_v3` scores shapes with `score_card_quality` and a
  soft `curve_penalty` (4/point below five early spells, 3/point above
  six five-plus spells), behind `SimConfig::builder_v3`, default
  **off** — the score feeds the gauntlet's shape softmax, so flipping
  it changes every generated field including the ladder's sealed gate
  decks (see adoption below). Separately, the client's sealed opponent
  was one noisy sample from the builder distribution; it is now
  `best_build_v3` — best-of-16 samples under the v3 static judge.

  **The first run measured its own design instead of the builder.** At
  the v2-gate shape (800 games × 12 pools) the two harness seeds
  contradicted each other in *both* races — v3 single 47.4/51.8, b16
  55.4/48.6 — each side "significant" by its own Wilson interval.
  Games are not independent units here: each pool is one deck-pair
  matchup and one *draw* per side from a builder distribution, and the
  pool-level sd is ~15 points, so 12 pools carry ±6.5 no matter how
  many games each gets. The same 9,600 games per race respent wide and
  flat (50 games × 192 pools) resolve it; the gate binary now prints
  the pool-level *t*-interval itself and bases its verdict on that,
  not Wilson (`t95`, exact table — the ±2·se habit at small n is the
  round-41 lesson restated on a new axis). Narrow-run artifacts kept
  as `.ladder/builder_v3_gate_s{43,97}.narrow.txt`.

  **Results (50 games × 192 pools per cell, pool-level 95% t):**

  | race | seed 43 | seed 97 |
  |---|---|---|
  | v3 single vs v2 single | **52.53** [50.55, 54.51] | **53.96** [52.08, 55.84] |
  | best-of-16 v3 vs v2 single | **53.61** [51.64, 55.59] | **52.44** [50.42, 54.45] |

  All four intervals clear 50%. Pooled: the scoring change alone is
  **+3.2** (53.24 over 384 pools), the client recipe **+3.0** (53.03).

  **Selection is worth ~nothing once the scoring is right.** b16 sits
  on top of v3-single (53.0 vs 53.2 pooled), so picking the best of
  sixteen mild-jitter samples by the same score that built them adds
  no strength the argmax build didn't already have — consistent with
  round 16, where static-judged best-of-32 was the control a *learned*
  judge beat by 10 points. If the client opponent should climb
  further, the lever is a better judge (the deck net, or the round-16
  trust-region climb), not more samples under this one. Best-of-16
  stays in the client anyway: milliseconds, and it hedges the jitter
  tail.

  **Adopted:** `static_build_score_v3` + `curve_penalty` as the
  client's sealed-opponent builder (`random_sealed_opponent_packs` →
  `best_build_v3(pool, 16, seed)`), deterministic in the seed as
  before. **Not adopted (yet): `builder_v3` as the default**, although
  +3 replicated is a real adoption case — flipping it changes the
  training field and the ladder's `sealed_archetypes` mirror decks, so
  every recorded gate reference (champion 52.7/51.2 included) stops
  being comparable. That flip is its own pre-registered round:
  re-baseline the champion and gate opponents on v3 fields first, then
  ask whether a pilot *trained* on v3 fields differs (untried; the
  training-deck-quality question in `TODO.md`).

  **What was built.** `SimConfig::builder_v3` (default off);
  `static_build_score_v3` / `curve_penalty` (`recommend.rs`, unit
  tests pin the composition and the penalty values);
  `heuristic_sealed_build_v3`, `build_candidates_cfg`,
  `static_deck_score_v3` (the v2 judge stays pinned as the control),
  `best_build_v3` (`selfplay.rs`); `--gate-builder-v3` with
  `CRAB_GATE_POOLS` and the pool-level verdict (`selfplay_train`).
  Training-path outputs are byte-identical by construction —
  `heuristic_sealed_build` and `build_candidates` are untouched.

- 🟢 **Round 42 — the search is still the biggest lever by a factor of
  six, and the client has been running at a quarter of the measured
  best. Part B killed deliberately.** `.ladder/run_r42_search_scaling.sh`.

  **The framing.** Round 41 put the best net change in the program at
  +0.4 paired. Round 26 put the search at +4 to +5 over the same net as
  a 1-ply pilot, and round 29 found raw iterations to be the only MCTS
  lever that pays. Yet everything adopted — the client's `local_bot`,
  every default profile — runs 64 iterations, while round 27 measured
  256 at +2.0 over it. That gap was never closed, and fifteen rounds
  went after the net instead.

  **Correction to round 27, found while designing this.** That curve
  (24→64→128→256 = 49.4→53.0→54.35→55.0 % vs the champion) used 1200
  games per cell, which is **±2.8**. Its 128→256 step of +0.65 was well
  inside its own noise, so "still climbing at 256" was never
  established. Same small-n optimism round 41 caught in the r38 band;
  two independent instances now, and the lesson is the same one.

  **Part A — 256 vs 64, head to head, 48 000 games.** Head-to-head
  rather than each-vs-pilot: it measures the difference directly
  instead of subtracting two noisy numbers, and the antithetic seat
  pairing applies to it.

  | ladder seed | 256 vs 64 |
  |---|---|
  | 43 | 52.1 % [51.6, 52.5] |
  | 97 | 52.7 % [52.3, 53.1] |
  | **pooled** | **52.4 %** |

  **+2.4 points for 4× the search**, at ±0.41 per cell — fifteen times
  round 27's precision, and it confirms that round's implied +2.0. Six
  times the entire encoder program's replicated yield, from a config
  constant, with no training run.

  **Cost, and the distinction that decides adoption.** Single-threaded
  seconds per game: 64 → **33.0**, 128 → **56.8**, 256 → **121.9**,
  512 → **249.4**. Linear in iterations (≈ 2.1 + 0.48·iters), so the
  strength curve is logarithmic against a linear cost.

  The ladder gets ~22 CPU-seconds per game at 256 because 23 concurrent
  games batch their net evaluations together. **The client cannot do
  that** — one game, one decision at a time — so 121.9 s/game is the
  client-facing figure, not 22. At the tens of searched decisions in a
  game that is ~2–3 s per decision at 256 against ~0.5–0.8 s at 64. The
  win is real and so is the cost; this is a trade, not a free lunch.

  **Part B — 512 vs 256 — started and killed at 3 h 19 m of a ~15 h
  two-cell run.** Pre-registered as underpowered, and the cost estimate
  turned out worse than scoped (368 s/game head-to-head). Twelve more
  hours to produce a ±1.4 estimate of a step that is probably +0.5–1.0
  is a bad trade, and the pre-registration said so before the machine
  was spent. **The curve above 256 is unresolved and unaffordable at
  this program's current precision** — resolving a +0.5 step at 512
  would cost ~35 h per cell. Recorded as a live unknown, not as a flat
  curve.

  **What should happen next, and did not happen here.** The client
  should not stay at 64. 128 is the unmeasured middle (~1–1.5 s per
  decision) and is the obvious default candidate; measuring 128 vs 64
  head-to-head costs ~2 h per cell against Part B's 7.6. Better still,
  iterations are a natural *difficulty dial* rather than a constant —
  the opponent already has a pack-count handicap axis, and search depth
  is the second one.

- 🟢 **Round 41 — encoder v7 replicates. Eight of eight paired
  differences positive, and the effect is real; the champion still does
  not move, for a reason the round only exposed by running four seeds.**
  Design pre-registered in `.ladder/run_r41_v7_replication.sh`: four
  training seeds (43, 97, 151, 199), paired within seed, control =
  `--ablate hist,exp,ctr` (byte-identical to v6), control cells gated
  under `CRAB_ABLATE` so its never-trained columns are not fed live
  features.

  **Paired differences (v7 − control), 24 k games per cell:**

  | seed | atk-sim | gang |
  |---|---|---|
  | 43 | +0.10 | +0.30 |
  | 97 | +0.45 | +0.30 |
  | 151 | +0.75 | +0.70 |
  | 199 | +0.15 | +0.45 |
  | **mean** | **+0.36** | **+0.44** |
  | 95 % (t, n=4) | [−0.12, +0.84] | **[+0.14, +0.74]** |

  **All eight are positive.** Treating the four seeds as the independent
  units (both gates share a net, so they are not two independent tests),
  four-for-four in the same direction on both gates is p ≈ 0.004 under
  the null. `gang`'s interval excludes zero on its own; `atk-sim`'s does
  not, and the *t* interval is the honest one — the script originally
  printed ±2·se, which at four seeds is badly optimistic (t(3) = 3.18),
  and has been fixed to use *t* and to print the positive count.

  So the round-40 reading survives: **encoder v7 is worth roughly +0.4
  against its own parity control.** After round 12, round 28f and four
  other flat rounds, that is the first representation change in the
  program with a replicated effect — and the leaf census supplied its
  mechanism in advance rather than after the fact (the `hist` block is
  1.8–3.6× denser in the positions the search evaluates than in the rows
  the trainer fits).

  **And yet the champion stays, which is the part four seeds bought.**
  Absolute pooled levels:

  | | atk-sim | gang |
  |---|---|---|
  | r41 control (v6 parity, 4 seeds) | 52.45 | 50.90 |
  | r41 v7 (4 seeds) | 52.81 | 51.34 |
  | champion | 52.7 | 51.2 |
  | r38 band (2 seeds) | 52.5–52.85 | 51.1–51.35 |

  **The control ran below the r38 band on both gates.** r38 and this
  control are the same recipe, so the four-seed mean is simply a better
  estimate of that recipe's level than r38's two-seed band was — the
  band was a lucky pair. Round 40 therefore measured a real +0.3 against
  an optimistic reference, and got the right answer for partly the wrong
  reason.

  The consequence is unglamorous: v7 lifts a slightly weak cohort back
  to roughly champion level. **+0.11 / +0.14 over the champion in
  absolute terms is not an adoption case**, however clean the paired
  effect is. What replicates is the *treatment*, not a champion-beating
  net. To move the champion, v7 needs to be stacked on something else
  that is also worth a few tenths, or run in a regime that starts higher
  than this one did.

  **Adopted:** encoder v7's quality claim, at +0.4 against parity, with
  the format already in-tree. **Not adopted:** any champion change.

  **Method notes, both of which outlive this round.** Pairing is what
  made a +0.4 effect legible at all: the control arm alone spans 52.20
  to 52.60 on atk-sim across seeds, so an unpaired four-seed comparison
  would still have been swamped. And the round-38 fleet made 95-minute
  runs possible three rounds before any experiment design used the
  slack — two-seed arms were leaving power on the table from r38 onward.
  Four paired seeds is the new floor for anything claiming under a point.

- 🟢 **Round 40 — the first encoder change since round 12 that isn't
  flat, a mechanically-explained retraction of round 28f's combat
  verdict, and a clean negative: combat rows cost the search 3 points.**
  Pre-registered in `.ladder/run_r40_encoder_v7.sh`.

  **The instrument came first, and it reshaped the round.**
  `selfplay_train --feature-census N` (new; no net, no GPU, one
  self-play pass) reports how often each encoder feature is non-zero in
  recorded positions. Run before any arm was launched, on 300 games /
  29 696 positions / 1.04 M objects under the v6 encoder and the
  round-39 recorder:

  | block | slots | non-zero rate |
  |---|---|---|
  | `hist` (new) | globals 43..=54 | 3.6–51.7 % of positions |
  | `exp` (new) | object feats 45..=47 | 0.13–0.16 % of objects |
  | `ctr` (new) | object feats 48..=52 | 0–1.6 % of objects |
  | round-28 combat | globals 36..=40, feats 37..=39 | **0.00 %** |
  | coarse combat phase / attackers | globals 11, 19; feat 10 | **0.00 %** |
  | blocking / blocked | feats 28, 29 | **0.00 %** |

  **Thirteen features are identically zero in every training row the
  program has ever recorded.** The recorder snapshots at each new turn,
  at post-combat main, and at end step: combat is over by the first and
  hasn't begun by the second, so no training row has ever been a combat
  row. Consequences, in order of importance:

  1. **Round 28f's combat arm could not have measured anything** — it
     trained the block against an all-zero column while its "v5-parity"
     control ablated a block that was already blank. The two arms were
     the same experiment. That entry now carries the caveat; its
     keyword/exile half is unaffected and its null stands.
  2. The columns are **live at inference** (the attack and block sims
     evaluate mid-combat positions), so the search fed real values into
     weights that never received a gradient.
  3. `exp` and `ctr` did not earn their own cells at 0.16 % occupancy.
     Two arms, not three.

  **Arms.** E = full v7 (`hist`+`exp`+`ctr`), recorder unchanged.
  C = full v7 + `--record-combat` (snapshots at declare-attackers,
  declare-blockers, first-strike/combat damage, end-of-combat), which
  brought every dead feature live (g36 19.0 %, g37 7.3 %, g38 18.7 %,
  g39/40 5.5 %, g11 45.0 %). Two training seeds each, r38 cells as the
  control, r38 recipe plus one flag.

  **Pilot gates** (1000 games/archetype × sealed × 2 ladder seeds; each
  pooled number is 4 cells / 48 k games):

  | arm | vs atk-sim | vs gang | cells (atk-sim / gang) |
  |---|---|---|---|
  | r40 E | **53.0** | **51.5** | 53.2 52.8 53.2 52.8 / 51.6 51.2 51.7 51.5 |
  | r40 C | **53.0** | **51.5** | 52.9 52.9 53.4 52.7 / 51.3 51.4 51.9 51.4 |
  | r38 control | 52.5–52.85 | 51.1–51.35 | |
  | champion | 52.7 | 51.2 | |

  **Arm E is +0.3 on both gates, sign-consistent in all eight cells and
  reproducing across training seeds (53.0/53.0, 51.4/51.6).** After
  round 12, round 28f and four other rounds of flat inputs, that is the
  first encoder change to move a gate at all. It is *not* an adoption
  case: r38's own two seeds spread 0.35 / 0.25, the same size as the
  effect, so two seeds cannot separate "+0.3" from "a good pair of
  seeds". The honest statement is a positive that needs replication —
  and the cheap replication is more seeds on arm E, not another block.

  (Round 41 ran that replication and the effect held at +0.36 / +0.44
  paired over four seeds. It also showed the r38 band used as the
  reference here was a lucky two-seed pair sitting above the recipe's
  real level, so this +0.3 was measured against an optimistic control —
  right answer, partly wrong reason. See round 41.)

  **Arm C adds exactly nothing to the pilot** (identical to E on both
  gates, to 0.05). One nuance kept for the record: C matched E while its
  500 k-row window covered ~55 % as many games, because it records
  ~82 % more rows per game (43.9 M vs 24.1 M rows over 250 k games — the
  census predicted +82 % and the runs delivered it). So the combat rows
  at least repaid the coverage they cost. Not separable from both
  effects being zero.

  **Arm C's search gate is a clean negative, and the control that
  proves it was missing from the pre-registration** — the script gated
  only arm C here, which would have left the narrowing attributable to
  encoder v7 as readily as to combat rows. The arm-E cells were run
  afterwards to close that. `mcts-net-deep` vs `net`, same weights both
  sides:

  | net | pooled | cells |
  |---|---|---|
  | r40 C (combat rows) | **51.9** | 51.2 / 52.1 / 51.0 / 53.2 |
  | r40 E (encoder only) | **54.75** | 55.4 / 54.7 / 53.2 / 55.7 |
  | r38 / r39 references | 54.85 / 53.95 | |

  The round-40 script only gated arm C here, which would have left "the
  search narrowed" attributable to encoder v7 as easily as to combat
  rows. Running the arm-E cells afterwards settles it: **arm E pools
  54.75, landing on the r38 reference of 54.85, while arm C pools
  51.9** — a 2.9-point loss that tracks the recorder flag and not the
  encoder, with the two arms' pooled intervals disjoint (±~1.0 each on
  four cells of 100 games). Since the
  two arms are *equal as 1-ply pilots*, a narrower gap cannot mean the
  pilot learned what the search knew — the searched player got worse.
  Mechanism: the value head is the rollout leaf evaluator, and arm C
  fit it to a distribution ~45 % of which is mid-combat, moving its
  calibration on exactly the states the rollouts reach.

  **Verdict.** `--record-combat` is a **negative**: it buys nothing as a
  pilot and costs 3 points as a search evaluator. It stays in-tree,
  default off, as the instrument that makes the combat features
  trainable at all — and as the measured control for anyone who
  re-proposes them. Encoder v7 is adopted as the format (information
  superset, per-block ablation, legacy checkpoints zero-pad in *both*
  net implementations now, 44.2 games/s against r38's 43.5–44.3, so no
  encode cost) with a **provisional** quality claim at +0.3 that the
  next round should try to replicate before the champion moves.

  **Cross-metric lesson, worth more than the result.** Arm C posted
  val_auc 0.8563 and val_policy 0.8855, both program records, and both
  were noise of different kinds. The AUC is measured on the arm's *own*
  holdout, which is ~45 % combat rows for C and 0 % for E — different
  validation distributions, not a better net (combat positions are
  closer to resolution and easier to call). The val_policy record was a
  seed: arm means are 0.829 (E) vs 0.846 (C) against within-arm spreads
  of 0.064 and 0.079. Third time a headline metric has failed to
  convert (r33 ranking, r39 SV, now this). **Holdout AUC is not
  comparable across arms that change what gets recorded.**

  **What was built.** `--feature-census` (occupancy per feature, block
  labelled, reads the live ablation so it also verifies a control is
  blanking what it claims); encoder v7 — `hist` globals 43..=54,
  `exp` feats 45..=47, `ctr` feats 48..=52, `OBJ_FEATS` 45→53,
  `GLOBAL_FEATS` 43→55, `SHARD_VERSION` 8; the ablation API replaced
  by a name table (`ABLATION_BLOCKS`) shared by the trainer and the
  ladder, with unknown names an error rather than a silent no-op —
  a typo'd control is a second copy of the arm; `--record-combat`;
  and `Trainer::load` now zero-pads older-generation checkpoints the
  way `PlayNet::load` always has. That last one was a latent bug the
  bump exposed: candle's `VarMap::load` is exact-shape, so any
  `--use-best` pilot from before an encoder bump loaded fine on the
  ladder and killed the trainer at startup. Both loaders now pad from
  one shared table, parity-tested.

  **Follow-up: the leaf-side census, and it explains arm E.** The census
  now reports two columns — `train` (the recorder's rows, what the
  weights are fit to) and `leaf` (the simulated positions the attack and
  cast searches evaluate, via the existing `leaf_capture` hooks). 200
  games: 19 688 training positions against 3 648 leaves.

  Never-trained and live at inference:

  | feature | train | leaf |
  |---|---|---|
  | g11 coarse combat phase | 0.00 % | **69.3 %** |
  | g38 damage / end-combat one-hot | 0.00 % | **69.3 %** |
  | g19 attackers | 0.00 % | 1.2 % |

  **Roughly two-thirds of everything the search evaluates is a
  post-combat-damage settled state, and the feature that says so has
  never once been non-zero in training.** The net cannot tell a combat
  leaf from a main-phase board; every position it ranks looks like the
  latter. Note g36/g37 (declare-attackers/blockers) are 0 % on *both*
  sides — `simulate_attack_outcome` runs combat to completion, so the
  leaf is the settled state, not the mid-combat one.

  The trained-but-skewed half is the more useful half. Ranked by
  leaf/train ratio over features present in ≥5 % of leaves, **the top
  ten are the round-40 `hist` block entire**, plus exile counts:

  | feature | ratio | train → leaf |
  |---|---|---|
  | g49/50 creatures died | 3.6× / 3.0× | 10.5 → 37.4 % |
  | g51/52 left graveyard | 2.7× / 1.8× | 3.5 → 9.4 % |
  | g53 cards exiled | 2.5× | 7.5 → 19.1 % |
  | g43/44 life gained | 2.2× / 2.0× | 6.9 → 15.0 % |
  | g45 instants cast | 2.2× | 17.4 → 37.4 % |
  | g47 spells cast | 1.8× | 51.9 → **95.1 %** |

  **This is a mechanism for arm E's +0.3, arrived at independently.**
  The block the recorder-side census picked as "the only one with real
  occupancy" is also the block the search meets 2–3.6× more often than
  the trainer does — the features aren't merely present, they are
  present disproportionately in the states the net is actually asked to
  rank. Consistent with the gate result rather than proof of it, but it
  is the first mechanistic story any encoder change has had.

  The whole mismatch is **one axis**: the leaves are uniformly *later in
  the turn and after combat* than the training rows. Creatures have
  died, spells have been cast, damage is marked, combat is over. Nothing
  scattered about it.

  **What this does not say is "close the gap".** Arm C closed the g38
  gap and cost 3 points of search strength, and round 13 measured the
  net's leaf AUC as *higher* than its snapshot AUC (0.84–0.85), so the
  mismatch is not obviously hurting prediction at all. A narrower
  experiment than arm C exists — record only the settled end-of-combat
  state, the shape that is actually 69 % of leaves, instead of all four
  combat steps (arm C's rows were only ~19 % that shape while diluting
  the window by 82 %) — but it inherits arm C's warning and should not
  be run on the strength of this table alone.

- 🔴 **Round 39 — both levers are nulls, and the belief redeal's null
  is the cleanest measurement in the program.** Arms SV
  (`--search-value-weight 0.25`) and OPP (`--opp-head`), each the r38
  recipe plus one flag, two seeds, r38 cells as controls.

  **Pilot gates** (1000 games/archetype × sealed × 2 ladder seeds, 4
  cells pooled per number):

  | arm | vs atk-sim | vs gang |
  |---|---|---|
  | r39 SV | 52.58 | 51.08 |
  | r39 OPP | 52.65 | 51.12 |
  | r38 control | 52.5–52.85 | 51.1–51.35 |
  | champion | 52.7 | 51.2 |

  Both arms land inside the r38 band and on the champion. Fourth
  consecutive round in which a real capability gain fails to convert:
  the champion is unmoved since round 22.

  **SV: the value head absorbs the search's values and it changes
  nothing.** The fit converges (`policy_sv` 0.016/0.017 weighted MSE)
  and `val_policy` is the best on record — **0.854/0.846, spread
  0.009**, against the r38 controls' 0.831/0.691 (spread 0.140). So
  the term does something real and *stabilising*: the ranking metric's
  seed lottery collapses. But `mcts-net-deep` vs `net` on these nets
  pools **53.95**, statistically the r38 figure (54.85) — search's
  edge over its own 1-ply pilot is untouched. Training the value head
  on search values does not teach the pilot what the search knows.
  Reading it against r27 (labels null at 50 k) and r38: the value
  target is not the binding constraint at any provenance, variance, or
  scale tried.

  **OPP: the belief head learns and the redeal is worth nothing.**
  `loss_opp` 0.098 → 0.080 (plateaued by step 14 k; a constant
  base-rate predictor scores ≈0.135) and s43 posted **AUC 0.8308**, the
  program record — an aux-target trunk gain, the first thing to beat
  r38's 0.8236. The payoff gates, identical weights both sides:

  | path | pooled | cells |
  |---|---|---|
  | sims (`bdet1` vs `det1`) | **50.10** | 50.2 / 49.9 / 50.3 / 50.0 |
  | rollouts (`bdeep` vs `deep`) | 51.15 | 50.3 / 52.1 / 50.9 / 51.3 |

  The sim path is the sharpest null the program has: four cells at
  ±0.4 over 12 k games each, all within 0.3 of parity. The rollout
  path's +1.15 is four cells at ±1.9 — sign-consistent, magnitude
  unresolved, and not separable from noise at this width.

  **Why the ceiling is low, and it is structural.**
  `determinize_hidden` already redeals from the opponent's *true*
  unseen cards — it knows their deck and only mis-sorts hand vs
  library, so the head's entire job is picking which ~5 of ~35 known
  cards are held. And a belief head can only learn tells its data
  contains: these pilots make almost none (the SoS probe measured 42
  cleanup discards per 60 games against **one** instant-timing cast;
  `atk-hold` gated 49.4 %). "They left two Islands untapped" carries
  little in a distribution where nobody holds up mana on purpose.
  Whether that generalises to a human opponent who does is a question
  a mirror ladder structurally cannot answer.

  **Not adopted; both stay in-tree, default off** (`--search-value-weight`
  0, `--opp-head` off, `mcts-net-bdeep`/`net-bdet1` as measured
  controls). `val_policy`'s variance collapse under SV is the one
  result worth re-using: it is cheap, and the ranking metric's seed
  spread has been a recurring nuisance since round 33. Open, and now
  in `TODO.md`: BCE cannot distinguish "belief is weak" from "belief
  does not matter" — a top-k recall diagnostic against the
  uniform-over-unseen baseline decides which, and the two answers want
  opposite follow-ups.

  **What was built** (all default off, `.ladder/run_r39_sv_belief.sh`).
  SV: captures carry per-arm rollout counts, and the policy step gains
  a visit-weighted MSE of the win head against the search's per-arm
  means on the same successor batch it already forwards — one trunk
  pass, both objectives. It *refuses Gumbel captures by construction*:
  those values are improved-policy logits, and regressing a sigmoid
  onto logits is a category error. OPP: shard v7 carries the
  opponent's true held names (recorder-only — the encoder still cannot
  see them), `head_opp.*` fits them with multi-label BCE at the
  auxiliary weight in both net implementations (parity-tested), and
  `determinize_hidden_belief` redeals by Efraimidis–Spirakis sampling
  over hold-odds. The uniform redeal stays a separate untouched
  function, so golden traces are byte-identical. A belief profile on a
  headless net is *inert* — the ladder prints which redeal is running
  and the run script aborts on INERT, because a cell that silently
  gates a no-op is worse than no cell.

- 🟡 **Round 38 — champion-scale distillation lands in the champion
  band, not above it; the champion survives; the mixed fleet is a 29×
  generation win.** The r36 live next step on the r37 footing: 250 k
  games/seed, mixed fleet (128 MCTS-64 + 128 value actors, 256
  threads), separate policy head, pilot = `nets_r28f_full_s43`
  (v6 champion-class), seeds 43/97, best-on-AUC artifacts, gates
  pre-registered in `.ladder/run_r38_champion_distill.sh`.

  **The fleet, first measured at release scale: 43.5/44.3 games/s
  cum** against the all-MCTS r35/r37 recipe's 1.5 — 29× — with the
  learner fully fed (~91 k steps in ~95 min per seed; champion-era
  runs got 70 k) and ~10.3 k searched games riding along per seed
  (~200 k decisions, the deque cap). 250 k games + 90 k steps now
  costs 1.6 h, which changes what is affordable to ask.

  **Part B, adoption (1000 games/archetype × sealed × 2 ladder seeds,
  pooled per net) — no clear, champion stays:**

  | vs champion's | atk-sim 52.7 | gang 51.2 |
  |---|---|---|
  | r38 s43 | 52.85 | 51.35 |
  | r38 s97 | 52.50 | 51.10 |

  Both candidates land *on* the champion to within ±0.2 — inside the
  ~±0.55 that a difference of pooled cells carries (r36) — at the top
  of the r30 fresh-draw band (50.9–52.1 / 52.7–53.7). The
  pre-registered r30 rule reads this as champion-band draws, not an
  improvement: **champion-scale distillation with the separate head
  does not push a pilot beyond the champion distribution.** With r27
  (MCTS labels null at 50 k), r36 (+0.40/+0.48 at 20 k vs weak
  controls), and this, the label/target-side account now spans three
  scales and keeps landing in the same place: distillation is real but
  small, and it does not compound into a stronger pilot at any scale
  tried. Screen note along the way: s97 posted **val_auc 0.8236**
  (above r18's 0.8204 own-distribution record, same caveat) while its
  `val_policy` was the *weak* seed (0.69 vs s43's 0.83, spread 0.14
  against r37's 0.025) — the AUC/ranking dissociation, fourth
  appearance.

  **Part C, the search question at scale — r36 replicates:**
  `mcts-net-deep` vs `net`, same weights both sides: s43 55.5 pooled
  (54.8/56.2), s97 54.2 (53.6/54.8). Search's margin over its own
  1-ply pilot on these 250 k-game nets is as large as it was on the
  20 k r35 distil nets (55.0) and larger than on the champion (52.15).
  Sixty-four iterations remain not-removable, and the r36 coupling
  survives the head: the pilot the gate races does not consume the
  policy head, so nothing about a stronger trunk shrinks the search's
  edge.

  **Part D, exploratory (2 cells): gumbel vs deep with the s43 scale
  head pools 49.3** (48.8/49.8) — the r37 null does not move with a
  champion-scale head. Consistent with the prior being distilled from
  UCB1's own conclusions; the genuinely different bet (gen-1: a head
  trained on completed-Q targets from Gumbel actors) remains untried.

  **Round verdict.** The program's strongest system is unchanged —
  champion(-class) net + mcts-net-deep at inference — and the
  distillation-for-pilot-strength thread is now bounded at three
  scales. What the round actually bought: the mixed fleet (a 29×
  measured generation win that makes 250 k-game experiments a
  ~90-minute question), two more champion-class spares
  (nets_r38_head_*), the first scale-trained policy heads, and the
  sharpest statement yet of where the remaining headroom is — inside
  the search, or in what the search consumes, not in the pilot's
  weights.

- 🟡 **Round 37 — the separate policy head works (ranking without the
  value tax); the Gumbel search that consumes it is a null at 64
  iterations.** Round 36 ended
  with a prescription, not just a negative: distillation into the win
  head makes the search *stronger* because the search eats the same
  scalar as its UCB1 leaf reward, and any future attempt must break
  that coupling — "a separate policy head the search does not consume,
  or a search whose leaf value is frozen." This round builds the first,
  and pairs it with the allocator that was designed for this exact
  budget shape.

  **The head.** `head_policy.*` (h2 → 1, ~257 params) over the shared
  trunk, in both implementations: candle trains it, the engine loads it
  all-or-nothing (like `attn.*`, because unlike the life/length heads
  it is *consumed at inference*) and serves it through a new
  `NetEvaluator::eval_policy` — the batched GPU collator carries it in
  the same forward for one extra `[B,h2]×[h2,1]` matmul. Round 35's
  distillation loss now trains this head when `--policy-head` built it;
  the win head receives no policy gradient, which is asserted directly
  (`policy_steps_do_not_touch_the_win_head`: `head_win.weight`
  bit-identical across policy steps). That is the mechanism that should
  cancel round 33's tax — the −0.010/−0.020 AUC the shared-head version
  paid was two objectives fighting over one scalar. The shared-head
  path stays bit-reachable as the control (no `--policy-head`), and a
  candle↔engine parity test pins the head's logit at 1e-4.

  **The search.** `MctsConfig::gumbel` (off by default; profile
  `mcts-net-gumbel`, 64/h3, the `mcts-net-deep` shape): Sequential
  Halving over Gumbel-perturbed prior logits, arms scored
  `g + logit + σ(q̂)`, σ(q̂) = (c_visit + max_visits)·c_scale·q̂ at the
  reference constants (50, 0.1) — Danihelka et al., ICLR 2022. Priors
  are the policy head over each arm's *successor state* (what the head
  was trained on); on a headless net the profile falls back to
  log-softmax candidate scores and *says which one it is running* at
  startup, because the fallback is a legitimate control arm (allocator
  alone) but a different experiment. This is not round 29 re-run: that
  negative fed the same scores in as a P-UCT visit bonus at temp 4,
  which starves the arms search exists to rescue; SH visits every
  survivor equally and halves on observed reward, with a
  policy-improvement guarantee at small budgets. The halving plan is a
  pure function (`sequential_halving_plan`, exact-spend at the 8×64
  profile shape: (8,2)→(4,6)→(2,12)) with property tests; the plan is
  the budget policy, so gumbel ignores `exploration`/`prior_weight`/
  `early_stop`/`extend_close`.

  **The target.** Gumbel roots capture *improved-policy logits*
  (`logit + σ(q̂)`, no noise — noise is exploration, not belief;
  unvisited arms completed by their prior), marked
  `values_are_logits` and softmaxed at temperature 1 downstream — the
  completed-Q construction, replacing round 35's
  `softmax(means / 0.1)` hand-picked temperature for these rows. UCB1
  captures are unchanged.

  **Two latent defects fixed on the way, both on this exact path.**
  (1) A truncated candidate set (`chosen ≥ POLICY_MAX_CANDIDATES`)
  trained on the *tail window* of successors while reading `values[0..n]`
  from the head of the array — every state paired with another arm's
  value. Latent (main-phase menus cap at 8), live the moment
  `--search-combat`'s uncapped block menus record; now sliced by the
  same window, with a regression test. (2) The decision holdout hashed
  the *game index* while row holdout hashes `traj` — two independent
  keys, so a game's positions could train while its decisions
  validated, and the comment claimed otherwise. Decisions now split by
  the same `(seed << 1) | seat` hash as the rows. Consequence:
  `val_policy` membership differs from rounds 33–35's draws — same
  distribution, different sample; cross-round comparisons carry that
  caveat. Also new: `--best-metric policy` publishes `best.safetensors`
  on held-out policy agreement instead of AUC, because round 34
  measured the two moving in *opposite directions* on one seed — a
  distillation run selected on AUC can publish its worst ranking
  checkpoint while reporting success.

  **Pre-registered design.** Part 1: gen-0 training, r35 recipe
  verbatim (20 k games, MCTS-64 actors over the headless
  `nets_r33_control_s43` pilot, `--policy-every 4`) plus
  `--policy-head --best-metric policy`, seeds 43/97 — the existing r35
  cells are the controls (same pilot, seeds, budget). Readouts:
  `val_policy` vs r35 distil's 0.726/0.814 (the head should match or
  beat the shared head), `val_auc` vs r35 control's 0.7815/0.7826 (and
  should NOT pay the shared-head tax; AUC seed spread is 0.023, read
  the pair). Part 2: `mcts-net-gumbel` vs `mcts-net-deep`, identical
  weights both sides (the r36 design), 100 games/archetype × sealed ×
  two ladder seeds, on both head nets plus the headless champion (the
  allocator-alone control). At ±2 per cell, one cell of a pair is not
  a reading. Not queued yet, pending Part 1+2 signal: gen-1 (Gumbel
  actors piloted by a head net, `--mcts-gumbel` — the full
  expert-iteration loop), and a champion-scale (250 k) run.

  **Part 1 result: the head carries the ranking and pays no value
  tax.** 20 k games per seed, ~47 k steps, `best` published on
  `val_policy`:

  | | val_policy | chance | pilot_policy | val_auc |
  |---|---|---|---|---|
  | r37 head s43 | **0.816** | 0.452 | 0.265 | 0.7964 |
  | r37 head s97 | **0.791** | 0.454 | 0.274 | 0.7680 |
  | r35 shared s43 | 0.726 | 0.390 | 0.376 | 0.7945 |
  | r35 shared s97 | 0.814 | 0.453 | 0.397 | 0.7685 |
  | r35 control s43/s97 | 0.296 / 0.330 | — | — | 0.7815 / 0.7826 |

  The separate head reaches 0.80 mean agreement with the search
  (shared head: 0.77 — mixed per seed, higher on the mean, with the
  holdout-redraw caveat above), while the value metrics sit exactly on
  the control (AUC mean 0.7822 vs 0.7821; the r33 shared-head tax does
  not appear). `pilot_policy` lands below chance in both cells — the
  third independent replication of round 34's "the evaluator
  contradicts its own search". The capability claim is clean: the
  ranking objective can be carried by its own 257 parameters without
  touching the win head.

  **Part 2, first gate: a large NEGATIVE with a found mechanism — the
  σ transform was unnormalized.** All six cells (both head nets, the
  champion fallback, two ladder seeds each) lost hard: 29.5/28.2/29.9/
  29.4 % for the head nets, 36.2/34.5 % for the champion control. The
  magnitude said implementation, not method, and the reference
  (mctx `qtransform_completed_by_mix_value`) confirmed it: completed
  Q-values are **min-max normalized across the decision's arms to
  [0, 1]** before the `(c_visit + max_visits)·c_scale` scaling. Ours
  fed raw win probabilities in, and two candidate lines in one position
  differ by a few *points* of win probability — σ gaps of ~0.3 logits
  against Gumbel noise of stddev ~1.28. The final argmax was a noise
  lottery over noise-selected survivors; 15–20 points down is what a
  lottery scores. Fixed (`completed_sigma`: per-decision min-max over
  visited arms, pinned by
  `completed_sigma_normalizes_rewards_across_arms`), applied to both
  the selection score and the captured improved-policy logits. The
  first-gate numbers stand in this entry as what an unnormalized σ
  costs, not as a verdict on Sequential Halving.

  **Part 2, re-gate with the normalized σ: the fix recovers the whole
  20 points, and lands on a null.**

  | cell (gumbel vs deep) | ladder 43 | ladder 97 |
  |---|---|---|
  | head s43 | 49.1 % [47.1, 51.1] | 48.5 % [46.6, 50.4] |
  | head s97 | 49.4 % [47.4, 51.4] | 48.0 % [46.1, 49.9] |
  | champion (heuristic priors) | 49.9 % [47.9, 51.9] | 48.2 % [46.2, 50.1] |

  Head nets pool to **48.75 %**, the champion control to 49.05 — all
  six point estimates a hair under 50, no interval clearly below.
  Sequential Halving with a learned prior neither beats nor
  measurably loses to UCB1 at this budget: round 29's conclusion
  ("only iterations pay; selection policy is at a local optimum the
  defaults already occupy") survives its strongest challenger yet,
  now with a prior that demonstrably matches the search's own ranking
  at 0.80. **Not adopted**; `mcts-net-gumbel` stays in-tree as a
  measured control alongside the r29 knobs, and `mcts-net-deep`
  remains the reference searcher. Caveats that keep a door open, not
  a claim: the head nets are 20 k-game gen-0 artifacts (a
  champion-scale head is untrained), and a prior distilled *from*
  UCB1's conclusions may be exactly the prior that cannot beat UCB1 —
  a gen-1 head trained on completed-Q targets from Gumbel actors is
  the version of this bet that is actually different, and is cheap to
  arm now that the infra exists.

  **Round verdict.** The head is the result: ranking capability at
  0.80 with the value head untouched — the precondition round 36
  demanded — plus the leak fix, the target-alignment fix, the
  best-metric selection, and the mixed-fleet infra. The consumer that
  converts it into play strength is still unfound: not as a search
  prior at 64 iterations (this round), previously not as a shared
  head (r33/35/36). The standing live next step is unchanged from
  round 36 but now properly equipped: champion-scale distillation
  (250 k games, mixed fleet, separate head, best-metric policy),
  gated as a pilot and as MCTS's leaf evaluator.

  **Infra found and fixed while the run was in flight: the MCTS-actor
  recipe has been under-provisioned since round 34.** The r34/r35/r37
  cells all ran the default `--actors` = cores − 2 = 22 against a
  256-state eval batch, and 22 blocked threads cannot fill a 256 batch
  — the `--gpu-eval` doc has warned exactly this since round 12's
  batched-eval work ("pair with a large `--actors` (hundreds)"), and
  round 27 measured MCTS generation at 256 threads, not 22. Symptom:
  ~9 busy cores of 24, a mostly-idle GPU flushing near-empty batches on
  the 1 ms timer, the learner asleep 85–86 % on the reuse throttle.
  The in-flight r37 cells were left as-is (pre-registered, and
  comparable to r35 only on the same recipe); the lever is recorded
  here so the next MCTS-actor run picks it up.

  Landed for that next run: **the mixed actor fleet**
  (`--mcts-fleet N` — the KataGo playout-cap-randomization idea
  translated to this loop). Only the first N actor threads pilot with
  MCTS and feed the decision stream; the rest play the plain net pilot
  for value-row volume and are excluded from capture by a thread-scoped
  override (`decision_capture::set_thread_enabled` — a value actor's
  picks are one-hot imitation targets that would bury the distillation
  targets at ~50–100× the searched games rate). `stats.jsonl` now
  carries `games_mcts` next to `games` so the mix is legible per
  checkpoint. Default 0 = all-MCTS, the historical behaviour;
  wiring smoke-tested (2-of-6 fleet: 11/120 games searched, decisions
  from the search fleet only). No throughput claim yet — that number
  comes release-built, at scale, with the actor count raised, per the
  house perf rule.

- 🔴 **Round 36 — MCTS is not removable, and distillation makes search
  *more* valuable rather than less.** Round 35 established the
  precondition: the net can absorb what the search concluded, going from
  below-chance agreement to 0.73/0.81. The proposal this round gates is
  the obvious consequence — if the net ranks candidates the way MCTS
  does, then scoring each successor directly should reach the search's
  strength at one eval per candidate, and 64-iteration rollouts (~50-100x
  the per-game cost) could be retired at inference.

  **Design.** The ladder has one net slot, so `mcts-net-deep` vs `net`
  puts identical weights on both sides and each cell asks exactly one
  question: does 64 iterations of search still add anything on top of
  *this* net? Five conditions — champion as the era anchor, the two r35
  controls (without which a narrowed gap could be "v6-era nets differ
  from the champion" rather than "distillation worked"), the two r35
  distil nets. Part 2 re-gates every net against atk-sim and gang,
  because both Part 1 sides share the weights and MCTS consumes them as
  the UCB1 leaf reward: a distilled net whose win head had *degraded*
  would handicap the search side too and narrow the gap for the wrong
  reason. 100 games/archetype for Part 1 (~480 s/cell), 1000 for Part 2
  (~1 s/120 games, so an order of magnitude more resolution for a
  fraction of the wall clock). 30 cells, ~252 k games.

  | net | mcts-deep vs net, s43 | s97 | pooled |
  |---|---|---|---|
  | champion | 50.3 | 54.0 | 52.15 |
  | ctrl43 | 53.6 | 54.1 | 53.85 |
  | ctrl97 | 53.6 | 54.4 | 54.0 |
  | dist43 | 55.6 | 55.2 | **55.4** |
  | dist97 | 53.4 | 55.8 | **54.6** |

  **All ten cells have MCTS ahead**, five net conditions × two ladder
  seeds, lowest reading 50.3. There is no configuration here in which
  scoring successors directly catches the search.

  **The bet inverted.** Distillation was supposed to make search
  redundant; the distilled nets are instead where search helps *most* —
  distil arm 55.0 pooled against the control arm's 53.9. The mechanism is
  the one Part 2 was built to expose: both sides share the weights, so a
  better net is also a better leaf evaluator, and MCTS converts the
  improvement more efficiently than direct ranking does. **You cannot
  distil your way out of a search that eats the same net you improved.**
  Any future attempt at this has to break that coupling — a separate
  policy head the search does not consume, or a search whose leaf value
  is frozen — or it will keep measuring this.

  Part 2, pooled over both ladder seeds (12 k games/cell):

  | net | vs atk-sim | vs gang |
  |---|---|---|
  | champion (250 k games) | **52.7** | **51.2** |
  | control (pooled) | 50.8 | 49.6 |
  | distil (pooled) | 51.2 | 50.0 |

  Distillation *is* a real pilot gain — **+0.40 atk-sim / +0.48 gang**,
  same sign in all four net×opponent comparisons — and it does not come
  from wrecking the value head, which is what Part 2 rules out. Pilot and
  search improved together. The split by training seed is the usual one
  (+0.75 on s43, +0.13 on s97), and the difference between two
  independent pooled cells carries ~±0.55, so only the s43 arm separates
  cleanly: sign-consistent, magnitude-unreliable.

  **What this does NOT close.** Every r35 net is a 20 k-game run against
  the champion's 250 k, and all four are weaker pilots than the champion
  — so this bounds the current artifact, not the direction. A
  champion-scale distillation is untested and is the live next step. Do
  not re-propose "distil the search away" at 20 k games; do not treat
  this as having refuted it at 250 k.

  **Cross-era warning, now concrete.** The round-26/27/29/31 MCTS numbers
  predate two commits that both change what the search does: `a24b2b7c0`
  (fixed hasher — container iteration order, hence tie-breaks and RNG
  consumption, shift globally) and `e1788ef65` (`determinize_hidden`
  sorts each zone by card id before shuffling, so the redeal is a
  function of the information set; the commit says outright it is not
  behaviour-preserving). The champion anchor pools to 52.15 today against
  round 26's 52.95, so those *verdicts* stand — but the individual
  figures are not comparable and should not be quoted against new runs.

  **Methodological note, learned the hard way this round.** The champion
  anchor's seed-43 cell read 50.3 against round 26's 53.4, and that was
  read live as evidence that the determinization fix had erased the
  search's advantage — a tidy mechanism, since the pre-fix redeal really
  did permute a hidden arrangement. Seed 97 then came in at 54.0 and the
  pair pooled to 52.15, reproducing. At ±2 per cell, **one cell of a pair
  is not a reading**, and a plausible mechanism makes it easier, not
  harder, to over-read one. Same lesson as round 32's AUC seed spread,
  now restated on the ladder.

- 🟢 **Round 35 — policy distillation from MCTS: the capability
  replicates on both seeds, the value metrics are a wash.** Round 33
  taught the win head to rank successors toward the *heuristic's* pick,
  which caps at the heuristic. This trains it toward what a 64-iteration
  search concluded, using each arm's mean reward (`2dd3ad240` records the
  search's root decision with per-arm means; one-hot would be imitation,
  and per-candidate values carry *how much* better each option looked).
  Arms: `--policy-every 4` vs a `--policy-every 0` control that records
  and scores the same decisions while training on none. 20 k games/cell
  — MCTS actors run ~2 games/s against heuristic actors' ~235, so the
  budget is decisions (~400 k/cell), not games.

  | | val_policy | chance | pilot_policy | val_auc |
  |---|---|---|---|---|
  | control s43 | 0.296 | 0.438 | 0.359 | 0.7815 |
  | distil s43 | **0.726** | 0.390 | 0.376 | 0.7945 |
  | control s97 | 0.330 | 0.433 | 0.413 | 0.7826 |
  | distil s97 | **0.814** | 0.453 | 0.397 | 0.7685 |

  **The policy result is unambiguous**: below chance → 0.73/0.81, both
  seeds, against a metric whose seed spread is 0.005 (round 33). The net
  can represent and learn the search's ranking.

  **`pilot_policy` closes round 34's stated limitation.** Round 34
  measured a *training* net against decisions made by a different
  (pilot) net, so some of the below-chance agreement could have been
  two-nets-disagreeing rather than search-vs-its-own-evaluator. This
  column scores the loaded pilot — the very net MCTS rolled out on — on
  the same holdout, and it lands **below chance in all four arms**
  (0.359/0.376/0.413/0.397 against 0.390–0.453). The round-34 finding is
  real and confounder-free: a net's immediate ranking contradicts the
  conclusions of a search built on that same net.

  **The value metrics are a null.** AUC +0.013 on s43, −0.015 on s97,
  mean −0.001 — inside the 0.023 seed spread round 32 measured. Reading
  either seed alone gives a confident and opposite answer, again.
  (`best` and `final` AUC agree to ~0.001 in all four runs; cosine decay
  left nothing to overfit into, so the standing "score `best` not
  `latest`" rule does not bite here.)

  **Not adopted on this evidence.** A ranking gain with flat value
  metrics is a screen result; round 36 gates it on play strength, and
  finds it a small real pilot gain that nonetheless makes search *more*
  valuable rather than less.

- 🟢 **Round 34 step 1 — the value net ranks immediate successors
  *worse than random* against what MCTS concludes, and improving it does
  not help.** Headroom probe for distillation: MCTS actors (64 iters over
  a v6 net, batched GPU eval), `--record-decisions --policy-every 0` so
  the net is measured and never trained on the decisions.

  | seed | final val_policy | chance | lift | AUC over the run |
  |---|---|---|---|---|
  | 43 | 0.3546 | 0.4475 | **0.79×** | 0.691 → 0.767 |
  | 97 | 0.2953 | 0.4435 | **0.67×** | 0.643 → 0.735 |

  All ten checkpoints across both seeds land below chance (lift
  0.66–0.91×, never 1.0). At n ≈ 4 050 holdout decisions the binomial SE
  is ~0.008, putting the finals 12 and 19 SE under — far outside noise
  even discounting within-game correlation.

  **The trend is the finding.** AUC rises steadily in both runs while
  agreement stays flat (seed 43) or falls (seed 97, 0.404 → 0.295).
  Getting better at predicting *who wins from here* does not make the
  net better at ranking *which move to make*, and the two may trade off.
  That is the sharpest evidence yet for the round-33 observation that
  AUC and ranking are only loosely coupled — here they move in opposite
  directions.

  Below chance is a stronger claim than weak agreement, and needs a
  mechanism. The plausible one: the net scores a successor by how good
  the board looks *immediately*, so it prefers casting now, while the
  rollouts often conclude that holding mana, declining a trade, or not
  overextending is better. That is a systematic disagreement in a
  consistent direction, which is what below-chance requires — random
  disagreement would sit at chance.

  **Two limitations, both real.** The measured net is undertrained
  (4 000 games, AUC 0.74–0.77 against the champion's ~0.81), though the
  cross-checkpoint trend partly answers that by showing improvement does
  not close the gap. And MCTS's rollouts run on the *pilot* net, not the
  training net, so some of the disagreement is two-different-nets rather
  than search-vs-evaluator. Scoring the loaded pilot itself against the
  same holdout would separate those and is the obvious next addition.

  **What it means for step 2.** Distillation has a large, consistent,
  and apparently learnable target: the search reaches conclusions its own
  evaluator's immediate ranking actively contradicts. Round 33 showed
  the net can absorb a policy target (0.35 → 0.81 on the heuristic's
  picks); this shows the MCTS target is very different from what the
  value objective produces on its own.

  Also found: `--gpu-eval --use-best nets/champion.safetensors` has been
  broken since encoder v6. The champion is a v5 checkpoint (trunk1
  [512, 1060] vs [512, 1067]); `PlayNet::load` zero-pads legacy files but
  candle's `VarMap::load` is a strict shape set and panics. The probe
  uses a v6-native net.

- 🟢 **Round 33 — the first policy training run, and the first number
  that measures *choosing*.** The net has only ever been taught to
  predict who wins from a position. `--record-decisions` captures the
  pilot's candidate set (as successor states) and the pick;
  `train_policy_step` teaches the existing win head to rank them —
  a softmax over its logits for the successors, target = the index
  played. No new head, no new parameters. `val_policy` is held-out top-1
  agreement, and the control arm (`--policy-every 0`) records and scores
  the same decisions while training on none of them.

  | | val_policy | val_auc | val_win |
  |---|---|---|---|
  | control s43 | 0.4292 | 0.7735 | 0.19102 |
  | policy s43 | **0.8144** | 0.7630 | 0.20386 |
  | control s97 | 0.4338 | 0.7919 | 0.18440 |
  | policy s97 | **0.8042** | 0.7718 | 0.19352 |

  **The value net agrees with the pilot's pick about 43 % of the time,
  against a measured chance rate of 0.354.** Both control seeds land
  within 0.005 of each other, so this is a stable property and not a
  seed accident. The candidate-count distribution was *measured* rather
  than assumed (87.5 % three-candidate, 12.5 % two — see
  `print_candidate_count_distribution`), because the first write-up of
  this entry said "chance is between 0.33 and 0.5" while the observed
  value sat inside that band, which settles nothing.

  Two things this does **not** show, both of which the first version of
  this entry got wrong:

  * The pilot here is `EvalWeights::default()` — the plain heuristic, no
    net in the loop (the run passes no `--use-best`). So this is
    net-vs-*heuristic* disagreement, and the heuristic's pick is not
    ground truth. It is a different, roughly equal-strength, also
    imperfect player. Calling the value net "close to coin-flipping"
    smuggled in the assumption that the heuristic is right.
  * +0.075 over chance is weak agreement, not none. The value net does
    carry some signal about which candidate the heuristic prefers.

  What it *does* show: two roughly equal players disagree on most
  decisions, and nothing in the program had ever measured that. AUC
  scores predictions over positions nobody chose; a search consumes
  rankings over candidates it must choose between. The two are only
  loosely coupled, which is why AUC 0.77–0.79 coexists with 0.43
  agreement.

  Policy training takes agreement to **0.80–0.81**, replicated on both
  seeds. The cost is a small, consistent regression in the value
  metrics: −0.010/−0.020 AUC and +0.013/+0.009 val_win. That trade is
  what two objectives on one head look like, and whether it is worth it
  is a ladder question this screen cannot answer.

  The clean claim is a **capability** one: the encoder and head can
  represent a policy over candidate successors, learning it from 0.35 to
  0.81. That is what had to be true before any of this direction was
  worth pursuing, and it was not previously known. But the target here
  is the heuristic's pick, so imitation caps out *at* the heuristic —
  which is exactly why the next step is distilling search rather than
  scaling this up.

  Methodological note worth as much as the result: `val_policy` has a
  seed spread of **0.005** against AUC's 0.023 (round 32). It is a far
  better-powered metric, because it measures the thing the search
  consumes rather than an average over positions no one chose.

  **Not adopted.** A ranking gain that costs value calibration has to be
  gated on play strength before it goes anywhere near the champion. Next:
  ladder the policy-trained net as a pilot, and — the reason this was
  built — distil MCTS visit counts instead of the heuristic's pick,
  which is the policy-improvement operator the stacking failure has been
  missing since round 14.

- 🟡 **Round 32 — cross-entropy on the win head: a two-seed NULL, and
  the screen says why it could not have been anything else.** The
  argument is mechanistic and still stands: MSE through a sigmoid has
  gradient proportional to `(p − y)·p·(1 − p)`, so rows where the net is
  confidently wrong produce the weakest updates, while BCE's gradient at
  the logit is `(p − y)`. A unit test confirms the gradient behaviour
  directly — from identical weights driven to a confidently wrong
  opinion, one BCE step moves the prediction further back than one MSE
  step. What does not follow is an end-to-end win.

  Screen scale (60 k games/cell, λ = 0.7, lr 1e-4, window 500 k, two
  seeds, det0 actors on both arms):

  | | val_auc | val_win | val_logloss |
  |---|---|---|---|
  | mse s43 | 0.8032 | 0.17969 | 0.53082 |
  | bce s43 | **0.8090** | **0.17807** | **0.52615** |
  | mse s97 | **0.7799** | **0.18931** | **0.55210** |
  | bce s97 | 0.7774 | 0.19060 | 0.55433 |

  Seed 43 favours BCE on every metric including MSE itself (the fair
  comparison, and the one MSE is directly optimising); seed 97 reverses
  it. Mean AUC 0.7916 vs 0.7932, **+0.0017**.

  **The measurement lesson is the durable part.** Within the MSE arm
  alone, seed variance is 0.8032 → 0.7799 = **0.023 AUC**, an order of
  magnitude larger than the effect. No amount of *longer* running fixes
  that; a two-cell-per-arm design cannot resolve an effect one tenth the
  size of its own noise, and reading either seed alone would have
  produced a confident and opposite conclusion. Resolving effects of
  this size needs more seeds or a variance-reduction design — the same
  lesson the antithetic paired ladder taught for win rates, now
  restated for AUC.

  `--bce` stays in-tree, default off. It costs nothing, the mechanism is
  real, and it is the natural arm to re-run if a future change makes the
  win head's gradient the binding constraint. Not adopted: no measured
  win means no adoption, and one seed is not a measurement.

- 🟢 **Instrumentation — the overfit readout has been unreadable since
  round 27, and fixing it explains why λ < 1 works.** `loss_win` is the
  training EMA against whatever the learner *fit* — the λ-return at
  λ < 1 — while `val_win` scores the holdout against the raw 0/1
  result. The checkpoint comment asserted they were "directly
  comparable (same MSE, different rows)", which holds only at λ = 1;
  every champion-class run since round 27 is λ = 0.7. So the champion's
  headline "`loss_win` 0.00926 vs `val_win` 0.18746" compared two
  different quantities and the +0.178 was not an overfit measurement.

  Each side is now scored against both labels: `train_raw` / `val_win`
  against the result, `train_tgt` / `val_tgt` against the λ-return
  (`lambda_targets`, factored out of the window's two relabel paths so
  the holdout — which is never relabelled — computes exactly what the
  window would have written). Two short verification runs, 3 000 games,
  50 k window, ~1 000 steps:

  | | train_raw | train_tgt | val_win | val_auc | honest gap |
  |---|---|---|---|---|---|
  | λ = 1.0 | 0.03557 | 0.03557 | 0.27251 | 0.7044 | **+0.237** |
  | λ = 0.7 | 0.14300 | 0.01331 | 0.20113 | 0.7569 | **+0.058** |

  λ = 1 collapses the two labels to the decimal, as the identity
  requires — the control that says the wiring is right. The λ = 0.7 row
  is the new information, and it says two things. First, **λ < 1 is
  acting as a regulariser**: at λ = 1 the net drives training error to
  0.036 against a holdout of 0.273 (it is memorising a 50 k window),
  and λ = 0.7 cuts that gap four-fold while scoring the better AUC. The
  program adopted λ = 0.7 in round 27 on gate results without a
  mechanism; this is the mechanism. Second, **the objective is nearly
  saturated by self-consistency**: `train_tgt` 0.013 against
  `train_raw` 0.143 means ~91 % of what the optimiser is minimising is
  "be smooth along a trajectory", not "predict who wins". That is how
  TD is supposed to work, but it is also exactly the quantity that a
  λ *schedule* toward small λ would push further, so round 34 gates
  λ-schedule × relabel-mode as a 2×2 rather than moving one knob.

  Caveat on magnitudes: 3 000-game runs at the default lr 1e-3 with a
  50 k window overfit far harder than a 250 k-game champion run at
  1e-4 / 500 k. The *identity* at λ = 1 and the *sign* of the λ = 0.7
  separation are what these runs establish; the champion-scale numbers
  come from the next full run.

- 🔴 **Round 31 — MCTS combat coverage: a large, clean NEGATIVE. Combat
  declarations need the sims' precision, not the rollouts' breadth.**
  The hypothesis was coverage: the round-26/27 wins searched only
  main-phase plays, so searching attack/block declarations too (same
  candidate menus the sim searches score, rollouts + net rewards in
  place of the one-turn sims) should extend the win to the decisions
  that decide limited games. Gate: `mcts-net-combat` vs `mcts-net-deep`,
  identical 64-iteration budgets, 100-game paired cells:

  | seed 43 | seed 97 |
  |---|---|
  | **39.4 %** [37.3, 41.5] | **38.4 %** [36.4, 40.4] |

  Eleven points down, both seeds, CIs nowhere near 50 — and +26 %
  wall clock on top. Why this direction reverses the main-phase result:
  a combat declaration's candidates differ by *fine margins* (hold one
  blocker back, chump or don't) that the sims resolve with exact
  engine damage math over one structured turn cycle, while ~9 noisy
  rollouts per arm resolve them with the variance of three turns of
  semi-random continuations. Main-phase candidates differ grossly
  (cast the bomb or don't), which is what rollout averaging can rank.
  Search breadth beats sim precision exactly where outcomes diverge
  coarsely, and loses where the decision hinges on arithmetic the
  engine already does perfectly. `search_combat` stays in-tree,
  default off; the adopted pilots are untouched (they never enabled
  it). Raising iterations was considered and not queued: at −11 the
  gap is not a budget artifact.

- 🟡 **Round 30 — champion re-baseline: the healthy regime reproduces
  champion-class nets; det0 data is a null at full training; the
  incumbent survives.** The 2×2 grid (actor data × training seed, all
  GPU learner, champion regime, encoder v6, gates = 2×100-game paired
  cells per opponent):

  | arm | steps | calib AUC | gang pooled | atk-sim pooled |
  |---|---|---|---|---|
  | det1 s43 (r28f) | 64k | 0.8115 | 52.1 % | 53.7 % |
  | det1 s97 (r28f) | 68k | 0.8182 | 51.2 % | 52.7 % |
  | det0 s43 | 56k | 0.8088 | 50.9 % | 52.7 % |
  | det0 s97 | 64k | 0.8062 | 51.2 % | 53.4 % |
  | champion (incumbent) | 70k | — | 51.7 % | 54.5 % |

  1. **The det0 +1pt reading from the crippled CPU runs does not
     replicate**: at full training the arms are indistinguishable (det1
     pooled 51.6/53.2 vs det0 51.1/53.0). The honest-sims default
     (`determinize: 1`) costs the training data nothing measurable —
     keep it; `--actor-det` stays as a control knob only.
  2. **No cell clears the incumbent** (pre-registered adoption rule):
     the four fresh draws cluster gang 50.9–52.1 / atk-sim 52.7–53.7,
     and the champion's 54.5 atk-sim sits at the top edge. Consistent
     with the champion being a good draw from this distribution, not a
     different distribution. Champion stays; four spare
     champion-class nets now exist (nets_r28f_full_*, nets_r30_det0_*).
  3. The repaired pipeline is trustworthy again: 4/4 healthy runs land
     in the champion band on the first attempt.

- 🔴 **Round 29 — MCTS internals: every knob is null-to-negative; only
  iterations pay.** Three tunings of the round-26/27 search, each gated
  head-to-head against the `mcts-net-deep` control (64 iters / h3 /
  c 1.0 / no priors / fixed budget), 100-game paired cells × seeds
  43/97, champion net both sides:

  | knob | profile(s) | vs deep, pooled | verdict |
  |---|---|---|---|
  | exploration c 0.5 / 1.4 / 2.0 | mcts-net-c05/c14/c20 | 49.8 / 49.3 / 49.4 % | flat — c 1.0 already fine |
  | P-UCT root priors (w 1.5, temp 4 units) | mcts-net-prior | **46.3 %** | NEGATIVE — candidate scores misallocate the budget |
  | adaptive budget (early-stop + 4× close-call extension) | mcts-net-adapt | 51.2 % | see below |

  The adaptive arm's 51.2 % is not a win: its measured wall clock ran
  ~1.6× the control cell — the close-call extension dominates the
  early-stop savings, so it *spends* ~2×, and the pre-registered claim
  was cost, not strength. The budget-matched cell settles it:
  **adapt vs fixed-128 = 49.7 % pooled** (50.1/49.2) — adaptive
  allocation is exactly worth its average spend, no more. The prior
  result is the interesting negative: at 64 iterations, seeding visits
  from `score_candidate` is worse than uniform UCB1 exploration —
  plausibly the softmax (temp 4 units) is too sharp and starves arms
  the heuristic underrates, which is the very case search exists for.
  A gentler dose is untested; the first dose is clearly harmful.

  Conclusion, sharpening round 27: **the only MCTS lever that pays is
  raw iterations** (24→256 = 49.4→55.0 % vs champion). Selection
  policy, exploration constant, and allocation shape are all at a
  local optimum the defaults already occupy. Client stays MctsBot-64;
  mcts-net-256 stays the strength reference. Infra stays (priors,
  adaptive budget, and the c profiles are config, defaults off/1.0,
  pure-function unit tests in `mcts.rs`).

- 🔴 **Round 28 — encoder v6, and the training regression it uncovered
  (arc open: champion-era reproduction run pending).** The plan was a
  representation round: encoder v6 (combat-structure block — counterpart
  P/T sums across block edges, fine combat-phase one-hots, unblocked
  incoming power, attack-target kind; keyword-class block — haste /
  hard-to-target / indestructible / hard-to-block / defender + exile
  counts; `OBJ_FEATS` 45, `GLOBAL_FEATS` 43, `SHARD_VERSION` 6, legacy
  checkpoints zero-padded at load so the champion and golden traces are
  untouched), trained under the champion regime, control = round 20.
  What it found instead is that **the champion regime no longer
  reproduces at all**, and five controlled probes later the cause is
  still not a training flag:

  | run (seed 43 unless noted) | encoder | actors | learner steps | calib AUC | gang | atk-sim |
  |---|---|---|---|---|---|---|
  | champion re-gate (today's binary) | — | — | — | — | **51.7 %** | **54.5 %** |
  | r28 (2 seeds) | v6 | det1 | ~8k | 0.775/0.778 | 48.3 | 50.1 |
  | r28b ablated control (2 seeds) | v5-parity | det1 | ~8k | 0.775/0.775 | 48.7 | 50.5 |
  | r28c-P (stale 48) | v6 | det1 | ~8k | 0.785 | 48.5 | 50.2 |
  | r28c-D (`--actor-det 0`) | v6 | det0 | ~8k | 0.790 | 49.4 | 51.0 |
  | r28d (`--tail-reuse 12`) | v6 | det1 | ~26k | 0.787 | 47.3 | 48.7 |
  | r28e (det0 + tail 12) | v6 | det0 | ~26k | 0.776 | 48.4 | 49.7 |

  What is established:

  1. **Encoder v6 is innocent and unmeasurable at this run shape**:
     r28 ≡ r28b to the third decimal. The representation verdict is
     *open, not null* — it cannot be read until training is healthy.
  2. **The measurement pipeline is stable**: the committed champion
     re-gated on today's binary reproduces its original 51.8/54.4 to
     within 0.1 pts. The regression lives in the trained nets.
  3. **The binding stop was a silent one**: the post-generation tail
     allowance (reuse/2 window-passes, hardcoded) became the stop on
     this ~40 %-faster container — every run died at ~8.4k steps while
     checkpoints were still setting AUC bests, and `--stop-after-stale`
     never fired. `--tail-reuse` makes it a knob. But repairing it
     (26k steps) did *not* repair the gates — tail-heavy training on a
     static window scored better holdout AUC and worse play, the
     calibration-vs-strength divergence again.
  4. **Determinize-era training data costs ~1 pt** (28c-D) — the r25
     honest-sims adoption silently swept the training generator along
     with the play default; `--actor-det 0` restores it. Real, small,
     not the missing 3-4.
  5. **No tested combination recovers the champion.** det0 × full
     length (28e) — the closest flag-level reconstruction of the r20
     recipe — lands at 48.4/49.7.
  6. Cross-era calib AUC comparisons are apples-to-oranges: `--calibrate`
     plays the *current default* pilots, which changed at r25. Gates
     are the only stable cross-era instrument.

  **ROOT CAUSE (found by the champion-era reproduction): the trainer
  was built without `--features cuda` all week.** The worktree repro
  (r19-21 commit `4f81029a7`, champion recipe and seed verbatim) also
  landed in the 50/51 band — same code, same flags, still short — and
  its stats.jsonl against the original run's told the whole story:

  | | learner steps/s | total steps | gen games/s | best AUC |
  |---|---|---|---|---|
  | original r20-s97 | 42–53 | **70,124** | 162.8 | 0.8090 @ step 54k |
  | worktree repro | 3.1–4.8 | 10,135 | 70.6 | 0.7803 @ step 10k |

  The original learner ran on the RTX 4090 (round 12: "GPU learner
  confirmed"); every binary built this session used
  `cargo build --release --bin selfplay_train` — no cuda feature — so
  the learner fell to CPU at a tenth the speed and every run since
  round 28 was silently undertrained (~8–10k steps where the champion
  got 70k, cosine never annealed, best checkpoints at step ~2k). The
  GPU was present and idle the whole time (`nvidia-smi` clean); the
  container was never the problem, and neither was the encoder. The
  build doc in crabomination_ml/Cargo.toml says the right command; the
  lesson is the learner-device line in the first log paragraph is a
  *gate*, not a detail — a CPU learner on this workload is a
  misconfiguration, and nothing downstream of it is measurable.

  Findings that survive the dissolution: `--tail-reuse` (the fixed
  tail allowance really was the binding stop for a slow learner, and
  remains a correctness knob), `CRAB_ABLATE` (gating ablated-trained
  nets under the full encoder feeds live features into never-trained
  random columns — always match), the champion re-gate (measurement
  pipeline stable across ~100 commits), and the traps: cross-era calib
  AUC is apples-to-oranges, and holdout-AUC improvement ≠ gate
  strength (r28d: tail-heavy CPU training raised AUC and lowered
  gates). The det0 +1pt reading (28c-D) was measured on crippled runs
  and needs re-verification before anyone acts on it.

  **Round 28f — the real experiment, and the final verdict: encoder v6
  is a NULL.** GPU learner restored (62–68k steps per run, cosine
  annealed, AUC back in the champion band; the run script hard-aborts
  unless the learner logs cuda), v6 vs v5-parity, two training seeds,
  800 paired games per arm per gate:

  | arm | calib AUC (s43/s97) | gang pooled | atk-sim pooled |
  |---|---|---|---|
  | v6 full | 0.8115 / 0.8182 | 51.6 % | 53.2 % |
  | v5-parity (`--ablate combat,kw`) | 0.8184 / 0.8166 | 51.6 % | 53.2 % |
  | champion re-gate (control) | — | 51.7 % | 54.5 % |

  Identical to the decimal on both gates. The combat-structure and
  keyword/exile blocks move nothing a healthy champion-regime net can
  use — the round-12 conclusion (representation additions beyond
  library + castability are null) survives a far stronger test, and
  the round-17b theme holds: regime and data dominate, inputs don't.
  The v6 format stays in-tree on the round-12 precedent — information
  superset, per-block ablation controls, zero measured encode cost,
  legacy checkpoints load padded — but no quality claim attaches to
  it. The champion stays champion (r28f arms match it on gang and
  trail ~1 pt on atk-sim — inside noise, no adoption case). Round 28
  closed.

  **Caveat added in round 40, and it is a large one: the combat half of
  this experiment could not have measured anything.** The feature
  census showed globals 36..=40 and object feats 37..=39 are non-zero
  in *zero* recorded training rows — the recorder never snapshots a
  combat step — so the "v6 full" arm above trained the combat block
  against an all-zero column and the v5-parity arm ablated a block that
  was already blank. The two arms were the same experiment. The
  keyword/exile half is unaffected (feats 40..=44 run 0–8 %, globals
  41..=42 18.6 %) and its null stands. See round 40.

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
- 🟡 **Round 27 loop verdict (2026-08-10) — the flywheel does not turn
  at this scale: MCTS-quality labels are a null.** 50 k MCTS-64-piloted
  games vs a 50 k net-piloted same-budget control, identical regime,
  seed 43: AUC 0.8180 vs 0.8130 (sub-threshold), gates 50.1/51.4 %
  pooled vs 49.75/52.1 % — indistinguishable (both below the 250 k-game
  champion, as the budget predicts). No signal → no seed-97 replication
  per the pre-registered rule. Third strike for label-source levers
  (rounds 14, 18, 27), and this one had a generator that was *provably*
  ~3 points stronger. Coherent picture: **search amplifies at
  inference, not through training** — the net's ceiling is set by
  representation/data diversity, not label quality. The strongest
  system remains champion net + MCTS at inference (256 iters for
  strength, 64 for latency). A 250 k-game MCTS arm (~130 h) is not
  worth queueing on a zero-signal 50 k result.
- 🟢 **Round 27 (2026-08-10) — the scaling curve climbs; MCTS-net ships
  to the client; the training loop is armed.** Three tracks on the
  round-26 result:

  *Scaling* (vs the `net` champion, 1 200 games/cell, two ladder
  seeds): 24 iters 49.4 % → 64 **53.0 %** → 128 **54.35 %** → 256
  **55.0 %** pooled — each doubling buys ~+1.4 then ~+0.7, still
  climbing at 256. Horizon 4 ≈ horizon 3 (53.0 %): three turns of
  rollout captures what these positions need; iterations are the axis.
  `mcts-net-256` is the strongest known pilot (~58–59 % vs gang
  implied).

  *Client adoption*: the server boots `nets/champion.safetensors` into
  SLOT_BEST when present (CRAB_NET overrides; bad file = boot error,
  not silent degradation) and lobby bot seats play `MctsBot` at the
  round-26 shape (64 iters, 3-turn, honest determinized rollouts) —
  the strongest adopted pilot, falling back to the heuristic on a bare
  checkout.

  *Training loop*: `play_recorded_game_mcts` + `selfplay_train
  --mcts-actors N`; rollout rewards route through SLOT_BEST, so
  `--gpu-eval` batches them on the collator with no extra plumbing.
  Measured generation: **2.6 games/s** (MCTS-64, 256 threads, batched
  eval, learner parked) — ~50× heuristic actors, as the round-26 cost
  ratio predicted. First pass armed: 50 k MCTS-piloted games vs a 50 k
  net-piloted same-budget control, seed 43, gates on both
  (`run_r27_loop.sh`) — the design isolates label quality from volume.
- 🟢 **Round 26 (2026-08-09) — MCTS-net: search amplification is real,
  and it's the new strongest pilot.** The rematch the heuristic-era
  verdict deserved: the champion net's win probability as a *native*
  UCB1 reward (calibrated [0,1], no logistic squash) with honest
  rollouts (hands redealt under determinize — the old library shuffle
  never covered held cards).

  | matchup (paired, 1 200 games/cell) | seed 43 | seed 97 |
  |---|---|---|
  | mcts-net (24 iters) vs net | 49.6 % | 49.2 % |
  | **mcts-net-deep (64 iters, 3-turn) vs net** | **53.4 % [50.6, 56.2]** | **52.5 % [49.7, 55.3]** |
  | mcts-net-deep vs gang | **56.1 % [53.3, 58.9]** | **54.4 % [51.6, 57.2]** |
  | mcts-net-deep vs atk-sim | **55.8 % [52.9, 58.5]** | **55.8 % [53.0, 58.6]** |

  **First profile ever to beat the adopted net pilot, using the same
  net** — and the best absolute numbers in program history (the gang
  gate jumped 51.8 → ~55.3 pooled). At 24 iterations the bandit only
  ties the decomposed searcher; at 64/3-turn it wins, so the scaling
  curve is live and unexplored upward. Cost: ~8–10 min per 1 200-game
  matchup (~50–100× the champion's per-game cost) — fine for client
  play (per-decision latency, human-paced), prohibitive for training
  generation without routing rollout evals through the batched
  inference server (they are exactly the batchable shape). Open next:
  the iteration/horizon scaling curve (128+), adoption as the ladder's
  strongest profile, and MCTS-net-generated training data.
- 🟢 **Determinized search priced (2026-08-09, task #25) — the
  information cheat was worth ~1–1.5 points and nothing rested on it.**
  Coverage fix first: the cast planner's dry-runs now redeal hidden
  zones under `determinize` too (one turn-keyed redeal shared by all
  finalists of a decision; the sequence recursion does NOT re-redeal —
  it continues a line through cards already drawn). New `net-det1`/
  `net-det3` profiles. The asymmetric gates (mirrors are blind — both
  seats cheat identically): `det1` vs `gang` 49.3/48.0 (the peek buys
  the heuristic ~1.4 pts); **`net-det1` vs `net` 48.9/50.0 — the
  champion barely uses the peek**; honest champion 50.4/50.7 vs gang,
  52.2/52.2 vs atk-sim. All these cells face *still-cheating*
  opponents, so they lower-bound the honest bot's standing vs humans.
  Consequences: the program's ladder conclusions survive un-asterisked;
  determinized deeper search (MCTS-net) is well-founded; and the client
  default should flip to `determinize: 1` — the honest bot is ~equal
  and it stops reading the human's hand (recommended, not yet flipped:
  it changes shipped gameplay).
- 🔴 **Round 24 (2026-08-09) — capacity is null-to-negative even under
  the regime that works.** The fair retest round 12 couldn't run: 2×
  representation width (emb 64, obj_hidden 128) on the champion config,
  heuristic actors (training throughput unchanged). AUC 0.8075/0.7871
  (below control, s97 notably weak), pooled gates **50.2 % gang /
  52.0 % atk-sim vs the r20 control's 51.8/54.4** — the atk-sim delta
  is marginally significant *negative*. Caveat: hyperparameters
  (lr, cosine horizon, patience) were tuned on the small model and were
  not re-searched for the wide one; "capacity at champion
  hyperparameters" is the claim. Within that claim the small-model
  story is now properly settled: 64-wide/32-emb is right-sized for
  this data distribution, and the round-11 "widening cannot recover
  discarded information" lesson holds from the other side too — width
  adds fitting surface, not knowledge. Next lever by elimination:
  **search amplification** (net-evaluated deeper/determinized search —
  the evaluator is now the best component; multiply it).
- 🔴 **Deck_duel rematch (2026-08-09) — the distilled judge's top pick
  is WORSE than the old net's.** The round-16 loop's final judge
  (`nets_distill7/deck-distilled.safetensors`, vs-static 61.0 %,
  exploit gate ~40 %) picked its best of 512 builds from
  `decks/sealed_pool.txt`; against the simulation judge's pick it lost
  **38.1 % [36.6, 39.6] / 39.2 % [37.7, 40.7]** (2 000 antithetic pairs
  × seeds 11/12) — below the pre-distillation net pick's 43.9/44.2 on
  the same protocol. The pick traded the sim build's four bodies for
  reactive spells + an 18th source (`decks/sealed_wb_distill.txt`).
  Reading: the vs-static gate scores *ranking over the whole candidate
  set against a weak judge*; a top pick is an argmax under judge error
  — winner's curse — and distillation fixed the exploit gate without
  fixing top-of-ranking value. The deck net's honest role is unchanged
  and now precisely bounded: fast surrogate ranker, never the pick.
  The sim judge sleeves the deck.
- 🟡 **Round 23 (2026-08-09) — actor-side softmax action sampling:
  null at the first dose.** Infrastructure landed and stays (thread-
  local `set_action_sampling` over the three live scored pickers,
  `--sample-temp`/`--sample-turns`, gates/sims argmax by construction,
  golden traces unchanged): temp 120 through turn 6 on the champion
  config gated **51.7 % gang / 53.7 % atk-sim pooled vs the r20
  control's 51.8/54.4** — null, atk-sim trending slightly down.
  Saturation improved a touch (8.0–8.5 % vs ~10 %). Untried: hotter/
  longer sampling doses, and sampling paired with net-piloted
  generation (where the argmax fixed point is the net's own). Note the
  sampled runs' holdout AUCs are not comparable to argmax rounds — the
  validation games contain exploration moves.
- 🟢 **Round 22 (2026-08-09) — blend is dead weight; data scaling is
  flat at 250 k; the round-20 net is the committed champion.** Two
  follow-ups to round 20, plus the adoption:

  *22a blend gates* (no training): with the r20 nets, `net-blend300`
  pools 52.1 % gang / 53.9 % atk-sim — indistinguishable from plain
  replacement (51.8/54.4) — and the quieter `net-blend` is *worse*
  (51.4/52.9). The historical ordering (blend > replacement, quieter >
  louder) has fully inverted: every unit of `eval_material` mixed in now
  dilutes a better evaluator. The heuristic crutch era is over;
  replacement is the pilot.

  *22b longer horizon + more games* (400 k games, cosine 90 k, two
  seeds): AUC 0.8189/0.8059 (s43 is the champion-line record), pooled
  gates 52.2 % gang / 54.0 % atk-sim — **null vs round 20** at 1.6× the
  compute. The games curve has flattened at 250 k under this
  architecture and window.

  *Adoption:* `nets/champion.safetensors` = the round-20 seed-97 net
  (AUC 0.8090; replacement gates 52.2/51.2 gang, 55.1/53.9 atk-sim),
  committed, and `bot_ladder` now falls back to it when `CRAB_NET` is
  unset — `--a net` works out of the box; CRAB_NET still overrides.
- 🟢 **Rounds 19–21 (2026-08-09) — cosine decay adopted; Muon and a 1M
  window are nulls.** Three regime-side levers on the champion config
  (attn + 500 k window + lr 1e-4 + 250 k games + patience 12), each
  implemented, unit-tested, and two-seeded against the r17b arm-A
  control (AUC 0.8126/0.8069, pooled gates 51.4 % gang / 52.9 %
  atk-sim):

  | round | change | AUC (s43/s97) | pooled gang | pooled atk-sim | verdict |
  |---|---|---|---|---|---|
  | 19 | Muon (muon-lr 0.02, adamw 3e-4) | 0.8140/0.7968 | 50.0 % | 51.1 % | ❌ null-to-negative |
  | 20 | cosine lr →10 % floor over 60 k | 0.8070/0.8090 | 51.8 % | **54.4 %** | ✅ **adopted** |
  | 21 | window 1M (on r20) | 0.8077/0.8084 | 52.3 % | 54.1 % | ➖ null-leaning-positive |

  **Round 20 is the win: best atk-sim gates in the program** (all four
  cells 53.3–55.1 %, both training seeds improved independently), with
  AUC flat — a pilot gain, not a predictor gain; the decayed tail
  fine-tunes instead of thrashing. Muon details: `--muon` hybrid
  (Newton-Schulz on hidden matrices, AdamW for emb/heads/biases/norms;
  routing + spectral-flattening + learning tests) — the lr probes
  showed muon-lr flat across 10× while the AdamW side starved at 1e-4
  (0.789 → 0.804 at 3e-4), and the full runs still landed below
  control with worse saturation (11.3/8.4 %). Window 1M: gang cells
  were the most consistent ever (all ≥51.7 %) but +0.5 pooled is
  inside noise — not adopted; harmless if RAM permits. **Champion
  config after this sweep: `--attn --window 500000 --lr 1e-4
  --lr-cosine 60000 --relabel-mode new --stop-after-stale 12`,
  250 k games** (`nets_r20_*`, pooled 51.8/54.4).
- 🟢 **Rounds 18 + 17b (2026-08-09) — the regime did it, not the
  blocks; self-play labels don't stack.** The two follow-ups to round
  17's sweep, both two-seeded:

  **17b attribution ablation.** Arm A (old `--attn` single-attention
  architecture under the NEW regime — 500 k window, lr 1e-4, 250 k
  games, patience 12): **all eight gates ≥ 50.7 %**, pooled **51.4 %
  vs gang / 52.9 % vs atk-sim**, AUC 0.8126/0.8069 — matches or beats
  round 17's blocks (50.8/52.5, AUC 0.8049/0.8025). Arm B (`--blocks 2`
  under the OLD regime — 250 k window, lr 1e-3, 90 k games, patience
  5): pooled 49.2 % / 51.1 %, AUC 0.7847/0.7776 — the historical
  pattern. **Verdict: the optimization regime is the entire effect; the
  transformer blocks add nothing measurable** (echoes round 11's
  "the ceiling was overfitting" — more data at lower lr with a patient
  stop was the binding constraint all along). By parsimony the champion
  config is `--attn` + new regime: simpler, cheaper for actors, and the
  best AUC on record for its class. Widening/deepening stays closed
  unless a capacity signal appears under the new regime.

  **Round 18 self-play promotion (r17 nets as pilots, own seed each,
  r17 regime).** Pooled gates **49.5 % vs gang / 51.5 % vs atk-sim** —
  within noise of round 17, trending a point down. The round-14
  "no compounding" result replicates at the new capability level;
  self-play labeling is closed as a stacking mechanism (two levels, two
  nulls). The seed-97 run posted **AUC 0.8204, the program record**, on
  identical gates — the predictor/pilot dissociation again (caveat: its
  holdout is its own pilot's distribution). Also the first production
  run of the batched eval: **95.7 games/s** net-piloted (vs 50.7 for
  the CPU-eval seed), 231 k games in 40 min, 0 stalls.
- 🟢 **Batched actor inference (2026-08-08) — 2.2× net-piloted
  generation.** User-designed game-pool architecture: hundreds of game
  threads block inside `NetEvaluator::eval` (a new engine seam —
  `net_eval` slots now hold `Arc<dyn NetEvaluator>`, `PlayNet` unchanged
  as the local impl); a collator (`BatchEvalServer`, `selfplay_train
  --gpu-eval --eval-batch N --eval-flush-us N`) batches their states and
  scores each batch in one candle forward on the GPU. Thread-per-game
  with blocking evals, not resumable searches — the OS scheduler does the
  game multiplexing, and the search code kept its shape. An 8-thread
  parity test holds the collator to the engine forward at 1e-4.
  Measured (3 000 net-piloted games, r17-s43 pilot, 2 blocks):
  **112.0 games/s (512 threads, batch 256, flush 200 µs) vs 51.5
  (22-thread CPU eval)** with the learner parked. Two findings around
  it: (1) collator knobs are flat (26.8–27.8 games/s across
  256–768 threads, flush 100–1000 µs) when the learner is active,
  because (2) **a training learner costs the actors ~4× in either arm**
  (batched 112→27.8, CPU 51.5→23.6) — GPU kernel contention plus the
  learner's CPU-side packing. This retro-explains round 18's 50.7
  games/s average (slow while the learner trained, fast after its early
  stop) and makes learner/actor scheduling the next infra lever if
  net-piloted generation stays the bottleneck. Strategic point: batch
  throughput is nearly flat in model size, so this is what makes
  wider/deeper nets affordable for actors at all.
- 🟢 **Round 17 (2026-08-08) — transformer blocks + longer/lower-lr
  training: first sweep of the replacement gates.** Proper pre-LN
  transformer blocks (`tblocks.*`: `x += attn(ln1(x));
  x += ffn2(relu(ffn1(ln2(x))))`, 4 heads, 2× FFN, group tag into the
  stream at stack entry) landed as a third tensor-presence-selected
  architecture with a 1e-4 parity test, and trained under a changed
  regime: **2 blocks, 500 k window (2×), lr 1e-4 (10×↓), 250 k games
  (~2.8×), stale patience 12**. Two training seeds; learner on the 4090
  (~27 steps/s, actors 123–137 games/s, 0 stalls, ~30–34 min/seed).
  Holdout AUC 0.8049 (s43) / 0.8025 (s97) — both above every prior run
  (r12 control 0.7973), replicated. The gates are the result:

  | replacement gate (paired, 1 200 games) | s43 net | s97 net |
  |---|---|---|
  | vs gang, ladder seed 43 | 51.0 % [48.2, 53.8] | 50.2 % [47.4, 53.1] |
  | vs gang, ladder seed 97 | 50.8 % [48.0, 53.7] | 51.1 % [48.3, 53.9] |
  | vs atk-sim, ladder seed 43 | 53.5 % [50.7, 56.3] | 52.2 % [49.3, 55.0] |
  | vs atk-sim, ladder seed 97 | 51.9 % [49.1, 54.7] | 52.4 % [49.6, 55.2] |

  **All eight point estimates ≥ 50 % — the first time the net as
  replacement pilot has swept both opponents on both training seeds.**
  Pooled: 52.5 % vs atk-sim (4 800 games, clearly above parity), 50.8 %
  vs gang (parity-to-slightly-above; every prior round lost this gate at
  47–49 %). Caveats: blocks, window, lr, and run length changed
  *together* per the round's design, so attribution among them is open —
  a `--blocks 2` run at the old regime (or `--attn` at the new one)
  would separate architecture from optimization; and vs-gang is parity,
  not dominance. Calibration saturation 9.2 %/7.5 % outside
  [0.05, 0.95], in line with prior nets. Next: promote a round-17 net as
  the self-play pilot (round-14 loop, gen 1 was the only prior
  above-50), and the attribution ablation if the levers matter for the
  next scale-up.
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
  builds. Actors can now judge best-of-32 candidates with it. **Unusable
  until a deck net is retrained**, and the reason is worth keeping: the
  embedding index used to be a card's position in the *sorted SOS pool*, so
  the eleven cards added since those nets were trained shifted every later
  row and retired all seven of them. Frozen at the fifty-fourth pass
  (`server::vocab_snapshot`) so it cannot recur, but a net from before the
  freeze cannot be recovered — nothing can say which card its rows meant.
  Judged builds also became affordable at the fifty-third and fifty-fourth
  passes (1.2 -> 83.2 games/s, and deck construction -68.8 % on top), so the
  retrain is the only thing left in the way.
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
- 🟡 **Mulligan decisions** — `HeuristicBot` ships flood/screw mulligans with
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

---

## Encoder and net follow-ups (moved from TODO.md, 2026-08-23)

Moved verbatim out of `TODO.md`'s Bot / AI section to keep that file under
its size line. Nothing here is edited; the "Next-round candidates" list at
the end is still the prioritized ML backlog.

### Belief-head recall diagnostic (round 39 follow-up)
The opponent-hand belief head (`head_opp`, round 39) trains and its
redeal gates flat: sims (`bdet1` vs `det1`) 50.2 / 49.9 at ±0.4 over
12 k games, rollouts (`bdeep` vs `deep`) 50.3 / 52.1 / 50.9 at ±1.9.
The null is **ambiguous between two stories that want opposite
follow-ups**, and `loss_opp` cannot tell them apart — at ~5 held names
in a 164-name vocabulary it is dominated by the easy zeros (0.080
against a ~0.135 constant-base-rate floor, plateaued by step 14 k).

*Measure recall, not BCE*: on held-out snapshots, how often is a card
the opponent actually holds inside the head's top-5, against the
uniform-over-unseen baseline the determinizer already samples? The
machinery `val_policy` uses scores exactly these rows; this is a
diagnostic mode, not a training change, and it runs in minutes.

- **Head is weak** (barely beats uniform) → the belief is the problem.
- **Head is strong and the gate is still flat** → *consumption* is the
  problem: more redeals per decision, or a rollout policy that uses the
  hand at all (rollouts currently play the redealt hand with
  `uniform_baseline()`, i.e. at random).

Two structural facts to keep in view before spending another round.
(a) The uniform baseline is stronger than it sounds — `determinize_hidden`
redeals from the opponent's *true* unseen cards, so it already knows
their deck and only gets hand-vs-library wrong; the head's whole job is
sorting ~5 of ~35 known cards. (b) A belief head can only learn tells its
training data contains, and these pilots barely make any: the SoS probe
measured 42 cleanup discards per 60 games against **one** instant-timing
cast, and `atk-hold` gated 49.4 %. "They left two Islands untapped"
carries little information in a distribution where nobody holds up mana
on purpose — which also means this direction may be capped in *this*
format while still mattering against a human who does.

### Encoder gaps — implemented as encoder v7 (round 40)
All three are in `server/encode.rs` behind their own ablation bits
(`hist`, `exp`, `ctr`; see `ABLATION_BLOCKS`), `SHARD_VERSION` is 8, and
older checkpoints zero-pad at load in *both* net implementations. The
gate is `.ladder/run_r40_encoder_v7.sh`; the verdict lives in
`ML_NOTES.md`, not here.

- **(a) Turn-scoped history** — globals 43..=54, six counters per seat
  (life gained, instants/sorceries cast, spells cast, creatures died,
  cards that left the graveyard, cards exiled). The block the census
  says is real: non-zero in 3.6–51.7 % of positions.
- **(b) Expiry** — object feats 45..=47: marked damage, and the P/T
  delta that expires at cleanup. Occupancy 0.13–0.16 % of objects under
  the default recorder, which is why it did not earn its own arm.
- **(c) Counter types** — object feats 48..=52: +1/+1, −1/−1, stun,
  Page, Growth, splitting the single scalar at feat 34. Occupancy
  0–1.6 %.

Not a gap, recorded so it is not re-proposed: **the prepared spell is
already covered.** `prepare_spell: Option<Box<CardDefinition>>` is a
field on `CardDefinition`, fixed per card, so the embedding carries
which spell is inset exactly as it carries a card's other abilities;
`f[9]` supplies the live "is prepared" state on top.

### Static-anthem P/T is invisible to the encoder
`encode_battlefield_object` reads `CardInstance::power()`, which sums
the printed base, `power_bonus` (until-end-of-turn pumps),
`perm_power_bonus` and the P/T counters — but *not* continuous effects
resolved through the layer system. So an anthem's or an aura's +2/+2
does not reach the net; the creature encodes at its unbuffed stats. The
relation flags (feats 31/32, own/opposing attachment) say an aura is
there, not what it does.

Deliberately not fixed: the correct value needs
`GameState::compute_battlefield`, whose effect gather is ~10 % of
simulator instructions, and `encode_state` runs once per net eval
inside the search. The SoS pool has five P/T-affecting statics in the
whole set (2 `PumpTeamIf`, 2 `PumpSelfIf`, 1 `PumpPT`), so the trade is
bad *for this format*. Revisit if the pool ever changes: a format with
real lords would make this the largest remaining encoder hole, well
ahead of anything in the v7 blocks.

2026-08-17: the `claude/modern_decks` branch is exactly such a pool
change. If modern decks enter the training or gating pool, this becomes
the priority encoder item ahead of everything in "Next-round candidates"
below except search throughput — and the ~10 % gather cost should be
re-measured (cached per-eval `compute_battlefield`, or an encode-time
approximation), not assumed from the SoS-era profile.

### Feature occupancy is a precondition, not an afterthought
`selfplay_train --feature-census N` reports how often each encoder
feature is non-zero in recorded self-play positions. It costs no GPU
and one self-play pass, and it found that thirteen features — the whole
round-28 combat block plus the attacking/blocking flags — are
identically zero in every training row, because the recorder never
snapshots a combat step. **Run it before proposing or gating any
feature block.** A block with 0 % occupancy cannot move a gate, and a
gate that ran anyway (round 28f) produced a null that means nothing.

Open follow-ups it raises:
- Global 36 (declare-attackers one-hot) is 0 % on *both* sides of the
  census and several object feats sit under 0.1 %. Worth a pass to
  decide which dead columns should be removed rather than carried.

**The leaf column is done** (`--feature-census` reports train vs leaf
side by side, plus the features the search meets most
disproportionately). Result in `ML_NOTES.md` round 40: two-thirds of
what the search evaluates is a settled post-combat state whose phase
flag has never been trained, and the whole `hist` block is 1.8–3.6×
denser at the leaves than in training. One remaining experiment it
suggests, with its own warning attached: record *only* the settled
end-of-combat state rather than all four combat steps, since arm C's
rows were only ~19 % that shape. Do not run it on the strength of the
table alone — arm C closed this exact gap and lost 3 points of search
strength.

### Next-round candidates (2026-08-17 analysis; post-r41, r42 in flight)

A prioritized reading of `ML_NOTES.md` rounds 26–41 plus round 42's cost
data. The through-line: pilot-weight interventions are nulls at every
scale tried, and the headroom is inside the search or in what the search
consumes (r38's verdict). House rules apply to every item — four
training seeds paired within seed, *t* intervals, `--feature-census`
before any feature block, pre-registered scripts in `.ladder/`.

1. **Iterations are the lever; make them affordable.** r42 Part A's
   first ladder seed has `mcts-net-256` over `mcts-net-deep` (64) at
   **+2.1 ±0.4 head-to-head** — ~5× the entire v7 effect, and the
   largest resolvable strength effect since round 27. Cost is linear in
   iterations (33.0 → 121.9 s/game serial at 64 → 256, r42 Part C), so
   adoption is latency-gated, and the highest-leverage work is
   search-eval throughput, not modeling — see the `PERF.md` candidate
   ("MCTS leaf-evaluation throughput"). Every 2× there is a rung on the
   only curve that climbs. **Part 1 landed 2026-08-19 (PERF.md
   forty-first pass): vectorized matvec, 64-iter 33.0 → 18.4 s/game,
   256-iter 121.9 → 73.0.** The remaining 88 % of search wall is the
   rollout sim (~63 engine actions/rollout), so the next rung is
   rollout-side and ladder-gated, or the engine's own action loop.
   **Round 44 closed the horizon shortcut: h1 loses 7 points to h3
   and cost-matched 3× iterations buy back +0.3 — the shallow leaf
   is biased, not noisy, and h0's −35 shows the net cannot score
   unsettled states at all. Do not respend here; the census/`head_leaf`
   direction (item 2) inherits the evidence.**
1a-bis. **A rare effect is not an unmeasurable one (round 50).** The
   walker cash-out fix reads +0.25 with both cells' intervals clearing
   50, at **±0.21 / ±0.12** against the usual ±0.57 — when most
   antithetic pairs are exact mirrors they add no variance, so the few
   differing games are measured precisely. The zero-incidence games are
   perfect controls, not waste. This splits a class we had lumped
   together: the five earlier "zero incidence" flags returned *exactly*
   50.0 ±0.00 (never fired at all), which is a different finding from
   "fires rarely and helps". **Run the rare-class flag before declaring
   it unmeasurable.**

1a-ter. **A search-level flag does not inherit round 50's free precision
   (round 51).** Same decision density (~0.25 fetches/game), two flags:
   the heuristic-level one measured at ±0.11 (46 of 6000 pairs diverged),
   the search-level one at ±0.60 (4682 diverged). Adding arms perturbs
   the search's own random stream, so the antithetic mirrors break and
   the pairing collapses. Budget ~4× the cells for a sub-0.5 effect
   inside the search. `fetch_arms` came back +0.35 unresolved on two
   seeds and is left in, off by default; the demand-aware ranking under
   it is a precise **zero** and is kept only as a consistency change.

1a. **The mulligan is closed for now — two mechanisms, both failed.**
   `mull_quality` (a better predicate) 50.2 % over 28 800 games;
   `mull_sim` (play both branches forward) **47.45 %**, i.e. actively
   worse, with cost, mulligan rate, sample noise and comparison
   fairness all ruled out (round 49). Mulligan is 25 % of all
   decisions, so the volume argument for attacking it is real and was
   not enough. Do not re-open without a *different* scoring signal —
   a short-horizon material eval is the thing that failed, not the
   idea of simulating.

1b. **Menu holes are the piloting lever; valuation refinements are not.**
   Round 46 adopted `target_arms` (+0.95, two seeds) — the second
   adoption in four rounds and the second whose mechanism was a
   *missing candidate* rather than a bad score, after chump blocks
   (+0.9, r43). Both let the search express a line it previously
   could not. The same round's abilarms re-run went the other way
   (48.9, replicating r43 on a fixed targeter), and the contrast is
   the design rule: a *variant of a play the search already likes*
   pays for its arm; an unvetted new action type displacing vetted
   casts under the six-arm cap does not. Look for menus that cannot
   express a line, not for weights that score one wrongly.

2. **A separate leaf-value head the search consumes and the pilot
   doesn't.** The census says ~two-thirds of what the search evaluates
   is a settled post-combat state whose phase flag has never been
   non-zero in training, and the `hist` block is 1.8–3.6× denser at the
   leaves (that skew is v7's confirmed mechanism). Arm C proved the
   in-place fix fails: retraining the *shared* win head on combat rows
   moved the pilot's calibration and cost the search 2.9 points. The
   untried cell splits the roles r37-style: recorder and `head_win`
   untouched, a `head_leaf` trained only on settled end-of-combat rows
   (or a leaf-matched mix), `mcts-net-deep` reads `head_leaf` while the
   1-ply pilot keeps `head_win`. Gate: `mcts-net-deep(head_leaf)` vs
   `mcts-net-deep(head_win)`, same trunk both sides. This *subsumes* the
   "record only the settled state" experiment above — do not run that
   one into the shared head. **Evidence upgraded twice since written:
   r44's h0/h1 cells showed the net's error on off-distribution state
   shapes is *bias* (iterations can't fix it, settlement is
   load-bearing), and r45 closed capacity (+0.01 ± 0.3 at 2× fed) —
   representation/distribution is the only training-side axis left
   open. This is the next training round to run; a Go-Exploit-style
   arm (seed some self-play starts from mid/post-combat snapshots) is
   the data-side twin and belongs in the same round.**
3. **Distill deep search into the leaf head — amortized iterations.**
   256-iteration search conclusions as `head_leaf` targets, consumed by
   a 64-iteration search. r36's coupling warning ("you cannot distil
   your way out of a search that eats the same net you improved") is
   about the pilot-vs-search gap; a separate leaf head is its own
   prescription. The mixed fleet makes the data nearly free (~10 k
   searched games rode along per seed in r38). Composes with item 2:
   same head, deeper-search targets instead of game outcome.
4. **v7 × 256 iterations, the cheap stack.** v7 replicates at +0.4 but
   needs a partner or a higher-starting regime. Its mechanism is
   leaf-side density, so it should express more, not less, at higher
   iteration counts — and the r41 v7 nets already exist, so the gate is
   ladder-only: `mcts-net-256` + v7 net vs `mcts-net-256` + champion,
   paired ladder seeds. This is the system-adoption question r42 sets
   up.

5. **Builder v3 default flip + training-deck-quality question.** The
   quality/curve shape ranker measured **+3.2 replicated** over v2
   (`ML_NOTES.md` "Builder v3", 2026-08-17) — adopted in the client's
   sealed opponent, still default-off in `SimConfig` because flipping
   it changes the training field *and* the ladder's sealed gate decks,
   ending comparability with every recorded reference. The follow-up
   round: re-baseline champion and gate opponents on v3 fields, then
   the untried ML question — does a pilot trained on stronger (v3)
   fields play better, or is builder noise useful curriculum? Pin the
   gate builder while the training builder varies, or the comparison
   confounds.

Not worth new rounds, per the accumulated record: pilot-weight
interventions (capacity, labels, value targets — three scales of
evidence), MCTS knobs other than iterations (r29), opponent modeling in
this format (run the recall diagnostic above, then likely park the
thread), gen-1 Gumbel completed-Q targets (the one untried Gumbel
variant; two nulls and the prior-starvation negative make it a worse bet
than anything above — on the list, not in the next round).

## The deck-net vocabulary freeze (moved from TODO.md at the fifty-seventh pass)

Verbatim from `TODO.md`'s "ML — defects" section; the index entry there
points here. Nothing is summarized away — the point of the record is that
nobody re-derives the half that is deliberately *not* fixed.

### Every committed deck net fails to load — FIXED for the future, not for those nets

**What it was.** `Vocab::sos_sealed()` derived its embedding indices from
`draft::sos_draft_pool()` in *sorted-name* order, so adding one card to the
SOS set shifted the index of every card sorting after it and silently
retired every net trained before it. It surfaced as
`deck net vocab != encoder vocab — left: 153 right: 164`: all seven
committed `*/deck-latest.safetensors` are eleven cards behind.

**The fix (fifty-fourth pass).** `server::vocab_snapshot::VOCAB_SNAPSHOT`
freezes the assignment — a name owns `position + 1` whether or not it is
still in the pool, and a pool name outside the snapshot is appended after it
in sorted order. So a card addition *or removal* grows the table at the end
and never moves an index a net depends on. `PlayNet` / `DeckNet::pad_vocab`
zero-extend a shorter table (and the vocabulary-sized opponent head), and
`vocab_fit` decides whether that is allowed.

**Verified end to end at the fifty-fourth pass.** A throwaway deck net
trained at the current vocabulary (`selfplay_train --actors 3 --games 4000
--steps 30`, 30 steps, val AUC 0.62 — not committed, it is far too
undertrained to be a judge) loads, pads and drives `--use-deck-best`: ~7,600
judged actor games, 0 stalls. So the loader path is not the thing standing
in the way; a *good* deck net is. The judged path now runs at **91.7 %** of
the unjudged rate (148.9 vs 162.3 games/s, best of four alternated), against
83.4 % at the fifty-third pass — best-of-32 building is where the deck-builder
work compounds thirty-two-fold.

**What is deliberately *not* fixed, and it is the interesting half.** A net
whose vocabulary is smaller than `FROZEN_VOCAB_SIZE` (164) predates the
freeze, so nothing can say which card each of its rows meant — padding it
would load cleanly and mean the wrong cards, which is worse than the loud
failure. `vocab_fit` refuses those by name. **The seven committed deck nets
are in that bucket and still need retraining**; `--use-deck-best` stays dead
until one is trained. The snapshot was seeded from the then-current sorted
order, so `nets/champion.safetensors` (164) is unaffected and **no net needs
retraining because of this change**.

**Also fixed while there:** both forward passes clamped an out-of-range card
index with `.min(emb.rows - 1)`, mapping an unknown card to *the last card's*
embedding rather than to index 0, the reserved unknown slot. Unreachable
before only because the hard size check rejected the net first; reachable the
moment padding exists.

## Representation correctness fixes (2026-08-30, from the state/decision audit)

Four defects where the encoding or the search's imagined future diverged
from engine truth. Landed as correctness changes (the house rule's
re-justification clause), not measured wins — what each invalidates or
confounds is the point of this entry.

- **MCTS rollouts now answer pending decisions with
  `decide_pending_policy`** (`server/mcts.rs`; heuristic rollouts only —
  the uniform-rollout control keeps `AutoDecider`, since its actions are
  random anyway). The rollout was the one lookahead still scoring every
  line under an AutoDecider future: no-op scries, first-candidate tutors,
  head-of-hand discards, declined optional triggers, amounts of zero.
  That is the exact divergence `decide_pending_policy`'s doc says it was
  extracted to close, applied to the seven heuristic sim loops and never
  to this one. Consequence for the record: mcts ladder numbers from
  before this date were measured under the weaker rollout policy; quote
  them as such next to any new mcts reading.
- **Combat encoding honours CR 509.1b** (`server/encode.rs`): feat 29
  and globals 39/40 read `blocked_attackers` (unioned with `block_map`
  so synthetic states still work), so an attacker whose blockers died to
  first strike or removal no longer feeds its full power into "incoming
  unblocked power" when the damage step will deal zero. Globals 39/40
  also now carry trample-through: the excess over live blockers'
  effective toughness (the default lethal-to-each assignment), which is
  everything once every blocker is gone. This corrects the signal at
  settled post-combat states — the exact leaf shape the round-40 census
  said the search evaluates most.
- **Dead trigger sources encode from LKI / the graveyard**
  (`encode_stack`): a dies-trigger — the most common trigger class on
  the stack — encoded as an all-zero object, including feature 27 at 0.0
  where every real object carries 0.25. The truly-unknown fallback now
  keeps that baseline too. Exile is deliberately not searched (face-down
  cards mix into it; see the audit's leak note).
- **Feature 34 no longer rides `rel`'s ablation bit.** The counter sum
  is battlefield state, not a relation; gating it under `rel` meant
  every `--ablate rel` arm ever run measured "relations plus counters"
  and its result is not attributable to either. Standing caveat on the
  round-12 rel attribution. Also: feat 42 now reads
  `CounterType::Indestructible` (CR 122.1), which the keyword walk
  cannot see.

Distribution note before the next gate: the champion and every committed
net trained on rows carrying the old combat/trigger signals. The encoder
now tells the truth, but a net trained on the lie will be consuming a
distribution it never saw at exactly the settled-combat leaves; a ladder
arm that mixes old nets with the new encoder measures the fix *and* that
mismatch together. Re-gate the mcts profiles (and re-run
`--feature-census` if sizing anything against occupancy) before quoting
new numbers against pre-fix baselines. The heuristic-pilot ladder is
unaffected (it never encodes).

## Round 52 — value-ordered combat damage (CR 510.1c): ADOPTED

The first cash-out of the state/decision representation audit's menu-hole
list. `Decision::CombatDamageOrder` had no policy arm anywhere — not in
`decide_pending_policy`, not in the search — so every multi-blocked
attacker in the program's history dealt lethal in declaration (CardId)
order, and the 510.1e mirror divided a multi-blocker's damage the same
way. The same missing-candidate shape as chump blocks (r43, +0.9), one
step later in the same combat.

**The change** (`EvalWeights::damage_order`, profile `dmgorder`):
`decide_combat_damage_order` simulates the engine's own
`default_damage_split` per candidate order — exhaustive to five victims,
one greedy order past that — and answers with the best signed
`permanent_value` outcome for the deciding seat. Signed, because banding
and Defensive Formation hand the decision to the victims' controller,
whose best order is the reverse; one policy serves both chairs.
Deathtouch prices lethal at one (CR 702.2c), as `combat_assignment_plan`
does. Strict improvement only: an order that merely ties the default
answers empty, so a game where the choice cannot matter plays — and
antithetically pairs — exactly as before the flag. The
`AssignCombatDamage` sibling deliberately keeps the engine default
(lethal-to-each-then-trample is the assigner's optimum outside the
banding deny-trample corner; the pool has no banding).

**Incidence first (r50 discipline)**: 181 asks / 1500 SOS probe games =
~0.12/game, 2.5 % of decisions (`.ladder/r52_probe.txt`) — rarer than
r51's fetch (0.25/game), same class as r50's walker (~0.18/game).

**The gate** (`.ladder/run_r52_dmgorder.sh`, pre-registered): `dmgorder`
vs `gang`, sealed mirrors, seeds 43 and 97, 12 000 games each.
**50.4 % [50.2, 50.6] and 50.3 % [50.1, 50.5]** — both cells' intervals
clear 50, replicated, pooled 50.35. The measurement behaved exactly as
the rare class predicts: 5 817 of 6 000 pairs split as exact mirrors
(within-pair rho −0.939, 12 000 games carrying the precision of
~197 000 independent ones), and the sweep asymmetry ran 115 A to 68 B.
Adopted per the pre-registered r50 rule; `EvalWeights::default()` now
carries `damage_order: true` with the dated comment, and the exact gate
remains re-runnable because neither `dmgorder` nor `gang` carries the
default's determinize/chump layers.

Worth keeping for the next menu-hole round: the two heuristic-level
levers this audit priced cheaply (this and the MCTS rollout-policy fix)
both landed inside a week of the audit; the remaining list
(mid-resolution ChooseTarget arms, ChooseModes, X-as-a-branch, Serum
Powder) is in the audit memory and the ranked shortlist stands.

## Round 53 — outcome-judged mid-resolution targets: measured, NOT adopted

The second menu item off the representation audit, and the honest null
to round 52's adoption. `Decision::ChooseTarget` on the suspending path
(trigger target picks, cast-slot / off-board picks) is answered by
`decide_choose_target` — a polarity guess: opponent's biggest, else our
cheapest, else lowest-life opponent. Wrong in three documented ways: a
beneficial trigger whose legal set spans both sides buffs THEIR biggest
creature; undersized removal marks the 3/3 instead of killing the 2/2;
an optional "up to one" target can never be declined. `target_eval`
(profile `targeteval`) settles the corner candidates through
`settle_answer` and replaces the guess on strict improvement only, at
real decisions only. Both failure modes are pinned by unit tests
through the real `drain_trigger_queue` suspend path
(`target_eval_tests` in bot.rs).

**Incidence**: 0.65 asks/game, 13.2 % of all decisions — the densest
family gated yet (`.ladder/r53_probe.txt`).

**The gate** (`.ladder/run_r53_targeteval.sh`, pre-registered):
`targeteval` vs `gang`, sealed, seeds 43/97, 12 000 games each.
**50.1 % on both seeds, intervals straddling 50** (the ladder's own
verdict line) — pooled +0.10, sweep asymmetry same-signed on both
(37/24 and 43/32). Per the pre-registered r50 rule this does **not**
adopt: flag stays off by default, code, tests and profile stay for
re-measurement — the `fetch_arms` disposition.

**The diagnostic worth keeping**: despite 5× round 52's decision
density, divergence was *rarer* — 5 939 and 5 925 of 6 000 pairs exact
mirrors (rho −0.975/−0.980; 12 000 games carrying ~500 000 games'
precision). The strict-improvement rule turns the flag into a
measurement of its own premise, and the answer is that in SOS sealed
the polarity guess is almost always already right: this pool's
mid-resolution targets are overwhelmingly removal-shaped, and
beneficial-span-both-sides triggers are a thin slice of the 0.65. That
is a pool property, not a mechanism failure — the same read as the
static-anthem deferral. **Re-run when modern/cube decks (denser
triggers, equipment, counters-matter) enter a pool**; it rides the
existing script unchanged.

Also recorded during implementation, for the next targeting round: the
seven INLINE `self.decider.decide` ChooseTarget sites in
`effects/mod.rs` (council votes, copy retargeting, "change the target",
effects/mod.rs:759/1335/1403/11736/14913/16436/16476) never reach any
policy at any flag setting — AutoDecider first-legal for every seat,
bots, sims and humans' opponents alike. The `Decider` trait has no
`&GameState` access, so these need per-site suspend plumbing
(`ResumeContext`), not a policy change. That is the remaining
ChooseTarget hole, and it is invisible to this round's gate.

## The vocabulary grows to the cube pool (2026-08-30) — modern precondition 1 of 3

The first precondition of the modern-pool track, landed while the pools
still train on SOS sealed so nothing depends on it yet. `VOCAB_SNAPSHOT`
grew from 163 to 2,272 names: the modern cube pool enumerated by the new
`cube::cube_pool_all()` (union of `colorless_pool` + `color_pool` over
all ten pairs — **2,109 distinct names, not the ~309 the stale cube.rs
comment claims**; the catalog expansion poured straight into the pools)
appended after the SOS seed in `Vocab::sos_sealed()` order, exactly the
append-only contract the freeze was built for. No SOS index moved.

**The one landmine, defused**: `FROZEN_VOCAB_SIZE` was derived as
`VOCAB_SNAPSHOT.len() + 1`, so the append would have jumped the
pre-freeze refusal floor from 164 to 2,273 and refused every committed
net — champion first. It is now a pinned literal (164, the freeze
*date's* size) with a compile-time guard, and
`the_freeze_boundary_is_pinned` holds position 162 ("Zimone's
Experiment") so a mid-seed insertion fails loudly.
`vocab_covers_the_cube_pool` mirrors the SOS coverage test for the new
segment.

**Verified live**: `nets/champion.safetensors` (164 rows) loads,
`vocab_fit(164, 2273)` passes, `pad_vocab` zero-extends, and the padded
net plays (7-1 vs baseline over a smoke run). A pre-append net reads
every cube card as unknown — exactly what it did before, via zero rows
instead of index 0.

What this does NOT do: cube cards still reach a pre-append net as
unknowns, and no training run has fed the new rows. The next
preconditions stand as written: (2) layer-aware encoding
(`compute_battlefield` — keywords/types/colors, not just P/T; five raw
`power()` sites), (3) the v8 feature block (artifact/enchantment type
bits, Lore/Charge/Shield/Finality counters, land-drop global) under one
ablation bit with SHARD_VERSION 9. Then a `--decks cube` leg and a
training-pool flag, at which point the r53 `targeteval` re-run and the
zero-incidence flags (walkerchip, buff2for1, convlands, impulse) all
become live again.

## Layer-aware encoding (2026-08-30) — modern precondition 2 of 3

The documented #1 encoder gap for a modern pool, closed. Battlefield
objects now encode layer-resolved truth: `encode_state` opens one
`with_frozen_layers` scope (nested scopes reuse it — the castability
block already opened one) and every battlefield read goes through
`computed_permanent_on`, memoized per permanent inside the scope. What
changed, site by site:

- **Feats 4/5 (effective P/T)**: computed power / toughness − damage.
  An anthem's +1/+1 finally reaches the net — 258 `PumpPT` statics in
  the catalog against the 5 the SOS deferral was priced on.
- **Feats 1/2/3 (type flags)**: computed card types — an animated
  manland reads as the creature it currently is.
- **Every keyword-derived feature (12..=19, 40..=44)**: rebuilt from
  `ComputedPermanent.keywords()`, the final word after static grants,
  EOT grants, CR 122.1b counters (all folded by the gather), removals
  and lose-all. Static grants never touched the instance fields the old
  walk read; granted ward now reaches feat 41. One deliberate contract
  change rode along: a keyword *counter* now reaches the class flags
  too (a hexproof-countered creature IS hard to target) — the old
  exclusion was `any_keyword`'s resolution limit, not a ruling; the
  test was updated with the reasoning.
- **The other contaminated sites**: `eff_pt` (combat feats 37/38 and
  the blocker sums), the lands/creatures/power totals (globals 14..=23),
  and the gl 39/40 through-power loop (computed power AND computed
  trample — a sword-granted trample was invisible to the raw walk).

Off-battlefield objects (hand, graveyards, library, stack) keep the
printed+instance walk — no layers apply off the battlefield.

**The cost, re-measured as prescribed** (not assumed from the SoS-era
~10 % gather profile): release `bot_ladder`, net-vs-net pilot (encodes
every eval), 1 600 games per cell, same seed —
fixed 3.0 s → 2.9 s, cube 3.4 s → 3.3 s (pre-layer → layer-aware).
**Within run-to-run noise on both pools, cube included** — the frozen
scope pays one gather per encode and one memoized layer pass per
permanent, and the branch's perf passes had already crushed the gather.
Single runs each side, so this is "no measurable cost at 3-second
resolution", not a speedup claim.

Distribution note, same as the combat fixes: nets trained on unbuffed
rows now see buffed ones wherever statics exist. In SOS sealed that is
five cards' worth of drift (the original deferral's own count), so the
champion is effectively unaffected there; on a cube pool the drift is
the whole point and only newly trained nets will have seen it.

Remaining precondition (3): the v8 feature block — artifact/enchantment
type bits, Lore/Charge/Shield/Finality counters, land-drop-remaining
global — under one ablation bit with SHARD_VERSION 9. Then the pool
flags, and the r53 re-run.

## The v8 feature block (2026-08-30) — modern precondition 3 of 3

SHARD_VERSION 9. OBJ_FEATS 53 → 59, GLOBAL_FEATS 55 → 57, one ablation
bit (`v8`), and `LEGACY_FEATS` gains the v7 pair (53, 55) so every
existing checkpoint zero-pads at load — verified live: the champion
loads through the pad and plays bit-identically to its pre-v8 smoke run
(same seed, same 8-game line), which is the pad computing exactly what
the old binary computed.

- **Object feats 53/54 — artifact / enchantment type bits**, printed in
  every zone, layer-computed on the battlefield. The round-4 flags left
  "not a creature, not a land, not a planeswalker" as one class, and
  with the embedding dead for off-vocab cards that class held most of a
  modern board's non-creature permanents: Chalice, a Signet and a Saga
  encoded identically.
- **Object feats 55..=58 — Lore /3, Charge /4, Shield /2, Finality /2.**
  The modern counter kinds the v7 split has no slot for; all previously
  folded into feature 34's undifferentiated sum (a saga on chapter III
  and a stunned creature read the same). Feature 34 still sums
  everything, the same deliberate double-encoding as +1/+1.
- **Globals 55/56 — land drops remaining**, self then opponent, /2.
  `can_player_play_land` gates locks and the count (not the turn, so
  the off-turn seat reads its standing drop); feature 26's next-turn
  castability assumed the drop was there, and now the net can see it.

Census note, recorded so the occupancy rule is not mis-applied: this
block is FOR the cube pool, and its sealed-row occupancy is expected
low-to-zero for the counter kinds (SOS has artifacts and enchantments,
no sagas or chalices). That is the vocab-extension situation again —
infrastructure ahead of the pool switch, adoption decided by training
rounds on the pool that exercises it, not by a sealed census. Run
`--feature-census` on cube rows when a cube training pool exists.

**The modern-pool preconditions are now all three landed** (vocabulary
2,272 frozen names; layer-aware encoding at noise-level cost; this
block). What remains before modern decks can enter ML pools is wiring,
not representation: a `--decks cube` ladder leg and a `selfplay_train`
pool flag — and that flip ends comparability with every recorded SOS
reference, so it is a program decision, not a code one. On the flip:
re-run `.ladder/run_r53_targeteval.sh` (parked on a pool property),
re-run the zero-incidence flags (walkerchip, buff2for1, convlands,
impulse), and run `--feature-census` before sizing anything new.

## Replay diagnostic: the client bot's combat judgment (2026-08-31)

Fifteen human-vs-bot client games from 2026-08-30 (`replays/replay-17881*.jsonl`,
with the shadow decision logs in `replays/decisions.jsonl/` — local, not
committed), reconstructed board-by-board with the decision logs' bot
counterfactuals. Both reported failure modes are real, reproducible, and
mechanically distinct. The client pilot under test:
`MctsBot { iterations: 64, weights: net_eval_det1, ..default }`
(`server/lobby.rs:44`) — and `search_combat` is off in that default, so
every attack declaration fell through to the heuristic's one-turn sims
with their settled leaves scored by the CHAMPION NET, not the material
eval.

**Under-attack, the smoking gun.** Game 5 (replay-1788118995-5), turns
18 and 20: the bot held two Campus Composers (3/4) and a FLYING Emeritus
of Ideation (5/5) against a defending board with no flyer and no reach —
five unblockable damage a turn, declined twice, at 3–8 life against
17–23, where racing was its only line. Tap-state verified from the event
stream: everything untapped at declare-attackers, spells sequenced
post-combat. It lost at −2. Game 14 shows the chronic form: five parity
turns holding the bigger board, then an attack with exactly the wrong
subset (a 1/2 and a 4/2 into five blockers; both died for a 2/2).

**Over-attack, convicted by the shadow log.** In game 3 the bot's
counterfactual in the HUMAN's seat would have rammed Shopkeeper's Bane
(4/2 trample) into two-then-three 2/2s on three separate turns — the
trade-down the human declined all game while winning. The old SOS probe's
82 %-of-eligible over-attack pattern, still alive: the one-turn sim
prices "even trade plus chip damage" positive, sees no tempo, and casts
nothing for the defender (the standing open lead).

**Three apparent errors that are not** (recorded so the next reader
doesn't re-flag them): Moseo attacking into bigger ground creatures
flies; the 0-power Pensive Professor attacks are Increment payoffs —
both seats make them; Rancorous Archaic reads 2/2 printed but enters
with converge counters, so the human attacks the bot declined were with
a much larger trampler than a naive replay read shows.

**Mechanism.** The r40 census finding, observed in the wild: the sims'
settled post-combat leaves are a state shape the v5-era champion never
trained on (two-thirds of what gets scored; r44 proved the error is
bias). Compounding it at lopsided life: the calibrated win head
SATURATES — the documented histogram failure ("a flat landscape in which
every candidate line scores the same... turns a better predictor into a
worse player"). At 3-vs-23, "attack with the unblockable flyer" and
"hold everything" both score ≈0 and the tie goes to passivity. That is
the game-5 hold, exactly.

**Remedies, in the order to try them:**
1. *Saturation fallback* (small, heuristic-level): when the net's win
   probability is in the saturated tails, hand the decision's scoring
   back to the material eval, which stays discriminative there. The
   ply-taper blend (`ply_blend_factor`) is the existing precedent for
   muting the net's voice where it is known-bad. Flag + r52-style gate;
   note the ladder may under-read it (both mirror seats saturate
   together, and saturated positions are often already decided) — the
   client-facing quality argument stands independent of the ladder
   number, the same shape as the determinized-search adoption.
2. *head_leaf* (queued, the real fix): a leaf head trained on
   settled/leaf-matched rows, consumed by sims and search only. These
   replays are its first client-visible evidence.
3. The over-attack half stays with the standing open lead (sims that
   respect the defender's open mana / cast for both sides).

## Round 54 — the saturation fallback: outcome-neutral, adopted for the client

The replay diagnostic's first remedy, built and measured same-day.
`net_tail_guard` (profile `net-guard`): the scored combat pickers
(`pick_attacks_scored` / `pick_blocks_scored`) evaluate the PRE-DECISION
state once and, on a net read outside the calibrate histogram's own
rankable band [0.05, 0.95], silence the net for that one decision — all
candidates score on the material eval, which stays discriminative where
the sigmoid's p·(1−p) sensitivity has collapsed below the tie-break
jitter. Keyed per decision, never per leaf (one argmax, one currency);
mid-band untouched; the decided-game clamp extended from "decided" to
"effectively decided". Contract pinned by `tail_guard_tests` with a
constant-probability stub net, including the game-5 shape: under a
saturated net the guarded picker declares the attack the material
weights find.

**The gate** (`.ladder/run_r54_tailguard.sh`, pre-registered):
`net-guard` vs `net-det1`, champion loaded, sealed, seeds 43/97.
**49.9 % [49.9, 50.0] and 50.0 % [49.9, 50.0]** — the most precise null
the ladder has produced: 5,977 and 5,985 of 6,000 pairs exact mirrors
(rho −0.992/−0.995; 12,000 games carrying the precision of 1.5–2.4
MILLION). The flag fires and changes play, and virtually never changes
who wins — exactly the pre-registered mechanism: saturated positions
are mostly already decided, so the ladder is structurally blind to
playing them better. The ~15–23 divergent pairs per seed lean 14 A / 23
B pooled — ~1.5σ, unresolved at even this precision, recorded rather
than chased.

**Adopted for the client, not the ladder** — the pre-registered clause,
decided on the determinized-search precedent (a client default the
ladder cannot price): `lobby.rs`'s pilot is now `net_tail_guard_on()`;
`net-det1` and every ladder control stay flagless so recorded
references remain comparable. The regression evidence is the 2026-08-30
game 5 (replay-1788118995-5, turns 18/20): a flying 5/5 the human could
not block, held at 3–8 life for two turns of a lost race, everything
untapped. The threshold stays anchored to the calibrate band —
pre-registered as not-a-knob.

The mid-band leaf bias stays head_leaf's job (this is a consumption
guard, not a model fix), and the over-attack half stays with the
open-mana sim lead.

## Round 55 — the attack chain: ADOPTED (2026-09-04)

The attack search's menu could only *drop* attackers: greedy, nobody, and
greedy-minus-one holdbacks (`attack_candidates_for_mcts`). A declaration
smaller than greedy-minus-one, or one carrying a creature the greedy
filters refused, was unreachable at any valuation — the missing-candidate
shape behind gang blocks (+1.3) and chump blocks (+0.9), one decision
earlier. `EvalWeights::attack_chain` (`attack_chain_candidate`, profiles
`atk-chain` on the gang base and `net-chain` on the net pilot) grows a
declaration from the repaired "nobody" one creature at a time. Each step
simulates "the set so far plus one more eligible creature" for every
remaining creature through the existing one-turn-cycle attack sim, with
"finalize" (the set so far, at its known score) as candidate 0 so a tie
stops the chain; a strict improvement is kept and the chain stops at six
additions. The pool is every creature `may_declare_attacker` accepts, not
the greedy set, so a greedy-refused body is on offer and priced by the sim
rather than by the rule that refused it. The finished set then joins the
menu for the picker's ONE argmax, where greedy holds index 0 and every
tie — so the chain only changes a declaration by strictly out-scoring the
whole menu on the same sim, and forward growth's blind spot (two attackers
that only pay together are each bad alone, so the chain stops at nobody)
is covered by greedy's alpha strike still being on the menu. Pinned by
`attack_chain_stops_at_nobody_where_only_the_pair_is_lethal`; legality by
`attack_chain_declaration_is_legal_and_keeps_the_obliged_attacker` (CR
508.1d repair at every step, factored into `repair_attack_subsets`).

**Incidence** (`.ladder/r55_census.txt`, `CRAB_ATTACK_CENSUS=1`, 600
sealed games, chain on seat A only): 6 334 searched declarations across
both seats, 3.79 menu candidates per search; the chain proposed a set the
menu lacked in 616 (9.7 % of all searched, so ~19 % of the chain seat's)
and that set won the argmax in 377 (6.0 % / ~12 %). Not rare-class: every
searched declaration runs it.

**The gate** (`.ladder/run_r55_atkchain.sh`, pre-registered, four seeds on
the gang leg per the r51 budget rule, 1 000 games × 12 sealed decks per
cell, paired):

| leg | seed | A win % | interval | pairs split | rho |
|---|---|---|---|---|---|
| `atk-chain` vs `gang` | 43 | **53.1** | [52.7, 53.5] | 5 373 / 6 000 | −0.80 |
| | 97 | **51.7** | [51.4, 52.0] | 5 579 | −0.86 |
| | 151 | **52.1** | [51.7, 52.4] | | |
| | 199 | **52.3** | [51.9, 52.7] | | |
| `net-chain` vs `net-det1` | 43 | **51.2** | [50.8, 51.6] | 5 456 | −0.82 |
| | 97 | **51.0** | [50.7, 51.4] | | |

Pooled **+2.3 on the gang base, +1.1 under the net** — every one of six
cells clears 50.7 at its low end, and the gang leg is the largest
menu-hole reading on record. The mirrors break more than dmgorder's (rho
−0.80 vs −0.94, 498 A-sweeps vs 129 B-sweeps on seed 43) because the flag
fires every combat, which is also why the cells are ±0.33–0.40 rather than
±0.22. The net leg reading half the gang leg is the r43/r46 pattern
(search-level changes read smaller under the net) and replicates.

**Cost.** Sealed mirror, 12 000 games on 23 threads: `gang` vs `gang`
4.7 s, `atk-chain` vs `atk-chain` 6.6 s — **+40 % wall-clock** with the
chain on both seats; the `--bench` profile (`gang` = `block_gang_search`)
is untouched, so PERF's committed baseline stands. A self-play actor on
the default will generate ~30 % fewer games per hour; that is the price
of the lever and belongs in the next run's `--games` sizing.

**Adopted**: `EvalWeights::default()` carries `attack_chain: 6` (r52
style — measured on the `gang` base, stacked onto the default); the net
ladder references (`net`, `net-det1`, `net-guard`) stay flagless so every
recorded net number keeps its control, and the lobby pilot composes
`EvalWeights::client_pilot()` = det1 + tail guard (r54) + chain. The
client crate's local bot (`crabomination_client/src/menu.rs`) still
constructs `net_eval_det1()` directly — it did not take r54 either and
does not build in this container; point it at `client_pilot()` when it
is next built. Golden digest seed 3 re-blessed (same winner, same 21
turns, 484 → 483 actions — one declaration differs); the full committed
trace and the other four seeds are untouched.

**Caveats, recorded.** (1) The sim's standing blindness — it casts
nothing for the defender — is unchanged, and the chain gives that bias
more room: a greedy-refused attacker priced as free by a sim that ignores
the defender's open mana can now be declared. The ladder says the net
effect is strongly positive against both pilots, but the over-attack half
of the replay diagnostic (2026-08-31) should be re-read on the next batch
of client replays with the chain live. (2) Both legs measure against the
same heuristic blocking; a win that partly exploits `gang`'s block picker
is still a win against the only opponent the program has, but the human
replays are the check. (3) Actor-side sampling (`set_action_sampling`)
routes through `choose_scored`, which the chain's finalize/add step also
uses — so `--sample-temp` now samples at every micro-step, not only at
the final argmax. Deliberate (it is the micro-step exploration the chain
was proposed with), but a re-run of the round-23 sampling arm on the new
default is a different experiment from the recorded one.

**Next in this shape, in order.** (1) The block chain: the block space is
(blocker, attacker) pairs, and a chain of "assign this blocker to that
attacker" reaches gang, double and chump blocks with one mechanism where
today three hand-written generators do; gate as `blk-chain` vs the new
default. (2) A backward chain from greedy (beyond minus-one) for the
boards where the answer is "hold two". (3) The r23 sampling re-run with
the chain as the exploration mechanism — the training-side half of the
proposal, unmeasured here. (4) PERF: where the +40 % goes (candidates
per search under the chain, and whether the finalize-score reuse is
firing); the attack search was already ~60 % of a cube game.

## Round 56 — the block chain and the wide attack chain: BOTH ADOPTED (2026-09-05)

The two chains round 55 left on the table, measured the same day on the
same protocol (`.ladder/run_r56_chains.sh`, pre-registered; base `dflt55`
= the default as it stood after round 55, frozen so adoption does not
consume its own control).

**The block chain** (`EvalWeights::block_chain`, `block_chain_candidate`,
profiles `blk-chain` / `net-bchain`) grows a block plan from "no blocks"
one move at a time: per step, one (blocker, attacker) pair for every free
blocker and every attacker it may legally block (the engine's
`blocker_can_block_attacker_pair`, resolved once), plus a per-attacker
*gang move* — the cheapest free blockers that together kill it, the
pair-level step single growth cannot take because each gang member is a
chump alone — each priced by the block sim, "finalize" as candidate 0,
strict improvement kept, four moves max. The finished plan joins the block
menu's argmax, greedy keeping index 0 and every tie.

The hole it closes is bigger than expected. `block_candidates_for_mcts`
returned bare "no blocks" whenever greedy found no profitable single block
and no chump was warranted, **and never generated gang candidates there**
— the gang generator only ever ran on a menu greedy had already seeded.
So the double block that trades a bear for a 3/3 was unreachable on
exactly the board where it is the only good play, and the r43 gang
adoption (+1.3) was measured with that hole in place. Pinned by
`block_chain_finds_the_gang_block_the_bare_menu_cannot` (menu = `[[]]`,
chain = both bears on the giant) and
`block_chain_reaches_a_double_gang_the_menu_cannot` (two giants into four
bears: the generator emits one gang per candidate, never both).

| leg | seed | A win % | interval | rho |
|---|---|---|---|---|
| `blk-chain` vs `dflt55` | 43 | **56.8** | [56.3, 57.3] | −0.69 |
| | 97 | **55.0** | [54.6, 55.5] | −0.74 |
| | 151 | **55.6** | [55.1, 56.0] | |
| | 199 | **55.7** | [55.3, 56.2] | |
| `net-bchain` vs `net-chain` | 43 | **57.7** | [57.2, 58.2] | −0.63 |
| | 97 | **55.7** | [55.2, 56.2] | |

**Pooled +5.8 on the heuristic base, +6.7 under the net** — two and a
half times the attack chain and the largest reading in the program's
record. Cross-checked because of the size: 59.0 vs `gang`, 60.3 vs
`atk-sim` (seed 43), and **54.5 / 54.5 on `--decks cube`** (seeds 43/97,
±0.9), so it is not a sealed-pool or a same-opponent artefact. Census
(`.ladder/r56_census_blk-chain.txt`): the chain proposed a plan the menu
lacked in 17.2 % of block searches and that plan won in 16.3 % — 95 % of
its proposals win, which is what "the menu had nothing" looks like.

**The wide attack chain** (`EvalWeights::attack_chain_wide`, profile
`atk-chain-wide` / `net-chain-wide`): the r55 chain never ran when greedy
declared *nobody* (a one-candidate menu returned before any sim), and
forward growth is blind to the overload (two attackers into one blocker
connect where each alone is blocked and traded, so the first step ties).
On, the chain runs from an empty greedy and its first step offers every
pair. Pinned by
`attack_chain_wide_overloads_the_lone_blocker_greedy_holds_against`.
Census: 30 % of searched declarations are now empty-greedy boards
(2 664 of 8 971) — the r55 chain was skipping a third of its decisions.

| leg | cells | pooled |
|---|---|---|
| `atk-chain-wide` vs `dflt55` (43/97/151/199) | 50.4 [50.1, 50.7] / 50.7 [50.4, 51.0] / 50.4 [50.1, 50.6] / 50.4 [50.1, 50.7] | **+0.48** |
| `net-chain-wide` vs `net-chain` (43/97) | 50.2 [49.9, 50.6] / 50.7 [50.4, 51.0] | +0.45 |

Small and replicated: every heuristic cell's interval clears 50 (the r50
rule), the net leg straddles on one seed. **Adopted on the default, not
on the client pilot** (pre-registered: the net leg decides the client).
Why so much smaller than the block chain when it fires on a third of
declarations: on those boards greedy's refusal is usually right, and the
sim agrees — the pair move's wins are the overload boards, which are a
sliver.

**Adopted**: `EvalWeights::default()` = `round55_default()` +
`block_chain: 4` + `attack_chain_wide: true`; `client_pilot()` = det1 +
tail guard + attack chain + block chain. Net ladder references stay
flagless. The r55 default reads 53.2 vs `gang` (seed 43); the new default
has not been re-read against the flagless controls yet — **every
reference before 2026-09-05 predates both chains.** Golden traces
re-blessed: the full committed trace moved at one line (a turn-7 block
declaration); digest seeds 3 and 4 flip winner and run 21 → 32 and 13 →
24 turns — the aggro deck's early swings are blocked instead of raced.

**Caveats.** (1) The block sim's horizon ends at end of combat; a block
plan is priced on who dies and life saved, never on the crack-back. Blocks
carry no tempo cost, so that is mostly right, but a gang that leaves the
best blocker dead before the opponent's second wave is invisible. (2) The
block chain runs on every trivial block menu at (free blockers ×
attackers) sims a step — the cost line below. (3) The size of the win
says the greedy trade table was the weakest hand-written policy in the
bot; the same shape (a sim-priced chain on a bare menu) should be looked
for in every other picker that returns a bare default: combat tricks,
defensive removal, the mulligan.

**Cost** (uncontended, sealed mirror, 12 000 games on 23 threads, the
same binary): `gang` 4.6 s, `dflt55` 6.8 s, `dflt55` + block chain 7.9 s,
`dflt55` + wide chain 8.7 s, the new default 9.8 s. So the block chain is
+16 % for +5.8 and the wide chain +28 % for +0.5 — the wide chain's pair
move (`C(n, 2)` full-turn-cycle sims at every chain's first step, not only
the empty-greedy ones it was built for) is the expensive half, and the
first perf candidate: pairs only when the chain starts from nobody, gated
for no loss. Census under the new default (both seats): the attack chain
runs 3.25 sims per searched declaration and 45 % of searched declarations
are now empty-greedy boards; the block chain runs 2.88 sims per block
search, proposes a new plan in 24 % and wins in 23 %, and reuses the
menu's start score 65 % of the time (why not ~100 % on a bare menu is the
second candidate). The whole default is now 2.1× `gang`'s wall-clock; a
self-play actor on it generates roughly half the games per hour the
round-54 default did, which is the price of the two largest levers the
program has found and belongs in every `--games` sizing from here.

**Next in this shape.** (1) The r57 sampling re-run (`.ladder/run_r57_sampling.sh`,
launched this session) — the training-side half. (2) Re-read the standing
references (champion + mcts-net-deep vs `atk-sim` / `gang`) on the new
default. (3) A backward chain from greedy beyond minus-one, and the
block-side release chain. (4) The client replays with both chains live.

## Round 57 — softmax sampling re-run with micro-step exploration: NULL again (2026-09-05)

The training-side half of the round-55 proposal. Round 23 measured
actor-side softmax sampling (`--sample-temp 120 --sample-turns 6`) as a
null (51.7 / 53.7 vs the control's 51.8 / 54.4) when the flag only ever
perturbed a decision's final argmax. Since rounds 55–56 the default pilot
grows its attack declaration and its block plan through chains whose
finalize-or-add step goes through the same `choose_scored`, so the same
flag now samples at every micro-step of both declarations — the coverage
argument round 23 lacked (partial declarations the argmax pilot never
visits get rows; a pair that only pays together can be found by the
outcome label). `.ladder/run_r57_sampling.sh`, pre-registered.

**Recipe**: round 23's, verbatim, on the round-56 default (heuristic
actors, no net pilot): `--attn --lambda 0.7 --games 250000 --steps 200000
--window 500000 --lr 1e-4 --lr-cosine 60000 --relabel-mode new
--stop-after-stale 12`; the arm adds `--sample-temp 120 --sample-turns 6`.
Four training seeds, paired within seed. **Regime**: the verbatim flags
generate at 1 245 games/s on this machine (7× round 23's 182, first
attempt killed at 86 k games), so actors were throttled to 3 and the seed
halves ran as two concurrent drivers, each with its own GPU learner:
217–240 games/s, 23–25 k learner steps, 5.9–6.5 M of ~24 M rows consumed
(a 0.25 pass; round 23's was 0.58 at 55 k steps — two learners sharing
one 4090 halve each one's rate). Both arms share it exactly; the
comparison is within seed. One asymmetry recorded: driver B's first two
gate cells ran on 23 threads while driver A was still training seed
151's control; B was stopped and its gates re-run after all training.

**Gate**: each net pilots `net-bchain` (det1 + attack chain + block chain,
the scored pilot the client's shape reduces to) vs `dflt` (the round-56
default), ladder seeds 43/97, 1 000 games × 12 sealed decks, paired.

| training seed | ctrl AUC | samp AUC | ctrl (g43 / g97) | samp (g43 / g97) | samp − ctrl |
|---|---|---|---|---|---|
| 43 | 0.8209 | 0.8178 | 50.0 / 50.2 | 50.6 / 49.8 | **+0.10** |
| 97 | 0.8116 | 0.8215 | 49.6 / 49.9 | 50.0 / 50.4 | **+0.45** |
| 151 | 0.7758 | 0.7996 | 50.1 / 50.0 | 49.6 / 49.3 | **−0.60** |
| 199 | 0.8434 | 0.8327 | 49.6 / 49.9 | 49.5 / 49.8 | **−0.10** |

**Pooled −0.04 over four seeds (sd 0.38), every cell ±0.6.** The
pre-registered reading: null again — sampling at the chains' micro-steps
is not the missing exploration, and the recorded remaining arm is
sampling under a NET-piloted generator, where the argmax fixed point is
the net's own. The AUC column is not comparable across arms (round 23's
caveat: the sampled arm's validation games contain exploration moves)
and swings 0.78–0.84 across seeds within an arm, the [[auc-seed-variance]]
band.

**Two things the run says beyond its own question.** (1) A net trained
on the round-56 default's self-play pilots the round-56 heuristic to
**exactly 50** as a scored pilot with both chains (eight control cells
49.6–50.2) — the standing "better predictor, not a better pilot" result,
now on a much stronger heuristic; the chains raised the floor the net has
to beat by ~8 points in two days without moving what the net adds on
top. The mcts-net-deep system reference (55.2 / 53.65) has still not been
re-read on this default. (2) Eight 250 k-game runs cost 2.5 hours
wall-clock on this machine with the machine mostly idle; the learner, not
generation, is the budget here, which inverts ML_PIPELINE's "generation
is the bottleneck by design" and is worth knowing before the next
training round is sized.

## Round 58 — the wide chain's pair move restricted: NO LOSS at −14.9 % wall clock, ADOPTED (2026-09-05)

The first perf candidate round 56 filed: the wide attack chain's pair
move priced `C(n, 2)` full-turn-cycle sims at *every* chain's first
step, though it was built for the empty-greedy overload, and cost +28 %
of the default's wall clock for its +0.5. Two restrictions as flags,
each keeping the overload board `attack_chain_wide_overloads_the_lone_
blocker_greedy_holds_against` pins (pre-registered in
`.ladder/run_r58_pairs.sh`; base `dflt56` = `round56_default()`,
frozen):

* `attack_pairs_empty_only` (`pairs-empty`): pairs only when greedy
  declared nobody.
* `attack_pairs_lazy` (`pairs-lazy`): singles first, pairs only when
  every single tied "finalize" — the overload's own definition; a single
  that wins grows the set and the second attacker is a single addition
  at the next step.
* both (`pairs-both`).

**Cost first** (step 0, one binary, sealed 200-game mirrors, arms
alternated, median of 5 per-rep ratios against `dflt56`): `pairs-empty`
0.872, `pairs-lazy` 0.893, `pairs-both` **0.851**. Chain sims per
searched declaration 3.20 → 2.19 / 2.48 / 2.18.

| leg (vs `dflt56`, sealed, 12 000 games a cell) | 43 | 97 | 151 | 199 | pooled |
|---|---|---|---|---|---|
| `pairs-empty` | 50.0 [49.9, 50.2] | 50.1 [49.9, 50.2] | 50.0 [49.9, 50.2] | 50.2 [50.1, 50.3] | +0.08 |
| `pairs-lazy` | 50.0 [50.0, 50.1] | 50.0 [49.9, 50.1] | 50.0 [49.9, 50.1] | 50.0 [49.9, 50.1] | 0.00 |
| `pairs-both` | 50.1 [49.9, 50.2] | 50.1 [49.9, 50.2] | 50.0 [49.9, 50.2] | 50.2 [50.1, 50.3] | **+0.10** |

Every cell's interval touches 50 and none sits below it: the
pre-registered "no loss — adopt the cheapest" reading, and `pairs-both`
is the cheapest. **Adopted**: `EvalWeights::default()` =
`round56_default()` + both flags. The client pilot does not carry the
wide chain and is unchanged; the net references are flagless.

What the cells say beyond the verdict: `pairs-lazy` is a strength no-op
to ±0.1 — when a single wins the first step, the pair it might have
preferred is reached by growth anyway — and `pairs-empty` leans
positive, i.e. the pairs beside a non-empty greedy declaration were, if
anything, buying an over-attack. The whole wide chain is therefore
worth its round-56 +0.5 at roughly half its round-56 cost.

**Cost, updated.** Sealed mirror per 12 000 games on this container's 4
cores (the r56 numbers were on 23 threads and are not comparable): the
new default runs at 0.851 of the round-56 default; against `gang` the
round-56 default was 2.1×, so the adopted default is ~1.8× `gang`.
`(-254)` (PERF) takes another 0.2 % off the same path.

**Next in this shape.** The chains' remaining cost is the attack chain
proper (3.17 sims a searched declaration, 45 % of searches on
empty-greedy boards where greedy's refusal is usually right): a cheap
pre-filter for the empty-greedy chain — skip it when no remaining
creature can connect or trade (the sim says "tie" on those) — is the next
candidate, gated the same way; then `attack_skip_open` re-read on this
default.

## Round 59 — the empty-greedy blocker gate: strength-neutral, cost-flat, NOT adopted; the census found the real waste (2026-09-05)

The candidate round 58 left at the top: the wide chain runs from an
empty greedy on 44 % of searched declarations and greedy's refusal is
usually right there. `attack_empty_gate` (profile `empty-gate`,
`.ladder/run_r59_emptygate.sh`, pre-registered) skips that chain when
the defender's untapped creatures that may block are at least as many as
this seat's untapped creatures — one blocker per attacker, nothing to
overload.

**Step 0 refuted it before the cells ran, twice.** Census on the r58
default (sealed, 2 400 games): 22 192 empty-greedy searches, the chain
proposed a set the menu lacked on 886 of them and won all 886; the gate
covers 3 316 of the searches — and **752 of the 886 wins**. It skips 15 %
of the chain's work and 85 % of what the chain finds. Paired wall clock
(5 reps, 200 × 12): **1.000 median**. The cells then read

| `empty-gate` vs `dflt` | 43 | 97 | 151 | 199 | pooled |
|---|---|---|---|---|---|
| | 49.9 [49.7, 50.2] | 50.0 [49.8, 50.3] | 50.2 [50.0, 50.5] | 50.4 [50.1, 50.6] | +0.12 |

— strength-neutral (so "the chain's argmax wins" on those boards are
not ladder wins; the sim's preference there is noise-level), and a
throughput device that buys no throughput is not adopted. The flag
stays off as the `empty-gate` control.

**What the census actually said.** `start reused` counts chains that
reached their start, and it was 32 014 against 50 768 searches: the
other 18 754 searches never built a chain at all, because the chain's
pool — every creature `may_declare_attacker` accepts — was **empty**.
An empty greedy under the wide flag is a one-candidate menu that the
r56 code sends to the sim before the chain says it has nothing to add,
and on 84 % of those boards every creature is summoning-sick, tapped or
barred. Each paid one full turn-cycle sim of "nobody" to feed an argmax
of one. Fixed as PERF `(-255)`: the pool is resolved ahead of the sims
(`attack_chain_pool`) and the empty case returns the menu as it stands.
Sealed, 1 200 games: searched declarations 25 384 → 16 060, outcomes
identical; callgrind −2.17 % Ir on the default, paired wall clock
0.984 median over 7 reps. The empty-greedy chain that remains runs on
1 876 searches per 1 200 games and wins 452 of them — the most
productive search in the bot per sim spent, which is the opposite of
what this round set out to prune.

**Rule that fell out.** Read a chain's `start reused` against its
`searched` before gating what the chain *decides*: the gap is the
searches that never reached a decision, and that is where the sims were
going.

## Round 60 — the open-board shortcut re-read on the round-58 default: NO LOSS at −4.1 % wall clock, ADOPTED on the default (2026-09-05)

`attack_skip_open` (`board_open_for_attack`: no opposing creature,
planeswalker or battle, so the greedy declaration is taken without a
sim) was read at `e725e5c2` on the `gang` base: −1.3 / −1.8 % wall clock
and −0.1 pt on 96 000 sealed games, filed as the opt-in `atk-open`.
Re-read here as `dflt-open` (the round-58 default + the flag,
`.ladder/run_r60_open.sh`, pre-registered) because the search it skips
has since grown to 3.3 menu sims plus 3.4 chain sims a declaration and
the creatureless board is 11.4 % of searched declarations (1 824 of
16 060 at 100 × 12, greedy winning 1 814 of them).

**Cost** (step 0, one binary, sealed 200 × 12 mirrors, 5 paired reps):
`dflt-open` **0.959** of the r58 default's wall clock.

| `dflt-open` vs `dflt58` | 43 | 97 | 151 | 199 | pooled |
|---|---|---|---|---|---|
| | 50.0 [49.9, 50.0] | 50.0 [49.9, 50.0] | 50.0 [49.9, 50.1] | 49.9 [49.8, 50.0] | −0.02 |

Every cell within ±0.08 and touching 50: the shortcut changes 10
declarations in 1 824, and the ladder cannot tell them from noise.
Cross-checked on the pools that carry attack-trigger creatures, because
the golden-trace decks (four Goblin Guides) moved three of five seeds:
`--decks cube` 50.2 [50.0, 50.3] / 50.0 [49.9, 50.2] (seeds 43/97,
3 200 games each) and `--decks fixed` 50.2 [49.8, 50.7] / 50.1
[49.6, 50.6] (1 600 each) — no loss there either.
**Adopted on the default only**: `EvalWeights::default()` =
`round58_default()` + `attack_skip_open`. The client pilot is built on
`block_gang_search` and keeps the sim's hold-backs (the Goblin Guide
case the r50-era reading found); the net references are flagless.

**Why the r50-era −0.1 did not reproduce.** That reading was on a
search a third the size, so the sim's rare correct hold-back weighed
more against a smaller saving; here the saving is 4 % and the cells are
the tightest in the program's record (the paired design's mirror
property again — 11 990 of 12 000 pairs play identically).

**Cost, cumulative for the run.** Against the round-56 default the
adopted default now runs at 0.851 (r58) × 0.984 (`(-255)`) × 0.959
(r60) ≈ **0.80** of its sealed wall clock, i.e. ~1.7× `gang` where it
was 2.1×, with both chains intact and no strength given up on any gate.
