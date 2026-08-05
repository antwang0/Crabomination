//! CR conformance for this run:
//! - CR 611.2c — "for as long as that Aura is attached to it" ends the moment
//!   the Aura moves off, not only when it leaves the battlefield.
//! - CR 702.51b — convoke helpers pay coloured pips of their own colour.
//! - CR 706.2 — the roll-extra-and-ignore-lowest replacement covers stored
//!   die results, not just result-table rolls.
//! - CR 603 / 400.7 — "the Nth time this ability has resolved this turn" is
//!   keyed per object, so a replacement permanent starts over.

use crabomination::card::CardType;
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood_mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// CR 611.2c — moving the Aura to another permanent ends the steal it was
/// holding, even though the Aura never left the battlefield.
#[test]
fn cr_611_2c_steal_ends_when_the_aura_moves_hosts() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::eriette_the_beguiler());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("aura");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0);

    g.battlefield_find_mut(aura).unwrap().attached_to = Some(other);
    g.check_state_based_actions();
    assert_eq!(
        g.battlefield_find(victim).unwrap().controller,
        1,
        "the clause ended with the attachment"
    );
}

/// CR 702.51b — a creature tapped for convoke may pay one mana of its own
/// colour, so an all-coloured activation cost is convokable.
#[test]
fn cr_702_51b_convoke_helpers_pay_colored_pips() {
    use crabomination::card::{ActivatedAbility, CardDefinition};
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::mana::{cost, g as green};
    let probe = CardDefinition {
        name: "Convoke Probe",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[green(), green()]),
            convoke: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, probe);
    let helpers: Vec<_> = (0..2)
        .map(|_| g.add_card_to_battlefield(0, catalog::llanowar_elves()))
        .collect();
    for c in &helpers {
        g.battlefield_find_mut(*c).unwrap().summoning_sick = false;
    }
    g.add_card_to_library(0, catalog::island());
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: src,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        helpers: helpers.clone(),
    })
    .expect("two green creatures pay {G}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// CR 706.2 — Pixie Guide's replacement applies to a stored-results roll too:
/// six dice are rolled for Centaur of Attention's five, and the lowest is
/// ignored.
#[test]
fn cr_706_2_ignore_lowest_covers_stored_die_results() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::pixie_guide());
    let centaur = g.add_card_to_hand(0, catalog::centaur_of_attention());
    flood_mana(&mut g, 0);
    // Six scripted faces; the 1 is the ignored low roll.
    g.decider = Box::new(ScriptedDecider::new(
        [1u8, 6, 6, 5, 4, 3].map(DecisionAnswer::DieRoll),
    ));
    g.perform_action(GameAction::CastSpell {
        card_id: centaur,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let stored = g.battlefield_find(centaur).unwrap().stored_die_results.clone();
    assert_eq!(stored.len(), 5, "five results are stored");
    assert!(!stored.contains(&1), "the lowest roll was ignored");
}

/// CR 603 / 400.7 — a replacement Victor is a new object, so its escalating
/// ability starts over at the first branch.
#[test]
fn cr_603_nth_resolution_tally_restarts_on_a_new_object() {
    let mut g = main_phase();
    let first = g.add_card_to_battlefield(0, catalog::victor_valgavoths_seneschal());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    g.players[1].hand.clear();
    g.add_card_to_hand(1, catalog::island());
    flood_mana(&mut g, 0);
    let cast_enchantment = |g: &mut GameState| {
        let e = g.add_card_to_hand(0, catalog::goblin_bombardment());
        g.perform_action(GameAction::CastSpell {
            card_id: e,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("enchantment");
        drain_stack(g);
    };
    cast_enchantment(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "first branch surveils");

    let mut evs = Vec::new();
    g.destroy_permanent(first, false, &mut evs);
    g.check_state_based_actions();
    g.add_card_to_battlefield(0, catalog::victor_valgavoths_seneschal());
    cast_enchantment(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "the new object is back on branch one");
}
