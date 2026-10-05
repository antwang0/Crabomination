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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    #[test]
    fn black_market_connections_stops_paying_near_the_floor() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![catalog::grizzly_bears()]);
        let bmc = g.add_card_to_battlefield(0, catalog::black_market_connections());
        assert_eq!(trim_costly_modes(&g, 0, bmc, 3, &[0, 1]), None, "40 life: the default");
        g.players[0].life = 12;
        assert_eq!(trim_costly_modes(&g, 0, bmc, 3, &[0, 1]), Some(DecisionAnswer::Modes(vec![0])), "the card goes first");
    }
}
