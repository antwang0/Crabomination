//! A learned sealed deck value — step 1 of the deck-building net (ML_NOTES
//! "A deck net that builds decks"), as `deck_value_train` exports it.
//!
//! `f(deck) = Σ_c n_c (a_c + b_c · c(ctx)) + h(ctx)`: per card a value `a_c`
//! and an interaction vector `b_c`, both fixed per card name once trained;
//! `ctx` the deck's mean card vector `x_c` ⊕ its 16 `encode_deck` features,
//! read by the context map `c` and the deck term `h` (one hidden ReLU layer
//! each). The additive arms export `a` alone (and `A0f` its deck-feature
//! MLP). Units are logits of one game: deck A beats deck B with
//! `σ(f(A) − f(B))`.
//!
//! [`crate::selfplay::deck_value_build`] polishes the rated builder's pick
//! under it, a few swaps from the builder's own deck — the trust region
//! round 15 measured the need for.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cube::CardFactory;

/// One dense layer, `w` row-major `[out][in]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub w: Vec<Vec<f32>>,
    pub b: Vec<f32>,
}

impl Layer {
    pub(crate) fn fwd(&self, x: &[f32], relu: bool) -> Vec<f32> {
        self.w
            .iter()
            .zip(&self.b)
            .map(|(row, b)| {
                let v = row.iter().zip(x).map(|(w, x)| w * x).sum::<f32>() + b;
                if relu { v.max(0.0) } else { v }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckValue {
    /// The arm that trained it (`A0` .. `A4`), for the record.
    pub arm: String,
    pub cards: Vec<String>,
    pub a: Vec<f32>,
    /// Per card: its encoder vector and interaction vector (empty for an
    /// additive arm).
    pub x: Vec<Vec<f32>>,
    pub b: Vec<Vec<f32>>,
    /// The context map `c` and the deck term `h`, two layers each.
    pub ctx: Option<[Layer; 4]>,
    /// `A0f`'s MLP over the deck features.
    pub deck_mlp: Option<[Layer; 2]>,
    /// Whether the deck features carry the [`MANA_FEATS`] mana-base summary
    /// ([`mana_feats`]) after `encode_deck`'s 16 (`deck_value_train --mana`).
    #[serde(default)]
    pub mana: bool,
    #[serde(skip)]
    index: HashMap<String, usize>,
}

impl DeckValue {
    pub fn new(arm: &str, cards: Vec<String>, a: Vec<f32>) -> Self {
        let mut d = Self { arm: arm.to_string(), cards, a, x: Vec::new(), b: Vec::new(), ctx: None, deck_mlp: None, mana: false, index: HashMap::new() };
        d.reindex();
        d
    }

    fn reindex(&mut self) {
        self.index = self.cards.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
    }

    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// A value from its exported JSON (a compiled-in table, or a file's text).
    pub fn from_json(text: &str) -> Result<Self, String> {
        let mut d: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        d.reindex();
        Ok(d)
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        std::fs::write(path, serde_json::to_string(self).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Whether the value was trained with `name` in its decks — a card it
    /// has no row for cannot be priced, so a polish never swaps one in.
    pub fn knows(&self, name: &str) -> bool {
        self.index.contains_key(name)
    }

    /// The deck's value, in game logits. Cards it does not know count for
    /// nothing. Summed in card order: a `HashMap`'s per-instance order made
    /// two calls on one deck round differently, and a polish broke its ties
    /// by that rounding — the same pool built two different decks.
    pub fn value(&self, deck: &[CardFactory]) -> f64 {
        let mut counts: std::collections::BTreeMap<usize, f32> = std::collections::BTreeMap::new();
        for f in deck {
            if let Some(&i) = self.index.get(crate::cube::card_def(*f).name) {
                *counts.entry(i).or_insert(0.0) += 1.0;
            }
        }
        let mut f: f32 = counts.iter().map(|(&i, &n)| n * self.a[i]).sum();
        let need_feats = self.ctx.is_some() || self.deck_mlp.is_some();
        let feats = if need_feats {
            let mut v = crate::server::encode::encode_deck(deck, crate::server::net_eval::vocab()).1.to_vec();
            if self.mana {
                v.extend_from_slice(&mana_feats(deck));
            }
            v
        } else {
            Vec::new()
        };
        if let Some([c1, c2, h1, h2]) = &self.ctx {
            let n: f32 = counts.values().sum::<f32>().max(1.0);
            let h = self.x.first().map_or(0, |v| v.len());
            let mut ctx = vec![0f32; h];
            let mut mb = vec![0f32; self.b.first().map_or(0, |v| v.len())];
            for (&i, &k) in &counts {
                for (c, v) in ctx.iter_mut().zip(&self.x[i]) {
                    *c += k * v / n;
                }
                for (m, v) in mb.iter_mut().zip(&self.b[i]) {
                    *m += k * v;
                }
            }
            ctx.extend_from_slice(&feats);
            let c = c2.fwd(&c1.fwd(&ctx, true), false);
            f += mb.iter().zip(&c).map(|(m, c)| m * c).sum::<f32>();
            f += h2.fwd(&h1.fwd(&ctx, true), false)[0];
        }
        if let Some([l1, l2]) = &self.deck_mlp {
            f += l2.fwd(&l1.fwd(&feats, true), false)[0];
        }
        f as f64
    }
}

/// How many numbers [`mana_feats`] adds to a deck's features.
pub const MANA_FEATS: usize = 5;

/// `P(X >= k)` for `X ~ Hypergeometric(n_total, succ, draws)`.
fn hyper_at_least(n_total: usize, succ: usize, draws: usize, k: usize) -> f64 {
    if k == 0 {
        return 1.0;
    }
    let draws = draws.min(n_total);
    if succ < k || draws < k {
        return 0.0;
    }
    let ln_choose = |n: usize, r: usize| -> f64 {
        if r > n {
            return f64::NEG_INFINITY;
        }
        (1..=r).map(|i| ((n - r + i) as f64).ln() - (i as f64).ln()).sum()
    };
    let total = ln_choose(n_total, draws);
    let below: f64 = (0..k)
        .filter(|&x| x <= draws && draws - x <= n_total - succ)
        .map(|x| (ln_choose(succ, x) + ln_choose(n_total - succ, draws - x) - total).exp())
        .sum();
    (1.0 - below).clamp(0.0, 1.0)
}

/// The deck's mana base against its spells' colour demands — what the
/// 16 `encode_deck` features cannot see (they count lands and pips, never
/// which colours the lands make; ML_NOTES "the same-pool deck value cannot
/// see the mana base"):
///
/// 0. mean on-curve castability: per spell, the hypergeometric chance that
///    the `6 + max(mv, 1)` cards seen by its turn hold enough sources of
///    each colour it needs, colours taken as independent;
/// 1. the worst spell's on-curve castability;
/// 2. the share of spells under 0.5 on curve (starved);
/// 3. mean castability with 15 cards seen (late game);
/// 4. the share of coloured lands that make no colour the spells need.
///
/// A land is a source of every colour it taps for
/// (`recommend::land_produced_colors`). A coloured or Phyrexian pip needs
/// its colour (Phyrexian life payment ignored); a two-colour hybrid pip is
/// read as whichever colour the deck has more sources of; mono- and
/// colourless-hybrid pips can be paid without the colour and need nothing.
pub fn mana_feats(deck: &[CardFactory]) -> [f32; MANA_FEATS] {
    use crate::mana::{Color, ManaSymbol as S};
    let mut sources = [0usize; 5];
    let mut land_colors = Vec::new();
    for f in deck {
        let def = crate::cube::card_def(*f);
        if def.is_land() {
            let made = crate::recommend::land_produced_colors(def);
            for c in Color::ALL {
                if made.contains(c) {
                    sources[c as usize] += 1;
                }
            }
            land_colors.push(made);
        }
    }
    let n = deck.len().max(1);
    let mut needed = crate::mana::ColorSet::empty();
    let (mut on_curve, mut late, mut worst, mut starved, mut spells) = (0f64, 0f64, 1f64, 0usize, 0usize);
    for f in deck {
        let def = crate::cube::card_def(*f);
        if def.is_land() {
            continue;
        }
        let mut need = [0usize; 5];
        for sym in &def.cost.symbols {
            match sym {
                S::Colored(c) | S::Phyrexian(c) => need[*c as usize] += 1,
                S::Hybrid(a, b) | S::PhyrexianHybrid(a, b) => {
                    let c = if sources[*b as usize] > sources[*a as usize] { *b } else { *a };
                    need[c as usize] += 1;
                }
                _ => {}
            }
        }
        let p = |draws: usize| -> f64 {
            Color::ALL
                .into_iter()
                .filter(|&c| need[c as usize] > 0)
                .map(|c| hyper_at_least(n, sources[c as usize], draws, need[c as usize]))
                .product()
        };
        for c in Color::ALL {
            if need[c as usize] > 0 {
                needed.insert(c);
            }
        }
        let now = p(6 + def.cost.cmc().max(1) as usize);
        on_curve += now;
        late += p(15);
        worst = worst.min(now);
        starved += usize::from(now < 0.5);
        spells += 1;
    }
    let colored: Vec<_> = land_colors.iter().filter(|m| !m.is_empty()).collect();
    let dead = colored.iter().filter(|m| !Color::ALL.into_iter().any(|c| m.contains(c) && needed.contains(c))).count();
    let k = spells.max(1) as f64;
    [
        (on_curve / k) as f32,
        if spells == 0 { 1.0 } else { worst as f32 },
        (starved as f64 / k) as f32,
        (late / k) as f32,
        dead as f32 / colored.len().max(1) as f32,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The additive form is the rating table's: a deck's value is the sum
    /// of its known cards' values, copies counted, unknown cards free —
    /// and a saved value reads back the same.
    #[test]
    fn an_additive_value_sums_its_cards_and_round_trips() {
        let bolt: CardFactory = crate::catalog::lightning_bolt;
        let shock: CardFactory = crate::catalog::shock;
        let island: CardFactory = crate::catalog::island;
        let d = DeckValue::new("A0", vec!["Lightning Bolt".into(), "Shock".into()], vec![0.5, -0.25]);
        assert!((d.value(&[bolt, bolt, shock, island]) - 0.75).abs() < 1e-6);
        assert!(d.knows("Shock") && !d.knows("Island"));
        let path = std::env::temp_dir().join(format!("deck_value_{}.json", std::process::id()));
        d.save(&path).expect("save");
        let back = DeckValue::load(&path).expect("load");
        std::fs::remove_file(&path).ok();
        assert!((back.value(&[bolt, shock]) - 0.25).abs() < 1e-6);
    }

    /// The mana summary sees sources: a red deck on Islands is starved and
    /// its lands dead; on Mountains it casts. And the hypergeometric is
    /// the textbook one (17 of 40, 7 seen, at least one: 1 - C(23,7)/C(40,7)).
    #[test]
    fn mana_feats_see_which_colours_the_lands_make() {
        let bolt: CardFactory = crate::catalog::lightning_bolt;
        let island: CardFactory = crate::catalog::island;
        let mountain: CardFactory = crate::catalog::mountain;
        let mut good = vec![bolt; 23];
        good.extend(std::iter::repeat_n(mountain, 17));
        let mut bad = vec![bolt; 23];
        bad.extend(std::iter::repeat_n(island, 17));
        let (g, b) = (mana_feats(&good), mana_feats(&bad));
        assert!(g[0] > 0.95 && b[0] == 0.0, "{g:?} {b:?}");
        assert_eq!((g[2], b[2]), (0.0, 1.0));
        assert_eq!((g[4], b[4]), (0.0, 1.0));
        let want = 1.0 - (0..7).map(|i| (23 - i) as f64 / (40 - i) as f64).product::<f64>();
        assert!((hyper_at_least(40, 17, 7, 1) - want).abs() < 1e-9);
    }
}
