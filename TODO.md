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
claude/modern_decks origin/claude/modern_decks` — the container clones `main`.
**Sessions run this branch concurrently: read PERF's Log before starting the
top candidate, re-read the base after every rebase, and budget two callgrind
rounds per commit.**
**And two sessions took (-48) in the same hour this run, each spending two
11-minute builds and forty minutes of runs on it.** Nothing stops that: both
read "the highest-value fresh item" and both had disk. **Before starting a
named candidate, `git fetch` and grep PERF's Log for its number** — that is
the whole mitigation, it costs one command, and it would have saved this run
an hour. If you take one anyway, say so in the Log as a replication rather
than rewriting the entry.

0. **The sixty-seventh pass: (-45)'s sibling table run, two commits.** End
   to end `fixed` **-0.360 %**, `sos` **-0.513 %** (`cube` -0.192 % on the
   second commit; no `cube` base was taken before the first — that column is
   a gap in the record, not a suspicion). Both came out of **one command**:
   `cg_edges.py --callers` on the `Vec::clone` and `grow_one` rows, which
   (-45) had flagged as "the sibling table nobody has run".
0a. **And the rule is which column you sort it by.** (-45) says to rank an
   allocation table by *calls* — that finds a `Vec` built to be thrown away.
   The two rows this pass took sit at #4 and #5 by calls and would never have
   been reached that way; they are outliers in **Ir/call**:
   `continue_spell_resolution` at **1,601 Ir a call** was deep-copying the
   resolving spell's whole `Effect` tree (it could not borrow, because `card`
   is moved to the graveyard later — clone the `Arc<CardDefinition>` first
   and it can), and `finalize_cast` at 677 was two
   `mem::take().into_iter().partition()` round trips over a
   `delayed_triggers` list that is empty on almost every cast. **Rank by
   calls for a thrown-away `Vec`; rank by Ir/call for a tree being
   deep-copied.** `alt_spell_half_of(&def)` is the shape for the borrow —
   the existing walker's pick against a definition the caller holds, one
   walker at two lifetimes, rather than a second copy of the pick.
0b. **The next rows are read and ranked, and the column that ranks them is
   grows *per call*.** `declare_blockers` is the one to take: **11,466 grows
   over 2,732 calls = 4.2 a call**, plus 1.8 `reserve_rehash` a call — four
   buffers and a map filled an element at a time (2.02 M, 0.134 % of `sos`).
   `advance_step` is **READ and refuted**: 0.94 a call is the single
   `events.push(StepChanged)` on a list the caller hands in empty, i.e. the
   allocation that holds the returned event. `check_state_based_actions` is
   1.7 a call and its named collects are *already* scan-gated; localizing
   the rest needs `cg_contexts.py --separate-callers`, not a source read.
   **Do NOT take `gather_continuous_effects_inner`'s row** — its buffer is
   `sa_cards`, empty on a vanilla board, and a blanket
   `+ battlefield.len()` reserve is the shape the fifty-eighth pass measured
   at **+1.54 %**. See PERF's (-45), which now carries both tables.
0c. **The archive triage is DONE — see item 10.**

0z. **The sixty-fourth pass: (-44) closed, five of (-45)'s rows taken, (-48)
   answered.** End to end `fixed` **-0.43 %**, `sos` **-1.06 %**, `cube`
   **-0.96 %**. (-44) was a token mint building an 8,232-byte
   `CardDefinition` *per token in the batch* (`sos` -0.605 %); (-45)'s row was
   `compute_permanent_pass` collecting an **empty** iterator on 83.6 % of its
   89,154 layer passes (`sos` -0.354 %); four more of its rows —
   `resolve_effect` x2, `fire_delayed_event_watchers` x2, `blockers_of` —
   went together for another `sos` -0.098 %, and **the tell there is
   syntactic: a `collect()` whose next line is an `is_empty()` on what it
   just built.**
0b. **Three rules out of it.** (a) **A collect is worth what its *empty*
   fraction is worth, and that fraction is a property of the pool** — cube
   moved least on the layer-pass gate because a cube board carries statics.
   Size the rest of (-45)'s table that way. (b) **The hoist, not the memo**:
   -0.53 % came from moving one build out of a loop, -0.04 % from memoizing
   it. (c) **Price a linear scan before writing a `Hash` impl** — (-44)
   deferred the token memo for needing `TokenDefinition: Hash`; a capped `Vec`
   over the derived `Eq` was enough and smaller. `cg_lines.py`'s location
   column now carries a directory, which named
   `check_state_based_actions`' largest row: `core/src/slice/iter/macros.rs`,
   the sweep's own walks.
1. **The sixty-third pass took (-47) and it read 5x its sizing.** Base
   `0036e238` -> tip, two commits: `fixed` **-0.579 %**, `sos` **-0.446 %**,
   `cube` **-1.289 %**. The entry costed only the attacker-resolution hoist
   (~0.24 % of cube); what it missed was that the pair loop *above* the
   check paid per pair for six attacker facts and two blocker facts, and
   that two of the twelve gates *inside* the check never name an attacker.
   `pick_blocks_inner` self on cube 24,906,488 -> 7,583,714.
2. **The rule that pass yields, and it is cheap to re-run: in a loop over
   pairs, ask of every term which side of the pair it belongs to.** The tell
   is a callee count that is a *multiple* of the pair count —
   `computed_permanent` sat at exactly 2x `blocker_can_block_attacker`.
   `cg_edges.py --callees <fn>` ranked by **calls** is the table. Other pair
   loops worth the same read: `legal_block_targets`, the combat-damage
   assignment loops, `pick_attacks`'s blocker scan (see 5).
3. **(-48) is CLOSED — mimalloc is 5.99 % faster and the memory is bought.**
   Eight ABBA blocks, 8/8, CI **-7.04 .. -4.95 %**, null control flat
   (+0.20 %, CI -0.79 .. +1.18 %); RSS 27.2 MiB against the system
   allocator's 17.5 on `release-fast`. Six percent is larger than any single
   perf commit in ten passes and 9.7 MiB an actor is not a constraint on any
   box that runs four of them. **The null resolved +/-0.99 % on the 2.80 GHz
   box, not the +/-2 % this file quotes for the 2.10 GHz one — run the null
   where you are.** **Replicated on a second container the same hour** (the
   other session took the entry concurrently): **+7.98 %**, CI +7.05..+8.91,
   8/8, null flat at +/-1.03 %; RSS system 17.4-17.8 vs mimalloc 26.8-28.9.
   The two CIs meet at ~7.0 % and do not overlap below it — **quote "6-8 %,
   host-dependent", not one number** — and plan actors off **~27 MiB**.
4. **Then (-43), the CoW clone cost, and its paying side is now read.**
   `Arc::clone_from_ref_in` is 85,650 calls / 64.0 M on `sos` (**4.20 %**)
   and 168,808 / 128.2 M on `cube` (**4.69 %**) — 19.4 % of `make_mut`
   unshares actually deep-copy, at ~747 Ir apiece. **The caller table is
   flat since the 58th tip; do not re-collect it.** The `cube` column is
   new and has two unread clone-shaped rows: `restore_payment_state`
   (553 Ir/call) and `place_card_at_resolved_zone` (629). **The bind-once
   half is done; don't grind it.** **(-44) is now closed on both halves** —
   its `__memcpy` table and its allocator table are both read and both flat —
   so after (-43) the fresh queue is (-45) (the cost of asking).
   **(-46) is deliberately last and should stay there** — see 9.
5. **What is left of (-47) is small and needs measuring, not assuming.**
   `pick_attacks`'s "unblockable by the current board" check is the same
   shape, but only ~4,900 of 28,374 pair checks come from outside
   `pick_blocks_inner`, and hoisting there resolves every opponent blocker
   eagerly on boards where the branch is never reached. The two
   `battlefield_find`s per composed `blocker_can_block_attacker` are (-38)'s.
6. **Three devices, all cheap to re-run, each found a pass's biggest
   commit.** (a) Read a caller table's **Ir/call** column: a cost far above
   the family mean is a copy of something big. **Do NOT re-run it on the
   allocator — both sides are read and both are flat.** (b) `cg_edges.py
   --callers SpecFromIterNested` ranked by **calls**, then ask which
   collects can be non-empty on the pools the actors play — that is (-45).
   (c) **Rank rows by `cube% / sos%`, not by either share** —
   `scripts/cg_ratio.py cg.cube.out cg.sos.out`, which exists now and reads
   `cg_edges.py`'s parse directly, so the truncation-reads-as-infinite
   failure cannot happen. That device found pass 62's second commit and
   pointed pass 63 at `pick_blocks_inner` (2.09x). **The ratio is a pointer,
   not a size** — confirm with Ir/call.
7. **Crash-freedom recipe (unchanged, nearly free).** Add `--decks cube`
   and `--decks sealed` (`--games 120 --threads 3`, two seeds) to the
   standing `--decks all` grid whenever a pass touches rules code: `all` is
   17 fixed archetypes and cannot reach a card they never draw, and the
   `overflow` build is the expensive part either way.
8. **Refuted, do not re-take:** a presence bit belongs in `sba_board_scan`
   only when the question has no early exit of its own (third loss for the
   fusion device in `creature_death_possible` alone). A collect whose drain
   touches `self` is load-bearing — check that line first. Per-event
   bitmasks for `is_event_hardcoded` read **+0.12 %**: see (-16), and the
   rule that a line's Ir is not what removing the line would save.
9. **Three measurement cautions before you rank anything.** (0) **RSS per
   actor is the ML-relevant number and the file was quoting the wrong
   build.** At one tip on one box: system allocator 17.6 MiB, shipped
   `release`/mimalloc **24.0-24.3**, `overflow` 27.2. The sixtieth pass's
   "-19 %, 17.7 MiB" is a `--no-default-features` reading and reproduces
   exactly — but **plan actor counts off ~24 MiB**. Nor does RSS compare
   across containers (2.10 GHz box 24.0-24.3, 2.80 GHz box 26.8-30.1). (a) Clock
   numbers go through `scripts/ab_wall.py` with its null control. The
   **+/-2 %** this file records was calibrated on the 2.10 GHz box; **both
   nulls run on the 2.80 GHz one this run resolved about +/-1 %**, so the
   resolution is a property of the host and the minute — run the null where
   you are and quote what it says. Ir over-reads by ~2x. (b)
   `name_index()` builds 22,568 `CardDefinition`s to read their names —
   104.7 M Ir, **6.8 % of a six-game `sos` total and 0 % of `cube` and
   `fixed`**. **Subtract it before quoting an `sos` share**, and note the
   three pools' totals are not comparable to each other at that scale. It is
   candidate (-46), ranked last on purpose: one-time per process, so
   ~0.001 % of a training actor. **A cost that is 6.8 % of the measurement
   and 0.001 % of the workload is not a perf candidate.**
10. **Housekeeping. The archive triage is DONE — don't re-take it.**
   ENGINE_BACKLOG 5.2k -> **3.8k** and CARD_BACKLOG 4.2k -> **4.0k**, both
   at the sixty-seventh pass. Shipped rows dropped *unless* the row carried
   an open residual (`Residual:` / `Remaining` / `still` / ⏳ / 🟡 — 111 and
   18 rows kept in place); no body edited; both files now open with an index
   table. ENGINE_BACKLOG is ordered bugs / mechanics / rules-coverage /
   tooling; CARD_BACKLOG is retitled by *subject* rather than by the run
   that found it ("Noticed this run (Mirage wave 5)" -> "Mirage wave 5") and
   ordered open-first. The ~400 lines of GUI backlog left with them, into
   **`CLIENT_BACKLOG.md`**. TODO **856**, PERF 7.6k. Suite is **14 test
   binaries / 19,170 tests**, not the "22" older blocks quote. The 47th
   through 50th Log entries are folded (the 49th and 50th at the 66th pass:
   344 lines to 103); **next folds are the 51st/52nd**. PERF is **7.7k** with
   the sixty-seventh pass's entry in it.
11. **Bugs: the parallel target-walker class is CLOSED** (`core_rules::
   target_walkers` 39 -> **0**, and it asserts `is_empty()` now — add the
   walker arm, do not reintroduce a threshold). 20 of the 39 were the test
   counting `Reflexive` / `ReflexiveTrigger` bodies the walker is
   deliberately blind to; **19 were real**, each a shipped card whose
   targeted effect resolved against an empty list. Verified on the wider
   crash-freedom grid at `aaadfdc2`: 11,600 games clean, `--bench`
   `decisions` still 196,220. **It landed after the tip the Baseline's Ir
   columns were measured at** — a `cube` / `sos` total taken now is not
   comparable to them; re-base first.
   **Both named successors are now CLOSED too, at the sixty-fifth pass.**
   (a) `Selector::TriggerSource` on a **self-scoped** block trigger — closed
   by `core_rules::block_trigger_selectors`, which found five abilities over
   four cards past the three already rewritten: Abomination (a local copy of
   `combat_partner_punisher` gating on `TriggerSource`; the card is black and
   the gate asks green-or-white, so it never fired), Infernal Medusa,
   Frostweb Spider, Tolarian Entrancer, Hedron Blade. **The scope is the
   whole distinction**: on `AnyPlayer` / `YourControl`, and on an
   `equipped_bonus` with `triggers_on_equipment`, the watcher is a third
   object and `TriggerSource` *is* the partner — correct, and the test
   exempts them. (b) `ChooseUnchosenMode` — `requires_target => false` is
   right (it picks at resolution) but nothing bound the chosen mode's targets
   either, so Silent Hallcreeper's copy mode has been dead since it shipped
   *and* burned one of its three picks. Resolution auto-targets now, per
   `Effect::Reflexive`; CR 601.2b drops unsatisfiable modes from the menu.
   **The device both share, and it is the one to re-run:** a test whose ctx
   hands in a target cannot see a binding bug, because a real trigger push
   hands an empty list. **Four** shipped tests passed vacuously this run:
   Abomination's blocker was dying to combat damage, Infernal Medusa's assert
   only covered the survivor, the Hallcreeper's copy mode was fed the target
   the engine never bound, and Absolver Thrull's enchantment was an unattached
   Aura that SBA removes on its own (CR 704.5m). **Ask of any per-card test:
   could this pass if the ability never fired?** Two cheap tells — the ctx
   hands in a target, or the victim would die anyway.
   **That audit was run at the sixty-fifth pass and it is now
   `core_rules::unbound_target_slots`.** First run: **eleven bodies over nine
   cards**, and eight were one missing arm each in `requires_target` —
   `PreventNextDamageFromChosenSource` (six Circle-shaped cards that built no
   `PreventionShield` at all), `SpellBecomesChosenColor`,
   `ExileThenBranchByController`. The invariant has three exempt families and
   they are the whole content of it: **resolution-time targeting**
   (`Reflexive`, `ReflexiveTrigger`, `ChooseUnchosenMode`), **cast-time
   modal** (the action carries the picks), and **deferred-fire**
   (`HauntCreature`, `ReplaceYourNextDrawThisTurn` — correct *only* because
   their fire sites call `auto_target*`, so a new entry has to be checked at
   its fire site, not its resolution).
   **The ninth card was the big one: creature Haunt has never worked.**
   `primary_target_filter` (what the auto-picker aims with) had no
   `HauntCreature` arm, and **`None` there is not "don't target" — it falls
   back to `Any`** and walks players as well as permanents. It handed the
   trigger `Target::Player(1)`; `target_filter_for_slot(0)` *does* have an arm
   and demands `Enchantment`; CR 608.2b then returned `Ok(vec![])` and the
   whole trigger did nothing, silently. Absolver Thrull and Orzhov Euthanist.
   The picker now falls back to the checker's own filter before `Any`, so the
   two agree by construction wherever the primary walker is silent. Verified
   on the wide grid at `b1a772ec`: 11,600 games clean, `decisions` 196,220,
   traces unchanged.
   **And the delayed-trigger fire site**: `WhenCardDies` / `WhenTokenDies` /
   `WhenHauntedCreatureDies` register with `target: None` and pushed with no
   slot bound. CR 603.7c — they auto-target at push now.
11b. **The next bug, and it is sized and diagnosed but not taken.** The
   picker (`primary_target_filter`) and the CR 608.2b legality check
   (`target_filter_for_slot(0)`) are two hand-written walks at opposite ends
   of one target's life, and **27 single-slot bodies aim with one filter and
   are checked against another** — e.g. `Jund Charm` picks `Creature` and
   checks `Player`, `Overload` picks `ManaValueAtMost(5)` and checks `(2)`,
   `Tear Asunder` picks `Nonland` and checks `Artifact|Enchantment`. Most are
   modal (slot 0 differs per mode) so the walkers answer honestly-different
   questions and a blanket invariant is a ratchet, not an invariant — which
   is why the sixty-fifth pass wrote one, watched it need 587 -> 83 -> 27
   exceptions, and **deleted it rather than ship a threshold**. Take it per
   card, or make `primary_target_filter` mode-aware. The *silent-fallback*
   half is already fixed: the picker now falls back to the checker's own
   filter before `Any`.
12. **Cards: `scripts/audit_dropped_may.py`.** The load-bearing "destroy /
   sacrifice / tap / discard" cluster is **read to the end**; the ~337
   remaining are the "you may draw / search / put into hand" tail, where
   declining is almost never right. **Top ML item is still a training run.**

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
