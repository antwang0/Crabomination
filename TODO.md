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

**FIRST COMMAND:** `git fetch origin claude/modern_decks && git checkout -B
claude/modern_decks origin/claude/modern_decks`. The container clones `main`,
and **`git branch -a` does not list this branch before that fetch** — so an
orient-yourself `git branch -a` reads as "it doesn't exist", `git checkout -b`
off `main` then builds and tests green 2,500 commits behind, and nothing says
so until the push is rejected.

**Sessions run this branch concurrently, and grepping the Log for a numbered
candidate is not enough when the entry is a table.** Two sessions took
`restore_payment_state` in the same hour at the sixty-eighth/sixty-ninth
passes — same let-chain, same line — because both were reading the top
Ir/call row of the `make_mut` caller table that (-43) points at. **Push the
code commit before you write the tracker prose**; that is the only signal the
other session's next fetch can see. **It happened again at the seventy-first
and seventy-second passes**, three hours apart in the same hour: one session
wrote item 1e's "grep the other pre-filters for the presence-vs-count shape"
while the other was three edits from writing the same per-colour budget.
Fetch before you start a candidate, not just before you push.

0. **Seventy-second pass: `fixed` -0.209 %, `sos` -0.183 %, `cube` -0.218 %**,
   two commits, both (-50). (a) The leave-the-battlefield chain's four no-op
   writes — `temporary_control`'s `mem::take`, `on_left_battlefield`'s
   `continuous_effects` pair, `remove_effects_from_source`,
   `expire_end_of_turn_effects`. (b) `blocked_attackers` /
   `blocks_declared_this_turn` out of `ColdState` into `GameState`, where
   their combat siblings already live.
0a. **Item 1c below was a mis-attribution and is now closed.** It said
   `on_left_battlefield`'s `make_mut` edge came from `find_card_anywhere_mut`;
   the callee table puts that function in its **own un-inlined row at 1.000x**,
   so it was never on the edge, and the seventy-first pass's +0.083 % was a
   correct measurement of the wrong hypothesis. **When a `--callers` row and a
   `--callees` row name the same function, the edge is the caller's own
   inlined code** — run `--callees` on the owner first.
0b. **The `ColdState` no-op-write vein is worked out**, and the reason to
   believe it is one short table: `cg_edges.py --callers "crabomination::game
   ::GameState as core::ops::deref::DerefMut"` is now 19,048 calls in 17 rows,
   3,020 of them real copies at 4,410 Ir, and every top row writes a value
   that changed. **Do not go hunting another cold no-op write.** Two rules
   priced in cash there: rank a `make_mut` edge by **Ir/call, not calls** (the
   14,152 calls gated out of `on_left_battlefield` were 25 Ir each and the
   5,232 left were 1,016), and before moving a field out of the cold group,
   **name the next cold write in the same frame** — `note_creature_death`
   absorbed 6.27 M of the 11.7 M that `declare_blockers` gave up.
0c. **What is left of (-50) is not a no-op write.** `make_mut`'s own copies
   are **146,820 / 108.4 M / 4.12 % of cube** and they are zone `Vec`s and
   cards. The sized piece: `on_left_battlefield`'s remaining 5,232 x ~1,016 Ir
   (**0.20 %**) is the CR 400.7 `cast_from_*` reset writing a card a probe
   clone shares — real, so it wants the reset done **where the placer still
   owns the card** (the callers have already unshared it), not a gate. Seven
   call sites and a stale-flag failure mode; see (-50).

1. **Seventy-fourth pass: `fixed` -0.081 %, `sos` -0.201 %, `cube` -0.596 %**,
   one commit — the colour budget reaches `sink_facts`, the presence mask that
   gates the whole `gated_pick!` ability chain. Activations reaching payment
   1,242 -> 996 on cube, every one of the 246 a rollback. **Two rules, both
   measured.** (a) *A gate is only cheap where what it reads is already paid
   for.* `main_phase_action_with` now owns one `SweepMana` for
   `cast_candidates` and `sink_facts` both, and even so an unconditional
   `have.get()` per ability read **+0.292 % of `fixed`** (whose abilities are
   `{T}` or generic, so the forced `available_mana` bought nothing — its
   rollbacks did not move). Testing the *printed* cost for a coloured pip
   first decides whether the read happens at all, and it is free. (b) *A
   widening must be widened to something the estimate does not also
   under-count.* Pass 71 widened `by_color` to `[total; 5]`, and `total`
   under-counts exactly the sources that force the widening — two Treasures
   and nothing else read `total = 0`, so the "unbounded" budget still
   rejected every coloured pip while the engine sacrificed one and paid.
   `u32::MAX` is what it meant. **Found by the oracle, third time.**
1. **Seventy-third pass: `fixed` -0.135 %, `sos` -0.456 %, `cube` -0.195 %**,
   one commit — the seventy-first pass's per-colour budget applied to the
   candidate blocks that had no pre-filter at all. **`restore_payment_state`
   at `--separate-callers=2` is the map**: of 2,960 rollbacks on `cube`, the
   pre-filtered cast path is the *best* of the six at 26 %, against
   `activate_ability_inner` 59 % and `cast_flashback` 67 %. Everything the bot
   proposes other than a cast reaches the engine on a ~50 k-Ir
   `would_accept_on` probe alone. `colors_coverable` is the drop-in half of
   the budget — **colour pips are the one part of a cost nothing in the
   engine's adjustment machinery moves** (every activation and graveyard-cast
   adjustment is `reduce_generic` / `add_generic`, `{X}` only adds pips, a
   coloured tax only adds them), so it is sound against a *printed* cost with
   no effective-cost computation. Next blocks to take it to:
   `cast_spell_alternative` (36 %) and `cast_face_down`. And note
   **`w.ability_arms` is off in every shipped profile**, so the 59 % is not
   reachable from the block that block filters — it comes through the
   `usable_abilities` pick_* helpers, and threading a *shared* `SweepMana`
   through `main_phase_action_with` is what that needs (a per-helper
   `available_mana` is ~3,000 Ir and pass 40 already refuted the eager read).
1. **Seventy-first pass: `fixed` -0.398 %, `sos` -1.363 %, `cube` -1.225 %**,
   one commit — the biggest single commit since the sixty-third pass and the
   first in ten that is not a presence gate. `AvailableMana` answered "is
   there a producer for this colour" where the payment funnel asks "are there
   *enough*", so `{G}{G}` off a lone Forest passed the bot's pre-filter and
   was thrown away at payment. A `[u32; 5]` budget (Hall's condition on the
   singleton colour sets) built in the walk `available_mana` already takes:
   cast attempts 7,110 -> 6,038, payment rollbacks 3,696 -> 2,716, dry-run
   probes 11,986 -> 10,910, **completed casts 4,720 -> 4,720 byte-identical**.
1d. **The reusable half is the oracle, and this branch should use it again.**
   A bot-side estimate of a rules question has an engine function that answers
   it exactly — here `could_pay_cost`, which runs `try_pay_with_auto_tap` on a
   clone. Wire it behind an env var at the *divergence* site, report only
   where the old estimate would have said yes, and sweep pools x seeds: the
   count went **6 -> 6 -> 240 -> 0** and each non-zero named the card that
   found the hole (Choreographed Sparks — `would_accept_on` accepts a
   *suspend*, use `could_pay_cost`; Crystalline Crawler — a mana ability with
   a counter cost and **no `{T}`**; Dryad of the Ilysian Grove — CR 305.6
   land-type rewrites reach `mana_source_table` and not
   `granted_abilities_of`). **The first two versions of that commit looked
   correct and were not.** Refuted on the way: deriving the budget from
   `untapped_mana_colors` is exact and costs 6,690 Ir a call against a ~4,600
   Ir win.
1e. **A wrong bot pre-filter is invisible to every invariant this file
   checks** — it costs Ir, not correctness, so a green suite, identical golden
   traces and a flat ladder all survive it indefinitely. The tell is the
   *ratio* between what the bot offers and what the engine completes; grep the
   other pre-filters (`ward_tax_payable`, `pick_combat_trick`,
   `max_affordable_x`) for the same presence-vs-count shape.
1. **Seventieth pass: `fixed` -0.399 %, `sos` -0.282 %, `cube` -0.394 %**, one
   commit — `attack_static_scan`, the third `*_scan` bitmask, on
   `declare_attackers_banded`'s four gateable static walks (two of them per
   attacker). **And one refutation that is worth more than the commit: the
   same device on `declare_blockers` reads `sos` +0.006 % and was reverted.**
   A `*_scan` bit is worth the walks it removes *from a loop*; the attack
   side's run once per attacker, the block side's once per declared blocker,
   and the bench pools declare far fewer blockers. **Count the loop's trips
   before writing the bit.** Second rule from the same commit: **a site is
   gateable iff it tests `sa.effect` directly** — `active_static` peels
   `WhileYourTurn`-style wrappers, so a raw-variant mask would miss a wrapped
   one; two of the six walks are ungated for that reason and re-gating them
   means a second copy of `active_static`'s wrapper list.
1a. **A rules commit landed on top of that tip and costs Ir**: the deck-out
   fix (item 5) reads `fixed` +0.055 %, `sos` +0.012 %, `cube` +0.082 %
   against `7ada03d9`, one flag read per seat per SBA sweep, `decisions`
   byte-identical and traces unchanged. **A base column taken at `4f42c6b4`
   is ~0.06 % above one taken at `7ada03d9` — that is the trade, not a
   regression.**
1c. **CLOSED at the seventy-second pass — see item 0a.** The 19,384-call edge
   was the `continuous_effects` pair and `temporary_control`, not
   `find_card_anywhere_mut`; the surviving 0.20 % is item 0c.
1b. **Where the device still has sites.** `cast_cost_scan` covers six of the
   nine its own function asks (the sixty-eighth pass's item, still open), and
   nothing has been scanned in `check_state_based_actions`, the layer pass or
   `resolve_combat`. The test is mechanical: grep a hot function for
   `static_abilities`, count the walks, count the loop trips around them.
2. **Sixty-eighth pass: `fixed` -1.032 %, `sos` -0.888 %, `cube` -1.879 %** in
   Ir, four perf commits plus one bug fix, all "what does this cost when it
   has nothing to do" —
   and **2-3 % of wall on `cube`** over two independent ABBA sittings, 11 of
   12 blocks, null flat. **The wall win is bigger than the Ir win**, which is
   the reverse of this branch's usual caution: the largest commit removes
   `Arc` deep copies, and a clone's cache misses are wall-expensive and
   Ir-cheap. **Size a clone-removal pass on the clock.** New candidates
   **(-50)** (the no-op CoW write — the class *and* its ranking rule) and
   **(-49)** (`wants_ui`, 0.07 %, wants the decision-plumbing audit's eye).
2a. **Sixty-ninth pass: `fixed` -0.130 %, `sos` -0.130 %, `cube` -0.296 %**,
   two commits, both **(-50)** at the *zone change* rather than the payment
   rollback, measured base `795a296e` -> tip `8147836b`. The rule they yield
   is in (-50): **a (-50) site is a chain, not a line.** Gating five of the
   zone-change chain's six writes moved an 8.5 M-Ir edge and the *program* by
   -0.050 %, because `send_to_graveyard`'s `counters.clear()` two frames down
   absorbed it; gating that too landed -0.221 % of cube. Gate from where the
   object is handed over to its last touch, in one commit — an intermediate
   step reads as +0.022 % on a pool. **The tell that finds a site:
   `cg_edges.py --callees <fn>` on a `make_mut` caller row, looking for a
   callee count that is an exact multiple of the function's own call count**
   — 2.000x is an unconditional line, a ragged ratio is the board width.
   **No wall-clock pair was taken for it** and the pass above says why that
   is a real loss: batch the next two or three (-50) sites and price the
   batch with `ab_wall.py` rather than paying the ~35-minute setup per
   commit.
2b. **Where the next one is, and it is not in a profile: read the three lines
   under an existing `*_scan` call.** Four of this pass's five commits were a
   whole-board question asked beside a mask that could have answered it —
   `cast_cost_scan` still covers only six of the nine its own function asks.
   Then: **the cast-failure lead is half taken** (item 1) — attempts are
   6,038 and rollbacks 2,716 now. **What is left of it is the generic half**:
   a per-colour budget cannot see a shortfall that is generic rather than
   coloured, and no sound generic bound exists from `ManaSourceInfo`, which
   carries colours but not amounts. The other half is **(-51)(a), a land tap
   at 7,555 Ir over 21,566 taps — 6.19 % of cube** and the second-largest
   call site in the simulator; it has never been costed and its obvious lever
   (a cheaper `keyword_grant_in_scope`) needs the per-definition keyword-grant
   bit, which is in the do-not-rebuild list. Also open: the layer pass's
   `printed_color_set`, **194,610 calls / 11.7 M / 0.44 % of cube**, one per
   pass (caching it on `CardDefinition` is blocked by the ~20 in-place
   definition mutations — see (-11)). Do **not** re-take the `sorted` `Vec`
   (item 4), and do not hoist `trigger_grant_sources` out of the combat
   damage loop — it is already once per damage event, and the remaining 1.7x
   would have to survive a rider resolving mid-loop.
3. **DONE — the `clone_from_ref_in` context table is in PERF's Profile of
   record** (`--separate-callers=2`, `cg_contexts.py`, tip `ee376912`).
   157,402 real deep copies against 806,878 `make_mut` calls, and the two
   tables rank differently. **The reusable column is the clone/ask ratio**:
   65 % (`activate_ability_inner`) means the first `&mut` after the checkpoint
   really is the first write and (-50) has nothing to gate there; 8 %
   (`do_untap`) means the handle was already unshared. The same block records
   that **the self table has stopped saying anything new** and that
   `dispatch_triggers_for_events` line-profiles diffuse at this tip (largest
   engine line 0.23 %) — do not spend another `profiling-lines` build on it.
   `cg_ratio.py` still ranks pool outliers: `affected_includes_gated` is
   **6.63x cube/sos and 0.46 % of cube**, i.e. the sixty-fourth pass's layer
   gate does not fire on a grant-heavy board.
4. **Refuted, do not re-take:** call-site guards on
   `clear_summoning_sickness` (the method is an inherent `impl CardInstance`
   one — its own guard is *not* dead) and gating
   `auto_tap_for_cost_inner`'s `wants_ui` pair (it is **true** in every
   measured workload); both left their `make_mut` edges byte-identical. And
   **skipping `compute_permanent_pass`'s collect** by sorting at the gather:
   `fixed` **+0.173 %**, `sos` **+0.208 %**, reverted, number in the code at
   the collect. **A gathered effect list is ~2 long, so that `collect` was
   never allocating** — check an allocation table's row *is* an allocation
   before removing it. And **the `*_scan` bitmask on `declare_blockers`**
   (`sos` +0.006 %, seventieth pass) — the shapes are identical to the attack
   side's and the loop is not. And **the multi-colour half of Hall's condition**
   in the bot's affordability filter (seventy-fourth pass): the singleton case
   is the one that pays, the subsets **rejected nothing at all** over the bench
   workload and cost `fixed` +0.105 %, `sos` +0.104 %, `cube` +0.107 %. The
   reason is worth keeping: **the widenings that keep the budget sound switch
   it off on exactly the boards that would violate a subset** — a Treasure, a
   filter land or a land-type rewrite makes the board `bounded = false`, and
   those are the boards with interesting mana. And **the `*_scan` bitmask on
   `do_untap`** —
   the forty-third pass built it and it read **+0.0001 %**: each of the six
   walks short-circuits on `definition.static_abilities.is_empty()`, so six
   specialised `any`s beat one general pass. `do_untap` is 1.55 % of cube and
   **none of it is in those walks** — read its callee table, not its walk
   count. Older refutations are in PERF's standing rules.
5. **Bugs. ENGINE_BACKLOG's P2 has no open correctness entries now.** The
   deck-out item is fixed: `pending_deck_loss` is armed by the failed draw and
   promoted by the SBA sweep (CR 104.3c), which also closed the half nobody had
   noticed — `objects_leave_with_player` runs only for the seats the *sweep*
   eliminated, so a decked player's board stayed on the battlefield forever
   (CR 800.4a). Two rules to keep from it: **a bug whose only symptom is a
   missing SBA leg is invisible to every test that does not run the sweep**,
   and **`two_player_game()` seats empty libraries**, so any test whose subject
   draws is decking itself — `game::stock_libraries(&mut g, n)` is the harness,
   and one of the nineteen tests it fixed had been passing vacuously with a
   dead ETB draw. Earlier bug of the same reading-the-profile kind: the payment
   snapshot keyed on `owner` where auto-tap taps by `controller` (`86ec1bd8`),
   and **the `perform_action` checkpoint hides that whole class** — a rollback
   bug is only observable where the failure is *handled* rather than
   propagated. Still open: ENGINE_BACKLOG P3's picker/checker disagreement
   (27 single-slot bodies aim with one filter and are checked against another).
6. **Housekeeping.** TODO **~850**, PERF **8.8k**,
   ENGINE_BACKLOG 3.8k, CARD_BACKLOG 4.1k, CLIENT_BACKLOG 428. Suite
   **19,009 passed / 0 failed / 5 ignored** — that figure is the workspace
   less the client; the two-crate gate this file prescribes builds 14
   binaries and reports **18,749**, and the other five crates hold 260.
   `cargo-nextest` **is** installable in this image
   (`curl -sSLf https://get.nexte.st/latest/linux | tar xzf - -C ~/.cargo/bin`,
   a few seconds) and runs the two-crate gate in **104 s** after the build,
   against `cargo test -j 2`'s ~25 minutes from cold; the note elsewhere in
   this file saying there is no nextest is stale. Both
   numbers are in PERF now, because the two commands disagree and nothing
   said which one the record quoted. 7 golden traces. Next PERF Log
   folds are the 51st/52nd. This NEXT was 262 lines before this pass; every
   item it dropped is in PERF's Log/candidates, ENGINE_BACKLOG or
   CARD_BACKLOG, not deleted.
7. **Two standing measurement facts** (the rest are in PERF's "How to
   measure"): plan actor counts off **~24 MiB** RSS, not the
   `--no-default-features` 17.7; and `--decks fixed` is the *bench* pool —
   a change to statics / grants / layers gets a `--decks cube` reading too.
   **A change whose soundness rests on a `debug_assert!` is audited by the
   `dev`-profile grid, not the `overflow` one**: release profiles compile
   the assertion out. **Callgrind Ir *is* portable across these containers**
   — two boxes read the same commit within 0.0004 % at the sixty-ninth pass —
   so a concurrent session's Ir column is a usable base; its wall-clock and
   RSS columns still are not.

## Standing rules for a perf pass

Durable, not per-run. Every refutation named here is written up in **PERF**'s
Log with its numbers; read the entry before re-proposing any of them.

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
- **Env.** No `cargo-nextest`; `cargo test -j 2 -p crabomination -p
  crabomination_tests` is the gate (18,728 / 0 / 5, 11 binaries running tests, at the
  fifty-sixth tip; ~25 min from cold). Workspace
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

**Still open, and it is a training run, not code.** `VOCAB_SNAPSHOT` froze
the embedding index at the fifty-fourth pass, `pad_vocab` zero-extends a
shorter table and `--use-deck-best` runs end to end at 91.7 % of the unjudged
rate. The seven committed `*/deck-latest.safetensors` predate the freeze, so
nothing can say which card each of their rows meant; `vocab_fit` refuses them
by name and they need retraining. `nets/champion.safetensors` is unaffected.
**The full write-up — what the bug was, what the fix does, what is
deliberately not fixed and why, and the out-of-range clamp found alongside it
— moved verbatim to `ML_NOTES.md` at the fifty-seventh pass.**

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
