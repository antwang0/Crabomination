//! CR 702.26 — permanents phased out and held by a source: they phase in
//! when it leaves (Out of Time) or on its own trigger (The Pandorica's
//! untap).

use crate::card::CardId;
use crate::game::GameState;
use crate::game::types::GameEvent;

impl GameState {
    /// CR 702.26 — a phased-out permanent returns to the battlefield with its
    /// counters, so a keyword counter re-arms the instance-keyword gate. The
    /// one way back for every phase-in path.
    pub(crate) fn phase_in_card(&mut self, c: crate::card::CardInstance) {
        self.board_instance_keywords |= !c.keyword_counters.is_empty();
        self.battlefield.push(c);
    }

    /// Phase in every permanent phased out "until" `source` does something.
    pub(crate) fn phase_in_held_by(&mut self, source: CardId, events: &mut Vec<GameEvent>) {
        let mut i = 0;
        let mut phased_in: Vec<CardId> = Vec::new();
        while i < self.phased_out.len() {
            if self.phased_out[i].phased_out_by == Some(source) {
                let mut c = self.phased_out.remove(i);
                c.phased_out_by = None;
                phased_in.push(c.id);
                self.phase_in_card(c);
            } else {
                i += 1;
            }
        }
        for card_id in phased_in {
            events.push(GameEvent::PermanentPhasedIn { card_id });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::card::Keyword;

    /// CR 702.26 — a permanent phases in with its counters. A keyword counter
    /// that phased out across a cleanup (which recounts the instance-keyword
    /// gate over the battlefield only) came back through The Pandorica's and
    /// `SwapPhasedState` (Time and Tide) paths without re-arming the gate: a flying
    /// counter on a Pandorica-held Steelbane Hydra (pod seed 5306015).
    #[test]
    fn cr_702_26_a_held_permanent_phases_in_with_its_keyword_counters() {
        let mut g = crate::game::two_player_game();
        let src = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let id = g.add_card_to_battlefield(1, crate::catalog::hill_giant());
        let pos = g.battlefield.iter().position(|c| c.id == id).unwrap();
        let mut c = g.battlefield.remove(pos);
        c.keyword_counters.add(Keyword::Flying, 1);
        c.phased_out_by = Some(src);
        g.phased_out.push(c);
        g.board_instance_keywords = false; // what the cleanup recount leaves
        g.phase_in_held_by(src, &mut Vec::new());
        assert!(g.battlefield_find(id).is_some());
        assert!(g.board_instance_keywords, "the flying counter re-arms the gate");
    }
}
