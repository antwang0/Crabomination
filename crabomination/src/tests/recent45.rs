//! Functionality tests for `catalog::sets::decks::recent45` — white removal.

use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::two_player_game;
use crate::game::*;

/// Resolve a single-target spell effect against `target`.
fn resolve_on(g: &mut GameState, def: crate::card::CardDefinition, target: Target) {
    let ctx = EffectContext::for_spell(0, Some(target), 0, 0);
    g.resolve_effect(&def.effect, &ctx).unwrap();
    drain_stack(g);
}

#[test]
fn fragmentize_only_hits_cheap_artifacts_and_enchantments() {
    let mut g = two_player_game();
    let cheap = g.add_card_to_battlefield(1, catalog::ratchet_bomb()); // MV 2 artifact
    resolve_on(&mut g, catalog::fragmentize(), Target::Permanent(cheap));
    assert!(g.battlefield_find(cheap).is_none(), "MV-2 artifact destroyed");
}

#[test]
fn erase_exiles_an_enchantment() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
    resolve_on(&mut g, catalog::erase(), Target::Permanent(ench));
    assert!(g.exile.iter().any(|c| c.id == ench), "enchantment exiled");
}

#[test]
fn rebuke_destroys_an_attacker() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking.push(crate::game::types::Attack {
        attacker: atk,
        target: crate::game::types::AttackTarget::Player(0),
    });
    resolve_on(&mut g, catalog::rebuke(), Target::Permanent(atk));
    assert!(g.battlefield_find(atk).is_none(), "attacking creature destroyed");
}

#[test]
fn depopulate_spares_tokens() {
    let mut g = two_player_game();
    let real = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // A token survives the nontoken-only wipe.
    let bear_token = crate::card::TokenDefinition {
        name: "Bear".into(),
        power: 2,
        toughness: 2,
        card_types: vec![crate::card::CardType::Creature],
        ..Default::default()
    };
    let tok = g.add_token_to_battlefield(0, &bear_token);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::depopulate().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(real).is_none(), "nontoken creature destroyed");
    assert!(g.battlefield_find(tok).is_some(), "token survives");
}

#[test]
fn crib_swap_exiles_and_gifts_a_shapeshifter() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_on(&mut g, catalog::crib_swap(), Target::Permanent(victim));
    assert!(g.exile.iter().any(|c| c.id == victim), "target exiled");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Shapeshifter"),
        "its controller gets a 1/1 Shapeshifter"
    );
}
