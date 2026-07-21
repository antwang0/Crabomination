//! Functionality tests for Ravnica: City of Guilds (RAV) gap cards.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

/// Static stat/keyword lines for the vanilla and french-vanilla RAV creatures.
#[test]
fn rav_stat_and_keyword_lines() {
    let sword = catalog::boros_swiftblade();
    assert_eq!((sword.power, sword.toughness), (1, 2));
    assert!(sword.keywords.contains(&Keyword::DoubleStrike));

    let hawk = catalog::courier_hawk();
    assert_eq!((hawk.power, hawk.toughness), (1, 2));
    assert!(hawk.keywords.contains(&Keyword::Flying) && hawk.keywords.contains(&Keyword::Vigilance));

    let eq = catalog::conclave_equenaut();
    assert!(eq.keywords.contains(&Keyword::Convoke) && eq.keywords.contains(&Keyword::Flying));
}

/// Barbarian Riftcutter sacrifices itself to destroy a land.
#[test]
fn barbarian_riftcutter_destroys_a_land() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let rift = g.add_card_to_battlefield(0, catalog::barbarian_riftcutter());
    g.clear_sickness(rift);
    let land = g.add_card_to_battlefield(1, catalog::forest());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rift, ability_index: 0,
        target: Some(Target::Permanent(land)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Riftcutter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "target land destroyed");
    assert!(g.battlefield_find(rift).is_none(), "Riftcutter sacrificed as a cost");
}

/// Dromad Purebred gains its controller 1 life whenever it's dealt damage.
#[test]
fn dromad_purebred_gains_life_on_damage() {
    let mut g = two_player_game();
    let dromad = g.add_card_to_battlefield(0, catalog::dromad_purebred());
    let life0 = g.players[0].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(dromad), 3, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1, "gained 1 life (once) from being dealt damage");
}

/// Gate Hound grants your creatures vigilance only while it's enchanted.
#[test]
fn gate_hound_team_vigilance_while_enchanted() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let hound = g.add_card_to_battlefield(0, catalog::gate_hound());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance),
        "no vigilance while unenchanted");
    // Enchant the hound with an aura (Pacifism), turning on the static.
    let pacifism = g.add_card_to_hand(0, catalog::pacifism());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: pacifism, target: Some(Target::Permanent(hound)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("attach aura to Gate Hound");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance),
        "your creatures gain vigilance while Gate Hound is enchanted");
}
