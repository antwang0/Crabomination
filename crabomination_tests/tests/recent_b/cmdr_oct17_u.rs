//! Commander: the 2026-10-17 (`3chree`) fifth closer batch
//! (`decks::cmdr_oct17_u`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
    panic!("never reached seat {seat} {step:?}");
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn attack_unblocked(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

// Extra imports beyond the cmdr_oct17_t.rs header: none (CounterType, Keyword,
// DecisionAnswer, ScriptedDecider, GameAction, Target, TurnStep already imported).

/// Heroes Remembered suspends for {W} with ten time counters; when the last
/// comes off it is cast free for 20 life.
#[test]
fn heroes_remembered_suspends_ten_and_gains_twenty() {
    let mut g = pod(2);
    let hr = g.add_card_to_hand(0, catalog::heroes_remembered());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::Suspend { card_id: hr })
        .expect("suspend for {W}");
    drain_stack(&mut g);
    let counters = |g: &GameState| {
        g.exile
            .iter()
            .find(|c| c.id == hr)
            .map(|c| c.counter_count(CounterType::Time))
    };
    assert_eq!(counters(&g), Some(10));
    g.exile
        .iter_mut()
        .find(|c| c.id == hr)
        .unwrap()
        .remove_counters(CounterType::Time, 9);
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(4),
    ));
    let life = g.players[0].life;
    let evs = g.process_suspend();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 20);
}

/// A.I.M. Scientists connives a nonland (a counter); basic landcycling fetches a basic.
#[test]
fn aim_scientists_connives_and_basic_landcycles() {
    let mut g = pod(2);
    let id = g.next_id();
    g.players[0].add_to_library_top(id, catalog::lightning_bolt());
    let s = g.add_card_to_hand(0, catalog::aim_scientists());
    flood(&mut g, 0);
    cast(&mut g, s, None).expect("scientists");
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == id),
        "drew and discarded the Bolt"
    );
    assert_eq!(
        g.battlefield_find(s)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1
    );

    let s2 = g.add_card_to_hand(0, catalog::aim_scientists());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Landcycle { card_id: s2 })
        .expect("basic landcycling {2}");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == s2));
    assert!(
        g.players[0]
            .hand
            .iter()
            .any(|c| c.definition.name == "Forest")
    );
}

/// Panther Pounce: the targeted player gets a Clue; the creature gets +1/+0,
/// flying, and untaps.
#[test]
fn panther_pounce_clue_pump_fly_untap() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let pp = g.add_card_to_hand(0, catalog::panther_pounce());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pp,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("pounce");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield
            .iter()
            .filter(|c| c.controller == 1 && c.definition.name == "Clue")
            .count(),
        1
    );
    assert_eq!(
        g.battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Clue")
            .count(),
        0
    );
    assert_eq!(pt(&g, bear), (3, 2));
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Vivien +2 puts two counters on a creature; −9 gives +4/+4 and trample.
#[test]
fn vivien_of_the_arkbow_counters_and_ultimate() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::vivien_of_the_arkbow());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        x_value: None,
    })
    .expect("+2");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        2
    );
    assert_eq!(
        g.battlefield_find(pw)
            .unwrap()
            .counter_count(CounterType::Loyalty),
        7
    );
    g.battlefield_find_mut(pw)
        .unwrap()
        .add_counters(CounterType::Loyalty, 3);
    g.battlefield_find_mut(pw).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 2,
        target: None,
        x_value: None,
    })
    .expect("-9");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (8, 8));
    assert!(g.permanent_has_keyword(bear, &Keyword::Trample));
}

/// Rhovanion Rampager eats a Hill Giant on attack for three counters; dying
/// as a 3/2 it amasses Goblins 3.
#[test]
fn rhovanion_rampager_sacrifices_and_amasses() {
    let mut g = pod(2);
    let rr = ready(&mut g, 0, catalog::rhovanion_rampager());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let opp = g.players[1].life;
    attack_unblocked(&mut g, vec![(rr, 1)]);
    assert!(g.battlefield_find(giant).is_none(), "sacrificed");
    assert_eq!(
        g.battlefield_find(rr)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        3
    );
    assert_eq!(g.players[1].life, opp - 6);

    let mut g = pod(2);
    let rr = ready(&mut g, 0, catalog::rhovanion_rampager());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(rr))).expect("bolt");
    assert!(g.battlefield_find(rr).is_none());
    let army = g
        .battlefield
        .iter()
        .find(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&crabomination::card::CreatureType::Army)
        })
        .expect("an Army");
    assert_eq!(army.counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Blessed Hippogriff: Tyr's Blessing makes a creature indestructible and the
/// card goes on an adventure; attacking, it gives a ground attacker flying.
#[test]
fn blessed_hippogriff_adventure_and_attack_trigger() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let h = g.add_card_to_hand(0, catalog::blessed_hippogriff());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastAdventure {
        card_id: h,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tyr's blessing");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::Indestructible));
    assert!(g.exile.iter().any(|c| c.id == h), "on an adventure");

    let mut g = pod(2);
    let hip = ready(&mut g, 0, catalog::blessed_hippogriff());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(!g.permanent_has_keyword(bear, &Keyword::Flying));
    attack_unblocked(&mut g, vec![(hip, 1), (bear, 1)]);
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying));
}

/// Ecstatic Awakener sacrifices another creature, draws, and becomes a 4/4 Awoken Demon.
#[test]
fn ecstatic_awakener_sacrifices_draws_and_transforms() {
    let mut g = pod(2);
    let ea = ready(&mut g, 0, catalog::ecstatic_awakener());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    activate(&mut g, ea, None).expect("awaken");
    assert!(g.battlefield_find(bear).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(
        g.battlefield_find(ea).unwrap().definition.name,
        "Awoken Demon"
    );
    assert_eq!(pt(&g, ea), (4, 4));
}

/// Elaborate Firecannon pings for 2, stays tapped through untap, and untaps at
/// upkeep only if you discard.
#[test]
fn elaborate_firecannon_untaps_only_by_discarding() {
    for discard in [false, true] {
        let mut g = pod(2);
        let fc = ready(&mut g, 0, catalog::elaborate_firecannon());
        let land = g.add_card_to_hand(0, catalog::forest());
        flood(&mut g, 0);
        let opp = g.players[1].life;
        activate(&mut g, fc, Some(Target::Player(1))).expect("fire");
        assert_eq!(g.players[1].life, opp - 2);
        if discard {
            g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        }
        step_to(&mut g, 0, TurnStep::Draw);
        assert_eq!(
            g.battlefield_find(fc).unwrap().tapped,
            !discard,
            "discard {discard}"
        );
        assert_eq!(g.players[0].graveyard.iter().any(|c| c.id == land), discard);
    }
}

/// Urborg Repossession kicked returns a creature card and another permanent card, and gains 2.
#[test]
fn urborg_repossession_kicked_returns_two() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let ur = g.add_card_to_hand(0, catalog::urborg_repossession());
    flood(&mut g, 0);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: ur,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(land)],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(g.players[0].hand.iter().any(|c| c.id == land));
    assert_eq!(g.players[0].life, life + 2);
}

/// Sadistic Obsession gives the enchanted creature "{B},{T}: -1/-1 counter".
#[test]
fn sadistic_obsession_grants_a_shrinker() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let so = g.add_card_to_hand(0, catalog::sadistic_obsession());
    flood(&mut g, 0);
    cast(&mut g, so, Some(Target::Permanent(bear))).expect("aura");
    assert_eq!(g.battlefield_find(so).unwrap().attached_to, Some(bear));
    activate(&mut g, bear, Some(Target::Permanent(giant))).expect("granted ability");
    assert_eq!(
        g.battlefield_find(giant)
            .unwrap()
            .counter_count(CounterType::MinusOneMinusOne),
        1
    );
    assert_eq!(pt(&g, giant), (2, 2));
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Whiteout grounds an opposing Serra Angel; saccing a snow land returns it to hand.
#[test]
fn whiteout_grounds_fliers_and_recurs_for_a_snow_land() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let wo = g.add_card_to_hand(0, catalog::whiteout());
    flood(&mut g, 0);
    cast(&mut g, wo, None).expect("whiteout");
    assert!(!g.permanent_has_keyword(angel, &Keyword::Flying));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == wo));
    let snow = g.add_card_to_battlefield(0, catalog::snow_covered_forest());
    activate(&mut g, wo, None).expect("graveyard ability");
    assert!(g.battlefield_find(snow).is_none(), "sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == wo));
}

/// Halo-Charged Skaab mills each player two, then tops a Bolt from your graveyard.
#[test]
fn halo_charged_skaab_mills_and_tops_an_instant() {
    let mut g = pod(2);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let s = g.add_card_to_hand(0, catalog::halo_charged_skaab());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bolt])]));
    cast(&mut g, s, None).expect("skaab");
    assert_eq!(g.players[1].graveyard.len(), 2, "opponent milled two");
    assert_eq!(g.players[0].library[0].id, bolt, "Bolt on top");
}

/// Defang: the enchanted attacker connects for nothing.
#[test]
fn defang_prevents_damage_from_enchanted_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let df = g.add_card_to_hand(0, catalog::defang());
    flood(&mut g, 0);
    cast(&mut g, df, Some(Target::Permanent(bear))).expect("defang");
    assert_eq!(
        g.battlefield_find(df).and_then(|c| c.attached_to),
        Some(bear)
    );
    let opp = g.players[1].life;
    attack_unblocked(&mut g, vec![(bear, 1)]);
    assert_eq!(
        g.players[1].life, opp,
        "all damage from the enchanted creature is prevented"
    );
}

/// Fork in the Road: one basic to hand, the other to the graveyard.
#[test]
fn fork_in_the_road_splits_two_basics() {
    let mut g = pod(2);
    let fork = g.add_card_to_hand(0, catalog::fork_in_the_road());
    flood(&mut g, 0);
    let lib = g.players[0].library.len();
    cast(&mut g, fork, None).expect("fork");
    assert_eq!(g.players[0].library.len(), lib - 2);
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .filter(|c| c.definition.name == "Forest")
            .count(),
        1
    );
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition.name == "Forest")
            .count(),
        1
    );
}

/// Iron Suitcase: {3} turns it into a 3/3 flier.
#[test]
fn iron_suitcase_animates_into_flying_construct() {
    let mut g = pod(2);
    let suit = ready(&mut g, 0, catalog::iron_suitcase());
    flood(&mut g, 0);
    activate(&mut g, suit, None).expect("animate");
    assert_eq!(pt(&g, suit), (3, 3));
    assert!(
        g.computed_permanent(suit)
            .unwrap()
            .keywords()
            .contains(&Keyword::Flying)
    );
}

/// Surtland Frostpyre's sacrifice ability sweeps 2/2s (ability index 1).
#[test]
fn surtland_frostpyre_sacrifices_to_deal_two_to_each_creature() {
    let mut g = pod(2);
    let pyre = ready(&mut g, 0, catalog::surtland_frostpyre());
    g.battlefield_find_mut(pyre).unwrap().tapped = false;
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pyre,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("frostpyre");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pyre).is_none(), "sacrificed");
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(giant).is_some(), "3/3 survives");
}

/// Cement Shoes: +3/+3 and the wearer taps itself at your end step.
#[test]
fn cement_shoes_pumps_and_taps_at_end_step() {
    let mut g = pod(2);
    let shoes = ready(&mut g, 0, catalog::cement_shoes());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip {
        equipment: shoes,
        target: bear,
    })
    .expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (5, 5));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(bear).unwrap().tapped,
        "granted end-step trigger tapped it"
    );
}

/// Wild Shape mode 1: a Grizzly Bears becomes a 1/5 reach Spider.
#[test]
fn wild_shape_spider_mode_sets_base_pt_and_reach() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ws = g.add_card_to_hand(0, catalog::wild_shape());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: ws,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("wild shape");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (1, 5));
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords()
            .contains(&Keyword::Reach)
    );
}

/// Word of Undoing returns the creature to its owner and your white Aura on
/// it to your hand.
#[test]
fn word_of_undoing_returns_creature_and_your_white_aura() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let pac = g.add_card_to_hand(0, catalog::pacifism());
    flood(&mut g, 0);
    cast(&mut g, pac, Some(Target::Permanent(giant))).expect("pacifism");
    let wou = g.add_card_to_hand(0, catalog::word_of_undoing());
    cast(&mut g, wou, Some(Target::Permanent(giant))).expect("word");
    assert!(g.players[1].hand.iter().any(|c| c.id == giant));
    assert!(
        g.players[0].hand.iter().any(|c| c.id == pac),
        "Aura back to your hand, not the graveyard"
    );
}

/// Pyre-Sledge Arsonist deals damage equal to permanents you sacrificed this turn.
#[test]
fn pyre_sledge_arsonist_scales_with_sacrifices() {
    let mut g = pod(2);
    let ars = ready(&mut g, 0, catalog::pyre_sledge_arsonist());
    g.players[0].permanents_sacrificed_this_turn = 3;
    flood(&mut g, 0);
    let opp = g.players[1].life;
    activate(&mut g, ars, Some(Target::Player(1))).expect("arsonist");
    assert_eq!(g.players[1].life, opp - 3);
}

/// Skyrider Patrol: paying {G}{U} at beginning of combat grows and lifts another creature.
#[test]
fn skyrider_patrol_pays_to_grow_another_creature() {
    let mut g = pod(2);
    let patrol = ready(&mut g, 0, catalog::skyrider_patrol());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1
    );
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords()
            .contains(&Keyword::Flying)
    );
    assert_eq!(
        g.battlefield_find(patrol)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        0
    );
}

/// Biblioplex Kraken bounces a friend on attack and becomes unblockable.
#[test]
fn biblioplex_kraken_bounces_to_go_unblockable() {
    let mut g = pod(2);
    let kraken = ready(&mut g, 0, catalog::biblioplex_kraken());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let opp = g.players[1].life;
    attack_unblocked(&mut g, vec![(kraken, 1)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bounced");
    assert!(
        g.computed_permanent(kraken)
            .unwrap()
            .keywords()
            .contains(&Keyword::Unblockable)
    );
    assert_eq!(g.players[1].life, opp - 4);
}

/// Attended Healer makes one Cat on the first life gain, none on the second.
#[test]
fn attended_healer_cat_on_first_life_gain_only() {
    let mut g = pod(2);
    let healer = ready(&mut g, 0, catalog::attended_healer());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(healer);
    let gain = crabomination::effect::Effect::GainLife {
        who: crabomination::effect::Selector::You,
        amount: crabomination::card::Value::Const(2),
    };
    for _ in 0..2 {
        let evs = g.resolve_effect(&gain, &ctx).expect("gain");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    let cats = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Cat")
        .count();
    assert_eq!(cats, 1);
}

/// Yuan-Ti Fang-Blade ventures into the dungeon on combat damage.
#[test]
fn yuan_ti_fang_blade_ventures_on_hit() {
    let mut g = pod(2);
    let snake = ready(&mut g, 0, catalog::yuan_ti_fang_blade());
    assert!(g.players[0].dungeon.is_none());
    attack_unblocked(&mut g, vec![(snake, 1)]);
    assert!(g.players[0].dungeon.is_some(), "entered a dungeon room");
}

fn on_top(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.next_id();
    g.players[seat].add_to_library_top(id, def);
    id
}

/// Bonded Fetch has haste, so it can loot the turn it enters, and it has defender.
#[test]
fn bonded_fetch_loots_with_haste() {
    let mut g = pod(2);
    let fetch = g.add_card_to_battlefield(0, catalog::bonded_fetch());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(g.permanent_has_keyword(fetch, &Keyword::Defender));
    let hand = g.players[0].hand.len();
    activate(&mut g, fetch, None).expect("haste: {T} works while summoning sick");
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Runic Shot can't target an untapped creature. Kicked, it destroys a tapped one.
#[test]
fn runic_shot_needs_a_tapped_target() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let shot = g.add_card_to_hand(0, catalog::runic_shot());
    flood(&mut g, 0);
    assert!(
        cast(&mut g, shot, Some(Target::Permanent(bear))).is_err(),
        "untapped is illegal"
    );
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: shot,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked runic shot");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Academy Raider connecting may rummage: the held card goes, a new one comes.
#[test]
fn academy_raider_rummages_on_combat_damage() {
    let mut g = pod(2);
    let raider = ready(&mut g, 0, catalog::academy_raider());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let opp = g.players[1].life;
    attack_unblocked(&mut g, vec![(raider, 1)]);
    assert_eq!(g.players[1].life, opp - 1);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == bear),
        "discarded"
    );
    assert_eq!(g.players[0].hand.len(), 1, "and drew a replacement");
}

/// Vaultbreaker's attack trigger rummages when accepted.
#[test]
fn vaultbreaker_rummages_on_attack() {
    let mut g = pod(2);
    let vb = ready(&mut g, 0, catalog::vaultbreaker());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_unblocked(&mut g, vec![(vb, 1)]);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Auriok Edgewright gets double strike only once you control three artifacts.
#[test]
fn auriok_edgewright_metalcraft_double_strike() {
    let mut g = pod(2);
    let e = ready(&mut g, 0, catalog::auriok_edgewright());
    ready(&mut g, 0, catalog::ornithopter());
    ready(&mut g, 0, catalog::ornithopter());
    assert!(!g.permanent_has_keyword(e, &Keyword::DoubleStrike));
    ready(&mut g, 0, catalog::ornithopter());
    assert!(g.permanent_has_keyword(e, &Keyword::DoubleStrike));
}

/// Tomb Robber's discard-explore: a land on top goes to hand; a nonland grows it.
#[test]
fn tomb_robber_discards_to_explore() {
    let mut g = pod(2);
    let tr = ready(&mut g, 0, catalog::tomb_robber());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    activate(&mut g, tr, None).expect("explore");
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == bear),
        "discard paid the cost"
    );
    assert!(
        g.players[0]
            .hand
            .iter()
            .any(|c| c.definition.name == "Forest"),
        "land to hand"
    );
    assert_eq!(
        g.battlefield_find(tr)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        0
    );
    on_top(&mut g, 0, catalog::lightning_bolt());
    activate(&mut g, tr, None).expect("explore again (discarding the Forest)");
    assert_eq!(
        g.battlefield_find(tr)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Casting an artifact (historic) makes D'Avenant Trapper tap an opponent's creature.
#[test]
fn davenant_trapper_taps_on_historic_cast() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::davenant_trapper());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let thopter = g.add_card_to_hand(0, catalog::ornithopter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(
        Target::Permanent(bear),
    )]));
    cast(&mut g, thopter, None).expect("ornithopter");
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Exquisite Huntmaster leaves a 1/1 Elf Warrior behind when it dies.
#[test]
fn exquisite_huntmaster_dies_into_an_elf_warrior() {
    let mut g = pod(2);
    let hm = ready(&mut g, 0, catalog::exquisite_huntmaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(hm))).expect("bolt");
    assert!(g.battlefield_find(hm).is_none());
    let elves = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Elf Warrior")
        .count();
    assert_eq!(elves, 1);
}

/// Vault Guardsman exiles an opponent's creature until it leaves the battlefield.
#[test]
fn vault_guardsman_exiles_until_it_leaves() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let vg = g.add_card_to_hand(0, catalog::vault_guardsman());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(
        Target::Permanent(bear),
    )]));
    cast(&mut g, vg, None).expect("guardsman");
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(vg))).expect("murder");
    assert!(
        g.battlefield
            .iter()
            .any(|c| c.controller == 1 && c.definition.name == "Grizzly Bears"),
        "bear returns"
    );
}

/// Trollhide makes a 4/4 that can regenerate itself through a Murder.
#[test]
fn trollhide_pumps_and_grants_regeneration() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let th = g.add_card_to_hand(0, catalog::trollhide());
    flood(&mut g, 0);
    cast(&mut g, th, Some(Target::Permanent(bear))).expect("trollhide");
    assert_eq!(pt(&g, bear), (4, 4));
    activate(&mut g, bear, None).expect("granted {1}{G}: regenerate");
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear))).expect("murder");
    let b = g.battlefield_find(bear).expect("regenerated");
    assert!(b.tapped, "regeneration taps it");
}

/// Aberrant Researcher mills an instant in upkeep and transforms.
#[test]
fn aberrant_researcher_transforms_on_milled_instant() {
    let mut g = pod(2);
    let ar = ready(&mut g, 0, catalog::aberrant_researcher());
    let bolt = on_top(&mut g, 0, catalog::lightning_bolt());
    step_to(&mut g, 0, TurnStep::Upkeep);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "milled"
    );
    assert!(
        g.battlefield_find(ar).unwrap().transformed,
        "became Perfected Form"
    );
    assert_eq!(pt(&g, ar), (5, 4));
}

/// Brambleback Brute enters as a 2/3 with two -1/-1 counters; spending one stops a blocker.
#[test]
fn brambleback_brute_removes_a_counter_to_stop_a_blocker() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bb = g.add_card_to_hand(0, catalog::brambleback_brute());
    flood(&mut g, 0);
    cast(&mut g, bb, None).expect("brute");
    assert_eq!(
        g.battlefield_find(bb)
            .unwrap()
            .counter_count(CounterType::MinusOneMinusOne),
        2
    );
    assert_eq!(pt(&g, bb), (2, 3));
    activate(&mut g, bb, Some(Target::Permanent(bear))).expect("remove a counter");
    assert!(g.permanent_has_keyword(bear, &Keyword::CantBlock));
    assert_eq!(pt(&g, bb), (3, 4));
}
