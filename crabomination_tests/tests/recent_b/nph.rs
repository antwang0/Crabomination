//! New Phyrexia (NPH) — Phyrexian mana, Infect, Living weapon and the Shrine
//! cycle (`catalog::sets::nph`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
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

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Every NPH body ships with its printed stats.
#[test]
fn nph_bodies_have_their_printed_stats() {
    for (def, p, t) in [
        (catalog::lost_leonin(), 2, 1),
        (catalog::slash_panther(), 4, 2),
        (catalog::thundering_tanadon(), 5, 4),
        (catalog::spinebiter(), 3, 4),
        (catalog::jor_kadeen_the_prevailer(), 5, 4),
    ] {
        assert_eq!((def.power, def.toughness), (p, t), "{}", def.name);
    }
}

/// Phyrexian mana is payable with life when the colour is missing.
#[test]
fn phyrexian_mana_can_be_paid_with_life() {
    let mut g = main_phase();
    let panther = g.add_card_to_hand(0, catalog::slash_panther());
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: panther,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast paying 2 life for {R/P}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18);
    assert!(g.battlefield_find(panther).is_some());
}

/// A Shrine ticks up at upkeep and on a matching spell, then cashes out.
#[test]
fn shrine_of_burning_rage_ticks_and_burns() {
    let mut g = main_phase();
    let shrine = g.add_card_to_battlefield(0, catalog::shrine_of_burning_rage());
    let bolt = g.add_card_to_hand(0, catalog::whipflare());
    cast(&mut g, 0, bolt, None);
    assert_eq!(
        g.battlefield_find(shrine).unwrap().counter_count(CounterType::Charge),
        1,
        "the red spell put a charge counter on"
    );
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(shrine).unwrap().counter_count(CounterType::Charge), 2);
    activate(&mut g, 0, shrine, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
}

/// Shrine of Loyal Legions trades its pile for that many Myr.
#[test]
fn shrine_of_loyal_legions_mints_a_myr_per_counter() {
    let mut g = main_phase();
    let shrine = g.add_card_to_battlefield(0, catalog::shrine_of_loyal_legions());
    g.battlefield_find_mut(shrine).unwrap().add_counters(CounterType::Charge, 3);
    activate(&mut g, 0, shrine, 0, None);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Myr").count(),
        3
    );
}

/// Living weapon mints a Germ and straps the Equipment onto it.
#[test]
fn sickleslicer_arrives_wearing_a_germ() {
    let mut g = main_phase();
    let slicer = g.add_card_to_hand(0, catalog::sickleslicer());
    cast(&mut g, 0, slicer, None);
    let germ = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Phyrexian Germ")
        .map(|c| c.id)
        .expect("a Germ");
    assert_eq!(g.battlefield_find(slicer).unwrap().attached_to, Some(germ));
    assert_eq!(g.computed_permanent(germ).unwrap().power, 2, "0/0 plus +2/+2");
}

/// Slag Fiend counts the artifact cards in every graveyard.
#[test]
fn slag_fiend_sizes_off_artifact_graveyards() {
    let mut g = main_phase();
    let fiend = g.add_card_to_battlefield(0, catalog::slag_fiend());
    assert_eq!(g.computed_permanent(fiend).unwrap().power, 0);
    for seat in 0..2 {
        g.add_card_to_graveyard(seat, catalog::darksteel_relic());
    }
    assert_eq!(g.computed_permanent(fiend).unwrap().power, 2);
    assert_eq!(g.computed_permanent(fiend).unwrap().toughness, 2);
}

/// Xenograft folds your whole board into the chosen tribe.
#[test]
fn xenograft_retypes_your_creatures() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let graft = g.add_card_to_hand(0, catalog::xenograft());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        CreatureType::Sliver,
    )]));
    cast(&mut g, 0, graft, None);
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .subtypes
            .creature_types
            .contains(&CreatureType::Sliver)
    );
}

/// Jor Kadeen's metalcraft anthem only fires with three artifacts out.
#[test]
fn jor_kadeen_needs_metalcraft() {
    let mut g = main_phase();
    let jor = g.add_card_to_battlefield(0, catalog::jor_kadeen_the_prevailer());
    assert_eq!(g.computed_permanent(jor).unwrap().power, 5);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::darksteel_relic());
    }
    assert_eq!(g.computed_permanent(jor).unwrap().power, 8);
}

/// Whipflare spares artifact creatures.
#[test]
fn whipflare_spares_artifact_creatures() {
    let mut g = main_phase();
    let flesh = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let metal = g.add_card_to_battlefield(1, catalog::hovermyr());
    let spell = g.add_card_to_hand(0, catalog::whipflare());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(flesh).is_none());
    assert!(g.battlefield_find(metal).is_some());
}

/// Mindcrank turns every point of life loss into a mill.
#[test]
fn mindcrank_mills_on_life_loss() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mindcrank());
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let caress = g.add_card_to_hand(0, catalog::caress_of_phyrexia());
    cast(&mut g, 0, caress, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 3);
}

/// Furnace Scamp trades itself for three damage after connecting.
#[test]
fn furnace_scamp_cashes_out_after_combat_damage() {
    let mut g = main_phase();
    let scamp = g.add_card_to_battlefield(0, catalog::furnace_scamp());
    g.clear_sickness(scamp);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: scamp,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, 16, "1 combat plus 3 from the sacrifice");
    assert!(g.battlefield_find(scamp).is_none());
}

/// Due Respect makes everything arrive tapped for the turn.
#[test]
fn due_respect_taps_the_rest_of_the_turn() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::due_respect());
    cast(&mut g, 0, spell, None);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None);
    assert!(g.battlefield_find(bear).unwrap().tapped);
    g.do_cleanup(&mut vec![]);
    assert!(!g.permanents_enter_tapped_this_turn, "cleanup clears the turn flag");
}

/// Corrupted Resolve only counters a poisoned player's spell.
#[test]
fn corrupted_resolve_needs_a_poisoned_controller() {
    let mut g = main_phase();
    g.active_player_idx = 1;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let counter = g.add_card_to_hand(0, catalog::corrupted_resolve());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bear");
    cast(&mut g, 0, counter, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_some(), "not poisoned, so it resolved");

    g.players[1].poison_counters = 1;
    let bear2 = g.add_card_to_hand(1, catalog::grizzly_bears());
    let counter2 = g.add_card_to_hand(0, catalog::corrupted_resolve());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bear");
    cast(&mut g, 0, counter2, Some(Target::Permanent(bear2)));
    assert!(g.battlefield_find(bear2).is_none(), "poisoned, so it was countered");
}

/// Unwinding Clock untaps your artifacts on the opponent's untap step.
#[test]
fn unwinding_clock_untaps_off_turn() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::unwinding_clock());
    let relic = g.add_card_to_battlefield(0, catalog::darksteel_relic());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [relic, bear] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    g.do_untap();
    assert!(!g.battlefield_find(relic).unwrap().tapped);
    assert!(g.battlefield_find(bear).unwrap().tapped, "only artifacts");
}

/// Surge Node walks its charge counters onto another artifact.
#[test]
fn surge_node_moves_a_charge_counter() {
    let mut g = main_phase();
    let node_card = g.add_card_to_hand(0, catalog::surge_node());
    cast(&mut g, 0, node_card, None);
    let node = node_card;
    let shrine = g.add_card_to_battlefield(0, catalog::shrine_of_burning_rage());
    g.clear_sickness(node);
    assert_eq!(g.battlefield_find(node).unwrap().counter_count(CounterType::Charge), 6);
    activate(&mut g, 0, node, 0, Some(Target::Permanent(shrine)));
    assert_eq!(g.battlefield_find(node).unwrap().counter_count(CounterType::Charge), 5);
    assert_eq!(g.battlefield_find(shrine).unwrap().counter_count(CounterType::Charge), 1);
}

/// Viral Drake proliferates for {3}{U}.
#[test]
fn viral_drake_proliferates() {
    let mut g = main_phase();
    let drake = g.add_card_to_battlefield(0, catalog::viral_drake());
    g.players[1].poison_counters = 2;
    activate(&mut g, 0, drake, 0, None);
    assert_eq!(g.players[1].poison_counters, 3);
}

/// Vital Splicer brings a Golem and can regenerate it.
#[test]
fn vital_splicer_makes_and_protects_a_golem() {
    let mut g = main_phase();
    let splicer = g.add_card_to_hand(0, catalog::vital_splicer());
    cast(&mut g, 0, splicer, None);
    let golem = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Phyrexian Golem")
        .map(|c| c.id)
        .expect("a Golem");
    let splicer_id = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Vital Splicer")
        .map(|c| c.id)
        .expect("the Splicer");
    activate(&mut g, 0, splicer_id, 0, Some(Target::Permanent(golem)));
    assert!(g.battlefield_find(golem).unwrap().regeneration_shields > 0);
}

/// Glistening Oil hands out infect and eats its host a counter at a time.
#[test]
fn glistening_oil_grants_infect_and_shrinks() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let oil = g.add_card_to_hand(0, catalog::glistening_oil());
    cast(&mut g, 0, oil, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Infect));
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
}

/// Isolation Cell taxes their creature spells two life if they won't pay {2}.
#[test]
fn isolation_cell_taxes_creature_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::isolation_cell());
    g.active_player_idx = 1;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    cast(&mut g, 1, bear, None);
    assert_eq!(g.players[1].life, 18);
}

/// Norn's Annex taxes attacks against you.
#[test]
fn norns_annex_taxes_attackers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::norns_annex());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "no mana to pay the tax"
    );
}

/// Fresh Meat pays out one 3/3 per creature you lost this turn.
#[test]
fn fresh_meat_counts_the_days_dead() {
    let mut g = main_phase();
    for _ in 0..2 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut events = vec![];
        g.destroy_permanent(bear, false, &mut events);
    }
    let spell = g.add_card_to_hand(0, catalog::fresh_meat());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Beast").count(), 2);
}

/// Mycosynth Wellspring fetches a basic on the way in and on the way out.
#[test]
fn mycosynth_wellspring_fetches_twice() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let well = g.add_card_to_hand(0, catalog::mycosynth_wellspring());
    let forest = g.players[0].library[0].id;
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));
    cast(&mut g, 0, well, None);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"));
}

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Mycosynth Fiend grows with every poison counter you have handed out.
#[test]
fn mycosynth_fiend_grows_with_opposing_poison() {
    let mut g = main_phase();
    let fiend = g.add_card_to_battlefield(0, catalog::mycosynth_fiend());
    assert_eq!(g.computed_permanent(fiend).unwrap().power, 2);
    g.players[1].poison_counters = 4;
    assert_eq!(g.computed_permanent(fiend).unwrap().power, 6);
    assert_eq!(g.computed_permanent(fiend).unwrap().toughness, 6);
}

/// Viridian Betrayers only has infect while an opponent is poisoned.
#[test]
fn viridian_betrayers_needs_a_poisoned_opponent() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::viridian_betrayers());
    assert!(!g.computed_permanent(elf).unwrap().keywords.contains(&Keyword::Infect));
    g.players[1].poison_counters = 1;
    assert!(g.computed_permanent(elf).unwrap().keywords.contains(&Keyword::Infect));
}

/// Phyrexian Swarmlord mints an infect Insect per opposing poison counter.
#[test]
fn phyrexian_swarmlord_mints_one_insect_per_poison() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::phyrexian_swarmlord());
    g.players[1].poison_counters = 3;
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Insect").count(),
        3
    );
}

/// Greenhilt Trainee's pump is gated on its own power.
#[test]
fn greenhilt_trainee_needs_four_power() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::greenhilt_trainee());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(elf);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: elf,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a 2/3 can't activate"
    );
    g.battlefield_find_mut(elf).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    activate(&mut g, 0, elf, 0, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6);
}

/// Ichor Explosion shrinks the board by the sacrificed creature's power.
#[test]
fn ichor_explosion_scales_off_the_sacrifice() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::flameborn_viron()); // 6/4 fodder
    let victim = g.add_card_to_battlefield(1, catalog::rotted_hystrix()); // 3/6
    let spell = g.add_card_to_hand(0, catalog::ichor_explosion());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(victim).is_none(), "-6/-6 killed the 3/6");
}

/// Act of Aggression steals, untaps and hastes for the turn.
#[test]
fn act_of_aggression_steals_a_tapped_creature() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::act_of_aggression());
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.controller, 0);
    assert!(!c.tapped);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Phyrexian Ingester wears the imprinted creature's stats.
#[test]
fn phyrexian_ingester_wears_what_it_swallowed() {
    let mut g = main_phase();
    let prey = g.add_card_to_battlefield(1, catalog::flameborn_viron()); // 6/4
    let ingester = g.add_card_to_hand(0, catalog::phyrexian_ingester());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, ingester, Some(Target::Permanent(prey)));
    assert!(g.battlefield_find(prey).is_none(), "exiled");
    assert_eq!(g.computed_permanent(ingester).unwrap().power, 9, "3 + 6");
    assert_eq!(g.computed_permanent(ingester).unwrap().toughness, 7, "3 + 4");
}

/// Lashwrithe scales with your Swamps.
#[test]
fn lashwrithe_scales_with_swamps() {
    let mut g = main_phase();
    let lash = g.add_card_to_hand(0, catalog::lashwrithe());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    cast(&mut g, 0, lash, None);
    let germ = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Phyrexian Germ")
        .map(|c| c.id)
        .expect("a Germ");
    assert_eq!(g.computed_permanent(germ).unwrap().power, 3);
}

/// Cathedral Membrane takes its blockers with it.
#[test]
fn cathedral_membrane_kills_what_it_blocked() {
    let mut g = main_phase();
    g.active_player_idx = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::flameborn_viron()); // 6/4
    let wall = g.add_card_to_battlefield(0, catalog::cathedral_membrane());
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, attacker)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(attacker).is_none(), "6 damage back");
}

/// Conversion Chamber banks artifact cards and spends them on Golems.
#[test]
fn conversion_chamber_banks_then_builds() {
    let mut g = main_phase();
    let chamber = g.add_card_to_battlefield(0, catalog::conversion_chamber());
    let relic = g.add_card_to_graveyard(0, catalog::darksteel_relic());
    g.clear_sickness(chamber);
    activate(&mut g, 0, chamber, 0, Some(Target::Permanent(relic)));
    assert_eq!(g.battlefield_find(chamber).unwrap().counter_count(CounterType::Charge), 1);
    g.battlefield_find_mut(chamber).unwrap().tapped = false;
    activate(&mut g, 0, chamber, 1, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Golem"));
}

/// Etched Monstrosity arrives crushed and pays out when freed.
#[test]
fn etched_monstrosity_starts_at_five_five() {
    let mut g = main_phase();
    let golem = g.add_card_to_hand(0, catalog::etched_monstrosity());
    cast(&mut g, 0, golem, None);
    assert_eq!(g.computed_permanent(golem).unwrap().power, 5, "10 minus five counters");
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let hand_before = g.players[0].hand.len();
    activate(&mut g, 0, golem, 0, Some(Target::Player(0)));
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
    assert_eq!(g.computed_permanent(golem).unwrap().power, 10);
}

/// Praetor's Grasp exiles from their deck and leaves it playable for you.
#[test]
fn praetors_grasp_steals_a_card_out_of_the_deck() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let pick = g.players[1].library[0].id;
    let spell = g.add_card_to_hand(0, catalog::praetors_grasp());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(pick))]));
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert!(g.exile.iter().any(|c| c.id == pick), "exiled out of their library");
}

/// Parasitic Implant eats its host and pays you a Myr.
#[test]
fn parasitic_implant_eats_its_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let implant = g.add_card_to_hand(0, catalog::parasitic_implant());
    cast(&mut g, 0, implant, Some(Target::Permanent(bear)));
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Phyrexian Myr"));
}

/// Exclusion Ritual locks the exiled card's name out.
#[test]
fn exclusion_ritual_locks_the_name() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ritual = g.add_card_to_hand(0, catalog::exclusion_ritual());
    cast(&mut g, 0, ritual, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().any(|c| c.id == bear));
    let copy = g.add_card_to_hand(1, catalog::grizzly_bears());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: copy,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the name is locked out"
    );
}

/// Tormentor Exarch pumps or shrinks on arrival.
#[test]
fn tormentor_exarch_shrinks_on_the_second_mode() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let exarch = g.add_card_to_hand(0, catalog::tormentor_exarch());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(1)]));
    cast(&mut g, 0, exarch, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "a 2/2 at -0/-2 dies to SBA");
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// A revealed Chancellor of the Dross drains three at the first upkeep.
#[test]
fn chancellor_of_the_dross_drains_from_the_opening_hand() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::chancellor_of_the_dross());
    g.fire_start_of_game_effects();
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[0].life, 23);
}

/// Chancellor of the Forge's ETB mints a Goblin per creature you control.
#[test]
fn chancellor_of_the_forge_swarms_on_arrival() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let chancellor = g.add_card_to_hand(0, catalog::chancellor_of_the_forge());
    cast(&mut g, 0, chancellor, None);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Goblin").count(),
        2,
        "the Bear plus the Chancellor itself"
    );
}

/// Chancellor of the Spires mills seven off the opening-hand reveal.
#[test]
fn chancellor_of_the_spires_mills_from_the_opening_hand() {
    let mut g = main_phase();
    for _ in 0..10 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    g.add_card_to_hand(0, catalog::chancellor_of_the_spires());
    g.fire_start_of_game_effects();
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 7);
}

/// Hex Parasite eats counters and grows by however many it removed.
#[test]
fn hex_parasite_grows_by_the_counters_it_ate() {
    let mut g = main_phase();
    let parasite = g.add_card_to_battlefield(0, catalog::hex_parasite());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: parasite,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.computed_permanent(parasite).unwrap().power, 4, "1 + 3 removed");
}

/// Rage Extractor only fires on spells with a Phyrexian symbol in their cost.
#[test]
fn rage_extractor_fires_on_phyrexian_costs() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::rage_extractor());
    let plain = g.add_card_to_hand(0, catalog::whipflare());
    cast(&mut g, 0, plain, None);
    assert_eq!(g.players[1].life, 20, "a plain red cost has no Phyrexian symbol");
    let phyrexian = g.add_card_to_hand(0, catalog::slash_panther());
    cast(&mut g, 0, phyrexian, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 15, "mana value 5");
}

/// Invader Parasite burns whoever plays a land matching its imprint.
#[test]
fn invader_parasite_punishes_the_named_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let parasite = g.add_card_to_hand(0, catalog::invader_parasite());
    cast(&mut g, 0, parasite, Some(Target::Permanent(land)));
    assert!(g.exile.iter().any(|c| c.id == land));
    let second = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(second)).expect("land drop");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Omen Machine stops the draw step outright.
#[test]
fn omen_machine_stops_draws() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::omen_machine());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].hand.clear();
    g.step = TurnStep::Upkeep;
    let _ = g.advance_step(Vec::new());
    assert_eq!(g.step, TurnStep::Draw);
    assert!(g.players[0].hand.is_empty(), "no turn-based draw");
}
