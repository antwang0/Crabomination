//! `SelectionRequirement::TargetsTakeDistinctTypeSlots` — "up to one target
//! artifact, up to one target creature, … and/or up to one target land" as
//! one target group (Chaotic Transformation): the targets chosen so far plus
//! the candidate must take distinct card types from the list, each one a type
//! it has (CR 601.2c, 115.3). At most a handful of objects and types, so a
//! backtracking matching is plenty.

use super::GameState;
use super::types::Target;
use crate::card::CardType;

impl GameState {
    pub(crate) fn targets_take_distinct_type_slots(
        &self,
        types: &[CardType],
        target: &Target,
        slots: &[Option<Target>],
    ) -> bool {
        let mut objects: Vec<crate::card::CardId> = Vec::new();
        for t in slots.iter().flatten().chain(std::iter::once(target)) {
            if let Target::Permanent(id) = t
                && !objects.contains(id)
            {
                objects.push(*id);
            }
        }
        let has: Vec<Vec<usize>> = objects
            .iter()
            .map(|id| {
                let own: Vec<CardType> = self
                    .computed_permanent(*id)
                    .map(|cp| cp.card_types().to_vec())
                    .or_else(|| self.find_card_anywhere(*id).map(|c| c.definition.card_types.clone()))
                    .unwrap_or_default();
                types.iter().enumerate().filter(|(_, t)| own.contains(t)).map(|(i, _)| i).collect()
            })
            .collect();
        fn assign(i: usize, has: &[Vec<usize>], used: &mut Vec<bool>) -> bool {
            let Some(options) = has.get(i) else { return true };
            for &k in options {
                if !used[k] {
                    used[k] = true;
                    if assign(i + 1, has, used) {
                        return true;
                    }
                    used[k] = false;
                }
            }
            false
        }
        objects.len() <= types.len() && assign(0, &has, &mut vec![false; types.len()])
    }
}
