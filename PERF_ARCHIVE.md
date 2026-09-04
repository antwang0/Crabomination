# PERF archive — Baseline closing states, eighty-ninth pass and older

Moved **verbatim** out of `PERF.md`'s "Baseline" section (which had grown to
~9,300 lines of per-pass closing-state narrative) so the working file stays
readable; nothing here is summarized or closed by the move. Newest first, as
in the source. Every `(-N)` reference resolves in `PERF.md`'s Log and
candidates; the standing rules these passes produced already live in
`PERF.md`'s "Standing rules for a perf pass".

### Eighty-ninth pass, the concurrent half — the actor's own clock, read at last, and the whole robustness gate run under a behaviour change

**STATE AT `863d80cb`+ (the ward gate, the CR 613.8 two-phase gather, and the
other half's eight perf commits).**

```text
whole-program Ir, callgrind, profiling-fast --no-default-features,
--a gang --b gang --games 6 --threads 1 --seed 1
  fixed  1,043,038,225   cube  3,169,628,393   sealed  3,115,488,706
  (the pass base `dc478735` read 1,059,426,578 / 3,217,542,883 /
   3,163,675,366 — **-1.55 % / -1.49 % / -1.52 %** across both halves)

suite   19,064 / 0 / 5, clippy clean, golden traces unmoved
--bench 195,528 decisions / 27.44 turns / 611.0 a game / 0 stalls
        (cap 0 / stuck 0 / draw 0), determinism ok,
        thread_determinism ok (3 vs 1 threads identical)
        — byte-identical to the committed invariant, across a rules change
        284.5 / 302.9 / 301.4 games/s, peak_rss_mib 28.2-30.1,
        host_calib_ms 53-72 (`release-fast`, mimalloc, 2.10 GHz Xeon,
        3 threads, box idle)
```

**THE ACTOR'S OWN CLOCK — `selfplay_train`, the committed recipe, and this
file has not carried a reading of it since the fifty-eighth pass.**
`--actors 3 --games 120 --steps 1 --seed 7`, `release-fast`, box idle:

```text
  149.6 / 149.8 / 149.8 / 149.8 / 149.8 / 149.8 games/s
  ~11,700 rows a run, 0 stalls
```

Two notes on the recipe itself, both against what "How to measure" says:

* **The 0.1 % repeatability claim is confirmed to the decimal** — six
  consecutive runs inside 0.13 %.
* **"Always discard the first run" did not reproduce here.** The bullet
  records a first run reading ~45 % low, every time, over four separate
  batches; this batch's first run is 149.6 against 149.8, i.e. 0.13 % — the
  same spread as the rest. Keep discarding it (a 45 % outlier costs nothing to
  guard against), but the effect is not unconditional, and a pass that sees it
  should say what else was running.

**THE ROBUSTNESS GATE, RUN IN FULL UNDER A BEHAVIOUR CHANGE.** The CR 613.8
fix moves what the layer system answers mid-gather, so it gets the whole
ladder rather than the suite:

```text
scripts/robustness_grid.sh   30 cells (5 pools x 6 seeds x 120 games)
                             33,120 games, 0 failures, 0 undecided anywhere
actor leg (the encoder-reachable memos the ladder cannot reach)
  target-audit/overflow/selfplay_train --actors 3, seeds 7/23 x 120
  and 41/97/5 x 400 — 1,440 games, 0 stalls, no assertion
seeded cube sweep, 4,000 pairings through `server::bot_rejection_count()`
  914.1 s under nextest in a debug build: every match terminated inside its
  180 s budget and the count did not move
CRAB_SIM_REJECTS=1  0 / 126,608 simulated declarations over six cells
  (cube + all, seeds 3/7/11, --games 20)
CRAB_PAY_FAILS=1    fixed 0/11,254 — still zero; cube 3,368/20,372 (16.5 %,
  3,292 probe / 76 committed); sos 956/9,114 (10.5 %, 952 / 4)
```

**And the undecided count finally answers its own question.** `--decks all
--games 120 --seed 21` reads **2 undecided of 2,040**, and the
`undecided_by` line this half added prints `cap 0 / stuck 0 / draw 2` — both
are rules draws, nothing capped and nothing stuck. That is the whole point of
the split: the bare count would have cost a rebuild to interpret, and the
grid's own 30 cells then read 0 undecided at the same game count, which is
what says a draw is rare rather than absent.

### Eighty-ninth pass — the zone memo does not generalise from the graveyard to the battlefield, and the reason is the zone's *write* rate

**`zone::Graveyard`'s device — a presence answer memoized on the zone and
invalidated by `DerefMut` — was built one zone over, for the largest single
presence gate left in the simulator, and it loses on `cube` in both policies
it can have.** The ceiling is real and the candidate stays open ((-79)); the
*memo* route is closed with a number.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.  Base 90e6161c.

THE CEILING — `keyword_grant_in_scope`'s battlefield leg deleted outright
(`false &&`), a deliberately wrong probe, one build:
  fixed   1,059,427,130 -> 1,040,913,506   -1.748 %
  cube    3,217,542,509 -> 3,145,267,534   -2.246 %

(1) `zone::Battlefield` — the CoW list plus a `u64` board summary (the OR
    over every permanent of `grant_bits::CONTAINER`, `SYNTH_KEYWORD` and the
    instance `suspected` flag), filled lazily on a miss, cleared by
    `DerefMut` / `&mut IntoIterator` / `cow_mut`:
      fixed   -0.477 %      cube   +0.489 %
      `card_can_grant_keyword` calls  fixed 515,328 -> 158,776   (-69 %)
                                      cube 1,802,138 -> 1,601,332 (-11 %)
      the memo's own miss path        fixed 11,918 out-of-line calls, 3.4 M Ir

(2) The same type with the miss made *free* — `cached_summary()` never walks,
    and the walk the caller was going to make fills it on the way past
    (`any_filling_summary`, short circuit dropped so the fill is complete):
      fixed   +0.235 %      cube   +0.608 %
```

**The rule, and `zone.rs`' own header is one word short of it: a zone memo is
worth the zone's *write* rate, not its read rate.** `Graveyard`'s answer
survives because a graveyard is written a few times a turn. The battlefield is
written on every tap, every damage mark and every counter — and `DerefMut`
cannot tell those from the entries and exits the summary actually depends on,
which is exactly what makes it sound. So the memo misses **28 %** of the
questions on `--decks fixed`, and a miss costs about what the walk it stands in
for costs. Count the writes before building the memo.

**And (2) is the second half of the lesson, because it is the obvious repair
and it is worse than (1) on both pools.** Filling from a pass the caller was
making anyway is only free if the fill does not change that pass: here it
removes the `any()` short circuit, and it turns every gate into "hit, or no
gate at all", so the gate's value gets multiplied by the hit rate the
invalidation had already ruined. **A memo you refuse to fill is not a cheaper
memo.**

**The whole switch cost zero call-site edits**, which is the one reusable
half of the construction: `Deref`/`DerefMut`/`IntoIterator`/`From` on the
newtype carried 43 `for` loops, 556 `battlefield_find` sites, six
`take_card(&mut self.battlefield, ..)` calls and `effective_loyalty_abilities
(card, &state.battlefield)` unchanged — `cargo check` came back clean on the
first build with only the field declaration and one `Default::default()`
edited. If a later pass finds a battlefield question whose *writes* are rare,
the type is four hours of git history away.

### Eighty-ninth pass — two operand-order wins: a tag list hoisted, and a board walk moved behind the test it is `&&`ed with

```text
Same workload and profile as above.

(1) `board_keyword_in_scope`'s `kws.contains(k)` -> a `SmallVec` of `kws`'
    discriminants, hoisted once a call.  Base 90e6161c.
      fixed   1,059,427,130 -> 1,053,235,770   -0.584 %
      cube    3,217,542,509 -> 3,205,483,587   -0.375 %
      sealed  3,163,676,295 -> 3,148,556,194   -0.478 %

(2) `check_state_based_actions_into`'s `player_unlife_active` behind its
    `effective_life(i) <= 0 &&`.  Base (1).
      fixed   -> 1,049,889,584   -0.318 %
      cube    -> 3,196,731,011   -0.273 %
      sealed  -> 3,137,755,593   -0.343 %

    Together: fixed -0.901 % / cube -0.647 % / sealed -0.820 %.
```

**(1)'s lesson is which hoist.** Three forms were built and measured against
the same base:

```text
  precomputed tag list, tag compare only      fixed -0.584 %  (shipped)
  precomputed tag list + exact `&& *w == k`   fixed -0.212 %
  `KeywordSlice::has_kw` (wanted tag only)    fixed -0.105 %
```

`has_kw` was already in the tree and looks like the same idea — it hoists
`discriminant(k)` out of the inner loop — but the loop it hoists out of is the
*wrong* one: it reloads every `kws` entry's tag once per board keyword.
**Hoisting the short list once beats hoisting a scalar often**, 5.6x here. And
the exact fallback costs 2.7x the win for an answer that is identical at every
call site, because every list any caller passes is unit variants; the gate is
documented as an over-approximation with `false` authoritative, so tag-only is
inside its contract rather than a widening of it.

**(2) is a one-site finding, and the sweep that says so is the value.** Three
other `let x = self.<board walk>()` bindings exist in the engine and all three
are hoists *out of loops*, not eager operands — so the class is closed, not
open.

### Eighty-eighth pass — a `Box` field that is written once and cloned 22,684 times

**The (-76) class has a third shape and nobody had swept for it: a `Box<T>`
field on a copy-on-write-adjacent struct that is *never written after it is
set*.** `GameState::resolving_spell_snapshot` is
`Option<Box<ResolvingSpell>>`, stamped at the top of each spell resolution
(CR 706, so a mid-resolution copy rider can still read the popped stack
entry) and read exactly once, by `Effect::MayCopyThisSpell`. `GameState::
clone` runs **22,684 times a `cube` run** and **21,440 of them — 94.5 % —
deep-copied it**, because the bot's probes clone mid-resolution.
`Option<Arc<ResolvingSpell>>` makes the clone a refcount bump.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.  Base f8e3d357.
  fixed   1,059,432,363 -> 1,056,387,174   -0.287 %
  cube    3,217,548,728 -> 3,211,031,131   -0.203 %
  sealed  3,163,681,558 -> 3,156,653,723   -0.222 %
  allocations 1,650,928 -> 1,629,032  (-21,896)
  `GameState::clone`'s callee list loses `Box::clone` entirely
  (525,972 -> 504,532 calls)
```

**The -21,896 is 456 more than the 21,440 `Box`es**, and those are the
`Vec<Target>` inside them: a `Box` deep copy clones what it points at, so the
allocation count is the box *plus* every heap field under it. **`fixed` reads
the largest of the three**, which is the opposite of most rows here — the
bench archetypes cast the same spells with fewer permanents around them, so
the clone is a bigger share of a smaller program.

**The sweep this suggests, and half of it is already done.** Grep `GameState`
and `PlayerData` for `Box<` and ask of each whether anything writes it after
it is set. `Arc` is free on a field nothing mutates and wrong on one that is,
and `Box` is the default only because it was written before the struct
started being cloned twenty thousand times a run. **The other `Box` field on
`GameState` is `decider: Box<dyn Decider + Send + Sync>` and it is already
free**: `Decider::decide` takes `&mut self`, so `Arc` is out, but the clone
is `self.decider.kind().into_boxed()` and `AutoDecider` is a unit struct —
`Box::new` of a ZST does not allocate. Checked, not assumed; do not re-open
it. **And the rest of the sweep is done and empty**: `PlayerData`,
`PlayerCold`, `ColdState`, `CardData` and `CardCold` carry **no** `Box`
field between them — the zones are `CowBox`, and the ~fourteen `Box` fields
on `CardDefinition` (`back_face`, `adventure`, `split`, `room`, …) sit
behind the `Arc<CardDefinition>` every clone shares, so they are copied only
by an `Arc::make_mut` and not by a `GameState::clone`. **The class is
closed** — one field, one commit, and the grep that closes it is
`grep -n ': *Box<' crabomination/src/player.rs crabomination/src/game/mod.rs
crabomination_base/src/card.rs`.

**Gate at the tip** (`e7793c7c`, which also carries the concurrent half's
`3cfa0435` and `539c67eb`): suite 19,063 / 0 / 5, clippy clean, golden
traces unmoved, `--bench` **byte-identical to the committed invariant** —
195,528 decisions / 27.44 turns / 611.0 a game / 0 stalls (cap 0 / stuck 0 /
draw 0) / determinism ok / thread_determinism ok (3 vs 1 threads identical),
206.5-226.1 games/s and peak_rss_mib 27.7-30.4 over three runs at
host_calib_ms 49-54. ⚠ That games/s column is this container's, not the
concurrent half's box — compare calib before comparing rates.

### Eighty-eighth pass — `dispatch_board_scan` IS the memo's row after all, and the refutation one entry up was measuring the wrong shape of it

**The eighty-seventh pass's concurrent half built three bits into
`dispatch_board_scan`, measured `fixed` +0.106 % and reverted it; this is the
same function, five bits, and it reads `fixed` -0.095 % / `cube` -0.317 % /
`sealed` -0.330 %.** Two differences, and both are in the entry below.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.  Base 8d1eae8c (landed on 2f5e4927;
the intervening commits are combat/bot flat_map sweeps in other functions).

(1) `card::dispatch_bits` — five definition bits, own valid flag (bit 31).
      fixed   1,068,008,315 -> 1,066,992,973   -0.095 %
      cube    3,253,015,531 -> 3,242,709,329   -0.317 %
      sealed  3,193,695,024 -> 3,183,146,742   -0.330 %
      dispatch_board_scan  fixed 20,438,442 (1.91 %) -> 16,869,624 (1.58 %)
                           cube  43,932,966 (1.35 %) -> 31,384,094 (0.97 %)
      dispatch_scan_bits (the miss path)  fixed 772,752 / cube 2,604,176
                           over 90,792 calls on `cube`

(2) THE NARROW PROBE — the same commit with every gate *outside*
    `dispatch_board_scan` removed (`trigger_grant_sources`,
    `statics_granted_dying_triggers`, `creature_dies_triggers_suppressed`,
    `card_can_strip_abilities` back to their hand walks). Built and thrown
    away; it is the attribution, not a candidate.
      fixed   1,067,348,681   +0.033 % against (1)
      cube    3,248,271,486   +0.171 %
      sealed  3,187,710,124   +0.143 %
```

**The first difference is the scan's own shape: `if bits == 0 { continue }`
before anything else.** Three of the four facts the scan gathers are
booleans, so the bit *is* the answer and the loop body for a permanent that
answers "no" to all four — most of the board — is one atomic load and one
branch, not a walk with three tests inside it. Bits that only *gate* a walk
buy the walk's length check; bits that *are* the answer buy the whole body.

**The second is (2), and it is a warning about reading a function's own
row.** `trigger_grant_sources` gets **worse** under (1) — `fixed` 6,405,566
-> 7,550,714, `cube` 14,365,404 -> 14,898,698 — which is exactly the
refutation one entry up predicts, a memo load in front of a walk that is one
length check. And removing that gate *and the other three* costs the program
0.171 % of `cube`. **The row moved one way and the program moved the other**,
so the four gates are paying somewhere their own rows do not show: the memo
they warm is read again by `dispatch_board_scan` on the same permanents, and
a miss deferred is a miss the next consumer does not pay. Same lesson as
(-76)'s "the field's own function is not where its reads are", from the other
direction.

**The family has its own valid flag, per `f264ba27`'s rule.** That commit
measured widening `type_scan_bits` from two bits to six at `cube` +0.145 %
because a slot's miss path is the sum of every family on it; `dispatch_bits`
is bit 31 and bits 36-40, so `sba_scan_bits`, `grant_scan_bits` and
`type_scan_bits` keep their own misses.

**`static_effect_strips_abilities` moved to `crabomination_base::effect`**
alongside `static_effect_changes_card_types` and for the same reason — the
answer is a property of the printed static — and `static_grants_triggered_
ability` is new there, peeling exactly `active_static`'s five gate wrappers.
The two must be edited together; the doc comment says so.

**(-77)'S LAST ROW WAS BUILT AND REVERTED IN THE SAME PASS, WHICH IS WHAT
CLOSES THE ENTRY.** `granted_abilities_of`'s six-boolean prologue, six exact
bits with their own valid flag: `fixed` **+0.024 %** / `cube` +0.005 % /
`sealed` +0.011 % against base `e398874c`. The bits *are* the answers and it
still loses, because the walk they replace is **one** `for sa in
&me.definition.static_abilities {}` on a definition with no statics. Read
that against the shipped row above: the difference is that
`dispatch_board_scan`'s body is four things and this one is one. The full
write-up is under (-77).

**STATE AT `e398874c`, the tip of this half.**

```text
suite  cargo nextest run --workspace --exclude crabomination_client
       19,062 / 0 / 5   (~82 s after the build)
clippy --workspace --exclude crabomination_client --all-targets   clean
golden traces  unmoved (they run inside the suite above)
--bench  195,528 decisions / 27.44 turns / 611.0 decisions a game /
         0 stalls (cap 0 / stuck 0 / draw 0) / determinism ok /
         thread_determinism ok (3 vs 1 threads identical)
         — **byte-identical to the committed invariant**
         214.3-233.3 games/s over three runs, peak_rss_mib 27.3-27.8,
         host_calib_ms 48-58 (`release-fast`, mimalloc, 2.10 GHz Xeon, 3
         threads). ⚠ NOT comparable to the concurrent half's 245.3-245.6 at
         calib 46: different container, and calib says so.
whole-program Ir, callgrind, profiling-fast --no-default-features,
--a gang --b gang --games 6 --threads 1 --seed 1
  fixed  1,057,644,796   cube  3,212,823,684   sealed  3,157,096,383
```

### Eighty-seventh pass, the concurrent half — an iterator adapter chain is a per-element branch, and on a whole-board walk that is half the row

Two commits, both the same reading at two scales, and the device is the
reusable half: **an adapter chain (`Chain`, `FlatMap`, `Filter`) costs per
element of the collection it wraps, whether or not the legs it is chaining
have anything in them.** On a whole-board walk that is more than the loop body.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.

(1) `all_static_sources()` is a visitor with the command half gated. Base
    1413f07e.
      fixed   1,081,256,853 -> 1,072,184,345   -0.839 %
      cube    3,290,762,784 -> 3,272,601,023   -0.552 %
      sealed  3,217,989,709 -> 3,200,012,745   -0.559 %
      trigger_grant_sources  fixed 13,043,782 -> 6,405,566   -50.9 %
                             cube  26,573,648 -> 14,365,404  -45.9 %
      grant_scan             fixed  9,091,212 -> 6,654,834   -26.8 %
                             cube  22,482,670 -> 16,523,484  -26.5 %

(2) `compute_permanent_pass`' sorted view is two `extend`s. Base 65357c25.
      fixed   1,068,706,202 -> 1,068,166,641   -0.050 %
      cube    3,259,259,680 -> 3,253,012,848   -0.192 %
      SpecFromIterNested::from_iter  cube 90,555,135 -> 85,635,653;
      allocations byte-identical (1,650,920), so only the branch moved
```

**(1) was priced by deleting the command half outright**, a deliberately wrong
probe — battlefield only — which read `fixed` -0.675 % / `cube` -0.406 %
before a line of the real fix existed. The shipped shape keeps CR 315.5's
command-zone sources behind a presence gate (every game without a conspiracy
or a commander has an empty command zone) and hands the body in as a closure,
so the battlefield half inlines exactly as the `for` loop it replaces. **The
probe is the entry**: `all_static_sources()` is
`Chain<slice::Iter, FlatMap<_, Filter<_>>>` and it was costing **~20 Ir a
permanent** in a loop whose body does nothing on `--decks fixed`, because the
bench archetypes carry no `static_abilities` entry at all.

**The two scales are the point.** (1) is `Chain` + `FlatMap` + `Filter` over
~23 permanents per call, 27,724 calls: 0.84 % of `fixed`. (2) is a bare
`Chain` over a dozen continuous effects, and it is 0.19 % of `cube` and
nothing on `fixed` — **a per-element cost splits by pool according to how long
the collection is**, which is why the same device reads four times differently
on the two pools.

**Three more commits finished the `flat_map` half of the sweep**, found by
reading `FlatMap::next`'s own self row (0.60 % of `cube`) and ranking its
sites with `cg_contexts.py` on a `--separate-callers=3` dump — two minutes,
no rebuild. Each is the same mechanical rewrite to nested `for` loops:

```text
(3) declare_attackers_banded's three whole-board flat_maps  -0.278 / -0.230 %
      FlatMap::next  19,602,312 (0.60 %) -> 8,769,162 (0.27 %)
(4) bot::cast_candidates' hand sweep (five filters, two
    nested flat_maps; candidate order preserved exactly)    -0.271 / -0.179 %
      FlatMap::next  8,769,162 -> 5,737,084 (0.18 %)
      `--bench` byte-identical, which is the gate for a bot-path change
(5) the last two combat flat_maps                           -0.049 / -0.038 %
      FlatMap::next off the profile as a named row
```

**(5) is the size of the finding at the tail and it is worth reading**: the
same mechanical change is 0.28 % on the attack declaration and 0.05 % here,
because these two walks run on few boards. Across all six commits the pass's
adapter sweep is `fixed` **-1.44 %** / `cube` **-1.13 %**.

**STATE AT `2f5e4927`, the tip of this half at the end of the pass.**

```text
suite  cargo nextest run --workspace --exclude crabomination_client
       19,062 / 0 / 5   (~72 s after the build; six new regression tests
       across the pass, all of them assertions their predecessors could not
       make)
clippy --workspace --exclude crabomination_client --all-targets   clean
golden traces  unmoved (they run inside the suite above)
--bench  195,528 decisions / 27.44 turns / 611.0 decisions a game /
         0 stalls (cap 0 / stuck 0 / draw 0) / determinism ok /
         thread_determinism ok (3 vs 1 threads identical)
         — **byte-identical to the committed invariant**, across eight
         perf commits and three behaviour-changing bug fixes
         245.3-245.6 games/s over three runs, peak_rss_mib 28.3-28.5,
         host_calib_ms 46 (`release-fast`, mimalloc, 2.10 GHz Xeon, 3
         threads), against 238.2-242.8 mid-pass and the pass base's
         224.4-228.9 at calib 45-48
robustness  --games 120 --threads 3 --seed 7 --decks all (five pools,
         2,040 games): 0 undecided, no panic
         seeded cube sweep, 4,000 pairings through
         `server::bot_rejection_count()`: **0 rejections, 883 s** — re-run
         with the pass's three behaviour changes in it
whole-program Ir, callgrind, profiling-fast --no-default-features,
--a gang --b gang --games 6 --threads 1 --seed 1
  fixed  1,058,826,730   cube  3,223,413,257   sealed  3,167,858,247
  (the pass base `aefae7bb` read 1,106,711,404 / 3,355,240,795 /
   3,291,519,150 — **-4.33 % / -3.93 % / -3.76 %** across both halves)
```

**AND THE PASS WAS PUT ON THE CLOCK, WHICH IS THE PROJECT'S #1 METRIC AND
NOT WHAT IR SAID.** `ab_wall.py`, both binaries `release-fast` + mimalloc,
base `aefae7bb` vs tip `8fcea726`:

```text
--a gang --b gang --games 2000 --decks fixed --threads 4 --seed 11
8 ABBA blocks = 32 runs

A/B      mean B/A 0.9841  (-1.59 %)   median 0.9836   blocks B faster 8/8
         95 % CI  -1.89 .. -1.28 %
         A spread 25.90-26.20 s (1.2 %)   B spread 25.50-25.70 s (0.8 %)
null     --bin-a base --bin-b base, same workload, same block count
         mean 1.0005 (+0.05 %)   CI -0.29 .. +0.39 %   blocks B faster 3/8
         FLAT — resolution +/-0.34 %
```

**-1.59 % on the clock against -4.33 % in Ir on the same pool.** The gap is
the *build*, not the measurement: callgrind reads `profiling-fast
--no-default-features` — system allocator, no LTO — and this pass's wins were
allocation-shaped almost throughout (the CoW `Vec` class, `SmallVec` locals,
the adapter chains that were feeding `Vec` builders). mimalloc already charges
less for what those changes removed, so a saved `malloc` is worth less on the
shipped binary than in the profile. **Ir is the ranking instrument; it is not
a wall-clock estimate, and this pass is the ratio between them: 2.7x.**

**Also: do NOT read the pass's win off the `--bench` `games/s` line above.**
It went 224.4-228.9 -> 245.3-245.6 across the pass, which is **+7.5 %** — five
times the paired reading, in the other direction from the Ir gap, and taken
hours apart on unpaired runs. That is exactly the estimator `ab_wall.py` was
written to replace, and the eighty-fourth pass's "the wall clock overstated a
change twenty-five-fold" from the other side. `host_calib_ms` does not save
it: 46 against 45-48 is the same reading, and the box still moved.

**The null also dates `ab_wall.py`'s own calibration.** Its docstring says
"this box resolves +/-2 % and nothing finer", measured on `--decks sos
--games 2000`. On `--decks fixed --games 2000` the null resolves **+/-0.34 %**
— six times finer, on a workload with a 1.6 % within-binary spread instead of
6.5 %. **The resolution is a property of the workload, not of the box**, so
run the null on the workload you are about to quote.

### Eighty-seventh pass — a refutation written against a *mechanism* that a later pass built, and the SBA board scan halves

**(-11) said a per-definition presence bitmask "cannot be a lazily-cached
field" because ~20 sites rewrite a definition in place through
`Arc::make_mut(&mut c.definition)`. The eighty-third pass built the
invalidation that answers it and nothing came back here to say so.**
`CardInstance::DerefMut` is the one point every `&mut` reach into a card
passes — `&mut c.definition` needs `&mut c` — so a memo cleared there is
provably fresh, which is exactly the argument `ColorMemo` already ships
with. The SBA sweep's board scan is the second answer off that device.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.  Base d9093fb5.

(2) `sba_board_scan` reads `CardData::sba_scan_bits` — 22 definition-derived
    presence bits memoized on the same atomic word as the printed colours.
      fixed   1,094,037,385 -> 1,084,604,977   -0.862 %
      cube    3,323,357,593 -> 3,300,104,855   -0.700 %
      sealed  3,254,761,426 -> 3,224,180,345   -0.940 %
      cube `sba_board_scan` self 52,822,246 -> 24,823,002  (-53.0 %)
           `CardDefinition::sba_scan_bits` (the miss path)  +5,807,662
           net on the pair -22.2 M of the run's -23.3 M

(3) (-77)'s first row: `card_can_grant_keyword`'s prologue reads the same
    memo — `grant_bits::SYNTH_KEYWORD` (the printed-keyword scan) and
    `CONTAINER` (the five container loads). `CardMemo` widens to an
    `AtomicU64` for it; `clear()` is still one store.  Base 77608b37.
      fixed   1,084,605,509 -> 1,081,260,056   -0.308 %
      cube    3,300,104,632 -> 3,290,764,033   -0.283 %
      sealed  3,224,178,492 -> 3,217,988,369   -0.192 %
      cube `card_can_grant_keyword` (10 monos) 62,871,580 -> 50,778,404,
      plus `grant_scan_bits` 2,075,052: the pair 1.91 % -> 1.60 %

(4) (-77)'s second row: `card_can_change_card_types` is two bits.  Base
    3aaad2b8.
      fixed   1,081,257,382 -> 1,077,783,071   -0.321 %
      cube    3,290,763,109 -> 3,277,427,006   -0.405 %
      sealed  3,217,988,572 -> 3,209,661,183   -0.259 %
      cube `card_type_change_unscoped` self 42,798,018 (1.30 %) leaves the
      table; `card_type_change_in_scope` 13,754,758 + `type_scan_bits`
      2,639,718 is what is there instead, 0.50 %
```

```text
(5) The combat prevention family: six battlefield passes become one
    `battlefield_find` and one walk on the target side, four become the same
    on the dealer side.  Base f264ba27.
      fixed   1,068,706,182 -> 1,065,745,431   -0.277 %
      cube    3,259,260,090 -> 3,244,176,500   -0.463 %
      sealed  3,191,360,312 -> 3,182,046,993   -0.292 %
```

**Row (5) is (-68)'s device on a second function, and the reason it pays is
the reason (-68) *under*-paid.** `combat_damage_prevented_to_self` asked five
questions, each its own `battlefield.iter().any()` and each filtering on the
same two things (`attached_to == Some(tgt)`, or the target's controller).
None of them is an early-exiting scan doing work someone else wanted — on the
ordinary board every one of them walks to the end to answer "no", so merging
them is a straight N-to-1 and nothing is given back. (-68)'s widened `find`
was the opposite case. **Ask whether the walks being fused terminate early on
the common board before pricing the fusion.**

Three single-caller helpers were folded away with it
(`permanent_prevents_all_combat_damage_to_self`, `combat_damage_sealed_by_aura`,
`combat_damage_sealed_for_your_creatures`); `damage_sealed_by_aura` stays,
because the noncombat funnel still asks it.

**Row (4)'s row *moved* rather than shrank, and that is worth one line:** the
gated body got small enough that the local inliner folded
`card_type_change_unscoped` into `card_type_change_in_scope`, so the two
dumps do not have a row in common. **The run total is the number; a self row
whose function stopped existing is not a delta.** -13.3 M on `cube`.

**Row (3) is (-11)'s own shape (i) — "a per-*definition* 'can this printing
grant any keyword at all' bit" — and it lands within a rounding error of the
~0.3 % that entry sized it at**, twelve passes after the entry declared it
unbuildable. **The sizing was right and the impossibility was wrong**, which
is the more useful half: a refutation that carries an arithmetic estimate
stays worth something after its argument dies.

**What is left of the row is the cards that *do* carry a container.**
`card_can_grant_keyword` is still 1.54 % of `cube`: the gate now answers
"no" in one load for a vanilla body, and everything with a `static_abilities`
entry still walks it per predicate. That half is (-61)'s "fewer walks, not a
cheaper one", unchanged.

**The scan's per-card body was five list walks and ~25 field reads at
~113 Ir a permanent, 20,260 times a `cube` run; it is now one memo load, one
`&` and four instance conditions.** Eighteen of the twenty-two flags are set
by the definition alone (`UNCONDITIONAL`); four need an instance condition as
well and carry only their definition half, so the scan ANDs them
(`!c.flipped`, `controller != owner`, `attached_to.is_some()`); the four that
are purely per-instance (`bestowed`, `soulbond`, `sector_set`, `pm_both`)
stay field reads. **`sba_scan_bits`'s 5.8 M is the miss path** — ~97.5 % of
the ~466,000 lookups hit, because a permanent's memo is only cleared when
that permanent is written.

**Two things this device does *not* need, and they are why (-11) was wrong
about it.** It is not keyed on the definition pointer, so `Arc::make_mut`
rewriting a uniquely-owned definition in place cannot leave it stale; and it
costs nothing extra on the invalidation, because widening `ColorMemo`'s
`AtomicU8` to a `CardMemo` `AtomicU32` keeps `clear()` a **single store**.
The `debug_assert!` that audits the colour half now audits this one too, over
19,060 tests.

**The standing-rules line this is the second instance of: a "do not re-open"
written against an *argument* dates the moment the evidence moves.** (-11)'s
argument was about the mutation sites; the mutation sites did not change, the
*chokepoint through them* got built.

### Eighty-seventh pass — the `grow_one` row named for a function that returns a `Vec<GameEvent>` was 10 % that buffer

**The box reproduces the eighty-sixth tip to 515 Ir** (`fixed`
1,106,711,919 here against the committed 1,106,711,404), so the rows below
are comparable with everything above them.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.  Base aefae7bb (fixed 1,106,711,919 /
cube 3,355,241,293 / sealed 3,291,518,912 on this box).

(1) (-75)'s first instalment: `check_state_based_actions_into(&mut events)`
    — the sweep appends to the caller's buffer instead of returning its own.
    73 engine call sites, 71 of them the literal
    `let mut sba = …; events.append(&mut sba);` pair.
      fixed   1,106,711,919 -> 1,106,322,059   -0.035 %
      cube    3,355,241,293 -> 3,353,741,195   -0.045 %
      sealed  3,291,518,912 -> 3,289,113,042   -0.073 %
      cube allocations 1,759,090 -> 1,756,116  (-2,974)
      cube grow_one      428,190 ->   425,410  (-2,780)
```

**-0.045 % for a 0.14 % row, and the gap is the entry's whole finding:
`check_state_based_actions`' 26,942 `grow_one` calls were 24,320 *internal
collects* and 2,622 the `events` buffer.** The sweep produces no events on
most of its 20,152 calls, so the `vec![]` that (-75) priced never allocated
there in the first place; the row it was read off is the gated `Vec<CardId>`
collects (-69) already audited. **This is (-71)'s own caution one level up —
"a `grow_one` row named for a function is not necessarily the buffer you
think" — and the return type is not evidence either.** The change is kept:
the sign is the same on all three pools, the mechanism is counted to the
unit (2,974 allocations at ~500 Ir of whole lifecycle each), and it deletes
62 lines.

**What it says about the rest of the family, before anyone spends the
refactor.** (-75) sized `advance_step` / `declare_blockers` / `resolve_combat`
/ `resolve_top_of_stack_inner` off the same table. Two of those are already
answered in this file and neither is takeable by threading:
`declare_blockers`' row is the `ids` lists (the sixty-seventh pass's
reserve refutation, and pass 86 inlined nine of them), and `advance_step`'s
is `events.push(StepChanged)` on a buffer its caller hands in **empty** —
the allocation holds an event that is *returned*, so threading moves it
rather than removing it. Removing that one needs the buffer to outlive the
action, i.e. a `perform_action_into` whose caller reuses one across actions,
which is a public-API change and not what (-75) describes.

### Eighty-sixth pass, the concurrent half — the CoW `Vec` class, and the sizing rule that reopens it

Eight commits on top of `aefae7bb` (the other half's tip): six perf, two
bugs. Whole-stack reading, callgrind, `profiling-fast --no-default-features`,
`--a gang --b gang --games 6 --threads 1 --seed 1`:

```text
                 base aefae7bb        tip a2e6ea33
  fixed        1,106,711,404 ->     1,094,185,204     -1.132 %
  cube         3,355,240,795 ->     3,324,283,340     -0.923 %
  sealed       3,291,519,150 ->     3,256,373,073     -1.068 %
  cube grow_one      428,190 ->           336,530    -91,660
  cube allocations 1,759,090 ->         1,653,894   -105,196
```

**The finding is one sentence: `Vec::clone` hands back `capacity == len`, so
every `Vec` inside a copy-on-write structure reallocates on its first push
after the copy.** See the Log for the two fixes it takes and for the sizing
rule that decides the second, which is the reusable half:
**`SmallVec<[T; N]>` is `8 + max(N * size_of::<T>(), 16)` bytes, so at
`N * size_of::<T>() <= 16` it is exactly the 24 bytes the `Vec` was** — and
the same change that reads `+0.137 %` at `[_; 4]/[_; 8]/[_; 4]` reads
`-0.463 %` at `[_; 1]/[_; 4]/[_; 1]` on an identical allocation saving.

Per-commit rows (each against the commit below it; the four middle ones were
measured on the pre-rebase chain, whose base is the same tip minus the other
half's `combat.rs` sweep, and they touch disjoint code):

```text
                                              fixed        cube
  CowBox<Vec<T>>::push materializes at len+1  -0.382 %   -0.239 %
  the per-turn cast accumulators, sized       -0.463 %   -0.294 %
  five combat bookkeeping lists, size-neutral -0.221 %   -0.230 %
  the granted-trigger list (a Vec of refs)    -0.074 %   -0.211 %
  the block side's Hollow Warrior gate        -0.134 %   -0.058 %
```

**STATE AT `a2e6ea33`, the pass's eighth commit and the tip of this half.**
The branch tip is `d9093fb5` on top of it (the other half's SBA buffer, whose
own block is above); this reading does not carry its -0.045 %.

```text
suite  cargo nextest run --workspace --exclude crabomination_client
       19,060 / 0 / 5   (~73 s after the build; +4 tests, all new regressions)
clippy --workspace --exclude crabomination_client --all-targets   clean
golden traces  unmoved (they run inside the suite above)
--bench  195,528 decisions / 27.44 turns / 611.0 decisions a game /
         0 stalls (cap 0 / stuck 0 / draw 0) / determinism ok /
         thread_determinism ok (3 vs 1 threads identical, CRAB_THREAD_CHECK=1)
         — **byte-identical to the committed invariant**, and the two bug
         fixes are in it: neither defect fires on `--decks fixed`
         238.2-242.8 games/s, peak_rss_mib 27.5-30.1, host_calib_ms 46-48
         over three runs (`release-fast`, mimalloc, 2.10 GHz Xeon, 3 threads)
         against the other half's 224.4-228.9 at calib 45-48
robustness  --games 120 --threads 3 --seed 7 --decks all (five pools,
         2,040 games): 0 undecided, no panic
whole-program Ir  fixed 1,094,185,204  cube 3,324,283,340
                  sealed 3,256,373,073
```
**CLOSING STATE — this session's last code commit (`3cfa0435`), all gates
re-run together.** Base `aefae7bb` (the pass's start), callgrind, `profiling-fast
--no-default-features`, `--a gang --b gang --games 6 --threads 1 --seed 1`,
**rustc 1.95.0 (59807616e 2026-04-14)** on an Intel Xeon @ 2.10 GHz
(`host_calib_ms` 52):

```text
              aefae7bb          3cfa0435              pass
  fixed    1,106,711,919 -> 1,052,410,526        -4.906 %
  cube     3,355,241,293 -> 3,187,946,489        -4.986 %
  sealed   3,291,518,912 -> 3,141,044,210        -4.571 %
  actor    (pass 83) 4,187,375,624 -> 3,824,255,309 before the vocab memo,
           -8.02 % with play byte-identical

suite   19,063 / 0 / 5      clippy  clean (workspace less the client)
golden traces  unmoved (they run inside the suite)
--bench 195,528 decisions / 27.44 turns / 611.0 a game / 0 stalls
        (cap 0 / stuck 0 / draw 0) / determinism ok / thread_determinism ok
        — byte-identical to the committed invariant at every commit of the
        pass, both halves
instruments  CRAB_SIM_REJECTS 0 in 25 cells; CRAB_PAY_FAILS `fixed` 0.00 %
robustness   -C debug-assertions=yes, 33,120 ladder games + 360 actor games,
             no assertion, no overflow, no panic, 0 undecided
```

**STATE AT the eighty-seventh pass's last perf commit (`3730f9d0`)** — the
gate was run in full at each of the five, not once at the end.

```text
suite  cargo nextest run --workspace --exclude crabomination_client
       19,062 / 0 / 5   (~80 s after the build)
clippy --workspace --exclude crabomination_client --all-targets   clean
golden traces  unmoved (they run inside the suite above)
--bench  195,528 decisions / 27.44 turns / 611.0 decisions a game /
         0 stalls (cap 0 / stuck 0 / draw 0) / determinism ok /
         thread_determinism ok (3 vs 1 threads identical, CRAB_THREAD_CHECK=1)
         — **byte-identical to the committed invariant at every commit**
actor  3,851,460,377 (--actors 1 --games 60 --steps 1 --seed 7), play
       byte-identical to the eighty-first and eighty-third readings
```

**This session's perf commits, each against its own named base** (a
concurrent session landed work between every pair of them, so these do not
sum to a tip-to-tip delta). The last two are measured on the **actor**,
because `--bench` never calls the encoder:

```text
                                     fixed      cube      sealed
  SBA sweep threads its buffer     -0.035 %  -0.045 %  -0.073 %
  sba_board_scan memo              -0.862 %  -0.700 %  -0.940 %
  keyword-grant gate memo          -0.308 %  -0.283 %  -0.192 %
  card-type-change gate memo       -0.321 %  -0.405 %  -0.259 %
  combat prevention fusion         -0.277 %  -0.463 %  -0.292 %
  walker drops three OnceCells     -0.412 %  -0.705 %  -0.496 %

                                     actor
  library encoder drops a BTreeMap  -0.494 %
  vocabulary index is a memo slot   -0.507 %
```

Two changes were built, measured and reverted in the same pass: the memo on
`dispatch_board_scan` and its extension to the creature/land type siblings.
Both are written up above with their numbers — and **the first of the two was
re-taken at the eighty-eighth pass in a different shape** (`cube` -0.317 %):
see that block. The refutation was right about the shape it measured and
wrong about the row, which is the best thing a refutation with numbers can
do.

**(6) A sixth commit is measured on the ACTOR, because `--bench` cannot see
it at all** — `encode_state` is never called on the ladder path.
`encode_library`'s `BTreeMap<u16, _>` becomes an inline buffer plus one sort.
Base a17ac6b0, `CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60
--steps 1 --seed 7`:

```text
  actor   3,843,232,693 -> 3,824,255,309   -0.494 %
  the whole btree family leaves the table:
    btree::node::Handle   6,855,659 -> 0
    btree IntoIter        5,670,369 -> 54
    btree Entry::or_insert 5,631,830 -> 0
  `encode_state`'s own row 44,922,940 -> 44,011,788, i.e. the linear scan and
  the sort are *cheaper* than the `entry()` calls that were inlined into it
```

**The emitted sequence is byte-identical and no net needs retraining**: the
keys are vocabulary indices and unique by construction, so sorting them is
the same total order the map iterated in, and the first card of a name still
wins. `the_library_encodes_as_a_deduplicated_multiset` is the regression test
and it asserts exactly that contract. Both runs produced 5,915 rows over 60
games with 0 stalls.

**(7) The second actor-only commit, same instrument.** `Vocab::index_of` is
a hash lookup on the definition's name, asked **438,318 times a sixty-game
run at ~49 Ir** — once per encoded object and once per library card, 0.57 %
of the actor. It is a pure function of the definition's name and the
vocabulary is frozen (`VOCAB_SNAPSHOT`, and `Vocab::sos_sealed` is its only
constructor), so it is a `CardMemo` slot like the bit families: bits 41-56
hold the `u16`, bit 57 is its flag, and the read carries the same
`debug_assert!` — which here audits the "one vocabulary" claim as well as the
invalidation. Base 80140c81:

```text
  actor   3,818,869,401 -> 3,799,505,669   -0.507 %
  `index_of` was inlined into its callers before and is a 2,219,318 row
  after — the miss path alone
```

**Both actor commits produced 5,915 rows over 60 games with 0 stalls**, i.e.
identical play, and neither can move `--bench`.

**THE TWO WORKLOAD INSTRUMENTS ARE RE-READ AT THE PASS'S TIP, AND ONE OF
THEM MOVED TO ZERO.** The standing rule is to read them before a profile;
neither had been re-run since the numbers in `(-55)` and `(-51)(b)` were
written.

**`CRAB_SIM_REJECTS` is 0 in 25 cells** — five pools (`fixed cube sos sealed
all`) x five seeds (1 11 23 41 97), `--games 12 --threads 3`, **246,000+
simulated declarations and not one rejection**, attack and block legs both.
`(-55)`'s guard holds at the tip and across this pass's eight commits.

**`CRAB_PAY_FAILS`, same config as the table in `(-51)(b)` (`--games 12
--threads 1`), and `--decks fixed` has gone to zero:**

```text
                 (-51)(b)'s table              this tip
  fixed  s1    704/12,002   5.87 %          0/11,298   0.00 %
  fixed  s11   764/12,018   6.36 %          0/11,254   0.00 %
  cube   s11 3,494/20,502  17.04 %      3,368/20,372  16.53 %
  all    s11 7,514/43,404  17.31 %      6,530/42,420  15.39 %
  sealed s11 3,220/23,346  13.79 %      2,620/22,746  11.52 %
  sos    s1    754/8,324    9.06 %        754/8,324    9.06 %   <- identical
  sos    s11   956/9,114   10.49 %        956/9,114   10.49 %   <- identical
```

**`sos` is byte-identical on both seeds, which is what makes the rest a
reading rather than drift** — the instrument and the comparison are sound.
`fixed`'s *attempt* count moved too (12,002 -> 11,298), because this pass
changed bot behaviour, so this is not a like-for-like numerator; **zero is
still zero**, and that entry's line "on `fixed` it is 100 % coloured — 704 of
704" is now history. The remaining rollbacks are ~99-100 % `pay_fails_costly`
(they had already built a source table), which is the half of `(-51)(b)` that
is still open.

**(8) A `OnceCell` THAT IS NEVER INITIALISED STILL RUNS ITS DROP GLUE, AND
THE REQUIREMENT WALKER BUILT THREE OF THEM PER CALL — `fixed` -0.412 %,
`cube` -0.705 %, `sealed` -0.496 %.** Base `4f48f637`.

`evaluate_requirement_static_hinted`'s `Target::Permanent` arm — most of the
requirement traffic — opened with nine closures and three `OnceCell`s
(`computed_cell`, `ct_gate`, `shallow_cell`) and then matched `req` to use
exactly one of them. **Every one of the three is single-use per invocation**
and the block's own comment already said why for the third: "one call
evaluates one `req`, the arms are exclusive and a composite requirement
recurses into a fresh frame". Counted rather than assumed: across the whole
match, `has_type` appears at seven *arms* and the other four closures at one
each, none twice and none in a loop, and `computed()` is called only from
inside those five.

```text
  where the -22,685,603 on `cube` lands
  -13,754,758  card_type_change_in_scope      <- the row disappears; without the
                                                 `get_or_init` closure the gate
                                                 inlines into the walker
   -4,220,844  OnceCell::try_init
   -2,193,616  drop_in_place<OnceCell<Option<Vec<CreatureType>>>>
   -1,645,212  drop_in_place<OnceCell<Option<Arc<ComputedPermanent>>>>
     -954,928  OnceCell::try_init'2
   -6,204,572 / -3,658,628 / +10,078,588  the walker's own three monos (net
                                          +0.2 M — the gate moved inside)
  computed_permanent 88,666,682 -> 88,666,682, byte-identical: the laziness
  the cells existed to provide is unchanged
```

**The drop rows are the part (-55) did not have.** That entry priced a cell
at "two constructions and a branch"; two of these three carry a `Vec` or an
`Arc` inside an `Option`, so they also carry **drop glue that runs on every
invocation whether or not the cell was ever filled** — 3.8 M of the 22.7 M,
more than `try_init` itself. **A `OnceCell<T>` where `T` needs drop is not
free at zero uses.** Count the call sites of what a cell memoizes before
writing one; if the answer is one, it is a `let`.

**THE ACTOR IS NOT A CLOCK INSTRUMENT, AND ITS NULL CONTROL IS THE PROOF.**
The pass's throughput verdict is the concurrent half's — `ab_wall.py` on
`bot_ladder --decks fixed`, **-1.59 % paired against -4.33 % in Ir**, with a
FLAT null. This block is the *other* attempt: the same question asked of
`selfplay_train` directly, `release-fast` + mimalloc, `CRAB_NO_JITTER=1
--actors 3 --games 3000 --steps 1 --seed 7`, hand-rolled ABBA blocks, first
run discarded, `aefae7bb` against `63083cfe`.

```text
                       per-block B/A ratio (lower = the tip is faster)
  A/B   6 blocks   0.65  0.87  1.06  1.19  1.04  1.15
  NULL  6 blocks   0.83  1.00  1.26  1.13  0.72  0.75     <- same binary!
```

**The null's spread is *wider* than the effect's, so this workload resolves
nothing** — and the concurrent half's reading on the same box, in the same
hours, at ±0.34 %, says the box is not the reason. **The actor's
within-binary spread is the reason**: three OS threads racing on four cores,
two sealed builds a game, and a learner thread, against `--decks fixed`'s
1.6 %. Load average read 3.53 during the run, which does not help, but it is
not the finding. **Use `ab_wall.py` on a ladder pool for a clock verdict and
keep the actor for Ir**, where it is deterministic and where its 94.8 %
`play_recorded_game_mcts` share makes it the right ranking instrument.

Two mechanics worth one line each. **`--games 200` was tried first and its
rates cluster on 199 because the *learner* polls on a 200 ms sleep**
(`selfplay_train.rs`, the `timing.sleep` leg) — a one-second workload's wall
time is that quantum, not the actors' work, and three identical runs read
124.6 / 199.3 / 199.1 games/s. It is not the print: the rate comes off an
`f64`, only the trailing `{:.0}s` is rounded. **Size an actor throughput run
so the poll quantum is noise.** And the fingerprint check did pass — both binaries produced **288,106 rows over 3,000
games with 0 stalls** — which is what says the two tips play the same games
and that only the clock was in doubt.

**AND THE QUESTION AN Ir PROFILE CANNOT ASK ABOUT THIS PASS: THE MEMO WORD IS
ON A `CardData` THAT ACTORS SHARE, SO WHY IS IT NOT FALSE SHARING?** Worth
writing down because the whole device rests on it and callgrind is
single-threaded.

`sealed_pool` and `sealed_game_template` are `OnceLock`s, so every actor's
deck is built from `CardInstance`s that **share one `Arc<CardData>`** with
the pool, and `CardMemo` is written through `&self`. Three facts make that
safe and quiet:

* **A write to a card never clears a *shared* memo.** `CardInstance`'s
  `DerefMut` is `Arc::make_mut` *then* `clear()`, and `make_mut` on a shared
  `Arc` clones first — so the `clear()` lands on the thread's own copy and
  the pool's original keeps its answers. There is no cross-thread
  invalidation to ping-pong on.
* **The only cross-thread write is the fill, and it is idempotent.** Every
  slot is a pure function of the definition, so N actors racing to fill the
  same word all store the same bits; a lost update costs a recompute. That is
  why the setters are a plain load-modify-store and not a `fetch_or`.
* **`Clone for CardMemo` carries the value**, so a card cloned out of the
  pool starts warm rather than cold.

**The steady state is therefore a shared-read line**, which is what a
thread-caching profile would want. If a future pass ever adds a slot that is
*not* a function of the definition alone, this paragraph stops being true and
the slot needs its own word.

**THE ROBUSTNESS GATE WAS RUN FOR ALL SEVEN MEMO FAMILIES, ON GAMES RATHER
THAN TESTS, AND IT IS CLEAN.** Every memo this pass added rests on a
`debug_assert!` that re-runs the computation on a hit, and (-58)'s rule is
that 19,062 tests are not the audit — 60 games of `cube` found what the suite
missed in four seconds. So:

```text
RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit \
  cargo build --profile overflow -p crabomination --bin bot_ladder
  ... --a gang --b gang --games 120 --threads 3 --seed <s> --decks <pool>

5 pools (fixed cube sos sealed all) x 6 seeds (1 3 7 11 23 97)
  30 cells, 33,120 games, 0 undecided, no assertion, no overflow, no panic
  (3.2 s a fixed cell to 15.5 s an `all` cell — the whole grid is ~4 min)

and the same flags on -p crabomination_ml --bin selfplay_train
  --actors 3 --games 120 --steps 2 --seed 7 / 19 / 41
  360 games, 34,619 rows, 0 stalls — this is the leg that audits the
  vocab-index memo, which no `bot_ladder` pool reaches
```

**Check the binary, not the flags** — `strings target-audit/overflow/bot_ladder
| grep "memo is stale"` prints the assertion messages and the `profiling-fast`
binary prints none, which is the two-second proof that `RUSTFLAGS` reached
the crate that carries the assertion.

**`scripts/robustness_grid.sh` is the whole thing, committed** (build, the
`strings` check, the grid, a non-zero exit on any failure). `POOLS`, `SEEDS`
and `GAMES` are environment overrides and `--no-build` reuses `target-audit/`.
The actor leg stays a hand-run because it needs the `crabomination_ml` build;
its command is in the script's header.

**The finding to carry: the actor has rows the ladder cannot see, and the
first of these was 0.63 % of it sitting in plain sight.** `encode_state` +
`encode_card_object` are 2.4 % of the actor and 0 % of every `--bench` pool;
so is the whole deck builder. Re-read the actor block in "Profile of record"
before assuming a candidate list built from `--decks cube` is complete.

### Eighty-sixth pass — the local accumulator that allocates, and a dependency feature that was worth 0.12 % of `fixed`

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.  Base c8ebea50 (fixed 1,113,113,776 /
cube 3,386,370,818 / sealed 3,321,932,106 on this box).

(1) (-71): `gather_continuous_effects_inner`'s `sa_cards` is inline storage
    (`SmallVec<[_; 16]>`), so the gather stops allocating once per gather.
      fixed   1,113,113,776 -> 1,112,977,809   -0.012 %
      cube    3,386,370,818 -> 3,368,986,404   -0.513 %
      sealed  3,321,932,106 -> 3,308,449,213   -0.406 %
      cube allocations  1,832,924 -> 1,782,746   -50,178  (-2.7 %)
      cube grow_one       546,800 ->   482,428   -64,372  (-11.8 %)

(2) (-73): `LayerFreezeState::perms` is inline storage too — the scope memo
    that `Clone for LayerFreeze` empties on every `GameState` clone.
    Base 0e86d36f (which carries (1) and a concurrent session's bot fix).
      fixed   1,112,827,516 -> 1,109,689,934   -0.282 %
      cube    3,368,684,452 -> 3,363,324,209   -0.159 %
      sealed  3,307,990,133 -> 3,301,557,606   -0.194 %
      `computed_permanent` leaves the `grow_one` table entirely
      (21,120 -> 0); cube allocations 1,782,746 -> 1,777,364
```

```text
(3) (-71)'s sweep, first instalment: nine `Vec::new()` locals in `combat.rs`
    are inline buffers — `declare_blockers`/`declare_attackers_banded`'s four
    `ids` and two `tapped`, `pt_deltas`, `life_paid`, and the combat-damage
    batch's `creature_damage`.  Base = row (2)'s tip.
      fixed   1,109,689,934 -> 1,106,711,404   -0.268 %
      cube    3,363,324,209 -> 3,355,240,795   -0.240 %
      sealed  3,301,557,606 -> 3,291,519,150   -0.304 %
      cube grow_one 461,308 -> 428,190 (-33,118); allocations
      1,777,364 -> 1,759,090; `declare_blockers`' own row 36,526 -> 25,220
```

**Across the pass's three perf commits, from `c8ebea50`:** `fixed`
1,113,113,776 -> 1,106,711,404 (**-0.575 %**), `cube` 3,386,370,818 ->
3,355,240,795 (**-0.919 %**), `sealed` 3,321,932,106 -> 3,291,519,150
(**-0.915 %**). A concurrent session's bot fix landed between (1) and (2) and
is inside those spans; each row above names its own base.

**Eight inline slots takes all 21,120 growths and sixteen would take none
more**, so the capacity question answered itself: only 5,382 of the 21,120
were *first* allocations — the rest were `realloc`s through `finish_grow`,
which is why the allocation count falls by so much less than the growth
count. **Read both columns before sizing an inline buffer.**

**(-72), the same device on a struct field, is refuted in the same pass** —
`players: SmallVec<[Player; 4]>` reads `fixed` +0.600 % / `cube` +0.490 % /
`sealed` +0.542 % while removing exactly the 22,684 allocations it promised.
The difference between the two is the *read count*: forty sites in one
function against ~35,000 across the workspace. See the candidates section.

**The `union` feature on `smallvec` is 0.12 % of `fixed` on its own, and
without it this entry does not ship.** Measured, both variants built:

```text
                              fixed      cube      sealed
  SmallVec<[_;16]>, enum     +0.108 %   -0.458 %   -0.351 %
  SmallVec<[_;16]>, union    -0.012 %   -0.513 %   -0.406 %
  SmallVec<[_;4]>,  enum     +0.108 %   -0.334 %   -0.315 %
  slice-shadowed, enum       +0.337 %   -0.261 %   -0.157 %
```

Without `union`, `SmallVecData` is an **enum**: every read matches a
discriminant on top of the `spilled()` compare, ~40 Ir per gather. On `cube`
that is repaid many times over by the allocations removed; on `--decks
fixed` it is not repaid **at all**, because the four bench archetypes carry
no permanent with a `static_abilities` entry — the base and the enum build
have *byte-identical* allocation tables there (643,753 allocations,
174,265 `grow_one`), so `sa_cards` never allocated on that pool in the first
place and the whole +0.108 % was bookkeeping. With the union it is a small
win on `fixed` too, and the change stops being a pool split.

**Two things not to rebuild.** Inline capacity **4** removes exactly the same
64,372 growths as 16 (`grow_one` 482,428 either way) and then pays 10,782
*spill* allocations, so the extra 192 bytes of frame are free and the smaller
buffer is strictly worse. And **shadowing the buffer with a `&[T]` slice**
after the prologue — one `Deref` for the whole function instead of ~forty —
is worse on every pool: holding a borrow of the buffer across 3,600 lines
costs more than the derefs it removes.

**STATE AT the eighty-sixth pass's third perf commit** — the gate was run in
full at each of the three, not once at the end.

```text
suite  cargo nextest run --workspace --exclude crabomination_client
       19,056 / 0 / 5   (~85 s after the build; nextest reinstalls in seconds)
clippy --workspace --exclude crabomination_client --all-targets   clean
golden traces  unmoved (they run inside the suite above)
--bench  195,528 decisions / 27.44 turns / 611.0 decisions a game /
         0 stalls (cap 0 / stuck 0 / draw 0) / determinism ok /
         thread_determinism ok (3 vs 1 threads identical, CRAB_THREAD_CHECK=1)
         — **byte-identical to the committed invariant at every commit**
         224.4-228.9 games/s, peak_rss_mib 27.9-28.3, host_calib_ms 45-48
         (`release-fast`, mimalloc, 2.80 GHz Xeon, 3 threads), against the
         pass base's 211.1-224.1 at calib 46-49 in the same container and
         202.3-215.1 at calib 51-61 in the middle of it. **Three readings
         of one configuration spanning 202-229 is why the Ir rows carry the
         claim and this column does not.**
whole-program Ir, callgrind, profiling-fast --no-default-features,
--a gang --b gang --games 6 --threads 1 --seed 1
  fixed  1,106,711,404   cube  3,355,240,795   sealed  3,291,519,150
```

⚠ **`target/debug/incremental` filled the disk mid-pass** and the fix is the
one TODO's Env bullet names: `rm -rf target/debug/incremental`. It costs the
next debug build its incremental cache and nothing else. Callgrind dumps are
the other pile — a `--separate-callers=2` dump is ~10 MB and a plain one
~1.3 MB; delete them once the numbers are in the file.

**Eighty-fifth pass, the concurrent half — four more perf commits on top of
the six below, and three of the four are a *stale abstraction* rather than a
hot loop.** A colour list that had to be a `Vec` before it was six bytes; a
per-card score whose colour term nobody had noticed was closed-form; a pool
constant rebuilt per pool; and a probe *template* whose cached value became
equal to the state it cached the moment a different pass deleted the work it
existed to amortise.

```text
bot_ladder --a gang --b gang --games 1 --threads 1 --seed 1 --decks sealed,
callgrind, profiling-fast --no-default-features.  (1)-(3) are deck
construction, which the ladder cannot see; each row's base is the row above.

(1) (-63): the 56-shape lattice's colour lists are inline `[Color; 5]`, so it
    stops allocating 60 `Vec`s a build and two more per shape.
    Base 3c939502 (the tip of the six commits below).
      run total                14,244,236 -> 13,335,416   -6.38 %
      heuristic_sealed_build    9,148,732 ->  8,269,322   -9.61 %
      program allocations           6,985 ->      4,897  -29.9 %

(2) (-63): the card score's colour term is `10*on - 4*total` over the seat's
    five-bit mask; everything else in it is a `CardBrief` field.
      run total                13,335,416 -> 12,130,329   -9.04 %
      heuristic_sealed_build    8,269,322 ->  7,058,782  -14.64 %
      static_build_score        2,044,015 ->    849,074  -58.5 %

(3) (-66): `sos_draft_pool` and `SosPacks::new` are `OnceLock`s.
      run total                12,130,329 -> 11,225,734   -7.46 %
      sealed_pool               3,856,940 ->  2,858,254  -25.9 %

    Across (1)-(3) the run is 14,244,236 -> 11,225,734 (**-21.2 %**), a
    heuristic sealed build **762,394 -> 594,245 Ir (-22.1 %)** on top of the
    -45.3 % below, and a sealed pool 321,412 -> 238,188. Decks byte-identical
    at every step (`--decks sealed --games 8 --threads 1 --seed 1`).

bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, same binary
configuration.

(4) (-67): the affordance sweep's probe template was a clone of the state
    that every probe cloned again.
      fixed  1,126,344,191 -> 1,116,191,782   -0.901 %
      cube   3,425,701,888 -> 3,398,074,516   -0.806 %
      GameState::clone   cube 29,652 -> 22,688 calls, fixed 13,640 -> 10,822

(5) (-68): `fire_combat_damage_triggers` gathers its six sources in two walks
    of the battlefield rather than six.  Base 1f5274f1, i.e. re-measured
    after the other half's last two commits — not carried from (4).
      fixed  1,115,491,648 -> 1,113,096,892   -0.215 %
      cube   3,397,164,462 -> 3,386,393,977   -0.317 %
      the function's own self row
        fixed 14,378,582 -> 11,838,840  -17.7 %
        cube  52,659,356 -> 41,428,664  -21.3 %
```

**A build is 1,394,414 Ir at the eighty-fourth tip and 594,245 after both
halves of this pass — 2.35x.** The actor figure is arithmetic, not a
measurement: `heuristic_sealed_build` was 4.02 % of a `selfplay_train` actor
at 1,404,242 Ir a build, so the two halves together are worth roughly
**2.3 % of the actor**, of which this half is ~0.5 %, plus ~0.1 % for
`sealed_pool`. Quote it that way or re-measure the actor.

**STATE AT `e9a509e6`, the pass's eleventh commit and the tip of this half.**

```text
suite  cargo test --workspace --exclude crabomination_client
       19,056 passed / 0 failed / 5 ignored   (no `cargo-nextest` in this
       container; `cargo test` runs the same binaries)
clippy --workspace --exclude crabomination_client --all-targets   clean
golden traces  unmoved (they run inside the suite above)
--bench  195,528 decisions / 27.44 turns / 611.0 decisions a game /
         0 stalls (cap 0 / stuck 0 / draw 0) / determinism ok /
         thread_determinism ok (3 vs 1 threads identical)
         — **byte-identical to the committed invariant**, now across all
         eleven commits of the pass
         games_per_s 152-162 at host_calib_ms 53-71, peak_rss_mib 27.5-29.7
         (`release-fast`, mimalloc, 2.80 GHz Xeon, 3 threads)
whole-program Ir at `e9a509e6`, `--a gang --b gang --games 6 --threads 1
--seed 1`, callgrind, profiling-fast --no-default-features
  fixed  1,113,096,892        cube  3,386,393,977
overflow + -C debug-assertions=yes, run at `2bb47a02` (which is `e9a509e6`
plus the other half's CR 508.1d fix and this file): 65 cells, five pools x
thirteen seeds x 120 games/archetype — **71,760 games, 0 undecided in every
cell, no panic, no assertion, no arithmetic overflow**. Sixth clean run of
this sweep, and the first that audits `(-68)`'s combat-damage trigger walk.
```

**⚠ The games/s row is not comparable to the 304.5-314.6 recorded above.**
That one is a 2.10 GHz box at `host_calib_ms` 67-74; this is a 2.80 GHz box
at 53-71 reading half the rate. Third pair of wall-clock rows in this file
that cross containers and cannot be differenced — the *invariant* (decisions,
turns, stalls, both determinism checks) is what carries across, and it is
identical.

**⚠ These rows' bases are the row above (except (5), which says its own), and
the `Base <hash>` lines inside the four earlier commit messages name
pre-rebase objects.** The commits landed on top
of the six below and were rebased before pushing. Sixth time this file has
recorded the hash-in-a-doc hazard; the durable form is in "Standing rules" —
**cite a hash only for a commit already on `origin`**, which for a Baseline
row means writing it after the push.

**Eighty-fifth pass. Six perf commits and one bug fix: one perf commit on the
ladder and five on deck construction, which the ladder cannot see.**

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.

(1) (-65): the granted-trigger walk returns `Vec<&TriggeredAbility>`, so the
    three callers that filter it on `event.kind` stop deep-copying what they
    drop.  Base b653ee0b.
      fixed  1,136,504,564 -> 1,126,484,194   -0.882 %
      cube   3,473,417,095 -> 3,431,926,862   -1.195 %

(2-6) (-63), five commits, measured on `--decks sealed --games 1` because
    `fixed`/`cube` build their decks once. Run total 21,864,561 ->
    14,248,408; `heuristic_sealed_build` **16,732,968 -> 9,146,042, -45.3 %**,
    which at 4.02 % of a `selfplay_train` actor is ~1.8 % of the actor.
    Per-commit rows in the Log's eighty-fifth-pass item 1.
      the ladder across all five:  fixed -0.021 %, cube -0.006 %
```

**STATE AT `fede5014`** — this half's six perf commits, its four bug commits
and the concurrent half above it. Suite, clippy and the 4,000-seed sweep were
re-run at this tip; the `--bench` and Ir rows were taken two commits below it,
at `7c725dc0`, and the commits between them touch only `decide_choose_cards`'s
`min` fill, which the bench pool does not reach. The two Ir totals are the exception and are
labelled: they were taken at `49748e1f`, the tip of the six perf commits below,
which is the base of the concurrent half's row (1); those four rows moved them
and each records its own.

```text
suite  --workspace --exclude crabomination_client   19,056 / 0 / 5
clippy --workspace --exclude crabomination_client --all-targets   clean
golden traces  unmoved (they run inside the suite above)
seeded cube smoke  4,000 pairings, all terminate; bot_rejection_count **0**,
                   where the same sweep before this pass read four. Run three
                   times over the pass: before the fixes (four), after them
                   (zero) and again at this tip (zero)
--bench   195,528 decisions / 27.44 turns / 0 stalls (cap 0 / stuck 0 /
          draw 0) / determinism ok / thread_determinism ok (3 vs 1)
          — **byte-identical to the committed invariant across every commit
          of the pass**, taken three times: after the six, after the CR
          508.1d fix and at this tip. That is what says the pass moved no
          play, including the two combat and targeting changes
          234.5-314.6 games/s, peak_rss_mib 27.7-30.0, host_calib_ms 46-74
          (`release-fast`, mimalloc, 3 threads — a *different profile and a
          different container* from the 242.9 `release` row recorded below,
          and the spread across the three readings is the container's, so
          none of it is comparable to anything; the invariant above is)
whole-program Ir **at `49748e1f`**, same configuration as the rows above
  fixed  1,126,243,545        cube  3,431,708,160
and at `7c725dc0`, i.e. `e9a509e6` (the concurrent half's recorded tip) plus
this half's two rules commits — the CR 508.1d cost gate and the off-board
gate's revert:
  fixed  1,113,096,892 -> 1,113,114,995   +0.0016 %
  cube   3,386,393,977 -> 3,386,372,363   -0.0006 %
```

**Those two rows are what says the rules changes cost nothing, and they are a
*cross-container* difference of one part in sixty thousand.** The gates are
the reason: `attack_cost_payable` sits behind `attack_tax_possible` *and*
behind the must-attack presence gate both callers already pay, so on a board
with no attack tax — which is both bench pools — the whole addition is one bit
test inside a function that is not reached. `cg_edges.py` reports 0 calls for
all four new symbols on both pools, which on `profiling-fast` means *inlined*
rather than *not executed*, so it is the totals above and not that table that
carry the claim.

**Those two totals are a cross-session control and they held.** A concurrent
session recorded `fixed 1,126,243,196 / cube 3,431,711,282` at `41ff9d00`,
which carries (-65) and the first (-63) commit; this run's tip carries four
more (-63) commits and reads within 350 and 3,100 Ir of it **on a different
box**. Deck-construction work is invisible on the ladder to five significant
figures, and the ladder is reproducible across containers to the same.

### Eighty-third pass — the pass below

**Eight perf commits, and the largest is a *census* cashed
in: (-13)'s remaining half was takeable the moment the instrument said the
rollback never runs.**

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.

(1) CR 613.8's `SecondPass::of` gate walk, hoisted out of the two ways an
    effect list reaches several permanents.  Base ae2b107d.
      fixed  1,167,670,783 -> 1,166,181,370   -0.128 %
      cube   3,550,805,805 -> 3,542,194,720   -0.242 %

(2) `sim_step` abandons a rejected action instead of rolling it back, so the
    simulation stops taking `perform_action`'s checkpoint.  Base 5f71b77e.
      fixed  1,166,252,055 -> 1,153,737,518   -1.073 %
      cube   3,542,580,434 -> 3,516,749,547   -0.729 %

(3) The same gate hoist as (1) on the third site — the SBA death sweep's
    `compute_battlefield_creatures`.  Base d919b606.
      fixed  1,154,148,607 -> 1,154,012,056   -0.012 %
      cube   3,518,102,194 -> 3,516,335,058   -0.050 %

(4) (-62): `crate::zone::Graveyard` carries the "is there a `GraveyardAnthem`
    here" answer, so neither `gather_continuous_effects_inner` nor
    `keyword_grant_in_scope` walks the zone to ask.  Base 408ed3fc.
      fixed  1,154,146,337 -> 1,148,391,412   -0.499 %
      cube   3,518,099,807 -> 3,508,715,284   -0.267 %
    ceiling, both walks deleted outright, same base
      fixed  1,145,865,968  -0.717 %   cube  3,503,216,156  -0.423 %

(5) The requirement walker takes its fallback chains in a `match` arm
    instead of two-to-eight `Option::or_else` links, and its card lookup's
    off-battlefield tail moves out of line.  Base 8e025be5.
      fixed  1,145,258,420 -> 1,140,173,403   -0.444 %
      cube   3,504,621,705 -> 3,484,724,591   -0.568 %
    `Option::or_else` 2,187,078 calls -> 54 on `cube`.

(6) (-64): `acted_on_own_turn` becomes a `u64` bitmask, so a state clone
    stops allocating, copying and freeing a two-element `Vec<bool>`.
    Base 3509ec81.
      fixed  1,140,632,403 -> 1,136,891,471   -0.328 %
      cube   3,485,011,167 -> 3,475,270,004   -0.280 %

(7) `ComputedPermanent::keywords` becomes a `PrintedList` — the override is
    a `Box<[Keyword]>`, so materializing it is one allocation rather than
    the `Vec` buffer plus the `Box` around its header.  Base 94c1f8b7.
      fixed  1,126,241,376 -> 1,126,329,500   +0.008 %
      cube   3,431,706,805 -> 3,425,948,531   -0.168 %
    the +8 bytes it costs, priced alone by padding the struct
      fixed  +0.040 %   cube  +0.058 %
    program allocations 1,908,884 -> 1,857,406

(8) `ColorMemo`: a permanent's printed colour set is memoized on `CardData`
    and cleared in `CardInstance`'s `DerefMut`, so `compute_permanent_pass`
    stops re-deriving it per pass.  Base 3c939502.
      fixed  1,126,329,500 -> 1,124,375,678   -0.173 %
      cube   3,425,948,531 -> 3,419,653,513   -0.184 %
    `compute_permanent_pass -> printed_color_set` 301,838 -> 55,050 on cube,
    95,792 -> 17,720 on fixed — an 81-82 % hit rate
```

**(2) is gated by `--vs`, not only by `--bench`, because `--bench` is one
pool and the rejection it changes was last seen on another.** Tip against the
`29cfaaf3` base, `release-fast`, `--games 200 --threads 3 --seed 11`: the
**null** on all three pools — `fixed` 400 pairs, `cube` 800, `sos` 500,
**50.0 % with every pair split and no engine-disagreement fault**, against a
base-vs-base control that reads the same. That is the strongest available
statement that the two binaries play byte-identical games, and it covers
`cube`, where the last non-zero `CRAB_SIM_REJECTS` reading lived.

**(2) is the largest single row on this branch in fifteen passes, and none of
it is cleverness — it is the census.** (-54b) proved the checkpoint could not
be removed by an atomicity argument (both declarations pay costs mid-
validation, and that is still true). The argument it did not try is that the
*simulation* never needs the rollback: `CRAB_SIM_REJECTS` reads **0 in 129
configurations** between the census's own 69 and the 60 swept for this commit,
and `sim_step` owns its state — a caller that gets `false` drops it unread.
**An instrument that has read zero for long enough is a licence, not just a
guard.** The live game's checkpoint is untouched.

**And (2) is worth more than the clone it removes, which is why (-13) kept
under-pricing it.** The checkpoint re-shares every CoW zone the simulation had
already unshared, so the action's first write to each zone deep-copies it
again; the clone and the drop are ~3.5 k Ir of the ~5.7 k it cost.

**Base re-measured for (2) rather than carried from (1)**, per the eightieth
pass's rule — the payment census landed between them and moved `fixed`
+0.006 % / `cube` +0.011 %. Both rows are behaviour-preserving: `--bench`
byte-identical across each (**195,528 decisions / 27.44 turns / 0 stalls /
determinism ok**, unchanged since the eighty-second pass's refresh and across
four concurrent commits), golden traces unmoved, suite 19,050 / 0 / 5, clippy
`--workspace --exclude crabomination_client --all-targets` clean.

**Robustness gate, and it is the largest sweep the branch has run:**
`-C debug-assertions=yes` on the `overflow` build, 65 cells over five pools x
thirteen seeds x 120 games/archetype — **71,760 games, no panic, no
assertion, no arithmetic overflow, 0 capped, 0 stuck**, 2 draws. Run twice:
once at the pre-(2) tip and once at `cfc684fc`, clean both times. (55,200 of
the same sweep at an earlier tip, also clean.)

**STATE AT `3509ec81`** — the row a later run compares against, and the only
one in this section measured on the whole pass rather than on one commit.
(First taken at `cfc684fc` and re-taken at `3509ec81`, two commits later:
every row below is identical across the pair.)

```text
suite  --workspace --exclude crabomination_client   19,054 / 0 / 5
clippy --workspace --exclude crabomination_client --all-targets   clean
--bench   195,528 decisions / 27.44 turns / 0 stalls / determinism ok
          thread_determinism ok (3 vs 1 threads identical)
          242.9 / 247.6 / 247.2 games/s at host_calib_ms 45-51
          peak_rss_mib 24.3-24.4      (release, 2.80 GHz Xeon, 3 threads)
golden traces  unmoved
overflow + -C debug-assertions=yes   65 cells, five pools x thirteen seeds
          x 120 games/archetype: 71,758 decided, 0 panic / 0 assertion /
          0 overflow, 2 draws (`all` s37) and nothing capped or stuck.
          Run at `cfc684fc`, `3509ec81`, the `(-64)` tip, the `PrintedList`
          tip and the `ColorMemo` tip — byte-identical all five times. The
          last three audit the `Graveyard` memo's `debug_assert!` and the
          last one the colour memo's, which is the whole reason those two
          invalidations are claims rather than hopes.
```

**Whole-program Ir at `41ff9d00`, so the next run has a base it did not have
to measure** (the same configuration as every row above — `--a gang --b gang
--games 6 --threads 1 --seed 1`, callgrind, `profiling-fast
--no-default-features`):

```text
  at 41ff9d00   fixed  1,126,243,196        cube  3,431,711,282
  at d752fc58   fixed  1,124,378,192        cube  3,419,656,317
  --bench 195,528 / 27.44 / 0 stalls / determinism ok at both, and
  thread_determinism ok (3 vs 1 threads identical) at the second
```

**Read your own base anyway if a commit landed after that one** — this branch
takes several a day from concurrent sessions, and the eightieth pass's rule
(re-measure the base rather than carrying it) is why every row above names
its own.

**Re-taken at the `(-64)` tip, and it is a fourth wall-clock fingerprint that
does not compare to the other three:**

```text
--bench, release, 3 threads, three consecutive runs
  247.2 / 255.2 / 284.7 games/s at host_calib_ms 78 / 68 / 82
  peak_rss_mib 24.3 / 24.4 / 26.0
  decisions 195,528 / turns 27.44 / 0 stalls / determinism ok
  thread_determinism ok (3 vs 1 threads identical)
```

**Higher games/s at a *worse* `host_calib_ms` than the 242.9-247.6 at 45-51
above** — the third hazard in "How to measure" exactly, and the third time
this file has recorded it. The two rows are not a before/after of anything.
What *is* comparable is everything above them: the decision count, the turn
count, the stall count and both determinism checks are byte-identical across
all six of this pass's perf commits, and `peak_rss_mib`'s 26.0 is one draw
from a distribution (the hazard note's "take three before calling a
difference" — the other two read 24.3 and 24.4).

**The `Graveyard` memo's `debug_assert!` is inside that sweep**, and it is the
reason to re-run it rather than carry the earlier reading forward: the
assertion compares a non-UNKNOWN memo against a fresh walk on every read, so
71,758 games of five pools is what says no write path reaches the zone's cards
without clearing it. 19,054 tests do the same thing on far duller boards.

**The wall-clock row is not comparable to the one this run opened with**
(239.9 at calib 53, same box, same binary configuration): the container was
building for most of the run and quiet at the end, and `host_calib_ms` moved
53 -> 45 with it. The four commits above are worth ~1.9 % of `fixed` in Ir
and this box resolves ±2 % at 32 runs a side (`ab_wall.py`'s calibration), so
**do not read the games/s difference as the change** — that is what the Ir
rows are for. It is also a *different* box from the eighty-second pass's
170-175 at 51-57; read "How to measure"'s hazards before comparing any two
wall-clock rows in this file.

**Eighty-second pass, the perf half. Two commits, both behaviour-preserving
(suite green and golden traces identical across each), and both are the same
device: stop redoing per-item what is a property of the batch.**

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.

(1) `Printed<Vec<_>>::push` materializes with `len + 1`.  Base c1450677.
      fixed  1,168,055,399 -> 1,167,067,975   -0.085 %
      cube   3,561,559,147 -> 3,540,505,639   -0.591 %
      compute_permanent_pass's callee list loses Vec::clone (51,706),
      grow_one (51,706) and __rust_alloc (51,706) entirely — 1,690,086
      calls -> 1,483,582.

(2) CR 510.2 — one `trigger_grant_sources` walk for the combat-damage
    batch instead of one per damaged creature.  Base 31eb7333.
      fixed  1,171,020,837 -> 1,170,949,456   -0.006 %
      cube   3,547,606,488 -> 3,536,989,067   -0.299 %
```

**The two bases differ because a concurrent session landed a play-changing
block-rule commit between them**, so the absolutes are not a chain; each row
is its own A/B on its own base. That is the branch's normal state now and the
reason every row here names one.

**`fixed` moves by a twentieth of what `cube` does on (1) and not at all on
(2), and both times for the same reason:** neither the `Printed` materialize
nor the grant walk has anything to do on a board with no keyword grants and
no `GrantTriggeredAbility` static. **A layers-or-grants change measured only
on `--bench` reads as noise.**

**THREAD SCALING IS LINEAR TO THE CORE COUNT, MEASURED AT LAST.** The
candidate list has carried "games/sec at 1, half, full actor counts — find
contention if sublinear" since it was seeded and nobody had run it. Four
cores, `--a gang --b gang --games 480 --seed 20250808`, `release-fast`
(mimalloc), 1,920 games a row, `--decks fixed`:

```text
  threads   wall            games/s   per thread   speedup
     1      20.1 / 19.8 / 19.1 s        95.5         95.5     1.00x
     2       9.7 s                     197.9         99.0     2.07x
     3       6.8 s                     282.4         94.1     2.96x
     4       5.3 / 5.2 / 5.4 s         362.3         90.6     3.79x
```

**3.5-3.9x on four cores across three repeats of the endpoints — 92 % of
linear, and the 2-thread row is above 1.00 per thread, so what is left is
cache and turbo, not contention.** Use `--games 120` or more or the queue
starves the workers and the flattening is the harness, not the program (see
"How to measure"). This says nothing about `selfplay_train` at 24 actors,
where the allocator row in PERF's mimalloc entry already shows the RSS cost
growing — but at the thread counts this box can run, **there is no contention
to find**, and the entry can stop asking.

**⚠ THE COMMITTED `--bench` INVARIANT MOVED AT `50a075fa` AND THAT COMMIT DID
NOT RECORD IT.** `decisions` **195,616 -> 195,528**; `turns_per_game` 27.44,
stalls 0 and `determinism ok` are unchanged. Measured here: 195,616 across
three runs at `27af76f4`, 195,528 at `73ed66e7`, and the only code commit
between them is `50a075fa` (CR 602.5g/h — `available_mana` stops counting a
summoning-sick creature's `{T}` mana). **That is a picker fix, so it is
allowed to move play, and this is the baseline refresh it needed.** The
seventy-seventh pass's block two sections down is the precedent for the shape.
The rule it restates: **`--bench`'s decision count is a committed invariant,
so a commit that moves it says so in its own message** — a later run reading
195,616 off this file and 195,528 off the binary has no way to tell an
intended change from a regression. Anything that moves them again without an
explanation is a regression.

**And the pass's most useful number is a refutation.** Four ways of removing
`compute_permanent_pass`'s *other* allocation — the `sorted` collect, 189,480
calls / 46,426,567 Ir — all read worse than the collect on `fixed` (see
(-56b)). The finding underneath is not about this function: **`Vec::from_iter`
is internal iteration and a hand-written loop is not.** A `Chain<Filter<_>>`
iterates internally through `fold` and externally through a state machine, and
the difference is worth ~0.15 % of `fixed` here — more than the allocation the
replacement was removing. Removing the stack buffer entirely read *worse*
still, which is what pins the cause on the iteration rather than the buffer.

**Eighty-first pass. Base `694fdb05`, two commits, and the second is the
measurement worth keeping.** Both are behaviour-preserving; `--bench` is
byte-identical across both (195,616 / 27.44 / 0 stalls) and
`CRAB_SIM_REJECTS=1` reports the same rejected/proposed counts on every pool
swept.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features.

(1) CR 508.1a — one attacker-legality walker for the gate, the three CR
    508.1d requirement loops and the bot's picker.  Base be4a9987.
      fixed  1,172,261,514 -> 1,171,526,095   -0.063 %
      cube   3,576,010,802 -> 3,571,844,511   -0.117 %

(2) `legal_blockers` — resolve the block planner's CR 509.1a set once a
    declaration instead of seven times.  Base 694fdb05.
      fixed  1,171,526,095 -> 1,168,055,373   -0.296 %
      cube   3,571,844,511 -> 3,561,558,835   -0.288 %
      computed_permanent, cube  455,592 calls / 99,866,368 Ir
                             -> 387,980 calls / 90,626,464 Ir
```

**Twenty restriction families were added to a hot filter for nothing, and the
reason is the shape of the question.** (1) replaced the picker's nine
hand-written families with the engine's ~26 and still read *cheaper*, because
`attacker_self_block` asks them in **one walk** over the computed keyword
slice. The first version asked each family its own `has_kw` / `iter().any()`
— twenty scans of the same three-element slice — and measured **`fixed`
+0.17 %**. A restriction list is a `match` inside one loop, not a
conjunction of scans.

**And the same commit's first draft cost `fixed` +0.62 %, from one missing
instance gate.** The picker's filter delegated to the engine walker for
*every permanent the seat controls*, where the version it replaced tested
`!tapped && definition.is_creature()` first. `computed_permanent` is ~1.5 k Ir
on a first read, so asking it about every land and enchantment is the whole
cost of the change and then some. **A delegation is only free if it inherits
the caller's cheap gates.**

**(2) is the block-side twin of the same lesson, and it says a Vec of ids is
the wrong return type.** `bot_can_block` is one `computed_permanent` per own
permanent, and `pick_blocks_inner` asked it over the whole battlefield five
times per declaration plus once per candidate helper — **117,028 calls /
65,311,295 Ir / 1.83 % of `cube`** at `be4a9987`, the largest single caller of
`computed_permanent` in the program, for an answer that cannot change inside
one declaration. Returning `Vec<CardId>` and testing `may_block.contains()`
inside the same battlefield walks was built and measured first: **cube
-0.110 %**, less than half. Returning `Vec<&CardInstance>` so the five passes
iterate the handful of legal blockers *instead of* the battlefield reads
**-0.288 %**. The membership test hands back in scan what it saves in layer
probes; the win is deleting the walk, not the predicate.

**⚠ The eightieth-pass block cost the bench pool 2.8 %, and nothing in this
file recorded it.** `1a05baa7` -> `be4a9987`, same instrument:

```text
  fixed  1,139,970,357 -> 1,172,261,514   +2.833 %
  cube   3,515,821,864 -> 3,576,010,802   +1.712 %
```

Play is identical across it (`--bench` byte-identical throughout), so it is
cost alone. The diff is almost entirely one row — **`computed_permanent`
+10.73 M on `fixed` (+57 %)**, plus its `compute_permanent_pass` +4.02 M,
`Arc::drop_slow` +3.01 M and ~6 M of allocator underneath — and its largest
new caller is `bot_can_block`, which (2) above takes back about a fifth of.
**A correctness commit is still a perf commit on the workload it runs in:
each of those commits reported `--bench` byte-identical decisions and none
reported Ir, and byte-identical play is exactly the condition under which a
cost shows up undiluted.**

**Eightieth pass, a measurement of the road not taken on CR 508.1d.** The
must-attack fix that shipped is `7384e79b`'s, from a concurrent session; an
independent one built here was discarded. One number from it is worth
keeping, because it says where a presence gate pays and where it does not.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1, callgrind,
profiling-fast --no-default-features. Play identical on `fixed` both sides
(--bench byte-identical, 195,616 / 27.44), so this is cost alone.

  the discarded fix, walking `raw_attackers`
  inside one `with_frozen_layers`                fixed +0.090 %  cube +0.087 %
  the same walk behind a hoisted
  `keyword_grant_in_scope` presence gate         fixed +0.275 %  cube +0.172 %
```

**The gate cost three times what it saved.** It is a whole battlefield walk
per combat, and what it buys is skipping `computed_permanent` reads that are
*inside a freeze* — sharing one gather the surrounding code already pays for.
**A presence gate prices against an ungated read, so its value depends on
what that read costs, and a frozen read is nearly free.** Note that
`7384e79b` uses a gate (`attack_requirement_present`) and is right to: its
repair loop walks the **whole battlefield** with **unfrozen**
`computed_permanent` calls, which is the expensive read the device exists
for. Same device, opposite verdicts, and the difference is the callee, not
the gate. This is the fourth reading of the shape — see the `has_atype` /
`has_stype` gates (pass 56, +0.123 % cube) in TODO's do-not-rebuild list.

**Eightieth pass. Base `8d21e898` (= the seventy-ninth tip) vs tip.** One
commit, the fifth in the observation encoder and the last item NEXT named
there: `Vocab::index_of` hashes a card name per encoded object, and a card's
name is a `&'static str` literal owned by its catalog factory, so the same
card always presents the same *pointer*. A second table keyed on
`(name.as_ptr(), name.len())` replaces the string hash and the `memcmp` that
confirms it with a pointer hash.

```text
selfplay_train --actors 1 --games 20 --steps 1 --seed 7, CRAB_NO_JITTER=1,
profiling-fast --no-default-features, callgrind. Identical workload both
sides — 1,788 rows, 1,990 `encode_state` calls.

  I refs                 1,276,484,237 -> 1,269,619,087   -0.538 %
  encode_state (incl.)      85,902,154 ->    78,971,603   -8.07 %

  index_of, 136,684 calls: 6,747,460 Ir at the tip, ~49 Ir/call. The base
  inlines it into all four callers, so there is no base row to subtract —
  the program delta is the measurement, and it is ~50 Ir a lookup.
```

**Cumulative over the five encoder passes: `encode_state` 156,090,720 ->
78,971,603, -49.4 %, and the actor -6.1 %.**

**The base number in the seventy-ninth block is NOT this pass's base, and
re-measuring is what caught it.** That block records 1,275,509,707 for its
tip; this pass measured the same source at **1,276,484,237**, +0.076 %,
because `611e9a3c` (a *correctness* commit, the sim's rollback fallback)
landed between them and moved the actor. Trusting the recorded row would
have credited this change with -0.569 % instead of -0.538 %. **A recorded
baseline is only a baseline until the next commit; if any commit has landed
since, re-measure the base — it is one run and it is the difference between
a number and a guess.**

**The `len` is load-bearing and costs 2.9 Ir a lookup.** Keying on the
address alone reads 1,269,223,838 (-0.031 % against the shipped form) and is
*unsound*: two `&str` at the same address with the same length are the same
bytes, but the address alone does not identify a string — a linker is free to
lay a short literal at the front of a longer one and hand both the same
pointer, and the cache would then answer a lookup for `Forest` with the index
of `Forestwalk`. Nothing in the suite would have caught it; the cache is a
pure optimization, so a wrong hit is a silently mislabelled embedding row.
**Buy the second `usize` compare.**

**The nested `if` and the let-chain are the same program**, 1,269,615,393 vs
1,269,619,087 over a 1.27 G run — 3,694 Ir, which is the clippy fix costing
nothing and is also this file's smallest reproduced difference. Two builds of
different source produced a **bit-identical** `encode_state` subtree
(78,971,603), which is the cleanest available proof that the workload really
was the same on both sides.

**A pure optimization needs a test that it is still an optimization.** No
other test in the suite can tell a cache that answers every lookup from one
that misses every lookup and falls through to the string map — the answers
are identical, only the Ir differs. `the_vocab_pointer_cache_hits_on_the_
pool_and_misses_off_it` asserts the hit on every pool factory's name, so the
day one of them builds its name at run time (a `format!`, a `String` field, a
name assembled from a set code) the suite says so instead of the encoder just
getting slower. **The same shape as the `*_scan` `debug_assert!`s: the
mechanism that makes the fast path fast is itself an invariant.**

**Seventy-ninth pass. Base `a2b19fea` (= the seventy-eighth tip) vs tip.**
One commit, the fourth in the observation encoder and the smallest edit of
the four: `ManaCost::colored_symbols` returns an iterator instead of a `Vec`.

```text
selfplay_train --actors 1 --games 20 --steps 1 --seed 7, CRAB_NO_JITTER=1,
profiling-fast --no-default-features, callgrind. Identical workload both
sides — 1,788 rows, 1,990 `encode_state` calls.

  I refs                 1,290,226,662 -> 1,275,509,707   -1.141 %
  encode_state (incl.)     100,714,414 ->    85,939,751   -14.7 %
```

**Cumulative over the four encoder passes: `encode_state` 156,090,720 ->
85,939,751, -45.0 %, and the actor 1,351,728,059 -> 1,275,509,707,
-5.64 %.**

**The `Vec` had four callers and three of them only counted it** —
`encode_card_object`'s colour-requirement features, the deck encoder's pip
histogram, `affordable_covered`'s pip count. The fourth collects to escape a
borrow and now says so. This file already documents the device one method
above the one it fixes: `colors()` -> `color_set()`, *"the `Vec` form was the
engine's fifth-largest source of `RawVec::grow_one`, and every consumer only
ever asks `contains` or iterates."* **A `-> Vec<T>` whose callers all write
`for x in f()` is a grep, not a profile** — and this one survived the earlier
sweep because it is on `ManaCost`, not on the type the sweep was reading.

**14.8 M against the 7.4 M the edge itself carried**, because
`affordable_covered` was allocating too and the malloc/free pair went with
both. `bot_ladder` is untouched by construction — three of the four sites are
in the encoder, which is **0 calls** there, and the fourth is one rare mill
effect — so no Ir column on that binary is claimed.

**Seventy-ninth pass, second half: `encode_library`'s `BTreeMap` is REFUTED
in both directions — built twice, measured twice, reverted twice.** It is the
one NEXT named as the encoder's remaining ~0.68 %, and it is not there.

```text
                                       vs a2b19fea+colored_symbols
                                       program        encode_state
A  dedupe on the card *name*, one       +0.217 %       +1.4 %
   `index_of` per distinct name
B  keep `index_of` per card, replace     +0.024 %       -1.5 %
   the map with a linearly-scanned Vec
```

**A is the useful refutation.** The map hashes every library card's name and
a forty-card library is ~fifteen distinct ones, so deduping on the name first
looks like it saves twenty-five fx hashes. It does — and a `&str` linear
scan over fifteen entries costs more than they did. **A `&str` compare is a
`memcmp`; a `u16` compare is one instruction.** Key a small scan on the
cheapest thing that distinguishes the entries, not on the thing you were
going to look up anyway.

**B is the more interesting one because the encoder really did get faster.**
`encode_state` fell 1.5 % — the `BTreeMap`'s 56,845 `or_insert` calls and
37,795 `IntoIter::dying_next` over twenty games are real — and **the program
did not move**. The likeliest reading is that ~57 k node allocations a run
were feeding the same allocator everything else uses, so removing them
changed malloc's free-list state rather than the program's work; the base
dump was lost to a container reset before it could be diffed, so that stays a
reading rather than a finding. **The rule that does not depend on it: a
subtree win is not a program win, and on an allocation-heavy path the two can
differ by the whole of the subtree.**

**Seventy-eighth pass. Base `c9bf0b78` (= the seventy-seventh tip) vs tip.**
Two commits, both in the observation encoder, both measured on the actor
path with `CRAB_NO_JITTER=1` and an identical workload throughout — 1,788
rows, 1,990 `encode_state` calls on every reading below.

```text
                       base            A: cover hoist   B: reserves
I refs                 1,309,077,782   1,295,972,442    1,290,226,662
                                        -1.001 %         -0.443 %
encode_state (incl.)     113,710,320     105,371,552      100,714,414
                                        -7.3 %           -4.4 %
```

**Cumulative over the two encoder passes: `encode_state` 156,090,720 ->
100,714,414, -35.5 %, and the actor 1,351,728,059 -> 1,290,226,662,
-4.55 %.**

* **A — the castability flags rebuilt the colour cover per hand card.**
  `affordable` is Hall's condition over the seat's untapped sources: 31
  masks, each counting how many sources make a colour in the mask. That count
  is a function of the *sources* alone, and the hand loop recomputed it per
  card — then the next-turn flag recomputed it again on a **clone** of the
  whole source slice, per card. `source_cover` hoists it; `cover_with_extra`
  derives the next-turn cover by adding one to every non-empty mask (a source
  that makes every colour intersects them all), so the clone is gone. The
  per-card half also stops looping five colours per mask — `need[mask]` comes
  off the subset recurrence in 32 adds.
* **B — every group grew from capacity zero.** An `EncodedObject` is ~190
  bytes and `EncodedState::default` starts all eight groups empty, so a
  battlefield of twenty is five reallocations and four memcpys of what was
  already written: **19,909 `grow_one` calls out of `encode_state`, ten a
  state, 9.7 M Ir -> 0.** Every size is known before its loop.
* **The rule both halves share, and it is why the encoder had three of these
  in a row:** the encoder is written per *object*, and per-object code
  inherits the loop's iteration count for anything it recomputes. The
  seventy-seventh pass's twelve keyword questions, A's 31-mask cover and B's
  eight `Vec`s are the same mistake at three scales. **What is left is the
  same shape**: `Vocab::index_of` is a name hash per object (1.02 % of the
  actor) and `encode_library` builds and destroys a `BTreeMap` keyed on the
  vocab index per state (~0.68 % across `or_insert`, `dying_next` and
  `__memcmp`).

Neither commit changes a feature value — no net needs retraining — and the
encoder is **0 calls** on `bot_ladder`, so no Ir column there moves. Suite
18,751 / 0 failed / 5 ignored, clippy clean.

**Seventy-seventh pass. Base `fd8c307f` vs tip, and it is measured on the
*actor* path, which this file had never profiled.** One commit: the
observation encoder's twelve keyword questions inverted into one pass.

```text
selfplay_train --actors 1 --games 20 --steps 1 --seed 7, CRAB_NO_JITTER=1,
profiling-fast --no-default-features, callgrind. Identical workload both
sides — 1,788 rows, 1,990 `encode_state` calls.

  I refs                 1,351,728,059 -> 1,309,077,782   -3.156 %
  encode_state (incl.)     156,090,720 ->   113,710,320   -27.2 %
  has_keyword calls            950,700 ->       110,124
```

**The finding is that a whole hot path was outside the profile of record.**
Everything in this file is `bot_ladder`; a `selfplay_train` actor runs a
different program on top of the same engine, and three of its top rows do not
appear on `bot_ladder` at all:

| row (self, `--actors 1 --games 20`) | actor | `--decks cube` |
|---|---|---|
| `CardInstance::has_keyword` | **3.21 %** | 0.77 % |
| `recommend::build_shape` (deck construction, two decks a game) | 1.37 % | 0 |
| `encode::encode_card_object` | 1.25 % | **0 calls** |
| `encode::Vocab::index_of` | 1.02 % | 0 |
| `encode::encode_state` | 0.97 % self, **11.6 % inclusive** | 0 |

`encode_state` at 11.6 % is the shape to keep: **422 `has_keyword` calls per
encoded state**, because `has_keyword` re-walks five lists per keyword asked
and the encoder asked twelve. Inverting it makes the cost the card's keyword
*count* instead of the encoder's question count. What is left of the encoder
after the commit is `Vocab::index_of` and `encode_state`'s own walk.

**And one artefact, so nobody chases it:** `rand_distr::Normal::sample` reads
2.68 % of a 20-game actor run and is candle initialising the net's weights —
722,816 draws through `CpuDevice::rand_normal`, once per process. It
amortises to nothing on a real run.

**⚠ `selfplay_train --seed N` DOES NOT REPRODUCE A RUN, and every number
above needed `CRAB_NO_JITTER=1` to be comparable.** One binary, one seed,
`--actors 1`:

```text
                  rows over 20 games
default           1,788 / 1,770 / 1,776     <- three runs, same seed
CRAB_NO_JITTER=1  1,788 / 1,788 / 1,788
```

The bot's tie-break draws go through `bot::jitter_below`, which uses the
*thread* RNG unless something installs a seeded stream — and
`set_jitter_seed` is the **ladder's** device for antithetic pairs. Nothing in
the `selfplay` actor path installs one, so the seed names the deck pool and
the shuffles and not the bot's tie-breaks. Consequences, in order of how
much they cost: **a training run cannot be replayed**; an A/B of two builds
on the actor path compares different workloads unless pinned (the first
reading of this pass came out -0.674 % against a base that had played 1 %
fewer rows, where the pinned pair reads -3.156 %); and `--games N` is not a
fixed amount of work. Pin it for any measurement, and see TODO for the open
question of whether the actor path *should* be seeded.

**Seventy-sixth pass, third commit — the combat planners stop proposing
declarations the engine rejects.** Base is the pass's second tip; the base
binary predates two `encode.rs` commits that are 0-call on `bot_ladder`.

```text
                          base            tip
I refs, --decks fixed     1,131,405,094   1,132,104,709   +0.062 %   17,064 decisions, identical
I refs, --decks sos       1,373,764,548   1,374,976,391   +0.088 %   16,368 decisions, identical
I refs, --decks cube      2,650,931,261   2,756,233,241   +3.97 %    25,532 -> 25,608
```

**`cube` +3.97 % is the game being played, not work added, and the diff is
unambiguous about it.** The direct cost of the change is one row —
`blocker_can_block_attacker_pair` **3,642,438 -> 5,762,458, +0.08 % of the
program**, the new per-pair evasion loop. Everything else that moves is
downstream and diffuse: `gather_continuous_effects_inner` +5.30 M,
`dispatch_triggers_for_events` +4.76 M, the allocator +10.4 M,
`compute_permanent_pass` +3.13 M. **Blocks that used to be rejected as a batch
now happen**, so creatures trade, triggers fire, and games run longer — the
golden trace for seed 3 goes 19 turns to 21. `fixed` and `sos` play
byte-identical games and their columns are the 0.06-0.09 % the loop costs.

**This is a throughput regression on the pool the ML phase actually runs, and
it is the right trade**: the defect it fixes is the bot declaring no blocks at
all on any board with an evasion keyword it could not see, which biases both
play strength and the training data. It also **re-opens (-54)**: the checkpoint
whose rollback this cost was paying for now fires zero times on every workload
measured.

**And the committed `--bench` invariants move with it** — `decisions`
195,886 -> **195,616**, `turns_per_game` 27.48 -> **27.44**, stalls still 0,
determinism ok. That is the intentional, explained change the baseline-refresh
rule asks for; anything that moves them again without one is a regression.

**Seventy-sixth pass. Base `5e4ec3bd` (code-identical to the seventy-fifth
tip) vs tip.** First commit: the bot's affordability pre-filter stops walking
the whole board three times per hand card. Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--threads 1 --seed 1`, **`CRAB_NO_JITTER=1` both columns**.

```text
                          base            tip
I refs, --decks fixed     1,138,298,501   1,131,317,108   -0.613 %
I refs, --decks sos       1,408,080,967   1,402,099,459   -0.425 %
I refs, --decks cube      2,665,352,881   2,649,976,954   -0.577 %
```

Decision counts byte-identical on all three pools (`fixed` 17,064, `sos`
16,240, `cube` 25,532), so the two builds play the same games; 7 golden
traces unchanged, suite 18,749 passed / 0 failed / 5 ignored.

**The base columns are the fourth independent confirmation that callgrind Ir
is portable across these containers.** Against the seventy-fifth pass's tip
columns for the same code — a different box, a different day — `fixed` reads
**+5,077**, `sos` **+4,953**, `cube` **+4,879**, i.e. **0.0004 %** and all
three the same absolute number, which is argv length.

**The change, and the reason it is sound with no gate to audit.**
`can_afford_in_state_with` calls `extra_cost_for_spell`,
`cost_reduction_for_spell_full` and `colored_spell_tax_for_spell` once per
hand card, and each walks every static source on the board. Most permanents on
these boards have an **empty** `static_abilities`, so most of every walk was
an `Arc` deref to read an empty slice. `CostStaticSources`
filters those out once per sweep and the three functions get `_over` forms
that take the list. There is no variant enumeration and therefore no
`debug_assert!` gate: dropping a source whose inner loop has nothing to
iterate cannot change an answer.

```text
cube, edges out of can_afford_in_state_with   base         tip
30,350 calls either side
  cost_reduction_for_spell_full               17,075,918    9,694,528
  extra_cost_for_spell                         7,221,562    2,561,506
  colored_spell_tax_for_spell                  6,075,898    1,440,040
                                              30,373,378   13,696,074
program                                                    -15,375,927
```

**How much of the board carries statics, read off the three rows**: the
`colored_spell_tax_for_spell` row is nothing *but* the walk and it drops
**76 %**, so roughly one permanent in four carries a `static_abilities` entry
at all. The other two rows drop less (65 % and 43 %) because they also do
per-card work the filter cannot touch — `extra_cost_for_spell`'s
`additional_cast_cost` reads and `cost_reduction_for_spell_full`'s ten
card-intrinsic reduction fields. **The commit message for this change says
"roughly half" and that is the one number in it that is wrong**; the edge
table above is the derivation.

**The gather costs ~1.3 M over 10,852 builds** (the difference between the
edge saving and the program), i.e. ~120 Ir a sweep, and it is lazy: only
10,852 of ~17,400 sweeps reach the affordability test at all.

**This re-opens an entry that says "do not re-open", and the arithmetic is
why.** The 2026-08-12 refutation measured **+0.066 %** for a *fused scan* on
this function, with the four static walks at **0.29 %** of the profile and
**1.13 cards per sweep**. Both halves have moved: the three walks are
**1.14 % of `cube`** now over **30,350** calls against 12,114, and **2.80**
cards reach the filter per sweep that reaches it at all. The old entry's own
numbers are what date it — see (-34)'s block, which now carries both
readings. The refutation's transferable half still stands and this change
obeys it: **the scan has to be lazy**, because an eager one on
`pick_combat_trick`'s empty sweeps cost +0.35 % at pass 40.

**What is left of it, and it is the half the old entry was about.** The three
edges are still **13,696,074 (0.52 % of `cube`)** and the walk that remains is
over the sources that *do* carry statics, almost none of which are of the
cost-changing families. A `cast_cost_scan`-style bitmask over
`CostStaticSources::gather`'s existing walk would take most of that — but it
is a hand-maintained enumeration of ~30 `StaticEffect` variants across the
three functions, so it needs the `debug_assert!`-at-the-gated-site device the
existing scan uses, and it is a separate commit.

**Seventy-fifth pass. Base `1b67c154` vs tip `475e4332`.** One commit:
`cast_candidates`' nineteen pure-filter specialty blocks stop probing eagerly.
**Both columns are `CRAB_NO_JITTER=1`** — see below for why that is the only
sound way to read this one; the decision counts are byte-identical on all
three pools either side (`fixed` 17,064, `sos` 16,240, `cube` 25,532), so the
two builds play the same games.

```text
                          base            tip
I refs, --decks fixed     1,137,083,850   1,138,293,424   +0.106 %
I refs, --decks sos       1,448,267,246   1,408,076,014   -2.775 %
I refs, --decks cube      2,709,179,195   2,665,348,002   -1.618 %
```

**The `fixed` column is layout drift and the diff says so.** That pool reaches
none of these blocks (`cast_candidates -> accept_on` is absent from its
profile), and the whole +1.2 M sits in `dispatch_triggers_for_events`
(+1,061,018) and `fire_combat_damage_triggers` (+341,320), neither of which
this commit touches; `cast_candidates` itself reads **-13,382**. A diffuse
profile whose top 45 rows are 68.5 % moves ±0.1 % on inlining alone.

**The change.** The main candidate block has been lazy since the fiftieth
pass — the engine dry run happens at the *pick site*, in score order, so a
tick probes one or two candidates instead of the whole hand. The twenty-four
specialty blocks were not: each ran `would_accept_on` per candidate and
dropped the state it produced. Nineteen used it as a pure filter and now push
`(action, false)`; the five that probe to *decide* what to emit (convoke's
fewest-helpers walk, the two kicker-subset searches, the two that drop the
plain cast of the same card) keep it. `castable` carries the flag instead of
being an all-validated list, so no candidate changes position.

```text
cube                              base     tip
probes (accept_on, all callers)   9,146    9,146
  <- cast_candidates              1,482      254
  <- main_phase_action_with       4,498    4,998
  <- sim_spell_action_inner       3,166    3,894
casts (finalize_cast <- cast_spell_with_convoke)   4,720    4,540
sos probes                        6,448    6,146
```

**The probe count is flat and the cast count drops, which is the whole
commit**: the winner's probe is now the state the caller adopts
(`Picked::Probed` / `Finalist::settled`) instead of a run thrown away ahead of
a second identical one.

**AND THE MEASUREMENT ITSELF NEEDED A NEW SWITCH.** With jitter live the same
pair reads `fixed` -0.036 %, `sos` -3.412 %, `cube` **+0.503 %** — and `cube`
takes 24,880 -> 25,012 decisions, i.e. the games diverge. The cause is not
the policy: **the scored pickers draw one `jitter_below(4)` per candidate**,
so offering a candidate that later fails validation consumes a draw and
re-aligns the tie-break stream for the rest of the game. Pinning the draws
(`CRAB_NO_JITTER=1`, one `OnceLock` read) makes both builds play the same
games and the columns above are what the code costs. **Read a bot-side
refactor's Ir per decision, or pin the jitter** — `cube` at +0.503 % was
+0.53 % more decisions at -0.03 % apiece.

**AND THE CLOCK WAS TAKEN, WHICH GIVES THIS FILE ITS FIRST Ir:WALL RATIO FOR
A PROBE-REMOVAL CHANGE.** `scripts/ab_wall.py`, eight ABBA blocks,
`release-fast` + mimalloc both sides (the shipped build), `CRAB_NO_JITTER=1`
so the two binaries play the same games,
`--a gang --b gang --games 2000 --decks sos --seed 11 --threads 4`:

```text
              mean B/A   95 % CI            blocks B faster   spread
A/B           0.9871     -2.19 .. -0.39 %   6/8               A 5.4 %, B 5.5 %
null control  1.0020     -0.70 .. +1.10 %   4/8      FLAT     resolution ±0.90 %
```

**-1.29 % of wall against -2.775 % of Ir — a ratio of 2.15x**, and the null
is flat on the same workload and block count within the same hour.
Compare (-48)'s allocator swap (Ir cannot see it at all) and the sixty-eighth
pass's clone removal (wall *bigger* than Ir, because a clone's cache misses
are Ir-cheap): **what this commit removes is whole action executions —
branches and loads at ordinary IPC — so Ir over-reads it by about two.**
That is the number to price the next sim-side commit with; the box resolves
±0.90 % at eight blocks, so anything under ~2 % of Ir will not show on the
clock here at all.

**Seventy-fourth pass. Base `1772f35e` vs tip.** One commit: the colour
budget reaches the *sink mask*, which is what gates the whole `gated_pick!`
ability chain. Ir readings `profiling-fast --no-default-features`, callgrind,
one thread, `--a gang --b gang --games 6 --threads 1 --seed 1`.

```text
                          base            tip
I refs, --decks fixed     1,143,433,386   1,142,504,409   -0.081 %
I refs, --decks sos       1,455,540,697   1,452,607,928   -0.201 %
I refs, --decks cube      2,596,680,136   2,581,192,429   -0.596 %
```

`sink_facts` lit a bit for any ability of the right *shape*, so a
`{1}{B}: destroy target creature` off two Mountains kept `sink::AB_DESTROY`
set and `pick_removal_destroy` walked the battlefield, built an action and
paid a ~50 k-Ir probe on a cost that could never be paid. Activations
reaching payment on `cube` **1,242 -> 996**, and all 246 removed were
rollbacks (`restore_payment_state` 2,852 -> 2,606).

**A gate is only cheap where the thing it reads is already paid for, and
this pass has the number.** `main_phase_action_with` now owns one
`SweepMana` and hands it to `cast_candidates` and `sink_facts` both. Even so,
an **unconditional** `have.get()` per ability read:

```text
                   unconditional gate   with the printed-pip pre-test
--decks fixed      +0.292 %             -0.081 %
--decks sos        -0.119 %             -0.201 %
--decks cube       -0.562 %             -0.596 %
```

`fixed`'s archetypes activate `{T}` and generic abilities, so the forced
`available_mana` (5,406 -> 6,290 calls) bought nothing there — its rollbacks
did not move at all (230 either way). **Testing the printed cost for a
coloured pip first decides whether the read happens**, and it is free: the
cost is already in hand. Same shape as pass 40's refuted eager read, one
level down.

**And the pass found a real hole in the seventy-first's widening.**
`by_color` was widened to `[total; 5]` wherever a colour could not be
bounded — but `total` deliberately under-counts the very sources that force
the widening. **Two Treasures and nothing else read `total = 0`**, so the
"unbounded" budget still rejected every coloured pip while the engine
sacrificed a Treasure and paid. `u32::MAX` is what the widening meant;
`cmc <= total` stays the separate test it always was. **The rule: a widening
must be widened to something the estimate does not also under-count.** Found
by the oracle on `--decks all` seeds 11/12 (Kessig Wolf Run's `{X}{R}`),
which is the third hole that harness has caught and the third that a reading
of the code did not.

```text
oracle           0 over seeds 1/7/11/12/23/31 x cube/sealed/all at 12 games
                 and --bench
decisions        195,886 byte-identical
turns_per_game   27.48, stalls 0, peak_rss_mib 18.4
ladder printout  identical on fixed / sos / cube
golden traces    7 passed, unchanged
suite            18,749 passed / 0 failed / 5 ignored
clippy           `-p crabomination --all-targets` clean
```

**Seventy-third pass. Base `db6abaa4` (= the seventy-second tip) vs tip.**
One commit: the seventy-first pass's per-colour budget, applied to the two
candidate blocks that never had a pre-filter at all. Ir readings
`profiling-fast --no-default-features`, callgrind, one thread, `--a gang --b
gang --games 6 --threads 1 --seed 1`.

```text
                          base            tip
I refs, --decks fixed     1,144,976,824   1,143,433,386   -0.135 %
I refs, --decks sos       1,462,215,557   1,455,540,697   -0.456 %
I refs, --decks cube      2,601,748,276   2,596,680,136   -0.195 %
```

**The rollback table is the map, and it took `--separate-callers=2` to read
it.** `restore_payment_state`'s one-level caller row is a single
`try_pay_after_snapshot_mode` entry and says nothing; at depth 2 the 2,960
rollbacks on `cube` split:

```text
   1,698  try_pay <- cast_spell_with_convoke     (of 6,418 casts,  26 %)
     734  try_pay <- activate_ability_inner      (of 1,242 activations, 59 %)
     214  try_pay <- try_pay_with_auto_tap_mode
     170  try_pay <- cast_flashback              (of   252 flashbacks,  67 %)
     100  try_pay <- cast_spell_alternative      (of   276,            36 %)
      26  try_pay <- cast_face_down
```

**Casts are the pre-filtered path and they are the *best* of the six.**
Everything else the bot proposes — activations, flashbacks, alternative
costs — reaches the engine on a `would_accept_on` probe alone, and a probe is
~50 k Ir.

`colors_coverable` is the reusable half of the budget: **colour pips are the
one part of a cost nothing in the engine's adjustment machinery moves.**
Every activation and graveyard-cast adjustment is `reduce_generic` /
`add_generic`, an `{X}` binding only *adds* pips, a coloured tax only adds
them — so the check is sound against a **printed** cost, with no
effective-cost computation and no `total` test. That is why it is worth
having as a separate helper from `can_afford_from`: it needs nothing but the
printed cost and the budget, so it drops into any candidate block.

The flashback block is the measured win (payments 252 -> 144, rollbacks
2,960 -> 2,852, probes 11,150 -> 11,010). The `w.ability_arms` block is the
other call site and **its flag is off in every shipped profile**, so it is
correct-but-unexercised here — worth knowing before anyone re-measures the
59 % and finds it unmoved. Its old prefilter was `cmc > pool + untapped land
count`, a count that says nothing about which colours those lands make.

```text
oracle           0 over seeds 1/7/11/12/23 x cube/sealed/all at 12 games,
                 seeds 5/42 x all at 40, and --bench
decisions        195,886 byte-identical
turns_per_game   27.48, stalls 0 (0.00 %)
ladder printout  identical on fixed / sos / cube
golden traces    7 passed, unchanged
suite            18,749 passed / 0 failed / 5 ignored
clippy           `-p crabomination --all-targets` clean
```

**Still open on this path, and sized:** the 1,698 cast rollbacks that
survive the colour budget are generic shortfalls, and `ManaSourceInfo`
carries colours but not amounts, so no sound generic bound exists from it.
See (-51).

**Seventy-second pass. Two commits, measured against two different bases —
the branch carried a concurrent session's seventy-first pass in between.**
Ir readings `profiling-fast --no-default-features`, callgrind, one thread,
`--a gang --b gang --games 6 --threads 1 --seed 1`.

```text
                        base (28f5c628)  A: 3cce6889     B: a35cf054 vs A
I refs, --decks fixed   1,149,551,851    1,148,002,377   1,147,144,140
                                          -0.135 %        -0.075 %
I refs, --decks sos     1,482,418,428    1,481,254,224   1,479,711,103
                                          -0.079 %        -0.104 %
I refs, --decks cube    2,634,005,617    2,630,813,719   2,628,245,776
                                          -0.121 %        -0.098 %
```

**A — the leave-the-battlefield chain's four no-op writes.** (-50) at four
sites on one call path: `on_left_battlefield`'s `mem::take` of
`temporary_control` (a `ColdState` field, so the take *and* the write-back
were two unshares of ~89 collections on every permanent that left), its
`continuous_effects` `iter_mut` + `retain` pair, `remove_effects_from_source`
one frame ahead of it on every removal, and `expire_end_of_turn_effects` at
cleanup. `on_left_battlefield`'s `GameState::deref_mut` edge on cube:
14,212 calls / 743,615 Ir -> **0**.

**And it corrects the profile of record.** NEXT's item 1c blamed that
function's `make_mut` edge on `find_card_anywhere_mut`, which is why gating
*that* read +0.083 % at the seventy-first pass. `find_card_anywhere_mut` is
its own row in the callee table — 7,106 calls, 1.000x, not inlined — so it
was never in the edge. The edge was the two collections above.
**Read the callee table of the function that owns the edge before naming
the callee that pays it.**

**B — `blocked_attackers` and `blocks_declared_this_turn` leave `ColdState`.**
A cold write unshares the whole group at **4,689 Ir**, and `declare_blockers`
wrote two cold fields per declared block: 9,124 of the program's 31,318
`GameState::deref_mut` calls and **11,697,365 Ir (0.445 % of cube)**, ten
times the next site. Both lists belong with `attacking` / `block_map` /
`blockers_declared`, already `GameState` fields; `#[serde(flatten)]` keeps
the JSON identical.

**The chain rule takes most of B back, and that is the row worth keeping.**
Cold clones 3,812 -> 3,020, their Ir 17.87 M -> 13.32 M (-4.55 M) — but
`note_creature_death` went **1,110,418 -> 7,383,488 Ir**: it is now the first
cold write in the frames `declare_blockers` used to own, and *its* writes are
real. The program moved 2.57 M, not 4.55 M, and ~2 M of the difference is the
two extra `Vec`s in `GameState::clone` over 32,580 clones. **A field moved
out of the cold group is worth the clone it removes, minus the clone it adds
to every state clone, minus whatever writes cold next in the same frame.**
The cold group's standing cost after B is 3,020 copies x 4,410 Ir = **0.51 %
of cube**, and the sites that pay it now write real values.

**Seventy-first pass. Base `28f5c628` vs tip.** One commit, and it is not a
`*_scan` bit: the bot's affordability pre-filter gets a **per-colour budget**.
Ir readings `profiling-fast --no-default-features`, callgrind, one thread,
`--a gang --b gang --games 6 --threads 1 --seed 1`.

```text
                          base (28f5c628)   tip
I refs, --decks fixed     1,149,551,833    1,144,976,824   -0.398 %
I refs, --decks sos       1,482,417,805    1,462,215,557   -1.363 %
I refs, --decks cube      2,634,006,362    2,601,748,276   -1.225 %
```

**Both columns are the change in isolation.** The commit was rebased onto a
concurrent session's `3cce6889` after the readings were taken, so its landed
parent is one commit past the base named above; nothing in `3cce6889` touches
the bot's pre-filter.

**Re-checked against the jitter stream after `CRAB_NO_JITTER` landed** (the
seventy-second pass's device: the scored pickers draw one `jitter_below` per
*candidate*, so a change to how many candidates reach a picker re-aligns the
stream and the run diverges even where the policy is identical). The tell is
`cg_edges.py --callers next_action_settled`, read off the dumps these columns
were taken from:

```text
                 base (28f5c628)   tip
--decks fixed    17,058            17,058     identical
--decks sos      16,044            16,020     -24
--decks cube     24,896            24,880     -16
```

**`fixed` played a byte-identical game and still read -0.398 %**, so that
column is pure work removed with no divergence in it at all; `sos` and `cube`
diverge by 0.15 % and 0.06 % of their decisions, far too little to account for
-1.363 % and -1.225 %, and the byte-identical `finalize_cast` count says the
same thing from the other side. **The seventy-third and seventy-fourth passes
are byte-identical on all three pools** (17,058 / 16,020 / 24,880 either
side), so their columns carry no divergence either.

**`AvailableMana` answered "is there a producer for this colour" and the
question is "are there enough".** `{G}{G}` off a lone Forest passed the
filter, reached the pick site, and was thrown away by the engine's payment —
which is where PERF (-51)'s **31.9 % of payments rolled back** comes from.
The budget is the singleton case of Hall's condition: one `[u32; 5]` built in
the walk `available_mana` already takes (pool mana plus each untapped
countable source's best single-activation amount, added to *every* colour it
could make, so it over-counts by construction), five adds and five compares
per hand card. What it removes, on `cube`:

```text
                                          base      tip
cast_spell -> cast_spell_with_convoke     7,110     6,038    -1,072 attempts
try_pay -> restore_payment_state          3,696     2,716      -980 rollbacks
bot::accept_on (dry-run probes)          11,986    10,910    -1,076 probes
cast_spell_with_convoke -> finalize_cast  4,720     4,720     byte-identical
```

**Every cast that used to happen still happens.** The filter removed 1,072
attempts and 980 payment rollbacks and not one completed cast.

**Sound is the whole commit, and four widenings are what makes it sound.**
The budget is switched off entirely — `by_color = [total; 5]` — whenever the
estimate cannot bound a colour, and each of the four was found by *measuring*
against the engine, not by reading the code:

| widening | found by |
|---|---|
| CR 609.4b spend-as-any-colour (Lattice, North Star, Unexpected Potential, Emissary's Ploy) | reasoning — `relax_cost_colors` is asked here with `seat: None` and misses the seat-scoped three |
| a mana-production doubler (Mana Reflection, Nyxbloom) | reasoning — `total` already under-counts it, and an under-counted *budget* becomes a rejection |
| an untapped source with a colour-producing ability `is_countable_mana_ability` rejects — filter lands, Lotus Petal, **Crystalline Crawler** (a counter cost and *no* `{T}`) | the oracle, `--decks sealed` and `--decks cube` |
| CR 305.6 land-type rewrite (Dryad of the Ilysian Grove) — `mana_source_table` reads it through `scan_land_type_rewrites`, `granted_abilities_of` does not | the oracle, `--decks cube` seed 11: `engine_table=["WUBRG", "WUBRG", …]` against nine untapped Mountains |

**The oracle is the transferable part of this pass.** The filter is only
allowed to reject what the engine would also reject, and there is an engine
function that answers exactly that — `could_pay_cost`, which runs
`try_pay_with_auto_tap` on a clone. Wiring it behind an env var at the
rejection site, printing only where the *old* colours test would have
accepted, and sweeping pools x seeds turns "is my model of payment right?"
into a count. It went 6 -> 6 -> 240 -> **0**, and each non-zero named the card
that found the hole. **Any change that tightens a bot pre-filter should be
landed this way**; the first two versions of this one looked correct and were
not.

```text
oracle sweep at the tip: --games 12, seeds 1/7/11/12/23/31 x cube/sealed/all,
                         --games 40 seeds 5/17/42 x all, and --bench
                         = 0 cases where `could_pay_cost` accepts a cost the
                           budget rejects
```

```text
decisions        196,220 -> 195,886       -334 (-0.17 %), see below
turns_per_game   27.53 -> 27.48
decisions_per_game 613.2 -> 612.1
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok (all pairs split)
ladder printout  identical on fixed / sos / cube (6 games, seed 1)
games_per_s      146.02 -> 149.57 (one run each, `--bench --threads 3`; inside
                 this box's spread, quoted for the record not as a claim)
peak_rss_mib     19.9 -> 20.0
suite            18,749 passed / 0 failed / 5 ignored under `debug_assertions`,
                 so the fused-scan `debug_assert_eq!` ran on every one
golden traces    7 tests; `seeded_games_match_their_digests` seed 2 re-blessed
                 — same winner, same 16 turns, same 384 actions, digest only
clippy           `-p crabomination --all-targets` clean
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores, host_calib_ms 50
```

**This pass is NOT behaviour-preserving and that is the point.** `decisions`
moves by 334 over 320 bench games and one golden seed re-blesses, because the
bot stops offering lines it cannot pay: where the top-scored candidate was
unpayable, the old code offered it and something downstream discarded it, and
`pick_combat_trick` (which picks without a probe) submitted it outright. The
guarantee that makes that safe is the oracle's zero, not a digest: **no line
the engine could have paid was removed** — the byte-identical `finalize_cast`
count is the same statement from the other side.

**Crash-freedom and determinism at the tip.** `overflow` profile
(`release-fast` + `overflow-checks`), `--a gang --b gang --threads 3`, seeds
11 and 12: `--decks all --games 200`, `--decks cube` and `--decks sealed` at
`--games 120` = **11,600 games, 0 undecided, no panic, no arithmetic
overflow**, every pair split.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState` or
`Vocab` change is in this pass — the change is to a bot pre-filter, not to
anything the net reads.

**Seventieth pass. Base `d9583dba` vs tip `7ada03d9`.** One commit: the
third `*_scan` bitmask, on the attack declaration. Ir readings
`profiling-fast --no-default-features`, callgrind, one thread, `--a gang --b
gang --games 6 --threads 1 --seed 1`.

```text
                          base (d9583dba)   tip (7ada03d9)
I refs, --decks fixed     1,153,518,398    1,148,918,904   -0.399 %
I refs, --decks sos       1,486,428,329    1,482,238,365   -0.282 %
I refs, --decks cube      2,642,266,152    2,631,861,683   -0.394 %
```

**All three pools move together, which is the tell that the cost was
per-attacker rather than per-board.** `declare_attackers_banded` asks six
whole-battlefield questions of `static_abilities` and three of them sit
inside the per-attacker loop, so the bill is (attackers x battlefield x
statics); `fixed`'s archetypes attack wide, `cube`'s boards are wide, and the
two arrive at the same number from opposite directions. Function self on
cube **26,953,356 -> 22,909,226 (-15.0 %)**, callee count 751,906 ->
735,444.

**The same device on the *block* declaration is REFUTED — built, measured,
reverted, and it is the useful half of this pass.** `declare_blockers` has
two static walks of the identical shape, both per blocker: Void Winnower's
`OpponentsCantBlockWithEvenMv` and `block_tax_for`'s
`BlockTaxToController`. Gating both, with the two extra bits and the two
`debug_assert!`s, read:

```text
                   vs 7ada03d9
--decks fixed      -0.003 %
--decks sos        +0.006 %      <- the wrong way
--decks cube       -0.044 %
```

**A declaration is not a loop over the board the way an attack is.** The
attack side's win comes from three walks *inside* a loop that runs once per
attacker; the block side's two walks run once per **declared blocker**, and
the bench pools declare far fewer blockers than attackers — so the branch
costs about what the walk did. **Do not re-take `declare_blockers` with this
device.** The general rule it yields: a `*_scan` bit pays for the walks it
removes from a loop, so count the loop's trips before writing the bit — four
gated sites and 0.4 %, or two gated sites and nothing, on the same file with
the same shapes.

```text
decisions        196,220                   byte-identical
turns_per_game   27.53
decisions_per_game 613.2
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok (all pairs split)
ladder printout  identical on fixed / sos / cube
peak_rss_mib     18.1 (profiling-fast, --no-default-features, system allocator)
suite            18,747 passed / 0 failed / 5 ignored, 14 test binaries, under
                 `debug_assertions` — so all four scan asserts ran and none fired
golden traces    7 passed, unchanged
clippy           `-p crabomination --all-targets` clean
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores, host_calib_ms 49
```

**Crash-freedom and determinism at the tip.** `overflow` profile
(`release-fast` + `overflow-checks`), `--a gang --b gang --threads 3`, seeds
11 and 12: `--decks all --games 200`, `--decks cube` and `--decks sealed` at
`--games 120` = **11,600 games, 0 undecided, no panic, no arithmetic
overflow**, every pair split. `cube` and `sealed` are the pools that can
actually put a Crawlspace / Ensnaring Bridge / Propaganda on the board, which
is what the four gates skip.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState` or
`Vocab` change is in this pass.

**And one rules commit landed on top of this tip, `4f42c6b4`, which costs
Ir.** The deck-out loss became a state-based action (CR 104.3c) and a decked
player's permanents now leave with them (CR 800.4a): `fixed` **+0.055 %**,
`sos` **+0.012 %**, `cube` **+0.082 %** against `7ada03d9`, one flag read per
seat per SBA sweep. `--bench` `decisions` is **196,220 byte-identical** and
the golden traces are unchanged — no bench or ladder seed decks a player —
so the cost is the sweep's, not a changed game. **Recorded here so the next
pass's base column is not read as a regression**: a total taken at
`4f42c6b4` is ~0.06 % above one taken at `7ada03d9`, and that is the trade
working.

**Sixty-ninth pass. Base `795a296e` vs tip `8147836b`.** Two commits, the
same class as the pass above it — **(-50)'s no-op write through a CoW
handle** — applied to the *other* end of a permanent's life, the zone change.
Ir readings `profiling-fast --no-default-features`, callgrind, one thread,
`--a gang --b gang --games 6 --threads 1 --seed 1`.

```text
                          base (795a296e)   tip (8147836b)
I refs, --decks fixed     1,156,961,796    1,155,462,053   -0.130 %
I refs, --decks sos       1,489,888,128    1,487,957,291   -0.130 %
I refs, --decks cube      2,653,962,531    2,646,120,404   -0.296 %
```

| step | commit | fixed | sos | cube | what |
|---|---|---|---|---|---|
| A | `6234a7ed` | -0.057 % | -0.084 % | **-0.221 %** | six writes on every permanent that leaves the battlefield, none of which usually change anything |
| B | `8147836b` | -0.072 % | -0.045 % | -0.072 % | `graveyard_exiled_for` *is* `graveyard_exile_redirects(..).0`, and both ran |

**Three rebases in this pass, and that is why both columns are here.** A and
B were first measured on top of `ae2f1fb8`, a commit byte-identical to the
concurrent session's `a585bff2` (both sessions took `restore_payment_state`
in the same hour — see the Log). The first rebase dropped it and put
`5c0b07cc` and `a951b378` underneath, so the base was **re-read** before
these deltas were quoted, per the standing rule, and **the per-commit
attributions transferred exactly**: -0.057/-0.072, -0.084/-0.045,
-0.221/-0.072 sum to -0.129/-0.129/-0.293 against the re-measured end-to-end
-0.130/-0.130/-0.296.

**A third rebase then put `86ec1bd8` … `ad39dcce` underneath, and those
columns were NOT re-read.** Those four commits are the concurrent session's
cast-path scans and its `restore_payment_state` owner/controller bug fix, all
in `actions.rs`; none touches the zone-change chain or
`graveyard_exile_redirects`, so **A and B's deltas stand and their absolute
totals no longer describe the branch tip**. The two re-reads above are what
licenses that: the same two commits transferred across one rebase without
moving, which is the evidence the standing rule actually wants. **A fourth
re-read was not worth two more 11-minute builds against a branch that took
four commits from another session in ninety minutes** — and the honest
version of that trade is to say so, not to quote the tip.

**And the two boxes agree on Ir to four parts in a million.** The concurrent
session's published `a951b378` column (`fixed` 1,156,966,317, `sos`
1,489,892,855, `cube` 2,653,967,864) reads **+0.00039 % / +0.00032 % /
+0.00020 %** above this box's reading of the same code, and the same constant
offset appears at `50dfa172`. Callgrind Ir is portable across these
containers at three orders of magnitude below anything this file quotes;
wall-clock and RSS still are not.

```text
decisions        196,220                   byte-identical
turns_per_game   27.53
decisions_per_game 613.2
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok (all pairs split, rho -1.000 on all 160)
ladder printout  identical on fixed / sos / cube at both commits
peak_rss_mib     18.2 (profiling-fast, --no-default-features, system allocator)
suite            19,007 passed / 0 failed / 5 ignored, workspace less client
                 = 18,747 in the 14 crabomination + crabomination_tests
                   binaries (the gate TODO prescribes) + 260 in the other five
                   crates. Re-run after the third rebase, on its tip.
golden traces    7 passed, unchanged
clippy           `--workspace --all-targets` clean, all eight crates
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores, host_calib_ms 46-50
```

**No wall-clock pair is quoted, and the pass above is why that is a real
loss rather than a formality.** The sixty-eighth pass measured its
clone-removal at **2-3 % of wall on `cube`** against -1.73 % in Ir — a
clone's cache misses are wall-expensive and Ir-cheap, so an Ir number
*understates* this class. -0.296 % of cube in Ir could therefore be worth
more on the clock; six ABBA blocks plus a null is ~35 min and two
`release-fast` builds, and this pass spent its build budget on re-reading the
base after the rebase instead. **The next pass that takes another (-50) site
should batch them and price the batch on the clock.**

**Crash-freedom and determinism at the tip.** `overflow` profile
(`release-fast` + `overflow-checks`), `--a gang --b gang --threads 3`, seeds
11 and 12:

```text
--decks all     --games 200   3,400 decided / 0 undecided per seed, 1,700 pairs all split
--decks cube    --games 120     960 decided / 0 undecided per seed,   480 pairs all split
--decks sealed  --games 120   1,440 decided / 0 undecided per seed,   720 pairs all split
```

**11,600 games, no panic, no arithmetic overflow, `rho -1.000` on every
pair.** `overflow` rather than the sixty-eighth pass's `dev` grid because
neither commit here rests on a `debug_assert!` — both are `Deref` reads in
front of writes that were already unconditional, and the 19,006-test suite
runs under debug assertions anyway. `cube` and `sealed` are in the grid
because the reset chain only has work to do on cards that are soulbonded,
face-down, flipped, transformed, prototyped or carrying counters, and those
are pool facts.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState` or
`Vocab` change is in this pass.

**Sixty-eighth pass. Base `50dfa172` vs tip `46d66933`.** Four perf commits
and one bug fix, all one question: **what does a rollback, a gate, or a probe
cost when it has nothing to do?** Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--threads 1 --seed 1`. The base column reproduces the sixty-seventh pass's
tip to four digits on this box (`cube` 2,700,797,247 against its
2,700,791,689), so the two blocks are comparable.

```text
                          base (50dfa172)   tip (46d66933)
I refs, --decks fixed     1,167,057,320    1,155,019,903   -1.032 %
I refs, --decks sos       1,501,695,839    1,488,365,013   -0.888 %
I refs, --decks cube      2,700,797,247    2,650,054,606   -1.879 %
```

| step | commit | fixed | sos | cube | what |
|---|---|---|---|---|---|
| A | `a585bff2` | -0.088 % | -0.190 % | **-0.866 %** | a payment rollback rewrote every tapped flag it had snapshotted |
| B | `5c0b07cc` | **-0.716 %** | **-0.548 %** | **-0.838 %** | the damage funnel asked the battlefield seven times per damage event |
| C | `a951b378` | -0.062 % | -0.049 % | -0.038 % | the layer pass asked three questions of the effect list, once per permanent |
| — | `86ec1bd8` | — | — | — | **bug**: the payment snapshot keyed on owner where auto-tap taps by controller |
| D | `46d66933` | -0.168 % | -0.103 % | -0.147 % | every cast asked the battlefield three more times, for three name locks |

**`cube` is 1.8x `fixed` and 2.1x `sos`** and A and B both drive that: A's
waste is proportional to how many permanents the payer controls, and B's to
how many static abilities the board carries. Full write-up, the three
refutations and the rules in **Log**.

**The bug fix is cost-neutral and was measured rather than assumed** —
`snapshot_payment_state`'s self cost is **byte-identical (1,270,624) across
the two binaries** and its inclusive cost moves 0.19 %, so D's column is D's.
It is `owner` -> `controller` on one filter: auto-tap taps what the payer
*controls*, so a stolen mana source a failed payment had tapped was never in
the snapshot and never came back. Invisible through a cast (`perform_action`'s
checkpoint undoes the whole action on `Err`) and visible through
`Effect::PayOrLoseGame`, which handles its own payment failure and carries on.
Regression test `core_rules::game::failed_payment_untaps_a_stolen_mana_source`
— **checked against the old filter, where it fails**; the first version of it
went through a cast and passed either way, which is the vacuous-test tell this
branch keeps re-learning.

**Wall clock, `--decks cube`, `scripts/ab_wall.py`, 6 ABBA blocks of
`--games 1500 --threads 4`, `release-fast` both sides, A = base
`50dfa172`. Two sittings, because the first had no valid null:**

```text
                          mean B/A   95 % CI            blocks B faster
A/B  B = A+B (5c0b07cc)   0.9788     -3.72 .. -0.53 %   6/6
A/B  B = A..C (a951b378)  0.9667     -6.03 .. -0.63 %   5/6
null control (base/base)  1.0012     -3.19 .. +3.42 %   4/6   FLAT
```

Both A/B rows predate `46d66933`, which adds another -0.147 % of `cube` in
Ir — inside the noise of either interval, so neither was re-run.

**The honest statement is "2-3 % on `cube`", not one number**, and the null
is why: it comes back flat but says this sitting cannot resolve anything
smaller than **+/-3.3 %**, so the second run's -3.33 % sits at its own
resolution. What carries the claim is that the two A/B sittings are
independent, agree in sign, and are **11 of 12 blocks faster** against a null
that splits 4/6.

**And the wall win is larger than the Ir win, which is the opposite of this
file's usual caution.** Ir reads -1.73 % on `cube`; the clock reads 2-3 %.
The standing note says Ir *over*-reads by ~1.7-2.8x — that is calibrated on
passes that remove battlefield walks and keyword scans, which are pure
instruction count. This pass's largest commit removes **`Arc` deep copies**:
an unshare is a handful of instructions plus an allocation and a cold write
over a `PlayerData`/`CardData`-sized object, so the machine pays cache misses
callgrind does not model. **A pass that removes clones should be sized on the
clock, not on Ir.**

**A null that resolves +/-3.3 % is worse than the +/-1 % this box gave the
sixty-fourth pass**, and the difference is the workload, not the hour:
`--decks cube --games 1500` is ~43 s a run against that entry's `--decks sos
--games 2000` at ~28 s, on four threads either way, and the cube pool's deck
build is seed-dependent in *content*. Use `sos` for a null-limited
comparison; use `cube` when the effect is one only a grant-heavy board
carries, as here, and accept the wider interval.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
decisions_per_game 613.2
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok (all pairs split); thread_determinism ok (3 vs 1 identical)
suite            19,007 passed / 0 failed / 5 ignored, 19 test binaries
golden traces    7 passed, all unchanged
clippy           `--workspace --all-targets` clean (client excluded — see below)
peak_rss_mib     26.9 / 26.9 / 26.9 over three `--bench` runs (release-fast, mimalloc)
games_per_s      200.34 / 214.93 / 222.69 at host_calib_ms 46-50
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores
```

**`--bench` does not resolve this pass and the three runs above say why**:
200.3 to 222.7 games/s is an 11 % spread within one binary on one box in one
minute, against an `--decks fixed` effect of -1.03 % in Ir. The wall-clock
claim is the `--decks cube` ABBA block above, which pairs the two binaries
run-for-run; `--bench` is here for `decisions`, `stalls` and determinism,
which are exact.

**Crash-freedom on the `dev` profile rather than `overflow`, on purpose.**
Commit B's soundness rests on the mask being a superset of what any gate can
find, and the two gates that ask more than "does this variant exist on the
battlefield" carry a `debug_assert!` that is compiled out of every release
profile — including `overflow`. `dev` carries **both** `debug-assertions` and
`overflow-checks`, so it is the build that audits this change. Same grid as
the standing recipe, seeds 11 and 12:

```text
--decks all     --games 100   1,700 decided / 0 undecided per seed
--decks cube    --games  60     480 decided / 0 undecided per seed
--decks sealed  --games  60     720 decided / 0 undecided per seed
```

**5,800 games, no panic, no arithmetic overflow, no assertion**, and the same
grid re-run at the tip after C and again after the bug fix + D reads the same
six lines each time. The count
is lower than the usual 11,600 because `dev` runs the engine at opt-level 0
(~10-12 games/s against `release-fast`'s ~200); the trade is deliberate —
these games check the invariant the release grid cannot see. The 19,006-test
suite runs under the same assertions.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState` or
`Vocab` change is in this pass.

**Sixty-third pass. Base `0036e238` vs tip `fa3bf671`.** Two commits, one
class: **a loop over pairs charged per pair for facts belonging to one side
of the pair.** Ir readings `profiling-fast --no-default-features`,
callgrind, one thread, `--a gang --b gang --games 6 --threads 1 --seed 1`.
Both binaries built at `0036e238`; the three commits the rebase put
underneath are documentation only (no `.rs` file differs between
`0036e238` and `c2cc6c01`), so the columns are comparable to the current
parent as measured.

```text
                          base (0036e238)   tip (fa3bf671)
I refs, --decks fixed     1,182,567,955    1,175,724,194   -0.579 %
I refs, --decks sos       1,530,678,137    1,523,856,909   -0.446 %
I refs, --decks cube      2,768,347,971    2,732,667,632   -1.289 %
```

| step | commit | fixed | sos | cube | what |
|---|---|---|---|---|---|
| A | `d9f459de` | -0.425 % | -0.359 % | **-0.791 %** | six attacker facts and two blocker facts read per (blocker x attacker) pair |
| B | `fa3bf671` | -0.154 % | -0.087 % | **-0.502 %** | block legality resolved both sides of the pair, per pair |

**`cube` is 2.2x `fixed` and 2.9x `sos`**, which is the pool-ratio device
paying off: `pick_blocks_inner` was the 2.09x row in the sixty-second pass's
ratio table, and a grant-heavy pool has wider boards, so the pair count
grows quadratically where the rest of the game loop grows linearly. Full
write-up, rows and the rule in **Log**.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
decisions_per_game 613.2
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok (all pairs split, rho -1.000 on all 160);
                 thread_determinism ok (3 vs 1 threads identical)
ladder printout  identical on fixed / sos / cube at both commits
peak_rss_mib     27.1 / 27.2 / 27.2 over three `--bench` runs
suite            18,736 passed / 0 failed / 5 ignored, 14 test binaries
golden traces    7 passed, all unchanged (both commits)
clippy           `--workspace --all-targets` clean, all eight crates
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores, host_calib_ms 50-69
```

**The `peak_rss_mib` and `games_per_s` rows are `release-fast`, not
`release`, and neither compares to a row above.** Three `--bench` runs read
**202.12 / 195.87 / 201.33 games/s** at `games_per_s_th` 67.4 / 65.3 / 67.1;
no base binary was built at this profile in this sitting, so there is no
pair and no claim. The RSS figure carries mimalloc (default features) on the
2.80 GHz box — read (-48) before comparing it to the 24.0-24.3 the other
session recorded on the 2.10 GHz one.

**No wall-clock pair is quoted.** At Ir's measured ~1.7x-2.8x over-read,
this pass is worth roughly -0.25 % of wall on `sos` and -0.45 % on `cube` —
well under the +/-2 % this box resolves at eight ABBA blocks. Nothing here
is allocation-shaped: the allocator family and `__memcpy` are flat to four
digits on both pools, so there is no "Ir counts a memcpy, the machine barely
does" discount either. What came off is battlefield walks and keyword scans.

**Crash-freedom and determinism at the tip, on the wider recipe.**
`overflow` build (release-fast + `overflow-checks`), `--a gang --b gang
--threads 3`, seeds 11 and 12:

```text
--decks all     --games 200   3,400 decided / 0 undecided per seed, 1,700 pairs all split
--decks cube    --games 120     960 decided / 0 undecided per seed,   480 pairs all split
--decks sealed  --games 120   1,440 decided / 0 undecided per seed,   720 pairs all split
```

**11,600 games, no panic, no arithmetic overflow, `rho -1.000` on every
pair.** Both commits are in combat code, which `--decks all`'s seventeen
fixed archetypes exercise heavily; `cube` and `sealed` are the pools that
can put a Rampage / Menace / protection / indestructible attacker into the
same board as a first-striker, which is the shape the `AttackerFacts` hoist
would break if it broke anything.

**Two commits landed *after* the tip these columns were measured at, and
they are a rules fix, not a perf change.** The target-walker class closed at
this tip (`core_rules::target_walkers` 39 -> 0: nineteen shipped cards
declared a `TargetFiltered` slot no walker surfaced, so their targeted
effects resolved against an empty list). None of the nineteen is in the
`--bench` archetypes or the golden-trace decks — decisions, turns, stalls and
all seven traces are unchanged — but they *are* in the cube and sealed pools,
so a `--decks cube` or `--decks sos` Ir total taken after `1399e86b` is not
comparable to the columns above. Re-base before quoting one.

Re-checked at `aaadfdc2` on the `overflow` build, same grid as above:
`--decks all --games 200` 3,400 decided / 0 undecided per seed, `cube` 960,
`sealed` 1,440 — **11,600 games, no panic, no arithmetic overflow, every pair
split**, and `--bench` `decisions` still **196,220** byte-identical with
`turns_per_game` 27.53 and zero stalls. Nineteen cards that used to resolve
against an empty target list now bind one, and nothing in the pools noticed.

Re-checked again at `2ad8b397` (the block-trigger class), same grid, same
two seeds: **11,600 games, no panic, no arithmetic overflow, `rho -1.000` on
every pair**, and `--bench` `decisions` **196,220** with `turns_per_game`
27.53 and zero stalls on both the mimalloc and system-allocator builds. That
commit adds a `SelectionRequirement` arm and rewrites five shipped block
triggers; none of the five is in the `--bench` archetypes or the golden-trace
decks, and the `cube` / `sealed` legs are the ones that can draw them.

Re-run once more at `b1a772ec`, which is the widest-blast-radius rules change
of the three: it moves the **auto-targeter's fallback** (the picker now aims
with `target_filter_for_slot(0)` before `Any` when `primary_target_filter` is
silent), so every auto-targeted spell, activation and trigger in every pool
goes through the changed line. Same grid, same two seeds: `all` 3,400 decided
/ 0 undecided per seed, `cube` 960, `sealed` 1,440 — **11,600 games, no panic,
no arithmetic overflow, `rho -1.000` on every pair**. `--bench` `decisions`
**196,220** byte-identical over two runs with `turns_per_game` 27.53, zero
stalls, `peak_rss_mib` 26.9 / 28.7, and all seven golden traces unchanged.
**A behaviour-preserving reading is the correct one here and it is not a
coincidence:** the nine cards the pass fixes are absent from the bench
archetypes and the trace decks, which is exactly why they went unnoticed, and
the fallback only fires where the primary walker returns `None` — every such
pick previously aimed at `Any` and was then re-checked against the same slot-0
filter the picker now uses, so a pick that survived CR 608.2b before still
survives it.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass.

**Sixty-first pass (the other session's line, run concurrently with 59 and
60). Base `ba15f249` vs this block's tip.** Four commits, one class and one
defect: **four questions the simulator asked with a walk or an iterator chain
it did not need to build.** All three pools move, which is the difference
from pass 58 — nothing here is on the deck builder. Ir readings
`profiling-fast --no-default-features`, callgrind, one thread, `--a gang --b
gang --games 6 --threads 1 --seed 1`.

**`ba15f249` is the base both columns were measured at, and pass 60's
`6344adf6` is in neither.** That commit (the deck-fill `CardDefinition`
memcpy, -2.9 % on `sos` by itself) landed underneath during the rebase that
brought this block onto the branch; it is on `cube::card_arc` and the deck
fill, which none of the four commits here touch, so the deltas stand — but
**do not diff the current tip against the current base and expect these
numbers.** Name the base, as this file has had to say four times.

```text
                          base (ba15f249)   tip (this block's)
I refs, --decks fixed     1,218,195,816    1,206,204,087   -0.984 %
I refs, --decks sos       1,593,831,453    1,580,084,804   -0.862 %
I refs, --decks cube      2,866,729,876    2,841,539,263   -0.879 %
```

**One caveat added after the fact, and it makes these numbers conservative:**
the sixty-second pass found that `name_index()` builds 22,568
`CardDefinition`s at startup — **104.7 M Ir, 6.8 % of a six-game `sos`
total** (see (-46)). Both columns above carry it, so the deltas are sound,
but the *shares* are diluted: net of startup, `sos` reads -0.923 % rather
than -0.862 %.

**The base column is a cross-check as well as a base.** `ba15f249`'s own
commit message reads `fixed` 1,218,193,228 and `sos` 1,593,828,683 for the
same commit, measured by the other session; this reading is 2,588 and 2,770
Ir above them, which is argv length and nothing else (see pass 49's note).
Two sessions, two containers, two build directories, the same binary
behaviour to five digits.

| step | commit | fixed | sos | what |
|---|---|---|---|---|
| A | `e0e64d12` | **-0.262 %** | -0.205 % | the combat-damage dispatch's `AnyPlayer` leg `collect()`ed an always-empty `Vec` |
| B | `7f8f94d2` | +0.012 % | +0.014 % | **defect**: converge had two oracles, disagreeing in both directions |
| C | `2336817d` | **-0.641 %** | **-0.564 %** | three whole-battlefield / command-zone walks at the top of every SBA sweep |
| D | `c676a229` | -0.095 % | -0.100 % | C's `sculptor` bit folded into the keyword walk the scan already does |

**The per-step column was measured on this session's own line, before the
rebase**, i.e. on `ff929e7f` plus a fifth commit of this session's that the
rebase dropped (below). The three-pool pair above was re-read at the rebased
base and tip, and it is the number to quote — but the two agree: the steps
sum to **-0.986 % on `fixed`** against the pair's -0.984 %, and **-0.855 %
on `sos`** against -0.862 %. The concurrent session's commits underneath are
on different code, and this is the check that says so.

**And the fifth commit is the reason to read this block before starting
anything: two sessions found `loop_fingerprint` in the same hour, and the
other one was right.** Both saw that the CR 104.4b watchdog's digest ran
`DefaultHasher` (SipHash-1-3) over ~84 small integers per call, 0.78 % of a
six-game `sos` run. This session replaced it with the engine's vendored
`FxHasher` and measured `sos` -0.706 %; the other replaced it with
**SplitMix64's finalizer** (`ba15f249`) for -0.578 % on its own base — and
its argument is the correct one. `fxhash`'s own doc says it is not
collision-resistant, and it exists for *map iteration determinism*; this
digest decides a **draw**, where the function's comment says a false positive
ends a live game. A cheaper hash with a weaker avalanche is the wrong trade
there even though the Ir was better. **This session's commit was dropped in
the rebase.** The rule, and it is new: when two sessions land the same
finding, keep the one whose *argument* survives, not the one whose number is
larger.

**B is a bug fix that costs 0.01 % and the number is quoted so nobody has to
re-derive it.** `CardDefinition::wants_converge` scanned the definition's
Debug rendering for `ConvergedValue` and missed converge's other spelling,
`SelectionRequirement::ManaValueAtMostConverged` — so **Bring to Light** and
**Sundering Archaic**, whose converge is entirely in a target filter, were
paid for with the mana-conserving order that the function's own doc says
"routinely counted one color on a five-color board". Measured directly rather
than by row attribution: on the pre-rebase line a base binary carrying only
this commit read `fixed` 1,220,954,533 against 1,220,812,183, so it is
**+142,350 Ir, +0.0117 %** — the second substring scan, once per card *name*
per process, amortized to nothing over a training run.

**The rows that moved, tip against base (pre-rebase line, `fixed` / `sos`):**

```text
                                 fixed                     sos
check_state_based_actions  28,960,956 -> 24,033,162   43,928,234 -> 37,493,708
Vec::from_iter (all monos) 37,336,782 -> 33,571,904   36,400,225 -> 31,767,323
Map::try_fold               6,167,108 ->  2,495,168    (same shape)
sba_board_scan             20,935,578 -> 20,927,838   28,013,796 -> 28,032,252
```

`sba_board_scan` is flat: the four bits C added (`shapeshifter`,
`sector_set`, `sculptor`, and D's fold of the last one) cost what the walks
they replaced cost per card, and the win is the three walks that stopped
happening at all. `Vec::from_iter` attributed to
`check_state_based_actions` went **53,362 calls / 21,358,086 Ir to 13,576 /
15,880,576** on `sos` — 4.02 collects a sweep down to 1.02.

**One thing was built, measured and reverted, and it is the pass's second
finding.** Serving `card_type_change_unscoped`'s battlefield leg off
`sba_board_scan` reads **+0.295 % on `fixed` / +0.255 % on `sos`**:
`card_type_change_unscoped` comes off in full (-5,948,694 on `sos`) and
`sba_board_scan` goes up **9,249,352** to pay for it. The standalone
`any(card_can_change_card_types)` short-circuits per card; a scan bit has to
run `static_effect_changes_card_types` over every static ability of every
permanent whether or not the answer is already known. **That is the third
refutation of the (-6) fusion device inside `creature_death_possible`
alone** (+0.55 % and +1.24 % are the other two, already in the code
comment). The rule it yields: *a presence bit belongs in a shared scan only
when the question has no early exit of its own.*

**And the shape all the wins share, which is the one to carry forward: ask
what the answer costs when it is "no".** None of these was a hot function.
A `filter`/`flat_map`/`filter`/`map` stack over an empty result, a
`flat_map` over two empty command zones, a battlefield `filter` for a card
nobody's deck contains — and, both sessions independently, a SipHash of ~84
small integers for a digest that is only compared with itself. Every one is
the cost of *asking*, paid on every sweep or every dispatch, on a board
where the answer is always no. `cg_edges.py --callers SpecFromIterNested`
found two of them in one table: rank the collect sites by *calls*, then ask
which of them can return non-empty on the bench pools.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok (all pairs split); thread_determinism ok (3 vs 1 identical)
peak_rss_mib     30.0 / 30.1 / 30.1 over three `--bench` runs
suite            18,736 passed / 0 failed / 5 ignored over 14 test binaries
                 (11 of them carry tests), at every commit
golden traces    7 passed, all unchanged (every commit)
clippy           `--workspace --all-targets` clean, all eight crates
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores, host_calib_ms 52-60
```

**No wall-clock pair is quoted, and the concurrent session's Ir-vs-clock
measurement is why.** That reading (see "How to measure") puts Ir at ~1.7x
the clock on `sos` and ~2.8x on `cube`, so these four commits are worth
roughly **-0.5 % of wall time on `sos` and -0.3 % on `cube`** — a quarter of
the +/-2 % this box resolves at eight ABBA blocks, and exactly the "under
~3 % of Ir is unseparable" case that note describes. The Ir column is the
attribution. None of the four is allocation-shaped —
`malloc`/`free`/`_int_malloc` are flat to five digits on both pools — so
there is no "Ir counts a memcpy, the machine barely does" discount on top:
what came off is iterator-adapter frames and battlefield loads.

**Crash-freedom and determinism at the tip, widest pool.** `release`, `--a
gang --b gang --games 200 --threads 3 --decks all`, seeds 11 / 12 / 13:
**10,200 games, 10,200 decided, no panic**, and all 5,100 mirrored pairs
split (`rho -1.000` on every seed). `--decks sealed --games 200 --seed 11`:
**2,400 decided, 0 undecided**. `CRAB_THREAD_CHECK=1 --bench` reads
`thread_determinism ok (3 vs 1 threads identical)`.

**The `release` throughput reading, for the record and not as a claim.**
Three `--bench` runs at the rebased tip: **171.72 / 167.88 / 167.94
games/s**, `decisions_per_s` 105,299 / ~102,900, at `host_calib_ms` 58 / 54 /
64. (The pre-rebase tip read 162.69 / 162.72 / 167.98 at calib 52 / 60 / 56,
two hours earlier on the same container — which is the spread this file keeps
warning about, not a result.) No base binary was built at `release` in this
sitting, so there is no pair, and the standing rule is that a cross-sitting
`games_per_s` difference is not evidence. What the run attests is the
invariants above it.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass.

**Sixtieth pass, base `58346b57` vs tip `6344adf6`, two commits, both about
a copy nobody needed.** `--decks sos` **-3.46 %**, `cube` **-2.82 %**,
`fixed` **-2.16 %**, `sealed` **-2.33 %**, and **peak RSS on `--bench`
21.9 -> 17.7 MiB, -19 %**. Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--seed 1`.

```text
                       base (58346b57)   (A) ba15f249    tip (B) 6344adf6
I refs, --decks sos      1,603,088,243   1,593,828,683   1,547,629,927  -3.460 %
I refs, --decks cube     2,878,150,309   2,866,725,942   2,797,004,247  -2.820 %
I refs, --decks fixed    1,219,893,985   1,218,193,228   1,193,591,244  -2.156 %
I refs, --decks sealed     3,276,783,848 (read at (A))   3,200,503,032  -2.328 %
deck build alone              21,800,071 (read at (A))      21,871,968  +0.330 %
peak_rss_mib, --bench               21.9                          17.7    -19 %
  ^ SYSTEM allocator (--no-default-features), like the Ir rows above it.
    The shipped mimalloc build reads ~24 MiB at the same tip — see
    "peak_rss_mib is an allocator reading too" in How to measure.
```

| step | commit | sos | cube | fixed | what |
|---|---|---|---|---|---|
| A | `ba15f249` | -0.578 % | -0.397 % | -0.139 % | the CR 104.4b loop watchdog ran SipHash over fifty small integers |
| B | `6344adf6` | **-2.899 %** | **-2.432 %** | **-2.019 %** | a deck-fill memcpy'd an 8,232-byte `CardDefinition` per card |

**(B) is the pass, and the number that found it is a struct size.**
`CardDefinition` is **8,232 bytes**; `CardInstance::new` takes
`impl Into<Arc<CardDefinition>>`, and every deck-fill site handed it a fresh
`f()`, so `Arc::new` copied all of it once per card in a library. In the sos
profile `CardInstance::new` is the second-largest `__memcpy` caller — 3,452
memcpys / 28,451,728 Ir, **8,242 Ir apiece** — and that per-call figure is
what named the cause: a memcpy that expensive is copying kilobytes, and only
one thing in this engine is that big. `cube::card_arc(f)` memoizes one
`Arc<CardDefinition>` per factory per thread; a deck-fill is now a refcount
bump. It is sound because `Arc<CardDefinition>` is already the CoW handle for
a definition — the ~twenty sites that rewrite one all go through
`Arc::make_mut`.

**The RSS half is the same change seen from the other side** and matters more
to `selfplay_train` than the Ir does: a forty-card deck holds ~twenty distinct
definitions instead of forty, and an actor count multiplies it.

**The one column that goes the wrong way is the cold deck-build
microbenchmark, +0.330 %, and the first version of (B) read +6.591 % there.**
Routing `card_arc` through `card_brief`'s memo made a *miss* also pay
`CardBrief::of` — the pip counts, the keyword walk, `is_fixing_card`'s whole
effect-tree walk — and `--decks sealed --games 1` is eighty template cards
that are all misses and no games. Its own memo costs a map lookup and an
`Arc` allocation, which is the +0.330 %. **A long-lived actor pays that once
per factory ever**; the microbenchmark is a cold process by construction.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok; CRAB_THREAD_CHECK=1 ok (3 vs 1 threads identical)
ladder output    the full printout diffs identically on fixed, cube, sos
                 and sealed
suite            18,735 passed / 0 failed / 5 ignored over 14 result blocks
golden traces    all unchanged
clippy           `--workspace --all-targets` clean
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores
```

**Fifty-eighth pass, base `c18552fd` (pass 57's tip) vs its own tip
`13f3521c`.** Four commits, one class: **the sealed builder's last three
per-shape re-derivations, plus the land walk that rediscovered what the pile
builder already knew.** The workload that moves is `--decks sealed --games 1`,
which plays no games and so is deck construction and nothing else:
**26,478,634 -> 23,574,309, -10.968 %.** Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, one argv throughout.

```text
                          base (c18552fd)   tip (13f3521c)
deck build (sealed, 1 g)     26,478,634       23,574,309   -10.968 %
I refs, --decks fixed     1,234,918,094    1,233,675,802    -0.101 %
I refs, --decks sos       1,658,496,337    1,657,624,397    -0.053 %
I refs, --decks cube      2,952,041,099      not re-read — no commit is on that path
```

| step | commit | deck build | what |
|---|---|---|---|
| A | `811cddec` | **-3.738 %** | the splash ranker reads the pool's memoized colours and scores |
| B | `432976a0` | -0.599 % | one `PoolScores` per pool, not one per random build |
| C | `88b97f25` | **-2.828 %** | fifty-seven shapes share three sorted orders |
| D | `13f3521c` | **-4.246 %** | the land walk reads the index the pile builder kept |
| — | *rebase onto pass 57's (D)+(E)*; the same four re-read `23,611,357` | | |
| E | `15cad53b` | **-7.782 %** | the copy cap counts by dense card id, not by `HashMap` |

**And a sixth commit that is not on the deck builder at all** — `8a0f11fb`,
candidate (-42)'s answer: `do_untap`'s per-turn seat reset writes ~55 fields
through `Player`, which is a CoW handle, so each was its own
`Arc::make_mut`. `make_mut` calls in that function **212,012 -> 80,148**;
`--decks sos` **1,644,049,924 -> 1,639,754,965, -0.261 %**, `--decks cube`
**2,910,850,945 -> 2,903,490,499, -0.253 %**. Same recipe, base re-read at
the tip that carries (E); a concurrent session's `cae6b605` (the grant list's
deep copy) then landed underneath it, and is not in either column. Two
cheaper explanations were built and refuted first; the line profile named
`Player::deref_mut`, not a hot line. See the Log and (-42).

**And a seventh, `cae6b605`, from the third concurrent session: the grant
list was deep-copied for callers that only ever read it.**
`granted_abilities_of` collected a permanent's granted activated abilities
into a `Vec<ActivatedAbility>` — a whole `Effect` tree, a cost and a dozen
filters per element — and its three callers either folded the list or wrapped
each element in `AbilityRef::Synth(Box::new(a))`, a second allocation, and
read it through `Deref`. Every source it draws from is reachable from
`&'a self`, `me` or the scan, so it returns `Vec<&'a ActivatedAbility>`; the
one ability with nothing to borrow from (CR 804.2's deploy-creatures grant,
all-constant fields) is a `LazyLock`. Base `7c7f2e5e` re-read at the tip that
carries (E) and `8a0f11fb`:

```text
                     base (7c7f2e5e)   tip (cae6b605)
I refs, --decks fixed  1,233,122,037   1,230,058,286   -0.248 %
I refs, --decks cube   2,910,851,269   2,896,273,896   -0.501 %
I refs, --decks sos    1,644,050,093   1,612,056,291   -1.946 %
```

The function's own callee table on `sos`: `ActivatedAbility::clone` **11,324
calls / 7,868,320 Ir** and `__memcpy` **11,324 / 2,501,090** both go to zero,
`RawVec::grow_one` **11,324 / 8,485,733 -> 9,782 / 1,252,111** (the `Vec` now
grows by pointers), and `evaluate_requirement_static_hinted` is unchanged at
39,430 / 19,241,574.

**And the clock cannot see any of it.** `scripts/ab_wall.py`, `release-fast` +
mimalloc, `--games 2000 --decks sos --threads 4`, **eight ABBA blocks**: mean
B/A **+0.18 %**, 95 % CI **-1.64 .. +2.00 %**, 3/8 blocks faster — and the
null control on the same workload reads -0.40 % with a comparable CI, so the
box's floor is +/-2 % and the effect is under it. The same workload on the
*system*-allocator binaries (`profiling-fast`) is flat too, which rules out
"mimalloc already made the allocation cheap" as the explanation. **Two earlier
readings of this same pair said otherwise and both were noise**: best-of over
nine hand-rolled pairs said +2.5 % slower, four ABBA blocks said +1.26 %
slower. That is what the tool exists for.

**This is pass 57's clock rule with its mechanism named, and it is the rule
this file should carry forward: `Ir` counts a `memcpy`; the machine barely
does.** A deep copy of a contiguous struct runs at high IPC out of a
just-written cache line, and the borrow that replaces it turns a hot-buffer
read into a pointer chase into the (cold, scattered) card definitions. The
commit is **kept** — the Ir is real, the ladder printout and the 18,728-test
suite are identical, and not deep-copying an immutable shared structure in
order to read it is a clarity win on its own — but **-1.95 % is not a
throughput claim**, and the next allocation-shaped candidate on this branch
should be sized with that in front of it.

**And an eighth, `4382bd43`, the rest of (-42)'s class** — `do_untap`'s tail, where the
runs of seat writes are split by `retain_cold!` calls on `self`. `make_mut`
in that function **80,148 -> 41,200**; `--decks sos` **1,607,757,957 ->
1,605,824,543, -0.120 %**, `--decks cube` **2,888,913,466 -> 2,885,591,189,
-0.115 %**. **Both base columns here are re-read at `d2a8320b`, i.e. with
`cae6b605` in** — quoting the sixth commit's `1,639,754,965` instead would
have read -2.069 %, seventeen times the real win, and the seventh commit
directly above is where that missing -1.946 % went. See (-42).

**And a ninth, the same device off the untap step entirely**, at the three
sites a call-count sweep of `make_mut`'s callers found: `advance_step`'s
cleanup-step seat reset **51,142 -> 9,198 (-82.0 %)**,
`deal_combat_damage_to_target` **21,012 -> 10,008 (-52.4 %)**, and
`clear_step_bounded_may_play`'s per-seat zone sweep. `--decks sos`
**1,605,824,543 -> 1,604,031,880, -0.112 %**, `--decks cube`
**2,885,591,189 -> 2,882,468,355, -0.108 %**. **Program-wide the three
CoW-handle commits take `make_mut` on `sos` from 582,552 calls to 475,676,
-18.3 %.**

**And a tenth, a second `--callers make_mut` sweep for the cheap-per-call
rows.** `finalize_cast`'s ten per-seat cast tallies take three bindings (a
`&mut self` call and two `GameState` field writes split the run)
**46,992 -> 26,474**, and `on_left_battlefield`'s six unconditional
`cast_from_*` clears take the gate-then-bind shape already used by the four
`CardCold` clears above them, **24,352 -> 8,494**. `--decks sos`
**1,604,031,880 -> 1,603,018,685, -0.063 %**, `--decks cube`
**2,882,468,355 -> 2,880,726,428, -0.060 %**; program-wide `make_mut`
**475,676 -> 439,300, -7.6 %**.

**So across the four CoW-handle commits `make_mut` on `sos` goes
582,552 -> 439,300, -24.6 %**; what is left, and the reason half of it is
*not* this device, is the new candidate (-43).

**Both game pools read slightly *down* rather than flat, and the reason is
the binary.** No commit here is on the game loop; `_dl_relocate_object` is
2.31 % of the deck-build workload and `build_shape` shrank, so the process
startup floor moved with it. Read -0.101 % and -0.053 % as "unchanged, and
the code got smaller", not as a game-loop win.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism      ok (all pairs split); thread_determinism ok (3 vs 1 identical)
peak_rss_mib     22.0
suite            18,728 passed / 0 failed / 5 ignored (A-D, at (E), and at each
                 of the four CoW commits). **11 binaries actually run tests**;
                 the per-binary sum is 553 + 2 + 2 + 6,087 + 1,724 + 370 +
                 3,464 + 1,858 + 1,813 + 649 + 2,206. The "22" this file has
                 carried since the 18,712 era is cargo's result-line count,
                 which includes the zero-test lines a concurrent session is
                 removing — so count the nonzero lines, not the lines.
golden traces    7 passed, all unchanged
clippy           `--workspace --all-targets` clean, all eight crates
throughput       119.8 games/s warm (`selfplay_train --actors 3 --games 120
                 --steps 1 --seed 7`), six consecutive runs within 0.1 %, 0
                 stalls. An absolute at this tip, not an A/B claim — no
                 commit in this pass is sized off it. See "How to measure".
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores, host_calib_ms 58-73
```

**Crash-freedom and determinism at the tip, widest pool.** `--a gang --b gang
--games 400 --threads 3 --decks all`, seeds 11 / 12 / 13: **20,400 games,
20,396 decided, no panic**, and all 10,198 mirrored pairs split
(`rho -1.000` on every seed). The 4 undecided are seed 11's standing rules
draws, the same four every pass since the forty-fourth has recorded. **Re-run
at the tip that carries the three CoW-handle commits: identical — 3,398 /
3,400 / 3,400 pairs, 0 A-sweeps, 0 B-sweeps, every pair split.**

Behaviour beyond the bench: `--decks sealed --games 6` and `--decks all
--games 20 --seed 11` are **byte-identical to the base at every one of the
four steps** apart from the wall-clock line. That pair is the check a
deck-builder change needs and the golden traces cannot give — they play
hand-built decks, so the builder never runs in them.

**No wall-clock pair is quoted, and the arithmetic is why.** Deck
construction is ~2.2 M Ir per pool-plus-build against ~50 M for a sealed
game, and a `selfplay_train` actor builds two decks per game — so the whole
builder is ~8 % of an actor's per-game work and this pass is **~0.9 %** of
it. That is an order of magnitude under what `--bench` resolves on this box
(an 11 % spread between runs of one binary), which is the case this file says
callgrind exists for.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState` or
`Vocab` change is in this pass; the decks the builder produces are
byte-identical.

**(A)-(D) were read at `13f3521c`; two of pass 57's commits then landed on
top** (`7b0477d4` and `2394206c`, the gather's two `Vec`s) and (E) is measured
above them. Nothing in this pass is on the gather and nothing in theirs is on
the deck builder, so the two compose — but their two commits move the
deck-build workload from **23,574,309 to 23,611,357** (+0.157 %, code size on
a workload whose `_dl_relocate_object` is 2.5 % of the total), which is why
(E)'s base is re-read rather than chained. **The pass end to end is
`26,478,634 -> 21,774,018`, -17.77 %**, and the 37,048 Ir their commits add is
inside that.

**A `--decks sealed --games 1` absolute moves with the size of the binary, not
just with the deck builder.** Two commits that touch neither `recommend.rs`
nor `selfplay.rs` moved it 37 k. Re-read the base after any rebase before
quoting a delta on this workload.

**Anchors at the branch tip** (this pass's (E) on top of pass 57's (D)+(E)),
same recipe, for whoever measures next: `--decks sos` **1,644,049,924**,
`--decks cube` **2,910,850,945**, deck build **21,774,018**.

**Fifty-seventh pass, base `28ae2416` (pass 56's tip) vs its own tip
`6c5dd0ab`.** Five commits in two classes. **The simulator's two largest
engine functions each ended in a fan of narrow walks that ask a question the
board has already answered** — (A) `cg_lines.py --rows`, because the shape is
a run of identically-costed rows below the default print; (B) the gather's
thirty-eight per-static passes get a variant bitmask; (C) the trigger
dispatcher stops evaluating grant filters for grants no event in the batch
could fire. Then **the gather allocated twice for every effect it emitted** —
(D) and (E), landed by a second session and rebased on top of (C). **`--decks
cube` -7.97 %, `sos` -4.21 %, `fixed` +0.51 %.** Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--seed 1`.

```text
                       base (28ae2416)   (B) da30d1c2    (C) 353105fe     tip (E)
I refs, --decks cube     3,162,426,135   3,082,752,911   2,952,044,117   2,910,120,990  -7.973 %
I refs, --decks sos      1,715,663,129   1,656,877,045   1,658,498,742   1,643,320,227  -4.216 %
I refs, --decks fixed    1,226,171,600   1,233,007,810   1,234,920,109   1,232,447,924  +0.512 %
I refs, --decks sealed     3,430,701,306   not re-read — neither the gather nor
deck build alone              26,570,012   the dispatcher is on that path
```

| step | commit | cube | sos | fixed | what |
|---|---|---|---|---|---|
| B | `da30d1c2` | -2.519 % | **-3.427 %** | +0.558 % | the gather's thirty-eight per-static passes ask a variant bitmask |
| C | `c6ef9af8` | **-4.241 %** | +0.098 % | +0.155 % | the dispatcher drops a grant no event in the batch could fire |
| D | `603d354b` | **-1.001 %** | -0.634 % | -0.011 % | `static_ability_to_effects` collected a `Vec` its only caller drains |
| E | `6c5dd0ab` | -0.424 % | -0.283 % | -0.189 % | thirty-three arms returned `vec![one]` per emitted effect |

**(D) and (E) were written concurrently with (B) and (C) on `28ae2416` and
read -0.937 / -0.301 % on cube there; on top of (B)+(C) they read -1.001 /
-0.424 %.** They are worth *more* after the mask, on every pool, which is the
composition to expect: the mask took the walking out, so what is left of the
gather is a larger share allocation. They are the only rows in this pass that
move `--decks fixed` **down**.

**The columns above were read on `353105fe`; the second rebase then put pass
58's four deck-builder commits underneath (D) and (E).** Re-read at the true
branch tip `7918d1e6`: fixed **1,233,103,048**, cube **2,910,805,300**, sos
**1,643,976,414**. Against `353105fe` that is -0.147 / -1.397 / -0.876 % for
pass 58 + (D) + (E) together, against -0.200 / -1.420 / -0.915 % for (D)+(E)
alone — i.e. pass 58 is within 0.05 % of flat on all three *game* pools, which
is what a deck-builder change should be, and the two do not overlap.

**The two rows are on different pools and that is the pass's rule.** `sos`
carries the static abilities the gather walks; `cube` carries the
`GrantTriggeredAbility` statics the dispatcher walks (`--decks fixed` carries
neither, which is why it only ever pays). Neither commit is visible on the
other's pool, and both are invisible on the committed bench.

**(B) was read once on pass 56's base and again on pass 57's, and the two
passes compose to the third decimal.** It was measured first against
`91f3ede3` (sos -3.422 %, cube -2.476 %, fixed +0.551 %) and re-measured
after the rebase onto pass 56's eight commits: **-3.427 / -2.519 / +0.558**.
Nothing pass 56 removed was already removed here, and nothing here makes
pass 56's rows smaller.

**`fixed` pays and does not collect, and that is the pool, not the gate.**
No permanent in the vanilla archetype decks has a printed static ability *or*
a `GrantTriggeredAbility` static, so `sa_cards` is empty on all 32,002
gathers and `trigger_grants` is empty on every dispatch: both fans of walks
were already free there, and a gate can only add the cost of asking. Three
variants of (B)'s gate were built and measured rather than argued, and two
placements of (C)'s — see the Log. **`sos` is the pool
`crabomination_ml::selfplay_train`'s actors play** (`Vocab::sos_sealed`).

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism      ok (all pairs split, both)
suite            18,728 passed / 0 failed / 5 ignored over 22 binaries
golden traces    all unchanged
clippy           `--workspace --all-targets` clean
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores
```

**Wall clock over the whole pass**, `release-fast` + mimalloc, 600 games /
1 thread / seed 1, interleaved `tip base base tip` so linear drift cancels,
six readings a side, base `28ae2416` vs tip `c6ef9af8`:

```text
--decks cube   base 62.98 63.37 63.96 64.67 65.48 66.20   best 62.98  med 64.3
               tip  61.52 62.59 63.00 63.59 65.12 65.55   best 61.52  med 63.3
                                                          -2.3 % best, -1.6 % median
--decks sos    base 38.33 38.62 39.55 39.64 41.12 41.22   best 38.33  med 39.6
               tip  37.20 37.95 38.06 38.66 39.06 39.12   best 37.20  med 38.4
                                                          -3.0 % best, -3.1 % median
```

**`sos` tracks its Ir (-3.0 % against -3.33 %); `cube` does not (-2.3 %
against -6.65 %), and the gap is what (C) removes.** A requirement evaluation
the batch filter skips is a short, perfectly predicted walk over an
L1-resident grant list — Ir counts every instruction of it and the machine
retires several per cycle. The rule this file has for allocation-shaped
change (Ir over-reads it) has a mirror: **a change that removes cheap,
predictable instructions under-delivers on the clock.** Quote both.

**The pass's first pair was measured separately** (base `91f3ede3` against
(B)) and read cube -4.4 % / sos -4.0 % on best-of-interleaved, on a sitting
where the same workload ran 56-59 s rather than 62-66 s. Neither absolute is
comparable to the other; both deltas are.

**`--bench`, paired** (`release-fast` + mimalloc, 3 threads, interleaved
`tip base base tip`, six a side):

```text
base  189.65 184.15 196.87 202.21 204.62 204.47    mean 197.00  best 204.62
tip   190.49 194.80 198.97 199.28 199.86 204.33    mean 197.96  best 204.33
                                                   +0.5 % mean, -0.1 % best
```

**Read it as flat**, which is what +0.71 % of Ir on that pool has to be
against an 11 % spread. Earlier in the same session a **`release` pair** was
built for (B) — base `91f3ede3` at `release` reads mean 210.48 / best 216.47
against the tip's 212.36 / 225.44 over nine readings a side, also flat, and
**the base binary reads 210 there against the 281 this file records at the
pass-55 tip for the same commit and profile**. A `--bench` absolute is never
a cross-sitting comparison; the anchor is not refreshed.

**Crash-freedom and determinism at the tip.** `release-fast`, `--a gang
--b gang --games 200 --threads 3`, seeds 11/12/13 x `--decks all` plus
`--decks sealed` at seed 11: **12,600 games, every cell decided, 0
undecided, no panic**. `CRAB_THREAD_CHECK=1 --bench` reads
**`thread_determinism ok (3 vs 1 threads identical)`**, `decisions` 196,220,
`turns_per_game` 27.53, `stalls_by` 0/0/0. The same grid ran clean at (B) on
a `release` build.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass.

**Fifty-sixth pass, base `00348ada` vs its own tip `3801f01f`,
re-read on the branch after a second rebase onto pass 55's (I)-(K).** Eight
commits in two classes: **five things the deck builder derived once per
shape from a pool that never changed** (the deck-build workload is
**-23.1 %**), and **three re-reads the game loop paid on every call**. Ir
readings `profiling-fast --no-default-features`, callgrind, one thread,
`--a gang --b gang --games 6 --seed 1`; both columns were read directly on
this box with one argv, so the deltas are exact.

```text
                          base (00348ada)   tip (3801f01f)
I refs, --decks fixed       1,243,883,799    1,236,397,437   -0.602 %
I refs, --decks cube        3,199,250,814    3,182,035,509   -0.538 %
I refs, --decks sos         1,735,661,867    1,728,941,302   -0.387 %
I refs, --decks sealed      3,483,347,170    3,459,192,362   -0.693 %
deck build alone               34,622,104       26,612,999  -23.13 %
  (--decks sealed --games 1: 0 games played, all setup)
```

**Re-read after the second rebase, and the two passes compose to the third
decimal.** Pass 55 landed its (I)-(K) under this chain mid-flight; measured
again on top of them (branch tip `9582d1ea`, and pass 55's (K) `c676cf48` as
the base column):

```text
                       pass 55 (K)      branch tip (9582d1ea)
I refs, --decks fixed   1,234,031,722    1,226,172,210   -0.637 %
I refs, --decks cube    3,179,782,586    3,162,426,697   -0.546 %
I refs, --decks sos     1,722,423,954    1,715,663,088   -0.392 %
I refs, --decks sealed         n/a       3,430,701,306
deck build alone               n/a          26,570,012
```

**-0.637 / -0.546 / -0.392 against -0.602 / -0.538 / -0.387 on the pass's
own base** — the same rows, so nothing this pass removes was already removed
by (I)-(K), and nothing it removes makes theirs smaller. The base column
there is pass 55's own reading on its own box; the offset between the two
containers is the argv string (this pass measured `00348ada` at +483 /
+1,319 / +707 Ir over pass 55's numbers for the same commit), which is three
orders below the deltas.

Per commit. **Measured on the pass's own chain against `00348ada`**, before
the second rebase.

| step | commit | pool it moves | delta |
|---|---|---|---|
| B | `1c223827` | cube / fixed / sos | -0.090 % / -0.018 % / -0.051 % — the requirement walker's stack leg was the one eager leg in a lazy chain |
| C | `02caa399` | fixed / cube / sos | **-0.291 %** / -0.188 % / -0.153 % — auto-tap's source table carries its battlefield index, candidate (-38)'s `actions.rs:12626` |
| D | `9c9afc74` | deck build | **-8.03 %** — `SosPacks`: six packs from one pool re-derived the pool's sheet and buckets six times |
| E | `1ca90507` | deck build | -3.04 % — `candidate_label` allocated a `String` per colour, a `Vec` and a join buffer, per candidate |
| F | `a8ced063` | deck build | -3.49 % — the builder looked the same `CardBrief` up three times per card per shape |
| G | `3b7d2c0b` | deck build | -3.94 % — every shape re-summed `colors_of_picks(pool)` |
| H | `708171c3` | deck build | **-7.35 %** — `PoolScores`: a card's base score is the same for all ~57 shapes, which follows from G |
| J | `4871ffb7` | fixed / cube / sos | **-0.280 %** / -0.214 % / -0.150 % — eleven sites cloned the computed keyword list to read it |
| — | dropped | — | **this pass's own (A) was the same optimization as pass 55's (B)** and was dropped on the first rebase. See the Log |
| — | reverted | cube +0.123 % | presence gates for `has_atype` / `has_stype`, candidate (-37)'s residue. See the Log |

```text
decisions          196,220 -> 196,220      byte-identical
turns_per_game     27.53   -> 27.53
stalls             0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism        ok (all pairs split, both)
thread determinism ok (3 vs 1 threads identical)
ladder output      --decks sealed --games 6's full printout diffs identically
                   base vs tip — the check that covers the decks a seed
                   *builds*, which five of the eight commits touch
suite              18,728 passed / 0 failed / 5 ignored over 22 binaries
golden traces      all unchanged
clippy             `--workspace --all-targets` clean
rustc              1.95.0 (59807616e 2026-04-14)
host_cpu           Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores
```

**Wall clock, `release-fast` + mimalloc, both binaries alternated in one
sitting on this box, and the deck build is the number this pass claims.**
100 invocations of `--decks sealed --games 1` (which plays no games), minus
the process-startup floor measured the same way with `--decks fixed`, four
alternated pairs:

```text
                base      tip
pair 1        0.6862    0.5292   -22.9 %
pair 2        0.5978    0.5911    -1.1 %
pair 3        0.6468    0.5578   -13.8 %
pair 4        0.6186    0.5128   -17.1 %       4/4 pairs positive
best-of       0.5978    0.5128   -14.2 %
floor (100 procs)  0.3093    0.3294
```

**-14 % against -23 % Ir, and two things account for the gap.** The
pass-54 caveat first: this work is allocation-shaped, callgrind runs the
*system* allocator and mimalloc ships. And the tip's own startup floor is
**6.5 % higher** (0.3294 s against 0.3093 s per 100 processes) — a bigger
binary to relocate, which is subtracted from both columns but is real time
the change added. A training actor never pays it: it builds its decks inside
one long-lived process.

The game loop, same binaries and sitting, `--decks cube` 600 games / 1
thread / seed 1, alternated: base **62.73 / 62.43** against tip **62.06 /
63.03**, best-of **-0.6 %**, which is the -0.538 % Ir to within the drift.

**`--bench` cannot see this pass's game-loop rows, and that is the expected
result rather than a null one.** Paired `release-fast`, 3 threads,
A/B/A/B/A/B: base 201.33 / 200.61 / 200.41 against tip 203.59 / 197.39 /
194.09 — **1/3 pairs positive, mean -1.2 %** against -0.602 % Ir on that
exact pool. A 0.6 % change is a quarter of this bench's spread; the Ir
column is the attribution and the cube pair above is the wall-clock claim.

**The committed `release` anchor is not refreshed, and the reading this
container gives is why.** `release` + mimalloc, 3 threads, three readings at
the pass's tip: **208.03 / 219.77 / 212.18**, `decisions` 196,220 on all
three, `turns_per_game` 27.53, `stalls_by` 0/0/0, `peak_rss_mib` 28.1-30.1,
`host_calib_ms` **45 / 47 / 46** — against pass 55's 270.23 / 276.73 /
269.17 at calib 55/53/55 on its box. This box's single-core probe is
*faster* and its three-thread throughput is a fifth lower, which is the
disagreement this file has now written up four times (the `bdc11c86`
re-anchor, the "two sittings, one binary-identical workload, 3.7 % apart"
pair, the note that the calib probe is single-threaded, and this). Nothing
intentional changed the workload, so the anchor stands and a
cross-container absolute is not evidence of anything.

**Crash-freedom and determinism at the tip.** `release-fast`, `--a gang
--b gang --games 200 --threads 3`, seeds 11/12/13 x `--decks all` plus
`--decks sealed` at seed 11: every cell **decided, 0 undecided, no panic,
all pairs split** — 12,600 games and 6,300 pairs. The `overflow` profile
(`release-fast` + `overflow-checks`) over seeds 11/12/13 x `--decks all`
reads the same counts with **no panic and no arithmetic overflow** — 10,200
games. `CRAB_THREAD_CHECK=1 --bench` reads **`thread_determinism ok (3 vs 1
threads identical)`**.

**No net needs retraining, and five of the eight commits are in the deck
builder, so that claim is checked rather than asserted.** No encoding, pool,
`TrainRow`, `EncodedState` or `Vocab` change is in this pass, and
`--decks sealed --games 6`'s full ladder printout — which covers the decks
each seed *builds*, not just the games they play — is byte-identical
between the base and tip binaries.

**Fifty-fifth pass, base `bf4917a5` (pass 54's tip) vs its own tip
`c676cf48`.** Eleven commits, one class after the first: **the simulator kept
building answers before asking whether anyone wanted them.** (A) the
requirement walker's subtype arms stop gathering where the printed line
answers; (B) the freeze scope's depth and gate slots come out of the mutex;
(C)-(K) nine helpers that allocated, cloned or re-found something their
caller discards or already holds. **`--decks cube` -20.75 %,
`--decks sos` -2.16 %, `--decks fixed` -1.15 %.** Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--seed 1` unless the row says otherwise.

```text
                          base (bf4917a5)   (A) 8779aa9f     tip (K)
I refs, --decks cube        4,012,095,058   3,332,029,985   3,179,782,586  -20.75 %
I refs, --decks fixed       1,248,407,927   1,249,622,086   1,234,031,722   -1.151 %
I refs, --decks sos         1,760,442,504   1,761,529,321   1,722,423,954   -2.160 %
I refs, --decks sealed      3,497,162,303   3,500,013,528     (B), below
deck build alone               34,506,869      34,859,382     (B), below
  (--decks sealed --games 1: 0 games played, all setup)
```

Per commit, the three pools each was measured on:

| step | cube | fixed | sos | what |
|---|---|---|---|---|
| A `8779aa9f` | **-16.95 %** | +0.097 % | +0.062 % | the requirement walker's subtype arms ask a presence gate before forcing the layer view |
| B `4c58c9c7` | -0.709 % | -0.276 % | -0.365 % | the freeze scope's depth and gate slots come out of the mutex |
| C `24860169` | -0.665 % | +0.001 % | -0.338 % | `affected_from_requirement`'s And-tree stack is an inline array, not a heap `Vec` |
| D `67fc39ab` | -0.114 % | -0.121 % | -0.064 % | `restore_payment_state` asks with a shared borrow before unsharing the battlefield |
| E `5f988142` | -1.021 % | flat | -0.442 % | `extract_power_gate` asks `requirement_mentions_power` before cloning the tree |
| F `9d9555e9` | -1.208 % | -0.034 % | -0.018 % | the per-card grant walk hands the permanent to the filters instead of re-finding it |
| G `863d882d` | -0.205 % | -0.028 % | -0.254 % | `granted_abilities_of` does the same for the mana sweep's grant scan |
| H `353273ef` | -0.126 % | flat | flat | thirteen more battlefield walks hand their card to the requirement walker |
| I `7ec4836c` | -0.256 % | **-0.356 %** | -0.291 % | the gather's two always-empty `collect()`s become `Vec::new()` |
| J `ca16b33a` | -0.291 % | -0.369 % | **-0.422 %** | the SBA sweep's game-over check is a walk, not two `Vec`s and a sort |
| K `c676cf48` | -0.062 % | -0.069 % | -0.051 % | combat's three per-attacker collects are gated on a presence scan |
| — | +0.40 % | +0.66 % | — | **REVERTED** — the presence gate on `board_keyword_matching`'s *frozen* leg. See the Log |
| — | +0.43 % | +0.12 % | — | **REVERTED** — a two-phase exactly-sized build in `statics_granted_triggers_with`. See the Log |

`sealed` and the deck build were read at (B) and not re-read; (C) through (K)
are engine paths the deck builder does not reach.

**The deck-build row was layout, and the profile says so rather than the
usual hand-wave.** At (A) it read +1.02 %, and
`creature_type_change_in_scope` / `land_type_change_in_scope` do not appear
in that dump at all — the deck builder never reaches the requirement walker.
The whole +352,513 sat in `LocalKey::with` (207 attributed calls before,
4,850 after), i.e. the `card_def` front cache inlining differently under a
bigger binary; (B) moved it back to +0.293 % without touching that code,
which is the confirmation. Check the callee before blaming a row like this;
here it cost one `cg_edges.py` run to rule out.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism      ok (all pairs split, both)
suite            18,827 passed / 0 failed / 5 ignored over 31 binaries
                 (the whole workspace, at the final tip)
golden traces    all 7 unchanged
clippy           `--workspace --all-targets` clean
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores
```

**Wall clock over the whole pass, and it is a quarter of the Ir.**
`release-fast` + mimalloc, 600 games / 1 thread / seed 1, alternated
base/tip in one sitting. The base here is **the pass's own engine diff
reverted at the current catalog** — the concurrent session's card commits
land between `bf4917a5` and this tip, and this isolates the engine work
from them.

```text
--decks cube    base 54.06 / 55.32 / 55.10   best 54.06
   600 games    tip  50.70 / 52.20 / 51.75   best 50.70   -6.2 %
--decks sos     base 31.90 / 31.80 / 31.73   best 31.73
   600 games    tip  30.85 / 30.95 / 30.67   best 30.67   -3.3 %
--decks fixed   base 59.44 / 59.79           tip 59.31 / 59.47   -0.2 %
   1200 games
```

The tip is faster in every one of the eight pairs. **`sos` -3.3 % is the
number the training loop gets** — that is the pool a `selfplay_train` actor
plays.

**The training loop itself was measured and is INCONCLUSIVE, which is worth
recording rather than rounding into a win.** `selfplay_train --actors 3
--games 6000 --steps 1 --seed 7`, both binaries `release-fast` + mimalloc,
alternated in one sitting:

```text
base   144.4 / 176.1 / 178.2 games/s      best 178.2
tip    185.1 / 177.7 / 179.5              best 185.1     (+3.9 %)
wall   base 34.2 / 34.0 / 33.0 s          tip 33.5 / 33.1 / 33.2 s   (flat)
```

The reported rate and the wall clock disagree in sign, and the base's spread
on the identical binary is **23 %** (144.4 to 178.2) against the tip's 4 %.
Three actors on a four-core box that is also running the harness is not an
instrument at this resolution. **Use `--decks sos` on the ladder as the
proxy** — same pool, one thread, 3/3 pairs, -3.3 % — and re-run the training
loop on the box that matters.

**-6.2 % wall clock against -20.7 % Ir, and the gap is the point.** A sixth
of the Ir this pass removed is the allocator family, callgrind runs the
*system* allocator, and mimalloc ships — so the Ir is the attribution and
the wall clock is what the training host gets. **An earlier sitting read
(A) alone at -6.8 %** on a different catalog; the two are not comparable and
neither is wrong, which is this file's standing warning about wall-clock
absolutes restated. Quote both numbers or neither.

The gap is the pass-54 caveat again: a sixth of the Ir saved is the
allocator family, callgrind runs the *system* allocator, and mimalloc ships.
Quote both numbers or neither.

**`--bench`, the committed throughput configuration** (`release` + mimalloc,
3 threads), three readings at the tip:

```text
games_per_s      277.25 / 281.06 / 274.34   best 281.06  (pass 54 tip: 269.41)
decisions        196,220 on all three
turns_per_game   27.53
stalls_by        cap 0 / stuck 0 / draw 0 on all three
peak_rss_mib     28.2 / 28.4 / 30.1         (pass 54 tip: 30.3)
host_calib_ms    52 / 65 / 55
```

**Read it as up, but not by 4 %.** `--bench` is `--decks fixed`, which moved
-1.08 % in Ir over the pass and -0.2 % on the release-fast wall clock above;
the 2.5 % spread across three back-to-back runs of *one* binary is this
file's standing warning about `--bench` absolutes, and the rest of the gap
to pass 54's 269.41 is a different sitting on a different host. No base
binary was built at `release` here — the Ir column is the attribution and
the release-fast pairs above are the wall-clock claim.

**Crash-freedom and determinism at the tip.** `release`, `--a gang --b gang
--games 200 --threads 3`, seeds 11/12/13 x `--decks all` plus `--decks
sealed` at seed 11: every cell **decided, 0 undecided, no panic, all pairs
split** — 12,600 games and 6,300 pairs, re-run unchanged at the final tip. `CRAB_THREAD_CHECK=1 --bench` reads
**`thread_determinism ok (3 vs 1 threads identical)`**.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass.

### The fifty-fourth pass's baseline

**Base `4369a0d6` (pass 53's tip) vs its own tip
`e1cbc390`.** Nine commits in two classes: **deck construction stops
re-deriving what a memoized definition already answers** (seven), and **two
gathers that nobody read** (two, found with a measuring device this pass
added — see `scripts/cg_contexts.py`). Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--seed 1` unless the row says otherwise.

```text
                          base (4369a0d6)   tip (e1cbc390)
I refs, --decks fixed       1,250,405,745   1,248,408,061   -0.160 %
I refs, --decks cube        4,026,141,796*  4,012,096,941   -0.349 %
I refs, --decks sos         1,760,202,906*  1,760,445,728   +0.014 %
I refs, --decks sealed      3,572,196,844*  3,497,168,270   -2.101 %
deck build alone              111,755,559      34,509,612  -69.12 %
  (--decks sealed --games 1: 0 games played, all setup)
```

**Re-read after a rebase onto a concurrent session's three vocabulary-loader
commits** (`796427ab`, `e8f5dbad`, `b36ba8f2`, which sit between this pass's
`457b3864` and its two engine commits). `fixed` moved 1,248,410,451 ->
1,248,408,061 and the deck build 34,511,759 -> 34,509,612 — 2,390 and 2,147
Ir, i.e. nothing, which is what a change confined to the net loaders should
do. The `cube` / `sos` / `sealed` rows above were read on the pre-rebase tip
and are not re-read; the `fixed` pair is what says they did not move.

\* the base `cube` / `sos` / `sealed` figures are the fifty-third pass's own
tip readings carried forward. This pass's base *is* that tip, and its `fixed`
re-read 1,250,405,745 against the recorded 1,250,409,741 — 3,996 Ir of argv,
the offset every pass sees.

**The deck-builder commits move the three non-sealed pools by code layout
alone** — `fixed`, `cube` and `sos` build their decks from hand lists or the
cube recipe and reach none of that code — and at the seventh commit that
drift had accumulated to +0.13 % on `fixed`. The last two commits are engine
work and take it back past zero. Read the per-commit table for which is
which; no single deck-builder commit's own `fixed` delta was over 0.08 %.

**The wall-clock number, and it is the one that matters for training.** The
deck-builder work is allocation-shaped, so callgrind's system allocator
overstates it; measured on the shipped `release` build with mimalloc, 30
invocations of `--decks sealed --games 1` (which plays no games), minus the
0.113 s process-startup floor measured the same way with `--decks fixed`:

```text
                base binary   tip binary
30 x deck build   0.458 s       0.163 s     2.81x
```

**And the number that decides whether any of it matters: the training loop.**
`selfplay_train --actors 3 --games 3000 --steps 1 --seed 7`, both binaries
`release-fast` + mimalloc, alternated A/B/A/B/A/B in one sitting, plus an
earlier `--games 900` pair the same way:

```text
base   117.3 / 156.5 / 136.7 / 129.1 / 156.5 games/s      best 156.5
tip    169.8 / 169.6 / 165.6 / 152.5 / 167.2 games/s      best 169.8   +8.5 %
rows/s 15,163 -> 16,498 at the two best readings
```

**And the judged builder, which is where the deck work compounds.**
`--use-deck-best` runs `best_build_by(pool, 32, ..)` — thirty-two
`build_random_deck`s per side per game — so every commit above pays
thirty-two times over on that path. It was untestable at the pass's start
(no committed deck net loads; see TODO's ML section), so a throwaway one was
trained at the current vocabulary to run it, which also **verifies the
vocabulary freeze end to end**: a net trained after the freeze loads, pads
and drives the actors.

```text
--actors 3 --steps 1 --seed 7, release-fast + mimalloc, alternated,
best of four (two pairs at --games 600, two at --games 1200)
  judged (--use-deck-best)   132.9 / 146.2 / 148.9 / 135.3   best 148.9
  unjudged, same sitting     152.6 / 155.9 / 158.3 / 162.3   best 162.3
  judged / unjudged                                          91.7 %
```

**The judged path is within 8 % of the unjudged one**, where the fifty-third
pass left it at 83.4 % (83.2 against 99.8 on a different box — the *ratio* is
what carries across hosts, not the absolutes). 0 stalls in ~7,600 judged
actor games.

**+8.5 % on best-of-five, and it is well under the ~19 % the Ir predicted.**
That gap is this file's own caveat and it is worth stating plainly: the
builder's cost is allocation-shaped, callgrind runs the *system* allocator,
and mimalloc — which is what ships — had already absorbed a good part of what
the Ir attributes to the change. Note also the spread: base reads 129.1 to
156.5 on the identical binary minutes apart while the tip reads 152.5 to
169.8, so take the best of each pair rather than the mean, and do not quote a
single run of this workload.

**`--bench`, the committed throughput configuration** (`release` + mimalloc,
3 threads), both binaries alternated A/B/A/B in one sitting, best of two:

```text
                 base            deck-builder tip   final tip
games_per_s      264.71          264.79             269.41
decisions        196,220         196,220            196,220   byte-identical
turns_per_game   27.53           27.53              27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0 (all three)
determinism      ok (all pairs split); CRAB_THREAD_CHECK ok (3 vs 1)
peak_rss_mib     31.3            30.2               30.3
ladder output    all four pools' full printout (20 games, seed 1) diffs
                 identically base vs tip — the strongest behaviour check
                 here, because it covers the decks a seed builds and not
                 just the games they play
suite            18,815 passed / 0 failed / 5 ignored over 31 binaries
golden traces    all unchanged
clippy           `--workspace --all-targets` clean (client included)
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores
host_calib_ms    53-70 across the readings
```

**Read the `--bench` column as flat and nothing else.** `--bench` is
`--decks fixed`, which builds no deck, so the seven deck-builder commits
cannot move it and did not (264.71 -> 264.79). The final +1.8 % is the two
engine commits plus host drift, and this file's own note two sections down
says a `--bench` absolute has read 3.7 % apart on one binary in one
container ninety minutes apart. The Ir column is the attribution.

**Crash-freedom and determinism at the tip.** `release`, `--a gang --b gang
--games 200 --threads 3`, seeds 11/12/13 x `--decks all` and `--decks sealed`
(the deck builder's own pool, which `--decks all` does not include): every
cell **decided, 0 undecided, no panic, all pairs split** — 17,400 games and
8,700 pairs. Re-run unchanged after the run's later card-defect commits **and after the
rebase onto the fifty-fifth pass**, at `1badee12`: same grid clean (11,600
games), `thread_determinism ok (3 vs 1)`, `--bench` decisions 196,220 / turns
27.53 / stalls 0 / **269.72 games/s best of four**, suite 18,827 passed /
0 failed / 5 ignored, clippy `--workspace --all-targets` clean. All four
pools' 20-game printout diffs identically against the pass base at the
pre-rebase tip.

**A caution the same sitting supplied.** The first two `--bench` runs after
clippy read 225.24 and 231.00 with the host still settling, against 252-270
across four runs minutes later on the same binary — a **16 % spread**, four
times the 3.7 % this file already records. Take the best of several, never
the first after a build.
**Read that last one narrowly**: eleven cards changed behaviour in this run
(a dropped "you may", a collapsed mode, three absent abilities), and a
20-game sample not separating them means the sample did not reach them, not
that nothing moved. Their per-card tests are the proof of those changes; the
ladder diff is only evidence that the *engine* did not move. `CRAB_THREAD_CHECK=1 --bench` reads **`thread_determinism ok
(3 vs 1 threads identical)`**. And the `[profile.overflow]` run that turns a
silent wrap into a panic, re-run here because the deck builder is the code
that moved: `--decks sealed` 2,400 games and `--decks all` 3,400 games at 3
threads, **0 panics**. The `selfplay_train` A/B above is a further ~19,500
actor games at the tip with no stall and no panic.

**The front cache's memory cost, since it is a new per-thread allocation.**
4,096 slots x (8-byte key + 8-byte pointer) = 64 KiB per thread, in `.tbss`
— zero image cost, and `peak_rss_mib` moved 31.3 -> 30.2 at three threads,
i.e. inside the noise.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass, and every pool's ladder output is
byte-identical — only the cost of building the decks moved.

Per commit, `--decks sealed --games 1` (the deck build in isolation), with
each commit's `--decks fixed` reading beside it:

| step | deck build, before -> after | fixed | what |
|---|---|---|---|
| A `3c154e8d` | 111,755,559 -> 104,842,406 (**-6.186 %**) | +2,194 | `colors_of_picks` returns `ColorCounts` (`[u32; 5]`) instead of a `HashMap<Color, u32>` |
| B `5489b9fa` | 104,842,406 -> 94,866,008 (**-9.516 %**) | +0.006 % | both shape rankers read the pip totals they already hold instead of five more walks of the spell list |
| C `b10fdebd` | 94,866,008 -> 70,513,288 (**-25.67 %**) | -0.005 % | a 4,096-slot direct-mapped front cache in front of `card_def`'s map probe |
| D `5ca71f05` | 70,513,288 -> 65,341,053 (**-7.335 %**) | +0.076 % | `pip_counts` walks a cost once instead of once per colour in it |
| E `9cc1175c` | 65,341,053 -> 43,588,088 (**-33.29 %**) | -0.014 % | `CardBrief`: the per-definition derived facts (pips, cmc, type flags, quality, the fixing walk) memoized with the definition |
| F `735e365d` | 43,588,088 -> 37,942,385 (**-12.95 %**) | +0.066 % | the builder's four piles are `with_capacity`, not grown a doubling at a time |
| G `ec138369` | 37,942,385 -> 34,861,499 (**-8.12 %**) | -0.0003 % | `land_produced_colors` is a `ColorSet` on the brief, not a `Vec` per land per shape |
| — | 34,861,499 -> 35,864,023 (**+2.88 %**) | — | **REVERTED** — iterating `ColorCounts` by zipping rather than indexing. See the Log |

And the two engine commits, measured on `--decks fixed` (the deck build is
not what they touch). Their base is `457b3864`, the vocab-freeze commit,
re-read at **1,252,225,395** — +0.016 % on `ec138369`, layout again:

| step | fixed, before -> after | what |
|---|---|---|
| — | 1,252,225,395 -> 1,252,445,508 (**+0.018 %**) | **REVERTED** — a freeze scope around `eval_material_inner`'s board walk. Zero gathers removed; see the Log |
| I `25438a8b` | 1,252,225,395 -> 1,250,520,577 (**-0.136 %**) | `do_phasing`'s presence gate asked from inside its own freeze scope, so it gathered the effect set it exists to avoid |
| J `e1cbc390` | 1,250,520,577 -> 1,248,410,451 (**-0.169 %**) | the gather's own buffer was a `Vec::clone` (`capacity == len`) and reallocated on its first static ability |

**Fifty-third and fifty-second passes, compacted.** The full blocks are in
git (`PERF.md` before the sixty-first pass) and the Log entries carry the
substance. The numbers worth keeping:

```text
pass 53  base d37f31d8 -> tip ae938ac3
  fixed  1,265,405,219 -> 1,250,409,741   -1.185 %
  cube   7,962,354,254 -> 4,026,141,796  -49.436 %
  sos    1,771,650,597 -> 1,760,202,906   -0.646 %
  — and the two largest wins were invisible on `--decks fixed`, which is
    why they survived fifty-two passes. That is where "which pool a change
    moves" comes from.
pass 52  base b906be3b -> tip 1,265,410,851
  fixed  1,314,290,577 -> 1,265,410,851   -3.716 %
  decisions 198,810 byte-identical, turns_per_game 27.94, stalls 0
  — the pickers that dry-run their picks hand the state out, so the driver
    adopts it instead of running the same action a second time.
```


**Fiftieth pass, base `e7b3b3d4` (pass 49's tip) vs its own tip**, both
`profiling-fast --no-default-features`, built and run in one sitting on one
box. Rebased onto the concurrent run of the same pass (`4107e017`,
`49fce1ff`) after measuring; those two commits touch only `scripts/*.py` and
the trackers, so both columns stand unrederived.

```text
                     base (e7b3b3d4)          tip
I refs (callgrind)   1,531,246,782            1,314,288,098   -14.168 %
decisions            196,220                  196,220         byte-identical
turns_per_game       27.53                    27.53
stalls               0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism          ok (all pairs split, both)
allocations          not re-read              755,521
peak_rss_mib         21.6                     22.5
suite                18,712 passed / 0 failed / 5 ignored over 22 binaries
golden traces        all 5 unchanged
clippy               `--workspace --all-targets` clean
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz
host_calib_ms        47-49 across every reading
```

**The base column is 11 Ir under pass 49's recorded 1,531,246,793 for the same
commit** — argv length again (this run's `--callgrind-out-file` name is a
character shorter; pass 49 saw 492 Ir of the same effect and pass 47 686). The
base was read *directly* here, so the delta is exact.

**This box is not pass 49's.** `host_cpu` reads 2.10 GHz against that pass's
2.80 GHz and `host_calib_ms` 47-49 against 52-57, so **no `games_per_s` from
this pass compares to one from that pass** — see the note under **How to
measure**. Ir transfers between containers to within the argv string, which is
why the Log rows are quoted in Ir and nothing else.

**Crash-freedom and determinism at the tip, widest pool — and the recipe
changed after the fiftieth pass, because the old one could not see a
determinism bug that had been on the branch the whole time.** It was three
seeds at one thread count; it is now **thirteen seeds across `--threads
1/2/3`**, and the number to read is the **sweep count**, not just the panic
count: in a `gang`-vs-`gang` mirror the two games of a pair are one game with
the seats relabelled, so a single sweep is a bug. `CRAB_PAIR_SWEEPS=1` names
the offending pair and prints the seed that replays it.

`--a gang --b gang --games 400 --decks all`, seeds 11-23: **88,400 games,
88,382 decided, no panic**, and **all 42,391 mirrored pairs split**. The
undecided are rules draws, the same ones passes 44-50 recorded. What the old
recipe missed: `restart_game` (CR 727) rebuilt the state with
`GameState::new`, whose `GameRng` is `from_entropy`, so a seeded game that
*restarted* stopped replaying — fixed in `c6898506`, written up as TODO's
twenty-first robustness filter.

**Passes 45-49's Ir blocks, compacted.** The full blocks are in git
(`git log -- PERF.md`); every one is `profiling-fast --no-default-features`,
callgrind, `--a gang --b gang --games 6 --seed 1`, and every one reads
`decisions` **196,220**, `turns_per_game` **27.53**, `stalls` 0
(`cap 0 / stuck 0 / draw 0`) and `determinism ok` — that invariant set is the
only column that chains across them.

| pass | base -> tip | I refs | delta |
|---|---|---|---|
| 45 | `8a384e5c` -> `fec179f0` | 1,810,336,693 -> 1,765,005,375 | -2.504 % |
| 46, own chain | `11792f4c` -> `61fb3007` | 1,771,223,960 -> 1,747,982,407 | -1.312 % |
| 46, rebased and with its (E) | `fec179f0` -> tip | 1,765,005,375 -> 1,735,997,491 | -1.643 % |
| 47, own chain | `c9606062` -> `3706f96f` | 1,727,336,594 -> 1,674,581,042 | -3.054 % |
| 47, rebased and with the `Keyword::eq` pair | `636902ca` -> `a98d39b0` | 1,715,304,981 -> 1,645,831,969 | -4.050 % |
| 48, own chain | `89f55a5c` -> `1b32e4fb` | 1,662,145,003 -> 1,643,104,718 | -1.146 % |
| 48, rebased (A-E, then with F) | `40fb5e31` -> branch tip | 1,645,831,968 -> 1,628,221,407 / 1,625,262,542 | -1.070 % / -1.250 % |
| 49, own chain | `40fb5e31` -> own tip | 1,645,831,476 -> 1,560,268,509 | -5.198 % |
| 49, rebased onto 48 | `04282f2e` -> final tip | 1,625,264,320 (derived) -> 1,531,246,793 | -5.785 % |

**The branch across passes 46 and 47: 1,765,005,375 -> 1,645,831,969,
-6.752 %**, and every adjacent pair **composes**. Pass 47's seven commits take
*more* off the branch after pass 46's `cast_cost_scan` landed underneath them
(-53,159,867) than they did before it (-52,755,552); pass 48's rows read
slightly *smaller* rebased (-1.070 % against -1.146 %) because pass 47's
`Keyword::eq` pair had already removed some of what its (B) and (E) reach; and
pass 49's read slightly *larger* (-5.359 % against -5.198 %) because pass 48's
(E) took the `mana_source_table` gathers out from under the same ticks.
Pass 49's intermediate rebased readings, for the Log rows that chain to them:
`bf658313` (derived 1,628,220,915) -> A+B 1,540,962,924 -> C 1,538,787,495,
with **908,931** allocations at the A+B tip. Allocation counts across the two:
967,377 -> 949,413 (48), 967,377 -> 926,895 (49).

**Two containers, one Ir apart — so an absolute *does* transfer, and the
forty-eighth pass first concluded the opposite and was wrong.** Pass 47's tip
`40fb5e31` read **1,645,831,968** on pass 48's box against the
**1,645,831,969** pass 47 recorded on its own. The thing that misled: pass
48's base `89f55a5c` reads 1,662,145,003, not the 1,674,581,042 pass 47's Log
records — because that Log number is pass 47's **pre-rebase** tip `3706f96f`,
and `89f55a5c` is the same seven commits *after* a concurrent session landed
pass 46's `cast_cost_scan` (-0.697 %) underneath them. The gap is that commit,
not the container. **The rule that survives is narrower and still worth
having: re-read your own base, because on a shared branch the commit you think
you are standing on may not be the one the last pass measured.**

**And argv length lands in the Ir total.** Pass 49's base columns are 492 Ir
below what passes 47 and 48 recorded for the same commits, because its
`--callgrind-out-file` name is a character shorter; pass 47 saw the same
effect at 686 Ir. `40fb5e31` was re-read directly there (1,645,831,476,
exactly 492 under pass 48's reading); `bf658313` and `04282f2e` were not, so
those bases are that 492 subtracted from pass 48's numbers. One argv
throughout a pass makes its deltas exact — the absolute transfers between
containers to within the argv string.

The three things those blocks were written to carry, which are why the
numbers can go:

- **None of passes 45-49 ran a `release` A/B.** Each row is callgrind plus
  the `--bench` invariant check, which is what this file asks for a sub-5 %
  change; the committed `release` block below is still the forty-fourth
  pass's.
- **A `profiling-fast` `games_per_s` settles nothing.** Pass 46's four
  unalternated readings ran 138.16 / 139.14 / 135.69 / 133.08 in commit
  order and then **143.36 at the rebased tip** — drift that tracks the box,
  against an Ir column falling monotonically. Pass 47 declined to quote a
  pair at all: `host_calib_ms` read 49 then 60 across the two runs it had,
  and pass 48's only `--bench` runs were taken between builds on a shared box.
- **Wide pool clean at every tip.** `--a gang --b gang --games 400
  --threads 3 --decks all`, seeds 11/12/13: 20,400 games, 20,396 decided, no
  panic, all 10,198 mirrored pairs split (`rho -1.000` every seed), and
  byte-identical across pass 48's two runs. The 4 undecided are seed 11's
  standing rules draws, the same four passes 44-48 recorded.
  `peak_rss_mib` 21.0-21.9 across both passes; `host_cpu` Intel Xeon @
  2.80GHz, `host_calib_ms` 52-57; suite 18,709 passed / 0 failed / 5 ignored
  over 22 binaries, clippy `--workspace --all-targets` clean.


**The forty-fourth pass is a paired `release` A/B over six alternated pairs.**
Both sides built `release` + mimalloc from one tree, run alternating in one
sitting on one box. Base is `c0f4e3b6` (the pass's own base), tip is
`36592fd8`:

```text
                     base (c0f4e3b6)                 tip (36592fd8)
games_per_s          151.02 160.50 162.14             160.26 168.56 164.72
                     149.06 154.04 153.28             158.98 160.33 155.73
mean                 155.01 (spread 8.4 %)            161.43 (7.9 %)  -> +4.14 %
per pair             +6.12 +5.02 +1.59 +6.66 +4.08 +1.60   -> 6/6 positive
host_calib_ms        47-59 (both sides interleaved)
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz (both)
decisions            196,220                          196,220   byte-identical
turns_per_game       27.53                            27.53
stalls               0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism          ok (all 160 pairs split, every run)
peak_rss_mib         29.9 - 31.0                      29.1 - 31.1
```

**+4.14 % wall-clock against -5.310 % Ir, and the gap is the one this file
predicts.** Two of the four rows are allocation removals measured under
callgrind's *system* allocator, where an allocation's instructions are counted
and mimalloc's cheaper path is not; -5.310 % Ir would be +5.61 % if every
instruction were worth the same, and it reads +4.14 % with the allocator that
ships. Quote the paired A/B.

**Behaviour preservation, and it is stronger than the golden traces here.**
The base and tip binaries both ran `--a gang --b gang --games 200 --threads 3
--decks all --paired` and printed **byte-identical output over 3,400 games
across 17 archetypes** — every archetype's record, 1,700 paired splits,
`rho -1.000`, **0 undecided**. Two tip processes agree with each other too.
That check also covers the concurrent session's `ProtectionKind` refactor,
which is in the same range.

**Crash-freedom at the tip.** `overflow` profile (`release-fast` +
`overflow-checks`), `--a gang --b gang --games 400 --threads 3 --decks all`,
seeds 11 / 12 / 13: **20,400 games, 20,396 decided, no panic and no arithmetic
overflow**, 41-44 s a seed against an 8m30s build. The 4 undecided are all on
seed 11 — rules draws, and fewer than the 8 the thirty-second pass recorded
because rounds 43-47's search changes moved them, not this pass.

**The forty-second pass is measured as a paired `release` A/B, which is what
this file has been asking for and had never been able to do.** Both sides
built `release` + mimalloc from the same tree, run alternating in one sitting
on one box, so host drift moves both. Base is `a81df3fe` (rounds 48-49, no
perf commits); tip is `b1a95b22` (the pass's seventh commit — the eighth,
`1032979c`, is a further -0.496 % Ir this reading does not include):

```text
                     base (a81df3fe)              tip (b1a95b22)
games_per_s          153.80 / 154.33 / 157.09     160.24 / 164.71 / 166.11
mean                 155.07                       163.69      -> +5.56 %
host_calib_ms        46 / 46 / 44                 46 / 45 / 45
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz (both)
decisions            196,220                      196,220     byte-identical
turns_per_game       27.53                        27.53
stalls               0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism          ok (all pairs split, every run)
peak_rss_mib         27.5 - 29.8 (tip)
```

**+5.56 % wall-clock against -5.48 % Ir, and the agreement is the point.**
The same A/B on `profiling-fast --no-default-features` (system allocator)
reads **132.84 -> 140.64, +5.87 %**. Three independent measurements of one
pass landing within half a point of each other is what a *non*-allocation-
shaped pass looks like: the fortieth's allocation rows read -11 % Ir and
+39 % wall-clock because Ir counts an allocation's instructions and not the
stalls behind them, and this file predicted the divergence. This pass's two
big rows are a **deep copy** and a **`memcpy`**, which Ir counts honestly.

**The bench invariants moved before this pass, not in it.** `decisions`
196,220 and `turns_per_game` 27.53 against the block below's 193,232 / 26.98:
the pass's own base binary reads the new values, so rounds 43-47's adopted
search changes (chump blocks, target arms, hostile targeting) own the drift.
**196,220 / 27.53 / 0 stalls is the invariant set to compare against from
here.**

**Checked at the true tip (`1032979c`), on the `profiling-fast` binary the
Ir rows are taken on** — the eighth commit is a measured -0.496 %, so this is
a confirmation, not a re-anchor:

```text
decisions            196,220        <- byte-identical with the base binary
turns_per_game       27.53
stalls               0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism          ok (all pairs split)
peak_rss_mib         20.5
```

An unpaired `release` reading at the tip, taken after the whole session's
builds had been thrashing the box: **163.48 / 158.72 / 160.92 (mean 161.04)**,
calib 45 / 46 / 52. It is *lower* than the paired A/B's tip mean of 163.69
even though the eighth commit removes another 0.5 % of instructions — which is
the file's standing warning in one line: **an unpaired absolute on this box
cannot resolve half a percent**, and the third run's calib of 52 says why.
Quote the paired A/B.

**And a third box, at the forty-third pass's tip, that neither the probe nor
the CPU string predicts.** Same `--bench`, `release` + mimalloc, three runs:
**121.31 / 122.83 / 120.71 (mean 121.62)**, `host_calib_ms` **48 / 51 / 53**,
`host_cpu` `Intel(R) Xeon(R) Processor @ 2.80GHz`, `peak_rss_mib` 29.6-30.0.
That is **-25 %** against the reading directly above, on a calib band that
overlaps it (48-53 vs 45-52) and a *higher* clock string than the paired
A/B's 2.10 GHz. **So `host_calib_ms` does not discriminate box class as
finely as this file has been assuming** — it caught the half-percent question
above, and it says nothing useful about a 25 % one. Do not read this reading
as a regression: the same tip measures **1,911,862,094 Ir** under callgrind
against the base's 1,918,781,907, base and tip in one sitting on one box.

**What is portable is the invariant set, and it is byte-identical on all
three boxes**: `decisions` **196,220**, `turns_per_game` **27.53**, `stalls`
0 (cap 0 / stuck 0 / draw 0), `determinism ok` (all 160 pairs split, every
run), `peak_rss_mib` 29.6-30.0 against the A/B's 27.5-29.8. **Check those
first; quote the paired A/B for anything else.**

Plus the wide pool at the tip — `--decks all --games 200 --paired`, **3,400
games over 17 decks, two processes, output byte-identical** (modulo the
wall-clock line), 1,699 pairs all split, **2 undecided (0.06 %)**, no panics.
That is inside the 0.12 % rules-draw band `TODO.md` records for `--decks
all`; `--decks fixed` still reads 0.

**On the absolute: 163.69 is not comparable to the 153.17 below and only
looks comparable to the 163.62 anchor by coincidence.** This box reads
`host_cpu` 2.10 GHz with `host_calib_ms` 44-46, where the 153.17 reading was
2.80 GHz at calib 44-46 and the 163.62 anchor was 2.10 GHz at calib 55-90 —
the probe and the CPU string disagree about which box class this is, and the
workload changed underneath both. **Quote the paired A/B, not the absolute.**

**The fortieth pass was the first `release` reading in five passes, and this
bench did resolve it.** Same `--bench`, `release` + mimalloc, three runs, on
a box with the same `host_cpu` string as the thirty-sixth and thirty-seventh
passes' readings below. Taken at the pass's **fourth** commit (`c185f313`);
the fifth is a further -0.978 % Ir that this reading does not include:

```text
games_per_s          147.22 / 153.84 / 158.44   (mean 153.17)
games_per_s_th       49.07 - 52.81
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        46 / 45 / 44               <- 48-51 at those two readings
decisions            193,232 byte-identical on all three
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         27.0 - 29.0
determinism          ok (all pairs split, on all 3 runs)
```

**110.40 (pass 36's tip), 110.34 (pass 37's tip), 153.17 (pass 40's fourth
commit) — +38.8 %.** Those first two readings established a **2.2 %** spread on this
box class, so a 38.8 % gap is signal, not noise; `host_calib_ms` is 44-46
against their 48-51, so a few points of it are a faster box. **What it covers
is passes 38, 39 and 40 together** — cumulatively **-11.1 % Ir** — and it
cannot be split among them: no `release` binary was built at the
thirty-eighth or thirty-ninth tip. **A -11 % Ir reading +39 % wall-clock is
the shape this file predicts for allocation-shaped work** (see the caveat in
**How to measure**): the fortieth pass alone removed **90,382 allocations**
and **35 % of `memcpy`**, and Ir counts an allocation's instructions, not the
stalls behind them.

**The 163.62 anchor is not replaced, because it is a different box** — the
readings below are 110-153 on 2.80 GHz hosts against its 2.10 GHz / calib
55-90, and nothing in this file has ever made those comparable. Compare a
future reading to **153.17 at calib 44-46** and check the probe first.

**NOT re-anchored at the thirty-ninth pass's tip, and the host said why.**
The three commits are behaviour-preserving and sum to **-2.255 % Ir** under
callgrind — inside the bench's noise band, and this sitting's box is not the
one the anchor was taken on: `host_cpu` reads **2.10 GHz** against the
recorded tip's 2.80 GHz and `host_calib_ms` **71** against 45. A `--bench`
absolute here would measure the box. What was checked at the tip is the
**invariants**, and they are byte-identical with the anchor — `decisions`
**193,232**, `turns_per_game` 26.98, stalls 0, determinism ok. The block is
in the thirty-ninth pass's **Log** entry. The anchor stands at 163.62.

**NOT re-anchored at the thirty-eighth pass's tip either, and this time no
`release` reading was taken at all.** The pass is one behaviour-preserving
commit measured at **-2.525 % Ir** under callgrind, which is inside the
bench's noise band, and the three readings below already establish that a
`--bench` absolute on this box cannot resolve it. What was checked at the tip
is the **invariants**, on the `profiling-fast --no-default-features` binary
the callgrind rows were taken on:

```text
decisions            193,232        <- byte-identical with the anchor
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
determinism          ok (all pairs split)
games_per_s          124.04         <- NOT comparable: profiling-fast, system
peak_rss_mib         21.0              allocator. Neither goes in this block.
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        45
```

Plus the wide pool at the tip — `--decks all --games 200`, **3,400 games over
17 decks, two processes, output byte-identical** (modulo the wall-clock line),
1,700 pairs all split, **0 undecided**, no panics. The anchor stands at
163.62.

**NOT re-anchored at the thirty-seventh pass's tip (`59c964dc`).** Same
`--bench`, `release` + mimalloc, three runs, on the same container and the
same `host_cpu` string as the thirty-sixth pass's reading below:

```text
games_per_s          109.79 / 109.96 / 111.27   (mean 110.34)
games_per_s_th       36.60 - 37.09
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        51 / 49 / 48
decisions            193,232 byte-identical on all three
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.3 - 31.2
determinism          ok (all pairs split, on all 3 runs)
```

**Three readings on this box now: 110.40 (pass 36's tip), 112.73
(`7af2b489`, three commits and -1.590 % Ir later) and 110.34 (`59c964dc`,
one more commit and -0.223 % later).** The spread is 2.2 % and the Ir moved
-1.809 % monotonically down across it, so the bench cannot resolve this
pass and does not try to. **What it does check is the invariants, and they
are identical**: `decisions` is 193,232 byte-identical with the anchor and
with every reading below, `turns_per_game` and the stall counts unchanged,
determinism ok on all six runs. Nothing to investigate; the anchor stands.

**The thirty-sixth pass's reading, kept because it is the file's third and
sharpest demonstration of why the anchor does not move.** Same `--bench`,
`release` + mimalloc, three runs at `898a9912`:

```text
games_per_s          111.58 / 108.32 / 111.29   (mean 110.40)
games_per_s_th       36.11 - 37.19
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz    <- 2.10GHz at the anchor
host_calib_ms        48 / 48 / 51                            <- 55-90 at the anchor
decisions            193,232 byte-identical on all three
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         31.0 - 31.2
determinism          ok (all pairs split, on all 3 runs)
```

The pass's *first* commit read **112.09 / 109.68 / 112.60 (mean 111.46)**
on the same box an hour earlier. The second commit is **-1.640 % in Ir** and
the two bench means are **0.95 % apart** — a textbook instance of the rule
above it: a sub-5 % change does not clear this bench's noise, whichever
direction it goes. Callgrind is the arbiter; the bench's job here is the
invariants, and they are identical across both readings.

**That is -32 % against the 163.62 anchor, on a change that callgrind
measures at -0.879 %.** It is the box, and this time the box says so out
loud: `host_cpu` reports a *different processor string* and `host_calib_ms`
reads **faster** (48-52 against 55-90) while three-thread throughput is a
third lower — the same single-threaded-probe blind spot recorded at the
anchor, now with the host difference visible in the CPU string too. Core
count is 4 on both.

**Every invariant is identical, and one of them is decisive**: `decisions`
is **193,232 byte-identical**, the anchor's exact figure, so the two
readings performed the same work. `games_per_s_th` fell uniformly by ~30 %,
which is a per-core-speed change, not something a code change that *removes*
20.6 M instructions can do. The callgrind base for this pass was built and
run on **this** container and reproduced the recorded `bdc11c86` figure to
within 3,123 Ir, so the -0.879 % is attributed on one box in one sitting.

Nothing to investigate; the anchor below stands. **Do not replace it with
111.46** — a baseline is only refreshed alongside an intentional, explained
change to what the program does, and this is neither.

**Re-anchored 2026-08-15 at `bdc11c86`** (`release`, mimalloc — the shipped
configuration), the thirty-fifth pass's tip.

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default)
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz
host_calib_ms        76 / 56 / 90 / 55
games                320
games_per_s          165.87 / 168.93 / 164.04 / 155.63
                     (mean 163.62, spread 8.5 %)
games_per_s_th       51.88 - 56.31
turns_per_game       26.98
decisions            193,232 byte-identical on all four
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.2 - 31.5
determinism          ok (all 160 pairs split, on all 4 runs)
```

**The same box read `169.85` (calib 53-66) at `8ff6daab` ninety minutes
earlier**, so this block is 3.7 % *under* the parent commit while the tip is
0.855 % *better* in Ir. Nothing to investigate: the drop is inside the
bench's noise band, calib moved the same way (55-90 against 53-66, and the
8.5 % spread against 5.1 % says the host was busier), and every invariant is
identical. Kept as two readings rather than one because the pair is the
cleanest demonstration in this file of why a `--bench` absolute is not an
attribution — **two sittings, one binary-identical workload, same
container, 3.7 % apart.**

**136.75 -> 163.62 is +20 %, and *none of it is the change*. Read this
before quoting either number.** The pass's change is **-2.177 % Ir**, and an
alternated `profiling-fast` A/B/A/B/A/B in one sitting read base 140.56 /
134.46 / 140.07 against new 139.55 / 138.93 / 136.06 — **-0.13 %,
distributions fully overlapping**, exactly what a 1-2 % change looks like
against this bench's ~5 % noise floor. The gap is the *box*.

**And that is a finding about the probe, not just about the box:
`host_calib_ms` did not detect it.** It read 53-66 against 47-57 at
the previous anchor — if anything *slower* — while three-thread throughput
is a quarter higher. The probe is a **single-threaded** ALU + 4 MiB
random-access loop, so it measures one core's speed and nothing about how
the host schedules three workers; two containers reporting the same
`host_cpu` string and the same calib can still differ by ~20 % on the
actual workload. The cross-check that *did* work is a configuration comparison:
`release-fast` + **system** allocator read ~138 games/s on this box, i.e.
*above* the committed `release` + **mimalloc** baseline of 136.75, which is
impossible on one host — release+mimalloc is strictly the faster
configuration. **Treat a `--bench` absolute as comparable only within one
sitting on one container**, calib agreement included, and use the
Log-row A/B or callgrind for anything else. A cheap improvement if this
comes up again: have the calib probe run one pass on `--threads N` as well
as single-threaded.

**The superseded anchors of passes 28-34, compacted.** Full `--bench` blocks
are in git (`git log -- PERF.md`). Every one is `release` + mimalloc, 320
games, and every one reads `decisions` **193,232**, `turns_per_game` **26.98**,
`stalls` **0** (`cap 0 / stuck 0 / draw 0`) and `determinism ok` — that
invariant set is the only column that chains across them.

| tip | pass | games/s (mean) | calib | RSS |
|---|---|---|---|---|
| `6cc0bdc3` | 34 | 136.75 (6 runs) | 47-57 | 29.1-30.8 |
| `35fdfce3` | 33 | 130.71 (6 runs) | 44-71 | 29.2-29.4 |
| `5174acd3` | 32 | 125.16 (4 runs) | 45-46 | 29.0-29.3 |
| `76804984` | 31 | 120.10 (4 runs) | 45-46 | 29.1-29.4 |
| `5034eb2f` | 30 | 115.40 (6 runs) | 44-48 | 29.0-29.4 |
| `ed4c152c` | 28 | 94.23 (3 runs) | 47-52 | 29.0-29.4 |

The three lessons those blocks were carrying, which are why the numbers can go:

- **Do not chain two blocks whose `host_calib_ms` bands don't overlap.**
  `130.71 -> 136.75` looks like +4.6 % and is a different container; the
  `bdc11c86` re-anchor to 163.62 is +20 % and *none of it is the change*.
- **A stall figure without its invocation is two figures.** `--decks all`
  reads **6 draws / 5,100 (0.12 %)** under `--bench --threads 1 --games 300
  --seed 11` and **2 undecided / 5,100 (0.039 %)** under `--a gang --b gang
  --games 300 --threads 3`. Quote the command.
- **Keep the pre-pass binary and diff its output, don't compare recorded
  constants.** At `645b978d` the base and tip both ran `--games 300 --threads
  3 --decks all` and printed byte-identical output over 5,100 games and 17
  archetypes. One `cp`, three minutes, and it answers what a recorded
  decision count only answers if the invocation matches.

**Crash-freedom re-run at `9b0cc470`, WIDER THAN THE STANDING RECIPE: 25,200
games, no panic and no arithmetic overflow.** `overflow` profile. The
standing grid (`--games 400 --threads 3 --decks all`, seeds 11/12/13) plus
two pools it does not reach:

| pool | invocation | games | decided | undecided |
|---|---|---|---|---|
| all | `--games 400 --threads 3`, seeds 11/12/13 | 20,400 | 20,396 | 4 (all seed 11) |
| cube | `--games 120 --threads 3`, seeds 11/12 | 1,920 | 1,920 | 0 |
| sealed | `--games 120 --threads 3`, seeds 11/12 | 2,880 | 2,880 | 0 |

**Why the two extra pools, and it is the point of the re-run.** `--decks
all` is 17 hand-built archetypes — a fixed, small card set. A panic in a
card those decks never draw is invisible to it however many games it plays,
and the catalog is 22,568 factories. `cube` and `sealed` build from
randomised pools, so they reach cards the standing grid cannot. Both came
back clean and neither has an undecided game. **Add them when a pass touches
rules code**; they cost seconds against the grid's minutes because the
`overflow` build is the expensive part and it is already paid.

**And the `--decks all` block is byte-identical to the run at `52e0b801`
below** — same 6796/4, 6800/0, 6800/0 — which makes it a 20,400-game
behaviour check on `655e1e47`, the layers commit between the two tips. A
reordering of `affected_includes_gated`'s predicates is exactly the change
that would show up as a different game outcome if the reorder were not
behaviour-preserving, and it does not.

**Crash-freedom at the sixty-second tip (`52e0b801`): clean, and identical to
the record.** `overflow` profile (`release-fast` + `overflow-checks`), `--a
gang --b gang --games 400 --threads 3 --decks all`, seeds 11 / 12 / 13:
**20,400 games, 20,396 decided, no panic and no arithmetic overflow**,
28.5-30.4 s a seed against a 10m11s build. The 4 undecided are all on seed
11, which is what every tip since the forty-second pass records.

**Run here because this tip is the first to carry both sessions' lines**,
and the other one's four commits reordered state-based actions on purpose —
`assign_sectors` and `sync_graveyard_shapeshifters` moved under
`sba_board_scan`, and the converge oracle changed which payment order two
cards take. Reordering SBAs is exactly the shape that turns into a stall or
a panic several thousand games out.

It did not, and the integration check is sharper than the grid:
`CRAB_THREAD_CHECK=1 --bench` on the same binary reads **decisions 196,220,
turns_per_game 27.53, 0 stalls (cap 0 / stuck 0 / draw 0), determinism ok
(all pairs split, rho -1.000), thread_determinism ok (3 vs 1 threads
identical)**, `host_calib_ms` 48 (in the 47-52 band). **196,220 is
byte-identical to the count at `b370d69e`, before any of the five commits.**
Five commits from two sessions, four of them behaviour-changing in the
rules, and the bench workload makes the same decisions in the same order.

Its `peak_rss_mib 27.3` is **not** comparable to the 17.7-18.3 MiB in the
Baseline block — that is an `overflow` build, and RSS is profile-dependent.

**Crash-freedom at the sixtieth tip: clean, and identical to the record.**
`overflow` profile (`release-fast` + `overflow-checks`), `--a gang --b gang
--games 400 --threads 3 --decks all`, seeds 11 / 12 / 13: **20,400 games,
20,396 decided, no panic and no arithmetic overflow**, 28-30 s a seed against
an 8m19s build. The 4 undecided are all on seed 11, which is what every tip
since the forty-second pass records. Worth running here because the pass
touched arithmetic on purpose (`ba15f249`'s `wrapping_*` mixer) and changed
three cube-pool cards' behaviour.

**Crash-freedom, the standing recipe and its record.** The `overflow` profile
(release-fast + `overflow-checks`) over `--a gang --b gang --games 400
--threads 3 --decks all`, seeds 11/12/13 (+ 3/29/41 at `5174acd3`): **no
panic and no arithmetic overflow** at every tip from the thirtieth pass on —
20,400 games each, 27,200 at `5174acd3` — with the 8 undecided always on seed
11 and always the same rules draws. ~50-90 s a seed against a 16-minute build.
Cheap enough to run every pass; do.

**The host fingerprint can disagree with itself, and the calibration probe
is the half to believe.** One container reported `host_cpu` *2.80 GHz* — the
`4f3e86c0` box — while `host_calib_ms` read **46-62**, overlapping the
2.10 GHz box's 47-52 and not the 2.80 box's own 53-70. That is why the probe
exists: compare `host_calib_ms` before comparing absolutes, and never chain
across two blocks whose probes don't overlap.

**Previous anchors, compacted to a table.** The full `--bench` blocks are in
git (`git log -- PERF.md`); every one of them was taken on the 2.80 GHz box
at `release` + mimalloc, 320 games, and **`turns_per_game` reads 26.98 and
`stalls` 0 in all of them**, which is the only column that chains.

| tip | pass | games/s (mean, spread) | calib | RSS |
|---|---|---|---|---|
| `4f3e86c0` | 19 | 67.31 (1.95 %) | 53-70 | 22.1-22.5 |
| `56986d65` | 18 | 71.29 (7.3 %) | 50-62 | 22.0-22.2 |
| `6ed3dbfc` | 17 | 69.13 (6.6 %) | 49-85 | 21.8-22.4 |
| `abb2b502` | 16 | 81.93 (9.2 %) | 45-55 | 21.7-22.2 |
| `28629ba9` | 15 | 64.19 (7.1 %) | 52-64 | 23.3-23.8 |
| `3e2ee6cb` | 14 | 62.48 (5.9 %) | 50-60 | 21.7-22.1 |

The lessons those six anchors were written to carry, which are the reason
the table above is safe to compress:

- **Consecutive anchors do not subtract.** 81.93 -> 69.13 -> 71.29 -> 67.31
  is four containers, not four regressions; the `calib` column moves with
  them and the instruction counts fall monotonically across the same span.
- **A -5.6 % anchor gap was checked, not asserted, and cost fifteen
  minutes.** At `4f3e86c0` the tip's two perf commits were reverted, both
  sides rebuilt `profiling-fast --no-default-features`, and the binaries
  alternated `--bench` in one sitting: base **56.55** against tip **57.58**,
  **+1.82 % paired, 4/6 pairs positive**. (A seventh pair was discarded —
  `host_calib_ms` read **525**, i.e. something else had the box.)
- **Keep the base binary and re-run it rather than reasoning about
  `host_calib_ms` alone.** At `3e2ee6cb` the base read 52.38 and, an hour
  later, 52.18 on the same box — ~0.4 % within-sitting drift, which is what
  made that pass's 6/6 paired **+11.10 %** trustworthy against an anchor gap
  pointing the other way. That pass agrees with its -11.56 % instruction
  count to half a point: the shape to expect when a change removes work *and*
  allocations.
- **An LTO confound is real and cheap to rule out.** With `codegen-units = 1`
  + thin LTO a 335-line addition elsewhere in the engine crate can move
  inlining crate-wide. Re-running callgrind on the *merged* tip at
  `6ed3dbfc` read +0.014 % against the pass's own tip, i.e. inside build
  noise.
- **70.65 and everything older belong to a different bench.** They predate
  `998b2433` making `EvalWeights::default()` carry `determinize: 1`, and
  `--bench` runs `gang` = `EvalWeights::default()`, so the *workload* changed
  underneath them.
- **Absolutes do not transfer between containers, and one pass proved it
  twice.** The same engine code read 60.49 and 64.42 in two sittings on one
  box (+6.5 % of pure drift), and a second container read 55.70 on its own
  tip where the first read 55.88 on a tip with 1.0 % *fewer* instructions.
  **Quote a paired A/B measured in one sitting, never a difference of
  anchors.**
