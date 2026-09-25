//! Two library-top deployments from Enhanced Evolution: Animist's
//! Awakening's "reveal the top X, put all land cards onto the battlefield
//! tapped, the rest on the bottom in a random order", and Nissa, Steward of
//! Elements' "look at the top card; if it's a land card or a creature card
//! with mana value at most [her loyalty], you may put it onto the
//! battlefield".

use crate::card::CardId;
use crate::effect::{Effect, LibraryPosition, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::RevealTopPutLandsRestBottomRandom`.
    pub(super) fn reveal_top_put_lands_rest_bottom_random(
        &mut self,
        count: &Value,
        tapped: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let n = self.evaluate_value(count, ctx).max(0) as usize;
        if n == 0 {
            return Ok(());
        }
        let mut lands = Vec::new();
        let mut rest: Vec<CardId> = Vec::new();
        for c in self.players[p].library.iter().take(n) {
            events.push(GameEvent::TopCardRevealed { player: p, card_name: c.definition.name, is_land: c.definition.is_land() });
            if c.definition.is_land() { lands.push(c.id) } else { rest.push(c.id) }
        }
        for id in lands {
            self.move_card_to(id, &ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped }, ctx, events);
        }
        use rand::seq::SliceRandom;
        rest.shuffle(&mut self.rng.draw());
        for id in rest {
            self.move_card_to(id, &ZoneDest::Library { who: PlayerRef::Seat(p), pos: LibraryPosition::Bottom }, ctx, events);
        }
        Ok(())
    }

    /// `Effect::LookTopMayPutLandOrCreatureMvAtMost`.
    pub(super) fn look_top_may_put_land_or_creature_mv_at_most(
        &mut self,
        max_mv: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let cap = self.evaluate_value(max_mv, ctx).max(0) as u32;
        let Some((id, fits)) = self.players[p].library.first().map(|c| {
            let d = &c.definition;
            (c.id, d.is_land() || (d.is_creature() && c.printed_cmc() <= cap))
        }) else {
            return Ok(());
        };
        if !fits {
            return Ok(());
        }
        // The "you may" rides `MayDo`, whose suspend-and-rerun comes back
        // through here with the same top card.
        let c = EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
        self.run_effect(
            &Effect::MayDo {
                description: "Put the top card of your library onto the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
            &c,
            events,
        )
    }
}
