//! Commander: the 2026-10-18 (`015XaGkQ`) one-deck closers
//! (`decks::cmdr_015x_a`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 10);
    }
}

fn activate(g: &mut GameState, card_id: CardId, ability_index: usize, x_value: Option<u32>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index,
        target: None,
        additional_targets: vec![],
        x_value,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn cast(g: &mut GameState, card_id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// April O'Neil draws one card per card type among the spells cast this
/// turn: an instant and an artifact are two, however many spells.
#[test]
fn april_oneil_draws_per_card_type_cast() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::april_oneil_hacktivist());
    flood(&mut g);
    for def in [catalog::lightning_bolt(), catalog::shock()] {
        let id = g.add_card_to_hand(0, def);
        cast(&mut g, id, Some(Target::Player(1)));
    }
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    let hand = g.players[0].hand.len();
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Bruce Banner draws X for {X}{X} and transforms into the Hulk, whose
/// enrage while attacking untaps him and banks another combat (CR 500.8).
#[test]
fn bruce_banner_draws_and_hulks_out() {
    let mut g = pod(2);
    let bruce = ready(&mut g, 0, catalog::bruce_banner());
    flood(&mut g);
    activate(&mut g, bruce, 0, Some(2));
    assert_eq!(g.players[0].hand.len(), 2);
    activate(&mut g, bruce, 1, None);
    assert_eq!(g.battlefield_find(bruce).unwrap().definition.name, "The Incredible Hulk");
    g.battlefield_find_mut(bruce).unwrap().tapped = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bruce, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bruce).unwrap().tapped);
    let bolt = Effect::DealDamage { to: Selector::ExactObjects(vec![bruce]), amount: Value::Const(3) };
    let evs = g.resolve_effect(&bolt, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let hulk = g.battlefield_find(bruce).unwrap();
    assert!(!hulk.tapped, "untapped");
    assert_eq!(hulk.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.additional_combat_phases, 1);
}

/// CR 702.188 — Amazing Spider-Man gives a colored legendary spell
/// web-slinging {G}{W}{U}: it is cast for that, returning a tapped creature.
#[test]
fn amazing_spider_man_grants_web_slinging_to_legendary_spells() {
    let mut g = pod(2);
    let peter = ready(&mut g, 0, catalog::peter_parker());
    flood(&mut g);
    activate(&mut g, peter, 0, None);
    assert_eq!(g.battlefield_find(peter).unwrap().definition.name, "Amazing Spider-Man");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.players[0].mana_pool = Default::default();
    for c in [Color::Green, Color::White, Color::Blue] {
        g.players[0].mana_pool.add(c, 1);
    }
    let legend = g.add_card_to_hand(0, catalog::isamaru_hound_of_konda());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: legend,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("web-slinging");
    drain_stack(&mut g);
    assert!(g.battlefield_find(legend).is_some());
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the tapped creature went home");
}

/// Peter Parker's entry makes a 2/1 reach Spider.
#[test]
fn peter_parker_enters_with_a_spider() {
    let mut g = pod(2);
    flood(&mut g);
    let peter = g.add_card_to_hand(0, catalog::peter_parker());
    cast(&mut g, peter, None);
    let spider = g.battlefield.iter().find(|c| c.definition.name == "Spider").expect("a Spider");
    assert_eq!((spider.definition.power, spider.definition.toughness), (2, 1));
}

/// CR 508.1d — Public Enemy on seat 1's creature: seat 2's creatures must
/// attack seat 1 if able — not seat 0, and not stay home. The enchanted
/// creature dying draws the Aura's controller a card.
#[test]
fn public_enemy_makes_everyone_attack_its_hosts_controller() {
    let mut g = pod(3);
    let host = ready(&mut g, 1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::public_enemy());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(host);
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    g.active_player_idx = 2;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 2;
    let declare = |g: &mut GameState, to: Option<usize>| {
        let attacks = to.map(|p| Attack { attacker: bear, target: AttackTarget::Player(p) }).into_iter().collect();
        g.clone().perform_action(GameAction::DeclareAttackers(attacks)).is_ok()
    };
    assert!(!declare(&mut g, None), "it must attack");
    assert!(!declare(&mut g, Some(0)), "and it must attack the host's controller");
    assert!(declare(&mut g, Some(1)));
    let hand = g.players[0].hand.len();
    let kill = Effect::Destroy { what: Selector::ExactObjects(vec![host]) };
    let evs = g.resolve_effect(&kill, &EffectContext::for_spell(2, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Maraxus of Keld counts your untapped artifacts, creatures and lands —
/// itself included — and a tap shrinks it (CR 604.3).
#[test]
fn maraxus_of_keld_counts_untapped_artifacts_creatures_and_lands() {
    let mut g = pod(2);
    let maraxus = ready(&mut g, 0, catalog::maraxus_of_keld());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(1, catalog::forest());
    let pt = |g: &GameState| g.computed_permanent(maraxus).map(|cp| (cp.power, cp.toughness));
    assert_eq!(pt(&g), Some((3, 3)));
    g.battlefield_find_mut(land).unwrap().tapped = true;
    assert_eq!(pt(&g), Some((2, 2)));
}

/// Codie copies a prepared spell (Peer Review: two Cadets, not one) and
/// re-prepares an unprepared Fateseer for five colors and a tap.
#[test]
fn codie_copies_prepared_spells_and_prepares_the_team() {
    let mut g = pod(2);
    let codie = ready(&mut g, 0, catalog::codie_ravenous_codex());
    let seer = ready(&mut g, 0, catalog::prudent_fateseer());
    g.battlefield_find_mut(seer).unwrap().add_counters(CounterType::Prepared, 1);
    flood(&mut g);
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: seer,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Peer Review");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Cadet").count(), 2);
    assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Prepared), 0);
    activate(&mut g, codie, 0, None);
    assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Prepared), 1);
    // An ordinary spell is no prepared spell.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17, "one Bolt, no copy");
}

/// Prowl exiles a tapped creature as it attacks; that card's owner plays it
/// from exile, and Prowl's controller draws while Prowl converts.
#[test]
fn prowl_exiles_a_tapped_creature_and_converts_when_it_is_played() {
    let mut g = pod(2);
    let prowl = ready(&mut g, 0, catalog::prowl_stoic_strategist());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: prowl, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "the tapped bear is exiled");
    g.step = TurnStep::PostCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("its owner casts it from exile");
    assert_eq!(g.players[1].mana_pool.total(), 0, "paying its own cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.battlefield_find(prowl).unwrap().definition.name, "Prowl, Pursuit Vehicle");
}
