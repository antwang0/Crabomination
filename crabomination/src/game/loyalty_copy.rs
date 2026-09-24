//! Planeswalker Party's loyalty-ability rules (CR 606): Oath of Teferi's
//! "twice each turn", the copy grants of Jaya's Phoenix, Leori and Repeated
//! Reverberation, Teyo's temporary attack direction and Guff Rewrites
//! History's shuffle-in-and-recast.

use crate::card::{CardId, PlaneswalkerSubtype};
use crate::effect::StaticEffect;
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::player::LoyaltyCopyGrant;

impl GameState {
    /// CR 606.3 — Oath of Teferi: `p` controls a permanent letting its
    /// planeswalkers activate loyalty abilities twice each turn.
    pub(crate) fn loyalty_twice_each_turn_for(&self, p: usize) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::LoyaltyAbilitiesTwiceEachTurn))
        })
    }

    pub(crate) fn grant_loyalty_copies(
        &mut self,
        p: usize,
        copies: u32,
        subtype: Option<PlaneswalkerSubtype>,
        once: bool,
    ) {
        self.players[p].loyalty_copy_grants.push(LoyaltyCopyGrant { copies, subtype, once });
    }

    /// Leori — the planeswalker type most common among `p`'s planeswalkers
    /// on the battlefield and in hand (ties to the first seen).
    pub(crate) fn grant_loyalty_copies_of_chosen_type(&mut self, p: usize) {
        let mut tally: Vec<(PlaneswalkerSubtype, u32)> = Vec::new();
        let hand = self.players[p].hand.iter().map(|c| &c.definition);
        let board = self.battlefield.iter().filter(|c| c.controller == p).map(|c| &c.definition);
        for def in board.chain(hand).filter(|d| d.is_planeswalker()) {
            for t in &def.subtypes.planeswalker_subtypes {
                match tally.iter_mut().find(|(k, _)| k == t) {
                    Some((_, n)) => *n += 1,
                    None => tally.push((*t, 1)),
                }
            }
        }
        let Some(best) = tally.iter().max_by_key(|(_, n)| *n).map(|(t, _)| *t) else { return };
        self.grant_loyalty_copies(p, 1, Some(best), false);
    }

    /// Called as `source`'s loyalty ability goes on the stack: push the
    /// copies `p`'s grants owe it (CR 707.10 — a copy keeps the original's
    /// targets), and spend the one-shot grants that applied.
    pub(crate) fn copy_loyalty_ability_for_grants(&mut self, p: usize, source: CardId) {
        if self.players[p].loyalty_copy_grants.is_empty() {
            return;
        }
        let types: Vec<PlaneswalkerSubtype> = self
            .battlefield_find(source)
            .map(|c| c.definition.subtypes.planeswalker_subtypes.clone())
            .unwrap_or_default();
        let applies = |g: &LoyaltyCopyGrant| g.subtype.is_none_or(|t| types.contains(&t));
        let copies: u32 =
            self.players[p].loyalty_copy_grants.iter().filter(|g| applies(g)).map(|g| g.copies).sum();
        if copies == 0 {
            return;
        }
        self.players[p].loyalty_copy_grants.retain(|g| !(g.once && applies(g)));
        let Some(item) = self.stack.last().cloned() else { return };
        for _ in 0..copies {
            self.stack.push(item.clone());
        }
    }

    /// Teyo's −2: pick left (mode 0) or right (mode 1) until `ctx.controller`'s
    /// next turn.
    pub(crate) fn choose_attack_direction_until_next_turn(&mut self, ctx: &EffectContext) {
        use crate::decision::{Decision, DecisionAnswer};
        let answer = self.decider.decide(&Decision::ChooseMode {
            source: ctx.source.unwrap_or(CardId(0)),
            num_modes: 2,
            mode_texts: vec!["Left".into(), "Right".into()],
        });
        let step = if matches!(answer, DecisionAnswer::Mode(1)) { -1 } else { 1 };
        self.temporary_attack_direction = Some((step, ctx.controller));
    }

    /// Guff Rewrites History: each permanent's owner shuffles it into their
    /// library; each player who controlled one reveals from the top until a
    /// nonland card, bottoms the rest in a random order and may cast that
    /// card free.
    pub(crate) fn shuffle_in_then_cast_from_top_free(
        &mut self,
        ids: &[CardId],
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        use crate::card::SelectionRequirement as R;
        use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
        let mut controllers: Vec<usize> = Vec::new();
        for &id in ids {
            let Some(c) = self.battlefield_find(id) else { continue };
            let (controller, owner) = (c.controller, c.owner);
            if !controllers.contains(&controller) {
                controllers.push(controller);
            }
            self.move_card_to(id, &ZoneDest::Library { who: PlayerRef::Seat(owner), pos: crate::effect::LibraryPosition::Top }, ctx, events);
            let _ = self.run_effect(&Effect::ShuffleLibrary { who: PlayerRef::Seat(owner) }, ctx, events);
        }
        for p in controllers {
            let pctx = EffectContext { controller: p, ..ctx.clone() };
            let mine = |f: R| Selector::ExiledThisResolution { filter: f.and(R::OwnedByYou) };
            let _ = self.run_effect(
                &Effect::Seq(vec![
                    Effect::ExileTopUntilNonland { who: PlayerRef::You },
                    Effect::CastWithoutPayingImmediate {
                        what: mine(R::Nonland),
                        source_zone: crate::card::Zone::Exile,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    },
                    Effect::Move {
                        what: mine(R::Land),
                        to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Bottom },
                    },
                ]),
                &pctx,
                events,
            );
        }
    }
}
