//! Multiplayer offers a player may take for a price paid to the offerer:
//! Orzhov Advokist's "each player may put two +1/+1 counters on a creature
//! they control; if a player does, creatures that player controls can't
//! attack you or planeswalkers you control until your next turn".

use crate::card::{CardId, CounterType, Keyword, SelectionRequirement};
use crate::decision::OptionalKind;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::EachPlayerMayCounterForPeace` — APNAP, each player with a
    /// creature is asked (log-replayed, so a prompting seat can suspend);
    /// each taker's greatest-power creature gets `counters`, and every
    /// creature a taker other than the offerer controls can't attack the
    /// offerer until the offerer's next turn (CR 508.1a).
    pub(super) fn each_player_may_counter_for_peace(
        &mut self,
        counters: u32,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let seats = self.apnap_sort(self.living_seats().collect());
        let best = |g: &GameState, q: usize| -> Option<CardId> {
            g.battlefield
                .iter()
                .filter(|c| c.controller == q)
                .filter_map(|c| g.computed_permanent(c.id).filter(|cp| cp.card_types().contains(&crate::card::CardType::Creature)).map(|cp| (cp.power, c.id)))
                .max_by_key(|&(p, id)| (p, std::cmp::Reverse(id.0)))
                .map(|(_, id)| id)
        };
        let mut cursor = 0usize;
        let mut takers: Vec<usize> = Vec::new();
        for q in seats {
            if best(self, q).is_none() {
                continue;
            }
            let Some(yes) = self.ask_seat_bool(
                &mut cursor,
                q,
                format!("Put {counters} +1/+1 counters on a creature? Its controller's creatures then can't attack the offerer until their next turn."),
                source,
                effect,
                OptionalKind::MayBody,
            ) else {
                return Ok(());
            };
            if yes {
                takers.push(q);
            }
        }
        self.clear_answer_log();
        for q in takers {
            let Some(id) = best(self, q) else { continue };
            self.run_effect(
                &Effect::AddCounter {
                    what: Selector::ExactObjects(vec![id]),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(counters as i32),
                },
                ctx,
                events,
            )?;
            if q != me {
                self.run_effect(
                    &Effect::GrantKeyword {
                        what: Selector::ControlledBy { who: PlayerRef::Seat(q), filter: SelectionRequirement::Creature },
                        keyword: Keyword::CantAttackPlayer(me),
                        duration: Duration::UntilNextTurn,
                    },
                    ctx,
                    events,
                )?;
            }
        }
        Ok(())
    }
}
