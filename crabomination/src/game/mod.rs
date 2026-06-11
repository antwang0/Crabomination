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
pub(crate) mod affordances;
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
#[cfg(test)]
#[path = "../tests/akh.rs"]
mod tests_akh;
#[cfg(test)]
#[path = "../tests/mkm.rs"]
mod tests_mkm;
#[cfg(test)]
#[path = "../tests/ogw.rs"]
mod tests_ogw;
#[cfg(test)]
#[path = "../tests/cr_rules.rs"]
mod tests_cr_rules;
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

/// Every from-hand affordance hint for one seat, produced in a single sweep
/// by [`GameState::compute_hand_affordances`]. Each field is the set of
/// CardIds the client should highlight for that affordance; the view layer
/// copies them straight into the matching `ClientView` fields. All-empty
/// when the seat doesn't currently hold priority.
#[derive(Debug, Clone, Default)]
pub struct HandAffordances {
    pub castable: Vec<CardId>,
    pub pitchable: Vec<CardId>,
    pub kickable: Vec<CardId>,
    pub buyback: Vec<CardId>,
    pub bestowable: Vec<CardId>,
    pub dashable: Vec<CardId>,
    pub blitzable: Vec<CardId>,
    pub suspendable: Vec<CardId>,
    pub foretellable: Vec<CardId>,
    pub plottable: Vec<CardId>,
    pub adventurable: Vec<CardId>,
    /// CR 709 — split cards whose **right** half is castable right now.
    pub splittable_right: Vec<CardId>,
    /// CR 702.176 — hand cards with Bargain that are castable right now, so the
    /// client can offer a "sacrifice an artifact/enchantment/token?" toggle.
    pub bargainable: Vec<CardId>,
    /// CR 702.157 — hand cards with Squad castable paying the squad cost at
    /// least once, so the client can offer a "pay Squad N times?" stepper.
    pub squadable: Vec<CardId>,
    /// CR 702.107 — hand cards with Replicate castable paying the replicate
    /// cost at least once, so the client can offer a "replicate N times?" stepper.
    pub replicatable: Vec<CardId>,
    /// CR 702.33c — hand cards with Multikicker castable paying the kicker
    /// cost at least once, so the client can offer a "kick N times?" stepper.
    pub multikickable: Vec<CardId>,
    /// CR 702.94 — hand cards with a live Miracle window (revealed as the
    /// turn's first draw): castable for the cheaper miracle cost via
    /// `GameAction::CastFromZoneWithoutPaying`.
    pub miracle: Vec<CardId>,
    pub activatable_permanents: Vec<CardId>,
    /// Hand cards carrying at least one `from_hand` activated ability (Talon
    /// Gates of Madara's `{4}: put this onto the battlefield`, the Spirit
    /// Guides' exile-for-mana). Surfaced so the client/bot can offer the
    /// from-hand activation; affordability is re-checked by `activate_ability`.
    pub hand_activatable: Vec<CardId>,
    /// CR 702.36 — hand cards with Morph/Megamorph/Disguise castable face down
    /// for {3} right now, so the client can offer the "cast face down" action.
    pub morphable: Vec<CardId>,
    /// CR 708.5 — face-down permanents the seat controls whose turn-up cost
    /// (Morph/Megamorph/Disguise cost, or a manifested/cloaked creature card's
    /// mana cost) is payable right now, so the client can offer "turn face up".
    pub turn_up_able: Vec<CardId>,
    /// CR 702.77 — hand cards with a Reinforce ability whose cost is payable
    /// right now (a legal creature target exists), so the client can offer the
    /// from-hand Reinforce activation.
    pub reinforceable: Vec<CardId>,
    /// CR 709.5 — Room hand cards whose left/right door is castable right
    /// now (`(card, door)` — door 0 = left, 1 = right).
    pub room_castable: Vec<(CardId, u8)>,
    /// CR 709.5e — Room permanents the seat controls with a locked door
    /// whose unlock cost is payable right now (`(card, door)`).
    pub room_unlockable: Vec<(CardId, u8)>,
}

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
    /// CR 702.26 — permanents that have phased out. They're treated as though
    /// they don't exist (every battlefield query iterates `battlefield`, so a
    /// phased-out permanent is invisible without per-site filtering), yet
    /// retain all state (counters, attachments, damage). They phase back in
    /// during their controller's untap step (`do_phasing`).
    #[serde(default)]
    pub phased_out: Vec<CardInstance>,
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
    /// CR 510.1c — the active player's chosen blocker order for each attacker
    /// that has multiple blockers, gathered (and cached) before combat damage
    /// is applied so the choice can suspend for a `wants_ui` player. Read by
    /// `resolve_combat_damage_with_filter`; reset per damage step and cleared
    /// at combat end. `#[serde(skip)]` — transient, like `pending_decision`'s
    /// resume context (a mid-combat snapshot can't resume anyway).
    #[serde(skip, default)]
    pub(crate) combat_damage_order: HashMap<CardId, Vec<CardId>>,
    /// CR 510.1c-d — the active player's chosen combat-damage assignment
    /// `(blocker, amount)` for each multi-blocker attacker. Gathered alongside
    /// `combat_damage_order`. `#[serde(skip)]` for the same reason.
    #[serde(skip, default)]
    pub(crate) combat_damage_assignment: HashMap<CardId, Vec<(CardId, u32)>>,
    /// Which damage step (`FirstStrikeDamage` / `CombatDamage`) the cached
    /// combat-damage choices above belong to. Lets the gather pass reset the
    /// caches once when moving from the first-strike step to the regular step,
    /// without wiping them on a mid-step decision resume.
    #[serde(skip, default)]
    pub(crate) combat_damage_plan_step: Option<TurnStep>,
    /// Set to true once `declare_blockers` has been called during the current DeclareBlockers step.
    pub(crate) blockers_declared: bool,
    /// Skip the draw on the very first turn (turn 1, first player).
    pub(crate) skip_first_draw: bool,
    /// Count of spells cast this turn (for Storm and related effects).
    pub spells_cast_this_turn: u32,
    /// CR 700.14 — running total of mana the active player has spent to
    /// cast spells this turn (Expend). Bumped in `finalize_cast` by each
    /// spell's `mana_spent`; reset at cleanup. `#[serde(default)]`.
    #[serde(default)]
    pub(crate) mana_spent_on_spells_this_turn: u32,
    /// CR 700.14 — the spell-mana total *before* the cost that produced
    /// the `Expended` event currently being dispatched. Read by
    /// `Predicate::ExpendReached` to detect threshold crossings.
    /// Transient scratch — `#[serde(skip)]`.
    #[serde(skip)]
    pub(crate) expend_prev_total: u32,
    /// Total spells cast during the previous turn (snapshotted from
    /// `spells_cast_this_turn` at Cleanup). Drives the classic Innistrad
    /// werewolf transform check ("if no spells were cast last turn …").
    /// `#[serde(default)]` so older snapshots load as 0.
    #[serde(default)]
    pub spells_cast_last_turn: u32,
    /// CR 702.69 — count of permanents put into a graveyard from the
    /// battlefield this turn (any controller, any type). Drives Gravestorm
    /// copy counts; reset at each turn's untap step.
    #[serde(default)]
    pub permanents_to_graveyard_this_turn: u32,
    /// Cards that entered the battlefield from a graveyard — or were cast
    /// from one — this turn. Stamped at the gy→battlefield move funnel and
    /// at every cast-from-graveyard site; read by
    /// `SelectionRequirement::EnteredFromGraveyardThisTurn` (Prized
    /// Amalgam's gate). Cleared at each turn's untap step.
    #[serde(default)]
    pub(crate) entered_from_graveyard_this_turn: std::collections::HashSet<CardId>,
    /// Delayed triggered abilities registered by resolved spells/abilities
    /// (Pact upkeep cost, Goryo's exile-at-EOT, etc.). Fired by the step
    /// dispatcher when the matching event occurs.
    pub delayed_triggers: Vec<DelayedTrigger>,
    /// Tokens minted by `Effect::CreateTokenAttacking` with a non-`None`
    /// cleanup (Mobilize sacrifice / Myriad exile). Drained when the combat
    /// phase ends (CR 511.3).
    #[serde(default)]
    pub(crate) attacking_token_cleanup: Vec<(CardId, crate::effect::AttackingTokenCleanup)>,
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
    /// Transient: mana value of the most-recently-sacrificed creature within
    /// the current effect/cost resolution. Set alongside `sacrificed_power`
    /// (including the `sac_other_filter` activation-cost path); read by
    /// `SelectionRequirement::ManaValueEqualsSacrificedPlus` (Birthing Pod).
    /// Reset between independent resolutions.
    #[serde(default)]
    pub(crate) sacrificed_mana_value: Option<u32>,
    /// Transient: the firing event's amount for the trigger currently being
    /// targeted or resolved (stamped in `drain_trigger_queue` and
    /// `continue_trigger_resolution_with_source`). For died events this is
    /// the dying card's mana value, read by
    /// `SelectionRequirement::ManaValueLessThanEventAmount` (Scrap Trawler).
    #[serde(default)]
    pub(crate) trigger_event_amount_scratch: u32,
    /// Transient: id of the most-recently-created token within the current
    /// effect resolution. Set by `Effect::CreateToken` and read by
    /// `Selector::LastCreatedToken` so a follow-up `AddCounter` /
    /// `PumpPT` / etc. in the same `Effect::Seq` can target the freshly
    /// minted token (Fractal Anomaly, Applied Geometry). Reset between
    /// independent resolutions.
    #[serde(skip)]
    pub(crate) last_created_token: Option<CardId>,
    /// CR 706.4 — the result of the most recent die roll, read by
    /// `Value::LastDieRoll`. Set by the `Effect::RollDie` resolver.
    #[serde(skip)]
    pub(crate) last_die_roll: u8,
    /// Transient generic cost-reduction folded into the next spell cast
    /// (CR 601.2f). Set by `cast_spell_sacrifice_reduce` to "{N} less per
    /// creature sacrificed" before delegating to the normal cast path, read
    /// in `cost_reduction_for_spell`, and cleared immediately after the cast.
    #[serde(skip)]
    pub(crate) extra_cast_reduction: u32,
    /// Transient: the cast in flight paid with Cavern-of-Souls-style
    /// restricted mana whose rider makes the spell uncounterable
    /// (`SpendRestriction::CreatureOfTypeUncounterable`). Set right after
    /// payment, consumed by `finalize_cast`.
    #[serde(skip)]
    pub(crate) cast_paid_uncounterable: bool,
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
    /// Transient: per-player count of *nonland* cards discarded within the
    /// current effect resolution. Read by `Predicate::DiscardedNonlandThisEffect`
    /// — Kroxa's "each opponent who didn't discard a nonland card this way
    /// loses 3 life." Reset to empty between independent resolutions.
    #[serde(skip)]
    pub(crate) nonland_cards_discarded_per_player_this_resolution:
        std::collections::HashMap<usize, u32>,
    /// Transient: set by `Effect::ShuffleSelfIntoLibrary` during spell
    /// resolution; the post-resolution routing reads it to send the
    /// resolving spell to its owner's library (shuffled) instead of the
    /// graveyard. Cleared once consumed. Beacon cycle.
    #[serde(skip)]
    pub(crate) shuffle_resolving_spell_into_library: bool,
    /// `Effect::ReturnResolvingSpellToHand` — same shape, hand-bound.
    #[serde(skip)]
    pub(crate) return_resolving_spell_to_hand: bool,
    /// `Effect::ExileResolvingSpell` — same shape, exile-bound.
    #[serde(skip)]
    pub(crate) exile_resolving_spell: bool,
    /// CR 728 — set by `Effect::EndTheTurn`; consumed after the current
    /// stack item finishes resolving (exile the stack, clear combat, jump
    /// to cleanup).
    #[serde(skip)]
    pub(crate) end_turn_requested: bool,
    /// CR 702.46 — Cipher. Set by `Effect::Cipher` to the creature the
    /// resolving spell should be exiled "encoded on"; the post-resolution
    /// routing consumes it to send the card to exile (with `encoded_on` stamped)
    /// instead of the graveyard. Cleared once consumed.
    #[serde(skip)]
    pub(crate) cipher_encode_pending: Option<CardId>,
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
    /// Transient: seats that sacrificed at least one permanent during the
    /// current resolution. Read by `Predicate::PlayerSacrificedThisResolution`
    /// so a follow-up step can gate on "if you sacrificed a permanent this way"
    /// (Deadly Brew). Reset between independent resolutions.
    #[serde(skip)]
    pub(crate) players_sacrificed_this_resolution: std::collections::HashSet<usize>,
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
    /// Transient: the permanents a `wants_ui` caster picked to satisfy a
    /// "sacrifice a permanent" additional cast cost (CR 601.2b). Set by
    /// `submit_decision`'s `CastSacrifice` resume just before it re-invokes
    /// the cast, and consumed (taken) by `pay_additional_costs` in lieu of
    /// the auto-pick. `None` for the auto-pick path (bots/tests, or a single
    /// legal choice). Never needs to survive a snapshot — it lives only
    /// across the synchronous resume → cast call.
    #[serde(skip, default)]
    pub(crate) pending_cast_sacrifices: Option<Vec<CardId>>,
    /// Transient sibling of [`pending_cast_sacrifices`] for a spell's
    /// "as an additional cost, discard a card" requirement
    /// (`AdditionalCastCost::Discard` — Big Score, Illuminate History). The
    /// cards a `wants_ui` caster picked to discard; consumed by
    /// `pay_additional_costs` in lieu of the first-N auto-pick. Never snapshots.
    #[serde(skip, default)]
    pub(crate) pending_cast_discards: Option<Vec<CardId>>,
    /// Transient: the answer to a "spend your floating mana, or tap lands
    /// instead?" confirmation (CR 601.2g — the player chooses their mana
    /// sources). `Some(true)` = spend the pre-existing float; `Some(false)` =
    /// keep it and pay from freshly-tapped sources. Set by `submit_decision`'s
    /// `CastFloatConfirm` resume just before it replays the cast, taken at the
    /// top of the cast. Never snapshots.
    #[serde(skip, default)]
    pub(crate) pending_cast_spend_float: Option<bool>,
    /// Transient: the permanent a `wants_ui` activator picked to satisfy an
    /// activated ability's "Sacrifice another …" cost (`sac_other_filter`).
    /// Set by `submit_decision`'s `ActivateAbilityChoice` resume just before it
    /// replays `activate_ability`, and consumed there in lieu of the auto-pick.
    /// `None` for the auto-pick path. Like `pending_cast_sacrifices`, it lives
    /// only across the synchronous resume → activate call and never snapshots.
    #[serde(skip, default)]
    pub(crate) pending_ability_sac_other: Option<CardId>,
    /// Transient sibling of [`pending_ability_sac_other`] for an activated
    /// ability's "Tap an untapped … you control" cost (`tap_other_filter`).
    /// Set by the `ActivateAbilityChoice` resume, consumed by `activate_ability`.
    #[serde(skip, default)]
    pub(crate) pending_ability_tap_other: Option<CardId>,
    /// Transient sibling of [`pending_ability_sac_other`] for an activated
    /// ability's "Exile N cards from your graveyard" cost (`exile_other_filter`).
    /// Carries the full chosen set (the cost can exile several — Grim
    /// Lavamancer exiles two). Set by the `ActivateAbilityChoice` resume,
    /// consumed by `activate_ability`.
    #[serde(skip, default)]
    pub(crate) pending_ability_exile_other: Option<Vec<CardId>>,
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
    /// One-shot validated answer for a resolution-time choice whose suspend
    /// re-queues the originating effect as its continuation (`ChooseN`,
    /// `Escalate`, `MayDo`, `DealDamageDivided`, `ChooseAmount` payers, and
    /// deferred trigger `ChooseMode`s). `apply_pending_effect_answer` stashes
    /// the sanitised answer here; the re-run effect `take()`s it instead of
    /// asking the decider again. Always consumed within the same
    /// `submit_decision` call, so it never crosses a serialization boundary.
    #[serde(skip, default)]
    pub(crate) stashed_resolution_answer: Option<DecisionAnswer>,
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
    /// CR 701.10f — transient mana-production doubler count for the mana
    /// ability currently resolving (Mana Reflection). Set before a tapped-
    /// for-mana ability resolves and reset to 0 after; the `AddMana` resolver
    /// multiplies each pip count by `2^doublers`. 0 = no doubling (1×).
    #[serde(default)]
    pub(crate) mana_production_doublers: u8,
    /// Identity of the spell currently resolving — (card id, caster, printed
    /// colors). Stamped around `resolve_effect` in spell resolution so
    /// source-aware damage replacements (Torbran) can read the controller and
    /// colors of a card that's in no visible zone mid-resolution. Transient.
    #[serde(skip)]
    pub(crate) resolving_source: Option<(CardId, usize, Vec<crate::mana::Color>)>,
    /// Reentrancy guard: true while `gather_continuous_effects` runs, so
    /// layer-aware type filters (`evaluate_requirement_static`) fall back to
    /// printed types instead of recursing through `computed_permanent`.
    #[serde(skip)]
    pub(crate) in_layer_gather: std::sync::atomic::AtomicBool,
    /// CR 505.1b — additional combat phases banked for the active player.
    /// `Effect::AdditionalCombatPhase` increments this; when the active
    /// player leaves the End of Combat step with it set, the turn loops back
    /// to Begin Combat (decrementing) instead of advancing to the postcombat
    /// main. Reset at cleanup so it can't bleed into the next turn.
    #[serde(default)]
    pub(crate) additional_combat_phases: u32,
    /// CR 505.1b — combat phases banked by `AdditionalCombatPhaseAfterMain`
    /// (Relentless Assault): when the active player leaves a main phase with
    /// one banked, the turn enters Begin Combat instead of the next phase
    /// (the follow-up main comes from the normal EndCombat → PostMain flow).
    /// Reset at cleanup.
    #[serde(default)]
    pub(crate) additional_post_main_combats: u32,
    /// CR 614.9 / 615 — creatures whose combat damage is prevented in both
    /// directions for the rest of the turn (Maze of Ith: "prevent all combat
    /// damage that would be dealt to and dealt by that creature"). The combat
    /// resolver skips dealing *and* receiving combat damage for any creature
    /// in this set. Cleared at cleanup.
    #[serde(default)]
    pub(crate) combat_damage_prevented_creatures: Vec<CardId>,
    /// CR 510.1c — attackers that became blocked this combat. An attacker
    /// stays blocked even if all its blockers leave combat (double-strike
    /// step-one kills, post-block removal): without trample it assigns no
    /// combat damage. Cleared when combat ends.
    #[serde(default)]
    pub(crate) blocked_attackers: Vec<CardId>,
    /// CR 614.13-style ETB-control replacement (Gather Specimens): seats
    /// whose opponents' creatures enter under their control instead this
    /// turn. Cleared at cleanup.
    #[serde(default)]
    pub(crate) creature_etb_steal_this_turn: Vec<usize>,
    /// Players who have paid the Leonin Arbiter search tax this turn
    /// (covers further searches until end of turn). Cleared at cleanup.
    #[serde(default)]
    pub(crate) search_tax_paid_this_turn: Vec<usize>,
    /// CR 615.7 — sources whose damage is prevented entirely this turn
    /// (Burrenton Forge-Tender's chosen source). Cleared at cleanup.
    #[serde(default)]
    pub(crate) damage_prevented_sources: Vec<CardId>,
    /// Per-pair "can't block" restrictions for the turn: `(blocker, attacker)`
    /// — the blocker can't block that specific attacker (Kozilek's Pathfinder's
    /// "{C}: Target creature can't block this creature this turn"). Cleared at
    /// cleanup. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) cant_block_pairs: Vec<(CardId, CardId)>,
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
    /// CR 603.10 / 608.2h — last-known-information snapshots for
    /// leaves-the-battlefield triggers that read the dying object's
    /// characteristics *as they last existed on the battlefield* (e.g.
    /// "when this dies, it deals damage equal to its power" — counters
    /// and pumps included). Keyed by the trigger's source CardId.
    /// Populated when such a trigger is pushed (`push_pending_trigger`)
    /// and removed once it resolves. `Value::PowerOf`/`ToughnessOf`
    /// consult it (priority over the graveyard's printed P/T) while
    /// `resolving_lki_source` names the trigger currently resolving.
    /// Transient scratch — `#[serde(skip)]`.
    #[serde(skip)]
    pub(crate) leaves_bf_lki: HashMap<CardId, CardInstance>,
    /// The source CardId of the leaves-battlefield trigger currently
    /// resolving, if it has a `leaves_bf_lki` snapshot. Scopes the LKI
    /// power/toughness read to that one resolution. `#[serde(skip)]`.
    #[serde(skip)]
    pub(crate) resolving_lki_source: Option<CardId>,
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
    /// CR 702.15 — the seat that should gain life from lifelink on damage dealt
    /// by the instant/sorcery spell currently resolving, if its controller has
    /// "your spells have lifelink" (Radiant Scrollwielder). Set around the
    /// spell's resolution and cleared after; transient, not serialized.
    #[serde(skip)]
    pub(crate) resolving_spell_lifelink_seat: Option<usize>,
    /// Reentrancy guard for the CR 121.2a draw-doubling replacement — the
    /// extra draws aren't themselves re-doubled (CR 614.5). Transient.
    #[serde(skip)]
    pub(crate) in_draw_double: bool,
    /// Reentrancy guard for CR 614.9 damage redirection (one redirect per
    /// damage event). Transient.
    #[serde(skip)]
    pub(crate) in_damage_redirect: bool,
    /// Temporary control changes awaiting reversion (Act of Treason /
    /// Threaten / Tempted by the Oriq). `Effect::GainControl` with a
    /// non-`Permanent` duration records the controller the permanent had
    /// immediately before the steal so control snaps back when the
    /// duration ends (CR 800.4 control-changing effects). `#[serde(default)]`
    /// for snapshot back-compat.
    #[serde(default)]
    pub(crate) temporary_control: Vec<TempControl>,
    /// Temporary "becomes a copy" definition swaps awaiting reversion
    /// (`Effect::BecomeCopyOfFor`, CR 707.2). Records the pre-copy
    /// definition; the swap snaps back when the duration ends, mirroring
    /// `temporary_control`. Entries whose card left the battlefield are
    /// dropped (a new object keeps nothing). `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub(crate) temporary_copies: Vec<TempCopy>,
    /// CR 702.143b — cards foretold this turn can't be cast from exile until
    /// a later turn. Tracks the cards a player foretold during the current
    /// turn; cleared at cleanup. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) foretold_this_turn: std::collections::HashSet<CardId>,
    /// CR 702.170 — cards currently plotted (exiled face-up, castable from
    /// exile without paying their mana cost on a later turn).
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) plotted_cards: std::collections::HashSet<CardId>,
    /// CR 702.170d — cards plotted *this* turn can't be cast until a later
    /// turn. Cleared at cleanup. `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub(crate) plotted_this_turn: std::collections::HashSet<CardId>,
    /// CR 603.3d — triggered abilities flagged `TriggeredAbility::once_per_turn`
    /// ("this ability triggers only once each turn") that have already fired
    /// this turn, keyed by (source card, trigger index). Cleared at cleanup.
    /// `#[serde(default)]` for snapshot back-compat. Powers Dramatic Finale.
    #[serde(default)]
    pub(crate) triggered_once_per_turn_used: std::collections::HashSet<(CardId, usize)>,
    /// CR 724 — the monarch (if any). The monarch draws a card at the
    /// beginning of their end step, and a creature dealing combat damage to
    /// the monarch makes its controller the new monarch. `#[serde(default)]`
    /// (None = no monarch) for snapshot back-compat.
    #[serde(default)]
    pub monarch: Option<usize>,
    /// CR 731 — the game's day/night designation (None = neither, the
    /// starting state). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub day_night: Option<crate::game::types::DayNight>,
    /// The active player of the turn that just ended — read by the CR 502.2
    /// day/night turn-based check (which consults the *previous* turn's
    /// active player's spell count). `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub(crate) previous_turn_active: Option<usize>,
}

/// A pending control-reversion entry — see `GameState.temporary_control`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TempControl {
    pub(crate) card: CardId,
    pub(crate) original_controller: usize,
    pub(crate) duration: crate::effect::Duration,
}

/// A pending copy-reversion entry — see `GameState.temporary_copies`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct TempCopy {
    pub(crate) card: CardId,
    /// Live handle to the pre-copy definition. Skipped in snapshots —
    /// recovered by name through the registry resolver on load (the same
    /// name-keyed round-trip `CardInstance` uses).
    #[serde(skip)]
    pub(crate) original: Option<std::sync::Arc<crate::card::CardDefinition>>,
    pub(crate) original_name: String,
    pub(crate) duration: crate::effect::Duration,
}

impl TempCopy {
    /// The pre-copy definition: the live Arc, or a registry lookup after a
    /// snapshot round-trip.
    fn original_def(&self) -> Option<std::sync::Arc<crate::card::CardDefinition>> {
        self.original.clone().or_else(|| {
            crabomination_base::registry::resolve_card(&self.original_name).map(std::sync::Arc::new)
        })
    }
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
            phased_out: self.phased_out.clone(),
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
            combat_damage_order: self.combat_damage_order.clone(),
            combat_damage_assignment: self.combat_damage_assignment.clone(),
            combat_damage_plan_step: self.combat_damage_plan_step,
            blockers_declared: self.blockers_declared,
            skip_first_draw: self.skip_first_draw,
            spells_cast_this_turn: self.spells_cast_this_turn,
            mana_spent_on_spells_this_turn: self.mana_spent_on_spells_this_turn,
            expend_prev_total: self.expend_prev_total,
            spells_cast_last_turn: self.spells_cast_last_turn,
            permanents_to_graveyard_this_turn: self.permanents_to_graveyard_this_turn,
            entered_from_graveyard_this_turn: self.entered_from_graveyard_this_turn.clone(),
            delayed_triggers: self.delayed_triggers.clone(),
            attacking_token_cleanup: self.attacking_token_cleanup.clone(),
            sacrificed_power: self.sacrificed_power,
            sacrificed_toughness: self.sacrificed_toughness,
            sacrificed_mana_value: self.sacrificed_mana_value,
            trigger_event_amount_scratch: self.trigger_event_amount_scratch,
            last_created_token: self.last_created_token,
            last_die_roll: self.last_die_roll,
            extra_cast_reduction: self.extra_cast_reduction,
            cast_paid_uncounterable: self.cast_paid_uncounterable,
            last_created_tokens: self.last_created_tokens.clone(),
            last_moved_cards: self.last_moved_cards.clone(),
            cards_discarded_this_resolution: self.cards_discarded_this_resolution,
            creature_cards_discarded_this_resolution: self.creature_cards_discarded_this_resolution,
            cards_discarded_per_player_this_resolution: self.cards_discarded_per_player_this_resolution.clone(),
            nonland_cards_discarded_per_player_this_resolution: self.nonland_cards_discarded_per_player_this_resolution.clone(),
            shuffle_resolving_spell_into_library: self.shuffle_resolving_spell_into_library,
            return_resolving_spell_to_hand: self.return_resolving_spell_to_hand,
            exile_resolving_spell: self.exile_resolving_spell,
            end_turn_requested: self.end_turn_requested,
            cipher_encode_pending: self.cipher_encode_pending,
            discarded_card_ids_this_resolution: self.discarded_card_ids_this_resolution.clone(),
            permanents_destroyed_this_resolution: self.permanents_destroyed_this_resolution,
            players_sacrificed_this_resolution: self.players_sacrificed_this_resolution.clone(),
            named_card_this_resolution: self.named_card_this_resolution.clone(),
            pending_cast_face: self.pending_cast_face,
            pending_cast_sacrifices: self.pending_cast_sacrifices.clone(),
            pending_cast_discards: self.pending_cast_discards.clone(),
            pending_cast_spend_float: self.pending_cast_spend_float,
            pending_ability_sac_other: self.pending_ability_sac_other,
            pending_ability_tap_other: self.pending_ability_tap_other,
            pending_ability_exile_other: self.pending_ability_exile_other.clone(),
            decider: self.decider.kind().into_boxed(),
            pending_decision: self.pending_decision.clone(),
            suspend_signal: self.suspend_signal.clone(),
            stashed_resolution_answer: self.stashed_resolution_answer.clone(),
            prevent_combat_damage_this_turn: self.prevent_combat_damage_this_turn,
            mana_production_doublers: self.mana_production_doublers,
            resolving_source: self.resolving_source.clone(),
            in_layer_gather: std::sync::atomic::AtomicBool::new(false),
            additional_combat_phases: self.additional_combat_phases,
            additional_post_main_combats: self.additional_post_main_combats,
            combat_damage_prevented_creatures: self.combat_damage_prevented_creatures.clone(),
            blocked_attackers: self.blocked_attackers.clone(),
            creature_etb_steal_this_turn: self.creature_etb_steal_this_turn.clone(),
            search_tax_paid_this_turn: self.search_tax_paid_this_turn.clone(),
            damage_prevented_sources: self.damage_prevented_sources.clone(),
            cant_block_pairs: self.cant_block_pairs.clone(),
            prevention_shields: self.prevention_shields.clone(),
            damage_cant_be_prevented_this_turn: self.damage_cant_be_prevented_this_turn,
            replacement_effects: self.replacement_effects.clone(),
            next_replacement_id: self.next_replacement_id,
            commander_cast_count: self.commander_cast_count.clone(),
            commander_damage: self.commander_damage.clone(),
            died_card_snapshots: self.died_card_snapshots.clone(),
            leaves_bf_lki: self.leaves_bf_lki.clone(),
            resolving_lki_source: self.resolving_lki_source,
            permanents_gained_counter_this_turn: self.permanents_gained_counter_this_turn.clone(),
            granted_triggers_eot: self.granted_triggers_eot.clone(),
            dies_to_exile_eot: self.dies_to_exile_eot.clone(),
            resolving_spell_lifelink_seat: self.resolving_spell_lifelink_seat,
            in_draw_double: self.in_draw_double,
            in_damage_redirect: self.in_damage_redirect,
            temporary_control: self.temporary_control.clone(),
            temporary_copies: self.temporary_copies.clone(),
            foretold_this_turn: self.foretold_this_turn.clone(),
            plotted_cards: self.plotted_cards.clone(),
            plotted_this_turn: self.plotted_this_turn.clone(),
            triggered_once_per_turn_used: self.triggered_once_per_turn_used.clone(),
            monarch: self.monarch,
            day_night: self.day_night,
            previous_turn_active: self.previous_turn_active,
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
            phased_out: Vec::new(),
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
            combat_damage_order: HashMap::new(),
            combat_damage_assignment: HashMap::new(),
            combat_damage_plan_step: None,
            blockers_declared: false,
            // Multiplayer (3+) doesn't skip the first draw — only the 2-player
            // starting player does.
            skip_first_draw: n <= 2,
            spells_cast_this_turn: 0,
            mana_spent_on_spells_this_turn: 0,
            expend_prev_total: 0,
            spells_cast_last_turn: 0,
            permanents_to_graveyard_this_turn: 0,
            entered_from_graveyard_this_turn: std::collections::HashSet::new(),
            delayed_triggers: Vec::new(),
            attacking_token_cleanup: Vec::new(),
            sacrificed_power: None,
            sacrificed_toughness: None,
            sacrificed_mana_value: None,
            trigger_event_amount_scratch: 0,
            last_created_token: None,
            last_die_roll: 0,
            extra_cast_reduction: 0,
            cast_paid_uncounterable: false,
            last_created_tokens: Vec::new(),
            last_moved_cards: Vec::new(),
            cards_discarded_this_resolution: 0,
            creature_cards_discarded_this_resolution: 0,
            cards_discarded_per_player_this_resolution: HashMap::new(),
            nonland_cards_discarded_per_player_this_resolution: HashMap::new(),
            shuffle_resolving_spell_into_library: false,
            return_resolving_spell_to_hand: false,
            exile_resolving_spell: false,
            end_turn_requested: false,
            cipher_encode_pending: None,
            discarded_card_ids_this_resolution: Vec::new(),
            permanents_destroyed_this_resolution: 0,
            players_sacrificed_this_resolution: std::collections::HashSet::new(),
            named_card_this_resolution: None,
            pending_cast_face: CastFace::Front,
            pending_cast_sacrifices: None,
            pending_cast_discards: None,
            pending_cast_spend_float: None,
            pending_ability_sac_other: None,
            pending_ability_tap_other: None,
            pending_ability_exile_other: None,
            decider: Box::new(AutoDecider),
            pending_decision: None,
            suspend_signal: None,
            stashed_resolution_answer: None,
            prevent_combat_damage_this_turn: false,
            mana_production_doublers: 0,
            resolving_source: None,
            in_layer_gather: std::sync::atomic::AtomicBool::new(false),
            additional_combat_phases: 0,
            additional_post_main_combats: 0,
            combat_damage_prevented_creatures: Vec::new(),
            blocked_attackers: Vec::new(),
            creature_etb_steal_this_turn: Vec::new(),
            search_tax_paid_this_turn: Vec::new(),
            damage_prevented_sources: Vec::new(),
            cant_block_pairs: Vec::new(),
            prevention_shields: Vec::new(),
            damage_cant_be_prevented_this_turn: false,
            replacement_effects: Vec::new(),
            next_replacement_id: 1,
            commander_cast_count: HashMap::new(),
            commander_damage: HashMap::new(),
            died_card_snapshots: HashMap::new(),
            leaves_bf_lki: HashMap::new(),
            resolving_lki_source: None,
            permanents_gained_counter_this_turn: std::collections::HashSet::new(),
            granted_triggers_eot: std::collections::HashMap::new(),
            dies_to_exile_eot: std::collections::HashSet::new(),
            resolving_spell_lifelink_seat: None,
            in_draw_double: false,
            in_damage_redirect: false,
            temporary_control: Vec::new(),
            temporary_copies: Vec::new(),
            foretold_this_turn: std::collections::HashSet::new(),
            plotted_cards: std::collections::HashSet::new(),
            plotted_this_turn: std::collections::HashSet::new(),
            triggered_once_per_turn_used: std::collections::HashSet::new(),
            monarch: None,
            day_night: None,
            previous_turn_active: None,
        }
    }

    /// CR 724 — make `player` the monarch. No-op if they already are; emits
    /// `MonarchChanged` on a real change.
    pub(crate) fn set_monarch(&mut self, player: usize, events: &mut Vec<GameEvent>) {
        if self.monarch == Some(player) {
            return;
        }
        self.monarch = Some(player);
        events.push(GameEvent::MonarchChanged { player });
    }

    /// CR 731 — set the game's day/night designation, emitting
    /// `DayNightChanged` on a real change.
    /// CR 712 — flip one DFC permanent to its other face in place. The object
    /// is unchanged (counters/tapped/attachments persist); fires `Transformed`.
    pub(crate) fn transform_permanent(&mut self, id: CardId, events: &mut Vec<GameEvent>) {
        let Some(c) = self.battlefield_find_mut(id) else { return };
        if !c.transformed {
            let Some(back) = c.definition.back_face.as_ref().map(|b| (**b).clone()) else { return };
            c.front_face = Some(c.definition.clone());
            c.definition = std::sync::Arc::new(back);
            c.transformed = true;
        } else {
            let Some(front) = c.front_face.take() else { return };
            c.definition = front;
            c.transformed = false;
        }
        events.push(GameEvent::Transformed { card_id: id });
    }

    pub(crate) fn set_day_night(&mut self, dn: crate::game::types::DayNight, events: &mut Vec<GameEvent>) {
        use crate::game::types::DayNight;
        if self.day_night == Some(dn) {
            return;
        }
        self.day_night = Some(dn);
        events.push(GameEvent::DayNightChanged { day_night: dn });
        // CR 702.146f/g — daybound/nightbound DFCs flip with the day/night
        // cycle: front (daybound) ↔ back (nightbound).
        let want = match dn {
            DayNight::Night => Keyword::Daybound,
            DayNight::Day => Keyword::Nightbound,
        };
        let to_flip: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.keywords.contains(&want))
            .map(|c| c.id)
            .collect();
        for id in to_flip {
            self.transform_permanent(id, events);
        }
    }

    /// CR 502.2 — the day/night turn-based check run as each turn begins.
    /// If it's day and the previous turn's active player cast no spells, it
    /// becomes night; if it's night and they cast two or more, it becomes
    /// day. No effect while the game is neither day nor night.
    pub(crate) fn check_day_night_transition(&mut self, events: &mut Vec<GameEvent>) {
        use crate::game::types::DayNight;
        let Some(current) = self.day_night else { return };
        let Some(prev) = self.previous_turn_active else { return };
        let cast = self.players.get(prev).map(|p| p.spells_cast_this_turn).unwrap_or(0);
        match current {
            DayNight::Day if cast == 0 => self.set_day_night(DayNight::Night, events),
            DayNight::Night if cast >= 2 => self.set_day_night(DayNight::Day, events),
            _ => {}
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
        // CR 614 — Tainted Remedy-style replacement: a would-be life *gain*
        // becomes an equal life *loss* instead. Applied before the cannot-
        // gain drop (the gain is replaced, not prevented) and re-routed as a
        // negative delta so the loss honors cannot-lose-life / shared pools.
        if delta > 0 && self.life_gain_becomes_loss_now(seat) {
            return self.adjust_life(seat, -delta);
        }
        if delta > 0 && self.player_cannot_gain_life_now(seat) {
            return self.effective_life(seat);
        }
        // CR 119.10 — a genuine life *gain* is increased by any active
        // "you gain that much plus N" replacement (Honor Troll). Folded in
        // before the gain applies so the bonus counts toward
        // `life_gained_this_turn` and any downstream lifegain triggers.
        let delta = if delta > 0 {
            delta.saturating_add(self.life_gain_bonus_now(seat))
        } else {
            delta
        };
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
        } else {
            // delta < 0 — this player lost life (CR 119.3). Powers Spectacle.
            self.players[seat].lost_life_this_turn = true;
            self.players[seat].life_lost_this_turn =
                self.players[seat].life_lost_this_turn.saturating_add((-delta) as u32);
        }
        new_total
    }

    /// Like [`adjust_life`] but returns the *applied* delta — after the
    /// cannot-gain/lose drops, gain→loss replacement, and gain bonuses
    /// (CR 119.7/119.10/614). Callers that emit `LifeGained`/`LifeLost`
    /// must use this so triggers don't fire on gains that never happened.
    pub fn adjust_life_applied(&mut self, seat: usize, delta: i32) -> i32 {
        let before = self.effective_life(seat);
        let after = self.adjust_life(seat, delta);
        after - before
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

    /// CR 614.5 — how many -1/-1 counters to shave off a placement onto one
    /// of `seat`'s creatures (Vizier of Remedies; one per copy).
    pub fn minus_counter_reduction_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::MinusCounterReduction))
                    .count() as u32
            })
            .sum()
    }

    /// Extra ETB counters granted by `StaticEffect::ChosenTypeEntersWithCounter`
    /// (Metallic Mimic). For a creature `entering` under `controller`, returns
    /// one counter spec per matching source: a *different* permanent the same
    /// player controls whose chosen creature type is among the entering
    /// creature's types. The entering card must already be on the battlefield.
    pub(crate) fn chosen_type_etb_counter_specs(
        &self,
        entering: CardId,
        controller: usize,
    ) -> Vec<crate::card::CounterType> {
        use crate::effect::StaticEffect;
        let Some(ec) = self.battlefield.iter().find(|c| c.id == entering) else {
            return vec![];
        };
        if !ec.definition.is_creature() {
            return vec![];
        }
        let entering_types = ec.definition.subtypes.creature_types.clone();
        let mut specs = vec![];
        for src in &self.battlefield {
            if src.controller != controller || src.id == entering {
                continue;
            }
            let Some(ct) = src.chosen_creature_type else { continue };
            if !entering_types.contains(&ct) {
                continue;
            }
            for sa in &src.definition.static_abilities {
                if let StaticEffect::ChosenTypeEntersWithCounter { kind } = &sa.effect {
                    specs.push(*kind);
                }
            }
        }
        specs
    }

    /// CR 702.32 / 702.62 — a permanent with Fading N / Vanishing N enters
    /// with N fade / time counters. Called from both ETB paths after the
    /// permanent is on the battlefield.
    pub(crate) fn apply_fading_vanishing_etb(
        &mut self,
        cid: CardId,
        events: &mut Vec<crate::game::GameEvent>,
    ) {
        use crate::card::{CounterType, Keyword};
        let Some(card) = self.battlefield_find(cid) else { return };
        let spec = card.definition.keywords.iter().find_map(|k| match k {
            Keyword::Fading(n) => Some((CounterType::Fade, *n)),
            Keyword::Vanishing(n) => Some((CounterType::Time, *n)),
            _ => None,
        });
        let Some((kind, n)) = spec else { return };
        if n == 0 {
            return;
        }
        if let Some(card_mut) = self.battlefield_find_mut(cid) {
            card_mut.add_counters(kind, n);
        }
        events.push(crate::game::GameEvent::CounterAdded {
            card_id: cid,
            counter_type: kind,
            count: n,
        });
    }

    /// CR 702.183 — a permanent cast for its Impending cost enters with N
    /// time counters (stamped on `CardInstance.impending_counters` at cast).
    pub(crate) fn apply_impending_etb(
        &mut self,
        cid: CardId,
        events: &mut Vec<crate::game::GameEvent>,
    ) {
        use crate::card::CounterType;
        let Some(card) = self.battlefield_find_mut(cid) else { return };
        let n = card.impending_counters;
        if n == 0 {
            return;
        }
        card.impending_counters = 0;
        card.add_counters(CounterType::Time, n);
        events.push(crate::game::GameEvent::CounterAdded {
            card_id: cid,
            counter_type: CounterType::Time,
            count: n,
        });
    }

    /// CR 702.183 — at the beginning of the active player's end step, remove
    /// one time counter from each Impending permanent they control. Unlike
    /// Vanishing there's no sacrifice: when the last counter comes off the
    /// permanent simply stops being a non-creature (the layer effect reads
    /// the live counter count) and turns into a creature.
    pub(crate) fn process_impending(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, Keyword};
        let active = self.active_player_idx;
        let mut events = Vec::new();
        let affected: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == active
                    && c.counter_count(CounterType::Time) > 0
                    && c.definition
                        .keywords
                        .iter()
                        .any(|k| matches!(k, Keyword::Impending(_)))
            })
            .map(|c| c.id)
            .collect();
        for id in affected {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.remove_counters(CounterType::Time, 1);
            }
            events.push(crate::game::GameEvent::CounterRemoved {
                card_id: id,
                counter_type: CounterType::Time,
                count: 1,
            });
        }
        events
    }

    /// CR 702.32 / 702.62 — at the beginning of the active player's upkeep,
    /// each Fading / Vanishing permanent they control removes a counter (and
    /// is sacrificed when it runs out). Processed as a turn-based action at
    /// upkeep before priority.
    pub(crate) fn process_fading_vanishing(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, Keyword};
        let active = self.active_player_idx;
        let mut events = Vec::new();
        // Snapshot the affected (id, fading?) pairs first to avoid borrow churn.
        let affected: Vec<(CardId, bool)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == active)
            .filter_map(|c| {
                c.definition.keywords.iter().find_map(|k| match k {
                    Keyword::Fading(_) => Some((c.id, true)),
                    Keyword::Vanishing(_) => Some((c.id, false)),
                    _ => None,
                })
            })
            .collect();
        for (id, is_fading) in affected {
            let kind = if is_fading { CounterType::Fade } else { CounterType::Time };
            let had = self.battlefield_find(id).map(|c| c.counter_count(kind)).unwrap_or(0);
            let sacrifice = if is_fading {
                // Fading: remove one; if none to remove, sacrifice.
                if had == 0 {
                    true
                } else {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.remove_counters(kind, 1);
                    }
                    events.push(crate::game::GameEvent::CounterRemoved {
                        card_id: id,
                        counter_type: kind,
                        count: 1,
                    });
                    false
                }
            } else {
                // Vanishing: remove one; sacrifice when the last is removed.
                if had > 0 {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.remove_counters(kind, 1);
                    }
                    events.push(crate::game::GameEvent::CounterRemoved {
                        card_id: id,
                        counter_type: kind,
                        count: 1,
                    });
                }
                had <= 1
            };
            if sacrifice {
                // CR 700.4 — the shared sacrifice helper emits the full
                // event set (CreatureDied included) + die snapshot.
                self.sacrifice_one(id, active, &mut events);
            }
        }
        events
    }

    /// CR 702.24 — Cumulative upkeep. As a turn-based action at the active
    /// player's upkeep, each permanent they control with
    /// `Keyword::CumulativeUpkeep(cost)` gets one age counter; its controller
    /// then pays `cost` once per age counter on it (mana from the pool, life,
    /// or sacrificing matching permanents), or sacrifices the permanent.
    /// (Following `PayManaOrElse`, mana is auto-paid from the pool when
    /// affordable — an interactive pay prompt is a follow-up.)
    pub(crate) fn process_cumulative_upkeep(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, CumulativeUpkeepCost, Keyword};
        let active = self.active_player_idx;
        let mut events = Vec::new();
        let affected: Vec<(CardId, CumulativeUpkeepCost)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == active)
            .filter_map(|c| {
                c.definition.keywords.iter().find_map(|k| match k {
                    Keyword::CumulativeUpkeep(cost) => Some((c.id, cost.clone())),
                    _ => None,
                })
            })
            .collect();
        for (id, cost) in affected {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.add_counters(CounterType::Age, 1);
            }
            events.push(crate::game::GameEvent::CounterAdded {
                card_id: id,
                counter_type: CounterType::Age,
                count: 1,
            });
            let n = self.battlefield_find(id).map(|c| c.counter_count(CounterType::Age)).unwrap_or(1);
            let paid = match &cost {
                CumulativeUpkeepCost::Mana(mc) => {
                    // Total = cost × age counters (repeat the pip list N times).
                    let mut symbols = Vec::new();
                    for _ in 0..n {
                        symbols.extend(mc.symbols.iter().cloned());
                    }
                    self.players[active].mana_pool.pay(&crate::mana::ManaCost::new(symbols)).is_ok()
                }
                CumulativeUpkeepCost::Life(per) => {
                    let total = per * n;
                    // Auto-pay life only while it leaves the player alive.
                    if self.players[active].life > total as i32 {
                        self.adjust_life(active, -(total as i32));
                        events.push(crate::game::GameEvent::LifeLost { player: active, amount: total });
                        true
                    } else {
                        false
                    }
                }
                CumulativeUpkeepCost::Sacrifice(filter) => {
                    // Need N matching permanents (other than the source) to pay.
                    let cands = self.sacrifice_candidates(active, filter, Some(id));
                    let cands: Vec<CardId> = cands.into_iter().filter(|&c| c != id).collect();
                    if cands.len() >= n as usize {
                        let pick = self.auto_pick_sacrifices(&cands, n as usize, Some(id), false, false);
                        for sid in pick {
                            self.sacrifice_one(sid, active, &mut events);
                        }
                        true
                    } else {
                        false
                    }
                }
            };
            if !paid {
                self.sacrifice_one(id, active, &mut events);
            }
        }
        events
    }

    /// CR 702.62d/e — remove one time counter from each suspended card the
    /// active player owns in exile; when the last counter comes off, cast
    /// the card without paying its mana cost (a creature so cast clears its
    /// summoning sickness — Suspend grants haste). Targets are auto-chosen,
    /// matching AutoDecider behavior for other free casts.
    pub(crate) fn process_suspend(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, Keyword};
        let active = self.active_player_idx;
        let mut events = Vec::new();
        // Snapshot suspended exiled cards (Suspend keyword + ≥1 time counter)
        // owned by the active player, so the borrow is released before casting.
        let suspended: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| {
                c.owner == active
                    && c.counter_count(CounterType::Time) > 0
                    && c.definition
                        .keywords
                        .iter()
                        .any(|k| matches!(k, Keyword::Suspend(..)))
            })
            .map(|c| c.id)
            .collect();
        for id in suspended {
            events.append(&mut self.remove_suspend_time_counter(id));
        }
        events
    }

    /// Uvilda, Dean of Perfection — at the active player's upkeep, remove one
    /// hone counter from each instant/sorcery they own in exile with hone
    /// counters. When the last comes off, grant them permission to cast it
    /// from exile for {4} less (the printed "you may cast it" window).
    pub(crate) fn process_hone(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, MayPlayDuration, MayPlayPermission};
        let active = self.active_player_idx;
        let turn = self.turn_number;
        let mut events = Vec::new();
        let honed: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| c.owner == active && c.counter_count(CounterType::Hone) > 0)
            .map(|c| c.id)
            .collect();
        for id in honed {
            let Some(card) = self.exile.iter_mut().find(|c| c.id == id) else { continue };
            card.remove_counters(CounterType::Hone, 1);
            events.push(crate::game::GameEvent::CounterRemoved {
                card_id: id,
                counter_type: CounterType::Hone,
                count: 1,
            });
            if card.counter_count(CounterType::Hone) > 0 {
                continue;
            }
            // Last hone counter removed — castable from exile for {4} less.
            let mut cost = card.definition.cost.clone();
            cost.reduce_generic(4);
            card.may_play_until = Some(MayPlayPermission {
                player: active,
                granted_turn: turn,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                exile_after: false,
            });
            card.granted_alt_cast_cost_eot = Some(cost);
        }
        events
    }

    /// Remove one time counter from a suspended card in exile; when the last
    /// is removed, free-cast it from exile (CR 702.62e–f). Shared by the
    /// upkeep tick (`process_suspend`) and accelerants (Deep-Sea Kraken).
    pub(crate) fn remove_suspend_time_counter(
        &mut self,
        id: CardId,
    ) -> Vec<crate::game::GameEvent> {
        use crate::card::CounterType;
        let mut events = Vec::new();
        let Some(card) = self.exile.iter_mut().find(|c| c.id == id) else { return events };
        if card.counter_count(CounterType::Time) == 0 {
            return events;
        }
        card.remove_counters(CounterType::Time, 1);
        events.push(crate::game::GameEvent::CounterRemoved {
            card_id: id,
            counter_type: CounterType::Time,
            count: 1,
        });
        if card.counter_count(CounterType::Time) > 0 {
            return events;
        }
        let owner = card.owner;
        // Last counter removed — cast it for free from exile. Compute an
        // auto-target against the card's effect (the owner chooses in real
        // play; we collapse to the AutoDecider's first-legal pick). Stamp the
        // suspend-cast flag so a creature gains haste on ETB (CR 702.62f).
        card.cast_from_suspend = true;
        let effect = card.definition.effect.clone();
        let auto_target = self.auto_target_for_effect_avoiding(&effect, owner, Some(id));
        // The suspending owner casts it; route priority so the cast helper
        // attributes it correctly.
        let saved_priority = self.priority.player_with_priority;
        self.priority.player_with_priority = owner;
        let cast = self.cast_card_for_free(
            owner,
            id,
            crate::card::Zone::Exile,
            auto_target,
            vec![],
            None,
            None,
            false,
        );
        self.priority.player_with_priority = saved_priority;
        // If it can't be cast (e.g. no legal target) CR 702.62e leaves it
        // exiled with 0 time counters.
        if let Ok(mut evs) = cast {
            events.append(&mut evs);
        }
        events
    }

    /// CR 702.62 accelerants — when `caster` casts a spell, tick a time
    /// counter off every opponent-owned suspended card that has
    /// `Keyword::SuspendAccelerant` (Deep-Sea Kraken).
    pub(crate) fn process_suspend_accelerants(
        &mut self,
        caster: usize,
    ) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, Keyword};
        let targets: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| {
                c.owner != caster
                    && c.counter_count(CounterType::Time) > 0
                    && c.definition.keywords.contains(&Keyword::SuspendAccelerant)
            })
            .map(|c| c.id)
            .collect();
        let mut events = Vec::new();
        for id in targets {
            events.append(&mut self.remove_suspend_time_counter(id));
        }
        events
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

    /// CR 614.5 — number of `StaticEffect::HalveDamageDealt` permanents on
    /// the battlefield (Ghosts of the Innocent). Each halves the dealt
    /// amount, rounded down; applied after any doublers.
    pub fn damage_halvers(&self) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::HalveDamageDealt))
                    .count() as u32
            })
            .sum()
    }

    /// Scale a pending damage event by the global doubling/halving
    /// replacements (CR 614.2 / 614.5): every doubler ×2, then every
    /// halver ÷2 rounded down.
    pub fn scale_damage(&self, amount: u32) -> u32 {
        let d = self.damage_doublers().min(16);
        let h = self.damage_halvers().min(16);
        amount.saturating_mul(1 << d) >> h
    }

    /// Source- and target-aware damage scaling: the global doublers/halvers,
    /// the side-scoped ones (Gisela, Blade of Goldnight —
    /// `DoubleDamageToOpponents` doubles events hitting an opponent's side,
    /// `HalveDamageToYou` halves events hitting the controller's own side,
    /// CR 614.5), and the source-scoped additive bonus (Torbran —
    /// `AddDamageToOpponents`, applied before the multipliers).
    pub fn scale_damage_to(
        &self,
        source: Option<CardId>,
        ent: crate::game::effects::EntityRef,
        amount: u32,
    ) -> u32 {
        use crate::effect::StaticEffect;
        use crate::game::effects::EntityRef;
        let affected = match ent {
            EntityRef::Player(p) => Some(p),
            EntityRef::Permanent(c) => self.battlefield_find(c).map(|c| c.controller),
            EntityRef::Card(_) => None,
        };
        // Source identity: a battlefield permanent's computed colors +
        // controller, else the resolving spell stamped by `resolve_spell`.
        let source_info: Option<(usize, Vec<crate::mana::Color>)> = source.and_then(|s| {
            self.computed_permanent(s)
                .map(|cp| (cp.controller, cp.colors.clone()))
                .or_else(|| match &self.resolving_source {
                    Some((id, caster, colors)) if *id == s => {
                        Some((*caster, colors.clone()))
                    }
                    _ => None,
                })
        });
        let mut amount = amount;
        let mut d = self.damage_doublers();
        let mut h = self.damage_halvers();
        if let Some(p) = affected {
            for c in &self.battlefield {
                for sa in &c.definition.static_abilities {
                    match &sa.effect {
                        StaticEffect::DoubleDamageToOpponents
                            if !self.same_team(c.controller, p) =>
                        {
                            d += 1;
                        }
                        StaticEffect::HalveDamageToYou if c.controller == p => h += 1,
                        StaticEffect::AddDamageToOpponents { source_color, amount: bonus }
                            if !self.same_team(c.controller, p) =>
                        {
                            // "+N if a [color] source you control" — needs a
                            // known source controlled by the static's owner.
                            if let Some((src_ctrl, src_colors)) = &source_info
                                && *src_ctrl == c.controller
                                && source_color.is_none_or(|sc| src_colors.contains(&sc))
                            {
                                amount = amount.saturating_add(*bonus);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        amount.saturating_mul(1 << d.min(16)) >> h.min(16)
    }

    /// CR 122.1 — true if any active `StaticEffect::CountersCantBePlaced`
    /// (Solemnity) is on the battlefield. While set, every counter-placement
    /// site drops the counters instead of adding them.
    pub fn counters_locked(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::CountersCantBePlaced))
        })
    }

    /// CR 614.6 — true if `card` bound for its owner's graveyard is exiled
    /// instead (Rest in Peace exiles everything; Leyline of the Void only
    /// an opponent's cards; Sanctifier en-Vec only black/red cards).
    /// Consulted by `route_to_graveyard` at every graveyard-placement site.
    pub fn graveyard_exiled_for(&self, card: &crate::card::CardInstance) -> bool {
        self.graveyard_exile_redirects(card).0
    }

    /// `(redirects, void_counter)` for `card`: whether some
    /// `ExileCardsBoundForGraveyard` static redirects it to exile, and
    /// whether any applicable redirect stamps a void counter on it
    /// (Dauthi Voidwalker).
    pub(crate) fn graveyard_exile_redirects(
        &self,
        card: &crate::card::CardInstance,
    ) -> (bool, bool) {
        use crate::effect::StaticEffect;
        let owner = card.owner;
        let mut redirects = false;
        let mut void = false;
        for c in &self.battlefield {
            for sa in &c.definition.static_abilities {
                if let StaticEffect::ExileCardsBoundForGraveyard {
                    opponents_only,
                    colors,
                    void_counter,
                } = &sa.effect
                {
                    let applies = (!opponents_only || c.controller != owner)
                        && colors.as_ref().is_none_or(|cs| {
                            card.definition.printed_colors().iter().any(|c| cs.contains(c))
                        });
                    if applies {
                        redirects = true;
                        void |= void_counter;
                    }
                }
            }
        }
        (redirects, void)
    }

    /// CR 614.5 — the actual mill count for `p` after doubling replacements
    /// (Bruvac the Grandiloquent: an opponent's mill is doubled, once per
    /// active static). 0 stays 0 (no event to replace).
    pub(crate) fn mill_count_for(&self, p: usize, n: usize) -> usize {
        use crate::effect::StaticEffect;
        if n == 0 {
            return 0;
        }
        let doublers = self
            .battlefield
            .iter()
            .filter(|c| {
                !self.same_team(c.controller, p)
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::OpponentMillDoubled))
            })
            .count()
            .min(16);
        n << doublers
    }

    /// CR 701.19c (Aven Mindcensor) — the number of cards from the top of
    /// the library `seat` may look at while searching, or `None` if
    /// unrestricted. The minimum across every opposing
    /// `OpponentsSearchTopN` static applies.
    pub(crate) fn search_top_limit_for(&self, seat: usize) -> Option<usize> {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| !self.same_team(c.controller, seat))
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match sa.effect {
                StaticEffect::OpponentsSearchTopN { count } => Some(count as usize),
                _ => None,
            })
            .min()
    }

    /// Leonin Arbiter — charge `seat` the search tax before a library
    /// search. Auto-pays {amount} per Arbiter from floating mana (the
    /// payment covers the rest of the turn); returns `false` when the tax
    /// is unpayable, in which case the search finds nothing.
    pub(crate) fn pay_search_tax(&mut self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        if self.search_tax_paid_this_turn.contains(&seat) {
            return true;
        }
        let tax: u32 = self
            .battlefield
            .iter()
            .flat_map(|c| c.definition.static_abilities.iter())
            .map(|sa| match sa.effect {
                StaticEffect::SearchTax { amount } => amount,
                _ => 0,
            })
            .sum();
        if tax == 0 {
            return true;
        }
        if self.players[seat].mana_pool.total() < tax {
            return false;
        }
        self.players[seat].mana_pool.spend_generic(tax);
        self.search_tax_paid_this_turn.push(seat);
        true
    }

    /// CR 614.10 — true when a battlefield static makes `player` skip
    /// `step` (Eon Hub's "players skip their upkeep steps", Stasis-style
    /// untap skipping).
    pub(crate) fn step_skipped_for(&self, player: usize, step: TurnStep) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| match &sa.effect {
                StaticEffect::SkipStep { step: s, all_players } if *s == step => {
                    *all_players || c.controller == player
                }
                _ => false,
            })
        })
    }

    /// Grafdigger's Cage — true while any battlefield permanent locks
    /// graveyards/libraries (no creature entries from them, no casts from
    /// them).
    pub(crate) fn graveyard_library_locked(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::GraveyardLibraryLockdown))
        })
    }

    /// CR 614 (Gather Specimens): if a creature would enter the battlefield
    /// under `intended`'s control while an opponent of theirs registered the
    /// steal-replacement this turn, it enters under that opponent instead.
    pub(crate) fn apply_etb_control_replacement(
        &self,
        card: &crate::card::CardInstance,
        intended: usize,
    ) -> usize {
        if !card.definition.is_creature() {
            return intended;
        }
        self.creature_etb_steal_this_turn
            .iter()
            .copied()
            .find(|b| !self.same_team(*b, intended))
            .unwrap_or(intended)
    }

    /// Mint a token onto the battlefield: applies the Gather Specimens ETB
    /// control replacement (CR 614), pushes the entry events, records
    /// `last_created_token(s)`, and fires self-source ETB triggers. The
    /// shared funnel for every token-creation site.
    pub(crate) fn mint_token_onto_battlefield(
        &mut self,
        def: CardDefinition,
        controller: usize,
        tapped: bool,
        events: &mut Vec<crate::game::GameEvent>,
    ) -> CardId {
        let id = self.next_id();
        let mut inst = crate::card::CardInstance::new_token(id, def, controller);
        // CR 111.2 — a token's owner is the player under whose control it
        // actually entered, so a stolen mint belongs to the thief.
        let ctrl = self.apply_etb_control_replacement(&inst, controller);
        inst.owner = ctrl;
        inst.controller = ctrl;
        inst.tapped = tapped;
        self.battlefield.push(inst);
        events.push(crate::game::GameEvent::TokenCreated { card_id: id });
        events.push(crate::game::GameEvent::PermanentEntered { card_id: id });
        self.last_created_token = Some(id);
        self.last_created_tokens.push(id);
        self.fire_self_etb_triggers(id, ctrl);
        id
    }

    /// Bookkeeping for a card leaving `p`'s graveyard: bumps the per-turn
    /// tally and emits `CardLeftGraveyard` so leaves-graveyard payoffs see
    /// mass exilers too (Rest in Peace, Go Blank, Surgical Extraction).
    pub(crate) fn note_left_graveyard(
        &mut self,
        p: usize,
        card_id: CardId,
        events: &mut Vec<crate::game::GameEvent>,
    ) {
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        events.push(crate::game::GameEvent::CardLeftGraveyard { player: p, card_id });
    }

    /// Place `card` into its owner's graveyard, or exile it instead when a
    /// graveyard-hate static (Rest in Peace / Leyline of the Void) is active
    /// for that owner. Pushes a `PermanentExiled` event and returns `true`
    /// when the card was redirected to exile, so callers can suppress their
    /// own graveyard-specific event (CardMilled, etc.).
    pub(crate) fn route_to_graveyard(
        &mut self,
        card: crate::card::CardInstance,
        events: &mut Vec<crate::game::GameEvent>,
    ) -> bool {
        // CR 712.16 — a melded shell dies as its two component cards.
        if !card.meld_parts.is_empty() {
            let mut card = card;
            let mut any_exiled = false;
            for part in std::mem::take(&mut card.meld_parts) {
                any_exiled |= self.route_to_graveyard(part, events);
            }
            return any_exiled;
        }
        let owner = card.owner;
        // CR 614.6 — "shuffle into its owner's library instead" (Darksteel
        // Colossus). The card never touches the graveyard.
        if card.definition.shuffles_into_library_instead {
            use rand::seq::SliceRandom;
            self.players[owner].library.push(card);
            let mut rng = rand::rng();
            self.players[owner].library.shuffle(&mut rng);
            return false;
        }
        if self.graveyard_exiled_for(&card) || card.disturb_back_exiles() {
            let cid = card.id;
            let mut card = card;
            if self.graveyard_exile_redirects(&card).1 {
                card.add_counters(crate::card::CounterType::Void, 1);
            }
            self.exile.push(card);
            events.push(crate::game::GameEvent::PermanentExiled { card_id: cid });
            true
        } else {
            let cid = card.id;
            let is_land = card.definition.card_types.contains(&crate::card::CardType::Land);
            self.players[owner].send_to_graveyard(card);
            events.push(crate::game::GameEvent::CardPutIntoGraveyard {
                player: owner,
                card_id: cid,
                is_land,
            });
            false
        }
    }

    /// CR 702.66 — true if player `seat` controls a permanent granting
    /// "spells you cast have delve" (Teval, Arbiter of Virtue). Lets the
    /// cast path accept a delve-cards list on any spell, not just those
    /// printed with `Keyword::Delve`.
    pub fn controller_grants_spells_delve(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::SpellsYouCastHaveDelve))
        })
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

    /// CR 615.12 — True if any active `StaticEffect::DamageCantBePrevented`
    /// is on the battlefield (Sulfuric Vortex, Sunspine Lynx). Consulted by
    /// `apply_prevention_shields` to bypass every shield.
    pub fn damage_cant_be_prevented_now(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|src| {
            src.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::DamageCantBePrevented))
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

    /// CR 705.3 — coin-flip advantage for `seat` from active
    /// `StaticEffect::CoinFlipAdvantage` permanents (Krark's Thumb). Summed
    /// so multiple sources stack; added to `Player.coin_flip_advantage` by
    /// the `Effect::FlipCoin` resolver.
    /// CR 705.1/705.3 — flip one coin for `player`, honoring Krark's-Thumb
    /// style advantage (replay + treat as heads if any replay is heads).
    /// Returns true for heads.
    pub(crate) fn flip_one_coin(&mut self, player: usize) -> bool {
        let advantage = self.players.get(player).map(|p| p.coin_flip_advantage).unwrap_or(0)
            + self.coin_flip_advantage_now(player);
        let mut heads = false;
        for _ in 0..(advantage as usize + 1) {
            let answer = self.decider.decide(&crate::decision::Decision::CoinFlip { player });
            if matches!(answer, crate::decision::DecisionAnswer::Bool(true)) {
                heads = true;
            }
        }
        heads
    }

    pub fn coin_flip_advantage_now(&self, seat: usize) -> u32 {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield.iter().map(|src| {
            src.definition.static_abilities.iter().filter(|sa| {
                if let StaticEffect::CoinFlipAdvantage { target } = &sa.effect {
                    match target {
                        PlayerStaticTarget::Controller => src.controller == seat,
                        PlayerStaticTarget::EachOpponent => src.controller != seat,
                        PlayerStaticTarget::EachPlayer => true,
                    }
                } else {
                    false
                }
            }).count() as u32
        }).sum()
    }

    /// CR 614 — True if a would-be life *gain* by `seat` should be replaced
    /// with an equal life *loss* (Tainted Remedy). Scans the battlefield for
    /// any active `StaticEffect::LifeGainBecomesLoss` whose `target` includes
    /// `seat`.
    pub fn life_gain_becomes_loss_now(&self, seat: usize) -> bool {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| {
                if let StaticEffect::LifeGainBecomesLoss { target } = &sa.effect {
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

    /// CR 119.10 / 614 — total life-gain bonus currently applied to `seat`
    /// by `StaticEffect::LifeGainBonus` statics (Honor Troll's "+1 to each
    /// gain"). Bonuses from multiple sources stack additively.
    pub fn life_gain_bonus_now(&self, seat: usize) -> i32 {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield
            .iter()
            .flat_map(|src| {
                src.definition.static_abilities.iter().filter_map(move |sa| {
                    if let StaticEffect::LifeGainBonus { target, amount } = &sa.effect {
                        let hits = match target {
                            PlayerStaticTarget::Controller => src.controller == seat,
                            PlayerStaticTarget::EachOpponent => src.controller != seat,
                            PlayerStaticTarget::EachPlayer => true,
                        };
                        hits.then_some(*amount)
                    } else {
                        None
                    }
                })
            })
            .sum()
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
        // Reentrancy guard — see `in_layer_gather`. Passes below that
        // evaluate selection requirements must not recurse back into
        // `computed_permanent`.
        use std::sync::atomic::Ordering;
        self.in_layer_gather.store(true, Ordering::Relaxed);
        let out = self.gather_continuous_effects_inner();
        self.in_layer_gather.store(false, Ordering::Relaxed);
        out
    }

    fn gather_continuous_effects_inner(&self) -> Vec<ContinuousEffect> {
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
            // Flat bonus plus optional board-scaled bonus (Nettlecyst: +1/+1
            // for each artifact/enchantment the Equipment's controller controls).
            let (mut bp, mut bt) = (bonus.power, bonus.toughness);
            if let Some(scale) = &bonus.scale {
                let n = match scale.count_self_counters {
                    Some(kind) => card.counter_count(kind) as i32,
                    None => self
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == card.controller
                                && self.evaluate_requirement_on_card(
                                    &scale.filter,
                                    c,
                                    card.controller,
                                )
                        })
                        .count() as i32,
                };
                bp += n * scale.per_power;
                bt += n * scale.per_toughness;
            }
            if bp != 0 || bt != 0 {
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![target]),
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(bp, bt),
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
        // CR 702.95 — Soulbond. A creature carrying a `soulbond_bonus` that's
        // paired confers the bonus on BOTH itself and its partner (P/T layer
        // 7c, keywords layer 6), for as long as both stay on the battlefield.
        for card in &self.battlefield {
            let Some(bonus) = &card.definition.soulbond_bonus else { continue };
            let Some(partner) = card.soulbond_partner else { continue };
            if !self.battlefield.iter().any(|c| c.id == partner) {
                continue;
            }
            for &id in &[card.id, partner] {
                if bonus.power != 0 || bonus.toughness != 0 {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.id.0 as u64,
                        source: card.id,
                        affected: AffectedPermanents::Specific(vec![id]),
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
                        affected: AffectedPermanents::Specific(vec![id]),
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw.clone()),
                    });
                }
            }
        }
        // CR 702.151c — a Reconfigure Equipment isn't a creature while it's
        // attached to a creature. Strip the Creature card type at layer 4
        // (the +1/+1 it confers still scales off its own counters; its equip
        // bonus and exile ability are unaffected). Lion Sash.
        for card in &self.battlefield {
            if card.attached_to.is_some() && card.definition.has_reconfigure().is_some() {
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![card.id]),
                    layer: Layer::L4Type,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::RemoveCardType(crate::card::CardType::Creature),
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
        // CR 700.9 — "Modified creatures you control have <keyword>"
        // (Kodama of the West Tree). `IsModified` needs the live battlefield
        // (attachments), so filters mentioning it resolve here into a
        // Specific id list per recompute; `affected_from_requirement` drops
        // them on the static path, so there's no double application.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::GrantKeyword { applies_to, keyword } = &sa.effect
                else {
                    continue;
                };
                let crate::effect::Selector::EachPermanent(req) = applies_to else { continue };
                if !requirement_mentions_modified(req) {
                    continue;
                }
                let ids: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        self.evaluate_requirement_static(
                            req,
                            &Target::Permanent(c.id),
                            card.controller,
                            Some(card.id),
                        )
                    })
                    .map(|c| c.id)
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
        // CR 702.183 — Impending: a permanent with the Impending keyword isn't
        // a creature while it has a time counter. Emit a layer-4
        // RemoveCardType(Creature) self-effect while counters remain.
        for card in &self.battlefield {
            let is_impending = card
                .definition
                .keywords
                .iter()
                .any(|k| matches!(k, crate::card::Keyword::Impending(_)));
            if is_impending && card.counter_count(crate::card::CounterType::Time) > 0 {
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
        // "As long as [condition], this creature gets +P/+T and has [keyword]."
        // (`StaticEffect::PumpSelfIf`) — evaluate the gating predicate live
        // against the source and, while it holds, emit a layer-7 pump plus an
        // optional keyword grant.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpSelfIf {
                    condition,
                    power,
                    toughness,
                    keywords,
                } = &sa.effect
                else {
                    continue;
                };
                let ctx = crate::game::effects::EffectContext::for_ability(
                    card.id,
                    card.controller,
                    None,
                );
                if !self.evaluate_predicate(condition, &ctx) {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(*power, *toughness),
                });
                for kw in keywords {
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
        }
        // "As long as [condition], [creatures the selector picks] get +P/+T."
        // (`StaticEffect::PumpTeamIf`) — the conditional team anthem. Evaluate
        // the gate against the source; while it holds, emit a layer-7 pump for
        // every permanent the selector resolves to (e.g. Beastmaster Ascension
        // at 7+ quest counters → all your creatures +5/+5).
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpTeamIf {
                    condition,
                    applies_to,
                    power,
                    toughness,
                } = &sa.effect
                else {
                    continue;
                };
                let ctx = crate::game::effects::EffectContext::for_ability(
                    card.id,
                    card.controller,
                    None,
                );
                if !self.evaluate_predicate(condition, &ctx) {
                    continue;
                }
                if let Some(affected) = selector_to_affected(applies_to, card) {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.id.0 as u64,
                        source: card.id,
                        affected,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(*power, *toughness),
                    });
                }
            }
        }
        // Chosen-type tribal anthem (`StaticEffect::AnthemForChosenType`) —
        // pumps the controller's creatures of the type named at the source's
        // ETB (`CardInstance.chosen_creature_type`). Adaptive Automaton,
        // Patchwork Banner.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::AnthemForChosenType { power, toughness, exclude_source } =
                    &sa.effect
                else {
                    continue;
                };
                let Some(ct) = card.chosen_creature_type else { continue };
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::AllWithCreatureType {
                        controller: Some(card.controller),
                        creature_type: ct,
                        exclude_source: *exclude_source,
                    },
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(*power, *toughness),
                });
            }
        }
        // CR 604.3 — characteristic-defining dynamic P/T injection. The
        // formula lives on `CardDefinition.dynamic_pt`; we resolve it here
        // on every layer recompute and emit a layer-7 SetPT effect.
        let goyf_n = self.distinct_card_types_in_all_graveyards() as i32;
        let lands_in_gys: i32 = self.players.iter()
            .map(|p| p.graveyard.iter().filter(|c| c.definition.is_land()).count() as i32)
            .sum();
        for card in &self.battlefield {
            let Some(formula) = card.definition.dynamic_pt else { continue };
            let (power, toughness) = match formula {
                crate::card::DynamicPt::DistinctTypesInAllGraveyards => {
                    (goyf_n, goyf_n + 1)
                }
                crate::card::DynamicPt::ControllerGraveyardSize => {
                    let n = self.players[card.controller].graveyard.len() as i32;
                    (n, n)
                }
                crate::card::DynamicPt::BasePlusOpponentGraveyards { base, creatures_only } => {
                    let n: i32 = self
                        .opponents_of(card.controller)
                        .iter()
                        .map(|&o| {
                            self.players[o]
                                .graveyard
                                .iter()
                                .filter(|c| !creatures_only || c.definition.is_creature())
                                .count() as i32
                        })
                        .sum();
                    (base + n, base + n)
                }
                crate::card::DynamicPt::BasePlusLandsInAllGraveyards { base_p, base_t } => {
                    (base_p + lands_in_gys, base_t + lands_in_gys)
                }
                crate::card::DynamicPt::BasePlusLandsInControllerGraveyard { base_p, base_t } => {
                    let n = self.players[card.controller].graveyard.iter()
                        .filter(|c| c.definition.is_land()).count() as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::BaseMinusControllerLife { base_p, base_t } => {
                    let life = self.players[card.controller].life;
                    (base_p - life, base_t - life)
                }
                crate::card::DynamicPt::ColorlessCreaturesControlled { base_t } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller
                            && c.definition.is_creature()
                            && is_colorless_by_cost(&c.definition)
                    }).count() as i32;
                    (n, base_t)
                }
                crate::card::DynamicPt::CreaturesControlled { base } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_creature()
                    }).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::CreaturesOfTypeControlled { creature_type } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller
                            && c.definition.is_creature()
                            && (c.definition.subtypes.creature_types.contains(&creature_type)
                                || c.has_keyword(&crate::card::Keyword::Changeling))
                    }).count() as i32;
                    (n, n)
                }
                crate::card::DynamicPt::LandsControlled { base } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_land()
                    }).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::ArtifactsControlled { base } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_artifact()
                    }).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::InstantsSorceriesInGraveyardAndExile { base_t } => {
                    let gy = &self.players[card.controller].graveyard;
                    let is_is = |c: &CardInstance| c.definition.is_instant() || c.definition.is_sorcery();
                    let n = gy.iter().filter(|c| is_is(c)).count() as i32
                        + self.exile.iter()
                            .filter(|c| c.owner == card.controller && is_is(c))
                            .count() as i32;
                    (n, base_t)
                }
                crate::card::DynamicPt::ExiledWithSourcePt { base_p, base_t } => self
                    .exile
                    .iter()
                    .find(|c| c.exiled_with == Some(card.id) && c.definition.is_creature())
                    .map(|c| (c.definition.base_power(), c.definition.base_toughness()))
                    .unwrap_or((base_p, base_t)),
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
                        color: None,
                        token: None,
                        colorless: false,
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
                            color: None,
                            token: None,
                        colorless: false,
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
                            color: None,
                            token: None,
                        colorless: false,
                        },
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(p, t),
                    });
                }
            }
            // CR 702.98 — Unleash's second static: a creature with the
            // Unleash keyword can't block while it has a +1/+1 counter.
            // Injected as a computed `CantBlock` so the existing block-
            // legality enforcement (`declare_blockers`) honors it.
            if card.definition.keywords.contains(&Keyword::Unleash)
                && card.counters.get(&crate::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0) > 0
            {
                all_effects.push(ContinuousEffect {
                    timestamp: card.id.0 as u64,
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(Keyword::CantBlock),
                });
            }
            // CR 701.60 — a suspected creature has menace and can't block.
            // Injected as computed keywords so combat-legality enforcement
            // honors them.
            if card.suspected && card.definition.is_creature() {
                for kw in [Keyword::Menace, Keyword::CantBlock] {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.id.0 as u64,
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw),
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
                                color: None,
                                token: None,
                        colorless: false,
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

    /// CR 702.16 — true if `target` has protection from any of `source`'s
    /// (computed) colors. Reads both sides through the layer system so granted
    /// protection / color-setting effects count. Backs damage prevention
    /// (702.16e) and equip/attach legality (702.16f).
    pub(crate) fn is_protected_from(&self, source: CardId, target: CardId) -> bool {
        self.damage_prevented_by_protection(source, target)
    }

    /// CR 702.16e — damage from a source is prevented if the target permanent
    /// has protection from any of that source's (computed) colors. Reads both
    /// sides through the layer system so granted protection / color-setting
    /// effects count.
    pub(crate) fn damage_prevented_by_protection(&self, source: CardId, target: CardId) -> bool {
        let Some(tgt) = self.computed_permanent(target) else { return false };
        let src_colors = self
            .computed_permanent(source)
            .map(|c| c.colors)
            .unwrap_or_else(|| {
                self.battlefield_find(source)
                    .map(|c| c.definition.cost.colors())
                    .unwrap_or_default()
            });
        tgt.keywords.iter().any(|kw| {
            matches!(kw, Keyword::Protection(color) if src_colors.contains(color))
        })
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

    /// Revert temporary control changes (Act of Treason / Threaten) whose
    /// `Duration` is in `which`. The stolen permanent returns to whoever
    /// controlled it immediately before the steal. Entries whose card has
    /// since left the battlefield are dropped without effect (CR 800.4 —
    /// the control-changing effect simply ends).
    pub(crate) fn revert_temporary_control(&mut self, which: &[crate::effect::Duration]) {
        let mut kept = Vec::new();
        for tc in std::mem::take(&mut self.temporary_control) {
            let on_battlefield = self.battlefield.iter().any(|c| c.id == tc.card);
            if !on_battlefield {
                continue; // card left play — nothing to revert
            }
            if which.contains(&tc.duration) {
                if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == tc.card) {
                    c.controller = tc.original_controller;
                }
            } else {
                kept.push(tc);
            }
        }
        self.temporary_control = kept;
    }

    /// CR 707 / 611.2c — a "becomes a copy" effect ends when the object
    /// leaves the battlefield: restore the pre-copy definition on the
    /// departing card and drop its pending revert entries. Called from the
    /// battlefield-leave funnels alongside `turn_face_up`.
    pub(crate) fn revert_copy_on_leave(&mut self, card: &mut crate::card::CardInstance) {
        // The oldest entry holds the original printed definition.
        if let Some(pos) = self.temporary_copies.iter().position(|tc| tc.card == card.id) {
            if let Some(def) = self.temporary_copies[pos].original_def() {
                card.definition = def;
            }
            self.temporary_copies.retain(|tc| tc.card != card.id);
        }
    }

    /// Revert temporary "becomes a copy" definition swaps
    /// (`Effect::BecomeCopyOfFor`) whose `Duration` is in `which`. Reverted
    /// in reverse order so stacked copies unwind to the oldest original.
    /// Entries whose card left the battlefield are dropped (the copy effect
    /// ended with the object).
    pub(crate) fn revert_temporary_copies(&mut self, which: &[crate::effect::Duration]) {
        let mut kept = Vec::new();
        for tc in std::mem::take(&mut self.temporary_copies).into_iter().rev() {
            if !self.battlefield.iter().any(|c| c.id == tc.card) {
                continue; // card left play — nothing to revert
            }
            if which.contains(&tc.duration) {
                if let Some(def) = tc.original_def()
                    && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == tc.card)
                {
                    c.definition = def;
                }
            } else {
                kept.push(tc);
            }
        }
        kept.reverse();
        self.temporary_copies = kept;
    }

    /// Expire all `UntilEndOfCombat` continuous effects (CR 511.2 —
    /// "Effects that last 'until end of combat' expire at the end of the
    /// combat phase"). Invoked from `do_combat_end` once the end-of-
    /// combat step finishes.
    pub(crate) fn expire_end_of_combat_effects(&mut self) {
        self.continuous_effects
            .retain(|e| e.duration != EffectDuration::UntilEndOfCombat);
    }

    /// Sacrifice/exile Mobilize/Myriad tokens registered by
    /// `Effect::CreateTokenAttacking` as the combat phase ends (CR 511.3).
    pub(crate) fn process_attacking_token_cleanup(&mut self) -> Vec<GameEvent> {
        use crate::effect::AttackingTokenCleanup;
        let mut events = Vec::new();
        for (id, kind) in std::mem::take(&mut self.attacking_token_cleanup) {
            if !self.battlefield.iter().any(|c| c.id == id) {
                continue; // already gone (died in combat, bounced, etc.)
            }
            let who = self.battlefield_find(id).map(|c| c.controller).unwrap_or(0);
            match kind {
                AttackingTokenCleanup::SacrificeAtEndOfCombat => {
                    // Shared sacrifice funnel — die snapshot included.
                    self.sacrifice_one(id, who, &mut events);
                }
                AttackingTokenCleanup::ExileAtEndOfCombat => {
                    self.remove_from_battlefield_to_exile(id);
                }
                AttackingTokenCleanup::None => {}
            }
        }
        events.append(&mut self.check_state_based_actions());
        events
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

    /// Put a card onto the battlefield through the real ETB movement funnel
    /// (`move_card_to`), so enters-with-counters / chosen-type / fading
    /// replacements and self-source ETB triggers all fire — unlike
    /// `add_card_to_battlefield`, which pushes directly. Test fixture for
    /// exercising entry-replacement statics (Metallic Mimic).
    pub fn move_card_to_battlefield_for_test(
        &mut self,
        player_idx: usize,
        def: CardDefinition,
    ) -> CardId {
        let id = self.next_id();
        self.players[player_idx].graveyard.push(CardInstance::new(id, def, player_idx));
        let ctx = crate::game::effects::EffectContext::for_ability(id, player_idx, None);
        let mut events = Vec::new();
        self.move_card_to(
            id,
            &crate::effect::ZoneDest::Battlefield { controller: crate::effect::PlayerRef::You, tapped: false },
            &ctx,
            &mut events,
        );
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

    /// Put a card into the exile zone owned by `player_idx` (convenience for
    /// tests — e.g. seeding an opponent-owned card to be processed).
    pub fn add_card_to_exile(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.exile.push(CardInstance::new(id, def, player_idx));
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
    /// CR 510.1c — attackers that became blocked this combat (they stay
    /// blocked even if every blocker has since left combat).
    pub fn blocked_attackers(&self) -> &[CardId] {
        &self.blocked_attackers
    }

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
        let computed = self.compute_battlefield();
        let blocker_computed = computed.iter().find(|c| c.id == blocker_id);
        let Some(blocker_cp) = blocker_computed else {
            return false;
        };
        // CR 509.1a — creature-ness from the computed view (animated lands /
        // crewed Vehicles can block).
        if !blocker_cp.card_types.contains(&crate::card::CardType::Creature) || blocker.tapped {
            return false;
        }
        // Honor `Keyword::CantBlock` from the computed keyword set —
        // transient grants from pump spells (Duel Tactics) and static
        // restrictions (Postmortem Professor) both surface here.
        if blocker_cp.keywords.contains(&Keyword::CantBlock) {
            return false;
        }
        if blocker_cp.keywords.contains(&Keyword::CantAttackOrBlockUnlessEvenCounters)
            && blocker.counters.values().sum::<u32>() % 2 != 0
        {
            return false;
        }
        self.attacking.iter().any(|atk| {
            let attacker = self.battlefield.iter().find(|c| c.id == atk.attacker);
            let atk_cp = computed.iter().find(|c| c.id == atk.attacker);
            let atk_kws = atk_cp.map(|c| c.keywords.as_slice()).unwrap_or(&[]);
            let atk_colors = atk_cp.map(|c| c.colors.as_slice()).unwrap_or(&[]);
            let atk_power = atk_cp.map(|c| c.power)
                .or_else(|| attacker.map(|a| a.power()))
                .unwrap_or(0);
            attacker.is_some()
                && can_block_attacker_computed(
                    blocker, blocker_cp, atk_kws, atk_colors, atk_power,
                )
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
        if !blocker_cp.card_types.contains(&crate::card::CardType::Creature) || blocker.tapped {
            return false;
        }
        if blocker_cp.keywords.contains(&Keyword::CantBlock) {
            return false;
        }
        if blocker_cp.keywords.contains(&Keyword::CantAttackOrBlockUnlessEvenCounters)
            && blocker.counters.values().sum::<u32>() % 2 != 0
        {
            return false;
        }
        let atk_cp = computed.iter().find(|c| c.id == attacker_id);
        let atk_kws = atk_cp.map(|c| c.keywords.as_slice()).unwrap_or(&[]);
        let atk_colors = atk_cp.map(|c| c.colors.as_slice()).unwrap_or(&[]);
        let atk_power = atk_cp.map(|c| c.power).unwrap_or_else(|| attacker.power());
        can_block_attacker_computed(blocker, blocker_cp, atk_kws, atk_colors, atk_power)
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
        // Revel in Silence-style lock: a silenced player can't cast spells
        // or activate loyalty abilities this turn. Gated here so every
        // Cast* action variant is covered at once.
        if action.is_cast_or_loyalty()
            && self.players[self.priority.player_with_priority].silenced_this_turn
        {
            return Err(GameError::SilencedThisTurn);
        }
        // Rule of Law-style one-spell-per-turn lock — gated here so every
        // Cast* variant is covered at once.
        if action.is_cast()
            && self.players[self.priority.player_with_priority].spells_cast_this_game_turn >= 1
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::OneSpellPerTurn)
                })
            })
        {
            return Err(GameError::SpellLimitReached);
        }
        // Voice of Victory — the active player's opponents can't cast spells
        // during that player's turn.
        if action.is_cast() {
            let caster = self.priority.player_with_priority;
            let active = self.active_player_idx;
            let locked = caster != active
                && !self.same_team(caster, active)
                && self.battlefield.iter().any(|c| {
                    c.controller == active
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::OpponentsCantCastDuringYourTurn
                            )
                        })
                });
            if locked {
                return Err(GameError::SilencedThisTurn);
            }
        }
        let events = match action {
            GameAction::PlayLand(id) => self.play_land(id),
            GameAction::PlayLandBack(id) => self.play_land_with_face(id, true),
            GameAction::PlayLandFromGraveyard(id) => self.play_land_from_graveyard(id),
            GameAction::CompanionToHand(card_id) => self.companion_to_hand(card_id),
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
            GameAction::CastSpellEntwine {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_entwine(card_id, target, additional_targets, mode, x_value),
            GameAction::CastBestow {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_bestow(card_id, target, additional_targets, mode, x_value),
            GameAction::CastRoomDoor { card_id, right } => self.cast_room_door(card_id, right),
            GameAction::UnlockRoomDoor { card_id, right } => self.unlock_room_door(card_id, right),
            GameAction::Suspend { card_id } => self.suspend_card(card_id),
            GameAction::Foretell { card_id } => self.foretell_card(card_id),
            GameAction::CastFaceDown { card_id } => self.cast_face_down(card_id),
            GameAction::TurnFaceUp { card_id } => self.turn_face_up_action(card_id),
            GameAction::CastForetold {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_foretold(card_id, target, additional_targets, mode, x_value),
            GameAction::CastAdventure {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_adventure(card_id, target, additional_targets, mode, x_value),
            GameAction::CastAdventureCreature {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_adventure_creature(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSplitRight {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_split_half(card_id, target, additional_targets, mode, x_value, false),
            GameAction::CastSplitFused {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_split_half(card_id, target, additional_targets, mode, x_value, true),
            GameAction::CastAftermath {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_aftermath(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellCasualty {
                card_id,
                sacrifice,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_casualty(card_id, sacrifice, target, additional_targets, mode, x_value),
            GameAction::CastSpellSquad {
                card_id,
                times,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_squad(card_id, times, target, additional_targets, mode, x_value),
            GameAction::CastSpellMultikicked {
                card_id,
                times,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_multikicked(card_id, times, target, additional_targets, mode, x_value),
            GameAction::CastSpellReplicate {
                card_id,
                times,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_replicate(card_id, times, target, additional_targets, mode, x_value),
            GameAction::CastSpellSacrificeReduce {
                card_id,
                sacrifices,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_sacrifice_reduce(
                card_id, sacrifices, target, additional_targets, mode, x_value,
            ),
            GameAction::CastSpellBargain {
                card_id,
                sacrifice,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_bargain(
                card_id, sacrifice, target, additional_targets, mode, x_value,
            ),
            GameAction::Plot { card_id } => self.plot_card(card_id),
            GameAction::CastPlotted {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_plotted(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellConvoke {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
                convoke_creatures,
            } => self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &convoke_creatures, &[], crate::game::actions::CastFlags::default()),
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
            GameAction::CastDisturb { card_id } => self.cast_disturb(card_id),
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
                x_value,
            } => self.activate_loyalty_ability(card_id, ability_index, target, x_value),
            GameAction::DeclareAttackers(ids) => self.declare_attackers(ids),
            GameAction::DeclareBlockers(assignments) => self.declare_blockers(assignments),
            GameAction::PassPriority => self.pass_priority(),
            GameAction::SubmitDecision(_) => unreachable!(),
            GameAction::Cycle { card_id } => self.cycle_card(card_id),
            GameAction::Reinforce { card_id, target } => self.reinforce_card(card_id, target),
            GameAction::Landcycle { card_id } => self.landcycle_card(card_id),
            GameAction::Equip { equipment, target } => self.equip(equipment, target),
            GameAction::Reconfigure { equipment, target } => {
                self.reconfigure(equipment, target)
            }
            GameAction::Crew { vehicle, crew_creatures } => self.crew(vehicle, &crew_creatures),
            GameAction::Saddle { mount, creatures } => self.saddle(mount, &creatures),
            GameAction::Ninjutsu { ninja, returning } => self.ninjutsu(ninja, returning),
            // Fallback attribution for direct (non-networked) callers — the
            // server intercepts `Concede` in `handle_action` and routes it to
            // the *sending* seat via `concede`, bypassing this path entirely.
            GameAction::Concede => Ok(self.concede(self.active_player_idx)),
        }?;
        self.dispatch_triggers_for_events(&events);
        Ok(events)
    }

    /// CR 104.3a — `seat` concedes and leaves the game immediately. Legal at
    /// any time, regardless of priority, so this does *not* go through the
    /// priority-gated action path. Marks the player eliminated, removes the
    /// objects that leave with them (CR 800.4a), then runs state-based
    /// actions, which resolve the win/draw for the remaining team(s).
    ///
    /// No-ops (returns no events) if `seat` is out of range, already
    /// eliminated, or the game is already over.
    pub fn concede(&mut self, seat: usize) -> Vec<GameEvent> {
        if seat >= self.players.len()
            || self.players[seat].eliminated
            || self.game_over.is_some()
        {
            return Vec::new();
        }
        self.players[seat].eliminated = true;
        let mut events = vec![GameEvent::PlayerConceded { player: seat }];
        // CR 800.4a — the conceding player's objects leave with them. SBAs
        // skip already-eliminated seats, so this won't fire for them there.
        self.objects_leave_with_player(seat);
        // Resolve the game-over / surviving-team determination.
        events.extend(self.check_state_based_actions());
        events
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
        let was_nonland = !card.definition.card_types.contains(&crate::card::CardType::Land);
        let madness = card.definition.madness_cost().cloned();

        // The discard happens regardless of the destination zone (CR
        // 701.8b), so emit the event + bump the discard-matters counters
        // up front, before resolving the Madness replacement.
        events.push(GameEvent::CardDiscarded { player: p, card_id });
        self.players[p].cards_discarded_this_turn =
            self.players[p].cards_discarded_this_turn.saturating_add(1);
        self.cards_discarded_this_resolution += 1;
        *self
            .cards_discarded_per_player_this_resolution
            .entry(p)
            .or_insert(0) += 1;
        self.discarded_card_ids_this_resolution.push(card_id);
        if was_creature {
            self.creature_cards_discarded_this_resolution += 1;
        }
        if was_nonland {
            *self
                .nonland_cards_discarded_per_player_this_resolution
                .entry(p)
                .or_insert(0) += 1;
        }

        let is_land = !was_nonland;
        match madness {
            None => {
                // CR 614.6 — through the graveyard funnel so Rest in Peace /
                // Leyline hate redirects the discard to exile.
                self.route_to_graveyard(card, events);
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
                        events.push(GameEvent::CardPutIntoGraveyard {
                            player: owner, card_id, is_land,
                        });
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
        // Locate the card in `seat`'s hand and clone the cycling cost —
        // mana (`Cycling`) or life ("Cycling—Pay 2 life", `CyclingLife`).
        let (cycling_cost, life_cost) = self.players[seat]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition.keywords.iter().find_map(|kw| match kw {
                    Keyword::Cycling(mc) => Some((Some(mc.clone()), 0)),
                    Keyword::CyclingLife(n) => Some((None, *n)),
                    _ => None,
                })
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        if life_cost > 0 && self.players[seat].life < life_cost as i32 {
            return Err(GameError::InsufficientLife);
        }
        // Pay the cycling cost from the floated mana pool.
        if let Some(mc) = &cycling_cost {
            self.players[seat].mana_pool.pay(mc).map_err(GameError::Mana)?;
        }
        if life_cost > 0 {
            self.adjust_life(seat, -(life_cost as i32));
        }
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

    /// CR 702.77 — Activate a Reinforce ability from the hand. Pays the cost,
    /// discards the card (firing discard triggers), then puts N +1/+1 counters
    /// on the targeted creature.
    fn reinforce_card(
        &mut self,
        card_id: crate::card::CardId,
        target: crate::game::types::Target,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::{CounterType, Keyword};
        let seat = self.player_with_priority();
        let (cost, n) = self.players[seat]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition.keywords.iter().find_map(|kw| match kw {
                    Keyword::Reinforce(n, mc) => Some((mc.clone(), *n)),
                    _ => None,
                })
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Target must be a creature on the battlefield.
        let crate::game::types::Target::Permanent(tid) = target else {
            return Err(GameError::InvalidTarget);
        };
        if !self
            .battlefield
            .iter()
            .any(|c| c.id == tid && c.definition.is_creature())
        {
            return Err(GameError::InvalidTarget);
        }
        self.players[seat].mana_pool.pay(&cost).map_err(GameError::Mana)?;
        let mut events = vec![];
        self.discard_card(seat, card_id, &mut events);
        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == tid) {
            c.add_counters(CounterType::PlusOnePlusOne, n);
        }
        Ok(events)
    }

    /// CR 702.29e — Activate a Landcycling ability. Pays the cost, discards the
    /// card (firing cycle/discard triggers, since typecycling *is* a cycling
    /// ability), then searches the library for a land of the keyword's land
    /// type and puts it into hand (shuffling after). The fetched land is the
    /// first matching card (a minor approximation for the rare multi-match
    /// case — usually a basic land).
    fn landcycle_card(&mut self, card_id: crate::card::CardId) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Keyword;
        use rand::seq::SliceRandom;
        let seat = self.player_with_priority();
        let (cycling_cost, land_type) = self.players[seat]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition.keywords.iter().find_map(|kw| match kw {
                    Keyword::Landcycling(mc, lt) => Some((mc.clone(), *lt)),
                    _ => None,
                })
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        self.players[seat].mana_pool.pay(&cycling_cost).map_err(GameError::Mana)?;
        let mut events = vec![];
        if self.discard_card(seat, card_id, &mut events) {
            events.push(GameEvent::CardCycled { player: seat, card_id });
        }
        // Search the library for a land of the named type; reveal + to hand.
        if let Some(pos) = self.players[seat]
            .library
            .iter()
            .position(|c| c.definition.is_land() && c.definition.subtypes.land_types.contains(&land_type))
        {
            let fetched = self.players[seat].library.remove(pos);
            self.place_card_in_dest(
                fetched,
                seat,
                &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::Seat(seat)),
                &mut events,
            );
        }
        self.players[seat].library.shuffle(&mut rand::rng());
        Ok(events)
    }

    /// CR 104.3c, with the 104.2 override — a failed draw from an empty
    /// library eliminates `p`, unless they control a "you win the game
    /// instead" static (Laboratory Maniac, Jace, Wielder of Mysteries):
    /// then every other player is eliminated and the SBA pass promotes the
    /// win.
    pub(crate) fn lose_to_empty_draw(&mut self, p: usize) {
        let wins = self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::WinInsteadOfDrawFromEmpty
                    )
                })
        });
        if wins {
            for (idx, pl) in self.players.iter_mut().enumerate() {
                if idx != p {
                    pl.eliminated = true;
                }
            }
        } else {
            self.players[p].eliminated = true;
        }
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
        let drew = match self.players[p].draw_top() {
            Some(id) => {
                events.push(GameEvent::CardDrawn { player: p, card_id: id });
                self.maybe_grant_miracle(p, id);
                true
            }
            None => false,
        };
        // CR 121.2a / 614 — "If you would draw a card, draw two instead"
        // (Thought Reflection). Each doubler applies once per draw event
        // (n doublers: 1 → 2^n); the replacement draws themselves aren't
        // re-doubled (CR 614.5), enforced by the reentrancy flag.
        if drew && !self.in_draw_double {
            let doublers = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(sa.effect, crate::effect::StaticEffect::ControllerDrawsDoubled)
                        })
                })
                .count() as u32;
            if doublers > 0 {
                self.in_draw_double = true;
                for _ in 0..(1u32 << doublers.min(8)) - 1 {
                    self.draw_one(p, events);
                }
                self.in_draw_double = false;
            }
        }
        drew
    }

    /// CR 702.94 — Miracle. If `card_id` was the first card `p` drew this
    /// turn and it has a printed miracle cost, grant the miracle alt-cost
    /// until end of turn (the owner may then cast it for that cost via
    /// `GameAction::CastFromZoneWithoutPaying`). The reveal is treated as
    /// automatic — the grant only adds a cheaper *option*, so revealing is
    /// never a downside for the engine; a human simply declines to cast.
    pub(crate) fn maybe_grant_miracle(&mut self, p: usize, card_id: CardId) {
        if self.players[p].cards_drawn_this_turn != 1 {
            return;
        }
        if let Some(card) = self.players[p].hand.iter_mut().find(|c| c.id == card_id)
            && let Some(cost) = card.definition.miracle.clone()
        {
            card.may_play_until = Some(crate::card::MayPlayPermission {
                player: p,
                granted_turn: self.turn_number,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                exile_after: false,
            });
            card.granted_alt_cast_cost_eot = Some(cost);
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
        // CR 702.16f — a creature can't be equipped by an Equipment whose
        // color it has protection from.
        if self.is_protected_from(equipment, target) {
            return Err(GameError::TargetHasProtection(target));
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

    /// CR 702.151 — Reconfigure. Pay the reconfigure cost to attach the
    /// Equipment-creature to a creature you control (`Some`), or to unattach
    /// it (`None`). Attach reuses the equip-legality checks; unattach simply
    /// clears the link, restoring its creature-ness (the layer-4
    /// "not a creature while attached" strip keys on `attached_to`).
    /// Sorcery-speed only (CR 702.151c).
    fn reconfigure(
        &mut self,
        equipment: crate::card::CardId,
        target: Option<crate::card::CardId>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let pos = self
            .battlefield
            .iter()
            .position(|c| c.id == equipment)
            .ok_or(GameError::CardNotOnBattlefield(equipment))?;
        if self.battlefield[pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        let cost = self.battlefield[pos]
            .definition
            .has_reconfigure()
            .cloned()
            .ok_or(GameError::NotEquipment(equipment))?;
        match target {
            Some(t) => {
                // Attach: target must be a creature you control (and not the
                // Equipment itself), honoring protection (CR 702.16f).
                let target_ok = t != equipment
                    && self.computed_permanent(t).is_some_and(|c| {
                        c.controller == p
                            && c.card_types.contains(&crate::card::CardType::Creature)
                    });
                if !target_ok {
                    return Err(GameError::InvalidTarget);
                }
                if self.is_protected_from(equipment, t) {
                    return Err(GameError::TargetHasProtection(t));
                }
                self.players[p].mana_pool.pay(&cost).map_err(GameError::Mana)?;
                self.battlefield[pos].attached_to = Some(t);
                Ok(vec![GameEvent::AttachmentMoved {
                    attachment: equipment,
                    attached_to: Some(t),
                }])
            }
            None => {
                // Unattach: only meaningful if currently attached.
                if self.battlefield[pos].attached_to.is_none() {
                    return Err(GameError::InvalidTarget);
                }
                self.players[p].mana_pool.pay(&cost).map_err(GameError::Mana)?;
                self.battlefield[pos].attached_to = None;
                Ok(vec![GameEvent::AttachmentMoved {
                    attachment: equipment,
                    attached_to: None,
                }])
            }
        }
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

    /// CR 702.171 — Saddle a Mount. Taps the listed other untapped creatures
    /// the activator controls (total power ≥ the Mount's saddle number) and
    /// marks the Mount saddled until end of turn. Sorcery speed (CR 702.171a).
    fn saddle(
        &mut self,
        mount: crate::card::CardId,
        creatures: &[crate::card::CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let mount_pos = self
            .battlefield
            .iter()
            .position(|c| c.id == mount)
            .ok_or(GameError::CardNotOnBattlefield(mount))?;
        if self.battlefield[mount_pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        let saddle_n = self.battlefield[mount_pos]
            .definition
            .saddle_cost()
            .ok_or(GameError::InvalidTarget)?;
        let computed = self.compute_battlefield();
        let mut seen = std::collections::HashSet::new();
        let mut total_power: i32 = 0;
        for &cid in creatures {
            if cid == mount || !seen.insert(cid) {
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
        if (total_power as u32) < saddle_n {
            return Err(GameError::SelectionRequirementViolated);
        }
        let mut events = vec![];
        for &cid in creatures {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
                c.tapped = true;
                events.push(GameEvent::PermanentTapped { card_id: cid });
            }
        }
        if let Some(m) = self.battlefield.iter_mut().find(|c| c.id == mount) {
            m.saddled = true;
        }
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
        // CR 702.49a — "unblocked attacker": once blocked it stays blocked
        // for the combat even if its blockers have since left (CR 510.1c).
        if self.block_map.values().any(|&a| a == returning)
            || self.blocked_attackers.contains(&returning)
        {
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
    /// CR 702.95 — when `entered` (a creature) comes onto the battlefield,
    /// pair it with an eligible unpaired creature its controller controls. The
    /// "may" is auto-resolved (pairing is value-positive); the partner with
    /// the lowest CardId is chosen for determinism. A Soulbond creature can
    /// initiate the pair; a non-Soulbond creature only pairs if its controller
    /// already has an unpaired Soulbond creature waiting.
    pub(crate) fn apply_soulbond_pairing(&mut self, entered: CardId) {
        use crate::card::Keyword;
        let Some(card) = self.battlefield_find(entered) else { return };
        if !card.definition.is_creature() || card.soulbond_partner.is_some() {
            return;
        }
        let controller = card.controller;
        let entered_has_soulbond = card.definition.keywords.contains(&Keyword::Soulbond);
        let partner = self
            .battlefield
            .iter()
            .filter(|c| {
                c.id != entered
                    && c.controller == controller
                    && c.definition.is_creature()
                    && c.soulbond_partner.is_none()
                    && (entered_has_soulbond
                        || c.definition.keywords.contains(&Keyword::Soulbond))
            })
            .map(|c| c.id)
            .min_by_key(|id| id.0);
        let Some(p) = partner else { return };
        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == entered) {
            c.soulbond_partner = Some(p);
        }
        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == p) {
            c.soulbond_partner = Some(entered);
        }
    }

    pub(crate) fn dispatch_triggers_for_events(&mut self, events: &[GameEvent]) {
        if events.is_empty() {
            return;
        }
        // CR 702.95 — Soulbond pairing. When a creature enters, attempt to pair
        // it (auto-resolved "may"). Done before trigger dispatch so a paired
        // creature's bonus is live for any subsequent ETB-trigger evaluation.
        // CR 603.4 — stamp the entry turn on every permanent that entered in
        // this batch, so `SelectionRequirement::EnteredThisTurn` (Shaile) can
        // compare against the current turn. Centralized here because every
        // battlefield-entry path emits a `PermanentEntered` event.
        let turn = self.turn_number;
        for e in events {
            if let GameEvent::PermanentEntered { card_id } = e {
                if let Some(c) = self.battlefield_find_mut(*card_id) {
                    c.entered_turn = Some(turn);
                }
                self.apply_soulbond_pairing(*card_id);
            }
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
        // Turn-scoped "whenever a creature you control enters this turn"
        // delayed triggers (CR 603.4 — First Day of Class). Fire once per
        // entering creature controlled by the trigger's controller; the
        // entering creature is the trigger source. These persist (not
        // fires_once) until cleanup.
        let entered_creatures: Vec<(CardId, usize)> = events
            .iter()
            .filter_map(|e| match e {
                GameEvent::PermanentEntered { card_id } => self
                    .battlefield_find(*card_id)
                    .filter(|c| c.definition.is_creature())
                    .map(|c| (*card_id, c.controller)),
                _ => None,
            })
            .collect();
        if !entered_creatures.is_empty() {
            use crate::game::types::DelayedKind;
            let watchers: Vec<crate::game::types::DelayedTrigger> = self
                .delayed_triggers
                .iter()
                .filter(|dt| {
                    matches!(dt.kind, DelayedKind::CreatureYouControlEntersThisTurn)
                })
                .cloned()
                .collect();
            for (cid, controller) in &entered_creatures {
                for dt in &watchers {
                    if dt.controller != *controller {
                        continue;
                    }
                    self.stack.push(crate::game::types::StackItem::Trigger {
                        source: dt.source,
                        controller: dt.controller,
                        effect: Box::new(dt.effect.clone()),
                        target: None,
                        mode: None,
                        x_value: 0,
                        converged_value: 0,
                        trigger_source: Some(crate::game::effects::EntityRef::Permanent(*cid)),
                        mana_spent: 0,
                        event_amount: 0,
                        intervening_if: None,
                    });
                }
            }
        }
        // Phase 1: collect candidate triggers while the borrow on
        // `self.battlefield` is shared. Phase 2 will mutate `self.stack`
        // and call `&self.evaluate_predicate` to gate each candidate by
        // the optional `EventSpec::filter`.
        let mut candidates: Vec<TriggerCandidate> = Vec::new();
        // Hushbringer (CR 614): suppress reaction creature-death triggers
        // ("whenever a creature dies") while a `SuppressCreatureEtbTriggers
        // { also_dies }` static is in play. (Self-death + SBA paths gate
        // separately in `stack.rs`.)
        let dies_suppressed = crate::game::actions::creature_dies_triggers_suppressed(self);
        // Resolve per-permanent layer state once so the dispatcher can
        // honour `Modification::RemoveAllAbilities` (Turn to Frog,
        // Mercurial Transformation, Lignify) — printed triggered abilities
        // are skipped while a strip-abilities effect is in scope per CR
        // 113.10b.
        let computed = self.compute_battlefield();
        // CR 603.3d — keys for `once_per_turn` triggers that fire in this
        // batch; merged into the turn-scoped set after the battlefield walk
        // (deferred so we don't mutate `self` mid-immutable-borrow).
        let mut once_fired_this_batch: std::collections::HashSet<(CardId, usize)> =
            std::collections::HashSet::new();
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
            // Rabid Attack-style "creatures gain '…trigger…' EOT"). Printed
            // triggers carry their definition index so `once_per_turn`
            // (CR 603.3d) can be tracked per (source, index); granted
            // triggers are never once-per-turn and use a sentinel index.
            let n_printed = card.definition.triggered_abilities.len();
            let all_triggers = card
                .definition
                .triggered_abilities
                .iter()
                .enumerate()
                .chain(self.granted_triggers(card.id).iter().map(|t| (usize::MAX, t)));
            for (trig_idx, ta) in all_triggers {
                // CR 603.3d — "triggers only once each turn": skip if it has
                // already fired this turn or earlier in this same batch.
                let once_key = (card.id, trig_idx);
                if ta.event.once_per_turn
                    && trig_idx < n_printed
                    && (self.triggered_once_per_turn_used.contains(&once_key)
                        || once_fired_this_batch.contains(&once_key))
                {
                    continue;
                }
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
                        | crate::effect::EventKind::EnergyGained
                        | crate::effect::EventKind::WonCoinFlip
                        | crate::effect::EventKind::LostCoinFlip
                        | crate::effect::EventKind::RolledDice
                        | crate::effect::EventKind::BecameTarget
                        // Enrage fires once per instance of damage
                        // (CR 702.130a) — fan out across the batch.
                        | crate::effect::EventKind::DealtDamage
                );
                // "Only once each turn" overrides fan-out: a single batch of
                // simultaneous events mints one trigger, not one per event.
                let fanout = fanout && !ta.event.once_per_turn;
                for ev in events {
                    if is_event_hardcoded(ev, &ta.event) {
                        continue;
                    }
                    if dies_suppressed && matches!(ev, GameEvent::CreatureDied { .. }) {
                        continue;
                    }
                    if crate::game::effects::event_matches_spec(self, ev, &ta.event, card) {
                        candidates.push(TriggerCandidate {
                            source: card.id,
                            effect: ta.effect.clone(),
                            controller: card.controller,
                            filter: ta.event.filter.clone(),
                            subject: crate::game::effects::event_subject(ev, &ta.event.kind),
                            event_amount: self.event_amount_for(ev),
                            triggered_by_etb: matches!(ev, GameEvent::PermanentEntered { .. }),
                        });
                        if ta.event.once_per_turn && trig_idx < n_printed {
                            once_fired_this_batch.insert(once_key);
                        }
                        if !fanout {
                            break;
                        }
                    }
                }
            }
        }
        for key in once_fired_this_batch.drain() {
            self.triggered_once_per_turn_used.insert(key);
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
                // SelfSource `DealtDamage` (Enrage on lethal damage) and
                // `PermanentSacrificed` ("when you sacrifice this") both fire
                // from LKI — the source has left the battlefield by dispatch.
                let lki_self = matches!(
                    ta.event.kind,
                    crate::effect::EventKind::DealtDamage
                        | crate::effect::EventKind::PermanentSacrificed
                );
                if !lki_self || ta.event.scope != crate::effect::EventScope::SelfSource {
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
                            event_amount: self.event_amount_for(ev),
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
                    let self_scope = matches!(
                        ta.event.scope,
                        crate::effect::EventScope::SelfSource
                    );
                    // CR 702.29c cycle triggers and "when this card is
                    // milled" triggers both fire off the card in the
                    // graveyard.
                    let cycle_self = matches!(
                        ta.event.kind,
                        crate::effect::EventKind::CardCycled
                    ) && self_scope;
                    let milled_self = matches!(
                        ta.event.kind,
                        crate::effect::EventKind::CardMilled
                    ) && self_scope;
                    // "When this is put into a graveyard from anywhere"
                    // (Emrakul) — also fires off the card in the graveyard.
                    let putgy_self = matches!(
                        ta.event.kind,
                        crate::effect::EventKind::PutIntoGraveyard
                    ) && self_scope;
                    if !from_gy_scope && !cycle_self && !milled_self && !putgy_self {
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
                                event_amount: self.event_amount_for(ev),
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
                                event_amount: self.event_amount_for(ev),
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
        // May suspend on a networked controller's `OrderTriggers` pick
        // (CR 603.3b) — the resume path re-enters
        // `push_ordered_trigger_candidates` with the finished order.
        let Some(candidates) = self.continue_trigger_ordering(Vec::new(), candidates) else {
            return;
        };
        self.push_ordered_trigger_candidates(candidates);
    }

    /// Phase 2 of trigger dispatch: enforce each candidate's
    /// `EventSpec::filter`, expand ETB multipliers/taxes, and drain the
    /// resulting queue onto the stack. Split from
    /// `dispatch_triggers_for_events` so the `OrderTriggers` resume path
    /// can re-enter after a networked controller picks their order.
    pub(crate) fn push_ordered_trigger_candidates(
        &mut self,
        candidates: Vec<TriggerCandidate>,
    ) {
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
                    bargained: false,
                    entwined: false,
                };
                if !self.evaluate_predicate(&filter, &ctx) {
                    continue;
                }
            }
            // CR 700.2b — modal triggered ability mode pick at push-time.
            let mode = self.pick_trigger_mode(&effect, source, controller);
            if triggered_by_etb {
                // Yarok / Elesh Norn replacement (CR 614). A `wants`-side
                // ETB-trigger multiplier scales how many times this
                // reaction trigger fires (0 = suppressed by an opponent's
                // Spotlight, 1 normally, 2+ with a doubler). Self-source ETB
                // triggers go through the hardcoded path in `actions.rs`
                // (also multiplied), so they aren't double-counted here.
                let mult = crate::game::actions::etb_trigger_multiplier(
                    self,
                    controller,
                    subject.as_ref().and_then(|s| s.as_permanent_id()),
                );
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
    /// CR 603.3b — walk contiguous same-controller runs of simultaneous
    /// triggers, letting each `wants_ui` controller pick a stack-push order.
    /// Suspends on `Decision::OrderTriggers` (parking progress in
    /// `ResumeContext::TriggerOrder`) and returns `None`; `submit_decision`
    /// re-enters with the answered run applied. Returns the fully ordered
    /// list otherwise. AutoDecider/bot seats keep the default order.
    pub(crate) fn continue_trigger_ordering(
        &mut self,
        mut ordered: Vec<TriggerCandidate>,
        rest: Vec<TriggerCandidate>,
    ) -> Option<Vec<TriggerCandidate>> {
        let mut i = 0;
        while i < rest.len() {
            let ctrl = rest[i].controller;
            let mut j = i + 1;
            while j < rest.len() && rest[j].controller == ctrl {
                j += 1;
            }
            let run = &rest[i..j];
            // A decision already pending (e.g. a racing combat choice) has
            // nowhere to park this batch — keep the default order, matching
            // `drain_trigger_queue`'s behavior.
            if run.len() < 2
                || !self.players.get(ctrl).is_some_and(|p| p.wants_ui)
                || self.pending_decision.is_some()
            {
                ordered.extend_from_slice(run);
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
                self.pending_decision = Some(PendingDecision {
                    decision: crate::decision::Decision::OrderTriggers {
                        player: ctrl,
                        triggers: labels,
                    },
                    resume: ResumeContext::TriggerOrder {
                        ordered,
                        run: run.to_vec(),
                        rest: rest[j..].to_vec(),
                    },
                });
                return None;
            }
            i = j;
        }
        Some(ordered)
    }

    /// Apply a `TriggerOrder(ids)` answer to `run`: entries named in `ids`
    /// first (in that order), unnamed ones after in original order — a
    /// partial or empty answer is always legal.
    pub(crate) fn apply_trigger_order(
        ordered: &mut Vec<TriggerCandidate>,
        run: Vec<TriggerCandidate>,
        order: Vec<CardId>,
    ) {
        let mut remaining: Vec<Option<TriggerCandidate>> = run.into_iter().map(Some).collect();
        for id in order {
            if let Some(pos) =
                remaining.iter().position(|c| c.as_ref().is_some_and(|c| c.source == id))
            {
                ordered.push(remaining[pos].take().unwrap());
            }
        }
        for slot in remaining.into_iter().flatten() {
            ordered.push(slot);
        }
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
        // already suspended on something else we can't surface a target
        // picker, so the whole batch falls back to auto-targeting (the
        // triggers still hit the stack; they must not vanish).
        let force_auto = self.pending_decision.is_some();
        // Walk the queue in *forward* (APNAP) order so the active
        // player's triggers push first and resolve last, matching CR
        // 603.3b. Using an iterator lets us collect the unconsumed
        // tail into `remaining` when we suspend mid-batch.
        let mut iter = queue.into_iter();
        // Per-copy target choice for doubled triggers (CR 603.3d): track the
        // targets already auto-picked for each source in this batch so an
        // Elesh-Norn-doubled ETB aims its second copy at a fresh target
        // instead of duplicating (and later fizzling on) the first pick.
        let mut picked_this_batch: Vec<(CardId, CardId)> = Vec::new();
        while let Some(pending) = iter.next() {
            // Event-amount-relative target filters (Scrap Trawler's
            // "lesser mana value than that artifact") read this scratch
            // during legal-target enumeration below.
            self.trigger_event_amount_scratch = pending.event_amount;
            let needs = pending.effect.requires_target();
            let wants_ui = !force_auto
                && self
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
            // Prefer a non-source target: an "another target creature" trigger
            // (OtherThanSource) must not auto-pick its own source, and even a
            // plain "target creature" trigger reads better picking a different
            // permanent (a self-target trigger uses `Selector::This`, not a
            // target slot). Falls back to the source if it's the only legal pick.
            let mut avoid = vec![pending.source];
            avoid.extend(
                picked_this_batch
                    .iter()
                    .filter(|(src, _)| *src == pending.source)
                    .map(|(_, t)| *t),
            );
            let auto = self.auto_target_for_effect_avoiding_set(
                &pending.effect,
                pending.controller,
                &avoid,
            );
            if let Some(Target::Permanent(tid)) = &auto {
                picked_this_batch.push((pending.source, *tid));
            }
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
        // CR 603.10 — if this trigger's source just left the battlefield
        // (it's in the die-snapshot cache), stash its last-known instance
        // so a "deals damage equal to its power" body reads the
        // counter/pump-boosted P/T rather than the graveyard's printed
        // value. Removed when the trigger resolves (`resolve_stack_item`).
        if let Some(snap) = self.died_card_snapshots.get(&source) {
            self.leaves_bf_lki.insert(source, snap.clone());
        }
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
        x_value: Option<u32>,
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
        // CR 701.35 — a detained planeswalker's loyalty abilities can't be
        // activated (same gate as the regular activation path).
        if self.battlefield[pos].detained_by.is_some() {
            return Err(GameError::InvalidTarget);
        }
        // CR 606.3 — once per turn, or twice with Urza, Planeswalker's
        // printed override.
        let allowed = if self.battlefield[pos].definition.loyalty_twice_each_turn { 2 } else { 1 };
        if self.battlefield[pos].loyalty_uses_this_turn >= allowed {
            return Err(GameError::LoyaltyAbilityAlreadyUsed(card_id));
        }

        // Printed abilities, plus any granted by another friendly
        // planeswalker's ability-sharing static (Kasmina, Enigma Sage) at
        // indices past the printed count.
        let mut abilities = self.battlefield[pos].definition.loyalty_abilities.clone();
        for c in &self.battlefield {
            if c.id != card_id
                && c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::OtherPlaneswalkersHaveSourceLoyaltyAbilities
                    )
                })
            {
                abilities.extend(c.definition.loyalty_abilities.iter().cloned());
            }
        }
        let ability = abilities
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

        // Apply loyalty cost. CR 606.5: a `-X` ability lets the player pick X
        // (0..=current loyalty); the cost paid is X loyalty and the body reads
        // X via `Value::XFromCost`. Fixed-cost abilities ignore `x_value`.
        let current_loyalty =
            self.battlefield[pos].counter_count(crate::card::CounterType::Loyalty) as i32;
        let x = if ability.x_cost {
            x_value.unwrap_or(0).min(current_loyalty.max(0) as u32)
        } else {
            0
        };
        let loyalty_change = if ability.x_cost { -(x as i32) } else { ability.loyalty_cost };
        let new_loyalty = current_loyalty + loyalty_change;
        if new_loyalty < 0 {
            return Err(GameError::NotEnoughLoyalty(card_id));
        }
        self.battlefield[pos]
            .counters
            .insert(crate::card::CounterType::Loyalty, new_loyalty as u32);
        self.battlefield[pos].loyalty_uses_this_turn =
            self.battlefield[pos].loyalty_uses_this_turn.saturating_add(1);
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
            x_value: x,
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
            ResumeContext::TriggerOrder { mut ordered, run, rest } => {
                // CR 603.3b — apply the controller's chosen order, then
                // continue the ordering walk (which may suspend again on a
                // later same-controller run) and finish the dispatch.
                let order = match answer {
                    DecisionAnswer::TriggerOrder(ids) => ids,
                    _ => return Err(GameError::DecisionAnswerMismatch),
                };
                Self::apply_trigger_order(&mut ordered, run, order);
                if let Some(all) = self.continue_trigger_ordering(ordered, rest) {
                    self.push_ordered_trigger_candidates(all);
                }
                vec![]
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
            ResumeContext::CleanupDiscard { player } => {
                // CR 514.1 — apply the player's chosen discards, then resume
                // the rest of cleanup and the step advance.
                let ids = match &answer {
                    DecisionAnswer::Discard(ids) => ids.clone(),
                    _ => return Err(GameError::DecisionAnswerMismatch),
                };
                let mut evs = Vec::new();
                for id in ids {
                    // CR 514.1 — discard down to, never past, the maximum.
                    // Once the hand is back at the limit, ignore any further
                    // ids so a buggy/hostile client can't force an
                    // over-discard with an oversized answer.
                    let over = self
                        .effective_max_hand_size(player)
                        .is_some_and(|max| self.players[player].hand.len() > max);
                    if !over {
                        break;
                    }
                    if self.players[player].hand.iter().any(|c| c.id == id) {
                        self.discard_card(player, id, &mut evs);
                    }
                }
                if !evs.is_empty() {
                    self.dispatch_triggers_for_events(&evs);
                }
                // Under-discard (the answer pitched too few): re-pose the
                // decision until the hand is back at the maximum.
                if let Some(max) = self.effective_max_hand_size(player)
                    && self.players[player].hand.len() > max
                {
                    let excess = (self.players[player].hand.len() - max) as u32;
                    self.set_cleanup_discard_decision(player, excess);
                    return Ok(evs);
                }
                self.finish_cleanup();
                return self.advance_step(evs);
            }
            ResumeContext::CombatDamage { player: _, attacker, kind } => {
                // CR 510.1c-d — cache the answered ordering/assignment choice,
                // then re-enter the current damage step. It re-runs the (now
                // cached) gather and either suspends on the next choice or
                // applies all combat damage. Mirrors the pass_priority combat
                // arms (give priority + dispatch triggers) on completion.
                self.apply_combat_decision_answer(attacker, kind, &answer);
                let evs = match self.step {
                    TurnStep::FirstStrikeDamage => self.resolve_first_strike_damage()?,
                    TurnStep::CombatDamage => self.resolve_combat()?,
                    _ => Vec::new(),
                };
                if self.pending_decision.is_none() {
                    self.give_priority_to_active();
                    self.dispatch_triggers_for_events(&evs);
                }
                return Ok(evs);
            }
            ResumeContext::CastAdditionalCost {
                caster,
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => {
                // CR 601.2b — the caster paid an additional cost choice. The
                // answer type says which: a permanent target (sacrifice) or a
                // discard list. Validate, stash it for `pay_additional_costs`,
                // and re-run the cast. The cast was suspended before any cost
                // was paid, so re-invoking from the top is a clean replay (no
                // double-spend / double-removal); it may suspend again for a
                // further additional cost.
                match &answer {
                    DecisionAnswer::Target(Target::Permanent(id))
                        if self.cast_sacrifice_choice_is_legal(caster, card_id, *id) =>
                    {
                        self.pending_cast_sacrifices = Some(vec![*id]);
                    }
                    DecisionAnswer::Discard(ids) => {
                        // Trust the option list that was posed (the caster's
                        // hand minus the card being cast); the apply path in
                        // `pay_additional_costs` re-checks each id is in hand.
                        self.pending_cast_discards = Some(ids.clone());
                    }
                    _ => return Err(GameError::DecisionAnswerMismatch),
                }
                // Priority is still the caster's (we never advanced it), so
                // `cast_spell` reads the right actor. Any cost failure (e.g.
                // mana shortfall) surfaces as a normal cast error.
                return self.cast_spell(card_id, target, additional_targets, mode, x_value);
            }
            ResumeContext::ActionFloatConfirm { actor, action } => {
                // CR 601.2g — the payer chose whether to spend floating mana.
                // Stash the choice and replay the exact originating action
                // (priority is still theirs, so it reads the right actor).
                let DecisionAnswer::Bool(spend) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let _ = actor;
                self.pending_cast_spend_float = Some(spend);
                return self.perform_action(*action);
            }
            ResumeContext::ActivateAbilityChoice {
                activator,
                card_id,
                ability_index,
                target,
                x_value,
                kind,
            } => {
                // CR 602.5 — the activator picked how to pay one of the
                // ability's "another …" costs. Stash it for the matching cost
                // and replay the activation from the top — nothing was paid
                // before the suspend, so this is a clean replay (which may
                // suspend again for a further choice). Sacrifice/tap come back
                // as a battlefield `Target`; the graveyard exile as `Cards`.
                use crate::game::types::AbilityCostChoice as K;
                match kind {
                    K::SacOther | K::TapOther => {
                        let DecisionAnswer::Target(Target::Permanent(id)) = answer else {
                            return Err(GameError::DecisionAnswerMismatch);
                        };
                        if id == card_id
                            || self.battlefield_find(id).is_none_or(|c| c.controller != activator)
                        {
                            return Err(GameError::DecisionAnswerMismatch);
                        }
                        if matches!(kind, K::SacOther) {
                            self.pending_ability_sac_other = Some(id);
                        } else {
                            self.pending_ability_tap_other = Some(id);
                        }
                    }
                    K::ExileOther => {
                        let DecisionAnswer::Cards(ids) = answer else {
                            return Err(GameError::DecisionAnswerMismatch);
                        };
                        // Trust the posed option list (the activator's graveyard
                        // minus the source); `activate_ability` re-checks each id
                        // is still in the graveyard and matches the filter.
                        self.pending_ability_exile_other = Some(ids);
                    }
                }
                return self.activate_ability(card_id, ability_index, target, x_value);
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
                            bound_token: None,
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
            PendingEffectState::RearrangePeeked { count, player } => {
                // Index / Spire Owl — every peeked card returns to the top in
                // the chosen order; the `bottom` list is treated as "kept on
                // top, after kept_top" so nothing is ever bottomed.
                let DecisionAnswer::ScryOrder { kept_top, bottom } = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut remaining: Vec<CardInstance> =
                    self.players[player].library.drain(..count).collect();
                let mut top_cards = Vec::with_capacity(count);
                for id in kept_top.iter().chain(bottom.iter()) {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        top_cards.push(remaining.remove(pos));
                    }
                }
                top_cards.extend(remaining);
                let lib = &mut self.players[player].library;
                for c in top_cards.into_iter().rev() {
                    lib.insert(0, c);
                }
                Ok(vec![GameEvent::ScryPerformed { player, looked_at: count, bottomed: 0 }])
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
            PendingEffectState::SearchPending { player, to, eligible } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                if let Some(card_id) = chosen_id
                    && eligible.as_ref().is_none_or(|e| e.contains(card_id))
                    && let Some(pos) = self.players[player].library.iter().position(|c| c.id == *card_id) {
                    // Grafdigger's Cage — a creature card can't leave the
                    // library for the battlefield while the lockdown is up.
                    let blocked = matches!(to, crate::effect::ZoneDest::Battlefield { .. })
                        && self.graveyard_library_locked()
                        && self.players[player].library[pos].definition.is_creature();
                    if !blocked {
                        let card = self.players[player].library.remove(pos);
                        self.place_card_in_dest(card, player, &to, &mut events);
                        // Surface the found card so a downstream `Selector::LastMoved`
                        // can inspect its type (Oriq Loremage's "if instant/sorcery").
                        self.last_moved_cards.push(*card_id);
                    }
                }
                Ok(events)
            }
            PendingEffectState::ImpulsePending { player, revealed, rest_to_graveyard, eligible, take, to_battlefield } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                // `None` eligible means "any revealed card" (no filter).
                let is_eligible = |id: &CardId| match &eligible {
                    None => true,
                    Some(v) => v.contains(id),
                };
                // The decision picks the first card; for take>1 (Consult the
                // Star Charts kicked) the rest auto-fill from the remaining
                // eligible revealed cards. AutoDecider / empty pick takes the
                // first eligible.
                let mut picks: Vec<CardId> = Vec::with_capacity(take);
                if let Some(id) = *chosen_id
                    && revealed.contains(&id)
                    && is_eligible(&id)
                {
                    picks.push(id);
                }
                for id in revealed.iter().copied() {
                    if picks.len() >= take {
                        break;
                    }
                    if is_eligible(&id) && !picks.contains(&id) {
                        picks.push(id);
                    }
                }
                let mut events = vec![];
                for &pick in &picks {
                    if let Some(pos) = self.players[player].library.iter().position(|c| c.id == pick) {
                        let card = self.players[player].library.remove(pos);
                        if to_battlefield {
                            // Collected Company — picks enter the battlefield
                            // (ETBs fire through the shared placement funnel).
                            self.place_card_in_dest(
                                card,
                                player,
                                &crate::effect::ZoneDest::Battlefield {
                                    controller: crate::effect::PlayerRef::Seat(player),
                                    tapped: false,
                                },
                                &mut events,
                            );
                        } else {
                            self.players[player].hand.push(card);
                            events.push(GameEvent::CardDrawn { player, card_id: pick });
                        }
                    }
                }
                // Move the rest of the revealed set to the bottom of the
                // library (or graveyard). They're still at the top of the
                // library after the picks were removed.
                for rid in &revealed {
                    if picks.contains(rid) {
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
                    events.push(GameEvent::ManaAdded { player, color: *c, source: None });
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
                    events.push(GameEvent::ManaAdded { player, color: *c, source: None });
                }
                Ok(events)
            }
            PendingEffectState::SacrificePending { player } => {
                // CR 701.16 — the player chose which permanent(s) to sacrifice.
                // A single sacrifice comes back as a `Target` (in-scene cursor);
                // a multi-sacrifice as `Cards` (the ChooseCards modal).
                let ids: Vec<CardId> = match answer {
                    DecisionAnswer::Target(Target::Permanent(id)) => vec![*id],
                    DecisionAnswer::Cards(ids) => ids.clone(),
                    _ => return Err(GameError::DecisionAnswerMismatch),
                };
                // Trust the option list that was posed (built from the legal
                // candidates), but guard against stale/hostile ids: each must
                // still be a permanent the sacrificing player controls.
                if ids.is_empty()
                    || !ids.iter().all(|id| {
                        self.battlefield_find(*id).is_some_and(|c| c.controller == player)
                    })
                {
                    return Err(GameError::DecisionAnswerMismatch);
                }
                let mut events = Vec::new();
                for id in ids {
                    self.sacrifice_one(id, player, &mut events);
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
            PendingEffectState::ExileChosenFromHandPending { target_player } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::with_capacity(card_ids.len());
                for cid in card_ids {
                    if let Some(pos) =
                        self.players[target_player].hand.iter().position(|c| c.id == *cid)
                    {
                        let card = self.players[target_player].hand.remove(pos);
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: *cid });
                    }
                }
                Ok(events)
            }
            PendingEffectState::HoneFromHandPending { target_player, count } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::new();
                for cid in card_ids {
                    if let Some(pos) =
                        self.players[target_player].hand.iter().position(|c| c.id == *cid)
                    {
                        let mut card = self.players[target_player].hand.remove(pos);
                        card.add_counters(crate::card::CounterType::Hone, count);
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: *cid });
                    }
                }
                Ok(events)
            }
            PendingEffectState::ExileFromHandTaxedPending { target_player, extra_cost } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let turn = self.turn_number;
                let mut events = Vec::new();
                for cid in card_ids {
                    if let Some(pos) =
                        self.players[target_player].hand.iter().position(|c| c.id == *cid)
                    {
                        let mut card = self.players[target_player].hand.remove(pos);
                        let owner = card.owner;
                        // Owner may play it, taxed `extra_cost` more, while exiled.
                        let mut taxed = card.definition.cost.clone();
                        if extra_cost > 0 {
                            taxed.symbols.push(crate::mana::ManaSymbol::Generic(extra_cost));
                        }
                        card.may_play_until = Some(crate::card::MayPlayPermission {
                            player: owner,
                            granted_turn: turn,
                            duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                            exile_after: false,
                        });
                        card.granted_alt_cast_cost_eot = Some(taxed);
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
            PendingEffectState::PutFromZonesPending { player } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                if let Some(cid) = chosen_id {
                    let from_hand =
                        self.players[player].hand.iter().position(|c| c.id == *cid);
                    let from_gy =
                        self.players[player].graveyard.iter().position(|c| c.id == *cid);
                    // Grafdigger's Cage — creature cards in graveyards can't
                    // enter the battlefield (hand picks are unaffected).
                    let gy_blocked = |g: &Self, pos: usize| {
                        g.graveyard_library_locked()
                            && g.players[player].graveyard[pos].definition.is_creature()
                    };
                    let card = match (from_hand, from_gy) {
                        (Some(pos), _) => Some(self.players[player].hand.remove(pos)),
                        (None, Some(pos)) if !gy_blocked(self, pos) => {
                            Some(self.players[player].graveyard.remove(pos))
                        }
                        _ => None,
                    };
                    if let Some(card) = card {
                        let dest = crate::effect::ZoneDest::Battlefield {
                            controller: crate::effect::PlayerRef::Seat(player),
                            tapped: false,
                        };
                        self.place_card_in_dest(card, player, &dest, &mut events);
                    }
                }
                Ok(events)
            }
            PendingEffectState::NameDiscardMatchingPending { who } => {
                let DecisionAnswer::NamedCard(name) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                let matching: Vec<CardId> = self.players[who]
                    .hand
                    .iter()
                    .filter(|c| c.definition.name == name)
                    .map(|c| c.id)
                    .collect();
                for cid in matching {
                    self.discard_card(who, cid, &mut events);
                }
                Ok(events)
            }
            PendingEffectState::NameRevealTopPending { player, count } => {
                let DecisionAnswer::NamedCard(name) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                let revealed: Vec<CardId> = self.players[player]
                    .library
                    .iter()
                    .take(count)
                    .map(|c| c.id)
                    .collect();
                for id in revealed {
                    let Some(pos) = self.players[player].library.iter().position(|c| c.id == id)
                    else {
                        continue;
                    };
                    let matches = self.players[player].library[pos].definition.name == name;
                    let card = self.players[player].library.remove(pos);
                    if matches {
                        self.players[player].hand.push(card);
                    } else {
                        self.route_to_graveyard(card, &mut events);
                    }
                }
                Ok(events)
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
            // ── Stash-and-rerun answers ──────────────────────────────────
            // These five suspend with the *originating effect* re-queued as
            // the continuation; the apply step only validates/sanitises the
            // answer and stashes it for the re-run to consume (see
            // `GameState.stashed_resolution_answer`).
            PendingEffectState::ModesAnswerPending { num_modes } => {
                let DecisionAnswer::Modes(v) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let sane: Vec<u8> =
                    v.iter().copied().filter(|&i| (i as usize) < num_modes).collect();
                self.stashed_resolution_answer = Some(DecisionAnswer::Modes(sane));
                Ok(Vec::new())
            }
            PendingEffectState::ModeAnswerPending { num_modes } => {
                let DecisionAnswer::Mode(i) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let sane = (*i).min(num_modes.saturating_sub(1));
                self.stashed_resolution_answer = Some(DecisionAnswer::Mode(sane));
                Ok(Vec::new())
            }
            PendingEffectState::AmountAnswerPending { max } => {
                let DecisionAnswer::Amount(n) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                self.stashed_resolution_answer = Some(DecisionAnswer::Amount((*n).min(max)));
                Ok(Vec::new())
            }
            PendingEffectState::MayDoAnswerPending => {
                let DecisionAnswer::Bool(b) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                self.stashed_resolution_answer = Some(DecisionAnswer::Bool(*b));
                Ok(Vec::new())
            }
            PendingEffectState::DivisionAnswerPending => {
                let DecisionAnswer::DamageDivision(v) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                // Raw stash — the re-run renormalises wrong length/sum.
                self.stashed_resolution_answer =
                    Some(DecisionAnswer::DamageDivision(v.clone()));
                Ok(Vec::new())
            }
            PendingEffectState::CreatureTypeAnswerPending => {
                let DecisionAnswer::CreatureType(ct) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                self.stashed_resolution_answer = Some(DecisionAnswer::CreatureType(*ct));
                Ok(Vec::new())
            }
        }
    }

    /// Heuristic candidates for a `ChooseCreatureType` decision, rendered by
    /// the client as pick buttons: every creature type on the battlefield,
    /// in any graveyard, or among `chooser`'s own hand and library (their
    /// deck is known to them; opponents' hidden zones are excluded so the
    /// suggestion list can't leak), most frequent first, padded with tribal
    /// staples and capped to keep the modal scannable.
    pub(crate) fn creature_type_suggestions(
        &self,
        chooser: usize,
    ) -> Vec<crate::card::CreatureType> {
        use crate::card::CreatureType;
        let mut counts: std::collections::HashMap<CreatureType, usize> =
            std::collections::HashMap::new();
        let public = self
            .battlefield
            .iter()
            .chain(self.players.iter().flat_map(|p| p.graveyard.iter()));
        let own = self
            .players
            .get(chooser)
            .into_iter()
            .flat_map(|p| p.hand.iter().chain(p.library.iter()));
        for c in public.chain(own) {
            for &ct in &c.definition.subtypes.creature_types {
                *counts.entry(ct).or_insert(0) += 1;
            }
        }
        let mut ranked: Vec<(CreatureType, usize)> = counts.into_iter().collect();
        ranked.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)))
        });
        let mut out: Vec<CreatureType> = ranked.into_iter().map(|(ct, _)| ct).collect();
        for ct in [
            CreatureType::Human,
            CreatureType::Elf,
            CreatureType::Goblin,
            CreatureType::Zombie,
            CreatureType::Merfolk,
            CreatureType::Dragon,
            CreatureType::Angel,
            CreatureType::Demon,
            CreatureType::Soldier,
            CreatureType::Wizard,
        ] {
            if !out.contains(&ct) {
                out.push(ct);
            }
        }
        out.truncate(24);
        out
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
        let effect = override_effect.unwrap_or_else(|| {
            if card.adventuring {
                // CR 715 — resolve the adventure half's effect, not the
                // creature body.
                card.definition
                    .adventure
                    .as_ref()
                    .map(|a| a.effect.clone())
                    .unwrap_or(Effect::Noop)
            } else if let (Some(half), Some(split)) =
                (card.split_cast, card.definition.split.as_ref())
            {
                // CR 709 — resolve the chosen half. Left (0) is the main body;
                // right (1) is `split.right`. Fused (2) resolves the left body
                // here, then the right half runs in a second pass below with
                // its own target (slot 0 of `additional_targets`).
                match half {
                    1 => split.right.effect.clone(),
                    _ => card.definition.effect.clone(),
                }
            } else {
                card.definition.effect.clone()
            }
        });
        // CR 608.2b — a spell whose single target is illegal as it tries to
        // resolve doesn't resolve. Applies when the primary target was a
        // battlefield permanent at cast time (zone-loose filters aimed at
        // graveyard cards are unaffected): the spell fizzles if the target
        // left the battlefield, stopped matching the spell's target filter,
        // or became illegal to target (granted Hexproof/Shroud). Token
        // copies additionally keep the bare filter re-check.
        if additional_targets.is_empty()
            && let Some(t) = &target
        {
            let filter_fails = |g: &Self| {
                effect
                    .target_filter_for_slot_in_mode_kicked(0, Some(mode), card.kicked)
                    .is_some_and(|f| !g.evaluate_requirement_static(f, t, caster, Some(card.id)))
            };
            let fizzled = if card.cast_target_was_battlefield
                && let Target::Permanent(tid) = t
            {
                self.battlefield_find(*tid).is_none()
                    || filter_fails(self)
                    || self.check_target_legality_with_source(t, caster, Some(card.id)).is_err()
            } else {
                card.is_token && filter_fails(self)
            };
            if fizzled {
                // A fizzled token copy ceases to exist (already off the
                // stack); a real card is countered into its owner's
                // graveyard — except a flashbacked/aftermath cast, whose
                // CR 702.34d exile rider applies wherever it leaves the
                // stack, so a fizzle can't make it re-flashbackable.
                let mut events = Vec::new();
                if !card.is_token {
                    if card.cast_via_flashback {
                        self.exile.push(card);
                    } else {
                        self.route_to_graveyard(card, &mut events);
                    }
                }
                return Ok(events);
            }
        } else if card.cast_target_was_battlefield
            && let Some(t0) = &target
        {
            // CR 608.2b — a multi-target spell fizzles only if EVERY target
            // is illegal on resolution; effects already skip individual
            // missing targets. Scoped to battlefield-aimed casts (slot 0 was
            // a battlefield permanent at cast time) so zone-loose multi-
            // target spells (graveyard returns) are unaffected.
            let slot_illegal = |g: &Self, slot: u8, t: &Target| {
                let gone = matches!(t, Target::Permanent(tid)
                    if g.battlefield_find(*tid).is_none());
                let filter_fail = effect
                    .target_filter_for_slot_in_mode_kicked(slot, Some(mode), card.kicked)
                    .is_some_and(|f| !g.evaluate_requirement_static(f, t, caster, Some(card.id)));
                gone
                    || filter_fail
                    || g.check_target_legality_with_source(t, caster, Some(card.id)).is_err()
            };
            let all_illegal = slot_illegal(self, 0, t0)
                && additional_targets
                    .iter()
                    .enumerate()
                    .all(|(i, t)| slot_illegal(self, i as u8 + 1, t));
            if all_illegal {
                let mut events = Vec::new();
                if !card.is_token {
                    if card.cast_via_flashback {
                        self.exile.push(card); // CR 702.34d
                    } else {
                        self.route_to_graveyard(card, &mut events);
                    }
                }
                return Ok(events);
            }
        }
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
        ctx.bargained = card.bargained;
        ctx.entwined = card.entwined;
        // Stamp the resolving spell's identity so source-aware damage
        // replacements (Torbran) can read its controller/colors while the
        // card is in no visible zone.
        let prev_src = self.resolving_source.replace((
            card.id,
            caster,
            card.definition.printed_colors(),
        ));
        let res = self.resolve_effect(&effect, &ctx);
        self.resolving_source = prev_src;
        let mut events = res?;
        // CR 709 / 702.102 — a fused split cast resolves its right half in a
        // second pass, reading its target from `additional_targets` slot 0
        // (the left half consumed `target`). Fusable halves are single-target.
        if card.split_cast == Some(2)
            && let Some(split) = card.definition.split.as_ref()
        {
            let right_effect = split.right.effect.clone();
            let right_ctx = EffectContext::for_spell_with_source_and_origin(
                card.id,
                card.definition.name,
                caster,
                additional_targets.first().cloned(),
                Vec::new(),
                mode,
                x_value,
                converged_value,
                mana_spent,
                card.cast_from_hand,
            );
            let mut right_events = self.resolve_effect(&right_effect, &right_ctx)?;
            events.append(&mut right_events);
        }
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
                bound_token: None,
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
        // CR 702.46 — Cipher. `Effect::Cipher` set `cipher_encode_pending` to
        // the creature this spell should be encoded on. Route the card to exile
        // with `encoded_on` stamped instead of the graveyard.
        if let Some(creature) = self.cipher_encode_pending.take() {
            let mut card = card;
            card.encoded_on = Some(creature);
            self.exile.push(card);
            return Ok(events);
        }
        // Beacon cycle: "Shuffle this card into its owner's library."
        // `Effect::ShuffleSelfIntoLibrary` flagged the resolving spell — route
        // it to its owner's library and shuffle instead of the graveyard.
        if self.shuffle_resolving_spell_into_library {
            self.shuffle_resolving_spell_into_library = false;
            use rand::seq::SliceRandom;
            let owner = card.owner;
            self.players[owner].library.push(card);
            self.players[owner].library.shuffle(&mut rand::rng());
            return Ok(events);
        }
        // Revel in Silence's "exile this" rider.
        if self.exile_resolving_spell {
            self.exile_resolving_spell = false;
            self.exile.push(card);
            return Ok(events);
        }
        // Journey to the Oracle's "return this to its owner's hand" rider.
        if self.return_resolving_spell_to_hand {
            self.return_resolving_spell_to_hand = false;
            let owner = card.owner;
            self.players[owner].hand.push(card);
            return Ok(events);
        }
        // Buyback (CR 702.27e): a spell cast paying its buyback cost returns
        // to its owner's hand instead of the graveyard as it resolves.
        if card.bought_back {
            let owner = card.owner;
            self.players[owner].hand.push(card);
            return Ok(events);
        }
        // CR 702.127e — an Aftermath half (right half cast from the graveyard)
        // is exiled on resolution rather than returning to the graveyard.
        if card.split_cast == Some(1)
            && card.definition.split.as_ref().is_some_and(|s| s.aftermath)
        {
            self.exile.push(card);
            return Ok(events);
        }
        // CR 715 — an adventure spell goes to exile (not the graveyard) on
        // resolution, marked so its creature half can be cast from exile.
        if card.adventuring {
            let mut card = card;
            card.adventuring = false;
            card.on_adventure = true;
            self.exile.push(card);
            return Ok(events);
        }
        // CR 728.1a — a spell that ended the turn is exiled along with the
        // rest of the stack instead of going to the graveyard (Day's
        // Undoing). The flag stays set; `resolve_top_of_stack` consumes it.
        if self.end_turn_requested {
            self.exile.push(card);
            return Ok(events);
        }
        // CR 614.6 — an instant/sorcery bound for the graveyard is exiled
        // instead under Rest in Peace / Leyline of the Void.
        self.route_to_graveyard(card, &mut events);
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
        // Event-amount-relative filters re-checked at resolution
        // (ManaValueLessThanEventAmount) read this scratch.
        self.trigger_event_amount_scratch = event_amount;
        // CR 608.2b — if the trigger's stored sole target is no longer legal
        // at resolution (left the zone, stopped matching the filter), the
        // ability doesn't resolve: none of its effects happen. It must NOT
        // re-aim at a fresh target.
        let resolved_target = match target.as_ref() {
            Some(t) => match effect.target_filter_for_slot(0) {
                Some(filter)
                    if !self.evaluate_requirement_static(filter, t, controller, Some(source)) =>
                {
                    return Ok(vec![]);
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
            ctx.bargained = src.bargained;
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

    /// The firing event's magnitude for `Value::TriggerEventAmount` /
    /// `ManaValueLessThanEventAmount`. Mostly the event payload's `amount`;
    /// for died events it's the dying card's mana value (Scrap Trawler's
    /// "lesser mana value than that artifact"), read from the death
    /// snapshot cache (tokens are already gone from every zone).
    pub(crate) fn event_amount_for(&self, ev: &GameEvent) -> u32 {
        if let GameEvent::CreatureDied { card_id } = ev {
            return self
                .died_card_snapshots
                .get(card_id)
                .or_else(|| self.find_card_anywhere(*card_id))
                .map(|c| c.definition.cost.cmc())
                .unwrap_or(0);
        }
        event_amount(ev)
    }

    pub(crate) fn battlefield_find_mut(&mut self, id: CardId) -> Option<&mut CardInstance> {
        self.battlefield.iter_mut().find(|c| c.id == id)
    }

    /// Look up a card instance by id across every visible zone in
    /// resolution order — battlefield → each player's graveyard / hand /
    /// library → exile → stack. General-purpose helper for predicates
    /// or effects that need to introspect a card regardless of where
    /// it currently lives.
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
        | GameEvent::PoisonAdded { amount, .. }
        | GameEvent::EnergyGained { amount, .. } => *amount,
        GameEvent::CounterAdded { count, .. } => *count,
        GameEvent::Expended { total, .. } => *total,
        _ => 0,
    }
}

/// True if a card definition is colorless from its printed characteristics:
/// it has Devoid (CR 702.114) or its mana cost carries no colored pips.
/// Used by the `ColorlessCreaturesControlled` dynamic-P/T formula; avoids the
/// layer-pass circularity of reading computed colors during the same recompute.
fn is_colorless_by_cost(def: &crate::card::CardDefinition) -> bool {
    use crate::mana::ManaSymbol;
    if def.keywords.contains(&crate::card::Keyword::Devoid) {
        return true;
    }
    !def.cost.symbols.iter().any(|s| {
        matches!(
            s,
            ManaSymbol::Colored(_)
                | ManaSymbol::Hybrid(_, _)
                | ManaSymbol::Phyrexian(_)
                | ManaSymbol::MonoHybrid(_, _)
        )
    })
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
        "Werebear" => Some((7, 3, 3)),
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
            StaticEffect::PumpPTPerCounterOnSource { applies_to, kind, per_power, per_toughness } => {
                let n = card.counter_count(*kind) as i32;
                if n == 0 {
                    return vec![];
                }
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(
                            n * per_power,
                            n * per_toughness,
                        ),
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
            StaticEffect::LoseKeyword { applies_to, keyword } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::RemoveKeyword(keyword.clone()),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::GrantAllBasicLandTypes { applies_to } => {
                use crate::card::LandType;
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::SetLandTypes(vec![
                            LandType::Plains,
                            LandType::Island,
                            LandType::Swamp,
                            LandType::Mountain,
                            LandType::Forest,
                        ]),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::GrantAllColors { applies_to } => {
                use crate::mana::Color;
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L5Color,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::SetColors(vec![
                            Color::White,
                            Color::Blue,
                            Color::Black,
                            Color::Red,
                            Color::Green,
                        ]),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::LandTypeChanger { applies_to, land_type, replace } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => {
                        let mk = |layer, modification| ContinuousEffect {
                            timestamp,
                            source,
                            affected: affected.clone(),
                            layer,
                            sublayer: None,
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification,
                        };
                        if *replace {
                            // Blood Moon — lose other land types + abilities;
                            // the intrinsic mana ability follows the type.
                            vec![
                                mk(Layer::L4Type, Modification::SetLandTypes(vec![*land_type])),
                                mk(Layer::L6Ability, Modification::RemoveAllAbilities),
                            ]
                        } else {
                            // Urborg — the type in addition.
                            vec![mk(Layer::L4Type, Modification::AddLandType(*land_type))]
                        }
                    }
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
            | StaticEffect::ControllerSpellsHaveFlash { .. }
            // DoubleTokens — read at `Effect::CreateToken` resolution time
            // via `GameState::token_doublers_for(seat)`; no layer effect.
            | StaticEffect::DoubleTokens
            // DoubleCounters — read at `Effect::AddCounter` resolution time
            // via `GameState::counter_doublers_for(seat)`; no layer effect.
            | StaticEffect::DoubleCounters
            // Damage doubling/halving — read at damage time via
            // `GameState::damage_doublers` / `damage_halvers` /
            // `scale_damage_to`; no layer effect.
            | StaticEffect::DoubleDamageDealt
            | StaticEffect::HalveDamageDealt
            | StaticEffect::DoubleDamageToOpponents
            | StaticEffect::HalveDamageToYou
            | StaticEffect::AddDamageToOpponents { .. }
            | StaticEffect::OpponentMillDoubled
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
            // SuppressCreatureEtbTriggers — read at trigger dispatch via
            // `creature_etb_triggers_suppressed` / `creature_dies_triggers_suppressed`;
            // no layer effect (Torpor Orb, Tocatli Honor Guard, Hushbringer).
            | StaticEffect::SuppressCreatureEtbTriggers { .. }
            // OtherPlaneswalkersHaveSourceLoyaltyAbilities — read at loyalty
            // activation time in `activate_loyalty_ability`; no layer effect.
            | StaticEffect::OtherPlaneswalkersHaveSourceLoyaltyAbilities
            // PlayFromLibraryTop / TopOfLibraryRevealed — read by the play/
            // cast paths and the view projection; no layer effect.
            | StaticEffect::PlayFromLibraryTop { .. }
            | StaticEffect::TopOfLibraryRevealed
            // SpellsYouCastHaveDelve (Teval) — read at cast time by
            // `controller_grants_spells_delve`; no layer effect.
            | StaticEffect::SpellsYouCastHaveDelve
            // EtbTriggerTax — read at ETB trigger push time by
            // `apply_etb_trigger_tax` (Strict Proctor); no layer effect.
            | StaticEffect::EtbTriggerTax { .. }
            // PlayerCannotGainLife — projected onto Player.cannot_gain_life
            // each recompute by apply_player_statics; no layer effect.
            | StaticEffect::PlayerCannotGainLife { .. }
            // PlayerCannotLoseLife — consulted dynamically by adjust_life /
            // damage paths via player_cannot_lose_life_now; no layer effect.
            | StaticEffect::PlayerCannotLoseLife { .. }
            // LifeGainBecomesLoss — consulted dynamically by adjust_life via
            // life_gain_becomes_loss_now (Tainted Remedy); no layer effect.
            | StaticEffect::LifeGainBecomesLoss { .. }
            // AttackTaxToController — consulted in declare_attackers; no layer.
            | StaticEffect::AttackTaxToController { .. }
            // BlockTaxToController — consulted in declare_blockers; no layer.
            | StaticEffect::BlockTaxToController { .. }
            // CapDrawsPerTurn — consulted at draw time via draw_cap_for; no
            // layer effect.
            | StaticEffect::CapDrawsPerTurn { .. }
            // CoinFlipAdvantage (Krark's Thumb) — consulted dynamically by
            // the FlipCoin resolver via coin_flip_advantage_now; no layer effect.
            | StaticEffect::CoinFlipAdvantage { .. }
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
            // AnyoneCastsCheapCreaturesFree (Aluren) — read by the free-cast
            // action via `player_casts_cheap_creature_free`; no layer effect.
            | StaticEffect::AnyoneCastsCheapCreaturesFree { .. }
            // GrantKeywordToAttackers — needs live combat state, resolved in
            // `compute_battlefield` against `GameState.attacking`.
            | StaticEffect::GrantKeywordToAttackers { .. }
            // GrantActivatedAbility — surfaced as a virtual activated ability
            // in `activate_ability`; not a characteristic layer effect.
            | StaticEffect::GrantActivatedAbility { .. }
            // Necrotic Ooze — surfaced via `granted_abilities_for`, not a layer.
            | StaticEffect::HasActivatedAbilitiesOfGraveyardCreatures
            | StaticEffect::CounteredCreaturesHaveAbilitiesOfExiledWithSource
            | StaticEffect::MayCastPermanentsFromGraveyard
            | StaticEffect::ActivationCostReduction { .. }
            | StaticEffect::GraveyardLibraryLockdown
            // SkipStep — consulted by `advance_step` (CR 614.10); no layer.
            | StaticEffect::SkipStep { .. }
            // AttackPowerCapByControllerHand — consulted in declare_attackers.
            | StaticEffect::AttackPowerCapByControllerHand
            // NotCreatureWhileDevotionBelow — needs live devotion count,
            // resolved in `gather_continuous_effects` against the GameState.
            | StaticEffect::NotCreatureWhileDevotionBelow { .. }
            // PumpSelfByControlledPermanents — needs a live battlefield
            // count; resolved in `gather_continuous_effects`.
            | StaticEffect::PumpSelfByControlledPermanents { .. }
            // PumpSelfIf — needs live predicate evaluation; resolved in
            // `gather_continuous_effects`.
            | StaticEffect::PumpSelfIf { .. }
            // PumpTeamIf — conditional team anthem, resolved in
            // `gather_continuous_effects` (needs live predicate eval).
            | StaticEffect::PumpTeamIf { .. }
            // AnthemForChosenType — reads the source's live chosen creature
            // type; resolved in `gather_continuous_effects`.
            | StaticEffect::AnthemForChosenType { .. }
            // ChosenTypeEntersWithCounter — read at ETB-counter time via
            // `chosen_type_etb_counter_specs`; no continuous-layer effect.
            | StaticEffect::ChosenTypeEntersWithCounter { .. }
            // ExileNontokenCreaturesNotCast (Containment Priest) — read at
            // battlefield-entry time by `nontoken_creature_etb_exile_active`;
            // no layer effect.
            | StaticEffect::ExileNontokenCreaturesNotCast
            // NoMaximumHandSize / OpponentsMaxHandSizeReduced — consulted
            // at cleanup via `effective_max_hand_size`; no layer effect.
            | StaticEffect::NoMaximumHandSize
            | StaticEffect::OpponentsMaxHandSizeReduced(_)
            // MayPlayLandsFromGraveyard — consulted by the land-play paths
            // via `player_may_play_lands_from_graveyard`; no layer effect.
            | StaticEffect::MayPlayLandsFromGraveyard
            // MayReturnFromGraveyardInsteadOfLearn — consulted at the top of
            // `Effect::Learn` (Retriever Phoenix); no layer effect.
            | StaticEffect::MayReturnFromGraveyardInsteadOfLearn
            // LifeGainBonus — consulted in `adjust_life` via
            // `life_gain_bonus_now` (Honor Troll); no layer effect.
            | StaticEffect::LifeGainBonus { .. }
            // DamageCantBePrevented — consulted in `apply_prevention_shields`
            // via `damage_cant_be_prevented_now` (Sulfuric Vortex); no layer.
            | StaticEffect::DamageCantBePrevented
            // ManaProductionDoubled — consulted at mana-ability resolution
            // via `mana_production_doublers_for`; no layer effect.
            | StaticEffect::ManaProductionDoubled
            // CreatureActivatedAbilitiesLocked — consulted in
            // `activate_ability` (Cursed Totem); no layer effect.
            | StaticEffect::CreatureActivatedAbilitiesLocked
            // CountersCantBePlaced (Solemnity) — consulted at every
            // counter-placement site via `counters_locked`; no layer effect.
            | StaticEffect::CountersCantBePlaced
            // ExileCardsBoundForGraveyard (Rest in Peace / Leyline of the
            // Void) — consulted at graveyard-placement time via
            // `graveyard_exiled_for`; no layer effect.
            | StaticEffect::ExileCardsBoundForGraveyard { .. }
            // Search statics (Aven Mindcensor / Leonin Arbiter) — consulted
            // in `Effect::Search` via `search_top_limit_for` /
            // `pay_search_tax`; no layer effect.
            | StaticEffect::OpponentsSearchTopN { .. }
            | StaticEffect::SearchTax { .. }
            // ActivationTax (Suppression Field) — consulted in
            // `activate_ability`; no layer effect.
            | StaticEffect::ActivationTax { .. }
            // UntapAllYoursEachUntapStep (Seedborn Muse) — consulted by
            // `do_untap`; no layer effect.
            | StaticEffect::UntapAllYoursEachUntapStep
            // ExileDyingOpponentCreatures (Valentin) — consulted in
            // `remove_from_battlefield_to_graveyard`; no layer effect.
            | StaticEffect::ExileDyingOpponentCreatures { .. }
            // YourInstantSorcerySpellsHaveLifelink (Radiant Scrollwielder) —
            // consulted in the non-combat damage path; no layer effect.
            | StaticEffect::YourInstantSorcerySpellsHaveLifelink
            // SelfCostReducedByGreatestPower (The Great Henge) — read by
            // `cost_reduction_for_spell` off the spell being cast; no layer.
            | StaticEffect::SelfCostReducedByGreatestPower
            // SelfCostReducedByDomain (Leyline Binding) — same, off the spell.
            | StaticEffect::SelfCostReducedByDomain
            // SacrificeCostReduction (Awaken the Blood Avatar) — an optional
            // additional cost consulted by `cast_spell_sacrifice_reduce`; no
            // continuous-layer effect.
            | StaticEffect::SacrificeCostReduction { .. }
            // BargainCostReduction — read by `cast_spell_bargain` at cast time.
            | StaticEffect::BargainCostReduction { .. }
            // OpponentsCantMakeYouSacrifice (Sigarda/Tamiyo) — consulted in
            // the `Effect::Sacrifice` resolver; no continuous-layer effect.
            | StaticEffect::OpponentsCantMakeYouSacrifice
            | StaticEffect::OpponentsCantMakeYouDiscard
            | StaticEffect::ControllerDrawsDoubled
            | StaticEffect::RedirectDamageToSelf
            | StaticEffect::ControllerCantCastPermanentSpells
            | StaticEffect::SelfCostReducedPerDiscardThisTurn { .. }
            | StaticEffect::WinInsteadOfDrawFromEmpty
            | StaticEffect::OneSpellPerTurn
            | StaticEffect::MinusCounterReduction
            | StaticEffect::OpponentsCantCastDuringYourTurn => vec![],
        })
        .collect()
}

/// Translate a selector into a `layers::AffectedPermanents` description for
/// those `StaticEffect` variants that express broad "lord-like" scope. Returns
/// `None` if the selector shape isn't representable in the layer system yet.
/// True if the filter tree contains `IsModified` (CR 700.9) — such filters
/// are resolved live in `gather_continuous_effects`, not through the static
/// `AffectedPermanents` decomposition.
fn requirement_mentions_modified(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::IsModified => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_modified(a) || requirement_mentions_modified(b)
        }
        R::Not(inner) => requirement_mentions_modified(inner),
        _ => false,
    }
}

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

/// Whether the conjunctive And-tree walker in `affected_from_requirement`
/// recognizes every leaf of `req`. Mirrors that walker's match arms; any
/// other leaf (or a disjunction) falls outside it.
fn simple_walker_can_handle(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::And(a, b) => simple_walker_can_handle(a) && simple_walker_can_handle(b),
        R::ControlledByYou | R::ControlledByOpponent | R::Creature | R::Artifact
        | R::Enchantment | R::Planeswalker | R::Land | R::HasCardType(_)
        | R::HasCreatureType(_) | R::WithCounter(_) | R::HasColor(_) | R::Colorless
        | R::IsToken | R::NotToken | R::OtherThanSource | R::Any | R::Permanent => true,
        _ => false,
    }
}

fn affected_from_requirement(
    req: &SelectionRequirement,
    source_controller: usize,
) -> Option<AffectedPermanents> {
    use SelectionRequirement as R;
    // Disjunctive / nonbasic-land filters can't be flattened into the simple
    // controller+type decomposition below (`card_types` is conjunctive, and
    // there's no plain CardType for "nonbasic land"). When every leaf is
    // computable from a card's printed characteristics, route the whole filter
    // through the card-local matcher instead of dropping the static (CR 614.13
    // Thalia, Heretic Cathar). Only used when the simple walker can't.
    if !simple_walker_can_handle(req) && crate::game::layers::requirement_is_card_only(req) {
        return Some(AffectedPermanents::CardMatch {
            source_controller,
            requirement: Box::new(req.clone()),
        });
    }
    // Decompose And-trees to extract controller filter + card-type filter.
    let mut ctrl: Option<Option<usize>> = None; // Outer Some(None) = all players; Some(Some(n)) = specific player
    let mut types: Vec<CardType> = vec![];
    let mut creature_type: Option<crate::card::CreatureType> = None;
    let mut counter_filter: Option<crate::card::CounterType> = None;
    let mut color_filter: Option<crate::mana::Color> = None;
    let mut colorless_filter = false;
    let mut token_filter: Option<bool> = None;
    // CR-driven "other" exclusion (push XXXV). `SelectionRequirement::
    // OtherThanSource` flips this to true; the resulting AffectedPermanents
    // variant carries `exclude_source: true` so the layer-time `affects()`
    // check skips the source permanent itself — matching printed "**other**
    // [type] you control" wording.
    let mut other_than_source = false;
    let mut opponent = false;
    let mut walk = vec![req];
    while let Some(r) = walk.pop() {
        match r {
            R::And(a, b) => {
                walk.push(a);
                walk.push(b);
            }
            R::ControlledByYou => ctrl = Some(Some(source_controller)),
            // Accumulate a flag rather than returning early, so the opponent
            // filter composes with type filters regardless of And-tree order
            // (`ControlledByOpponent.and(Creature)` and the reverse both work).
            R::ControlledByOpponent => opponent = true,
            R::Creature => types.push(CardType::Creature),
            R::Artifact => types.push(CardType::Artifact),
            R::Enchantment => types.push(CardType::Enchantment),
            R::Planeswalker => types.push(CardType::Planeswalker),
            R::Land => types.push(CardType::Land),
            R::HasCardType(t) => types.push(t.clone()),
            R::HasCreatureType(ct) => creature_type = Some(*ct),
            R::WithCounter(ct) => counter_filter = Some(*ct),
            R::HasColor(c) => color_filter = Some(*c),
            R::Colorless => colorless_filter = true,
            R::IsToken => token_filter = Some(true),
            R::NotToken => token_filter = Some(false),
            R::OtherThanSource => other_than_source = true,
            R::Any | R::Permanent => {}
            _ => return None,
        }
    }
    if opponent {
        // `friendly_seats` is populated by `compute_battlefield` /
        // `apply_enters_tapped_replacement` once the source's team is known
        // (this helper has no GameState handle). Counter/creature-type filters
        // on the opponent path aren't decomposed yet (tracked in TODO.md).
        return Some(AffectedPermanents::AllOpponents {
            source_controller,
            card_types: types,
            friendly_seats: Vec::new(),
        });
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
        color: color_filter,
        token: token_filter,
        colorless: colorless_filter,
    })
}


// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns true if `blocker` is legally allowed to block `attacker`.
/// Uses `blocker_kws` / `attacker_kws` as the effective keyword sets
/// (from `ComputedPermanent`) instead of the raw definition keywords.
pub(crate) fn can_block_attacker_computed(
    blocker: &CardInstance,
    blocker_computed: &ComputedPermanent,
    attacker_kws: &[Keyword],
    attacker_colors: &[crate::mana::Color],
    attacker_power: i32,
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
    // Skulk (CR 702.72a): can't be blocked by creatures with greater power.
    // Both sides use layer-computed power (an anthem-pumped Skulk attacker
    // dodges bigger blockers correctly).
    if attacker_kws.contains(&Keyword::Skulk) && blocker_computed.power > attacker_power {
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
    // CR 509.1b "can't be blocked except by [filter]" / "can't be blocked by
    // [filter]" — evaluate the blocker's computed characteristics against the
    // attacker's filter keywords.
    for kw in attacker_kws {
        match kw {
            Keyword::CantBeBlockedExceptBy(filter) => {
                if !blocker_matches_block_filter(blocker, blocker_computed, filter) {
                    return false;
                }
            }
            Keyword::CantBeBlockedBy(filter) => {
                if blocker_matches_block_filter(blocker, blocker_computed, filter) {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

/// Lightweight evaluation of a block-restriction filter against a blocker's
/// *computed* characteristics. Covers the subset of `SelectionRequirement`
/// that "can't be blocked except by [filter]" cards actually use (type,
/// color, keyword, power/toughness thresholds). Unsupported variants resolve
/// to `false` (conservatively excluding the blocker).
fn blocker_matches_block_filter(
    blocker: &CardInstance,
    computed: &ComputedPermanent,
    req: &SelectionRequirement,
) -> bool {
    use SelectionRequirement as R;
    match req {
        R::Any | R::Permanent | R::Creature => true,
        R::Artifact => blocker.definition.is_artifact(),
        R::Enchantment => blocker.definition.is_enchantment(),
        R::Land => blocker.definition.is_land(),
        R::IsToken => blocker.is_token,
        R::NotToken => !blocker.is_token,
        R::HasColor(c) => computed.colors.contains(c),
        R::Colorless => computed.colors.is_empty(),
        R::HasKeyword(k) => computed.keywords.contains(k),
        R::HasCreatureType(t) => blocker.definition.subtypes.creature_types.contains(t)
            || computed.keywords.contains(&Keyword::Changeling),
        R::HasArtifactSubtype(a) => blocker.definition.subtypes.artifact_subtypes.contains(a),
        R::PowerAtMost(n) => computed.power <= *n,
        R::PowerAtLeast(n) => computed.power >= *n,
        R::ToughnessAtMost(n) => computed.toughness <= *n,
        R::ToughnessAtLeast(n) => computed.toughness >= *n,
        R::HasCardType(ct) => blocker.definition.card_types.contains(ct),
        R::And(a, b) => {
            blocker_matches_block_filter(blocker, computed, a)
                && blocker_matches_block_filter(blocker, computed, b)
        }
        R::Or(a, b) => {
            blocker_matches_block_filter(blocker, computed, a)
                || blocker_matches_block_filter(blocker, computed, b)
        }
        R::Not(inner) => !blocker_matches_block_filter(blocker, computed, inner),
        _ => false,
    }
}
