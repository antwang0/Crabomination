//! Extra-turn spells — "take an extra turn after this one" (CR 500.7),
//! wired via `Effect::TakeExtraTurn` and the per-player `extra_turns`
//! bank consumed during turn advance.

use crate::card::{CardDefinition, CardType};
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{blue, cost, generic};

/// Take an extra turn after this one.
fn extra_turn_body() -> Effect {
    Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::Const(1) }
}

/// Time Walk — {1}{U} Sorcery. "Take an extra turn after this one."
pub fn time_walk() -> CardDefinition {
    CardDefinition {
        name: "Time Walk",
        cost: cost(&[generic(1), blue()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Time Warp — {3}{U}{U} Sorcery. "Take an extra turn after this one."
pub fn time_warp() -> CardDefinition {
    CardDefinition {
        name: "Time Warp",
        cost: cost(&[generic(3), blue(), blue()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Temporal Manipulation — {3}{U}{U} Sorcery. "Take an extra turn after
/// this one."
pub fn temporal_manipulation() -> CardDefinition {
    CardDefinition {
        name: "Temporal Manipulation",
        cost: cost(&[generic(3), blue(), blue()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Capture of Jingzhou — {3}{U}{U} Sorcery. "Take an extra turn after
/// this one." (Time Warp reprint.)
pub fn capture_of_jingzhou() -> CardDefinition {
    CardDefinition {
        name: "Capture of Jingzhou",
        cost: cost(&[generic(3), blue(), blue()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Nexus of Fate — {5}{U}{U} Instant. "Take an extra turn after this
/// one." (The shuffle-instead-of-graveyard rider is omitted — no
/// leaves-graveyard replacement primitive yet.)
pub fn nexus_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Nexus of Fate",
        cost: cost(&[generic(5), blue(), blue()]),
        card_types: vec![CardType::Instant],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog;
    use crate::game::*;
    use crate::mana::Color;

    fn cast_and_resolve(card: crate::card::CardDefinition, blue: u32, generic: u32) -> GameState {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, card);
        g.players[0].mana_pool.add(Color::Blue, blue);
        g.players[0].mana_pool.add_colorless(generic);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        g
    }

    #[test]
    fn time_walk_banks_one_extra_turn() {
        let g = cast_and_resolve(catalog::time_walk(), 1, 1);
        assert_eq!(g.players[0].extra_turns, 1);
    }

    #[test]
    fn time_warp_banks_one_extra_turn() {
        let g = cast_and_resolve(catalog::time_warp(), 2, 3);
        assert_eq!(g.players[0].extra_turns, 1);
    }

    #[test]
    fn temporal_manipulation_banks_one_extra_turn() {
        let g = cast_and_resolve(catalog::temporal_manipulation(), 2, 3);
        assert_eq!(g.players[0].extra_turns, 1);
    }

    #[test]
    fn capture_of_jingzhou_banks_one_extra_turn() {
        let g = cast_and_resolve(catalog::capture_of_jingzhou(), 2, 3);
        assert_eq!(g.players[0].extra_turns, 1);
    }

    #[test]
    fn nexus_of_fate_banks_one_extra_turn() {
        let g = cast_and_resolve(catalog::nexus_of_fate(), 2, 5);
        assert_eq!(g.players[0].extra_turns, 1);
    }

    #[test]
    fn extra_turn_then_taken_keeps_active_player() {
        let mut g = cast_and_resolve(catalog::time_walk(), 1, 1);
        g.active_player_idx = 0;
        g.do_untap();
        assert_eq!(g.active_player_idx, 0, "extra turn keeps the same player");
        assert_eq!(g.players[0].extra_turns, 0, "charge consumed");
    }
}
