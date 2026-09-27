//! CR 701.15 — a creature's own regeneration ability (Thrun, Mortivore,
//! Troll Ascetic, Accursed Duneyard) was activated by no bot path: a 6-seat
//! `--card-census` (seed 206001) never used one. The bot now answers a
//! regenerable destroy on top of the stack — aimed at its creature, or a
//! "destroy all" that covers it — with a shield on each creature it would
//! lose. Commander games only, so two-player play is unchanged.

use crate::card::CardId;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, StackItem, Target};

/// What a regenerable destroy in `e` reaches: its target slot, or a sweep.
fn destroys(e: &Effect) -> Option<bool> {
    match e {
        Effect::Destroy { what } => match what {
            Selector::Target(_) | Selector::TargetFiltered { .. } => Some(false),
            Selector::EachPermanent(_) => Some(true),
            _ => None,
        },
        Effect::Seq(v) => v.iter().find_map(destroys),
        Effect::ApplyToTargets { effect, .. } => destroys(effect).map(|_| false),
        _ => None,
    }
}

/// A regeneration activation for one of `seat`'s creatures the top of the
/// stack would destroy, skipping a creature that already holds a shield.
pub(super) fn pick_regen_response(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    let (effect, target) = match state.stack.last()? {
        StackItem::Spell { card, target, .. } => (&card.definition.effect, target.as_ref()),
        StackItem::Trigger { effect, target, .. } => (&**effect, target.as_ref()),
    };
    let sweep = destroys(effect)?;
    let threatened = |id: CardId| sweep || target == Some(&Target::Permanent(id));
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature() && c.regeneration_shields == 0)
        .filter(|c| threatened(c.id))
        .flat_map(|c| {
            c.definition.activated_abilities.iter().enumerate().filter_map(move |(i, ab)| {
                matches!(ab.effect, Effect::Regenerate { what: Selector::This }).then_some(GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target: None,
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                })
            })
        })
        .find(|a| state.would_accept(a.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Drudge Skeletons regenerate in response to a Murder aimed at it; a
    /// shield already up is not stacked again.
    #[test]
    fn a_regenerator_answers_murder() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        let skeletons = g.add_card_to_battlefield(0, crate::catalog::drudge_skeletons());
        let murder = g.add_card_to_hand(1, crate::catalog::murder());
        g.players[1].mana_pool.add(Color::Black, 3);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: murder, target: Some(Target::Permanent(skeletons)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("Murder");
        g.priority.player_with_priority = 0;
        assert!(pick_regen_response(&g, 0).is_none(), "no mana");
        g.players[0].mana_pool.add(Color::Black, 1);
        assert!(matches!(pick_regen_response(&g, 0), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == skeletons));
        g.battlefield_find_mut(skeletons).unwrap().regeneration_shields = 1;
        assert!(pick_regen_response(&g, 0).is_none(), "already shielded");
    }
}
