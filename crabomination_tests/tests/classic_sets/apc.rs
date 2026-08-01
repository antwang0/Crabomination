//! Apocalypse (APC) — the Disciple / Sanctuary cycles, the Bloodfire
//! sacrifice creatures and the kicker spells.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
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

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

/// The printed-keyword bodies.
#[test]
fn apc_keyword_bodies_carry_their_printed_keywords() {
    let cases: &[(fn() -> crabomination::card::CardDefinition, &[Keyword])] = &[
        (catalog::coastal_drake, &[Keyword::Flying]),
        (catalog::haunted_angel, &[Keyword::Flying]),
        (catalog::helionaut, &[Keyword::Flying]),
        (catalog::enlistment_officer, &[Keyword::FirstStrike]),
        (catalog::jungle_barrier, &[Keyword::Defender]),
        (catalog::kavu_mauler, &[Keyword::Trample]),
    ];
    for (factory, expected) in cases {
        let def = factory();
        for kw in *expected {
            assert!(def.keywords.contains(kw), "{} is missing {kw:?}", def.name);
        }
    }
}

/// Ana Disciple rents out flying for {U}.
#[test]
fn ana_disciple_grants_flying() {
    let mut g = main_phase();
    let disciple = g.add_card_to_battlefield(0, catalog::ana_disciple());
    g.clear_sickness(disciple);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, disciple, 0, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Ceta Disciple's second mode is a mana ability.
#[test]
fn ceta_disciple_makes_mana() {
    let mut g = main_phase();
    let disciple = g.add_card_to_battlefield(0, catalog::ceta_disciple());
    g.clear_sickness(disciple);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: disciple,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Dega Sanctuary pays 2, or 4 with both off-colours out.
#[test]
fn dega_sanctuary_scales_with_your_colours() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dega_sanctuary());
    upkeep(&mut g);
    assert_eq!(g.players[0].life, 20, "no black or red permanent");
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green — still nothing
    upkeep(&mut g);
    assert_eq!(g.players[0].life, 20);
    g.add_card_to_battlefield(0, catalog::shivan_dragon()); // red
    upkeep(&mut g);
    assert_eq!(g.players[0].life, 22, "a red permanent is the small mode");
}

/// Ana Sanctuary jumps to +5/+5 with both off-colours out.
#[test]
fn ana_sanctuary_gives_the_big_pump_with_both_colours() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::ana_sanctuary());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::coastal_drake()); // blue
    g.add_card_to_battlefield(0, catalog::grave_defiler()); // black
    upkeep(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 7, "2 + 5");
}

/// Bloodfire Kavu sweeps for two.
#[test]
fn bloodfire_kavu_sweeps_the_board() {
    let mut g = main_phase();
    let kavu = g.add_card_to_battlefield(0, catalog::bloodfire_kavu());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, kavu, 0, None);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(kavu).is_none(), "it sacrificed itself");
}

/// Bloodfire Dwarf spares fliers.
#[test]
fn bloodfire_dwarf_spares_fliers() {
    let mut g = main_phase();
    let dwarf = g.add_card_to_battlefield(0, catalog::bloodfire_dwarf());
    let flier = g.add_card_to_battlefield(1, catalog::coastal_drake());
    let ground = g.add_card_to_battlefield(1, catalog::savannah_lions());
    activate(&mut g, 0, dwarf, 0, None);
    assert!(g.battlefield_find(flier).is_some(), "flying dodged it");
    assert!(g.battlefield_find(ground).is_none());
}

/// Bloodfire Colossus hits players too.
#[test]
fn bloodfire_colossus_burns_everyone() {
    let mut g = main_phase();
    let colossus = g.add_card_to_battlefield(0, catalog::bloodfire_colossus());
    activate(&mut g, 0, colossus, 0, None);
    assert_eq!(g.players[0].life, 14);
    assert_eq!(g.players[1].life, 14);
}

/// Bloodfire Infusion sweeps for the sacrificed creature's power.
#[test]
fn bloodfire_infusion_sweeps_for_the_hosts_power() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let aura = g.add_card_to_hand(0, catalog::bloodfire_infusion());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    activate(&mut g, 0, host, 0, None);
    assert!(g.battlefield_find(theirs).is_none(), "5 damage killed it");
}

/// Death Grasp drains for X.
#[test]
fn death_grasp_drains_for_x() {
    let mut g = main_phase();
    let grasp = g.add_card_to_hand(0, catalog::death_grasp());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: grasp,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.players[0].life, 24);
}

/// Divine Light fogs your own board.
#[test]
fn divine_light_fogs_your_creatures() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let light = g.add_card_to_hand(0, catalog::divine_light());
    cast(&mut g, 0, light, None);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_some(), "the damage was prevented");
}

/// Dwarven Landslide kills a second land when kicked.
#[test]
fn dwarven_landslide_kicked_kills_two_lands() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::island());
    let b = g.add_card_to_battlefield(1, catalog::island());
    g.add_card_to_battlefield(0, catalog::mountain());
    let slide = g.add_card_to_hand(0, catalog::dwarven_landslide());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: slide,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Dwarven Patrol unlocks off a nonred spell.
#[test]
fn dwarven_patrol_untaps_on_a_nonred_spell() {
    let mut g = main_phase();
    let patrol = g.add_card_to_battlefield(0, catalog::dwarven_patrol());
    g.battlefield_find_mut(patrol).unwrap().tapped = true;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None);
    assert!(!g.battlefield_find(patrol).unwrap().tapped);
}

/// Enlistment Officer digs four for Soldiers.
#[test]
fn enlistment_officer_digs_for_soldiers() {
    let mut g = main_phase();
    g.players[0].library.clear();
    let soldier = g.add_card_to_library(0, catalog::helionaut()); // Human Soldier
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let officer = g.add_card_to_hand(0, catalog::enlistment_officer());
    cast(&mut g, 0, officer, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == soldier));
}

/// Evasive Action's tax scales with Domain.
#[test]
fn evasive_action_taxes_by_domain() {
    let mut g = main_phase();
    for land in [catalog::plains(), catalog::island(), catalog::swamp()] {
        g.add_card_to_battlefield(0, land);
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    g.players[1].mana_pool.empty();
    let counter = g.add_card_to_hand(0, catalog::evasive_action());
    cast(&mut g, 0, counter, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, 20, "three basic types is a tax of three it can't pay");
}

/// Glade Gnarr grows off anyone's blue spell.
#[test]
fn glade_gnarr_grows_on_a_blue_spell() {
    let mut g = main_phase();
    let gnarr = g.add_card_to_battlefield(0, catalog::glade_gnarr());
    let drake = g.add_card_to_hand(0, catalog::coastal_drake());
    cast(&mut g, 0, drake, None);
    assert_eq!(g.computed_permanent(gnarr).unwrap().power, 6);
}

/// Goblin Legionnaire's white half is a prevention shield.
#[test]
fn goblin_legionnaire_shields_for_two() {
    let mut g = main_phase();
    let goblin = g.add_card_to_battlefield(0, catalog::goblin_legionnaire());
    activate(&mut g, 0, goblin, 1, Some(Target::Player(0)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 19);
}

/// Haunted Angel arms the opponent on the way out.
#[test]
fn haunted_angel_gifts_an_angel() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::haunted_angel());
    let mut events = Vec::new();
    g.destroy_permanent(angel, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Angel"));
    assert!(g.exile.iter().any(|c| c.id == angel), "it exiles itself");
}

/// Jilt burns a second creature when kicked.
#[test]
fn jilt_kicked_bounces_and_burns() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let jilt = g.add_card_to_hand(0, catalog::jilt());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: jilt,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "bounced");
    assert!(g.battlefield_find(b).is_none(), "2 damage killed the 2/1");
}

/// Kavu Mauler scales with the rest of the Kavu attack.
#[test]
fn kavu_mauler_scales_with_other_attacking_kavu() {
    let mut g = two_player_game();
    let mauler = g.add_card_to_battlefield(0, catalog::kavu_mauler());
    let glider = g.add_card_to_battlefield(0, catalog::kavu_glider());
    for id in [mauler, glider] {
        g.clear_sickness(id);
    }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: mauler, target: AttackTarget::Player(1) },
        Attack { attacker: glider, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mauler).unwrap().power, 5, "4 + one other Kavu");
}

/// Diversionary Tactics taps two of yours to tap one of theirs.
#[test]
fn diversionary_tactics_taps_a_creature() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::diversionary_tactics());
    let tactics = g.battlefield.iter().find(|c| c.definition.name == "Diversionary Tactics").unwrap().id;
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, tactics, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).unwrap().tapped);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.tapped).count(), 2);
}

/// Foul Presence shrinks the host and rents out -1/-1.
#[test]
fn foul_presence_shrinks_and_pokes() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::foul_presence());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().power, 1);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(host);
    activate(&mut g, 0, host, 0, Some(Target::Permanent(victim)));
    assert_eq!(g.computed_permanent(victim).unwrap().power, 1);
}

/// Flowstone Charger trades toughness for power on the swing.
#[test]
fn flowstone_charger_swaps_stats_on_attack() {
    let mut g = two_player_game();
    let charger = g.add_card_to_battlefield(0, catalog::flowstone_charger());
    g.clear_sickness(charger);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: charger, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(charger).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 2));
}

/// Gerrard's Verdict pays 3 life per land discarded.
#[test]
fn gerrards_verdict_pays_for_lands() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    let verdict = g.add_card_to_hand(0, catalog::gerrards_verdict());
    cast(&mut g, 0, verdict, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 0);
    assert_eq!(g.players[0].life, 26, "two lands, 3 life each");
}

/// Dodecapod deploys with counters when an opponent makes you discard it.
#[test]
fn dodecapod_deploys_on_an_opponents_discard() {
    let mut g = main_phase();
    let pod = g.add_card_to_hand(1, catalog::dodecapod());
    let verdict = g.add_card_to_hand(0, catalog::gerrards_verdict());
    cast(&mut g, 0, verdict, Some(Target::Player(1)));
    let perm = g.battlefield_find(pod).expect("deployed instead of binned");
    assert_eq!(perm.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Your own discard still bins it.
#[test]
fn dodecapod_is_binned_by_your_own_discard() {
    let mut g = main_phase();
    let pod = g.add_card_to_hand(0, catalog::dodecapod());
    let mut events = Vec::new();
    g.discard_card(0, pod, &mut events);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pod));
}
