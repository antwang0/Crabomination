//! A `Gain` card pick over cards in exile or the command zone (Jeleva's "you
//! may cast an instant or sorcery exiled with her", Skyway Robber, Vault 13's
//! returns, Next of Kin's hand-or-command put). `decide_choose_cards` reads
//! only the hand, the seat's library, the battlefield and graveyards, so these
//! answered with nothing — every free cast and return declined.

use crate::card::{CardId, CardInstance};
use crate::decision::DecisionAnswer;
use crate::game::GameState;

fn in_exile_hand_or_command(state: &GameState, id: CardId) -> Option<&CardInstance> {
    state
        .exile
        .iter()
        .find(|c| c.id == id)
        .or_else(|| state.players.iter().find_map(|p| p.command.iter().chain(p.hand.iter()).find(|c| c.id == id)))
}

/// The biggest cards up to `max`, when every candidate is in exile, a hand or
/// a command zone and at least one is in exile or a command zone (a pure
/// hand pick has its own branch). `None` leaves the pick to the caller.
pub(super) fn pick_from_exile_or_command(
    state: &GameState,
    candidates: &[(CardId, String)],
    max: u32,
) -> Option<DecisionAnswer> {
    let cards: Vec<&CardInstance> =
        candidates.iter().map(|(id, _)| in_exile_hand_or_command(state, *id)).collect::<Option<_>>()?;
    let off_hand = |c: &CardInstance| !state.players.iter().any(|p| p.hand.iter().any(|h| h.id == c.id));
    if cards.is_empty() || !cards.iter().any(|c| off_hand(c)) {
        return None;
    }
    let mut ranked: Vec<(CardId, u32, i32)> =
        cards.iter().map(|c| (c.id, c.definition.cost.cmc(), c.definition.power)).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
    Some(DecisionAnswer::Cards(ranked.into_iter().take(max as usize).map(|(id, ..)| id).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    /// A free pick from exile takes the biggest card, not nothing.
    #[test]
    fn a_gain_pick_from_exile_takes_the_biggest() {
        let mut g = crate::game::multi_player_game(3);
        let bolt = g.add_card_to_battlefield(0, catalog::lightning_bolt());
        let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
        for id in [bolt, wurm] {
            g.remove_from_battlefield_to_exile(id);
        }
        let cands = vec![(bolt, String::new()), (wurm, String::new())];
        assert_eq!(pick_from_exile_or_command(&g, &cands, 1), Some(DecisionAnswer::Cards(vec![wurm])));
        let hand = g.add_card_to_hand(0, catalog::grizzly_bears());
        assert_eq!(pick_from_exile_or_command(&g, &[(hand, String::new())], 1), None, "a pure hand pick is not ours");
    }
}
