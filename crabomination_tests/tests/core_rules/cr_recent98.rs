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
                .keywords()
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

/// CR 702.26i — an Equipment that phased out directly phases back in
/// unattached when the creature it was on died meanwhile (a pod's Clever
/// Concealment left Lightning Greaves pointing at a dead Yuffie).
#[test]
fn cr_702_26i_equipment_phases_in_unattached_from_a_departed_host() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, bear("Host"));
    let greaves = g.add_card_to_battlefield(0, catalog::lightning_greaves());
    g.battlefield_find_mut(greaves).unwrap().attached_to = Some(host);
    let i = g.battlefield.iter().position(|c| c.id == greaves).unwrap();
    let c = g.battlefield.remove(i);
    g.phased_out.push(c);
    let j = g.battlefield.iter().position(|c| c.id == host).unwrap();
    let dead = g.battlefield.remove(j);
    g.players[0].graveyard.push(dead);
    g.active_player_idx = 0;
    g.do_phasing();
    let back = g.battlefield_find(greaves).expect("phased in");
    assert_eq!(back.attached_to, None, "its host is in the graveyard");
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

/// CR 704.5n — an Equipment whose host is gone (here: in a graveyard) is
/// unattached by the state-based sweep, the net under CR 702.26i.
#[test]
fn cr_704_5n_equipment_on_a_departed_host_is_unattached() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, bear("Host"));
    let greaves = g.add_card_to_battlefield(0, catalog::lightning_greaves());
    g.battlefield_find_mut(greaves).unwrap().attached_to = Some(host);
    let j = g.battlefield.iter().position(|c| c.id == host).unwrap();
    let dead = g.battlefield.remove(j);
    g.players[0].graveyard.push(dead);
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(greaves).unwrap().attached_to, None);
}

/// CR 704.3 / 502.1 — the untap step's phasing has no priority, but the
/// upkeep does: a legend that phases in beside its namesake meets the legend
/// rule before the active player can act.
#[test]
fn cr_704_3_a_phased_in_legend_meets_the_legend_rule_before_upkeep_priority() {
    let mut g = two_player_game();
    let legend = || CardDefinition { supertypes: vec![crabomination::card::Supertype::Legendary], ..bear("Twin Legend") };
    let a = g.add_card_to_battlefield(0, legend());
    g.add_card_to_battlefield(0, legend());
    let i = g.battlefield.iter().position(|c| c.id == a).unwrap();
    let c = g.battlefield.remove(i);
    g.phased_out.push(c);
    g.active_player_idx = 1;
    g.step = crabomination::TurnStep::End;
    g.priority.player_with_priority = 1;
    for _ in 0..8 {
        if g.active_player_idx == 0 && g.step == crabomination::TurnStep::Upkeep {
            break;
        }
        if g.pending_decision.is_some() {
            let d = g.pending_decision.as_ref().unwrap().decision.clone();
            g.submit_decision(crabomination::decision::Decider::decide(&mut crabomination::decision::AutoDecider, &d)).unwrap();
            continue;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!((g.active_player_idx, g.step), (0, crabomination::TurnStep::Upkeep));
    let n = g.battlefield.iter().filter(|c| c.definition.name == "Twin Legend").count();
    assert_eq!(n, 1, "the legend rule ran before upkeep priority");
}

/// CR 704.3 / 511.2 — an "until end of combat" boost expires as the combat
/// phase ends; a creature it was keeping alive dies before the postcombat
/// main phase's priority.
#[test]
fn cr_704_3_an_end_of_combat_expiry_is_swept_before_main_two() {
    use crabomination::effect::{Duration, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let pumped = g.add_card_to_battlefield(0, bear("Pumped"));
    let src = g.add_card_to_battlefield(0, bear("Source"));
    let pump = Effect::PumpPT {
        what: Selector::Target(0),
        power: Value::Const(2),
        toughness: Value::Const(2),
        duration: Duration::EndOfCombat,
    };
    let ctx = EffectContext::for_trigger(src, 0, Some(Target::Permanent(pumped)), 0);
    g.resolve_effect(&pump, &ctx).unwrap();
    g.battlefield_find_mut(pumped).unwrap().damage = 3;
    g.active_player_idx = 0;
    g.step = crabomination::TurnStep::EndCombat;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PassPriority).expect("pass");
    g.perform_action(GameAction::PassPriority).expect("pass");
    assert_eq!(g.step, crabomination::TurnStep::PostCombatMain);
    assert!(g.battlefield_find(pumped).is_none(), "3 damage on a 2/2 once the pump is gone");
}

/// CR 704.3 / 117.5 — casting a creature out of a graveyard is swept before
/// its caster acts again: Gravecrawler was the only creature card in any
/// graveyard, so Bonehoard's "+X/+X, X = creature cards in all graveyards"
/// falls to +0/+0 and the 0/0 it equips dies (a pod found a Phyrexian Germ
/// alive at 0/0 after a Vengeful Dead was cast from a graveyard).
#[test]
fn cr_704_3_a_card_leaving_a_graveyard_is_swept_after_the_action() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = crabomination::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let germ = g.add_card_to_battlefield(0, CardDefinition { power: 0, toughness: 0, ..bear("Germ") });
    let hoard = g.add_card_to_battlefield(0, catalog::bonehoard());
    g.battlefield_find_mut(hoard).unwrap().attached_to = Some(germ);
    g.add_card_to_battlefield(0, catalog::gravecrawler()); // the Zombie it needs
    let crawler = g.add_card_to_graveyard(0, catalog::gravecrawler());
    g.check_state_based_actions();
    assert!(g.battlefield_find(germ).is_some(), "1/1 while Gravecrawler is in the graveyard");
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastFlashback {
        card_id: crawler,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
        mode: None,
    })
    .expect("cast from the graveyard");
    assert!(g.battlefield_find(germ).is_none(), "0/0 once the graveyard is empty, before anyone acts");
}

/// CR 704.3 / 502.1 — an "until your next turn" effect ends as that turn
/// begins, in a step with no priority, and the sweep before the upkeep's
/// priority sees the result: a 0/0 a "+1/+1 until your next turn" kept alive
/// dies (a pod's Synth Infiltrator outlived its "until your next turn" copy).
#[test]
fn cr_704_3_an_until_your_next_turn_expiry_is_swept_before_upkeep_priority() {
    use crabomination::effect::{Duration, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let germ = g.add_card_to_battlefield(0, CardDefinition { power: 0, toughness: 0, ..bear("Germ") });
    let src = g.add_card_to_battlefield(0, bear("Source"));
    let pump = Effect::PumpPT {
        what: Selector::Target(0),
        power: Value::Const(1),
        toughness: Value::Const(1),
        duration: Duration::UntilNextTurn,
    };
    let ctx = EffectContext::for_trigger(src, 0, Some(Target::Permanent(germ)), 0);
    g.resolve_effect(&pump, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(germ).is_some(), "1/1 under the pump");
    g.active_player_idx = 1;
    g.step = crabomination::TurnStep::End;
    g.priority.player_with_priority = 1;
    for _ in 0..8 {
        if g.active_player_idx == 0 && g.step == crabomination::TurnStep::Upkeep {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!((g.active_player_idx, g.step), (0, crabomination::TurnStep::Upkeep));
    assert!(g.battlefield_find(germ).is_none(), "0/0 once its controller's turn began");
}
