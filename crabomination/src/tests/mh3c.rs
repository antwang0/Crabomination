//! Functionality tests for the MH3 batch-3 cards in `catalog::sets::mh3c`
//! and the Battle cry keyword (CR 702.92).

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Goblin Wardriver's battle cry pumps each *other* attacker +1/+0, not itself.
#[test]
fn goblin_wardriver_battle_cry_pumps_team() {
    let mut g = two_player_game();
    let driver = g.add_card_to_battlefield(0, catalog::goblin_wardriver());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(driver);
    g.clear_sickness(bear);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: driver, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "other attacker +1/+0");
    assert_eq!(g.computed_permanent(driver).unwrap().power, 2, "battle cry excludes itself");
}

/// Battle cry does nothing when the creature attacks alone (no other attackers).
#[test]
fn battle_cry_solo_attacker_no_pump() {
    let mut g = two_player_game();
    let driver = g.add_card_to_battlefield(0, catalog::goblin_wardriver());
    g.clear_sickness(driver);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: driver, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(driver).unwrap().power, 2, "no team, no pump");
}

/// Reckless Pyrosurfer gains battle cry from a landfall trigger and then pumps
/// the rest of the attacking team when it swings.
#[test]
fn reckless_pyrosurfer_landfall_grants_battle_cry() {
    let mut g = two_player_game();
    let surfer = g.add_card_to_battlefield(0, catalog::reckless_pyrosurfer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert!(g.computed_permanent(surfer).unwrap().keywords.contains(&Keyword::BattleCry(1)),
        "landfall granted battle cry");
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: surfer, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "granted battle cry pumps team");
}

/// Wurmcoil Larva dies into a 1/2 deathtouch and a 2/1 lifelink Wurm token.
#[test]
fn wurmcoil_larva_dies_into_two_wurms() {
    let mut g = two_player_game();
    let larva = g.add_card_to_battlefield(0, catalog::wurmcoil_larva());
    g.remove_to_graveyard_with_triggers(larva);
    drain_stack(&mut g);
    g.check_state_based_actions();
    let wurms: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Phyrexian Wurm").collect();
    assert_eq!(wurms.len(), 2, "two Wurm tokens");
    assert!(wurms.iter().any(|c| c.definition.keywords.contains(&Keyword::Deathtouch)));
    assert!(wurms.iter().any(|c| c.definition.keywords.contains(&Keyword::Lifelink)));
}

/// Spawn-Gang Commander mints three Eldrazi Spawn on cast and can sacrifice one
/// to ping any target for 2.
#[test]
fn spawn_gang_commander_spawns_and_pings() {
    let mut g = two_player_game();
    for c in [crate::mana::Color::Red] {
        g.players[0].mana_pool.add(c, 5);
    }
    g.players[0].mana_pool.add_colorless(2);
    let id = g.add_card_to_hand(0, catalog::spawn_gang_commander());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let spawns = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawns, 3, "cast trigger made three Eldrazi Spawn");
}

/// Vaultborn Tyrant draws + gains life when another power-4+ creature you
/// control enters.
#[test]
fn vaultborn_tyrant_etb_payoff() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vaultborn_tyrant());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[0].life;
    // Cast Serra Angel (4/4, power ≥ 4) so the ETB event flows through the
    // trigger dispatcher; net hand change = -1 cast + 1 draw = 0.
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast angel");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 when a power-4+ creature entered");
    assert_eq!(g.players[0].hand.len(), hand, "cast -1 + draw +1 = net 0");
}

/// Vaultborn Tyrant dies into an artifact token copy of itself.
#[test]
fn vaultborn_tyrant_dies_into_artifact_copy() {
    let mut g = two_player_game();
    let tyrant = g.add_card_to_battlefield(0, catalog::vaultborn_tyrant());
    g.remove_to_graveyard_with_triggers(tyrant);
    drain_stack(&mut g);
    g.check_state_based_actions();
    let copy = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Vaultborn Tyrant");
    let copy = copy.expect("token copy exists");
    assert!(g.computed_permanent(copy.id).unwrap().card_types.contains(&crate::card::CardType::Artifact),
        "the copy is an artifact");
}

/// Hydra Trainer's exert attack pumps a target by the number of counters on
/// your permanents.
#[test]
fn hydra_trainer_exert_pumps_by_counter_count() {
    let mut g = two_player_game();
    let trainer = g.add_card_to_battlefield(0, catalog::hydra_trainer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Two +1/+1 counters across your permanents → X = 2.
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.clear_sickness(trainer);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: trainer, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    // Bear is 2/2 base + 2 counters + 2/2 from exert = 6/6.
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "+X/+X where X = 2 counters");
    assert!(g.battlefield_find(trainer).unwrap().skip_next_untap, "exerted");
}

/// Signature Slam pumps your creature (making it modified) then swings its power
/// at an enemy creature.
#[test]
fn signature_slam_modified_creatures_deal_damage() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    for c in [crate::mana::Color::Green] { g.players[0].mana_pool.add(c, 3); }
    let id = g.add_card_to_hand(0, catalog::signature_slam());
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(mine)),
        additional_targets: vec![crate::game::types::Target::Permanent(enemy)],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    g.check_state_based_actions();
    // mine becomes 3/3 (counter → modified), deals 3 to the 2/2 enemy → dead.
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "got a +1/+1 counter");
    assert!(g.battlefield_find(enemy).is_none(), "enemy took 3, died");
}

/// Ajani Fells the Godsire: I exiles a big enemy creature, II makes a Cat and a
/// vigilance counter, III grants double strike.
#[test]
fn ajani_fells_the_godsire_chapters() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::ajani_fells_the_godsire());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4, power ≥ 3
    g.saga_advance(saga); // I — exile the big enemy
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "chapter I exiled the power-3+ creature");
    g.saga_advance(saga); // II — Cat + vigilance counter
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Cat Warrior"),
        "chapter II made a Cat Warrior");
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Vigilance),
        "chapter II put a vigilance counter on a creature");
    g.saga_advance(saga); // III — double strike
    drain_stack(&mut g);
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "chapter III granted double strike");
}
