//! "Choose a letter" (Katerina of Myra's Marvels): the letter is stored as
//! its index, A = 0 … Z = 25, in the source's `chosen_number`.

use crate::game::GameState;

/// The index of `name`'s first letter, ignoring anything before it
/// ("\"Ach! Hans, Run!\"" begins with A). `None` for a name with no letter.
pub(crate) fn initial(name: &str) -> Option<u32> {
    name.chars().find(|c| c.is_ascii_alphabetic()).map(|c| u32::from(c.to_ascii_uppercase() as u8 - b'A'))
}

impl GameState {
    /// The commonest initial among `seat`'s nonland cards in library, hand
    /// and graveyard — the headless pick. Ties go to the earlier letter, so
    /// the pick is seeded by the deck alone.
    pub(crate) fn commonest_spell_initial(&self, seat: usize) -> u32 {
        let mut counts = [0u32; 26];
        let Some(p) = self.players.get(seat) else { return 0 };
        for c in p.library.iter().chain(p.hand.iter()).chain(p.graveyard.iter()) {
            if !c.definition.is_land()
                && let Some(i) = initial(c.definition.name)
            {
                counts[i as usize] += 1;
            }
        }
        (0..26u32).max_by_key(|&i| (counts[i as usize], std::cmp::Reverse(i))).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::initial;

    #[test]
    fn initials_skip_punctuation() {
        assert_eq!(initial("\"Ach! Hans, Run!\""), Some(0));
        assert_eq!(initial("Zur's Weirding"), Some(25));
        assert_eq!(initial("1996 World Champion"), Some(22));
    }
}
