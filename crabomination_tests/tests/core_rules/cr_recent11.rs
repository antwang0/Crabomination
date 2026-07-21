//! CR conformance for rules exercised by the RAV/GPT gap wave: CR 702.36 (Fear
//! — blockable only by artifact and/or black creatures), CR 702.16b (protection
//! from the chosen color — Order of the Stars) and CR 702.107 (Replicate — the
//! spell is copied once per replicate payment; Siege of Towers).

use crabomination::card::{CardDefinition, CardType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{StackItem, Target};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, TurnStep};
use crabomination::game::{Attack, AttackTarget};
use crabomination::mana::{b, cost, g as green, Color};

fn colored_body(name: &'static str, p: i32, t: i32, pip: crabomination::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: pip, card_types: vec![CardType::Creature], power: p, toughness: t, ..Default::default() }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 702.36 — a creature with fear can be blocked only by artifact and/or
/// black creatures. A green blocker is illegal; a black one is legal.
#[test]
fn cr_702_36_fear_blockable_only_by_artifact_or_black() {
    let mut g = two_player_game();
    let mut atk = colored_body("Fearsome", 2, 2, cost(&[b()]));
    atk.keywords = vec![Keyword::Fear];
    let atk = g.add_card_to_battlefield(0, atk);
    let green_blk = g.add_card_to_battlefield(1, colored_body("Greenie", 2, 2, cost(&[green()])));
    let black_blk = g.add_card_to_battlefield(1, colored_body("Darkling", 2, 2, cost(&[b()])));
    g.clear_sickness(atk);

    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(green_blk, atk)])).is_err(),
        "a green creature can't block a fear attacker");
    g.perform_action(GameAction::DeclareBlockers(vec![(black_blk, atk)]))
        .expect("a black creature may block a fear attacker");
}

/// CR 702.16b — Order of the Stars enters, chooses a color, and gains
/// protection from that color.
#[test]
fn cr_702_16b_protection_from_chosen_color() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    let order = g.move_card_to_battlefield_for_test(0, catalog::order_of_the_stars());
    drain_stack(&mut g);
    assert!(g.computed_permanent(order).unwrap().keywords.contains(&Keyword::Protection(Color::Red)),
        "has protection from the chosen color");
}

/// CR 702.107 — Replicate copies the spell once per replicate payment. Siege of
/// Towers cast with one replicate payment puts the original plus one copy on the
/// stack.
#[test]
fn cr_702_107_replicate_copies_the_spell() {
    let mut g = two_player_game();
    let mtn_a = g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(0, catalog::mountain());
    let siege = g.add_card_to_hand(0, catalog::siege_of_towers());
    // {1}{R} base + {1}{R} replicate = {2}{R}{R}.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellReplicate {
        card_id: siege, times: 1,
        target: Some(Target::Permanent(mtn_a)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Siege of Towers replicated once");
    let copies = g.stack.iter().filter(|si| matches!(si, StackItem::Spell { .. })).count();
    assert_eq!(copies, 2, "original + one replicate copy on the stack");
}
