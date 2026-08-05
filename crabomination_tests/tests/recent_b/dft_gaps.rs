//! Aetherdrift gap batch (`decks::recent326`) — graveyard casts, exhaust,
//! speed payoffs and the Vehicles.

use crabomination::card::{CardDefinition, CardType, CounterType, Keyword, WardCost};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::actions::cost_reduction_for_spell;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Put `def` onto seat 0's battlefield and fire its ETB.
fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

fn activate(g: &mut GameState, id: CardId, index: usize, x: Option<u32>) -> Result<(), GameError> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// Wickerfolk Indomitable recasts itself out of the graveyard for 2 life plus
/// a sacrifice, and — unlike flashback — is not exiled on the way out.
#[test]
fn wickerfolk_indomitable_recasts_from_the_graveyard() {
    let mut g = main_phase();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let scarecrow = g.add_card_to_graveyard(0, catalog::wickerfolk_indomitable());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: scarecrow,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("graveyard cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == scarecrow), "it entered the battlefield");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "the sacrifice was paid");
    assert_eq!(g.players[0].life, 18, "2 life paid");
}

/// Daretti's power tracks the biggest artifact you control.
#[test]
fn daretti_power_is_the_greatest_artifact_mana_value() {
    let mut g = main_phase();
    let daretti = etb(&mut g, catalog::daretti_rocketeer_engineer());
    assert_eq!(g.computed_permanent(daretti).unwrap().power, 0, "no artifacts yet");
    g.add_card_to_battlefield(0, catalog::skyseers_chariot());
    assert_eq!(g.computed_permanent(daretti).unwrap().power, 2, "the Chariot costs two");
}

/// Mendicant Core's power counts your artifacts, itself included.
#[test]
fn mendicant_core_power_counts_artifacts() {
    let mut g = main_phase();
    let core = etb(&mut g, catalog::mendicant_core_guidelight());
    assert_eq!(g.computed_permanent(core).unwrap().power, 1, "it is its own artifact");
    g.add_card_to_battlefield(0, catalog::radiant_lotus());
    assert_eq!(g.computed_permanent(core).unwrap().power, 2);
}

/// Oviya cheats a Vehicle in from hand; an artifact entrant gets two counters.
#[test]
fn oviya_puts_a_vehicle_in_with_counters() {
    let mut g = main_phase();
    let oviya = etb(&mut g, catalog::oviya_automech_artisan());
    g.clear_sickness(oviya);
    let chariot = g.add_card_to_hand(0, catalog::skyseers_chariot());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![chariot])]));
    activate(&mut g, oviya, 0, None).expect("activate");
    let put = g.battlefield_find(chariot).expect("Chariot entered");
    assert_eq!(put.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Sita Varma's exhaust ability can only be activated once.
#[test]
fn sita_varma_exhaust_activates_once() {
    let mut g = main_phase();
    let sita = etb(&mut g, catalog::sita_varma_masked_racer());
    g.clear_sickness(sita);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    activate(&mut g, sita, 0, Some(2)).expect("first activation");
    assert_eq!(
        g.battlefield_find(sita).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
    assert!(activate(&mut g, sita, 0, Some(2)).is_err(), "exhaust is once only");
}

/// Winter shares her ward with your artifacts.
#[test]
fn winter_grants_ward_to_artifacts() {
    let mut g = main_phase();
    etb(&mut g, catalog::winter_cursed_rider());
    let lotus = g.add_card_to_battlefield(0, catalog::radiant_lotus());
    let kws = g.computed_permanent(lotus).unwrap().keywords;
    assert!(kws.contains(&Keyword::Ward(WardCost::Life(2))));
}

/// Redshift taps for as much mana as his power, spendable only on abilities.
#[test]
fn redshift_taps_for_power_in_ability_mana() {
    let mut g = main_phase();
    let redshift = etb(&mut g, catalog::redshift_rocketeer_chief());
    g.clear_sickness(redshift);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: redshift,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "power 2, ability-only");
}

/// Demonic Junker's affinity discounts it by each artifact you control.
#[test]
fn demonic_junker_has_affinity_for_artifacts() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::radiant_lotus());
    g.add_card_to_battlefield(0, catalog::skyseers_chariot());
    let junker = g.add_card_to_hand(0, catalog::demonic_junker());
    let card = g.find_card_anywhere(junker).unwrap().clone();
    assert_eq!(cost_reduction_for_spell(&g, 0, &card, None), 2, "two artifacts");
}

/// Riptide Gearhulk buries an opposing permanent third from the top.
#[test]
fn riptide_gearhulk_buries_third_from_the_top() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::mountain());
    }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bury = catalog::riptide_gearhulk().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(victim)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&bury, &ctx).unwrap();
    let lib = &g.players[1].library;
    assert_eq!(lib[lib.len() - 3].id, victim, "third from the top");
}

/// Radiant Lotus pays out three mana per artifact sacrificed.
#[test]
fn radiant_lotus_pays_three_per_artifact() {
    let mut g = main_phase();
    let lotus = g.add_card_to_battlefield(0, catalog::radiant_lotus());
    let extra = g.add_card_to_battlefield(0, catalog::skyseers_chariot());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Cards(vec![extra]),
        DecisionAnswer::Color(Color::Red),
    ]));
    let _ = extra;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: lotus,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3, "one artifact sacrificed");
}

/// Skyseer's Chariot taxes the abilities of the card name it names.
#[test]
fn skyseers_chariot_taxes_the_named_source() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::NamedCard(
        "Radiant Lotus".to_string(),
    )]));
    let chariot = g.add_card_to_battlefield(0, catalog::skyseers_chariot());
    let name_it = catalog::skyseers_chariot().as_enters_effect.unwrap();
    let ctx = EffectContext::for_ability(chariot, 0, None);
    g.resolve_effect(&name_it, &ctx).unwrap();
    let lotus = g.add_card_to_battlefield(0, catalog::radiant_lotus());
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: lotus,
            ability_index: 0,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(act(&mut g).is_err(), "the printed ability is free, but taxed two");
    g.players[0].mana_pool.add_colorless(2);
    act(&mut g).expect("payable once the tax is covered");
}

/// Push the Limit reanimates every Vehicle, animates them and hands out haste.
#[test]
fn push_the_limit_reanimates_the_vehicles() {
    let mut g = main_phase();
    let chariot = g.add_card_to_graveyard(0, catalog::skyseers_chariot());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::push_the_limit());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::NamedCard(
        "Radiant Lotus".to_string(),
    )]));
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == chariot), "the Vehicle came back");
    let view = g.computed_permanent(chariot).unwrap();
    assert!(view.card_types.contains(&CardType::Creature), "animated");
    assert!(view.keywords.contains(&Keyword::Haste));
}

/// Rise from the Wreck's four graveyard slots are all declinable.
#[test]
fn rise_from_the_wreck_takes_only_the_slots_you_fill() {
    let mut g = main_phase();
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::rise_from_the_wreck());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with one slot filled");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears));
}

/// Oviya's trample anthem reaches every attacker, not just her own.
#[test]
fn oviya_grants_trample_to_attackers() {
    let mut g = main_phase();
    etb(&mut g, catalog::oviya_automech_artisan());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bears);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }])
        .expect("attack");
    assert!(g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::Trample));
}

/// Lifecraft Engine makes your Vehicles the chosen type and pumps that type.
#[test]
fn lifecraft_engine_types_and_pumps_the_chosen_type() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        crabomination::card::CreatureType::Bear,
    )]));
    let engine = g.add_card_to_battlefield(0, catalog::lifecraft_engine());
    let name_it = catalog::lifecraft_engine().as_enters_effect.unwrap();
    g.resolve_effect(&name_it, &EffectContext::for_ability(engine, 0, None)).unwrap();
    let chariot = g.add_card_to_battlefield(0, catalog::skyseers_chariot());
    assert!(
        g.computed_permanent(chariot)
            .unwrap()
            .subtypes
            .creature_types
            .contains(&crabomination::card::CreatureType::Bear),
        "the Vehicle joined the chosen type"
    );
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bears).unwrap().power, 3, "+1/+1 for the chosen type");
}

/// Cursecloth Wrappings hands a graveyard creature a usable embalm ability.
#[test]
fn cursecloth_wrappings_grants_embalm() {
    let mut g = main_phase();
    let wraps = g.add_card_to_battlefield(0, catalog::cursecloth_wrappings());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wraps,
        ability_index: 0,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("grant embalm");
    drain_stack(&mut g);
    // Index 0 is the granted ability — Grizzly Bears prints none.
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bears,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("embalm it");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Grizzly Bears"),
        "an embalm token was created"
    );
}

/// Samut's anthem and discount both read your speed.
#[test]
fn samut_scales_with_your_speed() {
    let mut g = main_phase();
    etb(&mut g, catalog::samut_the_driving_force());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bears).unwrap().power, 3, "start your engines! → speed 1");
    g.players[0].speed = 3;
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5, "+3/+0 at speed 3");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let card = g.find_card_anywhere(bolt).unwrap().clone();
    assert_eq!(cost_reduction_for_spell(&g, 0, &card, None), 3, "noncreature discount");
}

/// Cycling Valor's Flagship for X mints X crew-boosting Pilots.
#[test]
fn valors_flagship_cycles_into_pilots() {
    let mut g = main_phase();
    let ship = g.add_card_to_hand(0, catalog::valors_flagship());
    flood(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: ship, x_value: Some(2) }).expect("cycle");
    drain_stack(&mut g);
    let pilots: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Pilot").map(|c| c.id).collect();
    assert_eq!(pilots.len(), 2, "X = 2");
    assert_eq!(g.crew_saddle_power_bonus(pilots[0]), 2, "crews as a 3-power creature");
}
