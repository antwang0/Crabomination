# The ML pipeline

How self-play data is generated, what the net sees, what the net is, and
how it trains. Snapshot of the system as of round 12 (2026-08-08).

This describes the *machinery*. The measurement history — what was tried,
what passed a gate, what was refuted — lives in `FEATURE_ROADMAP.md`, and
that is the file to read before proposing a change. Numbers quoted here
are the ones that justify a design decision, not a full results log.

Scope throughout is **SOS sealed**: one set's worth of card names (~164
including basics) keeps the embedding table small and well-fed.

---

## 1. Data generation — actors

`crabomination_ml/src/bin/selfplay_train.rs` runs one process: N actor
threads (cores − 2, 32 MB stacks) plus a learner thread, which is the main
thread.

Each actor claims a game index and derives every seed from
`args.seed ^ n·φ + k`, so a run is reproducible. Per game:

1. Roll two SOS sealed pools (`sealed_pool`).
2. Build a 40-card deck from each — the heuristic builder by default, or
   best-of-32 judged by the deck net under `--use-deck-best`. Same
   candidate generator either way, so only the judge differs.
3. Play it out (`play_recorded_game`, `crabomination/src/selfplay.rs:151`)
   with identical pilots on both seats: heuristic `EvalWeights::default()`,
   or `EvalWeights::net_eval()` when `--use-best` has loaded weights.

**Snapshots** are taken at each new turn plus at `PostCombatMain` and
`End` — about three per turn per seat — with consecutive duplicates
dropped. Both seats are snapshotted at the same instants, which is what
makes ply strata directly comparable: every stratum holds exactly one win
per loss, so the base rate is 0.5 throughout.

**Opening exploration.** The first `0..=EXPLORE_PLIES` (12) decisions of
each game are played by a uniform-random bot. Both seats otherwise run the
same deterministic policy, so without this the net only ever sees the
narrow band of positions that policy reaches — the classic self-play
distribution collapse. It is confined to the opening so the *outcome*
still reflects competent play and the win labels stay meaningful.

**Labels**, stamped once the game decides:

| field | meaning |
|---|---|
| `win` | 1.0 if the encoded seat won, 0.0 if not |
| `life_diff` | final (self − opp) life, clamped ±20, scaled 1/20 |
| `game_len` | turns the game still had to run, scaled 1/15 |
| `traj` | `(seed << 1) \| seat` |
| `ply` | position within the trajectory, ascending |
| `aux` | next-snapshot life/power/creature deltas + opp hand (round 12) |

One trajectory per **(game, seat)**, not per game: the two seats see
different information and end on opposite results, so a row must only ever
bootstrap through its own successors.

Every decided game also emits two `DeckRow`s for free — the two decklists,
labelled by who won. That is the deck net's entire training stream.

---

## 2. State representation

`crabomination/src/server/encode.rs`. Seat-relative and strictly
observable: the opponent's hand and either library's *contents* never
enter the feature vector, only their sizes. A net trained on these rows
cannot learn to peek.

### Eight zone groups (`NUM_GROUPS = 8`)

`G_BF_SELF`, `G_BF_OPP`, `G_HAND_SELF`, `G_GY_SELF`, `G_GY_OPP`,
`G_LIB_SELF`, `G_STACK_SELF`, `G_STACK_OPP`.

The stack groups are round 12's addition: one object per stack item (a
spell's own card, or a trigger's battlefield source), split by controller,
with depth-from-top in feat 36. Before them the stack was a single count —
"there is a spell on the stack" was representable, "it is their removal
aimed at my best creature" was not.

The library group is round 11's addition. It is deduplicated by card name
— one object per distinct name with its remaining count in feat 27 — and
emitted in **vocabulary-index order**, so the actual shuffle can never
reach the net whatever the architecture does downstream. Before it, the
library was a single scalar (`len / 40`), so "22 cards left, three of them
removal and one a bomb" and "22 lands" encoded identically. It is also the
one zone where the bag-of-cards prior is unambiguously correct: a library
really is an unordered set.

### Per object — 37 features

A vocabulary index into a card-name embedding table (index 0 = unknown, so
tokens and off-set cards are represented by their features alone), plus:

- 0–5: mana value, creature/land/planeswalker flags, printed P/T
- 6–11 (battlefield only): tapped, summoning sick, loyalty counters,
  prepared, attacking, is-token — and feats 4/5 are overwritten with
  *effective* P/T minus marked damage
- 12–19: evasion/combat keywords — flying, reach, menace, deathtouch,
  lifelink, trample, first-or-double strike, vigilance. Without these the
  pooled encoder saw a Serra Angel and a Hill Giant as the same 4-mana
  4/4-ish body.
- 20–26: the castability block — printed colour pips, castable now,
  castable next turn
- 27: library multiplicity
- 28–36: the round-12 relation block — is_blocking / is_blocked (from
  `block_map`), is_attached, has_own_attachment / has_opp_attachment (an
  own aura is a buff, an opposing one is a Pacifism — previously those
  two creatures encoded identically), targeted_by_stack, non-loyalty
  counter count, printed aura/equipment flag, and stack depth. Pooling
  cannot carry an *edge* between objects, but a flag summarising the edge
  from each endpoint survives it, and attention can match flagged
  endpoints across groups.

### 36 globals

Life and hand/library/graveyard sizes for both seats, turn number,
active-player flag, a one-hot phase bucket, untapped lands, creature
counts and total power for both seats, stack and attacker counts, and
(24–35) untapped mana **sources by colour** for both seats. The
opponent's half of that last block is public information and is what makes
"they have two untapped blue" — the shape of every instant-speed decision
— representable at all.

### Castability

`affordable()` is **exact** for the model it assumes (one mana per
source), by Hall's condition over the 32 colour subsets: a multiset of
coloured pips has a saturating assignment iff for every subset of colours,
the pips wanting those colours are no more numerous than the sources able
to make one of them. The generic remainder is then satisfied iff the total
source count covers the whole mana value.

It deliberately does *not* model sources that tap for two, cost reduction,
alternative costs, {X}, or hybrid pips. This is a "is this card roughly
live" feature, not a legality check — the real payment path is
`GameState::auto_tap_for_cost`.

`affordable_with_extra` adds one source that makes any colour: the land
drop the seat has not taken yet. Optimistic about colour on purpose.

### Ablation control

`set_encode_ablation(library, castability, relations)` **zeroes** any of
the three blocks rather than removing it, so feature counts and
`SHARD_VERSION` are unchanged and an ablated run produces shards
interchangeable with a full run. It is
process-global (the encoder is called from deep inside the search) and
therefore the tests that encode take a mutex.

This exists because the library group, the castability block, and a
vocabulary change all landed together, and "the new encoder scores worse"
had three candidate causes with no way to separate them.

---

## 3. Model

### Play net

Two implementations, held to each other by
`exported_weights_match_engine_inference` (<1e-4 on random states):

- `crabomination_nn/src/lib.rs` — hand-rolled, dependency-free, wasm-safe,
  built at `opt-level = 3` even in debug. This is what the bot runs inside
  its simulation loops.
- `crabomination_ml/src/lib.rs` — the candle mirror that trains.

Tensor names are the contract: `emb.weight`, `obj.{weight,bias}`,
`trunk1.*`, `trunk2.*`, `head_win.*`, the training-only `head_life.*` /
`head_len.*`, and `attn.*`.

Forward pass:

```
per object:  relu(W_obj · [emb(card) ⊕ 28 feats])        → 64   (shared weights)
optional:    4-head self-attention over ALL objects, + residual
pool:        mean and max within each of the 6 groups
trunk:       [8 · 2 · 64  ⊕  36 globals] = 1060 → 512 → 256
heads:       sigmoid win  (+ life-diff, game-length, and optional
             short-horizon aux in training)
```

Standard sizes: `EMB_DIM 32`, `OBJ_HIDDEN 64`, `TRUNK_H1 512`,
`TRUNK_H2 256`, overridable per run via `--emb-dim/--obj-hidden/--h1/--h2`
— the engine reads sizes from the tensor shapes, so width is a flag, not a
format change. The vocabulary is the exception and must match the
encoder's.

Round 12 also added an opt-in aux head (`--aux`, `head_aux.*`): four
short-horizon targets — next-snapshot life/power/creature deltas and
opponent hand size — labelled from the recorded trajectory. Dense and one
hop out where `win` is sparse and twenty turns away; training-only, the
engine ignores the tensors.

**The attention layer** is off by default (`--attn` opts in); the pooled
net is the control. Its reason to exist: mean/max pooling is permutation
invariant *per group*, so the trunk only ever learns "how much is in this
zone" and "what is the biggest thing in it". It cannot represent "my flier
gets through because their board has no flier and no reach" — that needs
my battlefield compared element-wise against theirs, and pooling has
discarded it before the trunk runs. Attention is the only place in the
network where an object on my battlefield and one on the opponent's are in
the same tensor. A learned per-group tag is added to the input so a query
can tell whose object it is looking at; the residual is against the
*pre-attention* state, so the tag never leaks into the pooled
representation.

The two attention implementations are structurally different — candle
attends over a padded `[B,N,N]` batch with a mask, the engine attends over
exactly the real objects with no padding — and agree only because the mask
bias is `-1e9` rather than `-inf`, which underflows to precisely zero
weight. `-inf` would make an all-padding row softmax to NaN.

A file carrying *some* attention tensors is rejected rather than loaded as
the pooled net: that would run the wrong architecture on the right weights
and silently produce nonsense.

### Deck net

Much smaller, and the only learned component that has cleared a house
gate — four times, at 60–62 % against the static build score with
identical pilots and candidate sets.

```
embed the 40-card multiset → pool (sum ×0.1, mean, max) ⊕ 16 deck feats
→ 128 → 64 → sigmoid
```

Deck features: seven curve buckets, land count, creature count, five
colour-pip counts, colour count, average mana value. Separate safetensors
file, so the shared tensor names don't collide.

---

## 4. Training loop

The learner samples uniformly from a `SampleWindow` (250k rows, FIFO
eviction) and runs AdamW at lr 1e-3, batch 256.

```
loss = MSE(win) + 0.25 · (MSE(life_diff) + MSE(game_len))
```

The auxiliary heads exist for credit assignment, not for play: a bare win
bit cannot say *which part* of a position lost the game (the KataGo
ownership-target lesson at MTG scale). Loss is tracked decomposed, because
a single number hid a real regime change once already.

### Reuse throttle

The learner may consume at most `reuse × rows_pushed` samples (default
6×); otherwise it sleeps 200 ms. Generation is the bottleneck by design,
and the learner spends most of its wall clock waiting — that is the cap
doing its job.

Once the actors finish, the global budget stops meaning "≤6 visits per
row": every further sample lands on the final window, concentrating
`budget / window_len` visits there (measured ~14× on one run). So the tail
gets an explicit budget — half the nominal reuse on the window it has —
and then the loop stops. Tail over-reuse was priced once: loss EMA fell
0.30 → 0.14 during it with **no strength change**. Pure memorisation.

### TD(λ)

`--lambda`, default 1.0 = pure Monte Carlo (the row's own game result),
which every pre-λ gate round trained on and which stays bit-reachable as a
control. Below 1.0, `SampleWindow::relabel_lambda` walks each trajectory
backwards:

```
G_T = z                                  (the actual result)
G_t = (1 − λ)·V(s_{t+1}) + λ·G_{t+1}
```

λ = 0 is one-step TD. Between them the target trades the variance of a
twenty-turn outcome for the bias of the net's own current estimate.
Targets are functions of the net, so they go stale as it trains and are
recomputed every `--relabel-every` steps (default 200) — one forward pass
over the window. Trajectories never cross: a trajectory whose tail has
been evicted simply bootstraps from its last surviving row.

### Holdout

`--holdout`, default 5 %. Split **by trajectory, not by row** — consecutive
snapshots of one game are near-duplicates, so a row-level split leaks the
answer across it and validation comes back reassuringly good. Membership
is a hash of `traj`, so actors decide independently and a game always
lands on the same side. Capped at 20k rows.

Scored at every checkpoint for MSE (directly comparable to the training
win loss — the gap between them *is* the overfit), log-loss, and AUC.

This exists because its absence cost six gate rounds: `stats.jsonl`
reported training loss only, and at λ=1 the net hit 0.017 MSE on the
window while its out-of-sample log-loss was 1.12 — *worse than predicting
0.5 every time*. Nothing in the loop could see it.

### Checkpointing

Every `--checkpoint-every` steps (default 2,000), by atomic rename:

- `latest.safetensors`, `deck-latest.safetensors` — resume points
- `best.safetensors` — republished on any holdout-AUC improvement
- one JSON line to `stats.jsonl`

`best` matters more than it sounds. `latest` is whatever the run happened
to end on, and a run that overfits ends on its *worst* net: in the round-11
pair, holdout AUC peaked around step 4–6k and then fell ~0.07 over the next
45k steps while training loss went to 0.001. Every gate and every
calibration before this was scored on a memorised checkpoint. Selecting
`best` instead of `latest` was worth **+0.073 AUC** (0.7071 → 0.7798) at
identical config, seed, and games.

The deck net rides along at a quarter cadence (its stream is 2 rows/game,
so training it every step would just churn the same rows), from a 200k-row
window, once it has 4,000 rows.

### Resume and seeding

A run with an existing `--out` resumes from `latest.safetensors` if the
shapes still match. `--seed-emb DECK.safetensors` initialises the play
net's embedding table from a trained deck net instead of from noise —
same vocabulary, same width, so the tensors are drop-in — and refuses on
any shape mismatch, since a moved vocabulary would silently scramble card
identity. Ignored on resume: embeddings already trained on real positions
should not be overwritten. Nothing is frozen; these are ordinary initial
values.

---

## 5. Consumption at play time

`crabomination/src/server/net_eval.rs` holds a global registry of loaded
nets keyed by a one-byte slot id carried on `EvalWeights`. `EvalWeights`
is `Copy` and threads through ~30 free evaluation functions, so handing
each an `Arc<PlayNet>` would mean rewriting every signature in the bot's
hot path. Slots also let two nets coexist in one process, which the
gatekeeper needs (candidate vs best). Threads cache the slot table and
refresh only when a generation counter moves, so evaluation inside
simulation loops never touches the `RwLock`.

`SLOT_BEST` is the promoted net; `SLOT_CANDIDATE` is the training loop's
un-promoted checkpoint. Loading checks the net's vocabulary against the
encoder's first.

Profiles: `EvalWeights::net_eval()` replaces the heuristic evaluation with
the net; `net_eval_blend()` keeps the heuristic and adds a bounded net
bias.

---

## 6. Diagnostics

All three live in `selfplay_train` and all three take `--use-best`.

**`--calibrate N`** — scores the net and the heuristic as *predictors of
the winner* on identical positions: log-loss, Brier, AUC, against the
constant-predictor floor. The heuristic is put on the same footing by
fitting a one-parameter logistic to its score by scan, so it is judged as
the best probability forecast that evaluation can support rather than
penalised for not being calibrated. Also prints the ply-stratified
breakdown and the output histogram.

The histogram is the saturation check: a sigmoid that piles up near 0 and
1 hands the search a flat landscape in which every candidate line scores
the same, which turns a better predictor into a worse player.

Ply strata exist because the two ends of a game are not the same problem.
Late positions are mostly already decided, and anything that can count
power and life gets them right; early positions are the contested ones and
the only ones the search can still change. A net that is better late and
worse early posts a better aggregate AUC *and plays worse*.

**`--pairwise N`** — local discrimination: can the evaluator order two
*adjacent* snapshots of the same game? AUC is a global ranking statistic,
and the search never asks the global question — it compares near-identical
boards differing by one attack or one block, dozens of times per decision,
and takes the argmax. An evaluator can be excellent at "who is winning"
and useless at "which of these two almost-identical lines is better", and
only the second is consumed inside a resolved simulation. Reports the
correct-rate among *separated* pairs, the tie rate, and mean separation on
each evaluator's own scale (never compared across the two — one is an
unbounded integer, the other a probability).

**`--gate-builder N`** — paired-pool race where net-judged and
static-judged best-of-32 builds come from the *same* candidate set and
race with identical pilots, so the result isolates the judge.
**`--gate-builder-v2 N`** does the same for the builder itself, no net
involved.

---

## 7. Where it stands

Round-11 ablation on `best.safetensors`, 500 fresh games each:

| encoder | AUC | log-loss | saturated |
|---|---|---|---|
| full | 0.7798 | 0.5575 | 3.3 % |
| no library | 0.7809 | 0.5720 | 11.6 % |
| no castability | 0.7748 | 0.5680 | 4.8 % |
| neither (old encoder, new vocab) | 0.7599 | 0.6033 | 12.8 % |

Both blocks stay. Castability buys ranking; the library buys calibration
and collapses saturation from 12.8 % to 3.3 %.

On `best`, the net beats the heuristic in **every** ply bucket, with the
margin peaking at ply 8–11 and decaying late — the opposite of the
late-game-only story the memorised checkpoint told.

The standing result: the play net is a strictly better predictor that
still loses gates as a pilot (42–45 % as a replacement, ~49 % blended).
The deck net is the component that works.
