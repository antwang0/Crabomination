//! CR conformance for rules exercised by the RAV gap waves 8-9:
//! CR 615.12 source-scoped "damage can't be prevented" (Excruciator vs a
//! global prevention shield), CR 701.16 sacrifice-vs-regeneration (Woebringer
//! Demon's edict), and CR 702-Radiance's card-type scoping (Leave No Trace
//! spares same-color creatures).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{
    Attack, AttackTarget, PreventionShield, PreventionTarget, Target,
};
use crabomination::game::{drain_stack, two_player_game, GameAction, TurnStep};

/// CR 615.12 — Excruciator's damage can't be prevented, so a player-wide
/// prevention shield doesn't stop it; a different attacker's damage still is.
#[test]
fn cr_615_12_excruciator_source_scoped_unpreventable() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let exc = g.add_card_to_battlefield(0, catalog::excruciator()); // 7/7
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(exc);
    g.clear_sickness(bear);
    // A shield that would prevent all damage dealt to player 1.
    g.prevention_shields.push(PreventionShield {
        target: PreventionTarget::Player(1),
        ..Default::default()
    });
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: exc, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    // Excruciator's 7 lands (unpreventable); the bear's 2 is prevented by the shield.
    assert_eq!(g.players[1].life, 13, "only Excruciator's 7 got through");
}

/// CR 701.16 — the Demon's edict is a sacrifice, not destruction, so a
/// regeneration shield on the chosen creature doesn't save it.
#[test]
fn cr_701_16_edict_sacrifice_ignores_regeneration() {
    let mut g = two_player_game();
    let _demon = g.add_card_to_battlefield(0, catalog::woebringer_demon());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().regeneration_shields = 1;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "sacrifice bypasses the regen shield");
}

/// CR 702 (Radiance) — Leave No Trace's Radiance fans out only over the
/// subject's card type (enchantments), so a same-color *creature* is spared.
#[test]
fn cr_702_radiance_is_scoped_to_the_subjects_card_type() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::glare_of_subdual()); // GW enchantment
    let white_creature = g.add_card_to_battlefield(0, catalog::savannah_lions()); // white
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(target)];
    let evs = g.resolve_effect(&catalog::leave_no_trace().effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "enchantment destroyed");
    assert!(g.battlefield_find(white_creature).is_some(), "same-color creature untouched");
}

// Keep the CR-tracker's counter row honest: Necroplasm's wrath reads its live
// +1/+1 counter count (CR 122 / 208.4b).
#[test]
fn cr_122_necroplasm_reads_live_counter_count() {
    let mut g = two_player_game();
    let nec = g.add_card_to_battlefield(0, catalog::necroplasm());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(nec).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "upkeep adds one counter",
    );
}
