//! Commander: commanders that were their own one-card gap of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_r`).

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

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
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

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}
/// CR 702.11 — Palladia-Mors is hexproof until it has dealt damage.
#[test]
fn palladia_mors_loses_hexproof_after_damage() {
    let mut g = pod(2);
    let pm = ready(&mut g, 0, catalog::palladia_mors_the_ruiner());
    assert!(
        g.computed_permanent(pm)
            .unwrap()
            .keywords()
            .contains(&Keyword::Hexproof)
    );
    declare(&mut g, vec![(pm, 1)]);
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 14);
    assert!(
        !g.computed_permanent(pm)
            .unwrap()
            .keywords()
            .contains(&Keyword::Hexproof)
    );
}

/// CR 506.5 — Jiang Yanggu loots for a lone attacker and grows it per discard.
#[test]
fn jiang_yanggu_loots_for_a_lone_attacker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::jiang_yanggu_alone());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    declare(&mut g, vec![(bear, 1)]);
    assert_eq!(plus(&g, bear), 1);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Jiang Yanggu, Alone wants a lone attacker on a PLAYER: one attacking a
/// planeswalker alone doesn't loot.
#[test]
fn jiang_yanggu_ignores_a_lone_attack_on_a_planeswalker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::jiang_yanggu_alone());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let walker = ready(&mut g, 1, catalog::jace_reality_sculptor());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Planeswalker(walker) }]))
        .expect("attack the planeswalker");
    drain_stack(&mut g);
    assert_eq!(plus(&g, bear), 0);
    assert!(g.players[0].graveyard.is_empty(), "no loot");
}

/// CR 122.1b — Super-Adaptoid counts legends and borrows keywords as counters.
#[test]
fn super_adaptoid_copies_keyword_counters() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let sa = ready(&mut g, 0, catalog::super_adaptoid());
    assert_eq!(pt(&g, sa), (1, 2), "itself the one legend");
    let etb = catalog::super_adaptoid().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, sa, Some(Target::Permanent(angel)), 0);
    let kw = g.computed_permanent(sa).unwrap().keywords().to_vec();
    assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::Vigilance));
    assert!(!kw.contains(&Keyword::Trample));
}

/// CR 707.2 — Impossible Man becomes a copy until end of turn.
#[test]
fn impossible_man_copies_until_end_of_turn() {
    let mut g = pod(2);
    let im = ready(&mut g, 0, catalog::impossible_man());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    activate(&mut g, im, 0, Some(Target::Permanent(angel)));
    assert_eq!(pt(&g, im), (4, 4));
}

/// CR 707.9b — "except his name is Impossible Man": copying your own legend
/// leaves both on the battlefield (no legend-rule clash, CR 704.5j).
#[test]
fn impossible_man_keeps_his_name_when_copying_a_legend() {
    let mut g = pod(2);
    let im = ready(&mut g, 0, catalog::impossible_man());
    let legend = ready(&mut g, 0, catalog::jiang_yanggu_alone());
    activate(&mut g, im, 0, Some(Target::Permanent(legend)));
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(pt(&g, im), (4, 4), "a copy of the 4/4");
    assert_eq!(g.battlefield_find(im).unwrap().battlefield_name(), "Impossible Man");
    assert!(g.battlefield_find(legend).is_some(), "the original legend stays");
}

/// Stangg: each Aura and Equipment on him is copied onto the Twin (CR 707.2;
/// it was dropped), and every token made this way goes at the end step.
#[test]
fn stangg_twin_carries_copies_of_his_attachments() {
    let mut g = pod(2);
    let stangg = ready(&mut g, 0, catalog::stangg_echo_warrior());
    let axe = ready(&mut g, 0, catalog::bonesplitter());
    g.battlefield_find_mut(axe).unwrap().attached_to = Some(stangg);
    declare(&mut g, vec![(stangg, 1)]);
    let twin = g.battlefield.iter().find(|c| c.definition.name == "Stangg Twin").map(|c| c.id).expect("twin");
    let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Bonesplitter").expect("copy");
    assert_eq!(copy.attached_to, Some(twin));
    assert_eq!(pt(&g, twin), (5, 4));
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 10);
    for _ in 0..4 {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        if g.step == TurnStep::End {
            break;
        }
    }
    assert_eq!(named(&g, 0, "Stangg Twin"), 0);
    assert_eq!(named(&g, 0, "Bonesplitter"), 1, "the copy is sacrificed, the original stays");
}

/// CR 508.3a — Stangg's Twin joins the attack and is gone at the end step.
#[test]
fn stangg_brings_a_twin() {
    let mut g = pod(2);
    let stangg = ready(&mut g, 0, catalog::stangg_echo_warrior());
    declare(&mut g, vec![(stangg, 1)]);
    assert_eq!(named(&g, 0, "Stangg Twin"), 1);
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 14);
    for _ in 0..4 {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        if g.step == TurnStep::End {
            break;
        }
    }
    assert_eq!(g.step, TurnStep::End);
    assert_eq!(named(&g, 0, "Stangg Twin"), 0, "sacrificed at the end step");
}
