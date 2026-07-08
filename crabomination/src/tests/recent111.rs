//! Functionality tests for `catalog::sets::decks::recent111` — Merfolk /
//! Elves tribal, artifact payoffs, and green engines.

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Silvergill Douser shrinks power by your Merfolk/Faerie count.
#[test]
fn silvergill_douser_shrinks_by_tribe_count() {
    let mut g = two_player_game();
    let douser = g.add_card_to_battlefield(0, catalog::silvergill_douser());
    g.add_card_to_battlefield(0, catalog::kumenas_speaker()); // Merfolk
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(douser);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: douser, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    })
    .expect("douse");
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 2), "-2/-0 from two Merfolk");
}

/// Merfolk Sovereign pumps other Merfolk and grants unblockable.
#[test]
fn merfolk_sovereign_lord_and_unblockable() {
    let mut g = two_player_game();
    let sovereign = g.add_card_to_battlefield(0, catalog::merfolk_sovereign());
    let speaker = g.add_card_to_battlefield(0, catalog::kumenas_speaker());
    g.clear_sickness(sovereign);
    let cp = g.computed_permanent(speaker).unwrap();
    // 1/1 base + lord +1/+1 + Speaker's own another-Merfolk +1/+1.
    assert_eq!((cp.power, cp.toughness), (3, 3), "lord pump stacks with self-pump");
    assert_eq!(g.computed_permanent(sovereign).unwrap().power, 2, "not self-pumped");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sovereign, ability_index: 0, target: Some(Target::Permanent(speaker)),
        additional_targets: vec![], x_value: None,
    })
    .expect("grant");
    drain_stack(&mut g);
    assert!(g.computed_permanent(speaker).unwrap().keywords.contains(&crate::card::Keyword::Unblockable));
}

/// Tidebinder Mage taps a red/green creature and locks its next untap.
#[test]
fn tidebinder_mage_taps_and_locks() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let mage = g.add_card_to_battlefield(0, catalog::tidebinder_mage());
    g.fire_self_etb_triggers(mage, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(dragon).unwrap();
    assert!(c.tapped, "tapped by the ETB");
    assert!(c.skip_next_untap, "untap-locked");
}

/// Master of Waves mints Elementals equal to blue devotion, pumped by its
/// own lord static (1/0 → 2/1).
#[test]
fn master_of_waves_devotion_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tempest_djinn()); // {U}{U}{U} → devotion 3
    let master = g.add_card_to_battlefield(0, catalog::master_of_waves());
    g.fire_self_etb_triggers(master, 0);
    drain_stack(&mut g);
    let tokens: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Elemental")
        .map(|c| c.id)
        .collect();
    assert_eq!(tokens.len(), 4, "devotion 3 + Master's own {{U}}");
    let cp = g.computed_permanent(tokens[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 1), "lord-pumped 1/0");
}

/// Loaming Shaman shuffles the target player's graveyard into their library.
#[test]
fn loaming_shaman_recycles_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let shaman = g.add_card_to_battlefield(0, catalog::loaming_shaman());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    g.fire_self_etb_triggers(shaman, 0);
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty(), "graveyard shuffled away");
    assert_eq!(g.players[1].library.len(), 2);
}

/// Defense of the Heart fires only against three opposing creatures.
#[test]
fn defense_of_the_heart_tutors_two() {
    let mut g = two_player_game();
    let heart = g.add_card_to_battlefield(0, catalog::defense_of_the_heart());
    let big1 = g.add_card_to_library(0, catalog::grizzly_bears());
    let big2 = g.add_card_to_library(0, catalog::savannah_lions());
    g.active_player_idx = 0;
    g.fire_step_triggers(crate::game::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(heart).is_some(), "no trigger below three creatures");
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(big1)),
        DecisionAnswer::Search(Some(big2)),
    ]));
    g.fire_step_triggers(crate::game::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(heart).is_none(), "sacrificed on trigger");
    assert!(g.battlefield_find(big1).is_some() && g.battlefield_find(big2).is_some());
}

/// Leaf-Crowned Visionary draws off an Elf cast when {G} is paid.
#[test]
fn leaf_crowned_visionary_elf_cantrip() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leaf_crowned_visionary());
    g.add_card_to_library(0, catalog::forest());
    let elf = g.add_card_to_hand(0, catalog::canopy_tactician());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Elf");
    drain_stack(&mut g);
    // -1 (cast) +1 (paid {G} draw) = unchanged.
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "paid {{G}}, drew a card");
}

/// Copperhorn Scout untaps the rest of the team on attack.
#[test]
fn copperhorn_scout_untaps_team() {
    let mut g = two_player_game();
    let scout = g.add_card_to_battlefield(0, catalog::copperhorn_scout());
    let mate = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mate).unwrap().tapped = true;
    g.clear_sickness(scout);
    g.active_player_idx = 0;
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: scout,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mate).unwrap().tapped, "untapped by the attack trigger");
}

/// Genesis Wave deploys permanents with MV ≤ X and bins the rest.
#[test]
fn genesis_wave_deploys_cheap_permanents() {
    let mut g = two_player_game();
    // Top 3 (X=3): bear (MV 2, deploy), bolt (nonpermanent, gy), djinn (MV 3, deploy).
    let djinn = g.add_card_to_library(0, catalog::tempest_djinn());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let wave = g.add_card_to_hand(0, catalog::genesis_wave());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(3);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: wave, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("cast X=3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert!(g.battlefield_find(djinn).is_some());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "instant to the graveyard");
}

/// Harald digs five for an Elf/Warrior card.
#[test]
fn harald_digs_for_an_elf() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let elf = g.add_card_to_library(0, catalog::canopy_tactician()); // top
    let harald = g.add_card_to_battlefield(0, catalog::harald_king_of_skemfar());
    g.fire_self_etb_triggers(harald, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == elf), "Elf picked to hand");
}

/// Skemfar Shadowsage drains for the largest shared-tribe count.
#[test]
fn skemfar_shadowsage_drains_by_tribe() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::canopy_tactician()); // Elf
    g.add_card_to_battlefield(0, catalog::leaf_crowned_visionary()); // Elf
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // Bear
    let sage = g.add_card_to_battlefield(0, catalog::skemfar_shadowsage()); // Elf
    let life1 = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.fire_self_etb_triggers(sage, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 3, "three Elves in common");
}

/// Rhonas can't attack alone but wakes up next to a power-4 creature.
#[test]
fn rhonas_needs_a_big_friend() {
    let mut g = two_player_game();
    let rhonas = g.add_card_to_battlefield(0, catalog::rhonas_the_indomitable());
    g.clear_sickness(rhonas);
    g.active_player_idx = 0;
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rhonas,
        target: AttackTarget::Player(1),
    }]));
    assert!(err.is_err(), "no other power-4 creature → can't attack");
    g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rhonas,
        target: AttackTarget::Player(1),
    }]))
    .expect("attacks with a power-4 friend");
}

/// Oath of Nissa digs three for a creature/land/planeswalker.
#[test]
fn oath_of_nissa_digs_three() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::counterspell());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // top
    let oath = g.add_card_to_battlefield(0, catalog::oath_of_nissa());
    g.fire_self_etb_triggers(oath, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Trash for Treasure eats an artifact to reanimate a bigger one.
#[test]
fn trash_for_treasure_swaps_artifacts() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::chrome_mox());
    let big = g.add_card_to_graveyard(0, catalog::metalwork_colossus());
    let tft = g.add_card_to_hand(0, catalog::trash_for_treasure());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tft, target: Some(Target::Permanent(big)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "artifact sacrificed as a cost");
    assert!(g.battlefield_find(big).is_some(), "Colossus reanimated");
}

/// Metalwork Colossus discounts by noncreature-artifact MV and self-recurs.
#[test]
fn metalwork_colossus_discount_and_recursion() {
    let mut g = two_player_game();
    // Two Shriekhorns ({1} each) + a Darksteel Garrison ({2}) = MV 4 → {7}.
    g.add_card_to_battlefield(0, catalog::shriekhorn());
    g.add_card_to_battlefield(0, catalog::shriekhorn());
    g.add_card_to_battlefield(0, catalog::darksteel_garrison());
    let colossus = g.add_card_to_hand(0, catalog::metalwork_colossus());
    g.players[0].mana_pool.add_colorless(7);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: colossus, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {7}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(colossus).is_some());
    // Recursion: dump it in the graveyard, sac two artifacts to return it.
    let evs = g.remove_to_graveyard_with_triggers(colossus);
    g.dispatch_triggers_for_events(&evs);
    g.perform_action(GameAction::ActivateAbility {
        card_id: colossus, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("graveyard recursion");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == colossus), "back to hand");
}

/// Jhoira's Familiar discounts historic (artifact/legendary/Saga) spells.
#[test]
fn jhoiras_familiar_discounts_historic() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jhoiras_familiar());
    let mox = g.add_card_to_hand(0, catalog::shriekhorn()); // {1} artifact → free
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: mox, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{1} artifact is free with the Familiar");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mox).is_some());
}

/// Grand Architect pumps blue creatures and funds artifacts off tapping one.
#[test]
fn grand_architect_pumps_and_funds() {
    let mut g = two_player_game();
    let architect = g.add_card_to_battlefield(0, catalog::grand_architect());
    let djinn = g.add_card_to_battlefield(0, catalog::tempest_djinn());
    let cp = g.computed_permanent(djinn).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 5), "blue lord pump (0/4 base)");
    // Tap the Djinn for {C}{C} restricted to artifacts.
    g.clear_sickness(djinn);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: architect, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tap a blue creature for {C}{C}");
    assert!(g.battlefield_find(djinn).unwrap().tapped, "the blue creature tapped");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "restricted {{C}}{{C}} floats");
    let horn = g.add_card_to_hand(0, catalog::shriekhorn());
    g.step = crate::game::TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: horn, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("artifact funded by the restricted mana");
}
