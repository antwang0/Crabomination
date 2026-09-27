//! Pumping an unblocked attacker to lethal after blocks — "{1}{R}: this
//! creature gets +1/+0" (firebreathing, Hellkite Igniter), "{X}: +X/+0"
//! (Lavaclaw Reaches). No path spent mana this way once blockers were in: the
//! material eval prices a +N/+0 that lasts one turn at nothing, so a pod bot
//! sat on the mana that would have killed a player. It now pumps, one
//! activation per priority, while the unblocked damage at a player is short of
//! their life but the affordable pumps reach it — or, for a commander, short
//! of 21 commander damage (CR 704.6c / 903.10a). Commander games only.

use crate::card::CardId;
use crate::effect::{Effect, Selector, Value};
use crate::game::GameState;
use crate::game::types::{AttackTarget, GameAction, TurnStep};

use super::bot::mana_upper_bound;

/// A self-pump's power per activation, and whether it scales with X.
fn self_pump(e: &Effect) -> Option<(i32, bool)> {
    match e {
        Effect::PumpPT { what: Selector::This, power: Value::Const(n), .. } if *n > 0 => Some((*n, false)),
        Effect::PumpPT { what: Selector::This, power: Value::XFromCost, .. } => Some((1, true)),
        _ => None,
    }
}

/// The next pump toward lethal for `seat`'s unblocked attackers, if one is
/// accepted.
pub(super) fn pick_lethal_pump(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty()
        || state.step != TurnStep::DeclareBlockers
        || state.active_player_idx != seat
        || !state.stack.is_empty()
    {
        return None;
    }
    let power = |id: CardId| state.computed_permanent(id).map_or(0, |cp| cp.power.max(0));
    let unblocked: Vec<(CardId, usize)> = state
        .attacking
        .iter()
        .filter(|a| !state.blocked_attackers.contains(&a.attacker))
        .filter(|a| state.battlefield_find(a.attacker).is_some_and(|c| c.controller == seat))
        .filter_map(|a| match a.target {
            AttackTarget::Player(q) if state.players[q].is_alive() => Some((a.attacker, q)),
            _ => None,
        })
        .collect();
    if unblocked.is_empty() {
        return None;
    }
    let budget = mana_upper_bound(state, seat) as i64;
    for &(attacker, q) in &unblocked {
        let Some(card) = state.battlefield_find(attacker) else { continue };
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Some((per, scales)) = self_pump(&ab.effect) else { continue };
            if ab.tap_cost || ab.sac_cost {
                continue;
            }
            let cmc = i64::from(ab.mana_cost.cmc());
            // A free repeatable pump would be taken every tick; none is printed.
            if cmc == 0 && !scales {
                continue;
            }
            let reach = if scales {
                (budget - cmc).max(0)
            } else {
                (budget / cmc) * i64::from(per)
            };
            let at_q: i64 = unblocked.iter().filter(|(_, t)| *t == q).map(|(a, _)| i64::from(power(*a))).sum::<i64>();
            let life = i64::from(state.effective_life(q));
            let by_life = at_q < life && at_q + reach >= life;
            let by_commander = state.is_commander(attacker) && {
                let dealt = i64::from(state.commander_damage.get(&(q, attacker)).copied().unwrap_or(0));
                let now = dealt + i64::from(power(attacker));
                now < 21 && now + reach >= 21
            };
            if !by_life && !by_commander {
                continue;
            }
            let x = scales.then(|| {
                let need = if by_life { life - at_q } else { 21 - i64::from(power(attacker)) };
                need.clamp(1, (budget - cmc).max(1)) as u32
            });
            let action = GameAction::ActivateAbility {
                card_id: attacker,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: x,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::Attack;
    use crate::mana::Color;

    /// An unblocked Shivan Dragon (5 power, {R}: +1/+0) with one red up pumps
    /// when the defender is at 6, not at 20 (keeps the mana) and not at 5
    /// (already lethal).
    #[test]
    fn firebreathing_pumps_only_to_lethal() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        g.active_player_idx = 0;
        let dragon = g.add_card_to_battlefield(0, crate::catalog::shivan_dragon());
        g.clear_sickness(dragon);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: dragon, target: AttackTarget::Player(1) }]))
            .expect("attack");
        crate::game::drain_stack(&mut g);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 1);
        assert!(pick_lethal_pump(&g, 0).is_none(), "20 life: 5 + 1 isn't lethal");
        g.players[1].life = 6;
        assert!(matches!(pick_lethal_pump(&g, 0), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == dragon));
        g.players[1].life = 5;
        assert!(pick_lethal_pump(&g, 0).is_none(), "already lethal");
        // CR 704.6c — as a commander with 15 dealt to seat 1, 5 + 1 reaches 21.
        g.players[1].life = 40;
        g.players[0].commanders.push(dragon);
        g.commander_damage.insert((1, dragon), 15);
        assert!(matches!(pick_lethal_pump(&g, 0), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == dragon));
    }
}
