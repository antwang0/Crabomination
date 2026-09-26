//! CR 615.1 — a fog (Darkness, Dawn Charm, Inkshield, Prismatic Strands) was
//! cast from the main-phase sweep like any spell, so a pod bot spent it on its
//! own turn where it prevents nothing. In a Commander game the bot now holds a
//! fog for the one window it answers: blocks are in, it is being attacked,
//! and the unblocked damage coming at it is dangerous. Prismatic Strands'
//! tap-flashback (CR 702.34) is taken from the graveyard in the same window.
//! Commander games only, so two-player play is unchanged.

use crate::card::{CardDefinition, CardId, Keyword};
use crate::effect::{Effect, PlayerRef};
use crate::game::GameState;
use crate::game::types::GameAction;
use crate::mana::Color;

/// Does this effect leaf stop combat damage to its caster?
fn is_fog_leaf(e: &Effect) -> bool {
    matches!(
        e,
        Effect::PreventAllCombatDamageThisTurn
            | Effect::PreventAllDamageFromChosenColorGlobally
            | Effect::PreventAllCombatDamageToPlayerThisTurn { who: PlayerRef::You }
            | Effect::PreventAllCombatDamageToPlayerAndWalkersThisTurn { who: PlayerRef::You }
    )
}

/// `Some(mode)` when `def` is an untargeted fog instant: `Some(None)` for the
/// whole spell, `Some(Some(i))` for mode `i` of a modal one (Dawn Charm).
/// Combat-only spells (Take the Bait) keep their own window.
pub(super) fn fog_mode(def: &CardDefinition) -> Option<Option<usize>> {
    if !def.is_instant_speed() || def.cast_only_during_combat {
        return None;
    }
    match &def.effect {
        Effect::ChooseMode(modes) => modes.iter().position(is_fog_leaf).map(Some),
        e if e.requires_target() => None,
        Effect::Seq(v) if v.iter().any(is_fog_leaf) => Some(None),
        e if is_fog_leaf(e) => Some(None),
        _ => None,
    }
}

/// The main-phase sweep skips a fog in a Commander game: [`pick_fog`] is its
/// route onto the stack.
pub(super) fn held_fog(state: &GameState, seat: usize, def: &CardDefinition) -> bool {
    !state.players[seat].commanders.is_empty() && fog_mode(def).is_some()
}

/// Unblocked combat damage aimed at `seat`, and whether it is dangerous:
/// lethal, lethal commander damage (CR 903.10a), or a third of the life total
/// and at least 6.
fn danger(state: &GameState, seat: usize) -> Option<Vec<CardId>> {
    let life = state.players[seat].life;
    let mut total = 0i32;
    let mut cmd_lethal = false;
    let mut incoming = Vec::new();
    for a in state.attacking() {
        if state.defender_for(a.target) != Some(seat) || !state.blockers_of(a.attacker).is_empty() {
            continue;
        }
        let Some(c) = state.battlefield_find(a.attacker) else { continue };
        let power = state.computed_permanent(c.id).map_or(c.power(), |cp| cp.power).max(0);
        total += power;
        if let Some(cmd) = state.commander_card_of(c.id) {
            let dealt = state.commander_damage.get(&(seat, cmd)).copied().unwrap_or(0);
            cmd_lethal |= dealt + power as u32 >= 21;
        }
        incoming.push(c.id);
    }
    (total >= life || cmd_lethal || (total >= 6 && total * 3 >= life)).then_some(incoming)
}

/// Cast a held fog, or flash Prismatic Strands back by tapping a white
/// creature, when [`danger`] says the unblocked damage warrants it.
pub(super) fn pick_fog(state: &GameState, seat: usize) -> Option<GameAction> {
    let p = &state.players[seat];
    if p.commanders.is_empty() || state.active_player_idx == seat {
        return None;
    }
    let has_fog = p.hand.iter().any(|c| fog_mode(&c.definition).is_some())
        || p.graveyard.iter().any(|c| flashback_tap_fog(&c.definition));
    if !has_fog {
        return None;
    }
    danger(state, seat)?;
    let cast = p.hand.iter().filter_map(|c| {
        fog_mode(&c.definition).map(|mode| GameAction::CastSpell {
            card_id: c.id,
            target: None,
            additional_targets: vec![],
            mode,
            x_value: None,
        })
    });
    let flashback = p.graveyard.iter().filter(|c| flashback_tap_fog(&c.definition)).filter_map(|c| {
        let tapper = state.battlefield.iter().find(|b| {
            b.controller == seat
                && !b.tapped
                && b.definition.is_creature()
                && state.computed_permanent(b.id).is_some_and(|cp| cp.colors.contains(&Color::White))
        })?;
        Some(GameAction::CastFlashbackTap {
            card_id: c.id,
            tap_creatures: vec![tapper.id],
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    });
    cast.chain(flashback).find(|a| state.would_accept(a.clone()))
}

/// A graveyard fog cast by tapping creatures (Prismatic Strands).
fn flashback_tap_fog(def: &CardDefinition) -> bool {
    def.keywords.iter().any(|k| matches!(k, Keyword::FlashbackTap { .. })) && fog_mode(def).is_some()
}

/// Prismatic Strands' color: the one with the most unblocked power aimed at
/// `seat`, `None` when nothing is attacking it (or outside Commander).
pub(super) fn fog_color(state: &GameState, seat: usize) -> Option<Color> {
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    let incoming = danger_power_by_color(state, seat);
    incoming.iter().copied().enumerate().filter(|(_, n)| *n > 0).max_by_key(|(i, n)| (*n, std::cmp::Reverse(*i))).map(
        |(i, _)| [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green][i],
    )
}

fn danger_power_by_color(state: &GameState, seat: usize) -> [i32; 5] {
    let mut by = [0i32; 5];
    for a in state.attacking() {
        if state.defender_for(a.target) != Some(seat) || !state.blockers_of(a.attacker).is_empty() {
            continue;
        }
        let Some(cp) = state.computed_permanent(a.attacker) else { continue };
        for (i, col) in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green].iter().enumerate() {
            if cp.colors.contains(col) {
                by[i] += cp.power.max(0);
            }
        }
    }
    by
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::{Attack, AttackTarget, TurnStep};

    fn attacked_pod() -> (GameState, CardId) {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(1, vec![crate::catalog::grizzly_bears()]);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        g.players[1].life = 10;
        let giant = g.add_card_to_battlefield(0, crate::catalog::craw_wurm());
        g.set_attacking(vec![Attack { attacker: giant, target: AttackTarget::Player(1) }]);
        g.set_blockers_declared(true);
        (g, giant)
    }

    /// CR 615.1 — the defender fogs a dangerous unblocked attack, and the
    /// main-phase sweep holds the fog in a Commander game.
    #[test]
    fn a_fog_is_held_then_cast_against_a_dangerous_attack() {
        let (mut g, _) = attacked_pod();
        let fog = g.add_card_to_hand(1, crate::catalog::darkness());
        assert!(held_fog(&g, 1, &crate::catalog::darkness()));
        assert!(pick_fog(&g, 1).is_none(), "no mana");
        g.players[1].mana_pool.add(Color::Black, 1);
        assert!(matches!(pick_fog(&g, 1), Some(GameAction::CastSpell { card_id, .. }) if card_id == fog));
        g.players[1].life = 40;
        assert!(pick_fog(&g, 1).is_none(), "6 of 40 life is not worth a fog");
    }

    /// Dawn Charm fogs through its first mode.
    #[test]
    fn a_modal_fog_casts_its_fog_mode() {
        let (mut g, _) = attacked_pod();
        let charm = g.add_card_to_hand(1, crate::catalog::dawn_charm());
        g.players[1].mana_pool.add(Color::White, 2);
        assert!(matches!(
            pick_fog(&g, 1),
            Some(GameAction::CastSpell { card_id, mode: Some(0), .. }) if card_id == charm
        ));
    }

    /// CR 702.34 — Prismatic Strands flashes back by tapping a white
    /// creature, and names the attacker's color.
    #[test]
    fn prismatic_strands_flashes_back_and_names_the_attackers_color() {
        let (mut g, _) = attacked_pod();
        let strands = g.add_card_to_graveyard(1, crate::catalog::prismatic_strands());
        let knight = g.add_card_to_battlefield(1, crate::catalog::savannah_lions());
        assert!(matches!(
            pick_fog(&g, 1),
            Some(GameAction::CastFlashbackTap { card_id, ref tap_creatures, .. })
                if card_id == strands && tap_creatures == &vec![knight]
        ));
        assert_eq!(fog_color(&g, 1), Some(Color::Green), "Craw Wurm is green");
    }
}
