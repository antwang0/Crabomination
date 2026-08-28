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

## NEXT (handoff — an INDEX. Every number lives in PERF; nothing here restates one.)

**FIRST:** `git fetch origin claude/modern_decks && git checkout -B
claude/modern_decks origin/claude/modern_decks` — the container clones `main`,
and sessions run concurrently: push code before tracker prose, rebase not force.

1. **Perf queue** — PERF "Perf candidates", ranked: **(-75)**'s unattributed
   rows (split each before threading it), (-70) (quiet window only), (-69)'s
   two unclaimed rows, (-51)(b), (-60), (-61), (-51)(a), (-59). Closed:
   (-71) and its sweep, (-73), the `CowBox<Vec<T>>` half of (-76) and six of
   its fields. Refuted **with numbers**: (-72), (-74), the ability-count
   reserve, three more (-71) sites, and **(-76)'s
   `affected_from_requirement` row, both halves** — it passed the byte test
   and failed the read-count test on its *consumer*.
2. **Perf method** — PERF "How to measure" and "Which pool a change moves",
   then the standing rules below. **Read all three pools**: pass 86 had one
   change that split by pool and one that did not. And **size an inline
   buffer before rejecting it**: the same three-field change is +0.137 % at
   one capacity and -0.463 % at another (the rule is under (-76)).
3. **Instruments before profiles** — `CRAB_SIM_REJECTS`, `CRAB_PAY_FAILS`,
   `server::bot_rejection_count`, `--bench`'s stall split.
4. **Encoding caution** — pool / `Vocab` / `TrainRow` / observation and deck
   encodings: a change **invalidates the trained nets**. Say so here. Pass
   86's `SmallVec` fields did **not** move it: the wire shape is a sequence
   either way, so no retrain.
5. **Bugs** — ENGINE_BACKLOG's live-match section; its sweep is a `0..4000u64`
   loop in the cube smoke test read through `bot_rejection_count()`, run
   **before and after**. Robustness gate: `-C debug-assertions=yes` on
   `[profile.overflow]`. Newly open there and both one-liners *given a
   resolving selector*: `Selector::BlockedAttacker` never resolves in an
   event filter (Righteous Indignation's colour clause is dead) and
   `EntityMatches` over `EachPermanent(…)` is an existence test whose empty
   case reads true (Tide Shaper's pump is unconditional). The sweep has not
   been re-run since the picker's off-board gate landed.
6. **ML** — deck judge 60.3 % pooled (ML_NOTES). Open, not unilateral: should
   `selfplay` seed `jitter_below` from `--seed`?
7. **Filters** — five read zero; `bot_rejection_count()` over the seeded cube
   sweep is the one that does not, because it watches the live-server path.
8. **Tip state** — PERF "Baseline"'s newest "STATE AT …".


## Standing rules for a perf pass

Durable, not per-run. Every refutation named here is written up in **PERF**'s
Log with its numbers; read the entry before re-proposing any of them.

**And when this file crosses ~1k lines, this section is the compaction** — it
is 380 of them and every rule is a refutation written up with its numbers in
PERF's Log, which is where the detail belongs. Collapse each to its
one-sentence claim plus the pass that measured it; do not delete one, because
the point of the section is that a rule refuted on a *mechanism* stays
refuted.

- **`Vec::clone` hands back `capacity == len`, so every `Vec` inside a
  copy-on-write structure reallocates on its first push after the copy**
  (pass 86, the concurrent half, (-76); `cube` -0.44 % over six sites). The
  `CowBox<Vec<T>>` half is closed centrally by an inherent `push` that
  materializes at `len + 1`; the plain-field half is per field. **Ask what
  else copies a `Vec` and then writes to the copy.**
- **The byte test is necessary and not sufficient — run the read-count test
  on the field's *consumer*** (pass 87; `affected_from_requirement`, both
  halves refuted). `AffectedPermanents::Specific` at `[CardId; 4]` is the
  same 24 bytes the `Vec` was and removes 30,534 allocations, and
  `affected_includes_gated` — the whole-board matcher that reads it once per
  (effect x permanent) pair — pays **+9.7 % of its own row** for them,
  because on a grant-heavy board the list spills anyway. `fixed` -0.212 % /
  `cube` +0.158 % / `sealed` +0.174 %: a pool split, and the field's own
  function is not where its reads are.
- **Size an inline buffer before rejecting it: `SmallVec<[T; N]>` is
  `8 + max(N * size_of::<T>(), 16)` bytes, so below 16 bytes of payload it is
  exactly the 24 bytes the `Vec` was** (same entry). The same three-field
  change reads `fixed` **+0.137 %** at `[_; 4]/[_; 8]/[_; 4]` and **-0.463 %**
  at `[_; 1]/[_; 4]/[_; 1]` **on an identical allocation saving** — the tax is
  the struct's bytes, priced at ~0.0009 % of `cube` a byte by (-74)'s padding
  probe. This **narrows (-72)**, which is the *read* count on `.players`
  (35,000 sites) and not a refutation of struct fields as such. Two tests, not
  one: read count first, byte count second.
- **A *returned* buffer is not disqualified from inline storage — the
  disqualifier is bytes moved** (same entry; `cube` -0.211 %).
  `statics_granted_triggers_inner` returns a `Vec` of *references*, so
  `SmallVec<[&T; 2]>` moves the same 24 bytes it always moved and the 0-2 case
  stops allocating. Read (-71)'s warning as arithmetic, not as a rule about
  ownership.
- **A definition-derived answer can be memoized on the object, and
  `CardInstance::DerefMut` is the invalidation** (pass 87; `fixed`
  -0.862 %, `cube` -0.700 %, `sealed` -0.940 %). `sba_board_scan`'s five
  per-card list walks are twenty-two bits in `CardMemo`, the atomic word
  that already carries the printed colours — one `clear()` store however
  many answers ride on it, and the read's `debug_assert!` is the audit.
  **This is also the counter-example the rule above needs: (-11) said such
  a cache "cannot be a lazily-cached field" because ~20 sites rewrite a
  definition through `Arc::make_mut`, and that was true until pass 83 built
  the chokepoint.** A refutation written against an *argument* dates; one
  written against a *measurement* does not. PERF's `(-77)` has the device
  and the three tests a candidate has to pass.

- **A "more exact" reserve is still a reserve, and `ContinuousEffect` is a
  large struct** (pass 86; `fixed` +0.461 %, `cube` +0.401 %, `sealed`
  +0.373 %). The gather sizes `all_effects` by a *card* count while
  `push_static_ability_effects` emits per *ability*, so sizing it by the
  ability count is strictly closer — and it is the fifty-fourth pass's
  `+ battlefield.len()` (+1.54 %) all over again, because most statics emit
  nothing and every extra slot is a kilobyte on 71,930 gathers. "Exact" in
  that entry means exact in *emitted effects*, which nothing cheap knows.
- **Compare a buffer's growth count to its *call* count before inlining it**
  (pass 86, `(-71)`'s sweep). A `grow_one` row says how often the buffer
  allocated; the `SmallVec` is paid on every call. The gather's `sa_cards` is
  69,896 growths over 71,930 calls (97 %) and shipped at `cube` -0.513 %;
  `statics_granted_triggers_inner` is 19,128 over **142,744** (13 %) and
  measured **nothing**, because a returned buffer costs a 40-byte move on the
  87 % of calls that never allocate. Reverted.
- **Inline storage is a *local's* device; on a struct field the read count
  pays for it** (pass 86, `(-72)`, built and refuted: `fixed` +0.600 %,
  `cube` +0.490 %, `sealed` +0.542 %). `players: SmallVec<[Player; 4]>`
  removed 22,684 allocations on `cube` — one per `GameState` clone, to the
  unit — and cost 16.5 M Ir, because `sa_cards` has ~forty read sites in one
  function and `.players` has **~35,000** across the workspace, each paying
  the `spilled()` compare. `dispatch_triggers_for_events` alone took
  +3.6 M Ir without touching an allocation. **Count the read sites before
  moving a buffer inline**, and do not retry it at a smaller inline capacity:
  the cost is per read, not per byte. **Confirmed a second time in the same
  pass**: `PlayerData`'s `spell_ids_cast_this_turn` and `spell_casts_this_turn`
  are `clear()`ed per turn and regrow only because the CoW deep copy's
  `Vec::clone` hands back `capacity == len`; inlining them removed all 22,930
  of `finalize_cast`'s growths — the largest `grow_one` row on `fixed` — and
  read `fixed` +0.366 % / `cube` +0.291 % / `sealed` +0.322 %.
- **A `SmallVec` without the `union` feature is an *enum*, and the
  discriminant match is the whole trade** (pass 86, `(-71)`; 0.12 % of
  `fixed`). Inlining the gather's `sa_cards` buffer read **+0.108 % on
  `fixed`** and -0.458 % on `cube` — a pool split — until the feature was
  turned on, after which it is -0.012 % / -0.513 %. Every read of a non-union
  `SmallVec` matches a discriminant on top of the `spilled()` compare, ~40 Ir
  per owner call. Two shapes measured and refuted alongside it: inline
  capacity 4 (removes the same growths, then pays 10,782 spill allocations)
  and shadowing the buffer with a `&[T]` after the fill (holding the borrow
  across 3,600 lines is worse on every pool).
- **The `grow_one` caller table ranks the local accumulators, and a row named
  for a function is not necessarily the buffer you think** (same entry).
  `gather_continuous_effects_inner`'s row on `fixed` is `all_effects`, not
  `sa_cards` — the four bench archetypes carry no permanent with a
  `static_abilities` entry, so that buffer never allocates on that pool, and
  the shipped change left `fixed`'s allocation table *byte-identical*. Read
  the row on the pool the change is aimed at.
- **A gate that rides on an existing *early-exiting* scan is not free**
  (pass 85, the concurrent half, `(-68)`; estimated 0.5-0.8 % of `cube` and
  measured 0.317 %). Three of `fire_combat_damage_triggers`' six battlefield
  walks were gated on facts the dealer lookup could compute on the way past —
  but that lookup was a short-circuiting `find`, so widening it to a full walk
  gave back part of the saving, and the arithmetic that priced the entry had
  not counted what the `find` was skipping.
- **When a pass deletes the work an abstraction existed to amortise, the
  abstraction is the next thing to read — and its doc comment will not tell
  you** (pass 85, the concurrent half, item 1; `fixed` -0.901 %, `cube`
  -0.806 %). `bot::ProbeCell` cached `state.affordance_probe_template()` for
  probes that each cloned it; the function had become plain `self.clone()`
  when a different pass deleted the library strip it existed to amortise, so
  the cached value was equal to the state it was made from. The cell was
  lazy, documented and carried real numbers — about work that no longer
  existed. **Grep for the shape, not the symptom:** `affordances.rs` still
  has ~12 of it.
- **Ask whether a function reads its parameter's *value* or its *support***
  (pass 85, the concurrent half, item 2; -58.5 % of `static_build_score`).
  `score_brief_with_colors` took a `ColorCounts` and touched it only through
  `is_empty()` and `get(c) > 0`, which makes its colour term a function of a
  five-bit set — and then `off = total - on` collapses two accumulators into
  one masked sum and the rest of the function into a memo field.
- **Memoize the pool, not just the cards** (pass 85, the concurrent half,
  item 3; `sealed_pool` -25.9 %). The card definitions had been memoized for
  thirty passes; `sos_draft_pool` and `SosPacks::new` rebuilt the *pool* made
  out of them per pool. **No profile row said so** — the cost was spread over
  three small ones. Ask what else in a prologue has no inputs.
- **Run a sweep on the range you found the bug in, before *and* after**
  (pass 85). A fix's own insurance was a second bug, on a seed the pre-fix
  sweep had passed; only the same 4,000-seed range re-run afterwards told a
  fix from a trade. **A sweep that runs only after the change cannot tell
  those apart**, and "the tests still pass" is not the same statement.
- **A thin candidate list is not an empty engine** (pass 84, and pass 85's
  concurrent half is three of four). Three of that
  pass's eight rows were on no candidate and in no self table —
  `Option::or_else`, the boxed keyword list, `printed_color_set` — and all
  three came from *counting* (call counts, allocations per call, a hot
  function's callee list), which is the three rules below this one.

- **Read the instruments before the profile, and know which kind of "no" you
  are holding.** `CRAB_SIM_REJECTS`, `CRAB_PAY_FAILS` and `--bench`'s stall
  split are counts of the *workload*, and the largest row in fifteen passes
  came from one of them rather than from a dump (PERF's Log, pass 83 item 0).
  **A "do not re-open" written against an *argument* dates the moment the
  evidence moves; one written against a *measurement* does not.**
- **A shrinking instrument is not a dead one** (pass 85 item 2). `--decks
  sealed --games 1` was retired in two sections of PERF for having fallen from
  2.9 G Ir to 21.9 M, and it is 76.5 % deck construction — the fall was five
  passes of work on the thing it measures. Read a candidate instrument's
  callee table before retiring it; an absolute is not a share.
- **Size a clone removal by the copy's whole lifecycle** (pass 85 item 0).
  (-65) was priced at ~0.2 % off `TriggeredAbility::clone`'s own row and
  shipped at 1.195 % of `cube`: the allocation, the `memcpy`, the `grow_one`,
  the drop and the `free` were all outside the row that named it, and the
  `clone` row was the smallest of the five.
- **After deferring work out of a hot path, re-read what the path still
  computes for it** (pass 85 item 1(d)). Moving land assembly off the shape
  lattice left two pool-sized vectors per shape being built, joined and
  dropped with no reader. **Removing a caller does not remove what fed it.**
- **Rank the dump by call count and read the Ir/call column** (pass 83's
  fifth commit, `fixed` -0.444 % / `cube` -0.568 %). `Option::or_else` was the
  most-called function in the program — 2,187,078 calls, all but 54 of them
  `evaluate_requirement_static_hinted`'s fallback chains, ~5 Ir apiece and
  invisible to a self table, a callee table and a line profile alike. A row
  with a million calls and single-digit Ir/call is pure call overhead, and the
  only question is which kind: a non-generic `crabomination_base` callee is a
  **profile artifact** (`release`'s thin LTO inlines it — the
  `CardDefinition::is_creature` trap), while a std generic the local inliner
  declined is **real**, and the fix is restructuring the call site, never an
  `#[inline]`.
- **Read a hot function's callee list and ask which rows are doing *work***
  (pass 83's eighth commit, `fixed` -0.173 % / `cube` -0.184 %).
  `compute_permanent_pass` makes three per-call definition reads;
  `base_power` and `base_toughness` are 13 Ir and `printed_color_set` is
  **56**, because it walks the keyword list, the colour indicator and every
  mana symbol. It is 32nd by call count and never rises above the noise in a
  self table. Three rows at the same call count with one an order of
  magnitude dearer is the tell.
- **A memo's invalidation point is priced by the *write* rate, and the
  tightest-looking key is not always sound** (same commit). Keying the colour
  memo on `Arc::as_ptr(&definition)` misses the MDFC face-swap and Mind
  Bend's override, which `Arc::make_mut` performs **in place** on a uniquely
  owned definition; `CardInstance::DerefMut` is the one point both must pass.
  It is also hot enough that the clear eats about half of what the hits save.
- **A redundancy you cannot remove without a refactor can still be priced by
  *adding another copy of it*** (pass 85, `(-70)`). `ComputedPermanent` holds
  four `Arc<CardDefinition>` handles to one definition; removing three is
  600-800 call sites, but *adding* three is a two-line field, and it reads
  `fixed` +0.532 % / `cube` +0.580 % — the change with the sign flipped. Pair
  it with the padding probe below and a struct's size and its handle count are
  both priced without touching a call site.
- **When a change trades a known saving against an unknown cost, build the
  cost alone first** (pass 83's seventh commit). Unboxing `layers::Printed`
  read +1.755 % and the narrower keyword-only version needed the *struct-size*
  half priced without the refactor: **a `u64` of padding on
  `ComputedPermanent`, two lines**, read `fixed` +0.040 % / `cube` +0.058 %,
  under the saving on both pools. That turned "do not take it on this
  arithmetic" into a decision in one build, and a linear extrapolation from
  the +208-byte data point would have over-priced it by 70 %.
- **A ceiling measured by short-circuiting a condition is an upper bound on
  the *code*, not on the walk** (pass 83's (-62), and it is 30 % of one).
  `false &&` and `std::iter::empty()` let the optimizer take the surrounding
  structure with it, so the graveyard pair priced at `fixed` -0.717 % by
  deletion shipped at -0.499 %. Both candidate explanations for the
  difference were built and measured **flat** — the memo's miss path
  (-0.003 %) and the memo read itself (0.027 % of the program; splitting it
  for inlining read +0.016 %). Read such a ceiling as "no implementation of
  this gate beats X" and do not spend a pass chasing the rest.
- **Before pricing a walk, grep for the other consumers of the fact it
  computes** (same entry, and it was 63 % more prize than the entry carried).
  (-62) sized `gather_continuous_effects_inner`'s `GraveyardAnthem` pass;
  `keyword_grant_in_scope` walks the same zone for the same variant and no
  entry had named it. Both match the variant by name, so the second walker
  was one grep away.
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
- **Do not rebuild these.** Unboxing `layers::Printed`'s override
  (`Option<Box<T>>` -> `Option<T>`; +1.755 % `fixed` / +1.317 % `cube`, and
  the narrower three-field version prices out worse than the boxes cost — the
  *keyword-only* `Box<[Keyword]>` is the one that shipped, see PERF's
  Baseline (7)),
  the board-presence epoch, the `GameState` husk
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
