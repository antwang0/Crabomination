//! Tests for the recent299 Ravnica batch 9 (land-animator + guild utility).

use crabomination::card::{CardType, LandType};
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState};
use crabomination::mana::Color;

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
    g.players[0].mana_pool.add_colorless(8);
}

#[test]
fn woodwraith_corrupter_animates_a_forest() {
    let mut g = two_player_game();
    let wc = g.add_card_to_battlefield(0, catalog::woodwraith_corrupter());
    g.clear_sickness(wc);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wc, ability_index: 0, target: Some(Target::Permanent(forest)),
        additional_targets: vec![], x_value: None,
    }).expect("animate the Forest");
    drain_stack(&mut g);
    let cp = g.computed_permanent(forest).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "Forest is a 4/4");
    assert!(cp.card_types.contains(&CardType::Creature), "now a creature");
    assert!(cp.subtypes.land_types.contains(&LandType::Forest), "still a Forest land");
}

#[test]
fn bond_of_agony_drains_each_opponent_for_x() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::bond_of_agony());
    flood(&mut g);
    let (me, foe) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast for X=3");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me - 3, "paid 3 life as an additional cost");
    assert_eq!(g.players[1].life, foe - 3, "each other player lost 3");
}

#[test]
fn enemy_of_the_guildpact_has_protection_from_multicolored() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::enemy_of_the_guildpact());
    assert!(g.computed_permanent(e).unwrap().keywords.contains(&Keyword::ProtectionFromMulticolored));
}

#[test]
fn court_hussar_digs_three_for_one() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    let hussar = g.add_card_to_hand(0, catalog::court_hussar());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: hussar, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Cast the Hussar (leaves hand), then its ETB digs one of three into hand
    // → net +1 over the pre-Hussar baseline.
    assert_eq!(g.players[0].hand.len(), hand + 1, "one of three cards dug into hand");
}

#[test]
fn overrule_counters_unless_paid_and_gains_life() {
    let mut g = two_player_game();
    // Opponent casts a bear with no spare mana to pay Overrule's {X}.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    let overrule = g.add_card_to_hand(0, catalog::overrule());
    flood(&mut g);
    let life = g.players[0].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: overrule, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: Some(2),
    }).expect("cast overrule X=2");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "bear countered (couldnt pay 2)");
    assert_eq!(g.players[0].life, life + 2, "gained X=2 life");
}
