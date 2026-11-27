//! Sulu, Heroic Helmsman's, Commander Spock's and Seven of Nine's decks
//! (`decks::cmdr_01wr6s_a`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// CR 702.122 — Sulu crews as though his power were tripled: his 2 power
/// crews Marshals' Pathcruiser's Crew 5 alone, where Grizzly Bears can't.
#[test]
fn cr_702_122_sulu_crews_with_tripled_power() {
    let mut g = pod(2);
    let bears = ready(&mut g, catalog::grizzly_bears());
    let car = g.add_card_to_battlefield(0, catalog::marshals_pathcruiser());
    assert!(g.perform_action(GameAction::Crew { vehicle: car, crew_creatures: vec![bears] }).is_err());
    let sulu = ready(&mut g, catalog::sulu_heroic_helmsman());
    g.perform_action(GameAction::Crew { vehicle: car, crew_creatures: vec![sulu] }).expect("Sulu crews 5");
    let cp = g.computed_permanent(car).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
}

/// CR 702.184a — Sulu stations as though his power were tripled: six charge
/// counters from a 2-power Sulu.
#[test]
fn cr_702_184a_sulu_stations_with_tripled_power() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ship = g.add_card_to_battlefield(0, catalog::uss_enterprise_d_galaxy_class());
    ready(&mut g, catalog::sulu_heroic_helmsman());
    activate(&mut g, ship, None).expect("station");
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 6);
}

/// DOT-7 Repair Squad untaps a creature once each turn.
#[test]
fn dot_7_untaps_once_each_turn() {
    let mut g = pod(2);
    let dot = ready(&mut g, catalog::dot_7_repair_squad());
    let bears = ready(&mut g, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    activate(&mut g, dot, Some(Target::Permanent(bears))).expect("untap");
    assert!(!g.battlefield_find(bears).unwrap().tapped);
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    assert!(activate(&mut g, dot, Some(Target::Permanent(bears))).is_err(), "only once each turn");
}

/// Commander Spock enters with three differently named lands: a revealed
/// 2-drop is cast free (so nothing is drawn), a revealed 5-drop stays on top
/// and is drawn, and a revealed land enters tapped (nothing drawn).
#[test]
fn commander_spock_casts_deploys_or_draws_the_top_card() {
    for (top, cast_free, drawn) in [
        (catalog::grizzly_bears(), true, false),
        (catalog::shivan_dragon(), false, true),
        (catalog::swamp(), true, false),
    ] {
        let mut g = pod(2);
        for land in [catalog::forest(), catalog::plains(), catalog::island()] {
            g.add_card_to_battlefield(0, land);
        }
        let top = g.add_card_to_library(0, top);
        g.add_card_to_library(0, catalog::mountain());
        let spock = g.add_card_to_hand(0, catalog::commander_spock());
        cast(&mut g, spock, None, None);
        let on_bf = g.battlefield_find(top);
        assert_eq!(on_bf.is_some(), cast_free, "{top:?} put onto the battlefield");
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == top), drawn);
        if let Some(c) = on_bf.filter(|c| c.definition.is_land()) {
            assert!(c.tapped, "the land enters tapped");
        }
    }
}

/// Captain Kirk's enters trigger takes a basic land from the graveyard (no
/// library search); Spacecraft creatures get +3/+3.
#[test]
fn captain_kirk_fetches_a_basic_from_the_graveyard() {
    let mut g = pod(2);
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    let kirk = g.add_card_to_hand(0, catalog::captain_kirk_boldly_going());
    let ship = g.add_card_to_battlefield(0, catalog::uss_enterprise_d_galaxy_class());
    g.battlefield_find_mut(ship).unwrap().add_counters(CounterType::Charge, 7);
    cast(&mut g, kirk, None, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
    let cp = g.computed_permanent(ship).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 8), "a Spacecraft creature gets +3/+3");
}

/// CR 700.2 — choosing Resistance Is Futile's mode is facing a dilemma:
/// Seven of Nine draws, Melkotian Adjudicator gets a counter.
#[test]
fn cr_700_2_seven_of_nine_and_adjudicator_hear_a_modal_spell() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    ready(&mut g, catalog::seven_of_nine());
    let adj = ready(&mut g, catalog::melkotian_adjudicator());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::resistance_is_futile());
    let hand = g.players[0].hand.len();
    cast(&mut g, spell, Some(Target::Permanent(bears)), Some(0));
    assert!(g.battlefield_find(bears).is_none(), "destroyed");
    assert_eq!(g.players[0].hand.len(), hand, "spell left, one card drawn");
    assert_eq!(g.battlefield_find(adj).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Resistance Is Futile's second mode returns an artifact creature card with
/// mana value 3 or less.
#[test]
fn resistance_is_futile_reanimates_a_small_artifact_creature() {
    let mut g = pod(2);
    let thopter = g.add_card_to_graveyard(0, catalog::ornithopter());
    let spell = g.add_card_to_hand(0, catalog::resistance_is_futile());
    cast(&mut g, spell, Some(Target::Permanent(thopter)), Some(1));
    assert!(g.battlefield_find(thopter).is_some());
}

/// Tell Me What You See: the opponent either has you draw or takes 4.
#[test]
fn tell_me_what_you_see_punishes_or_draws() {
    let mut g = pod(3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ench = g.add_card_to_hand(0, catalog::tell_me_what_you_see());
    let hand = g.players[0].hand.len();
    cast(&mut g, ench, Some(Target::Player(1)), None);
    let drew = g.players[0].hand.len() == hand;
    let lost = g.players[1].life == g.players[1].starting_life - 4;
    assert!(drew != lost, "exactly one branch: drew {drew}, lost {lost}");
    assert_eq!(g.players[2].life, g.players[2].starting_life, "only the target");
}

/// CR 119.4b — under Karn's Sylex, Greed's "pay 2 life" can't be paid, but
/// Mana Confluence's mana ability still can; {X}, {T}, exile it destroys
/// each nonland permanent with mana value X or less.
#[test]
fn cr_119_4b_karns_sylex_bans_nonmana_life_costs_then_sweeps() {
    let mut g = pod(2);
    let sylex = g.add_card_to_battlefield(0, catalog::karns_sylex());
    let greed = g.add_card_to_battlefield(0, catalog::greed());
    let confluence = g.add_card_to_battlefield(0, catalog::mana_confluence());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[0].life;
    assert!(activate(&mut g, greed, None).is_err(), "Greed's life cost is banned");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: confluence,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("a mana ability may still pay life");
    assert_eq!(g.players[0].life, life - 1);

    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    g.battlefield_find_mut(sylex).unwrap().tapped = false;
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sylex,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(4),
        mode: None,
    })
    .expect("sweep");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == sylex), "exiled as a cost");
    assert!(g.battlefield_find(bears).is_none() && g.battlefield_find(greed).is_none());
    assert!(g.battlefield_find(dragon).is_some(), "mana value 6 survives X=4");
    assert!(g.battlefield_find(confluence).is_some(), "lands survive");
}

/// Kirk, Man of Action pumps other Officers and Spacecraft creatures, not
/// other creatures or himself.
#[test]
fn kirk_man_of_action_pumps_his_crew() {
    let mut g = pod(2);
    let kirk = g.add_card_to_battlefield(0, catalog::kirk_man_of_action());
    let officer = g.add_card_to_battlefield(0, catalog::tactical_officer());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pt = |g: &GameState, id| {
        let c = g.computed_permanent(id).unwrap();
        (c.power, c.toughness)
    };
    assert_eq!(pt(&g, officer), (4, 3));
    assert_eq!(pt(&g, bears), (2, 2));
    assert_eq!(pt(&g, kirk), (3, 3));
}

/// Fulgent Distraction taps both targets and knocks their Equipment off.
#[test]
fn fulgent_distraction_taps_and_unequips_two_creatures() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let sword = g.add_card_to_battlefield(1, catalog::bonesplitter());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(b);
    let spell = g.add_card_to_hand(0, catalog::fulgent_distraction());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    assert_eq!(g.battlefield_find(sword).unwrap().attached_to, None);
}

/// D'Ghor Bloodletter's second mode: each of four players sacrifices a
/// creature (the Bloodletter is its controller's only one).
#[test]
fn dghor_bloodletter_edicts_every_seat() {
    let mut g = pod(4);
    let victims: Vec<_> = (1..4).map(|s| g.add_card_to_battlefield(s, catalog::grizzly_bears())).collect();
    let dg = g.add_card_to_hand(0, catalog::dghor_bloodletter());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: dg, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    // The ETB's mode is picked as the trigger goes on the stack: answer 1.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Mode(1)]));
    drain_stack(&mut g);
    assert!(victims.iter().all(|&v| g.battlefield_find(v).is_none()), "every opponent sacrificed");
    assert!(g.battlefield_find(dg).is_none(), "its controller sacrificed it");
}

/// Collector's Case stuns its ETB target: tapped with two stun counters.
#[test]
fn collectors_case_stuns_on_entry() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let case = g.add_card_to_hand(0, catalog::collectors_case());
    cast(&mut g, case, None, None);
    let c = g.battlefield_find(bears).unwrap();
    assert!(c.tapped);
    assert_eq!(c.counter_count(CounterType::Stun), 2);
}

/// CR 700.2 — Gabriel Lorca hears a dilemma from the graveyard: pay {B} and
/// he returns to hand.
#[test]
fn cr_700_2_gabriel_lorca_returns_from_the_graveyard_on_a_dilemma() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let lorca = g.add_card_to_graveyard(0, catalog::gabriel_lorca_mirror_menace());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::resistance_is_futile());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, spell, Some(Target::Permanent(bears)), Some(0));
    assert!(g.players[0].hand.iter().any(|c| c.id == lorca), "paid {{B}}, back in hand");
}

/// Overflow Submatrix: one artifact creature cast from the graveyard each
/// of your turns, entering with a finality counter.
#[test]
fn overflow_submatrix_casts_from_the_graveyard_with_finality() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::overflow_submatrix());
    let a = g.add_card_to_graveyard(0, catalog::ornithopter());
    let b = g.add_card_to_graveyard(0, catalog::memnite());
    cast(&mut g, a, None, None);
    let c = g.battlefield_find(a).expect("cast from the graveyard");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
    g.priority.player_with_priority = 0;
    let again = g.perform_action(GameAction::CastSpell { card_id: b, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(again.is_err(), "once each turn");
}

/// Collective Drone's power counts the artifact creatures you control
/// (itself included); toughness stays 2.
#[test]
fn collective_drone_counts_artifact_creatures() {
    let mut g = pod(2);
    let drone = g.add_card_to_battlefield(0, catalog::collective_drone());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::ornithopter());
    let c = g.computed_permanent(drone).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2));
}

/// Captain Benjamin Sisko's activation casts the exiled spells whose mana
/// value is at most his Officers (two here): Grizzly Bears, not Shivan Dragon.
#[test]
fn captain_sisko_casts_exiled_spells_up_to_his_officer_count() {
    let mut g = pod(2);
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let dragon = g.add_card_to_library(0, catalog::shivan_dragon());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let sisko = ready(&mut g, catalog::captain_benjamin_sisko());
    g.add_card_to_battlefield(0, catalog::tactical_officer());
    activate(&mut g, sisko, None).expect("activate");
    assert!(g.battlefield_find(bears).is_some(), "MV 2 cast free");
    assert!(g.exile.iter().any(|c| c.id == dragon), "MV 6 stays exiled");
}

/// Sickbay: a creature entering gains 1 life, then at the beginning of combat
/// its target gets 1 + (life gained this turn) +1/+1 counters.
#[test]
fn sickbay_scales_its_combat_counters_with_life_gained() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sickbay());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    cast(&mut g, bears, None, None);
    assert_eq!(g.players[0].life, life + 1);
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Vengeance Mission: each creature death drains its target opponent by 1
/// and adds a quest counter; at seven counters the drain is 2.
#[test]
fn vengeance_mission_drains_per_death_and_steps_up_at_seven() {
    let mut g = pod(3);
    let vm = g.add_card_to_battlefield(0, catalog::vengeance_mission());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let before: i32 = (1..3).map(|s| g.players[s].life).sum();
    cast(&mut g, bolt, Some(Target::Permanent(bears)), None);
    assert_eq!(g.battlefield_find(vm).unwrap().counter_count(CounterType::Quest), 1);
    assert_eq!((1..3).map(|s| g.players[s].life).sum::<i32>(), before - 1, "one opponent lost 1");
    g.battlefield_find_mut(vm).unwrap().add_counters(CounterType::Quest, 5);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bears)), None);
    assert_eq!((1..3).map(|s| g.players[s].life).sum::<i32>(), before - 3, "the seventh counter drains 2");
}

/// Hapatra's Mark strips every -1/-1 counter from its target.
#[test]
fn hapatras_mark_removes_all_minus_counters() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(giant).unwrap().add_counters(CounterType::MinusOneMinusOne, 2);
    let mark = g.add_card_to_hand(0, catalog::hapatras_mark());
    cast(&mut g, mark, Some(Target::Permanent(giant)), None);
    assert_eq!(g.battlefield_find(giant).unwrap().counter_count(CounterType::MinusOneMinusOne), 0);
}

/// The Destruction of Praxis: -X/-X with X = differently named lands; at six
/// names it also makes two hasty Klingon Warriors.
#[test]
fn destruction_of_praxis_scales_with_land_names() {
    let mut g = pod(2);
    for land in [catalog::forest(), catalog::plains(), catalog::island(), catalog::swamp(), catalog::mountain()] {
        g.add_card_to_battlefield(0, land);
    }
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let praxis = g.add_card_to_hand(0, catalog::the_destruction_of_praxis());
    cast(&mut g, praxis, None, None);
    assert!(g.battlefield_find(giant).is_none(), "3/3 under -5/-5");
    assert!(g.battlefield_find(dragon).is_none(), "5/5 under -5/-5");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Klingon Warrior"), "five names: no tokens");

    let mut g = pod(2);
    for land in [catalog::forest(), catalog::plains(), catalog::island(), catalog::swamp(), catalog::mountain(), catalog::qonos()] {
        g.add_card_to_battlefield(0, land);
    }
    let praxis = g.add_card_to_hand(0, catalog::the_destruction_of_praxis());
    cast(&mut g, praxis, None, None);
    let warriors = g.battlefield.iter().filter(|c| c.definition.name == "Klingon Warrior").count();
    assert_eq!(warriors, 2, "six names: two tokens, made after the -6/-6");
}

/// Red Alert! costs {2} less while a creature attacks you: {R} alone casts
/// it then.
#[test]
fn red_alert_is_cheap_while_you_are_attacked() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = pod(2);
    let spell = g.add_card_to_hand(0, catalog::red_alert());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "full price unpaid"
    );
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]))
        .expect("attack");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{R} is enough while attacked");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4);
}

/// Khaaaaaaaaaaaannn! deals twice X and exiles the creature it kills.
#[test]
fn khaaan_deals_twice_x_and_exiles_the_dead() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::khaaaaaaaaaaaannn());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == giant), "4 damage kills the 3/3, which is exiled");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == giant));
}

/// Tal Shiar Analyst pings each opponent and untaps on a noncreature cast.
#[test]
fn tal_shiar_analyst_pings_and_untaps_on_noncreature_spells() {
    let mut g = pod(3);
    let tsa = ready(&mut g, catalog::tal_shiar_analyst());
    activate(&mut g, tsa, None).expect("ping");
    assert!((1..3).all(|s| g.players[s].life == g.players[s].starting_life - 1));
    assert!(g.battlefield_find(tsa).unwrap().tapped);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)), None);
    assert!(!g.battlefield_find(tsa).unwrap().tapped, "untapped by a noncreature spell");
}
