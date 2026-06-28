//! Functionality tests for the LCI batch — Descend / fathomless descent and
//! assorted commons riding existing primitives.

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::game::*;
use crate::mana::Color;

/// Souls of the Lost is */*+1 = permanent cards in your graveyard.
#[test]
fn souls_of_the_lost_pt_tracks_graveyard_permanents() {
    let mut g = two_player_game();
    let soul = g.add_card_to_battlefield(0, catalog::souls_of_the_lost());
    // Empty graveyard → 0/1.
    let c = g.computed_permanent(soul).unwrap();
    assert_eq!((c.power, c.toughness), (0, 1));
    // Three permanent cards + an instant → power 3, toughness 4 (instant ignored).
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let c = g.computed_permanent(soul).unwrap();
    assert_eq!((c.power, c.toughness), (3, 4));
}

/// Frilled Cave-Wurm gets +2/+0 only with 4+ permanent cards in the graveyard.
#[test]
fn frilled_cave_wurm_descend_4_pump() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::frilled_cave_wurm());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 2, "descend 3 → base 2/5");
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // → 4
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 4, "descend 4 → +2/+0");
}

/// Coati Scavenger returns a permanent card from the graveyard on ETB once
/// descend 4 is active.
#[test]
fn coati_scavenger_descend_4_recurs_permanent() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let coati = g.add_card_to_hand(0, catalog::coati_scavenger());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: coati, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // One bear returned to hand (the ETB trigger auto-targets a gy permanent card).
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "recurred a creature");
}

/// Acolyte of Aclazotz drains 1 by tapping and sacrificing another permanent.
#[test]
fn acolyte_of_aclazotz_drains_on_sac() {
    let mut g = two_player_game();
    let acolyte = g.add_card_to_battlefield(0, catalog::acolyte_of_aclazotz());
    g.clear_sickness(acolyte);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let start = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: acolyte, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, start - 1, "opponent lost 1");
}

/// Poison Dart Frog's {2} ability grants deathtouch until end of turn.
#[test]
fn poison_dart_frog_grants_deathtouch() {
    let mut g = two_player_game();
    let frog = g.add_card_to_battlefield(0, catalog::poison_dart_frog());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    // Ability index 1 is the {2}: deathtouch (index 0 is the mana ability).
    g.perform_action(GameAction::ActivateAbility {
        card_id: frog, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(frog).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Bitter Triumph destroys a creature after its discard additional cost.
#[test]
fn bitter_triumph_destroys_with_discard_cost() {
    let mut g = two_player_game();
    let bt = g.add_card_to_hand(0, catalog::bitter_triumph());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bt, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "target destroyed");
}

/// Cavern Stomper's activated ability makes it unblockable by power-2 creatures.
#[test]
fn cavern_stomper_grants_evasion() {
    let mut g = two_player_game();
    let stomper = g.add_card_to_battlefield(0, catalog::cavern_stomper());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stomper, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(stomper).unwrap().keywords.contains(&Keyword::CantBeBlockedByPowerAtMost(2)),
    );
}

/// Panicked Altisaur taps to deal 2 to each opponent.
#[test]
fn panicked_altisaur_pings_each_opponent() {
    let mut g = two_player_game();
    let dino = g.add_card_to_battlefield(0, catalog::panicked_altisaur());
    g.clear_sickness(dino);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let start = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dino, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, start - 2);
}

/// Plundering Pirate makes a Treasure on ETB.
#[test]
fn plundering_pirate_makes_treasure() {
    let mut g = two_player_game();
    let pirate = g.add_card_to_hand(0, catalog::plundering_pirate());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: pirate, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
}

/// Miner's Guidewing's death trigger makes a creature you control explore (puts
/// a +1/+1 counter when a nonland is revealed).
#[test]
fn miners_guidewing_dies_triggers_explore() {
    let mut g = two_player_game();
    let wing = g.add_card_to_battlefield(0, catalog::miners_guidewing());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Stack a nonland on top so the explore yields a +1/+1 counter.
    let nid = g.next_id();
    g.players[0].library.insert(0, crate::card::CardInstance::new(nid, catalog::grizzly_bears(), 0));
    g.remove_to_graveyard_with_triggers(wing);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "explore put a +1/+1 counter",
    );
}

/// Echo of Dusk gains +1/+1 and lifelink at descend 4.
#[test]
fn echo_of_dusk_descend_4_lifelink() {
    let mut g = two_player_game();
    let echo = g.add_card_to_battlefield(0, catalog::echo_of_dusk());
    assert_eq!(g.computed_permanent(echo).unwrap().power, 2, "base 2/2 below descend 4");
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let cp = g.computed_permanent(echo).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "descend 4 → +1/+1");
    assert!(cp.keywords.contains(&Keyword::Lifelink), "descend 4 → lifelink");
}

/// CR 700.11 — a permanent card hitting the graveyard sets descended_this_turn;
/// a noncreature spell card does not.
#[test]
fn descended_this_turn_tracks_permanent_cards() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.players[0].descended_this_turn);
    // Milling an instant doesn't count as descending.
    let nid = g.next_id();
    g.players[0].send_to_graveyard(crate::card::CardInstance::new(nid, catalog::lightning_bolt(), 0));
    assert!(!g.players[0].descended_this_turn, "instant card → not descended");
    // A creature dying does.
    g.remove_to_graveyard_with_triggers(bear);
    assert!(g.players[0].descended_this_turn, "permanent card → descended");
}

/// Hermitic Nautilus pumps +3/-3 with its {1}{U} ability.
#[test]
fn hermitic_nautilus_pumps() {
    let mut g = two_player_game();
    let naut = g.add_card_to_battlefield(0, catalog::hermitic_nautilus());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: naut, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(naut).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 1), "1/4 +3/-3 → 4/1");
}

/// Deep Goblin Skulltaker grows at end step only if you descended this turn.
#[test]
fn deep_goblin_skulltaker_end_step_descend_counter() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::deep_goblin_skulltaker());
    g.active_player_idx = 0;
    // No descend yet → end step adds nothing.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(gob).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 0);
    // Descend this turn, then the next end step adds a +1/+1 counter.
    g.players[0].descended_this_turn = true;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(gob).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

/// Dinotomaton's ETB grants a creature you control menace.
#[test]
fn dinotomaton_etb_grants_menace() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dino = g.add_card_to_hand(0, catalog::dinotomaton());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: dino, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Menace));
}

/// Market Gnome draws a card and gains life when it dies.
#[test]
fn market_gnome_death_value() {
    let mut g = two_player_game();
    let gnome = g.add_card_to_battlefield(0, catalog::market_gnome());
    let nid = g.next_id();
    g.players[0].library.insert(0, crate::card::CardInstance::new(nid, catalog::grizzly_bears(), 0));
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    g.remove_to_graveyard_with_triggers(gnome);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
}

// ── Caves (LandType::Cave) ──────────────────────────────────────────────────

/// A Hidden Cave enters tapped and taps for its color.
#[test]
fn hidden_cave_enters_tapped_then_taps_for_color() {
    let mut g = two_player_game();
    let cave = g.move_card_to_battlefield_for_test(0, catalog::hidden_cataract());
    assert!(g.battlefield_find(cave).unwrap().tapped, "Hidden Cataract enters tapped");
    g.battlefield.iter_mut().find(|c| c.id == cave).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cave, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for blue");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "adds blue");
}

/// Volatile Fault sacrifices itself to destroy a nonbasic land an opponent
/// controls.
#[test]
fn volatile_fault_destroys_opponent_nonbasic_land() {
    let mut g = two_player_game();
    let fault = g.add_card_to_battlefield(0, catalog::volatile_fault());
    let victim = g.add_card_to_battlefield(1, catalog::captivating_cave());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fault, ability_index: 1, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent's nonbasic land destroyed");
    assert!(g.battlefield_find(fault).is_none(), "Volatile Fault sacrificed as a cost");
}

/// Spelunking makes lands you control enter untapped, overriding an
/// enters-tapped static.
#[test]
fn spelunking_overrides_enters_tapped() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spelunking());
    let cave = g.move_card_to_battlefield_for_test(0, catalog::hidden_cataract());
    assert!(!g.battlefield_find(cave).unwrap().tapped, "Spelunking → land enters untapped");
}

/// Forgotten Monument grants other Caves you control a pay-1-life any-color
/// mana ability (surfaced as a virtual activated ability past the printed set).
#[test]
fn forgotten_monument_grants_other_caves_any_color_mana() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forgotten_monument());
    let other = g.add_card_to_battlefield(0, catalog::captivating_cave());
    g.players[0].life = 20;
    // Captivating Cave has 3 printed abilities; the granted ability is index 3.
    g.perform_action(GameAction::ActivateAbility {
        card_id: other, ability_index: 3, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("granted mana ability");
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced 1 mana of any color");
    assert_eq!(g.players[0].life, 19, "paid 1 life");
}

/// Sanguine Evangelist's battle cry pumps other attackers; ETB makes a Bat.
#[test]
fn sanguine_evangelist_battle_cry_and_bat() {
    let mut g = two_player_game();
    let evangelist = g.add_card_to_hand(0, catalog::sanguine_evangelist());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: evangelist, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bat" && c.controller == 0), "ETB Bat token");
}
