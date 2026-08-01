//! Mercadian Masques (MMQ) gap closure, second wave.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast_alt(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alt cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn stock(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::forest());
    }
}

/// Fire `seat`'s upkeep triggers in place.
fn upkeep(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

// ── Alternative costs ───────────────────────────────────────────────────────

/// Thunderclap can be cast by sacrificing a Mountain instead of paying {2}{R}.
#[test]
fn thunderclap_can_be_paid_with_a_mountain() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let clap = g.add_card_to_hand(0, catalog::thunderclap());
    cast_alt(&mut g, clap, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(mountain).is_none(), "the Mountain was the cost");
    assert!(g.battlefield_find(victim).is_none(), "3 damage killed the 2/2");
    assert_eq!(g.players[0].mana_pool.total(), 0, "no mana spent");
}

/// Pulverize needs *two* Mountains for its alternative cost.
#[test]
fn pulverize_needs_two_mountains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mountain());
    let pulv = g.add_card_to_hand(0, catalog::pulverize());
    assert!(
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: pulv,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
            pitch_card: None,
        })
        .is_err(),
        "one Mountain isn't enough"
    );
    g.add_card_to_battlefield(0, catalog::mountain());
    let lens = g.add_card_to_battlefield(1, catalog::distorting_lens());
    cast_alt(&mut g, pulv, None);
    assert!(g.battlefield_find(lens).is_none());
}

/// Thwart's alternative cost bounces three Islands.
#[test]
fn thwart_returns_three_islands() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bears");
    let thwart = g.add_card_to_hand(0, catalog::thwart());
    g.priority.player_with_priority = 0;
    cast_alt(&mut g, thwart, Some(Target::Permanent(bears)));
    assert!(g.battlefield_find(bears).is_none(), "countered");
    assert_eq!(g.players[0].hand.len(), 3, "three Islands came back");
}

/// Cave-In's pitch cost exiles a red card from hand.
#[test]
fn cave_in_pitches_a_red_card() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cave = g.add_card_to_hand(0, catalog::cave_in());
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: cave,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: Some(fodder),
    })
    .expect("alt cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!(g.players[0].life, life - 2, "it hits its caster too");
    assert!(g.players[0].hand.is_empty());
    assert!(g.exile.iter().any(|c| c.id == fodder), "the Bolt paid the cost");
}

/// Land Grant's free cast is gated on holding no land cards.
#[test]
fn land_grant_is_free_only_with_no_lands_in_hand() {
    let mut g = two_player_game();
    let forest = g.add_card_to_hand(0, catalog::forest());
    let grant = g.add_card_to_hand(0, catalog::land_grant());
    assert!(
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: grant,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
            pitch_card: None,
        })
        .is_err(),
        "a land in hand shuts the alt cost off"
    );
    g.players[0].hand.retain(|c| c.id != forest);
    cast_alt(&mut g, grant, None);
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// Rouse's 2-life alternative needs a Swamp on the battlefield.
#[test]
fn rouse_pitches_life_only_with_a_swamp() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rouse = g.add_card_to_hand(0, catalog::rouse());
    assert!(
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: rouse,
            target: Some(Target::Permanent(bears)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
            pitch_card: None,
        })
        .is_err(),
        "no Swamp, no discount"
    );
    g.add_card_to_battlefield(0, catalog::swamp());
    let life = g.players[0].life;
    cast_alt(&mut g, rouse, Some(Target::Permanent(bears)));
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4);
}

// ── "Any player may activate" ───────────────────────────────────────────────

/// A Monger's ability is open to the table: an opponent can fire Warmonger.
#[test]
fn warmonger_can_be_activated_by_an_opponent() {
    let mut g = two_player_game();
    let monger = g.add_card_to_battlefield(0, catalog::warmonger());
    let flier = g.add_card_to_battlefield(0, catalog::wind_drake()); // flying, spared
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, survives
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    let life = g.players[0].life;
    activate(&mut g, monger, 0, None);
    assert!(g.battlefield_find(flier).is_some(), "fliers are spared");
    assert!(g.battlefield_find(ground).is_some(), "a 2/1 survives one damage");
    assert_eq!(g.players[0].life, life - 1, "every player takes one");
    let _ = monger;
}

/// Flailing Soldier's shrink half is likewise open to the opponent.
#[test]
fn flailing_soldier_can_be_shrunk_by_the_opponent() {
    let mut g = two_player_game();
    let soldier = g.add_card_to_battlefield(0, catalog::flailing_soldier());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    activate(&mut g, soldier, 1, None);
    assert_eq!(g.computed_permanent(soldier).unwrap().power, 1);
}

// ── Rishadan pirates ────────────────────────────────────────────────────────

/// Rishadan Cutpurse takes a permanent from an opponent who can't pay {1}.
#[test]
fn rishadan_cutpurse_taxes_each_opponent() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let purse = g.add_card_to_hand(0, catalog::rishadan_cutpurse());
    mana(&mut g, 0);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(false),
    ]));
    cast(&mut g, purse, None);
    assert!(g.battlefield_find(victim).is_none(), "declined the toll");
}

// ── Statics ─────────────────────────────────────────────────────────────────

/// Uphill Battle taps opponents' creatures on the way in, not yours.
#[test]
fn uphill_battle_taps_only_opponents_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::uphill_battle());
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    let yours = g.add_card_to_hand(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    cast(&mut g, yours, None);
    assert!(!g.battlefield_find(yours).unwrap().tapped);
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, theirs, None);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Vernal Equinox hands flash to both players' creature spells.
#[test]
fn vernal_equinox_gives_everyone_flash() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vernal_equinox());
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    mana(&mut g, 1);
    // Seat 0's turn, so seat 1 has no sorcery-speed window of its own.
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    cast(&mut g, theirs, None);
    assert!(g.battlefield_find(theirs).is_some(), "cast at instant speed off-turn");
}

/// Fountain Watch shrouds your artifacts and enchantments, not your creatures.
#[test]
fn fountain_watch_shrouds_your_noncreature_permanents() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fountain_watch());
    let lens = g.add_card_to_battlefield(0, catalog::distorting_lens());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(lens).unwrap().keywords.contains(&Keyword::Shroud));
    assert!(!g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Embargo keeps nonland permanents down through the untap step.
#[test]
fn embargo_locks_nonland_permanents_down() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::embargo());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    for id in [bears, forest] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(bears).unwrap().tapped, "creature stays down");
    assert!(!g.battlefield_find(forest).unwrap().tapped, "lands untap normally");
}

/// Embargo's upkeep clause bleeds its controller for 2.
#[test]
fn embargo_bleeds_its_controller() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::embargo());
    let life = g.players[0].life;
    upkeep(&mut g, 0);
    assert_eq!(g.players[0].life, life - 2);
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Kyren Toy banks charge counters and cashes X of them in for X+1 colorless.
#[test]
fn kyren_toy_pays_out_x_plus_one() {
    let mut g = two_player_game();
    let toy = g.add_card_to_battlefield(0, catalog::kyren_toy());
    for _ in 0..2 {
        g.battlefield_find_mut(toy).unwrap().tapped = false;
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, toy, 0, None);
    }
    assert_eq!(g.battlefield_find(toy).unwrap().counter_count(CounterType::Charge), 2);
    g.battlefield_find_mut(toy).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: toy,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cash in");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3, "X + 1");
}

/// Magistrate's Scepter buys an extra turn for three charge counters.
#[test]
fn magistrates_scepter_buys_an_extra_turn() {
    let mut g = two_player_game();
    let scepter = g.add_card_to_battlefield(0, catalog::magistrates_scepter());
    g.battlefield_find_mut(scepter).unwrap().add_counters(CounterType::Charge, 2);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: scepter,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "two counters isn't enough"
    );
    g.battlefield_find_mut(scepter).unwrap().add_counters(CounterType::Charge, 1);
    activate(&mut g, scepter, 1, None);
    assert_eq!(g.battlefield_find(scepter).unwrap().counter_count(CounterType::Charge), 0);
    assert!(g.players[0].extra_turns > 0, "an extra turn is queued");
}

/// Mercadian Atlas draws only on a turn you skipped your land drop.
#[test]
fn mercadian_atlas_draws_when_you_missed_your_land_drop() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mercadian_atlas());
    stock(&mut g, 0, 5);
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    let before = g.players[0].hand.len();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1);

    g.players[0].lands_played_this_turn = 1;
    let before = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "no draw after a land drop");
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Trap Runner pins an unblocked attacker so it deals no player damage.
#[test]
fn trap_runner_blocks_an_unblocked_attacker() {
    let mut g = two_player_game();
    let runner = g.add_card_to_battlefield(0, catalog::trap_runner());
    g.clear_sickness(runner);
    let attacker = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    activate(&mut g, runner, 0, Some(Target::Permanent(attacker)));
    let life = g.players[0].life;
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "the attacker is blocked by nothing");
}

/// Groundskeeper buys a basic land back out of the graveyard.
#[test]
fn groundskeeper_recycles_a_basic() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::groundskeeper());
    let forest = g.add_card_to_hand(0, catalog::forest());
    let i = g.players[0].hand.iter().position(|c| c.id == forest).unwrap();
    let c = g.players[0].hand.remove(i);
    g.players[0].graveyard.push(c);
    mana(&mut g, 0);
    activate(&mut g, keeper, 0, Some(Target::Permanent(forest)));
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
    let _ = keeper;
}

/// Silverglade Elemental fetches a Forest onto the battlefield untapped.
#[test]
fn silverglade_elemental_ramps_a_forest() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let elem = g.add_card_to_hand(0, catalog::silverglade_elemental());
    mana(&mut g, 0);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    cast(&mut g, elem, None);
    assert!(g.battlefield_find(forest).is_some());
    assert!(!g.battlefield_find(forest).unwrap().tapped);
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Natural Affinity turns every land into a 2/2 that's still a land.
#[test]
fn natural_affinity_animates_all_lands() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::forest());
    let theirs = g.add_card_to_battlefield(1, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::natural_affinity());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    for id in [mine, theirs] {
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2));
        assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
    }
}

/// Tectonic Break makes every player sacrifice X lands.
#[test]
fn tectonic_break_sacrifices_x_lands_each() {
    let mut g = two_player_game();
    for seat in [0, 1] {
        for _ in 0..3 {
            g.add_card_to_battlefield(seat, catalog::forest());
        }
    }
    let brk = g.add_card_to_hand(0, catalog::tectonic_break());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: brk,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    for seat in [0, 1] {
        let lands = g.battlefield.iter().filter(|c| c.controller == seat).count();
        assert_eq!(lands, 1, "seat {seat} sacrificed two");
    }
}

/// Misstep keeps a player's creatures tapped through their next untap step.
#[test]
fn misstep_locks_creatures_down_for_a_turn() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::misstep());
    mana(&mut g, 0);
    cast(&mut g, spell, Some(Target::Player(1)));
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bears).unwrap().tapped);
}

/// Honor the Fallen exiles every creature card in every graveyard and pays a
/// life point for each.
#[test]
fn honor_the_fallen_clears_the_graveyards() {
    let mut g = two_player_game();
    for seat in [0, 1] {
        for def in [catalog::grizzly_bears(), catalog::lightning_bolt()] {
            let id = g.add_card_to_hand(seat, def);
            let i = g.players[seat].hand.iter().position(|c| c.id == id).unwrap();
            let c = g.players[seat].hand.remove(i);
            g.players[seat].graveyard.push(c);
        }
    }
    let spell = g.add_card_to_hand(0, catalog::honor_the_fallen());
    mana(&mut g, 0);
    let life = g.players[0].life;
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].life, life + 2, "two creature cards exiled");
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.definition.is_creature()),
        "no creature cards survive in either graveyard"
    );
    assert_eq!(g.players[1].graveyard.len(), 1, "only the Bolt is left");
}
