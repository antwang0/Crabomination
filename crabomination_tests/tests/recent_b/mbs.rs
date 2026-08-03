//! Mirrodin Besieged (MBS) — Infect, Metalcraft, Battle cry, Living weapon
//! and the proliferate shells (`catalog::sets::mbs`).

use crabomination::card::{CardDefinition, CardId, CounterType, CreatureType, Keyword};
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

/// Three artifacts turn metalcraft on.
fn metalcraft_on(g: &mut GameState, seat: usize) {
    for _ in 0..3 {
        g.add_card_to_battlefield(seat, catalog::hexplate_golem());
    }
}

// ── Bodies ──────────────────────────────────────────────────────────────────

/// Printed stats and keywords, one table over the set.
#[test]
fn mbs_bodies_have_their_printed_stats() {
    let rows: &[(fn() -> CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::blightwidow, 2, 4, &[Keyword::Reach, Keyword::Infect]),
        (catalog::flensermite, 1, 1, &[Keyword::Infect, Keyword::Lifelink]),
        (catalog::priests_of_norn, 1, 4, &[Keyword::Vigilance, Keyword::Infect]),
        (catalog::phyrexian_juggernaut, 5, 5, &[Keyword::Infect, Keyword::MustAttack]),
        (catalog::quilled_slagwurm, 8, 8, &[]),
        (catalog::hexplate_golem, 5, 7, &[]),
        (catalog::lumengrid_gargoyle, 4, 4, &[Keyword::Flying]),
        (catalog::glissa_the_traitor, 3, 3, &[Keyword::FirstStrike, Keyword::Deathtouch]),
        (catalog::koths_courier, 2, 3, &[Keyword::Landwalk(crabomination::card::LandType::Forest)]),
    ];
    for (make, p, t, kws) in rows {
        let def = make();
        assert_eq!((def.power, def.toughness), (*p, *t), "{}", def.name);
        for kw in *kws {
            assert!(def.keywords.contains(kw), "{} missing {kw:?}", def.name);
        }
    }
}

// ── Metalcraft ──────────────────────────────────────────────────────────────

/// Mirran Mettle is +2/+2, or +4/+4 once three artifacts are down.
#[test]
fn mirran_mettle_doubles_under_metalcraft() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::tangle_mantis()); // 3/4
    let mettle = g.add_card_to_hand(0, catalog::mirran_mettle());
    cast(&mut g, 0, mettle, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5);
    metalcraft_on(&mut g, 0);
    let mettle2 = g.add_card_to_hand(0, catalog::mirran_mettle());
    cast(&mut g, 0, mettle2, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 9, "5 + 4");
}

/// Spire Serpent gets up and swings once metalcraft is on.
#[test]
fn spire_serpent_attacks_under_metalcraft() {
    let mut g = main_phase();
    let serpent = g.add_card_to_battlefield(0, catalog::spire_serpent());
    g.clear_sickness(serpent);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: serpent, target: AttackTarget::Player(1) }])
            .is_err(),
        "defender holds it back"
    );
    metalcraft_on(&mut g, 0);
    assert_eq!(g.computed_permanent(serpent).unwrap().power, 5, "3 + 2");
    g.declare_attackers(vec![Attack { attacker: serpent, target: AttackTarget::Player(1) }])
        .expect("metalcraft lets it attack");
}

/// Spiraling Duelist picks up double strike with three artifacts out.
#[test]
fn spiraling_duelist_gains_double_strike() {
    let mut g = main_phase();
    let duelist = g.add_card_to_battlefield(0, catalog::spiraling_duelist());
    assert!(!g.computed_permanent(duelist).unwrap().keywords.contains(&Keyword::DoubleStrike));
    metalcraft_on(&mut g, 0);
    assert!(g.computed_permanent(duelist).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Concussive Bolt's metalcraft half turns off their blocks.
#[test]
fn concussive_bolt_shuts_off_blocking_under_metalcraft() {
    let mut g = main_phase();
    metalcraft_on(&mut g, 0);
    let blocker = g.add_card_to_battlefield(1, catalog::tangle_mantis());
    let attacker = g.add_card_to_battlefield(0, catalog::ogre_resister());
    g.clear_sickness(attacker);
    let bolt = g.add_card_to_hand(0, catalog::concussive_bolt());
    let life = g.players[1].life;
    cast(&mut g, 0, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 4);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.declare_blockers(vec![(blocker, attacker)]).is_err(), "it can't block this turn");
}

// ── Infect / poison ─────────────────────────────────────────────────────────

/// Phyresis turns any creature into a poison clock.
#[test]
fn phyresis_grants_infect() {
    let mut g = main_phase();
    let beater = g.add_card_to_battlefield(0, catalog::quilled_slagwurm());
    let aura = g.add_card_to_hand(0, catalog::phyresis());
    cast(&mut g, 0, aura, Some(Target::Permanent(beater)));
    assert!(g.computed_permanent(beater).unwrap().keywords.contains(&Keyword::Infect));
}

/// Phyrexian Vatmother poisons its own controller every upkeep.
#[test]
fn phyrexian_vatmother_poisons_you() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::phyrexian_vatmother());
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].poison_counters, 1);
}

/// Septic Rats grows once the defender is already poisoned.
#[test]
fn septic_rats_grows_against_a_poisoned_defender() {
    let mut g = main_phase();
    let rats = g.add_card_to_battlefield(0, catalog::septic_rats());
    g.clear_sickness(rats);
    g.players[1].poison_counters = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: rats, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(rats).unwrap().power, 3, "2 + 1");
}

/// Pistus Strike kills the flier and poisons its controller.
#[test]
fn pistus_strike_kills_a_flier_and_poisons() {
    let mut g = main_phase();
    let flier = g.add_card_to_battlefield(1, catalog::lumengrid_gargoyle());
    let strike = g.add_card_to_hand(0, catalog::pistus_strike());
    cast(&mut g, 0, strike, Some(Target::Permanent(flier)));
    assert!(g.battlefield_find(flier).is_none());
    assert_eq!(g.players[1].poison_counters, 1);
}

/// Virulent Wound shrinks now and poisons when the creature dies.
#[test]
fn virulent_wound_poisons_on_the_kill() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::tangle_mantis()); // 3/4
    let wound = g.add_card_to_hand(0, catalog::virulent_wound());
    cast(&mut g, 0, wound, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    let mut events = vec![];
    g.destroy_permanent(victim, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 1);
}

/// Burn the Impure punishes the infect creature's controller too.
#[test]
fn burn_the_impure_hits_the_infect_controller() {
    let mut g = main_phase();
    let infector = g.add_card_to_battlefield(1, catalog::blightwidow()); // 2/4 infect
    let burn = g.add_card_to_hand(0, catalog::burn_the_impure());
    let life = g.players[1].life;
    cast(&mut g, 0, burn, Some(Target::Permanent(infector)));
    assert_eq!(g.players[1].life, life - 3, "the infect rider fired");
}

// ── Proliferate / counters ──────────────────────────────────────────────────

/// Core Prowler proliferates as it dies.
#[test]
fn core_prowler_proliferates_on_death() {
    let mut g = main_phase();
    let prowler = g.add_card_to_battlefield(0, catalog::core_prowler());
    g.players[1].poison_counters = 2;
    let mut events = vec![];
    g.destroy_permanent(prowler, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 3);
}

/// Plaguemaw Beast turns a creature into a proliferate.
#[test]
fn plaguemaw_beast_proliferates_for_a_creature() {
    let mut g = main_phase();
    let beast = g.add_card_to_battlefield(0, catalog::plaguemaw_beast());
    g.clear_sickness(beast);
    g.add_card_to_battlefield(0, catalog::oculus());
    g.players[1].poison_counters = 1;
    activate(&mut g, 0, beast, 0, None);
    assert_eq!(g.players[1].poison_counters, 2);
}

/// Melira's Keepers refuses every counter.
#[test]
fn meliras_keepers_cant_take_counters() {
    let mut g = main_phase();
    let keepers = g.add_card_to_battlefield(0, catalog::meliras_keepers());
    let wound = g.add_card_to_hand(1, catalog::virulent_wound());
    g.active_player_idx = 1;
    cast(&mut g, 1, wound, Some(Target::Permanent(keepers)));
    assert_eq!(g.battlefield_find(keepers).unwrap().counter_count(CounterType::MinusOneMinusOne), 0);
}

/// Choking Fumes shrinks the whole attacking team.
#[test]
fn choking_fumes_shrinks_every_attacker() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::tangle_mantis());
    let b = g.add_card_to_battlefield(1, catalog::ogre_resister());
    for id in [a, b] {
        g.attacking.push(Attack { attacker: id, target: AttackTarget::Player(0) });
    }
    let fumes = g.add_card_to_hand(0, catalog::choking_fumes());
    cast(&mut g, 0, fumes, None);
    for id in [a, b] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    }
}

/// Titan Forge banks three charges, then trades them for a 9/9.
#[test]
fn titan_forge_banks_charges_for_a_golem() {
    let mut g = main_phase();
    let forge = g.add_card_to_battlefield(0, catalog::titan_forge());
    for _ in 0..3 {
        activate(&mut g, 0, forge, 0, None);
        g.battlefield.iter_mut().find(|c| c.id == forge).unwrap().tapped = false;
    }
    assert_eq!(g.battlefield_find(forge).unwrap().counter_count(CounterType::Charge), 3);
    activate(&mut g, 0, forge, 1, None);
    let golem = g.battlefield.iter().find(|c| c.definition.name == "Golem").expect("a Golem");
    assert_eq!((golem.definition.power, golem.definition.toughness), (9, 9));
}

// ── Living weapon / Equipment ───────────────────────────────────────────────

/// Mortarpod arrives with its Germ and turns it into a Shock.
#[test]
fn mortarpod_makes_a_germ_and_shoots_it() {
    let mut g = main_phase();
    let pod = g.add_card_to_hand(0, catalog::mortarpod());
    cast(&mut g, 0, pod, None);
    let germ = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Phyrexian Germ")
        .map(|c| c.id)
        .expect("the Germ");
    let pod_id = g.battlefield.iter().find(|c| c.definition.name == "Mortarpod").unwrap().id;
    assert_eq!(g.battlefield_find(pod_id).unwrap().attached_to, Some(germ));
    assert_eq!(g.computed_permanent(germ).unwrap().toughness, 1, "0/0 + 0/+1");
    let life = g.players[1].life;
    activate(&mut g, 0, germ, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 1);
}

/// Skinwing's Germ flies.
#[test]
fn skinwing_germ_flies() {
    let mut g = main_phase();
    let wing = g.add_card_to_hand(0, catalog::skinwing());
    cast(&mut g, 0, wing, None);
    let germ = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Phyrexian Germ")
        .map(|c| c.id)
        .expect("the Germ");
    let cp = g.computed_permanent(germ).expect("computed");
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Piston Sledge attaches on entry and its equip cost eats an artifact.
#[test]
fn piston_sledge_equips_by_sacrificing_an_artifact() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::ogre_resister());
    let other = g.add_card_to_battlefield(0, catalog::tangle_mantis());
    let fodder = g.add_card_to_battlefield(0, catalog::hexplate_golem());
    let sledge = g.add_card_to_hand(0, catalog::piston_sledge());
    g.decider =
        Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(host))]));
    cast(&mut g, 0, sledge, None);
    let sledge_id = g.battlefield.iter().find(|c| c.definition.name == "Piston Sledge").unwrap().id;
    assert_eq!(g.battlefield_find(sledge_id).unwrap().attached_to, Some(host));
    assert_eq!(g.computed_permanent(host).unwrap().power, 7, "4 + 3");
    mana(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sledge_id, target: other }).expect("equip");
    assert!(g.battlefield_find(fodder).is_none(), "the equip cost ate an artifact");
    assert_eq!(g.battlefield_find(sledge_id).unwrap().attached_to, Some(other));
}

/// Copper Carapace is a big buff that turns the host off defense.
#[test]
fn copper_carapace_pumps_but_blocks_nothing() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::ogre_resister());
    let carapace = g.add_card_to_hand(0, catalog::copper_carapace());
    cast(&mut g, 0, carapace, None);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Copper Carapace").unwrap().id;
    mana(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: id, target: host }).expect("equip");
    let cp = g.computed_permanent(host).expect("computed");
    assert_eq!((cp.power, cp.toughness), (6, 5));
    assert!(cp.keywords.contains(&Keyword::CantBlock));
}

/// Training Drone sits out combat until it's carrying something.
#[test]
fn training_drone_needs_equipment_to_fight() {
    let mut g = main_phase();
    let drone = g.add_card_to_battlefield(0, catalog::training_drone());
    g.clear_sickness(drone);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: drone, target: AttackTarget::Player(1) }])
            .is_err(),
        "unequipped, it can't attack"
    );
    let claw = g.add_card_to_hand(0, catalog::viridian_claw());
    g.step = TurnStep::PreCombatMain;
    cast(&mut g, 0, claw, None);
    let claw_id = g.battlefield.iter().find(|c| c.definition.name == "Viridian Claw").unwrap().id;
    mana(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: claw_id, target: drone }).expect("equip");
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: drone, target: AttackTarget::Player(1) }])
        .expect("equipped, it can");
}

// ── Battle cry / combat ─────────────────────────────────────────────────────

/// Kuldotha Ringleader pumps every other attacker.
#[test]
fn kuldotha_ringleader_battle_cries() {
    let mut g = main_phase();
    let leader = g.add_card_to_battlefield(0, catalog::kuldotha_ringleader());
    let buddy = g.add_card_to_battlefield(0, catalog::ogre_resister());
    for id in [leader, buddy] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: leader, target: AttackTarget::Player(1) },
        Attack { attacker: buddy, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(buddy).unwrap().power, 5, "4 + 1");
    assert_eq!(g.computed_permanent(leader).unwrap().power, 4, "not itself");
}

/// Victory's Herald hands the team flying and lifelink.
#[test]
fn victorys_herald_lifts_the_team() {
    let mut g = main_phase();
    let herald = g.add_card_to_battlefield(0, catalog::victorys_herald());
    let ground = g.add_card_to_battlefield(0, catalog::ogre_resister());
    for id in [herald, ground] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: herald, target: AttackTarget::Player(1) },
        Attack { attacker: ground, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ground).expect("computed");
    assert!(cp.keywords.contains(&Keyword::Flying));
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

/// Spin Engine pushes a blocker out of the way for a red mana.
#[test]
fn spin_engine_clears_its_blocker() {
    let mut g = main_phase();
    let engine = g.add_card_to_battlefield(0, catalog::spin_engine());
    let blocker = g.add_card_to_battlefield(1, catalog::tangle_mantis());
    g.clear_sickness(engine);
    activate(&mut g, 0, engine, 0, Some(Target::Permanent(blocker)));
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: engine, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.declare_blockers(vec![(blocker, engine)]).is_err());
}

// ── Card-advantage / value ──────────────────────────────────────────────────

/// Psychosis Crawler is as big as your hand and drains on every draw.
#[test]
fn psychosis_crawler_scales_and_drains() {
    let mut g = main_phase();
    let crawler = g.add_card_to_battlefield(0, catalog::psychosis_crawler());
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::oculus());
        g.add_card_to_library(0, catalog::oculus());
    }
    assert_eq!(g.computed_permanent(crawler).unwrap().power, 4);
    let life = g.players[1].life;
    let mut events = vec![];
    g.draw_one(0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
    assert_eq!(g.computed_permanent(crawler).unwrap().power, 5);
}

/// Treasure Mage digs out a six-drop.
#[test]
fn treasure_mage_finds_a_big_artifact() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::tangle_mantis());
    let big = g.add_card_to_library(0, catalog::hexplate_golem()); // MV 7
    let mage = g.add_card_to_hand(0, catalog::treasure_mage());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(big)),
    ]));
    cast(&mut g, 0, mage, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == big));
}

/// Myr Sire leaves a replacement behind.
#[test]
fn myr_sire_leaves_a_myr() {
    let mut g = main_phase();
    let sire = g.add_card_to_battlefield(0, catalog::myr_sire());
    let mut events = vec![];
    g.destroy_permanent(sire, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Myr"));
}

/// Myr Turbine mints Myr, then tutors once five of them can tap.
#[test]
fn myr_turbine_mints_then_tutors() {
    let mut g = main_phase();
    let turbine = g.add_card_to_battlefield(0, catalog::myr_turbine());
    activate(&mut g, 0, turbine, 0, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Myr"));
    for _ in 0..5 {
        let id = g.add_card_to_battlefield(0, catalog::plague_myr());
        g.clear_sickness(id);
    }
    let wanted = g.add_card_to_library(0, catalog::plague_myr());
    g.battlefield.iter_mut().find(|c| c.id == turbine).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(wanted))]));
    activate(&mut g, 0, turbine, 1, None);
    assert!(g.battlefield_find(wanted).is_some());
}

/// Glissa, the Traitor rebuys an artifact when their creature dies.
#[test]
fn glissa_returns_an_artifact_on_their_death() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::glissa_the_traitor());
    let junk = g.add_card_to_graveyard(0, catalog::hexplate_golem());
    let theirs = g.add_card_to_battlefield(1, catalog::oculus());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(junk)),
    ]));
    let mut events = vec![];
    g.destroy_permanent(theirs, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == junk));
}

/// Magnetic Mine pings whoever lost the artifact.
#[test]
fn magnetic_mine_pings_the_artifacts_controller() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::magnetic_mine());
    let theirs = g.add_card_to_battlefield(1, catalog::hexplate_golem());
    let life = g.players[1].life;
    let mut events = vec![];
    g.destroy_permanent(theirs, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
}

/// Shimmer Myr lets artifacts land at instant speed.
#[test]
fn shimmer_myr_gives_artifacts_flash() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::shimmer_myr());
    let golem = g.add_card_to_hand(0, catalog::hexplate_golem());
    g.step = TurnStep::DeclareBlockers;
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: golem,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("artifacts have flash");
}

/// Cryptoplasm copies a creature but keeps its own upkeep trigger.
#[test]
fn cryptoplasm_copies_and_keeps_its_ability() {
    let mut g = main_phase();
    let plasm = g.add_card_to_battlefield(0, catalog::cryptoplasm());
    let model = g.add_card_to_battlefield(1, catalog::quilled_slagwurm()); // 8/8
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(model)),
    ]));
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(plasm).unwrap().power, 8);
    assert!(
        !g.battlefield_find(plasm).unwrap().definition.triggered_abilities.is_empty(),
        "it keeps the upkeep trigger"
    );
}

/// Steel Sabotage answers an artifact either way.
#[test]
fn steel_sabotage_bounces_an_artifact() {
    let mut g = main_phase();
    let golem = g.add_card_to_battlefield(1, catalog::hexplate_golem());
    let sabotage = g.add_card_to_hand(0, catalog::steel_sabotage());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: sabotage,
        target: Some(Target::Permanent(golem)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == golem));
}

/// Praetor's Counsel empties the graveyard and lifts the hand cap.
#[test]
fn praetors_counsel_refills_and_uncaps() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::oculus());
    }
    let counsel = g.add_card_to_hand(0, catalog::praetors_counsel());
    cast(&mut g, 0, counsel, None);
    assert_eq!(g.players[0].hand.len(), 3);
    assert!(g.players[0].graveyard.is_empty(), "it exiled itself, not to the yard");
    assert!(g.effective_max_hand_size(0).is_none());
}

/// Contested War Zone changes hands when its controller gets hit.
#[test]
fn contested_war_zone_changes_hands() {
    let mut g = main_phase();
    let zone = g.add_card_to_battlefield(0, catalog::contested_war_zone());
    let attacker = g.add_card_to_battlefield(1, catalog::ogre_resister());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(zone).unwrap().controller, 1);
}

/// Into the Core exiles two artifacts at once.
#[test]
fn into_the_core_exiles_two() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::hexplate_golem());
    let b = g.add_card_to_battlefield(1, catalog::lumengrid_gargoyle());
    let core = g.add_card_to_hand(0, catalog::into_the_core());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: core,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Gore Vassal shrinks and can regenerate what it shrank.
#[test]
fn gore_vassal_shrinks_then_saves() {
    let mut g = main_phase();
    let vassal = g.add_card_to_battlefield(0, catalog::gore_vassal());
    let target = g.add_card_to_battlefield(0, catalog::tangle_mantis()); // 3/4
    activate(&mut g, 0, vassal, 0, Some(Target::Permanent(target)));
    let c = g.battlefield_find(target).expect("still around");
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 1);
    assert!(c.regeneration_shields > 0, "toughness 3 ≥ 1, so it got a shield");
}

/// Nested Ghoul mints a 2/2 for every ping.
#[test]
fn nested_ghoul_mints_a_zombie_per_ping() {
    let mut g = main_phase();
    let ghoul = g.add_card_to_battlefield(0, catalog::nested_ghoul());
    let mut events = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(ghoul),
        1,
        None,
        &mut events,
    );
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Zombie"));
}

/// Rot Wolf draws off anything it poisoned that dies.
#[test]
fn rot_wolf_draws_off_its_kills() {
    let mut g = main_phase();
    let wolf = g.add_card_to_battlefield(0, catalog::rot_wolf());
    let victim = g.add_card_to_battlefield(1, catalog::tangle_mantis());
    g.add_card_to_library(0, catalog::oculus());
    g.battlefield.iter_mut().find(|c| c.id == victim).unwrap().damaged_by_this_turn.push(wolf);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    let mut events = vec![];
    g.destroy_permanent(victim, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Fangren Marauder banks five off every artifact that dies.
#[test]
fn fangren_marauder_gains_off_artifact_deaths() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::fangren_marauder());
    let golem = g.add_card_to_battlefield(1, catalog::hexplate_golem());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    let mut events = vec![];
    g.destroy_permanent(golem, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 5);
}

/// Phyrexian Rebirth wraths, then leaves an X/X for the bodies.
#[test]
fn phyrexian_rebirth_leaves_an_xx_horror() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::oculus());
    }
    let rebirth = g.add_card_to_hand(0, catalog::phyrexian_rebirth());
    cast(&mut g, 0, rebirth, None);
    let horror = g
        .battlefield
        .iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Horror))
        .map(|c| c.id)
        .expect("the Horror");
    assert_eq!(g.computed_permanent(horror).unwrap().power, 3);
}
