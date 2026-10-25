//! "Choose one or more" where the modes cost the chooser life (Black Market
//! Connections: a Treasure for 1, a card for 2, a 3/2 for 3). The fallback
//! took the card's default set every turn whatever the life total, so a pod
//! seat paid its way to death. Keep the default modes while the payment
//! leaves more than [`LIFE_FLOOR`]; past it, drop the dearest first, keeping
//! one. Commander games only, so two-player play is unchanged.

use super::bot::self_life_loss;
use crate::card::CardId;
use crate::decision::DecisionAnswer;
use crate::effect::Effect;
use crate::game::GameState;

const LIFE_FLOOR: i32 = 10;

fn chosen_modes(e: &Effect, n: usize) -> Option<&[Effect]> {
    match e {
        Effect::ChooseN { modes, .. } if modes.len() == n => Some(modes),
        Effect::Seq(v) => v.iter().find_map(|x| chosen_modes(x, n)),
        _ => None,
    }
}

pub(super) fn trim_costly_modes(
    state: &GameState,
    seat: usize,
    source: CardId,
    num_modes: usize,
    default: &[u8],
) -> Option<DecisionAnswer> {
    if state.players.get(seat).is_none_or(|p| p.commanders.is_empty()) {
        return None;
    }
    let def = &state.find_card_anywhere(source)?.definition;
    let modes = std::iter::once(&def.effect)
        .chain(def.triggered_abilities.iter().map(|t| &t.effect))
        .find_map(|e| chosen_modes(e, num_modes))?;
    let cost = |i: u8| modes.get(i as usize).map_or(0, self_life_loss);
    let mut keep: Vec<u8> = default.to_vec();
    let life = state.effective_life(seat);
    while keep.len() > 1 && life - keep.iter().map(|&i| cost(i)).sum::<i32>() <= LIFE_FLOOR {
        let dearest = keep.iter().enumerate().max_by_key(|(k, i)| (cost(**i), *k)).map(|(k, _)| k)?;
        keep.remove(dearest);
    }
    (keep.len() < default.len()).then_some(DecisionAnswer::Modes(keep))
}

/// A "destroy all [filter]" mode's net for `seat`: the value it destroys on
/// opposing sides minus what it destroys on `seat`'s own. `None` for a mode
/// that isn't a whole-battlefield destroy.
fn wipe_net(state: &GameState, seat: usize, mode: &Effect) -> Option<i64> {
    use crate::effect::Selector;
    let (Effect::Destroy { what: Selector::EachPermanent(f) } | Effect::DestroyNoRegen { what: Selector::EachPermanent(f) }) = mode
    else {
        return None;
    };
    let w = super::bot::EvalWeights::default();
    Some(
        state
            .battlefield
            .iter()
            .filter(|c| state.evaluate_requirement_static_on(f, c, seat, None))
            .map(|c| {
                let v = i64::from(super::bot::permanent_value(state, c.id, &w).max(1));
                if state.same_team(c.controller, seat) { -v } else { v }
            })
            .sum(),
    )
}

/// "Choose one or both —" between board wipes (Invasion of Fiora: all
/// legendary / all nonlegendary creatures): drop each wipe mode that costs
/// `seat` more than it takes from the table, keeping the best one.
/// Commander games only, as above.
pub(super) fn trim_self_wipe_modes(
    state: &GameState,
    seat: usize,
    source: CardId,
    num_modes: usize,
    default: &[u8],
) -> Option<DecisionAnswer> {
    if state.players.get(seat).is_none_or(|p| p.commanders.is_empty()) || default.len() < 2 {
        return None;
    }
    let def = &state.find_card_anywhere(source)?.definition;
    let modes = std::iter::once(&def.effect)
        .chain(def.triggered_abilities.iter().map(|t| &t.effect))
        .find_map(|e| chosen_modes(e, num_modes))?;
    let nets: Vec<i64> = default.iter().map(|&i| modes.get(i as usize).and_then(|m| wipe_net(state, seat, m))).collect::<Option<_>>()?;
    let mut keep: Vec<u8> = default.iter().zip(&nets).filter(|(_, n)| **n > 0).map(|(i, _)| *i).collect();
    if keep.is_empty() {
        let best = nets.iter().enumerate().max_by_key(|(k, n)| (**n, std::cmp::Reverse(*k))).map(|(k, _)| k)?;
        keep.push(default[best]);
    }
    (keep.len() < default.len()).then_some(DecisionAnswer::Modes(keep))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    #[test]
    fn black_market_connections_stops_paying_near_the_floor() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![catalog::grizzly_bears()]);
        let bmc = g.add_card_to_battlefield(0, catalog::black_market_connections());
        assert_eq!(trim_costly_modes(&g, 0, bmc, 3, &[0, 1, 2]), None, "40 life: the default");
        g.players[0].life = 12;
        assert_eq!(trim_costly_modes(&g, 0, bmc, 3, &[0, 1, 2]), Some(DecisionAnswer::Modes(vec![0])), "the 3/2, then the card, go first");
    }

    /// Invasion of Fiora beside your own legend: only the nonlegendary wipe.
    #[test]
    fn invasion_of_fiora_spares_your_own_legend() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![catalog::grizzly_bears()]);
        g.add_card_to_battlefield(0, catalog::isamaru_hound_of_konda());
        g.add_card_to_battlefield(1, catalog::craw_wurm());
        g.add_card_to_battlefield(2, catalog::grizzly_bears());
        let inv = g.add_card_to_battlefield(0, catalog::invasion_of_fiora());
        assert_eq!(trim_self_wipe_modes(&g, 0, inv, 2, &[0, 1]), Some(DecisionAnswer::Modes(vec![1])));
    }
}
