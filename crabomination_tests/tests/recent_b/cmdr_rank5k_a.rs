//! Commander: one-card gaps of EDHREC ranks 4501-5500 (`decks::cmdr_rank5k_a`).

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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x });
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn keywords(g: &GameState, id: CardId) -> Vec<Keyword> {
    g.computed_permanent(id).unwrap().keywords().to_vec()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Noncombat damage from `source` to player `to`, dispatched like a resolved ability.
fn ping(g: &mut GameState, source: CardId, to: usize) {
    let mut ctx = EffectContext::for_spell(0, Some(Target::Player(to)), 0, 0);
    ctx.source = Some(source);
    let fx = Effect::DealDamage { to: Selector::Target(0), amount: crabomination::card::Value::ONE };
    let evs = g.resolve_effect(&fx, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn attack_unblocked(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(defender),
    }]))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

/// CR 702.11 / CR 510.2 — Ruric Thar, Magecrusher keeps hexproof through
/// noncombat damage it deals and loses it once it deals combat damage
/// (`Predicate::SourceHasDealtCombatDamage`).
#[test]
fn ruric_thar_magecrusher_loses_hexproof_only_to_combat_damage() {
    let mut g = pod(2);
    let ruric = ready(&mut g, 0, catalog::ruric_thar_magecrusher());
    assert!(keywords(&g, ruric).contains(&Keyword::Hexproof));
    ping(&mut g, ruric, 1);
    assert!(keywords(&g, ruric).contains(&Keyword::Hexproof), "noncombat damage keeps it");
    let life = g.players[1].life;
    attack_unblocked(&mut g, ruric, 1);
    assert_eq!(g.players[1].life, life - 7);
    assert!(!keywords(&g, ruric).contains(&Keyword::Hexproof), "combat damage ends it");
}

/// Zhang Liao: any damage to an opponent — here noncombat — makes that
/// opponent discard, and only that opponent.
#[test]
fn zhang_liao_makes_the_damaged_opponent_discard() {
    let mut g = pod(3);
    let zhang = ready(&mut g, 0, catalog::zhang_liao_hero_of_hefei());
    for seat in 1..3 {
        g.add_card_to_hand(seat, catalog::island());
    }
    ping(&mut g, zhang, 2);
    assert_eq!(g.players[2].hand.len(), 0);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Inundate: every nonblue creature goes to its owner's hand, either side.
#[test]
fn inundate_bounces_only_nonblue_creatures() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let blue = ready(&mut g, 1, catalog::wind_drake());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inundate());
    cast(&mut g, id, None, None);
    assert!(g.battlefield_find(blue).is_some());
    assert!(g.battlefield_find(bear).is_none() && g.players[1].hand.iter().any(|c| c.id == bear));
    assert!(g.players[0].hand.iter().any(|c| c.id == mine));
}

/// Patrician's Scorn: the free cast needs *another* white spell first.
#[test]
fn patricians_scorn_is_free_after_a_white_spell() {
    let mut g = pod(2);
    let ench = ready(&mut g, 1, catalog::luminous_bonds());
    let scorn = g.add_card_to_hand(0, catalog::patricians_scorn());
    let free = |id| GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(g.perform_action(free(scorn)).is_err(), "no white spell cast yet");
    let soldier = g.add_card_to_hand(0, catalog::winter_soldier_bucky_barnes());
    cast(&mut g, soldier, None, None);
    g.players[0].mana_pool = Default::default();
    g.perform_action(free(scorn)).expect("free after a white spell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none());
}

/// Awakened Awareness: X counters on a creature whose base is now 1/1.
#[test]
fn awakened_awareness_makes_a_one_one_with_x_counters() {
    let mut g = pod(2);
    let ogre = ready(&mut g, 0, catalog::ruric_thar_magecrusher());
    let id = g.add_card_to_hand(0, catalog::awakened_awareness());
    cast(&mut g, id, Some(Target::Permanent(ogre)), Some(3));
    assert_eq!(pt(&g, ogre), (4, 4));
}

/// Black Tom Cassidy: the life rider needs *another* Mutant.
#[test]
fn black_tom_gains_life_only_beside_another_mutant() {
    let mut g = pod(2);
    let tom = ready(&mut g, 0, catalog::black_tom_cassidy());
    let life = g.players[0].life;
    activate(&mut g, tom, 0, None);
    assert_eq!(g.players[0].life, life);
    g.battlefield_find_mut(tom).unwrap().tapped = false;
    ready(&mut g, 0, catalog::black_tom_cassidy());
    activate(&mut g, tom, 0, None);
    assert_eq!(g.players[0].life, life + 1);
}

/// Windbrisk Raptor: lifelink on its controller's attackers only while attacking.
#[test]
fn windbrisk_raptor_gives_attackers_lifelink() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::windbrisk_raptor());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(!keywords(&g, bear).contains(&Keyword::Lifelink));
    let life = g.players[0].life;
    attack_unblocked(&mut g, bear, 1);
    assert_eq!(g.players[0].life, life + 2);
}

/// Runed Servitor and Miasmic Mummy fan out to every player at N = 4.
#[test]
fn runed_servitor_and_miasmic_mummy_reach_every_player() {
    let mut g = pod(4);
    for seat in 0..4 {
        g.add_card_to_hand(seat, catalog::island());
        g.add_card_to_library(seat, catalog::island());
    }
    let mummy = g.add_card_to_hand(0, catalog::miasmic_mummy());
    cast(&mut g, mummy, None, None);
    assert!(g.players.iter().all(|p| p.hand.is_empty()));
    let servitor = ready(&mut g, 0, catalog::runed_servitor());
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    cast(&mut g, wrath, None, None);
    assert!(g.battlefield_find(servitor).is_none());
    assert!(g.players.iter().all(|p| p.hand.len() == 1));
}

/// Fugitive Droid is unblockable only once an artifact entered under its
/// controller this turn.
#[test]
fn fugitive_droid_needs_an_artifact_arrival() {
    let mut g = pod(2);
    let droid = ready(&mut g, 0, catalog::fugitive_droid());
    assert!(!keywords(&g, droid).contains(&Keyword::Unblockable));
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None, None);
    assert!(keywords(&g, droid).contains(&Keyword::Unblockable));
}

/// Flash Thompson: both modes — tap the opponent's creature, untap his own.
#[test]
fn flash_thompson_taps_one_and_untaps_another() {
    let mut g = pod(2);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    let flash = g.add_card_to_hand(0, catalog::flash_thompson_spider_fan());
    act(
        &mut g,
        GameAction::CastSpell {
            card_id: flash,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
    assert!(g.battlefield_find(theirs).unwrap().tapped, "Heckle taps the opponent's creature");
    assert!(!g.battlefield_find(mine).unwrap().tapped, "Hero Worship untaps his controller's");
}

/// Chandra, Chill of Compliance's −X: tap and X stun counters.
#[test]
fn chandra_chill_of_compliance_stuns_for_x() {
    let mut g = pod(2);
    let chandra = ready(&mut g, 0, catalog::chandra_chill_of_compliance());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    act(
        &mut g,
        GameAction::ActivateLoyaltyAbility {
            card_id: chandra,
            ability_index: 2,
            target: Some(Target::Permanent(bear)),
            x_value: Some(2),
        },
    );
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped);
    assert_eq!(b.counter_count(CounterType::Stun), 2);
}

/// Shameless Charlatan: only a commander creature you own gets the copy
/// ability, and it keeps the ability after copying (the grant isn't copiable).
#[test]
fn shameless_charlatan_lets_the_commander_copy() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::shameless_charlatan());
    let cmdr = ready(&mut g, 0, catalog::black_tom_cassidy());
    g.players[0].commanders.push(cmdr);
    let other = ready(&mut g, 0, catalog::grizzly_bears());
    let drake = ready(&mut g, 1, catalog::wind_drake());
    flood(&mut g);
    let copy = |id| GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: Some(Target::Permanent(drake)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    assert!(g.perform_action(copy(other)).is_err(), "not a commander");
    // Black Tom's own mana ability is index 0; the granted one follows it.
    activate(&mut g, cmdr, 1, Some(Target::Permanent(drake)));
    assert_eq!(g.battlefield_find(cmdr).unwrap().definition.name, "Wind Drake");
    assert!(keywords(&g, cmdr).contains(&Keyword::Flying));
}
