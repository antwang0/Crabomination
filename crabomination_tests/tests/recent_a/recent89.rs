//! Functionality tests for `catalog::sets::decks::recent89`.

use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

#[test]
fn flame_burst_and_lightning_blast_burn_face() {
    for (mk, dmg, red, genc) in [
        (catalog::flame_burst as fn() -> crabomination::card::CardDefinition, 2, 1, 1),
        (catalog::lightning_blast as fn() -> crabomination::card::CardDefinition, 4, 1, 3),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, mk());
        g.players[0].mana_pool.add(Color::Red, red);
        g.players[0].mana_pool.add_colorless(genc);
        let life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - dmg);
    }
}

#[test]
fn inferno_hits_all_creatures_and_players() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inferno());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Inferno");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(),
        "6 damage cleared both 2/2s");
    assert_eq!(g.players[0].life, l0 - 6, "each player took 6");
    assert_eq!(g.players[1].life, l1 - 6);
}

#[test]
fn crater_hellion_sweeps_other_creatures_on_etb() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hellion = g.add_card_to_battlefield(0, catalog::crater_hellion());
    g.fire_self_etb_triggers(hellion, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "4 damage killed the 2/2");
    assert!(g.battlefield_find(hellion).is_some(), "the Hellion spared itself");
}
