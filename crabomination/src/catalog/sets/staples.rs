//! Commonly-reprinted burn and card-draw staples that only need
//! primitives the engine already has (damage to any target, draw, scry,
//! lifegain). Wall of Omens also exercises CR 702.3 defender.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{black, blue, cost, generic, red, white};

/// "Any target" — a creature, player, or planeswalker.
fn any_target() -> Selector {
    target_filtered(
        SelectionRequirement::Creature
            .or(SelectionRequirement::Player)
            .or(SelectionRequirement::Planeswalker),
    )
}

/// Shock — {R} Instant. Deal 2 damage to any target.
pub fn shock() -> CardDefinition {
    CardDefinition {
        name: "Shock",
        cost: cost(&[red()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: any_target(), amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Searing Spear — {1}{R} Instant. Deal 3 damage to any target.
pub fn searing_spear() -> CardDefinition {
    CardDefinition {
        name: "Searing Spear",
        cost: cost(&[generic(1), red()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: any_target(), amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Volcanic Hammer — {1}{R} Sorcery. Deal 3 damage to any target.
pub fn volcanic_hammer() -> CardDefinition {
    CardDefinition {
        name: "Volcanic Hammer",
        cost: cost(&[generic(1), red()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage { to: any_target(), amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Lava Spike — {R} Sorcery. Deal 3 damage to target player or planeswalker.
pub fn lava_spike() -> CardDefinition {
    CardDefinition {
        name: "Lava Spike",
        cost: cost(&[red()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Player.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Skewer the Critics — {1}{R} Sorcery. Deal 3 damage to any target.
/// (Spectacle alternative cost omitted.)
pub fn skewer_the_critics() -> CardDefinition {
    CardDefinition {
        name: "Skewer the Critics",
        cost: cost(&[generic(1), red()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage { to: any_target(), amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Char — {2}{R} Instant. Deal 3 damage to any target and 1 damage to you.
pub fn char() -> CardDefinition {
    CardDefinition {
        name: "Char",
        cost: cost(&[generic(2), red()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: any_target(), amount: Value::Const(3) },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Lightning Helix — {R}{W} Instant. Deal 3 damage to any target and you
/// gain 3 life.
pub fn lightning_helix() -> CardDefinition {
    CardDefinition {
        name: "Lightning Helix",
        cost: cost(&[red(), white()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: any_target(), amount: Value::Const(3) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Magma Jet — {1}{R} Instant. Deal 2 damage to any target, then scry 2.
pub fn magma_jet() -> CardDefinition {
    CardDefinition {
        name: "Magma Jet",
        cost: cost(&[generic(1), red()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: any_target(), amount: Value::Const(2) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Flame Slash — {R} Sorcery. Deal 4 damage to target creature.
pub fn flame_slash() -> CardDefinition {
    CardDefinition {
        name: "Flame Slash",
        cost: cost(&[red()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Lava Coil — {1}{R} Sorcery. Deal 3 damage to target creature.
/// (Exile-if-it-would-die rider omitted.)
pub fn lava_coil() -> CardDefinition {
    CardDefinition {
        name: "Lava Coil",
        cost: cost(&[generic(1), red()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Galvanic Blast — {R} Instant. Deal 2 damage to any target.
/// (Affinity-boosted 4 damage omitted.)
pub fn galvanic_blast() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Blast",
        cost: cost(&[red()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: any_target(), amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Divination — {2}{U} Sorcery. Draw two cards.
pub fn divination() -> CardDefinition {
    CardDefinition {
        name: "Divination",
        cost: cost(&[generic(2), blue()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Read the Bones — {1}{B} Sorcery. Scry 2, draw two cards, lose 2 life.
pub fn read_the_bones() -> CardDefinition {
    CardDefinition {
        name: "Read the Bones",
        cost: cost(&[generic(1), black()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Wall of Omens — {1}{W} Creature — Wall 0/4. Defender; ETB draw a card.
pub fn wall_of_omens() -> CardDefinition {
    CardDefinition {
        name: "Wall of Omens",
        cost: cost(&[generic(1), white()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog;
    use crate::game::*;
    use crate::mana::Color;

    /// Cast a spell from hand paying r red + u blue + c generic.
    fn cast_face(card: crate::card::CardDefinition, r: u32, u: u32, c: u32, target: Option<Target>) -> GameState {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, card);
        g.players[0].mana_pool.add(Color::Red, r);
        g.players[0].mana_pool.add(Color::Blue, u);
        g.players[0].mana_pool.add_colorless(c);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        g
    }

    #[test]
    fn shock_kills_a_2_2() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::shock());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "Shock kills a 2/2");
    }

    #[test]
    fn searing_spear_burns_face_for_3() {
        let g = cast_face(catalog::searing_spear(), 1, 0, 1, Some(Target::Player(1)));
        assert_eq!(g.players[1].life, 17);
    }

    #[test]
    fn volcanic_hammer_burns_face_for_3() {
        let g = cast_face(catalog::volcanic_hammer(), 1, 0, 1, Some(Target::Player(1)));
        assert_eq!(g.players[1].life, 17);
    }

    #[test]
    fn lava_spike_burns_face_for_3() {
        let g = cast_face(catalog::lava_spike(), 1, 0, 0, Some(Target::Player(1)));
        assert_eq!(g.players[1].life, 17);
    }

    #[test]
    fn skewer_the_critics_burns_face_for_3() {
        let g = cast_face(catalog::skewer_the_critics(), 1, 0, 1, Some(Target::Player(1)));
        assert_eq!(g.players[1].life, 17);
    }

    #[test]
    fn char_burns_target_3_and_self_1() {
        let g = cast_face(catalog::char(), 1, 0, 2, Some(Target::Player(1)));
        assert_eq!(g.players[1].life, 17, "opponent takes 3");
        assert_eq!(g.players[0].life, 19, "caster takes 1");
    }

    #[test]
    fn lightning_helix_burns_3_and_gains_3() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::lightning_helix());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].life = 17;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 17, "opponent takes 3");
        assert_eq!(g.players[0].life, 20, "caster gains 3");
    }

    #[test]
    fn magma_jet_burns_2() {
        let g = cast_face(catalog::magma_jet(), 1, 0, 1, Some(Target::Player(1)));
        assert_eq!(g.players[1].life, 18);
    }

    #[test]
    fn flame_slash_kills_a_bear() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::flame_slash());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "4 damage kills the bear");
    }

    #[test]
    fn lava_coil_deals_3_to_a_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::lava_coil());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "3 damage kills a 2/2");
    }

    #[test]
    fn galvanic_blast_burns_2() {
        let g = cast_face(catalog::galvanic_blast(), 1, 0, 0, Some(Target::Player(1)));
        assert_eq!(g.players[1].life, 18);
    }

    #[test]
    fn divination_draws_two() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let before = g.players[0].hand.len();
        let id = g.add_card_to_hand(0, catalog::divination());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        // +1 (Divination in hand) -1 (cast) +2 (draw) = before + 2.
        assert_eq!(g.players[0].hand.len(), before + 2);
    }

    #[test]
    fn read_the_bones_draws_two_and_loses_two() {
        let mut g = two_player_game();
        for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
        let before = g.players[0].hand.len();
        let id = g.add_card_to_hand(0, catalog::read_the_bones());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 2, "net +2 cards");
        assert_eq!(g.players[0].life, 18, "lost 2 life");
    }

    #[test]
    fn wall_of_omens_etb_draws_a_card() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let before = g.players[0].hand.len();
        let id = g.add_card_to_hand(0, catalog::wall_of_omens());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        // -1 (cast Wall) +1 (ETB draw) = before.
        assert_eq!(g.players[0].hand.len(), before, "ETB drew a card");
    }

    #[test]
    fn wall_of_omens_cannot_attack() {
        // CR 702.3b — defender can't be declared as an attacker.
        let mut g = two_player_game();
        let wall = g.add_card_to_battlefield(0, catalog::wall_of_omens());
        g.clear_sickness(wall);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: wall, target: AttackTarget::Player(1),
        }]));
        assert!(matches!(res, Err(GameError::CreatureCannotAttack(_))),
            "defender rejected as attacker");
    }
}
