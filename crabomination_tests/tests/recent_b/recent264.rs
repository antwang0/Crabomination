//! Functionality tests for `catalog::sets::decks::recent264` (MOM/BRO batch).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};

fn kw(g: &crabomination::game::GameState, id: crabomination::card::CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|cp| cp.keywords.contains(&k))
}

/// Alabaster Host Sanctifier is a 2/2 with lifelink.
#[test]
fn alabaster_host_sanctifier_has_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::alabaster_host_sanctifier());
    assert!(kw(&g, id, Keyword::Lifelink));
}

/// Nezumi Informant's ETB makes each opponent discard a card.
#[test]
fn nezumi_informant_opponent_discards() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    let before = g.players[1].hand.len();
    let effect = catalog::nezumi_informant().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded one");
}

/// Preening Champion flies and mints a 1/1 U/R Elemental on ETB.
#[test]
fn preening_champion_makes_elemental() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::preening_champion());
    assert!(kw(&g, champ, Keyword::Flying));
    let effect = catalog::preening_champion().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Elemental"),
        "Elemental token created"
    );
}

/// Knight of the New Coalition's ETB makes a vigilant Knight token.
#[test]
fn knight_of_new_coalition_makes_vigilant_knight() {
    let mut g = two_player_game();
    let effect = catalog::knight_of_the_new_coalition().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Knight").expect("Knight token");
    assert!(tok.definition.keywords.contains(&Keyword::Vigilance), "vigilant Knight");
}

/// Conscripted Infantry's real death makes a 1/1 Soldier artifact creature.
#[test]
fn conscripted_infantry_dies_into_soldier() {
    let mut g = two_player_game();
    let inf = g.add_card_to_battlefield(0, catalog::conscripted_infantry());
    // Destroy via the effect path, then dispatch the self-death trigger.
    let effect = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::Target(0),
    };
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(inf)), 0, 0);
    let evs = g.resolve_effect(&effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Soldier").expect("Soldier token");
    assert!(tok.definition.card_types.contains(&crabomination::card::CardType::Artifact));
}

/// Burrowing Razormaw mills four when it dies.
#[test]
fn burrowing_razormaw_dies_mills_four() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let before = g.players[0].graveyard.len();
    let effect = catalog::burrowing_razormaw().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].graveyard.len(), before + 4, "milled four");
}

/// Hoarding Recluse has reach + deathtouch and bottoms a graveyard card on death.
#[test]
fn hoarding_recluse_bottoms_graveyard_card() {
    let mut g = two_player_game();
    let recluse = g.add_card_to_battlefield(0, catalog::hoarding_recluse());
    assert!(kw(&g, recluse, Keyword::Reach) && kw(&g, recluse, Keyword::Deathtouch));
    let buried = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let lib_before = g.players[1].library.len();
    let effect = catalog::hoarding_recluse().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(buried)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].library.len(), lib_before + 1, "card moved to owner's library");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != buried), "left the graveyard");
}

/// Fallaji Chaindancer buys double strike with {2}.
#[test]
fn fallaji_chaindancer_grants_double_strike() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::fallaji_chaindancer());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(kw(&g, id, Keyword::DoubleStrike));
}

/// Iridescent Blademaster pumps itself +2/+2 with {3}{G}.
#[test]
fn iridescent_blademaster_firebreathes() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::iridescent_blademaster());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 became 4/4");
}

/// Air Marshal grants flying to a target Soldier.
#[test]
fn air_marshal_grants_flying_to_soldier() {
    let mut g = two_player_game();
    let marshal = g.add_card_to_battlefield(0, catalog::air_marshal());
    let ally = g.add_card_to_battlefield(0, catalog::alabaster_host_sanctifier()); // not a Soldier
    let soldier = g.add_card_to_battlefield(0, catalog::conscripted_infantry()); // Soldier
    let _ = ally;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: marshal, ability_index: 0,
        target: Some(Target::Permanent(soldier)), additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(kw(&g, soldier, Keyword::Flying), "the Soldier gained flying");
}

/// Onakke Javelineer taps to deal 2 to a player.
#[test]
fn onakke_javelineer_pings_a_player() {
    let mut g = two_player_game();
    let jav = g.add_card_to_battlefield(0, catalog::onakke_javelineer());
    g.battlefield_find_mut(jav).unwrap().summoning_sick = false;
    let before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: jav, ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 2);
}

/// Dreg Recycler sacrifices to drain one.
#[test]
fn dreg_recycler_drains_one() {
    let mut g = two_player_game();
    let dreg = g.add_card_to_battlefield(0, catalog::dreg_recycler());
    g.battlefield_find_mut(dreg).unwrap().summoning_sick = false;
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let (my_life, opp_life) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dreg, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1, "opponent lost one");
    assert_eq!(g.players[0].life, my_life + 1, "you gained one");
}

/// Coming In Hot pumps +1/+0, grants first strike, and scries.
#[test]
fn coming_in_hot_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::forest());
    let effect = catalog::coming_in_hot().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+0");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Cosmic Hunger makes your creature deal its power to another creature.
#[test]
fn cosmic_hunger_bites_with_power() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let effect = catalog::cosmic_hunger().effect;
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(mine), Target::Permanent(foe)];
    g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(foe).is_none(), "4 damage killed the 2/2");
}

/// Mirrodin Avenged destroys a damaged creature and draws.
#[test]
fn mirrodin_avenged_destroys_damaged_and_draws() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(foe).unwrap().dealt_damage_this_turn = true;
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    let effect = catalog::mirrodin_avenged().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(foe).is_none(), "destroyed the damaged creature");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Atraxa's Fall destroys a flyer.
#[test]
fn atraxas_fall_destroys_flyer() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    let effect = catalog::atraxas_fall().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(flyer)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(flyer).is_none(), "destroyed the flyer");
}

/// Furnace Host Charger has haste and mountaincycling.
#[test]
fn furnace_host_charger_haste_and_landcycling() {
    let g = two_player_game();
    let def = catalog::furnace_host_charger();
    assert!(def.keywords.contains(&Keyword::Haste));
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Landcycling(..))));
    let _ = g;
}

/// Phyrexian Pegasus grants flying to a nonflying attacker.
#[test]
fn phyrexian_pegasus_lifts_a_grounded_attacker() {
    let mut g = two_player_game();
    let _peg = g.add_card_to_battlefield(0, catalog::phyrexian_pegasus());
    let grunt = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no flying
    let effect = catalog::phyrexian_pegasus().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(grunt)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(kw(&g, grunt, Keyword::Flying), "grounded attacker gained flying");
}
