//! CR 603.6a / 608.2d — "choose a card with mana value 1, then do the same
//! for 2 and 3; return those cards" (`Effect::ChoosePerManaValue`): every
//! pick is made first and the cards enter together, so their enter triggers
//! see each other.

use crabomination::card::{CardDefinition, CardId};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::GameAction;
use crabomination::game::*;
use crabomination::mana::{cost, generic};
use crabomination::TurnStep;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

/// A three-mana Soul Warden: it hears the other two only if they enter with it.
fn big_warden() -> CardDefinition {
    CardDefinition { name: "Big Warden", cost: cost(&[generic(3)]), ..catalog::soul_warden() }
}

fn activate_kayla(g: &mut GameState, kayla: CardId) {
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kayla, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Queen Kayla returns her mana value 1, 2 and 3 discards together: the
/// three-drop warden gains life for the other two (one after another, it
/// entered last and heard nothing).
#[test]
fn cr_603_6a_queen_kayla_returns_her_picks_together() {
    let mut g = main_phase();
    let kayla = g.add_card_to_battlefield(0, catalog::queen_kayla_bin_kroog());
    g.clear_sickness(kayla);
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let warden = g.add_card_to_hand(0, big_warden());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let life = g.players[0].life;
    activate_kayla(&mut g, kayla);
    for id in [vanguard, bears, warden] {
        assert!(g.battlefield_find(id).is_some(), "each mana value's pick returns");
    }
    assert_eq!(g.players[0].life, life + 2, "the warden saw the other two enter with it");
}

/// "You may choose": a passed mana value stays in the graveyard.
#[test]
fn cr_608_2d_queen_kayla_may_pass_a_mana_value() {
    let mut g = main_phase();
    let kayla = g.add_card_to_battlefield(0, catalog::queen_kayla_bin_kroog());
    g.clear_sickness(kayla);
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![]), DecisionAnswer::Cards(vec![bears])]));
    activate_kayla(&mut g, kayla);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == vanguard), "the passed one-drop stays");
    assert!(g.battlefield_find(bears).is_some());
}

/// Gruesome Menagerie's picks are mandatory and one per value; a second
/// one-drop stays behind.
#[test]
fn cr_608_2d_gruesome_menagerie_takes_one_of_each_value() {
    let mut g = main_phase();
    let a = g.add_card_to_graveyard(0, catalog::elite_vanguard());
    let b = g.add_card_to_graveyard(0, catalog::elite_vanguard());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::centaur_courser());
    let spell = g.add_card_to_hand(0, catalog::gruesome_menagerie());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some() && g.battlefield_find(giant).is_some());
    let ones = [a, b].iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    assert_eq!(ones, 1, "one creature of mana value 1");
}

/// CR 707.10c — a copy is controlled by the player who copied it, so "whenever
/// an opponent copies a spell, they lose 1 life" (Mage Hunter) hits the
/// copier, not the original spell's caster (the copy event's subject was the
/// original, and `Triggerer` read its caster).
#[test]
fn cr_707_10c_a_copy_trigger_reads_the_copier() {
    use crabomination::game::types::Target;
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mage_hunter());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let rev = g.add_card_to_hand(1, catalog::reverberate());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: rev, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("reverberate");
    drain_stack(&mut g);
    // The copy may be re-aimed at seat 0's face; the Hunter costs it nothing.
    assert_eq!((20 - g.players[0].life) % 3, 0, "the original's caster copied nothing");
    assert_eq!(g.players[1].life, 18, "one for the cast, one for the copy");
}

/// CR 800.4a — a demonstrate ask answered after the spell ceased to exist (its
/// controller left the game while the ask waited, 8-seat sweep seed 2946692)
/// copies nothing and spends the answer (a strict answer log aborted on it).
#[test]
fn cr_800_4a_a_parked_demonstrate_ask_whose_spell_is_gone_copies_nothing() {
    use crabomination::decision::Decision;
    use crabomination::game::types::{StackItem, Target};
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].wants_ui = true;
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::healing_technique());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    let mut asked = false;
    for _ in 0..8 {
        match g.pending_decision.as_ref().map(|d| &d.decision) {
            Some(Decision::OptionalTrigger { .. }) => {
                asked = true;
                break;
            }
            Some(_) => break,
            None => {
                let _ = g.resolve_top_of_stack();
            }
        }
    }
    assert!(asked, "the caster is asked whether to demonstrate");
    g.stack.retain(|si| !matches!(si, StackItem::Spell { card, .. } if card.id == spell));
    let _ = g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(true)));
    assert!(g.stack.iter().all(|si| !matches!(si, StackItem::Spell { .. })), "no copy of a spell that is gone");
}

/// CR 614.6 — a card discarded under Rest in Peace goes to exile, and it is
/// still its owner's discard: Megrim's "deals 2 damage to that player" found no
/// player (`Triggerer` looked for the card in hand and graveyard only).
#[test]
fn cr_614_6_a_discard_replaced_into_exile_still_names_the_discarder() {
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::megrim());
    g.add_card_to_battlefield(0, catalog::rest_in_peace());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    let discard = Effect::Discard { who: Selector::Player(PlayerRef::Seat(1)), amount: Value::ONE, random: false };
    let evs = g.resolve_effect(&discard, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bears), "Rest in Peace exiled it");
    assert_eq!(g.players[1].life, 18, "Megrim hit the discarder");
}

/// CR 603.10a / 111.7 — a dying token has no zone to be found in afterwards:
/// Massacre Wurm's "that player loses 2 life" reads its last controller, so a
/// Wurm-killed token still costs its controller 2 (`Triggerer` read nobody).
#[test]
fn cr_603_10a_a_dying_token_names_its_last_controller() {
    use crabomination::effect::{Effect, Selector};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::massacre_wurm());
    let token = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(token).unwrap().is_token = true;
    let kill = Effect::Destroy { what: Selector::ExactObjects(vec![token]) };
    let evs = g.resolve_effect(&kill, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "the token's controller loses 2");
}
