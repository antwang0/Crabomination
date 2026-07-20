//! Tests for the recent296 Ravnica batch 6 (Radiance group, Blocks tapper,
//! guild auras/utility).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
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
fn rally_the_righteous_untaps_and_pumps_shared_colors() {
    let mut g = two_player_game();
    let g1 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    let g2 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    let white = g.add_card_to_battlefield(0, catalog::serra_angel()); // white
    for id in [g1, g2, white] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    let spell = g.add_card_to_hand(0, catalog::rally_the_righteous());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(g1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(g1).unwrap().tapped && !g.battlefield_find(g2).unwrap().tapped,
        "both green creatures untapped by radiance");
    assert_eq!(g.computed_permanent(g2).unwrap().power, 4, "shared-color creature got +2/+0");
    assert!(g.battlefield_find(white).unwrap().tapped, "white shares no color — still tapped");
    assert_eq!(g.computed_permanent(white).unwrap().power, 4, "unbuffed 4/4 Serra Angel");
}

#[test]
fn vertigo_spawn_taps_the_creature_it_blocks() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let spawn = g.add_card_to_battlefield(0, catalog::vertigo_spawn()); // 0/3 Defender
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(spawn, attacker)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).unwrap().tapped, "blocked attacker was tapped");
    assert!(g.battlefield_find(attacker).unwrap().skip_next_untap, "and won't untap next turn");
}

#[test]
fn souls_of_the_faultless_drains_the_attacker() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    let souls = g.add_card_to_battlefield(0, catalog::souls_of_the_faultless()); // 0/4 Defender
    let (my_life, foe_life) = (g.players[0].life, g.players[1].life);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(souls, attacker)])).expect("block");
    // Advance through combat damage; the trigger fires on the 2 combat damage.
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, my_life + 2, "you gained the 2 combat damage");
    assert_eq!(g.players[1].life, foe_life - 2, "attacking player lost that much");
}

#[test]
fn tin_street_hooligan_destroys_artifact_only_when_green_spent() {
    // Pay the {1} with green → {G} spent → destroy the artifact.
    let mut g = two_player_game();
    let signet = g.add_card_to_battlefield(1, catalog::azorius_signet());
    let hooligan = g.add_card_to_hand(0, catalog::tin_street_hooligan());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: hooligan, target: Some(Target::Permanent(signet)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast with green spent");
    drain_stack(&mut g);
    assert!(g.battlefield_find(signet).is_none(), "green spent → artifact destroyed");

    // No green spent → no destroy (ETB does nothing).
    let mut g = two_player_game();
    let signet = g.add_card_to_battlefield(1, catalog::azorius_signet());
    let hooligan = g.add_card_to_hand(0, catalog::tin_street_hooligan());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: hooligan, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast without green");
    drain_stack(&mut g);
    assert!(g.battlefield_find(signet).is_some(), "no green spent → artifact survives");
}

#[test]
fn petrahydrox_bounces_itself_when_targeted() {
    let mut g = two_player_game();
    let pet = g.add_card_to_battlefield(0, catalog::petrahydrox());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(pet)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pet).is_none(), "left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == pet), "returned to owner's hand");
}

#[test]
fn shadow_lance_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::shadow_lance());
    let ctx = EffectContext::for_ability(aura, 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&catalog::shadow_lance().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn shielding_plax_draws_and_grants_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::shielding_plax());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    let ctx = EffectContext::for_ability(aura, 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&catalog::shielding_plax().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof));
    assert_eq!(g.players[0].hand.len(), hand + 1, "ETB drew a card");
}

#[test]
fn dowsing_shaman_returns_enchantment_from_graveyard() {
    let mut g = two_player_game();
    let shaman = g.add_card_to_battlefield(0, catalog::dowsing_shaman());
    g.clear_sickness(shaman);
    let ench = g.add_card_to_graveyard(0, catalog::shadow_lance()); // an enchantment card
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shaman, ability_index: 0, target: Some(Target::Permanent(ench)),
        additional_targets: vec![], x_value: None,
    }).expect("return enchantment");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == ench), "enchantment back in hand");
}

#[test]
fn poison_the_well_kills_land_and_burns_its_controller() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::poison_the_well());
    flood(&mut g);
    let foe = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(land)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
    assert_eq!(g.players[1].life, foe - 2, "land's controller took 2");
}

#[test]
fn peregrine_mask_grants_evasion_keywords() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mask = g.add_card_to_battlefield(0, catalog::peregrine_mask());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: mask, target: bear }).expect("equip");
    drain_stack(&mut g);
    let kw = g.computed_permanent(bear).unwrap().keywords;
    assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::FirstStrike)
        && kw.contains(&Keyword::Defender), "mask grants flying + first strike + defender");
}

#[test]
fn congregation_at_dawn_stacks_three_creatures_on_top() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::grizzly_bears());
    let b = g.add_card_to_library(0, catalog::serra_angel());
    let c = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
        DecisionAnswer::Search(Some(c)),
    ]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::congregation_at_dawn().effect, &ctx).unwrap();
    drain_stack(&mut g);
    // All three fetched creatures now sit on top of the library.
    let top3: Vec<_> = g.players[0].library.iter().rev().take(3).map(|c| c.id).collect();
    assert!([a, b, c].iter().all(|id| top3.contains(id)), "three creatures placed on top");
}
