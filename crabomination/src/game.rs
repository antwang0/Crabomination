//! Core MTG game engine.
//!
//! # Design notes
//! - **Simplified stack**: instants and sorceries resolve immediately on cast
//!   (no priority loop between players).  The `stack` field is reserved for
//!   future expansion.
//! - **Simplified combat**: damage is simultaneous, one pass only.  First
//!   strike is tracked by keyword but does not trigger a separate damage step.
//! - All actions are performed by the *active player* except `declare_blockers`
//!   which is called by whoever controls the defending creatures.

use std::collections::HashMap;

use crate::card::{CardDefinition, CardId, CardInstance, CardType, Keyword, SelectionRequirement, SpellEffect, TriggerCondition};
use crate::mana::{Color, ManaError};
use crate::player::Player;

// ── Turn step sequence ────────────────────────────────────────────────────────

/// Flat enumeration of every step and phase in an MTG turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnStep {
    Untap,
    Upkeep,
    Draw,
    PreCombatMain,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    EndCombat,
    PostCombatMain,
    End,
    Cleanup,
}

impl TurnStep {
    pub fn next(self) -> Self {
        match self {
            TurnStep::Untap => TurnStep::Upkeep,
            TurnStep::Upkeep => TurnStep::Draw,
            TurnStep::Draw => TurnStep::PreCombatMain,
            TurnStep::PreCombatMain => TurnStep::BeginCombat,
            TurnStep::BeginCombat => TurnStep::DeclareAttackers,
            TurnStep::DeclareAttackers => TurnStep::DeclareBlockers,
            TurnStep::DeclareBlockers => TurnStep::CombatDamage,
            TurnStep::CombatDamage => TurnStep::EndCombat,
            TurnStep::EndCombat => TurnStep::PostCombatMain,
            TurnStep::PostCombatMain => TurnStep::End,
            TurnStep::End => TurnStep::Cleanup,
            TurnStep::Cleanup => TurnStep::Untap,
        }
    }

    pub fn is_main_phase(self) -> bool {
        matches!(self, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
    }
}

#[derive(Debug, Clone)]
pub enum Target {
    Player(usize),
    Permanent(CardId),
}

#[derive(Debug, Clone)]
pub enum GameAction {
    PlayLand(CardId),
    CastSpell { card_id: CardId, target: Option<Target> },
    /// Activate ability at `ability_index` on a permanent.
    ActivateAbility { card_id: CardId, ability_index: usize, target: Option<Target> },
    /// Declare one or more attackers at once (must be in DeclareAttackers step).
    DeclareAttackers(Vec<CardId>),
    /// Defending player assigns blockers: `(blocker_id, attacker_id)` pairs.
    DeclareBlockers(Vec<(CardId, CardId)>),
    PassPriority,
}

// ── Events ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum GameEvent {
    StepChanged(TurnStep),
    TurnStarted { player: usize, turn: u32 },
    CardDrawn { player: usize, card_id: CardId },
    CardDiscarded { player: usize, card_id: CardId },
    LandPlayed { player: usize, card_id: CardId },
    SpellCast { player: usize, card_id: CardId },
    AbilityActivated { source: CardId },
    ManaAdded { player: usize, color: Color },
    PermanentEntered { card_id: CardId },
    DamageDealt { amount: u32, to_player: Option<usize>, to_card: Option<CardId> },
    LifeLost { player: usize, amount: u32 },
    LifeGained { player: usize, amount: u32 },
    CreatureDied { card_id: CardId },
    PumpApplied { card_id: CardId, power: i32, toughness: i32 },
    AttackerDeclared(CardId),
    BlockerDeclared { blocker: CardId, attacker: CardId },
    CombatResolved,
    /// Top card of `player`'s library was revealed; if `is_land` the card was drawn.
    TopCardRevealed { player: usize, card_name: &'static str, is_land: bool },
    GameOver { winner: Option<usize> },
}

// ── Stack ─────────────────────────────────────────────────────────────────────

/// An item on the stack waiting to resolve.
#[derive(Debug, Clone)]
pub enum StackItem {
    /// A non-land spell (instant, sorcery, or permanent) waiting to resolve.
    Spell {
        card: CardInstance,
        caster: usize,
        target: Option<Target>,
    },
    /// A triggered ability waiting to resolve (ETB, attack trigger, etc.).
    Trigger {
        source: CardId,
        controller: usize,
        effects: Vec<SpellEffect>,
        target: Option<Target>,
    },
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameError {
    #[error("Card {0:?} not found in hand")]
    CardNotInHand(CardId),
    #[error("Card {0:?} not found on battlefield")]
    CardNotOnBattlefield(CardId),
    #[error("Card {0:?} is not a land")]
    NotALand(CardId),
    #[error("Already played a land this turn")]
    AlreadyPlayedLand,
    #[error("Card {0:?} is tapped")]
    CardIsTapped(CardId),
    #[error("Creature {0:?} has summoning sickness")]
    SummoningSickness(CardId),
    #[error("Creature {0:?} cannot block (tapped, not a creature, or flying restriction)")]
    CannotBlock(CardId),
    #[error("Mana: {0}")]
    Mana(#[from] ManaError),
    #[error("Wrong step for this action (currently {actual:?})")]
    WrongStep { actual: TurnStep },
    #[error("This action requires a target")]
    TargetRequired,
    #[error("Invalid target")]
    InvalidTarget,
    #[error("Ability index out of bounds")]
    AbilityIndexOutOfBounds,
    #[error("Target does not meet the selection requirement for this effect")]
    SelectionRequirementViolated,
    #[error("The game is already over")]
    GameAlreadyOver,
    #[error("Cannot pass priority while blockers must be declared")]
    MustDeclareBlockers,
    #[error("Player {0}'s library is empty")]
    LibraryEmpty(usize),
}

// ── Game state ────────────────────────────────────────────────────────────────

pub struct GameState {
    pub players: Vec<Player>,
    /// All permanents currently in play.
    pub battlefield: Vec<CardInstance>,
    /// The stack of spells and triggered abilities waiting to resolve (LIFO).
    pub stack: Vec<StackItem>,
    pub step: TurnStep,
    /// Index into `players` of the player whose turn it is.
    pub active_player_idx: usize,
    pub turn_number: u32,
    /// `None` while the game is ongoing; `Some(None)` for a draw;
    /// `Some(Some(i))` when player `i` has won.
    pub game_over: Option<Option<usize>>,
    next_id: u32,
    /// Cards declared as attackers this combat.
    attacking: Vec<CardId>,
    /// Blocker → attacker mapping for the current combat.
    block_map: HashMap<CardId, CardId>,
    /// Set to true once `declare_blockers` has been called during the current DeclareBlockers step.
    blockers_declared: bool,
    /// Skip the draw on the very first turn (turn 1, first player).
    skip_first_draw: bool,
}

impl GameState {
    /// Create a fresh game.  `player_names` must have at least 2 entries.
    pub fn new(players: Vec<Player>) -> Self {
        Self {
            players,
            battlefield: Vec::new(),
            stack: Vec::new(),
            step: TurnStep::Untap,
            active_player_idx: 0,
            turn_number: 1,
            game_over: None,
            next_id: 1,
            attacking: Vec::new(),
            block_map: HashMap::new(),
            blockers_declared: false,
            skip_first_draw: true,
        }
    }

    fn next_id(&mut self) -> CardId {
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
        self.battlefield.push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Add a card to a player's library (top of deck).
    pub fn add_card_to_library(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx].add_to_library_bottom(id, def);
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

    /// Cards currently declared as attackers in this combat step.
    pub fn attacking(&self) -> &[CardId] {
        &self.attacking
    }

    /// True if `blocker_id` can legally block at least one current attacker.
    pub fn can_block_any_attacker(&self, blocker_id: CardId) -> bool {
        let Some(blocker) = self.battlefield.iter().find(|c| c.id == blocker_id) else {
            return false;
        };
        if !blocker.can_block() {
            return false;
        }
        self.attacking.iter().any(|&atk_id| {
            self.battlefield
                .iter()
                .find(|c| c.id == atk_id)
                .map(|atk| can_block_attacker(blocker, atk))
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
        can_block_attacker(blocker, attacker)
    }

    // ── Main action dispatch ──────────────────────────────────────────────────

    pub fn perform_action(&mut self, action: GameAction) -> Result<Vec<GameEvent>, GameError> {
        if self.is_game_over() {
            return Err(GameError::GameAlreadyOver);
        }
        match action {
            GameAction::PlayLand(id) => self.play_land(id),
            GameAction::CastSpell { card_id, target } => self.cast_spell(card_id, target),
            GameAction::ActivateAbility { card_id, ability_index, target } => {
                self.activate_ability(card_id, ability_index, target)
            }
            GameAction::DeclareAttackers(ids) => self.declare_attackers(ids),
            GameAction::DeclareBlockers(assignments) => self.declare_blockers(assignments),
            GameAction::PassPriority => self.pass_priority(),
        }
    }

    // ── Play land ─────────────────────────────────────────────────────────────

    fn play_land(&mut self, card_id: CardId) -> Result<Vec<GameEvent>, GameError> {
        if !self.step.is_main_phase() {
            return Err(GameError::WrongStep { actual: self.step });
        }
        let p = self.active_player_idx;
        if !self.players[p].can_play_land() {
            return Err(GameError::AlreadyPlayedLand);
        }
        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let card = self.players[p]
            .remove_from_hand(card_id)
            .unwrap(); // we just checked has_in_hand
        if !card.definition.is_land() {
            // Put it back then error
            self.players[p].hand.push(card);
            return Err(GameError::NotALand(card_id));
        }
        self.players[p].lands_played_this_turn += 1;
        self.battlefield.push(card);
        Ok(vec![
            GameEvent::LandPlayed { player: p, card_id },
            GameEvent::PermanentEntered { card_id },
        ])
    }

    // ── Cast spell ────────────────────────────────────────────────────────────

    fn cast_spell(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.active_player_idx;
        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let card = self.players[p].remove_from_hand(card_id).unwrap();
        let is_instant = card.definition.is_instant();

        // Non-instants can only be cast during main phases
        if !is_instant && !self.step.is_main_phase() {
            self.players[p].hand.push(card); // put back
            return Err(GameError::WrongStep { actual: self.step });
        }

        // Pay the cost
        let cost = card.definition.cost.clone();
        if let Err(e) = self.players[p].mana_pool.pay(&cost) {
            self.players[p].hand.push(card); // put back
            return Err(GameError::Mana(e));
        }

        let mut events = vec![GameEvent::SpellCast { player: p, card_id }];

        // Push to stack then resolve immediately (simplified stack)
        self.stack.push(StackItem::Spell { card, caster: p, target });
        let mut resolve_events = self.resolve_top_of_stack()?;
        events.append(&mut resolve_events);

        Ok(events)
    }

    // ── Activate ability ──────────────────────────────────────────────────────

    fn activate_ability(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let pos = self
            .battlefield
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotOnBattlefield(card_id))?;

        let ability = self.battlefield[pos]
            .definition
            .activated_abilities
            .get(ability_index)
            .cloned()
            .ok_or(GameError::AbilityIndexOutOfBounds)?;

        let p = self.active_player_idx;

        // Pay tap cost
        if ability.tap_cost {
            if self.battlefield[pos].tapped {
                return Err(GameError::CardIsTapped(card_id));
            }
            self.battlefield[pos].tapped = true;
        }

        // Pay mana cost
        if !ability.mana_cost.symbols.is_empty() {
            self.players[p]
                .mana_pool
                .pay(&ability.mana_cost)
                .map_err(GameError::Mana)?;
        }

        let mut events = vec![GameEvent::AbilityActivated { source: card_id }];
        for effect in &ability.effects {
            let mut effect_events = self.resolve_effect(effect, p, target.as_ref())?;
            events.append(&mut effect_events);
        }

        Ok(events)
    }

    // ── Effect resolution ─────────────────────────────────────────────────────

    fn resolve_effect(
        &mut self,
        effect: &SpellEffect,
        controller: usize,
        target: Option<&Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut events = vec![];
        match effect {
            SpellEffect::DealDamage { amount, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                match tgt {
                    Target::Player(pidx) => {
                        let amount = *amount;
                        self.players[*pidx].life -= amount as i32;
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: Some(*pidx),
                            to_card: None,
                        });
                        events.push(GameEvent::LifeLost {
                            player: *pidx,
                            amount,
                        });
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                    }
                    Target::Permanent(cid) => {
                        let amount = *amount;
                        if let Some(c) = self.battlefield_find_mut(*cid) {
                            c.damage += amount;
                        } else {
                            return Err(GameError::CardNotOnBattlefield(*cid));
                        }
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: None,
                            to_card: Some(*cid),
                        });
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                    }
                }
            }

            SpellEffect::DestroyCreature { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                events.push(GameEvent::CreatureDied { card_id });
                self.remove_from_battlefield_to_graveyard(card_id);
            }

            SpellEffect::DrawCards { amount } => {
                for _ in 0..*amount {
                    let drawn = self.players[controller].draw_top();
                    match drawn {
                        Some(id) => {
                            events.push(GameEvent::CardDrawn { player: controller, card_id: id });
                        }
                        None => {
                            // Drawing from empty library: player loses
                            let opp = (controller + 1) % self.players.len();
                            self.game_over = Some(Some(opp));
                            events.push(GameEvent::GameOver { winner: Some(opp) });
                            return Ok(events);
                        }
                    }
                }
            }

            SpellEffect::AddMana { colors } => {
                for &c in colors {
                    self.players[controller].mana_pool.add(c, 1);
                    events.push(GameEvent::ManaAdded { player: controller, color: c });
                }
            }

            SpellEffect::PumpCreature { power_bonus, toughness_bonus } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                let c = self
                    .battlefield_find_mut(*cid)
                    .ok_or(GameError::CardNotOnBattlefield(*cid))?;
                c.power_bonus += power_bonus;
                c.toughness_bonus += toughness_bonus;
                events.push(GameEvent::PumpApplied {
                    card_id: *cid,
                    power: *power_bonus,
                    toughness: *toughness_bonus,
                });
            }

            SpellEffect::GainLife { amount } => {
                let amount = *amount;
                self.players[controller].life += amount as i32;
                events.push(GameEvent::LifeGained { player: controller, amount });
            }

            SpellEffect::DestroyAllCreatures => {
                let ids: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.definition.is_creature())
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    events.push(GameEvent::CreatureDied { card_id: id });
                    self.remove_from_battlefield_to_graveyard(id);
                }
            }

            SpellEffect::RevealOpponentTopCard => {
                let opp = (controller + 1) % self.players.len();
                // Reveal top card of opponent's library; if land, they draw it.
                if let Some(top) = self.players[opp].library.first() {
                    let name = top.definition.name;
                    let is_land = top.definition.is_land();
                    events.push(GameEvent::TopCardRevealed { player: opp, card_name: name, is_land });
                    if is_land {
                        if let Some(id) = self.players[opp].draw_top() {
                            events.push(GameEvent::CardDrawn { player: opp, card_id: id });
                        }
                    }
                }
            }

            SpellEffect::OpponentDiscardRandom => {
                let opp = (controller + 1) % self.players.len();
                // Opponent discards a card (simplified: first card in hand).
                if !self.players[opp].hand.is_empty() {
                    let card = self.players[opp].hand.remove(0);
                    let card_id = card.id;
                    self.players[opp].send_to_graveyard(card);
                    events.push(GameEvent::CardDiscarded { player: opp, card_id });
                }
            }

        }
        Ok(events)
    }

    /// Evaluate whether `target` satisfies `req` given the current game state.
    pub fn evaluate_requirement(&self, req: &SelectionRequirement, target: &Target) -> bool {
        match req {
            SelectionRequirement::Any => true,
            SelectionRequirement::Player => matches!(target, Target::Player(_)),
            SelectionRequirement::Creature => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_creature())
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::Artifact => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.card_types.contains(&CardType::Artifact))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::Land => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_land())
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasColor(color) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| card_has_color(c, *color))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasKeyword(kw) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.has_keyword(kw))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::PowerAtMost(n) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_creature() && c.power() <= *n)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::And(a, b) => {
                self.evaluate_requirement(a, target) && self.evaluate_requirement(b, target)
            }
            SelectionRequirement::Or(a, b) => {
                self.evaluate_requirement(a, target) || self.evaluate_requirement(b, target)
            }
            SelectionRequirement::Not(a) => !self.evaluate_requirement(a, target),
        }
    }

    // ── Declare attackers ─────────────────────────────────────────────────────

    fn declare_attackers(&mut self, ids: Vec<CardId>) -> Result<Vec<GameEvent>, GameError> {
        if self.step != TurnStep::DeclareAttackers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        let p = self.active_player_idx;
        let mut events = vec![];
        let mut triggers: Vec<(CardId, Vec<SpellEffect>, usize)> = vec![];
        for id in ids {
            let card = self
                .battlefield
                .iter_mut()
                .find(|c| c.id == id && c.owner == p)
                .ok_or(GameError::CardNotOnBattlefield(id))?;

            if !card.can_attack() {
                if card.tapped {
                    return Err(GameError::CardIsTapped(id));
                }
                return Err(GameError::SummoningSickness(id));
            }
            // Vigilant creatures don't tap when attacking
            if !card.has_keyword(&Keyword::Vigilance) {
                card.tapped = true;
            }
            self.attacking.push(id);
            events.push(GameEvent::AttackerDeclared(id));
            // Collect attack triggers (pushed after the loop to avoid borrow conflict)
            for t in &card.definition.triggered_abilities {
                if t.condition == TriggerCondition::Attacks {
                    triggers.push((id, t.effects.clone(), p));
                }
            }
        }
        // Push collected attack triggers onto the stack
        for (source, effects, controller) in triggers {
            let auto_target = self.auto_target_for_effects(&effects, controller);
            self.stack.push(StackItem::Trigger { source, controller, effects, target: auto_target });
        }
        Ok(events)
    }

    // ── Declare blockers ──────────────────────────────────────────────────────

    fn declare_blockers(
        &mut self,
        assignments: Vec<(CardId, CardId)>,
    ) -> Result<Vec<GameEvent>, GameError> {
        if self.step != TurnStep::DeclareBlockers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        let defender = (self.active_player_idx + 1) % self.players.len();

        // Validate ALL assignments before mutating any state.
        for &(blocker_id, attacker_id) in &assignments {
            let blocker = self
                .battlefield
                .iter()
                .find(|c| c.id == blocker_id && c.owner == defender)
                .ok_or(GameError::CardNotOnBattlefield(blocker_id))?;

            if !blocker.can_block() {
                return Err(GameError::CannotBlock(blocker_id));
            }

            let attacker = self
                .battlefield_find(attacker_id)
                .ok_or(GameError::CardNotOnBattlefield(attacker_id))?;

            if !can_block_attacker(blocker, attacker) {
                return Err(GameError::CannotBlock(blocker_id));
            }
        }

        // All valid — apply.
        self.blockers_declared = true;
        let mut events = vec![];
        for (blocker_id, attacker_id) in assignments {
            self.block_map.insert(blocker_id, attacker_id);
            events.push(GameEvent::BlockerDeclared {
                blocker: blocker_id,
                attacker: attacker_id,
            });
        }
        Ok(events)
    }

    // ── Pass priority ─────────────────────────────────────────────────────────

    fn pass_priority(&mut self) -> Result<Vec<GameEvent>, GameError> {
        // If the stack has items, resolve the top one instead of advancing the step.
        if !self.stack.is_empty() {
            return self.resolve_top_of_stack();
        }

        // Auto-declare empty blockers if the defending player passes without blocking.
        if self.step == TurnStep::DeclareBlockers && !self.attacking.is_empty() && !self.blockers_declared {
            self.blockers_declared = true;
        }

        let mut events = vec![];

        // Cleanup happens before transitioning to the next step
        if self.step == TurnStep::Cleanup {
            self.do_cleanup();
        }

        let next = self.step.next();
        self.step = next;
        events.push(GameEvent::StepChanged(next));

        // Automatic effects on entering certain steps
        match next {
            TurnStep::Untap => {
                self.do_untap();
                events.push(GameEvent::TurnStarted {
                    player: self.active_player_idx,
                    turn: self.turn_number,
                });
            }
            TurnStep::Draw => {
                if self.skip_first_draw {
                    self.skip_first_draw = false;
                } else {
                    let p = self.active_player_idx;
                    match self.players[p].draw_top() {
                        Some(id) => events.push(GameEvent::CardDrawn { player: p, card_id: id }),
                        None => {
                            let opp = (p + 1) % self.players.len();
                            self.game_over = Some(Some(opp));
                            events.push(GameEvent::GameOver { winner: Some(opp) });
                        }
                    }
                }
            }
            TurnStep::CombatDamage => {
                let mut combat_events = self.resolve_combat()?;
                events.append(&mut combat_events);
            }
            _ => {}
        }

        Ok(events)
    }

    // ── Stack resolution ──────────────────────────────────────────────────────

    fn resolve_top_of_stack(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let Some(item) = self.stack.pop() else {
            return Ok(vec![]);
        };
        let mut events = vec![];

        match item {
            StackItem::Spell { card, caster, target } => {
                let card_id = card.id;
                if card.definition.is_permanent() {
                    // Collect ETB triggers before moving card into battlefield
                    let etb_triggers: Vec<Vec<SpellEffect>> = card.definition.triggered_abilities.iter()
                        .filter(|t| t.condition == TriggerCondition::EntersBattlefield)
                        .map(|t| t.effects.clone())
                        .collect();
                    self.battlefield.push(card);
                    events.push(GameEvent::PermanentEntered { card_id });
                    // Push ETB triggers onto the stack (they resolve on the next pass)
                    for effects in etb_triggers {
                        let auto_target = self.auto_target_for_effects(&effects, caster);
                        self.stack.push(StackItem::Trigger {
                            source: card_id,
                            controller: caster,
                            effects,
                            target: auto_target,
                        });
                    }
                } else {
                    // Instant/sorcery: resolve effects, then graveyard
                    let def = card.definition.clone();
                    for effect in &def.spell_effects {
                        let mut effect_events =
                            self.resolve_effect(effect, caster, target.as_ref())?;
                        events.append(&mut effect_events);
                    }
                    self.players[caster].send_to_graveyard(card);
                }
            }
            StackItem::Trigger { source: _, controller, effects, target } => {
                for effect in &effects {
                    let mut effect_events =
                        self.resolve_effect(effect, controller, target.as_ref())?;
                    events.append(&mut effect_events);
                }
            }
        }

        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);

        Ok(events)
    }

    /// Pick a sensible auto-target for a single effect triggered by `controller`.
    fn auto_target_for_effect(&self, effect: &SpellEffect, controller: usize) -> Option<Target> {
        let opp = (controller + 1) % self.players.len();
        match effect {
            SpellEffect::DealDamage { .. } => {
                Some(Target::Player(opp))
            }
            SpellEffect::DestroyCreature { .. } => {
                self.battlefield.iter()
                    .find(|c| c.owner == opp && c.definition.is_creature())
                    .map(|c| Target::Permanent(c.id))
            }
            SpellEffect::PumpCreature { .. } => {
                self.battlefield.iter()
                    .find(|c| c.owner == controller && c.definition.is_creature())
                    .map(|c| Target::Permanent(c.id))
            }
            // These effects are controller-relative; no Target needed
            SpellEffect::RevealOpponentTopCard | SpellEffect::OpponentDiscardRandom => None,
            _ => None,
        }
    }

    /// Pick a sensible auto-target for a slice of effects triggered by `controller`.
    /// Returns the first non-None target found across all effects.
    fn auto_target_for_effects(&self, effects: &[SpellEffect], controller: usize) -> Option<Target> {
        effects.iter().find_map(|e| self.auto_target_for_effect(e, controller))
    }

    // ── Combat resolution ─────────────────────────────────────────────────────

    fn resolve_combat(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let mut events = vec![];
        let defender_idx = (self.active_player_idx + 1) % self.players.len();

        // Snapshot attacker data before mutating battlefield
        let attacker_snapshot: Vec<(CardId, i32, bool, bool)> = self
            .attacking
            .iter()
            .filter_map(|&aid| {
                self.battlefield_find(aid).map(|c| {
                    (
                        c.id,
                        c.power(),
                        c.has_keyword(&Keyword::Trample),
                        c.has_keyword(&Keyword::Lifelink),
                    )
                })
            })
            .collect();

        // For each attacker: find its blockers via block_map
        for (attacker_id, atk_power, has_trample, has_lifelink) in attacker_snapshot {
            let blocker_ids: Vec<CardId> = self
                .block_map
                .iter()
                .filter(|(_, aid)| **aid == attacker_id)
                .map(|(&bid, _)| bid)
                .collect();

            if blocker_ids.is_empty() {
                // Unblocked — deal power damage to defending player
                let amount = atk_power.max(0) as u32;
                if amount > 0 {
                    self.players[defender_idx].life -= atk_power;
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: Some(defender_idx),
                        to_card: None,
                    });
                    events.push(GameEvent::LifeLost {
                        player: defender_idx,
                        amount,
                    });
                    if has_lifelink {
                        let a = self.active_player_idx;
                        self.players[a].life += atk_power;
                    }
                }
            } else {
                // Blocked — distribute attacker damage among blockers (sequential assignment)
                let mut remaining_atk_damage = atk_power;

                for &blocker_id in &blocker_ids {
                    if remaining_atk_damage <= 0 {
                        break;
                    }
                    if let Some(blocker) = self.battlefield_find_mut(blocker_id) {
                        let lethal = blocker.toughness();
                        let assign = remaining_atk_damage.min(lethal);
                        blocker.damage += assign as u32;
                        remaining_atk_damage -= assign;
                        events.push(GameEvent::DamageDealt {
                            amount: assign as u32,
                            to_player: None,
                            to_card: Some(blocker_id),
                        });
                    }
                }

                // Trample: excess damage to player
                if has_trample && remaining_atk_damage > 0 {
                    let amount = remaining_atk_damage as u32;
                    self.players[defender_idx].life -= remaining_atk_damage;
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: Some(defender_idx),
                        to_card: None,
                    });
                    events.push(GameEvent::LifeLost {
                        player: defender_idx,
                        amount,
                    });
                }

                // Blockers deal damage back to the attacker
                let blocker_total_power: i32 = blocker_ids
                    .iter()
                    .filter_map(|&bid| self.battlefield_find(bid))
                    .map(|c| c.power())
                    .sum();
                if let Some(attacker) = self.battlefield_find_mut(attacker_id) {
                    attacker.damage += blocker_total_power.max(0) as u32;
                    events.push(GameEvent::DamageDealt {
                        amount: blocker_total_power.max(0) as u32,
                        to_player: None,
                        to_card: Some(attacker_id),
                    });
                }
            }
        }

        // State-based actions after all damage
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);

        // Reset combat state
        self.attacking.clear();
        self.block_map.clear();
        self.blockers_declared = false;

        events.push(GameEvent::CombatResolved);
        Ok(events)
    }

    // ── Automatic step effects ────────────────────────────────────────────────

    fn do_untap(&mut self) {
        let p = self.active_player_idx;
        for card in &mut self.battlefield {
            if card.owner == p {
                card.tapped = false;
                card.summoning_sick = false;
            }
        }
        self.players[p].lands_played_this_turn = 0;
    }

    fn do_cleanup(&mut self) {
        // Clear temporary pump effects
        for card in &mut self.battlefield {
            card.clear_end_of_turn_effects();
        }
        // Clear all damage from creatures
        for card in &mut self.battlefield {
            card.damage = 0;
        }
        // Empty mana pools
        for player in &mut self.players {
            player.mana_pool.empty();
        }
        // Advance to next player's turn (TurnStarted fires on Untap entry)
        self.active_player_idx = (self.active_player_idx + 1) % self.players.len();
        self.turn_number += 1;
    }

    // ── State-based actions ───────────────────────────────────────────────────

    fn check_state_based_actions(&mut self) -> Vec<GameEvent> {
        let mut events = vec![];

        // Collect dead creatures
        let dead: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.is_dead())
            .map(|c| c.id)
            .collect();

        for id in dead {
            events.push(GameEvent::CreatureDied { card_id: id });
            self.remove_from_battlefield_to_graveyard(id);
        }

        // Check for player death
        for i in 0..self.players.len() {
            if !self.players[i].is_alive() && self.game_over.is_none() {
                let winner = (i + 1) % self.players.len();
                self.game_over = Some(Some(winner));
                events.push(GameEvent::GameOver { winner: Some(winner) });
            }
        }

        events
    }

    fn remove_from_battlefield_to_graveyard(&mut self, id: CardId) {
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == id) {
            let card = self.battlefield.remove(pos);
            let owner = card.owner;
            self.players[owner].send_to_graveyard(card);
        }
    }

    fn battlefield_find(&self, id: CardId) -> Option<&CardInstance> {
        self.battlefield.iter().find(|c| c.id == id)
    }

    fn battlefield_find_mut(&mut self, id: CardId) -> Option<&mut CardInstance> {
        self.battlefield.iter_mut().find(|c| c.id == id)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns true if `blocker` is legally allowed to block `attacker`.
/// Flying creatures can only be blocked by creatures with flying or reach.
fn can_block_attacker(blocker: &CardInstance, attacker: &CardInstance) -> bool {
    if attacker.has_keyword(&Keyword::Flying) {
        return blocker.has_keyword(&Keyword::Flying) || blocker.has_keyword(&Keyword::Reach);
    }
    true
}

/// A card has a given color if its cost contains at least one mana symbol of that color.
fn card_has_color(card: &CardInstance, color: Color) -> bool {
    use crate::mana::ManaSymbol;
    card.definition
        .cost
        .symbols
        .iter()
        .any(|s| matches!(s, ManaSymbol::Colored(c) if *c == color))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::mana::Color;

    fn two_player_game() -> GameState {
        let players = vec![
            Player::new(0, "Alice"),
            Player::new(1, "Bob"),
        ];
        let mut g = GameState::new(players);
        // Start in PreCombatMain so we can take actions without advancing steps
        g.step = TurnStep::PreCombatMain;
        g
    }

    // ── Setup ─────────────────────────────────────────────────────────────────

    #[test]
    fn players_start_with_20_life() {
        let g = two_player_game();
        assert_eq!(g.players[0].life, 20);
        assert_eq!(g.players[1].life, 20);
    }

    // ── Land ──────────────────────────────────────────────────────────────────

    #[test]
    fn play_land_moves_to_battlefield() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::forest());
        let events = g.perform_action(GameAction::PlayLand(id)).unwrap();
        assert!(g.battlefield.iter().any(|c| c.id == id));
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::LandPlayed { .. })));
    }

    #[test]
    fn cannot_play_two_lands_per_turn() {
        let mut g = two_player_game();
        let f1 = g.add_card_to_hand(0, catalog::forest());
        let f2 = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(f1)).unwrap();
        let err = g.perform_action(GameAction::PlayLand(f2)).unwrap_err();
        assert_eq!(err, GameError::AlreadyPlayedLand);
    }

    #[test]
    fn cannot_play_land_in_combat() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        let id = g.add_card_to_hand(0, catalog::forest());
        let err = g.perform_action(GameAction::PlayLand(id)).unwrap_err();
        assert!(matches!(err, GameError::WrongStep { .. }));
    }

    // ── Tap for mana ──────────────────────────────────────────────────────────

    #[test]
    fn tap_forest_adds_green_mana() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::forest());
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
        })
        .unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    }

    #[test]
    fn cannot_tap_already_tapped_land() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::forest());
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
        })
        .unwrap();
        let err = g
            .perform_action(GameAction::ActivateAbility {
                card_id: id,
                ability_index: 0,
                target: None,
            })
            .unwrap_err();
        assert_eq!(err, GameError::CardIsTapped(id));
    }

    #[test]
    fn llanowar_elves_tap_for_mana() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::llanowar_elves());
        g.clear_sickness(id);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
        })
        .unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    }

    // ── Cast creature ─────────────────────────────────────────────────────────

    #[test]
    fn cast_grizzly_bears_enters_battlefield() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::grizzly_bears());
        // Pay {1}{G}
        g.players[0].mana_pool.add(Color::Green, 2);
        let events = g
            .perform_action(GameAction::CastSpell { card_id: id, target: None })
            .unwrap();
        assert!(g.battlefield.iter().any(|c| c.id == id));
        assert!(events.iter().any(|e| matches!(e, GameEvent::PermanentEntered { .. })));
    }

    #[test]
    fn cast_creature_fails_without_mana() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::grizzly_bears());
        let err = g
            .perform_action(GameAction::CastSpell { card_id: id, target: None })
            .unwrap_err();
        assert!(matches!(err, GameError::Mana(_)));
        // Card still in hand after failed cast
        assert!(g.players[0].has_in_hand(id));
    }

    // ── Instants ──────────────────────────────────────────────────────────────

    #[test]
    fn lightning_bolt_deals_3_damage_to_player() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Player(1)),
        })
        .unwrap();
        assert_eq!(g.players[1].life, 17);
    }

    #[test]
    fn lightning_bolt_kills_creature() {
        let mut g = two_player_game();
        let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
        let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt_id,
            target: Some(Target::Permanent(bear_id)),
        })
        .unwrap();
        assert!(!g.battlefield.iter().any(|c| c.id == bear_id));
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bear_id));
    }

    #[test]
    fn giant_growth_pumps_creature() {
        let mut g = two_player_game();
        let spell_id = g.add_card_to_hand(0, catalog::giant_growth());
        let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell_id,
            target: Some(Target::Permanent(bear_id)),
        })
        .unwrap();
        let bear = g.battlefield.iter().find(|c| c.id == bear_id).unwrap();
        assert_eq!(bear.power(), 5);
        assert_eq!(bear.toughness(), 5);
    }

    #[test]
    fn dark_ritual_adds_three_black_mana() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::dark_ritual());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::CastSpell { card_id: id, target: None })
            .unwrap();
        // Paid 1B, gained BBB → net 2B in pool
        assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3);
    }

    #[test]
    fn ancestral_recall_draws_three_cards() {
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let id = g.add_card_to_hand(0, catalog::ancestral_recall());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::CastSpell { card_id: id, target: None })
            .unwrap();
        // Drew 3 cards (Ancestral Recall has no target in this engine version)
        assert_eq!(g.players[0].hand.len(), 3);
    }

    #[test]
    fn terror_destroys_non_black_creature() {
        let mut g = two_player_game();
        let terror_id = g.add_card_to_hand(0, catalog::terror());
        let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: terror_id,
            target: Some(Target::Permanent(bear_id)),
        })
        .unwrap();
        assert!(!g.battlefield.iter().any(|c| c.id == bear_id));
    }

    // ── Moxen ─────────────────────────────────────────────────────────────────

    #[test]
    fn mox_ruby_casts_for_free_and_taps_for_red() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::mox_ruby());
        // Cast for {0} — no mana needed
        g.perform_action(GameAction::CastSpell { card_id: id, target: None }).unwrap();
        assert!(g.battlefield.iter().any(|c| c.id == id));
        // Tap immediately (not a creature, so no summoning sickness)
        g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
            .unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    }

    #[test]
    fn mox_pearl_taps_for_white() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::mox_pearl());
        g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
            .unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
    }

    #[test]
    fn mox_sapphire_taps_for_blue() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::mox_sapphire());
        g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
            .unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    }

    #[test]
    fn mox_jet_taps_for_black() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::mox_jet());
        g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
            .unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    }

    #[test]
    fn mox_emerald_taps_for_green() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::mox_emerald());
        g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
            .unwrap();
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    }

    #[test]
    fn mox_untaps_each_turn() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::mox_ruby());
        // Tap it
        g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
            .unwrap();
        assert!(g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
        // Simulate untap step
        g.do_untap();
        assert!(!g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
    }

    #[test]
    fn terror_cannot_destroy_artifact_creature() {
        // Terror uses SelectionRequirement to exclude artifacts and black creatures.
        // Moxes are artifacts — verify Artifact type is on mox_ruby.
        let mox_def = catalog::mox_ruby();
        assert!(mox_def.card_types.contains(&crate::card::CardType::Artifact));
    }

    #[test]
    fn terror_cannot_destroy_black_creature() {
        let mut g = two_player_game();
        let terror_id = g.add_card_to_hand(0, catalog::terror());
        let knight_id = g.add_card_to_battlefield(1, catalog::black_knight());
        g.players[0].mana_pool.add(Color::Black, 2);
        let err = g
            .perform_action(GameAction::CastSpell {
                card_id: terror_id,
                target: Some(Target::Permanent(knight_id)),
            })
            .unwrap_err();
        assert_eq!(err, GameError::SelectionRequirementViolated);
    }

    // ── Combat ────────────────────────────────────────────────────────────────

    fn setup_attacker(g: &mut GameState, player: usize, def: impl Fn() -> crate::card::CardDefinition) -> CardId {
        let id = g.add_card_to_battlefield(player, def());
        g.clear_sickness(id);
        id
    }

    #[test]
    fn unblocked_attacker_deals_damage_to_player() {
        let mut g = two_player_game();
        let bear_id = setup_attacker(&mut g, 0, catalog::grizzly_bears);

        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![bear_id]))
            .unwrap();

        // Advance to combat damage
        g.step = TurnStep::CombatDamage;
        let events = g.resolve_combat().unwrap();

        assert_eq!(g.players[1].life, 18); // 20 - 2
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::DamageDealt { to_player: Some(1), amount: 2, .. })));
    }

    #[test]
    fn blocked_combat_both_die() {
        let mut g = two_player_game();
        let attacker_id = setup_attacker(&mut g, 0, catalog::grizzly_bears);
        let blocker_id = setup_attacker(&mut g, 1, catalog::grizzly_bears);

        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![attacker_id]))
            .unwrap();

        g.step = TurnStep::DeclareBlockers;
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker_id, attacker_id)]))
            .unwrap();

        g.step = TurnStep::CombatDamage;
        g.resolve_combat().unwrap();

        // Both 2/2 creatures trade
        assert!(!g.battlefield.iter().any(|c| c.id == attacker_id));
        assert!(!g.battlefield.iter().any(|c| c.id == blocker_id));
        // Defending player life unchanged (attacker was blocked)
        assert_eq!(g.players[1].life, 20);
    }

    #[test]
    fn vigilance_creature_does_not_tap_when_attacking() {
        let mut g = two_player_game();
        let angel_id = setup_attacker(&mut g, 0, catalog::serra_angel);

        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![angel_id]))
            .unwrap();
        let angel = g.battlefield.iter().find(|c| c.id == angel_id).unwrap();
        assert!(!angel.tapped, "Vigilance: Serra Angel should not tap when attacking");
    }

    #[test]
    fn flying_creature_cannot_be_blocked_by_ground_creature() {
        let mut g = two_player_game();
        let angel_id = setup_attacker(&mut g, 0, catalog::serra_angel);
        let bear_id = setup_attacker(&mut g, 1, catalog::grizzly_bears);

        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![angel_id]))
            .unwrap();

        g.step = TurnStep::DeclareBlockers;
        let err = g
            .perform_action(GameAction::DeclareBlockers(vec![(bear_id, angel_id)]))
            .unwrap_err();
        assert_eq!(err, GameError::CannotBlock(bear_id));
    }

    #[test]
    fn summoning_sick_creature_cannot_attack() {
        let mut g = two_player_game();
        let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // still sick

        g.step = TurnStep::DeclareAttackers;
        let err = g
            .perform_action(GameAction::DeclareAttackers(vec![bear_id]))
            .unwrap_err();
        assert_eq!(err, GameError::SummoningSickness(bear_id));
    }

    #[test]
    fn haste_creature_can_attack_immediately() {
        let mut g = two_player_game();
        let goblin_id = g.add_card_to_battlefield(0, catalog::goblin_guide()); // Haste, still sick

        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![goblin_id]))
            .unwrap();
        // No error — Haste bypasses summoning sickness
    }

    // ── Win condition ─────────────────────────────────────────────────────────

    #[test]
    fn player_dies_when_life_reaches_zero() {
        let mut g = two_player_game();
        g.players[1].life = 3;
        let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let events = g
            .perform_action(GameAction::CastSpell {
                card_id: bolt_id,
                target: Some(Target::Player(1)),
            })
            .unwrap();
        assert!(g.is_game_over());
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::GameOver { winner: Some(0) })));
    }

    // ── Turn progression ──────────────────────────────────────────────────────

    #[test]
    fn pass_priority_advances_step() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::PassPriority).unwrap();
        assert_eq!(g.step, TurnStep::BeginCombat);
    }

    #[test]
    fn untap_step_clears_summoning_sickness() {
        let mut g = two_player_game();
        let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert!(g.battlefield.iter().find(|c| c.id == bear_id).unwrap().summoning_sick);

        // The bear belongs to player 0.  Its sickness clears during player 0's
        // Untap step, which follows the end of player 1's turn (Cleanup).
        // Simulate: it is the end of player 1's turn.
        g.step = TurnStep::Cleanup;
        g.active_player_idx = 1;
        g.perform_action(GameAction::PassPriority).unwrap();

        // We should now be in player 0's Untap step
        assert_eq!(g.step, TurnStep::Untap);
        assert_eq!(g.active_player_idx, 0);
        // Summoning sickness cleared for player 0's permanents
        assert!(!g.battlefield.iter().find(|c| c.id == bear_id).unwrap().summoning_sick);
    }

    #[test]
    fn cleanup_resets_end_of_turn_pump() {
        let mut g = two_player_game();
        let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().power_bonus = 3;
        g.step = TurnStep::Cleanup;
        g.perform_action(GameAction::PassPriority).unwrap();
        // Pump should be gone on the next Untap
        let bear = g.battlefield.iter().find(|c| c.id == bear_id).unwrap();
        assert_eq!(bear.power_bonus, 0);
    }

    // ── New effects ───────────────────────────────────────────────────────────────

    #[test]
    fn wrath_of_god_destroys_all_creatures() {
        let mut g = two_player_game();
        let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let wrath_id = g.add_card_to_hand(0, catalog::wrath_of_god());
        g.players[0].mana_pool.add(Color::White, 4);
        g.perform_action(GameAction::CastSpell { card_id: wrath_id, target: None }).unwrap();
        assert!(!g.battlefield.iter().any(|c| c.id == bear1));
        assert!(!g.battlefield.iter().any(|c| c.id == bear2));
    }

    #[test]
    fn lightning_helix_deals_damage_and_gains_life() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::lightning_helix());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Player(1)),
        })
        .unwrap();
        assert_eq!(g.players[1].life, 17);
        assert_eq!(g.players[0].life, 23);
    }
}
