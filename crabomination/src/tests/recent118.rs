//! Functionality tests for `catalog::sets::decks::recent118`.

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Arcane Epiphany costs {1} less with a Wizard and draws three.
#[test]
fn arcane_epiphany_wizard_discount() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::dark_confidant()); // Human Wizard
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::arcane_epiphany());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2); // {2}{U}{U} after the -{1} discount
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("discounted cast with a Wizard");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 3, "drew three");
}

/// Without a Wizard, the discount doesn't apply and four mana is short.
#[test]
fn arcane_epiphany_no_discount_without_wizard() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let spell = g.add_card_to_hand(0, catalog::arcane_epiphany());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2); // only 4 — needs 5
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no Wizard means the full five-mana cost");
}

/// Agate Assault's damage mode exiles a creature that would die.
#[test]
fn agate_assault_exiles_on_lethal() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::agate_assault());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Agate Assault (damage mode)");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "lethal creature exiled, not in graveyard");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != victim));
}

/// Bark-Knuckle Boxer gains indestructible when you expend 4.
#[test]
fn bark_knuckle_boxer_expend_indestructible() {
    let mut g = two_player_game();
    let boxer = g.add_card_to_battlefield(0, catalog::bark_knuckle_boxer());
    let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G}
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a 6-mana spell (crosses expend 4)");
    drain_stack(&mut g);
    assert!(g.computed_permanent(boxer).unwrap().keywords.contains(&crate::card::Keyword::Indestructible),
        "expend 4 grants indestructible");
}

/// Brambleguard Veteran pumps Raccoons you control on expend 4.
#[test]
fn brambleguard_veteran_expend_pumps_raccoons() {
    let mut g = two_player_game();
    let vet = g.add_card_to_battlefield(0, catalog::brambleguard_veteran()); // 3/4 Raccoon
    let moose = g.add_card_to_hand(0, catalog::galewind_moose());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a 6-mana spell");
    drain_stack(&mut g);
    let cp = g.computed_permanent(vet).unwrap();
    assert_eq!(cp.power, 4, "Raccoon +1/+1");
    assert!(cp.keywords.contains(&crate::card::Keyword::Vigilance), "and vigilance");
}

/// Attack-in-the-Box may pump itself +4/+0 when it attacks.
#[test]
fn attack_in_the_box_may_pump() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let box_id = g.add_card_to_battlefield(0, catalog::attack_in_the_box()); // 2/4
    g.clear_sickness(box_id);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: box_id, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(box_id).unwrap().power, 6, "opted into +4/+0");
}

/// Arbiter of Woe: sacrifice a creature to cast; ETB drains each opponent and
/// refills you.
#[test]
fn arbiter_of_woe_additional_cost_and_drain() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder to sacrifice
    g.add_card_to_hand(1, catalog::grizzly_bears()); // opponent has a card to discard
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let arbiter = g.add_card_to_hand(0, catalog::arbiter_of_woe());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let opp_hand = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: arbiter, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arbiter (sacrificing the bear)");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "opponent loses 2");
    assert_eq!(g.players[0].life, my_life + 2, "you gain 2");
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discards");
}
