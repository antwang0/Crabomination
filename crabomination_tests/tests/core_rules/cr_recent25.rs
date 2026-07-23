//! CR conformance for rules exercised by this run's DGM gap wave 4:
//! CR 702.97a (Scavenge activates only as a sorcery — Varolz's grant),
//! CR 509.1c (a "must block if able" creature has to be declared as a blocker
//! — Boros Battleshaper's grant), and CR 121.2a (a draw redirect is a
//! replacement effect — Notion Thief skips the opponent's draw rather than
//! triggering off it).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// CR 702.97a — Varolz's granted scavenge is a sorcery-speed activation; it is
/// rejected on an opponent's turn.
#[test]
fn cr_702_97a_scavenge_only_as_sorcery() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::varolz_the_scar_striped());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Opponent's turn: not a legal time for a sorcery-speed activation.
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let boost = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").unwrap().id;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: dead, ability_index: 0, target: Some(Target::Permanent(boost)),
            additional_targets: vec![], x_value: None,
        }).is_err(),
        "scavenge can't be activated at instant speed",
    );
}

/// CR 509.1c — a creature that "blocks if able" must be assigned to block; a
/// declaration that leaves it out (while it can legally block) is illegal.
#[test]
fn cr_509_1c_must_block_is_enforced() {
    let mut g = two_player_game();
    // Player 1 attacks; player 0's blocker is forced to block.
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(blocker).unwrap().granted_keywords_eot.push(Keyword::MustBlock);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("advance to blockers");
    }
    // Declaring no blockers is illegal while the MustBlock creature can block.
    let empty = g.perform_action(GameAction::DeclareBlockers(vec![]));
    assert!(empty.is_err(), "omitting a forced blocker is illegal");
    // Assigning it to the attacker is accepted.
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect("forced block is legal");
}

/// CR 121.2a / 614 — Notion Thief redirects (replaces) the opponent's draw: the
/// opponent's library is untouched and the thief's library loses the card.
#[test]
fn cr_121_2a_notion_thief_is_a_replacement() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::notion_thief());
    let opp_top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::hill_giant());
    let mut ev = Vec::new();
    g.draw_one(0, &mut ev); // a non-draw-step draw by the opponent
    assert!(
        g.players[0].library.iter().any(|c| c.id == opp_top),
        "opponent's card stays in their library (draw skipped)",
    );
    assert!(
        g.players[1].hand.iter().any(|c| c.definition.name == "Hill Giant"),
        "thief drew from their own library instead",
    );
}
