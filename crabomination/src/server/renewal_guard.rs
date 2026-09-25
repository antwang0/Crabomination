//! A sacrifice that only renews its own fodder is not a value play.

use crate::card::CardId;
use crate::effect::{ActivatedAbility, Effect};
use crate::game::GameState;

/// The names of the tokens `e` creates, one level through `Seq`.
fn created_token_names(e: &Effect, out: &mut Vec<String>) {
    match e {
        Effect::CreateToken { definition, .. } | Effect::CreateTokenAttacking { definition, .. } => {
            out.push(definition.name.clone())
        }
        Effect::Seq(steps) => steps.iter().for_each(|s| created_token_names(s, out)),
        _ => {}
    }
}

/// True when activating `ab` on `source` would sacrifice nothing but tokens of
/// a kind the ability itself makes — Tooth and Claw feeding on its own
/// Carnivores, which under a token doubler is a net-zero exchange the
/// evaluator can score above passing forever (3,936 activations in one turn,
/// pod seed 10530 game 241). The fodder is the engine's own auto-pick.
pub(super) fn renews_its_own_fodder(state: &GameState, seat: usize, source: CardId, ab: &ActivatedAbility) -> bool {
    let Some((filter, count)) = ab.sac_other_filter.as_ref() else { return false };
    let mut names = Vec::new();
    created_token_names(&ab.effect, &mut names);
    if names.is_empty() {
        return false;
    }
    let candidates: Vec<CardId> = state
        .battlefield
        .iter()
        .filter(|c| c.id != source && c.controller == seat && state.evaluate_requirement_on_card(filter, c, seat))
        .map(|c| c.id)
        .collect();
    let picks = state.auto_pick_lowest_power(&candidates, *count as usize);
    !picks.is_empty()
        && picks.iter().all(|id| {
            state.battlefield_find(*id).is_some_and(|c| c.is_token && names.iter().any(|n| n == c.definition.name))
        })
}

#[cfg(test)]
mod tests {
    use crate::catalog;
    use crate::game::types::{GameAction, TurnStep};
    use crate::game::*;
    use crate::server::bot::{EvalWeights, pick_sacrifice_value, pick_token_maker};

    /// Tooth and Claw under Primal Vigor: two Carnivores in, two out — the
    /// pod ran it 3,936 times in one turn (seed 10530, game 241). Bears are
    /// fodder worth trading; the Carnivores it made are not.
    #[test]
    fn a_sacrifice_that_only_renews_its_fodder_is_not_taken() {
        let mut g = multi_player_game(4);
        let me = 1;
        g.add_card_to_battlefield(me, catalog::tooth_and_claw());
        g.add_card_to_battlefield(me, catalog::primal_vigor());
        for _ in 0..2 {
            let id = g.add_card_to_battlefield(me, catalog::grizzly_bears());
            g.clear_sickness(id);
        }
        let tac = g.battlefield.iter().find(|c| c.definition.name == "Tooth and Claw").unwrap().id;
        let ab = g.battlefield_find(tac).unwrap().definition.activated_abilities[0].clone();
        assert!(!super::renews_its_own_fodder(&g, me, tac, &ab), "Bears are not Carnivores");
        g.active_player_idx = me;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = me;
        g.perform_action(GameAction::ActivateAbility {
            card_id: tac,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert!(super::renews_its_own_fodder(&g, me, tac, &ab), "two Carnivores for two Carnivores");
        g.step = TurnStep::PostCombatMain;
        for w in [EvalWeights::default(), EvalWeights::baseline()] {
            assert!(pick_sacrifice_value(&g, me, &w).is_none());
            assert!(pick_token_maker(&g, me, &w).is_none());
        }
    }
}
