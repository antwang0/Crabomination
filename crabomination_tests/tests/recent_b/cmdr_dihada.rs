//! Commander: the Legends' Legacy precon (DMC, Dihada, Binder of Wills,
//! `decks::cmdr_dihada`). The primitives it needed are tested in
//! `core_rules/commander_cards.rs`.

use crabomination::card::{CardDefinition, CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
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
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    if let Some(c) = g.battlefield_find_mut(id) {
        c.loyalty_uses_this_turn = 0;
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn connect(g: &mut GameState, attacker: CardId, at: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(at) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// CR 606 — Dihada's +2 protects a legend; −3 keeps the legends and makes a
/// Treasure per card binned.
#[test]
fn dihada_protects_and_digs_for_legends() {
    let mut g = pod(2);
    let dihada = g.add_card_to_battlefield(0, catalog::dihada_binder_of_wills());
    loyalty(&mut g, dihada, 0, None).expect("+2 with no legend: \"up to one\"");
    let arvad = g.add_card_to_battlefield(0, catalog::arvad_the_cursed());
    loyalty(&mut g, dihada, 0, Some(Target::Permanent(arvad))).expect("+2");
    assert!(g.permanent_has_keyword(arvad, &Keyword::Indestructible));
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let legend = g.add_card_to_library(0, catalog::moira_urborg_haunt());
    g.add_card_to_library(0, catalog::island());
    loyalty(&mut g, dihada, 1, None).expect("−3");
    assert!(g.players[0].hand.iter().any(|c| c.id == legend));
    assert_eq!(named(&g, 0, "Treasure").len(), 3, "three Islands binned");
}

/// CR 613.4c — Arvad pumps the other legends.
#[test]
fn arvad_pumps_other_legends() {
    let mut g = pod(2);
    let arvad = g.add_card_to_battlefield(0, catalog::arvad_the_cursed());
    let moira = g.add_card_to_battlefield(0, catalog::moira_urborg_haunt());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, moira), (5, 4));
    assert_eq!(pt(&g, bear), (2, 2));
    assert_eq!(pt(&g, arvad), (3, 3));
}

/// CR 603.4 — Ashling's third resolution in a turn cashes the counters in.
#[test]
fn ashling_explodes_on_the_third_resolution() {
    let mut g = pod(2);
    let ashling = g.add_card_to_battlefield(0, catalog::ashling_the_pilgrim());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    activate(&mut g, ashling, 0, None).expect("one");
    activate(&mut g, ashling, 0, None).expect("two");
    assert_eq!(pt(&g, ashling), (3, 3));
    activate(&mut g, ashling, 0, None).expect("three");
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(ashling).is_none(), "its counters gone, three damage kills it");
}

/// CR 406 — Bell Borca's power is the greatest mana value exiled this turn;
/// its upkeep exile feeds the note.
#[test]
fn bell_borca_grows_with_exiled_mana_value() {
    let mut g = pod(2);
    let bell = g.add_card_to_battlefield(0, catalog::bell_borca_spectral_sergeant());
    assert_eq!(pt(&g, bell), (0, 5));
    let top = g.add_card_to_library(0, catalog::solemn_simulacrum());
    step(&mut g, TurnStep::Upkeep);
    let card = g.exile.iter().find(|c| c.id == top).expect("exiled");
    assert!(card.may_play_until.is_some());
    assert_eq!(pt(&g, bell), (4, 5));
}

/// CR 510.3 — Bladewing's connection makes a Zombie Knight per creature card
/// in the graveyard.
#[test]
fn bladewing_raises_zombie_knights() {
    let mut g = pod(2);
    let dragon = ready(&mut g, 0, catalog::bladewing_deathless_tyrant());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::island());
    connect(&mut g, dragon, 1);
    assert_eq!(named(&g, 0, "Zombie Knight").len(), 2);
}

/// CR 704.5j — Cadric's token copy of a legend survives the legend rule and
/// is sacrificed at the end step.
#[test]
fn cadric_copies_legends_past_the_legend_rule() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::cadric_soul_kindler());
    let moira = g.add_card_to_hand(0, catalog::moira_urborg_haunt());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    cast(&mut g, moira, None).expect("cast");
    assert_eq!(named(&g, 0, "Moira, Urborg Haunt").len(), 2, "the card and its token");
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Moira, Urborg Haunt"), vec![moira]);
}

/// CR 701.21 — Lannery Storm makes a Treasure attacking and grows when one
/// is sacrificed.
#[test]
fn lannery_storm_loots_treasure() {
    let mut g = pod(2);
    let storm = ready(&mut g, 0, catalog::captain_lannery_storm());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: storm, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let treasure = named(&g, 0, "Treasure")[0];
    g.perform_action(GameAction::ActivateAbility {
        card_id: treasure,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice for mana");
    drain_stack(&mut g);
    assert_eq!(pt(&g, storm).0, 3);
}

/// CR 400.7 — Garna returns creature cards put into the graveyard this turn.
#[test]
fn garna_recovers_this_turns_dead() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let old = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("kill it");
    let garna = g.add_card_to_hand(0, catalog::garna_the_bloodflame());
    cast(&mut g, garna, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old), "died on an earlier turn");
}

/// CR 400.7 — Gerrard's Hourglass Pendant returns this turn's dead
/// permanents tapped.
#[test]
fn gerrards_hourglass_pendant_rewinds_the_turn() {
    let mut g = pod(2);
    let pendant = g.add_card_to_battlefield(0, catalog::gerrards_hourglass_pendant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("kill it");
    activate(&mut g, pendant, 0, None).expect("rewind");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert!(g.exile.iter().any(|c| c.id == pendant));
    assert!(g.extra_turn_denied_for(0) || g.battlefield_find(pendant).is_none());
}

/// CR 700.4 — Kothophed draws for each opponent-owned permanent going to the
/// graveyard.
#[test]
fn kothophed_hoards_souls() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kothophed_soul_hoarder());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    for target in [bear, mine] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, Some(Target::Permanent(target))).expect("kill it");
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "only the opponent's Bear");
    assert_eq!(g.players[0].life, life - 1);
}

/// CR 510.3 — Moira returns a creature that died this turn.
#[test]
fn moira_returns_the_fallen() {
    let mut g = pod(2);
    let moira = ready(&mut g, 0, catalog::moira_urborg_haunt());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("kill it");
    connect(&mut g, moira, 1);
    assert!(g.battlefield_find(bear).is_some());
}

/// CR 205.4d — Primevals' Glorious Rebirth needs a legend to cast, then
/// returns every legendary permanent card.
#[test]
fn primevals_glorious_rebirth_needs_a_legend() {
    let mut g = pod(2);
    let moira = g.add_card_to_graveyard(0, catalog::moira_urborg_haunt());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::primevals_glorious_rebirth());
    assert!(cast(&mut g, spell, None).is_err(), "no legendary creature");
    g.add_card_to_battlefield(0, catalog::arvad_the_cursed());
    cast(&mut g, spell, None).expect("now legal");
    assert!(g.battlefield_find(moira).is_some());
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 603.2 — Shanid draws on legendary spells and gives legends menace.
#[test]
fn shanid_rewards_legends() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::shanid_sleepers_scourge());
    g.add_card_to_library(0, catalog::island());
    let moira = g.add_card_to_hand(0, catalog::moira_urborg_haunt());
    let hand = g.players[0].hand.len();
    cast(&mut g, moira, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert!(g.permanent_has_keyword(moira, &Keyword::Menace));
}

/// CR 702.41a — The Circle of Loyalty costs less per Knight and makes Knights.
#[test]
fn the_circle_of_loyalty_knights() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::arvad_the_cursed());
    let circle = g.add_card_to_hand(0, catalog::the_circle_of_loyalty());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: circle,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("one Knight: {3}{W}{W}");
    drain_stack(&mut g);
    let moira = g.add_card_to_hand(0, catalog::moira_urborg_haunt());
    cast(&mut g, moira, None).expect("a legendary spell");
    let knights = named(&g, 0, "Knight");
    assert_eq!(knights.len(), 1);
    assert_eq!(pt(&g, knights[0]), (3, 3));
}

/// CR 707.10 — The Peregrine Dynamo copies another legend's ability.
#[test]
fn the_peregrine_dynamo_copies_a_legends_ability() {
    let mut g = pod(2);
    let dynamo = ready(&mut g, 0, catalog::the_peregrine_dynamo());
    let ashling = g.add_card_to_battlefield(0, catalog::ashling_the_pilgrim());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ashling,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("pump");
    activate(&mut g, dynamo, 0, Some(Target::Permanent(ashling))).expect("copy");
    assert_eq!(g.battlefield_find(ashling).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// CR 205.3 — Tyrite Sanctum makes a legend a God, then an indestructible God.
#[test]
fn tyrite_sanctum_deifies_a_legend() {
    let mut g = pod(2);
    let sanctum = g.add_card_to_battlefield(0, catalog::tyrite_sanctum());
    let arvad = g.add_card_to_battlefield(0, catalog::arvad_the_cursed());
    activate(&mut g, sanctum, 1, Some(Target::Permanent(arvad))).expect("a God");
    let c = g.computed_permanent(arvad).unwrap();
    assert!(c.subtypes().creature_types.contains(&CreatureType::God));
    assert_eq!(g.battlefield_find(arvad).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.battlefield_find_mut(sanctum).unwrap().tapped = false;
    activate(&mut g, sanctum, 2, Some(Target::Permanent(arvad))).expect("indestructible");
    assert_eq!(g.battlefield_find(arvad).unwrap().counter_count(CounterType::Indestructible), 1);
}

/// CR 707.10 — Verrak copies an ability life was paid for.
#[test]
fn verrak_copies_life_paid_abilities() {
    use crabomination::card::ActivatedAbility;
    use crabomination::effect::{Effect, Selector, Value};
    let pay_draw = CardDefinition {
        name: "Test Life Draw",
        card_types: vec![crabomination::card::CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::verrak_warped_sengir());
    let src = g.add_card_to_battlefield(0, pay_draw);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    activate(&mut g, src, 0, None).expect("pay 2");
    assert_eq!(g.players[0].hand.len(), hand + 2, "the ability and its copy");
    assert_eq!(g.players[0].life, life - 4);
}

/// CR 510.3 — Zeriam makes a Griffin for each Griffin that connects.
#[test]
fn zeriam_breeds_griffins() {
    let mut g = pod(2);
    let zeriam = ready(&mut g, 0, catalog::zeriam_golden_wind());
    connect(&mut g, zeriam, 1);
    assert_eq!(named(&g, 0, "Griffin").len(), 1);
}
