//! A creature's own "{cost}: this gains [keyword] until end of turn" in a
//! fight it is losing — Endling's {B}: deathtouch / undying, a first-strike
//! grant — was activated by no bot path (Mairsil, the Pretender census, seed
//! 9409). After blocks, the bot now buys the keyword that turns its side of a
//! blocked pair: deathtouch when it can't otherwise kill, first strike when it
//! kills but would die first, undying / indestructible when it would die.
//! Commander games only, so two-player play is unchanged.

use crate::card::{CounterType, Keyword};
use crate::effect::{Duration, Effect, Selector};
use crate::game::GameState;
use crate::game::types::GameAction;

/// After blocks, one self-grant activation that flips a fight `seat` is
/// losing in a blocked pair.
pub(super) fn pick_combat_self_grant(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    for (blocker, attacker) in state.block_map_snapshot() {
        let (Some(b), Some(a)) = (state.battlefield_find(blocker), state.battlefield_find(attacker)) else {
            continue;
        };
        let (ours, theirs) = if a.controller == seat && !state.same_team(b.controller, seat) {
            (a, b)
        } else if b.controller == seat && !state.same_team(a.controller, seat) {
            (b, a)
        } else {
            continue;
        };
        let (Some(o), Some(t)) = (state.computed_permanent(ours.id), state.computed_permanent(theirs.id)) else {
            continue;
        };
        let (okw, tkw) = (o.keywords(), t.keywords());
        let has = |kws: &[Keyword], k: &Keyword| kws.contains(k);
        let strikes_first = |kws: &[Keyword]| has(kws, &Keyword::FirstStrike) || has(kws, &Keyword::DoubleStrike);
        let kills = o.power > 0
            && !has(tkw, &Keyword::Indestructible)
            && (has(okw, &Keyword::Deathtouch) || o.power >= t.toughness - theirs.damage as i32);
        let dies = t.power > 0
            && !has(okw, &Keyword::Indestructible)
            && (has(tkw, &Keyword::Deathtouch) || t.power >= o.toughness - ours.damage as i32);
        for (index, ab) in ours.definition.activated_abilities.iter().enumerate() {
            let Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn | Duration::EndOfCombat } =
                &ab.effect
            else {
                continue;
            };
            if ab.tap_cost || has(okw, keyword) {
                continue;
            }
            let useful = match keyword {
                Keyword::Deathtouch => !kills && o.power > 0 && !has(tkw, &Keyword::Indestructible),
                Keyword::FirstStrike | Keyword::DoubleStrike => kills && dies && !strikes_first(tkw),
                Keyword::Indestructible => dies,
                Keyword::Undying => dies && ours.counter_count(CounterType::PlusOnePlusOne) == 0,
                _ => false,
            };
            if !useful {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: ours.id,
                ability_index: index,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
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
    use crate::game::types::{Attack, AttackTarget, TurnStep};

    /// Endling blocked by a bigger creature buys deathtouch.
    #[test]
    fn endling_buys_deathtouch_against_a_bigger_blocker() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        let endling = g.add_card_to_battlefield(0, crate::catalog::endling());
        let wall = g.add_card_to_battlefield(1, crate::catalog::serra_angel());
        g.clear_sickness(endling);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: endling, target: AttackTarget::Player(1) }]))
            .expect("attack");
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(wall, endling)])).expect("block");
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        let pick = pick_combat_self_grant(&g, 0);
        assert!(matches!(pick, Some(GameAction::ActivateAbility { card_id, ability_index: 1, .. }) if card_id == endling), "{pick:?}");
    }
}
