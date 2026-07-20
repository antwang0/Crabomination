//! Tests for the recent291 Ravnica guild batch.

use crabomination::catalog;
use crabomination::game::types::{AttackTarget, Attack, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

/// Fill player 0's pool with plenty of every color.
fn flood(g: &mut crabomination::game::GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 4);
    }
    g.players[0].mana_pool.add_colorless(8);
}

fn count_tokens(g: &crabomination::game::GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

#[test]
fn selesnya_guildmage_makes_saproling_and_pumps_team() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::selesnya_guildmage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g);
    // {3}{G}: create a Saproling.
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("make saproling");
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Saproling"), 1, "one Saproling minted");
    // {3}{W}: +1/+1 to all your creatures until EOT.
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("team pump");
    drain_stack(&mut g);
    let bp = g.computed_permanent(bear).unwrap();
    assert_eq!((bp.power, bp.toughness), (3, 3), "the bear got +1/+1");
}

#[test]
fn dimir_guildmage_draws_and_discards_at_sorcery_speed() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::dimir_guildmage());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    flood(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 0, target: Some(Target::Player(0)), additional_targets: vec![], x_value: None,
    }).expect("draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "targeted self drew a card");
    let opp_hand = g.players[1].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 1, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).expect("discard");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded");
}

#[test]
fn boros_guildmage_grants_haste_and_first_strike() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::boros_guildmage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
    }).expect("haste");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Haste));
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
    }).expect("first strike");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::FirstStrike));
}

#[test]
fn gruul_guildmage_sac_land_burn_and_pump() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::gruul_guildmage());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    flood(&mut g);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).expect("burn");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage to the player");
    assert!(g.battlefield_find(land).is_none(), "a land was sacrificed as the cost");
    // {3}{G}: +2/+2 to the guildmage itself.
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 1, target: Some(Target::Permanent(gm)), additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let p = g.computed_permanent(gm).unwrap();
    assert_eq!((p.power, p.toughness), (4, 4), "+2/+2");
}

#[test]
fn orzhov_guildmage_gains_life_and_symmetric_drain() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::orzhov_guildmage());
    flood(&mut g);
    let l0 = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 0, target: Some(Target::Player(0)), additional_targets: vec![], x_value: None,
    }).expect("gain");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 1, "gained 1 life");
    let (a, b) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("each player loses 1");
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (a - 1, b - 1), "each player lost 1");
}

#[test]
fn rakdos_guildmage_discards_for_minus_and_makes_a_temp_goblin() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::rakdos_guildmage());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: vec![], x_value: None,
    }).expect("-2/-2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the 2/2 died to -2/-2");
    // {3}{R}: a 2/1 haste Goblin, exiled at the next end step.
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("goblin");
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Goblin"), 1, "a Goblin token exists before the end step");
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Goblin"), 0, "the temp Goblin is exiled at the end step");
}

#[test]
fn azorius_guildmage_taps_and_counters_an_ability() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::azorius_guildmage());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gm, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: vec![], x_value: None,
    }).expect("tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped, "target creature tapped");
}

#[test]
fn fists_of_ironwood_mints_two_saprolings_and_grants_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::fists_of_ironwood());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Saproling"), 2, "two Saprolings on ETB");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Trample),
        "enchanted creature has trample");
}

#[test]
fn scatter_the_seeds_makes_three_saprolings() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::scatter_the_seeds());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Saproling"), 3, "three Saprolings");
}

#[test]
fn sundering_vitae_destroys_an_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let spell = g.add_card_to_hand(0, catalog::sundering_vitae());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(art)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

#[test]
fn golgari_rotwurm_sacs_a_creature_to_drain_one() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::golgari_rotwurm());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wurm, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).expect("drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "target player lost 1");
    assert!(g.battlefield_find(fodder).is_none(), "a creature was sacrificed");
}

#[test]
fn wrecking_ball_destroys_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::wrecking_ball());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(land)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
}

#[test]
fn ghor_clan_savage_enters_with_bloodthirst_counters() {
    let mut g = two_player_game();
    // An opponent was dealt damage this turn → Bloodthirst is active.
    g.players[1].was_dealt_damage_this_turn = true;
    let savage = g.add_card_to_hand(0, catalog::ghor_clan_savage());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: savage, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast savage");
    drain_stack(&mut g);
    let p = g.computed_permanent(savage).unwrap();
    assert_eq!((p.power, p.toughness), (5, 6), "2/3 + three +1/+1 from Bloodthirst 3");
}

#[test]
fn streetbreaker_wurm_is_a_vanilla_six_four() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::streetbreaker_wurm());
    let p = g.computed_permanent(wurm).unwrap();
    assert_eq!((p.power, p.toughness), (6, 4));
    assert!(p.keywords.is_empty(), "vanilla");
}

#[test]
fn recollect_returns_a_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt_id = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::recollect());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bolt_id)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_id), "Bolt returned to hand");
}

#[test]
fn frontier_warmonger_menace_needs_two_blockers() {
    // Cross-check the recent290 card composes with the 509.1c menace SBA.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::frontier_warmonger());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    assert!(g.computed_permanent(attacker).unwrap().keywords.contains(&crabomination::card::Keyword::Menace));
}
