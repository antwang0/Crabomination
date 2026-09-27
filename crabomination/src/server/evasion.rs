//! A pre-combat "target creature gains [keyword] until end of turn" (Rogue's
//! Passage, Sunhome, Witch's Clinic, Whirler Rogue) was activated by no bot
//! path: its main-phase score reads a temporary grant as idle, so a 400-game
//! census never activated one. The bot now spends idle first-main mana on one,
//! aimed at its biggest ready attacker that lacks the keyword, or on a
//! board-wide one ("all Zombies gain menace" — Lord of the Accursed) when a
//! ready attacker of ours gains; a grant that taps its source doesn't count
//! the source as an attacker. Commander games only, so two-player play is
//! unchanged.
//!
//! Haste grants had the same gap (a 6-seat `--a dflt` census, seed 1360001:
//! Flamekin Village, Crashing Drawbridge, Otepec Huntmaster, Skyship Stalker
//! never activated): [`pick_haste_grant`] gives haste to the biggest creature
//! that entered this turn (CR 302.6, 702.10b), so it can join the attack.

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
    // (source, ability, keyword, board-wide filter — `None` for a target
    // slot, whether it taps its source)
    type Grant<'a> = (crate::card::CardId, usize, &'a Keyword, Option<&'a crate::card::SelectionRequirement>, bool);
    let grants: Vec<Grant> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition.activated_abilities.iter().enumerate().filter_map(move |(i, ab)| match &ab.effect {
                Effect::GrantKeyword { what: Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. }, keyword, duration: Duration::EndOfTurn }
                    if attack_keyword(keyword) && !ab.sac_cost =>
                {
                    Some((c.id, i, keyword, None, ab.tap_cost))
                }
                // "All Zombies gain menace until end of turn" (Lord of the
                // Accursed): bought when one of our ready attackers gains.
                Effect::GrantKeyword { what: Selector::EachPermanent(filter), keyword, duration: Duration::EndOfTurn }
                    if attack_keyword(keyword) && !ab.sac_cost =>
                {
                    Some((c.id, i, keyword, Some(filter), ab.tap_cost))
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
    for (source, index, keyword, board, taps_source) in grants {
        for &(_, creature) in &ready {
            // A source the grant taps won't be attacking.
            if (taps_source && creature == source)
                || state.computed_permanent(creature).is_some_and(|cp| cp.keywords().has_kw(keyword))
            {
                continue;
            }
            if board.is_some_and(|f| {
                !state.evaluate_requirement_static(f, &Target::Permanent(creature), seat, Some(source))
            }) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: source,
                ability_index: index,
                target: board.is_none().then_some(Target::Permanent(creature)),
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

/// Whom a haste grant in `e` reaches, if `e` is one.
fn haste_reach(e: &Effect) -> Option<&Selector> {
    match e {
        Effect::GrantKeyword { what, keyword: Keyword::Haste, duration: Duration::EndOfTurn } => Some(what),
        _ => None,
    }
}

/// The first accepted haste grant that lets `seat`'s biggest summoning-sick
/// creature attack this turn, in `seat`'s first main phase of a Commander
/// game.
pub(super) fn pick_haste_grant(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty()
        || state.step != TurnStep::PreCombatMain
        || state.active_player_idx != seat
        || !state.stack.is_empty()
    {
        return None;
    }
    let mut sick: Vec<(i32, crate::card::CardId)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.summoning_sick && c.definition.is_creature())
        .filter_map(|c| {
            let cp = state.computed_permanent(c.id)?;
            let kws = cp.keywords();
            (!kws.has_kw(&Keyword::Haste) && !kws.has_kw(&Keyword::Defender) && cp.power > 0)
                .then_some((cp.power, c.id))
        })
        .collect();
    if sick.is_empty() {
        return None;
    }
    sick.sort_by_key(|(power, id)| (std::cmp::Reverse(*power), *id));
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (i, ab) in c.definition.activated_abilities.iter().enumerate() {
            let Some(what) = haste_reach(&ab.effect) else { continue };
            if ab.sac_cost || ab.sac_other_filter.is_some() || ab.discard_cost.is_some() || ab.life_cost > 0 {
                continue;
            }
            let aims: Vec<Option<Target>> = match what {
                Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. } => {
                    sick.iter().map(|&(_, id)| Some(Target::Permanent(id))).collect()
                }
                // "This creature gains haste", on a sick source; a board-wide
                // grant, when a sick creature is ours.
                Selector::This if sick.iter().any(|&(_, id)| id == c.id) => vec![None],
                Selector::EachPermanent(_) => vec![None],
                _ => continue,
            };
            for target in aims {
                let action = GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target,
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
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
        g.add_card_to_battlefield(0, crate::catalog::craw_wurm()); // summoning sick
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

    /// CR 702.10b — Flamekin Village gives the Hill Giant cast this turn
    /// haste in the first main phase; with nothing sick it stays a land.
    #[test]
    fn flamekin_village_hastes_a_fresh_creature() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let village = g.add_card_to_battlefield(0, crate::catalog::flamekin_village());
        g.clear_sickness(village);
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.players[0].mana_pool.add(Color::Red, 1);
        assert!(pick_haste_grant(&g, 0).is_none(), "nothing sick");
        let giant = g.add_card_to_battlefield(0, crate::catalog::hill_giant());
        let a = pick_haste_grant(&g, 0).expect("haste");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
            if card_id == village && t == giant));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.computed_permanent(giant).unwrap().keywords().has_kw(&Keyword::Haste));
    }

    /// Lord of the Accursed's "all Zombies gain menace" is bought for a ready
    /// Zombie attacker — not for the Lord alone, which the cost taps.
    #[test]
    fn lord_of_the_accursed_menaces_ready_zombies() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let lord = g.add_card_to_battlefield(0, crate::catalog::lord_of_the_accursed());
        g.clear_sickness(lord);
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.players[0].mana_pool.add(Color::Black, 2);
        assert!(pick_evasion_grant(&g, 0).is_none(), "the Lord taps; the Bear isn't a Zombie");
        let zombie = g.add_card_to_battlefield(0, crate::catalog::walking_corpse());
        g.clear_sickness(zombie);
        let a = pick_evasion_grant(&g, 0).expect("menace for the Corpse");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: None, .. } if card_id == lord));
    }
}
