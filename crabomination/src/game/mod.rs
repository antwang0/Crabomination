//! Core MTG game engine.
//!
//! # Design notes
//! - **Stack & priority**: a real LIFO `stack` of spells and abilities, drained
//!   by a multiplayer priority loop (`pass_priority`). Spells and abilities are
//!   pushed onto the stack and resolve only once all players pass in succession;
//!   players can respond while they hold priority.
//! - **Combat damage**: first-strike and regular combat damage are split into
//!   separate sub-steps (the `FirstStrikeDamage` step, skipped when no
//!   first/double-strike creatures are in combat). A DoubleStrike creature
//!   deals damage in both steps.
//! - **Hexproof/Shroud**: validated at targeting time.
//! - **Menace**: enforced in `declare_blockers` — an attacker with Menace must
//!   be blocked by ≥ 2 creatures or not blocked at all.
//! - **Dies triggers**: fired when a creature moves from battlefield to
//!   graveyard (via damage, destroy, or state-based actions).
//! - Actions are performed by whichever player currently holds priority (so a
//!   non-active player can cast instants / activate abilities in response);
//!   `declare_blockers` is called by whoever controls the defending creatures.

pub(crate) mod actions;
pub(crate) mod combat;
pub(crate) mod effects;
pub mod layers;
pub(crate) mod stack;
#[cfg(test)]
#[path = "../tests/game.rs"]
mod tests;
#[cfg(test)]
#[path = "../tests/modern.rs"]
mod tests_modern;
#[cfg(test)]
#[path = "../tests/sos.rs"]
mod tests_sos;
#[cfg(test)]
#[path = "../tests/stx/mod.rs"]
mod tests_stx;
#[cfg(test)]
#[path = "../tests/multiplayer.rs"]
mod tests_multiplayer;
#[cfg(test)]
#[path = "../tests/xtra.rs"]
mod tests_xtra;
#[cfg(test)]
#[path = "../tests/combat_keywords.rs"]
mod tests_combat_keywords;
#[cfg(test)]
#[path = "../tests/classic.rs"]
mod tests_classic;
#[cfg(test)]
#[path = "../tests/counters.rs"]
mod tests_counters;
#[cfg(test)]
#[path = "../tests/energy.rs"]
mod tests_energy;
#[cfg(test)]
#[path = "../tests/ktk.rs"]
mod tests_ktk;
pub mod types;

#[cfg(test)]
pub(crate) fn two_player_game() -> GameState {
    multi_player_game(2)
}

/// `n`-player game (n ≥ 1), pre-advanced to the active player's pre-combat
/// main phase. Players are named "P0", "P1", …. Use for free-for-all
/// multiplayer tests; for format-specific life totals call
/// `game_with_format(format, n)`.
#[cfg(test)]
pub(crate) fn multi_player_game(n: usize) -> GameState {
    let players: Vec<_> = (0..n)
        .map(|i| crate::player::Player::new(i, format!("P{i}")))
        .collect();
    let mut g = GameState::new(players);
    g.step = TurnStep::PreCombatMain;
    g
}

/// `n`-player game with format-specific setup applied (starting life, draw-on-
/// turn-1 rule). Pre-advanced to the pre-combat main phase like
/// `two_player_game`.
#[cfg(test)]
pub(crate) fn game_with_format(format: crate::format::Format, n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.apply_format(format);
    g
}

/// Pass priority for both players until the stack is empty, returning all
/// events produced during resolution. Callers that don't care about events
/// can simply discard the return value.
#[cfg(test)]
pub(crate) fn drain_stack(g: &mut GameState) -> Vec<GameEvent> {
    let mut all_events = Vec::new();
    while !g.stack.is_empty() {
        all_events.extend(g.perform_action(GameAction::PassPriority).unwrap());
        all_events.extend(g.perform_action(GameAction::PassPriority).unwrap());
    }
    all_events
}

/// Cast a spell with no target and drain the stack. Returns resolve events.
/// Tests with non-default `mode`/`x_value`, the error path, or that need to
/// inspect cast-time events separately should use `GameAction::CastSpell`
/// directly.
#[cfg(test)]
pub(crate) fn cast(g: &mut GameState, id: CardId) -> Vec<GameEvent> {
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast spell");
    drain_stack(g)
}

/// Cast a spell at a specific target and drain the stack.
#[cfg(test)]
pub(crate) fn cast_at(g: &mut GameState, id: CardId, target: Target) -> Vec<GameEvent> {
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(target), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast spell at target");
    drain_stack(g)
}

pub use types::*;

use crate::card::{CardDefinition, CardId, CardInstance, CardType, Keyword, SelectionRequirement};
use crate::decision::{AutoDecider, Decider, DeciderKind, Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::game::effects::EffectContext;
use crate::game::layers::{
    AffectedPermanents, ComputedPermanent, ContinuousEffect, EffectDuration, Layer, Modification,
    PtSublayer,
};
use crate::player::Player;
use std::collections::HashMap;

// ── Decider serde adapter ────────────────────────────────────────────────────
//
// `Box<dyn Decider>` can't directly derive serde, so we project it to
// `DeciderKind` (which IS serializable) on the wire and reconstitute on
// load. Custom deciders not modeled by the kind enum collapse to
// `AutoDecider` after a round-trip.

#[allow(clippy::borrowed_box)] // serde derive needs `&Box<T>` here
fn serialize_decider<S: serde::Serializer>(
    decider: &Box<dyn Decider + Send + Sync>,
    ser: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    decider.kind().serialize(ser)
}

fn deserialize_decider<'de, D: serde::Deserializer<'de>>(
    de: D,
) -> Result<Box<dyn Decider + Send + Sync>, D::Error> {
    use serde::Deserialize;
    let kind = DeciderKind::deserialize(de)?;
    Ok(kind.into_boxed())
}

// ── Game state ────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameState {
    pub players: Vec<Player>,
    /// Partition of seats into teams. Every seat appears in exactly one
    /// entry; free-for-all formats have one singleton team per seat,
    /// team formats (Two-Headed Giant) have multiple seats per team.
    /// Populated by `GameState::new`; reshape with `assign_teams`.
    /// Defaults to empty for snapshots predating the field — helpers
    /// (`team_of`, `teammates`, `opponents_of`) treat empty as "each
    /// seat is its own singleton team".
    #[serde(default)]
    pub teams: Vec<crate::team::Team>,
    /// All permanents currently in play.
    pub battlefield: Vec<CardInstance>,
    /// Cards that have been exiled.
    pub exile: Vec<CardInstance>,
    /// The stack of spells and triggered abilities waiting to resolve (LIFO).
    pub stack: Vec<StackItem>,
    pub step: TurnStep,
    /// Index into `players` of the player whose turn it is.
    pub active_player_idx: usize,
    pub turn_number: u32,
    /// `None` while the game is ongoing; `Some(None)` for a draw;
    /// `Some(Some(i))` when player `i` has won.
    pub game_over: Option<Option<usize>>,
    /// Priority state — tracks who can act and when the stack resolves.
    pub priority: PriorityState,
    /// Active continuous effects from resolved spells, abilities, and static abilities.
    pub continuous_effects: Vec<ContinuousEffect>,
    pub(crate) next_effect_timestamp: u64,
    pub(crate) next_id: u32,
    /// Attackers declared this combat, each with the player or planeswalker
    /// it is attacking.
    pub(crate) attacking: Vec<Attack>,
    /// Blocker → attacker mapping for the current combat.
    pub(crate) block_map: HashMap<CardId, CardId>,
    /// Set to true once `declare_blockers` has been called during the current DeclareBlockers step.
    pub(crate) blockers_declared: bool,
    /// Skip the draw on the very first turn (turn 1, first player).
    pub(crate) skip_first_draw: bool,
    /// Count of spells cast this turn (for Storm and related effects).
    pub spells_cast_this_turn: u32,
    /// Delayed triggered abilities registered by resolved spells/abilities
    /// (Pact upkeep cost, Goryo's exile-at-EOT, etc.). Fired by the step
    /// dispatcher when the matching event occurs.
    pub delayed_triggers: Vec<DelayedTrigger>,
    /// Transient: power of the most recently sacrificed creature within the
    /// current effect resolution. Set by `Effect::SacrificeAndRemember` and
    /// read by `Value::SacrificedPower` (e.g. Thud). Reset between
    /// independent spell/ability resolutions.
    pub(crate) sacrificed_power: Option<i32>,
    /// Transient: toughness of the most recently sacrificed creature within
    /// the current effect resolution. Set by `Effect::SacrificeAndRemember`
    /// alongside `sacrificed_power`; read by `Value::SacrificedToughness`
    /// (Tribute to Hunger). Reset between independent resolutions.
    #[serde(default)]
    pub(crate) sacrificed_toughness: Option<i32>,
    /// Transient: id of the most-recently-created token within the current
    /// effect resolution. Set by `Effect::CreateToken` and read by
    /// `Selector::LastCreatedToken` so a follow-up `AddCounter` /
    /// `PumpPT` / etc. in the same `Effect::Seq` can target the freshly
    /// minted token (Fractal Anomaly, Applied Geometry). Reset between
    /// independent resolutions.
    #[serde(skip)]
    pub(crate) last_created_token: Option<CardId>,
    /// Transient: ids of all tokens created within the current effect
    /// resolution. Set by `Effect::CreateToken`
    /// alongside `last_created_token` and read by
    /// `Selector::LastCreatedTokens` (plural) so a follow-up `AddCounter`
    /// in the same resolution can fan over every freshly-minted token
    /// (Fractal Spawning, Mascot Exhibition-style printed Oracles). Cleared
    /// at every resolution root start (see `reset_effect_scratch`).
    #[serde(skip)]
    pub(crate) last_created_tokens: Vec<CardId>,
    /// Transient: ids of every card moved within the current effect
    /// resolution. Populated by `Effect::Move` (and the mill/exile
    /// helpers) and read by `Selector::LastMoved` so a follow-up
    /// `GrantMayPlay` in the same `Effect::Seq` can target exactly the
    /// card(s) that were just lifted to exile/graveyard (Practiced
    /// Scrollsmith, Suspend Aggression, Tablet of Discovery, etc.).
    /// Cleared between resolutions.
    #[serde(skip)]
    pub(crate) last_moved_cards: Vec<CardId>,
    /// Transient: count of cards discarded within the current effect
    /// resolution. Bumped by every `GameEvent::CardDiscarded` emission
    /// inside `Effect::Discard` / `Effect::DiscardChosen` (random and
    /// player-chosen branches). Read by `Value::CardsDiscardedThisEffect`
    /// so a later step in the same `Effect::Seq` can draw N where N =
    /// "the number of cards discarded this way" (Borrowed Knowledge
    /// mode 1, Colossus of the Blood Age, etc.). Reset to 0 between
    /// independent resolutions.
    #[serde(skip)]
    pub(crate) cards_discarded_this_resolution: u32,
    /// Transient: count of *creature* cards discarded within the current
    /// effect resolution. Bumped alongside `cards_discarded_this_resolution`
    /// when the discarded card carries `CardType::Creature`. Read by
    /// `Value::CreatureCardsDiscardedThisEffect` so a follow-up step in
    /// the same `Effect::Seq` can fire only when a creature was discarded
    /// (Plargg, Dean of Chaos's printed conditional 2-damage rider).
    /// Reset to 0 between independent resolutions.
    #[serde(skip)]
    pub(crate) creature_cards_discarded_this_resolution: u32,
    /// Transient: per-player count of cards discarded within the current
    /// effect resolution, indexed by player seat. Bumped alongside the
    /// flat `cards_discarded_this_resolution` whenever a discard event
    /// fires, so a follow-up step in the same `Effect::Seq` can read the
    /// *greatest* count across players. Used by Windfall's printed
    /// "draws cards equal to the greatest number of cards a player
    /// discarded this way" via `Value::MaxCardsDiscardedThisEffectByAnyPlayer`.
    /// Reset to empty between independent resolutions.
    #[serde(skip)]
    pub(crate) cards_discarded_per_player_this_resolution: std::collections::HashMap<usize, u32>,
    /// Transient: the `CardId`s of cards discarded within the current
    /// effect resolution. Populated alongside the count fields above. Used
    /// by Mind Roots's "Put up to one land card discarded this way onto
    /// the battlefield tapped" rider — the engine walks this list at
    /// resolution time, finds the first Land card, and moves it onto the
    /// battlefield via `Effect::MoveDiscardedLandToBattlefield`. Reset
    /// to empty between independent resolutions.
    #[serde(skip)]
    pub(crate) discarded_card_ids_this_resolution: Vec<CardId>,
    /// Transient: count of permanents destroyed by `Effect::Destroy` within
    /// the current resolution. Read by `Value::PermanentsDestroyedThisResolution`
    /// so a follow-up `Effect::Seq` step can scale off the kill count
    /// (Culling Ritual's "Add {B} or {G} for each permanent destroyed this
    /// way"). Counts only permanents that actually reach the graveyard —
    /// indestructible / shielded survivors don't bump it. Reset to 0
    /// between independent resolutions.
    #[serde(skip)]
    pub(crate) permanents_destroyed_this_resolution: u32,
    /// Transient: the card name chosen by an `Effect::NameCard` within the
    /// current resolution. Read by `SelectionRequirement::NamedBySource` so a
    /// reveal-until-the-named-card chain (Spoils of the Vault) can match even
    /// when the naming source is a resolving spell held off to the side.
    /// Reset between independent resolutions.
    #[serde(skip)]
    pub(crate) named_card_this_resolution: Option<String>,
    /// Transient: which face / cast path the in-progress cast is using.
    /// Set by `cast_spell_back_face` (`Back`) and `cast_flashback`
    /// (`Flashback`); reset to `Front` after each emitted SpellCast
    /// event. Threaded into `GameEvent::SpellCast.face` so replays can
    /// distinguish a back-face MDFC cast from a normal hand cast.
    #[serde(skip, default)]
    pub(crate) pending_cast_face: CastFace,
    /// Resolves player choices encountered during effect resolution. Used for
    /// *non-suspending* decisions (e.g. `AddManaAnyColor` auto-picks a color).
    /// Suspending decisions (currently Scry) surface through `pending_decision`
    /// instead; the UI/bot replies via `submit_decision`.
    ///
    /// Serialized via the `decider_kind` adapter — see `DeciderKind` —
    /// so the trait object round-trips through JSON.
    #[serde(serialize_with = "serialize_decider", deserialize_with = "deserialize_decider")]
    pub decider: Box<dyn Decider + Send + Sync>,
    /// Set when effect resolution needs player input. Check each frame in the
    /// client to render the appropriate decision modal; clear via
    /// `submit_decision`. While `Some`, no other game actions are permitted.
    pub pending_decision: Option<PendingDecision>,
    /// One-shot signal from `resolve_effect` to the enclosing resolver when an
    /// effect needs to suspend. Callers check this after each effect call, wrap
    /// it up in `pending_decision` with the full resume context, and return.
    /// `remaining` carries any sibling effects still queued behind the one that
    /// suspended (e.g. `Draw` after `Scry` in a Seq).
    pub(crate) suspend_signal: Option<(Decision, PendingEffectState, Effect)>,
    /// True when an effect has flagged "prevent all combat damage this turn"
    /// (CR 615 — damage prevention as a replacement effect). Wired by
    /// Owlin Shieldmage's ETB trigger, Holy Day, Hallowed Burial-adjacent
    /// "fog" patterns. Cleared in `do_cleanup` alongside the other
    /// until-end-of-turn flags. Combat damage resolution (`resolve_combat_
    /// damage_with_filter`) consults this flag and skips dealing the
    /// damage half (lifelink, deathtouch, infect/wither, trigger emission
    /// for non-damage knock-ons all still resolve — only the damage
    /// number itself is set to 0 per CR 615.1).
    #[serde(default)]
    pub(crate) prevent_combat_damage_this_turn: bool,
    /// CR 614.9 / 615 — creatures whose combat damage is prevented in both
    /// directions for the rest of the turn (Maze of Ith: "prevent all combat
    /// damage that would be dealt to and dealt by that creature"). The combat
    /// resolver skips dealing *and* receiving combat damage for any creature
    /// in this set. Cleared at cleanup.
    #[serde(default)]
    pub(crate) combat_damage_prevented_creatures: Vec<CardId>,
    /// Active prevention shields (CR 615.1) around players/permanents.
    /// Created by `Effect::PreventNextDamage` / `PreventAllDamageThisTurn`;
    /// consulted by the non-combat damage path (`deal_damage_to_from`) and
    /// cleared at cleanup. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) prevention_shields: Vec<crate::game::types::PreventionShield>,
    /// CR 615.12 — "Damage can't be prevented this turn" (Skullcrack,
    /// Impractical Joke). While set, every prevention shield is ignored.
    /// Cleared at cleanup.
    #[serde(default)]
    pub(crate) damage_cant_be_prevented_this_turn: bool,
    /// Registered replacement effects (Phase H — Commander prerequisite).
    /// Walked by zone-change paths (`place_card_in_dest`,
    /// `remove_from_battlefield_to_*`) at placement time; a matching
    /// entry rewrites the destination zone.
    ///
    /// `#[serde(default)]` so snapshots written before this field
    /// existed deserialize cleanly as empty (no replacements active).
    #[serde(default)]
    pub replacement_effects: Vec<crate::replacement::ReplacementEffect>,
    /// Monotonic counter handing out `ReplacementId`s. Defaults to 0
    /// for snapshot back-compat.
    #[serde(default)]
    pub(crate) next_replacement_id: u32,
    /// Per-commander cast-from-command-zone counter (Phase L).
    /// Keyed by the commander's `CardId`; each entry tracks how many
    /// times that commander has been cast from the command zone this
    /// game. The commander tax is `{2}` × this value, added as
    /// generic mana on top of the printed cost (CR 903.8).
    ///
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub commander_cast_count: HashMap<CardId, u32>,
    /// 21-commander-damage tracker (Phase M / CR 704.5v). Keyed by
    /// `(victim_seat, commander_card_id)`; values are running totals
    /// of combat / direct damage dealt by that commander to that
    /// seat over the whole game. The SBA in
    /// `check_state_based_actions` eliminates a player when any of
    /// their entries crosses 21.
    ///
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub commander_damage: HashMap<(usize, CardId), u32>,
    /// Per-dying-card snapshot
    /// cache, populated at SBA emission time for every dying creature
    /// (token or non-token). Used by trigger-dispatch lookups
    /// (`game/effects/events.rs::event_matches_spec`,
    /// `evaluate_requirement_static` zone walk) so AnotherOfYours-
    /// scope triggers with creature-type filters (Witherbloom
    /// Pestmaster, Felisa, Fang of Silverquill) fire correctly when
    /// the dying subject is a token — CR 111.7c's "ceases to exist"
    /// SBA removes the token from every zone in the same sweep as
    /// the death event emission, so by the time
    /// `dispatch_triggers_for_events` runs the token is gone from
    /// both battlefield and graveyard. The cached `CardInstance`
    /// survives the SBA sweep, giving the dispatcher a reliable
    /// way to read both the controller AND the dying card's
    /// printed types / counters. Cleared after each dispatch pass.
    /// `#[serde(skip)]` because it's transient scratch — snapshots
    /// don't need to preserve mid-SBA state.
    #[serde(skip)]
    pub(crate) died_card_snapshots: HashMap<CardId, CardInstance>,
    /// Set of permanent CardIds that gained one or more counters during
    /// the current turn. Bumped in `Effect::AddCounter`'s resolver
    /// whenever a permanent gains counters; reset to empty in
    /// `do_cleanup`. Powers Fractal Tender's end-step "if you put a
    /// counter on this creature this turn, mint a Fractal" rider via
    /// the new `Predicate::SourceGainedCounterThisTurn` predicate.
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) permanents_gained_counter_this_turn: std::collections::HashSet<CardId>,
    /// Per-permanent transient triggered abilities granted by spells /
    /// continuous effects (Rabid Attack, Root Manipulation: "creatures
    /// you control gain '…trigger…' until end of turn"). The dispatcher
    /// walks this map alongside each permanent's printed
    /// `triggered_abilities` and fires matching events. Cleared in
    /// `do_cleanup` (the "until end of turn" expiry). Other durations
    /// (Permanent) would need a separate map; only EOT grants are
    /// modeled today since that's what the printed catalog needs.
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) granted_triggers_eot:
        std::collections::HashMap<CardId, Vec<crate::card::TriggeredAbility>>,
    /// Permanents whose death is replaced by exile for the rest of the
    /// turn — "if that creature would die this turn, exile it instead"
    /// (Wilt in the Heat). Checked in `remove_from_battlefield_to_graveyard`
    /// alongside the Finality-counter redirect; cleared at cleanup. The
    /// redirect lasts the whole turn, so it also catches deaths from later
    /// combat / removal, not just the spell's own damage. `#[serde(default)]`
    /// for snapshot back-compat.
    #[serde(default)]
    pub(crate) dies_to_exile_eot: std::collections::HashSet<CardId>,
}

/// Manual `Clone` impl so the bot can dry-run an action against a copy
/// of the state without committing it. `Box<dyn Decider>` blocks the
/// derive — we round-trip through `DeciderKind`. Custom deciders not
/// modeled by the kind enum collapse to `AutoDecider` on clone, which
/// is fine for the dry-run use case (we discard the clone immediately).
impl Clone for GameState {
    fn clone(&self) -> Self {
        Self {
            players: self.players.clone(),
            teams: self.teams.clone(),
            battlefield: self.battlefield.clone(),
            exile: self.exile.clone(),
            stack: self.stack.clone(),
            step: self.step,
            active_player_idx: self.active_player_idx,
            turn_number: self.turn_number,
            game_over: self.game_over,
            priority: self.priority.clone(),
            continuous_effects: self.continuous_effects.clone(),
            next_effect_timestamp: self.next_effect_timestamp,
            next_id: self.next_id,
            attacking: self.attacking.clone(),
            block_map: self.block_map.clone(),
            blockers_declared: self.blockers_declared,
            skip_first_draw: self.skip_first_draw,
            spells_cast_this_turn: self.spells_cast_this_turn,
            delayed_triggers: self.delayed_triggers.clone(),
            sacrificed_power: self.sacrificed_power,
            sacrificed_toughness: self.sacrificed_toughness,
            last_created_token: self.last_created_token,
            last_created_tokens: self.last_created_tokens.clone(),
            last_moved_cards: self.last_moved_cards.clone(),
            cards_discarded_this_resolution: self.cards_discarded_this_resolution,
            creature_cards_discarded_this_resolution: self.creature_cards_discarded_this_resolution,
            cards_discarded_per_player_this_resolution: self.cards_discarded_per_player_this_resolution.clone(),
            discarded_card_ids_this_resolution: self.discarded_card_ids_this_resolution.clone(),
            permanents_destroyed_this_resolution: self.permanents_destroyed_this_resolution,
            named_card_this_resolution: self.named_card_this_resolution.clone(),
            pending_cast_face: self.pending_cast_face,
            decider: self.decider.kind().into_boxed(),
            pending_decision: self.pending_decision.clone(),
            suspend_signal: self.suspend_signal.clone(),
            prevent_combat_damage_this_turn: self.prevent_combat_damage_this_turn,
            combat_damage_prevented_creatures: self.combat_damage_prevented_creatures.clone(),
            prevention_shields: self.prevention_shields.clone(),
            damage_cant_be_prevented_this_turn: self.damage_cant_be_prevented_this_turn,
            replacement_effects: self.replacement_effects.clone(),
            next_replacement_id: self.next_replacement_id,
            commander_cast_count: self.commander_cast_count.clone(),
            commander_damage: self.commander_damage.clone(),
            died_card_snapshots: self.died_card_snapshots.clone(),
            permanents_gained_counter_this_turn: self.permanents_gained_counter_this_turn.clone(),
            granted_triggers_eot: self.granted_triggers_eot.clone(),
            dies_to_exile_eot: self.dies_to_exile_eot.clone(),
        }
    }
}

impl GameState {
    /// Create a fresh game.  `players` must have at least 2 entries. Defaults
    /// to 20-life, 2-player rules; call [`apply_format`] (or set
    /// `skip_first_draw` / per-player `life` directly) to configure the game
    /// for a specific format or player count.
    pub fn new(players: Vec<Player>) -> Self {
        let n = players.len();
        // Default: one singleton team per seat (free-for-all semantics).
        // Team formats reshape this via `assign_teams`.
        let teams = (0..n)
            .map(|i| crate::team::Team {
                id: crate::team::TeamId(i),
                members: vec![i],
                shared_life: None,
            })
            .collect();
        Self {
            players,
            teams,
            battlefield: Vec::new(),
            exile: Vec::new(),
            stack: Vec::new(),
            step: TurnStep::Untap,
            active_player_idx: 0,
            turn_number: 1,
            game_over: None,
            priority: PriorityState::new(0),
            continuous_effects: Vec::new(),
            next_effect_timestamp: 1,
            next_id: 1,
            attacking: Vec::new(),
            block_map: HashMap::new(),
            blockers_declared: false,
            // Multiplayer (3+) doesn't skip the first draw — only the 2-player
            // starting player does.
            skip_first_draw: n <= 2,
            spells_cast_this_turn: 0,
            delayed_triggers: Vec::new(),
            sacrificed_power: None,
            sacrificed_toughness: None,
            last_created_token: None,
            last_created_tokens: Vec::new(),
            last_moved_cards: Vec::new(),
            cards_discarded_this_resolution: 0,
            creature_cards_discarded_this_resolution: 0,
            cards_discarded_per_player_this_resolution: HashMap::new(),
            discarded_card_ids_this_resolution: Vec::new(),
            permanents_destroyed_this_resolution: 0,
            named_card_this_resolution: None,
            pending_cast_face: CastFace::Front,
            decider: Box::new(AutoDecider),
            pending_decision: None,
            suspend_signal: None,
            prevent_combat_damage_this_turn: false,
            combat_damage_prevented_creatures: Vec::new(),
            prevention_shields: Vec::new(),
            damage_cant_be_prevented_this_turn: false,
            replacement_effects: Vec::new(),
            next_replacement_id: 1,
            commander_cast_count: HashMap::new(),
            commander_damage: HashMap::new(),
            died_card_snapshots: HashMap::new(),
            permanents_gained_counter_this_turn: std::collections::HashSet::new(),
            granted_triggers_eot: std::collections::HashMap::new(),
            dies_to_exile_eot: std::collections::HashSet::new(),
        }
    }

    /// Transient triggers granted to a permanent until EOT (Root
    /// Manipulation, Rabid Attack-style "creatures gain '…' EOT").
    /// Returns an empty slice when no grant is active — call sites can
    /// `.iter().chain(self.granted_triggers(id))` against the printed
    /// abilities without cloning.
    pub(crate) fn granted_triggers(
        &self,
        id: CardId,
    ) -> &[crate::card::TriggeredAbility] {
        self.granted_triggers_eot
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Apply format-specific setup: starting life total, turn-1 draw
    /// rule, and (for Two-Headed Giant) the team partition + shared
    /// life pool.
    pub fn apply_format(&mut self, format: crate::format::Format) {
        let rules = format.rules();
        let life = if self.players.len() > 2 {
            rules.multiplayer_starting_life.unwrap_or(rules.starting_life)
        } else {
            rules.starting_life
        };
        for p in &mut self.players {
            p.life = life;
        }
        self.skip_first_draw = self.players.len() <= 2;

        // Two-Headed Giant — Phase F. Default seating partitions
        // consecutive seat pairs into teams (0+1, 2+3, …) per
        // CR 810.2a and seeds each team's shared pool to the format's
        // starting life. Callers wanting a different pairing can
        // call `assign_teams` afterwards; the shared-life seeding
        // happens here regardless. An odd seat count leaves the
        // trailing odd seat as a singleton (silly setup, but keeps
        // the helper total — the caller likely wants `assign_teams`).
        if matches!(format, crate::format::Format::TwoHeadedGiant) {
            let n = self.players.len();
            let mut partitions: Vec<Vec<usize>> = Vec::new();
            let mut i = 0;
            while i < n {
                if i + 1 < n {
                    partitions.push(vec![i, i + 1]);
                    i += 2;
                } else {
                    partitions.push(vec![i]);
                    i += 1;
                }
            }
            self.teams = partitions
                .into_iter()
                .enumerate()
                .map(|(idx, members)| crate::team::Team {
                    id: crate::team::TeamId(idx),
                    members,
                    shared_life: Some(life),
                })
                .collect();
        }
    }

    /// Number of players that have not been eliminated.
    pub fn alive_count(&self) -> usize {
        self.players.iter().filter(|p| p.is_alive()).count()
    }

    /// Next non-eliminated seat strictly after `from` (wrapping). Returns
    /// `from` if no other alive players remain.
    pub fn next_alive_seat(&self, from: usize) -> usize {
        let n = self.players.len();
        for step in 1..=n {
            let i = (from + step) % n;
            if self.players[i].is_alive() {
                return i;
            }
        }
        from
    }

    /// Sort `seats` into APNAP order — active player first, then each other
    /// seat in turn order (CR 101.4). Used when a single effect affects
    /// "each player" so simultaneous-ish fan-outs (draws, mills, sacrifices)
    /// resolve in the canonical order rather than raw seat index.
    pub(crate) fn apnap_sort(&self, mut seats: Vec<usize>) -> Vec<usize> {
        let n = self.players.len().max(1);
        let active = self.active_player_idx;
        let rank = |seat: usize| -> usize {
            if seat == active {
                return 0;
            }
            let mut s = active;
            for r in 1..=n {
                s = self.next_alive_seat(s);
                if s == seat {
                    return r;
                }
                if s == active {
                    break;
                }
            }
            n + seat // eliminated / unreachable: stable tail
        };
        seats.sort_by_key(|&s| rank(s));
        seats
    }

    // ── Team partitioning ─────────────────────────────────────────────────────

    /// Team that contains `seat`. Falls back to a virtual singleton
    /// `TeamId(seat)` when `teams` is empty (e.g. snapshots from before
    /// the field was added).
    pub fn team_of(&self, seat: usize) -> crate::team::TeamId {
        for t in &self.teams {
            if t.members.contains(&seat) {
                return t.id;
            }
        }
        crate::team::TeamId(seat)
    }

    /// Seats sharing a team with `seat`, excluding `seat` itself. Empty
    /// for singleton-team seats.
    pub fn teammates(&self, seat: usize) -> Vec<usize> {
        let my_team = self.team_of(seat);
        for t in &self.teams {
            if t.id == my_team {
                return t.members.iter().copied().filter(|&s| s != seat).collect();
            }
        }
        Vec::new()
    }

    /// Seats on every team other than `seat`'s. Includes eliminated
    /// players; callers that need a live-only list should filter on
    /// `players[s].is_alive()` themselves.
    pub fn opponents_of(&self, seat: usize) -> Vec<usize> {
        if self.teams.is_empty() {
            // No teams declared — treat every other seat as an opponent.
            return (0..self.players.len()).filter(|&s| s != seat).collect();
        }
        let my_team = self.team_of(seat);
        let mut out = Vec::new();
        for t in &self.teams {
            if t.id != my_team {
                out.extend(t.members.iter().copied());
            }
        }
        out
    }

    /// True when `a` and `b` are on the same team. A seat is always its
    /// own teammate (returns true for `a == b`).
    pub fn same_team(&self, a: usize, b: usize) -> bool {
        self.team_of(a) == self.team_of(b)
    }

    // ── Life total helpers (Phase F) ──────────────────────────────────────

    /// Effective life total visible to `seat`. In 2HG (`Team.shared_life
    /// == Some(n)`) every member of the team sees the same number; in
    /// solo-team formats (1v1 / FFA / Commander) this is just the
    /// player's own `life` field. Callers checking lethal damage,
    /// "if you have ≤ X life" predicates, "the most life total" etc.
    /// should consult this rather than `players[seat].life`.
    pub fn effective_life(&self, seat: usize) -> i32 {
        if let Some(t) = self.teams.iter().find(|t| t.members.contains(&seat))
            && let Some(shared) = t.shared_life
        {
            return shared;
        }
        self.players[seat].life
    }

    /// Apply a life delta to `seat` — gain for `delta > 0`, loss for
    /// `delta < 0`. Routes through the team's shared pool when set
    /// (Phase F — 2HG), else mutates `players[seat].life` directly.
    /// Returns the post-mutation effective life total.
    ///
    /// Per-turn counters (`life_gained_this_turn`) are bumped on the
    /// *seat* receiving the change — they're a "you" payoff and the
    /// triggering side is still a specific player. For 2HG, CR 810.8
    /// also propagates the gain to teammates' "you gain life"
    /// triggers; that broader fan-out is handled at trigger-scope
    /// resolution time (`EventScope::YourControl` would need a
    /// team-aware variant), not here. This helper only owns the
    /// state-mutation half.
    pub fn adjust_life(&mut self, seat: usize, delta: i32) -> i32 {
        if delta == 0 {
            return self.effective_life(seat);
        }
        // CR 119.7: if `seat` can't gain life and the delta would
        // increase their life total, drop the gain on the floor. The
        // 119.10 rider — "If a player gains 0 life, no life gain event
        // would occur, and these effects won't apply" — is honored
        // implicitly: the gain never happens, no LifeGained event is
        // emitted, the `life_gained_this_turn` counter isn't bumped.
        //
        // The check ORs the directly-settable `Player.cannot_gain_life`
        // flag (set by emblems / once-per-game effects) with the
        // dynamic battlefield scan via `player_cannot_gain_life_now`
        // (consults `StaticEffect::PlayerCannotGainLife` statics on
        // the live battlefield).
        if delta > 0 && self.player_cannot_gain_life_now(seat) {
            return self.effective_life(seat);
        }
        // CR 119.8: symmetric drop for negative deltas (lose-life).
        if delta < 0 && self.player_cannot_lose_life_now(seat) {
            return self.effective_life(seat);
        }
        let team_idx = self
            .teams
            .iter()
            .position(|t| t.members.contains(&seat));
        let writes_to_shared = team_idx
            .and_then(|i| self.teams[i].shared_life)
            .is_some();

        let new_total = if writes_to_shared {
            let t = team_idx.unwrap();
            let current = self.teams[t].shared_life.unwrap();
            let next = current.saturating_add(delta);
            self.teams[t].shared_life = Some(next);
            next
        } else {
            let p = &mut self.players[seat];
            p.life = p.life.saturating_add(delta);
            p.life
        };

        if delta > 0 {
            self.players[seat].life_gained_this_turn =
                self.players[seat].life_gained_this_turn.saturating_add(delta as u32);
        }
        new_total
    }

    /// Overwrite the effective life total for `seat` (Effect::SetLife
    /// path). Routes through the shared pool when set, else writes
    /// `players[seat].life` directly. Does not bump
    /// `life_gained_this_turn` (set-to-N isn't a "gain").
    pub fn set_life(&mut self, seat: usize, new_total: i32) {
        if let Some(t) = self.teams.iter_mut().find(|t| t.members.contains(&seat))
            && t.shared_life.is_some()
        {
            t.shared_life = Some(new_total);
            return;
        }
        self.players[seat].life = new_total;
    }

    // ── Commander identity & damage (Phase J / M) ──────────────────────────

    /// True if `card_id` is a commander for any player. Used by the
    /// Phase M 21-damage accumulator and by Phase L's cast-from-CZ
    /// (a non-commander has no business hitting that path).
    pub fn is_commander(&self, card_id: crate::card::CardId) -> bool {
        self.players
            .iter()
            .any(|p| p.commanders.contains(&card_id))
    }

    /// Add `amount` to the commander-damage tally for
    /// `(victim_seat, source_card_id)`. Caller is responsible for
    /// checking `is_commander(source)` before invoking — invalid
    /// entries would otherwise pollute the SBA's view. Phase M's
    /// damage paths gate on this check.
    ///
    /// The SBA (`check_state_based_actions`) consults the table
    /// after every life mutation, so no immediate action is required
    /// here beyond bumping the counter.
    pub fn record_commander_damage(
        &mut self,
        victim_seat: usize,
        source_card_id: crate::card::CardId,
        amount: u32,
    ) {
        if amount == 0 {
            return;
        }
        let entry = self
            .commander_damage
            .entry((victim_seat, source_card_id))
            .or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    // ── Commander seating (Phase J) ────────────────────────────────────────

    /// Place each card in `defs` into `seat`'s command zone as a new
    /// `CardInstance`, and register the Commander zone-change
    /// replacement effect for each — CR 903.9b's "if a commander
    /// would be put into a graveyard, exile, hand, or library from
    /// anywhere, its owner may put it into the command zone
    /// instead." Phase L's cast-from-CZ machinery + commander-cast
    /// counter consult the command zone contents; this helper sets
    /// up that initial state.
    ///
    /// Returns the `CardId`s of the seated commanders so callers
    /// can use them as `Selector::CardInZone(Command)` targets, or
    /// pass them to test helpers.
    ///
    /// The replacement is registered with `optional: true` — CR 903.9b
    /// says the redirect is "may", so the owner can elect to let the
    /// commander land in the original zone (e.g. when they want to
    /// reanimate it from the graveyard rather than re-pay tax).
    /// `AutoDecider` defaults to "yes redirect" so tournament-style
    /// play matches expectations; tests can script the opposite via
    /// `ScriptedDecider` answering `DecisionAnswer::Bool(false)` to
    /// the `Decision::CommanderRedirect` prompt.
    pub fn seat_commanders(
        &mut self,
        seat: usize,
        defs: Vec<crate::card::CardDefinition>,
    ) -> Vec<crate::card::CardId> {
        let mut ids = Vec::with_capacity(defs.len());
        for def in defs {
            let id = crate::card::CardId(self.next_id);
            self.next_id = self.next_id.saturating_add(1);
            let card = crate::card::CardInstance::new(id, def, seat);
            self.players[seat].command.push(card);
            self.players[seat].commanders.push(id);

            // CR 903.9b replacement — graveyard / exile / hand /
            // library from anywhere → command zone. `from: None`
            // matches any origin; the destination set is the four
            // zones the rule names.
            self.register_replacement(crate::replacement::ReplacementEffect {
                id: crate::replacement::ReplacementId(0), // overwritten
                source: crate::replacement::ReplacementSource::Card(id),
                from: None,
                to_zones: vec![
                    crate::card::Zone::Graveyard,
                    crate::card::Zone::Exile,
                    crate::card::Zone::Hand,
                    crate::card::Zone::Library,
                ],
                redirect_to: crate::card::Zone::Command,
                optional: true,
            });
            ids.push(id);
        }
        ids
    }

    // ── Replacement effects (Phase H) ─────────────────────────────────────

    /// Register `effect` with the engine. Returns the assigned id so the
    /// caller can `unregister_replacement` it later (e.g. when the
    /// originating permanent leaves play). The caller-supplied `id`
    /// field is ignored — the engine stamps a fresh monotonic id.
    pub fn register_replacement(
        &mut self,
        mut effect: crate::replacement::ReplacementEffect,
    ) -> crate::replacement::ReplacementId {
        let id = crate::replacement::ReplacementId(self.next_replacement_id);
        self.next_replacement_id = self.next_replacement_id.saturating_add(1);
        effect.id = id;
        self.replacement_effects.push(effect);
        id
    }

    /// Drop the replacement with `id` if present. Returns true on hit.
    pub fn unregister_replacement(&mut self, id: crate::replacement::ReplacementId) -> bool {
        if let Some(pos) = self
            .replacement_effects
            .iter()
            .position(|r| r.id == id)
        {
            self.replacement_effects.remove(pos);
            true
        } else {
            false
        }
    }

    /// Walk the replacement registry for a zone change. Returns the
    /// destination zone after applying any matching replacement. Loops
    /// up to [`crate::replacement::MAX_REPLACEMENT_ITERATIONS`] times
    /// so chained replacements (e.g. graveyard → exile → command) can
    /// fully resolve while pathological loops still terminate. When
    /// the cap is hit, the most-recent destination is returned and a
    /// debug-assert fires.
    ///
    /// For `optional: true` replacements the card's owner is consulted
    /// via the installed `Decider` (`Decision::CommanderRedirect`).
    /// `AutoDecider` answers "yes" (matching the typical "save my
    /// commander" play), tests can script the opposite via
    /// `ScriptedDecider`. A declined optional replacement still
    /// counts as "applied" for CR 614.5 purposes so the same prompt
    /// isn't surfaced twice in one resolution walk.
    ///
    /// `&mut self` because the decider call is mutable. CR 616
    /// ordering ("affected card's controller chooses") is
    /// approximated by registration order.
    pub fn resolve_zone_change(
        &mut self,
        card_id: crate::card::CardId,
        from: crate::card::Zone,
        mut to: crate::card::Zone,
    ) -> crate::card::Zone {
        use crate::replacement::{ReplacementSource, MAX_REPLACEMENT_ITERATIONS};
        // Note: CR 122.1h finality counter redirect is applied at the
        // call site (`remove_from_battlefield_to_graveyard`) because by
        // the time we reach this resolver the card has already been
        // removed from the battlefield. The call site passes
        // `Zone::Exile` instead of `Zone::Graveyard` when finality is
        // present.
        let mut applied: Vec<crate::replacement::ReplacementId> = Vec::new();
        for _ in 0..MAX_REPLACEMENT_ITERATIONS {
            let mut fired = false;
            // Clone the small set of metadata we need so we can mutate
            // `self.decider` inside the loop without borrow-conflict
            // with `self.replacement_effects`.
            let candidates: Vec<_> = self
                .replacement_effects
                .iter()
                .map(|r| {
                    (
                        r.id,
                        r.source.clone(),
                        r.from,
                        r.to_zones.clone(),
                        r.redirect_to,
                        r.optional,
                    )
                })
                .collect();
            for (rid, source, r_from, to_zones, redirect_to, optional) in candidates {
                if applied.contains(&rid) {
                    // CR 614.5 — a replacement effect can apply at most
                    // once to a given event. Skip ones we've already
                    // used in this resolution.
                    continue;
                }
                match source {
                    ReplacementSource::Card(target) if target != card_id => continue,
                    ReplacementSource::Card(_) => {}
                }
                if let Some(f) = r_from
                    && f != from
                {
                    continue;
                }
                if !to_zones.contains(&to) {
                    continue;
                }
                // Optional replacement → consult the decider. Today
                // the only optional replacement we register is the
                // Commander redirect (CR 903.9b), so the
                // `CommanderRedirect` decision shape is the right
                // surface. If `optional` were used for some other
                // redirect later, this branch would need a generic
                // `OptionalReplacement` decision instead.
                if optional {
                    let answer = self.decider.decide(&crate::decision::Decision::CommanderRedirect {
                        commander: card_id,
                        would_be: to,
                    });
                    let say_yes = matches!(answer, crate::decision::DecisionAnswer::Bool(true));
                    applied.push(rid);
                    if !say_yes {
                        // Don't apply, but mark as asked so we don't
                        // re-prompt on this resolution.
                        continue;
                    }
                } else {
                    applied.push(rid);
                }
                to = redirect_to;
                fired = true;
                break;
            }
            if !fired {
                return to;
            }
        }
        debug_assert!(false, "replacement-effect resolution hit iteration cap");
        to
    }

    /// Number of `StaticEffect::DoubleTokens` permanents `seat` controls
    /// on the battlefield. Used by `Effect::CreateToken` to scale the
    /// token count by `2^n` — one Adrix and Nev, Twincasters in play
    /// means twice as many tokens are minted; two doublers means four
    /// times as many; etc. (CR 614.13: multiple replacement effects
    /// apply in any order chosen by the controller, but all functionally
    /// multiply rather than just add.)
    pub fn token_doublers_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::DoubleTokens))
                    .count() as u32
            })
            .sum()
    }

    /// Number of `StaticEffect::DoubleCounters` permanents `seat` controls
    /// on the battlefield. Used by `Effect::AddCounter` to scale the counter
    /// count by `2^n` per CR 614.16's "if one or more counters would be put
    /// on a permanent" replacement. One Doubling Season → 2×; one Doubling
    /// Season + one Hardened Scales → 4× (multiplicative, matching the
    /// printed Oracle).
    pub fn counter_doublers_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::DoubleCounters))
                    .count() as u32
            })
            .sum()
    }

    /// CR 614.x — true if any active `StaticEffect::ExileNontokenCreaturesNotCast`
    /// (Containment Priest) is on the battlefield. Consulted by
    /// `place_card_in_dest` to reroute non-cast nontoken creatures to exile.
    pub fn nontoken_creature_etb_exile_active(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::ExileNontokenCreaturesNotCast))
        })
    }

    /// CR 614.2 — number of `StaticEffect::DoubleDamageDealt` permanents on
    /// the battlefield (controller-agnostic: Furnace of Rath doubles *all*
    /// damage). Damage is scaled by `2^n`; `n` doublers → `2^n×`.
    pub fn damage_doublers(&self) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::DoubleDamageDealt))
                    .count() as u32
            })
            .sum()
    }

    /// True if `seat` cannot gain life *right now*, per CR 119.7. ORs:
    /// 1. The directly-settable `Player.cannot_gain_life` flag (set by
    ///    emblems / once-per-game state — currently dormant; reserved for
    ///    permanent grants).
    /// 2. Any active `StaticEffect::PlayerCannotGainLife` on the
    ///    battlefield whose `target` resolves to include `seat`.
    ///
    /// Consulted by `GameState::adjust_life` to drop positive deltas
    /// targeting `seat` on the floor.
    pub fn player_cannot_gain_life_now(&self, seat: usize) -> bool {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        if self.players[seat].cannot_gain_life || self.players[seat].cannot_gain_life_this_turn {
            return true;
        }
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| {
                if let StaticEffect::PlayerCannotGainLife { target } = &sa.effect {
                    match target {
                        PlayerStaticTarget::Controller => src.controller == seat,
                        PlayerStaticTarget::EachOpponent => src.controller != seat,
                        PlayerStaticTarget::EachPlayer => true,
                    }
                } else {
                    false
                }
            })
        })
    }

    /// CR 119.8 — True if `seat` cannot lose life right now. Mirror of
    /// `player_cannot_gain_life_now`. Scans the battlefield for any
    /// `StaticEffect::PlayerCannotLoseLife` whose `target` resolves to
    /// include `seat`. Consulted by `adjust_life` (negative deltas) and
    /// by the lose-life paths (`Effect::LoseLife`, drain-target gates).
    pub fn player_cannot_lose_life_now(&self, seat: usize) -> bool {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| {
                if let StaticEffect::PlayerCannotLoseLife { target } = &sa.effect {
                    match target {
                        PlayerStaticTarget::Controller => src.controller == seat,
                        PlayerStaticTarget::EachOpponent => src.controller != seat,
                        PlayerStaticTarget::EachPlayer => true,
                    }
                } else {
                    false
                }
            })
        })
    }

    /// CR 121.2b — the smallest per-turn draw cap currently imposed on
    /// `seat` by any active `StaticEffect::CapDrawsPerTurn`, or `None` if
    /// the seat may draw freely. Multiple caps take the strictest (min).
    pub fn draw_cap_for(&self, seat: usize) -> Option<u32> {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield
            .iter()
            .flat_map(|src| {
                src.definition.static_abilities.iter().filter_map(move |sa| {
                    if let StaticEffect::CapDrawsPerTurn { target, max } = &sa.effect {
                        let hits = match target {
                            PlayerStaticTarget::Controller => src.controller == seat,
                            PlayerStaticTarget::EachOpponent => src.controller != seat,
                            PlayerStaticTarget::EachPlayer => true,
                        };
                        hits.then_some(*max)
                    } else {
                        None
                    }
                })
            })
            .min()
    }

    /// Replace the current team partition. Every seat must appear in
    /// exactly one entry; partitions must be non-empty. Used by team
    /// formats (2HG) after `new()` to group seats.
    pub fn assign_teams(
        &mut self,
        partitions: Vec<Vec<usize>>,
    ) -> Result<(), crate::team::TeamError> {
        let n = self.players.len();
        let mut seen = vec![false; n];
        for (i, part) in partitions.iter().enumerate() {
            if part.is_empty() {
                return Err(crate::team::TeamError::EmptyTeam(i));
            }
            for &seat in part {
                if seat >= n {
                    return Err(crate::team::TeamError::UnknownSeat {
                        seat,
                        num_players: n,
                    });
                }
                if seen[seat] {
                    return Err(crate::team::TeamError::DuplicateSeat(seat));
                }
                seen[seat] = true;
            }
        }
        for (seat, was_seen) in seen.iter().enumerate() {
            if !was_seen {
                return Err(crate::team::TeamError::MissingSeat(seat));
            }
        }
        self.teams = partitions
            .into_iter()
            .enumerate()
            .map(|(i, members)| crate::team::Team {
                id: crate::team::TeamId(i),
                members,
                shared_life: None,
            })
            .collect();
        Ok(())
    }

    /// The player who currently holds priority.
    pub fn player_with_priority(&self) -> usize {
        self.priority.player_with_priority
    }

    /// Give priority to the active player and reset consecutive passes.
    pub(crate) fn give_priority_to_active(&mut self) {
        self.priority.player_with_priority = self.active_player_idx;
        self.priority.consecutive_passes = 0;
    }

    // ── Layer system ──────────────────────────────────────────────────────────

    /// Compute the current derived state of all battlefield permanents after
    /// applying all active continuous effects in layer order.
    pub fn compute_battlefield(&self) -> Vec<ComputedPermanent> {
        crate::game::layers::apply_layers(&self.battlefield, &self.gather_continuous_effects())
    }

    /// Collect every continuous effect currently active in the game: the
    /// resolved-spell/ability effects in `continuous_effects`, plus the
    /// implicit effects derived each recompute from static abilities,
    /// equipment attachments, combat keyword grants, characteristic-
    /// defining P/T, life-gain pump/anthem tables, and graveyard-resident
    /// anthems. Shared by [`compute_battlefield`] (applies to every
    /// permanent) and [`computed_permanent`] (applies to just one).
    fn gather_continuous_effects(&self) -> Vec<ContinuousEffect> {
        // Include static-ability effects from permanents currently on the battlefield.
        let mut all_effects: Vec<ContinuousEffect> = self.continuous_effects.clone();
        for card in &self.battlefield {
            let ts = card.id.0 as u64; // stable ordering by card id for static abilities
            let mut effects = static_ability_to_effects(card, ts);
            // Team-aware static abilities: `static_ability_to_effects` is a
            // free function with no GameState handle, so it can't fill in
            // `AllOpponents.friendly_seats` itself. Patch them now using
            // the source's actual team membership — in 1v1 / FFA this is
            // `[source_controller]` and behaves identically to the legacy
            // single-seat check; in team formats (2HG) it lists every
            // teammate so a Crackling Drake-style "creatures opponents
            // control" anthem doesn't accidentally buff the source's
            // partner.
            for e in &mut effects {
                if let AffectedPermanents::AllOpponents {
                    source_controller,
                    friendly_seats,
                    ..
                } = &mut e.affected
                    && friendly_seats.is_empty()
                {
                    let mut seats = self.teammates(*source_controller);
                    seats.push(*source_controller);
                    *friendly_seats = seats;
                }
            }
            all_effects.extend(effects);
        }
        // CR 702.6 — Equipment attachment statics. Each Equipment with a
        // live `attached_to` link and an `equipped_bonus` confers +P/+T
        // (layer 7c) and keyword grants (layer 6) on the creature it's
        // attached to, for as long as the Equipment stays on the battlefield.
        // The stale-link SBA in `stack.rs` clears `attached_to` when the
        // equipped creature leaves, so a dangling link can't leak a bonus.
        for card in &self.battlefield {
            let Some(bonus) = &card.definition.equipped_bonus else { continue };
            let Some(target) = card.attached_to else { continue };
            // Only apply while the target is still a creature on the bf.
            if !self.battlefield.iter().any(|c| c.id == target) {
                continue;
            }
            if bonus.power != 0 || bonus.toughness != 0 {
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![target]),
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(
                        bonus.power,
                        bonus.toughness,
                    ),
                });
            }
            for kw in &bonus.keywords {
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![target]),
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(kw.clone()),
                });
            }
        }
        // "Attacking creatures you control have <keyword>" (Blade Historian).
        // Resolved here because `affects()` can't see combat state — we read
        // the live `attacking` list and scope the grant to the source's own
        // attackers. Layer-6 keyword addition, like the equipment grants.
        if !self.attacking.is_empty() {
            for card in &self.battlefield {
                for sa in &card.definition.static_abilities {
                    let crate::effect::StaticEffect::GrantKeywordToAttackers { keyword } =
                        &sa.effect
                    else {
                        continue;
                    };
                    let ids: Vec<CardId> = self
                        .attacking
                        .iter()
                        .map(|a| a.attacker)
                        .filter(|id| {
                            self.battlefield
                                .iter()
                                .any(|c| c.id == *id && c.controller == card.controller)
                        })
                        .collect();
                    if ids.is_empty() {
                        continue;
                    }
                    all_effects.push(ContinuousEffect {
                        timestamp: card.id.0 as u64,
                        source: card.id,
                        affected: AffectedPermanents::Specific(ids),
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(keyword.clone()),
                    });
                }
            }
        }
        // CR 700.5 / Theros gods — "isn't a creature unless your devotion
        // to [colors] ≥ threshold." Emit a layer-4 RemoveCardType(Creature)
        // self-effect while the gate is unmet; reading devotion needs the
        // live GameState, so it can't route through static_ability_to_effects.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::NotCreatureWhileDevotionBelow {
                    colors,
                    threshold,
                } = &sa.effect
                else {
                    continue;
                };
                if (self.devotion_to(card.controller, colors) as u32) < *threshold {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.id.0 as u64,
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::RemoveCardType(CardType::Creature),
                    });
                }
            }
        }
        // "This creature gets +X/+Y for each [filter] you control."
        // (`StaticEffect::PumpSelfByControlledPermanents`) — count the
        // controller's matching battlefield permanents live and emit a
        // layer-7 ModifyPowerToughness self-effect.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpSelfByControlledPermanents {
                    filter,
                    per_power,
                    per_toughness,
                } = &sa.effect
                else {
                    continue;
                };
                let count = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == card.controller
                            && self.evaluate_requirement_on_card(filter, c, card.controller)
                    })
                    .count() as i32;
                if count == 0 {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(
                        count * per_power,
                        count * per_toughness,
                    ),
                });
            }
        }
        // CR 604.x — characteristic-defining dynamic P/T injection. The
        // per-card formula lookup lives in `dynamic_pt_for_name`; we
        // resolve it here on every layer recompute and emit a layer-7
        // SetPT effect. Adding a new dynamic-P/T card is one row in that
        // table — no engine-side `if name == "..."` branch.
        let goyf_n = self.distinct_card_types_in_all_graveyards() as i32;
        let lands_in_gys: i32 = self.players.iter()
            .map(|p| p.graveyard.iter().filter(|c| c.definition.is_land()).count() as i32)
            .sum();
        for card in &self.battlefield {
            let Some(formula) = dynamic_pt_for_name(card.definition.name) else { continue };
            let (power, toughness) = match formula {
                crate::card::DynamicPt::DistinctTypesInAllGraveyards => {
                    (goyf_n, goyf_n + 1)
                }
                crate::card::DynamicPt::ControllerGraveyardSize => {
                    let n = self.players[card.controller].graveyard.len() as i32;
                    (n, n)
                }
                crate::card::DynamicPt::BasePlusLandsInAllGraveyards { base_p, base_t } => {
                    (base_p + lands_in_gys, base_t + lands_in_gys)
                }
                crate::card::DynamicPt::BasePlusLandsInControllerGraveyard { base_p, base_t } => {
                    let n = self.players[card.controller].graveyard.iter()
                        .filter(|c| c.definition.is_land()).count() as i32;
                    (base_p + n, base_t + n)
                }
            };
            all_effects.push(ContinuousEffect {
                timestamp: card.id.0 as u64,
                source: card.id,
                affected: AffectedPermanents::Source,
                layer: Layer::L7PowerTough,
                sublayer: Some(PtSublayer::CharDefining),
                duration: EffectDuration::WhileSourceOnBattlefield,
                modification: Modification::SetPowerToughness(power, toughness),
            });
        }
        for card in &self.battlefield {
            let name = card.definition.name;
            // "As long as you've gained life this turn, +P/+T [and KW]"
            // self-pump consolidation: name lookup table at
            // `lifegain_selfpump_for_name`. Adds one helper-table row per
            // card instead of a new `if name == "..."` branch. Gate
            // evaluation happens every layer recompute, so mid-turn life
            // gain flips the pump on for the remainder of that turn and
            // the body snaps back at the next untap step.
            if let Some((p, t, kws)) = lifegain_selfpump_for_name(name)
                && self.players[card.controller].life_gained_this_turn > 0
            {
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(p, t),
                });
                for kw in kws {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.id.0 as u64,
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw.clone()),
                    });
                }
            }
            // "As long as there are N+ cards in your graveyard, this gets
            // +P/+T" self-pump (Elvish Reclaimer's Threshold rider). Lookup
            // table at `graveyard_threshold_selfpump_for_name`; gate is the
            // controller's graveyard size, re-evaluated every recompute.
            if let Some((threshold, p, t)) = graveyard_threshold_selfpump_for_name(name)
                && self.players[card.controller].graveyard.len() >= threshold
            {
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(p, t),
                });
            }
            // "Infusion — Creatures you control get +P/+T [and gain
            // keyword] as long as you gained life this turn." anthem
            // table: lookup at `lifegain_anthem_for_name`. Applies to
            // every creature the controller has on the battlefield
            // (including the source — printed "creatures you control"
            // is inclusive). Same recompute gate as the selfpump table.
            if let Some((p, t, kws)) = lifegain_anthem_for_name(name)
                && self.players[card.controller].life_gained_this_turn > 0
            {
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::All {
                        controller: Some(card.controller),
                        card_types: vec![CardType::Creature],
                        exclude_source: false,
                    },
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(p, t),
                });
                for kw in kws {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.id.0 as u64,
                        source: card.id,
                        affected: AffectedPermanents::All {
                            controller: Some(card.controller),
                            card_types: vec![CardType::Creature],
                            exclude_source: false,
                        },
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw.clone()),
                    });
                }
            }
            // "As long as this permanent has ≥ K [counter] counters on
            // it, [your] creatures get +P/+T" anthem consolidation. The
            // gate evaluates the source's own counter pool every layer
            // recompute, so a freshly added/removed counter flips the
            // anthem on/off immediately. Lookup table at
            // `self_counter_anthem_for_name`; adds one row per card
            // instead of new `if name == "..."` branches.
            if let Some((threshold, counter, p, t)) =
                self_counter_anthem_for_name(name)
            {
                let actual = card.counters.get(&counter).copied().unwrap_or(0);
                if actual >= threshold {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.id.0 as u64,
                        source: card.id,
                        affected: AffectedPermanents::All {
                            controller: Some(card.controller),
                            card_types: vec![CardType::Creature],
                            exclude_source: false,
                        },
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(p, t),
                    });
                }
            }
        }
        // Graveyard-resident static-ability injection — covers Anger / Wonder /
        // Filth-style Incarnations from STA whose printed Oracle reads "As
        // long as [this card] is in your graveyard and you control a [Land
        // subtype], creatures you control have [keyword]." Walks each
        // player's graveyard for entries in the `graveyard_anthem_for_name`
        // helper table; each match emits a continuous `AddKeyword` effect
        // affecting `AffectedPermanents::All` keyed on the gy-resident
        // card's owner — gated on the owner controlling at least one land
        // of the required subtype on the battlefield. The effect's
        // `source` is the gy card's id, so removing the gy card (zone
        // shuffles, exile, etc.) causes the effect to fall out via
        // `remove_effects_from_source` calls elsewhere.
        for player in &self.players {
            for card in &player.graveyard {
                if let Some((land_subtype, kw)) =
                    graveyard_anthem_for_name(card.definition.name)
                {
                    let controller_has_land = self.battlefield.iter().any(|c| {
                        c.controller == card.owner
                            && c.definition.subtypes.land_types.iter().any(|lt| lt == &land_subtype)
                    });
                    if controller_has_land {
                        all_effects.push(ContinuousEffect {
                            timestamp: card.id.0 as u64,
                            source: card.id,
                            affected: AffectedPermanents::All {
                                controller: Some(card.owner),
                                card_types: vec![CardType::Creature],
                                exclude_source: false,
                            },
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::AddKeyword(kw.clone()),
                        });
                    }
                }
            }
        }
        all_effects
    }

    /// Count of distinct card types (Artifact, Creature, Enchantment,
    /// Instant, Land, Planeswalker, Sorcery, Battle, Tribal) across every
    /// player's graveyard. Used by Tarmogoyf-style dynamic P/T.
    pub fn distinct_card_types_in_all_graveyards(&self) -> usize {
        let mut seen: std::collections::HashSet<CardType> = std::collections::HashSet::new();
        for player in &self.players {
            for card in &player.graveyard {
                for ct in &card.definition.card_types {
                    seen.insert(ct.clone());
                }
            }
        }
        seen.len()
    }

    /// Get the computed state of a single permanent (or None if not on battlefield).
    ///
    /// Gathers the same continuous-effect set as `compute_battlefield` but
    /// applies the layer pass to only the one target card, instead of
    /// building a `ComputedPermanent` for every permanent and discarding
    /// all but one.
    pub fn computed_permanent(&self, id: CardId) -> Option<ComputedPermanent> {
        let card = self.battlefield.iter().find(|c| c.id == id)?;
        Some(crate::game::layers::apply_layers_one(
            card,
            &self.gather_continuous_effects(),
        ))
    }

    /// Add a transient continuous effect (from a spell/ability resolution).
    pub fn add_continuous_effect(&mut self, effect: ContinuousEffect) {
        self.continuous_effects.push(effect);
    }

    /// Allocate a new monotonically-increasing timestamp.
    pub(crate) fn next_timestamp(&mut self) -> u64 {
        let ts = self.next_effect_timestamp;
        self.next_effect_timestamp += 1;
        ts
    }

    /// Remove all continuous effects whose source is `id` (source left battlefield).
    pub(crate) fn remove_effects_from_source(&mut self, id: CardId) {
        self.continuous_effects.retain(|e| e.source != id);
    }

    /// Expire all `UntilEndOfTurn` continuous effects (called during Cleanup).
    /// Also sweeps `UntilEndOfCombat` for cards that registered combat-
    /// scoped effects during a turn that ended without an actual combat
    /// phase (defensive cleanup so they don't leak indefinitely).
    pub(crate) fn expire_end_of_turn_effects(&mut self) {
        self.continuous_effects.retain(|e| {
            e.duration != EffectDuration::UntilEndOfTurn
                && e.duration != EffectDuration::UntilEndOfCombat
        });
    }

    /// Expire all `UntilEndOfCombat` continuous effects (CR 511.2 —
    /// "Effects that last 'until end of combat' expire at the end of the
    /// combat phase"). Invoked from `do_combat_end` once the end-of-
    /// combat step finishes.
    pub(crate) fn expire_end_of_combat_effects(&mut self) {
        self.continuous_effects
            .retain(|e| e.duration != EffectDuration::UntilEndOfCombat);
    }

    /// True if the stack is empty and it is `player`'s main phase — sorcery timing.
    pub fn can_cast_sorcery_speed(&self, player: usize) -> bool {
        self.stack.is_empty()
            && self.step.is_main_phase()
            && self.active_player_idx == player
            && self.priority.player_with_priority == player
    }

    pub(crate) fn next_id(&mut self) -> CardId {
        let id = CardId(self.next_id);
        self.next_id += 1;
        id
    }

    // ── Public setup helpers (useful in tests) ────────────────────────────────

    /// Add a card to a player's hand without going through library/draw mechanics.
    pub fn add_card_to_hand(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx]
            .hand
            .push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Put a card directly onto the battlefield (enters with summoning sickness unless cleared).
    pub fn add_card_to_battlefield(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.battlefield
            .push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Drop a token onto the battlefield directly (test fixture). Mirrors
    /// `add_card_to_battlefield` but uses `CardInstance::new_token` so the
    /// `is_token` flag is set — required for SBA path 704.5d (tokens not on
    /// the battlefield cease to exist) and for filters that consult
    /// `c.is_token`. Used by tribal-anthem and aristocrats tests that need
    /// a token board state without round-tripping through a spell cast.
    pub fn add_token_to_battlefield(
        &mut self,
        player_idx: usize,
        token: &crate::card::TokenDefinition,
    ) -> CardId {
        let id = self.next_id();
        let def = crate::game::effects::token_to_card_definition(token);
        self.battlefield
            .push(CardInstance::new_token(id, def, player_idx));
        id
    }

    /// Add a card to the **bottom** of `player_idx`'s library — appends to
    /// the end of the `library` vec. Note: with an empty library the
    /// first call pushes to index 0 (the top of the deck), so test
    /// fixtures that call this once per card end up with the
    /// **first-pushed** card on top and successive pushes building down.
    /// For top-of-deck inserts use `Player::add_to_library_top` directly.
    pub fn add_card_to_library(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx].add_to_library_bottom(id, def);
        id
    }

    /// Put a card into `player_idx`'s Lessons sideboard ("outside the
    /// game"). A Learn ability may later reveal it into hand. Used by deck
    /// construction and test fixtures exercising the Learn mechanic.
    pub fn add_card_to_sideboard(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx]
            .sideboard
            .push(crate::card::CardInstance::new(id, def, player_idx));
        id
    }

    /// Put a card directly into `player_idx`'s graveyard. Useful for test
    /// fixtures that exercise flashback / reanimate / dredge paths without
    /// the bookkeeping of casting and resolving the spell first.
    pub fn add_card_to_graveyard(
        &mut self,
        player_idx: usize,
        def: CardDefinition,
    ) -> CardId {
        let id = self.next_id();
        self.players[player_idx]
            .graveyard
            .push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Clear summoning sickness from a permanent (convenience for tests).
    pub fn clear_sickness(&mut self, id: CardId) {
        if let Some(c) = self.battlefield_find_mut(id) {
            c.summoning_sick = false;
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over.is_some()
    }

    /// Attackers declared in this combat step (with their chosen target).
    pub fn attacking(&self) -> &[Attack] {
        &self.attacking
    }

    // ── Snapshot accessors ────────────────────────────────────────────────────
    //
    // These are read/write helpers used by `crate::snapshot` to capture and
    // restore otherwise-private fields. They aren't intended for general
    // callers; the snapshot module guards round-trip correctness with tests.

    pub fn block_map(&self) -> &HashMap<CardId, CardId> {
        &self.block_map
    }

    /// Dry-run an action: clone the state, apply the action on the
    /// clone, return whether the engine would accept it. The caller's
    /// state is **not** modified. Used by the random bot to filter
    /// out actions the engine would reject — it's the most robust
    /// way to bottom out edge cases (Teferi sorcery-locking instants,
    /// Damping Sphere mana tax, hexproof targets, stolen permanents,
    /// summoning sickness, …) without re-implementing every engine
    /// rule on the bot side.
    ///
    /// Cost: one full `GameState::clone` + one `perform_action`. The
    /// random bot does this only on actions it's about to submit, so
    /// the overhead is bounded by the number of bot ticks, not by
    /// the size of the search space.
    pub fn would_accept(&self, action: GameAction) -> bool {
        // pending_decision routes through `submit_decision`, which
        // both reads AND writes `self.pending_decision`. Cloning is
        // safe but the dry-run can spuriously reject a legal answer
        // because the cloned decider drops scripted state. For the
        // bot's purposes — filtering out illegal `CastSpell` / land
        // taps / attacks — the no-pending-decision path is what
        // matters; if a decision is pending the bot uses
        // `SubmitDecision` directly which doesn't go through here.
        let mut probe = self.clone();
        probe.perform_action(action).is_ok()
    }

    /// CardIds in `caster`'s hand they could begin casting (or play, for
    /// lands) **right now**. Drives the client's "castable" hand-card
    /// highlight.
    ///
    /// Authoritative: every candidate is dry-run through [`would_accept`],
    /// so the result already reflects timing (sorcery vs. instant speed,
    /// remaining land drops), auto-tappable mana, cost taxes, and target
    /// availability — exactly the gates the real cast would hit. Mirrors
    /// the bot's candidate construction in `server::bot`, but only probes
    /// each card's default mode at X = 0, so a card castable *only* in a
    /// non-default mode (or at higher X) may be omitted — acceptable for a
    /// visual hint.
    ///
    /// Returns empty unless `caster` currently holds priority: you can't
    /// cast without it, and short-circuiting skips the per-card state
    /// clones on everyone else's priority, keeping view projection cheap.
    ///
    /// [`would_accept`]: Self::would_accept
    pub fn castable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        // Snapshot what each probe needs up front so the immutable borrow
        // of `self.players[caster].hand` is released before the cloning
        // probes run. The effect is cloned only for targeted non-lands.
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .map(|c| {
                let is_land = c.definition.is_land();
                let needs_target = !is_land && c.definition.effect.requires_target();
                (c.id, is_land, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();

        let mut out = Vec::new();
        for (id, is_land, targeted_effect) in &hand {
            let accepted = if *is_land {
                self.would_accept(GameAction::PlayLand(*id))
            } else {
                // Auto-pick targets the way the bot does so a targeted
                // removal spell isn't reported uncastable merely for lack
                // of a target argument. A spell that needs a target but has
                // no legal one stays `target: None`, which `would_accept`
                // correctly rejects (CR 601.2c).
                let (target, additional_targets) = match targeted_effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                };
                self.would_accept(GameAction::CastSpell {
                    card_id: *id,
                    target,
                    additional_targets,
                    mode: None,
                    x_value: None,
                })
            };
            if accepted {
                out.push(*id);
            }
        }
        out
    }

    /// CardIds of permanents `seat` controls with at least one activated
    /// ability they could activate **right now** — dry-run through
    /// [`would_accept`] so timing, mana, tap state, and target availability
    /// are all honored. Drives a client "this permanent can do something"
    /// highlight (legal-plays hint, roadmap Tier 7/8). Empty off-priority.
    ///
    /// [`would_accept`]: Self::would_accept
    pub fn activatable_permanents(&self, seat: usize) -> Vec<CardId> {
        if self.player_with_priority() != seat {
            return Vec::new();
        }
        // Snapshot (id, [ability effects]) so the borrow of `self.battlefield`
        // is released before the cloning probes run.
        let perms: Vec<(CardId, Vec<Option<Effect>>)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && !c.definition.activated_abilities.is_empty())
            .map(|c| {
                let effs = c
                    .definition
                    .activated_abilities
                    .iter()
                    .map(|a| a.effect.requires_target().then(|| a.effect.clone()))
                    .collect();
                (c.id, effs)
            })
            .collect();

        let mut out = Vec::new();
        for (id, ability_effects) in &perms {
            let any = ability_effects.iter().enumerate().any(|(idx, targeted)| {
                let target = match targeted {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, seat, None).0,
                    None => None,
                };
                self.would_accept(GameAction::ActivateAbility {
                    card_id: *id,
                    ability_index: idx,
                    target,
                    x_value: None,
                })
            });
            if any {
                out.push(*id);
            }
        }
        out
    }

    /// Creatures `seat` controls that could be declared as attackers right
    /// now — only meaningful during `seat`'s Declare Attackers step while it
    /// holds priority. Drives the client's legal-attacker highlight
    /// (roadmap Tier 8). Honors tapped / summoning-sickness / Defender /
    /// CantAttack via `CardInstance::can_attack`.
    pub fn legal_attackers(&self, seat: usize) -> Vec<CardId> {
        if self.step != crate::TurnStep::DeclareAttackers
            || self.active_player_idx != seat
            || self.player_with_priority() != seat
        {
            return Vec::new();
        }
        use crate::card::Keyword;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.can_attack())
            .filter(|c| {
                // Honor layer-granted Defender / can't-attack and the
                // per-defender attack restriction (Dandân) so the client's
                // highlight matches what `declare_attackers` will accept.
                let kws = self
                    .computed_permanent(c.id)
                    .map(|cp| cp.keywords.clone())
                    .unwrap_or_else(|| c.definition.keywords.clone());
                if kws.contains(&Keyword::Defender) || kws.contains(&Keyword::CantAttack) {
                    return false;
                }
                kws.iter().all(|kw| match kw {
                    Keyword::CanAttackOnlyIfDefenderControls(req) => {
                        // Legal if at least one alive opponent's board satisfies
                        // the filter (they could be chosen as the defender).
                        (0..self.players.len()).any(|d| {
                            d != seat
                                && self.players[d].is_alive()
                                && self.battlefield.iter().any(|p| {
                                    p.controller == d
                                        && self.evaluate_requirement_on_card(req, p, d)
                                })
                        })
                    }
                    _ => true,
                })
            })
            .map(|c| c.id)
            .collect()
    }

    /// Creatures `seat` controls that could legally block at least one of the
    /// currently-declared attackers — only meaningful during the Declare
    /// Blockers step with attackers on the board. Drives the client's
    /// legal-blocker highlight (roadmap Tier 8). Uses
    /// `can_block_any_attacker` so flying / menace-style restrictions apply.
    pub fn legal_blockers(&self, seat: usize) -> Vec<CardId> {
        if self.step != crate::TurnStep::DeclareBlockers || self.attacking().is_empty() {
            return Vec::new();
        }
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.can_block())
            .filter(|c| self.can_block_any_attacker(c.id))
            .map(|c| c.id)
            .collect()
    }

    /// Hand cards the viewer could cast *with their Kicker paid* right now
    /// (CR 702.32) — computed via a `CastSpellKicked` dry-run so it accounts
    /// for the full base+kicker cost, timing, and the kicked target set.
    /// Lets the client surface a "pay kicker?" affordance on those cards.
    /// Empty when the viewer doesn't hold priority.
    pub fn kickable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_kicker().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            // Use the kicked target set (the broader `If(SpellWasKicked, …)`
            // branch) so a kicked Tear Asunder reports castable at a creature.
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => {
                        self.auto_targets_for_effect_all_slots_kicked(eff, caster, None, true)
                    }
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if self.would_accept(GameAction::CastSpellKicked {
                card_id: *id,
                target,
                additional_targets,
                mode: None,
                x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CardIds in the caster's hand they could cast right now paying the
    /// optional Buyback cost (CR 702.27). Mirrors `kickable_hand_cards`.
    pub fn buyback_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_buyback().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if self.would_accept(GameAction::CastSpellBuyback {
                card_id: *id,
                target,
                additional_targets,
                mode: None,
                x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CardIds in the caster's hand that have Bestow and could be cast as an
    /// Aura on some creature right now (CR 702.103). The auto-target picks a
    /// creature; an empty result means no legal host or the cost is unpayable.
    pub fn bestowable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        let candidates: Vec<CardId> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_bestow().is_some())
            .map(|c| c.id)
            .collect();
        let mut out = Vec::new();
        for id in candidates {
            // Bestow needs a creature host; pick any legal creature target.
            let host = self
                .battlefield
                .iter()
                .find(|c| {
                    c.definition.is_creature()
                        && self
                            .check_target_legality_with_source(
                                &Target::Permanent(c.id),
                                caster,
                                Some(id),
                            )
                            .is_ok()
                })
                .map(|c| c.id);
            let Some(host) = host else { continue };
            if self.would_accept(GameAction::CastBestow {
                card_id: id,
                target: Some(Target::Permanent(host)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }) {
                out.push(id);
            }
        }
        out
    }

    /// Hand cards the player can activate a `from_hand` ability of right now
    /// (Spirit-Guide-style "Exile this from your hand: Add mana"). Lets the
    /// client surface a pitch affordance distinct from the castable-for-value
    /// highlight. Empty when it isn't the player's priority.
    pub fn pitchable_hand_cards(&self, player: usize) -> Vec<CardId> {
        if self.player_with_priority() != player {
            return Vec::new();
        }
        self.players[player]
            .hand
            .iter()
            .filter(|c| {
                c.definition
                    .activated_abilities
                    .iter()
                    .any(|a| a.from_hand)
            })
            .map(|c| c.id)
            .collect()
    }

    /// Hand cards the player could cast right now via their **Dash**
    /// alternative cost (CR 702.110). Lets the client surface a dash
    /// affordance distinct from the normal castable highlight. Empty when
    /// it isn't the player's priority.
    pub fn dashable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.alternative_cost.as_ref().is_some_and(|a| a.dash))
            .map(|c| c.id)
            .filter(|&id| {
                self.would_accept(GameAction::CastSpellAlternative {
                    card_id: id,
                    pitch_card: None,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// Extra generic mana the caster owes on top of `card`'s printed
    /// cost — Damping Sphere's "+1 after the first spell each turn,"
    /// Chancellor of the Annex's first-spell tax, etc. Public so the
    /// bot's affordability check can match the engine's payment path:
    /// `can_afford` ignoring this tax causes the bot to repeatedly
    /// submit a `CastSpell` that the engine then rejects with a mana
    /// shortfall, which (in spectate mode) deadlocks the match.
    pub fn extra_cost_for_card_in_hand(&self, caster: usize, card_id: CardId) -> u32 {
        let Some(card) = self.players[caster]
            .hand
            .iter()
            .find(|c| c.id == card_id)
        else {
            return 0;
        };
        crate::game::actions::extra_cost_for_spell(self, caster, card)
    }
    pub fn blockers_declared(&self) -> bool {
        self.blockers_declared
    }
    pub fn skip_first_draw(&self) -> bool {
        self.skip_first_draw
    }
    pub fn peek_next_id(&self) -> u32 {
        self.next_id
    }
    pub fn set_next_id(&mut self, value: u32) {
        self.next_id = value;
    }
    pub fn set_attacking(&mut self, attacks: Vec<Attack>) {
        self.attacking = attacks;
    }
    pub fn set_block_map(&mut self, map: HashMap<CardId, CardId>) {
        self.block_map = map;
    }
    pub fn set_blockers_declared(&mut self, value: bool) {
        self.blockers_declared = value;
    }
    pub fn set_skip_first_draw(&mut self, value: bool) {
        self.skip_first_draw = value;
    }

    /// Convenience: just the IDs of all declared attackers.
    pub fn attacking_ids(&self) -> Vec<CardId> {
        self.attacking.iter().map(|a| a.attacker).collect()
    }

    /// Snapshot of the current blocker → attacker assignments. Lets the
    /// view layer expose blocks per-permanent without making `block_map`
    /// public.
    pub fn block_map_snapshot(&self) -> Vec<(CardId, CardId)> {
        self.block_map.iter().map(|(b, a)| (*b, *a)).collect()
    }

    /// Look up the attack record for a given attacker id, if any.
    pub fn attack_for(&self, attacker: CardId) -> Option<&Attack> {
        self.attacking.iter().find(|a| a.attacker == attacker)
    }

    /// Resolve the defending player for a given attack target.
    pub fn defender_for(&self, target: AttackTarget) -> Option<usize> {
        match target {
            AttackTarget::Player(p) => Some(p),
            AttackTarget::Planeswalker(pw) => {
                self.battlefield_find(pw).map(|c| c.controller)
            }
        }
    }

    /// True if `blocker_id` can legally block at least one current attacker.
    pub fn can_block_any_attacker(&self, blocker_id: CardId) -> bool {
        let Some(blocker) = self.battlefield.iter().find(|c| c.id == blocker_id) else {
            return false;
        };
        if !blocker.can_block() {
            return false;
        }
        let computed = self.compute_battlefield();
        let blocker_computed = computed.iter().find(|c| c.id == blocker_id);
        let Some(blocker_cp) = blocker_computed else {
            return false;
        };
        // Honor `Keyword::CantBlock` from the computed keyword set —
        // transient grants from pump spells (Duel Tactics) and static
        // restrictions (Postmortem Professor) both surface here.
        if blocker_cp.keywords.contains(&Keyword::CantBlock) {
            return false;
        }
        self.attacking.iter().any(|atk| {
            let attacker = self.battlefield.iter().find(|c| c.id == atk.attacker);
            let atk_cp = computed.iter().find(|c| c.id == atk.attacker);
            let atk_kws = atk_cp.map(|c| c.keywords.as_slice()).unwrap_or(&[]);
            let atk_colors = atk_cp.map(|c| c.colors.as_slice()).unwrap_or(&[]);
            attacker
                .map(|a| can_block_attacker_computed(blocker, a, blocker_cp, atk_kws, atk_colors))
                .unwrap_or(false)
        })
    }

    /// True if `blocker_id` can legally block `attacker_id`.
    pub fn blocker_can_block_attacker(&self, blocker_id: CardId, attacker_id: CardId) -> bool {
        let Some(blocker) = self.battlefield.iter().find(|c| c.id == blocker_id) else {
            return false;
        };
        let Some(attacker) = self.battlefield.iter().find(|c| c.id == attacker_id) else {
            return false;
        };
        let computed = self.compute_battlefield();
        let blocker_cp = computed.iter().find(|c| c.id == blocker_id);
        let Some(blocker_cp) = blocker_cp else {
            return false;
        };
        if blocker_cp.keywords.contains(&Keyword::CantBlock) {
            return false;
        }
        let atk_cp = computed.iter().find(|c| c.id == attacker_id);
        let atk_kws = atk_cp.map(|c| c.keywords.as_slice()).unwrap_or(&[]);
        let atk_colors = atk_cp.map(|c| c.colors.as_slice()).unwrap_or(&[]);
        can_block_attacker_computed(blocker, attacker, blocker_cp, atk_kws, atk_colors)
    }

    // ── Main action dispatch ──────────────────────────────────────────────────

    pub fn perform_action(&mut self, action: GameAction) -> Result<Vec<GameEvent>, GameError> {
        if self.is_game_over() {
            return Err(GameError::GameAlreadyOver);
        }
        // Routing for decision answers is unconditional; everything else must
        // wait until the pending decision is resolved.
        if let GameAction::SubmitDecision(answer) = action {
            return self.submit_decision(answer);
        }
        if self.pending_decision.is_some() {
            return Err(GameError::DecisionPending);
        }
        let events = match action {
            GameAction::PlayLand(id) => self.play_land(id),
            GameAction::PlayLandBack(id) => self.play_land_with_face(id, true),
            GameAction::CastSpell {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellKicked {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_kicked(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellBuyback {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_buyback(card_id, target, additional_targets, mode, x_value),
            GameAction::CastBestow {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_bestow(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellConvoke {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
                convoke_creatures,
            } => self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &convoke_creatures, &[], false, false, false),
            GameAction::CastSpellDelve {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
                delve_cards,
            } => self.cast_spell_with_delve(card_id, target, additional_targets, mode, x_value, &delve_cards),
            GameAction::CastSpellAlternative {
                card_id,
                pitch_card,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_alternative(card_id, pitch_card, target, additional_targets, mode, x_value),
            GameAction::CastFlashback {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_flashback(card_id, target, additional_targets, mode, x_value),
            GameAction::CastRetrace {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_retrace(card_id, target, additional_targets, mode, x_value),
            GameAction::CastEscape {
                card_id,
                exile_cards,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_escape(card_id, &exile_cards, target, additional_targets, mode, x_value),
            GameAction::CastFlashbackTap {
                card_id,
                tap_creatures,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_flashback_tap(card_id, &tap_creatures, target, additional_targets, mode, x_value),
            GameAction::CastFromZoneWithoutPaying {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_from_zone_without_paying(
                card_id, target, additional_targets, mode, x_value,
            ),
            GameAction::CastFromCommandZone {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_from_command_zone(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellBack {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_back_face(card_id, target, additional_targets, mode, x_value),
            GameAction::ActivateAbility {
                card_id,
                ability_index,
                target,
                x_value,
            } => self.activate_ability(card_id, ability_index, target, x_value),
            GameAction::ActivateLoyaltyAbility {
                card_id,
                ability_index,
                target,
            } => self.activate_loyalty_ability(card_id, ability_index, target),
            GameAction::DeclareAttackers(ids) => self.declare_attackers(ids),
            GameAction::DeclareBlockers(assignments) => self.declare_blockers(assignments),
            GameAction::PassPriority => self.pass_priority(),
            GameAction::SubmitDecision(_) => unreachable!(),
            GameAction::Cycle { card_id } => self.cycle_card(card_id),
            GameAction::Equip { equipment, target } => self.equip(equipment, target),
            GameAction::Crew { vehicle, crew_creatures } => self.crew(vehicle, &crew_creatures),
            GameAction::Ninjutsu { ninja, returning } => self.ninjutsu(ninja, returning),
        }?;
        self.dispatch_triggers_for_events(&events);
        Ok(events)
    }

    /// CR 701.8 / 702.35 — discard `card_id` from player `p`'s hand. This
    /// is the single hand-to-graveyard discard path; the random/chosen
    /// `Effect::Discard` branches both route through it so the discard
    /// bookkeeping and the Madness replacement live in one place.
    ///
    /// The discard itself always happens (`CardDiscarded` fires and the
    /// per-resolution discard-matters counters bump) regardless of where
    /// the card ends up. CR 702.35a: a discarded card with
    /// `Keyword::Madness` is exiled instead of going to the graveyard, then
    /// its owner is offered a cast for the madness cost (see
    /// `offer_madness_cast`); declining or being unable to pay sends it on
    /// to the graveyard (CR 702.35b). Returns `true` if the card was found
    /// and discarded.
    pub(crate) fn discard_card(
        &mut self,
        p: usize,
        card_id: crate::card::CardId,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let Some(card) = self.players[p].remove_from_hand(card_id) else {
            return false;
        };
        let was_creature = card
            .definition
            .card_types
            .contains(&crate::card::CardType::Creature);
        let madness = card.definition.madness_cost().cloned();

        // The discard happens regardless of the destination zone (CR
        // 701.8b), so emit the event + bump the discard-matters counters
        // up front, before resolving the Madness replacement.
        events.push(GameEvent::CardDiscarded { player: p, card_id });
        self.cards_discarded_this_resolution += 1;
        *self
            .cards_discarded_per_player_this_resolution
            .entry(p)
            .or_insert(0) += 1;
        self.discarded_card_ids_this_resolution.push(card_id);
        if was_creature {
            self.creature_cards_discarded_this_resolution += 1;
        }

        match madness {
            None => {
                self.players[p].graveyard.push(card);
            }
            Some(cost) => {
                // CR 702.35a — exile instead of graveyard, then offer the
                // cast for the madness cost.
                self.exile.push(card);
                if !self.offer_madness_cast(p, card_id, &cost, events) {
                    // CR 702.35b — declined / unaffordable: the card goes
                    // from exile to its owner's graveyard.
                    if let Some(pos) = self.exile.iter().position(|c| c.id == card_id) {
                        let c = self.exile.remove(pos);
                        let owner = c.owner;
                        self.players[owner].graveyard.push(c);
                    }
                }
            }
        }
        true
    }

    /// CR 702.35b — offer the owner of an exiled Madness card a yes/no cast
    /// for `cost`, paid from their floated mana pool. Returns `true` if the
    /// spell was cast (it is now on the stack, sourced from exile). Mirrors
    /// the `Effect::MayPay` decision/payment shape; the `AutoDecider`
    /// declines by default so ordinary bot games never auto-cast.
    fn offer_madness_cast(
        &mut self,
        p: usize,
        card_id: crate::card::CardId,
        cost: &crate::mana::ManaCost,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let answer = self.decider.decide(&Decision::OptionalTrigger {
            source: card_id,
            description: "Cast for madness".to_string(),
        });
        if !matches!(answer, DecisionAnswer::Bool(true)) {
            return false;
        }
        // Pre-flight: try paying. On failure (unaffordable pool), decline.
        if self.players[p].mana_pool.pay(cost).is_err() {
            return false;
        }
        match self.cast_card_for_free(
            p,
            card_id,
            crate::card::Zone::Exile,
            None,
            vec![],
            None,
            None,
            false,
        ) {
            Ok(mut ev) => {
                events.append(&mut ev);
                true
            }
            Err(_) => false,
        }
    }

    /// CR 702.29a — Activate Cycling on `card_id` from the active
    /// player's hand. Pre-flight gates: card must be in someone's hand
    /// (we use the priority holder's hand), must carry
    /// `Keyword::Cycling(cost)`, and the controller must be able to
    /// pay the mana cost from their pool. On success: pays the cost,
    /// discards the card to the controller's graveyard, then draws a
    /// card. Per CR 702.29c, "When you cycle this card" triggers fire
    /// from the discarded zone (graveyard); the engine emits
    /// `GameEvent::CardDiscarded` from `discard_card_from_hand` so
    /// discard-matters triggers see the cycle.
    fn cycle_card(&mut self, card_id: crate::card::CardId) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Keyword;
        let seat = self.player_with_priority();
        // Locate the card in `seat`'s hand and clone the cycling cost.
        let cycling_cost = self.players[seat]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition.keywords.iter().find_map(|kw| {
                    if let Keyword::Cycling(mc) = kw {
                        Some(mc.clone())
                    } else {
                        None
                    }
                })
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Pay the cycling cost from the floated mana pool.
        self.players[seat]
            .mana_pool
            .pay(&cycling_cost)
            .map_err(GameError::Mana)?;
        // Discard the card from hand via the centralized path (handles the
        // graveyard move, CardDiscarded, discard-matters counters, and the
        // Madness replacement, CR 702.35).
        let mut events = vec![];
        if self.discard_card(seat, card_id, &mut events) {
            // CR 702.29c — emit the cycle-specific event in addition to
            // the discard event, so "When you cycle this card" triggers
            // distinguish cycle from a regular hand discard.
            events.push(GameEvent::CardCycled {
                player: seat,
                card_id,
            });
        }
        // Draw a card (Dredge can replace this draw, CR 702.52).
        self.draw_one(seat, &mut events);
        Ok(events)
    }

    /// Draw one card for `p`, first offering the Dredge replacement
    /// (CR 702.52). Returns `false` only when the draw couldn't be
    /// satisfied (empty library) and no dredge replacement applied — the
    /// caller is responsible for the resulting loss SBA. Pushes
    /// `CardDrawn` for a normal draw, or `CardMilled` ×N +
    /// `CardLeftGraveyard` for a dredge.
    pub(crate) fn draw_one(&mut self, p: usize, events: &mut Vec<GameEvent>) -> bool {
        if self.try_dredge_instead_of_draw(p, events) {
            return true;
        }
        match self.players[p].draw_top() {
            Some(id) => {
                events.push(GameEvent::CardDrawn { player: p, card_id: id });
                true
            }
            None => false,
        }
    }

    /// CR 702.52 — Dredge. If `p` has a card with `Keyword::Dredge(n)` in
    /// their graveyard and at least `n` cards in their library, the player
    /// may replace a draw by milling `n` cards and returning the dredge
    /// card to hand instead. Returns `true` when a dredge replacement was
    /// applied (caller skips the normal draw). The decision is surfaced as
    /// an `OptionalTrigger`, so the `AutoDecider` declines by default and
    /// ordinary games keep drawing normally.
    fn try_dredge_instead_of_draw(
        &mut self,
        p: usize,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        use crate::card::Keyword;
        use crate::decision::{Decision, DecisionAnswer};
        // First dredge card in the graveyard whose count the library can
        // satisfy (CR 702.52a — you can't dredge with fewer than N cards).
        let cand = self.players[p].graveyard.iter().find_map(|c| {
            c.definition.keywords.iter().find_map(|kw| match kw {
                Keyword::Dredge(n) if self.players[p].library.len() >= *n as usize => {
                    Some((c.id, *n))
                }
                _ => None,
            })
        });
        let Some((card_id, n)) = cand else { return false; };
        let answer = self.decider.decide(&Decision::OptionalTrigger {
            source: card_id,
            description: format!(
                "Dredge {n}: mill {n} card(s) and return this card to your hand instead of drawing?"
            ),
        });
        if !matches!(answer, DecisionAnswer::Bool(true)) {
            return false;
        }
        // Mill N from the top of the library.
        for _ in 0..n {
            if self.players[p].library.is_empty() {
                break;
            }
            let card = self.players[p].library.remove(0);
            let cid = card.id;
            self.players[p].graveyard.push(card);
            events.push(GameEvent::CardMilled { player: p, card_id: cid });
        }
        // Return the dredge card from the graveyard to its owner's hand.
        if let Some(pos) = self.players[p].graveyard.iter().position(|c| c.id == card_id) {
            let card = self.players[p].graveyard.remove(pos);
            self.players[p].hand.push(card);
            self.players[p].cards_left_graveyard_this_turn = self.players[p]
                .cards_left_graveyard_this_turn
                .saturating_add(1);
            events.push(GameEvent::CardLeftGraveyard { player: p, card_id });
        }
        true
    }

    /// CR 702.6 — Activate an Equipment's equip ability, attaching it to a
    /// creature its controller controls. Equip is a special activated
    /// ability usable only at sorcery speed (CR 702.6e) and only targeting a
    /// creature you control (CR 702.6c). The equip cost (`Keyword::Equip`) is
    /// paid from the controller's floated mana pool; on success the
    /// Equipment's `attached_to` is repointed at `target`, and its
    /// `equipped_bonus` flows onto the equipped creature via the layer
    /// system (see `compute_battlefield`). Re-equipping a creature that's
    /// already wearing the Equipment is legal (it just re-pays the cost);
    /// moving from one creature to another silently detaches the old link.
    fn equip(
        &mut self,
        equipment: crate::card::CardId,
        target: crate::card::CardId,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // Sorcery-speed gate (CR 702.6e).
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Locate the Equipment; it must be on the battlefield, controlled by
        // the activating player, and actually be an Equipment with an equip
        // cost.
        let equip_pos = self
            .battlefield
            .iter()
            .position(|c| c.id == equipment)
            .ok_or(GameError::CardNotOnBattlefield(equipment))?;
        if self.battlefield[equip_pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        if !self.battlefield[equip_pos].definition.is_equipment() {
            return Err(GameError::NotEquipment(equipment));
        }
        let equip_cost = self.battlefield[equip_pos]
            .definition
            .has_equip()
            .cloned()
            .ok_or(GameError::NotEquipment(equipment))?;
        // The target must be a creature the activating player controls
        // (CR 702.6c). Use the computed view so animated/becomes-a-creature
        // permanents are honored.
        let target_ok = self
            .computed_permanent(target)
            .is_some_and(|c| {
                c.controller == p
                    && c.card_types.contains(&crate::card::CardType::Creature)
            });
        if !target_ok {
            return Err(GameError::InvalidTarget);
        }
        // Pay the equip cost from the floated mana pool.
        self.players[p]
            .mana_pool
            .pay(&equip_cost)
            .map_err(GameError::Mana)?;
        // Attach.
        self.battlefield[equip_pos].attached_to = Some(target);
        Ok(vec![GameEvent::AttachmentMoved {
            attachment: equipment,
            attached_to: Some(target),
        }])
    }

    /// CR 702.122 — Crew a Vehicle. Taps the listed creatures (each an
    /// untapped creature the activator controls, none being the Vehicle
    /// itself) whose total power must meet or exceed the Vehicle's crew
    /// number. On success, registers an `UntilEndOfTurn` layer-4
    /// `AddCardType(Creature)` continuous effect so the Vehicle is an
    /// artifact creature for the rest of the turn (its printed P/T comes
    /// through the layer system via `base_power`/`base_toughness`). Crew is
    /// usable at instant speed (CR 702.122c), so there's no sorcery-speed
    /// gate. Re-crewing an already-crewed Vehicle is legal but pointless;
    /// the engine still taps the creatures and stacks a redundant effect.
    fn crew(
        &mut self,
        vehicle: crate::card::CardId,
        crew_creatures: &[crate::card::CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let veh_pos = self
            .battlefield
            .iter()
            .position(|c| c.id == vehicle)
            .ok_or(GameError::CardNotOnBattlefield(vehicle))?;
        if self.battlefield[veh_pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        let crew_n = self.battlefield[veh_pos]
            .definition
            .crew_cost()
            .ok_or(GameError::InvalidTarget)?;
        // Validate the crew: distinct, controlled by p, untapped creatures,
        // none being the Vehicle itself. Sum their computed power.
        let computed = self.compute_battlefield();
        let mut seen = std::collections::HashSet::new();
        let mut total_power: i32 = 0;
        for &cid in crew_creatures {
            if cid == vehicle || !seen.insert(cid) {
                return Err(GameError::InvalidTarget);
            }
            let Some(cp) = computed.iter().find(|c| c.id == cid) else {
                return Err(GameError::CardNotOnBattlefield(cid));
            };
            if cp.controller != p || !cp.card_types.contains(&crate::card::CardType::Creature) {
                return Err(GameError::InvalidTarget);
            }
            let tapped = self
                .battlefield
                .iter()
                .find(|c| c.id == cid)
                .map(|c| c.tapped)
                .unwrap_or(true);
            if tapped {
                return Err(GameError::CardIsTapped(cid));
            }
            total_power += cp.power.max(0);
        }
        if (total_power as u32) < crew_n {
            return Err(GameError::SelectionRequirementViolated);
        }
        // Tap the crew.
        let mut events = vec![];
        for &cid in crew_creatures {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
                c.tapped = true;
                events.push(GameEvent::PermanentTapped { card_id: cid });
            }
        }
        // Animate the Vehicle until end of turn.
        let ts = self.next_timestamp();
        self.add_continuous_effect(crate::game::layers::ContinuousEffect {
            timestamp: ts,
            source: vehicle,
            affected: crate::game::layers::AffectedPermanents::Source,
            layer: crate::game::layers::Layer::L4Type,
            sublayer: None,
            duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
            modification: crate::game::layers::Modification::AddCardType(
                crate::card::CardType::Creature,
            ),
        });
        events.push(GameEvent::VehicleCrewed { vehicle });
        Ok(events)
    }

    /// CR 702.49 — Ninjutsu. During the declare-blockers step, the active
    /// player returns an unblocked attacker (`returning`) to hand and puts
    /// `ninja` from hand onto the battlefield tapped and attacking the same
    /// defender, paying the ninjutsu cost.
    fn ninjutsu(
        &mut self,
        ninja: crate::card::CardId,
        returning: crate::card::CardId,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Keyword;
        if self.step != crate::TurnStep::DeclareBlockers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        let p = self.player_with_priority();
        // The returning creature must be one of this player's unblocked
        // attackers (not a value in `block_map`).
        let Some(atk) = self.attack_for(returning).copied() else {
            return Err(GameError::InvalidTarget);
        };
        let returning_controller = self
            .battlefield
            .iter()
            .find(|c| c.id == returning)
            .map(|c| c.controller);
        if returning_controller != Some(p) {
            return Err(GameError::NotYourPriority);
        }
        if self.block_map.values().any(|&a| a == returning) {
            return Err(GameError::InvalidTarget); // blocked — illegal
        }
        // The ninja must be in `p`'s hand and carry Ninjutsu; clone its cost.
        let cost = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == ninja)
            .and_then(|c| {
                c.definition.keywords.iter().find_map(|kw| match kw {
                    Keyword::Ninjutsu(mc) => Some(mc.clone()),
                    _ => None,
                })
            })
            .ok_or(GameError::CardNotInHand(ninja))?;
        self.players[p].mana_pool.pay(&cost).map_err(GameError::Mana)?;

        let mut events = vec![];
        // Return the unblocked attacker to its owner's hand (this prunes it
        // from `attacking` via `remove_from_combat` inside `move_card_to`).
        let owner = self.find_card_owner(returning).unwrap_or(p);
        let ctx = crate::game::effects::EffectContext::for_trigger(returning, p, None, 0);
        self.move_card_to(
            returning,
            &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::Seat(owner)),
            &ctx,
            &mut events,
        );
        // Put the ninja onto the battlefield tapped (ETB fires here).
        let ninja_ctx = crate::game::effects::EffectContext::for_trigger(ninja, p, None, 0);
        self.move_card_to(
            ninja,
            &crate::effect::ZoneDest::Battlefield {
                controller: crate::effect::PlayerRef::Seat(p),
                tapped: true,
            },
            &ninja_ctx,
            &mut events,
        );
        // It enters attacking the same defender the returned creature was
        // attacking — bypassing the declare-attackers timing/sickness gates.
        if self.battlefield.iter().any(|c| c.id == ninja) {
            self.attacking.push(Attack { attacker: ninja, target: atk.target });
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == ninja) {
                c.attacked_this_turn = true;
            }
            events.push(GameEvent::AttackerDeclared(ninja));
        }
        Ok(events)
    }

    /// Walk the battlefield looking for triggered abilities whose `EventSpec`
    /// matches any of `events`, and push matching triggers onto the stack.
    ///
    /// Events already handled by hardcoded trigger sites (ETB, attackers,
    /// spell-cast, dies, step changes) are skipped here to avoid double-firing.
    /// Everything else (TurnBegins, CardDrawn, LandPlayed, LifeGained, etc.)
    /// gains trigger capability through this path.
    pub(crate) fn dispatch_triggers_for_events(&mut self, events: &[GameEvent]) {
        if events.is_empty() {
            return;
        }
        // Event-keyed delayed triggers ("when [card] dies this turn, …").
        // Fire any `WhenCardDies(cid)` whose watched card appears in a
        // `CreatureDied` event in this batch, with its captured target.
        let died: Vec<CardId> = events
            .iter()
            .filter_map(|e| match e {
                GameEvent::CreatureDied { card_id } => Some(*card_id),
                _ => None,
            })
            .collect();
        if !died.is_empty() {
            use crate::game::types::DelayedKind;
            let mut fire: Vec<crate::game::types::DelayedTrigger> = Vec::new();
            let mut watched: Vec<CardId> = Vec::new();
            self.delayed_triggers.retain(|dt| {
                if let DelayedKind::WhenCardDies(cid) = dt.kind
                    && died.contains(&cid)
                {
                    fire.push(dt.clone());
                    watched.push(cid);
                    false
                } else {
                    true
                }
            });
            for (dt, cid) in fire.into_iter().zip(watched) {
                // Expose the dead creature as the trigger's source so bodies
                // can reference it (e.g. "exile it") via `Selector::This` /
                // `TriggerSource`; `target` still carries its controller.
                self.stack.push(crate::game::types::StackItem::Trigger {
                    source: dt.source,
                    controller: dt.controller,
                    effect: Box::new(dt.effect),
                    target: dt.target,
                    mode: None,
                    x_value: 0,
                    converged_value: 0,
                    trigger_source: Some(crate::game::effects::EntityRef::Card(cid)),
                    mana_spent: 0,
                    event_amount: 0,
                    intervening_if: None,
                });
            }
        }
        // Phase 1: collect candidate triggers while the borrow on
        // `self.battlefield` is shared. Phase 2 will mutate `self.stack`
        // and call `&self.evaluate_predicate` to gate each candidate by
        // the optional `EventSpec::filter`.
        let mut candidates: Vec<TriggerCandidate> = Vec::new();
        // Resolve per-permanent layer state once so the dispatcher can
        // honour `Modification::RemoveAllAbilities` (Turn to Frog,
        // Mercurial Transformation, Lignify) — printed triggered abilities
        // are skipped while a strip-abilities effect is in scope per CR
        // 113.10b.
        let computed = self.compute_battlefield();
        for card in &self.battlefield {
            let stripped = computed
                .iter()
                .find(|c| c.id == card.id)
                .map(|c| c.lost_all_abilities)
                .unwrap_or(false);
            if stripped {
                continue;
            }
            // Walk printed triggered abilities AND any transient
            // granted_triggers_eot for this permanent (Root Manipulation,
            // Rabid Attack-style "creatures gain '…trigger…' EOT").
            let all_triggers = card
                .definition
                .triggered_abilities
                .iter()
                .chain(self.granted_triggers(card.id));
            for ta in all_triggers {
                // For batch-fanout-friendly event kinds (Attacks,
                // CreatureDied, CardDrawn, CardDiscarded, CardLeftGraveyard,
                // CounterAdded, BlockerDeclared, AttackerWentUnblocked,
                // CardMilled, LifeGained, LifeLost) the trigger fires
                // ONCE PER MATCHING EVENT — CR 603.6 "whenever X happens"
                // fan-out. For other event kinds (ETB, StepBegins, …) we
                // fire at most once per (source, trigger) pair because
                // they don't naturally produce duplicate events in a
                // single batch.
                let fanout = matches!(
                    ta.event.kind,
                    crate::effect::EventKind::Attacks
                        | crate::effect::EventKind::CreatureDied
                        | crate::effect::EventKind::CreatureSacrificed
                        | crate::effect::EventKind::PermanentSacrificed
                        | crate::effect::EventKind::PermanentLeavesBattlefield
                        | crate::effect::EventKind::CardDrawn
                        | crate::effect::EventKind::CardDiscarded
                        | crate::effect::EventKind::CardLeftGraveyard
                        | crate::effect::EventKind::CounterAdded(_)
                        | crate::effect::EventKind::Blocks
                        | crate::effect::EventKind::BecomesBlocked
                        | crate::effect::EventKind::AttacksAndIsntBlocked
                        | crate::effect::EventKind::LifeGained
                        | crate::effect::EventKind::LifeLost
                        | crate::effect::EventKind::BecameTarget
                        // Enrage fires once per instance of damage
                        // (CR 702.130a) — fan out across the batch.
                        | crate::effect::EventKind::DealtDamage
                );
                for ev in events {
                    if is_event_hardcoded(ev, &ta.event) {
                        continue;
                    }
                    if crate::game::effects::event_matches_spec(self, ev, &ta.event, card) {
                        candidates.push(TriggerCandidate {
                            source: card.id,
                            effect: ta.effect.clone(),
                            controller: card.controller,
                            filter: ta.event.filter.clone(),
                            subject: crate::game::effects::event_subject(ev, &ta.event.kind),
                            event_amount: event_amount(ev),
                            triggered_by_etb: matches!(ev, GameEvent::PermanentEntered { .. }),
                        });
                        if !fanout {
                            break;
                        }
                    }
                }
            }
        }
        // CR 702.130a / 603.10a — Enrage on lethal damage. A creature that
        // dies from the same damage that would trigger its "whenever this is
        // dealt damage" ability still triggers (the ability uses last-known
        // information). Such a creature is no longer on the battlefield by
        // dispatch time, so walk the just-died snapshots for SelfSource
        // `DealtDamage` triggers matching a `DamageDealt` event in this batch.
        // (Other SelfSource trigger kinds — die/leave — are handled via their
        // own dedicated paths, so this is scoped to DealtDamage only.)
        for snap in self.died_card_snapshots.values() {
            for ta in &snap.definition.triggered_abilities {
                if ta.event.kind != crate::effect::EventKind::DealtDamage
                    || ta.event.scope != crate::effect::EventScope::SelfSource
                {
                    continue;
                }
                for ev in events {
                    if crate::game::effects::event_matches_spec(self, ev, &ta.event, snap) {
                        candidates.push(TriggerCandidate {
                            source: snap.id,
                            effect: ta.effect.clone(),
                            controller: snap.controller,
                            filter: ta.event.filter.clone(),
                            subject: crate::game::effects::event_subject(ev, &ta.event.kind),
                            event_amount: event_amount(ev),
                            triggered_by_etb: false,
                        });
                    }
                }
            }
        }
        // Also walk every player's graveyard for triggers scoped
        // `FromYourGraveyard` — recursion creatures (Bloodghast,
        // Ichorid, Silversmote Ghoul) fire from there. The trigger's
        // effective controller is the card's owner. Per CR 702.29c,
        // SelfSource cycle triggers ("When you cycle this card") also
        // fire here — the cycled card is in graveyard at dispatch
        // time, and the trigger's source matches the cycled card by id.
        for player in &self.players {
            for card in &player.graveyard {
                for ta in &card.definition.triggered_abilities {
                    let from_gy_scope = matches!(
                        ta.event.scope,
                        crate::effect::EventScope::FromYourGraveyard
                    );
                    let cycle_self = matches!(
                        ta.event.kind,
                        crate::effect::EventKind::CardCycled
                    ) && matches!(
                        ta.event.scope,
                        crate::effect::EventScope::SelfSource
                    );
                    if !from_gy_scope && !cycle_self {
                        continue;
                    }
                    for ev in events {
                        if is_event_hardcoded(ev, &ta.event) {
                            continue;
                        }
                        if crate::game::effects::event_matches_spec(self, ev, &ta.event, card) {
                            candidates.push(TriggerCandidate {
                                source: card.id,
                                effect: ta.effect.clone(),
                                controller: card.owner,
                                filter: ta.event.filter.clone(),
                                subject: crate::game::effects::event_subject(ev, &ta.event.kind),
                                event_amount: event_amount(ev),
                                triggered_by_etb: matches!(ev, GameEvent::PermanentEntered { .. }),
                            });
                            break;
                        }
                    }
                }
            }
        }
        // Player-level emblems (CR 114). Each player's emblems carry
        // triggered abilities that fire from the command zone alongside
        // battlefield permanents. Event-keyed emblem triggers are handled
        // here (step-keyed ones — "at the beginning of your upkeep" — fire
        // in `fire_step_triggers`). `event_amount` carries the magnitude
        // through to the body via `Value::TriggerEventAmount`. Professor
        // Dellian Fel's -6 emblem ("Whenever you gain life, each opponent
        // loses that much life") rides this path.
        for seat_idx in 0..self.players.len() {
            for em_idx in 0..self.players[seat_idx].emblems.len() {
                let triggers = self.players[seat_idx].emblems[em_idx].triggered.clone();
                for ta in &triggers {
                    if matches!(
                        ta.event.kind,
                        crate::effect::EventKind::StepBegins(_) | crate::effect::EventKind::TurnBegins
                    ) {
                        continue;
                    }
                    for ev in events {
                        if crate::game::effects::emblem_event_matches(self, ev, &ta.event, seat_idx) {
                            candidates.push(TriggerCandidate {
                                source: CardId(0),
                                effect: ta.effect.clone(),
                                controller: seat_idx,
                                filter: ta.event.filter.clone(),
                                subject: crate::game::effects::event_subject(ev, &ta.event.kind),
                                event_amount: event_amount(ev),
                                triggered_by_etb: false,
                            });
                        }
                    }
                }
            }
        }
        // CR 603.3b — APNAP. When multiple abilities trigger off the same
        // batch of events, the active player puts their triggers on the
        // stack first (in any order they choose), then each non-active
        // player in turn order. Since the stack is LIFO, the active
        // player's triggers resolve LAST. Without this sort, candidates
        // were pushed in battlefield-iteration order, which produced
        // observable wrong orderings the moment more than one player
        // controlled a triggering permanent (acute for 4-player FFA, 2HG,
        // and Commander — invisible in 1v1 where there's only one
        // non-active player). Within a player's group we keep the
        // gathered order: stable sort means each player's
        // battlefield-iteration order is preserved as their chosen
        // order — fine for AutoDecider; a real UI player would pick.
        let n_players = self.players.len();
        let active = self.active_player_idx;
        let apnap_rank = |seat: usize| -> usize {
            if seat == active {
                return 0;
            }
            let mut s = active;
            for r in 1..=n_players {
                s = self.next_alive_seat(s);
                if s == seat {
                    return r;
                }
                if s == active {
                    break;
                }
            }
            // Eliminated / unknown controller: sort to the back so it
            // pushes last → resolves first. Triggers from a dead
            // permanent's owner shouldn't really hit this path, but
            // keep behavior deterministic if they do.
            n_players
        };
        candidates.sort_by_key(|c| apnap_rank(c.controller));

        // Prowess: inject +1/+1 EOT pump for each creature with the
        // Prowess keyword that does NOT already carry its own prowess()
        // triggered ability. Cards wired via shortcut::prowess() already
        // have a SpellCast trigger on their definition; we skip those to
        // avoid doubling the pump.
        for ev in events {
            if let GameEvent::SpellCast { player, card_id, .. } = ev {
                let is_creature_spell = self.stack.iter().any(|si| matches!(
                    si,
                    crate::game::types::StackItem::Spell { card, .. } if card.id == *card_id && card.definition.is_creature()
                ));
                if !is_creature_spell {
                    let prowess_ids: Vec<_> = self.battlefield.iter()
                        .filter(|c| {
                            c.controller == *player
                                && c.has_keyword(&Keyword::Prowess)
                                && !c.definition.triggered_abilities.iter().any(|ta| {
                                    matches!(ta.event.kind, crate::effect::EventKind::SpellCast)
                                })
                        })
                        .map(|c| c.id)
                        .collect();
                    for pid in prowess_ids {
                        candidates.push(TriggerCandidate {
                            source: pid,
                            effect: Effect::PumpPT {
                                what: crate::effect::Selector::This,
                                power: crate::effect::Value::Const(1),
                                toughness: crate::effect::Value::Const(1),
                                duration: crate::effect::Duration::EndOfTurn,
                            },
                            controller: *player,
                            filter: None,
                            subject: None,
                            event_amount: 0,
                            triggered_by_etb: false,
                        });
                    }
                }
            }
        }

        // CR 603.3b — let a `wants_ui` controller order their own
        // simultaneous triggers. After the APNAP regroup (stable so the
        // prowess pumps appended above rejoin their controller's run), we
        // walk each contiguous same-controller run of length ≥2 and ask
        // that player's decider for a stack-push order. Gated on `wants_ui`
        // so AutoDecider/bot games (and the bulk of the test suite) are
        // untouched; AutoDecider would keep the default order anyway.
        candidates.sort_by_key(|c| apnap_rank(c.controller));
        candidates = self.order_same_controller_triggers(candidates);

        // Phase 2: enforce the optional `EventSpec::filter` predicate now
        // that we're free to call `&self.evaluate_predicate`. The trigger's
        // source permanent is bound as `ctx.source`, and the event's
        // subject (cast spell, dying creature, attacker, etc.) is bound as
        // `Selector::TriggerSource` so filters can reference it.
        // Build the queue of triggers waiting to be pushed onto the
        // stack. `drain_trigger_queue` walks the queue and either
        // pushes each trigger with an auto-picked target, or — when
        // the controller has `wants_ui` and the effect needs a target
        // — suspends on `Decision::ChooseTarget` so the human can
        // pick. Remaining queue items are saved in
        // `ResumeContext::TriggerTargetPick` and drained on answer.
        let mut queue: Vec<PendingTriggerPush> = Vec::new();
        for candidate in candidates {
            let TriggerCandidate {
                source,
                effect,
                controller,
                filter,
                subject,
                event_amount,
                triggered_by_etb,
            } = candidate;
            if let Some(filter) = filter {
                let ctx = crate::game::effects::EffectContext {
                    controller,
                    source: Some(source),
                    targets: vec![],
                    trigger_source: subject,
                    mode: 0,
                    x_value: 0,
                    converged_value: 0,
                    mana_spent: 0,
                    source_name: None,
                    cast_from_hand: true,
                    event_amount,
                    kicked: false,
                };
                if !self.evaluate_predicate(&filter, &ctx) {
                    continue;
                }
            }
            // CR 700.2b — modal triggered ability mode pick at push-time.
            let mode = self.pick_trigger_mode(&effect, source);
            if triggered_by_etb {
                // Yarok / Elesh Norn replacement (CR 614). A `wants`-side
                // ETB-trigger multiplier scales how many times this
                // reaction trigger fires (0 = suppressed by an opponent's
                // Spotlight, 1 normally, 2+ with a doubler). Self-source ETB
                // triggers go through the hardcoded path in `actions.rs`
                // (also multiplied), so they aren't double-counted here.
                let mult = crate::game::actions::etb_trigger_multiplier(self, controller);
                for _ in 0..mult {
                    // Strict Proctor's CR 614 tax applies once per fire; a
                    // declined / unpayable tax sacrifices the source and
                    // halts the remaining fires.
                    if !crate::game::actions::apply_etb_trigger_tax(self, source, controller) {
                        break;
                    }
                    queue.push(PendingTriggerPush {
                        source,
                        controller,
                        effect: effect.clone(),
                        subject,
                        event_amount,
                        mode,
                        intervening_if: None,
                    });
                }
            } else {
                queue.push(PendingTriggerPush {
                    source,
                    controller,
                    effect,
                    subject,
                    event_amount,
                    mode,
                    intervening_if: None,
                });
            }
        }
        self.drain_trigger_queue(queue);
        // Clear the per-die-event snapshot cache
        // after the dispatcher finishes with this event batch. Any
        // subsequent SBA cycle re-populates the entries it needs at
        // that cycle's die-time, so stale entries from prior batches
        // can't leak into later trigger resolution.
        self.died_card_snapshots.clear();
    }

    /// CR 603.3b — reorder each contiguous run of same-controller triggers
    /// per the controller's chosen stack-push order. Only consults the
    /// decider for a `wants_ui` controller whose run has ≥2 triggers; every
    /// other run is returned unchanged (AutoDecider keeps the default order
    /// regardless). The decider's `TriggerOrder(ids)` lists the desired
    /// push order; ids it omits keep their original relative order at the
    /// end, so a partial or empty answer is always legal.
    fn order_same_controller_triggers(
        &mut self,
        candidates: Vec<TriggerCandidate>,
    ) -> Vec<TriggerCandidate> {
        let mut out: Vec<TriggerCandidate> = Vec::with_capacity(candidates.len());
        let mut i = 0;
        while i < candidates.len() {
            let ctrl = candidates[i].controller;
            let mut j = i + 1;
            while j < candidates.len() && candidates[j].controller == ctrl {
                j += 1;
            }
            let run = &candidates[i..j];
            if run.len() < 2 || !self.players.get(ctrl).is_some_and(|p| p.wants_ui) {
                out.extend_from_slice(run);
            } else {
                let labels: Vec<(CardId, String)> = run
                    .iter()
                    .map(|c| {
                        let name = self
                            .battlefield_find(c.source)
                            .map(|b| b.definition.name.to_string())
                            .unwrap_or_else(|| "Triggered ability".to_string());
                        (c.source, name)
                    })
                    .collect();
                let answer = self.decider.decide(&crate::decision::Decision::OrderTriggers {
                    player: ctrl,
                    triggers: labels,
                });
                let order = match answer {
                    crate::decision::DecisionAnswer::TriggerOrder(ids) => ids,
                    _ => vec![],
                };
                // Apply the requested order: take run entries in `order`,
                // then append any not named (stable) so the answer can be
                // partial or empty.
                let mut remaining: Vec<Option<TriggerCandidate>> =
                    run.iter().cloned().map(Some).collect();
                for id in order {
                    if let Some(pos) = remaining
                        .iter()
                        .position(|c| c.as_ref().is_some_and(|c| c.source == id))
                    {
                        out.push(remaining[pos].take().unwrap());
                    }
                }
                for slot in remaining.into_iter().flatten() {
                    out.push(slot);
                }
            }
            i = j;
        }
        out
    }

    /// Walk a queue of pending triggers, pushing each onto the stack.
    /// Suspends on the first trigger whose controller has `wants_ui`
    /// and whose effect needs a target — emits
    /// `Decision::ChooseTarget` and parks the remaining queue in
    /// `ResumeContext::TriggerTargetPick`. The resume path
    /// (`submit_decision`) re-enters this function with the remaining
    /// queue once the user picks.
    pub(crate) fn drain_trigger_queue(&mut self, queue: Vec<PendingTriggerPush>) {
        // Don't stack up multiple pending decisions — if the engine
        // already suspended on something else, leave the queue alone.
        // Trigger queues are episodic per event batch and we have
        // nowhere outside `ResumeContext::TriggerTargetPick` to park
        // them, so this matches the pre-fix behaviour (auto-target
        // everything) for the rare case where a pending decision
        // races a trigger batch.
        if self.pending_decision.is_some() {
            if !queue.is_empty() {
                eprintln!(
                    "engine: dropping {} pending trigger(s) — a decision was already \
                     pending when the trigger batch arrived",
                    queue.len()
                );
            }
            return;
        }
        // Walk the queue in *forward* (APNAP) order so the active
        // player's triggers push first and resolve last, matching CR
        // 603.3b. Using an iterator lets us collect the unconsumed
        // tail into `remaining` when we suspend mid-batch.
        let mut iter = queue.into_iter();
        while let Some(pending) = iter.next() {
            let needs = pending.effect.requires_target();
            let wants_ui = self
                .players
                .get(pending.controller)
                .map(|p| p.wants_ui)
                .unwrap_or(false);
            if needs && wants_ui {
                let legal = self.enumerate_legal_targets(&pending.effect, pending.controller);
                // No legal targets → fall back to auto (which returns
                // None) so the trigger still resolves CR-correctly as
                // a no-op rather than blocking the game on an
                // unanswerable picker.
                if legal.is_empty() {
                    self.push_pending_trigger(pending, None);
                    continue;
                }
                let remaining: Vec<PendingTriggerPush> = iter.collect();
                let source_name = self
                    .find_card_anywhere(pending.source)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                let description = pending.effect.effect_short_text();
                self.pending_decision = Some(PendingDecision {
                    decision: Decision::ChooseTarget {
                        source: pending.source,
                        legal,
                        source_name,
                        description,
                    },
                    resume: ResumeContext::TriggerTargetPick {
                        pending,
                        remaining,
                    },
                });
                return;
            }
            let auto = self.auto_target_for_effect(&pending.effect, pending.controller);
            self.push_pending_trigger(pending, auto);
        }
    }

    /// Push a `PendingTriggerPush` onto the stack with the given
    /// (already-chosen) target. Mirrors the original inline push at
    /// the trigger-dispatch site.
    pub(crate) fn push_pending_trigger(
        &mut self,
        pending: PendingTriggerPush,
        target: Option<Target>,
    ) {
        let PendingTriggerPush {
            source,
            controller,
            effect,
            subject,
            event_amount,
            mode,
            intervening_if,
        } = pending;
        self.stack.push(StackItem::Trigger {
            source,
            controller,
            effect: Box::new(effect),
            target,
            mode,
            x_value: 0,
            converged_value: 0,
            trigger_source: subject,
            mana_spent: 0,
            event_amount,
            intervening_if,
        });
    }


    /// Activate a loyalty ability on a planeswalker (sorcery speed, once per turn).
    pub fn activate_loyalty_ability(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let pos = self
            .battlefield
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotOnBattlefield(card_id))?;
        if self.battlefield[pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        if !self.battlefield[pos].definition.is_planeswalker() {
            return Err(GameError::InvalidTarget);
        }
        if self.battlefield[pos].used_loyalty_ability_this_turn {
            return Err(GameError::LoyaltyAbilityAlreadyUsed(card_id));
        }

        let ability = self.battlefield[pos]
            .definition
            .loyalty_abilities
            .get(ability_index)
            .cloned()
            .ok_or(GameError::AbilityIndexOutOfBounds)?;

        // Validate target — both targeting legality (hexproof / shroud /
        // protection / Leyline-of-Sanctity) and the loyalty effect's
        // own selector requirement (Teferi -3's "nonland permanent
        // an opponent controls" filter, etc.). Spell casts and
        // activated-ability activations both gate on these; loyalty
        // abilities went unchecked and would happily aim a Teferi -3
        // at the controller's own permanent.
        if let Some(tgt) = &target {
            self.check_target_legality(tgt, p)?;
            if let Some(filter) = ability.effect.target_filter_for_slot(0)
                && !self.evaluate_requirement_static(filter, tgt, p, Some(card_id))
            {
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // Apply loyalty cost.
        let current_loyalty =
            self.battlefield[pos].counter_count(crate::card::CounterType::Loyalty) as i32;
        let new_loyalty = current_loyalty + ability.loyalty_cost;
        if new_loyalty < 0 {
            return Err(GameError::NotEnoughLoyalty(card_id));
        }
        self.battlefield[pos]
            .counters
            .insert(crate::card::CounterType::Loyalty, new_loyalty as u32);
        self.battlefield[pos].used_loyalty_ability_this_turn = true;

        let loyalty_change = ability.loyalty_cost;
        let mut events = vec![
            GameEvent::LoyaltyAbilityActivated {
                planeswalker: card_id,
                loyalty_change,
            },
            GameEvent::LoyaltyChanged {
                card_id,
                new_loyalty,
            },
        ];

        // Push ability effects onto the stack.
        self.stack.push(StackItem::Trigger {
            source: card_id,
            controller: p,
            effect: Box::new(ability.effect),
            target,
            mode: None,
            x_value: 0,
            converged_value: 0,
        trigger_source: None,
            mana_spent: 0,
            event_amount: 0,
            intervening_if: None,
        });
        self.give_priority_to_active();

        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        Ok(events)
    }

    /// Begin the pre-game London-mulligan phase. Deals 7 cards to each player
    /// and sets `pending_decision` for seat 0's opening-hand choice.
    /// Call this after constructing the `GameState` and before the first turn.
    pub fn start_mulligan_phase(&mut self) {
        let n = self.players.len();
        for i in 0..n {
            self.deal_to_hand(i, 7);
        }
        self.set_mulligan_decision(0, 0, if n > 1 { Some(1) } else { None });
    }

    fn deal_to_hand(&mut self, seat: usize, count: usize) {
        // Top of library is `library[0]` — `pop()` would deal from the
        // bottom, which produces the wrong opening hand for unshuffled
        // (test-fixture) decks. Drain the top `count` cards instead.
        for _ in 0..count {
            if self.players[seat].library.is_empty() {
                break;
            }
            let card = self.players[seat].library.remove(0);
            self.players[seat].hand.push(card);
        }
    }

    fn shuffle_hand_to_library(&mut self, seat: usize) {
        use rand::seq::SliceRandom;
        let hand = std::mem::take(&mut self.players[seat].hand);
        for card in hand {
            self.players[seat].library.push(card);
        }
        let mut rng = rand::rng();
        self.players[seat].library.shuffle(&mut rng);
    }

    fn set_mulligan_decision(&mut self, player: usize, mulligans_taken: usize, next_player: Option<usize>) {
        let hand: Vec<_> = self.players[player].hand
            .iter()
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        // Surface any in-hand Serum Powder–style mulligan helpers so the
        // decider can pick an alternative answer.
        let serum_powders: Vec<_> = self.players[player].hand
            .iter()
            .filter(|c| matches!(
                c.definition.opening_hand,
                Some(crate::effect::OpeningHandEffect::MulliganHelper),
            ))
            .map(|c| c.id)
            .collect();
        self.pending_decision = Some(PendingDecision {
            decision: Decision::Mulligan { player, hand, mulligans_taken, serum_powders },
            resume: ResumeContext::Mulligan { player, mulligans_taken, next_player },
        });
    }

    /// Submit an answer to the currently-pending decision and resume resolution.
    /// Fails if no decision is pending, or the answer shape doesn't match the
    /// decision kind.
    pub fn submit_decision(&mut self, answer: DecisionAnswer) -> Result<Vec<GameEvent>, GameError> {
        let pd = self
            .pending_decision
            .take()
            .ok_or(GameError::NoDecisionPending)?;
        let mut events = match pd.resume {
            ResumeContext::Spell {
                card,
                caster,
                target,
                additional_targets,
                mode,
                x_value,
                converged_value,
                mana_spent,
                in_progress,
                remaining,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more = self.continue_spell_resolution(
                    *card,
                    caster,
                    target,
                    additional_targets,
                    mode,
                    x_value,
                    converged_value,
                    mana_spent,
                    Some(remaining),
                )?;
                evs.append(&mut more);
                evs
            }
            ResumeContext::Trigger {
                source,
                controller,
                target,
                mode,
                in_progress,
                remaining,
                x_value,
                converged_value,
                mana_spent,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more = self.continue_trigger_resolution(
                    source, controller, remaining, target, mode, x_value, converged_value, mana_spent,
                )?;
                evs.append(&mut more);
                evs
            }
            ResumeContext::Ability {
                source,
                controller,
                target,
                in_progress,
                remaining,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more = self.continue_ability_resolution(
                    source, controller, remaining, target,
                )?;
                evs.append(&mut more);
                evs
            }
            ResumeContext::Mulligan { player, mulligans_taken, next_player } => {
                match answer {
                    DecisionAnswer::TakeMulligan => {
                        self.shuffle_hand_to_library(player);
                        self.deal_to_hand(player, 7);
                        self.set_mulligan_decision(player, mulligans_taken + 1, next_player);
                        return Ok(vec![]);
                    }
                    DecisionAnswer::Keep => {
                        if mulligans_taken > 0 {
                            let hand = self.players[player].hand
                                .iter()
                                .map(|c| (c.id, c.definition.name.to_string()))
                                .collect();
                            self.pending_decision = Some(PendingDecision {
                                decision: Decision::PutOnLibrary {
                                    player,
                                    count: mulligans_taken,
                                    hand,
                                },
                                // Carry the mulligan count forward so the
                                // PutOnLibrary handler below knows how many
                                // cards to bottom.
                                resume: ResumeContext::Mulligan { player, mulligans_taken, next_player },
                            });
                            return Ok(vec![]);
                        }
                        self.advance_mulligan(next_player);
                        return Ok(vec![]);
                    }
                    DecisionAnswer::PutOnLibrary(ids) => {
                        // London mulligan: chosen cards go to the BOTTOM of
                        // the library (not the top — `insert(0, …)` would put
                        // them on top, which is the bug we're fixing).
                        for card_id in ids.iter().take(mulligans_taken) {
                            if let Some(pos) = self.players[player].hand.iter().position(|c| c.id == *card_id) {
                                let card = self.players[player].hand.remove(pos);
                                self.players[player].library.push(card);
                            }
                        }
                        self.advance_mulligan(next_player);
                        return Ok(vec![]);
                    }
                    DecisionAnswer::SerumPowder(powder_id) => {
                        // Serum Powder: exile the entire current hand (the
                        // powder card itself goes with it), then draw a new
                        // seven. Doesn't bump `mulligans_taken` — Serum
                        // Powder is intentionally separate from the London
                        // mulligan ladder (so multiple powders can stack
                        // without progressively shrinking the eventual hand).
                        // Reject if the named Serum Powder isn't actually in
                        // hand or doesn't carry the `MulliganHelper` flag.
                        let valid = self.players[player].hand.iter().any(|c| {
                            c.id == powder_id
                                && matches!(
                                    c.definition.opening_hand,
                                    Some(crate::effect::OpeningHandEffect::MulliganHelper),
                                )
                        });
                        if !valid {
                            return Err(GameError::DecisionAnswerMismatch);
                        }
                        let exiled: Vec<crate::card::CardInstance> =
                            std::mem::take(&mut self.players[player].hand);
                        for card in exiled {
                            self.exile.push(card);
                        }
                        self.deal_to_hand(player, 7);
                        self.set_mulligan_decision(player, mulligans_taken, next_player);
                        return Ok(vec![]);
                    }
                    _ => return Err(GameError::DecisionAnswerMismatch),
                }
            }
            ResumeContext::TriggerTargetPick { pending, remaining } => {
                // Apply the answered target to the trigger that was
                // waiting on it, then continue draining the queue
                // (which may suspend again on the next targeted
                // trigger in the same batch).
                let target = match answer {
                    DecisionAnswer::Target(t) => Some(t),
                    _ => return Err(GameError::DecisionAnswerMismatch),
                };
                self.push_pending_trigger(pending, target);
                self.drain_trigger_queue(remaining);
                vec![]
            }
        };
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        self.dispatch_triggers_for_events(&events);
        Ok(events)
    }

    fn advance_mulligan(&mut self, next_player: Option<usize>) {
        match next_player {
            Some(p) => {
                let after = (p + 1 < self.players.len()).then_some(p + 1);
                self.set_mulligan_decision(p, 0, after);
            }
            None => {
                // All players kept — apply opening-hand effects (Leyline of
                // Sanctity / Gemstone Caverns start in play; Chancellor reveals
                // schedule delayed triggers) and start the game with priority
                // on seat 0.
                self.apply_opening_hand_effects();
                self.pending_decision = None;
                self.give_priority_to_active();
            }
        }
    }

    /// Walk every player's opening hand and apply each card's
    /// `OpeningHandEffect`. The default `Decider` answers "yes" to every
    /// optional reveal — the `AutoDecider` and the bot benefit from these
    /// effects in the demo decks, and a future UI can deny the reveal by
    /// returning `Bool(false)` from an `OptionalTrigger` decision (not yet
    /// surfaced — opening-hand effects auto-fire today).
    /// Backwards-compat alias used by some tests — fires every player's
    /// opening-hand effects immediately. Equivalent to (and delegates to)
    /// `apply_opening_hand_effects`.
    pub fn fire_start_of_game_effects(&mut self) {
        self.apply_opening_hand_effects();
    }

    pub(crate) fn apply_opening_hand_effects(&mut self) {
        let n = self.players.len();
        for p in 0..n {
            // Snapshot ids first so we can iterate without aliasing the hand.
            let ids: Vec<crate::card::CardId> =
                self.players[p].hand.iter().map(|c| c.id).collect();
            for cid in ids {
                let oh = self.players[p]
                    .hand
                    .iter()
                    .find(|c| c.id == cid)
                    .and_then(|c| c.definition.opening_hand.clone());
                let Some(oh) = oh else { continue };
                match oh {
                    crate::effect::OpeningHandEffect::StartInPlay { tapped, extra } => {
                        // Pull the card out of hand and place it on the
                        // battlefield under its owner's control.
                        if let Some(pos) = self.players[p].hand.iter().position(|c| c.id == cid) {
                            let mut card = self.players[p].hand.remove(pos);
                            card.controller = p;
                            card.tapped = tapped;
                            card.summoning_sick = card.definition.is_creature();
                            self.battlefield.push(card);
                            // Run the optional follow-up effect (e.g. Gemstone
                            // Caverns wants a luck counter on its newly-entered
                            // self).
                            if !matches!(extra, crate::effect::Effect::Noop) {
                                let ctx = crate::game::effects::EffectContext::for_ability(
                                    cid, p, None,
                                );
                                let _ = self.resolve_effect(&extra, &ctx);
                            }
                            // Fire any self-source ETB triggers (the same hook
                            // play_land uses), so static-as-replaced abilities
                            // and "enters with N counters" still fire if the
                            // card uses that idiom in addition to `extra`.
                            self.fire_self_etb_triggers(cid, p);
                        }
                    }
                    crate::effect::OpeningHandEffect::RevealForDelayedTrigger { kind, body } => {
                        // Card stays in hand; register a delayed trigger that
                        // fires later (next upkeep / first main / end step).
                        use crate::game::types::DelayedTrigger;
                        let dk = crate::game::effects::delayed_kind_from_effect(kind);
                        self.delayed_triggers.push(DelayedTrigger {
                            controller: p,
                            source: cid,
                            kind: dk,
                            effect: body,
                            target: None,
                            fires_once: true,
                        });
                    }
                    crate::effect::OpeningHandEffect::MulliganHelper => {
                        // Surfaces during mulligan only; nothing to do here.
                    }
                }
            }
        }
    }

    /// Complete the suspended effect using the player's answer. Returns the
    /// events generated by the now-finished effect (e.g. `ScryPerformed`).
    pub(crate) fn apply_pending_effect_answer(
        &mut self,
        state: PendingEffectState,
        answer: &DecisionAnswer,
    ) -> Result<Vec<GameEvent>, GameError> {
        match state {
            PendingEffectState::ScryPeeked { count, player } => {
                let DecisionAnswer::ScryOrder { kept_top, bottom } = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut remaining: Vec<CardInstance> =
                    self.players[player].library.drain(..count).collect();
                let mut top_cards = Vec::with_capacity(kept_top.len());
                for id in kept_top {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        top_cards.push(remaining.remove(pos));
                    }
                }
                let mut bottom_cards = Vec::with_capacity(bottom.len());
                for id in bottom {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        bottom_cards.push(remaining.remove(pos));
                    }
                }
                // Cards listed in neither bucket default to top (end of top).
                top_cards.extend(remaining);
                let bottomed = bottom_cards.len();
                let lib = &mut self.players[player].library;
                for c in bottom_cards {
                    lib.push(c);
                }
                for c in top_cards.into_iter().rev() {
                    lib.insert(0, c);
                }
                Ok(vec![GameEvent::ScryPerformed {
                    player,
                    looked_at: count,
                    bottomed,
                }])
            }
            PendingEffectState::SurveilPeeked { count, player } => {
                // Surveil: player chooses which cards go to the graveyard; rest go to top.
                let DecisionAnswer::ScryOrder {
                    kept_top,
                    bottom: to_graveyard,
                } = answer
                else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut remaining: Vec<CardInstance> =
                    self.players[player].library.drain(..count).collect();
                let mut top_cards = Vec::with_capacity(kept_top.len());
                for id in kept_top {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        top_cards.push(remaining.remove(pos));
                    }
                }
                let mut graveyard_cards = Vec::with_capacity(to_graveyard.len());
                for id in to_graveyard {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        graveyard_cards.push(remaining.remove(pos));
                    }
                }
                top_cards.extend(remaining);
                let graveyarded = graveyard_cards.len();
                for c in graveyard_cards {
                    self.players[player].graveyard.push(c);
                }
                let lib = &mut self.players[player].library;
                for c in top_cards.into_iter().rev() {
                    lib.insert(0, c);
                }
                Ok(vec![GameEvent::SurveilPerformed {
                    player,
                    looked_at: count,
                    graveyarded,
                }])
            }
            PendingEffectState::LearnPending { player } => {
                let DecisionAnswer::Learn(choice) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                self.apply_learn_choice(player, choice.clone(), &mut events);
                Ok(events)
            }
            PendingEffectState::SearchPending { player, to } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                if let Some(card_id) = chosen_id
                    && let Some(pos) = self.players[player].library.iter().position(|c| c.id == *card_id) {
                    let card = self.players[player].library.remove(pos);
                    self.place_card_in_dest(card, player, &to, &mut events);
                }
                Ok(events)
            }
            PendingEffectState::ImpulsePending { player, revealed, rest_to_graveyard, eligible } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                // `None` eligible means "any revealed card" (no filter).
                let is_eligible = |id: &CardId| match &eligible {
                    None => true,
                    Some(v) => v.contains(id),
                };
                // Default (AutoDecider returns None / out-of-set): take the
                // first *eligible* revealed card. When nothing is eligible
                // (Satyr Wayfinder revealing no land), take nothing.
                let pick = chosen_id
                    .filter(|id| revealed.contains(id) && is_eligible(id))
                    .or_else(|| revealed.iter().copied().find(|id| is_eligible(id)));
                let mut events = vec![];
                if let Some(pick) = pick
                    && let Some(pos) = self.players[player].library.iter().position(|c| c.id == pick) {
                    let card = self.players[player].library.remove(pos);
                    self.players[player].hand.push(card);
                    events.push(GameEvent::CardDrawn { player, card_id: pick });
                }
                // Move the rest of the revealed set to the bottom of the
                // library (or graveyard). They're still at the top of the
                // library after the pick was removed.
                for rid in &revealed {
                    if Some(*rid) == pick {
                        continue;
                    }
                    if let Some(pos) = self.players[player].library.iter().position(|c| c.id == *rid) {
                        let card = self.players[player].library.remove(pos);
                        if rest_to_graveyard {
                            self.players[player].graveyard.push(card);
                        } else {
                            self.players[player].library.push(card);
                        }
                    }
                }
                Ok(events)
            }
            PendingEffectState::PayLifeLookPending { player, revealed } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                // Default (AutoDecider / out-of-set): take the top revealed.
                let pick = chosen_id
                    .filter(|id| revealed.contains(id))
                    .or_else(|| revealed.first().copied());
                let mut events = vec![];
                if let Some(pick) = pick
                    && let Some(pos) = self.players[player].library.iter().position(|c| c.id == pick) {
                    let card = self.players[player].library.remove(pos);
                    self.players[player].hand.push(card);
                    events.push(GameEvent::CardDrawn { player, card_id: pick });
                }
                // Exile the rest of the revealed set.
                for rid in &revealed {
                    if Some(*rid) == pick { continue; }
                    if let Some(pos) = self.players[player].library.iter().position(|c| c.id == *rid) {
                        let card = self.players[player].library.remove(pos);
                        self.exile.push(card);
                    }
                }
                Ok(events)
            }
            PendingEffectState::PutOnLibraryPending { player, .. } => {
                let DecisionAnswer::PutOnLibrary(chosen) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                self.execute_put_on_library(player, chosen, &mut events);
                Ok(events)
            }
            PendingEffectState::AnyOneColorPending { player, count, restriction } => {
                let DecisionAnswer::Color(c) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    match restriction {
                        Some(r) => self.players[player].mana_pool.add_restricted(*c, 1, r),
                        None => self.players[player].mana_pool.add(*c, 1),
                    }
                    events.push(GameEvent::ManaAdded { player, color: *c });
                }
                Ok(events)
            }
            PendingEffectState::DevotionColorPending { player } => {
                let DecisionAnswer::Color(c) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let n = self.devotion_to(player, &[*c]).max(0) as u32;
                let mut events = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    self.players[player].mana_pool.add(*c, 1);
                    events.push(GameEvent::ManaAdded { player, color: *c });
                }
                Ok(events)
            }
            PendingEffectState::DiscardChosenPending { target_player } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::with_capacity(card_ids.len());
                for cid in card_ids {
                    // The zone move + CardDiscarded + discard-matters
                    // counters + Madness replacement (CR 702.35) are all
                    // centralized in `discard_card`.
                    self.discard_card(target_player, *cid, &mut events);
                }
                Ok(events)
            }
            PendingEffectState::ExileChosenUntilSourceLeavesPending {
                target_player,
                source,
                return_to,
            } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::with_capacity(card_ids.len());
                for cid in card_ids {
                    // Move the chosen card from hand to exile and link it to
                    // the source permanent.
                    if let Some(pos) =
                        self.players[target_player].hand.iter().position(|c| c.id == *cid)
                    {
                        let mut card = self.players[target_player].hand.remove(pos);
                        card.exiled_by = Some(crate::card::ExileLink {
                            source,
                            return_to,
                        });
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: *cid });
                    }
                }
                Ok(events)
            }
            PendingEffectState::ChooseCreatureTypePending { target_id } => {
                let DecisionAnswer::CreatureType(ct) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                if let Some(card) = self.battlefield_find_mut(target_id) {
                    card.chosen_creature_type = Some(*ct);
                }
                Ok(Vec::new())
            }
            PendingEffectState::NameCardPending { target_id } => {
                let DecisionAnswer::NamedCard(name) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                if !name.is_empty() {
                    if let Some(card) = self.find_card_anywhere_mut(target_id) {
                        card.named_card = Some(name.clone());
                    }
                    // Also record on the per-resolution scratchpad so a
                    // following `NamedBySource` reveal can match even when the
                    // naming source is a resolving spell held off-zone.
                    self.named_card_this_resolution = Some(name.clone());
                }
                Ok(Vec::new())
            }
        }
    }

    /// Resolve a spell's effect tree. On suspension, installs a
    /// `pending_decision` and returns events accumulated so far. `override_effect`
    /// is used on resume to continue with whatever Seq tail was left after the
    /// suspending effect — pass `None` for the initial resolution and `Some(...)`
    /// when continuing from `submit_decision`.
    //
    // The argument list is wide because the spell-state quartet (target, mode,
    // x_value, converged_value) must be preserved across suspend/resume so the
    // spell can re-run its effect tree with the original cast-time choices.
    // The two callers (initial cast in `stack.rs` and resume in
    // `submit_decision`) both hand off these fields directly from a
    // `StackItem::Spell` / `ResumeContext::Spell`, so wrapping them in a
    // struct doesn't reduce coupling at the call sites.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn continue_spell_resolution(
        &mut self,
        card: CardInstance,
        caster: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
        override_effect: Option<Effect>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let effect = override_effect.unwrap_or_else(|| card.definition.effect.clone());
        let mut ctx = EffectContext::for_spell_with_source_and_origin(
            card.id,
            card.definition.name,
            caster,
            target.clone(),
            additional_targets.clone(),
            mode,
            x_value,
            converged_value,
            mana_spent,
            card.cast_from_hand,
        );
        ctx.kicked = card.kicked;
        let events = self.resolve_effect(&effect, &ctx)?;
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Spell {
                    card: Box::new(card),
                    caster,
                    target,
                    additional_targets,
                    mode,
                    x_value,
                    converged_value,
                    mana_spent,
                    in_progress,
                    remaining,
                },
            });
            return Ok(events);
        }
        // Rebound: if this card has Keyword::Rebound and was cast from
        // hand, exile it instead of sending it to the graveyard, and
        // schedule a delayed trigger at the caster's next upkeep that
        // re-runs the spell's effect with a fresh auto-target.
        if card.cast_from_hand
            && card.definition.keywords.contains(&crate::card::Keyword::Rebound)
        {
            use crate::game::types::{DelayedKind, DelayedTrigger};
            let source = card.id;
            let body = card.definition.effect.clone();
            self.delayed_triggers.push(DelayedTrigger {
                controller: caster,
                source,
                kind: DelayedKind::YourNextUpkeep,
                effect: body,
                target: None, // re-pick at fire time
                fires_once: true,
            });
            self.exile.push(card);
            return Ok(events);
        }
        // Flashback (CR 702.34d): a spell cast via its Flashback cost is
        // exiled on resolution instead of going to the graveyard.
        // `cast_flashback` sets `cast_via_flashback = true`; the
        // resolver consults that flag (it used to overload `kicked`,
        // which collided with cards that have both Kicker and Flashback).
        if card.cast_via_flashback {
            self.exile.push(card);
            return Ok(events);
        }
        // CR 701.x — "Then exile this spell" rider. Cards with
        // `exile_on_resolve = true` route to exile after resolution
        // instead of their owner's graveyard. Used by Awaken the Ages,
        // Divergent Equation, Settle the Score's printed rider.
        // Bump the owner's `cards_exiled_this_turn` so the Ennis-style
        // "cards put into exile this turn" payoffs see the exile.
        if card.definition.exile_on_resolve {
            self.players[caster].cards_exiled_this_turn =
                self.players[caster].cards_exiled_this_turn.saturating_add(1);
            self.exile.push(card);
            return Ok(events);
        }
        // Buyback (CR 702.27e): a spell cast paying its buyback cost returns
        // to its owner's hand instead of the graveyard as it resolves.
        if card.bought_back {
            let owner = card.owner;
            self.players[owner].hand.push(card);
            return Ok(events);
        }
        self.players[caster].send_to_graveyard(card);
        Ok(events)
    }

    /// Resolve a triggered ability's effect tree.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn continue_trigger_resolution(
        &mut self,
        source: CardId,
        controller: usize,
        effect: crate::effect::Effect,
        target: Option<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.continue_trigger_resolution_with_source(
            source, controller, effect, target, mode, x_value, converged_value, mana_spent, None,
            0,
        )
    }

    /// Variant of `continue_trigger_resolution` that carries the
    /// trigger's "source entity" (the just-cast spell, the dying
    /// creature, etc.) into `ctx.trigger_source`. Used by spell-cast
    /// triggers whose body looks up the cast spell on the stack
    /// (e.g. Aziza's Magecraft copy, Conciliator's Duelist's Repartee
    /// exile-target). When `trigger_source_ent` is `None`, falls back
    /// to the legacy behavior (trigger_source = source permanent).
    ///
    /// `event_amount` carries the firing event's amount (life gained,
    /// life lost, damage dealt, …) so trigger bodies can read it via
    /// `Value::TriggerEventAmount` — used by Light of Promise's
    /// "Whenever you gain life, put that many +1/+1 counters …".
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn continue_trigger_resolution_with_source(
        &mut self,
        source: CardId,
        controller: usize,
        effect: crate::effect::Effect,
        target: Option<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
        trigger_source_ent: Option<crate::game::effects::EntityRef>,
        event_amount: u32,
    ) -> Result<Vec<GameEvent>, GameError> {
        // If the trigger has a stored target that's no longer legal (e.g.
        // an Elesh-Norn-doubled Solitude ETB whose first target was just
        // exiled by the prior copy), re-pick a fresh target on resolution.
        let resolved_target = match target.as_ref() {
            Some(t) => match effect.target_filter_for_slot(0) {
                Some(filter) if !self.evaluate_requirement_static(filter, t, controller, Some(source)) => {
                    self.auto_target_for_effect(&effect, controller)
                }
                _ => Some(t.clone()),
            },
            None => None,
        };
        let mut ctx =
            EffectContext::for_trigger(source, controller, resolved_target.clone(), mode);
        ctx.x_value = x_value;
        ctx.converged_value = converged_value;
        // CR 702.32 — an ETB/other trigger on a permanent reads the
        // source's `kicked` flag so "when ~ enters, if it was kicked, …"
        // riders (Goblin Bushwhacker) can branch on `SpellWasKicked`.
        if let Some(src) = self.battlefield.iter().find(|c| c.id == source) {
            ctx.kicked = src.kicked;
        }
        if let Some(ts) = trigger_source_ent {
            ctx.trigger_source = Some(ts);
        }
        ctx.mana_spent = mana_spent;
        ctx.event_amount = event_amount;
        let events = self.resolve_effect(&effect, &ctx)?;
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Trigger {
                    source,
                    controller,
                    target,
                    mode,
                    in_progress,
                    remaining,
                    x_value,
                    converged_value,
                    mana_spent,
                },
            });
        }
        Ok(events)
    }

    /// Resolve an activated ability's effect tree.
    pub(crate) fn continue_ability_resolution(
        &mut self,
        source: CardId,
        controller: usize,
        effect: crate::effect::Effect,
        target: Option<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let ctx = EffectContext::for_ability(source, controller, target.clone());
        let events = self.resolve_effect(&effect, &ctx)?;
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Ability {
                    source,
                    controller,
                    target,
                    in_progress,
                    remaining,
                },
            });
        }
        Ok(events)
    }

    /// Evaluate whether `target` satisfies `req` given the current game state.
    ///
    /// `controller` is the player who controls the spell or ability (used for
    /// `ControlledByYou` / `ControlledByOpponent` checks).
    pub fn evaluate_requirement(
        &self,
        req: &SelectionRequirement,
        target: &Target,
        controller: usize,
    ) -> bool {
        self.evaluate_requirement_static(req, target, controller, None)
    }

    pub(crate) fn battlefield_find(&self, id: CardId) -> Option<&CardInstance> {
        self.battlefield.iter().find(|c| c.id == id)
    }

    pub(crate) fn battlefield_find_mut(&mut self, id: CardId) -> Option<&mut CardInstance> {
        self.battlefield.iter_mut().find(|c| c.id == id)
    }

    /// Look up a card instance by id across every visible zone in
    /// resolution order — battlefield → each player's graveyard / hand /
    /// library → exile → stack. General-purpose helper for predicates
    /// or effects that need to introspect a card regardless of where
    /// it currently lives. Currently surfaced for the test suite
    /// (`#[allow(dead_code)]` keeps it warning-free until callers land).
    #[allow(dead_code)]
    pub(crate) fn find_card_anywhere(&self, id: CardId) -> Option<&CardInstance> {
        if let Some(c) = self.battlefield_find(id) {
            return Some(c);
        }
        for p in &self.players {
            if let Some(c) = p.graveyard.iter().find(|c| c.id == id) {
                return Some(c);
            }
            if let Some(c) = p.hand.iter().find(|c| c.id == id) {
                return Some(c);
            }
            if let Some(c) = p.library.iter().find(|c| c.id == id) {
                return Some(c);
            }
        }
        if let Some(c) = self.exile.iter().find(|c| c.id == id) {
            return Some(c);
        }
        for si in &self.stack {
            if let crate::game::types::StackItem::Spell { card, .. } = si
                && card.id == id
            {
                return Some(card);
            }
        }
        None
    }

    /// Mutable variant of `find_card_anywhere` — walks battlefield,
    /// each player's hand/library/graveyard, and exile (in that order).
    /// Used by `Effect::GrantMayPlay` to stamp `may_play_until` on a
    /// card regardless of where the granting effect happens to find it.
    pub(crate) fn find_card_anywhere_mut(
        &mut self,
        id: CardId,
    ) -> Option<&mut CardInstance> {
        if self.battlefield.iter().any(|c| c.id == id) {
            return self.battlefield.iter_mut().find(|c| c.id == id);
        }
        for p in &mut self.players {
            if let Some(c) = p.hand.iter_mut().find(|c| c.id == id) {
                return Some(c);
            }
            if let Some(c) = p.graveyard.iter_mut().find(|c| c.id == id) {
                return Some(c);
            }
            if let Some(c) = p.library.iter_mut().find(|c| c.id == id) {
                return Some(c);
            }
        }
        if let Some(c) = self.exile.iter_mut().find(|c| c.id == id) {
            return Some(c);
        }
        // A spell resolving its own effect (Spoils of the Vault's NameCard)
        // is still on the stack.
        for si in &mut self.stack {
            if let crate::game::types::StackItem::Spell { card, .. } = si
                && card.id == id
            {
                return Some(card);
            }
        }
        None
    }

    /// Look up which zone a card currently occupies. Returns `None` if
    /// the card isn't in any visible zone (battlefield, hand, library,
    /// graveyard, exile, stack). Used by the cast-from-zone path to
    /// confirm the card is still in the expected zone before lifting it.
    pub(crate) fn find_card_zone(&self, id: CardId) -> Option<crate::card::Zone> {
        use crate::card::Zone;
        if self.battlefield.iter().any(|c| c.id == id) {
            return Some(Zone::Battlefield);
        }
        for p in &self.players {
            if p.hand.iter().any(|c| c.id == id) {
                return Some(Zone::Hand);
            }
            if p.graveyard.iter().any(|c| c.id == id) {
                return Some(Zone::Graveyard);
            }
            if p.library.iter().any(|c| c.id == id) {
                return Some(Zone::Library);
            }
        }
        if self.exile.iter().any(|c| c.id == id) {
            return Some(Zone::Exile);
        }
        None
    }

    /// Look up the owner (seat index) of `id` across every public zone:
    /// battlefield, each player's graveyard, each player's hand, the
    /// stack, and exile. Returns `None` if no card with that id exists
    /// in any visible zone. Used by `PlayerRef::OwnerOf(...)` resolution
    /// to find the original owner of a target whose card has changed
    /// zones (e.g. destroyed and now in graveyard) by the time the
    /// owner-targeted effect resolves.
    pub(crate) fn find_card_owner(&self, id: CardId) -> Option<usize> {
        if let Some(c) = self.battlefield_find(id) {
            return Some(c.owner);
        }
        for (i, p) in self.players.iter().enumerate() {
            if p.graveyard.iter().any(|c| c.id == id)
                || p.hand.iter().any(|c| c.id == id)
                || p.library.iter().any(|c| c.id == id)
            {
                return Some(i);
            }
        }
        if self.exile.iter().any(|c| c.id == id) {
            return self.exile.iter().find(|c| c.id == id).map(|c| c.owner);
        }
        // Stack: a spell mid-resolution is on the stack but not yet in any
        // player's persistent zone. The spell's caster is its current
        // controller; `card.owner` is the printed owner (typically equal to
        // the caster, except for stolen spells like Wandering Archaic
        // copies). Cards on the stack via StackItem::Spell are findable here.
        for item in &self.stack {
            if let crate::game::types::StackItem::Spell { card, .. } = item
                && card.id == id
            {
                return Some(card.owner);
            }
        }
        None
    }

    /// Look up the caster (current controller) of a stack-resident spell
    /// by card id. Used by `PlayerRef::ControllerOf` to resolve "this
    /// spell's controller" — distinct from `find_card_owner`, which
    /// returns the printed `owner` even on the stack. Returns `None` if
    /// `id` is not currently a spell on the stack.
    pub(crate) fn stack_caster_for_card(&self, id: CardId) -> Option<usize> {
        for item in &self.stack {
            if let crate::game::types::StackItem::Spell { card, caster, .. } = item
                && card.id == id
            {
                return Some(*caster);
            }
        }
        None
    }

    /// Returns true if the permanent `id` has `kw` after all layer effects are applied.
    /// Falls back to `false` if the permanent is not on the battlefield.
    #[cfg(test)]
    pub(crate) fn permanent_has_keyword(&self, id: CardId, kw: &Keyword) -> bool {
        self.computed_permanent(id)
            .is_some_and(|c| c.keywords.contains(kw))
    }
}

/// Whether `ev` is already handled by a hardcoded trigger site for the
/// given `spec.scope`. Dispatched triggers should skip events for which
/// the hardcoded site would already fire — but other scopes still need
/// the unified dispatcher.
///
/// Coverage of hardcoded sites:
/// - `EnterBattlefield` + `SelfSource` → `fire_self_etb_triggers`
/// - `Attacks` + `SelfSource` → `declare_attackers`
/// - `CreatureDied` + `SelfSource` → SBA-time hook in remove-to-graveyard
/// - `SpellCast` (any scope) → `collect_self_cast_triggers` (SelfSource)
///   plus `fire_spell_cast_triggers` (YourControl/AnyPlayer)
/// - `StepBegins` (any scope) → `fire_step_triggers`
///
/// Non-SelfSource scopes for ETB / Attacks / CreatureDied are NOT covered
/// by a hardcoded site and need the unified dispatcher (Temur Ascendancy's
/// "another creature you control enters" trigger, etc.).
fn is_event_hardcoded(ev: &GameEvent, spec: &crate::effect::EventSpec) -> bool {
    use crate::effect::EventScope;
    match ev {
        GameEvent::PermanentEntered { .. } => matches!(spec.scope, EventScope::SelfSource),
        GameEvent::AttackerDeclared(_) => matches!(spec.scope, EventScope::SelfSource),
        GameEvent::CreatureDied { .. } => matches!(spec.scope, EventScope::SelfSource),
        GameEvent::SpellCast { .. } => true,
        GameEvent::StepChanged(_) => true,
        _ => false,
    }
}

/// Extract the per-event scalar amount carried by `event` — the life
/// gained on a `LifeGained`, life lost on a `LifeLost`, the count of
/// cards milled / drawn, etc. Threaded into `EffectContext.event_amount`
/// via the trigger dispatcher so trigger bodies can read it via
/// `Value::TriggerEventAmount`. Returns 0 for events that don't carry
/// a scalar amount (CreatureDied, PermanentEntered, …).
fn event_amount(event: &GameEvent) -> u32 {
    match event {
        GameEvent::LifeGained { amount, .. }
        | GameEvent::LifeLost { amount, .. }
        | GameEvent::DamageDealt { amount, .. }
        | GameEvent::PoisonAdded { amount, .. } => *amount,
        GameEvent::CounterAdded { count, .. } => *count,
        _ => 0,
    }
}

/// Characteristic-defining dynamic P/T table (CR 604.x). Maps a card
/// name to the formula the layer system should use to set its printed
/// P/T every recompute. Adding a new dynamic-P/T card is one row here;
/// formula variants live in `card::DynamicPt`.
///
/// Current entries:
/// - Tarmogoyf (MOR): P=N, T=N+1 where N = distinct card types in all
///   graveyards.
/// - Cosmogoyf (modern reprint of the same mechanic): same formula.
/// - Cruel Somnophage (MOM): P=T = controller's graveyard size.
fn dynamic_pt_for_name(name: &'static str) -> Option<crate::card::DynamicPt> {
    use crate::card::DynamicPt;
    match name {
        "Tarmogoyf" | "Cosmogoyf" => Some(DynamicPt::DistinctTypesInAllGraveyards),
        "Cruel Somnophage" => Some(DynamicPt::ControllerGraveyardSize),
        "Knight of the Reliquary" => Some(DynamicPt::BasePlusLandsInAllGraveyards {
            base_p: 2, base_t: 2,
        }),
        "Wight of the Reliquary" => Some(DynamicPt::BasePlusLandsInControllerGraveyard {
            base_p: 1, base_t: 1,
        }),
        _ => None,
    }
}

/// Compute-time conditional self-pump table: cards whose printed Oracle
/// is "As long as you've gained life this turn, this creature gets +P/+T
/// [and gains keyword(s)]." The pump and keyword grants are emitted as
/// short-lived continuous effects (P/T at layer 7b, keyword grants at
/// layer 6) every `compute_battlefield` pass when the controller's
/// `life_gained_this_turn` tally is non-zero.
///
/// Returns `Some((power_bump, toughness_bump, keywords))` if `name`
/// matches a known lifegain-self-pump card, else `None`. Adding a new
/// such card requires appending one row here instead of a new `if name
/// == "..."` branch in `compute_battlefield`.
///
/// Current entries:
/// - Honor Troll (STX): +2/+0 and Lifelink
/// - Ulna Alley Shopkeep (SOS Infusion): +2/+0 (no keyword)
/// Threshold-style self-pump table: cards reading "As long as there are N
/// or more cards in your graveyard, this creature gets +P/+T." Returns
/// `(graveyard_size_threshold, +power, +toughness)`. The gate is the
/// controller's graveyard size, re-evaluated every `compute_battlefield`.
///
/// Current entries:
/// - Elvish Reclaimer (MH1): +2/+2 (→ 3/4) with seven or more cards in
///   your graveyard.
fn graveyard_threshold_selfpump_for_name(name: &'static str) -> Option<(usize, i32, i32)> {
    match name {
        "Elvish Reclaimer" => Some((7, 2, 2)),
        _ => None,
    }
}

fn lifegain_selfpump_for_name(
    name: &'static str,
) -> Option<(i32, i32, &'static [crate::card::Keyword])> {
    use crate::card::Keyword;
    static HONOR_TROLL_KWS: &[Keyword] = &[Keyword::Lifelink];
    static NO_KWS: &[Keyword] = &[];
    match name {
        "Honor Troll" => Some((2, 0, HONOR_TROLL_KWS)),
        "Ulna Alley Shopkeep" => Some((2, 0, NO_KWS)),
        "Tenured Concocter" => Some((2, 0, NO_KWS)),
        _ => None,
    }
}

/// Graveyard-resident static-ability table: cards whose printed
/// Oracle is "As long as [this card] is in your graveyard and you
/// control a [Land subtype], creatures you control have [keyword]."
/// This is the Judgment Incarnation cycle (Anger / Wonder / Filth /
/// Brawn / Genesis / Valor) — STA reprinted a subset of these. The
/// engine walks each graveyard during `compute_battlefield` and
/// applies a continuous `AddKeyword` effect to the controller's
/// creatures when the gate (land subtype controlled) is met.
///
/// Returns `Some((land_subtype, keyword))` if `name` matches a known
/// gy-anthem Incarnation, else `None`.
///
/// Current entries:
/// - Anger (STA reprint, Judgment): controls Mountain → Haste anthem
/// - Wonder (STA reprint, Judgment): controls Island → Flying anthem
/// - Brawn (STA reprint, Judgment): controls Forest → Trample anthem
/// - Valor (STA reprint, Judgment): controls Plains → First Strike anthem
fn graveyard_anthem_for_name(
    name: &'static str,
) -> Option<(crate::card::LandType, crate::card::Keyword)> {
    use crate::card::{Keyword, LandType};
    match name {
        "Anger" => Some((LandType::Mountain, Keyword::Haste)),
        "Wonder" => Some((LandType::Island, Keyword::Flying)),
        "Brawn" => Some((LandType::Forest, Keyword::Trample)),
        "Valor" => Some((LandType::Plains, Keyword::FirstStrike)),
        "Filth" => Some((LandType::Swamp, Keyword::Landwalk(LandType::Swamp))),
        _ => None,
    }
}

/// Compute-time conditional "Infusion anthem" table: cards whose
/// printed Oracle is "Infusion — Creatures you control get +P/+T
/// [and gain keyword(s)] as long as you've gained life this turn."
/// Different from `lifegain_selfpump_for_name` in that the pump
/// applies to every creature the controller has on the battlefield
/// (including the source — matching the printed "creatures you
/// control" wording, which is inclusive). The gate evaluates the
/// controller's `life_gained_this_turn` tally every layer recompute.
///
/// Current entries:
/// - Thornfist Striker (SOS): +1/+0 and Trample
fn lifegain_anthem_for_name(
    name: &'static str,
) -> Option<(i32, i32, &'static [crate::card::Keyword])> {
    use crate::card::Keyword;
    static TRAMPLE_KWS: &[Keyword] = &[Keyword::Trample];
    match name {
        "Thornfist Striker" => Some((1, 0, TRAMPLE_KWS)),
        _ => None,
    }
}

/// Compute-time conditional self-counter anthem table: cards whose
/// printed Oracle is "As long as this permanent has [N] or more
/// [counter] counters on it, creatures you control get +P/+T."
/// The anthem is emitted as a short-lived continuous effect (P/T at
/// layer 7b, affecting `AffectedPermanents::All { controller: Some
/// (source.controller), card_types: [Creature], exclude_source: false
/// }`) every `compute_battlefield` pass when the source's own
/// counter pool meets the threshold. Adding a new such card requires
/// appending one row here instead of a new `if name == "..."` branch.
///
/// Returns `Some((threshold, counter_kind, power_bump, toughness_bump))`
/// if `name` matches a known counter-gated anthem card, else `None`.
///
/// Current entries:
/// - Comforting Counsel (SOS): ≥5 Growth → +3/+3 to your creatures
fn self_counter_anthem_for_name(
    name: &'static str,
) -> Option<(u32, crate::card::CounterType, i32, i32)> {
    use crate::card::CounterType;
    match name {
        "Comforting Counsel" => Some((5, CounterType::Growth, 3, 3)),
        _ => None,
    }
}

// ── Static ability conversion ─────────────────────────────────────────────────

/// Convert a `StaticAbility` from a source permanent into `ContinuousEffect`s.
/// Takes the full `CardInstance` so Equipment/Aura abilities can use `attached_to`.
fn static_ability_to_effects(card: &CardInstance, timestamp: u64) -> Vec<ContinuousEffect> {
    use crate::effect::StaticEffect;
    let source = card.id;

    card.definition
        .static_abilities
        .iter()
        .flat_map(|sa| match &sa.effect {
            StaticEffect::PumpPT { applies_to, power, toughness } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(*power, *toughness),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::GrantKeyword { applies_to, keyword } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(keyword.clone()),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::EntersTapped { .. }
            | StaticEffect::ExtraLandPerTurn
            | StaticEffect::CostReduction { .. }
            | StaticEffect::CostReductionTargetingFilter { .. }
            | StaticEffect::AdditionalCostAfterFirstSpell { .. }
            | StaticEffect::AdditionalCost { .. }
            | StaticEffect::ControllerHasHexproof
            | StaticEffect::LandsTapColorlessOnly
            // ArtifactActivatedAbilitiesLocked — consulted in
            // `activate_ability` (Collector Ouphe); no layer effect.
            | StaticEffect::ArtifactActivatedAbilitiesLocked
            // Teferi statics — handled at cast time via dedicated checks
            // (`player_locked_to_sorcery_timing` etc.); not modeled as
            // continuous-layer modifications here.
            | StaticEffect::OpponentsSorceryTimingOnly
            | StaticEffect::ControllerSorceriesAsFlash
            // DoubleTokens — read at `Effect::CreateToken` resolution time
            // via `GameState::token_doublers_for(seat)`; no layer effect.
            | StaticEffect::DoubleTokens
            // DoubleCounters — read at `Effect::AddCounter` resolution time
            // via `GameState::counter_doublers_for(seat)`; no layer effect.
            | StaticEffect::DoubleCounters
            // DoubleDamageDealt — read at non-combat damage time via
            // `GameState::damage_doublers`; no layer effect.
            | StaticEffect::DoubleDamageDealt
            // GrantAffinityToISSpells — read at cast time by
            // `cost_reduction_for_spell` directly; no layer effect.
            | StaticEffect::GrantAffinityToISSpells { .. }
            // ExtraEtbCountersForCreatureCasts — read at creature-spell
            // resolution time in `stack.rs::resolve_spell`; no layer effect.
            | StaticEffect::ExtraEtbCountersForCreatureCasts { .. }
            // EtbTriggerSpotlight / DoubleControllerEtbTriggers — read at ETB
            // trigger dispatch via `etb_trigger_multiplier`; no layer effect.
            | StaticEffect::EtbTriggerSpotlight
            | StaticEffect::DoubleControllerEtbTriggers
            // UncounterableCreaturesOfChosenType — read at cast time by
            // `caster_grants_uncounterable_with_x`; no layer effect.
            | StaticEffect::UncounterableCreaturesOfChosenType
            // EtbTriggerTax — read at ETB trigger push time by
            // `apply_etb_trigger_tax` (Strict Proctor); no layer effect.
            | StaticEffect::EtbTriggerTax { .. }
            // PlayerCannotGainLife — projected onto Player.cannot_gain_life
            // each recompute by apply_player_statics; no layer effect.
            | StaticEffect::PlayerCannotGainLife { .. }
            // PlayerCannotLoseLife — consulted dynamically by adjust_life /
            // damage paths via player_cannot_lose_life_now; no layer effect.
            | StaticEffect::PlayerCannotLoseLife { .. }
            // CapDrawsPerTurn — consulted at draw time via draw_cap_for; no
            // layer effect.
            | StaticEffect::CapDrawsPerTurn { .. }
            // PreventUntap — consulted by `do_untap` (CR 502.3); no layer
            // effect since it gates a turn-based action rather than a
            // characteristic.
            | StaticEffect::PreventUntap { .. }
            // SpellCostFloor (Trinisphere) — read at cast time by
            // `apply_spell_cost_floor`; no layer effect.
            | StaticEffect::SpellCostFloor { .. }
            // CastHandSpellsFree (Omniscience) — read by the free-cast
            // action via `player_casts_hand_spells_free`; no layer effect.
            | StaticEffect::CastHandSpellsFree
            // GrantKeywordToAttackers — needs live combat state, resolved in
            // `compute_battlefield` against `GameState.attacking`.
            | StaticEffect::GrantKeywordToAttackers { .. }
            // GrantActivatedAbility — surfaced as a virtual activated ability
            // in `activate_ability`; not a characteristic layer effect.
            | StaticEffect::GrantActivatedAbility { .. }
            // NotCreatureWhileDevotionBelow — needs live devotion count,
            // resolved in `gather_continuous_effects` against the GameState.
            | StaticEffect::NotCreatureWhileDevotionBelow { .. }
            // PumpSelfByControlledPermanents — needs a live battlefield
            // count; resolved in `gather_continuous_effects`.
            | StaticEffect::PumpSelfByControlledPermanents { .. }
            // ExileNontokenCreaturesNotCast (Containment Priest) — read at
            // battlefield-entry time by `nontoken_creature_etb_exile_active`;
            // no layer effect.
            | StaticEffect::ExileNontokenCreaturesNotCast => vec![],
        })
        .collect()
}

/// Translate a selector into a `layers::AffectedPermanents` description for
/// those `StaticEffect` variants that express broad "lord-like" scope. Returns
/// `None` if the selector shape isn't representable in the layer system yet.
fn selector_to_affected(
    sel: &crate::effect::Selector,
    card: &CardInstance,
) -> Option<AffectedPermanents> {
    use crate::effect::Selector;
    let controller = card.controller;
    match sel {
        Selector::This => Some(AffectedPermanents::Source),
        Selector::AttachedTo(inner) => {
            if matches!(inner.as_ref(), Selector::This)
                && let Some(attached_id) = card.attached_to
            {
                Some(AffectedPermanents::Specific(vec![attached_id]))
            } else {
                None
            }
        }
        Selector::EachPermanent(req) => affected_from_requirement(req, controller),
        _ => None,
    }
}

fn affected_from_requirement(
    req: &SelectionRequirement,
    source_controller: usize,
) -> Option<AffectedPermanents> {
    use SelectionRequirement as R;
    // Decompose And-trees to extract controller filter + card-type filter.
    let mut ctrl: Option<Option<usize>> = None; // Outer Some(None) = all players; Some(Some(n)) = specific player
    let mut types: Vec<CardType> = vec![];
    let mut creature_type: Option<crate::card::CreatureType> = None;
    let mut counter_filter: Option<crate::card::CounterType> = None;
    // CR-driven "other" exclusion (push XXXV). `SelectionRequirement::
    // OtherThanSource` flips this to true; the resulting AffectedPermanents
    // variant carries `exclude_source: true` so the layer-time `affects()`
    // check skips the source permanent itself — matching printed "**other**
    // [type] you control" wording.
    let mut other_than_source = false;
    let mut walk = vec![req];
    while let Some(r) = walk.pop() {
        match r {
            R::And(a, b) => {
                walk.push(a);
                walk.push(b);
            }
            R::ControlledByYou => ctrl = Some(Some(source_controller)),
            R::ControlledByOpponent => {
                return Some(AffectedPermanents::AllOpponents {
                    source_controller,
                    card_types: if types.is_empty() { vec![] } else { types.clone() },
                    // Populated by `compute_battlefield` once the source's
                    // team is known (this helper has no GameState handle).
                    friendly_seats: Vec::new(),
                });
            }
            R::Creature => types.push(CardType::Creature),
            R::Artifact => types.push(CardType::Artifact),
            R::Enchantment => types.push(CardType::Enchantment),
            R::Planeswalker => types.push(CardType::Planeswalker),
            R::Land => types.push(CardType::Land),
            R::HasCardType(t) => types.push(t.clone()),
            R::HasCreatureType(ct) => creature_type = Some(*ct),
            R::WithCounter(ct) => counter_filter = Some(*ct),
            R::OtherThanSource => other_than_source = true,
            R::Any | R::Permanent => {}
            _ => return None,
        }
    }
    if let Some(counter) = counter_filter {
        return Some(AffectedPermanents::AllWithCounter {
            controller: ctrl.flatten(),
            card_types: types,
            counter,
            at_least: 1,
        });
    }
    if let Some(ct) = creature_type {
        return Some(AffectedPermanents::AllWithCreatureType {
            controller: ctrl.flatten(),
            creature_type: ct,
            exclude_source: other_than_source,
        });
    }
    Some(AffectedPermanents::All {
        controller: ctrl.unwrap_or(None),
        card_types: types,
        exclude_source: other_than_source,
    })
}


// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns true if `blocker` is legally allowed to block `attacker`.
/// Uses `blocker_kws` / `attacker_kws` as the effective keyword sets
/// (from `ComputedPermanent`) instead of the raw definition keywords.
pub(crate) fn can_block_attacker_computed(
    blocker: &CardInstance,
    attacker: &CardInstance,
    blocker_computed: &ComputedPermanent,
    attacker_kws: &[Keyword],
    attacker_colors: &[crate::mana::Color],
) -> bool {
    let blocker_kws = &blocker_computed.keywords;
    // Unblockable: can't be blocked at all.
    if attacker_kws.contains(&Keyword::Unblockable) {
        return false;
    }
    // Flying: can only be blocked by fliers or reach.
    if attacker_kws.contains(&Keyword::Flying)
        && !blocker_kws.contains(&Keyword::Flying)
        && !blocker_kws.contains(&Keyword::Reach)
    {
        return false;
    }
    // Horsemanship: can only be blocked by other Horsemanship creatures.
    if attacker_kws.contains(&Keyword::Horsemanship)
        && !blocker_kws.contains(&Keyword::Horsemanship)
    {
        return false;
    }
    // Shadow: can only block/be blocked by other shadow creatures.
    if attacker_kws.contains(&Keyword::Shadow) && !blocker_kws.contains(&Keyword::Shadow) {
        return false;
    }
    if blocker_kws.contains(&Keyword::Shadow) && !attacker_kws.contains(&Keyword::Shadow) {
        return false;
    }
    // Skulk: can't be blocked by creatures with greater power.
    if attacker_kws.contains(&Keyword::Skulk) && blocker_computed.power > attacker.power() {
        return false;
    }
    // Fear (CR 702.36): can only be blocked by artifact creatures and/or
    // black creatures.
    if attacker_kws.contains(&Keyword::Fear) {
        let blocker_is_artifact = blocker.definition.is_artifact();
        let blocker_is_black = blocker_computed.colors.contains(&crate::mana::Color::Black);
        if !blocker_is_artifact && !blocker_is_black {
            return false;
        }
    }
    // Intimidate (CR 702.13): can only be blocked by artifact creatures
    // or creatures that share a color with the attacker. We compare the
    // attacker's *computed* colors (which include hybrid / mono-hybrid
    // pips and color-setting effects, via `ComputedPermanent.colors`)
    // against the blocker's computed colors — not raw `{C}` cost pips.
    if attacker_kws.contains(&Keyword::Intimidate) {
        let blocker_is_artifact = blocker.definition.is_artifact();
        let shares_color = blocker_computed
            .colors
            .iter()
            .any(|c| attacker_colors.contains(c));
        if !blocker_is_artifact && !shares_color {
            return false;
        }
    }
    // Protection from a color (CR 702.16e): the attacker can't be blocked
    // by a creature of a color it has protection from. Read the blocker's
    // computed colors so hybrid-pip and effect-granted colors count.
    for kw in attacker_kws {
        if let Keyword::Protection(color) = kw
            && blocker_computed.colors.contains(color)
        {
            return false;
        }
    }
    true
}
