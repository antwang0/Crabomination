//! Functionality tests for `catalog::sets::decks::recent88`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, extra: Vec<Target>, x: Option<u32>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id, target, additional_targets: extra, mode: None, x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

#[test]
fn searing_wind_deals_five() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::searing_wind());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[1].life;
    cast(&mut g, id, Some(Target::Player(1)), vec![], None);
    assert_eq!(g.players[1].life, life - 5);
}

#[test]
fn lava_burst_deals_x() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lava_burst());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    cast(&mut g, id, Some(Target::Player(1)), vec![], Some(3));
    assert_eq!(g.players[1].life, life - 3, "X=3 → 3 damage");
}

#[test]
fn jagged_lightning_hits_two_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::jagged_lightning());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id, Some(Target::Permanent(a)), vec![Target::Permanent(b)], None);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(),
        "3 damage killed both 2/2s");
}

#[test]
fn rain_of_embers_spares_flyers() {
    let mut g = two_player_game();
    let ground = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1, no flying
    let flyer = g.add_card_to_battlefield(1, catalog::suntail_hawk()); // 1/1 flying
    let id = g.add_card_to_hand(0, catalog::rain_of_embers());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id, None, vec![], None);
    assert!(g.battlefield_find(ground).is_none(), "the grounded 1/1 died");
    assert!(g.battlefield_find(flyer).is_some(), "the flyer was spared");
}

#[test]
fn thunderfoot_baloth_pumps_and_tramples_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let baloth = g.add_card_to_battlefield(0, catalog::thunderfoot_baloth());
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "other creature +2/+2");
    assert!(b.keywords.contains(&Keyword::Trample), "other creature has trample");
    // The Baloth doesn't buff itself.
    let self_ = cp.iter().find(|c| c.id == baloth).unwrap();
    assert_eq!((self_.power, self_.toughness), (5, 5));
    assert!(!self_.keywords.contains(&Keyword::Trample), "source excluded");
}
