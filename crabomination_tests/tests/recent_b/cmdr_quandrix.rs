//! Commander: the Quantum Quandrix precon (C21, Adrix and Nev,
//! `decks::cmdr_quandrix`) and the primitives it needed.

use crabomination::card::{
    CardDefinition, CardId, CardType, CounterType, CreatureType, SelectionRequirement as R,
    StaticAbility, Subtypes, Value,
};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, StaticEffect};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;
use std::sync::Arc;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn soldiers(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(crabomination::card::TokenDefinition {
            name: "Soldier".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
            ..Default::default()
        }),
    }
}

fn run(g: &mut GameState, seat: usize, e: &Effect) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    g.resolve_effect(e, &ctx).expect("resolve");
    drain_stack(g);
}

/// CR 614 — the first token batch on the static's controller's turn becomes
/// that many copies of the greatest-mana-value creature other than its
/// source; the second batch, and a batch on another player's turn, don't.
#[test]
fn cr_614_first_tokens_on_your_turn_become_copies_of_the_chosen_creature() {
    let mut g = pod(3);
    let esix_like = CardDefinition {
        name: "Test Bloom",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "first tokens are copies",
            effect: StaticEffect::FirstTokensOnYourTurnBecomeCopiesOfChosen,
        }],
        ..Default::default()
    };
    g.add_card_to_battlefield(0, esix_like);
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    run(&mut g, 0, &soldiers(2));
    assert_eq!(named(&g, 0, "Serra Angel").len(), 2, "two copies of the biggest creature, anyone's");
    assert!(named(&g, 0, "Soldier").is_empty());
    run(&mut g, 0, &soldiers(1));
    assert_eq!(named(&g, 0, "Soldier").len(), 1, "once a turn");

    let mut g = pod(3);
    g.active_player_idx = 1;
    g.add_card_to_battlefield(
        0,
        CardDefinition {
            name: "Test Bloom",
            card_types: vec![CardType::Creature],
            static_abilities: vec![StaticAbility {
                description: "first tokens are copies",
                effect: StaticEffect::FirstTokensOnYourTurnBecomeCopiesOfChosen,
            }],
            ..Default::default()
        },
    );
    g.add_card_to_battlefield(1, catalog::serra_angel());
    run(&mut g, 0, &soldiers(1));
    assert_eq!(named(&g, 0, "Soldier").len(), 1, "only during your turn");
}

/// CR 614 — after the steal, a token an opponent would create this turn is
/// created under the thief's control (and owned by it, CR 111.2).
#[test]
fn cr_614_an_opponents_tokens_are_created_under_the_thiefs_control() {
    let mut g = pod(3);
    run(&mut g, 0, &Effect::StealOpponentTokensThisTurn);
    run(&mut g, 2, &soldiers(2));
    assert_eq!(named(&g, 0, "Soldier").len(), 2);
    assert!(named(&g, 2, "Soldier").is_empty());
    run(&mut g, 0, &soldiers(1));
    assert_eq!(named(&g, 0, "Soldier").len(), 3, "your own tokens stay yours");
}

/// CR 603.4 — "whenever a nontoken creature an opponent controls enters this
/// turn": an opponent's cast creature fires it, the entering creature is the
/// trigger source; the watcher's own creature doesn't.
#[test]
fn cr_603_4_a_turn_scoped_trigger_watches_other_players_creatures_enter() {
    let mut g = pod(3);
    run(
        &mut g,
        0,
        &Effect::WheneverCreatureEntersThisTurn {
            filter: R::Creature.and(R::ControlledByOpponent).and(R::NotToken),
            body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
        },
    );
    let life = g.players[0].life;
    let bear = g.add_card_to_hand(2, catalog::grizzly_bears());
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    flood(&mut g, 2);
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1);
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: mine, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "not your own creature");
}

/// Each player's token gets the total power of the creatures they controlled
/// that were exiled; a seat with none gets a 0/0 that dies (CR 704.5f).
#[test]
fn exile_all_then_a_token_per_player_by_power() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    run(
        &mut g,
        0,
        &Effect::ExileAllThenTokenPerPlayerByPower {
            filter: R::Creature,
            definition: Arc::new(crabomination::card::TokenDefinition {
                name: "Fractal".into(),
                card_types: vec![CardType::Creature],
                ..Default::default()
            }),
        },
    );
    g.check_state_based_actions();
    let f0 = named(&g, 0, "Fractal");
    assert_eq!(f0.len(), 1);
    assert_eq!(g.battlefield_find(f0[0]).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
    let f1 = named(&g, 1, "Fractal");
    assert_eq!(g.battlefield_find(f1[0]).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(named(&g, 2, "Fractal").is_empty(), "the 0/0 died");
    assert!(named(&g, 0, "Serra Angel").is_empty());
}

// ── The cards ──

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, g.priority.player_with_priority);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn counters(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(kind))
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// X plus one per command-zone commander cast (CR 903.8's count).
#[test]
fn commanders_insight_counts_command_zone_casts() {
    let mut g = pod(3);
    let cmd = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[1].commanders.push(cmd);
    g.commander_cast_count.insert(cmd, 2);
    library(&mut g, 1, 6);
    let hand = g.players[1].hand.len();
    let insight = g.add_card_to_hand(0, catalog::commanders_insight());
    cast(&mut g, 0, insight, Some(Target::Player(1)), Some(3)).expect("cast");
    assert_eq!(g.players[1].hand.len(), hand + 5);
}

/// CR 614 — after Crafty Cutpurse enters, an opponent's tokens are yours.
#[test]
fn crafty_cutpurse_steals_the_turns_tokens() {
    let mut g = pod(3);
    let cutpurse = g.add_card_to_hand(0, catalog::crafty_cutpurse());
    assert!(catalog::crafty_cutpurse().keywords.contains(&crabomination::card::Keyword::Flash));
    cast(&mut g, 0, cutpurse, None, None).expect("cast");
    run(&mut g, 1, &soldiers(1));
    assert_eq!(named(&g, 0, "Soldier").len(), 1);
}

/// A creature token of yours connecting draws a card.
#[test]
fn curiosity_crafter_draws_off_token_hits() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::curiosity_crafter());
    run(&mut g, 0, &soldiers(1));
    let tok = named(&g, 0, "Soldier")[0];
    library(&mut g, 0, 3);
    let hand = g.players[0].hand.len();
    g.clear_sickness(tok);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: tok, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Magecraft: a Fractal with the spell's mana value in counters; {3}{U} makes
/// a creature token unblockable.
#[test]
fn deekah_makes_fractals_from_spells() {
    let mut g = pod(2);
    let deekah = g.add_card_to_battlefield(0, catalog::deekah_fractal_theorist());
    library(&mut g, 0, 3);
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, 0, div, None, None).expect("cast");
    let f = named(&g, 0, "Fractal");
    assert_eq!(f.len(), 1);
    assert_eq!(counters(&g, f[0], CounterType::PlusOnePlusOne), 3);
    activate(&mut g, deekah, 0, Some(Target::Permanent(f[0]))).expect("activate");
    assert!(g.computed_permanent(f[0]).unwrap().keywords().contains(&crabomination::card::Keyword::Unblockable));
}

/// The first token batch on Esix's controller's turn copies another creature.
#[test]
fn esix_turns_the_first_tokens_into_copies() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::esix_fractal_bloom());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    run(&mut g, 0, &soldiers(1));
    assert_eq!(named(&g, 0, "Serra Angel").len(), 2);
}

/// X counters on an equipped Fractal; attacking doubles them.
#[test]
fn fractal_harness_suits_up_a_fractal_and_doubles_it() {
    let mut g = pod(2);
    let harness = g.add_card_to_hand(0, catalog::fractal_harness());
    cast(&mut g, 0, harness, None, Some(2)).expect("cast");
    let f = named(&g, 0, "Fractal")[0];
    assert_eq!(counters(&g, f, CounterType::PlusOnePlusOne), 2);
    let h = named(&g, 0, "Fractal Harness")[0];
    assert_eq!(g.battlefield_find(h).unwrap().attached_to, Some(f));
    g.clear_sickness(f);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: f, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(counters(&g, f, CounterType::PlusOnePlusOne), 4);
}

/// +1 Beast; −3 draws the greatest power; −6 a Wurm per land.
#[test]
fn garruk_primal_hunter_abilities() {
    let mut g = pod(2);
    let garruk = g.add_card_to_battlefield(0, catalog::garruk_primal_hunter());
    let loyalty = |g: &mut GameState, i: usize| {
        g.battlefield_find_mut(garruk).unwrap().loyalty_uses_this_turn = 0;
        g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: garruk, ability_index: i, target: None, x_value: None })
            .expect("loyalty");
        drain_stack(g);
    };
    loyalty(&mut g, 0);
    assert_eq!(named(&g, 0, "Beast").len(), 1);
    library(&mut g, 0, 5);
    let hand = g.players[0].hand.len();
    loyalty(&mut g, 1);
    assert_eq!(g.players[0].hand.len(), hand + 3);
    g.battlefield_find_mut(garruk).unwrap().counters.insert(CounterType::Loyalty, 6);
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    loyalty(&mut g, 2);
    assert_eq!(named(&g, 0, "Wurm").len(), 2);
}

/// Commander creatures get +2/+2 and hexproof; others don't.
#[test]
fn guardian_augmenter_guards_commanders() {
    let mut g = pod(2);
    let cmd = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmd);
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::guardian_augmenter());
    let cp = g.computed_permanent(cmd).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Hexproof));
    assert_eq!(g.computed_permanent(other).unwrap().power, 2);
}

/// Level up (CR 702.87): nothing below level 2, one Elephant at 2-5, two at 6.
#[test]
fn kazandu_tuskcaller_levels_into_elephants() {
    let mut g = pod(2);
    let tusk = g.add_card_to_battlefield(0, catalog::kazandu_tuskcaller());
    g.clear_sickness(tusk);
    assert!(activate(&mut g, tusk, 1, None).is_err(), "level 0");
    activate(&mut g, tusk, 0, None).expect("level up");
    activate(&mut g, tusk, 0, None).expect("level up");
    activate(&mut g, tusk, 1, None).expect("level 2");
    assert_eq!(named(&g, 0, "Elephant").len(), 1);
    g.battlefield_find_mut(tusk).unwrap().tapped = false;
    assert!(activate(&mut g, tusk, 2, None).is_err(), "level 6 ability needs 6");
    g.battlefield_find_mut(tusk).unwrap().counters.insert(CounterType::Level, 6);
    assert!(activate(&mut g, tusk, 1, None).is_err(), "the 2-5 band ends at 5");
    activate(&mut g, tusk, 2, None).expect("level 6");
    assert_eq!(named(&g, 0, "Elephant").len(), 3);
}

/// Every creature exiled; each seat's Fractal has its lost power.
#[test]
fn oversimplify_trades_creatures_for_fractals() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::oversimplify());
    cast(&mut g, 0, spell, None, None).expect("cast");
    assert_eq!(counters(&g, named(&g, 0, "Fractal")[0], CounterType::PlusOnePlusOne), 2);
    assert_eq!(counters(&g, named(&g, 1, "Fractal")[0], CounterType::PlusOnePlusOne), 4);
    assert!(named(&g, 1, "Serra Angel").is_empty());
}

/// One growth counter on entry; each end step doubles them and mints a
/// Fractal of that size.
#[test]
fn paradox_zone_doubles_each_end_step() {
    let mut g = pod(2);
    let zone = g.add_card_to_hand(0, catalog::paradox_zone());
    cast(&mut g, 0, zone, None, None).expect("cast");
    let z = named(&g, 0, "Paradox Zone")[0];
    assert_eq!(counters(&g, z, CounterType::Growth), 1);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(counters(&g, z, CounterType::Growth), 2);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let sizes: Vec<u32> = named(&g, 0, "Fractal").iter().map(|f| counters(&g, *f, CounterType::PlusOnePlusOne)).collect();
    assert_eq!(sizes, vec![2, 4]);
}

/// Draw with the biggest creature on the battlefield; otherwise grow one.
#[test]
fn primal_empathy_draws_or_grows() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::primal_empathy());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    library(&mut g, 0, 2);
    let hand = g.players[0].hand.len();
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 1, "not the biggest: grow");
    assert_eq!(g.players[0].hand.len(), hand);
    g.add_card_to_battlefield(0, catalog::serra_angel());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "tied for the greatest: draw");
}

/// Vanilla creatures get +1/+1 and unblocked damage; entering regrows one.
#[test]
fn ruxa_rewards_vanilla_creatures() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gy_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ruxa = g.add_card_to_hand(0, catalog::ruxa_patient_professor());
    cast(&mut g, 0, ruxa, None, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == gy_bear), "returned to hand");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&crabomination::card::Keyword::AssignsDamageAsThoughUnblocked));
    let r = named(&g, 0, "Ruxa, Patient Professor")[0];
    assert_eq!(g.computed_permanent(r).unwrap().power, 4, "Ruxa has abilities");
}

/// A sea creature of yours connecting makes a 9/9 Kraken.
#[test]
fn spawning_kraken_spawns() {
    let mut g = pod(2);
    let kraken = g.add_card_to_battlefield(0, catalog::spawning_kraken());
    g.clear_sickness(kraken);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: kraken, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    let k: Vec<CardId> = named(&g, 0, "Kraken");
    assert_eq!(k.len(), 1);
    assert_eq!(g.computed_permanent(k[0]).unwrap().power, 9);
}

/// {1},{T}: one mana of any color carrying the commander-scry rider.
#[test]
fn study_hall_makes_rider_mana() {
    let mut g = pod(2);
    let hall = g.add_card_to_battlefield(0, catalog::study_hall());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hall,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for colored");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "the generic cost was paid");
    let r = g.players[0].mana_pool.restricted_breakdown();
    assert_eq!(r.len(), 1);
    assert_eq!((r[0].1, r[0].2), (1, crabomination::mana::SpendRestriction::CommanderCastScry));
}

/// This turn, an opponent's nontoken creature entering is copied for you.
#[test]
fn theoretical_duplication_copies_an_opponents_creature() {
    let mut g = pod(2);
    let dup = g.add_card_to_hand(0, catalog::theoretical_duplication());
    cast(&mut g, 0, dup, None, None).expect("cast");
    let angel = g.add_card_to_hand(1, catalog::serra_angel());
    g.active_player_idx = 1;
    cast(&mut g, 1, angel, None, None).expect("cast");
    assert_eq!(named(&g, 0, "Serra Angel").len(), 1);
    assert_eq!(named(&g, 1, "Serra Angel").len(), 1);
}
