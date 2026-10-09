//! CR 104.4b / 732.4 — the bot must not start a mandatory loop of its own.
//! Gremlin Tamer under Secret Arcade: each Gremlin token is an enchantment,
//! so its arrival re-triggers Eerie without end and the engine's growth-loop
//! watch (`game/growth_loop.rs`) draws the game. The static scorer can't see
//! a loop that only resolution makes, so a pod candidate whose settled state
//! leaves a self-feeding token trigger (one whose own token re-triggers it)
//! is probed here and dropped. Commander games only; two-player play is
//! unchanged.

use crate::card::CardInstance;
use crate::card::TokenDefinition;
use crate::effect::{Effect, EventKind, EventScope, PlayerRef};
use crate::game::GameState;
use crate::game::types::StackItem;

/// The mandatory "a permanent enters → create a token for you" triggers on
/// `def`, as `(ability index, token)`. A self-ETB, a `may`, a per-turn cap or
/// a token handed to someone else can't loop on its own.
fn token_triggers(def: &crate::card::CardDefinition) -> Vec<(usize, &TokenDefinition)> {
    def.triggered_abilities
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            t.event.kind == EventKind::EntersBattlefield
                && matches!(t.event.scope, EventScope::YourControl | EventScope::AnotherOfYours | EventScope::AnyPlayer)
                && !t.event.once_per_turn
        })
        .filter_map(|(i, t)| {
            let leaf = match &t.effect {
                Effect::Seq(steps) => steps.iter().find(|e| matches!(e, Effect::CreateToken { .. }))?,
                e => e,
            };
            match leaf {
                Effect::CreateToken { who: PlayerRef::You, definition, .. } => Some((i, &**definition)),
                _ => None,
            }
        })
        .collect()
}

fn seat_permanent_spells(state: &GameState, seat: usize) -> impl Iterator<Item = &CardInstance> {
    state.stack.iter().filter_map(move |item| match item {
        StackItem::Spell { card, caster, .. } if *caster == seat && card.definition.is_permanent() => Some(&**card),
        _ => None,
    })
}

/// Cheap gate: `seat` has (on the battlefield or as a spell) a source a
/// self-feeding loop could start from.
fn has_token_trigger(state: &GameState, seat: usize) -> bool {
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .chain(seat_permanent_spells(state, seat))
        .any(|c| !token_triggers(&c.definition).is_empty())
}

/// True when one of `seat`'s token triggers would be re-triggered by its own
/// token. Probed on a clone: each permanent spell of `seat`'s on the stack is
/// minted as the permanent it becomes, then each trigger's token is minted
/// and its entry matched against the trigger with the engine's own matcher,
/// so layer-added types (Secret Arcade) are seen exactly as resolution would.
pub(super) fn self_feeding(state: &GameState, seat: usize) -> bool {
    if !has_token_trigger(state, seat) {
        return false;
    }
    let mut h = state.clone();
    let mut events = Vec::new();
    let spells: Vec<_> = seat_permanent_spells(state, seat)
        .map(|c| (c.definition.arc(), c.definition.room.is_some().then_some(c.split_cast == Some(1))))
        .collect();
    for (def, room_door) in spells {
        let id = h.mint_token_onto_battlefield(def, seat, false, &mut events);
        // CR 709.5d — a Room enters with the cast door unlocked.
        if let Some(right) = room_door {
            h.set_room_door_unlocked(id, right, &mut events);
        }
    }
    let sources: Vec<(crate::card::CardId, usize, TokenDefinition)> = h
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| token_triggers(&c.definition).into_iter().map(move |(i, t)| (c.id, i, t.clone())))
        .collect();
    for (src, idx, token) in sources {
        events.clear();
        let def = crate::game::effects::token_to_card_definition(&token);
        h.mint_token_onto_battlefield(def, seat, false, &mut events);
        let Some(source) = h.battlefield_find(src) else { continue };
        let spec = &source.definition.triggered_abilities[idx].event;
        let token_id = events.iter().find_map(|ev| match ev {
            crate::game::GameEvent::PermanentEntered { card_id } => Some(*card_id),
            _ => None,
        });
        let fires = |ev: &crate::game::GameEvent| {
            crate::game::effects::events::event_matches_spec(&h, ev, spec, source)
                && spec.filter.as_ref().is_none_or(|f| {
                    let mut c = crate::game::effects::EffectContext::for_trigger(source.id, seat, None, 0);
                    c.trigger_source = token_id.map(crate::game::effects::EntityRef::Card);
                    h.evaluate_predicate(f, &c)
                })
        };
        if events.iter().any(fires) {
            return true;
        }
    }
    false
}

/// Per-tick gate for [`starts_loop`]: a Commander seat with a token trigger
/// in play, in hand or in the command zone. Off, the probe costs nothing.
pub(super) fn watch(state: &GameState, seat: usize) -> bool {
    let p = &state.players[seat];
    !p.commanders.is_empty()
        && (has_token_trigger(state, seat)
            || p.hand.iter().chain(p.command.iter()).any(|c| !token_triggers(&c.definition).is_empty()))
}

/// Whether taking the action that settled `pre` into `post` starts (or sets
/// up) a loop `seat` can't stop: the board becomes self-feeding, or already
/// was and the action brings in another permanent.
pub(super) fn starts_loop(pre: &GameState, post: &GameState, seat: usize) -> bool {
    if pre.players[seat].commanders.is_empty() || !has_token_trigger(post, seat) || !self_feeding(post, seat) {
        return false;
    }
    !self_feeding(pre, seat) || seat_permanent_spells(post, seat).count() > seat_permanent_spells(pre, seat).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::{GameAction, TurnStep};
    use crate::mana::Color;

    fn arcade_game() -> (GameState, crate::card::CardId) {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let room = g.add_card_to_battlefield(0, crate::catalog::secret_arcade_dusty_parlor());
        (g, room)
    }

    /// CR 104.4b — Gremlin Tamer cast under an unlocked Secret Arcade enters
    /// as an enchantment, and every Gremlin after it does too: the bot sees
    /// the loop before casting. Without the Arcade the same cast is fine.
    #[test]
    fn casting_gremlin_tamer_under_secret_arcade_is_a_loop() {
        let (mut g, room) = arcade_game();
        let tamer = g.add_card_to_hand(0, crate::catalog::gremlin_tamer());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        let cast = GameAction::CastSpell {
            card_id: tamer,
            target: None,
            additional_targets: Vec::new(),
            mode: None,
            x_value: None,
        };
        let locked = GameState::accept_on(&g, cast.clone()).expect("castable");
        assert!(!starts_loop(&g, &locked, 0), "a locked Arcade adds no type");
        g.set_room_door_unlocked(room, false, &mut Vec::new());
        let post = GameState::accept_on(&g, cast).expect("castable");
        assert!(starts_loop(&g, &post, 0));
    }

    /// CR 709.5d / 104.4b — casting Secret Arcade (it enters with that door
    /// unlocked) under Ghostly Dancers sets the loop up too.
    #[test]
    fn casting_secret_arcade_under_ghostly_dancers_is_a_loop() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, crate::catalog::ghostly_dancers());
        let room = g.add_card_to_hand(0, crate::catalog::secret_arcade_dusty_parlor());
        g.players[0].mana_pool.add(Color::White, 5);
        let cast = GameAction::CastSpell {
            card_id: room,
            target: None,
            additional_targets: Vec::new(),
            mode: None,
            x_value: None,
        };
        let post = GameState::accept_on(&g, cast).expect("castable");
        assert!(starts_loop(&g, &post, 0));
    }

    /// CR 104.4b — unlocking Secret Arcade under a Gremlin Tamer sets the loop
    /// up (any later permanent starts it); once set up, a permanent spell
    /// starts it and an instant doesn't.
    #[test]
    fn unlocking_secret_arcade_under_gremlin_tamer_is_a_loop() {
        let (mut g, room) = arcade_game();
        g.add_card_to_battlefield(0, crate::catalog::gremlin_tamer());
        g.players[0].mana_pool.add(Color::White, 5);
        let unlock = GameAction::UnlockRoomDoor { card_id: room, right: false };
        let post = GameState::accept_on(&g, unlock).expect("unlockable");
        assert!(starts_loop(&g, &post, 0));
        assert!(!self_feeding(&g, 0));
        assert!(self_feeding(&post, 0));
    }
}

