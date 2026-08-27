//! CR 508.1a/d — "attacks each combat if able", where *able* is the whole
//! restriction list the declaration gate checks rather than the five the
//! requirement loop used to spell out by hand.
//!
//! Two hand-maintained walkers answering the same question is the drift class
//! this repo keeps closing, and here it produced a state with **no legal
//! declaration at all**: the requirement loop said a Juggernaut under an
//! unmet CR 508.1a restriction was able, so leaving it home was rejected —
//! and declaring it was rejected by the gate. Each test below picks a
//! restriction family the old `able` never read.

use crabomination::card::{
    ArtifactSubtype, CardDefinition, CardType, Keyword, LandType, SelectionRequirement as R, Subtypes,
};
use crabomination::catalog;
use crabomination::game::types::TurnStep;
use crabomination::game::*;

fn forced(name: &'static str, p: i32, t: i32, extra: Vec<Keyword>) -> CardDefinition {
    let mut keywords = vec![Keyword::MustAttack];
    keywords.extend(extra);
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        power: p,
        toughness: t,
        keywords,
        ..Default::default()
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// The control: with nothing restricting it, a must-attack creature really
/// does have to be declared. Sacred — it is what stops the fix below from
/// being "delete the requirement".
#[test]
fn cr_508_1d_an_unrestricted_must_attacker_is_still_required() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, forced("Juggernaut Test", 3, 3, vec![]));
    g.clear_sickness(c);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![]))
        .expect_err("CR 508.1d — it is able, so it must attack");
}

/// CR 508.1a — Goblin Cohort's gate. The creature is not able, so declaring
/// nothing is legal; before the walkers were unified the seat had no legal
/// declaration in either direction.
#[test]
fn cr_508_1d_a_cast_gated_must_attacker_is_not_able() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(
        0,
        forced("Cohort Test", 3, 3, vec![Keyword::CantAttackUnlessCastCreatureThisTurn]),
    );
    g.clear_sickness(c);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    assert_eq!(g.players[0].creatures_cast_this_turn, 0);
    g.perform_action(GameAction::DeclareAttackers(vec![]))
        .expect("no creature was cast, so it is not able and nothing is required");
}

/// CR 508.1a — a hand-size gate (Hazoret-class) reached through the same
/// walker. The requirement loop reads the *live* board, so the same creature
/// flips back to required once the gate opens.
#[test]
fn cr_508_1d_a_hand_size_gate_switches_the_requirement_off_and_on() {
    let build = |hand: usize| {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(
            0,
            forced("Hazoret Test", 4, 4, vec![Keyword::CantAttackOrBlockUnlessHandSizeAtMost(0)]),
        );
        g.clear_sickness(c);
        g.players[0].hand.clear();
        for _ in 0..hand {
            g.add_card_to_hand(0, catalog::island());
        }
        g
    };
    let mut locked = build(1);
    advance_to(&mut locked, TurnStep::DeclareAttackers);
    locked
        .perform_action(GameAction::DeclareAttackers(vec![]))
        .expect("a full hand makes it unable, so nothing is required");

    let mut open = build(0);
    advance_to(&mut open, TurnStep::DeclareAttackers);
    open.perform_action(GameAction::DeclareAttackers(vec![]))
        .expect_err("an empty hand makes it able again");
}

/// CR 508.1a — Dandân's defender-side gate is the *target*-dependent half of
/// the split. Able means "some legal defender would accept it", so a lone
/// islandless opponent leaves the creature unable.
#[test]
fn cr_508_1d_a_defender_gated_must_attacker_is_not_able() {
    let dandan = |islands: usize| {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(
            0,
            forced(
                "Dandan Test",
                4,
                1,
                vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::HasLandType(LandType::Island)))],
            ),
        );
        g.clear_sickness(c);
        for _ in 0..islands {
            g.add_card_to_battlefield(1, catalog::island());
        }
        g
    };
    let mut dry = dandan(0);
    advance_to(&mut dry, TurnStep::DeclareAttackers);
    dry.perform_action(GameAction::DeclareAttackers(vec![]))
        .expect("no Island across the table, so it is not able");

    let mut wet = dandan(1);
    advance_to(&mut wet, TurnStep::DeclareAttackers);
    wet.perform_action(GameAction::DeclareAttackers(vec![]))
        .expect_err("an Island makes the defender legal, so it must attack");
}

/// CR 613 / 508.1a — an Ensnaring Bridge cap is a restriction too, and the
/// requirement loop only sees it because the cap gather is hoisted above it.
#[test]
fn cr_508_1d_a_power_capped_must_attacker_is_not_able() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, forced("Capped Test", 4, 4, vec![]));
    g.clear_sickness(c);
    g.add_card_to_battlefield(0, catalog::ensnaring_bridge());
    g.players[0].hand.clear();
    g.add_card_to_hand(0, catalog::island());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![]))
        .expect("a 4-power creature can't attack under a 1-card hand, so nothing is required");
}

/// CR 302.1 — a permanent that is not a creature right now cannot attack, so
/// it cannot be *required* to. The bestowed-Aura shape PERF (-55)'s census
/// found, read from the requirement side.
#[test]
fn cr_508_1d_a_non_creature_must_attacker_is_not_able() {
    let vehicle = CardDefinition {
        name: "Uncrewed Test",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::MustAttack],
        ..Default::default()
    };
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, vehicle);
    g.clear_sickness(c);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![]))
        .expect("an uncrewed Vehicle is not a creature, so it is not able");
}

// ── The block side of the same class ────────────────────────────────────────
//
// CR 702.39's Provoke and CR 509.1b's "must block if able" ask the same
// question about a *blocker*, and their `able` was four gates against the
// declaration gate's ~twenty. Same consequence, one step later in the turn:
// the requirement says a creature is able, the gate rejects it, and the
// defending seat has no legal block declaration in either direction.

fn provoked_board(prep: impl FnOnce(&mut GameState, CardId, CardId)) -> GameState {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(atk);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    // CR 702.39 — the provoke resolution sets `must_block` and untaps.
    g.battlefield_find_mut(blocker).expect("on the battlefield").must_block = Some(atk);
    prep(&mut g, blocker, atk);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g
}

/// The control, and it is what stops the fix from being "delete the
/// requirement": an unrestricted provoked creature really must block.
#[test]
fn cr_702_39_an_unrestricted_provoked_creature_is_still_required() {
    let mut g = provoked_board(|_, _, _| {});
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect_err("CR 702.39 — it is able, so it must block its provoker");
}

/// CR 701.35 — a detained creature can't block, so it is not able and
/// declaring no blocks is legal. Before the walkers were unified the seat had
/// no legal declaration in either direction.
#[test]
fn cr_702_39_a_detained_provoked_creature_is_not_able() {
    let mut g = provoked_board(|g, b, _| {
        g.battlefield_find_mut(b).expect("on the battlefield").detained_by = Some(0);
    });
    let blocker = g.battlefield.iter().find(|c| c.controller == 1).expect("blocker").id;
    let attacker = g.attacking()[0].attacker;
    g.clone()
        .perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect_err("CR 701.35 — a detained permanent can't block");
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("and so it is not able, and blocking with nobody is legal");
}

/// CR 702.15 — landwalk is a *pair* rule the two-creature check cannot see,
/// because it reads the defending player's board. A provoked creature that
/// cannot legally block its provoker is not able.
#[test]
fn cr_702_39_a_landwalked_provoked_creature_is_not_able() {
    let mut g = provoked_board(|g, _, atk| {
        g.battlefield_find_mut(atk)
            .expect("on the battlefield")
            .granted_keywords_eot
            .push(Keyword::Landwalk(LandType::Island));
        g.add_card_to_battlefield(1, catalog::island());
    });
    let blocker = g.battlefield.iter().find(|c| c.controller == 1).expect("blocker").id;
    let attacker = g.attacking()[0].attacker;
    g.clone()
        .perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect_err("CR 702.15 — islandwalk is unblockable while they control an Island");
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("and so it is not able, and blocking with nobody is legal");
}

/// CR 509.1b — and the drift ran the *other* way too: seven
/// "can't block unless …" families lived only in the bot's mirror, so the
/// declaration gate did not enforce them at all and the restriction did
/// nothing on the real play path.
#[test]
fn cr_509_1b_the_declaration_gate_enforces_the_unless_families() {
    for kw in [
        Keyword::CantAttackOrBlockUnlessCityBlessing,
        Keyword::CantAttackOrBlockUnlessDelirium,
        Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn,
        Keyword::CantAttackOrBlockUnlessDescend(3),
        Keyword::CantAttackOrBlockUnlessCardsInExile(7),
        Keyword::CantAttackOrBlockUnlessHandSizeAtMost(0),
    ] {
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, catalog::hill_giant());
        g.clear_sickness(atk);
        let gated = CardDefinition {
            name: "Gated Wall",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 4,
            keywords: vec![kw.clone()],
            ..Default::default()
        };
        let blocker = g.add_card_to_battlefield(1, gated);
        // A card in hand, so the hand-size gate has something to fail on.
        g.add_card_to_hand(1, catalog::plains());
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        advance_to(&mut g, TurnStep::DeclareBlockers);
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)]))
            .unwrap_err();
    }
}
