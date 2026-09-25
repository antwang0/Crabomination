//! Responses to a spell on the stack that neither counter nor buff it, and so
//! no other picker reaches: copying the bot's own spell from the graveyard
//! (Increasing Vengeance's flashback, Cooperate's aftermath), and punishing an
//! opponent's big spell with damage to its controller (Refuse, Parallectric
//! Feedback). Mystic Intellect's census left all three uncast in 1,000 pods.
//! Also the idle-main fallback for a card-neutral retrieval (Runic
//! Repetition): `eval_material` prices it at zero, so it only goes when
//! nothing else would.

use crate::card::CardType;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, StackItem, Target};

/// Graveyard casts that copy `spell_id`: flashback instants and aftermath
/// instant halves whose effect copies a target spell. `copies` is the bot's
/// shape test, so a new copy shape needs teaching in one place.
pub(super) fn graveyard_copy_casts(
    state: &GameState,
    seat: usize,
    spell_id: crate::card::CardId,
    copies: fn(&Effect) -> bool,
) -> Option<GameAction> {
    let target = Some(Target::Permanent(spell_id));
    state.players[seat].graveyard.iter().find_map(|c| {
        let def = &c.definition;
        let action = if def.has_flashback_ability() && def.is_instant() && copies(&def.effect) {
            GameAction::CastFlashback { card_id: c.id, target: target.clone(), additional_targets: vec![], mode: None, x_value: None }
        } else if let Some(split) = def.split.as_deref()
            && split.aftermath
            && split.right.card_types.contains(&CardType::Instant)
            && copies(&split.right.effect)
        {
            GameAction::CastAftermath { card_id: c.id, target: target.clone(), additional_targets: vec![], mode: None, x_value: None }
        } else {
            return None;
        };
        state.would_accept(action.clone()).then_some(action)
    })
}

/// True when the effect deals damage to its target spell's controller.
fn punishes_target_spell(eff: &Effect) -> bool {
    matches!(eff, Effect::DealDamage { to: Selector::Player(PlayerRef::ControllerOf(inner)), .. }
        if matches!(**inner, Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. }))
}

/// An opponent's spell of mana value 4 or more on top of the stack: hit its
/// controller for that much with a punishing instant from hand.
pub(super) fn pick_punish_response(state: &GameState, seat: usize) -> Option<GameAction> {
    let Some(StackItem::Spell { card, caster, .. }) = state.stack.last() else { return None };
    if *caster == seat || card.definition.cost.cmc() < 4 || state.same_team(seat, *caster) {
        return None;
    }
    let spell_id = card.id;
    state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.is_instant() && punishes_target_spell(&c.definition.effect))
        .map(|c| GameAction::CastSpell {
            card_id: c.id,
            target: Some(Target::Permanent(spell_id)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .find(|a| state.would_accept(a.clone()))
}

/// With nothing better to do in its own main phase, cast a spell whose whole
/// effect returns a card from a graveyard or exile to hand, at the
/// auto-picked target. One card for one card reads as no gain to the
/// evaluator, so no scored path cast Runic Repetition.
pub(super) fn pick_idle_retrieval(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::ZoneDest;
    let retrieves = |eff: &Effect| {
        matches!(eff, Effect::Move { what: Selector::TargetFiltered { filter, .. }, to: ZoneDest::Hand(_) }
            if filter.mentions_offboard_zone())
    };
    state.players[seat].hand.iter().filter(|c| retrieves(&c.definition.effect)).find_map(|c| {
        let target = state.auto_target_for_effect(&c.definition.effect, seat)?;
        let a = GameAction::CastSpell {
            card_id: c.id,
            target: Some(target),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        state.would_accept(a.clone()).then_some(a)
    })
}
