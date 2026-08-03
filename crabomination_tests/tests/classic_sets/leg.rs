//! Legends (LEG) — the CR 702.22 "bands with other" cycle
//! (`catalog::sets::leg`).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn bands(g: &GameState, id: crabomination::card::CardId) -> bool {
    g.computed_permanent(id)
        .map(|c| c.keywords.iter().any(|k| matches!(k, Keyword::BandsWithOther(_))))
        .unwrap_or(false)
}

/// Each band land grants only its own colour's legends.
#[test]
fn band_lands_grant_their_own_color_only() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::cathedral_of_serra());
    let green = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    assert!(!bands(&g, green), "white land, green legend");
    g.add_card_to_battlefield(0, catalog::adventurers_guildhouse());
    assert!(bands(&g, green));
}

/// The other three band lands round out the cycle and tap for {C}.
#[test]
fn the_rest_of_the_band_land_cycle_taps_for_colorless() {
    let mut g = main_phase();
    for def in
        [catalog::mountain_stronghold(), catalog::seafarers_quay(), catalog::unholy_citadel()]
    {
        let land = g.add_card_to_battlefield(0, def);
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("tap for mana");
    }
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3);
}

/// Master of the Hunt's Wolves band with each other.
#[test]
fn master_of_the_hunt_mints_banding_wolves() {
    let mut g = main_phase();
    let master = g.add_card_to_battlefield(0, catalog::master_of_the_hunt());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: master,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("mint");
    drain_stack(&mut g);
    let wolf = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Wolves of the Hunt")
        .map(|c| c.id)
        .expect("a Wolf");
    assert!(bands(&g, wolf));
}

/// Shelkin Brownie strips the grant for the turn.
#[test]
fn shelkin_brownie_strips_bands_with_other() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::adventurers_guildhouse());
    let brownie = g.add_card_to_battlefield(0, catalog::shelkin_brownie());
    g.clear_sickness(brownie);
    let legend = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    assert!(bands(&g, legend));
    g.perform_action(GameAction::ActivateAbility {
        card_id: brownie,
        ability_index: 0,
        target: Some(Target::Permanent(legend)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(!bands(&g, legend));
}

/// Tolaria's band-hosing tap only works during an upkeep step.
#[test]
fn tolaria_hoses_bands_only_at_upkeep() {
    let mut g = main_phase();
    let tolaria = g.add_card_to_battlefield(0, catalog::tolaria());
    let legend = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tolaria,
            ability_index: 1,
            target: Some(Target::Permanent(legend)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "not an upkeep step"
    );
}

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// The Kobold lords stack on each other, and skip themselves.
#[test]
fn the_kobold_lords_pump_each_other_but_not_themselves() {
    let mut g = main_phase();
    let kobold = g.add_card_to_battlefield(0, catalog::crimson_kobolds());
    let taskmaster = g.add_card_to_battlefield(0, catalog::kobold_taskmaster());
    g.add_card_to_battlefield(0, catalog::kobold_drill_sergeant());
    g.add_card_to_battlefield(0, catalog::kobold_overlord());
    let c = g.computed_permanent(kobold).unwrap();
    assert_eq!((c.power, c.toughness), (1, 2), "0/1 plus +1/+0 and +0/+1");
    assert!(c.keywords.contains(&Keyword::Trample));
    assert!(c.keywords.contains(&Keyword::FirstStrike));
    assert_eq!(
        g.computed_permanent(taskmaster).unwrap().power,
        1,
        "the Taskmaster doesn't pump itself"
    );
}

/// Free Kobolds are red despite having no mana cost.
#[test]
fn kobolds_are_red_with_no_mana_cost() {
    let def = catalog::kobolds_of_kher_keep();
    assert_eq!(def.cost.cmc(), 0);
    assert_eq!(def.printed_colors(), vec![Color::Red]);
}

/// Divine Offering pays back the artifact's mana value.
#[test]
fn divine_offering_refunds_the_mana_value() {
    let mut g = main_phase();
    let relic = g.add_card_to_battlefield(1, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::divine_offering());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(relic)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(relic).is_none());
    assert_eq!(g.players[0].life, 22);
}

/// Remove Soul only answers creature spells.
#[test]
fn remove_soul_only_counters_creatures() {
    let mut g = main_phase();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bear");
    let counter = g.add_card_to_hand(1, catalog::remove_soul());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Gaseous Form takes the creature out of combat entirely.
#[test]
fn gaseous_form_blanks_combat_damage() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_earth());
    g.clear_sickness(bear);
    let form = g.add_card_to_hand(0, catalog::gaseous_form());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: form,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        crabomination::game::types::Attack {
            attacker: bear,
            target: crabomination::game::types::AttackTarget::Player(1),
        },
    ]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 0, "no damage either way");
}

/// Immolation trades toughness for power — enough to kill a 2/2 outright.
#[test]
fn immolation_swings_the_stats() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::immolation());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "a 2/2 at +2/-2 is 4/0 and dies to SBA");
}

/// Amrou Kithkin walks past anything big.
#[test]
fn amrou_kithkin_dodges_big_blockers() {
    let def = catalog::amrou_kithkin();
    assert!(def.keywords.contains(&Keyword::CantBeBlockedByPowerAtLeast(3)));
}

/// The colour-shift cycle repaints a creature for the turn.
#[test]
fn touch_of_darkness_repaints_a_creature() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().colors, vec![Color::Green]);
    let spell = g.add_card_to_hand(0, catalog::touch_of_darkness());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().colors, vec![Color::Black]);
    g.do_cleanup(&mut vec![]);
    assert_eq!(g.computed_permanent(bear).unwrap().colors, vec![Color::Green]);
}

/// Transmutation flips a creature's stats for the turn.
#[test]
fn transmutation_switches_power_and_toughness() {
    let mut g = main_phase();
    let snake = g.add_card_to_battlefield(1, catalog::hornet_cobra()); // 2/1
    let spell = g.add_card_to_hand(0, catalog::transmutation());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(snake)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let c = g.computed_permanent(snake).unwrap();
    assert_eq!((c.power, c.toughness), (1, 2));
}
