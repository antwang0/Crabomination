//! Commander: the Eldrazi Unbound precon (CMM, Zhulodok, `decks::cmdr_zhulodok`).
//! The primitives it needed are tested in `core_rules/commander_cards.rs`.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod() -> GameState {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for s in 0..3 {
        for _ in 0..8 {
            g.add_card_to_library(s, catalog::wastes());
        }
    }
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, extra: Vec<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: extra, mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_with(g, id, target, vec![])
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// CR 702.85a — Zhulodok: a colorless 7+ spell from hand cascades twice.
#[test]
fn zhulodok_cascades_twice() {
    let mut g = pod();
    g.add_card_to_battlefield(0, catalog::zhulodok_void_gorger());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::sol_ring());
    }
    let big = g.add_card_to_hand(0, catalog::it_that_betrays());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    cast(&mut g, big, None).expect("cast");
    let rings = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Sol Ring").count();
    assert_eq!(rings, 2, "two cascades, two free Rings");
}

/// CR 406 — Calamity of the Titans exiles what's smaller than the reveal.
#[test]
fn calamity_of_the_titans_scales_with_the_reveal() {
    let mut g = pod();
    g.add_card_to_hand(0, catalog::flayer_of_loyalties());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let huge = g.add_card_to_battlefield(2, catalog::it_that_betrays());
    let c = g.add_card_to_hand(0, catalog::calamity_of_the_titans());
    cast(&mut g, c, None).expect("reveal the Flayer");
    assert!(g.battlefield_find(small).is_none());
    assert!(g.battlefield_find(huge).is_some(), "12 isn't less than 10");
}

/// CR 702.143 (Adamant) — Desecrate Reality exiles even permanents; three
/// colorless returns an odd one.
#[test]
fn desecrate_reality_exiles_even() {
    let mut g = pod();
    let even = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let odd = g.add_card_to_graveyard(0, catalog::sol_ring());
    let dr = g.add_card_to_hand(0, catalog::desecrate_reality());
    g.players[0].mana_pool.add_colorless(7);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: dr,
        target: Some(Target::Permanent(even)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("seven colorless");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == even));
    assert!(g.battlefield_find(odd).is_some(), "adamant");
}

/// CR 613.4b — Flayer of Loyalties borrows a 10/10.
#[test]
fn flayer_borrows_a_titan() {
    let mut g = pod();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let f = g.add_card_to_hand(0, catalog::flayer_of_loyalties());
    cast(&mut g, f, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    assert_eq!(pt(&g, bear), (10, 10));
    assert!(g.permanent_has_keyword(bear, &Keyword::Annihilator(2)));
}

/// Guildless Commons bounces a land; Mage-Ring Network stores mana.
#[test]
fn the_lands() {
    let mut g = pod();
    let wastes = g.add_card_to_battlefield(0, catalog::wastes());
    let gc = g.add_card_to_hand(0, catalog::guildless_commons());
    g.perform_action(GameAction::PlayLand(gc)).expect("land");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == wastes));
    let mr = g.add_card_to_battlefield(0, catalog::mage_ring_network());
    g.battlefield_find_mut(mr).unwrap().counters.insert(CounterType::Storage, 3);
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mr,
        ability_index: 2,
        target: None,
        additional_targets: vec![],
        x_value: Some(3),
        mode: None,
    })
    .expect("remove three");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// CR 122.1 — Investigator's Journal counts the biggest board.
#[test]
fn investigators_journal_counts_creatures() {
    let mut g = pod();
    for _ in 0..3 {
        g.add_card_to_battlefield(2, catalog::grizzly_bears());
    }
    let j = g.add_card_to_hand(0, catalog::investigators_journal());
    cast(&mut g, j, None).expect("cast");
    assert_eq!(g.battlefield_find(j).unwrap().counter_count(CounterType::Suspect), 3);
    let hand = g.players[0].hand.len();
    activate(&mut g, j, 0, None, None);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 701.21 — It That Betrays takes what opponents sacrifice.
#[test]
fn it_that_betrays_takes_sacrifices() {
    let mut g = pod();
    g.add_card_to_battlefield(0, catalog::it_that_betrays());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_hand(1, catalog::diabolic_edict());
    g.players[1].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: victim,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("edict themselves");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0), "their sacrificed Bears");
    let _ = ring;
}

/// CR 707.2 — Mirage Mirror copies a creature until end of turn.
#[test]
fn mirage_mirror_copies() {
    let mut g = pod();
    let m = g.add_card_to_battlefield(0, catalog::mirage_mirror());
    let big = g.add_card_to_battlefield(1, catalog::it_that_betrays());
    activate(&mut g, m, 0, Some(Target::Permanent(big)), None);
    assert_eq!(g.battlefield_find(m).unwrap().definition.name, "It That Betrays");
}

/// CR 702.33 / 115 — Myriad Construct: kicked counters per opposing nonbasic
/// land; targeted by a spell, it splits into Constructs.
#[test]
fn myriad_construct_scatters() {
    let mut g = pod();
    g.add_card_to_battlefield(1, catalog::guildless_commons());
    g.add_card_to_battlefield(2, catalog::mage_ring_network());
    let mc = g.add_card_to_hand(0, catalog::myriad_construct());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: mc,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(pt(&g, mc), (6, 6));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(mc))).expect("target it");
    let constructs = g.battlefield.iter().filter(|c| c.definition.name == "Construct").count();
    assert_eq!(constructs, 6);
}

/// CR 122.6 — Omarthis enters with X counters, grows, and manifests on death.
#[test]
fn omarthis_grows_and_manifests() {
    let mut g = pod();
    let o = g.add_card_to_hand(0, catalog::omarthis_ghostfire_initiate());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: o, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
        .expect("X = 2");
    drain_stack(&mut g);
    assert_eq!(pt(&g, o), (2, 2));
    let other = g.add_card_to_battlefield(0, catalog::it_that_betrays());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let ev = g.resolve_effect(
        &crabomination::effect::Effect::AddCounter {
            what: crabomination::effect::Selector::Target(0),
            kind: CounterType::PlusOnePlusOne,
            amount: crabomination::effect::Value::ONE,
        },
        &crabomination::game::effects::EffectContext {
            targets: vec![Target::Permanent(other)],
            ..crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)
        },
    )
    .unwrap();
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(pt(&g, o), (3, 3));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(o))).expect("bolt");
    let face_down = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).count();
    assert_eq!(face_down, 3);
}

/// CR 500.7 — Rise of the Eldrazi: destroy, draw four, extra turn, exiled.
#[test]
fn rise_of_the_eldrazi() {
    let mut g = pod();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let r = g.add_card_to_hand(0, catalog::rise_of_the_eldrazi());
    let hand = g.players[0].hand.len();
    cast_with(&mut g, r, Some(Target::Permanent(bear)), vec![Target::Player(0)]).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].hand.len(), hand - 1 + 4);
    assert!(g.exile.iter().any(|c| c.id == r));
}

/// CR 702.8 — Skittering Cicada: colorless spells at instant speed pump it.
#[test]
fn skittering_cicada_flashes() {
    let mut g = pod();
    let sc = g.add_card_to_battlefield(0, catalog::skittering_cicada());
    g.step = TurnStep::DeclareBlockers;
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None).expect("a colorless spell at instant speed");
    assert_eq!(pt(&g, sc), (3, 3));
}

/// CR 701.8 — Transmogrifying Wand: destroy a creature, its controller gets
/// an Ox.
#[test]
fn transmogrifying_wand_makes_an_ox() {
    let mut g = pod();
    let w = g.add_card_to_battlefield(0, catalog::transmogrifying_wand());
    g.battlefield_find_mut(w).unwrap().counters.insert(CounterType::Charge, 3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, w, 0, Some(Target::Permanent(bear)), None);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Ox"));
}

/// CR 701.34 — Ugin's Mastery manifests on a colorless creature spell.
#[test]
fn ugins_mastery_manifests() {
    let mut g = pod();
    g.add_card_to_battlefield(0, catalog::ugins_mastery());
    let a = g.add_card_to_hand(0, catalog::abstruse_archaic());
    cast(&mut g, a, None).expect("cast");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).count(), 1);
    let _ = (Attack { attacker: a, target: AttackTarget::Player(1) }, Keyword::Trample);
}
