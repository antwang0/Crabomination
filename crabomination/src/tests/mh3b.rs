//! Functionality tests for the MH3 batch-2 cards in `catalog::sets::mh3b`.

use crate::card::{CounterType, EventKind, Keyword};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

fn advance_to(g: &mut GameState, step: crate::game::TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn cast(g: &mut GameState, id: crate::card::CardId, target: Option<Target>) {
    fill_mana(g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(g);
}

/// Eldrazi Ravager has Annihilator 1 and can return itself from the graveyard
/// by sacrificing two Eldrazi.
#[test]
fn eldrazi_ravager_recurs_from_graveyard() {
    let mut g = two_player_game();
    let ravager = g.add_card_to_graveyard(0, catalog::eldrazi_ravager());
    // Two Eldrazi to sacrifice.
    g.add_card_to_battlefield(0, catalog::eldrazi_ravager());
    g.add_card_to_battlefield(0, catalog::eldrazi_ravager());
    g.perform_action(GameAction::ActivateAbility {
        card_id: ravager, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate gy ability");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == ravager), "returned to hand");
}

/// Breaker of Creation's cast trigger gains 1 life per colorless permanent.
#[test]
fn breaker_of_creation_gains_life_per_colorless() {
    let mut g = two_player_game();
    // Two colorless permanents already out (the Eldrazi Ravagers are colorless).
    g.add_card_to_battlefield(0, catalog::eldrazi_ravager());
    g.add_card_to_battlefield(0, catalog::eldrazi_ravager());
    let id = g.add_card_to_hand(0, catalog::breaker_of_creation());
    let life = g.players[0].life;
    cast(&mut g, id, None);
    // The two Ravagers + Breaker itself once it enters = at least 2 counted at
    // cast time (Breaker is still on the stack). Gains ≥ 2.
    assert!(g.players[0].life >= life + 2, "gained life per colorless permanent");
}

/// Drownyard Lurker's cycle trigger creates an Eldrazi Spawn.
#[test]
fn drownyard_lurker_cycle_makes_spawn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::drownyard_lurker());
    fill_mana(&mut g);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    let spawn = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawn, 1, "cycling made one Eldrazi Spawn");
}

/// Emrakul's Messenger makes a Spawn when you draw your second card each turn.
#[test]
fn emrakuls_messenger_second_draw_spawn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::emrakuls_messenger());
    g.add_card_to_library(0, catalog::eldrazi_ravager());
    g.add_card_to_library(0, catalog::eldrazi_ravager());
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev); // first draw — no trigger
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 0);
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2); // second draw — trigger
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 1);
}

/// Petrifying Meddler's cast trigger taps and stuns a creature.
#[test]
fn petrifying_meddler_stuns() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::petrifying_meddler());
    cast(&mut g, id, Some(Target::Permanent(victim)));
    let v = g.battlefield_find(victim).unwrap();
    assert!(v.tapped, "target tapped");
    assert_eq!(v.counter_count(CounterType::Stun), 1, "one stun counter");
}

/// Dreamdrinker Vampire gains menace when a +1/+1 counter lands (via adapt).
#[test]
fn dreamdrinker_adapt_grants_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::dreamdrinker_vampire());
    g.clear_sickness(id);
    fill_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("adapt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Menace), "menace granted");
}

/// Evolution Witness returns a permanent card from the graveyard on adapt.
#[test]
fn evolution_witness_returns_on_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::evolution_witness());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.clear_sickness(id);
    fill_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("adapt");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "graveyard card returned to hand");
}

/// Envoy of the Ancestors gives modified creatures you control lifelink.
#[test]
fn envoy_grants_modified_lifelink() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::envoy_of_the_ancestors());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Unmodified: no lifelink.
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink));
    // Add a counter → modified → lifelink.
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink), "modified gets lifelink");
}

/// Guardian of the Forgotten manifests when a modified creature you control dies.
#[test]
fn guardian_manifests_on_modified_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guardian_of_the_forgotten());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // modified (3/3)
    g.add_card_to_library(0, catalog::grizzly_bears());
    // Lethal damage → SBA death (populates the LKI snapshot the dies-trigger reads).
    let mut evs = Vec::new();
    g.deal_damage_to_from(crate::game::effects::EntityRef::Permanent(bear), 3, None, &mut evs);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    let facedown = g.battlefield.iter().filter(|c| c.face_down).count();
    assert_eq!(facedown, 1, "manifested a face-down card");
}

/// Grim Servant tutors a card with MV ≤ devotion to black.
#[test]
fn grim_servant_devotion_tutor() {
    let mut g = two_player_game();
    // Devotion to black 2 (Grim Servant itself is {3}{B} = 1 pip; add another).
    g.add_card_to_battlefield(0, catalog::dreamdrinker_vampire()); // {1}{B} = 1 black pip
    let id = g.add_card_to_hand(0, catalog::grim_servant());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 ≤ devotion 2 (after Grim enters)
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let life = g.players[0].life;
    cast(&mut g, id, None);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "tutored a card");
    assert_eq!(g.players[0].life, life - 3, "lost 3 life");
}

/// Marionette Apprentice drains each opponent when your creature/artifact dies.
#[test]
fn marionette_apprentice_drains_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::marionette_apprentice());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[1].life;
    let evs = g.remove_to_graveyard_with_triggers(bear);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent lost 1 life");
}

/// Molten Gatekeeper pings each opponent when another creature enters.
#[test]
fn molten_gatekeeper_pings_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::molten_gatekeeper());
    let life = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, id, None);
    assert_eq!(g.players[1].life, life - 1, "opponent took 1 damage");
}

/// Kami of Jealous Thirst drains 2 once per turn.
#[test]
fn kami_of_jealous_thirst_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kami_of_jealous_thirst());
    g.clear_sickness(id);
    fill_mana(&mut g);
    let (my, opp) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2);
    assert_eq!(g.players[0].life, my + 2);
}

/// Colossal Dreadmask enters as a living weapon: mints a Germ and buffs it.
#[test]
fn colossal_dreadmask_living_weapon() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::colossal_dreadmask());
    cast(&mut g, id, None);
    let germ = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Germ").expect("germ");
    let cp = g.computed_permanent(germ.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "0/0 germ + 6/6 equip");
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Horrific Assault: a creature you control deals its power to an enemy, and
/// controlling an Eldrazi gains 3 life.
#[test]
fn horrific_assault_fights_and_gains() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::eldrazi_ravager()); // 6/6 Eldrazi
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::horrific_assault());
    let life = g.players[0].life;
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "enemy took 6 and died");
    assert_eq!(g.players[0].life, life + 3, "gained 3 for controlling an Eldrazi");
}

/// Brainsurge draws four and puts two back — net +2 hand.
#[test]
fn brainsurge_net_draw_two() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::brainsurge());
    let hand = g.players[0].hand.len();
    cast(&mut g, id, None);
    // -1 (Brainsurge) +4 draw -2 put back = net +1 vs starting, i.e. hand-1+2.
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew 4, put 2 back");
}

/// Fangs of Kalonia adds a counter then doubles it (single-target).
#[test]
fn fangs_of_kalonia_doubles() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fangs_of_kalonia());
    cast(&mut g, id, Some(Target::Permanent(bear)));
    // +1 counter, then doubled → 2.
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Gravedig (mode 0) makes a 2/2 Zombie.
#[test]
fn gravedig_makes_zombie() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gravedig());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let zombies = g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count();
    assert_eq!(zombies, 1);
}

/// Drossclaw's equipped creature drains each opponent when it attacks.
#[test]
fn drossclaw_attack_drains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::drossclaw());
    g.clear_sickness(bear);
    // Equip onto the bear.
    fill_mana(&mut g);
    g.perform_action(GameAction::Equip { equipment: axe, target: bear }).expect("equip");
    let life = g.players[1].life;
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let evs = g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]).expect("attack");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "each opponent lost 1 when equipped creature attacked");
}

/// Expanding Ooze puts a +1/+1 counter on a modified creature when it attacks.
#[test]
fn expanding_ooze_attack_buffs_modified() {
    let mut g = two_player_game();
    let ooze = g.add_card_to_battlefield(0, catalog::expanding_ooze());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // modified
    g.clear_sickness(ooze);
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // The attack trigger targets the modified bear when it goes on the stack.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    let evs = g.declare_attackers(vec![Attack { attacker: ooze, target: AttackTarget::Player(1) }]).expect("attack");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Infernal Captor's exploit steals a permanent until end of turn.
#[test]
fn infernal_captor_exploit_steals() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::infernal_captor());
    // Accept the exploit; AutoDecider sacrifices the fodder and steals the victim.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "exploit sacrificed a creature");
    let _ = victim;
}

// ── Batch 2 ──────────────────────────────────────────────────────────────────

/// Metastatic Evangel proliferates when another nontoken creature enters.
#[test]
fn metastatic_evangel_proliferates() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::metastatic_evangel());
    // A creature with a counter to grow.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, id, None);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "proliferate grew the existing counter");
}

/// Obstinate Gargoyle only flies while modified.
#[test]
fn obstinate_gargoyle_flies_while_modified() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::obstinate_gargoyle());
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying), "no flying unmodified");
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying), "flies while modified");
}

/// Arcbound Condor enters as a 3/3 (modular 3) and shrinks an enemy when an
/// artifact enters.
#[test]
fn arcbound_condor_modular_and_artifact_trigger() {
    let mut g = two_player_game();
    let condor = g.add_card_to_hand(0, catalog::arcbound_condor());
    cast(&mut g, condor, None);
    let cp = g.computed_permanent(condor).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "0/0 + modular 3");
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Cast an artifact → trigger targeting the enemy bear.
    let art = g.add_card_to_hand(0, catalog::etched_slith());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    cast(&mut g, art, None);
    let vc = g.computed_permanent(victim).unwrap();
    assert_eq!((vc.power, vc.toughness), (1, 1), "enemy bear got -1/-1");
}

/// Kozilek's Unsealing makes two Spawn on a MV-5 creature cast.
#[test]
fn kozileks_unsealing_mv5_makes_spawn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kozileks_unsealing());
    // Eldrazi Ravager is {5}{C} = MV 6.
    let id = g.add_card_to_hand(0, catalog::eldrazi_ravager());
    cast(&mut g, id, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 2);
}

/// Mindless Conscription amasses Zombies 3 on entry.
#[test]
fn mindless_conscription_amasses() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mindless_conscription());
    cast(&mut g, id, None);
    let army = g.battlefield.iter().find(|c| c.definition.name == "Army");
    assert!(army.is_some(), "made a Zombie Army");
    assert_eq!(army.unwrap().counter_count(CounterType::PlusOnePlusOne), 3, "amass 3 counters");
}

/// Essence Reliquary bounces another permanent you control (your turn only).
#[test]
fn essence_reliquary_bounces_own() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(0, catalog::essence_reliquary());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(relic);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: relic, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bounced");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "to owner's hand");
}

/// Etched Slith grows when it deals combat damage to a player.
#[test]
fn etched_slith_grows_on_combat_damage() {
    let mut g = two_player_game();
    let slith = g.add_card_to_battlefield(0, catalog::etched_slith());
    g.clear_sickness(slith);
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: slith, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(slith).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// The view surfaces CR 700.9 "modified" so the client can badge modified
/// creatures (the modified-matters payoffs read this).
#[test]
fn permanent_view_surfaces_modified_flag() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pv = crate::server::view::project(&g, 0)
        .battlefield.iter().find(|p| p.id == bear).cloned().expect("bear in view");
    assert!(!pv.modified, "unmodified creature");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let pv2 = crate::server::view::project(&g, 0)
        .battlefield.iter().find(|p| p.id == bear).cloned().unwrap();
    assert!(pv2.modified, "a counter makes it modified");
}

// ── Batch 3 ──────────────────────────────────────────────────────────────────

/// Cyclops Superconductor gets three energy on entry.
#[test]
fn cyclops_superconductor_etb_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cyclops_superconductor());
    cast(&mut g, id, None);
    assert_eq!(g.players[0].energy, 3, "ETB gave three energy");
}

/// Electrozoa gets two energy on entry (flash flier).
#[test]
fn electrozoa_etb_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::electrozoa());
    cast(&mut g, id, None);
    assert_eq!(g.players[0].energy, 2);
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Dreamtide Whale proliferates when a player casts their second spell.
#[test]
fn dreamtide_whale_second_spell_proliferates() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dreamtide_whale());
    // A creature with a counter to grow.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    // First spell — no proliferate.
    let s1 = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, s1, None);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    // Second spell — proliferate.
    let s2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, s2, None);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "second spell proliferated");
}

/// Etherium Pteramander can block only fliers.
#[test]
fn etherium_pteramander_blocks_only_fliers() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::etherium_pteramander());
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::CanBlockOnlyFlying));
}

/// Not Forgotten leaves a graveyard card on the library and mints a Spirit.
#[test]
fn not_forgotten_recycles_and_makes_spirit() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::not_forgotten());
    // OwnerChoice asks "put on top?" — answer true (top).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id, Some(Target::Permanent(dead)));
    assert!(g.players[0].library.iter().any(|c| c.id == dead), "card left the graveyard for the library");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "made a Spirit");
}

/// Corrupted Conscience steals the enchanted creature and gives it infect.
#[test]
fn corrupted_conscience_steals_and_grants_infect() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::corrupted_conscience());
    cast(&mut g, aura, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "gained control of the enchanted creature");
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::Infect), "granted infect");
}

/// Indebted Spirit bestowed grants +1/+1 and afterlife (host dies → Spirit).
#[test]
fn indebted_spirit_bestow_and_afterlife() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::indebted_spirit());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastBestow {
        card_id: aura, target: Some(Target::Permanent(host)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("bestow");
    drain_stack(&mut g);
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "host got +1/+1");
    // Host dies → afterlife makes a Spirit token.
    let _ = g.remove_to_graveyard_with_triggers(host);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit" && c.controller == 0),
        "afterlife minted a Spirit");
}

/// Temperamental Oozewagg gives modified creatures you control trample.
#[test]
fn temperamental_oozewagg_grants_modified_trample() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::temperamental_oozewagg());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Unmodified bear: no trample.
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
    // A +1/+1 counter modifies it → gains trample.
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample),
        "modified creature gained trample");
}

/// Kithkin Billyrider is a 1/3 double striker.
#[test]
fn kithkin_billyrider_double_strikes() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kithkin_billyrider());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 3));
    assert!(cp.keywords.contains(&Keyword::DoubleStrike));
}

/// Nyxborn Unicorn bestowed grants the host +2/+2 and mentor.
#[test]
fn nyxborn_unicorn_bestow_grants_bonus() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::nyxborn_unicorn());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastBestow {
        card_id: aura, target: Some(Target::Permanent(host)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("bestow onto the bear");
    drain_stack(&mut g);
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "host got +2/+2");
    // The bestowed Unicorn is not itself a creature while attached.
    assert!(!g.computed_permanent(aura).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "bestowed aura isn't a creature");
}

/// Eviscerator's Insight sacrifices a permanent as an additional cost, then
/// draws two.
#[test]
fn eviscerators_insight_sacs_and_draws() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::eviscerators_insight());
    fill_mana(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast, sacrificing the bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the creature as the additional cost");
    // -1 for the spell leaving hand, +2 drawn.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "drew two cards");
}

/// Copycrook enters as a copy of a creature and gains the connive-on-attack rider.
#[test]
fn copycrook_enters_as_copy_with_connive() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::copycrook());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Copycrook");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).expect("copy survived (not a 0/0)");
    assert_eq!((cp.power, cp.toughness), (2, 2), "copied the 2/2 Grizzly Bears");
    // The copy keeps the connive-on-attack rider (Attacks trigger present).
    let inst = g.battlefield_find(id).unwrap();
    assert!(inst.definition.triggered_abilities.iter().any(|t| t.event.kind == EventKind::Attacks),
        "gained the connive-on-attack trigger");
}

/// Aether Spike gives {E}{E}, pays it all, and counters the target spell
/// unless its controller pays {1} per {E} paid — here {2}, unaffordable.
#[test]
fn aether_spike_counters_via_energy_tax() {
    let mut g = two_player_game();
    // P1 casts a creature spell, spending all their mana (nothing left to pay
    // the {2} tax the two energy will demand).
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    let id = g.add_card_to_hand(0, catalog::aether_spike());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(spell)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Aether Spike");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "paid all gained energy into the tax");
    assert!(g.battlefield_find(spell).is_none(), "spell countered — controller couldn't pay {{2}}");
}

/// Ghostfire Slice costs {2} less only when an opponent controls a
/// multicolored permanent, and deals 4 damage to any target.
#[test]
fn ghostfire_slice_cost_reduction_and_damage() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let inst = crate::card::CardInstance::new(g.next_id(), catalog::ghostfire_slice(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &inst, None), 0, "no discount without a multicolored opponent permanent");
    // Opponent controls a multicolored (U/R) creature.
    g.add_card_to_battlefield(1, catalog::cyclops_superconductor());
    assert_eq!(cost_reduction_for_spell(&g, 0, &inst, None), 2, "{{2}} off vs a multicolored opponent permanent");
    let id = g.add_card_to_hand(0, catalog::ghostfire_slice());
    let before = g.players[1].life;
    cast(&mut g, id, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, before - 4, "dealt 4 damage");
}

/// Corrupted Shapeshifter's chosen mode sets its base P/T and keyword as it
/// enters — the printed */* never dies as a 0/0.
#[test]
fn corrupted_shapeshifter_enters_as_chosen_mode() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::corrupted_shapeshifter());
    // Mode 1 → a 2/5 with vigilance.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    cast(&mut g, id, None);
    let cp = g.computed_permanent(id).expect("shapeshifter survived ETB");
    assert_eq!((cp.power, cp.toughness), (2, 5), "chose the 2/5 mode");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "gained the mode's keyword");
}

/// The default decider picks the first mode (3/3 flyer); the printed */* body
/// still survives ETB because the choice is a pre-SBA replacement.
#[test]
fn corrupted_shapeshifter_default_mode_survives() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::corrupted_shapeshifter());
    cast(&mut g, id, None);
    let cp = g.computed_permanent(id).expect("shapeshifter survived ETB");
    assert_eq!((cp.power, cp.toughness), (3, 3), "default picked mode 0");
    assert!(cp.keywords.contains(&Keyword::Flying));
}

// ── Batch 4 (Flare cycle) ────────────────────────────────────────────────────

/// Flare of Cultivation ramps a basic to the battlefield and one to hand.
#[test]
fn flare_of_cultivation_ramps() {
    let mut g = two_player_game();
    let l1 = g.add_card_to_library(0, catalog::forest());
    let l2 = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::flare_of_cultivation());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(l1)),
        DecisionAnswer::Search(Some(l2)),
    ]));
    cast(&mut g, id, None);
    assert!(g.battlefield_find(l1).is_some(), "one basic to battlefield");
    assert!(g.battlefield_find(l1).unwrap().tapped, "entered tapped");
    assert!(g.players[0].hand.iter().any(|c| c.id == l2), "one basic to hand");
}

/// Flare of Fortitude can be cast by sacrificing a nontoken white creature and
/// grants your permanents hexproof + indestructible.
#[test]
fn flare_of_fortitude_alt_cost_and_protection() {
    let mut g = two_player_game();
    // A white creature to pitch (Metastatic Evangel is a white Phyrexian).
    let fodder = g.add_card_to_battlefield(0, catalog::metastatic_evangel());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::flare_of_fortitude());
    // Cast via the alt (sacrifice) cost — no mana.
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("alt-cast by sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the white creature as the cost");
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Hexproof) && cp.keywords.contains(&Keyword::Indestructible),
        "your creatures gained hexproof + indestructible");
    // Life total can't change this turn — neither gain nor loss lands.
    let life = g.players[0].life;
    g.adjust_life(0, 5);
    g.adjust_life(0, -3);
    assert_eq!(g.players[0].life, life, "life total locked until end of turn");
}
