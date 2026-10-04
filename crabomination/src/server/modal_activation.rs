//! CR 601.2b / 602.2b — a modal activated ability's mode is the activator's,
//! chosen with its targets. A generator that offered the activation with
//! `mode: None` aimed its target at mode 0's slot and left the mode to the
//! resolution decider, so Breya's "3 damage / -4/-4 / gain 5" only ever
//! fired with a player-shaped target. Each mode is its own candidate.

use crate::effect::Effect;

/// The modes a generator offers for an activated ability's `effect`: every
/// mode of a top-level `ChooseMode` (with its own body, for targeting), or
/// the effect itself as one mode-less candidate.
pub(super) fn mode_variants(effect: &Effect) -> Vec<(Option<usize>, &Effect)> {
    match effect {
        Effect::ChooseMode(ms) if ms.len() > 1 => ms.iter().enumerate().map(|(i, m)| (Some(i), m)).collect(),
        other => vec![(None, other)],
    }
}

#[cfg(test)]
mod tests {
    use crate::game::types::{GameAction, Target, TurnStep};

    /// Breya's sacrifice ability is offered mode by mode, each aimed by its
    /// own slot: with an opposing Serra Angel the -4/-4 mode kills it.
    #[test]
    fn a_pod_bot_picks_breyas_mode_with_its_own_target() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, crate::catalog::breya_etherium_shaper());
        for _ in 0..2 {
            g.add_card_to_battlefield(0, crate::catalog::ornithopter());
        }
        let angel = g.add_card_to_battlefield(1, crate::catalog::serra_angel());
        g.players[0].mana_pool.add_colorless(2);
        let pick = super::super::bot::pick_sacrifice_value(&g, 0, &super::super::bot::EvalWeights::default());
        assert!(
            matches!(pick, Some(GameAction::ActivateAbility { mode: Some(1), target: Some(Target::Permanent(t)), .. }) if t == angel),
            "{pick:?}"
        );
    }
}
