//! Functionality tests for `catalog::sets::decks::recent278`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game, Target};

/// Bog Badger grants your team menace only when kicked.
#[test]
fn bog_badger_kicked_grants_menace() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let badger = g.add_card_to_battlefield(0, catalog::bog_badger());
    let effect = catalog::bog_badger().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_ability(badger, 0, None);
    ctx.kicked = true;
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Menace),
        "team gained menace when kicked",
    );
}

/// Colossal Growth is +3/+3 unkicked and +4/+4 with trample/haste kicked.
#[test]
fn colossal_growth_kicker_scales() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::colossal_growth().effect.clone(), &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "unkicked +3/+3");

    let mut g2 = two_player_game();
    let bear2 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut kctx = EffectContext { targets: vec![Target::Permanent(bear2)], ..EffectContext::for_spell(0, None, 0, 0) };
    kctx.kicked = true;
    g2.resolve_effect(&catalog::colossal_growth().effect.clone(), &kctx).unwrap();
    let p = g2.computed_permanent(bear2).unwrap();
    assert_eq!(p.power, 6, "kicked +4/+4");
    assert!(p.keywords.contains(&crabomination::card::Keyword::Trample), "kicked grants trample");
}

/// Civic Gardener untaps a target on attack.
#[test]
fn civic_gardener_untaps_on_attack() {
    let mut g = two_player_game();
    let cg = g.add_card_to_battlefield(0, catalog::civic_gardener());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    let effect = catalog::civic_gardener().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(land)], ..EffectContext::for_ability(cg, 0, None) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(!g.battlefield_find(land).unwrap().tapped, "the land untapped");
}

/// Celebrity Fencer grows when another creature enters.
#[test]
fn celebrity_fencer_alliance_grows() {
    let mut g = two_player_game();
    let fencer = g.add_card_to_battlefield(0, catalog::celebrity_fencer());
    let effect = catalog::celebrity_fencer().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(fencer, 0, None)).unwrap();
    assert_eq!(g.computed_permanent(fencer).unwrap().power, 4, "3/2 → 4/3");
}

/// Commune with Spirits digs an enchantment or land into hand.
#[test]
fn commune_with_spirits_digs() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(vec![forest]),
    ]));
    g.resolve_effect(&catalog::commune_with_spirits().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "the land went to hand");
}

/// Buy Your Silence exiles a nonland permanent and gives its controller a Treasure.
#[test]
fn buy_your_silence_exiles_and_compensates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::buy_your_silence().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "the creature is exiled");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Treasure"),
        "its controller got a Treasure",
    );
}

/// Case the Joint draws two.
#[test]
fn case_the_joint_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    g.resolve_effect(&catalog::case_the_joint().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
}
