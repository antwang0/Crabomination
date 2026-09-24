//! Commander: the Invent Superiority precon (C16, Breya,
//! `decks::cmdr_breya`) and the primitives it needed.

use crabomination::card::{
    CardDefinition, CardId, CardType, CounterType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    Subtypes, TriggeredAbility, Value,
};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::mana::Color;
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::TurnStep;
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// CR 800.4a — "when enchanted player loses the game": an Aura on the
/// departing player triggers as they leave, reading its counters; an Aura on
/// a player still in the game doesn't.
#[test]
fn cr_800_4a_an_aura_on_a_departing_player_triggers_as_they_leave() {
    let curse = || CardDefinition {
        name: "Test Curse",
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EnchantedPlayerLeftGame, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..Default::default()
    };
    let mut g = pod(3);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let on_one = g.add_card_to_battlefield(0, curse());
    let on_two = g.add_card_to_battlefield(0, curse());
    g.battlefield_find_mut(on_one).unwrap().attached_to_player = Some(1);
    g.battlefield_find_mut(on_one).unwrap().counters.insert(CounterType::Spite, 2);
    g.battlefield_find_mut(on_two).unwrap().attached_to_player = Some(2);
    g.battlefield_find_mut(on_two).unwrap().counters.insert(CounterType::Spite, 3);
    let hand = g.players[0].hand.len();
    g.players[1].life = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[1].eliminated);
    assert_eq!(g.players[0].hand.len(), hand + 2, "only the departed player's curse, at its 2 counters");
}

// ── The cards ──

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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, mode: Option<usize>) -> Result<(), String> {
    let seat = g.battlefield_find(id).expect("source").controller;
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Cast under a may-play permission (a graveyard card here).
fn cast_granted(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

fn yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
}

fn attack(g: &mut GameState, a: CardId) {
    g.clear_sickness(a);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
}

fn connect(g: &mut GameState, a: CardId) {
    attack(g, a);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Two Thopters on entry; sacrificing two artifacts pays for a mode.
#[test]
fn breya_makes_thopters_and_spends_them() {
    let mut g = pod(2);
    let breya = g.add_card_to_hand(0, catalog::breya_etherium_shaper());
    cast(&mut g, 0, breya, None).expect("cast");
    assert_eq!(named(&g, 0, "Thopter").len(), 2);
    let breya = named(&g, 0, "Breya, Etherium Shaper")[0];
    let life = g.players[1].life;
    activate(&mut g, breya, 0, Some(Target::Player(1)), Some(0)).expect("3 damage");
    assert_eq!(g.players[1].life, life - 3);
    assert!(named(&g, 0, "Thopter").is_empty(), "both Thopters paid");
    assert!(activate(&mut g, breya, 0, None, Some(2)).is_err(), "one artifact (Breya) can't pay two");
}

/// Draw a hand's worth, then discard as many.
#[test]
fn ancient_excavation_cycles_the_hand() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let ex = g.add_card_to_hand(0, catalog::ancient_excavation());
    cast(&mut g, 0, ex, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), 2, "drew two, discarded two");
    assert_eq!(g.players[0].graveyard.len(), 3, "the two discards and the spell");
}

/// Entering, it takes on your Equipment.
#[test]
fn armory_automaton_suits_up() {
    let mut g = pod(2);
    let boots = g.add_card_to_battlefield(0, catalog::swiftfoot_boots());
    let bracers = g.add_card_to_battlefield(0, catalog::battlemages_bracers());
    yes(&mut g);
    let a = g.add_card_to_hand(0, catalog::armory_automaton());
    cast(&mut g, 0, a, None).expect("cast");
    let a = named(&g, 0, "Armory Automaton")[0];
    for e in [boots, bracers] {
        assert_eq!(g.battlefield_find(e).unwrap().attached_to, Some(a));
    }
}

/// Entering, a creature of yours gains double strike and lifelink.
#[test]
fn bruse_tarl_arms_a_creature() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bruse = g.add_card_to_hand(0, catalog::bruse_tarl_boorish_herder());
    cast(&mut g, 0, bruse, None).expect("cast");
    let kws: Vec<_> = [bear, named(&g, 0, "Bruse Tarl, Boorish Herder")[0]]
        .iter()
        .filter(|&&id| {
            let k = g.computed_permanent(id).unwrap().keywords().to_vec();
            k.contains(&Keyword::DoubleStrike) && k.contains(&Keyword::Lifelink)
        })
        .copied()
        .collect();
    assert_eq!(kws.len(), 1, "one creature, both keywords");
}

/// The cursed player's spells add spite; their loss pays it out.
#[test]
fn curse_of_vengeance_collects_on_the_loss() {
    let mut g = pod(3);
    library(&mut g, 0, 5);
    let curse = g.add_card_to_hand(0, catalog::curse_of_vengeance());
    cast(&mut g, 0, curse, Some(Target::Player(1))).expect("cast");
    let curse = named(&g, 0, "Curse of Vengeance")[0];
    g.active_player_idx = 1;
    for _ in 0..2 {
        let t = g.add_card_to_hand(1, catalog::ornithopter());
        cast(&mut g, 1, t, None).expect("cast");
    }
    g.active_player_idx = 2;
    let t = g.add_card_to_hand(2, catalog::ornithopter());
    cast(&mut g, 2, t, None).expect("another player's spell");
    assert_eq!(g.battlefield_find(curse).unwrap().counter_count(CounterType::Spite), 2);
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    g.players[1].life = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[0].hand.len()), (life + 2, hand + 2));
}

/// {1}{W}{B}, {T}: destroy; {2}{U}: untap, and destroy again.
#[test]
fn ethersworn_adjudicator_destroys_twice() {
    let mut g = pod(2);
    let adj = g.add_card_to_battlefield(0, catalog::ethersworn_adjudicator());
    g.clear_sickness(adj);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::goblin_oriflamme());
    activate(&mut g, adj, 0, Some(Target::Permanent(a)), None).expect("destroy");
    activate(&mut g, adj, 1, None, None).expect("untap");
    activate(&mut g, adj, 0, Some(Target::Permanent(b)), None).expect("destroy the enchantment");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// An opponent's nontoken creature entering: an artifact copy for you; the
/// next one replaces it.
#[test]
fn faerie_artisans_keeps_the_latest_copy() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::faerie_artisans());
    g.active_player_idx = 1;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    cast(&mut g, 1, bear, None).expect("cast");
    let copy = named(&g, 0, "Grizzly Bears");
    assert_eq!(copy.len(), 1);
    assert!(g.battlefield_find(copy[0]).unwrap().definition.card_types.contains(&CardType::Artifact));
    let angel = g.add_card_to_hand(1, catalog::serra_angel());
    cast(&mut g, 1, angel, None).expect("cast");
    assert!(named(&g, 0, "Grizzly Bears").is_empty(), "the older copy is exiled");
    assert_eq!(named(&g, 0, "Serra Angel").len(), 1);
}

/// 3 life per artifact you control, itself included.
#[test]
fn filigree_angel_counts_artifacts() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mind_stone());
    let life = g.players[0].life;
    let angel = g.add_card_to_hand(0, catalog::filigree_angel());
    cast(&mut g, 0, angel, None).expect("cast");
    assert_eq!(g.players[0].life, life + 6);
}

/// A creature card from any graveyard joins you with haste.
#[test]
fn grave_upheaval_raises_anyones_dead() {
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(1, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::grave_upheaval());
    cast(&mut g, 0, s, Some(Target::Permanent(dead))).expect("cast");
    let a = named(&g, 0, "Serra Angel");
    assert_eq!(a.len(), 1);
    assert!(g.computed_permanent(a[0]).unwrap().keywords().contains(&Keyword::Haste));
}

/// Steal an Equipment onto a fresh Germ.
#[test]
fn grip_of_phyresis_steals_equipment() {
    let mut g = pod(2);
    // Swiftfoot Boots would leave the 0/0 Germ to die; this one gives +2/+2.
    let boots = g.add_card_to_battlefield(1, catalog::vulshok_morningstar());
    let s = g.add_card_to_hand(0, catalog::grip_of_phyresis());
    cast(&mut g, 0, s, Some(Target::Permanent(boots))).expect("cast");
    let germ = named(&g, 0, "Phyrexian Germ");
    assert_eq!(germ.len(), 1);
    let b = g.battlefield_find(boots).unwrap();
    assert_eq!((b.controller, b.attached_to), (0, Some(germ[0])));
}

/// Exiled: this turn a graveyard spell can be cast, and it's exiled after.
#[test]
fn magus_of_the_will_opens_the_graveyard() {
    let mut g = pod(2);
    let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_will());
    g.clear_sickness(magus);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    activate(&mut g, magus, 0, None, None).expect("activate");
    assert!(g.battlefield_find(magus).is_none(), "exiled as a cost");
    cast_granted(&mut g, 0, bolt, Some(Target::Player(1))).expect("cast from the graveyard");
    assert!(g.exile.iter().any(|c| c.id == bolt), "exiled instead of the graveyard");
}

/// Four flying Birds.
#[test]
fn migratory_route_releases_birds() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::migratory_route());
    cast(&mut g, 0, s, None).expect("cast");
    assert_eq!(named(&g, 0, "Bird").len(), 4);
}

/// Destroy; draw and lose a life per counter it had.
#[test]
fn parting_thoughts_reads_the_counters() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    let s = g.add_card_to_hand(0, catalog::parting_thoughts());
    cast(&mut g, 0, s, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!((g.players[0].life, g.players[0].hand.len()), (life - 2, hand + 2));
}

/// Entering, an artifact card returns from your graveyard.
#[test]
fn sharuum_returns_an_artifact() {
    let mut g = pod(2);
    let stone = g.add_card_to_graveyard(0, catalog::mind_stone());
    yes(&mut g);
    let s = g.add_card_to_hand(0, catalog::sharuum_the_hegemon());
    cast(&mut g, 0, s, Some(Target::Permanent(stone))).expect("cast");
    assert_eq!(named(&g, 0, "Mind Stone").len(), 1);
}

/// Connecting lets you cast an artifact card from your graveyard this turn.
#[test]
fn silas_renn_recasts_an_artifact() {
    let mut g = pod(2);
    let silas = g.add_card_to_battlefield(0, catalog::silas_renn_seeker_adept());
    let stone = g.add_card_to_graveyard(0, catalog::mind_stone());
    connect(&mut g, silas);
    g.step = TurnStep::PostCombatMain;
    cast_granted(&mut g, 0, stone, None).expect("cast from the graveyard");
    assert_eq!(named(&g, 0, "Mind Stone").len(), 1);
}

/// Entering, it may tutor an artifact creature.
#[test]
fn sphinx_summoner_finds_an_artifact_creature() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    g.add_card_to_library(0, catalog::ornithopter());
    yes(&mut g);
    let s = g.add_card_to_hand(0, catalog::sphinx_summoner());
    cast(&mut g, 0, s, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Ornithopter"));
}

/// {U}: a noncreature artifact becomes a creature sized by its mana value;
/// {W}{B}: an artifact creature gains deathtouch and lifelink.
#[test]
fn sydri_animates_artifacts() {
    let mut g = pod(2);
    let sydri = g.add_card_to_battlefield(0, catalog::sydri_galvanic_genius());
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    activate(&mut g, sydri, 0, Some(Target::Permanent(stone)), None).expect("animate");
    let cp = g.computed_permanent(stone).unwrap();
    assert!(cp.card_types().contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (2, 2));
    activate(&mut g, sydri, 1, Some(Target::Permanent(stone)), None).expect("grant");
    let kws = g.computed_permanent(stone).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Deathtouch) && kws.contains(&Keyword::Lifelink));
}
