//! Functionality tests for the MH3 batch-4 cards in `catalog::sets::mh3d`.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
}

fn cast(g: &mut GameState, id: crate::card::CardId, target: Option<Target>) {
    fill_mana(g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(g);
}

/// Ugin's Binding returns a nonland permanent an opponent controls to hand.
#[test]
fn ugins_binding_bounces_opponent_permanent() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::ugins_binding());
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced to owner's hand");
}

/// Abstruse Appropriation exiles a nonland permanent and grants a cast
/// permission that lasts as long as it stays exiled.
#[test]
fn abstruse_appropriation_exiles_and_grants_recast() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::abstruse_appropriation());
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("bear exiled");
    assert!(exiled.may_play_until.is_some(), "granted a cast permission");
}

/// Expel the Unworthy exiles a small creature; its controller gains life equal
/// to its mana value (Grizzly Bears = MV 2).
#[test]
fn expel_the_unworthy_exiles_small_and_gains_life() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::expel_the_unworthy());
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
    assert_eq!(g.players[1].life, life + 2, "controller gains life = mana value");
}

/// Kicked, Expel the Unworthy can exile a creature of any mana value.
#[test]
fn expel_the_unworthy_kicked_hits_large_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5
    let life = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::expel_the_unworthy());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == angel), "large creature exiled when kicked");
    assert_eq!(g.players[1].life, life + 5, "controller gains life = mana value 5");
}

/// Twisted Riddlekeeper's cast trigger taps two permanents and stuns each.
#[test]
fn twisted_riddlekeeper_taps_and_stuns_two() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::twisted_riddlekeeper());
    fill_mana(&mut g);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(a)),
        DecisionAnswer::Target(Target::Permanent(b)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("hardcast");
    drain_stack(&mut g);
    for id in [a, b] {
        let p = g.battlefield_find(id).unwrap();
        assert!(p.tapped, "target tapped");
        assert_eq!(p.counter_count(CounterType::Stun), 1, "stun counter added");
    }
}

/// Depth Defiler cast unkicked resolves a single chosen mode (bounce).
#[test]
fn depth_defiler_unkicked_bounces_one_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::depth_defiler());
    fill_mana(&mut g);
    // Mode 0 = bounce, then target the only creature.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Mode(0),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast unkicked");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "unkicked bounce mode");
}

/// Kicked, Depth Defiler performs both modes: bounce and draw-two-discard-one.
#[test]
fn depth_defiler_kicked_does_both() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let junk = g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let spell = g.add_card_to_hand(0, catalog::depth_defiler());
    fill_mana(&mut g);
    // The bounce leg auto-targets the only creature; only the discard needs a
    // scripted answer.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![junk])]));
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "kicked bounce leg");
    // Started with hand_before (incl. junk), removed Depth Defiler on cast,
    // drew 2, discarded 1 → net +1 over the post-cast hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "drew two, discarded one");
}

/// Dog Umbra's umbra armor redirects a lethal damage marking to itself.
#[test]
fn dog_umbra_saves_enchanted_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let umbra = g.add_card_to_hand(0, catalog::dog_umbra());
    cast(&mut g, umbra, Some(Target::Permanent(bear)));
    g.battlefield_find_mut(bear).unwrap().damage = 5;
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_some(), "creature saved by umbra armor");
    assert!(g.battlefield_find(umbra).is_none(), "aura destroyed instead");
}

/// Thief of Existence exiles an opponent's small noncreature permanent and
/// gains a leaves-the-battlefield draw trigger.
#[test]
fn thief_of_existence_exiles_and_grants_ltb_draw() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone()); // MV 2 artifact
    g.add_card_to_library(0, catalog::grizzly_bears());
    let thief = g.add_card_to_hand(0, catalog::thief_of_existence());
    fill_mana(&mut g);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(stone)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: thief, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == stone), "opponent artifact exiled");
    let hand_before = g.players[0].hand.len();
    // Kill the Thief (3/4) with lethal damage; its granted LTB trigger draws.
    g.battlefield_find_mut(thief).unwrap().damage = 4;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "LTB trigger drew a card");
}

/// Depth Charge Colossus enters 9/9 and, once tapped, stays tapped through the
/// untap step but untaps for its {3} ability.
#[test]
fn depth_charge_colossus_doesnt_untap_but_pays_to_untap() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::depth_charge_colossus());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (9, 9), "printed 9/9");
    g.battlefield_find_mut(id).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(id).unwrap().tapped, "stays tapped through untap step");
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("untap ability");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(id).unwrap().tapped, "paid untap ability untapped it");
}

/// Amphibian Downpour turns the enchanted creature into a 1/1 blue Frog with no
/// abilities.
#[test]
fn amphibian_downpour_makes_a_vanilla_frog() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying vigilance
    let aura = g.add_card_to_hand(0, catalog::amphibian_downpour());
    cast(&mut g, aura, Some(Target::Permanent(angel)));
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "base 1/1");
    assert!(cp.keywords.is_empty(), "lost all abilities");
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Frog), "is a Frog");
}

/// Herigast's cast trigger wheels your hand into three fresh cards.
#[test]
fn herigast_wheels_hand_on_cast() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::serra_angel());
    }
    let herigast = g.add_card_to_hand(0, catalog::herigast_erupting_nullkite());
    // MayDo defaults to "no" under AutoDecider — script the yes.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: herigast, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("hardcast");
    drain_stack(&mut g);
    // Two grizzlies exiled; three angels drawn.
    assert_eq!(g.players[0].hand.len(), 3, "wheeled to three fresh cards");
    assert!(g.players[0].hand.iter().all(|c| c.definition.name == "Serra Angel"), "drew from library");
}

/// Ondu Knotmaster grows when another modified creature you control dies.
#[test]
fn ondu_knotmaster_grows_on_modified_death() {
    let mut g = two_player_game();
    let knot = g.add_card_to_battlefield(0, catalog::ondu_knotmaster());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Modify the ally with a +1/+1 counter, then kill it.
    g.battlefield_find_mut(ally).unwrap().add_counters(crate::card::CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(ally).unwrap().damage = 3;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(knot).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 + two +1/+1 counters");
}

/// Throw a Line (Ondu Knotmaster's adventure) distributes two +1/+1 counters.
#[test]
fn throw_a_line_distributes_two_counters() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let knot = g.add_card_to_hand(0, catalog::ondu_knotmaster());
    fill_mana(&mut g);
    // Cast the adventure half (mode = Some(1) selects the adventure).
    g.perform_action(GameAction::CastAdventure {
        card_id: knot, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Throw a Line");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 + two counters on one target");
}

/// Hydroelectric Specimen is a 1/4 flash Weird with a mono-U land back.
#[test]
fn hydroelectric_specimen_stats_and_back_face() {
    let d = catalog::hydroelectric_specimen();
    assert_eq!((d.power, d.toughness), (1, 4));
    assert!(d.keywords.contains(&Keyword::Flash));
    let back = d.back_face.expect("has a land back");
    assert!(back.card_types.contains(&crate::card::CardType::Land), "back is a land");
}

/// Eladamri lets you cast a creature spell off the top of your library.
#[test]
fn eladamri_casts_creature_from_top_of_library() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::eladamri_korvecdal());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    // Grizzly Bears is now the top card; Eladamri lets it be cast from there.
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: top, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast creature off the top of library");
    drain_stack(&mut g);
    assert!(g.battlefield_find(top).is_some(), "the top creature resolved onto the battlefield");
}

/// Party Thrasher's first-main trigger digs two after a discard.
#[test]
fn party_thrasher_digs_two_after_discard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::party_thrasher());
    let pitch = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::serra_angel());
    }
    // Advance to (re-enter) the precombat main so the trigger fires; accept the
    // "may" and discard the pitch card.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Discard(vec![pitch]),
    ]));
    g.step = TurnStep::Upkeep;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("advance");
    }
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "discarded the pitch card");
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Serra Angel").count(), 2, "exiled top two");
}

/// Suppression Ray taps every creature the target player controls.
#[test]
fn suppression_ray_taps_all_target_players_creatures() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::suppression_ray());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped,
        "target player's creatures tapped");
    assert!(!g.battlefield_find(mine).unwrap().tapped, "your own creatures untouched");
}

/// Bloodsoaked Insight exiles the target opponent's top three cards and lets you
/// play them.
#[test]
fn bloodsoaked_insight_exiles_opponent_top_three() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::bloodsoaked_insight());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.owner == 1).count(), 3, "opponent's top three exiled");
    assert!(g.exile.iter().filter(|c| c.owner == 1).all(|c| c.may_play_until.is_some()),
        "you may play them");
}

/// Collective Resistance's base mode destroys a target artifact.
#[test]
fn collective_resistance_destroys_artifact() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let art = g.add_card_to_battlefield(1, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::collective_resistance());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(art)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast destroy-artifact mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Collective Resistance's protection mode grants hexproof and indestructible.
#[test]
fn collective_resistance_grants_protection() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::collective_resistance());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("cast protection mode");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Hexproof) && cp.keywords.contains(&Keyword::Indestructible),
        "gained hexproof + indestructible");
}
