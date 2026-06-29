//! Functionality tests for `catalog::sets::decks::recent41` — prevention,
//! graveyard-hate, and green-tempo staples. Covers the new
//! `PreventNoncombatDamageToYourCreatures` static (CR 615) and the
//! instant/sorcery-only `ExileCardsBoundForGraveyard` filter (CR 614.6).

use crate::catalog;
use crate::game::effects::{EffectContext, EntityRef};
use crate::game::two_player_game;
use crate::game::*;

#[test]
fn mark_of_asylum_prevents_noncombat_damage_to_your_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mark_of_asylum());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(mine), 3, None, &mut events);
    g.deal_damage_to_from(EntityRef::Permanent(theirs), 3, None, &mut events);
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "your creature's noncombat damage is prevented");
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 3, "the opponent's creature is unaffected");
}

#[test]
fn dryad_militant_exiles_instants_and_sorceries_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dryad_militant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // instant
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // creature
    let bolt_card = g.players[0].hand.iter().find(|c| c.id == bolt).unwrap().clone();
    let bear_card = g.players[0].hand.iter().find(|c| c.id == bear).unwrap().clone();
    assert!(g.graveyard_exiled_for(&bolt_card), "an instant bound for a graveyard is exiled");
    assert!(!g.graveyard_exiled_for(&bear_card), "a creature card is not");
}

#[test]
fn dryad_militant_bolt_lands_in_exile_after_resolving() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::dryad_militant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the spent Bolt is exiled, not in the graveyard");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt), "not in any graveyard");
}

#[test]
fn plated_geopede_grows_on_landfall() {
    let mut g = two_player_game();
    let pede = g.add_card_to_battlefield(0, catalog::plated_geopede());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: land }]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(pede).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "1/1 + landfall +2/+2 = 3/3");
}

#[test]
fn scale_up_makes_a_six_four_wurm() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&catalog::scale_up().effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 4), "becomes a 6/4");
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Wurm), "and a Wurm");
}

#[test]
fn spawning_pool_animates_to_a_skeleton() {
    let mut g = two_player_game();
    let pool = g.add_card_to_battlefield(0, catalog::spawning_pool());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pool, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(pool).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "1/1 Skeleton");
    assert!(cp.card_types.contains(&crate::card::CardType::Land), "still a land");
}
