use crate::card::{CardDefinition, CardId, CardInstance};
use crate::mana::ManaPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub usize);

#[derive(Debug, Clone)]
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
    pub fn draw_top(&mut self) -> Option<CardId> {
        if self.library.is_empty() {
            return None;
        }
        let card = self.library.remove(0);
        let id = card.id;
        self.hand.push(card);
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
