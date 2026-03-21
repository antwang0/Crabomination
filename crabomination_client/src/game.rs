use bevy::prelude::*;
use crabomination::{
    card::{CardDefinition, CardId},
    catalog::*,
    game::{GameEvent, GameState},
    mana::{Color as ManaColor, ManaCost},
    player::Player,
};
use rand::seq::SliceRandom;

pub const HUMAN: usize = 0;
pub const BOT: usize = 1;

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
    if idx == HUMAN { "You" } else { "Bot" }
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
pub struct BotTimer(pub Timer);

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

/// State for the graveyard card browser popup.
#[derive(Resource, Default)]
pub struct GraveyardBrowserState {
    pub open: bool,
    pub owner: usize,
}

/// Tracks the human player's blocker assignments during the DeclareBlockers step.
#[derive(Resource, Default)]
pub struct BlockingState {
    /// The human creature the player clicked to block with.
    pub selected_blocker: Option<CardId>,
    /// Confirmed (blocker_id, attacker_id) assignments to submit on Pass.
    pub assignments: Vec<(CardId, CardId)>,
}

type CardFactory = fn() -> CardDefinition;

/// Build a fresh game with two players, each with a shuffled deck and 7-card opening hand.
///
/// Human — Red/White aggro: burn, weenies, Lightning Helix, Wrath of God.
/// Bot   — Black/Red: Dark Ritual, discard threats, removal, big vampires.
pub fn build_game() -> GameResource {
    let mut state = GameState::new(vec![Player::new(HUMAN, "You"), Player::new(BOT, "Bot")]);

    // ── Human: Red/White (33 cards) ───────────────────────────────────────────
    let human_deck: &[CardFactory] = &[
        // Lands (12)
        plains, plains, plains, plains, plains, plains,
        mountain, mountain, mountain, mountain, mountain, mountain,
        // Creatures (11)
        savannah_lions, savannah_lions,
        white_knight, white_knight,
        hopeful_eidolon, hopeful_eidolon,
        goblin_guide, goblin_guide,
        serra_angel, serra_angel,
        shivan_dragon,
        // Spells (10)
        lightning_bolt, lightning_bolt, lightning_bolt, lightning_bolt,
        lightning_helix, lightning_helix,
        shock, shock, shock,
        wrath_of_god,
    ];

    // ── Bot: Black/Red (32 cards) ─────────────────────────────────────────────
    let bot_deck: &[CardFactory] = &[
        // Lands (12)
        swamp, swamp, swamp, swamp, swamp, swamp,
        mountain, mountain, mountain, mountain, mountain, mountain,
        // Creatures (8)
        black_knight, black_knight,
        goblin_guide, goblin_guide,
        hypnotic_specter, hypnotic_specter,
        sengir_vampire, sengir_vampire,
        // Spells (12)
        dark_ritual, dark_ritual, dark_ritual,
        lightning_bolt, lightning_bolt, lightning_bolt, lightning_bolt,
        terror, terror,
        shock, shock,
        terminate,
    ];

    let mut rng = rand::rng();

    for &f in human_deck { state.add_card_to_library(HUMAN, f()); }
    state.players[HUMAN].library.shuffle(&mut rng);
    for _ in 0..7 { state.players[HUMAN].draw_top(); }

    for &f in bot_deck { state.add_card_to_library(BOT, f()); }
    state.players[BOT].library.shuffle(&mut rng);
    for _ in 0..7 { state.players[BOT].draw_top(); }

    GameResource { state }
}
