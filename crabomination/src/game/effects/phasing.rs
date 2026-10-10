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
    pub(crate) fn phase_in_card(&mut self, mut c: crate::card::CardInstance) {
        self.board_instance_keywords |= !c.keyword_counters.is_empty();
        // Phased in, it is held by nothing any more.
        c.phased_out_by = None;
        // CR 702.26i — an Aura / Equipment that phased out directly phases in
        // unattached once its host has left the battlefield (no unattach
        // trigger, 702.26j). A host still phased out is still there.
        if let Some(h) = c.attached_to
            && self.battlefield.find_by_id(h).is_none()
            && !self.phased_out.iter().any(|p| p.id == h)
        {
            c.attached_to = None;
        }
        self.battlefield.push(c);
    }

    /// Phase in every permanent phased out "until" `source` does something.
    pub(crate) fn phase_in_held_by(&mut self, source: CardId, events: &mut Vec<GameEvent>) {
        let mut i = 0;
        let mut phased_in: Vec<CardId> = Vec::new();
        while i < self.phased_out.len() {
            if self.phased_out[i].phased_out_by == Some(source) {
                let mut c = self.phased_out.remove(i);
                // CR 800.4a — a departed owner's card leaves the game rather
                // than phase in: the departure pass exiled a permanent the
                // departing seat controlled but didn't own, and its leaving
                // phased in that seat's own Etali (held by Out of Time) before
                // the phased-out list was cleared (six-seat audit pod, seed
                // 172001). `return_linked_exiles` has the exile twin.
                if !self.players.get(c.owner).is_some_and(|pl| pl.is_alive()) {
                    continue;
                }
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

    /// CR 800.4a — a departed owner's card leaves the game; it does not phase
    /// in when its holder lets go (an Etali held by Out of Time, six-seat
    /// audit pod seed 172001).
    #[test]
    fn cr_800_4a_a_departed_owners_card_does_not_phase_in() {
        let mut g = crate::game::two_player_game();
        let src = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let id = g.add_card_to_battlefield(1, crate::catalog::hill_giant());
        let mut c = g.battlefield.take_by_id(id).unwrap();
        c.phased_out_by = Some(src);
        g.phased_out.push(c);
        g.players[1].eliminated = true;
        g.phase_in_held_by(src, &mut Vec::new());
        assert!(g.battlefield_find(id).is_none());
        assert!(g.phased_out.is_empty());
    }

    /// CR 800.4a — a phased-out permanent the departing seat controlled stops
    /// being theirs before the departure's control reverts run: exiling a
    /// stolen holder phases its held cards in, and a stolen Kate Stewart came
    /// back still under the departed seat (six-seat audit pod seed 172001).
    #[test]
    fn cr_800_4a_a_held_permanent_does_not_phase_in_under_a_departed_seat() {
        let mut g = crate::game::two_player_game();
        // p0's cards, both under p1's control with no effect under it.
        let holder = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let id = g.add_card_to_battlefield(0, crate::catalog::hill_giant());
        g.battlefield_find_mut(holder).unwrap().controller = 1;
        let mut c = g.battlefield.take_by_id(id).unwrap();
        c.controller = 1;
        c.phased_out_by = Some(holder);
        g.phased_out.push(c);
        g.players[1].eliminated = true;
        g.objects_leave_with_player(1, &mut Vec::new());
        assert!(g.battlefield.iter().all(|c| c.controller != 1), "nothing stays under the departed seat");
        assert!(g.phased_out.iter().all(|c| c.controller != 1));
    }
}
