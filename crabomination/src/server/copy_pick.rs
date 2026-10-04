//! "Enter as a copy of [a permanent]" (Clone, Cursed Mirror, Vesuva): which
//! permanent the bot copies. The generic battlefield `Gain` pick ranked the
//! seat's own permanents first and gave legends a premium, so a Clone copied
//! its controller's commander and the legend rule (CR 704.5j) binned one of
//! the two at once. The copy is ours whoever controls the original, so
//! candidates rank by value alone, and a legend we already hold is skipped
//! unless the copy drops the name or the supertype.

use super::bot::{EvalWeights, permanent_value};
use crate::card::{CardId, EntersAsCopy, Supertype};
use crate::decision::DecisionAnswer;
use crate::game::GameState;

/// True when copying `id` would hand `seat` a second legend of a name it
/// already controls (CR 704.5j).
fn copies_held_legend(state: &GameState, seat: usize, copier: CardId, id: CardId) -> bool {
    let Some(cp) = state.computed_permanent(id) else { return false };
    if !cp.supertypes().contains(&Supertype::Legendary) {
        return false;
    }
    let name = cp.def.name;
    state
        .battlefield
        .iter()
        .any(|c| c.controller == seat && c.id != copier && state.computed_permanent_on(c).is_some_and(|o| o.def.name == name))
}

/// The bot's answer to `apply_enters_as_copy`'s `ChooseCards`, or `None` when
/// `source` isn't an entering copier whose candidates are all on the
/// battlefield (graveyard / exile copy sources keep the generic pick).
pub(super) fn decide_copy_source(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    source: CardId,
    candidates: &[(CardId, String)],
    min: u32,
) -> Option<DecisionAnswer> {
    let spec: &EntersAsCopy = state.battlefield_find(source)?.definition.enters_as_copy.as_ref()?;
    if spec.from_graveyards
        || spec.from_exile_with_counter.is_some()
        || !candidates.iter().all(|(id, _)| state.battlefield_find(*id).is_some())
    {
        return None;
    }
    let legend_safe = spec.non_legendary || spec.keep_name;
    let mut ranked: Vec<(CardId, bool, i32)> = candidates
        .iter()
        .map(|&(id, _)| {
            let clash = !legend_safe && copies_held_legend(state, seat, source, id);
            (id, clash, permanent_value(state, id, w))
        })
        .collect();
    // Stable: equal values keep the engine's offer order (highest power first).
    ranked.sort_by(|a, b| a.1.cmp(&b.1).then(b.2.cmp(&a.2)));
    let pick = ranked.first().filter(|(_, clash, _)| !*clash || min > 0).map(|(id, ..)| *id);
    Some(DecisionAnswer::Cards(pick.into_iter().collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    fn pick(g: &GameState, copier: CardId, min: u32) -> Vec<CardId> {
        let candidates: Vec<(CardId, String)> =
            g.battlefield.iter().filter(|c| c.id != copier && g.computed_is_creature(c)).map(|c| (c.id, String::new())).collect();
        match decide_copy_source(g, 0, &EvalWeights::default(), copier, &candidates, min) {
            Some(DecisionAnswer::Cards(v)) => v,
            other => panic!("not a copy pick: {other:?}"),
        }
    }

    /// CR 704.5j — a copy of a legend its controller already holds loses one
    /// of the two to the legend rule, so the bot copies something else; a
    /// legend of another name, or a copy that "isn't legendary", is fine.
    #[test]
    fn copier_skips_a_legend_it_already_controls() {
        let mut g = crate::game::multi_player_game(4);
        let thalia = g.add_card_to_battlefield(0, catalog::thalia_guardian_of_thraben());
        let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mirror = g.add_card_to_battlefield(0, catalog::cursed_mirror());
        assert_eq!(pick(&g, mirror, 0), vec![bears], "an opponent's Bears, not a second Thalia");
        let spark = g.add_card_to_battlefield(0, catalog::spark_double());
        assert_eq!(pick(&g, spark, 1), vec![thalia], "Spark Double's copy isn't legendary");
        // A legend of a name the seat doesn't hold is no clash, whoever controls it.
        let mut g = crate::game::multi_player_game(3);
        g.add_card_to_battlefield(0, catalog::thalia_guardian_of_thraben());
        let ragavan = g.add_card_to_battlefield(2, catalog::ragavan_nimble_pilferer());
        let mirror = g.add_card_to_battlefield(0, catalog::cursed_mirror());
        assert_eq!(pick(&g, mirror, 0), vec![ragavan]);
    }

    /// A "you may" copier declines rather than copy only a held legend; a
    /// forced one still answers.
    #[test]
    fn optional_copier_declines_when_every_pick_clashes() {
        let mut g = crate::game::multi_player_game(3);
        let thalia = g.add_card_to_battlefield(0, catalog::thalia_guardian_of_thraben());
        let mirror = g.add_card_to_battlefield(0, catalog::cursed_mirror());
        assert!(pick(&g, mirror, 0).is_empty());
        assert_eq!(pick(&g, mirror, 1), vec![thalia]);
    }
}
