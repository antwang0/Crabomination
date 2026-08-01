//! Mercadian Masques (MMQ) gap closure, sixth (final) wave.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(
    g: &mut GameState,
    seat: usize,
    card_id: CardId,
    ability_index: usize,
    target: Option<Target>,
    additional_targets: Vec<Target>,
    x_value: Option<u32>,
) -> Result<(), crabomination::game::GameError> {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index,
        target,
        additional_targets,
        mode: None,
        x_value,
    })?;
    drain_stack(g);
    Ok(())
}

/// Charm Peddler shields a *creature* — not its controller — from one hit.
#[test]
fn charm_peddler_shields_target_creature() {
    let mut g = two_player_game();
    let peddler = g.add_card_to_battlefield(0, catalog::charm_peddler());
    g.clear_sickness(peddler);
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    // Prevent the Dreadmaw's damage to my Bears.
    script(&mut g, vec![DecisionAnswer::Cards(vec![theirs])]);
    activate(&mut g, 0, peddler, 0, Some(Target::Permanent(mine)), vec![], None).expect("activate");
    g.clear_sickness(theirs);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: theirs, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(mine, theirs)])).expect("block");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(mine).is_some(), "the shield ate the whole hit");
}

/// Cho-Arrim Alchemist converts the prevented damage into life.
#[test]
fn cho_arrim_alchemist_gains_life_for_prevented_damage() {
    let mut g = two_player_game();
    let alch = g.add_card_to_battlefield(0, catalog::cho_arrim_alchemist());
    g.clear_sickness(alch);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    script(&mut g, vec![DecisionAnswer::Cards(vec![theirs])]);
    activate(&mut g, 0, alch, 0, None, vec![], None).expect("activate");
    g.clear_sickness(theirs);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: theirs, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].life, 26, "6 prevented, 6 gained");
}

/// General's Regalia moves the hit onto a creature you control.
#[test]
fn generals_regalia_redirects_damage_to_your_creature() {
    let mut g = two_player_game();
    let regalia = g.add_card_to_battlefield(0, catalog::generals_regalia());
    let shield = g.add_card_to_battlefield(0, catalog::colossal_dreadmaw()); // 6/6
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    script(&mut g, vec![DecisionAnswer::Cards(vec![theirs])]);
    activate(&mut g, 0, regalia, 0, Some(Target::Permanent(shield)), vec![], None)
        .expect("activate");
    g.clear_sickness(theirs);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: theirs, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].life, 20, "nothing got through");
    assert_eq!(g.battlefield_find(shield).unwrap().damage, 2, "the Dreadmaw took it");
}

/// Bargaining Table's generic cost scales with the opponent's hand.
#[test]
fn bargaining_table_costs_the_opponents_hand_size() {
    let mut g = two_player_game();
    let table = g.add_card_to_battlefield(0, catalog::bargaining_table());
    g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: table,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "two mana can't cover a three-card hand"
    );
    activate(&mut g, 0, table, 0, None, vec![], None).expect("activate");
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Food Chain converts a creature into creature-only mana worth MV + 1.
#[test]
fn food_chain_makes_creature_only_mana() {
    let mut g = two_player_game();
    let chain = g.add_card_to_battlefield(0, catalog::food_chain());
    let fodder = g.add_card_to_battlefield(0, catalog::colossal_dreadmaw()); // MV 6
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: chain,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    assert!(g.exile.iter().any(|c| c.id == fodder), "the fodder is exiled, not sacrificed");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 7, "1 + mana value 6");
}

/// Caller of the Hunt's P/T counts every creature of the named type.
#[test]
fn caller_of_the_hunt_counts_the_named_type() {
    let mut g = two_player_game();
    script(&mut g, vec![DecisionAnswer::CreatureType(crabomination::card::CreatureType::Bear)]);
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // an opponent's Bear counts
    let caller = g.add_card_to_hand(0, catalog::caller_of_the_hunt());
    cast(&mut g, 0, caller, None);
    let cp = g.computed_permanent(caller).expect("computed");
    assert_eq!((cp.power, cp.toughness), (1, 1), "one Bear on the battlefield");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(caller).expect("computed");
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Charisma steals whatever the enchanted creature bites.
#[test]
fn charisma_steals_the_creature_its_host_damaged() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::prodigal_sorcerer());
    g.clear_sickness(host);
    let aura = g.add_card_to_hand(0, catalog::charisma());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    let victim = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // survives a ping
    activate(&mut g, 0, host, 0, Some(Target::Permanent(victim)), vec![], None).expect("ping");
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "changed sides");
}

/// Blood Oath deals 3 per card of the named type in the victim's hand.
#[test]
fn blood_oath_scales_with_the_named_card_type() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    g.add_card_to_hand(1, catalog::forest());
    let oath = g.add_card_to_hand(0, catalog::blood_oath());
    // AutoDecider's ChooseMode default is mode 0 = Creature.
    cast(&mut g, 0, oath, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14, "two creature cards → 6 damage");
}

/// Crooked Scales kills the opponent's creature on a won flip.
#[test]
fn crooked_scales_destroys_on_a_won_flip() {
    let mut g = two_player_game();
    let scales = g.add_card_to_battlefield(0, catalog::crooked_scales());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    script(&mut g, vec![DecisionAnswer::Bool(true)]); // the coin flip
    activate(
        &mut g,
        0,
        scales,
        0,
        Some(Target::Permanent(theirs)),
        vec![Target::Permanent(mine)],
        None,
    )
    .expect("activate");
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(mine).is_some());
}

/// A lost flip with a declined repeat kills your own creature.
#[test]
fn crooked_scales_eats_your_creature_when_you_stop() {
    let mut g = two_player_game();
    let scales = g.add_card_to_battlefield(0, catalog::crooked_scales());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // lost flip, then decline the {3} repeat.
    script(&mut g, vec![DecisionAnswer::Bool(false), DecisionAnswer::Bool(false)]);
    activate(
        &mut g,
        0,
        scales,
        0,
        Some(Target::Permanent(theirs)),
        vec![Target::Permanent(mine)],
        None,
    )
    .expect("activate");
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_some());
}

/// Kyren Archive banks library cards and cashes the stash into hand.
#[test]
fn kyren_archive_banks_and_returns_its_stash() {
    let mut g = two_player_game();
    let archive = g.add_card_to_battlefield(0, catalog::kyren_archive());
    let banked = g.add_card_to_library(0, catalog::grizzly_bears());
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == banked), "banked face down");

    let junk = g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, 0, archive, 0, None, vec![], None).expect("activate");
    assert!(g.players[0].hand.iter().any(|c| c.id == banked), "stash came back");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == junk), "hand was discarded");
    assert!(g.battlefield_find(archive).is_none(), "sacrificed");
}

/// Mercadian Lift cranks winch counters, then deploys a matching creature.
#[test]
fn mercadian_lift_deploys_a_creature_matching_its_winches() {
    let mut g = two_player_game();
    let lift = g.add_card_to_battlefield(0, catalog::mercadian_lift());
    g.clear_sickness(lift);
    let dreadmaw = g.add_card_to_hand(0, catalog::colossal_dreadmaw()); // MV 6
    for _ in 0..6 {
        activate(&mut g, 0, lift, 0, None, vec![], None).expect("crank");
        g.battlefield_find_mut(lift).unwrap().tapped = false;
    }
    assert_eq!(g.battlefield_find(lift).unwrap().counter_count(CounterType::Winch), 6);
    script(&mut g, vec![DecisionAnswer::Cards(vec![dreadmaw])]);
    activate(&mut g, 0, lift, 1, None, vec![], Some(6)).expect("deploy");
    assert!(g.battlefield_find(dreadmaw).is_some());
    assert_eq!(g.battlefield_find(lift).unwrap().counter_count(CounterType::Winch), 0);
}

/// Spiritual Focus pays off only when the *opponent* forces the discard.
#[test]
fn spiritual_focus_triggers_on_an_opponents_discard_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spiritual_focus());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let coercion = g.add_card_to_hand(1, catalog::mind_rot());
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    cast(&mut g, 1, coercion, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 22, "one discard → 2 life");
}

/// Thieves' Auction pots every nontoken permanent and drafts them back tapped.
#[test]
fn thieves_auction_redistributes_the_board() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let auction = g.add_card_to_hand(0, catalog::thieves_auction());
    // Seat 0 claims the Dreadmaw first; seat 1 is left with the Bears.
    script(&mut g, vec![DecisionAnswer::Cards(vec![theirs])]);
    cast(&mut g, 0, auction, None);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
    assert!(g.battlefield_find(theirs).unwrap().tapped, "claimed tapped");
}
