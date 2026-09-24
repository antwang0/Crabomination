//! Commander: the Eldrazi Incursion precon (M3C, Ulalek, `decks::cmdr_azlask`).
//! The primitives it needed are tested in `core_rules/commander_cards.rs`.

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod() -> GameState {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for s in 0..3 {
        for _ in 0..5 {
            g.add_card_to_library(s, catalog::wastes());
        }
    }
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
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

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// CR 122.1 — Azlask: a colorless creature dying gives experience; WUBRG
/// pumps the team by it and arms Spawns.
#[test]
fn azlask_banks_experience() {
    let mut g = pod();
    let azlask = g.add_card_to_battlefield(0, catalog::azlask_the_swelling_scourge());
    let spawn_src = g.add_card_to_hand(0, catalog::skittering_invasion());
    cast(&mut g, spawn_src, None).expect("five Spawn");
    assert_eq!(count_named(&g, 0, "Eldrazi Spawn"), 5);
    let spawn = g.battlefield.iter().find(|c| c.definition.name == "Eldrazi Spawn").unwrap().id;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(spawn))).expect("bolt a Spawn");
    assert_eq!(g.players[0].experience, 1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, azlask, 0, None);
    assert_eq!(pt(&g, bear), (3, 3));
    let other = g.battlefield.iter().find(|c| c.definition.name == "Eldrazi Spawn").unwrap().id;
    assert!(g.permanent_has_keyword(other, &Keyword::Indestructible));
}

/// CR 701.21 — Angelic Aberration trades 1-power creatures for 4/4 Angels.
#[test]
fn angelic_aberration_upgrades_spawn() {
    let mut g = pod();
    for _ in 0..2 {
        g.add_token_to_battlefield(0, &crabomination_base::tokens::eldrazi_spawn_token());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(3)]));
    let aa = g.add_card_to_hand(0, catalog::angelic_aberration());
    cast(&mut g, aa, None).expect("cast");
    assert!(g.battlefield_find(bear).is_some(), "a 2/2 isn't eligible");
    assert_eq!(count_named(&g, 0, "Eldrazi Angel"), 2);
}

/// CR 119.4 — Bismuth Mindrender's hit lets you cast the player's next
/// nonland card for life.
#[test]
fn bismuth_mindrender_steals_a_spell() {
    let mut g = pod();
    let mind = g.add_card_to_battlefield(0, catalog::bismuth_mindrender());
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.clear_sickness(mind);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: mind, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    let bolt = g.exile.iter().find(|c| c.definition.name == "Lightning Bolt").expect("exiled");
    assert!(bolt.may_play_until.is_some_and(|m| m.pay_life && m.player == 0));
}

/// CR 603.6c — Chittering Dispatcher leaves a Spawn behind.
#[test]
fn chittering_dispatcher_leaves_a_spawn() {
    let mut g = pod();
    let cd = g.add_card_to_battlefield(0, catalog::chittering_dispatcher());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(cd))).expect("bolt");
    assert_eq!(count_named(&g, 0, "Eldrazi Spawn"), 1);
}

/// CR 702.86 — Eldrazi Conscription: +10/+10, trample, annihilator 2.
#[test]
fn eldrazi_conscription_arms_a_creature() {
    let mut g = pod();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ec = g.add_card_to_hand(0, catalog::eldrazi_conscription());
    cast(&mut g, ec, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(pt(&g, bear), (12, 12));
    assert!(g.permanent_has_keyword(bear, &Keyword::Annihilator(2)));
}

/// CR 702.96 — Eldritch Immunity protects one creature, or all overloaded.
#[test]
fn eldritch_immunity_protects() {
    let mut g = pod();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ei = g.add_card_to_hand(0, catalog::eldritch_immunity());
    cast(&mut g, ei, Some(Target::Permanent(a))).expect("cast");
    assert!(g.permanent_has_keyword(a, &Keyword::Protection(Color::Red)));
    assert!(!g.permanent_has_keyword(b, &Keyword::Protection(Color::Red)));
    let ei = g.add_card_to_hand(0, catalog::eldritch_immunity());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: ei,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("overload");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(b, &Keyword::Protection(Color::Black)));
}

/// CR 613.1b — Hideous Taskmaster borrows one creature per opponent.
#[test]
fn hideous_taskmaster_borrows() {
    let mut g = pod();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().tapped = true;
    let ht = g.add_card_to_hand(0, catalog::hideous_taskmaster());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: ht,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in [a, b] {
        let c = g.battlefield_find(id).unwrap();
        assert_eq!(c.controller, 0);
        assert!(!c.tapped);
    }
    assert!(g.permanent_has_keyword(a, &Keyword::Annihilator(1)));
}

/// CR 613.4d — Inversion Behemoth switches the targets' power and toughness.
#[test]
fn inversion_behemoth_switches() {
    let mut g = pod();
    g.add_card_to_battlefield(0, catalog::inversion_behemoth());
    let wall = g.add_card_to_battlefield(1, catalog::spawnbed_protector());
    step(&mut g, TurnStep::BeginCombat);
    // Whichever targets the bot picks, a picked creature's P/T are swapped.
    let (p, t) = pt(&g, wall);
    assert!((p, t) == (6, 8) || (p, t) == (8, 6));
}

/// CR 601.2f — Morophon: spells of the chosen type cost WUBRG less; other
/// creatures of that type get +1/+1.
#[test]
fn morophon_discounts_its_type() {
    let mut g = pod();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(CreatureType::Bear)]));
    let m = g.add_card_to_hand(0, catalog::morophon_the_boundless());
    cast(&mut g, m, None).expect("cast");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bear), (3, 3));
    let bear2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bear2, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{G} less {G}");
}

/// Spawnbed Protector returns an Eldrazi and makes two Scions each end step;
/// Tomb of the Spirit Dragon gains a life per colorless creature.
#[test]
fn spawnbed_and_tomb() {
    let mut g = pod();
    g.add_card_to_battlefield(0, catalog::spawnbed_protector());
    let dead = g.add_card_to_graveyard(0, catalog::bismuth_mindrender());
    step(&mut g, TurnStep::End);
    assert_eq!(count_named(&g, 0, "Eldrazi Scion"), 2);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead));
    let tomb = g.add_card_to_battlefield(0, catalog::tomb_of_the_spirit_dragon());
    let life = g.players[0].life;
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, tomb, 1, None);
    assert_eq!(g.players[0].life, life + 3, "the Protector and two Scions");
}

/// CR 509.1b — Twins of Discord: attacking stops one parity from blocking.
#[test]
fn twins_of_discord_stops_blockers() {
    let mut g = pod();
    let twins = g.add_card_to_battlefield(0, catalog::twins_of_discord());
    let even = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(twins);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(1)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: twins, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(even, twins)])).is_err(),
        "a two-drop can't block after 'even'"
    );
}

/// CR 707.10 — Ulalek copies every spell you control for {C}{C}.
#[test]
fn ulalek_copies_everything() {
    let mut g = pod();
    g.add_card_to_battlefield(0, catalog::ulalek_fused_atrocity());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ei = g.add_card_to_hand(0, catalog::eldritch_immunity());
    g.perform_action(GameAction::CastSpell {
        card_id: ei,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("a Kindred Eldrazi instant");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 6, "the Bolt and its copy");
}

/// CR 702.21a — Ulamog's Dreadsire's ward demands a sacrifice; {T} makes a
/// 10/10.
#[test]
fn ulamogs_dreadsire_makes_titans() {
    let mut g = pod();
    let d = g.add_card_to_battlefield(0, catalog::ulamogs_dreadsire());
    g.clear_sickness(d);
    activate(&mut g, d, 0, None);
    let tok = g.battlefield.iter().find(|c| c.is_token).unwrap().id;
    assert_eq!(pt(&g, tok), (10, 10));
}

/// CR 702.32b — Wastescape Battlemage's two kickers.
#[test]
fn wastescape_battlemage_kicks_twice() {
    let mut g = pod();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let wb = g.add_card_to_hand(0, catalog::wastescape_battlemage());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellKickers {
        card_id: wb,
        kickers: vec![0, 1],
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("both kickers");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == ring));
    assert!(g.players[2].hand.iter().any(|c| c.id == bear));
    let _ = CounterType::PlusOnePlusOne;
}
