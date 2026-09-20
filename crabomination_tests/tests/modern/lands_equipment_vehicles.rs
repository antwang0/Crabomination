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

/// Three Tree City taps for {C}, and its second ability scales on the
/// creature type it named as it entered — it shipped as Gemstone Mine
/// (charge counters, sacrifice on the last) under this name.
#[test]
fn three_tree_city_taps_for_colorless_then_scales_on_the_named_type() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::CreatureType(crabomination::card::CreatureType::Bear),
        DecisionAnswer::Color(Color::Green),
    ]));
    let id = g.add_card_to_hand(0, catalog::three_tree_city());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::Charge),
        0,
        "no charge counters — that was Gemstone Mine"
    );
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("the free colorless mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "the printed colorless ability");
    assert!(g.battlefield_find(id).is_some(), "and it does not sacrifice itself");

    // Two Bears out, {2} paid → the second ability makes two green.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hill_giant()); // not a Bear
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("{2}, {T}: choose a color, add one per creature of the named type");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "one per Bear, the Giant excluded");
}

/// The printed cost is "{T}, Sacrifice **another creature**" — it was a land
/// here until the printed-keyword ratchet turned the card up.
#[test]
fn wight_of_the_reliquary_sacrifices_a_creature_to_fetch_a_land() {
    let mut g = two_player_game();
    let wight = g.add_card_to_battlefield(0, catalog::wight_of_the_reliquary());
    g.clear_sickness(wight);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let keep = g.add_card_to_battlefield(0, catalog::mountain());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: wight, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("{T}, sacrifice another creature: search a land");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == wight), "Wight survives — 'another' creature");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "the Bear was sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == keep), "the land was not the cost");
    let land = g.battlefield_find(forest).expect("fetched land on battlefield");
    assert!(land.tapped, "fetched land enters tapped");
}

/// "+1/+1 for each **creature** card in your graveyard" — it counted lands
/// here until the printed-keyword ratchet turned the card up.
#[test]
fn wight_of_the_reliquary_grows_with_creatures_in_your_graveyard() {
    let mut g = two_player_game();
    let wight = g.add_card_to_battlefield(0, catalog::wight_of_the_reliquary());
    // Base 1/1 with no creature cards in graveyard.
    let c = g.compute_battlefield();
    let w = c.iter().find(|c| c.id == wight).unwrap();
    assert_eq!((w.power, w.toughness), (1, 1), "base 1/1");
    // Two creature cards in your graveyard → 3/3. Lands and the opponent's
    // graveyard don't count.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let c = g.compute_battlefield();
    let w = c.iter().find(|c| c.id == wight).unwrap();
    assert_eq!((w.power, w.toughness), (3, 3),
        "1 base + 2 creature cards in your graveyard (land and opp's ignored)");
    assert!(w.keywords().contains(&crabomination::card::Keyword::Vigilance), "printed vigilance");
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
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Haste), "boots grant haste");
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
    assert!(!pre.card_types().contains(&CardType::Creature), "uncrewed = not a creature");

    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::Crew { vehicle: chariot, crew_creatures: vec![b1, b2] })
        .expect("crew 4 satisfied by two 2/2s");
    let post = g.computed_permanent(chariot).unwrap();
    assert!(post.card_types().contains(&CardType::Creature), "crewed = creature");
    assert_eq!(post.power, 4);
    assert_eq!(post.toughness, 4);
    assert!(g.battlefield_find(b1).unwrap().tapped);
    assert!(g.battlefield_find(b2).unwrap().tapped);

    g.expire_end_of_turn_effects();
    let after = g.computed_permanent(chariot).unwrap();
    assert!(!after.card_types().contains(&CardType::Creature), "animation wears off EOT");
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
    assert!(cp.card_types().contains(&CardType::Creature));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Flying));

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
    assert!(pre.card_types().contains(&CardType::Land));
    assert!(!pre.card_types().contains(&CardType::Creature));

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate for {3}{W}{U}");
    drain_stack(&mut g);

    let post = g.computed_permanent(land).unwrap();
    assert!(post.card_types().contains(&CardType::Creature), "now a creature");
    assert!(post.card_types().contains(&CardType::Land), "still a land");
    assert_eq!(post.power, 4);
    assert_eq!(post.toughness, 4);
    assert!(post.keywords().contains(&Keyword::Flying));
    assert!(post.keywords().contains(&Keyword::Vigilance));
    assert!(post.subtypes().creature_types.contains(&CreatureType::Elemental));

    g.expire_end_of_turn_effects();
    let after = g.computed_permanent(land).unwrap();
    assert!(!after.card_types().contains(&CardType::Creature), "reverts to land EOT");
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
    assert!(post.keywords().contains(&Keyword::Unblockable));
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
    assert!(post.card_types().contains(&CardType::Land), "still a land");
    assert!(post.keywords().contains(&Keyword::Changeling));
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
    assert!(post.keywords().contains(&Keyword::Flying));
    assert!(post.keywords().contains(&Keyword::Infect));
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
    assert!(post.subtypes().creature_types.contains(&CreatureType::AssemblyWorker));
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
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thoughtseize castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == victim_card),
        "opp's nonland card discarded");
    assert_eq!(g.players[0].life, p0_life - 2, "caster loses 2 life");
}

/// Searing Blaze deals 1 and 1 without a land drop; 3 and 3 under
/// landfall.
#[test]
fn searing_blaze_burns_creature_and_player() {
    let mut g = two_player_game();
    // The default bot profile aims a hostile player slot at an
    // opponent (`EvalWeights::default()`); a bare test seat does not.
    g.players[0].hostile_player_targets = true;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::searing_blaze());
    g.players[0].mana_pool.add(Color::Red, 2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(1)], mode: None, x_value: None,
    }).expect("Searing Blaze castable for {R}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "2/2 survives 1 damage");
    assert_eq!(g.players[1].life, p1_life - 1, "opp takes 1");
    // Landfall: a land played this turn makes it 3 and 3.
    let mountain = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(mountain)).expect("land drop");
    let id = g.add_card_to_hand(0, catalog::searing_blaze());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(1)], mode: None, x_value: None,
    }).expect("Searing Blaze castable for {R}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 dies to 3 damage under landfall");
    // Landfall raises only the creature's half: the player still takes 1.
    assert_eq!(g.players[1].life, p1_life - 2, "opp takes 1, then 1 again");
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
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquisition castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "MV-2 nonland discarded");
}

/// Collective Defiance mode 0 deals 4 damage to a creature.
#[test]
fn collective_defiance_mode0_burns_a_creature() {
    let mut g = two_player_game();
    // The default bot profile aims a hostile player slot at an
    // opponent (`EvalWeights::default()`); a bare test seat does not.
    g.players[0].hostile_player_targets = true;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::collective_defiance());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Collective Defiance's creature mode castable for {1}{R}{R}");
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
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: Some(2), x_value: None,
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

/// Kozilek's Command "choose two": the default picks run the Spawn mode
/// (X 0/1 tokens with the mana ability) and the scry-X-then-draw-one mode.
///
/// It shipped with three of its four printed modes wrong — X **1/1 vanilla**
/// Eldrazi, a bare draw X, and a -X/-X pump where the printing exiles — and
/// no graveyard mode at all.
#[test]
fn kozileks_command_chooses_two_modes() {
    let mut g = two_player_game();
    // Both modes aim at *target player*, so the Spawn and the draw land on
    // seat 1 here — the printed card lets you point them at an opponent.
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::kozileks_command());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        // Each target-bearing mode among the default `picks` owns its own
        // cast-time slot: the Spawn mode takes slot 0, the scry mode slot 1.
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![crabomination::game::types::Target::Player(1)],
        mode: None,
        x_value: Some(2),
    }).expect("Kozilek's Command castable for X=2");
    drain_stack(&mut g);
    let spawn: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").collect();
    assert_eq!(spawn.len(), 2, "the Spawn mode makes X=2 tokens");
    assert!(spawn.iter().all(|c| c.controller == 1), "target player creates them");
    assert_eq!(
        (spawn[0].definition.power, spawn[0].definition.toughness),
        (0, 1),
        "0/1, not the 1/1 it shipped as"
    );
    assert_eq!(spawn[0].definition.activated_abilities.len(), 1, "sacrifice for {{C}}");
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "the caster draws nothing");
    assert_eq!(g.players[1].hand.len(), 1, "target player scries X, then draws *one*");
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


// ── The filter-land cycle, all ten in one table ──────────────────────────────

/// The ten Shadowmoor/Eventide filter lands share one body
/// (`sets::hybrid_filter_land`), and that body is **two** activated abilities: the
/// printed card is `{T}: Add {C}.` plus a single `{A/B}, {T}: Add {A}{A},
/// {A}{B}, or {B}{B}` whose payout is chosen as it resolves. Three of the ten
/// used to ship as three abilities, one per payout — the same set of outcomes
/// out of a land with four abilities instead of two.
#[test]
fn every_filter_land_is_two_abilities_over_its_own_hybrid_pair() {
    use crabomination::mana::ManaSymbol;
    let cycle: [(Factory, &str, Color, Color); 10] = [
        (catalog::mystic_gate, "Mystic Gate", Color::White, Color::Blue),
        (catalog::sunken_ruins, "Sunken Ruins", Color::Blue, Color::Black),
        (catalog::graven_cairns, "Graven Cairns", Color::Black, Color::Red),
        (catalog::fire_lit_thicket, "Fire-Lit Thicket", Color::Red, Color::Green),
        (catalog::wooded_bastion, "Wooded Bastion", Color::Green, Color::White),
        (catalog::fetid_heath, "Fetid Heath", Color::White, Color::Black),
        (catalog::cascade_bluffs, "Cascade Bluffs", Color::Blue, Color::Red),
        (catalog::twilight_mire, "Twilight Mire", Color::Black, Color::Green),
        (catalog::rugged_prairie, "Rugged Prairie", Color::Red, Color::White),
        (catalog::flooded_grove, "Flooded Grove", Color::Green, Color::Blue),
    ];
    for (factory, name, a, b) in cycle {
        let def = factory();
        assert_eq!(def.name, name);
        assert!(def.card_types.contains(&CardType::Land), "{name} is a land");
        assert!(def.subtypes.land_types.is_empty(), "{name} has no basic land type");
        assert!(def.triggered_abilities.is_empty(), "{name} enters untapped");
        assert_eq!(def.activated_abilities.len(), 2, "{name}: {{C}} plus one filter");
        let filter = &def.activated_abilities[1];
        assert!(filter.tap_cost, "{name}'s filter taps");
        assert_eq!(
            filter.mana_cost.symbols,
            vec![ManaSymbol::Hybrid(a, b)],
            "{name} costs one hybrid of its own pair",
        );
    }
}

/// And the filter itself: one hybrid in, two mana of the pair out. Which two
/// is the decider's call — the three printed payouts are exactly the three
/// ways two pips can be drawn from the pair, so the assertion is the count.
#[test]
fn a_filter_land_turns_one_hybrid_pip_into_two_of_its_pair() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let id = g.add_card_to_battlefield(0, catalog::twilight_mire());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("{B/G}, {T}: add two of B/G");
    drain_stack(&mut g);
    let pool = &g.players[0].mana_pool;
    assert_eq!(
        pool.amount(Color::Black) + pool.amount(Color::Green),
        2,
        "one pip in, two pips of the pair out",
    );
    assert!(g.battlefield_find(id).unwrap().tapped);
}

/// The Shards tri-lands are the other half of the cycle the Khans wedges
/// already shipped: enters tapped, three single-colour mana abilities in the
/// printed order.
#[test]
fn every_shard_tri_land_enters_tapped_and_taps_for_its_three() {
    let cycle: [(Factory, &str, [Color; 3]); 5] = [
        (catalog::arcane_sanctum, "Arcane Sanctum", [Color::White, Color::Blue, Color::Black]),
        (catalog::crumbling_necropolis, "Crumbling Necropolis", [Color::Blue, Color::Black, Color::Red]),
        (catalog::savage_lands, "Savage Lands", [Color::Black, Color::Red, Color::Green]),
        (catalog::jungle_shrine, "Jungle Shrine", [Color::Red, Color::Green, Color::White]),
        (catalog::seaside_citadel, "Seaside Citadel", [Color::Green, Color::White, Color::Blue]),
    ];
    for (factory, name, colors) in cycle {
        let def = factory();
        assert_eq!(def.name, name);
        assert!(def.card_types.contains(&CardType::Land), "{name} is a land");
        assert!(def.subtypes.land_types.is_empty(), "{name} has no basic land type");
        assert_eq!(def.activated_abilities.len(), 3, "{name} taps for three colours");
        assert!(def.triggered_abilities.is_empty(), "{name}: CR 614.1c, not a trigger");
        assert_eq!(def.static_abilities.len(), 1, "{name} enters tapped");
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let id = g.add_card_to_hand(0, factory());
        g.perform_action(GameAction::PlayLand(id)).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).unwrap().tapped, "{name} enters tapped");
        for (i, color) in colors.iter().enumerate() {
            let mut g = two_player_game();
            g.step = TurnStep::PreCombatMain;
            let id = g.add_card_to_battlefield(0, factory());
            g.perform_action(GameAction::ActivateAbility {
                card_id: id, ability_index: i, target: None, additional_targets: Vec::new(),
                x_value: None, mode: None,
            })
            .unwrap_or_else(|e| panic!("{name} ability {i}: {e:?}"));
            drain_stack(&mut g);
            assert_eq!(g.players[0].mana_pool.amount(*color), 1, "{name} ability {i}");
        }
    }
}

/// The *other* cycle called "filter lands": one `{1}, {T}: Add {A}{B}` and
/// nothing else, ten cards. The allied five shipped as a `Seq` of two
/// single-colour `OfColors` adds — the same two pips, but routed through the
/// decider's colour choice over a one-element palette. Both pips are fixed on
/// the printed card, so there is nothing to choose.
#[test]
fn every_pay_one_filter_land_is_one_ability_for_two_fixed_pips() {
    let cycle: [(Factory, &str, Color, Color); 10] = [
        (catalog::skycloud_expanse, "Skycloud Expanse", Color::White, Color::Blue),
        (catalog::darkwater_catacombs, "Darkwater Catacombs", Color::Blue, Color::Black),
        (catalog::shadowblood_ridge, "Shadowblood Ridge", Color::Black, Color::Red),
        (catalog::mossfire_valley, "Mossfire Valley", Color::Red, Color::Green),
        (catalog::sungrass_prairie, "Sungrass Prairie", Color::Green, Color::White),
        (catalog::desolate_mire, "Desolate Mire", Color::White, Color::Black),
        (catalog::ferrous_lake, "Ferrous Lake", Color::Blue, Color::Red),
        (catalog::viridescent_bog, "Viridescent Bog", Color::Black, Color::Green),
        (catalog::sunscorched_divide, "Sunscorched Divide", Color::Red, Color::White),
        (catalog::overflowing_basin, "Overflowing Basin", Color::Green, Color::Blue),
    ];
    for (factory, name, a, b) in cycle {
        let def = factory();
        assert_eq!(def.name, name);
        assert!(def.card_types.contains(&CardType::Land), "{name} is a land");
        assert!(def.subtypes.land_types.is_empty(), "{name} has no basic land type");
        assert!(def.triggered_abilities.is_empty(), "{name} enters untapped");
        assert_eq!(def.activated_abilities.len(), 1, "{name}: one ability, no {{C}} mode");

        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let id = g.add_card_to_battlefield(0, factory());
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        })
        .unwrap_or_else(|e| panic!("{name}: {{1}}, {{T}}: {e:?}"));
        drain_stack(&mut g);
        let pool = &g.players[0].mana_pool;
        assert_eq!(pool.amount(a), 1, "{name} adds its first pip");
        assert_eq!(pool.amount(b), 1, "{name} adds its second pip");
        assert_eq!(pool.colorless_amount(), 0, "{name} spent the {{1}}");
        assert!(g.battlefield_find(id).unwrap().tapped);
    }
}

// ── Reveal-or-tapped lands: a replacement, not a trigger ────────────────────

/// All fifteen "As this land enters, you may reveal a [X] card from your hand.
/// If you don't, this land enters tapped." lands. CR 614.12 makes that a
/// replacement effect, so the shape is one `EntersTappedUnless` static and
/// **no** triggered ability — thirteen of them shipped as an ETB trigger that
/// tapped the land after it was already on the battlefield.
#[test]
fn every_reveal_land_taps_itself_by_replacement_not_by_trigger() {
    let cycle: [(Factory, &str); 15] = [
        (catalog::port_town, "Port Town"),
        (catalog::choked_estuary, "Choked Estuary"),
        (catalog::foreboding_ruins, "Foreboding Ruins"),
        (catalog::game_trail, "Game Trail"),
        (catalog::fortified_village, "Fortified Village"),
        (catalog::frostboil_snarl, "Frostboil Snarl"),
        (catalog::furycalm_snarl, "Furycalm Snarl"),
        (catalog::necroblossom_snarl, "Necroblossom Snarl"),
        (catalog::shineshadow_snarl, "Shineshadow Snarl"),
        (catalog::vineglimmer_snarl, "Vineglimmer Snarl"),
        (catalog::ancient_amphitheater, "Ancient Amphitheater"),
        (catalog::aunties_hovel, "Auntie's Hovel"),
        (catalog::secluded_glen, "Secluded Glen"),
        (catalog::wanderwine_hub, "Wanderwine Hub"),
        (catalog::gilt_leaf_palace, "Gilt-Leaf Palace"),
    ];
    for (factory, name) in cycle {
        let def = factory();
        assert_eq!(def.name, name);
        assert!(def.card_types.contains(&CardType::Land), "{name} is a land");
        assert!(
            def.subtypes.land_types.is_empty(),
            "{name} prints no basic land type — the type is the reveal's filter",
        );
        assert_eq!(def.activated_abilities.len(), 2, "{name} taps for two colours");
        assert!(
            def.triggered_abilities.is_empty(),
            "{name}: 'as this enters' is CR 614.12, not a trigger",
        );
        assert_eq!(def.static_abilities.len(), 1, "{name} has the one replacement");

        // With nothing to reveal it enters tapped, and it is tapped the moment
        // it is on the battlefield — no priority window with a trigger on the
        // stack in which its controller could tap it for mana.
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let id = g.add_card_to_hand(0, factory());
        g.perform_action(GameAction::PlayLand(id)).unwrap();
        assert!(
            g.battlefield_find(id).unwrap().tapped,
            "{name} enters tapped, before anything resolves",
        );
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).unwrap().tapped, "{name} stays tapped");
    }
}

/// The other half of the replacement: a matching card in hand and the land
/// enters untapped. One card per filter kind — a land type (Port Town wants a
/// Plains or an Island) and a creature type (Gilt-Leaf Palace wants an Elf).
#[test]
fn a_reveal_land_enters_untapped_when_the_hand_can_show_it() {
    for (factory, name, reveal) in [
        (catalog::port_town as Factory, "Port Town", catalog::island as Factory),
        (catalog::gilt_leaf_palace as Factory, "Gilt-Leaf Palace", catalog::llanowar_elves as Factory),
    ] {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_hand(0, reveal());
        let id = g.add_card_to_hand(0, factory());
        g.perform_action(GameAction::PlayLand(id)).unwrap();
        drain_stack(&mut g);
        assert!(
            !g.battlefield_find(id).unwrap().tapped,
            "{name} enters untapped with a card to reveal",
        );
    }
}

/// The ten triomes: three basic land types, enters tapped, Cycling {3}, three
/// single-colour mana abilities in the printed order. ⚠ Cabaretti Courtyard
/// is deliberately absent — same set, same three colours, and its oracle
/// sacrifices itself to fetch a basic.
#[test]
fn every_triome_carries_its_three_types_and_cycling_three() {
    use crabomination::card::{Keyword, LandType};
    let cycle: [(Factory, &str, [LandType; 3], [Color; 3]); 10] = [
        (catalog::indatha_triome, "Indatha Triome",
         [LandType::Plains, LandType::Swamp, LandType::Forest],
         [Color::White, Color::Black, Color::Green]),
        (catalog::ketria_triome, "Ketria Triome",
         [LandType::Forest, LandType::Island, LandType::Mountain],
         [Color::Green, Color::Blue, Color::Red]),
        (catalog::raugrin_triome, "Raugrin Triome",
         [LandType::Island, LandType::Mountain, LandType::Plains],
         [Color::Blue, Color::Red, Color::White]),
        (catalog::savai_triome, "Savai Triome",
         [LandType::Mountain, LandType::Plains, LandType::Swamp],
         [Color::Red, Color::White, Color::Black]),
        (catalog::zagoth_triome, "Zagoth Triome",
         [LandType::Swamp, LandType::Forest, LandType::Island],
         [Color::Black, Color::Green, Color::Blue]),
        (catalog::jetmirs_garden, "Jetmir's Garden",
         [LandType::Mountain, LandType::Forest, LandType::Plains],
         [Color::Red, Color::Green, Color::White]),
        (catalog::raffines_tower, "Raffine's Tower",
         [LandType::Plains, LandType::Island, LandType::Swamp],
         [Color::White, Color::Blue, Color::Black]),
        (catalog::xanders_lounge, "Xander's Lounge",
         [LandType::Island, LandType::Swamp, LandType::Mountain],
         [Color::Blue, Color::Black, Color::Red]),
        (catalog::ziatoras_proving_ground, "Ziatora's Proving Ground",
         [LandType::Swamp, LandType::Mountain, LandType::Forest],
         [Color::Black, Color::Red, Color::Green]),
        (catalog::sparas_headquarters, "Spara's Headquarters",
         [LandType::Forest, LandType::Plains, LandType::Island],
         [Color::Green, Color::White, Color::Blue]),
    ];
    for (factory, name, types, colors) in cycle {
        let def = factory();
        assert_eq!(def.name, name);
        assert_eq!(def.subtypes.land_types, types.to_vec(), "{name}'s printed types");
        assert!(
            def.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))),
            "{name} has cycling",
        );
        assert_eq!(def.activated_abilities.len(), 3, "{name} taps for three");

        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let id = g.add_card_to_hand(0, factory());
        g.perform_action(GameAction::PlayLand(id)).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).unwrap().tapped, "{name} enters tapped");
        for (i, color) in colors.iter().enumerate() {
            let mut g = two_player_game();
            g.step = TurnStep::PreCombatMain;
            let id = g.add_card_to_battlefield(0, factory());
            g.perform_action(GameAction::ActivateAbility {
                card_id: id, ability_index: i, target: None, additional_targets: Vec::new(),
                x_value: None, mode: None,
            })
            .unwrap_or_else(|e| panic!("{name} ability {i}: {e:?}"));
            drain_stack(&mut g);
            assert_eq!(g.players[0].mana_pool.amount(*color), 1, "{name} ability {i}");
        }
    }
}

// ── Utility lands: {T}: Add {C} plus one activated ability ──────────────────

/// Spire of Industry's second ability is gated on controlling an artifact and
/// costs a life; the colourless half is ungated.
#[test]
fn spire_of_industry_needs_an_artifact_and_a_life() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::spire_of_industry());
    let act = |g: &mut crabomination::game::GameState, i: usize| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: i, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        })
    };
    assert!(act(&mut g, 1).is_err(), "no artifact, no colour");

    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::spire_of_industry());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("with an artifact out");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "paid 1 life");
    assert_eq!(
        [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
            .iter()
            .map(|c| g.players[0].mana_pool.amount(*c))
            .sum::<u32>(),
        1,
        "one mana of some colour",
    );
}

/// Cabal Stronghold counts **basic** Swamps — a Swamp dual has the land type
/// without the supertype and does not count.
#[test]
fn cabal_stronghold_counts_only_basic_swamps() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::cabal_stronghold());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.add_card_to_battlefield(0, catalog::overgrown_tomb());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("{3}, {T}");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].mana_pool.amount(Color::Black),
        3,
        "three basic Swamps; Overgrown Tomb has the type, not the supertype",
    );
}

/// Gavony Township puts a counter on each creature you control, and on
/// nobody else's.
#[test]
fn gavony_township_counters_your_team_only() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::gavony_township());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("{2}{G}{W}, {T}");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "+1/+1");
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 2, "not theirs");
}

/// Hall of Heliod's Generosity puts a graveyard enchantment back on top of
/// the library, and it is legendary.
#[test]
fn hall_of_heliods_generosity_recurs_an_enchantment_to_the_top() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    assert!(catalog::hall_of_heliods_generosity().is_legendary());
    let land = g.add_card_to_battlefield(0, catalog::hall_of_heliods_generosity());
    let aura = g.add_card_to_graveyard(0, catalog::pacifism());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: Some(Target::Permanent(aura)),
        additional_targets: Vec::new(),
        x_value: None,
        mode: None,
    })
    .expect("{1}{W}, {T}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty(), "left the graveyard");
    assert_eq!(
        g.players[0].library.last().map(|c| c.definition.name),
        Some("Pacifism"),
        "on top of the library",
    );
}

// ── EDHREC Commander staples that needed no new primitive ───────────────────

/// Parallel Lives is Doubling Season's token half, and it is controller-
/// scoped: Dragon Fodder's two Goblins become four for its controller and
/// stay two for the opponent.
#[test]
fn parallel_lives_doubles_only_your_tokens() {
    let goblins = |g: &crabomination::game::GameState, seat: usize| {
        g.battlefield
            .iter()
            .filter(|c| c.definition.name == "Goblin" && c.controller == seat)
            .count()
    };
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::parallel_lives());

    let mine = g.add_card_to_hand(0, catalog::dragon_fodder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: mine, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Dragon Fodder for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(goblins(&g, 0), 4, "two Goblins, doubled");

    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let theirs = g.add_card_to_hand(1, catalog::dragon_fodder());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: theirs, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("their Dragon Fodder");
    drain_stack(&mut g);
    assert_eq!(goblins(&g, 1), 2, "not under their control");
    assert_eq!(goblins(&g, 0), 4, "and yours are unchanged");
}

/// Avacyn hands indestructible to every OTHER permanent you control —
/// permanents, not creatures — and to nothing of the opponent's.
#[test]
fn avacyn_makes_your_whole_board_indestructible() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let avacyn = g.add_card_to_battlefield(0, catalog::avacyn_angel_of_hope());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let rock = g.add_card_to_battlefield(0, catalog::sol_ring());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let kw = |g: &crabomination::game::GameState, id| {
        g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Indestructible)
    };
    assert!(kw(&g, avacyn), "printed on Avacyn herself");
    assert!(kw(&g, bears), "a creature");
    assert!(kw(&g, land), "a land — the filter is permanents, not creatures");
    assert!(kw(&g, rock), "an artifact");
    assert!(!kw(&g, theirs), "and nothing of theirs");
}

// ── "Enters tapped unless [a board fact]" is a replacement too ──────────────

/// The ten slow lands and the ten battle lands print the same shape of
/// clause — "this land enters tapped unless [something about your board]" —
/// and CR 614.1c makes it a replacement. The battle lands always had it; the
/// slow lands shipped as an `EntersBattlefield` trigger, which leaves the
/// land on the battlefield untapped with the trigger on the stack.
#[test]
fn every_conditional_tapland_taps_by_replacement_not_by_trigger() {
    let slow: [(Factory, &str); 10] = [
        (catalog::deathcap_glade, "Deathcap Glade"),
        (catalog::deserted_beach, "Deserted Beach"),
        (catalog::dreamroot_cascade, "Dreamroot Cascade"),
        (catalog::haunted_ridge, "Haunted Ridge"),
        (catalog::overgrown_farmland, "Overgrown Farmland"),
        (catalog::rockfall_vale, "Rockfall Vale"),
        (catalog::shattered_sanctum, "Shattered Sanctum"),
        (catalog::shipwreck_marsh, "Shipwreck Marsh"),
        (catalog::stormcarved_coast, "Stormcarved Coast"),
        (catalog::sundown_pass, "Sundown Pass"),
    ];
    let battle: [(Factory, &str); 10] = [
        (catalog::canopy_vista, "Canopy Vista"),
        (catalog::cinder_glade, "Cinder Glade"),
        (catalog::eclipsed_steppe, "Eclipsed Steppe"),
        (catalog::prairie_stream, "Prairie Stream"),
        (catalog::radiant_summit, "Radiant Summit"),
        (catalog::scorched_geyser, "Scorched Geyser"),
        (catalog::smoldering_marsh, "Smoldering Marsh"),
        (catalog::sodden_verdure, "Sodden Verdure"),
        (catalog::sunken_hollow, "Sunken Hollow"),
        (catalog::vernal_fen, "Vernal Fen"),
    ];
    for (factory, name) in slow.iter().chain(battle.iter()) {
        let def = factory();
        assert_eq!(def.name, *name);
        assert!(
            def.triggered_abilities.is_empty(),
            "{name}: 'enters tapped unless' is CR 614.1c, not a trigger",
        );
        assert_eq!(def.static_abilities.len(), 1, "{name} has the one replacement");
    }
    // ⚠ The two families spell their mana differently and that is not a
    // defect: a slow land prints no basic land type and carries two explicit
    // `{T}: Add {C}` abilities, while a battle land is a TYPED dual whose
    // colours also come from its land types, and models the pair as one.
    for (factory, name) in slow {
        assert_eq!(factory().activated_abilities.len(), 2, "{name}: two abilities");
        assert!(factory().subtypes.land_types.is_empty(), "{name}: untyped");
    }
    for (factory, name) in battle {
        assert_eq!(factory().subtypes.land_types.len(), 2, "{name}: a typed dual");
    }
}

/// And the behaviour either way round, on the pair whose threshold is the
/// printed "two or more OTHER lands": tapped on an empty board, and untapped
/// once two other lands are out — with the land already tapped before
/// anything resolves.
#[test]
fn a_slow_land_reads_the_other_lands_and_is_tapped_on_arrival() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let id = g.add_card_to_hand(0, catalog::deserted_beach());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    assert!(
        g.battlefield_find(id).unwrap().tapped,
        "no other lands: tapped, and tapped before anything resolves",
    );

    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::deserted_beach());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield_find(id).unwrap().tapped, "two other lands: untapped");

    // One other land is not two.
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::deserted_beach());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "one other land: still tapped");
}

// ── The top-1000 Commander Equipment gap ───────────────────────────────────

/// A legendary 2/2 to host the two "Equip legendary creature" cards, so the
/// restricted cost has something to match and the unrestricted one has
/// something to be compared against.
fn legendary_bear() -> crabomination::card::CardDefinition {
    crabomination::card::CardDefinition {
        name: "Test Legendary Bear",
        supertypes: vec![crabomination::card::Supertype::Legendary],
        ..catalog::grizzly_bears()
    }
}

/// Blackblade Reforged: "+1/+1 for each land you control", counted on the
/// Equipment controller's lands — and the **restricted second equip cost**.
/// Equip legendary creature {3} beats Equip {7} by four mana, so three
/// colourless is enough for a legendary host and not for anything else.
#[test]
fn blackblade_reforged_scales_with_lands_and_equips_a_legend_for_three() {
    let mut g = two_player_game();
    let eq = g.add_card_to_battlefield(0, catalog::blackblade_reforged());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    assert!(
        g.perform_action(GameAction::Equip { equipment: eq, target: plain }).is_err(),
        "a nonlegendary host pays the printed Equip {{7}}",
    );

    let legend = g.add_card_to_battlefield(0, legendary_bear());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::plains());
    g.perform_action(GameAction::Equip { equipment: eq, target: legend })
        .expect("Equip legendary creature {3}");
    let cp = g.computed_permanent(legend).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 plus one per land, two lands");

    // The count is live: a third land moves it again.
    g.add_card_to_battlefield(0, catalog::mountain());
    let cp = g.computed_permanent(legend).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "three lands");
}

/// Champion's Helm: the +2/+2 is unconditional and only the hexproof is gated
/// on the host being legendary, so the two clauses have to come apart.
#[test]
fn champions_helm_pumps_anything_and_shields_only_a_legend() {
    let mut g = two_player_game();
    let helm = g.add_card_to_battlefield(0, catalog::champions_helm());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: helm, target: plain }).expect("Equip {1}");
    let cp = g.computed_permanent(plain).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2 with no legendary gate");
    assert!(
        !cp.keywords().contains(&crabomination::card::Keyword::Hexproof),
        "a nonlegendary host gets no hexproof",
    );

    let legend = g.add_card_to_battlefield(0, legendary_bear());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: helm, target: legend }).expect("re-equip");
    let cp = g.computed_permanent(legend).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "the pump is the same");
    assert!(
        cp.keywords().contains(&crabomination::card::Keyword::Hexproof),
        "and a legendary host also has hexproof",
    );
}

/// Cast Mithril Coat for its printed {3} and settle the stack, returning its
/// battlefield id.
fn cast_the_coat(g: &mut GameState) -> crabomination::card::CardId {
    let id = g.add_card_to_hand(0, catalog::mithril_coat());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Mithril Coat for {3}");
    drain_stack(g);
    id
}

/// Mithril Coat: two printed indestructibles that are different clauses. The
/// keyword is on the Coat; the `equipped_bonus` is on whatever it is attached
/// to. The ETB attach finds the legendary creature on its own.
#[test]
fn mithril_coat_attaches_itself_and_both_indestructibles_are_real() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let legend = g.add_card_to_battlefield(0, legendary_bear());
    // ⚠ Cast it: `add_card_to_battlefield` places the instance and fires no
    // entry event at all, so an ETB trigger tested that way reads as absent.
    let coat = cast_the_coat(&mut g);

    assert_eq!(
        g.battlefield_find(coat).unwrap().attached_to,
        Some(legend),
        "the enters trigger attached it with no equip cost paid",
    );
    assert!(
        g.computed_permanent(coat).unwrap().keywords().contains(&Keyword::Indestructible),
        "the Coat's own printed indestructible",
    );
    assert!(
        g.computed_permanent(legend).unwrap().keywords().contains(&Keyword::Indestructible),
        "and the one it grants its host",
    );
    assert!(
        g.battlefield_find(coat).unwrap().definition.keywords.contains(&Keyword::Flash),
        "flash is what the card is played for",
    );
}

/// With no legendary creature the attach trigger has no legal target, so it is
/// removed from the stack (CR 603.3d) — and the Coat stays on the battlefield
/// unattached rather than the trigger taking it anywhere.
#[test]
fn cr_603_3d_mithril_coat_stays_put_with_nothing_legendary_to_attach_to() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let coat = cast_the_coat(&mut g);
    let on = g.battlefield_find(coat).expect("still on the battlefield");
    assert_eq!(on.attached_to, None, "no legal target, nothing attached");
}

/// The Reaver Cleaver: +1/+1, trample, and a granted combat-damage trigger
/// whose count is the damage dealt. CR 702.6e makes the granted ability the
/// creature's, so the Treasures arrive under the creature's controller.
#[test]
fn the_reaver_cleaver_mints_one_treasure_per_point_of_combat_damage() {
    let mut g = two_player_game();
    let eq = g.add_card_to_battlefield(0, catalog::the_reaver_cleaver());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: eq, target: bear }).expect("Equip {3}");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 plus +1/+1");
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Trample));

    let treasures = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.definition.name == "Treasure" && c.controller == 0).count()
    };
    assert_eq!(treasures(&g), 0, "nothing before combat");

    g.step = TurnStep::PreCombatMain;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("to attackers");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    for _ in 0..12 {
        if g.players[1].life < 20 && treasures(&g) > 0 {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 17, "a 3/3 connected");
    assert_eq!(treasures(&g), 3, "that many Treasures — one per point of damage");
}

// ── "Whenever an opponent …" watchers, at four seats ───────────────────────

/// Archivist of Oghma: "Whenever an opponent searches their library, you gain
/// 1 life and draw a card." The scope is per searching player, so each
/// opponent that tutors is its own trigger — the card is three times itself
/// in a pod, which is what makes it a staple.
#[test]
fn archivist_of_oghma_fires_once_per_opponent_that_searches() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::archivist_of_oghma());
    g.players[0].hand.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    let life = g.players[0].life;

    for seat in [1usize, 2, 3] {
        g.add_card_to_library(seat, catalog::forest());
        let tutor = g.add_card_to_hand(seat, catalog::lay_of_the_land());
        g.players[seat].mana_pool.add(Color::Green, 1);
        g.active_player_idx = seat;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = seat;
        g.perform_action(GameAction::CastSpell {
            card_id: tutor,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("tutor");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life + 3, "one life per opponent that searched");
    assert_eq!(g.players[0].hand.len(), 3, "and one card each");
}

/// And it watches opponents only — the controller's own tutor is not "an
/// opponent searches".
#[test]
fn archivist_of_oghma_ignores_its_own_controllers_search() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::archivist_of_oghma());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::forest());
    let tutor = g.add_card_to_hand(0, catalog::lay_of_the_land());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: tutor,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tutor");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "no life — you are not your own opponent");
}

/// Mangara, the Diplomat: "Whenever an opponent attacks with creatures, if two
/// or more of those creatures are attacking you and/or planeswalkers you
/// control, draw a card."
///
/// At four seats the gate is the point: seat 1 declares three attackers, only
/// one of them at Mangara's controller, and the card draws nothing. The
/// undirected "attacked with two or more" would have fired.
#[test]
fn cr_506_2_mangara_ignores_an_attack_that_is_mostly_pointed_elsewhere() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::mangara_the_diplomat());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::plains());
    let mut swing = Vec::new();
    for target in [AttackTarget::Player(0), AttackTarget::Player(2), AttackTarget::Player(3)] {
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(a);
        swing.push(Attack { attacker: a, target });
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(swing).expect("declare across three defenders");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "only one of the three is attacking seat 0");
}

/// Two at the player draws one card — **one**, not one per attacker: the
/// trigger is `once_per_batch` (CR 603.2c, one declaration is one event).
#[test]
fn cr_603_2c_mangara_draws_one_card_for_a_two_creature_attack_on_its_controller() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::mangara_the_diplomat());
    g.players[0].hand.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    let mut swing = Vec::new();
    for target in [AttackTarget::Player(0), AttackTarget::Player(0), AttackTarget::Player(2)] {
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(a);
        swing.push(Attack { attacker: a, target });
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(swing).expect("declare");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "one declaration, one card");
}

/// The planeswalker half of the same clause: one creature at the player and
/// one at a planeswalker they control is two, because the card says "you
/// and/or planeswalkers you control".
#[test]
fn cr_506_2_mangara_counts_a_planeswalker_attacker_towards_its_two() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::mangara_the_diplomat());
    let pw = g.add_card_to_battlefield(0, catalog::jace_beleren());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::plains());
    let mut swing = Vec::new();
    for target in [AttackTarget::Player(0), AttackTarget::Planeswalker(pw)] {
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(a);
        swing.push(Attack { attacker: a, target });
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(swing).expect("declare");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "the planeswalker's attacker counts");
}

/// The second clause: "Whenever an opponent casts their second spell each
/// turn, draw a card." Per opponent, and on their second spell only.
#[test]
fn mangara_draws_on_an_opponents_second_spell_and_not_their_first() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::mangara_the_diplomat());
    g.players[0].hand.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    let cast_a_bear = |g: &mut GameState| {
        let id = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast a bear");
        drain_stack(g);
    };
    cast_a_bear(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "the first spell does nothing");
    cast_a_bear(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "the second draws");
    cast_a_bear(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "and the third does not — it is not the second");
}

// ── Trouble in Pairs ───────────────────────────────────────────────────────

/// Run to the start of the next turn, so the extra-turn / pass decision at the
/// turn boundary really happens.
fn run_to_next_turn(g: &mut GameState) {
    let started = g.turn_number;
    for _ in 0..200 {
        if g.turn_number != started {
            return;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    panic!("the turn never ended");
}

/// CR 614 — "If an opponent would begin an extra turn, that player skips that
/// turn instead." The charge is still spent, which is what keeps one Time
/// Warp from re-offering the same extra turn on every pass.
#[test]
fn cr_614_trouble_in_pairs_eats_an_opponents_extra_turn() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::trouble_in_pairs());
    g.players[1].extra_turns = 1;
    g.active_player_idx = 1;

    run_to_next_turn(&mut g);
    assert_eq!(g.active_player_idx, 2, "the extra turn was skipped, not taken");
    assert!(!g.current_turn_is_extra);
    assert_eq!(g.players[1].extra_turns, 0, "and the charge was spent, not left pending");
}

/// …and it is an **opponent** clause: the enchantment's own controller keeps
/// their extra turns. Same board, the charge on seat 0 instead.
#[test]
fn trouble_in_pairs_leaves_its_own_controllers_extra_turn_alone() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::trouble_in_pairs());
    g.players[0].extra_turns = 1;
    g.active_player_idx = 0;

    run_to_next_turn(&mut g);
    assert_eq!(g.active_player_idx, 0, "seat 0 keeps the turn");
    assert!(g.current_turn_is_extra);
    assert_eq!(g.players[0].extra_turns, 0);
}

/// With no Trouble in Pairs out the extra turn is taken normally — the
/// control that says the two tests above are about the card.
#[test]
fn an_extra_turn_is_taken_normally_without_the_enchantment() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.players[1].extra_turns = 1;
    g.active_player_idx = 1;

    run_to_next_turn(&mut g);
    assert_eq!(g.active_player_idx, 1, "seat 1 takes its extra turn");
    assert!(g.current_turn_is_extra);
}

/// "Whenever an opponent … draws their second card each turn … you draw a
/// card" — per opponent, on their second draw only.
#[test]
fn trouble_in_pairs_draws_on_an_opponents_second_draw() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::trouble_in_pairs());
    g.players[0].hand.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.players[1].hand.clear();

    let draw_for_seat_one = |g: &mut GameState| {
        let mut events = Vec::new();
        g.draw_one(1, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(g);
    };
    draw_for_seat_one(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "their first draw does nothing");
    draw_for_seat_one(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "their second draws");
    draw_for_seat_one(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "and their third does not");
}

/// CR 506.2 — "attacks **you** with two or more creatures". Unlike Mangara,
/// an attacker on a planeswalker you control is **not** attacking you, so one
/// at the player plus one at the planeswalker is one, not two.
#[test]
fn cr_506_2_trouble_in_pairs_does_not_count_an_attack_on_your_planeswalker() {
    use crabomination::game::multi_player_game;
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::trouble_in_pairs());
    let pw = g.add_card_to_battlefield(0, catalog::jace_beleren());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::plains());
    let mut swing = Vec::new();
    for target in [AttackTarget::Player(0), AttackTarget::Planeswalker(pw)] {
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(a);
        swing.push(Attack { attacker: a, target });
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(swing).expect("declare");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        0,
        "only one creature is attacking the player — the printed word is \"you\"",
    );

    // Two at the player is the printed two.
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::trouble_in_pairs());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::plains());
    let mut swing = Vec::new();
    for _ in 0..2 {
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(a);
        swing.push(Attack { attacker: a, target: AttackTarget::Player(0) });
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(swing).expect("declare");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "two at the player draws one card");
}

// ── The doubling / tripling replacements ───────────────────────────────────

/// Alhammarret's Archive doubles life gain and doubles draws — **except** the
/// first draw of your own draw step, which is the clause that separates it
/// from Thought Reflection.
#[test]
fn cr_121_2a_alhammarrets_archive_doubles_draws_but_not_the_draw_step_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::alhammarrets_archive());
    g.players[0].hand.clear();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::plains());
    }

    // The first draw of your own draw step: one card.
    g.step = TurnStep::Draw;
    g.active_player_idx = 0;
    g.players[0].cards_drawn_this_step = 0;
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    assert_eq!(g.players[0].hand.len(), 1, "the draw-step draw is not doubled");

    // A second draw in the same step is doubled.
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    assert_eq!(g.players[0].hand.len(), 3, "the second draw of the step draws two");

    // And a draw outside the draw step is doubled.
    g.step = TurnStep::PreCombatMain;
    g.players[0].cards_drawn_this_step = 0;
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    assert_eq!(g.players[0].hand.len(), 5, "a main-phase draw draws two");
}

/// ⚠ "the first one you draw in each of **your** draw steps" — a draw taken
/// during someone *else's* draw step is not in your draw step, and is doubled.
#[test]
fn cr_121_2a_the_archives_exception_is_your_own_draw_step_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::alhammarrets_archive());
    g.players[0].hand.clear();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::plains());
    }
    g.step = TurnStep::Draw;
    g.active_player_idx = 1;
    g.players[0].cards_drawn_this_step = 0;
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    assert_eq!(g.players[0].hand.len(), 2, "not your draw step, so it doubles");
}

/// The Archive's other half, and the control that it is the Archive doing it.
#[test]
fn alhammarrets_archive_doubles_life_gain() {
    let mut g = two_player_game();
    let before = g.players[0].life;
    g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, before + 3, "no Archive, no doubling");

    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::alhammarrets_archive());
    let before = g.players[0].life;
    g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, before + 6, "twice that much life instead");
}

/// Teferi's Ageless Insight is the Archive's draw half and nothing else — so
/// it must *not* double life gain.
#[test]
fn teferis_ageless_insight_doubles_draws_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teferis_ageless_insight());
    g.players[0].hand.clear();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::plains());
    }
    let before = g.players[0].life;
    g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, before + 3, "no life clause on this one");

    g.step = TurnStep::PreCombatMain;
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    assert_eq!(g.players[0].hand.len(), 2, "but the draws still double");
}

/// CR 614.2 — "triple that damage", and it has to be **three**, not a second
/// doubling. Lightning Bolt's 3 becomes 9.
#[test]
fn cr_614_2_fiery_emancipation_triples_your_sources_damage() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::fiery_emancipation());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 9, "3 tripled is 9, not 6 and not 12");
}

/// City on Fire is the same multiplier, and two of them compose to ×9 rather
/// than to ×6 — the reason the factor is a multiplier and not a count.
#[test]
fn cr_614_2_two_triplers_compose_to_nine() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::fiery_emancipation());
    g.add_card_to_battlefield(0, catalog::city_on_fire());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 27, "3 x 3 x 3");
}

/// …and it is *your* sources: an opponent's Bolt is unaffected.
#[test]
fn fiery_emancipation_does_not_triple_an_opponents_damage() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::fiery_emancipation());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before - 3, "their source, their damage");
}

// ── Vivid — one mana of EACH colour among your permanents ──────────────────

fn tap_for_mana(g: &mut GameState, id: crabomination::card::CardId) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Vivid tap");
    drain_stack(g);
}

fn pool(g: &GameState, seat: usize) -> Vec<(Color, u32)> {
    [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
        .into_iter()
        .map(|c| (c, g.players[seat].mana_pool.amount(c)))
        .filter(|(_, n)| *n > 0)
        .collect()
}

/// Bloom Tender: "For **each** color among permanents you control, add one
/// mana of that color." A mono-green board taps for {G}; adding a white and a
/// blue permanent makes the same Elf tap for {W}{U}{G}.
///
/// ⚠ This is the clause that separates Vivid from Meteor Crater's "choose a
/// color of a permanent you control": the two agree exactly on a one-colour
/// board, so the single-colour case cannot tell them apart.
#[test]
fn bloom_tender_adds_one_mana_of_each_colour_among_your_permanents() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let tender = g.add_card_to_battlefield(0, catalog::bloom_tender());
    g.clear_sickness(tender);
    tap_for_mana(&mut g, tender);
    assert_eq!(pool(&g, 0), vec![(Color::Green, 1)], "the Elf is the only permanent, and it is green");

    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let tender = g.add_card_to_battlefield(0, catalog::bloom_tender());
    g.clear_sickness(tender);
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    g.add_card_to_battlefield(0, catalog::savannah_lions()); // white
    g.add_card_to_battlefield(0, catalog::delver_of_secrets()); // blue
    tap_for_mana(&mut g, tender);
    assert_eq!(
        pool(&g, 0),
        vec![(Color::White, 1), (Color::Blue, 1), (Color::Green, 1)],
        "one of EACH colour, not one chosen from among them",
    );
}

/// An opponent's colours are not "among permanents you control", and a
/// colourless board produces nothing at all.
#[test]
fn bloom_tender_reads_only_your_own_board() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let tender = g.add_card_to_battlefield(0, catalog::bloom_tender());
    g.clear_sickness(tender);
    g.add_card_to_battlefield(1, catalog::savannah_lions()); // theirs, white
    tap_for_mana(&mut g, tender);
    assert_eq!(pool(&g, 0), vec![(Color::Green, 1)], "their white is not yours");
}

/// Faeburrow Elder prints a **0/0** body and lives on its own pump: it is
/// itself a G/W permanent you control, so it is a 2/2 on an otherwise empty
/// board — and grows with each new colour.
#[test]
fn faeburrow_elder_is_a_two_two_alone_and_grows_with_each_colour() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let elder = g.add_card_to_battlefield(0, catalog::faeburrow_elder());
    let cp = g.computed_permanent(elder).expect("a 0/0 must not have died");
    assert_eq!((cp.power, cp.toughness), (2, 2), "its own two colours");
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Vigilance));

    g.add_card_to_battlefield(0, catalog::delver_of_secrets()); // blue
    let cp = g.computed_permanent(elder).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "a third colour");

    // And the mana ability reads the same set.
    g.clear_sickness(elder);
    tap_for_mana(&mut g, elder);
    assert_eq!(
        pool(&g, 0),
        vec![(Color::White, 1), (Color::Blue, 1), (Color::Green, 1)],
        "the pump and the mana read one colour set",
    );
}

// ── Blink — exile a permanent you control, then return it ──────────────────

// The observable signature of a blink in this engine: the permanent comes
// back **untapped, with its counters gone and summoning sickness back** — CR
// 400.7's "new object" in every respect a card can read.
//
// ⚠ **Not by `CardId`.** `Effect::ExileAndReturnToOwner` moves the same
// instance out and back, so the handle survives; the *state* does not. See
// ENGINE_BACKLOG on what that costs.
//
// ⚠ And both cards say "up to **one target** [permanent] you control", so on
// a board with two legal ones the auto-picker chooses. A test that names the
// expected victim asserts the picker's heuristic, not the card.

/// Displacer Kitten: "Whenever you cast a **noncreature** spell, exile up to
/// one target nonland permanent you control, then return it."
#[test]
fn displacer_kitten_blinks_on_a_noncreature_spell() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::displacer_kitten());
    // The only other nonland permanent, so the picker has one real choice
    // besides the Kitten itself — and a counter to lose either way.
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(bears).unwrap().tapped = true;

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast a noncreature spell");
    drain_stack(&mut g);

    let blinked = g.battlefield.iter().any(|c| {
        c.controller == 0 && !c.tapped && c.counter_count(CounterType::PlusOnePlusOne) == 0
            && c.definition.name == "Grizzly Bears"
    });
    let kitten_blinked = g
        .battlefield
        .iter()
        .any(|c| c.definition.name == "Displacer Kitten" && c.summoning_sick);
    assert!(
        blinked || kitten_blinked,
        "one of the two nonland permanents came back reset",
    );
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0).count(),
        2,
        "and nothing was lost on the way",
    );
}

/// …and **not** on a creature spell: "noncreature" is the printed word and is
/// the whole restriction on the trigger.
#[test]
fn displacer_kitten_ignores_a_creature_spell() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::displacer_kitten());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(bears).unwrap().tapped = true;

    let more = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: more,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast a creature spell");
    drain_stack(&mut g);

    let b = g.battlefield_find(bears).expect("still there");
    assert!(b.tapped, "still tapped");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1, "and still counting");
}

/// Teleportation Circle blinks at the beginning of **your** end step. Only the
/// Sol Ring is legal (the Circle is an enchantment), so this one can name its
/// victim.
#[test]
fn teleportation_circle_blinks_an_artifact_at_your_end_step() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::teleportation_circle());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.battlefield_find_mut(ring).unwrap().tapped = true;

    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(&mut g);

    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Sol Ring")
        .expect("it came back");
    assert!(!back.tapped, "it returned untapped — the tap did not come with it");
}

/// ⚠ "Up to one target" with nothing legal must still **resolve**, not be
/// removed from the stack: `min: 0` means the trigger has no target to miss
/// (CR 603.3d applies only when every required target is illegal). The Circle
/// alone is the case — an Enchantment is neither an artifact nor a creature.
#[test]
fn cr_603_3d_teleportation_circle_resolves_with_nothing_to_blink() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let circle = g.add_card_to_battlefield(0, catalog::teleportation_circle());

    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(circle).is_some(), "the Circle is still there and nothing panicked");
}

// ── {3} mana artifacts with a rider ────────────────────────────────────────

/// Relic of Legends: two mana a turn off one artifact, because the second
/// ability taps a **legendary creature you control** rather than the Relic.
#[test]
fn relic_of_legends_taps_itself_and_then_a_legend() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let relic = g.add_card_to_battlefield(0, catalog::relic_of_legends());
    let legend = g.add_card_to_battlefield(0, legendary_bear());
    g.clear_sickness(legend);

    tap_for_mana(&mut g, relic);
    assert_eq!(g.players[0].mana_pool.total(), 1, "the Relic's own tap");
    assert!(g.battlefield_find(relic).unwrap().tapped);

    g.perform_action(GameAction::ActivateAbility {
        card_id: relic,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap a legend for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2, "and a second off the legend");
    assert!(g.battlefield_find(legend).unwrap().tapped, "the legend paid the cost");
}

/// ⚠ The printed cost is "Tap **an untapped** legendary creature you control",
/// and both words are load-bearing: a nonlegendary creature cannot pay it, and
/// neither can a legend that is already tapped.
#[test]
fn relic_of_legends_second_ability_needs_an_untapped_legend() {
    let activate = |g: &mut GameState, relic| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: relic,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };

    // A nonlegendary creature is not a legal cost.
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let relic = g.add_card_to_battlefield(0, catalog::relic_of_legends());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bears);
    assert!(activate(&mut g, relic).is_err(), "Grizzly Bears is not legendary");

    // Nor is a legend that is already tapped.
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let relic = g.add_card_to_battlefield(0, catalog::relic_of_legends());
    let legend = g.add_card_to_battlefield(0, legendary_bear());
    g.clear_sickness(legend);
    g.battlefield_find_mut(legend).unwrap().tapped = true;
    assert!(activate(&mut g, relic).is_err(), "already tapped, so it cannot be tapped again");
}

/// Decanter of Endless Water: a mana rock that also turns off the cleanup
/// discard (CR 514.1).
#[test]
fn cr_514_1_decanter_of_endless_water_removes_the_maximum_hand_size() {
    let mut g = two_player_game();
    assert_eq!(g.effective_max_hand_size(0), Some(7), "the ordinary maximum, first");
    g.add_card_to_battlefield(0, catalog::decanter_of_endless_water());
    assert!(g.effective_max_hand_size(0).is_none(), "no maximum hand size");
    assert_eq!(g.effective_max_hand_size(1), Some(7), "and it is *your* hand, not theirs");
}

/// …and end to end through a real cleanup step: eleven cards survive the turn.
///
/// ⚠ The library is stocked on purpose. `two_player_game()` starts with an
/// empty one, so passing priority far enough to reach a cleanup step runs the
/// seat into its own draw and the turn ends in `GameAlreadyOver` — a fixture
/// failure that reads exactly like the card not working.
#[test]
fn cr_514_1_the_decanter_keeps_eleven_cards_through_cleanup() {
    let keep = |decanter: bool| {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        if decanter {
            g.add_card_to_battlefield(0, catalog::decanter_of_endless_water());
        }
        for seat in [0usize, 1] {
            for _ in 0..20 {
                g.add_card_to_library(seat, catalog::plains());
            }
        }
        g.players[0].hand.clear();
        for _ in 0..11 {
            g.add_card_to_hand(0, catalog::grizzly_bears());
        }
        let turn = g.turn_number;
        for _ in 0..80 {
            if g.turn_number != turn {
                break;
            }
            if g.perform_action(GameAction::PassPriority).is_err() {
                break;
            }
            drain_stack(&mut g);
        }
        g.players[0].hand.len()
    };
    assert_eq!(keep(true), 11, "no maximum hand size, so nothing is discarded");
    assert_eq!(keep(false), 7, "and the control discards down to seven");
}

// ── Cards the engine documented and had never shipped ──────────────────────

/// Boon Reflection: "If you would gain life, you gain twice that much life
/// instead." `StaticEffect::LifeGainMultiplier`'s doc named this card and
/// Rhox Faithmender; only the Faithmender was in the catalog.
#[test]
fn cr_614_boon_reflection_doubles_your_life_gain_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::boon_reflection());

    let yours = g.players[0].life;
    g.adjust_life(0, 4);
    assert_eq!(g.players[0].life, yours + 8, "twice that much");

    let theirs = g.players[1].life;
    g.adjust_life(1, 4);
    assert_eq!(g.players[1].life, theirs + 4, "and it is *your* life gain, not the table's");
}

/// Thousand-Year Elixir: "You may activate abilities of creatures you control
/// as though those creatures had haste."
///
/// CR 602.5g bars a summoning-sick creature from paying a `{T}` cost. The
/// Elixir exempts the controller's creatures; the test is that the *same*
/// activation fails before it lands and succeeds after.
#[test]
fn cr_602_5g_thousand_year_elixir_exempts_the_summoning_sickness_gate() {
    let mut g = two_player_game();
    let dancer = g.add_card_to_battlefield(0, catalog::sinew_dancer());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: dancer,
            ability_index: 0,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(act(&mut g).is_err(), "a sick creature can't tap-activate (CR 602.5g)");
    g.add_card_to_battlefield(0, catalog::thousand_year_elixir());
    act(&mut g).expect("the Elixir's static exempts the gate");
}

/// …and its second half is an ordinary targeted untap, which works on anyone's
/// creature — the two clauses are independent.
#[test]
fn thousand_year_elixir_untaps_a_creature_for_one_and_a_tap() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let elixir = g.add_card_to_battlefield(0, catalog::thousand_year_elixir());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: elixir,
        ability_index: 0,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{1}, {T}: Untap target creature");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(theirs).unwrap().tapped, "untapped, and it is not yours");
    assert!(g.battlefield_find(elixir).unwrap().tapped, "the Elixir paid its own tap");
}

// ── COMMANDER_BACKLOG top-1000, 2026-09-20 batch ────────────────────────────

/// "Whenever a creature you control deals combat damage to a player, you **may**
/// draw a card." Printed "a creature", not "one or more", so two connecting
/// creatures are two triggers (CR 603.2c) — the count is what this asserts.
#[test]
fn reconnaissance_mission_draws_once_per_connecting_creature() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::reconnaissance_mission());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    let hand = g.players[0].hand.len();
    g.attacking = vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ];
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 0;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "one card per connecting creature");
}

/// "Whenever a creature you control dies, you gain 1 life and draw a card."
#[test]
fn moldervine_reclamation_drains_and_draws_on_your_creature_dying() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::moldervine_reclamation());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    g.battlefield_find_mut(bear).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// "Whenever a creature you control enters, draw a card if its power is 3 or
/// greater. **Otherwise**, put two +1/+1 counters on it." The branches are
/// exclusive, so the small creature must draw nothing.
#[test]
fn tribute_to_the_world_tree_branches_on_the_entering_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tribute_to_the_world_tree());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();

    // Cast it, so the real ETB event dispatches to the watcher.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let small = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: small, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast the 2/2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "a 2/2 draws nothing");
    assert_eq!(
        g.battlefield_find(small)
            .and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
            .unwrap_or(0),
        2,
        "it gets two +1/+1 counters instead",
    );

    let big = g.add_card_to_hand(0, catalog::craw_wurm());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast the 6/4");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "a 6/4 draws");
    assert_eq!(
        g.battlefield_find(big)
            .and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
            .unwrap_or(0),
        0,
        "and gets no counters",
    );
}

/// "Whenever a Dragon you control enters, it deals X damage to any target,
/// where X is the number of Dragons you control" — the entering Dragon counts
/// itself, so one Dragon on an empty board deals 1.
#[test]
fn dragon_tempest_counts_the_entering_dragon() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dragon_tempest());
    let life = g.players[1].life;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let d = g.add_card_to_hand(0, catalog::shivan_dragon());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: d, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast the Dragon");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "one Dragon, one damage");
    assert!(
        g.battlefield_find(d).is_some_and(|c| c.granted_keywords_eot.contains(&crabomination::card::Keyword::Haste)
            || c.definition.keywords.contains(&crabomination::card::Keyword::Haste)),
        "and the flying half granted haste",
    );
}

/// "Whenever **one or more** artifact creatures you control deal combat damage
/// to a player, draw a card" — CR 603.2c, so two Thopters into one player is
/// one card, not two.
#[test]
fn thopter_spy_network_draws_once_a_batch() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thopter_spy_network());
    let a = g.add_card_to_battlefield(0, catalog::ornithopter());
    let b = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.clear_sickness(a);
    g.clear_sickness(b);
    // Ornithopter is 0/2; give both a body so the damage lands.
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().pump(2, 0);
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    g.attacking = vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ];
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 0;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "one card for the whole batch");
}

/// Witch's Cottage: the conditional entry is a **replacement** and the
/// recursion is a real trigger beside it, gated on the same predicate — and
/// the filter names **your graveyard**, so a creature on the battlefield is
/// not a legal target for it.
#[test]
fn witchs_cottage_enters_tapped_without_three_other_swamps() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witchs_cottage());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::PlayLand(id)).expect("a land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "fewer than three other Swamps");
    assert!(g.players[0].library.is_empty(), "and the trigger's `if` did not fire");
}

#[test]
fn witchs_cottage_with_three_other_swamps_enters_untapped_and_recurs() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let live = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witchs_cottage());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::PlayLand(id)).expect("a land");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(id).unwrap().tapped, "three other Swamps, so untapped");
    assert!(g.battlefield_find(live).is_some(), "the live creature is not a legal target");
    assert_eq!(
        g.players[0].library.last().map(|c| c.id),
        Some(dead),
        "the graveyard creature goes on top",
    );
}

/// "Whenever this creature is dealt damage, it deals that much damage to
/// target opponent" — on an indestructible 1/1, which is the whole card.
#[test]
fn brash_taunter_reflects_damage_at_an_opponent() {
    let mut g = two_player_game();
    let taunter = g.add_card_to_battlefield(0, catalog::brash_taunter());
    let life = g.players[1].life;
    let mut ev = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(taunter), 5, None, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 5, "five in, five at the opponent");
    assert!(g.battlefield_find(taunter).is_some(), "and it is indestructible");
}

/// Void Rend: "This spell can't be countered. Destroy target nonland
/// permanent." The uncounterable half is what makes it a staple.
#[test]
fn void_rend_cannot_be_countered_and_kills_a_nonland() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::void_rend());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(land)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a land is not a legal target",
    );
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("nonland permanent");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "destroyed");
    assert!(
        catalog::void_rend().keywords.contains(&crabomination::card::Keyword::CantBeCountered),
        "and it can't be countered",
    );
}

/// "At the beginning of **each** combat, double the power and toughness of
/// each creature you control" — each creature doubles its *own* P/T, so a
/// 2/2 and a 6/4 are 4/4 and 12/8, not both the same.
#[test]
fn unnatural_growth_doubles_each_creature_by_its_own_power() {
    use crabomination::game::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::unnatural_growth());
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(0, catalog::craw_wurm());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let pt = |id| {
        let c = g.computed_permanent(id).expect("on the battlefield");
        (c.power, c.toughness)
    };
    assert_eq!(pt(small), (4, 4), "the 2/2 doubles to 4/4");
    assert_eq!(pt(big), (12, 8), "and the 6/4 to 12/8");
}

/// "Whenever Ayara **or another** black creature you control enters, each
/// opponent loses 1 life and you gain 1 life." The scope already includes the
/// source, so Ayara's own entry fires it — and the drain is per opponent.
#[test]
fn ayara_drains_on_her_own_entry() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::ayara_first_of_locthwain());
    g.players[0].mana_pool.add(Color::Black, 3);
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Ayara");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, theirs - 1, "each opponent loses 1");
    assert_eq!(g.players[0].life, mine + 1, "and you gain 1");
}

/// Cut a Deal draws one card per opponent, so it is a cantrip in a duel and a
/// three-for-one in a four-seat pod — the count is the card.
#[test]
fn cut_a_deal_draws_one_card_per_opponent() {
    let mut g = crabomination::game::multi_player_game(4);
    for seat in 0..4 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::plains());
        }
    }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let id = g.add_card_to_hand(0, catalog::cut_a_deal());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before: Vec<usize> = (0..4).map(|s| g.players[s].hand.len()).collect();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Cut a Deal");
    drain_stack(&mut g);
    // -1 for the spell itself, +3 for the three opponents who drew.
    assert_eq!(g.players[0].hand.len(), before[0] - 1 + 3, "three opponents, three cards");
    for seat in 1..4 {
        assert_eq!(g.players[seat].hand.len(), before[seat] + 1, "each opponent drew one");
    }
}

/// Padeem draws only while **you** hold the greatest-mana-value artifact, and
/// the printed "or tied for" is what the comparison has to allow.
#[test]
fn padeem_draws_only_while_you_hold_the_biggest_artifact() {
    use crabomination::game::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::padeem_consul_of_innovation());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.active_player_idx = 0;
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "your one-mana Sol Ring is the greatest");

    // An opponent's bigger artifact turns it off.
    g.add_card_to_battlefield(1, catalog::hedron_archive());
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "theirs is bigger, so no draw");
}

/// "Each **noncreature** artifact you control becomes a 4/4 artifact creature
/// until end of turn" — and it keeps the artifact type, so the anthem above it
/// gives the animated Signet flying on the same resolution.
#[test]
fn cyberdrive_awakener_animates_your_noncreature_artifacts() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let awakener = g.add_card_to_battlefield(0, catalog::cyberdrive_awakener());
    g.fire_self_etb_triggers(awakener, 0);
    drain_stack(&mut g);
    let c = g.computed_permanent(ring).expect("on the battlefield");
    assert_eq!((c.power, c.toughness), (4, 4), "the Sol Ring is a 4/4");
    assert!(
        g.computed_permanent(ring).is_some_and(|c| c.card_types().contains(&CardType::Artifact)),
        "and it keeps the artifact type — `BecomeCreature`, not `BecomeCreatureLosingTypes`",
    );
    // The anthem half, on a permanent that was *printed* an artifact creature.
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter());
    assert!(
        g.computed_permanent(thopter)
            .is_some_and(|c| c.keywords().contains(&crabomination::card::Keyword::Flying)),
        "other artifact creatures you control have flying",
    );
}
