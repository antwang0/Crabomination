//! Functionality tests for `catalog::sets::decks::recent206`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Swiftblade Vindicator is a french-vanilla double strike / vigilance / trample.
#[test]
fn swiftblade_vindicator_has_three_keywords() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::swiftblade_vindicator());
    let cp = g.computed_permanent(v).unwrap();
    for kw in [Keyword::DoubleStrike, Keyword::Vigilance, Keyword::Trample] {
        assert!(cp.keywords.contains(&kw), "has {kw:?}");
    }
}

/// Progenitus shuffles into its owner's library instead of dying.
#[test]
fn progenitus_shuffles_in_instead_of_dying() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::progenitus());
    assert!(g.computed_permanent(p).unwrap().keywords.contains(&Keyword::ProtectionFromEverything));
    // Drop ten -1/-1 counters so the 10/10 dies to SBA.
    g.battlefield_find_mut(p).unwrap().counters.insert(CounterType::MinusOneMinusOne, 10);
    g.check_state_based_actions();
    assert!(g.battlefield_find(p).is_none(), "the 0/0 died to SBA");
    assert!(g.players[0].library.iter().any(|c| c.id == p), "shuffled into library, not graveyard");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == p), "not in the graveyard");
}

/// Rune-Scarred Demon tutors any card to hand on ETB.
#[test]
fn rune_scarred_demon_tutors_on_etb() {
    let mut g = two_player_game();
    let target = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    // Enter through the real ETB funnel so the self-source trigger fires.
    g.move_card_to_battlefield_for_test(0, catalog::rune_scarred_demon());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == target), "tutored the card to hand");
}

/// Micromancer only finds a cheap instant/sorcery.
#[test]
fn micromancer_finds_one_mv_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // MV 1 instant
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
    g.move_card_to_battlefield_for_test(0, catalog::micromancer());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "found the {{1}} instant");
}

/// Seismic Rupture hits grounded creatures but spares fliers.
#[test]
fn seismic_rupture_spares_fliers() {
    let mut g = two_player_game();
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let flier = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // 5/6 flying
    let spell = g.add_card_to_hand(0, catalog::seismic_rupture());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Seismic Rupture");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ground).is_none(), "grounded 2/2 took 2 and died");
    assert!(g.battlefield_find(flier).is_some(), "the flier was untouched");
}

/// An Offer You Can't Refuse counters a noncreature spell and gifts its
/// controller two Treasures.
#[test]
fn an_offer_counters_and_gives_treasures() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a bolt");
    let offer = g.add_card_to_hand(0, catalog::an_offer_you_cant_refuse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: offer, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter the bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt countered — no damage");
    let treasures = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Treasure").count();
    assert_eq!(treasures, 2, "the bolt's controller got two Treasures");
}

/// Involuntary Employment steals a creature for the turn with haste + a Treasure.
#[test]
fn involuntary_employment_steals_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::involuntary_employment());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Involuntary Employment");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.controller, 0, "gained control");
    assert!(!c.tapped, "untapped");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "has haste");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"), "made a Treasure");
}

/// Pilfer makes the opponent discard a nonland card of the caster's choosing.
#[test]
fn pilfer_discards_a_nonland() {
    let mut g = two_player_game();
    let spell_card = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::forest());
    let pilfer = g.add_card_to_hand(0, catalog::pilfer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: pilfer, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Pilfer");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell_card), "nonland was discarded");
}

/// Grow from the Ashes fetches one basic normally, two when kicked.
#[test]
fn grow_from_the_ashes_kicked_fetches_two() {
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::grow_from_the_ashes());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4); // {2}{G} + kicker {2}
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked Grow from the Ashes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(f1).is_some() && g.battlefield_find(f2).is_some(),
        "kicked fetch put both basics onto the battlefield");
}

/// Doubling Season doubles tokens an effect makes under your control.
#[test]
fn doubling_season_doubles_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doubling_season());
    g.add_card_to_battlefield(0, catalog::rite_of_the_dragoncaller());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a bolt to trigger Rite");
    drain_stack(&mut g);
    let dragons = g.battlefield.iter().filter(|c| c.definition.name == "Dragon").count();
    assert_eq!(dragons, 2, "Rite's one Dragon was doubled to two");
}
