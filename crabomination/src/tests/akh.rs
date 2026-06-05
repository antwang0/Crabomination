//! Functionality tests for Amonkhet Embalm (CR 702.88) / Eternalize
//! (CR 702.91) creatures (`catalog::sets::akh`).

use crate::catalog;
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;
use crate::game::{drain_stack, two_player_game};

/// Embalm exiles the card from the graveyard and mints a Zombie token copy
/// with the original's P/T.
#[test]
fn embalm_sacred_cat_mints_zombie_token_copy() {
    use crate::card::CreatureType;
    let mut g = two_player_game();
    let cat = g.add_card_to_graveyard(0, catalog::sacred_cat());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cat, ability_index: 0, target: None, x_value: None })
        .expect("Embalm {W} from graveyard");
    drain_stack(&mut g);
    // Original card is exiled (gone from graveyard).
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == cat), "card exiled by Embalm");
    assert!(g.exile.iter().any(|c| c.id == cat));
    // A 1/1 Zombie Cat token copy is on the battlefield.
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Sacred Cat")
        .expect("token copy minted");
    assert_eq!((tok.power(), tok.toughness()), (1, 1));
    assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Zombie),
        "token gains Zombie type");
    assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Cat),
        "token keeps original Cat type");
}

/// Eternalize mints a 4/4 token copy.
#[test]
fn eternalize_adorned_pouncer_mints_four_four() {
    let mut g = two_player_game();
    let p = g.add_card_to_graveyard(0, catalog::adorned_pouncer());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: p, ability_index: 0, target: None, x_value: None })
        .expect("Eternalize {3}{W}{W}");
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Adorned Pouncer")
        .expect("token copy minted");
    assert_eq!((tok.power(), tok.toughness()), (4, 4), "Eternalize token is 4/4");
    assert!(tok.definition.keywords.contains(&crate::card::Keyword::DoubleStrike),
        "token keeps Double Strike");
}

/// Embalm is sorcery-speed only: rejected on the opponent's turn.
#[test]
fn embalm_rejected_at_instant_speed() {
    let mut g = two_player_game();
    let cat = g.add_card_to_graveyard(0, catalog::sacred_cat());
    g.players[0].mana_pool.add(Color::White, 1);
    // Opponent's turn / stack-nonempty equivalents: hand priority to p0 during
    // an opponent main isn't sorcery speed for p0.
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: cat, ability_index: 0, target: None, x_value: None });
    assert!(res.is_err(), "Embalm only as a sorcery — rejected on opponent's turn");
}

/// Anointer Priest gains 1 life when a creature token you control enters.
#[test]
fn anointer_priest_gains_life_on_token_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::anointer_priest());
    let life = g.players[0].life;
    // Mint a creature token under p0's control.
    let servo = crate::card::TokenDefinition {
        name: "Servo".into(), power: 1, toughness: 1,
        card_types: vec![crate::card::CardType::Creature],
        ..Default::default()
    };
    let tok = g.add_token_to_battlefield(0, &servo);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: tok }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "Anointer Priest gains 1 life per token");
}

/// Angel of Sanctions exiles an opponent's permanent on ETB and returns it
/// when the Angel leaves.
#[test]
fn angel_of_sanctions_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_hand(0, catalog::angel_of_sanctions());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Angel of Sanctions");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear exiled by Angel ETB");
    // Angel leaves → bear returns.
    g.remove_from_battlefield_to_graveyard(angel);
    g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear returns when Angel leaves");
}
