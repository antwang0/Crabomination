//! "Look at the top card of [a] library and exile it face down; [a player] may
//! play it for as long as it remains exiled" — Gonti, Night Minister's grant
//! and Gandalf, Goblins' Bane's Wizard-gated Flameshape share this.

use super::EffectContext;
use crate::card::MayPlayDuration;
use crate::effect::ExiledPlaySpend;
use crate::game::GameState;
use crate::game::types::GameEvent;

impl GameState {
    /// Exile the top card of `lib`'s library face down, granting `to` a
    /// may-play permission of `duration` paid per `spend`. Nothing happens on
    /// an empty library.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn exile_top_face_down_granting(
        &mut self,
        lib: usize,
        to: usize,
        spend: ExiledPlaySpend,
        cast_only: bool,
        duration: MayPlayDuration,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let Some(top_id) = self.players[lib].library.first().map(|c| c.id) else {
            return;
        };
        let mut card = Self::take_card(&mut self.players[lib].library, top_id)
            .expect("id read off the top of this library just above");
        card.exiled_with = ctx.source;
        card.face_down = true;
        card.may_play_until = Some(crate::card::MayPlayPermission {
            cast_only,
            locks_further_casts: false,
            one_cast_group: None,
            player: to,
            granted_turn: self.turn_number,
            duration,
            exile_after: false,
            miracle: false,
            pay_life: false,
            bottom_after: false,
            undaunted: false,
        });
        // CR 609.4b — "mana of any type can be spent": paying the
        // mana value as generic is the same set of payments; "of any
        // color" turns only the coloured pips generic.
        card.granted_alt_cast_cost_eot = match spend {
            ExiledPlaySpend::AnyType => {
                Some(crate::mana::ManaCost::new(vec![crate::mana::generic(card.definition.cost.cmc())]))
            }
            ExiledPlaySpend::AnyColor => Some(card.definition.cost.colored_as_generic()),
            // Stamped, not left unset: an unset cost is a free cast.
            ExiledPlaySpend::Own => Some(card.definition.cost.clone()),
        };
        self.exile.push(card);
        events.push(GameEvent::PermanentExiled { card_id: top_id });
        self.note_exiled_from_library(lib, top_id, events);
    }
}
