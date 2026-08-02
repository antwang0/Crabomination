//! Invasion (INV) gap wave 5 — the pile-splitting rares and the last utility
//! shell.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
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
    cast_multi(g, seat, id, target, vec![]);
}

fn cast_multi(
    g: &mut GameState,
    seat: usize,
    id: CardId,
    target: Option<Target>,
    additional_targets: Vec<Target>,
) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets,
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// A scripted decider that picks `pile` as the first pile and answers the
/// chooser's yes/no with `take_first`.
fn split_decider(pile: Vec<CardId>, take_first: bool) -> Box<ScriptedDecider> {
    Box::new(ScriptedDecider::new([
        DecisionAnswer::Cards(pile),
        DecisionAnswer::Bool(take_first),
    ]))
}

// ── The pile-splitting cycle ────────────────────────────────────────────────

/// Do or Die destroys exactly the pile the target player picked.
#[test]
fn do_or_die_destroys_the_chosen_pile() {
    let mut g = main_phase();
    let doomed = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spared = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spite = g.add_card_to_hand(0, catalog::do_or_die());
    g.decider = split_decider(vec![doomed], true);
    cast(&mut g, 0, spite, Some(Target::Player(1)));
    assert!(g.battlefield.iter().all(|c| c.id != doomed), "chosen pile died");
    assert!(g.battlefield.iter().any(|c| c.id == spared), "the other pile lived");
}

/// Death or Glory exiles the pile the opponent picked and reanimates the rest.
#[test]
fn death_or_glory_splits_your_graveyard() {
    let mut g = main_phase();
    let exiled = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let raised = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::death_or_glory());
    g.decider = split_decider(vec![exiled], true);
    cast(&mut g, 0, spell, None);
    assert!(g.exile.iter().any(|c| c.id == exiled), "chosen pile exiled");
    assert!(g.battlefield.iter().any(|c| c.id == raised), "other pile returned");
}

/// Bend or Break destroys the chosen pile and taps the rest.
#[test]
fn bend_or_break_destroys_one_pile_and_taps_the_other() {
    let mut g = main_phase();
    let doomed = g.add_card_to_battlefield(0, catalog::forest());
    let tapped = g.add_card_to_battlefield(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::bend_or_break());
    // Seat 0 splits {doomed}; seat 1 (their opponent) takes that pile. Seat 1
    // controls no lands, so its own split is a no-op.
    g.decider = split_decider(vec![doomed], true);
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield.iter().all(|c| c.id != doomed), "chosen pile destroyed");
    assert!(
        g.battlefield.iter().any(|c| c.id == tapped && c.tapped),
        "the other pile is tapped"
    );
}

/// Fight or Flight bars the pile the attacker didn't pick.
#[test]
fn fight_or_flight_locks_the_unchosen_pile_out_of_attacking() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::fight_or_flight());
    let free = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let barred = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(free);
    g.clear_sickness(barred);
    g.active_player_idx = 1;
    g.decider = split_decider(vec![free], true);
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let has_lock = |g: &GameState, id| {
        g.computed_permanent(id).unwrap().keywords.contains(&Keyword::CantAttack)
    };
    assert!(has_lock(&g, barred), "unchosen pile can't attack");
    assert!(!has_lock(&g, free), "chosen pile still attacks");
}

/// Barrin's Spite sacrifices the picked creature and bounces the other.
#[test]
fn barrins_spite_sacrifices_one_and_bounces_the_other() {
    let mut g = main_phase();
    let sacked = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bounced = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::barrins_spite());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![sacked])]));
    cast_multi(
        &mut g,
        0,
        spell,
        Some(Target::Permanent(sacked)),
        vec![Target::Permanent(bounced)],
    );
    assert!(g.players[1].graveyard.iter().any(|c| c.id == sacked), "one was sacrificed");
    assert!(g.players[1].hand.iter().any(|c| c.id == bounced), "the other bounced");
}

// ── Utility ─────────────────────────────────────────────────────────────────

/// Coalition Victory only wins with all five basic types and all five colours.
#[test]
fn coalition_victory_needs_the_full_board() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::coalition_victory());
    g.add_card_to_battlefield(0, catalog::forest());
    cast(&mut g, 0, spell, None);
    assert!(!g.is_game_over(), "an incomplete board wins nothing");
}

/// Artifact Mutation trades the artifact for its mana value in Saprolings.
#[test]
fn artifact_mutation_mints_a_saproling_per_mana_value() {
    let mut g = main_phase();
    let rock = g.add_card_to_battlefield(1, catalog::worn_powerstone());
    let mv = g.battlefield_find(rock).unwrap().definition.cost.cmc();
    let spell = g.add_card_to_hand(0, catalog::artifact_mutation());
    cast(&mut g, 0, spell, Some(Target::Permanent(rock)));
    assert!(g.battlefield.iter().all(|c| c.id != rock), "the artifact died");
    let saps = g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count();
    assert_eq!(saps as u32, mv, "one Saproling per mana value");
}

/// Global Ruin leaves one land of each basic type standing.
#[test]
fn global_ruin_keeps_one_of_each_basic_type() {
    let mut g = main_phase();
    let keep = g.add_card_to_battlefield(0, catalog::forest());
    let extra = g.add_card_to_battlefield(0, catalog::forest());
    let island = g.add_card_to_battlefield(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::global_ruin());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield.iter().any(|c| c.id == keep), "a Forest survives");
    assert!(g.battlefield.iter().any(|c| c.id == island), "the Island survives");
    assert!(g.battlefield.iter().all(|c| c.id != extra), "the spare Forest is gone");
}

/// Desperate Research keeps the named copies and exiles the rest.
#[test]
fn desperate_research_takes_only_the_named_cards() {
    let mut g = main_phase();
    let wanted = g.add_card_to_library(0, catalog::grizzly_bears());
    let junk = g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::desperate_research());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard(
        "Grizzly Bears".into(),
    )]));
    cast(&mut g, 0, spell, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == wanted), "named card taken");
    assert!(g.exile.iter().any(|c| c.id == junk), "the rest is exiled");
}

/// Spreading Plague wipes every other creature sharing the newcomer's colour.
#[test]
fn spreading_plague_kills_the_newcomers_color() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::spreading_plague());
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let arrival = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, arrival, None);
    assert!(g.battlefield.iter().all(|c| c.id != green), "the old green creature died");
    assert!(g.battlefield.iter().any(|c| c.id == arrival), "the newcomer survived");
}

/// Temporal Distortion stamps an hourglass counter on anything that taps.
#[test]
fn temporal_distortion_stamps_tapped_permanents() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::temporal_distortion());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(forest).unwrap().counter_count(CounterType::Hourglass),
        1,
        "tapping added an hourglass counter"
    );
}

/// Pure Reflection sizes its token to the creature spell that made it.
#[test]
fn pure_reflection_mints_a_token_sized_to_the_spell() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::pure_reflection());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let mv = g.find_card_anywhere(bear).unwrap().definition.cost.cmc() as i32;
    cast(&mut g, 0, bear, None);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Reflection")
        .expect("a Reflection arrived");
    assert_eq!((token.definition.power, token.definition.toughness), (mv, mv));
}

/// Cauldron Dance is legal in combat and illegal in a main phase.
#[test]
fn cauldron_dance_is_combat_only() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::cauldron_dance());
    mana(&mut g, 0);
    let gy = g.players[0].graveyard[0].id;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(gy)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "rejected outside combat"
    );
    g.step = TurnStep::DeclareBlockers;
    cast(&mut g, 0, spell, Some(Target::Permanent(gy)));
    assert!(g.battlefield.iter().any(|c| c.id == gy), "reanimated in combat");
}

/// Vile Consumption asks each creature's controller for a life payment.
#[test]
fn vile_consumption_taxes_every_creature_at_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::vile_consumption());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = g.players[0].life;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let paid = g.players[0].life < before;
    let sacked = g.battlefield.iter().all(|c| c.id != bear);
    assert!(paid || sacked, "the upkeep tax either took life or the creature");
}

/// Yawgmoth's Agenda routes its controller's cards to exile.
#[test]
fn yawgmoths_agenda_exiles_your_dying_cards() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::yawgmoths_agenda());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.destroy_permanent(bear, false, &mut events);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "the dead creature was exiled");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bear));
}

/// Overabundance pings the player who tapped the land.
#[test]
fn overabundance_pings_the_land_tapper() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overabundance());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let before = g.players[1].life;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "the tapper took a point");
}

// ── The colour-matters shell ────────────────────────────────────────────────

/// Well-Laid Plans stops a creature hurting a creature of a shared colour.
#[test]
fn well_laid_plans_blanks_same_color_creature_damage() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::well_laid_plans());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(b),
        2,
        Some(a),
        &mut events,
    );
    assert_eq!(g.battlefield_find(b).unwrap().damage, 0, "shared-colour damage prevented");
}

/// Harsh Judgment sends the chosen colour's burn back at its caster.
#[test]
fn harsh_judgment_redirects_the_chosen_colors_burn() {
    let mut g = main_phase();
    let ward = g.add_card_to_battlefield(1, catalog::harsh_judgment());
    g.battlefield_find_mut(ward).unwrap().chosen_color = Some(Color::Red);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let (me, them) = (g.players[0].life, g.players[1].life);
    cast(&mut g, 0, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, them, "the enchantment's controller is untouched");
    assert_eq!(g.players[0].life, me - 3, "the caster took it instead");
}

/// Pulse of Llanowar overrides what your basics tap for.
#[test]
fn pulse_of_llanowar_rewrites_your_basics() {
    let mut g = main_phase();
    let pulse = g.add_card_to_battlefield(0, catalog::pulse_of_llanowar());
    g.battlefield_find_mut(pulse).unwrap().chosen_color = Some(Color::Blue);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "the Forest made blue");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0);
}

/// Mana Maze locks out a spell sharing the last cast's colour.
#[test]
fn mana_maze_blocks_the_last_casts_color() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mana_maze());
    let first = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, first, Some(Target::Player(1)));
    let second = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: second,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a second red spell is locked out"
    );
}

/// Traveler's Cloak hands out landwalk of the type it named.
#[test]
fn travelers_cloak_grants_the_chosen_landwalk() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let cloak = g.add_card_to_hand(0, catalog::travelers_cloak());
    cast(&mut g, 0, cloak, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Landwalk(_))),
        "the enchanted creature has landwalk"
    );
    assert_eq!(g.players[0].hand.len(), 1, "the Aura cantripped");
}

/// Mages' Contest counters the spell when nobody tops the opening bid.
#[test]
fn mages_contest_counters_when_the_bid_stands() {
    let mut g = main_phase();
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
    .expect("bolt on the stack");
    let contest = g.add_card_to_hand(0, catalog::mages_contest());
    let before = g.players[0].life;
    // Seat 1 passes on topping the opening bid of 1.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(0)]));
    cast(&mut g, 0, contest, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, before - 1, "the winner paid their bid");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "the Bolt was countered");
}

/// Pain // Suffering ships both halves.
#[test]
fn pain_suffering_has_a_right_half() {
    let def = catalog::pain_suffering();
    let split = def.split.as_ref().expect("split card");
    assert_eq!(split.right.cost.cmc(), 4, "Suffering costs four");
    assert!(!split.fuse && !split.aftermath);
}
