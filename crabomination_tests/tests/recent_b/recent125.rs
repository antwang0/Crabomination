//! Functionality tests for `catalog::sets::decks::recent125`.

use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn saddled_attack(g: &mut GameState, mount: CardId) {
    g.battlefield_find_mut(mount).unwrap().saddled = true;
    g.clear_sickness(mount);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: mount, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(g);
}

/// Bridled Bighorn makes a Sheep when it attacks saddled.
#[test]
fn bridled_bighorn_saddled_makes_sheep() {
    let mut g = two_player_game();
    let bighorn = g.add_card_to_battlefield(0, catalog::bridled_bighorn());
    saddled_attack(&mut g, bighorn);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Sheep").count(),
        1, "a Sheep token appeared"
    );
}

/// Drover Grizzly grants the team trample when it attacks saddled.
#[test]
fn drover_grizzly_saddled_grants_trample() {
    let mut g = two_player_game();
    let grizzly = g.add_card_to_battlefield(0, catalog::drover_grizzly());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    saddled_attack(&mut g, grizzly);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Trample),
        "other creatures gain trample");
}

/// Sun-Blessed Healer reanimates a cheap permanent only when kicked.
#[test]
fn sun_blessed_healer_kicked_reanimates() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let healer = g.add_card_to_hand(0, catalog::sun_blessed_healer());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: healer, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear && c.controller == 0),
        "the bear returned to the battlefield");
}

/// Unkicked, Sun-Blessed Healer just enters (no reanimation).
#[test]
fn sun_blessed_healer_unkicked_no_reanimate() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let healer = g.add_card_to_hand(0, catalog::sun_blessed_healer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: healer, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast unkicked");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "bear stays in the graveyard");
}
