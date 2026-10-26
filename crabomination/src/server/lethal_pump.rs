//! Pumping an unblocked attacker to lethal after blocks — "{1}{R}: this
//! creature gets +1/+0" (firebreathing, Hellkite Igniter), "{X}: +X/+0"
//! (Lavaclaw Reaches). No path spent mana this way once blockers were in: the
//! material eval prices a +N/+0 that lasts one turn at nothing, so a pod bot
//! sat on the mana that would have killed a player. It now pumps, one
//! activation per priority, while the unblocked damage at a player is short of
//! their life but the affordable pumps reach it — or, for a commander, short
//! of 21 commander damage (CR 704.6c / 903.10a). Commander games, and duels
//! under `EvalWeights::duel_pumps` — which also spends mana nothing in hand
//! could use, and stops a "+1/-1" pump at 1 toughness.

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

/// The toughness a fixed self-pump takes per activation (Undulating
/// Witness's "+1/-1": 1), zero for one that adds or keeps toughness.
fn toughness_cost(e: &Effect) -> i64 {
    match e {
        Effect::PumpPT { toughness: Value::Const(t), .. } if *t < 0 => i64::from(-*t),
        _ => 0,
    }
}

/// True when some nonland card in `seat`'s hand costs no more than
/// `budget`: mana that could still buy a spell this turn or the next
/// window isn't spare.
fn hand_could_use(state: &GameState, seat: usize, budget: i64) -> bool {
    state.players[seat]
        .hand
        .iter()
        .any(|c| !c.definition.is_land() && i64::from(c.definition.cost.cmc()) <= budget)
}

/// The next pump toward lethal for `seat`'s unblocked attackers, if one is
/// accepted.
pub(super) fn pick_lethal_pump(state: &GameState, seat: usize, w: &super::bot::EvalWeights) -> Option<GameAction> {
    let duel = w.duel_pumps && state.players.len() == 2;
    if (state.players[seat].commanders.is_empty() && !duel)
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
            // A pump that costs toughness stops at 1: past that it kills the
            // attacker before damage.
            let toughness = state.computed_permanent(attacker).map_or(1, |cp| i64::from(cp.toughness));
            let tcost = toughness_cost(&ab.effect);
            let affordable = if scales { 0 } else { budget / cmc };
            let activations = if tcost > 0 { affordable.min(((toughness - 1) / tcost).max(0)) } else { affordable };
            if !scales && activations == 0 {
                continue;
            }
            let reach = if scales {
                (budget - cmc).max(0)
            } else {
                activations * i64::from(per)
            };
            let at_q: i64 = unblocked.iter().filter(|(_, t)| *t == q).map(|(a, _)| i64::from(power(*a))).sum::<i64>();
            let life = i64::from(state.effective_life(q));
            let by_life = at_q < life && at_q + reach >= life;
            let by_commander = state.is_commander(attacker) && {
                let dealt = i64::from(state.commander_damage.get(&(q, attacker)).copied().unwrap_or(0));
                let now = dealt + i64::from(power(attacker));
                now < 21 && now + reach >= 21
            };
            // Duels: mana nothing in hand can use is spent on damage anyway.
            let spare = duel && !scales && !hand_could_use(state, seat, budget);
            if !by_life && !by_commander && !spare {
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
        assert!(pick_lethal_pump(&g, 0, &crate::server::bot::EvalWeights::default()).is_none(), "20 life: 5 + 1 isn't lethal");
        g.players[1].life = 6;
        assert!(matches!(pick_lethal_pump(&g, 0, &crate::server::bot::EvalWeights::default()), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == dragon));
        g.players[1].life = 5;
        assert!(pick_lethal_pump(&g, 0, &crate::server::bot::EvalWeights::default()).is_none(), "already lethal");
        // CR 704.6c — as a commander with 15 dealt to seat 1, 5 + 1 reaches 21.
        g.players[1].life = 40;
        g.players[0].commanders.push(dragon);
        g.commander_damage.insert((1, dragon), 15);
        assert!(matches!(pick_lethal_pump(&g, 0, &crate::server::bot::EvalWeights::default()), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == dragon));
    }

    /// `duel_pumps`: in a duel, an unblocked Undulating Witness ({2}: +1/-1,
    /// a 3/5) pumps with mana nothing in hand can use, never past 1
    /// toughness, and not at all without the flag or with a castable spell.
    #[test]
    fn a_duel_pumps_an_unblocked_witness_with_spare_mana() {
        let mut g = crate::game::two_player_game();
        g.active_player_idx = 0;
        let witness = g.add_card_to_battlefield(0, crate::catalog::undulating_witness());
        g.clear_sickness(witness);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: witness, target: AttackTarget::Player(1) }]))
            .expect("attack");
        crate::game::drain_stack(&mut g);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Blue, 2);
        let on = crate::server::bot::EvalWeights { duel_pumps: true, ..Default::default() };
        let off = crate::server::bot::EvalWeights::default();
        assert!(pick_lethal_pump(&g, 0, &off).is_none(), "duel, flag off");
        assert!(
            matches!(pick_lethal_pump(&g, 0, &on), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == witness),
            "spare mana, nothing to cast"
        );
        let bears = g.add_card_to_hand(0, crate::catalog::grizzly_bears());
        assert!(pick_lethal_pump(&g, 0, &on).is_none(), "the mana could cast Grizzly Bears");
        g.players[0].hand.retain(|c| c.id != bears);
        // Pumped to 7/1, a fifth +1/-1 would kill it.
        for _ in 0..4 {
            let act = GameAction::ActivateAbility {
                card_id: witness, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
            };
            g.players[0].mana_pool.add(Color::Blue, 2);
            g.perform_action(act).expect("pump");
            crate::game::drain_stack(&mut g);
        }
        g.players[0].mana_pool.add(Color::Blue, 2);
        assert!(pick_lethal_pump(&g, 0, &on).is_none(), "1 toughness left");
    }
}
