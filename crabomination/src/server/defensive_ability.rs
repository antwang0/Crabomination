//! A defender's removal *ability* aimed at an attacker (Soul Snare's "{W},
//! sacrifice: exile target creature that's attacking you") — the defensive
//! removal picker reads instants in hand only, so a 183-deck `--card-census
//! --a dflt` never activated Soul Snare in any of its seven decks. Commander
//! games only, so two-player play is unchanged.

use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

use super::bot::{EvalWeights, permanent_value, ward_gate_ok};

/// A targeted destroy or exile as the ability's first step.
fn removes_target(e: &Effect) -> bool {
    match e {
        Effect::Destroy { what } | Effect::DestroyNoRegen { what } | Effect::Exile { what } => {
            matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
        }
        Effect::Seq(v) => v.first().is_some_and(removes_target),
        _ => false,
    }
}

/// The first accepted removal activation on the most valuable creature
/// attacking `seat` that is worth a card.
pub(super) fn pick_defensive_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    let abilities: Vec<(crate::card::CardId, usize)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .filter(|(_, ab)| removes_target(&ab.effect) && !ab.from_hand && !ab.from_graveyard)
                .map(move |(i, _)| (c.id, i))
        })
        .collect();
    if abilities.is_empty() {
        return None;
    }
    let mut attackers: Vec<(i32, crate::card::CardId)> = state
        .attacking()
        .iter()
        .filter(|a| state.defender_for(a.target) == Some(seat))
        .map(|a| (permanent_value(state, a.attacker, w), a.attacker))
        .filter(|&(v, _)| v >= 6 * w.unit)
        .collect();
    attackers.sort_by_key(|&(v, id)| (std::cmp::Reverse(v), id));
    attackers.iter().find_map(|&(_, atk)| {
        abilities.iter().find_map(|&(card_id, ability_index)| {
            let action = GameAction::ActivateAbility {
                card_id,
                ability_index,
                target: Some(Target::Permanent(atk)),
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            (ward_gate_ok(state, seat, &action) && state.would_accept(action.clone())).then_some(action)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    use crate::mana::Color;

    /// Soul Snare exiles a big creature attacking its controller in a pod,
    /// and never in a game without commanders.
    #[test]
    fn soul_snare_exiles_a_worthwhile_attacker() {
        let mut g = crate::game::multi_player_game(3);
        let snare = g.add_card_to_battlefield(0, crate::catalog::soul_snare());
        let wurm = g.add_card_to_battlefield(1, crate::catalog::craw_wurm());
        g.clear_sickness(wurm);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.declare_attackers(vec![Attack { attacker: wurm, target: AttackTarget::Player(0) }]).expect("attack");
        g.players[0].mana_pool.add(Color::White, 1);
        g.priority.player_with_priority = 0;
        let w = EvalWeights::default();
        assert_eq!(pick_defensive_ability(&g, 0, &w), None, "no commander: not a pod");
        let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.players[0].commanders.push(cmd);
        assert!(matches!(
            pick_defensive_ability(&g, 0, &w),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == snare && t == wurm
        ));
    }
}
