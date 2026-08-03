//! Scourge (SCG) — Storm, the Decree cycle's cycling triggers, the
//! "turned face up" payoffs and the Dragon Auras (`catalog::sets::scg`).

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn cycle(g: &mut GameState, seat: usize, id: CardId) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle");
    drain_stack(g);
}

fn unmorph(g: &mut GameState, id: CardId) {
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().turn_face_down();
    mana(g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("unmorph");
    drain_stack(g);
}

fn power_of(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).expect("computed").power
}

// ── Vanilla / keyword bodies ────────────────────────────────────────────────

/// Printed stats and keywords, one table over the catalog.
#[test]
fn scg_bodies_have_their_printed_stats() {
    let rows: &[(fn() -> CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::coast_watcher, 1, 1, &[Keyword::Flying, Keyword::Protection(Color::Green)]),
        (catalog::silver_knight, 2, 2, &[Keyword::FirstStrike, Keyword::Protection(Color::Red)]),
        (
            catalog::dragonstalker,
            3,
            3,
            &[Keyword::ProtectionFromCreatureType(crabomination::card::CreatureType::Dragon)],
        ),
        (catalog::goblin_brigand, 2, 2, &[Keyword::MustAttack]),
        (catalog::titanic_bulvox, 7, 4, &[Keyword::Trample]),
        (catalog::noble_templar, 3, 6, &[Keyword::Vigilance]),
        (
            catalog::dragon_tyrant,
            6,
            6,
            &[Keyword::Flying, Keyword::Trample, Keyword::DoubleStrike],
        ),
        (catalog::scornful_egotist, 1, 1, &[]),
    ];
    for (make, p, t, kws) in rows {
        let def = make();
        assert_eq!((def.power, def.toughness), (*p, *t), "{}", def.name);
        for kw in *kws {
            assert!(def.keywords.contains(kw), "{} missing {kw:?}", def.name);
        }
    }
}

// ── Morph-matters ("whenever a permanent is turned face up") ────────────────

/// Aven Farseer grows off an opponent's flip, not just its own.
#[test]
fn aven_farseer_counts_every_flip_on_the_table() {
    let mut g = main_phase();
    let farseer = g.add_card_to_battlefield(0, catalog::aven_farseer());
    let theirs = g.add_card_to_battlefield(1, catalog::titanic_bulvox());
    g.battlefield.iter_mut().find(|c| c.id == theirs).unwrap().turn_face_down();
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::TurnFaceUp { card_id: theirs }).expect("unmorph");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(farseer).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Bonethorn Valesk pings on every flip.
#[test]
fn bonethorn_valesk_pings_on_a_flip() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::bonethorn_valesk());
    let bulvox = g.add_card_to_battlefield(0, catalog::titanic_bulvox());
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Player(1))]));
    unmorph(&mut g, bulvox);
    assert_eq!(g.players[1].life, life - 1);
}

/// Exiled Doomsayer taxes the turn-up cost by {2}.
#[test]
fn exiled_doomsayer_taxes_morph_costs() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::exiled_doomsayer());
    let egotist = g.add_card_to_battlefield(0, catalog::scornful_egotist()); // Morph {U}
    g.battlefield.iter_mut().find(|c| c.id == egotist).unwrap().turn_face_down();
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(
        g.perform_action(GameAction::TurnFaceUp { card_id: egotist }).is_err(),
        "one blue alone can't pay the taxed morph cost"
    );
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::TurnFaceUp { card_id: egotist }).expect("two more generic pays");
}

/// Riptide Survivor's flip is discard two, draw three.
#[test]
fn riptide_survivor_rummages_on_the_flip() {
    let mut g = main_phase();
    let survivor = g.add_card_to_battlefield(0, catalog::riptide_survivor());
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::silver_knight());
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::silver_knight());
    }
    unmorph(&mut g, survivor);
    assert_eq!(g.players[0].hand.len(), 3, "discarded two, drew three");
}

/// Woodcloaker's flip hands out trample.
#[test]
fn woodcloaker_grants_trample_on_the_flip() {
    let mut g = main_phase();
    let cloaker = g.add_card_to_battlefield(0, catalog::woodcloaker());
    let knight = g.add_card_to_battlefield(0, catalog::silver_knight());
    g.decider =
        Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(knight))]));
    unmorph(&mut g, cloaker);
    assert!(g.computed_permanent(knight).unwrap().keywords.contains(&Keyword::Trample));
}

// ── Storm (CR 702.40) ───────────────────────────────────────────────────────

/// Scattershot copies itself once per earlier spell this turn.
#[test]
fn scattershot_storms_off() {
    let mut g = main_phase();
    let target = g.add_card_to_battlefield(1, catalog::dragonstalker()); // 3/3
    g.spells_cast_this_turn = 2;
    let shot = g.add_card_to_hand(0, catalog::scattershot());
    cast(&mut g, 0, shot, Some(Target::Permanent(target)));
    // Original plus two copies = 3 damage on a 3/3.
    assert!(g.battlefield_find(target).is_none(), "the storm count killed it");
}

// ── "Greatest mana value among permanents you control" ──────────────────────

/// Torrent of Fire scales off your biggest permanent.
#[test]
fn torrent_of_fire_scales_with_your_biggest_permanent() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dragon_tyrant()); // mana value 10
    let torrent = g.add_card_to_hand(0, catalog::torrent_of_fire());
    let life = g.players[1].life;
    cast(&mut g, 0, torrent, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 10);
}

/// Dispersal Shield only counters what your board outweighs.
#[test]
fn dispersal_shield_gates_on_mana_value() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::silver_knight()); // mana value 2
    let big = g.add_card_to_hand(1, catalog::dragon_tyrant()); // mana value 10
    let shield = g.add_card_to_hand(0, catalog::dispersal_shield());
    g.active_player_idx = 1;
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: big,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: shield,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast shield");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_some(), "10 > 2, so the counter fizzles");
}

// ── The Dragon Auras ────────────────────────────────────────────────────────

/// Dragon Fangs comes back from the graveyard onto a big arrival.
#[test]
fn dragon_fangs_returns_on_a_six_drop() {
    let mut g = main_phase();
    let fangs = CardId(9401);
    g.players[0]
        .graveyard
        .push(crabomination::card::CardInstance::new(fangs, catalog::dragon_fangs(), 0));
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let tyrant = g.add_card_to_hand(0, catalog::dragon_tyrant());
    cast(&mut g, 0, tyrant, None);
    assert_eq!(g.battlefield_find(fangs).map(|c| c.attached_to), Some(Some(tyrant)));
    assert!(g.computed_permanent(tyrant).unwrap().keywords.contains(&Keyword::Trample));
}

// ── The Decree cycle ────────────────────────────────────────────────────────

/// Decree of Pain wraths and refills.
#[test]
fn decree_of_pain_wraths_and_draws() {
    let mut g = main_phase();
    for seat in [0, 1] {
        g.add_card_to_battlefield(seat, catalog::silver_knight());
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::silver_knight());
    }
    let decree = g.add_card_to_hand(0, catalog::decree_of_pain());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, decree, None);
    assert!(!g.battlefield.iter().any(|c| c.definition.is_creature()), "board wiped");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "two cards for two creatures");
}

/// Cycling Decree of Pain fires the -2/-2 rider instead.
#[test]
fn decree_of_pain_cycles_into_a_minus_two_sweep() {
    let mut g = main_phase();
    let knight = g.add_card_to_battlefield(1, catalog::silver_knight()); // 2/2
    g.add_card_to_library(0, catalog::silver_knight());
    let decree = g.add_card_to_hand(0, catalog::decree_of_pain());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    cycle(&mut g, 0, decree);
    assert!(g.battlefield_find(knight).is_none(), "-2/-2 killed the 2/2");
}

/// Decree of Savagery's cycling rider puts four counters on one creature.
#[test]
fn decree_of_savagery_cycles_into_four_counters() {
    let mut g = main_phase();
    let knight = g.add_card_to_battlefield(0, catalog::silver_knight());
    g.add_card_to_library(0, catalog::silver_knight());
    let decree = g.add_card_to_hand(0, catalog::decree_of_savagery());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(knight)),
    ]));
    cycle(&mut g, 0, decree);
    assert_eq!(g.battlefield_find(knight).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Decree of Silence counters opponents' spells and folds after three.
#[test]
fn decree_of_silence_counters_three_then_dies() {
    let mut g = main_phase();
    let decree = g.add_card_to_battlefield(0, catalog::decree_of_silence());
    g.active_player_idx = 1;
    for _ in 0..3 {
        let spell = g.add_card_to_hand(1, catalog::silver_knight());
        mana(&mut g, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(spell).is_none(), "countered");
    }
    assert!(g.battlefield_find(decree).is_none(), "three depletion counters, sacrificed");
}

/// Stabilizer shuts cycling off for everybody.
#[test]
fn stabilizer_stops_cycling() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::stabilizer());
    let decree = g.add_card_to_hand(0, catalog::wipe_clean());
    mana(&mut g, 0);
    assert!(g.perform_action(GameAction::Cycle { card_id: decree, x_value: None }).is_err());
}

// ── Odds and ends ───────────────────────────────────────────────────────────

/// Dawn Elemental soaks everything.
#[test]
fn dawn_elemental_takes_no_damage() {
    let mut g = main_phase();
    let elem = g.add_card_to_battlefield(0, catalog::dawn_elemental());
    let mut events = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(elem),
        99,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(elem).unwrap().damage, 0);
}

/// Ancient Ooze is as big as the rest of your board costs.
#[test]
fn ancient_ooze_totals_your_other_creatures() {
    let mut g = main_phase();
    let ooze = g.add_card_to_battlefield(0, catalog::ancient_ooze());
    g.add_card_to_battlefield(0, catalog::dragon_tyrant()); // 10
    g.add_card_to_battlefield(0, catalog::silver_knight()); // 2
    g.add_card_to_battlefield(1, catalog::dragon_tyrant()); // theirs — not counted
    assert_eq!(power_of(&g, ooze), 12);
}

/// Clutch of Undeath swings +3/+3 on a Zombie and -3/-3 on anything else.
#[test]
fn clutch_of_undeath_reads_the_hosts_type() {
    let mut g = main_phase();
    let zombie = g.add_card_to_battlefield(0, catalog::bladewings_thrall()); // 3/3 Zombie
    let tyrant = g.add_card_to_battlefield(0, catalog::dragon_tyrant()); // 6/6 Dragon
    for (host, expected) in [(zombie, 6), (tyrant, 3)] {
        let clutch = g.add_card_to_hand(0, catalog::clutch_of_undeath());
        cast(&mut g, 0, clutch, Some(Target::Permanent(host)));
        assert_eq!(power_of(&g, host), expected);
        g.remove_to_graveyard_with_triggers(clutch);
    }
}

/// Cabal Interrogator's {X} strips one of X revealed cards.
#[test]
fn cabal_interrogator_strips_one_of_x() {
    let mut g = main_phase();
    let interrogator = g.add_card_to_battlefield(0, catalog::cabal_interrogator());
    g.clear_sickness(interrogator);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::silver_knight());
    }
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: interrogator,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 2);
}

/// Final Punishment charges the turn's damage again as life loss.
#[test]
fn final_punishment_repeats_the_turns_damage() {
    let mut g = main_phase();
    g.players[1].damage_taken_this_turn = 7;
    let punishment = g.add_card_to_hand(0, catalog::final_punishment());
    let life = g.players[1].life;
    cast(&mut g, 0, punishment, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 7);
}

/// Krosan Drover makes the fatties cheaper.
#[test]
fn krosan_drover_discounts_the_fatties() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::krosan_drover());
    let tyrant = g.add_card_to_hand(0, catalog::dragon_tyrant()); // 8RR, discounted to 6RR
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: tyrant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("a 6+ drop costs two less");
}

/// Nefashu's attack shrinks up to five creatures.
#[test]
fn nefashu_shrinks_on_attack() {
    let mut g = main_phase();
    let nefashu = g.add_card_to_battlefield(0, catalog::nefashu());
    let a = g.add_card_to_battlefield(1, catalog::silver_knight());
    g.clear_sickness(nefashu);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(a))]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: nefashu,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(power_of(&g, a), 1, "-1/-1");
}

/// Mercurial Kite's hits stay tapped through the next untap step.
#[test]
fn mercurial_kite_locks_down_its_victim() {
    let mut g = main_phase();
    let kite = g.add_card_to_battlefield(0, catalog::mercurial_kite());
    let blocker = g.add_card_to_battlefield(1, catalog::dragonstalker()); // 3/3 flier
    g.clear_sickness(kite);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kite,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.advance_step(vec![]).expect("to blockers");
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, kite)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.advance_step(vec![]).expect("advance");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(blocker).unwrap().tapped, "tapped by the Kite");
}

/// Unspeakable Symbol turns life into counters.
#[test]
fn unspeakable_symbol_pays_life_for_counters() {
    let mut g = main_phase();
    let symbol = g.add_card_to_battlefield(0, catalog::unspeakable_symbol());
    let knight = g.add_card_to_battlefield(0, catalog::silver_knight());
    let life = g.players[0].life;
    activate(&mut g, 0, symbol, 0, Some(Target::Permanent(knight)));
    assert_eq!(g.players[0].life, life - 3);
    assert_eq!(g.battlefield_find(knight).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}
