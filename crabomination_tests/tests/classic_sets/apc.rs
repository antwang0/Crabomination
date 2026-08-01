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

/// Necra Sanctuary drains 1, or 3 with both off-colours out.
#[test]
fn necra_sanctuary_scales_with_your_colours() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::necra_sanctuary());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    upkeep(&mut g);
    assert_eq!(g.players[1].life, 19, "green alone is the small mode");
    g.add_card_to_battlefield(0, catalog::savannah_lions()); // white
    upkeep(&mut g);
    assert_eq!(g.players[1].life, 16);
}

/// Raka Sanctuary pings a creature for 1, or 3 with both out.
#[test]
fn raka_sanctuary_pings_for_three_with_both_colours() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::raka_sanctuary());
    g.add_card_to_battlefield(0, catalog::savannah_lions()); // white
    g.add_card_to_battlefield(0, catalog::coastal_drake()); // blue
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    upkeep(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "3 damage killed the 3/3");
}

/// Orim's Thunder burns for the destroyed permanent's mana value when kicked.
#[test]
fn orims_thunder_kicked_burns_for_mana_value() {
    let mut g = main_phase();
    let ench = g.add_card_to_battlefield(1, catalog::powerstone_minefield()); // MV 4
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    let thunder = g.add_card_to_hand(0, catalog::orims_thunder());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: thunder,
        target: Some(Target::Permanent(ench)),
        additional_targets: vec![Target::Permanent(victim)],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none());
    assert!(g.battlefield_find(victim).is_none(), "4 damage killed the 3/3");
}

/// Penumbra Kavu leaves a black body behind.
#[test]
fn penumbra_kavu_leaves_a_shadow() {
    let mut g = main_phase();
    let kavu = g.add_card_to_battlefield(0, catalog::penumbra_kavu());
    let mut events = Vec::new();
    g.destroy_permanent(kavu, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Kavu").expect("token");
    assert_eq!((token.definition.power, token.definition.toughness), (3, 3));
}

/// Quagmire Druid trades a creature for an enchantment.
#[test]
fn quagmire_druid_eats_an_enchantment() {
    let mut g = main_phase();
    let druid = g.add_card_to_battlefield(0, catalog::quagmire_druid());
    g.clear_sickness(druid);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(1, catalog::powerstone_minefield());
    activate(&mut g, 0, druid, 0, Some(Target::Permanent(ench)));
    assert!(g.battlefield_find(ench).is_none());
}

/// Quicksilver Dagger pings and cantrips off the host.
#[test]
fn quicksilver_dagger_pings_and_draws() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(host);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let dagger = g.add_card_to_hand(0, catalog::quicksilver_dagger());
    cast(&mut g, 0, dagger, Some(Target::Permanent(host)));
    let before = g.players[0].hand.len();
    activate(&mut g, 0, host, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Savage Gorilla trades itself for a shrink and a card.
#[test]
fn savage_gorilla_shrinks_and_draws() {
    let mut g = main_phase();
    let ape = g.add_card_to_battlefield(0, catalog::savage_gorilla());
    g.clear_sickness(ape);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, ape, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "-3/-3 killed the 2/2");
    assert!(g.battlefield_find(ape).is_none(), "it sacrificed itself");
}

/// Shield of Duty and Reason grants both protections.
#[test]
fn shield_of_duty_and_reason_grants_two_protections() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::shield_of_duty_and_reason());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    let kws = g.computed_permanent(host).unwrap().keywords;
    assert!(kws.contains(&Keyword::Protection(Color::Green)));
    assert!(kws.contains(&Keyword::Protection(Color::Blue)));
}

/// Spiritmonger grows when it bites a creature.
#[test]
fn spiritmonger_grows_off_combat_damage() {
    let mut g = two_player_game();
    let monger = g.add_card_to_battlefield(0, catalog::spiritmonger());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(monger);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: monger, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, monger)])).expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(monger).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Squee's Embrace hands the host back when it dies.
#[test]
fn squees_embrace_returns_the_host() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::squees_embrace());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().power, 4);
    let mut events = Vec::new();
    g.destroy_permanent(host, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == host));
}

/// Strength of Night's kicker stacks on Zombies.
#[test]
fn strength_of_night_kicked_stacks_on_zombies() {
    let mut g = main_phase();
    let zombie = g.add_card_to_battlefield(0, catalog::mournful_zombie());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(0, catalog::strength_of_night());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: pump,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(zombie).unwrap().power, 5, "2 + 1 + 2");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2 + 1");
}

/// Suffocating Blast counters and burns.
#[test]
fn suffocating_blast_counters_and_burns() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
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
    let blast = g.add_card_to_hand(0, catalog::suffocating_blast());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: blast,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![Target::Permanent(victim)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the Bolt was countered");
    assert!(g.battlefield_find(victim).is_none(), "3 damage killed the 2/2");
}

/// Sylvan Messenger digs four for Elves.
#[test]
fn sylvan_messenger_digs_for_elves() {
    let mut g = main_phase();
    g.players[0].library.clear();
    let elf = g.add_card_to_library(0, catalog::urborg_elf());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let messenger = g.add_card_to_hand(0, catalog::sylvan_messenger());
    cast(&mut g, 0, messenger, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == elf));
}

/// Temporal Spring puts a permanent on top of its library.
#[test]
fn temporal_spring_tucks_a_permanent() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spring = g.add_card_to_hand(0, catalog::temporal_spring());
    cast(&mut g, 0, spring, Some(Target::Permanent(bear)));
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(bear));
}

/// Tranquil Path sweeps enchantments and cantrips.
#[test]
fn tranquil_path_sweeps_enchantments() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(1, catalog::powerstone_minefield());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    let path = g.add_card_to_hand(0, catalog::tranquil_path());
    cast(&mut g, 0, path, None);
    assert!(g.battlefield_find(mine).is_none());
    assert_eq!(g.players[0].hand.len(), before + 1, "the Path left, a card came in");
}

/// Unnatural Selection rewrites a creature's type.
#[test]
fn unnatural_selection_rewrites_a_type() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::unnatural_selection());
    let sel = g.battlefield.iter().find(|c| c.definition.name == "Unnatural Selection").unwrap().id;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, sel, 0, Some(Target::Permanent(bear)));
    let types = g.computed_permanent(bear).unwrap().subtypes.creature_types;
    assert_eq!(types.len(), 1, "the printed Bear type was replaced");
    assert!(!types.contains(&crabomination::card::CreatureType::Bear));
}

/// Urborg Uprising returns two creatures and cantrips.
#[test]
fn urborg_uprising_returns_two_creatures() {
    let mut g = main_phase();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::hill_giant());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let uprising = g.add_card_to_hand(0, catalog::urborg_uprising());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: uprising,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a));
    assert!(g.players[0].hand.iter().any(|c| c.id == b));
}

/// Whirlpool Rider refreshes your hand.
#[test]
fn whirlpool_rider_refreshes_your_hand() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::hill_giant());
    }
    let rider = g.add_card_to_hand(0, catalog::whirlpool_rider());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, rider, None);
    assert_eq!(g.players[0].hand.len(), before - 1, "the Rider left the hand, the rest cycled");
}

/// Overgrown Estate trades lands for life.
#[test]
fn overgrown_estate_trades_lands_for_life() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overgrown_estate());
    let estate = g.battlefield.iter().find(|c| c.definition.name == "Overgrown Estate").unwrap().id;
    g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, 0, estate, 0, None);
    assert_eq!(g.players[0].life, 23);
}

/// Powerstone Minefield bites everything that fights.
#[test]
fn powerstone_minefield_bites_attackers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::powerstone_minefield());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // Through `perform_action` so the unified dispatcher sees AttackerDeclared
    // (`declare_attackers` alone only walks the hardcoded SelfSource scope).
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2 damage killed the 2/2");
}

/// Razorfin Hunter is a straight pinger.
#[test]
fn razorfin_hunter_pings() {
    let mut g = main_phase();
    let hunter = g.add_card_to_battlefield(0, catalog::razorfin_hunter());
    g.clear_sickness(hunter);
    activate(&mut g, 0, hunter, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19);
}

/// Urborg Elf taps for the Ana wedge.
#[test]
fn urborg_elf_taps_for_mana() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::urborg_elf());
    g.clear_sickness(elf);
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::ActivateAbility {
        card_id: elf,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Aether Mutation pays out Saprolings equal to the bounced creature's cost.
#[test]
fn aether_mutation_pays_out_saprolings() {
    let mut g = main_phase();
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // MV 6
    let mutation = g.add_card_to_hand(0, catalog::aether_mutation());
    cast(&mut g, 0, mutation, Some(Target::Permanent(dragon)));
    assert!(g.players[1].hand.iter().any(|c| c.id == dragon));
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 6);
}

/// Death Mutation kills a nonblack creature and pays out.
#[test]
fn death_mutation_kills_and_pays_out() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant()); // MV 4
    let mutation = g.add_card_to_hand(0, catalog::death_mutation());
    cast(&mut g, 0, mutation, Some(Target::Permanent(giant)));
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 4);
}

/// Desolation Angel takes only your lands unkicked.
#[test]
fn desolation_angel_unkicked_burns_only_your_lands() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::swamp());
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    let angel = g.add_card_to_hand(0, catalog::desolation_angel());
    cast(&mut g, 0, angel, None);
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_some());
}

/// Kicked, it takes everyone's.
#[test]
fn desolation_angel_kicked_burns_every_land() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    let angel = g.add_card_to_hand(0, catalog::desolation_angel());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: angel,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
}

/// Desolation Giant spares itself.
#[test]
fn desolation_giant_spares_itself() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_hand(0, catalog::desolation_giant());
    cast(&mut g, 0, giant, None);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(giant).is_some(), "it kept itself");
}

/// Brass Herald names a type, digs for it, and lords it.
#[test]
fn brass_herald_names_digs_and_lords() {
    let mut g = main_phase();
    g.players[0].library.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let herald = g.add_card_to_hand(0, catalog::brass_herald());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::CreatureType(
            crabomination::card::CreatureType::Bear,
        ),
    ]));
    cast(&mut g, 0, herald, None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "the Bear lord is on");
    assert_eq!(g.players[0].hand.len(), 4, "all four Bears came to hand");
}

/// Dragon Arch deploys a multicolored creature from hand.
#[test]
fn dragon_arch_deploys_a_gold_creature() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dragon_arch());
    let arch = g.battlefield.iter().find(|c| c.definition.name == "Dragon Arch").unwrap().id;
    let angel = g.add_card_to_hand(0, catalog::lightning_angel());
    activate(&mut g, 0, arch, 0, None);
    assert!(g.battlefield_find(angel).is_some());
}

/// Fervent Charge pumps every attacker.
#[test]
fn fervent_charge_pumps_attackers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fervent_charge());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
}

/// Fungal Shambler trades a connection for a card each way.
#[test]
fn fungal_shambler_draws_and_strips() {
    let mut g = two_player_game();
    let shambler = g.add_card_to_battlefield(0, catalog::fungal_shambler());
    g.clear_sickness(shambler);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: shambler,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "they discarded");
}

/// Gerrard Capashen taxes their hand each upkeep.
#[test]
fn gerrard_capashen_gains_per_card_in_hand() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::gerrard_capashen());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    upkeep(&mut g);
    assert_eq!(g.players[0].life, 23);
}

/// Goblin Trenches turns a land into two bodies.
#[test]
fn goblin_trenches_makes_two_goblins() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::goblin_trenches());
    let trenches = g.battlefield.iter().find(|c| c.definition.name == "Goblin Trenches").unwrap().id;
    g.add_card_to_battlefield(0, catalog::mountain());
    activate(&mut g, 0, trenches, 0, None);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Goblin Soldier").count(),
        2
    );
}

/// Last Caress drains one and cantrips.
#[test]
fn last_caress_drains_and_draws() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    let caress = g.add_card_to_hand(0, catalog::last_caress());
    cast(&mut g, 0, caress, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[0].life, 21);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Lightning Angel ships all three keywords.
#[test]
fn lightning_angel_keywords() {
    let def = catalog::lightning_angel();
    for kw in [Keyword::Flying, Keyword::Vigilance, Keyword::Haste] {
        assert!(def.keywords.contains(&kw));
    }
}
