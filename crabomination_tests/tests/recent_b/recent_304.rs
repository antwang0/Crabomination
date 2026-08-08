//! Tests for the recent304 Dissension batch 3.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

#[test]
fn rakdos_ragemutt_has_lifelink_and_haste() {
    let mut g = two_player_game();
    let rr = g.add_card_to_battlefield(0, catalog::rakdos_ragemutt());
    let kw = g.computed_permanent(rr).unwrap().keywords.clone();
    assert!(kw.contains(&Keyword::Lifelink) && kw.contains(&Keyword::Haste));
}

#[test]
fn delirium_skeins_makes_everyone_discard_three() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let ds = g.add_card_to_hand(0, catalog::delirium_skeins());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ds, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty() && g.players[1].hand.is_empty(), "each discarded three");
}

#[test]
fn vision_skeins_draws_two_each() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let vs = g.add_card_to_hand(0, catalog::vision_skeins());
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: vs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Caster nets +2 minus the Vision Skeins that left hand; opponent nets +2.
    assert_eq!(g.players[1].hand.len(), h1 + 2, "opponent drew two");
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 2, "caster drew two (spell left hand)");
}

#[test]
fn psychotic_fury_only_pumps_multicolored() {
    let mut g = two_player_game();
    let gold = g.add_card_to_battlefield(0, catalog::rakdos_ragemutt()); // B/R multicolored
    let pf = g.add_card_to_hand(0, catalog::psychotic_fury());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: pf, target: Some(Target::Permanent(gold)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast on the gold creature");
    drain_stack(&mut g);
    assert!(g.computed_permanent(gold).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

#[test]
fn might_of_the_nephilim_scales_with_colors() {
    let mut g = two_player_game();
    let gold = g.add_card_to_battlefield(0, catalog::rakdos_ragemutt()); // 2 colors → +4/+4
    let base = g.computed_permanent(gold).unwrap().power;
    let m = g.add_card_to_hand(0, catalog::might_of_the_nephilim());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: Some(Target::Permanent(gold)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(gold).unwrap().power, base + 4, "+2/+2 per color × 2 colors");
}

#[test]
fn stomp_and_howl_destroys_an_artifact_and_an_enchantment() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::codex_shredder());
    let ench = g.add_card_to_battlefield(1, catalog::pacifism());
    let sh = g.add_card_to_hand(0, catalog::stomp_and_howl());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: sh, target: Some(Target::Permanent(art)),
        additional_targets: vec![Target::Permanent(ench)], mode: None, x_value: None,
    }).expect("cast on artifact + enchantment");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none() && g.battlefield_find(ench).is_none(),
        "both destroyed");
}
