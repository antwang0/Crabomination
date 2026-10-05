//! CR 601.2d — dividing a spell's or ability's damage among its targets
//! (Inferno Titan, Fireball, Fire // Ice). The fallback split it evenly, which
//! ignores toughness: Inferno Titan's 3 over three targets killed nothing
//! bigger than an X/1. Each target takes the required 1; the rest goes to the
//! cheapest lethal on the most valuable enemy permanents, then to an opponent,
//! and never past what a creature needs. Commander games only, so two-player
//! play is unchanged.

use super::bot::{EvalWeights, permanent_value};
use crate::card::{CounterType, Keyword};
use crate::decision::DecisionAnswer;
use crate::game::GameState;
use crate::game::types::Target;

/// Damage that still kills `t` beyond what it already has, or `None` for a
/// player or a permanent damage won't kill (indestructible).
fn lethal_need(state: &GameState, t: &Target) -> Option<u32> {
    let Target::Permanent(id) = t else { return None };
    let c = state.battlefield_find(*id)?;
    let cp = state.computed_permanent_on(c)?;
    if cp.keywords().contains(&Keyword::Indestructible) {
        return None;
    }
    if cp.card_types().contains(&crate::card::CardType::Creature) {
        return Some((state.effective_toughness_on(c) - c.damage as i32).max(1) as u32);
    }
    if cp.card_types().contains(&crate::card::CardType::Planeswalker) {
        return Some(c.counter_count(CounterType::Loyalty).max(1));
    }
    None
}

pub(super) fn divide_damage(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    total: u32,
    targets: &[Target],
) -> Option<DecisionAnswer> {
    let n = targets.len();
    if state.players.get(seat).is_none_or(|p| p.commanders.is_empty()) || n == 0 || (total as usize) < n {
        return None;
    }
    let enemy = |t: &Target| match t {
        Target::Player(p) => !state.same_team(*p, seat),
        Target::Permanent(id) => state.battlefield_find(*id).is_some_and(|c| !state.same_team(c.controller, seat)),
    };
    let mut split = vec![1u32; n];
    let mut left = total - n as u32;
    // Enemy permanents, best value per extra point of lethal first.
    let mut kills: Vec<(usize, u32, i32)> = targets
        .iter()
        .enumerate()
        .filter(|(_, t)| enemy(t))
        .filter_map(|(i, t)| {
            let need = lethal_need(state, t)?;
            let id = match t {
                Target::Permanent(id) => *id,
                Target::Player(_) => return None,
            };
            Some((i, need.saturating_sub(1), permanent_value(state, id, w)))
        })
        .collect();
    kills.sort_by(|a, b| {
        let (va, vb) = (a.2 as i64 * (b.1 as i64 + 1), b.2 as i64 * (a.1 as i64 + 1));
        vb.cmp(&va).then(a.0.cmp(&b.0))
    });
    for &(i, extra, _) in &kills {
        if extra <= left {
            split[i] += extra;
            left -= extra;
        }
    }
    // The rest to an opponent's face, else the most valuable enemy permanent,
    // else wherever it can go.
    let sink = targets
        .iter()
        .position(|t| matches!(t, Target::Player(_)) && enemy(t))
        .or_else(|| kills.first().map(|k| k.0))
        .or_else(|| targets.iter().position(enemy))
        .unwrap_or(0);
    split[sink] += left;
    Some(DecisionAnswer::DamageDivision(split))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    /// Inferno Titan's 3 over a 2/2, a 1/1 and an opponent: lethal on both
    /// creatures (2 + 1), nothing wasted on the face.
    #[test]
    fn lethal_first_beats_an_even_split() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![catalog::grizzly_bears()]);
        let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
        let targets = [Target::Permanent(bears), Target::Permanent(elf), Target::Player(2)];
        assert_eq!(
            divide_damage(&g, 0, &EvalWeights::default(), 4, &targets),
            Some(DecisionAnswer::DamageDivision(vec![2, 1, 1]))
        );
        assert_eq!(
            divide_damage(&g, 0, &EvalWeights::default(), 6, &targets),
            Some(DecisionAnswer::DamageDivision(vec![2, 1, 3])),
            "the surplus goes to the opponent"
        );
        let two_seat = crate::game::multi_player_game(2);
        assert_eq!(divide_damage(&two_seat, 0, &EvalWeights::default(), 3, &[Target::Player(1)]), None, "not a Commander seat");
    }
}
