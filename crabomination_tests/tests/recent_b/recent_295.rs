//! Tests for the recent295 Ravnica batch 5 (Hellbent, Bloodthirst, combat).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, TurnStep};
use crabomination::mana::Color;

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
    g.players[0].mana_pool.add_colorless(8);
}

#[test]
fn bloodscale_prowler_enters_bigger_after_damage() {
    let mut g = two_player_game();
    g.players[1].was_dealt_damage_this_turn = true;
    let p = g.add_card_to_hand(0, catalog::bloodscale_prowler());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: p, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Bloodscale Prowler").unwrap().id;
    assert_eq!(g.computed_permanent(id).unwrap().power, 4, "3/1 + a bloodthirst counter → 4/2");
}

#[test]
fn ordruun_commando_prevents_damage_to_itself() {
    let mut g = two_player_game();
    let cmd = g.add_card_to_battlefield(0, catalog::ordruun_commando()); // 4/1
    g.players[0].mana_pool.add(Color::White, 2);
    // Stack two 1-damage shields (prevent 2 total).
    for _ in 0..2 {
        g.perform_action(GameAction::ActivateAbility {
            card_id: cmd, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        }).expect("shield");
        drain_stack(&mut g);
    }
    // Shock's 2 damage is fully prevented — the 1-toughness Commando survives.
    let bolt = g.add_card_to_hand(1, catalog::shock());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(cmd)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("shock");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cmd).is_some(), "both points of Shock prevented — it lives");
    assert_eq!(g.battlefield_find(cmd).unwrap().damage, 0, "no damage marked");
}

#[test]
fn feral_animist_doubles_its_own_power() {
    let mut g = two_player_game();
    let fa = g.add_card_to_battlefield(0, catalog::feral_animist());
    g.clear_sickness(fa);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fa, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(fa).unwrap().power, 4, "2/1 gets +2/+0 (X = its power)");
}

#[test]
fn coalhauler_swine_splashes_damage_to_each_player() {
    let mut g = two_player_game();
    let swine = g.add_card_to_battlefield(0, catalog::coalhauler_swine());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    // A 3-damage burn to the Swine bounces 3 to each player.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(swine)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt the swine");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 - 3);
    assert_eq!(g.players[1].life, l1 - 3);
}

#[test]
fn vigean_hydropon_grafts_but_cant_attack() {
    let mut g = two_player_game();
    let h = g.move_card_to_battlefield_for_test(0, catalog::vigean_hydropon());
    let c = g.computed_permanent(h).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "Graft 5 → 5/5");
    assert!(c.keywords.contains(&Keyword::CantAttack) && c.keywords.contains(&Keyword::CantBlock));
}

#[test]
fn twinstrike_pings_two_then_kills_when_hellbent() {
    // Cards in hand → deal 2 to each.
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card in hand
    let a = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let b = g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::twinstrike());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast with a card in hand");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().damage, 2, "only 2 damage — not hellbent");

    // Empty hand → destroy both.
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::serra_angel());
    let b = g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::twinstrike());
    flood(&mut g);
    // Casting the spell empties the hand (only Twinstrike was in it).
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast hellbent");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(),
        "hellbent → both destroyed");
}

#[test]
fn poisonbelly_ogre_bleeds_on_each_creature_enter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::poisonbelly_ogre());
    let life = g.players[0].life;
    // Your own creature entering triggers a 1-life loss for its controller (you).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "the entering creature's controller lost 1");
}

#[test]
fn devouring_light_exiles_an_attacker() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    let spell = g.add_card_to_hand(0, catalog::devouring_light());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(attacker)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("exile the attacker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "removed from combat");
    assert!(g.exile.iter().any(|c| c.id == attacker), "exiled, not destroyed");
}

#[test]
fn fangren_pathcutter_grants_team_trample() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let fangren = g.add_card_to_battlefield(0, catalog::fangren_pathcutter());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fangren);
    g.clear_sickness(ally);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: fangren, target: AttackTarget::Player(1) },
        Attack { attacker: ally, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Trample),
        "the other attacker gained trample");
}

#[test]
fn root_kin_ally_taps_two_to_pump() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::root_kin_ally());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(ally);
    // tap_n_filter auto-taps two untapped creatures you control as the cost.
    g.perform_action(GameAction::ActivateAbility {
        card_id: ally, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("tap two, pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ally).unwrap().power, 5, "3/3 + 2/2");
}

#[test]
fn cleansing_beam_radiance_hits_shared_colors_only() {
    let mut g = two_player_game();
    // Two green creatures (share green) and a white one (doesn't).
    let g1 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green 2/2
    let g2 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green 2/2
    let white = g.add_card_to_battlefield(1, catalog::serra_angel()); // white 4/4
    let spell = g.add_card_to_hand(0, catalog::cleansing_beam());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(g1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(g1).is_none() && g.battlefield_find(g2).is_none(),
        "both green creatures took 2 (radiance) and died");
    assert!(g.battlefield_find(white).is_some(), "the white creature shares no color — untouched");
    assert_eq!(g.battlefield_find(white).unwrap().damage, 0);
}

#[test]
fn wojek_embermage_radiance_pings() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::wojek_embermage());
    g.clear_sickness(mage);
    let g1 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let g2 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let white = g.add_card_to_battlefield(1, catalog::serra_angel()); // white
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(g1)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(g1).unwrap().damage, 1, "subject took 1");
    assert_eq!(g.battlefield_find(g2).unwrap().damage, 1, "the other green creature too");
    assert_eq!(g.battlefield_find(white).unwrap().damage, 0, "white shares no color");
}
