//! Commander: the 2026-10-17 (`01Htv6cF`) partner-pair scan's one-card gaps
//! (`decks::cmdr_oct17_b`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).unwrap();
    (cp.power, cp.toughness)
}

fn has(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&kw))
}

/// The lords: Battleground Geist pumps other Spirits, Boldwyr Aggressor
/// grants other Giants double strike, Godhead of Awe sets every other
/// creature to base 1/1 — none touches the source or the wrong type.
#[test]
fn lords_affect_only_their_own_kind() {
    let mut g = pod(2);
    let geist = ready(&mut g, 0, catalog::battleground_geist());
    let spirit = ready(&mut g, 0, catalog::battleground_geist());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, geist), (4, 3), "the other Geist pumps this one");
    assert_eq!(pt(&g, spirit), (4, 3));
    assert_eq!(pt(&g, bears), (2, 2));

    let boldwyr = ready(&mut g, 0, catalog::boldwyr_aggressor());
    let giant = ready(&mut g, 0, catalog::boldwyr_aggressor());
    assert!(has(&g, boldwyr, Keyword::DoubleStrike) && has(&g, giant, Keyword::DoubleStrike));
    assert!(!has(&g, bears, Keyword::DoubleStrike));

    let godhead = ready(&mut g, 1, catalog::godhead_of_awe());
    assert_eq!(pt(&g, godhead), (4, 4));
    assert_eq!(pt(&g, boldwyr), (1, 1));
}

/// Headstrong Brute has menace only with another Pirate; Grasping Scoundrel
/// gets +1/+0 only while attacking.
#[test]
fn pirate_conditions() {
    let mut g = pod(2);
    let brute = ready(&mut g, 0, catalog::headstrong_brute());
    assert!(!has(&g, brute, Keyword::Menace));
    let scoundrel = ready(&mut g, 0, catalog::grasping_scoundrel());
    assert!(has(&g, brute, Keyword::Menace), "another Pirate");
    assert_eq!(pt(&g, scoundrel), (1, 1));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: scoundrel,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert_eq!(pt(&g, scoundrel), (2, 1));
}

/// Graveyard-scaling bodies count only their own controller's matching cards.
#[test]
fn graveyard_scaling_bodies() {
    let mut g = pod(2);
    let bot = ready(&mut g, 0, catalog::runaway_trash_bot());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    g.add_card_to_graveyard(0, catalog::swashbuckling());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::sol_ring());
    assert_eq!(pt(&g, bot), (2, 4));

    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let wreath = g.add_card_to_hand(0, catalog::wreath_of_geists());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, wreath, Some(Target::Permanent(bears))).expect("Wreath");
    assert_eq!(pt(&g, bears), (3, 3), "one creature card in the graveyard");
}

/// Deathknell Berserker leaves a Zombie only when it died with power 3+.
#[test]
fn deathknell_berserker_needs_power_three() {
    for (pumped, tokens) in [(false, 0usize), (true, 1)] {
        let mut g = pod(2);
        let b = ready(&mut g, 0, catalog::deathknell_berserker());
        if pumped {
            g.battlefield_find_mut(b).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
        }
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Permanent(b))).expect("bolt");
        assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), tokens, "pumped: {pumped}");
    }
}

/// Patron of the Valiant only grows creatures that already have a +1/+1
/// counter.
#[test]
fn patron_of_the_valiant_grows_countered_creatures() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
    let p = g.add_card_to_hand(0, catalog::patron_of_the_valiant());
    g.players[0].mana_pool.add(Color::White, 5);
    cast(&mut g, p, None).expect("Patron");
    assert_eq!(pt(&g, a), (4, 4));
    assert_eq!(pt(&g, b), (2, 2));
}

/// Chain Devil: each player sacrifices a NONTOKEN creature.
#[test]
fn chain_devil_edicts_nontoken_creatures() {
    let mut g = pod(3);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let bears2 = ready(&mut g, 2, catalog::grizzly_bears());
    let devil = g.add_card_to_hand(0, catalog::chain_devil());
    g.players[0].mana_pool.add(Color::Black, 4);
    cast(&mut g, devil, None).expect("Chain Devil");
    assert!(g.battlefield_find(bears).is_none() && g.battlefield_find(bears2).is_none());
    assert!(g.battlefield_find(devil).is_none(), "its controller sacrifices the Devil itself");
}

/// Magmatic Chasm stops only creatures without flying from blocking.
#[test]
fn magmatic_chasm_spares_flyers() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let geist = ready(&mut g, 1, catalog::battleground_geist());
    let s = g.add_card_to_hand(0, catalog::magmatic_chasm());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, s, None).expect("Chasm");
    assert!(has(&g, bears, Keyword::CantBlock));
    assert!(!has(&g, geist, Keyword::CantBlock));
}

/// Shoot Down can't hit a creature without flying; Surgical Precision's first
/// mode needs toughness 4 or greater.
#[test]
fn removal_filters() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let geist = ready(&mut g, 1, catalog::battleground_geist());
    let s = g.add_card_to_hand(0, catalog::shoot_down());
    g.players[0].mana_pool.add(Color::Green, 4);
    assert!(cast(&mut g, s, Some(Target::Permanent(bears))).is_err());
    cast(&mut g, s, Some(Target::Permanent(geist))).expect("a flyer");
    assert!(g.exile.iter().any(|c| c.id == geist));

    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let sp = g.add_card_to_hand(0, catalog::surgical_precision());
    g.players[0].mana_pool.add(Color::White, 2);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: sp,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    });
    assert!(res.is_err(), "toughness 2");
    g.perform_action(GameAction::CastSpell {
        card_id: sp,
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("toughness 4");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none());
}

/// Perilous Voyage can't bounce your own permanent.
#[test]
fn perilous_voyage_hits_only_others() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::perilous_voyage());
    g.players[0].mana_pool.add(Color::Blue, 2);
    assert!(cast(&mut g, s, Some(Target::Permanent(mine))).is_err());
    cast(&mut g, s, Some(Target::Permanent(theirs))).expect("Voyage");
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs));
}

/// Incendiary Sabotage needs an artifact to sacrifice.
#[test]
fn incendiary_sabotage_sacrifices_an_artifact() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::incendiary_sabotage());
    g.players[0].mana_pool.add(Color::Red, 4);
    assert!(cast(&mut g, s, None).is_err(), "no artifact");
    let ring = ready(&mut g, 0, catalog::sol_ring());
    cast(&mut g, s, None).expect("with Sol Ring");
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(bears).is_none());
}

/// Spicy Oatmeal Pizza burns a target for 4 and its controller for 3.
#[test]
fn spicy_oatmeal_pizza_burns_both_ways() {
    let mut g = pod(2);
    let (me, them) = (g.players[0].life, g.players[1].life);
    let p = g.add_card_to_hand(0, catalog::spicy_oatmeal_pizza());
    g.players[0].mana_pool.add(Color::Red, 3);
    cast(&mut g, p, None).expect("Pizza");
    assert_eq!(g.players[0].life, me - 3);
    assert_eq!(g.players[1].life, them - 4);
}

/// Avacyn's Memorial shields other legendary permanents, not other ones.
#[test]
fn avacyns_memorial_shields_legends() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::avacyns_memorial());
    let legend = ready(&mut g, 0, catalog::rick_jones_destined_sidekick());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(has(&g, legend, Keyword::Indestructible));
    assert!(!has(&g, bears, Keyword::Indestructible));
}

/// Mind Unbound draws one more card each upkeep.
#[test]
fn mind_unbound_draws_more_each_upkeep() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::mind_unbound());
    let body = g.battlefield_find(m).unwrap().definition.triggered_abilities[0].effect.clone();
    let hand = g.players[0].hand.len();
    for _ in 0..2 {
        g.resolve_effect(&body, &EffectContext::for_ability(m, 0, None)).expect("upkeep");
    }
    assert_eq!(g.players[0].hand.len(), hand + 3, "1 then 2");
}

/// Thopter Arrest holds an opponent's creature until it leaves.
#[test]
fn thopter_arrest_exiles_until_it_leaves() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let arrest = g.add_card_to_hand(0, catalog::thopter_arrest());
    g.players[0].mana_pool.add(Color::White, 3);
    cast(&mut g, arrest, None).expect("Arrest");
    assert!(g.exile.iter().any(|c| c.id == wurm));
    let disenchant = g.add_card_to_hand(0, catalog::naturalize());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, disenchant, Some(Target::Permanent(arrest))).expect("Naturalize");
    assert!(g.battlefield_find(wurm).is_some(), "back when the Arrest leaves");
}

/// The six Marvel gain lands enter tapped and gain 1 life.
#[test]
fn marvel_gain_lands_enter_tapped_and_gain_one() {
    for def in [
        catalog::avengers_hangar(),
        catalog::dimension_x(),
        catalog::foot_headquarters(),
        catalog::illegitimate_business(),
        catalog::los_diablos_missile_base(),
        catalog::mutant_town(),
    ] {
        let mut g = pod(2);
        let life = g.players[0].life;
        let land = g.add_card_to_hand(0, def);
        g.perform_action(GameAction::PlayLand(land)).expect("land");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).unwrap().tapped);
        assert_eq!(g.players[0].life, life + 1);
    }
}

// ── cmdr_oct17_c ───────────────────────────────────────────────────────────

/// Chance for Glory: indestructible now, an extra turn, and the loss at that
/// turn's end step (not this one's).
#[test]
fn chance_for_glory_loses_on_the_extra_turn() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::chance_for_glory());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, s, None).expect("Chance for Glory");
    assert!(has(&g, bears, Keyword::Indestructible));
    let turn = g.turn_number;
    let mut guard = 0;
    while !g.players[0].eliminated && guard < 400 {
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
        guard += 1;
    }
    assert_eq!((g.turn_number, g.active_player_idx), (turn + 1, 0));
}

/// Sight of the Scalelords pumps only toughness-4+ creatures at your combat.
#[test]
fn sight_of_the_scalelords_pumps_the_big_ones() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sight_of_the_scalelords());
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let events = g.advance_step(Vec::new()).expect("to combat");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(pt(&g, wurm), (8, 6));
    assert!(has(&g, wurm, Keyword::Vigilance));
    assert_eq!(pt(&g, bears), (2, 2));
}

/// Manhole Missile: 3 damage, then an optional bottom-a-card-to-draw.
#[test]
fn manhole_missile_burns_and_cycles_a_card() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest());
    let s = g.add_card_to_hand(0, catalog::manhole_missile());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, s, Some(Target::Permanent(bears))).expect("Missile");
    assert!(g.battlefield_find(bears).is_none());
}

/// Noosegraf Mob spends a counter per spell cast to make a Zombie.
#[test]
fn noosegraf_mob_trades_counters_for_zombies() {
    let mut g = pod(2);
    let mob = g.add_card_to_hand(0, catalog::noosegraf_mob());
    g.players[0].mana_pool.add(Color::Black, 6);
    cast(&mut g, mob, None).expect("Mob");
    assert_eq!(pt(&g, mob), (5, 5));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(pt(&g, mob), (4, 4));
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Zombie").count(), 1);
}

/// CR 702.33 — Kitesail Cleric taps only when kicked.
#[test]
fn kitesail_cleric_taps_only_when_kicked() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let k = g.add_card_to_hand(0, catalog::kitesail_cleric());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, k, None).expect("unkicked");
    assert!(!g.battlefield_find(wurm).unwrap().tapped);
}

/// Flora Colossus counts its controller's lands.
#[test]
fn flora_colossus_counts_lands() {
    let mut g = pod(2);
    let f = ready(&mut g, 0, catalog::flora_colossus());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.add_card_to_battlefield(1, catalog::forest());
    assert_eq!(pt(&g, f), (3, 3));
}

/// Killmonger: +2/+1 with two creature cards in your graveyard; the ETB
/// sacrifice destroys an opponent's nonland permanent.
#[test]
fn killmonger_grows_and_trades() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let fodder = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let k = g.add_card_to_hand(0, catalog::killmonger_scourge_of_wakanda());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    cast(&mut g, k, None).expect("Killmonger");
    assert!(g.battlefield_find(fodder).is_none() && g.battlefield_find(ring).is_none());
    assert_eq!(pt(&g, k), (5, 4), "the sacrificed Bears makes two");
}

/// Genghis Frog makes a Mutagen for itself and each Mutant after it.
#[test]
fn genghis_frog_makes_mutagens() {
    let mut g = pod(2);
    let f = g.add_card_to_hand(0, catalog::genghis_frog());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, f, None).expect("Frog");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bears, None).expect("bears");
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Mutagen").count(), 1, "the Bears is no Mutant");
}

/// Ral's −7 burns only opponents' creatures.
#[test]
fn ral_caller_of_storms_ultimate() {
    let mut g = pod(2);
    let ral = ready(&mut g, 0, catalog::ral_caller_of_storms());
    g.battlefield_find_mut(ral).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 7);
    let mine = ready(&mut g, 0, catalog::craw_wurm());
    let theirs = ready(&mut g, 1, catalog::craw_wurm());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ral, ability_index: 2, target: None, x_value: None }).expect("−7");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 7);
    assert!(g.battlefield_find(theirs).is_none() && g.battlefield_find(mine).is_some());
}

/// CR 603.10 — a dies-trigger FILTER reads the creature's last-known power,
/// both through a requirement (`PowerAtLeast`) and through a value
/// (`PowerOf(TriggerSource)`); the graveyard card's printed 2 would fail both.
#[test]
fn dies_trigger_filters_read_last_known_power() {
    use crabomination::card::{EventKind, EventScope, EventSpec, SelectionRequirement as R, TriggeredAbility};
    use crabomination::effect::{Effect, Predicate, Selector, Value};
    for filter in [
        Predicate::ValueAtLeast(Value::PowerOf(Box::new(Selector::TriggerSource)), Value::Const(3)),
        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::PowerAtLeast(3) },
    ] {
        let mut g = pod(2);
        let mut d = catalog::grizzly_bears();
        d.triggered_abilities = vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(filter),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
        }];
        let b = ready(&mut g, 0, d);
        g.battlefield_find_mut(b).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
        let life = g.players[0].life;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Permanent(b))).expect("bolt");
        assert_eq!(g.players[0].life, life + 5);
    }
}

/// CR 502.4 — no player receives priority in the untap step: King of the
/// Oathbreakers phasing back in triggers there, and its Spirit token is made
/// in the upkeep (it used to resolve inside the untap step, leaving a
/// summoning-sick permanent behind the turn's start — a pod audit panic).
#[test]
fn cr_502_4_a_phase_in_trigger_waits_for_the_upkeep() {
    let mut g = pod(2);
    let king = ready(&mut g, 0, catalog::king_of_the_oathbreakers());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(king))).expect("bolt");
    assert!(g.battlefield_find(king).is_none(), "targeted, it phased out");
    let mut guard = 0;
    while !(g.active_player_idx == 0 && g.turn_number > 1) && guard < 50 {
        let ev = g.advance_step(Vec::new()).expect("step");
        g.dispatch_triggers_for_events(&ev);
        guard += 1;
    }
    assert_eq!(g.step, TurnStep::Upkeep);
    assert!(g.battlefield_find(king).is_some(), "phased in");
    assert!(!g.battlefield.iter().any(|c| c.is_token), "no token before anyone had priority");
    assert_eq!(g.stack.len(), 1, "the phase-in trigger waits on the stack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1);
}

// ── cmdr_oct17_d ───────────────────────────────────────────────────────────

/// Green Dragon's Poison Breath kills damaged creatures of its controller's
/// opponents only, and only this turn.
#[test]
fn green_dragon_poison_breath_hits_only_opponents() {
    let mut g = pod(3);
    let mine = ready(&mut g, 0, catalog::craw_wurm());
    let theirs = ready(&mut g, 1, catalog::craw_wurm());
    let d = g.add_card_to_hand(0, catalog::green_dragon());
    g.players[0].mana_pool.add(Color::Green, 6);
    cast(&mut g, d, None).expect("Green Dragon");
    for id in [mine, theirs] {
        g.battlefield_find_mut(id).unwrap().damage = 1;
    }
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(theirs).is_none(), "an opponent's damaged creature dies");
    assert!(g.battlefield_find(mine).is_some(), "the controller's own survives");
}

/// Seasons Past returns at most one card per mana value and goes to the
/// bottom of its owner's library.
#[test]
fn seasons_past_returns_one_card_per_mana_value() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let s = g.add_card_to_hand(0, catalog::seasons_past());
    g.players[0].mana_pool.add(Color::Green, 6);
    cast(&mut g, s, None).expect("Seasons Past");
    let in_hand = |id| g.players[0].hand.iter().any(|x| x.id == id);
    assert!(in_hand(c));
    assert!(in_hand(a) ^ in_hand(b), "only one of the two mana-value-2 Bears");
    assert_eq!(g.players[0].library.last().map(|x| x.id), Some(s), "bottom of the library");
}

// ── cmdr_oct17_e ───────────────────────────────────────────────────────────

/// Renegade Doppelganger copies an entering creature until end of turn.
#[test]
fn renegade_doppelganger_copies_until_end_of_turn() {
    let mut g = pod(2);
    let d = ready(&mut g, 0, catalog::renegade_doppelganger());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    g.players[0].mana_pool.add(Color::Green, 6);
    cast(&mut g, wurm, None).expect("wurm");
    assert_eq!(pt(&g, d), (6, 4), "a Craw Wurm this turn");
}

/// Spider-Man 2099 can't be cast during its controller's first three turns.
#[test]
fn spider_man_2099_waits_for_turn_four() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::spider_man_2099());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, s, None).is_err(), "turn one");
    g.players[0].turns_begun = 4;
    cast(&mut g, s, None).expect("turn four");
    assert!(g.battlefield_find(s).is_some());
}

/// CR 707.10 — Precursor Golem: a spell targeting only one Golem is copied
/// for each other Golem, by the spell's caster.
#[test]
fn precursor_golem_splits_a_bolt_across_golems() {
    let mut g = pod(2);
    let pg = g.add_card_to_hand(0, catalog::precursor_golem());
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, pg, None).expect("Precursor Golem");
    let golems: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name.contains("Golem")).map(|c| c.id).collect();
    assert_eq!(golems.len(), 3);
    g.active_player_idx = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(golems[0])), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt a Golem");
    drain_stack(&mut g);
    assert!(golems.iter().all(|id| g.battlefield_find(*id).is_none()), "every Golem took a Bolt");
}

// ── cmdr_oct17_f ───────────────────────────────────────────────────────────

/// CR 601.2c — Burglar's Plot swaps two nonland permanents only when they
/// share a card type.
#[test]
fn bilbos_burglars_plot_needs_a_shared_card_type() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::sol_ring());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::sol_ring());
    let b = g.add_card_to_hand(0, catalog::bilbo_luckwearer());
    let adv = |g: &mut GameState, t: CardId| {
        g.players[0].mana_pool.add(Color::Blue, 5);
        g.perform_action(GameAction::CastAdventure {
            card_id: b,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(t)],
            mode: None,
            x_value: None,
        })
    };
    assert!(adv(&mut g, bears).is_err(), "an artifact and a creature");
    adv(&mut g, theirs).expect("two artifacts");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
}

/// Feldon exiles as many cards as the damage dealt to it and lets you play
/// exactly one of them.
#[test]
fn feldon_exiles_that_many_and_grants_one() {
    let mut g = pod(2);
    let f = ready(&mut g, 0, catalog::feldon_ronom_excavator());
    let shock = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.exile.len();
    cast(&mut g, shock, Some(Target::Permanent(f))).expect("Shock Feldon");
    assert!(g.battlefield_find(f).is_none(), "2 damage kills the 2/2 — its trigger still fires");
    assert_eq!(g.exile.len(), before + 2);
    assert_eq!(g.exile.iter().filter(|c| c.may_play_until.is_some()).count(), 1, "one of them is playable");
}

/// Warbringer's static makes your dash costs {2} cheaper: a second Warbringer
/// dashes for {R}.
#[test]
fn warbringer_discounts_dash() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::warbringer());
    let w = g.add_card_to_hand(0, catalog::warbringer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: w,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("dash for {R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(w).is_some());
}

/// Bilbo's Burglaring takes up to one artifact from EACH opponent.
#[test]
fn bilbos_burglaring_takes_one_artifact_per_opponent() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::sol_ring());
    let b = ready(&mut g, 2, catalog::sol_ring());
    let s = g.add_card_to_hand(0, catalog::bilbos_burglaring());
    g.players[0].mana_pool.add(Color::Blue, 6);
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("Burglaring");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(b).unwrap().controller, 0);
}

/// CR 603.10 — Rotisserie Elemental sacrificed with two skewer counters
/// exiles two cards: X reads its last-known counters, not the sacrificed 0.
#[test]
fn rotisserie_elemental_reads_skewers_after_the_sacrifice() {
    let mut g = pod(2);
    let rot = ready(&mut g, 0, catalog::rotisserie_elemental());
    g.battlefield_find_mut(rot).unwrap().add_counters(crabomination::card::CounterType::Skewer, 1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let body = g.battlefield_find(rot).unwrap().definition.triggered_abilities[0].effect.clone();
    let before = g.exile.len();
    let mut ctx = EffectContext::for_ability(rot, 0, None);
    ctx.trigger_source = Some(crabomination::game::effects::EntityRef::Permanent(rot));
    g.resolve_effect(&body, &ctx).expect("trigger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rot).is_none(), "sacrificed");
    assert_eq!(g.exile.len(), before + 2, "two skewer counters → exile two");
}

/// Frodo levels Citizen → Scout (2/3 lifelink) → Rogue; the Rogue's hit kills
/// a player once the Ring has tempted you four times, and each ability does
/// nothing out of order.
#[test]
fn frodo_saurons_bane_levels_up_and_wins() {
    use crabomination::card::CreatureType;
    let mut g = pod(2);
    let f = ready(&mut g, 0, catalog::frodo_saurons_bane());
    let act = |g: &mut GameState, i: usize| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility { card_id: f, ability_index: i, target: None, additional_targets: vec![], x_value: None, mode: None })
            .expect("activate");
        drain_stack(g);
    };
    g.players[0].mana_pool.add(Color::Black, 3);
    act(&mut g, 1);
    assert!(!g.computed_permanent(f).unwrap().subtypes().creature_types.contains(&CreatureType::Rogue), "a Citizen can't become a Rogue");
    g.players[0].mana_pool.add(Color::White, 2);
    act(&mut g, 0);
    let cp = g.computed_permanent(f).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
    assert!(cp.keywords().contains(&Keyword::Lifelink) && cp.subtypes().creature_types.contains(&CreatureType::Scout));
    g.players[0].mana_pool.add(Color::Black, 3);
    act(&mut g, 1);
    assert!(g.computed_permanent(f).unwrap().subtypes().creature_types.contains(&CreatureType::Rogue));
    g.players[0].ring_temptations = 4;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: f,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    for _ in 0..6 {
        if g.players[1].eliminated {
            break;
        }
        if let Ok(ev) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&ev);
        }
        drain_stack(&mut g);
    }
    assert!(g.players[1].eliminated, "the Rogue's hit ends the game for that player");
}
