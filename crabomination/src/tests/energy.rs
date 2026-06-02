//! Functionality tests for the energy ({E}) resource system
//! (`Effect::AddEnergy` / `Effect::PayEnergy`, `Player.energy`) and the
//! Kaladesh (`catalog::sets::kld`) cards built on it.

use crate::card::CounterType;
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn cast_creature(g: &mut GameState, id: crate::card::CardId) {
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(g);
}

#[test]
fn attune_with_aether_fetches_basic_and_gives_two_energy() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    // Script the search to grab the Forest (AutoDecider would decline).
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::attune_with_aether());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 2, "you get {{E}}{{E}}");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest),
        "basic land tutored to hand");
}

#[test]
fn rogue_refiner_etb_draws_and_gives_two_energy() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::rogue_refiner());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    cast_creature(&mut g, id);
    // -1 cast + 1 draw = net same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB drew a card");
    assert_eq!(g.players[0].energy, 2);
}

#[test]
fn longtusk_cub_pays_energy_for_counter() {
    let mut g = two_player_game();
    let cub = g.add_card_to_battlefield(0, catalog::longtusk_cub());
    g.clear_sickness(cub);
    g.players[0].energy = 3;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cub, ability_index: 0, target: None, x_value: None,
    }).expect("activatable with 3 energy");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "spent {{E}}{{E}}{{E}}");
    assert_eq!(g.battlefield_find(cub).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn longtusk_cub_combat_damage_gives_two_energy() {
    let mut g = two_player_game();
    let cub = g.add_card_to_battlefield(0, catalog::longtusk_cub());
    g.clear_sickness(cub);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cub, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "combat damage to a player gives {{E}}{{E}}");
}

#[test]
fn bristling_hydra_etb_energy_then_pays_for_counter_and_hexproof() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bristling_hydra());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 3, "ETB grants {{E}}{{E}}{{E}}");
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(c.has_keyword(&Keyword::Hexproof), "gained hexproof EOT");
}

#[test]
fn pay_energy_without_enough_is_a_noop() {
    // Longtusk Cub with only 2 energy can't pay {E}{E}{E} → no counter, no spend.
    let mut g = two_player_game();
    let cub = g.add_card_to_battlefield(0, catalog::longtusk_cub());
    g.clear_sickness(cub);
    g.players[0].energy = 2;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cub, ability_index: 0, target: None, x_value: None,
    }).expect("activation itself is free");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "insufficient energy → not spent");
    assert_eq!(g.battlefield_find(cub).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn glint_sleeve_siphoner_attack_gives_one_energy() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::glint_sleeve_siphoner());
    g.clear_sickness(s);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: s, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 1, "attacking gives {{E}}");
}

#[test]
fn servant_of_the_conduit_etb_energy_and_taps_for_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::servant_of_the_conduit());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 2, "ETB grants {{E}}{{E}}");
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("mana ability");
    let pool = g.players[0].mana_pool.total();
    assert!(pool >= 1, "tapped for a mana");
}

#[test]
fn dynavolt_tower_pays_mana_and_energy_to_burn() {
    let mut g = two_player_game();
    let tower = g.add_card_to_battlefield(0, catalog::dynavolt_tower());
    g.players[0].energy = 5;
    g.players[0].mana_pool.add_colorless(5);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tower, ability_index: 0,
        target: Some(crate::game::types::Target::Player(1)),
        x_value: None,
    }).expect("activatable with {5} + 5 energy");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "deals 4 to any target");
    assert_eq!(g.players[0].energy, 0, "spent five {{E}}");
}

#[test]
fn dynavolt_tower_gains_energy_on_instant_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dynavolt_tower());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "casting an instant gives {{E}}{{E}}");
}

#[test]
fn aether_swooper_attack_pays_energy_for_thopter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aether_swooper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 2, "ETB grants {{E}}{{E}}");
    g.clear_sickness(id);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "paid {{E}}{{E}} on attack");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Thopter"),
        "minted a Thopter");
}

#[test]
fn sage_of_shailas_claim_etb_three_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sage_of_shailas_claim());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 3);
}

#[test]
fn live_fast_draws_loses_life_and_energy() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::live_fast());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "draw 2 (after -1 cast)");
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(g.players[0].energy, 2);
}

#[test]
fn highspire_infusion_pumps_and_gives_energy() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::highspire_infusion());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (5, 5), "+3/+3");
    assert_eq!(g.players[0].energy, 2);
}

#[test]
fn glimmer_of_genius_draws_two_and_energy() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::glimmer_of_genius());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len();
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "draw 2");
    assert_eq!(g.players[0].energy, 2);
}

#[test]
fn woodweavers_puzzleknot_etb_and_sac_payoff() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::woodweavers_puzzleknot());
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 3, "ETB {{E}}{{E}}{{E}}");
    assert_eq!(g.players[0].life, life + 3, "ETB gain 3");
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("sac ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 6, "sac adds {{E}}{{E}}{{E}}");
    assert_eq!(g.players[0].life, life + 6);
    assert!(g.battlefield_find(id).is_none(), "sacrificed");
}

#[test]
fn glassblowers_puzzleknot_etb_energy_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::glassblowers_puzzleknot());
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 2);
    // -1 cast + 1 ETB draw = net same.
    assert_eq!(g.players[0].hand.len(), hand);
}

#[test]
fn aether_poisoner_etb_and_combat_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aether_poisoner());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 1, "ETB {{E}}");
    g.clear_sickness(id);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "combat damage gives another {{E}}");
}

#[test]
fn aetherstream_leopard_pays_four_energy_for_unblockable() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aetherstream_leopard());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_creature(&mut g, id);
    assert_eq!(g.players[0].energy, 2);
    g.players[0].energy = 4;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Unblockable));
}

#[test]
fn riparian_tiger_pays_two_energy_for_hexproof() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::riparian_tiger());
    g.players[0].energy = 2;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Hexproof));
}

#[test]
fn voltaic_brawler_attack_pays_energy_to_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::voltaic_brawler());
    g.clear_sickness(id);
    g.players[0].energy = 2;
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 2), "paid {{E}}{{E}} for +1/+1");
    assert_eq!(g.players[0].energy, 0);
}

/// CR 107.14 — "To pay {E}, a player removes one energy counter from
/// themselves." A creature paying {E}{E}{E} removes exactly three.
#[test]
fn cr_107_14_paying_energy_removes_counters() {
    let mut g = two_player_game();
    let cub = g.add_card_to_battlefield(0, catalog::longtusk_cub());
    g.clear_sickness(cub);
    g.players[0].energy = 5;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cub, ability_index: 0, target: None, x_value: None,
    }).expect("activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "paid {{E}}{{E}}{{E}} of 5 → 2 remain");
}

#[test]
fn aether_hub_etb_gives_one_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aether_hub());
    g.perform_action(GameAction::PlayLand(id)).expect("land playable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 1, "ETB gives {{E}}");
    assert!(g.battlefield_find(id).is_some());
}
