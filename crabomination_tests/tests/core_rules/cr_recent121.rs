//! CR 603.3a — an Equipment's unquoted "whenever equipped creature …" is the
//! Equipment's own triggered ability: its controller controls it, even when
//! someone else controls the equipped creature (CR 702.6e grants only quoted
//! abilities).

use crabomination::card::{CardDefinition, CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction};
use crabomination::game::*;
use crabomination::TurnStep;

/// Seat 0 owns the Equipment; it sits on seat 1's creature, which attacks
/// seat 2 on seat 1's turn.
fn lent(def: fn() -> CardDefinition, host: fn() -> CardDefinition) -> (GameState, CardId, CardId) {
    let mut g = multi_player_game(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let c = g.add_card_to_battlefield(1, host());
    g.clear_sickness(c);
    let eq = g.add_card_to_battlefield(0, def());
    g.battlefield_find_mut(eq).unwrap().attached_to = Some(c);
    (g, c, eq)
}

fn attack(g: &mut GameState, c: CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: c, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Attack trigger: Armory of Iroas's counter still lands on "it", the
/// attacking creature, now that `This` is the Equipment.
#[test]
fn cr_603_3a_armory_of_iroas_counters_the_creature_it_equips() {
    let (mut g, bear, _) = lent(catalog::armory_of_iroas, catalog::grizzly_bears);
    attack(&mut g, bear);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Combat-damage trigger: Goldvein Pick's Treasure is the Pick controller's.
#[test]
fn cr_603_3a_goldvein_pick_treasure_goes_to_the_equipments_controller() {
    let (mut g, bear, _) = lent(catalog::goldvein_pick, catalog::grizzly_bears);
    attack(&mut g, bear);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!((named(&g, 0, "Treasure"), named(&g, 1, "Treasure")), (1, 0));
}

/// Dies trigger: Slayer's Plate's Spirit is the Plate controller's, and its
/// "if it was a Human" still reads the dead creature.
#[test]
fn cr_603_3a_slayers_plate_spirit_goes_to_the_equipments_controller() {
    use crabomination::effect::{Effect, Selector};
    let (mut g, human, _) = lent(catalog::slayers_plate, catalog::thraben_inspector);
    let ctx = crabomination::game::effects::EffectContext::for_spell(2, None, 0, 0);
    let ev = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![human]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(human).is_none());
    assert_eq!((named(&g, 0, "Spirit"), named(&g, 1, "Spirit")), (1, 0));
}

/// CR 603.3a / 508.4 — an Equipment's own attack trigger reads its host's
/// attack: Captain's Claws' Kor Ally attacks whichever seat the equipped
/// creature attacks, not its controller's default opponent.
#[test]
fn cr_603_3a_captains_claws_token_attacks_its_hosts_defender() {
    for defender in [0usize, 2] {
        let mut g = multi_player_game(3);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        let claws = g.add_card_to_battlefield(1, catalog::captains_claws());
        g.battlefield_find_mut(claws).unwrap().attached_to = Some(bear);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(defender) }]))
            .expect("attack");
        drain_stack(&mut g);
        let ally = g.battlefield.iter().find(|c| c.definition.name.contains("Kor Ally")).map(|c| c.id).expect("token");
        let a = g.attacking.iter().find(|a| a.attacker == ally).expect("attacking");
        assert_eq!(a.target, AttackTarget::Player(defender));
    }
}

/// Mage Slayer's own attack trigger: the equipped creature deals damage equal
/// to its power to the player it attacks (`AttackedBySource` reads the host).
#[test]
fn cr_603_3a_mage_slayer_hits_what_its_host_attacks() {
    let (mut g, bear, _) = lent(catalog::mage_slayer, catalog::grizzly_bears);
    attack(&mut g, bear);
    assert_eq!((g.players[0].life, g.players[2].life), (20, 18));
}

/// Heirloom Blade's own dies trigger: the Blade's controller reveals from
/// their library for a card sharing a type with the dead creature (a Human).
#[test]
fn cr_603_3a_heirloom_blade_finds_a_card_sharing_the_dead_creatures_type() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, Selector};
    let (mut g, human, _) = lent(catalog::heirloom_blade, catalog::thraben_inspector);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let inspector = g.add_card_to_library(0, catalog::thraben_inspector());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(2, None, 0, 0);
    let ev = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![human]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == inspector), "the Human went to the Blade controller's hand");
}

/// Saw — "sacrifice a permanent other than that creature or this Equipment"
/// (bug: any permanent you controlled, the attacker and the Saw included).
#[test]
fn saw_never_sacrifices_its_creature_or_itself() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let saw = g.add_card_to_battlefield(0, catalog::saw());
    g.battlefield_find_mut(saw).unwrap().attached_to = Some(bear);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some() && g.battlefield_find(saw).is_some());
    assert!(g.battlefield_find(forest).is_none(), "the Forest is the only legal sacrifice");
}

/// Slab Hammer — "If you do, the creature gets +2/+2": no land returned, no
/// pump (bug: the pump applied with no land to return).
#[test]
fn slab_hammer_pumps_only_if_a_land_was_returned() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let (mut g, bear, _) = lent(catalog::slab_hammer, catalog::grizzly_bears);
    g.battlefield.retain(|c| !c.definition.card_types.contains(&crabomination::card::CardType::Land));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, bear);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2));
}

/// Foot Chopper — "you may sacrifice it. If you do, draw cards equal to its
/// power" (bug: it drew first and drew even when the sacrifice failed). On a
/// lent creature the Chopper's controller can't sacrifice it (CR 701.21a),
/// so nothing is drawn; on its own, the draw reads the sacrificed power.
#[test]
fn foot_chopper_draws_only_if_it_sacrificed() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for own in [true, false] {
        let mut g = multi_player_game(3);
        let seat = if own { 0 } else { 1 };
        g.active_player_idx = seat;
        g.priority.player_with_priority = seat;
        let bear = g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.clear_sickness(bear);
        let chopper = g.add_card_to_battlefield(0, catalog::foot_chopper());
        g.battlefield.retain(|c| !c.definition.name.contains("Ninja"));
        g.battlefield_find_mut(chopper).unwrap().attached_to = Some(bear);
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let hand = g.players[0].hand.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        attack(&mut g, bear);
        g.step = TurnStep::CombatDamage;
        let ev = g.resolve_combat().expect("damage");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).is_none(), own, "own={own}: sacrificed only if controlled");
        assert_eq!(g.players[0].hand.len() - hand, if own { 2 } else { 0 }, "own={own}");
    }
}
