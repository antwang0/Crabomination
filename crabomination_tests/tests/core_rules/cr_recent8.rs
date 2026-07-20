//! CR conformance for rules exercised by the recent Dissension gap batch:
//! CR 122.1c (a shield counter replaces destruction — vs Kill-Suit Cultist's
//! damage→destroy shield), CR 702.15 (noncombat lifelink), and CR 614.6 (a
//! life-gain-becomes-loss replacement — Rain of Gore).

use crabomination::catalog;
use crabomination::card::{CounterType, Keyword};
use crabomination::game::effects::EntityRef;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

/// CR 122.1c — a Shield counter replaces destruction: Kill-Suit Cultist's
/// "destroy that creature instead" shield removes the shield counter rather
/// than destroying the creature, and the damage is still prevented.
#[test]
fn cr_122_1c_shield_counter_survives_kill_suit_destroy() {
    let mut g = two_player_game();
    let cultist = g.add_card_to_battlefield(0, catalog::kill_suit_cultist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::Shield, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cultist,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("activate Kill-Suit Cultist");
    drain_stack(&mut g);
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(bear), 1, None, &mut events);
    let _ = g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear survived — shield replaced the destroy");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Shield), 0, "shield spent");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "damage was still prevented");
}

/// CR 702.15 — lifelink on a noncombat damage source: the source's controller
/// gains life equal to the damage dealt.
#[test]
fn cr_702_15_noncombat_lifelink_gains_life() {
    let mut g = two_player_game();
    // Vampire Nighthawk has lifelink; simulate a noncombat damage event from it.
    let hawk = g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    assert!(g.computed_permanent(hawk).unwrap().keywords.contains(&Keyword::Lifelink));
    let life0 = g.players[0].life;
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(1), 2, Some(hawk), &mut events);
    assert_eq!(g.players[0].life, life0 + 2, "controller gained 2 from lifelink");
}

/// CR 614.6 — Rain of Gore replaces any life gain with an equal life loss for
/// every player; a would-be gain of 3 becomes a loss of 3 (and a 0-gain is a
/// no-op, no phantom loss).
#[test]
fn cr_614_6_rain_of_gore_flips_gain_to_loss() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rain_of_gore());
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    assert_eq!(g.adjust_life(0, 3), l0 - 3, "controller's gain became a loss");
    assert_eq!(g.adjust_life(1, 5), l1 - 5, "opponent's gain became a loss too (each player)");
    // A 0 gain does nothing.
    let l0b = g.players[0].life;
    assert_eq!(g.adjust_life(0, 0), l0b, "0-gain is a no-op");
}
