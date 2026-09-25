//! Commander: The Ruinous Powers precon (40K, Abaddon the Despoiler,
//! `decks::cmdr_abaddon`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..seats {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn run(g: &mut GameState, effect: Effect, source: CardId) {
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(source);
    let ev = g.resolve_effect(&effect, &ctx).expect("effect");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn in_zone<'a>(zone: impl IntoIterator<Item = &'a crabomination::card::CardInstance>, id: CardId) -> bool {
    zone.into_iter().any(|c| c.id == id)
}

/// CR 615.1 / CR 800.4 — Khârn the Betrayer: damage to it is prevented and
/// an opponent gains control of it; losing control of it draws its old
/// controller two cards (CR 603.6d "when you lose control").
#[test]
fn cr_615_kharn_changes_hands_instead_of_taking_damage() {
    let mut g = main_phase(4);
    let kharn = g.add_card_to_battlefield(0, catalog::kharn_the_betrayer());
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_as(&mut g, 0, bolt, Some(Target::Permanent(kharn))).expect("bolt");
    let k = g.battlefield_find(kharn).expect("Khârn survives: the damage was prevented");
    assert_eq!(k.controller, 1, "the next opponent in turn order takes it");
    assert_eq!(k.damage, 0);
    assert_eq!(g.players[0].hand.len(), hand + 2, "its old controller draws two");
}

/// CR 508.1d — Seeker of Slaanesh: its controller's opponent can't declare
/// no attackers while a creature of theirs is able to attack.
#[test]
fn cr_508_1d_seeker_of_slaanesh_forces_an_attack() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::seeker_of_slaanesh());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![])).is_err(), "no attack is illegal");
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(0),
    }]))
    .expect("one attacker satisfies it");
}

/// CR 508.1d — Seeker of Slaanesh doesn't bind its own controller.
#[test]
fn cr_508_1d_seeker_of_slaanesh_leaves_its_controller_free() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::seeker_of_slaanesh());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![])).expect("its controller may stay home");
}

/// CR 608.2c — Chaos Defiler: one nonland permanent is chosen per opponent,
/// and one of them, at random, is destroyed.
#[test]
fn cr_608_2c_chaos_defiler_destroys_one_pick_at_random() {
    let mut g = main_phase(4);
    let bears: Vec<CardId> = (1..4).map(|s| g.add_card_to_battlefield(s, catalog::grizzly_bears())).collect();
    let defiler = g.add_card_to_hand(0, catalog::chaos_defiler());
    cast_as(&mut g, 0, defiler, None).expect("Chaos Defiler");
    let gone = bears.iter().filter(|b| g.battlefield_find(**b).is_none()).count();
    assert_eq!(gone, 1, "exactly one of the three picks is destroyed");
}

/// CR 603.7 — Lucius the Eternal: dying exiles it and watches an opponent's
/// creature; when that creature leaves the battlefield, Lucius returns.
#[test]
fn cr_603_7_lucius_returns_when_the_watched_creature_leaves() {
    let mut g = main_phase(2);
    let lucius = g.add_card_to_battlefield(0, catalog::lucius_the_eternal());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![lucius]) }, lucius);
    assert!(in_zone(g.exile.iter(), lucius), "dying exiles Lucius");
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![bear]) }, bear);
    let back = g.battlefield_find(lucius).expect("Lucius returns");
    assert_eq!(back.controller, 0, "under its owner's control");
}

/// CR 702.138 — Helbrute's Sarcophagus: cast from the graveyard by exiling
/// another *creature* card; a land doesn't pay it.
#[test]
fn cr_702_138_helbrute_escapes_only_by_exiling_a_creature_card() {
    let mut g = main_phase(2);
    let helbrute = g.add_card_to_graveyard(0, catalog::helbrute());
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let escape = |exile: CardId| GameAction::CastEscape {
        card_id: helbrute,
        exile_cards: vec![exile],
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    flood(&mut g, 0);
    assert!(g.perform_action(escape(land)).is_err(), "a land card doesn't pay Sarcophagus");
    g.perform_action(escape(bear)).expect("a creature card does");
    drain_stack(&mut g);
    assert!(g.battlefield_find(helbrute).is_some());
    assert!(in_zone(g.exile.iter(), bear));
}

/// CR 119.3 — Great Unclean One: each opponent loses 2 at your end step, then
/// a Plaguebearer of Nurgle per opponent with less life than you.
#[test]
fn cr_119_3_great_unclean_one_drains_and_rewards_the_lead() {
    let mut g = main_phase(4);
    g.add_card_to_battlefield(0, catalog::great_unclean_one());
    let start = g.players[0].life;
    g.players[3].life = start + 10;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!((1..3).all(|s| g.players[s].life == start - 2));
    assert_eq!(g.players[3].life, start + 8);
    assert_eq!(named(&g, 0, "Plaguebearer of Nurgle").len(), 2, "one per opponent behind you");
}

/// CR 702.85a — Abaddon: on your turn, a spell cast from hand with mana value
/// at most the life your opponents lost this turn cascades.
#[test]
fn cr_702_85a_abaddon_grants_cascade_once_opponents_bleed() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::abaddon_the_despoiler());
    g.players[0].library.clear();
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_as(&mut g, 0, bears, None).expect("bears");
    assert!(in_zone(g.players[0].library.iter(), top), "no life lost yet: no cascade");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_as(&mut g, 0, bolt, Some(Target::Player(1))).expect("bolt");
    let more = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_as(&mut g, 0, more, None).expect("bears");
    assert!(!in_zone(g.players[0].library.iter(), top), "the second Bears cascaded into the Bolt");
}

/// CR 603.4 — Tallyman of Nurgle: after one death, your end step draws one
/// and loses 1.
#[test]
fn cr_603_4_tallyman_counts_the_dead() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::tallyman_of_nurgle());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![bear]) }, bear);
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, life - 1);
}

/// CR 601.2 — The Ruinous Powers: the exiled card of an opponent is cast
/// with any mana, and its owner loses life equal to its mana value.
#[test]
fn cr_601_2_the_ruinous_powers_casts_their_card_against_them() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::the_ruinous_powers());
    g.players[1].library.clear();
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(in_zone(g.exile.iter(), bolt), "the top card of their library is exiled");
    g.step = TurnStep::PreCombatMain;
    let life = g.players[1].life;
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable with colorless mana");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3 - 1, "Bolt's 3 plus its mana value");
}

/// CR 305.4 — The Lost and the Damned: a land put onto the battlefield (not
/// played) makes a Spawn; a land played from hand doesn't.
#[test]
fn cr_305_4_the_lost_and_the_damned_rewards_lands_from_elsewhere() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::the_lost_and_the_damned());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land drop");
    drain_stack(&mut g);
    assert!(named(&g, 0, "Spawn").is_empty());
    let other = g.add_card_to_graveyard(0, catalog::forest());
    run(
        &mut g,
        Effect::Move {
            what: Selector::ExactObjects(vec![other]),
            to: crabomination::effect::ZoneDest::Battlefield {
                controller: crabomination::effect::PlayerRef::You,
                tapped: false,
            },
        },
        other,
    );
    assert_eq!(named(&g, 0, "Spawn").len(), 1);
}

/// CR 122.1 — Venomcrawler grows with each other creature's death.
#[test]
fn cr_122_1_venomcrawler_grows_on_deaths() {
    let mut g = main_phase(2);
    let crawler = g.add_card_to_battlefield(0, catalog::venomcrawler());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![bear]) }, bear);
    assert_eq!(g.battlefield_find(crawler).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}
