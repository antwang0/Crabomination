use serde::{Deserialize, Serialize};

use crate::card::{CardDefinition, CardId, CardInstance};
use crate::mana::ManaPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub usize);

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
    /// How many lands this player has played on their current turn.
    pub lands_played_this_turn: u32,
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
    /// True once this player has lost the game (life ≤ 0, poison ≥ 10, or
    /// drew from an empty library). Eliminated players are skipped by turn
    /// and priority rotation; the game ends when ≤ 1 player remains.
    pub eliminated: bool,
    /// When true, decisions this player would make suspend via
    /// `pending_decision` so a UI can respond; when false, the engine calls
    /// the installed `Decider` synchronously (bot / tests).
    pub wants_ui: bool,
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
            lands_played_this_turn: 0,
            spells_cast_this_turn: 0,
            life_gained_this_turn: 0,
            cards_drawn_this_turn: 0,
            cards_left_graveyard_this_turn: 0,
            creatures_died_this_turn: 0,
            cards_exiled_this_turn: 0,
            instants_or_sorceries_cast_this_turn: 0,
            creatures_cast_this_turn: 0,
            first_spell_tax_charges: 0,
            sorceries_as_flash: false,
            poison_counters: 0,
            eliminated: false,
            wants_ui: false,
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.eliminated
    }

    pub fn can_play_land(&self) -> bool {
        self.lands_played_this_turn == 0
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
