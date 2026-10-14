//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_k`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::mana::Color;
use crabomination::game::*;

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

fn has(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&k)
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
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
}

/// CR 701.20 — Gollum's guess: exactly one branch runs — removed from combat,
/// or a card drawn and unblockable.
#[test]
fn gollum_runs_exactly_one_branch() {
    let mut g = pod(2);
    let gollum = ready(&mut g, 0, catalog::gollum_scheming_guide());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    declare(&mut g, vec![(gollum, 1)]);
    let attacking = g.attacking.iter().any(|a| a.attacker == gollum);
    let drew = g.players[0].hand.len() == hand + 1;
    assert!(attacking == drew, "removed and drew nothing, or kept attacking and drew");
    assert_eq!(has(&g, gollum, Keyword::Unblockable), drew);
}

/// An empty library is a right guess: Gollum leaves combat.
#[test]
fn gollum_with_an_empty_library_leaves_combat() {
    let mut g = pod(2);
    let gollum = ready(&mut g, 0, catalog::gollum_scheming_guide());
    g.players[0].library.clear();
    declare(&mut g, vec![(gollum, 1)]);
    assert!(!g.attacking.iter().any(|a| a.attacker == gollum));
}

/// Poison-Blade Mentor gives another attacking Assassin deathtouch.
#[test]
fn poison_blade_mentor_arms_an_assassin() {
    let mut g = pod(2);
    let mentor = ready(&mut g, 0, catalog::poison_blade_mentor());
    let other = ready(&mut g, 0, catalog::poison_blade_mentor());
    std::sync::Arc::make_mut(g.battlefield_find_mut(other).unwrap().definition_mut()).keywords.clear();
    declare(&mut g, vec![(mentor, 1)]);
    assert!(has(&g, other, Keyword::Deathtouch));
}

fn tapped(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).unwrap().tapped
}

/// Mudslide: a grounded creature skips its untap step, a flyer doesn't, and
/// the upkeep {2} buys the grounded one back.
#[test]
fn mudslide_holds_ground_creatures_until_paid_for() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mudslide());
    for seat in 0..2 {
        g.add_card_to_library(seat, catalog::island());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bird = g.add_card_to_battlefield(1, catalog::wind_drake());
    let lands: Vec<CardId> =
        (0..2).map(|_| g.add_card_to_battlefield(1, catalog::forest())).collect();
    for id in [bear, bird].into_iter().chain(lands.iter().copied()) {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    assert!(g.untap_prevented_by_static(bear));
    assert!(!g.untap_prevented_by_static(bird));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.active_player_idx == 0 || g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(!tapped(&g, bird), "the flyer untaps normally");
    assert!(!tapped(&g, bear), "the {{2}} at upkeep untapped the bear");
    assert!(lands.iter().all(|&l| tapped(&g, l)), "both lands paid for it");
}

/// With no mana to pay, the grounded creature stays tapped.
#[test]
fn mudslide_keeps_an_unpaid_creature_tapped() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mudslide());
    for seat in 0..2 {
        g.add_card_to_library(seat, catalog::island());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    while g.active_player_idx == 0 || g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(tapped(&g, bear));
}

/// String of Disappearances: the bounced creature's controller pays {U}{U}
/// and points the copy back at the caster's creature.
#[test]
fn string_of_disappearances_chains_back_at_the_caster() {
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::string_of_disappearances());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(mine)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "bounced home");
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "the copy bounced mine");
}

/// Asgardian Inspiration impulses one card; from the graveyard, a Shock to an
/// opponent buys it back for {2}, and a Shock to a creature doesn't.
#[test]
fn asgardian_inspiration_returns_on_noncombat_damage_to_an_opponent() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::asgardian_inspiration());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell));
    assert_eq!(g.exile.len(), 1, "the top card was exiled to play");

    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for (target, back) in [(Target::Permanent(bear), false), (Target::Player(1), true)] {
        let shock = g.add_card_to_hand(0, catalog::shock());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.perform_action(GameAction::CastSpell {
            card_id: shock,
            target: Some(target.clone()),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == spell), back, "{target:?}");
    }
}

/// Blessing of Frost cast off four snow lands: four counters on the lone
/// bear, which then has power 4+ and draws a card.
#[test]
fn blessing_of_frost_counts_snow_mana_spent() {
    for snow in [true, false] {
        let mut g = pod(2);
        g.add_card_to_library(0, catalog::island());
        for _ in 0..4 {
            let land = if snow { catalog::snow_covered_forest() } else { catalog::forest() };
            g.add_card_to_battlefield(0, land);
        }
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::blessing_of_frost());
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("castable");
        drain_stack(&mut g);
        let power = g.computed_permanent(bear).unwrap().power;
        assert_eq!(power, if snow { 6 } else { 2 }, "snow={snow}");
        assert_eq!(g.players[0].hand.len(), usize::from(snow), "snow={snow}");
    }
}

/// Swallow Whole taps the caster's bear as its cost, exiles their tapped
/// creature, and grows the bear that paid.
#[test]
fn swallow_whole_grows_the_creature_tapped_to_pay() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::swallow_whole());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "swallowed");
    assert!(tapped(&g, mine), "tapped as the cost");
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "and grew");
}

/// Without an untapped creature to tap, it can't be cast.
#[test]
fn swallow_whole_needs_a_creature_to_tap() {
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::swallow_whole());
    g.players[0].mana_pool.add(Color::White, 1);
    let r = g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(r.is_err());
}

/// Autumn Willow mints a Forest Dryad; tapping that land creature for mana
/// adds an extra {G}, a plain Forest doesn't.
#[test]
fn autumn_willow_doubles_land_creature_mana() {
    let mut g = pod(2);
    let willow = g.add_card_to_hand(0, catalog::autumn_willow_harmony());
    cast_flooded(&mut g, willow);
    let dryad = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.controller == 0)
        .map(|c| c.id)
        .expect("Forest Dryad");
    g.clear_sickness(dryad);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: dryad,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap the dryad");
    assert_eq!(g.players[0].mana_pool.total(), 2, "{{G}} plus the extra {{G}}");
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap the forest");
    assert_eq!(g.players[0].mana_pool.total(), 3, "a plain Forest adds one");
}

/// Donnie & April: both modes — the caster draws two and the opponent gets
/// a sorcery back.
#[test]
fn donnie_and_april_draws_and_regrows() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_graveyard(1, catalog::shock());
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Player(0)),
        DecisionAnswer::Target(Target::Player(1)),
    ]));
    let duo = g.add_card_to_hand(0, catalog::donnie_and_april_adorkable_duo());
    cast_flooded(&mut g, duo);
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[1].hand.len(), 1, "the Shock came back");
}

fn cast_flooded(g: &mut GameState, id: CardId) {
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(g);
    g.players[0].mana_pool = Default::default();
}

/// Weathered Bodyguards: an unblocked attacker's combat damage lands on the
/// untapped Bodyguards instead of their controller; tapped, it doesn't.
#[test]
fn weathered_bodyguards_absorb_unblocked_combat_damage() {
    for untapped in [true, false] {
        let mut g = pod(2);
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let guard = g.add_card_to_battlefield(1, catalog::weathered_bodyguards());
        g.battlefield_find_mut(guard).unwrap().tapped = !untapped;
        let life = g.players[1].life;
        declare(&mut g, vec![(bear, 1)]);
        while g.step != TurnStep::EndCombat {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        let damage = g.battlefield_find(guard).unwrap().damage;
        if untapped {
            assert_eq!((g.players[1].life, damage), (life, 2), "absorbed");
        } else {
            assert_eq!((g.players[1].life, damage), (life - 2, 0), "tapped: not absorbed");
        }
    }
}

/// Oriss's {T} shields a creature from a Shock for the turn.
#[test]
fn oriss_prevents_damage_to_target_creature() {
    let mut g = pod(2);
    let oriss = ready(&mut g, 0, catalog::oriss_samite_guardian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: oriss,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let shock = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: shock,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the shield held");
}
