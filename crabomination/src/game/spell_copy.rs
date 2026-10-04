//! CR 707.10 — "a copy of a spell … copies … all decisions made for it,
//! including modes, targets, the value of X, and additional or alternative
//! costs", and an effect of the copy that refers to objects used to pay its
//! costs uses the original's (Fling's discarded card). Every copy is a fresh
//! `CardInstance`, so these per-cast fields are carried over explicitly.
//! Not copied: anything about mana actually spent (a copy has none — converge
//! is 0, the 2015-08-25 rulings) and whether the spell was CAST (it wasn't).

use crate::card::CardInstance;

/// The per-cast decisions a copy inherits.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CastChoices {
    kicked: bool,
    kick_count: u32,
    kicked_options: Vec<u8>,
    spree_modes: Vec<u8>,
    entwined: bool,
    bargained: bool,
    gift_promised: bool,
    squad_count: u32,
    split_cast: Option<u8>,
    cast_discarded_mana_value: Option<u32>,
    cast_collected_evidence: bool,
}

impl CastChoices {
    pub(crate) fn of(c: &CardInstance) -> Self {
        Self {
            kicked: c.kicked,
            kick_count: c.kick_count,
            kicked_options: c.kicked_options.clone(),
            spree_modes: c.spree_modes.clone(),
            entwined: c.entwined,
            bargained: c.bargained,
            gift_promised: c.gift_promised,
            squad_count: c.squad_count,
            split_cast: c.split_cast,
            cast_discarded_mana_value: c.cast_discarded_mana_value,
            cast_collected_evidence: c.cast_collected_evidence,
        }
    }

    /// Stamp the decisions onto a fresh copy. A spell cast with none of them
    /// (nearly all) writes nothing.
    pub(crate) fn apply(&self, c: &mut CardInstance) {
        if *self == Self::default() {
            return;
        }
        c.kicked = self.kicked;
        c.kick_count = self.kick_count;
        c.kicked_options = self.kicked_options.clone();
        c.spree_modes = self.spree_modes.clone();
        c.entwined = self.entwined;
        c.bargained = self.bargained;
        c.gift_promised = self.gift_promised;
        c.squad_count = self.squad_count;
        c.split_cast = self.split_cast;
        c.cast_discarded_mana_value = self.cast_discarded_mana_value;
        c.cast_collected_evidence = self.cast_collected_evidence;
    }
}
