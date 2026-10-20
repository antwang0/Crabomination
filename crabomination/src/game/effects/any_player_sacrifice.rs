//! `Effect::AnyPlayerMaySacrificeToTopdeckSource` — "any player may sacrifice
//! a [filter] of their choice. If a player does, put this creature on top of
//! its owner's library" (Shivan Wumpus). Each living seat is asked in turn
//! order from the controller; the first who sacrifices ends the ask. A
//! headless opponent with plenty of lands to spare pays; the controller never
//! does unasked.

use crate::card::SelectionRequirement;
use crate::decision::OptionalKind;
use crate::effect::{Effect, LibraryPosition, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::{GameError, GameEvent, GameState};

/// Matching permanents a headless opponent keeps after paying.
const HEADLESS_SPARE: usize = 5;

impl GameState {
    pub(super) fn any_player_may_sacrifice_to_topdeck_source(
        &mut self,
        filter: &SelectionRequirement,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(src) = ctx.source else { return Ok(()) };
        if self.battlefield_find(src).is_none() {
            return Ok(());
        }
        let mut cursor = 0;
        for seat in self.seats_in_turn_order_from(ctx.controller) {
            if !self.players[seat].is_alive() {
                continue;
            }
            let owned = self
                .battlefield
                .iter()
                .filter(|c| c.controller == seat && self.evaluate_requirement_static(filter, &crate::game::Target::Permanent(c.id), seat, Some(src)))
                .count();
            if owned == 0 {
                continue;
            }
            let pays = if self.seat_prompts(seat) {
                let Some(yes) = self.ask_seat_bool(
                    &mut cursor,
                    seat,
                    "Sacrifice a permanent to put this creature on top of its owner's library?".into(),
                    src,
                    effect,
                    OptionalKind::Neutral,
                ) else {
                    return Ok(());
                };
                yes
            } else {
                seat != ctx.controller && owned > HEADLESS_SPARE
            };
            if pays {
                self.clear_answer_log();
                self.run_effect(
                    &Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::Seat(seat)),
                        count: Value::ONE,
                        filter: filter.clone(),
                    },
                    ctx,
                    events,
                )?;
                return self.run_effect(
                    &Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::This)), pos: LibraryPosition::Top },
                    },
                    ctx,
                    events,
                );
            }
        }
        self.clear_answer_log();
        Ok(())
    }
}
