//! The Pact cycle (Pact of Negation, Slaughter Pact, Summoner's Pact, Pact of
//! the Titan, Intervention Pact) costs nothing now and "pay {X} at the
//! beginning of your next upkeep or lose the game". A pod bot countered a spell
//! with Pact of Negation off three lands and lost at its upkeep (4-seat
//! `--pod-decks 1488,3487,3488,4965,5487,5935 --seed 11073 --first 2`). The
//! bot now casts a pact only when the mana sources it controls could pay every
//! pact it owes plus this one.

use crate::card::CardDefinition;
use crate::effect::Effect;
use crate::game::GameState;
use crate::mana::{Color, ManaCost};

/// The upkeep payment `def`'s own resolution schedules, if it is a pact.
fn pact_cost(def: &CardDefinition) -> Option<&ManaCost> {
    let Effect::Seq(parts) = &def.effect else {
        return None;
    };
    parts.iter().find_map(|e| match e {
        Effect::DelayUntil { body, .. } => match &**body {
            Effect::PayOrLoseGame { mana_cost, .. } => Some(mana_cost),
            _ => None,
        },
        _ => None,
    })
}

/// False for a pact `seat` could not pay at its next upkeep, on top of the
/// pact payments it already owes: one mana per mana source it controls (all
/// untap first), and each colored pip needs that many sources of its color.
pub(super) fn pact_payable(state: &GameState, seat: usize, def: &CardDefinition) -> bool {
    let Some(cost) = pact_cost(def) else {
        return true;
    };
    let owed = state
        .delayed_triggers
        .iter()
        .filter(|d| d.controller == seat)
        .filter_map(|d| match &d.effect {
            Effect::PayOrLoseGame { mana_cost, .. } => Some(mana_cost),
            _ => None,
        });
    let mut need_total = cost.cmc();
    let mut need = Color::ALL.map(|c| cost.pips_of(c));
    for o in owed {
        need_total += o.cmc();
        for (i, c) in Color::ALL.into_iter().enumerate() {
            need[i] += o.pips_of(c);
        }
    }
    let mut have_total = 0;
    let mut have = [0u32; 5];
    for p in state.battlefield.iter().filter(|p| p.controller == seat) {
        // A plain tap ability only: a sacrifice / discard / mana-cost source
        // (Skirk Prospector, a Signet) is no free mana at the upkeep.
        let mut abilities = p
            .definition
            .activated_abilities
            .iter()
            .filter(|a| crate::game::mana_shape::is_countable_mana_ability(a))
            .peekable();
        if abilities.peek().is_none() {
            continue;
        }
        let colors = abilities.fold(crate::mana::ColorSet::empty(), |acc, a| {
            acc.union(crate::game::actions::effect_produced_colors(&a.effect))
        });
        have_total += 1;
        for (i, c) in Color::ALL.into_iter().enumerate() {
            have[i] += u32::from(colors.contains(c));
        }
    }
    have_total >= need_total && need.iter().zip(have).all(|(n, h)| *n <= h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    /// Pact of Negation's {3}{U}{U} needs five sources, two of them blue; a
    /// pact already owed adds its own payment (bug fix: a pod bot cast it
    /// off three lands and lost at its upkeep).
    #[test]
    fn a_pact_waits_for_the_sources_that_pay_it() {
        let mut g = crate::game::multi_player_game(4);
        let pact = catalog::pact_of_negation();
        assert!(pact_payable(&g, 0, &catalog::lightning_bolt()), "not a pact");
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::island());
        }
        assert!(!pact_payable(&g, 0, &pact), "three lands");
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(0, catalog::sol_ring());
        assert!(pact_payable(&g, 0, &pact), "five sources, three blue");
        let mut owes = g.clone();
        let p = owes.add_card_to_hand(0, catalog::pact_of_negation());
        owes.delayed_triggers.push(crate::game::types::DelayedTrigger {
            controller: 0,
            source: p,
            kind: crate::game::types::DelayedKind::YourNextUpkeep,
            effect: Effect::PayOrLoseGame { mana_cost: pact_cost(&pact).unwrap().clone(), life_cost: 0 },
            target: None,
            bound_token: None,
            bound_subject: None,
            fires_once: true,
            expires_after_turn: None,
        });
        assert!(!pact_payable(&owes, 0, &pact), "ten owed off five sources");
        let mut reds = crate::game::multi_player_game(2);
        for _ in 0..5 {
            reds.add_card_to_battlefield(0, catalog::mountain());
        }
        assert!(!pact_payable(&reds, 0, &pact), "no blue source");
    }

    /// Bug fix (precon seat test, The Ninth Doctor seed 6308): Judith cast
    /// Slaughter Pact off a Swamp, a Mountain and Skirk Prospector — whose
    /// "sacrifice a Goblin: add {R}" counted as a third source — and lost at
    /// her upkeep.
    #[test]
    fn a_sacrifice_mana_ability_does_not_pay_a_pact() {
        let mut g = crate::game::multi_player_game(4);
        let pact = catalog::slaughter_pact();
        g.add_card_to_battlefield(0, catalog::swamp());
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(0, catalog::skirk_prospector());
        assert!(!pact_payable(&g, 0, &pact), "two lands and a sacrifice outlet");
        g.add_card_to_battlefield(0, catalog::mountain());
        assert!(pact_payable(&g, 0, &pact), "three lands");
    }
}
