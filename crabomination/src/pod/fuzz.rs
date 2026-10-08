//! `CRAB_POD_FUZZ=<n>` (pods) / `CRAB_LADDER_FUZZ=<n>` (two-player ladder
//! games): each seat's bot is wrapped so that `n` in 10,000 of its steps are
//! swapped for a random alternative — another of its own cast / activation
//! candidates at whatever priority window it holds, the other answer to a
//! yes/no ask, a random answer to an ask that lists its options (targets,
//! modes, cards, names, types, lessons, scry piles, divisions, trigger order,
//! mulligans), a random subset of its attack or block declaration. A sweep
//! tool: the tuned bots build the same boards over and over, and a panic or
//! invariant that only an odd line of play reaches is still a panic
//! self-play can hit. Seeded per game, so a fuzzed game replays. A swap the
//! engine rejects is simply not taken.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

use crate::decision::{Decision, DecisionAnswer};
use crate::game::{GameAction, GameState};
use crate::server::bot::{Bot, BotStep, EvalWeights};

fn rate_of(var: &str) -> Option<u32> {
    std::env::var(var).ok().and_then(|s| s.parse().ok()).filter(|&n| n > 0)
}

/// The pod fuzz rate in 10,000ths, read once (`None`: off).
pub(crate) fn fuzz_rate() -> Option<u32> {
    static RATE: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *RATE.get_or_init(|| rate_of("CRAB_POD_FUZZ"))
}

/// The two-player ladder's fuzz rate, read once.
pub(crate) fn ladder_fuzz_rate() -> Option<u32> {
    static RATE: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *RATE.get_or_init(|| rate_of("CRAB_LADDER_FUZZ"))
}

pub(crate) struct FuzzBot {
    inner: Box<dyn Bot>,
    rng: StdRng,
    rate: u32,
}

impl FuzzBot {
    pub(crate) fn wrap(inner: Box<dyn Bot>, rate: u32, seed: u64) -> Box<dyn Bot> {
        Box::new(Self { inner, rng: StdRng::seed_from_u64(seed), rate })
    }

    /// Keep each entry with even odds.
    fn subset<T: Clone>(&mut self, v: &[T]) -> Vec<T> {
        v.iter().filter(|_| self.rng.random_range(0..2u8) == 0).cloned().collect()
    }

    /// A random answer to the ask `seat` owes, where the ask lists its options.
    fn answer(&mut self, d: &Decision) -> Option<DecisionAnswer> {
        let pick = |rng: &mut StdRng, n: usize| (n > 0).then(|| rng.random_range(0..n));
        Some(match d {
            Decision::ChooseTarget { legal, optional, .. } => match pick(&mut self.rng, legal.len() + usize::from(*optional)) {
                Some(i) if i < legal.len() => DecisionAnswer::Target(legal[i].clone()),
                _ if *optional => DecisionAnswer::DeclineTarget,
                _ => return None,
            },
            Decision::ChooseMode { num_modes, .. } => DecisionAnswer::Mode(pick(&mut self.rng, *num_modes)?),
            Decision::ChooseOption { options, .. } => DecisionAnswer::Amount(pick(&mut self.rng, options.len())? as u32),
            Decision::ChooseColor { legal, .. } => DecisionAnswer::Color(legal[pick(&mut self.rng, legal.len())?]),
            Decision::ChooseModes { num_modes, count, .. } => {
                let mut modes: Vec<u8> = (0..*num_modes as u8).collect();
                modes.shuffle(&mut self.rng);
                modes.truncate(*count);
                DecisionAnswer::Modes(modes)
            }
            Decision::Discard { count, hand, .. } => {
                let mut ids: Vec<_> = hand.iter().map(|(id, _)| *id).collect();
                ids.shuffle(&mut self.rng);
                ids.truncate(*count as usize);
                DecisionAnswer::Discard(ids)
            }
            Decision::SearchLibrary { candidates, eligible, .. } => {
                let pool: Vec<_> = eligible.clone().unwrap_or_else(|| candidates.iter().map(|(id, _)| *id).collect());
                DecisionAnswer::Search(pick(&mut self.rng, pool.len() + 1).and_then(|i| pool.get(i).copied()))
            }
            Decision::ChooseCards { candidates, min, max, eligible, .. } => {
                let mut pool: Vec<_> = eligible.clone().unwrap_or_else(|| candidates.iter().map(|(id, _)| *id).collect());
                pool.shuffle(&mut self.rng);
                let hi = (*max as usize).min(pool.len());
                let lo = (*min as usize).min(hi);
                pool.truncate(self.rng.random_range(lo..=hi));
                DecisionAnswer::Cards(pool)
            }
            Decision::ChooseAmount { max, .. } => DecisionAnswer::Amount(self.rng.random_range(0..=*max)),
            Decision::OrderTriggers { triggers, .. } => {
                let mut ids: Vec<_> = triggers.iter().map(|(id, _)| *id).collect();
                ids.shuffle(&mut self.rng);
                DecisionAnswer::TriggerOrder(ids)
            }
            Decision::CombatDamageOrder { blockers, .. } => {
                let mut ids: Vec<_> = blockers.iter().map(|(id, _)| *id).collect();
                ids.shuffle(&mut self.rng);
                DecisionAnswer::DamageOrder(ids)
            }
            Decision::Scry { cards, .. } => {
                let mut ids: Vec<_> = cards.iter().map(|(id, _)| *id).collect();
                ids.shuffle(&mut self.rng);
                let cut = self.rng.random_range(0..=ids.len());
                let bottom = ids.split_off(cut);
                DecisionAnswer::ScryOrder { kept_top: ids, bottom }
            }
            Decision::PutOnLibrary { count, hand, .. } => {
                let mut ids: Vec<_> = hand.iter().map(|(id, _)| *id).collect();
                ids.shuffle(&mut self.rng);
                ids.truncate(*count);
                DecisionAnswer::PutOnLibrary(ids)
            }
            Decision::Mulligan { .. } => {
                if self.rng.random_range(0..2u8) == 0 { DecisionAnswer::Keep } else { DecisionAnswer::TakeMulligan }
            }
            // At least one to each target (CR 601.2d), the rest at random.
            Decision::DivideDamage { total, targets, .. } if *total as usize >= targets.len() && !targets.is_empty() => {
                let mut split = vec![1u32; targets.len()];
                for _ in 0..(*total as usize - targets.len()) {
                    let i = self.rng.random_range(0..split.len());
                    split[i] += 1;
                }
                DecisionAnswer::DamageDivision(split)
            }
            Decision::AssignCombatDamage { attacker_power, blockers, .. } if !blockers.is_empty() => {
                let mut split: Vec<(crate::card::CardId, u32)> = blockers.iter().map(|(id, _, _)| (*id, 0)).collect();
                for _ in 0..*attacker_power {
                    let i = self.rng.random_range(0..split.len());
                    split[i].1 += 1;
                }
                DecisionAnswer::CombatDamageAssignment(split)
            }
            Decision::ChooseCreatureType { suggestions, .. } => {
                DecisionAnswer::CreatureType(suggestions[pick(&mut self.rng, suggestions.len())?])
            }
            Decision::NameCard { suggestions, .. } => {
                DecisionAnswer::NamedCard(suggestions[pick(&mut self.rng, suggestions.len())?].clone())
            }
            Decision::Learn { lessons, hand, .. } => {
                let i = self.rng.random_range(0..lessons.len() + hand.len() + 1);
                DecisionAnswer::Learn(match (lessons.get(i), hand.get(i.wrapping_sub(lessons.len()))) {
                    (Some((id, _)), _) => crate::decision::LearnChoice::FetchLesson(*id),
                    (None, Some((id, _))) => crate::decision::LearnChoice::Rummage { discard: *id },
                    _ => crate::decision::LearnChoice::Decline,
                })
            }
            Decision::ChooseLegendToKeep { duplicates, .. } => {
                DecisionAnswer::KeptLegend(duplicates[pick(&mut self.rng, duplicates.len())?].0)
            }
            _ => return None,
        })
    }

    fn swap(&mut self, state: &GameState, seat: usize, action: &GameAction) -> Option<GameAction> {
        match action {
            GameAction::SubmitDecision(DecisionAnswer::Bool(b)) => {
                Some(GameAction::SubmitDecision(DecisionAnswer::Bool(!b)))
            }
            GameAction::SubmitDecision(_) => {
                let d = &state.pending_decision.as_ref()?.decision;
                self.answer(d).map(GameAction::SubmitDecision)
            }
            GameAction::DeclareAttackers(v) if !v.is_empty() => Some(GameAction::DeclareAttackers(self.subset(v))),
            GameAction::DeclareBlockers(v) if !v.is_empty() => Some(GameAction::DeclareBlockers(self.subset(v))),
            // Any window this seat holds priority in — a response in combat
            // or at end of turn as much as a main-phase play. The candidates
            // ignore timing; the engine refuses what it must.
            _ if state.pending_decision.is_none() && state.player_with_priority() == seat => {
                let mut c = crate::server::bot::main_phase_candidates_for_mcts(state, seat, &EvalWeights::default());
                c.push((GameAction::PassPriority, 0));
                let i = self.rng.random_range(0..c.len());
                Some(c.swap_remove(i).0)
            }
            _ => None,
        }
    }
}

impl Bot for FuzzBot {
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        self.next_action_settled(state, seat).map(|s| s.action)
    }

    fn next_action_settled(&mut self, state: &GameState, seat: usize) -> Option<BotStep> {
        let step = self.inner.next_action_settled(state, seat)?;
        if self.rng.random_range(0..10_000u32) >= self.rate {
            return Some(step);
        }
        Some(match self.swap(state, seat, &step.action) {
            Some(action) => BotStep::plain(action),
            None => step,
        })
    }

    fn push_seat_flags(&self, player: &mut crate::player::Player) {
        self.inner.push_seat_flags(player);
    }

    fn rewound(&mut self) {
        self.inner.rewound();
    }
}
