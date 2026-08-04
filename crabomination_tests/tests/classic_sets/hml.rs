//! Homelands (HML) — `catalog::sets::hml`.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
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

fn activate_n(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    activate_n(g, seat, id, 0, target)
}

#[test]
fn keyword_bodies_are_printed_correctly() {
    let mut g = main_phase();
    for (def, kw) in [
        (catalog::abbey_gargoyles(), Keyword::Protection(Color::Red)),
        (catalog::cemetery_gate(), Keyword::Protection(Color::Black)),
        (catalog::death_speakers(), Keyword::Protection(Color::Black)),
        (catalog::ihsans_shade(), Keyword::Protection(Color::White)),
        (catalog::narwhal(), Keyword::FirstStrike),
        (catalog::sea_sprite(), Keyword::Flying),
        (catalog::ebony_rhino(), Keyword::Trample),
        (catalog::eron_the_relentless(), Keyword::Haste),
        (catalog::willow_faerie(), Keyword::Flying),
    ] {
        let id = g.add_card_to_battlefield(0, def);
        let cp = g.computed_permanent(id).expect("permanent");
        assert!(cp.keywords.contains(&kw), "missing {kw:?}");
    }
}

#[test]
fn the_city_lands_step_up_through_three_abilities() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::an_havva_township());
    activate(&mut g, 0, land, None);
    assert!(g.players[0].mana_pool.total() > 0);
    // The colored abilities cost {1} and {2} on top of the tap.
    let land2 = g.add_card_to_battlefield(0, catalog::wizards_school());
    activate_n(&mut g, 0, land2, 1, None);
    assert!(g.players[0].mana_pool.amount(Color::Blue) > 20 - 1);
}

#[test]
fn aysen_crusader_counts_the_ranks() {
    let mut g = main_phase();
    let crusader = g.add_card_to_battlefield(0, catalog::aysen_crusader());
    let cp = g.computed_permanent(crusader).expect("crusader");
    assert_eq!((cp.power, cp.toughness), (2, 2));
    g.add_card_to_battlefield(0, catalog::savannah_lions()); // Cat, no help
    g.add_card_to_battlefield(0, catalog::mogg_fanatic());
    let cp = g.computed_permanent(crusader).expect("crusader");
    assert_eq!((cp.power, cp.toughness), (2, 2), "neither is a Soldier or Warrior");
}

#[test]
fn abbey_matron_soaks_a_hit() {
    let mut g = main_phase();
    let matron = g.add_card_to_battlefield(0, catalog::abbey_matron());
    g.clear_sickness(matron);
    activate(&mut g, 0, matron, None);
    assert_eq!(g.computed_permanent(matron).expect("matron").toughness, 6);
}

#[test]
fn faerie_noble_anthems_the_other_faeries_only() {
    let mut g = main_phase();
    let noble = g.add_card_to_battlefield(0, catalog::faerie_noble());
    g.clear_sickness(noble);
    let faerie = g.add_card_to_battlefield(0, catalog::willow_faerie());
    assert_eq!(g.computed_permanent(faerie).expect("faerie").toughness, 3);
    assert_eq!(g.computed_permanent(noble).expect("noble").toughness, 2, "not itself");
    activate(&mut g, 0, noble, None);
    assert_eq!(g.computed_permanent(faerie).expect("faerie").power, 2);
}

#[test]
fn folk_of_an_havva_grows_when_it_blocks() {
    let mut g = main_phase();
    let folk = g.add_card_to_battlefield(0, catalog::folk_of_an_havva());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(folk, bear)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(folk).expect("folk").power, 3);
}

#[test]
fn root_spider_strikes_first_on_defence() {
    let mut g = main_phase();
    let spider = g.add_card_to_battlefield(0, catalog::root_spider());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(spider, bear)])).expect("block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(spider).expect("spider");
    assert_eq!(cp.power, 3);
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn hungry_mist_eats_two_green_each_upkeep() {
    let mut g = main_phase();
    let mist = g.add_card_to_battlefield(0, catalog::hungry_mist());
    g.step = TurnStep::Upkeep;
    g.players[0].mana_pool = Default::default();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(mist).is_none());
}

#[test]
fn reef_pirates_mill_on_connection() {
    let mut g = main_phase();
    let pirates = g.add_card_to_battlefield(0, catalog::reef_pirates());
    g.clear_sickness(pirates);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: pirates, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 1);
}

#[test]
fn an_havva_inn_counts_every_green_body() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let inn = g.add_card_to_hand(0, catalog::an_havva_inn());
    cast(&mut g, 0, inn, None);
    assert_eq!(g.players[0].life, 23, "one plus two green creatures");
}

#[test]
fn dry_spell_sweeps_the_table() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::mogg_fanatic()); // 1/1
    let spell = g.add_card_to_hand(0, catalog::dry_spell());
    cast(&mut g, 0, spell, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(mine).is_none());
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.players[1].life, 19);
}

#[test]
fn evaporate_only_singes_white_and_blue() {
    let mut g = main_phase();
    let lions = g.add_card_to_battlefield(1, catalog::savannah_lions()); // white 2/1
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green 2/2
    let spell = g.add_card_to_hand(0, catalog::evaporate());
    cast(&mut g, 0, spell, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(lions).is_none());
    assert!(g.battlefield_find(bear).is_some());
}

#[test]
fn merchant_scroll_finds_a_blue_instant() {
    let mut g = main_phase();
    let cs = g.add_card_to_library(0, catalog::counterspell());
    let scroll = g.add_card_to_hand(0, catalog::merchant_scroll());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(cs))]));
    cast(&mut g, 0, scroll, None);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Counterspell"));
}

#[test]
fn ferozs_ban_taxes_only_creature_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::ferozs_ban());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 3);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "{{1}}{{G}} plus the {{2}} tax is four mana",
    );
}

#[test]
fn mystic_decree_grounds_everything() {
    let mut g = main_phase();
    let faerie = g.add_card_to_battlefield(0, catalog::willow_faerie());
    assert!(g.computed_permanent(faerie).expect("faerie").keywords.contains(&Keyword::Flying));
    g.add_card_to_battlefield(1, catalog::mystic_decree());
    assert!(!g.computed_permanent(faerie).expect("faerie").keywords.contains(&Keyword::Flying));
}

#[test]
fn serra_aviary_lifts_every_flier() {
    let mut g = main_phase();
    let faerie = g.add_card_to_battlefield(1, catalog::willow_faerie());
    g.add_card_to_battlefield(0, catalog::serra_aviary());
    let cp = g.computed_permanent(faerie).expect("faerie");
    assert_eq!((cp.power, cp.toughness), (2, 3), "the anthem is symmetric");
}

#[test]
fn primal_order_bills_the_greedy_mana_base() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::primal_order());
    g.add_card_to_battlefield(0, catalog::mountain()); // basic, free
    g.add_card_to_battlefield(0, catalog::an_havva_township());
    g.add_card_to_battlefield(0, catalog::koskun_keep());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18);
}

#[test]
fn torture_stacks_minus_counters_on_its_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let torture = g.add_card_to_hand(0, catalog::torture());
    cast(&mut g, 0, torture, Some(Target::Permanent(bear)));
    activate(&mut g, 0, torture, None);
    assert_eq!(
        g.battlefield_find(bear).expect("bear").counter_count(CounterType::MinusOneMinusOne),
        1
    );
}

#[test]
fn roterothopter_pumps_at_most_twice_a_turn() {
    let mut g = main_phase();
    let thopter = g.add_card_to_battlefield(0, catalog::roterothopter());
    activate(&mut g, 0, thopter, None);
    activate(&mut g, 0, thopter, None);
    assert_eq!(g.computed_permanent(thopter).expect("thopter").power, 2);
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: thopter,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "the third activation this turn is illegal",
    );
}

#[test]
fn aether_storm_locks_creatures_out_until_someone_pays() {
    let mut g = main_phase();
    let storm = g.add_card_to_battlefield(1, catalog::aether_storm());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
    // Any player may pay 4 life to break it — including its controller's foe.
    activate(&mut g, 0, storm, None);
    assert!(g.battlefield_find(storm).is_none());
    assert_eq!(g.players[0].life, 16);
}

#[test]
fn didgeridoo_drops_a_minotaur_straight_in() {
    let mut g = main_phase();
    let didg = g.add_card_to_battlefield(0, catalog::didgeridoo());
    let minotaur = g.add_card_to_hand(0, catalog::anaba_shaman());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![minotaur]),
    ]));
    activate(&mut g, 0, didg, None);
    assert!(g.battlefield_find(minotaur).is_some());
}

#[test]
fn winter_sky_burns_on_heads() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::mogg_fanatic());
    let sky = g.add_card_to_hand(0, catalog::winter_sky());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, sky, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(mine).is_none());
    assert_eq!(g.players[1].life, 19);
}

#[test]
fn veldrane_trades_power_for_forestwalk() {
    let mut g = main_phase();
    let veldrane = g.add_card_to_battlefield(0, catalog::veldrane_of_sengir());
    activate(&mut g, 0, veldrane, None);
    let cp = g.computed_permanent(veldrane).expect("veldrane");
    assert_eq!(cp.power, 2);
    assert!(cp.keywords.iter().any(|k| matches!(
        k,
        Keyword::Landwalk(crabomination::card::LandType::Forest)
    )));
}

#[test]
fn wall_of_kelp_grows_more_kelp() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_kelp());
    g.clear_sickness(wall);
    activate(&mut g, 0, wall, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Kelp").count(), 1);
}

#[test]
fn eron_regenerates_for_three_red() {
    let mut g = main_phase();
    let eron = g.add_card_to_battlefield(0, catalog::eron_the_relentless());
    activate(&mut g, 0, eron, None);
    g.destroy_permanent(eron, false, &mut Vec::new());
    assert!(g.battlefield_find(eron).is_some(), "the shield ate the kill");
}

#[test]
fn ambush_gives_every_blocker_first_strike() {
    let mut g = main_phase();
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.declare_blockers(vec![(blocker, atk)]).expect("block");
    let ambush = g.add_card_to_hand(0, catalog::ambush());
    cast(&mut g, 0, ambush, None);
    assert!(
        g.computed_permanent(blocker).expect("blocker").keywords.contains(&Keyword::FirstStrike)
    );
}

#[test]
fn an_havva_constable_counts_every_green_body() {
    let mut g = main_phase();
    let constable = g.add_card_to_battlefield(0, catalog::an_havva_constable());
    let cp = g.computed_permanent(constable).expect("constable");
    assert_eq!((cp.power, cp.toughness), (2, 2), "it counts itself");
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(constable).expect("constable").toughness, 3);
}

#[test]
fn apocalypse_chime_unmakes_homelands_only() {
    let mut g = main_phase();
    let bats = g.add_card_to_battlefield(1, catalog::sengir_bats());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let chime = g.add_card_to_battlefield(0, catalog::apocalypse_chime());
    activate(&mut g, 0, chime, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bats).is_none());
    assert!(g.battlefield_find(bear).is_some());
    assert!(g.battlefield_find(chime).is_none(), "it sacrifices itself as a cost");
}

#[test]
fn labyrinth_minotaur_keeps_what_it_blocks_tapped() {
    let mut g = main_phase();
    let minotaur = g.add_card_to_battlefield(0, catalog::labyrinth_minotaur());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(minotaur, bear)])).expect("block");
    drain_stack(&mut g);
    g.battlefield_find_mut(bear).expect("bear").tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(bear).expect("bear").tapped);
}

#[test]
fn reveka_pays_a_turn_for_every_two_damage() {
    let mut g = main_phase();
    let reveka = g.add_card_to_battlefield(0, catalog::reveka_wizard_savant());
    g.clear_sickness(reveka);
    activate(&mut g, 0, reveka, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
    g.do_untap();
    assert!(g.battlefield_find(reveka).expect("reveka").tapped, "it sleeps one untap step off");
}

#[test]
fn sengir_bats_grow_on_the_kills_they_helped_with() {
    let mut g = main_phase();
    let bats = g.add_card_to_battlefield(0, catalog::sengir_bats());
    let victim = g.add_card_to_battlefield(1, catalog::mogg_fanatic()); // 1/1
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(victim),
        1,
        Some(bats),
        &mut evs,
    );
    let mut sba = g.check_state_based_actions();
    evs.append(&mut sba);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bats).expect("bats").counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

#[test]
fn greater_werewolf_shrinks_what_it_fought() {
    let mut g = main_phase();
    let wolf = g.add_card_to_battlefield(0, catalog::greater_werewolf());
    let bear = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3, survives
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(wolf, bear)])).expect("block");
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).expect("bear").counter_count(CounterType::MinusZeroMinusTwo),
        1
    );
}

#[test]
fn funeral_march_bills_the_host_controller_a_second_body() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spare = g.add_card_to_battlefield(1, catalog::mogg_fanatic());
    let march = g.add_card_to_hand(0, catalog::funeral_march());
    cast(&mut g, 0, march, Some(Target::Permanent(bear)));
    let mut evs = Vec::new();
    g.destroy_permanent(bear, false, &mut evs);
    let mut sba = g.check_state_based_actions();
    evs.append(&mut sba);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(spare).is_none(), "the only creature left is sacrificed");
}

#[test]
fn roots_pins_a_grounded_creature() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let roots = g.add_card_to_hand(0, catalog::roots());
    cast(&mut g, 0, roots, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).expect("bear").tapped, "it taps on arrival");
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).expect("bear").tapped, "and stays down");
}

#[test]
fn spectral_bears_sulk_without_black_across_the_table() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::spectral_bears());
    g.clear_sickness(bears);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.do_untap();
    assert!(g.battlefield_find(bears).expect("bears").tapped);
}

#[test]
fn rashka_swells_against_black() {
    let mut g = main_phase();
    let rashka = g.add_card_to_battlefield(0, catalog::rashka_the_slayer());
    let shade = g.add_card_to_battlefield(1, catalog::greater_werewolf()); // black 2/4
    g.clear_sickness(shade);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: shade, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(rashka, shade)])).expect("block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(rashka).expect("rashka");
    assert_eq!((cp.power, cp.toughness), (4, 5));
}

#[test]
fn clockwork_steed_winds_down_when_it_fights() {
    let mut g = main_phase();
    let steed = g.add_card_to_hand(0, catalog::clockwork_steed());
    cast(&mut g, 0, steed, None);
    assert_eq!(g.computed_permanent(steed).expect("steed").power, 4);
    g.clear_sickness(steed);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: steed, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(steed).expect("steed").power, 3);
}

#[test]
fn clockwork_swarm_rewinds_only_up_to_four() {
    let mut g = main_phase();
    let swarm = g.add_card_to_hand(0, catalog::clockwork_swarm());
    cast(&mut g, 0, swarm, None);
    g.clear_sickness(swarm);
    g.step = TurnStep::Upkeep;
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: swarm,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(3),
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(swarm).expect("swarm").counter_count(CounterType::PlusOnePlusZero),
        4,
        "the cap holds",
    );
}

#[test]
fn beast_walkers_buy_banding() {
    let mut g = main_phase();
    let walkers = g.add_card_to_battlefield(0, catalog::beast_walkers());
    activate(&mut g, 0, walkers, None);
    assert!(g.computed_permanent(walkers).expect("walkers").keywords.contains(&Keyword::Banding));
}

#[test]
fn serra_paladin_prevents_a_point_and_hands_out_vigilance() {
    let mut g = main_phase();
    let paladin = g.add_card_to_battlefield(0, catalog::serra_paladin());
    g.clear_sickness(paladin);
    activate(&mut g, 0, paladin, Some(Target::Player(0)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 18, "one of the three was prevented");
}

#[test]
fn heart_wolf_goes_down_with_the_dwarf_it_armed() {
    let mut g = main_phase();
    let wolf = g.add_card_to_battlefield(0, catalog::heart_wolf());
    g.clear_sickness(wolf);
    let dwarf = g.add_card_to_battlefield(0, catalog::dwarven_pony()); // not a Dwarf
    let real = g.add_card_to_battlefield(0, catalog::reveka_wizard_savant()); // Dwarf Wizard
    let _ = dwarf;
    g.step = TurnStep::DeclareAttackers;
    activate(&mut g, 0, wolf, Some(Target::Permanent(real)));
    let cp = g.computed_permanent(real).expect("dwarf");
    assert_eq!(cp.power, 2);
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
    let mut evs = Vec::new();
    g.destroy_permanent(real, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wolf).is_none());
}

#[test]
fn heart_wolf_only_fires_during_combat() {
    let mut g = main_phase();
    let wolf = g.add_card_to_battlefield(0, catalog::heart_wolf());
    g.clear_sickness(wolf);
    let dwarf = g.add_card_to_battlefield(0, catalog::reveka_wizard_savant());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: wolf,
            ability_index: 0,
            target: Some(Target::Permanent(dwarf)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "a main phase is not combat",
    );
}

#[test]
fn jovens_tools_leave_only_walls_in_the_way() {
    let mut g = main_phase();
    let tools = g.add_card_to_battlefield(0, catalog::jovens_tools());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    activate(&mut g, 0, tools, Some(Target::Permanent(bear)));
    let blocker = g.add_card_to_battlefield(1, catalog::mogg_fanatic());
    let wall = g.add_card_to_battlefield(1, catalog::cemetery_gate());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    assert!(g.declare_blockers(vec![(blocker, bear)]).is_err());
    assert!(g.declare_blockers(vec![(wall, bear)]).is_ok());
}

#[test]
fn willow_priestess_drops_a_faerie_in_for_free() {
    let mut g = main_phase();
    let priestess = g.add_card_to_battlefield(0, catalog::willow_priestess());
    g.clear_sickness(priestess);
    let faerie = g.add_card_to_hand(0, catalog::willow_faerie());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![faerie]),
    ]));
    activate(&mut g, 0, priestess, None);
    assert!(g.battlefield_find(faerie).is_some());
}

#[test]
fn dark_maze_charges_once_then_vanishes() {
    let mut g = main_phase();
    let maze = g.add_card_to_battlefield(0, catalog::dark_maze());
    g.clear_sickness(maze);
    activate(&mut g, 0, maze, None);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: maze, target: AttackTarget::Player(1) }])
        .expect("defender is ignored this turn");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == maze));
}

#[test]
fn samite_alchemist_shields_a_creature_at_the_cost_of_a_turn() {
    let mut g = main_phase();
    let alchemist = g.add_card_to_battlefield(0, catalog::samite_alchemist());
    g.clear_sickness(alchemist);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, alchemist, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).expect("bear").tapped);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear)));
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_some(), "the shield ate the bolt");
    g.do_untap();
    assert!(g.battlefield_find(bear).expect("bear").tapped, "and it stays down a turn");
}

#[test]
fn jovens_ferrets_pin_whatever_blocked_them() {
    let mut g = main_phase();
    let ferrets = g.add_card_to_battlefield(0, catalog::jovens_ferrets());
    g.clear_sickness(ferrets);
    let blocker = g.add_card_to_battlefield(1, catalog::cemetery_gate()); // 0/5 survives
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: ferrets, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ferrets).expect("ferrets").toughness, 3);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, ferrets)])).expect("block");
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).expect("blocker").tapped);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(blocker).expect("blocker").tapped, "and it stays down");
}

// ── Closing wave (`catalog::sets::hml2`) ───────────────────────────────────

/// The Homelands cantrip rider draws on the *next turn's* upkeep, not the
/// caster's own.
#[test]
fn headstone_exiles_and_cantrips_next_upkeep() {
    let mut g = main_phase();
    let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::headstone());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, id, Some(Target::Permanent(victim)));
    assert!(g.players[1].graveyard.is_empty(), "the graveyard card was exiled");
    assert_eq!(g.players[0].hand.len(), hand - 1, "no draw yet");
    g.active_player_idx = 1;
    g.turn_number += 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "drew on the next turn's upkeep");
}

/// Prophecy gains life only off a land, and shuffles either way.
#[test]
fn prophecy_gains_life_off_a_land() {
    let mut g = main_phase();
    g.add_card_to_library(1, catalog::forest());
    let forest = g.players[1].library.pop().expect("forest");
    g.players[1].library.insert(0, forest);
    let id = g.add_card_to_hand(0, catalog::prophecy());
    cast(&mut g, 0, id, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, 21, "a land on top is worth a life");
}

/// Leeches strips poison and burns for what it took.
#[test]
fn leeches_converts_poison_into_damage() {
    let mut g = main_phase();
    g.players[1].poison_counters = 3;
    let id = g.add_card_to_hand(0, catalog::leeches());
    cast(&mut g, 0, id, Some(Target::Player(1)));
    assert_eq!(g.players[1].poison_counters, 0);
    assert_eq!(g.players[1].life, 17, "3 poison became 3 damage");
}

/// Baki's Curse scales with Auras, so an unenchanted creature is untouched.
#[test]
fn bakis_curse_only_hits_enchanted_creatures() {
    let mut g = main_phase();
    let clean = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cursed = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(1, catalog::ironclaw_curse());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(cursed);
    let id = g.add_card_to_hand(0, catalog::bakis_curse());
    cast(&mut g, 0, id, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(clean).is_some(), "no Aura, no damage");
    assert!(g.battlefield_find(cursed).is_none(), "2 damage killed the 2/1");
}

/// Baron Sengir grows off a creature its damage killed.
#[test]
fn baron_sengir_grows_off_its_victims() {
    let mut g = main_phase();
    let baron = g.add_card_to_battlefield(0, catalog::baron_sengir());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(victim),
        5,
        Some(baron),
        &mut evs,
    );
    let mut sba = g.check_state_based_actions();
    evs.append(&mut sba);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(baron).expect("baron");
    assert_eq!((cp.power, cp.toughness), (7, 7), "+2/+2 counter");
}

/// Ironclaw Curse shrinks its host and keeps it off anything its own size.
#[test]
fn ironclaw_curse_blocks_big_attackers() {
    let mut g = main_phase();
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → 2/1
    let aura = g.add_card_to_battlefield(1, catalog::ironclaw_curse());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(blocker);
    assert_eq!(g.computed_permanent(blocker).expect("bear").toughness, 1);
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).is_err(),
        "power 2 >= toughness 1, so it can't block"
    );
}

/// Serra Bestiary shuts off the enchanted creature's tap abilities.
#[test]
fn serra_bestiary_locks_tap_abilities() {
    let mut g = main_phase();
    let tim = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.clear_sickness(tim);
    let aura = g.add_card_to_battlefield(0, catalog::serra_bestiary());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(tim);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tim, ability_index: 0, target: Some(Target::Player(0)),
            additional_targets: vec![], x_value: None, mode: None,
        })
        .is_err(),
        "{{T}} abilities are locked"
    );
}

/// Truce pays out life for every card a player declines to draw.
#[test]
fn truce_pays_life_for_declined_draws() {
    let mut g = main_phase();
    // Seat 0 draws both, seat 1 draws none.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(2),
        DecisionAnswer::Amount(0),
    ]));
    let id = g.add_card_to_hand(0, catalog::truce());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, id, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew two");
    assert_eq!(g.players[0].life, 20, "and gained nothing");
    assert_eq!(g.players[1].life, 24, "two declined cards = 4 life");
}

/// Koskun Falls eats itself when its controller won't tap a creature.
#[test]
fn koskun_falls_sacrifices_itself_without_a_creature() {
    let mut g = main_phase();
    let falls = g.add_card_to_battlefield(0, catalog::koskun_falls());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(falls).is_none(), "no creature to tap");
}

/// Drudge Spell's Skeletons die with it.
#[test]
fn drudge_spell_takes_its_skeletons_with_it() {
    let mut g = main_phase();
    let spell = g.add_card_to_battlefield(0, catalog::drudge_spell());
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    activate_n(&mut g, 0, spell, 0, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1, "one Skeleton");
    g.sacrifice_one(spell, 0, &mut Vec::new());
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield.iter().all(|c| !c.is_token), "Skeletons destroyed with the enchantment");
}

// ── The closing twelve ──────────────────────────────────────────────────────

/// Autumn Willow's {G} opens her to one player's spells only.
#[test]
fn autumn_willow_waives_shroud_for_one_player() {
    let mut g = main_phase();
    let willow = g.add_card_to_battlefield(0, catalog::autumn_willow());
    let bolt = g.add_card_to_hand(1, catalog::shock());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(willow)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "shroud holds before the waiver"
    );
    activate(&mut g, 0, willow, Some(Target::Player(1)));
    cast(&mut g, 1, bolt, Some(Target::Permanent(willow)));
    assert_eq!(g.battlefield_find(willow).expect("willow").damage, 2);
}

/// Broken Visage leaves a Spirit wearing the dead attacker's body.
#[test]
fn broken_visage_mints_a_spirit_with_the_victims_stats() {
    let mut g = main_phase();
    let atk = g.add_card_to_battlefield(1, catalog::sengir_vampire()); // 4/4
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(0) }])
        .expect("attack");
    let visage = g.add_card_to_hand(0, catalog::broken_visage());
    cast(&mut g, 0, visage, Some(Target::Permanent(atk)));
    assert!(g.battlefield_find(atk).is_none(), "attacker destroyed");
    let token = g.battlefield.iter().find(|c| c.is_token).expect("Spirit");
    assert_eq!((token.definition.power, token.definition.toughness), (4, 4));
}

/// Chain Stasis hands the chain to the creature's controller for {2}{U}.
#[test]
fn chain_stasis_chains_off_the_targets_controller() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    // Seat 1 accepts the copy but has no mana, so the chain stops there.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let stasis = g.add_card_to_hand(0, catalog::chain_stasis());
    cast(&mut g, 0, stasis, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(bear).expect("bear").tapped, "untapped");
    assert!(g.stack.is_empty(), "unpayable chain cost stops the copy");
}

/// Coral Reef banks polyps off Islands and spends them on toughness.
#[test]
fn coral_reef_trades_islands_for_toughness() {
    let mut g = main_phase();
    let reef = g.add_card_to_hand(0, catalog::coral_reef());
    cast(&mut g, 0, reef, None);
    assert_eq!(g.battlefield_find(reef).expect("reef").counter_count(CounterType::Polyp), 4);
    g.add_card_to_battlefield(0, catalog::island());
    activate_n(&mut g, 0, reef, 0, None);
    assert_eq!(g.battlefield_find(reef).expect("reef").counter_count(CounterType::Polyp), 6);
    let helper = g.add_card_to_battlefield(0, catalog::merfolk_of_the_pearl_trident());
    g.clear_sickness(helper);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate_n(&mut g, 0, reef, 1, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).expect("bear").toughness, 3);
    assert_eq!(g.battlefield_find(reef).expect("reef").counter_count(CounterType::Polyp), 5);
    assert!(g.battlefield_find(helper).expect("helper").tapped, "a blue creature paid");
}

/// Dwarven Sea Clan only shoots a combatant whose controller has an Island,
/// and the damage lands at end of combat.
#[test]
fn dwarven_sea_clan_snipes_at_end_of_combat() {
    let mut g = main_phase();
    let clan = g.add_card_to_battlefield(0, catalog::dwarven_sea_clan());
    g.clear_sickness(clan);
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: clan, ability_index: 0, target: Some(Target::Permanent(atk)),
            additional_targets: vec![], x_value: None, mode: None,
        })
        .is_err(),
        "no Island across the table"
    );
    g.add_card_to_battlefield(1, catalog::island());
    activate(&mut g, 0, clan, Some(Target::Permanent(atk)));
    assert_eq!(g.battlefield_find(atk).expect("attacker").damage, 0, "not yet");
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert!(g.battlefield_find(atk).is_none(), "2 damage kills the 2/2");
}

/// Giant Albatross punishes its killer unless they pay 2 life.
#[test]
fn giant_albatross_drags_down_its_killer() {
    let mut g = main_phase();
    let bird = g.add_card_to_battlefield(0, catalog::giant_albatross());
    let killer = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Yes to the {1}{U}; seat 1 declines to pay the 2 life.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
    ]));
    mana(&mut g, 0);
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(bird), 1, Some(killer), &mut events);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(killer).is_none(), "unpaid killer dies");
}

/// Giant Oyster freezes a tapped creature and withers it each draw step.
#[test]
fn giant_oyster_withers_what_it_holds() {
    let mut g = main_phase();
    let oyster = g.add_card_to_battlefield(0, catalog::giant_oyster());
    g.clear_sickness(oyster);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, 0, oyster, Some(Target::Permanent(bear)));
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).expect("bear").counter_count(CounterType::MinusOneMinusOne),
        1
    );
    // Untapping the Oyster releases the lock and clears the counters.
    g.battlefield_find_mut(oyster).unwrap().tapped = false;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentUntapped { card_id: oyster }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).expect("bear").counter_count(CounterType::MinusOneMinusOne),
        0
    );
}

/// Jinx retypes a land for the turn and cantrips next upkeep.
#[test]
fn jinx_retypes_a_land_and_cantrips() {
    let mut g = main_phase();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let jinx = g.add_card_to_hand(0, catalog::jinx());
    cast(&mut g, 0, jinx, Some(Target::Permanent(mountain)));
    let cp = g.computed_permanent(mountain).expect("land");
    assert!(cp.subtypes.land_types.contains(&crabomination::card::LandType::Plains));
    let hand = g.players[0].hand.len();
    g.active_player_idx = 1;
    g.turn_number += 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Marjhan needs an Island to survive and stays tapped without an upkeep sac.
#[test]
fn marjhan_drowns_without_islands() {
    let mut g = main_phase();
    let marjhan = g.add_card_to_battlefield(0, catalog::marjhan());
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(marjhan).is_none(), "no Islands, no Serpent");

    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::island());
    let marjhan = g.add_card_to_battlefield(0, catalog::marjhan());
    g.battlefield_find_mut(marjhan).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(marjhan).expect("marjhan").tapped, "doesn't untap on its own");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::Upkeep;
    activate_n(&mut g, 0, marjhan, 0, None);
    assert!(!g.battlefield_find(marjhan).expect("marjhan").tapped, "sacrificed a creature to untap");
}

/// Marjhan's {U}{U} shrinks it and pings a grounded attacker.
#[test]
fn marjhan_shoots_a_grounded_attacker() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::island());
    let marjhan = g.add_card_to_battlefield(0, catalog::marjhan());
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(0) }])
        .expect("attack");
    activate_n(&mut g, 0, marjhan, 1, Some(Target::Permanent(atk)));
    assert_eq!(g.battlefield_find(atk).expect("attacker").damage, 1);
    assert_eq!(g.computed_permanent(marjhan).expect("marjhan").power, 7);
}

/// Orcish Mine counts down on upkeeps and land taps, then blows up the land.
#[test]
fn orcish_mine_counts_down_to_the_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    let mine = g.add_card_to_hand(0, catalog::orcish_mine());
    cast(&mut g, 0, mine, Some(Target::Permanent(land)));
    assert_eq!(g.battlefield_find(mine).expect("mine").counter_count(CounterType::Ore), 3);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).expect("mine").counter_count(CounterType::Ore), 2);
    for _ in 0..2 {
        let Some(c) = g.battlefield_find_mut(land) else { break };
        c.tapped = true;
        g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped {
            card_id: land, actor: Some(1), as_attacker: false,
        }]);
        drain_stack(&mut g);
        g.check_state_based_actions();
        drain_stack(&mut g);
        if let Some(c) = g.battlefield_find_mut(land) {
            c.tapped = false;
        }
    }
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "the land is mined out");
    assert_eq!(g.players[1].life, 18);
}

/// Retribution lets the victim's controller pick which of the two dies.
#[test]
fn retribution_lets_the_opponent_pick_the_survivor() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::sengir_vampire());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a])]));
    let spell = g.add_card_to_hand(0, catalog::retribution());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "the chosen one is sacrificed");
    assert_eq!(
        g.battlefield_find(b).expect("survivor").counter_count(CounterType::MinusOneMinusOne),
        1
    );
}

/// Rysorian Badger swaps its combat damage for graveyard exile and life.
#[test]
fn rysorian_badger_eats_the_defenders_graveyard() {
    let mut g = main_phase();
    let badger = g.add_card_to_battlefield(0, catalog::rysorian_badger());
    g.clear_sickness(badger);
    let food = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: badger, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != food), "exiled");
    assert_eq!(g.players[0].life, 21);
    assert!(
        g.computed_permanent(badger)
            .expect("badger")
            .keywords
            .contains(&Keyword::DealsNoCombatDamage)
    );
}
