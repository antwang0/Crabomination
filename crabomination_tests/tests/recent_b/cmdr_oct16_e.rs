//! Commander: the 2026-10-16 third-leg rescan's gaps (`decks::cmdr_oct16_e`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::catalog;
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn has(g: &GameState, id: CardId, k: &Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(k)
}

/// The Howling Abomination: a spell targeting it pumps it and burns every
/// opponent (an ability targeting it does not); trample needs three spells.
#[test]
fn howling_abomination_punishes_every_opponent_when_targeted_by_a_spell() {
    let mut g = pod(4);
    let abom = ready(&mut g, 0, catalog::the_howling_abomination());
    assert!(!has(&g, abom, &Keyword::Trample));
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, growth, Some(Target::Permanent(abom))).expect("Giant Growth");
    assert_eq!(g.computed_permanent(abom).unwrap().power, 5 + 3 + 2);
    assert_eq!(g.players[0].life, life[0]);
    for seat in 1..4 {
        assert_eq!(g.players[seat].life, life[seat] - 2, "seat {seat}");
    }
    for _ in 0..2 {
        assert!(!has(&g, abom, &Keyword::Trample));
        let growth = g.add_card_to_hand(0, catalog::giant_growth());
        g.players[0].mana_pool.add(Color::Green, 1);
        cast(&mut g, growth, Some(Target::Permanent(abom))).expect("Giant Growth");
    }
    assert!(has(&g, abom, &Keyword::Trample), "three spells this turn");
    assert_eq!(g.players[1].life, life[1] - 6);
}

/// Storm's ETB takes an artifact, an enchantment or a flier — never a
/// ground creature — and may take nothing.
#[test]
fn storm_destroys_a_flier_but_not_a_ground_creature() {
    let mut g = pod(3);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let bears = ready(&mut g, 2, catalog::grizzly_bears());
    let storm = g.add_card_to_hand(0, catalog::storm_shaker_of_skies());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, storm, None).expect("cast Storm");
    assert!(g.battlefield_find(angel).is_none(), "the flier goes");
    assert!(g.battlefield_find(bears).is_some() && g.battlefield_find(storm).is_some());
}

/// Wolfsbane's pump is once each turn.
#[test]
fn wolfsbane_pumps_once_each_turn() {
    let mut g = pod(2);
    let wolf = ready(&mut g, 0, catalog::wolfsbane_highland_hero());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility { card_id: wolf, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
    };
    act(&mut g).expect("first pump");
    drain_stack(&mut g);
    assert!(act(&mut g).is_err(), "second pump this turn");
    assert_eq!(g.computed_permanent(wolf).unwrap().power, 4);
}

/// Extended Absence exiles a planeswalker and drains every opponent by one.
#[test]
fn extended_absence_exiles_and_pings_each_opponent() {
    let mut g = pod(3);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let ea = g.add_card_to_hand(0, catalog::extended_absence());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, ea, Some(Target::Permanent(bears))).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bears));
    assert_eq!(g.players.iter().map(|p| p.life).collect::<Vec<_>>(), vec![life[0] + 1, life[1] - 1, life[2] - 1]);
}

/// Hexhaven Dueling Arena's {2} ability wants a creature that attacked this
/// turn, and only a creature with a prepare spell can become prepared.
#[test]
fn hexhaven_dueling_arena_prepares_only_preparation_creatures() {
    let mut g = pod(2);
    let arena = ready(&mut g, 0, catalog::hexhaven_dueling_arena());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    let act = |g: &mut GameState, idx, t| {
        g.perform_action(GameAction::ActivateAbility { card_id: arena, ability_index: idx, target: Some(Target::Permanent(t)), additional_targets: vec![], x_value: None, mode: None })
    };
    assert!(act(&mut g, 2, bears).is_err(), "no prepare spell");
    assert!(act(&mut g, 1, bears).is_err(), "did not attack, and no prepare spell");
}

fn attack(g: &mut GameState, attackers: &[(CardId, usize)]) {
    g.step = TurnStep::DeclareAttackers;
    let attacks = attackers.iter().map(|&(a, p)| Attack { attacker: a, target: AttackTarget::Player(p) }).collect();
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

fn soldiers_attacking(g: &GameState, seat: usize) -> usize {
    g.attacking
        .iter()
        .filter(|a| a.target == AttackTarget::Player(seat))
        .filter(|a| g.battlefield_find(a.attacker).is_some_and(|c| c.is_token && c.definition.name == "Soldier"))
        .count()
}

/// CR 726 / CR 603.4 — Undercellar Sweep: the Soldiers come when *a player
/// you're attacking* has the initiative, attacking that player; an
/// initiative held by a player you're not attacking makes none.
#[test]
fn undercellar_sweep_reads_the_initiative_of_the_player_attacked() {
    for (attacked, expect) in [(1, 2), (2, 0)] {
        let mut g = pod(3);
        ready(&mut g, 0, catalog::undercellar_sweep());
        let bears = ready(&mut g, 0, catalog::grizzly_bears());
        g.initiative = Some(1);
        attack(&mut g, &[(bears, attacked)]);
        assert_eq!(soldiers_attacking(&g, attacked), expect, "attacking seat {attacked}");
    }
    // You holding it is enough, whoever you attack.
    let mut g = pod(3);
    ready(&mut g, 0, catalog::undercellar_sweep());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.initiative = Some(0);
    attack(&mut g, &[(bears, 2)]);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Soldier").count(), 2);
}

/// CR 508.4 — attacking two players, the Soldiers' defender is one of them
/// (the fan-out "player you're attacking" resolved to a single seat, not a
/// dropped set — this debug-asserted in a 15-seat pod).
#[test]
fn undercellar_sweep_with_two_players_attacked() {
    let mut g = pod(4);
    ready(&mut g, 0, catalog::undercellar_sweep());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.initiative = Some(2);
    attack(&mut g, &[(a, 1), (b, 2)]);
    assert_eq!(soldiers_attacking(&g, 1) + soldiers_attacking(&g, 2), 2);
}

/// Awaken the Ancient makes its Mountain a 7/7 red haste Giant that is still
/// a land.
#[test]
fn awaken_the_ancient_animates_a_mountain() {
    let mut g = pod(2);
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    let aura = g.add_card_to_hand(0, catalog::awaken_the_ancient());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(mountain))).expect("cast");
    let cp = g.computed_permanent(mountain).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
    assert!(cp.keywords().contains(&Keyword::Haste));
    assert!(cp.card_types().contains(&CardType::Creature) && cp.card_types().contains(&CardType::Land));
}

/// Spider-Man No More: a 1/1 defender with nothing else — the granted
/// defender survives the ability wipe.
#[test]
fn spider_man_no_more_leaves_a_one_one_defender() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::spider_man_no_more());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(angel))).expect("cast");
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.keywords().contains(&Keyword::Defender));
    assert!(!cp.keywords().contains(&Keyword::Flying));
}

/// Aspect of Wolf splits the Forest count: half down to power, half up to
/// toughness.
#[test]
fn aspect_of_wolf_splits_forests_down_and_up() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::aspect_of_wolf());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(bears))).expect("cast");
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (2 + 1, 2 + 2));
}

/// Undercover Skrull: +2/+2 and every creature type once two creature cards
/// are in your graveyard.
#[test]
fn undercover_skrull_wakes_with_two_creatures_in_the_graveyard() {
    let mut g = pod(2);
    let skrull = ready(&mut g, 0, catalog::undercover_skrull());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(skrull).unwrap().power, 1);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(skrull).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords().contains(&Keyword::Changeling));
}

/// Matt Murdock: the reflexive counter at your beginning of combat, then the
/// countered creature carries ward {1} (a creature with no counter doesn't).
#[test]
fn matt_murdock_counters_a_creature_and_wards_it() {
    let mut g = pod(2);
    let matt = ready(&mut g, 0, catalog::matt_murdock_justice_seeker());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let ward = Keyword::Ward(crabomination::card::WardCost::Mana(crabomination::mana::cost(&[crabomination::mana::generic(1)])));
    assert!(!has(&g, bears, &ward) && !has(&g, matt, &ward));
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let combat = catalog::matt_murdock_justice_seeker().triggered_abilities[0].effect.clone();
    g.resolve_effect(&combat, &ctx).expect("resolve");
    drain_stack(&mut g);
    let countered: Vec<CardId> =
        [matt, bears].into_iter().filter(|&c| g.battlefield_find(c).unwrap().counter_count(CounterType::PlusOnePlusOne) == 1).collect();
    assert_eq!(countered.len(), 1, "the reflexive counter landed once");
    assert!(has(&g, countered[0], &ward));
    assert_eq!(g.players[0].mana_pool.total(), 0, "the {{1}} was paid");
}

/// Murdock's Crusade: one mode without teamwork; teamwork 4 (tapping power 4)
/// buys both.
#[test]
fn murdocks_crusade_teamwork_buys_both_modes() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let crusade = g.add_card_to_hand(0, catalog::murdocks_crusade());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, crusade, Some(Target::Permanent(wurm))).expect("cast");
    assert!(g.battlefield_find(wurm).is_none(), "toughness 4+ exiled");
}

/// Wondrous Revival returns Heroes, never a non-Hero.
#[test]
fn wondrous_revival_returns_only_heroes() {
    let mut g = pod(2);
    let wasp = g.add_card_to_graveyard(0, catalog::the_wasp_janet_van_dyne());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let rev = g.add_card_to_hand(0, catalog::wondrous_revival());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, rev, Some(Target::Permanent(wasp))).expect("cast");
    assert!(g.battlefield_find(wasp).is_some());
    assert!(g.battlefield_find(bears).is_none());
}

/// Captain America, Wings of Freedom pumps each other Hero by his toughness.
#[test]
fn wings_of_freedom_pumps_other_heroes_by_his_toughness() {
    let mut g = pod(2);
    let cap = ready(&mut g, 0, catalog::captain_america_wings_of_freedom());
    let wasp = ready(&mut g, 0, catalog::the_wasp_janet_van_dyne());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, &[(cap, 1)]);
    assert_eq!(g.computed_permanent(wasp).unwrap().power, 2);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 2, "not a Hero");
    assert_eq!(g.computed_permanent(cap).unwrap().power, 3, "not himself");
}

/// Captain America, Liberator makes a Soldier per attached Equipment.
#[test]
fn captain_america_liberator_counts_his_equipment() {
    let mut g = pod(2);
    let cap = ready(&mut g, 0, catalog::captain_america_liberator());
    let shield = g.add_card_to_battlefield(0, catalog::iconic_shield());
    g.battlefield_find_mut(shield).unwrap().attached_to = Some(cap);
    let before = g.battlefield.iter().filter(|c| c.is_token).count();
    attack(&mut g, &[(cap, 1)]);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before + 1);
    assert_eq!(g.computed_permanent(cap).unwrap().toughness, 7, "+1/+2 from the Shield");
}

/// CR 601.2c — Puca's Mischief's second target must cost no more than the
/// first: against only a pricier permanent there is no swap.
#[test]
fn pucas_mischief_swaps_only_for_equal_or_lesser_mana_value() {
    for (theirs_is_cheap, swapped) in [(true, true), (false, false)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::pucas_mischief());
        ready(&mut g, 0, catalog::grizzly_bears());
        let theirs = ready(&mut g, 1, if theirs_is_cheap { catalog::llanowar_elves() } else { catalog::craw_wurm() });
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.active_player_idx = 0;
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(theirs).unwrap().controller == 0, swapped, "theirs cheap: {theirs_is_cheap}");
    }
}

/// Bogslither's Embrace: with a creature the blight half is paid (a -1/-1
/// counter, two mana); with none it costs {3} more.
#[test]
fn bogslithers_embrace_blights_or_pays_three() {
    let mut g = pod(2);
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::craw_wurm());
    let e = g.add_card_to_hand(0, catalog::bogslithers_embrace());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast(&mut g, e, Some(Target::Permanent(victim))).expect("blight half");
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);

    let mut g = pod(2);
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::bogslithers_embrace());
    g.players[0].mana_pool.add(Color::Black, 2);
    assert!(cast(&mut g, e, Some(Target::Permanent(victim))).is_err(), "no creature: {{3}} more");
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, e, Some(Target::Permanent(victim))).expect("pay half");
    assert!(g.battlefield_find(victim).is_none());
}

/// Lady Sun bounces herself and another creature, only before attackers on
/// your turn.
#[test]
fn lady_sun_bounces_herself_and_another() {
    let mut g = pod(2);
    let sun = ready(&mut g, 0, catalog::lady_sun());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility { card_id: sun, ability_index: 0, target: Some(Target::Permanent(bears)), additional_targets: vec![], x_value: None, mode: None })
    };
    g.step = TurnStep::DeclareBlockers;
    assert!(act(&mut g).is_err(), "after attackers");
    g.step = TurnStep::PreCombatMain;
    act(&mut g).expect("activate");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bears));
    assert!(g.players[0].hand.iter().any(|c| c.id == sun));
}

/// Heirloom Auntie sheds a -1/-1 counter as another creature of yours dies.
#[test]
fn heirloom_auntie_grows_as_your_creatures_die() {
    let mut g = pod(2);
    let auntie = g.add_card_to_hand(0, catalog::heirloom_auntie());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, auntie, None).expect("cast");
    assert_eq!(g.computed_permanent(auntie).unwrap().power, 2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: None, x_value: None }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(auntie).unwrap().power, 3);
}

/// Black Panther counters a lone attacker, not one of two.
#[test]
fn black_panther_counters_a_lone_attacker() {
    for (two, expect) in [(false, 1), (true, 0)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::black_panther_claws_of_bast());
        let a = ready(&mut g, 0, catalog::grizzly_bears());
        let b = ready(&mut g, 0, catalog::grizzly_bears());
        let attackers: Vec<(CardId, usize)> = if two { vec![(a, 1), (b, 1)] } else { vec![(a, 1)] };
        attack(&mut g, &attackers);
        assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), expect, "two attackers: {two}");
    }
}

/// Barren Glory wins only with an empty hand and no other permanent.
#[test]
fn barren_glory_wins_with_nothing_else() {
    for (extra, wins) in [(false, true), (true, false)] {
        let mut g = pod(3);
        ready(&mut g, 0, catalog::barren_glory());
        if extra {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        g.players[0].hand.clear();
        g.active_player_idx = 0;
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        // The win ends the game, so priority passes stop answering.
        while !g.stack.is_empty() && g.perform_action(GameAction::PassPriority).is_ok() {}
        assert_eq!(!g.players[1].is_alive() && !g.players[2].is_alive(), wins, "extra permanent: {extra}");
    }
}

/// Chronatog Totem's {0} needs it animated, and then costs you a turn.
#[test]
fn chronatog_totem_pumps_only_as_a_creature() {
    let mut g = pod(2);
    let totem = ready(&mut g, 0, catalog::chronatog_totem());
    let act = |g: &mut GameState, idx| {
        let r = g.perform_action(GameAction::ActivateAbility { card_id: totem, ability_index: idx, target: None, additional_targets: vec![], x_value: None, mode: None });
        drain_stack(g);
        r
    };
    assert!(act(&mut g, 2).is_err(), "not a creature yet");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    act(&mut g, 1).expect("animate");
    act(&mut g, 2).expect("pump");
    let cp = g.computed_permanent(totem).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 5));
}
