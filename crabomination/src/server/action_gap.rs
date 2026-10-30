//! The legal-action gap: how often the pilot's move is not the best legal
//! move, by the search's own judgement, and which class the better move was
//! (`bot_puzzles --action-gap`).
//!
//! The question it answers. The search weighs only its own main phase —
//! up to six casts, the land drop and a pass
//! (`bot::main_phase_candidates_for_mcts`) — and every other choice is the
//! heuristic's: attacks, blocks, activated abilities, plays on the
//! opponent's turn. Whether that menu misses the right move is an empirical
//! question, and this is the measurement.
//!
//! # The method
//!
//! A game is played with the same pilot on both seats. At a sampled
//! decision point the pilot's action is recorded, then every action
//! [`puzzle::legal_actions`] enumerates (the engine-validated list, mana
//! abilities aside; pending prompts are not enumerated and not sampled)
//! is scored by [`MctsBot::probe_actions`]: `n` rollouts each, the search's
//! own rollout and leaf.
//!
//! Picking the best of many noisy means and reading its value off the same
//! sample overstates it — the winner's curse, and here it would read as a
//! gap that is not there. So the probe runs **twice, independently**: pass
//! A picks the best action, pass B prices it and the pilot's. The regret
//! `B(best_A) − B(chosen)` has expectation ≤ the true regret and zero when
//! every action is equal, so a positive mean is a real gap rather than noise.
//!
//! What "best" means: best under the search's rollout policy and leaf at a
//! full menu and a fixed budget. It is the search's judgement made wider,
//! not ground truth — a move it can't value (one whose payoff needs a turn
//! past the horizon) is invisible here too.

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::game::types::TurnStep;
use crate::game::{GameAction, GameState};

use super::bot::Bot;
use super::mcts::{MctsBot, MctsConfig};
use super::puzzle;

/// Where in the turn the decision was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Context {
    /// Own main phase, empty stack: the one window the search runs.
    OwnMain,
    /// Own attack declaration.
    Attack,
    /// Own block declaration.
    Block,
    /// Any other priority window on the seat's own turn (combat steps,
    /// end step, a main phase with something on the stack).
    OwnOther,
    /// A priority window on the opponent's turn.
    TheirTurn,
}

impl Context {
    pub const ALL: [Context; 5] = [Context::OwnMain, Context::Attack, Context::Block, Context::OwnOther, Context::TheirTurn];

    pub fn name(self) -> &'static str {
        match self {
            Context::OwnMain => "own main",
            Context::Attack => "attack",
            Context::Block => "block",
            Context::OwnOther => "own other",
            Context::TheirTurn => "their turn",
        }
    }

    fn of(g: &GameState, seat: usize) -> Context {
        if g.step == TurnStep::DeclareAttackers && g.active_player_idx == seat && g.attacking().is_empty() {
            return Context::Attack;
        }
        if g.step == TurnStep::DeclareBlockers && g.active_player_idx != seat && !g.blockers_declared() {
            return Context::Block;
        }
        if g.active_player_idx != seat {
            return Context::TheirTurn;
        }
        if g.stack.is_empty() && matches!(g.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain) {
            Context::OwnMain
        } else {
            Context::OwnOther
        }
    }
}

/// What kind of action a move is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Class {
    Pass,
    Land,
    Cast,
    Ability,
    Attack,
    Block,
    Other,
}

impl Class {
    pub const ALL: [Class; 7] = [Class::Pass, Class::Land, Class::Cast, Class::Ability, Class::Attack, Class::Block, Class::Other];

    pub fn name(self) -> &'static str {
        match self {
            Class::Pass => "pass",
            Class::Land => "land",
            Class::Cast => "cast",
            Class::Ability => "ability",
            Class::Attack => "attack",
            Class::Block => "block",
            Class::Other => "other",
        }
    }

    pub fn of(a: &GameAction) -> Class {
        match a {
            GameAction::PassPriority => Class::Pass,
            GameAction::PlayLand(_) => Class::Land,
            GameAction::CastSpell { .. } | GameAction::CastFlashback { .. } | GameAction::CastPrepareSpell { .. } => Class::Cast,
            GameAction::ActivateAbility { .. } | GameAction::ActivateLoyaltyAbility { .. } => Class::Ability,
            // An empty declaration is the "pass" of a combat step.
            GameAction::DeclareAttackers(v) if v.is_empty() => Class::Pass,
            GameAction::DeclareAttackers(_) => Class::Attack,
            GameAction::DeclareBlockers(v) if v.is_empty() => Class::Pass,
            GameAction::DeclareBlockers(_) => Class::Block,
            _ => Class::Other,
        }
    }
}

/// One sampled decision.
#[derive(Debug, Clone)]
pub struct Point {
    pub context: Context,
    pub turn: u32,
    pub legal: usize,
    /// Pass A's best is the pilot's play (up to the dedupe key).
    pub same: bool,
    pub chosen: GameAction,
    /// Pass A's best, priced by pass B.
    pub best: GameAction,
    pub chosen_value: f64,
    pub best_value: f64,
    /// For an own-main point: whether pass A's best was on the search's
    /// menu (its casts, the land drop, pass).
    pub best_on_menu: Option<bool>,
    /// A short description of the position for the worst-cases list.
    pub note: String,
}

impl Point {
    pub fn regret(&self) -> f64 {
        self.best_value - self.chosen_value
    }
}

/// Knobs for one audit.
#[derive(Debug, Clone, Copy)]
pub struct GapConfig {
    /// Rollouts per action per pass.
    pub probe: u32,
    /// Chance a decision point with two or more legal actions is sampled.
    pub sample: f64,
    /// Most actions probed at one point (the pilot's is always kept).
    pub max_actions: usize,
    /// Turn cap per game.
    pub max_turns: u32,
    /// Probe on the scored heuristic's rollouts (`heuristic_rollouts`)
    /// instead of the search's unscored ones. The unscored policy may never
    /// make a play the pilot would make a step later, so "pass now" against
    /// "cast it now" reads that weakness as the pilot's regret; this is the
    /// control for it, at a much higher price per rollout.
    pub scored_rollouts: bool,
}

impl Default for GapConfig {
    fn default() -> Self {
        Self { probe: 24, sample: 0.25, max_actions: 48, max_turns: 30, scored_rollouts: false }
    }
}

/// Play `template` with `pilot` on both seats and sample decision points.
pub fn audit_game(template: &GameState, pilot: MctsConfig, cfg: GapConfig, seed: u64) -> Vec<Point> {
    use rand::RngExt;
    let mut g = template.clone();
    let w = pilot.weights;
    for p in g.players.iter_mut() {
        p.smart_tap = w.smart_tap;
        p.converge_rarest = w.converge_rarest;
        p.hostile_player_targets = w.hostile_player_targets;
        p.duel_sinks = w.duel_sinks;
        p.tap_timing = w.tap_timing;
    }
    let mut rng = StdRng::seed_from_u64(seed);
    for seat in 0..2 {
        g.players[seat].library.shuffle(&mut rng);
    }
    g.rng.reseed(seed ^ 0x5EED_600D_C0DE_1234);
    g.start_mulligan_phase();
    let mut bots = [MctsBot::new(pilot), MctsBot::new(pilot)];
    let prober = MctsBot::new(MctsConfig { heuristic_rollouts: cfg.scored_rollouts, ..pilot });
    let mut out = Vec::new();
    let mut fuel = 6000u32;
    while fuel > 0 && !g.is_game_over() && g.turn_number <= cfg.max_turns {
        fuel -= 1;
        let mut acted = false;
        for seat in 0..2 {
            let sample = g.pending_decision.is_none() && rng.random::<f64>() < cfg.sample;
            let Some(a) = bots[seat].next_action(&g, seat) else { continue };
            if sample && let Some(p) = probe_point(&g, seat, &a, &prober, &w, cfg, &mut rng) {
                out.push(p);
            }
            if g.perform_action(a).is_ok() {
                acted = true;
                break;
            }
        }
        if !acted && g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn probe_point(
    g: &GameState,
    seat: usize,
    chosen: &GameAction,
    prober: &MctsBot,
    w: &super::bot::EvalWeights,
    cfg: GapConfig,
    rng: &mut StdRng,
) -> Option<Point> {
    // One action per distinct play: the enumerator tries every target on
    // every spell, and the engine accepts (and ignores) a target on one
    // that has none, so "cast X" came back once per permanent; two Swamps
    // are one land drop. Keyed by the canonical description.
    let mut keys: Vec<String> = Vec::new();
    let mut actions: Vec<GameAction> = Vec::new();
    for a in puzzle::legal_actions(g, seat).actions {
        let k = key(g, seat, &a);
        if !keys.contains(&k) {
            keys.push(k);
            actions.push(a);
        }
    }
    if actions.len() < 2 {
        return None;
    }
    let chosen_key = key(g, seat, chosen);
    let mut order: Vec<usize> = (0..actions.len()).collect();
    order.shuffle(rng);
    order.truncate(cfg.max_actions);
    let mut actions: Vec<GameAction> = order.iter().map(|&i| actions[i].clone()).collect();
    let mut keys: Vec<String> = order.iter().map(|&i| keys[i].clone()).collect();
    let ci = match keys.iter().position(|k| *k == chosen_key) {
        Some(i) => i,
        None => {
            actions.push(chosen.clone());
            keys.push(chosen_key.clone());
            actions.len() - 1
        }
    };
    let a = prober.probe_actions(g, seat, &actions, cfg.probe);
    let b = prober.probe_actions(g, seat, &actions, cfg.probe);
    let best = (0..actions.len()).filter(|&i| a[i].is_some()).max_by(|&i, &j| a[i].partial_cmp(&a[j]).unwrap_or(std::cmp::Ordering::Equal))?;
    let context = Context::of(g, seat);
    let best_on_menu = (context == Context::OwnMain).then(|| {
        let menu = super::bot::main_phase_candidates_for_mcts(g, seat, w);
        matches!(actions[best], GameAction::PassPriority | GameAction::PlayLand(_))
            || menu.iter().any(|(m, _)| key(g, seat, m) == keys[best])
    });
    Some(Point {
        context,
        turn: g.turn_number,
        legal: actions.len(),
        same: best == ci,
        chosen: chosen.clone(),
        best: actions[best].clone(),
        chosen_value: b[ci]?,
        best_value: b[best]?,
        best_on_menu,
        note: format!(
            "turn {} {:?}, life {}-{}, chose {} over {}",
            g.turn_number,
            g.step,
            g.players[seat].life,
            g.players[1 - seat].life,
            describe(g, chosen),
            describe(g, &actions[best]),
        ),
    })
}

/// The dedupe key: the description, with the target dropped from a spell
/// or ability that has none to take.
fn key(g: &GameState, seat: usize, a: &GameAction) -> String {
    let untargeted = |e: &crate::effect::Effect| !e.requires_target();
    let mut a = a.clone();
    match &mut a {
        GameAction::CastSpell { card_id, target, .. } => {
            if g.players[seat].hand.iter().find(|c| c.id == *card_id).is_some_and(|c| untargeted(&c.definition.effect)) {
                *target = None;
            }
        }
        GameAction::CastFlashback { card_id, target, .. } => {
            if g.players[seat].graveyard.iter().find(|c| c.id == *card_id).is_some_and(|c| untargeted(&c.definition.effect)) {
                *target = None;
            }
        }
        GameAction::ActivateAbility { card_id, ability_index, target, .. } => {
            if g.battlefield_find(*card_id)
                .and_then(|c| c.definition.activated_abilities.get(*ability_index))
                .is_some_and(|ab| untargeted(&ab.effect))
            {
                *target = None;
            }
        }
        _ => {}
    }
    describe(g, &a)
}

/// An action with card names in place of ids.
pub fn describe(g: &GameState, a: &GameAction) -> String {
    let name = |id: crate::card::CardId| {
        g.battlefield_find(id)
            .map(|c| c.definition.name)
            .or_else(|| g.players.iter().flat_map(|p| p.hand.iter().chain(p.graveyard.iter())).find(|c| c.id == id).map(|c| c.definition.name))
            .unwrap_or("?")
    };
    let target = |t: &Option<crate::game::Target>| match t {
        Some(crate::game::Target::Permanent(id)) => format!(" → {}", name(*id)),
        Some(crate::game::Target::Player(p)) => format!(" → player {p}"),
        None => String::new(),
    };
    match a {
        GameAction::PassPriority => "pass".into(),
        GameAction::PlayLand(id) => format!("land {}", name(*id)),
        GameAction::CastSpell { card_id, target: t, .. } => format!("cast {}{}", name(*card_id), target(t)),
        GameAction::ActivateAbility { card_id, ability_index, target: t, .. } => {
            format!("activate {}#{ability_index}{}", name(*card_id), target(t))
        }
        GameAction::ActivateLoyaltyAbility { card_id, ability_index, target: t, x_value } => {
            let x = x_value.map(|x| format!(" X={x}")).unwrap_or_default();
            format!("loyalty {}#{ability_index}{x}{}", name(*card_id), target(t))
        }
        GameAction::CastFlashback { card_id, target: t, .. } => format!("flashback {}{}", name(*card_id), target(t)),
        GameAction::DeclareAttackers(v) if v.is_empty() => "no attack".into(),
        GameAction::DeclareAttackers(v) => {
            format!("attack with {}", v.iter().map(|x| name(x.attacker)).collect::<Vec<_>>().join("+"))
        }
        GameAction::DeclareBlockers(v) if v.is_empty() => "no block".into(),
        GameAction::DeclareBlockers(v) => {
            format!("block {}", v.iter().map(|(b, a)| format!("{}→{}", name(*b), name(*a))).collect::<Vec<_>>().join(", "))
        }
        other => format!("{other:?}").chars().take(80).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::bot::EvalWeights;

    /// The probe's positive control, on a blunder position whose answer is
    /// an outcome: at 2 life behind two walls, Murder on the 2/2 flier lives
    /// and Murder on the walled 3/3 dies to the flier. The probe has to
    /// price the first clearly above the second — or a gap it reports is a
    /// reading of its own noise.
    #[test]
    fn the_probe_prices_the_flier_kill_above_the_ogre_kill() {
        let item = super::super::blunder::items().into_iter().find(|i| i.id == "kill_the_flier").expect("item");
        let g = (item.build)().state;
        let murder = g.players[0].hand.iter().find(|c| c.definition.name == "Murder").expect("Murder").id;
        let by_name = |n: &str| g.battlefield.iter().find(|c| c.definition.name == n).expect(n).id;
        let cast = |t| GameAction::CastSpell {
            card_id: murder,
            target: Some(crate::game::Target::Permanent(t)),
            additional_targets: Vec::new(),
            x_value: None,
            mode: None,
        };
        let probe = MctsBot::new(MctsConfig { horizon_turns: 3, weights: EvalWeights::default(), ..MctsConfig::default() });
        let v = probe.probe_actions(&g, 0, &[cast(by_name("Drake")), cast(by_name("Ogre"))], 8);
        let (drake, ogre) = (v[0].expect("legal"), v[1].expect("legal"));
        assert!(drake > ogre + 0.2, "flier {drake:.3} vs ogre {ogre:.3}");
    }
}
