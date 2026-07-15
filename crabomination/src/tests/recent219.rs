//! Functionality tests for `catalog::sets::decks::recent219`.

use crate::card::CounterType;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::effects::EffectContext;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Flamewake Phoenix returns from the graveyard at combat when you're Ferocious
/// and pay {R}.
#[test]
fn flamewake_phoenix_returns_when_ferocious() {
    let mut g = two_player_game();
    let phoenix = g.add_card_to_graveyard(0, catalog::flamewake_phoenix());
    // A power-4+ creature enables Ferocious.
    g.add_card_to_battlefield(0, catalog::griselbrand()); // 7/7
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.battlefield_find(phoenix).is_some(), "phoenix returned to the battlefield");
}

/// Cryptic Caves draws a card when you control five or more lands.
#[test]
fn cryptic_caves_draws_with_five_lands() {
    let mut g = two_player_game();
    let caves = g.add_card_to_battlefield(0, catalog::cryptic_caves());
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::forest()); }
    g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(caves);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: caves, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac to draw with five lands");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert!(g.battlefield_find(caves).is_none(), "land sacrificed");
}

/// New Horizons puts a +1/+1 counter on a creature you control when it enters.
#[test]
fn new_horizons_boosts_on_etb() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = catalog::new_horizons();
    let effect = aura.triggered_abilities[0].effect.clone();
    let nh = g.add_card_to_battlefield(0, catalog::new_horizons());
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_trigger(nh, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Drake Hatcher removes three incubation counters to mint a 2/2 flying Drake.
#[test]
fn drake_hatcher_hatches_a_drake() {
    let mut g = two_player_game();
    let hatcher = g.add_card_to_battlefield(0, catalog::drake_hatcher());
    g.battlefield_find_mut(hatcher).unwrap().add_counters(CounterType::Incubation, 3);
    g.clear_sickness(hatcher);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let drakes = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Drake").count();
    assert_eq!(drakes(&g), 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hatcher, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("remove three incubation counters");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hatcher).unwrap().counter_count(CounterType::Incubation), 0, "counters spent");
    assert_eq!(drakes(&g), 1, "a Drake token entered");
}

/// Myojin of Night's Reach enters with a divinity counter when cast and is
/// indestructible while it has one.
#[test]
fn myojin_divinity_grants_indestructible() {
    let mut g = two_player_game();
    let myojin = catalog::myojin_of_nights_reach();
    let effect = myojin.triggered_abilities[0].effect.clone();
    let id = g.add_card_to_battlefield(0, catalog::myojin_of_nights_reach());
    g.battlefield_find_mut(id).unwrap().entered_by_cast = true;
    let ctx = EffectContext::for_trigger(id, 0, None, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Divinity), 1, "divinity counter");
    let v = g.computed_permanent(id).unwrap();
    assert!(v.keywords.contains(&crate::card::Keyword::Indestructible), "indestructible while divinity present");
}

/// Myojin's activated ability empties each opponent's hand.
#[test]
fn myojin_ability_empties_opponent_hand() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::myojin_of_nights_reach());
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Divinity, 1);
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    g.clear_sickness(id);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("remove divinity: each opponent discards their hand");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "opponent hand emptied");
}
