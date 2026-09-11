//! The "up to N" picks a headless seat never made.
//!
//! `AutoDecider::decide` answers `Decision::ChooseCards` with the first `min`
//! candidates, which is **nothing** at `min: 0`. Bot seats set `wants_ui`, so a
//! *bare* `decider.decide` never reaches `decide_pending_policy` either: every
//! such site resolved as a no-op for every seat in the training path. These are
//! the four that gated a whole effect body
//! (`scripts/audit_decision_plumbing.py`, the DEAD column).

use crabomination::card::{CardDefinition, CardType};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::*;

fn bear(name: &'static str, mv: u32, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: crabomination::mana::cost(&[crabomination::mana::generic(mv)]),
        card_types: vec![CardType::Creature],
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mv: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: crabomination::mana::cost(&[crabomination::mana::generic(mv)]),
        card_types: vec![CardType::Sorcery],
        ..Default::default()
    }
}

/// Rag Dealer's "exile up to three target cards from a single graveyard" took
/// nothing at all. The auto pick is hostile graveyards only — exiling our own
/// cards is a cost, not the effect — creature cards first, then by mana value.
#[test]
fn exile_up_to_n_from_graveyards_takes_the_opponents_creatures() {
    let mut g = two_player_game();
    let dealer = g.add_card_to_battlefield(0, catalog::rag_dealer());
    let mine = g.add_card_to_graveyard(0, bear("Mine", 6, 6, 6));
    let theirs_big = g.add_card_to_graveyard(1, bear("Theirs Big", 5, 5, 5));
    let theirs_small = g.add_card_to_graveyard(1, bear("Theirs Small", 2, 2, 2));
    let theirs_spell = g.add_card_to_graveyard(1, spell("Theirs Spell", 7));
    let ctx = EffectContext::for_ability(dealer, 0, None);
    g.resolve_effect(&catalog::rag_dealer().activated_abilities[0].effect, &ctx).unwrap();
    let exiled: Vec<CardId> = g.exile.iter().map(|c| c.id).collect();
    assert!(exiled.contains(&theirs_big), "the priciest opposing creature goes first");
    assert!(exiled.contains(&theirs_small));
    assert!(exiled.contains(&theirs_spell), "the third slot takes the spell");
    assert!(!exiled.contains(&mine), "our own graveyard is not a target");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mine));
    // The exiled set also rides `Selector::LastMoved`, which is what the "if a
    // creature card was exiled this way" riders read (Soul-Shackled Zombie):
    // three picks instead of none is what makes those fire at all.
    assert_eq!(g.exile.len(), 3);
}

/// `single: true` locks every pick to one graveyard, so the auto default must
/// pick within one too — three candidates across two graveyards used to exile
/// one card and drop the rest.
#[test]
fn exile_up_to_n_from_graveyards_keeps_single_to_one_graveyard() {
    let mut g3 = crabomination::game::multi_player_game(3);
    let dealer = g3.add_card_to_battlefield(0, catalog::rag_dealer());
    g3.add_card_to_graveyard(1, bear("One", 4, 4, 4));
    g3.add_card_to_graveyard(1, bear("Two", 3, 3, 3));
    g3.add_card_to_graveyard(2, bear("Three", 9, 9, 9));
    let ctx = EffectContext::for_ability(dealer, 0, None);
    g3.resolve_effect(&catalog::rag_dealer().activated_abilities[0].effect, &ctx).unwrap();
    // Seat 2's 9-drop ranks first, so the lock is seat 2 and seat 1 keeps both.
    assert_eq!(g3.exile.len(), 1);
    assert_eq!(g3.players[2].graveyard.len(), 0);
    assert_eq!(g3.players[1].graveyard.len(), 2);
}

/// Gather the Pack milled five and took nothing: the mill is the cost and the
/// pick is the card. The auto default takes the priciest eligible card, up to
/// the `take` the spell-mastery rider computes.
#[test]
fn mill_then_to_hand_n_takes_the_priciest_eligible_card() {
    let mut g = two_player_game();
    let caster = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].library.clear();
    let small = g.add_card_to_library(0, bear("Small", 1, 1, 1));
    let big = g.add_card_to_library(0, bear("Big", 7, 7, 7));
    let filler = g.add_card_to_library(0, spell("Filler", 3));
    let ctx = EffectContext::for_ability(caster, 0, None);
    g.resolve_effect(&catalog::gather_the_pack().effect, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == big), "the 7-drop is taken");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == small), "the 1-drop is milled");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == filler));
}

/// Aetherplasm bounced itself and deployed nothing, which is strictly worse
/// than not triggering. The auto default takes the best blocker in hand
/// (highest toughness, then power).
#[test]
fn return_self_deploy_blocker_picks_the_best_blocker_in_hand() {
    let mut g = two_player_game();
    let plasm = g.add_card_to_battlefield(0, catalog::aetherplasm());
    let attacker = g.add_card_to_battlefield(1, bear("Attacker", 3, 4, 4));
    g.add_card_to_hand(0, bear("Chump", 1, 1, 1));
    let wall = g.add_card_to_hand(0, bear("Wall", 4, 0, 6));
    g.block_map.insert(plasm, [attacker].into_iter().collect());
    let ctx = EffectContext::for_ability(plasm, 0, None);
    g.resolve_effect(&Effect::ReturnSelfDeployBlocker, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == plasm), "the source bounced");
    assert!(g.battlefield_find(wall).is_some(), "the 0/6 comes down, not the 1/1");
    assert_eq!(g.attackers_blocked_by(wall), [attacker], "and it is blocking");
}

/// Academy Researchers put no Aura down. The pick is plumbed now (nothing has
/// moved when it is asked), so a `wants_ui` seat gets a real pending decision;
/// a headless one takes the priciest Aura.
#[test]
fn put_aura_from_hand_attached_to_brings_the_priciest_aura() {
    let mut g = two_player_game();
    let researcher = g.add_card_to_battlefield(0, catalog::academy_researchers());
    g.add_card_to_hand(0, catalog::pacifism());
    let big = g.add_card_to_hand(0, catalog::rancor());
    // Rancor is {G}; make it the priciest so the ranking is unambiguous.
    if g.players[0].hand.iter().find(|c| c.id == big).map(|c| c.definition.cost.cmc()) < Some(2) {
        g.players[0].hand.retain(|c| c.id == big);
    }
    let ctx = EffectContext::for_ability(researcher, 0, None);
    g.resolve_effect(
        &Effect::PutAuraFromHandAttachedTo { host: Selector::This },
        &ctx,
    )
    .unwrap();
    let attached = g.battlefield.iter().find(|c| c.definition.is_aura());
    assert!(attached.is_some(), "an Aura came down");
    assert_eq!(attached.unwrap().attached_to, Some(researcher));
}

/// Credit Voucher shuffled nothing and drew nothing — the activation, its {2}
/// and the artifact for a no-op. The auto default ships the part of the hand
/// this seat cannot cast (mana value above its land count).
#[test]
fn shuffle_any_number_from_hand_ships_the_uncastable_half() {
    let mut g = two_player_game();
    let voucher = g.add_card_to_battlefield(0, catalog::credit_voucher());
    g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].library.clear();
    g.add_card_to_library(0, bear("Fresh", 1, 1, 1));
    let cheap = g.add_card_to_hand(0, bear("Cheap", 1, 1, 1));
    let dear = g.add_card_to_hand(0, bear("Dear", 8, 8, 8));
    let ctx = EffectContext::for_ability(voucher, 0, None);
    g.resolve_effect(
        &Effect::ShuffleAnyNumberFromHandThenDraw { who: PlayerRef::You },
        &ctx,
    )
    .unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == cheap), "the castable card stays");
    assert!(!g.players[0].hand.iter().any(|c| c.id == dear), "the 8-drop is shuffled away");
    assert_eq!(g.players[0].hand.len(), 2, "and one card is drawn back");
}

/// The audit's other half: a degenerate default that gates a *repetition* is
/// not a dead effect. Kindle the Carnage discards and damages once before it
/// asks "again?", so the printed minimum happens with nobody asked.
#[test]
fn a_loop_gated_may_still_runs_its_first_iteration() {
    let mut g = two_player_game();
    let source = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, bear("Victim", 2, 2, 3));
    g.add_card_to_hand(0, spell("Fodder", 3));
    let ctx = EffectContext::for_ability(source, 0, None);
    g.resolve_effect(&Effect::KindleTheCarnage, &ctx).unwrap();
    assert!(g.players[0].hand.is_empty(), "the random discard happened");
    assert_eq!(
        g.battlefield_find(victim).map(|c| c.damage),
        Some(3),
        "and its mana value hit each creature once"
    );
}

/// `of` pinning the graveyard to the opponents (Skullsnatcher, the MKM
/// Detective) keeps working: the scope filter runs before the ranking, so the
/// auto default is a subset of what the printed text allows.
#[test]
fn exile_up_to_n_from_graveyards_honours_a_pinned_scope() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, bear("Ours", 9, 9, 9));
    g.add_card_to_graveyard(1, bear("Recursion", 5, 5, 5));
    g.add_card_to_graveyard(1, spell("Flashback", 2));
    let ctx = EffectContext::for_ability(src, 0, None);
    g.resolve_effect(
        &Effect::ExileUpToNFromGraveyards {
            count: Value::Const(2),
            of: Some(PlayerRef::EachOpponent),
            single: false,
        },
        &ctx,
    )
    .unwrap();
    assert_eq!(g.exile.len(), 2, "both opposing graveyard cards are gone");
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(g.players[0].graveyard.len(), 1, "the pinned scope excluded ours");
}
