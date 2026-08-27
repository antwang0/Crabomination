# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.


Split for size — this file is the working handoff, the archives below are
reference. All three carry an index table at the top; triaged at the
sixty-seventh pass, so don't re-take that.

- `ENGINE_BACKLOG.md` — engine backlog and audits, ordered **bugs /
  mechanics & primitives / rules coverage / tooling**: the correctness audit,
  the robustness filters, the decision-plumbing audit, missing mechanics,
  follow-ups not yet done, suggested next-up tasks, the CR coverage audit.
- `CARD_BACKLOG.md` — per-set card residuals: what each set still
  approximates, and the remaining gap lists. Titled by **subject**, ordered
  open-first, then the closed sets whose only content is residuals.
- `CLIENT_BACKLOG.md` — the Bevy GUI backlog (visualization / UX / refactor).
  The client **does** build here after four apt packages; that file's header
  has the command and the disk caveat.
- `PERF.md` — the perf record: baseline, log, profile of record, candidates.

## NEXT (handoff — rewrite each run, keep it terse)

**FIRST:** `git fetch origin claude/modern_decks && git checkout -B
claude/modern_decks origin/claude/modern_decks` — the container clones `main`
and `git branch -a` does not list this branch before that fetch. **Sessions
run this branch concurrently and this file is not a lock:** push the code
commit before the tracker prose; a fetch only rules out work already
*pushed*, so do not take the top-ranked item unless you can push within the
hour (pass 80 duplicated a whole item end-to-end), and diff before discarding
a lost race — the loser usually holds something the winner does not.

1. **Perf queue: (-56)'s last growth site SHIPPED at `31eb7333`** —
   `Printed<Vec<_>>`'s materialize reserved `len + 1`, `fixed` -0.085 %,
   `cube` -0.591 %, taken by a concurrent session from this entry's own
   pointer. That is the third reading of the `Vec::clone` hands back
   `capacity == len` trap and **the only one that pays on both pools, because
   it is the only one where the reserve is exact rather than headroom.**
   Three things in that entry are refuted with numbers, do not re-take them:
   the
   `sa_cards` reserve (both a whole-board and an exactly-counted one; it
   splits by pool and loses on `fixed`), a second freeze scope over the
   candidate menus (**+0.001 %** — `next_action` already freezes the whole
   tick, bot.rs:1661), and (-57)'s `eval_material` prize, which is one gather
   per evaluation and already at the floor. **And (-56b): the same
   function's `sorted` collect (189,480 calls / 46,426,567 Ir) is refuted
   four ways** — a twelve-slot stack buffer is the only one with a win and it
   reads `fixed` +0.151 % / `cube` -0.381 %, the same pool split. Durable
   lesson there: **`collect` is internal iteration and your loop is not**;
   replacing one costs the `Chain<Filter<_>>` specialisation, ~0.15 % of
   `fixed`, before it saves an allocation. (-52)/(-53)/(-54) closed;
   (-51)(a) wants a device on the do-not-rebuild list.
1e. **The three biggest unclaimed rows, all three seeded this pass with
   tables, and the biggest one is already answered.** **(-59)
   `dispatch_triggers_for_events` is the largest self row in the program —
   198,765,010 Ir / 5.58 % of `cube` / 139,500 calls — and its line profile is
   RUN (`profiling-lines` + `cg_lines.py`, in the entry): **there is no hot
   line.** Largest is 0.23 %; the per-card loop prologue is 0.44 % and the
   per-(trigger, event) loop 0.39 %, the rest thirty-odd million of iterator
   internals over a dozen narrow passes. The two loops a source read flags
   (graveyard, death snapshots) are not in the top forty. **Do not re-run that
   profile; the lever is fewer dispatches, and 114,834 of the 139,500 are
   `perform_action_inner` draining one action's events.** **(-60)
   `trigger_grant_sources` is 1.00 % of `cube` and finds 0.25 grants per
   call over 57,596 of them**; the CR 510.2 creature-damage batch's 12,858
   were hoisted this pass (`cube` -0.299 %, `fixed` -0.006 %), and
   `fire_step_triggers`' 23,526 are already one-per-call, so what is left
   there is *why 619 Ir*, not *how often*. **(-61) `keyword_grant_in_scope`
   is 1,713,848 `card_can_grant_keyword` calls / 1.67 % of `cube`** — but its
   largest site, the CR 702.64 Absorb gate, is **already a 4.5x trade** and
   the entry lists four routes off it that all fail. The one untried thing
   there is free and small: `activate_ability_inner`'s gates exist to avoid
   `bf_cp!()`, so `bf_cp.is_some()` short-circuits them exactly.
1c. **The gather is 30 % `resolve_combat`, the prize is `cube` -1.6 %, and
   the obvious route is REFUTED with a counterexample — see (-58).** Seeding
   the batch's per-pair freeze scopes from one gather measures **fixed
   -0.178 %, cube -1.615 %** with `--bench` byte-identical, and a
   `debug_assert!` auditing the seed against a fresh gather fired on `cube
   --seed 3` inside 60 games: a lifelink blocker gains life mid-batch and
   flips *Ulna Alley Shopkeep*'s "+2/+0 as long as you've gained life this
   turn" on. **Player life is a layer input**; no collection-length or
   timestamp epoch can see it, because the effect is derived and carries an
   old timestamp. What is left needs an invalidating memo (the board epoch,
   refuted at (-18)) or an incremental gather. **The device is the takeaway:
   a memo whose soundness is an argument gets a `debug_assert!` against the
   thing it replaces, and a `-C debug-assertions=yes` ladder run is the
   audit — 18,795 tests missed this, 60 games of `cube` found it in four
   seconds.**
1a. **⚠ The eightieth-pass block cost `fixed` +2.833 % and `cube` +1.712 %,
   and no commit in it recorded an Ir row.** Play is identical across it, so
   that is cost alone, and it is almost all one row: `computed_permanent`
   +57 % on `fixed`, whose largest new caller was `bot_can_block`. The
   eighty-first pass took about a fifth of it back (`legal_blockers`).
   **A correctness commit is a perf commit on the workload it runs in; report
   Ir on anything that lands in the picker, not just `--bench` decisions.**
1b. **The attack search is still the largest number in the pipeline and
   nothing has aimed at it.** PERF's **"THE ACTOR, at the eightieth tip"**:
   `pick_attacks_scored` **46.3 %** at 1.78 M Ir a decision,
   `main_phase_action_with` 27.9 %, allocation+copy 17.7 %, the whole encoder
   6.1 %. Profile actors at **60 games**, not 20. The sub-lever with a number
   on it is `sim_step -> perform_action`'s checkpoint: **9,628 clones /
   23.50 M + 9,628 drops / 15.11 M = 1.08 % of `cube`**, taken on a state the
   sim owns and throws away, to support a fallback whose failure rate the
   picker fixes have driven from 470/91,438 to ~14/8,610 on the one pool that
   still has any. **The atomicity proof this asked for is DISPROVED — see
   (-54b).** Both declarations pay costs mid-validation and reject afterwards
   (`declare_attackers_banded` at 1259/1313/1366 with `Err`s at 1306/1359/
   1387; `declare_blockers` at 1995/2022 with eight `Err`s after), so the
   checkpoint is load-bearing. Reordering all four cost families to select
   before any applies is **not** behaviour-preserving — the tax taps the
   lands a tap-another cost then looks for — so what is left is a CR 601.2h
   simultaneity question to price, not a deletion.
1f. **The actor is FLAT across this pass, and the base had moved — read
   PERF's "THE ACTOR RE-READ".** Same workload as the eightieth tip's
   profile, play byte-identical (32,402 `next_action`, 1,102
   `pick_attacks_scored`): recorded `a4b24308` 4,228,661,490 -> `be4a9987`
   **4,236,954,968 (+0.196 %)** -> `a828b393` **4,235,372,210 (-0.037 % vs
   base)**. Reading it against the recorded row instead of the real base
   would have reported a 0.16 % regression that does not exist. **A recorded
   total is a measurement of a commit, not of a branch — re-measure.** The
   row split is in the entry, and its finding is that **the attack-side and
   block-side walker unifications land on opposite sides here**: the attack
   one is a net win on the actor and on both ladder pools, the block one is a
   ladder win and ~+6.9 M on the actor, whose sealed boards are wider than
   `fixed`'s. Fifty-third pass's ranking rule, reappearing on the ML
   workload.

2. **Encoder is mined out** — passes 77-80, `encode_state` -49.4 %, actor
   -6.1 %, three refutations in PERF bounding what is left. A new lead there
   needs a fresh profile, not a list.
3. **(-55): the block half IS now closed, and "effectively closed" was a
   three-seed sample twice over.** A sweep of `cube` 1-24 + 42 at `--games
   20` found **186 block rejections across eight seeds** where the sampled
   census read 6 — four rules nothing had reached (`AllMustBlock` true Lure,
   blocker-side `MustBlock`, CR 509.1g `CantBeBlockedByMoreThanOne`, and a
   board with *no legal declaration at all*). Now **6, on s15 alone**;
   `all` s15/s23 also went 4 -> 0 and 8 -> 0. **Rule, and it is cheap:
   sweep 1-24 (~90 s at `--games 8`) before writing that a half is closed.**
   What is left: the **attack** half's `combat.rs:1114`, the tax's
   `available_mana` optimism — the (-51)(b) question, not a missing rule
   (cube s2 32, s11 10, s19 16, s20/s21/s22 12) — and the 6 on s15, which
   need a probe naming the *pass that built the plan*: the site tag names
   the clause that rejected it and two plausible fixes measured inert
   before a hand-built probe found the cause. **Generalisable finding:**
   a "must" and a "can't" written as two independent checks can be jointly
   unsatisfiable, and only the census finds it — CR 509.1b now gates CR
   509.1c through `block_requirement_binds`; ENGINE_BACKLOG P3 lists the two
   remaining pairs to audit.
4. **Anything that moves play is gated by `bot_ladder --vs PATH`:** run the
   null first (a byte-identical copy must read 50.0 %, every pair split),
   same `--a`/`--b` both sides, 1.9x wall. **Sweep `CRAB_SIM_REJECTS=1` over
   seeds before concluding a pool cannot reach your code** — `cube` deck
   content is seed-dependent and the sweep is ~4 s a seed.
5. **Encoding caution:** any change to the SOS/cube pool, `Vocab`,
   `TrainRow`/`EncodedState`, or the observation/deck encoding **invalidates
   the trained nets**. Say so prominently in the commit and here.
6. **Bugs:** ENGINE_BACKLOG P3's requirement-walker item is **closed for
   combat, both sides.** The attack side is one walker (`attacker_self_block`
   / `attacker_target_block` / `attacker_is_able` / `may_declare_attacker`)
   and so is the block side (`blocker_self_block` / `blocker_pair_block`, with
   `block_requirement_able` and `blocker_can_block_anything`/`_pair` as
   compositions of the two, and `bot_can_block` delegating). Each hid the same
   deadlock — a creature *required* to act and then *rejected* for acting, so
   the seat had no legal declaration at all. On the block side the drift also
   ran the other way: **seven `CantAttackOrBlock*` families (hand size,
   delirium, a creature died, Descend N, the city's blessing, cards in exile,
   Hollow Warrior) and Space Beleren's sector lock were enforced only in the
   bot's mirror**, so on the real declaration path those cards' blocking
   restrictions did nothing. Eleven tests in `cr_recent100` between the two
   sides. What is left in P3 is `evaluate_requirement_static` vs
   `evaluate_requirement_on_card`. P2 has no open correctness entries. Both audits clean at this tip —
   `audit_stubs` 0/21,795, `audit_incomplete` 0 needing review, and dead
   modes are now suite-gated against `audit::REVIEWED_DEAD_MODES`.
6a. **NEW ROBUSTNESS GATE, and it is one RUSTFLAG.** `release-fast` (and so
   `overflow`, which inherits it) has `debug_assertions` **off**, so the
   documented overflow sweep never reaches a single `debug_assert!` — and the
   suite does not either, because an assertion needs a *board* to fire on and
   18,795 tests carry fewer interesting boards than 60 games of `cube`. Adding
   `-C debug-assertions=yes` to the overflow build turns the ladder into an
   audit of every engine invariant at once. Recipe in Cargo.toml's
   `[profile.overflow]` comment; eighty-first pass reads **34,560 games over
   five pools x six seeds, no panic, no assertion, no overflow, 0
   undecided**. It is also what refuted (-58) in four seconds. **Run it after
   anything that adds an invariant.**
7. **State, re-run at `05015235` on an Intel Xeon @ 2.80 GHz
   (`host_calib_ms` 51-57 — a different host from the 224.6 and 297.7 rows
   this item used to carry, so those figures do not compare):**
   `--workspace --exclude crabomination_client` **19,063 / 0 / 5**; clippy
   `--workspace --all-targets` clean **including the client** (four apt
   packages, ~40 s; free `target/debug/incremental` first, 11 GB here); 7
   golden traces unmoved; `--bench` **195,616 decisions / 27.44 turns / 0
   stalls**, `determinism ok`, `thread_determinism ok (3 vs 1)`, **170.4 /
   175.3 / 171.7 games/s**. `--vs` null against a byte-identical copy:
   fixed 200/200, cube 400/400, sos 250/250 pairs split. `overflow`, seeds
   11/12 over `all`/`cube`/`sealed` at 600 games/archetype: **44,400 games,
   0 capped, 0 stuck, 22 draws, no panic, no arithmetic overflow** (measured
   at the eighty-first tip). **Re-run at `27af76f4`**, which does touch
   arithmetic-adjacent code (the CR 510.2 hoist): `overflow`, seeds 11/12/13
   over `all` plus seed 11 over `cube`/`sealed`/`sos` at 300 games/archetype
   — **22,800 games, 0 capped, 0 stuck, 4 draws, no panic, no arithmetic
   overflow**; and `release-fast --decks all --games 400 --seed 11`, 6,800
   games, 12 draws, all 3,394 pairs split.
7a. **`peak_rss_mib`: the step does not reproduce at `27af76f4`, and three
   runs is why.** The reading above (27.3-27.6 at `60cfef4c` -> 29.0-31.4 at
   `05015235`) was one run a tip. Three `--bench` runs back to back at
   `27af76f4` on a 2.10 GHz Xeon read **27.8 / 29.3 / 27.5** — one sample
   lands in each of the two "bands", and an earlier run at `c1450677` read
   29.5. **The spread is within-tip variance, not a step**, so nothing here
   needs owning. The transferable bit is the one this file already says about
   games/s and had not applied to RSS: **an allocator reading is a
   distribution; take three before you call a difference.**
7b. **Second host on the same day, for the wall-clock table:** 2.10 GHz Xeon,
   `host_calib_ms` 65-71, **299.8 / 300.1 / 307.9 games/s** at `27af76f4`,
   against the 2.80 GHz box's 170-175 at `host_calib_ms` 51-57. That is a
   *faster* games/s at a *worse* calib on a *slower* nominal clock, which is
   item 8's point made a third time: `host_calib_ms` fingerprints the host, it
   does not scale between them.
8. **Hazards.** ⚠ Wall-clock rows do not cross hosts — read `host_cpu` /
   `host_calib_ms` off the run before comparing any games/s in PERF; Ir is
   unaffected. **A third host, and it settles that `host_calib_ms` cannot be
   used as a correction factor:** a 2.80 GHz Xeon reading **173 games/s at
   `host_calib_ms` 49** — the *same* calib the eightieth tip recorded 262.2
   at. Two builds went into ruling that out as a regression; the check that
   ends it is an A/B in one sitting (a binary built from the recorded tip
   read 175.3 here against the current tip's 173.5), never a scaling
   correction. If a games/s row looks wrong, build both sides now rather
   than reasoning from the fingerprint. ⚠ A container reset wipes `target/`, removes `cargo-nextest`
   and checks the repo out on the *system-prompt* branch: commit each
   measured change as soon as it measures, re-run FIRST after any surprising
   `git status`, and reinstall with `curl -sSLf
   https://get.nexte.st/latest/linux | tar xzf - -C ~/.cargo/bin`. ⚠ Disk:
   free `target/debug/incremental` (7-15 GB) before a client clippy.
9. **Deck net:** `nets/deck-champion.safetensors`, gated **60.3 % pooled**
   over 19,200 games against the static judge, so `--use-deck-best` has a
   committed judge. Write-up in ML_NOTES ("Deck-net re-gate").
10. **Open question for a run with the ML context:** should `selfplay` seed
   `jitter_below` from `--seed`? It would make training runs replayable and
   `--games N` fixed work, and it removes per-actor tie-break diversity. Not
   to be changed unilaterally.

## Standing rules for a perf pass

Durable, not per-run. Every refutation named here is written up in **PERF**'s
Log with its numbers; read the entry before re-proposing any of them.

- **A number a sweep reports needs its breakdown at the same call site**, or
  the sweep only generates a follow-up question. The undecided count is the
  case: `SimCost::record` split capped / stuck / draw the moment a game
  ended, and only `--bench` printed the split, so a robustness sweep on
  `--decks` reported a bare "20 undecided in 44,400" and answering the one
  question that matters — rules outcome or broken loop? — cost a rebuild.
  They were all draws, i.e. **zero capped and zero stuck**, a much stronger
  result than the bare count looked like.
- **A share is a ratio whose denominator you chose.** Read the absolute Ir
  next to the percentage: one is about your code and the other is about
  everyone else's. Two ways this has bitten — a fixed startup cost that reads
  4.34 % at 20 games and 1.40 % at 60 with *identical* Ir (PERF's actor
  profile), and (-53)'s share falling a third further than its work did
  because the program grew 2.63 G -> 3.57 G around it.
- **Ask what the answer costs when it is "no"** (pass 59, ~1.5 % of `sos`
  across four sites). None of them was a hot function: a SipHash of ~84
  small integers for a digest compared only within one process, an
  iterator stack collected into an always-empty `Vec`, a `flat_map` over two
  always-empty command zones, a battlefield `filter` for a card no bench deck
  contains. A presence question is paid on every sweep or dispatch whether or
  not it can fire. `cg_edges.py --callers SpecFromIterNested` **ranked by
  calls, not Ir** is the table that finds them; PERF's (-44) has the rest.
- **A line profile's row is what that source construct's instructions cost,
  not what removing it would save** (pass 61, +0.12 % and reverted).
  `cg_lines.py` put `is_event_hardcoded`'s `match ev` at 0.38 % of `sos`
  inside the biggest engine row; replacing it with per-event bitmasks made
  the function 1.9 M Ir *slower* and moved nothing else in the program,
  because the loop still had to branch per event. **Ask what the loop still
  does when the line is gone** before costing the row.
- **A presence bit belongs in a shared scan only when the question has no
  early exit of its own** (pass 59, +0.29 % on `fixed` and reverted). Folding
  `card_type_change_unscoped`'s battlefield leg into `sba_board_scan` cost
  more than the walk it removed, because the standalone `any` short-circuits
  per card and a scan bit has to finish. Third loss for the (-6) fusion
  device inside `creature_death_possible` alone.
- **This branch is rebased constantly, so a hash in a doc is a liability**
  (pass 58, three stale ones in one session — `223c77b5` twice, and a
  write-up that named its own commit before the rebase renamed it). Cite a
  hash only for a commit that is *already* on `origin`; for the commit the
  paragraph is describing, name it by title, and re-grep for the short hash
  after every `rebase --continue`.
- **A memoized object is not a memoized *answer*** (pass 54, and it was
  -68.8 % of the deck build). `card_def` had cached the `CardDefinition`
  since pass 53, and the builders still re-derived everything they read off
  it — the pip counts, the mana value, four `Vec<CardType>::contains`
  scans, the keyword walk in `card_quality`, the whole effect-tree walk in
  `is_fixing_card` — per (pick x candidate x colour shape). `CardBrief` is
  where a new derived fact goes; the memo is sound because a leaked
  definition is never mutated.
- **Callgrind runs the system allocator and mimalloc ships** (pass 54). An
  allocation-shaped change reads larger in Ir than it measures: -68.8 % of
  the deck build was +8.5 % of `selfplay_train`'s throughput. Get the
  `selfplay_train` number before sizing the next allocation-shaped
  candidate, and alternate — the same base binary read 129.1 and 156.5
  games/s minutes apart, so quote best-of-N and never a single run.
- **Ask which pool the change lives on** (pass 53, and it is the rule that
  found the two largest costs in the simulator). `--decks fixed` carries no
  `GrantTriggeredAbility` static and builds its decks once, so the per-card
  grant walk and the whole deck builder are dead on the bench. A change to
  statics / grants / layers / the requirement walker gets a `--decks cube`
  reading too; a change under `draft.rs` / `recommend.rs` / `selfplay.rs`
  gets `--decks sealed --games 1`, which plays no games and so isolates deck
  construction exactly. PERF's "Which pool a change moves" has the recipes.
- **The freeze scope that pays is the one around a loop whose borrow already
  proves it** (pass 53). Three per-card grant walks each hold a shared borrow
  of `self.battlefield` for their whole body, so no `&mut self` call can
  happen inside — which is exactly the freeze-scope invariant, checked by the
  compiler rather than by hand. Bare `freeze_layers_push`/`pop`, not
  `with_frozen_layers` (the closure costs ~0.9 % because the loop's locals go
  through its environment), gated on a fact the loop already computes.
- **Rank the tail, not the function** (pass 49). A chain of narrow generators
  is invisible in a self-cost profile and in a callee table sorted by Ir; it
  shows up only in the **call counts** — twenty-two rows at exactly 2,176
  calls each, once per traversal, on a board with nothing for any of them,
  4.9 % together. Wherever the code reads as a fallback chain, count the rows
  before costing them. The device is `spec` / `gated_block!`'s and the debug
  audit is what makes a mask safe to over-approximate.
- **Ask what a tick pays when the answer is "nothing to do."** Four of pass
  49's five rows are that question at a different level: a
  twenty-two-generator fallback chain, three land blocks asking the same
  question, a gate that gathered to prove a negative, a freeze scope opened
  for a closure that returns immediately, two clones handed to a walk that
  skips.
- **Build the answer *after* asking whether anyone wants it** (pass 55,
  eight of its ten commits, -20.7 % of the cube pool between them). A gathered
  layer view a presence gate answers; a requirement tree cloned to build a
  residual the caller discards on 99.3 % of calls; a CoW zone unshared to
  restore flags that never moved; a `battlefield_find` for the card the
  caller is iterating. **In five of the eight the cheap question was already
  a function in the file** — `requirement_mentions_power`,
  `creature_type_change_in_scope`, `evaluate_requirement_static_on`. Grep
  for the cheap form before writing one.
- **Ask a `collect()` how often it is *empty*, not how big it is** (pass 55,
  the two commits that pay on the bench pool). An empty `collect()` still
  calls `Vec::from_iter`; two per gather and two per state-based-action
  sweep were -0.7 % of `--decks fixed` between them. The worklist is
  `cg_edges.py --callers __rust_alloc` ranked by call count, then `grow_one`
  one level up.
- **Rank a lazy cell by what is under it, not by its own row** (pass 55, and
  it was 16.9 % of the cube pool). Candidate (-37) was sized at
  `computed_permanent`'s 4.14 % + `compute_permanent_pass`'s 2.97 % — self
  cost. Read from the top, the requirement walker's `OnceCell::try_init` is
  **413,844 calls / 605,927,621 Ir inclusive, 15.03 %**. `cg_edges.py
  --callers <callee>` is the table; the self table cannot see a cell.
- **Two shapes that look like wins and are not** (pass 55, both measured and
  reverted). A `OnceCell` around a presence gate that runs *once* per call
  is two constructions and a branch that never pay (+1.24 M on `fixed`) —
  `evaluate_requirement_static` evaluates one `req` and its arms are
  exclusive. And asking the cheap question first — `!computed_absent()`
  before the gate — is **+0.066 %**, because inside a freeze scope the gate
  is a memo hit at 69 Ir and the atomic load is not free.
- **The gate rule, both halves.** Swap a gather for a presence gate **only
  where nothing else in the scope reads the gather** (pass 48's (E),
  -0.747 %); where the scope goes on to `compute_battlefield()` the same swap
  is **+0.30 %**. `layers_memoized()` answers "is the gather already built".
  Fusing a cheap per-card question into a walk that already happens has lost
  four times; removing a walk outright still pays.
- **The `Keyword::eq` device is not exhausted, and it has a trap.** No LTO
  here, so any small non-generic `crabomination_base` function is an
  out-of-line call in every profile this file quotes — but a bare `#[inline]`
  would be unmeasurable in the shipped `release` (thin LTO) build, so **do not
  take one on an Ir number**. What works is making the callee smaller than any
  inliner threshold, which is what `has_kw` does.
  `CardDefinition::is_creature` is the same family and the same trap.
- **A presence gate in front of a loop is only free if the loop was not
  already empty** (pass 57, and it is the one thing the second session
  measured that the first did not). The gather's thirty-eight `sa_cards`
  passes: gating by swapping the iterated slice read **+1.076 % on `--decks
  fixed`**, a board branch outside the loop +0.551 %, and the same test moved
  *inside* the loop (`if mask & bit == 0 { break; }`) **+0.234 %** — while
  taking cube from -1.85 to -2.26 % and sos from -2.75 to -3.16 % at the same
  time. `fixed`'s `sa_cards` is empty on all 32,002 gathers, so the walks were
  already free there and anything in front of them is pure charge.
- **Read the Ir/call column of a caller table, not just the calls or the
  total** (pass 60, -2.9 % of `sos` in one commit). `__memcpy` is 7.80 % of
  `sos` over forty diffuse rows — except `CardInstance::new`, 3,452 calls at
  **8,242 Ir each**. A memcpy costing eight thousand instructions is moving
  kilobytes, and `size_of::<CardDefinition>()` is **8,232**: every deck-fill
  site handed `CardInstance::new` a fresh `f()` and `Arc::new` copied the whole
  definition per card in a library.
- **A memo whose miss path is expensive is not a free memo** (same pass). The
  first version of `card_arc` rode `card_brief`'s memo, so a miss also paid the
  pip counts, the keyword walk and `is_fixing_card`'s effect-tree walk:
  **+6.591 % on `--decks sealed --games 1`**, which is all misses and no games.
  Its own memo reads +0.330 % there. Quote the *cold* workload for a memo.
- **The Ir does show up on the clock, and it over-reads by ~2x** (measured
  2026-08-25, and it is the first time anyone asked). Passes 57-59 together,
  `28ae2416` -> `49c7220d`, eight ABBA blocks a pool with a flat null control:
  **`sos` -6.57 % Ir / -3.87 % clock (8/8 blocks), `cube` -8.91 % / -3.16 %
  (7/8)**. So rank work by callgrind — deterministic, thirty times cheaper,
  and it found every one of those commits — but **halve an Ir delta before
  quoting it as throughput**, and expect a single commit under ~3 % of Ir to
  be unseparable on this box's clock.
- **Best-of is a biased estimator, and every clock number in this file before
  pass 59 was one.** `scripts/ab_wall.py` runs an ABBA schedule (linear host
  drift cancels inside a block), reports the mean per-block ratio with a 95 %
  t CI, fingerprints `decisions` on both sides and refuses to time two
  binaries that played different games. **Run its null control
  (`--bin-a X --bin-b X`) at the same block count before believing a
  verdict.** Calibrated on the routine box: `--games 2000 --decks sos
  --threads 4`, eight blocks, **+/-2 % and nothing finer** — four blocks
  called a null-equivalent result significant.
- **`Ir` counts a `memcpy`; the machine barely does** (`cae6b605`, and it is
  pass 57's clock rule with its mechanism named). Replacing
  `granted_abilities_of`'s deep-copied `Vec<ActivatedAbility>` with
  `Vec<&ActivatedAbility>` — 11,324 `ActivatedAbility::clone` and 11,324
  `__memcpy` a six-game run, gone — reads **-1.946 % on `sos`** and is **flat
  on the clock over nine alternated 20,000-game pairs**, under *both*
  allocators. A deep copy of a contiguous struct runs at high IPC out of a
  just-written cache line; the borrow that replaces it turns a hot-buffer read
  into a pointer chase into cold definitions. Keep such a change for its Ir and
  its clarity, but do not quote it as throughput.
- **A `Vec` returned to a caller that immediately drains it is two
  allocations, not one** (pass 57's (D) and (E), -1.4 % of cube between them
  on top of the mask). `static_ability_to_effects` collected a `Vec` per
  static-ability card and `static_effect_to_effects` built `vec![one]` per
  emitted effect; both were `extend`ed into `all_effects` one frame up. The
  tell is in the callee table, not the self table: `SpecFromIterNested::
  from_iter` and an `IntoIter::drop` at **exactly the same call count**. Write
  through the caller's buffer (`out: &mut Vec<_>`) and patch
  `out[start..]` where the caller was patching the temporary.
- **A presence gate is sized by its arm's call count, not by what the arm
  costs when it is taken** (pass 56, one gate paid and two lost in the same
  sitting; pass 55 closed the same entry by sizing rather than building and
  agrees). `creature_type_change_in_scope` was worth taking out of the mutex
  because `HasCreatureType` is **410,900 of the requirement walker's 654,950
  calls** on cube; `HasArtifactSubtype` / `HasSupertype` are rare, and gates
  for them measured **+0.123 %**. Count the arm before writing the predicate.
- **A gather is one per freeze *scope*, not one per unscoped read** (pass 56,
  and it closes three rows of the contexts table that look like candidates).
  `computed_permanent` gathers when the scope's memo is empty — its first
  computed read — so N gathers in a context is usually N scopes each paying
  for itself. `pick_blocks` and `eval_material` are both that shape.
- **`cg_sites.py`'s number is a floor, and there are two data points.** The
  auto-tap source table read 0.15 % of `fixed` in that table and measured
  **-0.291 %**; pass 53's two sites read 0.35 % and measured -0.611 %. Do not
  decline a site because its `cg_sites` row looks small.
- **A wall-clock number for the deck build carries the process floor with
  it** (pass 56). `--decks sealed --games 1` is ~6 ms of build on a ~3.3 ms
  startup floor, so quote both columns; the tip's floor was 6.5 % *higher*
  than the base's on a bigger binary, and that came straight off the measured
  win. A training actor never pays it — it builds decks in one process.
- **Ask what varies with the shape** (pass 56, and it was -23.1 % of the deck
  build in five commits). The sealed lattice runs ~57 shapes over one pool;
  the pack buckets, each card's brief, the pool's pip totals and each card's
  score are all properties of the *pool*, and each was being rebuilt per
  shape. `PoolScores` is where a new per-pool derived fact goes.
- **Measurement.** Read PERF's "How to measure" — pass 48 rewrote it.
  `scripts/cg_symbolize.py` + `scripts/cg_edges.py`, never
  `callgrind_annotate --tree` for a caller table. **`cg_edges.py`'s shares
  were ~18x low until `ac85463f`** — a table read before that commit ranked
  work upside down; re-derive anything carried forward from one. Re-read your
  own base: on a shared branch the commit you stand on may not be the one the
  last pass measured, and argv length lands in the Ir total (~500 Ir).
- **Measurement gotcha, hit three times.** The 6-game ladder printout (24
  decided / 12 splits) does **not** move on a change that breaks something;
  only `--bench`'s `decisions` and the Ir total catch it. Run
  `./target/profiling-fast/bot_ladder --bench --threads 3` on any change whose
  Ir moves more than its blast radius allows.
- **Ir over-reads a commit that removes whole action executions by about
  two** (pass 75's clock, `ab_wall.py`, 8 ABBA blocks, `release-fast` +
  mimalloc, `CRAB_NO_JITTER=1` both sides, `--games 2000 --decks sos --seed 11
  --threads 4`): **-1.29 %, CI -2.19 .. -0.39, 6/8 blocks** against a FLAT
  null (+0.20 %, CI -0.70 .. +1.10, resolution ±0.90 %), where Ir read
  -2.775 %. The direction is the opposite of a *clone* removal (wall bigger
  than Ir, pass 68) and of an allocator swap (Ir blind to it, pass 64).
  **Under ~2 % of Ir will not show on this box's clock at all**, so do not
  promise a wall number for one.
- **`selfplay_train --seed N` does not reproduce a run** (pass 77). Same
  binary, same seed, `--actors 1`: 1,788 / 1,770 / 1,776 rows over twenty
  games; with `CRAB_NO_JITTER=1`, 1,788 every time. `bot::jitter_below` falls
  back to the *thread* RNG unless a seeded stream is installed, and
  `set_jitter_seed` is the ladder's device for antithetic pairs — nothing in
  the actor path calls it. **Pin the jitter for any actor-path measurement**;
  an unpinned reading compared a base that had played 1 % fewer rows.
- **Four standing measurement facts**, kept here because each has cost a pass
  once. **Callgrind Ir is portable across these containers** — four
  independent readings of one commit on four boxes agree to 0.0004 %, and the
  whole difference is argv length — so another session's Ir column is a usable
  base; its wall-clock and RSS columns are not. **A change whose soundness
  rests on a `debug_assert!` is audited by the `dev`-profile grid, not the
  `overflow` one**: release profiles compile the assertion out. **Plan actor
  counts off ~24 MiB RSS**, not the `--no-default-features` 17.7 — mimalloc is
  the shipped allocator and costs ~9.7 MiB a process. And **`--decks fixed` is
  the bench pool**: a change to statics / grants / layers gets a `--decks
  cube` reading too, and one under `draft.rs` / `recommend.rs` / `selfplay.rs`
  gets `--decks sealed --games 1`.
- **A refutation carries the workload it was measured on, and this branch's
  workload moves.** The seventy-sixth pass re-opened `can_afford_in_state`,
  closed at +0.066 % on 2026-08-12, and it paid -0.6 % on all three pools: the
  entry's own figures — 0.29 % of the profile in the walks, 1.13 cards per
  sweep — had become 1.14 % and 2.80, because the attack search now runs 1,910
  sims a `cube` run and every one of them sweeps. **Re-read a refutation's
  numbers against the current dump before trusting its verdict**, and record
  numbers rather than verdicts so the next re-check is one `cg_edges.py` call.
  This does *not* license re-taking anything under "Do not rebuild these"
  below: those are refuted on a mechanism, not on a ratio.
- **A wrong bot pre-filter is invisible to every invariant this file checks.**
  It costs Ir, not correctness, so a green suite, identical golden traces and
  a flat ladder all survive it indefinitely. The tell is the *ratio* between
  what the bot offers and what the engine completes — `restore_payment_state`
  against `try_pay_after_snapshot_mode`, `finalize_cast` against
  `cast_spell_with_convoke`. Grep the other pre-filters (`ward_tax_payable`,
  `pick_combat_trick`, `max_affordable_x`) for the presence-vs-count shape the
  seventy-first pass found in `available_mana`.
- **The oracle, and use it again.** A bot-side estimate of a rules question
  usually has an engine function that answers it exactly (`could_pay_cost`,
  `would_accept_on`). Wire it behind an env var at the *divergence* site,
  report only where the old estimate would have said yes, and sweep pools x
  seeds. At the seventy-first pass the count went **6 -> 6 -> 240 -> 0** and
  every non-zero named the card that found the hole; **the first two versions
  of that commit looked correct and were not**, and no reading of the code
  found what the oracle did.
- **Do not rebuild these.** The board-presence epoch, the `GameState` husk
  pool, gating `do_untap`, narrowing `GameState`, splitting the big engine
  files for build time, the per-definition keyword-grant bit, fusing
  `card_type_change_in_scope`, the `LayerFreeze` depth shadow, the
  `sba_board_scan` definition bitmask, the trigger-carrier bitmask, the APNAP
  rank table, the headroom-reserving `Vec`, `board_keyword_matching`'s
  presence gate, presence gates for `has_atype` / `has_stype` (pass 56,
  +0.123 % cube), and (-31)'s `improves_this_turn` reuse. And **never** skip
  `push_ordered_trigger_candidates` on an empty batch (+7.3 % *and* a
  correctness bug — it owns the per-batch `died_card_snapshots.clear()`).
- **Env.** `cargo-nextest` **is** installable in this image and this bullet
  said the opposite for twenty passes:
  `curl -sSLf https://get.nexte.st/latest/linux | tar xzf - -C ~/.cargo/bin`
  takes seconds, and `cargo nextest run -p crabomination -p
  crabomination_tests` runs the gate in **~110 s** after the build against
  `cargo test -j 2`'s ~25 minutes from cold. Cold everything (deps + catalog +
  engine, two profiles in parallel on 4 cores) is **~45 min a profile**; a
  warm engine-only rebuild is ~15 min, and a change in `crabomination_base`
  costs the catalog too, so **batch base-crate edits**. Workspace
  clippy needs `apt-get update && apt-get install -y libwayland-dev
  libasound2-dev libudev-dev libxkbcommon-dev`. Cold `profiling-fast` engine
  build ~14 min, warm rebuild ~4m30s; callgrind ~4 min and contention-immune.
  Wide-pool sweep ~55 s a seed — no excuse to skip it. Quote callgrind under
  5 %; a `profiling-fast` games/s compares to nothing, and a clock A/B needs
  `scripts/ab_wall.py` with eight blocks *and* its null control (~35 min).
- **Trackers.** TODO 1.0k, ROADMAP 0.66k, PERF 6.0k (**passes 45-49's
  Baseline blocks are one table plus the lessons they carried, and passes 45
  and 46's Log entries are folded, at the 58th tip**; the 48th's and 49th's
  Log entries are the next fold). ENGINE_BACKLOG **3.8k**, CARD_BACKLOG
  **4.0k**, CLIENT_BACKLOG 0.4k — all three triaged and indexed at the
  sixty-seventh pass; see NEXT item 10.

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

## Engine — Robustness / defects

**No open entries.** The determinism class, the panic/unwrap census and the
twenty-three robustness filters are all closed; each filter is a *shape* that
fails the way a training run notices (a silent wrap at game 400 k, a loud
panic, a hang), and nine of them found nothing, which is the result worth
keeping. **The whole record — what each filter hunted, what it found, and the
rule it yielded — moved verbatim to `ENGINE_BACKLOG.md` at the fifty-fourth
pass.** Read it before proposing a robustness sweep; re-deriving a closed
filter is the failure mode it exists to prevent.

Standing goals that outlive it: no panic/unwrap reachable from bot self-play;
cross-process determinism at a fixed seed (golden traces assert it, and
`CRAB_THREAD_CHECK=1 --bench` asserts it across thread counts); stall rate
tracked on `--bench` (`stalls_by cap / stuck / draw` — re-open only if `cap`
or `stuck` goes non-zero).


## ML — defects (index)

### Every committed deck net fails to load — FIXED for the future, not for those nets

**The code half is the fifty-fourth pass's freeze; the artifact half
closed 2026-08-24 (next paragraph).** `VOCAB_SNAPSHOT` froze
the embedding index at the fifty-fourth pass, `pad_vocab` zero-extends a
shorter table and `--use-deck-best` runs end to end at 91.7 % of the unjudged
rate. The seven committed `*/deck-latest.safetensors` predate the freeze, so
nothing can say which card each of their rows meant; `vocab_fit` refuses them
by name and they need retraining. `nets/champion.safetensors` is unaffected.
**The full write-up — what the bug was, what the fix does, what is
deliberately not fixed and why, and the out-of-range clamp found alongside it
— moved verbatim to `ML_NOTES.md` at the fifty-seventh pass.**

**Resolved 2026-08-24 for the artifact, not the class.** No retrain was
needed: the deck stream rides along in every `selfplay_train` run, so the
recent run directories already hold vocab-164 deck nets. Two were re-gated
at the round-11 shape (`.ladder/run_deck_regate.sh`, 800 games × 12 pools
per cell, seeds 43/97): `nets_r45_ctrl_s43` 59.6/61.1, `nets_r41_v7_s43`
59.7/61.5 — pooled 60.3/60.6, a statistical tie on the historical 60–62
band. The r45 artifact (newest champion-class run) is committed as
`nets/deck-champion.safetensors`; `--use-deck-best`, `--gate-builder-hc`
and `--distill-gen` are live again. No retraining "for those nets" was
ever needed — the run directories held current-vocab spares all along.
The structural class is closed by the freeze above; this artifact is at
the frozen size (164), so `vocab_fit` accepts it and future card
additions only pad it.

## Engine — Missing Mechanics

Moved verbatim to `ENGINE_BACKLOG.md` at the sixty-second pass, when this
file passed the ~1k-line trigger again — it is a backlog of unimplemented
mechanics, which is what that file is for. Nothing is closed by the move.

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

### Phases 0 and 1 — ✅ shipped
Seeded serialized `GameState.rng` behind every "at random" (so undo cannot
re-roll a shuffle and replay is bit-exact); persisted-history serde fidelity
is still gated on the `CardInstanceWire` dropped-fields fix and the round-trip
property test (Infrastructure → Snapshot Round-Trip Test), which in-memory
undo does not need. Then a checkpoint at the top of `perform_action`, restored
on `Err`, for **every** action, pinned by
`cow::tests::rejected_action_restores_state_exactly`. Three semantics worth
remembering, because each was a bug first:
- Suspension is not failure. `GameError::ManualTapRequired` is exempted —
  it deliberately leaves forced pips tapped and mana floating for the
  client's pending-cast driver (pinned by the `sos::mana_shapes` tests).
- The restore keeps the **live** decider; the checkpoint clone holds a
  blank one, and swapping that in wipes a `ScriptedDecider` mid-script.
- A failed *resume* restores to the suspended state, not to before the
  original action. Multi-step atomicity across a suspend/resume chain is
  future work if it is ever needed.

**Perf:** the checkpoint is the engine's largest single structural cost —
`-5.47 %` of the whole program on the bench workload, and it is *never*
read there (see PERF's forty-third pass and candidate (-13)). Any narrowing
of it is a rules-correctness argument first and a perf change second.


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
race-aware "is this worth a card" valuation. Round 43: the buff 2-for-1
(`buff_2for1`, kill the creature under the opponent's own pump) is
built and zero-incidence in bot mirrors — human-facing, default off.

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
walker fills up. Round 43: the chip candidate exists (`walker_chip`, one declaration at
the lowest-loyalty unfinishable walker, sims judge it) — zero-incidence
in the walker-free sealed gate pools, so it stays default off on the
strength of the recorded ten-turn-ultimate loss, not a ladder number.
Still open: the inverse case (a low-loyalty walker not worth committing
trample beaters to).

### Smarter Mana Rock Usage
The bot taps mana rocks eagerly before knowing what it wants to cast.  A
"plan this turn's spending first" pass before mana-ability activation would
avoid situations where it taps a Sol Ring with nothing to cast.

### Encoder / net follow-ups — moved to `ML_NOTES.md`
The belief-head recall diagnostic, the encoder-v7 gap list, the
static-anthem P/T hole, the feature-occupancy precondition and the
prioritized next-round candidate list live in **`ML_NOTES.md` → "Encoder and
net follow-ups (moved from TODO, 2026-08-23)"**, verbatim. They are long
experiment narratives and they exist so nobody re-derives the dead ends.
One of them is load-bearing for *this* branch: **static-anthem P/T is
invisible to the encoder**, and a modern-decks pool is exactly the pool
change that makes it the top encoder item.

### Multiple Difficulty Levels
- Easy: current random bot
- Medium: rule-based heuristics (responsive countering, threat assessment)
- Hard: Monte-Carlo tree search or minimax over the simplified game state

---

## Infrastructure / Dev

### Engine test coverage — the old gap list is stale, and the real finding is a ratio

The three "priority gaps" this section listed (combat, the layer system,
stack ordering) all closed without anyone striking them out: `core_rules`
alone carries 75 combat tests in `combat_keywords.rs`, 57 that cite CR 613 /
layer ordering / timestamps, and `golden_trace.rs` compares whole games
action-for-action, which is the only thing that catches a reordered
iteration. Checked and removed at the fifty-fourth pass rather than left to
be re-derived.

**What is open is the pure-data sweep, and it is smaller than it looks.**
`scripts/find_data_tests.sh` finds **174 non-sacred pure-data tests** (plus
25 marked `[CR]`, which are sacred) across 62 files — asserts that only echo
`CardDefinition` shape, the largest cluster being `stx/part_23.rs` (19),
`modern/lands_equipment_vehicles.rs` (11), `core_rules/format.rs` (11) and
`classic_sets/rna.rs` (11). Folding them into one table-driven audit per set
is the standing rule.

**Do not justify it on build time.** 174 tests at ~8 lines is ~1,400 of the
suite's **375,047** lines — 0.37 % — and PERF's "Build time" section already
measured a 537-line cleanup as inside the noise band, because the rebuild is
dominated by relinking the integration binaries. The justification, if a run
takes it, is maintenance shape only.

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
and deterministic bug reproduction. Partially covered: `CRAB_REPLAY_DIR`
(`server/replay.rs`) logs the event stream per match, and
`CRAB_DECISION_LOG` (`server/decision_log.rs`, 2026-08-18) logs every
*human* action beside what the heuristic bot would have done from the
same position — one JSONL line per decision with an `agree` flag and a
disagreement tally in the footer. That second log is the bot-debugging
instrument: sort by `agree:false` and read the disagreements (it's how
converge-blind payment would have been caught from game data). The
viewer's first tier exists (2026-08-18): replay files are v2 — each
line carries first-appearance card names, since wire events hold only
ids and a file has no live state to resolve them — and
`cargo run -p crabomination --bin replay_view [file] [--all]` narrates
one as readable prose (newest file under `$CRAB_REPLAY_DIR` by
default; `--all` includes the mana/tap noise). Still open: an in-client
replay mode driving the real renderer with step/seek, and state-hash
checkpoints for deterministic reproduction.

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

**Status legend:** ✅ done, 🟡 partial, ⏳ todo. Phases **A, D, E, G, H** are
shipped — N-player construction, multiplayer combat, APNAP priority,
team-aware loss/game-end, and the replacement-effect framework. Git history
carries the per-phase detail; only what is still open is listed here.

- **Phase F — shared turns (2HG)** 🟡. Shared life pool and its polish are
  done. ⏳ CR 810.5 shared-turn priority ("active team's primary player
  first, may yield to teammate"): rotation is per-seat today and both
  teammates already get priority inside the 4-passes-to-advance loop, so
  this is cosmetic.
- **Phase H — known limitation, accepted for the phase's scope.** Inline
  `graveyard.push` / `hand.push` / `exile.push` sites outside the three
  wired entry points bypass the replacement resolver. `Effect::Destroy`,
  `Effect::Exile`-from-battlefield and `move_card_to` all hit the wired
  paths; ETB-triggered direct pushes are the gap, and likely do not need
  replacement coverage for Commander.

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

**`PERF.md` is the record — baseline, log, profile of record, candidates.
Nothing here duplicates it.** What this section keeps is the one structural
fact the rest of the file leans on: the heavy zones (battlefield / stack /
exile / per-player library, hand, graveyard, command, sideboard /
continuous_effects) are `CowBox`-wrapped (`crate::cow`), so a
`GameState::clone` — probe, probe template, `evaluate_action_outcome`, the
`perform_action` checkpoint — is reference bumps plus only the zones the
action mutates. The sharp edge that follows from it, and the one that keeps
producing perf rows: **any `&mut` access unshares, including an `iter_mut`
used read-only or a write that changes nothing.**

**The second structural fact, added 2026-08-24: an actor's per-game work is
not all simulation.** `selfplay_train`'s `actor_loop` calls `sealed_pool`
twice and `heuristic_sealed_build` twice **per game** — 32 candidate builds
a side under `--deck-judge` — and until pass 53 that was ~485 M Ir a game
against ~48 M for the game itself. `cube::card_def` memoizes
`CardFactory -> CardDefinition` and it is now ~20 M. Measure deck work with
`bot_ladder --decks sealed --games 1`, which plays no games at all.

Remaining scaling levers that are *not* engine instructions: the racing
schedule (`racing_rounds` + small `games_per_pairing`); the deck builder's
own residual ((-39) in PERF); and, if ever needed, early adjudication of
stalled games via `eval_material`.
