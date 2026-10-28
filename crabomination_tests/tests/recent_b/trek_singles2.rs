//! Star Trek: the second batch of EDHREC closers (`decks::trek_singles2`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn counters(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(kind))
}

/// Dr. M'Benga (CR 614.16): counters on your creatures double, and your other
/// countered creatures have hexproof.
#[test]
fn mbenga_doubles_creature_counters_and_shields_them() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::dr_joseph_mbenga());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ring = g.add_card_to_battlefield(0, catalog::class_f_shuttlecraft());
    assert!(!g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Hexproof));
    let doc = catalog::the_doctor_emh().triggered_abilities[0].effect.clone();
    g.resolve_effect(&doc, &crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0))
        .expect("resolve");
    // One counter doubled, then the "each creature with a counter" one doubled.
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 4);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Hexproof));
    assert_eq!(counters(&g, ring, CounterType::Charge), 0);
}

/// Lt. Cmdr. Geordi La Forge: charge counters on your artifacts double;
/// other kinds don't.
#[test]
fn geordi_doubles_charge_counters_only() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::lt_cmdr_geordi_la_forge());
    let ship = g.add_card_to_battlefield(0, catalog::class_f_shuttlecraft());
    activate(&mut g, ship, 1, None);
    assert_eq!(counters(&g, ship, CounterType::Charge), 6, "stationed by the 3-power Geordi, doubled");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let doc = catalog::the_doctor_emh().triggered_abilities[0].effect.clone();
    g.resolve_effect(&doc, &crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0))
        .expect("resolve");
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 2, "+1/+1 counters aren't doubled");
}

/// Data, New Life grows with each card you draw.
#[test]
fn data_new_life_grows_on_draws() {
    let mut g = pod(2);
    let data = g.add_card_to_battlefield(0, catalog::data_new_life());
    for _ in 0..3 {
        let mut ev = Vec::new();
        g.draw_one(0, &mut ev);
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
    }
    assert_eq!(counters(&g, data, CounterType::PlusOnePlusOne), 3);
}

/// Jadzia Dax: her first-main counter, then {G} per counter.
#[test]
fn jadzia_dax_ramps_with_her_counters() {
    let mut g = pod(2);
    let jadzia = g.add_card_to_battlefield(0, catalog::jadzia_dax_living_legacy());
    g.battlefield_find_mut(jadzia).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.step = TurnStep::Draw;
    for _ in 0..20 {
        if g.step == TurnStep::PreCombatMain {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(counters(&g, jadzia, CounterType::PlusOnePlusOne), 3);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
}

/// Federation Outpost enters tapped with one other land, untapped (and with a
/// rider) with two.
#[test]
fn federation_outpost_needs_two_other_lands() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::plains());
    let a = g.add_card_to_hand(0, catalog::federation_outpost());
    g.perform_action(GameAction::PlayLand(a)).expect("land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped);
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let life = g.players[0].life;
    let b = g.add_card_to_hand(0, catalog::federation_outpost());
    g.perform_action(GameAction::PlayLand(b)).expect("land");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(b).unwrap().tapped);
    assert_eq!(g.players[0].life, life + 2);
}

/// Captain Archer's Federation pump counts creature types among non-Borg
/// creatures you control (Porthos the Dog included).
#[test]
fn archer_pumps_by_creature_types() {
    let mut g = pod(2);
    let archer = g.add_card_to_hand(0, catalog::captain_jonathan_archer());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: archer,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(archer);
    flood(&mut g, 0);
    activate(&mut g, archer, 0, Some(Target::Permanent(archer)));
    // Human, Officer, Dog, Bear: four types.
    let c = g.computed_permanent(archer).unwrap();
    assert_eq!((c.power, c.toughness), (7, 7));
}

/// Excalbian Gamemaster: an Officer per differently named land.
#[test]
fn excalbian_gamemaster_counts_land_names() {
    let mut g = pod(2);
    for land in [catalog::plains(), catalog::plains(), catalog::island(), catalog::forest()] {
        g.add_card_to_battlefield(0, land);
    }
    let ex = g.add_card_to_hand(0, catalog::excalbian_gamemaster());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: ex, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let officers = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Officer").count();
    assert_eq!(officers, 3);
}

/// Sela: one or more cards leaving your graveyard at once make one Romulan.
#[test]
fn sela_makes_a_romulan_per_graveyard_batch() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sela_tal_shiar_paradox());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    let purge = crabomination::effect::Effect::Exile {
        what: crabomination::effect::Selector::CardsInZone {
            who: crabomination::effect::PlayerRef::You,
            zone: crabomination::card::Zone::Graveyard,
            filter: crabomination::card::SelectionRequirement::Any,
        },
    };
    let ev = g.resolve_effect(&purge, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let romulans: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Romulan").collect();
    assert_eq!(romulans.len(), 1);
    assert!(romulans[0].tapped);
}
