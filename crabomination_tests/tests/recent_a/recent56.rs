//! Functionality tests for `catalog::sets::decks::recent56` — lifegain-matters.

use crabomination::card::{CardType, CreatureType, Keyword, Subtypes};
use crabomination::catalog;
use crabomination::game::*;

/// A vanilla 1/1 white Angel token body for death/enter tests.
fn angel_1_1() -> crabomination::card::CardDefinition {
    crabomination::card::CardDefinition {
        name: "Test Angel",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

fn gain_life(g: &mut GameState, seat: usize, amount: i32) {
    let before = g.players[seat].life;
    g.adjust_life(seat, amount);
    let delta = g.players[seat].life - before;
    if delta > 0 {
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: seat, amount: delta as u32 }]);
        drain_stack(g);
    }
}

#[test]
fn bishop_of_wings_gains_on_angel_enter_and_makes_spirit_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bishop_of_wings());
    let life = g.players[0].life;
    // An Angel entering under your control → gain 4.
    let angel = g.add_card_to_battlefield(0, angel_1_1());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: angel }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained 4 when an Angel entered");
    // That Angel dying → make a 1/1 flying Spirit.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the angel");
    drain_stack(&mut g);
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit");
    assert!(spirit.is_some(), "an Angel dying made a Spirit");
    assert!(spirit.unwrap().has_keyword(&Keyword::Flying), "Spirit flies");
}

#[test]
fn youthful_valkyrie_grows_on_another_angel() {
    let mut g = two_player_game();
    let val = g.add_card_to_battlefield(0, catalog::youthful_valkyrie());
    let angel = g.add_card_to_battlefield(0, angel_1_1());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: angel }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(val).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        1,
        "another Angel entering grew the Valkyrie",
    );
}

#[test]
fn righteous_valkyrie_gains_and_anthems_at_high_life() {
    let mut g = two_player_game();
    let val = g.add_card_to_battlefield(0, catalog::righteous_valkyrie());
    let life = g.players[0].life;
    // A 1/1 Angel entering → gain life = its toughness (1).
    let angel = g.add_card_to_battlefield(0, angel_1_1());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: angel }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained life equal to entering creature's toughness");
    // Bump to starting+7 and confirm the team anthem kicks in.
    g.adjust_life(0, 7);
    let cp = g.compute_battlefield();
    let v = cp.iter().find(|c| c.id == val).unwrap();
    assert_eq!((v.power, v.toughness), (4, 6), "+2/+2 anthem while 7 above starting");
}

#[test]
fn twinblade_paladin_grows_and_gains_double_strike() {
    let mut g = two_player_game();
    let pal = g.add_card_to_battlefield(0, catalog::twinblade_paladin());
    gain_life(&mut g, 0, 1);
    assert_eq!(
        g.battlefield_find(pal).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        1,
        "gaining life grew the Paladin",
    );
    g.adjust_life(0, 5); // 26 total ≥ 25
    let cp = g.compute_battlefield();
    assert!(
        cp.iter().find(|c| c.id == pal).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "double strike while at 25+ life",
    );
}

#[test]
fn rhox_faithmender_doubles_life_gain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rhox_faithmender());
    let life = g.players[0].life;
    g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, life + 6, "life gain doubled");
}

#[test]
fn vito_drains_opponent_on_life_gain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vito_thorn_of_the_dusk_rose());
    let opp = g.players[1].life;
    gain_life(&mut g, 0, 3);
    assert_eq!(g.players[1].life, opp - 3, "opponent lost life equal to the gain");
}

#[test]
fn angelic_chorus_gains_life_equal_to_toughness() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::angelic_chorus());
    let life = g.players[0].life;
    // A 2/2 entering → gain 2.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained life = entering creature's toughness");
}

#[test]
fn exquisite_blood_gains_when_opponent_loses_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::exquisite_blood());
    let life = g.players[0].life;
    g.adjust_life(1, -4);
    g.dispatch_triggers_for_events(&[GameEvent::LifeLost { player: 1, amount: 4 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained life equal to opponent's loss");
}

#[test]
fn epicure_of_blood_drains_each_opponent_on_life_gain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::epicure_of_blood());
    let opp = g.players[1].life;
    gain_life(&mut g, 0, 5);
    assert_eq!(g.players[1].life, opp - 1, "each opponent lost 1 on life gain");
}

#[test]
fn celestial_unicorn_and_gideons_company_grow_on_life_gain() {
    let mut g = two_player_game();
    let uni = g.add_card_to_battlefield(0, catalog::celestial_unicorn());
    let comp = g.add_card_to_battlefield(0, catalog::gideons_company());
    gain_life(&mut g, 0, 1);
    let p1p1 = crabomination::card::CounterType::PlusOnePlusOne;
    assert_eq!(g.battlefield_find(uni).unwrap().counter_count(p1p1), 1, "Unicorn +1 counter");
    assert_eq!(g.battlefield_find(comp).unwrap().counter_count(p1p1), 2, "Gideon's Company +2 counters");
}

#[test]
fn dauntless_bodyguard_grants_indestructible_to_chosen() {
    let mut g = two_player_game();
    let ward = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let guard = g.add_card_to_battlefield(0, catalog::dauntless_bodyguard());
    // ETB: choose the ward (only other creature) as the protected creature.
    g.fire_self_etb_triggers(guard, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(guard).unwrap().chosen_permanent, Some(ward), "remembered the ward");
    // Sacrifice the guard → the chosen creature gains indestructible.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: guard, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac the bodyguard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(guard).is_none(), "bodyguard sacrificed");
    let cp = g.compute_battlefield();
    assert!(
        cp.iter().find(|c| c.id == ward).unwrap().keywords.contains(&Keyword::Indestructible),
        "the chosen creature gained indestructible",
    );
}

#[test]
fn griffin_aerie_makes_griffin_after_gaining_three() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::griffin_aerie());
    g.adjust_life(0, 3); // gained 3 this turn
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Griffin"), "made a Griffin");
}

#[test]
fn crested_sunmare_makes_horse_and_shields_other_horses() {
    let mut g = two_player_game();
    let mare = g.add_card_to_battlefield(0, catalog::crested_sunmare());
    // Gain life, then end step → make a Horse.
    g.adjust_life(0, 2);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let horse = g.battlefield.iter().find(|c| c.definition.name == "Horse").map(|c| c.id);
    assert!(horse.is_some(), "made a Horse token");
    // The token Horse is indestructible (lord); the Sunmare itself is not.
    let cp = g.compute_battlefield();
    assert!(
        cp.iter().find(|c| c.id == horse.unwrap()).unwrap().keywords.contains(&Keyword::Indestructible),
        "other Horses are indestructible",
    );
    assert!(
        !cp.iter().find(|c| c.id == mare).unwrap().keywords.contains(&Keyword::Indestructible),
        "the Sunmare itself is not (only *other* Horses)",
    );
}

#[test]
fn linden_gains_when_white_creature_attacks() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::linden_the_steadfast_queen());
    let life = g.players[0].life;
    // A white creature attacking → gain 1. (Linden herself is white.)
    let soldier = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.dispatch_triggers_for_events(&[GameEvent::AttackerDeclared(soldier)]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 when a white creature attacked");
}

#[test]
fn kambal_drains_on_opponent_noncreature_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kambal_consul_of_allocation());
    let (my, opp) = (g.players[0].life, g.players[1].life);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a noncreature spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "the casting opponent lost 2 (plus the bolt's 0 to self)");
    assert!(g.players[0].life >= my - 3, "you gained 2 (bolt still hits you for 3)");
}

#[test]
fn sunscorch_regent_grows_and_gains_on_opponent_spell() {
    let mut g = two_player_game();
    let reg = g.add_card_to_battlefield(0, catalog::sunscorch_regent());
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(reg).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        1,
        "Regent grew on the opponent's spell",
    );
    // gained 1 from the Regent, lost 3 to the bolt → net -2.
    assert_eq!(g.players[0].life, life + 1 - 3);
}

#[test]
fn souls_grace_gains_life_equal_to_target_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let grace = g.add_card_to_hand(0, catalog::souls_grace());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: grace, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Soul's Grace");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained life equal to target's power");
}

#[test]
fn valkyrie_harbinger_and_regal_bloodlord_make_tokens_at_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::valkyrie_harbinger());
    g.add_card_to_battlefield(0, catalog::regal_bloodlord());
    g.adjust_life(0, 4); // ≥4 for both
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Angel"), "Harbinger made an Angel");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bat"), "Bloodlord made a Bat");
}

/// The new `#[serde(default)]` state — Player.starting_life and
/// CardInstance.chosen_permanent — survives a full-state snapshot round-trip.
#[test]
fn new_serde_fields_survive_snapshot_roundtrip() {
    let mut g = two_player_game();
    g.players[0].starting_life = 40;
    let ward = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let guard = g.add_card_to_battlefield(0, catalog::dauntless_bodyguard());
    g.fire_self_etb_triggers(guard, 0);
    drain_stack(&mut g);
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: GameState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g2.players[0].starting_life, 40, "starting_life round-trips");
    assert_eq!(
        g2.battlefield_find(guard).unwrap().chosen_permanent, Some(ward),
        "chosen_permanent round-trips",
    );
}
