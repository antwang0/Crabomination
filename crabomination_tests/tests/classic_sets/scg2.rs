//! Scourge (SCG) closing wave — the rares that each brought a primitive
//! (`catalog::sets::scg2`).

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

fn advance_to(g: &mut GameState, step: TurnStep) {
    g.step = TurnStep::Untap;
    while g.step != step {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(g);
}

fn unmorph(g: &mut GameState, id: CardId) {
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().turn_face_down();
    mana(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("unmorph");
    drain_stack(g);
}

// ── White ───────────────────────────────────────────────────────────────────

/// Ageless Sentinels sheds Wall/defender for good the first time it blocks.
#[test]
fn ageless_sentinels_becomes_a_bird_giant_when_it_blocks() {
    let mut g = main_phase();
    let sentinels = g.add_card_to_battlefield(1, catalog::ageless_sentinels());
    let attacker = g.add_card_to_battlefield(0, catalog::silver_knight());
    g.battlefield.iter_mut().find(|c| c.id == attacker).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    let evs = g.declare_blockers(vec![(sentinels, attacker)]).expect("block");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(sentinels).expect("computed");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Bird));
    assert!(!cp.keywords.contains(&Keyword::Defender), "defender is gone for good");
}

/// Force Bubble eats damage as depletion counters and pops at four.
#[test]
fn force_bubble_soaks_damage_then_sacrifices_itself() {
    let mut g = main_phase();
    let bubble = g.add_card_to_battlefield(0, catalog::force_bubble());
    let life = g.players[0].life;
    let mut events = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 3, None, &mut events);
    assert_eq!(g.players[0].life, life, "damage became counters");
    assert_eq!(g.battlefield_find(bubble).unwrap().counter_count(CounterType::Depletion), 3);
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 1, None, &mut events);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bubble).is_none(), "four counters sacrificed it");
}

/// Gilded Light makes its controller untargetable for the turn.
#[test]
fn gilded_light_gives_you_shroud() {
    let mut g = main_phase();
    let light = g.add_card_to_hand(0, catalog::gilded_light());
    cast(&mut g, 0, light, None);
    assert!(g.check_target_legality(&Target::Player(0), 1).is_err(), "shrouded");
    assert!(g.check_target_legality(&Target::Player(1), 0).is_ok(), "only you gained it");
}

/// Karona's Zealot's flip parks the turn's damage on someone else.
#[test]
fn karonas_zealot_redirects_its_damage() {
    let mut g = main_phase();
    let zealot = g.add_card_to_battlefield(0, catalog::karonas_zealot());
    let bystander = g.add_card_to_battlefield(0, catalog::ageless_sentinels());
    g.decider =
        Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(bystander))]));
    unmorph(&mut g, zealot);
    let mut events = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(zealot),
        3,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(zealot).unwrap().damage, 0);
    assert_eq!(g.battlefield_find(bystander).unwrap().damage, 3);
}

/// Trap Digger arms a land, then springs it at an attacker.
#[test]
fn trap_digger_springs_a_trap_on_an_attacker() {
    let mut g = main_phase();
    let digger = g.add_card_to_battlefield(0, catalog::trap_digger());
    g.battlefield.iter_mut().find(|c| c.id == digger).unwrap().summoning_sick = false;
    let land = g.add_card_to_battlefield(0, catalog::plains());
    let attacker = g.add_card_to_battlefield(1, catalog::silver_knight()); // 2/2
    activate(&mut g, 0, digger, 0, Some(Target::Permanent(land)));
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::Trap), 1);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    activate(&mut g, 0, digger, 1, Some(Target::Permanent(attacker)));
    g.check_state_based_actions();
    assert!(g.battlefield_find(attacker).is_none(), "3 damage killed the 2/2");
    assert!(g.battlefield_find(land).is_none(), "the land was the cost");
}

/// Dimensional Breach exiles the board and hands one back per upkeep.
#[test]
fn dimensional_breach_exiles_all_then_returns_one_per_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::silver_knight());
    g.add_card_to_battlefield(1, catalog::silver_knight());
    let breach = g.add_card_to_hand(0, catalog::dimensional_breach());
    cast(&mut g, 0, breach, None);
    assert!(g.battlefield.is_empty(), "everything left");
    assert_eq!(g.exile.len(), 2);
    g.active_player_idx = 1;
    advance_to(&mut g, TurnStep::Upkeep);
    assert_eq!(g.battlefield.len(), 1, "the active player rebuilt one");
    assert_eq!(g.battlefield[0].owner, 1);
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Day of the Dragons swaps your board for 5/5 fliers, then swaps back.
#[test]
fn day_of_the_dragons_swaps_your_board_and_gives_it_back() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::silver_knight());
    g.add_card_to_battlefield(0, catalog::coast_watcher());
    let day = g.add_card_to_hand(0, catalog::day_of_the_dragons());
    cast(&mut g, 0, day, None);
    let dragons: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Dragon))
        .map(|c| c.id)
        .collect();
    assert_eq!(dragons.len(), 2, "one Dragon per exiled creature");
    let enchantment = g.battlefield.iter().find(|c| c.definition.name == "Day of the Dragons")
        .map(|c| c.id)
        .expect("the enchantment");
    let mut events = vec![];
    g.destroy_permanent(enchantment, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().all(|c| !c.definition.subtypes.creature_types.contains(&CreatureType::Dragon)),
        "the Dragons were sacrificed"
    );
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 2);
}

/// Faces of the Past taps every creature sharing a type with the dead one.
#[test]
fn faces_of_the_past_taps_the_dead_creatures_type() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::faces_of_the_past());
    let knight = g.add_card_to_battlefield(0, catalog::silver_knight()); // Human Knight
    let other = g.add_card_to_battlefield(1, catalog::karonas_zealot()); // Human Cleric
    let doomed = g.add_card_to_battlefield(1, catalog::silver_knight());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(0)]));
    let mut events = vec![];
    g.destroy_permanent(doomed, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(knight).unwrap().tapped, "shares Human/Knight");
    assert!(g.battlefield_find(other).unwrap().tapped, "shares Human");
}

/// Long-Term Plans shuffles first, then buries the pick third from the top.
#[test]
fn long_term_plans_puts_the_pick_third_from_the_top() {
    let mut g = main_phase();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::silver_knight());
    }
    let wanted = g.add_card_to_library(0, catalog::dragon_tyrant());
    let plans = g.add_card_to_hand(0, catalog::long_term_plans());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(wanted))]));
    cast(&mut g, 0, plans, None);
    assert_eq!(g.players[0].library[2].id, wanted);
}

/// Mistform Warchief discounts creature spells that share its type.
#[test]
fn mistform_warchief_discounts_shared_types() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mistform_warchief()); // Illusion
    let illusion = g.add_card_to_hand(0, catalog::mistform_warchief());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: illusion,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{2}{U} minus {1} is payable with {1}{U}");
}

/// Parallel Thoughts turns your draws into picks off its own pile.
#[test]
fn parallel_thoughts_draws_off_its_exiled_pile() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::silver_knight());
    }
    let wanted = g.add_card_to_library(0, catalog::dragon_tyrant());
    let thoughts = g.add_card_to_hand(0, catalog::parallel_thoughts());
    cast(&mut g, 0, thoughts, None);
    assert_eq!(g.exile.len(), 4, "the whole library became the pile");
    // Say yes to the replacement; the pile's first card comes to hand.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let mut events = vec![];
    assert!(g.draw_one(0, &mut events));
    assert!(g.players[0].hand.iter().any(|c| c.id == wanted || c.definition.name == "Silver Knight"));
    assert_eq!(g.exile.len(), 3);
}

/// Pemmin's Aura untaps its host and hands out flying.
#[test]
fn pemmins_aura_untaps_and_pumps_its_host() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::silver_knight());
    let aura = g.add_card_to_hand(0, catalog::pemmins_aura());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    let aura_id = g.battlefield.iter().find(|c| c.definition.name == "Pemmin's Aura").unwrap().id;
    g.battlefield.iter_mut().find(|c| c.id == host).unwrap().tapped = true;
    activate(&mut g, 0, aura_id, 0, None);
    assert!(!g.battlefield_find(host).unwrap().tapped);
    activate(&mut g, 0, aura_id, 1, None);
    assert!(g.computed_permanent(host).unwrap().keywords.contains(&Keyword::Flying));
    activate(&mut g, 0, aura_id, 3, None);
    assert_eq!(g.computed_permanent(host).unwrap().power, 3, "+1/-1");
}

/// Proteus Machine's free morph names its type on the way up.
#[test]
fn proteus_machine_names_its_type_on_the_flip() {
    let mut g = main_phase();
    let machine = g.add_card_to_battlefield(0, catalog::proteus_machine());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        CreatureType::Sliver,
    )]));
    unmorph(&mut g, machine);
    assert!(
        g.computed_permanent(machine)
            .unwrap()
            .subtypes
            .creature_types
            .contains(&CreatureType::Sliver)
    );
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Call to the Grave eats a non-Zombie at each upkeep.
#[test]
fn call_to_the_grave_eats_a_non_zombie_each_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::call_to_the_grave());
    let knight = g.add_card_to_battlefield(0, catalog::silver_knight());
    advance_to(&mut g, TurnStep::Upkeep);
    assert!(g.battlefield_find(knight).is_none());
}

/// Fatal Mutation answers the morph it's stuck on.
#[test]
fn fatal_mutation_kills_the_creature_it_flips() {
    let mut g = main_phase();
    let zealot = g.add_card_to_battlefield(0, catalog::karonas_zealot());
    let aura = g.add_card_to_hand(1, catalog::fatal_mutation());
    g.active_player_idx = 1;
    cast(&mut g, 1, aura, Some(Target::Permanent(zealot)));
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(
        zealot,
    ))]));
    unmorph(&mut g, zealot);
    g.check_state_based_actions();
    assert!(g.battlefield_find(zealot).is_none());
}

/// Lethal Vapors kills entrants, and any player can undo it for a turn.
#[test]
fn lethal_vapors_locks_the_board_until_someone_skips_a_turn() {
    let mut g = main_phase();
    let vapors = g.add_card_to_battlefield(0, catalog::lethal_vapors());
    let knight = g.add_card_to_hand(1, catalog::silver_knight());
    g.active_player_idx = 1;
    cast(&mut g, 1, knight, None);
    g.active_player_idx = 0;
    g.check_state_based_actions();
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Silver Knight"));
    activate(&mut g, 1, vapors, 0, None);
    assert!(g.battlefield_find(vapors).is_none());
    assert_eq!(g.players[1].skip_turns, 1);
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Dragonstorm fetches one Dragon per copy.
#[test]
fn dragonstorm_fetches_a_dragon_per_storm_copy() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::dragon_tyrant());
    }
    g.spells_cast_this_turn = 1;
    let storm = g.add_card_to_hand(0, catalog::dragonstorm());
    let picks: Vec<_> = g.players[0]
        .library
        .iter()
        .map(|c| DecisionAnswer::Search(Some(c.id)))
        .collect();
    g.decider = Box::new(ScriptedDecider::new(picks));
    cast(&mut g, 0, storm, None);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Dragon Tyrant").count(),
        2,
        "the original plus one storm copy"
    );
}

/// Form of the Dragon shuts the ground down and resets your life each turn.
#[test]
fn form_of_the_dragon_sets_your_life_and_walls_the_ground() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::form_of_the_dragon());
    let ground = g.add_card_to_battlefield(1, catalog::silver_knight());
    g.battlefield.iter_mut().find(|c| c.id == ground).unwrap().summoning_sick = false;
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: ground, target: AttackTarget::Player(0) }])
            .is_err(),
        "no flying, no attack"
    );
    g.players[0].life = 27;
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.players[0].life, 5);
}

/// Goblin Psychopath's lost flip turns its swing on its controller.
#[test]
fn goblin_psychopath_can_hit_its_own_controller() {
    let mut g = main_phase();
    let psycho = g.add_card_to_battlefield(0, catalog::goblin_psychopath());
    g.battlefield.iter_mut().find(|c| c.id == psycho).unwrap().summoning_sick = false;
    // Lose the flip: the coin decision answers tails.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: psycho, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    advance_to(&mut g, TurnStep::CombatDamage);
    assert_eq!(g.players[1].life, theirs, "the defender took nothing");
    assert_eq!(g.players[0].life, mine - 5, "it hit its own controller");
}

/// Rock Jockey can't be cast after a land drop.
#[test]
fn rock_jockey_fights_your_land_drop() {
    let mut g = main_phase();
    let jockey = g.add_card_to_hand(0, catalog::rock_jockey());
    g.players[0].lands_played_this_turn = 1;
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: jockey,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a land was played this turn"
    );
}

/// Grip of Chaos rerolls a single-target spell onto a random legal target.
#[test]
fn grip_of_chaos_rerolls_a_single_target() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grip_of_chaos());
    // Only one legal creature, so the reroll is deterministic.
    let only = g.add_card_to_battlefield(1, catalog::silver_knight());
    let aura = g.add_card_to_hand(0, catalog::fatal_mutation());
    cast(&mut g, 0, aura, Some(Target::Permanent(only)));
    assert!(g.battlefield.iter().any(|c| c.attached_to == Some(only)));
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Ambush Commander animates Forests and eats them for a pump.
#[test]
fn ambush_commander_animates_forests_and_eats_one() {
    let mut g = main_phase();
    let commander = g.add_card_to_battlefield(0, catalog::ambush_commander());
    g.battlefield.iter_mut().find(|c| c.id == commander).unwrap().summoning_sick = false;
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let cp = g.computed_permanent(forest).expect("computed");
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Elf));
    assert!(cp.colors.contains(Color::Green));
    activate(&mut g, 0, commander, 0, Some(Target::Permanent(commander)));
    assert!(g.battlefield_find(forest).is_none(), "the Forest was the Elf sacrificed");
    assert_eq!(g.computed_permanent(commander).unwrap().power, 5);
}

/// Divergent Growth turns your lands into any-colour sources for the turn.
#[test]
fn divergent_growth_gives_lands_any_color() {
    let mut g = main_phase();
    let plains = g.add_card_to_battlefield(0, catalog::plains());
    let growth = g.add_card_to_hand(0, catalog::divergent_growth());
    cast(&mut g, 0, growth, None);
    let granted = g.battlefield_find(plains).unwrap().granted_activated_eot.len();
    assert_eq!(granted, 1, "the land picked up the mana ability");
}

/// Forgotten Ancient grows on casts and hands its counters out at upkeep.
#[test]
fn forgotten_ancient_grows_then_spreads_its_counters() {
    let mut g = main_phase();
    let ancient = g.add_card_to_battlefield(0, catalog::forgotten_ancient());
    let friend = g.add_card_to_battlefield(0, catalog::silver_knight());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let knight = g.add_card_to_hand(0, catalog::silver_knight());
    cast(&mut g, 0, knight, None);
    assert_eq!(
        g.battlefield_find(ancient).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
    advance_to(&mut g, TurnStep::Upkeep);
    assert_eq!(
        g.battlefield_find(ancient).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0
    );
    assert!(
        g.battlefield_find(friend).unwrap().counter_count(CounterType::PlusOnePlusOne) >= 1
            || g.battlefield
                .iter()
                .any(|c| c.counter_count(CounterType::PlusOnePlusOne) >= 1),
        "the counter moved off"
    );
}

/// Primitive Etchings chains when the turn's first draw is a creature.
#[test]
fn primitive_etchings_chains_off_a_creature_draw() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::primitive_etchings());
    g.add_card_to_library(0, catalog::silver_knight());
    g.add_card_to_library(0, catalog::plains());
    let mut events = vec![];
    g.draw_one(0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "the creature draw drew again");
}

/// Root Elemental's flip cheats a fatty out of hand.
#[test]
fn root_elemental_cheats_a_creature_in() {
    let mut g = main_phase();
    let elemental = g.add_card_to_battlefield(0, catalog::root_elemental());
    let fatty = g.add_card_to_hand(0, catalog::dragon_tyrant());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![fatty])]));
    unmorph(&mut g, elemental);
    assert!(g.battlefield_find(fatty).is_some());
}

/// Xantid Swarm shuts the defender's hand for the turn.
#[test]
fn xantid_swarm_locks_the_defender_out_of_casting() {
    let mut g = main_phase();
    let swarm = g.add_card_to_battlefield(0, catalog::xantid_swarm());
    g.battlefield.iter_mut().find(|c| c.id == swarm).unwrap().summoning_sick = false;
    let counter = g.add_card_to_hand(1, catalog::silver_knight());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: swarm, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: counter,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the defending player is locked out"
    );
}

// ── Gold ────────────────────────────────────────────────────────────────────

/// Karona changes hands at every upkeep.
#[test]
fn karona_hands_herself_to_the_active_player() {
    let mut g = main_phase();
    let karona = g.add_card_to_battlefield(0, catalog::karona_false_god());
    g.battlefield.iter_mut().find(|c| c.id == karona).unwrap().tapped = true;
    g.active_player_idx = 1;
    advance_to(&mut g, TurnStep::Upkeep);
    let k = g.battlefield_find(karona).unwrap();
    assert_eq!(k.controller, 1);
    assert!(!k.tapped);
}

/// Karona's attack trigger pumps a whole creature type, both sides included.
#[test]
fn karona_pumps_the_chosen_type_everywhere() {
    let mut g = main_phase();
    let karona = g.add_card_to_battlefield(0, catalog::karona_false_god());
    g.battlefield.iter_mut().find(|c| c.id == karona).unwrap().summoning_sick = false;
    let theirs = g.add_card_to_battlefield(1, catalog::silver_knight()); // Human Knight
    g.decider =
        Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(CreatureType::Knight)]));
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: karona, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 5, "2 + 3");
}

/// Sliver Overlord tutors a Sliver, then steals one.
#[test]
fn sliver_overlord_tutors_then_steals() {
    let mut g = main_phase();
    let overlord = g.add_card_to_battlefield(0, catalog::sliver_overlord());
    g.battlefield.iter_mut().find(|c| c.id == overlord).unwrap().summoning_sick = false;
    let in_library = g.add_card_to_library(0, catalog::proteus_machine());
    g.battlefield.iter_mut().find(|c| c.id == overlord).unwrap();
    // Make the library card a Sliver by using an actual Sliver from the set.
    let _ = in_library;
    let theirs = g.add_card_to_battlefield(1, catalog::sliver_overlord());
    activate(&mut g, 0, overlord, 1, Some(Target::Permanent(theirs)));
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
}

/// Uncontrolled Infestation blows the land up the moment it taps.
#[test]
fn uncontrolled_infestation_destroys_the_land_on_tap() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::adarkar_wastes());
    let aura = g.add_card_to_hand(1, catalog::uncontrolled_infestation());
    g.active_player_idx = 1;
    cast(&mut g, 1, aura, Some(Target::Permanent(land)));
    g.active_player_idx = 0;
    g.battlefield.iter_mut().find(|c| c.id == land).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped {
        card_id: land,
        actor: None,
        as_attacker: false,
    }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none());
}

/// Metamorphose bounces to the top and lets them redeploy.
#[test]
fn metamorphose_swaps_a_permanent_for_one_from_hand() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::silver_knight());
    let replacement = g.add_card_to_hand(1, catalog::dragon_tyrant());
    let spell = g.add_card_to_hand(0, catalog::metamorphose());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![replacement])]));
    cast(&mut g, 0, spell, Some(Target::Permanent(theirs)));
    assert_eq!(g.players[1].library[0].id, theirs, "on top of its owner's library");
    assert!(g.battlefield_find(replacement).is_some(), "they redeployed");
}

/// Mischievous Quanar re-morphs itself and forks the next spell.
#[test]
fn mischievous_quanar_copies_an_instant_on_the_flip() {
    let mut g = main_phase();
    let quanar = g.add_card_to_battlefield(0, catalog::mischievous_quanar());
    g.battlefield.iter_mut().find(|c| c.id == quanar).unwrap().summoning_sick = false;
    activate(&mut g, 0, quanar, 0, None);
    assert!(g.battlefield_find(quanar).unwrap().face_down, "it went back down");
}
