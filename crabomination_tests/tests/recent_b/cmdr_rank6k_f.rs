//! Commander: one- and two-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_f`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::card::Value;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::effects::EffectContext;
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
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

fn declare(g: &mut GameState, attackers: Vec<CardId>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.into_iter().map(|attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// CR 602.5d — Starnheim Memento's pump is a sorcery-speed +1/+1 and flying.
#[test]
fn starnheim_memento_pumps_and_grants_flying() {
    let mut g = pod(2);
    let memento = ready(&mut g, 0, catalog::starnheim_memento());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, memento, 1, Some(Target::Permanent(bears)));
    let c = g.computed_permanent(bears).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3));
    assert!(c.keywords().contains(&Keyword::Flying));
}

/// CR 603.3d — Trusty Packbeast returns an artifact card to hand.
#[test]
fn trusty_packbeast_returns_an_artifact() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let beast = g.add_card_to_hand(0, catalog::trusty_packbeast());
    cast(&mut g, beast, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
}

/// CR 603.4 — Kirk's attack trigger checks total power 8 or greater: with
/// Serra Angel beside him (4 + 4) the top card is exiled; alone it isn't.
#[test]
fn kirk_exiles_only_at_total_power_eight() {
    let mut g = pod(2);
    let kirk = ready(&mut g, 0, catalog::kirk_enterprising_captain());
    g.add_card_to_library(0, catalog::forest());
    declare(&mut g, vec![kirk]);
    assert!(g.exile.is_empty(), "total power 4");

    let mut g = pod(2);
    let kirk = ready(&mut g, 0, catalog::kirk_enterprising_captain());
    ready(&mut g, 0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::forest());
    declare(&mut g, vec![kirk]);
    assert!(g.exile.iter().any(|c| c.definition.name == "Forest"), "total power 8");
}

/// CR 119.10 — Christine Chapel gains one life per differently named land,
/// then her own life-gain trigger adds two +1/+1 counters.
#[test]
fn christine_chapel_counts_distinct_land_names() {
    let mut g = pod(2);
    for land in [catalog::forest(), catalog::forest(), catalog::island()] {
        ready(&mut g, 0, land);
    }
    let life = g.players[0].life;
    let chapel = g.add_card_to_hand(0, catalog::christine_chapel_combat_medic());
    cast(&mut g, chapel, None);
    assert_eq!(g.players[0].life, life + 2);
    assert_eq!(plus(&g, chapel), 2);
}

/// CR 603.6c / 603.10 — Titanbones' discard trigger gains 3 life.
#[test]
fn titanbones_gains_life_when_discarded() {
    let mut g = pod(2);
    let tb = g.add_card_to_hand(0, catalog::titanbones_towering_heart());
    let life = g.players[0].life;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(&Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == tb));
    assert_eq!(g.players[0].life, life + 3);
}

/// CR 702.21a / 509.1b — Trelane is a 4/5 with vigilance, ward {2} and
/// can't be blocked.
#[test]
fn trelane_keywords() {
    let d = catalog::trelane_squire_of_gothos();
    assert_eq!((d.power, d.toughness), (4, 5));
    assert!(d.keywords.contains(&Keyword::Vigilance));
    assert!(d.keywords.contains(&Keyword::Unblockable));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// CR 707.9b — Quicksilver Gargantuan copies a creature but stays 7/7.
#[test]
fn quicksilver_gargantuan_copies_as_a_7_7() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::grizzly_bears());
    let qg = g.add_card_to_hand(0, catalog::quicksilver_gargantuan());
    cast(&mut g, qg, None);
    let c = g.battlefield_find(qg).expect("on battlefield");
    assert_eq!(c.definition.name, "Grizzly Bears");
    let cp = g.computed_permanent(qg).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
}
