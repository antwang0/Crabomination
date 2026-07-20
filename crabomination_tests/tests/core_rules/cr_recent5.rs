//! CR conformance for recent293 (Ravnica batch 3) behaviours:
//! CR 702.55 (Haunt — dies to exile haunting a creature, payoff on that
//! creature's death), CR 702.58 (Graft — move a +1/+1 counter to a creature as
//! it enters), and CR 702.6e / 303 (an Aura's granted "at the beginning of your
//! upkeep" trigger keys on the enchanted permanent's controller).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, Target, TurnStep};
use crabomination::mana::Color;

fn kill(g: &mut GameState, id: CardId) {
    let ctrl = g.battlefield_find(id).unwrap().controller;
    let ctx = EffectContext::for_ability(id, ctrl, Some(Target::Permanent(id)));
    g.resolve_effect(&Effect::SacrificePermanent { what: Selector::Target(0) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(g);
}

/// CR 702.55 — a haunt creature that dies is exiled haunting a creature (not
/// put into its graveyard); the payoff fires again when the haunted creature
/// dies.
#[test]
fn cr_702_55_haunt_exiles_and_pays_off_on_haunted_death() {
    let mut g = two_player_game();
    let hunter = g.add_card_to_battlefield(0, catalog::blind_hunter());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    kill(&mut g, hunter);
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == hunter),
        "a dead haunt creature is exiled, not put into the graveyard");
    assert!(g.exile.iter().any(|c| c.id == hunter), "it's in exile, haunting");
    let foe = g.players[1].life;
    kill(&mut g, victim);
    assert_eq!(g.players[1].life, foe - 2, "the haunted creature's death drains 2");
}

/// CR 702.58 — Graft: when another creature enters, its controller may move a
/// +1/+1 counter from the graft creature onto the newcomer.
#[test]
fn cr_702_58_graft_moves_a_counter_to_an_entering_creature() {
    let mut g = two_player_game();
    // Real placement so Graft 4's `enters_with_counters` applies.
    let sages = g.move_card_to_battlefield_for_test(0, catalog::novijen_sages());
    assert_eq!(g.computed_permanent(sages).unwrap().power, 4, "4/4 from four counters");
    // Say yes to the "you may move a counter" graft trigger.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "bear grew to 3/3 from the moved counter");
    assert_eq!(g.computed_permanent(sages).unwrap().power, 3, "Novijen dropped to 3/3");
}

/// CR 702.6e / 303 — a triggered ability an Aura grants to the enchanted
/// permanent fires on the *host's* controller's step, and "you" in that ability
/// is the host's controller (Pillory of the Sleepless bleeds the enchanted
/// creature's controller, not the Aura's).
#[test]
fn cr_702_6e_aura_granted_upkeep_trigger_keys_on_host_controller() {
    let mut g = two_player_game();
    // P0 controls the Aura; it enchants P1's creature.
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::pillory_of_the_sleepless());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(creature);
    g.battlefield_find_mut(aura).unwrap().controller = 0;
    let comp = g.computed_permanent(creature).unwrap();
    assert!(comp.keywords.contains(&Keyword::CantAttack) && comp.keywords.contains(&Keyword::CantBlock));
    // On the Aura controller's (P0's) upkeep, nothing happens.
    g.active_player_idx = 0;
    let (p0, p1) = (g.players[0].life, g.players[1].life);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0, "Aura's controller is unaffected");
    // On the host controller's (P1's) upkeep, P1 loses 1.
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 1, "the enchanted creature's controller bleeds");
}
