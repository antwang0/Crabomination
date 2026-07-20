//! Functionality tests for Dissension (DIS) gap cards in `catalog::sets::dis`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

/// Assault Zeppelid is a 3/3 with flying and trample.
#[test]
fn assault_zeppelid_flying_trample() {
    let z = catalog::assault_zeppelid();
    assert_eq!((z.power, z.toughness), (3, 3));
    assert!(z.keywords.contains(&Keyword::Flying));
    assert!(z.keywords.contains(&Keyword::Trample));
}

/// Sky Hussar untaps all your creatures when it enters.
#[test]
fn sky_hussar_untaps_your_creatures_on_etb() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.move_card_to_battlefield_for_test(0, catalog::sky_hussar());
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "ETB untapped your creature");
}

/// Stalking Vengeance turns a dying creature's power into damage to a player.
#[test]
fn stalking_vengeance_death_burns_target() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stalking_vengeance());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let life1 = g.players[1].life;
    // Kill the bear via lethal damage → SBA death → Stalking Vengeance trigger.
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2, "dying 2/2 dealt 2 to the opponent");
}

/// Azorius Herald is unblockable, gains 4 on entry, and sticks when {U} was
/// spent to cast it.
#[test]
fn azorius_herald_stays_when_cast_with_blue() {
    let mut g = two_player_game();
    let herald = g.add_card_to_hand(0, catalog::azorius_herald());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: herald, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Azorius Herald with {U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 4, "gained 4 life");
    assert!(g.battlefield.iter().any(|c| c.id == herald), "not sacrificed — U was spent");
    assert!(catalog::azorius_herald().keywords.contains(&Keyword::Unblockable));
}
