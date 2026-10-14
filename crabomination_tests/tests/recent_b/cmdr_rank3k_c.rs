//! Commander: one-card gaps of EDHREC ranks 2501-3500 (`decks::cmdr_rank3k_c`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
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

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn resolve(g: &mut GameState, effect: &Effect) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn swing(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn try_cast_with(g: &mut GameState, id: CardId, mana: &[(Color, u32)], colorless: u32) -> bool {
    for &(c, n) in mana {
        g.players[0].mana_pool.add(c, n);
    }
    g.players[0].mana_pool.add_colorless(colorless);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .is_ok()
}

/// Traxos costs {2} less after a noncreature spell.
#[test]
fn traxos_discounts_after_a_noncreature_spell() {
    let mut g = pod(2);
    let traxos = g.add_card_to_hand(0, catalog::traxos_academy_guardian());
    let ponder = g.add_card_to_hand(0, catalog::opt());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    cast(&mut g, ponder, None);
    g.players[0].mana_pool.empty();
    assert!(try_cast_with(&mut g, traxos, &[(Color::Blue, 1)], 1), "{{3}}{{U}} less 2");
}

/// Zahid can be cast for {3}{U} by tapping an artifact.
#[test]
fn zahid_taps_an_artifact_for_its_alt_cost() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let zahid = g.add_card_to_hand(0, catalog::zahid_djinn_of_the_lamp());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: zahid,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{3}{U} + tap Sol Ring");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).unwrap().tapped);
    assert_eq!(named(&g, 0, "Zahid, Djinn of the Lamp"), 1);
}

/// Targ Nar doubles its power and toughness.
#[test]
fn targ_nar_doubles() {
    let mut g = pod(2);
    let targ = ready(&mut g, 0, catalog::targ_nar_demon_fang_gnoll());
    activate(&mut g, targ, 0, None);
    assert_eq!(pt(&g, targ), (4, 4));
}

/// Sardian Cliffstomper hits only on your turn with four Mountains.
#[test]
fn sardian_cliffstomper_needs_four_mountains_on_your_turn() {
    let mut g = pod(2);
    let mino = ready(&mut g, 0, catalog::sardian_cliffstomper());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::mountain());
    }
    assert_eq!(pt(&g, mino), (0, 4));
    ready(&mut g, 0, catalog::mountain());
    assert_eq!(pt(&g, mino), (4, 4));
    g.active_player_idx = 1;
    assert_eq!(pt(&g, mino), (0, 4));
}

/// Kingpin makes two Treasures off a sacrifice, once a turn.
#[test]
fn kingpin_pays_once_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kingpin_wilson_fisk());
    for _ in 0..2 {
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        resolve(&mut g, &Effect::SacrificePermanent { what: Selector::ExactObjects(vec![bear]) });
    }
    assert_eq!(named(&g, 0, "Treasure"), 2);
}

/// Ka-Zar brings Zabu, who grows on landfall.
#[test]
fn ka_zar_brings_zabu() {
    let mut g = pod(2);
    let kazar = g.add_card_to_hand(0, catalog::ka_zar_of_the_savage_land());
    cast(&mut g, kazar, None);
    let zabu = g.battlefield.iter().find(|c| c.definition.name == "Zabu").expect("Zabu").id;
    let forest = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(forest));
    assert_eq!(plus(&g, zabu), 1);
}

/// Guul Draz Assassin's -2/-2 needs level 2, its -4/-4 level 4.
#[test]
fn guul_draz_assassin_levels_into_removal() {
    let mut g = pod(2);
    let assassin = ready(&mut g, 0, catalog::guul_draz_assassin());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    g.priority.player_with_priority = 0;
    flood(&mut g);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: assassin,
            ability_index: 1,
            target: Some(Target::Permanent(angel)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "level 0"
    );
    g.battlefield_find_mut(assassin).unwrap().add_counters(CounterType::Level, 4);
    assert_eq!(pt(&g, assassin), (4, 4));
    activate(&mut g, assassin, 2, Some(Target::Permanent(angel)));
    assert!(g.battlefield_find(angel).is_none());
}

/// Geralf's Masterpiece shrinks per card in hand and returns for three
/// discards.
#[test]
fn geralfs_masterpiece_returns_for_three_discards() {
    let mut g = pod(2);
    let horror = g.add_card_to_graveyard(0, catalog::geralfs_masterpiece());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    activate(&mut g, horror, 0, None);
    assert!(g.battlefield_find(horror).unwrap().tapped);
    assert!(g.players[0].hand.is_empty());
    assert_eq!(pt(&g, horror), (7, 7));
    g.add_card_to_hand(0, catalog::island());
    assert_eq!(pt(&g, horror), (6, 6));
}

/// Faramir draws at your end step after one of your creatures died.
#[test]
fn faramir_draws_after_a_death() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::faramir_field_commander());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear)));
    fire(&mut g, TurnStep::End);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Ebondeath can be cast from the graveyard once another creature died.
#[test]
fn ebondeath_returns_after_another_death() {
    let mut g = pod(2);
    let dragon = g.add_card_to_graveyard(0, catalog::ebondeath_dracolich());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    let recast = |g: &mut GameState| {
        g.perform_action(GameAction::CastFlashback {
            card_id: dragon,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(recast(&mut g).is_err(), "nothing died");
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear)));
    flood(&mut g);
    g.priority.player_with_priority = 0;
    recast(&mut g).expect("a creature died");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).unwrap().tapped, "enters tapped");
}

/// Dread destroys an opposing creature that damages you.
#[test]
fn dread_kills_attackers() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::dread());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        // `perform_action` re-dispatches what `advance_step` returns.
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, 18);
    assert!(g.battlefield_find(bear).is_none());
}

/// Dawn of a New Age draws a card each end step and goes out with 4 life.
#[test]
fn dawn_of_a_new_age_counts_down() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let dawn = g.add_card_to_hand(0, catalog::dawn_of_a_new_age());
    cast(&mut g, dawn, None);
    assert_eq!(g.battlefield_find(dawn).unwrap().counter_count(CounterType::Hope), 1);
    fire(&mut g, TurnStep::End);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.battlefield_find(dawn).is_none());
    assert_eq!(g.players[0].life, 24);
}

/// Cloudkill reads the commander's mana value.
#[test]
fn cloudkill_scales_with_the_commander() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::serra_angel());
    g.players[0].commanders.push(cmdr);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let kill = g.add_card_to_hand(0, catalog::cloudkill());
    cast(&mut g, kill, None);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(cmdr).is_none(), "-5/-5 on everything");
    assert!(g.battlefield_find(giant).is_some(), "6/6 survives -5/-5");
}

/// Bofur's adventure makes a creature hexproof and indestructible.
#[test]
fn bofur_concerted_care() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bofur = g.add_card_to_hand(0, catalog::bofur_reliable_guardian());
    act(&mut g, GameAction::CastAdventure {
        card_id: bofur,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Hexproof) && kws.contains(&Keyword::Indestructible));
}

/// Black Panther redirects damage dealt to him.
#[test]
fn black_panther_hits_back() {
    let mut g = pod(2);
    let panther = ready(&mut g, 0, catalog::black_panther_most_dangerous());
    let bolt = Effect::DealDamage { to: Selector::ExactObjects(vec![panther]), amount: Value::Const(2) };
    resolve(&mut g, &bolt);
    assert_eq!(g.players[1].life, 18, "seat 0 at {}", g.players[0].life);
}

/// Bane's Contingency counters; with a commander targeted it also draws.
#[test]
fn banes_contingency_rewards_protecting_the_commander() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::serra_angel());
    g.players[0].commanders.push(cmdr);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let murder = g.add_card_to_hand(1, catalog::murder());
    g.players[1].mana_pool.add(Color::Black, 3);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(cmdr)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("murder");
    let bane = g.add_card_to_hand(0, catalog::banes_contingency());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bane,
        target: Some(Target::Permanent(murder)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cmdr).is_some());
    assert_eq!(g.players[0].hand.len(), 1, "scry 2, draw");
}

/// Ayara's Oathsworn tutors when the fourth counter lands.
#[test]
fn ayaras_oathsworn_tutors_at_four() {
    let mut g = pod(2);
    let knight = ready(&mut g, 0, catalog::ayaras_oathsworn());
    g.battlefield_find_mut(knight).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.add_card_to_library(0, catalog::island());
    swing(&mut g, &[knight], 1);
    assert_eq!(plus(&g, knight), 4);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// April draws per damage and discards one.
#[test]
fn april_draws_per_damage() {
    let mut g = pod(2);
    let april = ready(&mut g, 0, catalog::april_reporter_of_the_weird());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    swing(&mut g, &[april], 1);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Amber Gristle O'Maul trades the hand for a card per attacked player.
#[test]
fn amber_gristle_draws_per_attacked_player() {
    let mut g = pod(3);
    let amber = ready(&mut g, 0, catalog::amber_gristle_omaul());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: amber, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("attack two players");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[0].graveyard.len(), 3);
}

/// Sly Requisitioner makes a Servo when a nontoken artifact dies.
#[test]
fn sly_requisitioner_makes_servos() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sly_requisitioner());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let shatter = g.add_card_to_hand(0, catalog::shatter());
    cast(&mut g, shatter, Some(Target::Permanent(ring)));
    assert_eq!(named(&g, 0, "Servo"), 1);
}

/// Subjugator Angel taps every opposing creature; Shattered Perception
/// wheels the hand.
#[test]
fn subjugator_angel_and_shattered_perception() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let angel = g.add_card_to_hand(0, catalog::subjugator_angel());
    cast(&mut g, angel, None);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    assert!(!g.battlefield_find(mine).unwrap().tapped);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::mountain());
        g.add_card_to_library(0, catalog::island());
    }
    let wheel = g.add_card_to_hand(0, catalog::shattered_perception());
    cast(&mut g, wheel, None);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Island").count(), 2);
}

/// Mister Hyde grows at upkeep (first mode).
#[test]
fn mister_hyde_grows_at_upkeep() {
    let mut g = pod(2);
    let hyde = ready(&mut g, 0, catalog::mister_hyde_monster_within());
    fire(&mut g, TurnStep::Upkeep);
    assert_eq!(plus(&g, hyde), 1);
}

/// Nazar stores feeding counters and cashes three in on attack.
#[test]
fn nazar_feeds_then_draws() {
    let mut g = pod(2);
    let nazar = ready(&mut g, 0, catalog::nazar_the_velvet_fang());
    for _ in 0..3 {
        resolve(&mut g, &Effect::GainLife { who: Selector::You, amount: Value::ONE });
    }
    assert_eq!(g.battlefield_find(nazar).unwrap().counter_count(CounterType::Feeding), 3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    swing(&mut g, &[nazar], 1);
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].life, 20);
}

/// Instrument of the Bards tutors a creature at its harmony count and pays a
/// Treasure for a legend.
#[test]
fn instrument_of_the_bards_tutors_by_harmony() {
    let mut g = pod(2);
    let lute = ready(&mut g, 0, catalog::instrument_of_the_bards());
    g.battlefield_find_mut(lute).unwrap().add_counters(CounterType::Harmony, 4);
    let angel = g.add_card_to_library(0, catalog::traxos_academy_guardian());
    g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, lute, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == angel));
    assert_eq!(named(&g, 0, "Treasure"), 1);
}

/// Popular Entertainer gives your commander a goading combat trigger.
#[test]
fn popular_entertainer_goads_on_damage() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::popular_entertainer());
    let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    let victim = ready(&mut g, 2, catalog::serra_angel());
    swing(&mut g, &[cmdr], 2);
    assert!(g.goaded_by_player(g.battlefield_find(victim).unwrap(), 0));
}

/// Pelargir Survivor's mana pays only for instants and sorceries.
#[test]
fn pelargir_survivor_mana_is_restricted() {
    let mut g = pod(2);
    let peasant = ready(&mut g, 0, catalog::pelargir_survivor());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: peasant,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana");
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err()
    );
}

/// Baru pumps green creatures on a Forest; grandeur makes a land-sized Wurm.
#[test]
fn baru_pumps_and_makes_wurms() {
    let mut g = pod(2);
    let baru = ready(&mut g, 0, catalog::baru_fist_of_krosa());
    let forest = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(forest));
    assert_eq!(pt(&g, baru), (5, 5));
    g.add_card_to_hand(0, catalog::baru_fist_of_krosa());
    activate(&mut g, baru, 0, None);
    let wurm = g.battlefield.iter().find(|c| c.definition.name == "Wurm").expect("Wurm").id;
    assert_eq!(pt(&g, wurm).0, 1, "one land");
}

/// Leatherhead spends her hexproof counter to destroy an artifact.
#[test]
fn leatherhead_trades_her_counter() {
    let mut g = pod(2);
    let gator = g.add_card_to_hand(0, catalog::leatherhead_swamp_stalker());
    cast(&mut g, gator, None);
    g.clear_sickness(gator);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    swing(&mut g, &[gator], 1);
    assert!(g.battlefield_find(ring).is_none());
}

/// The Duke moves his counter and hexproof to another creature.
#[test]
fn the_duke_passes_his_counter() {
    let mut g = pod(2);
    let duke = g.add_card_to_hand(0, catalog::the_duke_rebel_sentry());
    cast(&mut g, duke, None);
    g.clear_sickness(duke);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, duke, 0, Some(Target::Permanent(bear)));
    assert_eq!(plus(&g, bear), 1);
    assert_eq!(plus(&g, duke), 0);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Hexproof));
}

/// Temporary Truce: everyone draws two (auto) or banks life; Green Goblin
/// trades a discard for each opponent's.
#[test]
fn temporary_truce_and_green_goblin() {
    let mut g = pod(3);
    for seat in 0..3 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let truce = g.add_card_to_hand(0, catalog::temporary_truce());
    cast(&mut g, truce, None);
    let total: usize = (0..3).map(|s| g.players[s].hand.len()).sum::<usize>();
    let life: i32 = (0..3).map(|s| g.players[s].life - 20).sum();
    assert_eq!(total as i32 * 2 + life, 12, "each card declined is 2 life");
    ready(&mut g, 0, catalog::green_goblin_back_for_more());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let before: Vec<usize> = (0..3).map(|s| g.players[s].graveyard.len()).collect();
    fire(&mut g, TurnStep::BeginCombat);
    for seat in 1..3 {
        if !g.players[seat].hand.is_empty() || before[seat] != g.players[seat].graveyard.len() {
            assert_eq!(g.players[seat].graveyard.len(), before[seat] + 1, "seat {seat} discards");
        }
    }
}

