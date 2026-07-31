//! Fifth Dawn gap batch 1 (`decks::recent322`).

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Beacon of Creation mints an Insect per Forest and shuffles itself back.
#[test]
fn beacon_of_creation_counts_forests_then_reshuffles() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let beacon = g.add_card_to_hand(0, catalog::beacon_of_creation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: beacon, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Insect").count(), 3);
    assert!(g.players[0].library.iter().any(|c| c.id == beacon), "shuffled back in");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == beacon));
}

/// Beacon of Unrest reanimates out of any graveyard.
#[test]
fn beacon_of_unrest_reanimates_from_any_graveyard() {
    let mut g = main_phase();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let corpse = g.players[1].graveyard[0].id;
    let beacon = g.add_card_to_hand(0, catalog::beacon_of_unrest());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: beacon, target: Some(Target::Permanent(corpse)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(corpse).unwrap().controller, 0);
}

/// Beacon of Tomorrows hands an extra turn to its target.
#[test]
fn beacon_of_tomorrows_grants_an_extra_turn() {
    let mut g = main_phase();
    let beacon = g.add_card_to_hand(0, catalog::beacon_of_tomorrows());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: beacon, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_turns, 1);
}

/// Clock of Omens taps two artifacts to untap a third.
#[test]
fn clock_of_omens_untaps_with_two_helpers() {
    let mut g = main_phase();
    let clock = g.add_card_to_battlefield(0, catalog::clock_of_omens());
    let a = g.add_card_to_battlefield(0, catalog::tanglebloom());
    let b = g.add_card_to_battlefield(0, catalog::tanglebloom());
    let target = g.add_card_to_battlefield(0, catalog::tanglebloom());
    g.battlefield_find_mut(target).unwrap().tapped = true;
    for id in [a, b] {
        g.clear_sickness(id);
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: clock, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(target).unwrap().tapped);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
}

/// Gemstone Array banks generic mana and pays it back in any colour.
#[test]
fn gemstone_array_banks_and_returns_colored_mana() {
    let mut g = main_phase();
    let array = g.add_card_to_battlefield(0, catalog::gemstone_array());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: array, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("bank");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(array).unwrap().counter_count(CounterType::Charge), 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: array, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("spend");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Blind Creeper shrinks on every spell anyone casts.
#[test]
fn blind_creeper_shrinks_on_each_cast() {
    let mut g = main_phase();
    let creeper = g.add_card_to_battlefield(0, catalog::blind_creeper());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(creeper).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Desecration Elemental eats a creature on every cast.
#[test]
fn desecration_elemental_eats_a_creature_per_cast() {
    let mut g = main_phase();
    let elem = g.add_card_to_battlefield(0, catalog::desecration_elemental());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the cheapest creature went");
    assert!(g.battlefield_find(elem).is_some());
}

/// Goblin Brawler refuses every Equipment (CR 702.6c).
#[test]
fn goblin_brawler_cant_be_equipped() {
    let mut g = main_phase();
    let brawler = g.add_card_to_battlefield(0, catalog::goblin_brawler());
    let blade = g.add_card_to_battlefield(0, catalog::banshees_blade());
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::Equip { equipment: blade, target: brawler }).is_err());
}

/// Armed Response scales with the Equipment you control.
#[test]
fn armed_response_counts_equipment() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::banshees_blade());
    g.add_card_to_battlefield(0, catalog::worldslayer());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_hand(0, catalog::armed_response());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(attacker)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "2 Equipment kills a 2/2");
}

/// Devour in Shadow costs you the creature's toughness in life.
#[test]
fn devour_in_shadow_charges_toughness_in_life() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::arachnoid());
    let spell = g.add_card_to_hand(0, catalog::devour_in_shadow());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.players[0].life, 14, "a 2/6 costs six");
}

/// Composite Golem cashes itself in for one of each colour.
#[test]
fn composite_golem_makes_five_colors() {
    let mut g = main_phase();
    let golem = g.add_card_to_battlefield(0, catalog::composite_golem());
    g.perform_action(GameAction::ActivateAbility {
        card_id: golem, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("sac");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 5);
    assert!(g.battlefield_find(golem).is_none());
}

/// Cosmic Larva eats two lands each upkeep, or itself.
#[test]
fn cosmic_larva_demands_lands() {
    let mut g = main_phase();
    let larva = g.add_card_to_battlefield(0, catalog::cosmic_larva());
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(g.battlefield_find(larva).is_none(), "no lands to feed it");
}

/// Fleshgrafter pitches an artifact for +2/+2.
#[test]
fn fleshgrafter_discards_an_artifact_to_pump() {
    let mut g = main_phase();
    let grafter = g.add_card_to_battlefield(0, catalog::fleshgrafter());
    g.add_card_to_hand(0, catalog::tanglebloom());
    g.perform_action(GameAction::ActivateAbility {
        card_id: grafter, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(grafter).unwrap().power, 4);
    assert!(g.players[0].hand.is_empty());
}

/// Dawn's Reflection adds two extra mana whenever the land taps.
#[test]
fn dawns_reflection_triples_the_land() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_hand(0, catalog::dawns_reflection());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(forest)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Blinkmoth Infusion's affinity makes it castable, and it untaps everything.
#[test]
fn blinkmoth_infusion_untaps_all_artifacts() {
    let mut g = main_phase();
    let rock = g.add_card_to_battlefield(0, catalog::tanglebloom());
    g.battlefield_find_mut(rock).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::blinkmoth_infusion());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(11);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("affinity for one artifact");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(rock).unwrap().tapped);
}

/// Ferocious Charge pumps and digs.
#[test]
fn ferocious_charge_pumps_and_scries() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::ferocious_charge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6);
    assert_eq!(g.players[0].library.len(), 3, "scry doesn't draw");
}

/// Ferropede strips a counter when it connects.
#[test]
fn ferropede_strips_a_counter_on_connect() {
    let mut g = main_phase();
    let pede = g.add_card_to_battlefield(0, catalog::ferropede());
    let target = g.add_card_to_battlefield(1, catalog::clockwork_dragon());
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 6);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.clear_sickness(pede);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: pede, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

/// Eyes of the Watcher offers a {1} scry on each instant or sorcery.
#[test]
fn eyes_of_the_watcher_scries_for_one() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::eyes_of_the_watcher());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "the extra mana was paid");
}

/// Baton of Courage enters with sunburst counters and spends them as pumps.
#[test]
fn baton_of_courage_spends_sunburst_counters() {
    let mut g = main_phase();
    let baton = g.add_card_to_hand(0, catalog::baton_of_courage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: baton, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(baton).unwrap().counter_count(CounterType::Charge), 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: baton, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    let _ = (CardType::Artifact, Keyword::Flash);
}

// ── Fifth Dawn gap batches 2–3 (`decks::recent323` / `recent324`) ──

/// Install a decider that says yes to `n` optional triggers.
fn accept_optionals(g: &mut GameState, n: usize) {
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        std::iter::repeat_n(crabomination::decision::DecisionAnswer::Bool(true), n),
    ));
}

/// Cast a card from hand with the given colored + colorless mana floated.
fn cast_with(
    g: &mut GameState,
    def: crabomination::card::CardDefinition,
    colors: &[Color],
    colorless: u32,
) -> crabomination::card::CardId {
    let id = g.add_card_to_hand(0, def);
    for c in colors {
        g.players[0].mana_pool.add(*c, 1);
    }
    g.players[0].mana_pool.add_colorless(colorless);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(g);
    id
}

/// The batch's plain bodies keep their printed stats and keywords.
#[test]
fn fdn5_vanilla_bodies_match_print() {
    for (def, pt, kws) in [
        (catalog::skyhunter_prowler(), (1, 3), vec![Keyword::Flying, Keyword::Vigilance]),
        (catalog::plasma_elemental(), (4, 1), vec![Keyword::Unblockable]),
        (catalog::iron_barb_hellion(), (5, 4), vec![Keyword::Haste, Keyword::CantBlock]),
        (catalog::razorgrass_screen(), (2, 1), vec![Keyword::Defender, Keyword::MustBlock]),
    ] {
        assert_eq!((def.power, def.toughness), pt, "{}", def.name);
        for kw in kws {
            assert!(def.keywords.contains(&kw), "{} missing {kw:?}", def.name);
        }
    }
}

/// Silent Arbiter caps the whole combat at one attacker (CR 506.2).
#[test]
fn silent_arbiter_caps_attackers_at_one() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::silent_arbiter());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [a, b] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ])
        .is_err(),
        "two attackers rejected"
    );
    g.declare_attackers(vec![Attack { attacker: a, target: AttackTarget::Player(1) }])
        .expect("one attacker is fine");
}

/// Silent Arbiter caps blockers the same way (CR 509.1b).
#[test]
fn silent_arbiter_caps_blockers_at_one() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::silent_arbiter());
    let atk_a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let atk_b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blk_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blk_b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [atk_a, atk_b] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    // The Arbiter is the defender's, so the attack cap binds the attacker too.
    g.declare_attackers(vec![Attack { attacker: atk_a, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.declare_blockers(vec![(blk_a, atk_a), (blk_b, atk_a)]).is_err(), "two blockers rejected");
    g.declare_blockers(vec![(blk_a, atk_a)]).expect("one blocker is fine");
    let _ = atk_b;
}

/// Synod Centurion sacrifices itself the moment it's the only artifact.
#[test]
fn synod_centurion_sacrifices_itself_when_alone() {
    let mut g = main_phase();
    let other = g.add_card_to_battlefield(0, catalog::tanglebloom());
    let cent = g.add_card_to_battlefield(0, catalog::synod_centurion());
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(cent).is_some(), "another artifact is present");
    g.battlefield.retain(|c| c.id != other);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(cent).is_none(), "no other artifact left");
}

/// Vedalken Shackles holds a creature only while it stays tapped (CR 611.2c).
#[test]
fn vedalken_shackles_releases_when_it_untaps() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let shackles = g.add_card_to_battlefield(0, catalog::vedalken_shackles());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shackles, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("steal");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    g.battlefield_find_mut(shackles).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1, "the lock released");
}

/// Doubling Cube doubles every floating pip.
#[test]
fn doubling_cube_doubles_the_pool() {
    let mut g = main_phase();
    let cube = g.add_card_to_battlefield(0, catalog::doubling_cube());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cube, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    // Five floated minus the {3} paid leaves two, doubled to four.
    assert_eq!(g.players[0].mana_pool.total(), 4);
}

/// Fist of Suns lets any hand card be cast for WUBRG (CR 118.9).
#[test]
fn fist_of_suns_grants_a_wubrg_alternative_cost() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::fist_of_suns());
    let fatty = g.add_card_to_hand(0, catalog::bringer_of_the_blue_dawn());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fatty, pitch_card: None, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast for WUBRG");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fatty).is_some());
}

/// The Bringers carry the printed WUBRG alternative cost themselves.
#[test]
fn bringer_casts_for_five_colors_and_pays_off_at_upkeep() {
    let mut g = main_phase();
    let bringer = g.add_card_to_hand(0, catalog::bringer_of_the_green_dawn());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: bringer, pitch_card: None, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    accept_optionals(&mut g, 1);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Beast").count(), 1);
}

/// Ion Storm accepts either counter kind as its cost.
#[test]
fn ion_storm_spends_a_charge_or_plus_one_counter() {
    let mut g = main_phase();
    let storm = g.add_card_to_battlefield(0, catalog::ion_storm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: storm, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Relentless Rats counts every other copy on the battlefield.
#[test]
fn relentless_rats_count_each_other() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::relentless_rats());
    g.add_card_to_battlefield(0, catalog::relentless_rats());
    g.add_card_to_battlefield(1, catalog::relentless_rats());
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "two other Rats, either side");
}

/// Retaliate kills everything that got through to you this turn.
#[test]
fn retaliate_destroys_creatures_that_damaged_you() {
    let mut g = main_phase();
    let hitter = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bystander = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 2, Some(hitter), &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let spell = g.add_card_to_hand(0, catalog::retaliate());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hitter).is_none());
    assert!(g.battlefield_find(bystander).is_some());
}

/// Vicious Betrayal pumps by two per creature sacrificed to cast it.
#[test]
fn vicious_betrayal_scales_with_the_sacrifices() {
    let mut g = main_phase();
    let survivor = g.add_card_to_battlefield(0, catalog::plasma_elemental());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::vicious_betrayal());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(survivor)), additional_targets: vec![],
        mode: None, x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(survivor).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 5), "+4/+4 from two sacrifices");
}

/// Myr Servitor pulls every copy out of every graveyard at once.
#[test]
fn myr_servitor_returns_all_copies() {
    let mut g = main_phase();
    let alive = g.add_card_to_battlefield(0, catalog::myr_servitor());
    g.add_card_to_graveyard(0, catalog::myr_servitor());
    g.add_card_to_graveyard(1, catalog::myr_servitor());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Myr Servitor").count(),
        3,
        "both graveyards emptied"
    );
    let _ = alive;
}

/// Roar of Reclamation gives every player their artifacts back.
#[test]
fn roar_of_reclamation_rebuilds_both_boards() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::tanglebloom());
    g.add_card_to_graveyard(1, catalog::tanglebloom());
    let spell = g.add_card_to_hand(0, catalog::roar_of_reclamation());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 1);
}

/// Acquire pulls an artifact out of the opponent's library onto your side.
#[test]
fn acquire_steals_an_artifact_from_the_library() {
    let mut g = main_phase();
    let rock = g.add_card_to_library(1, catalog::tanglebloom());
    let spell = g.add_card_to_hand(0, catalog::acquire());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(rock)),
    ]));
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let stolen = g.battlefield.iter().find(|c| c.definition.name == "Tanglebloom").expect("stolen");
    assert_eq!(stolen.controller, 0);
}

/// Mana Geyser pays out one red per tapped land the opponents control.
#[test]
fn mana_geyser_counts_opposing_tapped_lands() {
    let mut g = main_phase();
    for _ in 0..3 {
        let l = g.add_card_to_battlefield(1, catalog::mountain());
        g.battlefield_find_mut(l).unwrap().tapped = true;
    }
    g.add_card_to_battlefield(1, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::mana_geyser());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3, "untapped land doesn't count");
}

/// Granulate sweeps only the cheap artifacts.
#[test]
fn granulate_spares_expensive_artifacts() {
    let mut g = main_phase();
    let cheap = g.add_card_to_battlefield(1, catalog::tanglebloom());
    let dear = g.add_card_to_battlefield(1, catalog::summoning_station());
    let spell = g.add_card_to_hand(0, catalog::granulate());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cheap).is_none());
    assert!(g.battlefield_find(dear).is_some(), "{{7}} survives");
}

/// Vanquish only aims at a creature that's actually blocking.
#[test]
fn vanquish_needs_a_blocking_creature() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.declare_blockers(vec![(blocker, attacker)]).expect("block");
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::vanquish());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(attacker)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .is_err(),
        "the attacker isn't blocking"
    );
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(blocker)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).is_none());
}

/// Mephidross Vampire makes your team Vampires that grow on damage.
#[test]
fn mephidross_vampire_types_and_grows_the_team() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::mephidross_vampire());
    let victim = g.add_card_to_battlefield(1, catalog::skyhunter_prowler());
    assert!(g
        .computed_permanent(bear)
        .unwrap()
        .subtypes
        .creature_types
        .contains(&crabomination::card::CreatureType::Vampire));
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.declare_blockers(vec![(victim, bear)]).expect("block");
    let _ = g.resolve_combat();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Raksha Golden Cub only pumps the Cats while it's carrying Equipment.
#[test]
fn raksha_golden_cub_needs_equipment() {
    let mut g = main_phase();
    let raksha = g.add_card_to_battlefield(0, catalog::raksha_golden_cub());
    assert_eq!(g.computed_permanent(raksha).unwrap().power, 3, "unequipped");
    let blade = g.add_card_to_battlefield(0, catalog::banshees_blade());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: blade, target: raksha }).expect("equip");
    let cp = g.computed_permanent(raksha).unwrap();
    assert_eq!(cp.power, 5, "+2/+2 from its own static");
    assert!(cp.keywords.contains(&Keyword::DoubleStrike));
}

/// Battered Golem untaps whenever any artifact enters.
#[test]
fn battered_golem_untaps_on_an_artifact_etb() {
    let mut g = main_phase();
    let golem = g.add_card_to_battlefield(0, catalog::battered_golem());
    g.battlefield_find_mut(golem).unwrap().tapped = true;
    accept_optionals(&mut g, 1);
    let bloom = g.add_card_to_hand(0, catalog::tanglebloom());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bloom, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(golem).unwrap().tapped);
}

/// Hoverguard Sweepers bounces up to two creatures on the way in.
#[test]
fn hoverguard_sweepers_bounces_two() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sweeper = g.add_card_to_hand(0, catalog::hoverguard_sweepers());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: sweeper, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Heliophial's sunburst counters set the size of its burn.
#[test]
fn heliophial_burns_for_its_charge_counters() {
    let mut g = main_phase();
    let phial = cast_with(&mut g, catalog::heliophial(), &[Color::White, Color::Red], 3);
    assert_eq!(g.battlefield_find(phial).unwrap().counter_count(CounterType::Charge), 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: phial, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("fire");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Infused Arrows shrinks by however many counters it spends.
#[test]
fn infused_arrows_shrinks_by_x() {
    let mut g = main_phase();
    let arrows = cast_with(&mut g, catalog::infused_arrows(), &[Color::Blue, Color::Black], 2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: arrows, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: Some(2), mode: None,
    })
    .expect("fire");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-2/-2 kills a 2/2");
}

/// Opaline Bracers pumps by its own sunburst counters.
#[test]
fn opaline_bracers_scale_with_charge_counters() {
    let mut g = main_phase();
    let bracers =
        cast_with(&mut g, catalog::opaline_bracers(), &[Color::White, Color::Blue, Color::Green], 1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: bracers, target: bear }).expect("equip");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "2 base + 3 counters");
}

/// The 5DN Equipment can attach for their colored cost without the equip step.
#[test]
fn sparring_collar_attaches_for_its_colored_cost() {
    let mut g = main_phase();
    let collar = g.add_card_to_battlefield(0, catalog::sparring_collar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: collar, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("attach");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(collar).unwrap().attached_to, Some(bear));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Chimeric Coils animates itself at whatever X you paid.
#[test]
fn chimeric_coils_becomes_an_x_x() {
    let mut g = main_phase();
    let coils = g.add_card_to_battlefield(0, catalog::chimeric_coils());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: coils, ability_index: 0, target: None, additional_targets: vec![], x_value: Some(3), mode: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(coils).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.card_types.contains(&CardType::Creature));
}

/// Skullcage spares an opponent holding exactly three or four cards.
#[test]
fn skullcage_only_burns_off_the_window() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::skullcage());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "three cards is safe");
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "five cards takes two");
}

/// Salvaging Station untaps on every death.
#[test]
fn salvaging_station_untaps_when_a_creature_dies() {
    let mut g = main_phase();
    let station = g.add_card_to_battlefield(0, catalog::salvaging_station());
    g.battlefield_find_mut(station).unwrap().tapped = true;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    accept_optionals(&mut g, 1);
    let mut evs = Vec::new();
    g.destroy_permanent(bear, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(station).unwrap().tapped);
}

/// Avarice Totem trades itself for anything nonland.
#[test]
fn avarice_totem_swaps_control() {
    let mut g = main_phase();
    let totem = g.add_card_to_battlefield(0, catalog::avarice_totem());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: totem, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("swap");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(totem).unwrap().controller, 1);
}

/// Rite of Passage grows anything of yours that survives damage.
#[test]
fn rite_of_passage_counters_damaged_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::rite_of_passage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(bear), 1, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Stasis Cocoon shuts the enchanted artifact's abilities off.
#[test]
fn stasis_cocoon_locks_the_artifact() {
    let mut g = main_phase();
    let station = g.add_card_to_battlefield(0, catalog::summoning_station());
    let cocoon = g.add_card_to_hand(0, catalog::stasis_cocoon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: cocoon, target: Some(Target::Permanent(station)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g
        .computed_permanent(station)
        .unwrap()
        .keywords
        .contains(&Keyword::CantActivateAbilities));
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: station, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None, mode: None,
        })
        .is_err(),
        "the Pincher ability is locked off"
    );
}

/// Sylvok Explorer taps for a color the other side's lands could make.
#[test]
fn sylvok_explorer_reads_the_opposing_lands() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::island());
    let explorer = g.add_card_to_battlefield(0, catalog::sylvok_explorer());
    g.clear_sickness(explorer);
    g.perform_action(GameAction::ActivateAbility {
        card_id: explorer, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Joiner Adept turns every land you control into an any-color source.
#[test]
fn joiner_adept_fixes_your_lands() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::joiner_adept());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(forest);
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 1, target: None, additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("the granted any-color ability");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Tornado Elemental clears the skies on the way in.
#[test]
fn tornado_elemental_sweeps_fliers() {
    let mut g = main_phase();
    let flier = g.add_card_to_battlefield(1, catalog::skyhunter_prowler());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elem = g.add_card_to_hand(0, catalog::tornado_elemental());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: elem, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flier).is_none());
    assert!(g.battlefield_find(ground).is_some());
}

/// Razormane Masticore eats itself when you can't pay the discard.
#[test]
fn razormane_masticore_sacrifices_on_an_empty_hand() {
    let mut g = main_phase();
    let core = g.add_card_to_battlefield(0, catalog::razormane_masticore());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(core).is_none());
}

/// Mycosynth Golem hands affinity for artifacts to your artifact creatures.
#[test]
fn mycosynth_golem_grants_affinity() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mycosynth_golem());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::tanglebloom());
    }
    let synod = g.add_card_to_hand(0, catalog::synod_centurion());
    // {4} minus affinity for 4 artifacts (the Golem + three Tanglebloom) = {0}.
    g.perform_action(GameAction::CastSpell {
        card_id: synod, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast for free");
    drain_stack(&mut g);
    assert!(g.battlefield_find(synod).is_some());
}

/// Vedalken Orrery gives everything you cast flash.
#[test]
fn vedalken_orrery_grants_flash() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::vedalken_orrery());
    g.step = TurnStep::DeclareBlockers;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast at instant speed");
}

/// Door to Nothingness ends a player outright.
#[test]
fn door_to_nothingness_kills_a_player() {
    let mut g = main_phase();
    let door = g.add_card_to_battlefield(0, catalog::door_to_nothingness());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 2);
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: door, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(!g.players[1].is_alive());
}

/// Nim Grotesque grows with each artifact you control.
#[test]
fn nim_grotesque_counts_your_artifacts() {
    let mut g = main_phase();
    let nim = g.add_card_to_battlefield(0, catalog::nim_grotesque());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::tanglebloom());
    }
    let cp = g.computed_permanent(nim).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 6));
}

// ── The Fifth Dawn remainder ──

/// All Suns' Dawn returns one card per colour and exiles itself.
#[test]
fn all_suns_dawn_returns_one_per_colour_then_exiles() {
    let mut g = main_phase();
    for def in [
        catalog::skyhunter_prowler(),
        catalog::plasma_elemental(),
        catalog::nim_grotesque(),
        catalog::lightning_bolt(),
        catalog::grizzly_bears(),
    ] {
        g.add_card_to_graveyard(0, def);
    }
    let ids: Vec<_> = g.players[0].graveyard.iter().map(|c| c.id).collect();
    let spell = g.add_card_to_hand(0, catalog::all_suns_dawn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(ids[0])),
        additional_targets: ids[1..].iter().map(|i| Target::Permanent(*i)).collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in &ids {
        assert!(g.players[0].hand.iter().any(|c| c.id == *id), "one card of each colour");
    }
    assert!(g.exile.iter().any(|c| c.id == spell), "exiles itself");
}

/// Endless Whispers hands every dead creature to an opponent at the end step.
#[test]
fn endless_whispers_reanimates_under_an_opponent() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::endless_whispers());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.destroy_permanent(bear, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(1));
}

/// Fold into Aether counters a spell and offers its caster a free creature.
#[test]
fn fold_into_aether_gives_the_caster_a_creature() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let freebie = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("bolt");
    g.priority.player_with_priority = 0;
    let fold = g.add_card_to_hand(0, catalog::fold_into_aether());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fold, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("fold");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the bolt was countered");
    assert_eq!(g.battlefield_find(freebie).map(|c| c.controller), Some(1));
}

/// Ouphe Vandals counters an artifact's ability and kills the artifact.
#[test]
fn ouphe_vandals_counters_and_destroys() {
    let mut g = main_phase();
    let ouphe = g.add_card_to_battlefield(0, catalog::ouphe_vandals());
    let station = g.add_card_to_battlefield(1, catalog::summoning_station());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: station, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("mint a Pincher");
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ouphe, ability_index: 0, target: Some(Target::Permanent(station)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("counter it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(station).is_none(), "the source died too");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Pincher").count(), 0);
}

/// Possessed Portal turns off every draw.
#[test]
fn possessed_portal_skips_draws() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::possessed_portal());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    let mut evs = Vec::new();
    assert!(!g.draw_one(0, &mut evs), "the draw is skipped");
    assert_eq!(g.players[0].hand.len(), before);
}

/// Possessed Portal's end step costs each player a card or a permanent.
#[test]
fn possessed_portal_taxes_each_end_step() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::possessed_portal());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "no card to pitch, so it sacrificed");
}

/// Reversal of Fortune copies an instant out of their hand and casts it free.
#[test]
fn reversal_of_fortune_casts_their_spell() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::reversal_of_fortune());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "the original stays in hand");
    assert_eq!(g.players[1].life, 17, "the free copy resolved");
}

/// Spectral Shift rewrites a basic land type on a permanent (CR 612).
#[test]
fn spectral_shift_rewrites_a_land_type() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let shift = g.add_card_to_hand(0, catalog::spectral_shift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Color(Color::Green), // replace "Forest"...
        crabomination::decision::DecisionAnswer::Color(Color::Blue),  // ...with "Island"
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: shift, target: Some(Target::Permanent(forest)), additional_targets: vec![],
        mode: Some(0), x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let types = g.computed_permanent(forest).unwrap().subtypes.land_types;
    assert!(!types.contains(&crabomination::card::LandType::Forest), "the type was replaced");
}

/// Summoner's Egg hatches the creature it imprinted.
#[test]
fn summoners_egg_hatches_its_imprint() {
    let mut g = main_phase();
    let hidden = g.add_card_to_hand(0, catalog::grizzly_bears());
    let egg = g.add_card_to_hand(0, catalog::summoners_egg());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: egg, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == hidden), "imprinted face down");
    let mut evs = Vec::new();
    g.destroy_permanent(egg, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hidden).map(|c| c.controller), Some(0));
}
