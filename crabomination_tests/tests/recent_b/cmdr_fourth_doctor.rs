//! Commander: the Blast from the Past precon (WHO, The Fourth Doctor + Sarah
//! Jane Smith, `decks::cmdr_fourth_doctor`), and its primitives: granted read
//! ahead (CR 714.3c), a non-legendary spell copy (CR 707.9b), a guess against
//! a live threshold with an else branch, one added counter of a chosen kind
//! per permanent, and "counter all other spells" without a draw.

use crabomination::card::{CardId, CounterType, Keyword, Supertype, Value};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("castable");
    drain_stack(g);
    id
}

/// Resolve `source`'s `i`th triggered ability as if it fired.
fn fire(g: &mut GameState, source: CardId, i: usize) {
    let effect = g.battlefield_find(source).unwrap().definition.triggered_abilities[i].effect.clone();
    let ctx = EffectContext::for_ability(source, 0, None);
    let evs = g.resolve_effect(&effect, &ctx).expect("resolves");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

fn count(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(kind))
}

/// CR 714.3c — Barbara Wright gives your Sagas read ahead: The Sea Devils
/// can start on chapter II, skipping chapter I's Salamander.
#[test]
fn barbara_wright_gives_your_sagas_read_ahead() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::barbara_wright());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(2)]));
    let saga = cast(&mut g, catalog::the_sea_devils());
    assert_eq!(count(&g, saga, CounterType::Lore), 2);
    assert_eq!(named(&g, "Alien Salamander"), 1, "only chapter II fired");
}

/// Without Barbara the same Saga starts at chapter I.
#[test]
fn a_saga_without_read_ahead_starts_at_one() {
    let mut g = pod(2);
    let saga = cast(&mut g, catalog::the_sea_devils());
    assert_eq!(count(&g, saga, CounterType::Lore), 1);
    assert_eq!(named(&g, "Alien Salamander"), 1);
}

/// CR 707.9b — The Sixth Doctor copies a legendary spell, the copy not
/// legendary, so both survive the legend rule.
#[test]
fn the_sixth_doctor_copies_a_historic_spell_without_legendary() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_sixth_doctor());
    cast(&mut g, catalog::jo_grant());
    assert_eq!(named(&g, "Jo Grant"), 2);
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Jo Grant" && c.is_token).expect("a token copy");
    assert!(!copy.definition.supertypes.contains(&Supertype::Legendary));
    cast(&mut g, catalog::barbara_wright());
    assert_eq!(named(&g, "Barbara Wright"), 1, "once each turn");
}

/// The Seventh Doctor: with no card in hand no spell is cast, so it
/// investigates.
#[test]
fn the_seventh_doctor_investigates_when_nothing_is_cast() {
    let mut g = pod(2);
    let doc = g.add_card_to_battlefield(0, catalog::the_seventh_doctor());
    g.players[0].hand.clear();
    fire(&mut g, doc, 0);
    assert_eq!(named(&g, "Clue"), 1);
}

/// Attacking, the defending player guesses: a right guess ("greater" for a
/// 5-drop over zero artifacts) casts nothing and investigates; a wrong one
/// casts the card free.
#[test]
fn the_seventh_doctor_casts_on_a_wrong_guess() {
    for (guess, clues, angels) in [(true, 1, 0), (false, 0, 1)] {
        let mut g = pod(2);
        let doc = g.add_card_to_battlefield(0, catalog::the_seventh_doctor());
        g.clear_sickness(doc);
        g.players[0].hand.clear();
        g.add_card_to_hand(0, catalog::serra_angel());
        g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(guess)]));
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: doc, target: AttackTarget::Player(1) }]))
            .expect("attack");
        drain_stack(&mut g);
        assert_eq!((named(&g, "Clue"), named(&g, "Serra Angel")), (clues, angels), "guess {guess}");
    }
}

/// The Caves of Androzani II: one more counter of a kind already there on
/// each non-Saga permanent — good ones on yours, bad ones on theirs, none on
/// a Saga.
#[test]
fn the_caves_of_androzani_adds_a_counter_to_each_permanent() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let stunned = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let saga = g.add_card_to_battlefield(0, catalog::the_sea_devils());
    for (id, kind) in [
        (mine, CounterType::PlusOnePlusOne),
        (theirs, CounterType::PlusOnePlusOne),
        (stunned, CounterType::Stun),
        (saga, CounterType::Lore),
    ] {
        g.battlefield_find_mut(id).unwrap().add_counters(kind, 1);
    }
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let def = catalog::the_caves_of_androzani();
    let evs = g.resolve_effect(&def.saga_chapters[1].1, &ctx).expect("chapter II");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(count(&g, mine, CounterType::PlusOnePlusOne), 2);
    assert_eq!(count(&g, theirs, CounterType::PlusOnePlusOne), 1, "not an opponent's +1/+1");
    assert_eq!(count(&g, stunned, CounterType::Stun), 2);
    assert_eq!(count(&g, saga, CounterType::Lore), 1, "non-Saga permanents only");
}

/// Reverse the Polarity's first mode counters every other spell and draws
/// nothing (unlike Swift Silence).
#[test]
fn counter_all_other_spells_draws_nothing() {
    let mut g = pod(2);
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    let hand = g.players[0].hand.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::CounterAllOtherSpells, &ctx).expect("counter");
    g.dispatch_triggers_for_events(&evs);
    assert!(g.stack.is_empty());
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears));
    assert_eq!(g.players[0].hand.len(), hand);
}

/// Reverse the Polarity's third mode is a rule for the turn (CR 611.2c): a
/// creature that enters after it resolves can't be blocked either.
#[test]
fn reverse_the_polarity_covers_creatures_that_arrive_later() {
    let mut g = pod(2);
    let there = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::reverse_the_polarity());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: Some(2), x_value: None })
        .expect("castable");
    drain_stack(&mut g);
    let later = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for c in [there, later] {
        assert!(g.computed_permanent(c).unwrap().keywords().contains(&Keyword::Unblockable));
    }
}

/// Gallifrey Stands: at upkeep a Doctor comes down from hand, and thirteen
/// Doctors win the game.
#[test]
fn gallifrey_stands_wins_with_thirteen_doctors() {
    let mut g = pod(2);
    let gs = g.add_card_to_battlefield(0, catalog::gallifrey_stands());
    for _ in 0..12 {
        g.add_card_to_battlefield(0, catalog::the_fifth_doctor());
    }
    g.add_card_to_hand(0, catalog::the_first_doctor());
    let effect = g.battlefield_find(gs).unwrap().definition.triggered_abilities[1].effect.clone();
    let ctx = EffectContext::for_ability(gs, 0, None);
    let _ = g.resolve_effect(&effect, &ctx).expect("upkeep");
    assert_eq!(named(&g, "The First Doctor"), 1, "put onto the battlefield from hand");
    assert!(g.players[1].eliminated, "CR 104.2a — you win: every opponent is out");
}

/// The Fifth Doctor: at your end step, a creature that attacked this turn
/// gets nothing; one that sat out gets a counter and untaps.
#[test]
fn the_fifth_doctor_rewards_the_creatures_that_rested() {
    let mut g = pod(2);
    let doc = g.add_card_to_battlefield(0, catalog::the_fifth_doctor());
    let rested = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [doc, rested, attacker] {
        g.clear_sickness(id);
        g.battlefield_find_mut(id).unwrap().entered_turn = None;
    }
    g.battlefield_find_mut(rested).unwrap().tapped = true;
    g.battlefield_find_mut(attacker).unwrap().attacked_this_turn = true;
    fire(&mut g, doc, 0);
    assert_eq!(count(&g, rested, CounterType::PlusOnePlusOne), 1);
    assert!(!g.battlefield_find(rested).unwrap().tapped);
    assert_eq!(count(&g, attacker, CounterType::PlusOnePlusOne), 0);
}

/// The War Games I: every player gets three tapped Warriors, goaded.
#[test]
fn the_war_games_gives_everyone_goaded_warriors() {
    let mut g = pod(3);
    cast(&mut g, catalog::the_war_games());
    for p in 0..3 {
        let warriors: Vec<_> = g.battlefield.iter().filter(|c| c.controller == p && c.definition.name == "Warrior").collect();
        assert_eq!(warriors.len(), 3, "seat {p}");
        assert!(warriors.iter().all(|c| c.tapped));
    }
}

/// Trenzalore Clocktower: each tap adds a time counter; twelve pay for the
/// wheel once you control a Time Lord.
#[test]
fn trenzalore_clocktower_needs_twelve_time_counters_and_a_time_lord() {
    let mut g = pod(2);
    let tower = g.add_card_to_battlefield(0, catalog::trenzalore_clocktower());
    g.battlefield_find_mut(tower).unwrap().add_counters(CounterType::Time, 12);
    flood(&mut g, 0);
    let wheel = GameAction::ActivateAbility {
        card_id: tower,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    assert!(g.perform_action(wheel.clone()).is_err(), "no Time Lord");
    g.add_card_to_battlefield(0, catalog::susan_foreman());
    g.perform_action(wheel).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tower).is_none());
    assert_eq!(g.players[0].hand.len(), 7);
}

/// Sarah Jane Smith investigates on the first historic spell each turn only.
#[test]
fn sarah_jane_smith_investigates_once_a_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sarah_jane_smith());
    cast(&mut g, catalog::sol_ring());
    cast(&mut g, catalog::mind_stone());
    cast(&mut g, catalog::grizzly_bears());
    assert_eq!(named(&g, "Clue"), 1);
}

/// Vrestin enters with X counters and X flying Insects.
#[test]
fn vrestin_brings_x_insects() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::vrestin_menoptra_leader());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(count(&g, id, CounterType::PlusOnePlusOne), 3);
    assert_eq!(named(&g, "Alien Insect"), 3);
}

/// Crisis of Conscience's first mode destroys only tokens.
#[test]
fn crisis_of_conscience_destroys_tokens() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: std::sync::Arc::new(crabomination_base::tokens::clue_token()),
            },
            &ctx,
        )
        .expect("clues");
    g.dispatch_triggers_for_events(&evs);
    let def = catalog::crisis_of_conscience();
    let Effect::ChooseMode(modes) = &def.effect else { panic!("modal") };
    let evs = g.resolve_effect(&modes[0], &ctx).expect("tokens");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Clue"), 0);
    assert!(g.battlefield_find(bear).is_some());
    let _ = Selector::This;
}

/// Nyssa of Traken — sacrifice any number of artifacts; WHEN YOU DO (CR
/// 603.12), tap up to that many target creatures: the reflexive trigger
/// targets as it goes on the stack, capped at the count, and you draw that many.
#[test]
fn nyssa_of_traken_taps_up_to_that_many_targets() {
    let mut g = pod(2);
    let nyssa = g.add_card_to_battlefield(0, catalog::nyssa_of_traken());
    let a = g.add_card_to_battlefield(0, catalog::ornithopter());
    let b = g.add_card_to_battlefield(0, catalog::ornithopter());
    let bears: Vec<CardId> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    let hand = g.players[0].hand.len();
    fire(&mut g, nyssa, 0);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 2);
    let tapped = bears.iter().filter(|&&id| g.battlefield_find(id).unwrap().tapped).count();
    assert_eq!(tapped, 2, "up to two targets: no more than were sacrificed");
}

/// The Second Doctor — "each opponent who [draws] can't attack YOU or
/// permanents you control during their next turn" (CR 508.1a): a ban on the
/// drawing PLAYER, so a creature they get afterwards is barred too, and it
/// reaches the Doctor's controller's planeswalkers; it holds on their next
/// turn only, and an opponent who declines is free.
#[test]
fn the_second_doctor_bars_the_drawer_from_attacking_you() {
    let mut g = pod(3);
    let doc = g.add_card_to_battlefield(0, catalog::the_second_doctor());
    let walker = g.add_card_to_battlefield(0, catalog::teyo_geometric_tactician());
    let declined = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false), // you
        DecisionAnswer::Bool(true),  // player 2 draws
        DecisionAnswer::Bool(false), // player 3 doesn't
    ]));
    fire(&mut g, doc, 0);
    let later = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let attack = |g: &GameState, seat: usize, id: CardId, target: AttackTarget| {
        let mut g = g.clone();
        g.turn_number += 1;
        g.active_player_idx = seat;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = seat;
        g.clear_sickness(id);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: id, target }])).is_ok()
    };
    assert!(!attack(&g, 1, later, AttackTarget::Player(0)), "can't attack the Doctor's controller");
    assert!(!attack(&g, 1, later, AttackTarget::Planeswalker(walker)), "nor their planeswalker");
    assert!(attack(&g, 1, later, AttackTarget::Player(2)), "another opponent is fair game");
    assert!(attack(&g, 2, declined, AttackTarget::Player(0)), "the decliner is free");
    // The ban ends with the drawer's next turn.
    let mut after = g.clone();
    after.turn_number += 1;
    after.active_player_idx = 1;
    after.step = TurnStep::Cleanup;
    let _ = after.do_cleanup(&mut Vec::new());
    assert!(attack(&after, 1, later, AttackTarget::Player(0)));
}

/// CR 614.12 — Displaced Dinosaurs: a historic permanent enters already a
/// 7/7 Dinosaur creature (no trigger on the stack), an artifact included;
/// a nonhistoric one is untouched.
#[test]
fn displaced_dinosaurs_historic_permanents_enter_as_7_7_dinosaurs() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::displaced_dinosaurs());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("castable");
    g.resolve_top_of_stack().expect("resolve");
    assert!(g.stack.is_empty(), "no trigger");
    let cp = g.computed_permanent(ring).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Dinosaur));
    let bear = cast(&mut g, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "not historic");
}

/// CR 702.51 / 700.6 — Peri Brown: only the FIRST historic spell each turn
/// has convoke. A Sol Ring convoked with a Bear goes through; a second
/// historic spell that turn can't convoke.
#[test]
fn peri_brown_convokes_only_the_first_historic_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::peri_brown());
    let bears: Vec<CardId> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    let convoke = |g: &mut GameState, id: CardId, helper: CardId| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpellConvoke {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
            convoke_creatures: vec![helper],
        })
    };
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    convoke(&mut g, ring, bears[0]).expect("the first historic spell convokes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears[0]).unwrap().tapped, "the Bear paid");
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    g.players[0].mana_pool.add_colorless(1);
    assert!(convoke(&mut g, stone, bears[1]).is_err(), "the second one this turn doesn't");
}

/// The Curse of Fenric I — "for each creature destroyed this way, its
/// controller creates a 3/3 Mutant": the Bears' controller gets one, the
/// indestructible creature's controller doesn't (CR 702.12b).
#[test]
fn curse_of_fenric_mutants_only_for_the_destroyed() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut tough = catalog::grizzly_bears();
    tough.keywords.push(Keyword::Indestructible);
    let tough = g.add_card_to_battlefield(2, tough);
    let saga = catalog::the_curse_of_fenric();
    let chapter = saga.saga_chapters[0].1.clone();
    let src = g.add_card_to_battlefield(0, saga);
    let mut ctx = EffectContext::for_ability(src, 0, None);
    ctx.targets = vec![
        crabomination::game::types::Target::Permanent(bear),
        crabomination::game::types::Target::Permanent(tough),
    ];
    g.resolve_effect(&chapter, &ctx).expect("resolves");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(tough).is_some());
    let mutants = |seat| g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == "Mutant").count();
    assert_eq!((mutants(1), mutants(2)), (1, 0));
}

/// The Curse of Fenric II — the target becomes a 6/6 legendary Horror with
/// no abilities (CR 613.1d/f layers 4, 6 and 7b).
#[test]
fn curse_of_fenric_ii_makes_a_legendary_horror() {
    let mut g = pod(2);
    let mut flier = catalog::grizzly_bears();
    flier.keywords.push(Keyword::Flying);
    let bear = g.add_card_to_battlefield(1, flier);
    let saga = catalog::the_curse_of_fenric();
    let chapter = saga.saga_chapters[1].1.clone();
    let src = g.add_card_to_battlefield(0, saga);
    let ctx = EffectContext::for_ability(src, 0, Some(crabomination::game::types::Target::Permanent(bear)));
    g.resolve_effect(&chapter, &ctx).expect("resolves");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6));
    assert!(cp.supertypes().contains(&Supertype::Legendary));
    assert_eq!(cp.subtypes().creature_types, vec![crabomination::card::CreatureType::Horror]);
    assert!(!cp.keywords().contains(&Keyword::Flying));

/// The Fourth Doctor — "once each turn, you may play a historic land or cast
/// a historic spell from the top of your library. When you do, create a
/// Food": a land played that way makes one too, the grant is spent, and a
/// historic spell off the top the next turn makes the next.
#[test]
fn the_fourth_doctor_feeds_a_land_or_a_spell_played_off_the_top() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_fourth_doctor());
    g.players[0].library.clear();
    let cradle = g.add_card_to_library(0, catalog::gaeas_cradle());
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    g.perform_action(GameAction::PlayLand(cradle)).expect("a historic land off the top");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Food"), 1, "a land played this way feeds too");
    flood(&mut g, 0);
    let cast_ring = GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None };
    assert!(g.perform_action(cast_ring.clone()).is_err(), "once each turn");
    g.players[0].cast_from_library_top_this_turn = false;
    g.perform_action(cast_ring).expect("the next turn's charge");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Food"), 2);
}

/// The Fourth Doctor's Food is for its own grant only: a historic spell cast
/// off the top by another permission (Mystic Forge) makes none.
#[test]
fn the_fourth_doctor_ignores_a_top_cast_another_permission_allowed() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_fourth_doctor());
    g.add_card_to_battlefield(0, catalog::mystic_forge());
    g.players[0].library.clear();
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Mystic Forge covers an artifact");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some());
    assert_eq!(named(&g, "Food"), 0);
}

/// Peri Brown — CR 702.51: only "the first historic spell you cast each
/// turn" has convoke; the second historic spell that turn pays in mana.
#[test]
fn peri_brown_convokes_only_the_first_historic_spell_each_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::peri_brown());
    let helpers: Vec<CardId> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    let convoke = |card_id, convoke_creatures| GameAction::CastSpellConvoke {
        card_id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        convoke_creatures,
    };
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.perform_action(convoke(ring, vec![helpers[0]])).expect("the first historic spell convokes");
    drain_stack(&mut g);
    let signet = g.add_card_to_hand(0, catalog::arcane_signet());
    assert!(g.perform_action(convoke(signet, vec![helpers[1], helpers[2]])).is_err(), "the second doesn't");
    assert!(!g.battlefield_find(helpers[1]).unwrap().tapped);
}

/// The Eighth Doctor — one allowance a turn for "a historic land or a
/// historic permanent spell" from the graveyard (a nonhistoric land isn't
/// covered), and the permanent it lets you cast gains "if this permanent
/// would leave the battlefield, exile it instead" (CR 614.1a).
#[test]
fn the_eighth_doctor_shares_one_allowance_and_exiles_what_it_returns() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_eighth_doctor());
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    let cradle = g.add_card_to_graveyard(0, catalog::gaeas_cradle());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    assert!(g.perform_action(GameAction::PlayLandFromGraveyard(forest)).is_err(), "not historic");
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("a historic permanent spell from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some());
    assert!(g.perform_action(GameAction::PlayLandFromGraveyard(cradle)).is_err(), "the cast spent the allowance");
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![ring]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == ring), "exiled instead of put into the graveyard");
}

/// The Eighth Doctor — the allowance taken as a historic land instead.
#[test]
fn the_eighth_doctor_plays_a_historic_land_from_the_graveyard() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_eighth_doctor());
    let cradle = g.add_card_to_graveyard(0, catalog::gaeas_cradle());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    g.perform_action(GameAction::PlayLandFromGraveyard(cradle)).expect("a historic land");
    assert!(g.battlefield_find(cradle).is_some());
    flood(&mut g, 0);
    let cast = GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None };
    assert!(g.perform_action(cast).is_err(), "one allowance, not one each");
}
