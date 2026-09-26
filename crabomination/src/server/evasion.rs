//! A pre-combat "target creature gains [keyword] until end of turn" (Rogue's
//! Passage, Sunhome, Witch's Clinic, Whirler Rogue) was activated by no bot
//! path: its main-phase score reads a temporary grant as idle, so a 400-game
//! census never activated one. The bot now spends idle first-main mana on one,
//! aimed at its biggest ready attacker that lacks the keyword. Commander games
//! only, so two-player play is unchanged.

use crate::card::{Keyword, KeywordSlice};
use crate::effect::{Duration, Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, Target, TurnStep};

/// Keywords worth buying for an attack.
fn attack_keyword(k: &Keyword) -> bool {
    matches!(
        k,
        Keyword::Unblockable
            | Keyword::Flying
            | Keyword::Menace
            | Keyword::Trample
            | Keyword::DoubleStrike
            | Keyword::Lifelink
    )
}

/// The first accepted grant of an attack keyword to `seat`'s biggest creature
/// able to attack this turn that lacks it. `None` outside `seat`'s first main
/// phase, outside Commander, or without such an ability on the board.
pub(super) fn pick_evasion_grant(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty()
        || state.step != TurnStep::PreCombatMain
        || state.active_player_idx != seat
        || !state.stack.is_empty()
    {
        return None;
    }
    let grants: Vec<(crate::card::CardId, usize, &Keyword)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition.activated_abilities.iter().enumerate().filter_map(move |(i, ab)| match &ab.effect {
                Effect::GrantKeyword { what: Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. }, keyword, duration: Duration::EndOfTurn }
                    if attack_keyword(keyword) && !ab.sac_cost =>
                {
                    Some((c.id, i, keyword))
                }
                _ => None,
            })
        })
        .collect();
    if grants.is_empty() {
        return None;
    }
    let caps = state.attack_power_caps(crate::game::combat::attack_static_scan(state));
    let mut ready: Vec<(i32, crate::card::CardId)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .filter_map(|c| {
            let cp = state.computed_permanent(c.id)?;
            state.attacker_self_block(seat, c, Some(&cp), &caps).is_none().then_some((cp.power, c.id))
        })
        .filter(|(power, _)| *power > 0)
        .collect();
    ready.sort_by_key(|(power, id)| (std::cmp::Reverse(*power), *id));
    for (source, index, keyword) in grants {
        for &(_, creature) in &ready {
            if state.computed_permanent(creature).is_some_and(|cp| cp.keywords().has_kw(keyword)) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: source,
                ability_index: index,
                target: Some(Target::Permanent(creature)),
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
    use crate::mana::Color;

    /// Rogue's Passage makes the biggest ready attacker unblockable in the
    /// first main phase of a Commander game, and not in a duel.
    #[test]
    fn rogues_passage_is_aimed_at_the_biggest_ready_attacker() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let passage = g.add_card_to_battlefield(0, crate::catalog::rogues_passage());
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let giant = g.add_card_to_battlefield(0, crate::catalog::hill_giant());
        let wurm = g.add_card_to_battlefield(0, crate::catalog::craw_wurm());
        for id in [bear, giant] {
            g.clear_sickness(id);
        }
        g.players[0].mana_pool.add(Color::Green, 4);
        assert!(pick_evasion_grant(&g, 0).is_none(), "outside Commander");
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        assert!(matches!(
            pick_evasion_grant(&g, 0),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == passage && t == giant
        ), "the sick Wurm can't attack; the Giant outsizes the Bear");
    }
}
