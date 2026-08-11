# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.


Split for size (this file is the working handoff; the two archives below are
reference and want their own triage pass):

- `ENGINE_BACKLOG.md` — the long engine/rules/client backlogs and audits
  (follow-ups not yet done, suggested next-up tasks, the CR coverage audit,
  the decision-plumbing audit, client UX/visualization).
- `CARD_BACKLOG.md` — per-set and per-run card residuals: what each closed set
  still approximates, and the remaining gap lists.
- `PERF.md` — the perf record: baseline, log, profile of record, candidates.

## NEXT (handoff — rewrite each run, keep under 15 lines)

Branch `claude/modern_decks`. **Twenty-fifth pass, and it collided with a
concurrent session**: both pulled candidate (9) shape (a) from base
`10cb8fbf` and wrote functionally identical code. Theirs is kept
(`56f6623f`, -0.740 %); mine was dropped in the rebase. On top of it:
`5d4b5402` (the same shape somewhere else) + the stall instrumentation and
`STALE_ROUNDS`, **3,177,885,139 -> 3,159,019,265 Ir (-0.594 %)**. Bench
output identical, **18,612 tests green** (1 ignored, pre-existing),
`cargo clippy --workspace --all-targets` **clean**, golden traces
unchanged.

- **Next up — new candidate (10), the cheapest thing on the list.** Shape
  (a) is a *family*, not a damage-path fix: any `&self` function reading
  the layer system twice is one `with_frozen_layers` from paying, it
  cannot change behaviour (the closure cannot mutate), and
  `check_target_legality_with_source` was -0.516 % in ten minutes.
  **Enumerate rather than profile**, and read `--tree=caller` on
  `computed_permanent` at *the callers of the leaf helpers*, not the
  helpers — the two damage leaves still gathering take one gather each and
  have nothing to fold. Then (9)(b) (a behaviour change, golden traces
  decide) and `can_afford_in_state`'s five whole-board walks per call.
  Profile of record is **two rows stale**; retake before pulling anything
  under ~0.3 %.
- **START HERE: the cube pool is not deterministic on a fixed seed.** Three
  identical `--bench --threads 1 --decks cube --games 300 --seed 11` runs
  read **decisions 1,129,690 / 1,130,785 / 1,130,706** and determinism
  ok / FAIL 1 / FAIL 5. `--decks fixed` is clean (six runs, decisions
  193,232 identical), so it is not the harness, the loop, the threading or
  the deck construction — it is a *card*, or a rules path only cube cards
  reach, leaking `HashMap` order or unseeded RNG into game logic. Full
  write-up and the bisect recipe are the P0 at the top of the robustness
  section. **This outranks every perf item**: it makes every cube/`all`
  measurement unreproducible, including the "~0.1 % stall rate" this file
  treated as a stable number.
- **The stall question is answered and needs no more work.** `stalls_by`
  (`419d2ea6`) reads **cap 0 / stuck 0 / draw 4-6** on `--decks all
  --games 300 --seed 11`: every undecided game is a rules draw, not a
  simulator failure. Nothing to fix — but the count moves run to run,
  which is the P0 above, not a stall problem.
- **Filters.** Eleventh (`15ec11c1`) is the first that found anything:
  `stale < 8` written out six times across five files, now one
  `STALE_ROUNDS`. A **twelfth** is owed; the natural next from the same
  family is *a predicate two callers each re-derive*. Pass 24's
  clone-then-narrow filter is still unswept semantically —
  `.keywords.to_vec()` inside an `.any()` survives at `mod.rs:5721`,
  `actions.rs:10472`, `movement.rs:835` (all small; arithmetic says under
  the floor, so cost before writing).
- **Collision hygiene, if it happens again.** Fetch before starting a perf
  row, not just before pushing. The rebase cost more than the row did, and
  `git add -A` had swept tracker edits into a code commit, which is what
  made the conflicts messy — stage explicitly.
- **Env.** No `cargo-nextest`; `cargo test -p crabomination -p
  crabomination_tests` is the gate (~25 min cold, ~45 s warm, always with
  `CARGO_INCREMENTAL=0`). Cold `profiling-fast` build ~12 min, **engine-
  only rebuild ~3.5 min**, callgrind ~7. `release` (cgu 1 + thin LTO) is
  ~22 min — budget for it before starting, it is what the `--bench` anchor
  needs. The SessionStart hook again left the client apt deps uninstalled
  (`pkg-config --exists wayland-client` false); the four-package
  `apt-get install` below fixes it in a minute and clippy needs it.
- **Trackers.** PERF **1.78k**, TODO ~1.08k, roadmap 660. PERF is well over
  the ~1k guidance and the compaction is *shovel-ready*: passes 20-25 fold
  into the Log index exactly as 1-19 already did (~150 lines -> ~45), and
  passes 12-18's frozen candidate snapshots in the candidates section
  (~287 lines, every entry paid or restated above at a fresher share)
  collapse to a pointer. Hoist two things rather than dropping them: the
  "Ir over-weights allocation and representation changes" warning into the
  methodological notes, and the `ability_strip_in_scope` soundness device
  into the longer-lived list. A session did exactly this and lost it to the
  rebase — it is worth ~330 lines.

## Environment note

The `crabomination_client` (Bevy GUI) needs system libs the base image lacks.
They install cleanly via apt in the routine environment:
`apt-get update && apt-get install -y libwayland-dev libasound2-dev
libudev-dev libxkbcommon-dev`. After that `cargo build/test -p
crabomination_client` compiles (first build ~6 min). The GUI still can't be
*run* headless (no GPU/display — see the `verifier-client` skill), but client
code and its unit tests now compile and test here.

Two gotchas seen this run: `apt-get install` without a preceding `apt-get
update` 404s on a stale index (the `.claude/hooks/install-client-deps.sh` hook
already does the update, but its failures are silenced, so check
`pkg-config --exists wayland-client` before blaming the crate); and a full
`cargo test --workspace` including the client can fill the disk — `rm -rf
target/debug/incremental` reclaims several GB without a full rebuild.

## Engine — Robustness / defects (open)

### Determinism — closed 2026-08-11 (`841dd40b`)

The cube pool's fixed-seed nondeterminism is fixed and the whole class is
shut: `crate::fxhash::HashMap` / `HashSet` (rustc's seedless FxHasher)
replace `std`'s across the engine, so no map's walk order can differ
between two runs of one seed. `--decks cube` reads decisions **1,130,728**
identically over three runs, `all` 2,548,986 and `sos` 684,268 over two
each, `determinism ok` on every one; `--decks fixed` is unchanged at
193,232. `--decks all`'s stall rate is now a stable **6 draws / 5,100
games (0.12 %)** — all rules draws, nothing to fix. A separate leak fixed
in the same sitting (`125108c1`): CR 705.1 coin flips read
`rand::random()` inside `AutoDecider`, and Mana Crypt is in the cube pool.

**What is left of it, as a rules question, not a determinism one.** A map
whose walk order picks a *game outcome* is still arbitrary, just
reproducibly so. Known site: `actions.rs`'s discard-cost gate does
`by_name.values().find(|ids| ids.len() >= count)` for "discard N cards with
the same name" (Kozilek-style), which picks an arbitrary qualifying name
rather than the cheapest. Sweep the ~110 map/set locals for siblings when
someone wants a rules pass; none of them can desynchronize a run any more.

Found by profiling. Not speculative — the code is quoted.

*(No open entries. The sibling `counters` `HashMap`-order defect was fixed
in `df87c2d1`: `CardData.counters` is now a `CounterBag`, an
insertion-ordered `Vec` newtype, because `Effect::RemoveAnyCounter` reads
"the first present kind" off the map and six other sites collect the kinds
into a `Vec` and act on them in order — `RandomState` reseeds that order per
process, so two runs on one seed could diverge.
`cr_122_counter_bag_order_is_insertion_order` pins it. **That audit is now
done and its one finding is fixed.** The survey — every `HashMap`/`HashSet`
field of `GameState`, `ColdState` and `Player`, asking of each consumer
whether it sums / maxes (safe), tests membership or counts (safe), looks up
by key (safe), or `find`s / `collect`s / iterates into an ordered structure
(not) — turned up exactly one leak in 31 fields:
`dispatch_triggers_for_events` walked `died_card_snapshots.values()`
pushing `TriggerCandidate`s, and a candidate's *position* decides where its
ability lands on the stack, so two dying creatures with LKI triggers
(Enrage on lethal damage, a granted "when this dies") stacked in
`RandomState` order — different in every process. Fixed by making the field
an `IdMap`, the insertion-ordered `Vec` newtype in `game/types.rs`; die
order is also the order CR 603.3b wants. Everything else was keyed lookup,
membership, or an order-independent fold — including the three sites that
*look* risky and are not: `encode.rs` sums `block_map` into a
`blocker_sums` map read by key, `bot.rs`'s two `block_map.keys().collect()`
are `contains` + `len` only, and `combat.rs`'s
`block_map.keys().for_each(want)` decides which permanents get computed,
never what a reader sees. The `keyword_counters` `HashMap`-order defect was fixed
in `86670250`: `KeywordCounters` is an insertion-ordered `Vec` newtype, which
sidesteps `Keyword` having no `Ord`, and `cr_122_1b_keyword_counter_grant_
order_is_insertion_order` pins it. The actor-sampler panic was fixed in
`a67c5b9a`. Mirror Gallery aborting the whole SBA sweep was fixed in
`9db8557c` — CR 704.5j's `LegendRuleDoesntApply` check sat inside the
legend-group block and used `return Vec::new()`, so a board with one out
skipped every later state-based action (deaths, loss conditions, the Aura
and Equipment sweeps) and discarded the sweep's events; the game could not
be won or lost. Regression test in `classic_sets/bok`. **The filter that
found it: a `return` inside a `let … = { … };` initializer block**, which
exits the whole function rather than the block. The workspace was swept for
the class — the other nine hits are all `Err` / `let-else` guards that
legitimately abort their function, so this was the only one.)*

**Open: the panic/unwrap sweep of the self-play path.** ~183
`unwrap()`/`expect()` under `game/` + `bot.rs`. It wants **triage, not a
blanket rewrite** — every site spot-checked so far was already guarded by a
preceding test (`worlds.len() > 1` before `.max().unwrap()`,
`battlefield.push` before `find(…).unwrap()`, `writes_to_shared` before the
two `teams` unwraps, `mayhem` set from `mayhem_cost().is_some()`). The
filter that actually found `a67c5b9a` is narrower and worth reusing: **a
`debug_assert!` standing in for a runtime guard**, or a `len() - 1` / bare
index on a slice whose emptiness the *caller* tolerates. `sample_scored_index`
had both, on the one path only a training actor takes.

**Both of that filter's halves were swept 2026-08-10 and are clean** — a
negative result worth not re-deriving. The `len() - 1` half: 13 sites under
`game/` + `bot.rs`, every one either preceded by an `is_empty()` early
return (`apply_enters_as_choice`, `pick_trigger_mode`, the Captive Audience
mode picker, `EscalatingThisTurn`), taken on a `const` array of five card
types, taken right after the matching `push`, or guarded by a `first()`
let-else. The `debug_assert!` half: only two sites remain
(`mod.rs:3900`'s replacement-effect iteration cap, `stack.rs:5911`'s
unsupported redirect target), and both fall through to a defined release
behaviour rather than standing in for a guard. What is left of the item is
the ~183 `unwrap()`/`expect()`, which still wants triage rather than a
blanket rewrite — and a *third* filter, since these two are exhausted.

**The third filter was run 2026-08-10 and is also clean.** Two shapes, both
chosen because they fail *silently* in release (wrapping) and loudly in
debug, i.e. the profile of a bug that only appears at game 400 k:

- **Unsigned `len() - k` where the caller tolerates an empty collection.**
  16 hits under `game/` + `bot.rs`. Every one is guarded: an `is_empty()`
  early return, a `push` on the line above (`stack.len() - 1` in
  `CreateTokenCopyOf`), a `const` array, a `first()` let-else, or a
  `hand.len() > max` test in the same condition (the two cleanup-discard
  sites). `selfplay.rs:521`'s bare index is in a `#[cfg(test)]` body.
- **A stale index across a mutation** — `position()` / `iter().position`
  followed by `battlefield[pos]` after something that can remove a card.
  53 index sites, and the one path that genuinely mutates in between (the
  equip sacrifice) already re-finds by id and carries a comment saying why.

**The fourth filter was run 2026-08-10 and is also clean.** Two shapes,
both chosen because they wrap silently in release — a negative index or
count that only appears at game 400 k:

- **`evaluate_value(…) as usize` without a `.max(0)`.** `Value` evaluation
  returns `i32` and is trivially negative (a `Diff`, a `PowerOf` on a
  -X/-X'd creature). One hit in the whole workspace, and it is `.max(1)`
  (`mod.rs:20879`). Every other one of the ~270 sites carries `.max(0)`.
- **`power()` / `toughness()` / `life` cast to `usize`.** One hit,
  `bot.rs`'s `LIFE_TENTHS[life as usize]`, and the `life <= 0` and
  `life <= MAX` branches above it are exactly the guard.

**The fifth filter was run 2026-08-10 and is also clean.** It looked where
the four before it did not — at a precondition *some* sites enforce and a
sibling might not, rather than at the site alone. Two sweeps:

- **Documented preconditions.** Eight `///` blocks under `game/` + `bot.rs`
  state one ("must already be", "assumes", "must be non-empty"). Every one
  is either validated in the body (`assign_teams` returns a typed
  `TeamError` for each of empty / unknown / duplicate / missing seat) or
  holds structurally. No caller-side gap.
- **The `len() - 1` clamp family, the shape the third filter opened and did
  not close across `effects/`.** Ten sites clamp a decider-supplied index
  with `i.min(xs.len() - 1)`, which underflows to `usize::MAX` on an empty
  `xs` and then indexes out of bounds. **All ten are guarded**, and by
  three different idioms — an explicit `if xs.is_empty() { return … }`
  (`apply_enters_as_choice`, `apply_enters_mode_choice`,
  `EscalatingThisTurn`, `pick_trigger_mode`, and both `available`
  builders), a pattern guard (`Some(modes) if !modes.is_empty()` in
  `clamp_activated_mode`), a `first()` let-else (`LoseKeyword`), or a
  `const` five-element array (the two `CardType` pickers). The precondition
  is real and nobody forgot it.

**The sixth filter was run 2026-08-11 and is also clean** — the one the
five before it pointed at, and the only one that does not pattern-match on
syntax. `[profile.overflow]` (`release-fast` + `overflow-checks = true`,
committed in `Cargo.toml` with the invocation in its comment) turns every
silent wrap into a panic with a backtrace. Run:

- `bot_ladder --a gang --b gang --games 300 --threads 3`, four seeds across
  all four deck pools (`all` x2, `sealed`, `cube`, `sos`) — **17,693 games
  decided, 0 panics**.
- `selfplay_train --actors 3 --games 600 --steps 60 --batch 64` — the real
  actor path, encoder and learner included — **600 games, 56,353 rows, 0
  panics, 0 stalls**.

So the arithmetic is clean on ~18 k games across every pool the bench and
the trainer touch. What is left of the item is still the ~183
`unwrap()`/`expect()`, wanting triage rather than a blanket rewrite, and a
*seventh* filter — the six above are exhausted. Rerun the overflow profile
after any change to counters, damage, mana or the encoder; it costs one
9-minute build and two minutes of games.

**The seventh filter was run 2026-08-11 and is also clean.** The six above
all hunt a *silent* wrap; this one hunts the opposite — an integer or float
division/modulo whose denominator is a runtime count the caller can make
zero, which panics loudly (or goes `NaN`) rather than wrapping. Every
`/` or `%` by a non-constant under `game/`, `bot.rs` and
`crabomination_ml/` was read: the seat-rotation family is `% players.len()`
and a game always has a seat; `DealDamageDividedEvenly` guards with
`targets.is_empty()`; `DigToHandLoseLife` guards with `per > 0`; the
Praetor's Grasp-style `order[seat % order.len()]` builds `order` with the
controller already pushed; `max_affordable_x` clamps `x_pips` with
`.max(1)`; the bot's race math is inside `total_raw_power > 0 && opp_clock
> 0`; every ML rate is `.max(1)`. The one float site,
`sample_scored_index`'s `/ temp`, is reached only from `sampling_temp`,
which the trainer sets behind `args.sample_temp > 0`. No hit.

**The eighth filter was run 2026-08-11 and is also clean.** The first six
hunt a silent wrap and the seventh a zero denominator; this one hunts the
*other* loud panic — a std collection or slice operation whose runtime
argument is a length, not an index, so the third filter's bare-index sweep
never looked at it. Four shapes, whole workspace:

- **`split_off(n)` / `split_at(n)` / `copy_from_slice`.** Five `split_off`
  sites. The four in `effects/mod.rs` (the copy-a-spell repointing) take
  `taken.split_off(t.iter().len())` where `taken` was *initialized* from
  `t` and only pushed to since, so `len >= n` structurally;
  `bot.rs:7052`'s redeal is `library.len().saturating_sub(n)`. The four
  `copy_from_slice` sites in `crabomination_ml` copy `[f32; AUX_FEATS]` /
  `[f32; GLOBAL_FEATS]` / `[f32; OBJ_FEATS]` **arrays**, not slices, into
  same-width windows — a length mismatch is a compile error, not a panic.
- **`chunks(n)` / `chunks_exact(n)` / `step_by(n)` with a runtime `n`**
  (all three panic on zero). One hit each: `make_batch`'s
  `rows.chunks(chunk.max(1))`, and `EachPlayerSplitsAndSacrificesRandom
  Pile`'s `step_by(n)` where `n = (*piles).max(1)`.
- **Runtime range slicing `&xs[a..b]`.** Two sites, both in
  `continue_trigger_ordering`'s same-controller run walk, both `i < j <=
  rest.len()` by the loop that computed them.
- **`Vec::remove(i)` / `insert(i, _)` with a computed index.** ~30 sites
  under `game/` + `bot.rs`; every one is an `if let Some(pos) =
  …position(…)` on the same collection, an index below a `len()` the loop
  condition holds (`hybrids.remove(idx)` inside `while !hybrids.is_empty()`
  with `unwrap_or(0)`), or a `0..greedy.len()` enumeration.

No hit. The item is still the ~183 `unwrap()`/`expect()` wanting triage.

**The ninth filter was run 2026-08-11 and is also clean.** The first eight
hunt a wrap, a zero denominator, or a length-argument panic; this one hunts
the third loud panic std can raise — **a comparator that is not a total
order**, which `sort_by`/`sort_unstable_by` detect and panic on, and which
a `NaN` produces for free. Every `partial_cmp` / `sort_by` /
`sort_unstable_by` / `max_by` / `min_by` / `binary_search_by` under
`game/`, `bot.rs`, `crabomination_ml/` and `crabomination_nn/` was read.
There is **no `partial_cmp(…).unwrap()` in the workspace**. Every float
comparator is either `total_cmp` (`selfplay.rs`'s argmax,
`recommend_pool`'s ranking, `selfplay_train`'s quantile) or
`partial_cmp(…).unwrap_or(Equal)`; the latter is only inconsistent if a
`NaN` reaches it, and the three sites that feed one to a `sort_by` are all
in `recommend.rs` — off the self-play path — with `win_rate()` guarded at
`decided() == 0` and `best_delta()` built from it. Every comparator on the
engine's own hot paths (`layers.rs`'s layer/sublayer/timestamp sort,
`bot.rs:2871`, `view.rs:560`, `effects/mod.rs`'s five descending-index
sorts) is integer `cmp` and total by construction.

**The tenth filter was run 2026-08-11 and is also clean.** It hunts the
failure mode a training run notices as a *hang* rather than a panic: **an
unbounded `loop` / `while` whose exit condition is game state**. All eight
`loop {` and ~40 non-`while let` `while` sites under `game/` and `bot.rs`
were read. Three shapes, all bounded: a collection that strictly shrinks
each round (every `while !library.is_empty()` / `while !pool.is_empty()` /
`while live.len() > 1`, and the CR 616.1e draw-replacement loop, whose
`declined` list grows monotonically and filters `applicable`); a structural
peel that descends a finite effect tree (`active_static`'s wrapper loop,
`pick_trigger_mode`'s `MayDo`/`CapTargetsAt` peel); or an explicit counter
(`bot.rs`'s three sim loops carry `fuel`, the coin-flip loop a
`wins >= 64` backstop, `subgame.rs` `MAX_ACTIONS` + `stale < 8`, the
`source_zone` sweep a `budget`). **The one that is not bounded by any of
the three is the top-level game loop itself** — `play_one_game_traced`'s
`while !g.is_game_over() && actions < max_actions && stale < 8` — and it is
bounded by its own two counters, which is exactly what a *stall* is. That
is the open item below, not a new one.

**The eleventh filter was run 2026-08-11 and is the first of the eleven
that found something** (`15ec11c1`). It looks at neither syntax nor a
precondition but at *duplication*: **one invariant written out by hand in
more than one place.** `stale < 8` — the "neither bot volunteered an
accepted action for N consecutive rounds, give the game up" fixed point —
appeared **six times across five files**: the ladder loop
(`recommend.rs`), the recording loop (`selfplay.rs`), the subgame loop
(`game/subgame.rs`, shipped CR 729 rules code), the MCTS rollout
(`server/mcts.rs`), `bot_probe`, and a bot test — plus `bot_probe`'s
`stale >= 8` report threshold. Nothing tied them together, so **the
ladder's stall rate and the training actor's were never the same
measurement**, and a change to one would silently not reach the others.
All six read `recommend::STALE_ROUNDS` now; no value changed. The
per-context *action* budgets sitting next to them (4,000 in the training
actor, 20,000 in `bot_probe` and the golden traces, 50,000 in the bot
test, `MAX_ACTIONS` in the subgame) are deliberately different and were
left alone — the filter is for a fixed point over the bots, not for every
literal.

A *twelfth* filter is owed. The natural next one, from the same family:
**a predicate two callers each re-derive**, rather than a constant two
callers each spell out.

**Stall rate — the top cause is now askable, which was the blocker**
(`419d2ea6`). The measurement stands: the wider pools stall at **~0.1 %** —
`all` seed 11 5/5,100, `cube` seed 41 2/2,400, `all` seed 7 0/5,100,
`sealed` 0/3,600, `sos` 0/1,500; `--decks fixed` reads 0 and always has,
which is what hid it. What was missing was not games but *attribution*: the
loop ends on either `actions >= max_actions` or `stale >= STALE_ROUNDS` and
reported neither. `recommend::StopReason` now carries which, `GameOutcome`
and `RecordedGame` both hold it, `bot_ladder --bench` prints a
`stalls_by cap / stuck / draw` line beside `stalls`, and
`selfplay_train`'s `stats.jsonl` gains `stalls_capped` / `stalls_stuck`
next to `stalls` — so a training run that starts stalling says why in the
same file that says how fast it is going. **Next step is a measurement,
not a change**: `--decks all --games 300 --seed 11 --bench` (= the 5,100
games above) and read `stalls_by`. `cap` and `stuck` want opposite fixes —
a budget too small for a grindy pool, against a genuine no-legal-move fixed
point — so do not guess which before the line says. Not run this session:
the release link was cut short by a rebase onto a concurrent session.

## Engine — Missing Mechanics

### Replacement Effects
The engine has no general replacement-effect primitive.  Many real cards need one:
- ETB replacements (Containment Priest, Torpor Orb, Rest in Peace)
- Damage replacements (protection, preventing damage):
  - 🟡 **Combat damage prevention** (Owlin Shieldmage, Holy Day, Constant
    Mists) is partially supported via the new `Effect::PreventAllCombatDamage
    ThisTurn` primitive + `GameState.prevent_combat_damage_this_turn` flag
    (CR 615.1). Per-source / per-N shields (Wojek Apothecary, Stave Off,
    Lapse of Certainty) are still ⏳. Non-combat damage prevention
    (Reverse Damage, Mending Hands) is also ⏳.
- Draw replacements (Leyline of the Void)
- Death replacements (Kalitas, Oubliette)
Until this lands, cards with "instead" clauses are either stubbed or collapsed
into a close approximation.

### Per-Activation Mana-Spent Introspection
Reckless Amplimancer reads "+X/+X where X is the amount of mana spent to
activate this ability". The engine tracks per-cast `mana_spent` on
`StackItem::Spell` and per-trigger on `StackItem::Trigger`, but the
activated-ability path (`activate_ability`) doesn't capture mana spent.
Adding this requires:
1. An `x_value: Option<u32>` field on `GameAction::ActivateAbility` for
   X-cost activations (parallel to `CastSpell.x_value`).
2. Threading `mana_spent` through the activation's `StackItem::Trigger`
   construction in `activate_ability` (the field exists but is always 0).
3. Wiring `Value::CastSpellManaSpent` to read from the stack item.
Then Reckless Amplimancer's +3/+3 hardcode can be replaced with
`Value::CastSpellManaSpent` for printed-Oracle parity. Tracked as engine
work — same shape would unlock other X-cost activations (Berta's
{X},{T}: Create Fractal with X counters).

### Cast-From-Exile Pipeline
Many cards exile a spell/card temporarily and later cast it (Foretell,
Suspend, Rebound, Flashback-from-exile, Escape, Adventure second cast,
Cascade resolution).  Currently each is handled ad-hoc or omitted.  A shared
"cast from alternate zone" code path would unlock dozens of cards.

### Triggered-Ability Event Gaps
`EventKind` is missing several commonly-needed triggers:
- `PermanentLeftBattlefield(CardId)` — needed for general "LTB" abilities.
  (Linked exile-until-LTB now handled directly via `return_linked_exiles`
  / `CardInstance.exiled_by`, not via an event.)
- `DamageDealtToCreature` — needed for enrage, lifelink gain on creature damage
- `TokenCreated` — needed for populate, alliance triggers
- `CounterAdded / CounterRemoved` — needed for proliferate payoffs, Heliod combo
- `SpellCopied` — storm payoffs, Bonus Round
- `PlayerAttackedWith` — needed for Battalion and similar attack-count effects
- ~~`SpellCastTargetingCreature` (or a `Predicate::SpellTargetsCreature`
  knob) — needed for Strixhaven Repartee.~~ **Done**: see
  `Predicate::CastSpellTargetsMatch` + `effect::shortcut::repartee()`.
  Stirring Hopesinger, Rehearsed Debater, Informed Inkwright, Inkling
  Mascot, Snooping Page, Lecturing Scornmage, Melancholic Poet, and
  Graduation Day all use it. Remaining Repartee cards are blocked on
  separate primitives (exile-until-X, copy-spell). Ward enforcement
  (mana-cost variant) shipped in push (modern_decks) — see Inkshape
  Demonstrator promotion + `push_ward_triggers_for_cast` in
  `game/actions.rs`.
- ~~`CardLeftGraveyard` — needed for Lorehold "cards leave your
  graveyard" payoffs.~~ **Done** in push V: see
  `EventKind::CardLeftGraveyard` + `Predicate::CardsLeftGraveyardThisTurnAtLeast`.
  Hardened Academic, Spirit Mascot, Garrison Excavator, Living
  History all wired. Remaining gy-leave-aware cards (Ark of Hunger,
  Owlin Historian, Primary Research, Wilt in the Heat) need only
  catalog wiring against the event.

### Multi-Card Batch Triggers
The engine emits `CardLeftGraveyard` per card removed; printed cards
say "Whenever **one or more** cards leave your graveyard". We
approximate by firing the trigger per-card (a strict power upgrade
on multi-card-removal turns, but harmless in 2-player play where
single-card returns dominate). A future refinement: collapse a
batch of `CardLeftGraveyard` events emitted in the same resolution
window into one trigger fire (similar to MTG's "looks back in time"
rule for batch triggers). Same shape applies to `CardDiscarded`,
`CreatureDied`, and any future per-zone-move event.

**Per-event fan-out fix (push c4b7b14)**: The dispatcher previously
broke after the first matching event per (source, trigger) pair,
silently swallowing later events in the same batch. This was a
regression for multi-attacker swings (Sparring Regimen) and any
"whenever X happens" trigger over a batch of N events. The
dispatcher now keeps iterating over events for batch-fanout-friendly
event kinds (Attacks, CreatureDied, CardDrawn, CardDiscarded,
CardLeftGraveyard, CounterAdded, Blocks, BecomesBlocked, LifeGained,
LifeLost, BecameTarget) — one trigger fires per matching event,
matching the printed Oracle wording. Other event kinds (ETB,
StepBegins, …) keep the at-most-once guard because they don't emit
duplicate events in a single batch.

### Spell-Side Predicate: Mana-Spent-On-Cast
SOS introduces **Increment** ("if mana spent > this creature's P or T,
+1/+1 counter") and **Opus** ("Whenever you cast an instant or sorcery,
do X. If five or more mana was spent, do bigger X"). Both need a
per-cast "mana value paid" snapshot exposed as a `Value` (or a
`Predicate::ManaSpentAtLeast(n)`). The engine already retains the cost
on the `StackItem`; lifting that into the `EffectContext` for trigger
filters should unlock a few dozen Strixhaven cards.

### X-Cost and Converge
`Value::XFromCost` exists but converge (number of *distinct colors* of mana
spent) is not tracked per cast.  `Value::ConvergedValue` is a stub that always
returns 0 for non-Prismatic-Ending uses.  Fix: record color set paid at cast
time and expose it as a `Value` primitive.

### Cost-Reduction Stacking
Delve, Improvise, Convoke, and generic cost-reducers each have separate
branches.  There is no unified "reduce mana cost by X before payment" hook,
making cards like Hogaak (Convoke + Delve) or Affinity impossible to express
cleanly.

### Target-Aware Cost Reduction
"This spell costs {X} less to cast if it targets [some condition]" is a
Strixhaven design pattern (Ajani's Response, Brush Off, Run Behind,
Mavinda, Killian, Orysa). Today we either drop the discount and ship the
spell at its printed full cost, or omit the spell entirely. Engine fix:
let `CostReduction` static / per-card alt-cost evaluate against the
candidate-cast's chosen target before payment. Probably a new
`SelectionRequirement`-keyed cost discount that the cast path consults.

### Mana Ability from Non-Battlefield Zone
`activate_ability` only walks the battlefield.  Cards like Elvish Spirit Guide
and Simian Spirit Guide (exile from hand: add mana) ship as vanilla bodies;
the "exile from hand: add mana" half needs a from-hand activation zone (adding
an `ActivatedAbility.from_hand` flag parallel to `from_graveyard` would mean
touching ~240 literal constructors — migrate them to `..Default::default()`
first).

### Delirium-conditional static buffs
`Predicate::DeliriumActive` now gates spell effects (Unholy Heat). A
*continuous* delirium buff — "as long as you have delirium, this gets +2/+2
and has flying" (Dragon's Rage Channeler, Traverse the Ulvenwald-adjacent
cards) — needs a layer-system static whose application is gated on a
predicate. DRC isn't implemented yet pending this.

### Client build CAN be verified in the web sandbox (pkg-config + linker shim)
`crabomination_client` links Bevy, whose `wayland-sys`/`alsa-sys`/`libudev-sys`
build scripts call `pkg-config`, and the linker then wants `.so` dev symlinks —
the runtime `.so.N` files exist here but the `.pc` files and `.so` symlinks
don't. Since the Bevy 0.18→0.19 bump the toolchain floor is **rustc 1.95**
(`rustup toolchain install 1.95.0 && rustup override set 1.95.0`). Drop shims +
symlinks in a temp dir and point both `PKG_CONFIG_PATH` and `LIBRARY_PATH` at
it (pkgconf 1.8 rejects a `.pc` with no `Description:` field, and 0.19 added
the `libudev` dep):
```sh
mkdir -p /tmp/pc && cd /tmp/pc
for m in client cursor egl; do printf 'libdir=/usr/lib/x86_64-linux-gnu\nName: wayland-%s\nDescription: shim\nVersion: 1.22.0\nLibs: -L${libdir} -lwayland-%s\nCflags:\n' $m $m > wayland-$m.pc; done
printf 'libdir=/usr/lib/x86_64-linux-gnu\nName: alsa\nDescription: shim\nVersion: 1.2.0\nLibs: -L${libdir} -lasound\nCflags:\n' > alsa.pc
printf 'libdir=/usr/lib/x86_64-linux-gnu\nName: libudev\nDescription: shim\nVersion: 250\nLibs: -L${libdir} -ludev\nCflags:\n' > libudev.pc
ln -sf /usr/lib/x86_64-linux-gnu/libwayland-client.so.0 libwayland-client.so
ln -sf /usr/lib/x86_64-linux-gnu/libwayland-cursor.so.0 libwayland-cursor.so
ln -sf /usr/lib/x86_64-linux-gnu/libwayland-egl.so.1 libwayland-egl.so
ln -sf /usr/lib/x86_64-linux-gnu/libudev.so.1 libudev.so
ln -sf /usr/lib/x86_64-linux-gnu/libasound.so.2 libasound.so
PKG_CONFIG_PATH=/tmp/pc LIBRARY_PATH=/tmp/pc cargo clippy -p crabomination_client
```
Runtime/GPU verification (opening a window) still needs the local
`verifier-client` skill — only compile-checking works headless.

### Damage-as-(-1/-1)-counters replacement
Soul-Scar Mage / Phyrexian Vatmother-style "if a source you control would
deal noncombat damage to a creature, it deals that much in -1/-1 counters
instead" needs a damage-replacement hook. Soul-Scar Mage ships as 1/2 Prowess
without it. (Native Infect/Wither on the non-combat funnel shipped —
`deal_damage_to_from` lands -1/-1 counters / poison; CR 702.80a/702.90e.)

### Phyrexian mana
Mutagenic Growth ({G/P}), Gut Shot, Dismember, etc. — a mana symbol payable
with 2 life. Mutagenic Growth ships at the {G} cost (the life-pay alt is
omitted).

### "Look At Top X, Pick One, Put Rest in Graveyard" Primitive
Stirring Honormancer ("look at top X cards where X is creatures you
control, put one in hand, rest into graveyard") and similar look-and-
sort effects need a "look at top N, choose K, mill the rest" primitive
to express faithfully. `Effect::Surveil` covers the "look + may put in
graveyard" shape but with a fixed number; the SOS variant is dynamic
and forces the rest-to-graveyard branch unconditionally.

### Choice of "Which Zone" for a Tutor Result
Dina's Guidance ("search a creature, put into hand or graveyard")
exposes a 2-option destination prompt that no other primitive currently
needs. Adding a `Effect::Search` flavor with `to: Either(ZoneDest,
ZoneDest)` (or a separate decision shape) would honor the toggle for
this and a handful of black/green search effects.

### Multi-Target Prompt for Sorceries / Instants
A handful of SOS cards specify two target slots with different filters
(Render Speechless: opponent + creature; Cost of Brilliance: player +
creature; Homesickness: player + up to two creatures). The engine
today only exposes a single-target slot per spell at cast time, so
these collapse one of the two halves. A multi-target cast prompt
(`Vec<Target>` in `GameAction::CastSpell`) would unlock all of them.

### Auto-Target Picker: Source-Avoidance + Best-Pick Heuristics
~~The current `auto_target_for_effect` walks the battlefield in `Vec`
order and returns the first legal match.~~ **Source-avoidance done**:
the new `auto_target_for_effect_avoiding(eff, controller, avoid_source)`
takes the trigger source and prefers any *other* legal target,
falling back to the source only when nothing else is legal. All
trigger-creation paths (`stack.rs`'s `flush_pending_triggers`,
`actions.rs`'s ETB triggers, `combat.rs`'s combat triggers, the
delayed-trigger fire path, Dies/PermanentLeavesBattlefield triggers)
now pass the source ID. Quandrix Apprentice's Magecraft pump now
deterministically targets the bear over the Apprentice, and the test
suite asserts the source-fallback when no other target is legal.

~~Prefer the highest-power creature for friendly pumps.~~ **Done** in
push VI: `auto_target_for_effect_avoiding` now sorts the primary-player
candidate set by descending current power when the effect prefers a
friendly target (Magecraft / Repartee fan-outs, transient PumpPT
spells). Hostile picks still use first-match.

Remaining best-pick heuristics still ⏳:
- Prefer creatures whose current power matches what the pump would
  unlock (lethal swing, post-pump unblockable, etc.).

### Mana-Cost Reduction with Target Predicate
Killian, Ink Duelist's "spells you cast that target a creature cost
{2} less" needs a `StaticEffect::CostReduction` variant whose filter
inspects the cast spell's targets. Today's `CostReduction` filters
on the spell card's own attributes only. Plumbing the cast-time
target list into the cost-reduction site would unlock this card and
similar Lorehold/Witherbloom cost-cutters.

### Transient Triggered-Ability Grants on Pump Spells
SOS Root Manipulation ("Until end of turn, creatures you control get
+2/+2 and gain menace and 'Whenever this creature attacks, you gain
1 life.'") needs a way to attach a *triggered* ability to a creature
for a duration, on top of the keyword-grant primitive. Today the engine
has `Effect::GrantKeyword { what, keyword, duration }` but no
`Effect::GrantTriggeredAbility { what, ability, duration }`. Adding
this would unlock the third clause of Root Manipulation, similar
"creatures gain combat-damage trigger until EOT" pump spells, and
the on-attack rider on tokens (Pest token's "gain 1 on attack",
Spirit token combat triggers).

### Self-Counter-Scaled Cost Reduction
SOS Diary of Dreams's `{5},{T}: Draw a card` activation costs `{1}`
less per page counter on the source. There's no
`StaticEffect::CostReduction` variant whose discount scales off the
source's own counter count. Adding a `CostReduction { delta:
Value::CountersOn { what: Selector::This, kind: Charge } }` shape
would unlock Diary of Dreams cleanly, plus other counter-scaled cost
reducers (M21 Mazemind Tome).

### Counter-Removal Activation Cost
✅ Shipped as `ActivatedAbility.remove_counter_cost` (Walking Ballista's
`Remove a +1/+1 counter: deal 1`, Barkhide Troll's hexproof pump).
Experiment One's `Remove two: Regenerate` still pending a per-card pass.

### Page Counter Type
SOS Diary of Dreams (and the rest of the SOS book/grandeur subtheme)
references "page counter" but the engine `CounterType` enum has no
`Page` variant. Diary is currently approximated with `CounterType::
Charge`, which is fine in 2-player play (no other card uses Charge as
a payoff source) but obscures the printed identity. Adding `Page`,
`Knowledge`, and the small handful of other novelty counters from
recent sets would close the gap.

### `Move`-with-count for Selecting One Card from a Zone
Today `Effect::Move { what: Selector::CardsInZone { zone: Graveyard, ... } }`
moves *every* matching card. Cards like Heated Argument's "you may
exile a card from your graveyard" need a "move at most one matching
card" primitive. A `Selector::OneOf(inner)` wrapper, or a `count` knob
on `CardsInZone`, would fix this. The current workaround for Heated
Argument collapses the optionality into "always do the rider".

### "Choose Up To N Modes (with Repetition)" for `ChooseMode`
Strixhaven's "Choose up to four. You may choose the same mode more
than once." pattern (Moment of Reckoning, Witherbloom Charm-style
spells with N copies) needs an extension on `Effect::ChooseMode` that
takes a list of (index, target) tuples per cast. Today the engine's
modal flow picks exactly one mode and one target per cast — the
"choose up to N" wrappers collapse to single-mode resolution.

### "X Life as Additional Cost" Primitive
Vicious Rivalry, Fix What's Broken, and a handful of SOS sorceries
have "As an additional cost to cast this spell, pay X life." The
engine has no per-cast life-payment cost — we approximate by reading
X from the spell's `{X}` slot and running `LoseLife X` at resolution
time, but that double-counts X (paying X mana via XFromCost AND X
life). A `cost.life: Value` field on `CardDefinition` (or an
`alternative_cost` variant whose payment also requires the life)
would make this faithful.

### "Track Cards Discarded by This Effect" Counter
Borrowed Knowledge ("draw cards equal to the number of cards
discarded this way") needs a per-resolution counter that
`Effect::Discard` increments. The mode 1 path is currently
approximated as "draw 7" — a flat-7 reload that misses the printed
"draw exactly as many as you discarded" precision but preserves the
card-advantage tally for typical hand sizes.

### Capture-As-Target From Selector (Repartee Exile-Until-End-Step)
Conciliator's Duelist's Repartee body wants to:
1. Exile the cast spell's chosen creature target
   (`Selector::CastSpellTarget(0)` — wired).
2. Schedule a delayed trigger that returns *the exiled card* to
   battlefield at next end step.

Step (2) collides with `Effect::DelayUntil`'s capture model — it
captures `ctx.targets.first()`, but a Repartee trigger has no
target slot of its own (the selector is what tracks the spell's
target). Need either:
- An `Effect::CaptureTargetFromSelector { slot, selector }` that
  mutates ctx.targets so the subsequent DelayUntil reads it back, OR
- An `Effect::ExileWithDelayedReturn { what, kind, controller }`
  combinator that pre-resolves the selector at registration time.

The latter is more general. (Tidehollow Sculler / Banisher Priest /
Fiend Hunter are now handled by the dedicated
`Effect::ExileUntilSourceLeaves` / `ExileChosenUntilSourceLeaves`
primitives — see FEATURE_ROADMAP Tier-1 #4.) The former is smaller
surface but introduces effect-side mutation of ctx.

### "Move at most one matching card" — `Selector::OneOf`
Several SOS effects exile/move "a card" from a graveyard, hand, or
top of library where the count is at most 1 (Heated Argument's "may
exile a card from your graveyard", Practiced Scrollsmith's "exile
target noncreature/nonland card from your graveyard"). Today
`Selector::CardsInZone { ... }` returns ALL matching cards. Adding
`Selector::OneOf(Box<Selector>)` (or a `count` knob on `CardsInZone`)
would let these spells correctly pick exactly one. Without it, the
catalog approximates by "exile every matching card" which over-
shoots when the graveyard has multiple matches.

### Snow Mana Validation
`ManaPool` tracks a `snow` counter but `pay()` never validates that a `Snow`
mana symbol must be paid from a snow source.  Any mana from any land currently
satisfies a `{S}` pip.

### Multiplayer / Commander Format
- Command zone: `Zone::Command` exists but `ClientView` has no field for it;
  the server never moves cards there.
- Commander damage tracking (21 from the same commander = loss).
- "Your opponents" vs. "each other player" distinctions (multiplayer targeting
  semantics differ from 2-player).
- Four-player free-for-all match setup in `run_match` / `build_cube_state`.
- Commander-specific rules: color identity deck building, commander tax.

### Planeswalker Interactions
- Planeswalkers can be attacked directly — `AttackTarget::Planeswalker` is in
  `types.rs` but the bot never chooses it and the client has no UI for it.
- "Planeswalker redirect" rule (damage that would be dealt to a player can be
  redirected) is unimplemented.
- Emblems are not modelled.

### Saga Lore Counters
✅ Non-DFC Sagas ship via `CardDefinition.saga_chapters` + `saga_advance`
(ETB chapter I, +1 lore each precombat main, final-chapter sacrifice SBA).
History of Benalia, The Eldest Reborn. Remaining ⏳: DFC/transforming sagas
(The Everflowing Well saga-land) and read-ahead chapter-choice variants.

### Vehicle / Crew
`CardType::Artifact` exists but there is no `CrewN` keyword or "becomes a
creature until end of turn" mechanism.  Vehicle subtype is in `ArtifactSubtype`
but nothing uses it.

### Proper Split-Damage Distribution
Effects like Pyrokinesis ("deals 4 damage divided as you choose among any
number of targets") are collapsed to a single-target 4-damage hit.  A
`DealDamageDivided { total, targets: Vec<Selector> }` effect would express
the real card.

### Affinity / Self-Permanent-Scaled Cost Reduction
Witherbloom, the Balancer's "Affinity for creatures (this spell costs
{1} less to cast for each creature you control)" needs a per-cast cost
reduction whose discount scales off the caster's permanent count.
`StaticEffect::CostReduction { filter, amount }` is a fixed amount
today. Generalising to `amount: Value::CountOf(Selector)` (or a sister
variant `AffinityCostReduction { filter, scaler: Selector }`) would
unlock Affinity for Artifacts (Modern Affinity / Cranial Plating-era
shells), Affinity for X (Strixhaven Witherbloom + future), and Awaken
the Woods-style "X = forests" payoff costs.

### Exile Zone as Viewable State
Exile is a zone in the engine (`Zone::Exile`) and cards move there.
`ClientView.exile` now projects the shared exile zone with each card's
owner so the UI can render an exile browser (added with the
Strixhaven coverage push). Remaining gaps:
- The 3D client has no exile browser UI yet.
- Graveyard-order information is lost (cards are a flat Vec).

---

## Engine — Approximation Cleanups

Most prior approximations have been resolved (Windfall, Dark Confidant,
Biorhythm, Coalition Relic, Fellwar Stone, Static Prison, Rofellos, Grim
Lavamancer, Ichorid, Render Speechless — see `git log -p -- TODO.md` for the
per-card primitive + tests). Still open:

| Card / Feature | Current Approximation | Correct Behaviour |
|---|---|---|
| Spectral Procession | `{2}{W}` (most-permissive collapse of the three `{2/W}` hybrid pips onto the generic side) | Real Oracle `{(2/W)}{(2/W)}{(2/W)}`. Needs an engine-wide `ManaSymbol::HybridGeneric(u32, Color)` variant before the true hybrid cost is faithful. |

### Prepare Mechanic (SOS)

The June 2026 rework replaced the incorrect MDFC model with the printed
mechanic (`prepare_spell` + `CastPrepareSpell`; see `.claude/prepared.md`).
All 36 preparation cards audited against Scryfall oracle. Residual
approximations (each documented at the card site):

| Card / Feature | Current Approximation | Correct Behaviour |
|---|---|---|
| Copy-in-exile object | The spell copy materializes at cast time | A copy sits in exile while the creature is prepared, so "cast from exile" zone-watch triggers should see it |
| Bot play | Bot casts prepare spells: main-phase candidates scored by the inset spell, instant insets fired in response to removal on the prepared body (`pick_prepare_response`), off-card re-prepare abilities used as a mana sink (`pick_reprepare`), the counter priced into `permanent_value`, and X-cost inset spells sized like hand casts (`max_affordable_x_for_def`) | — |
| Emeritus of Truce ETB | Inkling token minted for *you* | "Target player creates…" (needs a trigger-scoped target slot for `CreateToken`) |
| Harmonized Trio | Activation taps one other untapped creature | "Tap two untapped creatures you control" |
| Scrollboost | Single target +2/+2 | "One or two target creatures" |
| Secret Rendezvous | Each player draws 3 (equivalent in 1v1) | "You and target opponent" |
| Striking Palette | Armed window consumed by the next spell of any type (copy still gated to instant/sorcery) | "When you next cast an instant or sorcery spell this turn, copy it" |
| Bind to Life | Mill 7, then return a creature from the graveyard | "…from among the milled cards" (needs a scratch selector) |
| Oracle's Gift | Counters land on the freshly-minted batch only | "…on each Fractal you control" (pre-existing Fractals miss out) |
| Swords to Plowshares rider | Lifegain approximated off the target's controller-as-resolved | Printed: target's controller gains the life — verify the `PlayerRef::ControllerOf` shape is exact |

---

## Engine — Rollback / Undo system (plan)

Two deliverables share one mechanism: (a) **transactional action
application** inside the engine — every rejected `GameAction` restores the
exact pre-action state, structurally killing the audit-P0 partial-mutation
family (Squad/Casualty under-pay, `declare_attackers` mid-loop corruption,
back-face land corruption, madness mana loss); (b) **player-facing
undo/take-back** — instant in single-player vs the bot (the main UX win),
consent-gated in multiplayer. The same checkpoint recorder later feeds the
replay scrubber (Client UX Tier 3) and crash recovery.

**Approach: whole-state snapshots, not inverse commands.** `GameState` has
a hand-written `Clone` (`game/mod.rs:859`) and full serde; the affordance
prober and bot dry-runs already clone the state per candidate action, so
the cost profile is known-acceptable. Inverse ops for a ~9k-line effect
resolver would be unmaintainable and would inherit every funnel-bypass bug
the audit found.

### Phase 0 — prerequisites
- ⏳ **Seeded, serialized RNG.** Shuffles call thread-local `rand::rng()`
  inline (`game/mod.rs:2462`, `4495`, `5968`, `7239`; grep for stragglers).
  Add `GameState.rng` (e.g. `Pcg64`, serde via seed+stream state) and route
  every random site through it — otherwise undo lets a player re-roll
  shuffles/flips until they like the outcome, and bit-exact replay is
  impossible. Fold in the audit-P1 coin-flip fix (`Decision::CoinFlip`
  must draw from this RNG, not constant heads) while touching it.
- ⏳ **Serde fidelity.** Not needed for in-memory undo (which uses `Clone`),
  but required before any persisted history/replay: fix the audit-P1
  `CardInstanceWire` six dropped fields + `TokenDefinition.static_abilities`,
  and land the property-based round-trip test (see Infrastructure →
  Snapshot Round-Trip Test).

### Phase 1 — transactional `perform_action` — ✅ DONE (2026-07)
- ✅ Checkpoint at the top of `perform_action`, restore on `Err` — for
  EVERY action, not just human-submitted ones: `GameState`'s heavy zones
  are now `CowBox`-wrapped (`crate::cow` — Arc + make_mut copy-on-write),
  so the checkpoint costs reference bumps and a failing action only pays
  for zones it touched before erroring. Affordance probes skip the
  checkpoint via `perform_action_inner` (their state is discarded either
  way). Regression: `cow::tests::rejected_action_restores_state_exactly`.
- ✅ Suspension is not failure: restore happens only on `Err`, and
  `GameError::ManualTapRequired` is exempted — it deliberately leaves
  forced pips auto-tapped + mana floating for the client's pending-cast
  driver (pinned by `sos::mana_shapes` tests). The restore keeps the
  *live* decider (the checkpoint clone holds a blank one; swapping it in
  would wipe a `ScriptedDecider` mid-script).
- Per-call semantics: a failed resume restores to the *suspended* state,
  not to before the original action. Full multi-step atomicity across a
  suspend/resume chain remains future work if ever needed.
- Keep the targeted P0 fixes anyway (validate-before-mutate is still
  better); the transaction is the backstop that makes the *class*
  unexploitable.

### Phase 2 — engine history ring
- ⏳ `UndoHistory { ring: VecDeque<(UndoPoint, Box<GameState>)> }` on the
  server-side game session (not inside `GameState` — snapshots must not
  contain the history). Push at decision boundaries: before each accepted
  human `GameAction` and before each `Decision` answer. `UndoPoint` carries
  seat + monotonic id + a human label ("cast Lightning Bolt", "declared
  blockers") for the UI.
- ⏳ Cap (e.g. 32 entries) and measure real `GameState` sizes; if memory
  matters, serialize+compress entries older than the last few.

### Phase 3 — server protocol + consent
- ⏳ Wire actions: `RequestUndo { to: UndoPointId }` /
  `RespondUndo { accept }` + a pending-request broadcast. On accept:
  swap in the snapshot, bump a view generation, re-broadcast full per-seat
  views (the existing per-seat projection path is the resync mechanism).
- ⏳ Policy: single-player undo is unconditional and instant. Multiplayer
  requires every opponent's consent. Bot policy: auto-accept (configurable
  later). Optionally restrict to "within the current priority window /
  before new hidden information was revealed" as a server setting.
- **Hidden-information stance (documented, not solved):** information a
  player already saw stays seen (the casual-play standard). The Phase-0
  seeded RNG guarantees a restored pre-shuffle state re-shuffles
  identically, so undo cannot be used to fish randomness; it *can* still
  be used to act on glimpsed information — consent is the mitigation.

### Phase 4 — client UX
- ⏳ Undo button + keybind, greyed when no eligible `UndoPoint`; opponent
  banner with accept/decline; game-log entry ("Eric took back: cast …").
  Supersedes the bare "Undo / Take-Back" stub under Client — UX.

---

## Bot / AI

### Instant-Speed Responses
~~The bot never responds to spells on the stack.~~ `pick_stack_response`
now counters an opponent's spell when it targets the bot's permanents /
the bot, or costs 3+ — cheapest affordable counter first, `would_accept`
dry-run as the final gate (so Spell Snare's MV filter etc. are honored).
Future: respond with removal/protection instants, not just counters;
race-aware "is this worth a card" valuation.

### Sacrifice Prioritisation
~~When forced to sacrifice, the bot always picks the first eligible
permanent.~~ Now sorts candidates: **tokens first, then by lowest CMC,
then by lowest power**. This is enforced inside `Effect::Sacrifice` so
both Innocent-Blood-style edict flow and forced sacrifices from
activated abilities see the same ordering. Future improvements:
respect "you may sacrifice" optionality (skip when the cheapest
candidate is more valuable than the payoff).

### Planeswalker Targeting
~~The bot never attacks planeswalkers.~~ Now redirects attackers at an
opponent's planeswalker when total attacking power can finish it off in
one swing (push claude/modern_decks `b34a23a`). Smallest-power-first
allocation keeps beefy attackers free to face-attack the player when the
walker fills up. Future improvement: handle chip attacks (attacking a
walker we can't finish but that's still threatening) and the inverse case
where a low-loyalty walker isn't worth committing trample beaters to
because the opp can clean up with a blocker.

### Smarter Mana Rock Usage
The bot taps mana rocks eagerly before knowing what it wants to cast.  A
"plan this turn's spending first" pass before mana-ability activation would
avoid situations where it taps a Sol Ring with nothing to cast.

### Multiple Difficulty Levels
- Easy: current random bot
- Medium: rule-based heuristics (responsive countering, threat assessment)
- Hard: Monte-Carlo tree search or minimax over the simplified game state

---

## Infrastructure / Dev

### Engine Test Coverage
Current test density is low outside `effects.rs` and card-specific unit tests.
Priority gaps:
- **Combat module** (`game/combat.rs`) has zero standalone tests.
- **Layer system** (`game/layers.rs`) — continuous effects, P/T ordering,
  timestamp tracking — has no dedicated tests.
- **Stack resolution ordering** — no tests for multi-item LIFO resolution,
  replacement effects, or trigger ordering.

### Snapshot Round-Trip Test
`GameSnapshot` and `GameState` serialisation exist.  Add a property-based test
that plays N random actions, serialises/deserialises the state, and asserts
game continuity — catching any `Serialize`/`Deserialize` drift.

### Card Correctness CI
`scripts/verify_cards.py` (with its Scryfall cache) verifies CMC, P/T, types,
and keywords.  Wire it as a CI step that runs against `scripts/.scryfall_cache.json`
(no network) to catch regressions when catalog entries change.

### Bot vs. Bot Simulation
Automate a "run 1 000 cube games bot vs. bot, report win rates by colour pair"
script.  Useful for catching degenerate card interactions and unbalanced pools
without manual play.

### Replay / Game Log Export
The server already collects `GameEventWire` events.  A replay file format
(sequence of `(action, resulting_state_hash)`) would enable post-game review
and deterministic bug reproduction.

### Scryfall Art Pre-fetch CLI
`all_cube_cards()` drives the in-game prefetch, but there is no standalone CLI
tool to warm the asset cache before a session.  A `cargo run --bin prefetch_art`
that downloads missing Scryfall images to the local cache would speed up first-
session load times.

### WASM / Web Build
`Cargo.toml` already has a `wasm-release` profile.  Completing the web build
(removing native-only dependencies, adding a WASM server bridge) would make
the game playable in a browser without installation.

---

## Formats

### Commander + Two-Headed Giant — phased rollout

Roadmap for the `Format::Commander` and `Format::TwoHeadedGiant` variants
already declared in `format.rs`. Strategy: build the multiplayer
foundation first (any-N seats, teams, opponent semantics), then add
shared resources for 2HG, then layer Commander-specific mechanics on
top. The `Format` enum entries currently only affect deck validation
and starting life; everything below is the runtime engine work.

**Status legend:** ✅ done, 🟡 partial, ⏳ todo.

#### Phase A — N-player game construction ✅
- Engine was already N-player aware (`pass_priority` uses
  `alive_count`, turn rotation uses `next_alive_seat`, attack target
  validation is bounds-checked).

#### Phase D — Multiplayer combat ✅
Each attacking creature chooses a defending player or a planeswalker
controlled by one of them; in 2HG the choice is the defending *team*
and damage may be assigned to either teammate's creatures/planeswalkers.

#### Phase E — Priority & APNAP for N players ✅
- Note: triggers within a single declare-attackers / declare-blockers
  batch (`game/combat.rs:50, 110`) share one controller (the active
  player), so APNAP within those is moot. The fix is concentrated on
  the unified dispatcher because that's the only fan-out path where
  multiple controllers can produce simultaneous triggers from one
  event.

#### Phase F — Shared life pool & shared turns (2HG) 🟡 (shared pool ✅; shared turn / cross-team triggers ⏳)
The 2HG-specific consumer of the teams abstraction.

**Shared pool — done:**

**Polish — done:**

**Still ⏳ (low-impact polish):**
- ⏳ Shared turn priority (CR 810.5) — strict "active team's primary
  player first, can yield to teammate" ordering. Current rotation
  is per-seat; both teammates already get priority in the
  4-passes-to-advance loop, so this is cosmetic.

#### Phase G — Team-aware loss & game end ✅
**G-lite done** (independent of Phase F):

**Shared-life half — now done via Phase F-3:**

#### Phase H — Replacement-effect framework (Commander prerequisite) ✅
- Known limitation (acceptable for Phase H scope): inline
  `graveyard.push` / `hand.push` / `exile.push` sites outside the
  three wired entry points bypass the resolver. Effects routed
  through `Effect::Destroy`, `Effect::Exile`-from-battlefield, and
  `move_card_to` all hit the wired paths; ETB-triggered direct
  pushes are the main gap and likely don't need replacement-effect
  coverage for Commander.

#### Phase N — Polish ⏳
- ⏳ Audit any remaining `PlayerRef::EachOpponent` / "your"/"opponent"
  effects in card catalog text for team-awareness (Phase C handles
  the engine layer; some cards may have bespoke logic).
- ⏳ CLI / deck-loader entry points should accept format.
- ⏳ Update format coverage tests after Phase J/K land.

---

#### Dependency graph
```
A → B → C → D → E
        ↓
        F → G   (2HG-specific consumers of teams)
        ↓
        H → I → J → K → L → M   (Commander mechanics on the multiplayer base)
```

#### Open design questions
1. **Partner / Background commanders** — in scope, or v2? `Deck.commanders:
   Vec<…>` accommodates either way.
2. **Brawl / Oathbreaker** — same machinery as Commander; opportunistic
   to plan in once L/M land.
3. **CR 810.5 priority timing within a team** — strict per-CR, or start
   with a simplified "active team's primary player has priority first,
   can pass to teammate"?
4. **Range of influence** — Commander uses unlimited (everyone in range).
   Default to unlimited; skip the option unless explicitly requested.

### Draft
- 8-player booster draft simulation
- Bot drafters with a basic pick-order heuristic
- Deck construction phase before play begins

### Sealed
- Generate 6 booster packs per player
- Deck construction phase
- Best-of-3 match support

### Brawl / Historic Brawl
- Lighter-weight commander variant (60-card, Standard-legal)
- Good stepping stone before full Commander

---

## Card Implementations (high-priority unblocked cards)

These cards are in the cube or demo decks and need only existing primitives —
no new engine features required:

Every row in this table has shipped (Bloodtithe Harvester's sac-a-Blood
ping, Dread Return's flashback sacrifice, Balefire Dragon's power-scaled
sweep, and Karn, Scion of Urza's real text included — earlier ⏳ marks
were stale). See git history for the per-card details.

## Simulation throughput

The recommender's dominant cost is `would_accept` dry-runs. Two
stepping stones landed (2026-07): the bot's per-tick candidate sweep
shares one library-stripped `affordance_probe_template` (a light clone
per probe instead of a full one), and the main castable block validates
*lazily* in descending score order at the pick site — a typical tick
probes 1-3 candidates instead of the whole hand. Match-template cloning
+ factory elimination in `simulate_match_games` was neutral — setup was
never the bottleneck. The big lever LANDED (2026-07): the heavy zones
(battlefield / stack / exile / per-player library / hand / graveyard /
command / sideboard / continuous_effects) are `CowBox`-wrapped
(`crate::cow`), so every `GameState::clone` — probes, probe templates,
`evaluate_action_outcome`, the `perform_action` transaction checkpoint —
is reference bumps plus only the zones the action actually mutates.
Remaining scaling comes from the racing schedule (`racing_rounds` +
small `games_per_pairing`) and, if ever needed, early adjudication of
stalled games via `eval_material`.
