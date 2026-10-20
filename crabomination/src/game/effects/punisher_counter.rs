//! `Effect::AnyPlayerMayPayLifeToCounterThis` — "When you cast this spell,
//! any player may pay N life. If a player does, counter [this spell]" (Dash
//! Hopes). Each living seat is asked in turn order from the caster; only a
//! seat that can pay is asked (CR 119.4). A prompting seat answers; a
//! headless one pays when the spell is aimed at a spell of its own and it
//! keeps a margin of life after paying. The caster never pays to counter its
//! own spell unasked. The first payer counters the spell and the ask ends.

use crate::card::CardId;
use crate::decision::{OptionalKind, PayFor};
use crate::effect::{Effect, Selector};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, StackItem, Target};
use crate::game::{GameError, GameState};

/// Life a headless seat keeps after paying, so it never pays itself into
/// lethal range for a counterspell.
const HEADLESS_MARGIN: i32 = 5;

impl GameState {
    /// The caster of the spell the stack spell `spell` targets in slot 0.
    fn caster_of_spell_targeted_by(&self, spell: CardId) -> Option<usize> {
        let target = self.stack.iter().find_map(|si| match si {
            StackItem::Spell { card, target: Some(Target::Permanent(t)), .. } if card.id == spell => Some(*t),
            _ => None,
        })?;
        self.stack.iter().find_map(|si| match si {
            StackItem::Spell { card, caster, .. } if card.id == target => Some(*caster),
            _ => None,
        })
    }

    pub(super) fn any_player_may_pay_life_to_counter_this(
        &mut self,
        life: u32,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(spell) = ctx.source else { return Ok(()) };
        if !self.stack.iter().any(|si| matches!(si, StackItem::Spell { card, .. } if card.id == spell)) {
            return Ok(());
        }
        let victim = self.caster_of_spell_targeted_by(spell);
        let mut cursor = 0;
        for seat in self.seats_in_turn_order_from(ctx.controller) {
            if !self.players[seat].is_alive() || self.players[seat].life < life as i32 {
                continue;
            }
            let pays = if self.seat_prompts(seat) {
                let Some(yes) = self.ask_seat_bool(
                    &mut cursor,
                    seat,
                    format!("Pay {life} life to counter this spell?"),
                    spell,
                    effect,
                    OptionalKind::PayLife { life, purpose: PayFor::DenyEffect },
                ) else {
                    return Ok(());
                };
                yes
            } else {
                seat != ctx.controller
                    && victim == Some(seat)
                    && self.players[seat].life - life as i32 >= HEADLESS_MARGIN
            };
            if pays {
                self.pay_life_cost(seat, life);
                return self.run_effect(&Effect::CounterSpell { what: Selector::This }, ctx, events);
            }
        }
        Ok(())
    }
}
