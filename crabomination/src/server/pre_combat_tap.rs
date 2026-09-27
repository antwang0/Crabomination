//! Tapping a would-be attacker before it attacks — Azorius Guildmage's
//! "{2}{W}: tap target creature", Air Servant's flyer tap, Shacklegeist,
//! Dawnglare Invoker's "tap all creatures target player controls". A 6-seat
//! census (seed 1270001) never activated one: the main-phase enumeration
//! scores a tapped creature at nothing, and no path looked at an opponent's
//! beginning of combat (CR 507), the last window before attackers are
//! declared. A pod bot now taps the active player's biggest untapped
//! creature there (or all of them, for a player-targeted tap) when its power
//! is at least `MIN_POWER`. Commander games only, so two-player play is
//! unchanged.

use crate::card::CardId;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, Target, TurnStep};

/// Power below which a tap isn't worth the mana.
const MIN_POWER: i32 = 2;

/// Whom a tap ability in `e` reaches: a creature in its target slot, or every
/// creature of its target player.
enum Reach {
    Creature,
    Player,
}

fn reach(e: &Effect) -> Option<Reach> {
    match e {
        Effect::Tap { what: Selector::Target(_) | Selector::TargetFiltered { .. } } => Some(Reach::Creature),
        Effect::Tap { what: Selector::ControlledBy { who: PlayerRef::Target(0), .. } } => Some(Reach::Player),
        _ => None,
    }
}

/// The first accepted tap on the active player's biggest untapped creature,
/// at their beginning of combat.
pub(super) fn pick_pre_combat_tap(state: &GameState, seat: usize) -> Option<GameAction> {
    let active = state.active_player_idx;
    if state.players[seat].commanders.is_empty()
        || state.step != TurnStep::BeginCombat
        || state.same_team(active, seat)
        || !state.stack.is_empty()
    {
        return None;
    }
    let mut foes: Vec<(i32, CardId)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == active && c.definition.is_creature() && !c.tapped)
        .filter_map(|c| state.computed_permanent(c.id).map(|cp| (cp.power, c.id)))
        .filter(|(p, _)| *p >= MIN_POWER)
        .collect();
    if foes.is_empty() {
        return None;
    }
    foes.sort_by_key(|&(p, id)| (std::cmp::Reverse(p), id));
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (i, ab) in c.definition.activated_abilities.iter().enumerate() {
            let costs_ok = !ab.sac_cost
                && ab.sac_other_filter.is_none()
                && ab.discard_cost.is_none()
                && ab.life_cost == 0
                && ab.energy_cost == 0
                && ab.remove_counter_cost.is_none()
                && ab.remove_all_counters_cost.is_none()
                && ab.exile_other_filter.is_none()
                && !ab.mana_cost.has_x();
            let Some(r) = reach(&ab.effect).filter(|_| costs_ok) else { continue };
            let targets: Vec<Target> = match r {
                Reach::Creature => foes.iter().map(|&(_, id)| Target::Permanent(id)).collect(),
                Reach::Player => vec![Target::Player(active)],
            };
            for t in targets {
                let action = GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target: Some(t),
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

    /// CR 507 — at an opponent's beginning of combat, Azorius Guildmage taps
    /// their Hill Giant; a 1/1 isn't worth it, and on our own turn it doesn't.
    #[test]
    fn azorius_guildmage_taps_the_biggest_would_be_attacker() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        g.add_card_to_battlefield(0, crate::catalog::azorius_guildmage());
        let elf = g.add_card_to_battlefield(1, crate::catalog::llanowar_elves());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::BeginCombat;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 0;
        assert!(pick_pre_combat_tap(&g, 0).is_none(), "a 1/1");
        let giant = g.add_card_to_battlefield(1, crate::catalog::hill_giant());
        let a = pick_pre_combat_tap(&g, 0).expect("tap");
        assert!(matches!(a, GameAction::ActivateAbility { target: Some(Target::Permanent(t)), .. } if t == giant));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.battlefield_find(giant).unwrap().tapped);
        assert!(!g.battlefield_find(elf).unwrap().tapped);
        g.active_player_idx = 0;
        assert!(pick_pre_combat_tap(&g, 0).is_none(), "own turn");
    }
}
