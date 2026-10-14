//! Commander: two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_i`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::Effect;
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
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
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
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
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

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)));
}

fn resolve_at(
    g: &mut GameState,
    effect: &Effect,
    source: CardId,
    target: Option<Target>,
    mode: usize,
) {
    let mut ctx = EffectContext::for_spell(0, target, mode, 0);
    ctx.source = Some(source);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
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
/// CR 603.2 — Spider-Man grows a creature on life gain; Amazing Alliance
/// anthems and pays life per attacking legend.
#[test]
fn spider_man_and_amazing_alliance() {
    let mut g = pod(2);
    let spidey = ready(&mut g, 0, catalog::spider_man_peter_parker());
    ready(&mut g, 0, catalog::amazing_alliance());
    assert_eq!(pt(&g, spidey), (5, 5));
    let mut ctx = EffectContext::for_spell(0, Some(Target::Permanent(spidey)), 0, 0);
    ctx.source = Some(spidey);
    let gain = Effect::GainLife {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::Const(1),
    };
    let evs = g.resolve_effect(&gain, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(plus(&g, spidey), 1);
    assert!(
        g.computed_permanent(spidey)
            .unwrap()
            .keywords()
            .contains(&Keyword::Indestructible)
    );
    let life = g.players[0].life;
    declare(&mut g, vec![(spidey, 1)]);
    assert_eq!(g.players[0].life, life + 1, "one legendary attacker");
}

/// CR 610.3c — Silver Surfer blinks your permanents until the end step.
#[test]
fn silver_surfer_blinks_until_end_step() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let surfer = ready(&mut g, 0, catalog::silver_surfer_cosmic_voyager());
    let etb = catalog::silver_surfer_cosmic_voyager().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, surfer, Some(Target::Permanent(bear)), 0);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// CR 400.7 — Spaceshift returns the creature with a +1/+1 counter.
#[test]
fn spaceshift_flickers_with_a_counter() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::grizzly_bears());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::spaceshift());
    cast(&mut g, s, Some(Target::Permanent(bear)));
    let back = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        .unwrap()
        .id;
    assert_eq!(plus(&g, back), 1);
}

/// CR 602.1 — Patchwork Crawler borrows an exiled creature's activated ability.
#[test]
fn patchwork_crawler_borrows_abilities() {
    let mut g = pod(2);
    let crawler = ready(&mut g, 0, catalog::patchwork_crawler());
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    activate(&mut g, crawler, 0, Some(Target::Permanent(elf)));
    assert_eq!(plus(&g, crawler), 1);
    assert!(g.exile.iter().any(|c| c.id == elf));
    assert!(
        !g.granted_abilities_for(crawler).is_empty(),
        "the Elves' mana ability is borrowed"
    );
}

/// CR 701.9 — Compelling Deterrence bounces and, with a Zombie, makes a discard.
#[test]
fn compelling_deterrence_bounces_and_discards() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::patchwork_crawler());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    let s = g.add_card_to_hand(0, catalog::compelling_deterrence());
    cast(&mut g, s, Some(Target::Permanent(bear)));
    assert_eq!(g.players[1].hand.len(), 1, "bear back, one card discarded");
    assert_eq!(g.players[1].graveyard.len(), 1);
}

/// CR 707.2 — Permeating Mass turns what it damages into a copy of itself.
#[test]
fn permeating_mass_copies_onto_its_victim() {
    let mut g = pod(2);
    let mass = ready(&mut g, 0, catalog::permeating_mass());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let trig = catalog::permeating_mass().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &trig, mass, Some(Target::Permanent(angel)), 0);
    assert_eq!(
        g.computed_permanent(angel).unwrap().def.name,
        "Permeating Mass"
    );
    assert_eq!(pt(&g, angel), (1, 3));
}

/// CR 122.1b — Invoke the Ancients' Spirits get a keyword counter each.
#[test]
fn invoke_the_ancients_makes_two_spirits() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::invoke_the_ancients());
    cast(&mut g, s, None);
    let spirits: Vec<CardId> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Spirit")
        .map(|c| c.id)
        .collect();
    assert_eq!(spirits.len(), 2);
    for id in spirits {
        assert_eq!(pt(&g, id), (4, 5));
        assert!(
            g.computed_permanent(id)
                .unwrap()
                .keywords()
                .contains(&Keyword::Reach)
        );
    }
}

/// CR 122.1b — Myojin of Grim Betrayal returns this turn's dead creatures.
#[test]
fn myojin_of_grim_betrayal_mass_reanimates() {
    let mut g = pod(2);
    let myojin = g.add_card_to_hand(0, catalog::myojin_of_grim_betrayal());
    cast(&mut g, myojin, None);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    kill(&mut g, theirs);
    let old = g.add_card_to_graveyard(1, catalog::serra_angel());
    g.players[1].graveyard_ids_this_turn.retain(|id| *id != old);
    activate(&mut g, myojin, 0, None);
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0));
    assert!(g.battlefield_find(old).is_none(), "not put there this turn");
}

/// CR 702.2 — Urborg Scavengers takes keywords from the cards it exiles.
#[test]
fn urborg_scavengers_steals_keywords() {
    let mut g = pod(2);
    let angel = g.add_card_to_graveyard(1, catalog::serra_angel());
    let s = ready(&mut g, 0, catalog::urborg_scavengers());
    let etb = catalog::urborg_scavengers().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, s, Some(Target::Permanent(angel)), 0);
    assert_eq!(plus(&g, s), 1);
    let kw = g.computed_permanent(s).unwrap().keywords().to_vec();
    assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::Vigilance));
    assert!(!kw.contains(&Keyword::Trample));
}
