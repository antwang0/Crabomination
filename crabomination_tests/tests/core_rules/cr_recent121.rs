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
