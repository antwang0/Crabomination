//! CR 115.1 / 608.2 — "return target spell or creature to its owner's hand"
//! (Brutal Expulsion, Venser's kin): a `Move` whose target is a *spell* has
//! to lift it off the stack. `move_card_to` only walks the battlefield and
//! the card zones, so a targeted spell stayed put and resolved anyway.
//!
//! The lift goes through `MoveSpellToZone`, the one path that already
//! handles flashback (CR 702.34a) and the commander redirect (CR 903.9b).

use crate::card::CardId;
use crate::effect::{CounteredSpellZone, Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{StackItem, Target};
use crate::game::{GameEvent, GameState};

impl GameState {
    /// Lift `cid` off the stack toward `to` when it is a spell there and `to`
    /// names a zone a spell can be put into. `true` when the move was handled
    /// here (the caller skips its own); `false` for anything not on the stack.
    pub(crate) fn move_stack_spell(
        &mut self,
        cid: CardId,
        to: &ZoneDest,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        if !self.stack.iter().any(|si| matches!(si, StackItem::Spell { card, .. } if card.id == cid)) {
            return false;
        }
        let zone = match to {
            ZoneDest::Hand(PlayerRef::OwnerOfMoved | PlayerRef::OwnerOf(_)) => CounteredSpellZone::OwnerHand,
            ZoneDest::Exile => CounteredSpellZone::Exile,
            ZoneDest::Library { who: PlayerRef::OwnerOfMoved | PlayerRef::OwnerOf(_), pos: LibraryPosition::Top } => {
                CounteredSpellZone::OwnerLibraryTop
            }
            ZoneDest::Library { who: PlayerRef::OwnerOfMoved | PlayerRef::OwnerOf(_), pos: LibraryPosition::Bottom } => {
                CounteredSpellZone::OwnerLibraryBottom
            }
            _ => return false,
        };
        let mut sub = ctx.clone();
        sub.targets = vec![Target::Permanent(cid)];
        let lift = Effect::MoveSpellToZone { what: Selector::Target(0), zone };
        let _ = self.run_effect(&lift, &sub, events);
        true
    }
}
