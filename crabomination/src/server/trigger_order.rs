//! CR 603.3b — ordering one's own simultaneous triggers. A drain resolves
//! before a token maker: Eloise, Nephalia Sleuth under March of the
//! Machines investigates into a 0/0 Clue that dies and investigates again,
//! and with the default order every Bastion of Remembrance drain stayed
//! buried under the next Eloise trigger until the action cap (4-seat
//! `--pod-decks 1174,1170,1172,1176 --seed 11128 --first 9`). Drains first
//! end that game. Commander games only, so two-player play is unchanged.

use crate::card::CardId;
use crate::decision::DecisionAnswer;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::GameState;
use crate::game::types::ResumeContext;

fn hurts_opponents(e: &Effect) -> bool {
    match e {
        Effect::Drain { .. } | Effect::DealDamage { .. } => true,
        Effect::LoseLife { who, .. } => !matches!(who, Selector::You | Selector::Player(PlayerRef::You)),
        Effect::Seq(v) => v.iter().any(hurts_opponents),
        Effect::If { then, else_, .. } => hurts_opponents(then) || hurts_opponents(else_),
        _ => false,
    }
}

fn makes_tokens(e: &Effect) -> bool {
    match e {
        Effect::CreateToken { .. } | Effect::Investigate { .. } => true,
        Effect::Seq(v) => v.iter().any(makes_tokens),
        Effect::If { then, else_, .. } => makes_tokens(then) || makes_tokens(else_),
        _ => false,
    }
}

/// The push order (last resolves first) putting opponent-hurting triggers
/// on top and token makers at the bottom — `None` to keep the default.
pub(super) fn order(state: &GameState, seat: usize) -> Option<DecisionAnswer> {
    if state.players.get(seat).is_none_or(|p| p.commanders.is_empty()) {
        return None;
    }
    let pd = state.pending_decision.as_ref()?;
    let ResumeContext::TriggerOrder { run, .. } = &pd.resume else { return None };
    let score = |e: &Effect| i32::from(hurts_opponents(e)) * 2 - i32::from(makes_tokens(e));
    let scores: Vec<i32> = run.iter().map(|c| score(&c.effect)).collect();
    if scores.iter().all(|s| *s == scores[0]) {
        return None;
    }
    let mut idx: Vec<usize> = (0..run.len()).collect();
    idx.sort_by_key(|&i| scores[i]);
    Some(DecisionAnswer::TriggerOrder(idx.into_iter().map(|i| run[i].source).collect::<Vec<CardId>>()))
}

#[cfg(test)]
mod tests {
    use crate::decision::DecisionAnswer;
    use crate::game::types::{GameAction, Target, TurnStep};

    /// CR 603.3b — a creature dying under Eloise and Bastion of Remembrance:
    /// the Bastion drain is pushed last, so it resolves first.
    #[test]
    fn a_drain_resolves_before_a_token_maker() {
        let mut g = crate::game::multi_player_game(2);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[0].wants_ui = true;
        g.add_card_to_battlefield(0, crate::catalog::eloise_nephalia_sleuth());
        let bastion = g.add_card_to_battlefield(0, crate::catalog::bastion_of_remembrance());
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(1, crate::catalog::lightning_bolt());
        g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        for _ in 0..4 {
            if g.pending_decision.is_some() {
                break;
            }
            let _ = g.perform_action(GameAction::PassPriority);
        }
        let pd = g.pending_decision.clone().expect("an order ask");
        let w = crate::server::bot::EvalWeights::default();
        let DecisionAnswer::TriggerOrder(order) = crate::server::bot::decide_pending_policy(&g, 0, &w, &pd.decision, false)
        else {
            panic!("not an order answer")
        };
        assert_eq!(order.last(), Some(&bastion), "the drain is pushed last: {order:?}");
    }
}
