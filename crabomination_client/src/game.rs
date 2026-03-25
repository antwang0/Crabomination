use bevy::time::Timer;
use bevy::prelude::*;
use crabomination::{
    card::{CardDefinition, CardId},
    catalog::*,
    game::{GameEvent, GameState},
    mana::{Color as ManaColor, ManaCost},
    player::Player,
};
use rand::seq::SliceRandom;

pub const PLAYER_0: usize = 0;
pub const PLAYER_1: usize = 1;

#[derive(Resource)]
pub struct GameResource {
    pub state: GameState,
}

#[derive(Resource, Default)]
pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn push(&mut self, msg: impl Into<String>) {
        self.entries.push(msg.into());
        if self.entries.len() > 16 {
            self.entries.remove(0);
        }
    }

    pub fn apply_events(&mut self, events: &[GameEvent]) {
        let mut pending_step: Option<String> = None;
        for event in events {
            if matches!(event, GameEvent::StepChanged(_)) {
                // Buffer the step label; only flush it if something happens in this step
                pending_step = Some(describe_event(event));
            } else {
                if let Some(step_msg) = pending_step.take() {
                    self.push(step_msg);
                }
                self.push(describe_event(event));
            }
        }
        // Trailing StepChanged with nothing following = empty phase, discard it
    }
}

fn player_label(idx: usize) -> &'static str {
    if idx == PLAYER_0 { "Player 0" } else { "Player 1" }
}

fn describe_event(event: &GameEvent) -> String {
    match event {
        GameEvent::StepChanged(step) => format!(">>> {step:?}"),
        GameEvent::TurnStarted { player, turn } => {
            format!("Turn {turn}: {}'s turn", player_label(*player))
        }
        GameEvent::CardDrawn { player, .. } => format!("{} draws", player_label(*player)),
        GameEvent::CardDiscarded { player, .. } => format!("{} discards", player_label(*player)),
        GameEvent::LandPlayed { player, .. } => format!("{} plays land", player_label(*player)),
        GameEvent::SpellCast { player, .. } => format!("{} casts spell", player_label(*player)),
        GameEvent::AbilityActivated { .. } => "Ability activated".into(),
        GameEvent::ManaAdded { player, color } => {
            format!("{} adds {:?}", player_label(*player), color)
        }
        GameEvent::PermanentEntered { .. } => "Permanent enters".into(),
        GameEvent::DamageDealt { amount, to_player, .. } => {
            if let Some(p) = to_player {
                format!("{amount} dmg to {}", player_label(*p))
            } else {
                format!("{amount} dmg to creature")
            }
        }
        GameEvent::LifeLost { player, amount } => {
            format!("{} loses {amount} life", player_label(*player))
        }
        GameEvent::LifeGained { player, amount } => {
            format!("{} gains {amount} life", player_label(*player))
        }
        GameEvent::CreatureDied { .. } => "Creature died".into(),
        GameEvent::PumpApplied { power, toughness, .. } => {
            format!("+{power}/+{toughness} until EOT")
        }
        GameEvent::AttackerDeclared(_) => "Attacker declared".into(),
        GameEvent::BlockerDeclared { .. } => "Blocker declared".into(),
        GameEvent::CombatResolved => "Combat resolved".into(),
        GameEvent::TopCardRevealed { player, card_name, is_land } => {
            let suffix = if *is_land { " (land — drawn!)" } else { " (not a land)" };
            format!("Revealed: {} for {}{}", card_name, player_label(*player), suffix)
        }
        GameEvent::GameOver { winner } => match winner {
            Some(p) => format!("GAME OVER: {} wins!", player_label(*p)),
            None => "GAME OVER: Draw!".into(),
        },
    }
}

/// Format a player's mana pool as a compact string, e.g. "R:3".
pub fn format_mana_pool(state: &GameState, player_idx: usize) -> String {
    let pool = &state.players[player_idx].mana_pool;
    let colors = [
        (ManaColor::White, 'W'),
        (ManaColor::Blue, 'U'),
        (ManaColor::Black, 'B'),
        (ManaColor::Red, 'R'),
        (ManaColor::Green, 'G'),
    ];
    let parts: Vec<String> = colors
        .iter()
        .filter_map(|(c, sym)| {
            let n = pool.amount(*c);
            if n > 0 { Some(format!("{sym}:{n}")) } else { None }
        })
        .collect();
    if parts.is_empty() { "0".into() } else { parts.join(" ") }
}

#[derive(Resource)]
pub struct P1Timer(pub Timer);

/// Tracks the "targeting mode" UI state when a spell needs a player-chosen target.
#[derive(Resource, Default)]
pub struct TargetingState {
    /// Whether we're currently waiting for the player to pick a target.
    pub active: bool,
    /// The spell card the player is trying to cast.
    pub pending_card_id: Option<CardId>,
    /// Full mana cost of the pending spell (for color-aware auto-tapping).
    pub pending_cost: ManaCost,
}

/// Tracks the opening-hand mulligan phase (before the game starts).
#[derive(Resource)]
pub struct MulliganState {
    /// Whether the mulligan phase is currently active.
    pub active: bool,
    /// Whether initial 7-card hands have been dealt yet.
    pub initial_deal_done: bool,
    /// Whether player 0 has decided to keep their current hand.
    pub p0_kept: bool,
    /// Whether player 1 has decided to keep their current hand.
    pub p1_kept: bool,
    /// Number of times player 0 has mulliganed.
    pub p0_mulligans: usize,
    /// Number of times player 1 has mulliganed.
    pub p1_mulligans: usize,
    /// Timer controlling when the bot (P1) makes its mulligan decision.
    pub p1_timer: Timer,
    /// London mulligan: how many cards P0 still needs to click to send to the bottom.
    /// Non-zero means the bottoming phase is active.
    pub p0_cards_to_bottom: usize,
}

impl Default for MulliganState {
    fn default() -> Self {
        Self {
            active: true,
            initial_deal_done: false,
            p0_kept: false,
            p1_kept: false,
            p0_mulligans: 0,
            p1_mulligans: 0,
            p1_timer: Timer::from_seconds(2.0, bevy::time::TimerMode::Once),
            p0_cards_to_bottom: 0,
        }
    }
}

/// State for the graveyard card browser popup.
#[derive(Resource, Default)]
pub struct GraveyardBrowserState {
    pub open: bool,
    pub owner: usize,
}

/// Tracks player 0's blocker assignments during the DeclareBlockers step.
#[derive(Resource, Default)]
pub struct BlockingState {
    /// The player 0 creature the player clicked to block with.
    pub selected_blocker: Option<CardId>,
    /// Confirmed (blocker_id, attacker_id) assignments to submit on Pass.
    pub assignments: Vec<(CardId, CardId)>,
}

type CardFactory = fn() -> CardDefinition;

/// Build a fresh game with two players, each with a shuffled deck and 7-card opening hand.
///
/// Player 0 — Red/White aggro (60 cards): burn, weenies, Lightning Helix, Wrath of God.
/// Player 1 — Black/Red midrange (60 cards): Dark Ritual, discard threats, removal, vampires.
pub fn build_game() -> GameResource {
    let mut state = GameState::new(vec![Player::new(PLAYER_0, "Player 0"), Player::new(PLAYER_1, "Player 1")]);

    // ── Player 0: Red/White Aggro (60 cards) ──────────────────────────────────
    let p0_deck: &[CardFactory] = &[
        // Lands (24)
        plains, plains, plains, plains, plains, plains,
        plains, plains, plains, plains,
        mountain, mountain, mountain, mountain, mountain, mountain,
        mountain, mountain, mountain, mountain, mountain, mountain,
        mountain, mountain,
        // Creatures (20)
        savannah_lions, savannah_lions, savannah_lions, savannah_lions,
        white_knight, white_knight, white_knight, white_knight,
        hopeful_eidolon, hopeful_eidolon, hopeful_eidolon, hopeful_eidolon,
        goblin_guide, goblin_guide, goblin_guide, goblin_guide,
        serra_angel, serra_angel,
        shivan_dragon, shivan_dragon,
        // Spells (16)
        lightning_bolt, lightning_bolt, lightning_bolt, lightning_bolt,
        lightning_helix, lightning_helix, lightning_helix, lightning_helix,
        shock, shock, shock, shock,
        wrath_of_god, wrath_of_god, wrath_of_god, wrath_of_god,
    ];

    // ── Player 1: Black/Red Midrange (60 cards) ───────────────────────────────
    let p1_deck: &[CardFactory] = &[
        // Lands (24)
        swamp, swamp, swamp, swamp, swamp, swamp,
        swamp, swamp, swamp, swamp, swamp, swamp,
        swamp, swamp,
        mountain, mountain, mountain, mountain, mountain, mountain,
        mountain, mountain, mountain, mountain,
        // Creatures (16)
        black_knight, black_knight, black_knight, black_knight,
        goblin_guide, goblin_guide, goblin_guide, goblin_guide,
        hypnotic_specter, hypnotic_specter, hypnotic_specter, hypnotic_specter,
        sengir_vampire, sengir_vampire, sengir_vampire, sengir_vampire,
        // Spells (20)
        dark_ritual, dark_ritual, dark_ritual, dark_ritual,
        lightning_bolt, lightning_bolt, lightning_bolt, lightning_bolt,
        terror, terror, terror, terror,
        shock, shock, shock, shock,
        terminate, terminate, terminate, terminate,
    ];

    let mut rng = rand::rng();

    for &f in p0_deck { state.add_card_to_library(PLAYER_0, f()); }
    state.players[PLAYER_0].library.shuffle(&mut rng);

    for &f in p1_deck { state.add_card_to_library(PLAYER_1, f()); }
    state.players[PLAYER_1].library.shuffle(&mut rng);

    // Initial hands are dealt during the mulligan phase by mulligan_system.

    GameResource { state }
}
