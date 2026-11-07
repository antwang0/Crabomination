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
use crate::mana::{Color, ColorSet};

/// The fewest spells a build may stop at; the rated builder plays 22–24.
pub const MIN_SPELLS: usize = 20;
/// The most spells a build takes before it must stop.
pub const MAX_SPELLS: usize = 25;
/// A deck is 40 cards.
pub const DECK_SIZE: usize = 40;
/// The fewest lands that make a splash colour.
pub const SPLASH_SOURCES: usize = 3;

/// Can a deck in `want` cast `f` ({G/U} needs G or U, not both)?
fn fits(f: CardFactory, want: ColorSet) -> bool {
    crate::cube::card_brief(f).fits(want)
}

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
    /// `cand1`'s card block applied to each card's vector, plus its bias:
    /// the part of a candidate's hidden layer that does not depend on the
    /// state, computed once (a 1024-wide policy would otherwise multiply
    /// `[H x 2H]` per candidate per step).
    #[serde(skip)]
    card_part: Vec<Vec<f32>>,
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
        let mut p = Self { cards, x, state, cand1, cand2, stop1, stop2, index: HashMap::new(), card_part: Vec::new() };
        p.reindex();
        p
    }

    fn reindex(&mut self) {
        self.index = self.cards.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
        let h = self.x.first().map_or(0, |v| v.len());
        self.card_part = self
            .x
            .iter()
            .map(|xi| self.cand1.w.iter().zip(&self.cand1.b).map(|(row, b)| row[..h].iter().zip(xi).map(|(w, x)| w * x).sum::<f32>() + b).collect())
            .collect();
    }

    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// A policy from its exported JSON (a compiled-in net, or a file's text).
    pub fn from_json(text: &str) -> Result<Self, String> {
        let mut p: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        p.reindex();
        Ok(p)
    }

    /// Does the policy know every nonland card in `pool`? A card it never
    /// trained on is not offered by [`step`](Self::step), so a pool outside
    /// its sheets would be built from whatever it recognises; the sealed
    /// opponent falls back to the rated build instead.
    pub fn covers(&self, pool: &[CardFactory]) -> bool {
        let mut spells = 0usize;
        for &f in pool {
            if crate::cube::card_def(f).is_land() {
                continue;
            }
            spells += 1;
            if self.known(f).is_none() {
                return false;
            }
        }
        spells > 0
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
        // `cand1`'s state block, once per step; the card block is cached.
        let state_part: Vec<f32> = self.cand1.w.iter().map(|row| row[h..2 * h].iter().zip(&u).map(|(w, x)| w * x).sum()).collect();
        let mut cards = Vec::new();
        for (f, i, copies) in avail {
            let used = in_deck.get(&i).copied().unwrap_or(0.0);
            let left = copies - used;
            if left <= 0.0 {
                continue;
            }
            let hid: Vec<f32> = self
                .cand1
                .w
                .iter()
                .zip(&self.card_part[i])
                .zip(&state_part)
                .map(|((row, c), s)| (c + s + row[2 * h] * (used / 2.0) + row[2 * h + 1] * (left / 2.0)).max(0.0))
                .collect();
            cards.push((f, self.cand2.fwd(&hid, false)[0]));
        }
        let stop = (deck.len() >= MIN_SPELLS || cards.is_empty()).then(|| self.stop2.fwd(&self.stop1.fwd(&u, true), false)[0]);
        StepScores { cards, stop }
    }

    /// The spells of one build: greedy with `sample = None`, else each step
    /// drawn from the softmax at the given temperature.
    pub fn spells(&self, pool: &[CardFactory], sample: Option<(&mut StdRng, f32)>) -> Vec<CardFactory> {
        let mut deck = Vec::new();
        self.extend(pool, &mut deck, sample, None);
        deck
    }

    /// Continue `deck` until the policy stops (or [`MAX_SPELLS`]), offered
    /// only cards whose coloured pips sit inside `allowed` when given.
    fn extend(&self, pool: &[CardFactory], deck: &mut Vec<CardFactory>, mut sample: Option<(&mut StdRng, f32)>, allowed: Option<ColorSet>) {
        while deck.len() < MAX_SPELLS {
            let mut s = self.step(pool, deck);
            if let Some(a) = allowed {
                s.cards.retain(|c| fits(c.0, a));
            }
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
    }

    /// The spells of one build under the splash rule the rated builder's
    /// `splash_cards` keeps: the two colours with the most pips are the
    /// main colours; cards outside them stay only as **one** splash colour,
    /// at most three cards, each with a single pip of it. Otherwise the
    /// off-colour cards go and the policy keeps choosing among main-colour
    /// and colourless cards. Returns the spells, the main colours and the
    /// splash colour.
    pub fn castable_spells(&self, pool: &[CardFactory], mut sample: Option<(&mut StdRng, f32)>) -> (Vec<CardFactory>, Vec<Color>, Option<Color>) {
        let mut deck = self.spells(pool, sample.as_mut().map(|(r, t)| (&mut **r, *t)));
        let mut weight: Vec<(Color, u32)> =
            Color::ALL.into_iter().map(|c| (c, deck.iter().map(|f| crate::cube::card_brief(*f).pips.get(c)).sum())).filter(|w| w.1 > 0).collect();
        weight.sort_by_key(|w| std::cmp::Reverse(w.1));
        let main: Vec<Color> = weight.iter().take(2).map(|w| w.0).collect();
        let main_set = main.iter().fold(ColorSet::empty(), |mut s, &c| {
            s.insert(c);
            s
        });
        let off: Vec<CardFactory> = deck.iter().copied().filter(|f| !fits(*f, main_set)).collect();
        if off.is_empty() {
            return (deck, main, None);
        }
        // The splash colour: the one off colour every off card fits with.
        let splash = Color::ALL.into_iter().filter(|c| !main_set.contains(c)).find(|&c| {
            let mut with = main_set;
            with.insert(c);
            off.iter().all(|f| fits(*f, with))
        });
        if let Some(c) = splash
            && off.len() <= 3
            && off.iter().all(|f| crate::cube::card_brief(*f).off_pips(main_set) == 1)
        {
            return (deck, main, Some(c));
        }
        deck.retain(|f| fits(*f, main_set));
        self.extend(pool, &mut deck, sample, Some(main_set));
        (deck, main, None)
    }

    /// A 40-card deck: [`castable_spells`](Self::castable_spells), the rated
    /// builder's lands for them, and a splash's three sources
    /// ([`crate::recommend::ensure_sources`]).
    pub fn build(&self, pool: &[CardFactory], sample: Option<(&mut StdRng, f32)>) -> Vec<CardFactory> {
        let (mut deck, main, splash) = self.castable_spells(pool, sample);
        let mut lands = crate::recommend::lands_for_spells(pool, &deck, DECK_SIZE.saturating_sub(deck.len()) as u32);
        if let Some(c) = splash {
            crate::recommend::ensure_sources(&mut lands, c, SPLASH_SOURCES, &main);
        }
        deck.extend(lands);
        deck
    }

    /// The greedy build held to the two colours `pair` (and colourless
    /// cards), with the rated land step — a race candidate per colour pair
    /// (ML_NOTES step 3c). `None` when the pool has too few such cards to
    /// reach [`MIN_SPELLS`].
    pub fn build_in(&self, pool: &[CardFactory], pair: [Color; 2]) -> Option<Vec<CardFactory>> {
        let mut want = ColorSet::empty();
        want.insert(pair[0]);
        want.insert(pair[1]);
        let mut deck = Vec::new();
        self.extend(pool, &mut deck, None, Some(want));
        if deck.len() < MIN_SPELLS {
            return None;
        }
        let lands = crate::recommend::lands_for_spells(pool, &deck, DECK_SIZE.saturating_sub(deck.len()) as u32);
        deck.extend(lands);
        Some(deck)
    }

    /// The step-2 build, before the splash rule: [`spells`](Self::spells)
    /// and the land step alone — the control the rule was gated against.
    pub fn build_unchecked(&self, pool: &[CardFactory], sample: Option<(&mut StdRng, f32)>) -> Vec<CardFactory> {
        let mut deck = self.spells(pool, sample);
        let lands = crate::recommend::lands_for_spells(pool, &deck, DECK_SIZE.saturating_sub(deck.len()) as u32);
        deck.extend(lands);
        deck
    }

    /// G2b: the greedy build, `samples` sampled ones (temperature 1) and,
    /// since step 5, the greedy build held to each colour pair
    /// ([`build_in`](Self::build_in)) — the one `value` rates highest.
    pub fn build_valued(&self, pool: &[CardFactory], value: &crate::deck_value::DeckValue, samples: usize, rng: &mut StdRng) -> Vec<CardFactory> {
        let mut best = self.build(pool, None);
        let mut best_v = value.value(&best);
        let mut consider = |d: Vec<CardFactory>| {
            let v = value.value(&d);
            if v > best_v {
                best = d;
                best_v = v;
            }
        };
        for _ in 0..samples {
            consider(self.build(pool, Some((&mut *rng, 1.0))));
        }
        for (a, &c1) in Color::ALL.iter().enumerate() {
            for &c2 in &Color::ALL[a + 1..] {
                if let Some(d) = self.build_in(pool, [c1, c2]) {
                    consider(d);
                }
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

    /// The user's pool built a W/G deck splashing two {B} spells off one
    /// Swamp (ML_NOTES step 3, the user-pool case study): a splash's basics
    /// come out of the heaviest main colour until it has three sources.
    #[test]
    fn a_splash_gets_three_sources_from_the_heaviest_main_colour() {
        use crate::catalog::{forest, plains, swamp};
        let mut lands: Vec<CardFactory> = [vec![plains as CardFactory; 9], vec![forest; 6], vec![swamp]].concat();
        crate::recommend::ensure_sources(&mut lands, Color::Black, SPLASH_SOURCES, &[Color::White, Color::Green]);
        let n = |b: CardFactory| lands.iter().filter(|f| **f as usize == b as usize).count();
        assert_eq!((n(plains), n(forest), n(swamp)), (7, 6, 3));
    }

    /// `step` splits `cand1` into a cached card block and a per-step state
    /// block; on random weights it must score every candidate as the full
    /// `[card, state, used, left]` product does.
    #[test]
    fn the_split_candidate_layer_matches_the_full_product() {
        use rand::SeedableRng;
        let pool = crate::selfplay::sealed_pool_of(crate::fra_mode::SealedSet::Sos, 3, crate::selfplay::SEALED_PACKS);
        let mut names: Vec<String> = pool.iter().map(|f| crate::cube::card_def(*f).name.to_string()).collect();
        names.sort();
        names.dedup();
        let h = 6;
        let mut rng = StdRng::seed_from_u64(9);
        let mut rand_layer = |o: usize, i: usize| Layer {
            w: (0..o).map(|_| (0..i).map(|_| rng.random_range(-1.0f32..1.0)).collect()).collect(),
            b: (0..o).map(|_| rng.random_range(-0.5f32..0.5)).collect(),
        };
        let (state, cand1, cand2, stop1, stop2) = (rand_layer(h, 2 * h + 17), rand_layer(h, 2 * h + 2), rand_layer(1, h), rand_layer(2, h), rand_layer(1, 2));
        let x: Vec<Vec<f32>> = (0..names.len()).map(|_| (0..h).map(|_| rng.random_range(0.0f32..1.0)).collect()).collect();
        let p = DeckPolicy::new(names, x, state, cand1, cand2, stop1, stop2);
        let deck: Vec<CardFactory> = pool.iter().copied().filter(|f| !crate::cube::card_def(*f).is_land()).take(5).collect();
        let s = p.step(&pool, &deck);
        assert!(!s.cards.is_empty());
        // The full product, recomputed from the step's own state vector.
        let mut sin = vec![0f32; 2 * h];
        let known: Vec<CardFactory> = pool.iter().copied().filter(|f| !crate::cube::card_def(*f).is_land() && p.known(*f).is_some()).collect();
        for f in &known {
            for (m, v) in sin[..h].iter_mut().zip(&p.x[p.known(*f).unwrap()]) {
                *m += v / known.len() as f32;
            }
        }
        for f in &deck {
            for (m, v) in sin[h..].iter_mut().zip(&p.x[p.known(*f).unwrap()]) {
                *m += v / deck.len() as f32;
            }
        }
        sin.extend_from_slice(&crate::server::encode::encode_deck(&deck, crate::server::net_eval::vocab()).1);
        sin.push(deck.len() as f32 / 23.0);
        let u = p.state.fwd(&sin, true);
        for (f, score) in &s.cards {
            let i = p.known(*f).unwrap();
            let copies = known.iter().filter(|g| p.known(**g) == Some(i)).count() as f32;
            let used = deck.iter().filter(|g| p.known(**g) == Some(i)).count() as f32;
            let mut cin = p.x[i].clone();
            cin.extend_from_slice(&u);
            cin.push(used / 2.0);
            cin.push((copies - used) / 2.0);
            let full = p.cand2.fwd(&p.cand1.fwd(&cin, true), false)[0];
            assert!((full - score).abs() < 1e-4, "{}: split {score} vs full {full}", crate::cube::card_def(*f).name);
        }
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
        // A pair build stays in its pair, or declines.
        let mut built = 0;
        for pair in [[Color::White, Color::Blue], [Color::Black, Color::Green], [Color::Red, Color::Green]] {
            let mut want = ColorSet::empty();
            want.insert(pair[0]);
            want.insert(pair[1]);
            if let Some(d) = p.build_in(&pool, pair) {
                built += 1;
                assert_eq!(d.len(), DECK_SIZE);
                assert!(d.iter().filter(|f| !crate::cube::card_def(**f).is_land()).all(|f| fits(*f, want)), "{pair:?} build leaves its pair");
            }
        }
        assert!(built > 0, "a 6-pack pool supports some colour pair");
    }
}
