//! Commander-specific effect resolution (CR 903).
//!
//! Its own module rather than another arm body in `effects/mod.rs`, which is
//! 30k lines: the arms here call in, so the Commander rules stay readable and
//! the big match keeps one line per variant.

use crate::card::CardId;
use crate::decision::{Decision, DecisionAnswer};
use crate::game::GameState;

impl GameState {
    /// CR 903.8 / Command Beacon — move one of `seat`'s commanders from the
    /// command zone to their hand.
    ///
    /// The CR 903.9b replacement ("if a commander would be put into its
    /// owner's hand … that player may put it into the command zone instead")
    /// is optional, and a player using this ability always declines it —
    /// applying it would make the ability do nothing — so the move is direct.
    ///
    /// With two commanders (Partner) the controller chooses which one; a
    /// headless seat takes the first, which is what `ChooseCards`'s forced
    /// `min == 1` default does.
    ///
    /// No event: the engine has no generic zone-change event, and the two it
    /// does have for reaching a hand (`PermanentReturnedToHand`) describe a
    /// permanent leaving the battlefield, which this is not.
    pub(crate) fn commander_to_hand_from_command_zone(
        &mut self,
        seat: usize,
        source: Option<CardId>,
    ) {
        let Some(player) = self.players.get(seat) else { return };
        let candidates: Vec<(CardId, String)> = player
            .command
            .iter()
            .filter(|c| player.commanders.contains(&c.id))
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        let chosen = match candidates.len() {
            0 => return,
            1 => candidates[0].0,
            _ => {
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source: source.unwrap_or(CardId(0)),
                    prompt: "Put which commander into your hand?".into(),
                    candidates: candidates.clone(),
                    min: 1,
                    max: 1,
                    eligible: None,
                    value: crate::decision::PickValue::Gain,
                });
                match answer {
                    DecisionAnswer::Cards(ids)
                        if ids.first().is_some_and(|id| {
                            candidates.iter().any(|(c, _)| c == id)
                        }) =>
                    {
                        ids[0]
                    }
                    _ => candidates[0].0,
                }
            }
        };
        let Some(pos) = self.players[seat].command.iter().position(|c| c.id == chosen) else {
            return;
        };
        let card = self.players[seat].command.remove(pos);
        self.players[seat].hand.push(card);
        self.offboard_keyword_grants = true;
    }
}
