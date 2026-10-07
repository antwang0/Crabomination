//! CR 400.7 / 607 — a card that leaves exile is a new object: the exiler's
//! "until this leaves" link (`exiled_by`) does not follow it into its next
//! zone, nor into a later exile. Plus a CR 608.2 answer-log leak.

use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn resolve(g: &mut GameState, source: crabomination::card::CardId, target: Option<Target>, e: &Effect) {
    let events = g.resolve_effect(e, &EffectContext::for_ability(source, 0, target)).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Foreboding Steamboat exiles a Bear, its attack trigger puts the Bear into
/// the graveyard, and a later exile of that card is a plain exile: the
/// Steamboat leaving doesn't bring it back. (Two 6-seat debug pods, seed
/// 130026 games 10 and 27, found the card still "exiled until the Steamboat
/// leaves" with the Steamboat long gone.)
#[test]
fn cr_400_7_a_card_out_of_exile_loses_its_exilers_link() {
    let mut g = main_phase();
    let boat = g.add_card_to_battlefield(0, catalog::foreboding_steamboat());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::foreboding_steamboat().triggered_abilities[0].effect.clone();
    resolve(&mut g, boat, None, &etb);
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_by.is_some()));
    let to_gy = Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Graveyard };
    resolve(&mut g, boat, None, &to_gy);
    let in_gy = g.players[1].graveyard.iter().find(|c| c.id == bear).expect("in the graveyard");
    assert!(in_gy.exiled_by.is_none() && in_gy.exiled_with.is_none());
    resolve(&mut g, boat, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, boat, None, &Effect::Destroy { what: Selector::ExactObjects(vec![boat]) });
    assert!(g.exile.iter().any(|c| c.id == bear), "a plain exile: the Steamboat leaving doesn't return it");
}

/// Hostage Taker exiles a Bear; the Bear comes back by another route, is
/// exiled again by something else, and Hostage Taker leaving must not return
/// it (CR 610.3c reads only the object it exiled).
#[test]
fn cr_400_7_a_reexiled_card_does_not_return_with_its_old_exiler() {
    let mut g = main_phase();
    let taker = g.add_card_to_battlefield(0, catalog::hostage_taker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::hostage_taker().triggered_abilities[0].effect.clone();
    resolve(&mut g, taker, Some(Target::Permanent(bear)), &etb);
    assert!(g.exile.iter().any(|c| c.id == bear), "the Taker holds it");
    resolve(
        &mut g,
        taker,
        None,
        &Effect::Move {
            what: Selector::ExactObjects(vec![bear]),
            to: ZoneDest::Battlefield { controller: PlayerRef::Seat(1), tapped: false },
        },
    );
    assert!(g.battlefield_find(bear).is_some_and(|c| c.exiled_by.is_none()));
    resolve(&mut g, taker, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, taker, None, &Effect::Destroy { what: Selector::ExactObjects(vec![taker]) });
    assert!(g.battlefield_find(bear).is_none(), "the Taker's leave returns only what it holds");
}

/// CR 400.7 / 607 — a card cast from exile leaves the link behind with the
/// exile: the Bear a Steamboat held resolves as a new permanent, and when a
/// later effect exiles it, it is not "exiled with" the Steamboat again.
#[test]
fn cr_400_7_a_card_cast_from_exile_is_not_exiled_with_its_old_exiler() {
    use crabomination::game::types::StackItem;
    let mut g = main_phase();
    let boat = g.add_card_to_battlefield(0, catalog::foreboding_steamboat());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let mut card = g.players[0].hand.pop().unwrap();
    assert_eq!(card.id, bear);
    card.exiled_with = Some(boat);
    card.cast_from_exile = true;
    g.stack.push(StackItem::Spell {
        card: Box::new(card),
        caster: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: 0,
        converged_value: 0,
        mana_spent: 2,
        uncounterable: false,
    });
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the Bear resolved");
    g.remove_from_battlefield_to_exile(bear);
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("exiled");
    assert_eq!(exiled.exiled_with, None, "a plain exile, not the Steamboat's");
}

/// CR 608.2 — an asked pick whose candidates all left while it waited spends
/// its answer. Yuffie took Lightning Greaves "for as long as you control
/// Yuffie" and asked which Equipment to attach; the steal ended before the
/// answer came back (Clever Concealment phased Yuffie out), the re-run found
/// no Equipment, and the stashed pick leaked (a strict 6-seat debug pod, seed
/// 201019 game 18). Nextest runs each test in its own process, so the strict
/// leak check (read once) is switched on here.
#[test]
fn cr_608_2_a_pick_whose_candidates_left_spends_its_answer() {
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::GameAction;
    // SAFETY: set before any thread reads the environment.
    unsafe { std::env::set_var("CRAB_ANSWER_LOG", "strict") };
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let greaves = g.add_card_to_battlefield(1, catalog::lightning_greaves());
    let yuffie = g.add_card_to_hand(0, catalog::yuffie_materia_hunter());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: yuffie, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Yuffie");
    let mut asked = false;
    for _ in 0..20 {
        if let Some(d) = g.pending_decision.as_ref() {
            let answer = match &d.decision {
                Decision::ChooseTarget { .. } => DecisionAnswer::Target(Target::Permanent(greaves)),
                Decision::OptionalTrigger { .. } => DecisionAnswer::Bool(true),
                Decision::ChooseCards { .. } => {
                    // The steal ends before the answer comes back.
                    g.battlefield_find_mut(greaves).unwrap().controller = 1;
                    asked = true;
                    DecisionAnswer::Cards(vec![greaves])
                }
                other => panic!("unexpected ask {other:?}"),
            };
            g.submit_decision(answer).expect("answer");
            continue;
        }
        if g.stack.is_empty() {
            break;
        }
        g.resolve_top_of_stack().expect("resolve");
    }
    assert!(asked, "the Equipment pick was asked");
    assert!(g.pending_decision.is_none());
    assert_eq!(g.battlefield_find(greaves).and_then(|c| c.attached_to), None, "nothing to attach");
}
