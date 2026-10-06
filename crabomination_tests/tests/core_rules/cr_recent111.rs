//! CR 603.3d — a triggered ability's controller chooses its targets as it is
//! put on the stack. A prompting seat is asked for every trigger family the
//! engine used to push with its own pick: a non-cast ETB, an attack trigger,
//! a spell-cast listener and a "when you cast this spell" trigger.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{Decider, Decision, DecisionAnswer};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].wants_ui = true;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

/// Answer every pending decision: a trigger's target is `pick` (counted),
/// anything else the headless answer. Drains the stack in between.
fn settle(g: &mut GameState, pick: Target) -> usize {
    let mut asked = 0;
    for _ in 0..40 {
        if let Some(p) = g.pending_decision.as_ref() {
            let answer = match &p.decision {
                Decision::ChooseTarget { legal, .. } if legal.contains(&pick) && asked == 0 => {
                    asked += 1;
                    DecisionAnswer::Target(pick.clone())
                }
                d => crabomination::decision::AutoDecider.decide(d),
            };
            g.submit_decision(answer).expect("answer");
            continue;
        }
        if g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    asked
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
}

/// A reanimated Flametongue Kavu's ETB: the prompting seat picks the creature.
#[test]
fn cr_603_3d_a_reanimated_etb_asks_for_its_target() {
    let mut g = main_phase(2);
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let kavu = g.add_card_to_graveyard(0, catalog::flametongue_kavu());
    let zombify = g.add_card_to_hand(0, catalog::zombify());
    cast(&mut g, zombify, Some(Target::Permanent(kavu)));
    assert_eq!(settle(&mut g, Target::Permanent(small)), 1, "the ETB's target was asked");
    assert!(g.battlefield_find(small).is_none());
    assert!(g.battlefield_find(big).is_some());
}

/// Goblin Heelcutter's attack trigger: the prompting attacker picks.
#[test]
fn cr_603_3d_an_attack_trigger_asks_for_its_target() {
    let mut g = main_phase(2);
    let heel = g.add_card_to_battlefield(0, catalog::goblin_heelcutter());
    g.clear_sickness(heel);
    let a = g.add_card_to_battlefield(1, catalog::serra_angel());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: heel, target: AttackTarget::Player(1) }]))
        .expect("attack");
    assert_eq!(settle(&mut g, Target::Permanent(b)), 1);
    let cant = |id| g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&crabomination::card::Keyword::CantBlock));
    assert!(cant(b) && !cant(a));
}

/// Niblis of Frost's spell-cast trigger: the prompting caster picks.
#[test]
fn cr_603_3d_a_cast_listener_asks_for_its_target() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::niblis_of_frost());
    let a = g.add_card_to_battlefield(1, catalog::serra_angel());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt, None);
    assert_eq!(settle(&mut g, Target::Permanent(b)), 1);
    assert!(g.battlefield_find(b).unwrap().tapped);
    assert!(!g.battlefield_find(a).unwrap().tapped);
}

/// Ulamog's "when you cast this spell, exile two target permanents": the
/// prompting caster picks.
#[test]
fn cr_603_3d_a_self_cast_trigger_asks_for_its_targets() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let gone = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ulamog = g.add_card_to_hand(0, catalog::ulamog_the_ceaseless_hunger());
    cast(&mut g, ulamog, None);
    assert_eq!(settle(&mut g, Target::Permanent(gone)), 1);
    assert!(g.battlefield_find(gone).is_none());
}
