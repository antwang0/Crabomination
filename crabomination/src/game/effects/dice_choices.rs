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
            events.push(GameEvent::DieResult { player: ctx.controller, result: face });
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
            events.push(GameEvent::DieResult { player: q, result: face });
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
    /// `Effect::AddManaKeptThisTurnAnyOneColor` — one color chosen for the
    /// whole amount (Photon, Mighty Marvel).
    pub(super) fn add_mana_kept_this_turn_any_one_color(
        &mut self,
        who: &PlayerRef,
        amount: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let n = self.evaluate_value(amount, ctx).max(0) as u32 * self.mana_production_multiplier.max(1);
        if n == 0 {
            return Ok(());
        }
        let legal = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
        let color = self.chosen_mana_color(p, &legal, ctx.source);
        self.players[p].mana_pool.add(color, n);
        self.players[p].kept_mana_this_turn.add(color, n);
        for _ in 0..n {
            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
        }
        Ok(())
    }

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
            // "Spend this mana only to cast spells."
            let only = crate::mana::SpendRestriction::SpellsOnly;
            self.players[p].mana_pool.add_restricted(color, 1, only);
            self.players[p].kept_mana_this_turn.add_restricted(color, 1, only);
            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
        }
        Ok(())
    }

    /// `Effect::AddManaKeptThisTurnOfColors` (Grand Warlord Radha): each pip
    /// a color from `colors`, chosen per pip, unrestricted.
    pub(super) fn add_mana_kept_this_turn_of_colors(
        &mut self,
        who: &PlayerRef,
        colors: &[Color],
        amount: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        if colors.is_empty() {
            return Ok(());
        }
        let n = self.evaluate_value(amount, ctx).max(0) as u32;
        let mult = self.mana_production_multiplier.max(1);
        for _ in 0..(n * mult) {
            let color = self.chosen_mana_color(p, colors, ctx.source);
            self.players[p].mana_pool.add(color, 1);
            self.players[p].kept_mana_this_turn.add(color, 1);
            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
        }
        Ok(())
    }

    pub(super) fn add_colorless_kept_this_turn(
        &mut self,
        who: &PlayerRef,
        amount: &Value,
        restriction: Option<crate::mana::SpendRestriction>,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let n = (self.evaluate_value(amount, ctx).max(0) as u32) * self.mana_production_multiplier.max(1);
        if n == 0 {
            return Ok(());
        }
        match restriction {
            Some(r) => {
                self.players[p].mana_pool.add_restricted_colorless(n, r);
                self.players[p].kept_mana_this_turn.add_restricted_colorless(n, r);
            }
            None => {
                self.players[p].mana_pool.add_colorless(n);
                self.players[p].kept_mana_this_turn.add_colorless(n);
            }
        }
        for _ in 0..n {
            events.push(GameEvent::ColorlessManaAdded { player: p, source: ctx.source });
        }
        Ok(())
    }
}

impl GameState {
    /// CR 706.2 — Night Shift of the Living Dead: once each turn, after `seat`
    /// rolls, it may pay 1 life to move one die's result by one. The headless
    /// pick takes +1 when that lands a die in a later results-table arm, or,
    /// with no table, on a result threshold one of its die triggers names; it
    /// never pays below 6 life. A deciding seat gets the ballot, headless
    /// answer first; a UI seat takes the headless pick (the roll can't park
    /// mid-resolution — INCOMPLETE_CARDS).
    pub(super) fn offer_die_shift(&mut self, seat: usize, rolls: &mut [u8], results: &[(u8, u8, Effect)]) {
        use crate::card::Predicate;
        use crate::decision::{Decision, DecisionAnswer};
        use crate::effect::{EventKind, KeywordAct, StaticEffect};
        if self.players[seat].die_shift_used_this_turn || self.players[seat].life < 6 {
            return;
        }
        let mine = || self.battlefield.iter().filter(|c| c.controller == seat);
        if !mine().any(|c| {
            c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::PayLifeToShiftDieOnceEachTurn))
        }) {
            return;
        }
        fn thresholds(p: &Predicate, out: &mut Vec<u8>) {
            match p {
                Predicate::DieResultAtLeast(n) => out.push(*n),
                Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(n)) => out.push(*n as u8),
                Predicate::All(ps) | Predicate::Any(ps) => ps.iter().for_each(|q| thresholds(q, out)),
                _ => {}
            }
        }
        let mut wanted: Vec<u8> = Vec::new();
        for c in mine() {
            for t in &c.definition.triggered_abilities {
                if matches!(t.event.kind, EventKind::RolledDice | EventKind::Performed(KeywordAct::DieResult))
                    && let Some(f) = &t.event.filter
                {
                    thresholds(f, &mut wanted);
                }
            }
        }
        let arm = |r: u8| results.iter().position(|(lo, hi, _)| r >= *lo && r <= *hi);
        let better = |r: u8| {
            let up = r.saturating_add(1);
            if results.is_empty() { wanted.contains(&up) } else { arm(up).is_some_and(|b| arm(r).is_none_or(|a| b > a)) }
        };
        // The ballot: the headless answer first, then keeping the results,
        // then +1 / -1 on each die in order.
        let mut choices: Vec<Option<(usize, bool)>> = vec![rolls.iter().position(|&r| better(r)).map(|k| (k, true))];
        choices.push(None);
        for k in 0..rolls.len() {
            choices.extend([Some((k, true)), Some((k, false))]);
        }
        choices.dedup();
        let label = |c: &Option<(usize, bool)>| match *c {
            None => "Keep the results".to_string(),
            Some((k, up)) => {
                let to = if up { rolls[k].saturating_add(1) } else { rolls[k].saturating_sub(1) };
                format!("Pay 1 life: die {} {} -> {}", k + 1, rolls[k], to)
            }
        };
        let pick = match self.decider.kind() {
            crate::decision::DeciderKind::Auto => choices[0],
            _ => {
                let decision = Decision::ChooseOption {
                    source: CardId(0),
                    prompt: "Night Shift of the Living Dead".to_string(),
                    options: choices.iter().map(label).collect(),
                };
                match self.decider.decide(&decision) {
                    DecisionAnswer::Amount(i) => choices.get(i as usize).copied().unwrap_or(choices[0]),
                    _ => choices[0],
                }
            }
        };
        let Some((k, up)) = pick else { return };
        rolls[k] = if up { rolls[k].saturating_add(1) } else { rolls[k].saturating_sub(1) };
        self.players[seat].die_shift_used_this_turn = true;
        self.pay_life_cost(seat, 1);
    }
}
