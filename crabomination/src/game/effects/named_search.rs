//! Sphinx Ambassador: search a player's library for a card, that player names a
//! card, and a creature card that doesn't have the name may join your side.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::{OptionalKind, PickValue};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// "Search that player's library for a card, then that player chooses a
    /// card name. If you searched for a creature card that doesn't have that
    /// name, you may put it onto the battlefield under your control. Then that
    /// player shuffles." The searcher's headless pick is the greatest-mana-value
    /// creature card; the named player chooses among the creature names in
    /// their library (no other name can matter), offered in a scrambled order
    /// so a headless guess is a guess.
    pub(super) fn search_against_named_card(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let Some(them) = self.resolve_player(who, ctx).filter(|&q| self.players.get(q).is_some()) else {
            self.clear_answer_log();
            return Ok(());
        };
        let source = ctx.source.unwrap_or(CardId(0));
        let library: Vec<(CardId, String, bool, u32)> = self.players[them]
            .library
            .iter()
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.is_creature(), c.definition.cost.cmc()))
            .collect();
        if library.is_empty() {
            self.clear_answer_log();
            return Ok(());
        }
        let mut cursor = 0;
        let auto = library
            .iter()
            .filter(|c| c.2)
            .max_by_key(|c| (c.3, std::cmp::Reverse(c.0.0)))
            .unwrap_or(&library[0])
            .0;
        // CR 701.19b — a search for "a card" with no quality finds one.
        let Some(picked) = self.ask_seat_cards_logged(
            &mut cursor,
            me,
            "Search that player's library for a card".into(),
            source,
            library.iter().map(|c| (c.0, c.1.clone())).collect(),
            1,
            1,
            PickValue::Gain,
            effect,
            vec![auto],
        ) else {
            return Ok(());
        };
        let found = picked.first().copied().unwrap_or(auto);
        let mut names: Vec<String> = library.iter().filter(|c| c.2).map(|c| c.1.clone()).collect();
        names.sort();
        names.dedup();
        // A stable scramble (FNV-1a keyed on the turn and the source) rather
        // than the RNG: a suspend re-runs this arm, and the logged index has
        // to name the same card the second time.
        let salt = (u64::from(self.turn_number) << 32) ^ u64::from(source.0);
        let key = |n: &str| {
            n.bytes().fold(0xcbf2_9ce4_8422_2325_u64 ^ salt, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3))
        };
        names.sort_by_cached_key(|n| key(n));
        let named = if names.is_empty() {
            None
        } else {
            let Some(i) =
                self.ask_seat_option(&mut cursor, them, "Choose a card name".into(), source, names.clone(), effect)
            else {
                return Ok(());
            };
            names.get(i).cloned()
        };
        let card = library.iter().find(|c| c.0 == found);
        let steal = card.is_some_and(|c| c.2 && named.as_deref() != Some(c.1.as_str()));
        if steal {
            let Some(yes) = self.ask_seat_bool(
                &mut cursor,
                me,
                "Put the creature card onto the battlefield under your control?".into(),
                source,
                effect,
                OptionalKind::MayBody,
            ) else {
                return Ok(());
            };
            if yes {
                self.move_card_to(
                    found,
                    &ZoneDest::Battlefield { controller: PlayerRef::Seat(me), tapped: false },
                    ctx,
                    events,
                );
            }
        }
        self.clear_answer_log();
        self.shuffle_library(them, events);
        Ok(())
    }
}
