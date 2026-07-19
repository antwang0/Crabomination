//! Functionality tests for `catalog::sets::decks::recent228`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

fn count_named(g: &crabomination::game::GameState, ctrl: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == ctrl && c.definition.name == name).count()
}

/// Pearl Medallion makes a white spell cost {1} less — a {1}{W} creature is
/// castable off a single {W}.
#[test]
fn pearl_medallion_reduces_white_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pearl_medallion());
    let spell = g.add_card_to_hand(0, catalog::seasoned_consultant()); // {1}{W}
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {W} after the {1} reduction");
}

/// All five Medallions carry a matching-color cost reduction.
#[test]
fn medallions_all_reduce_their_color() {
    use crabomination::effect::StaticEffect;
    use crabomination::card::SelectionRequirement as R;
    for (f, c) in [
        (catalog::pearl_medallion as fn() -> _, Color::White),
        (catalog::sapphire_medallion, Color::Blue),
        (catalog::jet_medallion, Color::Black),
        (catalog::ruby_medallion, Color::Red),
        (catalog::emerald_medallion, Color::Green),
    ] {
        let def = f();
        assert!(def.static_abilities.iter().any(|s| matches!(
            &s.effect,
            StaticEffect::CostReduction { filter: R::HasColor(col), amount: 1 } if *col == c
        )));
    }
}

/// Meteoric Mace grants +4/+0 and trample to the creature it equips.
#[test]
fn meteoric_mace_buffs_equipped() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mace = g.add_card_to_battlefield(0, catalog::meteoric_mace());
    g.battlefield_find_mut(mace).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 6, "2 + 4 = 6");
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Reef Worm leaves a 3/3 Fish when it dies.
#[test]
fn reef_worm_makes_a_fish() {
    let mut g = two_player_game();
    let worm = g.add_card_to_battlefield(0, catalog::reef_worm());
    let effect = catalog::reef_worm().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(worm, 0, None, 0)).unwrap();
    assert_eq!(count_named(&g, 0, "Fish"), 1, "a Fish swims up");
}

/// Deserted Temple untaps a tapped land.
#[test]
fn deserted_temple_untaps_a_land() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(forest).unwrap().tapped = true;
    let temple = g.add_card_to_battlefield(0, catalog::deserted_temple());
    let effect = catalog::deserted_temple().activated_abilities[1].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(forest)], ..EffectContext::for_ability(temple, 0, None) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(!g.battlefield_find(forest).unwrap().tapped, "land untapped");
}

/// Barbarian Ring pings you for its red mana.
#[test]
fn barbarian_ring_pings_you() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(0, catalog::barbarian_ring());
    let effect = catalog::barbarian_ring().activated_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(ring, 0, None)).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].life, 19, "1 damage to you");
}

/// Glass Casket exiles a cheap opposing creature.
#[test]
fn glass_casket_exiles_cheap_creature() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // mv 2
    let casket = g.add_card_to_battlefield(0, catalog::glass_casket());
    let effect = catalog::glass_casket().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_trigger(casket, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.exile.iter().any(|c| c.id == enemy), "creature exiled");
}

/// Crystal Grotto's second ability makes one mana of any color.
#[test]
fn crystal_grotto_makes_any_color() {
    let mut g = two_player_game();
    let grotto = g.add_card_to_battlefield(0, catalog::crystal_grotto());
    let effect = catalog::crystal_grotto().activated_abilities[1].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(grotto, 0, None)).unwrap();
    assert_eq!(g.players[0].mana_pool.total(), 1, "one mana produced");
}

/// Molten Duplication mints a hasty artifact copy.
#[test]
fn molten_duplication_copies_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::molten_duplication().effect.clone(), &ctx).unwrap();
    let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Grizzly Bears").expect("token copy");
    let cp = g.computed_permanent(copy.id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Haste), "gains haste");
    assert!(cp.card_types.contains(&crabomination::card::CardType::Artifact), "also an artifact");
}

/// Shackle Slinger taps an untapped opposing creature.
#[test]
fn shackle_slinger_taps_untapped() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ss = g.add_card_to_battlefield(0, catalog::shackle_slinger());
    let effect = catalog::shackle_slinger().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_trigger(ss, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(enemy).unwrap().tapped, "untapped creature gets tapped");
}

/// Thunder Salvo deals at least 2 (base) to a creature.
#[test]
fn thunder_salvo_burns_a_creature() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::reef_worm()); // 0/1
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::thunder_salvo().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == enemy), "0/1 dies to the salvo");
}

/// Thunder Salvo scales: base 2 plus each *other* spell cast this turn. With
/// the Salvo itself counted (spells_cast_this_turn = 3 → two others) it deals 4.
#[test]
fn thunder_salvo_scales_with_other_spells() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::academy_wall()); // 0/5, survives 4
    g.players[0].spells_cast_this_turn = 3; // Salvo + two others
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::thunder_salvo().effect.clone(), &ctx).unwrap();
    assert_eq!(g.battlefield_find(enemy).unwrap().damage, 4, "2 + 2 other spells");
}

/// Fledgling Dragon grows to 5/5 with threshold active.
#[test]
fn fledgling_dragon_grows_with_threshold() {
    let mut g = two_player_game();
    let drag = g.add_card_to_battlefield(0, catalog::fledgling_dragon());
    assert_eq!(g.computed_permanent(drag).unwrap().power, 2, "2/2 without threshold");
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    assert_eq!(g.computed_permanent(drag).unwrap().power, 5, "+3/+3 with seven cards in gy");
}

/// Annoyed Altisaur has reach, trample, and a cascade trigger.
#[test]
fn annoyed_altisaur_has_cascade() {
    let def = catalog::annoyed_altisaur();
    assert!(def.keywords.contains(&Keyword::Reach) && def.keywords.contains(&Keyword::Trample));
    assert!(!def.triggered_abilities.is_empty(), "carries a cascade trigger");
}
