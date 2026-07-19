//! Functionality tests for `catalog::sets::decks::recent274`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::two_player_game;

/// Emergent Haunting flips into a 3/3 flying Spirit when the end-step trigger
/// resolves (it starts as a noncreature enchantment, so the gate passes).
#[test]
fn emergent_haunting_self_animates() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::emergent_haunting());
    let effect = catalog::emergent_haunting().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(e, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    let p = g.computed_permanent(e).unwrap();
    assert!(p.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
    assert_eq!((p.power, p.toughness), (3, 3), "3/3 body");
    assert!(p.keywords.contains(&crabomination::card::Keyword::Flying), "gains flying");
}

/// The Surveil activated ability is present and cheap.
#[test]
fn emergent_haunting_has_surveil() {
    let def = catalog::emergent_haunting();
    assert_eq!(def.activated_abilities.len(), 1, "the {{2}}{{U}} surveil ability");
}

/// Jolene's attack trigger only fires when a power-4+ attacker is declared, and
/// mints a Treasure when it does.
#[test]
fn jolene_makes_treasure_on_beefy_attack() {
    let mut g = two_player_game();
    let jolene = g.add_card_to_battlefield(0, catalog::jolene_plundering_pugilist());
    let effect = catalog::jolene_plundering_pugilist().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(jolene, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "a Treasure token entered"
    );
}

/// Jolene's sacrifice-a-Treasure ping targets any target.
#[test]
fn jolene_ping_ability_costs_a_treasure() {
    let def = catalog::jolene_plundering_pugilist();
    let ab = &def.activated_abilities[0];
    assert!(ab.sac_other_filter.is_some(), "requires sacrificing a Treasure");
}
