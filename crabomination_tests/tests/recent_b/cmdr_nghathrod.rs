//! Commander: the Mind Flayarrrs precon's missing cards (`decks::cmdr_nghathrod`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::Blue, Color::Black] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
}

fn cast(g: &mut GameState, card_id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn always_yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(16),
    ));
}

/// CR 601.2b — Dusk Mangler's "sacrifice a creature, discard a card, or pay 4
/// life": exactly one is paid. At 40 life the caster pays life and keeps the
/// board and hand; with a token around, the token goes instead. The ETB hits
/// each opponent three ways.
#[test]
fn cr_601_2b_dusk_mangler_pays_one_of_three_costs() {
    let mut g = main_phase();
    g.players[0].life = 40;
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let keep = g.add_card_to_hand(0, catalog::island());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    let opp_life = g.players[1].life;
    let mangler = g.add_card_to_hand(0, catalog::dusk_mangler());
    flood(&mut g);
    cast(&mut g, mangler, None);
    assert_eq!(g.players[0].life, 36, "paid the 4 life");
    assert!(g.battlefield_find(bears).is_some() && g.players[0].hand.iter().any(|c| c.id == keep));
    assert!(g.battlefield_find(theirs).is_none(), "each opponent sacrificed");
    assert!(g.players[1].hand.is_empty(), "and discarded");
    assert_eq!(g.players[1].life, opp_life - 4, "and lost 4");

    let mut g = main_phase();
    g.players[0].life = 40;
    let token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(token).unwrap().is_token = true;
    let mangler = g.add_card_to_hand(0, catalog::dusk_mangler());
    flood(&mut g);
    cast(&mut g, mangler, None);
    assert_eq!(g.players[0].life, 40, "a token is the cheapest option");
    assert!(g.battlefield_find(token).is_none());
}

/// Port of Karfell mills four, then returns a creature card — a milled one
/// counts — tapped.
#[test]
fn port_of_karfell_mills_then_reanimates_tapped() {
    let mut g = main_phase();
    let port = g.add_card_to_battlefield(0, catalog::port_of_karfell());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::serra_angel());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: port,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(port).is_none(), "sacrificed");
    let angel = g.battlefield.iter().find(|c| c.definition.name == "Serra Angel").expect("returned");
    assert!(angel.tapped);
}

/// Sewer Nemesis is as big as the chosen player's graveyard and mills them
/// when they cast a spell.
#[test]
fn sewer_nemesis_sizes_off_and_mills_the_chosen_player() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let nemesis = g.add_card_to_hand(0, catalog::sewer_nemesis());
    flood(&mut g);
    cast(&mut g, nemesis, None);
    let nemesis = g.battlefield.iter().find(|c| c.definition.name == "Sewer Nemesis").unwrap().id;
    assert_eq!(g.computed_permanent(nemesis).unwrap().power, 3);
    // The chosen opponent casts a spell.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponent casts");
    drain_stack(&mut g);
    // Three + the milled card + the resolved Bolt.
    assert_eq!(g.players[1].graveyard.len(), 5);
    assert_eq!(g.computed_permanent(nemesis).unwrap().power, 5);
}

/// Memory Plunder casts an instant from an opponent's graveyard for free.
#[test]
fn memory_plunder_casts_an_opponents_instant_free() {
    let mut g = main_phase();
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let plunder = g.add_card_to_hand(0, catalog::memory_plunder());
    always_yes(&mut g);
    let life = g.players[1].life;
    flood(&mut g);
    let pool = g.players[0].mana_pool.total();
    cast(&mut g, plunder, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[1].life, life - 3, "the plundered Bolt resolved");
    assert_eq!(pool - g.players[0].mana_pool.total(), 4, "only Plunder itself was paid for");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "back to its owner's yard");
}

/// CR 702.62b — Nihilith's trigger functions only while it is suspended: a
/// card put into an opponent's graveyard takes a time counter off it in
/// exile, and a Nihilith on the battlefield ignores the same event.
#[test]
fn cr_702_62b_nihilith_ticks_while_suspended() {
    let mut g = main_phase();
    let nihilith = g.add_card_to_hand(0, catalog::nihilith());
    flood(&mut g);
    g.perform_action(GameAction::Suspend { card_id: nihilith }).expect("suspend");
    drain_stack(&mut g);
    let counters = |g: &GameState| {
        g.exile.iter().find(|c| c.id == nihilith).map(|c| c.counter_count(CounterType::Time))
    };
    assert_eq!(counters(&g), Some(7));
    always_yes(&mut g);
    for _ in 0..2 {
        g.add_card_to_library(1, catalog::island());
    }
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let mill = Effect::Mill {
        who: Selector::Player(crabomination::effect::PlayerRef::EachOpponent),
        amount: crabomination::card::Value::Const(2),
    };
    let evs = g.resolve_effect(&mill, &ctx).expect("mill");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(counters(&g), Some(5), "one counter per card");

    // On the battlefield it is not suspended, so nothing triggers.
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::nihilith());
    g.add_card_to_library(1, catalog::island());
    let evs = g.resolve_effect(&mill, &ctx).expect("mill");
    g.dispatch_triggers_for_events(&evs);
    assert!(g.stack.is_empty());
}

/// From the Catacombs steals a creature card from an opponent's graveyard
/// with a corpse counter, hands the caster the initiative, and the creature
/// is exiled rather than bounced (CR 614).
#[test]
fn from_the_catacombs_reanimates_takes_the_initiative_and_exiles_on_leave() {
    let mut g = main_phase();
    let angel = g.add_card_to_graveyard(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::from_the_catacombs());
    flood(&mut g);
    cast(&mut g, spell, Some(Target::Permanent(angel)));
    let on_bf = g.battlefield_find(angel).expect("reanimated");
    assert_eq!(on_bf.controller, 0);
    assert_eq!(on_bf.counter_count(CounterType::Corpse), 1);
    assert_eq!(g.initiative, Some(0));
    let unsummon = g.add_card_to_hand(0, catalog::unsummon());
    flood(&mut g);
    cast(&mut g, unsummon, Some(Target::Permanent(angel)));
    assert!(g.exile.iter().any(|c| c.id == angel), "exiled instead of returning to hand");
}

/// Haunted One grants its commander creatures "whenever this becomes tapped,
/// it and your creatures sharing a type get +2/+0 and undying"; a
/// non-commander creature gets nothing.
#[test]
fn haunted_one_pumps_the_commanders_tribe_when_it_taps() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::haunted_one());
    let captain = g.add_card_to_battlefield(0, catalog::captain_nghathrod());
    g.players[0].commanders.push(captain);
    let horror = g.add_card_to_battlefield(0, catalog::dusk_mangler());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(captain)), 0, 0);
    let evs = g.resolve_effect(&Effect::Tap { what: Selector::Target(0) }, &ctx).expect("tap");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let p = |g: &GameState, id| g.computed_permanent(id).unwrap().power;
    assert_eq!(p(&g, captain), 5, "the commander itself");
    assert_eq!(p(&g, horror), 7, "a fellow Horror");
    assert_eq!(p(&g, bears), 2, "a Bear shares no type");
    assert!(g.computed_permanent(horror).unwrap().keywords().contains(&Keyword::Undying));
}
