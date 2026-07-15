//! Functionality tests for `catalog::sets::decks::recent141` (WOE wave 14).

use crabomination::catalog;
use crabomination::card::CounterType;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

/// Lady of Laughter draws at your end step when Celebration is active.
#[test]
fn lady_of_laughter_celebration_draw() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::lady_of_laughter());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.players[0].nonland_permanents_entered_this_turn = 2;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "Celebration end-step draw");
}

/// Sharae taps an opponent's creature and puts a stun counter on it.
#[test]
fn sharae_taps_and_stuns() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sharae_of_numbing_depths());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id, Some(Target::Permanent(enemy)), None);
    let e = g.battlefield_find(enemy).unwrap();
    assert!(e.tapped, "opponent creature tapped");
    assert_eq!(e.counter_count(CounterType::Stun), 1, "stun counter added");
}

/// Sharae's "whenever you tap …" fires only when the tap is your effect, not
/// when the opponent's own creature becomes tapped.
#[test]
fn sharae_you_tapped_actor_gating() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sharae_of_numbing_depths());
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    // Opponent taps their own creature — no draw.
    let hand = g.players[0].hand.len();
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(1) }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "opponent self-tap does not trigger");
    // You tap it — draw once.
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(0) }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "your tap draws a card");
}

/// Ingenious Prodigy enters with X +1/+1 counters and cashes one for a card.
#[test]
fn ingenious_prodigy_x_counters_then_draw() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::ingenious_prodigy());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.add_card_to_library(0, catalog::grizzly_bears());
    cast(&mut g, id, None, Some(2));
    let prod = g.battlefield.iter().find(|c| c.definition.name == "Ingenious Prodigy").unwrap().id;
    assert_eq!(g.battlefield_find(prod).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(prod).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "spent a counter");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Malevolent Witchkite sacrifices tokens on entry and draws that many.
#[test]
fn malevolent_witchkite_sacs_and_draws() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    g.add_token_to_battlefield(0, &crabomination::game::effects::treasure_token());
    let id = g.add_card_to_hand(0, catalog::malevolent_witchkite());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    cast(&mut g, id, None, None);
    // Witchkite leaves hand (−1) and two draws (+2) → net +1.
    assert_eq!(hand + 1, g.players[0].hand.len(), "drew two for two sacrifices");
    assert!(!g.battlefield.iter().any(|c| c.is_token), "both tokens sacrificed");
}

/// Obyra drains each opponent when another Faerie you control enters.
#[test]
fn obyra_drains_on_faerie_entry() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::obyra_dreaming_duelist());
    let faerie = g.add_card_to_hand(0, catalog::spellstutter_sprite());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    cast(&mut g, faerie, None, None);
    assert_eq!(g.players[1].life, life - 1, "opponent loses 1 on Faerie ETB");
}

/// Old Flitterfang makes a Food at end step when a creature died this turn.
#[test]
fn old_flitterfang_end_step_food() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::old_flitterfang());
    g.players[0].creatures_died_this_turn = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "made a Food");
}

/// Unruly Catapult untaps when you cast an instant.
#[test]
fn unruly_catapult_untaps_on_instant() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let cat = g.add_card_to_battlefield(0, catalog::unruly_catapult());
    g.battlefield_find_mut(cat).unwrap().tapped = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1)), None);
    assert!(!g.battlefield_find(cat).unwrap().tapped, "catapult untapped on instant cast");
}

/// Realm-Scorcher Hellkite, when bargained, adds four mana on entry.
#[test]
fn realm_scorcher_bargained_ramps() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let token = g.add_token_to_battlefield(0, &crabomination::game::effects::treasure_token());
    let id = g.add_card_to_hand(0, catalog::realm_scorcher_hellkite());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id,
        sacrifice: Some(token),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 4, "bargained ETB floated four mana");
}

/// Tough Cookie mints a Food and can animate a noncreature artifact into a 4/4.
#[test]
fn tough_cookie_food_and_animate() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::tough_cookie());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id, None, None);
    let cookie = g.battlefield.iter().find(|c| c.definition.name == "Tough Cookie").unwrap().id;
    let food = g.battlefield.iter().find(|c| c.definition.name == "Food").unwrap().id;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cookie,
        ability_index: 0,
        target: Some(Target::Permanent(food)),
        additional_targets: vec![],
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    let f = g.computed_permanent(food).unwrap();
    assert_eq!((f.power, f.toughness), (4, 4), "Food animated to 4/4");
    assert!(f.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
}
