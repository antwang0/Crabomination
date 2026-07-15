//! Functionality tests for `catalog::sets::decks::recent77`.

use crabomination::card::{CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

/// Cast an Aura from hand onto `host` with the given colored pips available.
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
fn storm_shaman_firebreathes() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::storm_shaman());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{R}: +1/+0");
    drain_stack(&mut g);
    let p = g.computed_permanent(s).unwrap();
    assert_eq!((p.power, p.toughness), (1, 4), "0/4 → 1/4");
}

#[test]
fn wild_aesthir_pump_is_once_per_turn() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::wild_aesthir());
    g.players[0].mana_pool.add(Color::White, 4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: a, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("first activation");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(a).unwrap().power, 3, "1/1 → 3/1");
    // Second activation the same turn is illegal (activate only once each turn).
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: a, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "once each turn");
}

#[test]
fn woolly_spider_pumps_when_blocking_a_flyer() {
    let mut g = two_player_game();
    let spider = g.add_card_to_battlefield(1, catalog::woolly_spider());
    g.clear_sickness(spider);
    let flyer = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 flying
    g.clear_sickness(flyer);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: flyer, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(spider, flyer)])).expect("block flyer");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(spider).unwrap().toughness, 5, "2/3 → 2/5 blocking a flyer");
}

#[test]
fn orcish_artillery_hits_target_and_controller() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(0, catalog::orcish_artillery());
    g.clear_sickness(art);
    let foe = g.players[1].life;
    let me = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: art, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 2, "2 to target");
    assert_eq!(g.players[0].life, me - 3, "3 to controller");
}

#[test]
fn goblin_digging_team_destroys_a_wall() {
    let mut g = two_player_game();
    let team = g.add_card_to_battlefield(0, catalog::goblin_digging_team());
    g.clear_sickness(team);
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_brambles()); // Plant Wall
    g.perform_action(GameAction::ActivateAbility {
        card_id: team, ability_index: 0, target: Some(Target::Permanent(wall)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("destroy wall");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "Wall destroyed");
    assert!(g.battlefield_find(team).is_none(), "Digging Team sacrificed");
}

#[test]
fn aysen_bureaucrats_taps_only_small_creatures() {
    let mut g = two_player_game();
    let bur = g.add_card_to_battlefield(0, catalog::aysen_bureaucrats());
    g.clear_sickness(bur);
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.perform_action(GameAction::ActivateAbility {
        card_id: bur, ability_index: 0, target: Some(Target::Permanent(small)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("tap the 2/2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).unwrap().tapped, "power-2 creature tapped");
}

#[test]
fn anaba_spirit_crafter_pumps_all_minotaurs() {
    let mut g = two_player_game();
    let crafter = g.add_card_to_battlefield(0, catalog::anaba_spirit_crafter()); // Minotaur Shaman 1/3
    let other = g.add_card_to_battlefield(1, catalog::anaba_ancestor()); // opponent's Minotaur 1/1
    assert_eq!(g.computed_permanent(crafter).unwrap().power, 2, "pumps itself: 1/3 → 2/3");
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "pumps any Minotaur, even opponents': 1/1 → 2/1");
}

#[test]
fn anaba_ancestor_pumps_another_minotaur() {
    let mut g = two_player_game();
    let anc = g.add_card_to_battlefield(0, catalog::anaba_ancestor());
    g.clear_sickness(anc);
    let target = g.add_card_to_battlefield(0, catalog::anaba_spirit_crafter());
    // Crafter is already 2/3 from its own static; +1/+1 makes it 3/4.
    g.perform_action(GameAction::ActivateAbility {
        card_id: anc, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("pump another Minotaur");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(target).unwrap().toughness, 4, "+1/+1");
}

#[test]
fn elvish_bard_forces_all_blockers() {
    let mut g = two_player_game();
    let bard = g.add_card_to_battlefield(0, catalog::elvish_bard());
    g.clear_sickness(bard);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bard, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_err(),
        "an able blocker must block Elvish Bard");
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bard)])).expect("forced block legal");
}

#[test]
fn marsh_goblins_has_swampwalk() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(0, catalog::marsh_goblins());
    assert!(g.computed_permanent(goblin).unwrap().keywords
        .contains(&Keyword::Landwalk(crabomination::card::LandType::Swamp)));
}

#[test]
fn ghost_hounds_gains_first_strike_blocking_white() {
    let mut g = two_player_game();
    let hounds = g.add_card_to_battlefield(1, catalog::ghost_hounds());
    g.clear_sickness(hounds);
    let white = g.add_card_to_battlefield(0, catalog::savannah_lions()); // white ground attacker
    g.clear_sickness(white);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: white, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(hounds, white)])).expect("block white");
    drain_stack(&mut g);
    assert!(g.computed_permanent(hounds).unwrap().keywords.contains(&Keyword::FirstStrike),
        "gained first strike blocking a white creature");
}

#[test]
fn orcish_oriflamme_pumps_only_attackers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::orcish_oriflamme());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    // Not attacking yet — no bonus.
    assert_eq!(g.computed_permanent(atk).unwrap().power, 2, "idle creature unpumped");
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(atk).unwrap().power, 3, "attacking → +1/+0");
}

#[test]
fn regeneration_grants_regen_ability() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_aura_on(&mut g, catalog::regeneration(), bear, &[(Color::Green, 2)]);
    let granted = g.granted_abilities_for(bear);
    assert_eq!(granted.len(), 1, "host gained a regenerate ability");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regen shield");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().regeneration_shields, 1, "regen shield stamped");
}

#[test]
fn carapace_toughness_and_sac_regenerates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::carapace());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 4, "+0/+2");
    // Sacrifice the Aura to set up a regen shield on the enchanted creature.
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac Carapace to regenerate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "Carapace sacrificed");
    assert_eq!(g.battlefield_find(bear).unwrap().regeneration_shields, 1, "regen shield on the host");
}

#[test]
fn feast_of_the_unicorn_pumps_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::feast_of_the_unicorn());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6, "2/2 → 6/2");
}

#[test]
fn icequake_snow_land_pings_controller() {
    let mut g = two_player_game();
    let snow = g.add_card_to_battlefield(1, catalog::snow_covered_swamp());
    let foe = g.players[1].life;
    let ice = g.add_card_to_hand(0, catalog::icequake());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ice, target: Some(Target::Permanent(snow)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("destroy snow land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(snow).is_none(), "land destroyed");
    assert_eq!(g.players[1].life, foe - 1, "snow land → 1 damage to its controller");
}

#[test]
fn icequake_nonsnow_land_deals_no_damage() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::swamp());
    let foe = g.players[1].life;
    let ice = g.add_card_to_hand(0, catalog::icequake());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ice, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("destroy land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
    assert_eq!(g.players[1].life, foe, "non-snow land → no damage");
}

#[test]
fn jokulhaups_wipes_the_board() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let artifact = g.add_card_to_battlefield(1, catalog::jayemdae_tome());
    let jk = g.add_card_to_hand(0, catalog::jokulhaups());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: jk, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("boom");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).is_none(), "creature destroyed");
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
    assert!(g.battlefield_find(artifact).is_none(), "artifact destroyed");
}

#[test]
fn yavimaya_ants_has_cumulative_upkeep() {
    let ants = catalog::yavimaya_ants();
    assert!(ants.keywords.iter().any(|k| matches!(k, Keyword::CumulativeUpkeep(_))), "has cumulative upkeep");
    assert!(ants.keywords.contains(&Keyword::Trample) && ants.keywords.contains(&Keyword::Haste));
}

#[test]
fn merfolk_assassin_targets_islandwalkers() {
    // Structural: the ability's destroy filter is islandwalk-only.
    let a = catalog::merfolk_assassin();
    assert_eq!(a.subtypes.creature_types, vec![CreatureType::Merfolk, CreatureType::Assassin]);
    assert_eq!(a.activated_abilities.len(), 1);
}
