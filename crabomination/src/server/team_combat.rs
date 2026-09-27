//! A team combat ability after blocks: "attacking creatures you control get
//! +X/+X" (Jazal Goldmane) or "creatures you control gain deathtouch and
//! lifelink" (Vault of the Archangel). `pick_team_pump` sits in the main-phase
//! enumerator, where nothing is attacking yet, so neither fired in play: a
//! 183-deck `--card-census --a dflt` listed both in six decks each. Commander
//! games only, so two-player play is unchanged.

use crate::card::Keyword;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::GameAction;

use super::bot::{EvalWeights, requirement_restricts_to_your_creatures, sink_sacrifice_cost};

/// Keywords that change a combat already declared.
const COMBAT_KEYWORDS: [Keyword; 5] =
    [Keyword::Deathtouch, Keyword::Lifelink, Keyword::DoubleStrike, Keyword::FirstStrike, Keyword::Trample];

/// Does `what` name your own creatures, all of them or all your attackers?
fn your_team(what: &Selector) -> bool {
    match what {
        Selector::EachPermanent(req) | Selector::EachMatching { filter: req, .. } => {
            requirement_restricts_to_your_creatures(req)
        }
        _ => false,
    }
}

/// Is `e` a team pump worth its mana this combat, or a team combat-keyword
/// grant one of `attackers` still lacks?
fn team_combat_effect(
    state: &GameState,
    e: &Effect,
    ctx: &crate::game::effects::EffectContext,
    attackers: &[crate::card::CardId],
) -> bool {
    let lacks = |k: &Keyword| {
        COMBAT_KEYWORDS.contains(k)
            && attackers.iter().any(|&a| state.computed_permanent(a).is_some_and(|c| !c.keywords().contains(k)))
    };
    match e {
        Effect::PumpPT { what, power, .. } => your_team(what) && state.evaluate_value(power, ctx) > 0,
        Effect::GrantKeyword { what, keyword, .. } => your_team(what) && lacks(keyword),
        Effect::GrantKeywords { what, keywords, .. } => your_team(what) && keywords.iter().any(lacks),
        _ => false,
    }
}

/// The first accepted team combat ability once two or more of `seat`'s
/// creatures attack and blocks are in.
pub(super) fn pick_team_combat_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    let attackers: Vec<crate::card::CardId> = state
        .attacking()
        .iter()
        .filter(|a| state.battlefield_find(a.attacker).is_some_and(|c| c.controller == seat))
        .map(|a| a.attacker)
        .collect();
    if attackers.len() < 2 {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if sink_sacrifice_cost(ab, w) || (ab.tap_cost && card.tapped) {
                continue;
            }
            let ctx = crate::game::effects::EffectContext::for_trigger(card.id, seat, None, 0);
            if !team_combat_effect(state, &ab.effect, &ctx, &attackers) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    use crate::mana::Color;

    /// Vault of the Archangel gives two attackers deathtouch and lifelink in a
    /// pod once blocks are in; one attacker, or no commander, keeps it back.
    #[test]
    fn vault_of_the_archangel_arms_two_attackers() {
        let mut g = crate::game::multi_player_game(3);
        let vault = g.add_card_to_battlefield(0, crate::catalog::vault_of_the_archangel());
        let a = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.attacking = vec![Attack { attacker: a, target: AttackTarget::Player(1) }];
        let w = EvalWeights::default();
        let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.players[0].commanders.push(cmd);
        assert_eq!(pick_team_combat_ability(&g, 0, &w), None, "one attacker");
        g.attacking.push(Attack { attacker: b, target: AttackTarget::Player(2) });
        assert!(matches!(
            pick_team_combat_ability(&g, 0, &w),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == vault
        ));
        g.players[0].commanders.clear();
        assert_eq!(pick_team_combat_ability(&g, 0, &w), None, "no commander: not a pod");
    }
}
