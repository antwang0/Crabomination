//! A spell castable only during combat (Spinal Embrace) has one window, and
//! the default profile's combat window casts pure pump tricks only — so
//! without this such a card was castable by no path (never cast in 1,000
//! 21-seat pods, seed 10031). Mirror Match ("cast only during the declare
//! blockers step") rides the same window once something attacks the seat.

use super::bot::{EvalWeights, cast_candidates};
use crate::game::GameState;
use crate::game::types::GameAction;

/// The first accepted cast of a combat-only card in `seat`'s hand, in the
/// order `cast_candidates` ranks them. `None` without one in hand — a hand
/// walk, so the common case costs nothing more.
pub(super) fn pick_combat_only_spell(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    let hand = &state.players[seat].hand;
    let attacked = || state.attacking().iter().any(|a| state.defender_for(a.target) == Some(seat));
    // Mirror Match copies what attacks *you*: worth its six mana only then.
    let combat_only = |def: &crate::card::CardDefinition| {
        def.cast_only_during_combat
            || (matches!(def.effect, crate::effect::Effect::CopyAttackersAsBlockers) && attacked())
    };
    if !hand.iter().any(|c| combat_only(&c.definition)) {
        return None;
    }
    cast_candidates(state, seat, w, None)
        .into_iter()
        .map(|(a, _)| a)
        .filter(|a| {
            matches!(a, GameAction::CastSpell { card_id, .. }
                if hand.iter().any(|c| c.id == *card_id && combat_only(&c.definition)))
        })
        .find(|a| state.would_accept(a.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::two_player_game;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Spinal Embrace is offered in combat and only there.
    #[test]
    fn a_combat_only_spell_is_cast_in_combat_and_not_before() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let spell = g.add_card_to_hand(0, crate::catalog::spinal_embrace());
        g.add_card_to_battlefield(1, crate::catalog::hill_giant());
        for c in [Color::Blue, Color::Blue, Color::Black] {
            g.players[0].mana_pool.add(c, 1);
        }
        g.players[0].mana_pool.add_colorless(3);
        let w = EvalWeights::default();
        g.step = TurnStep::PreCombatMain;
        assert_eq!(pick_combat_only_spell(&g, 0, &w), None);
        g.step = TurnStep::DeclareBlockers;
        let picked = pick_combat_only_spell(&g, 0, &w);
        assert!(matches!(picked, Some(GameAction::CastSpell { card_id, .. }) if card_id == spell));
    }

    /// Mirror Match is cast by the defender once blocks are in, and not by a
    /// seat nobody is attacking (its copies would block nothing).
    #[test]
    fn mirror_match_is_cast_only_when_attacked() {
        use crate::game::types::{Attack, AttackTarget, GameAction as A};
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 1;
        let mm = g.add_card_to_hand(0, crate::catalog::mirror_match());
        g.add_card_to_hand(2, crate::catalog::mirror_match());
        let giant = g.add_card_to_battlefield(1, crate::catalog::hill_giant());
        g.clear_sickness(giant);
        for seat in [0, 2] {
            g.players[seat].mana_pool.add(Color::Blue, 2);
            g.players[seat].mana_pool.add_colorless(4);
        }
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 1;
        g.perform_action(A::DeclareAttackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }]))
            .expect("attack seat 0");
        g.step = TurnStep::DeclareBlockers;
        let w = EvalWeights::default();
        g.priority.player_with_priority = 2;
        assert_eq!(pick_combat_only_spell(&g, 2, &w), None, "seat 2 isn't attacked");
        g.priority.player_with_priority = 0;
        let picked = pick_combat_only_spell(&g, 0, &w);
        assert!(matches!(picked, Some(A::CastSpell { card_id, .. }) if card_id == mm));
    }
}
