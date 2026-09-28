//! Protective keyword grants as responses — "{1}{G}{U}: creatures you control
//! gain shroud" (Aerie Mystics), "discard a card: hexproof" (Prognostic
//! Sphinx), "{5}: permanents you control gain indestructible" (Soul of New
//! Phyrexia). A 183-deck `--card-census` never activated them: the grant
//! scores as nothing until a removal spell is on the stack, and no response
//! path offered them. A pod bot now answers an opponent's removal on top of
//! the stack with a grant that stops it — hexproof/shroud against a targeted
//! spell or ability (CR 702.11b / 702.18a: the target becomes illegal),
//! indestructible against destroy or damage (CR 702.12b), a sweep included.
//! Failing a grant, lethal removal is answered with a "when that creature dies
//! this turn, return it to its owner's hand" ability (Together Forever — never
//! activated in eight decks, census seed 3100001).
//! Commander games only, so two-player play is unchanged.

use crate::card::{CardId, Keyword};
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, StackItem, Target};

/// How the removal in `e` reaches its victims.
#[derive(Clone, Copy, PartialEq)]
enum Threat {
    /// Destroy or damage aimed at a target slot: any of the three keywords.
    TargetedLethal,
    /// Exile / bounce / steal aimed at a target slot: hexproof or shroud.
    TargetedOther,
    /// "Destroy all …": indestructible only.
    Sweep,
}

fn threat(e: &Effect) -> Option<Threat> {
    let targeted = |w: &Selector| matches!(w, Selector::Target(_) | Selector::TargetFiltered { .. });
    match e {
        Effect::Destroy { what } if targeted(what) => Some(Threat::TargetedLethal),
        Effect::Destroy { what: Selector::EachPermanent(_) } => Some(Threat::Sweep),
        Effect::DealDamage { to, .. } if targeted(to) => Some(Threat::TargetedLethal),
        Effect::Exile { what } | Effect::Move { what, .. } | Effect::GainControl { what, .. } if targeted(what) => {
            Some(Threat::TargetedOther)
        }
        Effect::Seq(v) => v.iter().find_map(threat),
        Effect::ApplyToTargets { effect, .. } => threat(effect),
        _ => None,
    }
}

/// The protective keyword a grant in `e` gives, and whom: itself, its target
/// slot, or a board-wide set.
fn shield(e: &Effect) -> Option<(Keyword, &Selector)> {
    let protective = |k: &Keyword| matches!(k, Keyword::Hexproof | Keyword::Shroud | Keyword::Indestructible);
    match e {
        Effect::GrantKeyword { what, keyword, .. } if protective(keyword) => Some((keyword.clone(), what)),
        Effect::GrantKeywords { what, keywords, .. } => {
            keywords.iter().find(|k| protective(k)).map(|k| (k.clone(), what))
        }
        Effect::Seq(v) => v.iter().find_map(shield),
        _ => None,
    }
}

/// A protective activation that saves one of `seat`'s permanents from the
/// opponent's removal on top of the stack, if one is accepted.
pub(super) fn pick_keyword_shield(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    let (effect, target, owner) = match state.stack.last()? {
        StackItem::Spell { card, target, caster, .. } => (&card.definition.effect, target.as_ref(), *caster),
        StackItem::Trigger { effect, target, controller, .. } => (&**effect, target.as_ref(), *controller),
    };
    if state.same_team(owner, seat) {
        return None;
    }
    let threat = threat(effect)?;
    let victim: Option<CardId> = match (threat, target) {
        (Threat::Sweep, _) => None,
        (_, Some(Target::Permanent(id))) if state.battlefield_find(*id).is_some_and(|c| c.controller == seat) => Some(*id),
        _ => return None,
    };
    let fits = |k: &Keyword| match threat {
        Threat::TargetedLethal => true,
        Threat::TargetedOther => matches!(k, Keyword::Hexproof | Keyword::Shroud),
        Threat::Sweep => matches!(k, Keyword::Indestructible),
    };
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in c.definition.activated_abilities.iter().enumerate() {
            let Some((kw, who)) = shield(&ab.effect) else { continue };
            if !fits(&kw) {
                continue;
            }
            // Whom it covers: the source only, a target (the victim), or a
            // board-wide set — a sweep needs the board-wide form.
            let aim = match (who, victim) {
                (Selector::This, Some(v)) if v == c.id => None,
                (Selector::Target(_) | Selector::TargetFiltered { .. }, Some(v)) => Some(Target::Permanent(v)),
                (Selector::EachPermanent(_), _) => None,
                _ => continue,
            };
            let action = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: idx,
                target: aim,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    if threat == Threat::TargetedOther {
        return None;
    }
    // A sweep: insure the biggest creature the ability will take.
    let mut victims: Vec<(i32, CardId)> = match victim {
        Some(v) => vec![(0, v)],
        None => state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.is_creature())
            .map(|c| (state.computed_permanent(c.id).map_or(0, |cp| cp.power), c.id))
            .collect(),
    };
    victims.sort_by_key(|&(power, id)| (std::cmp::Reverse(power), id));
    let insurers: Vec<(CardId, usize)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .filter(|(_, ab)| matches!(ab.effect, Effect::WhenTargetDiesThisTurn { slot: 0, .. }))
                .map(move |(i, _)| (c.id, i))
        })
        .collect();
    victims.iter().find_map(|&(_, v)| {
        insurers.iter().find_map(|&(card_id, ability_index)| {
            let action = GameAction::ActivateAbility {
                card_id,
                ability_index,
                target: Some(Target::Permanent(v)),
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Aerie Mystics gives its controller's creatures shroud in answer to a
    /// Murder aimed at one of them; with no mana it can't.
    #[test]
    fn aerie_mystics_answers_murder_with_shroud() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        let mystics = g.add_card_to_battlefield(0, crate::catalog::aerie_mystics());
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let murder = g.add_card_to_hand(1, crate::catalog::murder());
        g.players[1].mana_pool.add(Color::Black, 3);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: murder, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("Murder");
        g.priority.player_with_priority = 0;
        assert!(pick_keyword_shield(&g, 0).is_none(), "no mana");
        for c in [Color::Green, Color::Blue] {
            g.players[0].mana_pool.add(c, 1);
        }
        g.players[0].mana_pool.add_colorless(1);
        let a = pick_keyword_shield(&g, 0).expect("shroud");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, .. } if card_id == mystics));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "Murder lost its target");
    }

    /// Together Forever insures a countered creature against Murder; an
    /// uncountered one is outside its filter.
    #[test]
    fn together_forever_insures_a_countered_creature_against_murder() {
        for (counter, fires) in [(true, true), (false, false)] {
            let mut g = crate::game::multi_player_game(3);
            g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
            g.active_player_idx = 1;
            g.step = TurnStep::PreCombatMain;
            let tf = g.add_card_to_battlefield(0, crate::catalog::together_forever());
            let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
            if counter {
                g.battlefield_find_mut(bear).unwrap().add_counters(crate::card::CounterType::PlusOnePlusOne, 1);
            }
            let murder = g.add_card_to_hand(1, crate::catalog::murder());
            g.players[1].mana_pool.add(Color::Black, 3);
            g.priority.player_with_priority = 1;
            g.perform_action(GameAction::CastSpell {
                card_id: murder, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
            })
            .expect("Murder");
            g.priority.player_with_priority = 0;
            g.players[0].mana_pool.add_colorless(1);
            let picked = pick_keyword_shield(&g, 0);
            assert_eq!(
                matches!(picked, Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                    if card_id == tf && t == bear),
                fires,
                "counter {counter}"
            );
        }
    }
}
