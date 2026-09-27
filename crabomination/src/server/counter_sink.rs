//! "Put a +1/+1 (or loyalty) counter on target [permanent]" as an activated
//! ability (Forge of Heroes, a Commander staple) was activated by no bot path
//! — a 6-seat `--card-census` over every deck (seed 206001) never used Forge
//! of Heroes in four decks. The bot now spends idle main-phase mana on one,
//! aimed at its own biggest creature or planeswalker the filter allows.
//! Level up (CR 702.87 — Kazandu Tuskcaller, Hada Spy Patrol, Coralhelm
//! Commander) had the same gap: a level counter shows on no board until a
//! tier, so idle mana now levels up to the card's top tier.
//! Commander games only, so two-player play is unchanged.

use crate::card::CounterType;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// Does `e` grow its slot-0 target with a +1/+1 or loyalty counter?
fn grows_target(e: &Effect) -> bool {
    match e {
        Effect::AddCounter {
            what: Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. },
            kind: CounterType::PlusOnePlusOne | CounterType::Loyalty,
            ..
        } => true,
        Effect::If { then, else_, .. } => grows_target(then) || grows_target(else_),
        _ => false,
    }
}

/// The first accepted counter activation on one of `seat`'s own creatures or
/// planeswalkers, biggest first.
pub(super) fn pick_counter_sink(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    let abilities: Vec<(crate::card::CardId, usize)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .filter(|(_, ab)| grows_target(&ab.effect) && !ab.sac_cost && ab.sac_other_filter.is_none())
                .map(move |(i, _)| (c.id, i))
        })
        .collect();
    if abilities.is_empty() {
        return pick_level_up(state, seat);
    }
    let mut own: Vec<(i32, crate::card::CardId)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && (c.definition.is_creature() || c.definition.is_planeswalker()))
        .map(|c| (state.computed_permanent(c.id).map_or(0, |cp| cp.power), c.id))
        .collect();
    own.sort_by_key(|(power, id)| (std::cmp::Reverse(*power), *id));
    abilities.into_iter().find_map(|(source, index)| {
        own.iter()
            .map(|&(_, id)| GameAction::ActivateAbility {
                card_id: source,
                ability_index: index,
                target: Some(Target::Permanent(id)),
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            })
            .find(|a| state.would_accept(a.clone()))
    })
    .or_else(|| pick_level_up(state, seat))
}

/// CR 702.87a — "Level up [cost]": `AddCounter { This, Level }`.
fn levels_self(e: &Effect) -> bool {
    matches!(e, Effect::AddCounter { what: Selector::This, kind: CounterType::Level, .. })
}

/// The highest level a tier of `def` asks for — its `LevelBand` minimums and
/// its `SourceHasCountersAtLeast` thresholds on level counters — past which a
/// level counter buys nothing.
fn top_tier(def: &crate::card::CardDefinition) -> u32 {
    let text = format!("{def:?}");
    ["counter: Level, n: ", "LevelBand { min: "]
        .iter()
        .flat_map(|key| text.split(key).skip(1))
        .filter_map(|rest| rest.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok())
        .max()
        .unwrap_or(0)
}

/// A level-up activation on one of `seat`'s permanents still below its top
/// tier, if one is accepted.
fn pick_level_up(state: &GameState, seat: usize) -> Option<GameAction> {
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        let idx = c.definition.activated_abilities.iter().position(|ab| levels_self(&ab.effect))?;
        if c.counter_count(CounterType::Level) >= top_tier(&c.definition) {
            return None;
        }
        let action = GameAction::ActivateAbility {
            card_id: c.id,
            ability_index: idx,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
            mode: None,
        };
        state.would_accept(action.clone()).then_some(action)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;

    /// Forge of Heroes grows a commander that entered this turn, and only one.
    #[test]
    fn forge_of_heroes_grows_a_fresh_commander() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        let forge = g.add_card_to_battlefield(0, crate::catalog::forge_of_heroes());
        g.add_card_to_battlefield(0, crate::catalog::hill_giant());
        let cmd = g.seat_commanders(0, vec![crate::catalog::grizzly_bears()])[0];
        assert!(pick_counter_sink(&g, 0).is_none(), "the commander is not on the battlefield");
        g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
        g.perform_action(GameAction::CastFromCommandZone {
            card_id: cmd, target: None, additional_targets: vec![], mode: None, x_value: None,
            alternative: false, pitch_card: None,
        })
        .expect("cast the commander");
        crate::game::drain_stack(&mut g);
        g.priority.player_with_priority = 0;
        assert!(matches!(
            pick_counter_sink(&g, 0),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == forge && t == cmd
        ), "the Giant is bigger but not a commander that entered this turn");
    }

    /// Kazandu Tuskcaller levels up on idle mana until its top tier (6), then
    /// stops.
    #[test]
    fn idle_mana_levels_up_to_the_top_tier() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let tusk = g.add_card_to_battlefield(0, crate::catalog::kazandu_tuskcaller());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
        assert!(matches!(pick_counter_sink(&g, 0), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == tusk));
        g.battlefield_find_mut(tusk).unwrap().add_counters(CounterType::Level, 6);
        assert!(pick_counter_sink(&g, 0).is_none(), "level 6 is the top tier");
        // A `LevelBand` card reads its bands' minimums (Coralhelm: 4).
        let coral = g.add_card_to_battlefield(0, crate::catalog::coralhelm_commander());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        assert!(matches!(pick_counter_sink(&g, 0), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == coral));
    }
}
