//! Deification's two halves: the as-enters planeswalker-type choice and the
//! "all but one loyalty counter" damage floor (CR 614.1a).

use crate::card::{CardId, PlaneswalkerSubtype};
use crate::effect::{Effect, Selector, StaticEffect};
use crate::game::GameState;
use crate::game::effects::{EffectContext, EntityRef};

impl GameState {
    /// The planeswalker types among cards `seat` owns, most-held first (name
    /// order breaks ties, so the ballot is seed-stable).
    fn owned_planeswalker_types(&self, seat: usize) -> Vec<PlaneswalkerSubtype> {
        let mut tally: Vec<(PlaneswalkerSubtype, u32)> = Vec::new();
        let pl = &self.players[seat];
        let owned = self
            .battlefield
            .iter()
            .filter(|c| c.owner == seat)
            .chain(pl.hand.iter())
            .chain(pl.library.iter())
            .chain(pl.graveyard.iter())
            .chain(pl.command.iter());
        for c in owned {
            for t in &c.definition.subtypes.planeswalker_subtypes {
                match tally.iter_mut().find(|(k, _)| k == t) {
                    Some((_, n)) => *n += 1,
                    None => tally.push((*t, 1)),
                }
            }
        }
        tally.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0))));
        tally.into_iter().map(|(t, _)| t).collect()
    }

    /// `Effect::NamePlaneswalkerType` — the controller picks from the types
    /// they own (a seat with none names nothing); the pick is stamped on the
    /// entering permanent.
    pub(crate) fn name_planeswalker_type(&mut self, what: &Selector, ctx: &EffectContext, effect: &Effect) {
        let Some(id) = self
            .resolve_selector(what, ctx)
            .into_iter()
            .find_map(|e| match e {
                EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                _ => None,
            })
            .or(ctx.source)
        else {
            return;
        };
        let types = self.owned_planeswalker_types(ctx.controller);
        if types.is_empty() {
            return;
        }
        let labels = types.iter().map(|t| format!("{t:?}")).collect();
        let mut cursor = 0;
        let Some(i) =
            self.ask_seat_option(&mut cursor, ctx.controller, "Choose a planeswalker type".into(), id, labels, effect)
        else {
            return;
        };
        self.clear_answer_log();
        if let Some(c) = self.battlefield_find_mut(id) {
            c.chosen_planeswalker_type = types.get(i).copied();
        }
    }

    /// The loyalty `pw` can't be damaged below: 1 while a
    /// `DamageLeavesYourPlaneswalkersAtOneLoyalty` its controller controls
    /// covers it, else 0.
    pub(crate) fn planeswalker_loyalty_floor(&self, pw: CardId) -> u32 {
        let Some(p) = self.battlefield_find(pw) else { return 0 };
        let ctrl = p.controller;
        let target = crate::game::Target::Permanent(pw);
        let covered = self.battlefield.iter().any(|src| {
            src.controller == ctrl
                && src.definition.static_abilities.iter().any(|sa| match &sa.effect {
                    StaticEffect::DamageLeavesYourPlaneswalkersAtOneLoyalty { filter, requires_creature } => {
                        (!requires_creature
                            || self.battlefield.iter().any(|c| c.controller == ctrl && self.computed_is_creature(c)))
                            && self.evaluate_requirement_static(filter, &target, ctrl, Some(src.id))
                    }
                    _ => false,
                })
        });
        u32::from(covered)
    }

    /// Loyalty after `amount` damage to `pw`, honouring its floor: damage that
    /// would remove every counter leaves one instead.
    pub(crate) fn loyalty_after_damage(&self, pw: CardId, current: u32, amount: u32) -> u32 {
        let after = current.saturating_sub(amount);
        if after == 0 && current > 0 && amount > 0 {
            after.max(self.planeswalker_loyalty_floor(pw))
        } else {
            after
        }
    }
}
