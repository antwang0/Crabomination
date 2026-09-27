//! "{1}: Attach this to target creature an opponent controls" on an
//! Equipment whose host is goaded (Bloodthirsty Blade). No generator built
//! the move and the material eval prices the +2/+0 it hands over but not the
//! goad, so the generic sink declined it: a 183-deck `--card-census --a dflt`
//! never activated it in six decks. A pod bot now parks it on the strongest
//! opposing creature once it isn't already on one. Commander games only.

use crate::effect::{Effect, StaticEffect};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// The attach onto the most powerful creature an opponent controls, for the
/// first goading Equipment `seat` controls that isn't already on one.
pub(super) fn pick_goad_attach(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    let goads = |c: &crate::card::CardInstance| {
        c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::AttachedIsGoaded))
    };
    let on_opponent = |c: &crate::card::CardInstance| {
        c.attached_to.and_then(|h| state.battlefield_find(h)).is_some_and(|h| h.controller != seat)
    };
    let mut hosts: Vec<(i32, crate::card::CardId)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller != seat && c.definition.is_creature() && !state.same_team(seat, c.controller))
        .map(|c| (state.computed_permanent(c.id).map_or(0, |cp| cp.power), c.id))
        .collect();
    hosts.sort_by_key(|&(p, id)| (std::cmp::Reverse(p), id));
    state.battlefield.iter().filter(|c| c.controller == seat && goads(c) && !on_opponent(c)).find_map(|eq| {
        let idx = eq
            .definition
            .activated_abilities
            .iter()
            .position(|ab| matches!(ab.effect, Effect::AttachSourceTo { .. }))?;
        hosts.iter().find_map(|&(_, host)| {
            let action = GameAction::ActivateAbility {
                card_id: eq.id,
                ability_index: idx,
                target: Some(Target::Permanent(host)),
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;

    /// Bloodthirsty Blade goes onto the biggest opposing creature, once.
    #[test]
    fn bloodthirsty_blade_goads_the_biggest_opposing_creature() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let blade = g.add_card_to_battlefield(0, crate::catalog::bloodthirsty_blade());
        g.add_card_to_battlefield(1, crate::catalog::grizzly_bears());
        let wurm = g.add_card_to_battlefield(2, crate::catalog::craw_wurm());
        g.players[0].mana_pool.add_colorless(2);
        assert_eq!(pick_goad_attach(&g, 0), None, "no commander: not a pod");
        let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.players[0].commanders.push(cmd);
        assert!(matches!(
            pick_goad_attach(&g, 0),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == blade && t == wurm
        ));
        g.battlefield_find_mut(blade).unwrap().attached_to = Some(wurm);
        assert_eq!(pick_goad_attach(&g, 0), None, "already goading");
    }
}
