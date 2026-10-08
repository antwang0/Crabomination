//! Commander: the one-card gaps in most-built commanders' EDHREC average
//! decks (`decks::cmdr_edhrec`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

fn connect(g: &mut GameState, attacker: CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(g);
    }
}

/// Cadira's hit makes a Rabbit for each token you control, counted as it
/// resolves.
#[test]
fn cadira_makes_a_rabbit_per_token() {
    let mut g = pod(2);
    let cadira = ready(&mut g, 0, catalog::cadira_caller_of_the_small());
    let t = crabomination_base::tokens::treasure_token();
    g.add_token_to_battlefield(0, &t);
    g.add_token_to_battlefield(0, &t);
    connect(&mut g, cadira);
    assert_eq!(named(&g, "Rabbit"), 2);
}

/// The Unbeatable Squirrel Girl: a Squirrel on entering, then {1}{G}{G}{G}
/// doubles the Squirrels (she is one herself).
#[test]
fn squirrel_girl_doubles_the_squirrels() {
    let mut g = pod(2);
    let sg = g.add_card_to_hand(0, catalog::the_unbeatable_squirrel_girl());
    flood(&mut g);
    cast(&mut g, sg, None);
    assert_eq!(named(&g, "Squirrel"), 1);
    activate(&mut g, sg, None);
    assert_eq!(named(&g, "Squirrel"), 3, "two Squirrels counted: her and the token");
}

/// Forced Fruition draws seven for the opponent who cast, not the others.
#[test]
fn forced_fruition_draws_the_caster_seven() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::forced_fruition());
    for _ in 0..10 {
        g.add_card_to_library(2, catalog::island());
    }
    let bolt = g.add_card_to_hand(2, catalog::lightning_bolt());
    g.players[2].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!([g.players[1].hand.len(), g.players[2].hand.len()], [0, 7]);
}

/// Lesser Masticore costs a discard to cast and pings a creature for {4}.
#[test]
fn lesser_masticore_discards_and_pings() {
    let mut g = pod(2);
    let fodder = g.add_card_to_hand(0, catalog::island());
    let m = g.add_card_to_hand(0, catalog::lesser_masticore());
    flood(&mut g);
    cast(&mut g, m, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "discarded as a cost");
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.clear_sickness(m);
    activate(&mut g, m, Some(Target::Permanent(elf)));
    assert!(g.battlefield_find(elf).is_none());
}

/// Dark Deal — every hand is discarded and redrawn one card short.
#[test]
fn dark_deal_redraws_one_fewer() {
    let mut g = pod(3);
    for (seat, n) in [(0, 3), (1, 4), (2, 0)] {
        for _ in 0..n {
            g.add_card_to_hand(seat, catalog::island());
        }
    }
    let deal = g.add_card_to_hand(0, catalog::dark_deal());
    flood(&mut g);
    cast(&mut g, deal, None);
    assert_eq!([g.players[0].hand.len(), g.players[1].hand.len(), g.players[2].hand.len()], [2, 3, 0]);
}

/// Agent of Treachery steals a permanent; with three stolen its end step
/// draws three.
#[test]
fn agent_of_treachery_steals_and_draws_with_three() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let agent = g.add_card_to_hand(0, catalog::agent_of_treachery());
    flood(&mut g);
    cast(&mut g, agent, Some(Target::Permanent(ring)));
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0);
    for _ in 0..2 {
        let b = ready(&mut g, 1, catalog::grizzly_bears());
        g.battlefield_find_mut(b).unwrap().controller = 0;
    }
    let hand = g.players[0].hand.len();
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Worldfire leaves an empty table and every life total at 1.
#[test]
fn worldfire_resets_the_table() {
    let mut g = pod(3);
    ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_hand(2, catalog::island());
    g.add_card_to_graveyard(1, catalog::island());
    let fire = g.add_card_to_hand(0, catalog::worldfire());
    flood(&mut g);
    cast(&mut g, fire, None);
    assert!(g.battlefield.is_empty());
    assert!(g.players.iter().all(|p| p.hand.is_empty() && p.life == 1));
    // Worldfire itself goes to the graveyard after it resolves.
    let graves: Vec<Vec<&str>> = g.players.iter().map(|p| p.graveyard.iter().map(|c| c.definition.name).collect()).collect();
    assert_eq!(graves, [vec!["Worldfire"], vec![], vec![]]);
}

/// Ganax makes a Treasure for itself and for each other Dragon entering.
#[test]
fn ganax_treasures_for_dragons() {
    let mut g = pod(2);
    let ganax = g.add_card_to_hand(0, catalog::ganax_astral_hunter());
    flood(&mut g);
    cast(&mut g, ganax, None);
    assert_eq!(named(&g, "Treasure"), 1);
    let drake = g.add_card_to_hand(0, catalog::shivan_dragon());
    cast(&mut g, drake, None);
    assert_eq!(named(&g, "Treasure"), 2);
}

/// Afterlife Insurance's granted afterlife makes a Spirit when a creature
/// dies this turn.
#[test]
fn afterlife_insurance_grants_afterlife() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ins = g.add_card_to_hand(0, catalog::afterlife_insurance());
    flood(&mut g);
    cast(&mut g, ins, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1, "a Spirit");
}

/// Dockside Chef may sacrifice itself to draw.
#[test]
fn dockside_chef_sacrifices_to_draw() {
    let mut g = pod(2);
    let chef = ready(&mut g, 0, catalog::dockside_chef());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    activate(&mut g, chef, None);
    assert!(g.battlefield_find(chef).is_none());
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Wizard's Staff: the equipped creature has prowess, and its triggered
/// abilities trigger twice (CR 603.2d) — two cast pumps per spell.
#[test]
fn wizards_staff_grants_prowess_and_doubles_its_triggers() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let staff = ready(&mut g, 0, catalog::wizards_staff());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: staff, target: bear }).expect("equip");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "prowess fired twice");
}

/// Sorcerer Class: loot two on entering; level 2 creatures tap for
/// restricted {U}/{R}; level 3 an instant pings each opponent per spell.
#[test]
fn sorcerer_class_levels_up() {
    let mut g = pod(3);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::island());
    }
    let class = g.add_card_to_hand(0, catalog::sorcerer_class());
    flood(&mut g);
    let hand = g.players[0].hand.len();
    cast(&mut g, class, None);
    assert_eq!(g.players[0].hand.len(), hand - 1, "drew two, discarded two");
    activate(&mut g, class, None);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: class,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("level 3");
    drain_stack(&mut g);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let pool = g.players[0].mana_pool.restricted_total();
    activate(&mut g, bear, None);
    assert_eq!(g.players[0].mana_pool.restricted_total(), pool + 1, "granted restricted mana ability");
    let lives = [g.players[1].life, g.players[2].life];
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert_eq!([g.players[1].life, g.players[2].life], [lives[0] - 1, lives[1] - 1]);
}

/// Echo of Eons wheels every seat into seven fresh cards.
#[test]
fn echo_of_eons_wheels_everyone() {
    let mut g = pod(3);
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_graveyard(2, catalog::island());
    let echo = g.add_card_to_hand(0, catalog::echo_of_eons());
    flood(&mut g);
    cast(&mut g, echo, None);
    assert!(g.players.iter().all(|p| p.hand.len() == 7));
    assert!(g.players[2].graveyard.is_empty());
}

/// Nine-Lives Familiar enters with eight revival counters when cast and
/// comes back at the next end step with seven.
#[test]
fn nine_lives_familiar_returns_with_one_fewer() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let cat = g.add_card_to_hand(0, catalog::nine_lives_familiar());
    flood(&mut g);
    cast(&mut g, cat, None);
    assert_eq!(g.battlefield_find(cat).unwrap().counter_count(CounterType::Revival), 8);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(cat)));
    assert!(g.battlefield_find(cat).is_none());
    to_end_step(&mut g);
    assert_eq!(g.battlefield_find(cat).expect("returned").counter_count(CounterType::Revival), 7);
}

/// Protean Hydra prevents damage by shedding counters, then regrows two per
/// counter at the next end step.
#[test]
fn protean_hydra_sheds_and_regrows() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let hydra = g.add_card_to_hand(0, catalog::protean_hydra());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: hydra, target: None, additional_targets: vec![], mode: None, x_value: Some(4) })
        .expect("cast");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(hydra)));
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    to_end_step(&mut g);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 7);
}

/// Valakut Exploration: a land drop exiles the top card; the end step bins
/// it and deals 1 to each opponent.
#[test]
fn valakut_exploration_burns_what_went_unplayed() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::valakut_exploration());
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), 1);
    let lives = [g.players[1].life, g.players[2].life];
    to_end_step(&mut g);
    assert!(g.exile.is_empty());
    assert_eq!([g.players[1].life, g.players[2].life], [lives[0] - 1, lives[1] - 1]);
}

/// The Queen of Dale recruits on an opponent's first noncreature spell of
/// the turn — and not on their second.
#[test]
fn queen_of_dale_recruits_once_a_turn_per_opponent() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_queen_of_dale());
    g.decider = Box::new(ScriptedDecider::new([]));
    for n in 0..2 {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), 1, "one recruit discard after spell {n}");
    }
}

/// Liberator grows when a spell cast with more mana than its power.
#[test]
fn liberator_grows_on_big_spells() {
    let mut g = pod(2);
    let lib = ready(&mut g, 0, catalog::liberator_urzas_battlethopter());
    flood(&mut g);
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    assert_eq!(g.computed_permanent(lib).unwrap().power, 1, "1 mana isn't more than 1");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(g.computed_permanent(lib).unwrap().power, 2);
}

/// Ratadrabik turns a dead legend into a nonlegendary 2/2 black Zombie copy.
#[test]
fn ratadrabik_copies_a_dead_legend() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ratadrabik_of_urborg());
    let legend = ready(&mut g, 0, catalog::radagast_the_brown());
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(legend)));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(legend)));
    let tok = g.battlefield.iter().find(|c| c.is_token).expect("copy").id;
    let cp = g.computed_permanent(tok).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(!cp.supertypes().contains(&crabomination::card::Supertype::Legendary));
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Zombie));
    assert!(cp.colors.contains(&Color::Black) && cp.colors.contains(&Color::Green));
}

/// The Cabbage Merchant makes Food off an opponent's noncreature spell and
/// taps two Foods for a mana.
#[test]
fn cabbage_merchant_foods_and_mana() {
    let mut g = pod(2);
    let merchant = ready(&mut g, 0, catalog::the_cabbage_merchant());
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, "Food"), 2);
    let pool = g.players[0].mana_pool.total();
    activate(&mut g, merchant, None);
    assert_eq!(g.players[0].mana_pool.total(), pool + 1);
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Food").all(|c| c.tapped));
}
