//! Commander: the Arm for Battle precon (CMR, Wyleth, `decks::cmdr_wyleth`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn declare(g: &mut GameState, attacker: CardId, defender: usize) -> Result<(), String> {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(defender),
    }]))
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0)
}

fn attach(g: &mut GameState, what: CardId, host: CardId) {
    g.battlefield_find_mut(what).unwrap().attached_to = Some(host);
}

/// +1/+0 per opponent (two at a three-seat table), and damage dealt to the
/// bearer is dealt again to any target.
#[test]
fn blazing_sunsteel_scales_with_opponents_and_returns_damage() {
    let mut g = main_phase(3);
    let force = g.add_card_to_battlefield(0, catalog::celestial_force());
    let s = g.add_card_to_battlefield(0, catalog::blazing_sunsteel());
    attach(&mut g, s, force);
    assert_eq!(pt(&g, force), (9, 7));
    let lives = [g.players[1].life, g.players[2].life];
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(force)]);
    let lost = (lives[0] - g.players[1].life) + (lives[1] - g.players[2].life);
    assert_eq!(lost, 3, "the three damage comes back out at an opponent");
}

#[test]
fn dawn_charm_regenerates_a_creature() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::dawn_charm());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: c, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("regenerate mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().regeneration_shields > 0);
}

/// O-ring on an Aura: the stolen creature comes back when the Aura leaves.
#[test]
fn faith_unbroken_exiles_until_it_leaves() {
    let mut g = main_phase(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::celestial_force());
    let f = g.add_card_to_hand(0, catalog::faith_unbroken());
    cast(&mut g, f, &[Target::Permanent(mine)]);
    assert_eq!(pt(&g, mine), (4, 4));
    assert!(g.battlefield_find(theirs).is_none(), "exiled");
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, kill, &[Target::Permanent(mine)]);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Celestial Force" && c.controller == 1));
}

#[test]
fn haunted_cloak_grants_three_keywords() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::haunted_cloak());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: c, target: bear }).expect("equip");
    drain_stack(&mut g);
    for k in [Keyword::Vigilance, Keyword::Trample, Keyword::Haste] {
        assert!(g.permanent_has_keyword(bear, &k));
    }
}

/// A legendary creature entering may take the Blade for free.
#[test]
fn heros_blade_jumps_to_an_entering_legend() {
    let mut g = main_phase(2);
    let blade = g.add_card_to_battlefield(0, catalog::heros_blade());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let odric = g.add_card_to_hand(0, catalog::odric_lunarch_marshal());
    cast(&mut g, odric, &[]);
    assert_eq!(g.battlefield_find(blade).unwrap().attached_to, Some(odric));
    assert_eq!(pt(&g, odric), (6, 5));
}

#[test]
fn ironclad_slayer_returns_an_equipment_card() {
    let mut g = main_phase(2);
    let cloak = g.add_card_to_graveyard(0, catalog::haunted_cloak());
    let s = g.add_card_to_hand(0, catalog::ironclad_slayer());
    cast(&mut g, s, &[]);
    assert!(g.players[0].hand.iter().any(|c| c.id == cloak));
}

/// CR 205.4e — legendary sorcery: needs a legend, then X to each target.
#[test]
fn jayas_inferno_needs_a_legend_and_hits_three_targets() {
    let mut g = main_phase(3);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let j = g.add_card_to_hand(0, catalog::jayas_immolating_inferno());
    let cast_it = |g: &mut GameState| {
        flood(g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: j,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Player(1), Target::Player(2)],
            mode: None,
            x_value: Some(2),
        })
    };
    assert!(cast_it(&mut g).is_err(), "no legendary creature");
    g.add_card_to_battlefield(0, catalog::odric_lunarch_marshal());
    let lives = [g.players[1].life, g.players[2].life];
    cast_it(&mut g).expect("with Odric out");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none());
    assert_eq!([g.players[1].life, g.players[2].life], [lives[0] - 2, lives[1] - 2]);
}

#[test]
fn memorial_to_war_trades_itself_for_a_land() {
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::memorial_to_war());
    let land = g.add_card_to_battlefield(1, catalog::swamp());
    activate(&mut g, m, 1, Some(Target::Permanent(land))).expect("activate");
    assert!(g.battlefield_find(land).is_none());
    assert!(g.battlefield_find(m).is_none());
}

/// At the beginning of combat, a keyword any of your creatures has spreads.
#[test]
fn odric_shares_keywords_at_the_beginning_of_combat() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::odric_lunarch_marshal());
    g.add_card_to_battlefield(0, catalog::celestial_force());
    let flier = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.permanent_has_keyword(flier, &Keyword::Flying));
    while g.step != TurnStep::DeclareAttackers {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying));
    assert!(g.permanent_has_keyword(bear, &Keyword::Vigilance));
}

/// CR 613.1d — the enchanted creature becomes legendary (so Jaya's is live).
#[test]
fn on_serras_wings_makes_its_host_legendary() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::on_serras_wings());
    cast(&mut g, w, &[Target::Permanent(bear)]);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.supertypes().contains(&crabomination::card::Supertype::Legendary));
    assert_eq!(pt(&g, bear), (3, 3));
    for k in [Keyword::Flying, Keyword::Vigilance, Keyword::Lifelink] {
        assert!(g.permanent_has_keyword(bear, &k));
    }
}

fn connect(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// CR 702.111 — renown, then the Equipment search.
#[test]
fn relic_seeker_finds_equipment_when_it_becomes_renowned() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::haunted_cloak());
    let s = g.add_card_to_battlefield(0, catalog::relic_seeker());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    connect(&mut g, s, 1);
    assert_eq!(counters(&g, s), 1);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Haunted Cloak"));
}

#[test]
fn response_hits_an_attacker_and_resurgence_adds_a_combat() {
    let mut g = main_phase(2);
    let r = g.add_card_to_hand(0, catalog::response_resurgence());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSplitRight {
        card_id: r, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Resurgence");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::FirstStrike));
    assert!(g.permanent_has_keyword(bear, &Keyword::Vigilance));
    assert_eq!(g.additional_post_main_combats, 1, "a combat after this main phase");
}

/// Response: 5 to an attacking creature.
#[test]
fn response_deals_five_to_an_attacker() {
    let mut g = main_phase(2);
    g.active_player_idx = 1;
    let force = g.add_card_to_battlefield(1, catalog::celestial_force());
    g.clear_sickness(force);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: force, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    let r = g.add_card_to_hand(0, catalog::response_resurgence());
    cast(&mut g, r, &[Target::Permanent(force)]);
    assert_eq!(g.battlefield_find(force).unwrap().damage, 5);
}

fn to_draw_step(g: &mut GameState) {
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// A white bearer grows each upkeep; a green one does not.
#[test]
fn ring_of_thune_grows_a_white_bearer_only() {
    let mut g = main_phase(2);
    let white = g.add_card_to_battlefield(0, catalog::relic_seeker());
    let ring = g.add_card_to_battlefield(0, catalog::ring_of_thune());
    attach(&mut g, ring, white);
    to_draw_step(&mut g);
    assert_eq!(counters(&g, white), 1);
    assert!(g.permanent_has_keyword(white, &Keyword::Vigilance));
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attach(&mut g, ring, green);
    to_draw_step(&mut g);
    assert_eq!(counters(&g, green), 0);
}

#[test]
fn ring_of_valkas_grows_a_red_bearer() {
    let mut g = main_phase(2);
    let red = g.add_card_to_battlefield(0, catalog::goblin_guide());
    let ring = g.add_card_to_battlefield(0, catalog::ring_of_valkas());
    attach(&mut g, ring, red);
    to_draw_step(&mut g);
    assert_eq!(counters(&g, red), 1);
}

/// An Equipment put into the graveyard comes back at the next end step.
#[test]
fn tiana_returns_a_dead_equipment_at_the_end_step() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::tiana_ships_caretaker());
    let cloak = g.add_card_to_battlefield(0, catalog::haunted_cloak());
    let shatter = g.add_card_to_hand(0, catalog::shatter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, shatter, &[Target::Permanent(cloak)]);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cloak));
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == cloak));
}

#[test]
fn timely_ward_grants_indestructible() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::timely_ward());
    cast(&mut g, t, &[Target::Permanent(bear)]);
    assert!(g.permanent_has_keyword(bear, &Keyword::Indestructible));
}

/// Copy the opponent's Bolt; the copy may take new targets.
#[test]
fn wild_ricochet_copies_an_instant() {
    let mut g = main_phase(2);
    g.active_player_idx = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    let life = [g.players[0].life, g.players[1].life];
    let w = g.add_card_to_hand(0, catalog::wild_ricochet());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: w, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("ricochet");
    drain_stack(&mut g);
    let dealt = (life[0] - g.players[0].life) + (life[1] - g.players[1].life);
    assert_eq!(dealt, 6, "the bolt and its copy both resolve");
}
