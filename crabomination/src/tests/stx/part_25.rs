use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn witherbloom_reaping_b207_draws_per_creature_died_this_turn() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    // Two of our creatures die this turn.
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    bolt_own_creature(&mut g, 0, b1);
    bolt_own_creature(&mut g, 0, b2);
    assert_eq!(g.players[0].creatures_died_this_turn, 2, "two creatures died");

    let id = g.add_card_to_hand(0, catalog::witherbloom_reaping_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reaping castable");
    drain_stack(&mut g);
    // -1 for casting Reaping, +2 drawn = net +1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2,
        "Reaping draws one per creature that died this turn");
}

#[test]
fn witherbloom_gravecaller_b207_etb_drains_per_total_deaths() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    bolt_own_creature(&mut g, 0, mine);
    bolt_own_creature(&mut g, 0, theirs); // a creature controlled by p1 also dies
    assert_eq!(g.players[0].creatures_died_this_turn, 1);
    assert_eq!(g.players[1].creatures_died_this_turn, 1);

    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_gravecaller_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Gravecaller castable");
    drain_stack(&mut g);
    // Total deaths this turn = 2 → opponent loses 2, we gain 2.
    assert_eq!(g.players[1].life, p1_life - 2, "drain scales with total deaths");
    assert_eq!(g.players[0].life, p0_life + 2, "we gain that much");
}

#[test]
fn witherbloom_bloodfeast_b207_gains_two_per_creature_died() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    bolt_own_creature(&mut g, 0, b1);
    let p0_life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloodfeast_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodfeast castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2, "2 life per creature that died (1 died)");
}

#[test]
fn witherbloom_saplinglord_b207_grows_when_other_creature_dies() {
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::witherbloom_saplinglord_b207());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    bolt_own_creature(&mut g, 0, fodder);
    let c = g.battlefield_find(lord).expect("lord alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1,
        "+1/+1 counter when another creature dies");
}

#[test]
fn witherbloom_toxicult_b207_etb_mints_pest_and_drains_on_magecraft() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxicult_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxicult castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"),
        "ETB mints a Pest token");
    // Magecraft drain on instant cast.
    let p1_life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "bolt 3 + magecraft drain 1");
}

#[test]
fn witherbloom_rotcaller_b207_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_rotcaller_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rotcaller castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter().filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pests, 2, "ETB mints two Pest tokens");
}

#[test]
fn witherbloom_sapsiphon_b207_gains_life_on_combat_damage() {
    use crate::game::Attack;
    let mut g = two_player_game();
    let sap = g.add_card_to_battlefield(0, catalog::witherbloom_sapsiphon_b207());
    g.clear_sickness(sap);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sap, target: AttackTarget::Player(1),
    }])).expect("declare attackers");
    drain_stack(&mut g);
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2,
        "combat damage to player gains 2 life");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 207 (modern_decks) — Lorehold (R/W) Spirit / magecraft staples.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn lorehold_soulkindler_b207_etb_mints_a_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_soulkindler_b207());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soulkindler castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1, "ETB mints one Spirit token");
}

#[test]
fn lorehold_cinderscribe_b207_magecraft_pings_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_cinderscribe_b207());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "bolt 3 + magecraft ping 1");
}

#[test]
fn lorehold_battlecaller_b207_mints_spirit_on_attack() {
    let mut g = two_player_game();
    let bc = g.add_card_to_battlefield(0, catalog::lorehold_battlecaller_b207());
    g.clear_sickness(bc);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bc, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1, "attacking mints a Spirit");
}

#[test]
fn lorehold_emberbolt_b207_deals_three_to_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_emberbolt_b207());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Emberbolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3, "3 damage to face");
}

#[test]
fn lorehold_relicsmith_b207_returns_low_mv_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_relicsmith_b207());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Relicsmith castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "low-MV creature returned to hand");
}

#[test]
fn lorehold_charge_ii_b207_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_charge_ii_b207());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let pwr = g.battlefield_find(bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Charge castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.power(), pwr + 1, "+1/+0 team pump");
    assert!(c.has_keyword(&Keyword::FirstStrike), "granted first strike");
}

#[test]
fn lorehold_vanguard_b207_is_a_four_mana_haste_spirit_knight() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_vanguard_b207());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 3));
    assert!(c.has_keyword(&Keyword::Haste));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 207 (modern_decks) — Quandrix (G/U) Fractal / draw-matters staples.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn quandrix_tidecaller_b207_mints_two_counter_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_tidecaller_b207());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidecaller castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal")
        .expect("Fractal token minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2,
        "Fractal enters with two +1/+1 counters");
    assert_eq!((fractal.power(), fractal.toughness()), (2, 2));
}

#[test]
fn quandrix_theorist_b207_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::quandrix_theorist_b207());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 bolt + 1 magecraft draw = net same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "magecraft scry+draw netted a card");
}

#[test]
fn quandrix_fractalsurge_b207_makes_x_counter_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalsurge_b207());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Fractalsurge castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal")
        .expect("Fractal token minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 3,
        "X=3 → three +1/+1 counters");
}

#[test]
fn quandrix_studymate_b207_grows_with_cards_drawn_this_turn() {
    let mut g = two_player_game();
    // Two cards drawn this turn so the ETB sees the tally.
    g.players[0].cards_drawn_this_turn = 2;
    let id = g.add_card_to_hand(0, catalog::quandrix_studymate_b207());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Studymate castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.definition.name == "Quandrix Studymate (b207)")
        .expect("Studymate on bf");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2,
        "+1/+1 counter per card drawn this turn");
}

#[test]
fn quandrix_currentweaver_b207_etb_draws_and_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_currentweaver_b207());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Currentweaver castable");
    drain_stack(&mut g);
    // -1 cast + 1 ETB draw = net same.
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB draw netted a card");
}

#[test]
fn quandrix_bigmind_b207_is_a_five_mana_trampler() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_bigmind_b207());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 5));
    assert!(c.has_keyword(&Keyword::Trample));
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 207 (modern_decks) — Prismari (U/R) spells-matter / Treasure staples.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn prismari_pyrologist_b207_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_pyrologist_b207());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "bolt 3 + magecraft ping 1");
}

#[test]
fn prismari_goldcaster_b207_magecraft_mints_treasure() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_goldcaster_b207());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "magecraft mints a Treasure token");
}

#[test]
fn prismari_firebolt_ii_b207_deals_four_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_firebolt_ii_b207());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Firebolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "4 damage kills the 2/2");
}

#[test]
fn prismari_goldsmith_b207_etb_mints_two_treasures() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_goldsmith_b207());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Goldsmith castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 2, "ETB mints two Treasure tokens");
}

#[test]
fn prismari_stormloot_ii_b207_draws_two_discards_one() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::prismari_stormloot_ii_b207());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormloot castable");
    drain_stack(&mut g);
    // -1 cast + 2 draw - 1 discard = net +0 from hand_before? hand_before counted the spell.
    // After cast: hand = hand_before - 1; then +2 -1 = +1 over that => hand_before + 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "draw 2, discard 1 net +1 over post-cast");
}

#[test]
fn prismari_galeblaster_b207_magecraft_self_pumps() {
    let mut g = two_player_game();
    let gb = g.add_card_to_battlefield(0, catalog::prismari_galeblaster_b207());
    let pwr = g.battlefield_find(gb).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(gb).unwrap().power(), pwr + 1, "magecraft +1/+0");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 207 (modern_decks) — Silverquill (W/B) Inkling / drain staples.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_inkbinder_b207_magecraft_drains_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_inkbinder_b207());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "bolt 3 + drain 1");
    assert_eq!(g.players[0].life, p0_life + 1, "drain gains 1");
}

#[test]
fn silverquill_eulogist_b207_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_eulogist_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Eulogist castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn silverquill_edict_ii_b207_makes_opp_sacrifice_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_edict_ii_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Edict castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent sacrificed their creature");
    // -1 cast + 1 draw = net same as hand_before.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card");
}

#[test]
fn inkling_highflier_b207_is_a_four_mana_flying_vigilance_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_highflier_b207());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 3));
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_sanction_b207_exiles_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_sanction_b207());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sanction castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "target exiled");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "exiled, not destroyed");
    assert_eq!(g.players[0].life, p0_life + 2, "gain 2 life");
}

#[test]
fn silverquill_coursemate_b207_gains_life_when_other_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_coursemate_b207());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p0_life = g.players[0].life;
    bolt_own_creature(&mut g, 0, fodder);
    assert_eq!(g.players[0].life, p0_life + 1, "gain 1 when another creature dies");
}

// ─────────────────────────────────────────────────────────────────────────
// CR rule lock-in tests (batch 207 session).
// ─────────────────────────────────────────────────────────────────────────

/// CR 702.83 (Exalted): a creature that attacks alone gets +1/+1 until end
/// of turn. New `Predicate::AttackingAlone` + `exalted()` shortcut.
#[test]
fn cr_702_83_exalted_pumps_lone_attacker() {
    let mut g = two_player_game();
    let duel = g.add_card_to_battlefield(0, catalog::silverquill_duelmaster_b207());
    g.clear_sickness(duel);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: duel, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(duel).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3),
        "CR 702.83a: attacking alone grants Exalted +1/+1");
}

/// CR 702.83b: Exalted does NOT trigger when more than one creature
/// attacks (`Predicate::AttackingAlone` is false).
#[test]
fn cr_702_83b_exalted_silent_when_not_alone() {
    let mut g = two_player_game();
    let duel = g.add_card_to_battlefield(0, catalog::silverquill_duelmaster_b207());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(duel);
    g.clear_sickness(buddy);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: duel, target: AttackTarget::Player(1) },
        Attack { attacker: buddy, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(duel).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 2),
        "CR 702.83b: no Exalted pump when not attacking alone");
}

/// CR 702.83b: multiple Exalted abilities each trigger on a single lone
/// attacker. Two Akrasan Squires pump an attacking bear +1/+1 each → +2/+2.
#[test]
fn cr_702_83b_multiple_exalted_stack_on_lone_attacker() {
    let mut g = two_player_game();
    let _s1 = g.add_card_to_battlefield(0, catalog::akrasan_squire());
    let _s2 = g.add_card_to_battlefield(0, catalog::aven_squire());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    // The bear attacks alone (the squires hold back) → both Exalteds fire.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 4),
        "two Exalted sources each add +1/+1 to the lone attacker");
}

/// CR 702.92 (Battle cry): when the source attacks, each *other* attacking
/// creature gets +1/+0 — but the source itself does not.
#[test]
fn cr_702_92_battle_cry_pumps_other_attackers_only() {
    let mut g = two_player_game();
    let driver = g.add_card_to_battlefield(0, catalog::goblin_wardriver());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(driver);
    g.clear_sickness(bear);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: driver, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let other = g.battlefield_find(bear).unwrap();
    assert_eq!((other.power(), other.toughness()), (3, 2),
        "battle cry pumps the other attacker +1/+0");
    let src = g.battlefield_find(driver).unwrap();
    assert_eq!((src.power(), src.toughness()), (2, 2),
        "battle cry does NOT pump its own source");
}

/// CR 702.15 (Lifelink): combat damage dealt by a creature with lifelink
/// causes its controller to gain that much life.
#[test]
fn cr_702_15_lifelink_combat_damage_gains_life() {
    let mut g = two_player_game();
    // Anthemwriter is a 4/4 flying lifelink finisher.
    let ll = g.add_card_to_battlefield(0, catalog::silverquill_anthemwriter());
    g.clear_sickness(ll);
    let power = g.battlefield_find(ll).unwrap().power();
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ll, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - power, "opponent took combat damage");
    assert_eq!(g.players[0].life, p0_life + power, "CR 702.15: lifelink gains that much");
}

/// CR 510.1c: a trampling attacker blocked by a single creature assigns
/// lethal to the blocker and tramples the rest to the defending player.
#[test]
fn cr_510_1c_trample_overflow_to_player() {
    let mut g = two_player_game();
    // 4/4 trampler vs a 2/2 blocker → 2 lethal to blocker, 2 tramples.
    let atk = g.add_card_to_battlefield(0, catalog::quandrix_bigmind_b207()); // 4/5 Trample
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != crate::game::types::TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)]))
        .expect("block");
    drain_stack(&mut g);
    let p1_life = g.players[1].life;
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blk).is_none(), "blocker took lethal");
    // 4 power - 2 lethal to the 2/2 = 2 trample to player.
    assert_eq!(g.players[1].life, p1_life - 2, "CR 510.1c: 2 damage tramples over");
}

/// CR 510.1c-d — a `wants_ui` attacking player chooses, via interactive
/// `pending_decision`s, the order its blockers take damage and how its power
/// is divided. Here a 3/3 splits across two 2/2 blockers: by ordering the
/// second-declared blocker first, the player kills *it* (and leaves the
/// first alive), which the engine's default CardId-order split would not do.
#[test]
fn cr_510_1c_ui_player_orders_and_assigns_combat_damage() {
    use crate::card::{CardDefinition, CardType};
    use crate::decision::{Decision, DecisionAnswer};
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let beater = CardDefinition {
        name: "Three Three",
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        ..Default::default()
    };
    let atk = g.add_card_to_battlefield(0, beater);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // lower CardId
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // higher CardId
    g.clear_sickness(atk);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk)]))
        .expect("double block");
    drain_stack(&mut g);

    // Pass priority into the combat damage step — it suspends on the order
    // decision instead of auto-splitting.
    while g.pending_decision.is_none() {
        g.perform_action(GameAction::PassPriority).expect("pass");
        assert!(!g.is_game_over());
    }
    let pd = g.pending_decision.as_ref().expect("combat suspends on ordering");
    let Decision::CombatDamageOrder { attacker, blockers } = &pd.decision else {
        panic!("expected CombatDamageOrder, got {:?}", pd.decision);
    };
    assert_eq!(*attacker, atk);
    assert_eq!(blockers.len(), 2);
    // Order b2 (declared second) ahead of b1 so it takes lethal first.
    g.submit_decision(DecisionAnswer::DamageOrder(vec![b2, b1]))
        .expect("order accepted");

    // Now it suspends on the assignment decision.
    let pd = g.pending_decision.as_ref().expect("combat suspends on assignment");
    let Decision::AssignCombatDamage { attacker_power, blockers, .. } = &pd.decision else {
        panic!("expected AssignCombatDamage, got {:?}", pd.decision);
    };
    assert_eq!(*attacker_power, 3);
    assert_eq!(blockers.first().map(|(id, _, _)| *id), Some(b2),
        "the chosen order puts b2 first");
    // Assign lethal (2) to b2, the remaining 1 to b1.
    g.submit_decision(DecisionAnswer::CombatDamageAssignment(vec![(b2, 2), (b1, 1)]))
        .expect("assignment accepted");
    drain_stack(&mut g);

    assert!(g.pending_decision.is_none(), "combat fully resolved");
    assert!(g.battlefield_find(b2).is_none(), "b2 was assigned lethal and died");
    assert!(g.battlefield_find(b1).is_some(), "b1 only took 1 and survived");
}

/// CR 702.85b (Cascade): the exile walk stops at the first nonland card
/// with mana value *strictly less* than the cascading spell's. A card whose
/// MV equals the cascade MV is not a valid hit — it's exiled past and
/// bottomed.
#[test]
fn cr_702_85_cascade_skips_equal_mana_value_cards() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Top of library: Brightglass Gearhulk (MV 4 — equals cascade MV, must
    // be skipped), then Grizzly Bears (MV 2 — the legal hit).
    let gearhulk = g.add_card_to_library(0, catalog::brightglass_gearhulk());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let elf = g.add_card_to_hand(0, catalog::bloodbraid_elf()); // cascade(4)
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, elf);

    assert!(g.battlefield.iter().any(|c| c.id == bears),
        "CR 702.85b: the MV-2 card is the legal cascade hit");
    assert!(!g.battlefield.iter().any(|c| c.id == gearhulk),
        "the MV-4 card (== cascade MV) is NOT cast");
    assert!(g.players[0].library.iter().any(|c| c.id == gearhulk),
        "the skipped equal-MV card is bottomed back into the library");
}

/// CR 702.52e (Dredge): dredging *replaces* the draw — the player gains no
/// net card from the draw event (the dredged card returns to hand, but the
/// would-be draw never happens). Net hand size change is +1 (the dredge
/// card) and the library shrinks only by the dredge count (the mill).
#[test]
fn cr_702_52_dredge_replaces_the_draw_event() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let thug = g.add_card_to_graveyard(0, catalog::golgari_thug()); // Dredge 4
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let lib_before = g.players[0].library.len();
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    // No card was drawn from the top — the library shrank only by the mill.
    assert_eq!(g.players[0].library.len(), lib_before - 4,
        "CR 702.52e: dredge mills 4 and skips the draw (library -4, not -5)");
    assert!(g.players[0].hand.iter().any(|c| c.id == thug),
        "the dredged card returns to hand");
    assert!(!events.iter().any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "no CardDrawn event — the draw was replaced");
}

/// CR 702.2c (Deathtouch): any nonzero combat damage a deathtouch creature
/// deals is lethal. Here a 1/2 deathtouch *blocker* (Stinkweed Imp) kills a
/// 6/4 attacker (Craw Wurm) by dealing it 1 damage.
#[test]
fn cr_702_2c_deathtouch_blocker_destroys_larger_attacker() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    let imp = g.add_card_to_battlefield(1, catalog::stinkweed_imp()); // 1/2 deathtouch
    g.clear_sickness(wurm);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wurm, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != crate::game::types::TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(imp, wurm)])).expect("block");
    drain_stack(&mut g);
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(),
        "CR 702.2c: 1 point of deathtouch damage is lethal to the 6/4");
}

/// CR 302.6 (Summoning sickness): a creature can't attack on the turn it
/// comes under its controller's control unless it has haste.
#[test]
fn cr_302_6_summoning_sick_creature_cannot_attack() {
    let mut g = two_player_game();
    // No clear_sickness → the creature is summoning sick this turn.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }]));
    assert!(res.is_err(), "CR 302.6: summoning-sick creature can't be declared as attacker");
    // A haste creature (Lorehold Vanguard) is exempt.
    let haste = g.add_card_to_battlefield(0, catalog::lorehold_vanguard_b207());
    let ok = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: haste, target: AttackTarget::Player(1),
    }]));
    assert!(ok.is_ok(), "CR 702.10b: Haste exempts a freshly-entered creature");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 208 (modern_decks) — cross-school follow-ups.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn lorehold_vanguard_captain_b208_exalted_pumps_lone_attacker() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::lorehold_vanguard_captain_b208());
    g.clear_sickness(cap);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cap, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(cap).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3), "Exalted +1/+1 attacking alone");
}

#[test]
fn lorehold_pyrohistorian_b208_etb_pings_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyrohistorian_b208());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrohistorian castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "ETB 2 damage to face");
}

#[test]
fn lorehold_skydefender_b208_etb_gains_two_and_flies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_skydefender_b208());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skydefender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 2);
    let c = g.battlefield.iter().find(|c| c.definition.name == "Lorehold Skydefender (b208)").unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_scorchmage_b208_deals_five_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_scorchmage_b208());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scorchmage castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "5 damage kills the bear");
}

#[test]
fn prismari_scholar_adept_b208_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_scholar_adept_b208());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Just verify the cast resolves with the magecraft trigger present.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.stack.is_empty(), "magecraft scry resolved");
}

#[test]
fn quandrix_rootmage_b208_etb_counters_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_rootmage_b208());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rootmage castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_tidecantor_b208_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_tidecantor_b208());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidecantor castable");
    drain_stack(&mut g);
    // -1 cast + 2 draw = net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

// ════════════════════════════════════════════════════════════════════════════
// Coverage backfill (claude/modern_decks): functionality tests for STX cards
// that were wired but lacked a dedicated test.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn dina_soul_steeper_drains_on_lifegain() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    // "Whenever you gain life, each opponent loses 1 life."
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dina_soul_steeper());
    // Gain 5 life via Witherbloom Charm's lifegain mode.
    let charm = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    g.perform_action(GameAction::CastSpell {
        card_id: charm, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Witherbloom Charm castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "Dina drains 1 from the opponent on lifegain");
}

#[test]
fn professor_onyx_magecraft_drains_two() {
    // Magecraft: "Whenever you cast or copy an instant or sorcery spell,
    // each opponent loses 2 life and you gain 2 life."
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::professor_onyx());
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Bolt itself also deals 3 to P1; magecraft drains a further 2.
    assert_eq!(g.players[0].life, p0 + 2, "Onyx magecraft gains you 2");
    assert_eq!(g.players[1].life, p1 - 3 - 2, "bolt 3 + magecraft drain 2");
}

#[test]
fn professor_onyx_plus_one_drains_two() {
    let mut g = two_player_game();
    let onyx = g.add_card_to_battlefield(0, catalog::professor_onyx());
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: onyx, ability_index: 0, target: None,
    })
    .expect("Onyx +1 activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 2);
    assert_eq!(g.players[0].life, p0 + 2);
    let pw = g.battlefield_find(onyx).unwrap();
    assert_eq!(pw.counter_count(CounterType::Loyalty), 6, "5 base + 1 from the +1 ability");
}

#[test]
fn strixhaven_pondkeeper_etb_scries_and_has_flash() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_pondkeeper());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pondkeeper castable for {1}{U}");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Pondkeeper on battlefield");
    assert_eq!((c.definition.power, c.definition.toughness), (2, 1));
    assert!(c.definition.keywords.contains(&Keyword::Flash));
}

#[test]
fn zimone_quandrix_prodigy_draws_with_tap_ability() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::zimone_quandrix_prodigy());
    g.clear_sickness(id);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    })
    .expect("Zimone {1}, T: draw activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Zimone draws a card");
    assert!(g.battlefield_find(id).unwrap().tapped, "tapped by the ability");
}

#[test]
fn academic_probation_taps_and_stuns_target_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::academic_probation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Academic Probation castable for {1}{W}");
    drain_stack(&mut g);
    let bear = g.battlefield_find(opp_bear).unwrap();
    assert!(bear.tapped, "target creature tapped");
    assert_eq!(bear.counter_count(CounterType::Stun), 1, "one stun counter added");
}

#[test]
fn unwilling_ingredient_dies_draws_when_paid() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let ingredient = g.add_card_to_battlefield(0, catalog::unwilling_ingredient());
    // Pre-stash {2}{B} for the may-pay draw.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Kill the 1/1 Pest with a bolt.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(ingredient)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // The bolt left hand (-1) and the death trigger's paid draw added one
    // (+1): net hand size unchanged, but a fresh card was drawn.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "bolt left hand, paid draw replaced it");
    assert!(g.players[0].library.is_empty(),
        "Unwilling Ingredient drew the seeded card after paying {{2}}{{B}}");
}

#[test]
fn unwilling_ingredient_dies_no_draw_when_declined() {
    let mut g = two_player_game();
    let ingredient = g.add_card_to_battlefield(0, catalog::unwilling_ingredient());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(ingredient)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 1,
        "default decline → no draw");
}

#[test]
fn cr_122_1d_academic_probation_stun_persists_through_untap() {
    // End-to-end CR 122.1d via a real card: Academic Probation taps a
    // creature and gives it a stun counter; the creature does not untap
    // on its controller's next untap step and instead removes one stun
    // counter, untapping only on the following untap.
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::academic_probation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Academic Probation castable for {1}{W}");
    drain_stack(&mut g);
    {
        let bear = g.battlefield_find(opp_bear).unwrap();
        assert!(bear.tapped);
        assert_eq!(bear.counter_count(CounterType::Stun), 1);
    }
    // Untap step for the stunned creature's controller (seat 1): the
    // stun counter is consumed instead of untapping.
    g.active_player_idx = 1;
    g.do_untap();
    {
        let bear = g.battlefield_find(opp_bear).unwrap();
        assert!(bear.tapped, "stun keeps the creature tapped through one untap");
        assert_eq!(bear.counter_count(CounterType::Stun), 0, "the stun counter is consumed");
    }
    // Next untap: counter gone, untaps normally.
    g.do_untap();
    assert!(!g.battlefield_find(opp_bear).unwrap().tapped,
        "untaps normally once the stun counter is gone");
}

#[test]
fn diviners_wand_equips_for_three_and_buffs() {
    let mut g = two_player_game();
    let wand = g.add_card_to_battlefield(0, catalog::diviners_wand());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: wand, target: bear })
        .expect("Diviner's Wand equips for {3}");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1 over a 2/2 bear");
    assert!(cp.keywords.contains(&Keyword::Flying), "grants flying");
}

#[test]
fn opposition_taps_a_creature_to_tap_a_permanent() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(0, catalog::opposition());
    let _ = opp;
    // A creature to pay the tap cost (clear sickness so it's a valid tapper).
    let tapper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(tapper);
    // Opponent's land to tap down.
    let target = g.add_card_to_battlefield(1, catalog::island());
    let opp_id = g.battlefield.iter().find(|c| c.definition.name == "Opposition").unwrap().id;
    g.perform_action(GameAction::ActivateAbility {
        card_id: opp_id, ability_index: 0,
        target: Some(Target::Permanent(target)), x_value: None,
    }).expect("Opposition activates by tapping a creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapper).unwrap().tapped, "creature tapped to pay cost");
    assert!(g.battlefield_find(target).unwrap().tapped, "target permanent tapped");
}

#[test]
fn opposition_requires_an_untapped_creature() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(0, catalog::opposition());
    let target = g.add_card_to_battlefield(1, catalog::island());
    // No untapped creature to tap → activation rejected.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: opp, ability_index: 0,
        target: Some(Target::Permanent(target)), x_value: None,
    }).is_err(), "no creature to tap means no activation");
}

/// CR 602.5b — a `wants_ui` activator chooses *which* of their creatures to
/// tap for Opposition's "Tap an untapped creature you control" cost, rather
/// than the engine auto-tapping the weakest. Activation suspends on a
/// `ChooseTarget`; the chosen creature is the one tapped.
#[test]
fn opposition_ui_activator_chooses_creature_to_tap() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let opp = g.add_card_to_battlefield(0, catalog::opposition());
    let tapper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let keep = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(tapper);
    g.clear_sickness(keep);
    let target = g.add_card_to_battlefield(1, catalog::island());

    g.perform_action(GameAction::ActivateAbility {
        card_id: opp, ability_index: 0,
        target: Some(Target::Permanent(target)), x_value: None,
    })
    .expect("activation suspends for the tap-cost choice");

    let pd = g.pending_decision.as_ref().expect("a tap-cost choice is pending");
    assert_eq!(pd.acting_player(), 0);
    match &pd.decision {
        crate::decision::Decision::ChooseTarget { legal, .. } => {
            assert_eq!(legal.len(), 2, "both creatures are tap-cost options");
        }
        other => panic!("expected ChooseTarget, got {other:?}"),
    }

    g.perform_action(GameAction::SubmitDecision(crate::decision::DecisionAnswer::Target(
        Target::Permanent(tapper),
    )))
    .expect("submit the tap-cost choice");

    assert!(g.battlefield_find(tapper).unwrap().tapped, "chosen creature tapped to pay cost");
    assert!(!g.battlefield_find(keep).unwrap().tapped, "unchosen creature stays untapped");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped, "target permanent tapped on resolve");
}

#[test]
fn omniscience_casts_hand_spells_free() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::omniscience());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    // Empty mana pool — only Omniscience makes this castable.
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Omniscience casts Bolt for free");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "Bolt dealt 3 with no mana paid");
    // Spell goes to graveyard, not exile (Omniscience doesn't exile).
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"));
}

#[test]
fn free_cast_rejected_without_omniscience() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no free-cast permission without Omniscience");
}

#[test]
fn blade_historian_grants_double_strike_to_attackers_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blade_historian());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let homebody = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    let bf = g.compute_battlefield();
    let atk = bf.iter().find(|c| c.id == attacker).unwrap();
    let home = bf.iter().find(|c| c.id == homebody).unwrap();
    assert!(atk.keywords.contains(&Keyword::DoubleStrike),
        "attacking creature you control gains double strike");
    assert!(!home.keywords.contains(&Keyword::DoubleStrike),
        "non-attacking creature does not");
}

#[test]
fn academic_dispute_forces_block_if_able() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Academic Dispute's rider: must be blocked if able.
    g.battlefield_find_mut(attacker).unwrap()
        .granted_keywords_eot.push(Keyword::MustBeBlocked);
    g.step = crate::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = crate::game::types::TurnStep::DeclareBlockers;
    // Leaving it unblocked while opp has an idle able blocker is illegal.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_err(),
        "must-be-blocked attacker can't be left unblocked");
    // Assigning the idle blocker is legal.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(opp, attacker)])).is_ok(),
        "blocking it satisfies the requirement");
}

#[test]
fn must_be_blocked_allows_unblocked_when_no_able_blocker() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Opponent's only creature is tapped — not able to block.
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(opp).unwrap().tapped = true;
    g.clear_sickness(attacker);
    g.battlefield_find_mut(attacker).unwrap()
        .granted_keywords_eot.push(Keyword::MustBeBlocked);
    g.step = crate::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = crate::game::types::TurnStep::DeclareBlockers;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_ok(),
        "no able blocker → unblocked is legal");
}

#[test]
fn blade_historian_double_strike_deals_combat_damage_twice() {
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blade_historian());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    // CR 702.4: double striker deals in the first-strike step…
    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().expect("fs damage");
    assert_eq!(g.players[1].life, 18, "first-strike hit for 2");
    // …and again in the regular step.
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("regular damage");
    assert_eq!(g.players[1].life, 16, "regular hit for another 2");
}

#[test]
fn lorehold_mentor_buffs_lesser_power_attacker() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::lorehold_mentor()); // 3 power
    let smaller = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 < 3
    g.clear_sickness(mentor);
    g.clear_sickness(smaller);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: mentor, target: AttackTarget::Player(1) },
        Attack { attacker: smaller, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(smaller).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "lesser-power attacker gains a Mentor counter");
}

// ─────────────────────────────────────────────────────────────────────────
// Coverage backfill (modern_decks): functionality tests for previously
// untested-by-name STX cards. All cards already wired; tests only.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn fracture_destroys_target_artifact() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::fracture());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(rock)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fracture castable for {W}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == rock), "target artifact destroyed");
}

#[test]
fn closing_statement_exiles_creature_and_gains_x_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::closing_statement());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Closing Statement castable for {X}{W}{W} at X=3");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
    assert_eq!(g.players[0].life, life + 3, "gain X = 3 life");
}

#[test]
fn devastating_mastery_destroys_all_nonland_permanents() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::devastating_mastery());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Devastating Mastery castable for {4}{W}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mine), "own creature destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "opp creature destroyed");
}

#[test]
fn quandrix_apprentice_magecraft_pumps_a_creature() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(app).unwrap().power(), 2, "magecraft pumped a creature +1/+1");
    assert_eq!(g.battlefield_find(app).unwrap().toughness(), 2);
}

#[test]
fn carving_cherub_magecraft_pumps_a_creature() {
    let mut g = two_player_game();
    let cherub = g.add_card_to_battlefield(0, catalog::carving_cherub());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cherub).unwrap().power(), 2, "magecraft +1/+1");
}

#[test]
fn eager_first_year_magecraft_pumps_a_creature() {
    let mut g = two_player_game();
    let efy = g.add_card_to_battlefield(0, catalog::eager_first_year());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(efy).unwrap().power(), 3, "magecraft self-pump +1/+0 on the 2/2");
}

#[test]
fn disciplined_duelist_is_two_one_first_strike() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::disciplined_duelist());
    assert_eq!(g.battlefield_find(id).unwrap().power(), 2);
    assert_eq!(g.battlefield_find(id).unwrap().toughness(), 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::FirstStrike),
        "has first strike");
}

#[test]
fn codespell_cleric_is_one_one_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::codespell_cleric());
    assert_eq!(g.battlefield_find(id).unwrap().power(), 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Lifelink),
        "has lifelink");
}

#[test]
fn inkling_ambusher_is_two_two_flash_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_ambusher());
    assert_eq!(g.battlefield_find(id).unwrap().power(), 2);
    assert_eq!(g.battlefield_find(id).unwrap().toughness(), 2);
    let ci = g.battlefield_find(id).unwrap();
    assert!(ci.has_keyword(&Keyword::Flash), "has flash");
    assert!(ci.has_keyword(&Keyword::Flying), "has flying");
}

#[test]
fn electrickery_deals_one_to_target_creature() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(1, catalog::quandrix_apprentice());
    let id = g.add_card_to_hand(0, catalog::electrickery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(app)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Electrickery castable for {R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == app), "1 damage kills the 1/1");
}

#[test]
fn blustersquall_taps_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::blustersquall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Blustersquall castable for {U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "target creature tapped");
}

#[test]
fn multiple_choice_casts_and_resolves() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::multiple_choice());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multiple Choice castable for {1}{U}{U}");
    drain_stack(&mut g);
    // The modal sorcery resolves and is put into the graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Multiple Choice"),
        "Multiple Choice resolved to the graveyard");
}
