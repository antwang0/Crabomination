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

// ── Coverage backfill: previously-untested real cards (decks / mod_set) ──────

#[test]
fn naturalize_destroys_target_artifact() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::pithing_needle()); // a {1} artifact
    let nat = g.add_card_to_hand(0, catalog::naturalize());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: nat, target: Some(Target::Permanent(rock)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Naturalize castable for {1}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == rock), "artifact destroyed");
}

#[test]
fn chalice_of_the_void_counters_matching_mana_value_spell() {
    // Chalice for X=1 → 1 charge counter; a MV-1 spell is countered on cast
    // (CR 614.12 + the SpellCast/MV-match trigger). Chalice counters *any*
    // player's matching spell, including its own controller's.
    let mut g = two_player_game();
    let chalice = g.add_card_to_hand(0, catalog::chalice_of_the_void());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: chalice, target: None, additional_targets: vec![],
        mode: None, x_value: Some(1),
    }).expect("Chalice castable for {X}{X} at X=1");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().find(|c| c.id == chalice)
            .map(|c| c.counter_count(crabomination::card::CounterType::Charge)),
        Some(1), "Chalice enters with 1 charge counter");

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before, "Chalice counters the MV-1 bolt");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "bolt countered to graveyard");
}

#[test]
fn candelabra_of_tawnos_untaps_up_to_x_lands() {
    let mut g = two_player_game();
    let cand = g.add_card_to_battlefield(0, catalog::candelabra_of_tawnos());
    g.clear_sickness(cand);
    // Two tapped Forests on the battlefield.
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    for id in [f1, f2] {
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) { c.tapped = true; }
    }
    g.players[0].mana_pool.add_colorless(2); // for {X} = 2
    g.perform_action(GameAction::ActivateAbility {
        card_id: cand, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: Some(2), mode: None,
    }).expect("Candelabra activates with X=2");
    drain_stack(&mut g);
    assert!([f1, f2].iter().all(|id|
        g.battlefield.iter().find(|c| c.id == *id).map(|c| !c.tapped).unwrap_or(false)),
        "both lands untapped by Candelabra");
}

#[test]
fn basking_broodscale_etb_makes_two_spawn_and_enters_two_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::basking_broodscale());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Basking Broodscale castable for {1}{G}");
    drain_stack(&mut g);
    let spawn = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawn, 2, "ETB makes two Eldrazi Spawn");
    let view = g.compute_battlefield();
    let bs = view.iter().find(|c| c.id == id).unwrap();
    assert_eq!((bs.power, bs.toughness), (2, 3), "0/1 + two +1/+1 counters = 2/3");
}

#[test]
fn sowing_mycospawn_etb_searches_a_land_to_battlefield() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::sowing_mycospawn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    let lands_before = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.is_land()).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sowing Mycospawn castable for {4}{G}");
    drain_stack(&mut g);
    let lands_after = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.is_land()).count();
    assert_eq!(lands_after, lands_before + 1, "ETB tutors a land onto the battlefield");
}

#[test]
fn archdruids_charm_mode_one_adds_two_counters_to_your_creature() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::archdruids_charm());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Archdruid's Charm castable for {G}{G}{G}");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let b = view.iter().find(|c| c.id == bears).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "2/2 + two +1/+1 counters = 4/4");
}

#[test]
fn awaken_the_honored_dead_returns_all_your_creatures() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::awaken_the_honored_dead());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Awaken the Honored Dead castable");
    drain_stack(&mut g);
    assert!([c1, c2].iter().all(|id| g.battlefield.iter().any(|c| c.id == *id)),
        "both creatures reanimated from the graveyard");
}

#[test]
fn summoners_pact_searches_a_green_creature_to_hand() {
    let mut g = two_player_game();
    let elf = g.add_card_to_library(0, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(elf))]));
    let id = g.add_card_to_hand(0, catalog::summoners_pact());
    // Free to cast ({0}).
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Summoner's Pact is free to cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == elf),
        "a green creature is tutored to hand");
}

#[test]
fn finale_of_devastation_searches_creature_to_battlefield() {
    let mut g = two_player_game();
    let elf = g.add_card_to_library(0, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(elf))]));
    let id = g.add_card_to_hand(0, catalog::finale_of_devastation());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Finale castable at X=2");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == elf),
        "the tutored creature enters the battlefield");
}

#[test]
fn dakkon_enters_with_loyalty_equal_to_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::swamp());
    let dakkon = g.add_card_to_hand(0, catalog::dakkon_shadow_slayer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, dakkon);
    assert_eq!(
        g.battlefield_find(dakkon).unwrap().counter_count(crabomination::card::CounterType::Loyalty),
        3, "loyalty = lands you control",
    );
}

#[test]
fn dakkon_shadow_slayer_minus_three_exiles_a_creature() {
    let mut g = two_player_game();
    let dakkon = g.add_card_to_battlefield(0, catalog::dakkon_shadow_slayer());
    g.battlefield_find_mut(dakkon).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 4);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: dakkon, ability_index: 1, target: Some(Target::Permanent(victim)),
    }).expect("Dakkon -3 activates");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "target creature exiled");
}

#[test]
fn dakkon_minus_six_puts_an_artifact_from_graveyard_onto_battlefield() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::dakkon_shadow_slayer());
    g.battlefield_find_mut(pw).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 7);
    let stone = g.add_card_to_graveyard(0, catalog::mind_stone());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: pw, ability_index: 2, target: None,
    }).expect("Dakkon -6 castable at 7 loyalty");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_some(), "artifact entered from the graveyard");
}

#[test]
fn saheeli_rai_minus_seven_fetches_three_artifacts() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let saheeli = g.add_card_to_battlefield(0, catalog::saheeli_rai());
    g.battlefield_find_mut(saheeli).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 7);
    let a = g.add_card_to_library(0, catalog::mind_stone());
    let b = g.add_card_to_library(0, catalog::pithing_needle());
    let c = g.add_card_to_library(0, catalog::shuko());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
        DecisionAnswer::Search(Some(c)),
    ]));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: saheeli, ability_index: 2, target: None,
    }).expect("Saheeli -7 castable at 10 loyalty");
    drain_stack(&mut g);
    for id in [a, b, c] {
        assert!(g.battlefield_find(id).is_some(), "searched artifact on battlefield");
    }
}

/// Saheeli -7 "with different names": a second copy of an already-fetched
/// name isn't a legal pick.
#[test]
fn saheeli_rai_minus_seven_rejects_duplicate_names() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let saheeli = g.add_card_to_battlefield(0, catalog::saheeli_rai());
    g.battlefield_find_mut(saheeli).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 7);
    let a = g.add_card_to_library(0, catalog::mind_stone());
    let dup = g.add_card_to_library(0, catalog::mind_stone());
    let c = g.add_card_to_library(0, catalog::shuko());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(dup)),
        DecisionAnswer::Search(Some(c)),
    ]));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: saheeli, ability_index: 2, target: None,
    }).expect("Saheeli -7 activatable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some());
    assert!(g.battlefield_find(dup).is_none(), "duplicate name stays in the library");
    assert!(g.battlefield_find(c).is_some());
}

#[test]
fn containment_priest_is_a_two_two_with_flash() {
    let g_def = catalog::containment_priest();
    assert_eq!((g_def.power, g_def.toughness), (2, 2));
    assert!(g_def.keywords.contains(&crabomination::card::Keyword::Flash),
        "Containment Priest has Flash");
}

/// Containment Priest exiles a reanimated (non-cast) nontoken creature
/// instead of letting it enter the battlefield.
#[test]
fn containment_priest_exiles_reanimated_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::containment_priest());

    let atraxa = g.add_card_to_library(0, catalog::atraxa_grand_unifier());
    let pos = g.players[0].library.iter().position(|c| c.id == atraxa).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    let id = g.add_card_to_hand(0, catalog::reanimate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(atraxa)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reanimate castable for {B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == atraxa),
        "reanimated creature should not reach the battlefield");
    assert!(g.exile.iter().any(|c| c.id == atraxa),
        "Containment Priest exiles the non-cast creature instead");
}

/// Containment Priest does not interfere with a normally-cast creature
/// spell (it was cast, so the replacement doesn't apply).
#[test]
fn containment_priest_allows_cast_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::containment_priest());
    let id = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "a cast creature still enters normally under Containment Priest");
}

/// `legal_attackers` lists only untapped, non-sick creatures during the
/// viewer's Declare Attackers step; `legal_blockers` lists creatures that
/// can block a declared attacker during Declare Blockers.
#[test]
fn client_view_legal_attacker_and_blocker_hints() {
    let mut g = two_player_game();
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let ready = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(ready);
    let tapped = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(tapped);
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == tapped) { c.tapped = true; }

    let atk = g.legal_attackers(0);
    assert!(atk.contains(&ready), "untapped non-sick creature is a legal attacker");
    assert!(!atk.contains(&tapped), "tapped creature is not a legal attacker");
    // Off-step / wrong seat yields nothing.
    assert!(g.legal_attackers(1).is_empty(), "non-active seat has no legal attackers");

    // Declare an attack, move to blockers, check the defender's blocker hint.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ready, target: AttackTarget::Player(1),
    }])).expect("declare attackers");
    g.step = TurnStep::DeclareBlockers;
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    let blk = g.legal_blockers(1);
    assert!(blk.contains(&blocker), "untapped creature can block the declared attacker");
}

#[test]
fn legal_attackers_honors_defender_board_restriction() {
    let mut g = two_player_game();
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::island()); // your Island doesn't count
    let dd = g.add_card_to_battlefield(0, catalog::dandan());
    g.clear_sickness(dd);
    assert!(!g.legal_attackers(0).contains(&dd),
        "Dandân isn't a legal attacker while no opponent controls an Island");
    g.add_card_to_battlefield(1, catalog::island());
    assert!(g.legal_attackers(0).contains(&dd),
        "Dandân becomes a legal attacker once an opponent controls an Island");
}

/// Reverberate copies a spell and the copy can be repointed at a new
/// target (CR 115.7). Bolt hits bear1; the copy is steered onto bear2.
#[test]
fn reverberate_copies_spell_with_new_target() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // P0 casts Lightning Bolt at bear1.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");

    // Reverberate the bolt; the copy chooses bear2 as its new target.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Target(Target::Permanent(bear2)),
    ]));
    let rev = g.add_card_to_hand(0, catalog::reverberate());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: rev, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reverberate castable");
    drain_stack(&mut g);

    // Both bears die — bear1 to the original bolt, bear2 to the repointed copy.
    assert!(g.battlefield_find(bear1).is_none(), "bear1 killed by original bolt");
    assert!(g.battlefield_find(bear2).is_none(), "bear2 killed by repointed copy");
}

/// Redirect repoints a spell's target (CR 115.7). Bolt aimed at my bear
/// is redirected back onto the opponent's bear.
#[test]
fn redirect_changes_a_spells_target() {
    let mut g = two_player_game();
    let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // P0 casts Lightning Bolt at its own bear (the spell to be redirected).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(my_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");

    // P0 redirects the bolt onto opp_bear.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Target(Target::Permanent(opp_bear)),
    ]));
    let redir = g.add_card_to_hand(0, catalog::redirect());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: redir, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Redirect castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(my_bear).is_some(), "my bear survives the redirect");
    assert!(g.battlefield_find(opp_bear).is_none(), "opponent's bear takes the bolt");
}

/// Fork is a {R}{R} copy-with-new-targets instant.
#[test]
fn fork_is_a_red_copy_spell() {
    let d = catalog::fork();
    assert!(matches!(d.effect, crabomination::effect::Effect::CopySpellMayChooseTargets { .. }));
    assert_eq!(d.cost.cmc(), 2);
}

#[test]
fn simian_spirit_guide_is_a_two_two_ape_spirit() {
    let d = catalog::simian_spirit_guide();
    assert_eq!((d.power, d.toughness), (2, 2));
    assert!(d.has_creature_type(crabomination::card::CreatureType::Ape));
}

/// Simian Spirit Guide: "Exile this from your hand: Add {R}." pitches for
/// a red mana and exiles itself.
#[test]
fn simian_spirit_guide_pitches_from_hand_for_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::simian_spirit_guide());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("pitch ability activates from hand");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red");
    assert!(g.exile.iter().any(|c| c.id == id), "pitched card is exiled");
}

#[test]
fn clone_enters_as_a_copy_of_a_creature() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 Flying Vigilance
    let id = g.add_card_to_hand(0, catalog::clone_card());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Clone castable for {3}{U}");
    drain_stack(&mut g);
    let clone = g.battlefield.iter().find(|c| c.id == id).expect("Clone on battlefield");
    assert_eq!(clone.definition.name, "Serra Angel");
    assert_eq!((clone.definition.power, clone.definition.toughness), (4, 4));
    assert!(clone.definition.keywords.contains(&Keyword::Flying));
}

/// Phyrexian Metamorph copies a creature and is an artifact in addition to
/// the copied types (the `extra_card_types` rider).
#[test]
fn phyrexian_metamorph_copies_creature_and_stays_artifact() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 Flying Vigilance
    let id = g.add_card_to_hand(0, catalog::phyrexian_metamorph());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Metamorph castable for {3}{U/P}");
    drain_stack(&mut g);
    let m = g.battlefield.iter().find(|c| c.id == id).expect("Metamorph on battlefield");
    assert_eq!(m.definition.name, "Serra Angel");
    assert_eq!((m.definition.power, m.definition.toughness), (4, 4));
    assert!(m.definition.card_types.contains(&CardType::Artifact),
        "an artifact in addition to its copied types");
    assert!(m.definition.card_types.contains(&CardType::Creature));
}

/// Phyrexian Metamorph can copy a noncreature artifact too (and the copy is
/// still an artifact, naturally).
#[test]
fn phyrexian_metamorph_can_copy_an_artifact() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::phyrexian_metamorph());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let m = g.battlefield.iter().find(|c| c.id == id).expect("on battlefield");
    assert_eq!(m.definition.name, "Sol Ring");
    assert!(m.definition.card_types.contains(&CardType::Artifact));
}

/// Force of Vigor destroys two artifact/enchantment targets when hard-cast.
#[test]
fn force_of_vigor_destroys_two_targets() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let arena = g.add_card_to_battlefield(1, catalog::phyrexian_arena());
    let id = g.add_card_to_hand(0, catalog::force_of_vigor());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![Target::Permanent(arena)],
        mode: None, x_value: None,
    }).expect("hard-castable for {2}{G}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none(), "Sol Ring destroyed");
    assert!(g.battlefield_find(arena).is_none(), "Phyrexian Arena destroyed");
}

/// Force of Vigor's pitch alt-cost works on the opponent's turn (exile a
/// green card from hand instead of paying mana).
#[test]
fn force_of_vigor_pitch_cast_on_opponents_turn() {
    let mut g = two_player_game();
    g.active_player_idx = 1; // opponent's turn
    g.priority.player_with_priority = 0;
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::force_of_vigor());
    let pitch = g.add_card_to_hand(0, catalog::grizzly_bears()); // a green card
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: Some(pitch),
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("pitch-castable on opponent's turn");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == pitch), "green card exiled to pitch");
    assert!(g.battlefield_find(ring).is_none(), "Sol Ring destroyed");
}

#[test]
fn clone_with_no_creature_to_copy_dies_as_zero_zero() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::clone_card());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id),
        "a 0/0 Clone with nothing to copy dies to SBA");
}

#[test]
fn steady_progress_proliferates_then_draws() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::steady_progress());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {2}{U}");
    drain_stack(&mut g);
    let n = g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(n, 2, "proliferate grew the +1/+1 counter");
    // -1 cast from hand, +1 drawn.
    assert_eq!(g.players[0].hand.len(), hand_before, "Steady Progress drew a card");
}

/// CR 701.34 / 606 — Proliferate adds a loyalty counter to a planeswalker
/// you control (loyalty counters are counters).
#[test]
fn cr_701_34_proliferate_adds_loyalty_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
    let before = g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty);
    let id = g.add_card_to_hand(0, catalog::steady_progress());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty),
        before + 1,
        "proliferate added one loyalty counter to your planeswalker",
    );
}

#[test]
fn volt_charge_burns_then_proliferates() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == mine).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    let id = g.add_card_to_hand(0, catalog::volt_charge());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable for {2}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 3, "3 damage to the opponent");
    let n = g.battlefield.iter().find(|c| c.id == mine).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(n, 2, "proliferate grew your +1/+1 counter");
}

#[test]
fn contagion_clasp_etb_shrinks_a_creature() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::contagion_clasp());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable for {4}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counter_count(CounterType::MinusOneMinusOne), 1, "ETB -1/-1 counter");
}

#[test]
fn inexorable_tide_proliferates_on_your_spell() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inexorable_tide());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    // Cast any cheap spell; the cast trigger proliferates.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 2, "Inexorable Tide proliferated on cast");
}

#[test]
fn spike_feeder_enters_with_two_counters_and_trades_one_for_life() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spike_feeder());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {2}{G}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 2, "enters with two +1/+1 counters");
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "gained 2 life");
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 1, "spent a counter");
}

#[test]
fn journey_to_nowhere_exiles_and_returns_on_leave() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let jtn = g.add_card_to_hand(0, catalog::journey_to_nowhere());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: jtn, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature exiled");
    // Journey leaving returns the creature.
    g.remove_to_graveyard_with_triggers(jtn);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "creature returned when Journey left");
}

#[test]
fn banishing_light_exiles_a_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bl = g.add_card_to_hand(0, catalog::banishing_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bl, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "permanent exiled");
}

#[test]
fn seal_of_cleansing_sacrifices_to_destroy_an_artifact() {
    let mut g = two_player_game();
    let seal = g.add_card_to_battlefield(0, catalog::seal_of_cleansing());
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.players[0].mana_pool.add_colorless(0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: seal, ability_index: 0, target: Some(Target::Permanent(rock)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac ability");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == seal), "Seal sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == rock), "artifact destroyed");
}

#[test]
fn dissolve_counters_a_spell_and_scrys() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    // Opponent casts a creature; we counter it.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp bear cast");
    let dissolve = g.add_card_to_hand(0, catalog::dissolve());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    // Player 0 gets priority to respond to the bear on the stack.
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: dissolve, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Dissolve cast");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "the bear spell was countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "countered spell to graveyard");
}

#[test]
fn soul_warden_gains_life_when_another_creature_enters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::soul_warden());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "Soul Warden gained 1 when the bear entered");
}

/// Helper: cast a Grizzly Bears for P0 and resolve only the bear (two
/// priority passes), leaving any ETB triggers sitting on the stack.
#[cfg(test)]
fn cast_bear_and_resolve_only_it(g: &mut GameState) {
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
}

fn stack_trigger_sources(g: &GameState) -> Vec<crabomination::card::CardId> {
    g.stack.iter().filter_map(|si| match si {
        crabomination::game::types::StackItem::Trigger { source, .. } => Some(*source),
        _ => None,
    }).collect()
}

#[test]
fn same_controller_triggers_keep_default_order_for_bots() {
    // Two Soul Wardens (P0). A bot/AutoDecider controller doesn't get a
    // CR 603.3b ordering prompt, so the triggers push in battlefield order.
    let mut g = two_player_game();
    let w1 = g.add_card_to_battlefield(0, catalog::soul_warden());
    let w2 = g.add_card_to_battlefield(0, catalog::soul_warden());
    cast_bear_and_resolve_only_it(&mut g);
    assert_eq!(stack_trigger_sources(&g), vec![w1, w2],
        "default same-controller order is battlefield order");
}

#[test]
fn wants_ui_controller_orders_own_simultaneous_triggers() {
    // CR 603.3b — a wants_ui controller chooses the stack-push order of
    // their two simultaneous Soul Warden triggers via Decision::OrderTriggers.
    let mut g = two_player_game();
    let w1 = g.add_card_to_battlefield(0, catalog::soul_warden());
    let w2 = g.add_card_to_battlefield(0, catalog::soul_warden());
    g.players[0].wants_ui = true;
    cast_bear_and_resolve_only_it(&mut g);
    // The dispatch suspends on Decision::OrderTriggers for the networked seat.
    assert!(matches!(
        g.pending_decision.as_ref().map(|pd| &pd.decision),
        Some(crabomination::decision::Decision::OrderTriggers { .. })
    ), "suspends for the ordering pick");
    g.submit_decision(DecisionAnswer::TriggerOrder(vec![w2, w1])).expect("order applied");
    assert_eq!(stack_trigger_sources(&g), vec![w2, w1],
        "controller's chosen push order is applied (w1 on top → resolves first)");
}

/// Cloudfin Raptor evolves (CR 702.100) when a creature with greater
/// power or toughness enters under your control.
#[test]
fn cloudfin_raptor_evolves_when_bigger_creature_enters() {
    let mut g = two_player_game();
    let raptor = g.add_card_to_battlefield(0, catalog::cloudfin_raptor());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // 2/2 > 0/1
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let r = view.iter().find(|c| c.id == raptor).unwrap();
    assert_eq!((r.power, r.toughness), (1, 2), "evolve added a +1/+1 counter");
}

/// Evolve does not trigger for a creature that isn't bigger (CR 702.100b).
#[test]
fn experiment_one_does_not_evolve_for_equal_creature() {
    let mut g = two_player_game();
    let exp = g.add_card_to_battlefield(0, catalog::experiment_one()); // 1/1
    // A second 1/1 is not greater in power or toughness → no evolve.
    let other = g.add_card_to_hand(0, catalog::experiment_one());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: other, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("second Experiment One castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let e = view.iter().find(|c| c.id == exp).unwrap();
    assert_eq!((e.power, e.toughness), (1, 1), "no evolve trigger for an equal creature");
}

/// Fathom Mage draws a card when it evolves (its CounterAdded trigger).
#[test]
fn fathom_mage_draws_when_it_evolves() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fathom_mage()); // 1/1
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // 2/2 > 1/1
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    // Bear left hand (-1); evolve→counter→draw refilled it (+1) → net even.
    assert_eq!(g.players[0].hand.len(), hand_before, "evolve counter drew a card");
}

/// Phyrexian Rager ETB: draw a card and lose 1 life.
#[test]
fn phyrexian_rager_etb_draws_and_loses_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::phyrexian_rager());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
    // Rager left hand (-1), drew a card (+1) → net even.
    assert_eq!(g.players[0].hand.len(), hand);
}

/// Carven Caryatid is a 0/5 Defender that draws on ETB.
#[test]
fn carven_caryatid_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::carven_caryatid());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand, "ETB draw refilled the cast");
    let c = g.battlefield.iter().find(|c| c.definition.name == "Carven Caryatid").unwrap();
    assert!(c.has_keyword(&crabomination::card::Keyword::Defender));
}

/// Doomed Traveler leaves a 1/1 flying Spirit when it dies.
#[test]
fn doomed_traveler_dies_into_a_spirit() {
    let mut g = two_player_game();
    let trav = g.add_card_to_battlefield(0, catalog::doomed_traveler());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    cast_at(&mut g, bolt, Target::Permanent(trav));
    assert!(!g.battlefield.iter().any(|c| c.id == trav), "traveler died");
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit").unwrap();
    assert!(spirit.definition.keywords.contains(&crabomination::card::Keyword::Flying));
}

/// Festering Goblin shrinks a creature when it dies.
#[test]
fn festering_goblin_dies_gives_minus_one() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::festering_goblin());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    cast_at(&mut g, bolt, Target::Permanent(gob));
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let b = view.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (1, 1), "bear got -1/-1 from the death trigger");
}

/// Spore Frog sacrifices itself to fog the turn.
#[test]
fn spore_frog_sacrifices_to_fog() {
    let mut g = two_player_game();
    let frog = g.add_card_to_battlefield(0, catalog::spore_frog());
    g.perform_action(GameAction::ActivateAbility {
        card_id: frog, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Spore Frog sac ability activatable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == frog), "frog sacrificed");
    assert!(g.prevent_combat_damage_this_turn, "combat damage prevented this turn");
}

/// Aven Fisher draws a card when it dies (opting in to the may-draw).
#[test]
fn aven_fisher_dies_draws_a_card() {
    let mut g = two_player_game();
    // Opt into the optional death-draw.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let fisher = g.add_card_to_battlefield(0, catalog::aven_fisher());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    cast_at(&mut g, bolt, Target::Permanent(fisher));
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fisher), "fisher died");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card on death");
}

/// Prodigal Pyromancer taps to ping any target for 1.
#[test]
fn prodigal_pyromancer_pings_for_one() {
    let mut g = two_player_game();
    let tim = g.add_card_to_battlefield(0, catalog::prodigal_pyromancer());
    g.clear_sickness(tim);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tim, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Tim ability activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "pinged for 1");
}

/// Gravedigger returns a creature card from your graveyard on ETB.
#[test]
fn gravedigger_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::gravedigger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead),
        "the dead bear returned to hand");
}

#[test]
fn essence_warden_is_a_green_soul_warden() {
    let d = catalog::essence_warden();
    assert_eq!(d.cost.cmc(), 1);
    assert!(d.triggered_abilities.iter().any(|t|
        t.event.kind == crabomination::card::EventKind::EntersBattlefield));
}

#[test]
fn suture_priest_drains_opponent_on_their_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::suture_priest());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent lost 1 from their creature entering");
}

#[test]
fn llanowar_visionary_draws_and_taps_for_green() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::llanowar_visionary());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "ETB drew a card (cast -1, draw +1)");
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("mana ability");
    assert!(g.players[0].mana_pool.amount(Color::Green) >= 1, "tapped for green");
}

#[test]
fn augur_of_bolas_digs_into_hand() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let id = g.add_card_to_hand(0, catalog::augur_of_bolas());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "ETB pulled a card (cast -1, dig +1)");
}

/// Augur only grabs an instant/sorcery; the rest go to the bottom of the library.
#[test]
fn augur_of_bolas_filters_to_spells_and_bottoms_rest() {
    let mut g = two_player_game();
    // Top of library: a creature (not grabbable), then an instant, then a land.
    g.add_card_to_library(0, catalog::forest());          // ends up bottom-most pushed last
    g.add_card_to_library(0, catalog::lightning_bolt());  // the only eligible pick
    g.add_card_to_library(0, catalog::grizzly_bears());   // top, not eligible
    let id = g.add_card_to_hand(0, catalog::augur_of_bolas());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "grabbed the instant");
    assert!(!g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the creature was not eligible to grab");
    // The two non-picked cards are now at the bottom of the library.
    let bottom2: Vec<&str> = g.players[0].library.iter().rev().take(2)
        .map(|c| c.definition.name).collect();
    assert!(bottom2.contains(&"Grizzly Bears") && bottom2.contains(&"Forest"),
        "the rest went to the bottom, got {bottom2:?}");
}

#[test]
fn pestermite_taps_a_permanent_on_etb() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::pestermite());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(land)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let tgt = g.battlefield.iter().find(|c| c.id == land).unwrap();
    assert!(tgt.tapped, "Pestermite tapped it");
    assert!(tgt.skip_next_untap, "the tapped permanent won't untap next untap step");
}

#[test]
fn knight_of_autumn_mode_zero_gains_four_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::knight_of_autumn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "default mode gains 4 life");
}

#[test]
fn flame_javelin_deals_four_to_a_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::flame_javelin());
    g.players[0].mana_pool.add(Color::Red, 3); // pay the {2/R} pips with red
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable with three red");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4);
}

#[test]
fn pongify_destroys_and_gives_owner_an_ape() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pongify());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable for {U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature destroyed");
    let ape = g.battlefield.iter().find(|c| c.definition.name == "Ape").expect("Ape made");
    assert_eq!(ape.controller, 1, "the Ape goes to the destroyed creature's controller");
}

#[test]
fn arc_trail_splits_two_and_one_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::arc_trail());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)], mode: None, x_value: None,
    }).expect("castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage to the player");
    // 1 damage marked on the 2-toughness bear (survives).
    assert_eq!(g.battlefield.iter().find(|c| c.id == bear).unwrap().damage, 1);
}

#[test]
fn history_of_benalia_saga_chapters_and_sacrifice() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::history_of_benalia());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {1}{W}{W}");
    drain_stack(&mut g);
    // Chapter I fired on ETB: 1 lore counter, 1 Knight.
    let saga = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(saga.counter_count(CounterType::Lore), 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Knight").count(), 1);
    // Chapter II (next precombat main): a second Knight.
    g.saga_advance(id);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Knight").count(), 2);
    // Chapter III: +2/+1 to Knights, then the Saga is sacrificed by SBA.
    g.saga_advance(id);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == id), "saga sacrificed after final chapter");
    let knight = g.battlefield.iter().find(|c| c.definition.name == "Knight").unwrap();
    assert_eq!(knight.power(), 4, "Knights got +2/+1 from chapter III");
}

#[test]
fn grapple_with_the_past_mills_then_returns_a_creature() {
    let mut g = two_player_game();
    // A creature in the graveyard for the return half.
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let id = g.add_card_to_hand(0, catalog::grapple_with_the_past());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Serra Angel"), "returned to hand");
    assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Plains").count() >= 3,
        "milled three cards");
}

#[test]
fn fumigate_destroys_creatures_and_gains_life_per_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fumigate());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.is_creature()), "all creatures destroyed");
    assert_eq!(g.players[0].life, life + 3, "gained 1 per destroyed creature (3)");
}

#[test]
fn gerrards_wisdom_gains_two_life_per_card_in_hand() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gerrards_wisdom());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::plains()); // hand = 3 incl. the spell
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // After casting, the spell left hand → 2 cards remain → gain 4.
    assert_eq!(g.players[0].life, life + 4, "2 life per remaining hand card");
}

#[test]
fn planar_cleansing_destroys_nonland_permanents_only() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(1, catalog::ghostly_prison());
    let land = g.add_card_to_battlefield(1, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::planar_cleansing());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == creature), "creature destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == ench), "enchantment destroyed");
    assert!(g.battlefield.iter().any(|c| c.id == land), "land survives");
}

#[test]
fn final_judgment_exiles_all_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::final_judgment());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mine || c.id == theirs), "all creatures gone");
    // Exiled, not in graveyard.
    assert!(g.exile.iter().any(|c| c.id == mine), "exiled, not destroyed");
}

#[test]
fn mind_spring_draws_x_cards() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::mind_spring());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3); // X=3
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("castable X=3");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 3, "drew X=3 (spent the spell)");
}

#[test]
fn fireball_deals_x_damage_to_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fireball());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4); // X=4
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: Some(4),
    }).expect("castable for X=4");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4, "X=4 damage");
}

#[test]
fn flame_sweep_hits_nonfliers_only() {
    let mut g = two_player_game();
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, dies
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer, survives
    let id = g.add_card_to_hand(0, catalog::flame_sweep());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ground), "non-flier took 2 and died");
    assert!(g.battlefield.iter().any(|c| c.id == flier), "flier untouched");
}

#[test]
fn lead_the_stampede_puts_creatures_from_top_five_into_hand() {
    let mut g = two_player_game();
    // Top of library: 2 creatures + a land among the top five.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::lead_the_stampede());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let creatures_before = g.players[0].hand.iter()
        .filter(|c| c.definition.is_creature()).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let creatures_after = g.players[0].hand.iter()
        .filter(|c| c.definition.is_creature()).count();
    assert_eq!(creatures_after, creatures_before + 2, "both creatures went to hand, land did not");
}

#[test]
fn reclaim_returns_graveyard_card_to_top_of_library() {
    let mut g = two_player_game();
    let card = g.add_card_to_graveyard(0, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::reclaim());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(card)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.last().is_some_and(|c| c.definition.name == "Serra Angel"),
        "card on top of library");
}

#[test]
fn ajanis_welcome_gains_life_on_your_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ajanis_welcome());
    let life0 = g.players[0].life;
    // Your creature entering (cast so ETB fires) triggers +1 life.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1, "gained 1 on your creature ETB");
}

#[test]
fn zombify_reanimates_from_your_graveyard() {
    let mut g = two_player_game();
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::zombify());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Serra Angel" && c.controller == 0),
        "reanimated under your control");
}

#[test]
fn pressure_point_taps_a_creature_and_cantrips() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pressure_point());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().find(|c| c.id == target).unwrap().tapped, "target tapped");
    assert_eq!(g.players[0].hand.len(), hand0, "net hand unchanged: spent the spell, drew one");
}

#[test]
fn seal_of_strength_sacrifices_to_pump() {
    let mut g = two_player_game();
    let seal = g.add_card_to_battlefield(0, catalog::seal_of_strength());
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.perform_action(GameAction::ActivateAbility {
        card_id: seal, ability_index: 0, target: Some(Target::Permanent(creature)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac to pump");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.id == creature).unwrap();
    assert_eq!((c.power(), c.toughness()), (5, 5), "+3/+3");
    assert!(!g.battlefield.iter().any(|c| c.id == seal), "Seal sacrificed");
}

#[test]
fn dark_prophecy_draws_and_loses_life_when_your_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dark_prophecy());
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
    let hand0 = g.players[0].hand.len();
    let life0 = g.players[0].life;
    // Kill the creature with a burn spell so the death event dispatches the
    // "another creature you control dies" watcher.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == creature), "bear died");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card on death");
    assert_eq!(g.players[0].life, life0 - 1, "lost 1 life on death");
}

#[test]
fn seal_of_doom_sacrifices_to_destroy_nonblack_creature() {
    let mut g = two_player_game();
    let seal = g.add_card_to_battlefield(0, catalog::seal_of_doom());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // white 4/4
    g.perform_action(GameAction::ActivateAbility {
        card_id: seal, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac to destroy");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "nonblack creature destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == seal), "Seal sacrificed as cost");
}

#[test]
fn felidar_sovereign_wins_at_forty_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::felidar_sovereign());
    // Below 40: upkeep trigger's intervening-if fails, nothing happens.
    g.players[0].life = 39;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!g.is_game_over(), "no win below 40 life");
    // At 40: the intervening-if win-con fires.
    g.players[0].life = 40;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.is_game_over(), "win at 40 life");
}

#[test]
fn the_birth_of_meletis_chapters_tutor_wall_and_gain_life() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed a basic Plains in the library so chapter I can find it.
    let plains = g.add_card_to_library(0, catalog::plains());
    // Pick the Plains when chapter I's search prompts.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(plains))]));
    let id = g.add_card_to_hand(0, catalog::the_birth_of_meletis());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Chapter I: a Plains is now in hand.
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), "tutored a Plains");
    // Chapter II: a 0/4 Wall token.
    g.saga_advance(id);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Wall"), "made a Wall");
    // Chapter III: gain 2 life.
    let life = g.players[0].life;
    g.saga_advance(id);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

#[test]
fn triumph_of_gerrard_pumps_then_buffs_greatest_power() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4, greatest power
    let _small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::triumph_of_gerrard());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {1}{W}");
    drain_stack(&mut g);
    // Chapter I put a +1/+1 counter on the greatest-power creature (Serra).
    assert_eq!(g.battlefield.iter().find(|c| c.id == big).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 1);
    // Chapter III: grants flying/first strike/lifelink to the greatest.
    g.saga_advance(id); // II
    drain_stack(&mut g);
    g.saga_advance(id); // III
    drain_stack(&mut g);
    let serra = g.battlefield.iter().find(|c| c.id == big).unwrap();
    assert!(serra.has_keyword(&crabomination::card::Keyword::Lifelink), "chapter III grants lifelink");
}

#[test]
fn the_eldest_reborn_chapter_three_reanimates_from_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::the_eldest_reborn());
    g.add_card_to_graveyard(0, catalog::serra_angel());
    g.saga_advance(id); drain_stack(&mut g); // I
    g.saga_advance(id); drain_stack(&mut g); // II
    g.saga_advance(id); drain_stack(&mut g); // III — reanimate target
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Serra Angel" && c.controller == 0),
        "chapter III reanimated the creature under your control");
}

#[test]
fn the_eldest_reborn_chapter_one_forces_opponent_sacrifice() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::the_eldest_reborn());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {4}{B}");
    drain_stack(&mut g);
    // Chapter I: opponent's only creature is sacrificed.
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "opponent sacrificed its creature");
}

#[test]
fn cone_of_flame_splits_one_two_three_across_three_targets() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::cone_of_flame());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear), Target::Permanent(big)],
        mode: None, x_value: None,
    }).expect("castable for {3}{R}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "1 damage to the player (slot 0)");
    // bear (slot 1) took 2 → dies; big (slot 2) took 3 → marked, survives.
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2 damage kills the 2/2");
    assert_eq!(g.battlefield.iter().find(|c| c.id == big).unwrap().damage, 3, "3 damage on the 4/4");
}

#[test]
fn prey_upon_makes_two_creatures_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::prey_upon());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("castable for {G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "the 2/2 died to the 4/4");
    // The 4/4 took 2 and survives.
    assert_eq!(g.battlefield.iter().find(|c| c.id == mine).unwrap().damage, 2);
}

#[test]
fn hedron_archive_taps_for_two_and_sacs_to_draw() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::hedron_archive());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    // Mana ability.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("mana ability");
    assert_eq!(g.players[0].mana_pool.total(), 2, "added two colorless");
    // Untap so the sac-to-draw ability (also a tap cost) is legal.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    // Sac-to-draw.
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac-draw ability");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
}

#[test]
fn spark_double_copies_with_an_extra_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 you control
    let id = g.add_card_to_hand(0, catalog::spark_double());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let dbl = g.battlefield.iter().find(|c| c.id == id).expect("on battlefield");
    assert_eq!(dbl.definition.name, "Grizzly Bears");
    assert_eq!(dbl.counter_count(CounterType::PlusOnePlusOne), 1,
        "Spark Double enters with the extra +1/+1 counter (via CR 707.5 copied trigger)");
}

#[test]
fn reflector_mage_bounces_an_opponent_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::reflector_mage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "opp creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "returned to owner's hand");
}

#[test]
fn reflector_mage_locks_the_bounced_creatures_name_until_your_next_turn() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::reflector_mage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);

    // The bounced creature's name is locked against its owner.
    assert!(
        g.players[0].opponents_cant_cast_named.iter().any(|n| n == "Grizzly Bears"),
        "Reflector Mage records the recast lock",
    );

    // Enforcement: on P1's turn, P1 can't recast the bounced Grizzly Bears.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    let recast = g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(
        matches!(recast, Err(crabomination::game::types::GameError::SpellNameLocked)),
        "the owner can't recast the locked name (got {recast:?})",
    );

    // The lock clears as the controller's (P0's) next turn begins.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(
        g.players[0].opponents_cant_cast_named.is_empty(),
        "the lock expires at your next turn",
    );
}

#[test]
fn man_o_war_bounces_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::man_o_war());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn siege_gang_commander_makes_three_goblins_and_pings() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::siege_gang_commander());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {3}{R}{R}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(), 3);
    // Sac a Goblin to ping the opponent for 2.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac-goblin ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "ping dealt 2");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(), 2,
        "one Goblin sacrificed as cost");
}

#[test]
fn walking_ballista_enters_with_x_counters_and_pings() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::walking_ballista());
    g.players[0].mana_pool.add_colorless(4); // X=2 → {X}{X} = 4 generic
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("castable for X=2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 2, "enters with X=2 counters");
    // Remove a counter to ping the opponent.
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("ping ability");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "ping dealt 1 damage");
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 1, "spent a counter");
}

#[test]
fn walking_ballista_counter_is_a_real_cost_not_overactivatable() {
    // CR 602.5b: the +1/+1 counter is paid when the ability is announced,
    // so a 1-counter Ballista can only announce one ping — not three off
    // the stack (the old in-effect RemoveCounter bug).
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::walking_ballista());
    g.players[0].mana_pool.add_colorless(2); // X=1 → one counter
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    }).expect("castable for X=1");
    drain_stack(&mut g);
    // First ping spends the only counter.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("first ping");
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 0, "counter paid as cost immediately");
    // Second activation must fail — no counter left to pay the cost.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "can't activate without a counter to remove");
}

#[test]
fn triskelion_enters_with_three_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::triskelion());
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {6}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn hangarback_walker_makes_thopters_when_it_dies() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::hangarback_walker());
    g.players[0].mana_pool.add_colorless(4); // X=2
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 2);
    // Kill it; the dies trigger makes one Thopter per +1/+1 counter.
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    let thopters = g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count();
    assert_eq!(thopters, 2, "two +1/+1 counters → two Thopters on death");
}

#[test]
fn frogmite_affinity_reduces_cost_per_artifact() {
    let mut g = two_player_game();
    // Three artifacts → Frogmite costs {4} - 3 = {1}.
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::ornithopter()); }
    let id = g.add_card_to_hand(0, catalog::frogmite());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Frogmite cast for {{1}} via Affinity");
}

#[test]
fn chief_of_the_foundry_buffs_other_artifact_creatures() {
    let mut g = two_player_game();
    let chief = g.add_card_to_battlefield(0, catalog::chief_of_the_foundry());
    let myr = g.add_card_to_battlefield(0, catalog::ornithopter()); // 0/2 artifact creature
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == myr).map(|c| (c.power, c.toughness)), Some((1, 3)),
        "other artifact creature gets +1/+1");
    // The Chief doesn't buff itself.
    assert_eq!(cp.iter().find(|c| c.id == chief).map(|c| (c.power, c.toughness)), Some((2, 3)));
}

#[test]
fn court_homunculus_grows_with_another_artifact() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::court_homunculus());
    // Alone: 1/1.
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((1, 1)));
    g.add_card_to_battlefield(0, catalog::ornithopter());
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((2, 2)),
        "+1/+1 with another artifact");
}

#[test]
fn cathodion_adds_three_colorless_when_it_dies() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cathodion());
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3, "dies → add {{C}}{{C}}{{C}}");
}

#[test]
fn bottle_gnomes_sacrifices_for_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::bottle_gnomes());
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sacrifice for life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
    assert!(g.battlefield_find(id).is_none(), "sacrificed");
}

#[test]
fn universal_automaton_is_every_creature_type() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::universal_automaton());
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    assert!(c.keywords.contains(&crabomination::card::Keyword::Changeling), "Changeling");
}

#[test]
fn sea_gate_oracle_etb_draws_into_hand() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::sea_gate_oracle());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Cast removed the card from hand (-1) but ETB put one of top two into hand (+1).
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB pulled a card into hand");
    // The unpicked card of the top two is on the bottom (CR — not left on top).
    let lib = g.players[0].library.len();
    assert!(lib >= 1, "library still has the bottomed card");
    assert_eq!(g.players[0].library[lib - 1].definition.name, "Island",
        "the unchosen card was put on the bottom");
}

#[test]
fn fertilid_trades_a_counter_to_fetch_a_basic() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let forest_in_lib = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::fertilid());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fertilid castable for {2}{G}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest_in_lib)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("ability activates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest_in_lib).is_some(),
        "fetched a basic onto the battlefield");
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 1, "spent a +1/+1 counter");
}

#[test]
fn grim_affliction_puts_a_minus_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::grim_affliction());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable for {2}{B}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counter_count(CounterType::MinusOneMinusOne), 1);
}

#[test]
fn throne_of_geth_sacrifices_to_proliferate() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let throne = g.add_card_to_battlefield(0, catalog::throne_of_geth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: throne, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activates");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == throne), "Throne sacrificed itself");
    assert_eq!(g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 2, "proliferated the +1/+1 counter");
}

#[test]
fn thrummingbird_proliferates_on_combat_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bird = g.add_card_to_battlefield(0, catalog::thrummingbird());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    // Fire the combat-damage trigger directly (full combat resolution is
    // covered by the combat suite); we exercise the proliferate payload.
    let trig = catalog::thrummingbird().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(bird, 0, None, 0);
    let _ = g.resolve_effect(&trig, &ctx);
    assert_eq!(g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 2,
        "combat damage to player proliferated your +1/+1 counter");
}

#[test]
fn karns_bastion_has_a_proliferate_ability() {
    let d = catalog::karns_bastion();
    assert!(d.card_types.contains(&CardType::Land));
    assert!(d.activated_abilities.iter().any(|a|
        matches!(a.effect, crabomination::effect::Effect::Proliferate)),
        "Karn's Bastion has a Proliferate activated ability");
}

#[test]
fn cr_702_135_mentor_counters_lesser_power_attacker() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let stalwart = g.add_card_to_battlefield(0, catalog::sunhome_stalwart()); // 2/1 Mentor
    let small = g.add_card_to_battlefield(0, catalog::elvish_visionary()); // 1/1
    g.clear_sickness(stalwart);
    g.clear_sickness(small);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: stalwart, target: AttackTarget::Player(1) },
        Attack { attacker: small, target: AttackTarget::Player(1) },
    ])).unwrap();
    drain_stack(&mut g);
    let n = g.battlefield.iter().find(|c| c.id == small).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(n, 1, "Mentor put a +1/+1 counter on the lesser-power attacker");
    // The mentor itself isn't a legal target (OtherThanSource + not lesser).
    assert_eq!(g.battlefield.iter().find(|c| c.id == stalwart).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn cr_707_5_clone_fires_copied_etb_trigger() {
    // Clone copying Elvish Visionary (ETB: draw a card) should itself draw.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::elvish_visionary()); // 1/1, ETB draw 1
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::clone_card());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Clone left hand (-1) and the copied ETB drew a card (+1): net even,
    // and the copy is a Visionary on the battlefield.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "copied Elvish Visionary ETB drew a card (CR 707.5)");
    assert_eq!(g.battlefield.iter().find(|c| c.id == id).unwrap().definition.name,
        "Elvish Visionary");
}

#[test]
fn cr_707_2_clone_copies_printed_pt_not_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 base
    // A +1/+1 counter on the original makes it a 3/3, but copiable values
    // (CR 707.2) are the printed characteristics — counters aren't copied.
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    let id = g.add_card_to_hand(0, catalog::clone_card());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let clone = g.battlefield.iter().find(|c| c.id == id).expect("on battlefield");
    assert_eq!((clone.definition.power, clone.definition.toughness), (2, 2),
        "copiable P/T excludes the original's +1/+1 counter");
    assert_eq!(clone.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 0,
        "counters are not part of copiable values (CR 707.2)");
}

#[test]
fn mirror_image_copies_only_your_own_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::serra_angel()); // opponent's — not copyable
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 yours
    let _ = mine;
    let id = g.add_card_to_hand(0, catalog::mirror_image());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mirror Image castable for {1}{U}");
    drain_stack(&mut g);
    let img = g.battlefield.iter().find(|c| c.id == id).expect("on battlefield");
    // Copies the controller's Grizzly Bears, never the opponent's Serra Angel.
    assert_eq!(img.definition.name, "Grizzly Bears");
}

/// CR 707.2e — Mirror Image's copy is not legendary, so it dodges the legend
/// rule and both it and the original survive.
#[test]
fn mirror_image_copy_is_not_legendary() {
    use crabomination::card::Supertype;
    let mut g = two_player_game();
    // A legendary creature to copy — without the "not legendary" rider the
    // copy would trigger the legend rule and one of them would die.
    let legend = g.add_card_to_battlefield(0, catalog::thalia_guardian_of_thraben());
    let id = g.add_card_to_hand(0, catalog::mirror_image());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let img = g.battlefield_find(id).expect("copy survives the legend rule");
    assert_eq!(img.definition.name, "Thalia, Guardian of Thraben", "copied the legend");
    assert!(!img.definition.supertypes.contains(&Supertype::Legendary),
        "the copy is not legendary (CR 707.2e)");
    assert!(g.battlefield_find(legend).is_some(), "the original legend also survives");
}

#[test]
fn stunt_double_keeps_flash_after_copying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 vanilla, no Flash
    let id = g.add_card_to_hand(0, catalog::stunt_double());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let dbl = g.battlefield.iter().find(|c| c.id == id).expect("on battlefield");
    assert_eq!(dbl.definition.name, "Grizzly Bears");
    assert!(dbl.definition.keywords.contains(&Keyword::Flash),
        "Stunt Double keeps Flash on the copy");
}

#[test]
fn cackling_counterpart_makes_a_token_copy() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 you control
    let id = g.add_card_to_hand(0, catalog::cackling_counterpart());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Serra Angel").count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable for {1}{U}{U}");
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Serra Angel").count();
    assert_eq!(after, before + 1, "a token copy of Serra Angel was created");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Serra Angel" && c.is_token));
}

#[test]
fn phantasmal_image_copies_and_keeps_illusion_plus_sacrifice_rider() {
    use crabomination::card::{CreatureType, EventKind};
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::phantasmal_image());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Phantasmal Image castable for {U}");
    drain_stack(&mut g);
    let img = g.battlefield.iter().find(|c| c.id == id).expect("on battlefield");
    assert_eq!(img.definition.name, "Serra Angel");
    assert_eq!((img.definition.power, img.definition.toughness), (4, 4));
    assert!(img.definition.subtypes.creature_types.contains(&CreatureType::Illusion),
        "Illusion in addition to copied types");
    assert!(img.definition.triggered_abilities.iter()
        .any(|t| t.event.kind == EventKind::BecameTarget),
        "gained 'when targeted, sacrifice it' rider");
}

#[test]
fn phantasmal_image_sacrifices_itself_when_targeted() {
    // The copy's "when this becomes the target of a spell or ability,
    // sacrifice it" rider now actually fires (BecameTarget SelfSource).
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::phantasmal_image());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Opponent bolts the Image — targeting it sacrifices it.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.cast_spell(bolt, Some(Target::Permanent(id)), vec![], None, None)
        .expect("Bolt targets the Image");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id),
        "Phantasmal Image sacrificed on becoming a target");
}

#[test]
fn phantasmal_image_refires_copied_etb_trigger() {
    // Copying Elvish Visionary (ETB: draw a card) re-fires the ETB per
    // CR 707.5 — the controller draws when the Image enters.
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::elvish_visionary());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::phantasmal_image());
    let hand_before = g.players[0].hand.len();
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {U}");
    drain_stack(&mut g);
    // Cast the Image (−1 from hand) then drew 1 from the copied ETB → net 0.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "copied Elvish Visionary ETB draw re-fires (cast −1, draw +1)");
}

#[test]
fn mockingbird_copies_your_creature_but_keeps_its_name() {
    let mut g = two_player_game();
    // A creature you control to copy.
    g.add_card_to_battlefield(0, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::mockingbird());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mockingbird castable for {1}{U}");
    drain_stack(&mut g);
    let mb = g.battlefield.iter().find(|c| c.id == id).expect("on battlefield");
    // Copies the body (4/4 flier) but keeps its own name (CR 707.2).
    assert_eq!((mb.definition.power, mb.definition.toughness), (4, 4));
    assert!(mb.definition.keywords.contains(&Keyword::Flying), "copied Flying");
    assert_eq!(mb.definition.name, "Mockingbird", "name retained per CR 707.2");
}

#[test]
fn culling_ritual_destroys_small_nonland_permanents() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::culling_ritual());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Culling Ritual castable for {2}{B}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "MV-2 nonland creature destroyed by Culling Ritual");
}

#[test]
fn culling_ritual_adds_mana_per_permanent_destroyed() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    g.add_card_to_battlefield(1, catalog::llanowar_elves()); // MV 1
    let id = g.add_card_to_hand(0, catalog::culling_ritual());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Cost was fully paid, so the pool now holds only the rider's mana:
    // one B/G per destroyed permanent (two destroyed → two mana).
    assert_eq!(g.players[0].mana_pool.total(), 2,
        "two permanents destroyed → two B/G mana added");
}

#[test]
fn rushed_rebirth_fetches_creature_to_battlefield_when_target_dies() {
    let mut g = two_player_game();
    let elf = g.add_card_to_library(0, catalog::llanowar_elves());
    // The creature we target — a 2/2 that will die to a follow-up.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(elf))]));
    let id = g.add_card_to_hand(0, catalog::rushed_rebirth());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rushed Rebirth castable for {B}{G}");
    drain_stack(&mut g);
    // No fetch yet — the watched creature is still alive.
    assert!(!g.battlefield.iter().any(|c| c.id == elf), "no fetch before the target dies");

    // Bolt the bear; its death fires Rushed Rebirth's watch.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let fetched = g.battlefield.iter().find(|c| c.id == elf);
    assert!(fetched.is_some(), "creature fetched onto the battlefield on death");
    assert!(fetched.unwrap().tapped, "fetched creature enters tapped");
}

#[test]
fn rushed_rebirth_lesser_mana_value_filter_blocks_costlier_creature() {
    let mut g = two_player_game();
    // Only fetch candidate is Grizzly Bears (MV 2); the watched creature is
    // a 1-drop, so "lesser mana value" leaves nothing to fetch.
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let id = g.add_card_to_hand(0, catalog::rushed_rebirth());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(elf)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rushed Rebirth castable");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(elf)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "MV-2 Grizzly Bears can't be fetched when the MV-1 watched creature dies");
}

#[test]
fn callous_bloodmage_etb_makes_a_pest_token() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let id = g.add_card_to_hand(0, catalog::callous_bloodmage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Callous Bloodmage castable for {2}{B}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"),
        "ETB mode 0 mints a Pest token");
}

#[test]
fn mesmeric_orb_mills_each_player_on_upkeep() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mesmeric_orb());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let gy0 = g.players[0].graveyard.len();
    let gy1 = g.players[1].graveyard.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy0 + 3, "P0 mills 3");
    assert_eq!(g.players[1].graveyard.len(), gy1 + 3, "P1 mills 3");
}

#[test]
fn swords_to_plowshares_exiles_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::swords_to_plowshares());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swords castable for {W}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled by Swords");
}

#[test]
fn hymn_to_tourach_discards_two_at_random() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::hymn_to_tourach());
    g.players[0].mana_pool.add(Color::Black, 2);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hymn castable for {B}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 2, "target discards two cards");
}

#[test]
fn baleful_strix_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::baleful_strix());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Baleful Strix castable for {U}{B}");
    drain_stack(&mut g);
    // Cast (-1 from hand) + ETB draw (+1) = net unchanged; body on battlefield.
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB draw offsets the cast");
    assert!(g.battlefield.iter().any(|c| c.id == id), "Strix entered the battlefield");
}

#[test]
fn armageddon_destroys_all_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::armageddon());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Armageddon castable for {2}{W}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.is_land()), "all lands destroyed");
}

#[test]
fn mox_sapphire_taps_for_blue() {
    let mut g = two_player_game();
    let mox = g.add_card_to_battlefield(0, catalog::mox_sapphire());
    g.clear_sickness(mox);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Mox Sapphire mana ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "Mox Sapphire adds blue mana");
}

#[test]
fn planar_nexus_taps_for_any_color() {
    let mut g = two_player_game();
    let nexus = g.add_card_to_battlefield(0, catalog::planar_nexus());
    g.clear_sickness(nexus);
    g.perform_action(GameAction::ActivateAbility {
        card_id: nexus, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Planar Nexus mana ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "Planar Nexus adds one mana of any color");
}

#[test]
fn trinisphere_floors_cheap_spells_at_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::trinisphere());
    let id = g.add_card_to_hand(0, catalog::ponder());
    // {U} alone is short of the {3} floor.
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "Ponder can't be paid for under Trinisphere with only {{U}}");
    // Top up to three total; now it pays.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ponder castable once three mana are available");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "all three mana consumed");
}

#[test]
fn trinisphere_does_not_tax_when_tapped() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let tri = g.add_card_to_battlefield(0, catalog::trinisphere());
    g.battlefield.iter_mut().find(|c| c.id == tri).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::ponder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("tapped Trinisphere imposes no floor");
    drain_stack(&mut g);
}

#[test]
fn ravens_crime_retrace_recasts_from_graveyard_for_a_land() {
    let mut g = two_player_game();
    // Opponent has two cards to discard.
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    // Put Raven's Crime in the graveyard and a land in hand for the cost.
    let crime = g.add_card_to_graveyard(0, catalog::ravens_crime());
    g.add_card_to_hand(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Black, 1);
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastRetrace {
        card_id: crime, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Raven's Crime retraces by discarding a land");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opponent discarded a card");
    assert!(!g.players[0].hand.iter().any(|c| c.definition.is_land()), "land discarded as cost");
    // Retrace returns the spell to the graveyard (not exile) — recastable.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Raven's Crime"),
        "Raven's Crime back in graveyard for another retrace");
}

#[test]
fn ravens_crime_retrace_requires_a_land_in_hand() {
    let mut g = two_player_game();
    let crime = g.add_card_to_graveyard(0, catalog::ravens_crime());
    g.players[0].mana_pool.add(Color::Black, 1);
    // No land in hand → retrace rejected, mana untouched.
    assert!(g.perform_action(GameAction::CastRetrace {
        card_id: crime, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "retrace needs a land to discard");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1, "mana not spent on failed retrace");
}

