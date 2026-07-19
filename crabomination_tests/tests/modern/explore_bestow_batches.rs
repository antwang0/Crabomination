#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Explore (CR 701.40) ──────────────────────────────────────────────────────

/// Merfolk Branchwalker explores; a nonland on top means a +1/+1 counter
/// (and the card stays in the library).
#[test]
fn merfolk_branchwalker_explores_nonland_grows() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears()); // nonland on top
    let id = g.add_card_to_hand(0, catalog::merfolk_branchwalker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Branchwalker castable for {1}{G}");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let bw = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((bw.power, bw.toughness), (3, 2), "nonland explore added a +1/+1 counter");
    assert_eq!(g.players[0].library.len(), 1, "revealed nonland stayed on top");
}

/// Merfolk Branchwalker explores; a land on top goes to hand (no counter).
#[test]
fn merfolk_branchwalker_explores_land_to_hand() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::forest()); // land on top
    let id = g.add_card_to_hand(0, catalog::merfolk_branchwalker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Branchwalker castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let bw = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((bw.power, bw.toughness), (2, 1), "land explore grants no counter");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
        "revealed land went to hand");
    assert!(g.players[0].library.is_empty(), "land left the library");
}

/// Jadelight Ranger explores twice; two nonlands on top → two counters.
#[test]
fn jadelight_ranger_explores_twice() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::jadelight_ranger());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Jadelight castable for {1}{G}{G}");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let jr = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((jr.power, jr.toughness), (4, 3), "two nonland explores → +2/+2");
}

/// Wildgrowth Walker's explore payoff: a creature you control exploring puts
/// a +1/+1 counter on it and gains you 3 life.
#[test]
fn wildgrowth_walker_grows_and_gains_life_on_explore() {
    let mut g = two_player_game();
    let ww = g.add_card_to_battlefield(0, catalog::wildgrowth_walker());
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears()); // nonland → explore counter
    let id = g.add_card_to_hand(0, catalog::merfolk_branchwalker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Branchwalker castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let w = view.iter().find(|c| c.id == ww).unwrap();
    assert_eq!((w.power, w.toughness), (1, 4), "Wildgrowth Walker grew from the explore");
    assert_eq!(g.players[0].life, life_before + 3, "gained 3 life on explore");
}

// ── Goad (CR 701.38) ─────────────────────────────────────────────────────────

/// Disrupt Decorum goads every creature its caster doesn't control.
#[test]
fn disrupt_decorum_goads_opponents_creatures() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dd = g.add_card_to_hand(0, catalog::disrupt_decorum());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: dd, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Disrupt Decorum castable for {3}{R}{R}");
    drain_stack(&mut g);
    let c = g.battlefield_find(opp).unwrap();
    assert!(c.goaded_by.contains(&0), "opponent creature is goaded by player 0");
}

/// A goaded creature must be declared as an attacker when able (CR 508.1d).
#[test]
fn goaded_creature_must_attack() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(c).unwrap().goaded_by = vec![0];
    g.clear_sickness(c);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;

    let err = g.declare_attackers(vec![]).unwrap_err();
    assert!(matches!(err, GameError::CannotAttack(id) if id == c),
        "goaded creature must attack, got {err:?}");
    g.declare_attackers(vec![Attack { attacker: c, target: AttackTarget::Player(0) }])
        .expect("declaration including the goaded creature is legal");
}

/// Goad expires when the goader's next turn begins (CR 701.38a).
#[test]
fn goad_expires_at_goaders_next_turn() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(c).unwrap().goaded_by = vec![0];
    // Player 0's untap step clears their goad.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(c).unwrap().goaded_by.is_empty(),
        "goad lifted once the goader's turn began");
}

// ── Monstrosity (CR 701.31) ──────────────────────────────────────────────────

/// Nessian Wilds Ravager's monstrosity grows it by five +1/+1 counters and
/// marks it monstrous.
#[test]
fn nessian_wilds_ravager_becomes_monstrous() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::nessian_wilds_ravager());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Monstrosity 5 activatable for {6}{G}{G}");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let r = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((r.power, r.toughness), (11, 11), "6/6 + five +1/+1 counters");
    assert!(g.battlefield_find(id).unwrap().monstrous, "flagged monstrous");
}

/// Monstrosity is once-only (CR 701.31) — a second activation does nothing.
#[test]
fn monstrosity_is_once_only() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::nessian_wilds_ravager());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.players[0].mana_pool.add_colorless(12);
    g.priority.player_with_priority = 0;
    for _ in 0..2 {
        let _ = g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        });
        drain_stack(&mut g);
    }
    let view = g.compute_battlefield();
    let r = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((r.power, r.toughness), (11, 11), "second activation adds no more counters");
}

/// Ember Swallower's become-monstrous trigger makes each player sacrifice
/// three lands.
#[test]
fn ember_swallower_monstrous_trigger_sacrifices_lands() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ember_swallower());
    for p in 0..2 {
        for _ in 0..3 {
            g.add_card_to_battlefield(p, catalog::mountain());
        }
    }
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Monstrosity 3 activatable for {3}{R}{R}");
    drain_stack(&mut g);
    for p in 0..2 {
        let lands = g.battlefield.iter()
            .filter(|c| c.controller == p && c.definition.is_land()).count();
        assert_eq!(lands, 0, "player {p} sacrificed all three lands");
    }
}

// ── More explore / dinosaurs (claude/modern_decks) ───────────────────────────

#[test]
fn seekers_squire_explores_on_etb() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::seekers_squire());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Seekers' Squire castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let s = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((s.power, s.toughness), (2, 3), "nonland explore grew it");
}

#[test]
fn emperors_vanguard_explores_on_attack() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::emperors_vanguard());
    g.clear_sickness(id);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }])
        .expect("attack declared");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let v = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((v.power, v.toughness), (5, 4), "attack-trigger explore grew it");
}

#[test]
fn path_of_discovery_explores_each_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::path_of_discovery());
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let bear = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((bear.power, bear.toughness), (3, 3), "entering creature explored (nonland → counter)");
}

#[test]
fn arbor_colossus_monstrous_destroys_a_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::arbor_colossus());
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Monstrosity 3 activatable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    assert_eq!(view.iter().find(|c| c.id == id).unwrap().power, 9, "6/6 + 3 = 9/9");
    assert!(!g.battlefield.iter().any(|c| c.id == flier), "flier destroyed on becoming monstrous");
}

#[test]
fn ripjaw_raptor_enrage_draws() {
    let mut g = two_player_game();
    let raptor = g.add_card_to_battlefield(0, catalog::ripjaw_raptor()); // 4/5
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(raptor)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt (-1 hand) + enrage draw (+1) = net 0; Ripjaw (4/5) survives 3 damage.
    assert_eq!(g.players[0].hand.len(), hand_before, "enrage drew a card to offset the Bolt");
    assert!(g.battlefield.iter().any(|c| c.id == raptor), "Ripjaw survives 3 damage");
}

#[test]
fn thrashing_brontodon_sacrifices_to_destroy_artifact() {
    let mut g = two_player_game();
    let bronto = g.add_card_to_battlefield(0, catalog::thrashing_brontodon());
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bronto, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(art)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac ability activatable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == art), "artifact destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == bronto), "Brontodon sacrificed");
}

#[test]
fn regisaur_alpha_makes_a_token_and_grants_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::regisaur_alpha());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Regisaur Alpha castable");
    drain_stack(&mut g);
    // ETB made a 3/3 Dinosaur token.
    let token = g.battlefield.iter()
        .find(|c| c.definition.name == "Dinosaur" && c.controller == 0)
        .expect("3/3 Dinosaur token created");
    let tok_id = token.id;
    // The token (another Dinosaur) has haste from Regisaur's static.
    let view = g.compute_battlefield();
    let tok = view.iter().find(|c| c.id == tok_id).unwrap();
    assert!(tok.keywords.contains(&crabomination::card::Keyword::Haste), "other Dinosaurs gain haste");
}

#[test]
fn farhaven_elf_fetches_a_basic_land_tapped() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::farhaven_elf());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Farhaven Elf castable");
    drain_stack(&mut g);
    let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest" && c.controller == 0);
    assert!(forest.is_some(), "fetched a Forest onto the battlefield");
    assert!(forest.unwrap().tapped, "land entered tapped");
}

#[test]
fn tishanas_wayfinder_explores_on_etb() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tishanas_wayfinder());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tishana's Wayfinder castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let w = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((w.power, w.toughness), (3, 3), "ETB explore grew it");
}

#[test]
fn ill_tempered_cyclops_monstrosity() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ill_tempered_cyclops());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Monstrosity 2 activatable for {3}{R}");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let c = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "3/3 + two +1/+1 counters");
}

#[test]
fn charging_monstrosaur_has_trample_and_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::charging_monstrosaur());
    let view = g.compute_battlefield();
    let c = view.iter().find(|c| c.id == id).unwrap();
    assert!(c.keywords.contains(&crabomination::card::Keyword::Trample));
    assert!(c.keywords.contains(&crabomination::card::Keyword::Haste));
    assert_eq!((c.power, c.toughness), (5, 5));
}

#[test]
fn grazing_whiptail_has_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::grazing_whiptail());
    let view = g.compute_battlefield();
    let c = view.iter().find(|c| c.id == id).unwrap();
    assert!(c.keywords.contains(&crabomination::card::Keyword::Reach));
}

#[test]
fn frilled_deathspitter_enrage_burns_opponent() {
    let mut g = two_player_game();
    let dino = g.add_card_to_battlefield(0, catalog::frilled_deathspitter()); // 2/2
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(dino)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Dino dies to 3, but enrage fires first (dealt damage) → 2 to each opp.
    assert_eq!(g.players[1].life, opp_life - 2, "enrage dealt 2 to the opponent");
}

#[test]
fn raptor_hatchling_enrage_makes_a_token() {
    let mut g = two_player_game();
    let dino = g.add_card_to_battlefield(0, catalog::raptor_hatchling()); // 1/1
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(dino)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Hatchling dies, but enrage made a 3/3 Dinosaur token.
    let tokens = g.battlefield.iter()
        .filter(|c| c.definition.name == "Dinosaur" && c.controller == 0).count();
    assert_eq!(tokens, 1, "enrage created a 3/3 Dinosaur token");
}

#[test]
fn pounce_makes_creatures_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::charging_monstrosaur()); // 5/5
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::pounce());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Permanent(mine)),
        additional_targets: vec![crabomination::game::types::Target::Permanent(theirs)],
        mode: None, x_value: None,
    }).expect("Pounce castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "2/2 took 5 and died");
    assert!(g.battlefield.iter().any(|c| c.id == mine), "5/5 survived 2 damage");
}

#[test]
fn atzocan_archer_etb_fight() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::atzocan_archer()); // 1/4
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    // Script the MayDo (yes) + the fight target.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(crabomination::game::types::Target::Permanent(theirs)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Atzocan Archer castable");
    drain_stack(&mut g);
    // 1/4 archer vs 2/2: the bear takes 1, the archer takes 2 — the fight
    // resolved (no kills, but damage was exchanged).
    assert_eq!(g.battlefield_find(id).unwrap().damage, 2, "archer took 2 from the fight");
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 1, "bear took 1 from the fight");
}

#[test]
fn ranging_raptors_enrage_ramps() {
    let mut g = two_player_game();
    let raptor = g.add_card_to_battlefield(0, catalog::ranging_raptors()); // 3/3
    g.players[0].library.clear();
    let forest = g.add_card_to_library(0, catalog::forest());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(raptor)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let lands = g.battlefield.iter()
        .filter(|c| c.definition.name == "Forest" && c.controller == 0).count();
    assert_eq!(lands, 1, "enrage fetched a basic land");
}

#[test]
fn otepec_huntmaster_discounts_dinosaurs() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::otepec_huntmaster());
    // Charging Monstrosaur ({3}{R}{R}) costs {2}{R}{R} with the discount.
    let dino = g.add_card_to_hand(0, catalog::charging_monstrosaur());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: dino, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dinosaur castable at the {1}-less rate");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dino), "discounted Dinosaur resolved");
}

#[test]
fn kinjallis_caller_discounts_dinosaurs() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kinjallis_caller());
    let dino = g.add_card_to_hand(0, catalog::charging_monstrosaur()); // {3}{R}{R}
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2); // {2}{R}{R} after the {1} discount
    g.perform_action(GameAction::CastSpell {
        card_id: dino, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("discounted Dinosaur castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dino));
}

#[test]
fn territorial_hammerskull_taps_on_attack() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let hammer = g.add_card_to_battlefield(0, catalog::territorial_hammerskull());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(hammer);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: hammer, target: AttackTarget::Player(1) }])
        .expect("attack declared");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).unwrap().tapped, "attack trigger tapped the blocker");
}

// ── Aristocrats / sac-fodder batch (claude/modern_decks) ─────────────────────

#[test]
fn zulaport_cutthroat_drains_when_your_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zulaport_cutthroat());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Bolt my own fodder so the full SBA+dispatch path fires Zulaport.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp = g.players[1].life;
    let me = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the fodder");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1 to Zulaport");
    assert_eq!(g.players[0].life, me + 1, "controller gained 1");
}

/// CR 603.10a — "this creature or another creature you control dies" fires for
/// Zulaport's own death (YourControl self-death funnel).
#[test]
fn zulaport_cutthroat_drains_on_its_own_death() {
    let mut g = two_player_game();
    let zula = g.add_card_to_battlefield(0, catalog::zulaport_cutthroat());
    let opp = g.players[1].life;
    let me = g.players[0].life;
    g.battlefield_find_mut(zula).unwrap().damage = 1; // lethal on the 1/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1 to Zulaport's own death");
    assert_eq!(g.players[0].life, me + 1, "controller gained 1");
}

#[test]
fn doomed_dissenter_leaves_a_zombie_when_it_dies() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let dd = g.add_card_to_battlefield(0, catalog::doomed_dissenter());
    g.battlefield_find_mut(dd).unwrap().toughness_bonus -= 1;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    let zombies = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&CreatureType::Zombie)).count();
    assert_eq!(zombies, 1, "Doomed Dissenter dies → one 2/2 Zombie");
}

#[test]
fn nantuko_husk_sacrifices_for_a_pump() {
    let mut g = two_player_game();
    let husk = g.add_card_to_battlefield(0, catalog::nantuko_husk());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    g.perform_action(GameAction::ActivateAbility {
        card_id: husk, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac ability activatable");
    drain_stack(&mut g);
    let c = g.battlefield_find(husk).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3), "Nantuko Husk pumps to 3/3");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 1,
        "the fodder creature was sacrificed");
}

// ── ETB-value + keyword bodies batch (claude/modern_decks) ───────────────────

#[test]
fn fleshbag_marauder_edicts_each_player() {
    let mut g = two_player_game();
    // Each player has one creature to lose.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fleshbag_marauder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {2}{B}");
    drain_stack(&mut g);
    // P0 keeps the Marauder (sacs the bear); P1 sacs its bear.
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 1,
        "P0 sacrificed the bear, kept the Marauder");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 0,
        "P1 sacrificed its only creature");
}

#[test]
fn kor_skyfisher_returns_a_permanent_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::kor_skyfisher());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {1}{W}");
    drain_stack(&mut g);
    // ETB bounced one of the controller's permanents back to hand.
    assert_eq!(g.players[0].hand.len(), 1, "exactly one permanent returned to hand");
}

#[test]
fn mogg_fanatic_sacrifices_to_ping() {
    let mut g = two_player_game();
    let fanatic = g.add_card_to_battlefield(0, catalog::mogg_fanatic());
    let opp = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fanatic, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac ability activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "Mogg Fanatic pinged for 1");
    assert!(g.battlefield_find(fanatic).is_none(), "Mogg Fanatic sacrificed itself");
}

#[test]
fn spectral_sailor_draws_with_its_ability() {
    let mut g = two_player_game();
    let sailor = g.add_card_to_battlefield(0, catalog::spectral_sailor());
    g.clear_sickness(sailor);
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: sailor, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("draw ability activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "Spectral Sailor drew a card");
}

#[test]
fn keyword_bodies_have_correct_stats() {
    use crabomination::card::Keyword;
    let sky = catalog::skyknight_legionnaire();
    assert!(sky.keywords.contains(&Keyword::Flying) && sky.keywords.contains(&Keyword::Haste));
    let hawk = catalog::healers_hawk();
    assert!(hawk.keywords.contains(&Keyword::Flying) && hawk.keywords.contains(&Keyword::Lifelink));
    let dryad = catalog::gnarlwood_dryad();
    assert!(dryad.keywords.contains(&Keyword::Deathtouch));
    let rats = catalog::typhoid_rats();
    assert!(rats.keywords.contains(&Keyword::Deathtouch));
    let elem = catalog::lightning_elemental();
    assert_eq!((elem.power, elem.toughness), (4, 1));
    assert!(elem.keywords.contains(&Keyword::Haste));
}

// ── Filigree Familiar / Sporemound / vanilla bodies ──────────────────────────

#[test]
fn filigree_familiar_gains_life_on_etb_and_draws_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::filigree_familiar());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {2}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "ETB gained 2 life");
    let hand = g.players[0].hand.len();
    // Bolt it to trigger the dies-draw.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the familiar");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "spent Bolt, drew 1 on death → net same");
    assert!(g.battlefield_find(id).is_none(), "Filigree Familiar died");
}

#[test]
fn sporemound_makes_a_saproling_on_landfall() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::sporemound());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land drop");
    drain_stack(&mut g);
    let saps = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Saproling").count();
    assert_eq!(saps, 1, "landfall minted one Saproling");
}

#[test]
fn green_keyword_bodies_have_correct_stats() {
    use crabomination::card::Keyword;
    let scout = catalog::gladecover_scout();
    assert!(scout.keywords.contains(&Keyword::Hexproof));
    let recluse = catalog::deadly_recluse();
    assert!(recluse.keywords.contains(&Keyword::Reach)
        && recluse.keywords.contains(&Keyword::Deathtouch));
    let courser = catalog::centaur_courser();
    assert_eq!((courser.power, courser.toughness), (3, 3));
}

// ── Borderland Ranger / Viashino Pyromancer ──────────────────────────────────

#[test]
fn borderland_ranger_fetches_a_basic_land_to_hand() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    // Script the ETB search to grab the Forest (AutoDecider would decline).
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::borderland_ranger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {2}{G}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest),
        "Borderland Ranger fetched the Forest to hand");
}

#[test]
fn viashino_pyromancer_burns_a_player_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::viashino_pyromancer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "Viashino Pyromancer dealt 2 to the opponent");
}

// ── Combat-trigger / dies-token / sac-ping batch (claude/modern_decks) ───────

#[test]
fn thieving_magpie_draws_on_combat_damage() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let magpie = g.add_card_to_battlefield(0, catalog::thieving_magpie());
    g.clear_sickness(magpie);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: magpie, target: AttackTarget::Player(1),
    }])).expect("magpie attacks");
    let hand = g.players[0].hand.len();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "Magpie drew on combat damage");
}

#[test]
fn abyssal_specter_forces_discard_on_combat_damage() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let spec = g.add_card_to_battlefield(0, catalog::abyssal_specter());
    g.clear_sickness(spec);
    g.add_card_to_hand(1, catalog::grizzly_bears()); // a card to discard
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: spec, target: AttackTarget::Player(1),
    }])).expect("specter attacks");
    let opp_hand = g.players[1].hand.len();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat resolves");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "defender discarded a card");
}

#[test]
fn penumbra_spider_leaves_a_token_on_death() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let spider = g.add_card_to_battlefield(0, catalog::penumbra_spider());
    g.battlefield_find_mut(spider).unwrap().toughness_bonus -= 4;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    let tokens = g.battlefield.iter().filter(|c| c.controller == 0
        && c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spider)).count();
    assert_eq!(tokens, 1, "Penumbra Spider leaves a 2/4 Spider token");
}

#[test]
fn ember_hauler_sacrifices_to_deal_two() {
    let mut g = two_player_game();
    let hauler = g.add_card_to_battlefield(0, catalog::ember_hauler());
    g.players[0].mana_pool.add_colorless(2);
    let opp = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hauler, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac-ping activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "Ember Hauler dealt 2");
    assert!(g.battlefield_find(hauler).is_none(), "Ember Hauler sacrificed itself");
}

#[test]
fn fire_imp_burns_a_creature_on_etb() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fire_imp());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {2}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "Fire Imp's 2 damage killed the 2/2");
}

#[test]
fn bloodgift_demon_draws_and_loses_one_at_upkeep() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bloodgift_demon());
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
}

// ── Tap-to-ping / pump-grant / vanilla bodies (claude/modern_decks) ──────────

#[test]
fn rootwater_hunter_pings_a_creature() {
    let mut g = two_player_game();
    let hunter = g.add_card_to_battlefield(0, catalog::rootwater_hunter());
    g.clear_sickness(hunter);
    // Target a 1-toughness creature so the single point is lethal.
    let target = g.add_card_to_battlefield(1, catalog::mogg_fanatic()); // 1/1
    g.perform_action(GameAction::ActivateAbility {
        card_id: hunter, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    }).expect("tap-ping activatable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "1 damage killed the 1/1");
    assert!(g.battlefield_find(hunter).unwrap().tapped, "Rootwater Hunter tapped for its ability");
}

#[test]
fn goblin_balloon_brigade_grants_itself_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::goblin_balloon_brigade());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gob, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("flying-grant activatable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(gob).expect("Goblin in play");
    assert!(cp.keywords.contains(&Keyword::Flying), "gained flying until end of turn");
}

#[test]
fn vanilla_keyword_bodies_round_two() {
    use crabomination::card::Keyword;
    let peg = catalog::stormfront_pegasus();
    assert!(peg.keywords.contains(&Keyword::Flying) && (peg.power, peg.toughness) == (2, 1));
    let hawk = catalog::suntail_hawk();
    assert!(hawk.keywords.contains(&Keyword::Flying));
    let giant = catalog::thundering_giant();
    assert!(giant.keywords.contains(&Keyword::Haste) && (giant.power, giant.toughness) == (4, 3));
    let ox = catalog::pillarfield_ox();
    assert_eq!((ox.power, ox.toughness), (2, 4));
}

#[test]
fn maze_of_ith_prevents_combat_damage_to_and_from_attacker() {
    // CR 614.9 — Maze of Ith's controller untaps a target attacker and
    // prevents all combat damage to and by it for the turn, so the defender
    // takes nothing from a 5-power Juggernaut.
    use crabomination::game::types::AttackTarget;
    let mut g = two_player_game();
    let jug = g.add_card_to_battlefield(0, catalog::juggernaut());
    g.clear_sickness(jug);
    let maze = g.add_card_to_battlefield(1, catalog::maze_of_ith());
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: jug, target: AttackTarget::Player(1) },
    ])).expect("Juggernaut attacks");
    drain_stack(&mut g);

    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);

    let life_before = g.players[1].life;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: maze, ability_index: 0, target: Some(Target::Permanent(jug)), additional_targets: Vec::new(), x_value: None,
    }).expect("Maze of Ith targets the attacker");
    drain_stack(&mut g);

    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("advance combat");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life_before,
        "Maze prevents all of the Juggernaut's combat damage");
}

// ── Buyback (CR 702.27) — Corpse Dance ───────────────────────────────────────

#[test]
fn corpse_dance_buyback_returns_to_hand_and_reanimates() {
    // A creature in the graveyard + Corpse Dance cast paying its {2} buyback
    // should reanimate the creature AND return Corpse Dance to hand (not gy).
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let dance = g.add_card_to_hand(0, catalog::corpse_dance());
    // {1}{B}{B} base + {2} buyback = {3}{B}{B}.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpellBuyback {
        card_id: dance, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Corpse Dance castable with buyback");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the top creature is reanimated");
    assert!(g.players[0].hand.iter().any(|c| c.id == dance),
        "buyback returns Corpse Dance to its owner's hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == dance),
        "bought-back spell does not go to the graveyard");
}

#[test]
fn corpse_dance_without_buyback_goes_to_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let dance = g.add_card_to_hand(0, catalog::corpse_dance());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: dance, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Corpse Dance castable for its base cost");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == dance),
        "without buyback Corpse Dance resolves to the graveyard");
    assert!(!g.players[0].hand.iter().any(|c| c.id == dance));
}

// ── Bestow (CR 702.103) — Baleful Eidolon ────────────────────────────────────

#[test]
fn baleful_eidolon_bestowed_buffs_host_and_is_not_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eid = g.add_card_to_hand(0, catalog::baleful_eidolon());
    // Bestow cost {4}{B}.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastBestow {
        card_id: eid, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Baleful Eidolon castable via bestow at a creature");
    drain_stack(&mut g);

    // The bestowed Eidolon is an Aura enchantment, not a creature.
    let ecp = g.computed_permanent(eid).expect("Eidolon on battlefield");
    assert!(!ecp.card_types.contains(&CardType::Creature),
        "a bestowed permanent is not a creature");
    assert!(ecp.card_types.contains(&CardType::Enchantment));
    // The host gains +1/+1 and deathtouch.
    let bcp = g.computed_permanent(bear).unwrap();
    assert_eq!((bcp.power, bcp.toughness), (3, 3), "host gets +1/+1 from bestow");
    assert!(bcp.keywords.contains(&crabomination::card::Keyword::Deathtouch),
        "host gains deathtouch from bestow");
}

#[test]
fn baleful_eidolon_reverts_to_creature_when_host_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eid = g.add_card_to_hand(0, catalog::baleful_eidolon());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastBestow {
        card_id: eid, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bestow cast");
    drain_stack(&mut g);

    // Destroy the host; the bestowed Eidolon stays and becomes a creature.
    g.players[0].graveyard.push(g.battlefield.remove(
        g.battlefield.iter().position(|c| c.id == bear).unwrap()));
    g.check_state_based_actions();

    let ecp = g.computed_permanent(eid).expect("Eidolon stays on battlefield");
    assert!(ecp.card_types.contains(&CardType::Creature),
        "Eidolon reverts to a creature when its host leaves");
    assert_eq!((ecp.power, ecp.toughness), (1, 1), "it's a 1/1 creature again");
}

#[test]
fn hopeful_eidolon_bestow_grants_lifelink_to_host() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eid = g.add_card_to_hand(0, catalog::hopeful_eidolon());
    // Bestow {3}{W}.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastBestow {
        card_id: eid, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hopeful Eidolon castable via bestow");
    drain_stack(&mut g);

    let bcp = g.computed_permanent(bear).unwrap();
    assert_eq!((bcp.power, bcp.toughness), (3, 3), "host gets +1/+1");
    assert!(bcp.keywords.contains(&crabomination::card::Keyword::Lifelink),
        "host gains lifelink from bestow");
    assert!(!g.computed_permanent(eid).unwrap().card_types.contains(&CardType::Creature),
        "bestowed Hopeful Eidolon is not a creature");
}

