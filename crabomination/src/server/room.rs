//! CR 709.5 — no bot path unlocked a Room's second door or cast its right
//! door, so a Room played as its left half for the whole game. The bot now
//! spends idle main-phase mana on a door: unlock one on the battlefield, else
//! cast a Room from hand for the door the plain cast (always left) can't.
//! Commander games only, so two-player play is unchanged.

use crate::game::GameState;
use crate::game::types::GameAction;

/// The most expensive door `seat` can unlock (CR 709.5e) or, failing that,
/// cast (CR 709.5) right now. `None` without a Room in play or in hand.
pub(super) fn pick_room_door(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    let door_cmc = |c: &crate::card::CardInstance, right: bool| {
        c.definition.room.as_deref().map_or(0, |r| if right { &r.right } else { &r.left }.cost.cmc())
    };
    let unlock = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.room.is_some())
        .flat_map(|c| {
            [(1u8, false), (2, true)]
                .into_iter()
                .filter(|(bit, _)| c.unlocked_doors & bit == 0)
                .map(move |(_, right)| (door_cmc(c, right), GameAction::UnlockRoomDoor { card_id: c.id, right }))
        });
    let cast = state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.room.is_some())
        .flat_map(|c| {
            [false, true]
                .into_iter()
                .map(move |right| (door_cmc(c, right), GameAction::CastRoomDoor { card_id: c.id, right }))
        });
    let mut options: Vec<(u32, GameAction)> = unlock.collect();
    if options.is_empty() {
        options.extend(cast);
    }
    // Stable sort: equal-cost doors keep battlefield/hand order, left first.
    options.sort_by_key(|(cmc, _)| std::cmp::Reverse(*cmc));
    options.into_iter().map(|(_, a)| a).find(|a| state.would_accept(a.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    fn main_phase() -> GameState {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g
    }

    /// CR 709.5e — a Room on the battlefield with a locked door gets it
    /// unlocked with idle mana.
    #[test]
    fn a_locked_door_is_unlocked_with_idle_mana() {
        let mut g = main_phase();
        let room = g.add_card_to_battlefield(0, crate::catalog::spiked_corridor_torture_pit());
        g.set_room_door_unlocked(room, true, &mut Vec::new());
        assert!(pick_room_door(&g, 0).is_none(), "no mana, no unlock");
        g.players[0].mana_pool.add(Color::Red, 4);
        let picked = pick_room_door(&g, 0);
        assert!(matches!(
            picked,
            Some(GameAction::UnlockRoomDoor { card_id, right: false }) if card_id == room
        ));
    }

    /// CR 709.5 — with a Room in hand and nothing in play, the pick is a door cast.
    #[test]
    fn a_room_in_hand_is_cast_for_a_door() {
        let mut g = main_phase();
        let room = g.add_card_to_hand(0, crate::catalog::spiked_corridor_torture_pit());
        g.players[0].mana_pool.add(Color::Red, 4);
        assert!(matches!(
            pick_room_door(&g, 0),
            Some(GameAction::CastRoomDoor { card_id, .. }) if card_id == room
        ));
    }
}
