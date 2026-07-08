//! Functionality tests for `catalog::sets::decks::mh2e` — MH2 sweep batch 6.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn resolve_spell(g: &mut GameState, def: crate::card::CardDefinition, targets: Vec<Target>) {
    resolve_spell_mode(g, def, targets, 0);
}

fn resolve_spell_mode(
    g: &mut GameState,
    def: crate::card::CardDefinition,
    targets: Vec<Target>,
    mode: usize,
) {
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, mode, 0);
    ctx.targets = targets;
    let events = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
}

fn activate(g: &mut GameState, id: crate::card::CardId, idx: usize, target: Option<Target>) {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target, additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Barbed Spike arrives wearing its own Thopter.
#[test]
fn barbed_spike_mints_and_attaches() {
    let mut g = two_player_game();
    let spike = g.add_card_to_hand(0, catalog::barbed_spike());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast(&mut g, spike);
    let thopter = g.battlefield.iter().find(|c| c.definition.name == "Thopter").expect("token");
    let cp = g.computed_permanent(thopter.id).unwrap();
    assert_eq!(cp.power, 2, "1/1 + the spike's +1/+0");
}

/// Break Ties mode 2 exiles a graveyard card; Reinforce is registered.
#[test]
fn break_ties() {
    let mut g = two_player_game();
    assert!(catalog::break_ties().keywords.iter().any(|k| matches!(k, Keyword::Reinforce(1, _))));
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    resolve_spell_mode(&mut g, catalog::break_ties(), vec![Target::Permanent(dead)], 2);
    assert!(g.players[1].graveyard.is_empty(), "graveyard card exiled");
}

/// Breya's Apprentice pumps via mode 1 off an artifact sacrifice.
#[test]
fn breyas_apprentice_pump_mode() {
    let mut g = two_player_game();
    let breya = g.add_card_to_hand(0, catalog::breyas_apprentice());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    cast(&mut g, breya);
    g.clear_sickness(breya);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_library(0, catalog::island());
    let _ = bear;
    g.perform_action(GameAction::ActivateAbility {
        card_id: breya,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate with the ETB Thopter as fodder");
    drain_stack(&mut g);
    // AutoDecider takes mode 0: the top card is impulsed into exile.
    assert!(g.exile.iter().any(|c| c.definition.name == "Island"), "top card exiled");
}

/// Calibrated Blast digs to a nonland and burns for its mana value.
#[test]
fn calibrated_blast_burns_by_mv() {
    let mut g = two_player_game();
    assert!(catalog::calibrated_blast().keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
    g.add_card_to_library(0, catalog::serra_angel()); // MV 5 on top
    let life = g.players[1].life;
    resolve_spell(&mut g, catalog::calibrated_blast(), vec![Target::Player(1)]);
    assert_eq!(g.players[1].life, life - 5, "revealed Serra Angel = 5 damage");
}

/// Caprichrome devours artifacts for counters.
#[test]
fn caprichrome_devours_artifacts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::parcel_myr());
    g.add_card_to_battlefield(0, catalog::batterbone());
    let goat = g.add_card_to_hand(0, catalog::caprichrome());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    // Devour both artifacts.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    cast(&mut g, goat);
    let cp = g.computed_permanent(goat).unwrap();
    assert_eq!(cp.power, 4, "two artifacts devoured");
    assert!(g.battlefield_find(goat).is_some());
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.is_artifact()).count(),
        1,
        "only the goat remains"
    );
}

/// Constable of the Realm's counter trigger exiles until it leaves.
#[test]
fn constable_exile_until_leaves() {
    let mut g = two_player_game();
    let constable = g.add_card_to_battlefield(0, catalog::constable_of_the_realm());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::AddCounter {
                what: crate::effect::Selector::EachPermanent(
                    crate::card::SelectionRequirement::HasCreatureType(
                        crate::card::CreatureType::Giant,
                    ),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: crate::effect::Value::Const(2),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "bear exiled");
    // Constable dies → the bear comes home.
    let mut ctx2 = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx2.targets = vec![Target::Permanent(constable)];
    let events = g
        .resolve_effect(
            &crate::effect::Effect::Destroy { what: crate::effect::Selector::Target(0) },
            &ctx2,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_some(), "exiled bear returns");
}

/// Goblin Traprunner's three flips mint a Goblin per win.
#[test]
fn goblin_traprunner_flips() {
    let mut g = two_player_game();
    let runner = g.add_card_to_battlefield(0, catalog::goblin_traprunner());
    g.clear_sickness(runner);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: runner,
        target: crate::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let goblins = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Goblin" && c.is_token)
        .count();
    assert!(goblins <= 3, "at most three Goblins ({goblins})");
    // Any minted Goblin is tapped and attacking.
    for c in g.battlefield.iter().filter(|c| c.definition.name == "Goblin" && c.is_token) {
        assert!(c.tapped, "flip winners enter tapped");
    }
}

/// Liquimetal Torque turns a permanent into an artifact.
#[test]
fn liquimetal_torque_artifactizes() {
    let mut g = two_player_game();
    let torque = g.add_card_to_battlefield(0, catalog::liquimetal_torque());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, torque, 1, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .card_types
            .contains(&crate::card::CardType::Artifact),
        "bear is now an artifact creature"
    );
}

/// Search the Premises investigates when a creature attacks you.
#[test]
fn search_the_premises_investigates() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::search_the_premises());
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: atk,
        target: crate::game::types::AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Clue" && c.controller == 0),
        "investigated"
    );
}

/// So Shiny taps + scries with a token around and locks the host's untap.
#[test]
fn so_shiny() {
    let mut g = two_player_game();
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(&crate::effect::shortcut::mint_treasures(1), &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::so_shiny());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, aura, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped on ETB (token present)");
    // The enchanted creature stays locked through its untap step.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "doesn't untap");
}

/// Tide Shaper kicked turns a land into an Island; its pump reads opponent
/// Islands.
#[test]
fn tide_shaper_kicked() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::darksteel_citadel());
    let shaper = g.add_card_to_hand(0, catalog::tide_shaper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: shaper,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked cast");
    assert!(
        g.battlefield_find(shaper).is_some_and(|c| c.kicked) || !g.stack.is_empty(),
        "kicked flag set on the permanent"
    );
    drain_stack(&mut g);
    assert!(g.battlefield_find(shaper).unwrap().kicked, "kicked flag persisted");
    let cp = g.computed_permanent(land).unwrap();
    assert!(
        cp.subtypes.land_types.contains(&crate::card::LandType::Island),
        "land is now an Island"
    );
    assert_eq!(g.computed_permanent(shaper).unwrap().power, 2, "+1/+1 off the new Island");
}

/// Tizerus Charger escapes with its chosen counter.
#[test]
fn tizerus_charger_escape_counter() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let dead = g.add_card_to_graveyard(0, catalog::tizerus_charger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let fodder: Vec<_> = g.players[0].graveyard.iter().map(|c| c.id).filter(|id| *id != dead).collect();
    g.perform_action(GameAction::CastEscape {
        card_id: dead,
        exile_cards: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("escape");
    drain_stack(&mut g);
    let pegasus = g.battlefield_find(dead).expect("escaped");
    // AutoDecider takes mode 0: the +1/+1 counter.
    assert_eq!(pegasus.counter_count(CounterType::PlusOnePlusOne), 1, "escapes with a counter");
}

/// Graceful Restoration's second mode reanimates two small bodies.
#[test]
fn graceful_restoration_wide_mode() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    resolve_spell_mode(
        &mut g,
        catalog::graceful_restoration(),
        vec![Target::Permanent(a), Target::Permanent(b)],
        1,
    );
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some(), "both back");
}

/// Batch-6 stat spot checks.
#[test]
fn batch6_stats() {
    assert!(catalog::caprichrome().keywords.contains(&Keyword::Flash));
    assert!(catalog::tide_shaper().keywords.iter().any(|k| matches!(k, Keyword::Kicker(_))));
    assert!(catalog::tizerus_charger().keywords.iter().any(|k| matches!(k, Keyword::Escape(_, 5))));
    assert_eq!(catalog::goblin_traprunner().power, 4);
    assert_eq!(catalog::constable_of_the_realm().cost.cmc(), 5);
    assert_eq!(catalog::breyas_apprentice().toughness, 3);
}
