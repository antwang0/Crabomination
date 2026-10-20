//! Reality Fracture (`sets::fra`): the main-set cards the Commander work
//! didn't reach.

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Bots aim hostile player slots at an opponent (the auto-targeter's
    // profile switch; off is the ladder control).
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>, mode: Option<usize>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    cast_with(g, id, target, vec![], None)
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: idx, target, additional_targets: vec![], x_value: None, mode: None })?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: idx, target, x_value: None })?;
    drain_stack(g);
    Ok(())
}

/// Resolve `e` as seat 0's and put its triggers on the stack.
fn run(g: &mut GameState, e: Effect) {
    let evs = g.resolve_effect(&e, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn empower(g: &mut GameState, n: i32) {
    run(g, Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(n) });
}

fn jace(g: &GameState, seat: usize) -> Option<CardId> {
    g.battlefield.iter().find(|c| c.controller == seat && c.definition.name == "Jace").map(|c| c.id)
}

fn jace_loyalty(g: &GameState, seat: usize) -> Option<u32> {
    jace(g, seat).map(|j| g.battlefield_find(j).unwrap().counter_count(CounterType::Loyalty))
}

fn token(g: &GameState, seat: usize, name: &str) -> Option<CardId> {
    g.battlefield.iter().find(|c| c.controller == seat && c.definition.name == name).map(|c| c.id)
}

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).unwrap().power
}

fn to_step(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// CR 701.71a — each "enters, empower Jace N" permanent makes a Jace token
/// and puts N loyalty counters on it.
#[test]
fn cr_701_71a_entering_empowers_jace() {
    let rows: [(fn() -> CardDefinition, u32); 6] = [
        (catalog::mindseeker_oculus, 4),
        (catalog::theorists_proxy, 3),
        (catalog::arcane_amphisbaena, 2),
        (catalog::keeper_of_the_quiet_hour, 2),
        (catalog::way_of_the_deathbringer, 5),
        (catalog::way_of_the_wildspeaker, 7),
    ];
    for (def, n) in rows {
        let mut g = pod(2);
        let name = def().name;
        let id = g.add_card_to_hand(0, def());
        flood(&mut g, 0);
        cast(&mut g, id, None).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(jace_loyalty(&g, 0), Some(n), "{name}");
    }
}

/// The empowering spells do their own half, then empower Jace.
#[test]
fn empowering_spells_resolve_both_halves() {
    let bear_up = |g: &GameState, b: CardId| power(g, b) == 4 && g.permanent_has_keyword(b, &Keyword::Flying);
    let face = |g: &GameState, _: CardId| g.players[1].life == 17;
    let counter = |g: &GameState, b: CardId| {
        g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne) == 1
            && g.permanent_has_keyword(b, &Keyword::Vigilance)
    };
    let rows: [(fn() -> CardDefinition, bool, u32, fn(&GameState, CardId) -> bool); 3] = [
        (catalog::academic_ascent, true, 2, bear_up),
        (catalog::no_admittance, false, 1, face),
        (catalog::tams_resistance, true, 4, counter),
    ];
    for (def, at_bear, n, check) in rows {
        let mut g = pod(2);
        let name = def().name;
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def());
        flood(&mut g, 0);
        let target = if at_bear { Target::Permanent(bear) } else { Target::Player(1) };
        cast(&mut g, id, Some(target)).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert!(check(&g, bear), "{name}");
        assert_eq!(jace_loyalty(&g, 0), Some(n), "{name}");
    }
}

/// The color-hosers: each takes only its two colors, and removes them.
#[test]
fn color_hosers_take_only_their_colors() {
    // (card, a legal creature, an illegal one, exiled rather than dead)
    let rows: [(fn() -> CardDefinition, fn() -> CardDefinition, fn() -> CardDefinition, bool); 3] = [
        (catalog::refute_destiny, catalog::grizzly_bears, catalog::hill_giant, true),
        (catalog::terminal_criticism, catalog::hill_giant, catalog::grizzly_bears, false),
        (catalog::essence_burn, catalog::grizzly_bears, catalog::hill_giant, true),
    ];
    for (def, ok, bad, exiled) in rows {
        let mut g = pod(2);
        let name = def().name;
        let good = ready(&mut g, 1, ok());
        let wrong = ready(&mut g, 1, bad());
        let id = g.add_card_to_hand(0, def());
        flood(&mut g, 0);
        assert!(cast(&mut g, id, Some(Target::Permanent(wrong))).is_err(), "{name}: wrong color");
        cast(&mut g, id, Some(Target::Permanent(good))).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert!(g.battlefield_find(good).is_none(), "{name}");
        assert_eq!(g.exile.iter().any(|c| c.id == good), exiled, "{name}");
    }
}

/// Terminal Criticism's rider: you gain 1 life.
#[test]
fn terminal_criticism_gains_a_life() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let tc = g.add_card_to_hand(0, catalog::terminal_criticism());
    flood(&mut g, 0);
    cast(&mut g, tc, Some(Target::Permanent(giant))).expect("cast");
    assert_eq!(g.players[0].life, 21);
}

/// Memory Trap holds an opponent's permanent until it leaves.
#[test]
fn memory_trap_exiles_until_it_leaves() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let mt = g.add_card_to_hand(0, catalog::memory_trap());
    flood(&mut g, 0);
    cast(&mut g, mt, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear));
    run(&mut g, Effect::Destroy { what: crabomination::effect::Selector::ExactObjects(vec![mt]) });
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1), "back");
}

/// Shatterwing Pegasus pumps the team.
#[test]
fn shatterwing_pegasus_pumps_your_creatures() {
    let mut g = pod(2);
    let pg = ready(&mut g, 0, catalog::shatterwing_pegasus());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    flood(&mut g, 0);
    activate(&mut g, pg, 0, None).expect("{4}{W}");
    assert_eq!((power(&g, pg), power(&g, bear), power(&g, theirs)), (3, 3, 2));
}

/// CR 122.1b — Cryotheory Adept stuns a creature from the graveyard, as a
/// sorcery, exiling itself.
#[test]
fn cryotheory_adept_stuns_from_the_graveyard() {
    let mut g = pod(2);
    let ca = g.add_card_to_graveyard(0, catalog::cryotheory_adept());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    flood(&mut g, 0);
    activate(&mut g, ca, 0, Some(Target::Permanent(bear))).expect("activate");
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped);
    assert_eq!(b.counter_count(CounterType::Stun), 1);
    assert!(g.exile.iter().any(|c| c.id == ca));
}

/// Theoretical Necromancer exiles itself to return another creature card.
#[test]
fn theoretical_necromancer_returns_another_creature_card() {
    let mut g = pod(2);
    let tn = g.add_card_to_graveyard(0, catalog::theoretical_necromancer());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    assert!(activate(&mut g, tn, 0, Some(Target::Permanent(tn))).is_err(), "another");
    activate(&mut g, tn, 0, Some(Target::Permanent(bear))).expect("activate");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(g.exile.iter().any(|c| c.id == tn));
}

/// Theorist's Proxy's sacrifice leaves the next spell uncounterable.
#[test]
fn theorists_proxy_protects_the_next_spell() {
    let mut g = pod(2);
    let tp = ready(&mut g, 0, catalog::theorists_proxy());
    flood(&mut g, 0);
    activate(&mut g, tp, 0, None).expect("{U}, sacrifice");
    assert!(g.battlefield_find(tp).is_none());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let cs = g.add_card_to_hand(1, catalog::counterspell());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    let _ = g.perform_action(GameAction::CastSpell { card_id: cs, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None });
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "the bolt resolved");
}

/// Extrapolate the Impossible — two differently named sideboard cards are
/// revealed; the opponent's pick goes to hand, the other stays outside.
#[test]
fn extrapolate_the_impossible_hands_over_the_opponents_pick() {
    let mut g = pod(2);
    let bear = g.add_card_to_sideboard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_sideboard(0, catalog::lightning_bolt());
    let ei = g.add_card_to_hand(0, catalog::extrapolate_the_impossible());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear, bolt]), DecisionAnswer::Cards(vec![bear])]));
    cast(&mut g, ei, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the opponent's pick");
    assert!(g.players[0].sideboard.iter().any(|c| c.id == bolt), "the other stays outside the game");

    // One name outside the game: the reveal can't be made.
    let mut g = pod(2);
    let a = g.add_card_to_sideboard(0, catalog::lightning_bolt());
    let b = g.add_card_to_sideboard(0, catalog::lightning_bolt());
    let ei = g.add_card_to_hand(0, catalog::extrapolate_the_impossible());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b]), DecisionAnswer::Cards(vec![a])]));
    cast(&mut g, ei, None).expect("cast");
    assert_eq!(g.players[0].sideboard.len(), 2);
}

/// Solve for Disappointment takes a nonland PERMANENT card, then empowers.
#[test]
fn solve_for_disappointment_takes_a_permanent_card() {
    let mut g = pod(2);
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let sd = g.add_card_to_hand(0, catalog::solve_for_disappointment());
    flood(&mut g, 0);
    cast(&mut g, sd, Some(Target::Player(1))).expect("cast");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == bolt), "not a permanent card");
    assert_eq!(jace_loyalty(&g, 0), Some(1));
}

/// Stinging Vitriol burns the opponent and strips a nonland card.
#[test]
fn stinging_vitriol_burns_and_strips() {
    let mut g = pod(2);
    let land = g.add_card_to_hand(1, catalog::mountain());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let sv = g.add_card_to_hand(0, catalog::stinging_vitriol());
    flood(&mut g, 0);
    cast(&mut g, sv, Some(Target::Player(1))).expect("cast");
    assert_eq!(g.players[1].life, 18);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
    assert!(g.players[1].hand.iter().any(|c| c.id == land));
}

/// Awaken the Inferno: 6 to an opponent's creature, a counter on yours.
#[test]
fn awaken_the_inferno_burns_and_grows() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ai = g.add_card_to_hand(0, catalog::awaken_the_inferno());
    flood(&mut g, 0);
    cast_with(&mut g, ai, Some(Target::Permanent(giant)), vec![Target::Permanent(bear)], None).expect("cast");
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Fulminous Forte's sweep spares your side.
#[test]
fn fulminous_forte_pings_only_opponents() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::llanowar_elves());
    let theirs = ready(&mut g, 1, catalog::llanowar_elves());
    let ff = g.add_card_to_hand(0, catalog::fulminous_forte());
    flood(&mut g, 0);
    cast_with(&mut g, ff, None, vec![], Some(0)).expect("mode one");
    assert!(g.battlefield_find(mine).is_some());
    assert!(g.battlefield_find(theirs).is_none());
}

/// Heartstring Puller's Cadet.
#[test]
fn heartstring_puller_brings_a_cadet() {
    let mut g = pod(2);
    let hp = g.add_card_to_hand(0, catalog::heartstring_puller());
    flood(&mut g, 0);
    cast(&mut g, hp, None).expect("cast");
    let cadet = token(&g, 0, "Cadet").expect("a Cadet");
    assert_eq!(power(&g, cadet), 2);
}

/// Skilled Battlecarver has first strike only on your turn.
#[test]
fn skilled_battlecarver_first_strikes_on_your_turn() {
    let mut g = pod(2);
    let sb = ready(&mut g, 0, catalog::skilled_battlecarver());
    assert!(g.permanent_has_keyword(sb, &Keyword::FirstStrike));
    flood(&mut g, 0);
    activate(&mut g, sb, 0, None).expect("{1}{R}");
    assert_eq!(power(&g, sb), 3);
    g.active_player_idx = 1;
    assert!(!g.permanent_has_keyword(sb, &Keyword::FirstStrike));
}

/// CR 603.12 — Tether Technician's discard pays for a reflexive 2 damage.
#[test]
fn cr_603_12_tether_technician_discards_for_damage() {
    let mut g = pod(2);
    let fodder = g.add_card_to_hand(0, catalog::island());
    let tt = g.add_card_to_hand(0, catalog::tether_technician());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, tt, None).expect("cast");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder));
    assert_eq!(g.players[1].life, 18);
}

/// CR 120.10 — Violent Echoes empowers Jace by the excess over lethal.
#[test]
fn cr_120_10_violent_echoes_empowers_by_the_excess() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ve = g.add_card_to_hand(0, catalog::violent_echoes());
    flood(&mut g, 0);
    cast(&mut g, ve, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(jace_loyalty(&g, 0), Some(4), "6 damage, 2 lethal");
}

/// Wrath of the Bloodmane costs {1}{R} beside a legendary creature.
#[test]
fn wrath_of_the_bloodmane_is_cheaper_with_a_legend() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::isamaru_hound_of_konda());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let wb = g.add_card_to_hand(0, catalog::wrath_of_the_bloodmane());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, wb, Some(Target::Permanent(giant))).expect("two mana");
    assert!(g.battlefield_find(giant).is_none());
}

/// Bestial Incursion's Beast.
#[test]
fn bestial_incursion_makes_a_trampling_beast() {
    let mut g = pod(2);
    let bi = g.add_card_to_hand(0, catalog::bestial_incursion());
    flood(&mut g, 0);
    cast(&mut g, bi, None).expect("cast");
    let beast = token(&g, 0, "Beast").expect("a Beast");
    assert_eq!(power(&g, beast), 4);
    assert!(g.permanent_has_keyword(beast, &Keyword::Trample));
}

/// Flourishing Grapple strips the angel's abilities and bites it.
#[test]
fn flourishing_grapple_strips_and_bites() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let fg = g.add_card_to_hand(0, catalog::flourishing_grapple());
    flood(&mut g, 0);
    assert!(cast_with(&mut g, fg, Some(Target::Permanent(bear)), vec![Target::Permanent(angel)], None).is_err(), "not red or white");
    cast_with(&mut g, fg, Some(Target::Permanent(angel)), vec![Target::Permanent(bear)], None).expect("cast");
    assert!(!g.permanent_has_keyword(angel, &Keyword::Flying));
    assert_eq!(g.battlefield_find(angel).unwrap().damage, 2);
}

/// Inspired Tethermage grows as you put loyalty counters on a planeswalker:
/// an empower (its own included), a planeswalker entering, a + loyalty cost.
#[test]
fn inspired_tethermage_grows_on_loyalty_counters() {
    let mut g = pod(2);
    let it = ready(&mut g, 0, catalog::inspired_tethermage());
    empower(&mut g, 2);
    assert_eq!(g.battlefield_find(it).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    flood(&mut g, 0);
    activate(&mut g, it, 0, None).expect("{6}");
    assert_eq!(jace_loyalty(&g, 0), Some(4));
    assert_eq!(g.battlefield_find(it).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    // CR 122.6 — a planeswalker enters with loyalty counters put on it; CR
    // 606.4 — a + loyalty cost puts them on too.
    let gk = g.add_card_to_hand(0, catalog::garruk_veiled_butcher());
    flood(&mut g, 0);
    cast(&mut g, gk, None).expect("Garruk");
    assert_eq!(g.battlefield_find(it).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    loyalty(&mut g, gk, 0, None).expect("+2");
    assert_eq!(g.battlefield_find(it).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Sureshot Sower is discarded from hand to shoot down a flier.
#[test]
fn sureshot_sower_shoots_a_flier_from_hand() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ss = g.add_card_to_hand(0, catalog::sureshot_sower());
    flood(&mut g, 0);
    assert!(activate(&mut g, ss, 0, Some(Target::Permanent(bear))).is_err(), "no flying");
    activate(&mut g, ss, 0, Some(Target::Permanent(angel))).expect("activate");
    assert!(g.battlefield_find(angel).is_none());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ss));
}

/// Wrecking Gecko's {6}{G}{G}.
#[test]
fn wrecking_gecko_swells() {
    let mut g = pod(2);
    let wg = ready(&mut g, 0, catalog::wrecking_gecko());
    flood(&mut g, 0);
    activate(&mut g, wg, 0, None).expect("activate");
    assert_eq!(power(&g, wg), 9);
    assert!(g.permanent_has_keyword(wg, &Keyword::Trample));
}

/// Charge the Sanctum's single-target mode: +2/+0, first strike, a counter.
#[test]
fn charge_the_sanctum_arms_one_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let cs = g.add_card_to_hand(0, catalog::charge_the_sanctum());
    flood(&mut g, 0);
    cast_with(&mut g, cs, Some(Target::Permanent(bear)), vec![], Some(1)).expect("mode two");
    assert_eq!(power(&g, bear), 5);
    assert!(g.permanent_has_keyword(bear, &Keyword::FirstStrike));
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Stingerquill Charm's third mode: a hasty Cadet.
#[test]
fn stingerquill_charm_makes_a_hasty_cadet() {
    let mut g = pod(2);
    let sc = g.add_card_to_hand(0, catalog::stingerquill_charm());
    flood(&mut g, 0);
    cast_with(&mut g, sc, None, vec![], Some(2)).expect("mode three");
    let cadet = token(&g, 0, "Cadet").expect("a Cadet");
    assert!(g.permanent_has_keyword(cadet, &Keyword::Haste));
}

/// Clash of Elements — the owner picks: on top for 2 damage, or the bottom.
#[test]
fn clash_of_elements_owner_chooses_top_or_bottom() {
    for (pick, top, life) in [(1u32, true, 18), (0, false, 20)] {
        let mut g = pod(2);
        let bear = ready(&mut g, 1, catalog::grizzly_bears());
        let ce = g.add_card_to_hand(0, catalog::clash_of_elements());
        flood(&mut g, 0);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(pick)]));
        cast(&mut g, ce, Some(Target::Permanent(bear))).expect("cast");
        let lib = &g.players[1].library;
        let at = if top { lib.first() } else { lib.last() };
        assert_eq!(at.map(|c| c.definition.name), Some("Grizzly Bears"), "pick {pick}");
        assert_eq!(g.players[1].life, life, "pick {pick}");
    }
}

/// Desperate Futurescribe: +1/+1 at combat, a counter instead after a
/// surveil.
#[test]
fn desperate_futurescribe_rewards_a_surveil() {
    for surveil in [false, true] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::desperate_futurescribe());
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        if surveil {
            run(&mut g, Effect::Surveil { who: PlayerRef::You, amount: Value::ONE });
        }
        to_step(&mut g, TurnStep::BeginCombat);
        let counters = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
        assert_eq!(power(&g, bear), 3, "surveil {surveil}");
        assert_eq!(counters, u32::from(surveil), "surveil {surveil}");
    }
}

/// Grim Repriser comes back only after an opponent took noncombat damage.
#[test]
fn grim_repriser_returns_after_noncombat_damage() {
    let mut g = pod(2);
    let gr = g.add_card_to_graveyard(0, catalog::grim_repriser());
    flood(&mut g, 0);
    assert!(activate(&mut g, gr, 0, None).is_err(), "no damage yet");
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Player(1))).expect("shock");
    activate(&mut g, gr, 0, None).expect("{B}{R}");
    assert_eq!(g.battlefield_find(gr).expect("back").counter_count(CounterType::Finality), 1);
}

/// Mind Meanderer fights on the way in, and has vigilance beside a Jace.
#[test]
fn mind_meanderer_fights_and_wants_a_jace() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let mm = g.add_card_to_hand(0, catalog::mind_meanderer());
    flood(&mut g, 0);
    cast(&mut g, mm, None).expect("cast");
    assert!(g.battlefield_find(bear).is_none(), "fought");
    assert!(!g.permanent_has_keyword(mm, &Keyword::Vigilance));
    empower(&mut g, 1);
    assert!(g.permanent_has_keyword(mm, &Keyword::Vigilance));
}

/// CR 702.36 — Null Summoner's exiled card is castable, with any mana, only
/// while you have threshold.
#[test]
fn cr_702_36_null_summoner_casts_the_exiled_card_with_threshold() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::mountain());
    let ns = g.add_card_to_hand(0, catalog::null_summoner());
    flood(&mut g, 0);
    cast(&mut g, ns, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bolt), "exiled from their hand");
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Blue, 1);
    let cast_bolt = |g: &mut GameState| {
        g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
    };
    assert!(cast_bolt(&mut g).is_err(), "no threshold");
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    cast_bolt(&mut g).expect("threshold, blue mana");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Vindictive Triumph steals a cheap permanent until the end step and only
/// exiles an expensive one.
#[test]
fn vindictive_triumph_borrows_the_cheap_ones() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    for target in [bear, angel] {
        let vt = g.add_card_to_hand(0, catalog::vindictive_triumph());
        flood(&mut g, 0);
        cast(&mut g, vt, Some(Target::Permanent(target))).expect("cast");
    }
    assert!(g.exile.iter().any(|c| c.id == angel), "mana value 5 stays exiled");
    let borrowed = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("returned");
    assert_eq!((borrowed.controller, borrowed.tapped), (0, true));
    to_step(&mut g, TurnStep::End);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "exiled at the end step");
}

/// Kiora of Salt and Sand: the granted −8 is a loyalty activation, so the
/// attack after it untaps an attacker and makes it unblockable.
#[test]
fn kiora_of_salt_and_sand_rewards_a_loyalty_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kiora_of_salt_and_sand());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    empower(&mut g, 9);
    let j = jace(&g, 0).unwrap();
    loyalty(&mut g, j, 2, None).expect("the granted −8");
    let lev = token(&g, 0, "Leviathan").expect("a Leviathan");
    assert!(g.permanent_has_keyword(lev, &Keyword::Hexproof));
    to_step(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
    assert!(g.permanent_has_keyword(bear, &Keyword::Unblockable));
}

/// Formidable Commons enters tapped unless you control a planeswalker.
#[test]
fn formidable_commons_wants_a_planeswalker() {
    for walker in [false, true] {
        let mut g = pod(2);
        if walker {
            empower(&mut g, 1);
        }
        let land = g.add_card_to_hand(0, catalog::formidable_commons());
        g.perform_action(GameAction::PlayLand(land)).expect("play");
        assert_eq!(g.battlefield_find(land).unwrap().tapped, !walker, "walker {walker}");
    }
}

/// Garruk, Veiled Butcher: opponents' creatures are exiled instead of dying;
/// +2 shrinks, −2 trades creatures for a Beast, −3 draws for a short discard.
#[test]
fn garruk_veiled_butcher_abilities() {
    let setup = || {
        let mut g = pod(2);
        let gk = ready(&mut g, 0, catalog::garruk_veiled_butcher());
        let bear = ready(&mut g, 1, catalog::grizzly_bears());
        (g, gk, bear)
    };
    let (mut g, gk, bear) = setup();
    loyalty(&mut g, gk, 0, Some(Target::Permanent(bear))).expect("+2");
    assert_eq!(power(&g, bear), -2);
    assert_eq!(g.battlefield_find(gk).unwrap().counter_count(CounterType::Loyalty), 7);

    let (mut g, gk, bear) = setup();
    ready(&mut g, 0, catalog::grizzly_bears());
    loyalty(&mut g, gk, 1, None).expect("−2");
    assert!(g.exile.iter().any(|c| c.id == bear), "theirs is exiled instead");
    assert!(token(&g, 0, "Beast").is_some(), "you sacrificed one");

    let (mut g, gk, _) = setup();
    g.add_card_to_hand(1, catalog::mountain());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let hand = g.players[0].hand.len();
    loyalty(&mut g, gk, 2, None).expect("−3");
    assert!(g.players[1].hand.is_empty());
    assert_eq!(g.players[0].hand.len(), hand + 1, "only one nonland was discarded");
}

/// The Way enchantments' granted abilities make Beasts: the Deathbringer's
/// for a sacrifice, the Wildspeaker's outright.
#[test]
fn way_enchantments_grant_beast_abilities() {
    let mut g = pod(2);
    let fodder = ready(&mut g, 0, catalog::grizzly_bears());
    let wd = g.add_card_to_hand(0, catalog::way_of_the_deathbringer());
    flood(&mut g, 0);
    cast(&mut g, wd, None).expect("cast");
    let j = jace(&g, 0).unwrap();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    loyalty(&mut g, j, 2, None).expect("the granted −2");
    assert!(g.battlefield_find(fodder).is_none());
    assert!(token(&g, 0, "Beast").is_some());

    let mut g = pod(2);
    let ww = g.add_card_to_hand(0, catalog::way_of_the_wildspeaker());
    flood(&mut g, 0);
    cast(&mut g, ww, None).expect("cast");
    let j = jace(&g, 0).unwrap();
    loyalty(&mut g, j, 2, None).expect("the granted −4");
    assert!(token(&g, 0, "Beast").is_some());
    assert_eq!(jace_loyalty(&g, 0), Some(3));
}
