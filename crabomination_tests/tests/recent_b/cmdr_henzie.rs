//! Commander: the Riveteers Rampage precon (NCC, Henzie, `decks::cmdr_henzie`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[(CardId, usize)]) -> Result<(), GameError> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&(a, p)| Attack { attacker: a, target: AttackTarget::Player(p) }).collect(),
    ))?;
    drain_stack(g);
    Ok(())
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Henzie (CR 903.3) grants blitz at the spell's mana cost to a creature
/// spell of mana value 4+ (CR 702.152), {1} less per commander cast.
#[test]
fn henzie_grants_discounted_blitz() {
    assert!(catalog::henzie_toolbox_torre().can_be_commander);
    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::henzie_toolbox_torre());
    g.players[0].commanders.push(h);
    g.commander_cast_count.insert(h, 2);
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let alt = |g: &mut GameState, id: CardId| {
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: id,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(alt(&mut g, bear).is_err(), "mana value 2: no blitz");
    // {4}{G}{G} less {2}: exactly four mana.
    g.players[0].mana_pool.add_colorless(2);
    alt(&mut g, wurm).expect("blitz");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some_and(|c| c.blitzed));
}

/// CR 702.152 — a spell with its own blitz and Henzie's has two blitz
/// abilities: Workshop Warchief's printed {4}{G}{G} or its mana cost
/// {3}{G}{G}, and the cheaper one is taken.
#[test]
fn henzie_blitz_beats_a_dearer_printed_blitz() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::henzie_toolbox_torre());
    let rhino = g.add_card_to_hand(0, catalog::workshop_warchief());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: rhino,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("blitz for {3}{G}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rhino).is_some_and(|c| c.blitzed));
}

/// CR 118.9a — only one alternative cost applies, and the caster picks it:
/// Goblin Heelcutter's printed dash, or the blitz Henzie grants it.
#[test]
fn henzie_blitz_or_a_printed_dash() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::henzie_toolbox_torre());
    let dash = g.add_card_to_hand(0, catalog::goblin_heelcutter());
    let blitz = g.add_card_to_hand(0, catalog::goblin_heelcutter());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: dash,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("dash for {2}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dash).is_some_and(|c| c.dashed && !c.blitzed));
    // Blitz at its mana cost, {3}{R}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellGrantedAlternative {
        card_id: blitz,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("blitz for {3}{R}");
    assert_eq!(g.players[0].mana_pool.total(), 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(blitz).is_some_and(|c| c.blitzed && !c.dashed));
}

/// Without a grant there is no granted alternative cost to take.
#[test]
fn a_printed_dash_alone_has_no_granted_alternative() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::goblin_heelcutter());
    flood(&mut g, 0);
    let r = g.perform_action(GameAction::CastSpellGrantedAlternative {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(r.is_err(), "{r:?}");
}

/// Jolene: an attack on your opponent makes the attacker a Treasure, and your
/// own Treasure creations make one more.
#[test]
fn jolene_plunders() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::jolene_the_plunder_queen());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack(&mut g, &[(bear, 1)]).expect("attack");
    assert_eq!(named(&g, 0, "Treasure"), 2, "the attack's Treasure plus Jolene's");
}

/// Grime Gorger exiles one card per card type from the defender's graveyard
/// (CR 205.2a: a card of two types stands for one) and grows per card.
#[test]
fn grime_gorger_eats_one_per_type() {
    let mut g = pod(2);
    let gg = g.add_card_to_battlefield(0, catalog::grime_gorger());
    for f in [catalog::grizzly_bears, catalog::craw_wurm, catalog::lightning_bolt, catalog::forest] {
        g.add_card_to_graveyard(1, f());
    }
    g.add_card_to_graveyard(1, catalog::ornithopter());
    g.clear_sickness(gg);
    attack(&mut g, &[(gg, 1)]).expect("attack");
    // creature, instant, land, and the artifact creature as artifact.
    assert_eq!(g.battlefield_find(gg).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    assert_eq!(g.players[1].graveyard.len(), 1);
}

/// Weathered Sentinels (CR 508.1a) attacks only a player who attacked you
/// during their last turn.
#[test]
fn weathered_sentinels_answers_attackers() {
    let mut g = pod(3);
    let ws = g.add_card_to_battlefield(0, catalog::weathered_sentinels());
    g.clear_sickness(ws);
    assert!(attack(&mut g, &[(ws, 1)]).is_err(), "no one attacked us");
    g.players[2].attacked_players_this_turn.push(0);
    assert!(attack(&mut g, &[(ws, 1)]).is_err(), "seat 1 didn't attack us");
    attack(&mut g, &[(ws, 2)]).expect("seat 2 did");
    assert_eq!(g.computed_permanent(ws).map(|c| c.power), Some(5));
}

/// Turf War: combat damage takes one of the damaged player's contested lands.
#[test]
fn turf_war_takes_contested_land() {
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_battlefield(0, catalog::mountain());
    let tw = g.add_card_to_hand(0, catalog::turf_war());
    cast(&mut g, tw, &[]);
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::Contested), 1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack(&mut g, &[(bear, 1)]).expect("attack");
    finish_combat(&mut g);
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0));
}

/// "One of those lands of their choice": the attacking creature's controller
/// picks the contested land — the Forest here, not the Command Tower the
/// headless default (nonbasic first) would take.
#[test]
fn turf_war_attacker_picks_the_contested_land() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let tower = g.add_card_to_battlefield(1, catalog::command_tower());
    g.add_card_to_battlefield(0, catalog::turf_war());
    for id in [forest, tower] {
        g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Contested, 1);
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack(&mut g, &[(bear, 1)]).expect("attack");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![forest])]));
    finish_combat(&mut g);
    assert_eq!(g.battlefield_find(forest).map(|c| c.controller), Some(0));
    assert_eq!(g.battlefield_find(tower).map(|c| c.controller), Some(1));
}

/// Turf War's entry TARGETS one land per player (CR 601.2c): every seat gets
/// one counter, and a shroud land can't be chosen (CR 702.18a), so its
/// controller's other land takes the counter.
#[test]
fn turf_war_targets_a_land_per_player() {
    use crabomination::card::Keyword;
    let mut g = pod(3);
    let mine = g.add_card_to_battlefield(0, catalog::mountain());
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    let mut tower = catalog::command_tower();
    tower.keywords.push(Keyword::Shroud);
    let shrouded = g.add_card_to_battlefield(2, tower);
    let plains = g.add_card_to_battlefield(2, catalog::plains());
    let tw = g.add_card_to_hand(0, catalog::turf_war());
    cast(&mut g, tw, &[]);
    let n = |g: &GameState, id| g.battlefield_find(id).unwrap().counter_count(CounterType::Contested);
    assert_eq!([n(&g, mine), n(&g, theirs), n(&g, plains)], [1, 1, 1]);
    assert_eq!(n(&g, shrouded), 0, "shroud can't be targeted");
}

/// Bellowing Mauler: at your end step a player without a nontoken creature
/// to sacrifice loses 4.
#[test]
fn bellowing_mauler_taxes() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::bellowing_mauler());
    end_step(&mut g);
    assert_eq!(g.players[1].starting_life - g.players[1].life, 4);
}

/// Wave of Rats returns only after it dealt combat damage to a player.
#[test]
fn wave_of_rats_returns_after_hitting() {
    let mut g = pod(2);
    let w = g.add_card_to_battlefield(0, catalog::wave_of_rats());
    g.clear_sickness(w);
    attack(&mut g, &[(w, 1)]).expect("attack");
    finish_combat(&mut g);
    let m = g.add_card_to_hand(0, catalog::murder());
    g.step = TurnStep::PostCombatMain;
    cast(&mut g, m, &[Target::Permanent(w)]);
    assert!(g.battlefield_find(w).is_some());
    let m = g.add_card_to_hand(0, catalog::murder());
    let w2 = g.add_card_to_battlefield(0, catalog::wave_of_rats());
    cast(&mut g, m, &[Target::Permanent(w2)]);
    assert!(g.battlefield_find(w2).is_none());
}

/// Kresh grows by a dying creature's last-known power (CR 603.10).
#[test]
fn kresh_feeds() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kresh_the_bloodbraided());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(wurm)]);
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
}

/// Caldaia Guardian: a mana value 4+ creature of yours dying makes two
/// Citizens; a two-drop doesn't.
#[test]
fn caldaia_guardian_makes_citizens() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::caldaia_guardian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert_eq!(named(&g, 0, "Citizen"), 0);
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(wurm)]);
    assert_eq!(named(&g, 0, "Citizen"), 2);
}

/// The Beamtown Bullies hand a graveyard creature to the opponent whose turn
/// it is, goaded (CR 701.15), exiled at the next end step.
#[test]
fn beamtown_bullies_loan_a_creature() {
    let mut g = pod(3);
    let bb = g.add_card_to_battlefield(0, catalog::the_beamtown_bullies());
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    // Slot 0 is TARGET opponent whose turn it is: seat 2 isn't (not its turn).
    let act = |g: &mut GameState, opp: usize| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bb,
            ability_index: 0,
            target: Some(Target::Player(opp)),
            additional_targets: vec![Target::Permanent(wurm)],
            mode: None,
            x_value: None,
        })
    };
    assert!(act(&mut g, 2).is_err(), "not the opponent whose turn it is");
    act(&mut g, 1).expect("activate");
    drain_stack(&mut g);
    let c = g.battlefield_find(wurm).expect("loaned");
    assert_eq!(c.controller, 1);
    assert!(g.is_goaded(c));
    end_step(&mut g);
    assert!(g.exile.iter().any(|c| c.id == wurm));
}

/// Industrial Advancement trades a creature for a look at its mana value in
/// cards, deploying a creature from among them.
#[test]
fn industrial_advancement_digs() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::industrial_advancement());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_library(0, catalog::craw_wurm());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Cards(vec![wurm]),
    ]));
    end_step(&mut g);
    assert!(g.battlefield_find(wurm).is_some());
}

/// First Responder's end-step return is a choice made on resolution, not a
/// target: it takes the creature and grows by that creature's power.
#[test]
fn first_responder_returns_a_chosen_creature() {
    let mut g = two_player_game();
    let fr = g.add_card_to_battlefield(0, catalog::first_responder());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let trigger = g.battlefield_find(fr).unwrap().definition.triggered_abilities[0].effect.clone();
    assert!(!trigger.requires_target(), "nothing is targeted");
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    g.resolve_effect(&trigger, &EffectContext::for_ability(fr, 0, None)).expect("end step");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert_eq!(g.battlefield_find(fr).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Next of Kin — "from your hand or from the command zone": a commander of
/// lesser mana value comes in when the enchanted creature dies, and it's not
/// a cast (CR 903.8 — no tax is owed for it).
#[test]
fn next_of_kin_can_bring_the_commander_from_the_command_zone() {
    let mut g = pod(2);
    let ids = g.seat_commanders(0, vec![catalog::henzie_toolbox_torre()]);
    let henzie = ids[0];
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let kin = g.add_card_to_hand(0, catalog::next_of_kin());
    cast(&mut g, kin, &[Target::Permanent(wurm)]);
    let mut events = Vec::new();
    g.destroy_permanent(wurm, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(henzie).is_some(), "Henzie (3) under the Wurm's 6");
    assert_eq!(g.commander_cast_count.get(&henzie).copied().unwrap_or(0), 0, "not a cast");
    end_step(&mut g);
    assert!(g.battlefield_find(kin).is_some_and(|c| c.attached_to == Some(henzie)), "returns attached");
}

/// Protection Racket — per opponent in turn order, reveal your top card and
/// THAT player may pay life equal to its mana value to exile it (else it's
/// yours). Each card is offered to one opponent only.
#[test]
fn protection_racket_asks_each_opponent_once() {
    use crabomination::decision::{DecisionAnswer, DeciderKind, ScriptedDecider};
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::protection_racket());
    let a = g.add_card_to_library(0, catalog::serra_angel());
    let b = g.add_card_to_library(0, catalog::grizzly_bears());
    // Seat 1 is asked about the first card and declines; seat 2 pays for the second.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false), DecisionAnswer::Bool(true)]));
    let life2 = g.players[2].life;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let (kept, lost) = if g.players[0].hand.iter().any(|c| c.id == a) { (a, b) } else { (b, a) };
    assert!(g.players[0].hand.iter().any(|c| c.id == kept), "the first card is yours");
    assert!(g.exile.iter().any(|c| c.id == lost), "seat 2 paid for the second");
    let mv = if lost == a { 5 } else { 2 };
    assert_eq!(g.players[2].life, life2 - mv);
    if let DeciderKind::Scripted { asked, .. } = g.decider.kind() {
        assert_eq!(asked.len(), 2, "one ask per revealed card");
    }
}
