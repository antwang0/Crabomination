//! Functionality tests for `catalog::sets::decks::recent179` (FDN batch).

use crabomination::card::{ArtifactSubtype, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Twinblade Blessing grants the enchanted creature double strike.
#[test]
fn twinblade_blessing_grants_double_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::twinblade_blessing());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Twinblade Blessing");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "enchanted creature has double strike"
    );
}

/// Tragic Banshee gives -1/-1 with no death, -13/-13 once a creature has died.
#[test]
fn tragic_banshee_morbid_scales() {
    let mut g = two_player_game();
    // No creature died yet → -1/-1 to a 5/5.
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    g.move_card_to_battlefield_for_test(0, catalog::tragic_banshee());
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "no morbid → -1/-1");
}

/// With a creature already dead this turn, Tragic Banshee applies -13/-13.
#[test]
fn tragic_banshee_morbid_full() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant());
    // Record a death this turn.
    let doomed = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(doomed);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: doomed }]);
    drain_stack(&mut g);
    g.move_card_to_battlefield_for_test(0, catalog::tragic_banshee());
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "morbid -13/-13 killed the 3/3");
}

/// Midnight Snack makes a Food at end step if you attacked, and drains for the
/// life you gained this turn.
#[test]
fn midnight_snack_food_and_drain() {
    let mut g = two_player_game();
    let snack = g.add_card_to_battlefield(0, catalog::midnight_snack());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    let foods = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Food))
        .count();
    assert_eq!(foods, 1, "Raid made a Food at end step");
    // Now gain 4 life and sac Midnight Snack to drain the opponent for 4.
    g.adjust_life(0, 4);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: snack,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 4, "drained the life gained this turn");
}

/// Uncharted Voyage tucks a creature into its owner's library (bottom default).
#[test]
fn uncharted_voyage_tucks_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::uncharted_voyage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.add_card_to_library(0, catalog::grizzly_bears()); // surveil fodder
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Uncharted Voyage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature left the battlefield");
    assert!(g.players[1].library.iter().any(|c| c.id == foe), "went to owner's library");
}

/// Raise the Past returns only the MV≤2 creatures from your graveyard.
#[test]
fn raise_the_past_returns_small_creatures() {
    let mut g = two_player_game();
    let small = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let big = g.add_card_to_graveyard(0, catalog::hill_giant()); // MV 4
    let spell = g.add_card_to_hand(0, catalog::raise_the_past());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Raise the Past");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_some(), "small creature returned");
    assert!(g.battlefield_find(big).is_none(), "MV4 creature stayed in graveyard");
}

/// Sylvan Scavenging's end-step modal resolves (counter or Raccoon).
#[test]
fn sylvan_scavenging_end_step_modal() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sylvan_scavenging());
    let beater = g.add_card_to_battlefield(0, catalog::hill_giant()); // power 3
    let before_counters =
        *g.battlefield_find(beater).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0);
    let before = g.battlefield.len();
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    let after_counters =
        *g.battlefield_find(beater).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0);
    let raccoon = g
        .battlefield
        .iter()
        .any(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Raccoon));
    assert!(
        after_counters > before_counters || raccoon || g.battlefield.len() > before,
        "a mode resolved (counter placed or Raccoon made)"
    );
}

/// Ravenous Amulet stores soul counters on sac-to-draw, then drains for them.
#[test]
fn ravenous_amulet_stores_and_drains() {
    let mut g = two_player_game();
    let amulet = g.add_card_to_battlefield(0, catalog::ravenous_amulet());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Sac the bear → draw a card + a charge counter.
    g.perform_action(GameAction::ActivateAbility {
        card_id: amulet,
        ability_index: 0,
        target: Some(Target::Permanent(fodder)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate sac-to-draw");
    drain_stack(&mut g);
    let counters =
        *g.battlefield_find(amulet).unwrap().counters.get(&CounterType::Charge).unwrap_or(&0);
    assert_eq!(counters, 1, "stored a soul counter");
    // Untap (a fresh turn) then sac the amulet → opponent loses 1 (one counter).
    g.battlefield_find_mut(amulet).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(4);
    let opp = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: amulet,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "drained for the stored soul counter");
}

/// Zul Ashur lets you cast a Zombie from your graveyard this turn.
#[test]
fn zul_ashur_grants_graveyard_zombie_cast() {
    let mut g = two_player_game();
    let zul = g.add_card_to_battlefield(0, catalog::zul_ashur_lich_lord());
    g.clear_sickness(zul);
    // A Zombie creature card in the graveyard.
    let mut zombie = catalog::grizzly_bears();
    zombie.name = "Rotting Ghoul";
    zombie.subtypes.creature_types = vec![CreatureType::Zombie];
    let ghoul = g.add_card_to_graveyard(0, zombie);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: zul,
        ability_index: 0,
        target: Some(Target::Permanent(ghoul)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate graveyard-cast grant");
    drain_stack(&mut g);
    let granted = g.players[0]
        .graveyard
        .iter()
        .find(|c| c.id == ghoul)
        .map(|c| c.may_play_until.is_some())
        .unwrap_or(false);
    assert!(granted, "the Zombie may now be cast from the graveyard");
}

/// Twinflame Tyrant doubles damage your sources deal to opponents.
#[test]
fn twinflame_tyrant_doubles_damage_to_opponents() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::twinflame_tyrant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // 3 to any target
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let opp = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Lightning Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 6, "3 damage doubled to 6");
}

/// High Fae Trickster lets you cast a sorcery at instant speed.
#[test]
fn high_fae_trickster_grants_flash_to_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::high_fae_trickster());
    let sorc = g.add_card_to_hand(0, catalog::divination());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    // It's the opponent's turn (instant speed only) — the sorcery is castable.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    let ok = g
        .perform_action(GameAction::CastSpell {
            card_id: sorc,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok();
    assert!(ok, "sorcery cast at instant speed via granted flash");
}

/// Electroduplicate makes a haste token copy of your creature.
#[test]
fn electroduplicate_copies_your_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::electroduplicate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Electroduplicate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), before + 1, "made a token copy");
    let token_id = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .map(|c| c.id)
        .expect("token copy exists");
    assert!(
        g.computed_permanent(token_id).unwrap().keywords.contains(&Keyword::Haste),
        "copy has haste"
    );
}

/// Fear of Falling shrinks and grounds a blocker when it attacks.
#[test]
fn fear_of_falling_debuffs_on_attack() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(0, catalog::fear_of_falling());
    g.clear_sickness(flyer);
    let mut blocker = catalog::grizzly_bears();
    blocker.keywords.push(Keyword::Flying);
    let foe = g.add_card_to_battlefield(1, blocker); // 2/2 flyer
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: flyer,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!(cp.power, 0, "-2/-0 applied");
    assert!(!cp.keywords.contains(&Keyword::Flying), "lost flying");
}
