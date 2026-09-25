//! Two library reveals from Arcane Maelstrom: Rashmi, Eternities Crafter's
//! "cast it free if it's a spell with lesser mana value, else put it into
//! your hand", and Nascent Metamorph's "reveal until a creature card, become a
//! copy of it until end of turn".

use crate::card::CardId;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::RevealTopCastFreeIfLesserElseHand` — reveal your top card; a
    /// nonland card with mana value less than the triggering spell's may be
    /// cast without paying its mana cost; if it isn't cast, it goes to hand.
    pub(super) fn reveal_top_cast_free_if_lesser_else_hand(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let Some(top) = self.players[p].library.first().map(|c| (c.id, c.definition.is_land(), c.printed_cmc()))
        else {
            return Ok(());
        };
        let (id, is_land, mv) = top;
        let spell_mv = self.evaluate_value(&Value::ManaValueOf(Box::new(Selector::TriggerSource)), ctx).max(0) as u32;
        if !is_land && mv < spell_mv {
            let c = EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
            let _ = self.run_effect(
                &Effect::CastWithoutPayingImmediate {
                    what: Selector::Target(0),
                    source_zone: crate::card::Zone::Library,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
                &c,
                events,
            );
        }
        if self.players[p].library.iter().any(|c| c.id == id) {
            self.move_card_to(id, &ZoneDest::Hand(PlayerRef::You), ctx, events);
        }
        Ok(())
    }

    /// `Effect::RevealUntilCreatureBecomeCopy` — `who` reveals from the top
    /// until a creature card; the source becomes a copy of it until end of
    /// turn; every revealed card goes to the bottom in a random order.
    pub(super) fn reveal_until_creature_become_copy(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let Some(pos) = self.players[seat].library.iter().position(|c| c.definition.is_creature()) else {
            return Ok(());
        };
        let revealed: Vec<CardId> = self.players[seat].library.iter().take(pos + 1).map(|c| c.id).collect();
        let found = revealed[pos];
        let c = EffectContext { targets: vec![Target::Permanent(found)], ..ctx.clone() };
        let _ = self.run_effect(
            &Effect::BecomeCopyOfFor {
                what: Selector::This,
                source: Selector::Target(0),
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
            &c,
            events,
        );
        use rand::seq::SliceRandom;
        let mut rest = revealed;
        rest.shuffle(&mut self.rng.draw());
        for id in rest {
            if let Some(card) = Self::take_card(&mut self.players[seat].library, id) {
                self.players[seat].library.push(card);
            }
        }
        Ok(())
    }
}
