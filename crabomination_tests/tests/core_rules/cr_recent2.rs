//! Comprehensive-Rules conformance for behaviours exercised by the recent274–276
//! batches: layer-changing self-animation (CR 613 / 711), once-only Renown
//! (CR 702.112), and Deathtouch making any nonzero damage lethal (CR 702.2c).

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{two_player_game, Target};

/// CR 613 / 711.2 — when a noncreature enchantment becomes a creature it is a
/// creature *in addition to* its other types. Emergent Haunting keeps its
/// Enchantment type after the end-step animation flips it into a 3/3 Spirit.
#[test]
fn cr_613_self_animated_enchantment_keeps_enchantment_type() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::emergent_haunting());
    let effect = catalog::emergent_haunting().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(e, 0, None)).unwrap();
    let p = g.computed_permanent(e).unwrap();
    assert!(p.card_types.contains(&CardType::Creature), "gained Creature");
    assert!(p.card_types.contains(&CardType::Enchantment), "still an Enchantment (CR 711.2)");
}

/// CR 702.112c — Renown triggers only while the creature "isn't renowned": the
/// counters and renowned flag are set once, and a second combat-damage
/// resolution adds nothing further.
#[test]
fn cr_702_112_renown_fires_once() {
    let mut g = two_player_game();
    let berserker = g.add_card_to_battlefield(0, catalog::scab_clan_berserker()); // renown 1
    let effect = catalog::scab_clan_berserker()
        .triggered_abilities
        .iter()
        .find(|t| matches!(t.effect, crabomination::effect::Effect::If { .. }))
        .map(|t| t.effect.clone())
        .expect("renown trigger present");
    let ctx = EffectContext::for_ability(berserker, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(berserker).unwrap().renowned, "becomes renowned");
    assert_eq!(g.computed_permanent(berserker).unwrap().power, 3, "2/2 → 3/3");
    // Second resolution: the "if it isn't renowned" gate suppresses it.
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(berserker).unwrap().power, 3, "no further growth");
}

/// CR 702.2c — any nonzero amount of damage from a deathtouch source is lethal.
/// One point of deathtouch damage kills a 1/3 at the next SBA check even though
/// its toughness is 3.
#[test]
fn cr_702_2c_deathtouch_one_point_is_lethal() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::coastal_bulwark()); // 1/3
    let dt = g.add_card_to_battlefield(0, catalog::toxic_scorpion()); // deathtouch
    assert!(catalog::toxic_scorpion().keywords.contains(&Keyword::Deathtouch));
    let ctx = EffectContext {
        targets: vec![Target::Permanent(target)],
        ..EffectContext::for_ability(dt, 0, None)
    };
    g.resolve_effect(
        &crabomination::effect::Effect::DealDamage {
            to: crabomination::effect::Selector::Target(0),
            amount: crabomination::effect::Value::ONE,
        },
        &ctx,
    )
    .unwrap();
    g.check_state_based_actions();
    assert!(
        !g.battlefield.iter().any(|c| c.id == target),
        "1 point of deathtouch damage is lethal to the 1/3",
    );
}
