//! Commander games only: casts paid by convoke / improvise / delve where
//! `cast_candidates` never builds the payment — a commander cast from the
//! command zone with its own helper keyword (Kasla, Tasigur, Bennie Bracks),
//! and a spell no mana can pay (Hogaak, Arisen Necropolis, from hand,
//! graveyard or command zone). Taken in the seat's own main phase with an
//! empty stack when no plain cast of the card is legal; delve first (a
//! graveyard card is free), then summoning-sick, low-power creatures.

use crate::card::{CardId, CardInstance, Keyword, KeywordSlice};
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};
use crate::mana::ManaSymbol;

use super::bot::BotStep;

pub(super) fn pick_helper_cast(state: &GameState, seat: usize) -> Option<BotStep> {
    let pl = &state.players[seat];
    if pl.commanders.is_empty()
        || state.active_player_idx != seat
        || !matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
        || !state.stack.is_empty()
    {
        return None;
    }
    let helpers = |c: &CardInstance| {
        let kw = &c.definition.keywords;
        kw.has_kw(&Keyword::Convoke) || kw.has_kw(&Keyword::Improvise) || kw.has_kw(&Keyword::Delve)
    };
    let cards = pl
        .command
        .iter()
        .filter(|c| helpers(c))
        .chain(pl.hand.iter().chain(pl.graveyard.iter()).filter(|c| c.definition.no_mana_spend));
    for card in cards {
        let in_command = pl.command.iter().any(|c| c.id == card.id);
        if in_command && !card.definition.no_mana_spend && GameState::would_accept_on(state, plain(card.id)) {
            continue;
        }
        if card.definition.effect.requires_target() {
            continue;
        }
        let tax = if in_command { state.commander_tax_for(card).0 } else { 0 };
        let Some((convoke, delve)) = payment(state, seat, card, tax) else { continue };
        // Fewest helpers first, but a dry run is a state clone, so at most four.
        let (c, d) = (convoke.len(), delve.len());
        let mut tries: Vec<(usize, usize)> =
            if card.definition.no_mana_spend { vec![(c, d)] } else { vec![(0, d), (c / 3, d), (2 * c / 3, d), (c, d)] };
        tries.dedup();
        for (n, d) in tries {
            let action = GameAction::CastSpellConvokeDelve {
                card_id: card.id,
                target: None,
                additional_targets: Vec::new(),
                mode: None,
                x_value: None,
                convoke_creatures: convoke[..n].to_vec(),
                delve_cards: delve[..d].to_vec(),
            };
            if let Some(g) = GameState::accept_on(state, action.clone()) {
                return Some(BotStep { action, settled: Some(Box::new(g)) });
            }
        }
    }
    None
}

fn plain(card_id: CardId) -> GameAction {
    GameAction::CastFromCommandZone {
        card_id,
        target: None,
        additional_targets: Vec::new(),
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    }
}

/// The most `card`'s keywords let it pay with: convoke / improvise helpers
/// (one per colored pip they can cover first, then generic) and delve cards,
/// enough between them for its whole cost plus `tax` — `None` if a spell that
/// can't spend mana can't be paid in full.
fn payment(state: &GameState, seat: usize, card: &CardInstance, tax: u32) -> Option<(Vec<CardId>, Vec<CardId>)> {
    let kw = &card.definition.keywords;
    let (convoke, improvise, delve) =
        (kw.has_kw(&Keyword::Convoke), kw.has_kw(&Keyword::Improvise), kw.has_kw(&Keyword::Delve));
    let mut ranked: Vec<(bool, i32, CardId, Vec<crate::mana::Color>)> = state
        .battlefield
        .iter()
        .filter(|h| {
            h.controller == seat
                && !h.tapped
                && ((convoke && state.computed_is_creature(h))
                    || (improvise && state.computed_has_card_type(h, crate::card::CardType::Artifact)))
        })
        .map(|h| {
            let colors = state.computed_permanent(h.id).map(|cp| cp.colors.to_vec()).unwrap_or_default();
            (!h.summoning_sick, h.power(), h.id, colors)
        })
        .collect();
    ranked.sort_by_key(|(healthy, pow, id, _)| (*healthy, *pow, *id));
    let mut taps: Vec<CardId> = Vec::new();
    let mut generic = tax;
    for sym in &card.definition.cost.symbols {
        let fits: Option<Box<dyn Fn(&[crate::mana::Color]) -> bool>> = match *sym {
            ManaSymbol::Colored(c) if convoke => Some(Box::new(move |cs| cs.contains(&c))),
            ManaSymbol::Hybrid(a, b) if convoke => Some(Box::new(move |cs| cs.contains(&a) || cs.contains(&b))),
            ManaSymbol::Generic(n) => {
                generic += n;
                None
            }
            _ if card.definition.no_mana_spend => return None,
            _ => None,
        };
        if let Some(fits) = fits {
            match ranked.iter().position(|(_, _, _, cs)| fits(cs)) {
                Some(i) => taps.push(ranked.remove(i).2),
                None if card.definition.no_mana_spend => return None,
                None => {}
            }
        }
    }
    let delved: Vec<CardId> = if delve {
        state.players[seat].graveyard.iter().filter(|c| c.id != card.id).map(|c| c.id).take(generic as usize).collect()
    } else {
        Vec::new()
    };
    let rest = generic as usize - delved.len();
    taps.extend(ranked.iter().take(rest).map(|r| r.2));
    let short = rest.saturating_sub(ranked.len());
    if card.definition.no_mana_spend && short > 0 {
        return None;
    }
    Some((taps, delved))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hogaak as a commander: the bot pays {5}{B/G}{B/G} with two black
    /// creatures and five delved cards, and the cast is legal (CR 903.8 +
    /// 702.51 / 702.66); with one black creature short it offers nothing.
    #[test]
    fn the_bot_convokes_and_delves_hogaak_from_the_command_zone() {
        let mut g = crate::game::game_with_format(crate::format::Format::Commander, 3);
        let hogaak = g.seat_commanders(0, vec![crate::catalog::hogaak_arisen_necropolis()])[0];
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let knight = g.add_card_to_battlefield(0, crate::catalog::vampire_nighthawk());
        for _ in 0..5 {
            g.add_card_to_graveyard(0, crate::catalog::island());
        }
        assert!(pick_helper_cast(&g, 0).is_none(), "one hybrid pip has no creature");
        let second = g.add_card_to_battlefield(0, crate::catalog::vampire_nighthawk());
        let step = pick_helper_cast(&g, 0).expect("payable");
        let GameAction::CastSpellConvokeDelve { card_id, convoke_creatures, delve_cards, .. } = &step.action else {
            panic!("{:?}", step.action)
        };
        assert_eq!((*card_id, convoke_creatures.len(), delve_cards.len()), (hogaak, 2, 5));
        assert!(convoke_creatures.contains(&knight) && convoke_creatures.contains(&second));
        assert!(step.settled.is_some_and(|s| s.stack.iter().any(|i| matches!(i,
            crate::game::types::StackItem::Spell { card, .. } if card.id == hogaak))));
    }

    /// CR 903.9a — the owner's "may": the bot keeps a commander that went on
    /// an Adventure in exile (its creature is castable there, untaxed) and
    /// sends any other exiled commander home.
    #[test]
    fn the_bot_keeps_an_adventured_commander_in_exile() {
        use crate::decision::{Decision, DecisionAnswer};
        let mut g = crate::game::game_with_format(crate::format::Format::Commander, 3);
        let beorn = g.seat_commanders(0, vec![crate::catalog::beorn_reluctant_host()])[0];
        let mut card = g.players[0].command.remove(0);
        card.on_adventure = true;
        g.exile.push(card);
        let ask = Decision::CommanderRedirect { commander: beorn, would_be: crate::card::Zone::Exile };
        let w = super::super::bot::EvalWeights::default();
        let answer = |g: &GameState| super::super::bot::decide_pending_policy(g, 0, &w, &ask, false);
        assert!(matches!(answer(&g), DecisionAnswer::Bool(false)));
        g.exile.iter_mut().for_each(|c| c.on_adventure = false);
        assert!(matches!(answer(&g), DecisionAnswer::Bool(true)));
    }
}
