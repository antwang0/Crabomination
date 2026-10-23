//! "Ward—Waterbend {N}" (The Unagi of Kyoshi Island, CR 701.67): each untapped
//! artifact or creature the payer taps pays {1} of it, mana pays the rest.

use super::GameState;
use crate::card::{CardId, CardType};
use crate::game::GameEvent;

impl GameState {
    /// Pay waterbend {`n`} as `payer` with the fewest helpers that leave the
    /// rest affordable: noncreature artifacts first, then creatures smallest
    /// first. `false` (nothing tapped) when no split covers it.
    pub(super) fn pay_waterbend(&mut self, payer: usize, n: u32, events: &mut Vec<GameEvent>) -> bool {
        let mut helpers: Vec<(bool, i32, CardId)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == payer && !c.tapped)
            .filter_map(|c| {
                let cp = self.computed_permanent_on(c)?;
                let creature = cp.card_types().contains(&CardType::Creature);
                (creature || cp.card_types().contains(&CardType::Artifact))
                    .then_some((creature, if creature { cp.power } else { 0 }, c.id))
            })
            .collect();
        helpers.sort();
        for k in 0..=helpers.len().min(n as usize) {
            let rest = n - k as u32;
            let mut probe = self.clone();
            for &(_, _, id) in &helpers[..k] {
                if let Some(c) = probe.battlefield.find_by_id_mut(id) {
                    c.tapped = true;
                }
            }
            if !probe.pay_generic(payer, rest) {
                continue;
            }
            for &(_, _, id) in &helpers[..k] {
                if let Some(c) = self.battlefield.find_by_id_mut(id) {
                    c.tapped = true;
                    events.push(GameEvent::PermanentTapped { card_id: id, actor: Some(payer), as_attacker: false });
                }
            }
            return self.pay_generic(payer, rest);
        }
        false
    }
}
