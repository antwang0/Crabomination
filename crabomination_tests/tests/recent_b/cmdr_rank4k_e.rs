//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_e` and `_f`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}


fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)), None);
}

/// Pippin's Bravery: +2/+2, or +4/+4 off a sacrificed Food.
#[test]
fn pippins_bravery_scales_with_a_food() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let p = g.add_card_to_hand(0, catalog::pippins_bravery());
    cast(&mut g, p, Some(Target::Permanent(bear)), None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let food_def = crabomination_base::tokens::food_token();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &crabomination::effect::Effect::CreateToken {
            who: crabomination::effect::PlayerRef::You,
            count: crabomination::effect::Value::ONE,
            definition: std::sync::Arc::new(food_def),
        },
        &ctx,
    )
    .unwrap();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let p = g.add_card_to_hand(0, catalog::pippins_bravery());
    cast(&mut g, p, Some(Target::Permanent(bear)), None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6);
}

/// Dueling Rapier attaches itself on entry (+2/+0).
#[test]
fn dueling_rapier_attaches_on_entry() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let r = g.add_card_to_hand(0, catalog::dueling_rapier());
    cast(&mut g, r, None, None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
}

/// Jessica Jones exiles her power's worth and taps herself down with a stun.
#[test]
fn jessica_jones_impulses_her_power() {
    let mut g = pod(2);
    let jj = ready(&mut g, 0, catalog::jessica_jones_private_eye());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    activate(&mut g, jj, 0, None);
    assert_eq!(g.players[0].library.len(), 2);
    assert_eq!(
        g.battlefield_find(jj).unwrap().counter_count(CounterType::Stun),
        1
    );
}

/// Mona Lisa mints a Mutagen whenever anyone casts a creature spell.
#[test]
fn mona_lisa_mints_on_any_creature_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mona_lisa_ever_adaptable());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None, None);
    assert_eq!(named(&g, 0, "Mutagen"), 1);
}

/// Cloakwood Swarmkeeper counts a batch of tokens once.
#[test]
fn cloakwood_swarmkeeper_counts_a_batch_once() {
    let mut g = pod(4);
    let keeper = ready(&mut g, 0, catalog::cloakwood_swarmkeeper());
    let goliath = g.add_card_to_hand(0, catalog::goblin_goliath());
    cast(&mut g, goliath, None, None);
    assert_eq!(named(&g, 0, "Goblin"), 3);
    assert_eq!(plus(&g, keeper), 1);
}

/// Gardenize banks a charge per death and pays {G} per charge at your first
/// main phase.
#[test]
fn gardenize_pays_green_per_death() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::gardenize());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    kill(&mut g, a);
    kill(&mut g, b);
    g.players[0].mana_pool.empty();
    step(&mut g, TurnStep::PreCombatMain);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2);
}

/// Momo burns a tapped opposing creature; Brimstone Dragon has haste.
#[test]
fn momo_hits_a_tapped_creature() {
    let mut g = pod(2);
    let ogre = ready(&mut g, 1, catalog::hill_giant());
    g.battlefield_find_mut(ogre).unwrap().tapped = true;
    let momo = g.add_card_to_hand(0, catalog::momo_rambunctious_rascal());
    cast(&mut g, momo, None, None);
    assert!(g.battlefield_find(ogre).is_none());
}

/// Smaug's adventure burns for 5, then the Dragon is castable from exile.
#[test]
fn smaug_spews_flame_then_flies() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let smaug = g.add_card_to_hand(0, catalog::smaug_the_great_calamity());
    act(
        &mut g,
        GameAction::CastAdventure {
            card_id: smaug,
            target: Some(Target::Permanent(angel)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
    assert!(g.battlefield_find(angel).is_none());
    assert!(g.exile.iter().any(|c| c.id == smaug));
}

/// Shocker's ETB burns a creature and its controller.
#[test]
fn shocker_shocks_creature_and_controller() {
    let mut g = pod(3);
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::shocker_unshakable());
    cast(&mut g, s, None, None);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[2].life, 18);
}

/// Icefall Regent locks the tapped creature while you control it.
#[test]
fn icefall_regent_locks_its_target() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let r = g.add_card_to_hand(0, catalog::icefall_regent());
    cast(&mut g, r, None, None);
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Sally Pride makes a Mutant per nontoken creature you control.
#[test]
fn sally_pride_counts_nontoken_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::sally_pride_lioness_leader());
    cast(&mut g, s, None, None);
    assert_eq!(named(&g, 0, "Mutant"), 2, "the Bears and Sally herself");
}

/// Renet bounces every other nonland permanent that entered this turn.
#[test]
fn renet_rewinds_this_turns_arrivals() {
    let mut g = pod(2);
    let old = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(old).unwrap().entered_turn = None;
    let new = g.add_card_to_battlefield(1, catalog::hill_giant());
    let now = g.turn_number;
    g.battlefield_find_mut(new).unwrap().entered_turn = Some(now);
    let r = g.add_card_to_hand(0, catalog::renet_temporal_apprentice());
    cast(&mut g, r, None, None);
    assert!(g.battlefield_find(new).is_none());
    assert!(g.battlefield_find(old).is_some());
    assert!(g.battlefield_find(r).is_some());
}

/// Living Brain animates a noncreature artifact into a 3/3.
#[test]
fn living_brain_animates_an_artifact() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::living_brain_mechanical_marvel());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    step(&mut g, TurnStep::BeginCombat);
    let c = g.computed_permanent(ring).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3));
}

/// Trapjaw Tyrant exiles an opposing creature when dealt damage.
#[test]
fn trapjaw_tyrant_bites_back() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::trapjaw_tyrant());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(t)), None);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Spider-UK draws and gains at your end step after two creatures entered.
#[test]
fn spider_uk_rewards_two_arrivals() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::spider_uk());
    g.add_card_to_library(0, catalog::island());
    let guard = g.add_card_to_hand(0, catalog::wakandan_shield_guard());
    cast(&mut g, guard, None, None);
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[0].life, 22);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Iron Lad's discount and Wiccan's impulse on a noncreature spell.
#[test]
fn iron_lad_and_wiccan_reward_noncreature_spells() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::iron_lad_young_avenger());
    ready(&mut g, 0, catalog::wiccan_young_avenger());
    g.add_card_to_library(0, catalog::island());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ring,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sol Ring is free");
    drain_stack(&mut g);
    assert!(g.players[0].library.is_empty(), "Wiccan exiled the top card");
}

/// Summon Undead reanimates a creature card after an optional mill.
#[test]
fn summon_undead_reanimates() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    kill(&mut g, bear);
    let s = g.add_card_to_hand(0, catalog::summon_undead());
    cast(&mut g, s, None, None);
    assert_eq!(named(&g, 0, "Grizzly Bears"), 1);
}
