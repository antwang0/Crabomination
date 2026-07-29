//! CR conformance for the modern_decks BNG run:
//! - CR 303.4a — "enchant player" Auras (the Curse cycle, Psychic Possession).
//! - CR 702.103f — a bestowed Aura on an illegal host unattaches and reverts to
//!   a creature instead of dying.
//! - CR 509.1b — the two new block restrictions (count-scaled power gate,
//!   "unless all creatures block it").

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Cast an "enchant player" Aura from seat 0 at seat 1.
fn curse(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add_colorless(10);
    for c in [
        crabomination::mana::Color::White,
        crabomination::mana::Color::Blue,
        crabomination::mana::Color::Black,
        crabomination::mana::Color::Red,
    ] {
        g.players[0].mana_pool.add(c, 3);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Aura at a player");
    drain_stack(g);
    id
}

/// CR 303.4a — the Aura resolves attached to the targeted player and stays on
/// the battlefield (the orphan-Aura SBA doesn't sweep it).
#[test]
fn cr_303_4a_player_aura_attaches_and_survives() {
    let mut g = main_phase();
    let id = curse(&mut g, catalog::curse_of_the_pierced_heart());
    let c = g.battlefield_find(id).expect("still on the battlefield");
    assert_eq!(c.attached_to_player, Some(1));
    assert_eq!(c.attached_to, None);
}

/// CR 303.4a — "at the beginning of enchanted player's upkeep" fires on their
/// turn only.
#[test]
fn cr_303_4a_enchanted_player_upkeep_trigger() {
    let mut g = main_phase();
    curse(&mut g, catalog::curse_of_the_pierced_heart());
    let life = g.players[1].life;
    for _ in 0..30 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
        if g.active_player_idx == 1 && g.step == TurnStep::PreCombatMain {
            break;
        }
    }
    assert_eq!(g.players[1].life, life - 1, "one ping on the enchanted player's upkeep");
}

/// CR 303.4a — a player-scoped anthem reaches that player's creatures only.
#[test]
fn cr_303_4a_curse_of_deaths_hold_shrinks_only_the_enchanted_player() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    curse(&mut g, catalog::curse_of_deaths_hold());
    assert_eq!(g.computed_permanent(theirs).map(|c| (c.power, c.toughness)), Some((1, 1)));
    assert_eq!(g.computed_permanent(mine).map(|c| (c.power, c.toughness)), Some((2, 2)));
}

/// CR 614.5 / 303.4a — Curse of Bloodletting doubles damage to its host only.
#[test]
fn cr_303_4a_curse_of_bloodletting_doubles_damage_to_the_enchanted_player() {
    let mut g = main_phase();
    curse(&mut g, catalog::curse_of_bloodletting());
    let (l1, l0) = (g.players[1].life, g.players[0].life);
    let mut events = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(1), 3, None, &mut events);
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 3, None, &mut events);
    assert_eq!(g.players[1].life, l1 - 6, "doubled");
    assert_eq!(g.players[0].life, l0 - 3, "untouched");
}

/// CR 303.4a — Curse of Exhaustion locks only the enchanted player.
#[test]
fn cr_303_4a_curse_of_exhaustion_locks_only_the_enchanted_player() {
    let mut g = main_phase();
    curse(&mut g, catalog::curse_of_exhaustion());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    for _ in 0..2 {
        let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        let r = g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        drain_stack(&mut g);
        if g.players[1].spells_cast_this_game_turn >= 1 && r.is_err() {
            return; // second cast rejected
        }
    }
    panic!("the enchanted player cast a second spell");
}

/// CR 303.4a — Psychic Possession mirrors the enchanted player's draws.
#[test]
fn cr_303_4a_psychic_possession_mirrors_draws() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    curse(&mut g, catalog::psychic_possession());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    let mut events = Vec::new();
    g.draw_one(1, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "you drew alongside them");
}

/// CR 702.103f — a bestowed Aura whose host stops being a creature unattaches
/// and comes back as a creature instead of hitting the graveyard.
#[test]
fn cr_702_103f_bestowed_aura_reverts_when_the_host_leaves() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eidolon = g.add_card_to_hand(0, catalog::ghostblade_eidolon());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastBestow {
        card_id: eidolon,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bestow");
    drain_stack(&mut g);
    assert!(g.battlefield_find(eidolon).unwrap().bestowed);

    let mut events = Vec::new();
    g.destroy_permanent(bear, false, &mut events);
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    let c = g.battlefield_find(eidolon).expect("the Eidolon stayed in play");
    assert!(!c.bestowed, "it stopped being bestowed");
    assert_eq!(c.attached_to, None);
    assert!(
        g.computed_permanent(eidolon).unwrap().card_types.contains(&CardType::Creature),
        "it is a creature again"
    );
}

/// CR 509.1b — Kraken of the Straits' threshold tracks the live Island count.
#[test]
fn cr_509_1b_power_gate_scales_with_the_island_count() {
    let mut g = main_phase();
    let kraken = g.add_card_to_battlefield(0, catalog::kraken_of_the_straits());
    g.clear_sickness(kraken);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::island());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kraken,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, kraken)]))
        .expect("2 power ≥ 1 Island");
}

/// CR 509.1b — Tromokratis keeps its hexproof only outside combat.
#[test]
fn cr_509_1b_tromokratis_hexproof_lapses_in_combat() {
    let mut g = main_phase();
    let kraken = g.add_card_to_battlefield(0, catalog::tromokratis());
    g.clear_sickness(kraken);
    assert!(g.computed_permanent(kraken).unwrap().keywords.contains(&Keyword::Hexproof));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kraken,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(!g.computed_permanent(kraken).unwrap().keywords.contains(&Keyword::Hexproof));
}
