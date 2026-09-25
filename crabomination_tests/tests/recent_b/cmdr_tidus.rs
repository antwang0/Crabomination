//! Commander: the Counter Blitz precon (FIC, Tidus, `decks::cmdr_tidus`), and
//! its primitives: a copy that keeps its name (CR 707.9b), stripping counters
//! from any number of permanents, and "you put" counters (CR 122.6).

use crabomination::card::{CardId, CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    cast_x(g, id, &[], None).expect("castable");
    id
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Resolve "put `n` counters of `kind` on `id`" as `seat`'s effect.
fn put(g: &mut GameState, seat: usize, id: CardId, kind: CounterType, n: i32) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::AddCounter { what: Selector::ExactObjects(vec![id]), kind, amount: Value::Const(n) },
            &ctx,
        )
        .expect("counters");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn kill(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![id]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Attack `defender` with `attackers`, no blocks, through combat damage.
fn connect(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

// ── Tidus ────────────────────────────────────────────────────────────────────

/// Tidus's Cheer draws and proliferates once a turn, however many of your
/// countered creatures connect.
#[test]
fn tidus_cheers_once_a_turn() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::tidus_yunas_guardian());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    put(&mut g, 0, a, CounterType::PlusOnePlusOne, 1);
    put(&mut g, 0, b, CounterType::PlusOnePlusOne, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    connect(&mut g, &[a, b], 1);
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw");
    assert_eq!((plus(&g, a), plus(&g, b)), (2, 2), "proliferated");
}

/// Tidus moves a counter between two of your creatures at your combat.
#[test]
fn tidus_moves_a_counter() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::tidus_yunas_guardian());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    put(&mut g, 0, a, CounterType::PlusOnePlusOne, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!((plus(&g, a), plus(&g, b)), (1, 1));
}

// ── "You put" counters (CR 122.6) ─────────────────────────────────────────────

/// Generous Patron draws when YOU put counters on a creature you don't
/// control — not on your own, and not when its owner does.
#[test]
fn generous_patron_draws_for_counters_on_theirs() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::generous_patron());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    put(&mut g, 0, mine, CounterType::PlusOnePlusOne, 1);
    put(&mut g, 1, theirs, CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.players[0].hand.len(), hand);
    put(&mut g, 0, theirs, CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Rikku makes a creature you put counters on unblockable this turn.
#[test]
fn rikku_counters_make_it_unblockable() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::rikku_resourceful_guardian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    put(&mut g, 1, other, CounterType::PlusOnePlusOne, 1);
    assert!(!g.computed_permanent(other).unwrap().keywords().contains(&Keyword::Unblockable), "not yours");
    put(&mut g, 0, bear, CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// Rikku's Steal moves a counter from an opponent's creature onto yours.
#[test]
fn rikku_steals_a_counter() {
    let mut g = pod(2);
    let rikku = g.add_card_to_battlefield(0, catalog::rikku_resourceful_guardian());
    g.clear_sickness(rikku);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    put(&mut g, 1, theirs, CounterType::PlusOnePlusOne, 2);
    activate(&mut g, rikku, 0, &[Target::Permanent(theirs), Target::Permanent(mine)]).expect("Steal");
    assert_eq!((plus(&g, theirs), plus(&g, mine)), (1, 1));
}

/// CR 122.6 — Lord Jyscal investigates at an end step only if its
/// controller put a counter on a creature that turn.
#[test]
fn lord_jyscal_investigates_on_your_counters() {
    let mut g = pod(2);
    g.turn_number = 4;
    g.add_card_to_battlefield(0, catalog::lord_jyscal_guado());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    put(&mut g, 1, bear, CounterType::PlusOnePlusOne, 1);
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Clue"), 0, "an opponent's counter");
    put(&mut g, 0, bear, CounterType::PlusOnePlusOne, 1);
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Clue"), 1);
}

// ── Copies and counter strips ─────────────────────────────────────────────────

/// CR 707.9b — Kimahri becomes a copy of the creature it tapped, but keeps
/// its name, vigilance, its counters and the ability.
#[test]
fn kimahri_copies_but_keeps_its_name() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kimahri_valiant_guardian());
    let giant = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::BeginCombat);
    assert!(g.battlefield_find(giant).unwrap().tapped);
    assert_eq!(g.battlefield_find(k).unwrap().definition.name, "Kimahri, Valiant Guardian");
    let c = g.computed_permanent(k).unwrap();
    assert!(c.keywords().contains(&Keyword::Vigilance));
    assert_eq!((c.power, c.toughness), (7, 5), "a 6/4 Craw Wurm plus Kimahri's counter");
    assert_eq!(g.battlefield_find(k).unwrap().definition.triggered_abilities.len(), 1, "this ability");
}

/// Sin strips counters from opponents' permanents (the headless pick) and
/// enters with twice as many.
#[test]
fn sin_enters_with_twice_the_counters_removed() {
    let mut g = pod(3);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    put(&mut g, 1, a, CounterType::PlusOnePlusOne, 2);
    put(&mut g, 2, b, CounterType::PlusOnePlusOne, 1);
    put(&mut g, 0, mine, CounterType::PlusOnePlusOne, 1);
    let sin = cast(&mut g, catalog::sin_unending_cataclysm());
    assert_eq!((plus(&g, a), plus(&g, b), plus(&g, mine)), (0, 0, 1));
    assert_eq!(plus(&g, sin), 6);
}

/// Sin dying hands its counters to one of your creatures and shuffles into
/// its owner's library.
#[test]
fn sin_passes_its_counters_on_death() {
    let mut g = pod(2);
    let sin = g.add_card_to_battlefield(0, catalog::sin_unending_cataclysm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    put(&mut g, 0, sin, CounterType::PlusOnePlusOne, 4);
    kill(&mut g, sin);
    assert_eq!(plus(&g, bear), 4);
    assert!(g.players[0].library.iter().any(|c| c.id == sin), "shuffled in");
}

// ── The Summons ──────────────────────────────────────────────────────────────

/// Summon: Valefor's chapter I bounces each opponent's greatest-mana-value
/// creature, and only theirs.
#[test]
fn valefor_bounces_each_opponents_biggest() {
    let mut g = pod(3);
    let big = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::craw_wurm());
    cast(&mut g, catalog::summon_valefor());
    assert!(g.players[1].hand.iter().any(|c| c.id == big));
    assert!(g.battlefield_find(small).is_some());
    assert!(g.players[2].hand.iter().any(|c| c.id == other));
    assert!(g.battlefield_find(mine).is_some(), "yours stay");
}

/// Summon: Yojimbo's chapter IV makes a Treasure per opponent with a
/// power-4 creature.
#[test]
fn yojimbo_treasure_per_big_opponent() {
    let mut g = pod(4);
    g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.add_card_to_battlefield(2, catalog::craw_wurm());
    g.add_card_to_battlefield(2, catalog::craw_wurm());
    g.add_card_to_battlefield(3, catalog::grizzly_bears());
    let y = g.add_card_to_battlefield(0, catalog::summon_yojimbo());
    g.battlefield_find_mut(y).unwrap().add_counters(CounterType::Lore, 3);
    put(&mut g, 0, y, CounterType::Lore, 1);
    assert_eq!(named(&g, 0, "Treasure"), 2);
}

/// Summoner's Sending: a big creature card exiled from a graveyard makes a
/// Spirit with a counter.
#[test]
fn summoners_sending_spirit_grows_on_a_big_card() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::summoners_sending());
    g.add_card_to_graveyard(1, catalog::craw_wurm());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::End);
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit").map(|c| c.id).expect("a Spirit");
    assert_eq!(plus(&g, spirit), 1);
    assert!(g.players[1].graveyard.is_empty());
}

// ── Counter engines ──────────────────────────────────────────────────────────

/// Tromell: your other nontoken creatures enter with a counter, and its
/// proliferate counts the nontoken creatures that entered this turn.
#[test]
fn tromell_adds_a_counter_and_proliferates_per_entrant() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::tromell_seymours_butler());
    g.clear_sickness(t);
    let a = cast(&mut g, catalog::grizzly_bears());
    let b = cast(&mut g, catalog::grizzly_bears());
    assert_eq!((plus(&g, a), plus(&g, b)), (1, 1));
    activate(&mut g, t, 0, &[]).expect("proliferate X");
    assert_eq!((plus(&g, a), plus(&g, b)), (3, 3), "X = 2");
}

/// Wakka's Blitzball Captain: a turn Wakka got a counter, your other
/// creatures get one at your end step.
#[test]
fn wakka_captain_spreads_counters() {
    let mut g = pod(2);
    g.turn_number = 3;
    let wakka = g.add_card_to_battlefield(0, catalog::wakka_devoted_guardian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    step(&mut g, TurnStep::End);
    assert_eq!(plus(&g, bear), 0);
    put(&mut g, 0, wakka, CounterType::PlusOnePlusOne, 1);
    step(&mut g, TurnStep::End);
    assert_eq!((plus(&g, bear), plus(&g, wakka)), (1, 1));
}

/// Yuna: another permanent of yours dying with counters passes that many on.
#[test]
fn yuna_passes_on_a_dead_permanents_counters() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::yuna_grand_summoner());
    let dying = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let heir = g.add_card_to_battlefield(0, catalog::craw_wurm());
    put(&mut g, 0, dying, CounterType::PlusOnePlusOne, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Target(Target::Permanent(heir))]));
    kill(&mut g, dying);
    assert_eq!(plus(&g, heir), 3);
}

/// Grand Summon: the next creature spell this turn enters with two more
/// +1/+1 counters.
#[test]
fn yuna_grand_summon_grows_the_next_creature() {
    let mut g = pod(2);
    let yuna = g.add_card_to_battlefield(0, catalog::yuna_grand_summoner());
    g.clear_sickness(yuna);
    activate(&mut g, yuna, 0, &[]).expect("Grand Summon");
    let a = cast(&mut g, catalog::grizzly_bears());
    let b = cast(&mut g, catalog::grizzly_bears());
    assert_eq!((plus(&g, a), plus(&g, b)), (2, 0));
}

/// Shelinda: a smaller entrant gets the counter, an equal one gives it to
/// Shelinda.
#[test]
fn shelinda_grows_the_smaller() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::shelinda_yevon_acolyte());
    let elf = cast(&mut g, catalog::llanowar_elves());
    assert_eq!((plus(&g, elf), plus(&g, s)), (1, 0));
    cast(&mut g, catalog::grizzly_bears());
    assert_eq!(plus(&g, s), 1, "a 2/2 isn't smaller");
}

/// Maester Seymour puts its power in counters on another creature of yours.
#[test]
fn maester_seymour_shares_its_power() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::maester_seymour());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!((plus(&g, bear), plus(&g, s)), (1, 0));
}

/// O'aka draws for a counter removed from a nonland permanent you control.
#[test]
fn oaka_spends_a_counter_to_draw() {
    let mut g = pod(2);
    let o = g.add_card_to_battlefield(0, catalog::oaka_traveling_merchant());
    g.clear_sickness(o);
    let hand = g.players[0].hand.len();
    assert!(activate(&mut g, o, 0, &[]).is_err(), "no counter to remove");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    put(&mut g, 0, bear, CounterType::PlusOnePlusOne, 1);
    activate(&mut g, o, 0, &[]).expect("draw");
    assert_eq!((g.players[0].hand.len(), plus(&g, bear)), (hand + 1, 0));
}

/// Gatta and Luzzu: damage to the chosen creature becomes +1/+1 counters.
#[test]
fn gatta_and_luzzu_turn_damage_into_counters() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gl = g.add_card_to_hand(0, catalog::gatta_and_luzzu());
    cast_x(&mut g, gl, &[], None).expect("flash creature");
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::DealDamage { to: Selector::ExactObjects(vec![bear]), amount: Value::Const(3) },
            &ctx,
        )
        .expect("damage");
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(plus(&g, bear), 3);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0);
}

/// Yuna's Whistle finds a creature card for your hand and puts its mana
/// value in counters on one of your creatures.
#[test]
fn yunas_whistle_counts_the_find() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::craw_wurm());
    let lib = g.players[0].library.len();
    cast(&mut g, catalog::yunas_whistle());
    assert!(g.players[0].hand.iter().any(|c| c.id == top));
    assert_eq!(g.players[0].library.len(), lib - 1);
    assert_eq!(plus(&g, bear), 6);
}

/// Blitzball Stadium: support X on entry. Bug fix: the "up to N targets"
/// filler read a trigger's `CapTargetsAt` X as 0 (CR 601.2b — a permanent's
/// trigger reads the X stamped on it), so only the first slot was filled.
#[test]
fn blitzball_stadium_supports_x() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::blitzball_stadium());
    cast_x(&mut g, s, &[], Some(2)).expect("X = 2");
    assert_eq!(plus(&g, a) + plus(&g, b) + plus(&g, c), 2);
}

/// Auron's attack exiles a smaller creature of the defending player's until
/// Auron leaves. Bug fix (CR 603.7d): a reflexive trigger picked its targets
/// without its source, so "power less than Auron's" matched nothing.
#[test]
fn auron_exiles_a_smaller_defender() {
    let mut g = pod(3);
    let auron = g.add_card_to_battlefield(0, catalog::auron_venerated_guardian());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elsewhere = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    connect(&mut g, &[auron], 1);
    assert!(g.battlefield_find(small).is_none(), "exiled");
    assert!(g.battlefield_find(elsewhere).is_some(), "not the defending player's");
    kill(&mut g, auron);
    assert!(g.battlefield_find(small).is_some(), "back when Auron leaves");
}

/// Lulu stuns a creature attacking you.
#[test]
fn lulu_stuns_an_attacker() {
    let mut g = pod(2);
    g.active_player_idx = 1;
    g.add_card_to_battlefield(0, catalog::lulu_stern_guardian());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Stun), 1);
}

