//! Bot answers for "any player may [pay]. If a player does, …; otherwise …"
//! (`Effect::PlayersMayAccept`: Browbeat, Vexing Devil, Risk Factor, Argothian
//! Wurm). The generic "may" screen finds no `MayDo` body for it and takes
//! every offer, so both bots of a cube pair at 2 and 4 life took Browbeat's
//! 5 damage and drew the game (seed 730001, six of its seven draws).

use crate::card::CardId;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::GameState;

/// Life a yes must leave standing: an accept is denial, not a race.
const ACCEPT_LIFE_BUFFER: i32 = 10;

/// `Some(answer)` when `source`'s ask is a `PlayersMayAccept` offer. The
/// offerer turns down its own offer (its card's payoff is the "otherwise");
/// another seat pays a life or damage cost only with
/// [`ACCEPT_LIFE_BUFFER`] to spare. A cost the walk can't price falls back
/// to the generic screen (`None`).
pub(super) fn accept_offer_answer(state: &GameState, seat: usize, source: CardId, description: &str) -> Option<bool> {
    let (def, offerer) = state
        .battlefield
        .find_by_id(source)
        .map(|c| (&**c.definition, c.controller))
        .or_else(|| {
            state.stack.iter().find_map(|si| match si {
                crate::game::types::StackItem::Spell { card, .. } if card.id == source => {
                    Some((&**card.definition, card.controller))
                }
                _ => None,
            })
        })
        // A resolving spell is held off-zone in the paused resume.
        .or_else(|| match state.pending_decision.as_ref().map(|d| &d.resume) {
            Some(crate::game::types::ResumeContext::Spell { card, caster, .. }) if card.id == source => {
                Some((&**card.definition, *caster))
            }
            _ => None,
        })?;
    let on_accept = std::iter::once(&def.effect)
        .chain(def.triggered_abilities.iter().map(|t| &t.effect))
        .find_map(|e| offer_body(e, description))?;
    if seat == offerer {
        return Some(false);
    }
    let cost = accepter_life_cost(on_accept);
    (cost > 0).then(|| state.effective_life(seat) - cost > ACCEPT_LIFE_BUFFER)
}

/// The `on_accept` of the offer prompting `desc`, through `Seq` / `If`.
fn offer_body<'a>(e: &'a Effect, desc: &str) -> Option<&'a Effect> {
    match e {
        Effect::PlayersMayAccept { description, on_accept, .. } if description == desc => Some(on_accept),
        Effect::Seq(v) => v.iter().find_map(|x| offer_body(x, desc)),
        Effect::If { then, else_, .. } => offer_body(then, desc).or_else(|| offer_body(else_, desc)),
        _ => None,
    }
}

/// Damage or life loss the accepter (slot 0) takes for a yes.
fn accepter_life_cost(e: &Effect) -> i32 {
    let accepter = |s: &Selector| matches!(s, Selector::Target(0) | Selector::Player(PlayerRef::Target(0)));
    match e {
        Effect::DealDamage { to, amount: Value::Const(n) } | Effect::LoseLife { who: to, amount: Value::Const(n) }
            if accepter(to) =>
        {
            (*n).max(0)
        }
        Effect::Seq(v) => v.iter().map(accepter_life_cost).sum(),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::types::{GameAction, Target};

    /// The bots of a cube pair took Browbeat's 5 at 2 and 4 life and drew
    /// the game: the caster turns its own offer down, an opponent takes it
    /// only with life to spare.
    #[test]
    fn browbeat_is_taken_only_with_life_to_spare() {
        let mut g = crate::game::two_player_game();
        g.players[0].wants_ui = true;
        let bb = g.add_card_to_hand(0, catalog::browbeat());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: bb,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        g.players[1].wants_ui = true;
        g.resolve_top_of_stack().expect("resolve");
        assert!(g.stack.is_empty() && g.pending_decision.is_some(), "asked mid-resolution");
        let ask = "Have Browbeat deal 5 damage to you?";
        assert_eq!(accept_offer_answer(&g, 0, bb, ask), Some(false), "the caster wants the draw");
        g.players[1].life = 20;
        assert_eq!(accept_offer_answer(&g, 1, bb, ask), Some(true), "20 life buys the denial");
        g.players[1].life = 4;
        assert_eq!(accept_offer_answer(&g, 1, bb, ask), Some(false), "4 life does not");
    }
}
