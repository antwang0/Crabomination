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

// ── Lands ────────────────────────────────────────────────────────────────────

#[test]
fn godless_shrine_pays_two_life_and_taps_for_white_or_black() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::godless_shrine());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(!card.tapped, "shockland enters untapped (AutoDecider pays 2 life)");
    assert_eq!(g.players[0].life, 18, "paid 2 life");
    // Taps for white (ability 0) or black (ability 1).
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("white mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

#[test]
fn blooming_marsh_enters_untapped_with_few_lands() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blooming_marsh());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield_find(id).unwrap().tapped,
        "fastland enters untapped with no other lands");
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("green mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn meticulous_archive_enters_tapped_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::meticulous_archive());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.tapped, "surveil land enters tapped");
    // Taps for white or blue once untapped.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("blue mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
}

#[test]
fn darkbore_pathway_plays_either_face() {
    // Front face: Swamp (taps for black).
    let mut g = two_player_game();
    let front = g.add_card_to_hand(0, catalog::darkbore_pathway());
    g.perform_action(GameAction::PlayLand(front)).unwrap();
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: front, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("black mana ability on front face");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1, "front face taps for black");

    // Back face: Forest (taps for green).
    let mut g2 = two_player_game();
    let back = g2.add_card_to_hand(0, catalog::darkbore_pathway());
    g2.perform_action(GameAction::PlayLandBack(back)).unwrap();
    drain_stack(&mut g2);
    g2.perform_action(GameAction::ActivateAbility {
        card_id: back, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("green mana ability on back face");
    drain_stack(&mut g2);
    assert_eq!(g2.players[0].mana_pool.amount(Color::Green), 1, "back face taps for green");
}

#[test]
fn amped_raptor_etb_gets_two_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::amped_raptor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Amped Raptor castable for {1}{R}");
    drain_stack(&mut g);
    // Only 2 energy after ETB — can't afford the {E}{E}{E}{E} free-cast, so
    // the top card stays exiled and energy is untouched.
    assert_eq!(g.players[0].energy, 2, "ETB grants {{E}}{{E}}");
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.definition.power, c.definition.toughness), (2, 1));
}

#[test]
fn amped_raptor_pays_energy_to_free_cast_exiled_card() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    // Pre-float 2 energy → 4 after the ETB {E}{E}, enough to pay {E}×4.
    g.players[0].energy = 2;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_hand(0, catalog::amped_raptor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Amped Raptor castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "paid all 4 energy to free-cast");
    assert!(g.battlefield.iter().any(|c| c.id == bears),
        "the exiled top card was cast for free onto the battlefield");
}

#[test]
fn bonecrusher_giant_is_a_four_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::bonecrusher_giant());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.definition.power, c.definition.toughness), (4, 3));
}

#[test]
fn magda_brazen_outlaw_pumps_other_dwarves() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::magda_brazen_outlaw());
    // Practiced Scrollsmith is a Dwarf Cleric (3/2). Magda's anthem
    // gives it +1/+0 → 4/2.
    let dwarf = g.add_card_to_battlefield(0, catalog::practiced_scrollsmith());
    let c = g.computed_permanent(dwarf).unwrap();
    assert_eq!(c.power, 4, "other Dwarf gets +1/+0 from Magda's anthem");
    assert_eq!(c.toughness, 2, "toughness unchanged by +1/+0");
}

#[test]
fn three_tree_city_enters_with_charge_and_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::three_tree_city());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Charge), 3,
        "Three Tree City enters with three charge counters");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("{T}, remove a charge: add one mana of any color");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "produced blue mana");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Charge), 2,
        "one charge counter spent");
}

#[test]
fn three_tree_city_sacrifices_itself_when_last_charge_removed() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::three_tree_city());
    // Seed with a single charge so the next activation empties it.
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Charge, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("removing the last charge taps for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "produced green mana");
    assert!(g.battlefield_find(id).is_none(), "sacrificed once charges hit zero");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "in the graveyard");
}

#[test]
fn wight_of_the_reliquary_sacrifices_a_land_to_fetch_a_land() {
    let mut g = two_player_game();
    let wight = g.add_card_to_battlefield(0, catalog::wight_of_the_reliquary());
    g.clear_sickness(wight);
    let sac_land = g.add_card_to_battlefield(0, catalog::mountain());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: wight, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("{T}, sacrifice a land: search a land");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == wight), "Wight survives — a land was the cost");
    assert!(!g.battlefield.iter().any(|c| c.id == sac_land), "the Mountain was sacrificed");
    let land = g.battlefield_find(forest).expect("fetched land on battlefield");
    assert!(land.tapped, "fetched land enters tapped");
}

#[test]
fn wight_of_the_reliquary_grows_with_lands_in_your_graveyard() {
    let mut g = two_player_game();
    let wight = g.add_card_to_battlefield(0, catalog::wight_of_the_reliquary());
    // Base 1/1 with no lands in graveyard.
    let c = g.compute_battlefield();
    let w = c.iter().find(|c| c.id == wight).unwrap();
    assert_eq!((w.power, w.toughness), (1, 1), "base 1/1");
    // Two lands in your graveyard → 3/3. Opponent's gy lands don't count.
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::mountain());
    g.add_card_to_graveyard(1, catalog::island());
    let c = g.compute_battlefield();
    let w = c.iter().find(|c| c.id == wight).unwrap();
    assert_eq!((w.power, w.toughness), (3, 3),
        "1 base + 2 lands in your graveyard (opp's land ignored)");
}

/// Shared helper for the deck dual-land cycle: play the land, optionally
/// untap it, then assert mana abilities 0 and 1 tap for the two colors.
fn assert_deck_dual_land(
    def_fn: fn() -> crabomination::card::CardDefinition,
    c0: Color,
    c1: Color,
    expect_tapped_on_etb: bool,
) {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, def_fn());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().tapped, expect_tapped_on_etb,
        "ETB tapped-ness");
    for (idx, color) in [(0usize, c0), (1usize, c1)] {
        g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
        g.players[0].mana_pool = crabomination::mana::ManaPool::default();
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: idx, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("mana ability");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.amount(color), 1, "ability {idx} taps for {color:?}");
    }
}

#[test]
fn hallowed_fountain_shockland_white_blue() {
    // Shockland: AutoDecider pays 2 life → enters untapped.
    assert_deck_dual_land(catalog::hallowed_fountain, Color::White, Color::Blue, false);
}

#[test]
fn overgrown_tomb_shockland_black_green() {
    assert_deck_dual_land(catalog::overgrown_tomb, Color::Black, Color::Green, false);
}

#[test]
fn copperline_gorge_fastland_red_green() {
    // Fastland: untapped with no other lands.
    assert_deck_dual_land(catalog::copperline_gorge, Color::Red, Color::Green, false);
}

#[test]
fn shadowy_backstreet_surveil_land_white_black() {
    // Surveil land: enters tapped.
    assert_deck_dual_land(catalog::shadowy_backstreet, Color::White, Color::Black, true);
}

#[test]
fn undercity_sewers_surveil_land_blue_black() {
    assert_deck_dual_land(catalog::undercity_sewers, Color::Blue, Color::Black, true);
}

/// Shared helper for the Onslaught/Zendikar fetchland cycle (zen::lands):
/// {T}, pay 1 life, sacrifice: search for a land of one of two types and
/// put it onto the battlefield untapped. Seeds `basic` in the library and
/// asserts it is fetched, the fetchland is sacrificed, and 1 life is paid.
fn assert_fetchland_fetches(
    fetch_fn: fn() -> crabomination::card::CardDefinition,
    basic_fn: fn() -> crabomination::card::CardDefinition,
) {
    let mut g = two_player_game();
    let basic = g.add_card_to_library(0, basic_fn());
    let fetch = g.add_card_to_battlefield(0, fetch_fn());
    g.clear_sickness(fetch);
    let life_before = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: fetch, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("fetchland {T}, pay 1 life, sac: search a land");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fetch), "fetchland sacrificed");
    let fetched = g.battlefield_find(basic).expect("fetched basic on battlefield");
    assert!(!fetched.tapped, "fetchland puts the land in untapped");
    assert_eq!(g.players[0].life, life_before - 1, "paid 1 life");
}

#[test]
fn polluted_delta_fetches_island() {
    assert_fetchland_fetches(catalog::polluted_delta, catalog::island);
}

#[test]
fn bloodstained_mire_fetches_mountain() {
    assert_fetchland_fetches(catalog::bloodstained_mire, catalog::mountain);
}

#[test]
fn wooded_foothills_fetches_forest() {
    assert_fetchland_fetches(catalog::wooded_foothills, catalog::forest);
}

#[test]
fn windswept_heath_fetches_forest() {
    assert_fetchland_fetches(catalog::windswept_heath, catalog::forest);
}

#[test]
fn misty_rainforest_fetches_island() {
    assert_fetchland_fetches(catalog::misty_rainforest, catalog::island);
}

#[test]
fn scalding_tarn_fetches_mountain() {
    assert_fetchland_fetches(catalog::scalding_tarn, catalog::mountain);
}

#[test]
fn verdant_catacombs_fetches_forest() {
    assert_fetchland_fetches(catalog::verdant_catacombs, catalog::forest);
}

#[test]
fn arid_mesa_fetches_mountain() {
    assert_fetchland_fetches(catalog::arid_mesa, catalog::mountain);
}

#[test]
fn marsh_flats_fetches_plains() {
    assert_fetchland_fetches(catalog::marsh_flats, catalog::plains);
}

// ── Equipment (CR 702.6) — attach-based equip via GameAction::Equip ─────────

/// Bonesplitter equips a creature for {1} and grants +2/+0 via the layer
/// system.
#[test]
fn bonesplitter_equips_and_grants_plus_two_zero() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: boner, target: bear })
        .expect("equip {1} should succeed");
    let cp = g.computed_permanent(bear).expect("bear alive");
    assert_eq!(cp.power, 4, "2/2 + 2/0 = 4 power");
    assert_eq!(cp.toughness, 2, "toughness unchanged");
    // The equipment link is recorded.
    let eq = g.battlefield.iter().find(|c| c.id == boner).unwrap();
    assert_eq!(eq.attached_to, Some(bear));
}

/// Shuko equips for free ({0}) and grants +1/+0.
#[test]
fn shuko_equips_for_free_and_grants_plus_one_zero() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let shuko = g.add_card_to_battlefield(0, catalog::shuko());
    // No mana floated — equip {0} should still succeed.
    g.perform_action(GameAction::Equip { equipment: shuko, target: bear })
        .expect("equip {0} should succeed with no mana");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "2/2 + 1/0 = 3 power");
    assert_eq!(cp.toughness, 2);
}

/// Lavaspur Boots grants +1/+1 and haste while attached.
#[test]
fn lavaspur_boots_grants_haste_and_plus_one_one() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boots = g.add_card_to_battlefield(0, catalog::lavaspur_boots());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: boots, target: bear })
        .expect("equip {1} should succeed");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3);
    assert_eq!(cp.toughness, 3);
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Haste), "boots grant haste");
}

/// Skullclamp's equip-granted "dies → draw two" trigger fires when the
/// equipped creature dies (CR 702.6e).
#[test]
fn skullclamp_draws_two_when_equipped_creature_dies() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let clamp = g.add_card_to_battlefield(0, catalog::skullclamp());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: clamp, target: bear })
        .expect("equip {1}");
    // 2/2 + 1/-1 = 3/1. One damage is lethal.
    let hand_before = g.players[0].hand.len();
    g.battlefield_find_mut(bear).unwrap().damage = 1;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear died");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "Skullclamp drew two");
}

/// The equip-granted dies trigger also fires through the Destroy/sacrifice
/// funnel, not just the SBA lethal-damage path (CR 702.6e).
#[test]
fn skullclamp_draws_two_when_equipped_creature_destroyed() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let clamp = g.add_card_to_battlefield(0, catalog::skullclamp());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: clamp, target: bear })
        .expect("equip {1}");
    let hand_before = g.players[0].hand.len();
    g.remove_to_graveyard_with_triggers(bear);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "Skullclamp drew two");
}

/// Equip rejects a creature you don't control (CR 702.6c).
#[test]
fn equip_rejects_creature_you_dont_control() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    let err = g
        .perform_action(GameAction::Equip { equipment: boner, target: opp_bear })
        .expect_err("cannot equip an opponent's creature");
    assert!(matches!(err, GameError::InvalidTarget), "got {err:?}");
}

/// Equipping a non-Equipment artifact is rejected.
#[test]
fn equip_rejects_non_equipment() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Coveted Jewel is an artifact but not Equipment.
    let jewel = g.add_card_to_battlefield(0, catalog::coveted_jewel());
    let err = g
        .perform_action(GameAction::Equip { equipment: jewel, target: bear })
        .expect_err("Coveted Jewel is not Equipment");
    assert!(matches!(err, GameError::NotEquipment(_)), "got {err:?}");
}

/// When the equipped creature dies, the equipment's link is cleared by the
/// SBA scan and the bonus stops applying (the equipment stays on the bf).
#[test]
fn equip_bonus_falls_off_when_creature_dies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: boner, target: bear })
        .expect("equip ok");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    // Kill the bear (move to graveyard) and run SBAs.
    g.remove_from_battlefield_to_graveyard_raw(bear);
    g.check_state_based_actions();
    let eq = g.battlefield.iter().find(|c| c.id == boner).unwrap();
    assert_eq!(eq.attached_to, None, "stale link cleared by SBA");
}

/// Kor Outfitter's ETB attaches a target Equipment you control to a target
/// creature you control — the two-slot trigger auto-targeter fills slot 1
/// (the creature) after slot 0 (the Equipment).
#[test]
fn kor_outfitter_etb_attaches_equipment_to_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // ETB self-source trigger fires only through the real movement funnel.
    g.move_card_to_battlefield_for_test(0, catalog::kor_outfitter());
    drain_stack(&mut g);
    let eq = g.battlefield.iter().find(|c| c.id == boner).unwrap();
    assert_eq!(eq.attached_to, Some(bear), "Bonesplitter attached to the bear, not the Outfitter");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 from the attached Bonesplitter");
}

/// Brass Squire's {T} ability attaches a chosen Equipment you control to a
/// chosen creature you control (the activated two-slot path).
#[test]
fn brass_squire_taps_to_attach_equipment() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    let squire = g.add_card_to_battlefield(0, catalog::brass_squire());
    g.clear_sickness(squire);
    g.perform_action(GameAction::ActivateAbility {
        card_id: squire,
        ability_index: 0,
        target: Some(Target::Permanent(boner)),
        additional_targets: vec![Target::Permanent(bear)],
        x_value: None, mode: None,
    })
    .expect("Brass Squire activates");
    drain_stack(&mut g);
    let eq = g.battlefield.iter().find(|c| c.id == boner).unwrap();
    assert_eq!(eq.attached_to, Some(bear));
    assert!(g.battlefield.iter().find(|c| c.id == squire).unwrap().tapped, "Squire tapped for the cost");
}

/// Equip is sorcery-speed only — rejected when it isn't the controller's
/// main phase.
#[test]
fn equip_rejects_at_instant_speed() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::DeclareAttackers;
    let err = g
        .perform_action(GameAction::Equip { equipment: boner, target: bear })
        .expect_err("equip is sorcery speed only");
    assert!(matches!(err, GameError::SorcerySpeedOnly), "got {err:?}");
}

// ── Vehicles & Crew (CR 702.122) ────────────────────────────────────────────

/// Crewing Esika's Chariot (Crew 4) by tapping two 2/2 creatures turns it
/// into a 4/4 artifact creature until end of turn.
#[test]
fn crew_animates_vehicle_until_end_of_turn() {
    let mut g = two_player_game();
    let chariot = g.add_card_to_battlefield(0, catalog::esikas_chariot());
    let pre = g.computed_permanent(chariot).unwrap();
    assert!(!pre.card_types.contains(&CardType::Creature), "uncrewed = not a creature");

    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::Crew { vehicle: chariot, crew_creatures: vec![b1, b2] })
        .expect("crew 4 satisfied by two 2/2s");
    let post = g.computed_permanent(chariot).unwrap();
    assert!(post.card_types.contains(&CardType::Creature), "crewed = creature");
    assert_eq!(post.power, 4);
    assert_eq!(post.toughness, 4);
    assert!(g.battlefield_find(b1).unwrap().tapped);
    assert!(g.battlefield_find(b2).unwrap().tapped);

    g.expire_end_of_turn_effects();
    let after = g.computed_permanent(chariot).unwrap();
    assert!(!after.card_types.contains(&CardType::Creature), "animation wears off EOT");
}

/// Crew is rejected when the tapped creatures' total power is below the crew
/// number.
#[test]
fn crew_rejects_insufficient_power() {
    let mut g = two_player_game();
    let chariot = g.add_card_to_battlefield(0, catalog::esikas_chariot()); // Crew 4
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
    let err = g
        .perform_action(GameAction::Crew { vehicle: chariot, crew_creatures: vec![b1] })
        .expect_err("2 power < crew 4");
    assert!(matches!(err, GameError::SelectionRequirementViolated), "got {err:?}");
    assert!(!g.battlefield_find(b1).unwrap().tapped);
}

/// Crew rejects an already-tapped creature.
#[test]
fn crew_rejects_tapped_crew_creature() {
    let mut g = two_player_game();
    let copter = g.add_card_to_battlefield(0, catalog::smugglers_copter()); // Crew 1
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let err = g
        .perform_action(GameAction::Crew { vehicle: copter, crew_creatures: vec![bear] })
        .expect_err("can't crew with a tapped creature");
    assert!(matches!(err, GameError::CardIsTapped(_)), "got {err:?}");
}

/// Smuggler's Copter (Crew 1) becomes a 3/3 flier and loots when it attacks.
#[test]
fn smugglers_copter_crews_and_loots_on_attack() {
    use crabomination::decision::ScriptedDecider;
    let mut g = two_player_game();
    let copter = g.add_card_to_battlefield(0, catalog::smugglers_copter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(copter);
    let lib_id = g.next_id();
    g.players[0].library.push(crabomination::card::CardInstance::new(
        lib_id, catalog::grizzly_bears(), 0));
    let _hand = g.add_card_to_hand(0, catalog::grizzly_bears());

    g.perform_action(GameAction::Crew { vehicle: copter, crew_creatures: vec![bear] })
        .expect("crew 1 satisfied by a 2/2");
    let cp = g.computed_permanent(copter).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Flying));

    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: copter,
        target: AttackTarget::Player(1),
    }]))
        .expect("crewed copter attacks");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "copter looted: a card was discarded to the graveyard");
}

/// An uncrewed Vehicle can't be declared as an attacker (it isn't a creature).
#[test]
fn uncrewed_vehicle_cannot_attack() {
    let mut g = two_player_game();
    let copter = g.add_card_to_battlefield(0, catalog::smugglers_copter());
    g.clear_sickness(copter);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let err = g
        .perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: copter,
            target: AttackTarget::Player(1),
        }]))
        .expect_err("uncrewed vehicle is not a creature and can't attack");
    let _ = err;
}

// ── Manlands (creature-lands via Effect::BecomeCreature) ────────────────────

/// Celestial Colonnade animates into a 4/4 flying-vigilance Elemental that's
/// still a land, then reverts at end of turn.
#[test]
fn celestial_colonnade_animates_into_a_4_4_flier() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::celestial_colonnade());
    let pre = g.computed_permanent(land).unwrap();
    assert!(pre.card_types.contains(&CardType::Land));
    assert!(!pre.card_types.contains(&CardType::Creature));

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate for {3}{W}{U}");
    drain_stack(&mut g);

    let post = g.computed_permanent(land).unwrap();
    assert!(post.card_types.contains(&CardType::Creature), "now a creature");
    assert!(post.card_types.contains(&CardType::Land), "still a land");
    assert_eq!(post.power, 4);
    assert_eq!(post.toughness, 4);
    assert!(post.keywords.contains(&Keyword::Flying));
    assert!(post.keywords.contains(&Keyword::Vigilance));
    assert!(post.subtypes.creature_types.contains(&CreatureType::Elemental));

    g.expire_end_of_turn_effects();
    let after = g.computed_permanent(land).unwrap();
    assert!(!after.card_types.contains(&CardType::Creature), "reverts to land EOT");
}

/// Creeping Tar Pit animates into a 3/2 unblockable Elemental.
#[test]
fn creeping_tar_pit_animates_unblockable() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::creeping_tar_pit());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate for {1}{U}{B}");
    drain_stack(&mut g);
    let post = g.computed_permanent(land).unwrap();
    assert_eq!(post.power, 3);
    assert_eq!(post.toughness, 2);
    assert!(post.keywords.contains(&Keyword::Unblockable));
}

/// An animated manland can be declared as an attacker (it's a creature).
#[test]
fn animated_manland_can_attack() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::creeping_tar_pit());
    g.clear_sickness(land);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: land,
        target: AttackTarget::Player(1),
    }]))
    .expect("animated manland attacks");
}

/// Mutavault taps for {C} and animates (for {1}) into a 2/2 Changeling that's
/// still a land.
#[test]
fn mutavault_taps_for_c_and_animates_into_changeling() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::mutavault());
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "produced colorless mana");
    // Untap so the animate ability (no tap cost) just needs {1}.
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate for {1}");
    drain_stack(&mut g);
    let post = g.computed_permanent(land).unwrap();
    assert_eq!((post.power, post.toughness), (2, 2));
    assert!(post.card_types.contains(&CardType::Land), "still a land");
    assert!(post.keywords.contains(&Keyword::Changeling));
}

/// Inkmoth Nexus animates into a 1/1 flier with infect.
#[test]
fn inkmoth_nexus_animates_into_flying_infect() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::inkmoth_nexus());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate for {1}");
    drain_stack(&mut g);
    let post = g.computed_permanent(land).unwrap();
    assert_eq!((post.power, post.toughness), (1, 1));
    assert!(post.keywords.contains(&Keyword::Flying));
    assert!(post.keywords.contains(&Keyword::Infect));
}

/// Mishra's Factory animates into a 2/2 Assembly-Worker.
#[test]
fn mishras_factory_animates_into_assembly_worker() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::mishras_factory());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate for {1}");
    drain_stack(&mut g);
    let post = g.computed_permanent(land).unwrap();
    assert_eq!((post.power, post.toughness), (2, 2));
    assert!(post.subtypes.creature_types.contains(&CreatureType::AssemblyWorker));
}

// ── Coverage backfill: burn / discard / sacrifice spells ────────────────────

/// Char deals 4 to the targeted player and 2 to its caster.
#[test]
fn char_burns_target_and_pings_caster() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::char());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Char castable for {2}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "target takes 4");
    assert_eq!(g.players[0].life, p0_life - 2, "caster takes 2");
}

/// Thud sacrifices a creature and deals damage equal to its power.
#[test]
fn thud_sacrifices_and_deals_power_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::thud());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thud castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "deals 2 (sacrificed bear's power)");
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear sacrificed");
}

/// Thoughtseize makes an opponent discard a nonland card and costs 2 life.
#[test]
fn thoughtseize_discards_nonland_and_costs_two_life() {
    let mut g = two_player_game();
    let victim_card = g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::thoughtseize());
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thoughtseize castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == victim_card),
        "opp's nonland card discarded");
    assert_eq!(g.players[0].life, p0_life - 2, "caster loses 2 life");
}

/// Searing Blaze kills a small creature and burns its controller.
#[test]
fn searing_blaze_burns_creature_and_player() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::searing_blaze());
    g.players[0].mana_pool.add(Color::Red, 2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Searing Blaze castable for {R}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 dies to 3 damage");
    assert_eq!(g.players[1].life, p1_life - 3, "opp takes 3");
}

/// Inquisition of Kozilek makes an opponent discard a chosen nonland card
/// with mana value 3 or less.
#[test]
fn inquisition_of_kozilek_discards_low_cmc_nonland() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears()); // MV 2, nonland
    let id = g.add_card_to_hand(0, catalog::inquisition_of_kozilek());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquisition castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "MV-2 nonland discarded");
}

/// Collective Defiance mode 0 deals 4 damage to a creature.
#[test]
fn collective_defiance_mode0_burns_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::collective_defiance());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Collective Defiance mode 0 castable for {1}{R}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "4 dmg kills the 2/2");
}

/// Collective Defiance mode 2 deals 3 damage to each opponent.
#[test]
fn collective_defiance_mode2_burns_opponent() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::collective_defiance());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("Collective Defiance mode 2 castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3, "each opponent takes 3");
}

/// Mystical Dispute counters a spell whose controller can't pay {3}.
#[test]
fn mystical_dispute_counters_unpaid_spell() {
    let mut g = two_player_game();
    // P1 casts a Lightning Bolt at P0.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("P1 bolt on stack");
    // P0 responds with Mystical Dispute; P1 has no mana to pay {3}.
    let disp = g.add_card_to_hand(0, catalog::mystical_dispute());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: disp, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mystical Dispute targets the bolt");
    drain_stack(&mut g);
    // Bolt countered → P0 took no damage.
    assert_eq!(g.players[0].life, 20, "bolt was countered, no damage");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "bolt in graveyard");
}

/// Plunge into Darkness mode 0 sacrifices a creature to gain 3 life.
#[test]
fn plunge_into_darkness_mode0_sacrifices_any_number_for_life() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Sacrifice both → gain 3 each = 6.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    let id = g.add_card_to_hand(0, catalog::plunge_into_darkness());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Plunge mode 0 castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear || c.id == bear2),
        "both creatures sacrificed");
    assert_eq!(g.players[0].life, life + 6, "gained 3 life per creature");
}

/// Coveted Jewel draws three cards when it enters.
#[test]
fn coveted_jewel_cast_etb_draws_three_cards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        let lid = g.next_id();
        g.players[0].library.push(crabomination::card::CardInstance::new(
            lid, catalog::grizzly_bears(), 0));
    }
    let id = g.add_card_to_hand(0, catalog::coveted_jewel());
    g.players[0].mana_pool.add_colorless(6);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coveted Jewel castable for {6}");
    drain_stack(&mut g);
    // -1 for the Jewel leaving hand, +3 drawn.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3, "drew 3 on ETB");
}

/// The Mightstone and Weakstone draws two cards (ETB mode 0).
#[test]
fn the_mightstone_and_weakstone_etb_draws_two() {
    let mut g = two_player_game();
    for _ in 0..2 {
        let lid = g.next_id();
        g.players[0].library.push(crabomination::card::CardInstance::new(
            lid, catalog::grizzly_bears(), 0));
    }
    let id = g.add_card_to_hand(0, catalog::the_mightstone_and_weakstone());
    g.players[0].mana_pool.add_colorless(5);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Mightstone castable for {5}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "drew 2 on ETB mode 0");
}

/// Kozilek's Command "choose two": the default picks run both the Scion
/// mode (X tokens) and the Draw-X mode in one cast.
#[test]
fn kozileks_command_chooses_two_modes() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::kozileks_command());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Kozilek's Command castable for X=2");
    drain_stack(&mut g);
    let scions = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Scion").count();
    assert_eq!(scions, 2, "Scion mode makes X=2 Eldrazi Scions");
    // Cast (-1) + Draw X=2 (+2) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "Draw-X mode also fired");
}

/// Eldrazi Confluence "choose three, modes may repeat": the Scion mode is
/// taken thrice by default, minting three tokens.
#[test]
fn eldrazi_confluence_chooses_scion_mode_three_times() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eldrazi_confluence());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Eldrazi Confluence castable for {4}");
    drain_stack(&mut g);
    let scions = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Scion").count();
    assert_eq!(scions, 3, "choose-three repeats the Scion mode for three tokens");
}

