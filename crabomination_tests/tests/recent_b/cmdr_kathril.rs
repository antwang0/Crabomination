//! Commander: the Symbiotic Swarm precon (C20, Kathril, `decks::cmdr_kathril`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_by(g, 0, id, targets).expect("cast");
}

fn has(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&kw))
}

/// Kathril: a keyword counter (CR 122.1b) per listed keyword in your
/// graveyard's creature cards, and a +1/+1 counter per counter placed.
#[test]
fn kathril_harvests_graveyard_keywords() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::serra_angel()); // flying, vigilance
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let k = g.add_card_to_hand(0, catalog::kathril_aspect_warper());
    cast(&mut g, k, &[]);
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let flying = [bear, k].into_iter().filter(|&id| has(&g, id, Keyword::Flying)).count();
    let vigilance = [bear, k].into_iter().filter(|&id| has(&g, id, Keyword::Vigilance)).count();
    assert_eq!((flying, vigilance), (1, 1));
}

/// Cairn Wanderer has a keyword while a creature card in any graveyard does.
#[test]
fn cairn_wanderer_borrows_from_graveyards() {
    let mut g = pod(2);
    let cw = g.add_card_to_battlefield(0, catalog::cairn_wanderer());
    assert!(!has(&g, cw, Keyword::Flying));
    g.add_card_to_graveyard(1, catalog::serra_angel());
    assert!(has(&g, cw, Keyword::Flying));
    assert!(has(&g, cw, Keyword::Vigilance));
}

/// CR 400.7 — Cairn Wanderer reads a graveyard card as it is in the
/// graveyard: a creature that *had* double strike on the battlefield (a
/// granted keyword) and died lends none. The death snapshot used to be read
/// instead, and its clearing moved the count under the gather memo.
#[test]
fn cr_400_7_cairn_wanderer_reads_graveyard_cards_not_their_last_known_selves() {
    let mut g = pod(2);
    let cw = g.add_card_to_battlefield(0, catalog::cairn_wanderer());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(bear);
    let grant = crabomination::effect::Effect::GrantKeyword {
        what: crabomination::effect::Selector::ExactObjects(vec![bear]),
        keyword: Keyword::DoubleStrike,
        duration: crabomination::effect::Duration::EndOfTurn,
    };
    g.resolve_effect(&grant, &ctx).expect("grant");
    let kill = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::ExactObjects(vec![bear]) };
    let ev = g.resolve_effect(&kill, &ctx).expect("destroy");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
    // Before and after the death snapshots are cleared — the answer must not
    // move with them.
    assert!(!has(&g, cw, Keyword::DoubleStrike), "Grizzly Bears has no double strike in the graveyard");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(!has(&g, cw, Keyword::DoubleStrike));
}

/// Soulflayer: a delved creature card's keywords (CR 702.66).
#[test]
fn soulflayer_keeps_what_it_delved() {
    let mut g = pod(2);
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::soulflayer());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellDelve {
        card_id: s,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        delve_cards: vec![angel],
    })
    .expect("delve");
    drain_stack(&mut g);
    assert!(has(&g, s, Keyword::Flying));
    assert!(!has(&g, s, Keyword::Trample));
}

/// Archon of Valor's Reach: nobody casts the chosen type (CR 601.2 — the
/// cast is illegal).
#[test]
fn archon_bars_the_chosen_type() {
    let mut g = pod(2);
    let a = g.add_card_to_hand(0, catalog::archon_of_valors_reach());
    cast(&mut g, a, &[]);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    assert!(cast_by(&mut g, 1, bolt, &[Target::Player(0)]).is_err(), "instant is the default pick");
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    assert!(cast_by(&mut g, 1, bear, &[]).is_ok());
}

/// Nikara draws when another creature of yours leaves with counters.
#[test]
fn nikara_draws_on_counters_leaving() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::nikara_lair_scavenger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(plain)]);
    assert_eq!(g.players[0].hand.len(), 0);
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Selective Adaptation: one keyworded card per keyword; one to the
/// battlefield, the rest chosen to hand, everything else to the graveyard.
#[test]
fn selective_adaptation_sorts_by_keyword() {
    let mut g = pod(2);
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::selective_adaptation());
    cast(&mut g, s, &[]);
    assert!(g.battlefield_find(angel).is_some());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// Majestic Myriarch counts double and borrows keywords at each combat.
#[test]
fn majestic_myriarch_grows_and_borrows() {
    let mut g = pod(2);
    let mm = g.add_card_to_battlefield(0, catalog::majestic_myriarch());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    assert_eq!(g.computed_permanent(mm).map(|c| (c.power, c.toughness)), Some((4, 4)));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(has(&g, mm, Keyword::Flying));
    assert!(has(&g, mm, Keyword::Vigilance));
}

/// Slippery Bogbonder moves *any number* of your creatures' counters onto its
/// target (CR 122.5): asked per kind, so one of two +1/+1 counters may stay.
#[test]
fn slippery_bogbonder_gathers_counters() {
    let mut g = pod(2);
    let donor = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(donor).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let host = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let sb = g.add_card_to_hand(0, catalog::slippery_bogbonder());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Amount(1),
    ]));
    cast(&mut g, sb, &[Target::Permanent(host)]);
    let t = [donor, host, sb]
        .into_iter()
        .find(|&id| g.battlefield_find(id).is_some_and(|c| !c.keyword_counters.is_empty()))
        .expect("a hexproof counter landed");
    assert!(has(&g, t, Keyword::Hexproof));
    let on = |id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    if t == donor {
        assert_eq!(on(donor), 2, "nothing moves onto its own source");
    } else {
        assert_eq!((on(t), on(donor)), (1, 1), "one of the two moved");
    }
}

/// Netherborn Altar returns your commander to hand for 3 life a soul counter
/// (CR 903.8 — a card in hand is cast normally, tax included).
#[test]
fn netherborn_altar_fetches_the_commander() {
    let mut g = pod(2);
    g.seat_commanders(0, vec![catalog::kathril_aspect_warper()]);
    let na = g.add_card_to_battlefield(0, catalog::netherborn_altar());
    g.perform_action(GameAction::ActivateAbility {
        card_id: na,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("altar");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Kathril, Aspect Warper"));
    assert_eq!(g.players[0].starting_life - g.players[0].life, 3);
}

/// Abzan Ascendancy counters the team and leaves Spirits; Ever After brings
/// one back and goes to the bottom of the library.
#[test]
fn abzan_ascendancy_and_ever_after() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aa = g.add_card_to_hand(0, catalog::abzan_ascendancy());
    cast(&mut g, aa, &[]);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spirit"));
    let ea = g.add_card_to_hand(0, catalog::ever_after());
    cast(&mut g, ea, &[Target::Permanent(bear)]);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(ea), "bottom of its owner's library");
}

/// CR 115.3 — "up to two target creature cards": the bot names two
/// different cards, or one when only one exists (it used to fill both slots
/// with the same card, and the cast was always rejected).
#[test]
fn ever_after_names_distinct_cards() {
    let mut g = pod(2);
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let ea = catalog::ever_after();
    let (t, extra) = g.auto_targets_for_effect_all_slots(&ea.effect, 0, None);
    assert_eq!(t, Some(Target::Permanent(wurm)));
    assert!(extra.is_empty(), "{extra:?}");
}

/// Vitality Hunter — becoming monstrous with X = 2 puts a lifelink counter on
/// each of up to two target creatures: X is bound as the trigger goes on the
/// stack (CR 603.3d), so both targets are picked, any creatures, not the two
/// biggest of yours.
#[test]
fn vitality_hunter_targets_up_to_x() {
    let mut g = pod(2);
    let vh = g.add_card_to_battlefield(0, catalog::vitality_hunter());
    g.add_card_to_battlefield(0, catalog::craw_wurm());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: vh,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("monstrosity 2");
    drain_stack(&mut g);
    let lifelinked = g
        .battlefield
        .iter()
        .filter(|c| c.keyword_counters.get(&Keyword::Lifelink).is_some_and(|n| *n > 0))
        .count();
    assert_eq!(lifelinked, 2, "two targets for X = 2");
}

/// Yannik, Scavenging Sentinel — "exile another creature you control" is the
/// controller's choice as it resolves (CR 608.2d), and X is that creature's
/// power: exiling the Bears puts two counters on the target.
#[test]
fn yannik_exiles_the_chosen_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let yannik = g.add_card_to_battlefield(0, catalog::yannik_scavenging_sentinel());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    let etb = catalog::yannik_scavenging_sentinel().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(yannik, 0, None, 0);
    g.resolve_effect(&etb, &ctx).expect("etb");
    assert!(g.exile.iter().any(|c| c.id == bear), "the chosen one is exiled");
    // CR 603.12 — "when you do" is its own trigger, targeted after the exile.
    retarget_reflexive(&mut g, vec![Target::Permanent(wurm)]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wurm).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Point the reflexive trigger on top of the stack at `targets`, as its
/// controller would pick them.
fn retarget_reflexive(g: &mut GameState, targets: Vec<Target>) {
    match g.stack.last_mut() {
        Some(crabomination::game::types::StackItem::Trigger { target, additional_targets, .. }) => {
            *target = targets.first().cloned();
            *additional_targets = targets.into_iter().skip(1).collect();
        }
        _ => panic!("no reflexive trigger on the stack"),
    }
}

/// The 2020-04-17 ruling: X is the exiled creature's power "as it last
/// existed on the battlefield" — counters and pumps included.
#[test]
fn yannik_reads_the_exiled_creatures_last_power() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let yannik = g.add_card_to_battlefield(0, catalog::yannik_scavenging_sentinel());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    let etb = catalog::yannik_scavenging_sentinel().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(yannik, 0, None, 0);
    g.resolve_effect(&etb, &ctx).expect("etb");
    retarget_reflexive(&mut g, vec![Target::Permanent(wurm)]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wurm).unwrap().counter_count(CounterType::PlusOnePlusOne), 5, "2 + 3 counters");
}

/// CR 608.2d / 111.7 — the same with Yannik's controller PROMPTING and the
/// exiled creature a token: it ceases to exist while the division is asked,
/// and "it will still let the ability distribute counters" (2020-04-17
/// ruling) — its power over the targets still on the battlefield.
#[test]
fn yannik_prompting_controller_picks_and_divides() {
    use crabomination::decision::DecisionAnswer;
    let mut g = pod(2);
    let yannik = g.add_card_to_battlefield(0, catalog::yannik_scavenging_sentinel());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.battlefield_find_mut(bear).unwrap().is_token = true;
    g.players[0].wants_ui = true;
    let etb = catalog::yannik_scavenging_sentinel().triggered_abilities[0].effect.clone();
    g.stack.push(crabomination::game::types::StackItem::Trigger {
        source: yannik,
        controller: 0,
        effect: Box::new(etb),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        trigger_player: None,
        intervening_if: None,
        additional_targets: vec![],
        mana_spent_by_color: Vec::new(),
        activated: false,
        source_transformed_since_push: false,
        ability_id: 0,
    });
    let _ = g.resolve_top_of_stack();
    assert_eq!(g.pending_decision.as_ref().expect("the exile pick").acting_player(), 0);
    g.submit_decision(DecisionAnswer::Cards(vec![bear])).expect("exile the bear");
    retarget_reflexive(&mut g, vec![Target::Permanent(wurm), Target::Permanent(elf)]);
    let _ = g.resolve_top_of_stack();
    assert!(g.pending_decision.is_some(), "the division");
    g.submit_decision(DecisionAnswer::DamageDivision(vec![2, 0])).expect("all on the wurm");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the token was exiled");
    assert_eq!(g.battlefield_find(wurm).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}
