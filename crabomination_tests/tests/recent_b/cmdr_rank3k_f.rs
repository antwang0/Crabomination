//! Commander: one- and two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_f`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
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

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)));
}

fn resolve(g: &mut GameState, effect: &Effect) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

/// Imperious Mindbreaker pairs and mills by toughness on attack.
#[test]
fn imperious_mindbreaker_mills_by_toughness() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for seat in 1..3 {
        for _ in 0..6 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let mind = g.add_card_to_hand(0, catalog::imperious_mindbreaker());
    cast(&mut g, mind, None);
    g.clear_sickness(mind);
    declare(&mut g, vec![(mind, 1), (bear, 1)]);
    assert_eq!(g.players[1].graveyard.len(), 6, "4 + 2 from the pair");
    assert_eq!(g.players[2].graveyard.len(), 6);
}

/// Mizzix copies a spell cast from outside your hand.
#[test]
fn mizzix_copies_a_graveyard_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mizzix_replica_rider());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let _ = (bear, angel);
    let think = g.add_card_to_graveyard(0, catalog::think_twice());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    act(&mut g, GameAction::CastFlashback { card_id: think, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert_eq!(g.players[0].hand.len(), 2, "Think Twice and its copy");
}

/// Dual Strike copies the next cheap instant.
#[test]
fn dual_strike_copies_the_next_instant() {
    let mut g = pod(2);
    let strike = g.add_card_to_hand(0, catalog::dual_strike());
    cast(&mut g, strike, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14);
}

/// Embermaw Hellion adds one to another red source's damage.
#[test]
fn embermaw_hellion_amplifies_red() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::embermaw_hellion());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 16);
}

/// Ajani's Anguish deals X on entry and gives your creatures trample.
#[test]
fn ajanis_anguish_burns_and_tramples() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let anguish = g.add_card_to_hand(0, catalog::ajanis_anguish());
    act(&mut g, GameAction::CastSpell {
        card_id: anguish,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    });
    assert_eq!(g.players[1].life, 17);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// Iron Man pumps attackers and makes a Robot per red spell; Hawkeye pings
/// per Hero as Heroes arrive.
#[test]
fn iron_man_and_hawkeye() {
    let mut g = pod(2);
    let stark = ready(&mut g, 0, catalog::iron_man_tony_stark());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(named(&g, 0, "Robot Hero"), 1);
    declare(&mut g, vec![(stark, 1)]);
    assert_eq!(pt(&g, stark), (4, 3));
    let mut g = pod(2);
    ready(&mut g, 0, catalog::iron_man_tony_stark());
    let hawk = g.add_card_to_hand(0, catalog::hawkeye_trick_shot());
    cast(&mut g, hawk, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17, "Iron Man's Robot Hero lands first: three Heroes");
}

/// Doomskar Warrior digs by its combat damage; Hamlet Captain rallies Humans.
#[test]
fn karvanista_pair() {
    let mut g = pod(2);
    let doom = ready(&mut g, 0, catalog::doomskar_warrior());
    let capt = ready(&mut g, 0, catalog::hamlet_captain());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    declare(&mut g, vec![(doom, 1), (capt, 1)]);
    assert_eq!(pt(&g, doom), (5, 4), "Hamlet Captain's rally");
    finish_combat(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Tigra grows on life gain; Staff of the Wild Magus gains on green spells
/// and Forests.
#[test]
fn tigra_pair() {
    let mut g = pod(2);
    let tigra = ready(&mut g, 0, catalog::tigra_feline_fury());
    ready(&mut g, 0, catalog::staff_of_the_wild_magus());
    let forest = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(forest));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(g.players[0].life, 22);
    assert_eq!(plus(&g, tigra), 2);
}

/// Donatello grows on artifacts; Krang refills the hand to four and counts
/// artifacts.
#[test]
fn april_oneil_turtles() {
    let mut g = pod(2);
    let don = ready(&mut g, 0, catalog::donatello_way_with_machines());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    assert_eq!(plus(&g, don), 1);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::island());
    let krang = g.add_card_to_hand(0, catalog::krang_master_mind());
    cast(&mut g, krang, None);
    assert_eq!(g.players[0].hand.len(), 4);
    assert_eq!(pt(&g, krang).0, 2, "one other artifact");
}

/// Iroh pumps the attackers; Reckless Blaze sweeps and pays {R} per death of
/// yours.
#[test]
fn uncle_iroh_pair() {
    let mut g = pod(2);
    let iroh = ready(&mut g, 0, catalog::iroh_firebending_instructor());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(iroh, 1), (bear, 1)]);
    assert_eq!(pt(&g, bear), (3, 3));
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    let blaze = g.add_card_to_hand(0, catalog::reckless_blaze());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: blaze, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| !c.definition.is_creature()));
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "one of yours died");
}

/// Casey Jones doubles his power for anyone's {4}; Improvised Arsenal counts
/// artifacts and copies itself.
#[test]
fn casey_jones_asphalt_pair() {
    let mut g = pod(2);
    let casey = ready(&mut g, 0, catalog::casey_jones_asphalt_hooligan());
    activate(&mut g, casey, 0, None);
    assert_eq!(pt(&g, casey).0, 4);
    let arsenal = ready(&mut g, 0, catalog::improvised_arsenal());
    act(&mut g, GameAction::Equip { equipment: arsenal, target: casey });
    assert_eq!(pt(&g, casey).0, 5);
    activate(&mut g, arsenal, 0, None);
    assert_eq!(named(&g, 0, "Improvised Arsenal"), 2);
    assert_eq!(pt(&g, casey).0, 6);
}

/// Black Widow mills on entry and drains by creature cards on a hit.
#[test]
fn black_widow_daring_operative_drains() {
    let mut g = pod(3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let widow = g.add_card_to_hand(0, catalog::black_widow_daring_operative());
    cast(&mut g, widow, None);
    g.clear_sickness(widow);
    declare(&mut g, vec![(widow, 1)]);
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 20 - 3 - 3);
    assert_eq!(g.players[2].life, 17);
}

/// Koth gains on landfall and counters on a Plains; Fall of the Thran
/// razes, then regrows two lands each.
#[test]
fn koth_and_fall_of_the_thran() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::koth_of_the_homestead());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let plains = g.add_card_to_hand(0, catalog::plains());
    act(&mut g, GameAction::PlayLand(plains));
    assert_eq!(g.players[0].life, 21);
    let counters: u32 = g.battlefield.iter().map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
    assert_eq!(counters, 1);
    let _ = bear;
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..3 {
            ready(&mut g, seat, catalog::forest());
        }
    }
    let fall = g.add_card_to_hand(0, catalog::fall_of_the_thran());
    cast(&mut g, fall, None);
    assert!(g.battlefield.iter().all(|c| !c.definition.is_land()));
    let def = catalog::fall_of_the_thran();
    let ii = def.saga_chapters.iter().find(|(n, _)| *n == 2).unwrap().1.clone();
    resolve(&mut g, &ii);
    assert_eq!(named(&g, 0, "Forest"), 2);
    assert_eq!(named(&g, 1, "Forest"), 2);
}

/// Lim-Dûl pays {1}{B} to take a dying opposing creature as a Zombie.
#[test]
fn lim_dul_steals_the_dead() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::lim_dul_the_necromancer());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    kill(&mut g, bear);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("returned");
    assert_eq!(back.controller, 0);
    let id = back.id;
    assert!(g.computed_permanent(id).unwrap().subtypes().creature_types.contains(&crabomination::card::CreatureType::Zombie));
}

/// Fraying Omnipotence halves life, hand and creatures, rounding up.
#[test]
fn fraying_omnipotence_halves_everything() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 1, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::fraying_omnipotence());
    cast(&mut g, spell, None);
    assert_eq!(g.players[1].life, 10);
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(named(&g, 1, "Grizzly Bears"), 1);
}

/// Thor exiles cards off noncombat damage to an opponent.
#[test]
fn thor_exiles_off_noncombat_damage() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thor_guardian_of_midgard());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[0].library.len(), 0, "three exiled");
}

/// Magus of the Future lets you cast off the top of your library.
#[test]
fn magus_of_the_future_casts_from_the_top() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::magus_of_the_future());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    act(&mut g, GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(g.battlefield_find(bear).is_some());
}

