//! Per-seat projection of [`GameState`] into a [`ClientView`].
//!
//! Hides information the viewer isn't entitled to see: opponent hand contents
//! are `Hidden`; libraries surface only their size (no reveal tracking yet);
//! stack items are fully visible (no face-down spells yet). When the engine
//! gains reveal-to-seat metadata, this file is where it plugs in.

use crate::card::{CardId, CardInstance};
use crate::effect::{Effect, Selector};
use crate::game::{GameState, StackItem};
use crate::mana::ManaSymbol;
use crate::net::{
    AbilityView, ClientView, GraveyardCardView, HandCardView, KnownCard, KnownStackItem,
    LibraryView, PendingDecisionView, PermanentView, PlayerView, StackItemView,
};
use crate::player::Player;

/// Project the authoritative `state` into the view visible to `seat`.
pub fn project(state: &GameState, seat: usize) -> ClientView {
    let computed = state.compute_battlefield();

    ClientView {
        your_seat: seat,
        active_player: state.active_player_idx,
        priority: state.player_with_priority(),
        step: state.step,
        turn: state.turn_number,
        players: state
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| project_player(p, i, seat))
            .collect(),
        battlefield: {
            let attacker_ids = state.attacking_ids();
            state
                .battlefield
                .iter()
                .map(|c| project_permanent(c, &computed, &attacker_ids))
                .collect()
        },
        stack: state
            .stack
            .iter()
            .map(|item| project_stack(item, state, seat))
            .collect(),
        pending_decision: state.pending_decision.as_ref().map(|pd| {
            let acting = pd.acting_player();
            PendingDecisionView {
                acting_player: acting,
                // Only the acting player sees decision specifics; spectators
                // see that someone is deciding but not the private contents.
                decision: (acting == seat).then(|| (&pd.decision).into()),
            }
        }),
        game_over: state.game_over,
    }
}

fn project_player(player: &Player, player_seat: usize, viewer_seat: usize) -> PlayerView {
    PlayerView {
        seat: player_seat,
        name: player.name.clone(),
        life: player.life,
        poison_counters: player.poison_counters,
        mana_pool: player.mana_pool.clone(),
        library: LibraryView {
            size: player.library.len(),
            known_top: Vec::new(),
        },
        graveyard: player.graveyard.iter().map(graveyard_entry).collect(),
        hand: player
            .hand
            .iter()
            .map(|c| project_hand_card(c, player_seat, viewer_seat))
            .collect(),
        lands_played_this_turn: player.lands_played_this_turn,
    }
}

fn project_hand_card(card: &CardInstance, owner_seat: usize, viewer_seat: usize) -> HandCardView {
    if owner_seat == viewer_seat {
        HandCardView::Known(known_card(card))
    } else {
        HandCardView::Hidden { id: card.id }
    }
}

fn known_card(card: &CardInstance) -> KnownCard {
    KnownCard {
        id: card.id,
        name: card.definition.name.to_string(),
        cost: card.definition.cost.clone(),
        card_types: card.definition.card_types.clone(),
        needs_target: spell_needs_target(&card.definition.effect),
        has_alternative_cost: card.definition.alternative_cost.is_some(),
        back_face_name: card
            .definition
            .back_face
            .as_ref()
            .map(|b| b.name.to_string()),
    }
}

fn graveyard_entry(card: &CardInstance) -> GraveyardCardView {
    GraveyardCardView {
        id: card.id,
        name: card.definition.name.to_string(),
    }
}

fn project_permanent(
    card: &CardInstance,
    computed: &[crate::game::layers::ComputedPermanent],
    attacking: &[CardId],
) -> PermanentView {
    let cp = computed.iter().find(|c| c.id == card.id);
    PermanentView {
        id: card.id,
        name: card.definition.name.to_string(),
        controller: card.controller,
        owner: card.owner,
        card_types: cp
            .map(|c| c.card_types.clone())
            .unwrap_or_else(|| card.definition.card_types.clone()),
        tapped: card.tapped,
        damage: card.damage,
        summoning_sick: card.summoning_sick,
        power: cp.map(|c| c.power).unwrap_or_else(|| card.power()),
        toughness: cp.map(|c| c.toughness).unwrap_or_else(|| card.toughness()),
        keywords: cp
            .map(|c| c.keywords.clone())
            .unwrap_or_else(|| card.definition.keywords.clone()),
        counters: card.counters.iter().map(|(k, v)| (*k, *v)).collect(),
        attached_to: card.attached_to,
        is_token: card.is_token,
        attacking: attacking.contains(&card.id),
        abilities: project_abilities(card),
    }
}

fn project_abilities(card: &CardInstance) -> Vec<AbilityView> {
    card.definition
        .activated_abilities
        .iter()
        .enumerate()
        .map(|(i, a)| AbilityView {
            index: i,
            cost_label: ability_cost_label(a),
            effect_label: ability_effect_label(&a.effect).to_string(),
            needs_target: spell_needs_target(&a.effect),
            is_mana: is_mana_ability(&a.effect),
        })
        .collect()
}

fn ability_cost_label(ability: &crate::effect::ActivatedAbility) -> String {
    let mut parts: Vec<String> = Vec::new();
    if ability.tap_cost {
        parts.push("{T}".into());
    }
    for sym in &ability.mana_cost.symbols {
        let tok = match sym {
            ManaSymbol::Colored(c) => format!("{{{c:?}}}"),
            ManaSymbol::Generic(n) => format!("{{{n}}}"),
            ManaSymbol::Colorless(n) => format!("{{{n}}}"),
            ManaSymbol::Hybrid(a, b) => format!("{{{a:?}/{b:?}}}"),
            ManaSymbol::Phyrexian(c) => format!("{{{c:?}/P}}"),
            ManaSymbol::Snow => "{S}".into(),
            ManaSymbol::X => "{X}".into(),
        };
        parts.push(tok);
    }
    if parts.is_empty() { "0".into() } else { parts.join(", ") }
}

fn ability_effect_label(effect: &Effect) -> &'static str {
    match effect {
        Effect::AddMana { .. } => "Add mana",
        Effect::Seq(steps) => steps.first().map(ability_effect_label).unwrap_or("Activate"),
        Effect::LoseLife { .. } => "Pay life / fetch land",
        Effect::Search { .. } => "Search library",
        Effect::Move { .. } => "Move permanent",
        Effect::DealDamage { .. } => "Deal damage",
        Effect::Draw { .. } => "Draw cards",
        Effect::Destroy { .. } => "Destroy permanent",
        Effect::Exile { .. } => "Exile permanent",
        Effect::GainLife { .. } => "Gain life",
        Effect::AddCounter { .. } => "Add counter",
        Effect::CreateToken { .. } => "Create token",
        _ => "Activate",
    }
}

/// "Auto-handled" mana abilities — ones that produce a fixed payload with no
/// player choice. The client filters these out of the right-click ability
/// menu (auto-tap activates them on the user's behalf). Choice-bearing mana
/// abilities like Black Lotus (`AnyOneColor`) ARE shown in the menu so the
/// player can pick a specific color before casting an off-color spell.
fn is_mana_ability(effect: &Effect) -> bool {
    use crate::effect::ManaPayload;
    match effect {
        Effect::AddMana { pool, .. } => matches!(
            pool,
            ManaPayload::Colors(_) | ManaPayload::Colorless(_)
        ),
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(is_mana_ability),
        _ => false,
    }
}

fn spell_needs_target(effect: &Effect) -> bool {
    fn has_target_selector(e: &Effect) -> bool {
        match e {
            Effect::DealDamage { to, .. } => matches!(to, Selector::Target(_)),
            Effect::Destroy { what }
            | Effect::Exile { what }
            | Effect::CounterSpell { what }
            | Effect::Move { what, .. } => matches!(what, Selector::Target(_)),
            Effect::PumpPT { what, .. } => matches!(what, Selector::Target(_)),
            Effect::Seq(steps) => steps.iter().any(has_target_selector),
            Effect::If { then, else_, .. } => {
                has_target_selector(then) || has_target_selector(else_)
            }
            _ => false,
        }
    }
    has_target_selector(effect)
}

fn project_stack(item: &StackItem, state: &GameState, _viewer_seat: usize) -> StackItemView {
    match item {
        StackItem::Spell { card, caster, target, .. } => StackItemView::Known(KnownStackItem {
            source: card.id,
            controller: *caster,
            name: card.definition.name.to_string(),
            target: target.clone(),
        }),
        StackItem::Trigger { source, controller, target, .. } => {
            let name = state
                .battlefield
                .iter()
                .find(|c| c.id == *source)
                .map(|c| c.definition.name.to_string())
                .unwrap_or_else(|| "Triggered ability".to_string());
            StackItemView::Known(KnownStackItem {
                source: *source,
                controller: *controller,
                name,
                target: target.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::GameState;
    use crate::net::HandCardView;
    use crate::player::Player;

    fn two_player_game() -> GameState {
        GameState::new(vec![
            Player::new(0, "P0"),
            Player::new(1, "P1"),
        ])
    }

    #[test]
    fn own_hand_is_known_opponent_hidden() {
        let mut state = two_player_game();
        state.add_card_to_hand(0, catalog::plains());
        state.add_card_to_hand(1, catalog::swamp());

        let view_p0 = project(&state, 0);
        assert!(matches!(view_p0.players[0].hand[0], HandCardView::Known(_)));
        assert!(matches!(view_p0.players[1].hand[0], HandCardView::Hidden { .. }));

        let view_p1 = project(&state, 1);
        assert!(matches!(view_p1.players[0].hand[0], HandCardView::Hidden { .. }));
        assert!(matches!(view_p1.players[1].hand[0], HandCardView::Known(_)));
    }

    #[test]
    fn library_size_public_contents_hidden() {
        let mut state = two_player_game();
        state.add_card_to_library(0, catalog::plains());
        state.add_card_to_library(0, catalog::plains());

        let view = project(&state, 1);
        assert_eq!(view.players[0].library.size, 2);
        assert!(view.players[0].library.known_top.is_empty());
    }

    #[test]
    fn graveyard_is_public() {
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Move it to graveyard directly for test purposes.
        let idx = state.battlefield.iter().position(|c| c.id == id).unwrap();
        let card = state.battlefield.remove(idx);
        state.players[0].graveyard.push(card);

        let view = project(&state, 1);
        assert_eq!(view.players[0].graveyard.len(), 1);
        assert_eq!(view.players[0].graveyard[0].name, "Grizzly Bears");
    }

    #[test]
    fn battlefield_uses_computed_power_toughness() {
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert_eq!(perm.power, 2);
        assert_eq!(perm.toughness, 2);
    }
}
