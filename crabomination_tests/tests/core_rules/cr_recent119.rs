//! CR 608.2 — a resolution that asks each seat in turn and then moves a card
//! per seat: every answer is taken before any card moves, so a replay after a
//! later seat's park sees the same top cards.

use crabomination::card::{CardDefinition, CardType};
use crabomination::catalog;
use crabomination::decision::DecisionAnswer;
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;

/// Three prompting seats each reveal a Forest over an Island; seat 1 declines.
/// Moving seat 0's Forest before seat 1's ask made the replay reveal seat 0's
/// Island and hand seat 1 seat 0's answer.
#[test]
fn cr_608_2_each_seat_answers_before_any_top_card_moves() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let mut forests = Vec::new();
    for s in 0..3 {
        g.players[s].wants_ui = true;
        g.players[s].library.clear();
        forests.push(g.add_card_to_library(s, catalog::forest()));
        g.add_card_to_library(s, catalog::island());
    }
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Each Reveals",
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealTopLandToBattlefieldElseHand {
            who: PlayerRef::EachPlayer,
            tapped: false,
            nonland_stays: false,
            may: true,
        },
        ..Default::default()
    });
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.resolve_top_of_stack().expect("resolve");
    for yes in [true, false, true] {
        assert!(g.pending_decision.is_some(), "a seat is asked");
        g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(yes))).expect("answer");
    }
    assert!(g.pending_decision.is_none());
    assert!(g.battlefield_find(forests[0]).is_some(), "seat 0 put its Forest onto the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == forests[1]), "seat 1 declined: Forest to hand");
    assert!(g.battlefield_find(forests[2]).is_some(), "seat 2 put its Forest onto the battlefield");
    for s in 0..3 {
        assert_eq!(g.players[s].library.len(), 1, "each Island stays on top of seat {s}'s library");
    }
}

/// CR 701.40a / 708 — a manifest that cannot put the card onto the battlefield
/// (Grafdigger's Cage) leaves it face up where it stayed. Orochi Soul-Reaver's
/// "manifest the top card of that player's library" turned it face down first
/// and left it so in the library (audit pod seed 2814860).
#[test]
fn cr_701_40a_a_blocked_manifest_leaves_the_library_card_face_up() {
    use crabomination::game::effects::EffectContext;
    let mut g = multi_player_game(2);
    g.add_card_to_battlefield(1, catalog::grafdiggers_cage());
    let src = g.add_card_to_battlefield(0, catalog::orochi_soul_reaver());
    g.players[1].library.clear();
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    let manifest = Effect::ManifestTopOfLibraryUnderYou { who: PlayerRef::Seat(1) };
    g.resolve_effect(&manifest, &EffectContext::for_ability(src, 0, None)).expect("resolves");
    let card = g.players[1].library.iter().find(|c| c.id == top).expect("still in the library");
    assert!(!card.face_down, "face up again where it stayed");
}

/// CR 610.3c — "exile … until [this] leaves the battlefield" exiles nothing
/// once the source has already left: a token copy of Abdel Adrian the legend
/// rule removed before its ETB resolved still exiled every chosen permanent,
/// with no leave left to return them (audit pod seed 2941996, six seats).
#[test]
fn cr_610_3c_a_gone_source_exiles_nothing_until_it_leaves() {
    use crabomination::game::effects::EffectContext;
    let mut g = multi_player_game(2);
    let abdel = g.add_card_to_battlefield(0, catalog::abdel_adrian_gorions_ward());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let etb = catalog::abdel_adrian_gorions_ward().triggered_abilities[0].effect.clone();
    let mut gone = g.clone();
    gone.resolve_effect(&Effect::Exile { what: Selector::ExactObjects(vec![abdel]) }, &EffectContext::for_ability(abdel, 0, None))
        .expect("exile Abdel");
    gone.resolve_effect(&etb, &EffectContext::for_ability(abdel, 0, None)).expect("the ETB resolves");
    assert!(gone.battlefield_find(bear).is_some(), "nothing exiled: the source already left");
    g.resolve_effect(&etb, &EffectContext::for_ability(abdel, 0, None)).expect("the ETB resolves");
    assert!(g.exile.iter().any(|c| c.id == bear), "with Abdel present the Bears are exiled");
}
