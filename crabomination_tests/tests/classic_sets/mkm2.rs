//! Tests for the MKM second gap wave (`catalog::sets::mkm2`).

use crabomination::TurnStep;
use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn attack_with(g: &mut GameState, id: CardId) {
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

fn solve_now(g: &mut GameState) {
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    drain_stack(g);
}

/// Goblin Maskmaker's attack trigger drops the face-down cast cost to {2} for
/// the rest of the turn, and it resets at cleanup.
#[test]
fn goblin_maskmaker_discounts_face_down_casts_for_the_turn() {
    let mut g = two_player_game();
    let mask = g.add_card_to_battlefield(0, catalog::goblin_maskmaker());
    assert_eq!(g.face_down_cast_cost(0), 3);
    attack_with(&mut g, mask);
    assert_eq!(g.face_down_cast_cost(0), 2, "attacking shaves {{1}}");
    g.players[0].face_down_discount_this_turn = 0; // cleanup (CR 514.2)
    assert_eq!(g.face_down_cast_cost(0), 3, "the grant ends at cleanup");
}

/// Tin Street Gossip's mana funds a face-down cast but nothing else.
#[test]
fn tin_street_gossip_mana_only_pays_for_face_down_casts() {
    let mut g = two_player_game();
    let gossip = g.add_card_to_battlefield(0, catalog::tin_street_gossip());
    g.clear_sickness(gossip);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gossip,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for RG");
    // A plain creature spell can't touch it…
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bears,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "restricted mana can't fund a normal cast"
    );
    // …but a morph cast can.
    let morph = g.add_card_to_hand(0, catalog::exalted_angel());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastFaceDown { card_id: morph }).expect("face-down cast");
    assert_eq!(g.players[0].mana_pool.total(), 0, "all three paid the face-down cast");
}

/// Judith's first mode makes the instant she watched deal lifelinking,
/// deathtouching damage.
#[test]
fn judith_grants_the_spell_deathtouch_and_lifelink() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::judith_carnage_connoisseur());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flier
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].life = 10;
    g.players[0].mana_pool.add(Color::Red, 1);
    // Mode 0 = "that spell gains deathtouch and lifelink".
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "3 deathtouch damage kills the 4/4");
    assert_eq!(g.players[0].life, 13, "and lifelink paid three back");
}

/// Case of the Burning Masks solves once three of your sources have dealt
/// damage this turn.
#[test]
fn case_of_the_burning_masks_counts_distinct_damage_sources() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::case_of_the_burning_masks());
    let mut evs = vec![];
    for _ in 0..2 {
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.deal_damage_to_from(
            crabomination::game::effects::EntityRef::Player(1),
            1,
            Some(src),
            &mut evs,
        );
    }
    solve_now(&mut g);
    assert!(!g.battlefield_find(id).unwrap().case_solved, "two sources is not enough");
    let third = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1),
        1,
        Some(third),
        &mut evs,
    );
    solve_now(&mut g);
    assert!(g.battlefield_find(id).unwrap().case_solved, "the third source solves it");
}

/// Case of the Gorgon's Kiss solves off three creature cards reaching
/// graveyards and then animates itself as a 4/4 deathtouch lifelinker.
#[test]
fn case_of_the_gorgons_kiss_solves_and_animates() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::case_of_the_gorgons_kiss());
    let mut evs = vec![];
    for _ in 0..3 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.destroy_permanent(bear, false, &mut evs);
    }
    solve_now(&mut g);
    assert!(g.battlefield_find(id).unwrap().case_solved);
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

/// Burden of Proof pumps your own Detective and shrinks anything else to 1/1
/// with a Detective-shaped blocking ban.
#[test]
fn burden_of_proof_rewards_detectives_and_punishes_the_rest() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let aura = g.add_card_to_battlefield(0, catalog::burden_of_proof());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(theirs);
    let cp = g.computed_permanent(theirs).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "a non-Detective is shrunk");
    assert!(cp.keywords.contains(&Keyword::CantBlockCreatureType(
        crabomination::card::CreatureType::Detective
    )));

    let mine = g.add_card_to_battlefield(0, catalog::novice_inspector()); // 1/2 Detective
    let aura2 = g.add_card_to_battlefield(0, catalog::burden_of_proof());
    g.battlefield_find_mut(aura2).unwrap().attached_to = Some(mine);
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "your own Detective gets +2/+2");
}

/// Break Out deploys a cheap creature with haste and bottoms the rest.
#[test]
fn break_out_deploys_a_cheap_creature_with_haste() {
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bears])]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&catalog::break_out().effect, &ctx).expect("Break Out");
    let cp = g.computed_permanent(bears).expect("bears deployed");
    assert!(cp.keywords.contains(&Keyword::Haste));
    assert_eq!(g.players[0].library.len(), 5, "the rest went to the bottom");
}

/// Sudden Setback's permanent mode sends a nonland permanent to its owner's
/// library.
#[test]
fn sudden_setback_libraries_a_permanent() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(victim)],
        mode: 1,
        ..crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None)
    };
    g.resolve_effect(&catalog::sudden_setback().effect, &ctx).expect("Setback");
    assert!(g.battlefield_find(victim).is_none());
    assert!(g.players[1].library.iter().any(|c| c.id == victim), "back into the library");
}

/// Pull reanimates two creature cards with haste and schedules their sacrifice.
#[test]
fn pull_reanimates_two_with_haste_and_a_fuse() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(1, catalog::serra_angel());
    let right = &catalog::push_pull().split.as_ref().unwrap().right.effect.clone();
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        ..crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None)
    };
    g.resolve_effect(right, &ctx).expect("Pull");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 2);
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::Haste));
    assert_eq!(g.delayed_triggers.len(), 2, "both are on the end-step fuse");
}

/// Flotsam mills three and leaves a Clue behind.
#[test]
fn flotsam_mills_and_investigates() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&catalog::flotsam_jetsam().effect, &ctx).expect("Flotsam");
    assert_eq!(g.players[0].graveyard.len(), 3);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"));
}

/// Jetsam casts exactly one free spell out of an opponent's graveyard.
#[test]
fn jetsam_caps_the_free_cast_at_one_per_opponent() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let right = catalog::flotsam_jetsam().split.as_ref().unwrap().right.effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&right, &ctx).expect("Jetsam");
    assert_eq!(g.stack.len(), 1, "one opponent → one free cast");
}

/// Bustle pumps the team and flips a face-down creature up.
#[test]
fn bustle_pumps_and_unmasks() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hidden = g.add_card_to_battlefield(0, catalog::exalted_angel());
    g.battlefield_find_mut(hidden).unwrap().turn_face_down();
    let right = catalog::hustle_bustle().split.as_ref().unwrap().right.effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&right, &ctx).expect("Bustle");
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert!(!g.battlefield_find(hidden).unwrap().face_down, "the mask came off");
}

/// Hustle's grant forces the creature into combat.
#[test]
fn hustle_forces_an_attack() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bears);
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(bears)],
        ..crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None)
    };
    g.resolve_effect(&catalog::hustle_bustle().effect, &ctx).expect("Hustle");
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![])).is_err(),
        "a must-attack creature can't sit out"
    );
}

/// Coveted Falcon's attack trigger repossesses a permanent you own.
#[test]
fn coveted_falcon_reclaims_what_you_own() {
    let mut g = two_player_game();
    let falcon = g.add_card_to_battlefield(0, catalog::coveted_falcon());
    let stolen = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(stolen).unwrap().controller = 1;
    attack_with(&mut g, falcon);
    assert_eq!(g.battlefield_find(stolen).unwrap().controller, 0, "clawed back");
}

/// Yarus returns a dead face-down creature and flips it face up.
#[test]
fn yarus_replays_a_dead_face_down_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::yarus_roar_of_the_old_gods());
    let hidden = g.add_card_to_battlefield(0, catalog::exalted_angel());
    g.battlefield_find_mut(hidden).unwrap().turn_face_down();
    let mut evs = vec![];
    g.destroy_permanent(hidden, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g.battlefield_find(hidden).expect("returned to the battlefield");
    assert!(!back.face_down, "and turned face up");
}

/// Yarus's anthem hastes your other creatures but not himself.
#[test]
fn yarus_hastes_the_rest_of_the_team() {
    let mut g = two_player_game();
    let yarus = g.add_card_to_battlefield(0, catalog::yarus_roar_of_the_old_gods());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::Haste));
    assert!(!g.computed_permanent(yarus).unwrap().keywords.contains(&Keyword::Haste));
}

/// Illicit Masquerade marks your creatures, then trades a marked death for a
/// reanimation.
#[test]
fn illicit_masquerade_trades_a_marked_death_for_a_reanimation() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let buried = g.add_card_to_graveyard(0, catalog::serra_angel());
    let mask = g.add_card_to_battlefield(0, catalog::illicit_masquerade());
    let ctx = crabomination::game::effects::EffectContext::for_ability(mask, 0, None);
    g.resolve_effect(&catalog::illicit_masquerade().triggered_abilities[0].effect, &ctx)
        .expect("ETB");
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::Impostor), 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![buried])]));
    let mut evs = vec![];
    g.destroy_permanent(bears, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bears), "the marked corpse is exiled");
    assert!(g.battlefield_find(buried).is_some(), "and the Angel walks");
}

/// Blood Spatter Analysis stains on each death and pops at five.
#[test]
fn blood_spatter_analysis_pops_at_five_stains() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::blood_spatter_analysis());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.add_card_to_graveyard(0, catalog::serra_angel());
    let death = &catalog::blood_spatter_analysis().triggered_abilities[1].effect;
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, 0, None);
    for _ in 0..4 {
        g.resolve_effect(death, &ctx).expect("death trigger");
    }
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Bloodstain), 4);
    g.resolve_effect(death, &ctx).expect("fifth death trigger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "the fifth stain sacrifices it");
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Serra Angel"),
        "and buys back a creature card"
    );
}

/// Connecting the Dots banks an exiled card per attack and hands the pile back.
#[test]
fn connecting_the_dots_banks_then_returns_the_pile() {
    let mut g = two_player_game();
    let dots = g.add_card_to_battlefield(0, catalog::connecting_the_dots());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack_with(&mut g, attacker);
    assert_eq!(g.exile.len(), 1, "one card banked");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dots,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cash in");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "the banked card came back");
}

/// Lazav's attack trigger exiles a graveyard card and investigates.
#[test]
fn lazav_exiles_and_investigates_on_attack() {
    let mut g = two_player_game();
    let lazav = g.add_card_to_battlefield(0, catalog::lazav_wearer_of_faces());
    let corpse = g.add_card_to_graveyard(1, catalog::serra_angel());
    attack_with(&mut g, lazav);
    assert!(g.exile.iter().any(|c| c.id == corpse));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"));
}

/// Niv-Mizzet counts distinct exactly-two-color pairs and can't be targeted by
/// a multicolored spell.
#[test]
fn niv_mizzet_guildpact_counts_pairs_and_dodges_gold_removal() {
    let mut g = two_player_game();
    let niv = g.add_card_to_battlefield(0, catalog::niv_mizzet_guildpact());
    assert!(
        g.computed_permanent(niv).unwrap().keywords.contains(&Keyword::HexproofFromMulticolored)
    );
    // Judith is exactly two colors — one distinct pair.
    g.add_card_to_battlefield(0, catalog::judith_carnage_connoisseur());
    g.players[0].life = 20;
    g.players[1].life = 20;
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Player(1), Target::Player(0)],
        ..crabomination::game::effects::EffectContext::for_ability(niv, 0, None)
    };
    g.resolve_effect(&catalog::niv_mizzet_guildpact().triggered_abilities[0].effect, &ctx)
        .expect("combat trigger");
    assert_eq!(g.players[0].life, 21, "one pair → one life");
    assert_eq!(g.players[1].life, 19, "and one damage");
}
