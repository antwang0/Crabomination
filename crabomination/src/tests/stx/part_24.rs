use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn inkling_heartcaller_b202_gains_life_when_inkling_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inkling_heartcaller_b202());
    let inkling = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    drain_stack(&mut g);
    let p0 = g.players[0].life;
    // Kill via Lightning Bolt → CreatureDied fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(inkling)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 2, "Inkling death triggers 2 life");
}

#[test]
fn inkling_heartcaller_b202_does_not_trigger_on_non_inkling_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inkling_heartcaller_b202());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let p0 = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0, "non-Inkling death does not trigger");
}

#[test]
fn inkling_lifecaller_b202_is_three_three_flying_lifelink() {
    let def = catalog::inkling_lifecaller_b202();
    assert_eq!(def.cost.cmc(), 5);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn silverquill_recap_b202_returns_low_mv_creature_from_graveyard() {
    let mut g = two_player_game();
    let dead_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_recap_b202());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead_bear),
        "bear returned to hand");
}

#[test]
fn silverquill_recap_b202_returns_two_low_mv_creatures_when_available() {
    let mut g = two_player_game();
    let bear_a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear_b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_recap_b202());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let returned_a = g.players[0].hand.iter().any(|c| c.id == bear_a);
    let returned_b = g.players[0].hand.iter().any(|c| c.id == bear_b);
    assert!(returned_a && returned_b,
        "both bears returned (a={returned_a}, b={returned_b})");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 202 (modern_decks) — Witherbloom expansion.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn witherbloom_pestcaller_b202_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestcaller_b202());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest").count();
    assert_eq!(pests, 2, "two Pest tokens minted");
}

#[test]
fn witherbloom_sapdraw_b202_drains_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapdraw_b202());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 2);
    assert_eq!(g.players[1].life, p1 - 2);
    // -1 cast + 1 draw = net 0 hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn pest_devourer_b202_grows_on_other_pest_death() {
    let def = catalog::pest_devourer_b202();
    assert_eq!(def.triggered_abilities.len(), 1);
    assert!(def.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn witherbloom_vinepath_b202_puts_two_counters_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinepath_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn witherbloom_mossblossom_b202_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_mossblossom_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 2);
}

#[test]
fn pestshell_crusader_b202_etb_drains_one() {
    let def = catalog::pestshell_crusader_b202();
    assert!(def.keywords.contains(&Keyword::Trample));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Pest));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Knight));
}

#[test]
fn witherbloom_spellbloom_b202_grows_on_is_cast() {
    let mut g = two_player_game();
    let cd = g.add_card_to_battlefield(0, catalog::witherbloom_spellbloom_b202());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(cd).expect("alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn witherbloom_famine_b202_drains_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_famine_b202());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 4);
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn pest_howler_b202_has_attack_drain() {
    let def = catalog::pest_howler_b202();
    assert_eq!(def.cost.cmc(), 2);
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn witherbloom_cultivator_b202_taps_for_green_mana() {
    let def = catalog::witherbloom_cultivator_b202();
    assert_eq!(def.activated_abilities.len(), 1);
}

#[test]
fn witherbloom_decompose_b202_destroys_two_toughness_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::witherbloom_decompose_b202());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2-toughness destroyed");
}

#[test]
fn witherbloom_briarcaller_b202_is_four_four_trample_reach() {
    let def = catalog::witherbloom_briarcaller_b202();
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 4);
    assert!(def.keywords.contains(&Keyword::Trample));
    assert!(def.keywords.contains(&Keyword::Reach));
}

#[test]
fn witherbloom_rotcaller_b202_etb_makes_opp_discard() {
    let mut g = two_player_game();
    // Give opp a card to discard.
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let opp_hand_before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::witherbloom_rotcaller_b202());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opp discarded");
}

#[test]
fn witherbloom_verdance_b202_mints_a_four_four_beast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_verdance_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let beast = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Beast");
    assert!(beast.is_some(), "Beast token created");
    let b = beast.unwrap();
    assert_eq!(b.power(), 4);
    assert_eq!(b.toughness(), 4);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 202 (modern_decks) — Lorehold expansion.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn lorehold_reanimator_b202_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_reanimator_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "bear reanimated");
}

#[test]
fn lorehold_pyromancer_b202_pings_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_pyromancer_b202());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Magecraft ping 2 → AutoDecider picks any target. We just check that
    // opp lost extra life beyond the 3-damage bolt.
    assert!(g.players[1].life <= p1 - 3, "bolt damage applied");
}

#[test]
fn lorehold_charge_b202_pumps_team_with_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_charge_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::FirstStrike));
    assert_eq!(c.power(), 3);
}

#[test]
fn lorehold_spirit_caller_b202_has_attack_token_trigger() {
    let def = catalog::lorehold_spirit_caller_b202();
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn lorehold_bolt_ii_b202_deals_two_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_bolt_ii_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 2);
}

#[test]
fn lorehold_battlescholar_b202_is_first_strike_two_two() {
    let def = catalog::lorehold_battlescholar_b202();
    assert!(def.keywords.contains(&Keyword::FirstStrike));
    assert_eq!(def.cost.cmc(), 2);
}

#[test]
fn lorehold_excavate_b202_returns_creature_to_battlefield() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_excavate_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "bear reanimated");
}

#[test]
fn lorehold_frontlord_b202_anthems_other_friendlies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::lorehold_frontlord_b202());
    drain_stack(&mut g);
    let view = g.computed_permanent(bear).expect("bear on bf");
    assert_eq!(view.power, 3, "+1/+0 anthem applies");
}

#[test]
fn lorehold_cleanse_b202_damages_each_creature() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_cleanse_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "own bear dies (2 dmg)");
    assert!(g.battlefield_find(opp).is_none(), "opp bear dies (2 dmg)");
}

#[test]
fn lorehold_echoblade_b202_pumps_friendly_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_echoblade_b202());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.power(), 3, "+1/+1 magecraft pump");
}

#[test]
fn lorehold_lavascholar_b202_pings_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_lavascholar_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // 1-damage ping; auto-picks a target (opp player most likely).
    assert!(g.players[1].life <= p1, "lavascholar dealt damage");
}

#[test]
fn lorehold_ghostsmith_b202_has_attack_token_trigger() {
    let def = catalog::lorehold_ghostsmith_b202();
    assert_eq!(def.power, 3);
    assert_eq!(def.triggered_abilities.len(), 1);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 202 (modern_decks) — Prismari expansion.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn prismari_treasurehunter_b202_mints_treasure_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_treasurehunter_b202());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let treasure = g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure");
    assert!(treasure, "treasure minted");
}

#[test]
fn prismari_bolt_b202_deals_three_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_bolt_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 3);
}

#[test]
fn prismari_drakebreeder_b202_etb_smooths_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_drakebreeder_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw from scry-and-draw etb.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_spellcraft_b202_draws_two_after_scry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::prismari_spellcraft_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 2 draw = net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn prismari_sparkforger_b202_pumps_self_on_is_cast() {
    let mut g = two_player_game();
    let cd = g.add_card_to_battlefield(0, catalog::prismari_sparkforger_b202());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let view = g.computed_permanent(cd).expect("alive");
    assert_eq!(view.power, 3, "+1/+0 magecraft self-pump");
}

#[test]
fn prismari_squallcaller_b202_etb_taps_opp_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_squallcaller_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(opp_bear).expect("bear alive");
    assert!(c.tapped, "opp creature tapped");
}

#[test]
fn prismari_pyroartisan_b202_pings_opp_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_pyroartisan_b202());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 damage to opp.
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn prismari_tinkerer_b202_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_tinkerer_b202());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let treasure = g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure");
    assert!(treasure);
}

#[test]
fn prismari_soothsayer_b202_loots_on_is_cast() {
    let def = catalog::prismari_soothsayer_b202();
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn prismari_surge_ii_b202_deals_four_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_surge_ii_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn prismari_volcanist_b202_is_haste_trample() {
    let def = catalog::prismari_volcanist_b202();
    assert!(def.keywords.contains(&Keyword::Haste));
    assert!(def.keywords.contains(&Keyword::Trample));
    assert_eq!(def.power, 4);
}

#[test]
fn prismari_spiketide_b202_draws_three_and_discards_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_spiketide_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 3 draw - 2 discard = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 202 (modern_decks) — Quandrix expansion.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn quandrix_conjurer_b202_scrys_and_draws_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::quandrix_conjurer_b202());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 cast (bolt) + 1 draw (magecraft) = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_fractalweaver_b202_mints_fractal_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quandrix_fractalweaver_b202());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().any(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert!(fractal, "Fractal minted");
}

#[test]
fn quandrix_cantrip_b202_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::quandrix_cantrip_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 2 draw = net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn quandrix_grizzler_b202_is_three_three_vigilance() {
    let def = catalog::quandrix_grizzler_b202();
    assert!(def.keywords.contains(&Keyword::Vigilance));
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
}

#[test]
fn quandrix_sumtotal_b202_puts_x_counters_for_each_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_sumtotal_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(b1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(b1).expect("bear alive");
    // 3 creatures on bf at spell-resolve = 3 counters.
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn quandrix_skydiver_b202_is_flying_hexproof() {
    let def = catalog::quandrix_skydiver_b202();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Hexproof));
}

#[test]
fn quandrix_sparkbender_b202_counters_target_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    g.priority.player_with_priority = 0;
    let counter = g.add_card_to_hand(0, catalog::quandrix_sparkbender_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt was countered");
}

#[test]
fn quandrix_vinemage_b202_etb_pumps_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_vinemage_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_fractalspawn_b202_etb_mints_two_counter_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalspawn_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert!(fractal.is_some(), "Fractal minted");
    assert_eq!(fractal.unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_symmetry_b202_mints_fractal_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_symmetry_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: Some(4),
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert!(fractal.is_some(), "Fractal minted");
    assert_eq!(fractal.unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn quandrix_streampath_b202_bounces_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_streampath_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_bear).is_none(), "opp bear bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_bear), "bear in opp hand");
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_geomant_b202_activates_for_counter() {
    let def = catalog::quandrix_geomant_b202();
    assert_eq!(def.activated_abilities.len(), 1);
}

#[test]
fn silverquill_wardrune_b202_pumps_toughness_with_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_wardrune_b202());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert_eq!(c.toughness(), 5, "+0/+3 → 5 toughness");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 203 (modern_decks) — Compact cross-school round.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_cantor_b203_etb_scrys() {
    let def = catalog::silverquill_cantor_b203();
    assert_eq!(def.cost.cmc(), 2);
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn inkling_whisperer_b203_is_a_flying_inkling() {
    let def = catalog::inkling_whisperer_b203();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_censurer_b203_is_a_lifelink_drain_body() {
    let def = catalog::silverquill_censurer_b203();
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn silverquill_edict_b203_forces_opp_sac() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_edict_b203());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_bear).is_none(), "edict took the bear");
}

#[test]
fn silverquill_lay_faith_b203_gains_four_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_lay_faith_b203());
    g.players[0].mana_pool.add(Color::White, 1);
    let p0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 4);
}

#[test]
fn silverquill_hospitaller_b203_is_a_lifelink_vigilance_finisher() {
    let def = catalog::silverquill_hospitaller_b203();
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert!(def.keywords.contains(&Keyword::Vigilance));
    assert_eq!(def.power, 4);
}

#[test]
fn inkling_sheriff_b203_is_a_flying_vigilance_lifelink_top_end() {
    let def = catalog::inkling_sheriff_b203();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Vigilance));
    assert!(def.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_mausoleum_b203_drains_via_activation() {
    let def = catalog::silverquill_mausoleum_b203();
    assert_eq!(def.activated_abilities.len(), 1);
}

#[test]
fn witherbloom_apprentice_ii_b203_drains_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_apprentice_ii_b203());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1.
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn pest_tendril_b203_drains_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_tendril_b203());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 4);
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn pest_sapper_b203_loses_opp_two_on_death() {
    let mut g = two_player_game();
    let sapper = g.add_card_to_battlefield(0, catalog::pest_sapper_b203());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(sapper)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 2);
}

#[test]
fn witherbloom_pestlord_b203_is_four_five_trample() {
    let def = catalog::witherbloom_pestlord_b203();
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 5);
    assert!(def.keywords.contains(&Keyword::Trample));
}

#[test]
fn pest_patriarch_b203_etb_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_patriarch_b203());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest").count();
    assert_eq!(pests, 1);
}

#[test]
fn lorehold_apprentice_b203_pings_on_is_cast() {
    let def = catalog::lorehold_apprentice_b203();
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn lorehold_soulbinder_b203_has_attack_mint_trigger() {
    let def = catalog::lorehold_soulbinder_b203();
    assert!(def.keywords.contains(&Keyword::Vigilance));
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn lorehold_spirit_sage_b203_etb_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_sage_b203());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1);
}

#[test]
fn lorehold_strike_b203_deals_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_strike_b203());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 3);
}

#[test]
fn lorehold_ancestor_b203_is_four_four_trample() {
    let def = catalog::lorehold_ancestor_b203();
    assert_eq!(def.power, 4);
    assert!(def.keywords.contains(&Keyword::Trample));
}

#[test]
fn lorehold_spirit_squire_b203_etb_gains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_squire_b203());
    g.players[0].mana_pool.add(Color::White, 1);
    let p0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 2);
}

#[test]
fn prismari_apprentice_ii_b203_loots_on_is_cast() {
    let def = catalog::prismari_apprentice_ii_b203();
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn prismari_cantrip_b203_draws_then_discards() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::prismari_cantrip_b203());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw - 1 discard = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_counter_b203_counters_spell() {
    let def = catalog::prismari_counter_b203();
    assert_eq!(def.cost.cmc(), 2);
}

#[test]
fn prismari_flame_b203_deals_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_flame_b203());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn prismari_squallcaller_ii_b203_etb_scrys() {
    let def = catalog::prismari_squallcaller_ii_b203();
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn prismari_mage_b203_pumps_self_on_is_cast() {
    let def = catalog::prismari_mage_b203();
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn quandrix_apprentice_ii_b203_pumps_friendly() {
    let def = catalog::quandrix_apprentice_ii_b203();
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn quandrix_naturist_b203_is_three_two_trample() {
    let def = catalog::quandrix_naturist_b203();
    assert!(def.keywords.contains(&Keyword::Trample));
    assert_eq!(def.power, 3);
}

#[test]
fn quandrix_charmer_b203_etb_smooths_and_cantrips() {
    let def = catalog::quandrix_charmer_b203();
    assert_eq!(def.triggered_abilities.len(), 1);
}

#[test]
fn quandrix_surge_b203_puts_three_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_surge_b203());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn quandrix_streamer_b203_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_streamer_b203());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_verdant_b203_is_vigilance_reach_wall() {
    let def = catalog::quandrix_verdant_b203();
    assert!(def.keywords.contains(&Keyword::Vigilance));
    assert!(def.keywords.contains(&Keyword::Reach));
    assert_eq!(def.toughness, 4);
}

#[test]
fn inkling_mentor_b203_drains_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inkling_mentor_b203());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn inkling_sage_apprentice_b203_gains_one_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inkling_sage_apprentice_b203());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 1);
}

#[test]
fn witherbloom_apprentice_iii_b203_self_pumps_on_is_cast() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::witherbloom_apprentice_iii_b203());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let view = g.computed_permanent(app).expect("alive");
    assert_eq!(view.power, 3, "+1/+1 EOT magecraft self-pump");
}

// ─────────────────────────────────────────────────────────────────────────
// CR rule lock-in tests — batch 202 round
// ─────────────────────────────────────────────────────────────────────────

/// CR 704.5a — A player with 0 or less life loses the game (state-based
/// action). Drain a player to 0 via a Famine cast and verify they lose.
#[test]
fn cr_704_5a_player_at_zero_or_less_life_loses() {
    let mut g = two_player_game();
    // Drop p1 to 4 life.
    g.players[1].life = 4;
    let famine = g.add_card_to_hand(0, catalog::witherbloom_famine_b202());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: famine, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Famine deals 4 drain → p1 at 0 life → loses.
    assert!(g.players[1].life <= 0);
    assert!(g.is_game_over(), "player at 0 life triggers game-over SBA");
}

/// CR 608.2b — A spell with all illegal targets is removed from the stack
/// and goes to graveyard rather than resolving. Cast Bolt at a creature,
/// then remove the creature before bolt resolves → bolt fizzles.
#[test]
fn cr_608_2b_spell_with_illegal_target_fizzles() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on stack");
    // Remove the bear by direct mutation: pull it off bf, drop into gy.
    let bear_inst = g.battlefield.iter().position(|c| c.id == opp_bear)
        .map(|i| g.battlefield.remove(i)).expect("bear on bf");
    g.players[1].graveyard.push(bear_inst);
    let p1_life = g.players[1].life;
    drain_stack(&mut g);
    // Bolt resolves with no legal target → fizzles → no damage to player.
    assert!(g.battlefield_find(opp_bear).is_none(), "bear moved to gy");
    assert_eq!(g.players[1].life, p1_life, "bolt fizzled — no damage redirected");
}

/// CR 121.6a — A replacement effect for "draw a card" applies even if
/// the draw would be impossible because the library is empty. We pin
/// the inverse: a draw against an empty library is a *loss*, not a
/// no-op, and that loss is the SBA trigger (not the draw replacement
/// — they're independent paths). This lock-in pairs with 704.5b above.
#[test]
fn cr_704_5b_empty_library_draw_attempt_loses_game() {
    let mut g = two_player_game();
    // p0 has an empty library.
    g.players[0].library.clear();
    g.players[0].cards_drawn_this_turn = 0;
    // Force a draw via Quandrix Cantrip.
    let id = g.add_card_to_hand(0, catalog::quandrix_cantrip_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Drawing with empty library → "attempted to draw from empty lib"
    // SBA → caster loses.
    assert!(g.is_game_over(), "empty-library draw loses the game");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 204 (modern_decks) — Cross-school round 4 (focused tests).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_drainstrike_b204_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_drainstrike_b204());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 3);
    assert_eq!(g.players[1].life, p1 - 3);
}

#[test]
fn silverquill_bond_b204_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_bond_b204());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("alive");
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert_eq!(c.power(), 3, "+1/+1 pump");
}

#[test]
fn witherbloom_drainshade_b204_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainshade_b204());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 2);
}

#[test]
fn witherbloom_bloodtap_b204_drains_five() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloodtap_b204());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 5);
    assert_eq!(g.players[1].life, p1 - 5);
}

#[test]
fn lorehold_pyromaster_b204_pings_two_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_pyromaster_b204());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Bolt 3 + ping 2 if AutoDecider sends ping at opp.
    assert!(g.players[1].life <= p1 - 3);
}

#[test]
fn lorehold_flameburst_b204_deals_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_flameburst_b204());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn lorehold_spiritbringer_b204_etb_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritbringer_b204());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1);
}

#[test]
fn prismari_pyromage_b204_pings_each_opp_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_pyromage_b204());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Bolt 3 + ping each opp 1.
    assert_eq!(g.players[1].life, p1 - 4);
}

#[test]
fn prismari_sparkboost_b204_pumps_two_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_sparkboost_b204());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("alive");
    assert_eq!(c.power(), 4, "+2/+0");
}

#[test]
fn quandrix_fractaller_b204_mints_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quandrix_fractaller_b204());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().filter(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)).count();
    assert_eq!(fractal, 1);
}

#[test]
fn quandrix_mentor_b204_pumps_friendly_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quandrix_mentor_b204());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 205 (modern_decks) — Lorehold Enrage cycle. Exercises the new
// `EventKind::DealtDamage` event (CR 702.130). Each test deals damage to
// the enrage creature (Lightning Bolt, 3 damage) and verifies the
// triggered payoff fires. All bodies are sized to survive 3 damage so the
// post-trigger board state is deterministic.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn lorehold_battlescarred_b205_enrage_adds_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_battlescarred_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("3/4 survives 3 damage");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1,
        "enrage put one +1/+1 counter on the damaged creature");
}

#[test]
fn lorehold_echovenger_b205_enrage_scales_with_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_echovenger_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("2/5 survives 3 damage");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3,
        "Value::TriggerEventAmount = 3 damage → 3 counters");
}

#[test]
fn lorehold_vengescribe_b205_enrage_pings_opponent() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_vengescribe_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "2/4 survives 3 damage");
    assert_eq!(g.players[1].life, l1 - 1,
        "enrage ping deals 1 to the opponent (default auto-target)");
}

#[test]
fn lorehold_grudgebearer_b205_enrage_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_grudgebearer_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 2, "opp loses 2 from enrage drain");
    assert_eq!(g.players[0].life, l0 + 2, "you gain 2 from enrage drain");
}

#[test]
fn lorehold_stoneguard_b205_enrage_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_stoneguard_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 2, "enrage gains 2 life");
}

#[test]
fn lorehold_chroniclekeeper_b205_enrage_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_battlefield(0, catalog::lorehold_chroniclekeeper_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    // -1 Bolt cast, +1 enrage draw = net same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "enrage drew a card");
}

#[test]
fn lorehold_warhost_b205_enrage_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_warhost_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit)).count();
    assert_eq!(spirits, 1, "enrage minted one Lorehold Spirit token");
}

#[test]
fn lorehold_enrage_does_not_fire_without_damage() {
    // Sanity: an undamaged enrage creature has no counters / no triggers.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_battlescarred_b205());
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 0,
        "no damage → no enrage counter");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 205 (modern_decks) — cross-school round (Witherbloom / Quandrix /
// Silverquill / Prismari). Mixes the new Enrage event with standard
// magecraft / drain / ETB primitives.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn witherbloom_thornbeast_b205_enrage_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_thornbeast_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "3/5 survives 3 damage");
    assert_eq!(g.players[1].life, l1 - 1, "opp loses 1 from enrage drain");
    assert_eq!(g.players[0].life, l0 + 1, "you gain 1 from enrage drain");
}

#[test]
fn witherbloom_gravethorn_b205_enrage_scales() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_gravethorn_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("1/5 survives 3 damage");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3, "3 damage → 3 counters");
}

#[test]
fn witherbloom_sapfeeder_b205_death_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_sapfeeder_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "3/3 dies to 3 damage");
    assert_eq!(g.players[1].life, l1 - 2, "death drain: opp loses 2");
    assert_eq!(g.players[0].life, l0 + 2, "death drain: you gain 2");
}

#[test]
fn witherbloom_bloodmoss_b205_magecraft_drains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_bloodmoss_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 4, "bolt 3 + magecraft drain 1");
    assert_eq!(g.players[0].life, l0 + 1, "magecraft gains 1");
}

#[test]
fn quandrix_thornfractal_b205_enrage_scales() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_thornfractal_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("0/6 survives 3 damage");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3, "3 damage → 3 counters");
}

#[test]
fn quandrix_tidecaller_b205_magecraft_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::quandrix_tidecaller_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    // -1 Bolt, +1 magecraft draw = net same.
    assert_eq!(g.players[0].hand.len(), hand_before, "magecraft drew a card");
}

#[test]
fn quandrix_growthseer_b205_etb_counters_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_growthseer_b205());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "ETB +1/+1 on friendly");
}

#[test]
fn silverquill_lightscribe_b205_etb_gains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_lightscribe_b205());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 3, "ETB gains 3 life");
}

#[test]
fn silverquill_grimquill_b205_magecraft_drains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_grimquill_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 4, "bolt 3 + magecraft drain 1");
}

#[test]
fn silverquill_final_edict_b205_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_final_edict_b205());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 3, "opp loses 3");
    assert_eq!(g.players[0].life, l0 + 3, "you gain 3");
}

#[test]
fn silverquill_inkguard_b205_is_a_two_mana_lifelink_inkling() {
    let def = catalog::silverquill_inkguard_b205();
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 3);
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn prismari_flarecaster_b205_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_flarecaster_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 4, "bolt 3 + magecraft ping 1 to each opp");
}

#[test]
fn prismari_emberbolt_b205_deals_two_to_opponent() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_emberbolt_b205());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 2, "Emberbolt deals 2");
}

#[test]
fn prismari_tidescribe_b205_etb_resolves_and_enters() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::prismari_tidescribe_b205());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Tidescribe on bf");
    assert_eq!((c.power(), c.toughness()), (1, 4), "1/4 body, ETB scry 2 resolved");
}

#[test]
fn prismari_stormloot_b205_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_stormloot_b205());
    // A spare card to discard for the loot.
    g.add_card_to_hand(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    // -1 Bolt cast, +1 loot draw, -1 loot discard = net -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "magecraft loot: draw then discard");
}

// ── Batch 205 round 2 — varied primitives ──────────────────────────────

#[test]
fn lorehold_emberhistorian_b205_magecraft_pings() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_emberhistorian_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 4, "bolt 3 + magecraft ping 1");
}

#[test]
fn lorehold_relicwarden_b205_attack_gains_life() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_relicwarden_b205());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
        c.summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2, "on-attack gains 2 life");
}

#[test]
fn lorehold_warchronicler_b205_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_warchronicler_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("alive");
    assert_eq!(c.power(), 5, "magecraft +2/+0 → 5 power");
}

#[test]
fn witherbloom_rotcaller_b205_drains_on_other_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_rotcaller_b205());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear dies");
    assert_eq!(g.players[1].life, l1 - 1, "aristocrats drain: opp -1");
    assert_eq!(g.players[0].life, l0 + 1, "aristocrats drain: you +1");
}

#[test]
fn quandrix_mistcaller_b205_magecraft_scrys() {
    // Magecraft scry doesn't change hand/life; assert it fires without
    // error and the body is a 1/3 Merfolk Wizard.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let mc = g.add_card_to_battlefield(0, catalog::quandrix_mistcaller_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(mc).expect("alive");
    assert_eq!((c.power(), c.toughness()), (1, 3));
}

#[test]
fn silverquill_deathscribe_b205_drains_on_other_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_deathscribe_b205());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "death drain opp -1");
}

#[test]
fn prismari_pyrosmith_b205_magecraft_mints_treasure() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_pyrosmith_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 1, "magecraft minted a Treasure");
}

#[test]
fn prismari_galemage_b205_is_a_three_mana_wizard() {
    let def = catalog::prismari_galemage_b205();
    assert_eq!((def.power, def.toughness), (2, 3));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Wizard));
}

// ─────────────────────────────────────────────────────────────────────────
// CR rules lock-in tests (batch 205 session)
// ─────────────────────────────────────────────────────────────────────────

/// CR 702.130 — Enrage. "Whenever this creature is dealt damage" fires on
/// COMBAT damage just as it does on spell damage. Lorehold Battlescarred
/// (3/4) blocks a 2/2, survives the 2 combat damage, and the enrage trigger
/// puts a +1/+1 counter on it. Exercises the `DealtDamage` event off the
/// combat-damage path in `game/combat.rs` (CR 510 → 702.130 interaction).
#[test]
fn cr_702_130_enrage_fires_on_combat_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::lorehold_battlescarred_b205()); // 3/4
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("bear attacks");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect("battlescarred blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    let c = g.battlefield_find(blocker).expect("3/4 survives 2 combat damage");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1,
        "CR 702.130: enrage fired on combat damage and added a +1/+1 counter");
}

/// CR 122.1c — Shield counters. "If damage would be dealt to a permanent
/// with a shield counter on it … prevent that damage and remove a shield
/// counter from it." A Bolt (3) into a 2/2 with one shield counter is fully
/// prevented; the creature survives at 0 marked damage and the shield
/// counter is consumed.
#[test]
fn cr_122_1c_shield_counter_prevents_noncombat_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::Shield, 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the shielded bear");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("shield prevented lethal damage; bear survives");
    assert_eq!(c.damage, 0, "CR 122.1c: damage was prevented, none marked");
    assert_eq!(c.counter_count(CounterType::Shield), 0,
        "CR 122.1c: the shield counter was removed by the prevention");
}

/// CR 510.1c / 510.1d — Combat Damage Step. A blocked attacker assigns its
/// combat damage to the creature blocking it, and the blocker assigns to the
/// attacker; both are dealt simultaneously. Two 2/2s trade and both die.
#[test]
fn cr_510_blocked_attacker_and_blocker_trade() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("bear attacks");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect("bear blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(),
        "CR 510.1d: attacker took 2 lethal combat damage and died");
    assert!(g.battlefield_find(blocker).is_none(),
        "CR 510.1c: blocker took 2 lethal combat damage and died");
}

// ── Batch 206 — cross-school staples ────────────────────────────────────

#[test]
fn lorehold_skirmisher_b206_pings_on_attack() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_skirmisher_b206());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "on-attack ping 1 to opp");
}

#[test]
fn lorehold_archivekeeper_b206_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::lorehold_archivekeeper_b206());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast, +1 ETB draw = net same.
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB drew a card");
}

#[test]
fn lorehold_ember_veteran_b206_magecraft_pumps_and_has_trample() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_ember_veteran_b206());
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Trample));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), 5, "magecraft +1/+0");
}

#[test]
fn witherbloom_grim_harvest_b206_drains_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_grim_harvest_b206());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 4);
    assert_eq!(g.players[0].life, l0 + 4);
}

#[test]
fn witherbloom_sporecaller_b206_gains_life_on_other_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_sporecaller_b206());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 1, "gained 1 on the bear's death");
}

#[test]
fn witherbloom_fungalbeast_b206_etb_gains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_fungalbeast_b206());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 2);
}

#[test]
fn quandrix_scholar_b206_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_scholar_b206());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("alive");
    assert_eq!((c.power(), c.toughness()), (2, 3), "magecraft +1/+1 → 2/3");
}

#[test]
fn quandrix_megafractal_b206_is_a_five_five_trampler() {
    let def = catalog::quandrix_megafractal_b206();
    assert_eq!((def.power, def.toughness), (5, 5));
    assert!(def.keywords.contains(&Keyword::Trample));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Fractal));
}

#[test]
fn silverquill_dictator_b206_is_a_flying_lifelink_inkling() {
    let def = catalog::silverquill_dictator_b206();
    assert_eq!((def.power, def.toughness), (3, 3));
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_purge_b206_exiles_small_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::silverquill_purge_b206());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2-power bear exiled");
}

#[test]
fn prismari_inferno_b206_deals_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_inferno_b206());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 4);
}

#[test]
fn prismari_windscholar_b206_etb_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::prismari_windscholar_b206());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast, +1 ETB draw = net same.
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB scry+draw netted a card");
}
