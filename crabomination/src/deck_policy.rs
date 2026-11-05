//! The add-one-card deck builder — step 2 of the deck-building net (ML_NOTES
//! "A deck net that builds decks"), as `deck_policy_train` exports it.
//!
//! A sealed deck's spells are chosen one pool card at a time. The state is
//! the pool's mean card vector, the partial deck's, the partial deck's 16
//! [`encode_deck`](crate::server::encode::encode_deck) features and its size;
//! each available pool card is scored from its own vector, the state, its
//! copies already in the deck and its copies left; a separate score stops.
//! Stopping is masked below [`MIN_SPELLS`] and forced at [`MAX_SPELLS`]. The
//! spells then get the rated builder's own land step
//! ([`crate::recommend::lands_for_spells`]).

use std::collections::HashMap;

use rand::RngExt;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::cube::CardFactory;
use crate::deck_value::Layer;

/// The fewest spells a build may stop at; the rated builder plays 22–24.
pub const MIN_SPELLS: usize = 20;
/// The most spells a build takes before it must stop.
pub const MAX_SPELLS: usize = 25;
/// A deck is 40 cards.
pub const DECK_SIZE: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckPolicy {
    pub cards: Vec<String>,
    /// Each card's encoder vector.
    pub x: Vec<Vec<f32>>,
    /// `[pool mean, deck mean, deck features, size / 23]` → the state.
    pub state: Layer,
    /// `[card, state, copies in deck / 2, copies left / 2]` → hidden → score.
    pub cand1: Layer,
    pub cand2: Layer,
    /// state → hidden → the stop score.
    pub stop1: Layer,
    pub stop2: Layer,
    #[serde(skip)]
    index: HashMap<String, usize>,
}

/// One step's choices: each available card's score, and stop's (`None`
/// when stopping is not allowed yet).
#[derive(Debug, Clone)]
pub struct StepScores {
    pub cards: Vec<(CardFactory, f32)>,
    pub stop: Option<f32>,
}

impl DeckPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(cards: Vec<String>, x: Vec<Vec<f32>>, state: Layer, cand1: Layer, cand2: Layer, stop1: Layer, stop2: Layer) -> Self {
        let mut p = Self { cards, x, state, cand1, cand2, stop1, stop2, index: HashMap::new() };
        p.reindex();
        p
    }

    fn reindex(&mut self) {
        self.index = self.cards.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
    }

    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut p: Self = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        p.reindex();
        Ok(p)
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        std::fs::write(path, serde_json::to_string(self).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", path.display()))
    }

    fn known(&self, f: CardFactory) -> Option<usize> {
        self.index.get(crate::cube::card_def(f).name).copied()
    }

    /// The scores for adding each pool card to `deck` (spells so far), and
    /// for stopping. Pool lands, and cards the policy never trained on, are
    /// not offered.
    pub fn step(&self, pool: &[CardFactory], deck: &[CardFactory]) -> StepScores {
        let h = self.x.first().map_or(0, |v| v.len());
        // Distinct non-land known pool cards, first-seen order, with copies.
        let mut avail: Vec<(CardFactory, usize, f32)> = Vec::new();
        let mut pool_mean = vec![0f32; h];
        let mut pool_n = 0f32;
        for &f in pool {
            if crate::cube::card_def(f).is_land() {
                continue;
            }
            let Some(i) = self.known(f) else { continue };
            for (m, v) in pool_mean.iter_mut().zip(&self.x[i]) {
                *m += v;
            }
            pool_n += 1.0;
            match avail.iter_mut().find(|a| a.1 == i) {
                Some(a) => a.2 += 1.0,
                None => avail.push((f, i, 1.0)),
            }
        }
        pool_mean.iter_mut().for_each(|m| *m /= pool_n.max(1.0));
        let mut deck_mean = vec![0f32; h];
        let mut in_deck: HashMap<usize, f32> = HashMap::new();
        for &f in deck {
            if let Some(i) = self.known(f) {
                for (m, v) in deck_mean.iter_mut().zip(&self.x[i]) {
                    *m += v;
                }
                *in_deck.entry(i).or_insert(0.0) += 1.0;
            }
        }
        deck_mean.iter_mut().for_each(|m| *m /= (deck.len() as f32).max(1.0));
        let (_, feats) = crate::server::encode::encode_deck(deck, crate::server::net_eval::vocab());
        let mut sin = pool_mean;
        sin.extend_from_slice(&deck_mean);
        sin.extend_from_slice(&feats);
        sin.push(deck.len() as f32 / 23.0);
        let u = self.state.fwd(&sin, true);
        let mut cards = Vec::new();
        for (f, i, copies) in avail {
            let used = in_deck.get(&i).copied().unwrap_or(0.0);
            let left = copies - used;
            if left <= 0.0 {
                continue;
            }
            let mut cin = self.x[i].clone();
            cin.extend_from_slice(&u);
            cin.push(used / 2.0);
            cin.push(left / 2.0);
            cards.push((f, self.cand2.fwd(&self.cand1.fwd(&cin, true), false)[0]));
        }
        let stop = (deck.len() >= MIN_SPELLS || cards.is_empty()).then(|| self.stop2.fwd(&self.stop1.fwd(&u, true), false)[0]);
        StepScores { cards, stop }
    }

    /// The spells of one build: greedy with `sample = None`, else each step
    /// drawn from the softmax at the given temperature.
    pub fn spells(&self, pool: &[CardFactory], mut sample: Option<(&mut StdRng, f32)>) -> Vec<CardFactory> {
        let mut deck: Vec<CardFactory> = Vec::new();
        while deck.len() < MAX_SPELLS {
            let s = self.step(pool, &deck);
            if s.cards.is_empty() {
                break;
            }
            // Choice `n` is stop.
            let n = s.cards.len();
            let scores: Vec<f32> = s.cards.iter().map(|c| c.1).chain(s.stop).collect();
            let pick = match sample.as_mut() {
                None => scores.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(n, |(i, _)| i),
                Some((rng, t)) => {
                    let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                    let w: Vec<f64> = scores.iter().map(|&x| (((x - max) / t.max(1e-3)) as f64).exp()).collect();
                    let mut r = rng.random_range(0.0..w.iter().sum::<f64>());
                    let mut at = w.len() - 1;
                    for (i, wi) in w.iter().enumerate() {
                        if r < *wi {
                            at = i;
                            break;
                        }
                        r -= wi;
                    }
                    at
                }
            };
            if pick >= n {
                break;
            }
            deck.push(s.cards[pick].0);
        }
        deck
    }

    /// A 40-card deck: [`spells`](Self::spells), then the rated builder's
    /// lands for them.
    pub fn build(&self, pool: &[CardFactory], sample: Option<(&mut StdRng, f32)>) -> Vec<CardFactory> {
        let mut deck = self.spells(pool, sample);
        let lands = crate::recommend::lands_for_spells(pool, &deck, DECK_SIZE.saturating_sub(deck.len()) as u32);
        deck.extend(lands);
        deck
    }

    /// G2b: the greedy build and `samples` sampled ones (temperature 1),
    /// the one `value` rates highest.
    pub fn build_valued(&self, pool: &[CardFactory], value: &crate::deck_value::DeckValue, samples: usize, rng: &mut StdRng) -> Vec<CardFactory> {
        let mut best = self.build(pool, None);
        let mut best_v = value.value(&best);
        for _ in 0..samples {
            let d = self.build(pool, Some((&mut *rng, 1.0)));
            let v = value.value(&d);
            if v > best_v {
                best = d;
                best_v = v;
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zeros(o: usize, i: usize) -> Layer {
        Layer { w: vec![vec![0.0; i]; o], b: vec![0.0; o] }
    }

    /// The stop window and the land step, on a policy with no opinion:
    /// every score ties, so greedy stops the moment it may — at
    /// [`MIN_SPELLS`] — and the rated land step fills the deck to 40.
    #[test]
    fn a_build_stops_inside_its_window_and_fills_forty() {
        let pool = crate::selfplay::sealed_pool_of(crate::fra_mode::SealedSet::Sos, 3, crate::selfplay::SEALED_PACKS);
        let mut names: Vec<String> = pool.iter().map(|f| crate::cube::card_def(*f).name.to_string()).collect();
        names.sort();
        names.dedup();
        let h = 4;
        let x = vec![vec![0.0; h]; names.len()];
        let p = DeckPolicy::new(names, x, zeros(h, 2 * h + 17), zeros(h, 2 * h + 2), zeros(1, h), zeros(2, h), zeros(1, 2));
        let deck = p.build(&pool, None);
        let spells = deck.iter().filter(|f| !crate::cube::card_def(**f).is_land()).count();
        assert_eq!(spells, MIN_SPELLS, "ties go to stop once it is allowed");
        assert_eq!(deck.len(), DECK_SIZE);
        let mut rng = <StdRng as rand::SeedableRng>::seed_from_u64(5);
        let sampled = p.spells(&pool, Some((&mut rng, 1.0)));
        assert!((MIN_SPELLS..=MAX_SPELLS).contains(&sampled.len()), "a sampled build stops inside the window: {}", sampled.len());
    }
}
