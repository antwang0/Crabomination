//! CR 707.10 — a copy of a spell is not cast. Magecraft says "whenever you
//! cast OR COPY an instant or sorcery spell", so it fires for the copy too
//! (`EventSpec::or_copy`, run by the dispatcher once per `SpellsCopied`),
//! while a plain "whenever you cast" trigger still does not.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

/// Bolt, then Reverberate copying it: returns how many of `name` seat 0 has.
fn bolt_and_copy(listener: crabomination::card::CardDefinition, name: &str) -> usize {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, listener);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let rev = g.add_card_to_hand(0, catalog::reverberate());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rev,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reverberate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "the Bolt and its copy both hit");
    g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == name).count()
}

#[test]
fn cr_707_10_magecraft_fires_for_a_copy() {
    // Bolt cast, Reverberate cast, the copy: three.
    assert_eq!(bolt_and_copy(catalog::storm_kiln_artist(), "Treasure"), 3);
}

#[test]
fn cr_707_10_a_cast_trigger_ignores_the_copy() {
    // "Whenever you cast an instant or sorcery spell": the two casts only.
    assert_eq!(bolt_and_copy(catalog::young_pyromancer(), "Elemental"), 2);
}

/// CR 603.2 — "Whenever THIS creature becomes the target of a spell or
/// ability an opponent controls" (Mossdog): an opponent's Giant Growth on
/// another of your creatures doesn't grow it; one on Mossdog does.
#[test]
fn cr_603_2_mossdog_counts_only_itself_being_targeted() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::mossdog());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let counters = |g: &GameState| g.battlefield_find(dog).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let growth_on = |g: &mut GameState, target| {
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PreCombatMain;
        let spell = g.add_card_to_hand(1, catalog::giant_growth());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Giant Growth");
        drain_stack(g);
    };
    growth_on(&mut g, bear);
    assert_eq!(counters(&g), 0, "another creature was targeted");
    growth_on(&mut g, dog);
    assert_eq!(counters(&g), 1, "Mossdog itself was targeted");
}
