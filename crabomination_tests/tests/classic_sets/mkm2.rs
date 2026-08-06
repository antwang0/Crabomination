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

// ── batch 2 ───────────────────────────────────────────────────────────────

/// Tail the Suspect makes a Clue and unlocks a second land drop.
#[test]
fn tail_the_suspect_investigates_and_adds_a_land_drop() {
    let mut g = two_player_game();
    let adv = catalog::kellan_inquisitive_prodigy().adventure.as_ref().unwrap().effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&adv, &ctx).expect("Tail the Suspect");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"));
    assert_eq!(g.players[0].extra_land_plays, 1);
}

/// Kellan's attack blows up an artifact and only draws when it was yours.
#[test]
fn kellan_draws_only_off_his_own_artifact() {
    let mut g = two_player_game();
    let kellan = g.add_card_to_battlefield(0, catalog::kellan_inquisitive_prodigy());
    let theirs = g.add_card_to_battlefield(1, catalog::sol_ring());
    let trigger = catalog::kellan_inquisitive_prodigy().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(theirs)],
        ..crabomination::game::effects::EffectContext::for_ability(kellan, 0, None)
    };
    let before = g.players[0].hand.len();
    g.resolve_effect(&trigger, &ctx).expect("attack trigger");
    assert!(g.battlefield_find(theirs).is_none(), "their Sol Ring is gone");
    assert_eq!(g.players[0].hand.len(), before, "no draw off their artifact");
}

/// Aurelia's Vindicator exiles on unmask and hands the exiles back when it
/// leaves.
#[test]
fn aurelias_vindicator_exiles_then_refunds() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::aurelias_vindicator());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let def = catalog::aurelias_vindicator();
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(victim)],
        x_value: 1,
        ..crabomination::game::effects::EffectContext::for_ability(angel, 0, None)
    };
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("unmask");
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled by the Angel");
    g.resolve_effect(&def.triggered_abilities[1].effect, &ctx).expect("leaves");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returned to its owner's hand");
}

/// Branch of Vitu-Ghazi's unmask adds two mana that survives the phase.
#[test]
fn branch_of_vitu_ghazi_banks_two_mana() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::branch_of_vitu_ghazi());
    let ctx = crabomination::game::effects::EffectContext::for_ability(land, 0, None);
    g.resolve_effect(&catalog::branch_of_vitu_ghazi().triggered_abilities[0].effect, &ctx)
        .expect("turn face up");
    assert_eq!(g.players[0].mana_pool.total(), 2);
    assert_eq!(g.players[0].kept_mana_this_turn.total(), 2, "the mana survives the phase");
}

/// Tenth District Hero grows into a Detective, then into Mileva, whose
/// indestructible anthem only switches on at 5/5.
#[test]
fn tenth_district_hero_levels_into_mileva() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::tenth_district_hero());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let def = catalog::tenth_district_hero();
    let ctx = crabomination::game::effects::EffectContext::for_ability(hero, 0, None);
    g.resolve_effect(&def.activated_abilities[0].effect, &ctx).expect("become a Detective");
    let cp = g.computed_permanent(hero).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Detective));
    assert!(!g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Indestructible));

    g.resolve_effect(&def.activated_abilities[1].effect, &ctx).expect("become Mileva");
    assert_eq!(g.computed_permanent(hero).unwrap().power, 5);
    assert!(
        g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Indestructible),
        "Mileva shields the rest of the team"
    );
}

/// Urgent Necropsy collects evidence equal to the targets' total mana value,
/// then destroys one of each type.
#[test]
fn urgent_necropsy_sweeps_four_types() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::serra_angel()); // MV 5 each
    }
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(ring), Target::Permanent(bear)],
        ..crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None)
    };
    g.resolve_effect(&catalog::urgent_necropsy().effect, &ctx).expect("Necropsy");
    assert!(g.battlefield_find(ring).is_none());
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[0].graveyard.len() < 4, "evidence was collected");
}

/// Deadly Cover-Up wraths, and with evidence collected strips every copy of a
/// named card out of an opponent's zones.
#[test]
fn deadly_cover_up_wraths_and_name_strips() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let marked = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::lightning_bolt());
    let mut ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(marked)],
        ..crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None)
    };
    ctx.cast_collected_evidence = true;
    g.resolve_effect(&catalog::deadly_cover_up().effect, &ctx).expect("Cover-Up");
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert_eq!(
        g.players[1].library.iter().filter(|c| c.definition.name == "Lightning Bolt").count(),
        0,
        "the library copy is gone too"
    );
}

/// Expose the Culprit's first mode flips a face-down creature up.
#[test]
fn expose_the_culprit_unmasks() {
    let mut g = two_player_game();
    let hidden = g.add_card_to_battlefield(0, catalog::exalted_angel());
    g.battlefield_find_mut(hidden).unwrap().turn_face_down();
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(hidden)],
        mode: 0,
        ..crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None)
    };
    g.resolve_effect(&catalog::expose_the_culprit().effect, &ctx).expect("Expose");
    assert!(!g.battlefield_find(hidden).unwrap().face_down);
}

/// Hedge Whisperer animates a land as a 5/5 hasty Plant Boar.
#[test]
fn hedge_whisperer_animates_a_land() {
    let mut g = two_player_game();
    let whisperer = g.add_card_to_battlefield(0, catalog::hedge_whisperer());
    g.battlefield_find_mut(whisperer).unwrap().tapped = true;
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(land)],
        ..crabomination::game::effects::EffectContext::for_ability(whisperer, 0, None)
    };
    g.resolve_effect(&catalog::hedge_whisperer().activated_abilities[0].effect, &ctx)
        .expect("animate");
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.keywords.contains(&Keyword::Haste));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "it's still a land");
}

/// Doppelgang at X=2 makes two copies of each of two permanents.
#[test]
fn doppelgang_squares_the_board() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::serra_angel());
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        x_value: 2,
        ..crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None)
    };
    g.resolve_effect(&catalog::doppelgang().effect, &ctx).expect("Doppelgang");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Serra Angel").count(), 3);
}

/// Kylox eats the team, exiles that much library, and free-casts the spells.
#[test]
fn kylox_converts_power_into_free_spells() {
    let mut g = two_player_game();
    let kylox = g.add_card_to_battlefield(0, catalog::kylox_visionary_inventor());
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4 power
    g.add_card_to_library(0, catalog::lightning_bolt());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ctx = crabomination::game::effects::EffectContext::for_ability(kylox, 0, None);
    g.resolve_effect(&catalog::kylox_visionary_inventor().triggered_abilities[0].effect, &ctx)
        .expect("attack trigger");
    assert_eq!(g.exile.len() + g.stack.len(), 4, "four power exiled four cards");
    assert_eq!(g.stack.len(), 1, "and the Bolt among them is cast free");
}

/// Kylox's Voltstrider animates off evidence and casts one exiled spell on
/// attack.
#[test]
fn kyloxs_voltstrider_animates_then_casts_one() {
    let mut g = two_player_game();
    let vehicle = g.add_card_to_battlefield(0, catalog::kyloxs_voltstrider());
    let def = catalog::kyloxs_voltstrider();
    let ctx = crabomination::game::effects::EffectContext::for_ability(vehicle, 0, None);
    g.resolve_effect(&def.activated_abilities[0].effect, &ctx).expect("animate");
    assert!(
        g.computed_permanent(vehicle)
            .unwrap()
            .card_types
            .contains(&crabomination::card::CardType::Creature)
    );
    for _ in 0..2 {
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
        g.players[0].library.retain(|c| c.id != bolt);
        let mut inst = crabomination::card::CardInstance::new(bolt, catalog::lightning_bolt(), 0);
        inst.exiled_with = Some(vehicle);
        g.exile.push(inst);
    }
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("attack trigger");
    assert_eq!(g.stack.len(), 1, "the cap is one spell per attack");
}

/// Reenact the Crime copies a card that hit a graveyard this turn and casts
/// the copy for free.
#[test]
fn reenact_the_crime_copies_a_fresh_corpse() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = vec![];
    g.destroy_permanent(bear, false, &mut evs);
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(bear)],
        ..crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None)
    };
    g.resolve_effect(&catalog::reenact_the_crime().effect, &ctx).expect("Reenact");
    assert!(g.exile.iter().any(|c| c.id == bear), "the original is exiled");
    assert_eq!(g.stack.len(), 1, "and a free copy is on the stack");
}

/// Anzrag's Rampage wrecks their artifacts and digs as deep as the turn's
/// artifact deaths.
#[test]
fn anzrags_rampage_digs_per_dead_artifact() {
    let mut g = two_player_game();
    let mut evs = vec![];
    for _ in 0..2 {
        let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
        g.destroy_permanent(ring, false, &mut evs);
    }
    let theirs = g.add_card_to_battlefield(1, catalog::sol_ring());
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::forest());
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&catalog::anzrags_rampage().effect, &ctx).expect("Rampage");
    assert!(g.battlefield_find(theirs).is_none(), "their artifact is destroyed");
    let cp = g.computed_permanent(angel).expect("the Angel was slammed");
    assert!(cp.keywords.contains(&Keyword::Haste));
    assert_eq!(g.delayed_triggers.len(), 1, "and is scheduled back to hand");
}

/// Agency Outfitter fetches both gadgets straight onto the battlefield.
#[test]
fn agency_outfitter_fetches_both_gadgets() {
    let mut g = two_player_game();
    let glass = g.add_card_to_library(0, catalog::magnifying_glass());
    let cap = g.add_card_to_graveyard(0, catalog::thinking_cap());
    let id = g.add_card_to_battlefield(0, catalog::agency_outfitter());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(glass)),
        DecisionAnswer::Search(Some(cap)),
    ]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, 0, None);
    g.resolve_effect(&catalog::agency_outfitter().triggered_abilities[0].effect, &ctx)
        .expect("ETB");
    for name in ["Magnifying Glass", "Thinking Cap"] {
        assert!(g.battlefield.iter().any(|c| c.definition.name == name), "{name} fetched");
    }
}

/// Thinking Cap equips a Detective for {1} and anyone else for {3}.
#[test]
fn thinking_cap_equips_detectives_cheaply() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::thinking_cap());
    let sleuth = g.add_card_to_battlefield(0, catalog::novice_inspector());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: cap, target: sleuth }).expect("equip for {1}");
    let cp = g.computed_permanent(sleuth).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "+1/+2 from the Cap");

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::Equip { equipment: cap, target: bear }).is_err(),
        "a non-Detective still pays {{3}}"
    );
}

/// Thinking Cap's printed "Equip Detective {1}" now applies (it used to ship
/// as the flat Equip {3}).
#[test]
fn thinking_cap_equips_a_detective_for_one() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::thinking_cap());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::Equip { equipment: cap, target: bear }).is_err(),
        "a non-Detective pays the flat {{3}}"
    );
    let sleuth = g.add_card_to_battlefield(0, catalog::novice_inspector());
    g.perform_action(GameAction::Equip { equipment: cap, target: sleuth }).expect("equip for {1}");
    let cp = g.computed_permanent(sleuth).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "+1/+2 from the Cap");
}
