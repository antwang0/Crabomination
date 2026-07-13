//! Functionality tests for `catalog::sets::decks::tdm`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::*;
use crate::mana::Color;

/// Alesha's Legacy grants deathtouch + indestructible to your creature.
#[test]
fn aleshas_legacy_grants_two_keywords() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::aleshas_legacy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Alesha's Legacy");
    drain_stack(&mut g);
    let kws = g.computed_permanent(mine).unwrap().keywords;
    assert!(kws.contains(&Keyword::Deathtouch), "gained deathtouch");
    assert!(kws.contains(&Keyword::Indestructible), "gained indestructible");
}

/// Fire-Rim Form pumps +2/+0 and grants first strike on enter.
#[test]
fn fire_rim_form_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::fire_rim_form());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Fire-Rim Form");
    drain_stack(&mut g);
    let cp = g.computed_permanent(creature).unwrap();
    assert_eq!(cp.power, 4, "+2/+0 → 4 power");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "ETB granted first strike");
}

/// Jade-Cast Sentinel bottoms a graveyard card.
#[test]
fn jade_cast_sentinel_bottoms_graveyard_card() {
    let mut g = two_player_game();
    let sentinel = g.add_card_to_battlefield(0, catalog::jade_cast_sentinel());
    g.clear_sickness(sentinel);
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sentinel,
        ability_index: 0,
        target: Some(Target::Permanent(dead)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("bottom a graveyard card");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "left the graveyard");
    assert_eq!(g.players[1].library.last().unwrap().id, dead, "went to owner's library bottom");
}

/// Gurmag Nightwatch digs three, keeps one on top, mills the rest.
#[test]
fn gurmag_nightwatch_digs_and_mills() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let gy = g.players[0].graveyard.len();
    let lib = g.players[0].library.len();
    let creature = g.add_card_to_battlefield(0, catalog::gurmag_nightwatch());
    g.fire_self_etb_triggers(creature, 0);
    drain_stack(&mut g);
    // One kept on top, two milled → library down 2, graveyard up 2.
    assert_eq!(g.players[0].graveyard.len(), gy + 2, "milled two");
    assert_eq!(g.players[0].library.len(), lib - 2, "kept one on top");
}
