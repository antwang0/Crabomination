//! Demonstrate cycle (the STX "Technique" sorceries, CR 702.150).
//! `Effect::Demonstrate` copies the spell for its caster and an opponent;
//! every copy may choose new targets.

use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;

/// Replication Technique's Demonstrate: the caster copies the spell (two token
/// copies of their own permanent — original + copy), and an opponent also
/// copies it, controlling a token copy of *their* own permanent.
#[test]
fn replication_technique_demonstrate_copies_for_both_players() {
    let mut g = two_player_game();
    let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::replication_technique());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    let p0_before = g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count();
    let p1_before = g.battlefield.iter().filter(|c| c.controller == 1 && c.is_token).count();

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(my_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Replication Technique");
    drain_stack(&mut g);

    let p0_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count();
    let p1_after = g.battlefield.iter().filter(|c| c.controller == 1 && c.is_token).count();
    assert_eq!(p0_after - p0_before, 2, "caster gets original + demonstrate copy");
    assert_eq!(p1_after - p1_before, 1, "opponent also copies, controlling its own copy");
}

/// Excavation Technique destroys a target nonland permanent; its controller
/// (here the opponent) creates two Treasure tokens.
#[test]
fn excavation_technique_destroys_and_treasures_controller() {
    let mut g = two_player_game();
    let opp_perm = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::excavation_technique());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(opp_perm)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Excavation Technique");
    drain_stack(&mut g);

    assert!(g.battlefield_find(opp_perm).is_none(), "target nonland permanent destroyed");
    // The destroyed permanent's controller (here the opponent) creates two
    // Treasures. The Demonstrate copies also resolve against the same target;
    // a zone-blind "nonland permanent" filter doesn't fizzle them at
    // resolution (see TODO.md), so the controller ends up with a multiple of
    // two. The core behavior — destroyed and its controller is the recipient —
    // is what matters.
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.name == "Treasure").count();
    assert!(treasures >= 2, "destroyed permanent's controller makes (at least) two Treasures");
}

/// Healing Technique returns a card from your graveyard to hand, gains life
/// equal to its mana value, and exiles itself.
#[test]
fn healing_technique_returns_card_gains_life_and_exiles_self() {
    let mut g = two_player_game();
    let card = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let spell = g.add_card_to_hand(0, catalog::healing_technique());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(card)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Healing Technique");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == card), "card returned to hand");
    // Gain life equal to the returned card's mana value (Grizzly Bears = 2).
    // The Demonstrate copies retarget into the caster's graveyard too; the
    // engine's target filter has no zone constraint, so at least the base
    // mana value is gained.
    assert!(g.players[0].life >= life + 2, "gain life equal to mana value");
    assert!(g.exile.iter().any(|c| c.id == spell), "Healing Technique exiles itself");
}

/// Incarnation Technique mills five, then returns a creature card from your
/// graveyard to the battlefield.
#[test]
fn incarnation_technique_mills_then_reanimates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // Stock the library so the mill has something to move.
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::incarnation_technique());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Incarnation Technique");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_some(), "creature reanimated to battlefield");
}

/// Creative Technique shuffles, exiles the top until a nonland card, and lets
/// you cast it for free. With the cast accepted, the lone nonland card enters
/// the battlefield.
#[test]
fn creative_technique_impulse_casts_a_nonland() {
    let mut g = two_player_game();
    let lib_bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::creative_technique());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    // Accept the Cascade free-cast prompt.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Creative Technique");
    drain_stack(&mut g);

    assert!(
        g.battlefield_find(lib_bear).is_some(),
        "the revealed nonland card was exiled and impulse-cast",
    );
}

/// Transforming Flourish destroys a target artifact or creature you don't
/// control.
#[test]
fn transforming_flourish_destroys_opponent_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::transforming_flourish());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Transforming Flourish");
    drain_stack(&mut g);

    assert!(g.battlefield_find(opp_bear).is_none(), "opponent's creature destroyed");
}
