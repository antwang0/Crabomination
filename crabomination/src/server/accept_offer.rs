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

/// `Some(answer)` for a "you may [draw]; if you do, you can't attack this turn"
/// ask another seat's permanent puts to the active player (`MayDoBy` with a
/// `CantAttackThisTurn` body — Peace in Our Time). The generic screen sees no
/// self-cost and takes every draw, so no one ever attacked into it. The
/// source's controller takes it (its counter is their win); anyone else takes
/// it only with no creature ready to attack. A "may exile your spell, or else"
/// ask (Brazen Bluff) is declined: the seat keeps its spell.
pub(super) fn stand_down_answer(state: &GameState, seat: usize, source: CardId, description: &str) -> Option<bool> {
    fn stands_down(e: &Effect, desc: &str) -> bool {
        match e {
            Effect::MayDoBy { description, body, .. } if description == desc => {
                matches!(&**body, Effect::Seq(v) if v.iter().any(|x| matches!(x, Effect::CantAttackThisTurn { .. })))
            }
            Effect::Seq(v) => v.iter().any(|x| stands_down(x, desc)),
            _ => false,
        }
    }
    // "That player may exile [their spell]. If they don't, you may cast …"
    // (Brazen Bluff): the asked seat keeps its spell.
    fn gives_up_spell(e: &Effect, desc: &str) -> bool {
        match e {
            Effect::MayDoBy { description, body, .. } if description == desc => {
                matches!(&**body, Effect::MoveSpellToZone { .. })
            }
            Effect::Seq(v) => v.iter().any(|x| gives_up_spell(x, desc)),
            _ => false,
        }
    }
    let on_stack = state.stack.iter().find_map(|si| match si {
        crate::game::types::StackItem::Spell { card, .. } if card.id == source => Some(&**card.definition),
        _ => None,
    });
    let resolving = match state.pending_decision.as_ref().map(|d| &d.resume) {
        Some(crate::game::types::ResumeContext::Spell { card, .. }) if card.id == source => Some(&**card.definition),
        _ => None,
    };
    if on_stack.or(resolving).is_some_and(|d| gives_up_spell(&d.effect, description)) {
        return Some(false);
    }
    let c = state.battlefield.find_by_id(source)?;
    if !c.definition.triggered_abilities.iter().any(|t| stands_down(&t.effect, description)) {
        return None;
    }
    if seat == c.controller {
        return Some(true);
    }
    let ready = state.battlefield.iter().any(|p| {
        p.controller == seat
            && !p.tapped
            && state.computed_permanent(p.id).is_some_and(|cp| {
                use crate::card::Keyword;
                let kws = cp.keywords();
                cp.card_types().contains(&crate::card::CardType::Creature)
                    && cp.power > 0
                    && !kws.contains(&Keyword::Defender)
                    && (!p.summoning_sick || kws.contains(&Keyword::Haste))
            })
    });
    Some(!ready)
}

/// Miss Highwater's offer — "discard your hand, draw seven, take a contract
/// counter". The generic screen prices the draw and not the contract, so it
/// took every wheel. Taken only from a hand of two or fewer cards with a
/// library deep enough that seven draws can't deck the seat.
pub(super) fn contract_answer(state: &GameState, seat: usize, source: CardId, description: &str) -> Option<bool> {
    fn contracts(e: &Effect, desc: &str) -> bool {
        match e {
            Effect::MayDoBy { description, body, .. } if description == desc => {
                matches!(&**body, Effect::Seq(v) if v.iter().any(|x| matches!(x, Effect::GiveContractCounter { .. })))
            }
            Effect::Seq(v) => v.iter().any(|x| contracts(x, desc)),
            _ => false,
        }
    }
    let c = state.battlefield.find_by_id(source)?;
    if !c.definition.triggered_abilities.iter().any(|t| contracts(&t.effect, description)) {
        return None;
    }
    let p = state.players.get(seat)?;
    Some(p.hand.len() <= 2 && p.library.len() >= 14)
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
