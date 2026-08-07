//! CR conformance for this run:
//! - CR 702.24 — cumulative upkeep: the age counter goes on first, the cost
//!   scales with it, and multiple instances each trigger off the shared count.
//! - CR 702.26 — phasing: a phased-out permanent doesn't exist, its
//!   attachments phase out with it, and its counters survive.
//! - CR 602.5 — an "activate only once each turn" restriction rides the
//!   object, not its controller.

use crabomination::card::{CardDefinition, CardId, CounterType, CumulativeUpkeepCost, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn to_upkeep(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = seat;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

/// CR 702.24a — the age counter is added before the payment, so the very
/// first upkeep already costs one instance of the cumulative upkeep.
#[test]
fn cr_702_24a_age_counter_precedes_the_payment() {
    let mut g = two_player_game();
    let efreet = ready(&mut g, 0, catalog::uktabi_efreet()); // cumulative upkeep {G}
    let forest = ready(&mut g, 0, catalog::forest());
    to_upkeep(&mut g, 0);
    assert_eq!(g.battlefield_find(efreet).unwrap().counter_count(CounterType::Age), 1);
    assert!(g.battlefield_find(forest).unwrap().tapped, "one Forest paid the first tick");
}

/// CR 702.24a — partial payments aren't allowed: short of the full cost, the
/// permanent is sacrificed and nothing is paid.
#[test]
fn cr_702_24a_partial_payment_is_not_allowed() {
    let mut g = two_player_game();
    let wolves = ready(&mut g, 0, catalog::arctic_wolves()); // cumulative upkeep {2}
    g.battlefield_find_mut(wolves).unwrap().add_counters(CounterType::Age, 2);
    let forests: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::forest())).collect();
    to_upkeep(&mut g, 0);
    assert!(g.battlefield_find(wolves).is_none(), "two mana per counter, three counters");
    assert!(
        forests.iter().all(|&f| !g.battlefield_find(f).unwrap().tapped),
        "no partial payment"
    );
}

/// CR 702.24b — two instances trigger separately, and each reads the whole
/// age-counter pool.
#[test]
fn cr_702_24b_multiple_instances_share_the_age_counters() {
    let mut g = two_player_game();
    let mut def = catalog::uktabi_efreet();
    def.keywords.push(Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Life(1)));
    let efreet = ready(&mut g, 0, def);
    ready(&mut g, 0, catalog::forest());
    to_upkeep(&mut g, 0);
    // One age counter per instance, so the life half pays 2 (both counters).
    assert_eq!(g.battlefield_find(efreet).unwrap().counter_count(CounterType::Age), 2);
    assert_eq!(g.players[0].life, 18);
}

/// CR 702.26b — a phased-out permanent is treated as though it doesn't exist,
/// so nothing can target it.
#[test]
fn cr_702_26b_phased_out_permanent_cant_be_targeted() {
    let mut g = two_player_game();
    let familiar = ready(&mut g, 1, catalog::ertais_familiar());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.active_player_idx = 1;
    g.do_phasing();
    drain_stack(&mut g);
    assert!(g.phased_out.iter().any(|c| c.id == familiar));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(familiar)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
}

/// CR 702.26g — Auras attached to a phasing permanent phase out with it, and
/// CR 702.26d — counters ride along.
#[test]
fn cr_702_26g_attachments_phase_out_with_their_host() {
    let mut g = two_player_game();
    let familiar = ready(&mut g, 0, catalog::ertais_familiar());
    g.battlefield_find_mut(familiar).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let chains = g.add_card_to_battlefield(0, catalog::mana_chains());
    g.battlefield_find_mut(chains).unwrap().attached_to = Some(familiar);
    g.do_phasing();
    drain_stack(&mut g);
    assert!(g.phased_out.iter().any(|c| c.id == chains), "the Aura went too");
    let out = g.phased_out.iter().find(|c| c.id == familiar).expect("phased out");
    assert_eq!(out.counter_count(CounterType::PlusOnePlusOne), 2, "counters survive");
}

/// CR 702.26a — the permanent phases back in during its controller's next
/// untap step, and the Aura comes back with it.
#[test]
fn cr_702_26a_phases_back_in_at_the_next_untap_step() {
    let mut g = two_player_game();
    let familiar = ready(&mut g, 0, catalog::ertais_familiar());
    let chains = g.add_card_to_battlefield(0, catalog::mana_chains());
    g.battlefield_find_mut(chains).unwrap().attached_to = Some(familiar);
    g.do_phasing();
    drain_stack(&mut g);
    assert!(g.battlefield_find(familiar).is_none());
    g.do_phasing();
    drain_stack(&mut g);
    assert!(g.battlefield_find(familiar).is_some(), "phased back in");
    assert!(g.battlefield_find(chains).is_some());
}

/// CR 602.5b — a once-each-turn restriction rides the object, so a control
/// change doesn't refresh it.
#[test]
fn cr_602_5b_once_per_turn_survives_a_control_change() {
    let mut g = two_player_game();
    let avizoa = ready(&mut g, 0, catalog::avizoa());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: avizoa,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("first activation");
    drain_stack(&mut g);
    g.battlefield_find_mut(avizoa).unwrap().controller = 1;
    g.clear_sickness(avizoa);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: avizoa,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the restriction stayed with the permanent"
    );
}
