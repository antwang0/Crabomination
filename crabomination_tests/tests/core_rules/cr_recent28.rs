//! CR conformance for rules wired by this run's WAR walker batch:
//! CR 306.9 (combat damage to a planeswalker removes loyalty and fires a
//! "deals combat damage to a planeswalker" trigger — the new
//! `DealsCombatDamageToPlaneswalker` event), CR 509.1 (a creature declared as
//! a blocker is flagged for the turn — `R::BlockedThisTurn`), and CR 615
//! (`StaticEffect::PreventAllDamageToThis` blanks both combat and noncombat
//! damage to its source).

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 306.9 — combat damage dealt to a planeswalker removes that many loyalty
/// counters, and a `DealsCombatDamageToPlaneswalker` trigger (Vraska's Assassin
/// token) sees it and destroys the walker.
#[test]
fn cr_306_9_combat_damage_to_planeswalker_triggers_and_removes_loyalty() {
    let mut g = two_player_game();
    let vraska = g.add_card_to_battlefield(0, catalog::vraska_swarms_eminence());
    let walker = g.add_card_to_battlefield(1, catalog::jace_arcane_strategist()); // loyalty 4
    // Make Vraska's 1/1 deathtouch Assassin (destroys walkers it damages).
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: vraska, ability_index: 0, target: None, x_value: None }).expect("-2");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Assassin").unwrap().id;
    g.clear_sickness(token);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: token, target: AttackTarget::Planeswalker(walker) }])).expect("attack walker");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.battlefield_find(walker).is_none(), "the trigger destroyed the damaged planeswalker");
}

/// CR 509.1 — a creature declared as a blocker carries `blocked_this_turn`
/// (matched by `R::BlockedThisTurn`) for the rest of the turn.
#[test]
fn cr_509_1_declared_blocker_is_flagged_for_the_turn() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    // A second creature that never blocks stays unflagged.
    let bench = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    assert!(g.battlefield_find(blk).unwrap().blocked_this_turn, "declared blocker flagged");
    assert!(!g.battlefield_find(bench).unwrap().blocked_this_turn, "non-blocker unflagged");
}

/// CR 615 — `PreventAllDamageToThis` (Gideon Blackblade during your turn) blanks
/// both combat-marked and noncombat damage to its source; the planeswalker loses
/// no loyalty from a burn hit.
#[test]
fn cr_615_prevent_all_damage_to_this_covers_both_paths() {
    let mut g = two_player_game();
    let gid = g.add_card_to_battlefield(0, catalog::gideon_blackblade());
    g.active_player_idx = 0; // its controller's turn → the prevention is live
    assert!(g.computed_permanent(gid).unwrap().card_types.contains(&CardType::Creature));
    let before = g.battlefield_find(gid).unwrap().counter_count(CounterType::Loyalty);
    // Noncombat (burn) damage: prevented, no loyalty lost.
    let mut evs = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(gid), 4, None, &mut evs);
    assert_eq!(g.battlefield_find(gid).unwrap().counter_count(CounterType::Loyalty), before, "noncombat damage prevented");
    assert!(g.combat_damage_prevented_to_self(gid), "combat damage to it is also prevented");
    // Indestructible too — it survives a lethal marking on its own turn.
    assert!(g.computed_permanent(gid).unwrap().keywords.contains(&Keyword::Indestructible));
}
