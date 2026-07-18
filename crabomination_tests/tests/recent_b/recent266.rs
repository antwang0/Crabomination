//! Functionality tests for `catalog::sets::decks::recent266`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};

fn kw(g: &crabomination::game::GameState, id: crabomination::card::CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|cp| cp.keywords.contains(&k))
}

/// Fungal Infection shrinks a creature and makes a Saproling.
#[test]
fn fungal_infection_shrinks_and_spawns() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let effect = catalog::fungal_infection().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "-1/-1 applied");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Saproling"),
        "Saproling token created"
    );
}

/// Prakhata Pillar-Bug buys lifelink with {B}.
#[test]
fn prakhata_pillar_bug_gains_lifelink() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prakhata_pillar_bug());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(kw(&g, id, Keyword::Lifelink));
}

/// Savai Sabertooth is a vanilla 3/1.
#[test]
fn savai_sabertooth_is_vanilla() {
    let d = catalog::savai_sabertooth();
    assert_eq!((d.power, d.toughness), (3, 1));
    assert!(d.keywords.is_empty() && d.triggered_abilities.is_empty());
}

/// Territorial Boar grows when a big creature enters under your control.
#[test]
fn territorial_boar_grows_on_big_creature() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let boar = g.add_card_to_battlefield(0, catalog::territorial_boar());
    // A 4/4 entering (cast, so the watcher trigger fires) grows the boar.
    let angel = g.add_card_to_hand(0, catalog::serra_angel()); // 4/4, {3}{W}{W}
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the angel");
    drain_stack(&mut g);
    let cp = g.computed_permanent(boar).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "boar pumped to 3/3");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "gained vigilance");
}

/// Might of Murasa gives +3/+3, or +5/+5 when kicked.
#[test]
fn might_of_murasa_pumps_scaled_by_kicker() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let effect = catalog::might_of_murasa().effect;
    // Unkicked: +3/+3 → 5/5.
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "+3/+3 base");
    // Kicked: +5/+5 instead → another +5 power on top.
    let mut kicked = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    kicked.kicked = true;
    g.resolve_effect(&effect, &kicked).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 10, "kicked adds +5/+5");
}
