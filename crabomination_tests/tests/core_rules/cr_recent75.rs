//! CR conformance: 502.3 untap restrictions, 508.1g attack costs, 614.9
//! redirection ordering, 615.1 partial prevention, and 120.3 damage memory.

use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// CR 502.3 — an "attacked during your last turn" untap gate reads the
/// previous *own* turn, so the creature untaps normally the cycle after.
#[test]
fn cr_502_3_attacked_last_turn_untap_gate_is_one_turn_only() {
    let mut g = main_phase();
    let sled = g.add_card_to_battlefield(0, catalog::goblin_rock_sled());
    g.clear_sickness(sled);
    g.add_card_to_battlefield(1, catalog::mountain());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: sled, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);

    g.do_untap(); // the turn after the attack: still down
    assert!(g.battlefield_find(sled).expect("sled").tapped);
    g.do_untap(); // and the one after that: free again
    assert!(!g.battlefield_find(sled).expect("sled").tapped);
}

/// CR 502.3 — the gate is read off the *computed* keywords, so an Aura's
/// grant locks a creature that doesn't have the keyword printed.
#[test]
fn cr_502_3_granted_untap_gate_locks_the_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let kelp = g.add_card_to_battlefield(1, catalog::tangle_kelp());
    g.battlefield_find_mut(kelp).expect("kelp").attached_to = Some(bear);
    g.battlefield_find_mut(bear).expect("bear").tapped = false;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.do_untap();
    assert!(g.battlefield_find(bear).expect("bear").tapped);
}

/// CR 508.1g — a sacrifice attack cost is paid from a shared pool, so two
/// attackers that each want two Islands need four.
#[test]
fn cr_508_1g_sacrifice_attack_costs_share_one_pool() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::leviathan());
    let b = g.add_card_to_battlefield(0, catalog::leviathan());
    for id in [a, b] {
        g.clear_sickness(id);
        g.battlefield_find_mut(id).expect("leviathan").tapped = false;
    }
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ])
        .is_err(),
        "three Islands can't pay for two attackers",
    );
    // Nothing was spent on the rejected declaration.
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 3);
}

/// CR 614.9 / 614.5 — a redirect applies once per damage event; the
/// redirected damage isn't redirected again.
#[test]
fn cr_614_9_creature_damage_redirect_applies_once() {
    let mut g = main_phase();
    let blood = g.add_card_to_hand(0, catalog::blood_of_the_martyr());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: blood,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.deal_damage_to_from(EntityRef::Permanent(bear), 2, None, &mut vec![]);
    assert_eq!(g.battlefield_find(bear).expect("bear").damage, 0);
    assert_eq!(g.players[0].life, 18);
}

/// CR 615.1 — a "prevent half, rounded down" shield only soaks its share;
/// the remainder is still dealt. CR 615.13: it fires once.
#[test]
fn cr_615_1_half_prevention_soaks_only_its_share() {
    let mut g = main_phase();
    let sphere = g.add_card_to_battlefield(0, catalog::dark_sphere());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sphere,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    g.deal_damage_to_from(EntityRef::Player(0), 7, None, &mut vec![]);
    assert_eq!(g.players[0].life, 16, "7 damage, 3 prevented");
    g.deal_damage_to_from(EntityRef::Player(0), 7, None, &mut vec![]);
    assert_eq!(g.players[0].life, 9, "the shield is spent");
}

/// CR 120.3 — a source remembers what it has damaged. The record is
/// per-source and survives across turns.
#[test]
fn cr_120_3_a_source_remembers_what_it_damaged() {
    let mut g = main_phase();
    let fallen = g.add_card_to_battlefield(0, catalog::the_fallen());
    let other = g.add_card_to_battlefield(0, catalog::the_fallen());
    g.deal_damage_to_from(EntityRef::Player(1), 1, Some(fallen), &mut vec![]);
    assert_eq!(g.battlefield_find(fallen).expect("fallen").damaged_players_this_game, vec![1]);
    assert!(g.battlefield_find(other).expect("other").damaged_players_this_game.is_empty());
    // A second hit on the same seat doesn't duplicate the entry.
    g.deal_damage_to_from(EntityRef::Player(1), 1, Some(fallen), &mut vec![]);
    assert_eq!(g.battlefield_find(fallen).expect("fallen").damaged_players_this_game, vec![1]);
}

/// CR 115.6 — a "can't be the target of spells unless…" restriction doesn't
/// stop abilities from targeting.
#[test]
fn cr_115_6_spell_only_target_restriction_lets_abilities_through() {
    let mut g = main_phase();
    let lurker = g.add_card_to_battlefield(1, catalog::lurker());
    let cage = g.add_card_to_battlefield(0, catalog::barls_cage());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cage,
        ability_index: 0,
        target: Some(Target::Permanent(lurker)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("an ability may target it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lurker).expect("lurker").skip_next_untap);
}

/// CR 104.4b / 732.4 — a loop of mandatory triggered abilities with no way to
/// stop draws the game. A state-neutral trigger that keeps re-resolving never
/// moves the fingerprint the watchdog samples, so the watchdog fires.
#[test]
fn cr_104_4b_mandatory_trigger_loop_draws_the_game() {
    use crabomination::effect::Effect;
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..(GameState::MANDATORY_LOOP_DRAW_REPEATS + 2) {
        if g.game_over.is_some() {
            break;
        }
        g.stack.push(TriggerPush::new(src, 0, Effect::Noop).build());
        g.resolve_top_of_stack().expect("resolve");
    }
    assert_eq!(g.game_over, Some(None), "mandatory loop is a draw");
}

/// The watchdog must not fire on a *progressing* trigger chain — one that
/// changes the game state each time still resolves normally.
#[test]
fn cr_104_4b_progressing_trigger_chain_is_not_a_draw() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..(GameState::MANDATORY_LOOP_DRAW_REPEATS + 2) {
        g.stack.push(
            TriggerPush::new(src, 0, Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            })
            .build(),
        );
        g.resolve_top_of_stack().expect("resolve");
    }
    assert!(g.game_over.is_none(), "a chain that gains life each time is progress");
}

/// CR 104.4b / 732.4 — a loop whose *period* is longer than one resolution
/// (two states alternating) is still a mandatory loop. The watchdog anchors a
/// fingerprint and counts returns to it, so a period-2 chain draws too.
#[test]
fn cr_104_4b_period_two_trigger_loop_draws_the_game() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gain = Effect::GainLife { who: Selector::You, amount: Value::ONE };
    let lose = Effect::LoseLife { who: Selector::You, amount: Value::ONE };
    for i in 0..(2 * GameState::MANDATORY_LOOP_DRAW_REPEATS + 4) {
        if g.game_over.is_some() {
            break;
        }
        let e = if i % 2 == 0 { gain.clone() } else { lose.clone() };
        g.stack.push(TriggerPush::new(src, 0, e).build());
        g.resolve_top_of_stack().expect("resolve");
    }
    assert_eq!(g.game_over, Some(None), "a period-2 mandatory loop is a draw");
}

/// The loop the 2026-09-09 fresh-seed sweep found, played out: two Portable
/// Holes and a *mandatory* "exile target artifact" ETB (Leonin Relic-Warder
/// before its "you may" was restored). Hole A exiles Hole B, whose return link
/// brings the Warder back; the Warder's ETB has only its controller's own
/// Hole A to exile, which returns Hole B, whose ETB exiles the Warder, which
/// returns Hole A — period 3, forever. The old watch reset on every step and
/// the game ran to the action cap; it must end as a draw (CR 732.4).
#[test]
fn cr_732_4_two_portable_holes_and_a_mandatory_warder_end_as_a_draw() {
    use crabomination::card::{CardDefinition, CardType, ExileReturnZone, SelectionRequirement};
    use crabomination::effect::Effect;
    use crabomination::effect::shortcut::{etb, target_filtered};
    let warder = CardDefinition {
        name: "Mandatory Relic-Warder",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    };
    let mut g = main_phase();
    let w = g.add_card_to_battlefield(0, warder);
    // p1's Hole exiles the Warder (the only nonland MV<=2 permanent p0 has).
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    let hole_b = g.add_card_to_hand(1, catalog::portable_hole());
    g.players[1].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: hole_b,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Hole B cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == w), "the Warder is under Hole B");
    // p0's Hole exiles Hole B, and the loop begins.
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let hole_a = g.add_card_to_hand(0, catalog::portable_hole());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: hole_a,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Hole A cast");
    let mut fuel = 2 * 3 * (GameState::MANDATORY_LOOP_DRAW_REPEATS as usize + 4) + 50;
    while g.game_over.is_none() && fuel > 0 {
        g.perform_action(GameAction::PassPriority).expect("pass");
        fuel -= 1;
    }
    assert_eq!(g.game_over, Some(None), "the period-3 loop is a draw, not an action cap");
    assert!(fuel > 0, "the draw arrives inside the watchdog's budget");
}

/// CR 104.4 — a loop whose period is a whole TURN is a draw too, and the
/// resolution watchdog above cannot see it: that one samples after a trigger
/// resolution and its digest carries the turn number, which a turn-long loop
/// moves on every cycle.
///
/// The board that asked for it, from the 2026-09-12 fresh-seed sweep (`cube`
/// 1069, and `cube` 1018 before it): a one-card library holding Beacon of
/// Immortality. Draw it, cast it, double your life, shuffle it back — the
/// library never empties, the life total saturates at the ceiling, and from
/// then on neither seat can be killed or decked. Both seats did it from turn
/// 81 to the 6,000-action cap at turn 259.
///
/// Here the same shape without the Beacon: nothing on the board but a bear
/// whose static makes every player skip their draw step, so no zone, life
/// total or permanent moves from one turn to the next.
#[test]
fn cr_104_4_no_progress_turn_loop_draws_the_game() {
    use crabomination::card::{StaticAbility, StaticEffect};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let mut def = catalog::grizzly_bears();
    def.static_abilities = vec![StaticAbility {
        description: "Players skip their draw steps.",
        effect: StaticEffect::SkipStep { step: TurnStep::Draw, all_players: true },
    }];
    g.add_card_to_battlefield(0, def);
    for _ in 0..8_000 {
        if g.game_over.is_some() {
            break;
        }
        g.advance_step(Vec::new()).expect("advance");
    }
    assert_eq!(g.game_over, Some(None), "a turn that repeats forever is a draw");
    // The watch starts at `NO_PROGRESS_WATCH_FROM_TURN` and samples one turn
    // in `NO_PROGRESS_SAMPLE_EVERY`, so the draw lands a couple of periods
    // past `+ SAMPLE_EVERY * DRAW_REPEATS` — not at the 50,000-action cap.
    let ceiling = GameState::NO_PROGRESS_WATCH_FROM_TURN
        + GameState::NO_PROGRESS_SAMPLE_EVERY
            * (GameState::NO_PROGRESS_DRAW_REPEATS + GameState::NO_PROGRESS_MAX_PERIOD + 2);
    assert!(g.turn_number < ceiling, "drew too late: turn {}", g.turn_number);
}

/// The turn watch must not fire on a game that is getting somewhere. The same
/// board with the draw step left alone mills a card a seat a turn, so the
/// library size moves every cycle and the watch never anchors.
#[test]
fn cr_104_4_a_turn_that_draws_a_card_is_not_a_no_progress_loop() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for seat in 0..2 {
        for _ in 0..300 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    // Run twice as long as the loop above took to draw, then stop — the
    // libraries are finite and decking out is a real ending, not this watch.
    let until = GameState::NO_PROGRESS_WATCH_FROM_TURN
        + GameState::NO_PROGRESS_SAMPLE_EVERY
            * (GameState::NO_PROGRESS_DRAW_REPEATS + GameState::NO_PROGRESS_MAX_PERIOD + 4);
    for _ in 0..8_000 {
        if g.game_over.is_some() || g.turn_number > until {
            break;
        }
        g.advance_step(Vec::new()).expect("advance");
    }
    assert!(g.turn_number > until, "stopped early at turn {}", g.turn_number);
    assert!(g.game_over.is_none(), "drawing a card every turn is progress");
}

/// CR 104.4 — the turn watch's digest, tested as a FIELD LIST.
///
/// ⚠ **WHAT IS OUT OF THIS DIGEST IS WHY `all` 1159 RAN TO TURN 18,202.**
/// `CRAB_PROGRESS_WATCH=2000` printed every quantity it reads on that board
/// and the sampled stream was two states that differed in **one permanent's
/// tap** — the bot taking a 33rd land for the extort in bursts. A tap the
/// next untap step takes back is not a game getting anywhere, so it is out;
/// anything a turn can move and keep is in, and the three counters that have
/// no other witness (energy, experience, the dungeon room) are in because a
/// board can advance through them while every zone size stands still.
///
/// The same field is IN the other two digests, which sample within a turn
/// where a tap is a cost paid. Only `progress_fingerprint` drops it.
#[test]
fn cr_104_4_the_turn_digest_reads_progress_and_not_bookkeeping() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let base = g.progress_fingerprint();

    // Out: a tap, and a life total the game can no longer be decided by.
    g.battlefield_find_mut(id).expect("permanent").tapped = true;
    assert_eq!(g.progress_fingerprint(), base, "a tap is not progress across a turn");
    g.players[0].life = i32::MAX;
    let saturated = g.progress_fingerprint();
    g.players[0].life = i32::MAX - 1_000;
    assert_eq!(g.progress_fingerprint(), saturated, "a drained saturated life is not progress");

    // In: everything a turn can move and keep. Each is asserted on its own, so
    // a field that stops being read names itself instead of hiding behind the
    // next one.
    macro_rules! moves {
        ($what:literal, $body:expr) => {{
            let before = g.progress_fingerprint();
            $body;
            assert_ne!(g.progress_fingerprint(), before, "{} must be progress", $what);
        }};
    }
    moves!("a life total inside the band", g.players[0].life = 20);
    moves!("damage on a permanent", g.battlefield_find_mut(id).unwrap().damage = 1);
    moves!(
        "a counter on a permanent",
        g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 1)
    );
    moves!("a card in hand", g.add_card_to_hand(0, catalog::mountain()));
    moves!("a card in the library", g.add_card_to_library(0, catalog::mountain()));
    moves!("a permanent entering", g.add_card_to_battlefield(1, catalog::mountain()));
    moves!("poison", g.players[1].poison_counters = 1);
    moves!("energy", g.players[1].energy = 1);
    moves!("an experience counter", g.players[1].experience = 1);
}

/// Feed a digest stream to the turn watch's state machine and report how many
/// samples it took to declare the draw, if it ever did.
fn samples_to_draw(stream: impl IntoIterator<Item = u64>) -> Option<u32> {
    let mut watch = (0u64, 0u32, 0u32);
    for (i, fp) in stream.into_iter().enumerate() {
        let (next, draw) = GameState::no_progress_step(watch, fp);
        watch = next;
        if draw {
            return Some(i as u32 + 1);
        }
    }
    None
}

/// CR 104.4 — what the turn watch's three constants actually buy, priced on a
/// digest stream rather than on a board.
///
/// ⚠ **THIS IS THE TEST THE CAPPED GAMES ASKED FOR.** Whether the watch fires
/// is a property of the digest *stream*, and the streams that defeat it are
/// not reachable from any board small enough to write down: `all` 1159 took
/// 2,270 turns and a whole deck pool to produce one. Four lines here say the
/// same thing, so the next run can price a constant in seconds instead of in
/// a forty-minute ladder leg.
///
/// The three properties, in the order they matter:
///
/// * A stream that never moves draws on the `NO_PROGRESS_DRAW_REPEATS`-th
///   *return*, so one sample later than that — the first sample is the anchor,
///   not a repeat.
/// * A miss does not cost the count. A loop whose period is longer than one
///   sampling interval has to be allowed to come back, and an excursion of up
///   to `NO_PROGRESS_MAX_PERIOD` samples keeps everything it had.
/// * **An excursion one sample longer than that costs the whole count**, and
///   that is the cliff. A game that returns to one state but takes a long
///   excursion once per cycle never draws, however many turns it is given —
///   the count restarts every cycle and `NO_PROGRESS_DRAW_REPEATS` is never
///   reached. It is not a slow draw; it is no draw at all.
#[test]
fn cr_104_4_the_no_progress_watch_prices_its_excursions() {
    let repeats = GameState::NO_PROGRESS_DRAW_REPEATS;
    let patience = GameState::NO_PROGRESS_MAX_PERIOD;

    // A frozen game: anchor on the first sample, draw on the last return.
    assert_eq!(
        samples_to_draw(std::iter::repeat_n(0xA11CEu64, 4 * repeats as usize)),
        Some(repeats + 1),
        "a stream that never moves draws on the {repeats}th return",
    );

    // An excursion of exactly `NO_PROGRESS_MAX_PERIOD` samples is free: the
    // draw lands `patience` samples later than the frozen stream's and not a
    // repeat later, so nothing was forgotten while the game was away.
    let with_excursion = |away: u32| {
        let mut s = vec![0xA11CEu64; 5];
        s.extend((0..away).map(u64::from));
        s.extend(std::iter::repeat_n(0xA11CEu64, 4 * repeats as usize));
        s
    };
    assert_eq!(
        samples_to_draw(with_excursion(patience)),
        Some(repeats + 1 + patience),
        "an excursion of {patience} samples keeps the count",
    );

    // One sample longer and the anchor is abandoned, count and all: the draw
    // is late by everything that came before it.
    assert!(
        samples_to_draw(with_excursion(patience + 1)).unwrap() > repeats + 1 + patience,
        "an excursion of {} samples must cost the count",
        patience + 1,
    );

    // ⚠ The cliff, and the reason a real unwinnable board can run for ever:
    // repeat that excursion once a cycle and the watch never gets there. The
    // stream below returns to one state 5 times in every cycle — 100,000
    // samples, 500,000 turns at `NO_PROGRESS_SAMPLE_EVERY` — and draws never.
    let mut cyclic = Vec::with_capacity(100_000);
    for cycle in 0..(100_000 / (5 + patience as usize + 1)) {
        cyclic.extend(std::iter::repeat_n(0xA11CEu64, 5));
        // Distinct every cycle, so the excursion never anchors either.
        cyclic.extend((0..=patience).map(|i| 1 + cycle as u64 * 64 + u64::from(i)));
    }
    assert_eq!(
        samples_to_draw(cyclic),
        None,
        "a long excursion once a cycle costs the count every cycle",
    );
}

/// CR 104.4 — a life total past the saturation band is a STATE, not a number,
/// and moving it by one is not progress.
///
/// ⚠ **THIS RAN TO THE ACTION CAP AT TURN 2,270.** `all` seed 1159 of the
/// 2026-09-12 fresh-seed sweep is the Beacon of Immortality board above with
/// one addition: a Basilica Screecher, whose extort takes 1 life off a seat
/// already at `i32::MAX` and gives it to the other, and the next Beacon doubles
/// it back. Both seats are unkillable and the game cannot end — but `p.life`
/// moved every sample, so the turn watch reached `repeats 2/12`, re-anchored,
/// and never reached the one verdict such a game has. It was the first cap in
/// the sweep's history to survive the 50,000-action re-run.
///
/// The digest clamps a life above `SCALE_CEILING * 1_000` — ten million, the
/// same band `cap_diagnosis` calls saturated and a total no ordinary game
/// reaches — so the drift stops counting. Two seats that differ only up there
/// differ in nothing that can end the game. Without the clamp this board runs
/// past turn 5,700 and never draws.
///
/// ⚠ **THE CLAMP IS HALF OF `all` 1159 AND THE TAP IS THE OTHER HALF.** With
/// only the clamp that cell still read `cap 2`; `CRAB_PROGRESS_WATCH=2000`
/// then printed the digest's whole field list on the capped game and the
/// second half named itself — see
/// `cr_104_4_the_turn_digest_reads_progress_and_not_bookkeeping`. The guess on
/// record here, that the library "toggles 1/0 depending on whether the Beacon
/// is on the stack", was wrong: `l1` on every one of the 271 sampled turns.
/// This test pins the clamp; that one pins the tap.
#[test]
fn cr_104_4_a_drained_saturated_life_is_still_no_progress() {
    use crabomination::card::{StaticAbility, StaticEffect, TriggeredAbility};
    use crabomination::effect::{
        Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value,
    };
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    // Neither seat can be killed, and neither can be decked.
    for seat in 0..2 {
        g.players[seat].life = i32::MAX;
    }
    let mut def = catalog::grizzly_bears();
    def.static_abilities = vec![StaticAbility {
        description: "Players skip their draw steps.",
        effect: StaticEffect::SkipStep { step: TurnStep::Draw, all_players: true },
    }];
    // …and one life a turn leaves the other seat, for ever.
    def.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::Upkeep),
            EventScope::YourControl,
        ),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        },
    }];
    g.add_card_to_battlefield(0, def);
    for _ in 0..40_000 {
        if g.game_over.is_some() {
            break;
        }
        g.advance_step(Vec::new()).expect("advance");
        while !g.stack.is_empty() {
            let _ = g.resolve_top_of_stack();
        }
    }
    assert_eq!(g.game_over, Some(None), "an unkillable board is a draw");
    assert!(
        g.players[1].life < i32::MAX,
        "the drain really ran ({} life)",
        g.players[1].life,
    );
    let ceiling = GameState::NO_PROGRESS_WATCH_FROM_TURN
        + GameState::NO_PROGRESS_SAMPLE_EVERY
            * (GameState::NO_PROGRESS_DRAW_REPEATS + GameState::NO_PROGRESS_MAX_PERIOD + 2);
    assert!(g.turn_number < ceiling, "drew too late: turn {}", g.turn_number);
}
