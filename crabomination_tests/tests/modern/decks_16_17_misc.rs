#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── modern_decks-16: new cube cards ──────────────────────────────────────────

#[test]
fn electrolyze_deals_two_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::electrolyze());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Electrolyze castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 dies to 2 damage");
    assert_eq!(g.players[0].hand.len(), hand_before, "cast(-1) + draw(+1) = net 0");
}

/// Electrolyze divides its 2 damage across a creature and a player, then
/// still draws — exercises the `DealDamageDivided` slot wiring in a `Seq`.
#[test]
fn electrolyze_divides_damage_across_creature_and_player_then_draws() {
    let mut g = two_player_game();
    let one_one = g.add_card_to_battlefield(1, catalog::pteramander()); // 1/1
    let id = g.add_card_to_hand(0, catalog::electrolyze());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.add_card_to_library(0, catalog::island());
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(one_one)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    }).expect("Electrolyze castable with two targets");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == one_one), "1/1 dies to its 1 damage");
    assert_eq!(g.players[1].life, 19, "player takes the other 1");
}

#[test]
fn collective_brutality_mode_zero_shrinks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::collective_brutality());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Collective Brutality castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 dies to -2/-2");
}

#[test]
fn expressive_iteration_exiles_top_three() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::expressive_iteration());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Expressive Iteration castable");
    drain_stack(&mut g);

    assert!(g.players[0].library.len() < lib_before, "Top 3 should be exiled from library");
}

#[test]
fn kitchen_finks_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::kitchen_finks());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Kitchen Finks castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2, "ETB gains 2 life");
}

#[test]
fn wall_of_omens_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::wall_of_omens());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wall of Omens castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before, "cast(-1) + draw(+1) = net 0");
    let wall = g.battlefield.iter().find(|c| c.definition.name == "Wall of Omens").unwrap();
    assert_eq!(wall.definition.power, 0);
    assert_eq!(wall.definition.toughness, 4);
}

#[test]
fn mulldrifter_etb_draws_two() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::mulldrifter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mulldrifter castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "cast(-1) + draw(+2) = net +1");
}

#[test]
fn mulldrifter_evoke_draws_two_then_sacrifices_self() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::mulldrifter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mulldrifter evoke for {2}{U}");
    drain_stack(&mut g);
    // ETB draw two fires, then evoke sacrifices it to the graveyard.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "cast(-1) + draw(+2)");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "evoked Mulldrifter sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn thragtusk_etb_gains_five_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::thragtusk());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thragtusk castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 5, "ETB gains 5 life");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Thragtusk"));
}

#[test]
fn lingering_souls_creates_two_spirit_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lingering_souls());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lingering Souls castable");
    drain_stack(&mut g);

    let spirits: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").collect();
    assert_eq!(spirits.len(), 2, "Two Spirit tokens created");
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

#[test]
fn firebolt_deals_two_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::firebolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Firebolt castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 dies to 2 damage");
}

#[test]
fn chainers_edict_forces_sacrifice() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chainers_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Chainer's Edict castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "opponent forced to sacrifice");
}

#[test]
fn deep_analysis_draws_two() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::deep_analysis());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Deep Analysis castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "cast(-1) + draw(+2) = net +1");
}

#[test]
fn tireless_provisioner_creates_treasure_on_landfall() {
    let mut g = two_player_game();
    let _prov = g.add_card_to_battlefield(0, catalog::tireless_provisioner());
    let land_id = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land_id)).expect("play land");
    drain_stack(&mut g);

    let treasures: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").collect();
    assert!(!treasures.is_empty(), "Treasure token created on landfall");
}

#[test]
fn courser_of_kruphix_gains_life_on_landfall() {
    let mut g = two_player_game();
    let _courser = g.add_card_to_battlefield(0, catalog::courser_of_kruphix());
    let land_id = g.add_card_to_hand(0, catalog::forest());
    let life_before = g.players[0].life;

    g.perform_action(GameAction::PlayLand(land_id)).expect("play land");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 1, "Landfall gains 1 life");
}

#[test]
fn bloodbraid_elf_has_haste_and_cascades() {
    // Bloodbraid Elf is now a real cascade card (CR 702.85): declining
    // the cascade (AutoDecider default) just resolves the Elf with haste.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::bloodbraid_elf());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodbraid Elf castable");
    drain_stack(&mut g);

    let bbe = g.battlefield.iter().find(|c| c.definition.name == "Bloodbraid Elf").unwrap();
    assert!(bbe.definition.keywords.contains(&crabomination::card::Keyword::Haste));
    assert!(bbe.definition.triggered_abilities.iter().any(|t|
        matches!(t.effect, crabomination::effect::Effect::Cascade { .. })),
        "Bloodbraid Elf carries a cascade trigger");
}

#[test]
fn oko_plus_two_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::oko_thief_of_crowns());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Oko castable");
    drain_stack(&mut g);

    let oko = g.battlefield.iter().find(|c| c.definition.name == "Oko, Thief of Crowns").unwrap();
    let oko_id = oko.id;

    // Activate +2
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: oko_id, ability_index: 0, target: None,
    }).expect("+2 activation");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 3, "Oko +2 gains 3 life");
}

#[test]
fn oko_plus_one_turns_target_into_a_three_three_elk() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let oko = g.add_card_to_battlefield(0, catalog::oko_thief_of_crowns());
    // A 2/2 with abilities (flying) — Oko strips it to a vanilla 3/3 Elk.
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flier

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: oko, ability_index: 1, target: Some(Target::Permanent(target)),
    }).expect("+1 activation");
    drain_stack(&mut g);

    let cp = g.computed_permanent(target).expect("target still on battlefield");
    assert_eq!((cp.power, cp.toughness), (3, 3), "becomes 3/3");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Elk), "is an Elk");
    assert!(cp.lost_all_abilities, "loses all abilities (no more flying)");
    assert!(!cp.keywords.contains(&Keyword::Flying), "flying stripped");
}

#[test]
fn become_basic_land_taps_for_the_new_color() {
    use crabomination::effect::{Effect, Selector, Duration};
    // A Forest converted to an Island via `BecomeBasicLand` taps for blue
    // (intrinsic basic-land mana) and no longer makes green.
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());

    let ctx = crabomination::game::effects::EffectContext::for_ability(
        crabomination::card::CardId(0), 0, Some(Target::Permanent(land)),
    );
    g.resolve_effect(
        &Effect::BecomeBasicLand {
            what: Selector::Target(0),
            land_type: crabomination::card::LandType::Island,
            duration: Duration::Permanent,
        },
        &ctx,
    ).unwrap();

    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.subtypes.land_types.contains(&crabomination::card::LandType::Island));
    assert!(!cp.subtypes.land_types.contains(&crabomination::card::LandType::Forest));

    // Auto-tap for {U} should tap the now-Island and fill the blue pool.
    let cost = crabomination::mana::cost(&[crabomination::mana::u()]);
    g.auto_tap_for_cost(0, &cost);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "taps for blue");
    assert!(g.battlefield.iter().find(|c| c.id == land).unwrap().tapped);
}

/// Blood Moon: nonbasic lands are Mountains — type line swapped, printed
/// abilities gone, and the land taps for red. Basics untouched.
#[test]
fn blood_moon_turns_nonbasics_into_mountains() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blood_moon());
    let manland = g.add_card_to_battlefield(1, catalog::celestial_colonnade());
    let island = g.add_card_to_battlefield(1, catalog::island());
    let cp = g.computed_permanent(manland).unwrap();
    assert_eq!(cp.subtypes.land_types, vec![LandType::Mountain], "nonbasic is a Mountain");
    assert!(cp.lost_all_abilities, "printed abilities stripped");
    let cp = g.computed_permanent(island).unwrap();
    assert_eq!(cp.subtypes.land_types, vec![LandType::Island], "basics unaffected");
    // The moonscaped land taps for {R}.
    g.battlefield_find_mut(manland).unwrap().summoning_sick = false;
    let cost = crabomination::mana::cost(&[crabomination::mana::r()]);
    g.auto_tap_for_cost(1, &cost);
    assert_eq!(g.players[1].mana_pool.amount(Color::Red), 1, "taps for red");
}

/// Urborg: every land is a Swamp in addition — it taps for black, keeping
/// its other types.
#[test]
fn urborg_makes_every_land_a_swamp_in_addition() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::urborg_tomb_of_yawgmoth());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let cp = g.computed_permanent(forest).unwrap();
    assert!(cp.subtypes.land_types.contains(&LandType::Swamp), "Swamp in addition");
    assert!(cp.subtypes.land_types.contains(&LandType::Forest), "keeps Forest");
    let cost = crabomination::mana::cost(&[crabomination::mana::b()]);
    g.auto_tap_for_cost(1, &cost);
    assert_eq!(g.players[1].mana_pool.amount(Color::Black), 1, "Forest taps for black");
}

/// Mind Bend permanently swaps a color word (CR 612) — survives cleanup.
#[test]
fn mind_bend_swaps_color_word_permanently() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, crabomination::card::CardDefinition {
        name: "Test Knight",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Protection(Color::Red)],
        ..Default::default()
    });
    let mb = g.add_card_to_hand(0, catalog::mind_bend());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::Red),
        DecisionAnswer::Color(Color::Green),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: mb, target: Some(Target::Permanent(knight)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Mind Bend castable");
    drain_stack(&mut g);
    g.expire_end_of_turn_effects();
    let computed = g.computed_permanent(knight).unwrap();
    assert!(computed.keywords.contains(&Keyword::Protection(Color::Green)),
        "swap survives end of turn");
}

/// Yavimaya: each land is also a Forest — taps for green alongside its
/// printed color.
#[test]
fn yavimaya_makes_every_land_a_forest_in_addition() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::yavimaya_cradle_of_growth());
    let island = g.add_card_to_battlefield(1, catalog::island());
    let cp = g.computed_permanent(island).unwrap();
    assert!(cp.subtypes.land_types.contains(&LandType::Forest));
    assert!(cp.subtypes.land_types.contains(&LandType::Island));
}

/// Ensnaring Bridge: a creature with power above the bridge controller's
/// hand size can't attack; one at or under the cap can.
#[test]
fn ensnaring_bridge_caps_attacker_power_by_hand_size() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::ensnaring_bridge());
    g.players[1].hand.clear();
    g.add_card_to_hand(1, catalog::island()); // hand size 1
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let small = g.add_card_to_battlefield(0, catalog::ornithopter()); // 0/2
    g.clear_sickness(big);
    g.clear_sickness(small);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: big, target: AttackTarget::Player(1) },
    ])).is_err(), "power 4 > hand 1 can't attack");
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: small, target: AttackTarget::Player(1) },
    ])).expect("power 0 attacks under the bridge");
}

/// Surgical Extraction rips every copy of the targeted graveyard card.
#[test]
fn surgical_extraction_strips_all_copies() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let in_hand = g.add_card_to_hand(1, catalog::lightning_bolt());
    let in_lib = g.add_card_to_library(1, catalog::lightning_bolt());
    let se = g.add_card_to_hand(0, catalog::surgical_extraction());
    // {B/P} paid with 2 life — no mana needed.
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: se, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Surgical castable for Phyrexian black");
    drain_stack(&mut g);
    for id in [dead, in_hand, in_lib] {
        assert!(g.exile.iter().any(|c| c.id == id), "copy exiled");
    }
    assert_eq!(g.players[0].life, life_before - 2, "paid 2 life for Phyrexian pip");
}

/// Kaldra Compleat: living weapon Germ becomes a 5/5; combat damage to a
/// blocking creature exiles it.
#[test]
fn kaldra_compleat_germ_exiles_blockers() {
    let mut g = two_player_game();
    let kaldra = g.add_card_to_battlefield(0, catalog::kaldra_compleat());
    g.fire_self_etb_triggers(kaldra, 0);
    drain_stack(&mut g);
    let germ = g.battlefield.iter()
        .find(|c| c.definition.name == "Phyrexian Germ")
        .map(|c| c.id)
        .expect("Germ minted and equipped");
    let cp = g.computed_permanent(germ).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "Germ wears Kaldra");
    // Attack; a blocker dies to first-strike damage and is exiled.
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: germ, target: AttackTarget::Player(1) },
    ])).expect("Germ has haste");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, germ)])).expect("block");
    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().expect("first strike");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == blocker), "damaged blocker exiled");
}

#[test]
fn toxic_deluge_sweeps_small_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::toxic_deluge());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Toxic Deluge castable with X=3");
    drain_stack(&mut g);

    let creatures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.card_types.contains(&CardType::Creature))
        .collect();
    assert!(creatures.is_empty(), "All 2/2s die to -3/-3");
}

#[test]
fn sinkhole_destroys_target_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::sinkhole());
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(land)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sinkhole castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == land), "land destroyed");
}

#[test]
fn wear_tear_right_half_destroys_enchantment() {
    // CR 709 — casting the Tear (right) half destroys target enchantment.
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::bad_moon());
    let id = g.add_card_to_hand(0, catalog::wear_tear());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSplitRight {
        card_id: id, target: Some(Target::Permanent(ench)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tear castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == ench), "enchantment destroyed");
}

#[test]
fn wear_tear_fused_destroys_both() {
    // CR 702.102 — fused cast destroys an artifact (Wear) and an enchantment
    // (Tear) for the combined cost {1}{R}{W}. Left target rides `target`,
    // right target rides `additional_targets` slot 0.
    let mut g = two_player_game();
    let artifact = g.add_card_to_battlefield(1, catalog::sol_ring());
    let ench = g.add_card_to_battlefield(1, catalog::bad_moon());
    let id = g.add_card_to_hand(0, catalog::wear_tear());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSplitFused {
        card_id: id,
        target: Some(Target::Permanent(artifact)),
        additional_targets: vec![Target::Permanent(ench)],
        mode: None, x_value: None,
    }).expect("fused cast");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == artifact), "artifact destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == ench), "enchantment destroyed");
}

#[test]
fn murderous_cut_destroys_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::murderous_cut());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Murderous Cut castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature destroyed");
}

#[test]
fn fiery_confluence_burns_opponent_for_six_via_repeated_mode() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fiery_confluence());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    let opp_life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fiery Confluence castable");
    drain_stack(&mut g);

    // Default picks repeat the 2-damage-each-opponent mode three times.
    assert_eq!(g.players[1].life, opp_life - 6, "choose-three burns for 2×3 = 6");
}

#[test]
fn intervention_pact_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::intervention_pact());
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Intervention Pact castable at {0}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 5, "gained 5 life");
}

#[test]
fn baleful_mastery_exiles_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::baleful_mastery());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Baleful Mastery castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature exiled");
}

#[test]
fn elite_spellbinder_etb_strips_card() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let hand_before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::elite_spellbinder());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Elite Spellbinder castable");
    drain_stack(&mut g);

    assert!(g.players[1].hand.len() < hand_before, "opponent lost a card");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Elite Spellbinder"));
}

#[test]
fn explore_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::explore());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Explore castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before, "cast(-1) + draw(+1) = net 0");
}

// ── modern_decks-17 tests ──────────────────────────────────────────────────

/// Grim Flayer: 2/2 with trample and a DealsCombatDamageToPlayer trigger.
/// Grim Flayer: combat damage trigger surveils 2.
#[test]
fn grim_flayer_combat_trigger_surveils() {
    let mut g = two_player_game();
    let flayer = g.add_card_to_battlefield(0, catalog::grim_flayer());
    g.clear_sickness(flayer);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lib_before = g.players[0].library.len();
    // Fire the trigger effect directly.
    let trig = catalog::grim_flayer().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(
        flayer, 0, None, 0,
    );
    let _ = g.resolve_effect(&trig, &ctx);
    // Surveil 2 puts cards on bottom or into graveyard; library shrinks by 2
    // (auto-decider sends both to bottom — effectively they leave the top).
    assert!(g.players[0].library.len() <= lib_before,
        "surveil should process library cards");
}

#[test]
fn fallen_shinobi_ninjutsu_swaps_in_for_an_unblocked_attacker() {
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    // An unblocked 2/2 is attacking p1; the Shinobi waits in p0's hand.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let shinobi = g.add_card_to_hand(0, catalog::fallen_shinobi());
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::Ninjutsu { ninja: shinobi, returning: bear })
        .expect("Ninjutsu activates on an unblocked attacker");

    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "the unblocked attacker returns to hand");
    assert!(g.battlefield.iter().any(|c| c.id == shinobi),
        "the Shinobi enters the battlefield");
    assert!(g.attacking.iter().any(|a| a.attacker == shinobi),
        "the Shinobi is now attacking");
    let sh = g.battlefield.iter().find(|c| c.id == shinobi).unwrap();
    assert!(sh.tapped, "the Shinobi enters tapped and attacking");
}

#[test]
fn fallen_shinobi_ninjutsu_rejected_on_blocked_attacker() {
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let shinobi = g.add_card_to_hand(0, catalog::fallen_shinobi());
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.block_map.insert(blocker, bear); // bear is blocked
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::Ninjutsu { ninja: shinobi, returning: bear }).is_err(),
        "Ninjutsu can't return a blocked attacker");
}

#[test]
fn fallen_shinobi_exiles_two_and_grants_may_play() {
    let mut g = two_player_game();
    let shinobi = g.add_card_to_battlefield(0, catalog::fallen_shinobi());
    g.clear_sickness(shinobi);
    let a = g.add_card_to_library(1, catalog::lightning_bolt());
    let b = g.add_card_to_library(1, catalog::shock());
    // Fire the combat-damage trigger with seat 1 stamped as the damaged
    // (defending) player.
    let trig = catalog::fallen_shinobi().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(
        shinobi, 0, Some(Target::Player(1)), 0,
    );
    g.resolve_effect(&trig, &ctx).unwrap();
    // Both top cards of the *defender's* library are exiled, each with a
    // may-play permission for the Shinobi's controller (seat 0).
    for id in [a, b] {
        let card = g.exile.iter().find(|c| c.id == id)
            .expect("defender's top card is exiled");
        let perm = card.may_play_until.as_ref().expect("may-play granted");
        assert_eq!(perm.player, 0, "the Shinobi's controller may play it");
    }
}

/// Young Pyromancer: magecraft trigger creates an Elemental token.
#[test]
fn young_pyromancer_creates_elemental_on_instant_cast() {
    let mut g = two_player_game();
    let _pyro = g.add_card_to_battlefield(0, catalog::young_pyromancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lightning Bolt castable");
    drain_stack(&mut g);

    let elementals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Elemental")
        .collect();
    assert_eq!(elementals.len(), 1,
        "Young Pyromancer should create one Elemental token on instant cast");
}

/// Young Pyromancer: stat check.
/// Monastery Swiftspear: 1/2 with Haste and Prowess.
/// Snapcaster Mage: 2/1 Flash creature, ETB draws a card.
#[test]
fn snapcaster_mage_etb_grants_may_play() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let snap = g.add_card_to_hand(0, catalog::snapcaster_mage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: snap, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Snapcaster Mage castable for {1}{U}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.definition.name == "Snapcaster Mage"));
}

/// Snapcaster Mage: has Flash keyword.
/// Grisly Salvage: mills 5 then scries 1.
#[test]
fn grisly_salvage_mills_five_and_scries() {
    let mut g = two_player_game();
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::grisly_salvage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let lib_before = g.players[0].library.len();
    let yard_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Grisly Salvage castable for {B}{G}");
    drain_stack(&mut g);

    // Mill 5 puts 5 cards in graveyard (plus the spell itself = 6).
    assert!(g.players[0].graveyard.len() >= yard_before + 5,
        "should mill at least 5 cards into graveyard");
    // Library lost 5 from mill (+1 from scry bottom potentially).
    assert!(g.players[0].library.len() <= lib_before - 5,
        "library should shrink by at least 5");
}

/// Thought Erasure: discard a nonland card + surveil 1.
#[test]
fn thought_erasure_strips_nonland_and_surveils() {
    let mut g = two_player_game();
    // Give opponent a nonland card.
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let opp_hand_before = g.players[1].hand.len();
    // Stock library for surveil.
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::thought_erasure());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thought Erasure castable for {U}{B}");
    drain_stack(&mut g);

    assert!(g.players[1].hand.len() < opp_hand_before,
        "opponent should lose a nonland card");
}

/// Lightning Greaves: {2} artifact with Equipment subtype.
/// Lightning Greaves: equipping ({0}) grants haste and shroud to the
/// equipped creature via the layer system.
#[test]
fn lightning_greaves_grants_haste_and_shroud_when_equipped() {
    let mut g = two_player_game();
    let greaves = g.add_card_to_battlefield(0, catalog::lightning_greaves());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Bear is summoning sick by default.
    assert!(g.battlefield_find(bear).unwrap().summoning_sick);

    g.perform_action(GameAction::Equip { equipment: greaves, target: bear })
        .expect("Greaves equips for {0}");

    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Haste), "haste granted");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Shroud), "shroud granted");
}

/// Tasigur, the Golden Fang: 4/5 Legendary creature.
/// Tasigur: activated ability mills 2.
#[test]
fn tasigur_activated_ability_mills() {
    let mut g = two_player_game();
    let tasigur = g.add_card_to_battlefield(0, catalog::tasigur_the_golden_fang());
    g.clear_sickness(tasigur);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    // Put a nonland card in graveyard for the Move half.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let lib_before = g.players[0].library.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: tasigur, ability_index: 0, target: None,
        additional_targets: Vec::new(),
        x_value: None,
    }).expect("Tasigur ability activates for {2}{G}");
    drain_stack(&mut g);

    assert!(g.players[0].library.len() <= lib_before - 2,
        "should mill at least 2 cards");
}

#[test]
fn tasigur_activated_ability_hybrid_pip_payable_with_blue() {
    // {2}{G/U}: pay the hybrid pip with blue instead of green.
    let mut g = two_player_game();
    let tasigur = g.add_card_to_battlefield(0, catalog::tasigur_the_golden_fang());
    g.clear_sickness(tasigur);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let lib_before = g.players[0].library.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: tasigur, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Tasigur ability activates for {2}{U} via the hybrid pip");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before - 2);
}

/// Tasigur can be cast for {B} after delving five graveyard cards.
#[test]
fn tasigur_delve_pays_generic_from_graveyard() {
    let mut g = two_player_game();
    let gy: Vec<_> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let id = g.add_card_to_hand(0, catalog::tasigur_the_golden_fang());
    g.players[0].mana_pool.add(Color::Black, 1); // {5} covered by delve, {B} by mana
    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy.clone(),
    }).expect("Tasigur castable for {B} after delving five");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Tasigur resolved onto the battlefield");
}

/// Stonecoil Serpent: 0/0 artifact creature with trample and reach.
/// Stonecoil Serpent: definition shape check.
// ── modern_decks-17: agent-implemented cards ────────────────────────────────

#[test]
fn young_pyromancer_creates_token_on_is_cast() {
    let mut g = two_player_game();
    let _yp = g.add_card_to_battlefield(0, catalog::young_pyromancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt cast");
    drain_stack(&mut g);

    let tokens: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Elemental" && c.definition.power == 1).collect();
    assert!(!tokens.is_empty(), "Young Pyromancer created at least one Elemental token");
}

#[test]
fn thought_erasure_strips_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let hand_before = g.players[1].hand.len();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::thought_erasure());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thought Erasure castable");
    drain_stack(&mut g);

    assert!(g.players[1].hand.len() < hand_before, "opponent lost a card");
}

#[test]
fn grisly_salvage_mills_and_draws() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::grisly_salvage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Grisly Salvage castable");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.len() > gy_before, "cards milled to graveyard");
}

// ── Chain Lightning ─────────────────────────────────────────────────────────

#[test]
fn chain_lightning_deals_three_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::chain_lightning());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Chain Lightning castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3, "3 damage to opponent");
}

#[test]
fn chain_lightning_kills_a_three_toughness_creature() {
    let mut g = two_player_game();
    // Centaur Courser is a 3/3 (we use a card with 3 toughness).
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::chain_lightning());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Chain Lightning targets creature");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 dies to 3 damage");
}

// ── Rift Bolt ───────────────────────────────────────────────────────────────

#[test]
fn rift_bolt_deals_three_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rift_bolt());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rift Bolt castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3, "3 damage to opponent");
}

// ── Exquisite Firecraft ─────────────────────────────────────────────────────

#[test]
fn exquisite_firecraft_deals_four_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::exquisite_firecraft());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Exquisite Firecraft castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 4, "4 damage to opponent");
}

// ── Sulfuric Vortex ─────────────────────────────────────────────────────────

#[test]
fn sulfuric_vortex_deals_damage_at_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sulfuric_vortex());
    let life_before = g.players[0].life;

    // Roll to Alice's upkeep so the trigger fires.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 0;
    for _ in 0..30 {
        if g.is_game_over() {
            break;
        }
        if g.active_player_idx == 0
            && g.step == TurnStep::Upkeep
            && g.stack.is_empty()
            && g.players[0].life < life_before
        {
            break;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }

    assert_eq!(g.players[0].life, life_before - 2,
        "Sulfuric Vortex should deal 2 to the active player at upkeep");
}

#[test]
fn sulfuric_vortex_locks_out_lifegain_for_everyone() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sulfuric_vortex());
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    // Both players' lifegain is suppressed while the Vortex is in play.
    g.adjust_life(0, 5);
    g.adjust_life(1, 5);
    assert_eq!(g.players[0].life, p0, "controller can't gain life");
    assert_eq!(g.players[1].life, p1, "opponent can't gain life either");
}

// ── Kari Zev, Skyship Raider ───────────────────────────────────────────────

#[test]
fn kari_zev_creates_ragavan_on_attack() {
    let mut g = two_player_game();
    let kari = g.add_card_to_battlefield(0, catalog::kari_zev_skyship_raider());
    g.clear_sickness(kari);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kari,
        target: AttackTarget::Player(1),
    }]))
    .expect("Kari Zev attacks");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Ragavan"),
        "Attacking with Kari Zev should create a Ragavan token");
}

// ── Scavenging Ooze ─────────────────────────────────────────────────────────

#[test]
fn scavenging_ooze_gains_counter_and_life() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let ooze = g.add_card_to_battlefield(0, catalog::scavenging_ooze());
    g.clear_sickness(ooze);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: ooze,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("Scavenging Ooze ability activates");
    drain_stack(&mut g);

    let counters = g.battlefield.iter().find(|c| c.id == ooze)
        .and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(counters, 1, "Ooze should have one +1/+1 counter");
    assert_eq!(g.players[0].life, life_before + 1, "Should gain 1 life");
}

// ── Push XVII continued: ETB creatures ─────────────────────────────────────

#[test]
fn fiend_hunter_exiles_opponent_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::fiend_hunter());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().find(|c| c.id == bear).is_none(), "bear should be exiled");
    assert!(g.exile.iter().any(|c| c.id == bear), "bear in exile linked to Fiend Hunter");
    // Fiend Hunter leaves → the exiled creature returns to the battlefield.
    g.remove_from_battlefield_to_graveyard_raw(id);
    assert!(g.battlefield_find(bear).is_some(), "bear returns when Fiend Hunter leaves");
}

#[test]
fn flametongue_kavu_etb_deals_four() {
    let mut g = two_player_game();
    // Use a 5-toughness creature so 4 damage doesn't kill it.
    let big = g.add_card_to_battlefield(1, catalog::devourer_of_destiny());
    let id = g.add_card_to_hand(0, catalog::flametongue_kavu());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(big)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let target = g.battlefield.iter().find(|c| c.id == big).unwrap();
    assert_eq!(target.damage, 4, "should deal 4 damage to target");
}

// ── Push XVII unique cards ─────────────────────────────────────────────────

#[test]
fn esikas_chariot_etb_creates_two_cat_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::esikas_chariot());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Esika's Chariot castable");
    drain_stack(&mut g);

    assert!(g.battlefield.len() >= bf_before + 3, "Self + 2 Cat tokens");
    let cats: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Cat").collect();
    assert_eq!(cats.len(), 2, "Should create exactly 2 Cat tokens");
}

#[test]
fn magda_sacrifices_five_treasures_to_tutor_a_dragon() {
    use crabomination::card::ArtifactSubtype;
    use crabomination::effect::{PlayerRef, Value};
    let mut g = two_player_game();
    let magda = g.add_card_to_battlefield(0, catalog::magda_brazen_outlaw());
    g.clear_sickness(magda);
    // Mint five Treasure tokens for p0.
    let ctx = crabomination::game::effects::EffectContext::for_trigger(magda, 0, None, 0);
    let mint = Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(5),
        definition: crabomination::game::effects::treasure_token(),
    };
    g.resolve_effect(&mint, &ctx).unwrap();
    let treasures_before = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure))
        .count();
    assert_eq!(treasures_before, 5);
    let dragon = g.add_card_to_library(0, catalog::balefire_dragon());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(dragon))]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: magda, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Magda's five-Treasure tutor activates");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "the tutored Dragon enters the battlefield");
    let treasures_after = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure))
        .count();
    assert_eq!(treasures_after, 0, "all five Treasures are sacrificed as the cost");
}

#[test]
fn magda_cannot_tutor_without_five_treasures() {
    use crabomination::effect::{PlayerRef, Value};
    let mut g = two_player_game();
    let magda = g.add_card_to_battlefield(0, catalog::magda_brazen_outlaw());
    g.clear_sickness(magda);
    let ctx = crabomination::game::effects::EffectContext::for_trigger(magda, 0, None, 0);
    g.resolve_effect(&Effect::CreateToken {
        who: PlayerRef::You, count: Value::Const(4),
        definition: crabomination::game::effects::treasure_token(),
    }, &ctx).unwrap();
    g.add_card_to_library(0, catalog::balefire_dragon());
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: magda, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "four Treasures isn't enough to pay the sacrifice cost");
}

#[test]
fn robber_of_the_rich_has_reach_and_haste() {
    let card = catalog::robber_of_the_rich();
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);
    assert!(card.keywords.contains(&crabomination::card::Keyword::Reach));
    assert!(card.keywords.contains(&crabomination::card::Keyword::Haste));
}

#[test]
fn robber_of_the_rich_exiles_top_when_defender_has_more_cards() {
    let mut g = two_player_game();
    let robber = g.add_card_to_battlefield(0, catalog::robber_of_the_rich());
    g.clear_sickness(robber);
    // Defender (p1) holds 2 cards; attacker (p0) holds 0 — condition met.
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: robber, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == top);
    assert!(exiled.is_some(), "the top card is exiled by Robber's attack trigger");
    assert!(exiled.unwrap().may_play_until.is_some(),
        "the exiled card carries a may-play permission");
}

#[test]
fn robber_of_the_rich_no_exile_when_defender_has_fewer_cards() {
    let mut g = two_player_game();
    let robber = g.add_card_to_battlefield(0, catalog::robber_of_the_rich());
    g.clear_sickness(robber);
    g.add_card_to_hand(0, catalog::island()); // attacker has more — no exile
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: robber, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert!(!g.exile.iter().any(|c| c.id == top),
        "no exile when the defender doesn't have more cards in hand");
}

// ── Push XXIV: 3 more body stubs ──────────────────────────────────────────

#[test]
fn phyrexian_revoker_is_a_two_one_phyrexian_construct() {
    let card = catalog::phyrexian_revoker();
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 1);
    assert!(card.card_types.contains(&crabomination::card::CardType::Artifact));
    assert!(card.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(card.subtypes.creature_types.contains(&crabomination::card::CreatureType::Phyrexian));
    assert!(card.subtypes.creature_types.contains(&crabomination::card::CreatureType::Horror));
}

#[test]
fn solemn_simulacrum_etb_may_search_for_basic_land() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed P0's library with a Forest to tutor for.
    let forest_id = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::solemn_simulacrum());
    g.players[0].mana_pool.add_colorless(4);
    // Accept both MayDo (search) and the eventual Search decision.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest_id)),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Solemn Simulacrum castable for {4}");
    drain_stack(&mut g);

    // Solemn Simulacrum + Forest should both be on the battlefield.
    let sim = g.battlefield.iter().find(|c| c.id == id).expect("Simulacrum on bf");
    assert_eq!(sim.definition.power, 2);
    let forest_view = g.battlefield.iter().find(|c| c.id == forest_id)
        .expect("Forest should be tutored to battlefield");
    assert!(forest_view.tapped, "Forest enters tapped");
}

#[test]
fn solemn_simulacrum_has_dies_draw_trigger() {
    let card = catalog::solemn_simulacrum();
    assert_eq!(card.triggered_abilities.len(), 2,
        "Solemn Simulacrum should have ETB + dies triggers");
    assert!(card.subtypes.creature_types.contains(&crabomination::card::CreatureType::Golem));
}

#[test]
fn inquisitive_puppet_etb_scrys_one() {
    let mut g = two_player_game();
    // Seed library with a card so Scry has something to look at.
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inquisitive_puppet());
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquisitive Puppet castable for {1}");
    drain_stack(&mut g);

    // Puppet on battlefield.
    let puppet = g.battlefield.iter().find(|c| c.id == id).expect("puppet on bf");
    assert_eq!(puppet.definition.power, 0);
    assert_eq!(puppet.definition.toughness, 2);
}


// ── modern_decks: sac-a-Blood activated ability (Bloodtithe Harvester) ──────

#[test]
fn bloodtithe_harvester_sacs_blood_to_deal_two_damage() {
    use crabomination::game::effects::blood_token;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bh = g.add_card_to_battlefield(0, catalog::bloodtithe_harvester());
    g.clear_sickness(bh);
    // Give the controller a Blood token to feed the sacrifice cost.
    let blood = g.add_token_to_battlefield(0, &blood_token());
    g.players[0].mana_pool.add_colorless(1);
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bh,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("{1}, Sacrifice a Blood: 2 damage activates");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "deals 2 to the targeted player");
    assert!(
        !g.battlefield.iter().any(|c| c.id == blood),
        "the Blood token was sacrificed to pay the cost",
    );
}

#[test]
fn bloodtithe_harvester_cannot_activate_without_a_blood() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bh = g.add_card_to_battlefield(0, catalog::bloodtithe_harvester());
    g.clear_sickness(bh);
    g.players[0].mana_pool.add_colorless(1);
    // No Blood token on the battlefield → the sac cost cannot be paid.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: bh,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: Vec::new(),
        x_value: None,
    });
    assert!(res.is_err(), "no Blood to sacrifice → activation rejected");
}

// ── modern_decks: Tireless Tracker sac-a-Clue counter trigger ───────────────

#[test]
fn tireless_tracker_gains_counter_when_a_clue_is_sacrificed() {
    use crabomination::game::effects::clue_token;
    let mut g = two_player_game();
    let tracker = g.add_card_to_battlefield(0, catalog::tireless_tracker());
    g.clear_sickness(tracker);
    let clue = g.add_token_to_battlefield(0, &clue_token());
    // Clue's ability: {2}, Sacrifice this artifact: Draw a card.
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: clue,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("Clue sacrifices for {2}");
    drain_stack(&mut g);
    assert!(
        !g.battlefield.iter().any(|c| c.id == clue),
        "Clue was sacrificed",
    );
    assert_eq!(
        g.battlefield_find(tracker).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "sacrificing a Clue puts a +1/+1 counter on Tireless Tracker",
    );
}


// ── modern_decks: Sentinel of the Nameless City Ward {2} (CR 702.21) ────────

#[test]
fn sentinel_of_the_nameless_city_ward_counters_unpaid_spell() {
    use crabomination::game::types::{Target, TurnStep};
    let mut g = two_player_game();
    let sentinel = g.add_card_to_battlefield(0, catalog::sentinel_of_the_nameless_city());
    g.clear_sickness(sentinel);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // Only {R} for the bolt — nothing left for Ward {2}.
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(sentinel)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt casts; Ward is a trigger, not a cast restriction");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.id == sentinel),
        "Ward 2 counters the unpaid Bolt; Sentinel survives",
    );
}

#[test]
fn sentinel_of_the_nameless_city_is_a_merfolk_warrior_scout() {
    use crabomination::card::CreatureType;
    let s = catalog::sentinel_of_the_nameless_city();
    assert!(s.has_creature_type(CreatureType::Merfolk));
    assert!(s.has_creature_type(CreatureType::Warrior));
    assert!(s.has_creature_type(CreatureType::Scout));
}

#[test]
fn sylvan_safekeeper_cannot_activate_without_a_forest() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let sk = g.add_card_to_battlefield(0, catalog::sylvan_safekeeper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(sk);
    // No Forest to sacrifice → activation rejected pre-resolution.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: sk,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(),
        x_value: None,
    });
    assert!(res.is_err(), "no Forest to sacrifice → activation rejected");
    use crabomination::card::Keyword;
    let computed = g.compute_battlefield();
    let view = computed.iter().find(|c| c.id == bear).unwrap();
    assert!(!view.keywords.contains(&Keyword::Shroud), "no shroud granted");
}

#[test]
fn zuran_orb_cannot_activate_without_a_land() {
    let mut g = two_player_game();
    let orb = g.add_card_to_battlefield(0, catalog::zuran_orb());
    g.clear_sickness(orb);
    let life_before = g.players[0].life;
    // No land to sacrifice → activation rejected pre-resolution.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: orb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(res.is_err(), "no land to sacrifice → activation rejected");
    assert_eq!(g.players[0].life, life_before, "no life gained when cost unpayable");
}

// ─────────────────────────────────────────────────────────────────────────
// Delve (CR 702.66) — graveyard cards pay {1} of the generic cost.
// ─────────────────────────────────────────────────────────────────────────

/// Treasure Cruise with seven graveyard cards delved away costs just {U}
/// and exiles those seven cards.
#[test]
fn delve_treasure_cruise_pays_generic_with_graveyard() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    // Seven cards in the graveyard to delve.
    let gy: Vec<_> = (0..7).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let id = g.add_card_to_hand(0, catalog::treasure_cruise());
    // Only one blue mana — the {7} generic must be paid entirely by delve.
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    let exile_before = g.exile.len();

    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy.clone(),
    }).expect("Treasure Cruise castable for U after delving seven");
    drain_stack(&mut g);

    // Net hand: -1 (cast) +3 (draw) = +2.
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
    let _ = gy_before;
    // None of the seven delved cards remain in the graveyard (the resolved
    // Cruise itself lands there, so the raw count isn't zero).
    assert!(gy.iter().all(|id| !g.players[0].graveyard.iter().any(|c| c.id == *id)),
        "delved cards left the graveyard");
    assert_eq!(g.exile.len(), exile_before + 7, "delved cards moved to exile");
    assert_eq!(g.players[0].cards_exiled_this_turn, 7);
}

/// Partial delve: exiling three cards reduces {7}{U} to {4}{U}.
#[test]
fn delve_partial_reduces_generic_portion() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let gy: Vec<_> = (0..3).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let id = g.add_card_to_hand(0, catalog::treasure_cruise());
    // {4} generic + {U} after delving 3.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy.clone(),
    }).expect("4U after delving three");
    drain_stack(&mut g);
    assert!(gy.iter().all(|id| !g.players[0].graveyard.iter().any(|c| c.id == *id)),
        "all three delved out of gy");
    assert_eq!(g.players[0].mana_pool.total(), 0, "exact mana consumed");
}

/// Delve can't reduce the colored pip: with no mana, even a full delve
/// leaves the {U} unpayable and the cast is rejected (card returns to hand,
/// graveyard untouched).
#[test]
fn delve_cannot_pay_colored_pip() {
    let mut g = two_player_game();
    let gy: Vec<_> = (0..7).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let id = g.add_card_to_hand(0, catalog::treasure_cruise());
    // No mana at all — the {U} can't be paid by delve.
    let res = g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy.clone(),
    });
    assert!(res.is_err(), "the colored pip cannot be delved away");
    assert!(g.players[0].has_in_hand(id), "card returns to hand on failed cast");
    assert_eq!(g.players[0].graveyard.len(), 7, "graveyard untouched on failed cast");
    assert_eq!(g.exile.len(), 0, "no cards exiled on failed cast");
}

/// Delving a card that isn't in the caster's graveyard is rejected.
#[test]
fn delve_rejects_card_not_in_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::treasure_cruise());
    let bogus = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let res = g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: vec![bogus],
    });
    assert!(matches!(res, Err(GameError::CardNotInGraveyard(_))));
}

/// Delve listed on a non-Delve spell is rejected.
#[test]
fn delve_rejects_spell_without_keyword() {
    let mut g = two_player_game();
    let gy = g.add_card_to_graveyard(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let res = g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None, delve_cards: vec![gy],
    });
    assert!(res.is_err(), "Lightning Bolt has no Delve");
}

/// Murderous Cut delves to {B} and destroys a creature.
#[test]
fn delve_murderous_cut_kills_for_one_black() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gy: Vec<_> = (0..4).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let id = g.add_card_to_hand(0, catalog::murderous_cut());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy,
    }).expect("{B} after delving four");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
}

/// Gurmag Angler — Delve on a creature spell exiles the delve cards as part
/// of paying its cost, landing a 5/5 on the battlefield.
#[test]
fn delve_gurmag_angler_enters_as_five_five() {
    let mut g = two_player_game();
    let gy: Vec<_> = (0..6).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let id = g.add_card_to_hand(0, catalog::gurmag_angler());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy.clone(),
    }).expect("{B} after delving six");
    drain_stack(&mut g);
    let angler = g.battlefield.iter().find(|c| c.definition.name == "Gurmag Angler")
        .expect("Angler resolved onto the battlefield");
    assert_eq!((angler.power(), angler.toughness()), (5, 5));
    assert!(gy.iter().all(|gid| g.exile.iter().any(|c| c.id == *gid)), "delve cards exiled");
}

/// Dig Through Time delves to {U}{U} and draws two off a Scry 7.
#[test]
fn delve_dig_through_time_draws_two() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let gy: Vec<_> = (0..6).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let id = g.add_card_to_hand(0, catalog::dig_through_time());
    g.players[0].mana_pool.add(Color::Blue, 2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy.clone(),
    }).expect("UU after delving six");
    drain_stack(&mut g);
    // Net hand: -1 (cast) +2 (draw) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert!(gy.iter().all(|id| !g.players[0].graveyard.iter().any(|c| c.id == *id)),
        "six delved out of gy");
}

// ─────────────────────────────────────────────────────────────────────────
// Regeneration (CR 701.15) — a shield replaces the next destruction.
// ─────────────────────────────────────────────────────────────────────────

/// A regeneration shield saves a creature from `Effect::Destroy`: it taps,
/// heals, and survives, and the shield is consumed (a second destroy kills).
#[test]
fn regen_shield_replaces_destroy() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(0, catalog::drudge_skeletons());
    g.clear_sickness(skel);
    g.players[0].mana_pool.add(Color::Black, 1);
    // Stamp a regeneration shield via the {B}: Regenerate ability.
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate activates");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(skel).unwrap().regeneration_shields, 1);

    // Destroy it — the shield replaces the destruction. Murderous Cut has
    // no color restriction (Doom Blade can't target a black creature).
    let cut = g.add_card_to_hand(1, catalog::murderous_cut());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: cut, target: Some(Target::Permanent(skel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Murderous Cut cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(skel).expect("Skeletons survived via regen");
    assert!(c.tapped, "regenerated creature is tapped");
    assert_eq!(c.regeneration_shields, 0, "shield consumed");
}

/// A regeneration shield replaces lethal combat damage: the blocker taps,
/// heals, and stays on the battlefield.
#[test]
fn regen_shield_replaces_lethal_combat_damage() {
    let mut g = two_player_game();
    // Bob's Skeletons block Alice's bear; bear deals 2, lethal to the 1/1.
    let skel = g.add_card_to_battlefield(1, catalog::drudge_skeletons());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.players[1].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate");
    drain_stack(&mut g);

    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("bear attacks");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(skel, bear)])).expect("skel blocks");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    let c = g.battlefield_find(skel).expect("Skeletons regenerated out of combat");
    assert_eq!(c.regeneration_shields, 0, "shield consumed by lethal damage");
    assert!(c.tapped, "regenerated creature is tapped");
    assert_eq!(c.damage, 0, "marked damage healed");
}

/// Regeneration does NOT save a creature whose toughness is reduced to 0
/// (CR 701.15e — that's not destruction).
#[test]
fn regen_does_not_save_zero_toughness() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(0, catalog::drudge_skeletons());
    g.clear_sickness(skel);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    // Drop toughness to 0 with two -1/-1 counters.
    if let Some(c) = g.battlefield_find_mut(skel) {
        c.add_counters(CounterType::MinusOneMinusOne, 1);
    }
    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(skel).is_none(), "0-toughness death bypasses regeneration");
}

/// Regeneration shields expire at end of turn (CR 701.15g).
#[test]
fn regen_shield_expires_at_cleanup() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(0, catalog::drudge_skeletons());
    g.clear_sickness(skel);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(skel).unwrap().regeneration_shields, 1);
    // End-of-turn cleanup clears the shield.
    if let Some(c) = g.battlefield_find_mut(skel) {
        c.clear_end_of_turn_effects();
    }
    assert_eq!(g.battlefield_find(skel).unwrap().regeneration_shields, 0);
}

// ─────────────────────────────────────────────────────────────────────────
// Can't be regenerated (CR 701.15g) — DestroyNoRegen bypasses the shield.
// ─────────────────────────────────────────────────────────────────────────

/// A regeneration shield does NOT save a creature from a "can't be
/// regenerated" destroy effect like Terminate (CR 701.15g).
#[test]
fn terminate_ignores_regeneration_shield() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(0, catalog::drudge_skeletons());
    g.clear_sickness(skel);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate activates");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(skel).unwrap().regeneration_shields, 1,
        "shield is up before Terminate");

    // Terminate destroys it and it can't be regenerated — the shield does
    // not save it.
    let term = g.add_card_to_hand(1, catalog::terminate());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: term, target: Some(Target::Permanent(skel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Terminate cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(skel).is_none(),
        "Skeletons destroyed despite the regeneration shield (can't be regenerated)");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == skel),
        "Skeletons hit the graveyard");
}

/// Plain `Destroy` (no can't-regen clause) still honors a shield, proving
/// the distinction is real and not just "all destroys ignore shields now".
#[test]
fn plain_destroy_still_honors_regeneration_shield() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(0, catalog::drudge_skeletons());
    g.clear_sickness(skel);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate activates");
    drain_stack(&mut g);

    // Murderous Cut is a plain Destroy — the shield saves the creature.
    let cut = g.add_card_to_hand(1, catalog::murderous_cut());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: cut, target: Some(Target::Permanent(skel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Murderous Cut cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(skel).expect("Skeletons survive plain Destroy via shield");
    assert_eq!(c.regeneration_shields, 0, "shield consumed");
}

// ─────────────────────────────────────────────────────────────────────────
// Intimidate (CR 702.13) — shares-a-color check uses computed colors, so a
// color from a hybrid pip counts (regression for the raw-pip-scan bug).
// ─────────────────────────────────────────────────────────────────────────

/// Spectacle Mage's colors (blue + red) come entirely from its {U/R}{U/R}
/// hybrid pips. With Intimidate, a red creature shares red and CAN block;
/// a green creature can't. Previously the shares-a-color check only
/// scanned `{C}` cost pips and would have wrongly treated the hybrid-only
/// attacker as colorless (blockable by nothing but artifacts).
#[test]
fn intimidate_shares_color_counts_hybrid_pip_color() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::spectacle_mage());
    // Grant Intimidate to the attacker.
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(mage).unwrap().definition).keywords.push(Keyword::Intimidate);
    g.clear_sickness(mage);
    let goblin = g.add_card_to_battlefield(1, catalog::goblin_guide()); // red
    // Spectacle Mage flies, so give the blocker reach to isolate the Intimidate
    // colour check (this test is about colour-sharing, not evasion).
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(goblin).unwrap().definition).keywords.push(Keyword::Reach);

    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: mage, target: AttackTarget::Player(1),
    }])).expect("Spectacle Mage attacks");
    g.step = TurnStep::DeclareBlockers;
    // Red goblin shares red (from the hybrid pip) → legal block.
    assert!(g.blocker_can_block_attacker(goblin, mage),
        "red creature can block a red/blue Intimidate attacker (shared color)");
}

#[test]
fn intimidate_off_color_creature_cannot_block_hybrid_attacker() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::spectacle_mage()); // U/R
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(mage).unwrap().definition).keywords.push(Keyword::Intimidate);
    g.clear_sickness(mage);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green

    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: mage, target: AttackTarget::Player(1),
    }])).expect("attacks");
    g.step = TurnStep::DeclareBlockers;
    let res = g.perform_action(GameAction::DeclareBlockers(vec![(bear, mage)]));
    assert!(matches!(res, Err(GameError::CannotBlock(_))),
        "green creature shares no color with a U/R Intimidate attacker");
}

// ─────────────────────────────────────────────────────────────────────────
// Fear (CR 702.36) — only artifact and/or black creatures can block.
// ─────────────────────────────────────────────────────────────────────────

/// A non-black, non-artifact creature can't block a Fear attacker.
#[test]
fn fear_cannot_be_blocked_by_green_creature() {
    let mut g = two_player_game();
    let legion = g.add_card_to_battlefield(0, catalog::severed_legion());
    g.clear_sickness(legion);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: legion, target: AttackTarget::Player(1),
    }])).expect("Legion attacks");
    g.step = TurnStep::DeclareBlockers;
    let res = g.perform_action(GameAction::DeclareBlockers(vec![(bear, legion)]));
    assert!(matches!(res, Err(GameError::CannotBlock(_))), "green bear can't block Fear");
}

/// A black creature CAN block a Fear attacker.
#[test]
fn fear_can_be_blocked_by_black_creature() {
    let mut g = two_player_game();
    let legion = g.add_card_to_battlefield(0, catalog::severed_legion());
    g.clear_sickness(legion);
    let skel = g.add_card_to_battlefield(1, catalog::drudge_skeletons()); // black
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: legion, target: AttackTarget::Player(1),
    }])).expect("Legion attacks");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(skel, legion)]))
        .expect("black Skeletons may block a Fear attacker");
}

/// Hooting Mandrills delves to {G} and enters as a 4/4 trampler.
#[test]
fn delve_hooting_mandrills_enters_with_trample() {
    let mut g = two_player_game();
    let gy: Vec<_> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::forest())).collect();
    let id = g.add_card_to_hand(0, catalog::hooting_mandrills());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy,
    }).expect("{G} after delving five");
    drain_stack(&mut g);
    let mand = g.battlefield.iter().find(|c| c.definition.name == "Hooting Mandrills").unwrap();
    assert_eq!((mand.power(), mand.toughness()), (4, 4));
    assert!(mand.has_keyword(&Keyword::Trample));
}

/// Become Immense delves to {G} and pumps a creature +6/+6.
#[test]
fn delve_become_immense_pumps_six() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gy: Vec<_> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::forest())).collect();
    let id = g.add_card_to_hand(0, catalog::become_immense());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy,
    }).expect("{G} after delving five");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (8, 8), "2/2 + 6/6");
}

/// Tombstalker delves to {B}{B} and enters as a 5/5 flier.
#[test]
fn delve_tombstalker_enters_five_five_flying() {
    let mut g = two_player_game();
    let gy: Vec<_> = (0..6).map(|_| g.add_card_to_graveyard(0, catalog::swamp())).collect();
    let id = g.add_card_to_hand(0, catalog::tombstalker());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpellDelve {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: gy,
    }).expect("BB after delving six");
    drain_stack(&mut g);
    let t = g.battlefield.iter().find(|c| c.definition.name == "Tombstalker").unwrap();
    assert_eq!((t.power(), t.toughness()), (5, 5));
    assert!(t.has_keyword(&Keyword::Flying));
}

/// Wall of Bone regenerates from lethal combat damage and stays a Defender.
#[test]
fn wall_of_bone_regenerates_from_combat() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_bone());
    let big = g.add_card_to_battlefield(0, catalog::gurmag_angler()); // 5/5
    g.clear_sickness(big);
    g.players[1].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: big, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, big)])).expect("wall blocks");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat");
    let c = g.battlefield_find(wall).expect("Wall regenerated");
    assert_eq!(c.damage, 0, "marked damage healed");
    assert!(c.has_keyword(&Keyword::Defender));
}

/// Will-o'-the-Wisp regenerates from a Destroy, staying on the battlefield
/// tapped.
#[test]
fn will_o_the_wisp_regenerates_from_destroy() {
    let mut g = two_player_game();
    let wisp = g.add_card_to_battlefield(0, catalog::will_o_the_wisp());
    g.clear_sickness(wisp);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wisp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    // Opponent destroys it.
    let cut = g.add_card_to_hand(1, catalog::murderous_cut());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: cut, target: Some(Target::Permanent(wisp)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Murderous Cut");
    drain_stack(&mut g);
    let c = g.battlefield_find(wisp).expect("Wisp survived via regen");
    assert!(c.tapped);
    assert_eq!(c.regeneration_shields, 0);
}

// ─────────────────────────────────────────────────────────────────────────
// Indestructible counters (CR 122.1 / 702.12) + Zopandrel's activation.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn zopandrel_activation_sacs_two_creatures_and_adds_indestructible_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let zop = g.add_card_to_battlefield(0, catalog::zopandrel_hunger_dominus());
    g.clear_sickness(zop);
    let fodder1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fodder2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Pay {G/P}{G/P} with two green.
    g.players[0].mana_pool.add(Color::Green, 2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: zop, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Zopandrel activation: {G}{G} + sac two creatures");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == fodder1),
        "first fodder creature sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder2),
        "second fodder creature sacrificed");
    let z = g.battlefield_find(zop).expect("Zopandrel still on bf");
    assert_eq!(z.counter_count(CounterType::Indestructible), 1,
        "Zopandrel gains an indestructible counter");
}

#[test]
fn zopandrel_doubles_each_creatures_power_and_toughness_at_combat() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zopandrel_hunger_dominus()); // 4/6
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let b = view.iter().find(|c| c.id == bear).expect("bear present");
    // True doubling: 2/2 → 4/4 (not a flat +4/+4, which would be 6/6).
    assert_eq!((b.power, b.toughness), (4, 4), "bear's P/T doubled");
}

#[test]
fn zopandrel_activation_rejected_without_two_other_creatures() {
    let mut g = two_player_game();
    let zop = g.add_card_to_battlefield(0, catalog::zopandrel_hunger_dominus());
    g.clear_sickness(zop);
    // Only one other creature — can't pay the "sacrifice two" cost.
    let _only = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);

    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: zop, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    });
    assert!(res.is_err(), "activation rejected without two other creatures to sac");
}

#[test]
fn indestructible_counter_survives_destroy() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::Indestructible, 1);
    }
    // Wrath of God (DestroyNoRegen) shouldn't kill an indestructible-countered
    // creature — can't-be-regenerated bypasses regen, not indestructibility.
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: wrath, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wrath of God castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "creature with an indestructible counter survives Wrath of God");
}

#[test]
fn indestructible_counter_survives_lethal_combat_damage() {
    use crabomination::card::CounterType;
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    if let Some(c) = g.battlefield_find_mut(blocker) {
        c.add_counters(CounterType::Indestructible, 1);
    }
    let attacker = g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attacks");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("blocks");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    assert!(g.battlefield.iter().any(|c| c.id == blocker),
        "indestructible-countered blocker survives lethal combat damage");
}

// ════════════════════════════════════════════════════════════════════════════
// Coverage backfill (claude/modern_decks): functionality tests for modern-deck
// cards (creatures, spells, and dual/shock/fast/surveil/pathway lands) that
// were wired but lacked a dedicated test.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn wall_of_blossoms_etb_draws_and_is_a_zero_four_defender() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wall_of_blossoms());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wall of Blossoms castable for {1}{G}");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.definition.power, c.definition.toughness), (0, 4));
    assert!(c.definition.keywords.contains(&crabomination::card::Keyword::Defender));
    assert_eq!(g.players[0].library.len(), lib_before - 1, "ETB drew a card");
}

#[test]
fn monastery_swiftspear_prowess_pumps_on_instant() {
    let mut g = two_player_game();
    let spear = g.add_card_to_battlefield(0, catalog::monastery_swiftspear());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let c = g.computed_permanent(spear).unwrap();
    assert_eq!(c.power, 2, "Prowess: 1/2 base +1/+1 = 2 power on a noncreature cast");
}

#[test]
fn relic_of_progenitus_exiles_opponent_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let relic = g.add_card_to_battlefield(0, catalog::relic_of_progenitus());
    g.clear_sickness(relic);
    g.perform_action(GameAction::ActivateAbility {
        card_id: relic, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("Relic {T}: exile a card from each opponent's graveyard");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty(),
        "opponent's graveyard exiled by Relic of Progenitus");
}

#[test]
fn stonecoil_serpent_enters_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::stonecoil_serpent());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("Stonecoil Serpent castable for X=3");
    drain_stack(&mut g);
    let c = g.computed_permanent(id).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "X=3 → three +1/+1 counters");
    assert!(c.keywords.contains(&crabomination::card::Keyword::Trample));
    assert!(c.keywords.contains(&crabomination::card::Keyword::Reach));
    assert!(c.keywords.contains(&crabomination::card::Keyword::ProtectionFromMulticolored));
}

/// CR 702.16 — protection from multicolored: a two-color spell can't target
/// Stonecoil Serpent, but a mono-color spell can.
#[test]
fn stonecoil_serpent_protection_from_multicolored() {
    let mut g = two_player_game();
    // X=3 body so it survives long enough to be a target.
    let snake = g.add_card_to_battlefield(0, catalog::stonecoil_serpent());
    g.battlefield_find_mut(snake).unwrap()
        .add_counters(crabomination::card::CounterType::PlusOnePlusOne, 3);
    // Terminate ({B}{R}) is multicolored → illegal target.
    let term = g.add_card_to_hand(1, catalog::terminate());
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: term, target: Some(Target::Permanent(snake)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "a multicolored spell can't target protection-from-multicolored");
    // A mono-color spell (Doom Blade-style — use Dark Banishing? use Terror) can.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(snake)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_ok(), "a mono-color spell may target it");
}

#[test]
fn decree_of_justice_makes_x_angels() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::decree_of_justice());
    // {X}{X}{2}{W}{W} with X=2 → pay 2+2+2 generic + WW.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Decree of Justice castable for X=2");
    drain_stack(&mut g);
    let angels: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Angel" && c.controller == 0).collect();
    assert_eq!(angels.len(), 2, "X=2 → two Angel tokens");
    assert!(angels[0].definition.keywords.contains(&crabomination::card::Keyword::Flying));
}

#[test]
fn spell_queller_etb_counters_a_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    g.priority.player_with_priority = 0;
    let queller = g.add_card_to_hand(0, catalog::spell_queller());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: queller, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Spell Queller castable at flash speed");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the bolt was countered by Spell Queller's ETB");
    assert!(g.battlefield_find(queller).is_some(), "Queller resolved onto the battlefield");
}

