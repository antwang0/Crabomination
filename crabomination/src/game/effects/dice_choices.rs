//! Dice rolls whose results feed a choice or a comparison: Wild Endeavor's
//! "roll two d4 and choose one result", Chaos Dragon's "each player rolls a
//! d20", and Klauth's any-colors kept mana that goes with them in the Draconic
//! Rage precon.

use super::EffectContext;
use crate::card::{CardId, Keyword};
use crate::effect::{Duration, Effect, PlayerRef, ScratchBinding, Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};
use crate::mana::Color;

impl GameState {
    /// `Effect::RollTwoDiceAssign` — CR 706.2: both dice are rolled (and the
    /// roll triggers, CR 706.3a) before anything reads them; the assignment
    /// then resolves as `AssignTwoDieResults` so a suspended choice replays
    /// with these faces rather than rolling again.
    pub(super) fn roll_two_dice_assign(
        &mut self,
        sides: u8,
        first: &Effect,
        second: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let sides = sides.max(2);
        let a = self.roll_one_die(ctx.controller, sides);
        let b = self.roll_one_die(ctx.controller, sides);
        events.push(GameEvent::DiceRolled { player: ctx.controller, count: 2, high: a.max(b) });
        for face in [a, b] {
            if face == sides {
                events.push(GameEvent::RolledNaturalMax { player: ctx.controller });
            }
        }
        let assign = Effect::AssignTwoDieResults {
            a,
            b,
            first: Box::new(first.clone()),
            second: Box::new(second.clone()),
        };
        self.run_effect(&assign, ctx, events)
    }

    /// `Effect::AssignTwoDieResults` — the controller chooses which face
    /// `first` gets. Option 0 hands it the higher face, which is what a
    /// headless seat takes.
    pub(super) fn assign_two_die_results(
        &mut self,
        (a, b): (u8, u8),
        first: &Effect,
        second: &Effect,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let (hi, lo) = (a.max(b), a.min(b));
        let (to_first, to_second) = if hi == lo {
            (hi, lo)
        } else {
            let mut cursor = 0usize;
            let Some(pick) = self.ask_seat_option(
                &mut cursor,
                ctx.controller,
                format!("Rolled {hi} and {lo}: choose the first result"),
                ctx.source.unwrap_or(CardId(0)),
                vec![format!("{hi} first, {lo} second"), format!("{lo} first, {hi} second")],
                effect,
            ) else {
                return Ok(());
            };
            self.clear_answer_log();
            if pick == 0 { (hi, lo) } else { (lo, hi) }
        };
        let body = Effect::Seq(vec![
            Effect::BindScratch {
                scratch: ScratchBinding::LastDieRoll(to_first),
                body: Box::new(first.clone()),
            },
            Effect::BindScratch {
                scratch: ScratchBinding::LastDieRoll(to_second),
                body: Box::new(second.clone()),
            },
        ]);
        self.run_effect(&body, ctx, events)
    }

    /// `Effect::EachPlayerRollsSourceCantAttackHighest` — CR 706.2: every
    /// living seat rolls its own die, starting with the controller. Each
    /// opponent whose result ties the table's highest is barred from the
    /// source's attacks this combat (it and its planeswalkers,
    /// `Keyword::CantAttackPlayer`). The controller topping the table alone
    /// bars nobody.
    pub(super) fn each_player_rolls_source_cant_attack_highest(
        &mut self,
        sides: u8,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let sides = sides.max(2);
        let me = ctx.controller;
        let seats: Vec<usize> = self
            .seats_in_turn_order_from(me)
            .into_iter()
            .filter(|&q| self.players[q].is_alive())
            .collect();
        let mut rolls: Vec<(usize, u8)> = Vec::with_capacity(seats.len());
        for q in seats {
            let face = self.roll_one_die(q, sides);
            events.push(GameEvent::DiceRolled { player: q, count: 1, high: face });
            if face == sides {
                events.push(GameEvent::RolledNaturalMax { player: q });
            }
            rolls.push((q, face));
        }
        let best = rolls.iter().map(|&(_, r)| r).max().unwrap_or(0);
        for (q, r) in rolls {
            if r == best && q != me && !self.same_team(me, q) {
                let grant = Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::CantAttackPlayer(q),
                    duration: Duration::EndOfCombat,
                };
                self.run_effect(&grant, ctx, events)?;
            }
        }
        Ok(())
    }

    /// `Effect::AddManaKeptThisTurnAnyColors` — each pip's color is chosen
    /// on its own (the needs-aware pick for a headless seat), added to the
    /// pool and to the kept-this-turn pool that step ends re-seed.
    pub(super) fn add_mana_kept_this_turn_any_colors(
        &mut self,
        who: &PlayerRef,
        amount: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let n = self.evaluate_value(amount, ctx).max(0) as u32;
        let mult = self.mana_production_multiplier.max(1);
        let legal = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
        for _ in 0..(n * mult) {
            let color = self.chosen_mana_color(p, &legal, ctx.source);
            self.players[p].mana_pool.add(color, 1);
            self.players[p].kept_mana_this_turn.add(color, 1);
            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
        }
        Ok(())
    }
}
