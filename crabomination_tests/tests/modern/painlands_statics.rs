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

// ── Painlands / rainbow lands / mana rocks (cube staples) ────────────────────

/// Each painland taps for {C} (no damage) and for either color, dealing 1.
#[test]
fn painlands_tap_for_colors_and_ping() {
    let cases: &[(Factory, Color, Color)] = &[
        (catalog::adarkar_wastes, Color::White, Color::Blue),
        (catalog::underground_river, Color::Blue, Color::Black),
        (catalog::sulfurous_springs, Color::Black, Color::Red),
        (catalog::karplusan_forest, Color::Red, Color::Green),
        (catalog::brushland, Color::Green, Color::White),
        (catalog::caves_of_koilos, Color::White, Color::Black),
        (catalog::shivan_reef, Color::Blue, Color::Red),
        (catalog::llanowar_wastes, Color::Black, Color::Green),
        (catalog::yavimaya_coast, Color::Green, Color::Blue),
        (catalog::battlefield_forge, Color::Red, Color::White),
    ];
    for (make, c1, _c2) in cases {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, make());
        g.clear_sickness(land);
        // Colorless ability (index 0): no damage.
        let life = g.players[0].life;
        g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap C");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
        assert_eq!(g.players[0].life, life, "{{C}} ability deals no damage");
        // Untap and use the first colored ability (index 1): pings for 1.
        g.battlefield_find_mut(land).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap color");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.amount(*c1), 1);
        assert_eq!(g.players[0].life, life - 1, "colored ability deals 1 damage");
    }
}

/// City of Brass taps for any color and pings its controller for 1.
#[test]
fn city_of_brass_taps_any_color_with_ping() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::city_of_brass());
    g.clear_sickness(land);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "produces one mana");
    assert_eq!(g.players[0].life, life - 1, "deals 1 damage to you");
}

/// Mana Confluence pays 1 life for one mana of any color.
#[test]
fn mana_confluence_pays_life_for_any_color() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::mana_confluence());
    g.clear_sickness(land);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
    assert_eq!(g.players[0].life, life - 1, "pays 1 life");
}

/// Ghost Quarter sacrifices to destroy a target land.
#[test]
fn ghost_quarter_destroys_target_land() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let gq = g.add_card_to_battlefield(0, catalog::ghost_quarter());
    let victim = g.add_card_to_battlefield(1, catalog::island());
    g.clear_sickness(gq);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gq, ability_index: 1, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate sac-destroy");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != victim), "target land destroyed");
    assert!(g.battlefield.iter().all(|c| c.id != gq), "Ghost Quarter sacrificed");
}

/// Thran Dynamo / Ur-Golem's Eye / Dreamstone Hedron tap for colorless burst.
#[test]
fn colorless_rocks_tap_for_burst() {
    for (make, n) in [
        (catalog::thran_dynamo as fn() -> crabomination::card::CardDefinition, 3u32),
        (catalog::ur_golems_eye, 2),
        (catalog::dreamstone_hedron, 3),
    ] {
        let mut g = two_player_game();
        let rock = g.add_card_to_battlefield(0, make());
        g.clear_sickness(rock);
        g.perform_action(GameAction::ActivateAbility { card_id: rock, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.colorless_amount(), n);
    }
}

/// Prismatic Lens taps for {C} on its first ability.
#[test]
fn prismatic_lens_taps_for_colorless() {
    let mut g = two_player_game();
    let lens = g.add_card_to_battlefield(0, catalog::prismatic_lens());
    g.clear_sickness(lens);
    g.perform_action(GameAction::ActivateAbility { card_id: lens, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
}

/// Gilded Lotus taps for three mana of one color.
#[test]
fn gilded_lotus_taps_for_three_of_one_color() {
    let mut g = two_player_game();
    let lotus = g.add_card_to_battlefield(0, catalog::gilded_lotus());
    g.clear_sickness(lotus);
    g.perform_action(GameAction::ActivateAbility { card_id: lotus, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Commander's Sphere can be sacrificed to draw a card.
#[test]
fn commanders_sphere_sacrifices_to_draw() {
    let mut g = two_player_game();
    let sphere = g.add_card_to_battlefield(0, catalog::commanders_sphere());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility { card_id: sphere, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("sac to draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(g.battlefield.iter().all(|c| c.id != sphere), "sphere sacrificed");
}

/// Temple Bell makes every player draw a card.
#[test]
fn temple_bell_each_player_draws() {
    let mut g = two_player_game();
    let bell = g.add_card_to_battlefield(0, catalog::temple_bell());
    g.clear_sickness(bell);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::island());
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.perform_action(GameAction::ActivateAbility { card_id: bell, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("ring");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 1);
    assert_eq!(g.players[1].hand.len(), h1 + 1);
}

/// Wayfarer's Bauble fetches a basic land onto the battlefield tapped.
#[test]
fn wayfarers_bauble_fetches_basic_land() {
    let mut g = two_player_game();
    let bauble = g.add_card_to_battlefield(0, catalog::wayfarers_bauble());
    g.clear_sickness(bauble);
    let forest = g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    g.perform_action(GameAction::ActivateAbility { card_id: bauble, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("crack bauble");
    drain_stack(&mut g);
    let lands_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands_after, lands + 1, "a basic land entered");
    assert!(g.battlefield_find(forest).is_some_and(|c| c.tapped), "fetched land enters tapped");
}

/// Guardian Idol can animate into a 2/2 Golem.
#[test]
fn guardian_idol_animates_to_golem() {
    let mut g = two_player_game();
    let idol = g.add_card_to_battlefield(0, catalog::guardian_idol());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: idol, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(idol).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Batterskull's living weapon mints a Germ and attaches, making a 4/4.
#[test]
fn batterskull_living_weapon_makes_four_four() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::batterskull());
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, id);
    let germ = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Phyrexian Germ")
        .expect("Germ token created");
    let id = germ.id;
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "Germ is a 4/4 while equipped");
    assert!(cp.keywords.contains(&Keyword::Vigilance) && cp.keywords.contains(&Keyword::Lifelink));
}

/// Simple stat/keyword Equipment grants its bonus and keywords when attached.
#[test]
fn simple_equipment_grants_bonus_and_keywords() {
    use crabomination::card::Keyword;
    let cases: &[(Factory, i32, i32, &[Keyword])] = &[
        (catalog::swiftfoot_boots, 0, 0, &[Keyword::Hexproof, Keyword::Haste]),
        (catalog::vulshok_morningstar, 2, 2, &[]),
        (catalog::bone_saw, 1, 0, &[]),
        (catalog::accorders_shield, 0, 3, &[Keyword::Vigilance]),
        (catalog::strider_harness, 1, 1, &[Keyword::Haste]),
        (catalog::loxodon_warhammer, 3, 0, &[Keyword::Trample, Keyword::Lifelink]),
        (catalog::sword_of_vengeance, 2, 0, &[Keyword::FirstStrike, Keyword::Vigilance, Keyword::Trample, Keyword::Haste]),
        (catalog::fireshrieker, 0, 0, &[Keyword::DoubleStrike]),
        (catalog::whispersilk_cloak, 0, 0, &[Keyword::Unblockable, Keyword::Shroud]),
        (catalog::darksteel_plate, 0, 0, &[Keyword::Indestructible]),
    ];
    for (make, dp, dt, kws) in cases {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let eq = g.add_card_to_battlefield(0, make());
        g.battlefield_find_mut(eq).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (2 + dp, 2 + dt),
            "{} grants +{}/+{}", g.battlefield_find(eq).unwrap().definition.name, dp, dt);
        for kw in *kws {
            assert!(cp.keywords.contains(kw), "equipped creature gains {kw:?}");
        }
    }
}

/// Darksteel Plate is itself indestructible.
#[test]
fn darksteel_plate_is_indestructible() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let plate = g.add_card_to_battlefield(0, catalog::darksteel_plate());
    assert!(g.computed_permanent(plate).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Rogue's Gloves draws a card when its equipped creature connects.
#[test]
fn rogues_gloves_draws_on_combat_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // opp has no blockers
    let gloves = g.add_card_to_battlefield(0, catalog::rogues_gloves());
    g.battlefield_find_mut(gloves).unwrap().attached_to = Some(attacker);
    g.add_card_to_library(0, catalog::island());
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.players[0].hand.len() > hand { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.players[0].hand.len() > hand, "Rogue's Gloves draws on combat damage");
}

/// Specter's Shroud forces a discard when its equipped creature connects.
#[test]
fn specters_shroud_discards_on_combat_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::looter_il_kor());
    let shroud = g.add_card_to_battlefield(0, catalog::specters_shroud());
    g.battlefield_find_mut(shroud).unwrap().attached_to = Some(attacker);
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let opp_hand = g.players[1].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.players[1].hand.len() < opp_hand { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.players[1].hand.len() < opp_hand, "defender discards on combat damage");
}

/// Mask of Memory nets +1 card (draw two, discard one) on combat damage.
#[test]
fn mask_of_memory_draws_two_discards_one() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // opp has no blockers
    let mask = g.add_card_to_battlefield(0, catalog::mask_of_memory());
    g.battlefield_find_mut(mask).unwrap().attached_to = Some(attacker);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.players[0].hand.len() > hand { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "draw two, discard one → net +1");
}

#[test]
fn careful_study_draws_two_discards_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::careful_study());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let before = g.players[0].hand.len(); // includes Careful Study itself
    cast(&mut g, id);
    drain_stack(&mut g);
    // -1 (cast) +2 (draw) -2 (discard) = net -1 vs before.
    assert_eq!(g.players[0].hand.len(), before - 1);
    assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count() >= 1
        || g.players[0].graveyard.len() >= 2, "discarded two cards");
}

#[test]
fn ancient_stirrings_finds_a_colorless_card() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let rock = g.add_card_to_library(0, catalog::mind_stone()); // colorless artifact
    g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(rock))]));
    let id = g.add_card_to_hand(0, catalog::ancient_stirrings());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, id);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == rock), "took the colorless card");
}

#[test]
fn condemn_tucks_attacker_and_gains_life() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // P1 attacker (2/2). Mark it attacking.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.attacking.push(crabomination::game::types::Attack {
        attacker: bear,
        target: crabomination::game::types::AttackTarget::Player(0),
    });
    let lib_before = g.players[1].library.len();
    let p1_life = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::condemn());
    g.players[0].mana_pool.add(Color::White, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "attacker left the battlefield");
    assert_eq!(g.players[1].library.len(), lib_before + 1, "tucked to library");
    assert_eq!(g.players[1].library.last().unwrap().id, bear, "on the bottom");
    assert_eq!(g.players[1].life, p1_life + 2, "controller gained life = toughness");
}

#[test]
fn arbor_elf_untaps_a_forest() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(forest).unwrap().tapped = true;
    let elf = g.add_card_to_battlefield(0, catalog::arbor_elf());
    g.clear_sickness(elf);
    g.perform_action(GameAction::ActivateAbility {
        card_id: elf, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(forest)), additional_targets: Vec::new(), x_value: None,
    }).expect("Arbor Elf untaps a Forest");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(forest).unwrap().tapped, "Forest untapped");
}

#[test]
fn scrapheap_scrounger_returns_itself_exiling_a_creature() {
    let mut g = two_player_game();
    let scrap = g.add_card_to_graveyard(0, catalog::scrapheap_scrounger());
    let fodder = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: scrap, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Scrapheap recursion");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == scrap), "Scrapheap returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == fodder), "exiled a creature as cost");
}

#[test]
fn mental_note_mills_two_and_draws() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let gy0 = g.players[0].graveyard.len();
    let hand0 = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::mental_note());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, id);
    drain_stack(&mut g);
    // milled two + the spell to gy = +3 graveyard; +1 hand from cast(-1)+draw(... net)
    assert!(g.players[0].graveyard.len() >= gy0 + 3, "milled two plus the spell");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "add +1, cast -1, draw +1");
}

#[test]
fn izzet_charm_burn_mode_kills_a_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::izzet_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    // Mode 1 = deal 2 damage to target creature.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Izzet Charm burn mode");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2 damage killed the 2/2");
}

#[test]
fn flame_of_anor_can_destroy_an_artifact() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::flame_of_anor());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Mode 1 = destroy target artifact.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(rock)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Flame of Anor destroy mode");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == rock), "destroyed the artifact");
}

#[test]
fn crackling_doom_pings_and_forces_a_sacrifice() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let p1 = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::crackling_doom());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, id);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 2, "each opponent took 2");
    assert!(!g.battlefield.iter().any(|c| c.id == big), "opponent sacrificed a creature");
}

#[test]
fn crackling_doom_forces_sacrifice_of_greatest_power() {
    let mut g = two_player_game();
    // Opp has a cheap big-power creature and an expensive small-power one.
    // Crackling Doom takes the greatest *power*, not greatest mana value.
    let big_power = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let small_power = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flier, MV 5
    // Make grizzly the highest power so MV vs power diverge.
    g.battlefield_find_mut(big_power).unwrap().add_counters(CounterType::PlusOnePlusOne, 5);
    let id = g.add_card_to_hand(0, catalog::crackling_doom());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, id);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == big_power), "sacrificed the 7-power creature");
    assert!(g.battlefield.iter().any(|c| c.id == small_power), "kept the higher-MV but lower-power creature");
}

#[test]
fn kroxa_hardcast_sacrifices_itself() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt()); // opp has a nonland to discard
    let id = g.add_card_to_hand(0, catalog::kroxa());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, id);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "hard-cast Kroxa sacrificed itself (didn't escape)");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Kroxa went to the graveyard");
}

#[test]
fn kroxa_escaped_stays_and_drains_on_land_discard() {
    let mut g = two_player_game();
    let kroxa = g.add_card_to_graveyard(0, catalog::kroxa());
    let fodder: Vec<_> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::lightning_bolt())).collect();
    // Opponent's only card is a land → no nonland discarded → loses 3 life.
    g.add_card_to_hand(1, catalog::tropical_island());
    let life1 = g.players[1].life;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastEscape {
        card_id: kroxa, exile_cards: fodder, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Kroxa escapable for BBRR + exile five");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == kroxa), "escaped Kroxa stays on the battlefield");
    assert_eq!(g.players[1].life, life1 - 3, "opponent discarded a land, so lost 3 life");
}

#[test]
fn kroxa_no_drain_when_nonland_discarded() {
    let mut g = two_player_game();
    let kroxa = g.add_card_to_graveyard(0, catalog::kroxa());
    let fodder: Vec<_> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::lightning_bolt())).collect();
    g.add_card_to_hand(1, catalog::lightning_bolt()); // nonland in hand
    let life1 = g.players[1].life;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastEscape {
        card_id: kroxa, exile_cards: fodder, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Kroxa escapable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1, "opponent discarded a nonland — no life loss");
}

#[test]
fn uro_escaped_stays_gains_life_and_draws() {
    let mut g = two_player_game();
    let uro = g.add_card_to_graveyard(0, catalog::uro());
    let fodder: Vec<_> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::lightning_bolt())).collect();
    g.add_card_to_library(0, catalog::lightning_bolt()); // something to draw
    let life0 = g.players[0].life;
    let hand0 = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastEscape {
        card_id: uro, exile_cards: fodder, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Uro escapable for GGUU + exile five");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == uro), "escaped Uro stays on the battlefield");
    assert_eq!(g.players[0].life, life0 + 3, "gained 3 life");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

#[test]
fn endless_one_enters_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::endless_one());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast Endless One for X=3");
    drain_stack(&mut g);
    let c = g.computed_permanent(id).expect("endless one");
    assert_eq!((c.power, c.toughness), (3, 3), "X=3 → 3/3");
}

#[test]
fn cruel_celebrant_drains_when_your_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cruel_celebrant());
    let token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Bolt my own fodder so the full SBA + death-trigger dispatch fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    cast_at(&mut g, bolt, Target::Permanent(token));
    assert_eq!(g.players[1].life, l1 - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, l0 + 1, "you gained 1");
}

/// CR 603.10a — "this or another creature you control dies" fires on Cruel
/// Celebrant's own death.
#[test]
fn cruel_celebrant_drains_on_its_own_death() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::cruel_celebrant());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.battlefield_find_mut(cc).unwrap().damage = 2; // lethal on the 1/2
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "opponent lost 1 to its own death");
    assert_eq!(g.players[0].life, l0 + 1, "you gained 1");
}

/// CR 603.10a — the destroy/sacrifice funnel fires the aristocrat's own
/// self-death too (parity with the SBA lethal-damage path).
#[test]
fn cruel_celebrant_drains_on_its_own_sacrifice() {
    use crabomination::game::GameEvent;
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::cruel_celebrant());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    if let Some(c) = g.dying_snapshot(cc) {
        g.died_card_snapshots.insert(cc, c);
    }
    let mut evs = vec![GameEvent::CreatureDied { card_id: cc }];
    evs.append(&mut g.remove_to_graveyard_with_triggers(cc));
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "opponent lost 1 to its own sacrifice");
    assert_eq!(g.players[0].life, l0 + 1, "you gained 1");
}

#[test]
fn mayhem_devil_pings_on_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mayhem_devil());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tribute = g.add_card_to_hand(0, catalog::tribute_to_hunger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: tribute, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tribute forces opp sacrifice");
    drain_stack(&mut g);
    // Opp sacrificed a permanent → Mayhem Devil pinged 1 (auto-targeted at them).
    assert_eq!(g.players[1].life, l1 - 1, "Mayhem Devil dealt 1 on the sacrifice");
}

#[test]
fn mana_dorks_tap_for_their_color() {
    let mut g = two_player_game();
    let bd = g.add_card_to_battlefield(0, catalog::boreal_druid());
    let ap = g.add_card_to_battlefield(0, catalog::avacyns_pilgrim());
    g.clear_sickness(bd);
    g.clear_sickness(ap);
    g.perform_action(GameAction::ActivateAbility { card_id: bd, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("Boreal Druid taps");
    g.perform_action(GameAction::ActivateAbility { card_id: ap, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("Avacyn's Pilgrim taps");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "Boreal Druid added {{C}}");
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1, "Avacyn's Pilgrim added {{W}}");
}

#[test]
fn portable_hole_exiles_cheap_permanent_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::portable_hole());
    g.players[0].mana_pool.add(Color::White, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear exiled under Portable Hole");
    // Destroy the Hole → the bear returns.
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear returns when the Hole leaves");
}

#[test]
fn giver_of_runes_grants_protection_to_another_creature() {
    let mut g = two_player_game();
    let giver = g.add_card_to_battlefield(0, catalog::giver_of_runes());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(giver);
    g.perform_action(GameAction::ActivateAbility {
        card_id: giver, ability_index: 0, target: Some(Target::Permanent(ally)), additional_targets: Vec::new(), x_value: None })
        .expect("Giver targets another creature");
    drain_stack(&mut g);
    // Can't target itself.
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: giver, ability_index: 0, target: Some(Target::Permanent(giver)), additional_targets: Vec::new(), x_value: None });
    assert!(err.is_err(), "Giver of Runes can't target itself");
}

#[test]
fn exclude_counters_creature_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Active player P0 casts a creature; respond to its own spell with Exclude.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature");
    let id = g.add_card_to_hand(0, catalog::exclude());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let h0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Exclude the creature spell");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "creature spell countered");
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature never resolved");
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 1, "cast Exclude (-1) and drew (+1)");
}

#[test]
fn miscalculation_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::miscalculation());
    g.players[0].mana_pool.add_colorless(2);
    let h0 = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Miscalculation cycles for {2}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "cycled card in graveyard");
    assert_eq!(g.players[0].hand.len(), h0, "discarded one, drew one");
}

#[test]
fn goblin_grenade_sacrifices_a_goblin_for_five() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::raging_goblin());
    let id = g.add_card_to_hand(0, catalog::goblin_grenade());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1 = g.players[1].life;
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, p1 - 5, "dealt 5 damage");
    assert!(!g.battlefield.iter().any(|c| c.id == gob), "sacrificed a Goblin");
}

#[test]
fn groundbreaker_is_sacrificed_at_end_step() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::groundbreaker());
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "sacrificed at end step");
}

#[test]
fn empty_the_warrens_makes_two_goblins_plus_storm() {
    let mut g = two_player_game();
    // Cast a prior spell so Storm copies once → 4 goblins total.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    let id = g.add_card_to_hand(0, catalog::empty_the_warrens());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let goblins = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Goblin" && c.controller == 0).count();
    assert_eq!(goblins, 4, "two from the base + two from one Storm copy");
}

#[test]
fn burning_inquiry_wheels_three_each() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); g.add_card_to_library(1, catalog::island()); }
    for _ in 0..2 { g.add_card_to_hand(0, catalog::lightning_bolt()); g.add_card_to_hand(1, catalog::lightning_bolt()); }
    let id = g.add_card_to_hand(0, catalog::burning_inquiry());
    let h0 = g.players[0].hand.len(); // includes the inquiry until cast
    let h1 = g.players[1].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, id);
    // P0: -1 (cast) +3 draw -3 discard = h0-1; P1: +3 -3 = h1.
    assert_eq!(g.players[0].hand.len(), h0 - 1, "caster net -1 (the spell itself)");
    assert_eq!(g.players[1].hand.len(), h1, "opponent net zero (drew 3, discarded 3)");
}

#[test]
fn desperate_ritual_makes_three_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::desperate_ritual());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "added RRR");
}

/// Desperate Ritual splices onto an Arcane spell: pay the splice cost, add the
/// {R}{R}{R}, and keep the ritual in hand (CR 702.47).
#[test]
fn desperate_ritual_splices_onto_arcane() {
    let mut g = two_player_game();
    let mists = g.add_card_to_hand(0, catalog::reach_through_mists()); // U Arcane, draws 1
    let ritual = g.add_card_to_hand(0, catalog::desperate_ritual());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1); // splice {1}{R}
    g.perform_action(GameAction::CastSpellSpliced {
        card_id: mists, splice_cards: vec![ritual],
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reach Through Mists splicing Desperate Ritual");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "spliced ritual added RRR");
    assert!(g.players[0].hand.iter().any(|c| c.id == ritual),
        "the spliced card stays in hand (CR 702.47a)");
}

#[test]
fn cabal_coffers_scales_with_swamps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cabal_coffers());
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::swamp()); }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("Cabal Coffers activates");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3, "B per Swamp (3 swamps)");
}

#[test]
fn staggershock_deals_two_and_rebounds() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::staggershock());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1 = g.players[1].life;
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, p1 - 2, "dealt 2 damage");
    // Rebound: exiled (not graveyard) + a delayed trigger queued.
    assert!(g.exile.iter().any(|c| c.id == id), "rebounded into exile");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id), "not in graveyard");
}

#[test]
fn bump_in_the_night_drains_three_and_has_flashback() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bump_in_the_night());
    g.players[0].mana_pool.add(Color::Black, 1);
    let p1 = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[1].life, p1 - 3, "opponent lost 3 life");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "in graveyard for flashback");
    // Flashback from the graveyard for {5}{R}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flashback castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 6, "flashback drained another 3");
    assert!(g.exile.iter().any(|c| c.id == id), "flashback exiles the card");
}

#[test]
fn chromatic_sphere_fixes_mana_and_cantrips() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::chromatic_sphere());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("Chromatic Sphere ability activates");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
    assert!(!g.battlefield.iter().any(|c| c.id == id), "sacrificed itself");
}

#[test]
fn cabal_ritual_scales_with_threshold() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cabal_ritual());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3, "no threshold → BBB");
    // With seven cards in the graveyard, threshold yields BBBBB.
    let mut g = two_player_game();
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::cabal_ritual());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 5, "threshold → BBBBB");
}

#[test]
fn deaths_shadow_scales_inversely_with_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::deaths_shadow());
    g.players[0].life = 5;
    let c = g.computed_permanent(id).expect("shadow");
    assert_eq!((c.power, c.toughness), (8, 8), "13 - 5 life = 8/8");
    // At >=13 life it's 0/0 and dies to SBA.
    g.players[0].life = 13;
    g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == id), "0/0 Death's Shadow dies at 13 life");
}

#[test]
fn silverquill_silencer_punishes_named_spell() {
    let mut g = two_player_game();
    let silencer = g.add_card_to_battlefield(0, catalog::silverquill_silencer());
    g.battlefield_find_mut(silencer).unwrap().named_card = Some("Lightning Bolt".into());
    g.add_card_to_library(0, catalog::island()); // something to draw
    let p1 = g.players[1].life;
    let hand0 = g.players[0].hand.len();
    // Opponent casts the named spell → they lose 3 life and the Silencer's
    // controller draws.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("P1 casts the named Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 3, "named-spell caster lost 3 life");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "Silencer's controller drew a card");
}

#[test]
fn silverquill_silencer_ignores_unnamed_spell() {
    let mut g = two_player_game();
    let silencer = g.add_card_to_battlefield(0, catalog::silverquill_silencer());
    g.battlefield_find_mut(silencer).unwrap().named_card = Some("Counterspell".into());
    let p1 = g.players[1].life;
    let hand0 = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("P1 casts a non-named Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1, "no life loss for an unnamed spell");
    assert_eq!(g.players[0].hand.len(), hand0, "no draw");
}

#[test]
fn beacon_effects_render_short_text_for_ui() {
    // The UI surfaces effects via `effect_short_text`; the new effects must
    // not render as empty strings.
    let txt = catalog::beacon_of_immortality().effect.effect_short_text();
    assert!(txt.contains("double"), "got: {txt}");
    assert!(txt.contains("shuffle"), "got: {txt}");
}

#[test]
fn modal_modes_render_nonempty_short_text() {
    // Callous Bloodmage's modal "choose one" modes must each surface readable
    // text in the client's mode-picker (scry/mill/discard/sacrifice were
    // previously blank).
    use crabomination::effect::Effect;
    let modes = match &catalog::callous_bloodmage().triggered_abilities[0].effect {
        Effect::ChooseMode(m) => m.clone(),
        other => match other {
            // ETB wraps the ChooseMode; unwrap one layer if needed.
            Effect::Seq(v) => v.iter().find_map(|e| match e {
                Effect::ChooseMode(m) => Some(m.clone()), _ => None }).expect("modes"),
            _ => panic!("expected ChooseMode"),
        },
    };
    for m in &modes {
        assert!(!m.effect_short_text().is_empty(), "mode rendered empty: {m:?}");
    }
}

/// Mercurial Transformation and Academic Probation are modal — their picker
/// labels must render readable text (ResetCreature / BecomeColor /
/// NameOpponentCastLock previously rendered blank in the client modal).
#[test]
fn new_modal_cards_render_nonempty_mode_text() {
    use crabomination::effect::Effect;
    for card in [catalog::mercurial_transformation(), catalog::academic_probation()] {
        let Effect::ChooseMode(modes) = &card.effect else {
            panic!("{} should be a ChooseMode", card.name);
        };
        for m in modes {
            assert!(!m.effect_short_text().is_empty(),
                "{} mode rendered empty: {m:?}", card.name);
        }
    }
}

#[test]
fn beacon_of_immortality_doubles_life_and_reshuffles() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::beacon_of_immortality());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.players[1].life = 17;
    let lib0_before = g.players[0].library.len();
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, 34, "CR 701.10d: target player's life doubled");
    // Beacon shuffled into owner's library — not the graveyard.
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id), "not in graveyard");
    assert_eq!(g.players[0].library.len(), lib0_before + 1, "back in library");
}

#[test]
fn beacon_of_immortality_auto_targets_caster() {
    // The bot/auto-targeter should double its own life, not the opponent's.
    let g = two_player_game();
    let t = g.auto_target_for_effect(&catalog::beacon_of_immortality().effect, 0);
    assert_eq!(t, Some(Target::Player(0)), "auto-target is the caster");
}

#[test]
fn beacon_of_destruction_burns_and_reshuffles() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::beacon_of_destruction());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let p1 = g.players[1].life;
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, p1 - 5, "dealt 5 damage");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id), "shuffled in, not in graveyard");
    assert!(g.players[0].library.iter().any(|c| c.id == id), "Beacon back in library");
}

#[test]
fn noble_hierarch_taps_for_a_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::noble_hierarch());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("Noble Hierarch's mana ability activates");
    // AutoDecider falls back to the first listed color (Green).
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn noble_hierarch_exalts_a_lone_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::noble_hierarch());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3), "Exalted pumped the lone attacker +1/+1");
}

#[test]
fn legion_warboss_makes_a_goblin_at_combat() {
    let mut g = two_player_game();
    let boss = g.add_card_to_battlefield(0, catalog::legion_warboss());
    g.clear_sickness(boss);
    let goblins_before = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Goblin").count();
    g.fire_step_triggers(crabomination::game::TurnStep::BeginCombat);
    drain_stack(&mut g);
    let goblins_after = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Goblin").count();
    assert_eq!(goblins_after, goblins_before + 1, "begin-combat made a Goblin");
    let gob = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Goblin").unwrap();
    assert!(gob.definition.keywords.contains(&Keyword::Haste), "the Goblin has haste");
}

#[test]
fn flusterstorm_counters_an_instant_unless_paid() {
    let mut g = two_player_game();
    // P1 casts a bolt; P0 Flusterstorms it. With no mana to pay {1}, it's countered.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("P1 casts Bolt");
    let fs = g.add_card_to_hand(0, catalog::flusterstorm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fs, target: Some(crabomination::game::types::Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("P0 casts Flusterstorm at the Bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt was countered (unpaid)");
}

// ── No-maximum-hand-size + play-lands-from-graveyard statics ────────────────

#[test]
fn reliquary_tower_skips_cleanup_discard() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::reliquary_tower());
    for _ in 0..10 {
        g.add_card_to_hand(0, catalog::forest());
    }
    assert_eq!(g.effective_max_hand_size(0), None, "Reliquary Tower removes the maximum");
    let outcome = g.do_cleanup(&mut Vec::new());
    assert!(
        !matches!(outcome, crabomination::game::stack::CleanupOutcome::Suspended),
        "cleanup completes synchronously"
    );
    assert_eq!(g.players[0].hand.len(), 10, "no cards discarded with no max hand size");
}

#[test]
fn no_reliquary_tower_discards_to_seven() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    for _ in 0..10 {
        g.add_card_to_hand(0, catalog::forest());
    }
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.players[0].hand.len(), 7, "without the static, discard down to seven");
}

#[test]
fn crucible_of_worlds_plays_a_land_from_the_graveyard() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::crucible_of_worlds());
    let land = g.add_card_to_graveyard(0, catalog::forest());
    assert!(g.player_may_play_lands_from_graveyard(0));
    g.perform_action(GameAction::PlayLandFromGraveyard(land)).expect("land replays from gy");
    assert!(g.battlefield.iter().any(|c| c.id == land), "land is on the battlefield");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == land), "removed from graveyard");
}

#[test]
fn play_land_from_graveyard_rejected_without_static() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_graveyard(0, catalog::forest());
    assert!(g.perform_action(GameAction::PlayLandFromGraveyard(land)).is_err(),
        "no Crucible → can't play lands from graveyard");
}

// ── Coiling Oracle (RevealTopLandToBattlefieldElseHand) ────────────────────

#[test]
fn coiling_oracle_puts_revealed_land_onto_battlefield() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let land = g.add_card_to_library(0, catalog::forest());
    let oracle = g.add_card_to_battlefield(0, catalog::coiling_oracle());
    g.fire_self_etb_triggers(oracle, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == land), "revealed land enters the battlefield");
    assert!(!g.players[0].library.iter().any(|c| c.id == land), "land left the library");
}

#[test]
fn coiling_oracle_puts_revealed_nonland_into_hand() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let spell = g.add_card_to_library(0, catalog::lightning_bolt());
    let oracle = g.add_card_to_battlefield(0, catalog::coiling_oracle());
    g.fire_self_etb_triggers(oracle, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spell), "revealed nonland goes to hand");
    assert!(!g.battlefield.iter().any(|c| c.id == spell), "nonland does not enter the battlefield");
}

// ── Shield Sphere (-0/-1 block counter) ────────────────────────────────────

#[test]
fn shield_sphere_gains_minus_zero_minus_one_counter_when_it_blocks() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let shield = g.add_card_to_battlefield(0, catalog::shield_sphere());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(shield, attacker)])).unwrap();
    drain_stack(&mut g);
    let s = g.battlefield_find(shield).expect("shield still alive");
    assert_eq!(s.counter_count(CounterType::MinusZeroMinusOne), 1, "gained a -0/-1 counter");
    assert_eq!(s.power(), 0, "power unchanged at 0");
    assert_eq!(s.toughness(), 5, "toughness dropped from 6 to 5");
}

// ── CR 122.2 — counters cease to exist on zone change ──────────────────────

#[test]
fn counters_are_removed_when_a_creature_changes_zones() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    // Blink it with Ephemerate (exile then return = a new object, CR 400.7).
    let eph = g.add_card_to_hand(0, catalog::ephemerate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eph, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ephemerate blinks the bear");
    drain_stack(&mut g);
    let returned = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears")
        .expect("bear returned to the battlefield");
    assert_eq!(returned.counter_count(CounterType::PlusOnePlusOne), 0,
        "the new object enters with no +1/+1 counters (CR 122.2)");
}

// ── CR 705.2 — Mana Clash (two-player flip-off loop) ───────────────────────

#[test]
fn mana_clash_damages_the_player_who_flips_tails_until_both_heads() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // Round 1: caster (P0) tails → takes 1; opp (P1) heads. Round 2: both heads → stop.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(false), // P0 round 1 → tails
        DecisionAnswer::Bool(true),  // P1 round 1 → heads
        DecisionAnswer::Bool(true),  // P0 round 2 → heads
        DecisionAnswer::Bool(true),  // P1 round 2 → heads (loop ends)
    ]));
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::mana_clash());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mana Clash at P1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life - 1, "caster took 1 from their tails flip");
    assert_eq!(g.players[1].life, p1_life, "opponent flipped heads, took no damage");
}

// ── CR 701.10f — Mana Reflection doubles mana production ────────────────────

#[test]
fn mana_reflection_doubles_a_tapped_for_mana_ability() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mana_reflection());
    let dork = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(dork);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Llanowar Elves taps for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2,
        "Mana Reflection doubles the green to GG");
}

#[test]
fn mana_reflection_does_not_double_a_ritual_spell() {
    // Mana Reflection only doubles tapping a permanent for mana (CR 701.10f),
    // not a ritual spell's mana.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mana_reflection());
    let ritual = g.add_card_to_hand(0, catalog::dark_ritual());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ritual, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dark Ritual");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3,
        "Dark Ritual still nets BBB (not doubled)");
}

// ── Font of Mythos / Venser's Journal / Sensei's Divining Top ───────────────

#[test]
fn font_of_mythos_draws_two_extra_on_your_draw_step() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    g.add_card_to_battlefield(0, catalog::font_of_mythos());
    let before = g.players[0].hand.len();
    g.fire_step_triggers(crabomination::game::types::TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 2, "drew two additional cards");
}

#[test]
fn vensers_journal_gains_life_per_card_in_hand() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..4 { g.add_card_to_hand(0, catalog::forest()); }
    g.add_card_to_battlefield(0, catalog::vensers_journal());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len() as i32;
    g.fire_step_triggers(crabomination::game::types::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + hand, "gained 1 life per card in hand");
}

#[test]
fn senseis_divining_top_draws_and_returns_itself_to_library() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::forest());
    let top = g.add_card_to_battlefield(0, catalog::senseis_divining_top());
    g.clear_sickness(top);
    g.perform_action(GameAction::ActivateAbility {
        card_id: top, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Top's draw ability activates");
    drain_stack(&mut g);
    assert!(g.players[0].library.iter().any(|c| c.id == top),
        "the Top is on top of the library");
    assert!(!g.battlefield.iter().any(|c| c.id == top), "no longer on the battlefield");
}

// ── Spellbook / spell-tax stax artifacts / Pestilence ──────────────────────

#[test]
fn spellbook_removes_maximum_hand_size() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spellbook());
    assert_eq!(g.effective_max_hand_size(0), None);
}

#[test]
fn thorn_of_amethyst_taxes_only_noncreature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thorn_of_amethyst());
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear_id = g.add_card_to_hand(0, catalog::grizzly_bears());
    let bolt = g.players[0].hand.iter().find(|c| c.id == bolt_id).unwrap().clone();
    let bear = g.players[0].hand.iter().find(|c| c.id == bear_id).unwrap().clone();
    assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 0, &bolt, None), 1, "noncreature taxed");
    assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 0, &bear, None), 0, "creature untaxed");
}

#[test]
fn lodestone_golem_taxes_nonartifact_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lodestone_golem());
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    let sol_id = g.add_card_to_hand(0, catalog::sol_ring());
    let bolt = g.players[0].hand.iter().find(|c| c.id == bolt_id).unwrap().clone();
    let sol = g.players[0].hand.iter().find(|c| c.id == sol_id).unwrap().clone();
    assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 0, &bolt, None), 1, "nonartifact taxed");
    assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 0, &sol, None), 0, "artifact untaxed");
}

#[test]
fn pestilence_pings_each_creature_and_player() {
    let mut g = two_player_game();
    let pest = g.add_card_to_battlefield(0, catalog::pestilence());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pest, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Pestilence ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 - 1, "P0 took 1");
    assert_eq!(g.players[1].life, p1 - 1, "P1 took 1");
    assert_eq!(g.battlefield_find(bear).map(|c| c.damage), Some(1), "bear took 1");
}

// ── Cursed Totem / Damping Matrix (creature activated-ability lock) ─────────

#[test]
fn cursed_totem_locks_creature_nonmana_abilities_but_not_mana() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cursed_totem());
    let pinger = g.add_card_to_battlefield(0, catalog::prodigal_sorcerer());
    g.clear_sickness(pinger);
    let dork = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(dork);
    // Non-mana creature ability is locked.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: pinger, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).is_err(), "Cursed Totem locks the pinger's tap ability");
    // Mana ability still works.
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("mana ability is exempt");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

// ── Explosive Vegetation / Skyshroud Claim (two-land ramp) ─────────────────

#[test]
fn explosive_vegetation_fetches_two_basics_tapped() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::forest());
    let b = g.add_card_to_library(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
    ]));
    let id = g.add_card_to_hand(0, catalog::explosive_vegetation());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Explosive Vegetation castable");
    drain_stack(&mut g);
    for land in [a, b] {
        let c = g.battlefield.iter().find(|c| c.id == land).expect("basic on battlefield");
        assert!(c.tapped, "basic enters tapped");
    }
}

#[test]
fn skyshroud_claim_fetches_two_forests_untapped() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::forest());
    let b = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
    ]));
    let id = g.add_card_to_hand(0, catalog::skyshroud_claim());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skyshroud Claim castable");
    drain_stack(&mut g);
    for land in [a, b] {
        let c = g.battlefield.iter().find(|c| c.id == land).expect("forest on battlefield");
        assert!(!c.tapped, "Skyshroud forests enter untapped");
    }
}

// ── Jace's Ingenuity / Corrupt (Swamps-matters drain) ──────────────────────

#[test]
fn jaces_ingenuity_draws_three() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::jaces_ingenuity());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // -1 for the spell leaving hand, +3 drawn.
    assert_eq!(g.players[0].hand.len(), before - 1 + 3);
}

#[test]
fn corrupt_damages_and_gains_life_per_swamp() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::swamp()); }
    let id = g.add_card_to_hand(0, catalog::corrupt());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Black, 1);
    let p1 = g.players[1].life;
    let p0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Corrupt at P1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 3, "3 damage = 3 swamps");
    assert_eq!(g.players[0].life, p0 + 3, "gained 3 life");
}

// ── Congregate / Smite the Monstrous ───────────────────────────────────────

#[test]
fn congregate_gains_two_life_per_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::congregate());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Congregate at self");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 6, "2 life × 3 creatures");
}

#[test]
fn smite_the_monstrous_only_hits_power_four_or_more() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::smite_the_monstrous());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "can't target a 2/2");
}

