//! CR conformance for this run:
//! - CR 315 — conspiracies live in the command zone and never leave it.
//! - CR 701.38 — a council's-dilemma body runs once per vote, bound to the
//!   seat that cast it.
//! - CR 905 — draft notes carry into the game; the pre-game exile pile too.
//! - CR 612/613.1c — a text-changing name grant adds names without removing
//!   the printed one.

use crabomination::card::{CreatureType, Keyword, SelectionRequirement as R};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::draft::{ANIMUS_OF_PREDATION, DraftNotes, PALIANO_VANGUARD};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;

fn resolve_sorcery(g: &mut GameState, def: crabomination::card::CardDefinition) {
    let id = g.add_card_to_hand(0, def);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(12);
    for c in [crabomination::mana::Color::Blue, crabomination::mana::Color::Green] {
        g.players[0].mana_pool.add(c, 2);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// CR 315.1 — a conspiracy is never a permanent and is never cast; seating one
/// puts it in the command zone, not on the battlefield or the stack.
#[test]
fn cr_315_1_a_conspiracy_is_never_a_permanent() {
    let mut g = two_player_game();
    let id = g.seat_conspiracy(0, catalog::emissarys_ploy(), None);
    assert!(g.battlefield_find(id).is_none());
    assert!(g.stack.is_empty());
    assert!(g.players[0].command.iter().any(|c| c.id == id));
}

/// CR 315.5 — a face-up conspiracy's static abilities function from the
/// command zone; CR 315.5b — a face-down one has no characteristics, so its
/// abilities do nothing until it is turned face up.
#[test]
fn cr_315_5b_a_face_down_conspiracy_grants_nothing() {
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let id = g.seat_double_agenda(
        0,
        catalog::summoners_bond(),
        "Grizzly Bears",
        "Savannah Lions",
    );
    let lions = g.add_card_to_library(0, catalog::savannah_lions());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(lions))]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Bears");
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().all(|c| c.id != lions),
        "the agenda is still face down, so nothing triggered"
    );
    assert!(g.reveal_hidden_agenda(0, id), "CR 702.106b — turn it face up");
}

/// CR 701.38 — council's dilemma runs one copy of the body per vote, and the
/// body can name the player who cast that vote.
#[test]
fn cr_701_38_a_dilemma_body_names_its_own_voter() {
    let mut g = two_player_game();
    let mine = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    let theirs = g.move_card_to_battlefield_for_test(1, catalog::savannah_lions());
    // Both seats vote Money; each vote takes a permanent from its own voter.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(1),
        DecisionAnswer::Amount(1),
        DecisionAnswer::Cards(vec![mine]),
        DecisionAnswer::Cards(vec![theirs]),
    ]));
    resolve_sorcery(&mut g, catalog::expropriate());
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.players[0].extra_turns, 0, "no Time votes were cast");
}

/// CR 905.2b — the values noted during the draft are readable in the game that
/// follows, and only under the noting card's own name.
#[test]
fn cr_905_2b_draft_notes_are_scoped_to_the_noting_card_name() {
    let mut g = two_player_game();
    let mut notes = DraftNotes::default();
    notes.note_keywords(ANIMUS_OF_PREDATION, &[Keyword::Flying]);
    notes.note_creature_types(PALIANO_VANGUARD, &[CreatureType::Bear]);
    g.players[0].draft_notes = notes;
    assert_eq!(g.players[0].draft_notes.noted_keywords(ANIMUS_OF_PREDATION), [Keyword::Flying]);
    assert!(g.players[0].draft_notes.noted_keywords(PALIANO_VANGUARD).is_empty());
    assert!(g.players[1].draft_notes.noted_keywords(ANIMUS_OF_PREDATION).is_empty());
}

/// CR 905.4 — a card exiled before the game with a draft-matters card is in
/// exile, not in the deck, and answers to that card's name.
#[test]
fn cr_905_4_the_pre_game_exile_pile_is_public_and_named() {
    let mut g = two_player_game();
    let id = g.seat_draft_exile(0, "Arcane Savant", catalog::divination());
    assert!(g.exile.iter().any(|c| c.id == id), "in exile");
    assert!(g.players[0].library.iter().all(|c| c.id != id), "not in the deck");
    assert!(g.players[0].draft_notes.has_name("Arcane Savant", "Divination"));
}

/// CR 613.1c — a name grant is a text-changing effect that *adds* names; the
/// printed one still matches, and only nonlegendary creature names are added.
#[test]
fn cr_613_1c_a_name_grant_adds_names_without_replacing_the_printed_one() {
    let mut g = two_player_game();
    let bear = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    let kit = g.move_card_to_battlefield_for_test(0, catalog::spy_kit());
    g.battlefield_find_mut(kit).unwrap().attached_to = Some(bear);
    let named = |g: &GameState, n: &str| {
        g.evaluate_requirement_static(&R::HasName(n.to_string()), &Target::Permanent(bear), 0, None)
    };
    assert!(named(&g, "Grizzly Bears"), "the printed name still matches");
    assert!(named(&g, "Savannah Lions"), "and every nonlegendary creature name");
    assert!(!named(&g, "Queen Marchesa"), "but not a legendary one");
}
