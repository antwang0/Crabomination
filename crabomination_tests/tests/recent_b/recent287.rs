//! Functionality tests for `catalog::sets::decks::recent287` (OTJ gap batch).

use crabomination::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, Subtypes,
};
use crabomination::game::{two_player_game, GameEvent, GameState};

/// A bare 2/2 Mount creature for exercising Miriam's Mount/Vehicle riders.
fn mount_2_2() -> CardDefinition {
    CardDefinition {
        name: "Test Mount",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mount], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Miriam grants your Mounts/Vehicles hexproof during your turn only (CR 611.2).
#[test]
fn miriam_turn_gated_mount_hexproof() {
    let mut g = two_player_game();
    let _miriam = g.add_card_to_battlefield(0, crabomination::catalog::miriam_herd_whisperer());
    let mount = g.add_card_to_battlefield(0, mount_2_2());
    let has_hexproof =
        |g: &GameState| g.computed_permanent(mount).unwrap().keywords.contains(&Keyword::Hexproof);
    g.active_player_idx = 0;
    assert!(has_hexproof(&g), "Mount has hexproof on your turn");
    g.active_player_idx = 1;
    assert!(!has_hexproof(&g), "no hexproof on the opponent's turn");
}

/// Miriam puts a +1/+1 counter on a Mount you control when it attacks.
#[test]
fn miriam_counters_attacking_mount() {
    let mut g = two_player_game();
    let _miriam = g.add_card_to_battlefield(0, crabomination::catalog::miriam_herd_whisperer());
    let mount = g.add_card_to_battlefield(0, mount_2_2());
    g.dispatch_triggers_for_events(&[GameEvent::AttackerDeclared(mount)]);
    crabomination::game::drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(mount).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "attacking Mount gets a +1/+1 counter"
    );
}

/// Vadmir gains a counter once per turn on committing a crime, and gains
/// menace + lifelink once it has four or more +1/+1 counters.
#[test]
fn vadmir_crime_counters_and_keyword_threshold() {
    let mut g = two_player_game();
    let vadmir = g.add_card_to_battlefield(0, crabomination::catalog::vadmir_new_blood());
    // Two crimes in one turn → only one counter (once each turn).
    g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
    crabomination::game::drain_stack(&mut g);
    g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
    crabomination::game::drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(vadmir).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "crime counter triggers only once each turn"
    );
    // Below threshold: no menace yet.
    assert!(!g.computed_permanent(vadmir).unwrap().keywords.contains(&Keyword::Menace));
    // Bump to four counters → menace + lifelink.
    g.battlefield_find_mut(vadmir).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let c = g.computed_permanent(vadmir).unwrap();
    assert!(c.keywords.contains(&Keyword::Menace), "menace at 4+ counters");
    assert!(c.keywords.contains(&Keyword::Lifelink), "lifelink at 4+ counters");
}

/// Skyserpent Seeker's exhaust ability reveals until two lands, puts them onto
/// the battlefield tapped, and grows itself with a +1/+1 counter.
#[test]
fn skyserpent_seeker_ramps_two_lands() {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::{drain_stack, effects::EffectContext};
    let mut g = two_player_game();
    let snake = g.add_card_to_battlefield(0, catalog::skyserpent_seeker());
    // Library top → bottom: a nonland, then two Forests.
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let ability = catalog::skyserpent_seeker().activated_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(snake, 0, None);
    g.resolve_effect(&ability, &ctx).unwrap();
    drain_stack(&mut g);
    let forests = g.battlefield.iter().filter(|c| c.definition.name == "Forest").count();
    assert_eq!(forests, 2, "two Forests put onto the battlefield");
    assert!(
        g.battlefield.iter().filter(|c| c.definition.name == "Forest").all(|c| c.tapped),
        "the revealed lands enter tapped",
    );
    assert_eq!(
        g.battlefield_find(snake).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Skyserpent grows a +1/+1 counter",
    );
}
