//! A defender's removal *ability* aimed at an attacker (Soul Snare's "{W},
//! sacrifice: exile target creature that's attacking you") — the defensive
//! removal picker reads instants in hand only, so a 183-deck `--card-census
//! --a dflt` never activated Soul Snare in any of its seven decks. Stalking
//! Leonin's once-only exile hides behind a resolution-time `If` (the attacker's
//! controller is the chosen player) — the `If` is read first, so the
//! activation isn't spent on a fizzle (census seed 3100001, four decks).
//! Commander games only, so two-player play is unchanged.

use crate::effect::{Effect, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::types::{GameAction, Target};

use super::bot::{EvalWeights, permanent_value, ward_gate_ok};

/// A targeted destroy, exile or bounce as the ability's first step, an `If`
/// read for `ctx` on the way down.
fn removes_target(state: &GameState, e: &Effect, ctx: &EffectContext) -> bool {
    let targeted = |w: &Selector| matches!(w, Selector::Target(_) | Selector::TargetFiltered { .. });
    match e {
        Effect::Destroy { what } | Effect::DestroyNoRegen { what } | Effect::Exile { what } => targeted(what),
        Effect::Move { what, to: ZoneDest::Exile | ZoneDest::Hand(_) } => targeted(what),
        Effect::If { cond, then, else_ } => {
            removes_target(state, if state.evaluate_predicate(cond, ctx) { then } else { else_ }, ctx)
        }
        Effect::Seq(v) => v.first().is_some_and(|e| removes_target(state, e, ctx)),
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
            let ctx = EffectContext::for_ability(c.id, seat, None);
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .filter(move |(_, ab)| removes_target(state, &ab.effect, &ctx) && !ab.from_hand && !ab.from_graveyard)
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

    /// Stalking Leonin exiles the chosen opponent's attacker, and holds its
    /// once-only activation against anyone else's.
    #[test]
    fn stalking_leonin_answers_only_the_chosen_players_attack() {
        for (chosen, fires) in [(1usize, true), (2, false)] {
            let mut g = crate::game::multi_player_game(3);
            let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
            g.players[0].commanders.push(cmd);
            let leonin = g.add_card_to_battlefield(0, crate::catalog::stalking_leonin());
            g.battlefield_find_mut(leonin).unwrap().chosen_player = Some(chosen);
            let wurm = g.add_card_to_battlefield(1, crate::catalog::craw_wurm());
            g.clear_sickness(wurm);
            g.step = TurnStep::DeclareAttackers;
            g.active_player_idx = 1;
            g.priority.player_with_priority = 1;
            g.declare_attackers(vec![Attack { attacker: wurm, target: AttackTarget::Player(0) }]).expect("attack");
            g.priority.player_with_priority = 0;
            let picked = pick_defensive_ability(&g, 0, &EvalWeights::default());
            assert_eq!(
                matches!(picked, Some(GameAction::ActivateAbility { card_id, .. }) if card_id == leonin),
                fires,
                "chosen seat {chosen}"
            );
        }
    }
}
