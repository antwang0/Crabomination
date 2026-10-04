//! Pods only: "{cost}, exile (or sacrifice) this: return target card from
//! your graveyard to your hand" (Nyx Weaver). The outcome score trades a body
//! for a card about even, so a 183-deck census never saw one activated. With
//! the hand nearly empty ([`MAX_HAND`]) at the end step before the seat's
//! turn, a small body (or a noncreature) buys back the costliest nonland
//! card.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameAction, Target, TurnStep};

/// Hand size at or under which a card back is worth a permanent.
const MAX_HAND: usize = 1;
/// The most power a creature source may have and still be spent.
const MAX_SOURCE_POWER: i32 = 2;

fn regrows_to_hand(e: &Effect) -> bool {
    matches!(
        e,
        Effect::Move {
            what: Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. },
            to: ZoneDest::Hand(PlayerRef::You),
        }
    )
}

pub(super) fn pick_regrowth(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players.len() <= 2
        || state.step != TurnStep::End
        || state.active_player_idx == seat
        || state.next_alive_seat(state.active_player_idx) != seat
        || !state.stack.is_empty()
        || state.players[seat].hand.len() > MAX_HAND
    {
        return None;
    }
    let mut picks: Vec<(u32, crate::card::CardId)> = state.players[seat]
        .graveyard
        .iter()
        .filter(|c| !c.is_token && !c.definition.is_land())
        .map(|c| (c.definition.cost.cmc(), c.id))
        .collect();
    picks.sort_by_key(|&(mv, id)| (std::cmp::Reverse(mv), id));
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        let small = state
            .computed_permanent(c.id)
            .is_none_or(|cp| !state.computed_is_creature(c) || cp.power <= MAX_SOURCE_POWER);
        if !small {
            return None;
        }
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            if !(ab.exile_self_cost || ab.sac_cost) || !regrows_to_hand(&ab.effect) {
                return None;
            }
            picks.iter().filter(|&&(_, id)| id != c.id).find_map(|&(_, id)| {
                let action = GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target: Some(Target::Permanent(id)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                state.would_accept(action.clone()).then_some(action)
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nyx Weaver exiles itself for the costliest nonland card when the hand
    /// is empty, and not with a full hand.
    #[test]
    fn nyx_weaver_buys_back_on_an_empty_hand() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 2;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let weaver = g.add_card_to_battlefield(0, crate::catalog::nyx_weaver());
        g.add_card_to_graveyard(0, crate::catalog::grizzly_bears());
        let wurm = g.add_card_to_graveyard(0, crate::catalog::craw_wurm());
        g.add_card_to_graveyard(0, crate::catalog::forest());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
        assert!(matches!(
            pick_regrowth(&g, 0),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == weaver && t == wurm
        ));
        for _ in 0..3 {
            g.add_card_to_hand(0, crate::catalog::island());
        }
        assert!(pick_regrowth(&g, 0).is_none(), "a full hand keeps the Weaver");
    }
}
