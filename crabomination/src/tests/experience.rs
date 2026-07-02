//! Functionality tests for `catalog::sets::decks::experience` (the experience
//! counter cycle + `Player.experience`, `Effect::AddExperience`,
//! `Value::ControllerExperience`, `CostReductionPerControllerExperience`, and
//! `DynamicPt::ControllerExperience`).

use crate::catalog;
use crate::card::CounterType;
use crate::game::actions::cost_reduction_for_spell;
use crate::game::two_player_game;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Cast Lightning Bolt from P0 at P1's face (an instant cast).
fn p0_bolt_face(g: &mut GameState) {
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(g);
}

#[test]
fn mizzix_gains_experience_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mizzix_of_the_izmagnus());
    p0_bolt_face(&mut g);
    assert_eq!(g.players[0].experience, 1, "casting an instant gave an experience counter");
}

#[test]
fn mizzix_reduces_is_cost_by_experience() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mizzix_of_the_izmagnus());
    g.players[0].experience = 2;
    // A sorcery in hand: its generic cost should be reduced by 2.
    let id = g.add_card_to_hand(0, catalog::rise_from_the_tides());
    let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
    assert_eq!(cost_reduction_for_spell(&g, 0, &card, None), 2, "reduced by experience count");
}

#[test]
fn ezuri_gains_experience_on_small_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ezuri_claw_of_progress());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Memnite is a 1/1 (power ≤ 1) and free to cast.
    let m = g.add_card_to_hand(0, catalog::memnite());
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memnite castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].experience, 1, "a power-1 creature entering gave experience");
}

#[test]
fn ezuri_distributes_counters_equal_to_experience() {
    let mut g = two_player_game();
    // Target creature first in battlefield order so the combat trigger auto-targets it.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::ezuri_claw_of_progress());
    g.players[0].experience = 3;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "put X +1/+1 counters where X = experience",
    );
}

#[test]
fn daxos_makes_experience_sized_spirit() {
    let mut g = two_player_game();
    let daxos = g.add_card_to_battlefield(0, catalog::daxos_the_returned());
    g.players[0].experience = 3;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: daxos, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None,
    }).expect("Daxos token ability");
    drain_stack(&mut g);
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit").expect("Spirit minted");
    let cp = g.computed_permanent(spirit.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "token P/T = experience at mint time");
}

#[test]
fn daxos_gains_experience_on_enchantment_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::daxos_the_returned());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Any enchantment spell; give plenty of mana.
    for _ in 0..6 { g.players[0].mana_pool.add_colorless(1); }
    g.players[0].mana_pool.add(Color::White, 2);
    let e = g.add_card_to_hand(0, catalog::dawn_of_hope());
    g.perform_action(GameAction::CastSpell {
        card_id: e, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchantment castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].experience, 1, "casting an enchantment gave experience");
}

#[test]
fn kalemne_grows_with_experience() {
    let mut g = two_player_game();
    let k = g.add_card_to_battlefield(0, catalog::kalemne_disciple_of_iroas());
    // Base 2/4.
    let cp = g.computed_permanent(k).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "base P/T");
    g.players[0].experience = 3;
    let cp = g.computed_permanent(k).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 7), "+1/+1 per experience counter");
}

#[test]
fn kalemne_experience_only_on_big_creature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kalemne_disciple_of_iroas());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Memnite (mv 0) — no experience.
    let m = g.add_card_to_hand(0, catalog::memnite());
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memnite castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].experience, 0, "a cheap creature gives no experience");
}

#[test]
fn meren_gains_experience_when_your_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::meren_of_clan_nel_toth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().damage = 2; // lethal → CreatureDied
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].experience, 1, "another creature dying gave experience");
}

#[test]
fn meren_reanimates_when_experience_is_enough() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::meren_of_clan_nel_toth());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // mv 2
    g.players[0].experience = 3;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "mv ≤ experience → returned to the battlefield");
}

#[test]
fn meren_returns_to_hand_when_experience_is_short() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::meren_of_clan_nel_toth());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // mv 2
    g.players[0].experience = 1; // < 2
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "not reanimated");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "mv > experience → to hand");
}
