//! CR 701.27f — an ability can't transform a permanent that has already
//! transformed since the ability went on the stack.
//!
//! "If an activated or triggered ability of a permanent that isn't a delayed
//! triggered ability of that permanent tries to transform it, the permanent
//! does so only if it hasn't transformed or converted since the ability was
//! put onto the stack. … if the permanent has already transformed or
//! converted, an instruction to do either is ignored."
//!
//! Without it, two copies of one ability in a batch flip the face back and
//! forth and the card ends where it started. `EventKind::PermanentSacrificed`
//! has fanned out since long before this file, so Daring Sleuth — "whenever
//! you sacrifice a Clue, transform this" — read *untransformed* off a
//! two-Clue sacrifice and transformed off a one- or three-Clue one.

use crabomination::catalog;
use crabomination::game::effects::clue_token;
use crabomination::game::*;

/// Two Clues sacrificed in one batch are two triggers, and the second one's
/// transform is ignored.
#[test]
fn cr_701_27f_a_second_transform_of_the_same_source_is_ignored() {
    let mut g = two_player_game();
    let sleuth = g.add_card_to_battlefield(0, catalog::daring_sleuth());
    let evs: Vec<GameEvent> = (0..2)
        .map(|_| {
            let id = g.add_token_to_battlefield(0, &clue_token());
            GameEvent::PermanentSacrificed { card_id: id, who: 0 }
        })
        .collect();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.stack.len(), 2, "CR 603.6 — one trigger per sacrificed Clue");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sleuth).expect("still there").definition.name,
        "Bearer of Overwhelming Truths",
        "the second trigger's transform is ignored, not a flip back",
    );
}

/// The rule is about the ability's OWN permanent. A single trigger still
/// transforms it, so the guard cannot be reading "any transform at all".
#[test]
fn cr_701_27f_one_trigger_still_transforms() {
    let mut g = two_player_game();
    let sleuth = g.add_card_to_battlefield(0, catalog::daring_sleuth());
    let clue = g.add_token_to_battlefield(0, &clue_token());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentSacrificed { card_id: clue, who: 0 }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sleuth).expect("still there").definition.name,
        "Bearer of Overwhelming Truths",
    );
}

/// Three is the count a toggle would get right by accident. Every copy after
/// the first is ignored, so the face is the same as it is for two.
#[test]
fn cr_701_27f_a_third_copy_is_ignored_too() {
    let mut g = two_player_game();
    let sleuth = g.add_card_to_battlefield(0, catalog::daring_sleuth());
    let evs: Vec<GameEvent> = (0..3)
        .map(|_| {
            let id = g.add_token_to_battlefield(0, &clue_token());
            GameEvent::PermanentSacrificed { card_id: id, who: 0 }
        })
        .collect();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.stack.len(), 3);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sleuth).expect("still there").definition.name,
        "Bearer of Overwhelming Truths",
    );
}
