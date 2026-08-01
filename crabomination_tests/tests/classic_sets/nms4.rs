//! Nemesis (NMS), fourth wave — the set's last sixteen cards.

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::effects::EntityRef;
use crabomination::game::*;
use crabomination::mana::Color;

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
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

/// Activate without pre-floating mana, so the pool assertion is the ability's.
fn tap_ability(g: &mut GameState, seat: usize, card_id: CardId, index: usize) {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
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

/// Defiant Vanguard drags what it blocked down with it at end of combat.
#[test]
fn defiant_vanguard_destroys_itself_and_what_it_blocked() {
    let mut g = two_player_game();
    let vanguard = g.add_card_to_battlefield(1, catalog::defiant_vanguard());
    let attacker = g.add_card_to_battlefield(0, catalog::colossal_dreadmaw());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(vanguard, attacker)])).expect("block");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(vanguard).is_none(), "the Vanguard goes");
    assert!(g.battlefield_find(attacker).is_none(), "and so does what it blocked");
}

/// Divining Witch exiles six, then digs the named card out of the rest.
#[test]
fn divining_witch_tutors_the_named_card() {
    let mut g = main_phase();
    let witch = g.add_card_to_battlefield(0, catalog::divining_witch());
    g.clear_sickness(witch);
    g.add_card_to_hand(0, catalog::lightning_bolt());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let prize = g.add_card_to_library(0, catalog::shivan_dragon());
    script(&mut g, vec![DecisionAnswer::NamedCard("Shivan Dragon".to_string())]);
    activate(&mut g, 0, witch, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == prize), "the named card lands in hand");
    assert_eq!(g.exile.len(), 6, "the top six are gone");
}

/// Eye of Yawgmoth digs as deep as the sacrificed creature's power and exiles
/// what it doesn't take.
#[test]
fn eye_of_yawgmoth_digs_for_the_sacrificed_power() {
    let mut g = main_phase();
    let eye = g.add_card_to_battlefield(0, catalog::eye_of_yawgmoth());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let before = g.players[0].hand.len();
    activate(&mut g, 0, eye, 0, None);
    assert_eq!(g.players[0].hand.len(), before + 1);
    assert_eq!(g.exile.len(), 1, "the other revealed card is exiled, not bottomed");
}

/// Flowstone Armor's shrink holds only while the Armor stays tapped.
#[test]
fn flowstone_armor_shrinks_while_it_stays_tapped() {
    let mut g = main_phase();
    let armor = g.add_card_to_battlefield(0, catalog::flowstone_armor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, armor, 0, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!((cp.power, cp.toughness), (3, 1));

    g.battlefield_find_mut(armor).unwrap().tapped = false;
    g.check_state_based_actions();
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!((cp.power, cp.toughness), (2, 2), "untapping drops the effect");
}

/// Fog Patch blocks the whole attack without a blocker.
#[test]
fn fog_patch_blocks_every_attacker() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    let fog = g.add_card_to_hand(1, catalog::fog_patch());
    cast(&mut g, 1, fog, None);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, 20, "a blocked attacker hits nobody");
}

/// Harvest Mage turns your land taps into a colour of your choice.
#[test]
fn harvest_mage_rewrites_your_land_taps() {
    let mut g = main_phase();
    let mage = g.add_card_to_battlefield(0, catalog::harvest_mage());
    g.clear_sickness(mage);
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let island = g.add_card_to_battlefield(0, catalog::island());
    script(&mut g, vec![DecisionAnswer::Color(Color::Red)]);
    activate(&mut g, 0, mage, 0, None);
    g.players[0].mana_pool.empty();
    tap_ability(&mut g, 0, island, 0);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "one red, not blue");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 0);
}

/// Kill Switch holds every other artifact down while it stays tapped, and
/// releases them once it untaps.
#[test]
fn kill_switch_locks_other_artifacts_until_it_untaps() {
    let mut g = main_phase();
    let switch = g.add_card_to_battlefield(0, catalog::kill_switch());
    let other = g.add_card_to_battlefield(0, catalog::sol_ring());
    activate(&mut g, 0, switch, 0, None);
    assert!(g.battlefield_find(other).unwrap().tapped);

    g.step = TurnStep::Untap;
    g.do_untap();
    assert!(g.battlefield_find(other).unwrap().tapped, "still locked while the Switch is tapped");
    assert!(!g.battlefield_find(switch).unwrap().tapped, "the Switch itself untaps");

    g.do_untap();
    assert!(!g.battlefield_find(other).unwrap().tapped, "the lock released");
}

/// Mana Cache banks a counter per untapped land and pays out to anyone on
/// their own turn.
#[test]
fn mana_cache_banks_counters_and_pays_any_player() {
    let mut g = two_player_game();
    let cache = g.add_card_to_battlefield(0, catalog::mana_cache());
    g.add_card_to_battlefield(1, catalog::island());
    g.add_card_to_battlefield(1, catalog::island());
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cache).unwrap().counter_count(CounterType::Charge), 2);

    g.step = TurnStep::PreCombatMain;
    g.players[1].mana_pool.empty();
    tap_ability(&mut g, 1, cache, 0);
    assert_eq!(g.players[1].mana_pool.colorless_amount(), 1, "the opponent cashed one in");
}

/// Mogg Toady sits out unless it has the numbers.
#[test]
fn mogg_toady_needs_more_creatures_than_the_other_player() {
    let mut g = two_player_game();
    let toady = g.add_card_to_battlefield(0, catalog::mogg_toady());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(toady);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: toady, target: AttackTarget::Player(1) }])
            .is_err(),
        "1 creature vs 1 is not more"
    );
    assert!(!g.blocker_can_block_attacker(toady, theirs), "and it can't block either");

    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.blocker_can_block_attacker(toady, theirs), "2 vs 1 blocks");
    g.declare_attackers(vec![Attack { attacker: toady, target: AttackTarget::Player(1) }])
        .expect("2 vs 1 attacks");
}

/// Oracle's Attendants takes a chosen source's damage for a creature.
#[test]
fn oracles_attendants_soaks_a_chosen_sources_damage() {
    let mut g = main_phase();
    let attendants = g.add_card_to_battlefield(0, catalog::oracles_attendants());
    g.clear_sickness(attendants);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    activate(&mut g, 0, attendants, 0, Some(Target::Permanent(bear)));
    let mut events = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(bear), 2, Some(dragon), &mut events);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "the bear is untouched");
    assert_eq!(g.battlefield_find(attendants).unwrap().damage, 2, "the Attendants ate it");
}

/// Overlaid Terrain eats your lands and doubles what the next one makes.
#[test]
fn overlaid_terrain_eats_lands_then_doubles_the_next() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let terrain = g.add_card_to_hand(0, catalog::overlaid_terrain());
    cast(&mut g, 0, terrain, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 0);

    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.empty();
    script(&mut g, vec![DecisionAnswer::Color(Color::Blue)]);
    tap_ability(&mut g, 0, forest, 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 2);
}

/// Pale Moon flattens nonbasic lands to colorless for the turn.
#[test]
fn pale_moon_makes_nonbasics_produce_colorless() {
    let mut g = main_phase();
    let dual = g.add_card_to_battlefield(0, catalog::steam_vents());
    let basic = g.add_card_to_battlefield(0, catalog::island());
    let moon = g.add_card_to_hand(0, catalog::pale_moon());
    cast(&mut g, 0, moon, None);
    g.players[0].mana_pool.empty();
    tap_ability(&mut g, 0, dual, 0);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "the dual makes colorless");
    tap_ability(&mut g, 0, basic, 0);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "basics are untouched");
}

/// Rootwater Thief pays {2} on connect to strip a card out of their library.
#[test]
fn rootwater_thief_exiles_from_the_damaged_players_library() {
    let mut g = two_player_game();
    let thief = g.add_card_to_battlefield(0, catalog::rootwater_thief());
    g.clear_sickness(thief);
    let prize = g.add_card_to_library(1, catalog::shivan_dragon());
    g.add_card_to_library(1, catalog::forest());
    script(&mut g, vec![DecisionAnswer::Bool(true), DecisionAnswer::Search(Some(prize))]);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: thief, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::EndCombat {
        // The {2} comes out of the floated pool, which empties each step.
        mana(&mut g, 0);
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.exile.iter().any(|c| c.id == prize), "the pick is exiled");
}

/// Saproling Burst's tokens are as big as the counters left, and die with it.
#[test]
fn saproling_burst_tokens_scale_and_die_with_it() {
    let mut g = main_phase();
    let burst = g.add_card_to_hand(0, catalog::saproling_burst());
    cast(&mut g, 0, burst, None);
    assert_eq!(g.battlefield_find(burst).unwrap().counter_count(CounterType::Fade), 7);
    activate(&mut g, 0, burst, 0, None);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Saproling")
        .map(|c| c.id)
        .expect("token");
    let cp = g.computed_permanent(token).expect("computed");
    assert_eq!((cp.power, cp.toughness), (6, 6), "six fade counters left");

    g.sacrifice_one(burst, 0, &mut vec![]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(token).is_none(), "the tokens go with it");
}

/// Sivvi's Valor takes a creature's damage onto its controller, and can be
/// paid by tapping a creature while you control a Plains.
#[test]
fn sivvis_valor_redirects_damage_to_you() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let valor = g.add_card_to_hand(0, catalog::sivvis_valor());
    cast(&mut g, 0, valor, Some(Target::Permanent(bear)));
    let mut events = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(bear), 3, Some(dragon), &mut events);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0);
    assert_eq!(g.players[0].life, 17, "the controller ate it");
}

/// Stronghold Gambit puts the cheapest revealed creature onto the battlefield.
#[test]
fn stronghold_gambit_deploys_the_cheapest_revealed_creature() {
    let mut g = main_phase();
    let cheap = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::shivan_dragon());
    let gambit = g.add_card_to_hand(0, catalog::stronghold_gambit());
    cast(&mut g, 0, gambit, None);
    assert_eq!(g.battlefield_find(cheap).map(|c| c.controller), Some(1));
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Shivan Dragon"), "the pricier one stays");
}


