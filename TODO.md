# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.


Split for size — this file is the working handoff, the archives below are
reference. All three carry an index table at the top; triaged at the
sixty-seventh pass, so don't re-take that.

- `ENGINE_BACKLOG.md` — engine backlog and audits, ordered **bugs /
  mechanics & primitives / rules coverage / tooling**: the correctness audit,
  the robustness filters, the decision-plumbing audit, missing mechanics,
  follow-ups not yet done, suggested next-up tasks, the CR coverage audit —
  **and, at the end, "The 2026-09-12/13 handoff detail, moved verbatim from
  TODO's NEXT"**: the printed-body skip taxonomy, the four shapes in the `cap`
  bucket, the sweep-table discipline, the closed-with-a-reason leads, and — in
  its own dated sub-block — the whole `EventKind` fan-out census, the modal
  census, the collision log and the seed discipline. NEXT had grown past its
  own 15-line bound twice, once to 163 lines and once to seven paragraphs, by
  holding them; findings go there, not here.
- `CARD_BACKLOG.md` — per-set card residuals: what each set still
  approximates, and the remaining gap lists. Titled by **subject**, ordered
  open-first, then the closed sets whose only content is residuals.
- `CLIENT_BACKLOG.md` — the Bevy GUI backlog (visualization / UX / refactor).
  The client **does** build here after four apt packages; that file's header
  has the command and the disk caveat.
- `PERF.md` — the perf record: how to measure, **the standing rules**,
  baseline, log, profile of record, candidates. `PERF_ARCHIVE.md` holds,
  verbatim, the Log entries from `(-249)` down, the Baseline's closing
  states from the `(-185)` tip down, and the candidates section's entries
  from `(-192)` down (the `(-90)`/`(-92)` maps and `(-174)`'s open half
  among them).
## NEXT — the handoff. Rewritten each run; <= 15 lines. Every number lives in PERF.

1. **FIRST:** `git fetch origin claude/modern_decks && git checkout -B claude/modern_decks origin/claude/modern_decks`. Rebase, never force; code before tracker prose; **fetch before every push AND before you start** — a concurrent ML session landed three commits mid-run again (four Commander rules bugs in `mod.rs`/`stack.rs`/`bot.rs`/`card.rs`, Rounds 77b/79) and that forced a **second base** mid-session. Gotchas in **CLAUDE.md**, measurement in **PERF's "How to measure" + "Standing rules"**, the 2026-09-12/13 detail in **ENGINE_BACKLOG's "moved verbatim"** block. NEXT is 15 lines and stays 15.
2. **Gates at the closing tip `cf9a3ad1`** (the branch's, after nine concurrent commits merged in): suite **20,327 / 0 / 6** (`CRAB_ANSWER_LOG=strict`); clippy **0** over the workspace; `--bench` **195,806 / 27.49 / 611.9 / 0 stalls — byte-identical to the committed invariant** across every one of this run's 45 cards and two engine fields, `determinism ok (all pairs split)`, `thread_determinism ok (3 vs 1)`, `peak_rss_mib` 29.6; `audit_panics` **0 bare**; every mirror column at its ratchet (`seat_walks` 0, `retarget` 0, `each_player` 1-allowlisted, `target_opponent` 1 known, the three invented-* columns 0, `loop_splice` 13/13/0/0) and **both 614 gates clean — `audit_enters_tapped` at 0 with the gate on the whole census**. CR_COVERAGE **146/146**. **Pod smoke at a FRESH seed 9103: 2,000 games at each of 2..8 seats = 14,000 games, 14,000 DECIDED, every `undecided_by` column zero, zero panics.** turns/game 19.36 / 32.58 / 45.60 / 57.27 / 64.15 / 78.18 / 92.63. **FRONTIER 9109** — ⚠ and the pod smoke was then re-run at **scale**: five fresh seeds (9104-9108) × 2..8 seats × 2,000 games = **70,000 pod games, all 35 blocks 2,000/2,000 decided, every `undecided_by` column zero, zero panics** (`panic = "abort"` means a panic takes the process, so a complete block IS the crash-freedom result). Mean turns/game 19.45 / 32.51 / 45.31 / 56.94 / 64.39 / 77.80 / 92.58 — **linear in seats at ~12.2 turns a seat above two**, which is the curve to watch: a run that bends it is doing something to the format rather than to a card. ⚠ **And the NON-Commander side, which `--bench` cannot reach:** it plays `--decks fixed` (basic lands + 34 spells), so none of the 83 converted taplands is in it. Four fresh seeds × cube/sos/sealed × 300 games an archetype = **30,000 two-player games, 0 undecided, 0 panics**, in under 15 seconds total. **A byte-identical `--bench` is a statement about the fixed pool and nothing else** — when a change touches cards, run the pools that hold them. ⚠ Two suite numbers were measured this run and **both were right**: the Bevy client's ~200 tests build only when its apt dependencies are present, so the count moves 20,0xx ↔ 20,2xx with the box, not with the tree.
3. 🔎🔎 **THE FIND, AND IT IS A METHOD: RANK A CLONE BY WHETHER ITS OWNER NEEDS TO OWN — not by the line the allocation census names.** All three rows are the same question asked of a `Clone`, none is a buffer, and the largest one (`(-364)`, -0.530 % of `cube`) was found by walking **up** from the census row (`card.rs` lines inside a derived `Clone`, nothing to do there) to the **field type** that asked for the copy. Candidate **(R)** in PERF is this census and it is where to start.
4. 📐📐 **THE THREE SHAPES, IN THE ORDER THEY PAID:** ① a clone inside a loop whose trip count is "1 + something nobody has" → `std::iter::repeat_n`, which yields the ORIGINAL last (`(-361)`, six sites, one device); ② a table materialized before anyone asked whether it was needed → ask first, and **an EMPTY table is the cleared table** (`(-362)`, four lines, -0.548 / -0.410 / -0.513 %); ③ a `Box<T>` in a type the engine clones, where T is immutable → `Arc<T>` (`(-364)`, 395 files of sed, three misses caught by `cargo check --workspace --all-targets`).
5. 💡💡 **THE TOP LEAD IS PERF'S CANDIDATE (S) — its cheap half is TAKEN as `(-370)` (-0.023 / -0.050 / -0.026 %, and it reached only 5.6 % of the unshares, because most of the census row is a zone's FIRST push and no headroom can remove one), and the expensive half is CENSUSED AND PRICED, NO BUILD SPENT: `CowBox<Vec<T>>::push`'s UNSHARE PATH COSTS TWO ALLOCATIONS — 36,724 of `cube`'s 664,995 (5.5 %), ~10.2 M Ir, 0.69 %.** `cow.rs:107` is the buffer, `cow.rs:110` is the `Arc` box, 18,362 shared-path pushes apiece; top callers `send_to_graveyard` 12,888 / `finalize_cast` 5,358 / `resolve_top_of_stack_inner` 4,068. **The device already exists in the tree** — `fx_pool::alloc_with` (`mod.rs:1029`) parks a replaced `Arc` and takes one back once its refcount has dropped, `(-166)`'s rule, 68-82 % hit rate one level out. ✅✅ **ITS ONE STRUCTURAL UNKNOWN IS NOW MEASURED AND IT IS FREE: an EMPTY `impl Drop for CowBox<T>` reads +0.004 / +0.001 / +0.00004 %** (a null experiment, two builds, no pool). **That also corrects `(-369)`'s rule — drop FLAGS on a value moved through returns cost; a FIELD's drop glue does not.** What is left to price is a uniqueness check per drop (~110 k atomic loads) and a TLS park bounded by the 18,362 unshares, against ~36,724 allocations recovered: **roughly -0.4 % of `cube`, and the remaining unknown is the pool's HIT RATE, not its cost.** The `Drop` forces one edit, `into_inner`'s `ManuallyDrop` + `ptr::read` (five lines, written and reverted with the experiment). ⚠⚠ **THE OBVIOUS PARK POINT IS STILL WRONG AND (S) SAYS WHY:** parking the handle `push` REPLACES manufactures the very unshares the pool exists to serve — it is shared right now, so a pool reference leaves `strong_count` at 2 and the *other* holder's next write deep-copies where it would have written in place. **Park only at a release site where the handle is already unique.** Two further traps in (S): `Drop` cannot be specialized to `CowBox<Vec<T>>` (and would forbid `into_inner`'s move out of `self`), and a `thread_local!` in a generic body is ONE static for every instantiation — use a `CowPooled` trait with a macro-generated static per impl; the tree needs three (`CardInstance`, `StackItem`, `ContinuousEffect`). Budget ~0.22 % here, ~3x that shipped.
6. 💡 **SHAPE ③ HAS STOCK LEFT TOO.** `Effect` still carries `Box<Effect>` (149 fields, 3,354 `Box::clone` / 1.24 M incl on `cube`), `Box<SelectionRequirement>` (42 fields, ~0.28 M) and `Box<Selector>`/`Box<Value>`. ⚠ **`Box<Effect>` is NOT `(-364)`'s safety case** — a nested effect *is* rewritten (X substitution, modes), so the swap needs `Arc::make_mut` and **the write sites have to be found before the read win is priced**. `(-364)` was safe precisely because the workspace compiled with zero of them.
7. ⚠⚠ **`(-363)` IS THE FLOOR UNDER SHAPE ③ AND IT COST A BUILD: AN EXACT ALLOCATION PREDICTION IS NOT A PREDICTION OF THE ROW.** `players` as `SmallVec<[Player; 2]>` removed **exactly** the 22,034 spine allocations it promised and cost **+0.754 / +0.652 / +0.722 %** — `self.players[…]` is the most-read field in the engine and a `SmallVec` read is a branch where a `Vec` read is a load. The allocator gave back 4.0 M; the deref spent 10.2 M. **`Box`→`Arc` is one pointer hop and is fine; inlining a table the engine indexes is not.**
8. 📐📐 **THE POOL TRILOGY, AND IT IS TWO NUMBERS: allocations recovered PER CALL, and FRAMES THE GUARD CROSSES.** `(-367)` one buffer / one frame / 0.96 a call **WINS -0.127 %**; `(-368)` two buffers / one frame / 0.55 a call **LOSES +0.126 %** (an empty accumulator never allocated, so the `take` is pure overhead on half the calls); `(-369)` one buffer / **four frames** / 0.92 a call **LOSES +0.037 %** — a `Drop` guard costs drop-flag bookkeeping at every frame it is returned through, and `perform_action_inner` alone picked up +764 k for a value it only passes along. `(-359)`'s ~52 Ir a `LocalKey::with` pair is the per-touch constant. **Ask how many frames the guard crosses before reaching for it.** 📐 **AND AN ALLOCATION HAS A PRICE RANGE, NOT A PRICE: 111 / 222 / 245 / 278 / 509 Ir**, all measured by deletion (`(-355)` / `(-362)` / `(-364)`). A buffer a caller builds is 222; one `GameState::clone` copies is 278 (the fill loop and the drop go too); one carrying a `String` and six `Vec`s is 509 (the `memcpy` goes too). **Size a lead by the BYTES it copies, not only by the count.** ⚠ **AND THE BOX WAS A NON-EVENT, WHICH IS ITSELF A READING**: the base missed its prediction by **+0.017 / +0.010 / +0.016 %**, an order of magnitude under the two previous fresh-container steps. **A fresh container is a fresh box, but "fresh box" does not mean "the last offset" — re-take the base and read what it says.**
9. ⚠ **AN `Arc` SWAP NEEDS THE ATOMIC CHECK AND IT PASSED**: `bench_ab.py` over 32 pairs reads **-0.19 % median / -0.26 % mean (sd 2.74)**, i.e. nothing, and the reason is structural — `simulate_match_pairs_piloted` takes `&[CardFactory]`, so every thread builds its own `CardDefinition`s and **no `Arc<TokenDefinition>` is ever shared across threads.** Check the sharing before worrying about the refcount.
10. **Perf queue: ✅ (O)'s `TokenDefinition` row TAKEN as `(-364)` (one level up), its `lethals` row as `(-365)`, four short-lived lists as `(-366)`, `mana_source_table`'s buffer as `(-367)` and the CoW unshare's headroom as `(-370)`, `GameState::clone`'s two lines split ✅ `(-362)` / ❌ `(-363)`.** ❌ **The auto-tap `events` buffer is DECLINED, censused by consumer, no build spent**: 15 of its 21 consumers *move* the buffer into their own accumulator, so an out-param buys at most ~29 % of the row for a five-site signature change. ❌ **`PrintedList::push` is DECLINED as (T)** — 18,534 allocations, but the `Vec` form either reallocs at `into_boxed_slice` or puts +8 bytes on `ComputedPermanent` (226,734 constructions), and the rate is 0.082 pushes a pass. Open on the shortlist: `affected_from_requirement` `types` (3,684), `computed_permanent_hinted` (5,852 sealed), `compute_permanents`' `pass` buffer ❌ REFUTED as `(-369)`. Also open: **(H)** (price the clone side first), **(N)**, **(P)**.
11. ⚠ **`(-361)`'s `fixed` COLUMN IS THE CONTROL AND IT IS HOW TO READ A THREE-POOL SPLIT.** That pool fires almost no triggers and read **+0.010 %** while the two trigger-heavy pools paid 0.19 / 0.10 %. **A row whose pools split like that is a WORKLOAD row; one that moves all three together is a codegen row** — and `fixed` is the cheapest pool to check it against.
12. **THE GLOBAL LINE TABLE SAYS THE *Ir* PROFILE IS FLAT — do not re-derive it.** Largest single source line 0.97 %, 12,537 lines hold 86.9 %. ⚠ **But "flat" was never true of the ALLOCATION table, and it is not true of the CLONE table either** — 3,005 `TokenDefinition::clone` calls were 0.53 % of `cube` between them. ⚠ `Battlefield`'s lane word has exactly ONE two-bit field left (shift 62), with a `const` assertion in `zone.rs`.
13. ⚠⚠ **SIX TRAPS, ALL LIVE.** `cargo build --profile profiling-fast -p crabomination --bin bot_ladder --no-default-features` — **both flags load-bearing**; `nm <bin> | grep -cE " (T|t) (_)?mi_"` = 0 before filing a number. ⚠ **DO NOT EDIT THE TREE WHILE A MEASUREMENT BINARY IS BUILDING.** ⚠ **Never overlap two sweep blocks on four cores.** ⚠ **DISK: this box started with 30 GB free and hit `No space left on device` TWICE mid-run**, and it fails a build as a *compile error* (`couldn't create a temp dir`) — `target/debug/incremental` reached **12 GB** on its own and `rm -rf` returns all of it with nothing to rebuild; do it BEFORE the release build, not after. ⚠ **An A/B's base must be rebuilt after every rebase**: a concurrent ML/Commander session landed seven commits across this run and three of them touched the engine, so `(-362)`, `(-366)` and `(-368)` each needed their own fresh base. ⚠ Box timings, SIXTEENTH box (4 cores, 15 GB, Xeon @ 2.80 GHz): cold `profiling-fast` **~9 m**, engine-only **3m27-3m51s** (10m37s when a base-crate commit lands under it), cold `release` **31m32s**, three pools' callgrind in parallel **~60 s**, full suite **119-127 s**, workspace clippy ~4 m.
14. **Build times are CLOSED as a lever** (PERF's three build-time sections). The one thing to keep doing: `cargo check -p crabomination --no-default-features` while iterating, the affected test binary only, full suite at the end.
15. **Cards & rules, leftover only:** `audit_dropped_may` 199; `audit_keyword_drift`'s last 4 (bestow / escalate / flashback / kicker) each need a PRIMITIVE; card skips `nocolors` 950 / `noback` 60 / `nosubtypes` 43 / `nokeywords` 29; `audit_incomplete` 858 documented approximations. `audit_panics` clean (70 sites off the bin/test paths: 59 guarded, 11 lock-poison, **0 bare**). ⚠ 605 `definition.is_*()` reads remain and **that is NOT a grep to run to the end**.
## NEXT — Commander. The other 3-hour handoff; <= 15 lines. Rewritten each Commander run.

1. ⚠⚠ **SEVERAL SESSIONS SHARE THIS BRANCH. FETCH BEFORE YOU START AND BEFORE EVERY PUSH.** Rebase, never force. A new pod seat goes at the END of `target_decks()` with the next free number — seats moved three times in one run (57 → 59) as others landed first; renumber the comment, the commit message and DECK_FEATURES when you rebase. ⚠ **DISK** fills silently (a background command's output vanishes): `rm -rf target/debug/incremental` + `find target/debug/deps -maxdepth 1 -type f -size +50M -mmin +20 -delete` before any optimized build. `release-fast` `bot_ladder` ~8.5 min, full suite ~170 s, clippy ~3 min. Wait on a **pid**, never `pkill -f`.
2. **Gates at this run's tip (2026-09-25, session `018BnccM` run 5):** suite **22,115 / 2 / 7** (strict answer log) — the two are `cr_rules` catalog audits over other sessions' just-landed cards (Grave Endeavor and Midnight Pathlighter, fixed upstream since; Audacious Swap's slot-0 filter still open); workspace clippy **0**; `--bench` **195,806 / 0 stalls — byte-identical**, `determinism ok`; 4-seat pods ≥ 999/1,000 decided per new seat (open queue names the undecided); strict debug pods 600/600; cube/sos/sealed 7,500 decided; every card of every seated list played in the census.
3. **Instruments:** `scripts/precon_scan.py --deck <MTGJSON name>` (missing cards; `--top 45` ranks every precon), `bot_ladder --commander --pod-decks I,J,.. --card-census` (what a list never plays), `--first N --games 1` (replay one game), and debug pods `CRAB_ANSWER_LOG=strict target/debug/bot_ladder --commander --seats 21 --games 24 --seed N` — `debug_assert!` guards are where pod bugs are. **A new card is the cheapest probe of an old primitive**; a 1,000-game pod is the cheapest probe of the bot.
4. **Official precons are seats 11-147, card for card** (per-seat notes and residuals: DECK_FEATURES; open residual rows: INCOMPLETE_CARDS' Commander routine table). **Session `016DJoKA` landed Coven Counters (MIC, Leinore) as seat 120, Cavalry Charge (MOC, Sidar Jabari) as seat 121, Eternal Bargain (C13, Oloro) as seat 126 and Power Hungry (C13, Prossh) as seat 130 and Living Energy (DRC, Saheeli, Radiant Creator) as seat 138** (CR 118.9 energy as an alternative cost, Stridehangar's artifact-token rider, doubling a player's counters; ⚠ the bot looped Whirler Virtuoso under Decoction Module + Panharmonicon to 980 Thopters — `board_is_saturated` stops a token sink once the board is overkill; ⚠ Eminence "whenever you attack" now fires from the command zone, CR 113.6b; ⚠ cards drawn "this turn" reset for every player; the bot's any-turn graveyard-grant gate; `Effect::ExchangePower` CR 701.10g, `EachPlayerTakesCreatureOfNext` — Order of Succession's pass around the table; ⚠ the bot looped Tooth and Claw under Primal Vigor 3,936 times — `renews_its_own_fodder`) **Session `018BnccM` landed Blame Game (MKC, Nelly Borca) as seat 93, Temur Roar (TDC, Eshki) as seat 97, Adaptive Enchantment (C18, Estrid) as seat 102 and Aura of Courage (AFC, Galea) as seat 106 Ruthless Regiment (C20, Jirina) as seat 111 and Most Wanted (OTC, Olivia) as seat 116 Tricky Terrain (M3C, Omo) as seat 122 Everyone's Invited! (SLD, Morophon) as seat 131 and Planar Portal (AFC, Prosper) as seat 141** (⚠ `EntersAsCopy.legendary` was never read; "until your next turn" copies reverted at this turn's cleanup; ⚠ a turn-granted trigger's "you control" filter read every seat's creatures as yours — Predators' Hour, Arm with Aether; `SacrificeAndRemember` skipped the resolution's sacrifice bookkeeping) (⚠ a pod seat's as-enters creature-type naming always named Demon — off-stack asks went to the headless decider; a dying token never matched `TriggerObjectIsChosenType`). **Session `01SSG31v` landed Upgrades Unleashed (NEC, Chishiro) as seat 109 Mardu Surge (TDC, Zurgo) as seat 113 and Miracle Worker (DSC, Aminatou, Veil Piercer) as seat 118 and Timeless Wisdom (C20, Gavi) as seat 128** (⚠ Glimmerpoint Stag's blink returned the card to its controller with a counter; the pod bot copied Extravagant Replication tokens into a board cap) (⚠ Mobilize now sacrifices at the next end step, CR 702.181a — it was end of combat; the bot never picks an X for Gix's "Discard X cards:") (the printed deck ships two Mossfire Valleys — CR 903.5b; the seat runs a thirteenth Mountain). Its finds: ⚠ **death snapshots dropped static/attachment P/T** (CR 603.10 — `GameState::lki_clone`, every `died_card_snapshots` site); `R::IsModified` looks back at `auras_at_death`; `ApplyToTargets` read hostile to the auto-targeter, so no "up to N target creatures you control" friendly slot was ever filled (Silkguard uncast in 1,800 pods); the bot played a karoo as its only land every turn (a no-progress draw). **This run: Entropic Uprising (C16, Yidris) LANDED as seat 51, Endless Punishment (DSC, Valgavoth) LANDED as seat 59, Arcane Wizardry (C17, Inalla) LANDED as seat 66, Urza's Iron Alliance (BRC, Urza) LANDED as seat 69, Exquisite Invention (C18, Saheeli) LANDED as seat 76, Mishra's Burnished Banner (BRC, Mishra) LANDED as seat 83, Tinker Time (MOC, Gimbal) LANDED as seat 84, Legends' Legacy (DMC, Dihada) LANDED as seat 85, Planeswalker Party (CMM, Commodore Guff) LANDED as seat 91, Reign of Dragons (FDC, Lathliss) LANDED as seat 95, Eldrazi Incursion (M3C, Ulalek) LANDED as seat 100, Eldrazi Unbound (CMM, Zhulodok) LANDED as seat 108, Subjective Reality (C18, Aminatou) LANDED as seat 114, Spirit Squadron (VOC, Millicent) LANDED as seat 124, Jump Scare! (DSC, Zimone) LANDED as seat 133, Obscura Operation (NCC, Kamiz) LANDED as seat 143 — both past `MAX_SEATS` (64), so `--pod-decks` only** (also Fae Dominion 45 and Invent Superiority 48 earlier). **Session `01XnuL2a` landed Open Hostility (C16, Saskia) as seat 64 and Phantom Premonition (KHC, Ranar) as seat 70 Squirreled Away (BLC, Hazel) as seat 75, Evasive Maneuvers (C13, Derevi) as seat 81 and Draconic Destruction (SCD, Atarka) as seat 87 Family Matters (BLC, Zinnia) as seat 94 Virtue and Valor (WOC, Ellivere) as seat 96 — Auras put onto the battlefield attached, CR 303.4f — Divine Convocation (MOC, Kasla) as seat 101 (⚠ convoke taps used to fire no "becomes tapped" trigger, CR 702.51) Symbiotic Swarm (C20, Kathril) as seat 103 (⚠ an up-to-N graveyard slot named one card twice, CR 115.3) and Painbow (DMC, Jared) as seat 110 (⚠ cast triggers read X as 0; multicolored anthems dropped) Arcane Maelstrom (C20, Kalamax) as seat 117 (⚠ the bot never cast a pure retarget spell — Deflecting Swat), Enhanced Evolution (C20, Otrimi) as seat 119, Riveteers Rampage (NCC, Henzie — granted blitz; Weathered Sentinels' per-defender Defender waiver) as seat 125, Grand Larceny (OTC, Gonti — `game/effects/theft.rs`; spells-you-don't-own mana) as seat 129 and Party Time (CLB, Nalia — ⚠ a once-each-turn trigger was spent by an event that failed its condition) as seat 136, Faceless Menace (C19, Kadena — CR 702.37 first-face-down discount, Rayami's any-player death exile) as seat 142** (the bot now casts a power-doubling trick, Unleash Fury; ⚠ a flickered creature no longer re-reads as kicked, CR 400.7) (seats 42, 46, 49, 53, 55 earlier); ⚠ **bots never foretold** (no bot path foretold or cast a foretold card — now both), and CR 702.143a: foretelling is legal any time you have priority on your turn. **Session `01546QdT` landed Seize Control (38), Rebellion Rising (41), Feline Ferocity (52), Primal Genesis (56) and Breed Lethality (C16, Atraxa) as seat 61** (its census found the bot looping two Enduring Scalelords — fixed, CR 732.2) **and Enduring Enchantments (CMM, Anikthea) as seat 68** (Demon of Fate's Design's pay-life cast, Narci's Saga final-chapter rider; the bot now casts through board-granted alt costs — Kentaro and Fist of Suns were invisible to it too) **, Exit from Exile (CLB, Faldorn) as seat 72** (foretold / adventure / plotted casts and lands played from exile now count as "from exile") **, Death Toll (DSC, Winter) as seat 77** (batch card-type counts, a sacrifice-to-cast-from-top static) **, Mind Seize (C13, Jeleva) as seat 89** (protection from a chosen player, kicked split second) **, 20 Ways to Win (SLD, Go-Shintai) as seat 92** (alternate wins; Tragic Arrogance's caster now chooses) **, Cabaretti Cacophony (NCC, Kitt Kanto) as seat 98** (`Effect::EachOpponentChooses`) **and Draconic Rage (AFC, Vrondiss) as seat 104** (dice: ignore-lowest, two results assigned, each player rolls; ⚠ attaching to nothing detached the Aura, CR 701.3b) **and Draconic Dissent (CLB, Firkraag) as seat 107** (board-wide goad statics read by `goaders`; goaded creatures that can't block) **and Prismari Performance (C21, Zaffai) as seat 112** (Pyromancer's Goggles' copying mana rider; ⚠ an emblem's graveyard flashback grant was never read, CR 114.4; Radiant Performer's entry trigger had no spell target). **Session `011A87o2` landed Devour for Power (CMD, The Mimeoplasm) as seat 58, Mirror Mastery (CMD, Riku) as seat 62 First Flight (SCD, Isperia — Gideon Jura's CR 508.1d lure) as seat 65 Undead Unleashed (MIC, Wilhelt) as seat 73 and Nature's Vengeance (C18, Lord Windgrace) as seat 80. ⚠ The field is past `MAX_SEATS` (64, the `u64` seat masks): `--seats` stops there, later lists need `--pod-decks`.** **Session `01P5KQkY` landed seats 32-34, 36-37, 40, 44, 47, 50, 54, 57, 60, 63 (Raining Cats and Dogs, SLD, Rin and Seri), 67 (Growing Threat, MOC, Brimaz — planar die as an effect, `game/effects/planar.rs`), 74 (Call for Backup, MOC, Bright-Palm — CR 700.2 per-mode target checks, LookPick's X) 79 (World Shaper, EOC, Hearthhull — CR 611.3a attacked-this-turn statics) 82 (Peace Offering, BLC, Ms. Bumbleflower — CR 603.3d cast-trigger target slots, emblem free casting, named-token replacement) and 88 (Nature of the Beast, C13, Marath — Mystic Barrier's CR 508.1a direction lock; the bot spends remove-X-counters abilities)**; its bot finds: pod seats cycle, choose-two/three spells are cast, attacking-or-blocking instants ride the combat window, **Incubators are transformed** (`server/transform_sink.rs` — Brimaz 1.7 → 12 %). **Session `015cKV6u` landed Hatsune Miku (SLD, Trostani) as seat 71** (`--pod-decks` only): ⚠ the bot stacked its own Aetherflux shots forever (CR 117.3c — fixed, `stacks_onto_own_activation`); `ColorCountOf` missed the stack. ⚠ **Stalls, fixed:** one token batch mints at most `MAX_BATTLEFIELD + 1` (a bot probe of Storm Herd at 15,197 life minted 15,197 — 22 s a decision); a token-making mana sink waits for the second main phase (Emmara tapped 900 Saprolings for Selesnya Guildmage instead of attacking). **Fixed later:** Storm Herd cast at 1,000+ life board-capped 2 / 4,000 — the bot now holds a token spell that would carry the board past `MAX_BATTLEFIELD` (`cast_candidates`' board-bound gate). **It also landed Witherbloom Witchcraft (C21, Willowdusk) as seat 78**: cast creatures never reached `creatures_entered_this_turn` (fixed in the central `PermanentEntered` funnel); `PowerOf`/`ToughnessOf` missed library and stack; `CastSpellMatches` answered source atoms false. And **Witherbloom Pestilence (SOC, Dina) as seat 86**: the bot now casts a sacrifice-X spell (Immoral Bargain) and no prompting seat is asked for a target past an X cap. And **Silverquill Statement (C21, Breena) as seat 90**: CR 508.1's "whenever a player attacks one of your opponents" (`EventScope::OpponentOfYoursAttacked`); the pinger skips indestructible creatures and goes face with a paid repeat shot. And **Draconic Domination (C17, The Ur-Dragon) as seat 99**: `EntityMatches` on a player now reads the requirement; a became-target trigger's "that player" is its actor; Niv-Mizzet, Dracogenius drew on damage taken. **Silverquill Influence (SOC, Killian) as seat 105**: `R::EnchantedByYourAura`; `RevealTopMayCastOneFree` takes a card filter. **Prismari Artistry (SOC, Rootha) as seat 115**: ⚠ CR 202.3e — a cast trigger read an X spell's mana value without X; ⚠ `R::HasName` wasn't card-only (name-scoped statics were dropped); board-bound gates for cast-trigger fan-out (Leitmotif Composer), attack triggers (Redoubled Stormsinger × Harmonic Prodigy) and free copies (Surge to Victory); the bot declines Orzhov Advokist's peace offer from the last opponent standing. Then more board-bound gates (8-seat seeds 11100/11101 1,000/1,000): suspend free casts, Mirrorwing Dragon's copies capped at the room, stack-pending tokens counted, all stopping at `recommend::BOARD_GATE` (an eighth short of `MAX_BATTLEFIELD`). (Its Miracle Worker build was dropped: `01SSG31v` landed the deck first as seat 118.) **Desert Bloom (OTC, Yuma) as seat 123** — Its Scute Swarm pod caps are fixed later: a self-copying token mints nothing past `BOARD_GATE` (pods 1,000/1,000; 6-seat wall +50 %). **Lorehold Spirit (SOC, Quintorius) as seat 127**: `ExileFromGraveyard` now records what it exiled this resolution. **Explorers of the Deep (LCC, Hakbal) as seat 145** (session `01TfrB1i`): explore-twice replacement, filtered retrace grants, a targeted-ability tax (Kopala). **Counter Intelligence (EOC, Inspirit) as seat 139** (session `01TfrB1i`): ⚠ CR 613.8 — a card-type-filtered set now reads layer-4 type changes (an anthem reached no animated land); `SpellGainsSunburst`, `PutCountersOf`; ⚠ no bot ever stationed a Spacecraft (CR 721 — fixed: the self-counter sink stations after combat); Inspirit still wins 3-8 %. **Deep Clue Sea (MKC, Morska) as seat 134** (session `01TfrB1i`): ⚠ permanent ascend is a static checked on every entry / control change (`game/ascend.rs`, CR 702.131b — six cards re-checked it at upkeep); `Effect::Ungoad`, a silence until your next turn, Tezzeret's first-artifact discount; Thirst for Knowledge kept its artifact discard. **Dungeons of Death (AFC, Sefris) as seat 132**: `PlayerRef::PlayerToYourRight`, doubled dungeon room abilities (Hama Pashar), Rod of Absorption's exile-instead static and total-mana-value free casts. **Deadly Disguise (MKC, Kaust) as seat 135**: `R::TurnedFaceUpThisTurn`, return face down, the next-face-down discount, turned-face-up trigger doubling, `turned_face_up_counters`. **Quandrix Unlimited (SOC, Zimone) as seat 137**: filtered counter adders, a per-counter first-X-spell discount, next-matching-spell triggers, X doubling, all-spell flash; ⚠ `HasXInCost` inside `CastSpellMatches` is false on the stack (CR 202.3e). **Maestros Massacre (NCC, Anhelo) as seat 140**: granted casualty (`casualty_for`), Audacious Swap's shuffle-and-replace. **Creative Energy (M3C, Satya) as seat 144**: energy doubling, `Value::EnergyOf` / `EnergySpentThisTurn`, `R::PowerExactlyXFromCost`, a kept tapped-attacking copy. ⚠ Fixed: `BecomeCopyOf` kept-ability accumulation (an OOM), the bot's proactive energy self-blink loop. **Riders of Rohan (LTC, Éowyn) as seat 147**: composed from existing parts (monarch, sagas, dash, keyword counters). ⚠ Found in the suite: `blocker_matches_block_filter` answered `HasSupertype` false (Midnight Pathlighter's attackers were unblockable), and a target inside a dice branch is never surfaced at cast (Grave Endeavor).
5. **⚠ OPEN CLAIMS — don't duplicate:** **Abzan Armor (TDC, Felothar the Steadfast, 23; `015cKV6u`, 2026-09-25 run 7).** **Bedecked Brokers (NCC, Perrie, the Pulverizer, 21; `016DJoKA`, 2026-09-25 run 1).** **Dance of the Elements (ECC, Ashling, the Limitless, 22; `018BnccM`, 2026-09-25 run 5).** **Elven Council (LTC, Galadriel, Elven-Queen, 23; `01P5KQkY`, 2026-09-25 run 3).** **Buckle Up (NEC, Kotori, Pilot Prodigy, 23; `01TfrB1i`, 2026-09-25 run 1).** **Mystic Intellect (C19, Sevinne, the Chronoclasm, 23; `01RyGJHK`, 2026-09-25 run 1).** Claim in this line, pushed on its own, before starting a list.
6. 🔎 **This run's engine finds** (ENGINE_BACKLOG): **⚠ the bot shuffled Belt of Giant Strength between two 11/11s 11,365 times** (a base-P/T setter lowers a big host, so "equip the biggest" flipped every tick; an action-capped 12-seat pod) — `pick_equip` now needs a strictly stronger new host. **Session `01XnuL2a`:** convoke taps now fire "becomes tapped" (CR 702.51); a flickered creature no longer reads as kicked (CR 400.7); an up-to-N graveyard slot names a card once (CR 115.3); cast triggers read the spell's X; multicolored anthems route to the layers; the bot's mana-value gate sees next-spell affinity / turn discounts; Unleash Fury-style self-power pumps are combat tricks. The bot answers a spell aimed at its side with a held retarget instant (Deflecting Swat, Redirect — `pick_retarget_shield`). ⚠ **A once-each-turn trigger was spent by an event failing its condition** (CR 603.3d — marked at gather, before the filter; Deep Gnome Terramancer, Folk Hero), and a static-granted trigger could not be once-each-turn at all. `R::DamagedAPlayerThisTurn` reads a dying creature's snapshot (Wave of Rats). **⚠ in pods a bot never cast an "up to N targets" spell it under-filled** — every pod seat is `wants_ui`, the cast suspended into the extra-slot prompt and `would_accept` read that as a rejection (Desecrate Reality 0 for 1,000); the bot now fills extra slots with `auto_extra_targets_for` on prompting seats, and the prompt honors `ForEachOpponentTarget`. `ManaSymbol::ColorlessHybrid` ({C/W}) exists now. a resolving permanent spell and `fire_self_etb_triggers` gather self-ETBs in two walkers — a per-instance rider belongs in `statics_granted_triggers_for`, which both read (Galea); **CR 903.9: a commander going home now emits `CommanderPutIntoCommandZone`** (both the SBA and the replacement route; Myth Unbound); two Scalelord Reckoners stacked 3,905 triggers retargeting each other's Dragon — the bot's hostile pick now skips a permanent its own stack items already aim at; **a battlefield cast trigger picked its target before the cast spell's mana value was in scope** (Skyfire Kirin, Hammerhead Tyrant — `ManaValueEqualsTriggerAmount` never matched at targeting); **goad has one reader, `GameState::goaders`** (`game/goad.rs`: resolved + held CR 611.2b + attached `AttachedIsGoaded`) — read goad through it, never `goaded_by`; five Auras/Equipment that re-goaded on triggers are statics now. Boros Reckoner's bare `Target(0)` on a `DealtDamage` trigger bound itself and looped forever under indestructible; the bot's `ChooseTarget` fallback hit its own board before an opponent's face. a mana source paying its own {1} overflowed the stack (CR 601.2g, found by a 1,000-game pod — `pod::tests` replays it); SpellCast triggers never named their caster (CR 603.2 — "another target player" hit the caster or nobody); no bot path suspended a no-cost card (Wheel of Fate), and the census didn't count suspend/foretell as plays. ⚠ **Chatterfang's token rider re-applied on every resume of a suspended resolution** (CR 614.13 — the board doubled per answer and a bot lookahead hung a pod). ⚠ **Postcombat-main triggers never fired in real games** (CR 505.1a — `advance_step` skipped `StepBegins(PostCombatMain)`; Survival, Florian, Tymna were dead, their tests fired the step by hand). Arcane Wizardry: **The Abyss destroyed every nonartifact creature** (it is one, of their choice); "tap N untapped [X] you control" never let the source pay; **both death funnels filled only a trigger's first target slot**; and the extra-slot filler only ever offered the *first* opponent — a two-seat assumption. Urza's Iron Alliance: an any-kind "remove a counter" cost never saw **keyword counters** (CR 122.1b), and a `WhileCondition`-gated "prevent all damage to this" never applied.
7. 💡 **Open queue:** ⚠ **A Scute Swarm board below the gate is slow, not capped** (measured as `--pod-decks 128,127,126,125,124,123 --seed 11107 --first 259` with Sefris at 128, now 132: 814 Swarms, 8,171 no-op landfall triggers each cycling six seats of priority — 104 s for one decided game); ⚠ **Scute Swarm copies itself to the board cap** (Desert Bloom, seat 123: 4 / 1,000 six-seat pods, seed 10521 games 57, 246, 536, 718 — `--pod-decks 126,125,124,123,122,119`; ~950 Swarms, the landfall fan-out from non-land-drop land entries — fetches, Crawling Sensation — which the land-drop gate doesn't see); ⚠ **Extravagant Replication copies itself to the board cap** (Miracle Worker, seat 118: 2 / 1,000 six-seat pods, seed 10511 games 727 and 873 — `--pod-decks 121,118,117,116,115,114`; ~975 Replications, a mandatory upkeep copy whose bot target is another Replication) — the board-bound gates don't reach a trigger's target pick; X burn at "any target" auto-targets the opponent's face first and the bot burns face only for lethal, so Fireball goes uncast (Nature of the Beast census); Suspend X (Aeon Chronicler); a random-opponent damage split to planeswalkers (Vial Smasher); life-lost counting for "each player who lost life" (Blood Tyrant); "up to one target" choosing none (Star Athlete, Key to the City); `Effect::ExileFromHand` for a prompting seat; bots never emit 6 cast variants; per-slot trick evaluation, `audit_dropped_may`, the invented-target census. Condemn (Nahiri) was unplayed at 31-32 seats. A Chishiro / Nelly two-seat endgame drew on no progress (`--pod-decks 116,93,106,109 --seed 10411 --first 37`; Chishiro answered 447 decisions). Again beside Otrimi (`--pod-decks 122,109,113,119 --seed 10423 --first 442`, 432 decisions). A Scute Swarm board still reaches the board cap through lands put onto the battlefield by effects (landfall fires on any land entry; the bot's gate covers only its hand land drop — `--pod-decks 122,116,111,106 --seed 10420 --first 422`).
8. ⚠ **CR 511.3** (combatants leave combat as end of combat ENDS) and **CR 601.2c** (a required target nothing can fill makes the cast illegal) are both enforced now — a card body written around the old timing / an empty target is a bug, not a quirk.
9. **The rules table is DONE** (FEATURE_ROADMAP's Commander and 800-series sections, bar CR 800.4i and the N-seat encoder). What is left is precons and the residuals they surface. `CR_COVERAGE.md` is generated: `scripts/cr_coverage.py`.

## Standing index (every number lives in PERF, ENGINE_BACKLOG or
INCOMPLETE_CARDS; a line here that restates one is a line to delete)

0. **THE BUILD IS THE LEVER, NOT THE SOURCE.** PGO is a ~24 % win on both
   pools and `-C target-cpu=native` is flat — width buys nothing here, layout
   buys everything. **`panic = "abort"` is landed on every optimized profile
   (`(-250)`: +4.5-5.5 % paired wall clock, -3.1..-3.8 % Ir on all three
   pools, found off the cachegrind I1-miss table)** — CLAUDE.md carries the
   one consequence (exit 134, tests still unwind). **And the profile this file measures on is 8.3 % slower
   than `release`, which nobody had priced.** Ladder against it: LTO 0.917,
   `release-fast`+PGO 0.762, `release`+PGO **0.724** — they stack. **A profile
   must be raised under the profile it is consumed under**, or it is partially
   applied with no warning and the win vanishes; the binary size is the check.
   Matrix in PERF's Baseline.
   `scripts/pgo_build.sh`, **opt-in and staying opt-in** so committed readings
   stay plain `release-fast` ones (CLAUDE.md carries the hazard). Measured on
   the actor too, and **the same binaries read -23.1 % or -4.9 % depending on
   the learner/actor balance** — PERF says why, and the rule that fell out is
   the more useful half: print `t_step_ms` against `elapsed_s` before quoting
   any `selfplay_train` throughput number. **BOLT is blocked here** — no
   `llvm-bolt` in the toolchain and no `perf` in the image.
1. **Perf queue** — PERF "Perf candidates" and its Baseline carry every
   number; this is the state and the rules only.
   **`(-87)` IS TAKEN — pass 97 priced it, pass 98 built it as the
   `zone::Battlefield` lane** (`walk_and_store`, hit 75.6 / 79.5 / 72.9 %
   against the ask counts the census predicted 93.95 / 75.76 / 74.30 % for,
   and the pass reads 78.5 / 73.1 / 80.2 % of its own deletion ceiling). The
   device then generalised to `(-89)`'s damage-shield walks at four times the
   rate of its first two lanes. **The transferable half is the method, not the
   entry: a census that prices a memo's hit rate before anyone builds it cost
   one `release-fast` build and turned a "nobody has priced it" into a
   shipped 0.6-0.9 %.** `(-90)` and `(-92)` below are maps to read first, not
   leads.
   **`(-90)` is a map, not a lead** — the
   ninety-sixth pass's whole-profile re-read at `3b8bfd03`, with three sized
   leads (`compute_permanent_pass`' 279.5 Ir body, the diffuse allocation
   table, `granted_abilities_of_inner`) and, more usefully, **four things it
   refutes by inspection with no build spent**: every freeze scope worth
   widening (the gather count is at its floor — `cg_contexts.py` says all six
   top contexts are one gather per distinct game state, and Ir/call says
   "gather" and is wrong about all of them), and a batch event-kind mask in
   front of `dispatch_triggers_for_events` (11 `event_matches_spec` calls a
   dispatch, so the inner fan-out is not the cost) — **that last one is
   TAKEN as `(-195)` in the per-pair form** (cube -1.10 %, sealed -1.34 %):
   the fan-out is small per dispatch but the mask also skips the per-pair
   setup ahead of the loop.
   **Read `(-90)` before pulling anything else off this queue.**
   **Open, in order:**
   **`(-82)` IS NOW CLOSED — its `available_mana` half is refuted off the
   dumps with no build spent** (pass 97): 12,240 builds against 12,478
   `cast_candidates` calls is **0.98 builds a call**, 92 % of builds are
   inside `cast_candidates`, its 5,474 real-board calls already take
   `main_phase_action_with`'s shared cell, and its 7,004 simulation calls sit
   in a loop that `dry_run`s a mutation into the board on every iteration —
   different board, no cache can span it. What is left is 694 builds (5.7 %,
   ~0.11 % of `cube`) in the picker chain. **The rule: `asks / builds` is a
   ratio in the dump — ask "is this already shared?" with
   `cg_edges.py --callers` before theorising about who could share.** Then
   `(-83)`, `(-9)`'s open half, `(-80)` rows 3/4,
   `(-51)(a)`, `(-69)`, `(-61)`, `(-59)`. **`(-92)` IS THE ENTRY TO READ
   FIRST AND IT IS A MAP, NOT A LEAD: the profile is FLAT.** The whole-program
   line profile (new, in PERF's "Profile of record") says the hottest single
   line is 0.97 %, 11,165 lines hold 86 % of the run, and slice iteration
   plus the pointer/`Vec` machinery under it is **~30 %** of the program.
   Six functions have been read by line across three passes and all six say
   "no hot line" — **stop looking for one**; `cg_calls.py` and
   `cg_contexts.py` are the two questions a flat profile still answers.
   `(-92)` names the three shapes it does still see, cheapest first.
   **`(-91)` IS TAKEN** (`cd0842e9`,
   `fixed` -0.673 / `cube` -0.721 / `sealed` -0.813 %, recomputes down
   89-93 %, robustness grid green over 33,120 games). The device generalises
   and the Baseline states it as a rule: **put the cache INSIDE the handle it
   is keyed on**, not beside it — a `Deref`-only newtype makes every write a
   compile error, and moving the memo into it makes the one expression the
   newtype cannot stop (`c.definition = other.definition.clone()`) *correct*
   instead of rare. **It read 95 / 63 / 43 % of the ceiling that entry
   filed and the gap is now explained: there is none.** A second, independent
   implementation of the same change, built at `cd4ad277` by a concurrent
   session, lands within 0.009 / 0.014 / 0.011 % of the ceiling probe's own
   *absolute* totals on all three pools. The percentage shrank because
   `3b8bfd03` took 31 M of `sealed` out from under the ceiling between the
   probe and the implementation. **A ceiling is a percentage of the base it
   was read at** — Baseline, ninety-seventh pass.
   **Two entries at the top of the queue, both sized on all three pools at
   the ninety-fifth pass, one of them now half taken:**
   `(-89)` (the combat-damage prevention cascade, **1.55 / 2.34 / 1.69 %**,
   ten questions a damage event; its two largest rows are already taken for
   -0.087 / -0.114 / -0.098 %, eight are left, and **both taken mechanisms
   were then applied to a neighbouring row in the same cascade and both lost
   there** — read the refutations before repeating either).
   **`(-88)` IS CLOSED — half taken, half refuted with a census.** Taken at
   `ec5dc3a9` (`fixed` -0.559 / `cube` -0.751 / `sealed` -0.533 %): the SBA
   death sweep's whole-board layer pass is now a pass over the permanents its
   own gate names, because `card_death_possible` is a *necessary* condition
   for death and the sweep was throwing that walk's result away. Refuted at
   `6e44ce7c`: `CRAB_SBA_CENSUS` says **17.8-19.0 %** of sweeps re-see a
   state, and a skip loses anyway — **price the witness against the work; a
   witness over the same collection as the work is never cheaper than the hit
   rate.** The instrument stays, gated and free (+0.004 / +0.001 / +0.003 %).
   **The `call_mut` census is CLOSED**
   — all six rows swept at the ninety-fifth pass, and the capture rule
   predicted the order on every one (see the Baseline; `process_echo` is read
   and left at 0.01 %, which is the result, not an omission).
   **The `#[inline]` family is CLOSED past the card-type
   predicates** — `has_keyword` + `counter_count` + `same_team` read +0.45 /
   +0.54 / +0.42 %, and +0.15 / +0.17 / +0.12 % with `has_keyword` dropped, so
   every combination loses including the 10-Ir one-liner. Do not rebuild it.
   **`same_team` then won by the other route in the same hours** (one team row
   instead of two, -0.135 / -0.105 / -0.112 %), which is the two halves of one
   rule meeting from opposite sides: read a hot small function's body for a
   repeated question *before* reaching for the attribute. **And then a second
   time, from the callers** — four walks stopped asking it per permanent
   (-0.200 / -0.160 / -0.169 %, calls -72 / -73 / -71 %). The function is now
   1.4 M Ir of `cube` and the family is done.
   **Taken/closed:** `(-84)`(a) and (b), `(-70)`, `(-79)`, `(-77)`, `(-60)`,
   `(-39)`, the `Box` class, `(-80)` rows 1-2, `(-82)`'s targeting half.
   **Refuted with a ledger, do not rebuild:** `(-85)` (a gate is read 3.5x per
   scope exit, so a slot costs ~113 k Ir and nothing else — add gates freely),
   `(-86)`, `(-84)`(c), the Cauldron bit, `(-80)` row 1.
   **The rules this queue runs on, none of which restate a number:**
   * **Run `cg_edges.py --callees` on a 3 % row before theorising about it.**
     `(-82)` sized a function from outside and got the conclusion backwards.
   * **Rank a `call_mut` census by the closure's CAPTURES, not its call
     count** — 5x between the top two rows of one table. **And check the
     CONSUMER before applying the rule**: it is about `collect()` into a
     `Vec`; `sum`/`fold`/`count`/`for_each`/`any` are already internal
     iteration and a hand-written loop loses to them
     (`combat_damage_shaved_for`, +0.010 % on `cube`, reverted).
   * **Halve a no-LTO `#[inline]` reading and halve it again**;
     `[profile.profiling-lto]` is the instrument that checks it (`profiling`
     OOMs on the candidate side only — Cargo.toml says why). **And pick the
     candidate by what its body EXPANDS TO at the call site, not by its self
     Ir** — a 10-Ir row whose one statement is a call to something large is
     the worst candidate on the table and the one a count ranks first.
   * **Divide a loop's item count by its call count before hoisting a
     per-item board walk out of it — and when the hoist fails because the loop
     is short, ask whether the scope is long.**
   * **A per-definition presence bit pays only when the walk it replaces is
     over a list that is usually non-empty.**
   * **`SmallVec` cannot replace a `Vec` of BORROWED data that outlives its
     last use** — no `#[may_dangle]` on its `Drop`, so the shared borrow runs
     to the drop point and every `&mut self` call after it is an error.
     `DispatchScan`'s two lists are that shape (29,474 allocations a `cube`
     run, ~0.15 %) and are **not** reachable without a borrow refactor of two
     long functions. Tried, ten errors, reverted.
   * **Price a `Vec -> SmallVec` swap as `alloc + free` minus `n x (spill
     check + external next)`.** `Vec::from_iter` specializes to *internal*
     iteration and `SmallVec`'s `Extend` does not, so the collect becomes an
     external `next()` loop: `blockers_of` gave back 4.19 M of the 4.15 M +
     2.2 M its allocations cost, for -0.072 % of `cube` net. **And the same
     rule at seven times the size, which is the one to quote** (pass 97): the
     SBA death gate's candidate list read -0.065 / -0.362 / **+0.007** % as a
     `collect()` into a `SmallVec` and -0.580 / -0.766 / -0.549 % as a `for`
     loop. One collect over a battlefield gave the whole win back on two
     pools and reversed the sign on the third. **Fill a `SmallVec` with a
     loop**; the inline storage is fine, the iteration protocol is not.
   * **A `same_team`-shaped call inside a collection walk is a per-*seat*
     question asked per element.** Three answers: hoist it when an earlier
     term of the same `&&` pins its argument, put a cheaper term in front of
     it when one exists, or build a per-seat mask — and the mask pays only
     where the loop is long and nothing cheaper already gates the walk (its
     own build and closure ate three quarters of what it saved in
     `eval_material_inner`). ~140 sites were left alone on that test.
   * **A scope only gathers if a read inside it asks for a computed view** —
     read its first `&self` calls in source order; `(-81)`'s named remaining
     first-reads are the cheapest leads.
   * **Fold a `'N` row into its parent before ranking a self table**, and
     **Ir/call on a function a gate is about to split is the average of two
     populations.**
   * **A dormant gate's cost is its CALL COUNT, and `(-85)`'s "add gates
     freely" was calibrated at ~20 k calls** (pass 97). In front of
     `presence_gate` (242,788 asks a `cube` run) a census hook read `cube`
     **+0.049 %** as a `OnceLock<bool>` and **+0.187 %** as an `#[inline]`
     `AtomicU8` — worse, because `#[inline]` expanded the reader's *cold*
     branch (an `env::var` that allocates a `String`) into every call site.
     **Never `#[inline]` a reader whose slow path allocates.**
   * **A rebase shrinks a patch without shrinking its measurement**, and
     **cite an anchor by a hash already on `origin`** — the last one was
     filed at a hash that no longer resolves.
   * **Split a patch whose rows you want to attribute, or attribute
     nothing** (pass 97). Extracting the SBA death filter's fat-capture
     closure into a method moved `call_mut` -14.5 M against the new method's
     +5.0 M on `cube` — which reads like a 0.33 % win for the extraction and
     is not: measured alone the extraction is **-0.015 %**. That `call_mut`
     fall belonged to the narrowing landed beside it, because the filter ran
     over fewer cards.
   **One open question, not re-landed unilaterally:** the Cauldron revert is a
   20x asymmetric pool split — 0.006 % of the bench pool against 0.125 % of
   `sealed`, and `sealed`/`cube` are the pools the training loop plays.
   Decide it deliberately or leave it.
2. **Perf method** — PERF's "How to measure", "Standing rules for a perf
   pass", "Which pool a change moves". Read all three pools; a pool split is
   a revert.
3. **Instruments** — **`--feature-census` is now deterministic and is the
   ENCODER'S regression check**: `selfplay_train --feature-census 8 --seed 5`
   twice, diff the two outputs, and a byte-identical pair says the encoding
   did not move over ~29 k encoded objects. Use it on anything touching
   `encode.rs` (item 4's caution had no such check before). It was
   *not* reproducible until the ninety-sixth pass — the census's own games
   left `bot::jitter_below` unseeded. **That hole is closed on the training
   actors too now** (item 7): `play_recorded_game_mcts` pins the stream per game,
   so the census's own pin is belt-and-braces and any recorded game replays. **And PERF's "Profile of record" now carries the ACTOR's profile**
   — three of its top rows (`encode_state` 5.7 %, deck building 1.8 %,
   `__memcpy` at twice `cube`'s share) have no row in any `bot_ladder`
   profile at all. Then: `CRAB_SIM_REJECTS`, `CRAB_PAY_FAILS`,
   `CRAB_SBA_CENSUS` (new at pass 97 — the SBA re-sweep rate; it closed
   `(-88)`'s open half and stays as the instrument),
   `server::bot_rejection_count`, `--bench`'s stall split and `undecided_by`,
   and `selfplay_train`'s new `actors:` line (plus `actor_s` /
   `actor_games_per_s` in `stats.jsonl`). **Quote `actors:`, never `done:`,
   for anything about the simulator** — the old rate divided games by a clock
   the learner owns and read up to 3x low.
4. **Encoding caution** — pool / `Vocab` / `TrainRow` / observation and deck
   encodings invalidate the trained nets. Nothing since `dc478735` touches
   `encode.rs`, `crabomination_ml` or `crabomination_nn`.
   ⚠ **A CARD FIX IS NOT AN ENCODING CHANGE, BUT IT IS A SEEDED-GAME CHANGE.**
   The 2026-09-12 subtype / keyword fixes move no pool membership, no vocab
   index and no encoded feature (the encoder reads neither subtypes nor
   keywords), so no net needs a retrain — but `clearwater_pathway` is in the
   cube pool and `lonesome_unicorn` in the deck pools, so a seeded game that
   draws one can play out differently from the same seed before them. Say which
   of the two a card commit is; they are not the same claim.
5. **Robustness gate** — `scripts/robustness_grid.sh` + the actor leg, the
   seeded 4,000-pairing cube sweep (arm it in
   `bot_vs_bot_random_cube_decks_terminate`), the two census env vars.
   **Re-run at the ninety-seventh pass** (`6e44ce7c`), because that pass lands
   a `debug_assert!` in the SBA death sweep: same recipe, **12,000 games,
   1,157,922 rows, 0 stalls, rc 0, no panic / assertion / overflow**, both the
   "memo is stale" and the new "is not a necessary condition" strings verified
   present. Earlier, at `9b1fa94b` — the same
   `-C debug-assertions=yes` + `overflow` build of `-p crabomination_ml --bin
   selfplay_train`, 4 seeds x `--actors 3 --games 3000 --steps 20`:
   **12,000 games, 1,161,305 rows, 0 stalls, rc 0, no panic / assertion /
   overflow**, with the audit binary's 5 assertion strings verified present.
   It is the only leg that reaches the encoder-side memos (the vocab-index
   slot), and nothing in this file had ever recorded a run of it. 123-126
   games/s there, which is an *audit* build and not a throughput number.
   **Green at `599825ba`: 30 cells, 33,120 games, 0 undecided, 0 failures**
   (and at `10a794a8` before it), with the audit binary's assertion strings
   verified present (the script's own header check — `strings | grep -c
   "memo is stale"` must be > 0, and a 0 means `RUSTFLAGS` did not reach the
   crate). **That count is a LINE count, not a message count** — Rust's
   string literals land contiguously in `.rodata`, so several messages share
   one `strings` line; there are eight such assertions in the tree and the
   header prints 5. The gate is "> 0"; do not read the number as a census,
   and do not "fix" it by expecting it to track the assertion count. Run it
   after any pass that
   lands a `debug_assert!` or a presence gate: the 19 k-test suite is not
   their audit, because an assertion needs a *board* to fire on. Five earlier
   runs (through `b635037f`) were also clean and are in git; do not add a
   line per run here. **The 4,000-pairing cube sweep is RE-ARMED AND CLEAN at `527f872f`** —
   981.7 s under nextest in a debug build, every match terminated inside its
   180 s budget, `bot_rejection_count()` unmoved. It had not been run since
   the eighty-seventh pass, and the twelve passes since carry the `CardMemo`
   redesign, the SBA death sweep's narrowed scope, the target-walker
   unification and the encoder's in-place write. Re-arm it after a pass that
   changes what the *bot* proposes or what the engine accepts; the recipe is
   in the test's own doc comment. Previously: the
   passes since have been behaviour-preserving by construction and `--bench`
   is byte-identical through them (195,528 / 27.44 / 611.0 / 0 stalls /
   `determinism ok` / `thread_determinism ok 3 vs 1`).
6. **Bugs** — ENGINE_BACKLOG's live-match section: **no open entry left.**
   Card audits clean — see INCOMPLETE_CARDS.
7. **ML** — ML_NOTES. ~~Open: should `selfplay` seed `jitter_below` from
   `--seed`?~~ **ANSWERED YES (2026-09-11)**: `play_recorded_game_mcts` pins the
   stream per game, so a recorded game replays from its seed —
   `sample_scored_index` already documented the seeded stream as the
   reproducibility hook and the actors were the one path that installed none.
   ⚠ **Runs started after this generate different games than runs before it**
   (same distribution, different draws; no encoding / pool / `TrainRow` change,
   so **no retrain**). A whole net-in-the-loop run is still not reproducible —
   `n` comes off a shared atomic, so the weights game `n` sees vary. ML_NOTES
   has the reasoning and the test.
7b. **Test suite** — `find_data_tests.sh` was wrong **five ways** and is
   fixed; its output is a DELETE list and every bug put live engine tests on
   it. Bugs 4 and 5 are the ninety-third pass's. **(4)** is bug 1 one function
   down: a helper's body ended on the line after its signature unless the
   opening brace was on that line, so every multi-line-signature helper
   spliced its own signature into each caller — sixteen live engine tests
   (`modern/lands_equipment_vehicles.rs`'s fourteen, two in
   `core_rules/xtra.rs`). **(5)** the marker list asked "does it touch a
   `GameState`", and the deck-format engine does not need one — nineteen
   legality tests (all of `core_rules/format.rs`, `multiplayer.rs`'s two
   commander validators, CR 100.2a/100.4a, CR 407.3, cns's Proclamation).
   **A pure-data test is one with no LOGIC under it, not one with no
   `GameState` in it.** **Population 200 (189 + 11 sacred)**; neither fix
   ADDED a row, which is the check to run on the next one, and both were found
   by **reading three tests in the file at the top of the by-file count — a
   filter's false positives cluster, because they share a helper or an API.**
   **Seven slices taken**: `stx/part_23` (19), `classic_sets/ogw` (8),
   `modern/aggro_allied_batches` (11, including three copy-paste batch blocks
   — the convention's other half), `modern/singles_and_legends` (7),
   `modern/cube_rounds` (4), `stx/part_00` (5) and `modern/decks_11_13` (4).
   The pattern is in the tree seven times over, and **the sacred list is
   byte-identical across all of them — run that diff after a fold.** Two
   rules found doing it: **a test that pins a card-specific *effect shape* — a
   modal `min`/`max`, a `Search` filter, a `CantBeBlockedBy(_)` /
   `Madness(_)` / `Suspend(4, _)` / `Typecycling(_)` variant — is not an echo
   and does not fold** (`rna.rs`, `mh/mh2b.rs` and most of
   `cube_expansion_singles.rs` are those), and **a per-cycle table
   that already names its card in the failure message is the folded form** —
   leave `ths.rs` / `rtr.rs`'s `*_stat_lines` and `cube_rounds`'s two
   `*_cycle_definitions` alone.
   **AND THE REMAINDER IS SIZED, WHICH CHANGES WHAT IT IS WORTH.** Of the 189
   non-sacred rows, **65 are the effect-shape kind that does not fold and 124
   are plain echoes that do — and those 124 are spread over 77 files at one to
   five each.** The big slices are gone; every further commit is a 1-5-test
   file with a table of its own. Since the sweep buys **no build time** (see
   PERF's "Test-suite cleanup"), a whole pass spent on a 77-file long tail is
   a poor trade against anything on the perf queue. Take a file when you are
   already editing it; do not open a pass for this. One commit per file,
   binary green either side.
8. **Tip state / build time / filters** — PERF's newest Baseline blocks.
   **Anchor, MEASURED at `79019a42` (on `origin`): `fixed` 935,959,897 /
   `cube` 2,777,133,820 / `sealed` 2,773,877,997** — the interval off
   `181ce81a` is **-1.137 / -1.857 / -1.115 %** over six commits. Earlier:
   `b12393f9` 942,328,403 / 2,805,505,066 / 2,793,794,620 — the ninety-eighth pass is
   **-0.464 / -0.854 / -0.405 %** off `181ce81a` over two commits. Earlier:
   `181ce81a` 946,720,798 / 2,829,667,509 / 2,805,164,959; `a69a8287`
   946,679,422 / 2,829,634,060 / 2,805,064,393 — the whole
   ninety-sixth pass is **-3.25 / -3.70 / -4.00 %** off `e0bc5c46`, over
   eight commits whose individual readings compose to it (PERF's closing
   block has the per-commit table). Earlier anchors on `origin`:
   `42235e7e` 952,230,291 / 2,851,583,965 / 2,820,631,303; `cd0842e9`
   957,023,198 / 2,865,614,181 / 2,831,048,757; `599825ba` 963,502,971 /
   2,886,424,672 / 2,854,266,716, with `--bench`
   byte-identical to the committed invariant (195,528 / 27.44 / 611.0 / 0
   stalls, determinism **and** thread_determinism ok), suite 19,029 / 0 / 5,
   golden traces 7/7 unmoved, clippy clean, and the robustness grid green
   at both memo commits. Further back, `e0cbb4a7`:
   963,706,773 / 2,899,018,180 / 2,885,278,610, and `e0bc5c46`:
   978,492,848 / 2,938,264,442 / 2,921,980,262.
   **Two sessions read `e0bc5c46` independently and agreed to 435 Ir on
   `cube`, 58 on `fixed` and 81,279 on `sealed`** — the third such check, and
   the reason an anchor is checkable at all.
   **⚠ `--bench` IS NOT A THROUGHPUT INSTRUMENT and this pass proved it
   three times.** Three release gates, one container, `host_calib_ms`
   53 / 55 / 55, `games_per_s` 364.80 / 362.97 / **337.51** — the clock went
   the *wrong way* by 7 % while Ir fell monotonically by 3.3-4.0 %. (And the
   whole container reads 337-365 against the last one's 217 at the same
   thread count.) Quote Ir, or `ab_wall.py`'s five ABBA blocks. What
   `--bench` is for is the invariant: decisions / turns / stalls /
   determinism, byte-identical at every gate this pass.
   **`games_per_s` IS NOT PORTABLE ACROSS CONTAINERS and the reason is the
   thread count, not the box**: `--bench` defaults to
   `available_parallelism - 1`, this container reports `nproc` 4 and so runs
   **3** threads where the run that filed `games_per_s 292.65` ran 4 —
   217.09 at `--threads 3`, 280.47 at `--threads 4`, same binary, same
   minute. The header line carries the count and the Baseline blocks quote
   only the numbers block, which is how a 33 % lever went unrecorded.
   **Quote `games_per_s_th`, or pin `--threads` and say so.** `host_calib_ms`
   and `bin_bytes` do *not* catch this — both agreed to within 0.11 % and
   15 % respectively across the gap. Earlier anchors are in git and in PERF's Baseline blocks; do not
   add a line per anchor here. Three rules the series produced, and they are
   the whole reason to keep it:
   **cite an anchor by a hash already on `origin`** (one was filed at
   `c1e4363c`, which no longer resolves — a doc written between two rebases);
   **re-read the anchor, don't sum the rows** (one row was filed ~7x high
   because its A/B ran in a worktree whose base predated the change it
   widened); and **two sessions reading the same tip agree to ~100 Ir on 1-3
   G**, which is what makes an anchor checkable at all.
   Then PERF's "Build time"
   — **the critical-path question it left open is answered**: the target was
   `crabomination_catalog`'s own test harness, 110.7 s of a 213.5 s workspace
   makespan for zero tests, now `test = false` (**-11.3 %** on a base-crate
   edit, **flat** on the engine-file loop; quote a build number with the file
   it touched). The section also now carries the ABBA rule for build-time A/B
   and why a one-sided series across a restart read the sign backwards. Then
   ENGINE_BACKLOG for the
   seven filters. **`--bench` is a 1.2-s run on a shared box: check
   `host_calib_ms` (idle 44-49, and it read 54 then 60 across one session's
   base and tip readings — an ~11 % container slowdown over three hours)
   **and `bin_bytes`** before believing a `games_per_s`.** The second is new
   and is item 0's own rule made automatic: LTO, PGO and `target-cpu` leave no
   `cfg` and no path difference, so a PGO binary prints `release-fast build`
   exactly like a plain one — the size is what tells them apart, and a
   partially-applied profile is what it catches. For a clock comparison use
   `ab_wall.py`: 5 ABBA blocks of `--games 2000 --decks fixed` resolve
   **±2.40 %** and nothing smaller, so a sub-1 % Ir change will read FLAT and
   that is the expected answer, not a contradiction.
   **`cargo nextest` is not in the image** — `curl -sSLf
   https://get.nexte.st/latest/linux -o /tmp/nt.tar.gz && tar -xzf
   /tmp/nt.tar.gz -C ~/.cargo/bin` is the whole setup; don't fall back to
   `cargo test`. Build budgets, re-measured on a quiet box: cold
   `profiling-fast` bot_ladder ~11 min, an engine-only rebuild of it ~3 min,
   `release` ~20 min, a debug suite build + run ~9 min, one `--decks cube`
   callgrind ~1-2 min. **A whole three-pool A/B is therefore ~10 min when the
   caches are warm**, not the hour the earlier budgets imply — the difference
   is contention, so run the base callgrinds *while* the candidate builds and
   nothing else. **Take `release` once at the tip as the closing gate,
   not per candidate.**

**Compacted at the ninety-first pass from 90 lines** — a paragraph per pass
restating numbers that live one file away, which the header forbids. Add a
pointer, not a paragraph.

## Standing rules for a perf pass — in `PERF.md`

Moved verbatim (555 lines, every rule a refutation with numbers) to `PERF.md`,
under "How to measure". Read it before proposing anything on the perf queue.

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

**A full disk arrives as `rustc` exit 101, which reads exactly like a compiler
error** (ninety-first pass): `cargo test --no-run` failed on `classic_sets`
with a bare "process didn't exit successfully ... (exit status: 101)" and no
diagnostic, because the writes that would have printed one also failed. Check
`df` before debugging the crate. What filled it that run was `target` at 28 GB
— `deps` 19 GB and `incremental` back to 5.3 GB — plus five full `release-fast`
rebuilds under different `RUSTFLAGS`, which cargo never garbage-collects.
`rm -rf target/debug/incremental target/debug/examples` freed 5.8 GB and
`rm -rf target/release-fast` another 7.7 GB, both without a workspace rebuild.

**The audit builds are the other 3 GB, and this run hit 97 % (1.3 GB free).**
`scripts/robustness_grid.sh` fills `target-audit/` (~1.5 GB) and the actor leg
adds `selfplay_train` to the same dir for another ~1.5; on top of that, an
A/B session accumulates copied `profiling-fast` binaries at **215 MB each** in
whatever scratch dir the base/candidate pair lives in. Delete the A/B binaries
as soon as their callgrind dumps are taken (the dumps are 1-3 MB and are the
thing worth keeping), and `rm -rf target-audit` once the grid is green.

**Two cold builds concurrently did NOT OOM at the ninetieth pass** — a `release`
bot_ladder and a `profiling-fast` one in a second worktree ran together to
completion-ish on 15 GB, peaking ~10 GB used with 4 GB free. So the "sequential
builds only" line above holds for a *throughput* reason, not a memory one: load
average hit 8 on 4 cores and each build took roughly twice its solo wall time,
which is worse than running them in series. Size the rule by cores, not by RAM.

**Killing `cargo` does not kill its `rustc` children.** `pkill -f "cargo build
…"` leaves the running `rustc` processes alive, each still holding a core — one
orphan from a cancelled `release` build burned 8 CPU-minutes competing with the
build that replaced it before it was noticed. After cancelling a build, check
`pgrep rustc` and read `/proc/<pid>/cwd` plus `--crate-name` off the cmdline to
tell whose it is; a worktree build and a main-tree build look identical
otherwise.

**A build started before a `git rebase` compiles the pre-rebase source.**
Concurrent sessions land engine commits every few minutes here, so a long
`release` build straddling a rebase silently produces a binary that is not the
tip — and `--bench` run on it is measuring the wrong commit. Rebase first, then
build; if a rebase lands mid-build and touched any crate in the graph, restart
it rather than trusting the artifact.

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
