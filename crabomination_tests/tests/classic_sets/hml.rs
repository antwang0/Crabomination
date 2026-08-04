//! Homelands (HML) — `catalog::sets::hml`.

use crabomination::card::{CardId, CounterType, Keyword};
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
