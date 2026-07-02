//! Functionality tests for `catalog::sets::decks::recent85`.

use crate::card::{CreatureType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::two_player_game;
use crate::game::*;

/// Choose `ct` at the next creature-type decision, then fire `id`'s ETB.
fn enter_choosing(g: &mut GameState, id: CardId, ct: CreatureType) {
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(ct)]));
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
}

#[test]
fn steely_resolve_grants_shroud_to_chosen_type() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // Bear
    let sr = g.add_card_to_battlefield(0, catalog::steely_resolve());
    enter_choosing(&mut g, sr, CreatureType::Bear);
    let cp = g.compute_battlefield();
    assert!(cp.iter().find(|c| c.id == bear).unwrap().keywords.contains(&Keyword::Shroud),
        "chosen-type Bear has shroud");
}

#[test]
fn kindred_boon_grants_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let kb = g.add_card_to_battlefield(0, catalog::kindred_boon());
    enter_choosing(&mut g, kb, CreatureType::Bear);
    assert!(g.compute_battlefield().iter().find(|c| c.id == bear).unwrap()
        .keywords.contains(&Keyword::Indestructible), "chosen-type Bear is indestructible");
}

#[test]
fn cover_of_darkness_grants_fear() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cd = g.add_card_to_battlefield(0, catalog::cover_of_darkness());
    enter_choosing(&mut g, cd, CreatureType::Bear);
    assert!(g.compute_battlefield().iter().find(|c| c.id == bear).unwrap()
        .keywords.contains(&Keyword::Fear), "chosen-type Bear has fear");
}

/// CR 702.36 — Fear granted by Cover of Darkness restricts blockers through the
/// computed-keyword combat path (a green blocker can't block; an artifact can).
#[test]
fn cr_702_36_granted_fear_restricts_blockers() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green Bear
    let cd = g.add_card_to_battlefield(0, catalog::cover_of_darkness());
    enter_choosing(&mut g, cd, CreatureType::Bear);
    let atk_kws = g.computed_permanent(attacker).unwrap().keywords.clone();
    assert!(atk_kws.contains(&Keyword::Fear), "the Bear was granted Fear");
    let check = |g: &mut GameState, def: crate::card::CardDefinition| -> bool {
        let blk = g.add_card_to_battlefield(1, def);
        let inst = g.battlefield_find(blk).unwrap().clone();
        let cp = g.computed_permanent(blk).unwrap();
        crate::game::can_block_attacker_computed(&inst, &cp, &atk_kws, &[], 2)
    };
    assert!(!check(&mut g, catalog::grizzly_bears()), "green creature can't block granted Fear");
    assert!(check(&mut g, catalog::ornithopter()), "artifact creature still can");
}

#[test]
fn elvish_clancaller_anthems_other_elves() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // Elf
    let caller = g.add_card_to_battlefield(0, catalog::elvish_clancaller());
    let cp = g.compute_battlefield();
    let e = cp.iter().find(|c| c.id == elf).unwrap();
    assert_eq!((e.power, e.toughness), (2, 2), "other Elf gets +1/+1");
    // The Clancaller does not pump itself.
    let c = cp.iter().find(|c| c.id == caller).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1), "the lord excludes itself");
}

#[test]
fn elvish_clancaller_tutors_a_copy() {
    let mut g = two_player_game();
    let caller = g.add_card_to_battlefield(0, catalog::elvish_clancaller());
    g.clear_sickness(caller);
    let copy = g.add_card_to_library(0, catalog::elvish_clancaller());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(copy))]));
    g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: caller, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tutor");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "found the named copy");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Elvish Clancaller"));
}
