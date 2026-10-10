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
use crate::game::types::{GameAction, StackItem};

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

/// A token-making trigger that isn't a one-shot self-ETB: the piece every
/// growing loop needs (Polyraptor's damage copy, an Eerie mint).
fn makes_tokens_repeatably(def: &crate::card::CardDefinition) -> bool {
    def.triggered_abilities.iter().any(|t| {
        let self_etb = t.event.kind == EventKind::EntersBattlefield && t.event.scope == EventScope::SelfSource;
        !(self_etb || t.event.once_per_turn)
            && t.effect.any_nested(&|e| matches!(e, Effect::CreateToken { .. } | Effect::CreateTokenCopyOf { .. }))
    })
}

/// Per-tick gate for the probes: a Commander seat with a repeatable token
/// trigger anywhere it could come from (battlefield, hand, graveyard, command
/// zone), a face-down permanent's real face included — turning a manifested
/// Ghostly Dancers up under Secret Arcade started the loop (pod sweep 182001,
/// four seats, decks 271/233/285/18, game 3). Off, the probes cost nothing.
pub(super) fn watch(state: &GameState, seat: usize) -> bool {
    let p = &state.players[seat];
    !p.commanders.is_empty()
        && state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .chain(p.hand.iter())
            .chain(p.graveyard.iter())
            .chain(p.command.iter())
            .any(|c| {
                makes_tokens_repeatably(&c.definition)
                    || c.face_up_def.as_deref().is_some_and(makes_tokens_repeatably)
            })
}

/// Passes per seat the resolution probe may spend.
const PROBE_PASSES_PER_SEAT: usize = 72;
/// Board growth that, with the stack never draining, reads as a loop.
const PROBE_GROWTH: usize = 24;

/// What resolving a stack to empty showed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Probe {
    /// A CR 104.4b draw, or a stack that never drained while the board grew
    /// by [`PROBE_GROWTH`].
    Loop,
    /// The stack emptied (or the game ended decided).
    Drained,
    /// Out of budget without growth, or the engine refused a step.
    Unknown,
}

/// Resolve `g` in place with every seat passing and each ask answered by the
/// asked seat's policy.
fn resolve_probe(g: &mut GameState, w: &super::bot::EvalWeights) -> Probe {
    let board0 = g.battlefield.len();
    for _ in 0..PROBE_PASSES_PER_SEAT * g.players.len() {
        if g.is_game_over() {
            return if matches!(g.game_over, Some(None)) { Probe::Loop } else { Probe::Drained };
        }
        if g.stack.is_empty() && g.pending_decision.is_none() {
            return Probe::Drained;
        }
        let action = match g.pending_decision.as_ref() {
            Some(p) => {
                let who = p.acting_player();
                GameAction::SubmitDecision(super::bot::decide_pending_policy(g, who, w, &p.decision, false))
            }
            None => GameAction::PassPriority,
        };
        if g.perform_action_inner(action).is_err() {
            return Probe::Unknown;
        }
    }
    // Not "no shallower than it began": a loop under a Room spell and two
    // cast triggers settles at depth 2 below their 3 (seed 4600500 game 4).
    if !g.stack.is_empty() && g.battlefield.len() >= board0 + PROBE_GROWTH { Probe::Loop } else { Probe::Unknown }
}

/// Resolve `post` (an action's settled state): a loop already running — Polyraptor cast beside Marauding Raptor, a Room door
/// returning Ghostly Dancers under Secret Arcade. A finite cascade drains.
#[cfg(test)]
fn resolution_loops(post: &GameState, w: &super::bot::EvalWeights) -> bool {
    !post.stack.is_empty() && resolve_probe(&mut post.clone(), w) == Probe::Loop
}

/// Whether firing one of `seat`'s repeatable token triggers on the quiet
/// board `g` (stack empty) loops: Marauding Raptor beside Polyraptor waits
/// for the first damage, Secret Arcade beside Gremlin Tamer for the next
/// enchantment. One probe per distinct trigger; a targeted one is skipped.
fn primed(g: &GameState, seat: usize, w: &super::bot::EvalWeights) -> bool {
    let mut seen: Vec<(&str, usize)> = Vec::new();
    for c in g.battlefield.iter().filter(|c| c.controller == seat) {
        for (i, t) in c.definition.triggered_abilities.iter().enumerate() {
            let self_etb = t.event.kind == EventKind::EntersBattlefield && t.event.scope == EventScope::SelfSource;
            if self_etb
                || t.event.once_per_turn
                || t.effect.requires_target()
                || !t.effect.any_nested(&|e| matches!(e, Effect::CreateToken { .. } | Effect::CreateTokenCopyOf { .. }))
                || seen.contains(&(c.definition.name, i))
            {
                continue;
            }
            seen.push((c.definition.name, i));
            let mut h = g.clone();
            h.push_stack(
                crate::game::types::TriggerPush::new(c.id, seat, t.effect.clone())
                    .trigger_source(Some(crate::game::effects::EntityRef::Permanent(c.id)))
                    .build(),
            );
            if resolve_probe(&mut h, w) == Probe::Loop {
                return true;
            }
        }
    }
    false
}

/// The action that settled `pre` into `post` leaves a loop running or primed
/// that `pre` didn't have.
pub(super) fn sets_up_loop(pre: &GameState, post: &GameState, seat: usize, w: &super::bot::EvalWeights) -> bool {
    let mut g = post.clone();
    match resolve_probe(&mut g, w) {
        Probe::Loop => true,
        Probe::Unknown => false,
        Probe::Drained => {
            if g.is_game_over() || !g.stack.is_empty() || !primed(&g, seat, w) {
                false
            } else {
                !(pre.stack.is_empty() && primed(pre, seat, w))
            }
        }
    }
}

/// The last gate on any pod action but a pass, an answer or a combat
/// declaration: one that starts or primes a mandatory loop is declined. The
/// finalist and Room pickers ask first; this catches the rest — Marina's
/// door flip or a sink's activation priming Secret Arcade under Ghostly
/// Dancers (seed 4600500).
pub(super) fn action_starts_loop(
    state: &GameState,
    seat: usize,
    step: &super::bot::BotStep,
    w: &super::bot::EvalWeights,
) -> bool {
    if matches!(
        step.action,
        GameAction::PassPriority
            | GameAction::SubmitDecision(_)
            | GameAction::DeclareAttackers(_)
            | GameAction::DeclareBlockers(_)
    ) || !watch(state, seat)
    {
        return false;
    }
    let owned;
    let post = match step.settled.as_deref() {
        Some(g) => g,
        None => match GameState::accept_on(state, step.action.clone()) {
            Some(g) => {
                owned = g;
                &owned
            }
            None => return false,
        },
    };
    starts_loop(state, post, seat) || sets_up_loop(state, post, seat, w)
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

    /// CR 104.4b — Polyraptor cast beside Marauding Raptor: each copy enters,
    /// is dealt 2, and copies itself. Only resolution shows it; the probe
    /// does, and a plain creature cast is no loop.
    #[test]
    fn polyraptor_beside_marauding_raptor_resolves_into_a_loop() {
        let (mut g, _) = arcade_game();
        g.add_card_to_battlefield(0, crate::catalog::marauding_raptor());
        let poly = g.add_card_to_hand(0, crate::catalog::polyraptor());
        let bear = g.add_card_to_hand(0, crate::catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 8);
        let w = crate::server::bot::EvalWeights::default();
        let cast = |id| GameAction::CastSpell { card_id: id, target: None, additional_targets: Vec::new(), mode: None, x_value: None };
        let post = GameState::accept_on(&g, cast(poly)).expect("castable");
        assert!(resolution_loops(&post, &w));
        let post = GameState::accept_on(&g, cast(bear)).expect("castable");
        assert!(!resolution_loops(&post, &w));
    }

    /// CR 104.4b — Marauding Raptor cast beside Polyraptor starts nothing yet,
    /// but primes the loop for Polyraptor's first damage; a plain creature
    /// primes nothing.
    #[test]
    fn marauding_raptor_beside_polyraptor_sets_up_a_loop() {
        let (mut g, _) = arcade_game();
        g.add_card_to_battlefield(0, crate::catalog::polyraptor());
        let raptor = g.add_card_to_hand(0, crate::catalog::marauding_raptor());
        let bear = g.add_card_to_hand(0, crate::catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add(Color::Green, 2);
        let w = crate::server::bot::EvalWeights::default();
        let cast = |id| GameAction::CastSpell { card_id: id, target: None, additional_targets: Vec::new(), mode: None, x_value: None };
        let post = GameState::accept_on(&g, cast(raptor)).expect("castable");
        assert!(!resolution_loops(&post, &w), "nothing loops yet");
        assert!(sets_up_loop(&g, &post, 0, &w));
        let post = GameState::accept_on(&g, cast(bear)).expect("castable");
        assert!(!sets_up_loop(&g, &post, 0, &w));
    }

    /// CR 104.4b — turning a manifested Ghostly Dancers face up under an
    /// unlocked Secret Arcade, with a creature spell of yours on the stack,
    /// starts the loop: the gate read only the face-down 2/2 and let it through
    /// (pod sweep 182001, decks 271/233/285/18, game 3).
    #[test]
    fn turning_ghostly_dancers_face_up_under_secret_arcade_is_a_loop() {
        let (mut g, room) = arcade_game();
        g.set_room_door_unlocked(room, false, &mut Vec::new());
        let dancers = g.add_card_to_graveyard(0, crate::catalog::ghostly_dancers());
        let manifest = Effect::ManifestFromGraveyard {
            who: PlayerRef::Seat(0),
            filter: crate::card::SelectionRequirement::Creature,
        };
        let ctx = crate::game::effects::EffectContext::for_ability(room, 0, None);
        g.resolve_effect(&manifest, &ctx).expect("manifest");
        assert!(g.battlefield_find(dancers).is_some_and(|c| c.face_down));
        let bear = g.add_card_to_hand(0, crate::catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: Vec::new(), mode: None, x_value: None })
            .expect("bear on the stack");
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::White, 5);
        let w = crate::server::bot::EvalWeights::default();
        let up = crate::server::bot::BotStep::plain(GameAction::TurnFaceUp { card_id: dancers });
        assert!(action_starts_loop(&g, 0, &up, &w));
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

