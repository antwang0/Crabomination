//! Functionality tests for `catalog::sets::decks::recent76`.

use crabomination::card::{CounterType, CreatureType, EventKind, Keyword};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

fn cast_aura_on(g: &mut GameState, aura: crabomination::card::CardDefinition, host: CardId, pips: &[(Color, u32)]) {
    let id = g.add_card_to_hand(0, aura);
    for (c, n) in pips {
        g.players[0].mana_pool.add(*c, *n);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(host)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(g);
}

#[test]
fn giant_strength_pumps_plus_two_plus_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_aura_on(&mut g, catalog::giant_strength(), bear, &[(Color::Red, 2)]);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (4, 4), "2/2 → 4/4");
}

#[test]
fn web_grants_toughness_and_reach() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_aura_on(&mut g, catalog::web(), bear, &[(Color::Green, 1)]);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (2, 4), "+0/+2");
    assert!(p.keywords.contains(&Keyword::Reach), "gained reach");
}

#[test]
fn firebreathing_grants_pump_ability_to_host() {
    // CR 604.3 — the Aura grants the enchanted creature an activated ability.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_aura_on(&mut g, catalog::firebreathing(), bear, &[(Color::Red, 1)]);
    let granted = g.granted_abilities_for(bear);
    assert_eq!(granted.len(), 1, "host gained the firebreathing ability");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate granted {R}: +1/+0");
    drain_stack(&mut g);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (3, 2), "2/2 → 3/2");
}

#[test]
fn lure_forces_all_blockers() {
    // CR 509.1c — every creature able to block the enchanted creature must.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_aura_on(&mut g, catalog::lure(), bear, &[(Color::Green, 3)]);
    g.clear_sickness(bear);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    // Declaring no blocks is illegal — the able blocker must block the lured creature.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_err(),
        "an able blocker must block the lured attacker");
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bear)])).expect("forced block legal");
}

#[test]
fn blanchwood_armor_scales_with_forests() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    cast_aura_on(&mut g, catalog::blanchwood_armor(), bear, &[(Color::Green, 3)]);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (4, 4), "2/2 + 2 Forests = 4/4");
}

#[test]
fn ironclaw_orcs_cant_block_big_attackers() {
    let mut g = two_player_game();
    let orcs = g.add_card_to_battlefield(0, catalog::ironclaw_orcs());
    let big = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → still ≥2
    let tiny = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1
    g.clear_sickness(big);
    g.clear_sickness(small);
    g.clear_sickness(tiny);
    assert!(!g.blocker_can_block_attacker(orcs, big), "can't block power 3");
    assert!(!g.blocker_can_block_attacker(orcs, small), "can't block power 2");
    assert!(g.blocker_can_block_attacker(orcs, tiny), "can block power 1");
}

#[test]
fn dwarven_warriors_makes_target_unblockable() {
    let mut g = two_player_game();
    let dw = g.add_card_to_battlefield(0, catalog::dwarven_warriors());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(dw);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dw, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("grant unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
}

#[test]
fn frozen_shade_pumps_with_black() {
    let mut g = two_player_game();
    let shade = g.add_card_to_battlefield(0, catalog::frozen_shade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shade, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let p = g.computed_permanent(shade).unwrap();
    assert_eq!((p.power, p.toughness), (1, 2), "0/1 → 1/2");
}

#[test]
fn wall_of_brambles_regenerates() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_brambles());
    assert!(catalog::wall_of_brambles().keywords.contains(&Keyword::Defender));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wall).unwrap().regeneration_shields, 1, "regen shield stamped");
}

#[test]
fn whirling_dervish_grows_on_combat_damage() {
    let mut g = two_player_game();
    let dervish = g.add_card_to_battlefield(0, catalog::whirling_dervish());
    g.clear_sickness(dervish);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dervish, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(dervish).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "grew after dealing combat damage to a player");
    assert!(catalog::whirling_dervish().keywords.contains(&Keyword::Protection(Color::Black)));
}

#[test]
fn femeref_archers_shoots_attacking_flyer() {
    let mut g = two_player_game();
    let archers = g.add_card_to_battlefield(0, catalog::femeref_archers());
    g.clear_sickness(archers);
    let flyer = g.add_card_to_battlefield(0, catalog::bird_maiden()); // 1/2 flyer
    g.clear_sickness(flyer);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: flyer, target: AttackTarget::Player(1),
    }])).expect("flyer attacks");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: archers, ability_index: 0, target: Some(Target::Permanent(flyer)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("shoot the attacking flyer");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flyer).is_none(), "took 4, died");
}

#[test]
fn fyndhorn_elder_taps_for_two_green() {
    let mut g = two_player_game();
    let elder = g.add_card_to_battlefield(0, catalog::fyndhorn_elder());
    g.clear_sickness(elder);
    g.perform_action(GameAction::ActivateAbility {
        card_id: elder, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "added two green");
}

#[test]
fn wyluli_wolf_pumps_a_target() {
    let mut g = two_player_game();
    let wolf = g.add_card_to_battlefield(0, catalog::wyluli_wolf());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(wolf);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wolf, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("pump the bear");
    drain_stack(&mut g);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (3, 3), "2/2 → 3/3 until EOT");
}

#[test]
fn goblin_elite_infantry_shrinks_when_it_blocks() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(1, catalog::goblin_elite_infantry());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(goblin, attacker)])).expect("block");
    drain_stack(&mut g);
    let p = g.computed_permanent(goblin).unwrap();
    assert_eq!((p.power, p.toughness), (1, 1), "2/2 → 1/1 for blocking");
}

#[test]
fn jayemdae_tome_draws() {
    let mut g = two_player_game();
    let tome = g.add_card_to_battlefield(0, catalog::jayemdae_tome());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: tome, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

#[test]
fn aladdins_ring_deals_four() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(0, catalog::aladdins_ring());
    g.players[0].mana_pool.add_colorless(8);
    let foe = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 4, "4 damage to the opponent");
}

#[test]
fn lifegain_cycle_triggers_on_matching_color_spell() {
    // Structural: each battery watches its own color's casts.
    for (card, kind) in [
        (catalog::throne_of_bone(), EventKind::SpellCast),
        (catalog::wooden_sphere(), EventKind::SpellCast),
        (catalog::iron_star(), EventKind::SpellCast),
        (catalog::crystal_rod(), EventKind::SpellCast),
        (catalog::ivory_cup(), EventKind::SpellCast),
    ] {
        assert_eq!(card.triggered_abilities[0].event.kind, kind);
    }
}

#[test]
fn recent76_static_stats() {
    assert!(catalog::cockatrice().keywords.contains(&Keyword::Flying));
    assert!(catalog::cockatrice().keywords.contains(&Keyword::Deathtouch));
    assert!(catalog::thicket_basilisk().keywords.contains(&Keyword::Deathtouch));
    assert!(catalog::bird_maiden().keywords.contains(&Keyword::Flying));
    assert!(catalog::alaborn_grenadier().keywords.contains(&Keyword::Vigilance));
    assert_eq!((catalog::skeletal_snake().power, catalog::skeletal_snake().toughness), (2, 1));
    assert!(catalog::skeletal_snake().subtypes.creature_types.contains(&CreatureType::Skeleton));
    assert!(catalog::ironclaw_orcs().keywords.contains(&Keyword::CantBlockPowerAtLeast(2)));
}
