//! Commander: the Abzan Armor precon (TDC, Felothar the Steadfast,
//! `decks::cmdr_felothar`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
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

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn run(g: &mut GameState, effect: Effect, source: CardId) {
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(source);
    let events = g.resolve_effect(&effect, &ctx).expect("effect");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Felothar — a defender attacks and hits for its toughness; the sacrifice
/// draws toughness and discards power.
#[test]
fn felothar_turns_walls_into_attackers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::felothar_the_steadfast());
    let tree = g.add_card_to_battlefield(0, catalog::tree_of_redemption());
    attack(&mut g, &[tree]);
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 7, "13 toughness in damage");
    // The sacrifice: Indomitable Ancients (2/10) draws ten, discards two.
    let mut g = pod(2);
    let f = g.add_card_to_battlefield(0, catalog::felothar_the_steadfast());
    g.add_card_to_battlefield(0, catalog::indomitable_ancients());
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    g.clear_sickness(f);
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, f, 0, None).expect("felothar");
    assert_eq!(g.players[0].hand.len(), hand + 8, "drew ten, discarded two");
}

/// Assault Formation — damage by toughness, and +0/+1 for {2}{G}.
#[test]
fn assault_formation_toughness_damage() {
    let mut g = pod(2);
    let af = g.add_card_to_battlefield(0, catalog::assault_formation());
    let ancients = g.add_card_to_battlefield(0, catalog::indomitable_ancients());
    activate(&mut g, 0, af, 1, None).expect("pump");
    assert_eq!(pt(&g, ancients), (2, 11));
    attack(&mut g, &[ancients]);
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 9);
}

/// Baldin — on your turn everything hits by toughness; attacking adds your
/// hand size to toughness.
#[test]
fn baldin_hits_by_toughness() {
    let mut g = pod(2);
    let b = g.add_card_to_battlefield(0, catalog::baldin_century_herdmaster());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    attack(&mut g, &[b]);
    assert_eq!(pt(&g, b), (0, 9));
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 11);
}

/// Behind the Scenes — skulk for your team.
#[test]
fn behind_the_scenes_grants_skulk() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::behind_the_scenes());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Skulk));
}

/// Betor — the end step turns life gained into counters and reanimates by
/// life lost.
#[test]
fn betor_grows_and_reanimates() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::betor_ancestors_voice());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    g.players[0].life_gained_this_turn = 3;
    g.players[0].life_lost_this_turn = 4;
    step(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(g.battlefield_find(giant).is_some(), "mana value 4 ≤ 4 life lost");
}

/// Blight Pile — drains per defender.
#[test]
fn blight_pile_drains_per_defender() {
    let mut g = pod(2);
    let bp = g.add_card_to_battlefield(0, catalog::blight_pile());
    g.add_card_to_battlefield(0, catalog::tree_of_redemption());
    g.clear_sickness(bp);
    activate(&mut g, 0, bp, 0, None).expect("drain");
    assert_eq!(g.players[1].life, 18);
}

/// Canopy Gargantuan — each other creature gets counters equal to its
/// toughness at your upkeep.
#[test]
fn canopy_gargantuan_doubles_toughness_as_counters() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::canopy_gargantuan());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(pt(&g, bears), (4, 4));
}

/// Colfenor's Urn — three big deaths come back at the end step.
#[test]
fn colfenors_urn_returns_three() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::colfenors_urn());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let mut ids = vec![];
    for _ in 0..3 {
        let a = g.add_card_to_battlefield(0, catalog::indomitable_ancients());
        run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![a]) }, a);
        ids.push(a);
    }
    assert!(ids.iter().all(|id| g.exile.iter().any(|c| c.id == *id)));
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Indomitable Ancients").len(), 3);
}

/// Dragonlord Dromoka — opponents can't cast on your turn.
#[test]
fn dromoka_silences_your_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::dragonlord_dromoka());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    assert!(cast(&mut g, 1, bolt, Some(Target::Player(0))).is_err());
}

/// Indulging Patrician — 3 life gained drains each opponent 3.
#[test]
fn indulging_patrician_drains() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::indulging_patrician());
    g.players[0].life_gained_this_turn = 3;
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, 17);
}

/// Jaws of Defeat — the gap between power and toughness drains.
#[test]
fn jaws_of_defeat_drains_the_gap() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::jaws_of_defeat());
    let a = g.add_card_to_hand(0, catalog::indomitable_ancients());
    cast(&mut g, 0, a, None).expect("ancients");
    assert_eq!(g.players[1].life, 12);
}

/// Protector of the Wastes — exiles an opposing artifact on entering.
#[test]
fn protector_of_the_wastes_exiles() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let p = g.add_card_to_hand(0, catalog::protector_of_the_wastes());
    cast(&mut g, 0, p, None).expect("protector");
    assert!(g.battlefield_find(ring).is_none());
}

/// Rampart Architect — Walls on entering and attacking; a dead Wall ramps.
#[test]
fn rampart_architect_builds_walls() {
    let mut g = pod(2);
    let ra = g.add_card_to_hand(0, catalog::rampart_architect());
    cast(&mut g, 0, ra, None).expect("architect");
    let walls = named(&g, 0, "Wall");
    assert_eq!(walls.len(), 1);
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![walls[0]]) }, walls[0]);
    assert_eq!(named(&g, 0, "Forest").len(), 1);
}

/// Reunion of the House — up to 10 power of creatures back.
#[test]
fn reunion_of_the_house_returns_ten_power() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::hill_giant());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    let r = g.add_card_to_hand(0, catalog::reunion_of_the_house());
    cast(&mut g, 0, r, None).expect("reunion");
    assert_eq!(named(&g, 0, "Hill Giant").len(), 3, "9 power fits, 12 doesn't");
    assert!(g.exile.iter().any(|c| c.id == r));
}

/// Slaughter the Strong — each player keeps 4 power's worth.
#[test]
fn slaughter_the_strong_keeps_four_power() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let wall = g.add_card_to_battlefield(1, catalog::tree_of_redemption());
    let s = g.add_card_to_hand(0, catalog::slaughter_the_strong());
    cast(&mut g, 0, s, None).expect("slaughter");
    assert_eq!(named(&g, 1, "Grizzly Bears").len(), 2);
    assert!(g.battlefield_find(wall).is_some(), "0 power always fits");
}

/// Tip the Scales — a big sacrifice shrinks everything.
#[test]
fn tip_the_scales_shrinks_everything() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::hill_giant());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::tip_the_scales());
    cast(&mut g, 0, t, None).expect("tip");
    assert!(g.battlefield_find(bears).is_none());
}

/// Towering Titan — enters with your creatures' total toughness as counters.
#[test]
fn towering_titan_sizes_by_toughness() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::indomitable_ancients());
    let tt = g.add_card_to_hand(0, catalog::towering_titan());
    cast(&mut g, 0, tt, None).expect("titan");
    assert_eq!(pt(&g, tt), (10, 10));
}

/// Tree of Redemption — swaps your life with its toughness.
#[test]
fn tree_of_redemption_swaps() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::tree_of_redemption());
    g.clear_sickness(t);
    g.players[0].life = 5;
    activate(&mut g, 0, t, 0, None).expect("swap");
    assert_eq!(g.players[0].life, 13);
    assert_eq!(pt(&g, t), (0, 5));
}

/// Walking Bulwark — a defender attacks for its toughness. Regression (CR
/// 508.1a): a granted `AttacksAsThoughNoDefender` was ignored by the
/// declare-attackers check.
#[test]
fn walking_bulwark_unleashes_a_wall() {
    let mut g = pod(2);
    let wb = g.add_card_to_battlefield(0, catalog::walking_bulwark());
    let tree = g.add_card_to_battlefield(0, catalog::tree_of_redemption());
    activate(&mut g, 0, wb, 0, Some(Target::Permanent(tree))).expect("bulwark");
    attack(&mut g, &[tree]);
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 7);
}

/// Will of the Abzan — each opponent sacrifices their biggest and loses 3.
#[test]
fn will_of_the_abzan_punishes() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::will_of_the_abzan());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: w, target: None, additional_targets: vec![], mode: Some(0), x_value: None })
        .expect("will");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert!(g.battlefield_find(bears).is_some());
    assert_eq!(g.players[1].life, 17);
}

/// Arbor Adherent — X mana from the greatest toughness.
#[test]
fn arbor_adherent_taps_for_toughness() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::arbor_adherent());
    g.add_card_to_battlefield(0, catalog::indomitable_ancients());
    g.clear_sickness(a);
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: a,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana");
    assert_eq!(g.players[0].mana_pool.total(), 10);
}
