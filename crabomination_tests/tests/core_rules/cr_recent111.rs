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

/// Throat Slitter's combat-damage trigger: "target nonblack creature that
/// player controls" is the prompting attacker's pick, not the engine's.
#[test]
fn cr_603_3d_a_combat_damage_trigger_asks_for_its_target() {
    let mut g = main_phase(2);
    let slitter = g.add_card_to_battlefield(0, catalog::throat_slitter());
    g.clear_sickness(slitter);
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(big).unwrap().tapped = true;
    g.battlefield_find_mut(small).unwrap().tapped = true;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: slitter, target: AttackTarget::Player(1) }]))
        .expect("attack");
    let mut asked = 0;
    for _ in 0..60 {
        if let Some(p) = g.pending_decision.as_ref() {
            let answer = match &p.decision {
                Decision::ChooseTarget { legal, .. } if legal.contains(&Target::Permanent(small)) => {
                    asked += 1;
                    DecisionAnswer::Target(Target::Permanent(small))
                }
                d => crabomination::decision::AutoDecider.decide(d),
            };
            g.submit_decision(answer).expect("answer");
            continue;
        }
        if matches!(g.step, TurnStep::EndCombat | TurnStep::PostCombatMain) && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(asked, 1);
    assert!(g.battlefield_find(small).is_none(), "the chosen creature is destroyed");
    assert!(g.battlefield_find(big).is_some());
}

/// A defender's "whenever a creature attacks you, tap target creature" (a
/// slot that can't be a player, so not bound to the attacker's controller):
/// the prompting defender picks.
#[test]
fn cr_603_3d_a_defenders_attack_trigger_asks_for_its_target() {
    use crabomination::card::{CardDefinition, CardType, EventKind, EventScope, EventSpec, SelectionRequirement as R, TriggeredAbility};
    use crabomination::effect::{Effect, Selector};
    let mut g = main_phase(2);
    g.players[0].wants_ui = false;
    g.players[1].wants_ui = true;
    let watch = CardDefinition {
        name: "Test Sentry",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::Tap { what: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
        }],
        ..Default::default()
    };
    g.add_card_to_battlefield(1, watch);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let other = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    assert_eq!(settle(&mut g, Target::Permanent(other)), 1);
    assert!(g.battlefield_find(other).unwrap().tapped);
}

/// "Whenever a creature deals combat damage to you, tap target creature": the
/// prompting damaged player picks.
#[test]
fn cr_603_3d_a_damaged_players_trigger_asks_for_its_target() {
    use crabomination::card::{CardDefinition, CardType, EventKind, EventScope, EventSpec, SelectionRequirement as R, TriggeredAbility};
    use crabomination::effect::{Effect, Selector};
    let mut g = main_phase(2);
    g.players[0].wants_ui = false;
    g.players[1].wants_ui = true;
    let watch = CardDefinition {
        name: "Test Grudge",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::SelfSource),
            effect: Effect::Tap { what: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
        }],
        ..Default::default()
    };
    g.add_card_to_battlefield(1, watch);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let other = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    let mut asked = 0;
    for _ in 0..60 {
        if let Some(p) = g.pending_decision.as_ref() {
            let answer = match &p.decision {
                Decision::ChooseTarget { legal, .. } if legal.contains(&Target::Permanent(other)) => {
                    asked += 1;
                    DecisionAnswer::Target(Target::Permanent(other))
                }
                d => crabomination::decision::AutoDecider.decide(d),
            };
            g.submit_decision(answer).expect("answer");
            continue;
        }
        if matches!(g.step, TurnStep::EndCombat | TurnStep::PostCombatMain) && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(asked, 1);
    assert!(g.battlefield_find(other).unwrap().tapped);
}

/// Ride the Avalanche's delayed "when you next cast a spell this turn" names
/// its "up to one target creature" through the prompting seat.
#[test]
fn cr_603_3d_a_delayed_cast_trigger_asks_for_its_target() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::serra_angel());
    let ride = g.add_card_to_hand(0, catalog::ride_the_avalanche());
    cast(&mut g, ride, None);
    settle(&mut g, Target::Player(1));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(settle(&mut g, Target::Permanent(a)), 1);
    let plus = |id| g.battlefield_find(id).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne);
    assert_eq!((plus(a), plus(b)), (1, 0), "Bolt's mana value onto the chosen creature");
}

/// CR 603.7 — a "when you do" payoff is a reflexive trigger: Caesar's
/// "damage … to target opponent" goes on the stack for a prompting seat, and
/// that seat names the opponent.
#[test]
fn cr_603_7_a_reflexive_payoff_asks_for_its_target() {
    let mut g = main_phase(3);
    let caesar = g.add_card_to_battlefield(0, catalog::caesar_legions_emperor());
    g.clear_sickness(caesar);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: caesar, target: AttackTarget::Player(1) }]))
        .expect("attack");
    let mut asked = 0;
    for _ in 0..40 {
        if let Some(p) = g.pending_decision.as_ref() {
            let answer = match &p.decision {
                Decision::ChooseModes { .. } => DecisionAnswer::Modes(vec![0, 2]),
                Decision::ChooseTarget { legal, .. } if legal.contains(&Target::Player(2)) => {
                    asked += 1;
                    DecisionAnswer::Target(Target::Player(2))
                }
                Decision::OptionalTrigger { .. } => DecisionAnswer::Bool(true),
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
    assert_eq!(asked, 1);
    assert!(g.players[2].life < l2, "the chosen opponent takes the damage");
    assert_eq!(g.players[1].life, l1);
}

/// CR 702.85a / 601.2c — a cascaded spell's target is its caster's to name: a
/// prompting seat's Bloodbraid Elf cascades into Lightning Bolt and aims it.
#[test]
fn cr_702_85a_a_cascaded_spell_asks_for_its_target() {
    let mut g = main_phase(3);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let elf = g.add_card_to_hand(0, catalog::bloodbraid_elf());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    cast(&mut g, elf, None);
    let mut asked = 0;
    for _ in 0..40 {
        if let Some(p) = g.pending_decision.as_ref() {
            let answer = match &p.decision {
                Decision::ChooseTarget { legal, .. } if legal.contains(&Target::Player(2)) => {
                    asked += 1;
                    DecisionAnswer::Target(Target::Player(2))
                }
                Decision::OptionalTrigger { .. } => DecisionAnswer::Bool(true),
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
    assert_eq!(asked, 1);
    assert_eq!(g.players[2].life, l2 - 3, "the chosen opponent takes the Bolt");
    assert_eq!(g.players[1].life, l1);
}
