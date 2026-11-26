//! Protective keyword grants as responses — "{1}{G}{U}: creatures you control
//! gain shroud" (Aerie Mystics), "discard a card: hexproof" (Prognostic
//! Sphinx), "{5}: permanents you control gain indestructible" (Soul of New
//! Phyrexia). The grant scores as nothing until removal is on the stack and
//! no response path offered it, so a census never saw one activated.
//!
//! A pod bot answers an opponent's removal on top of the stack with:
//! - a grant that stops it: hexproof/shroud against a targeted spell or
//!   ability (CR 702.11b / 702.18a), indestructible against destroy or
//!   damage (CR 702.12b), a sweep included;
//! - against a coloured one, "protection from the colour of your choice"
//!   (Mother of Runes), naming the removal's colour;
//! - failing a grant, for lethal removal a "when that creature dies this
//!   turn, return it to its owner's hand" ability (Together Forever), and
//!   for any targeted removal an exile its controller may take (Livio's
//!   aegis, whose may was always declined).
//!
//! Commander games only, so two-player play is unchanged — except under the
//! `RF_DUEL_SHIELD` rating fix, which opens the same picker to a seat with
//! no commander, restricted to removal that actually takes the creature
//! (damage short of lethal is let through) and never spending a discard on
//! a token.

use crate::card::{CardId, Keyword};
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
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

/// "Its controller may exile it" over the ability's target (Livio,
/// Oathsworn Sentinel's aegis exile): the may's description.
fn shelter_exile(e: &Effect) -> Option<&str> {
    let Effect::MayDoBy { who: PlayerRef::ControllerOf(_), description, body } = e else { return None };
    let exiles_target = |e: &Effect| {
        matches!(e, Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile } | Effect::Exile { what: Selector::Target(0) })
    };
    let first = match &**body {
        Effect::Seq(v) => v.first()?,
        e => e,
    };
    exiles_target(first).then_some(description.as_str())
}

/// The answer to a [`shelter_exile`] may asked of `seat` by `source`'s
/// ability: take it while an opposing spell or ability on the stack aims at
/// one of `seat`'s permanents — the bot activates it only as that answer.
pub(super) fn shelter_exile_answer(state: &GameState, seat: usize, source: CardId, description: &str) -> Option<bool> {
    let card = state.battlefield_find(source)?;
    card.definition.activated_abilities.iter().find(|ab| shelter_exile(&ab.effect) == Some(description))?;
    let mine: Vec<CardId> = state.battlefield.iter().filter(|c| c.controller == seat).map(|c| c.id).collect();
    Some(state.stack_threat(seat, &mine).is_some())
}

/// "Return all exiled cards [matching] to the battlefield under their owners'
/// control" (Livio's second ability, the way back from [`shelter_exile`]):
/// the filter, for an untargeted `Move` (or `ForEach`) over exile.
pub(super) fn exile_mass_return(e: &Effect) -> Option<&crate::card::SelectionRequirement> {
    match e {
        Effect::Move {
            what: Selector::EachMatching { zone: crate::effect::ZoneRef::Exile, filter },
            to: ZoneDest::Battlefield { .. },
        } => Some(filter),
        Effect::ForEach { selector: Selector::EachMatching { zone: crate::effect::ZoneRef::Exile, filter }, body } => {
            matches!(&**body, Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Battlefield { .. } })
                .then_some(filter)
        }
        _ => None,
    }
}

/// An [`exile_mass_return`] is worth it when it brings back at least one of
/// `seat`'s cards and no more of anyone else's.
pub(super) fn exile_mass_return_pays(state: &GameState, seat: usize, e: &Effect) -> bool {
    let Some(filter) = exile_mass_return(e) else { return false };
    let (mut mine, mut theirs) = (0usize, 0usize);
    for c in state.exile.iter().filter(|c| state.evaluate_requirement_on_card(filter, c, seat)) {
        if c.owner == seat {
            mine += 1;
        } else {
            theirs += 1;
        }
    }
    mine > 0 && mine >= theirs
}

/// The constant damage the first removal leaf of `e` deals to its target,
/// when that leaf is damage — [`threat`]'s walk, so the two agree on which
/// leaf is the threat. `None` for destroy / exile, or an amount only known
/// at resolution.
fn threat_damage(e: &Effect) -> Option<i32> {
    match e {
        Effect::DealDamage { amount: crate::effect::Value::Const(n), .. } if threat(e).is_some() => Some(*n),
        Effect::Seq(v) => v.iter().find(|e| threat(e).is_some()).and_then(threat_damage),
        Effect::ApplyToTargets { effect, .. } => threat_damage(effect),
        _ => None,
    }
}

/// A protective activation that saves one of `seat`'s permanents from the
/// opponent's removal on top of the stack, if one is accepted.
///
/// `duel` (`RF_DUEL_SHIELD`) lets a seat with no commander use it too. The
/// picker is reactive already (it reads only the top of the stack); the duel
/// path adds two limits the pod path never had: constant damage that leaves
/// the victim alive is not answered, and a discard cost (Liliana the
/// Faultless) is not paid to save a token.
pub(super) fn pick_keyword_shield(state: &GameState, seat: usize, duel: bool) -> Option<GameAction> {
    let pod = !state.players[seat].commanders.is_empty();
    if !pod && !duel {
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
    // The duel limits. `spend_ok`: may this ability's cost go to the victim?
    let victim_card = victim.and_then(|v| state.battlefield_find(v));
    if !pod
        && let (Some(c), Some(dmg)) = (victim_card, threat_damage(effect))
        && state.computed_permanent(c.id).is_some_and(|cp| {
            cp.card_types().contains(&crate::card::CardType::Creature) && dmg < cp.toughness - c.damage as i32
        })
    {
        return None;
    }
    let spend_ok = |ab: &crate::effect::ActivatedAbility| {
        pod || ab.discard_cost.is_none() || !victim_card.is_some_and(|c| c.is_token)
    };
    let fits = |k: &Keyword| match threat {
        Threat::TargetedLethal => true,
        Threat::TargetedOther => matches!(k, Keyword::Hexproof | Keyword::Shroud),
        Threat::Sweep => matches!(k, Keyword::Indestructible),
    };
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in c.definition.activated_abilities.iter().enumerate() {
            let Some((kw, who)) = shield(&ab.effect) else { continue };
            if !fits(&kw) || !spend_ok(ab) {
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
    // "Protection from the colour of your choice" (Mother of Runes, Gods
    // Willing's ability kin): a coloured removal spell or ability aimed at
    // one of ours loses its target (CR 702.16b), and the grant names that
    // colour as it resolves (`stack_threat_color`).
    // Giver of Runes' colorless choice answers a colorless threat too.
    let colored = victim.is_some_and(|v| state.stack_threat_color(seat, &[v]).is_some());
    let colorless = victim.is_some_and(|v| state.stack_threat(seat, &[v]) == Some(None));
    if let Some(v) = victim
        && (colored || colorless)
    {
        for c in state.battlefield.iter().filter(|c| c.controller == seat) {
            for (idx, ab) in c.definition.activated_abilities.iter().enumerate() {
                let what = match &ab.effect {
                    Effect::GrantProtectionFromChosenColor { what, .. } if colored => what,
                    Effect::GrantProtectionFromChosenColorOrColorless { what, .. }
                    | Effect::GrantProtectionFromChosenCardType { what, .. } => what,
                    _ => continue,
                };
                if !spend_ok(ab) {
                    continue;
                }
                let aim = match what {
                    Selector::This if v == c.id => None,
                    Selector::Target(_) | Selector::TargetFiltered { .. } => Some(Target::Permanent(v)),
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
    }
    if let Some(v) = victim {
        for c in state.battlefield.iter().filter(|c| c.controller == seat) {
            for (idx, ab) in c.definition.activated_abilities.iter().enumerate() {
                if shelter_exile(&ab.effect).is_none() || !spend_ok(ab) {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: idx,
                    target: Some(Target::Permanent(v)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
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
        assert!(pick_keyword_shield(&g, 0, false).is_none(), "no mana");
        for c in [Color::Green, Color::Blue] {
            g.players[0].mana_pool.add(c, 1);
        }
        g.players[0].mana_pool.add_colorless(1);
        let a = pick_keyword_shield(&g, 0, false).expect("shroud");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, .. } if card_id == mystics));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "Murder lost its target");
    }

    /// Mother of Runes answers a Murder aimed at a Bear with protection from
    /// black: the Murder loses its target (CR 702.16b / 608.2b).
    #[test]
    fn mother_of_runes_answers_murder_with_protection_from_black() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        let mother = g.add_card_to_battlefield(0, crate::catalog::mother_of_runes());
        g.clear_sickness(mother);
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let murder = g.add_card_to_hand(1, crate::catalog::murder());
        g.players[1].mana_pool.add(Color::Black, 3);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: murder, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("Murder");
        g.priority.player_with_priority = 0;
        let a = pick_keyword_shield(&g, 0, false).expect("protection");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
            if card_id == mother && t == bear));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "Murder lost its target");
    }

    /// Livio's aegis exile answers a Murder aimed at a Bear, and its may is
    /// taken only while something hostile aims at one of ours.
    #[test]
    fn livio_shelters_a_murdered_bear() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        let livio = g.add_card_to_battlefield(0, crate::catalog::livio_oathsworn_sentinel());
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let ask = match &crate::catalog::livio_oathsworn_sentinel().activated_abilities[0].effect {
            Effect::MayDoBy { description, .. } => description.clone(),
            _ => unreachable!(),
        };
        assert_eq!(shelter_exile_answer(&g, 0, livio, &ask), Some(false), "nothing threatens the Bear");
        let murder = g.add_card_to_hand(1, crate::catalog::murder());
        g.players[1].mana_pool.add(Color::Black, 3);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: murder, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("Murder");
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        let a = pick_keyword_shield(&g, 0, false).expect("aegis exile");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
            if card_id == livio && t == bear));
        g.perform_action(a).expect("activate");
        assert_eq!(shelter_exile_answer(&g, 0, livio, &ask), Some(true), "the Murder aims at the Bear");
        assert_eq!(shelter_exile_answer(&g, 1, livio, &ask), Some(false), "nothing aims at seat 1's");
    }

    /// The bot activates Livio's return for its own sheltered card in a main
    /// phase — not when it would hand back more of an opponent's.
    #[test]
    fn livio_returns_a_sheltered_card() {
        use crate::server::bot::Bot;
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let livio = g.add_card_to_battlefield(0, crate::catalog::livio_oathsworn_sentinel());
        g.clear_sickness(livio);
        let angel = g.add_card_to_exile(0, crate::catalog::serra_angel());
        g.exile.iter_mut().find(|c| c.id == angel).unwrap().add_counters(crate::card::CounterType::Aegis, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        let mut b = crate::server::bot::HeuristicBot::default();
        let a = b.next_action(&g, 0);
        assert!(matches!(a, Some(GameAction::ActivateAbility { card_id, ability_index: 1, .. }) if card_id == livio), "{a:?}");
        let ret = &crate::catalog::livio_oathsworn_sentinel().activated_abilities[1].effect;
        assert!(exile_mass_return_pays(&g, 0, ret));
        for _ in 0..2 {
            let theirs = g.add_card_to_exile(1, crate::catalog::serra_angel());
            g.exile.iter_mut().find(|c| c.id == theirs).unwrap().add_counters(crate::card::CounterType::Aegis, 1);
        }
        assert!(!exile_mass_return_pays(&g, 0, ret), "two of theirs for one of ours");
    }

    /// `RF_DUEL_SHIELD`: in a duel, Liliana the Faultless pays {1}, tap and a
    /// discard to give a Craw Wurm hexproof against the Murder aimed at it
    /// (0 activations in 1,880 FRA games — the picker was Commander-only).
    /// A Shock that leaves the 6/4 alive is let through, an empty stack
    /// offers nothing, and without the bit the duel bot never shields.
    #[test]
    fn liliana_the_faultless_shields_in_a_duel() {
        use crate::server::bot::{Bot, EvalWeights, HeuristicBot, RF_DUEL_SHIELD};
        let board = |removal: Option<fn() -> crate::card::CardDefinition>| {
            let mut g = crate::game::two_player_game();
            g.active_player_idx = 1;
            g.step = TurnStep::PreCombatMain;
            let liliana = g.add_card_to_battlefield(0, crate::catalog::liliana_the_faultless());
            g.clear_sickness(liliana);
            let wurm = g.add_card_to_battlefield(0, crate::catalog::craw_wurm());
            g.add_card_to_hand(0, crate::catalog::island());
            g.players[0].mana_pool.add_colorless(1);
            if let Some(def) = removal {
                let spell = g.add_card_to_hand(1, def());
                g.players[1].mana_pool.add(Color::Black, 3);
                g.players[1].mana_pool.add(Color::Red, 1);
                g.priority.player_with_priority = 1;
                g.perform_action(GameAction::CastSpell {
                    card_id: spell, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: None,
                })
                .expect("removal");
            }
            g.priority.player_with_priority = 0;
            (g, liliana, wurm)
        };
        let shields = |a: &Option<GameAction>, liliana: CardId, wurm: CardId| {
            matches!(a, Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if *card_id == liliana && *t == wurm)
        };
        let off = EvalWeights::fra_fixes_on();
        let on = EvalWeights { rating_fixes: RF_DUEL_SHIELD, ..off };

        let (mut g, liliana, wurm) = board(Some(crate::catalog::murder));
        assert!(pick_keyword_shield(&g, 0, false).is_none(), "no commander, no bit");
        let a = HeuristicBot::with_weights(off).next_action(&g, 0);
        assert!(!shields(&a, liliana, wurm), "off: the duel bot lets the Murder resolve, got {a:?}");
        let a = HeuristicBot::with_weights(on).next_action(&g, 0);
        assert!(shields(&a, liliana, wurm), "on: hexproof in answer to the Murder, got {a:?}");
        g.perform_action(a.unwrap()).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.battlefield_find(wurm).is_some(), "the Murder lost its target");
        assert!(g.players[0].hand.is_empty(), "the discard was paid");

        let (g, liliana, wurm) = board(Some(crate::catalog::shock));
        assert!(pick_keyword_shield(&g, 0, true).is_none(), "2 damage leaves a 6/4 alive");
        let a = HeuristicBot::with_weights(on).next_action(&g, 0);
        assert!(!shields(&a, liliana, wurm), "got {a:?}");

        let (g, liliana, wurm) = board(None);
        assert!(pick_keyword_shield(&g, 0, true).is_none(), "nothing on the stack");
        let a = HeuristicBot::with_weights(on).next_action(&g, 0);
        assert!(!shields(&a, liliana, wurm), "got {a:?}");
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
            let picked = pick_keyword_shield(&g, 0, false);
            assert_eq!(
                matches!(picked, Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                    if card_id == tf && t == bear),
                fires,
                "counter {counter}"
            );
        }
    }
}
