//! A repeatable "exile target card from a graveyard" (Ghost Vacuum's {T},
//! Scavenging Ooze's {G}) was never activated in a 6-seat census (seed
//! 2750001, three decks):
//! nothing asks for graveyard hate outside a removal read. At an opponent's
//! end step the tap and mana are spare, so a pod bot exiles an opponent's
//! best graveyard card: a creature card first (the reanimation target), then
//! the greatest mana value. A sorcery-speed one (Dino DNA's imprint) goes in
//! its controller's main phase with the mana otherwise idle. Commander games
//! only, so two-player play is unchanged.

use crate::effect::{Effect, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// Exiles one targeted card from ANY graveyard (not "your graveyard" — that
/// is fuel, not hate).
fn exiles_a_graveyard_card(e: &Effect) -> bool {
    match e {
        Effect::Move { what: Selector::TargetFiltered { slot: 0, filter }, to: ZoneDest::Exile | ZoneDest::ExileWithSourceStamp }
        | Effect::ExileLinked { what: Selector::TargetFiltered { slot: 0, filter } } => {
            let f = format!("{filter:?}");
            f.contains("InGraveyard") && !f.contains("InYourGraveyard")
        }
        // Scavenging Ooze's exile leads its counter / life rider.
        Effect::Seq(v) => v.first().is_some_and(exiles_a_graveyard_card),
        _ => false,
    }
}

/// The first accepted graveyard exile aimed at an opponent's card, at an
/// opponent's end step.
pub(super) fn pick_graveyard_hate(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.active_player_idx == seat {
        return None;
    }
    pick_hate(state, seat, false)
}

/// [`pick_graveyard_hate`] for a sorcery-speed ability, in `seat`'s own main
/// phase.
pub(super) fn pick_sorcery_graveyard_hate(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.active_player_idx != seat {
        return None;
    }
    pick_hate(state, seat, true)
}

fn pick_hate(state: &GameState, seat: usize, sorcery: bool) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    let abilities: Vec<(crate::card::CardId, usize)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition.activated_abilities.iter().enumerate().filter_map(move |(i, ab)| {
                (ab.sorcery_speed == sorcery
                    && super::end_step_ping::costs_only_mana(ab)
                    && exiles_a_graveyard_card(&ab.effect))
                .then_some((c.id, i))
            })
        })
        .collect();
    if abilities.is_empty() {
        return None;
    }
    let mut cards: Vec<(bool, u32, crate::card::CardId)> = (0..state.players.len())
        .filter(|&q| !state.same_team(q, seat))
        .flat_map(|q| state.players[q].graveyard.iter())
        .map(|c| (c.definition.is_creature(), c.definition.cost.cmc(), c.id))
        .collect();
    cards.sort_by(|a, b| b.cmp(a));
    for (card_id, i) in abilities {
        for &(_, _, target) in cards.iter().take(4) {
            let action = GameAction::ActivateAbility {
                card_id,
                ability_index: i,
                target: Some(Target::Permanent(target)),
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;

    /// Ghost Vacuum exiles the opponent's creature card at their end step,
    /// not on its controller's turn.
    #[test]
    fn ghost_vacuum_eats_the_reanimation_target() {
        let mut g = crate::game::multi_player_game(4);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let vac = g.add_card_to_battlefield(0, crate::catalog::ghost_vacuum());
        g.add_card_to_graveyard(2, crate::catalog::lightning_bolt());
        let fatty = g.add_card_to_graveyard(2, crate::catalog::serra_angel());
        g.step = TurnStep::End;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(pick_graveyard_hate(&g, 0).is_none(), "own turn");
        g.active_player_idx = 1;
        let a = pick_graveyard_hate(&g, 0).expect("hate");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
            if card_id == vac && t == fatty));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == fatty));
    }

    /// Dino DNA's sorcery-speed imprint exiles an opponent's creature card in
    /// its controller's main phase, linked to the artifact — never on an
    /// opponent's turn.
    #[test]
    fn dino_dna_imprints_in_its_controllers_main_phase() {
        let mut g = crate::game::multi_player_game(4);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let dna = g.add_card_to_battlefield(0, crate::catalog::dino_dna());
        g.players[0].mana_pool.add_colorless(1);
        let fatty = g.add_card_to_graveyard(2, crate::catalog::serra_angel());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 0;
        assert!(pick_sorcery_graveyard_hate(&g, 0).is_none(), "not on an opponent's turn");
        assert!(pick_graveyard_hate(&g, 0).is_none(), "the end-step sink skips sorcery speed");
        g.active_player_idx = 0;
        let a = pick_sorcery_graveyard_hate(&g, 0).expect("imprint");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
            if card_id == dna && t == fatty));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == fatty && c.exiled_with == Some(dna)));
    }
}
