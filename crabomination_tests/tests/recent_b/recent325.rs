//! Tests for the recent325 Duskmourn / Bloomburrow / Tarkir gap batch.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::effects::EffectContext;
use crabomination::game::{GameState, drain_stack, two_player_game};
use crabomination::mana::Color;

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(12);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Miasma Demon shrinks exactly as many creatures as it discarded cards.
#[test]
fn miasma_demon_shrinks_one_per_discard() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pitch = g.add_card_to_hand(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Discard(vec![pitch])]));
    let etb = catalog::miasma_demon().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&etb, &ctx).unwrap();
    let shrunk = [a, b]
        .iter()
        .filter(|id| g.computed_permanent(**id).is_some_and(|c| c.power < 2))
        .count();
    assert_eq!(shrunk, 1, "one card discarded, one creature shrunk");
}

/// Stay Hidden, Stay Silent taps its host and keeps it down.
#[test]
fn stay_hidden_taps_and_locks_the_host() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::stay_hidden_stay_silent());
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).unwrap().tapped, "the Aura taps on entry");
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "and it doesn't untap");
}

/// Chainsaw pings on entry, then banks a rev counter per death and pumps by it.
#[test]
fn chainsaw_revs_on_every_death() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wielder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let saw = g.add_card_to_hand(0, catalog::chainsaw());
    cast(&mut g, saw, Some(Target::Permanent(victim)));
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "3 damage killed the 2/2");
    assert_eq!(g.battlefield_find(saw).unwrap().counter_count(CounterType::Rev), 1);
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: saw, target: wielder }).expect("equip");
    assert_eq!(g.computed_permanent(wielder).unwrap().power, 3, "+1/+0 per rev counter");
}

/// Dissection Tools manifests its own wielder and attaches to it.
#[test]
fn dissection_tools_arms_the_creature_it_manifests() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let tools = g.add_card_to_hand(0, catalog::dissection_tools());
    cast(&mut g, tools, None);
    let host = g.battlefield_find(tools).unwrap().attached_to.expect("attached to the manifest");
    let view = g.computed_permanent(host).unwrap();
    assert!(view.keywords.contains(&Keyword::Deathtouch) && view.keywords.contains(&Keyword::Lifelink));
    assert_eq!(view.power, 4, "a 2/2 manifest with +2/+2");
}

/// Unidentified Hovership exiles a small creature until it leaves.
#[test]
fn hovership_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ship = g.add_card_to_hand(0, catalog::unidentified_hovership());
    cast(&mut g, ship, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    let mut events = vec![];
    g.destroy_permanent(ship, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "back when the ship leaves");
}

/// Thornvault Forager's forage ability pays two mana of any colors.
#[test]
fn thornvault_forager_forages_for_two() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_graveyard(0, catalog::mountain());
    }
    let forager = g.add_card_to_battlefield(0, catalog::thornvault_forager());
    g.clear_sickness(forager);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forager,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("forage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 3, "three cards foraged away");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Hoarder's Overflow banks a stash counter on entry and cashes them in.
#[test]
fn hoarders_overflow_trades_stash_counters_for_cards() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::mountain());
    }
    g.add_card_to_hand(0, catalog::mountain());
    let overflow = g.add_card_to_hand(0, catalog::hoarders_overflow());
    cast(&mut g, overflow, None);
    let counters = g.battlefield_find(overflow).unwrap().counter_count(CounterType::Stash);
    assert_eq!(counters, 1, "one on entry");
    activate(&mut g, overflow, 0, None);
    assert_eq!(g.players[0].hand.len(), 1, "hand discarded, one card back per counter");
}

/// Festival of Embers exiles what would hit your graveyard.
#[test]
fn festival_of_embers_exiles_your_graveyard_bound_cards() {
    let mut g = two_player_game();
    let festival = g.add_card_to_hand(0, catalog::festival_of_embers());
    cast(&mut g, festival, None);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 0, "nothing reached the graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Camellia hands out menace and turns eaten Food into Squirrels.
#[test]
fn camellia_menaces_squirrels_and_eats_food() {
    let mut g = two_player_game();
    let camellia = g.add_card_to_battlefield(0, catalog::camellia_the_seedmiser());
    let food = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    let mut events = vec![];
    g.sacrifice_one(food, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let squirrel = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Squirrel")
        .expect("a Squirrel joined the board")
        .id;
    assert!(g.computed_permanent(squirrel).unwrap().keywords.contains(&Keyword::Menace));
    assert!(g.battlefield_find(camellia).is_some());
}

/// Reverberating Summons animates only after two spells.
#[test]
fn reverberating_summons_needs_two_spells() {
    let mut g = two_player_game();
    let enchant = g.add_card_to_hand(0, catalog::reverberating_summons());
    cast(&mut g, enchant, None);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(!g.computed_permanent(enchant).unwrap().card_types.contains(&crabomination::card::CardType::Creature), "one spell isn't enough");
    let second = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, second, None);
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let view = g.computed_permanent(enchant).unwrap();
    assert!(view.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(view.power == 3 && view.keywords.contains(&Keyword::Haste));
}

/// Stillness in Motion mills you and, on an empty library, rebuilds the top.
#[test]
fn stillness_in_motion_rebuilds_an_empty_library() {
    let mut g = two_player_game();
    let stillness = g.add_card_to_battlefield(0, catalog::stillness_in_motion());
    for _ in 0..8 {
        g.add_card_to_graveyard(0, catalog::mountain());
    }
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 5, "five cards came back on top");
    assert!(g.battlefield_find(stillness).is_none(), "the enchantment exiled itself");
}

/// Rite of Renewal buys back permanents and exiles itself.
#[test]
fn rite_of_renewal_exiles_itself() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let rite = g.add_card_to_hand(0, catalog::rite_of_renewal());
    cast(&mut g, rite, Some(Target::Player(0)));
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
    assert!(g.exile.iter().any(|c| c.id == rite), "exiled on resolution");
}

/// Dalkovan Encampment taps for colorless, then for R or W, then makes Warriors.
#[test]
fn dalkovan_encampment_pays_and_musters() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::dalkovan_encampment());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
    g.battlefield.iter_mut().find(|c| c.id == land).unwrap().tapped = false;
    activate(&mut g, land, 2, None);
    let warriors = g.battlefield.iter().filter(|c| c.definition.name == "Warrior").count();
    assert_eq!(warriors, 2);
}

/// Silent Hallcreeper never repeats a mode.
#[test]
fn silent_hallcreeper_picks_a_fresh_mode_each_hit() {
    let mut g = two_player_game();
    let creeper = g.add_card_to_battlefield(0, catalog::silent_hallcreeper());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hit = catalog::silent_hallcreeper().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        source: Some(creeper),
        targets: vec![Target::Permanent(ally)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&hit, &ctx).unwrap();
    let after_one = g.battlefield_find(creeper).unwrap().modes_chosen.len();
    g.resolve_effect(&hit, &ctx).unwrap();
    let after_two = g.battlefield_find(creeper).unwrap().modes_chosen.len();
    assert_eq!((after_one, after_two), (1, 2), "a different mode each time");
}

/// Leyline of Mutation can start the game on the battlefield.
#[test]
fn leyline_of_mutation_starts_in_play() {
    let leyline = catalog::leyline_of_mutation();
    assert!(leyline.opening_hand.is_some(), "opening-hand clause present");
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::leyline_of_mutation());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("WUBRG pays for anything");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
}
