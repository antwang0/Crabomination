//! Grant-then-cast abilities — "{T}, pay 2 life: until end of turn, you may
//! cast a creature spell from among cards exiled with this" (Serpent's
//! Soul-Jar), "{T}: choose target artifact card in your graveyard; you may cast
//! it this turn" (Emry). The grant alone changes no board, so the one-ply
//! scorer never activated one (a 183-deck `--card-census`). A pod bot now
//! activates it when a granted card is castable afterwards and that cast beats
//! passing. Commander games only, so two-player play is unchanged.

use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

use super::bot::{EvalWeights, eval_material, evaluate_action_outcome, settle_to_quiescence};

/// The best grant activation of `seat`'s idle main phase, if the cast it opens
/// beats doing nothing.
pub(super) fn pick_grant_cast(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty()
        || !state.stack.is_empty()
        || state.active_player_idx != seat
        || !matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
    {
        return None;
    }
    let grants = |e: &Effect| matches!(e, Effect::GrantMayPlay { .. });
    let sources: Vec<(crate::card::CardId, usize)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .filter(move |(_, ab)| !crate::game::actions::is_mana_ability(&ab.effect) && ab.effect.any_nested(&grants))
                .map(move |(i, _)| (c.id, i))
        })
        .collect();
    if sources.is_empty() {
        return None;
    }
    let granted_to_me = |g: &GameState| -> Vec<crate::card::CardId> {
        g.exile
            .iter()
            .chain(g.players.iter().flat_map(|p| p.graveyard.iter()))
            .filter(|c| !c.definition.is_land() && c.may_play_until.is_some_and(|m| m.player == seat))
            .map(|c| c.id)
            .collect()
    };
    let before = granted_to_me(state);
    let base = eval_material(state, seat, w);
    let mut best: Option<(i32, GameAction)> = None;
    for (card_id, idx) in sources {
        let ab = &state.battlefield_find(card_id)?.definition.activated_abilities[idx];
        let (target, additional_targets) = if ab.effect.requires_target() {
            match state.auto_targets_for_effect_all_slots_x(&ab.effect, seat, None, false, Some(card_id), None) {
                (Some(t), extra) => (Some(t), extra),
                (None, _) => continue,
            }
        } else {
            (None, Vec::new())
        };
        let action =
            GameAction::ActivateAbility { card_id, ability_index: idx, target, additional_targets, x_value: None, mode: None };
        let Some(g) = settle_to_quiescence(state, &action, None, w) else { continue };
        for id in granted_to_me(&g).into_iter().filter(|id| !before.contains(id)) {
            let Some(card) = g.find_card_anywhere(id) else { continue };
            let (target, additional_targets) = if card.definition.effect.requires_target() {
                g.auto_targets_for_effect_all_slots(&card.definition.effect, seat, None)
            } else {
                (None, Vec::new())
            };
            let cast = GameAction::CastFromZoneWithoutPaying { card_id: id, target, additional_targets, mode: None, x_value: None };
            if let Some(ev) = evaluate_action_outcome(&g, seat, &cast, None, w)
                && ev > base
                && best.as_ref().is_none_or(|(b, _)| ev > *b)
            {
                best = Some((ev, action.clone()));
            }
        }
    }
    best.map(|(_, a)| a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::Target;

    /// An idle pod bot taps Serpent's Soul-Jar when an Elf is exiled with it
    /// and it can pay for the recast; with nothing exiled it keeps the Jar.
    #[test]
    fn a_pod_bot_opens_the_soul_jar_for_an_exiled_elf() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.players[0].commanders.push(cmd);
        let jar = g.add_card_to_battlefield(0, crate::catalog::serpents_soul_jar());
        for _ in 0..3 {
            g.add_card_to_battlefield(0, crate::catalog::forest());
        }
        let w = EvalWeights::default();
        assert_eq!(pick_grant_cast(&g, 0, &w), None, "nothing exiled with the Jar");
        let elf = g.add_card_to_battlefield(0, crate::catalog::llanowar_elves());
        let bolt = g.add_card_to_hand(0, crate::catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(elf)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Bolt");
        crate::game::drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == elf), "the Elf was exiled with the Jar");
        assert!(matches!(
            pick_grant_cast(&g, 0, &w),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == jar
        ));
    }
}
