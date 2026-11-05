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
    fn fwd(&self, x: &[f32], relu: bool) -> Vec<f32> {
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
    #[serde(skip)]
    index: HashMap<String, usize>,
}

impl DeckValue {
    pub fn new(arm: &str, cards: Vec<String>, a: Vec<f32>) -> Self {
        let mut d = Self { arm: arm.to_string(), cards, a, x: Vec::new(), b: Vec::new(), ctx: None, deck_mlp: None, index: HashMap::new() };
        d.reindex();
        d
    }

    fn reindex(&mut self) {
        self.index = self.cards.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
    }

    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut d: Self = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
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
    /// nothing.
    pub fn value(&self, deck: &[CardFactory]) -> f64 {
        let mut counts: HashMap<usize, f32> = HashMap::new();
        for f in deck {
            if let Some(&i) = self.index.get(crate::cube::card_def(*f).name) {
                *counts.entry(i).or_insert(0.0) += 1.0;
            }
        }
        let mut f: f32 = counts.iter().map(|(&i, &n)| n * self.a[i]).sum();
        let need_feats = self.ctx.is_some() || self.deck_mlp.is_some();
        let feats = if need_feats {
            crate::server::encode::encode_deck(deck, crate::server::net_eval::vocab()).1.to_vec()
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
}
