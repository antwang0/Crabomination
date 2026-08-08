//! CR conformance for this run's engine work:
//! - CR 704.5m — an Aura attached to an illegal object goes to its owner's
//!   graveyard, and CR 702.16k's "this effect doesn't remove this Aura"
//!   exempts the Aura that granted the protection.
//! - CR 702.26c — a phased-out permanent is treated as though it doesn't
//!   exist; `Keyword::CantPhaseOut` pins one in phase through its untap step.
//! - CR 615.7 / 614.9 — a "next time a source of your choice would deal
//!   damage" shield that redirects deals the prevented damage to that
//!   source's controller, wherever the damage was headed.

use crabomination::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes,
};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, StaticEffect};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::{Color, cost, generic, w};

fn bear(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// A white Aura that hands its host protection from white — with and without
/// the printed "this effect doesn't remove this Aura" rider.
fn white_ward(name: &'static str, keeps_self: bool) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: crabomination::effect::shortcut::target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            protection_keeps_self: keeps_self,
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has protection from white.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Protection(Color::White),
            },
        }],
        ..Default::default()
    }
}

/// CR 704.5m — protection from the Aura's own colour sheds it, unless the
/// Aura carries the CR 702.16k self-exemption.
#[test]
fn cr_704_5m_protection_sheds_the_aura_unless_it_exempts_itself() {
    for keeps_self in [false, true] {
        let mut g = two_player_game();
        let host = g.add_card_to_battlefield(0, bear("Host"));
        let aura = g.add_card_to_hand(0, white_ward("Pale Ward", keeps_self));
        g.players[0].mana_pool.add(Color::White, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: aura,
            target: Some(Target::Permanent(host)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        g.check_state_based_actions();
        assert_eq!(
            g.battlefield_find(aura).is_some(),
            keeps_self,
            "keeps_self={keeps_self}: the Aura {}",
            if keeps_self { "stays on" } else { "is shed" }
        );
        assert!(
            g.computed_permanent(host)
                .unwrap()
                .keywords
                .contains(&Keyword::Protection(Color::White))
                == keeps_self,
            "the grant only survives while its Aura does"
        );
    }
}

/// CR 702.26c — a phased-out permanent is treated as though it doesn't exist,
/// so its static ability stops applying while it's out.
#[test]
fn cr_702_26c_phased_out_permanents_stop_applying() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, bear("Other"));
    let anthem = g.add_card_to_battlefield(0, CardDefinition {
        keywords: vec![Keyword::Phasing],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::OtherThanSource),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..bear("Ghostly Anthem")
    });
    assert_eq!(g.computed_permanent(other).unwrap().power, 3, "anthem applies while in phase");

    // Its controller's untap step phases it out (CR 502.1, before untapping).
    g.active_player_idx = 0;
    g.do_phasing();
    assert!(g.phased_out.iter().any(|c| c.id == anthem), "phased out");
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "and stops existing for layers");
}

/// CR 702.26 — `Keyword::CantPhaseOut` (Spatial Binding) pins a permanent that
/// would otherwise phase out during its controller's untap step.
#[test]
fn cr_702_26_cant_phase_out_pins_the_permanent() {
    let mut g = two_player_game();
    let ghost = g.add_card_to_battlefield(0, CardDefinition {
        keywords: vec![Keyword::Phasing, Keyword::CantPhaseOut],
        ..bear("Pinned Ghost")
    });
    g.active_player_idx = 0;
    g.do_phasing();
    assert!(g.battlefield_find(ghost).is_some(), "pinned in phase despite Phasing");
    assert!(g.phased_out.is_empty());
}

/// CR 615.7 / 614.9 — Reflect Damage's floating shield soaks the chosen
/// source's next damage event anywhere and deals it to that source's
/// controller instead.
#[test]
fn cr_615_7_anywhere_shield_reflects_to_the_sources_controller() {
    let mut g = two_player_game();
    let pinger = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.clear_sickness(pinger);
    let victim = g.add_card_to_battlefield(0, bear("Bystander"));
    let spell = g.add_card_to_hand(0, catalog::reflect_damage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell);
    drain_stack(&mut g);
    // The shield floats over every recipient — it soaks the ping aimed at a
    // creature, not just one aimed at a player.
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pinger,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(0), "prevented");
    assert_eq!(g.players[1].life, 19, "dealt to the source's controller instead");
}
