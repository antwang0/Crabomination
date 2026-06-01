use serde::{Deserialize, Serialize};

use crate::card::{CardDefinition, CardId, CardInstance};
use crate::mana::ManaPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub usize);

/// CR 114 — an emblem owned by a player. Has no characteristics other
/// than the triggered abilities it grants its owner, and sits in the
/// command zone for the rest of the game (emblems never leave). Created
/// by planeswalker ultimates via `Effect::CreateEmblem`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emblem {
    /// Source name, for display (e.g. "Professor Dellian Fel").
    pub name: String,
    /// Abilities the emblem grants its owner.
    pub triggered: Vec<crate::effect::TriggeredAbility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub life: i32,
    pub mana_pool: ManaPool,
    /// Top of library is `library[0]`.
    pub library: Vec<CardInstance>,
    pub hand: Vec<CardInstance>,
    pub graveyard: Vec<CardInstance>,
    /// The command zone — Commander commanders, Conspiracies, etc.
    /// (Phase I.) Cards arrive here either at game start (initial
    /// commander seating via `seat_commanders`) or via a zone-change
    /// replacement effect when they would otherwise leave the
    /// battlefield (CR 903.9b).
    ///
    /// `#[serde(default)]` so snapshots written before the field
    /// existed deserialize cleanly as empty.
    #[serde(default)]
    pub command: Vec<CardInstance>,
    /// CR 406 / 701.45 — the Lessons "sideboard" (cards owned from outside
    /// the game). A Learn ability may reveal a Lesson card here and put it
    /// into hand. Populated by deck construction; empty by default (in
    /// which case Learn falls back to the legacy `Draw 1` approximation).
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub sideboard: Vec<CardInstance>,
    /// CardIds of cards this player has designated as Commanders
    /// (Phase J). Populated by `GameState::seat_commanders`. Read by
    /// the Phase M 21-commander-damage SBA via
    /// `GameState::is_commander`. Note this is *which cards are
    /// commanders for this player*, independent of the command zone
    /// — a commander on the battlefield (or any other zone) is
    /// still a commander, so the entry survives zone changes.
    #[serde(default)]
    pub commanders: Vec<CardId>,
    /// How many lands this player has played on their current turn.
    pub lands_played_this_turn: u32,
    /// Extra land plays granted this turn (Explore, Oracle of Mul Daya,
    /// Dryad of the Ilysian Grove, etc.). Defaults to 0. The player can
    /// play `1 + extra_land_plays` lands per turn total.
    #[serde(default)]
    pub extra_land_plays: u32,
    /// How many spells this player has cast this turn. Reset on
    /// `TurnStarted`. Powers Damping Sphere's "second-and-onward spells
    /// cost {1} more" static.
    pub spells_cast_this_turn: u32,
    /// Total life gained by this player this turn (sum of every
    /// `Effect::GainLife` and `Effect::Drain`-to-this-player resolution).
    /// Reset to 0 in `do_untap`. Powers Strixhaven's **Infusion** rider —
    /// "If you gained life this turn, …" — and any future "you've gained
    /// life this turn" payoffs without needing a custom event log scan.
    /// Default-deserializes to 0 for snapshots predating the field.
    #[serde(default)]
    pub life_gained_this_turn: u32,
    /// Number of cards this player has drawn on the current turn. Reset
    /// to 0 in `do_untap`. Powers Strixhaven's Quandrix scaling — e.g.
    /// Fractal Anomaly creates a 0/0 with X +1/+1 counters where X is
    /// "cards drawn this turn." Surfaced through `PlayerView` so client
    /// UIs can preview the scaling. Defaults to 0 for snapshot
    /// backwards-compatibility.
    #[serde(default)]
    pub cards_drawn_this_turn: u32,
    /// Number of times a card has left this player's graveyard on the
    /// current turn. Reset to 0 in `do_untap`. Powers Strixhaven Lorehold
    /// "if a card left your graveyard this turn" payoffs (Living History,
    /// Primary Research's end-step draw rider, Wilt in the Heat's cost
    /// reduction). Backed by the `CardLeftGraveyard` event emission in
    /// `move_card_to`. Defaults to 0 for snapshot back-compat.
    #[serde(default)]
    pub cards_left_graveyard_this_turn: u32,
    /// Number of creatures controlled by this player that died this turn.
    /// Reset to 0 in `do_untap`. Powers Witherbloom "if a creature died
    /// under your control this turn, …" end-step payoffs (Essenceknit
    /// Scholar). Bumped from `apply_state_based_actions`'s SBA dies
    /// handler keyed off the dying creature's controller. Defaults to 0
    /// for snapshot back-compat.
    #[serde(default)]
    pub creatures_died_this_turn: u32,
    /// True if this player has been dealt damage so far this turn. Set in
    /// `deal_damage_to_from`'s player branch (combat or non-combat, incl.
    /// infect/poison), reset for *all* players at the active player's
    /// `do_untap` so it reflects "damaged since this turn began" — the
    /// Bloodthirst (CR 702.54) window. Defaults to false for snapshot
    /// back-compat.
    #[serde(default)]
    pub was_dealt_damage_this_turn: bool,
    /// Number of cards this player has caused to be put into exile on
    /// the current turn. Reset to 0 in `do_untap`. Powers Strixhaven
    /// "if one or more cards were put into exile this turn" payoffs
    /// (Ennis the Debate Moderator). Bumped from `place_card_in_dest`'s
    /// exile branch and the battlefield-to-exile path in
    /// `Effect::Exile`. Defaults to 0 for snapshot back-compat.
    #[serde(default)]
    pub cards_exiled_this_turn: u32,
    /// Number of instant or sorcery spells this player has cast on the
    /// current turn. Reset to 0 in `do_untap`. Refines
    /// `spells_cast_this_turn` (which counts every spell type) so cards
    /// like Potioner's Trove can gate "activate only if you've cast an
    /// instant or sorcery spell this turn" precisely. Bumped in
    /// `finalize_cast` whenever the resolving spell card carries the
    /// Instant or Sorcery card type. Defaults to 0 for snapshot
    /// back-compat.
    #[serde(default)]
    pub instants_or_sorceries_cast_this_turn: u32,
    /// Number of creature spells this player has cast on the current
    /// turn. Reset to 0 in `do_untap`. Powers creature-cast magecraft
    /// payoffs ("if you've cast a creature spell this turn, …") and
    /// future creature-spell-matters cards. Defaults to 0 for snapshot
    /// back-compat.
    #[serde(default)]
    pub creatures_cast_this_turn: u32,
    /// Pending "first spell costs {1} more" taxes against this player.
    /// Each spell cast consumes one charge, charging the caster {1} extra
    /// generic in `extra_cost_for_spell`. Set by Chancellor of the Annex's
    /// opening-hand reveal (one charge per Annex revealed by an opponent).
    pub first_spell_tax_charges: u32,
    /// True if this player can cast sorceries at instant speed until their
    /// next turn. Set by Teferi, Time Raveler's +1; cleared in `do_untap`
    /// when this player's own turn begins.
    pub sorceries_as_flash: bool,
    /// Poison counters (player loses at 10).
    pub poison_counters: u32,
    /// True if this player has no maximum hand size for the rest of the
    /// game. Set by `Effect::SetNoMaxHandSize` (Wisdom of Ages, Reliquary
    /// Tower-style effects). When true, the cleanup-step CR 514.1 enforcement
    /// in `do_cleanup` skips the discard-down-to-7 step.
    #[serde(default)]
    pub no_maximum_hand_size: bool,
    /// True once this player has lost the game (life ≤ 0, poison ≥ 10, or
    /// drew from an empty library). Eliminated players are skipped by turn
    /// and priority rotation; the game ends when ≤ 1 player remains.
    pub eliminated: bool,
    /// Number of upcoming turns this player must skip. Read by the
    /// turn-advance logic in `do_cleanup` — when the engine would hand
    /// the next turn to this player, the counter is decremented and the
    /// turn is bypassed (advancing to the player after). Set by
    /// `Effect::SkipTurns` (Ral Zarek, Guest Lecturer's -7 ult). Defaults
    /// to 0 for snapshot back-compat.
    #[serde(default)]
    pub skip_turns: u32,
    /// CR 500.7 — extra turns this player will take. When `advance_turn`
    /// would pass the turn, an active player with `extra_turns > 0`
    /// decrements it and keeps the turn instead (Time Walk, Ral Zarek's
    /// -7 coin-flip emblem). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub extra_turns: u32,
    /// CR 114 — emblems this player owns. Each carries a name (for
    /// display) and a set of triggered abilities that fire from the
    /// command zone; emblems never leave once created. The trigger
    /// dispatcher walks every player's emblems alongside battlefield
    /// permanents (event-keyed kinds in `dispatch_triggers_for_events`,
    /// step-keyed kinds in `fire_step_triggers`). Created by
    /// `Effect::CreateEmblem` (planeswalker ultimates). `#[serde(default)]`
    /// for snapshot back-compat.
    #[serde(default)]
    pub emblems: Vec<Emblem>,
    /// True while a continuous effect on the battlefield prevents this
    /// player from gaining life (CR 119.7). Set by
    /// `StaticEffect::CannotGainLife` in `compute_battlefield`'s player-
    /// static pass, reset there each recompute. Honored by
    /// `GameState::adjust_life` — a positive delta is dropped while the
    /// flag is set. Powers Tainted Remedy / Erebos / Sulfuric Vortex
    /// style effects.
    #[serde(default)]
    pub cannot_gain_life: bool,
    /// Sticky one-turn "you can't gain life" lock — separate from the
    /// recomputed `cannot_gain_life` static. Set by `Effect::LifeGainLockThisTurn`
    /// (Skullcrack, Rampaging Ferocidon's one-shot version), reset in
    /// `do_untap`. Honored by `GameState::adjust_life` (treated identically
    /// to `cannot_gain_life`, but persists across `compute_battlefield`
    /// recomputes since no permanent backs it).
    #[serde(default)]
    pub cannot_gain_life_this_turn: bool,
    /// True while spells this player controls can't be countered for the
    /// rest of the turn (Veil of Summer's "spells your opponents control
    /// can't counter spells you control this turn"). Set by
    /// `Effect::GrantSpellsUncounterableThisTurn`; reset for every player at
    /// the active player's `do_untap`. Consulted by
    /// `caster_grants_uncounterable_with_x`. `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub spells_uncounterable_this_turn: bool,
    /// True once this player has cast a blue or black spell this turn. Set
    /// in `finalize_cast`; reset for every player at the active player's
    /// `do_untap`. Powers Veil of Summer's "draw a card if an opponent has
    /// cast a blue or black spell this turn" gate. `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub cast_blue_or_black_this_turn: bool,
    /// When true, decisions this player would make suspend via
    /// `pending_decision` so a UI can respond; when false, the engine calls
    /// the installed `Decider` synchronously (bot / tests).
    pub wants_ui: bool,
    /// CR 705.3 — Krark's Thumb-style coin-flip advantage. When non-zero,
    /// every coin flip this player makes is replayed `coin_flip_advantage`
    /// extra times and they get to keep the result they prefer. Practically
    /// modelled in `Effect::FlipCoin` as "do `1 + N` flips and treat the
    /// flipper as winning if any of them came up heads" — the standard
    /// rules interpretation of stacking Krark's Thumbs (each Thumb lets
    /// you "ignore one and choose the other," so two Thumbs = three flips,
    /// pick the best).
    ///
    /// `#[serde(default)]` keeps snapshots from before this field forward-
    /// compatible. Stacks additively when multiple Krark's Thumbs are on
    /// the battlefield (compute_battlefield sums the contributing
    /// static-ability counts when this primitive is eventually wired to
    /// a permanent — for now only one Krark's Thumb is needed and we set
    /// the value directly via the Thumb card body).
    #[serde(default)]
    pub coin_flip_advantage: u32,
}

impl Player {
    pub fn new(idx: usize, name: impl Into<String>) -> Self {
        Self {
            id: PlayerId(idx),
            name: name.into(),
            life: 20,
            mana_pool: ManaPool::new(),
            library: Vec::new(),
            hand: Vec::new(),
            graveyard: Vec::new(),
            command: Vec::new(),
            sideboard: Vec::new(),
            commanders: Vec::new(),
            lands_played_this_turn: 0,
            extra_land_plays: 0,
            spells_cast_this_turn: 0,
            life_gained_this_turn: 0,
            cards_drawn_this_turn: 0,
            cards_left_graveyard_this_turn: 0,
            creatures_died_this_turn: 0,
            was_dealt_damage_this_turn: false,
            cards_exiled_this_turn: 0,
            instants_or_sorceries_cast_this_turn: 0,
            creatures_cast_this_turn: 0,
            cannot_gain_life_this_turn: false,
            spells_uncounterable_this_turn: false,
            cast_blue_or_black_this_turn: false,
            first_spell_tax_charges: 0,
            sorceries_as_flash: false,
            poison_counters: 0,
            no_maximum_hand_size: false,
            eliminated: false,
            skip_turns: 0,
            extra_turns: 0,
            emblems: Vec::new(),
            cannot_gain_life: false,
            wants_ui: false,
            coin_flip_advantage: 0,
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.eliminated
    }

    /// Baseline per-turn land-play check — `true` iff this player has
    /// not yet played any land this turn. NOTE: this is a vanilla CR
    /// 305.2 default and **does not** consult
    /// `StaticEffect::ExtraLandPerTurn` (Exploration, Azusa). For the
    /// CR-correct check that honors continuous-effect grants, use
    /// `GameState::can_player_play_land(seat)` which sums
    /// `extra_land_plays_per_turn(seat)` into the cap.
    pub fn can_play_land(&self) -> bool {
        self.lands_played_this_turn < 1 + self.extra_land_plays
    }

    /// Draw the top card into hand.  Returns `None` if the library is empty.
    /// Increments `cards_drawn_this_turn` so per-turn draw payoffs (e.g.
    /// Strixhaven's Quandrix scaling) see a fresh count.
    pub fn draw_top(&mut self) -> Option<CardId> {
        if self.library.is_empty() {
            return None;
        }
        let card = self.library.remove(0);
        let id = card.id;
        self.hand.push(card);
        self.cards_drawn_this_turn = self.cards_drawn_this_turn.saturating_add(1);
        Some(id)
    }

    /// Return all hand cards to the bottom of the library.
    /// Call `library.shuffle(&mut rng)` afterwards to randomize.
    pub fn return_hand_to_library(&mut self) {
        while let Some(card) = self.hand.pop() {
            self.library.push(card);
        }
    }

    pub fn has_in_hand(&self, id: CardId) -> bool {
        self.hand.iter().any(|c| c.id == id)
    }

    pub fn remove_from_hand(&mut self, id: CardId) -> Option<CardInstance> {
        self.hand
            .iter()
            .position(|c| c.id == id)
            .map(|i| self.hand.remove(i))
    }

    pub fn send_to_graveyard(&mut self, card: CardInstance) {
        self.graveyard.push(card);
    }

    /// Push a card definition directly into the library (top of deck = index 0).
    pub fn add_to_library_top(&mut self, id: CardId, definition: CardDefinition) {
        self.library.insert(0, CardInstance::new(id, definition, self.id.0));
    }

    /// Push a card definition to the bottom of the library.
    pub fn add_to_library_bottom(&mut self, id: CardId, definition: CardDefinition) {
        self.library.push(CardInstance::new(id, definition, self.id.0));
    }
}
