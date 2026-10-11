//! CR 723.5 — while another player controls a seat (Mindslaver, Secret of
//! Bloodbending), its controller makes that seat's choices. A pod's bots each
//! play their own seat, so this stands in for the controller with the plainest
//! hostile policy: the controlled seat casts, plays and activates nothing and
//! declares no attackers. Choices the rules force (a discard, a target) stay
//! with the seat's own bot.

use crate::game::{GameAction, GameState};

/// The action the controller makes `seat` take instead of `action`, when
/// `seat` is controlled right now and `action` is one the policy declines.
pub(crate) fn controller_override(g: &GameState, seat: usize, action: &GameAction) -> Option<GameAction> {
    if g.acting_seat_for(seat) == seat {
        return None;
    }
    use GameAction as A;
    match action {
        A::DeclareAttackers(a) if a.is_empty() => None,
        A::DeclareAttackers(_) | A::DeclareAttackersBanded { .. } | A::DeclareAttackersExerting { .. } => {
            Some(A::DeclareAttackers(Vec::new()))
        }
        a if a.is_cast_or_loyalty()
            || matches!(a, A::ActivateAbility { .. } | A::PlayLand(_) | A::PlayLandBack(_) | A::PlayLandFromGraveyard(_)) =>
        {
            Some(A::PassPriority)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardId;

    /// CR 723.5 — a controlled seat's casts become passes and its attack
    /// declaration is empty; an uncontrolled seat is untouched.
    #[test]
    fn cr_723_5_controller_declines_the_controlled_seats_plays() {
        let mut g = crate::game::multi_player_game(3);
        let cast = GameAction::CastSpell { card_id: CardId(1), target: None, additional_targets: vec![], mode: None, x_value: None };
        assert!(controller_override(&g, 1, &cast).is_none());
        g.set_player_controller(1, Some(0));
        assert!(matches!(controller_override(&g, 1, &cast), Some(GameAction::PassPriority)));
        let attack = GameAction::DeclareAttackers(vec![crate::game::types::Attack {
            attacker: CardId(2),
            target: crate::game::types::AttackTarget::Player(2),
        }]);
        assert!(matches!(controller_override(&g, 1, &attack), Some(GameAction::DeclareAttackers(a)) if a.is_empty()));
        assert!(controller_override(&g, 1, &GameAction::PassPriority).is_none());
        assert!(controller_override(&g, 2, &cast).is_none());
    }
}
