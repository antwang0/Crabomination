//! Values read over a selector's cards: the distinct card types among them
//! (Occult Epiphany, CR 205.2a) and the opponents who control one of them
//! (Sudden Salvation).

use crate::effect::Selector;
use crate::game::effects::{EffectContext, EntityRef};
use crate::game::GameState;

impl GameState {
    fn among_card_ids(&self, sel: &Selector, ctx: &EffectContext) -> Vec<crate::card::CardId> {
        self.resolve_selector(sel, ctx)
            .into_iter()
            .filter_map(|e| match e {
                EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                _ => None,
            })
            .collect()
    }

    /// `Value::CardTypesAmong` — CR 205.2a card types, each counted once.
    pub(crate) fn card_types_among(&self, sel: &Selector, ctx: &EffectContext) -> i32 {
        let mut seen: Vec<crate::card::CardType> = Vec::new();
        for id in self.among_card_ids(sel, ctx) {
            let Some(c) = self.find_card_anywhere(id) else { continue };
            for t in &c.definition.card_types {
                if !seen.contains(t) {
                    seen.push(t.clone());
                }
            }
        }
        seen.len() as i32
    }

    /// `Value::OpponentsControllingAnyOf` — opponents of the resolving
    /// controller who control at least one of the selected permanents.
    pub(crate) fn opponents_controlling_any_of(&self, sel: &Selector, ctx: &EffectContext) -> i32 {
        let opponents = self.opponents_of(ctx.controller);
        let mut seen: Vec<usize> = Vec::new();
        for id in self.among_card_ids(sel, ctx) {
            if let Some(c) = self.battlefield_find(id)
                && opponents.contains(&c.controller)
                && !seen.contains(&c.controller)
            {
                seen.push(c.controller);
            }
        }
        seen.len() as i32
    }
}
