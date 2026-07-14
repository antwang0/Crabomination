//! Functionality tests for `catalog::sets::decks::recent194`.

use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Double Down copies an outlaw (Rogue) spell you cast.
#[test]
fn double_down_copies_outlaw_spell() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::double_down());
    // Grizzly Bears is a Bear (not outlaw) → no copy; use a Rogue creature spell.
    let rogue = g.add_card_to_hand(0, catalog::servant_of_the_stinger()); // Warlock = outlaw
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rogue,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast outlaw spell");
    drain_stack(&mut g);
    let servants = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Servant of the Stinger" && c.controller == 0)
        .count();
    assert_eq!(servants, 2, "outlaw spell copied → two Servants (copy is a token)");
}

/// Mystical Tether exiles an opponent's creature until it leaves.
#[test]
fn mystical_tether_exiles_until_leaves() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tether = g.add_card_to_battlefield(0, catalog::mystical_tether());
    g.fire_self_etb_triggers(tether, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent's creature exiled");
    // Destroy the Tether → the creature returns.
    let ctx = crate::game::effects::EffectContext::for_ability(tether, 0, None);
    let evs = g
        .resolve_effect(&crate::effect::Effect::Destroy { what: crate::effect::Selector::This }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1),
        "creature returned when the Tether left",
    );
}

/// High Noon bars a second spell each turn.
#[test]
fn high_noon_one_spell_per_turn() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::high_noon());
    g.players[0].spells_cast_this_game_turn = 1; // already cast one this turn
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(), "second spell barred by High Noon");
}

/// High Noon's sacrifice ability burns for 5.
#[test]
fn high_noon_sac_burns_for_five() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let noon = g.add_card_to_battlefield(0, catalog::high_noon());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let opp = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: noon,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("sac High Noon for 5");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 5, "5 damage to the opponent");
    assert!(g.battlefield_find(noon).is_none(), "High Noon sacrificed");
}

