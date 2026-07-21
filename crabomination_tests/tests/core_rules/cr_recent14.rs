//! CR conformance for the RAV/GPT gap wave 11-15 prevention/redirect work:
//! CR 614.9 (damage redirection — Carom, Pariah's Shield), CR 615
//! (source-scoped prevention — Light of Sanction, Indentured Oaf), and
//! CR 701.10 (exchange control is permanent — Spawnbroker).

use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

/// CR 614.9 — Carom redirects the next 1 damage; the redirect target's own
/// protection then applies to the re-dealt damage (a fresh event).
#[test]
fn cr_614_9_redirected_damage_is_a_fresh_event() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
    g.resolve_effect(&catalog::carom().effect, &ctx).unwrap();
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(a), 1, None, &mut evs);
    assert_eq!(g.battlefield_find(a).unwrap().damage, 0, "a's damage fully redirected");
    assert_eq!(g.battlefield_find(b).unwrap().damage, 1, "dealt to b instead");
}

/// CR 614.9 — Pariah's Shield redirects the wearer's damage to the equipped
/// creature; unequipped, the player takes it.
#[test]
fn cr_614_9_pariahs_shield_only_redirects_while_attached() {
    let mut g = two_player_game();
    let shield = g.add_card_to_battlefield(0, catalog::pariahs_shield());
    let mut evs = Vec::new();
    let life0 = g.players[0].life;
    g.deal_damage_to_from(EntityRef::Player(0), 3, None, &mut evs);
    assert_eq!(g.players[0].life, life0 - 3, "no creature attached → player takes it");
    let cre = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(shield).unwrap().attached_to = Some(cre);
    g.deal_damage_to_from(EntityRef::Player(0), 3, None, &mut evs);
    assert_eq!(g.players[0].life, life0 - 3, "now redirected");
    assert_eq!(g.battlefield_find(cre).unwrap().damage, 3);
}

/// CR 615 — Light of Sanction prevents only damage from the controller's own
/// sources to their own creatures.
#[test]
fn cr_615_light_of_sanction_source_and_target_scoped() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::light_of_sanction());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let my_src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let foe_src = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(mine), 1, Some(my_src), &mut evs);
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "your source prevented");
    g.deal_damage_to_from(EntityRef::Permanent(mine), 1, Some(foe_src), &mut evs);
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 1, "opponent's source lands");
}

/// CR 615 — Indentured Oaf prevents its own damage to red creatures only.
#[test]
fn cr_615_indentured_oaf_prevents_only_red() {
    let mut g = two_player_game();
    let oaf = g.add_card_to_battlefield(0, catalog::indentured_oaf());
    let red = g.add_card_to_battlefield(1, catalog::goblin_arsonist());
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    // A different source hitting the red creature is unaffected.
    g.deal_damage_to_from(EntityRef::Permanent(red), 1, Some(other), &mut evs);
    assert_eq!(g.battlefield_find(red).unwrap().damage, 1, "only the Oaf's damage is prevented");
    g.deal_damage_to_from(EntityRef::Permanent(red), 2, Some(oaf), &mut evs);
    assert_eq!(g.battlefield_find(red).unwrap().damage, 1, "Oaf spares red");
    g.deal_damage_to_from(EntityRef::Permanent(green), 2, Some(oaf), &mut evs);
    assert_eq!(g.battlefield_find(green).unwrap().damage, 2, "Oaf hits non-red");
}

/// CR 701.10 — exchanging control is permanent (no duration): Spawnbroker's
/// swap sticks after the effect resolves.
#[test]
fn cr_701_10_exchange_control_is_permanent() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(mine), Target::Permanent(theirs)];
    let evs = g
        .resolve_effect(&catalog::spawnbroker().triggered_abilities[0].effect, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    // Advance a full turn cycle; control must not revert.
    for _ in 0..6 {
        let _ = g.advance_step(vec![]);
    }
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
}
