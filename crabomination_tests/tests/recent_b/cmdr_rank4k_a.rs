//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
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

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn has(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&k)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}

/// CR 506.5 — Ma Chao attacking alone can't be blocked; beside another
/// attacker it can.
#[test]
fn ma_chao_is_unblockable_only_alone() {
    let mut g = pod(2);
    let ma = ready(&mut g, 0, catalog::ma_chao_western_warrior());
    declare(&mut g, vec![(ma, 1)]);
    assert!(has(&g, ma, Keyword::Unblockable));

    let mut g = pod(2);
    let ma = ready(&mut g, 0, catalog::ma_chao_western_warrior());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(ma, 1), (bear, 1)]);
    assert!(!has(&g, ma, Keyword::Unblockable));
}

/// CR 702.164 — Joven and Chandler's backup grants its artifact-breaking
/// trigger; the backed-up creature's combat hit destroys that player's
/// artifact.
#[test]
fn joven_and_chandler_backs_up_an_artifact_breaker() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ring = ready(&mut g, 2, catalog::sol_ring());
    let joven = g.add_card_to_hand(0, catalog::joven_and_chandler());
    cast(&mut g, joven, None, None);
    // Backup's target is auto-picked: whichever got the counters attacks.
    assert_eq!(plus(&g, bear) + plus(&g, joven), 2);
    let backed = if plus(&g, bear) == 2 { bear } else { joven };
    declare(&mut g, vec![(backed, 2)]);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none(), "seat 2's Sol Ring was destroyed");
}

/// Jiang Yanggu makes legendary Mowu, and untaps your tokens at your end step.
#[test]
fn jiang_yanggu_untaps_tokens_at_end_step() {
    let mut g = pod(2);
    let jiang = g.add_card_to_hand(0, catalog::jiang_yanggu_never_alone());
    cast(&mut g, jiang, None, None);
    let mowu = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Mowu")
        .map(|c| c.id)
        .expect("Mowu");
    g.battlefield_find_mut(mowu).unwrap().tapped = true;
    step(&mut g, TurnStep::End);
    assert!(!g.battlefield_find(mowu).unwrap().tapped);
}

/// Luke Cage makes a Treasure at the beginning of combat on your turn only.
#[test]
fn luke_cage_mints_treasure_at_your_combat() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::luke_cage_hero_for_hire());
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(named(&g, 0, "Treasure"), 1);
    g.active_player_idx = 1;
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(named(&g, 0, "Treasure"), 1);
}

/// Machinesmith Automaton and Decorated Champion grow off another arrival
/// of their kind; Sengir Connoisseur grows once a turn however many die.
#[test]
fn arrival_and_death_counters() {
    let mut g = pod(2);
    let smith = ready(&mut g, 0, catalog::machinesmith_automaton());
    let champ = ready(&mut g, 0, catalog::decorated_champion());
    let sengir = ready(&mut g, 0, catalog::sengir_connoisseur());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None, None);
    assert_eq!(plus(&g, smith), 1);
    assert_eq!(plus(&g, champ), 0);
    let warrior = g.add_card_to_hand(0, catalog::marwyn_the_clearcutter());
    cast(&mut g, warrior, None, None);
    assert_eq!(plus(&g, champ), 1);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, wrath, Some(Target::Permanent(a)), None);
    let wrath = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, wrath, Some(Target::Permanent(b)), None);
    assert_eq!(plus(&g, sengir), 1, "once each turn");
}

/// Big Bertha enters with X +1/+1 counters (CR 107.3).
#[test]
fn big_bertha_enters_with_x() {
    let mut g = pod(2);
    let bertha = g.add_card_to_hand(0, catalog::big_bertha());
    cast(&mut g, bertha, None, Some(4));
    assert_eq!(plus(&g, bertha), 4);
}

/// Zamriel's grant is "during your turn" only (CR 611.3a).
#[test]
fn zamriel_shields_equipped_creatures_on_your_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::zamriel_seraph_of_steel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let plate = ready(&mut g, 0, catalog::bonesplitter());
    act(&mut g, GameAction::Equip { equipment: plate, target: bear });
    assert!(has(&g, bear, Keyword::Indestructible));
    g.active_player_idx = 1;
    assert!(!has(&g, bear, Keyword::Indestructible));
}

/// Gandalf's {6}: he goes into his owner's library and they draw three.
#[test]
fn gandalf_shuffles_home_and_draws_three() {
    let mut g = pod(2);
    let gandalf = ready(&mut g, 0, catalog::gandalf_wandering_wizard());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let lib = g.players[0].library.len();
    activate(&mut g, gandalf, 0, None);
    assert!(g.battlefield_find(gandalf).is_none());
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].library.len(), lib + 1 - 3);
}

/// Diligent Excavator mills a target player two whenever you cast a
/// historic spell.
#[test]
fn diligent_excavator_mills_on_historic() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::diligent_excavator());
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None, None);
    let milled = g.players[0].graveyard.len() + g.players[1].graveyard.len();
    assert_eq!(milled, 0, "a Bear isn't historic");
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None, None);
    let milled = g.players[0].graveyard.len() + g.players[1].graveyard.len();
    assert_eq!(milled, 2);
}
