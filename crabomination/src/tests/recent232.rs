//! Functionality tests for `catalog::sets::decks::recent232`.

use crate::card::{CardType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::effects::EffectContext;
use crate::game::{drain_stack, two_player_game};

/// Haunted Screen taps for W or B.
#[test]
fn haunted_screen_taps_for_wb() {
    use crate::mana::Color;
    let mut g = two_player_game();
    let screen = g.add_card_to_battlefield(0, catalog::haunted_screen());
    let effect = catalog::haunted_screen().activated_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(screen, 0, None)).unwrap();
    let pool = g.players[0].mana_pool.total();
    assert_eq!(pool, 1, "one mana produced");
    assert!(
        g.players[0].mana_pool.amount(Color::White) == 1 || g.players[0].mana_pool.amount(Color::Black) == 1,
        "the mana is white or black",
    );
}

/// Haunted Screen's {7} ability makes it a 7/7 Spirit creature, once.
#[test]
fn haunted_screen_animates_to_7_7_spirit() {
    let mut g = two_player_game();
    let screen = g.add_card_to_battlefield(0, catalog::haunted_screen());
    let effect = catalog::haunted_screen().activated_abilities[2].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(screen, 2, None)).unwrap();
    let cp = g.computed_permanent(screen).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "becomes a creature");
    assert!(cp.card_types.contains(&CardType::Artifact), "still an artifact");
    assert_eq!(cp.power, 7, "0/0 + seven +1/+1 counters = 7/7");
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Spirit), "gains Spirit");
}

/// Fear of Infinity is a flying, lifelink Nightmare that can't block.
#[test]
fn fear_of_infinity_keywords() {
    let def = catalog::fear_of_infinity();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert!(def.keywords.contains(&Keyword::CantBlock));
    assert!(def.card_types.contains(&CardType::Enchantment) && def.card_types.contains(&CardType::Creature));
}

/// Its Eerie recursion returns it from the graveyard to hand when you accept.
#[test]
fn fear_of_infinity_recurs_from_graveyard() {
    let mut g = two_player_game();
    let fear = g.add_card_to_graveyard(0, catalog::fear_of_infinity());
    // Resolve the recursion body (the "may" is accepted).
    let effect = catalog::fear_of_infinity().triggered_abilities[0].effect.clone();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.resolve_effect(&effect, &EffectContext::for_trigger(fear, 0, None, 0)).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fear), "returned to hand");
}
