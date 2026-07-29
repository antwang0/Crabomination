//! Functionality tests for Born of the Gods (BNG) — `catalog::sets::bng`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Cast `card` from seat 0's hand at `target` with `colorless` generic plus
/// the listed colored mana.
fn cast(
    g: &mut GameState,
    def: crabomination::card::CardDefinition,
    target: Option<Target>,
    colorless: u32,
    colors: &[(Color, u32)],
) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add_colorless(colorless);
    for (c, n) in colors {
        g.players[0].mana_pool.add(*c, *n);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
    id
}

/// Stat / keyword lines for the BNG bodies.
#[test]
fn bng_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::akroan_skyguard, 1, 1, &[Keyword::Flying]),
        (catalog::chorus_of_the_tides, 3, 2, &[Keyword::Flying]),
        (catalog::cyclops_of_one_eyed_pass, 5, 2, &[]),
        (catalog::deepwater_hypnotist, 2, 1, &[]),
        (catalog::elite_skirmisher, 3, 1, &[]),
        (catalog::forsaken_drifters, 4, 2, &[]),
        (catalog::great_hart, 2, 4, &[]),
        (catalog::griffin_dreamfinder, 1, 4, &[Keyword::Flying]),
        (catalog::impetuous_sunchaser, 1, 1, &[Keyword::Flying, Keyword::Haste, Keyword::MustAttack]),
        (catalog::kragma_butcher, 2, 3, &[]),
        (catalog::loyal_pegasus, 2, 1, &[Keyword::Flying, Keyword::CantAttackOrBlockAlone]),
        (catalog::marshmist_titan, 4, 5, &[]),
        (catalog::nyxborn_eidolon, 2, 1, &[]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
}

/// Heroic triggers: a counter for the Skyguard, a scry for the Chorus.
#[test]
fn bng_heroic_payoffs() {
    let mut g = main_phase();
    let sky = g.add_card_to_battlefield(0, catalog::akroan_skyguard());
    cast(&mut g, catalog::mortals_ardor(), Some(Target::Permanent(sky)), 0, &[(Color::White, 1)]);
    assert_eq!(g.battlefield_find(sky).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(sky).unwrap().keywords.contains(&Keyword::Lifelink));

    let chorus = g.add_card_to_battlefield(0, catalog::chorus_of_the_tides());
    g.add_card_to_library(0, catalog::great_hart());
    cast(
        &mut g,
        catalog::mortals_resolve(),
        Some(Target::Permanent(chorus)),
        1,
        &[(Color::Green, 1)],
    );
    assert!(g.computed_permanent(chorus).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Inspired fires when the creature untaps: the Butcher pumps itself, the
/// Hypnotist shrinks an opposing creature.
#[test]
fn bng_inspired_triggers_on_untap() {
    let mut g = main_phase();
    let butcher = g.add_card_to_battlefield(0, catalog::kragma_butcher());
    g.battlefield_find_mut(butcher).unwrap().tapped = true;
    for _ in 0..40 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
        if g.active_player_idx == 0 && !g.battlefield_find(butcher).unwrap().tapped {
            break;
        }
    }
    assert_eq!(g.computed_permanent(butcher).map(|c| c.power), Some(4), "+2/+0 on untap");
}

/// Asphyxiate only hits untapped creatures; Excoriate only tapped ones.
#[test]
fn bng_tap_state_removal() {
    let mut g = main_phase();
    let tapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let asphyxiate = g.add_card_to_hand(0, catalog::asphyxiate());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: asphyxiate,
            target: Some(Target::Permanent(tapped)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a tapped creature is an illegal Asphyxiate target"
    );
    g.perform_action(GameAction::CastSpell {
        card_id: asphyxiate,
        target: Some(Target::Permanent(untapped)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("untapped is legal");
    drain_stack(&mut g);
    assert!(g.battlefield_find(untapped).is_none(), "destroyed");

    cast(&mut g, catalog::excoriate(), Some(Target::Permanent(tapped)), 3, &[(Color::White, 1)]);
    assert!(g.battlefield_find(tapped).is_none(), "the tapped one was exiled");
}

/// Eye Gouge shrinks anything but outright kills a Cyclops.
#[test]
fn eye_gouge_executes_cyclopes() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, catalog::eye_gouge(), Some(Target::Permanent(bear)), 0, &[(Color::Black, 1)]);
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((1, 1)));

    let cyclops = g.add_card_to_battlefield(1, catalog::cyclops_of_one_eyed_pass());
    cast(&mut g, catalog::eye_gouge(), Some(Target::Permanent(cyclops)), 0, &[(Color::Black, 1)]);
    assert!(g.battlefield_find(cyclops).is_none(), "the Cyclops was destroyed");
}

/// Fall of the Hammer swings your creature's power at another creature.
#[test]
fn fall_of_the_hammer_hits_for_power() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::cyclops_of_one_eyed_pass());
    let theirs = g.add_card_to_battlefield(1, catalog::great_hart());
    let spell = g.add_card_to_hand(0, catalog::fall_of_the_hammer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "5 damage killed the 2/4");
}

/// Bolt of Keranos burns and scries; Hold at Bay soaks the next 7 damage.
#[test]
fn bng_burn_and_prevention() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::great_hart());
    let life = g.players[1].life;
    cast(&mut g, catalog::bolt_of_keranos(), Some(Target::Player(1)), 1, &[(Color::Red, 2)]);
    assert_eq!(g.players[1].life, life - 3);

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast(&mut g, catalog::hold_at_bay(), Some(Target::Permanent(bear)), 1, &[(Color::White, 1)]);
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        2,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "prevented");
}

/// Nullify counters a creature spell.
#[test]
fn nullify_counters_creature_spells() {
    let mut g = main_phase();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bear on the stack");
    g.priority.player_with_priority = 0;
    let nullify = g.add_card_to_hand(0, catalog::nullify());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: nullify,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// Glimpse the Sun God taps X creatures.
#[test]
fn glimpse_the_sun_god_taps_x() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::glimpse_the_sun_god());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
}

/// Marshmist Titan's devotion discount pays for its generic pips.
#[test]
fn marshmist_titan_devotion_discount() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::nyxborn_eidolon());
    }
    let titan = g.add_card_to_hand(0, catalog::marshmist_titan());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: titan,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("three black pips shaved {3} off");
    drain_stack(&mut g);
    assert!(g.battlefield_find(titan).is_some());
}

/// Felhide Brawler can't block without another Minotaur.
#[test]
fn felhide_brawler_needs_a_friend() {
    let mut g = main_phase();
    let brawler = g.add_card_to_battlefield(0, catalog::felhide_brawler());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!g.blocker_can_block_attacker(brawler, attacker), "alone it can't block");
    g.add_card_to_battlefield(0, catalog::kragma_butcher());
    assert!(g.blocker_can_block_attacker(brawler, attacker), "another Minotaur unlocks it");
}

/// Forsaken Drifters mills four when it dies; Griffin Dreamfinder rebuys an
/// enchantment.
#[test]
fn bng_graveyard_value() {
    let mut g = main_phase();
    let drifters = g.add_card_to_battlefield(0, catalog::forsaken_drifters());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::great_hart());
    }
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(drifters)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kill it");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 2, "milled four");

    let aura = g.add_card_to_graveyard(0, catalog::nyxborn_eidolon());
    let griffin = g.add_card_to_hand(0, catalog::griffin_dreamfinder());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: griffin,
        target: Some(Target::Permanent(aura)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Griffin");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "enchantment card returned");
}

/// Stat lines for the second BNG wave.
#[test]
fn bng_wave2_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32)] = &[
        (catalog::akroan_phalanx, 3, 3),
        (catalog::ashioks_adept, 1, 3),
        (catalog::graverobber_spider, 2, 4),
        (catalog::nyxborn_shieldmate, 1, 2),
        (catalog::nyxborn_triton, 2, 3),
        (catalog::nyxborn_wolf, 3, 1),
    ];
    for (f, p, t) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        assert!(d.bestow.is_some() || !d.name.starts_with("Nyxborn"), "{} bestows", d.name);
    }
}

/// Ephara's Radiance grants its host a tap-for-life ability.
#[test]
fn granting_auras_hand_over_their_ability() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    cast(&mut g, catalog::epharas_radiance(), Some(Target::Permanent(bear)), 0, &[(Color::White, 1)]);
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("granted ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the tap cost hit the host");
}

/// Gorgon's Head hands the equipped creature deathtouch.
#[test]
fn gorgons_head_grants_deathtouch() {
    let mut g = main_phase();
    let head = g.add_card_to_battlefield(0, catalog::gorgons_head());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: head, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Graverobber Spider scales with the creature cards in your graveyard.
#[test]
fn graverobber_spider_scales_with_the_yard() {
    let mut g = main_phase();
    let spider = g.add_card_to_battlefield(0, catalog::graverobber_spider());
    g.battlefield_find_mut(spider).unwrap().summoning_sick = false;
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::great_hart());
    }
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: spider,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(spider).map(|c| (c.power, c.toughness)), Some((5, 7)));
}

/// Ashiok's Adept's heroic strips a card from each opponent.
#[test]
fn ashioks_adept_heroic_discards() {
    let mut g = main_phase();
    let adept = g.add_card_to_battlefield(0, catalog::ashioks_adept());
    g.add_card_to_hand(1, catalog::great_hart());
    cast(&mut g, catalog::mortals_ardor(), Some(Target::Permanent(adept)), 0, &[(Color::White, 1)]);
    assert!(g.players[1].hand.is_empty(), "the opponent discarded");
}
