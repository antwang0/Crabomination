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

/// Equip `sword` to a fresh attacker for seat 0 and swing at seat 1.
fn swing_with_sword(g: &mut GameState, sword: crabomination::card::CardDefinition) -> CardId {
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let s = g.add_card_to_battlefield(0, sword);
    g.battlefield_find_mut(s).unwrap().attached_to = Some(bear);
    g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    bear
}

/// CR 115.4 / 603.3d — Sword of Fire and Ice's "2 damage to any target" is
/// the controller's pick, not bound to the damaged player.
#[test]
fn cr_603_3d_a_combat_damage_any_target_is_not_bound_to_that_player() {
    let mut g = main_phase(2);
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    g.battlefield_find_mut(elf).unwrap().tapped = true;
    swing_with_sword(&mut g, catalog::sword_of_fire_and_ice());
    let life = g.players[1].life;
    let mut asked = 0;
    for _ in 0..60 {
        if let Some(p) = g.pending_decision.as_ref() {
            let answer = match &p.decision {
                Decision::ChooseTarget { legal, .. } if legal.contains(&Target::Permanent(elf)) => {
                    assert!(legal.contains(&Target::Player(1)), "a player is still legal");
                    asked += 1;
                    DecisionAnswer::Target(Target::Permanent(elf))
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
    assert!(g.battlefield_find(elf).is_none(), "the chosen creature took the 2");
    assert_eq!(g.players[1].life, life - 4, "only the bear's combat damage (2 + the Sword's +2)");
}

/// CR 115.1c — Niv-Mizzet, Guildpact's combat-damage trigger fills its second
/// slot ("target player draws X cards"): a bot seat's draw is not lost.
#[test]
fn cr_115_1c_a_combat_damage_triggers_second_slot_is_filled() {
    let mut g = main_phase(2);
    g.players[0].wants_ui = false;
    let niv = g.add_card_to_battlefield(0, catalog::niv_mizzet_guildpact());
    g.clear_sickness(niv);
    g.add_card_to_battlefield(0, catalog::judith_carnage_connoisseur());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let hands = g.players[0].hand.len() + g.players[1].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: niv, target: AttackTarget::Player(1) }]))
        .expect("attack");
    let life = g.players[0].life;
    for _ in 0..60 {
        if let Some(p) = g.pending_decision.as_ref() {
            let a = crabomination::decision::AutoDecider.decide(&p.decision);
            g.submit_decision(a).expect("answer");
            continue;
        }
        if matches!(g.step, TurnStep::EndCombat | TurnStep::PostCombatMain) && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].life, life + 1, "the trigger resolved (one pair → X = 1)");
    assert_eq!(g.players[0].hand.len() + g.players[1].hand.len(), hands + 1, "and someone drew X");
}

/// Answer every pending decision with "yes" / `pick` for a target / the
/// headless answer otherwise, passing priority until the stack is empty.
fn settle_yes(g: &mut GameState, pick: Target) -> usize {
    let mut asked = 0;
    for _ in 0..60 {
        if let Some(p) = g.pending_decision.as_ref() {
            let answer = match &p.decision {
                Decision::ChooseTarget { legal, .. } if legal.contains(&pick) => {
                    asked += 1;
                    DecisionAnswer::Target(pick.clone())
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
    asked
}

/// CR 603.7c — a reflexive trigger that resolves later still reads the event
/// that made it: Ziatora's "damage equal to that creature's power", asked of a
/// prompting seat (so on the stack), is the sacrificed Hill Giant's 3.
#[test]
fn cr_603_7c_a_stacked_reflexive_trigger_reads_the_sacrificed_power() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::ziatora_the_incinerator());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    assert_eq!(settle_yes(&mut g, Target::Player(1)), 1);
    assert_eq!(g.players[1].life, 17, "three damage, the sacrificed creature's power");
}

/// CR 603.7c + 608.2b — the stacked payoff's target filter reads the carried
/// discard too: Argentum Masticore's "mana value ≤ the discarded card's" keeps
/// a Sol Ring legal after a Serra Angel discard, even with a Bolt resolving
/// in between, so it is destroyed.
#[test]
fn cr_603_7c_a_stacked_reflexive_targets_filter_reads_the_discard() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::argentum_masticore());
    g.add_card_to_hand(0, catalog::serra_angel());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    // Resolve the upkeep trigger, stopping once its payoff is on the stack.
    for _ in 0..20 {
        if let Some(p) = g.pending_decision.as_ref() {
            let answer = match &p.decision {
                Decision::ChooseTarget { legal, .. } if legal.contains(&Target::Permanent(ring)) => {
                    DecisionAnswer::Target(Target::Permanent(ring))
                }
                Decision::OptionalTrigger { .. } => DecisionAnswer::Bool(true),
                d => crabomination::decision::AutoDecider.decide(d),
            };
            g.submit_decision(answer).expect("answer");
            continue;
        }
        if g.players[0].graveyard.iter().any(|c| c.definition.name == "Serra Angel") {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.stack.len(), 1, "the reflexive payoff waits on the stack");
    cast(&mut g, bolt, Some(Target::Player(1)));
    settle_yes(&mut g, Target::Permanent(ring));
    assert!(g.battlefield_find(ring).is_none(), "MV 1 <= the discarded MV 5");
}

/// CR 603.7 — a bot seat's "when you do" payoff is a trigger on the stack too
/// (it was resolved inline): Ziatora's damage waits there, a window to
/// respond, and still deals the sacrificed creature's power.
#[test]
fn cr_603_7_a_bot_seats_reflexive_payoff_uses_the_stack() {
    let mut g = main_phase(2);
    g.players[0].wants_ui = false;
    g.players[0].hostile_player_targets = true;
    g.add_card_to_battlefield(0, catalog::ziatora_the_incinerator());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    g.perform_action(GameAction::PassPriority).expect("pass");
    g.perform_action(GameAction::PassPriority).expect("pass");
    assert_eq!(g.players[1].life, 20, "the payoff has not resolved yet");
    assert_eq!(g.stack.len(), 1, "the reflexive trigger is on the stack");
    settle(&mut g, Target::Player(1));
    assert_eq!(g.players[1].life, 17);
}
