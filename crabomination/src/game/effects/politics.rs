//! Multiplayer offers each player may take: Orzhov Advokist's "each player
//! may put two +1/+1 counters on a creature they control; if a player does,
//! creatures that player controls can't attack you or planeswalkers you
//! control until your next turn", Agitator Ant's goading sibling, Kwain's
//! "each player may draw a card, then each player who drew a card this way
//! gains 1 life", and the two secret choices of Blame Game (Mob Verdict's
//! vote for a player, Prisoner's Dilemma's silence or snitch).

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
    /// With `goad` (Agitator Ant, `Effect::EachPlayerMayCounterThenGoad`) the
    /// rider is instead "goad each creature that had counters put on it this
    /// way" (CR 701.15), the offerer's own taker included.
    pub(super) fn each_player_may_counter_for_peace(
        &mut self,
        counters: u32,
        goad: bool,
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
            let rider = if goad {
                "It is then goaded."
            } else {
                "Its controller's creatures then can't attack the offerer until their next turn."
            };
            let Some(yes) = self.ask_seat_bool(
                &mut cursor,
                q,
                format!("Put {counters} +1/+1 counters on a creature? {rider}"),
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
            if goad {
                self.run_effect(&Effect::Goad { what: Selector::ExactObjects(vec![id]) }, ctx, events)?;
            } else if q != me {
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

    /// `Effect::EachPlayerMayDrawThenTakersGainLife` — APNAP, each living
    /// player is asked (log-replayed); the takers each draw a card, then each
    /// gains `life`.
    pub(super) fn each_player_may_draw_then_gain(
        &mut self,
        life: u32,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let source = ctx.source.unwrap_or(CardId(0));
        let seats = self.apnap_sort(self.living_seats().collect());
        let mut cursor = 0usize;
        let mut takers: Vec<usize> = Vec::new();
        for q in seats {
            let Some(yes) = self.ask_seat_bool(
                &mut cursor,
                q,
                format!("Draw a card? Each player who does gains {life} life."),
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
        for &q in &takers {
            self.run_effect(&Effect::Draw { who: Selector::Player(PlayerRef::Seat(q)), amount: Value::ONE }, ctx, events)?;
        }
        for &q in &takers {
            self.run_effect(
                &Effect::GainLife { who: Selector::Player(PlayerRef::Seat(q)), amount: Value::Const(life as i32) },
                ctx,
                events,
            )?;
        }
        Ok(())
    }

    /// `Effect::EachPlayerVotesForAPlayer` — CR 701.38: starting with the
    /// controller, in turn order, each living player votes for another living
    /// player (secret council: nobody's vote is shown until all are cast, so
    /// the `Voted` events go out together). Each vote then runs its body: a
    /// vote for an opponent binds that opponent as `PlayerRef::Triggerer`.
    ///
    /// The ballot lists the controller's opponents first, most life first, so
    /// a headless voter's first option is the table's leader, never itself.
    pub(super) fn each_player_votes_for_a_player(
        &mut self,
        on_opponent: &Effect,
        on_you: &Effect,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let voters = self.seats_in_turn_order_from(me);
        let mut cursor = 0usize;
        let mut cast: Vec<(usize, usize)> = Vec::new();
        for &v in &voters {
            let mut ballot: Vec<usize> = voters.iter().copied().filter(|&q| q != v).collect();
            ballot.sort_by_key(|&q| (q == me, std::cmp::Reverse(self.players[q].life), q));
            if ballot.is_empty() {
                continue;
            }
            let labels: Vec<String> = ballot.iter().map(|q| format!("Player {}", q + 1)).collect();
            let asked = self.vote_controller_this_turn.unwrap_or(v);
            for _ in 0..1 + self.additional_votes_for(v) {
                let prompt = "Vote for a player".to_string();
                let Some(pick) = self.ask_seat_option(&mut cursor, asked, prompt, source, labels.clone(), effect) else {
                    return Ok(());
                };
                cast.push((v, ballot[pick.min(ballot.len() - 1)]));
            }
        }
        self.clear_answer_log();
        for &(v, q) in &cast {
            events.push(GameEvent::Voted { player: v, choice: format!("Player {}", q + 1) });
        }
        events.push(GameEvent::VotingFinished);
        let body = Effect::seq(
            cast.iter()
                .map(|&(_, q)| {
                    if q == me {
                        on_you.clone()
                    } else {
                        Effect::ForEach { selector: Selector::Player(PlayerRef::Seat(q)), body: Box::new(on_opponent.clone()) }
                    }
                })
                .collect(),
        );
        self.run_effect(&body, ctx, events)
    }

    /// `Effect::OpponentsChooseSilenceOrSnitch` — Prisoner's Dilemma. Each
    /// living opponent, in turn order, secretly picks; the damage is dealt
    /// once every choice is in. A headless opponent snitches: it never takes
    /// more damage for it, whatever the others chose.
    pub(super) fn opponents_choose_silence_or_snitch(
        &mut self,
        (all_silence, all_snitch, mixed): (u32, u32, u32),
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let opps: Vec<usize> =
            self.seats_in_turn_order_from(me).into_iter().filter(|&q| q != me && !self.same_team(me, q)).collect();
        let mut cursor = 0usize;
        let mut silent: Vec<bool> = Vec::with_capacity(opps.len());
        for &q in &opps {
            let Some(pick) = self.ask_seat_option(
                &mut cursor,
                q,
                "Secretly choose".into(),
                source,
                vec!["Snitch".into(), "Silence".into()],
                effect,
            ) else {
                return Ok(());
            };
            silent.push(pick == 1);
        }
        self.clear_answer_log();
        let hits: Vec<(usize, u32)> = if silent.iter().all(|&s| s) {
            opps.iter().map(|&q| (q, all_silence)).collect()
        } else if silent.iter().all(|&s| !s) {
            opps.iter().map(|&q| (q, all_snitch)).collect()
        } else {
            opps.iter().zip(&silent).filter(|(_, s)| **s).map(|(&q, _)| (q, mixed)).collect()
        };
        for (q, n) in hits {
            self.run_effect(
                &Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Seat(q)),
                    amount: Value::Const(n as i32),
                },
                ctx,
                events,
            )?;
        }
        Ok(())
    }
}
