//! Functionality tests for the `catalog::sets::decks::recent4` batch.

use crabomination::catalog;
use crabomination::game::actions::extra_cost_for_spell;
use crabomination::game::types::TurnStep;
use crabomination::game::*;
use crabomination::mana::Color;

/// Ritual of Soot destroys creatures with mana value 3 or less, sparing bigger.
#[test]
fn ritual_of_soot_kills_small_creatures_only() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let big = g.add_card_to_battlefield(1, catalog::grave_titan()); // MV 6
    let id = g.add_card_to_hand(0, catalog::ritual_of_soot());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ritual of Soot");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "MV2 creature destroyed");
    assert!(g.battlefield_find(big).is_some(), "MV6 creature survives");
}

/// Recurring Nightmare: sac a creature + bounce itself to reanimate a graveyard
/// creature.
#[test]
fn recurring_nightmare_reanimates_sacrificing_a_creature() {
    let mut g = two_player_game();
    let nightmare = g.add_card_to_battlefield(0, catalog::recurring_nightmare());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
    g.perform_action(GameAction::ActivateAbility {
        card_id: nightmare, ability_index: 0,
        target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("activate Recurring Nightmare");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "graveyard creature reanimated");
    assert!(g.battlefield_find(fodder).is_none(), "fodder creature sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == nightmare),
        "Recurring Nightmare returned to its owner's hand");
}

/// Survival of the Fittest: pay {G} + discard a creature → tutor a creature to
/// hand.
#[test]
fn survival_of_the_fittest_tutors_a_creature() {
    let mut g = two_player_game();
    let survival = g.add_card_to_battlefield(0, catalog::survival_of_the_fittest());
    let discard = g.add_card_to_hand(0, catalog::grizzly_bears());
    let fetch = g.add_card_to_library(0, catalog::grave_titan());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(fetch)),
    ]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: survival, ability_index: 0,
        target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Survival of the Fittest");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fetch),
        "tutored creature is in hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == discard),
        "discarded creature is in the graveyard");
}

/// Footsteps of the Goryo reanimates a graveyard creature, then sacrifices it
/// at the next end step.
#[test]
fn footsteps_of_the_goryo_reanimates_then_sacrifices() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
    let id = g.add_card_to_hand(0, catalog::footsteps_of_the_goryo());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Footsteps of the Goryo");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
    // Walk to the end step — the delayed trigger sacrifices it.
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_none(), "reanimated creature sacrificed at end step");
}

/// Apprentice Necromancer: {B}, {T}, sac itself → reanimate a graveyard creature
/// with haste; sacrifice it at the next end step.
#[test]
fn apprentice_necromancer_reanimates_with_haste() {
    let mut g = two_player_game();
    let appr = g.add_card_to_battlefield(0, catalog::apprentice_necromancer());
    g.clear_sickness(appr);
    let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: appr, ability_index: 0,
        target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("activate Apprentice Necromancer");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
    assert!(g.computed_permanent(dead).unwrap().keywords.contains(&crabomination::card::Keyword::Haste),
        "reanimated creature has haste");
    assert!(g.battlefield_find(appr).is_none(), "Apprentice Necromancer sacrificed itself");
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_none(), "reanimated creature sacrificed at end step");
}

/// Deafening Silence: a second noncreature spell can't be cast; creature spells
/// are unaffected.
#[test]
fn deafening_silence_limits_noncreature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::deafening_silence());
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("first noncreature spell ok");
    drain_stack(&mut g);
    let second = g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(second, Err(GameError::SpellLimitReached)),
        "second noncreature spell blocked, got {second:?}");
    // A creature spell is still castable.
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("creature spell still castable");
}

/// Ethersworn Canonist: a second nonartifact spell can't be cast; artifact
/// spells are unaffected.
#[test]
fn ethersworn_canonist_limits_nonartifact_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ethersworn_canonist());
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let sol = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("first nonartifact spell ok");
    drain_stack(&mut g);
    let second = g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(second, Err(GameError::SpellLimitReached)),
        "second nonartifact spell blocked, got {second:?}");
    g.perform_action(GameAction::CastSpell {
        card_id: sol, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("artifact spell still castable");
}

/// Defense Grid taxes spells {3} unless cast during the caster's own turn.
#[test]
fn defense_grid_taxes_off_turn_spells() {
    let mut g = two_player_game(); // player 0 active
    g.add_card_to_battlefield(0, catalog::defense_grid());
    let id = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt = g.players[1].hand.iter().find(|c| c.id == id).unwrap().clone();
    assert_eq!(extra_cost_for_spell(&g, 1, &bolt, None, 0), 3,
        "opponent's spell taxed {{3}} on the active player's turn");
    assert_eq!(extra_cost_for_spell(&g, 0, &bolt, None, 0), 0,
        "active player's own spell untaxed");
}

/// Bontu's Last Reckoning destroys all creatures and keeps the caster's lands
/// from untapping next untap step.
#[test]
fn bontus_last_reckoning_wipes_board_and_locks_lands() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::bontus_last_reckoning());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bontu's Last Reckoning");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "all creatures destroyed");
    // Tap the land and run the untap step — it must stay tapped (charge spent).
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped, "land stays tapped after Bontu's");
    // The lock is one-shot: the following untap step untaps normally.
    g.do_untap();
    assert!(!g.battlefield_find(land).unwrap().tapped, "land untaps the next step");
}

/// Syphon Mind makes each opponent discard and draws you one per discard.
#[test]
fn syphon_mind_discards_each_opponent_and_draws() {
    let mut g = crabomination::game::multi_player_game(3);
    for p in 1..3 {
        g.add_card_to_hand(p, catalog::forest());
    }
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::syphon_mind());
    let hand0 = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Syphon Mind");
    drain_stack(&mut g);
    // Two opponents each discard one; caster draws two (loses Syphon Mind, +2).
    assert_eq!(g.players[1].hand.len(), 0, "opponent 1 discarded");
    assert_eq!(g.players[2].hand.len(), 0, "opponent 2 discarded");
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "caster drew one per discard");
}

/// Prosperity makes each player draw X.
#[test]
fn prosperity_each_player_draws_x() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::prosperity());
    let h0 = g.players[0].hand.len();
    let h1 = g.players[1].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Prosperity for X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 2, "caster drew 2 (lost Prosperity)");
    assert_eq!(g.players[1].hand.len(), h1 + 2, "opponent drew 2");
}

/// Ondu Giant fetches a basic land onto the battlefield tapped.
#[test]
fn ondu_giant_etb_fetches_basic_tapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::ondu_giant());
    drain_stack(&mut g);
    let fetched = g.battlefield_find(forest).expect("basic land fetched to battlefield");
    assert!(fetched.tapped, "fetched land enters tapped");
}

/// Roiling Regrowth sacrifices a land to fetch up to two basics tapped.
#[test]
fn roiling_regrowth_sacrifices_land_for_two_basics() {
    let mut g = two_player_game();
    let sacland = g.add_card_to_battlefield(0, catalog::mountain());
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(f1)),
        crabomination::decision::DecisionAnswer::Search(Some(f2)),
    ]));
    let id = g.add_card_to_hand(0, catalog::roiling_regrowth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Roiling Regrowth");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sacland).is_none(), "a land was sacrificed");
    assert!(g.battlefield_find(f1).is_some_and(|c| c.tapped), "first basic on battlefield tapped");
    assert!(g.battlefield_find(f2).is_some_and(|c| c.tapped), "second basic on battlefield tapped");
}

/// Roar of the Wurm makes a 6/6 Wurm and can be flashed back from the graveyard.
#[test]
fn roar_of_the_wurm_token_and_flashback() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::roar_of_the_wurm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Roar of the Wurm");
    drain_stack(&mut g);
    let wurms = g.battlefield.iter().filter(|c| c.definition.name == "Wurm").count();
    assert_eq!(wurms, 1, "one 6/6 Wurm token");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Roar is in the graveyard");
    // Flashback it for {3}{G}.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flashback Roar of the Wurm");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Wurm").count(), 2,
        "flashback made a second Wurm");
}

/// Chart a Course draws two and only forces a discard when you haven't attacked.
#[test]
fn chart_a_course_discards_unless_attacked() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::chart_a_course());
    let h = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chart a Course");
    drain_stack(&mut g);
    // No attack this turn → draw 2, then discard 1 → net +1, minus the spell.
    assert_eq!(g.players[0].hand.len(), h - 1 + 2 - 1, "drew two, discarded one (no attack)");
}

/// Living Death swaps graveyard creatures for battlefield creatures.
#[test]
fn living_death_swaps_graveyards_and_battlefields() {
    let mut g = two_player_game();
    // Player 0: a creature in play, a creature in the graveyard.
    let in_play = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let in_grave = g.add_card_to_graveyard(0, catalog::grave_titan());
    let id = g.add_card_to_hand(0, catalog::living_death());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Living Death");
    drain_stack(&mut g);
    assert!(g.battlefield_find(in_play).is_none(), "battlefield creature sacrificed");
    assert!(g.battlefield_find(in_grave).is_some(), "graveyard creature reanimated");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == in_play),
        "the sacrificed creature is in the graveyard");
}

/// Show and Tell lets the caster put a permanent from hand onto the battlefield.
#[test]
fn show_and_tell_puts_permanent_from_hand() {
    let mut g = two_player_game();
    let titan = g.add_card_to_hand(0, catalog::grave_titan());
    let id = g.add_card_to_hand(0, catalog::show_and_tell());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Show and Tell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(titan).is_some(), "the highest-mana permanent entered the battlefield");
    assert!(!g.players[0].hand.iter().any(|c| c.id == titan), "it left hand");
}

/// Sylvan Tutor puts a creature card on top of the library.
#[test]
fn sylvan_tutor_tops_a_creature() {
    let mut g = two_player_game();
    let titan = g.add_card_to_library(0, catalog::grave_titan());
    g.add_card_to_library(0, catalog::island()); // a non-creature to leave behind
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(titan)),
    ]));
    let id = g.add_card_to_hand(0, catalog::sylvan_tutor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sylvan Tutor");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(titan),
        "tutored creature is on top of the library");
}

/// Final Parting puts one card to hand and another to graveyard.
#[test]
fn final_parting_splits_two_cards() {
    let mut g = two_player_game();
    let to_hand = g.add_card_to_library(0, catalog::grave_titan());
    let to_grave = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(to_hand)),
        crabomination::decision::DecisionAnswer::Search(Some(to_grave)),
    ]));
    let id = g.add_card_to_hand(0, catalog::final_parting());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Final Parting");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == to_hand), "first card to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == to_grave), "second card to graveyard");
}

/// Altar's Reap sacrifices a creature and draws two.
#[test]
fn altars_reap_sacrifices_and_draws() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::altars_reap());
    let h = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Altar's Reap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert_eq!(g.players[0].hand.len(), h - 1 + 2, "drew two cards");
}

/// Corpse Knight drains each opponent when another creature you control enters.
#[test]
fn corpse_knight_drains_on_creature_entry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::corpse_knight());
    let life = g.players[1].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent lost 1 when a creature entered");
}

/// Harvester of Souls draws when another nontoken creature dies.
#[test]
fn harvester_of_souls_draws_on_creature_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::harvester_of_souls());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    // "you may draw" — accept the optional trigger.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let h = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "bear died");
    assert_eq!(g.players[0].hand.len(), h + 1, "Harvester drew a card on the death");
}

/// Snap returns a creature to hand and untaps up to two lands.
#[test]
fn snap_bounces_creature_and_untaps_lands() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let l1 = g.add_card_to_battlefield(0, catalog::island());
    let l2 = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(l1).unwrap().tapped = true;
    g.battlefield_find_mut(l2).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::snap());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Snap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature bounced");
    assert!(!g.battlefield_find(l1).unwrap().tapped && !g.battlefield_find(l2).unwrap().tapped,
        "two lands untapped");
}

/// Throttle gives -4/-4, killing a small creature.
#[test]
fn throttle_shrinks_and_kills() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::throttle());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Throttle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 dies to -4/-4");
}

/// Trophy Mage tutors an artifact with mana value 3.
#[test]
fn trophy_mage_tutors_mv3_artifact() {
    let mut g = two_player_game();
    let rock = g.add_card_to_library(0, catalog::darksteel_ingot()); // MV 3 artifact
    g.add_card_to_library(0, catalog::sol_ring()); // MV 1 — should be ineligible
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(rock)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::trophy_mage());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == rock), "MV3 artifact tutored to hand");
}

/// Thirst for Knowledge draws three then discards two.
#[test]
fn thirst_for_knowledge_draws_three_discards_two() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::thirst_for_knowledge());
    let h = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Thirst for Knowledge");
    drain_stack(&mut g);
    // -1 (the spell) +3 drawn -2 discarded = net 0.
    assert_eq!(g.players[0].hand.len(), h - 1 + 3 - 2, "drew three, discarded two");
}

/// Kavu Predator grows when an opponent gains life.
#[test]
fn kavu_predator_grows_on_opponent_lifegain() {
    let mut g = two_player_game();
    let kavu = g.add_card_to_battlefield(0, catalog::kavu_predator());
    // The opponent gains 3 life — Kavu's controller's opponent.
    g.adjust_life(1, 3);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 1, amount: 3 }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(kavu).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        3, "Kavu Predator gained three +1/+1 counters");
}

/// Seal Away exiles a tapped creature until it leaves; the creature returns
/// when Seal Away is destroyed.
#[test]
fn seal_away_exiles_tapped_creature_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::seal_away());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Seal Away");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "tapped creature exiled");
    // Destroy Seal Away → the creature returns.
    let seal = g.battlefield.iter().find(|c| c.definition.name == "Seal Away").unwrap().id;
    let bolt = g.add_card_to_hand(0, catalog::disenchant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(seal)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("destroy Seal Away");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "creature returns when Seal Away leaves");
}

/// Conclave Tribunal exiles a nonland permanent an opponent controls.
#[test]
fn conclave_tribunal_exiles_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::conclave_tribunal());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Conclave Tribunal");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent's creature exiled");
}

/// Fiery Cannonade deals 2 to each non-Pirate creature.
#[test]
fn fiery_cannonade_spares_pirates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, not a Pirate
    let id = g.add_card_to_hand(0, catalog::fiery_cannonade());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fiery Cannonade");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 non-Pirate dies to 2 damage");
}

/// Magmaquake deals X to each non-flying creature, sparing flyers.
#[test]
fn magmaquake_spares_flyers() {
    let mut g = two_player_game();
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 no flying
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let id = g.add_card_to_hand(0, catalog::magmaquake());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast Magmaquake for X=3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ground).is_none(), "ground creature dies to 3");
    assert!(g.battlefield_find(flyer).is_some(), "flyer untouched");
}

/// Star of Extinction destroys a land and wipes the board with 20 damage.
#[test]
fn star_of_extinction_destroys_land_and_wipes() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let titan = g.add_card_to_battlefield(1, catalog::grave_titan()); // 6/6
    let id = g.add_card_to_hand(0, catalog::star_of_extinction());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Star of Extinction");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "target land destroyed");
    assert!(g.battlefield_find(titan).is_none(), "6/6 wiped by 20 damage");
}

/// Pit Fight makes your creature fight an opponent's; deathtouch-free trade.
#[test]
fn pit_fight_resolves_a_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grave_titan()); // 6/6
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::pit_fight());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Pit Fight");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "2/2 dies to the 6/6");
    assert!(g.battlefield_find(mine).is_some(), "6/6 survives 2 damage");
}

/// Hunt the Weak grows your creature, then it fights.
#[test]
fn hunt_the_weak_buffs_then_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::hunt_the_weak());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Hunt the Weak");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "their 2/2 dies to the buffed 3/3");
    assert!(g.battlefield_find(mine).is_some(), "your 3/3 survives 2 damage");
}

/// Bramblecrush destroys a noncreature permanent (a land).
#[test]
fn bramblecrush_destroys_noncreature() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::bramblecrush());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bramblecrush");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "noncreature permanent destroyed");
}

/// Creeping Corrosion destroys all artifacts.
#[test]
fn creeping_corrosion_wipes_artifacts() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::creeping_corrosion());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Creeping Corrosion");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "artifact destroyed");
    assert!(g.battlefield_find(bear).is_some(), "creature untouched");
}

/// Devour Flesh makes the target player sacrifice a creature and gain its
/// toughness in life.
#[test]
fn devour_flesh_edicts_and_grants_toughness_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grave_titan()); // 6/6 — only creature
    let life = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::devour_flesh());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Devour Flesh");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.controller == 1 && c.definition.is_creature()),
        "opponent sacrificed their creature");
    assert_eq!(g.players[1].life, life + 6, "opponent gained 6 life (the 6/6's toughness)");
}

/// Mudbutton Torchrunner deals 3 to any target when it dies.
#[test]
fn mudbutton_torchrunner_deals_3_on_death() {
    let mut g = two_player_game();
    let mud = g.add_card_to_battlefield(0, catalog::mudbutton_torchrunner());
    let life = g.players[1].life;
    // Sacrifice it to a free outlet to trigger the death damage at a player.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Player(1)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(mud)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the 1/1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mud).is_none(), "Torchrunner died");
    assert_eq!(g.players[1].life, life - 3, "death trigger dealt 3 to the player");
}

/// Llanowar Mentor discards a card to mint a mana-producing Elf token.
#[test]
fn llanowar_mentor_makes_mana_elf() {
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::llanowar_mentor());
    g.clear_sickness(mentor);
    let pitch = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mentor, ability_index: 0,
        target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Llanowar Mentor");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "a card was discarded");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Llanowar Elves" && c.is_token),
        "a 1/1 mana Elf token was created");
}
