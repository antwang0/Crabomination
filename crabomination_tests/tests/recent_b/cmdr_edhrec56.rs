//! Commander: EDHREC ranks 2001-2500, one-card gaps, second batch
//! (`decks::cmdr_edhrec56`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn fill_pool(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
    g.priority.player_with_priority = seat;
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    fill_pool(g, 0);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    fill_pool(g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Staunch Throneguard makes you the monarch (CR 724.1); Rambunctious Mutt
/// destroys an opponent's artifact.
#[test]
fn throneguard_crowns_and_mutt_destroys() {
    let mut g = pod(3);
    let guard = g.add_card_to_hand(0, catalog::staunch_throneguard());
    cast_at(&mut g, guard, None).expect("throneguard");
    assert_eq!(g.monarch, Some(0));
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let mutt = g.add_card_to_hand(0, catalog::rambunctious_mutt());
    cast_at(&mut g, mutt, Some(Target::Permanent(ring))).expect("mutt");
    assert!(g.battlefield_find(ring).is_none());
}

/// Rage Reflection grants double strike; Scattershot Archer pings every
/// flier; Mechanical Glider attaches as it enters and grants flying.
#[test]
fn rage_reflection_archer_and_glider() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rage_reflection());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::DoubleStrike));

    let glider = g.add_card_to_hand(0, catalog::mechanical_glider());
    cast_at(&mut g, glider, None).expect("glider");
    assert_eq!(g.battlefield_find(glider).unwrap().attached_to, Some(bears));
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Flying));

    let archer = ready(&mut g, 1, catalog::scattershot_archer());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: archer,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("archer");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().damage, 1, "the gliding Bears took 1");
}

/// Thalia, the Survivor taxes an opponent's noncreature spell, not yours.
#[test]
fn thalia_the_survivor_taxes_opponents() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thalia_the_survivor());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(), "{{R}} no longer pays for Bolt");
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{R} plus the tax");
}

/// Sylvan Awakening animates your lands until your next turn.
#[test]
fn sylvan_awakening_animates_lands() {
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::sylvan_awakening());
    cast_at(&mut g, spell, None).expect("cast");
    let cp = g.computed_permanent(forest).unwrap();
    let types = cp.card_types();
    assert!(types.contains(&crabomination::card::CardType::Creature) && types.contains(&crabomination::card::CardType::Land));
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords().contains(&Keyword::Indestructible) && cp.keywords().contains(&Keyword::Reach));
}

/// Skyclave Shadowcat grows by sacrificing, and draws when a creature of yours
/// with a +1/+1 counter dies (CR 603.10a).
#[test]
fn skyclave_shadowcat_grows_and_draws() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let cat = ready(&mut g, 0, catalog::skyclave_shadowcat());
    let fodder = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, cat, 0, None).expect("sac the Bears");
    assert!(g.battlefield_find(fodder).is_none());
    assert_eq!(g.battlefield_find(cat).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let hand = g.players[0].hand.len();
    let mut evs = Vec::new();
    g.destroy_permanent(cat, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "it had a counter");
}

/// Syr Carah's ping to a player exiles the top card for this turn.
#[test]
fn syr_carah_impulses_on_damage() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let top = g.add_card_to_library(0, catalog::island());
    let carah = ready(&mut g, 0, catalog::syr_carah_the_bold());
    let life = g.players[1].life;
    activate(&mut g, carah, 0, Some(Target::Player(1))).expect("ping");
    assert_eq!(g.players[1].life, life - 1);
    assert!(g.exile.iter().any(|c| c.id == top), "exiled to play this turn");
}

/// Sarevok drains the active player at their end step only if no permanent
/// left the battlefield this turn.
#[test]
fn sarevok_punishes_a_quiet_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sarevok_deathbringer());
    g.active_player_idx = 1;
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    g.players[0].permanent_left_battlefield_this_turn = true;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "something left: no drain");
}

/// Tax Collector's default mode taxes opponents' spells until your next turn.
#[test]
fn tax_collector_taxes_or_detains() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let tc = g.add_card_to_hand(0, catalog::tax_collector());
    cast_at(&mut g, tc, Some(Target::Permanent(giant))).expect("cast");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let taxed = g
        .perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err();
    let detained = g.battlefield_find(giant).unwrap().detained_by.is_some();
    assert!(taxed || detained, "one mode applied");
}
