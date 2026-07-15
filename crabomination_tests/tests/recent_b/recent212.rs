//! Functionality tests for `catalog::sets::decks::recent212`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Goblin Smuggler makes a small creature unblockable.
#[test]
fn goblin_smuggler_grants_unblockable() {
    let mut g = two_player_game();
    let smug = g.add_card_to_battlefield(0, catalog::goblin_smuggler());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.clear_sickness(smug);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: smug, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("grant unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Joraga Invocation pumps your team and forces blocks.
#[test]
fn joraga_invocation_pumps_and_lures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::joraga_invocation());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Joraga Invocation");
    drain_stack(&mut g);
    let v = g.computed_permanent(bear).unwrap();
    assert_eq!((v.power, v.toughness), (5, 5), "+3/+3");
    assert!(v.keywords.contains(&Keyword::MustBeBlocked), "must be blocked");
}

/// Aurelia untaps your team and grants an extra combat on her first attack.
#[test]
fn aurelia_untaps_and_adds_combat() {
    let mut g = two_player_game();
    let aurelia = g.add_card_to_battlefield(0, catalog::aurelia_the_warleader());
    let tapped = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    g.clear_sickness(aurelia);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: aurelia, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(tapped).unwrap().tapped, "team untapped");
    assert!(g.additional_combat_phases > 0, "an extra combat phase is queued");
}

/// Mindsparker zaps an opponent for casting a blue instant.
#[test]
fn mindsparker_zaps_on_blue_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mindsparker());
    // Opponent bolts our face with a blue counterspell target… use a bolt on stack.
    g.step = TurnStep::PreCombatMain;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on stack");
    let blue = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: blue, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a blue spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "Mindsparker dealt 2 to the caster");
}

/// Ingenious Leonin puts a counter on an attacker and grants a Cat first strike.
#[test]
fn ingenious_leonin_counters_and_grants_first_strike() {
    let mut g = two_player_game();
    let leonin = g.add_card_to_battlefield(0, catalog::ingenious_leonin());
    // A second Cat attacker (Leonin can't target itself in the printed text).
    let cat = g.add_card_to_battlefield(0, catalog::ingenious_leonin());
    g.clear_sickness(cat);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: cat, target: AttackTarget::Player(1) }]).expect("attack");
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: leonin, ability_index: 0, target: Some(Target::Permanent(cat)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("pump the attacking Cat");
    drain_stack(&mut g);
    let v = g.computed_permanent(cat).unwrap();
    assert_eq!(*g.battlefield_find(cat).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 1);
    assert!(v.keywords.contains(&Keyword::FirstStrike), "Cat gained first strike");
}

/// Crossway Troublemakers gives attacking Vampires deathtouch + lifelink.
#[test]
fn crossway_buffs_attacking_vampires() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crossway_troublemakers());
    let vamp = g.add_card_to_battlefield(0, catalog::highborn_vampire()); // 4/3 Vampire Warrior
    g.clear_sickness(vamp);
    // Not attacking yet → no grant.
    let idle = g.computed_permanent(vamp).unwrap();
    assert!(!idle.keywords.contains(&Keyword::Deathtouch), "idle Vampire has no bonus");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: vamp, target: AttackTarget::Player(1) }]).expect("attack");
    let atk = g.computed_permanent(vamp).unwrap();
    assert!(atk.keywords.contains(&Keyword::Deathtouch), "attacking Vampire gains deathtouch");
    assert!(atk.keywords.contains(&Keyword::Lifelink), "and lifelink");
}
