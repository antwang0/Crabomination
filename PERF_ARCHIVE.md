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


# PERF archive — Perf candidates, the closed entries from `(-192)` down

Moved **verbatim** out of `PERF.md`'s "Perf candidates" section at the
`(-241)` tip: every entry from `(-188)..(-192)` down to the oldest, ~10.5 k
lines of taken / refuted / closed candidates whose numbers are in the Log.
Still worth reading from here, not re-deriving: **`(-174)`** (its open
half, "the entry in the Log is the map"), **`(-90)`** and **`(-92)`** (the
two whole-profile maps — "the profile is flat"), and the census / refutation
entries the Standing rules cite by number. Nothing is closed or summarized
by the move.

**(-188)..(-192) TAKEN — the keyword presence gates, five legs priced by
one probe and taken one at a time; cumulative `fixed` -1.64 % / `cube`
-1.61 % / `sealed` -0.83 % against `fb120400` (Baseline, top).** The
`(-187)` entry below is closed by them: the row's cost was every leg, and
the way to see that was three `#[inline(never)]` fns and a ten-second
dump, not a device. What is left of the family on `fixed` at `(-192)`:
`board_keyword_in_scope` ~2.6 M self over 17,788 calls (the
`frozen_effects_memoized` lock at 32 Ir, the instance walk on the turns
`board_instance_keywords` is set), and `keyword_grant_in_scope` at
~100 Ir over 46,498 calls (three dynamic `synth` probes at 34, the
`continuous_effects` walk, the member-list read, two graveyard-anthem
lane reads). Nothing there is worth a build on its own.

**Re-read at `(-192)`, `cube`, ranked by call count — the rows a device
could still name:**
* `affected_includes_gated` 548,110 calls x 49 Ir = 26.9 M (1.16 %): the
  layer pass's per-(effect, permanent) selector match, already
  cheapest-first. A per-pass "which permanents does this effect reach"
  mask would be the device, and `compute_permanent_pass` (279,444 calls,
  293 Ir) is where it would live.
* `LocalKey::with` 261,084 x 67 = 17.5 M (0.75 %): 171 k from
  `computed_permanent_hinted`'s box alloc, 79 k from `Unfreeze::drop`.
  ⚠ **`(-170)(b)`'s batching is REFUTED as `(-193)`** (Log: `fixed`
  +1.288 %) — the 67 Ir is the pop and the `Arc::get_mut`, not the
  thread-local, and a batch moves every handle twice more. Closed.
* `has_keyword` 274,578 x 32 = 8.7 M plus `Keyword::eq` 346,155 x 13 =
  4.5 M: the tag split is in and the exact half is out of line; read the
  callers before pricing anything.
* `drop_in_place<ComputedPermanent>` 268,198 x 35 = 9.4 M, 167 k of them
  under the pool's own `LocalKey::with` — a recycled box is *overwritten*
  (`*slot = cp`), so the old view drops there; the cost is the view's
  `Vec` fields, which is `(-107)`'s by-value question again.
* `Unfreeze::drop` 213,134 x 34 = 7.2 M self: the pop, the gate clear
  and the lock, three in four scopes with nothing to park.

**`Vec::from_iter` RE-READ AT `72e7bf37` (2.87 % of `cube`, 2.72 % of
`fixed`) AND STILL DIFFUSE:** `cg_chain.py cg.nd 'from_iter' --top 8 --up
2` on `--demangle=no` dumps ranks its owners as `compute_permanents`
(20,932 calls on `cube`, 73 M inclusive — the layer pass itself, collected),
`fire_step_triggers` (24,216 / 2.0 M), two `Map::next` chains (3.0 M /
2.5 M), `pick_defensive_removal`, `can_afford_in_state_with`'s `OnceCell`
init, `pick_combat_trick`. No collect owns more than ~0.3 % of a pool on its
own; `(-92)`'s reading stands and this row is not a candidate.

**(-187) BUILT AND REVERTED, A LOSS (`fixed` +0.172 % / `cube` +0.110 % /
`sealed` +0.096 %):** a 63-bucket keyword tag mask per definition (a third
`CardMemo` word) in front of `board_keyword_matching`'s printed-list scan,
plus `is_empty` tests on the two instance lists. `board_keyword_in_scope`'s
self row *rose* (11.4 M -> 12.0 M on `fixed`), which says where its 640 Ir
a call are not: the printed lists are one or two keywords long, so the scan
the mask gates costs about what the mask read does. **The row's cost is the
gate the negative path pays** — `keyword_grant_in_scope` inlined into it,
the `continuous_effects` walk and the per-player command / emblem /
graveyard legs (`(-180)`'s residual, which this entry now sizes at most of
the row rather than ~1 M). A lane over those legs is the device; the
printed scan is not.

**(-186) BUILT AND REVERTED, FLAT (`cube` -0.000 % / `fixed` -0.001 % /
`sealed` -0.008 %):** the SBA sweep's `no_other` filter, its equipment
`attach_only_filter` legality and `do_untap`'s per-step untap caps on the
evaluator. All three are cold on every pool — no Synod Centurion, Konda's
Banner or untap-cap static in six games of any of them — so the walker's
remaining `check_state_based_actions_into` context (5,896 calls / 3.3 M
on `cube`, a `collect` closure) is not one of the three; `--demangle=no`
on the `--separate-callers=4` dump would name the closure. **A walker
context worth 0.14 % is below the bar on its own; the lane is mined out at
~145 k calls / 1.4 % of `cube`.**

**(-185) TAKEN — `fixed` -1.057 % / `cube` -0.133 % / `sealed` -0.104 %:**
the graveyard sweep on the evaluator's off-battlefield twin (Log). **What
is left of the walker on `cube` after it is ~167 k calls**: its own
recursion from callers still on the plain form, the gather's
condition-gated statics (27,132 / 5.7 M), and the SBA sweep's
`no_other` filter. The evaluator's own residuals stand: the card-type
closure out of line, ~35 Ir a node, `PrintedGrantFilter` and a per-batch
`PrintedGates` for the trigger-grant walk.

**(-184) TAKEN — `cube` -0.135 %, flat elsewhere:** the trigger-grant walk
on the evaluator, every entry point hinted (Log).

**(-183) TAKEN — `sealed` -0.698 % / `fixed` -0.682 % / `cube` -0.524 %:**
`printed_requirement` answers the targeting enumerator's and the selector
resolver's per-permanent asks before the walker (Log). **What is left of
the walker on `cube` after `(-184)` is 185,860 calls**, the largest
contexts now `first_legal_graveyard_card` (18,612 / 7.8 M — a
graveyard-card twin of the evaluator, and read the walker's `_` arm first:
it looks the card up through `died_card_snapshots` / `leaves_bf_lki`
*before* the graveyard, so the twin must read instance fields off the
same object the walker finds), the gather's condition-gated statics
(27,132 / 5.7 M), and the walker's own recursion (74,536 / 18.7 M — the
`Target::Permanent` block reached from callers still on the plain form:
`cg_contexts.py` with `--separate-callers=4` names them). Residuals of the
evaluator itself: the card-type closure is out of line (3.07 M on `cube`),
the recursion is ~34 Ir a node, `PrintedGrantFilter` could move onto it
(one build, measured against the `(-182)` row), and a per-batch
`PrintedGates` for the trigger-grant walk (~0.07 % of `cube`).

**(-182) TAKEN — `cube` -1.164 % / `fixed` +0.008 % / `sealed` +0.013 %:**
the grant filter is reduced to printed-line tests once per `grant_scan`
(Log). The walker's 105,436 calls under `granted_abilities_of_inner` are
gone; what is left of that function on `cube` is its own 16 M self (the
six-flag `static_abilities` pass, the `out` `Vec`) and `grant_scan`'s
12.8 M. **Next on the walker (~7.5 % of `cube` inclusive after this):**
re-take the `--separate-callers=4` context table for
`evaluate_requirement_static_hinted` — the largest remaining entries were
`fire_combat_damage_triggers` and the dispatcher's `event_matches_spec`
filters, and the `PrintedGrantFilter` device transfers to any caller that
evaluates one requirement against many permanents inside one `&self`
scope: resolve once, test per permanent, assert against the walker.
Residuals of `(-182)` itself: the `graveyard` grant leg; the
`HasCreatureType` leaf is unmeasured (no sliver board in seed 1's six
games).

**(-181) TAKEN — `cube` -1.166 % / `sealed` +0.011 % / `fixed` +0.025 %:**
`fire_step_triggers` filters the trigger grants to this step's kind before
the per-permanent requirement walk (Log).

**(-180) TAKEN — `fixed` -0.088 % / `cube` -0.072 % / `sealed` -0.063 %:**
`board_keyword_matching` asks the presence gate before a scope's first
gather. Marginal, and the Log says why: the gate is ~150 Ir an ask and is
paid on every scope-first call, so the saving is the ~1,500-2,800 gathers a
run that served an empty `compute_permanents`. **Residual:** a cheaper
negative for `keyword_grant_in_scope` — its `continuous_effects` walk and
the per-player command / emblem / graveyard legs are what the battlefield
grant lane does not cover — is worth the remaining ~1 M on `fixed`; a
candidate only bundled with something else in the same function.

**Re-read at `094b361a` (this pass, three pools, `--separate-callers=4` on
`fixed`): the self table is the `(-179)` one to within 0.1 points on every
row.** What the context table names that no entry has: `activate_ability_inner`
is 8,828 calls at **1,897 Ir of self** (1.95 % of `fixed`, 1.91 % of `cube`)
with every callee small — the cost is the gate prelude itself, spread over
~30 gates, so a line read (a `profiling-lines` build) is the next instrument,
not a device. `fire_combat_damage_triggers` is 7,258 calls at 1,749 Ir self
(1.48 % / 1.60 %), four battlefield walks and a per-kind graveyard walk that
sum to it; the graveyard leg would take a `Graveyard` lane ("any
`FromYourGraveyard` combat trigger") but is sized at ~0.25 % of `fixed`.

⚠ **The largest throughput result of the hundred-and-nineteenth pass is not in
this file's Log, because it is a bug fix.** `abilarms` — a shipped gate pilot —
took **846,610 ms → 304 ms** on `--decks cube --games 4` once two unbounded
activation loops were closed (~50,000 copies of one ability on one stack, each).
`--bench` is byte-identical either side, so no row here moves; **the point is
that a pilot's throughput is not covered by anything in this file**, and the
instrument that found it is `scripts/robustness_grid.sh --pilots`. Write-up:
**ENGINE_BACKLOG's first section**.

**The CR 704.3 sweep is a rules price, not a candidate** (Log, top): the
sweep after every cast/activation is gated on the payment's events and
costs +0.12-0.19 % as landed. The two things that could shave it are not
worth a build on their own: a cheaper `check_state_based_actions_into`
head for a board with nothing to sweep (the `sba_board_scan` is already
that), and skipping the sweep inside the bot's dry-run probes (they call
`perform_action` too — but a probe that misses a death mis-evaluates the
line, so that is a strength question first).

**Sized this run and left, so nobody re-prices them:**
* **A per-card hint for the graveyard sweep** (`(-174)`'s remaining device,
  signature-free: `first_legal_graveyard_card` stores `(id, seat, index)` in a
  `Relaxed` atomic on the state and `requirement_card_off_battlefield` reads
  it after the two LKI maps) saves only the *graveyard* leg — the exile leg
  runs for cards that are not in a graveyard at all — so its ceiling is
  ~1.5 M Ir, **~0.17 % of `fixed`**. Below the bar for a build; take it only
  bundled with something else in the same function.
* **`cast_lock_scan` in front of every cast/activate action** walks every
  permanent's statics, but it is ~4 k calls a six-game `fixed` run (casts,
  not activations — those come through `sim_step`), so a `zone::Battlefield`
  lane on "any cast-lock static present" is worth ~1 M Ir. Not a lead.
* **`check_state_based_actions_into` is 10,606 `collect()`s a run** through
  the slow `from_iter` path (15.6 M inclusive, but the closures are most of
  it). One of its ten `collect` sites runs once per sweep; find it with
  `--separate-callers=2` before pricing a scratch buffer, and expect
  ~0.15 %.
* **`fingerprint`** is 7.5 M on `fixed` after `(-179)`: one `Arc` deref per
  permanent (`damage`/`counters` live in `CardData`) and the call count,
  which the `(-176)` Log says an exact guard cannot reduce.

**(-179) TAKEN — `cube` -0.234 % / `sealed` -0.213 % / `fixed` -0.206 %:**
`(-176)`'s residual, one word per permanent with a tagged extra. See the Log.

**(-176) TAKEN — `cube` -0.686 % / `sealed` -0.675 % / `fixed` -0.563 %,
and the row it halved was NEW: `fingerprint` runs at every activation since
`01ac62e3` removed the `is_free()` gate (the CR 732.3 fix), 1.6-2.0 % of a
run.** What is left is 965 Ir a call, one whole-board walk per announcement,
and the Log says why an exact guard cannot skip it. The next device is a
cheaper walk, not a cheaper policy. ⚠ **A bug fix that widens a guard is a
perf change; read the top of the dump after one.**

**(-174) STILL OPEN, AND THE ENTRY IN THE LOG IS THE MAP: the prize is
~2.85 M Ir and the thing that killed the first attempt was the SIGNATURE, not
the device.** The census is the starting point — 106,488 hint hits / 7,558
misses that find the card / **31,116 misses for a card that is not on the
battlefield**, and every miss has `hint == None`. Those 31,116 are the
graveyard sweep, and the walker re-finds each card through
`requirement_card_off_battlefield` (0.37 % of the run;
`cg_sites` splits it 1.51 M graveyard leg / 0.98 M exile leg / 0.33 M the two
LKI maps).

⚠ **What is NOT available: widening `evaluate_requirement_static_hinted`'s
hint from 8 bytes to 16.** That was built and it cost **5.37 M Ir inside
`dispatch_triggers_for_events`**, which inlines the walker and is 6 % of the
program on its own — six times the saving, for a diff that does not touch it.
Any device here has to leave that signature alone. Two that might:

* the **exile leg** (0.98 M) runs only for calls that got past both
  graveyards, so a large share of the 31,116 are *not* graveyard cards at all
  — count them before assuming the sweep is the whole row;
* `died_card_snapshots.get` and the `leaves_bf_lki` pair are 0.33 M of hashing
  on maps that are empty outside a death trigger — an `is_empty()` in front of
  each is a signature-free change and costs one build to price.

**(-170) THE ALLOCATION CENSUS RE-TAKEN AT `9f9a5eac`, BY CALLER, AND IT NEEDS
NO `profiling-lines` BUILD.** `cg_edges.py --callers "__rustc::__rust_alloc"`
on the ordinary `profiling-fast` dump ranks every allocation by the function
that made it, and `--separate-callers=2` on the *same binary* (a 25-second
re-run, no rebuild) splits each row by its caller's caller. That is the
instrument `(-163)` spent a cold `profiling-lines` build to approximate, and it
answers the same question one level coarser for free. Use it first.

```text
  fixed — 362,942 engine allocations (was 500,822 at (-163)); allocator is
  8.3 % of the pool, was 10.2 %
    86,936  RawVecInner::finish_grow            ← 74,530 of them via grow_one
    63,076  Arc::clone_from_ref_in              ← 58,440 of them via make_mut
    53,224  SpecFromIterNested::from_iter
    26,488  Vec::clone                          ← 21,452 under clone_from_ref_in
    23,954  GameState::clone                    ← 10,130 under affordances
    21,740  CowBox<Vec<T>>::push
    10,616  PrintedList::push                   ✗ dead as filed, see below
```

**Three leads, in order.**

*(a) ✅ THE CHECK HALF IS TAKEN by `(-177)` — `fixed` -0.554 % / `sealed`
-0.517 % / `cube` -0.498 %: uniqueness is a `strong == 1` load inline once
the `Weak` case is ruled out by construction (no `downgrade` in the
workspace). ⚠ The clock read FLAT for ~330 k `lock cmpxchg` removed — see the
Log before pricing any other atomic by its cycle count. The copy half
(58,440 deep copies) is still what `(-99)`/`(-100)`/`(-143)` left it.*

*(a) The CoW unshare is the largest named family and it is 92 % of one
callee.* 58,440 of `fixed`'s 63,076 `clone_from_ref_in` allocations are
`Arc::make_mut` deep-copying a `CowBox` field, and `make_mut` itself is
**387,452 calls at 29 Ir** — so 85 % of the calls are the *check*, not the
copy. Top callers by call count: `cast_spell_with_convoke` 60,406 (17.9 M Ir
inclusive), `resolve_top_of_stack_inner` 43,400, `do_untap` 42,072,
`resolve_combat` 33,366, `finalize_cast` 18,050. ⚠ `(-99)`/`(-100)`/`(-143)`
worked the *copies*; what no entry has ranked is the **check** — 11.2 M Ir of
`make_mut` self on a path where the answer is "unique" 85 % of the time. And
⚠ `(-157)`: a `release-fast` reading over-states this family by ~20 % because
thin LTO deletes a fifth of its calls.

*(b) ✅ CLOSED by `(-175)` — `fixed` -0.422 % / `sealed` -0.389 % / `cube`
-0.300 %, and NOT by the device below, which was built and is a regression on
all three pools (+0.20 to +0.29 %). The caller table says 89,500 of the 99,796
calls are scope EXITS, three in four of them parking nothing; a `memo.is_none()`
guard in `end_of_scope` is the whole fix. The paragraph is kept for the
refutation; see the Log.*

*(b) `LocalKey::with` is 99,796 calls / 4.92 M Ir / **0.57 % of `fixed`**, and
it is `(-166)`/`(-168)`'s own overhead.* One TLS access per pooled box:
69,822 from `cp_pool::alloc`, 23,512 from `fx_pool::alloc_with`, the rest from
the scope exits. It is 49.3 Ir a call because the pooled types need `Drop`, so
`LocalKey::with` cannot take std's const-init fast path and is not inlined.
**The fix is batching, not a leak.** Give `LayerFreezeState` a `spare` list,
fill it from the thread-local on the scope's *first* `alloc` and return the
remainder at `end_of_scope`: two TLS accesses a scope instead of ~5, ~69,822
calls down to ~32,000. Worth ~0.2-0.3 % on `fixed`. ⚠ **Do not reach for a
non-`Drop` thread-local instead** — the only way there is `ManuallyDrop`, which
leaks a bounded amount *per thread ever created*, and `recommend.rs`'s game
workers are not obviously a fixed pool.

*(c) `SpecFromIterNested::from_iter` is 177,758 calls / 23.1 M self Ir /
129.9 a call = **2.65 % of `fixed`**, and 53,224 of them allocate.* It is
`collect()`'s slow path, taken whenever the iterator's lower size hint is not
exact — i.e. after a `filter`. ⚠ **`(-156)` prices a std-adapter rewrite at
~10 % of the adapter's self Ir**, so the removable part is ~2.3 M, not 23 M;
and the concurrent session already ruled the row out once at 92 callers. What
is *not* ruled out is the 53,224 allocations: a `filter().collect()` whose
result is immediately iterated and dropped wants a reused scratch buffer, and
`(-168)` is the pattern.

⚠ **`(-163)(b)`, `PrintedList::push`, IS DEAD AS FILED and should not be
re-pulled.** Its `__rust_dealloc` edge is 316 against 10,616 pushes, so
**99.3 % of its calls are a *first* push** — the `Vec`-instead-of-`Box<[T]>`
change amortises repeats that do not exist, and the +8 bytes on
`ComputedPermanent` would be paid for nothing.

**(-167) CLOSED — TAKEN, `cube` -0.699 % / `fixed` -0.369 % / `sealed`
-0.293 %, and the half that was *not* the depth is a refutation. See the Log.**
The depth was the whole thing (`CAP` 8 -> 32, `cube`'s pool misses 53,721 ->
4,289); a FIFO reordering moved the allocation count by exactly zero and cost
`fixed` +0.096 %. **Size a recycle list by the tail of its per-scope demand,
not by the mean**, and ask "choosing badly or running out?" before building a
policy change.

**(-163)(a) TAKEN by `(-166)` — `fixed` -0.640 % / `cube` -0.628 % / `sealed`
-0.636 %, the largest three-pool win since `(-144)`. The entry below is kept
for its census; `(b)` is still open and is re-priced under it.**

**(-163) THE PROGRAM'S ALLOCATIONS RANKED BY SOURCE LINE — THE CENSUS THIS
FILE HAS NEVER HAD, AND IT DOES NOT AGREE WITH THE GROWTH CENSUS ABOUT
ANYTHING.** `(-161)` proved `cg_growth.py` reads **zero** on `vec![x]` and on
`with_capacity(n)`, so the growth ranking every candidate since `(-103)` was
pulled off is a ranking of *re*-growths, not of allocations. This is the other
one: `scripts/cg_alloc_sites.py` over the whole binary (needle `crabomination`),
`profiling-lines`, `--dump-instr=yes`, six games, one thread, seed 1, at
`0d4b6a62`.

```text
  fixed — 500,822 allocations, 341,190 attributed to a crabomination line
    53,640  mod.rs:13570   computed_permanent_hinted   Arc::new(apply_layers_one_gated)
    27,040  mod.rs:10236   gather_continuous_effects_inner   ← (-161) took it
    16,182  mod.rs:13585   computed_permanent_hinted   Arc::new(gather_continuous_effects())
    16,182  mod.rs:13587   computed_permanent_hinted   Arc::new(apply_layers_one_gated)
    11,280  mod.rs:2755    GameState::clone
    10,942  mod.rs:2776    GameState::clone
    10,616  layers.rs:482  PrintedList<P>::push
     9,572  mod.rs:10209   gather_continuous_effects_inner   ← (-161)
     7,330  mod.rs:9688    frozen_effects
     6,954  cow.rs:62      CowBox<Vec<T>>::push
     6,954  cow.rs:65      CowBox<Vec<T>>::push
     6,012  mod.rs:10206   gather_continuous_effects_inner   ← (-161)
     5,794  mod.rs:10233   gather_continuous_effects_inner   ← (-161)
     5,022  actions.rs:11899
     4,748  stack.rs:464   advance_step

  cube — 797,395 allocations
   137,850  mod.rs:13570   computed_permanent_hinted
    46,292  layers.rs:482  PrintedList<P>::push
    39,224  mod.rs:9862    gather_continuous_effects_inner  ← (-162)
    28,992  mod.rs:13585   computed_permanent_hinted
    28,992  mod.rs:13587   computed_permanent_hinted
    22,698  mod.rs:2755    GameState::clone
    22,196  mod.rs:2776    GameState::clone
    12,408  mod.rs:3690    dispatch_board_scan   ← dropck, see (-158) row 1
    11,918  mod.rs:10209   gather_continuous_effects_inner  ← (-161)
    11,862  cow.rs:62      CowBox<Vec<T>>::push
    11,862  cow.rs:65      CowBox<Vec<T>>::push
    11,806  mod.rs:9688    frozen_effects
    11,480  actions.rs:11899
    11,168  mod.rs:3699    dispatch_board_scan   ← dropck
    10,648  mod.rs:25831   affected_from_requirement   ← (-158) row 3
```

**Read the two leads at the top before anything else on this list.**

*(a) — ✅ TAKEN, `(-166)`. The device was neither of the two the entry
proposed: not a value return (`(-111)`'s refutation stands) and not an arena,
but a thread-local **recycle list with the uniqueness check deferred to the
reuse site**, which is `(-27)` with one thing changed and it turns 12.5 %
into 68-82 %. Read `(-166)` before proposing anything else here.*

*(a) `computed_permanent_hinted` is 85,004 allocations of `fixed` (17 % of the
program's) and 195,834 of `cube` (25 %), in three lines, and it has never
appeared on this queue as an allocation row.* All three are `Arc::new` — two of
`ComputedPermanent` (72 bytes, size-asserted) and one of the gathered effect
list. ⚠ **`(-111)` is the prior art and it is a REFUTATION**: `computed_permanent`
by value was built, measured and reverted. The `Arc` here is the freeze-scope
memo's storage (`st.perms.push((id, cp.clone()))` keeps it, the caller gets the
clone), so "return by value" is the same change `(-111)` already priced. What
`(-111)` did **not** price is the allocation *count* — it had no instrument that
could see an exact-size `Arc::new` — so the row is worth re-opening only with a
device that keeps the memo and removes the box: an arena or a slab indexed by
`CardId`, not a value return. Do not re-run `(-111)`'s experiment.

*(b) `PrintedList<P>::push` — `layers.rs:482`, 46,292 on `cube` (the second
largest line there) and 10,616 on `fixed`, and nothing has ever named it.*
`push` materializes `printed ++ [value]` into a fresh `Box<[P::Item]>` **on
every call**, so n pushes to one list are n allocations of growing size. The
obvious fix — make `over` a `Vec` and push in place — costs eight bytes per
overlay list field inside `ComputedPermanent`, whose 72 bytes are held by an
assertion in `crabomination_tests`; `(-152)`'s ceiling rule applies and the
count of overlay fields decides it. **Price the struct first, then the row.**

⚠ **The two `GameState::clone` lines and the two `CowBox::push` lines are the
CoW family**, which `(-99)`/`(-100)`/`(-143)` worked to exhaustion — and
`(-157)` says a `release-fast` reading over-states that family by ~20 %
because thin LTO deletes a fifth of its calls. Discount before pricing.

**(-162) CLOSED — SUBSUMED BY `(-168)`, NOT TAKEN AS FILED.** The row is gone,
but not by the reuse pool this entry designed: `(-168)` recycles the
`Arc<Vec<ContinuousEffect>>` the freeze scope already holds, and a parked
handle brings its buffer back with it, so the `with_capacity` this entry
ranked is a `reserve` that no-ops. **The transferable half: when a buffer's
owner is already an `Arc` somebody parks, pool the OWNER, not the buffer** —
one device, two allocations, and no RAII guard or `into_vec()` escape hatch
needed. The entry is kept for its census.

**(-162) THE ONE ROW `(-161)` LEFT INSIDE `gather_continuous_effects_inner`,
AND IT IS `cube`'s, NOT THE BENCH'S.** `mod.rs:9862`, `all_effects`'
`Vec::with_capacity(base.len() + sa_cards.len())`: **39,224 of a six-game
`cube` run's allocations, the largest single line in the function on that
pool, and absent from `fixed`** — where the capacity is usually zero and
`with_capacity(0)` does not allocate. The buffer is the function's return
value, consumed as `&[ContinuousEffect]` and dropped by nine of its eleven
call sites, so an inline slot is wrong (the struct is large and the value
escapes) and the device is a **reuse pool**. The constraint is that
`gather_continuous_effects` takes `&self` and `GameState` must stay `Sync` —
which is why `in_layer_gather` is an `AtomicBool` and not a `Cell` — so a
`thread_local!` free-list with an RAII guard is the only shape that satisfies
both, and the reentrancy the guard has to survive is exactly the one
`in_layer_gather` documents (`board_keyword_matching`,
`permanents_with_abilities_removed` and the debug cross-checks all gather from
inside a gather). Two call sites take the value permanently (`Arc::new(...)`)
and need an `into_vec()` escape hatch.

⚠ **Price it with `cg_alloc_sites.py`, not `cg_growth.py`** — see `(-161)`:
`with_capacity(n)` is an exact-size allocation and the growth census reads
zero on it, exactly as it did on `vec![x]`.

**(-160) THE GROWTH CENSUS RE-TAKEN AT `4e10eefa`, ON ALL THREE POOLS AND
RANKED BY COUNT — `(-158)`'s OWN RULE APPLIED TO ITS OWN TABLE.** Fresh
`profiling-fast --no-default-features` dumps, six games, one thread, seed 1.
`(-158)` printed the top ten *by Ir*; ranking the same dump by **count** is
what its "rank first allocations by COUNT" rule actually asks for, and it
turns up nine engine rows the Ir ranking never showed. Base totals: `fixed`
905,228,901 / `cube` 2,466,595,486 / `sealed` 2,475,598,365.

```text
  cube — growth callers by count (315,914 growths over 114 callers)
    growths    calls   /call   grow Ir
    38,212    163,574   0.23   7,812,347  Vec::push_mut            ⓘ
    23,576     80,728   0.29   2,829,120  dispatch_board_scan      ✗ (-165)(a)
    19,684    365,837   0.05  13,653,258  Vec::from_iter           ⓘ
    16,394     11,310   1.45   2,892,179  auto_tap_for_cost_inner  ← (-159)
    16,366      6,128   2.67   9,507,753  resolve_combat           ✗ (-158)
    11,410     26,660   0.43   1,509,276  grant_scan
    10,688     26,820   0.40   1,725,806  IdSet::insert
    10,648     45,002   0.24   1,309,986  affected_from_requirement
     9,194     82,924   0.11   1,110,797  effective_mana_abilities_into
     8,684      7,032   1.23   1,204,770  deal_combat_damage_to_target
     8,296     41,856   0.20   5,590,426  CowBox<Vec<T>>::push
     8,286     93,874   0.09   1,220,715  granted_abilities_of_inner
     8,248      8,526   0.97   1,127,415  mana_source_table        ✗ its `reserve(16)`
     7,632      9,884   0.77   1,723,749  remove_from_battlefield_to_graveyard_raw
     7,566     12,538   0.60     958,846  cast_candidates
     7,336     49,786   0.15     880,989  trigger_grant_sources    ✗ (-165)(a)
     7,180     37,848   0.19   2,365,260  advance_step
     6,848      4,910   1.39   2,085,026  declare_blockers         ✗ (-158) took it
     6,832     12,294   0.56   1,398,257  send_to_graveyard
     6,722     10,062   0.67   1,025,177  resolve_top_of_stack_inner
     5,460      6,572   0.83   1,016,780  declare_attackers_banded
     5,262     59,456   0.09     990,780  gather_continuous_effects_inner

  fixed — the same ranking, and it is a DIFFERENT list
    13,922     47,304   0.29   2,142,688  Vec::push_mut            ⓘ
    11,806     30,234   0.39   2,970,367  gather_continuous_effects_inner  ← 0.328 %
     7,042      5,022   1.40   1,095,383  auto_tap_for_cost_inner  ← (-159)
     6,590      3,308   1.99   5,303,210  resolve_combat           ✗ (-158)
     5,650      9,556   0.59     764,678  IdSet::insert
     4,726     24,642   0.19   1,412,940  advance_step

  sealed — and a third list again
    17,820      8,458   2.11  11,926,373  resolve_combat           ✗ (-158)
    16,272     12,076   1.35   2,956,061  auto_tap_for_cost_inner  ← (-159)
    14,264     29,710   0.48   2,081,946  IdSet::insert
    11,924      9,450   1.26   1,604,800  deal_combat_damage_to_target
    11,068     31,010   0.36   7,075,133  fire_step_triggers       ← 0.286 %, sealed only
```

⚠ **The two `✗ (-165)(a)` rows are refuted TWICE OVER now, and the second
refutation is the one to read.** `12753f84` closed them on `SmallVec`'s missing
`#[may_dangle]`; `(-165)` got past that and built the binary, which reads
`fixed` **+0.9896 %** / `cube` +0.3243 %. `DispatchScan` is *returned*, so its
inline buffer is copied by every caller and filled only by the pools that carry
a grant static. Do not re-open either row, and read `(-165)` before proposing an
inline buffer anywhere.

**Three things this reading establishes.**

*One — `(-158)` verifiably worked, and the census is the receipt.* `declare_blockers`
went 19,616 → 6,848 growths and `pick_blocks_inner` 11,086 → 5,000 on `cube`, at
the same call counts. Neither is a lead any more; do not re-pull them off the
old table.

*Two — `auto_tap_for_cost_inner` is the largest engine growth row on ALL THREE
pools that is not already refuted*, at 1.35-1.45 growths a call: `cube` 16,394 /
0.117 %, `fixed` 7,042 / 0.121 %, `sealed` 16,272 / 0.119 %. That is `(-159)`,
taken this pass.

*Three — the top row on `fixed` is one no `cube` ranking would ever surface.*
`gather_continuous_effects_inner` is **0.328 % of `fixed` and 0.040 % of
`cube`** — an eight-fold pool inversion, and the mirror image of
`dispatch_board_scan`, which is 0.116 % of `cube` and *zero* of `fixed`.
CLAUDE.md's pool rule again, in the direction that catches the **bench**:
a row can be the bench pool's largest and near-invisible on the pool the queue
is usually ranked on. **It is the next entry to price**, and the shape is known
before a build is spent: the buffer is the returned `Vec<ContinuousEffect>`,
0.39 allocations a call, consumed as `&[ContinuousEffect]` at nine of its
eleven call sites and dropped. An inline slot is wrong (the struct is large and
the value escapes); the device is a **reuse pool**, and the constraint is that
`gather_continuous_effects` takes `&self` and `GameState` must stay `Sync` —
which is why `in_layer_gather` is an `AtomicBool` and not a `Cell`. A
`thread_local!` free-list with an RAII guard is the only shape that satisfies
both; the reentrancy the guard has to survive is the same one
`in_layer_gather` documents.

**FOUR MORE ROWS PRICED AND RULED OUT, so nobody re-reads them.**

* **`PrintedList::push` — two facts for `(-163)`'s lead, not a refutation of
  it.** 46,292 allocations, 5,565,774 Ir of `__rust_alloc` alone, 11.83 M
  inclusive (0.48 % of `cube`). (i) Its `__rust_dealloc` edge is **316**, so
  **99.3 % of pushes are the first push on a fresh list** — amortising repeated
  pushes (a `Vec` instead of a re-materialised `Box<[T]>`) buys essentially
  nothing, and the whole row is one allocation per (permanent, grant). (ii) The
  box escapes through `into_overlay` into the cached `ComputedPermanent`, so an
  inline slot there is priced at that struct's size by `(-165)`, and its element
  is a `Keyword`, not `(-161)`'s free `u32`. **Price `ComputedPermanent` before
  spending the build**, which is what `(-163)`'s entry already says.
* **`CowBox<Vec<T>>::push` (8,296 growths / 5,590,426 Ir) is the CoW unshare
  itself** — one allocation for the copy, which is the type's whole point. Its
  37,472 `__rust_alloc` calls are `Arc::make_mut`'s materialisation with room
  for the appended element, a previous pass's fix.
* **`Vec::spec_from_iter_nested::from_iter` is 2.70 % of `cube` self and it is
  diffuse.** 399,015 calls over **92 callers**, largest `Map::fold` at 25,982
  (6.5 %), largest engine caller `check_state_based_actions_into` at 24,326.
  `(-156)` prices a std-adapter rewrite at ~10 % of the row's self Ir *per
  site*, and there is no site — it is every `.filter(..).collect()` in the
  program.
* **`Decider::kind`'s thirty-eight discriminant gates are worth nothing.**
  `matches!(self.decider.kind(), DeciderKind::Auto)` materialises a 56-byte enum
  with drop glue through a virtual call, thirty-eight times over — and
  `AutoDecider::kind` reads **91,128 Ir, 0.004 % of `cube`**. An `is_auto()` on
  the trait is a tidy, not a candidate.

ⓘ **`Vec::push_mut` and `Vec::from_iter` are not rows, they are the
un-inlined remainder.** Callgrind charges a growth to the innermost frame it
can name, so every `push` the local inliner declined to fold into its engine
caller lands in one of these two buckets instead of in the function that owns
the buffer. They are the largest "callers" on every pool and there is nothing
at either address to fix — read past them.

**(-158) PARTLY TAKEN — the `Id*` locals, `fixed` -0.212 % / `cube` -0.139 % /
`sealed` -0.187 %; see the Log. What is still open is listed at the end of this
entry.**

**(-158) THE ALLOCATION CENSUS, RE-TAKEN WITH THE ONE COLUMN IT NEVER HAD:
`malloc` VS `realloc`. 81 % OF THE PROGRAM'S GROWTHS ARE FIRST ALLOCATIONS, SO
`with_capacity` IS THE WRONG TOOL AND `cg_growth.py`'S OWN RANKING POINTS AT
THE WRONG ROWS.** `finish_grow` is **336,724 calls / 16.3 M self / 76.2 M
inclusive = 3.17 % of `cube`** (`malloc` 273,215 / 25.95 M, `realloc` 63,509 /
33.92 M). Fresh dumps, `profiling-lto`, six games, one thread, seed 1, taken one
commit either side of `39528f0f`; `fixed` is 125,607 growths over 68 callers.

`--separate-callers=2` splits each grow context by which allocator entry it
took, and that is the column that decides the tool:

```text
  cube, grow contexts by allocator entry     malloc   realloc   realloc%
    dispatch_board_scan (two Vec fields)     23,576         0       0.0 %
    affected_from_requirement                10,648         0       0.0 %
    actions:: (four separate contexts)       33,122         0       0.0 %
    trigger_grant_sources                     7,336         0       0.0 %
    combat::resolve_combat (one context)     11,482        82       0.7 %
    bot::pick_blocks_inner                    7,758     1,532      16.5 %
    CowBox<Vec<T>>::push                         50     7,552      99.3 %
```

**A 0 % row is a buffer that reaches the heap once and never re-grows**, and
`cg_growth.py`'s doc already says a reserve buys nothing there — but the script
ranks by *growths per call*, which puts `declare_blockers`' 4.00 at the top and
`dispatch_board_scan`'s 0.29 at rank 40, when the second is the larger row in
absolute allocations and every one of them is avoidable. **Rank first
allocations by COUNT and re-growths by PER-CALL; they are two different
census questions and one script column cannot answer both.** ⚠ And 4.00
growths a call is *not* a growth ladder here: `declare_blockers` allocates four
separate buffers once each, which is why `(-71)`'s inline-slot device is the
only thing that touches it and a `with_capacity` is not.

**The ranked engine rows, by the Ir their growths actually cost** (the
script's fourth column, both pools, so a row can be sized before it is taken):

```text
                              cube growths  grow Ir     %      fixed growths  grow Ir     %
  resolve_combat                    16,366  9,401,948  0.391          6,590  5,324,895  0.606
  gather_continuous_effects_inner        —          —      —         11,806  2,920,019  0.332
  auto_tap_for_cost_inner           16,394  2,858,851  0.119          7,042  1,083,388  0.123
  dispatch_board_scan               23,576  2,781,968  0.116              0          0  0.000
  pick_blocks_inner                 11,086  2,545,945  0.106          4,348    828,038  0.094
  CowBox<Vec<T>>::push               7,602  4,659,644  0.194          3,324  1,951,258  0.222
  declare_blockers                  19,616  4,266,421  0.177          9,216  1,106,781  0.126
  advance_step                       7,180  2,335,948  0.097          4,726  1,406,484  0.160
  affected_from_requirement         10,648  1,291,732  0.054              —          —      —
  trigger_grant_sources              7,336    865,648  0.036              0          0  0.000
```

⚠ **`dispatch_board_scan` and `trigger_grant_sources` are `cube`/`sealed`-only
and `--bench` cannot see either** — the `fixed` archetypes carry no
`GrantTriggeredAbility` static, so both vectors stay empty and never allocate.
CLAUDE.md's pool rule, in its sharpest form yet: a row can be the program's
largest allocation site on one pool and *literally zero* on the one the bench
measures.

**`resolve_combat` is the top allocation row on BOTH pools — 0.61 % of `fixed`,
0.39 % of `cube` — and it is the one to take next.** Its own body holds almost
no buffers; the growths are `resolve_combat_damage_with_filter` and the rest of
the combat-damage chain inlined into it, and **808 Ir a growth on `fixed`
against 118-226 everywhere else** says these are the big-buffer reallocs, not
the small first allocations the rest of the table is made of. ⚠ **Attribute the
sites before patching**: two levels of `--separate-callers` cannot see inside an
inlined callee, so this row needs `profiling-lines` + `cg_lines.py`, which is
the cold build this pass did not spend. `(-71)`'s eight inline slots are the
precedent and its rule is the one to re-read first — "**read both columns
before sizing an inline buffer**", and the second column now exists.

**WHAT `scripts/cg_alloc_sites.py` SAID WHEN IT WAS POINTED AT
`resolve_combat`, AND WHY THAT ROW IS NOT THE LEAD IT LOOKED LIKE.** Eight
lines carry its 9,194 `fixed` allocations, and none of them is takeable:

```text
  fixed, allocations by source line, inside resolve_combat
    2,878  combat.rs:3402  `events.reserve(32)` — a PREVIOUS pass's fix; the
                            comment records the 34,438 growths it removed
    1,962  combat.rs:3667  } `b.damaged_by_this_turn.push(..)` — a Vec field on
    1,750  combat.rs:3852  } `CardInstance`, i.e. inside the CoW card group
    1,600  combat.rs:2815  `default_damage_split`'s `.collect()`
      384  combat.rs:2998  `combat_damage_order.get(..).cloned()`
      190  combat.rs:3581  `combat_damage_assignment.get(..).cloned()`
```

The largest site is a deliberate `reserve` and the next two are a *stored*
`Vec` on the card, where an inline buffer is priced at the CoW group's size
class rather than at the allocation (`(-152)`'s ceiling rule). **A row that is
0.61 % of a pool can be entirely made of allocations nobody should remove**,
and that is worth more than the row was: it is why the ranking has to be read
down to the line before a build is spent.

**THE THREE THAT WERE OPEN, AND WHERE EACH LANDED:**

1. **`dispatch_board_scan`'s two `Vec` fields — REFUTED, AND THE REASON IS A
   BORROW-CHECKER PROPERTY OF `SmallVec`, NOT A COST.** `mod.rs:3690` /
   `mod.rs:3699`, 23,576 allocations, 0 re-growths, 0.116 % of `cube` and ZERO
   of `fixed`; `trigger_grant_sources`' `out` (7,336, 0.036 %) is the same
   buffer built by the other walker, and it falls with it. `Vec`'s destructor
   carries `#[may_dangle]` (the dropck eyepatch) and `SmallVec`'s does not, so
   a `SmallVec<[TriggerGrant<'a>; N]>` — whose element borrows the battlefield
   out of `&'a self` — holds that borrow to the end of the enclosing scope
   instead of releasing it at the last use. All three consumers take `&mut
   self` afterwards, and the swap turns twelve of those into `E0502`/`E0506`:
   `actions.rs:11828 / 11839 / 11857` (`self.stack.push`), `stack.rs:1059 /
   1101` (`self.delayed_triggers`), `mod.rs:18899 / 18902 / 19410 / 19413 /
   19417` and two more.
   ⚠ **The `Vec` is load-bearing for the borrow checker, not for storage, and
   this is a CLASS, not one row.** Every inline-buffer candidate whose element
   type carries a lifetime out of `&self` has the same blocker — which is why
   `(-71)`'s eight slots and `(-158)`'s twelve `Id*` locals are all
   `CardId`-shaped. **Ask "does the element borrow?" before pricing an inline
   slot.** If it does, the entry is no longer the mechanical change the census
   makes it look like: it costs a scope restructure (hoisting the grant
   consumers into a block that ends before the first `&mut self`), and the
   lifetime-erased scratch buffer that would sidestep it needs `unsafe`, of
   which the engine has none outside `server/`'s three `set_var` sites.
2. **`auto_tap_for_cost_inner` — TAKEN as `(-159)`; see the Log.**
3. **`affected_from_requirement`, 10,648 / 0.054 % of `cube`, 0 re-growths —
   still open, and priced.** The buffer is `types: Vec<CardType>`, which is
   *moved into* `AffectedPermanents::All.card_types`, so an inline slot there
   does not remove the allocation, it moves it into a struct that
   `ContinuousEffect` clones once per gather — `(-152)`'s ceiling rule, and the
   same shape that made `resolve_combat`'s two largest lines untakeable. Take
   it only with a reading of how often that struct is cloned.

**(-157) THE PROFILE OF RECORD TRANSFERS TO THE SHIPPED BINARY — MEASURED ON
THE WHOLE PROFILE, NOT JUST ON `(-156)`'s ROW.** Every row in this file is a
`release-fast` (cgu 16, no LTO) reading; every ML binary `selfplay_train` runs
is `--release` (cgu 1 + thin LTO). `(-156)` raised the doubt in its own text —
"a win measured on `release-fast` may be worth **zero** in the binary the actor
runs" — and its gate answered it *for one row*. This is the same question asked
of the other forty. Two `profiling-lto` dumps against the two `release-fast`
ones, `--games 6 --threads 1 --seed 1`, `--demangle=no`, joined on the
demangled key with the `17h<hash>E` and `.llvm.N` suffixes stripped (taken at
the free-activation tip, one commit either side of `39528f0f`):

```text
  pool    profiling-lto     release-fast    LTO delta
  cube    2,404,499,361    2,471,534,012     -2.712 %
  fixed     879,376,175      907,757,788     -3.127 %
```

**Thin LTO is worth ~3 % of Ir and it does NOT re-shape the profile.** Of the
forty largest rows on each pool, every engine function sits within **±0.15
points** of the share it has under `release-fast` — `dispatch_triggers_for_events`
5.428 vs 5.423, `gather_continuous_effects_inner` 3.387 vs 3.367,
`compute_permanent_pass` 3.232 vs 3.216, `computed_permanent_hinted` 2.427 vs
2.407 — and **the call counts are identical to the unit** on all of them. The
ranking a candidate is pulled off is the same ranking in the shipped binary.

The whole -3 % is four rows, and each one is LTO **deleting calls**, not
speeding them up:

```text
  cube                        lto calls    rf calls   lto Ir      rf Ir
    Vec::clone                  368,165     459,559   17.8 M     23.5 M   -21 % of calls
    Arc::clone_from_ref_in      101,624     120,824   56.6 M     57.4 M   -16 %
    Arc::drop_slow              371,315     371,315   21.5 M     24.3 M
    sba_board_scan               20,506      20,506   21.6 M     24.0 M
  and one row LTO makes WORSE at an identical call count:
    RawVecInner::finish_grow    333,830     333,830   16.3 M     13.4 M   +22 %
```

`fixed` reproduces all five signs. **The transferable half: a `release-fast`
reading over-states any change to the CoW `Vec`/`Arc` clone family by about a
fifth, because a fifth of those calls do not exist in the shipped binary — and
understates nothing else.** Rank on `release-fast` as before; discount a
clone-family ceiling by ~20 % before quoting it as an actor win.

**(-156) CLOSED — MEASURED, REFUTED, REVERTED. Read to the end of the entry
before re-proposing it: the shim survives thin LTO, and rewriting its largest
instance recovered 10.9 % of the row it deleted.** The original filing follows,
because the reasoning that led here is the useful part.

**THE `FnMut for &mut F` SHIM IS STILL 0.61 % OF `cube` AND IT IS
STD'S OWN ADAPTER CODE, NOT OURS — PRICE IT AGAINST `release` BEFORE
BUILDING.** `zone::Battlefield::lane`'s doc records that passing a predicate
by reference cost 18.8 M Ir through
`core::ops::function::impls::<impl FnMut<A> for &mut F>::call_mut`. At
`26cea814` that row still reads **99,572 calls / 14,967,638 self Ir / 150.3 a
call = 0.61 % of `cube`**, and no engine site passes a closure by reference
any more. Traced to its source with `scripts/cg_chain.py` over a
`--demangle=no` dump — two levels of caller, because one is not enough when
the immediate caller is itself a std generic and the merged demangled table
cannot tell its monomorphizations apart:

```text
  shim callers (calls / inclusive Ir)     ... and *their* caller
    Vec::from_iter        12,992 / 30.6 M   bot::pick_blocks_inner   4,578 calls / 32.4 M
    SmallVec::extend      20,714 /  4.6 M   combat::resolve_combat   6,128 / 5.5 M
    Vec::from_iter        11,292 /  8.4 M   bot::pick_attacks_inner  4,548 / 9.6 M
    Vec::from_iter        25,026 /  0.7 M   GameState::process_echo  2,812 / 1.5 M
```

**It is `Filter::next` and `FilterMap::next` themselves.** Both are written in
std as `self.iter.find(&mut self.predicate)` / `find_map(&mut self.f)`, so
every `.filter(..).filter_map(..).collect()` instantiates the adapter with
`F = &mut Closure` and the local inliner at `codegen-units = 16` with no LTO
sometimes declines it. `pick_blocks_inner`'s `attacker_info` chain is the
largest single instance.

⚠ **This is the one candidate on the list whose size is profile-dependent in
the direction that matters.** `cg_calls.py`'s own rule says a std generic the
inliner declined is real — but the ML binaries are built with `--release`
(cgu 1 + thin LTO), where these adapters do inline, so a win measured on
`release-fast` may be worth **zero** in the binary `selfplay_train` actually
runs. **Take a `profiling-lto` reading of the shim's call count first** (PERF's
"How to measure" names it as exactly this instrument); if it is near zero
there, close this entry and do not rewrite the chains. Only if it survives LTO
is the fix — an explicit `for` loop with `push` in `pick_blocks_inner` —
worth the ~0.1-0.2 % it could carry. `(-119)` is the warning: a restructure of
this shape read **+0.07 %** and was reverted.

**THE `profiling-lto` READING IS TAKEN AND THE SHIM SURVIVES — the gate above
opens, not closes.** Both sides at `007893c7`, `--no-default-features`, six
games, one thread, seed 1, `cube`:

```text
                       total Ir       shim calls   shim self Ir   share
  profiling-fast   2,470,323,725         99,572     14,967,638    0.606 %
  profiling-lto    2,404,419,437         74,482     16,875,046    0.702 %
```

**The gate asked one question — "is it near zero under LTO" — and 74,482 calls
is not near zero.** Thin LTO removes a quarter of the call sites (99,572 →
74,482) and leaves three quarters standing, so the call overhead the candidate
is about is not something LTO deletes, and a win here is not worth zero in the
`--release` binary `selfplay_train` runs.

⚠ **Do not read the self-Ir column as "the survivors got dearer".** 150.3 Ir a
call becomes 226.6, and there are two explanations these two dumps cannot
separate: LTO kept the expensive adapters and dropped the cheap ones, or LTO
inlined the *closure bodies into* the adapters, moving cost from the caller's
row into the shim's. The second is at least as likely and would mean the total
did not grow at all, only moved. **A self-Ir comparison across two inlining
regimes is not a like-for-like measurement** — the call count is, which is why
the gate was written on the call count.

**THE TWO EXPLANATIONS ABOVE ARE SEPARABLE, AND `readelf -sW` SEPARATES THEM
FOR THE COST OF ONE COMMAND.** It is the second one, and it also predicts the
10.9 % the rewrite below measured. Rank the seventeen `profiling-lto`
instances by Ir/call beside their **symbol size**:

```text
  cube, profiling-lto        calls    self Ir   Ir/call   bytes
    ee7dc5f2 (pick_blocks)  12,992  6,574,752     506.1    4,122
    6b9566ad (resolve_comb) 20,714  4,453,628     215.0    1,993
    2b88a4b1 (dispatch_trg)  9,862  2,371,560     240.5      239
    bd66cc81 (sba sweep)     5,096  1,732,612     340.0      456
    e4b6b6df (pick_attacks) 11,292  1,166,662     103.3    1,753
    ---- the nine genuine forwarders ----
    (9 rows)                19,298    263,572      13.7   98-545
    TOTAL                   92,024 17,115,446     186.0
```

**A `call_mut` 4,122 bytes long is a closure body the inliner folded into
std's symbol, not a three-instruction shim.** Callgrind keys the row by the
symbol the code landed in, so the name says "adapter" and the size says
"predicate". The nine rows that really are forwarders run 13.7 Ir a call and
hold **0.011 % of `cube`** between them; the five that carry 96 % of the cost
are bodies a `for` loop would still execute. The removable part is the call
and return per element — 92,024 calls at ~6-10 Ir, **0.023-0.038 %** — which
is the same answer, arrived at without a build, as the -0.0193 % the rewrite
below actually measured on one instance.

⚠ **THE RULE THIS FILE DID NOT HAVE: price a std-generic row by its CODE SIZE,
not by its name.** `readelf -sW <bin> | grep <symbol>` is one command and it
tells a forwarder from a body. Every `Filter::next` / `FilterMap::next` /
`SpecFromIterNested` row in this file's history was ranked on the assumption
that the name describes the work; on this dump that assumption is wrong for
the five rows holding 96 % of the cost and right for the nine holding 1.5 %.

⚠ **The dumps do not symbolize the same way and one of them lies if you
believe `cg_symbolize.py`'s header.** The `profiling-fast` dump comes back as
raw addresses and needs the script (bias `0x108000`, 2,350 of 2,395 addresses
in a FUNC); the `profiling-lto` dump is **already symbolized by valgrind**, and
running the script on it anyway reports "2/43 addresses in a FUNC", which reads
like a failure and is not — there were only 43 addresses left to resolve.
PERF's standing advice ("run `cg_edges.py` on the raw dump first; symbolize
only if the rows come out as addresses") is the check; apply it per dump, not
per session.

⚠ **And LTO merges every monomorphization into one row**, so the per-instance
attribution that names `pick_blocks_inner` only exists on the `profiling-fast`
side. Take the caller table there:

```text
  shim instance (calls / self Ir)        caller (calls / Ir incl)
    12,992 / 4,375,134   Vec::from_iter    pick_blocks_inner    4,578 / 32.4 M
    20,714 / 4,453,628   SmallVec::extend  resolve_combat
     9,862 / 2,371,560   Vec::from_iter    (auto_target_for_effect_avoiding)
     5,096 / 1,707,132   Vec::from_iter    check_state_based_actions_into 3,906
    11,292 /   805,076   Vec::from_iter    pick_attacks_inner   4,548 / 9.6 M
    25,026 /   675,180   Vec::from_iter    process_echo         2,812 / 1.5 M
    10,124 /   394,938   Vec::from_iter    auto_target_for_effect_avoiding
```

`pick_blocks_inner` is the top row by self Ir and the ceiling on rewriting it
alone is **0.177 % of `cube`** — the shim's whole self cost is 0.606 %, so no
single site is the entry.

**(-156) CLOSED — THE FIX WAS BUILT, IT WORKED EXACTLY AS INTENDED, AND IT IS
WORTH A TENTH OF WHAT ITS ROW SAYS. REVERTED.** `attacker_info`'s
`.filter().filter_map().collect()` rewritten as a `for` loop with `push`,
nothing else changed, `--bench` byte-identical (195,806 / 27.49 / 0 stalls,
determinism ok):

```text
  pool    base            candidate       delta
  cube    2,470,323,725   2,469,847,990   **-0.0193 %**   (-475,735 Ir)
  fixed     907,147,703     907,004,231   **-0.0158 %**   (-143,472 Ir)

  and the change did land, precisely:
  shim calls     99,572 -> 86,580       -12,992  = the whole instance
  shim self Ir   14,967,638 -> 10,592,504   -4,375,134  = the whole row
```

**Read those two blocks against each other, because they are the finding.**
The rewrite deleted the adapter's entire row — 4,375,134 Ir of self cost, the
exact number the candidate was written on — and the program got only 475,735
Ir cheaper. **10.9 % of the row was real overhead; the other 89 % moved into
the `for` loop**, because a std adapter's self Ir is mostly the *iteration it
is driving* — the bounds check, the advance, the `find` loop — and a hand
loop has to drive it too. Only the call-and-return frame is removable.

**THE RULE, and it applies to every std-generic row this file ranks:
`cg_calls.py`'s "a std generic the inliner declined is real" says the row is
not an artifact. It does not say the row is an opportunity.** Price an adapter
rewrite at ~10 % of the adapter's self Ir, not at its self Ir. On that rate
the whole shim — all 99,572 calls, 0.606 % of `cube` — is worth **0.066 %** if
every site were rewritten, which is below the floor this file carries and an
order of magnitude under the 0.1-0.2 % the entry estimated. `(-119)` is the
same shape from the other side: a restructure that read +0.07 % and was
reverted.

Reverted rather than kept. The win is real and deterministic (Ir is exact for
a fixed binary and workload), but 0.019 % is unmeasurable in wall clock —
`scripts/bench_ab.py` separates ~2 % and not 0.3 % — and keeping it would
leave a perf-justified shape in `pick_blocks_inner` that the numbers do not
support, which is the exact failure mode CLAUDE.md's "a profile number in a
code comment" rule exists to prevent.

**(-155) CLOSED — TAKEN, `cube` -0.114 % / `fixed` -0.163 %.** The CR
704.5a/c loss check's `effective_life` / `effective_poison` walked the team
table with a heap-`Vec` `contains` before testing the `shared_life` that
rejects every team outside 2HG. See the Log for the rows; the rule is
`(-116)`'s and this is its fourth application.

**(-128)'s LAST OPEN HALF IS CLOSED — THE SWEEP BODY IS SPLIT, AND IT IS THE
RULES' OWN OBLIGATION PLUS THE LOSS CHECK.** `(-128)` ended with "what is
still open here is the 2,397 Ir of sweep body, of which the CR 704.5f/g/h
death sweep is an obligation and the rest is gate evaluation. That has never
been split out." Split here off fresh dumps at `26cea814`
(`profiling-fast --no-default-features`, six games, one thread, seed 1;
`cube` 2,449,582,202 Ir, `fixed` 895,110,008), `cg_edges.py --callees
check_state_based_actions_into`. Costs are inclusive, as callgrind records
edges:

```text
  cube, 20,506 sweeps          calls    incl Ir     what it is
    sba_board_scan             20,506   25,243,116  the scan, 1.03 % (as (-128) priced it)
    Vec::from_iter (nested)    23,138   37,464,826  every collect in the body, merged
    remove_from_bf_to_gy_raw    8,882   43,240,854  the deaths — 1.77 %
    compute_permanents          3,746   19,159,611  the death sweep's scoped layer pass
    Battlefield::walk_and_store 8,772    9,113,342  the lane misses under it
    note_creature_death         8,860    7,109,226
    dying_snapshot              8,860    4,497,594
    effective_life             41,226    1,833,626  } the loss check, 2.01 and
    effective_poison           40,182    1,748,924  } 1.96 calls a sweep
  fixed, 10,230 sweeps
    sba_board_scan             10,230   10,942,878  1.22 % — the inverted ratio again
    effective_life             20,610      917,620
    effective_poison           20,314      883,222
```

**There is no ungated collect left in the body, and that is the finding.**
`--demangle=no` separates the merged `from_iter` row into eight
monomorphizations and **none of them is called once per sweep**: the two
largest are 4,390 calls x 2,534 Ir and 3,906 x 2,907, which track the ~4,000
sweeps that are not `DeathSweep::Skip`, and an 8,860-call row is one per
death. Every other `collect()` in the function sits behind an `if scan.*` that
an ordinary board leaves false — `(-88)`'s gating did its job and nothing has
rotted back.

So the body is exactly three things: the scan `(-128)` already priced, the
death legs charged to 8,882 actual deaths (an obligation — the rules have to
compute layers for the creatures that might die and move the ones that do),
and the loss check. **The only row in it that was neither is the loss check,
and `(-155)` took it.** Nothing else here is worth a build; do not re-split
this function.

**(-152) CLOSED — see the hundred-and-nineteenth pass.** `cube` **-0.422 %** /
`fixed` **-0.376 %** cumulative for a token lane on the four card zones plus
the sixteen bytes it needed. Two things it leaves behind:

* **`PlayerData` has 40 bytes of headroom and a hard ceiling at 1,016.** The
  next field added to the hot player group is priced at glibc's smallbin
  boundary, not at its copy — the entry has the 11.3 M reading.
* **The lane device now covers instance state**, which widens what
  `zone::Battlefield`'s existing `lane(shift, walk)` can answer. The
  candidate below is the first user of that.

**(-153) CLOSED — TAKEN, `cube` -0.134 %, `fixed` -0.142 %, and it is HALF the
ceiling filed below.** See the hundred-and-twenty-first pass for the row
table. The one line worth carrying forward: **price a memo by
reads-per-invalidation, not by the size of the walk it replaces.**
`Battlefield`'s lanes are cleared by membership changes, which happen several
times a turn against ~20,012 sweeps, so a once-per-sweep question misses about
three times in four. The entry as it was filed:

**(-153) `pt_reduction_in_scope` IS AN UNGATED WHOLE-BATTLEFIELD WALK ONCE PER
SBA SWEEP, AND `zone::Battlefield` ALREADY HAS THE LANE FOR IT.**
`game/mod.rs:9558` is
`self.continuous_effects.iter().any(..) || self.battlefield.iter().any(card_can_reduce_toughness)`,
called from `check_state_based_actions_into`'s death-sweep scope check — once
a sweep, over the whole board. The line profile shows the predicate's body as
a run of **eight identically-costed rows at 781,104 Ir each** (`game/mod.rs`
23938-23964), the signature of a straight-line fallback chain run ~781 k times
a six-game `cube` run — ~39 card visits a sweep, which is the board.

```text
  cube, --in check_state_based_actions_into, after (-152)
    game/mod.rs:23939/23943/23945/23946/23957/23964 …  781,104 each
    game/mod.rs:23806 (the type-scan half)           1,562,208
```

**Ceiling ~0.25-0.35 % of `cube`**, and the shape is precedented twice over:
`Battlefield::has_damage_scaler` and its five siblings are exactly
`lane(LANE_X, walk)` over a whole-board predicate, and `(-152)` established
that an *instance*-reading predicate is sound on one of these lanes because
every `&mut` route clears the word. `card_can_reduce_toughness` reads
`attached_to` and `soulbond_partner` besides the definition, so it needs that
argument; it also reads `def.static_abilities`, `station`, `level_bands` and
`equipped_bonus`, which is a real walk rather than a length check — the
condition `(-87)` says a presence bit needs.

⚠ **Price the hit rate first.** `Battlefield`'s three `&mut` chokepoints fire
**139,280 times a `cube` run against 20,012 sweeps**, so the lane is cold
about seven reaches out of every eight if the reaches interleave evenly.
`(-152)`'s piles had the same shape on paper and still hit often enough to
save 10 M, because the writes cluster — but that was luck, not a prediction.
`zone::BATTLEFIELD_REACHES` (the `trig-census` counter `(-128)` added) already
counts them; extend it to count *sweeps since the last reach* and the hit rate
is a number before a build is spent.
**(-151) THE QUEUE WAS RE-PROFILED FROM SCRATCH AT `b8f05380` AND NOTHING HAS
MOVED SINCE `bda1a69d` — WHICH IS THE FINDING, AND IT IS THE SECOND TIME.**
Fresh dumps, `profiling-fast --no-default-features`, six games, one thread,
seed 1, `--dump-instr=yes`: `cube` **2,455,861,932**, `fixed`
**905,973,489**. The `fixed` figure reproduces the committed
905,973,251 to **+238 Ir (0.00003 %)**, so the instrument and the tip are both
where the Baseline says they are.

Every row is `(-138)`'s three groups again, in the same order and within a
tenth of a point of the same share. **Rank by call count and read Ir/call**
(`cg_calls.py`) turns up nothing new above the floor either: the top rows are
the allocator (1,278,254 allocations), `__memcpy` (1,037,197 calls, 54.1
Ir/call, still 2,319 callers), `event_matches_spec` at the 14.7 Ir/call
`(-129)` left it at, and `has_keyword` at the 31.3 `(-114)` refuted.
`SpecFromIterNested::from_iter` is 395,865 calls over **92 callers** whose
largest is 25,730 (6.5 %) — diffuse, unchanged, do not re-open.

**Three readings are new and each one closes something, so nobody spends a
build on them.**

*(a) `computed_permanent_hinted`'s allocation is 1.5 % of `cube` and both
devices for it are already refuted.* 354,566 calls, and **201,452 of them
allocate (56.8 %)** — 22,056,133 Ir on the alloc side alone (0.90 %), and with
the matching free at `(-139)`'s 183 Ir a malloc+free pair, **~36.9 M = 1.50 %**.
The callee table splits it exactly: **172,648 memo misses** each taking one
`Arc<ComputedPermanent>` and one `compute_permanent_pass`, plus **35,964
scope-first gathers** each taking one `Arc<Vec<ContinuousEffect>>`. That is the
largest single allocation site in the engine and it stays open only in the
sense that *nothing safe reaches it*: the recycling pool is refuted by the
standing rules ("a pool can only recycle a handle whose lifetime the pool's
owner bounds", `fixed` +0.137 %) and the by-value form is `(-111)`, built and
reverted.

*(b) THE `perms` LINEAR SCAN IS NOT THE PER-CALL COST — do not build an index
for it.* The function reads **166.5 Ir/call self on `cube` against 153.0 on
`fixed`**, i.e. **8.8 %** more for a board several times larger. A scan over
the scope's memo would scale with the board and this does not; the ~150 Ir is
the fixed body — the reentrancy load, the freeze-depth load, the mutex
acquire/release the guard inlines, the `Arc::clone` and the return. Same
device as `(-128)`'s inverted `fixed`/`cube` ratio, and it costs nothing to
take before proposing a memo.

*(c) The CoW deep copy is 3.94 % of `cube` and its unshare rate is 12.4 %.*
`Arc::clone_from_ref_in` is 119,938 calls at **479.7 Ir self**, 96.8 M
inclusive, and **112,034 of them come from `make_mut`** — 12.4 % of its
905,060 calls. Against 22,556 `GameState::clone`s that is **~5 unshares a
clone**, which is the per-`CardData` zone barrier doing what `(-100)` said it
does (`CardData` is 48 % of the deep copies), not a group that could be made
to unshare less. `GameState::clone` self is now **501.2 Ir/call** (544 after
`(-144)`) over ~6 `Vec::clone` a clone.

⚠ **And one number to stop reading as ours**: `__rustc::__rust_no_alloc_shim_
is_unstable_v2` is **1,278,510 calls at 1 Ir**, one per allocation, and with
`__rust_alloc` / `__rdl_alloc` / `__rust_dealloc` / `__rdl_dealloc` the whole
Rust allocator *shim* is **16.6 M = 0.68 %** on top of glibc's own malloc/free
rows. It is rustc-emitted and not reachable from this source tree; it is
already inside every allocation-count ceiling this file quotes.

**So the honest state of the queue is: `(-145)` is the only entry with a
number on it and it is deliberately refused.** The next real move here is not
another micro-row — it is either taking `(-145)` with the Miri'd `unsafe` box
and an explicit decision that the trade is wanted, or accepting that the
profile is at its floor for this shape of engine and spending runs on the
build lever (PGO, which is measured at -23 % and opt-in) instead.
**FILED WITH NUMBERS, NOT TAKEN: `cast_spell_with_convoke` IS THE LARGEST
SINGLE `make_mut` CALLER AND ~46 % OF ITS CALLS ARE REJECTIONS.** A fresh
`cube` dump at `5cba7ddc` ranks the `make_mut` callers, and the top row is
not close:

```text
  callers of make_mut (cube, 6 games)      calls    Ir incl
    cast_spell_with_convoke              132,054  37,900,852   1.55 %
    resolve_combat                       101,432   6,464,048
    resolve_top_of_stack_inner            77,510  10,407,253
```

**13.3 `make_mut` a call, and the shape says one deep copy plus twelve
refcount checks.** The function is entered 9,898 times and reaches
`finalize_cast` 5,320 — so **4,578 casts (46 %) are rejected**, and the
rejection paths all sit *after* `self.players[p].remove_from_hand(card_id)`
at `actions.rs:6643`, with **~50 `self.players[p].hand.push(card)` sites**
putting it back. Each rejected cast therefore pays one full `PlayerData`
unshare (~2,000 Ir) to take a card out of a hand and put it back.

**Ceiling ~0.19 % of `cube`**, and it is filed rather than taken because the
device is a restructure of a **1,363-line** function: the ~50 checks between
the removal and `finalize_cast` would have to run against a `&CardInstance`
still in the hand, and several of them mutate the owned `card`
(`cast_from_hand`, `kicked`, the waterbend rider). ⚠ **Price the rejection
rate again before starting** — 46 % is a bot-policy number, not a constant,
and the whole win is proportional to it.

**(-154) CLOSED — see the hundred-and-twentieth pass.** `cube` **-0.224 %**
at eighteen games (-0.148 % at six) / `fixed` **-0.263 %** for gating three
damage tallies on the one card each serves. ⚠ **The `debug_flag` word is now
the vehicle for this whole class and it has a fixed cost**: 233 `format!`s a
process, once per distinct card name, five `contains` over each. Adding a
sixth flag is free at run scale and adds ~0.7 M Ir to a six-game dump — read
a six-game number for a new flag as an overstatement.

**(-150) CLOSED — see the hundred-and-eighteenth pass.** `cube` **-0.297 %** /
`fixed` **-0.286 %** for making `CastProfile` four bytes and `Copy`. **The hot
player group is now done**: what is left in `PlayerData` is nine `Vec`s that
are empty in most games, and an empty `Vec` clone is ~10 Ir with no
allocation. **`CardData` is the next group to read** — 32,562 unshares at 500
Ir each, `CardMemo::clone` its identifying callee.

**(-149) CLOSED — see the hundred-and-seventeenth pass.** `cube` **-0.250 %**
/ `fixed` **-0.460 %** for gating one `HashMap` write on the one card that
reads it. **The device generalises and the list below is thin, so this is the
question to ask next**: for each collection still in a *hot* CoW group, who
reads it, and can the write be gated on them? `--demangle=no` +
`cg_edges.py --callers make_mut` names the group and ranks the reaches;
`PlayerData` unshares 22,060 times a six-game `cube` run against
`PlayerCold`'s 2,880, so a hot-group collection is priced ~7.7x a cold-group
one. What is left in `PlayerData` that owns heap and is not a zone:
`creatures_entered_this_turn` / `_last_turn`,
`creatures_that_damaged_me_this_turn`, `prowl_types_this_turn`,
`graveyard_ids_this_turn`, `spell_casts_this_turn` (a `SmallVec` whose
`extend` is 2.49 M = 0.10 % on its own), `dungeon`,
`pending_creature_etb_counters`, `pending_is_discounts`,
`pending_spell_discounts`. Each is an *empty* `Vec` clone in most games (~10
Ir, no allocation), so none is worth `(-149)`'s size on its own — but
`spell_casts_this_turn` is not empty and is the one to read first.

**FRESH PROFILE AT `3f1296ee`, AND THE SHAPE HAS NOT MOVED.** Taken because
this list had gone thin: `cube` 2,455,809,519 Ir, one thread, seed 1.
`dispatch_triggers_for_events` 5.34 %, `gather_continuous_effects_inner`
3.39 %, `compute_permanent_pass` 3.22 %, `check_state_based_actions_into`
2.93 %, `SpecFromIterNested::from_iter` 2.67 %, `computed_permanent_hinted`
2.40 %, `Arc::clone_from_ref_in` 2.34 %, the allocator cluster ~11 %, and
`__memcpy` 2.28 %. **Three things were priced and none is a lever**:

* **`grow_one` is 2.02 % and it is finished.** 279,584 growths, but over ~100
  callers of which the largest is 37,866 (`Vec::push`'s own std helper, itself
  split across `static_effect_to_effects`, `activate_ability_inner`,
  `declare_attackers_banded` and `run_effect`) and the largest *engine* site
  is `dispatch_board_scan` at 23,576 = **0.115 %**. `(-71)` already took the
  one big one. ⚠ And `with_capacity` is the wrong instinct for most of them:
  a `Vec` that ends at <= 4 elements grows exactly once, so reserving saves
  nothing — the device that pays is `SmallVec`, and only where the buffer
  never escapes.
* **The gather's output `Vec` must NOT be reserved.** 56,814 allocations over
  71,930 gathers is **0.8 per gather**, i.e. most gathers emit nothing and
  `Vec::new()` never allocates. `with_capacity` would make every empty gather
  allocate.
* **`cast_spell_with_convoke` is the largest CoW row — 131,364 `make_mut`
  reaches over 9,856 calls, 40.0 M inclusive = 1.63 %** — and at 305 Ir a
  reach those are real unshares of the zones a cast moves cards between, not
  `(-143)`'s no-op reaches. Lazy copy working as designed; there is nothing to
  guard.

**(-145) ✅ TAKEN by `(-177)` + `(-178)` — `cube` -0.746 % / `fixed` -0.911 % /
`sealed` -0.835 % across the two, against the 0.73 % ceiling priced below —
and NOT by the device this entry feared.** No hand-rolled `Arc`, no
`get_mut_unchecked`: ten lines in `crabomination_base::cow` over std's own
`Arc` — `strong_count == 1`, an `Acquire` fence, `&mut` through `Arc::as_ptr`
— behind one type with its own tests, the `weak_count == 0` invariant under
`debug_assert!` for the sweep, and the Miri run this entry asked for (see the
Log). The trade is wanted: the failure mode needs a `Weak` that nothing in the
workspace can construct.

**(-145) `Arc::make_mut` IS 29 Ir A CALL AND 905,060 OF THEM ARE THE ENGINE'S
CoW WRITE BARRIER — `cube` 26,238,578 self = 1.07 %.** After `(-143)` the
engine has five `CowBox` groups (the card zones, `PlayerData`, `CardData`,
`ColdState`, `ResolutionScratch`) and every `&mut` reach through one pays
`Arc::make_mut`'s uniqueness check as an out-of-line call. The check itself is
std's weak-count dance — `weak.compare_exchange(1, usize::MAX, Acquire, …)`,
the strong load, the release store back — plus call and return. **The engine
never constructs a `Weak` to any of them**, so most of that is a check for a
hazard that cannot occur here.

```text
  cube    make_mut calls   905,060     self 26,238,578    29.0 Ir/call
  fixed   make_mut calls   390,046     self 11,318,472    29.0 Ir/call
  top callers (cube, calls / inclusive Ir)
    131,364  40,056,297  cast_spell_with_convoke      <- 1.63 % of the run
    100,280   6,449,795  resolve_combat
     77,054  10,825,627  resolve_top_of_stack_inner
     62,592   1,836,180  do_untap
     47,372   1,501,452  CardInstance::clear_end_of_turn_effects
```

**Ceiling: a weak-free refcount box brings the check to ~5 instructions, so
~20 Ir x 905 k = 18 M = 0.73 % of `cube`.** That is one of the largest single
numbers left in the profile.

⚠ **AND IT IS DELIBERATELY NOT TAKEN HERE.** The only way to get it is a
hand-rolled `Arc` (`NonNull<Inner<T>>` + `AtomicUsize`, no weak count) or
`Arc::get_mut_unchecked` (unstable): ~60 lines of `unsafe` on the write
barrier of every zone in the engine, where the failure mode is a
use-after-free at game 400 k rather than a slow game. The standing robustness
goal outranks 0.73 %. **Take it only with**: the box behind one type with its
own `#[test]` battery including a Miri run, landed alone, and an explicit
decision that the trade is wanted. `triomphe::Arc` is the same device as a
dependency and would need the same review.

**Refuted here without a build — the deck builder is not a lever, and
`StaticEffect`'s 2,232 bytes are not either.** `size_of` ranks
`CardDefinition` at **8,232** bytes, `StaticEffect` at **2,232** (one variant,
`GrantActivatedAbility { ability: ActivatedAbility }`, and `ActivatedAbility`
alone is 1,960 — its own sibling `GrantActivatedAbilityFromGraveyard` already
boxes it), `ActivatedAbility` 1,960, `PlayerData` 1,016, `TriggeredAbility`
736. Boxing the fat variant would cut the `Vec<StaticAbility>` stride ~4x. But
the whole of deck construction — `--decks sealed --games 1`, which plays no
games — is **11,275,137 Ir**, i.e. **0.45 % of a six-game `sealed` run** and
~2.7 % of one `selfplay_train` game (two decks a game), and `__memcpy` is
15.6 % of *that*. The ceiling on the whole angle is ~0.07 % of a training
game, against 105 call sites, 66 of them in the catalog. **Definitions are
`Arc`d and built once; their size is a cache fact, and Ir cannot see it.**
(`rank_shape` is 29.4 % of those 11.3 M if anyone ever needs the builder to be
faster.)

**(-146)/(-147)/(-148) CLOSED — THE SMALL-TABLE SWEEP IS DONE.** `cube`
**-0.255 %** / `fixed` **-0.307 %** retaken on `0d7aa507`; the -1.080 % /
-0.872 % first reading was against the pre-rebase base, and two of the twelve
tables are inside the group `(-143)` put behind a `CowBox`, so that share of
the win belongs to the other device. **Two devices aimed at the same row do
not add.** Twelve `hashbrown` tables whose size is bounded
by the board, the batch or the seat count became `IdMap`/`IdSet` (a
`Vec<(K, V)>` with linear scans): `GameState::block_map`, eight per-call
locals in `declare_blockers` / `pick_blocks_inner`, two in
`declare_attackers_banded`, one in `simulate_attack_outcome_once`, and the two
per-seat discard maps that were **exactly two `RawTable::clone` calls on every
`GameState` clone**. The cluster is 17 M -> 4.59 M. **What is left is the
cold-group deep copies, and it stops there**: `cycled_counts` is keyed by card
name and grows all game, so a `Vec` would trade a 41-Ir clone for an unbounded
lookup. ⚠ **`SipHash` was never the cost** (`core::hash::sip` is 0.003 % of
the run), which also closes the standing "hash choice for internal maps"
candidate — what a small map costs here is the *table*, not the hash.
⚠ **And deck construction is not a lever either**: `--decks sealed --games 1`
builds twelve sealed decks for 11.3 M Ir total, ~0.9 M a deck against a
~415 M-Ir game, so the two decks a `selfplay_train` actor builds per game are
**0.4 % of it**. `rank_shape` is 29 % of that 0.9 M and PERF already called it
hot-line-free.

**(-140)/(-141)/(-142) CLOSED — THE CLONE-GROUPING DEVICE IS EXHAUSTED.**
Grouping a hot struct's plain field copies *after* the ones that call
something took `cube` **-0.232 %** / `fixed` **-0.308 %** across `GameState`
(a literal), `PlayerData` and `CardData` (declaration order, since they
derive). **What is left is not worth taking**: `PlayerCold`'s 15 fields are
all collections and `ColdState`'s 83 of 84 are, so neither has a run to
group; `CardCold` is 29/7 and is the 0.28 % row's *fourth* contributor. And
`(-141)`'s microbenchmark prices the structural version — hoisting every
scalar into a `#[derive(Clone, Copy)]` sub-struct — at **45 static
instructions over the free reorder** on a 150-field fixture, i.e. a
thousand-site refactor with a `serde` schema change for a third of what the
reorder already got. **The collections are the cost of these clones, and the
only thing that touches them is `(-138)`'s CoW grouping.**

**(-139) CLOSED — DO NOT RETAKE. `noalias` ON `&GameState` IS WORTH NOTHING.**
Seven inline atomics (`rng`, `in_layer_gather`, `layer_freeze`, and
`Battlefield`'s four lanes) cost every `&GameState` and `&Battlefield` in the
engine their `noalias readonly`; boxing all of them restored the attribute —
proved on both sides with the IR probe in the Log — and measured **+0.656 % on
both pools**. `compute_permanent_pass` came out *byte-identical*. The state's
hot data is all behind `Vec`/`Arc`/`CowBox`, so the attribute has almost
nothing to protect. **The same argument closes `&CardData`** (`Definition`
holds `CardMemo`'s two `AtomicU64` inline, so `&CardData` has no `noalias`
either): same shape, same payload-behind-a-pointer, and it would cost an
allocation per CoW unshare. Not worth a build.
**Two numbers to keep from it**: a malloc+free pair is **183 Ir** under the
system allocator in this workload (score allocation trades with it), and the
60-second IR probe recipe is in the Log.

**(-138) THE QUEUE WAS RE-PROFILED FROM SCRATCH AT `bda1a69d` AND THE HONEST
**(-144) CLOSED — TAKEN, `cube` -0.340 % / `fixed` -0.647 % / `sealed`
-0.493 %.** `size_of::<GameState>()` was **3,008 bytes** and two fields that
are `None` on essentially every clone were **1,600 of them**:
`pending_decision` 856 and `suspend_signal` 744. Boxed, the struct is **1,424**
and `__memcpy` falls 29 % on `fixed`. `crate::cow::game_state_stays_small`
caps it at 1,536 bytes so the next inline giant has to bring a reading.
**What is left of the clone is 165 scalar fields in 1,424 bytes and there is
no second `Option` of that size** — `PendingEffectState` 208, `StackItem` 328
and `DecisionAnswer` 48 are all already behind a `Vec` or a `Box`. Do not
retake the size angle without a fresh `size_of` reading.

**(-138) TAKEN BY `(-143)` FOR ITS `GameState::clone` HALF (`cube` -1.274 %
/ `fixed` -1.373 % / `sealed` -0.926 %) — the rest of the re-profile stands.
THE QUEUE WAS RE-PROFILED FROM SCRATCH AT `bda1a69d` AND THE HONEST
HEADLINE IS THAT NOTHING NEW IS BIG.** Fresh `--dump-instr=yes` dumps, both
pools, six games, one thread, seed 1, `profiling-fast --no-default-features`:
`cube` **2,489,474,925**, `fixed` **838,570,681**. (The `cube` figure is
0.54 % under the 2,502,879,696 the previous tip quoted; the card fixes
between them are the only engine difference, so read it as a new base rather
than as a win.)

Everything the four instruments rank splits into three groups, and **only the
third is a candidate**:

* **Diffuse, already line-profiled, refuse to concentrate.**
  `dispatch_triggers_for_events` 5.20 %, `gather_continuous_effects_inner`
  3.34 %, `compute_permanent_pass` 3.17 % (`(-135)` line-profiled it: 102
  lines, none above 0.22 %), `check_state_based_actions_into` 2.89 %
  (`(-128)`), `__memcpy` 2.77 % over 2,319 callers.
* **The allocator, and it is not one row.** `_int_free` 3.20 + `malloc` 2.44 +
  `_int_malloc` 2.23 + `free` 1.99 + `unlink_chunk` 0.23 + `_int_free_merge`
  0.26 = **10.35 %**, over **1,268,320 allocations**. The two tables that rank
  them are both long tails: `__rust_alloc`'s callers are topped by
  `finish_grow` 239,701 (18.9 %) and `computed_permanent_hinted` 201,554
  (15.9 %) and then fall away over 438 more rows; `finish_grow`'s own callers
  are `grow_one` 243,266 spread over **100+ engine sites**, the largest being
  `dispatch_board_scan` at 23,576. `Vec::SpecFromIterNested::from_iter` is
  2.64 % of the run over **395,799 calls from 90+ callers** at 166 Ir each.
  `(-92)`'s "stop looking for a hot line" is the correct reading of all three.
* **`GameState::clone`, which is the one row with a *shape* nobody has
  attacked.** See below.

**`GameState::clone` — 1,175 instructions on EVERY call, 91.5 % of its own
self cost, and the largest reducible fixed cost in the program.**

```text
  calls                      22,558     (cube, six games)
  self Ir                28,965,990     1.16 % of the run
  fixed (cg_arms --sweep)  1,175/call   26,505,650 Ir = 1.065 %
  inclusive                ~49.6 M      1.99 %, ~2,200 Ir a clone
  callees per clone        8.0 Vec::clone, 7.4 memcpy, 3.0 RawTable::clone,
                           2.4 __rust_alloc
  callers   accept_on::{{closure}} 11,764 (52 %) / perform_action 4,988 /
            sim_start_state 2,336 / evaluate_action_sequence 1,422 / ...
```

**Why 1,175 is the number to attack and not the 2,200.** The clone is a
hand-written 200-field literal (`impl Clone for GameState`), and it is 91.5 %
*fixed* because nearly every field is copied unconditionally whether or not it
holds anything: ~40 `Vec`/`HashMap`/`IdMap` fields, of which the callee table
says only **2.4 actually allocate**, i.e. eight in ten of those clones are a
branch on an empty length. The population is one shape — `*_this_resolution`,
`*_scratch`, `pending_*`, `resolving_*`, `sacrificed_*`, `last_*` — about 75
fields that are meaningful only *inside* a resolution, and a probe clones at
priority.

**The device is `cold: CowBox<ColdState>` again, for a second group** (a
`CowBox<ResolutionScratch>`): one `Arc` bump per clone instead of ~100 field
copies and ~29 `Vec`/map clones. The CoW sharp edge is on the write side and
the write side is the resolution, so the win is only real for probes that
never write the group — and **that census is the first build, not the
refactor.** It has now been run (`CRAB_SCRATCH_CENSUS`, `scratch_census` in
`affordances.rs`), six games, one thread, seed 1, and it **settles which
group to build**:

```text
  pool     probes   never wrote the group      never wrote its COLLECTIONS
  fixed     4,766   1,436  30.13 %             4,448  93.33 %
  cube     11,762   3,778  32.12 %             9,500  80.77 %
  sealed   13,348   5,604  41.98 %            11,764  88.13 %
```

**The whole 101-field group is the wrong group and the 29 collection fields
are the right one.** Two thirds of probes write *something* in the wide group
— `finalize_cast` sets `pending_cast_*`, `perform_action_inner` bumps a
counter — so a wide `CowBox` would unshare on two calls in three and pay the
same copy plus an `Arc`. The collection half is untouched by **four probes in
five**, and it is where the cost is: the eight non-inlined `Vec::clone`s, the
three `RawTable::clone`s and most of the 2.4 allocations a clone makes.

**So the entry that is open is narrower than the one it was filed as**: group
the 29 `Vec`/`Option<Vec>`/map/`IdSet` scratch fields — `last_moved_cards`,
`last_created_tokens`, the four `*_card_ids_this_resolution`,
`resolution_targets`, `cost_sacrificed_batch`, `damaged_this_resolution`,
`pending_cast_sacrifices`/`_discards`, `pending_cost_events`,
`pending_permanent_deaths`, `pending_control_changes`,
`resolution_answer_log`, the two per-player discard maps,
`players_sacrificed_this_resolution`, `names_this_resolution`,
`suspend_signal`, `stashed_resolution_answer`, `resolving_source`,
`resolving_spell_snapshot`, … — and leave the ~70 scalars where they are.
**474 references over 13 files**, 377 of them in `game/mod.rs` and
`game/effects/mod.rs`; `suspend_signal` alone is 85 and is control flow, so
it is the field to split out first or leave behind. One commit per field
group, landed fully green or reverted — **never left half-threaded.**

⚠ **The census over-reports in the device's favour and says so**: the
fingerprint is `{:?}` over the group, but the two `fxhash` maps and the
`IdSet` are read by `len()` because their `Debug` order is not stable, so a
probe that swaps a key without changing the count reads as unchanged.

⚠⚠ **AND IT OVER-REPORTED BY 17x FOR A SECOND REASON THE BUILD FOUND — see
`(-143)`.** A value fingerprint cannot see a *reach*: `clear()` on an empty
`Vec`, `mem::take` of one, `.take()` on a `None`, an assignment of an equal
value are all "never wrote the group" to this census and all a deep copy
under a `CowBox`. The device is priced by `reaches x cost(make_mut)`, and the
unguarded build read 437 k reaches on `fixed` against the 10.7 k clones it
saves. Guard every no-op write (`clear_scratch!` and friends) or do not build
the group; and take the denominator off `cg_edges.py --callers make_mut` on
the existing binary before proposing the next one.

**Refuted here without a build: `controlled_by`.** It reads as an obvious
per-clone allocation (`Vec<Option<usize>>`, one entry a seat) and it is not —
`GameState::new` leaves it **empty** and only `control_player_turn`
(Mindslaver) ever `resize`s it, so on a normal game it allocates nothing. The
2.37 allocations a clone are `players` and a small tail.

**Also read and not taken: `dispatch_board_scan`'s two grant lists.**
`DispatchScan { trigger_grants: Vec<_>, equip_grants: Vec<_> }` starts both
empty and pushes; 23,576 `grow_one` calls over 80,222 scans (0.29/call) cost
**2,829,321 Ir inclusive = 0.114 %** on the alloc side, roughly double with
the free. A `SmallVec` would remove them — but `DispatchScan` is returned by
value and its lists are then *moved out* by the caller (`let trigger_grants =
scan.trigger_grants;`), which is exactly the by-value `IntoIter` hazard
`(-106)` measured: a `SmallVec`'s inline buffer is copied on every move, and
`TriggerGrant` carries a whole `SelectionRequirement`. Worth ~0.2 % if the
move can be avoided; **measure the struct's size before assuming the sign.**

**And the gather is still at its floor**, which is worth restating because it
looks like the largest lever and is not: 59,022 gathers at the tip against the
59,002 the ninety-third pass's `cg_contexts` survey called the floor for this
call graph, 152.9 M Ir inclusive = **6.14 %**. The lever there is the cost of
one gather, not the count.

**(-128) STATE-BASED ACTIONS ARE THE LARGEST UNFILED CLUSTER AND THE ONLY TOP
ROW THAT IS *BIGGER* ON `fixed` THAN ON `cube`.** Read off the `8e2d5df9`
dumps (six games, one thread, `profiling-fast --no-default-features`):

```text
                                  cube                      fixed
  check_state_based_actions_into  20,210 c / 72.0 M / 2.85%  9,146 c / 26.2 M / 3.09%
    Ir per sweep                  3,564                      2,862
  sba_board_scan                  20,312 c / 23.7 M / 0.94%  9,146 c /  9.3 M / 1.10%
    Ir per sweep                  1,167                      1,017
  together                                 95.7 M / 3.78%            35.5 M / 4.19%
```

**3,564 Ir is the highest per-call self cost of any engine function in either
table**, and the ratio being *inverted* (4.19 % of `fixed` against 3.78 % of
`cube`) says the cost is per-sweep overhead rather than board size — which is
the shape a memo answers and a board-size optimization does not. Sweeps are
already gated: 20,210 of them against 121,802 `perform_action_inner` calls on
`cube`, one per six or seven actions.

**⚠ THE OBVIOUS DEVICE IS ALREADY IN THE TREE ONE LEVEL DOWN — DO NOT SPEND A
BUILD REDISCOVERING IT.** The first shape this invites is `(-115)`'s: the scan
is a whole-battlefield walk producing a handful of bits, `zone::Battlefield`
already memoizes board predicates against `definition_epoch`, so cache the
scan on the lane. **That is wrong twice.** `CardInstance::sba_scan_bits` is
*already* a single relaxed load off `CardDefinition.memo` — the per-card
definition work the lane would have cached is one atomic load and has been
since the memo landed. And the scan reads instance state the lanes
deliberately survive: `flipped`, `controller != owner`, `attached_to`,
`bestowed`, `soulbond_partner`, `sector`, and a `counters` probe. A counter
lands on a permanent constantly, so a lane-epoch memo of the whole scan would
be both unsound and near-always cold.

**The transferable rule: before proposing a memo, look for it one level down.**
The bits this entry first proposed to cache on the zone were already cached on
the definition, and the note above cost nothing only because it was checked
before the build.

**What the 3,564 Ir a sweep actually is, and where the lever must be.** 1,167
is the scan — a board walk plus seven instance reads a card, with the
definition half already free. The other 2,397 is the body, and a large part of
it is the CR 704.5f/g/h death sweep, which is the rules' own per-sweep
obligation and not overhead. **So the lever is not making a sweep cheaper; it
is running fewer of them.** 20,210 sweeps against 121,802 `perform_action_inner`
calls on `cube` says the engine already gates them; nobody has measured how
many of the survivors follow an action that changed nothing an SBA can read.

**AND THAT NUMBER ALREADY EXISTS — `CRAB_SBA_CENSUS` HAS BEEN IN THE TREE
SINCE `6e44ce7c` AND NOTHING HAD RUN IT AGAINST THIS QUESTION.** It fingerprints
every SBA-relevant field on the board (definition pointer, controller, owner,
tapped, flipped, bestowed, attached_to, sector, damage, both toughness bonuses,
deathtouch flag, soulbond partner, every counter) and counts sweeps whose
fingerprint equals the previous sweep's. Six games, one thread:

```text
  fixed    1,736 / 9,146  repeats  18.98 %
  cube     3,633 / 20,210 repeats  17.98 %
  sealed   5,594 / 31,452 repeats  17.79 %
```

**So the ceiling on "skip a sweep whose inputs did not change" is ~18 % of the
whole 3.78 % / 4.19 % cluster — 0.68 % of `cube` and 0.80 % of `fixed`**, and
it is the entire sweep, not just the scan. That is a better ceiling than most
of the rows in the Log.

**The open problem is the gate, and it is the whole difficulty.** The
fingerprint that measured this is a board walk hashing twelve fields a card —
far more than the sweep it would skip, so it cannot *be* the gate. The obvious
cheap gate is a dirty counter on `zone::Battlefield`'s three `&mut` chokepoints
(`deref_mut` / `iter_mut` / `get_mut`, which the type's own doc says is where
every tap, damage, counter and untap write goes) — sound, because it
over-approximates, but **over-approximating is exactly the risk**: those
accessors fire on any mutable *reach*, not on an SBA-relevant *write*, and the
same doc counts 139,280 of them a `cube` run against 20,210 sweeps. Seven
reaches a sweep against an 18 % unchanged rate says most of those reaches
change nothing the sweep reads.

**THAT COUNTER IS NOW IN THE TREE AND IT KILLED THE DEVICE — NOT ON SIZE, ON
SOUNDNESS.** `zone::BATTLEFIELD_REACHES` (the three chokepoints, `trig-census`)
against the same census:

```text
  pool     sweeps   fingerprint unchanged      after no &mut reach
  fixed     9,146    1,736  18.98 %             946  10.34 %
  cube     20,210    3,633  17.98 %           2,564  12.69 %
  sealed   31,452    5,594  17.79 %           6,036  19.19 %
```

**`sealed`'s no-reach count is LARGER than its unchanged count, and that is the
whole answer.** The two would nest if the battlefield were the only SBA input;
6,036 > 5,594 says 442 sweeps on `sealed` changed SBA-relevant state *with zero
battlefield reaches*. They are the player-side inputs — life totals, poison,
empty-library draws, the Ring's temptations — and the stack (schemes and
phenomena). **A dirty bit on the battlefield alone would skip the sweep that
notices a player at zero life**, which is the one SBA that must never be
skipped.

So a sound gate has to cover battlefield writes *and* every player-side and
stack input, as a new global invariant maintained by hand across the whole
engine, where **missing one write path silently produces a wrong game** rather
than a slow one. For ~12 % of 3.78 % on `cube` — **0.45 %, and 0.43 % on
`fixed`** — against an obligation of that shape. **Recommended against, and
this entry is the reason rather than the arithmetic.**

**The transferable rule: when a candidate's gate has to be maintained by hand
at every write site, price the soundness burden as part of the candidate.** A
0.45 % win that can silently produce a wrong game is not a 0.45 % win. The
`(-115)` lane is the counter-example and the contrast is the point — it is
sound *by construction* because `zone::Battlefield` owns the only routes to a
write, and the reason this one is not is precisely that the SBA reads state
those routes do not cover.

**What is still open here is the 2,397 Ir of sweep body**, of which the CR
704.5f/g/h death sweep is an obligation and the rest is gate evaluation. That
has never been split out. Nothing else in this entry is worth retaking.

Note also that the scan is retaken up to three times *inside* a single sweep
(shapeshifter sync, sector assignment, flip legs), and those retakes are
already inside the 1,167 Ir.

**(-135) CLOSED BEFORE IT WAS OPENED — `compute_permanent_pass` IS THE FIXED-
COST SWEEP'S TOP ROW ON BOTH POOLS AND IT HAS NO HOT LINE.**

`cg_arms.py --sweep` ranks it first at the tip: **165 instructions on every one
of its 269,492 `cube` calls = 44,466,180 Ir, 1.777 % of the run and 56.3 % of
its own self cost** (`fixed`: 182 instructions x 87,750 calls, 1.885 %, 74.9 %).
That is the largest single "paid on every call" figure in the program, so it
was worth the cold `profiling-lines` build the sweep's note asks for before
anything is proposed.

The line profile (`cg_lines.py --in compute_permanent_pass`, same workload)
resolves 102 lines and **not one is above 0.22 % of the run**:

```text
   4,702,442  0.22 %  iter/macros.rs:?     the filter walk over `effects`
   4,372,076  0.20 %  layers.rs:1117       `affects` -> affected_includes_gated
   3,556,684  0.17 %  iter/macros.rs:180
   3,503,396  0.16 %  layers.rs:764        the prologue         13 Ir/call
   3,503,396  0.16 %  layers.rs:1082       `ComputedPermanent { .. }`  13 Ir/call
   1,347,460  0.06 %  layers.rs:1073/1074  the 7c P/T arithmetic  5 Ir/call each
   1,077,968  0.05 %  layers.rs:378/1059/1078   `Printed::new`, set_pt, counters
```

**The 165 are obligatory, and they are four groups**: entering (13), the
per-effect filter walk (~1.9 effects examined per card, and its cost is the
call to `affected_includes_gated`, not the walk), the layer-7 arithmetic that
every permanent needs, and constructing the return value (13). There is no
lazily-skippable arm here — this is the third row in this file, after
`dispatch_triggers_for_events` and the gather, to read as *diffuse* under a
line profile.

**The rule, and it is the sweep's own note earned the hard way: a high fixed
share is a shape question, not a size one.** 100 % of self with a small
instruction count is a frame to delete; 56 % of self across 165 instructions
is a function whose body is straight-line, and straight-line body is the one
thing neither outlining nor inlining touches. **Do not re-open this on the
1.777 %.**

⚠ **CORRECTED BY MEASUREMENT — THE CEILING BELOW IS LOW BY 2.3x AND THE
ARITHMETIC THAT PRODUCED IT IS THE REASON.** The entry prices a row at
`calls x prologue x 2` and says 0.172 % is what the class can pay "with every
frame removed outright". `(-136)` removed five of these frames and measured
`cube` **-0.321 %** — nearly twice the stated ceiling for the whole class —
because **once the fast path is a handful of instructions, LLVM inlines the
function away entirely**, so the saving is the frame *plus* the call, the
return and the argument setup at every site.
`ability_strip_off_battlefield`, `counter_drain_cost` and
`drop_pending_choices_if_game_over` stopped being functions. **The frame
column is a floor on a private function and an estimate on a `pub` one**
(`(-134)`'s cross-crate `has_keyword` stayed a call and landed at 0.215 %
against its 0.23 % prediction). Read the rest of this entry with that
correction; its per-row *ranking* held up, only its ceiling did not.

**(-132) CLOSED WITHOUT A BUILD — THE CLASS IS EXHAUSTED. THE TWO ROWS OVER
0.2 % ARE BOTH REFUTED AND EVERY ROW THAT IS ACTUALLY THE SHAPE IS UNDER
0.06 %.** Re-read at the tip (`cube` 2,502,879,696 Ir) after `(-129)`/`(-131)`
landed, filtering the table below on its own criterion — 1-3 body calls, i.e.
something cold to outline:

```text
                                   ~Ir in frame   share   body calls
  has_keyword                         5,752,488  0.230 %      5   REFUTED, built
  computed_permanent_hinted           4,966,136  0.198 %     83   the frame IS the calls
  ability_strip_off_battlefield       1,463,728  0.058 %      3
  can_grant_keyword                   1,215,536  0.049 %      2
  can_block_attacker_computed           857,332  0.034 %      4
  effect_produced_colors                777,440  0.031 %      2
  CardInstance::toughness               636,720  0.025 %      0   nothing to outline
                                   ------------
  the four that are the shape         4,314,036  0.172 %   if the frame vanished entirely
```

**0.172 % is the ceiling with every frame removed outright, and none of the
four can reach it** — each fast path still needs some of what the prologue
saves. `has_keyword` was built and cost +0.35 % (see the addendum below), and
`computed_permanent_hinted`'s seven pushes hold live values across 83 calls.
**The remaining ~0.7 % in the table is the rows with dozens or hundreds of
body calls** (`dispatch_triggers_for_events` 384, `perform_action_inner` 139,
`static_effect_to_effects` 86), where the frame is the code and there is
nothing cold to move.

**Two things worth keeping from the read.** `effect_produced_colors` has no
jump table at all — LLVM turned its `Seq` recursion into a loop over the tail
and dispatches the rest as a compare chain, so its "2 body calls" are one
self-call in an already-optimal shape. (⚠ **"already-optimal" was wrong**:
`(-136)` put both recursive arms behind an `#[inline(never)]` half and the
row went 2,744,822 -> 1,548,200 Ir. A tail-recursion loop still needs the
callee-saved registers, and 80 % of the asks never enter it.) And
`CardInstance::toughness` has
**zero** body calls with a four-instruction prologue: LLVM wanted the
callee-saved registers for its own arithmetic, which is the column's
`0 means nothing to outline` note, confirmed.

**The rule: filter a class table by its own criterion before quoting its
total.** This one's headline was ~17 M Ir / 0.7 %; the part that matches the
shape it was built to find is 4.3 M / 0.17 %, and the two biggest rows in it
are the two that are not the shape.

---

**The original entry.**

**(-132) THE FRAME CLASS — `cg_frames.py`'S REMAINING ROWS, AND WHICH OF THEM
ARE ACTUALLY THE SHAPE.** `(-129)`/`(-131)` established that a cold path
needing a frame charges every hot call for it, and the instrument that finds
it is committed. The table at `43844faf` (`cube`, six games, one thread),
ranked by `calls x prologue Ir`:

```text
  ~Ir in frame       calls  prol  body calls  insns  function
     5,752,488     410,892     7           5    124  CardInstance::has_keyword
     4,966,136     354,724     7          83   1018  computed_permanent_hinted
     1,463,728     104,552     7           3    125  ability_strip_off_battlefield
     1,330,140      95,010     7          17    153  recycle_events
     1,215,536      86,824     7           2    253  CardDefinition::can_grant_keyword
       857,332      61,238     7           4    354  can_block_attacker_computed
       777,440      77,744     5           2    381  actions::effect_produced_colors
```

**`has_keyword` WAS the largest row and this entry said it was not the shape.
That was wrong, and `(-134)` took it — `cube` -0.215 %, and the row's frame is
gone.** The reasoning here (124 instructions of four linear scans at a 0x28
stride, callee-saved registers holding the loops' induction variables, nothing
cold to outline) is a correct description of the *old body* and never asked
what the scans were for: the exact `Keyword::eq` comparison they exist for is
needed on 4 % of asks, and the other 96 % are answered by a tag test that needs
no call and therefore no frame. See the hundred-and-fourteenth pass. **Read a
frame row as "what does the common *answer* cost", not "what does this function
cost".** What remains of the 40-byte `Keyword` scanned linearly is a separate
candidate and is now much smaller.

**CLOSED at the hundred-and-fourteenth pass — `(-134)` + `(-136)` took `cube`
-0.536 % between them, and the entry's ranking question is now a column rather
than a reading.** `cg_frames.py` prints `out/call` (calls actually made per
invocation, out of the dump's edge records); **`body calls` > 0 with
`out/call` ~ 0 is the shape, proved.** That column immediately settled the two
rows this entry recommended and should not have: `can_grant_keyword` reads
0.88 (67,406 of its 86,824 asks do reach `static_effect_grants_keyword`, so a
split moves the frame rather than removing it) and
`requirement_live_leaves`/`requirement_mentions_power` read 0.54.

What is left on the table, and why neither was taken:

* `can_block_attacker_computed` (4 body calls / 0.00 out, 857,332 Ir) and
  `active_static` (4 / 0.00, 674,520 Ir). **`out/call` ~ 0 is necessary and
  not sufficient** — both put their cold calls inside a `for`/`loop` body, and
  LLVM cannot shrink-wrap a prologue into a loop: the induction variables have
  to survive the call whether it runs or not. A device here has to hoist the
  cold arms out of the loop, which is a rewrite of the matcher, not an
  outlining.
* `ActivatedAbility::is_free` (83 / 0.01, 344,932 Ir) and the four
  `pick_*_response` rows (15-25 / 0.00-0.28, ~1.46 M between them) have their
  calls spread over too many arms for one gate; worth ~0.06 % as a group and
  only if a single early-out turns up.

**Re-run the instrument after anything lands — outlining changes what gets
inlined**, and three of `(-136)`'s five stopped being functions at all.


**ADDENDUM, MEASURED — `has_keyword` WAS BUILT ANYWAY AND THE READING ABOVE IS
RIGHT, WITH A MECHANISM IT DID NOT NAME.** A concurrent session put `#[inline]`
on it off its own sweep and measured `cube` **+0.35 %** (`2,532,782,307`
against `2,529,863,533` at `7244e86d`, in a build that also carried
`(-131)`'s -0.25 %; the two were separated by rebuilding without this half).
Two things happened, neither of them the intended one:

```text
  has_keyword                 410,736 calls — UNMOVED, LLVM still declined it
  has_keyword self Ir          -1,057,680
  KeywordCounters::get         +9,809,688   came OUT of line
```

**`#[inline]` is a request about a function's callers and an unasked-for
decision about its callees.** Making a body available across codegen units
re-runs the inliner on everything inside it, and a callee that was folded in
at one call site is a separate function once there are sixteen. Price an
`#[inline]` on the *callee table*, not on the frame you want to delete. Note
also that measuring two `#[inline]`s in one build read **+0.115 %** and would
have buried `(-131)`'s real -0.249 %: one attribute per build.

**AND THE `release-fast`-ARTIFACT RULE DOES NOT COVER THIS CLASS.** "How to
measure" says a non-generic `crabomination_base` callee reading a million
calls is a profile artifact because `release`'s thin LTO inlines it and
`release-fast` does not. `profiling-lto` — this file's instrument for exactly
that question — leaves `has_keyword`, `compute_permanent_gated` and
`event_matches_spec` **out of line with identical call counts and identical
fixed instruction counts**. Thin LTO declines them. Check the rule per callee.
That run also prices thin LTO itself for the first time: `profiling-lto` runs
`cube` in **2,468,606,469** Ir against `profiling-fast`'s **2,529,863,533**,
**-2.42 %**, both at `7244e86d`.

**A second instrument for the same class**: `cg_arms.py --sweep` ranks every
function by the instructions whose *execution count equals its call count* —
the measured counterpart to `cg_frames.py`'s static prologue count, and it
includes the shared epilogue and the body's unconditional head, which the
disassembly scan does not. Use `cg_frames.py` to find the shape and `--sweep`
to price it, and read a row for what its instructions *are*: 100 % of self
with a small `n` is a frame to delete, a high share on a discriminant
predicate is the predicate (`event_matches_spec`, 24 of 31 Ir a call, is the
kind test it exists to run).


**(-133) `requirement_mentions_power` IS THE FOURTH COPY OF `(-130)`'S WALK,
AT A DIFFERENT CALL SITE.** 44,690 calls at depth 1 and 53,136 at depth 2,
2,638,842 Ir / 0.104 % of `cube`. `(-130)` folded three of these walkers into
`requirement_live_leaves`; this one survives because its caller is
`extract_power_gate`, not `requirement_needs_live_resolution`. The two call
counts are 44,690 and 44,084 — **near enough that they are plausibly walking
the same tree in the same pass**, and if they are, one walk returning a
four-bit mask serves both. Establish that they are the same trees before
building anything: a `--separate-callers=3` context table over both is the
cheap way, and if the callers are unrelated the candidate is worth 0.05 % at
most and should be closed.

**(-129) CLOSED — THE COST WAS THE FRAME, NOT THE MATCH, AND THE CENSUS SAID
SO BEFORE A BUILD.** 31.0 -> 14.7 Ir a call over 1,028,014 calls; `cube`
-0.607 %, `sealed` -0.896 %, `fixed` -0.181 % across `87c8c840` and
`16dd491d`. Full working in the Log's hundred-and-thirteenth pass. What that
entry establishes and this list should not re-take:

* The 10x pool ratio is the **(permanent, trigger) pairs** (3.86x), not the
  batch (1.26x). `ems_census` on `trig-census` prints both.
* **93 % of pairs are dead** — no event in the batch can match the trigger's
  kind — but a per-pair gate in front of the event loop *is* the loop, since
  a dead pair scans the whole batch to find out. Do not re-propose it.
* A per-dispatch memo keyed on the `EventKind` **discriminant** is worth 42 %
  of the kind evaluations (2.15 distinct kinds a dispatch against 3.71
  pairs), and it is *unsound* as written: `CounterAdded(k)`, `StepBegins(s)`,
  `CounterRemoved(k)` and the two `NOrMore` kinds answer per payload and
  share a discriminant. A hand-kept exclusion list for those is `(-128)`'s
  soundness burden again. Closed.
* A static `EventKind -> GameEvent`-variant mask would remove the whole
  93 %, and it is a second hand-written copy of a 150-arm match that must
  agree with the first. Not without a `debug_assert!` audit against the real
  matcher on every call, and at that point price the audit.
* What is left on the function is 14.7 Ir a call of jump table and compare,
  which is the match doing its job.

**(-126) CLOSED — BOTH DEVICES PRICED AND NEITHER IS IN THE TREE. THE
RECURSION IS 42 % OF THE WALKER'S CALLS AND ALMOST NONE OF IT IS REMOVABLE.**

First, the question the entry asked is settled: `--demangle=no` on `cube`
gives **one** hash (`…34evaluate_requirement_static_hinted17he4084a869c4209edE
.llvm.11086542593442499384`) across every context, so the three demangled rows
are one function at three depths and there is no monomorphization to separate.
The ninety-second pass's correction stands and nothing further needs it.

`req_census` (below), six games, one thread:

```text
pool     calls   combinator arms   recursive calls   to a leaf   to a combinator
fixed   165,760      11,864 ( 7.2%)        13,938      12,042      1,896 (13.6%)
cube  1,030,812     437,960 (42.5%)       532,980     393,574    139,406 (26.2%)
sealed  870,956     376,300 (43.2%)       421,718     391,588     30,130 ( 7.1%)
```

**Device 1, a flattened `All`/`Any` slice variant: refuted on ratio without a
build.** It collapses only a combinator whose *child* is another combinator —
**139,406 calls on `cube`, 30,130 on `sealed`, 1,896 on `fixed`.** At the
~25-30 Ir a removed call is worth that is **~0.16 % of `cube`** and under
0.05 % elsewhere, for a `SelectionRequirement` shape change across a
619 k-line catalog and every filter in it. The count that made the device look
attractive (65 % of calls are recursion) is not the count it can serve.

**Device 2, splitting the leaves out of line so the combinator arms stop
paying a hundred-arm prologue: BUILT, MEASURED AND REVERTED.**

```text
                 base 8e2d5df9     leaves out of line      delta
  cube          2,529,860,411          2,553,645,862     +0.940 %
  fixed           847,241,596            853,792,111     +0.773 %
  sealed        2,572,254,610          2,593,298,357     +0.818 %
```

The reasoning was sound and the arithmetic was not. Combinator arms did stop
paying the big prologue — 437,960 of them on `cube` — but **the leaves are the
majority and every one of them now pays a second call**: 592,852 entries on
`cube` reach a leaf, and each goes outer-call, three-arm dispatch, inner-call,
where before it went call, dispatch. The saved prologue is smaller than the
added call because the outer function still carries six arguments and the same
`&self`, so its own frame is not free.

**The transferable rule: a split pays in proportion to the share of calls that
take the *small* half, and here that share is 42 %, not 65 %.** The 65 % figure
counts recursive *edges*; the 42 % counts the nodes that would have benefited.
Read the population of the half you are making cheaper, not the population of
the traffic you noticed.

**What is left on this function is the leaf work itself**, which is a hundred
different questions about a card and not a shape with a device. Do not re-open
it on the call count.

The instrument is `zone::req_census`, on the `trig-census` feature beside the
other two; `bot_ladder` prints a `req_census` line. A **second** device was
built against this same entry by a concurrent session and also reverted — it
follows, and the original entry after it.

**(-126), THE SECOND DEVICE — THE WHOLE SPINE WALKED IN ONE FRAME. BUILT AND
REVERTED, `cube` **+1.218 %** / `fixed` **+1.171 %**. IT REMOVED EXACTLY THE
CALLS IT AIMED AT AND LOST ANYWAY, AND THE MECHANISM IS A RULE THIS FILE DID
NOT HAVE.**

Measured by a concurrent session against the same base (`7244e86d`) as the
entry above; the two priced different devices and both reverted.

The entry asked for the recursive edge to be sized before an `All`/`Any`
slice variant was proposed. It was sized — with a new instrument — and the
answer moved the candidate somewhere else entirely, so the slice was never
built and should not be.

**What the arm census says (`scripts/cg_arms.py`, new this session;
`profiling-fast --no-default-features`, `--games 6 --threads 1 --seed 1`).**
A 205-arm jump-table `match` has one line region and one caller, so no
existing script could say which arms run. This reads the table out of the
binary and the Ir at each target out of a `--dump-instr=yes` dump — an
instruction's Ir *is* its execution count, so the count at an arm's first
instruction is how often that arm was taken.

```text
evaluate_requirement_static_hinted, cube            entries     share
  outermost                                         497,024
  its own recursion ('2)                            533,788     51.8 %
                                                  ---------
  total calls                                     1,030,812

  the second-level dispatch (170 variants merged)    542,592     52.6 %
  And                                                427,428     41.5 %
  ControlledByYou                                     42,196      4.1 %
  Or / Not / ControlledByOpponent / Player            11,246      1.1 %

  fixed per call: 27 instructions run on EVERY call = 27,831,924 Ir
                  = 42.0 % of the function's 66,234,946 self Ir
                  = 1.100 % of the whole cube run
```

**The 27 are the whole prologue and the whole epilogue** — six `push`es, a
408-byte frame, four argument moves, the bounds check and the indirect jump,
then one shared `add`/six `pop`/`ret`. Every one of the function's twenty-seven
hottest *instructions* is one of them. The body is 14,856 bytes; entering and
leaving it is the largest single component of its cost, and **no arm-level
change can touch it — only calling it less often can.**

**So the candidate was rewritten to remove the calls, not the arms.** The
combinator spine was walked in one frame: an explicit `SmallVec` of pending
right-operands, `And`/`Or`/`Not` consumed by a loop, and the 205-arm body
split into `evaluate_requirement_leaf` marked `#[inline(always)]` so the
leaves stay in the same frame. Short-circuiting is identical by construction.
`#[inline(always)]` is load-bearing: out of line the leaves keep paying the 27
and the spine adds a *second* call, which prices at +0.9 M Ir on these counts.

**It did exactly what it was designed to do and lost anyway.**

```text
                        base 7244e86d        walked in-frame       delta
  cube total           2,529,863,533          2,560,664,786      +1.218 %
  fixed total            847,230,007            857,154,183      +1.171 %

  walker calls             1,030,812                497,024      -533,788
  walker self Ir          66,234,946            106,797,856      +40,562,910
  Ir per call                   64.3                  214.9        x3.34
  its closures (unchanged)   394,254 calls @ 30,158,832 Ir — byte-identical
```

**The 533,788 calls went away exactly as predicted and the per-call cost
tripled.** The closure rows are untouched, so the whole delta is the walker's
own frame. Its new prologue is the reason: with the body inlined into a loop,
everything loop-invariant is **hoisted to the function entry**, and the
entry now precomputes the target discriminant, the target's card id, three
`same_team`-shaped comparisons and their negations, and spills them — work
that used to be done lazily by the one arm that needed it, now done by every
call whether or not that arm runs.

**THE RULE, and it is new here: inlining a large `match` body into a loop
converts per-arm lazy work into unconditional per-call work.** The saving is
`(calls removed) x (fixed per call)` and is bounded and knowable — 14.4 M Ir
here. The cost is `(calls kept) x (whatever LLVM finds loop-invariant across
205 arms)` and is neither. **Price the first before the build and treat the
second as unbounded**: a body worth inlining into a loop is one whose arms
share their prologue, and a 205-arm dispatch is the opposite of that.

**Two corrections to this entry's own premises, both from the disassembly.**

(a) The ninety-first pass read the prologue as "13,769,886 Ir / 0.47 % of
`cube`" and called it "the body's callee-saved traffic". It is 19
instructions — six `push`es, the frame, four argument moves, eight of
dispatch — and with the epilogue it is 27 and 1.100 %, not 0.47 %. A prologue
number taken off a line profile is the `push` block only; the dispatch and the
shared epilogue are attributed elsewhere and belong in the same total.

(b) "Six arguments still fit the SysV registers, so bundling the signature
will not move it" is wrong. `Option<CardId>` is passed as *two* 32-bit
registers, so `hint` is the seventh slot and arrives on the stack
(`mov 0x1d0(%rsp),%r13` is the first thing the function does). A bundled
`&Ctx` would remove that load and three spills — **3 Ir of the 27, 0.122 % of
`cube`** — against an unknown number of added loads inside the arms that
currently read those values out of registers. That is not worth a build at
this file's band, and it is the only unexhausted line left on this function.

**What is NOT worth retaking**: the `All`/`Any` slice variant. It removes only
combinator nodes nested under another combinator — 138,444 of the 1,030,812
calls on `cube`, and 78 % of `And` nodes short-circuit on their left operand
so the deep spines are rarely walked to the end. At 27 Ir a call that is
0.148 % *before* the hoisting cost above, for a `SelectionRequirement` shape
change across the catalog and ~25 hand-written walkers of the same tree.

**And a pool note.** The walker is a `cube` cost: 1,030,812 calls there
against **165,760** on `fixed`, where `And` is 5.85 % of calls and the
recursion 8.4 %. CLAUDE.md already says a walker change needs a `cube`
reading; this is the size of that difference.


**(-126) THE REQUIREMENT WALKER IS THE LARGEST ENGINE
FUNCTION IN THE FILE WITH NOTHING FILED AGAINST IT, AND 65 % OF ITS CALLS ARE
ITS OWN RECURSION.**
Fresh dumps at `9f06d76a` (`--games 6 --threads 1 --seed 1`, `profiling-fast
--no-default-features`), folding the three rows as the ninety-second pass's
correction requires:

```text
                                    cube                    fixed
  evaluate_requirement_static_hinted  3.81 %  1,425,066 c    1.44 %+
    of which self-recursive           65.1 %    927,234 c    (And / Or / Not)
  its callees
    bf_hint_or_find                 578,888 c @  17.1 Ir     (the hint hits)
    has_keyword                     136,530 c @  44.9 Ir
    requirement_card_off_battlefield 22,654 c @  91.8 Ir
```

Ranked by folded self cost it is behind only `dispatch_triggers_for_events`
(5.12 %) and the gather (3.29 %), and both of those are worked out: the
dispatch lane is closed and what is left there is a body with no hot line
(pass 91 says so and says not to re-profile it), and the gather's line profile
reads the same.

**What is not the lever: fewer asks.** `(-122)` just refuted a precompute in
front of the second-largest caller, and the caller table has nineteen entries
with a long tail — no single one is worth a device.

**What might be: the 927,234 recursive calls.** `And`/`Or`/`Not` are binary,
so a three-term requirement is two extra calls and two extra dispatches
through a jump table over a large enum. A slice-shaped `All`/`Any` variant
would collapse them, but it is a `SelectionRequirement` shape change across
the catalog and must not be proposed on the call count alone.

**Size the recursion's per-call overhead first, and the instrument matters
twice here.** The prologue alone read **13,769,886 Ir / 0.47 % of `cube`** in
the ninety-first pass's line profile, which is what a six-argument function
with a large body costs to enter — but six arguments still fit the SysV
registers, so that number is the body's callee-saved traffic and not argument
spilling, and bundling the signature will not move it. Get the recursive
edge's own cost with `--separate-callers` and `--demangle=no` (the three rows
are one instantiation at three depths, not three monomorphizations — see the
ninety-second pass's correction) before touching the enum.

**(-127) BUILT, MEASURED AND REVERTED — `cube` +0.727 % / `fixed` +0.994 %.
`players` IS THE ONE ZONE THAT IS NOT `CowBox` BECAUSE 82.5 % OF THE CLONES
WRITE IT, AND THAT IS THE HIGHEST SHARE OF ANY ZONE IN THE STATE.**

```text
                                    base 9f06d76a      wrapped        delta
  cube total                       2,529,866,528  2,548,260,472    +0.727 %
  fixed total                        847,225,763    855,648,798    +0.994 %

  GameState::clone       calls          22,558         22,558            0
                         self Ir    28,965,990     28,069,712     -896,278
  Arc::make_mut          calls         846,772      1,160,176     +313,404
                         self Ir    24,502,124     32,362,326   +7,860,202
  Arc::clone_from_ref_in calls         107,874        126,480      +18,606
  __rust_alloc           calls       1,268,320      1,282,990      +14,670
```

**The census the entry asked for is in the delta, and it is decisive.**
`clone_from_ref_in` rises by **18,606** against 22,558 `GameState` clones:
**82.5 % of clones reach `&mut players` and deep-copy it anyway**, so the
eager `malloc` is deferred rather than removed on all but 3,952 of them. The
clone itself did get 896,278 Ir cheaper — the field's own row behaved exactly
as predicted — and every other row went the other way by twenty times as much.

**Two rules, both general and both about the shape of the cost rather than
this field.**

(a) **A CoW wrap is worth `(clones that never write) x (the eager copy)` minus
`(reaches) x (the gate)`, and the second term is counted per *reach*, not per
clone.** `make_mut` rose by 313,404 against 18,606 copies — **seventeen `&mut
players` reaches per clone that writes one**, each paying an `Arc::get_mut`
refcount load that a bare `Vec` does not have. At ~28 Ir apiece that gate is
7.9 M Ir, 0.31 % of `cube` on its own and more than four times the saving. The
zones this device already wraps are reached once or twice per clone; `players`
is indexed by nearly every arm of `perform_action`.

(b) **"Every other zone is wrapped" is an argument about the code, not about
the population.** The uniformity read as an oversight; it is the measurement.
`players` was the field left bare because it is the one every action touches,
and pass 91's line table — which is where this entry came from — could see the
472 Ir a clone and could not see the 82.5 %.

The change itself is two lines (`CowBox<Vec<Player>>` plus one `CowBox::new`
at the reseat in `assign_teams`); `Deref`/`DerefMut` covered all 1,000-odd
call sites and the workspace compiled with no other edit. **Do not rebuild it
to check** — it is cheap to write and that is exactly why it needs this entry.
 The ninety-first pass's line table has
the number and nothing has been filed against it since: `players:
self.players.clone()` is **472 Ir a clone over 34,522 clones = 16,294,384 Ir,
0.55 % of `cube` and 35 % of `GameState::clone`'s whole inline group.** Every
other zone in `GameState` is `CowBox`-wrapped and clones for a refcount bump;
`players` is a bare `Vec<Player>` and clones for a `malloc` plus two `Arc`
bumps per seat.

The wrap is precedented and mechanical (`CowBox`'s `Deref`/`DerefMut` cover
the call sites, as they did for every other zone). **It is also only a win on
the clones that never write a player, and a loss on the rest** — a clone that
does write pays the refcount bump *and* the unshare's `malloc`, where today it
pays one `malloc`. Almost every action touches a mana pool, a life total or a
hand, so the population is the whole question and the Ir table cannot answer
it: this is allocation-shaped, and pass 54's rule says Ir overstates
allocation under mimalloc.

**Decide it with a counter, not a build.** Two numbers settle it — `GameState`
clones, and how many of them reach `&mut players` before they are dropped —
and `zone::grant_census` is the pattern (four atomics on the `trig-census`
feature, one build, and `(-122)` was answered outright). Do not build the wrap
until the census says the never-writes share is large.

**(-125) NOT A CANDIDATE — A NOTE ON `perms`, AND THE RULE THAT KEPT IT FROM
BECOMING ONE.** `LayerFreezeState::perms` is a
`SmallVec<[(CardId, Arc<ComputedPermanent>); 8]>`, and the encoder's scope
caches ~23 permanents, so it spills and then grows on each of its 6,282 scopes.
That is real. **It is not the explanation of `permanent_value_with`'s 1.29
allocations per call** — the entry above this one, written by a concurrent
session off dumps that already existed, answers that with the memo's overlay
`Box`, and this session's spill hypothesis was reasoned from the inline size
rather than measured. Theirs stands.

**What is worth keeping is why the spill was not filed as a candidate.** Both
fixes are bad trades: raising the inline capacity adds ~384 bytes to
`GameState`, whose clone is 1.23 % of the actor and calls `__memcpy` 218,188
times — `(-74)`'s lever, which did not pay — and reserving at the push is a
loss unconditionally, because a bot scope asks about one or two permanents and
would spill where the inline eight served it. Either is worth **~0.2 % of the
actor**, and `(-123)` measured this workload's ambient codegen band at
**0.18 %**.

**The rule: when the effect you are chasing is smaller than the instrument's
band, change the instrument, not the effect.** On the actor that means pricing
a change on *allocation counts* (`cg_contexts.py __rust_alloc`, before and
after) and treating the Ir total as confirmation at best. Nothing in this file
had said that, because on `bot_ladder` the band is 0.006 % and the question
never came up.

**(-124) REFUTED BEFORE A BUILD WAS SPENT ON IT, BY THE DUMP THAT PROPOSED IT
— AND THE MISTAKE IS `(-82)`'s RULE, VERBATIM.** The entry claimed the encoder
was a caller whose reads are never shared, so that the `Arc` and the freeze
scope's `perms` cache were both pure overhead there and a `_into` form reusing
one buffer would take 145,476 allocations to ~6,282. The reasoning was
arithmetic on a callee table — 23.2 computed permanents a state on a board of
about that many, "so each is asked once" — and it is wrong.

`--separate-callers=2` on the same dump answers it directly:

```text
__rust_alloc by calling context (1,712,610 total)
  213,180  finish_grow <- grow_one                     12.4 %
  161,598  Arc::clone_from_ref_in <- Arc::make_mut      9.4 %   the CoW unshare
   78,900  computed_permanent_hinted <- encode_state_inner
   66,524  computed_permanent_hinted <- bot::permanent_value_with
   44,587  computed_permanent_hinted <- bot::legal_blockers
   33,797  gather_continuous_effects_inner <- computed_permanent_hinted
```

**78,900 allocations on 145,476 calls is a 54 % miss rate, not 100 %.** The
encoder asks through four separate loops (`encode.rs` 278 / 428 / 536 / 1159)
and the scope's memo serves 46 % of them — which is what the comment at
`encode.rs:229` already said and what this entry did not read. So `(-111)`'s
refutation applies here almost exactly: it measured a 57 % miss rate and
priced the `Arc` at 115 Ir a read against 160 for a value handle, and **54 %
is within a point of 57 %**. A `_into` form would buy the misses and lose the
hits, which is the trade `(-111)` already measured and reverted.

**The rule, and it is `(-82)`'s with a new example: "is this already shared?"
is a question for `--separate-callers`, not for arithmetic on a callee count.**
A callee table gives calls; only a context split gives *allocations per
context*, and the ratio between them is the whole answer. `(-82)` sized a
function from outside and got the conclusion backwards; this sized a caller
from outside and did the same. **The dump that suggests a candidate can
usually refute it, and it costs one run with `--separate-callers=2`.**

**What survives**: `computed_permanent_hinted` is still 297,560 allocations
and the largest single caller of `__rust_alloc`, and `(-111)` closed the shape
— "only a change to how often a scope MISSES can move it". The rest of the
allocation table's top is `grow_one` and the CoW unshare, both of which have
their own entries.

**AND `permanent_value_with`'s "more than one allocation per call" IS
EXPLAINED, off the same dumps and with no build spent — it is not that
function's row at all.** `permanent_value_with` has exactly three callees, and
two of them allocate nothing:

```text
callees of permanent_value_with (actor, --separate-callers=3)
  64,481   1,827,137  ManaCost::cmc                  walks a Vec, allocates 0
  60,856     709,625  CardInstance::counter_count    scans a Vec, allocates 0
  60,083  42,578,519  computed_permanent_hinted      <- every allocation

__rust_alloc by context, and ONE context reaches this function:
  64,168  computed_permanent_hinted <- permanent_value_with <- eval_material_inner
```

So the ratio is `computed_permanent_hinted`'s, seen through one caller, and
the excess over 1.0 is **the memo's overlay `Box`**: a miss allocates the
`Arc` *and*, whenever the permanent has a modified characteristic,
`PrintedList::push` allocates the overlay — 4,702 pushes with 4,702
`Keyword::clone`s beside them under `compute_permanent_pass`, against a gap of
~4,085 here. Consistent, and the same overlay `(-111)` priced at "~23 k box
copies over 356 k reads *on top of* the store side".

**The rule, and it is `(-82)`'s and this entry's own restated once more: a
caller's allocation count is its callees', and a function whose callees are
enumerable is refuted by reading them, not by sizing it from the outside.**
Three callees took one `cg_edges.py --callees` and one `cg_contexts.py` on
dumps that already existed. **Nothing here is a candidate**: the row is
`computed_permanent_hinted`'s, `(-111)` closed its shape, and the overlay half
is a *characteristic-modifying board*, not a fixable allocation.

**(-123) TAKEN, 2026-08-31 — THE ACTOR'S PROFILE OF RECORD IS STALE BY
FIFTEEN PASSES AND EVERY CANDIDATE FOR THAT PATH IS READ OFF IT.** The actor
block in "Profile of record" is `bb67895a`. Since then: `(-115)`'s member-list
lane, `(-120)`/`(-121)` (the largest pass in the file), the encoder's v8
rewrite and a vocabulary that went 164 -> 2,273 entries — four changes that all
land on the actor, none of them measured there. `(-107)`'s rows (the encoder's
growths, `computed_permanent_hinted`'s 433,775 `Arc`s) are quoted from that
stale block and are the next candidate in NEXT, so the number they would be
sized against is fifteen passes old.

**`bot_ladder` cannot stand in for it and the file has said so since the
fifty-third pass**: the bench builds its decks once where an actor builds two
per game, and it runs the encoder on no pool at all. This is a re-read, not a
lead — the deliverable is the callee/growth/allocation tables at the current
tip and whatever they promote or refute, which is the only thing that makes
the actor lane's queue current.

Recipe is in "How to measure" (the `crabomination_ml` flag is load-bearing).

**(-122) CLOSED, REFUTED BY CENSUS AT `9f06d76a` — THE MASK COSTS 2.86x THE
WALK IT WOULD REPLACE, AND ON TWO OF THREE POOLS THE WALK DOES NOT EXIST.**
`CRAB_TRIG_CENSUS=1`, six games, one thread, `grant_census` (below):

```text
pool     scans with an EachPermanent grant   mask would evaluate   the walk does
fixed         0 / 11,672                                    0               0
cube      5,030 / 26,540                             301,516         105,436
sealed        0 / 30,536                                    0               0
```

**The cross-check is exact**: the census's 105,436 is callgrind's
`granted_abilities_of_inner -> evaluate_requirement_static_hinted` edge to the
call, 31,641,022 Ir = **1.25 % of `cube`** — so the ceiling this entry asked
to be sized is confirmed at 1.25 %, on one pool of three, and the device
proposed to collect it *spends* 2.86x that.

**Why the premise was wrong, and it is the transferable half.** The entry read
the body — "`for (applies_to, ability, src) in &scan.statics` with an
evaluation per pair, and its own comment says it runs for every untapped
permanent" — and inferred O(permanents x grants). The comment is accurate
about the *sweep*; it is not accurate about what reaches the loop.
`granted_abilities_of`'s ten-check grants-nothing gate stands in front of it
and turns ~65 % of the board away: **21.0 asks per grant-carrying scan against
the 59.9 board slots a mask has to fill anyway.** A board-wide precompute can
only win against a walk that visits the whole board; this one already does
not. **A gate in front of a walk is part of the walk's cost model** — read the
caller's population, not the loop's shape, before pricing a precompute that
covers everything the loop might have been asked.

The `(-115)` device was the right family for a walk with no event kind to
filter on. It is the population, not the device, that refutes it.

**What is left on this traffic is the price per evaluation, not the count.**
31,641,022 Ir over 105,436 evaluations is **300.1 Ir apiece** for what is
usually a type test, and `evaluate_requirement_static_hinted`'s recursive
instance is the top row of the ratio table (1.47 % of `cube` against 0.13 % of
`fixed`, 11.68x). That is a candidate about the *walker*, reached from
nineteen callers of which this is one — not about the grant scan. Nothing is
filed for it yet: it needs its own contexts dump, because the ratio row is the
merged monomorphization and `--demangle=no` is what separates the three.

The census that answered this is `zone::grant_census`, on the existing
`trig-census` feature and gated on the same variable; `bot_ladder` prints a
`grant_census` line beside `trig_census`. It is four counters and it cost one
build. **Re-run it before believing this closure on a pool whose card set has
changed** — `fixed` and `sealed` carry no `GrantActivatedAbility` with an
`EachPermanent` selector *today*.

The original entry, for the record. Read off `p.cube` at
`cab8d5d7` with the ratio device (`cg_ratio.py cube fixed --floor 0.45`),
which ranks it 4th at **2.69x**:

```text
  granted_abilities_of_inner   93,778 calls / 31,641,022 incl / 1.25 % of cube
  its callers                  44,738  effective_mana_abilities_into
                               35,416  bot::available_mana
                               12,958  bot::main_phase_action_with
  the walker it drives         evaluate_requirement_static_hinted, 3 instances,
                               3.81 % of cube against 1.87 % of fixed; the
                               recursive instance alone is 1.47 vs 0.13 —
                               **11.68x, the top row of the whole ratio table**
```

The body is `for (applies_to, ability, src) in &scan.statics` with a
`self.evaluate_requirement_static_on(req, me, …)` per pair — **O(permanents x
grants) `SelectionRequirement` evaluations, and its own comment says it runs
"for every untapped permanent" on the mana sweep.** That is precisely the
shape `statics_granted_triggers_inner` had before this pass.

**The device that fits is `(-115)`'s, not `(-121)`'s**: an activated grant has
no event kind, so there is nothing to pre-filter against a batch. But
`Selector::EachPermanent(req)` is a *board-wide* question being asked one
permanent at a time — evaluate each grant's `req` **once over the board into a
`u64` index mask** when the scan is built, and every later ask is a bit test.
`GrantScan` is already the right place (it has `LANE_ACT_GRANT` and fills it
on the walk it was making anyway), and `(-115)`'s four refutations apply
verbatim to how the mask is *read*, so read them before building.

**Size it before taking it**: the ceiling is the `evaluate_requirement_static_on`
share reached through this caller, not `granted_abilities_of_inner`'s own
1.25 %. Get that with `cg_contexts.py` (`--separate-callers=3`), which nothing
has run on the walker; the entry table above is one level and the walker's own
recursion (533,788 of its 1,425,066 calls are itself) will hide inside it.

**(-120) CLOSED at `dc829911` — `fixed` -2.9565 % / `cube` -0.6237 % /
`sealed` -0.0000 %. See the Log; `fixed`'s is the largest single row in this
file.**

**(-120) TAKEN, 2026-08-30 — THE GRANT-LIVE HALF OF `(-115)`: FOUR GATES MAKE
A DISPATCH GRANT-LIVE AND ONLY ONE OF THEM IS EVENT-KIND FILTERED.**
`(-115)` left `cube`'s 17,000 grant-live dispatches (31 % of asks, 797,574 of
its 1,479,052 visits, on boards twice the size of a no-grant one) as the one
part of the walk nothing has touched. A lane cannot help them *while they are
grant-live* — so the question is how many of them have to be.

`no_grants` is the AND of four facts read at `game/mod.rs:18269-18273`:

```text
  trigger_grants            board GrantTriggeredAbility statics   FILTERED by
                                                                  event kind
  turn_granted_triggers     CR 611.2 turn-scoped watchers         not filtered
  granted_triggers_eot      per-card EOT grants (id-keyed)        not filtered
  equip_grants              equipped_bonus triggers (id-keyed)    not filtered
```

The first has carried an `event_kind_matches` retain since `(-83)`'s window and
the note there says why: a grant whose ability no event in the batch could
match contributes nothing, and finding that out costs one
`SelectionRequirement` per (grant, permanent) pair. **The same argument applies
verbatim to the other three and none of them makes it.** Two of the three are
keyed by `CardId`, so they cannot reach a permanent the caller did not name —
an over-approximating member mask can hold them.

**THE CENSUS ANSWERED IT IN ONE RUN AND THE ANSWER IS NOT A FILTER — IT IS A
DEFECT.** `trig-census`' reason mask at `06e4e878`, six games, one thread:

```text
pool     bucket   dispatches      visits     under the event-kind filter
fixed    none         34,874      85,600     43,598 / 270,426
fixed    equip         8,724     184,826          0 /       0
cube     none         57,788     269,066     74,788 / 797,574
cube     equip        17,000     528,508          0 /       0
sealed   none        103,016     517,614    103,016 / 517,614
```

**Every grant-live dispatch on every pool is the `equip` gate alone**, and not
one of them survives the filter — which is what pointed at the cause.
`EQUIP_TRIGGER_GRANT` is set for *any* `equipped_bonus` that is not
Jitte-shaped, **whether or not it carries a triggered ability**. So a plain
buff Aura or Equipment — a Rancor, a Bonesplitter — pushes `(host, &[])` into
`equip_grants`, `any_equip_grant` reads true, and every dispatch for the rest
of the game gives up its member lane, its fast-path gate and a freeze
push/pop **for an empty list**. `dispatch_board_scan` and
`equip_granted_trigger_sources` both push it, so the `debug_assert!` that
cross-checks them agrees on the wrong answer.

**The fix is the bit, not a filter**: require `!triggered_abilities.is_empty()`
in `dispatch_scan_bits`, and re-ask it at both push sites (the
`STRIP_ATTACHED` half of the scan's gate lets an attachment in for its
`remove_abilities`). No event scan per dispatch, and by the census it takes
grant-live to **zero** on both pools that have it.

**The event-kind retain on the other three gates is therefore NOT taken** —
on this workload it buys nothing the structural fix does not, and it would add
a per-dispatch scan. It stays written down here for the day a pool carries a
real equip trigger: the argument is `trigger_grants`' own, verbatim.

**(-121) — AND THAT DAY WAS THE NEXT HOUR. TAKEN at `0c4d6ece`, `cube`
-1.9733 %.** Re-running the census after `(-120)` landed reads `cube` at
**11,506** grant-live dispatches (411,802 visits, 27.8 % of its walk), still
`filtered 0` — but now the lists are *non-empty*: they are real equip
triggers whose event kind the batch never carries. `fixed` and `sealed` are at
zero and the retain never runs there. See the Log; the transferable half is
**a filter measured worthless behind a defect is not measured at all.**
The `turn_granted_triggers` and `granted_triggers_eot` gates are still
unfiltered and still read *zero* dispatches on all three pools, so there is
nothing to price on them — leave them until a census says otherwise.

**(-120)/(-121) FOOTNOTE — THE HOST-UNION WAS BUILT AND MEASURED AND IS
SUPERSEDED, AND WHICH OF TWO FIXES WINS IS THE RULE.** A third session, working
the same traffic from `(-115)`'s side, unioned the equip grants' **hosts** into
the member word: an equipment grant is keyed by `CardId`, so its reachable set
is a list and the walk can stay a member walk while the dispatch is grant-live.
It works — `cube` **-1.0912 %** against `dc829911`, `fixed` -0.0505, `sealed`
-0.0399, ladder byte-identical, and it took `cube`'s surviving 11,506 equip
dispatches to an 85.7 % lane hit — and it is **not in the tree**, because
`(-121)` then took the same 411,802 visits to *zero* by a more general route.

**The rule: a fix that deletes the cause beats a fix that routes around it, and
the second one is dead code the moment the first lands.** The union only ever
pays where a grant is live *and* id-keyed, and grant-live is now zero on every
pool. Do not rebuild it unless the census shows grant-live dispatches again —
at which point the patch is in this branch's history and the entry above says
what it measured.

**It also corrected `(-120)`'s own closing claim, which is worth keeping.** That
entry read "by the census it takes grant-live to zero on both pools that have
it". It did on `fixed`; on `cube` **11,506 dispatches and 411,802 visits
survived**, real equipment triggers rather than the empty lists the bug pushed.
The census column that predicted zero was the *event-kind filter*'s, not the
structural fix's — two different changes, one table. **Re-run a census after
the fix, not only before it.**

**(-114) BUILT, MEASURED AND REVERTED — THE COLD-GROUP DEREF IS ALREADY
COMMON-SUBEXPRESSION-ELIMINATED, AND `has_keyword` IS **BYTE-IDENTICAL** WITH
AND WITHOUT THE HOIST.** 18,593,922 self Ir over 410,892 calls on both sides,
to the instruction; the whole three-pool delta is **-0.000 %** and every row in
it is libc startup (`__vfscanf_internal`, `strtoul`). At `opt-level = 3` LLVM
sees through `Arc<CardData>` -> `CowBox<CardCold>` -> `Arc<CardCold>` when
nothing between the reads can invalidate it, which on a `&self` method is
always.

**So `(-100)` note (4) does not generalise to the read side, and the reason is
worth keeping**: that note's 2.3 M Ir over 79,496 calls was `make_mut`, one per
cold field written — the *unshare check*, not the deref. A read has no
unshare, so hoisting it saves nothing. **Before hoisting a repeated projection,
ask whether the repetition is a load or a call**: a load is the optimiser's
problem and it has already solved it.

**What the 45.3 Ir actually is, since the deref is not it**: five presence
questions — `removed_keywords_eot`, `removed_keywords`,
`definition.keywords`, `granted_keywords_eot`, `keyword_counters` — each a
pointer+length load and a not-taken branch, and `Keyword::eq` runs only
343,075 times across all 410,892 calls, so the *scans* are not it either.
**The only shape left is a presence bit that says "this instance has no
keyword modification at all"**, letting the function fall straight to
`definition.keywords`. That is four length-checks replaced by one, on the
*instance* rather than the definition — so `CardMemo` cannot hold it and every
write to those four lists has to maintain it. Priced at ~0.3 % of `cube`,
written down rather than taken: `(-87)`'s refutation is precisely about a bit
whose read costs what the walk it replaces did, and four not-taken branches is
the cheapest walk that rule has ever been applied to. **The other lever is the
call count** — 133,214 of them are `evaluate_requirement_static_hinted`
answering a `HasKeyword` requirement, and 116,216 are one closure.

**The entry as it was claimed, kept for the sizing that was right and the
mechanism that was wrong:**

**(-114) `CardInstance::has_keyword` IS 410,892 CALLS AT 45.3 Ir AND IT
DEREFERENCES THE COLD GROUP FOUR TIMES.**

Read off `cg.cube` at `874d6e53`: **410,892 calls / 18,593,922 self Ir /
45.3 a call, 0.71 % of `cube`**, plus `Keyword::eq` at 343,075 calls / 4.47 M
— and the eq count says the *scan* is not the cost. Fewer than one comparison
per call means the lists are empty or match on the first element; the 45 Ir is
the walk to reach them.

```rust
if self.removed_keywords_eot.has_kw(kw) || self.removed_keywords.has_kw(kw) { … }
self.definition.keywords.has_kw(kw)
    || self.granted_keywords_eot.has_kw(kw)
    || self.keyword_counters.get(kw).…
```

**Four of those five fields are `CardCold`**, and `CardInstance` reaches them
through `Arc<CardData>` -> `CowBox<CardCold>` -> `Arc<CardCold>` — a chain
walked once *per field*. This is `(-100)` note (4) exactly, on the read side:
that note priced the *write* version at 2.3 M Ir over 79,496 `make_mut` calls
and said "hoist one `&mut CardCold`". The read version is five times the call
count.

**The fix is two `let`s, no new state and no memo**, so `(-87)`'s refutation
(a presence bit that costs as much as the walk it replaces) does not apply —
nothing is being replaced, only not repeated. Callers, `cube`:
`evaluate_requirement_static_hinted` 133,214 / a closure 116,216 /
`pick_attacks_inner` 44,202 / `is_indestructible` 42,454 /
`pick_blocks_inner` 42,398.

**And the shape is a family**: any `&self` method reading three or more cold
fields pays the same. `has_toxic`, `has_modular` and `is_indestructible` are
the ones next to it.

**(-116) TAKEN at `89917b05` — `fixed` -0.1185 % / `cube` -0.4652 % /
`sealed` -0.3698 %, the pass's largest change and it reorders four guards.**
See the Log; the rule it adds is **rank a chain of pure guards by cost x
rejection rate**, and the entry as claimed is kept there.

**(-119) BUILT, MEASURED AND REVERTED — `fixed` +0.0983 % / `cube` +0.0699 % /
`sealed` +0.0440 %. THE RUNTIME TAG LIST IS BUYING ONE INSTANTIATION OF THE
WALK, AND SEVEN `matches!` CLOSURES BUY SEVEN.** The arithmetic on the row
itself was right and it is not where the answer lives:

```text
cube, base -> the seven-closure build
  board_keyword_in_scope        14,906,578 ->          0   -14.91 M
  board_keyword_matching                 0 ->  9,212,782    +9.21 M
  SmallVec::Extend              15,130,078 -> 13,171,050    -1.96 M
                    the helper and its tag list, net       -7.66 M
  declare_attackers_banded      27,656,068 -> 31,574,748    +3.92 M
  declare_blockers              27,047,108 -> 29,069,004    +2.02 M
  a stack.rs row                   135,828 ->  2,252,048    +2.12 M
  Arc::drop_slow                14,222,866 -> 16,768,658    +2.55 M
  Vec::IntoIter::next              475,338 ->  1,878,335    +1.40 M
  drop_in_place<PlayerData>      2,744,638 ->          0    -2.74 M
                                              program net   +1.81 M
```

**`board_keyword_matching` is generic over `pred`, so a `matches!` closure at
each of seven call sites is seven monomorphizations of a ~23-permanent
three-list board walk.** The wrapper's `SmallVec<[Discriminant; 8]>` — the
69 Ir a call this entry set out to remove — is exactly what keeps that walk to
**one** copy: a `&[Keyword]` is a value, not a type. The drop rows moving
(`PlayerData` to zero, `Arc::drop_slow` up 2.5 M, `IntoIter::next` up 1.4 M)
are the same content coupling `(-109)`/`(-110)` name — the callers were
re-laid-out around seven new inlined copies, and they grew by more than the
helper shrank.

**The rule, and it is the general form of the one this entry got backwards:
a shared walk behind a monomorphic value parameter is a code-size *saving*,
and turning the parameter into a type is a de-optimisation whatever it does to
the parameter's own cost.** Before removing a runtime list in favour of a
`matches!`, count the call sites; at seven copies of a whole-board walk the
list is the cheap side. The doc comment on `board_keyword_in_scope` carries
this now.

**And the row's own history already said so** — the entry did not read it.
That doc records the tag form measured at **-0.584 %** against
`&& *w == k`'s -0.212 % and `has_kw`'s -0.105 %, i.e. three variants of this
exact question were priced at the ninetieth pass and hoisting the short list
*once* won all three. **`(-118)`'s lesson, twice in one session: grep the
entry (and here the doc comment) before costing a row.**

**The entry as it was claimed, kept for the sizing, which was right:**

**(-119) `board_keyword_in_scope` BUILDS A `SmallVec` OF
DISCRIMINANTS ON EVERY CALL AND THEN LINEAR-SCANS IT ONCE PER KEYWORD PER
PERMANENT.** Read off `cg3.cube` at `172a40c8`: **28,510 calls / 14,906,578
self Ir / 523 a call, 0.58 % of `cube`**, plus **1,959,028 Ir (69 a call) in
`SmallVec::extend`** building the tag list.

```text
callers, cube                       calls    Ir (incl)
  declare_attackers_banded          6,538   21,556,375
  declare_blockers                  4,876   17,847,167
  enforce_block_requirements        4,576    2,991,216
  enforce_block_caps                4,576    2,675,400
  pick_attacks_inner                4,526    2,890,126
  do_phasing                        2,772    2,112,278
callees
  frozen_effects                   28,510   33,489,892   <- NOT this entry
  SmallVec::extend                 28,510    1,959,028   <- this entry
```

**What this is NOT**: `frozen_effects`' 33.5 M is the scope's first gather,
which the `compute_permanents` after it would pay anyway — the ninety-first
pass measured hoisting the gate past it at +0.30 %. Leave it alone. **That
half of the entry stands**: the 33.5 M is not addressable, and the remaining
0.58 % is now known to be structural.

**(-118) IS A RE-DERIVATION OF `(-52)`, WHICH CLOSED ACTOR SCALING FORTY-FIVE
PASSES AGO — AND THE PROCESS FAILURE IS THE POINT.** The seed list's "actor
scaling, find contention if sublinear" line was read as an open item off
TODO's NEXT without grepping this file for it; `(-52)` had already run
1/2/3/4/6 actors, found per-actor throughput flat, and written the planning
rule (one actor per core; `--window` is the memory knob, not `--actors`).
**This is the exact failure the Standing rules name for `cg_lines.py` rows,
in a new place: the seed list at the top of this section is not a status,
and an item on it may have been closed by an entry below.** Grep the entry
numbers before spending a run on a seed-list line.

**Three things the re-read does add**, `release` + mimalloc at the
hundred-and-seventh tip, same 4-core 2.80 GHz Xeon:

```text
RUST_MIN_STACK=33554432 CRAB_NO_JITTER=1 target/release/selfplay_train \
  --actors N --games 2000 --steps 1 --seed 7 --out /tmp/scale.N
read the `actors:` line, not `done:` — `--steps` can outlast the actors.

  actors   games/s (two reps)   vs 1 actor    (-52), release-fast, 1200 games
     1     65.9 / 63.0            1.00x        37.2 / 41.1
     2    135.1                   2.09x        79.6 / 80.2
     4    277.2 / 293.3           4.36x       165.2 / 156.2   <- 4 cores
     8    280.6                   4.35x       169.8 / 160.0 at 6 actors
```

(a) **The shape is unchanged and now holds past saturation**: eight actors on
four cores is flat, not worse — the fleet does not thrash when
oversubscribed, which `(-52)` stopped one step short of. (b) **`rows` is
193,736 on every one of the eight runs, at 1, 2, 4 and 8 actors.** The game
index is one `fetch_add` and each game's seed is a pure function of it, so
the *set* of games is actor-count-independent by construction — this measures
it, and it is the cross-process determinism filter applied to the fleet
dimension rather than the thread one. Nothing else in the suite covers that
axis. (c) **The absolute numbers are ~1.75x `(-52)`'s** at every actor count.
**Do not read that as a throughput win**: the profiles differ (`release`
here, `release-fast` there) and the pools and decks have moved over
forty-five passes, so the two columns are not an A/B and nothing in this file
should quote the ratio. It is recorded only so the next reader does not
mistake the new column for a regression against the old one.

**And a sizing rule the run earned.** The first attempt used `--games 120`
and read 51.8 / 168.8 / 292.0 — **3.26x on two actors**, which is impossible.
At 0.4-2.3 s of wall the run is mostly one-time cost. **A scaling curve needs
a workload long enough that the parallel section dominates; a superlinear
step in the shape means the workload is too small, not that the code is
fast.** `(-52)`'s own caution about `--bench --threads N` is the same rule on
the other instrument.

**(-117) TAKEN at `172a40c8` — `fixed` -0.0051 % / `cube` -0.0599 % /
`sealed` -0.0839 %, and it REFUTES THE LINE ROW THAT NAMED IT.** The entry
below priced `block_sides_seen`'s per-(permanent, trigger) construction off
`(-115)`'s `vec/mod.rs:464` row (10,533,512 Ir, 0.40 % of `cube`); removing
the construction and the drop moves `dispatch_triggers_for_events` by
**1,545,810** — a seventh of it. See the Log. **The rule: a `std` line row
inside an inlined-to-death function does not belong to whichever construct in
the source you can see from it.** `vec/mod.rs:464`'s remaining ~9 M is
unattributed and is not a candidate until something names it by removal.

**The entry as it was claimed, kept for the mechanism, which was right:**

**(-117) `block_sides_seen` IS CONSTRUCTED AND DROPPED
ONCE PER (PERMANENT, TRIGGER) AND ONLY FOUR `EventKind`s EVER PUSH TO IT.**
`(-115)`'s reading, item (b): `vec/mod.rs:464` is 10,533,512 Ir (0.40 % of
`cube`) inside `dispatch_triggers_for_events`, and it is
`let mut block_sides_seen: Vec<CardId> = Vec::new();` at the top of the
trigger loop — a construction and a drop on every pair, for a dedupe set that
`Blocks` / `BecomesBlocked` / `BlocksNOrMore` / `BecomesBlockedByNOrMore` are
the only readers of. Hoist it above the battlefield walk and `clear()` per
trigger, `(-71)`'s device.

**(-115)'s MEMBER-LIST LANE IS TAKEN at `ddb9e930` — `fixed` -0.3308 % /
`sealed` -0.1793 % / `cube` **+0.0467 %**, and the positive column is
measured, not conceded.** `Battlefield` carries the dispatch gate's answer
positionally (a `u64` of the indices whose definition has a printed trigger or
a Station band, on the lane word every write path already clears), so a
dispatch with no grant live visits the members instead of the board. Ladder
output byte-identical on all three pools.

**The instrument is the finding: `CRAB_TRIG_CENSUS` (the `trig-census`
feature) prices the lane before and after any change to it.** At `7557554a`,
six games, one thread:

```text
pool     asks    hits        grant-live (board)   board/ask  members/hit   visits left
fixed   34,874  30,676 (87.96 %)  8,724 (21.19)     17.95        0.69      270,426/810,680   33.4 %
cube    57,788  49,218 (85.17 %) 17,000 (31.09)     16.45        2.79      797,574/1,479,052  53.9 %
sealed 103,016  88,364 (85.78 %)      0 ( 0.00)     15.49        3.52      517,614/1,595,290  32.5 %
```

**The lane answers 86 % of asks on every pool — the hit rate is not the
variable.** What separates the pools is the *grant-live* column: 36 % of
`cube`'s whole walk is dispatches with a grant in play, where no member list
can help and the visit still pays the walk's one loop-invariant branch.
`fixed` is 33 % of dispatches but only 23 % of visits; `sealed` is zero.
**So the rule is: a member list is worth its branch in proportion to the
share of the walk that can use it, and that share is a census question, not a
hit-rate question.**

**Four shapes were built and measured on the way to it. Each is a refutation
with a number; do not rebuild them.**

* **One bit walk for both paths — index the miss path too — reads `cube`
  +0.156 %.** It is the obvious simplification (no per-element branch) and it
  loses: random access costs a bounds check and a live index where a slice
  iterator costs neither, and the miss path is where most of the visits are.
* **Hoisting the slice out of the zone to kill that (`let cards: &[_] =
  board;`) makes it worse again**, ~+0.11 % on `cube` on top. LLVM already
  sinks the `Battlefield` -> `CowBox` -> `Vec` deref out of the loop; what the
  hoist adds is a live `(ptr, len)` pair spilling across a 280-line body.
  **A hoist is only a win if the thing hoisted was actually being recomputed.**
* **Carrying the fill's index by hand instead of `enumerate()`** — so a
  grant-live visit does not advance a counter it never reads — reads `fixed`
  +0.069 / `cube` +0.071 / `sealed` +0.028 %. A hand-rolled counter inside a
  conditional is a loop-carried dependency; the iterator's is not.
* **The census, env-gated in the dispatch preamble with the gate never
  taken, is not free**: `fixed` +0.044 / `cube` +0.037 / `sealed` +0.028 %.
  An out-of-line call makes every value the loop keeps live spill around it.
  Moving it past the walk did **not** fix it; compile-time gating did.
  **A never-taken runtime gate in front of a call is not a zero-cost
  instrument in a register-starved loop.**

**And it is not the forty-eighth pass's refuted trigger-carrier mask
(+0.58 %).** That one rebuilt the same bits inside `dispatch_board_scan` on
every dispatch, so the loads it added to one walk paid for the loads it
removed from another — the fusion rule. These bits outlive the dispatch, and
the fill rides the walk a miss was making anyway.

**What is left of the entry: the `cube` column.** The 528 k grant-live visits
are the only part of the walk nothing has touched, they are 36 % of it, and
their board is *twice* the size of a no-grant one (31.09 against 16.45). A
lane cannot help them; what could is the question that makes them grant-live
in the first place — see `(-107)`'s `statics_granted_triggers_on` path.

**(-115) ANSWERED — see the other session's reading below, which is the same
`profiling-lines` build at the same tip (both dumps put dispatch's self at
194,723,966 to the instruction). Two sessions paid ~12 min for it
independently on the same afternoon; the claim line was pushed but the other
half of the protocol — re-read the entry before spending the build — was not
run. **Do not take a third line profile of this function.**

**Two things this session adds to that reading.** (a) **The diffuse shape is
not a property of this function.** `activate_ability_inner` (44.4 M self,
22,826 calls) and `declare_attackers_banded` (27.7 M, 6,538) were line-read
off the same dump and both come out `ptr/non_null.rs` and `iter/macros.rs` to
the top of their tables, with no engine line above 0.09 %. **A diffuse
iteration profile is the normal shape for a whole-board walker here**, so it
is `(-59)`'s "no hot line, the lever is fewer calls" and not a finding.
(b) **The fast-path gate sub-lane is priced and not taken**: 18301 + 18302 +
the station line are ~3.9 M, the two `is_empty()`s are `CardInstance` ->
`Arc<CardData>` -> `Arc<CardDefinition>` -> ptr+len twice, and `CardMemo` has
a free bit pair on the word `dispatch_scan_bits` already reads — so the gate
becomes one word load and a mask, ~3-4 Ir off a visit that costs ~10.
**That caps the per-card-bit version of the lane at ~2.5 M, 0.10 % of
`cube`**, which is `(-114)`'s refutation territory. The `Battlefield`
member-list version is the one that could take the iteration cost with it,
and it is the one that needs new invalidation state.

**The entry as it was claimed, kept for the sizing, which was right:**

**(-115) `dispatch_triggers_for_events` IS 7.46 % OF
`cube` IN *SELF* Ir, 2.3x THE NEXT ROW, AND NOTHING IN THIS FILE HAS EVER
ASKED WHAT THAT SELF COST IS.** Read off `cg.cube` at `2b38c673`:
**139,430 calls / 194,723,966 self Ir / 1,396 a call**, against
`gather_continuous_effects_inner`'s 83.2 M (3.19 %) in second place. Its
callee table sums to ~110 M inclusive, so the self number is the function's
own body — and the body's one loop is `for card in &self.battlefield`, which
every dispatch runs whole.

```text
cube, callers of dispatch_triggers_for_events   139,430 calls
  114,822  perform_action_inner
    6,986  finalize_cast
    2,106  do_untap
    2,074  submit_decision
```

**The arithmetic that makes it a candidate**: ~139 k dispatches x a ~15-card
board is ~2.1 M loop bodies at ~93 Ir apiece, which is the whole self cost to
within the estimate's precision. The loop already has a fast-path `continue`
(`no_grants && no printed trigger && no station`), so most of that is the
*iteration and the two definition loads*, not work the card needed — and
`Battlefield` already carries the shape that answers it (`dispatch_scan_bits`,
`LANE_LISTENER`), as a presence bit rather than a member list.

**Do not design off this paragraph.** The 93 Ir is an estimate from a board
size nobody has measured on this pool, and `(-110)`'s rule is that a program
number is worth nothing until it is attributed to rows.

**THE `profiling-lines` READING THIS ENTRY ASKS FOR IS DONE — the other
session took it at the same tip (its dispatch self reads 194,723,966 to the
instruction, so the two readings are the same workload).** Do not spend the
build again.

```text
cg_lines.py --in dispatch_triggers_for_events, cube, 6 games, at this tip
  18,219,494  game/mod.rs:?          the function's unattributed remainder
  10,615,652  iter/macros.rs:?       }  the `for ev in events` iteration
   6,856,802  iter/macros.rs:180     }  and its inner adapter
  10,533,512  vec/mod.rs:464         `block_sides_seen`'s per-trigger Vec::new
  10,240,198  game/mod.rs:23365      is_event_hardcoded's `match ev`   REFUTED
    8,308,440  ptr/non_null.rs:1720  }
    7,498,218  ptr/non_null.rs:444   }  slice/Vec pointer machinery
    6,705,280  ptr/mut_ptr.rs:?      }
    5,284,716  src/alloc.rs:115      }
    5,089,866  raw_vec/mod.rs:612    }  allocation inside the walk
    4,041,104  game/mod.rs:18449     event_subject
    3,198,948  game/mod.rs:18448     event_matches_spec's call setup
    3,182,554  game/mod.rs:3610      }  dispatch_board_scan, inlined
    1,375,632  game/mod.rs:3487      }
    2,429,028  game/mod.rs:18302     }  the `for e in events` prologue match
    1,478,768  game/mod.rs:18301     }
    2,287,634  game/mod.rs:18431     the `for ev in events` loop head
    2,063,370  game/mod.rs:18315     PermanentEntered's timestamp block
    2,028,064  game/mod.rs:18368     the trigger_grants retain
    1,608,704  game/mod.rs:18434     the dies_suppressed test
```

**AND THE LARGEST NAMED LINE IN IT IS A TRAP — line 23365 IS CLOSED.**
Tabulating `is_event_hardcoded` per event moved the function's own self cost
by **-58,324 Ir** and cost **+0.555 / +0.438 / +0.634 %** in `SmallVec`;
pass 61 had already built and reverted the same idea. See the Log
(hundred-and-sixth pass (3)) and the Standing rules bullet, which now names
the function. **A `cg_lines.py` row on an inlined callee is where its
instructions execute, not a deletion ceiling** — that is the reading this
entry needed and it cuts the other way from the one it expected.

**What the table leaves standing**, in order: (a) the *iteration itself* —
`iter/macros.rs` + the `ptr` rows are ~35 M between them and they are the
`for card in &self.battlefield` and `for ev in events` walks, which is
exactly this entry's thesis; (b) **`vec/mod.rs:464` at 10.5 M, which is
`block_sides_seen: Vec<CardId> = Vec::new()` constructed and dropped once per
(permanent, trigger)** even though only four `EventKind`s ever push to it —
hoist it out of the trigger loop and `clear()` per trigger, or build it only
for those kinds; (c) the `dispatch_board_scan` prologue at ~4.6 M. **(b) is
the cheapest untried thing in the function and it is not a tabulation**, so
`(-110)`'s and pass 61's refutations do not reach it.

NEXT item 2 has been carrying "what dispatch's self cost actually is" as an
open question for two passes; this closes the measurement half of it.

**THE GROWTH HALF IS TAKEN AND SHIPPED at `90a629a1` — `fixed` -0.0935 % /
`cube` -0.0719 % / `sealed` -0.1281 %.** `attacker_infos` and the strike-back
`dealing_blocker_ids` take inline storage (`(-71)`'s device); `resolve_combat`
falls 2.77 -> 2.11 growths a call and ~1 of what is left is the deliberate
`events.reserve(32)`. **The entry's own framing was the wrong test**: a row's
growths-per-call decides whether a *reserve* pays, and neither of these was a
reserve — the question for inline storage is whether the buffer escapes the
frame. See the Log. **What remains open here is nothing**; the row is closed.

**(-113) THE CHEAP HALF IS TAKEN at `9b3470a4`** — `fixed` -0.0458 % /
`cube` -0.0791 % / `sealed` -0.0706 %, and the entry's mechanism for it was
wrong (the `any()` short-circuits on `c.id == attacker`, so the cost was the
find, not the keyword scan). See the Log. **The growth half below is still
open**, and note before taking it that ~1 of the 3.62 growths a call is the
deliberate `events.reserve(32)` from `0367c09c`, which the division folds in
and must not be counted as slack.

**(-113) `resolve_combat` IS NOW THE ACTOR'S LARGEST REAL GROWTH ROW —
22,366 GROWTHS OVER 7,465 CALLS, 3.00 A CALL, ~12.0 M Ir (0.38 %).** Read off
`cg_growth.py` on the actor dump at `d402e5da`, after `(-107)`'s buffer took
`encode_state` from 5.23 growths a call to **exactly 1.00** (33,446 -> 6,390,
and total program growths 386,320 -> 359,264). Every row above
`resolve_combat` in the division is a once-per-process or once-per-game
caller; it is the top row that runs thousands of times.

```text
  actor, --actors 1 --games 60 --steps 1 --seed 7, d402e5da
    growths  calls  per call  name
      2,865     60     47.75  play_recorded_game_mcts   (60 calls — a game)
     22,366  7,465      3.00  resolve_combat            <- this entry
      1,628    585      2.78  deal_damage_to_from
      2,311    881      2.62  mint_token_onto_battlefield
     20,931 13,929      1.50  auto_tap_for_cost_inner   ((-108), still 1.50)
      9,218  6,904      1.34  Iterator::partition       (a generic row)
     10,519  8,768      1.20  deal_combat_damage_to_target
      6,390  6,390      1.00  encode_state              (at its floor)

  split: 16,678 grow_one (3.31 M Ir) + 5,688 do_reserve_and_handle (8.66 M)
```

**`(-112)` listed `resolve_combat` as "reserved (`(-103)`)" and closed the
lane — but it never divided that row by its call count**, and on the actor it
is 3.00, well above the 1.0 floor `(-80)` sets. **The lane is closed for the
rows `(-112)` divided, not for the two it took on trust.** `auto_tap_for_cost_inner`
is the other one, still 1.50 after `(-108)`.

**Where to look first, from the callee table** (the body is thin — the
growths are inlined out of `resolve_combat_damage_with_filter`):

```text
  callees of resolve_combat            calls        Ir (incl)
   98,929   6,063,958  Arc::make_mut                  the CoW unshare
   39,107   4,090,045  __rust_dealloc
   32,210   4,548,475  blockers_of                    141 Ir a call
   32,210   2,507,077  free_division_targets          78 Ir a call, and it
                                                      whole-board scans for a
                                                      keyword no deck has
   24,318  12,338,863  Vec::from_iter
   16,678   3,306,045  grow_one
```

`combat_damage_computed()` returns a `Vec<ComputedPermanent>` per call and
`drop_in_place<ComputedPermanent>` is 37,044 of the callees, so the per-call
board snapshot is a candidate in its own right. **`free_division_targets` is
the cheap half and the obvious one**: it walks the whole computed board asking
whether anything has `DividesCombatDamageAmongDefenders` (Butcher Orgg), 32,210
times, and returns `vec![]` — a `PresenceGate` slot answers it once a freeze
scope. Price it against `(-87)`'s refutation first: the walk is a keyword test
per card, not a length check, so it is on the paying side of that rule.

**(-111) BUILT, MEASURED AND REVERTED — `computed_permanent` BY VALUE IS
`fixed` +0.335 % / `cube` +0.654 % / `sealed` +0.163 %, AND THE REASON IS A
RULE ABOUT WHAT AN `Arc` IS FOR.** The allocation half of the arithmetic below
was right — the allocator family falls **-34.0 M** and `Arc::drop_slow`
another **-7.1 M**, i.e. -41 M against the -42.2 M predicted. Every other
line was wrong.

```text
cube, 381fac97 -> the by-value build
  ComputedPermanent::clone                      0 -> 20,616,270   +20.6 M
  drop_in_place<ComputedPermanent>      3,593,712 -> 22,055,048   +18.5 M
  computed_permanent_hinted            57,159,722 -> 62,444,342    +5.3 M
  Box<[T]>::clone                               0 ->  4,751,512    +4.8 M
  __memcpy                             71,930,348 -> 76,326,556    +4.4 M
  Keyword::clone + its drop + into_boxed_slice                     +3.3 M
  the allocator family + Arc::drop_slow                           -41.1 M
                                                            net   +17.2 M
```

**The rule: an `Arc`'s point is that the drop glue runs ONCE, and a by-value
handle moves a five-field clone *and* a five-field drop onto every read.**
`ComputedPermanent` is 72 bytes but it is not nine words of `memcpy` — it is
`Arc<CardDefinition>` plus four `Option<Box<..>>` overlays, so a clone is a
refcount pair and four branches (and a `Box<[Keyword]>` deep copy whenever an
overlay is set), and the drop is the same again. Measured: **~58 Ir a clone
and ~52 Ir a drop, over 355,838 reads**, against an allocation that was ~203 Ir
on the **57 %** of reads that miss the memo and ~10 Ir on the rest. 0.567 x 203
= 115 Ir a read for the `Arc`; 160 Ir a read for the value. The handle loses on
the hits, which is exactly where a shared handle is supposed to win.

**Two corollaries, so nobody re-derives them.** (a) `(-100)`'s "a deep copy is
~0.44 Ir a byte" prices a *flat* type; for a type whose fields are handles the
per-field logic dominates and the byte count is not the estimator — the
prediction here was off by 6x because it used one. (b) The overlay rate is
higher than the 17 % the `PrintedList::push` census implies: `Box<[T]>::clone`
+ `Keyword::clone` come to ~+8 M, i.e. ~23 k box copies over 356 k reads *on
top of* the store side. **`(-27)`'s pool is refuted, `(-92)` lead 2 is priced
at nearly nothing, and the by-value route is refuted here — the row is closed
as a shape and only a change to how often a scope MISSES can move it
(`(-91)`'s vein).**

**The entry as it was claimed, kept for the arithmetic that failed:**

**(-111) `computed_permanent`'s `Arc` IS 201,820 OF `cube`'s 1,296,530
ALLOCATIONS (15.6 %) AND THE HANDLE IS 72 BYTES.**

Read off `cg.cube` at `381fac97`: `computed_permanent_hinted` is **355,838
calls / 57.16 M self Ir / 160.6 Ir a call**, and **201,820 of those calls go
straight to `__rust_alloc`** — one `Arc<ComputedPermanent>` per memo miss and
per unfrozen read. At ~209 Ir a malloc/free round trip that is **~42 M Ir,
1.61 % of `cube`**, the largest single allocation row left in the program.

**`(-27)`'s pool is refuted and `(-92)` lead 2 (`Arc::new_uninit`) is priced at
nearly nothing — the struct is 72 bytes since `(-101)`, so the copy the
`new_uninit` shape removes is nine words.** The allocation is the cost, not
the copy, and the way to not allocate is to not have an `Arc`: `def` is the
one handle inside, the four characteristics are `Option<Box<..>>` overlays
that are `None` on ~83 % of computed permanents (`PrintedList::push` is 46,304
allocations over 270,098 passes), and a `Clone` is therefore an atomic bump
plus nine words on most of them.

**The arithmetic to check against a build**, `cube`:

```text
  save   201,820 Arc allocations                            -42.2 M
  pay    ~34 k overlay Box clones on the memo store          +7.1 M
  pay    355,838 nine-word copies (misses + hits)            +3.2 M
  pay    355,838 extra `def` refcount bumps                  +0.4 M
                                                     net    ~ -31 M  (-1.2 %)
```

**The refutation to watch for is the one `(-100)` rule (a) predicts**: a
by-value handle turns every *hit* into a copy, and `legal_blockers` /
`AttackerFacts` hold these in `Vec`s whose element grows 8 -> 72 bytes. If the
hit rate is higher than 43 % on another pool, or the block planner's vectors
are longer than the combat sample says, the trade inverts. Ten sites name the
type explicitly; the rest read through `Deref` and do not care.

**(-112) THE RESERVE LANE IS CLOSED — `(-103)`'s DIVISION APPLIED TO EVERY
REMAINING LEADER AND NOTHING IS LEFT ABOVE 1.3 GROWTHS A CALL.** Read at the
hundred-and-fifth pass's tip, `--decks cube`, after `snapshot_payment_state`
(the table's largest context, 1.56 a call) went at `8a59e648`:

```text
  do_reserve_and_handle, 68,261 calls          growths   calls   per call
    Vec::from_iter (nested)                     19,538       —   diffuse, ~93
                                                                 callers ((-96))
    dispatch_triggers_for_events                11,506 139,422    0.08
    auto_tap_for_cost_inner                      8,680       —   TAKEN ((-108))
    mana_source_table                            8,146       —   reserved ((-103))
    resolve_combat                               4,700       —   reserved ((-103))

  grow_one, 255,090 calls
    Vec::push_mut                               37,920       —   the generic row
    dispatch_board_scan                         29,466  80,214    0.37
    resolve_combat                              17,192       —   reserved ((-103))
    grant_scan                                  11,410  26,546    0.43
    affected_from_requirement                   10,434  44,788    0.23
    deal_combat_damage_to_target                 8,608   7,110    1.21  <- only one
```

**Every remaining named context is below one growth a call except
`deal_combat_damage_to_target` at 1.21, and that one is 8,608 growths — a
third of what `snapshot_payment_state` was worth.** Below 1.0 the Vec never
leaves its first allocation on most calls, so `(-80)`'s rule bites: a reserve
**moves** that allocation earlier, it does not remove it, and it charges the
majority of calls that never grew.

**So the allocation mass that is left is FIRST allocations, and a reserve
cannot touch them.** Two shapes can: a different buffer (`(-107)`'s
single-backing-store for `EncodedState`'s eight groups, 66,485 -> 12,660) or
fewer objects (`computed_permanent_hinted`'s `Arc`s — but see `(-111)`, which
built the by-value form and reverted it). **Neither is a reserve, and the
reserve lane should not be re-swept.**

**(-110) THE Ir INSTRUMENT HAS NO NULL CONTROL, AND `(-109)` SAYS IT NEEDS
ONE.** `ab_wall.py` ships a null control for the wall-clock instrument; the Ir
instrument — which every Log row in this file is measured on — has never had
one. `(-109)` established a ~0.7 % floor in `actions.rs` *incidentally*, off
three builds spent on something else, and its rule 2 ("a change to that module
worth less than that cannot be attributed on this instrument") is therefore
resting on a by-product rather than a measurement.

**The device: a change with provably ZERO executed instructions.** Add an
uncalled `pub fn` to `actions.rs` — a lib crate keeps it, the bench never
enters it, so the executed instruction stream is unchanged *by construction*
and **any Ir delta the three pools report is pure instrument noise**. That is
a floor reading that owes nothing to a judgement about what a diff "should"
have cost. If it reads ~0, mass alone is not the mechanism and `(-109)`'s
perturbation needs a real function body — which narrows the mechanism and is
worth the same two builds.

**Then the same null under `profiling-lto`**, which is the one instrument
question `(-109)` leaves open: cgu 16 + thin LTO is buildable here where
`profiling` is not, and if global inlining collapses the null the file gains a
way to attribute sub-floor changes in the oversized modules — the alternative
to splitting them.

**TAKEN AND ANSWERED, 2026-08-30 — and the answer is that THE FLOOR IS NOT
AMBIENT: two nulls read `+0.006 %` at worst.** Both are uncalled `pub fn`s in
`actions.rs`, so the executed instruction stream is unchanged by construction
and every Ir below is instrument noise and nothing else:

```text
base 5ea962ac, callgrind, profiling-fast --no-default-features
                                    fixed      cube      sealed
  null 1  a ~25-line uncalled fn   -0.001 %  -0.000 %   -0.001 %
  null 2  the same, and it also    +0.006 %  +0.001 %   -0.000 %
          instantiates SmallVec<[&ContinuousEffect; 8]>::extend
```

**The instrument now has all three of its noise levels in one place, and they
span five orders of magnitude:**

```text
  same binary, run twice            105 Ir on `fixed`     ~0.1 ppm   ((-102))
  rebuild, zero-content source edit  <=60 ppm, 3 pools    ~6-60 ppm  (here)
  `(-109)`'s claimed ambient floor   7,000 ppm            REFUTED
  wall clock, ab_wall.py quiet       +/-3,400 ppm at best
```

**The middle row is the one an A/B actually needs** and it is the one nobody
had measured: a Log row is always a *rebuild*, never a re-run, so `(-102)`'s
same-binary number was never the right bound to quote and `(-109)`'s guess was
three orders too pessimistic.

**Null 2 is the sharp one**: it is a second user of the *exact*
monomorphization `(-109)` blames for the ±0.7 % swing, referenced from
`actions.rs`, and it moves nothing. **So mass does not perturb this
instrument, and neither does a second instantiation** — the reproducibility of
a zero-content change is about **1 part in 15,000**, not 1 part in 140.

**`(-109)`'s rule 2 is therefore wrong as stated and is corrected below.**
There is no floor to subtract from an `actions.rs` reading. What `(-109)`
actually saw was *content* coupling: its half (b) altered live call sites and
flipped an unrelated monomorphization's inlining. Its rule 1 — attribute a
program number to rows before believing it — is the whole of the lesson and it
stands.

**And correcting rule 2 immediately paid: `(-109)` HALF (a) ALONE HAD NEVER
BEEN MEASURED, AND IT IS A WIN ON ALL THREE POOLS** — see the Log. The entry
read (a)+(b) and (b) alone, declared the pair unattributable, and reverted
both; (a) by itself is **-0.565 / -0.453 / -0.420 %**, which is what the
entry's own two numbers imply if they are differenced. **Never revert a bundle
on a program number: split it and read the halves.**


**(-109) HALF (a) IS TAKEN AT THE HUNDRED-AND-FIFTH PASS
(-0.565 / -0.453 / -0.420 %) AND THIS ENTRY'S HEADLINE CLAIM IS REFUTED BY
`(-110)`'s NULL CONTROLS — read rule 2 below before anything else here.
The entry is kept in full because the way it went wrong is the lesson.**

**(-109, as filed) THE MEASUREMENT FLOOR IN `actions.rs` IS ~0.7 % OF `cube`, AND THIS
ENTRY IS THREE BUILDS SPENT ESTABLISHING IT.** Two changes to the payment
checkpoint were built and reverted, and the reason is the instrument rather
than the code.

**What was built.** (a) `snapshot_payment_state`'s `collect` over a `Filter`
walks the 0->4->8->16 ladder — **17,838 `do_reserve_and_handle` growths over
11,420 snapshots, 1.56 a call, the largest single context in `(-108)`'s
table** — so `Vec::with_capacity(battlefield.len())` + `extend` replaces it
with one allocation. (b) `restore_payment_state`'s two walks each ask the
snapshot about every permanent through a linear `find`, i.e.
O(board x snapshot): **2,036 Ir a call over 2,760 calls**. Sorting the owned
snapshot once (2,760 sorts against 11,420 captures) makes both a binary
search.

**Both are real at the function level and neither survives the program.**

```text
base a019fee3, callgrind, profiling-fast --no-default-features
                              fixed       cube      sealed
  (a) + (b) together        -0.247 %   +0.226 %   -0.313 %
  (b) alone                 +0.317 %   +0.689 %   +0.118 %

  the function rows, (a)+(b), cube:
    restore_payment_state    5,620,000 ->  3,879,746   -1.74 M  (the binary search)
    snapshot_payment_state   1,257,020 ->  4,295,146   +3.04 M  } net -0.85 M,
    Vec::from_iter (nested) 69,007,175 -> 65,119,141   -3.89 M  } the collect moving in
    _int_malloc / realloc / _int_realloc            -6.30 M
  and the row that decides the sign:
    compute_permanent_pass  79,087,884 -> 69,289,222   -9.80 M
    SmallVec::extend        14,210,326 -> 41,161,530  +26.95 M   <- OUTLINED
```

**The two halves are not additive and (b) alone is worse than (a)+(b) on
every pool.** That is the tell: `compute_permanent_pass`'s
`sorted.extend` — `(-106)`'s own shipped shape — is a separate `SmallVec`
`Extend` monomorphization, and a ten-line edit *in another file* flipped the
inliner's decision about it, worth **+17 M net on `cube`, 0.65 %**. The
change's own content is ~2.6 M. **Six times the signal is the perturbation.**

**The push-loop alternative was built too, and it is a genuine pool split
rather than codegen** — the whole delta is `compute_permanent_pass`'s own row
on all three:

```text
  sorted.extend(..) -> for e in .. { sorted.push(e) }
    fixed  -0.028 %   cube  -0.158 %   sealed  +0.062 %
    and the delta IS compute_permanent_pass: -0.12 M / -4.15 M / +1.56 M
```

`SmallVec::extend` fills to capacity through an unsafe loop with no
per-element capacity check, which is worth more than the outlining risk on
`sealed` and less on `cube`. **So there is no version of this that is a win on
three pools, and the `extend` stays.**

**The rules, and they are what the three builds bought.**
1. **Attribute a program number to a function row before believing it.** Every
   reading above is explained by one row; two of the three are explained by a
   row the change does not touch.
2. ~~**At `codegen-units = 16` with no LTO, `actions.rs` has a ~0.7 % noise
   floor on `cube`**~~ — **WRONG, and `(-110)` measured it.** Two null
   controls (uncalled `pub fn`s, zero executed instructions, one of them
   instantiating the very `SmallVec::extend` monomorphization blamed below)
   read **+0.006 % at worst on three pools**. There is no ambient floor: this
   instrument reproduces a zero-content change to about 1 part in 15,000. What
   this entry saw was *content* coupling from half (b), and rule 1 is the
   whole lesson. **Half (a) alone, never measured here, is
   -0.565 / -0.453 / -0.420 % and shipped at the hundred-and-fifth pass** — so
   this rule also cost a real win. Splitting the module remains a build-time
   question and is *not* a measurement one.
3. **A `SmallVec::extend` in a hot frame is a monomorphization the inliner can
   move**, so `(-106)`'s shape carries a 17 M downside that nothing in the
   source hints at. The push loop removes the risk and costs `sealed` 0.06 %;
   priced, not taken.

**(-108) `(-103)`'s DIVISION APPLIES TO EVERY TABLE THAT REACHES
`finish_grow`, NOT JUST `grow_one` — AND THE SECOND TABLE HAD ONE ROW LEFT.
TAKEN AT `a019fee3` FOR -0.336 / -0.288 / -0.356 % AND -0.343 % OF THE
ACTOR.** `do_reserve_and_handle` is the `reserve` / `extend` / `append` path
and 159,482 of `finish_grow`'s 440,455 calls; divided by call count it names
`auto_tap_for_cost_inner` at 1.72 (`cube`) and 1.93 (actor). Log has the
ledger. **`Vec::append` is a reserve site** — it reserves `other.len()` every
time, so an append loop walks the growth ladder exactly as pushes would and
appears in *neither* the `grow_one` table nor any push census.

**What the second table has left, at `927aa505`, `cube`:**

```text
  37,868  Vec::from_iter (nested)          diffuse, ~93 callers ((-96))
  19,306  auto_tap_for_cost_inner          TAKEN -> 8,680
  11,506  dispatch_triggers_for_events     0.14 a call — a first allocation
   8,146  mana_source_table                already reserved at (-103)
   4,704  resolve_combat                   already reserved at (-103)
```

and on the actor `encode_state`'s 66,485 is second only to `Vec::from_iter` —
that one is `(-107)`, and it is a first allocation per group rather than a
re-growth, so a reserve moves it and only a different buffer removes it.

**THE SINGLE-BACKING-BUFFER HALF IS TAKEN AT `d402e5da` — actor
-0.5275 %, a third larger than this entry predicted.** `EncodedState`'s eight
group `Vec`s are one `Vec<EncodedObject>` + `[u32; NUM_GROUPS]` counts,
reserved once. The Log has the row table; the two costs this entry did not
price are **the growth ladder's `__memcpy`** (-3.4 M, the largest single row)
and **drop glue** (`EncodedState::default` + `drop_in_place<TrainRow>`, -540 k,
neither an allocation). **The rule: an allocation-count estimate is a lower
bound on what removing a per-object `Vec` is worth** — count the copy and the
drop as well.

**What is still open here is the THIRD row below**, the `Arc` census —
`(-111)` built and reverted the by-value form of it, so a different shape is
needed, not another attempt at that one.

**(-107) THE ACTOR HAS TWO ALLOCATION ROWS `--bench` CANNOT SEE, AND ONE OF
THEM IS THE SECOND-LARGEST `reserve` GROWER IN THE PROGRAM.** Read off the
actor profile at `bb67895a` (Profile of record). `encode_state` is **66,485
`do_reserve_and_handle` growths over 12,660 encoded states — ~5.25 of the
eight group `Vec`s allocated per state, 28,387,649 Ir inclusive, 0.48 % of the
actor at 427 Ir a growth** — and `bot_ladder` runs the encoder on no pool.
`EncodedState::default` builds `NUM_GROUPS` empty `Vec`s and the reserves then
allocate every non-empty one; the states are retained by the recorder, so a
scratch buffer needs a reuse discipline rather than a swap.

**The only shape that removes them is one backing buffer for all eight groups**
(`Vec<EncodedObject>` plus `[u16; NUM_GROUPS + 1]` offsets) — 12,660
allocations instead of 66,485, ~23 M Ir / **0.39 % of the actor**. That is a
refactor across `crabomination_nn` and the trainer's forward pass
(`trunk1_w.cols == NUM_GROUPS * 2 * obj_hidden + GLOBAL_FEATS` reads the
groups positionally), **priced and deliberately not taken in one sitting**: it
changes no encoded *value* and so needs no retrain, but a half-threaded
version of it would. **Do not change what the encoder emits** — `EncodedState`'s
emitted layout is in the retrain caution — but its *buffers* are free to change.

**`encode_state`'s callee table at the tip, so nobody re-derives it** (12,660
calls, 7,297 Ir of self apiece):

```text
  1,015,070   10,629,038  CardInstance::counter_count      80 a state
    452,509  113,052,080  encode_card_object_into
    186,632   12,917,900  CardInstance::power
    145,010   11,213,746  CardInstance::toughness
     77,342   10,848,943  affordable_covered
     66,485   28,387,649  do_reserve_and_handle            <- this entry
     12,660   58,899,136  with_frozen_layers
     11,892    9,778,684  insertion_sort_shift_left        } counts.sort,
        768    1,041,864  ipnsort                          } 10.8 M / 0.18 %
      7,920      388,300  Vocab::index_of                  memoized on the card
```

The library dedup's `counts.iter_mut().find` does **not** appear as a callee —
it is inlined into the 7,297 Ir of self — and its sort is 0.18 % on its own.
The sort is what makes two shuffles of one library encode identically, so it
is not droppable.

**`encode_state` is now the top removable growth row on the actor.** After
`2e0450f4` the actor's `cg_growth.py` table reads
`play_recorded_game_mcts` 51.28 (6,154 growths, 2.1 M Ir — `opp_hands`' inner
collects, one allocation each) and `encode_state` 5.25 (66,485, 27.5 M), so
this entry is what is left there.

**The second row was the snapshot loop's own copy, CLOSED at `817b9736`
for -1.360 % of the actor** (Log, pass (5)):
`selfplay::play_recorded_game_mcts` kept its de-duplication key as a second
copy of the last pair (`Option<[EncodedState; 2]>`), so every accepted
snapshot cloned two whole states — ~5 allocations and a copy of every encoded
object apiece — purely to keep a comparison alive. The key is now an index
into `snaps`, which already holds that pair. `play_recorded_game_mcts` was the
program's largest single `__memcpy` caller (541,135 calls / 19.1 M Ir), a row
`(-97)` charged to "the MCTS state copy, inherent" — a third of it was this.

**And the third is `computed_permanent_hinted`'s 433,775 `Arc` allocations —
12.8 % of every allocation the actor makes**, against 201,780 on a six-game
`cube` run. `(-92)` lead 2 (`Arc::new_uninit` + unsafe) is the only shape
anyone has proposed for it and `(-27)`'s pool is refuted; it is the largest
untaken allocation row in the program.

**(-105) A MEMO ON A NAMED HELPER HAS A `_mut` TWIN, AND `(-102)` CLOSED ONLY
THE READ HALF. TAKEN AT THE HUNDRED-AND-THIRD PASS FOR -0.119 / -0.195 /
-0.234 % — LARGER THAN THE READ SWEEP THAT PRECEDED IT.**
`GameState::battlefield_find_mut` was `iter_mut().find(|c| c.id == id)` and 73
other engine sites spelled that longhand. `Battlefield::find_by_id_mut`
locates on the shared side and takes `&mut` only on a hit. Log has the rows.

**Two rules, and the second is a refutation with a number.** (a) `(-102)`'s
"grep for the query's longhand" is not finished until the `_mut` spelling is
grepped too — it is a *third* spelling of the same question and the only one
with a write on the end. (b) **The `CowBox` sharp edge is real here and worth
~16 k Ir**: the miss path this removes fires 34 times out of 107,912 deep
copies on `cube`, so the whole win is the memo. That prices the identical
shape over the other CoW zones (11 `hand` sites, 30 `exile`, 3 `command`,
plus `command_card_mut`, which takes `&mut` on every seat's `PlayerData` to
answer one id) at **near zero** — they have no memo to gain and only that miss
path. Refuted without a build; do not re-open it on the sharp edge alone.

**(-106) `compute_permanent_pass`'s `sorted` IS THE PROGRAM'S SINGLE LARGEST
ALLOCATION CONTEXT — 61,518 OF 1,384,794 (4.4 %) ON `cube`.**
**CLOSED at `cfa883ae` for -0.018 / -0.341 / +0.019 %** — the Log entry has
the ledger and the one-character finding (`SmallVec`'s by-value `IntoIter`
carries the inline buffer, and reading the list by value inverts the sign on
two pools). What is left is ~21 Ir a pass of inline-or-spilled branch, which
is what `sealed`'s +0.019 % is; it is not worth a narrower buffer, because
the cost is the branch rather than the width. Read off
`--separate-callers=3` at `ae429ae7`: `finish_grow <- do_reserve_and_handle
<- compute_permanent_pass`, and the only `extend` in that function is
`sorted`. At ~209 Ir a malloc/free round trip on this pool (1,384,794
allocations against a 297 M allocator family) that is ~12.9 M Ir, **0.48 % of
`cube`**, and it is a *pure local* consumed by the loop directly below it —
`(-103)`'s rule (b) shape exactly.

**`(-56b)` refuted four ways of not allocating it and named the fifth**: "the
way to keep both would be a stack-backed `Extend` target (a `SmallVec`), which
is a dependency decision, not a code one." `smallvec` is a dependency now.
**And `(-56b)`'s own reason not to has expired**: it measured against a
`collect()`, whose `Vec::from_iter` takes the internal-iteration path; the
function has since moved to two `extend`s, and `Vec::extend` over a `Filter`
is `extend_desugared` — external iteration already. So a `SmallVec` swap is
apples to apples on the iteration axis and only moves the buffer. Size it at
8 slots (`&ContinuousEffect` is 8 bytes, and `affected_includes_gated` runs
520,142 times over 270,078 passes, i.e. ~1.9 effects considered apiece).

**The next four contexts after it, same dump, so nobody re-derives them:**

```text
  61,518  finish_grow <- do_reserve_and_handle <- compute_permanent_pass
  56,084  finish_grow <- grow_one <- Vec::push_mut          <- see (-104)
  48,216  computed_permanent_hinted <- legal_blockers <- pick_blocks_inner
  46,304  PrintedList<P>::push <- compute_permanent_pass <- ..._gated
  40,374  computed_permanent_hinted <- permanent_value_with <- eval_material
  34,794  clone_from_ref_in <- make_mut <- cast_spell_with_convoke
  29,474  finish_grow <- grow_one <- dispatch_board_scan
```

`computed_permanent_hinted`'s 201,780 are the `Arc<ComputedPermanent>` itself
— `(-92)` lead 2, and `(-27)`'s pool is refuted. `compute_permanent_pass` is
**107,822 allocations between its two rows, 7.8 % of the program's**, which is
the largest single function in the census.

**(-103) RANK A `grow_one` ROW BY GROWTHS PER **CALL**, NOT BY GROWTHS —
TAKEN AT `0367c09c` FOR -0.323 / -0.242 / -0.195 %, AND THE FOUR ROWS THAT
QUALIFIED ARE SPENT.** The Profile of record's caller table is ranked by
growth count, which mixes a *first* push (a reserve **moves** it) with a
*re-growth* through `finish_grow` (a reserve **removes** it). Divide the row
by its call count and only `resolve_combat` (3.5), `pick_attacks_inner` (2.9)
and `mana_source_table` (1.6) were above 1.5 at that tip; `dispatch_board_scan`
(0.37) and the rest of the table are first allocations. Log has the ledger.

**Two follow-ons, and the second is the general one.** (a) Combat's pair only
fell 34,438 -> 25,812 growths, so a `reserve(32)` does not reach every batch
and the residue is *other* `Vec`s inlined into `resolve_combat` — a
`--separate-callers` read would name them, but at ~9 k growths it is worth
~0.02 % and is written down rather than taken. (b) **A pure local of
references takes inline storage, not a reserve**: `SmallVec<[&T; 16]>` costs
stack and zero allocations where a reserve still costs one, and the price is
`SmallVec`'s inline-or-spilled branch per deref (+0.47 M on `cube`, against
the allocator's -6.4 M). That is the rule `(-71)`'s sweep found and this pass
confirms on a *bot* local rather than an engine one.

**The residue, read with `--separate-callers=4` at `0367c09c`** (277,950
growths), which answers follow-on (a) as far as this instrument can: the two
contexts still under `resolve_combat` are `<- advance_step <- pass_priority`
(10,342) and `<- submit_decision <- perform_action_inner` (6,208), i.e. the
same function on two entry paths, not two different `Vec`s a caller table
could separate. Naming the buffer needs source reading, not another dump.

```text
  23,454  push_mut <- activate_ability_inner <- activate_ability
                   <- auto_tap_for_cost_inner        1.03 a call
  22,804  push_mut <- run_effect <- resolve_effect
                   <- continue_ability_resolution_x  0.75 a call
  19,562  dispatch_board_scan <- dispatch_triggers_for_events   0.37
  10,434  affected_from_requirement <- selector_to_affected     0.23
  10,342  resolve_combat <- advance_step <- pass_priority
```

**Both leaders are one allocation per call and neither is a reserve
candidate** — they are the `Result<Vec<GameEvent>, GameError>` these two
frames *return*. `(-104)` is what is left of them.

**(-104) CLOSED AT THE HUNDRED-AND-FOURTH PASS — TWO HALVES TAKEN
(`da4a6ca2` -0.245 / -0.221 / -0.257 %, `381fac97` -0.013 / -0.034 /
-0.061 %), THE THIRD PRICED AND DECLINED.** The splice route was the one the
entry ranked first and it worked: the extra-mana events are buffered and
spliced at `mark`, so the batch order is unchanged and the traces do not move.
Log has both ledgers and the shape rule the declined half produced (**an
out-parameter conversion stops being mechanical once the accumulator is
returned from more than a handful of places** — `continue_spell_resolution` has
22 such sites for ~0.023 % of `cube`). What is left of the family is
`(-107)`'s `encode_state`, which `--bench` cannot see.

**(-104) THE TWO REMAINING GROWTH LEADERS ARE A `Vec<GameEvent>` RETURN, NOT A
MISSING RESERVE — 46,258 ALLOCATIONS, 3.3 % OF EVERY ALLOCATION THE PROGRAM
MAKES.** `resolve_effect` (30,508 calls) and `activate_ability_inner` (22,876)
each open a `Vec<GameEvent>`, hand it up as `Result<Vec<GameEvent>, GameError>`
and have it appended into the caller's buffer and freed. **`run_effect` is
*already* the `&mut Vec<GameEvent>` shape** — the `push_mut <- run_effect`
context is `resolve_effect`'s vector, one frame out, which is why the row
reads 0.75 growths a call. At ~217 Ir an allocate/free pair the ceiling is
**~10 M Ir, 0.38 % of `cube`**, plus the append's `memcpy`.

**Read this before taking it: an out-parameter only *removes* the allocation
where the caller already has an accumulator to append into.** Where the caller
would have to open a fresh `Vec` to pass down, the allocation has *moved*,
which is `(-80)`'s whole lesson in the other direction. So the entry is a
census first: `resolve_effect` has **25** call sites and `activate_ability`
**8**, and the question for each is whether it appends into something it
already holds. The two halves are independent; `check_state_based_actions_into`
and `effective_mana_abilities_into` are the same conversion already shipped and
are the shape to copy. Take it as its own commit with the suite green either
side, or not at all.

**The census is done for the dominant path and it says the allocation is
genuinely removable, three frames down.** 23,474 of `resolve_effect`'s 30,552
calls come through one chain, and its consumer already holds a buffer:

```text
  resolve_effect                                  30,552 calls
    <- continue_ability_resolution_x  23,474   (forwards the Vec unchanged)
         <- activate_ability_inner    21,434   `events.append(&mut resolved)`
    <- continue_trigger_resolution_with_source     4,632
    <- continue_spell_resolution                   2,446
```

**And the friction to plan for is EVENT ORDER, not the borrow checker.**
`activate_ability_inner` does, in this order:

```rust
  if ability.tap_cost {
      self.resolve_extra_mana_on_land_tap(card_id, p, &resolved, &mut events);
  }
  events.append(&mut resolved);
```

so the accumulator ends up `[… tap events][extra-mana events][the ability's
own ManaAdded]`. Resolving straight into `events` puts the ability's own
events *first* and the extra mana after — a different batch order, which
`dispatch_triggers_for_events` then sees, so **golden traces move and the
change stops being behaviour-preserving.** (Whether the current order is even
right is a separate question: CR 605.1b has the triggered mana ability
resolving *after* the tapping one, which is the new order, not the old. Do not
settle that in the same commit as the allocation.)

Three ways out, in the order a taker should price them: **splice** the
extra-mana events in at `mark` (buffer them locally, and only the boards that
actually carry such a grant pay for it — the function's own presence gate
returns before touching anything on the rest); **reorder** deliberately and
justify the trace move as the CR 605.1b fix; or a **reusable scratch buffer**
on `GameState`, which preserves order exactly but adds a `Vec` to a struct
cloned 22,510 times a run and needs `gather_partial`'s take/put discipline
because `activate_ability_inner` can re-enter itself. The borrow itself is the
easy half: the two reads of `resolved`
(`ColorlessManaAdded`-present and the `Mirror` colour) are **loop-invariant
scalars**, so hoisting them above the grant loop makes the whole function need
`&events[mark..]` only once, before its first push.

**(-101) A STORED `fn` POINTER WHOSE VALUE IS FIXED PER FIELD IS TWO COSTS,
AND THE SECOND ONE HAD NEVER BEEN PRICED HERE. TAKEN ON `ComputedPermanent`
AT `de350c5c` FOR -0.481 / -0.394 / -0.497 %; THE DEVICE IS THE ENTRY.**
`Overlay { proj, over }` — `(-70)`'s own fix — stored a
`fn(&CardDefinition) -> &T` per characteristic. Four of them: 32 bytes on a
struct built 289,098 times a `cube` run, **and** an indirect call on every
printed-side read, which is the common one and which nothing in this file had
costed. A `DefProj` type parameter makes the projection a compile-time
constant: the struct is 104 -> 72 bytes and the read is a field offset. The
padding probe splits it — 32 bytes is ~0.16 / 0.23 %, so **the read side was
the larger half on both pools**, and the self table says the fall landed on
the *readers* (`attacker_self_block`, `blocker_pair_block`,
`can_block_attacker_computed`, `pick_blocks_inner`'s closure), not on
`compute_permanent_pass`. Log has the table.

**Where to look for the same shape**: a struct that carries a "how to read
me" alongside "what I hold", where the "how" is fixed at every construction
site. Grep for `: fn(` in a struct definition; the engine has none left on a
hot type after this one, so this is a *rule* for the next struct rather than
an open lead.

**(-102) FIFTY SITES SPELLING `battlefield_find` LONGHAND — TAKEN AT
`c92f3851` FOR -0.047 / -0.078 / -0.057 %, AND THE READING TO KEEP IS THE
NOISE FLOOR.** `(-38)`'s memo shipped on `Battlefield::find_by_id` and the
556 `battlefield_find` sites went through it; fifty others still wrote
`self.battlefield.iter().find(|c| c.id == x)` and scanned. **A repeat run of
the same binary reads 105 Ir apart on `fixed`** (890,025,461 vs 890,025,356),
so a 0.05 % Ir delta is four orders of magnitude above the instrument's own
noise and is worth quoting — which a 0.05 % *wall-clock* delta on this box
never is (`ab_wall.py` resolves +/-0.34 % quiet at best). **The generalisable
half: when a memo lands on a named helper, grep for the query's longhand
before closing the entry.** The same question spelled two ways is two code
paths, and only one of them gets the memo.

**(-100) THE CoW DEEP COPIES SPLIT BY TYPE — `CardData` IS 48 % OF THEM, AND
BOTH HALVES OF ITS COST ARE NOW TAKEN. -0.60 % (width) + -0.95/-1.24 %
(count) AT THE HUNDRED-AND-FIRST PASS; WHAT IS LEFT IS IN THE TABLE.**
`(-99)` read this family by calling context four times and never by type,
because a default callgrind dump cannot show it (see "How to measure":
`--demangle=no`). The split at `cfdf69f2`, `--decks cube`:

```text
  copies      self Ir     share   T
  68,610   50,746,290    1.87 %   CardData
  22,032   19,027,208    0.70 %   PlayerData
  30,100    6,549,548    0.24 %   Vec<CardInstance> (the zones)
   3,312    6,972,236    0.26 %   ColdState
   7,568    3,044,656    0.11 %   CardCold
   2,876      819,660    0.03 %   PlayerCold
   1,682      692,424    0.03 %   CowBox<T>, the rest
```

**Two rules fall out and both are general.** (a) **A deep copy is ~0.44 Ir a
byte of `T` plus its per-field clone logic**, so *width is a lever in
proportion to the copy count* — a byte out of `CardData` is worth 68,610
copies, out of `PlayerData` a third of that and out of `ColdState` a
twentieth. This is why `(-74)`'s padding probe read a real number and its
conclusion ("the lever is fewer deep copies, not smaller ones") does not
generalise: it was true of the ~150 bytes *it* could move. (b) **A field
written on a card the writer changes nothing else on does not belong inside
the group at all** — that is the count half, and `CardInstance`'s six flags
are it.

**What is left, in order.**
1. **`PlayerData`, 1,016 bytes over 22,032 copies (0.70 % self, ~1.05 %
   inclusive).** Same width lever at a third the rate: ~0.010 % of `cube` per
   100 bytes moved. The plausible movers are `dungeon: Option<(String, u8)>`
   (a `String` clone on every copy of a seat that has never entered a
   dungeon — 32 bytes), `pending_creature_etb_counters`,
   `prowl_types_this_turn`, `pending_is_discounts`, `pending_spell_discounts`,
   `creatures_entered_last_turn` — ~150 bytes into `PlayerCold`, so ~0.06 %
   of `cube`, **and the `resolve_combat` trap of (2) applies**: check the
   write frequency of each against `PlayerCold`'s own copy count (2,876)
   before moving it. `spells_cast_by_name_this_game` is the one `RawTable`
   clone in the copy (176 Ir apiece, 3.9 M / 0.14 % of `cube`) and is
   **written once per cast** — moving it buys the 3.9 M and pays a
   `PlayerCold` unshare per `finalize_cast`; that is a near-wash, do not take
   it on the byte count alone.
2. **`CardData`'s remaining width, and it is worth HALF what it was.** The
   count half took its copies **68,610 -> 32,630**, so a byte out of
   `CardData` is now worth ~0.0011 % of `cube`, not 0.0023 %. Everything
   below is priced at the new rate. 568 bytes after the two commits. The
   fattest block left is six `Option<usize>` at 16 bytes each
   (`attached_to_player`, `chosen_player`, `combat_damager_controller`,
   `regeneration_control_grant`, `detained_by`, `protected_by`) — 96 bytes
   for six seat indices. Narrowing the payload to `u32` halves them (48
   bytes, ~0.05 % of `cube` at the new rate) and costs a type change at every
   read; moving the five rare ones to `CardCold` is 80 bytes for no type
   change and no new cold write of any consequence — **and 0.04 %, which is
   why it is written down rather than taken.** **`combat_damager_controller` is not one of
   the five** — it is written on combat damage, which is (2)'s counter-example
   again. After that it is ~30 cast-time rider bools, which a bitfield would
   take from 30 bytes to 4: 0.03 % at the new rate, and a large diff.
   **The copy table at the closing tip, for whoever prices the next one:**
   `PlayerData` 22,032 copies / 19.03 M self (0.71 %) is now the largest,
   `CardData` 32,630 / 16.36 M (0.61 %), `ColdState` 3,312 / 6.86 M (0.26 %),
   `CardCold` 10,280 / 5.52 M (0.21 %) — the group grew by what left
   `CardData`, exactly as intended. `PlayerData`'s copies are **not** the
   handle-field shape: `cast_spell_with_convoke` is 11,290 `make_mut` calls at
   897 Ir apiece (i.e. nearly all copy) and it writes the hand *and* four
   per-turn counters in the same action, so moving the zones out only moves
   the copy to `finalize_cast`. Width is the only lever left there.
3. **`ColdState` at 2,192 bytes is the widest type in the program and the
   cheapest to leave alone**: 3,312 copies means a byte is worth 1,457 Ir over
   the whole run. Do not spend a pass on its width. Its *count* is `(-50)`'s
   closed vein.
4. **The Cleanup sweep's own derefs.** Moving six fields into `CardCold` at
   (1) added **79,496 `make_mut` calls at 29 Ir = 2.3 M Ir on `cube`**
   (0.085 %): `clear_end_of_turn_effects` expands `eot_wear_off!` into one
   `self.$f = …` per field and each cold field re-derefs the group. The fix
   is to hoist one `&mut CardCold` for the cold arms, which means splitting
   the macro's list in two — and the whole point of that macro is that the
   clear and its guard cannot diverge, so a split has to keep generating both
   from one source. Priced, not built.

**(-99) IS CLOSED BY `(-100)`, AND THE WAY IT CLOSED IS THE LESSON: NEITHER
OF ITS TWO DIRECTIONS WAS THE ANSWER.** The entry's warning ("do not open it
without a day for it") was right about the family and wrong about the axis.
Direction 1 (a no-op-write audit) it closed itself; direction 2 (a narrower
probe clone) was never built and is now moot — the copies are not too many,
they are of a *type*, and both halves of that type's cost came out with
mechanical field moves once the dump was read with `--demangle=no`. **A
family read four times by one lens is not mined; it is read four times by
one lens.** Kept below in full because its context table is still the right
map of *who* pays.

**(-99) THE 132,370 CoW DEEP COPIES, ATTRIBUTED BY CALLING CONTEXT FOR THE
FIRST TIME — TWO CONTEXTS ARE 42 % OF THEM.** `(-1)`/`(-14)` have been read
four times by *call count of `make_mut`*; nobody had asked which caller's
`&mut` reach actually pays the copy. `--separate-callers=3` +
`cg_contexts.py clone_from_ref_in`, `--decks cube`, `633acc3e`+ (the run reads
2,716,755,506 Ir, so the contexts cost nothing):

```text
  141,690 calls to clone_from_ref_in, by context
   31,838  22.5 %  make_mut <- activate_ability_inner <- activate_ability
   27,566  19.5 %  make_mut <- cast_spell_with_convoke <- cast_spell
   16,870  11.9 %  make_mut <- declare_attackers_banded <- perform_action_inner
    8,174   5.8 %  make_mut <- declare_blockers <- perform_action_inner
    6,810   4.8 %  make_mut <- do_untap <- advance_step
    5,430   3.8 %  make_mut <- on_left_battlefield <- remove_..._to_graveyard_raw
    4,664   3.3 %  make_mut <- resolve_top_of_stack_inner <- resolve_top_of_stack
    2,554   1.8 %  make_mut <- cast_spell_with_convoke <- perform_action_inner
   … 10,386 calls in 86 further contexts
```

**`activate_ability_inner`'s copy RATE is the number to look at, not its
count: 31,838 copies out of 49,240 `make_mut` calls is 64.7 %**, against
`resolve_combat`'s 100,452 calls at 33 Ir apiece (i.e. almost none). A caller
that copies two times in three is one whose handle is **freshly shared every
time it is reached** — which is what the bot's per-candidate `GameState::clone`
does, so the first write after each clone must copy and the copy is the price
of the search, not a bug.

**That is why this entry is a measurement and not yet a candidate.** Two
directions it opens, in order, and both are design work rather than a row:

1. **Which of those first writes are no-op writes — MEASURED AND CLOSED, no
   source read spent.** `cg_sites.py deref_mut` on a `profiling-lines` build
   at the same tip: the whole `deref_mut` family is **11,794,986 Ir / 0.51 %**
   and its largest single site is `stack.rs:6090` at **0.07 %**; after that,
   `Battlefield::cards_unchecked_mut` 0.04 %, `iter_mut` 0.03 %, and a tail.
   **Neither of the two hot contexts above appears in the table at all** — so
   the copies they pay do not come from a few identifiable `DerefMut` lines
   that could carry a no-op guard, they are spread over many distinct write
   sites inside two ~3,000-line functions. A site-by-site no-op audit has no
   target; `(-50)`'s question is answered "nowhere concentrated" here.
2. **Whether the bot's clone can be made narrower than the whole state** for
   the probe paths specifically — `(-13)`'s and `(-27)`'s territory, and both
   have refutations on file for the *pooling* form of it, not for a narrower
   clone.

So **direction 2 is the only one left**, and the entry stands as a warning as
much as a lead: **do not open it without a day for it.** Every previous pass
into this family that took less came back with a refutation, and direction 1
is now a third.

**(-98) `cg_sites.py battlefield_find` RE-READ AFTER THE MEMO — 61.5 M /
2.35 % BEFORE, 45,813,904 / 1.99 % NOW, AND THE RESIDUE IS TWO SITES.**
`profiling-lines`, `--decks cube`, `--dump-instr=yes`, at `633acc3e`
(2,716,751,269 Ir against `profiling-fast`'s 2,717,721,142 — 0.036 % apart, so
the attribution transfers). Read as a floor, per the tool's own rule.

```text
  6,427,518  0.28 %  mod.rs:23087   find_card_anywhere's first leg
  6,276,282  0.27 %  eval.rs:3144   bf_hint_or_find's fallback
  3,208,280  0.14 %  bot.rs:10018   pick_blocks_inner's closure
  1,990,344  0.09 %  combat.rs:2490 first_strike_damage_step
  … then 60+ sites under 0.08 % each
```

**Both leaders are misses by construction and neither is a candidate.**
`find_card_anywhere` asks about cards that are mostly *not* on the battlefield
— that is what the function is for — so its first leg is a full scan every
time and the memo cannot help it; its visit order was already tuned (the
comment records the cast path's libraries-first version at ~65 % of the
function). `eval.rs:3144` is the requirement walker's *unhinted* remainder,
i.e. exactly the calls the ninety-fifth pass's `_on` conversion could not
reach. **The 556-site class is now a long tail with no head**: after the two
leaders, no site is above 0.14 %, which is the same shape `(-96)` found at the
function level. `(-38)` is closed and this is the last reading it needs.

**(-97) THE ACTOR'S OWN PROFILE, RE-READ AT `633acc3e` WITH `(-95)`'s LENS
APPLIED, AND IT CONFIRMS "AN ENGINE PERCENT IS AN ACTOR PERCENT" WITH
NUMBERS.** `selfplay_train --actors 1 --games 60/120 --steps 1 --seed 7`,
`profiling-fast -p crabomination_ml --no-default-features` (0 `libmimalloc`
frames in the dump — the flag has to name `crabomination_ml`, see "How to
measure"). Two run lengths so the once-per-process block can be separated:
3,308,033,402 Ir at 60 games, 6,148,474,954 at 120 (x1.859).

**The startup block is 83 rows / 57,253,725 Ir = 1.73 % of the 60-game run and
0 % of a training run**, and it is candle's, not the engine's:

```text
                                 60 games      120 games   growth
  rand_distr Normal::sample       722,816 ->    722,816    x1.00  35.3 M, 1.07 %
  candle impl_avx2                 23,099 ->     23,099    x1.00  12.8 M
  rand refill_wide                 23,099 ->     23,099    x1.00   0.4 M
```

Net weight initialisation, charged whole to whatever games the run plays. **A
60-game actor profile therefore reads `Normal::sample` at 1.07 % and a real
run reads it at nothing** — the same trap `(-95)` found in `bot_ladder`, in a
different crate, and the same `--games`-doubling diff finds it.

**The per-game table, at 120 games so the block is already halved. Thirteen of
the top fifteen rows are engine:**

```text
  374,722,938  6.09 %  dispatch_triggers_for_events
  308,368,422  5.02 %  __memcpy_avx_unaligned_erms
  223,754,938  3.64 %  _int_free          }
  223,610,691  3.64 %  clone_from_ref_in  }  the allocator family is
  202,102,545  3.29 %  _int_malloc        }  11.9 % here against 11.1 %
  169,155,800  2.75 %  malloc             }  on `bot_ladder`'s cube
  158,048,663  2.57 %  check_state_based_actions_into
  135,489,984  2.20 %  free
  127,359,932  2.07 %  Vec::from_iter
  127,171,806  2.07 %  gather_continuous_effects_inner
  118,601,411  1.93 %  activate_ability_inner
  108,092,295  1.76 %  compute_permanent_pass
  102,372,062  1.66 %  computed_permanent_hinted
   93,895,348  1.53 %  encode_state             <- actor-only
   89,840,110  1.46 %  encode_card_object_into  <- actor-only
   66,637,955  1.08 %  rank_shape               <- actor-only
```

**The only actor-only rows above 1 % are the encoder (2.99 % between its two)
and `rank_shape` (1.08 %)** — 4.07 % in total, against ~40 % of the run in
engine rows above them. The standing claim is now measured, not asserted.

**The one row that is genuinely different from `bot_ladder` is `__memcpy`:
5.02 % here against 2.73 % on `cube`. It is not a lever.** 4,070,536 call
edges over ~355 callers; the top three are `play_recorded_game_mcts` (539,847,
13.3 % — the MCTS state copy, inherent), `Arc::clone_from_ref_in` (526,472)
and `GameState::clone` (432,864), i.e. the CoW/clone family `(-1)`/`(-14)`
already own. **The actor copies more because it searches, not because it
encodes**, and no single caller is worth a pass.

**(-96) THE PROFILE RE-READ AT `633acc3e`, ALL THREE POOLS, AND WHAT IS
ACTUALLY LEFT.** Six levers landed since the last whole-program read (three
lanes, the listener lane, the shared reader, the find memo), so this is the
replenishment the list was down to. `cube` self costs, with each row's status
so nobody re-derives one:

```text
  191,671,398  7.05 %  dispatch_triggers_for_events   (-59)/(-90): no hot line,
                                                      mask ceiling 0.86 %
  ~301,000,000 11.1 %  the allocator family           1,426,345 allocs; UPPER
   (malloc/_int_malloc/free/_int_free/shims)          BOUND — callgrind runs
                                                      the system allocator and
                                                      the shipped build is
                                                      mimalloc
  115,723,654  4.26 %  Arc::make_mut's deep copies    132,370 of 865,758
                       (inclusive)                    make_mut calls actually
                                                      copy — 15.3 %. (-1)/(-14),
                                                      mined across four passes
   89,015,336  3.28 %  clone_from_ref_in (self)       same family
   82,622,298  3.04 %  gather_continuous_effects_inner
   75,148,674  2.77 %  compute_permanent_pass         (-92) lead 2: needs
                                                      Arc::new_uninit + unsafe
   71,707,978  2.64 %  check_state_based_actions_into (-69) refutation, read it
   69,528,097  2.56 %  Vec::from_iter (SpecFromIter-  407,863 call edges over
                       Nested), 185 Ir/call            ~93 callers — DIFFUSE
   59,684,668  2.20 %  computed_permanent_hinted      355,828 calls, 201,780
                                                      allocs; (-27)'s pool is
                                                      refuted
   45,301,768  1.67 %  activate_ability_inner
```

**Nothing concentrated is left, and that is the finding.** The two rows a
reader would reach for first are both already-mined families with their
refutations on file, the third is the allocator (whose number is an upper
bound this instrument cannot tighten), and the fourth — `Vec::from_iter` at
2.56 % — is 93 callers with no single one above 78 M *inclusive*. The last six
passes each took 0.3-0.9 % out of rows that were 1-4 % and concentrated in one
or two call sites; there is no such row now.

**So the next pass should change instrument, not target.** Three that have
never been run here, cheapest first: (a) `--games 12` call-count diffs on all
three pools, which `(-95)` just used to refute a candidate and which nothing
else in this file has ever applied; (b) `cg_sites.py` on a `profiling-lines`
build for a *second* always-inlined needle now that `battlefield_find` is
closed — `computed_permanent`, `evaluate_requirement_static_hinted`'s hint
path and `CardInstance::has_keyword` (411,884 calls, 45.2 Ir) are the
candidates; (c) the actor's own profile, which is where the workload actually
is and which has been read once.

**(-95, EXTENDED) THE LENS RUN ON ALL THREE POOLS AT `ca2bf1a6`, AND THE
INSTRUMENT NEEDS ONE CORRECTION: THE PROGRAM'S OWN GROWTH IS POOL-SPECIFIC
AND ONE POOL IS SUPERLINEAR.** `--games 6` -> `--games 12` doubles the games
exactly (24 -> 48, 48 -> 96, 72 -> 144) and the program does not double:

```text
                        fixed           cube            sealed
  program              x2.490          x1.666          x2.023
  perform_action_inner  x2.32           x1.85           x1.97
  dispatch_triggers..    x2.31           x1.84           x1.97
  resolve_combat         x2.46           x1.82           x1.94
  gather_..._inner       x2.34           x1.77           x1.93
  compute_permanent_pass x2.62           x1.70           x2.05
```

**The engine rows move with the program, so this is the workload, not a fixed
cost**: on `fixed` the second 24 games are *longer* than the first 24, on
`cube` the second 48 are shorter. So `--games 6` and `--games 12` on one pool
are the same *configuration* at two lengths but not the same workload per
game, and the lens's threshold must be read against the pool's own engine
growth (2.3 / 1.8 / 2.0), never against 2. At x1.0-1.15 the classification is
unambiguous either way.

**The saturating share, measured on all three for the first time:**

```text
  fixed    583 rows   2,520,782 Ir   0.28 % of the six-game run
  cube     939 rows  15,195,643 Ir   0.56 %
  sealed   788 rows   9,992,784 Ir   0.37 %
```

so the six-game workload's once-per-process contamination is **under 0.6 % on
every pool**, and `cube` — the worst — is half what it read before the last
two passes' levers landed. Every other row in the profile of record is a
genuine per-game cost on every pool.

**Two rows worth naming.** `String as fmt::Write::write_str` is 4.44 M /
0.164 % of `cube` at **x1.13** — the `(-86)` / `wants_converge` family,
confirmed saturating rather than argued to be. And **`recommend::rank_shape`
is 684 calls at exactly x1.00 on `sealed`**: the ladder builds its sealed
decks once per process, so its 3.3 M / 0.123 % there is a fixed cost — while
`(-97)` prices the same function at **0.98 % of the actor**, which builds two
decks a game. **A deck-builder row read off `bot_ladder` is a floor and a
misleading one**; read it off `selfplay_train`. That is the one place "an
engine percent is an actor percent" does not hold.

**(-95) THE SIX-GAME WORKLOAD CHARGES EVERY ONCE-PER-PROCESS COST TO SIX
GAMES. One row is big enough to matter, and it is REFUTED BY MEASUREMENT:
`CardDefinition::wants_converge`'s `format!`, 0.459 % of `cube` and ~0 % of a
training run.** It renders the whole definition with `{self:?}` and substring-
searches for `ConvergedValue` / `ManaValueAtMostConverged` — 217 calls,
**12,468,533 Ir inclusive, 57 k Ir apiece**, the largest single *concentrated*
source line left in the profile and the only `fmt::format` caller in the
engine's hot path. It is also the only one whose call count **saturates**:

```text
--decks cube, profiling-fast, /tmp/base3, one binary, two run lengths
                        --games 6         --games 12        growth
  program            2,717,721,142     4,529,284,272        x1.667
  wants_converge's
    format! calls              217               242        x1.12
    Ir                  12,468,533        13,916,258        x1.12
    share of program        0.459 %           0.307 %       and falling
  effect_short_text          612 calls       1,606 calls    x2.62 (per-game)
```

**The cache is process-wide** (a `OnceLock<RwLock<HashMap<String,bool>>>`
behind a thread-local L1), so the `format!` runs once per *card name per
process*, bounded by the catalog. Six games see 217 distinct names; a 30 k-game
gate sees a few thousand, once, and the row is gone. **Do not spend a build on
it**, and do not replace the Debug-string oracle with a structural walk either
— its doc comment records that the hand-written walker it replaced "had
already rotted in both directions", and a scanning `fmt::Write` sink swaps one
short memcpy per chunk for two substring searches per chunk, which is very
likely a loss.

**THE INSTRUMENT IS THE TRANSFERABLE PART: run the same pool at `--games 12`
and diff the CALL COUNTS.** A row whose calls grow x1.0-1.15 while the program
grows x1.67 is a once-per-process cost and its share of a training run is ~0;
rank it at its 12-game share or lower, never at its 6-game one. **And the
sweep bounds the damage: 138 rows are saturating and they are 27,399,327 Ir
between them, 1.01 % of the six-game `cube` run** — so every other row in this
file's profile of record is a genuine per-game cost and the instrument stands.
`scripts/cg_calls.py` on the two dumps is the whole recipe.


**(-94) THE FIND MEMO'S OWN CODE SIZE — BUILT, MEASURED AND REVERTED at
`fixed` +0.209 % / `cube` +0.270 % / `sealed` +0.315 %.** The residue
`(-38)`'s memo left was `bf_hint_or_find` and `battlefield_find` de-inlining
at some sites (net +1.6 / +2.1 / +3.5 M against the caller rows they left),
and the obvious fix was to put the scan behind `#[inline(never)]` so
`find_by_id`'s inlined half is *smaller* than the `iter().find()` it replaced.
**The re-inlining happens and the change still loses**, on all three pools:

```text
base 633acc3e, callgrind, profiling-fast --no-default-features
  fixed     911,042,520 ->   912,950,419   +0.209 %
  cube    2,717,721,142 -> 2,725,065,287   +0.270 %
  sealed  2,700,016,601 -> 2,708,531,049   +0.315 %

  bf_hint_or_find      10,146,168 ->  4,337,936   the re-inlining is real
  find_card_anywhere    9,431,130 ->  3,982,242
  declare_blockers     27,655,192 -> 23,906,972
  resolve_combat       28,134,550 -> 24,526,900
  find_by_id_scan               0 -> 47,316,688   286,320 calls, 165 Ir each
```

**The miss path is half the traffic, and out of line it costs 165 Ir where
inlined it cost ~58.** `find_by_id_scan` is called **286,320 times on a
six-game `cube` run**; the memo's own win (-12.17 M) over a ~50 Ir saving per
hit puts the hit count at about the same number, so **the hit rate is near one
half** and a call on the miss path is paid as often as the hit is collected.
That is the number this A/B was actually worth: it prices the class.

**A SECOND SLOT IS ALSO REFUTED, same sitting, and it says what the misses
are.** Packing two indices into the same `AtomicU32` (a two-deep FIFO, so an
alternating source/target pair both stay hot — which is what
`resolve_combat`'s scope looked like) reads `fixed` **-0.103 %** / `cube`
**+0.005 %** / `sealed` **-0.008 %** off `633acc3e`: a pool split with zero on
the two pools the training loop plays, so a revert under the standing rule.
**The second slot buying nothing means the remaining misses are not an
alternating pair — they are genuinely distinct ids**, which is what a board
walk asking about each permanent in turn looks like, and no depth of
same-collection memo reaches those. One entry is the whole class.

**The rule: an out-of-line miss path is priced at the miss rate, and a memo
whose hit rate is one half has no cheap miss path.** The four rows that
re-inline give back 18.8 M on `cube` and `find_by_id_scan` takes 47.3 M — the
call frame, the lost per-site specialization of the loop, and the `Option`
returned through a register pair rather than folded into the caller's control
flow. `(-38)`'s entry is CLOSED by this: the memo ships as measured, code size
and all.


**(-93) THE CALLER-FILLED LANE — a `zone::Battlefield` lane whose miss costs
NOTHING. Ten lanes now; the hundredth pass took five
(`LANE_SHIELD` x2, `LANE_GRANT`, `LANE_LISTENER`, `LANE_ACT_GRANT`), window
`840b5ccd -> 98feda21` **-0.888 / -1.088 / -1.137 %** and then a further
-0.278 / -0.075 / -0.115 % over the two after it. **The three split lanes
share one `split_lane(shift, audit, what)` reader** (`5be9908c`, Ir identity
to 726 Ir on 2.7 G), so a fourth costs a shift constant, a predicate and two
one-line accessors. The
closure form (`lane(shift, predicate)`) walks the board itself on a miss, so
it can only be pointed at a walk that *is* a predicate, and it breaks even at
a hit rate of about one half — which is where the `DerefMut` invalidation
actually sits (48.1 / 38.3 / 32.2 %, `(-87)`). The split form
(`dispatch_lane() -> Result<bool, u64>` + `store_dispatch(epoch, found)`)
removes that constraint entirely: the caller walks, the caller stores, and a
miss costs the walk it was making anyway. **Any whole-board walk whose answer
is a pure function of the definitions can use it, whatever else the walk
produces.** Taken twice at the ninety-ninth pass (`fixed` -0.869 / -0.328 %,
`sealed` -0.693 / -0.375 %); both Log blocks carry the numbers.

**Three conditions, and the last two are what refute candidates.** The walk's
per-card body must be more than a length check (`cast_lock_scan`'s mask is
refuted at +0.034 % on exactly that); nothing cheaper may already gate it; and
**the fill must be free — the walk has to already ask the lane's question.**
The third is `fire_step_triggers` below, built and reverted at
`fixed` -0.221 / `cube` +0.013 / `sealed` +0.053 %: adding one `matches!` per
printed ability to fill the lane costs more on a board where the lane answers
`PRESENT` than the skip saves on a board where it does not, and the boards
where it answers `PRESENT` are `cube` and `sealed`. A fourth, learned here:
**a second consumer of an existing lane is
priced on the lane's ANSWER, not on the row it gates** — where the answer is
usually `PRESENT` it buys only the misses the first consumer would have paid,
which is why `trigger_grant_sources` reads `cube` +1.2 % on its own row and
the program still falls.

**AND THE HIT RATE IS ALREADY IN EVERY DUMP YOU HAVE — measure it before you
build.** For a walk that calls something only on the cards its lane predicate
selects, `callee calls / walk calls` is the mean number of selected cards a
board carries, and a lane buys the asks where that is zero. `LANE_GRANT` read
**0.70 / 1.15 / 0.017** (`can_grant_keyword` over `keyword_grant_in_scope`
asks) and then landed **-0.481 / -0.276 / -0.621 %**, in exactly the order the
ratio predicted. `cg_calls.py` on a dump you already have answers this in
seconds; the reverted `fire_step_triggers` lane is what happens when nobody
asks.

**LANES 11-16 ARE FREE AND A SEVENTEENTH IS NOT.** `type_gates` is a `u32` at
two bits a lane and ten are used (`LANE_ACT_GRANT` is at shift 18), so the next
six cost nothing but a shift constant. Past sixteen the word has to widen.

**The remaining consumers, sized off the `fixed` tip profile at `841c7b9b`,
cheapest first. One is built and reverted, one is taken; the other two are
not built. THREE MORE WERE FOUND OFF THE SELF TABLE RATHER THAN OFF THIS
LIST AND ALL THREE ARE TAKEN**: `keyword_grant_in_scope`'s battlefield leg
(`98feda21`, -0.481 / -0.276 / -0.621 %),
`fire_combat_damage_triggers`' phase-1.5/1.6 listener walk (`623e935a`,
-0.127 / -0.040 / -0.010 %) and `grant_scan`'s `GrantActivatedAbility` walk
(`ca2bf1a6`, -0.151 / -0.035 / -0.105 %). **That is the lesson: this list was
seeded from one function's callee table, and the biggest consumers were not in
that function.** The method that found all three is `cg_edges.py --rows 40` on
the self table, read for the word *walk* — any `battlefield.iter()` whose
per-card body is a definition read is a candidate, and its hit rate is the
callee ratio above.

1. **`fire_step_triggers`' first battlefield loop — BUILT, MEASURED AND
   REVERTED, `fixed` **-0.221 %** / `cube` **+0.013 %** / `sealed`
   **+0.053 %**.** The lane predicate is `!station.is_empty() ||
   triggered_abilities.iter().any(|t| matches!(t.event.kind, StepBegins(_)))`
   — definition-only, and the loop computes it while it runs, so the
   caller-filled form applies exactly as it does to the two takes above. **It
   is a pool split, and the pools it loses on are the two the training loop
   plays**, so it is a revert under the standing rule.

```text
base 0541c28e+, callgrind, profiling-fast --no-default-features
  fixed     922,987,143 ->   920,947,107   -0.221 %
  cube    2,761,041,323 -> 2,761,395,369   +0.013 %
  sealed  2,739,549,090 -> 2,741,000,221   +0.053 %
```

   **The entry predicted the sign before the build and the numbers are the
   rule, so keep both.** The four bench archetypes carry not one step-keyed
   trigger, so `fixed` is a board where the lane always answers `ABSENT`; a
   cube or sealed board carries upkeep triggers, the lane answers `PRESENT`,
   the walk runs anyway, and what is left is the caller-fill's own cost — one
   `matches!` per printed ability and one `station.is_empty()` per permanent,
   paid on every step of every turn. **So the third condition for a
   caller-filled lane is that the fill be free, and it is only free when the
   walk already asks the lane's question.** `dispatch_board_scan` and
   `trigger_grant_sources` both load the same memo word they would have loaded
   anyway; this loop had to add a test. A variant that hoists the fill behind
   `lane.is_err()` would recover `cube` and `sealed` to flat and leave the
   change worth +0.22 % of the bench pool and nothing on the two that matter —
   which is not worth carrying, and is why it was not built.
2. **TAKEN at the hundredth pass** (`fixed` -0.280 / `cube` -0.611 / `sealed`
   -0.375 %): the damage-shield family, and it was bigger than "noncombat"
   because six of the seven walks are asked from `resolve_combat`, not from
   the noncombat funnel. One lane (`LANE_SHIELD`), one predicate that is the
   **union** of the seven walks' variant sets — see the Log block for why the
   union is sound and why sharing one lane beats seven. `resolve_combat` lost
   24-33 % of its self Ir. **The last two joined the same lane in a second
   commit** (`fixed` -0.128 / `cube` -0.204 / `sealed` -0.145 %): Light of
   Sanction and Indentured Oaf, two more variants in the union. The family is
   closed — `damage_prevented_by_protection` and
   `damage_from_source_prevented_by_keyword` are keyword-gated, not
   static-gated, and belong to `(-89)`. **The lane's floor is 27.0 Ir a call
   on every pool**; price any further consumer against that.
3. **`dispatch_triggers_for_events`' per-card loop, but only with an
   event-kind mask, not a lane.** The lane question ("does any permanent carry
   a printed trigger at all") is `PRESENT` on every real board, so a lane buys
   nothing. What would work is a per-definition mask of the `EventKind`
   *discriminants* its triggers carry, OR'd at the board level and tested
   against the batch's own kinds — but `EventKind` has **113 variants**, so it
   needs a `u128` and a 113-arm `kind_index()`, and `(-90)` already refuted the
   *inner* form of the same idea (11 `event_matches_spec` calls a dispatch).
   The outer form is untested; price the loop's prologue first (~13 M / 0.44 %
   of `cube` at the eighty-ninth pass's line profile) — that is the ceiling.
4. **`sba_board_scan` is NOT a candidate and this is why**: it reads
   `flipped`, `controller != owner`, `attached_to`, `bestowed`,
   `soulbond_partner`, `sector` and `counters` — six instance fields — so no
   definition-keyed lane can hold its answer. A lane needs a predicate that
   reads the definition and nothing else; that is the whole membership rule.

**And the allocation table, re-read at `841c7b9b` on `fixed` (526,301
allocations, the family ~10.4 %), says the reserve sweep is not worth a pass.**
Growths per call at the top `grow_one` sites: `mana_source_table` 1.95,
`deal_combat_damage_to_target` 2.20, `pick_attacks_inner` 2.16,
`resolve_combat` 1.73, `send_to_graveyard` 0.75,
`effective_mana_abilities_into` 0.13. A `with_capacity` removes only the
*second* growth — `(-80)` row 2's rule, "reserving capacity moves that
allocation earlier; it does not remove it" — so the whole family is worth
~0.3 % of `fixed` spread over six sites, each needing its own size argument.
Take one when you are already in the function.

**(-91) TAKEN at `cd0842e9`, `fixed` -0.673 / `cube` -0.721 / `sealed`
-0.813 %** — the memo now lives *inside* a `Deref`-only `Definition` handle,
so the invariant is structural and not merely compiler-enforced. The
Baseline block carries the numbers, the device, the lld-ICF warning the diff
produced, and why this reads 95 / 63 / 43 % of the ceiling below rather than
all of it. **The entry's own analysis stands and is kept for the shape of the
argument:**

**(-91) THE PER-OBJECT MEMO IS THROWN AWAY BY EVERY WRITE THAT REACHES A
CARD, AND THE DELETION CEILING IS `fixed` -0.711 % / `cube` -1.146 % /
`sealed` **-1.899 %** — THE LARGEST NUMBER ON THIS LIST.** Measured at
`e0cbb4a7` by `(-78)`'s device: comment out the one line
`d.memo.clear()` in `impl DerefMut for CardInstance` (`card.rs`) and read the
three pools. **Not committable as measured** — an in-place definition rewrite
then leaves a stale memo — but the ceiling is real and the fix is a design,
not a hack.

```text
base e0cbb4a7, callgrind, profiling-fast --no-default-features
  fixed     963,706,773 ->   956,853,484   -0.711 %
  cube    2,899,018,180 -> 2,865,781,687   -1.146 %
  sealed  2,885,278,610 -> 2,830,495,172   -1.899 %

the recompute family, `sealed` (`cube` is the same shape):
  CardDefinition::sba_scan_bits        7,338,304 ->   775,056
              its call count              76,540 ->     8,044   (-89.5 %)
  CardDefinition::printed_color_set    5,091,060 -> 1,743,454
  CardDefinition::grant_scan_bits      4,470,670 ->   327,744
  CardDefinition::gather_scan_bits     4,321,372 ->   416,518
  CardDefinition::type_scan_bits       3,209,220 ->   300,626
  CardDefinition::dispatch_scan_bits   2,834,244 ->   195,320
                             family   27,264,870 -> 3,758,718   -23.5 M
                            program                             -54.8 M
```

**89.5 % of every memo recompute in the program is caused by a write that
cannot have invalidated it** — a tap, a damage point, a counter, a
`must_block = None`. And the family is only 23.5 M of the 54.8 M the program
moves: the rest is the two atomic stores `clear()` does on every `&mut` reach
into a card, plus the miss path inlined into each of the ~1.7 M per-card
scan reads.

**Every value on `CardMemo` is a pure function of `definition`** — checked,
one by one: `printed_color_set` reads `color_override` / `keywords` /
`color_indicator` / `cost`; `vocab_index` reads the name; the five
`*_scan_bits` families read the definition's own fields. Nothing on
`CardData` outside `definition` is an input to any of them. The clear is
**conservative, not necessary**, and its own comment says so.

**The pointer key is unsound and the comment is right about why**: a
definition is uniquely owned on nearly every card (each `CardInstance::new`
takes its own `Arc`), so `Arc::make_mut(&mut c.definition)` rewrites it *in
place* and the pointer does not move. 17 such sites exist.

**So the fix is to clear at the definition-write sites instead.** Greppable:
17 `make_mut(&mut _.definition)` and 45 `.definition = ` (`card.rs` 19,
`effects/mod.rs` 16, `actions.rs` 14, `game/mod.rs` 9, `stack.rs` 2,
`snapshot.rs` 1, `bot.rs` 1 — some of those are constructor-adjacent and a
fresh `CardData` already has a fresh memo, so the real set is smaller).
**Two things make this safer than it sounds, and a taker should use both:**

* **The audit already exists and is the one this repo runs.** Every one of
  the seven accessors carries a `debug_assert_eq!` against a fresh recompute
  whose message is literally "a definition rewrite did not clear it". The
  robustness grid — five pools x six seeds x 120 games, 33,120 games — is
  what fires it, and the 19 k-test suite is not (an assertion needs a
  *board*).
* **The compiler can enforce total coverage.** Give `CardData::definition` a
  newtype with `Deref` to `Arc<CardDefinition>` and **no** `DerefMut`: every
  read (`c.definition.name`, thousands of them) keeps compiling and every
  write stops, until it goes through an accessor that clears. That turns a
  forgotten clear from a silent wrong-rules bug into a compile error, which
  is the difference between this being a good change and a bad one.

**Size it against the class it is in before starting.** This is `(-50)`'s
"no-op write through a CoW handle" family seen from the other end: that entry
asked which writes are no-ops, and this one asks which writes invalidate
what. Both are about `DerefMut` being the only chokepoint and therefore
carrying everyone's conservatism.
**(-92) lead 1 is TAKEN** (`848fc5b4` + `42235e7e`, `fixed` -0.50 / `cube`
-0.49 / `sealed` -0.37 % between them); leads 2 and 3 are open. The Baseline
carries the rule the pair produced: an `_on` conversion is worth its top few
callers and nothing else.

**(-92) THE PROFILE IS FLAT AND THE LEVER IS ELEMENT VISITS — read the
whole-program line profile in "Profile of record" before pulling anything
off this queue.** The program's hottest single *line* is 0.97 % and 11,165
lines hold 86 % of the run; `iter/macros.rs` + `ptr/non_null.rs` + the `Vec`
machinery are **~30 %** of it, and no engine source file is above 10.3 %.
Six functions have now been read by line across three passes and all six say
"no hot line". **So: stop looking for hot lines. The two questions a flat
profile still answers are `cg_calls.py` (how many calls) and
`cg_contexts.py` (whose).**

Three shapes the profile does still name, in order:

1. **`computed_permanent`'s battlefield `find` is 19.6 M / 0.68 % of `cube`**
   — `iter/macros.rs:349` 11.2 M plus `CardId::eq` 8.4 M, one linear find per
   `perms` miss and there are 289,098 of those. The repo already has the
   convention for it (the ninety-fifth pass's `_on` form): a
   `computed_permanent_on(&CardInstance)` skips the find for every caller
   that already holds the card, and the four biggest — `permanent_value_with`
   (which is *handed* `Some(c)` and throws it away), `legal_blockers`,
   `pick_attacks_inner`, `pick_blocks_inner`'s closure — are ~135 k calls
   between them. Size it at ~68 Ir a skipped find.
2. **`ComputedPermanent`'s construction and move is ~30 M / 1.05 %** —
   `compute_permanent_pass`'s line 1017 plus the overlay/`Vec` moves under
   it, 289,098 times. Not a pool ((-13), +2.60 %); it would have to be
   construction in place, which means `Arc::new_uninit` and unsafe.
3. **`Arc::clone_from_ref_in` at `ptr/mod.rs:1917` is the program's hottest
   line at 0.97 %** — the CoW deep copy itself, 132,370 of them. That is
   `(-1)`/`(-14)`'s family and the entry to read is those.

**(-90) THE GATHER COUNT IS AT ITS FLOOR AND THE PROFILE IS NOW TOPPED BY
THREE ROWS NOTHING CAN GATE.** Read at `3b8bfd03`, `--decks cube`, after the
ninety-sixth pass's two commits. The self table:

```text
  dispatch_triggers_for_events   6.59 %   (-59): no hot line, fewer calls only
  the allocator family          10.8 %    1,432,620 allocations, ~217 Ir a pair
  compute_permanent_pass         2.79 %   289,098 calls at 279.5 Ir self
  gather_continuous_effects_inner 2.86 %  59,002 calls, floor (see the Log)
  computed_permanent             2.56 %   355,828 calls, 81 % `perms` miss
  check_state_based_actions_into 2.4 %
  __memcpy                       2.48 %   1,026,751 calls
  evaluate_requirement_static_hinted x3   3.55 % over 1,425,996 calls
```

**Three leads, in the order a taker should size them:**

1. **`compute_permanent_pass` is 289,098 calls at 279.5 Ir of self, and the
   count is a floor — the lever is the price of one pass.** `perms` misses
   81 % of `computed_permanent`'s 355,828 calls, which reads like a broken
   memo and is not: `cg_contexts.py --separate-callers=4` says the passes are
   ~one per (scope, *distinct* card).

```text
  48,216  computed_permanent <- legal_blockers <- pick_blocks_inner
  43,226  compute_permanents <- combat_damage_computed          (whole board)
  34,994  computed_permanent <- permanent_value_with <- eval_material_inner
  24,000  compute_permanents <- check_state_based_actions_into <- resolve_combat
  21,778  compute_permanents <- declare_blockers                (whole board)
```

   `legal_blockers` asks about 10.85 cards a call and pays 10.59 passes;
   `eval_material_inner` pays 5.4 over 6,468 calls, each on a *different*
   simulated state whose scope starts empty. Nothing here is a scope to
   widen. What is left is the body: 1.9 `affected_includes_gated` calls a
   pass (the gathered list is short), 0.17 `PrintedList::push` (most passes
   modify no list) and a sort on ~10 % — so the 279.5 Ir is the layer walk
   itself, and it has never been read. (-27)'s allocation half stands
   separately: one `Arc<ComputedPermanent>` per miss is 201,780 on `cube`,
   14 % of every allocation in the program.
2. **The allocation table is diffuse and its top row is `finish_grow`
   (377,079, 26 %)**, spread over ~90 `grow_one` sites none of which is above
   0.17 %. The named rows below it are `computed_permanent` 201,780,
   `Arc::clone_from_ref_in` 201,610 (the CoW deep copies, inherent) and
   `Vec::from_iter` 127,441. **Read the mimalloc caveat before pricing any of
   it**: callgrind runs the system allocator and the shipped build does not,
   so the ~10.8 % is an upper bound on what a removed allocation is worth.
3. **`evaluate_requirement_static_hinted` is 3.55 % over 1,425,996 calls and
   927,746 of them are its own recursion.** The external entries are
   `statics_granted_triggers_inner` 142,094 (1.39 % inclusive) and
   `granted_abilities_of_inner` 105,436 (1.07 %). The first is already gated
   by the dispatcher's batch pre-filter; the second has not been read.

**Refuted by inspection this pass, no build spent, and do not re-derive
them:** a batch event-kind mask in front of `dispatch_triggers_for_events`'
inner loop (`event_matches_spec` is 820,738 calls over 74,798 dispatches =
**11 per dispatch**, so the inner fan-out is not the cost and a mask can only
reach 0.86 %) — **taken after all as `(-195)`, and it read `cube` -1.10 % /
`sealed` -1.34 %: the 0.86 % ceiling priced the calls, and the per-pair
gate also skips the guards and the `fanout` match ahead of the loop**; and
four freeze scopes to widen, all of which turn out to be one gather per
distinct game state (see the Log's context table).


**(-89) HALF TAKEN AT THE NINETY-EIGHTH PASS: three of the cascade's rows now
open on a `zone::Battlefield` presence lane, `fixed` -0.228 / `cube` -0.645 /
`sealed` -0.354 %** (`79019a42`, Log). The entry's own prescription — "the
`combat_damage_prevented_for_dealer` shape applied to the other eight, a bail
that costs single-digit Ir ahead of whatever each function does now" — turned
out to be answerable without a `PresenceGate` slot at all: the walks read
instance fields but every static they can find is printed, so the *board-level*
question is definition-only and the zone memo answers it.
`combat_damage_prevented_from` 11.20 M -> 0.92 M, `scale_damage_to` 7.87 M ->
1.24 M, `combat_damage_prevented_to_self` gated inside `resolve_combat`.

**What is left of the cascade is the keyword-gated rows** —
`damage_prevented_by_protection`, `creature_redirects_damage_to_controller`,
`damage_from_source_prevented_by_keyword`, `apply_prevention_shields_with` —
whose cost is the memo read plus the keyword scan, not a board walk. Read
`(-61)` before proposing anything there; and **the rule the taken half
produced is the ranking one**: a presence lane is worth what its caller does
*not* already gate, so rank candidates by that and not by row size.

**(-89) RE-READ TWICE AT THE HUNDREDTH PASS — four rows closed, and the four
that are left are all one shape.** `LANE_SHIELD` took
`source_damage_to_color_prevented` from 182 to **27.0** Ir a call on `cube`,
`damage_from_your_source_to_your_creature_prevented` to **21.3**, and the
three `damage_to_attacker` / `creature_token` / `your_creature` shields to
**21.0**; then `LANE_GRANT` took 6-14 % more off the three keyword-gated rows
on `cube` and 39-47 % on `sealed`. `cg_edges.py --callees resolve_combat`,
`--decks cube`, at `98feda21`:

```text
  apply_prevention_shields_with    12,754   812 Ir   0.379 %
  creature_redirects_damage_to..   12,786   769      0.360 %
  damage_prevented_by_protection   12,786   763      0.357 %
  damage_from_source_prevented..   12,786   681      0.319 %
  prevent_combat_to_target          7,570   752      0.208 %
  combat_damage_prevented_to_self  12,786   295      0.138 %
  scale_damage_to                  20,324   135      0.100 %
  combat_damage_prevented_from     19,326   123      0.087 %
  ironscale_replace                12,754   161      0.075 %
  source_damage_to_color_prev..    12,786    27      TAKEN (LANE_SHIELD)
  damage_from_your_source_to_..    12,786    21      TAKEN (LANE_SHIELD)
  the three board shields           5,590    21      TAKEN (LANE_SHIELD)
  combat_damage_prevented_for_d.   19,920     4      the model
```

**The top four are 1.42 % of `cube` and three of them run
`card_keyword_possible`, whose callee table is the same rows in each**
(`Graveyard::has_anthem` exactly 2 a call, `CardDefinition::can_grant_keyword`,
`Keyword::eq`). So they are not three leads, they are one:
`card_keyword_possible` = `battlefield_find(id)` +
`keyword_grant_in_scope(pred)`, and the second half is now lane-gated.

**The `battlefield_find` in front of it is BUILT AND REVERTED at +0.099 /
+0.406 / +0.195 %** — the `_on` conversion of all three, Log block (4). The
premise was wrong: inside a freeze scope `computed_permanent(id)` looks the id
up in `perms` and never reaches its find, so the hoist is pure addition, and
even the pure `computed_permanent_on` swap in
`creature_redirects_damage_to_controller` read +1.5-1.9 %. **Do not rebuild
it.** What that leaves in these rows is `card_keyword_possible_on`'s own body
and the `perms` lookup, and neither has been sized by line.

**Also refuted without a build: a lane over `keyword_grant_in_scope`'s
`continuous_effects` leg** — it is not a battlefield walk, the zone cannot key
it, and the leg runs before the board one in the `||` chain.

**TAKEN, and not by an `_on` conversion** (hundredth pass, fourth commit).
`(-38)`'s one-entry index memo makes the second and later finds of the same id
a load and a compare, so the four rows fell together without any signature
changing: `damage_from_source_prevented_by_keyword` -20.3 %,
`creature_redirects_damage_to_controller` -16.9 %,
`apply_prevention_shields_with` -21.2 %,
`combat_damage_prevented_to_self` -72.8 % on `cube`. **"Four rows asking for
the same id inside one freeze scope" was the right reading and the memo was
the cheaper answer to it** — an `_on` conversion of this family is now worth a
load and a compare per row, i.e. re-size it before writing it.

**(-89, as re-read) RE-READ AT `a69a8287` AND ONE OF ITS PREMISES IS WRONG — the cascade
is still ~1.9 % of `cube`, but "ten whole-battlefield walks a damage event"
is not where the Ir is, and a taker who fuses the walks will be disappointed.**
The rows at that tip, `cg_edges.py --callees resolve_combat`, `--decks cube`:

```text
  apply_prevention_shields_with   12,754   862 Ir   0.39 %
  creature_redirects_damage_to..  12,786   821      0.37 %
  damage_from_source_prevented..  12,786   732      0.33 %
  scale_damage_to                 20,324   378      0.27 %
  prevent_combat_to_target         7,570   927      0.25 %
  combat_damage_prevented_from    12,656   536      0.24 %
  ironscale_replace               12,754   161      TAKEN (e0cbb4a7)
  combat_damage_prevented_for_de. 13,250     4      the model
                                                 ~1.9 % together
```

**A battlefield walk in this family costs ~230-300 Ir, not the 900 the row
does**, because its per-card body is `for sa in &card.definition
.static_abilities` and that list is *empty on nearly every permanent* — one
length check, which is exactly the shape `(-87)` and `(-77)`'s fourth row
were refuted on. The whole-program line profile prices
`prevent_static_scan`'s inner loop (`effects/movement.rs:73`, inlined into
`resolve_combat`) at **2.5 M / 0.10 %** for all 12,754 of its calls
together. **So hoisting `prevent_static_scan` out of the per-event
`apply_prevention_shields` — the obvious fused-scan move, and the one this
entry's shape suggests — has a ceiling of about 0.05 %.** The Ir is in the
gated legs' own bodies (`battlefield_find` on the source, the keyword
predicates, the shield lists), not in the scans in front of them. Size a leg
before gating it.

**(-89, original) THE COMBAT-DAMAGE PREVENTION CASCADE IS 1.55 / 2.34 / 1.69 % OF THE
THREE POOLS AND IT IS TEN QUESTIONS PER DAMAGE EVENT, ASKED ON BOARDS THAT
CARRY NONE OF THE CARDS THEY ARE ABOUT.** Read at `86936dc9` + the `by_kind`
patch, `cg_edges.py --callees resolve_combat`, all three pools, one binary.
Every row is a direct callee of `resolve_combat` (i.e. of
`resolve_combat_damage_with_filter`, inlined into it):

```text
                                    fixed            cube            sealed
                                 calls  Ir/call   calls  Ir/call   calls  Ir/call
  prevent_combat_to_target        2,686    907    7,570   1,140    9,454    912
  creature_redirects_damage_..    3,752    686   12,786     976   12,340    640
  apply_prevention_shields_with   3,752    654   12,754     862   12,340    625
  damage_prevented_by_protection  1,852    570    6,116     915    5,856    530
  damage_from_source_prevented..  3,752    503   12,786     732   12,340    465
  combat_damage_prevented_from    4,172    390   12,656     535   14,350    381
  ironscale_replace               3,752    357   12,754     476   12,340    356
  scale_damage_to                 6,438    230   20,324     378   21,794    254
  source_damage_to_color_prev..   1,852    140    6,116     182    5,856    139
  combat_damage_prevented_for_d.  4,428      4   13,250       4   15,176      4
                       total  15,143,052       68,793,909       49,392,824
                                   1.55 %           2.34 %           1.69 %
```

**The last row is the shape the other nine want.**
`combat_damage_prevented_for_dealer` is **4 Ir a call** — it opens on a
`ColdState` `is_empty()` and stops. The nine above it are 140-1,140 Ir each
and every one of them ends in "no" on a board with no fog, no Ironscale
Hydra, no protection, no Absorb and no damage-scaling static.

**Two things are already right and are not the lead.**
`damage_prevented_by_protection` and `creature_redirects_damage_to_controller`
each carry a `!layers_memoized() && !card_keyword_possible(..)` gate, and the
resolver's per-pair freeze scope means the gate is *skipped* once the gather
has happened — so their 900-ish Ir is the memo read plus the keyword scan, not
a board walk. Read `(-79)`/`(-61)` before proposing anything keyword-shaped:
the board-level OR of the per-card grant gates is in the do-not-rebuild list
twice over.

**THREE ROWS ARE TAKEN IN THE PASS THAT FILED THE ENTRY, ALL THREE BY READING
THE FUNCTION RATHER THAN THE TABLE.** `prevent_combat_to_target` 1,140 -> 927
Ir a call on `cube` (the Serra's Emissary walk tests the instance field
first), `creature_redirects_damage_to_controller` 976 -> 846 (the presence
gate takes the permanent the function already found), and `ironscale_replace`
477 -> 161 (three questions about one card's statics become one walk).
Together `fixed` -0.180 / `cube` -0.252 / `sealed` -0.201 %. See the three
Baseline blocks — **and read the two refutations in them before repeating any
of the edits at a neighbouring site: two of the three mechanisms were applied
to a second row in the same cascade and both lost there.**

**What is left, and what has still not been tried: the
`combat_damage_prevented_for_dealer` shape applied to the other eight** — a
*cold-group / instance* `is_empty()`-class bail that costs single-digit Ir,
ahead of whatever each function does now. The open questions, in the order a
taker should ask them: (1) does each row have a state field whose emptiness is
authoritative for it — the `combat_damage_prevented_creatures` /
`prevent_shields` / `damage_scaling` families — or does it need a walk to find
out; (2) if it needs a walk, is the question a `PresenceGate` slot (a fifth
costs ~113 k Ir on `cube`, see `(-85)`, which is 0.004 % against a 2.34 % row)
or does the caller hold `&mut self`; (3) `apply_prevention_shields_with` is now
the largest at 862 Ir a call, and `(-61)` already says its Absorb gate is a
4.5x trade that must not be deleted — so what is on offer there is the
`prevent_static_scan` walk beside it, not the gate.

**(-88) `check_state_based_actions_into` IS A 2.4-3.1 % SELF ROW ON EVERY
POOL AND NO ENTRY HAS EVER NAMED IT.** Same tip and same dumps as `(-89)`.
**CLOSED — half taken, half refuted with a census.**

**Taken at `ec5dc3a9` for -0.559 / -0.751 / -0.533 %**: the death sweep's
whole-board layer pass is now a pass over the permanents its own gate names
(Log, ninety-seventh pass).

**Refuted at `6e44ce7c`: this entry's own first question, measured.**
`CRAB_SBA_CENSUS=1` fingerprints the sweep's read set and counts consecutive
sweeps that see the same state — **3,613 / 20,152 on `cube` (17.93 %), 1,736 /
9,146 on `fixed` (18.98 %), 5,594 / 31,452 on `sealed` (17.79 %)**, and
interleaving with the bot's simulation clones makes those a lower bound. **It
loses by arithmetic before soundness is reached.** A repeat sweep is by
construction one whose predecessor changed nothing, so the skip saves the
*no-op* sweep — `sba_board_scan` plus the gated `any` walks, ~2.5-3.5 k Ir —
on ~18 % of sweeps, while the fingerprint that proves the state unchanged is a
whole-board walk of the same shape as the `sba_board_scan` it would skip, paid
on 100 % of them. And a 64-bit hash is not a correctness gate over millions of
games; an exact snapshot is the walk again plus the storage. **The rule:
before proposing a memo keyed on "nothing changed", price the witness against
the work — a witness over the same collection as the work is never cheaper
than the hit rate.** The instrument stays, gated (+0.004 / +0.001 / +0.003 %
off, i.e. the ~113 k Ir a gate costs).

```text
                     self Ir     % of pool    calls    self Ir/call
  fixed           25,580,616       2.61 %     9,146       2,797
  cube            71,659,780       2.44 %    20,152       3,556
  sealed          89,865,358       3.07 %    31,452       2,857

callers (cube):  resolve_top_of_stack_inner 8,774 / resolve_combat 4,704 /
  do_cleanup 2,766 / submit_decision 1,528 / run_effect 2,106 / the rest 274
its two biggest callees (cube):
  36,150   80,923,213  2.75 %  Vec::from_iter (nested)   <- the sweep's collects
  20,260   30,837,328  1.05 %  sba_board_scan
```

**~2,800-3,600 Ir of *self* is the sweep's fixed cost before any SBA fires**,
and that is after `sba_board_scan`'s bit table already gates a dozen rare
legs. `dispatch_triggers_for_events` (6.49 % of `cube`) is the only self row
above it, and `(-59)` says of that one "there is no hot line, the lever is
fewer calls" — **ask the same question here first, because it is answerable
from the caller table above and costs no build**: 20,152 sweeps a `cube` run
against 66,612 `perform_action_inner` calls is one sweep per 3.3 actions, and
CR 704.3 says a sweep happens whenever a player would receive priority. How
many of these are re-sweeps of a board nothing has touched since the last
one? **Do not run a line profile on this** — `(-59)` spent a cold
`profiling-lines` build to learn that a function of this shape has no hot
line, and this one has the same shape (a long chain of gated legs).

**(-85) BUILT, MEASURED AND REVERTED — packing `LayerFreeze::gates` into one
`AtomicU32` is a pool split (`cube` **+0.011 %**), and the reason is a ratio
worth carrying: a presence gate is READ 3.5x more often than a scope EXITS.**
Filed by `(-84)(b)`'s counter-row (a fourth slot cost `Unfreeze::drop`
+113,000 / +59,772 / +149,374 Ir), on the argument that `clear_gates` is one
store per slot at every outermost scope exit and so taxes every future gate.
The argument is right and the fix still loses:

```text
base 1924fb21, callgrind, profiling-fast --no-default-features
  fixed    998,835,007 ->   998,663,701   -0.017 %
  cube   2,988,414,558 -> 2,988,740,744   +0.011 %
  sealed 2,972,594,700 -> 2,972,297,675   -0.010 %

cube, the whole diff, and it is two sides:
  Unfreeze::drop                     10,348,176 -> 10,009,176   -339,000
  freeze_layers_pop                   1,753,442 ->  1,660,370    -93,072
  evaluate_requirement_static_hinted 28,217,494 -> 28,888,870   +671,376
  available_mana                     16,372,603 -> 16,421,127    +48,524
  scan_land_type_rewrites               654,686 ->    702,076    +47,390
  blocker_self_block                 10,728,974 -> 10,765,042    +36,068
```

**The clear was exactly what it was thought to be — 1 Ir a slot a pop, so
four slots is ~452 k of a `cube` run (0.015 %), and the packed clear takes
three quarters of it back.** What the entry never counted is the other side:
`gate()` on an array is one `movzx` of a byte; on a packed word it is a load,
a shift and a mask. **+2 Ir a read, ~400 k reads a `cube` run against ~113 k
outermost pops** — 3.5 reads an exit, so a per-read cost cannot be paid for
by a per-exit saving. `fixed` and `sealed` ask fewer gates per scope and both
win; `cube`'s requirement walker asks `creature_type_change_in_scope`
constantly and loses more than the clear returns.

**Standing reading, and the useful half:** the array is the right structure,
the per-slot clear is the right price for it, and **a fifth gate costs ~113 k
Ir a `cube` run** (0.004 %) — cheap enough that the scaling worry this entry
was filed on is not one. Do not rebuild the packing. If a device that clears
in O(1) *and* reads in one byte load is ever wanted, it has to be an epoch
stamp, and an epoch stamp is two loads and a compare per read, i.e. the same
trade in the same direction.

**(-87) IS CLOSED. Taken over two commits at the ninety-eighth pass, `fixed`
-0.464 / `cube` -0.854 / `sealed` -0.405 %**, which is **78.5 / 73.1 / 80.2 %
of the entry's own deletion ceiling** (`(-78)`'s device, read at the same
tip: 941,127,494 / 2,796,595,503 / 2,790,994,042). `GameState.battlefield` is
a `zone::Battlefield`; the two gates read a packed two-bit memo off it, kept
by a `DerefMut`/`push` clear for membership and a `DEFINITION_EPOCH` stamp for
definition rewrites, with the predicates widened to read no instance field.
The two Log blocks carry the numbers, the three inlining refutations the build
sequence produced, and the census correction — which held twice, in the same
direction, on the same pool.

**What is left is +0.128 / +0.319 / +0.100 % against a memo that never
invalidates at all, i.e. the walks that genuine membership and definition
changes force.** That is a floor for any correct invalidation, not a lead.
Do not reopen this without a different mechanism.

**The remaining walks' per-card BODY was a separate lever and it is taken**
(ninety-ninth pass, `fixed` -0.028 / `cube` -0.071 / `sealed` -0.033 %): the
two predicates were the last of the layer-4 family still walking
`static_abilities` plus `station` off a `CardDefinition`, and the widening
above is what let them onto `CardMemo` as `layer4_bits`. A third off the
per-walk cost, 495.9 -> 330.2 Ir on `cube`. **And fusing the two lanes into
one pass loses** (+0.31 / +0.36 / +0.27 %) — the lanes are not asked in the
same invalidation epoch, so it removes 3.4 % of `cube`'s walks and 0 % of the
other two pools' while costing 82 % more per walk. The Log has the shape.

**The historical analysis, kept for the shape of the argument:**

* The shipped invalidation is the cheap `DerefMut` chokepoint and it serves
  **48.1 / 38.3 / 32.2 %** of the walks. The remaining walks are 37,868 /
  12,674 / 38,054 at ~430 / 303 / 291 Ir, i.e. **0.58 / 0.41 / 0.39 % of the
  three pools still on the table.**
* The exact invalidation — bump on battlefield *membership*, on a definition
  rewrite, and on `attached_to` — would take a share of those. Do not size it
  off `CRAB_GATE_CENSUS`'s 93.95 / 75.76 / 74.30 % without applying this
  pass's correction: those are ask-population rates and the freeze scope
  already owns 70 % of `cube`'s asks. The honest read is the *ratio* the two
  censuses give — exact/broad = 1.11 / 2.11 / 2.53 — against the walk rates
  measured here, which puts the remaining win at roughly `cube` -0.1 % and
  `fixed` / `sealed` -0.2 % each. **Measure it before building it.**
* **The newtype the exact version needs now exists, so its 106-site cost is
  already paid.** "The invalidation sized" below was filed by a concurrent
  session hours before this landed and reaches the same shape from the other
  end: membership mutators bump an epoch, `iter_mut` / `get_mut` do not, and a
  *thread-local* (never a global `AtomicU64` — one thread's rewrite would
  invalidate another's memo at `--threads 3`) counter at `(-91)`'s two
  compile-enforced definition-write accessors covers the fourth key part.
  What is left of that work is moving the clear out of `DerefMut` and into
  `Battlefield`'s membership entry points.
* **And the fourth key part can be deleted rather than tracked.**
  `card_can_change_*_types` reads `attached_to`, an *instance* field, which is
  why a definition-only epoch is unsound as written. But both gates are
  over-approximations — only a stale `false` is unsound — so dropping the
  `attached_to` conjunct and answering `true` for any permanent whose
  definition carries an `equipped_bonus` land/creature-type field is sound by
  widening. That makes the memo's input a pure function of the *definitions on
  the battlefield*, and costs only the gathers that a land-type-setting
  Equipment sitting unattached would newly trigger.
* The other lever is the write rate itself: every `&mut` reach into the
  battlefield now costs a memo clear as well as a CoW unshare, so a hot
  `for c in &mut self.battlefield` that only *reads* is worth twice what
  `(-1)`/`(-14)` price it at. Nobody has swept those sites.

**(-87, original) THE LAYER-4 TYPE GATES MISS 45 k / 27 k TIMES A `cube` RUN AND EACH
MISS IS A 262-642 Ir BOARD WALK — 1.13 % OF `cube`, AND HALF OF IT IS ASKED
FROM `&mut self`, WHERE NO SCOPE CAN EVER MEMOIZE IT. Both of the entry's own
proposals are now built and refuted; the two blocks at the end are the
result, and the second is a rule.** Read at `efebd811`
off the same dumps as `(-84)(b)`. The rows are the gates' *closures*, which
run only on a miss:

```text
                                    fixed             cube             sealed
  land_type_change_in_scope     17,648  5,013,924   45,082 15,882,434   47,596 12,487,024
                       Ir/call            284.1                352.3              262.4
  creature_type_change_in_scope  2,892    936,378   27,794 17,847,758    8,564  2,872,202
                                          323.8                642.1              335.4
                     % of pool             0.60 %               1.13 %             0.52 %

land gate, by caller (calls):        fixed    cube   sealed
  activate_ability_inner             8,418  19,320   22,794    <- &mut self
  bot::available_mana                5,900  12,110   15,444
  scan_land_type_rewrites            3,330   9,478    9,146
creature gate, cube:
  evaluate_requirement_static_hinted 18,900
  dying_snapshot                      8,894
```

**`activate_ability_inner` is 48-54 % of the land gate's misses and it holds
`&mut self`, so it is *provably* outside every freeze scope** — the same
property `card_type_change_unscoped` was written for. An unscoped twin saves
the depth load and nothing else: the walk is the whole cost. And
`available_mana`'s misses are 1:1 with `(-82)`'s per-tick `available_mana`
builds (5,900 on `fixed`, 12,110 on `cube`), which says those asks are each in
a scope of their own — worth confirming before theorising, because if they are
*not*, the memo is being reset by something and that is a bug, not a cost.

**THE SPLIT IS NOW PRICED AND IT IS ALL ONE SIDE.** Probe at `90c0e850`,
`(-78)`'s device: drop the `continuous_effects.iter().any(..)` operand from
*both* gates and the program reads `cube` -360,464 (**-0.012 %**) / `fixed`
-176,163 (**-0.018 %**). That operand is **1 %** of a 33.7 M-Ir row. The gate
IS `battlefield.iter().any(card_can_change_*_types)`: 72,876 calls a `cube`
run x 23 permanents = **1.68 M card visits at ~20 Ir apiece**. (Not a
committable change — dropping it loses Spreading Seas and Magical Hack.)

**THE PER-DEFINITION BIT IS BUILT AND REFUTED — it costs `cube` +0.138 %, and
the reason generalises.** `type_bits` extended from two bits to six (a
`LAND_ATTACHED` / `LAND_ALWAYS` / `CREATURE_ATTACHED` / `CREATURE_ALWAYS`
quartet on the same `CardMemo` word and the same valid flag), the two
`static_effect_changes_*_types` predicates moved to `crabomination_base` next
to their card-type twin, and both `card_can_change_*_types` reduced to the
memo read that `card_can_change_card_types` already used:

```text
base 90c0e850, callgrind, profiling-fast --no-default-features
  fixed    989,693,709 ->   990,876,369   +0.119 %
  cube   2,965,902,369 -> 2,969,989,765   +0.138 %
  sealed 2,947,622,521 -> 2,949,902,159   +0.077 %

cube, the two gates and the callers that inlined their prologues:
  land_type_change_in_scope::{{closure}}      15,882,434 ->          0
  creature_type_change_in_scope::{{closure}}  17,847,758 ->          0
  land_type_change_in_scope (now out of line)          0 -> 17,755,668
  creature_type_change_in_scope                        0 -> 20,511,332
  CardDefinition::type_scan_bits               2,639,718 ->  4,725,394
  evaluate_requirement_static_hinted          28,217,494 -> 26,212,316
  scan_land_type_rewrites                        654,686 ->    484,146
  activate_ability_inner                      48,438,564 -> 48,320,802
  bot::available_mana                         16,397,083 -> 16,281,971
  check_state_based_actions_into              71,440,496 -> 71,367,824
                                       program            +4,087,396
```

**It is not a memo-miss story — the memo works.** `type_scan_bits` recomputes
28,832 times a `cube` run against 1.68 M card visits, a **98.3 % hit rate**,
and the recompute is only 1.84 M of the 4.09 M regression.

**The rule is about what the walk was, not about the memo.** The two
predicates walked `def.static_abilities` and `def.station` with
`iter().any(..)` — and both lists are **empty on nearly every permanent**, so
that "walk" was already a pointer load, a length load and a not-taken branch.
The memo read is a word load, a valid-bit test and two mask tests against the
*same* `CardData` deref: the same handful of instructions, plus a recompute
that now covers three families where the card-type gate's covered one.
**A per-definition presence bit pays only when the walk it replaces is over a
list that is usually non-empty** — which is why `sba_scan_bits` (five inner
loops) and `dispatch_scan_bits` (every static, plus every Station band) both
won and this one cannot.

**What is left, and it is the whole entry now:** the walk is irreducible at
~20 Ir a permanent, so the only lever is **calling it less**. 72,876 calls a
`cube` run, of which `activate_ability_inner`'s 19,320 are `&mut self` and can
never reach a freeze scope. A `GameState`-level memo with real invalidation is
the only shape left; it is `(-62)`'s class and the reason that class is refuted
is the battlefield's *write* rate — but note this question does not read tap
state, so the invalidation it actually needs is **zone membership plus
`continuous_effects`**, not `DerefMut`. Nobody has priced *that* write rate.
Price it before building anything.

**`(-87)`'S OPEN LEVER WAS PRICED AT PASS 97 AND BUILT AT PASS 98** — the
`zone::Battlefield` lane, hit 75.6 / 79.5 / 72.9 % against the ask counts
below. Kept because the *method* transfers and the prediction can be
checked against the outcome: a census costing one `release-fast` build
turned this entry's "nobody has priced that write rate" into a shipped
0.6-0.9 %, and the exact-vs-broad split below is what told the taker not
to invalidate on `&mut battlefield`.
The entry ends "nobody has priced *that* write rate. Price it before building
anything." `CRAB_GATE_CENSUS` — built, read, and then taken back out, see
below — fingerprints the gates' inputs at every ask and counts consecutive
asks that see the same fingerprint:

```text
                asks     exact repeats        broad repeats
  cube        242,788   228,106  93.95 %    204,716  84.32 %
  fixed        20,540    15,562  75.76 %      7,384  35.95 %
  sealed       56,722    42,146  74.30 %     16,688  29.42 %
```

**exact** = what an epoch bumped at zone-membership / `continuous_effects` /
definition writes would see. **broad** = what the *cheap* invalidation a
`CowBox<Vec<CardInstance>>` chokepoint can actually detect: any `&mut` reach
into the battlefield, tap and damage included. Both are lower bounds — the
bot's simulation clones ask on the same thread, so every hop between two
states reads as a change.

**Two things fall out, and the second is the design.**

1. The gates walk on 72,876 of `cube`'s 242,788 asks (the freeze-scope memo
   serves the other 70 %). A `GameState`-level memo at the exact rate would
   walk on ~6 % — **~80 % of the gates' 33.7 M Ir, ~0.9 % of `cube`**, and it
   reaches the ~50 % of misses that arrive holding `&mut self` and can never
   see a freeze scope, which is the half this entry says is unreachable today.
2. **The invalidation has to be precise, and the pool split says so loudly.**
   On `cube` the cheap chokepoint keeps 84 of the 94 points; on `fixed` and
   `sealed` it keeps 36 of 76 and 29 of 74 — **it gives back half the win on
   one pool and 60 % on the other**, because tap and damage writes dominate
   there. Do not ship the `&mut battlefield` version.

**What a taker still owes**: the enumeration. `continuous_effects` mutations
and battlefield membership changes are the easy two; the third is a definition
rewrite on a battlefield permanent, and after `(-91)` those are the compile-
enforced `Definition` accessors — but they are `CardData` methods with no
`GameState` to bump. The saving grace is that **both gates are
over-approximations — `false` is authoritative, `true` only means the gather
has to run** — so only a stale *`false`* is unsound, and the audit is the one
`gather_continuous_effects` already runs: `debug_assert!` the implication and
let `robustness_grid.sh` fire it over 33,120 games.

**THE INVALIDATION SIZED, SO THE NEXT RUN CAN BUDGET IT.** The key wants to
be O(1). Three of its four parts already are — `battlefield.len()`,
`continuous_effects.len()` and `next_effect_timestamp`, which
`GameState::next_timestamp` advances on **every effect install and on the four
battlefield entries that stamp a permanent** (`movement.rs:2456`,
`mod.rs:14298 / 14344 / 14508`). The fourth is a definition rewrite on a
battlefield permanent, and after `(-91)` that is **two** compile-enforced
sites (`CardData::set_definition` / `definition_mut`), so a thread-local
counter bumped there costs ~8 k increments a run and covers it — a global
`AtomicU64` does not, because one thread's rewrite would invalidate another's
memo at `--threads 3`.

**What is NOT covered, and it is the whole remaining cost: the nine
battlefield pushes that stamp nothing** (`movement.rs:2551 / 2654`,
`effects/mod.rs:2536`, `actions.rs:3174`, `stack.rs:1933 / 2846 / 6960`,
`mod.rs:7755 / 20855` — checked, none has a `next_timestamp()` within twelve
lines) plus the removals. A permanent leaving and a different one entering
between two asks leaves `len` and the timestamp both unchanged, and that is a
stale *`false`*. So the design needs a membership epoch, and the two ways to
get one are:

* **19 engine sites** (`self.battlefield.{push,remove,retain}`) bumping it by
  hand — but ~25 more live in `#[cfg(test)]` blocks in `encode.rs` /
  `view.rs` / `bot.rs` / `snapshot.rs`, which would go stale silently in
  `release` and loudly under the audit. A grep, not a claim.
* **A `Battlefield` newtype** — `Deref` to `[CardInstance]`, membership
  mutators that bump the epoch, `iter_mut` / `get_mut` that do not. That is
  the `(-91)` device again and it is the right one, and it is **106 mutable-use
  sites** of `battlefield` across the workspace against 871 reads. Budget a
  run for it, and take the branch to yourself: it is not a patch to carry
  across a concurrent session's pushes.

**And the instrument itself produced a rule about gates.** `(-85)` says "a
gate is read 3.5x per scope exit, so a slot costs ~113 k Ir and nothing else —
add gates freely." That was calibrated on a ~20 k-call site. In front of
`presence_gate`, which is asked **242,788** times a `cube` run, a dormant
census hook measured:

```text
  OnceLock<bool>, not inlined     fixed +0.009 %  cube +0.049 %  sealed +0.009 %
  #[inline] AtomicU8 fast path    fixed +0.055 %  cube +0.187 %  sealed +0.052 %
```

The second is *worse*, and the reason is the file's own inlining rule seen
from a new side: `#[inline]` on the reader expanded its **cold** branch — an
`env::var` returning a `String` — into all 242,788 call sites. **A dormant
gate is ~5 Ir a call, so its cost is the call count; price it against the site
before adding one, and never `#[inline]` a reader whose slow path allocates.**
Both censuses' hooks came out on that reading; `CRAB_SBA_CENSUS` (20,152
calls, +0.001-0.004 %) stayed.

**(-86) `String as fmt::Write` IS 158,146 CALLS IN THE SIMULATOR AND IT IS
NOT A LEAD — READ THIS BEFORE CHASING IT.** A `{:?}` Debug format of a whole
`CardDefinition` runs **217 times** on a six-game `cube` run at ~57,000 Ir
apiece — 12,464,496 Ir inclusive, **0.42 % of the run** — and every one of
them is `CardDefinition::wants_converge`. It is *already* two-level cached
(a thread-local table keyed on the name's pointer, then a process-wide map),
so the format runs **at most once per card name per process**: 217 formats
is 217 distinct names, and a `selfplay_train` actor amortises the whole
family to nothing over millions of games. It is visible only because the
bench is six games long. **The `format!` is also the robust half of that
function** — its predecessor enumerated fifteen `Effect` arms and had rotted
in both directions — so replacing it with a structural walk trades a real
0.4 % of a *short* run for the exact defect class the function's doc
comment records. Leave it; and when a `fmt` row appears in a profile, check
whether it is per-name or per-call before pricing it.

**(-84) THE BLOCK-LEGALITY TRIO IS 1.46 % OF `cube` AND TWO OF ITS THREE
ROWS HAVE A NAMED SHAPE.** Read at `ea2cb263`, `--decks cube`, and all three
are off the top thirty on `fixed` — this is a `cube`/`sealed` entry, so size
it there.

```text
  74,692 calls   16,206,302   0.540 %   blocker_self_block
  62,366         14,267,772   0.475 %   blocker_pair_block
  61,408         13,400,800   0.447 %   can_block_attacker_computed
```

**(a) TAKEN at the ninety-third pass (2) — `fixed` -0.125 % / `cube`
-0.262 % / `sealed` -0.176 %, and the row fell 58 %.** See the Baseline; the
sizing below stands as written and read 14 % low on `cube`.
`can_block_attacker_computed` made 22 passes over two short keyword
slices.** Thirteen over `attacker_kws` (six `has_kw`, three
`iter().any(matches!)`, two more `has_kw` for Fear/Intimidate, then the
protection loop and the `CantBeBlockedExceptBy` loop) and nine over
`blocker_kws`. The function is a **pure disjunction** — every arm returns
`false`, the fallthrough returns `true` — so the order of the tests is not
load-bearing and one pass per side accumulating flags is behaviour-preserving
by construction. Every keyword it tests by `has_kw` is a *unit* variant, so a
`match` arm is exactly the discriminant compare `has_kw` does. **Size it
against `(-78)`'s test 1 before building**: these are short *per-call*
slices, not a board walk, so what is on offer is the 20 loop set-ups and not
the elements.

**(b) TAKEN at the ninety-fourth pass, by a mechanism the entry never
named — `fixed` -0.032 % / `cube` -0.179 % / `sealed` -0.093 %, which is
28 / 54 / 42 % of the deletion ceiling below, and it wins on all three
pools where the hoist split them.** See the Baseline. The question moved
into a `LayerFreeze::gates` **presence-gate** slot: it is a printed static
on the battlefield, so a freeze scope cannot change the answer, and the
planner already runs inside one — no signature, no scan, and the walk
happens once a scope instead of once per even blocker. `TypeGate` is now
`PresenceGate`. **Before threading a scan through N callers, ask whether
the question belongs in a `PresenceGate` slot**: the whole test is "can a
freeze scope change the answer", and the answer here was yes for every
caller the refutation below measured.

What is left of the ceiling is the callers that are *not* in a scope —
`declare_blockers` per assignment, and any bot path reached on a fresh
clone, whose `LayerFreeze` resets to unfrozen. They pay the walk exactly
as before.

The refutation of the *hoist*, kept in full — a mechanism refuted stays
refuted, and this one is the reason the gate was worth looking for:

**(b) BUILT, MEASURED TWICE AND REFUTED at the ninety-third pass (2) — the
hoist costs `fixed` **+0.090 %**. The deletion ceiling is real and it is not
reachable this way; read this before proposing a block-side scan again.**
The probe the entry asked for was run first and the leg *is* worth taking:
deleting it outright (`if false && …`) at `c1e4363c` reads

```text
  fixed   998,970,064 ->   997,845,100   -0.113 %
  cube  2,993,004,323 -> 2,983,037,131   -0.333 %
  sealed 2,976,502,367 -> 2,969,904,182  -0.222 %
```

**and then the obvious fix loses.** A `block_static_scan` twin of
`attack_static_scan`, hoisted out of `declare_blockers`' assignment loop,
`legal_blockers`' filter and `pick_blocks_inner`' blocker loop, with the
single-pair convenience APIs passing `None` to keep their old per-card gate
(the `GrantScan::land_types_rewritten` shape):

```text
  u32 everywhere      fixed +0.079 %   cube -0.150 %   sealed -0.041 %
  Option<u32>         fixed +0.090 %   cube -0.131 %   sealed -0.023 %
```

**The arithmetic the entry did not do: count the blockers per pass.** The
leg is *conditional* (`cmc` even) and its walk is a whole-board `any` that
exits on the first `static_abilities` entry — on `fixed` there are none at
all, so it is ~23 derefs and 23 length loads. The scan is **unconditional**
and walks the same board. A hoist out of a loop of N blockers is a win only
when N is large, and `legal_blockers` / `pick_blocks_inner` run over **one to
three** candidates a pass while being called thousands of times: the scan
therefore runs about as often as the leg did, at roughly twice the cost.
**Before hoisting a per-item board walk out of a loop, divide the loop's
total item count by its call count** — a "once per pass" walk is only cheap
if a pass has many items.

What is left of the ceiling is not the number of walks but the walk itself,
and the two devices that could remove it are both spoken for: a
per-definition presence bit (`CardMemo`'s `dispatch_bits` family) keeps all
23 derefs and 23 atomic loads — `dispatch_board_scan` measures that shape at
226 Ir, against the ~267 Ir this leg costs when it runs — and a battlefield
zone memo is `(-62)`'s refuted class (worth the zone's *write* rate, and the
battlefield's `DerefMut` fires on every tap).

The original sizing, kept because the shape reading is still right:
`blocker_self_block`'s Void Winnower leg is a whole-board
`static_abilities` walk taken on half of all blockers.** Its only gate is
`blocker.definition.cost.cmc().is_multiple_of(2)`, and **zero is even**, so
every colourless and every even-cost blocker pays it;
`OpponentsCantBlockWithEvenMv` is one card. The shape wants a block-side twin
of `attack_static_scan` threaded through `blocker_can_block_anything` (whose
doc already says it is asked once per blocker, not per pair), and the
signature change reaches `blocker_can_block_attacker`, which is `pub`.
**Probe it first** with `(-78)`'s device — replace the walk with `false`,
build, read `cube` — because an early-exiting `any` over ~23 cards is exactly
the shape the ninety-second pass's concurrent third found to be a third of
its arithmetic.

**(c) `blocker_pair_block` HAS THE SAME SHAPE AS (a) AND THE ARITHMETIC SAYS
LEAVE IT.** 62,366 calls / 14,267,772 / 0.475 % of `cube`, 228.8 Ir of self.
Counted by hand at `2bbf26f9`: three passes over `atk_kws` (the
`CantBeBlockedIfDefenderControls` `any`, `block_barred_by_protection_filter`,
and the landwalk-family `for kw in`) and five over `blocker_kws`
(`CantBlockUnlessMoreCreaturesThanAttacker`, `blocker_matching_restriction_bars`,
`CantBlockCreatureType`, `CantBlockPowerAtLeastOwnToughness`,
`CantBlockUnlessMoreLandsThanAttacker`) — **eight, against (a)'s
twenty-two.** (a) measured ~6.3 Ir per removed pass, so fusing six of these
is ~38 Ir a call, **~0.08 % of `cube`**, and it costs restructuring a
rules-critical function plus a signature change on
`blocker_matching_restriction_bars` (two callers) to keep its predicate
single-sourced. **Count the passes before applying (a)'s device: the win is
linear in the pass count and this one is a third of the site that paid.**

**(d) NOT NEW, twice over.** `granted_abilities_of_inner`'s residue is
93,770 calls / 15,319,136 / 0.51 % of `cube`, and the requirement-walker half
of it is `(-83)`'s `granted_abilities_of` row — read that entry, not this
one. And `Unfreeze::drop` (207,446 calls / 10,235,176 self) has one callee,
`Arc::drop_slow` at **205,032 calls / 39,365,409 Ir inclusive, 1.31 % of
`cube`** — freeing the `ComputedPermanent`s the scope memoized, i.e.
`(-80)`'s pool, built and refuted with a ledger. Roughly one `Arc` per scope,
so a "skip `end_of_scope` when the scope memoized nothing" bit would fire on
almost nothing.

**(-82) THE BOT'S HAND SWEEP IS 5.8-6.9 % OF EVERY POOL AND IT IS TWO
QUESTIONS: "CAN I PAY FOR IT" AND "WHAT WOULD I AIM IT AT".** Read at the
ninety-second tip, all three pools, one binary. Nothing in this file has ever
sized `cast_candidates`, and its two big callees are 75-84 % of it.

```text
                        fixed 1,015,163,661   cube 3,086,949,158   sealed 3,037,627,253
  bot::cast_candidates    70,042,439  6.90 %   178,143,586 5.77 %   209,475,243  6.90 %
    <- sim_spell_action_inner  3,704            7,004                8,638   calls
    <- main_phase_action_with  3,506            5,474                8,018   calls
  can_afford_in_state_with 24,055,267 2.37 %    87,166,167 2.82 %    80,122,729  2.64 %
                          12,422 calls          32,360               47,146
  auto_targets_for_effect_ 32,124,278 3.16 %    47,053,286 1.52 %    96,175,091  3.17 %
    all_slots               2,878 calls          3,492               11,098
                            ------              ------               ------
  the two, as a share of the sweep    80 %        75 %                 84 %
```

**The affordability filter is not a per-card cost; it is a per-tick board
walk that a `OnceCell` amortises over two cards.** Of
`can_afford_in_state_with`'s subtree, `OnceCell::try_init -> bot::available_mana`
is **16,827,383 / 57,308,860 / 38,460,654 Ir (1.66 / 1.86 / 1.27 % of the
pool)** over **5,900 / 12,110 / 15,442 builds** — and on `fixed` that is
5,900 builds against 12,422 checks, i.e. **2.1 checks an amortisation**. The
per-check body is cheap (`extra_cost_for_spell_over`,
`cost_reduction_for_spell_full_over`, `colored_spell_tax_for_spell_over`,
`relax_cost_colors_known`, `can_afford_from` — 13,648 calls each on `fixed`,
5.65 M between them). **Anything aimed here has to remove *builds*, not
checks**, and the two callers that share a `SweepMana` inside one tick already
do: 7,210 sweeps produce 5,900 builds. The 3,704 that come from
`sim_spell_action_inner` are on simulation clones that tap their own mana, so
they cannot share the real board's build. Read (-41) before proposing a filter
*inside* `available_mana`: that was built by two sessions and rejected nothing.

**`auto_targets_for_effect_all_slots` is 11,162 / 13,475 / 8,666 Ir a call and
the shape is one requirement question per battlefield permanent per slot.** On
`fixed`, 2,878 calls make 57,314 `evaluate_requirement_static_hinted` calls —
**19.9 a call, i.e. the board**. Three things about it are already right and
should not be re-taken:

* **the affordability filter runs first.** `cast_candidates` does
  `can_afford_in_state_with` before it targets, so this is not (-51)'s
  "reject earlier in the bot" a second time.
* **the legality check is correctly second.** `is_legal_bf` is
  `evaluate_requirement_static_on(..) && check_target_legality(..)`, and the
  `&&` means only 4,806 of `fixed`'s 57,314 candidates reach
  `check_target_legality_with_source` (245 Ir). The requirement rejects 92 %
  first, which is the cheap-question-first ordering already applied.
* **`computed_permanent` is behind both filters.** 292 calls on `fixed` from
  the whole targeter — the `.map(|c| … power …)` runs only on survivors.

**What is left is the *number* of targeting calls — and the shape that would
remove them is nearly refuted by the probe counts in the same dumps, which is
why it is written down here rather than built.** The dry run at the pick site
is already lazy (the comment in `cast_candidates` says a typical tick probes
one or two candidates in descending score order); **target selection is not**,
because the target is a field of the `GameAction` the scorer ranks. So the
question is how many targeted candidates are built and never probed — and the
probe census bounds it:

```text
                                       fixed        sealed
  cast_candidates calls                7,210        16,656
  auto_targets_for_effect_all_slots    2,878   0.40  11,098  0.67  a sweep
  probes (pay_census::in_probe) from
    main_phase_action_with             1,932         6,572
    sim_spell_action_inner             1,452         4,630
                                       -----        ------
                                       3,384   0.47  11,202  0.67  a sweep
```

**A sweep builds 0.40 targeted candidates and runs 0.47 probes on `fixed`, and
0.67 of each on `sealed`.** The candidate set a tick produces is already about
one, so "probe one or two of many" is not what is happening — there is
almost nothing to defer. The bound is not tight (a probe may be of an
untargeted candidate, so this does not *prove* the waste is zero), and the
counter that would close it is one pair of tallies at the two sites. **But do
not build the deferral on the strength of the 3.16 % row**: price the waste
first, and the two ratios above say to expect it to be small.


**`(-82)`'S `available_mana` HALF IS REFUTED, AND THE COUNTS DO IT WITH NO
BUILD SPENT** (pass 97, read off the ninety-seventh tip's dumps). The entry
put the weight of itself here — "can a tick's build be shared with more of the
tick?" — and the answer is that it already is:

```text
cube, --games 6 --threads 1 --seed 1
  available_mana builds                      12,240
  cast_candidates calls                      12,478   -> 0.98 builds a call
  where the builds are (--separate-callers=3):
    can_afford_in_state_with <- cast_candidates      11,206   91.6 %
    can_afford_in_state_with <- pick_defensive_removal   282
    main_phase_action_with <- next_action_inner          260
    can_afford_in_state_with <- pick_stack_response      152
    max_affordable_x_for_def <- cast_candidates          130
  and where cast_candidates' calls come from:
    sim_spell_action_inner <- simulate_attack_outcome_once  7,004
    main_phase_action_with <- next_action_inner             5,474
```

**One build per `cast_candidates` call, and the two halves of that call count
are both unshareable for different reasons.** The 5,474 real-board calls
*already* take `main_phase_action_with`'s cell through the `shared:
Option<&SweepMana>` parameter — the build is attributed to `cast_candidates`
only because the `OnceCell` initialises lazily at the first affordability
check, which is inside it. The 7,004 simulation calls sit inside
`simulate_attack_outcome_once`'s `while !g.is_game_over()` loop, which
`dry_run`s a mutation into `g` on **every iteration** (24,348
`sim_spell_action_inner` calls a `cube` run, 12 per simulation, 3.5 of them
reaching `cast_candidates`), so two consecutive calls are on different boards
by construction and no cache can span them.

**What is left is 694 builds — 5.7 %** — in the priority-response chain
(`pick_defensive_removal` / `pick_stack_response` / `main_phase_action_with`
each build their own where `next_action_inner` could hold one for the whole
tick). At `cube`'s 55.7 M / 12,240 = 4,552 Ir a build that is ~3.2 M, **0.11 %
of the pool**, for threading a `&SweepMana` through five picker signatures.
Filed as not worth it; the entry's targeting half is separately bounded above.

**The rule, and it is the third time this file has reached it from a
different direction:** a cache's value is `(asks - builds) x build cost`, and
`asks / builds` is a *ratio in the dump* — `cg_edges.py --callers` on the
cached function against `--callers` on its owner answers "is this already
shared?" before any theory about who could share with whom. The two censuses
this pass built (`(-88)`, `(-87)`) were the same question where the dump could
not answer it; here it could.

**Which puts the weight of this entry on `available_mana`'s 5,900 builds, not
on targeting.** That is the half with a *measured* amortisation ratio (2.1)
and a clear question — can a tick's build be shared with more of the tick? —
rather than an inferred one.

**Refuted here rather than left open: a shared `grant_scan`.** Three bot-side
callers build one per call — `available_mana` 5,900, `mana_source_table`
3,330, `main_phase_action_with` 2,442 on `fixed` (6.89 M / 0.68 % between
them; cube 26,546 calls / 0.58 %, sealed 30,552 / 0.48 %). Only the first and
third are inside one `SweepMana`'s tick, so the reachable share is 2,442 scans
= **~0.15 % of `fixed`, ~0.08 % of the other two** — and `mana_source_table`'s
cannot join them because it runs *during* payment, after taps have moved the
board. The standing rule applies as written: a board memo is worth the board's
write rate, and tapping is the write.

**THE TARGETING HALF IS PART-TAKEN AT THE NINETY-THIRD PASS, AND THE ENTRY
RANKED IT WRONG — read that before proposing anything else here.** The lever
was never the call count this entry spent its space bounding: opening the
function's callee table showed **three quarters of the row is not the
requirement walk** — the adapters (`call_mut`, 1.25 % of `sealed` on its own),
a per-slot `Vec` for one seat, a deep-cloned filter, and a slot-0 lookup run
twice. Two commits took the function's own row **-10.4 / -12.2 / -17.0 %** and
the program -0.290 / -0.206 / -0.573 %; the Baseline has both tables. **The
generalisable half: this entry sized a function from *outside* — total, calls,
Ir/call, and the counts of its two biggest callees — and every one of those
numbers was right while the conclusion drawn from them was wrong. One
`cg_edges.py --callees` on a 3 % row costs nothing and is the first thing to
run, not the last.**

What is left inside it, at `b635037f`: `evaluate_requirement_static_hinted`
(the semantics, and `(-83)`'s row), `check_target_legality_with_source`'s
17,792 `computed_permanent` calls at 1,106 Ir — a `perms` miss, **not** a
gather, and the freeze scope that looks like the fix is built and refuted in
the Baseline — and `has_hostile_ward`, which re-finds by id a permanent the
ranking loop is holding (8,838 calls / 1.23 M on `sealed`, so ~0.02 %: a
clarity fix with a number, not a candidate).

**(-83) THE REQUIREMENT WALKER IS THE THIRD-LARGEST ENGINE FUNCTION IN THE
PROGRAM AND ITS CALLERS ARE THREE DIFFERENT STORIES.** `evaluate_requirement_
static_hinted`, folded across its recursion level and its closure (see the
ninety-second pass's Baseline note on `'N` rows), is **2.17 % of `fixed`,
3.36 % of `cube`, 2.42 % of `sealed`** — behind only
`dispatch_triggers_for_events` and the gather on `cube`. Read at `636ddb17`.

```text
callers, by calls                     fixed 180,372   cube 1,468,226   sealed 982,538
  itself (recursion)                     28,550         957,198          529,590
  statics_granted_triggers_inner              -         142,094                -
  granted_abilities_of                        -         105,436                -
  auto_targets_for_effect_all_slots      57,314          41,064          147,846
  legal_targets_for_filter                    -           4,748           51,166
  auto_target_for_effect_avoiding           442          13,900           34,788
  call_mut / Map::try_fold / Map::fold   91,648          94,218          201,208
  gather_continuous_effects_inner             -          33,494                -
(a "-" is a caller below the table's tail: 330 Ir over 2 rows on `fixed`,
 7,036 over 9 on `sealed`, so it is "not this pool", not "not measured".)
```

**The pool split is the whole entry.** On `fixed` and `sealed` the grant
machinery does not reach the walker at all — the bench archetypes carry no
`GrantTriggeredAbility` static, and this was already true *before* the
ninety-second pass's gate, which is why that gate's `fixed` win came out of
the body and not out of this row — so on those two pools the walker is
**targeting**; on `cube` two thirds of it is
recursion and the external half is the grant machinery; on `sealed` it is
targeting again, at five times `fixed`'s volume. **A change aimed at this row
must say which of the three it is aimed at**, and a `cube`-only reading will
name the grant machinery for a cost that is targeting on the pool the training
loop plays.

**There is no prologue to gate.** The function is a bare `match req` with no
shared preamble — 71 Ir of self a call on `cube` is the jump table plus one
arm — so the (-79)/(-82) device does not apply. The lever is fewer calls, and
65 % of `cube`'s are the composite-requirement recursion, which is the
semantics. Its non-recursive callees are small and already memo-backed:
`has_keyword` 136,530 / 6.13 M, `creature_type_change_in_scope`'s closure
18,900 / 12.88 M, `type_scan_bits` 10,700 / 0.40 M.


**(-81) THE GATHER IS 71,930 CALLS AND 8.5 % OF `cube` INCLUSIVE, AND THE
CONTEXT CENSUS SAYS EVERY ONE OF THEM IS A SCOPE THAT HAD TO GATHER. READ
THIS ENTRY AS A CLOSED DOOR WITH THE ONE KEY NAMED, NOT AS A LEAD.** Whole
program 3,095,128,492 (`fixed` 1,021,019,898,
`sealed` 3,049,332,123), `--decks cube`, at `9c501f42`.

```text
callers of gather_continuous_effects_inner   71,930 calls, ~3,655 Ir each
  48,856   182,247,650   5.89 %  computed_permanent
  11,740    37,550,954   1.21 %  frozen_effects
   6,842    25,126,396   0.81 %  compute_permanents
   4,376    17,797,980   0.57 %  check_state_based_actions_into
     116       287,030           permanents_with_abilities_removed

callers of compute_permanents               18,258 calls
   6,842    68,897,482   2.23 %  combat_damage_computed   <- (-9)'s open half
   4,870    19,893,463   0.64 %  declare_blockers
   6,546    11,231,980   0.36 %  declare_attackers_banded
```

**`computed_permanent`'s 48,856 is a *scope* count, not a call count.** The
function is called 387,848 times; the freeze memo answers all but 48,856
without a gather, and 207,446 `with_frozen_layers` scopes plus 33,304
`freeze_layers_pop`s exist — so **most scopes never take a gather at all.**

**AND THE CONTEXT CENSUS IS RUN, WHICH IS THE ANSWER RATHER THAN THE
QUESTION** (`--separate-callers=4`, `cg_contexts.py`, same tip; 62,432 of the
71,930 in the 25 contexts below, 9,498 in 79 further ones):

```text
  6,956  computed_permanent <- resolve_combat <- advance_step <- pass_priority
  6,546  frozen_effects <- board_keyword_in_scope <- declare_attackers_banded
  5,830  computed_permanent <- resolve_combat <- submit_decision
  4,870  frozen_effects <- board_keyword_in_scope <- declare_blockers
  4,848  compute_permanents <- combat_damage_computed <- resolve_combat
  4,380  computed_permanent <- call_mut <- from_iter <- bot::pick_blocks_inner
  4,290  computed_permanent <- with_frozen_layers <- declare_blockers
  4,232  computed_permanent <- call_mut <- from_iter <- bot::pick_attacks_inner
  4,040  computed_permanent <- permanent_value_with <- eval_material_inner
  2,724  computed_permanent <- Map::next <- SmallVec::extend <- with_frozen_layers
  2,184  check_state_based_actions_into <- resolve_combat <- advance_step
  1,318  computed_permanent <- permanent_is_creature <- from_iter <- check_sba_into
  1,278  computed_permanent <- permanent_value_with <- eval_material_inner
  1,188  compute_permanents <- combat_damage_computed <- resolve_combat <- submit
    998  computed_permanent <- check_target_legality_with_source <- cast_spell
    974  check_state_based_actions_into <- resolve_top_of_stack_inner
```

**Every row is one gather per scope, and every scope is one per read of a
distinct game state.** The two biggest — `resolve_combat`'s 12,786 — are
`(-58)`, whose per-pair scopes were merged three-into-one and whose
across-the-batch sharing is *unsound* (a lifelink blocker's damage flips an
"as long as you've gained life this turn" static mid-batch; the counterexample
is in the entry). The next two are `declare_attackers_banded` and
`declare_blockers`, and their `board_keyword_in_scope` gather is **already
shared with the `compute_permanents` in the same `with_frozen_layers`** —
which is why neither shows a second context. The bot's two collects are one
scope per call of `pick_blocks_inner` / `pick_attacks_inner`.

**So there is no unscoped-read population to wrap, and this row is not a
caller-side fix.**

**AND THAT CONCLUSION IS HALF WRONG, WHICH IS THE ENTRY'S MOST USEFUL
SENTENCE.** "One gather per scope, one scope per distinct game state" makes
the count look irreducible — a scope is not something you can delete. But
**a scope only gathers if a read inside it asks for a computed view**, and the
top two rows fell by 12,920 in the same pass by presence-gating the *first
two reads* in `resolve_combat`'s pair scope (`fixed` -0.755 % / `cube`
-2.226 % / `sealed` -0.856 %; see the Baseline). **Before concluding a scope
has to gather, read its first few `&self` calls in source order and ask which
one asks for a computed view without a gate.** What is left after that is what
`(-58)` says is left: either an invalidation *at* the mutation (the board
epoch, `(-18)`, built and refuted) or an incremental gather.

**Do not re-run this census**; it cost one `--separate-callers=4` run and this
table is its whole result. Its remaining rows in first-read order:
`declare_attackers_banded` / `declare_blockers` (`board_keyword_in_scope`),
the bot's two candidate collects, `permanent_value_with`,
`check_state_based_actions_into`.

**The first of those is BUILT AND REFUTED — `fixed` -0.026 / `cube` -0.044 /
`sealed` **+0.005** %, a pool split, see the Log.** `board_keyword_in_scope`
asks `frozen_effects` before its presence gate, and inside a scope that *is*
the gather, so the shape looks identical to the pair-scope fix. It is not:
both callers go straight on to `compute_permanents` in the same scope, so the
gather is moved and the gate is paid for nothing. **The pair-scope fix worked
because every other read in that scope was already gated. Before gating a
scope's first read, check what the scope does *after* it.**

**`combat_damage_computed` is (-9)'s remaining half re-sized and it has
doubled since that entry** — 3,274 calls / 0.71 % there, 6,842 / **2.23 %**
here, 10,070 Ir apiece. It is one gather plus a per-participant layer pass,
taken once by `resolve_first_strike_damage` and once by `resolve_combat`, and
(-9)'s two shapes are unchanged: hand the first-strike step's set forward
(unsound as written — CR 510.2 is a fresh assignment and damage is dealt in
between), or have `advance_step`'s first-striker gate and the resolver share
one freeze scope. **The second is the one to price**, and note (-58) before
building it: sharing *one gather* across the damage batch's pair loops was
built and refuted with a counterexample, but that entry is about the loop
inside one step, not about the two steps' two calls.

**(-80) THE ALLOCATOR IS 10.4 % OF `fixed` OVER 575,795 ALLOCATIONS, AND
HERE IS WHERE THEY COME FROM.** PERF has said for ten passes that the
allocator family "is the one thing no pass has aimed at directly" and never
carried the census that would let one. This is it, at `9f993c48`, and the
headline is that **the biggest single context is already minimal** — read that
before proposing anything here.

```text
bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1 --decks fixed,
callgrind, profiling-fast --no-default-features.  Total Ir 1,030,094,767.

  _int_free  3.43 %   malloc 2.60 %   free 2.18 %   _int_malloc 1.83 %
  _int_free_merge_chunk 0.40 %   realloc 0.35 %          = 10.79 %
  __memcpy_avx_unaligned_erms                             = 2.04 %
  575,795 `__rust_alloc` calls  ->  ~180 Ir an allocation, freed included

top contexts (--separate-callers=5, cg_contexts.py __rust_alloc):
  30,396  computed_permanent <- call_mut <- from_iter <- bot::pick_blocks_inner
                                                      <- with_frozen_layers
  22,820  finish_grow <- grow_one <- advance_step <- pass_priority
  18,144  clone_from_ref_in <- make_mut <- cast_spell_with_convoke <- cast_spell
  15,142  clone_from_ref_in <- make_mut <- activate_ability_inner
                            <- activate_ability <- auto_tap_for_cost_inner
   9,194  finish_grow <- grow_one <- push_mut <- run_effect <- resolve_effect
   9,076  from_iter <- cast_spell_with_convoke <- cast_spell
   8,506  finish_grow <- grow_one <- push_mut <- activate_ability_inner
   7,904  finalize_cast <- cast_spell_with_convoke <- cast_spell
   7,596  finish_grow <- grow_one <- dispatch_board_scan
                      <- dispatch_triggers_for_events
   6,660  CowBox::push <- finalize_cast <- cast_spell_with_convoke
```

**Row 1 is a refutation, not a candidate.** 30,396 allocations is one
`Arc<ComputedPermanent>` per computed read — `Printed`/`PrintedList` (the
eighty-fourth pass) already keep the struct's five list fields at *zero*
allocations unless a layer actually overrides one, and the freeze scope's
`perms` memo already serves the repeats. ~4 distinct permanents a
`pick_blocks_inner` collect, one `Arc` each. There is nothing left in it short
of not allocating the `Arc`, which the memo's shape requires.

**AND THE ONE THING SHORT OF THAT WAS BUILT AND REFUTED AT THE NINETY-FIRST
PASS — `fixed` +0.511 % / `cube` +0.049 % / `sealed` +0.357 %, ON 84,558
FEWER ALLOCATIONS.** The shape's requirement is the *handle*, not a fresh
allocation for it: an entry whose only handle is `perms`' at `end_of_scope`
is uniquely owned (`Arc::get_mut`), so a `Vec<Arc<ComputedPermanent>>` pool on
`LayerFreezeState` can hand it to the next scope and `computed_permanent`
overwrites it in place. It works — cube's `__rust_alloc` count goes
1,588,542 -> 1,503,984, **5.3 % of every allocation the program makes** — and
the program is slower on all three pools. The whole ledger, `cube`:

```text
  removed, the allocator            added, the bookkeeping
   _int_free            -4.30        end_of_scope             +11.34
   malloc               -4.21        dispatch_triggers_for_events +4.14
   Unfreeze::drop       -4.28        drop_in_place<ComputedPermanent> +3.22
   Arc::drop_slow       -3.83        computed_permanent        +2.00
   free                 -3.30        drop_in_place<LayerFreeze> +1.51
   _int_malloc          -2.54        realloc                   +1.22
   __rdl_alloc          -0.76        _int_realloc              +0.85
   malloc_consolidate   -0.67        finish_grow               +0.83
                       ------                                 ------
                       -23.89 M                               +25.11 M
```

**The reclaim loop runs on every scope exit and the allocations are not.**
`end_of_scope` stopped being a `clear()` the compiler folded into the guard's
drop and became a named 72.2-Ir row over **156,944 calls**, against 84,558
allocations recovered — 1.86 loops per allocation, 134 Ir of loop for 283 Ir
of allocator. That is the whole trade on `cube` and it is nearly a wash;
`fixed`, where scopes are as frequent and boards are smaller (74,926 loops for
25,894 allocations, 2.89 to one), is where it loses outright. **Count the
reclaim points against the allocations reclaimed before building a pool.**

**THE SINGLE-SLOT VARIANT IS NOW BUILT AND IT LOSES TOO — `fixed` +0.137 % /
`cube` +0.097 % / `sealed` +0.097 %, and the reason is structural, not
arithmetic.** Built at `2b16088a`: an `Option<Arc<ComputedPermanent>>` on
`LayerFreezeState`, filled at `end_of_scope` from `perms.pop()` when
`Arc::get_mut` says the entry is unique, drained by an `arc_computed` that
overwrites it in place. It removes all three of the `Vec` pool's own costs —
no pool `Vec` to allocate on a `GameState` clone, no reclaim *loop*, one
branch and one `get_mut` at each end. The ledger:

```text
base 2b16088a, callgrind, profiling-fast --no-default-features
  fixed     938,100,211 ->   939,382,280   +0.137 %
  cube    2,795,139,482 -> 2,797,849,468   +0.097 %
  sealed  2,783,686,742 -> 2,786,395,475   +0.097 %

cube, top movers                    __rust_alloc  1,426,336 -> 1,407,250
  arc_computed          +9.06 M       165,844 calls @ 54.6 Ir
  end_of_scope          +6.61 M       150,732 calls @ 43.9 Ir
  computed_permanent_hinted -4.57 M   } code moved into the two rows above,
  Unfreeze::drop            -4.28 M   } not saved
  the allocator family      -4.37 M   (_int_free, malloc, free, _int_malloc,
                                       drop_slow, __rdl_alloc, consolidate)
  drop_in_place<ComputedPermanent> +0.62 M, drop_in_place<LayerFreeze> +0.40 M
```

**316,576 bookkeeping calls for 19,086 allocations — an 11.5 % hit rate, and
that is the whole finding.** `Arc::get_mut` fails on seven of every eight
scope exits, because `computed_permanent` pushes one handle into `perms` and
**returns the other to the caller**, and the caller is usually
`compute_battlefield`-shaped: it collects the handles inside
`with_frozen_layers` and returns the collection *out* of the scope. The scope
that computed the `Arc` does not outlive it. **A pool can only recycle a
handle whose lifetime the pool's owner bounds**; check that before counting
the allocations, because the arithmetic above (150,732 scopes x ~217 Ir a
pair, against 15.7 M of bookkeeping, predicts -0.57 % on `cube`) is right and
the premise is wrong. `(-27)`'s allocation half is closed: both variants
built, both refuted, and the last named one refuted on a property no tuning
of the probe can move.

**Two of the added rows are the device paying for itself in its own coin.**
`drop_in_place<LayerFreeze>` +1.51 M, `realloc` +1.22 M and `finish_grow`
+0.83 M are **the pool's own `Vec`** — `Clone for LayerFreeze` is `default()`,
so every one of the run's 22,684 `GameState` clones allocates and grows a
fresh pool to avoid allocating. A single-slot `Option<Arc<_>>` pool removes
all three (~4.4 M) and shrinks `end_of_scope` to one branch and one
`get_mut` (~3 M rather than 11.3), against a recovery of at most the
156,944 one-per-scope entries. **That variant is arithmetic away from a
~0.25 % `cube` win and nothing on `fixed`, which is why it is written down
rather than taken**: `dispatch_triggers_for_events`' +4.14 M is a codegen
shift of the same size as the whole remaining margin, and it is not
predictable from the diff. The third data point for the class the
eighty-ninth pass opened (`1be9d90a`, "removing 8,940 rehashes and 8,316
allocations made the program slower"): **an allocation count is not a cost.**

**ROW 2 IS TAKEN — `fixed` -0.256 % / `cube` -0.105 % / `sealed` -0.200 %,
and 575,795 allocations -> 555,641.** The fix is a `Vec<GameEvent>` scratch
slot on `GameState`: `pass_priority` `mem::take`s it instead of allocating,
and the callers that *discard* the events hand the buffer back through
`recycle_events`. Four sites do (`bot::sim_step`'s PassPriority arm — 34,298
of the run's 66,612 `perform_action_inner` calls — `simulate_through_combat`,
the `recommend` self-play driver, and `bot`'s policy loop); a caller that does
not is unaffected, because `take` on an empty slot yields `Vec::new()` and the
first push allocates exactly as before. **-20,154 allocations against the
22,820 this entry predicted: 88 % captured**, the rest being `accept_on`,
which clones per probe and so never reuses.

**And the removed allocations cost 131 Ir each, not the 179 the family
average implies** — 2,639,952 Ir over 20,154 of them. The average is over
every size class; a short `Vec<GameEvent>` is at the cheap end. Divide the
family total by the call count for a *ranking*, not for a price.

**What row 2 said before it was taken, kept because the reasoning is the
reusable half — it is *not* a `with_capacity` fix.**
`pass_priority` hands `advance_step` a `vec![]` and `advance_step` pushes
`GameEvent::StepChanged` unconditionally — so the growth count (22,820) is
**exactly** the call count (22,820), one first-push allocation per step
advance and no second growth. Reserving capacity moves that allocation
earlier; it does not remove it. What removes it is **recycling the buffer** —
a scratch `Vec<GameEvent>` on `GameState` that `pass_priority` `mem::take`s
and the drain point hands back, which is (-75)'s device one level out. Worth
~0.40 % of `fixed` and it costs a change to the event plumbing's ownership,
which is why it is written down rather than taken.

**Rows 3 and 4 are (-74) with a name: 33,286 of the run's 60,524 `Arc`
deep copies are the cast and activate pipelines** unsharing after the
`perform_action` checkpoint clone. A deep copy is 593 Ir and carries 1.3
allocations, 1.8 `Vec::clone`s, 1.4 `SmallVec::extend`s and 0.5
`RawTable::clone`s. The lever is fewer *shared* handles at the moment of the
first write, not a cheaper copy.

**(-79) `keyword_grant_in_scope`'S BATTLEFIELD LEG IS `fixed` -1.75 % /
`cube` -2.25 %, MEASURED BY DELETION, AND THE MEMO ROUTE TO IT IS CLOSED.**
Eight callers reach `card_can_grant_keyword` **515,328 times over six `fixed`
games** and 1,802,138 over six `cube` ones — `card_keyword_possible`,
`damage_prevented_by_protection`, `apply_prevention_shields_with`,
`advance_step`, `process_cumulative_upkeep`, `board_keyword_in_scope`,
`do_untap`, `tap_ability_summoning_sick`. The per-card body is already two
memo-backed loads (eighty-seventh pass row (3), 31 Ir a card from 43.6), so
what is left is the *number of cards*, one whole board per question.

**A board-level OR of the per-card gates was built twice and reverted twice**
— see the eighty-ninth pass's Baseline entry for both numbers and for the
rule (**a zone memo is worth the zone's write rate, not its read rate**; the
battlefield's `DerefMut` fires on every tap). Do not rebuild it. What is left
is the **number of questions**, and it is not obviously reachable: the three
biggest callers each ask *once per event*, not in a hoistable loop —
`apply_prevention_shields_with` once per damage event (Absorb),
`damage_prevented_by_protection` once per damage event (protection),
`activate_ability_inner` once per activation (`CantActivateTapAbilities`).
**THE ONE UNTRIED SHAPE IS TAKEN AT THE NINETIETH PASS — `fixed` -0.966 % /
`cube` -1.325 % / `sealed` -0.939 %, and it is 55 % of the ceiling above.**
The finer per-definition bit this entry proposed (`grant_bits::ANY_GRANT`,
"this definition can grant **some** keyword") shipped; see the Baseline block.
The entry priced it against the eighty-eighth rule — *count the work items the
bit elides*, here five container walks — and that framing **understated it**,
because the win is not the per-card body at all: it is that the walk behind the
gate went from 1,802,138 runs to 49,974. `card_can_grant_keyword`'s 1.60 %
row is gone from the profile and every other row on `cube` is byte-identical.

**What is left of this entry is the residue and it is small.** The deletion
ceiling (`fixed` -1.75 % / `cube` -2.25 %) is the whole battlefield leg,
including the ~2.3 Ir a card the gate still costs over 1.8 M visits and the
49,974 walks that survive. Both of those are the *number of questions* again,
which the paragraph above says is not obviously reachable, and the two
board-level OR memos stay in the do-not-rebuild list.

**(-78) AN ITERATOR ADAPTER CHAIN IS A PER-ELEMENT BRANCH. THE `flat_map`
HALF IS **SWEPT** AT THE EIGHTY-SEVENTH PASS'S CONCURRENT HALF — SIX COMMITS,
`fixed` **-1.44 %** / `cube` **-1.13 %** BETWEEN THEM, AND `FlatMap::next` IS
OFF THE PROFILE AS A NAMED ROW (0.60 % OF `cube` -> NOTHING, 57,600 CALLING
CONTEXTS -> 0). WHAT IS LEFT IS `Chain`, AND IT IS A THIRD OF THAT.**

```text
  all_static_sources() is a visitor, command half gated   -0.839 / -0.552 %
  compute_permanent_pass' sorted view is two extends      -0.050 / -0.192 %
  declare_attackers_banded's three whole-board flat_maps  -0.278 / -0.230 %
  bot::cast_candidates' hand sweep is a loop              -0.271 / -0.179 %
  the last two combat flat_maps                           -0.049 / -0.038 %
                                            (fixed / cube, each on the one below)
```

The finding: `all_static_sources()` —
`battlefield.iter().chain(players.iter().flat_map(|p| p.command.iter()
.filter(...)))` — cost **~20 Ir a permanent** in a loop whose body does
*nothing* on `--decks fixed`, because the bench archetypes carry no
`static_abilities` entry. Half of `trigger_grant_sources`' entire row was the
iterator, not the work. See the Baseline for both commits and for the
two-line probe that priced it before the fix existed (**delete the rare leg
and measure**; it costs one build and answers the question outright).

**The selection rule, and it has three tests, all cheap:**

1. **Is the wrapped collection walked per element, or is the adapter set up
   per *call*?** `Chain` over 23 permanents 27,724 times is 0.84 % of `fixed`;
   `Chain` over a dozen continuous effects is 0.19 % of `cube` and 0.05 % of
   `fixed`. **A per-element cost splits by pool according to how long the
   collection is** — read both pools or you will read one of these as the
   other.
2. **Is the chained leg usually empty?** That is what makes the branch pure
   overhead rather than a fair share. Command zones, `granted_triggers_eot`,
   `eot_grants`, `removed_keywords` — all empty on almost every card.
3. **What replaces it?** A closure visitor when the body is long (the
   monomorphized closure inlines, so the hot leg becomes the plain slice loop
   it was), two `extend`s when the target is a `Vec`, a presence-gated second
   loop when the leg is rare. **A `continue` in the body becomes a `return`
   when it moves into a closure** — check for one before quoting a line count.

**WHAT IS LEFT — AND THE FIRST LIST HERE RANKED THE WRONG THING, WHICH IS
ITSELF THE ENTRY'S TEST 1.** The rows below are the *functions* the remaining
chains sit in, and a function's row is not its chain's cost: nearly all of
them turn out to be a chain over a short list **per call**, not per element of
a board walk.

```text
  2.38 %  check_state_based_actions_into  stack.rs:5412 — REFUTED BY READING:
          the chain is inside `.map(|c| …)` over the *dying* cards, so it runs
          per death, not per card per sweep
  1.47 %  activate_ability_inner          BUILT AND REFUTED — nine whole-board
          `battlefield.iter().flat_map(|c| &c.definition.static_abilities)`
          walks, deleted outright (`battlefield[..0]`), read `fixed`
          **-0.121 %** / `cube` **-0.167 %** *in total*. That is the ceiling
          on any gated-scan rewrite of all nine, against ~100 lines of the
          `cast_static` / `sba_board_scan` bit-per-family device. Most sit
          behind `!ability_is_mana` or a source-type gate, and the function's
          50,058 calls are dominated by mana taps that skip them.
  1.29 %  fire_combat_damage_triggers     REFUTED BY READING: combat.rs:5320's
          double chain is inside `if let Some(c) = dealer`, i.e. once a call
  1.17 %  evaluate_requirement_static_hinted  eval.rs:3770-3836, five
          `target.iter().chain(additional_targets.iter())` over 1-2 targets
  0.39 %  do_untap                        stack.rs:2965-3228, five sites, and
          these *are* whole-board — the only unread ones of the shape
  0.24 %  cost_reduction_for_spell_full_over  takes the iterator as a
          **parameter** (`CostStaticSources`), so the fix is a signature change
  0.12 %  chosen_type_etb_counter_specs   `continue`s the *outer* loop and
          wants test 3's `return`
```

**So the entry's own test 1 is the whole selection rule, and it is cheaper to
apply than to skip**: every site that shipped wrapped a **whole-board walk
executed per call**, and every candidate that failed failed on that question
alone. `dispatch_triggers_for_events` (5.93 %) carries a *triple* chain at
mod.rs:17792 and is left for the same reason plus one more: its four legs are
already behind an all-empty `continue` gate, and its body is long enough that
moving it into a closure is a refactor of the hottest function in the
simulator.

**THE INSTRUMENT IS THE REUSABLE HALF, AND IT COSTS A CALLGRIND RUN WITH NO
REBUILD.** `FlatMap::next` and `Chain`'s monos are **named rows in the self
table**, so a program's adapter cost can be read off directly rather than
inferred; `--separate-callers=3` plus `cg_contexts.py <dump> FlatMap` then
ranks the *sites*. That is two minutes, and it is what found the four that
shipped after this entry's first list ranked function rows and got three of
five wrong. Re-run it before proposing another one.

**WHAT IS LEFT — `Chain`, 25,480 contexts against the `flat_map`s' 57,600, at
`3c0042db` on `--decks cube`:**

```text
  11,762  bot::ward_gate_ok <- Vec::retain <- bot::cast_candidates
   4,456  do_untap <- advance_step <- pass_priority   (five whole-board sites)
   3,774  bot::pick_prepare_response <- sim_spell_action_inner
   2,576  dispatch_triggers_for_events <- perform_action_inner
     994  bot::pick_prepare_response <- next_action_inner
     718  bot::ward_gate_ok <- main_phase_action_with
```

None of them is a whole-board walk of the kind that paid: `ward_gate_ok`'s is
a `find_card_anywhere` chain per candidate (0.09 % of `cube` at its largest
context), `do_untap`'s five run once a turn. Expect a tenth of what the
`flat_map` sweep read, and read the self rows first.

**(-77) THE MEMO DEVICE THE EIGHTY-SEVENTH PASS PROVED HAS FOUR MORE
CALLERS, AND THEY ARE 4.75 % OF `cube` BETWEEN THEM.** `sba_board_scan` was
1.59 % and is 0.75 %; the shape it used — *a whole-board walk asking a
question that is a pure function of each permanent's `CardDefinition`* —
repeats in the self table, at `--decks cube`, post-(2):

```text
  1.87 %  card_can_grant_keyword     <- TAKEN, row (3); -0.283 % of `cube`
  1.33 %  dispatch_board_scan        <- TAKEN at the 88th, `cube` -0.317 %;
                                        the 87th's refutation of it is below
                                        and is superseded, not deleted
  1.30 %  card_type_change_unscoped  <- TAKEN, row (4); -0.405 % of `cube`
  0.54 %  creature_type_change_in_scope  <- REFUTED with the row below
  0.49 %  land_type_change_in_scope      <- REFUTED with the row below
  0.79 %  trigger_grant_sources      <- (-60); CLOSED by the visitor rewrite
                                        (87th concurrent) plus the 88th's gate
  0.76 %  granted_abilities_of       <- REFUTED at the 88th; `fixed` +0.024 %
```

**THE LAST ROW IS REFUTED TOO, AND THE ENTRY IS CLOSED.**
`granted_abilities_of`'s prologue computes six booleans — Myr Welder,
Necrotic Ooze, Marvin, Kraj, Safehouse, Conspicuous Snoop — in one
`static_abilities` walk, every one an exact function of the definition (they
match printed variants, no wrapper peel), over ~272 k calls a `cube` run.
Six bits with their own valid flag (bit 41, per `f264ba27`): **built,
measured and reverted at the eighty-eighth pass.**

```text
Base e398874c, same instrument.
  fixed   1,057,644,796 -> 1,057,894,717   +0.024 %
  cube    3,212,823,684 -> 3,212,970,309   +0.005 %
  sealed  3,157,096,383 -> 3,157,450,866   +0.011 %
```

**And it is the eighty-eighth pass's own distinction failing its second
test.** The bits *are* the answers — nothing iterates afterwards — but the
walk they replace is **one** `for sa in &me.definition.static_abilities {}`
on a definition with no statics, i.e. one length check, and a new family's
miss path plus a load does not beat that. What made `dispatch_board_scan` win
is not "the bit is the answer": it is that its per-card body does **four**
things (an `equipped_bonus` load, a `static_abilities` walk, a `station`
walk, an `active_static` loop) and `bits == 0` skips all four. **The real
rule is the count of work items the bit elides, not their kind.** Six of the
device's rows have now been priced and the two that paid are the two whose
bodies were multi-part.

**AND SO IS THE OBVIOUS EXTENSION OF THE ROW THAT WON — `card_can_change_
creature_types` AND `card_can_change_land_types` ON THE SAME SLOT READ
`fixed` **+0.135 %**, `cube` **+0.145 %**, `sealed` **+0.097 %**, BUILT AND
REVERTED.** The two siblings of row (4) are the same three-walk shape minus
the keyword leg, and folding four more bits into `type_bits` (no new valid
flag, no new slot) made every pool worse — including `cube`, where the row
that shipped had won.

**The second half of that is the finding, and it is a cost this file had not
named: a memo slot's miss path is the sum of every family on it, and it is
paid by whichever consumer touches the card first.** `type_scan_bits` went
from two bits to six, i.e. **three times the miss**, and the misses did not
get rarer — so row (4)'s own win was eaten by the extension riding along with
it. **Adding a consumer to a warm slot is not free. Give a new family its own
valid flag, or measure the slot's miss again after widening it.** The first
half is the rule below, twice confirmed: the walks removed here are
`static_abilities.iter().any()` and `station.iter().any()` on definitions
that have neither — two length checks.

**AND THE EIGHTY-NINTH PASS BUILT THE PRESCRIPTION ABOVE — A `change_bits`
FAMILY WITH ITS OWN VALID FLAG (bit 58, payload 59-62) — AND IT IS FLAT.**
`fixed` **+0.031 %**, `cube` **-0.040 %**, `sealed` **+0.015 %**; built,
measured, reverted. So the shared slot explained the *loss* and there was
never a *win* underneath it:

```text
base dc478735, profiling-fast --no-default-features, --games 6 --seed 1
  fixed   1,059,426,578 -> 1,059,751,948   +0.031 %
  cube    3,217,542,883 -> 3,216,244,748   -0.040 %
  sealed  3,163,675,366 -> 3,164,163,014   +0.015 %
  creature_type_change_in_scope's walk  17,847,758 -> 17,672,056   (-1.0 %)
  land_type_change_in_scope's walk      16,008,238 -> 15,707,964   (-1.9 %)
  CardDefinition::change_scan_bits (the new miss path)  1,946,496
```

**The two rows are 1.05 % of `cube` between them and the memo moves 1.5 % of
them, because the per-card body was never where they spend.** The closures
run **27,794** and **45,394** times at **642** and **353 Ir**, i.e. ~20 cards
at ~22 Ir apiece, and those 22 Ir are the walk's own machinery — an `Arc`
deref per `CardInstance`, the iterator, the `any` closure — not the
`Option` check and two length checks the bit replaced. The miss path costs
**more** than the whole saving on its own. **This is the eighty-eighth
pass's distinction read from the losing side one more time, and it now has
its sharpest statement: price the *walk*, not the *function's row* — a row
that is 0.55 % of the program can be 98 % iteration.** The lever on these
two is fewer walks (they are already gated per freeze scope, at a 13 % miss
rate over ~207 k scopes), and there is nothing left in the body.

**`dispatch_board_scan` WAS REFUTED AT THE EIGHTY-SEVENTH PASS AND TAKEN AT
THE EIGHTY-EIGHTH — the refutation below is kept verbatim because its
*measurement* stands and only its conclusion moved.** The eighty-eighth
pass's Baseline block has the two differences: bits that *are* the answer
(three of the four facts are booleans, so the whole loop body is skipped, not
just a walk gated) and four sibling gates whose own rows get worse while the
program gets better. Read the two entries together before pointing the device
anywhere else — the selection rule below is right about a bit that only gates
a walk, and wrong about one that replaces the body.

**As found: `fixed` **+0.106 %**, `cube` -0.067 %, `sealed`
-0.078 %.** Three bits (`SUPPRESS_DIES`, `STRIPS`, `GRANT_TRIGGER`) replaced
the per-card `static_abilities` walk, the `station` walk and the ungated
`active_static` loop; the suite's own `debug_assert!` against the four walks
it fuses passed, so the bits were right and the trade was not.

**And it is the entry's selection rule, arrived at from the losing side: the
memo pays for the walk it replaces, and this walk is over ONE list that is
empty on most cards.** Compare what the three winning rows removed per
permanent — `sba_board_scan` five list walks and ~25 field reads (~113 Ir),
`card_can_grant_keyword` five pointer-chased loads into a large
`CardDefinition`, `card_can_change_card_types` the keyword list plus the
station bands plus every printed static — against
`dispatch_board_scan`'s `for sa in &def.static_abilities {}` on a definition
with no statics, which is **one length check**. A memo load, a mask and three
tests do not beat that, and on `--decks fixed` (which carries no
`GrantTriggeredAbility` at all, per CLAUDE.md) there is nothing else for them
to win back. **Before pointing the device at a row, price the walk on the
card that answers "no", not on the card that answers "yes".**

**`card_type_change_unscoped` was not on the list when it was written, and
the reason it belongs is a refutation this file already carried.** The
fifty-ninth pass measured folding its battlefield leg **into
`sba_board_scan`** at `sos` +0.255 % / `fixed` +0.295 %, because "the
standalone `any` short-circuits per card, while a scan bit runs
`static_effect_changes_card_types` for every static ability of every
permanent". **That is an argument about *eager* computation and the memo is
lazy**: the bits are computed once per printing, not once per sweep, so the
per-card cost the refutation priced never happens. **Re-read every "a bit
costs more than the walk" refutation in this file against that distinction** —
it is the same dating as (-11)'s.

**The first row is TAKEN and its residue is the entry's own caution (c) in
practice.** Two bits (`grant_bits::SYNTH_KEYWORD`, `CONTAINER`) took
`card_can_grant_keyword` from 1.91 % to 1.60 % including the miss path —
i.e. **the memo answers the vanilla body, and the cards that carry a
container still walk it**. Rank the remaining three by *how many of their
per-card visits end in "no"*, not by their share: this row was worth taking
because 1.7 M calls a run mostly answer no, and `dispatch_board_scan` is the
next one where that is plausibly true.

**The device, in full, so nobody re-derives it:** put the answer's bits in
`CardMemo` (`crabomination_base/src/card.rs`), one atomic word on `CardData`,
cleared in `CardInstance::DerefMut`. There are spare bits — the SBA mask uses
8-29 and the colours 0-6 — and `clear()` stays one store however many
answers ride on it. The `debug_assert!` on the read is the audit and the
functional suite is the sweep.

**Three things to check before pointing it at one of these rows.** (a) The
question must be a function of the **definition only**; the instance half
gets ANDed at the call site (the SBA scan has four such flags and they are
the whole reason its mask is 22 bits rather than 18). (b) The caller must ask
it **more often than the permanent is written**, because a write clears the
whole word — `sba_scan_bits` hit ~97.5 % of ~466,000 lookups, and a caller
inside a mutation loop would not. (c) Its **miss path is now a function call
where it used to be inlined arithmetic**; `sba_scan_bits` cost 5.8 M of the
28 M it saved, so a row under ~0.4 % will not clear its own overhead.

**And (-61)'s own note says its per-card disqualifier is "five `is_empty`
loads", i.e. exactly one bit** — that is the largest of the four and the one
to price first.

**(-71) TAKEN at the eighty-sixth pass — `cube` -0.513 %, `sealed` -0.406 %,
`fixed` -0.012 %; 50,178 fewer allocations and 64,372 fewer `grow_one` calls
a `cube` run.** `gather_continuous_effects_inner`'s `sa_cards` is
`SmallVec<[(&CardInstance, u64); 16]>`. **The `union` feature on the
dependency is the entry**, not the inline buffer: without it the same change
is **+0.108 % on `fixed`**, because `SmallVecData` is an enum and every read
matches a discriminant. See the Baseline for the four-variant table and the
two shapes not to rebuild (capacity 4, and shadowing the buffer with a
`&[T]`).

**The generalisation's first instalment is TAKEN in the same pass** — nine
`Vec::new()` locals in `combat.rs` (four `ids`, two `tapped`, `pt_deltas`,
`life_paid`, `creature_damage`), `fixed` **-0.268 %**, `cube` **-0.240 %**,
`sealed` **-0.304 %**, `grow_one` 461,308 -> 428,190. All nine are small
elements (4-12 bytes), a handful of read sites each, and die in their own
frame. **What is left in `combat.rs` is the `events: Vec<GameEvent>`
accumulators**, and those are a different problem: they are *returned*, the
element is a large enum, and the fix for that family is threading one
`&mut Vec<GameEvent>` through the callees rather than an inline buffer —
`advance_step` already takes its events by value, so the shape half-exists.
Sized at ~0.8 % of `cube` across `advance_step`, `declare_blockers`,
`check_state_based_actions`, `resolve_combat`, `deal_combat_damage_to_target`
and `resolve_top_of_stack_inner`; it is a signature change across the engine
core and every `?` path that discards events has to be re-read, so it is not
a quiet-afternoon change.

**TWO MORE SITES WERE BUILT AND BOTH ARE REFUTED, AND BETWEEN THEM THEY GIVE
THE ENTRY ITS SELECTION RULE: COMPARE THE GROWTH COUNT TO THE *CALL* COUNT.**
A `grow_one` row says how often a buffer allocated; the SmallVec is paid on
every call whether or not that call would have allocated.

```text
                                     growths   calls    ratio   measured
  gather's `sa_cards`                 69,896   71,930    97 %   cube -0.513 %
  combat.rs's nine locals             33,118   (high)     -     cube -0.240 %
  statics_granted_triggers_inner      19,128  142,744    13 %   ~0 (-0.015 %)
  PlayerData's two per-turn tallies   22,930   (high)     -     cube +0.291 %
```

* **`statics_granted_triggers_inner`** returns its buffer, so a
  `SmallVec<[&TriggeredAbility; 4]>` is a 40-byte move on **142,744** calls to
  save an allocation on 19,128 of them. Measured at `fixed` -0.015 % — i.e.
  nothing — and reverted for being more machinery than a null deserves.
* **`PlayerData::spell_ids_cast_this_turn` and `spell_casts_this_turn`** are
  `clear()`ed per turn, so they look like they allocate once; what actually
  regrows them is the CoW deep copy, because `Vec::clone` hands back
  `capacity == len`. Inlining them removed all 22,930 of `finalize_cast`'s
  growths — the largest single `grow_one` row on `fixed` — and read `fixed`
  **+0.366 %**, `cube` **+0.291 %**, `sealed` **+0.322 %**. Same verdict as
  (-72) on `GameState::players`, and the same shape: a hot struct field read
  through `Deref` far more often than it is grown. **Both halves were measured
  separately** (the pair together read `fixed` +0.351 %), which is what says
  the seat tallies are the whole regression.

**AND THE `all_effects` RESERVE IS REFUTED A THIRD TIME, WITH THE TIGHTEST
ESTIMATE ANYONE HAS TRIED.** The gather sizes `all_effects` at `base.len() +
sa_cards.len()`, i.e. a *card* count, while `push_static_ability_effects`
emits per *ability* — so sizing it at `base.len() + sa_total` (the sum of
`def.static_abilities.len()`, which the prologue walk already reads, so the
count is free) is strictly closer to the truth. It reads **`fixed`
+0.461 %, `cube` +0.401 %, `sealed` +0.373 %**. `ContinuousEffect` is a large
struct, so a handful of extra slots is a kilobyte of allocation on every one
of 71,930 gathers, and the emitted count is far *below* the ability count
because most statics emit nothing. **Same verdict as the fifty-fourth pass's
`+ battlefield.len()` (+1.54 %) and for the same reason; the entry's demand
for an "exact" size means exact in *emitted effects*, which nothing cheap
knows.** Do not rebuild this.

**The rest of the worklist, and it is the allocation table's `grow_one`
half.** `declare_blockers` (36,526 growths / 7,966,197 Ir),
`finalize_cast` (22,930 / 7,335,377), `advance_step` (37,670 / 4,522,660),
`check_state_based_actions` (26,942 / 4,718,160) and `dispatch_board_scan`
(29,474 / 3,544,522) are the same shape — a `Vec::new()` local accumulator
whose first `push` is the allocation — at half the size each, ~1.2 % of
`cube` between them. Two cautions from this entry before taking any of them:

* **Check the pool before pricing the row.** The `fixed` half of (-71)'s own
  row was a *different buffer* (`all_effects`), and the tell was that the
  shipped change left `fixed`'s allocation table byte-identical.
* **A buffer that is returned is not this shape.** `dispatch_board_scan`'s two
  `Vec`s go out in a `DispatchScan`, so an inline buffer moves its bytes at
  every return; the entry above is specifically about a local that dies in
  its own frame.

**(-76) THE COPY-ON-WRITE `Vec` CLASS — SIZE-NEUTRAL INLINE STORAGE, AND WHAT
IS LEFT OF IT. FIVE FIELDS AND ONE RETURN TAKEN AT THE EIGHTY-SIXTH PASS'S
CONCURRENT HALF (`cube` -0.44 % between them); THE SWEEP IS NOT FINISHED.**

The class: `Vec::clone` hands back `capacity == len`, so **every `Vec` inside
a copy-on-write structure reallocates on its first push after the copy** —
22,684 `GameState` clones and 244,032 CoW unshares a `cube` run. The
`CowBox<Vec<T>>` half is closed centrally (an inherent `push` that
materializes at `len + 1`, zero call sites). The plain-`Vec`-field half is
per field, and the sizing rule is the entry:

```text
SmallVec<[T; N]> is 8 + max(N * size_of::<T>(), 16) bytes.
Vec<T> is 24. So N * size_of::<T>() <= 16 is FREE, and above it the
struct's bytes are the tax — measured at ~0.0009 % of `cube` a byte
((-74)'s padding probe), i.e. 176 bytes is 0.16 %.
```

Taken: `blocked_attackers` `[CardId; 4]`, `blocks_declared_this_turn`
`[(CardId, CardId); 2]`, `block_map`'s value `[CardId; 4]`,
`CardData::blocked_attackers_this_turn` `[CardId; 4]`,
`damage_by_source_this_turn` `[(CardId, u32); 2]` — all 24 bytes, all
size-neutral — plus `PlayerData`'s three per-turn cast lists at the narrow
sizing and `statics_granted_triggers_inner`'s **returned** `Vec` of
references (eight-byte elements, so the return moves the same 24 bytes).

**Two tests before taking one, and they are not the same test.** (a) the read
count — (-72) is 35,000 sites on `.players` and that is what killed it; (b)
the byte count above. A field that fails (a) is dead whatever its size; a
field that passes (a) is free below 16 bytes of payload and priced above it.

**What is left, from the `grow_one` caller table at `a2e6ea33` (`--decks
cube`, 336,530 growths):**

```text
  61,044  10,865,703  Vec::push_mut        <- out-of-line push; its callers are
                                           `static_effect_to_effects`
                                           (`all_effects`, REFUTED — see the
                                           standing rules), `activate_ability_
                                           inner` 44,776 and `run_effect`
                                           24,970 calls, both unread
  37,670   4,522,488  advance_step         <- (-75), the events family
  29,474   3,537,038  dispatch_board_scan  <- returns its two `Vec`s in a
                                           `DispatchScan`; `TriggerGrant`
                                           carries an owned filter, so this
                                           one really does move bytes
  26,852   4,604,936  check_state_based_actions   <- (-75)
  16,066   2,289,388  affected_from_requirement   <- BUILT AND REFUTED,
                                           both halves; see below
  15,856   3,960,971  resolve_combat              <- (-75)
  13,104   2,468,718  deal_combat_damage_to_target
  11,410   1,433,420  grant_scan           <- returns a `GrantScan`
   9,146   1,099,494  effective_mana_abilities_into
   8,286   1,082,988  granted_abilities_of
   8,262   4,573,173  CowBox<Vec<T>>::push <- the amortised growth that is
                                           left after the unshare fix; real
                                           work, not a defect
```

**`affected_from_requirement` LOOKED like the next commit and it is REFUTED,
in two halves, at the eighty-seventh pass — and the reason is (-72)'s read
count, on a field that *passed* the byte test.** `AffectedPermanents` lives
inside every `ContinuousEffect`; its `card_types` is a `Vec<CardType>` of a
**one-byte unit enum**, so `SmallVec<[CardType; 8]>` is still exactly the 24
bytes the `Vec` was, and `Specific(Vec<CardId>)` at `[CardId; 4]` likewise.
Both were built. Base `af5327d5`:

```text
                                       fixed        cube       sealed
  card_types only                    +0.035 %    -0.049 %    +0.009 %
  card_types + Specific + the
    three `Vec<CardId>` producers    -0.212 %    +0.158 %    +0.174 %
  (so the Specific half alone)       -0.247 %    +0.207 %    +0.165 %
```

**Both builds remove exactly the growths they promised** — `grow_one` 333,750
-> 317,684 in *both* — and the `Specific` half removes 30,534 allocations
more, **and `affected_includes_gated` pays 2,757,492 Ir for them, +9.7 % of
its own row.** That function is the whole-board matcher: it reads
`AffectedPermanents` once per (effect x permanent) pair, and on a grant-heavy
board the `Specific` list is routinely longer than four ids — so the
allocation is *still made* and every read pays the `spilled()` compare on top.
On `--decks fixed`, where the lists are short and the pool carries no
`GrantTriggeredAbility`, the same change is a 0.25 % **win**. A textbook pool
split, and the lesson is that **the byte test is necessary and not
sufficient: run the read-count test on the field's *consumer*, not on the
function that fills it.** The `card_types` half on its own is a wash on all
three pools and does not ship either.

Everything under (-75) wants that entry's signature change instead.

**(-75) THE `events: Vec<GameEvent>` ACCUMULATOR FAMILY IS ~0.8 % OF `cube`
**(-75) FIRST INSTALMENT TAKEN AT THE EIGHTY-SEVENTH PASS — `fixed`
-0.035 %, `cube` -0.045 %, `sealed` -0.073 % — AND THE ENTRY'S SIZING IS
REFUTED BY IT.** `check_state_based_actions_into` was the family's second-
largest row (26,942 growths) and **2,622 of those were the `events` buffer**;
the other 24,320 are the sweep's own gated `Vec<CardId>` collects. The sweep
fires no event on most of its 20,152 calls, so the `vec![]` this entry
priced was never allocating. See the Baseline's eighty-seventh block for the
counts and for why two more of the entry's rows (`declare_blockers`,
`advance_step`) are not takeable by threading at all. **What is left of the
entry is `resolve_combat` (15,856) and `resolve_top_of_stack_inner` (8,900),
unattributed — read each row's split before taking it, because the one row
that was split came out 10 % events.** The entry as filed:

**(-75, as found) THE `events: Vec<GameEvent>` ACCUMULATOR FAMILY IS ~0.8 % OF `cube`
IN `grow_one` ALONE, AND IT IS WHAT IS LEFT AFTER (-71)'s SWEEP.** Read at
the eighty-sixth tip, `--decks cube`, from the `grow_one` caller table:

```text
  37,670   4,522,709  stack::advance_step
  26,942   4,705,310  check_state_based_actions
  25,220   4,853,034  combat::declare_blockers          (post-sweep residue)
  22,930   7,237,460  actions::finalize_cast
  19,578   ~4,900,000 combat::resolve_combat
  13,104   ~2,460,000 combat::deal_combat_damage_to_target
   8,900   ~3,040,000 stack::resolve_top_of_stack_inner
```

Every one of these is the same shape: a function builds a
`Vec<GameEvent>` from `vec![]`, pushes a handful, returns it, and its caller
does `events.append(&mut theirs)`. **The first `push` is the allocation** —
`grow_one` fires ~1.3 times per call, so the row is essentially one malloc
and one free per call, on a workload that makes 114,834 actions a six-game
`cube` run.

**Inline storage is the wrong fix here and (-71)/(-72) say why**: the buffer
is *returned*, so an inline `SmallVec` moves its bytes at every return, and
`GameEvent` is a large enum, so even four slots is a big move. **The fix is
threading one `&mut Vec<GameEvent>` down through the callees** so the whole
action's events accumulate in one buffer that the caller already owns.
`advance_step` already takes `mut events: Vec<GameEvent>` by value, so the
shape half-exists and the precedent is in the file.

**Why it is filed rather than taken:** it is a signature change across the
engine's core (`check_state_based_actions` alone has ~30 call sites), and
every `?` path that today discards a callee's events by dropping its `Vec`
would instead have to unwind a shared buffer — which is a *rules* question
(does a rejected action leave its events behind?) and not a refactor. Take it
with the `perform_action` checkpoint's contract in hand, one callee per
commit, and keep the golden traces identical.

**(-74)'s CONCLUSION IS NARROWED BY `(-100)`: the probe number is right and
"the lever is fewer deep copies, not smaller ones" is true only of the ~150
bytes this entry could see.** 296 bytes came out of `CardData` at the
hundred-and-first pass for **-0.60 %** across three pools, moved into the
`CardCold` group that already existed rather than into the second warm group
this entry costed. Read the padding probe as what it is — a *marginal
memcpy* rate for trailing bytes — and add the per-field clone logic (a `Vec`,
an `Option<Arc>`, an `Option<ManaCost>` each cost more than their bytes) when
pricing a real field.

**(-74) THE `CardData` SIZE LEVER IS PRICED BY PROBE AND IT DOES NOT PAY —
+64 BYTES IS `cube` **+0.058 %**, i.e. **0.00091 % A BYTE**, SO THE ~150
BYTES OF MOVABLE CAST-TIME RIDERS ARE **0.14 %** AND A SECOND CoW GROUP IS
NOT WORTH BUILDING.** Measured at the eighty-sixth pass on top of (-73), the
eighty-fourth pass's padding-probe device (`_probe_pad: [u64; 8]` on
`CardData`, two lines, no refactor):

```text
  fixed   1,109,689,934 -> 1,110,067,791   +0.034 %
  cube    3,363,324,209 -> 3,365,270,311   +0.058 %
  sealed  3,301,557,606 -> 3,303,729,394   +0.066 %
```

**The question it was asked, and the arithmetic that made it look promising.**
The CoW deep copy (`Arc::clone_from_ref_in`) is 160,364 calls and ~4.2 % of
`cube` including its allocation and `Vec::clone` subtrees, and `CardData` —
~130 fields, a dozen `Vec`s, ~800 bytes — is the biggest `T` going through
it. Extending the `CardCold` split with a second, *warm* group for the ~45
cast-time rider bools and ~10 `u32`s looked like it should move real money.
It does not: `ComputedPermanent`'s probe reads +8 bytes = +0.058 % of `cube`
on 227,430 allocations, i.e. **8x more per byte**, and extrapolating from
that number is what over-priced this one.

**And the context table says which copies they are.** `--separate-callers=2`
on `cg.cube` at the eighty-sixth tip, by calling context:

```text
160,364 CoW deep copies
  32,374  make_mut <- activate_ability_inner
  30,758  make_mut <- cast_spell_with_convoke
  17,260  make_mut <- declare_attackers_banded
   8,174  make_mut <- declare_blockers
   6,810  make_mut <- do_untap
   6,000  make_mut <- resolve_top_of_stack_inner
   5,962  make_mut <- PlayerData::send_to_graveyard
   5,446  make_mut <- on_left_battlefield
   5,414  make_mut <- finalize_cast
   4,522  make_mut <- PlayerData::draw_top
```

**Of the 160,364, exactly 68,610 are `CardData`** — `ColorMemo::clone` only
exists on that type and is called 68,610 times from inside the copy — and
2,876 are `PlayerData` (`String::clone`, the seat name). The rest are
`CardCold`, `PlayerCold` and the `CowBox` zones. `Vec::clone` runs 287,910
times inside the copies (1.8 each) and `__rust_alloc` 244,032 (1.52 each).
**So the lever is fewer deep copies, not smaller ones**, and the two biggest
contexts are the mana-tap paths — which is (-51)(a), where the entry already
says what not to try.

**(-73) TAKEN at the eighty-sixth pass — `fixed` -0.282 %, `cube` -0.159 %,
`sealed` -0.194 %, and `computed_permanent` leaves the `grow_one` table
entirely (21,120 -> 0).** `LayerFreezeState::perms` — the freeze scope's
per-card layer memo — is `SmallVec<[(CardId, Arc<ComputedPermanent>); 8]>`.
`clear()` keeps capacity, so the field looked like it allocated once per
*process*; what it actually does is allocate once per **`GameState` clone**,
because `Clone for LayerFreeze` is `default()` and hands every probe a fresh
empty `Vec`. 21,120 growths against 22,684 clones on `cube`.

**It passes (-72)'s test where `players` failed it: three read sites** (the
linear scan, the push, the clear), all inside `computed_permanent` and
`end_of_scope`. The +128 bytes it puts on `GameState` cost less than the
allocation on all three pools.

**Only 5,382 of the 21,120 growths were first allocations** — the rest are
`realloc`s through `finish_grow`, which is why the allocation count falls by
a quarter of the growth count. Sixteen slots would take no more growths than
eight does.

**(-72) BUILT, MEASURED AND REFUTED at the eighty-sixth pass — `fixed`
**+0.600 %**, `cube` **+0.490 %**, `sealed` **+0.542 %**, and the allocation
saving was exactly what the entry predicted.** `players: SmallVec<[Player;
4]>` removed **22,684 allocations on `cube`, one per `GameState` clone, to
the unit** — and the run got 16.5 M Ir *slower*, because the saving is ~6.6 M
and the read tax is ~23 M.

**⚠ NARROWED at the same pass's concurrent half: this refutation is about
`players`, not about struct fields.** `SmallVec<[T; N]>` is
`8 + max(N * size_of::<T>(), 16)` bytes, so at `N * size_of::<T>() <= 16` it
is **exactly the 24 bytes `Vec<T>` is** and the only cost left is the read
count — which is what kills `players` (35,000 sites) and not, say,
`blocked_attackers` (30). The same three-field change on `PlayerData`'s
per-turn cast lists reads `+0.137 %` at `[_; 4]/[_; 8]/[_; 4]` (+176 bytes)
and **`-0.463 %` at `[_; 1]/[_; 4]/[_; 1]`** (+16) on an identical allocation
saving. See (-76) and the Log. **Check the byte count before quoting this
entry as a refusal.**

**(-71)'s device does not generalise from a local to a struct field, and the
reason is the read count, not the size.** `sa_cards` has ~forty read sites
inside one function; `.players` has **~35,000** across the workspace, and
every one of them pays `SmallVec`'s `spilled()` compare and pointer select —
even with the `union` feature, which is what made (-71) ship.
`dispatch_triggers_for_events` alone got **+3,633,386 Ir** without touching
an allocation. `GameState::clone`'s own self row moved by -585,498, i.e. the
malloc it stopped making is a rounding error against what the field costs to
*read*.

```text
base 0e86d36f                fixed              cube             sealed
  SmallVec<[Player; 4]>    +0.600 %          +0.490 %          +0.542 %
  cube allocations         1,782,746 -> 1,760,062  (-22,684, as predicted)
```

**Do not rebuild it with a smaller inline capacity.** The +16 bytes on
`GameState` are worth ~2 Ir a clone (45 k Ir over the run); capacity 2 would
recover that and nothing else, because the cost is per *read*, not per clone
or per byte. **And do not reach for `CowBox` on the strength of this
refutation either** — the arithmetic below still holds, and a `CowBox` read
is a pointer load at all 35,000 sites on top of a doubled write path.

**The entry as found, kept because the 472 Ir is real and something else may
yet collect it:**

**`GameState::clone`'s `players` FIELD IS A MALLOC AND A FREE FOR
SIXTEEN BYTES, ON EVERY CLONE — 472 Ir, 35 % OF THE CLONE'S WHOLE INLINE
GROUP, 0.32 % OF `cube` AND 0.46 % OF `fixed`.** Priced by the eighty-third
pass's line profile (`game/mod.rs:2283`, 16,294,384 Ir / 0.55 % at that tip's
clone count) and re-based on the eighty-sixth tip's counts (22,684 clones on
`cube`, 10,822 on `fixed`); the Profile of record's `GameState::clone` block
has the callee table.

`players: Vec<Player>` where `Player` is an eight-byte `Arc<PlayerData>`
handle. **Every other zone in `GameState` is `CowBox`-wrapped and clones for
a refcount; this one clones for an allocation** — and it is the only field of
~250 whose clone costs more than the whole rest of the struct's memcpy.

**`CowBox` is the wrong tool here and the arithmetic says so before the
build.** `CowBox<Vec<Player>>` is `Arc<Vec<Player>>`: the clone becomes one
atomic increment, but the first `&mut` reach allocates *twice* (the `Arc`
header and the `Vec` buffer) where today's clone allocates once, and a bot
probe that resolves any damage or taps any mana writes a seat. It pays only
if most clones never touch `players`, which is exactly the assumption this
file's CoW entries keep getting wrong.

**Inline storage was the shape and it is the thing that was refuted**, above:
`SmallVec<[Player; 4]>` is a three-line change (`GameState::players`,
`GameState::new`'s conversion, `shuffle_seating`'s scratch buffer) that
compiles the whole workspace with **no call-site edits at all** — the
`Deref<Target = [Player]>` covers every one of the ~35,000 `.players` uses —
and that is exactly why it loses. The wire format did stay a sequence
(smallvec's `serde` feature), so nothing was invalidated; the change simply
costs more to read than it saves to clone.

**(-68) TAKEN at `e9a509e6` — `fixed` -0.215 %, `cube` -0.317 %, and the
function's own row -17.7 % / -21.3 %.** Two walks instead of six: the dealer
lookup stops short-circuiting and answers the equipment / aura / soulbond
gates on the way past, and the `YourControl` and `AnyPlayer` listener walks
fuse behind a buffer that is `Vec::new()` on every board without an
`AnyPlayer` listener. **The entry's estimate was 0.5-0.8 % and it was high
for a reason worth keeping: a gate that rides on an existing *early-exiting*
scan is not free.** Widening the dealer `find` into a full walk gave back
part of what the three gates saved, and nothing in the arithmetic below
priced that. Count what a scan was skipping before you widen it. The entry
as found:

**(-68, as found) `fire_combat_damage_triggers` WALKS THE BATTLEFIELD SIX
TIMES PER DAMAGE EVENT — 51,274,236 Ir OF SELF / 1.51 % OF `cube` / 19,830 TOP-LEVEL
CALLS / 2,586 Ir OF SELF EACH, AND NO ENTRY HAS EVER NAMED IT.** Read at
`a63b1934`, `--decks cube`, `cg_edges.py` self table. Its callee list is
almost empty (20,022 `__rust_alloc` for 1.1 M — the one
`Vec<Vec<DamageTrigger>>` a call), so essentially all of it is the function's
own inlined walks:

```text
  battlefield.iter().find(|c| c.id == source)     the dealer  (short-circuits)
  phase 1b   for eq in &self.battlefield          equipment-granted
  phase aura for aura in &self.battlefield        EnchantedBySource
  phase soul for src in &self.battlefield         CR 702.95 soulbond
  phase 1.5  for c in &self.battlefield           YourControl listeners
  phase 1.6  for c in &self.battlefield           AnyPlayer listeners
```

Six walks of ~10 permanents at ~50 Ir a visit is 2,500, which is the 2,586.
**Two of them fuse and three of them gate**, and the ordering constraint is
what makes it fiddly rather than free: the phases push into `by_kind[i]` in
phase order and that order reaches the stack, so a naive single walk
interleaves them. 1.5 and 1.6 fuse if 1.6's hits go through a buffer that is
`Vec::new()` (so allocation-free) on every board without an `AnyPlayer`
listener and is drained after; 1b / aura / soulbond gate on "anything
attached to the dealer" and "a soulbond pair involving the dealer", both of
which the dealer lookup can compute for free by not short-circuiting.

**(-70) TAKEN at the ninety-first pass — `fixed` -0.626 % / `cube` -0.706 % /
`sealed` -0.741 %, i.e. 18-28 % MORE than the probe below, and the reason is
in the Baseline block.** The quiet window this entry waited for came; the
refactor is the one it describes (one `Arc` on the struct, an `Overlay` per
field) with one change of plan that is why it fitted in a pass: `Printed` /
`PrintedList` were kept as the layer pass's **borrowing** working types, so
`compute_permanent_pass`'s body is untouched and only `into_overlay()` at the
struct literal is new. **The 600-800 call sites were 2,991 lines over 282
files and every one of them was found by the compiler**, because the fields
went private in the same commit — a private field is an error with a
`call it with parentheses` suggestion, not a silent fallback, so the whole
sweep is `rustc --message-format=json` plus a byte-exact patcher. What was
left after the E0616 pass: ten `keywords().as_slice()`, seventeen
`keywords().clone()` (a `PrintedList` clone that is now a slice reborrow —
`to_vec()`), four `assert_eq!` against a `Vec`, three temporaries that
`let x = &cp.subtypes().field` no longer lifetime-extends, and 23
`needless_borrow`. **The blast radius was the entry's whole objection and it
was an afternoon.**

**(-70, as found) `ComputedPermanent` CARRIES **FOUR** `Arc<CardDefinition>`
HANDLES TO THE SAME DEFINITION, AND THE THREE REDUNDANT ONES ARE `fixed`
**0.532 %** / `cube` **0.580 %**. PRICED BY PROBE, NOT BY ARGUMENT; NOT TAKEN,
AND THE REASON IS THE BLAST RADIUS.**

`Printed<T>` and `PrintedList<T>` each hold their own
`src: Arc<CardDefinition>` so their `Deref` can project into it, and
`ComputedPermanent` has four of them — `card_types`, `supertypes`, `subtypes`,
`keywords` — every one pointing at the same definition. The struct is built
**302,018 times** and `Arc`-allocated 227,430 times a six-game `cube` run, so
that is three extra atomic increments, three extra decrements (each with the
zero-check that `Arc::drop_slow` hangs off) and 24 extra bytes, per computed
permanent.

```text
measured by adding a `[Arc<CardDefinition>; 3]` field and cloning `def` into
it — the proposed change with the sign flipped, two lines, no refactor.
profiling-fast --no-default-features, base 2bb47a02
  fixed  1,113,117,966 -> 1,119,039,885   +0.532 %
  cube   3,386,397,338 -> 3,406,055,793   +0.580 %
```

Split by half, using the eighty-fourth pass's padding probe (+8 bytes =
`fixed` +0.040 % / `cube` +0.058 %): the 24 bytes are ~0.12 % / ~0.17 % and
**the clone-and-drop pairs are the other ~0.41 % on both pools.** For scale,
`LayerFreezeState::end_of_scope` alone reaches `Arc::drop_slow` 218,782 times
for 51.6 M Ir, 1.52 % of `cube`, tearing these down.

**The shape that would fix it is one `Arc` on `ComputedPermanent` and an
`Overlay { proj, over }` per field**, with the definition passed in at the
read — and that is why it is not taken here: `Deref` on the field is what
keeps `cp.keywords.has_kw(..)` working, and turning one field into a method
is **205 call sites** (280 errors) by rustc's own count, so four fields is
600-800. That is a whole session's mechanical diff on files two concurrent
sessions rebase every twenty minutes. **Take it in a quiet window, one field
per commit, driven by the compiler's error list.**

**And the device is the reusable half: a redundancy you cannot remove without
a refactor can still be priced by *adding another copy of it*.** Three extra
`Arc` handles cost exactly what three redundant ones cost, and the probe is
two lines. Same trick as the +8-byte padding probe one entry up; between them
they price a struct's size and its handle count without touching a call site.
**Amended by the take: "cost exactly what three redundant ones cost" is the
half that was wrong.** The probe adds handles to a struct that keeps all its
drop glue and its allocation size class; removing them changes both, and the
real number came in 18-28 % over. A sign-flipped probe is a **floor**.

**(-69) `Vec::from_iter`'s TWO MONOS ARE 3.45 % OF `cube` AND THE CALLER TABLE
BY COUNT HAS TWO ROWS NOBODY HAS COSTED.** Read at `a63b1934`, 778,489 calls:

```text
  234,574     56,658,210  compute_permanent_pass          <- (-56b), REFUTED
   37,196     27,872,140  declare_attackers_banded
   36,150     98,042,596  check_state_based_actions       <- unclaimed
   23,726      4,993,835  blockers_of
   18,258     80,643,480  compute_permanents              <- unclaimed
   15,966    114,396,124  pick_blocks_inner
   12,774    170,721,347  cast_candidates
```

The inclusive columns are subtrees, not the collect, so they rank the *call
site* rather than the allocation. Read (-56b) before proposing a replacement:
`collect` is internal iteration and a hand-written loop is not, and that cost
~0.15 % of `fixed` the last time somebody swapped one.

**`check_state_based_actions`' SHARE OF THIS IS REFUTED AS A *COLLECT*
ENTRY — read at `e9a509e6`, by reading rather than by a second profile, and
the answer is that every one of its collects is already gated.** 20,152 SBA
calls on `cube`; the `from_iter` edge under it is 33,694 calls / 91,688,893 Ir
inclusive, **2.71 % of the run**, 1.67 collects a call at 2,721 Ir of subtree
each — which looks exactly like (-44)'s shape and is not. All twenty-two
`.collect()` sites in the function were checked one by one:

* the first half's nine (`flip_keyword`, `flip_predicate`, `sacrifice_when`,
  `state_trigger`, `steal_penalty`, `no_other`, and the three
  continuous-effect lapse sweeps) are `if scan.<flag> { … } else {
  Vec::new() }` or sit behind an `any()` presence test;
* the second half's (Saga, planeswalker, battle, bestow, orphaned Auras,
  stale Roles, Equipment links, Soulbond, the legend rule) are the same
  shape against their own `sba_board_scan` flags;
* and the **death sweep** — the only one that is not conditional on a
  narrow card class — is behind `creature_death_possible(&scan)`, which
  the code's own comment measures as skipping **86.5 % of sweeps on
  `fixed`**.

**So the 2.71 % is not an ungated collect; it is `compute_battlefield_
creatures` doing real work on the sweeps where a death genuinely is
possible**, which on a grant-heavy pool is a much larger fraction than on
`fixed`. That is the layer gather, i.e. (-62)/(-58) territory and the
most-worked path on the branch — **not a `Vec` nobody wanted.** Rank the
entry's other two unclaimed rows above this one.

Two rows from the same callee table recorded as *non*-leads so nobody
re-reads them: `CardDefinition::is_creature` at 352,032 calls (17.5 an SBA
call, 5,358,714 Ir) is the standing rules' named profile artifact —
`release`'s thin LTO inlines it — and `card_type_change_unscoped` at 19,508
calls / 10,844,182 is one an SBA call, a gate doing its job.

**(-67) TAKEN at `a63b1934` — `fixed` -0.901 %, `cube` -0.806 %.** The bot's
affordance sweep cached a `GameState` clone as a "probe template" and every
probe cloned *that*; `affordance_probe_template` had become plain
`self.clone()` when the library strip came off it, so the cached value was
equal to the state it was made from. `GameState::clone` -23.5 % of its calls
on `cube`. **The find is a stale abstraction, not a hot loop** — see the Log.
`affordances.rs` has ~12 more of the same shape, unmeasured and untouched.

**(-66) TAKEN at `abdb6373` — `--decks sealed --games 1` -7.46 %,
`sealed_pool` -25.9 %.** `sos_draft_pool` and `SosPacks::new` are functions
of nothing and were rebuilt per pool, i.e. twice a game in a training actor;
both are `OnceLock`s now. **The device is the fifty-fourth pass's memo rule
one level out:** memoizing the card definitions does not help if the *pool*
built from them is re-derived. Ask what else in a prologue has no inputs.

**(-63) TAKEN AT THE EIGHTY-FIFTH PASS IN FIVE COMMITS — A HEURISTIC SEALED
BUILD IS 16,732,968 -> 9,146,042 Ir, `-45.3 %`, WHICH AT 4.02 % OF THE ACTOR
IS ~1.8 % OF IT — AND THE CONCURRENT SESSION TOOK ANOTHER **-22.1 %** OUT OF
IT IN THE SAME PASS (762,394 -> 594,245 a build; see the Baseline's
concurrent-half block), SO A BUILD IS 2.35x CHEAPER ACROSS THE TWO HALVES.
WHAT IS LEFT IS `rank_shape`'s OWN BODY — 3,315,654 Ir of self over 672
shapes, 29.5 % of `--decks sealed --games 1`, no hot line, and the only
remaining lever is **fewer shapes**, which is the training-distribution
argument this entry has always flagged as not-unilateral. THE ENTRY'S OWN FRAMING WAS WRONG IN THE USEFUL DIRECTION:
IT SAID THE ONLY FREE LEVER WAS "A CHEAPER 56", MEANING THE PER-SHAPE
ALLOCATIONS, AND FOUR OF THE FIVE COMMITS INSTEAD REMOVED WORK THE 56 SHAPES
DID FOR NOBODY.** See the eighty-fifth pass's Log item 1 for the rows.

**AND THE INSTRUMENT THIS ENTRY DECLARED DEAD IS ALIVE — `--decks sealed
--games 1` IS 76.5 % DECK CONSTRUCTION AND COSTS TWENTY SECONDS UNDER
CALLGRIND.** The entry (and "Which pool a change moves") said it "plays no
games *and* builds no decks, so it isolates nothing" because it had fallen
from 2.9 G Ir to 21.9 M. It plays no games; it still builds **twelve** sealed
decks, and `heuristic_sealed_build` inclusive was **16,732,968 of the
21,864,561** at `15558656`. The 130x fall is what five passes of deck-builder
work did to it, not evidence that the decks stopped being built. **A shrinking
instrument is not a dead one: read its callee table before retiring it** — the
`selfplay_train --actors 1 --games 4` recipe the entry sent readers to costs a
minute under callgrind and a `crabomination_ml` build, and this costs neither.

**(-63, as found) DECK CONSTRUCTION IS 5.37 % OF THE TRAINING ACTOR AND
`--bench` CANNOT SEE ANY OF IT. FRESH AT THE EIGHTY-THIRD TIP; NOBODY HAD READ
IT SINCE THE FIFTY-FOURTH PASS TOOK IT 3.28x.**

One `actor_loop` iteration over 60 games, from the actor profile above:
`heuristic_sealed_build` **168,509,048 Ir over 120 builds** (two a game),
plus `encode_deck` 28,058,541, `sealed_pool` 17,852,492 and
`sealed_game_template` 10,279,007 — **224.7 M, 5.37 %**, at ~1.87 M Ir a
build. For scale that is 87 % of what the *encoder* costs (6.21 %), and the
encoder has had four passes of work against this entry's zero since the
builder fix landed.

`recommend::build_shape` is the largest row inside it: **54,029,042 Ir of
self, 1.29 % of the actor, 6,840 calls** — 6,720 of them from
`enumerate_candidates_with`, i.e. 56 shapes per build. The fifty-fourth
pass's finding is the one to read first (`CardBrief`, "a memoized object is
not a memoized *answer*"), because the shape it names — re-deriving per
(pick x candidate x colour shape) what a memo already holds — is exactly
what a 56-shapes-per-build row looks like.

**THE INSTRUMENT IS `selfplay_train --actors 1 --games 4`, NOT
`--decks sealed --games 1`, AND THAT RECIPE IS STALE.** `bot_ladder --decks
sealed --games 1` ran 2.9 G Ir when "Which pool a change moves" was written
and runs **21,864,400** now: it plays no games *and* no longer builds any
decks, so it isolates nothing. The four-game actor run does — **1 minute
under callgrind**, and it reports the subtree exactly:

```text
CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 4 --steps 1 --seed 7
callgrind, profiling-fast --no-default-features.  324,740,275 Ir total.
  8   11,339,499  heuristic_sealed_build      (1.42 M a build)
  8    3,348,647  sealed_pool
  8    1,924,030  encode_deck
  inside one build:  enumerate_candidates_with 10,418,468 (92 %)
                       build_shape  448 calls = 56 a build, 9,053,055
                         assemble_lands       455  1,823,327
                         static_build_score   455  1,336,082
                         __rust_alloc       2,980    505,687   (6.65 a shape)
```

**AND IT HAS NO HOT LINE EITHER — line-profiled at the eighty-third pass, do
not re-run it.** `profiling-lines` + `--dump-instr=yes` + `cg_lines.py --in
build_shape`: the inline group is 3,465,136 Ir and **its largest line is
0.11 % of the run** (`Filter::next` inside `suggest_main_deck_shape`, which
inlines wholly into `build_shape`); `recommend.rs:534`, the `allow` bitmask
write in the pool walk, is 0.10 %. `assemble_lands` and
`static_build_score` group at 0.1 % each.

**So this is a *count* entry, and the count is 56.** The lattice is 10 pairs
+ 30 pair-plus-splash + 16 wide, each one a whole-pool walk; the fifty-fourth
pass already took the per-walk cost (`PoolScores`, the `allow` bitmask, the
pre-sized vectors), so what is left is the 56. **Two levers and only one is
free:**

* **Fewer shapes** — changes which deck the softmax lands on, i.e. the
  *training data distribution*. That is the encoding caution's territory even
  though it touches no encoding: a field of differently-shaped decks is a
  different dataset. **Not to be taken unilaterally**, and if taken it needs
  a strength gate, not a byte-identity one.
* **A cheaper 56** — the per-shape allocations are the only thing here that
  is free to be wrong about: 6.65 `__rust_alloc` a shape, of which
  `assemble_lands`' `duals`/`taken` are two and both have a bound. Read
  (-56) first: an *exact* size ships and headroom does not, and `duals` is
  usually 0-3 against a `total` of ~17, so reserving `total` is the trap.

**WHAT IS LEFT, at `49748e1f`, `--decks sealed --games 1` = 14,248,408 Ir of
which `build_random_deck` is 9,146,042 over twelve builds (762 k a build).**
Nothing here is ranked above the ladder's queue; it is written down so the
next reader does not re-derive it.

```text
  3,230,204  rank_shape self          684 calls, 4,722 each: two walks of an
                                      ~85-card pool per shape, ~28 Ir a card
  2,032,767  static_build_score       684 calls; 1,393,624 of it is 15,733
                                      score_brief_with_colors at 89 Ir
    476,338  SpecFromIterNested       the splash collect, was 1,611,224
    351,866  HashMap::insert          seen_mains (672) + PoolScores ids (1,020)
    297,670  __rust_alloc from lattice 1,392 calls = 116 a build, of which ~60
                                      are the shape lattice's own Vec<Color>s
    227,961  driftsort_main           out.sort_by_key over 56 ShapeBuilds,
                                      +93 k when the struct grew a field
    145,488  color_subsets            24 calls, 2 a build
```

Three things are shaped like a next commit and none is priced: (i)
**`static_build_score`'s `colors_of_picks(main)` is a second walk of the cards
`take` just chose** — `take` holds the index and `PoolScores::cards` holds the
brief, so the `ColorCounts` could be accumulated there for free; (ii) **the
shape lattice is 56 fixed `(Vec<Color>, Vec<Color>)` pairs rebuilt per deck
build** — `color_subsets` plus ~60 small allocations, worth ~440 k, and a
`LazyLock` static answers it; (iii) `out.sort_by_key` moves 56 x ~136 bytes
and could sort indices instead.

**(-64) THE `Vec<bool>` HALF TAKEN AT THE EIGHTY-FOURTH PASS — `fixed`
-0.328 % / `cube` -0.280 %, see Log item 4 and Baseline (6). THE REST OF THE CLONE IS STILL
HERE AND STILL UNTAKEN.** What shipped is the `acted_on_own_turn` bitmask
only; the `Box<dyn Decider>` clone (28,172 calls / 3.36 M) and the ~2.5
remaining `__rust_alloc` a clone are open, and the reading below is the
inventory to work from.

**(-64, as found) THE STATE CLONE ALLOCATES 3.5 TIMES, AND ONE OF THEM IS A
TWO-ELEMENT `Vec<bool>`.** `<GameState as Clone>::clone` is **29,692 calls /
~70.2 M Ir / 2.01 % of `cube`** at `3509ec81` (11,976 from `accept_on`'s
probe, 6,964 from a `OnceCell::try_init`, 4,968 from `perform_action`'s
checkpoint). Per clone: **1,357 Ir of self**, 6.0 `Vec::clone` (39 Ir each,
so most are empty), 3.0 `RawTable::clone` (26 Ir, likewise), 5.4 `__memcpy`
and **3.48 `__rust_alloc`**.

Reading the field list against those counts:

* `acted_on_own_turn: Vec<bool>` is the **only** small `Vec` on `GameState`
  that is always allocated — `do_untap` resizes it to `players.len()` every
  turn — so it is one `__rust_alloc` + one `__rust_dealloc` + one
  `Vec::clone` per state clone, ~257 Ir, **~0.22 % of `cube`**. Seats are
  single digits; a `u64` bitmask removes the heap entirely. The only cost is
  serde: the field is on the snapshot format, so it wants a new name plus
  `#[serde(default)]` rather than a type change in place.
* `controlled_by: Vec<Option<usize>>` is `Vec::new()` unless a seat is
  Mindslavered, so it is a 39-Ir empty clone and nothing else. Not worth a
  commit on its own.
* `decider: Box<dyn Decider + Send + Sync>` costs 28,172 `Box::clone` /
  3.36 M. **`Arc` is not the fix**: `Decider::decide` takes `&mut self`.
* **`ColdState` is the wrong home for any of them.** A cold write unshares
  ~89 collections at ~4,689 Ir (see (-50)), so a field written even a few
  thousand times costs more there than the clone it saves.

**(-65) TAKEN AT THE EIGHTY-FIFTH PASS AND IT IS SIX TIMES THE ENTRY'S
ESTIMATE: `fixed` -0.882 %, `cube` -1.195 %.** The entry priced it at "~0.2 %
of `cube` on Ir" off the clone's own rows and that was the mistake — **a
dropped deep copy costs its allocation, its `memcpy`, its `grow_one`, its
drop and its `free`, and only the first was in the entry.** Measured:
`TriggeredAbility::clone` under the walk 7,706,934 -> 0 and `grow_one`
10,701,492 -> 2,498,092, i.e. 18.4 M of named rows — against a whole-program
delta of **41.5 M**, the rest being `__memcpy` (-18,699,215),
`IntoIter::drop` (-4,792,050) and `_int_malloc` (-4,418,071). See the
eighty-fifth pass's Log item 0. **The rule: size a clone removal by the
clone's whole lifecycle, not by its `clone` row.** The reading below is what
the entry carried.

**(-65, as found) `statics_granted_triggers_inner` DEEP-COPIES ABILITIES ITS
CALLER FILTERS OUT.** 142,744 calls at `3509ec81` on `cube`; its callee list is
`evaluate_requirement_static_hinted` 142,094 / 48.7 M (**1.39 %**),
`TriggeredAbility::clone` 20,886 / **7.7 M**, `grow_one` 19,128 / **10.8 M**
and `__memcpy` 20,886 / 2.1 M. 85,066 of the calls are `fire_step_triggers`,
which does `for t in self.statics_granted_triggers_on(c, &grants)` and then
**drops every `t` whose `event.kind` is not this step's** — the printed-
trigger loop three lines above it already filters on the borrowed ability and
clones only the survivors, and this one does not.

Every source the function pushes from is borrowable (`grants` is
`&[TriggerGrant<'_>]`, `turn_granted_triggers` is `&self`, the station bands
are `&card.definition`), so `Vec<&TriggeredAbility>` type-checks. **Size it
before building it**: this is pass 57's `granted_abilities_of` shape, which
read `sos` -1.946 % on Ir and was **flat on the clock** — the difference here
is that the clones are *dropped*, not moved, so the removal is real work
rather than a copy turned into a pointer chase. Worth ~0.2 % of `cube` on Ir
and the clock question is open.

**(-62) TAKEN, SAME PASS, BY THE SHAPE THIS ENTRY NAMED.** `crate::zone::
Graveyard` wraps the CoW card list with the memo; `fixed` **-0.499 %**,
`cube` **-0.267 %**, and it caught `keyword_grant_in_scope`'s off-battlefield
tail as well, which this entry had not noticed was the same walk. See
Baseline (4). **The memo captures 70 % / 63 % of the deletion ceiling, and
the rest is not the miss path** — that reading was the obvious one and it is
refuted with numbers in Log item 2a, along with the memo read itself. What
the difference is is the deletion probe over-reading: `false &&` lets the
optimizer delete the structure around the walk too. The entry stays below
because the four-shape analysis is the reusable part — in particular the one
that says *why* the newtype and not the flag.

**(-62, as priced) THE LAYER GATHER WALKS EVERY CARD IN EVERY GRAVEYARD ON
EVERY RECOMPUTE, UNGATED, AND IT IS WORTH `fixed` -0.440 % / `cube`
-0.251 %. PRICED BY DELETION AT THE EIGHTY-THIRD PASS; THE REASON IT WAS NOT
TAKEN IN THE SAME BREATH IS THE GATE, NOT THE NUMBER.**

`gather_continuous_effects_inner`'s tail (`game/mod.rs`, the
`GraveyardAnthem` pass — the Incarnation cycle's "as long as this card is in
your graveyard and you control a [Land subtype], creatures you control have
[keyword]") is

```text
for player in &self.players { for card in &player.graveyard {
    for sa in &card.definition.static_abilities { ... } } }
```

with no presence gate in front of it. Every other whole-collection pass in
that function is behind one (`sa_mask`, `any_dynamic_pt`, `any_soulbond_
bonus`, …); this one is not, and it is ~8 Ir a graveyard card — three
pointer chases, `CardInstance` -> `CardData` -> `Arc<CardDefinition>` ->
the `Vec` header — times a graveyard that reaches twenty-plus cards a side
by mid-game, times **every** gather.

```text
measured by replacing the outer iterator with `std::iter::empty()`,
profiling-fast --no-default-features, base 34d118fe (the SecondPass tip)
  fixed  1,166,181,370 -> 1,161,055,975   -0.440 %
  cube   3,542,194,720 -> 3,533,307,199   -0.251 %
  --bench 195,528 / 27.44 / 0 stalls — byte-identical, so neither pool
  carries a GraveyardAnthem card and the deletion moved no play
```

**The walk *is* the gate**, the way `trigger_grant_sources`' is in (-60), so
the only lever is a maintained answer. Four shapes considered:

* **A `GameState` flag invalidated at the graveyard write sites.** 334 sites
  match a write-shaped grep (19 of them are *insertions*, which is all the
  flag needs — a stale "yes" only costs the walk). Missing one is a silent
  wrong game (an Incarnation's anthem stops applying), not a crash.
* **A flag on `PlayerData`, cleared in `Player`'s `DerefMut`** — one plain
  byte store at the single point every `&mut` reach into a seat goes through,
  so the invalidation is *provable* rather than enumerated, and it needs no
  newtype. **Ruled out on frequency, measured before building:**
  `cg_sites.py` prices the whole inlined `Player::deref_mut` family at
  **2,361,894 Ir (0.08 % of `cube`)**, i.e. ~120-240 k calls against the
  gather's 71,884 — two to three player mutations per gather — and its
  largest single site is `check_state_based_actions`, which runs after every
  action. **The cache would be cold at almost every read.** The rule:
  **price the invalidation point's frequency against the memo's read
  frequency before writing either.** That ratio, not soundness, is what
  decides a zone memo, and it is one `cg_sites.py` run.
* **A newtype around `graveyard: CowBox<Vec<CardInstance>>` carrying the
  answer, cleared in its `DerefMut` and its `&mut IntoIterator`** — the two
  entry points (-18) established are *complete*, so the invalidation is
  provable rather than enumerated. `Deref` keeps the 2,033 read sites
  untouched; the work is the ~75 construction/assignment sites and serde.
  **This is the shape that would ship**, and it is a session's work on its
  own, not a corner of one.
* **A board-epoch key** — (-18), refuted, and its lesson applies here in the
  narrow direction: the counter is only free if it is on the *one* zone that
  wants it, which is what the newtype is.
* **A length-based key** (`(Arc::as_ptr, Vec::as_ptr, len)`) — unsound: a
  remove-then-push of a different card can hold all three fixed.

**And the general finding is bigger than the entry: this is the one pass in
the gather nobody gated, and it was found by *reading the function for
ungated whole-collection walks*, not by a profile row** — it has no row of
its own, being inlined into a 3,581-line function. Grep the gather for
`self.players` / `graveyard.iter` / `exile.iter` before trusting a self
table to have found everything in it.

**(-59) `dispatch_triggers_for_events` IS THE LARGEST SELF ROW IN THE
PROGRAM AND NO ENTRY HAS EVER NAMED IT — 198,765,010 Ir / 5.58 % OF `cube` /
139,500 CALLS / 1,425 Ir OF SELF EACH.** Read at `c1450677`, `--decks cube`,
`cg_edges.py` self table. For scale, the next four rows are the gather
(4.81 %) and three allocator symbols.

```text
callers                                     calls    inclusive Ir
  perform_action_inner                    114,834     281,333,427   7.90 %
  declare_attackers_banded                 13,092         615,324
  finalize_cast                             7,008      23,794,641
  submit_decision                           2,078      22,855,387
  do_untap                                  2,102       9,699,368
its own callees, the two that are rows of their own
  dispatch_board_scan                      74,800      44,863,784   1.26 %
  statics_granted_triggers_inner (via the per-card loop)
```

**1,425 Ir of *self* is the batch's fixed cost before a single trigger
fires**, and the function's shape says where it goes: an `events` walk to
stamp entry turns, four more `events.iter()` passes (the graveyard-batch
collapse, the synthesis fold, the delayed-trigger filters), a whole-
battlefield loop, and a `died_card_snapshots` walk at the tail. The
`no_grants` fast skip inside the battlefield loop is already there and is not
the question.

**THE LINE PROFILE IS RUN AND THE ANSWER IS "THERE IS NO HOT LINE".**
`profiling-lines` + `--dump-instr=yes` + `cg_lines.py --in
dispatch_triggers_for_events`, `--decks cube`, at `bb22e774`. The inline group
is **203,632,136 Ir, 6.9 % of the run** (larger than the self row because it
carries everything inlined into it), and the largest single line in it is
**0.23 %**:

```text
  19,459,084  0.66%  game/mod.rs:?          (unattributed within the fn)
  12,593,348  0.43%  vec/mod.rs:464
  11,575,534  0.39%  iter/macros.rs:?
   8,699,548  0.29%  ptr/non_null.rs:444
   6,939,582  0.23%  game/mod.rs:22260      is_event_hardcoded's `match ev`
   6,907,298  0.23%  ptr/mut_ptr.rs:961
   4,442,364  0.15%  game/mod.rs:17315      the battlefield loop's skip check
   4,066,796  0.14%  game/mod.rs:17461      event_subject, innermost loop
   3,937,416  0.13%  game/mod.rs:17443      is_event_hardcoded call site
   2,405,960  0.08%  game/mod.rs:17460      event_matches_spec call site
```

Grouped by region: the **per-card battlefield loop prologue** (17314-17316,
17328, 17353-17359, 17374 — the skip check plus the three grant lookups) is
**~13 M / 0.44 %**; the **per-(trigger, event) inner loop** (17443-17461) is
**~11.4 M / 0.39 %**; the rest is ~30 M of iterator and pointer internals
spread across a dozen narrow passes, none above 0.1 %. **The graveyard walk
and the death-snapshot walk — the two ungated whole-collection loops a source
read flags as suspects — do not appear in the top forty at all.**

**So the lever is fewer dispatches, not a cheaper dispatch.** 114,834 of the
139,500 come from `perform_action_inner` draining one action's event list.
Nothing here is takeable by a gate; anything that moves it has to remove
whole calls. **Do not re-run this line profile** — it cost a cold
`profiling-lines` build and this table is its whole result.

**(-61) `keyword_grant_in_scope` IS 1.7 MILLION `card_can_grant_keyword`
CALLS — 59,569,000 Ir OF SELF / 1.67 % OF `cube` — AND ITS THREE CALLERS ARE
EACH ONE WHOLE-BOARD WALK PER QUESTION.** Read at `f51e695b`, `--decks cube`.
The presence gate itself is already tight (~35 Ir a card, and the per-card
disqualifier is five `is_empty` loads), so the lever is **fewer walks**, not a
cheaper one:

```text
callers of card_can_grant_keyword     1,713,848 calls
  543,758   21,702,216  card_keyword_possible        <- 21.6 cards a question
  461,668   13,594,100  apply_prevention_shields_with
  430,086   16,602,010  damage_prevented_by_protection
  104,598    3,198,300  advance_step
   52,560    1,649,470  do_untap

and one level up
  25,204   29,946,272  activate_ability_inner -> card_keyword_possible  0.85 %
  12,740   23,300,482  resolve_combat -> apply_prevention_shields_with  0.66 %
```

**`activate_ability_inner` asks three of these** (CR 602.5's
`CantActivateTapAbilities`, `Haste`, `CantActivateAbilities`) at 1,188 Ir a
question, over 50,058 calls — but only **0.5 questions per call**, so fusing
them into one union walk buys almost nothing and was dropped before it was
built. The two prevention callers are inside `resolve_combat`'s **per-pair**
freeze scopes, so a scope-level memo (the `TypeGate` device, which has three
slots and no keyword slot) would collapse repeats *within* a pair and not
across them — the same wall (-58) hit.

**And the largest single one of these gates is already a 4.5x trade — do not
take it.** `apply_prevention_shields_with`'s 461,668 calls are one site: the
**CR 702.64 Absorb gate** (movement.rs:496), which asks "can any permanent
have `Absorb`" before reading the damaged permanent's computed view. It is
**1,068 Ir a damage event, 13,594,100 Ir, 0.38 % of `cube`, and 58 % of the
whole prevention funnel's cost** — and it exists to avoid an unfrozen
`computed_permanent`, which is ~4,900 Ir. Four things were considered and all
four are worse or unsound:

* **Fold it into `prevent_static_scan`'s existing walk.** Same frequency (the
  scan is per damage event too, inlined at movement.rs:220), and the per-card
  cost is `card_can_grant_keyword` either way — the shared iteration is ~10 %
  of it. Nothing.
* **Hoist the scan to once per damage batch.** This is the (-58) shape: the
  mask is a function of the whole state, a rider (New Way Forward) can resolve
  mid-batch, and the audit that would make it safe is (-58)'s
  `debug_assert!`-plus-`-C debug-assertions=yes` ladder run. Priceable, not
  free.
* **Memoize it in a freeze scope.** `apply_prevention_shields` runs *after*
  `resolve_combat_damage_with_filter`'s per-pair scope closes — that scope
  ends at the first `&mut self`, which is this call.
* **Delete the gate.** Four and a half times worse.

**What has not been tried:** the local shortcut, on the *other* caller.
`activate_ability_inner`'s three gates exist only to avoid `bf_cp!()`, its
take-once computed-permanent read (actions.rs:13971) — and PERF's forty-
seventh pass rule is that **a gate that stands in for a gather stops paying
once the gather has run**. `bf_cp.is_some()` short-circuits the gate exactly,
for free, and `layers_memoized()` is the same device one level up. It is
small — `bf_cp` is itself behind three presence gates and is usually still
`None` by then — but it is the only thing here that costs nothing to be wrong
about.

**(-60) `trigger_grant_sources` WALKS EVERY STATIC ABILITY ON THE BOARD TO
FIND A QUARTER OF A GRANT, 57,596 TIMES — 35,656,442 Ir OF SELF / 1.00 % OF
`cube`.** Its callee list is 2.1 M, so essentially all of it is the walk plus
the inlined `active_static`; `resolve_named_by_source` fires 14,444 times
across those 57,596 calls, i.e. **0.25 grants found per call**.

The CR 510.2 creature-damage batch (12,858 of them) was taken at the
eighty-second pass — hoisted to one walk per batch, `cube` **-0.299 %**,
`fixed` -0.006 %. What is left, with the shape each one wants:

```text
  23,526  fire_step_triggers          already one per call; the walk itself
                                      is the cost (14.6 M / 0.41 % of cube)
   7,070  fire_combat_damage_to_player_triggers   per-attacker, deep inside
                                      the mutating target match — the same
                                      hoist, harder
   4,164  resolve_top_of_stack_inner
   3,352  fire_self_etb_triggers
```

`fire_step_triggers` is the interesting one because it is *already* hoisted
correctly and still costs 0.41 %: there the question is not "how often" but
"why 619 Ir". A presence gate does not answer it — the walk **is** the gate,
and on `cube` the grants exist. The shape that would is a per-`CardDefinition`
"carries a `GrantTriggeredAbility` static" bit, computed once for an
immutable `Arc<CardDefinition>` rather than per walk; `static_effect_gather_bits`
is the existing precedent for the shape question and `gather_continuous_effects_inner`
already recomputes it per card per gather, so the bit would pay twice.

**(-56b) REFUTED: `compute_permanent_pass`'s `sorted` COLLECT IS 189,480
CALLS / 46,426,567 Ir AND FOUR WAYS OF NOT ALLOCATING IT ARE ALL WORSE.**
Base `31eb7333`, same instrument, all four built and measured:

```text
                                                  fixed        cube
  [Option<&CE>; 12] + `for` loop                +0.151 %    -0.381 %
  peel the first two by hand, Vec beyond        +0.184 %    -0.322 %
  `for_each` into one/many accumulator          +0.311 %    +0.054 %
  [Option<&CE>; 4]  + `for_each`                +0.269 %    +0.0005 %
```

**Only the first has a win and it splits by pool, which is the same verdict
the two `sa_cards` reserves got one entry down.** On `fixed` the filtered list
is empty on most passes, so `collect` never allocates there and every one of
these is pure added cost; on `cube` it allocates often enough to pay.

**Two things here are durable and neither was obvious.** (a) *The stack buffer
is not the expensive part.* Removing it entirely (row 2) read **worse** on
`fixed` than the twelve-slot version, so the ~0.15 % is not the array's
initialisation — it is the hand-written iteration. (b) **`collect` is
internal iteration and a hand-written loop is not.** A `Chain<Filter<_>>`
iterates internally through `fold` and externally through a state machine;
`Vec::from_iter` takes the internal path and every replacement here took the
external one. Rows 3 and 4 confirm it from the other side: moving back to
`for_each` (internal) but with a per-element accumulator body gave the cost
back and lost the `cube` win as well, because the closure's captured `n` and
spill vector defeat what the collect loop keeps in registers. **A collect is
not just an allocation — replacing one costs the iterator specialisation
too, and that is worth ~0.15 % of `fixed` here.** The way to keep both would
be a stack-backed `Extend` target (a `SmallVec`), which is a dependency
decision, not a code one.

**(-54b) CLOSED AT THE EIGHTY-THIRD PASS — AND NOT BY THE PROOF THIS ENTRY
ASKED FOR. `fixed` -1.073 %, `cube` -0.729 %; see Baseline (2).** Everything
below stays true and is the reason the *live game* keeps its checkpoint:
neither declaration is atomic, so no reordering removes it there. What the
entry did not consider is that the **simulation** does not need a rollback at
all — it owns its state and drops it unread on `false` — and that
`CRAB_SIM_REJECTS` has now read zero in 129 configurations, so the retry the
rollback existed to enable never fires. **The entry asked for a proof about
the engine when the answer was a count about the workload.**

**(-54b, as found) THE `sim_step` CHECKPOINT CANNOT BE REMOVED BY AN
ATOMICITY PROOF, BECAUSE NEITHER DECLARATION IS ATOMIC. DISPROVED BY READING,
NO BUILD.**
NEXT's item 1b asks for "an atomicity proof, not a deletion" of
`perform_action`'s checkpoint on the two declaration kinds (1.08 % of `cube`).
Both functions mutate before their last `Err`:

```text
declare_attackers_banded        combat.rs (line numbers at f51e695b)
  1259  try_pay_with_auto_tap    <- the CR 508.1g attack tax, PAID
  1306  return Err               <- Floodtide Serpent's cost unpayable
  1313  move_card_to             <- that cost, APPLIED
  1359  return Err               <- Leviathan's sacrifice cost unpayable
  1366  sacrifice_one            <- that cost, APPLIED
  1387  return Err               <- Hollow Warrior's tap cost unpayable
declare_blockers                (line numbers at f51e695b)
  1995  try_pay_with_auto_tap    <- the CR 509.1b block tax, PAID
  2022  battlefield_find_mut     <- a block cost, APPLIED
  2039, 2055, 2089, 2114, 2137, 2166, 2194, 2222   eight more `return Err`
```

Each cost family is already select-all-then-apply *internally*; what is not
atomic is the **sequence** of families, and reordering them so all four
select before any applies is not behaviour-preserving: the tax taps lands, and
`find_tap_helper` looks for an *untapped* permanent, so a tap-another cost
would gain access to the lands the tax spent. That is a simultaneity question
(CR 601.2h) with a real answer, not a refactor. **The checkpoint is the
cheapest correct implementation of it**, and this entry exists so the next
run prices the restructure rather than the deletion.

**What is left of (-13) after this**: the *live* game's checkpoint on every
non-pass action. It is pinned by `cow::tests::rejected_action_restores_state_
exactly`, it is what structurally kills the audit-P0 partial-mutation family,
and narrowing it is a rules-correctness argument first. The simulation's half
is gone and there is no third half.

**(-58) THE COMBAT DAMAGE BATCH'S PER-PAIR GATHER IS WORTH `cube` -1.6 %,
AND SHARING IT IS UNSOUND. BUILT, MEASURED, REFUTED — WITH THE
COUNTEREXAMPLE.**

`resolve_combat_damage_with_filter` opens three `&self` freeze scopes inside
its pair loops — ~12,700 on a six-game `cube` run — and each takes its own
gather, which is 68 % of `resolve_combat`'s `computed_permanent` calls and
**2.61 % of the program**. Seeding every scope from one gather taken at the
top of the batch (`with_frozen_effects`, re-derives per-permanent views inside
the scope so counters and damage marks are still seen) measures:

```text
callgrind, --games 6 --threads 1 --seed 1
  --decks fixed   1,168,055,373 -> 1,165,974,624   -0.178 %
  --decks cube    3,561,558,835 -> 3,504,054,147   -1.615 %
```

`--bench` byte-identical, `CRAB_SIM_REJECTS=1` identical on nine pool/seed
cells, suite 18,795 / 0 / 5. **And it is wrong.** The seed carried a
`debug_assert!` comparing it against a fresh gather on every call, and a
ladder run built with `-C debug-assertions=yes` fired it on `cube --seed 3`
inside 60 games:

```text
FXDIFF seeded=0 fresh=1
  +ADD ts=11 src=CardId(119) name="Ulna Alley Shopkeep"
       L=L7PowerTough m=ModifyPowerToughness(2, 0) affected=Source
```

**"Infusion — +2/+0 as long as you've gained life this turn."** A lifelink
blocker deals damage in the *same batch*, its controller gains life, the
static's condition flips, and the gather grows an effect. The epoch this was
guarded with — `(battlefield.len(), continuous_effects.len(),
next_effect_timestamp)` — cannot see it in any direction: nothing entered or
left, and the effect is **derived**, so it carries the source's old
`battlefield_timestamp` (11) rather than a fresh one.

**The rule, and it is the reusable half: a continuous-effect gather is a
function of the whole game state, not of a few collections.** Player life
totals are a layer input. Any memo that spans a mutation therefore needs
invalidation *at* the mutation, which is the board epoch — built, measured
and refuted at the forty-fifth pass, **(-18)**. What is left of the 2.61 %
needs either that, or a way to make the gather itself incremental. **Do not
re-take the seeded-scope shape without one of those.**

**And the device that caught it is worth more than the entry.** A memo whose
soundness is an argument gets a `debug_assert!` comparing it against the
thing it replaces, and then the suite and any `-C debug-assertions=yes`
ladder run are the audit — 18,795 tests missed this and 60 games of `cube`
found it in four seconds, because the suite has no Shopkeep-plus-lifelink
board and the ladder deals it every few games. **Build the audit before the
optimization; it is one `debug_assert!` and it is the difference between a
refutation and a silent wrong game.**

**(-56) HALF-REFUTED AT THE EIGHTY-FIRST PASS: THE `sa_cards` RESERVE IS THE
FIFTY-FOURTH PASS'S TRAP ON A SECOND VEC, AND IT SPLITS BY POOL.**
`gather_continuous_effects_inner` runs **71,884 times on a six-game `cube`
run and grows a Vec 69,890 times — 0.97 growths a gather**, which is
`sa_cards`' first allocation and (on a busier board) its first doubling. Two
ways to remove it, both built and measured against `32fa1675`:

```text
                                                 fixed        cube
  Vec::with_capacity(battlefield.len())         +0.439 %     -0.242 %
  Vec::with_capacity(<counted static cards>)    +0.371 %     -0.041 %
```

**Neither ships.** The whole-board reserve is the fifty-fourth pass's finding
exactly — a bigger allocation on every gather costs more than the growths it
removes — and it only *looks* different here because the element is 16 bytes
rather than ~100, so the trade flips sign between the two pools instead of
being negative on both. The exact count is worse still: the extra battlefield
walk (~15 cards of `static_abilities.is_empty()`) costs about what one growth
costs, and on `fixed` there was **no** growth to remove — `sa_cards` never
outgrows its first capacity there, so the reserve is pure loss. **A growth
count is per-pool, and a reserve that pays for itself on one board is a tax
on the board that never grows.** What is left of this entry is
`compute_permanent_pass`'s 51,706 growths (19,552,222 Ir) — **and a
concurrent session took exactly the shape this entry named** at `31eb7333`:
`Printed<Vec<_>>`'s materialize, where `Vec::clone` hands back
`capacity == len` so the first layer write reallocates and memcpys the whole
printed list. `fixed` -0.085 %, `cube` -0.591 %. That is the third reading of
the same trap and the only one where the fix is **exact** (`len + 1`) rather
than headroom, which is why it is also the only one that pays on both pools.

**AND THE FREEZE SCOPE IS ALREADY THERE: the candidate menus are inside one,
and wrapping them again is a no-op to five decimal places.**
`block_candidates_for_mcts` and `attack_candidates_for_mcts` call
`pick_blocks` / `pick_attacks` and their helpers, each of which opens its own
`with_frozen_layers`; wrapping both menus in one outer scope so the inner ones
share a memo read **`fixed` +0.001 %, `cube` +0.0004 %** — the push and the
pop and nothing else. `HeuristicBot::next_action` already runs *the whole
tick* inside one scope (bot.rs:1661), so every read on the live state was
memoized before any of this. **Check for an enclosing scope before proposing
one.**

**So where do 48,810 unfrozen `computed_permanent` gathers come from?** From
the sims, and the table is worth keeping (`--separate-callers=3`,
`cg_contexts.py`, `--decks cube`, eighty-first tip):

```text
71,884 gathers, by calling context
  8,598  computed_permanent <- {closure} <- Vec::from_iter
  6,956  computed_permanent <- resolve_combat <- advance_step
  6,546  frozen_effects <- board_keyword_in_scope <- declare_attackers_banded
  6,036  compute_permanents <- combat_damage_computed <- resolve_combat
  5,816  computed_permanent <- resolve_combat <- submit_decision
  5,394  computed_permanent <- permanent_value_with <- eval_material_inner
  4,870  frozen_effects <- board_keyword_in_scope <- declare_blockers
  4,290  computed_permanent <- with_frozen_layers <- declare_blockers
  2,184  check_state_based_actions <- resolve_combat <- advance_step
```

**`resolve_combat` drives ~21,500 of them, 30 % of every gather in the
program** — `resolve_combat -> computed_permanent` is 18,888 calls /
92,837,604 Ir / **2.61 % of `cube`**, and 68 % of those calls rebuild the
list. It runs inside `sim_step` on the sim's *cloned* state, whose
`LayerFreeze` is `default()` (see `Clone for GameState`), so the tick's scope
does not cover it — and it **cannot** simply be wrapped, because it mutates:
damage, deaths, triggers. A memo that survives a mutation is a stale layer
view, which is a wrong game, not a slow one. The shape that would work is an
invalidating memo, i.e. the board epoch — **built, measured and refuted at the
forty-fifth pass, (-18)**. What has *not* been tried is freezing the
read-only sub-regions between `resolve_combat`'s mutations, or handing its
readers the `combat_damage_computed` snapshot it already builds.

**(-56, as found) THE ALLOCATION TABLE IS A THIRD REALLOCATION, AND TWO
CALLERS OWN HALF OF IT.** Read at the eighty-first tip, `--decks cube`, the table that
has found the most in this file (callers of `__rust_alloc` by *count*):

```text
1,988,682 allocations
  521,425  RawVecInner::finish_grow          26.2 %   <- a Vec that outgrew its capacity
  261,200  Arc::clone_from_ref_in            13.1 %
  227,436  GameState::computed_permanent     11.4 %
  174,717  Vec::from_iter (nested)            8.8 %
  122,896  <GameState as Clone>::clone        6.2 %

callers of `grow_one` (607,072 of the 715,503 `finish_grow` calls)
  69,890  gather_continuous_effects_inner    14,742,457 Ir
  62,866  Vec::push_mut                      12,144,426
  51,706  layers::compute_permanent_pass     19,552,222
  37,670  stack::advance_step                 4,522,534
  36,478  combat::declare_blockers            8,253,535
```

**A quarter of every allocation in the program is a `Vec` that started at
zero and doubled**, and the two biggest are inside the layer machinery. The
fifty-fourth pass already took the top-level one (`all_effects` is sized off
the `sa_cards` walk) and its note is the warning to read first: a blanket
`+ battlefield.len()` headroom measured **+1.54 %**, because a bigger
allocation on all 32,002 gathers is worse than the 10,040 growths it removes.
So this entry wants **exact or near-exact** sizes, not headroom —
`sa_cards`'s own `Vec::new()` (~2.2 growths a gather) is a two-pass count
away, and `compute_permanent_pass` needs a line profile before anyone guesses
which of its pushes grow.

**(-57) MOSTLY ANSWERED BY THE CONTEXT TABLE ABOVE, AND THE ANSWER IS "ONE
GATHER PER EVALUATION, WHICH IS THE FLOOR".** `eval_material` opens its own
freeze scope (bot.rs:10979), so of its 36,668 `computed_permanent` calls only
**5,394 rebuild the list** — one per evaluation, on the sim's cloned state
where no outer scope exists. The remaining 31,274 are memo probes. The entry
below over-stated the prize by 6x; what is actually available is the
per-evaluation gather, and that is the same question `resolve_combat` asks.

**(-57, as found) `eval_material_inner` IS THE LAST BIG `computed_permanent`
CALLER — 36,668 calls / 57,985,103 Ir / 1.63 % OF `cube`.** It walks the whole
battlefield and asks `permanent_value_with` for every non-land permanent,
which is one `computed_permanent` apiece. Unlike (2) in the Baseline above
this is **not** a repeated question — one pass per evaluation — so the shape
that pays is not "resolve once" but "resolve the board in one call":
`apply_layers` computes `SecondPass::of(effects)` once for the whole
battlefield where `apply_layers_one` recomputes it per card. **Price
`SecondPass::of` first**; if it is small the entry is closed, and if it is
not it is the same win on every whole-board consumer, not just this one.

**Ranking rule added by the fifty-third pass, and it is about the workload,
not the code: ask which pool the change lives on before you cost it.** Two
of that pass's finds were larger than anything in this list and neither was
visible on the bench — the per-card grant walk (49 % of a cube game) and
deck construction (96 % of a deck build, and a training actor builds two
decks a game). "Which pool a change moves" at the top of this file is the
device; the short version is that `--decks fixed` carries no
`GrantTriggeredAbility` static and builds its decks once.

**(-55) PARTLY PAID at the seventy-sixth pass — five picker/engine
disagreements fixed, and every *block* rejection on three of the four
workloads is gone.** Same instrument, `--games 20 --threads 1`:

```text
                     before                             after
cube seed 7    82/9,664  (0.85 %) atk 18  blk 64    0/9,862    (0.00 %)
cube seed 11  434/13,034 (3.33 %) atk 324 blk 110   372/13,428 (2.77 %) atk 324 blk 48
all  seed 3    64/33,608 (0.19 %) atk 0   blk 64    0/33,714   (0.00 %)
sos  seed 5     0/6,892                             0/6,892
```

The five are in ENGINE_BACKLOG P3 with the regression tests; four of them are
one shape — **a legality question answered off the printed or instance view
where the engine answers it off the computed one**. What is left is seed 11's
324 attack rejections, and the `Angel` row below is still the lead: **build
the per-site tag on `declare_attackers_banded`'s thirty `CannotAttack` returns
first**, because nothing in the card's own keywords explains it.

**The fix costs `cube` +3.97 % of Ir** and the Baseline block says why: blocks
that used to be rejected as a batch now happen, so the games are longer. The
direct cost is `blocker_can_block_attacker_pair` +0.08 %.

**Two more, on the attack side, landed the same hour (`d0d1162d`).** The
picker filtered attackers on `c.definition.is_creature()` — the *printed*
line — and consulted the computed view for Defender and `CantAttack` but
never for creature-ness, so a **bestowed Kestia (an Aura) and a de-animated
Vehicle** were declared and the batch was rejected whole. And
`is_creature_now` leads `declare_attackers_banded`'s conjunction but was
missing from the error cascade under it, so that rejection came out as
**`SummoningSickness` on a card whose `summoning_sick` is `false`** — the
contradiction that made the census legible in the first place. 52 -> 0 on
`cube --seed 11 --games 12`; `decisions` byte-identical on all three pinned
callgrind workloads, which carry no permanent of the shape.

**THE PER-SITE TAG IS BUILT AND THE RESIDUAL IS ONE RULE, NOT TEN.**
`attack_reject(line!(), e)` now wraps all twenty-eight of
`declare_attackers_banded`'s rejection returns (off behind
`game::reject_trace_level`, the reader `bot::sim_rejects` shares). On
`--decks cube --seed 11 --games 12 --threads 1`:

```text
718  combat.rs:1188  CannotAttack   CR 508.1g — the attack tax
 22  combat.rs:710   CannotAttack   CR 508.1d — "attacks each combat if able"
  0  the other twenty-six sites
fixed / sos / sealed: zero attack rejections at any site
```

**97 % of them are the attack tax, and the picker does not model it at all.**
Propaganda / Ghostly Prison / Oppressive Rays / Sphere of Safety:
`pick_attacks_inner` declares the whole board, `try_pay_with_auto_tap` cannot
pay the sum, and the engine rejects the **batch**, blaming `attacks[0]` —
which is why the card census read "154 `CannotAttack(Angel)`" when Angel was
merely first in the list. **Naming the card was the wrong question and cost
two rounds; the site tag answered it in one.** In the simulation the fallback
then passes priority, so the modelled opponent attacks with *nothing*; on the
real declaration path the action is rejected outright, so against a
Propaganda this bot may never attack.

**THE ATTACK TAX IS PAID — and it is the first change on this branch gated
on a win rate rather than an argument.** `attack_tax_for(attacks, statics,
keyword_tax)` is the extracted walker; `declare_attackers_banded` charges it
and `trim_attacks_to_payable_tax` (last in `pick_attacks_inner`, because the
tax depends on what each attacker is aimed at) prices the batch against
`available_mana` and drops taxed attackers by damage-per-mana ascending.
Untaxed attackers are never dropped and neither is a must-attack one — CR
508.1d would reject the batch for its absence instead — and the budget stays
deliberately optimistic, so the trim errs toward letting the engine reject
rather than declining a legal attack.

```text
CRAB_SIM_REJECTS=1, --games 12 --threads 3      base            tip
cube  seed 11                                 156/8,656       54/8,596
  of which attack                                 110              10
cube seed 1 / seed 42, fixed/sos/sealed seed 11   unchanged (0, 4, 0)
```

**Strength, `bot_ladder --vs`, `release-fast`, tip = A, 400 games x 8 cube
archetypes = 3,200 games a seed.** The eight seeds are the ones whose pool
carries a tax at all (see the sweep below); the pools that carry none play
byte-identical games, which is the change's own null.

```text
seed     8     9    10    11    18    20    21    22   total
A-sw     3     5     0    18     2    11     5     5     49
B-sw     1     4     0     3     2     3     1     0     14
A win% 50.1  50.0  50.0  50.5  50.0  50.2  50.1  50.2
cube s5 / s23 (no tax): 1,600/1,600 pairs split, 50.0 %
fixed s11    (no tax):    800/800   pairs split, 50.0 %
```

49 decisive pairs to A against 14 to B over 25,600 games, never worse on a
seed. Clock: `ab_wall.py`, 6 ABBA blocks, `--decks fixed`, where play is
identical both sides so the timing is the added `attack_static_scan` and
nothing else — **mean +0.56 %, CI -0.26 .. +1.39 %, FLAT**, and the null
control on the same workload resolves only +/-2.34 %. `--bench` byte-
identical (195,616 / 27.44 / 0 stalls).

**AND FOUR MORE OF THE SAME SHAPE, ALL FOUND BY A SITE TAG AND ALL FIXED THE
SAME WAY.** Every one is a per-attacker or per-blocker rule the planner did
not model, whose violation the engine rejects the declaration **whole** for —
so the cost is the bot's entire combat, not one body, and in a simulation the
`sim_step` fallback then passes priority so the modelled side declares
nothing at all.

```text
site              rule                       pool/seed    before  after
combat.rs:771     CR 613 Ensnaring Bridge    cube s10        410      0
combat.rs:710     CR 508.1d must-attack      cube s11         22      0
combat.rs:2324    CR 702.39 Provoke          cube s11         82      0
combat.rs:2136    CR 509.1b Menace subsets   cube s1 / sealed 16      0
combat.rs:1762    CR 509.1a computed CantBlock  fixed s11      4      0
```

Two devices did all of it. **A shared predicate where the planner had its own
copy** — `attack_requirement_able`, `provoked_block_is_able`, and
`attack_tax_for` are each one method the engine and the picker both call.
**And a repair pass over the search menu**, because
`attack_candidates_for_mcts` and `block_candidates_for_mcts` both offer
*subsets* of a legal declaration, and a subset can leave an obliged attacker
home, release a provoked creature, or strip the second blocker off a Menace
attacker. Those candidates were not rejected loudly — their opening dry run
failed, they scored `None`, and the menu silently shrank.

**None of the four moves strength measurably**, and that is the honest
statement: `--vs` reads 6 A-sweeps to 4 on the Bridge seed (9,600 games),
2 to 2 for the must-attack repair (19,200 games), 8 to 1 for the block three
(3,200 games, 50.2 %). Only the attack tax has a win rate. **They are worth
having as correctness**, and the census is what says so: 740 attack and 102
block rejections at the seventy-ninth tip, **44 and 6** now.

**AND THE "44 AND 6" ABOVE WAS A THREE-SEED SAMPLE, WHICH IS THE SAME
MISTAKE THIS SECTION ENDS BY WARNING ABOUT.** A sweep of `cube` seeds 1-24
and 42 at `--games 20` — four seconds a seed — found **186** block rejections
across **eight** seeds where the three-seed census read 6. None of them is a
rule the fixes above missed; they are four rules nothing had reached, because
no seed in the sample carried them:

```text
cube, --games 20 --threads 1, block rejections    32fa1675    tip
s15   CR 509.1c Lure, and the contradiction below      92       6
s23   CR 509.1g CantBeBlockedByMoreThanOne             50       0
s5    CR 509.1c true Lure (AllMustBlock)               18       0
s42   CR 509.1c MustBlock, asked of the blocker         8       0
s19   Lure                                              8       0
s13 / s24 / s10                                    4 / 4 / 2    0
                                             total     186       6
```

`all` seeds 15 and 23 go 4 -> 0 and 8 -> 0 on the same change; the attack half
is untouched on every seed. **So the standing rule now applies to the census
itself, not only to `--vs`: sweep 1-24 before writing that a half is closed.**
A `--games 8` sweep over 24 seeds is ~90 s and it is the cheapest claim-check
in this file.

**And one of the eight was not a planner bug at all — it was a board with no
legal block declaration.** An attacker with Lure *and* Menace facing exactly
one able blocker: block with nobody and CR 509.1c rejects it, block with the
one body and CR 509.1b rejects it. CR 509.1c already resolves it — the
defender picks the *legal* declaration satisfying the most requirements, so a
requirement no legal declaration can meet does not bind — and
`block_requirement_binds` is that gate, consulted by all four requirement
loops and by the planner. **Worth generalising: wherever this engine models a
"must" and a "can't" as two independent checks, the pair can be
unsatisfiable, and the census is the only thing that finds it** — the tag
named CR 509.1b on a board whose bug was CR 509.1c.

**A note on what the site tag cannot do, earned over two wasted builds.** It
names the *clause* that rejected a declaration; it never names the *pass that
built it*. Two plausible fixes to the wrong pass measured as exactly inert
before a probe printing the plan at the point of rejection showed the pin came
from somewhere else. If this recurs, build that probe first.

**AND THE MEASUREMENT LESSON, WHICH IS ABOUT THE POOL AND NOT THE CODE.**
This file briefly recorded that the `--vs` ladder "cannot gate that fix"
because `--bench` and the wide sweep read bit-identically across the walker
extraction. That is true of `fixed` — four hand-built decks, no Propaganda —
and **false of `cube`, whose deck *content* is seed-dependent**, which is a
rule this file already states two sections up. `CRAB_SIM_REJECTS=1` at
`--games 6` costs ~4 s a seed and answers it outright:

```text
cube seeds 1-24, base binary, attack rejections
  0 at seeds 1-7, 12-17, 19, 23, 24
  6/1,426 (s8)  16/1,422 (s9)  84/1,428 (s10)  92/2,020 (s11)
 12/1,242 (s18) 20/1,854 (s20)  8/1,282 (s21)  60/2,000 (s22)
```

**Sweep the census before concluding a pool cannot reach the code.** The
same sweep is how to find a gating pool for the next play-changing fix.

**AND THE ATTACK HALF REACHES ZERO, BY DELETING AN ESTIMATE RATHER THAN
CORRECTING ONE.** Every attack rejection left anywhere was the same shape:
`trim_attacks_to_payable_tax` trims until `available_mana(state, seat).total`
covers the CR 508.1g tax, the engine then pays with `try_pay_with_auto_tap`,
and on ordinary `cube` boards the two disagreed by one or two mana.
`payable_generic_budget` asks `could_pay_generic` — the engine's own dry run —
with a binary search that costs **one** probe whenever the declaration is
affordable, and only on a board that charges a tax at all.

```text
CRAB_SIM_REJECTS=1, cube 1-24+42 at --games 20 plus four pools x five seeds
                     base 50a075fa      tip
cube s2        32/13,290 (0.24 %)         0
all  s11       36/11,898 (0.30 %)         0
cube s19                       16         0
cube s20 / s21 / s22       12 each        0
cube s11                       10         0
cube s15        6/10,340 (block)          6   the one open CR 509.1c site
                    ----------------------------
              total          118          6
```

**The general rule, and it is the durable part: an estimate consumed as a
*budget* has an asymmetric failure mode.** `available_mana` is biased
*downward* everywhere on purpose — sac costs, dynamic amounts and doublers are
left out so the bot does not commit to a line it can only pay by spending
something it would rather keep. That is right for a *casting* decision, where
an over-estimate wastes a probe. It is wrong for a *trim*, where an
over-estimate does not make the bot optimistic — it makes the engine reject
the declaration **whole**, and the bot attacks with nobody. Two consumers,
opposite requirements, one number. Ask the engine at the sites that trim.

**Cost, and it is real: `cube` +1.83 %.** `ab_wall`, 6 ABBA blocks, 400 games
x 8 archetypes, seed 11, 3 threads: mean B/A 1.0183, **CI +0.35 .. +3.32 %**,
0 of 6 blocks faster, against a null on the same workload resolving
+/-1.47 %. That is a `mana_source_table` build (6,690 Ir at the (-51) sizing)
and its taps, per taxed declaration. **The cheaper form does not fall out:**
`ManaSourceInfo` carries a colour set and a cost rank but **no amount**, so
summing the table is not a sound bound — a filter land (`{R}, {T}: add
{R}{R}`) is an entry that nets nothing. A non-mutating "what can auto-tap
reach" that returns a quantity is the open lever here.

**And measure the budget where it is wrong, not everywhere it is used.**
Doing the same to `trim_blocks_to_payable_tax` reads **+2.11 % (CI +0.99 ..
+3.24 %)** — significant where the attack half is marginal — because that trim
runs once per *candidate* in `block_candidates_for_mcts` rather than once per
declaration, and it buys nothing: the block half was already 0 on every pool
and seed swept. Shipped attack-side only.

**Strength: flat.** `--vs` against the base, 400 games x 8 archetypes on each
of the four taxed seeds (12,800 games): s2 9 A-sweeps to 5 at 50.1 %, s22 4 to
1, s11 2 to 0, s19 0 to 1. Directionally positive and resolvable by nothing at
this sample — which is what 118 rejections in a 20-game census predicts.

**AND THE LAST SIX WERE A CONTRADICTION BETWEEN TWO *REQUIREMENTS*, WHICH IS
THE 509.1b/509.1c TRAP ONE LEVEL UP. (-55) IS CLOSED.** Two requirements that
name the same creature can never both be satisfied — it blocks one attacker —
and the five requirement loops each asked whether *their* requirement was met
in isolation. A Lure attacker, a provoker, and one able defender: block
nobody, block the Lure, block the provoker, **all three rejected**. CR 509.1c
already resolves it — the defender satisfies the *maximum number* of
requirements, and a creature blocking something that obliges it is the most
any declaration gets out of it — so `block_spoken_for_elsewhere` excuses a
blocker that is already assigned to an attacker whose own requirement binds
it. The planner needed no change: `enforce_block_requirements` pins one
requirement per blocker, first come, which was already the maximal
declaration; the checker would not take it.

```text
CRAB_SIM_REJECTS=1, every configuration run          rejections
cube 1-24 + 42   --games 20                                   0
cube 25-45       --games 12                                   0
all/sealed/sos/fixed seeds 1-12  --games 8                    0
                                                    -----------
                          69 configurations                   0
```

From 470/91,438 when the instrument landed, 118 at `50a075fa`, 6 before this.
Flat on the clock (`ab_wall` 6 blocks on `cube` s15, mean 0.9894, CI -3.23 ..
+1.12 %, null +/-2.18 %) and `--bench` byte-identical.

**Two notes on method, both earned expensively.**
**`--vs` cannot gate a change to how the engine *resolves*** — it compares how
two binaries *choose*, and this one read 50.0 % with all 160 pairs split over
320 games on the very seed that carries the board, because the cross-play
schedule draws its own jitter and never reached it. The census and a direct
`declare_blockers` test are the gate.
**The site tag names the clause that rejected a declaration and never the
pass that built it.** Two plausible fixes to the wrong pass measured *exactly*
inert before a throwaway probe printing the plan at each pass boundary found
the cause in one run. Build that probe first.

**(-55, as found) THE SIMULATION'S OWN PICKERS PROPOSE DECLARATIONS THE ENGINE THROWS
OUT — 470 of 91,438, AND ON ONE `cube` BOARD 6.8 % OF THE ATTACKS.** Measured
with `CRAB_SIM_REJECTS=1` (see "How to measure"), `--games 12 --threads 3`,
four pools x three seeds:

```text
pool     seed   rejected / proposed          attack        block
fixed    1/11/42   28/6,644  24/6,002  24/4,782      0        28 / 24 / 24
cube     1         42/7,744  (0.54 %)             0/3,174     42/4,554
cube     11       330/8,920  (3.70 %)         258/3,784      72/5,088
cube     42         6/9,116  (0.07 %)             0/3,716      6/5,376
sos      1/11/42    0 / 10 / 0                    0            0 / 10 / 0
sealed   1/11/42    0 / 0 / 6                     0            0 / 0 / 6
```

**`Picked::Plain` never fails: 0 rejections in 88 proposals**, which is what
`would_accept_on`/`accept_on` having already run it predicts. Every rejection
is a *declaration*, and `CRAB_SIM_REJECTS=names` names them:

```text
154  CannotAttack        Angel                    sick=false  kw=[Flying]
 52  SummoningSickness   Kestia, the Cultivator   sick=false  kw=[]
 24  CannotAttack        Kestia, the Cultivator   sick=false  kw=[]
 20  CannotBlock         Veteran Armorer          sick=false  kw=[]
 16  MustBeBlockedIfAble Crested Craghorn         sick=false  kw=[Haste]
 12  CannotAttack        Whitemane Lion           sick=false  kw=[Flash]
 10  CannotAttack        Offender at Large        sick=false  kw=[Disguise]
```

**`SummoningSickness` on a card whose `summoning_sick` is `false` is a
contradiction and the best lead in the table** — the picker's own gate is
`!c.summoning_sick || c.has_keyword(Haste)` read off the *instance*, where
`declare_attackers_banded` reads the computed view, so this is the
presence-vs-computed shape the seventy-first and seventy-fifth passes both
found one level up. `Angel` at 154 is one token on one board against one of
`declare_attackers_banded`'s thirty `CannotAttack` sites; **that function
needs a per-site tag before the next run can bisect it**, which is the first
thing to build here. Each rejection costs a full `perform_action` +
checkpoint + restore plus the retry pass, so this is a correctness lead with
a perf tail, not the other way round — and at 0.51 % overall the perf tail is
small. Fix the pickers, not the fallback.

**(-54) CLOSED at the seventy-fifth pass — THE FALLBACK IS LOAD-BEARING AND
HERE IS THE NUMBER.** The entry asked for a failure count before anything
else: it is **470 of 91,438 non-pass `sim_step` calls (0.51 %)**, non-zero on
every pool and as high as 3.70 % on one `cube` seed (table in (-55) above).
So `sim_step`'s rollback-and-retry really does run, the checkpoint stays, and
`fallibility_closure.py` agrees from the static side — `declare_attackers`
reaches 6 `Result` functions and 2 raise, `declare_attackers_banded` alone
carrying 38 `Err(` sites. The one shape that *is* provably infallible is
`Picked::Plain` (0/88), and it is 0.5 % of the non-pass calls, so a fast path
for it buys nothing. **What the count did find is (-55).**

**(-54, original) THE SIMULATION TAKES A TRANSACTION CHECKPOINT ON A STATE IT
OWNS AND THROWS AWAY — ~0.93 % OF `cube`, AND THE MISSING NUMBER IS A FAILURE
COUNT.**
`sim_step -> perform_action` is **4,322 calls / 177,669,748 Ir / 6.66 % of
`cube`** at the seventy-fifth tip, and ~5,750 Ir of each is the checkpoint
(-13) prices: clone 1,194, drop 2,324, plus the CoW unshares the action pays
only because the checkpoint re-shared zones the sim had already unshared. The
sim's `g` is a throwaway clone; the *only* reason the checkpoint is not
removable is `sim_step`'s documented fallback, which rolls a rejected
declaration back and retries it as a priority pass — and `declare_blockers` /
`declare_attackers_banded` hold 82 of the engine's `Err` sites between them.

**(-53) THE COST-STATIC WALKS THAT SURVIVE THE SEVENTY-SIXTH PASS —
13,696,074 Ir / 0.52 % OF `cube`, AND THE FIX IS AN ENUMERATION.**
`can_afford_in_state_with` now walks only the sources that carry
`static_abilities` (see (-34) and the seventy-sixth Baseline), which took 55 %
of the three edges. What is left is walking those sources' statics per hand
card for families a normal board does not have: `AdditionalCost` and its eight
siblings (`extra_cost_for_spell`), `ColoredSpellTax`
(`colored_spell_tax_for_spell`), and the twenty-two-variant reduction family
(`cost_reduction_for_spell_full`). A bitmask over `CostStaticSources::gather`'s
existing walk gates all three; it is the `cast_cost_scan` device exactly, with
its `debug_assert!`-at-the-gated-site audit, and the non-board channels
(`first_spell_tax_charges`, `turn_scoped_spell_taxes`, `turn_spell_discounts`,
`extra_cast_reduction`, and the ~10 card-intrinsic reduction fields) stay
outside the gate as cheap per-card reads. **Price the ~30 variants of drift
surface against 0.5 % before taking it** — the structural filter that already
shipped needed none.

**CLOSED AT THE EIGHTIETH PASS, AND NOT BECAUSE IT SHRANK — BECAUSE THE LOOP
IT WOULD GATE NO LONGER EXISTS.** Re-sized on the current tip, same workload
(`--decks cube --games 6 --threads 1 --seed 1`):

```text
                              recorded        now
  the three families        13,696,074     7,448,194 Ir   -46 %
  share of cube                  0.52 %        0.209 %
  callers, now:  extra_cost_for_spell        7,360 <- cast_spell_with_convoke
                 cost_reduction_for_spell_full 7,360 <- cast_spell_with_convoke
                 colored_spell_tax_for_spell     0 <- the cast path at all
  can_afford_in_state_with: 32,570 calls, and its callee list reaches
  NONE of the three.
```

**The entry's premise was "walking those sources' statics *per hand card*",
and the bot does not do that any more** — the seventy-sixth pass's structural
filter took the last of it, and what remains is the engine paying each family
**once per actual cast attempt** on the real cast path. So the `*_scan` rule
answers it: *a bit pays for the walks it removes from a loop, so count the
loop's trips*, and the trips are now one. A per-cast scan costs about what
the walk it replaces costs; the only version that could win is one **cached
across casts and invalidated on board change**, which is the board-presence
epoch on TODO's do-not-rebuild list. **Do not take this. ~30 variants of
drift surface for at most 0.209 %, against a device that is refuted.**

Note the share fell further than the work did: the program grew 2.63 G ->
3.57 G over the same passes (the attack search), so a third of the drop in
*percent* is denominator. **Re-size a candidate before ranking it, and read
the absolute Ir next to the share** — one of them is about your code and the
other is about everyone else's.

**(-52) CLOSED — ACTOR SCALING IS LINEAR TO THE CORE COUNT, AND RSS IS THE
REPLAY WINDOW, NOT THE ACTORS.** Measured, not inferred:
`release-fast selfplay_train --games 1200 --steps 1 --seed 7`, two reps, on
the 4-core 2.80 GHz Xeon.

```text
actors   games/s (rep 1 / rep 2)   per actor   vs 1 actor
1        37.2 / 41.1               39.2        1.00x
2        79.6 / 80.2               40.0        1.02x
3       122.1 / 115.5              39.6        1.01x
4       165.2 / 156.2              40.2        1.03x
6       169.8 / 160.0              27.5        +2.6 % over 4 — CPU-saturated
```

**There is no shared-state contention to find**: per-actor throughput is flat
to the core count and the sixth actor buys nothing on four cores. The seed
list's "find contention if sublinear" is answered — plan **one actor per
core** and stop.

**And the RSS planning figure this file carried is `bot_ladder`'s, not
`selfplay_train`'s.** Peak RSS (`VmHWM`, same runs, 1000 games):

```text
actors 1   968 MiB      window  25,000   520 MiB   (4 actors, 600 games)
actors 2   983 MiB      window 250,000   805 MiB   (same run otherwise)
actors 4   987 MiB
```

**An extra actor costs ~6 MiB. The replay window costs ~1.3 KiB a row.** A
`selfplay_train` process is ~0.5 GiB of fixed footprint plus its window, so
the thing to size against a box's memory is `--window`, and the thing to size
against its cores is `--actors`. The two knobs do not trade against each
other, which is the opposite of what "plan actor counts off ~24 MiB RSS"
implies.

**A caution about the other scaling number in this file.** `bot_ladder
--bench --threads N` reads 64.5 / 114.9 / 167.0 / 208.8 games/s at 1/2/3/4 —
**83 % per-thread efficiency at four threads**, which looks like contention
and is not: `--bench` is 320 games and 1.5 s of wall at four threads, so
process startup and the main thread's aggregation are most of the gap. The
actor sweep above runs 30 s at one actor and shows none of it. **Do not size
parallel efficiency off `--bench`.**

**(-51)(a)'s `make_mut` HALF IS TAKEN at the hundred-and-first pass.** The
~1,055 Ir of deep copy this entry priced inside the tap is gone:
`activate_ability_inner`'s `make_mut` edge on `cube` reads 49,240 calls /
19.95 M Ir -> 27,678 / 4.87 M, because `tapped` now lives on the
`CardInstance` handle rather than inside the `Arc` (Log, pass 101 (3)). What
is left of (a) is `card_keyword_possible` (0.91 % of `cube`) and
`card_type_change_unscoped` (0.40 %) — and the entry's warning against a
parallel activation walker still stands.

**(-51) A LAND TAP COSTS 7,555 Ir, AND A THIRD OF THE PAYMENTS ARE THROWN
AWAY.** Sized at `ee376912` on `cube`; the two halves share a call path and
neither has ever been costed in this file.

**(a) The tap.** `auto_tap_for_cost_inner -> activate_ability` is **21,566
calls / 162,930,901 Ir / 6.19 % of cube** — the second-largest single call
site in the simulator after the bot's dry-run probe. Every one of them is a
`{T}: add mana` ability run through the full CR 602.5 activation gauntlet.
The costed parts: `card_keyword_possible` **1,146 Ir** (its
`keyword_grant_in_scope` board walk is 726 of that — 467,366
`card_can_grant_keyword` calls come from here, 42 % of the program's 1.1 M),
`card_type_change_unscoped` ~506, `make_mut` ~1,055 (32,782 real deep copies,
the largest single source in the program — see the context table in Profile of
record), and ~1,700 of `activate_ability_inner`'s own frame.
**Do not open this with a parallel fast path**: a second activation walker is
the exact class ENGINE_BACKLOG P3 tracks. What it wants is either fewer taps
(see (b)) or a cheaper `keyword_grant_in_scope`, and the per-definition
keyword-grant bit that would do the latter is in TODO's do-not-rebuild list.

**(b) The waste.** `restore_payment_state` runs **3,712 times against 11,634
`try_pay_after_snapshot_mode` calls — 31.9 % of every payment the simulator
attempts is rolled back**. A failed payment has already built its
`mana_source_table` (8,986 calls / 60.1 M / 2.28 %, 6,690 Ir apiece) and
tapped whatever it *could* reach before `pay_for_spell` rejects the rest, and
`restore_payment_state` then unwinds all of it. Pro-rata that is ~1.9 % of
cube in taps plus ~0.7 % in tables, spent on payments that were never going
to complete.

**RE-SIZED AT THE SEVENTY-FIFTH TIP (`5e4ec3bd`), `--decks cube`, and both
halves have shrunk.** (a) `auto_tap_for_cost_inner -> activate_ability` is
**20,070 calls / 153,837,786 Ir / 5.77 %** (7,665 Ir a tap; it was 6.19 %),
of which `card_keyword_possible` is 21,274 / 24,216,608 / **0.91 %** and
`card_type_change_unscoped` 21,158 / 10,579,726 / **0.40 %** — that second row
is *not* the summoning-sickness gate (a land ETBs with `summoning_sick =
definition.is_creature()`, i.e. false) but the CR 106.12 `creature_source`
read one block earlier. (b) `restore_payment_state` is **2,416 against 10,134
`try_pay_after_snapshot_mode` calls — 23.8 %**, down from 31.9 %.
`mana_source_table` is 7,426 / 51,527,862 / **1.93 %**.

**And the rollbacks now have a caller table** (`--separate-callers=2`,
`cg_contexts.py`), which supersedes the seventy-third pass's:

```text
  1,672  <- cast_spell_with_convoke        of 6,342   26.4 %
    410  <- activate_ability_inner         of   816   50.2 %
    152  <- try_pay_with_auto_tap_mode     of   164   92.7 %
     92  <- cast_spell_alternative         of   162   56.8 %
     46  <- cast_flashback                 of    80   57.5 %
     26  <- cast_face_down                 of    26  100.0 %
     16  <- run_effect <- resolve_effect
```

**The pile is `cast_spell_with_convoke`'s 1,672 and the bot pre-filters that
path already** — `can_afford_from` tests `cmc + extra > have.total` *and* the
per-colour budget, so these are failures the bot's estimate did not predict:
an over-optimistic `total`, a cost the estimate does not model, or an auto-tap
that stranded a colour it could have covered. **The last of those is a
correctness bug, not a perf one** (a payable line becomes invisible), and
nothing here distinguishes them yet. **The oracle is the instrument** — see
1d — and it wants to classify a failed payment rather than a rejected
candidate. The engine-side bail this entry proposes cannot be sized until
that split is known: a colour bail needs an over-approximating colour set
(`effect_produced_colors` returns **empty** for `Restricted` /
`DevotionOfChosenColor` / `ChosenColorOfSource` payloads, and the *generic*
tap loop can still tap those sources), and a generic bail needs a per-source
mana *amount*, which `ManaSourceInfo` does not carry.

**(b) IS HALF PAID at the seventy-first pass — the bot half, and it read
`cube` -1.225 %.** Cast attempts 7,110 -> 6,038, payment rollbacks
3,696 -> 2,716, probes 11,986 -> 10,910, completed casts byte-identical. See
**Baseline**. What is left of this entry is **(a)**, the 7,555-Ir land tap,
and the *engine-side* bail below.

**AND THE INSTRUMENT THIS ENTRY ASKED FOR IS BUILT, WHICH REFUTES THE LINE
THAT USED TO SIT HERE.** It read *"the remaining 2,716 rollbacks are the ones
whose shortfall is generic rather than coloured, which no per-colour budget
can see"* — an inference, never measured. `CRAB_PAY_FAILS` (see "How to
measure") classifies each failure by the `ManaError` it actually failed with:

```text
--games 12 --threads 1       fails/attempts        generic  coloured  hybrid
cube   s1                 5,478/21,174  25.87 %      2,570     2,658     250
cube   s11                3,494/20,502  17.04 %      1,478     1,988      28
all    s11                7,514/43,404  17.31 %      3,066     4,384      64
all    s1                 5,084/36,068  14.10 %      2,970     1,888     226
sealed s11                3,220/23,346  13.79 %        654     1,928     638
sealed s1                 2,304/23,502   9.80 %        762     1,432     110
sos    s11                  956/9,114   10.49 %        288       604      64
sos    s1                   754/8,324    9.06 %        374       368      12
fixed  s11                  764/12,018   6.36 %          4       760       0
fixed  s1                   704/12,002   5.87 %          0       704       0
```

**Coloured is the larger class on four of the five pools, and on `fixed` it is
100 %** — 704 of 704, zero generic. The per-colour budget has not run out of
room; the residue is the *assignment* problem behind it, which is the half
refuted below for multi-colour subsets and still open for singletons. A
generic bail sized off the old sentence would have been aimed at a third of
`fixed`'s zero.

**The pool split is the usable part.** `fixed` is pure colour, `cube` is about
half and half, `sealed` carries the only real hybrid population (638 on s11).
Anything aimed at this row says which pool it moves before it claims a win.

**Cross-validated:** at this entry's own workload (`cube --games 6 --threads
1`) the census reads 2,814/11,470 = **24.53 %** against callgrind's
2,416/10,134 = 23.8 % at the seventy-fifth tip — two independent instruments
on one population, nine passes of code apart.

**AND THEN THE SPLIT THAT REWRITES THE ENTRY: 99 % OF THOSE ROLLBACKS ARE THE
BOT *ASKING*, NOT THE BOT BEING WRONG.** A failed affordance probe is the
answer mechanism working; a failed *committed* payment is an estimate that
missed. Both roll back identically, and this entry's "spent on payments that
were never going to complete" reads as if they were all the second kind.
`in_probe` is a thread-local depth counter (not a `GameState` field: the probe
runs on a clone, and a field would unshare the cold group per template to
answer a question only the census asks):

```text
--games 12 --threads 1     failures    probe   committed
cube s1 / s11           5,478/3,494  5,468/3,418     10/76
all  s1 / s11           5,084/7,514  5,084/7,390      0/124
sealed s1 / s11         2,304/3,220  2,244/3,184     60/36
sos  s1 / s11             754/956      754/952         0/4
fixed s1 / s11            704/764      704/764         0/0
                            ------      ------      ------
                            30,272      29,962         310
```

**310 of 30,272 — 1.02 %**, and zero on four of the ten runs. The bot-side
half that landed at the seventy-first pass did its job. **What is left in the
23.8 % is the sweep, so the lever is a cheaper or a rarer probe, not a better
estimate** — which is a different entry from the one that was written here.

**Taken, and it was a path with no filter at all.** `pick_stack_response`
went from "is this an instant that counters spells" straight to the dry run.
`fixed`'s 704 failures were **700 of one cost, `{U}{U}`** — "azorius skies"
asking about its Counterspell against Islands that were not there — and
adding the `can_afford_in_state_with` the main hand sweep already runs took
that pool to **4**, worth **`fixed` -2.20 %** (`ab_wall` 6 blocks, 2,000
games x 4, CI -3.63 .. -0.78 %, 6 of 6 blocks faster) with `--bench`
byte-identical. `cube` -1.00 % with the CI straddling zero, and the census
says why: cube's failures are mostly elsewhere.

**The method note is the durable half.** The first hypothesis — that the
per-colour budget had been *widened* to `[u32::MAX; 5]` and so was not
filtering — was good and wrong, and a counter settled it in one run: the
widening fires **0.0 % of the time on `fixed`** and 3.9-11.9 % elsewhere,
with the spend-as-any-colour leg never firing at all. **Count the thing you
are about to blame.** Tightening the widening would have bought nothing on
the one pool that had the problem.

**THE MULTI-COLOUR HALF OF HALL'S CONDITION IS REFUTED — built, measured,
reverted (seventy-fourth pass).** The singleton case is the one that pays;
the subsets are not. `{U}{B}` off one Dimir dual and three Mountains passes
every singleton test and has no assignment, so the shape is real — and over
the bench workload it **rejected nothing at all**: `restore_payment_state`
2,606 either way, and the `[u32; 32]` per-mask fill plus the subset walk read
**`fixed` +0.105 %, `sos` +0.104 %, `cube` +0.107 %**. Two reasons, and the
second is the transferable one: the singleton test already catches the
colour failures these pools produce, and **the widenings that keep the budget
sound switch the whole thing off on exactly the boards that would violate a
subset** — a cube board with a Treasure, a filter land or a land-type rewrite
is `bounded = false`, and those are the boards with interesting mana. Do not
re-take it.

**Two ways in, and the second is the sound one.**
*Reject earlier in the bot.* **TAKEN, and the doc comment's warning was
right**: the first two versions of the tightened filter each rejected a cast
the engine could pay (Crystalline Crawler's counter-cost mana ability; a
Dryad of the Ilysian Grove land-type rewrite). Both were found by the oracle
this entry asked for, and neither by reading the code.
*The original sizing, kept for the record.* `bot::available_mana` is documented as
**deliberately optimistic** — it ignores the assignment problem, so a hand
card whose pips each have *some* producer passes the filter even when no
assignment covers them. Tightening it to a sound assignment test (Hall's
condition over ≤5 colours and ≤~10 sources is ~300 ops) removes the probe
entirely and is behaviour-preserving **only in the direction that rejects what
the engine would also have rejected** — an over-tight filter makes a legal
line permanently invisible, which the function's own doc comment calls out as
the failure it exists to prevent. Any version of this needs the engine's
answer as the oracle: run the tightened filter *alongside* the current one
over a bench run and assert it never rejects a cast `would_accept_on` accepts.
*Bail before tapping in the engine.* After `mana_source_table` is built,
`auto_tap_for_cost_inner` knows every colour it can reach; a still-needed
colour with no producer in the table means the colour loop will no-op on that
pip whatever else it does. Returning early there skips the other pips' taps.
**Sound only with two guards**, and both are outside `auto_tap`'s knowledge
today: `channel_life_for_mana` (the caller's post-auto-tap retry reads the
floated pool) and the `forced_only` human path (which deliberately keeps
eager-tapped mana floating for the retry). The generic half is *not* soundly
testable from the table at all — `ManaSourceInfo` carries colours, not
amounts, so "N sources for M generic" under-counts a source that makes two.

**(-50) THE NO-OP WRITE THROUGH A CoW HANDLE — the sixty-eighth pass's class,
and it is the one that pays.** A handle the code genuinely wrote is already
unshared by the write that moved it, so `make_mut` on it is a 29-Ir refcount
check. It is the **untouched** object — still shared with every probe clone —
whose redundant rewrite deep-copies at ~750-960 Ir. `restore_payment_state`'s
19,264 removed writes cost **964 Ir apiece**; its 15,062 survivors cost 29.

**So do not rank a CoW site by how often it is written** — that finds the
cheap half, which is what the bind-once sweep of (-42) was and why it has a
0.53 % ceiling. Rank it by **how often the value it writes is the value that
was already there**. The shapes to grep for:

- a rollback / restore that writes back a whole snapshot rather than the
  entries that moved (`restore_payment_state`, TAKEN);
- a reset chain: several unconditional writes on one handle in a row
  (the zone-change chain, TAKEN at the sixty-ninth pass — see below);
- an `Option::take()` on a CoW-held field. It needs `&mut`, so it unshares
  *before* it discovers the `None`; four of the zone-change chain's six
  writes were takes;
- a sweep that assigns a constant to a field on every object in a zone
  (`= false` in a cleanup or untap loop) rather than only where it differs;
- a save / set / restore pair around a call, where the set is usually a
  no-op — but **check the value first**, because
  `auto_tap_for_cost_inner`'s `wants_ui` pair looked exactly like this and
  is real (see the Log's refutation 2).

**THE CHAIN'S NEXT LINK IS TAKEN (seventy-second pass), AND THE ENTRY THAT
POINTED AT IT NAMED THE WRONG CALLEE.** After the zone-chain commit,
`on_left_battlefield`'s `make_mut` edge on cube went **19,384 calls /
510,460 Ir -> 19,384 / 5,665,537** — same calls, eleven times the cost. This
entry, and NEXT's item 1c after it, said those calls came from
`find_card_anywhere_mut(id)`. They did not: `cg_edges.py --callees
on_left_battlefield` puts that function in its **own row, 7,106 calls,
1.000x, un-inlined**, so its unshares were never on this edge. Gating it
therefore measured **`fixed` +0.083 %, `sos` +0.036 %, `cube` +0.053 %** at
the seventy-first pass and was reverted — a correct measurement of the wrong
hypothesis. (The finding that survives is still worth having: a `_mut`
lookup's cost is its *search*, and the card is already in a graveyard, so
both walks scan the battlefield and the stack first. **A `_mut` lookup is
not a (-50) site just because it usually writes nothing** — price the walk
before gating it.)

**What the edge actually was, and what is left.** 14,212 of the 19,384 calls
were `continuous_effects.iter_mut()` + `retain`, gated at the seventy-second
pass along with `temporary_control`'s `mem::take`,
`remove_effects_from_source` and `expire_end_of_turn_effects`
(`fixed` -0.135 %, `sos` -0.079 %, `cube` -0.121 %). **They were worth
351,900 Ir of the 5,665,537 — 25 Ir apiece**, because that list was already
unshared by the time the function ran. The **5,232 calls that remain are the
whole 5.31 M (0.20 % of cube)**, ~1,016 Ir each and every one a real
`CardData` deep copy: the CR 400.7 reset clearing `cast_from_hand` and its
five siblings on a card that a probe clone still shares. That write is not a
no-op — the flag really is set on any permanent that was cast — so (-50)'s
device does not reach it. What would: clearing the flags **while the placer
still owns the card**, before it is pushed into the destination zone, where
the callers have already unshared it (`card.tapped = false`,
`card.counters.clear()`). Seven call sites, not all of which hold the card by
value, and a missed path leaves a stale `cast_from_hand` — price the
correctness risk against 0.20 % before taking it.

**A (-50) SITE IS A CHAIN, NOT A LINE — the sixty-ninth pass's addition, and
it cost that pass a build to learn.** Gating the first unconditional write on
a handle hands the bill to the next one. Five of the zone-change chain's six
writes gated took `place_card_at_resolved_zone`'s `make_mut` edge from
8,514,910 Ir to 209,225 and moved the whole program by **-0.050 % of cube**,
because `send_to_graveyard`'s `counters.clear()` two frames down went
**2,244,366 -> 10,319,209** and absorbed it. Gating that too landed
-0.221 %. **If the program moves by a fraction of the edge you removed, the
copy has moved rather than gone** — the `make_mut` caller table names where
it went. Gate the whole chain, from the point the object is handed over to
its last touch, and ship it as one commit: an intermediate step can read as a
regression on a pool (that one is +0.022 % of `fixed` alone).

**The tell that finds a site, and it is one column of a table this file
already reads.** `cg_edges.py --callees <fn>` on a `make_mut` caller-table
row: **a callee count that is an exact multiple of the function's own call
count is a line that runs unconditionally**.
`place_card_at_resolved_zone` was 6,758 calls with 13,516 `make_mut` *and*
13,516 `graveyard_exile_redirects` — 2.000x on both, one a no-op write and
one a duplicated walk. A ragged ratio (`restore_payment_state` at 9.2x) is
the board width instead. Same tell as the sixty-third pass's pair loop, one
level down.

**Sites already gated, for the shape:** `reset_room_doors`, `reset_case`,
`revert_copy_on_leave`, and now `turn_face_up` / `revert_flip` /
`revert_transform` / `revert_prototype` / `undo_licid_aura`,
`send_to_graveyard`'s two `clear()`s and `place_card_at_resolved_zone`'s
`soulbond_partner`, and at the seventy-second pass `on_left_battlefield`'s
`temporary_control` + `continuous_effects` pair, `remove_effects_from_source`
and `expire_end_of_turn_effects`.

**THE `ColdState` HALF OF THIS VEIN IS WORKED OUT (seventy-second pass).**
A cold write unshares ~89 collections at **4,689 Ir**, so it ranked above
everything else per call; after the two commits above and the
`blocked_attackers` / `blocks_declared_this_turn` field move, cold unshares
are **3,020 / 13.32 M / 0.51 % of cube** and every remaining caller
(`note_creature_death`, `remove_from_battlefield_to_graveyard_raw`,
`finish_cleanup`, `run_effect`, `discard_card`) writes a value that changed.
Do not go looking for another no-op cold write; the table to re-read is
`cg_edges.py --callers "crabomination::game::GameState as
core::ops::deref::DerefMut"`, and it is short.

**Unswept and named by the `make_mut` caller table on
`cube` at the sixty-ninth base**, by Ir/call — `cast_spell_with_convoke`
(119,138 calls / 26.5 M / 223 Ir), `activate_ability_inner` (50,594 / 23.0 M
/ 454), `declare_attackers_banded` (63,090 / 17.9 M / 284), `declare_blockers`
(14,940 / 5.79 M / 388), `try_pay_after_snapshot_mode` (11,456 / 3.59 M /
314), `PlayerData::draw_top` (7,420 / 2.01 M / 271). The first of those is
NEXT's top item and is a different mechanism (a card taken out of hand ahead
of fifty gates), not a no-op write.

**And check which impl block a method lives in before applying (-14) to it.**
(-14) says an internal `if self.field` guard is dead behind a CoW handle
because `DerefMut` runs first. That is true for a method on the *inner* type
and false for one on the handle: `CardInstance::clear_summoning_sickness` is
an inherent `impl CardInstance` method and guards correctly. Eleven
call-site guards on it moved `do_untap`'s `make_mut` edge by **zero calls**.

**(-49) `wants_ui` IS A `PlayerData` FIELD AND THE SCRIPTED-TAP DEVICE
TOGGLES IT TWICE PER ANY-COLOUR TAP.** `auto_tap_for_cost_inner` swaps in a
`ScriptedDecider` for an `AnyOneColor` source and forces synchronous
resolution by writing `players[player].wants_ui = false`, then writing it
back. `wants_ui` is **true** in every measured workload —
`recommend.rs`'s match simulator sets it on both seats and so does the
`selfplay` actor path — so both writes are real `PlayerData` unshares:
**19,380 `make_mut` calls / 1,915,356 Ir on `cube`, 0.07 %**, two per
scripted tap over 9,690 of them.

The fix is not a guard (measured, byte-identical, see the Log) — it is to
stop expressing "resolve this inline" as a write to a CoW-held player flag.
A plain `GameState` field (`self.decider` is already one, so the struct has
non-CoW fields) read alongside `wants_ui` wherever a decision decides whether
to suspend would make the toggle free. Small, and it touches decision
plumbing, so it wants the decision-plumbing audit's eye rather than a perf
pass's.

**(-48) CLOSED at the sixty-fourth pass: mimalloc is 5.99 % faster and costs
9.7 MiB of RSS per process. The default is right and the memory is bought.**
`scripts/ab_wall.py`, eight ABBA blocks, `release-fast` both sides at the
sixty-fourth tip, `--a gang --b gang --games 2000 --decks sos --seed 11
--threads 4`, A = `--no-default-features` (system), B = default (mimalloc):

```text
              mean B/A   95 % CI            blocks B faster
A/B           0.9401     -7.04 .. -4.95 %   8/8
null control  1.0020     -0.79 .. +1.18 %   4/8      FLAT
```

```text
build (release-fast)   allocator   peak_rss_mib   games_per_s (--bench, one run)
--no-default-features  system      17.5           156.22
default                mimalloc    27.2           164.63
```

**Six percent is larger than any single perf commit in the last ten passes,
and it costs 9.7 MiB a process.** The entry framed RSS as the thing that caps
actors per box; at 27 MiB an actor that is not the constraint on any box that
can run four of them, so the trade is not close. Keep mimalloc.

**And the null resolved +/-0.99 % here, not the +/-2 % this file records.**
That figure was calibrated on a 2.10 GHz box; this one is a 2.80 GHz Xeon
with `host_calib_ms` 50-57 and a within-binary spread of 8-9 %. **Run the null
on the box you are on** — the resolution is a property of the host and the
minute, not of the harness.

**The RSS row also reproduces across three passes and two builds**: the
sixtieth pass's 17.7 MiB and the sixty-third's 17.6 are this run's 17.5, all
`--no-default-features`; the shipped number is the other column. Nothing got
heavier.

**Replicated independently the same hour, on a different container, at
`0c6fb73e`** — the two sessions ran this entry concurrently without knowing
it, which is the one useful thing to come of the duplication: the effect now
has two boxes under it and a spread.

```text
                     mean B/A (system vs mimalloc)   95 % CI          blocks
sixty-fourth tip     +5.99 %                         +4.95 .. +7.04   8/8
0c6fb73e             +7.98 %                         +7.05 .. +8.91   8/8
null (0c6fb73e)      +0.02 %                         -1.01 .. +1.05   4/8  FLAT
peak_rss_mib         system 17.4/17.8/17.5   mimalloc 28.9/27.0/26.8
```

**The two CIs meet at ~7.0 % and do not overlap below it, so the honest
statement is "6-8 %, host-dependent", not a single number** — the same
caution this file already applies to `games_per_s` and RSS, now with the
allocator delta itself inside it. Both nulls are flat and both resolve about
+/-1 %, which is the second reading of "run the null on the box you are on".
`decisions 196,220` byte-identical on all six binaries. Direction, sign and
the RSS gap are the same on both boxes; **nothing here re-opens the entry.**

**(-47) DONE at the sixty-third pass — `d9f459de` + `fa3bf671`, and it read
5x its sizing.** The entry costed the attacker-resolution hoist alone at
~6.6 M / ~0.24 % of cube. Measured: **-1.289 % of cube**, -0.579 % of fixed,
-0.446 % of sos. The sizing missed two things and both are the transferable
part — see the Log's sixty-third pass.

1. The pair loop above the legality check was paying per pair for *six*
   attacker facts (Rampage, first strike, indestructible, trample, must-be-
   blocked, min-blockers) and two blocker facts, each a battlefield `find`
   plus a keyword walk. `pick_blocks_inner` self went 24,906,488 -> 7,583,714.
2. Two of the twelve gates inside `blocker_can_block_attacker` never name an
   attacker at all (`blocker_side_gates_allow_block`, 193 Ir a call, and the
   computed creature-ness test), so the hoistable half was bigger than the
   resolutions.

**The rule: in a loop over pairs, ask of every term which side of the pair
it belongs to.** The tell is a callee count that is a *multiple* of the pair
count — `computed_permanent` sat at exactly 2x `blocker_can_block_attacker`.

**What is left of this entry, and it is small.** `pick_attacks`'s
"unblockable by the current board" check is the same shape over the ~4,900
pair checks that do not come from `pick_blocks_inner`; hoisting there means
resolving every opponent blocker eagerly on boards where the branch is never
reached, so it needs measuring rather than assuming. The two
`battlefield_find`s per composed call are still **(-38)**'s.

**Two things this entry proposed on its first writing and which are
REFUTED — recorded so nobody re-proposes them.**

1. **"Make the `layer_freeze` memo a map instead of a linear scan."** No:
   `LayerFreezeState::perms`' own doc already says why it is a `Vec` —
   *"Short — one entry per permanent actually asked about — so a linear scan
   beats hashing."* And the lock is not the cost either: `Mutex::lock` under
   `computed_permanent` is **34,268 calls / 890,968 Ir = 26 Ir each** on
   cube. Read the struct comment before optimising the structure.
2. **"Wrap the bot's scoring loop in `with_frozen_layers`."** Already done.
   `eval_material` and `eval_material_summon_sick_blind` both open a scope
   around `eval_material_inner`, and the code says so at `bot.rs:10132`.
   `permanent_value_with`'s 1,404 Ir/call is not an unfrozen recompute — it
   is the **first** gather of each scope, amortised over ~5 permanents,
   6,248 scopes deep, because the sims score a *cloned* state per candidate
   and a clone cannot inherit the memo. That is **(-13)**'s cost, not a
   missing freeze.

**What the caller table of `computed_permanent` on `cube` does say, and it
updates (-30).** (-30) was read on `fixed` at the forty-eighth tip and named
three engine callers. On `cube` at the sixty-second the table is 267,098
calls and splits cleanly by Ir/call — **~100-233 is a memo hit inside a
scope, ~800-3,600 is a scope's first gather**:

```text
callers of computed_permanent, --decks cube, sixty-second tip
  calls        Ir       Ir/call  caller
  56,748   13,238,631      233   blocker_can_block_attacker      <- hits, this entry
  41,688    4,189,463      100   damage_prevented_by_protection  <- hits
  34,876   48,980,416    1,404   bot::permanent_value_with       <- scope-first gathers
  30,406   24,863,248      818   bot::attacker_damage_value
  17,022   30,231,407    1,776   FnMut::call_mut (a bot closure)
  12,864   46,938,249    3,649   resolve_combat                  <- (-30), (-25)
  12,316   19,420,255    1,577   Map::fold
  10,626   20,324,276    1,913   check_target_legality_with_source <- (-30)
   7,296   20,113,269    2,757   with_frozen_layers
```

**The seven expensive rows are 210.9 M, 7.6 % of cube**, and the honest
reading of that number is that it is **one gather per freeze scope times the
number of scopes**, not waste inside any single caller. So the lever is not
"freeze more" — it is "open fewer scopes", i.e. fewer cloned candidate
states, which is (-13) and was costed and refused. Two rows here are new to
the record and neither changes that: `permanent_value_with` and
`attacker_damage_value` are bot scoring, they are already frozen, and cube
carries more of them than `fixed` because a grant-heavy board makes each
gather cost more.

**(-45) THE COST OF ASKING — the sixty-first pass's class, and the table it
came out of still has rows.** Every one of that
pass's wins was a presence question whose *asking* cost the same whether the
answer was yes or no, on a board where it is always no: a SipHash of ~84
small integers for a fingerprint (both sessions found that one
independently), a `filter`/`flat_map`/`filter`/`map` stack collected into an
always-empty `Vec`, a `flat_map` over two always-empty command zones, and a
battlefield `filter` for a card no bench deck contains. **None of them is a
hot function**; together they are ~1.5 % of `sos`.

**The device is one command**, and it is why this entry exists rather than a
list of sites: `python3 scripts/cg_edges.py cg.out --callers
SpecFromIterNested`, **ranked by calls, not by Ir**, then ask of each row
"can this collect be non-empty on the pools the actors play?". The
sixty-first tip's table on `--decks sos` (1,580,084,804), with the two rows
that pass already took removed:

```text
  calls        Ir        caller
  90,170    6,537,071    layers::compute_permanent_pass       <- TAKEN, 64th pass:
                                                                 90,170 -> 14,784 calls,
                                                                 `sos` -0.354 %
  37,420    1,336,242    resolve_effect                       <- 36 Ir each
  21,912    7,323,096    declare_attackers_banded
  18,750      944,725    fire_delayed_event_watchers          <- 50 Ir each
  13,576   15,884,891    check_state_based_actions            <- 1.02 per sweep now,
                                                                 was 4.02
  11,334    2,445,441    finalize_cast
  11,004    1,543,826    fire_combat_damage_to_player_triggers
  10,836      769,032    blockers_of
  10,328   20,436,507    bot::pick_attacks_inner
   9,374   13,233,290    compute_permanents
 149,490         —      103 more rows (38.07 % of the calls)
```

**The top row is PAID at the sixty-fourth pass and it was the cheapest kind:
the chain was empty on 83.6 % of the passes.** `affected_includes_gated`, the
filter body inside that `from_iter`, runs 29,436 times over 89,154 layer
passes on `sos` — a third of an effect apiece — and only 1,284 of the 90,170
collects allocated. Gating the collect on "is there anything to filter"
read `fixed` **-0.319 %**, `sos` **-0.354 %**, `cube` **-0.131 %**. **Cube
moves least because a cube board carries statics**, so its gathered list is
non-empty more often; the gate is worth what the *empty* fraction is worth,
which is a pool question. That is the sizing rule for the rest of the table.

**Four more rows PAID at the sixty-fourth pass, and every one of them was
followed by an `is_empty()` test that could have been asked first.**
`resolve_effect`'s two (the resolution's target list, and the Quina rider
walk over `last_created_tokens`), `fire_delayed_event_watchers`' two (the
batch's deaths and its attacker declarations), and `blockers_of`. Together
`fixed` **-0.069 %**, `cube` **-0.079 %**, `sos` **-0.098 %**; 45,430 collects
removed. `resolve_effect` **37,420 calls -> 4,930**,
`fire_delayed_event_watchers` **18,638 -> 10,170**, `blockers_of` **10,836 ->
6,364`. **The tell to grep for is a `collect()` whose very next line is an
`is_empty()` on what it just built.**

**The row this entry calls its largest is a call count, and skipping the
build is REFUTED (sixty-eighth pass).** `compute_permanent_pass`'s collect is
140,238 calls on `cube`, but a gathered effect list is **about two effects
long**, so the filtered result is 0-1 elements and the `collect` does not
allocate — it is `from_iter` *call* overhead. Sorting at the gather so the
pass can walk a `filter` instead read `fixed` **+0.173 %**, `sos`
**+0.208 %** (`cube` -0.205 %) and reverted: the `is_layer_sorted` guard that
keeps the skip sound for `apply_layers_one`'s hand-built callers costs more
than the `Vec`. **Before removing an allocation, check there is one** — a
`from_iter` row appears in an allocation table because it is *reached from*
one, not because every call allocates. The number is in the code at the
collect.

**Read the two columns against each other.** A row with many calls and few
Ir apiece (`compute_permanent_pass`, `resolve_effect`,
`fire_delayed_event_watchers`) is a `Vec` being built to be thrown away —
the sixty-first pass's shape, and the cheapest kind to fix, because the
replacement is a loop. A row with few calls and a lot of Ir apiece
(`pick_attacks_inner`, `compute_permanents`) is the *iterator body* being
re-reported through `from_iter`, not collect overhead — **(-17)'s standing
caveat**, and those rows are not this entry's.

**And the third kind, which is the one to check before writing any code: a
collect that exists because a `&mut self` follows it.** Phase 1.6 of
`fire_combat_damage_triggers` was removable precisely because its loop body
touched nothing but a local. `declare_attackers_banded`'s two big ones
(`listeners`, `you_attack` — both whole-battlefield `flat_map`s over
`triggered_abilities`) push onto `self.stack` in the drain, so the buffer is
load-bearing and only a presence gate could help there — and the gate is the
walk. **The tell is one line: does the drain touch `self`?** Check it first;
it takes seconds and it is the difference between a one-line fix and a
refactor that cannot pay.

**The one refutation this entry ships with, because it is the boundary:**
folding a per-sweep presence question into `sba_board_scan` — the walk that
already visits every permanent — read **+0.295 % / +0.255 %** for
`card_type_change_unscoped`'s battlefield leg. The standalone
`any(card_can_change_card_types)` short-circuits per card; a scan bit cannot.
**A presence bit belongs in a shared scan only when the question has no early
exit of its own.** Third refutation of the (-6) fusion device inside
`creature_death_possible` alone (+0.55 %, +1.24 %, +0.29 %).

**The sibling table is RUN, at the sixty-seventh pass, and it paid twice.**
`--callers` on `Vec::clone` and `grow_one`, **ranked by Ir/call rather than
by calls**, named `continue_spell_resolution` (1,601 Ir/call, a whole
`Effect` tree) and `finalize_cast` (677) — rows 4 and 5 by *calls*, which is
why ranking by calls alone had never reached them. Both are taken; `fixed`
-0.360 %, `sos` -0.513 % end to end. **Rank an allocation table by calls to
find a `Vec` built to be thrown away; rank it by Ir/call to find a tree being
deep-copied.**

**What those two tables still hold** (`sos`, sixty-seventh tip, 1,501,691,374):

```text
callers of grow_one — 249,744 calls
  34,670    5,654,416   Vec::push_mut                        (generic)
  30,758    3,951,887   gather_continuous_effects_inner      <- `sa_cards`; see below
  22,604    2,893,995   check_state_based_actions            <- unread
  20,664    2,479,680   advance_step                         <- unread
  15,754    4,764,983   finalize_cast                        <- 302 Ir/call, (-28)'s
  11,466    2,017,154   declare_blockers                     <- unread
   9,802    1,269,800   granted_abilities_of
   8,196    2,093,114   computed_permanent

callers of __memcpy — 1,159,403 calls
 103,694    6,245,840   GameState::clone                     <- (-13)
  90,004    7,833,802   finalize_cast                        <- (-28)
  65,285    1,029,267   String as fmt::Write::write_str
  60,720    1,821,600   computed_permanent
```

**`gather_continuous_effects_inner`'s row is NOT a reserve candidate** — the
buffer it grows is `sa_cards`, which is *empty* on a vanilla board, and a
blanket `+ battlefield.len()` headroom is the shape the fifty-eighth pass
already measured at **+1.54 %** (item I: "the reserve has to be where the
pushes are, not where the clone is").

**The other three were read at the sixty-seventh tip, and the number that
sorts them is grows *per call*, not grows.** A row at ~1 grow a call is one
allocation a `Vec` genuinely needs; a row at 4 is a buffer being filled an
element at a time.

- `advance_step` — **READ, and it is not a candidate.** 22,162 grows over
  23,660 calls is **0.94 a call**, which is the single `events.push(
  GameEvent::StepChanged(next))` on a list the caller hands in empty and
  gets back. The allocation holds the event that is returned; there is
  nothing to reserve and nothing to skip.
- `check_state_based_actions` — 22,604 over 13,274 calls, **1.7 a call**,
  and **its named collects are already scan-gated** (`scan.flip_predicate`,
  `scan.sacrifice_when`, `scan.state_trigger` — (-45)'s treatment landed
  here in an earlier pass). What is left is spread across `events` and the
  inner helpers; localizing it needs `cg_contexts.py` over
  `--separate-callers`, not a read of the source.
- `declare_blockers` — looked like the best of the three (11,466 grows over
  **2,732 calls = 4.2 a call**, plus `RawTable::reserve_rehash` 5,034 = 1.8
  a call) and is **REFUTED**, twice, at the sixty-seventh pass.

  Reserving the `ids` list in both combat gates (`declare_blockers`'
  `assignments.len() * 2 + attacking.len() + block_map.len()`,
  `declare_attackers`' `attacks.len() + attacking.len()`) **plus** the
  `batch_blocks` map read `fixed` -0.027 %, **`sos` +0.046 %**, `cube`
  -0.063 % — net zero with a pool going the wrong way. Isolating the map
  (`HashMap::with_capacity_and_hasher(assignments.len(), ..)`, the row with
  the clean 1.8-rehash mechanism) read `fixed` **-0.005 %**, `sos`
  **-0.002 %**, `cube` **-0.014 %**: free, and worth nothing. By
  subtraction the two `ids` reserves are the regression.

  **The rule, and it corrects the sizing device this entry ships with:
  grows-per-call ranks a row, but the length the buffer *reaches* decides
  whether a reserve pays.** 4.2 grows a call over a list that ends at a
  handful of ids is `1 -> 4 -> 8`, i.e. two reallocs of 32 and 64 bytes; a
  right-sized `with_capacity` replaces two small reallocs with one larger
  allocation and can cost more than it saves. A reserve pays when the buffer
  is **long**, not when it is grown **often**. Both reverted.

**(-46) `name_index()` BUILDS 22,568 `CardDefinition`s TO READ 22,568
STRINGS — 104,687,400 Ir. RANKED LOW ON PURPOSE; READ THE SIZING BEFORE
TAKING IT.** `card_registry::name_index()`'s `OnceLock` calls every catalog
factory, keeps `def.name` (and the MDFC back face's), and drops the
definition. That is the whole cost: one edge, 104.7 M Ir, **6.8 % of a
six-game `--decks sos` run** and 0 % of `cube` and `fixed` (call tables in
**How to measure**).

**It is one-time per process, and that is the entry.** A `selfplay_train`
actor plays thousands of games on one process, so 104.7 M is ~0.001 % of it;
a test binary that resolves any name pays it once, tens of milliseconds
against a 27 s suite. The 6.8 % is a *measurement* artefact — it inflates every short `sos`
callgrind total — not a throughput one. **Do not take this ahead of anything
in the game loop.**

The fix, if a pass ever wants it cheaply: the names are known at build time,
so `crabomination_catalog` can emit a `&[(&'static str, CardFactory)]`
alongside `all_known_factories` and the index becomes a map build over a
static slice. That is codegen work in the catalog crate for a startup
number, which is why it sits here rather than getting done. A narrower
half-measure — have `apply_pending_effect_answer` resolve a `NameCard`
answer against the same pool the suggestions were ranked from before falling
back to the registry (the sixty-first pass did exactly this for the
*suggestion* side) — would keep bot self-play off the index entirely, but it
only moves *when* the build happens for anything that names an off-board
card, so measure that it removes the edge rather than assuming it.

**(-44) `__memcpy` IS 5.55 % OF `sos` AND THE ALLOCATOR FAMILY 12.7 %, AND
NEITHER HAS EVER HAD A CALLER TABLE READ BY Ir/CALL.** The sixtieth pass took
`__memcpy` from 7.80 % to 5.55 % with one commit, and the way it found the row
is the entry: the table is forty rows of a hundred-odd Ir each, and one row —
`CardInstance::new`, 3,452 calls at **8,242 Ir apiece** — is an outlier in the
*ratio*, not in the calls or the total. 8,242 Ir is `size_of::<CardDefinition>
()` = 8,232 bytes moving. Do the same read on `_int_free` (4.25 %) and
`malloc` + `_int_malloc` (5.78 %): an allocation whose Ir/call is far above
the family's mean is an allocation of something big, and that is a shape, not
a diffuse cost.

**The allocator half of this entry was read the same way and there is no
outlier — record kept so nobody redoes it.** 1,072,958 `__rust_alloc` calls a
six-game `sos` run; by Ir/call the table is flat: `finish_grow` 241,644 at 86,
`Arc::clone_from_ref_in` 140,010 at 123, `Vec::clone` 39,750 at **174** (the
highest, and it is a moderate `Vec`, not a kilobyte), `Box::clone` 38,129 at
79, `finalize_cast` 13,868 at 119, `mana::cost` 28,024 at 58. The engine
functions that allocate *directly* and look wrong — `mana::cost` 28,024,
`SelectionRequirement::and` 16,992, `::or` 8,464 — are **catalog
constructors**, i.e. definition-build time, which `card_arc` and `card_brief`
now amortize to once per factory per thread; the only game-loop caller of
`mana::cost` is `declare_blockers` at 1,258 calls / 191,426 Ir (0.012 %).
`finish_grow`'s 22.5 % share is (-28)'s `Vec::clone` capacity story and the
`GrowVec` newtype that measured **+0.050 %**. **So: the allocator is a real
12.7 % and it is genuinely diffuse.**

**The *dealloc* side is now read too, at the sixty-second tip, and it is the
same answer — so this half of the entry is CLOSED, both sides.** The alloc
table above is `__rust_alloc`'s; `_int_free` (4.26 %) and `free` (2.71 %)
are reached through `__rust_dealloc`, which is a different table and had
never been read. It is 1,071,319 calls and flat to within a factor of 1.3
by Ir/call — there is no `CardInstance::new`-shaped outlier in it:

```text
callers of __rust_dealloc, --decks sos, sixty-second tip
  calls        Ir        Ir/call  caller
 243,857   27,078,427      111    Arc::drop_slow
  89,930   11,310,723      126    Arc::drop_slow'2
  87,118    9,574,823      110    drop_in_place<GameState>
  70,374    7,663,336      109    drop_in_place<CardDefinition>
  42,176    4,169,983       99    check_state_based_actions
  39,191    3,859,235       98    drop_in_place<Box<SelectionRequirement>>
  30,302    2,919,322       96    gather_continuous_effects_inner
  27,830    3,294,958      118    drop_in_place<Result<Vec<GameEvent>,_>>
  26,748    2,755,679      103    auto_tap_for_cost_inner
  25,405    3,168,277      125    IntoIter::drop
 322,448         —          —     440 more rows (30.10 %)
```

**95 to 126 Ir a call, top to bottom.** The sixtieth pass's device works by
finding a ratio outlier; there isn't one here, on either side. **Do not
spend a build re-reading `_int_free` or `malloc` by Ir/call** — a
handoff-note in TODO was still pointing at it, and this is the record that
retires it. The allocator's 12.75 % is a million small frees, and the two
rows with any shape to them (`Arc::drop_slow` at 333,787 calls between them,
`drop_in_place<CardDefinition>` at 70,374) are the *paying* side of the CoW
unshares that **(-43)** already prices at 80.9 M — counted there, not here.

The sos table at the sixtieth tip, top of `__memcpy`'s callers:

| caller | calls | Ir | Ir/call |
|---|---|---|---|
| `GameState::clone` | 103,330 | 6,222,884 | 60 |
| `finalize_cast` | 92,590 | 7,832,712 | 85 |
| `String::write_str` | 65,285 | 1,029,267 | 16 |
| `computed_permanent` | 60,556 | 1,816,680 | 30 |
| `Vec::clone` | 34,547 | 1,875,090 | 54 |

**Nothing left in that table is an outlier**, which is what "diffuse" should
mean and usually does not. `finalize_cast`'s 24.7 memcpys a call are
(-28)'s and that entry is closed to everything but a `CardTypeSet` bitset.

**Both halves under `CardInstance::new` are PAID, and together they came in
above the ~0.6 % this entry sized them at.** `definition_matches_requirement`'s
deep clone went at the sixty-second pass (`71cee718`, and the find was one
level up — the caller was looking a name back up in the global index to test a
definition it was already holding). `mint_token_onto_battlefield` went at the
sixty-fourth: **370 calls / 6,649,391 Ir -> 296,405, -95.5 %**, `sos`
**-0.605 %**, `cube` **-0.747 %**.

**This entry's own reason for not starting them was wrong and the correction
generalises.** It said a token memo needs `TokenDefinition: Hash`, which the
type does not derive. It does not: a capped `Vec` scanned with the derived
`Eq` is enough when the distinct-key count is small (four shapes over six
games), and `name: String` being field 0 makes a miss one compare. **Price the
linear scan before writing a `Hash` impl.** The larger half was not the memo
at all — it was that both `CreateToken` loops rebuilt the same 8,232-byte
definition *per token in the batch*, which is -0.53 % of `sos` on its own.

**RE-READ AT THE SIXTY-THIRD TIP AND IT IS FLAT — do not spend a callgrind
round re-collecting this table.** `make_mut` on `sos` is **440,300** calls
against the 439,300 below, and every row is within a percent of its
fifty-eighth-tip value (`cast_spell_with_convoke` 52,410 / 12.06 M,
`activate_ability_inner` 30,404 / 13.97 M, `declare_attackers_banded` 28,722
/ 9.71 M, `do_untap` 41,198 / 4.59 M). The `cube` column, which this entry
never had, is 858,130 calls with the same shape and two rows that are
larger there than on `sos`: `restore_payment_state` **34,326 / 18,996,709 =
553 Ir/call** and `place_card_at_resolved_zone` **13,516 / 8,496,828 = 629**
— both in the clone-shaped half, both above `activate_ability_inner`'s
Ir/call.

**`restore_payment_state` is TAKEN at the sixty-eighth pass and it was
almost all waste** — 34,326 calls / 19.0 M -> 15,062 / 436 k, `cube`
-0.866 %; see (-50) for the class and the ranking rule that comes out of it.
**`place_card_at_resolved_zone` is still unread**, and the shape to expect
there is the opposite one: 13,516 `make_mut` over ~13,500 calls is one write
per card placed, and a card that changes zone genuinely moves, so read it
before costing it.

**The clone-shaped rows on `cube` after that commit, by real deep copies
(`--separate-callers=2` on `clone_from_ref_in`, 159,018 total), which is the
table to start from rather than the `make_mut` one:**

```text
  32,782  make_mut <- activate_ability_inner        (50,402 calls: 0.65 clones each)
  30,670  make_mut <- cast_spell_with_convoke       ( 9,600 calls: 3.2 each)
  19,052  make_mut <- declare_attackers_banded      ( 7,578 calls: 2.5 each)
   7,382  make_mut <- declare_blockers
   6,658  make_mut <- place_card_at_resolved_zone
   5,632  make_mut <- do_untap
   2,402  GameState::deref_mut <- declare_blockers   <- the ColdState clone, (-14)
```

`cast_spell_with_convoke`'s **3.2 deep copies per cast attempt** is the
largest unread number in it, and the sixty-eighth pass read it far enough to
say what it is *not*. The function removes the card from hand before its ~50
validation gates and pushes it back on each failure path — but on `cube`
**7,790 non-recursive attempts reach `finalize_cast` 4,720 times, a 39 %
failure rate**, against the 94 % completion (-24) measured on `fixed` at the
forty-fifth tip. So the removal is genuine work on the 61 % that finish, and
the waste is the 39 % that do not: **the lever is the bot's affordability
filter, not the cast's ordering.** `try_pay_after_snapshot_mode` fails 3,712
times on `cube`, which is the same population. See (-41) / (-34). **The paying side is
`Arc::clone_from_ref_in`: 85,650 calls / 64,030,880 Ir on `sos` (4.20 %) and
168,808 / 128,187,067 on `cube` (4.69 %), i.e. 19.4 % of unshares actually
deep-copy, at ~747 Ir apiece.** That is the size of the prize and it is the
largest unclaimed number in the profile.

**(-43) THE CoW-HANDLE FAMILY, READ FROM THE TOP AT THE FIFTY-EIGHTH TIP.
`make_mut` on `sos` is 439,300 calls after four commits took it down from
582,552 (-24.6 %), and the table below is where the rest of it is.** This
is (-42) generalised: `Player`, `CardInstance` and `CowBox` are all CoW
handles whose `DerefMut` is `Arc::make_mut`, so a run of writes through one
pays an unshare per write.

```text
callers of make_mut, --decks sos, at the fifty-eighth tip
  calls        Ir        Ir/call  caller
  52,110   12,005,430      230    cast_spell_with_convoke
  41,200    4,572,433      111    do_untap            (paid; residual is singletons)
  37,532    3,959,023      105    resolve_top_of_stack_inner
  30,208   13,848,574      458    activate_ability_inner
  28,722    9,731,410      339    declare_attackers_banded
  26,474    1,700,225       64    finalize_cast       (paid, 46,992 -> 26,474)
  26,538      875,532       33    resolve_combat
   8,494      224,380       26    on_left_battlefield (paid, 24,352 -> 8,494)
 222,990         —          —     62 more rows (50.8 %)
```

**Read the Ir/call column before picking a row, because it splits the family
in two and only one half is the (-42) device.** A caller at ~30 Ir/call is
paying the refcount check and nothing else — that is a run of writes on an
already-unshared handle, and `let x = &mut *…` collapses it, which is what
the three commits did (`advance_step` 51,142 -> 9,198, `do_untap` 80,148 ->
41,200, `deal_combat_damage_to_target` 21,012 -> 10,008). A caller at
230-458 Ir/call is *actually cloning*: the `Arc` is genuinely shared at that
point, and binding once saves nothing because the second write through the
same binding was already cheap. `activate_ability_inner` (13.85 M Ir, the
largest in the table) and `cast_spell_with_convoke` (12.01 M) are that
second kind, and **the question for them is not "bind once", it is "who else
holds this `Arc`, and does the write have to happen while they do"** — a
snapshot, a `clone()` kept across the call, or a handle parked in a local.
Size the prize off the Ir column, not the calls column: the two clone-shaped
rows are 25.9 M Ir together, 1.6 % of `sos`, against 8.5 M for the whole
bind-once half.

**STOP GRINDING THE BIND-ONCE HALF — it has a measured ceiling and four
commits already took most of it.** A `profiling-lines` build plus
`cg_sites.py … deref_mut` prices the *entire* inlined `deref_mut` family at
**6,551,438 Ir, 0.53 % of the run**, across 111 sites, and the largest single
one is 1,193,304 Ir (0.10 %). That is the whole remaining prize for binding
handles, against the 80.9 M (5.04 %) of actual clones below. The four
commits of this pass took `make_mut` 582,552 -> 439,300 and there is no
site left that is worth a pass of its own:

```text
inlined deref_mut call sites, --decks sos, after the four commits
  1,193,304 (0.10%)  stack.rs:5777    check_state_based_actions (token sweep)
    623,910 (0.05%)  layers.rs:487    compute_permanent_pass
    548,064 (0.04%)  mod.rs:2153      GameState::deref_mut
    527,502 (0.04%)  card.rs:6513     CardInstance::deref_mut
    389,528 (0.03%)  mod.rs:3812      Vec::index_mut
    356,520 (0.03%)  layers.rs:581    compute_permanent_pass
    267,360 (0.02%)  mod.rs:17738     dispatch_triggers_for_events
    267,360 (0.02%)  mod.rs:17872     dispatch_triggers_for_events
    800,194 (0.06%)  86 more sites
```

**`resolve_top_of_stack_inner` and `resolve_combat` looked like the two rows
left and are not takeable**: both are short functions whose `make_mut` count
belongs to *inlined callees*, so there is no run of writes to bind — that is
why they need `cg_sites.py` rather than a read of the function. Same for
`check_state_based_actions`: its eight seat writes are all singletons and all
already gated, and its `Player::deref_mut` line (`player.rs:1053`,
1,361,936 Ir) has nothing to collapse.

**The by-line read of `check_state_based_actions` is a separate lead and a
bigger one — it is 47,133,588 Ir, 3.8 % of the run**, and diffuse: its top
row is a dependency's `macros.rs:332` at 7,724,320 (0.62 %), then `cmp.rs:412`
at 2,770,838. Nothing in it is a CoW handle. Sized here, not chased.

**Then the family was read one level further down, and the real number is
much larger than the caller table suggests. `make_mut` genuinely *clones*
85,322 times out of 475,676 (17.9 %), at 748 Ir each.** Counting the
specialised `deref_mut`s alongside the generic row, the CoW bodies are
deep-copied **91,478 times for 80,868,330 Ir — 5.04 % of `sos`**:

```text
callers of Arc::clone_from_ref_in, --decks sos
  85,322   63,860,853   Arc::make_mut (the generic, inlined row)
   3,102   14,285,495   GameState::deref_mut          <- 4,605 Ir a clone
   1,692    1,532,689   PlayerData::deref_mut
     842      767,056   CardInstance::deref_mut
     460      367,273   CowBox::deref_mut
      60       54,964   Player::deref_mut
```

Inside one clone: **1.64 allocations** (140,010 `__rust_alloc` / 17,044,619
Ir, the single largest line item), ~2 `Vec` clones (168,822 / 10,123,493),
half a `RawTable` (43,362 / 2,923,028).

**And the cause is not in any of those callers — it is the bot's probe
machinery, one frame up.** `GameState::clone` runs **19,086 times, 24.8 M Ir
self (1.55 %)**, essentially all of it speculative: `accept_on` 6,660,
`perform_action`'s failure checkpoint 5,606, `ProbeCell::try_init` 3,376,
`sim_start_state` 1,226,
`evaluate_action_sequence` 896, `main_phase_action_with` 894. Each such
clone is cheap by design — it bumps refcounts — and then **every subsequent
write on either side pays the deferred deep copy.** So the probe machinery's
true price is the 24.8 M of clones *plus* the 80.9 M they force:
**105.7 M Ir, 6.6 % of `sos`**.

**The value of this entry is the 80.9 M deferred half, not the sub-candidate
list — because every item on that list turns out to be an existing entry
that was already answered.** That is the finding: the CoW clone cost has
been costed three times from the *causing* side ((-13) on the checkpoint,
(-41) on `available_mana`, `ProbeCell` on the probe templates) and never
once from the *paying* side, which is 3.2x larger than the clones that cause
it and does not appear in any caller table of `GameState::clone`. Read it
that way before proposing anything:

1. **`perform_action`'s checkpoint (5,606 clones) — this is (-13), it is
   already costed, and the answer was no. Do not re-open it without a new
   argument.** Pass 43's (A) took the provable half (-2.842 %): the
   round-closing `PassPriority` skips the checkpoint, and
   `GameState::clone` from `perform_action` went 18,208 -> 8,266. **The
   remainder is explicitly the half where the checkpoint earns its keep**,
   and `scripts/fallibility_closure.py` is the tool that decides it —
   `play_land` reaches 6 `Result` functions of which 2 raise, but
   `submit_decision` reaches **137 of which 70 raise**, which is exactly why
   the rest is not proven arm by arm. TODO's "Engine — Rollback / Undo
   system" has the other half of the argument: the checkpoint is what
   structurally kills the audit-P0 partial-mutation family (Squad/Casualty
   under-pay, `declare_attackers` mid-loop corruption, back-face land
   corruption, madness mana loss), it is pinned by
   `cow::tests::rejected_action_restores_state_exactly`, and **any narrowing
   of it is a rules-correctness argument first and a perf change second**.
2. **The `ProbeCell` inits (3,376 clones) — already largely paid, and the
   `try_init` caller table will mislead you into re-taking it.** Three
   different `OnceCell`s share that table and only one holds a `GameState`.
   The 3,376 are exactly `sim_spell_action_inner` 1,280 +
   `main_phase_action_with` 1,152 + `cast_candidates` 944, and `ProbeCell`'s
   whole point is that a previous pass *already* made them lazy (its doc
   comment: `sim_spell_action_inner` alone took 3,732 templates per six
   bench games and probed on at most 1,552). What is left is inits that were
   genuinely needed. **The table's two big rows are not `GameState` at
   all** — `evaluate_requirement_static_hinted` 106,242 / 8,520,486 is a
   static-eval memo at 80 Ir a call, and `can_afford_in_state_with` 5,712 /
   19,593,552 is `SweepMana`'s `AvailableMana` cell, i.e. `available_mana`,
   which is **(-41)**, not this entry. Check which cell a `try_init` row
   belongs to before costing it; this write-up got it wrong on the first
   pass for exactly that reason.
3. **`accept_on` (6,660 clones), and it is the one to leave alone.** The
   divergence is the point: probing N candidate actions against one template
   means N of them must diverge, and CoW already makes that cost
   proportional to what each action touches. The only waste is the probes
   that return `None`, and that is not knowable in advance.

**Size any of it against the clock before believing the Ir**, and this
family is the exact shape the standing rules warn about twice over: 17.0 M
of the 80.9 M is `__rust_alloc` under callgrind's *system* allocator when
mimalloc ships, and the rest is `memcpy`, which pass 57's `cae6b605` showed
reads -1.95 % in Ir and flat on nine alternated `selfplay_train` pairs.
Get the `selfplay_train` number first.

**(-42) PAID at the fifty-eighth pass, in two commits: `sos` -0.261 % then
-0.120 %, `cube` -0.253 % then -0.115 %, and `do_untap`'s `make_mut` calls
went 212,012 -> 41,200.** The
answer was **`Player` is itself a CoW handle** — `Player::deref_mut` is
`Arc::make_mut` — so the `for pl in &mut self.players` per-turn reset, about
fifty-five field writes per seat, took one unshare *per field*. One
`let pl = &mut **pl;` collapses them.

**The generalisable shape, and it is the CowBox sharp edge from the other
side:** a run of writes through a CoW *handle* pays a refcount check each,
and the handles in this engine are `Player`, `CardInstance`, `CowBox` and the
`cold` groups behind them. Wherever a sweep writes several fields of one
handle in a row — the cleanup step's per-turn resets are the obvious next
place — bind the target once. The reads are already free: rustc picks `Deref`
for a place used immutably even through a `&mut` binding (checked with a
standalone test; see below).

**The tail is paid too, in a second commit: 80,148 -> 41,200 calls
(-48.6 %), `sos` -0.120 %, `cube` -0.115 %.** Three runs of
`self.players[p].X = …` (7, 5 and 5 writes) took a `let me = &mut
*self.players[p];`, and three `for pl in &mut self.players` loops took the
`let pl = &mut **pl;`. The interleaved `retain_cold!`/`clear_cold!` calls on
`self` are what had kept them out of the first commit; they only split the
runs, they do not force a re-unshare, so each run collapses on its own.
Ir saved 1,933,414 on `sos` against 1,168,440 of `make_mut` self — the other
765 k is the per-write preamble (index, `Arc` load, refcount load, branch)
that went with it.

**What is left is 41,200 calls, 27.5 per untap step**, in the main untap
loop's per-card writes. `clear_end_of_turn_effects` is already one
`deref_mut` for its whole macro-expanded reset *and* gated on
`end_of_turn_effects_are_clear()`, so the residual is the gated singletons
(`granted_flashback_eot`, `granted_harmonize_eot`, `damage`) — one write
each, nothing left to bind once. This entry is closed as a site; the
*device* is open everywhere else (see the generalisable shape above, and the
cleanup step).

**The base was re-read before the delta was quoted, and it is the whole
reason the number above is 0.120 % and not 2.069 %.** The first reading
compared against `1,639,754,965`, the figure in this pass's Baseline block —
but that was measured before a concurrent session's `cae6b605` rebased in
underneath, and the true base at `d2a8320b` is **1,607,757,957**. The stale
column would have credited this commit with 17x its actual win — and that
session's own write-up, landing in the same rebase, prices `cae6b605` on
`sos` at **-1.946 %**, which is exactly the gap. The rule
already in TODO's NEXT ("re-read the base after a rebase before quoting a
delta") was written for `--decks sealed --games 1` and binary size; it
applies just as hard to a game pool when the branch is shared. **A predicted
size that comes in 17x high is the signal — model the win first, then let a
miss that large indict the base rather than the change.**

The original reading, for the sizing: 1,498 untap steps, **212,012 `make_mut`
calls** at 45.7 Ir each — 9,694,094 Ir, **0.58 % of the run** — against 9,832
`clear_summoning_sickness` calls and 6,960 Ir of `remove_counters`. On cube
it was **363,462 calls / 15,771,096 Ir**, the largest `make_mut` caller in the
program by count (27 % of all of them) and 12 % of the program's 1.33 M
`make_mut` calls for one turn-based action.

**TWO explanations are refuted, and between them they narrow the entry to
"not the obvious places". Read both before proposing anything.**

* **Reads through `&mut` do not pay a `DerefMut`.** rustc picks `Deref` for a
  place expression used immutably even when the base binding is `&mut`, and a
  standalone test of exactly this shape (an `Arc::make_mut` in `DerefMut` with
  a counter) reads **zero** `deref_mut` calls after a loop body of pure reads
  and one after a single write. So the 141.5 are real writes.
* **They are not the per-turn flag roll-over loop**, which is the obvious
  candidate: `goaded_by` / `detained_by` / `attacked_this_turn` /
  `blocked_this_turn` / `blocked_attackers_this_turn` / `attacked_last_turn` /
  `attacked_own_turn` / `attack_ban`, up to seven writes per permanent. Built
  and measured at the fifty-eighth tip: the reads moved to a shared reborrow,
  the whole block gated on "does anything want a write at all", and the writes
  taking **one** `&mut **card` between them. `do_untap`'s `make_mut` count
  went **212,012 -> 211,298** — 714 calls, 0.34 % of them — and the run read
  `sos` **+0.0008 %**, `cube` **+0.0004 %**. Reverted. Pass 43's per-write
  gates were already doing their job; the loop writes almost nothing.

**The line profile is what named it, and the entry is worth reading for how
cheap the answer was once the right tool ran.** `cg_lines.py --in do_untap`
on a `profiling-lines` build (11m32s cold, 552 MB binary) reports `do_untap`
as *diffuse by line* — its largest row is 365,638 Ir, 0.03 % of the run —
which is exactly the wrong-looking answer. The row that mattered was two
lines down the list: **`player.rs:1053` at 155,792 Ir, which is
`Player::deref_mut`**, sitting inside `do_untap`. The line profile does not
rank the fix; it names the *type* being unshared, and that was the whole
find. Both cheap refutations above assumed the card was the handle.

**(-41) `available_mana` walks the grants of every untapped permanent, and it
is the one caller of `granted_abilities_of` that can be pre-filtered.**
`--decks sos` at the fifty-eighth tip: `granted_abilities_of` is **68,538
calls / ~48 M Ir, 2.9 % of the run**, over three callers —
`effective_mana_abilities_into` 27,846 / 18.88 M, **`bot::available_mana`
25,680 / 18,159,501 (1.09 %)**, `main_phase_action_with` (via
`usable_abilities`) 15,012 / 10.91 M. Inside it,
`evaluate_requirement_static_hinted` is 39,430 calls / 19,242,702 and the
pushes cost `ActivatedAbility::clone` 11,324 / 7.87 M plus `grow_one` 11,324
/ 8.49 M.

The lever is pass 57's (C) at a different site: **ask the cheap question
first.** `available_mana` keeps only abilities that pass
`is_countable_mana_ability` (a handful of field tests), so testing the
*ability* before evaluating the grant's `SelectionRequirement` against this
permanent skips both the evaluation and the clone for every grant that is not
a mana ability.

**It is sound for `available_mana` only.** The other two callers index into
the returned list — `effective_mana_abilities_into` pushes
`(printed_count + j, …)` and `usable_abilities` `(n + i, …)`, and
`activate_ability` resolves granted abilities by that index — so a filtered
list would renumber them. Skipping a grant's requirement evaluation also
means not knowing whether it *would* have been included, so the indices
cannot be preserved by carrying the original position either.

**AND IT IS WORTH NOTHING ON `sos`, WHICH IS THE POOL THAT MATTERS. Do not
build it for the actors.** The filter only pays for grants whose ability is
*not* a countable mana ability, and **SOS has exactly one
`GrantActivatedAbility` static in the whole set** — Petrified Hamlet's
"lands with the chosen name have `{T}: Add {C}`" — which is a mana ability
and survives the filter. `--decks fixed` carries no grant at all. So the
ceiling on the shipped workload is zero, and `available_mana`'s 18.16 M is
the walk itself, not the filter evaluations inside it.

Catalog-wide there are **68 `GrantActivatedAbility` sites** and roughly a
third of them grant mana, so a *cube* board carrying one of the other
two-thirds would pay. If someone wants this, measure it on `--decks cube`
and quote that pool; it is the pass-53 rule ("which pool does the change
live on") pointing the other way for once — the change lives on a pool the
training loop does not play.

**It was also built and measured, by a second session, which is the same
answer from the other end.** `granted_abilities_of_where(.., keep: fn(&Ac
tivatedAbility) -> bool)` with `available_mana` passing
`is_countable_mana_ability`: `granted_abilities_of`'s edge to
`evaluate_requirement_static_hinted` came back at **exactly 39,430 calls /
19,241,574 Ir — unchanged to the instruction**, and `ActivatedAbility::clone`
at exactly 11,324, i.e. the filter rejected nothing at all. What it added was
24,926 `FnOnce::call_once` (the `|_| true` the two other callers now pass) and
14,504 `is_countable_mana_ability` calls, for **+0.049 % on `sos`,
+0.019 % on cube**. Reverted. **The entry stays closed for the shipped pools,
and the useful part of it is elsewhere: the same table is what named
`cae6b605`'s 11,324 clones.**

**`wants_converge` is 12,507,301 Ir / 0.42 % of a six-game cube run over 217
calls, and it is startup, not steady state.** Pass 46 gave it a thread-local
L1 in front of the process-wide map so the `format!("{self:?}")` probe runs at
most once per card name per process; those 217 are the cube pool's distinct
names, and the whole `core::fmt` family is 0.3 % of the run behind them
(158,350 `String::write_str` calls, 78,120 of them `DebugStruct::field`). A
training actor amortises it to nothing over thousands of games. **What it
does mean is that a six-game benchmark carries a fixed ~12 M of per-name
setup** — do not read a 0.4 % move on this workload as a steady-state result
without checking whether it landed there.

**(-40) THE CUBE POOL, READ FROM THE TOP AT THE FIFTY-SEVENTH TIP
(2,952,041,750).** Self cost, whole program:

| row | % | note |
|---|---|---|
| `__memcpy_avx_unaligned_erms` | **5.73** | the CoW/clone family's memory traffic |
| `dispatch_triggers_for_events` | **4.96** | was 5.14 % before this pass's (C); its self cost is what is left after the grant walk came off, and it is **diffuse by line** — the largest row inside it is 1.06 % and unresolved |
| `gather_continuous_effects_inner` | 4.00 | 118.0 M absolute, down from 195.9 M at the pass-55 tip |
| allocator family | ~11.5 | `_int_free` 3.65, `malloc` 2.75, `_int_malloc` 2.57, `free` ~2.2 |
| `Arc::clone_from_ref_in` | 3.20 | |
| `Vec::from_iter` | 2.87 | |
| `evaluate_requirement_static_hinted` | ~1.9 | 106.6 M of it came off with (C) |
| `check_state_based_actions` | ~2.1 | |
| `compute_permanent_pass` / `computed_permanent` | ~1.5 each | |
| `GameState::clone` / `Arc::make_mut` | ~1.4 / ~1.3 | |
| `sba_board_scan` | ~1.4 | |

**`dispatch_triggers_for_events` has now been read from the top, and the
answer was one callee, not the function.** Its caller table is
`perform_action_inner` 103,598 of 125,820 calls; its callee table put
`statics_granted_triggers_inner` at **235,062 calls / 113.1 M inclusive**,
and (C) took all of it. What is left is genuinely diffuse: by line the
largest row inside it is 1.06 % and unresolved, then 0.36 %, 0.36 %,
0.29 % — the forty-ninth pass's "measured diffuse" verdict, now with a
measurement behind it. **The remaining named callees are
`dispatch_board_scan` (67,360 calls / 35.5 M), `event_matches_spec` (662,098
/ 19.2 M at 29 Ir each) and `push_ordered_trigger_candidates` (67,352 /
13.2 M)**; none is a fan of narrow walks and none has an obvious question to
ask first.

**The one thing left to rank off this table is the clone/allocator family:
`__memcpy` + the allocator + `Arc::clone_from_ref_in` + `make_mut` +
`GameState::clone` is ~23 % between them**, which is (-10)/(-13)'s
checkpoint-sharing cost seen from the profile side.

**READ FROM THE TOP AT THE FIFTY-EIGHTH TIP, ON `sos` — the pool the shipped
workload plays — AND THE ANSWER IS: DIFFUSE. It is bigger there than on
cube and there is still no caller to take.** `--decks sos` is 1,658,496,337
and the family is **26.5 %** of it: `__memcpy` **8.21** (against cube's
5.73), the allocator 12.66 (`_int_free` 4.06, `malloc` 3.16, `_int_malloc`
2.85, `free` 2.59), `clone_from_ref_in` 2.92, `GameState::clone` 1.50,
`make_mut` 1.26.

`__memcpy`'s caller table on `sos` is 1,293,413 calls, and **the top twenty
callers hold barely half of them — 630,711 are spread over 21,130 rows**
(48.76 %). The named half: `GameState::clone` 103,330, `finalize_cast`
92,590, `String::write_str` 65,285 (the `wants_converge` startup below),
`computed_permanent` 60,556, `Vec::clone` 34,547, `mana::cost` 28,704 (a card
factory building the pool's definitions — startup), `clone_from_ref_in`
28,010, `RawTable::clone` 24,672, `ActivatedAbility::clone` 23,118,
`Iterator::partition` 20,424, `effective_mana_abilities_into` 18,880.
`GameState::clone` itself is **32,600 calls on cube**: `accept_on` 12,012,
the `perform_action` checkpoint 8,964, the bot's probe cells 6,300 (see
`ProbeCell`), `sim_start_state` 2,170, `evaluate_action_sequence` 1,290,
`main_phase_action_with` 1,252, `score_settled_state` 496 — i.e. the dry-run
probes and the transaction, which (-10)/(-13) costed and refused.

`Arc::make_mut` is **1,331,820 calls on cube** of which only 169,420 actually
unshare (12.7 %), so the *fast* path is the cost and it is a call count
problem, not a copying one. Its biggest caller by count is `do_untap` —
see (-42), which is the one row of this family that is not diffuse.

**So: do not open the clone family as a whole again.** What is left in it are
the two named rows with their own entries, (-41) and (-42), and the
allocation table by call count, which is (-23)'s recipe.

**The gather is the target, and the question is scope count, not gating.**
59,470 gathers for six games; a freeze scope's *first* computed read gathers
to fill its memo, so the count is roughly "how many scopes, plus every
unscoped read". Gating that first read is refuted (see the fifty-fifth
pass's Log).

**The gather's internals are now read by line, and the `from_iter` row was
the smaller half.** The fifty-seventh pass ran `profiling-lines` +
`cg_lines.py --in gather_continuous_effects_inner` on cube, which this entry
had been asking for. The gather's 195.9 M self / 6.12 % breaks down as
iteration machinery, not allocation: `macros.rs:?` 29.8 M,
`non_null.rs:444` 18.7 M, `non_null.rs:1720` 15.7 M, `mod.rs:?` 12.9 M,
`macros.rs:180` 9.8 M — the thirty-eight per-static passes walking
`sa_cards` and re-reading every card's `static_abilities`. **The tell is a
run of identically-costed rows** (ten at 511,188 Ir, eight at 425,990, seven
at 340,792), and none of them was inside `cg_lines.py`'s default forty-five;
`--rows` was added for it. Pass 57 took those with a variant bitmask
(`sos` -3.43 %, `cube` -2.52 %). The two `collect()`s pass 55's (I) removed
were the other half and worth a fifth as much on `cube`.

**Where the gathers come from** (`cg_contexts.py`, `--separate-callers=3`,
cube, the fifty-fifth tip — the top of 74 contexts):

```text
6,098  computed_permanent <- resolve_combat <- advance_step
5,976  frozen_effects <- board_keyword_in_scope <- declare_attackers_banded
5,166  computed_permanent <- permanent_value_with <- eval_material_inner
5,114  compute_permanents <- combat_damage_computed <- resolve_combat
4,398  frozen_effects <- board_keyword_in_scope <- declare_blockers
3,828  computed_permanent <- with_frozen_layers <- declare_blockers
2,588  computed_permanent <- resolve_combat <- submit_decision
2,000  check_state_based_actions <- resolve_combat <- advance_step
```

**Combat is over a third of it** — `resolve_combat` accounts for 13,212
between three of those rows, and it is `&mut self`, so a freeze scope is not
the tool. The `declare_attackers` / `declare_blockers` rows are scope-firsts
and their gather is prepaid work for the rest of the scope, not waste.

**(-39) RE-READ AT THE EIGHTY-NINTH PASS AND IT IS ESSENTIALLY CLOSED:
`--decks sealed --games 1` is **11,223,669 Ir**, and a fifth of that is not
deck building at all.** Twenty seconds under callgrind, no rebuild of
anything but `bot_ladder`:

```text
  3,328,829  29.66 %  recommend::rank_shape              (self; 672 shape ranks)
  1,762,068  15.70 %  __memcpy                           <- 76 % is the warm-up
    843,224   7.51 %  recommend::static_build_score      (684 calls, 1,233 Ir)
  ~1,020,000   9.1 %  the allocator family
    ~558,000   5.0 %  _dl_relocate_object + tunables_init + do_lookup_x
  inclusive
  7,132,432  63.55 %  selfplay::heuristic_sealed_build   (twelve builds)
  2,858,326  25.47 %  selfplay::sealed_pool
  2,385,902  21.26 %  OnceLock::initialize   <- 163 leaked definitions, once
                                                per *thread*, not per build
```

**The 21 % `OnceLock::initialize` and the 12 % `LocalKey::with -> __memcpy`
under it are the per-thread memo warm-up — 163 distinct factories at ~8.2 k Ir
of `memcpy` apiece, which is `Box::new(f())` moving a `CardDefinition` out of
the factory's frame.** A ladder process pays it once; a `selfplay_train`
actor pays it once in a run of hundreds of thousands of games. **So this
instrument's absolute over-states the per-build cost by ~a quarter, and a
pass ranking rows in it has to subtract the warm-up first** — the steady
state is ~**600 k Ir a build**, against ~1.87 M at the eighty-third pass's
actor reading. Deck construction is now ~2 % of an actor, not 5.37 %, and a
30 % cut of `rank_shape` — the only row left with any size — is 0.3 % of the
actor. **Take something else first.**

The entry as it stood:

**(-39) THE DECK BUILDER — 111.8 M -> 34.9 M at the fifty-fourth pass
(-68.80 %), 34.6 M -> 26.6 M at the fifty-sixth (-23.13 % Ir, -14 % wall
clock) and 26.5 M -> 23.6 M at the fifty-eighth (-10.97 %), for twelve pools
+ twelve builds.** It was ~28 % of what a `selfplay_train` actor does per
game and is now ~7 %. Read all three passes' Log entries before proposing
anything here. Pass 54's device was "a definition is memoized, but everything
read off it is not", and `CardBrief` is where a new derived fact belongs;
pass 56's is one level up — **ask what varies with the shape**; pass 58 is
that question asked of the three things pass 56 left (the splash ranker, the
score sort, the land walk) and the answer was "nothing" all three times.

**The copy-cap counter is PAID at the fifty-eighth pass's (E), -7.782 %**, and
it is worth recording that this entry sized it at "~8 % of the build" off one
self-cost row and the measurement agreed to the first decimal. What is left
here is `build_shape`'s 24.77 % residual, which is the `allowed` filter chain
and the two output piles and has been diffuse for three passes; `__memcpy`
9.49 %, which is the twelve pools' definitions being built once; and
`score_brief_with_colors` 6.81 %, whose colour argument genuinely varies per
shape (one attempt on the scorer is refuted at +2.88 %).

**The structural answer this entry used to propose — resolving a pool's
definitions once into a `Vec<Arc<CardDefinition>>` and indexing it, a
signature change through `draft.rs` / `recommend.rs` — is superseded and
should not be started.** The 4,096-slot direct-mapped front cache
(`b10fdebd`) took the same 20.8 % without touching a signature, and the
derived-facts memo (`9cc1175c`) took the re-derivation the index change
would not have.

The leftover table is in the fifty-eighth pass's Log entry, read at its tip
(21,774,018). `generate_sos_pack`, `candidate_label`, `Vec::retain` and the
copy-cap `HashMap` are all off it now, and the sorts fell from ~8 % across
three rows to 2.74 % in one.

**A second-order note the fifty-sixth pass's wall-clock pair exposed: the
tip's process startup floor is 6.5 % higher than the base's** (0.3294 s
against 0.3093 s per 100 processes). `PoolScores` and `SosPacks` add code and
the binary got bigger; on a workload this short that is a real fraction of
what the change saved, and it is why -23.1 % in Ir read -14 % on the clock.
A training actor never pays it — it builds its decks in one long-lived
process.

**The `--use-deck-best` configuration stays CLOSED** — see the fifty-third
pass. `best_build_by(pool, 32, ..)` runs thirty-two `build_random_deck`s per
side per game; that pass's lattice hoist took the judged path from 1.2 to
83.2 games/s against 99.8 for the unjudged one, and every build the fifty-
fourth pass made cheaper is inside it.

**(-38) TAKEN WHOLE AT THE HUNDREDTH PASS' FOURTH COMMIT** — `fixed`
-0.283 / `cube` -0.446 / `sealed` -0.299 %, a one-entry validated index memo
on `Battlefield` (`find_by_id`; the Log block carries the device and the row
table). Six passes had been paying this class off one call site at a time
with `_on` forms; the memo takes the *repeat* half of all 556 of them at once,
and it needs no invalidation because a hit is verified rather than trusted.
**The entry is CLOSED.** Its one residue — the two-path body being bigger
than the bare `iter().find()`, so `bf_hint_or_find` and `battlefield_find`
de-inlined at some sites, net +1.6 / +2.1 / +3.5 M — was built as `(-94)` and
is not recoverable: an out-of-line scan re-inlines those rows and loses more
than it gives back, because the hit rate is near one half.
**The `_on` conversions this entry's remaining rows asked for are now worth
much less than they were** — a second find of a permanent the caller already
looked up is a load and a compare — so re-size one before writing it.

**(-38, original) `battlefield_find` is 4.03 % of the program and has never had an
entry, because it never appears as a function row — it always inlines.**
`self.battlefield.iter().find(|c| c.id == id)`, 556 call sites.
**`scripts/cg_sites.py` is the tool that finds this shape** (added with the
entry): it groups hot addresses by the frame just outside the needle, i.e.
by call site, and it is the only one of the three scripts that can see a
function with no row and no line of its own. At the fifty-third tip the same
table on `--decks sealed` reads 2.40 %, still the largest unnamed structural
cost. The
fifty-third pass took the largest one (`eval.rs:3271`, 1.72 % alone) by
handing the permanent in; the rest of the table, from `cg_lines.py` at that
pass's base:

| call site | Ir | % |
|---|---|---|
| `evaluate_requirement_static` (`eval.rs:3271`) | 18,008,634 | 1.71 — **PAID** (`9bf2ae2e`) |
| `find_card_anywhere`'s first leg (`mod.rs:21269`) | 3,614,532 | 0.34 |
| `all_damage_to_player_prevented` (`mod.rs:12212`) | 2,230,452 | 0.21 — **PAID** (`4a951123`) |
| `auto_tap_for_cost_inner`'s source table (`actions.rs:12626`) | 1,621,360 | 0.15 — **PAID** (`02caa399`, and it measured -0.291 %) |
| `bot::permanent_value` (`bot.rs:3003`) | 1,527,960 | 0.14 — **PAID** (`4a951123`, `permanent_value_with`) |
| `pick_blocks_inner` (`bot.rs:8937` / `8702`) | 1,050,364 | 0.10 |

**Three of those six rows are paid.** `all_damage_to_player_prevented` and
`bot::permanent_value` came off at `4a951123` (the fifty-third pass's (H),
-0.611 % on `fixed`); the auto-tap source table came off at the fifty-sixth
pass's `02caa399` — a `bf_idx` hint on `ManaSourceInfo`, and it measured
**-0.291 % on `fixed`, twice what this table sized it at.** That is
`cg_sites.py`'s own "read the number as a floor" with a second data point
(pass 53's two sites read 0.35 % and measured -0.611 %): **do not decline a
site because its `cg_sites` row looks small.** What is left unclaimed is
`find_card_anywhere`'s first leg (0.34 %) and `pick_blocks_inner` (0.10 %),
and on the floor rule both are worth more than they read.

The rest are candidate (11)'s
shape — a helper that opens with `battlefield_find` and is called from a
battlefield loop. **The structural fix that would take all of them at once
was costed and refused**: putting the `CardId` in the `CardInstance` handle
(so the scan reads a dense array instead of chasing an `Arc` per element)
buys ~20 % of the scan's Ir but doubles the handle to 16 bytes, and
`Vec<CardInstance>` is cloned on every `GameState` clone and every CoW
unshare. Do not take it on the Ir alone.

**(-37) MOSTLY PAID at the fifty-fifth pass: `--decks cube` -16.92 %, and
the entry had sized it at half that.** `has_ctype` and `has_ltype` now ask
the two presence gates that already existed;
`computed_permanent` went 680,960 -> 267,116 calls. Read that pass's Log
entry before touching the rest — it also records the two shapes that lost
(a `OnceCell` around a gate that runs once, and gating on
`!computed_absent()` first).

**The residual was sized at the pass's tip and it is nothing — this entry is
CLOSED.** `has_atype` and `has_stype` are still ungated, and gating them
would need two new predicates (`SetArtifactSubtypes` / `AddArtifactSubtype`
fold into a battlefield-shape scan — Bludgeon Brawl's `brawl_equip_mv`,
`equipped_bonus.set_artifact_types`, the `AddCardType`-with-subtype static;
`AddSupertype` has two emitters, neither a printed card shape). **Do not
write them.** The requirement walker's `OnceCell::try_init` at the
fifty-fifth tip is **117,334 calls at 101 Ir**, against 581,256 at 1,084 Ir
at the pass's base — `computed_permanent` no longer appears in its caller
table at all. The two arms the pair above gated were the whole of it.

**And the fifty-sixth pass closed it from the other side, by building both
gates and measuring them: cube +0.123 %, fixed +0.075 %, sos +0.052 %,
reverted.** Two independent closes agreeing is worth the line, and together
they give the sizing rule this file did not have: **a presence gate is worth
what its arm's *call count* is worth, not what the arm costs when it is
taken.** `has_ctype`'s gate paid because `HasCreatureType` is 410,900 of the
requirement walker's 654,950 calls on cube. Count the arm.

**Ranking rule added by the fiftieth pass, and it found 13 % in one sitting:
ask what is done *twice*.** Not "what is expensive" and not "what is called
often" — what does the program compute, throw away, and then compute again?
`would_accept`'s dry run *is* the action: it clones the state and runs the
action to completion, and every caller then ran the identical action on a
state equal to the one the probe started from. Three commits took that
duplication out of the simulator, the shared pickers and the finalist
evaluators for **-13.128 %**, and none of it shows up as a hot function —
`accept_on` is a thin wrapper, and the work it repeats is spread over
`cast_spell`, `auto_tap_for_cost_inner` and `finalize_cast`, where it looks
like ordinary casting. **The tell is a validate-then-do pair**: a
`would_accept` / `is_ok()` probe followed by the same action being performed.
Grep for those pairs before ranking functions.

**(-32) LARGELY CLOSED by pass 52 (`Bot::next_action_settled` +
`.map(Picked::into_step)` + land-play adopts), for -3.72 % across
pass-52's chain. The 1,040 driver `perform_action` calls skipped there
came off `main_phase_action_with`'s finalist, `pick_stack_response`,
`pick_combat_trick`, the three `pick_land_to_play` blocks and
`legacy_pretap`.** What is left is the pre-validated finalists whose
state `cast_candidates` discards at build time — and on `--decks fixed`
they **never win**: the vanilla archetype decks reach none of the
specialty-cast `castable` blocks, so `castable` is empty and every scored
winner is already lazily probed with `settled: Some` (see the Log's "what
is left" note). The `cast_candidates` refactor that would capture them is
therefore bench-dead for the fixed-Ir number; its ~7 % ceiling is real
only on `all`/`cube`/`sos`. Budget it only if the goal is those pools.
The doc's earlier concern about `ScriptedDecider` survival is
moot: `DeciderKind` derives `Clone` and `ScriptedDecider::kind()`
carries `answers` + `asked` verbatim, so `state.decider.kind().into_boxed()`
reconstructs the queue at the same position. Server / interactive
callers keep calling `next_action` (plain form) precisely so their
`perform_action` result — the event list they broadcast — is not lost.

**ACTOR SCALING — measured 2026-08-24, and it is CLOSED at the scale this box
can test.** The seed list has asked for "games/sec at 1, half, full actor
counts — find contention if sublinear" since the candidates section was
written, and nobody had run it. `release-fast` (mimalloc — the shipped
allocator, which is the point: allocator contention is what a scaling sweep
exists to find, and callgrind's system-allocator build cannot see it),
`--a gang --b gang --games 400 --decks all --seed 11` = 6,800 games a run,
thread counts alternated 1/2/4/4/2/1 in one sitting so host drift moves both
ends. Best of two per count:

```text
threads   wall     games/s   per-thread   speedup   efficiency
  1      100.4 s     67.7       67.7        1.00x      100 %
  2       50.8 s    133.9       66.9        1.976x    98.8 %
  4       25.5 s    266.7       66.7        3.937x    98.4 %
host: Intel(R) Xeon(R) @ 2.10GHz, 4 cores, host_calib_ms 46
```

**Linear to the core count — there is no contention to find here.** What the
sweep also shows is why it wants the alternating order: the two one-thread
runs read 100.4 s and 112.6 s, a **12 % spread** on the same binary minutes
apart, while the two four-thread runs differed by 0.8 %. The long runs drift
more, so take the best of each pair, not the mean.

**What this does not answer**, and the next run should not read it as if it
did: four cores is the whole range this box has. A training host running 8,
16 or 32 `selfplay_train` actors is asking a different question, and this
sweep says nothing about it beyond "the game loop itself is not the shared
resource". Re-run it on the box that matters before sizing an actor count.

**(-34) `cast_candidates` READ FROM THE TOP at the fiftieth tip, and the
answer is one function.** The entry that said "3.0 %, never read from the top"
is closed: `cast_candidates` is **105,425,302 / 7.93 % over 7,238 calls**, and
**93,499,518 of it (7.09 %) is the single `.collect()` that builds the plain-cast
candidate list** — 12,917 Ir a call. Inside that collect, by caller edge:

| row | calls | Ir | % | Ir/call |
|---|---|---|---|---|
| `auto_targets_for_effect_all_slots` | 2,942 | 54,523,024 | **4.13** | **18,533** |
| `can_afford_in_state_with` | 12,986 | 31,754,663 | 2.41 | 2,445 |

Neither is redundant work: the target walk runs 0.41 times per
`cast_candidates` call (only targeted candidates reach it) and the
affordability check 1.8 times (the five hand filters drop the rest first).
**The cost is inside the target walk**, and it is one row —
`evaluate_requirement_static` **113,726 calls / 30,595,070 / 2.32 % at 269 Ir
each, i.e. 38.7 requirement evaluations per targeted candidate**: every
possible target for every slot, against the slot's `SelectionRequirement`. A
card offering three modes pays three whole enumerations, and the requirement
differs per mode, so there is nothing to share between them.

**(-35) HALF PAID (`9bf2ae2e`). The 269 Ir this entry told its taker to look
at was mostly one line: `eval.rs:3271`'s `battlefield_find`, 18,188,014 Ir /
1.72 % of the program, 46 % of the function's own cost.** Handing the
permanent in took `fixed` -0.642 %. What is left is the enumeration, which
is the bot's job, and the four ungated `computed()` arms — see (-37).

**(-35, historical) `evaluate_requirement_static` — 182,532 calls, 33,530,088 self /
2.52 %, the largest non-allocator self row after
`dispatch_triggers_for_events`.** Callers: the target walk 113,726, a
`Map::try_fold` 30,374, its own recursion 28,000 (two arities), the cast's
own closure 1,426. Its callees say the lazy shape is already in place — two
`OnceCell`s per call, `try_init` firing on only **15,562 of the 182,532**
(8.5 %) for 8,834,020 + 9,536,344 of closure. Both cells are keyed to the
*target* being tested, so they cannot be hoisted out of the per-target loop
above them. **Whoever takes this should look at the 269 Ir, not the count:**
the count is the enumeration and the enumeration is the bot's job.

**(-33) `trigger_grant_sources` — 29,448 whole-board grant walks, 14,134,170
Ir / 1.06 %, all of it self, and this is the first pass that could see it.**
The fixed `cg_lines.py` (see **How to measure**) puts the cost inside it at
`option.rs` 4.32 M (the `active_static` peel per static ability), the slice
iteration 7.2 M across `mod.rs`/`mut_ptr.rs`/`non_null.rs`, and `out.push`
0.24 M — i.e. **530,064 battlefield-source visits looking for a
`GrantTriggeredAbility` static that the bench boards do not have.** Callers:
`fire_step_triggers` 14,898 (already hoisted — one walk per call, gated per
card, and it is called once per step) and `statics_granted_triggers_for`
14,550, which is the *per-card shim* and rebuilds the whole list every time.

**The shim's five callers never took the `_with` form its own doc was written
for** ("so a caller asking about every battlefield permanent peels and
resolves the grants once instead of once per card"):
`declare_attackers_banded` 3,960, `fire_combat_damage_to_creature_triggers`
3,826, `fire_combat_damage_to_player_triggers` 2,694,
`resolve_top_of_stack_inner` 2,276, `fire_self_etb_triggers` 1,784. Only the
first three are in a loop with anything to hoist to — an attacker loop and a
damage event — so **hoisting is worth ~6,000 of the 29,448 walks, ~2.9 M,
~0.22 %**, and it means threading a grant list through the combat damage
path. Size it before starting.

**What will *not* work here, and the file has the measurements:** there is no
cheap presence gate, because the gate is the walk — most sources have no
static abilities at all, so the full walk already skips them and a pre-pass
costs the same. A board-level memo with an epoch is (-18), refuted at
+0.727 %. Fusing the question into a walk that already happens is the device
that has now lost four times.

**Ranking rule added by the forty-ninth pass, and it is about how you read
the profile rather than about the code:** **rank the tail, not the function.**
A chain of narrow generators — `main_phase_action_with`'s twenty-two `pick_*`
fallbacks, and anything else shaped like one — is invisible in a self-cost
profile (none of those reached 0.8 %) and invisible in a callee table sorted
by Ir. It shows up only when the **call counts** are read: twenty-two rows at
exactly 2,176 calls each, once per traversal, on a board that had nothing for
any of them. Together they were **4.9 %**. Sorting by Ir will never find one.
Wherever the code reads as a fallback chain, count the rows before costing
them, and gate the whole call rather than reordering its prologue.

**And the corollary that made it safe:** `spec` / `gated_block!`'s debug audit
(run the block anyway in a debug build, assert it produced nothing) is what
lets a mask over-approximate freely. The 18,709-test suite becomes the
mask's proof on real boards, so a bit can be added without re-deriving the
generator's filter by hand. `gated_pick!` is the same macro for a generator
that returns rather than appends.

**Ranking rule, corrected by the thirty-seventh pass's two null results:** a
`computed_permanent` site is worth a gate iff it is reachable only from
`&mut self` code, because a freeze scope holds `&GameState` and no `&mut self`
call can happen inside one. Ir/call does not answer this — `apply_layers_one`
alone spans ~760 to ~2,200 Ir — and two candidates picked on Ir/call alone
measured -0.019 % and -0.015 %. Check the caller's `self` binding first.

**Ranking rule added by the forty-second pass, and it is the one that paid:**
before ranking a *function*, ask what an ordinary action pays that it cannot
possibly need. Three of that pass's seven rows were a question asked more
times than it has answers (thirteen `is_mana_ability` walks, fourteen
battlefield walks, two whole-board `HashSet`s per untap step) and two were a
write that changed nothing. None of the five was on this list, and none shows
up as an expensive function — they land in `make_mut`, `memcpy` and
`_int_malloc`.

**Ranking rule the forty-seventh pass's last two commits added, and it is
about the *build*, not the code:** `release-fast` / `profiling-fast` have
**no LTO**, so a small non-generic function in `crabomination_base` is an
out-of-line call in every profile this file quotes — while the shipped
`release` profile (`lto = "thin"`) would inline it. **That makes a pure
`#[inline]` change unmeasurable here and possibly worthless there: do not
take one on an Ir number.** What *is* sound is making the callee smaller
than any inliner threshold, which is what `[Keyword]::has_kw` does — a
three-instruction discriminant test in front of a 200-arm `match` no
inliner would ever take. `CardDefinition::is_creature` (~11.5 M / 0.67 %
over ~950 k calls, a `Vec<CardType>::contains`) is the same family and the
same trap: an `#[inline]` on it would read as a win here and be nothing in
`release`. A `CardTypeSet` bitset is the only shape that beats it and it is
(-11)'s staleness hazard again.

**Ranking rule added by the forty-seventh pass, and it is the inverse of the
one this file has applied since the thirty-eighth:** a **presence gate is a
loss where the gather it avoids has already happened.**
`keyword_grant_in_scope` costs one `card_can_grant_keyword` per battlefield
permanent *plus* per command-zone card *plus* per graveyard card — ~93 calls
on a late-game board, more than a `computed_permanent` memo lookup — so
before adding or keeping one, ask whether the caller runs inside a live
freeze scope. `layers_memoized()` answers that without gathering and is the
device to reach for; the forty-seventh pass's (B) is -0.570 % of exactly
this. It cuts the other way too: the land tap's CR 602.5 gate runs from
`&mut self` with no scope open, and there the gate is still the cheap side.

**The clause the forty-eighth pass had to supply, having measured both
sides of it in one sitting:** the question is not "has the gather happened"
but **"does anyone else in this scope read it".** `scan_land_type_rewrites`
is the first computed read in `mana_source_table`'s scope and nothing after
it touches the memo, so trading its gather for
`land_type_change_in_scope` took **-0.747 %**. `board_keyword_matching` is
also a scope's first computed read, but all three of its callers go on to
`compute_battlefield()` or a run of `computed_permanent`s in the same scope
— the gather is *used* — and the same trade there read **+0.30 %** and was
reverted. Read what the scope does after the question before swapping a
gather for a gate.

**And the rule that has now lost four times: do not fuse a cheap per-card
question into a walk that is already happening.** (-8b) hoisting
`card_type_change_in_scope` into `sba_board_scan` (+0.77 %), the `do_untap`
gate (+0.0001 %), `creature_death_possible`'s three-into-one (+0.55 %, and
+1.24 % for the wider version), and the forty-eighth pass's trigger-carrier
bitmask out of `dispatch_board_scan` (+0.58 %). The loads you add to the
first walk cost what you save in the second, every time. What *does* pay is
removing a walk outright — the forty-eighth pass's (B), three whole-board
`static_abilities` scans in the cast's uncounterable check collapsed to one,
**-0.170 %**.

**(-28) `finalize_cast` is 111,559,898 / 6.79 % over 7,172 casts and was read
from the top for the first time at the forty-eighth pass. Two rows came off
it; what is left is one class, and it is not local to this function.** Its
callee table at the forty-eighth tip: `dispatch_triggers_for_events`
21,488,540 over 10,898, `__memcpy` **13,451,064 over 142,796 calls — 19.9 per
cast**, `grow_one` 8,998,811 over **28,878 — 4.03 per cast**,
`Vec::from_iter` 5,313,642 over 21,516, `__rust_alloc` 2,766,997 over
**24,108 — 3.36 per cast**, `find_card_anywhere` 3,272,638 over 21,736.

**The 4.03 growths looked like the class and are not a lever — that half of
this entry is CLOSED.** `Vec::clone` hands back `capacity == len`, so the
first push into any `Vec` after a checkpoint clone or a CoW unshare
reallocates; `finalize_cast`'s four are two of `PlayerData`'s per-turn cast
logs, `spell_names_cast_this_turn`, and `self.stack`, and the same shape is
every large `grow_one` caller in the program (224,481 growths). **A `GrowVec`
newtype whose clone reserves headroom was built and measured at +0.050 %** —
the clone-side cost of `with_capacity` + `extend_from_slice` is the same order
as the growth it removes. See the forty-eighth pass's Log; do not rebuild it.

**What is left of this entry, and it is small.** `CastProfile.card_types` is
a `Vec<CardType>` cloned per cast *and* per `PlayerData` unshare — a
`CardTypeSet` bitset over 14 variants removes both allocations and custom
serde keeps the wire; that is one of `finalize_cast`'s 3.36 allocations per
cast, so size it at ~0.1 %. Everything else here is **diffuse**: the line
profile's largest row inside `finalize_cast` is 331,402 Ir.

**(-29) `Arc::clone_from_ref_in` — 152,062 allocations, 15,118,806 Ir of
allocation and 52,912,414 / 3.22 % of self, the largest unclaimed structural
row.** The CoW unshare of `CardInstance` / `PlayerData`, ~6 per checkpointed
action. Under it: `Vec::clone` 194,386 calls (mostly empty, 42 Ir each) and
`RawTable::clone` 34,220 at ~137 Ir. **The `RawTable` half is PAID** — the
forty-eighth pass's (F), the two per-turn `PlayerData` sets to `IdSet`,
-0.182 %; what is left there is `spells_cast_by_name_this_game`, which is
game-long and is data. **The rest of the entry is (-13)'s and is measured
dead**: the unshare itself is the checkpoint sharing every zone, `clone_from`
lost at +2.60 % and narrowing `GameState` was costed and refused. What has
never been read is the *`CardData` side*: which of its ~110 fields actually
cost something in a `clone_from_ref_in`, given the `Vec::clone`s under it are
42 Ir each and therefore mostly empty.

**(-30) Three callers of `computed_permanent` pay a whole-game gather per
call, and they are named now.** From the forty-eighth tip's caller table
(Ir/call is the tell — ~2,000 is a gather, ~300 a memo hit):
`resolve_combat` **5,682 at 2,080** (diffuse, see (-25)),
`check_target_legality_with_source` **4,692 at 2,206**, and
`push_ward_triggers_for_targets` **1,536 at 2,997** — 26.8 M / 1.63 %
together. `check_target_legality_with_source` opens its *own*
`with_frozen_layers` per call, and nested freezes reuse the outer memo, so
the lever is a scope around the caller's whole target announcement: the cast
path calls it 2,718 times over ~7,640 casts (~1.36 slots a targeted cast, so
~700 gathers), and a `.collect()` caller reaches it **1,616** times — find
that loop first, it is the one with real fan-out. The Ward one cannot be
collapsed: it is ~one call per cast that has a warded target, so there is
nothing to share it with.

**(-27) `computed_permanent` allocates 93,570 `Arc<ComputedPermanent>` over
six games — the largest *named* allocator row, unclaimed, and worth
**~0.2 %, not the 0.75 % this entry first claimed**.** One `Arc::new` per
memo miss. The correction: a *frozen* miss has to hand its `Arc` to the
memo, so only the **~25,358 unscoped** calls can avoid one — and each of
those also pays a full gather (1,953 Ir) that dwarfs the ~130 Ir of
alloc+free. Outside a freeze scope the `Arc` is created, read once and
dropped with nothing to share it with. The shape to cost first: a return type that can be
owned-or-shared (`Cow`-like, `Deref` to `ComputedPermanent`) so the unscoped
path stays on the stack. It touches every caller that writes
`.map(|c| …)` / `.is_some_and(…)` on the current `Option<Arc<…>>`, so size
the call-site churn before starting. **Do not** reach for a pool: (-13)
measured the husk-pool shape at +2.60 %.

**(-31) `improves_this_turn` — READ FROM THE TOP AND REFUTED ON COST by the
forty-ninth pass. Do not build the reuse; see that pass's Log entry for the
call counts.** 842 evaluated finalists across 920 `pick_by_outcome` calls
means at least 499 of them evaluate nothing, so on more than half the ticks
that reach it there is no prior evaluation of the winner to lift. What is
left of the entry is the strength question — the gate costs ~6 % of simulator
throughput — and the analysis below, kept because it is the map of the path.

`main_phase_action_with`'s summon-sick / hold-instants gate calls it once per
winning line (474 calls, 948 combat sims). The `before` half fast-forwards an
idle clone through combat; the `after` half resolves `best`, runs it to
quiescence and fast-forwards *that* through combat. But `pick_by_outcome` ->
`evaluate_action_outcome` -> `evaluate_action_sequence` -> `score_settled_state`
already did exactly that for `best`, on a clone, at depth 0, before recursing
into follow-ups. Only the final scoring function differs
(`eval_material` vs `eval_material_summon_sick_blind`), and both read the same
settled state.

**Three things stop it being a straight lift, and all three are behaviour, not
plumbing.** (i) `evaluate_action_sequence` returns the max over follow-up
sequences, so the value it hands back is not the depth-0 state's — the depth-0
`score_settled_state` call is the one to tee off, not the return. (ii) With
`w.determinize > 0` the outcome eval runs on a *redealt* `sim_start_state`
while `improves_this_turn` uses the true state, so a shared score changes what
the gate answers. (iii) `pick_by_outcome` skips the outcome eval entirely for
`action_outcome_is_temporary` candidates, so a temporary winner has no state
to reuse. Ceiling is about half of the 6.1 %; budget it as a search-plumbing
pass with a determinize-on A/B and a golden-trace diff, not as a drive-by.

**(-26) `main_phase_action_with` — CLOSED by the forty-ninth pass, -4.867 %.**
The tail below the cast block was twenty-four narrow generators, all reached
2,176 times, and gating them behind one board-facts mask was the whole entry.
What is left of the function is search and the engine: `pick_by_outcome`
7.42 % over 920 (a search-quality decision, as this entry suspected — the
finalist count is `EVAL_TOP = 3` and the cost per call is the dry-run, not
enumeration), `would_accept` 6.6 % over 2,036 lazy candidate probes,
`improves_this_turn` 6.1 % ((-31)), `cast_candidates` 3.0 % over 3,506. **Do
not reopen this entry as a whole**; (-31) is refuted and `cast_candidates` was
read from the top at the fiftieth tip — see (-34). The historical entry:

**(-26, historical) `main_phase_action_with` is 552,113,968 / 32.97 % and has never been
read from the top.** Second-largest bot subtree after `pick_attacks_scored`.
Its callee table at the forty-seventh tip:

| callee | Ir | % | calls |
|---|---|---|---|
| `pick_by_outcome` | 119,663,481 | 7.08 | **920** (130,069 Ir a call) |
| `would_accept*` (affordances) | 108,264,447 | 6.40 | — |
| `cast_candidates` | 47,284,337 | 2.80 | 3,506 |
| `perform_action` | 45,339,965 | 2.68 | — |
| `computed_permanent` | 20,737,266 | 1.23 | — |
| `pick_land_to_play` | 15,420,727 | 0.91 | 1,488 |
| `pick_sacrifice_value` | 11,990,848 | 0.71 | 2,176 |

`pick_by_outcome` at 130,069 Ir a call over 920 calls is the thing to read
first, and it is search, not engine — so **check whether the count is a
search-quality decision before treating it as a perf one**, the same answer
(-21) reached for `attack_candidates_for_mcts`. `next_action_inner` runs
inside a `with_frozen_layers` scope (89 % of the program is under one), so
the `computed_permanent` row is memo lookups and layer passes, not gathers;
the probes' clones reset `LayerFreeze` to unfrozen, which is why
`would_accept` gathers.

**REFUTED, forty-seventh pass — do not rebuild either of these.**

* **A lock-free depth shadow on `LayerFreeze`** (`AtomicU32` mirror of
  `state.depth`, written under the mutex, read `Relaxed`, so every unfrozen
  `frozen_effects` / `computed_permanent` / `layers_memoized` answers without
  an acquisition). Correct, suite green, invariants identical, **+0.027 %**.
  An uncontended `std::sync::Mutex` acquire is a handful of instructions.
  The one thing that could overturn it is a *wall-clock* A/B — Ir undercounts
  a lock-prefixed instruction badly and this removes ~100 k acquires over six
  games — and that measurement was not run. Do not reopen it on Ir alone.
* **A per-`CardDefinition` cached bitmask for `sba_board_scan`'s ten
  definition-only flags** (2,277 Ir a sweep, ~65 Ir a card, five inner `Vec`
  loops). Not built, because it is **unsound**: the engine rewrites
  definitions in place through `Arc::make_mut` (MDFC face swap, "loses all
  abilities", keyword grants), and a mutation that turns a bit from `false`
  to `true` leaves a stale `false` — the same hazard that demoted (-11).
  A `Clone` impl that resets the cache covers the `make_mut`-clones-first
  case but not the uniquely-owned one.

**(-25) `resolve_combat` — **168,952,362 / 10.09 % at the forty-seventh tip,
down from 11.86 %**, and two more of its rows are now closed.** The
forty-seventh pass's (A) took `quality_band_assigner` (5,017,771 / 0.29 % over
846 — a full gather per band question) off it outright, and (C)'s
`assign_sectors` fix took the SBA sweeps it drags behind it. The step
machinery above it at the forty-seventh tip: `advance_step` 272,354,213 /
16.26 % over 22,892, of which `resolve_combat` 168,952,362 / 10.09 %,
`do_cleanup` ~26 M over 1,764 (14,914 Ir a cleanup, `finish_cleanup`'s SBA
sweep 16.5 M of it), `fire_step_triggers` ~19 M over 13,134. **The table
below is the forty-fifth tip's** and its absolutes are stale by two passes;
the ratios and the call counts still hold.

| callee of `resolve_combat` | Ir | % | calls |
|---|---|---|---|
| `check_state_based_actions` | 71,644,252 | 4.06 | 2,646 (**27,076 Ir a sweep** — combat sweeps kill, so they are ~10x an ordinary one) |
| `deal_combat_damage_to_target` | 20,368,973 | 1.15 | 2,612 |
| `combat_damage_computed` | 16,121,577 | 0.91 | 3,226 |
| `fire_combat_damage_to_creature_triggers` | 14,152,432 | 0.80 | 3,806 |
| **`computed_permanent`** | **11,968,280** | **0.68** | **3,806 at 3,144 Ir — i.e. a full gather each, outside every scope** |
| `apply_prevention_shields` | 11,216,490 | 0.64 | 3,806 |
| `prevent_combat_to_target` | 10,391,336 | 0.59 | 2,682 |
| `quality_band_assigner` | 5,015,730 | 0.28 | 846 |
| `combat_damage_prevented_to_self` | 3,322,842 | 0.19 | 3,806 |

**The `computed_permanent` row is the one to take**, and it is (-22)'s
lexical route rather than (-18)'s dead one: 3,806 gathers is one per damage
instance, and `resolve_combat` already opens three `with_frozen_layers`
scopes around neighbouring reads (the pair prefix at combat.rs:3594, the
strike-back gate, the player-damage pair). The call is not a literal
`computed_permanent` in `resolve_combat`'s own body — it is inlined from a
helper — so **find it with `--tree=calling` on the tip binary and read the
3,806-call edges, then decide whether it can join a scope that already
exists**. Do not widen a scope across an `&mut self` call to get there; that
is unsound, and `freeze_layers_push`/`pop` make it *look* possible.

**The forty-sixth pass read the same table and narrowed it — two of the
three leads above are closed** (and the forty-seventh closed a fourth —
`quality_band_assigner` is **PAID**). The SBA row is *deaths*: 2,646 sweeps at
27,065 Ir with `remove_from_battlefield_to_graveyard_raw` 16.3 M of it over
2,938, i.e. real work, not a gate. And **the `computed_permanent` row is
diffuse, not one inlined helper**: `--tree=calling` at the 46th tip spreads
those 3,806 calls over many source lines, the largest of which is **748
calls**, and the damage loop already opens three `with_frozen_layers` scopes.
There is no single scope to widen. What is still unread and unclaimed:
`combat_damage_computed` at 4,997 Ir over 3,226 (one gather plus a layer pass
per attacker and blocker — mostly essential) and `prevent_combat_to_target`
at 3,875 over 2,682. **`quality_band_assigner` is paid** — the forty-seventh
pass's (A). What the forty-seventh pass read and did *not* take:
`apply_prevention_shields` is 3,806 calls of a funnel of ~a dozen rare-shield
tests, and its per-line annotation has **no hot line at all** — the cost is
spread over the funnel, so there is no single gate to add.

**(-24) `cast_spell` — the forty-sixth pass took `spell_kind` and the land
tap's effect clone off it, and what is left is (-12).** `try_pay_after_snapshot_mode`
-> `auto_tap_for_cost_inner` is 258,552,458 / 14.60 % over 9,034 at the 46th
tip; `activate_ability` inside it is 153,425,087 / 8.66 % over 18,830, i.e.
**the cost of a cast is 2.5 land taps**. The table below is the forty-fifth
pass's reading, from when the entry was opened; the ratios still hold.

| row | Ir | % | calls |
|---|---|---|---|
| `cast_spell` inclusive | 526,228,783 | 29.07 | 7,640 |
| -> `cast_spell_with_convoke` | 523,041,911 | 28.89 | 7,640 |
| -> `try_pay_after_snapshot_mode` | 281,184,997 | 15.53 | 8,656 |
| -> `auto_tap_for_cost_inner` | 262,553,222 | 14.50 | 18,340 |
| -> `finalize_cast` | 121,353,406 | 6.70 | 7,172 |
| `would_accept` inclusive | 337,688,158 | 18.65 | 5,260 |

**Half of a cast is paying for it**, and (-12) already owns that half. What is
new here is the denominator: 7,640 cast *attempts* for 7,172 finished casts,
so the bot's filtering is already tight and the cost is the real payment path,
not rejected probes. `would_accept`'s 64,200 Ir a probe is the same cast:
the `GameState` clone and drop are ~3,500 of it, so **the probe is not the
clone, it is the cast** — do not go after the clone.

**(-23) The class the forty-fourth pass's (C) and (D) came from, and it is
not exhausted: an allocation on a path every action takes, for a mechanic the
board does not have.** (C) was `vec![false; 2]` 53,838 times for The Ring
(-1.308 %); (D) was a `Vec` per land tap and a `Vec` per attacker-lock
(-0.227 %). **How to find the next one** — `--tree=caller` on the allocator
entries and read the *call counts*, not the Ir:

**The whole allocator caller table at the forty-fifth tip, by call count.**
1,112,945 allocations; `__rust_alloc` is 84,648,612 / 4.68 % inclusive and the
free side 117,236,275 / 6.48 %.

| direct caller of `__rust_alloc` | allocs | Ir | % |
|---|---|---|---|
| `RawVecInner::finish_grow` | 212,245 | 17,256,045 | 0.95 |
| `Vec::from_iter` (nested) | 186,820 | 13,304,418 | 0.73 |
| `Arc::clone_from_ref_in` | 152,062 | 15,001,917 | 0.83 |
| `computed_permanent` | **96,544** | 7,638,352 | 0.42 |
| `GameState::clone` | 79,204 | 4,538,793 | 0.25 |
| `gather_continuous_effects_inner` | 40,704 | 3,311,572 | 0.18 |
| `Vec::clone` | 38,748 | 4,229,994 | 0.23 |
| `RawTable::clone` | 29,428 | 1,812,606 | 0.10 |
| `Box::clone` | 26,386 | 1,458,050 | 0.08 |
| `finalize_cast` | 24,108 | 1,711,585 | 0.09 |
| `ManaPayload::clone` | 19,306 | 1,059,282 | 0.06 |
| `RawTable::reserve_rehash` | 19,280 | 1,084,448 | 0.06 |
| `frozen_effects` | 17,702 | 1,272,260 | 0.07 |
| `Vec::from_iter` (in-place) | 16,230 | 942,347 | 0.05 |
| `can_afford_in_state_with` / `can_afford_from` / `relax_cost_colors` | 12,986 **each** | ~2,300,000 | 0.13 |
| `ManaCost::reduce_generic` | 7,550 | 843,761 | 0.05 |

`can_afford_from`'s three allocations per affordability question are **PAID**
(the forty-fifth pass's (D)): the clone only existed so `reduce_generic` could
mutate, and a `Cow` borrows on the path most costs take.

**Refreshed at the forty-sixth tip: 1,021,777 allocations, down from
1,112,945.** How to get this table right — and the forty-sixth pass got it
wrong first, so read this: `callgrind_annotate --tree=caller` prints a
function's callers as `<` lines *immediately above* its `*` line, and its
callees as `>` lines below. **Take only the contiguous `<` block directly
above the `__rust_alloc` node.** A regex over a window instead picks up `>`
edges belonging to neighbouring nodes and invents rows — that produced two
plausible, entirely fictional leads ("`perform_action_inner` allocates once
per action", "`push_ordered_trigger_candidates` allocates once per
dispatch"). The second was built and measured before the mistake surfaced:
short-circuiting `continue_trigger_ordering`'s copy read **-107,468 Ir /
-0.006 %**, i.e. nothing, and was reverted. `candidates` is empty on nearly
every dispatch, so there was never a copy to skip.

The real table, `<`-block only, by call count:

| direct caller of `__rust_alloc` | allocs | note |
|---|---|---|
| `RawVecInner::finish_grow` | 212,177 | Vec growth, all callers |
| `Arc::clone_from_ref_in` | 152,062 | the CoW unshares |
| `Vec::from_iter` (nested) | 149,696 | the `.collect()`s |
| `computed_permanent` | 96,166 | one `Arc::new(ComputedPermanent)` per memo miss |
| `GameState::clone` | 79,204 | ~4 per clone — the non-`CowBox` fields. (-13) costed narrowing and said no |
| `gather_continuous_effects_inner` | 40,408 | |
| `Vec::clone` | 31,068 | |
| `RawTable::clone` | 29,428 | hash maps cloned inside the state clones. **Unread** |
| `Box::clone` | 26,386 | **Unread** |
| `finalize_cast` | 24,108 | **3.4 per cast, and the cheapest unclaimed named row**: the three per-turn cast logs plus `CastProfile.card_types`' `Vec<CardType>` clone. The logs regrow after every `PlayerData` clone because `Vec::clone` gives capacity == len, so `clear()`ing them per turn does not help. `CastProfile` is serialized in snapshots, so a bitset there is a wire change |
| `RawTable::reserve_rehash` | 19,280 | |
| `frozen_effects` | 17,702 | one per freeze scope |
| `Vec::from_iter` (in-place) | 16,230 | |
| `relax_cost_colors` / `can_afford_in_state_with` | 12,986 **each** | **PAID** — the forty-sixth pass's (E) |
| `auto_tap_for_cost_inner` | 9,544 | |
| `ManaCost::reduce_generic` | 7,550 | |
| `fire_combat_damage_triggers` | 6,530 | |
| `compute_permanent_pass` | 6,298 | |
| `bot::ward_gate_ok` | 4,082 | |
| `spell_kind` | 4,248 | `creature_types.clone()`, all that is left of it after (A)/(C) |

A count in the tens of thousands on a six-game workload is one per action or
one per permanent per action; that is the tell. **Rank by call count and then
read the source.**

**And `grow_one`'s own callers, 224,927 growths / 39,576,235 / 2.19 %:**

| site | growths | Ir |
|---|---|---|
| `Vec::push_mut` | 42,354 | 5,684,749 |
| `finalize_cast` | 28,878 | 8,899,235 |
| `advance_step` | 22,892 | 2,747,040 |
| `gather_continuous_effects_inner` | 13,730 | 4,498,998 |
| `declare_blockers` | 13,122 | 2,142,128 |
| `dispatch_board_scan` | 11,654 | 1,398,480 |
| `auto_tap_for_cost_inner` | 7,550 | 886,420 |
| `effective_mana_abilities_into` | 7,490 | 899,697 |
| `compute_permanent_pass` | 6,338 | 2,492,443 |
| `computed_permanent` | 5,450 | 1,270,798 |

A count in the tens of thousands on a six-game workload is one per action or
one per permanent per action; that is the tell. The Ir column lies here — a
`from_iter` row carries the iterator's own body — so **rank by call count and
then read the source**. `alloc_zeroed` is now zero calls; it was the cheapest
possible find and there is only ever one of those.

**(-22) SCOPE WIDENING PAID, AND LARGE — the fifty-third pass took
`--decks cube` from 7.95 G to 4.05 G (-49.1 %) with three scopes, and the
gather there fell from 32.51 % to 7.99 %.** This entry has said since the
forty-fourth pass that "widening scopes is what reaches those" and it was
right; what it could not say was *where*, because the three loops that
mattered (`dispatch_triggers_for_events`' phase 1, `fire_step_triggers`,
`fire_spell_cast_triggers`) are **dead on `--decks fixed`**. The device that
made them safe is not a new one: each already holds a shared borrow of
`self.battlefield` for its whole body, so the borrow checker has proved no
`&mut self` call happens inside. **Look for that shape before proposing
anything else here.** Use bare `freeze_layers_push`/`pop`, not
`with_frozen_layers` (the closure costs ~0.9 % because the loop's locals go
through its environment), and gate it on a fact the loop already computes.

**The caller table below is the forty-fourth tip's `fixed` reading** and is
kept for its shape; the live numbers are the four-pool table in "Profile of
record".

**(-22, historical) The gather's caller table — the *only* live entry on the gathers,
because (-18) is refuted and scope widening is the one route left.** 53,806 continuous-effect gathers on
the forty-fourth pass's tip at ~1,900 Ir each, ~102 M / 5.5 %:

| caller | Ir | % | gathers |
|---|---|---|---|
| `computed_permanent` | 63,472,587 | 3.42 | 32,510 |
| `frozen_effects` | 31,783,907 | 1.71 | 17,702 |
| `compute_permanents` | 7,255,816 | 0.39 | 3,594 |
| `check_state_based_actions` | 2,992,879 | 0.16 | — |

`gather_continuous_effects_inner` is already lean — one `sa_cards` walk
folding eleven whole-board passes, 645 Ir self — so the win is in the count,
not the gather. **(-18)'s epoch reaches the `computed_permanent` row.** The
**`frozen_effects` row is a different problem and the epoch does not touch
it**: those 17,702 are one gather per freeze *scope*, i.e. every scope that
opens and closes around a handful of reads pays a full one. Widening scopes is
what reaches those — the forty-fourth pass's (B) took 1.02 % from two of them
— but **measure each site**: two more written in the same sitting read
-11,778 Ir together, i.e. nothing, on `--decks fixed`.


**(-21) The bot's attack search is 53 % of the program, and it is the largest
single subtree in the profile.** **Refreshed at the fiftieth tip
(1,330,233,580):** `pick_attacks_scored` **706,842,699 / 53.14 %** over 438
calls; `simulate_attack_outcome_once` 699,394,707 / 52.58 % over 1,170
candidates = **597,773 Ir per candidate**, down from 825,854 — the fiftieth
pass took a whole cast out of every simulated cast, and this is where most of
it came from. `sim_step` is now **31,874 passes / 278,835,203 (21.0 %) plus
2,636 checkpointed actions / 72,020,298**, where it was 4,568 / 209,220,325.
**What is left under it is the engine's per-step cost**: `advance_step` at
11,688 Ir a step, of which `resolve_combat` is 55,816 Ir a combat. The
numbers below are the forty-ninth tip's and are kept for the shape.

* `pick_attacks_scored` **974,200,078 / 52.45 %** inclusive over 438 calls
* -> `simulate_attack_outcome_once` **966,249,395 / 52.02 % over 1,170
  candidates** = **825,854 Ir per candidate**
* -> `sim_step` **536,967,173 / 28.91 % over 35,316** = 15,204 Ir a step, ~30
  steps per candidate. **31,874 of the 36,442 `sim_step`s are `PassPriority`**
  and already take the checkpoint-free path; ~4,568 take `perform_action`.
* -> `pick_blocks` 34,213,150 over 1,508; the per-candidate state clone
  (`sim_start_state` 3,087,194 + `drop_in_place<GameState>` 12,931,434 over
  1,170).
* `attack_candidates_for_mcts` yields `2 + min(attack_search, |greedy|)`
  candidates (+1 for `walker_chip`), all distinct — there is no duplicate to
  memoize, so **the count is a search-quality decision, not a perf one**. What
  is on the table is the cost *per* step, which is the engine, and the ~4,568
  checkpoints, which are (-13)'s other half.

**(-14) A CoW handle's `DerefMut` is a deep copy, and a no-op write pays it in
full — the rule, now that the sweep is done.** `ColdState` (~90 collections
behind one `CowBox`) and `PlayerData` are both written through a handle that
is *always* shared, because `perform_action` holds a checkpoint. `clear_cold!`
/ `retain_cold!` in `game/mod.rs` are the guarded forms; `mem::take`,
`iter_mut` and whole-field assignment have to be guarded by hand. **An
extension trait cannot do this** — `self.field.method()` fires `DerefMut`
before the method body runs, so the read must be at the call site.
**The forty-sixth pass measured "guarding one promotes the next" directly, and
it is the reason not to take the biggest remaining row.**
`declare_blockers`' `self.blocks_declared_this_turn.push(...)` is
**7,088,925 Ir / 0.41 % over 2,070 pushes — 3,425 Ir each, all of it the
`ColdState` deep copy**. The very next cold write in the same loop
(`blocked_attackers.push`) reads 52,020 over 1,734 *because the group is
already unshared*. Guarding or moving out the first write therefore buys about
1.2 M of the 7.1 M and hands the rest to the second. The honest shape is one
`ColdState` clone per simulated block declaration — CoW working as designed on
a state the bot cloned — and the only lever is a cheaper `ColdState`, where
(-13) already measured `clone_from` losing.

`GameState::deref_mut` was **4,608,931 / 0.24 % over 6 sites** at the 42nd tip
and the survivors are real writes; the standing work is (i) re-read that table
after *any* change that guards or moves a cold write, because guarding one
promotes the next, and (ii) the two survivors —
`creature_deaths_this_turn.push` (2.3 M over 3,104) and
`life_gain_flag_pending.push` (1.4 M over 1,294) — are move-out candidates
like (D)'s: `life_gain_flag_pending` is `#[serde(skip)]` and empty between
batches, so moving it out costs an empty-`Vec` clone (free) and buys ~1.4 M.
`creature_deaths_this_turn` holds `CardInstance`s through the turn, so its
move-out would trade ~2.3 M for a `Vec` clone per checkpoint — probably a
wash, measure before taking it.

**Re-asked at the fifty-third pass and still refused.** The epoch's payoff
side is eight times bigger on the cube pool than on the `fixed` one it was
refuted against — but both of the refutation's reasons are workload-
independent (the counter is not free where the writes are; a ~700 Ir
predicate inlined into its caller is too cheap to memoize behind a call),
and the scope-widening route took 49 % of that pool without any new state.
Take the remaining scopes first.

**(-18) THE BOARD EPOCH — BUILT, MEASURED, REFUTED (forty-fifth pass). Do
not build it again; read the Log entry before proposing anything shaped like
it.** The write counter on `CowBox::deref_mut`, the memo reachable from
`&self`, the recompute-and-compare `debug_assert` — all of it was written,
all of it was sound (the 18,708-test suite ran green with the audit armed),
and it measured **+0.727 %** behind a `Mutex` and **+0.490 %** lock-free.

The two reasons generalise and are the useful part:

1. **The counter is not free where the writes are.** `Arc::make_mut` runs
   945,272+ times over six games, almost all of it `CardInstance`'s own
   handle. Every one pays the increment for a memo that never reads it.
2. **A ~700 Ir predicate inlined into its caller is too cheap to memoize
   behind a call.** The hit rate does not pay back the lost inlining.

**And the thing that *would* pay — the gather, ~1,900 Ir x 48,466 — is
exactly the one whose key is not enumerable.** `gather_continuous_effects_inner`
reads life totals (Aettir and Priwen's base P/T), hand sizes (Kagemaro's
Clutch), graveyard contents, `statics_ignored_this_turn`, and
`evaluate_predicate` through `active_static`'s `WhileCondition` at ~30 sites.
This file said three times that "a board-level memo with an epoch is the
shape". It is not. **The remaining route to those gathers is lexical: widen
freeze scopes, site by site, each one measured** — the forty-fourth pass's (B)
took 1.02 % from two of them and two more written the same day read
-11,778 Ir, i.e. nothing.

What is left of the entry's *numbers*, still unpaid and now without a plan:
`dispatch_board_scan` 24,561,076 / 1.36 % over 53,838 and
`permanents_with_abilities_removed` over the same 53,838 are whole-board walks
with no scope to widen, and `card_type_change_in_scope` is 22.1 M / 1.22 %
over ~30 k calls at ~736 Ir. **Do not reach for fusion there either** —
(-8b) is the same site, and hoisting it into `sba_board_scan` (a walk that
was already happening) read **+0.77 %**. Between (-8b) and this entry the
site has now lost to fusion three times and to a memo twice; and the "ask it less often" shape is **gone**:
the claim that `activate_ability_inner` asks it twice per activation is stale
— callgrind at the forty-sixth tip reads 18,864 calls over 18,830
activations, i.e. once, and one call site is left in that function.

**(-16) READ BY LINE ON `sos` FOR THE FIRST TIME (sixty-first pass), AND THE
VERDICT IS "DIFFUSE" WITH A MEASUREMENT BEHIND IT — including one commit
built against the table and reverted.** `dispatch_triggers_for_events` has
been the largest engine self row since pass 43 and every previous read of it
was on `fixed` or `cube`. `profiling-lines` + `cg_lines.py --in
dispatch_triggers_for_events` on `--decks sos` (1,535,739,641), everything
whose inline chain mentions the function — **91,537,964 Ir, 7.5 % of the
run**:

```text
 14,467,414 (1.18%)  mod.rs:?          <- no line info; the largest single row
  5,708,410 (0.47%)  macros.rs:?
  3,785,518 (0.31%)  mod.rs:22000      is_event_hardcoded's `match ev`
  3,196,160 (0.26%)  mod.rs:464        a ColdState field read (doc-comment line)
  2,948,808 (0.24%)  mod.rs:1815       "
  2,919,532 (0.24%)  mut_ptr.rs:961    slice iteration
  2,868,740 (0.23%)  non_null.rs:444   "
  2,745,712 (0.22%)  non_null.rs:1720  "
  2,718,104 (0.22%)  mod.rs:17108      `if any_static_grant || !station.is_empty()`
  2,359,736 (0.19%)  mod.rs:612        a ColdState field read
  2,222,190 (0.18%)  macros.rs:180
  2,151,812 (0.18%)  mod.rs:17241      `event_subject(ev, &ta.event.kind)`
  2,019,616 (0.17%)  mod.rs:17223      `if is_event_hardcoded(ev, &ta.event)`
  1,587,184 (0.13%)  mod.rs:17154      the FromYourGraveyard scope test
  1,431,576 (0.12%)  macros.rs:332
  1,279,136 (0.10%)  mod.rs:17240      `event_matches_spec(...)`
```

**The largest *named* engine line is 0.31 % and the top two rows have no line
info at all.** That is what diffuse looks like when you finally see it. (The
percentages are of the run total, which carries `name_index()`'s 104.7 M of
startup — see (-46); net of it every share here is ~7 % larger, and the
ranking is unchanged.)

**And the table's most promising row was taken, measured and reverted, which
is the entry's real contribution.** `is_event_hardcoded` reads
5,805,134 Ir / 0.38 % across its two rows, it is asked once per
(permanent x trigger x event), and its answer is a function of the *event*
alone plus one bit of the trigger's scope — the textbook (-45) shape. Three
`u64` masks over the batch (hardcoded-always, hardcoded-if-`SelfSource`, and
the `CreatureDied` skip that folds in Hushbringer suppression and CR 700.4's
replaced deaths), built once per dispatch, allocation-free, with a
`> 64 events` fallback. It reads **+0.123 % on `sos` and +0.187 % on
`fixed`**, and the whole regression is inside the function: `+1,917,670` on
`dispatch_triggers_for_events` and *nothing else in the program moves*.

**The rule that comes out of it, and it is about `cg_lines.py`, not about
this function: a line's Ir is what the instructions attributed to that
source construct cost, not what removing the construct would save.** The
`match ev` was already compiled to a jump the loop had to make anyway; the
mask replaced it with a branch, an `enumerate`, a shift and a test, and paid
more. **Before costing a line-profile row, ask what the loop still has to do
when the line is gone.** Two rows here answer "the same thing".

**(-16) PARTLY PAID (`36e998aa`): the phase-1 board walk runs under a freeze
scope now, which is worth nothing on `fixed` (5.29 % self, unchanged) and
took the function from 59.6 % to 3.91 % *inclusive* on `--decks cube`. The
diffuse `fixed` cost this entry describes is what is left, and it is still
diffuse — the fifty-third pass's line profile puts its largest single engine
line at 3,288,498 Ir / 0.31 % (`mod.rs:16621`, the per-card static-grant
branch test).**

**(-16, historical) `dispatch_triggers_for_events` — 141,288,450 / 7.61 % at the
forty-third pass's tip, and the entry's own open question is now answered.**
`perform_action_inner` drains every action's event list through it; **53,838
dispatches get past the empty-batch early return**. The question this entry
asked was how many of those produce a candidate trigger. **Almost none:
phase 2 (`push_ordered_trigger_candidates`) costs 1,726,302 Ir self and
~2.5 M inclusive over the whole run**, and its per-candidate work runs on the
order of **1,300 times** — so **~97.5 % of full dispatches walk the whole
board and produce nothing.**

Where the 7.61 % sits, measured:
* `dispatch_board_scan` **24,561,076 / 1.32 %** over 53,838 (456 Ir a call) —
  board-only, event-independent, and therefore **(-18)'s**, not this entry's.
* `permanents_with_abilities_removed` **7,160,454 / 0.39 %** over 53,838 —
  same, also **(-18)'s**.
* `event_matches_spec` 2,915,040 over 97,168; `apply_soulbond_pairing`
  1,440,654 over 4,060.
* The rest is the function's own walking, spread across its `slice/iter`,
  `ptr/non_null` and `vec/mod` entries. Its in-body lines are all under 1 M
  Ir — no single line is the cost.

**So the tractable 1.71 % of this entry moved to (-18), and what is left here
is a diffuse walk with no hot line.** Do not gate the battlefield loop on an
`EventKind` presence mask: building the mask is the same
`battlefield x triggered_abilities` walk the loop's fast path already does,
which is the forty-third pass's `do_untap` null result exactly.

**AND THE MEMO DEVICE DOES NOT RESCUE THAT MASK, checked at the ninety-second
pass because it is the obvious thing to try now that the device exists.** The
eighty-seventh pass's `CardMemo` families would make a *per-definition* mask
free at dispatch time — no per-board build, so the refutation above does not
apply to it — but the mask still buys nothing, and one line of source says
why: `event_matches_spec`'s **first statement** is
`if !event_kind_matches(state, event, spec, ..) { return false }`. The kind
test is already the first thing the call does, so a bit tested one frame
earlier replaces a discriminant compare with a load, a shift and a test. That
is the `is_event_hardcoded` result in this entry, restated at a different
frame. The row is real — `event_matches_spec` is **1,174,968 calls / 35.16 M /
1.16 % of `sealed`** at the ninety-second tip, 30 Ir a call and 6.0 calls a
dispatch — and it is 30 Ir of *matching*, near the floor. Fewer dispatches,
as (-59) says; not a cheaper match.

**(-17) `check_state_based_actions`' `.collect()`s — 55,720 calls to
`SpecFromIterNested::from_iter` for 35,174,999 Ir / 1.84 %, over 10,670
sweeps.** 5.2 collects per sweep. Most of the collects inside the function are
already behind an `sba_board_scan` flag, so the 5.2 are the *unguarded* ones —
find them before designing. Note the caveat that applies to every row in this
family: a `from_iter` inclusive number contains the whole nested-iterator
body, not just the allocation, so it is a *ceiling* on what a guard could
save. The same caveat applies to the two biggest `from_iter` callers in the
program, `bot::cast_candidates` (**103,318,517 / 5.39 %** over 7,238) and
`auto_tap_for_cost_inner`'s `mana_source_table` (**41,282,703 / 2.16 %** over
7,550) — those are the bot's enumeration cost re-reported, not collect
overhead.

**(-13) `perform_action`'s checkpoint — `drop_in_place<GameState>` 80,831,019
/ 4.21 % plus `GameState::clone` 41,569,486 / 2.16 %, ~6.4 % together, and the
largest *structural* cost left. Two of its three shapes are now measured
dead; read the forty-third pass's Log entry before touching this.**

* **Reuse the checkpoint's buffers — REFUTED, +2.60 %.** Built in full
  (exhaustive `clone_from`, thread-local husk pool, CoW-release on return),
  all invariants green, **-100,388 allocations**, and it still lost by
  49,813,302 Ir. `clone_from` on a 195-field struct costs ~834 Ir more per
  call than `clone` does, because `clone`'s `Self { … }` is one bulk copy of
  175 contiguous `Copy` fields and `clone_from` must interleave a drop and a
  construct per field.
* **Make `GameState` narrower — costed, not worth it.** 175 of the 195
  fields are `Copy`, so they carry no drop glue; a resolution-scratch
  `CowBox` group would take a slice of the 458-Ir `Self { … }` line and
  nothing of the 2,324-Ir drop.
* **Widen the no-checkpoint path — HALF PAID, `-2.842 %`, forty-fourth
  pass.** The whole shape was sized at `-5.47 %` by a probe that skipped
  every checkpoint (**1,810,396,553**, never panicked): the restore fires
  **zero** times in 18,208 checkpointed actions, at **5,750 Ir each** —
  clone 1,194, drop 2,324, and ~2,230 of CoW unshares the action pays only
  because the checkpoint shares its zones (63 % of the drop, **50,619,793 /
  2.64 %**, is `Arc` drop glue freeing exactly those copies). The
  round-closing `PassPriority` — **9,942 actions, 55 % of them** — is now
  skipped alongside the trivial one, for `-54,330,715`. The proof and the
  debug audit that stands in for the missing type-level one are in the
  forty-fourth pass's Log entry.
* **The closure has now been run at the other shapes, and it says no.**
  `scripts/fallibility_closure.py` (committed this pass, and the device that
  made the round-closing pass provable): `play_land` reaches **6**
  `Result` functions and **2** raise — but land drops are ~660 of the 8,266,
  so proving it buys nothing. `submit_decision` reaches **137** and **70**
  raise. A shape with seventy raisers is not proven arm by arm.
* **What is left of it: 8,266 checkpointed actions, ~45 M Ir, ~2.43 %, and
  it is the half where the checkpoint earns its keep.** Casts, activations
  and combat declarations — mana paid then a target rejected,
  `declare_attackers` failing mid-loop — i.e. the partial-mutation family
  Phase 1 exists for. `declare_blockers` and `declare_attackers_banded`
  hold 82 of the engine's `Err` sites between them. A per-arm proof here
  has to read as well as the pass one did; measuring zero failures on one
  workload does not.
* **The count, for whoever sizes it.** `perform_action` ran 18,208x on this
  workload before the pass and 8,266x after, against
  `perform_action_inner`'s 70,418; the bot's probes (`would_accept`,
  `sim_step`) already take the un-checkpointed path, so those are real
  actions.

**(-12) `auto_tap_for_cost_inner` — 277,149,670 Ir / 14.37 % inclusive over
18,340 calls, still the second-largest engine subtree.** The forty-second pass
took **-61,693,204 / -18.2 %** off it (A, B, C, E, F all land here) and the
shape has changed:

* **`activate_ability` is 163,106,449 / 8.46 % of it**, 18,340 calls at ~8,900
  Ir each, from ~13,200. The cold-group unshare, the thirteen
  `is_mana_ability` walks and ten of the fourteen battlefield walks are gone.
* `mana_source_table` is **75,281,100 / 3.90 % over 7,370**, from 81,987,227.
* **18,832 of the activations are still a land tapping for mana.** What that
  land still pays, in order: `card_keyword_possible` for CR 602.5's
  `CantActivateTapAbilities` (**21.6 M / ~1,148 Ir a call, one per
  activation** — see the new note under (-11)), `continue_ability_resolution_x`
  + `resolve_effect` (~58 M over 18,750), the trigger dispatch and the SBA
  sweep each tap drags behind it.
* Where to start next: `--tree=calling` on `activate_ability_inner` again — the
  table is different now.

**⚠ THIS ENTRY'S SOUNDNESS ARGUMENT IS DEAD — read the eighty-seventh
pass's Baseline before reusing it anywhere.** "It cannot be a lazily-cached
field because ~20 sites rewrite a definition through `Arc::make_mut`" was
true when written and stopped being true at the eighty-third pass, which
built `ColorMemo` — a memo cleared in `CardInstance::DerefMut`, the one point
every `&mut` reach into a card (including `&mut c.definition`) has to pass.
The eighty-seventh pass used the same device to memoize `sba_board_scan`'s
twenty-two definition bits for `fixed` **-0.862 %** / `cube` **-0.700 %** /
`sealed` **-0.940 %**. What survives below is the *sizing* (~0.3 % for this
particular caller's presence bit), not the impossibility.

**(-11) `card_can_grant_keyword` — re-costed at the forty-fourth pass's tip
and DEMOTED: 28.3 M / 1.52 % over 648,698 calls (43.6 Ir each), plus
`card_keyword_possible`'s own 7.75 M / 0.42 %.** The per-definition presence
bit (shape (i) below) is worth ~10-15 Ir on the *vanilla* cards only — the
ones whose five containers are all empty — so ~0.3 %, and it cannot be a
lazily-cached field: ~20 sites in `effects/mod.rs` mutate a definition in
place through `Arc::make_mut(&mut c.definition)` (keywords pushed, statics
added), so a cached bit goes stale in the unsound direction. Making it safe
needs a `CowBox`-style handle on `CardData::definition` so `Arc::make_mut`
stops compiling — real churn in `crabomination_base` for 0.3 %.
`CardDefinition` does **not** derive `PartialEq`, so the older note's reason
was wrong; the reason it is still a bad trade is the mutation sites.

**(-11, original note) Second correction, and it points somewhere new.** The fortieth pass's note
(the union device saves nothing, because a land tap reaches the *first* gate
and stops) still stands. What the forty-second pass measured is the size of
that first gate: **`card_keyword_possible` costs 21,620,266 Ir over 18,830
calls from `activate_ability_inner` alone — 1,148 Ir per activation**, and
~830 of that is `keyword_grant_in_scope` walking ~18 permanents at ~46 Ir
each. The 46 is not a decision: `card_can_grant_keyword` loads five fields
(`equipped_bonus`, `soulbond_bonus`, `level_bands`, `station`,
`static_abilities`) off a large `CardDefinition`, i.e. it is pointer-chasing,
not computing. **Two shapes, neither tried:** (i) a per-*definition* "can this
printing grant any keyword at all" bit, which collapses the five loads to one
— but `CardDefinition` is built by ~thousands of catalog factories with
`..Default::default()` and derives `PartialEq`, so it needs a normalisation
pass or a side table, not a field; (ii) a board-level memo with an epoch,
which nothing on `&self` can hold today. Do not take the fusion device; it has
lost three times.

**(-10) Allocation is still the program: 1,216,241 allocations and 1,269,976
frees in six games, ~13 % of the tip — and the forty-third pass measured what
drives a large slice of it.** `Arc::make_mut` unshares **120,004 times for
83,959,478 Ir / 4.38 %** (~700 Ir each), and 63 % of the checkpoint's drop is
the `Arc` glue freeing those same copies. **Most of that exists because the
checkpoint shares every zone**, so (-13) and this entry are the same cost seen
from two sides — take (-13) first. The per-caller breakdown below is from the
forty-second pass's tip and the counts have moved slightly; the shapes have
not. The forty-second pass took 61 k
allocations off it with (E) and (F) — the same rule as the fortieth's: **ask
what a `Vec<T>`'s `T` costs, and whether the enclosing loop runs per
permanent, before asking how often the `Vec` is built.** By direct caller at
the tip:

* `finish_grow` **215,649** — Vec growth. Top source-level caller is
  `finalize_cast` (28,878 growths, and the top `memcpy` caller in the program
  at 126,492 copies): the per-turn cast logs
  (`spell_names_cast_this_turn`, `spell_ids_cast_this_turn`,
  `spell_casts_this_turn`) each take a `push` per cast through `PlayerData`'s
  CoW handle. `CastProfile.card_types` is still a `Vec<CardType>` clone per
  cast — the `ColorSet` treatment (F) applied to card types would remove it,
  but `CardType` has no bitset yet.
* `from_iter` **187,120** — the `.collect()`s. `cast_candidates` 5.36 % and
  `mana_source_table` 2.65 % are the two named callers.
* `clone_from_ref_in` **177,500** — CoW unshares of `CardInstance` /
  `PlayerData`. Inherent to the checkpoint; see (-13).
* `computed_permanent` **103,318** — one `Arc::new(ComputedPermanent)` per
  memo miss, plus the `Vec`s inside it. 7,960,150 Ir in allocation alone.
* `effect_produces_color` — **PAID, -0.496 %, the pass's eighth commit.**
  `effect_produced_colors` is the set-valued primitive and the predicate
  delegates to it. The remaining `effect_produces_color` callers
  (`untapped_producers_of_inner`, the two affordance probes,
  `source_color_signature`) still ask one colour at a time inside a loop over
  colours — the same rewrite applies and has not been measured there.
* `granted_abilities_of` (9.96 M self over 57,484) walks
  `me.definition.static_abilities` **twice** — once for the five
  `HasActivatedAbilitiesOf*` flags, once for Conspicuous Snoop's
  `HasActivatedAbilitiesOfLibraryTop`. Fold the second into the first.

**(-15) `advance_step` — 308,637,209 / 16.09 % over 22,892, the largest engine
subtree. Both of the forty-second pass's leads into it are now closed and it
needs re-profiling from the top.**
* **The `do_untap` gate — two implementations, two answers, and the second one
  PAID `-0.168 %`** (forty-third pass, rows in the Log). Gating the *walks*
  read +0.0001 % and was reverted: a short-circuiting `any` over an empty
  `static_abilities` is nearly free. Gating the *blocks* that build
  collections around them — `filtered_untap`'s `Vec` + `HashSet`,
  `matrix_choice`'s `HashMap`, `untap_caps`'s `flat_map` — landed.
  `do_untap`'s remaining ~34 M is ~9 M of self across every `file:function`
  entry, 5.1 M of `do_phasing`, and ~20 M in callees nobody has attributed.
  **Read the callee table first.**
* **`do_cleanup` — 27,454,488 / 1.43 %, read, and there is no easy win.**
  `finish_cleanup` is 24,937,836 of it and `check_state_based_actions` is
  **16,643,338 of that** — the CR 514.3a sweep, which is real work.
  `cleanup_wear_off` is the remaining ~5 M over 1,764 calls and its ~60
  `clear_cold!`s are already guarded.

**(-8b) `card_type_change_in_scope`, the gate's own residue — CLOSED, and
it is the third measured loss for the same device.** It reads **21,776,000 Ir
/ ~1.0 % over 10,670 calls**, and **19.3 M of that is `slice/iter/macros.rs`
+ `ptr/non_null.rs`** — walking, not deciding. So: hoist the bit into
`sba_board_scan`, which already walks the same battlefield and every card's
`static_abilities` one block earlier in the same function, and read
`scan.type_change` for free. Written, suite-green (the staleness
`debug_assert!` held over all 18,645 tests), and **+16,934,080 Ir /
+0.77 %**. Reverted.

**Three attempts, three losses, and the rule they establish.** Folding the
gate's walks into one pass: **+0.55 %**. Folding them and the death legs:
**+1.24 %**. Hoisting one of them into an existing walk: **+0.77 %**. The
third is the interesting one, because it is not "one more walk" — the walk
was already happening — so the cost has to be the *body*:
`sba_board_scan`'s per-card loop is already large (a dozen `|=`, four nested
`for`s over subtype vectors), and inlining `card_can_change_card_types` into
it slows the whole scan by more than the separate specialised `any` saved.
**A tight, specialised, short-circuiting `any` over a `Vec` is very cheap on
this codebase — cheaper than the marginal cost of adding its body to a loop
that is already big.** (-6) is the standing counter-example and its
arithmetic still holds *there*, where the four walks were over
`definition.static_abilities.is_empty()` inside a function called 18,386
times, not 10,670. Cost the body against the iteration before assuming
fusion pays; on the evidence here it usually does not.

**What is actually left at this site — and it is now less than it was.** Not
fusion, and (forty-fifth pass) **not a memo either**: the board-level
memo/epoch this paragraph asked for was built against exactly this predicate
and measured **+0.490 %** lock-free; see (-18). Four losses at one site.
The remaining shape is to drop the question (asking it less often is out —
`activate_ability_inner` asks it *once*, 18,864 calls over 18,830 activations
at the forty-sixth tip; the older "twice" note was wrong):
the gate only needs it to decide whether non-creatures join the death legs,
and a permanent that is not a printed creature contributes nothing unless
something animates it *and* the animation leaves it at ≤ 0 toughness.

**(-8) `check_state_based_actions`' death sweep — PAID, `-2.277 %`,
thirty-eighth pass.** All the numbers and the two devices are in the Log
block. What the entry taught, kept because both halves recur:
**(i) a `compute_*` site costs gather + layer pass + collect** — this one was
costed off its 0.83 % gather row and paid 2.28 %, so read the
`spec_from_iter` caller table beside the gather table before ranking the next
one. **(ii) A layer-7 presence gate has to be *signed* to be worth
anything.** The entry's own prediction — that a `pt_change_in_scope`
predicate would be a wash because every anthem is layer 7 — was correct, and
the way past it is to ask whether computed toughness can come out *below*
instance toughness, which every positive anthem answers `false` to for free.
The prediction that the sixth leg needed instrumenting before the first five
was also correct and cost one probe build: **9,228 / 10,670 sweeps (86.5 %)
are quiet on instance reads alone, and all 9,228 are quiet on the exact
gathered test too.**

**(-9) `combat_damage_computed`'s `compute_permanents` — 15,918,359 Ir /
0.71 % over 3,274 calls (4,862 each).** One gather plus a per-participant
layer pass per combat-damage step, on a `&mut self` path, so it gathers every
time. It is genuinely needed — the whole resolver reads its `computed` — but
it is taken **twice per combat** whenever the first-strike step runs, and
`has_first_strikers` (now gated, thirty-seventh pass) has just decided the same
question off the same board immediately before. Two shapes worth costing: hand
the first-strike step's computed set forward to the regular step (unsound as
written — damage is dealt in between and CR 510.2 is a fresh assignment), or
have `advance_step`'s gate and the resolver share one scope. The
`compute_battlefield` half of this entry is **paid** (`59c964dc`, -0.223 %):
the 310 calls at 16,939 Ir each were `apply_combat_decision_answer` taking a
whole-board view for a participant-scoped question, not the free-divider
fallback. **That is the second finding this pass that Ir/call did not
produce** — the device that did was asking which sites take a whole-board
view to answer a question about two to six named permanents. `view.rs` (two
sites), `bot.rs:1711`/`1752` and `eval.rs:1309` are the unread siblings.

**(-7) `scale_damage_to` — PAID, `-1.640 %`, thirty-sixth pass.** One gate
in front of the whole function; the site is off the caller table and
`scale_damage_to_inner` never runs on the `fixed` workload. What the entry
taught, kept because the shape recurs: **gate the site, not the read** — it
removed a third as many gathers as (-3) for nearly twice the Ir, because
the cost was ~8 battlefield walks and an allocation per call, not the
gather. And a gate with no shared choke point can audit against its own
outcome (`debug_assert_eq!` that the guarded body is a no-op) instead of
against an enumeration.

**What is left here, unmeasured and probably small:** `source_cp` is still
taken eagerly *inside* the body for two uses — the source's computed colours
and controller, and `source_cp.is_none()` as an "is the source a battlefield
permanent?" test. The second needs no gather (`battlefield_find(s).is_none()`
answers it); the first is a genuinely new family (layer 2 control-change,
layer 5 colour) and would need its own enumeration. Only worth doing if a
profile puts the body back on the table, which on this workload it is not.

**(-6) Four presence gates, four walks of one list — the thirty-sixth
pass's residue, and the arithmetic is measured, not guessed.** Closing (-3)
removed 18,386 gathers worth ~49 M Ir and netted 20.6 M, because the gates
that replaced them cost **~28 M Ir / ~1,540 per activation**. They are four
separate `battlefield.iter()` passes — `ability_strip_in_scope`,
`card_type_change_in_scope`, `land_type_change_in_scope` and
`card_keyword_possible`'s `keyword_grant_in_scope` — over the same list,
each mostly spent on `definition.static_abilities.is_empty()` for a board
of vanilla creatures and basic lands. **The fix is the shape
`gather_continuous_effects_inner` already uses on itself**: it folds eleven
whole-board passes into one `sa_cards` walk that sets a flag per family,
"so the emitted effect sequence is unchanged". A `PresenceBits` built in one
walk and asked four times is the same trick one level up.

Two things to settle before writing it, because they decide whether it
wins. **(a) Laziness is worth real money on this path** — a land tapping
for mana never asks the land-type gate (`printed_land_mana_basic` returns
`None` for a non-basic, and `is_mana_ability` skips the third CR 602.5
gate), so a fused walk that computes every bit eagerly gives some of the
win back; measure eager-fused against the four lazy walks rather than
assuming. **(b) `keyword_grant_in_scope` is not battlefield-only** — it also
walks command, emblems and graveyards, and it is `pred`-parameterized, so
it fuses with the other three only for the battlefield leg. Start with the
three type-family gates, which are the same walk over the same list with no
parameter.

**(-5) `evaluate_requirement_static`'s lazy layer view — CLOSED. Card-type
half paid `-1.333 %` (thirty-fifth pass) and the land-type twin shipped in
the thirty-sixth.** The `OnceCell` at `eval.rs` serves five layer-4
families: `has_type` (card types, gated), `has_ctype` (creature types),
`has_atype` (artifact subtypes), `has_ltype` (land types — the predicate now
exists as `land_type_change_in_scope`, built for (-3)) and `has_stype`
(supertypes). The three ungated ones no longer appear on the
`computed_permanent` caller table at all — the whole `eval.rs` row went to
zero — so **their traffic is not measurable on the `fixed` bench and none of
them is worth a predicate until a profile puts one back on the table.** The
one fact kept so it is not re-derived: creature types, artifact subtypes and
supertypes have **no cheap predicate** — `AddCreatureType` /
`SetCreatureTypes` alone have ~20 emitters, and `shallow_creature_types`
already spares `has_ctype` the gather on the mid-gather path.

**The device the paid half used, because the next family will want it.**
`permanents_with_abilities_removed`'s cross-check re-gathers whenever its
gate says "no", and a gate asked 15,574 times per six games cannot afford
that in a debug suite. Check the implication in the *sound* direction
instead — a `debug_assert!` in `gather_continuous_effects` that a gathered
set carrying the modification implies the gate said `true`. It runs only
where a gather already happened, and the whole suite audits it.

**(-3) `activate_ability_inner`'s gather — PAID, `-0.879 %`, thirty-sixth
pass.** All 18,386 gathers removed; the row is off the `computed_permanent`
caller table. Four legs in the end, and each is a reusable predicate:
`ability_strip_in_scope`, `card_type_change_in_scope`,
`land_type_change_in_scope` (built here) and `card_keyword_possible`. What
the entry taught, kept because (-6) is its direct descendant: **removing
100 % of a 2.09 % site netted 0.88 %**, because four battlefield walks per
activation is not free. See the Log block and (-6).

**The auto-tap scope is *not* the way in, and here is the arithmetic so it
is not re-derived.** `auto_tap_for_cost_inner -> activate_ability` is 18,832
calls over 8,892 payments (2.1 per payment), so a scope spanning the tapping
loop would fold ~52 % of these gathers — but tapping is a layer input in at
least four places inside `gather_continuous_effects_inner` (relative lines
726, 2571, 2589-2593, and every `WhileCondition`), so the scope is unsound
without a guard over all of them. Moot now that (-3) is paid with an exact
gate, and kept only so the idea is not re-proposed.

**(-2b) `apply_prevention_shields`' Absorb read — PAID, `-0.855 %` with
`damage_prevented_by_protection`, thirty-fifth pass.** Both now ask
`card_keyword_possible` before gathering. What the entry taught, kept
because the next gate will want it: the keyword-grant predicate is the
device, and **its own cost is the thing to watch** — the first version was
a wash at 1,643 Ir a call. See the Log block.

**(-4) The rest of the "eager view above a wide `match`" sweep — cheap,
mechanical, and one hit already paid -2.128 %.** *Two of the four named
below were read on the 2026-08-14 retake and are **not** hits*:
`evaluate_predicate` (eval.rs:1499) and `evaluate_value` (eval.rs:135) bind
nothing above their `match` — every layer read is inside an arm — which is
what the paragraph's own warning predicted. `evaluate_requirement_on_card`
and `resolve_selector_inner` are still unread. The thirty-third pass took
`evaluate_requirement_static`'s. The siblings are the other wide dispatchers
that bind something expensive before the arm is known: `evaluate_predicate`
(eval.rs:1499, 10 `computed_permanent` sites across arms),
`evaluate_value` (eval.rs:135, 9, incl. a `compute_battlefield`),
`evaluate_requirement_on_card` (4) and `resolve_selector_inner`
(effects/mod.rs, 4). **Read each for a `let` *above* the `match`, not for
the count** — a call inside one arm costs nothing on the other arms, and
all four of those counts are per-arm, which is why none of them is
automatically a hit. The enumeration script that produced the list is three
lines of `awk` over `fn` signatures: `&self`, no `with_frozen_layers`, two
or more layer reads. Rank by profile before writing: `evaluate_value` and
`evaluate_predicate` do not appear on the tip's caller table at all.

**(-2) `CardCold` — PAID, `5174acd3`, -1.969 %.** What is left of the
entry, so the next run does not re-derive it. The `CardData` deep-copy
table (`--tree=caller` on `Arc::make_mut`, rows whose *file* is
`crabomination_base/src/card.rs`) reads **~44.5 M / 1.76 %** at the tip,
and its largest row is `activate_ability_inner` **15,346,614 over 18,312 =
838 Ir each**. 838 is the ~126-field struct memcpy plus the collections
that stayed hot on purpose, i.e. **there is no second `CardCold`** — the
tenth pass's rule (*group size x unshare probability < sum of the
individual clone costs*) is what stopped the group at 22, and the fields
deliberately left hot are `damaged_by_this_turn`,
`damage_by_source_this_turn`, `damage_by_source_name_this_turn`,
`blocked_attackers_this_turn`, `damaged_players_this_game`,
`damaged_permanents_this_game`, `granted_keywords_eot`(`_ts`), `counters`,
`keyword_counters` and the five `Option<Arc<CardDefinition>>` faces. The
next thing on this shape is **cost per clone, not clone count**: a
`CowBox` on one individual hot-but-large field, when its clone count far
exceeds its write count. Cost the ratio before writing it.

**(-1a) The `iter_mut().find()` half of the unshare sweep — the syntactic
filter misses it.** The thirty-first pass's sweep matched
`for \w+ in &mut self\.` and `\.iter_mut\(\)` over a *zone*; a
`self.battlefield.iter_mut().find(|c| c.id == id)` unshares the zone and
then deep-copies the found card, and reads as an ordinary lookup. One was
taken at `5174acd3` (the `pending_etb_counters` drain, once per permanent
entering). **54 more sites match
`\.(battlefield|hand|graveyard|library|exile|command)\.iter_mut\(\)` under
`game/effects/` and `server/`** and were not audited; nearly all sit in a
`run_effect` arm where the write is genuine, so this wants the profile's
call counts rather than a syntactic pass — an arm that runs on a bench deck
*and* writes conditionally is the shape. Read the `make_mut` caller table
first; anything not on it is not worth touching.


**(-1) The CoW-unshare sweep — re-run it, but with the new recipe.** The
twenty-eighth pass took four of these for -7.257 %, the twenty-ninth two
more for -1.993 %, and the thirty-first two more for -2.466 %. The
`=> …deref_mut` sort that found the first four no longer resolves (see the
twenty-ninth pass's Log block): `Player::deref_mut` is inlined now, and the
40 surviving `deref_mut` call lines are 0.12 % between them. **Two sorts
still work: `--tree=caller` on `alloc::sync::Arc<T,A>::make_mut`** (7.50 %
inclusive at the tip), **and — the one that found the thirty-first pass's
largest row — `--tree=caller` on `alloc::sync::Arc<T,A>::clone_from_ref_in`,
reading the caller names for a function that has no business deep-copying
anything.** The second is the better first look: `make_mut`'s table is
dominated by writes that are genuinely needed, while a `clone_from_ref_in`
caller at four figures of calls is usually a sweep writing values the fields
already hold (`finish_cleanup`, 2.62 %, gone). **Read either table by the
*inlining* function, not the leaf**: every `<` row whose file is
`alloc/src/sync.rs` and whose function is an engine function names a
function that deep-copies a CoW handle, with the call count beside it —
~1,900 Ir a call is `CardData`, ~900 is `PlayerData`, ~30 means the handle
was already unique and there is nothing to take. **The standing filter is a
`for … in &mut` over a zone in a function that runs on a turn or step
boundary**, and the syntactic sweep of those is clean under `game/` as of
`62e6dd42`; `server/` and `effects/` were never swept the same way. What is
already known and does not need re-deriving:

* **`remove_from_hand` is not a candidate**, nor is `attacked_this_turn`,
  nor is hoisting the zones out of `PlayerData`. The unshare is inherited by
  whatever the action writes next. *Only a seat's **sole** write in an
  action is removable* — the twenty-ninth pass's Log block has the
  arithmetic.
* What is left on `PlayerData` is **cost per clone**. `PlayerCold` took the
  fifteen heap-owning rare fields; the remaining per-clone cost is the
  struct memcpy (~110 scalars) plus the collections that are genuinely hot
  — `spell_casts_this_turn` (3,330,696 Ir over 24,852 clones, i.e. often
  non-empty), `spells_cast_by_name_this_game`, `graveyard_ids_this_turn`,
  `creatures_entered_this_turn`. Each is written on the same action that
  already unshared the seat, so a *second* CoW group would not pay — a
  `CowBox` on the individual field might, when clone count (24,852) far
  exceeds write count (~7,000 for the cast-path ones). Cost the ratio first.
* The new `PlayerCold` group unshares **1,944 times at 891 Ir**; the
  per-turn reset loops guard their `clear()`s on `is_empty()` to keep it
  there. **Any new `clear()` on a cold collection in those loops needs the
  same guard**, or it hands the row back.
* Genuine writes already costed, listed so the next sweep skips them:
  `pay_for_spell` 4,768,455 / x8,892, `pending_creature_etb` 930,435.

**The `compute_battlefield` table is closed.** Every site the nineteenth
and twentieth passes ranked is paid: `declare_attackers_banded`
(`31116d43`), `declare_blockers` x2 (`42f59829`, `911cf298`),
`resolve_combat` + `resolve_first_strike_damage` (`4f3e86c0`),
`do_phasing` / `do_untap` / `process_cumulative_upkeep` (`ed4c152c`).
**Calls 17,718 -> 5,488 -> 310**, and the 310 are all `submit_decision`
(741,960 Ir / 0.02 %). Do not re-open this table; the layer system's
remaining cost is per-card `computed_permanent` and the gather itself.

**Four rows costed on the thirty-first pass's base and not taken**, listed
here rather than as numbered entries because each is one measurement short
of being actionable. All from `a58447d9`, `--tree=caller` /
`--tree=calling` / `--auto=yes` on the fixed six-game workload:

* **Auto-tap's activations, `auto_tap_for_cost_inner` -> `activate_ability`
  253,177,209 / 9.12 % over 18,832 calls (13,444 Ir each)** — the single
  largest callee edge outside the search. `try_pay_after_snapshot_mode` is
  14.84 % inclusive and `auto_tap_for_cost_inner` 14.16 %, i.e. **paying for
  a spell is a seventh of the program** and ~2.1 land taps per payment.
  Inside each activation, `activate_ability_inner`'s single
  `with_frozen_layers` costs **49,055,654 / 1.77 % over 18,386 calls (2,668
  Ir each — a gather)**, and it exists to answer three questions:
  `lost_all_abilities`, is-a-creature, and `land_mana_lost`. **All three
  already have presence gates** (`ability_strip_in_scope`, the layer-4
  card-type gate `compute_battlefield_creatures` documents,
  `rewrites_land_types`) — but every gate tests the *gathered* set, so
  inside a scope it creates they cost exactly what they save. The lever, if
  there is one, is a scope spanning `auto_tap_for_cost_inner`'s tapping
  loop, and it is **unsound as written**: tapping a permanent is a layer
  input (`UntapOnlyChosenTypeWhileUntapped` and any `WhileCondition` keyed
  on an untapped permanent read `!c.tapped`). Cost how many of the 18,832
  taps happen on a board carrying either before designing anything.
* **`dying_snapshot`, 10,815,151 Ir over 3,420 calls (3,162 each — a
  gather)**, one per death from the SBA sweep and four sacrifice paths. Its
  `computed_permanent` exists only to notice that the computed creature
  types differ from the printed ones, which needs a *live type-changer*;
  `compute_permanent` already computes `has_type_changer` from
  `Modification::SetCreatureTypes | AddCreatureType`. Same gate-needs-the-
  gather circularity as the row above, and the callers are `&mut self`, so
  a scope cannot wrap them.
* **`printed_color_set`, 17,572,024 / 0.63 % over 237,652 calls (74 Ir
  each)** — one per `compute_permanent_pass`, and a pure function of the
  `CardDefinition`. Not memoizable *on* the definition (candidate (13)'s
  reasoning: `Arc::make_mut(&mut card.definition)` mutates in place), so it
  is a construction-time field or nothing — i.e. it belongs to the same
  `CardDefinition` rework as the `Keyword` bitset, not to its own row.
* **`compute_permanent_pass` is 134,165,234 / 4.83 % over 237,652 cards
  (565 Ir each)** and its own annotated lines are all under 0.25 %; the cost
  is spread across four `Printed::new` constructions, `base_power` /
  `base_toughness` (3,341,698 each, attributed to `raw_vec`), and
  `affected_includes_gated` (4,086,736 over 91,256). **There is no single
  line to take here** — record it so the next run doesn't re-annotate
  layers.rs hoping for one.

0. **`pick_attacks_scored`, 1,475,649,076 / 51.05 % over 630 calls — half
   the program, and *not* a pruning item.** The search itself, not its
   inner loop, which every pass since the seventeenth has made cheaper.
   **This is a bot-quality question as much as a perf one** — a narrower
   search is a different player, so it needs a `bot_ladder` win-rate gate,
   not an Ir number.

   *Costed and probed 2026-08-12; do not re-run either.* The 630
   declarations run **1,166 simulations — 1.85 per search, 1.33 M Ir each
   (0.0425 % of the program per simulation)**, so on this workload nearly
   every scored declaration is the binary *swing with the one creature or
   don't*. The pruning probe (a counter on `choose_scored`'s returned
   index, `--decks all`, 10,200 games, 110,000 searches) reads **greedy
   54.0 % / the empty declaration 35.0 % / a greedy-minus-one 11.0 %**:
   the search departs from greedy **46 %** of the time and every candidate
   class pays for itself. **So the only lever on this entry is making one
   simulation cheaper, which is what the rest of this list does.** Note
   `--decks fixed` under-represents the search by ~1.8x (3.37 simulations
   per search on `all` against 1.85 on `fixed`) — the fixed archetypes
   rarely present a multi-attacker board. Re-run the probe only if the
   bot's declaration policy changes.
1. **`would_accept`, 491,192,545 / 15.34 %** — the affordance probe, one
   `GameState::clone` + one `perform_action_inner` per candidate action.
   What is unexamined is the *probe count*, i.e. how many candidates the
   bot dry-runs that no scoring pass could have chosen.
2. **The `trigger_grant_sources` scan itself — costed 2026-08-11 and
   deliberately not taken.** The peel is not the cost: `active_static`
   returns on its *first* `match` arm for any non-wrapper effect, and a
   card with no `static_abilities` runs an empty inner loop, so the ~480 Ir
   per call is ~24 Ir per card of iterator overhead over ~20 cards. A
   syntactic peel saves the `EffectContext` only on the rare
   `WhileCondition` static; `resolve_named_by_source` only runs on a hit.
   **The only lever with real headroom is a cached board-level flag**
   ("does any static source carry a `GrantTriggeredAbility`"), and that
   needs invalidation at every battlefield mutation — `sa_cards` is a
   *local* built inside the gather, not a state field, so it cannot be
   borrowed here. Cost the maintenance before writing it. Original
   measurement, kept: 53 M / 1.57 %
   over 110,540 calls at the twenty-second pass's base; `1112e709` took the
   28,564 that were per-*kind*, leaving ~89 k that are genuinely one per
   batch (the dispatcher 52,332, `fire_step_triggers` 14,462,
   `fire_combat_damage_triggers` 7,118, `resolve_top_of_stack` 7,046,
   `cast_spell` 14,092 …). ~480 Ir each for a walk of `all_static_sources()`
   x `static_abilities` calling `active_static` per ability. **The lever
   left is making the scan cheaper, not calling it less**: the same
   presence-gate device on `permanents_with_abilities_removed` — a
   syntactic peel of the duration/predicate wrappers with no
   `EffectContext` and no `resolve_named_by_source` — or a board-level
   flag. Cost it against the walk's ~24 Ir per card before writing it.
   **Since the twenty-fourth pass three of the scan's callers — the
   dispatcher's 52,332, `cast_spell`'s 14,092 and `fire_step_triggers`'
   14,462 — run inside `dispatch_board_scan` or behind its presence gate,
   so a cached board flag now has one place to be read and three to be
   maintained.**
3. **The `RawTable::clone` block — the `#[serde(skip)]` half is now paid;
   what is left is behind serde.** `--tree=caller` on `RawTable::clone` has
   exactly two callers, and `271c7d14` took the larger one's set half.
   `62e6dd42` took the five `GameState` fields that carry `#[serde(skip)]`
   — `combat_damage_order`, `combat_damage_assignment`,
   `names_this_resolution`, `leaves_bf_lki`,
   `players_sacrificed_this_resolution` — for **-0.211 %** across a span
   that also carried two no-Ir commits, against 268,816 table clones under
   `GameState::clone` at the time. Left: the **seven `ColdState`
   `HashMap` fields** (same CoW unshare, ~44.8 k times for six games) and
   `block_map` plus the two per-player-discard maps. **The gate on all of
   them is serde**: a `HashMap` serializes as a JSON object and a `Vec`
   newtype as an array of pairs, so any field that reaches a snapshot needs
   a custom impl or a format bump. Check the field's serde attribute before
   costing it — that check is the whole of what made the five cheap.
4. **PAID, both legs** — `fire_combat_damage_triggers` bucketing its five
   kind-independent walks (`08cbc9c3`, -1.051 %, calls 28,564 -> 7,118) and
   `fire_step_triggers` no longer cloning every printed `TriggeredAbility`
   before filtering on kind (`006d5966`, -1.885 %). Nothing left at either
   site. **What survives is the shape**: *an iterator that clones and then
   narrows*. The syntactic sweep is clean workspace-wide; the semantic
   question — how much of what a loop builds does it keep — is still
   unswept over the hot trigger/candidate builders
   (`push_ordered_trigger_candidates`, `statics_granted_triggers_with`,
   `cast_candidates`' `flat_map`).
5. **`pick_by_outcome`, 210,019,714 / 6.56 %, essentially all of it one
   `collect()` in `bot.rs`.** Never profiled at line level. Read
   `--auto=yes` on it before guessing; the eighteenth pass's candidate (1)
   is the warning that the cost is the *iterator body*, not the container.
   Here it is `evaluate_action_outcome` per finalist — a clone and a
   resolution each — so the container is certainly not the cost, and the
   real question is the *finalist count*, which is a bot-quality question
   like (0) and (1).
6. **`dispatch_triggers_for_events`, now 208,322,657 / 7.21 % over 52,332
   calls** (237,986,906 / 7.64 % before `e02767aa` took the per-permanent
   `Vec`). **The remaining self cost is iteration, not allocation** — the
   `alloc::vec` + `alloc::raw_vec` block that was 69 M is spent, so the
   next attempt here is the (b) below or nothing. Note also that
   `event_matches_spec` runs only **63,846 times for 52,332 dispatches**
   (1.22 per dispatch, 30 Ir each): the battlefield x trigger x event loop
   is *not* the cost and an event-kind presence mask over it would gate
   almost nothing. Historical numbers, kept: `c7bdd850` took the four cheapest blocks, `b925063c` the
   delayed-trigger collects, and `f28faaa0` (lever a', generalized) fused
   the four board walks at the top of the function into one
   `dispatch_board_scan`. Its collect row is **94,608 collects /
   14,001,655 / 0.44 %**, down from 146,940 / 22,068,557 / 0.66 %. What is
   left, in order: (b) a `u32` presence mask over the batch's event kinds
   filled in one pass, with each block gated on its bit — the
   `gated_block!` device from `52f4311a`, `debug_assertions` audit
   included; and `push_ordered_trigger_candidates` (~1.1 %, exactly one
   per dispatch). The `synthesized` chain is **not** worth a gate on its
   own any more — `Vec::from_iter` on an empty iterator is `Vec::new()`,
   so the empty case is one `next()` over two empty vectors. **The
   per-card `all_triggers` Vec is *not* a target** — 33,980 Ir over the
   whole run, checked 2026-08-11.
7. **`mana_source_table`, 116,895,124 / 3.65 % over 7,370 calls** — down
   from 8,892 since the gate (`1ec589d1`). What is left is probe (b),
   untried: the table is a pure function of (battlefield tap state, layer
   inputs, `creature_only`), so a per-freeze-scope memo may be the
   `computed_permanent` trick again. **The surviving calls are exactly the
   ones that go on to tap**, which is what invalidates the memo, so cost
   the hit rate before writing it. `untapped_producers_of` — five frozen
   board scans where one would do — **does not appear on this profile at
   all**; the hybrid path is cold on the bench decks, so leave it.
8. **The collect table, re-measured on the twenty-fourth pass's tip** —
   `--tree=caller` on `Vec::from_iter`, inclusive, so these are *iterator
   body* costs (see the eighteenth pass's correction below, which is still
   the warning that matters):
   `cast_candidates` **168,130,962 / 5.25 %** over 7,024 calls — **broken
   down 2026-08-11**, see below;
   `check_state_based_actions` **132,514,326 / 4.14 %** over 82,634;
   `mana_source_table` **108,286,211 / 3.38 %** over 7,370;
   `pick_attacks_inner` 0.67 % over 7,842;
   `pick_removal_ping` 0.67 % over 37,710;
   `dispatch_triggers_for_events` 0.44 % over 94,608.
   **`fire_step_triggers` is off this table** (`006d5966`); `empty_mana_pools`
   was paid at `ef731ecc`. Only three rows are over 3 %, and the top two are
   the ones whose iterator body is real work rather than container overhead
   — `cast_candidates` is 23,900 Ir per call of auto-targeting per hand
   candidate, `check_state_based_actions` is `compute_battlefield_creatures`
   plus the `dead` filter. Read `--auto=yes` on either before costing it.

   **The SBA row, denominated at the thirty-second pass's tip — and the
   denominator is the whole finding.** `check_state_based_actions` is
   **10,670 calls / 151,182,228 Ir inclusive / 5.98 %**, i.e. ~14,170 Ir a
   pass. Three separate `--tree=caller` rows read exactly **10,670**: the
   gather (18,599,381), its own `stack.rs` self (23,915,320) and a
   `game/mod.rs` row (469,480). **So a pass takes exactly one gather and
   there is nothing to fold inside one** — the two layer sites in the
   function body (the CR 603.8 flip `compute_battlefield`, the CR 704.5j
   legend-rule `computed_permanent`) are both behind `sba_board_scan` flags
   that are false on every bench board. The `spec_from_iter` collect row is
   **82,634 over the same 10,670 passes = 7.74 collects a pass**,
   129,943,199 / 5.14 %, i.e. **1,572 Ir a collect and 12,178 Ir a pass**.
   The lever is therefore *fewer passes* or *a cheaper per-pass body*, and
   **not** deduplicating gathers. Do not re-derive this.
   **`cast_candidates`, broken down** (`--auto=yes` + `--tree=calling`,
   twenty-fourth pass's tip). Its *own* self cost is under 0.2 % across six
   file rows — the 5.62 % inclusive is callees inlined into the
   `Vec::from_iter` frame, so read the annotated source, not the function
   list. Two callers, 3,382 and 3,642. By callee:
   * `can_afford_in_state` — **RE-OPENED AND PAID at the seventy-sixth pass
     (`fixed` -0.613 %, `sos` -0.425 %, `cube` -0.577 %), and the reason the
     2026-08-12 refutation below stopped applying is arithmetic, not
     judgement.** That refutation is correct for what it measured — an
     *eager* fused scan, four walks worth **0.29 %**, **1.13 cards per
     sweep**. At the seventy-fifth tip the three surviving walks are
     **1.14 % of `cube`** over **30,350** calls against 12,114, and **2.80**
     cards reach the filter per sweep that reaches it at all. What shipped is
     not the fused scan either: `CostStaticSources` drops the sources whose
     `static_abilities` is empty (**~3 in 4** of these boards' permanents — see
     the Baseline's edge table) and hands the list to
     `_over` forms of the three functions, so there is no enumeration and no
     gate. **The transferable half of the refutation still holds and the fix
     obeys it — the list is lazy**, because an eager read on
     `pick_combat_trick`'s empty sweeps cost +0.35 % at pass 40. **What is
     left is 0.52 % of `cube`** in walks over the sources that *do* carry
     statics; a `cast_cost_scan`-style bitmask would take most of it and
     needs a ~30-variant enumeration with the `debug_assert!`-at-the-site
     device. See the seventy-sixth pass's Baseline block.
     **The 2026-08-12 reading, kept verbatim.** The
     fused-scan fix this entry prescribed measured **+0.066 %** and was
     reverted; see the twenty-seventh pass's Log block for why (1.13 cards
     per sweep, not 1.72 — the filter is not in `cast_candidates`). What is
     left of the item after measurement: the four static walks are **0.29 %**
     between them, `available_mana` is **1.14 %** and was 60 % of the
     function, and the part of `available_mana` that was real cost —
     `granted_abilities_with`'s redundant `battlefield_find` — is **paid**
     (`granted_abilities_of`, -0.552 %). The original
     measurement, kept because the arithmetic below is still the warning:
     **56,500,015 / 1.76 % over 12,114 calls (4,664 Ir each)**.
     Its body per hand card: `extra_cost_for_card_in_hand`,
     `cost_reduction_for_spell`, **a `ManaCost` clone** to append
     `colored_spell_tax_for_spell` (empty on every bench card),
     `relax_cost_colors`, and `available_mana(state, seat)`. **The lever is
     not the clone and not a hoist — it is that these are five separate
     whole-battlefield walks per call**, four of them `battlefield x
     static_abilities` (`extra_cost_for_spell`, `cost_reduction_for_spell`,
     `colored_spell_tax_for_spell`, `relax_cost_colors`) plus
     `available_mana`'s producer scan. Same shape, and the same fix, as the
     `DispatchScan` row that paid -1.118 %: one pass filling a small struct.
     **Two levers that do *not* pay, by arithmetic, so they need no A/B:**
     the `ManaCost` clone is ~12 k allocations, i.e. under 0.1 %; and
     hoisting `available_mana` out of the filter cannot pay either —
     12,114 calls over 7,024 `cast_candidates` calls is **1.72 per call**,
     so at most 42 % of one walk is duplicated. That is also the correct
     reading of the fourth pass's no-win on exactly that hoist: the
     asymptotics were real, the *filter* just never reaches seven cards.
   * `affordance_probe_template` **31,110,605 / 0.97 % over 3,382**
     (9,199 each) — one library-stripped `GameState` clone per tick.
   * `pick_land_to_play` 26,629,385 / 0.83 % over 1,410;
     `beneficial_aura_host` 3,392,725 over 606; `Vec::retain` 0.09 %.
   The sibling rows at the same call site are `would_accept`
   **168,728,436 / 5.27 % over 1,534** (the lazy final gate, 110 k Ir per
   probe) and `pick_by_outcome` 210,019,714 / 6.56 % over 908 — both
   candidate (1)/(5), i.e. probe-count questions.

   The allocator block re-reads **~17.0 %** on this base (`_int_malloc`
   5.34 / `_int_free` 4.38 / `malloc` 3.23 / `free` 1.96 / merge 0.88 /
   arena 0.77 / consolidate 0.74 / unlink 0.73),
   `__memcpy_avx_unaligned_erms` 3.34 %, `Arc::clone_from_ref_in` **16.09 %
   inclusive** / ~5.4 % self over ~1.6 M CoW unshares,
   `gather_continuous_effects_inner` 2.74 % self / 9.74 % inclusive.
9. **The gathers `computed_permanent` still takes, by caller — the top
   candidate now.** PERF's own rule — divide each caller's inclusive Ir by
   its call count, >2,000 means it is outside a freeze scope — names five,
   all in the damage path, re-read on the twenty-fourth pass's tip:
   `scale_damage_to` 49,405,906 / 14,624 (**3,378**),
   `damage_prevented_by_protection` 39,667,284 / 18,986 (**2,089**),
   `permanent_has_keyword` 21,973,585 / 8,328 (**2,638**),
   `damage_from_source_prevented_by_keyword` 15,610,446 / 4,450 (**3,508**),
   `dying_snapshot` 11,819,145 / 3,420 (**3,456**) — **138.5 M / 4.33 %
   between them**, and `computed_permanent`'s gathers are 7.60 % of the
   program. **What was checked on the twenty-fourth pass and is now known:
   every one of the five already takes its own `with_frozen_layers` scope
   internally** (`damage_prevented_by_protection` since `96129f68`,
   `scale_damage_to` since the eleventh pass), so each per-call cost is
   *one* gather plus its work — there is no intra-call duplicate left to
   remove. **The only lever left is a scope spanning several calls**, and
   the blocker is real: they sit under `resolve_combat`'s damage loop,
   which writes `dealt_damage_this_turn` / `damaged_by_this_turn` and
   **changes life totals via lifelink between the reads**, and a
   `WhileCondition` static can read life (Serra Ascendant), so a blanket
   freeze over the loop is unsound. Two tractable shapes, in order:
   (a) the strictly-`&self` prefix of one (attacker, blocker) iteration —
   `damage_prevented_by_protection`, the three per-source prevention
   checks, and `scale_damage_to` all run before `apply_prevention_shields`
   takes `&mut self`, so one scope over that prefix folds ~2 gathers into
   1 and is sound by construction; (b) note that
   `resolve_combat_damage_with_filter` **already holds `computed:
   &[ComputedPermanent]` for the whole step** and the five helpers
   re-derive from scratch what it contains — a `_with(computed)` variant
   would be nearly free, but it pins the layer view to the step's start,
   which is arguably *more* CR 510.2-correct and is still a behaviour
   change. Golden traces decide (b); (a) needed nothing and **is PAID**
   (`56f6623f`, -0.740 %) — three scopes, one per prefix. (b) is all that
   is left *of the damage path*, and it is a behaviour question, not a perf
   one. **But shape (a) turned out not to be a damage-path item at all** —
   see (10): the same wrapping paid again immediately, on a function with
   nothing to do with combat.

   *Re-read at the thirty-second pass's tip `5174acd3`, and the entry has
   shrunk under the passes that took the combat scopes.* The gather is
   **118,978 calls / ~228 M inclusive / 9.04 %** by caller:
   `computed_permanent` **85,808** (169,154,302 / 6.69 %),
   `frozen_effects` 18,916, `check_state_based_actions` 10,670,
   `compute_permanents` 3,274, `compute_battlefield` 310. And
   `computed_permanent`'s own callers now read *under* the 2,000 threshold:
   `damage_prevented_by_protection` 28,907,705 / 18,986 (**1,523**),
   `scale_damage_to` 25,846,331 / 14,624 (**1,767**),
   `permanent_has_keyword` 10,889,788 / 8,328 (**1,308**),
   `dying_snapshot` 10,739,232 / 3,420 (**3,140**),
   `blocker_can_block_attacker` 4,415,123 / 15,368 (**287** — in a scope).
   **The threshold moved because the gather got cheaper, not because the
   callers got scoped**; 2,000 was calibrated when a gather was ~2,000 Ir
   and it is now ~1,900 with a memo hit near zero, so rank by *total*, not
   by the ratio.

   *And the obvious remaining site was costed and is not one.*
   `deal_damage_to_from` — the noncombat damage funnel in
   `effects/movement.rs` — runs ~14 `&self` layer reads before its first
   write (`computed_permanent`, `damage_prevented_by_protection`,
   `damage_sealed_by_aura`, the three per-source prevention checks, the
   Iroas/Glacial-Chasm/`player_protection_card_types` trio, the four
   `permanent_prevents_*` shields, `scale_damage_to`) and is the one
   damage path the combat passes never scoped. It **does not appear at all
   in `--inclusive=yes --threshold=99`**, i.e. it is under 0.5 % on the
   bench decks, which are creature-combat archetypes. The refactor is real
   but must not be priced off the noncombat funnel: the prefix is broken
   into two `&self`-only runs by three `&mut self` redirect branches
   (Martyrs of Korlis, Reverberation, Harsh Judgment), so it is two
   `with_frozen_layers` closures, not one. **Do it only if a wider deck
   pool moves it onto the profile.**

   *The memo itself is healthy — checked, not assumed.* Inside a scope
   `computed_permanent` hits `st.perms` by id and returns a refcount bump,
   and a miss reuses `st.memo`'s gather; the first computed read of a scope
   fills both under one lock. So a caller inside a scope pays
   `apply_layers_one` once per *card*, not per call. Do not go looking for
   a broken memo.

10. **The rest of the "reads the layer system twice" family — the cheapest
    item on this list, and mechanical.** Candidate (9) framed shape (a) as
    a damage-loop fix; `5d4b5402` shows it is a *shape*, not a site. Any
    `&self` function that reads the layer system more than once is one
    `with_frozen_layers` away from paying, the wrapping is mechanical, and
    **it cannot change behaviour because the closure cannot mutate** — so a
    wrong guess costs a rebuild, not a bug, which makes this the one
    candidate worth attacking by enumeration rather than by profile.
    `check_target_legality_with_source` was worth -0.516 % and took ten
    minutes.

    *The search*, and the part candidate (9) got wrong: PERF's rule is
    `--tree=caller` on `computed_permanent`, inclusive Ir over call count,
    >2,000 means it is gathering — but applied to the **leaf helpers** it
    finds only sites where one gather is already the floor. Apply it to
    **their callers**, which is where a second read can exist. On the
    twenty-fifth pass's tip the two damage leaves that are still gathering,
    `damage_from_source_prevented_by_keyword` (3,507 Ir over 4,450 calls)
    and `dying_snapshot` (3,458 over 3,420), each take exactly *one*
    `computed_permanent` and so have nothing to fold — the win is in
    whoever calls them, or in the `&self` functions that call several
    different leaves. `blocker_can_block_attacker` at 286 Ir/call over
    15,368 calls is what a caller already inside someone's scope looks
    like; imitate its caller.

11a. **The zone-walk family — the ordering half is CLOSED (`76d31eb8`,
    `1f68d1b0`, merged).** Five hand-written walkers ask "where is this
    card": `find_card_anywhere` (paid, -0.583 %), `find_card_anywhere_mut`,
    `find_card_zone`, `find_card_owner` and `death_was_replaced`. All are
    libraries-last now; the walkers that scan a *push-only* zone (graveyard,
    exile) do it back to front, so a card that just moved there hits on the
    first comparison, and `death_was_replaced` — a disjunction, not a search
    — answers the unreplaced case off the graveyards without touching a
    library at all. `find_card_owner` also stopped walking exile twice for
    one id and looks at the stack before the seats. **No Ir claimed** on any
    of it: none of the three reordered walkers is above the 99 % threshold.
    `find_card_zone`'s doc claimed the stack among its zones and its body
    never looked there; corrected to match the body. Do not re-open the
    ordering.

    **What is left, and it is the only part of this entry still open.**
    `find_card_anywhere` is still **19.6 M / 0.70 % over 104,240 calls**
    after the reorder, and the residual is calls that return `None` — they
    walk every zone by construction, whatever the order. Costing that needs
    a miss counter (one instrumented `profiling-fast` build, a two-minute
    run) *before* anyone designs an id→zone index, which is the only fix and
    is state to maintain on every zone move. **And the general filter this
    family taught is worth more than the family**: a `for`/`find` whose
    *first* branch is the rarest case is cheaper to check than to profile —
    neither of the thirtieth pass's two large rows was visible as an
    expensive function.
11. **A helper that opens with `battlefield_find` and is called from a
    battlefield loop — the generalization of the twenty-seventh pass's
    row.** `granted_abilities_with(card_id, scan)` was -0.552 % on its own
    because three of its callers already held the `&CardInstance` and it
    re-scanned the battlefield to recover it; the thirtieth pass took
    `effective_mana_abilities` the same way for **-0.158 %** (54,570 calls,
    three callers, all inside `battlefield.iter()`). **Two hits paid, the
    enumeration is not finished** — the remaining named sites are
    `colors_produced_by`, `creature_prevents_combat_damage_grows`,
    `creature_redirects_damage_to_controller`, `source_power_lki`,
    `combat_damage_sealed_for_your_creatures`; none of them appears on the
    current profile, so read the caller before costing one. *The search*: grep the engine
    for `fn .*card_id: CardId` bodies whose first statement is
    `battlefield_find` / `find_card_anywhere`, then read each caller and ask
    whether it is inside a `battlefield.iter()`. The fix is mechanical (an
    `_of(&CardInstance)` twin, the `CardId` form kept as a find plus
    delegate for the off-battlefield fallback) and **cannot change
    behaviour** — the twin is the same body with the lookup removed — so a
    wrong guess costs a rebuild, not a bug. Same property that made
    candidate (10) worth attacking by enumeration.
13. **The gather's battlefield pre-scan, and the keyword loop inside it.**
    Newly costed 2026-08-14, `--auto=yes` on the twenty-ninth pass's base.
    `gather_continuous_effects_inner` opens with one walk of the battlefield
    filling `sa_cards` and eleven `any_*` flags — the fold that replaced
    eleven walks and is still the right shape. What it costs on the mod.rs
    attribution alone: **`for kw in &def.keywords { match kw { … } }`
    6,420,524 / 0.222 %**, `static_abilities.is_empty()` 5,113,186 /
    0.177 %, `attached_to.is_some()` 2,492,666 / 0.086 %, over 127,878
    gathers x ~20 cards. Five of the eleven flags are a **pure function of
    `CardDefinition`**, so the loop re-derives per gather what is fixed for
    the life of the card. The lever is candidate 5's keyword bitset, not a
    memo on the flags: item 1's reasoning applies —
    `Arc::make_mut(&mut card.definition)` mutates a uniquely owned
    definition in place, so a cache *on* a definition is unsound, and it has
    to be a representation the definition carries from construction.

    **Costed 2026-08-14 and deliberately ranked down; do not re-derive.**
    The bitset's whole addressable total is **~0.6 %** and its cost is a
    catalog-wide representation change. `Keyword::eq` is 14,393,938 Ir /
    0.51 % self, but that is **~1.3 M calls at 11 Ir each spread over thirty
    call sites** — `printed_color_set` 209,154, `CardInstance::has_keyword`
    170,112, `board_keyword_in_scope` 135,886, `can_block_attacker_computed`
    115,502, `declare_attackers_banded` 80,056, then a long tail — so there
    is no single site to fix and the win is the 11 Ir, not the call. The
    pre-scan loop is 0.23 % and *its* half was taken a cheaper way (the
    thirtieth pass's `sa_cards` row, -1.250 %, left the pre-scan alone).
    `permanent_has_keyword`'s cost is `computed_permanent`, not the
    `contains`. Against that: **7,716 `keywords: vec![…]` literals in the
    catalog**, plus ~10 runtime sites that push a keyword onto a definition
    and would have to maintain the mask or silently desync it. Worth doing
    only as part of a `CardDefinition` construction rework, not on its own.
12. **`grant_scan`, 20,282,413 / 0.65 % over ~26,000 calls (~780 Ir each)**
    — `available_mana` 12,702, `mana_source_table` 7,370, the three
    `pick_removal_*` ~2,080 each. It walks `battlefield x static_abilities`
    through `active_static` and builds four `Vec`s that are empty on every
    bench board. Same shape as candidate (2) and the same blocker: a cached
    board flag needs invalidation at every battlefield mutation. The cheaper
    half is the caller side — `available_mana` and `mana_source_table` are
    both called from sweeps that could share one scan — but read the
    twenty-seventh pass's negative result first and **count the sweeps, not
    the calls**, before costing it.

**The profile of record was retaken at `5034eb2f`** (2026-08-13, the
twenty-eighth pass's tip). Only the top table was re-derived; every count,
per-caller list and share inside the candidates above is from the
`f814a13b` retake and reads ~7 % high in absolute Ir. Re-read a candidate
on the current tip before costing it, not before ranking it — the ordering
did not move.

Methodological notes, each learned the hard way:

- **Ir over-weights allocation and representation changes.** Callgrind runs
  the *system* allocator (valgrind replaces malloc), so a row that removes
  allocations reads far larger there than it ships at `release` + mimalloc:
  the `Printed<T>` row measured **-17.09 % Ir**, **+13.5 %** wall-clock on
  the system allocator, and **+1.7 %** at `release` with mimalloc. Ir tells
  you whether a change cut work and by how much; only a `release` run tells
  you what ships. For anything allocator- or cache-shaped, do both before
  quoting a throughput number. Hoisted here from the frozen candidate
  snapshots so it is not re-derived.
- **The shape that paid the eleventh pass: a `&mut self` path taking a
  `computed_permanent` outside a freeze scope.** Each one re-gathers *every
  continuous effect in the game* (~2,900 Ir then, ~2,100 now) to answer one
  question about one card, and they cluster: `do_untap` asked twice per
  permanent, `activate_ability` + `_inner` four times per activation,
  `scale_damage_to` twice per call. Four rows, **-5.74 % between them**,
  gathers from `computed_permanent` 260,370 → 151,776. **How to find the
  next one: `--tree=caller` on `computed_permanent`, divide each caller's
  inclusive Ir by its call count.** >2,000 per call means unfrozen — it is
  gathering, and deduplicating it is worth a gather each. A few hundred
  means the caller is already inside a scope and only memo *hits* are left;
  deduplicating those is worth ~200 Ir each, which is how the land-mana row
  got costed at 2 % and returned 0.26 %.
- **Check whether LLVM already hoists the scan — but check it on *your*
  build.** Three per-card `any()` walks of the effect set inside
  `apply_layers`' whole-board loop read **-0.007 %** when hoisted by hand on
  the tenth pass's base (self cost 31,937,700 — already hoisted) and
  **-0.63 %** on the branch point's base (self cost 74,510,172 — not
  hoisted). Same source, same profile, two codegen outcomes. The tell is a
  function whose *self* cost is tiny even though the source reads O(n x m);
  read the tell, don't assume the answer. The hoist is not in the tree.
- **Build-to-build noise on this profile is ~0.02 %.** Two links of
  `87d76144` read 6,150,469,969 and 6,151,471,423 (and a third, from a
  different session on a different container, 6,151,455,670). Nothing under
  ~0.1 % should be claimed from a single A/B pair.
- **Read the caller rows under `malloc`, not the function totals.** The
  eleventh pass's `do_untap` row (4.33 % of the program) never appears in a
  self-cost list at all — its cost is entirely in callees, and what named it
  in the second session was a `spec_from_iter.rs:do_untap` row sitting third
  in `--tree=caller` under `__rust_alloc`. The self-cost list is a list of
  *leaves*; the work is in the callers.
- **A CoW group pays only while it stays unwritten.** Grouping 90 rarely
  written collections behind one `CowBox` was -0.93 %; widening the same
  group to 126 by adding the per-resolution scratch was **+1.23 %**, because
  one big unshare on nearly every action costs more than the empty clones it
  replaced. Group size x unshare probability < sum of individual clone costs.

- **The dominant shape in this engine is "a per-card loop rebuilds a
  board-level scan".** Four of this run's five rows were that one shape and
  they were -10.0 % between them. The tell in the profile is a function
  whose cost sits in `slice/iter/macros.rs` + `ptr/non_null.rs` rather than
  its own file, and whose caller tree shows it called tens of thousands of
  times. **When a scan is hoisted into a `_with` variant, grep the `_for`
  shim's callers for a surrounding loop** — the trigger half of candidate 2
  had had its `_with` variant for a run and was still O(cards²), because
  three loops still called the shim.
- **A CoW handle pays off in proportion to how many siblings share the
  unshare.** Twenty cards in a zone with one written: -25.6 %. Two seats
  with one written: -2.2 %. Count the siblings before costing the next one.

- **This box cannot resolve a sub-5 % change by wall-clock.** Eight
  alternated `--bench` pairs of a change worth exactly -1.91 % instructions
  read +0.7 % mean with 5/8 pairs positive. Use `callgrind` instruction
  counts for anything you expect to land under ~5 %: same binaries, same
  fixed workload, deterministic to the instruction. Wall-clock stays the
  arbiter for anything allocator- or cache-shaped, where Ir lies.
- **An inclusive share is an upper bound on a memo, not an estimate.**
  `computed_permanent` was 18.45 % inclusive; memoizing it per freeze scope
  hit 34.5 % of calls and returned 1.91 %. Half of its calls come from
  `depth == 0` mutating paths that cannot be frozen at all — check the
  frozen fraction before costing the next memo.
- **A representation change can beat a dozen call-site fixes.** Candidate 7
  read as "audit 79 `iter_mut()` sites"; the fix that landed was one type
  (`CardInstance` = `Arc<CardData>` + `Deref`/`DerefMut`) touching three
  files and two borrow-checker conflicts, for **-25.6 %** instructions. When
  a cost is *structural* — here, "every write to any card deep-copies the
  whole zone" — look for the type that makes the whole class impossible
  before enumerating its instances.

**The frozen candidate snapshots for passes twelve to eighteen are in git,
not here** (~290 lines, `git log -- PERF.md`). Every entry in them was
either paid by a later pass or restated above at a fresher share, so
keeping both meant reading two lists to learn one thing. The two items in
them that were *not* restated are hoisted: the "Ir over-weights allocation
and representation changes" warning is in the methodological notes above,
and the `ability_strip_in_scope` soundness device is in the Log's
filters-and-devices list. What follows is the longer-lived list — items
that outlive any one profile.


0. **Audit the bot's other whole-sweep calls the same way** — one caller
   asking for 40 answers and reading 1. `cast_candidates` is clean; the
   grep discipline stands: a bot-path call to any `compute_*` /
   `*_hand_cards` aggregate that reads one field is the smell. `view.rs` is
   the only legitimate caller of `compute_hand_affordances`. **(a)**
   `available_mana` per hand card — tried, no win, reverted twice; do not
   redo it without a board-size argument.
0.25 **`check_state_based_actions`** — 6.31 % at the thirteenth pass's base,
   9.67 % inclusive now. What is left is **the collects that *find*
   something**: 82,634 of them, ~1,700 Ir each, after the presence scan
   removed ~21 empty walks per sweep. An empty `collect()` does not
   allocate, so cost it with `--auto=yes` before gating.
0.5 **The transaction checkpoint.** The unconditional `perform_action`
   clone was ~9.5 % of the program; the fourteenth pass took the bot's dry
   runs and the mid-round priority pass out of it (-11.56 % net), and
   candidate 7 removed the zone unshare that followed. What remains is
   irreducible while the snapshot is unconditional. **The audit rule the
   correction taught: a `dry_run` site is only sound when the caller cannot
   read the state after an `Err`.**
1. **Make `ComputedPermanent` cheap to build** — both legs done
   (`Printed<T>` -17.09 % Ir, `ColorSet` -2.55 %, per-scope memo -1.91 %,
   hit rate 34.5 % against a 50 % ceiling). **The reasoning to keep**: a
   cache on `CardDefinition` is unsound because
   `Arc::make_mut(&mut card.definition)` mutates a uniquely owned
   definition in place (MDFC face-swap, "loses all abilities", keyword
   grants) — which is why the fix had to be a representation change.
1.5 **`effective_mana_abilities`** — 1.88 % over 67,468 calls (1,327 Ir
   each) after the `Cow` row and the land-type gate (-44.7 %). What is left:
   `battlefield_find`'s linear scan (candidate 11's shape),
   `granted_abilities_with`, `intrinsic_land_mana_abilities_with`, the
   `frozen_effects` lock, the `out` `Vec`. **Do not re-try freezing it**: a
   `with_frozen_layers` scope inside it measured **+0.03 %, a null.**
3. **The gather.** Still the largest engine function (9.98 % inclusive,
   ~7.6 % self across five file attributions). Every named sub-item the
   nineteenth-to-twenty-fifth passes listed is paid; what is left is the
   walk itself — the pre-scan loop over the battlefield (0.22 % in the
   per-card `keywords` match alone) plus the `GraveyardAnthem` pass, which
   walks both graveyards unconditionally and has no cheap presence gate.
4. **Memoize the gather outside freeze scopes.** Unchanged, and the blocker
   is invalidation, not caching. The layer system is the program:
   `computed_permanent` 14.76 % inclusive, the gather 9.98 %, heavily
   overlapping. Sub-item **(a)** — does `compute_battlefield` need to
   *materialize*, or can its callers take an iterator? — is closed by the
   nineteenth/twentieth passes (310 calls left).
5. **`Keyword::eq` — 0.50 % self, mostly from `compute_permanent_pass`** —
   linear scans of `Vec<Keyword>`. A bitset for the common keywords makes
   `has_keyword` O(1), shrinks `CardData`, and would also kill the gather
   pre-scan's per-card keyword loop. Rides along with item 1's reasoning:
   it must be a representation change, not a memo.
6. ~~`HashMap` hash choice~~ — **done, -0.942 %** (`841dd40b`,
   `crate::fxhash`), pulled as a determinism fix.
7. ~~CowBox sharp edge / per-card CoW~~ — **done, -25.6 %**
   (`CardInstance = Arc<CardData>`). *When a cost is structural — "every
   write to any card deep-copies the whole zone" — look for the type that
   makes the class impossible before enumerating its instances.*
8. **`legal_block_targets` per-pair requirement evaluation** — still does
   not appear on this profile; a view-layer path, not a bot path.
9. **Actor scaling — re-measure on real cores.** Four-core boxes make
   everything past 4 actors oversubscription.
10. **Effect-resolution recursion depth.** The 32 MB worker stacks
    (`RUST_MIN_STACK` in `.cargo/config.toml`, plus explicit `stack_size` on
    every worker) are still required — a robustness constraint, listed here
    so nobody "cleans up" the stack sizes.
11. **MCTS leaf-evaluation throughput — the strength-conversion lever.**
    r42 Part C measured 33.0 / 56.8 / 121.9 / 249.4 s per game serial at
    64 / 128 / 256 / 512 iterations — linear, so every leaf pays a full
    net eval and nothing is amortized across a search. Rounds 27/29/42
    say raw iterations are the only search lever that pays (256 vs 64
    head-to-head: +2.1 ±0.4 on r42's first ladder seed), and adoption of
    higher rungs into the client is latency-gated — so a 2× here is not
    a wall-clock nicety, it converts directly into playing strength at
    fixed latency. Shapes to cost, cheapest first: batching leaf evals
    (the training-side GPU collator already batches; the ladder/client
    search path evaluates one state at a time — cost the CPU-batching
    win before assuming it), a per-search transposition cache for
    repeated leaf states, early adjudication of settled rollouts.
    Anything that changes *which* states get evaluated needs a
    `bot_ladder` win-rate gate per the round-29 house rule; pure
    batching needs only the perf numbers and a golden-trace check.
    **Part 1 landed (forty-first pass): the forward pass was scalar** —
    vectorized matvec took 64-iter games −44 % and 256-iter −40 %, and
    the leaf from 39 % of search wall to 12 %. The measured split
    (`CRAB_MCTS_TIMING=1`) says the remaining cost is the **rollout
    sim: 88 %**, ~63 engine actions per rollout at ~16 µs — so the next
    rungs here are rollout-side (early adjudication of settled
    rollouts, a cheaper rollout policy tick), which change what gets
    evaluated and therefore gate on the ladder, or the engine action
    loop itself (the (-12) auto-tap subtree above).

**Closed / ruled out**, one line each; the reasoning is in git.

- *State-clone allocation traffic.* CoW-wrapping the per-turn tally
  collections moved the bench 0.1 %. Right about the tallies, wrong about
  the conclusion — the checkpoint cost was the zone *unshare* that followed,
  which candidate 7 removed.
- *The bot's affordance sweep* — `cast_candidates` calling
  `compute_hand_affordances` for one field. Fixed (+42 % fixed decks,
  +52.7 % sealed); the lesson generalises and is item 0.
- *`can_afford_in_state`'s fused scan* (+0.066 %), *`SimBases`* (+0.083 %),
  *hoisting `compute_permanent`'s CR 613.8 gate scans* (-0.007 %, LLVM had
  already hoisted them), *`CounterBag`* (+0.051 %, kept as a determinism
  fix), *widening `ColdState` to 126 fields* (+1.23 %), *a freeze scope
  inside `effective_mana_abilities`* (+0.03 %), *an event-kind mask over the
  trigger dispatcher* (`event_matches_spec` runs 1.22 times per dispatch —
  nothing to gate).
