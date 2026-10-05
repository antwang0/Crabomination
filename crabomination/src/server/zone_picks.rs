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

/// A Commander seat choosing which of an opponent's cards they discard: the
/// nonland cards first, priciest first (the fallback took the first N).
pub(super) fn pick_from_opponents_hand(
    state: &GameState,
    seat: usize,
    hand: &[(CardId, String)],
    count: usize,
) -> Option<DecisionAnswer> {
    if state.players.get(seat).is_none_or(|p| p.commanders.is_empty()) {
        return None;
    }
    let theirs = |id: CardId| {
        state.players.iter().enumerate().find_map(|(i, p)| (i != seat).then(|| p.hand.iter().find(|c| c.id == id)).flatten())
    };
    let cards: Vec<&CardInstance> = hand.iter().map(|(id, _)| theirs(*id)).collect::<Option<_>>()?;
    let mut ranked: Vec<(CardId, bool, u32)> =
        cards.iter().map(|c| (c.id, !c.definition.is_land(), c.definition.cost.cmc())).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
    Some(DecisionAnswer::Discard(ranked.into_iter().take(count).map(|(id, ..)| id).collect()))
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

    fn ask(cands: &[CardId], max: u32, eligible: Option<Vec<CardId>>) -> crate::decision::Decision {
        crate::decision::Decision::ChooseCards {
            source: CardId(0),
            prompt: String::new(),
            candidates: cands.iter().map(|&id| (id, String::new())).collect(),
            min: 0,
            max,
            eligible,
            value: crate::decision::PickValue::Gain,
        }
    }

    /// Only `eligible` cards are picks: the bot used to rank all the revealed
    /// cards, offer ineligible ones, and lose them (an optional pick is final).
    #[test]
    fn a_gain_pick_keeps_to_the_eligible_cards() {
        let mut g = crate::game::multi_player_game(2);
        let wurm = g.add_card_to_library(0, catalog::craw_wurm());
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
        let bears = g.add_card_to_library(0, catalog::grizzly_bears());
        let w = super::super::bot::EvalWeights::default();
        let d = ask(&[wurm, bolt, bears], 1, Some(vec![bolt, bears]));
        assert_eq!(super::super::bot::decide_pending_policy(&g, 0, &w, &d, false), DecisionAnswer::Cards(vec![bears]));
    }

    /// A gain from an OPPONENT's library (Gonti) takes the best card, not the
    /// top one.
    #[test]
    fn a_gain_pick_from_an_opponents_library_takes_the_best() {
        let mut g = crate::game::multi_player_game(2);
        let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
        let wurm = g.add_card_to_library(1, catalog::craw_wurm());
        let w = super::super::bot::EvalWeights::default();
        let mut d = ask(&[bolt, wurm], 1, None);
        if let crate::decision::Decision::ChooseCards { min, .. } = &mut d {
            *min = 1;
        }
        assert_eq!(super::super::bot::decide_pending_policy(&g, 0, &w, &d, false), DecisionAnswer::Cards(vec![wurm]));
    }

    /// Picking an opponent's discard takes their priciest spell.
    #[test]
    fn an_opponents_discard_loses_their_best_spell() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![catalog::grizzly_bears()]);
        let land = g.add_card_to_hand(1, catalog::forest());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        let wurm = g.add_card_to_hand(1, catalog::craw_wurm());
        let hand: Vec<(CardId, String)> = [land, bolt, wurm].iter().map(|&id| (id, String::new())).collect();
        assert_eq!(pick_from_opponents_hand(&g, 0, &hand, 1), Some(DecisionAnswer::Discard(vec![wurm])));
    }
}
