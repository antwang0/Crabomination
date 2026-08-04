//! Antiquities (ATQ) — the artifact war (`catalog::sets::atq`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    activate_n(g, seat, id, 0, target)
}

fn activate_n(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn end_turn(g: &mut GameState) {
    let started = g.turn_number;
    while g.turn_number == started {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// A vanilla artifact that pings its controller's opponent for 3.
fn cannon() -> crabomination::card::CardDefinition {
    use crabomination::card::{ActivatedAbility, CardDefinition, CardType};
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    CardDefinition {
        name: "Test Cannon",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Artifacts ──────────────────────────────────────────────────────────────

#[test]
fn amulet_of_kroog_soaks_one_point() {
    let mut g = main_phase();
    let amulet = g.add_card_to_battlefield(0, catalog::amulet_of_kroog());
    activate(&mut g, 0, amulet, Some(Target::Player(0)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 20 - 2, "one of the three points was prevented");
}

#[test]
fn armageddon_clock_ticks_up_and_burns_everyone() {
    let mut g = main_phase();
    let clock = g.add_card_to_battlefield(0, catalog::armageddon_clock());
    // Two of seat 0's upkeeps → two doom counters, and two draw-step volleys.
    for _ in 0..4 {
        end_turn(&mut g);
    }
    while g.step != TurnStep::PreCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(clock).expect("clock").counter_count(CounterType::Doom), 2);
    // 1 damage off the first draw step, 2 off the second.
    assert_eq!(g.players[0].life, 20 - 3);
    assert_eq!(g.players[1].life, 20 - 3);
}

#[test]
fn armageddon_clock_can_be_wound_back_by_the_opponent() {
    let mut g = main_phase();
    let clock = g.add_card_to_battlefield(0, catalog::armageddon_clock());
    if let Some(c) = g.battlefield_find_mut(clock) {
        c.add_counters(CounterType::Doom, 3);
    }
    g.step = TurnStep::Upkeep;
    activate(&mut g, 1, clock, None);
    assert_eq!(g.battlefield_find(clock).expect("clock").counter_count(CounterType::Doom), 2);
}

#[test]
fn ashnods_battle_gear_holds_the_pump_while_tapped() {
    let mut g = main_phase();
    let gear = g.add_card_to_battlefield(0, catalog::ashnods_battle_gear());
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_stone()); // 0/8
    activate(&mut g, 0, gear, Some(Target::Permanent(wall)));
    let cp = g.computed_permanent(wall).expect("wall");
    assert_eq!((cp.power, cp.toughness), (2, 6));
    if let Some(c) = g.battlefield_find_mut(gear) {
        c.tapped = false;
    }
    g.check_state_based_actions();
    let cp = g.computed_permanent(wall).expect("wall");
    assert_eq!((cp.power, cp.toughness), (0, 8), "untapping ends the effect");
}

#[test]
fn ashnods_transmogrant_makes_the_creature_an_artifact() {
    let mut g = main_phase();
    let tm = g.add_card_to_battlefield(0, catalog::ashnods_transmogrant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, tm, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).expect("bear");
    assert_eq!(cp.power, 3);
    assert!(cp.card_types.contains(&crabomination::card::CardType::Artifact));
}

#[test]
fn cursed_rack_caps_the_chosen_opponents_hand() {
    let mut g = main_phase();
    let rack = g.add_card_to_hand(0, catalog::cursed_rack());
    cast(&mut g, 0, rack, None);
    assert_eq!(g.effective_max_hand_size(1), Some(4));
    assert_eq!(g.effective_max_hand_size(0), Some(7), "your own hand is untouched");
}

#[test]
fn the_rack_burns_the_chosen_opponent_for_the_cards_they_lack() {
    let mut g = main_phase();
    let rack = g.add_card_to_hand(0, catalog::the_rack());
    cast(&mut g, 0, rack, None);
    g.players[1].hand.clear();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    end_turn(&mut g); // seat 1's upkeep
    assert_eq!(g.players[1].life, 20 - 2, "3 minus one card in hand");
}

#[test]
fn mightstone_and_weakstone_cancel_out_on_attackers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mightstone());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 3);
    g.add_card_to_battlefield(1, catalog::weakstone());
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 2);
}

#[test]
fn mishras_workshop_mana_only_pays_for_artifacts() {
    let mut g = main_phase();
    let shop = g.add_card_to_battlefield(0, catalog::mishras_workshop());
    let chalice = g.add_card_to_hand(0, catalog::urzas_chalice());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: shop,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap the Workshop");
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "workshop mana can't pay for a creature"
    );
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: chalice,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("artifacts are fine");
}

#[test]
fn jalum_tome_rummages() {
    let mut g = main_phase();
    let tome = g.add_card_to_battlefield(0, catalog::jalum_tome());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, tome, None);
    assert_eq!(g.players[0].hand.len(), hand);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

#[test]
fn urzas_miter_draws_off_a_dying_artifact() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::urzas_miter());
    let scrap = g.add_card_to_battlefield(0, catalog::urzas_chalice());
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::naturalize());
    mana(&mut g, 0);
    g.active_player_idx = 1;
    cast(&mut g, 1, bolt, Some(Target::Permanent(scrap)));
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

#[test]
fn tablet_of_epityr_gains_life_off_a_dying_artifact() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::tablet_of_epityr());
    let scrap = g.add_card_to_battlefield(0, catalog::urzas_chalice());
    let nat = g.add_card_to_hand(1, catalog::naturalize());
    mana(&mut g, 0);
    g.active_player_idx = 1;
    cast(&mut g, 1, nat, Some(Target::Permanent(scrap)));
    assert_eq!(g.players[0].life, 21);
}

#[test]
fn obelisk_of_undoing_only_rewinds_what_you_own() {
    let mut g = main_phase();
    let obelisk = g.add_card_to_battlefield(0, catalog::obelisk_of_undoing());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: obelisk,
            ability_index: 0,
            target: Some(Target::Permanent(theirs)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err()
    );
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, obelisk, Some(Target::Permanent(mine)));
    assert!(g.players[0].hand.iter().any(|c| c.id == mine));
}

#[test]
fn rocket_launcher_waits_a_turn_then_blows_itself_up() {
    let mut g = main_phase();
    let launcher = g.add_card_to_battlefield(0, catalog::rocket_launcher());
    let turn = g.turn_number;
    if let Some(c) = g.battlefield_find_mut(launcher) {
        c.entered_turn = Some(turn);
    }
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: launcher,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "summoning-sick the turn it lands"
    );
    for _ in 0..2 {
        end_turn(&mut g);
    }
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, 0, launcher, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19);
    end_turn(&mut g);
    assert!(g.battlefield_find(launcher).is_none(), "it destroys itself at end of turn");
}

// ── Creatures ──────────────────────────────────────────────────────────────

#[test]
fn argothian_treefolk_shrugs_off_artifact_damage() {
    let mut g = main_phase();
    let treefolk = g.add_card_to_battlefield(1, catalog::argothian_treefolk());
    let catapult = g.add_card_to_battlefield(0, catalog::grapeshot_catapult());
    g.clear_sickness(catapult);
    // Non-artifact damage still lands.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(treefolk)));
    assert_eq!(g.battlefield_find(treefolk).expect("alive").damage, 3);
    // The Catapult's ping is prevented outright.
    let flier = g.add_card_to_battlefield(1, catalog::argothian_treefolk());
    if let Some(c) = g.battlefield_find_mut(flier) {
        c.granted_keywords_eot.push(Keyword::Flying);
    }
    activate(&mut g, 0, catapult, Some(Target::Permanent(flier)));
    assert_eq!(g.battlefield_find(flier).expect("alive").damage, 0);
}

#[test]
fn argothian_pixies_dodge_artifact_blockers() {
    let g = main_phase();
    let pixies = catalog::argothian_pixies();
    assert!(pixies.keywords.iter().any(|k| matches!(k, Keyword::CantBeBlockedBy(_))));
    drop(g);
}

#[test]
fn martyrs_of_korlis_eats_the_artifact_damage() {
    let mut g = main_phase();
    let martyr = g.add_card_to_battlefield(0, catalog::martyrs_of_korlis());
    let gun = g.add_card_to_battlefield(1, cannon());
    g.clear_sickness(gun);
    activate(&mut g, 1, gun, None);
    assert_eq!(g.players[0].life, 20, "the Martyr took it");
    assert_eq!(g.battlefield_find(martyr).expect("alive").damage, 3);
}

#[test]
fn martyrs_of_korlis_stops_shielding_once_tapped() {
    let mut g = main_phase();
    let martyr = g.add_card_to_battlefield(0, catalog::martyrs_of_korlis());
    if let Some(c) = g.battlefield_find_mut(martyr) {
        c.tapped = true;
    }
    let gun = g.add_card_to_battlefield(1, cannon());
    g.clear_sickness(gun);
    activate(&mut g, 1, gun, None);
    assert_eq!(g.players[0].life, 17);
}

#[test]
fn reverse_polarity_pays_back_double_the_artifact_damage() {
    let mut g = main_phase();
    let gun = g.add_card_to_battlefield(1, cannon());
    g.clear_sickness(gun);
    activate(&mut g, 1, gun, None);
    assert_eq!(g.players[0].life, 17);
    let rp = g.add_card_to_hand(0, catalog::reverse_polarity());
    cast(&mut g, 0, rp, None);
    assert_eq!(g.players[0].life, 17 + 6);
}

#[test]
fn gaeas_avenger_counts_the_other_sides_artifacts() {
    let mut g = main_phase();
    let avenger = g.add_card_to_battlefield(0, catalog::gaeas_avenger());
    assert_eq!(g.computed_permanent(avenger).expect("avenger").power, 1);
    g.add_card_to_battlefield(1, catalog::urzas_chalice());
    g.add_card_to_battlefield(1, catalog::urzas_chalice());
    g.add_card_to_battlefield(0, catalog::urzas_chalice());
    let cp = g.computed_permanent(avenger).expect("avenger");
    assert_eq!((cp.power, cp.toughness), (3, 3), "yours don't count");
}

#[test]
fn citanul_druid_grows_off_opposing_artifact_spells() {
    let mut g = main_phase();
    let druid = g.add_card_to_battlefield(0, catalog::citanul_druid());
    let chalice = g.add_card_to_hand(1, catalog::urzas_chalice());
    g.active_player_idx = 1;
    cast(&mut g, 1, chalice, None);
    assert_eq!(
        g.battlefield_find(druid).expect("druid").counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

#[test]
fn priest_of_yawgmoth_converts_scrap_to_black_mana() {
    let mut g = main_phase();
    let priest = g.add_card_to_battlefield(0, catalog::priest_of_yawgmoth());
    g.clear_sickness(priest);
    g.add_card_to_battlefield(0, catalog::jalum_tome()); // mana value 3
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: priest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3);
}

#[test]
fn sage_of_lat_nam_trades_an_artifact_for_a_card() {
    let mut g = main_phase();
    let sage = g.add_card_to_battlefield(0, catalog::sage_of_lat_nam());
    g.clear_sickness(sage);
    g.add_card_to_battlefield(0, catalog::urzas_chalice());
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, sage, None);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

#[test]
fn phyrexian_gremlins_pin_an_artifact_down() {
    let mut g = main_phase();
    let gremlins = g.add_card_to_battlefield(0, catalog::phyrexian_gremlins());
    g.clear_sickness(gremlins);
    let tome = g.add_card_to_battlefield(1, catalog::jalum_tome());
    activate(&mut g, 0, gremlins, Some(Target::Permanent(tome)));
    assert!(g.battlefield_find(tome).expect("tome").tapped);
    assert!(g.untap_prevented_by_static(tome), "held down while the Gremlins stay tapped");
    if let Some(c) = g.battlefield_find_mut(gremlins) {
        c.tapped = false;
    }
    assert!(!g.untap_prevented_by_static(tome));
}

#[test]
fn yawgmoth_demon_bites_when_it_isnt_fed() {
    let mut g = main_phase();
    let demon = g.add_card_to_battlefield(0, catalog::yawgmoth_demon());
    for _ in 0..2 {
        end_turn(&mut g);
    }
    assert_eq!(g.players[0].life, 18);
    assert!(g.battlefield_find(demon).expect("demon").tapped);
}

// ── Enchantments ───────────────────────────────────────────────────────────

#[test]
fn damping_field_lets_only_one_artifact_untap() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::damping_field());
    let a = g.add_card_to_battlefield(0, catalog::urzas_chalice());
    let b = g.add_card_to_battlefield(0, catalog::jalum_tome());
    for id in [a, b] {
        if let Some(c) = g.battlefield_find_mut(id) {
            c.tapped = true;
        }
    }
    for _ in 0..2 {
        end_turn(&mut g);
    }
    let untapped = [a, b].iter().filter(|id| !g.battlefield_find(**id).expect("live").tapped).count();
    assert_eq!(untapped, 1);
}

#[test]
fn haunting_wind_bites_on_a_tap_but_not_twice() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::haunting_wind());
    let tome = g.add_card_to_battlefield(1, catalog::jalum_tome());
    g.clear_sickness(tome);
    activate(&mut g, 1, tome, None);
    assert_eq!(g.players[1].life, 19, "a tap activation counts once, via the tap");
}

#[test]
fn haunting_wind_bites_on_a_tapless_activation() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::haunting_wind());
    let gun = g.add_card_to_battlefield(1, cannon());
    g.clear_sickness(gun);
    activate(&mut g, 1, gun, None);
    assert_eq!(g.players[1].life, 19, "the Wind bites the gun's controller");
    assert_eq!(g.players[0].life, 17, "and the gun fires at them");
}

#[test]
fn powerleech_only_feeds_off_opposing_artifacts() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::powerleech());
    let mine = g.add_card_to_battlefield(0, catalog::jalum_tome());
    g.clear_sickness(mine);
    activate(&mut g, 0, mine, None);
    assert_eq!(g.players[0].life, 20, "your own artifacts don't pay");
    let theirs = g.add_card_to_battlefield(1, catalog::jalum_tome());
    g.clear_sickness(theirs);
    activate(&mut g, 1, theirs, None);
    assert_eq!(g.players[0].life, 21);
}

#[test]
fn gate_to_phyrexia_eats_a_creature_for_an_artifact() {
    let mut g = main_phase();
    let gate = g.add_card_to_battlefield(0, catalog::gate_to_phyrexia());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tome = g.add_card_to_battlefield(1, catalog::jalum_tome());
    g.step = TurnStep::Upkeep;
    activate(&mut g, 0, gate, Some(Target::Permanent(tome)));
    assert!(g.battlefield_find(tome).is_none());
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

#[test]
fn artifact_ward_hides_the_creature_from_machines() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ward = g.add_card_to_hand(1, catalog::artifact_ward());
    g.active_player_idx = 1;
    cast(&mut g, 1, ward, Some(Target::Permanent(bear)));
    g.active_player_idx = 0;
    let catapult = g.add_card_to_battlefield(0, catalog::grapeshot_catapult());
    g.clear_sickness(catapult);
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.granted_keywords_eot.push(Keyword::Flying);
    }
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: catapult,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "artifact abilities can't target it"
    );
}

#[test]
fn artifact_possession_burns_the_artifacts_controller() {
    let mut g = main_phase();
    let tome = g.add_card_to_battlefield(1, catalog::jalum_tome());
    g.clear_sickness(tome);
    let aura = g.add_card_to_hand(0, catalog::artifact_possession());
    cast(&mut g, 0, aura, Some(Target::Permanent(tome)));
    activate(&mut g, 1, tome, None);
    assert_eq!(g.players[1].life, 18);
}

#[test]
fn circle_of_protection_artifacts_blanks_the_chosen_gun() {
    let mut g = main_phase();
    let cop = g.add_card_to_battlefield(0, catalog::circle_of_protection_artifacts());
    let gun = g.add_card_to_battlefield(1, cannon());
    g.clear_sickness(gun);
    activate(&mut g, 0, cop, None);
    activate(&mut g, 1, gun, None);
    assert_eq!(g.players[0].life, 20);
}

// ── Spells ─────────────────────────────────────────────────────────────────

#[test]
fn artifact_blast_only_counters_artifact_spells() {
    let mut g = main_phase();
    let blast = g.add_card_to_hand(0, catalog::artifact_blast());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bear");
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: blast,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
}

#[test]
fn crumble_pays_the_owner_for_their_artifact() {
    let mut g = main_phase();
    let tome = g.add_card_to_battlefield(1, catalog::jalum_tome()); // mana value 3
    let crumble = g.add_card_to_hand(0, catalog::crumble());
    cast(&mut g, 0, crumble, Some(Target::Permanent(tome)));
    assert!(g.battlefield_find(tome).is_none());
    assert_eq!(g.players[1].life, 23);
}

#[test]
fn detonate_matches_x_to_the_artifacts_mana_value() {
    let mut g = main_phase();
    let tome = g.add_card_to_battlefield(1, catalog::jalum_tome()); // mana value 3
    let det = g.add_card_to_hand(0, catalog::detonate());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: det,
            target: Some(Target::Permanent(tome)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(2),
        })
        .is_err(),
        "X must equal the artifact's mana value"
    );
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: det,
        target: Some(Target::Permanent(tome)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("X = 3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tome).is_none());
    assert_eq!(g.players[1].life, 17);
}

#[test]
fn reconstruction_pulls_an_artifact_out_of_your_graveyard() {
    let mut g = main_phase();
    let scrap = g.add_card_to_graveyard(0, catalog::jalum_tome());
    let recon = g.add_card_to_hand(0, catalog::reconstruction());
    cast(&mut g, 0, recon, Some(Target::Permanent(scrap)));
    assert!(g.players[0].hand.iter().any(|c| c.id == scrap));
}

#[test]
fn drafnas_restoration_stacks_artifacts_back_on_the_library() {
    let mut g = main_phase();
    let a = g.add_card_to_graveyard(1, catalog::jalum_tome());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let dr = g.add_card_to_hand(0, catalog::drafnas_restoration());
    cast(&mut g, 0, dr, Some(Target::Player(1)));
    assert!(g.players[1].library.iter().any(|c| c.id == a));
    assert_eq!(g.players[1].graveyard.len(), 1, "the creature stays put");
}

#[test]
fn clay_statue_regenerates() {
    let mut g = main_phase();
    let statue = g.add_card_to_battlefield(0, catalog::clay_statue());
    activate(&mut g, 0, statue, None);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(statue)));
    assert!(g.battlefield_find(statue).is_some(), "the shield ate the lethal damage");
}

#[test]
fn colossus_of_sardia_stays_down_until_you_pay() {
    let mut g = main_phase();
    let colossus = g.add_card_to_battlefield(0, catalog::colossus_of_sardia());
    if let Some(c) = g.battlefield_find_mut(colossus) {
        c.tapped = true;
    }
    for _ in 0..2 {
        end_turn(&mut g);
    }
    assert!(g.battlefield_find(colossus).expect("live").tapped);
    g.step = TurnStep::Upkeep;
    activate(&mut g, 0, colossus, None);
    assert!(!g.battlefield_find(colossus).expect("live").tapped);
}

#[test]
fn battering_ram_bands_at_combat_and_knocks_down_its_wall() {
    let mut g = main_phase();
    let ram = g.add_card_to_battlefield(0, catalog::battering_ram());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    g.clear_sickness(ram);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(ram).expect("ram").keywords.contains(&Keyword::Banding),
        "banding at the beginning of combat"
    );
    g.declare_attackers(vec![Attack { attacker: ram, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, ram)])).expect("block");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(wall).is_none(), "the Wall comes down at end of combat");
}

#[test]
fn dwarven_weaponsmith_only_works_at_upkeep() {
    let mut g = main_phase();
    let smith = g.add_card_to_battlefield(0, catalog::dwarven_weaponsmith());
    g.clear_sickness(smith);
    g.add_card_to_battlefield(0, catalog::urzas_chalice());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: smith,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err()
    );
    g.step = TurnStep::Upkeep;
    activate(&mut g, 0, smith, Some(Target::Permanent(bear)));
    assert_eq!(
        g.battlefield_find(bear).expect("bear").counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

#[test]
fn orcish_mechanics_burns_for_an_artifact() {
    let mut g = main_phase();
    let orcs = g.add_card_to_battlefield(0, catalog::orcish_mechanics());
    g.clear_sickness(orcs);
    g.add_card_to_battlefield(0, catalog::urzas_chalice());
    activate(&mut g, 0, orcs, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
}

#[test]
fn argivian_archaeologist_recurs_an_artifact() {
    let mut g = main_phase();
    let arch = g.add_card_to_battlefield(0, catalog::argivian_archaeologist());
    g.clear_sickness(arch);
    let scrap = g.add_card_to_graveyard(0, catalog::jalum_tome());
    activate(&mut g, 0, arch, Some(Target::Permanent(scrap)));
    assert!(g.players[0].hand.iter().any(|c| c.id == scrap));
}

#[test]
fn argivian_blacksmith_patches_a_machine() {
    let mut g = main_phase();
    let smith = g.add_card_to_battlefield(0, catalog::argivian_blacksmith());
    g.clear_sickness(smith);
    let engine = g.add_card_to_battlefield(0, catalog::dragon_engine());
    activate(&mut g, 0, smith, Some(Target::Permanent(engine)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(engine)));
    assert_eq!(g.battlefield_find(engine).expect("alive").damage, 1, "2 of the 3 prevented");
}

#[test]
fn staff_of_zegon_blunts_an_attacker() {
    let mut g = main_phase();
    let staff = g.add_card_to_battlefield(0, catalog::staff_of_zegon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, staff, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 0);
}

#[test]
fn tawnoss_weaponry_holds_its_pump_while_tapped() {
    let mut g = main_phase();
    let weap = g.add_card_to_battlefield(0, catalog::tawnoss_weaponry());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, weap, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 3);
    if let Some(c) = g.battlefield_find_mut(weap) {
        c.tapped = false;
    }
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 2);
}

#[test]
fn tawnoss_wand_only_helps_small_creatures() {
    let mut g = main_phase();
    let wand = g.add_card_to_battlefield(0, catalog::tawnoss_wand());
    let big = g.add_card_to_battlefield(0, catalog::colossus_of_sardia()); // 9/9
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: wand,
            ability_index: 0,
            target: Some(Target::Permanent(big)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err()
    );
    activate(&mut g, 0, wand, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).expect("bear").keywords.contains(&Keyword::Unblockable));
}

#[test]
fn onulet_pays_back_when_it_breaks() {
    let mut g = main_phase();
    let onulet = g.add_card_to_battlefield(0, catalog::onulet());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(onulet)));
    assert_eq!(g.players[0].life, 22);
}

#[test]
fn dragon_engine_pumps_itself() {
    let mut g = main_phase();
    let engine = g.add_card_to_battlefield(0, catalog::dragon_engine());
    activate(&mut g, 0, engine, None);
    activate(&mut g, 0, engine, None);
    assert_eq!(g.computed_permanent(engine).expect("engine").power, 3);
}

#[test]
fn rakalite_shields_then_bounces_itself() {
    let mut g = main_phase();
    let rak = g.add_card_to_battlefield(0, catalog::rakalite());
    activate(&mut g, 0, rak, Some(Target::Player(0)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 18);
    end_turn(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == rak));
}

#[test]
fn coral_helm_pumps_at_the_cost_of_a_random_card() {
    let mut g = main_phase();
    let helm = g.add_card_to_battlefield(0, catalog::coral_helm());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, helm, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 4);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

#[test]
fn urzas_chalice_gains_life_off_any_artifact_spell() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::urzas_chalice());
    let tome = g.add_card_to_hand(1, catalog::jalum_tome());
    mana(&mut g, 0);
    g.active_player_idx = 1;
    cast(&mut g, 1, tome, None);
    assert_eq!(g.players[0].life, 21);
}

#[test]
fn power_artifact_discounts_the_enchanted_artifacts_abilities() {
    let mut g = main_phase();
    let tome = g.add_card_to_battlefield(0, catalog::jalum_tome()); // {2}, {T}
    let aura = g.add_card_to_hand(0, catalog::power_artifact());
    cast(&mut g, 0, aura, Some(Target::Permanent(tome)));
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tome,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{2} floored to one mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

#[test]
fn titanias_song_animates_artifacts_and_blanks_them() {
    let mut g = main_phase();
    let tome = g.add_card_to_battlefield(0, catalog::jalum_tome()); // mana value 3
    let engine = g.add_card_to_battlefield(0, catalog::dragon_engine());
    g.add_card_to_battlefield(0, catalog::titanias_song());
    let cp = g.computed_permanent(tome).expect("tome");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(
        g.computed_permanent(tome).expect("tome").lost_all_abilities,
        "a noncreature artifact loses its abilities"
    );
    // An artifact that was already a creature is untouched.
    let cp = g.computed_permanent(engine).expect("engine");
    assert_eq!((cp.power, cp.toughness), (1, 3));
}

#[test]
fn mishras_war_machine_bites_when_you_dont_feed_it() {
    let mut g = main_phase();
    let machine = g.add_card_to_battlefield(0, catalog::mishras_war_machine());
    g.players[0].hand.clear();
    for _ in 0..2 {
        end_turn(&mut g);
    }
    assert_eq!(g.players[0].life, 17);
    assert!(g.battlefield_find(machine).expect("live").tapped);
}

// ── The last nine (2026-08 wave) ───────────────────────────────────────────

#[test]
fn clockwork_avian_winds_down_after_attacking() {
    let mut g = main_phase();
    let avian = g.add_card_to_hand(0, catalog::clockwork_avian());
    cast(&mut g, 0, avian, None);
    assert_eq!(g.computed_permanent(avian).expect("avian").power, 4);
    g.clear_sickness(avian);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: avian, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(
        g.battlefield_find(avian).expect("avian").counter_count(CounterType::PlusOnePlusZero),
        3,
    );
}

#[test]
fn clockwork_avian_rewind_stops_at_four() {
    let mut g = main_phase();
    let avian = g.add_card_to_hand(0, catalog::clockwork_avian());
    cast(&mut g, 0, avian, None);
    g.battlefield_find_mut(avian).expect("avian").remove_counters(CounterType::PlusOnePlusZero, 3);
    g.clear_sickness(avian);
    g.step = TurnStep::Upkeep;
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: avian,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(5),
        mode: None,
    })
    .expect("wind up");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(avian).expect("avian").counter_count(CounterType::PlusOnePlusZero),
        4,
        "the cap holds even though X was 5",
    );
}

#[test]
fn clockwork_avian_cant_be_wound_outside_your_upkeep() {
    let mut g = main_phase();
    let avian = g.add_card_to_battlefield(0, catalog::clockwork_avian());
    g.clear_sickness(avian);
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: avian,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: Some(1),
            mode: None,
        })
        .is_err()
    );
}

#[test]
fn goblin_artisans_counters_your_own_spell_on_tails() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    let artisans = g.add_card_to_battlefield(0, catalog::goblin_artisans());
    g.clear_sickness(artisans);
    let chalice = g.add_card_to_hand(0, catalog::urzas_chalice());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: chalice,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast chalice");
    activate(&mut g, 0, artisans, Some(Target::Permanent(chalice)));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == chalice));
}

#[test]
fn golgothian_sylex_wipes_the_antiquities_board() {
    let mut g = main_phase();
    let sylex = g.add_card_to_battlefield(0, catalog::golgothian_sylex());
    let atog = g.add_card_to_battlefield(1, catalog::atog());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, sylex, None);
    assert!(g.battlefield_find(atog).is_none(), "Antiquities card sacrificed");
    assert!(g.battlefield_find(sylex).is_none(), "the Sylex eats itself too");
    assert!(g.battlefield_find(bear).is_some(), "a non-Antiquities card is spared");
}

#[test]
fn primal_clay_enters_as_the_chosen_body() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(2)]));
    let clay = g.add_card_to_hand(0, catalog::primal_clay());
    cast(&mut g, 0, clay, None);
    let cp = g.computed_permanent(clay).expect("clay");
    assert_eq!((cp.power, cp.toughness), (1, 6));
    assert!(cp.keywords.contains(&Keyword::Defender));
}

#[test]
fn tawnoss_coffin_returns_its_prisoner_when_it_untaps() {
    let mut g = main_phase();
    let coffin = g.add_card_to_battlefield(0, catalog::tawnoss_coffin());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).expect("bear").add_counters(CounterType::PlusOnePlusOne, 2);
    activate(&mut g, 0, coffin, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    g.battlefield_find_mut(coffin).expect("coffin").tapped = false;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentUntapped { card_id: coffin }]);
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("returned");
    assert!(back.tapped, "returns tapped");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 2, "noted counters restored");
}

#[test]
fn tetravus_trades_counters_for_fliers_and_back() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Amount(2),
        DecisionAnswer::Amount(0),
    ]));
    let tet = g.add_card_to_hand(0, catalog::tetravus());
    cast(&mut g, 0, tet, None);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tet).expect("tetravus").counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Tetravite").count(),
        2,
    );
    // Second upkeep: reabsorb both tokens. The exile trigger resolves first,
    // so it takes the leading answer.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Amount(2),
        DecisionAnswer::Amount(0),
    ]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tet).expect("tetravus").counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Tetravite").count(), 0);
}

#[test]
fn transmute_artifact_swaps_down_for_free() {
    let mut g = main_phase();
    let chalice = g.add_card_to_battlefield(0, catalog::urzas_chalice()); // {1}
    let target = g.add_card_to_library(0, catalog::millstone()); // {2}
    let ta = g.add_card_to_hand(0, catalog::transmute_artifact());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Cards(vec![target]),
        DecisionAnswer::Bool(true),
    ]));
    cast(&mut g, 0, ta, None);
    assert!(g.battlefield_find(chalice).is_none(), "the chalice was the cost");
    assert!(g.battlefield_find(target).is_some(), "paid the one-mana difference");
}

#[test]
fn transmute_artifact_bins_the_find_when_the_surcharge_is_declined() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::urzas_chalice()); // {1}
    let target = g.add_card_to_library(0, catalog::millstone()); // {2}
    let ta = g.add_card_to_hand(0, catalog::transmute_artifact());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Cards(vec![target]),
        DecisionAnswer::Bool(false),
    ]));
    cast(&mut g, 0, ta, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == target));
}

#[test]
fn urzas_avenger_buys_flying_by_shrinking() {
    let mut g = main_phase();
    let av = g.add_card_to_battlefield(0, catalog::urzas_avenger());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: av,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: Some(1),
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(av).expect("avenger");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

#[test]
fn xenic_poltergeist_animates_an_artifact_at_its_mana_value() {
    let mut g = main_phase();
    let ghost = g.add_card_to_battlefield(0, catalog::xenic_poltergeist());
    g.clear_sickness(ghost);
    let stone = g.add_card_to_battlefield(0, catalog::millstone()); // {2}
    activate(&mut g, 0, ghost, Some(Target::Permanent(stone)));
    let cp = g.computed_permanent(stone).expect("millstone");
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (2, 2));
}
