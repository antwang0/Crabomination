//! A damage trigger's target is worth an opposing permanent only when the
//! damage can hurt it. Phyrexian Vindicator redirected its prevented damage
//! into an indestructible Zetalpa, whose Wrathful Raptors trigger aimed the
//! damage back at the Vindicator — prevented and redirected again, a CR
//! 104.4b draw neither bot was forced into (6-seat pod `--pod-decks
//! 1112,1113,1114,1110,1111,1109 --seed 10842 --first 41`).

use crate::effect::{Effect, StaticEffect};
use crate::game::GameState;
use crate::game::types::{ResumeContext, Target};

/// True when `e` deals damage (to whatever it targets).
fn deals_damage(e: &Effect) -> bool {
    match e {
        Effect::DealDamage { .. } | Effect::DealDamageFrom { .. } => true,
        Effect::Seq(v) => v.iter().any(deals_damage),
        Effect::If { then, else_, .. } => deals_damage(then) || deals_damage(else_),
        _ => false,
    }
}

/// Damage to `id` does nothing lasting: it's indestructible, or the damage
/// is prevented (and perhaps thrown back).
fn unhurt(state: &GameState, id: crate::card::CardId) -> bool {
    let Some(c) = state.computed_permanent(id) else { return false };
    let preventable = !state.prevention_off();
    c.keywords().contains(&crate::card::Keyword::Indestructible)
        || state.permanent_prevents_all_damage_to_self(id)
        || preventable
            && state.battlefield_find(id).is_some_and(|b| {
                b.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::PreventDamageToThisRedirect))
            })
}

/// The legal targets minus opposing permanents a pending damage trigger
/// can't hurt — `None` when nothing is dropped or nothing would be left.
pub(super) fn trim_unhurt(state: &GameState, seat: usize, legal: &[Target]) -> Option<Vec<Target>> {
    let pd = state.pending_decision.as_ref()?;
    let ResumeContext::TriggerTargetPick { pending, .. } = &pd.resume else { return None };
    if !deals_damage(&pending.effect) {
        return None;
    }
    let drop = |t: &Target| match t {
        Target::Permanent(id) => {
            state.battlefield_find(*id).is_some_and(|c| c.controller != seat) && unhurt(state, *id)
        }
        _ => false,
    };
    let kept: Vec<Target> = legal.iter().filter(|t| !drop(t)).cloned().collect();
    // Never trimmed down to our own face or board: an unhurt opposing
    // permanent ends the chain, our own Pariah-shielded face feeds it
    // (Black Panther behind an opponent's Shalai, seed 923534).
    let hostile_left = kept.iter().any(|t| match t {
        Target::Player(p) => *p != seat,
        Target::Permanent(id) => state.battlefield_find(*id).is_some_and(|c| c.controller != seat),
    });
    (kept.len() < legal.len() && hostile_left).then_some(kept)
}

#[cfg(test)]
mod tests {
    use crate::decision::DecisionAnswer;
    use crate::game::types::{GameAction, Target, TurnStep};

    /// CR 104.4b — Phyrexian Vindicator's redirect skips an indestructible
    /// Zetalpa (its damage would only feed the Raptors loop) for a creature
    /// it can kill.
    #[test]
    fn a_redirect_skips_an_indestructible_target() {
        let mut g = crate::game::multi_player_game(2);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, crate::catalog::phyrexian_vindicator());
        let vindicator = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Vindicator").map(|c| c.id).unwrap();
        let zetalpa = g.add_card_to_battlefield(1, crate::catalog::zetalpa_primal_dawn());
        g.players[0].wants_ui = true;
        let bolt = g.add_card_to_hand(1, crate::catalog::lightning_bolt());
        g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(vindicator)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        for _ in 0..4 {
            if g.pending_decision.is_some() {
                break;
            }
            let _ = g.perform_action(GameAction::PassPriority);
        }
        let pd = g.pending_decision.clone().expect("the redirect asks for a target");
        let w = crate::server::bot::EvalWeights::default();
        let answer = crate::server::bot::decide_pending_policy(&g, 0, &w, &pd.decision, false);
        assert!(
            !matches!(answer, DecisionAnswer::Target(Target::Permanent(id)) if id == zetalpa),
            "not the indestructible Zetalpa: {answer:?}"
        );
    }

    /// CR 104.4b — with the opponent's face out of reach (hexproof)
    /// the indestructible Zetalpa stays a candidate: trimming it would leave
    /// only the redirector's own controller, whose damage feeds the chain
    /// (Black Panther behind an opponent's Shalai, seed 923534).
    #[test]
    fn the_trim_never_leaves_only_our_own_side() {
        let mut g = crate::game::multi_player_game(2);
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        let vindicator = g.add_card_to_battlefield(0, crate::catalog::phyrexian_vindicator());
        let zetalpa = g.add_card_to_battlefield(1, crate::catalog::zetalpa_primal_dawn());
        g.players[0].wants_ui = true;
        let bolt = g.add_card_to_hand(1, crate::catalog::lightning_bolt());
        g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(vindicator)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        for _ in 0..4 {
            if g.pending_decision.is_some() {
                break;
            }
            let _ = g.perform_action(GameAction::PassPriority);
        }
        assert!(g.pending_decision.is_some(), "the redirect asks for a target");
        // The opponent's face out of reach: only our own face and Zetalpa.
        let legal = [Target::Player(0), Target::Permanent(zetalpa)];
        assert_eq!(super::trim_unhurt(&g, 0, &legal), None);
    }
}
