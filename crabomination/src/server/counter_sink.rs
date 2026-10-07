//! "Put a +1/+1 (or loyalty) counter on target [permanent]" as an activated
//! ability (Forge of Heroes, a Commander staple) was activated by no bot path
//! — a 6-seat `--card-census` over every deck (seed 206001) never used Forge
//! of Heroes in four decks. The bot now spends idle main-phase mana on one,
//! aimed at its own biggest creature or planeswalker the filter allows.
//! Level up (CR 702.87 — Kazandu Tuskcaller, Hada Spy Patrol, Coralhelm
//! Commander) had the same gap: a level counter shows on no board until a
//! tier, so idle mana now levels up to the card's top tier.
//! A self-growing tap sink — Hangarback Walker's "{1}, {T}: put a +1/+1
//! counter on this", a storage land's "{1}, {T}: put a storage counter on
//! this" (Mage-Ring Network), a charge accumulator's (Titan Forge, Coalition
//! Relic, Lux Cannon, Midnight Clock) — was never activated either (6-seat
//! census, seed 1270001); it now takes an opponent's end step's spare mana,
//! when the tap costs nothing before the controller's untap.
//! Oblivion Stone's "put a fate counter on another target permanent" had the
//! same gap in four decks (census seed 3100001), so its sweep took its
//! controller's board too: the same end step now marks the controller's most
//! valuable unmarked nonland permanent.
//! Commander games only, so two-player play is unchanged.

use crate::card::CounterType;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// Does `e` grow its slot-0 target with a +1/+1 or loyalty counter, or mark
/// it with a divinity counter (Kindred Boon's indestructible)?
fn grows_target(e: &Effect) -> bool {
    target_counter_kind(e).is_some()
}

/// The counter `e` puts on its slot-0 target, of the kinds the sink buys.
fn target_counter_kind(e: &Effect) -> Option<CounterType> {
    match e {
        Effect::AddCounter {
            what: Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. },
            kind: kind @ (CounterType::PlusOnePlusOne | CounterType::Loyalty | CounterType::Divinity),
            ..
        } => Some(*kind),
        Effect::If { then, else_, .. } => target_counter_kind(then).or_else(|| target_counter_kind(else_)),
        _ => None,
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
        // A marking counter (divinity) does its work once: skip a creature
        // that already carries one. +1/+1 and loyalty stack.
        let marks = state.battlefield_find(source).and_then(|c| {
            target_counter_kind(&c.definition.activated_abilities[index].effect)
                .filter(|k| *k == CounterType::Divinity)
        });
        own.iter()
            .filter(|&&(_, id)| marks.is_none_or(|k| state.battlefield_find(id).is_some_and(|c| c.counter_count(k) == 0)))
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

/// "Put a +1/+1 / storage / charge counter on this", with no X in the cost.
fn grows_self(ab: &crate::card::ActivatedAbility) -> bool {
    matches!(
        ab.effect,
        Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne | CounterType::Storage | CounterType::Charge | CounterType::Hour,
            ..
        }
    ) && !ab.mana_cost.has_x()
        // A free untapped one would be taken every priority.
        && (ab.tap_cost || ab.mana_cost.cmc() > 0)
        && !ab.sac_cost
        && ab.sac_other_filter.is_none()
        && ab.tap_other_filter.is_none()
        && ab.discard_cost.is_none()
        && ab.life_cost == 0
        && ab.energy_cost == 0
        && ab.remove_counter_cost.is_none()
        && ab.remove_all_counters_cost.is_none()
        && ab.remove_counter_among_filter.is_none()
        && ab.exile_other_filter.is_none()
}

/// The first accepted self-counter sink on `seat`'s permanents — called at
/// an opponent's end step, where a tapped source untaps before it is needed.
pub(super) fn pick_self_counter_sink(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() || state.active_player_idx == seat {
        return None;
    }
    // Midnight Clock's hour counters run toward "shuffle your hand and
    // graveyard in, draw seven" — worth hurrying only with little in hand
    // (a 183-deck census never saw one bought, in seven decks).
    let small_hand = state.players[seat].hand.len() <= 2;
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        c.definition.activated_abilities.iter().enumerate().filter(|(_, ab)| grows_self(ab)).find_map(|(i, ab)| {
            if matches!(ab.effect, Effect::AddCounter { kind: CounterType::Hour, .. }) && !small_hand {
                return None;
            }
            let action = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: i,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

/// Lands at which a land in hand is spare enough to discard.
const SPARE_LAND_AT: usize = 4;
/// Hand size at which the cheapest card is spare even when it isn't a land.
const SPARE_HAND_AT: usize = 2;

/// "{1}, {T}, discard a card: put a study counter on this" (Grimoire of the
/// Dead — never charged in a 183-deck census, so its mass reanimation never
/// came): at an opponent's end step, with [`SPARE_LAND_AT`] lands out and a
/// land in hand or [`SPARE_HAND_AT`] cards for the cost to take (it pays with
/// the cheapest card). Six lands and a land in hand charged it in none of 12
/// four-seat games (seed 280000), so the payoff could never come; it taps, so
/// it charges at most once a round and the gate has to be loose.
pub(super) fn pick_discard_charge(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() || state.active_player_idx == seat {
        return None;
    }
    let lands = state.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land()).count();
    let hand = &state.players[seat].hand;
    if lands < SPARE_LAND_AT || !(hand.len() >= SPARE_HAND_AT || hand.iter().any(|c| c.definition.is_land())) {
        return None;
    }
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            let charges = matches!(ab.effect, Effect::AddCounter { what: Selector::This, .. })
                && ab.discard_cost.as_ref().is_some_and(|(_, n)| *n == 1)
                && !ab.discard_cost_x
                && !ab.discard_cost_same_name
                && !ab.sac_cost;
            if !charges {
                return None;
            }
            let action = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: i,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

/// "Put a fate counter on [another] target permanent" (Oblivion Stone).
fn marks_fate(e: &Effect) -> bool {
    matches!(
        e,
        Effect::AddCounter {
            what: Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. },
            kind: CounterType::Fate,
            ..
        }
    )
}

/// A fate counter on `seat`'s most valuable nonland permanent without one —
/// called at an opponent's end step, like [`pick_self_counter_sink`].
pub(super) fn pick_fate_sink(state: &GameState, seat: usize, w: &super::bot::EvalWeights) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() || state.active_player_idx == seat {
        return None;
    }
    let sinks: Vec<(crate::card::CardId, usize)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .filter(|(_, ab)| marks_fate(&ab.effect))
                .map(move |(i, _)| (c.id, i))
        })
        .collect();
    if sinks.is_empty() {
        return None;
    }
    let mut own: Vec<(i32, crate::card::CardId)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && !c.definition.is_land() && c.counter_count(CounterType::Fate) == 0)
        .map(|c| (super::bot::permanent_value(state, c.id, w), c.id))
        .collect();
    own.sort_by_key(|&(v, id)| (std::cmp::Reverse(v), id));
    own.iter().find_map(|&(_, id)| {
        sinks.iter().find_map(|&(card_id, ability_index)| {
            let action = GameAction::ActivateAbility {
                card_id,
                ability_index,
                target: Some(Target::Permanent(id)),
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

/// The fewest other creatures a "+1/+1 counter on each other creature you
/// control" tap must reach before it beats the same card's self-pump.
const MIN_SPREAD: usize = 2;

/// Mikaeus, the Lunarch's "{T}, remove a +1/+1 counter: put a +1/+1 counter
/// on each other creature you control" was never activated in a 183-deck
/// census: its own "{T}: put a +1/+1 counter on Mikaeus" takes the tap first,
/// every turn, through the self-pump sink. A tap that spreads +1/+1 counters
/// over at least [`MIN_SPREAD`] other creatures is taken in either main phase
/// of the controller's turn, ahead of that sink (before combat, the pumped
/// team attacks bigger).
pub(super) fn pick_team_counter_spread(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::game::types::TurnStep;
    if state.players.len() <= 2
        || !state.stack.is_empty()
        || state.active_player_idx != seat
        || !matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
    {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Effect::AddCounter { what: Selector::EachPermanent(filter), kind: CounterType::PlusOnePlusOne, .. } =
                &ab.effect
            else {
                continue;
            };
            if !ab.tap_cost || ab.sac_cost || ab.exhaust {
                continue;
            }
            let reached = state
                .battlefield
                .iter()
                .filter(|c| c.id != card.id && c.controller == seat && state.computed_is_creature(c))
                .filter(|c| state.evaluate_requirement_static(filter, &Target::Permanent(c.id), seat, Some(card.id)))
                .count();
            if reached < MIN_SPREAD {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
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

    /// Kindred Boon's divinity counter (indestructible) goes on the biggest
    /// creature of the chosen type without one, and not on one that has it.
    #[test]
    fn kindred_boon_marks_an_unmarked_creature() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        let boon = g.add_card_to_battlefield(0, crate::catalog::kindred_boon());
        g.battlefield_find_mut(boon).unwrap().chosen_creature_type = Some(crate::card::CreatureType::Bear);
        let big = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let small = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.battlefield_find_mut(big).unwrap().add_counters(CounterType::Divinity, 1);
        g.players[0].mana_pool.add(crate::mana::Color::White, 2);
        assert!(matches!(
            pick_counter_sink(&g, 0),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == boon && t == small
        ));
    }

    /// Grimoire of the Dead charges by discarding a spare land, and not with
    /// no land in hand.
    #[test]
    fn grimoire_of_the_dead_charges_with_a_spare_land() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        let book = g.add_card_to_battlefield(0, crate::catalog::grimoire_of_the_dead());
        for _ in 0..6 {
            g.add_card_to_battlefield(0, crate::catalog::swamp());
        }
        g.players[0].mana_pool.add_colorless(1);
        g.add_card_to_hand(0, crate::catalog::grizzly_bears());
        assert!(pick_discard_charge(&g, 0).is_none(), "no spare land in hand");
        g.add_card_to_hand(0, crate::catalog::swamp());
        assert!(matches!(
            pick_discard_charge(&g, 0),
            Some(GameAction::ActivateAbility { card_id, ability_index: 0, .. }) if card_id == book
        ));
    }

    /// With three study counters, the Grimoire is cashed in for the creature
    /// cards in every graveyard (the resolved-outcome sacrifice picker).
    #[test]
    fn grimoire_of_the_dead_cashes_in_when_charged() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        let book = g.add_card_to_battlefield(0, crate::catalog::grimoire_of_the_dead());
        g.battlefield_find_mut(book).unwrap().add_counters(CounterType::Study, 3);
        for seat in 1..3 {
            g.add_card_to_graveyard(seat, crate::catalog::craw_wurm());
        }
        let got = super::super::bot::pick_sacrifice_value(&g, 0, &super::super::bot::EvalWeights::default());
        assert!(matches!(got, Some(GameAction::ActivateAbility { card_id, ability_index: 1, .. }) if card_id == book), "{got:?}");
    }

    /// Midnight Clock's "{2}{U}: put an hour counter" is bought at an
    /// opponent's end step only when the seat's hand is nearly empty.
    #[test]
    fn midnight_clock_hurries_only_on_a_small_hand() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        let clock = g.add_card_to_battlefield(0, crate::catalog::midnight_clock());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 3);
        for _ in 0..4 {
            g.add_card_to_hand(0, crate::catalog::island());
        }
        assert!(pick_self_counter_sink(&g, 0).is_none(), "four cards in hand");
        g.players[0].hand.truncate(1);
        assert!(matches!(
            pick_self_counter_sink(&g, 0),
            Some(GameAction::ActivateAbility { card_id, ability_index: 1, .. }) if card_id == clock
        ));
    }

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

    /// Hangarback Walker grows at an opponent's end step with a spare mana,
    /// never on its controller's own turn (the tap would cost an attack).
    #[test]
    fn hangarback_grows_at_an_opponents_end_step() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let walker = g.add_card_to_battlefield(0, crate::catalog::hangarback_walker());
        // A 0/0 with no counter dies to the sweep after any action.
        g.battlefield_find_mut(walker).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.clear_sickness(walker);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::End;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(pick_self_counter_sink(&g, 0).is_none(), "own turn");
        g.active_player_idx = 1;
        let a = pick_self_counter_sink(&g, 0).expect("grow");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, .. } if card_id == walker));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert_eq!(g.battlefield_find(walker).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    }

    /// Oblivion Stone marks its controller's best nonland permanent (never
    /// itself — "another target permanent") at an opponent's end step.
    #[test]
    fn oblivion_stone_marks_the_best_permanent_it_would_sweep() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let stone = g.add_card_to_battlefield(0, crate::catalog::oblivion_stone());
        g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let wurm = g.add_card_to_battlefield(0, crate::catalog::craw_wurm());
        g.add_card_to_battlefield(0, crate::catalog::forest());
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::End;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 0;
        let w = crate::server::bot::EvalWeights::default();
        let on_itself = GameAction::ActivateAbility {
            card_id: stone, ability_index: 0, target: Some(Target::Permanent(stone)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        };
        assert!(!g.would_accept(on_itself), "another target permanent");
        let a = pick_fate_sink(&g, 0, &w).expect("mark");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
            if card_id == stone && t == wurm));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert_eq!(g.battlefield_find(wurm).unwrap().counter_count(CounterType::Fate), 1);
    }

    /// Mikaeus, the Lunarch spreads a counter over two others before its
    /// self-pump can take the tap, and grows itself with only one other.
    #[test]
    fn mikaeus_spreads_before_its_self_pump() {
        use crate::game::types::TurnStep;
        let mut g = crate::game::multi_player_game(4);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let m = g.add_card_to_battlefield(0, crate::catalog::mikaeus_the_lunarch());
        g.clear_sickness(m);
        g.battlefield_find_mut(m).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
        g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        assert!(pick_team_counter_spread(&g, 0).is_none(), "one other creature: not worth a counter");
        g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        assert!(matches!(
            pick_team_counter_spread(&g, 0),
            Some(GameAction::ActivateAbility { card_id, ability_index: 1, .. }) if card_id == m
        ));
    }
}
