//! CR conformance for the RNA mechanics wave: CR 702.135 (Afterlife — the
//! minted Spirits are white AND black with flying), CR 702.134 (Mentor — the
//! counter lands only on a lesser-power *attacking* creature and does nothing
//! with no legal target), and CR 702.108 (Adapt — a creature that already has a
//! +1/+1 counter is not adapted again).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// CR 702.135a — Afterlife N mints N 1/1 white-and-black Spirits with flying.
#[test]
fn cr_702_135_afterlife_spirits_are_white_black_flying() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::ministrant_of_obligation()); // Afterlife 2
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(m), 2, None, &mut evs);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 2, "Afterlife 2 → two Spirits");
    for s in spirits {
        let cols = s.definition.printed_colors();
        assert!(cols.contains(&Color::White) && cols.contains(&Color::Black), "Spirit is white and black");
        assert!(s.definition.keywords.contains(&Keyword::Flying), "Spirit has flying");
    }
}

/// CR 702.134a — Mentor targets an *attacking* creature with lesser power; with
/// no such creature the trigger does nothing.
#[test]
fn cr_702_134_mentor_needs_lesser_power_attacker() {
    // A lone Mentor attacker (2/2) has no lesser-power ally → no counter placed.
    let mut g = two_player_game();
    let stalwart = g.add_card_to_battlefield(0, catalog::sunhome_stalwart());
    g.clear_sickness(stalwart);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: stalwart, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(stalwart).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "Mentor can't target itself and has no other legal target");

    // A non-attacking 1/1 ally is also not a legal target (must be attacking).
    let mut g = two_player_game();
    let stalwart = g.add_card_to_battlefield(0, catalog::sunhome_stalwart());
    let idle = crabomination::card::TokenDefinition {
        name: "Idle".into(), power: 1, toughness: 1,
        card_types: vec![crabomination::card::CardType::Creature], ..Default::default()
    };
    let idle = g.add_token_to_battlefield(0, &idle);
    g.clear_sickness(stalwart);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: stalwart, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(idle).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "a non-attacking ally is not a legal Mentor target");
}

/// CR 702.108b — Adapt N does nothing while the creature already has a +1/+1
/// counter (the "if it has no +1/+1 counters" gate).
#[test]
fn cr_702_108_adapt_skips_when_already_countered() {
    let mut g = two_player_game();
    let eel = g.add_card_to_battlefield(0, catalog::skitter_eel()); // {2}{U}: Adapt 2
    g.clear_sickness(eel);
    // First adapt: no counters → gains two.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: eel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("adapt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(eel).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    // Second adapt: already has counters → no change.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: eel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("adapt again");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(eel).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "adapt is a no-op with counters already present");
}
